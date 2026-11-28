// SPDX-License-Identifier: GPL-3.0-or-later
//! The simulated physical world: storage capacitor, charger, internal
//! discharge load, output bridge, patient, pads, cancel button, battery and
//! the message wire between the two controllers.
//!
//! Both controllers see this one plant, each through its own view
//! ([`crate::hw::MainView`] and [`crate::hw::SupervisorView`]) with its own
//! sensors and its own clock. Nothing a controller does reaches the other
//! controller except through the plant's physics or the message wire.
//!
//! **The shock is a process in time, not an event** (D-59, review R-1).
//! While anything in the shock path is moving, the plant steps in
//! [`PlantConfig::shock_step_us`] (10 µs) instead of 1 ms: the interlock
//! rail rising, the patient relay's contacts breaking, travelling and
//! bouncing, both front ends reading open while the pads are off them, the
//! sequence latch ([`aed_core::interlock::Latch`]), each phase discharging
//! the capacitor continuously into the patient (the same exponential as the
//! aed-waveform model, integrated step by step), the hardware time cap and
//! over-current trip, the relay returning, the front ends recovering, the
//! lockout, and the permit-duration cap. The consent lines are sampled at
//! every step: withdrawn before the latch, nothing happens; after it, the
//! pulse completes.
//!
//! SIMULATION ONLY. Every electrical figure is a model figure from simple
//! arithmetic. See the crate README for what is not modelled.

use std::collections::VecDeque;

use aed_core::interlock::{
    charger_may_run, discharge_path_active, Latch, LatchInputs, LatchTiming,
};
use aed_core::sequencer::{
    Sequencer, SequencerJob, SequencerLines, SequencerOutputs, SequencerTiming,
};
use aed_core::{
    BatteryStatus, BridgeTestOutcome, BridgeTestStep, Config, HoldReadback, MainStatus,
    PadCartridge, Prompt, RelayPosition, SequenceEnd, SequenceStatus, SupervisorReport,
};
use aed_rhythm::synth::{Rhythm, Synth, Xorshift32};
use aed_types::{EcgSample, ImpedanceSample, ShockPlan, Volts};
use aed_waveform::{discharge, DischargeResult, WaveformConfig};

/// A sensor with a gain error, an offset error and an optional stuck-at fault.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorModel {
    /// Multiplies the true value (1.0 means no gain error).
    pub gain: f64,
    /// Added after the gain, in the sensor's unit.
    pub offset: f64,
    /// If set, the sensor reads this value whatever the truth is.
    pub stuck_at: Option<f64>,
}

impl SensorModel {
    /// A perfect sensor.
    #[must_use]
    pub const fn ideal() -> Self {
        SensorModel {
            gain: 1.0,
            offset: 0.0,
            stuck_at: None,
        }
    }

    /// What the sensor reports for a true value.
    #[must_use]
    pub fn read(&self, truth: f64) -> f64 {
        self.stuck_at.unwrap_or(truth * self.gain + self.offset)
    }
}

impl Default for SensorModel {
    fn default() -> Self {
        Self::ideal()
    }
}

/// The hardware timings and thresholds of design revision 0.3 that the
/// firmware relies on, for ONE simulated unit. Each has a tolerance band;
/// [`HwTiming::NOMINAL`], [`HwTiming::EARLIEST`] (every figure at the low
/// end), [`HwTiming::LATEST`] (high end) and [`HwTiming::ADVERSE`] (the
/// worst unit for a legitimate sequence: every delay the firmware waits on
/// at its longest, every window that limits the firmware at its shortest).
/// A firmware wait shorter than the hardware's worst case shows up as a
/// refused start in a test at the corner where it matters (the class of bug
/// integration issue I-9 found).
///
/// Sources, all SIMULATED or CALCULATED, none measured:
/// `hardware/aed-main/sim/logic-relay/notes.md` (LR) §1.1–§1.3 and §3.2,
/// `sim/rev3/notes.md` (R3) §2, `docs/HARDWARE_DESIGN.md` v0.3 §B13.3 and
/// the change order `capture-notes/rev3-changes.md` (C-2, E-1, G-3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HwTiming {
    /// `RAIL_READY` after the rail reads armed, µs. 21.56–26.67 ms (LR §1.3).
    pub rail_ready_us: u32,
    /// The relay-transferred timer `XFER` after the rest contact last left
    /// rest, µs. 10.42–12.17 ms (LR §1.3; restarts on a bounce with D-14).
    pub xfer_us: u32,
    /// The coil start window after `K_EN`, µs: a sequence may start only
    /// inside it. 43.40–51.20 ms (LR §1.3).
    pub coil_window_us: u32,
    /// The coil cap after `K_EN`, µs: the coil drive is cut then.
    /// 83.65–96.38 ms (LR §1.3).
    pub coil_cap_us: u32,
    /// The test-relay timer `BTOK` after `BTEST_EN`, µs. 4.74–5.52 ms with
    /// change D-16 (LR §1.3).
    pub btok_us: u32,
    /// The permit detector keeps "consent" this long after the supervisor's
    /// clock stops, µs (and a single edge or pulse gives up to this much).
    /// ≤ 7.50 ms with change D-1 (LR §1.2).
    pub permit_dropout_us: u32,
    /// The coil reservoir is full this long after `K_SUPPLY_EN`, µs.
    /// 74–99 ms (LR §3.2).
    pub reservoir_fill_us: u32,
    /// Relay contacts transfer this long after `K_EN` with the reservoir
    /// full, µs. 15.9 ms in every battery case (LR §3.2, relay mechanics
    /// estimated).
    pub relay_transfer_us: u32,
    /// ... and with the reservoir NOT yet full (no pre-charge), µs.
    /// 23.7–31.8 ms (LR §3.1).
    pub relay_transfer_slow_us: u32,
    /// A discharge path reads "held" this long after its hold clock starts,
    /// µs. 19.2–32.9 ms (R3 §2).
    pub hold_engage_us: u32,
    /// ... and "not held" this long after the clock stops (low), µs.
    /// 1.4–22.2 ms (R3 §2).
    pub hold_release_us: u32,
    /// The supervisor-side over-voltage latch trips at this capacitor
    /// voltage, V. 2,073–2,142 V (change C-2, calculated).
    pub ovp_trip_v: f64,
    /// `OVP_RST` must stay high at least this long to reset the latch, µs.
    /// About 100 µs through the 220 Ω series resistor (controller capture).
    pub ovp_reset_min_us: u32,
    /// The charger delivers nothing for this long after it is (re)started,
    /// µs (WP-E C7: ≥ 15 ms allowed by the firmware).
    pub charger_start_us: u32,
    /// The charger's own last-resort primary-side trip, V: it ends the
    /// charge and reports `CHG_DONE_N`. 2,116–2,316 V (change E-1).
    pub charger_trip_v: f64,
}

impl HwTiming {
    /// The nominal unit.
    pub const NOMINAL: HwTiming = HwTiming {
        rail_ready_us: 23_920,
        xfer_us: 11_300,
        coil_window_us: 48_910,
        coil_cap_us: 92_520,
        btok_us: 5_130,
        permit_dropout_us: 6_630,
        reservoir_fill_us: 86_000,
        relay_transfer_us: 15_900,
        relay_transfer_slow_us: 27_300,
        hold_engage_us: 27_800,
        hold_release_us: 1_700,
        ovp_trip_v: 2_107.0,
        ovp_reset_min_us: 100,
        charger_start_us: 10_000,
        charger_trip_v: 2_216.0,
    };
    /// Every figure at the low end of its band.
    pub const EARLIEST: HwTiming = HwTiming {
        rail_ready_us: 21_560,
        xfer_us: 10_420,
        coil_window_us: 43_400,
        coil_cap_us: 83_650,
        btok_us: 4_740,
        permit_dropout_us: 4_630,
        reservoir_fill_us: 74_000,
        relay_transfer_us: 15_900,
        relay_transfer_slow_us: 23_700,
        hold_engage_us: 19_200,
        hold_release_us: 1_400,
        ovp_trip_v: 2_073.0,
        ovp_reset_min_us: 100,
        charger_start_us: 5_000,
        charger_trip_v: 2_116.0,
    };
    /// Every figure at the high end of its band.
    pub const LATEST: HwTiming = HwTiming {
        rail_ready_us: 26_670,
        xfer_us: 12_170,
        coil_window_us: 51_200,
        coil_cap_us: 96_380,
        btok_us: 5_520,
        permit_dropout_us: 7_500,
        reservoir_fill_us: 99_000,
        relay_transfer_us: 15_900,
        relay_transfer_slow_us: 31_800,
        hold_engage_us: 32_900,
        hold_release_us: 22_200,
        ovp_trip_v: 2_142.0,
        ovp_reset_min_us: 100,
        charger_start_us: 15_000,
        charger_trip_v: 2_316.0,
    };
    /// The worst unit for a legitimate sequence: what the firmware waits
    /// for at its slowest, what limits the firmware at its shortest.
    pub const ADVERSE: HwTiming = HwTiming {
        coil_window_us: 43_400,
        coil_cap_us: 83_650,
        ovp_trip_v: 2_073.0,
        charger_trip_v: 2_116.0,
        ..HwTiming::LATEST
    };
}

/// A unit at one corner of the tolerance bands (see [`HwTiming`] and
/// [`LatchTiming`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwCorner {
    /// Nominal.
    Nominal,
    /// Every figure at the low end.
    Earliest,
    /// Every figure at the high end.
    Latest,
    /// Worst for a legitimate sequence: delays long, windows and caps short
    /// (the caps at their LOWEST, so they come closest to a planned pulse),
    /// re-arm at its longest.
    Adverse,
}

impl HwCorner {
    /// Every corner.
    pub const ALL: [HwCorner; 4] = [
        HwCorner::Nominal,
        HwCorner::Earliest,
        HwCorner::Latest,
        HwCorner::Adverse,
    ];
}

/// The physical constants and imperfections of the simulated device.
///
/// Every default is a provisional modelling choice, not a measured value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlantConfig {
    /// The shock design (capacitor size, internal resistance, charge voltages).
    pub waveform: WaveformConfig,
    /// Power the charger puts into the capacitor, watts. Default 30 W: a
    /// full adult charge (190 J at 1,950 V) then takes about 6.3 s, inside
    /// the "under 8 seconds" requirement in ARCHITECTURE.md. Model choice.
    pub charger_watts: f64,
    /// Highest current the charger can push at low voltage, amperes.
    /// Default 0.5 A (model choice; stops the constant-power law dividing by zero).
    pub charger_max_current_a: f64,
    /// Highest voltage the charger circuit can physically reach if its
    /// regulation fails (only used by the "runaway" fault). Default 2,600 V.
    pub charger_runaway_volts: f64,
    /// Resistance of discharge path A (let go by the main controller),
    /// ohms. Default 4,000 Ω, and the same for path B: together 2,000 Ω,
    /// which with 100 µF empties the capacitor with a 0.2 s time constant,
    /// so 1,950 V falls below 50 V in about 0.75 s, as before the split;
    /// one path alone takes twice as long. Model choice (the hardware
    /// design proposes 4.5 kΩ per path, HARDWARE_DESIGN.md §B5.3).
    pub path_a_ohms: f64,
    /// Resistance of discharge path B (let go by the supervisor), ohms.
    pub path_b_ohms: f64,
    /// The capacitor's own slow leakage (bleeder), ohms. Default 50 MΩ
    /// (time constant about 80 minutes). Model choice.
    pub bleed_ohms: f64,
    /// If true (the default), the charger is switched off in hardware
    /// whenever the discharge path is active: the hardware requirement
    /// `aed_core::interlock::charger_may_run`. Set false only to show what
    /// happens without that rule (finding F-3: a frozen main controller then
    /// leaves the charger feeding the discharge load).
    pub charger_gated_by_discharge: bool,
    /// Regulation error of the charger: it charges to this multiple of the
    /// voltage it is asked for (1.0: exact). Default 1.0 (model choice). Used
    /// to test how wrong the capacitor voltage can be when a shock still fires.
    pub charger_set_point_gain: f64,
    /// Delay of each message on the wire between the controllers, ms. Default 1 ms.
    pub link_delay_ms: u32,
    /// Chance that any one message is lost (0.0 to 1.0). Default 0.
    pub link_drop_probability: f64,
    /// Main controller clock error, parts per million (positive runs fast).
    pub main_clock_ppm: i32,
    /// Supervisor clock error, parts per million (positive runs fast).
    pub supervisor_clock_ppm: i32,
    /// Main controller's capacitor-voltage sensor.
    pub main_voltage_sensor: SensorModel,
    /// Supervisor's capacitor-voltage sensor.
    pub supervisor_voltage_sensor: SensorModel,
    /// Main controller's pad-resistance measurement.
    pub main_resistance_sensor: SensorModel,
    /// Supervisor's pad-resistance measurement.
    pub supervisor_resistance_sensor: SensorModel,
    /// What a resistance measurement reads when the pads are off (open circuit), ohms.
    pub open_circuit_reading: u16,
    /// How long the heart-signal input reads flat after a shock while the
    /// front end recovers, ms. Default 1,500 ms (model choice).
    pub post_shock_blanking_ms: u32,
    /// Pad-impedance (compression sensing) front end: random noise, ohms
    /// RMS. Default 5 mΩ, inside the 10 mΩ the detector assumes (aed-cpr
    /// `FrontEndAssumptions`). Model choice.
    pub impedance_noise_ohms: f64,
    /// How long the impedance front end reads saturated after a shock, ms.
    /// Default 300 ms. Model choice, not measured.
    pub impedance_shock_saturation_ms: u32,
    /// After that, the impedance reads this much high and recovers
    /// exponentially (pad polarisation after a shock), ohms. Default 3 Ω.
    /// Model choice, not measured.
    pub impedance_shock_offset_ohms: f64,
    /// Time constant of that recovery, ms. Default 1,500 ms. Model choice.
    pub impedance_shock_recovery_ms: f64,

    // ----- the shock as a process in time (D-59) ---------------------------
    /// Time step while anything in the shock path is moving, µs. Default
    /// 10 µs: phase 1 at 25 Ω has a 3 ms time constant, so each step moves
    /// the capacitor by about 0.3 %. The rest of the simulation steps in 1 ms.
    pub shock_step_us: u32,
    /// The hardware's time cap, permit-duration cap and re-arm time.
    /// Default: the proposed figures ([`LatchTiming::PROPOSED`]).
    pub latch_timing: LatchTiming,
    /// True (default): the hardware latches the sequence at the first
    /// conducting moment (D-59). False: the hardware of the first design
    /// (review R-1), where removing any consent cuts the drive at once,
    /// mid-pulse. Only to demonstrate the old problem.
    pub sequence_latch: bool,
    /// From the rail rising to the relay coil and drivers being powered, µs.
    /// Default 1 ms (HARDWARE_DESIGN.md §B8.4: ≤ 2 ms). Model choice.
    pub rail_rise_us: u32,
    /// The rest contact opens this long before the bridge contact first
    /// closes, µs. Default 1 ms. Model choice; no relay is chosen (D-64).
    pub relay_break_lead_us: u32,
    /// How long the bridge contact bounces after first closing, µs.
    /// Default 1 ms. Model choice.
    pub relay_bounce_us: u32,
    /// From the coil released to the front-end contact closing again, µs.
    /// Default 5 ms (§B8.4: ≤ 10 ms, unverified). Model choice.
    pub relay_return_us: u32,
    /// The over-current trip, amperes. Default 80 A (v0.3 change G-1; band
    /// 74–86 A).
    pub over_current_a: f64,
    /// The fast idle-current comparator, amperes (v0.3 G-3: 2.6–3.5 A).
    pub idle_fast_a: f64,
    /// The hardware's own limit for the test permit (`LOWV_OK`): the test
    /// rail rises only below this capacitor voltage. Default 60 V (§B14.1).
    pub test_voltage_limit_v: f64,
    /// The bridge test load, ohms. Default 200 Ω (§B14.4).
    pub test_load_ohms: f64,
    /// The design v0.3 hardware timings (see [`HwTiming`]). Default nominal.
    pub hw: HwTiming,
    /// The firmware configuration the main controller's board layer (the
    /// delivery sequencer) runs with. The simulator sets it to the
    /// controllers' configuration.
    pub firmware: Config,
    /// The battery is cold and nearly empty: a full-power capacitor charge
    /// overlapping the coil reservoir's pre-charge browns the rails out
    /// (patient-interface capture: about 3.7 V). Default false.
    pub battery_weak: bool,
    /// The over-voltage latch powers up already tripped (its safe state).
    /// Default false.
    pub ovp_powers_up_tripped: bool,
    /// After the relay returns, both pad-resistance readings read open for
    /// this long while the front ends recover, ms. Default 300 ms (the same
    /// as the impedance saturation). Model choice, not measured.
    pub pad_reading_recovery_ms: u32,
    /// Each controller's voltage converter delivers a new reading every
    /// this many ms (sample and hold). Default 1 (a fresh reading every
    /// step). The review quotes 8 ms for the converter in the hardware
    /// design (AD7171, 125 samples/s).
    pub voltage_sample_ms: u32,
    /// Each controller's pad-resistance reading is refreshed every this many
    /// ms. Default 1. No figure is known for the chosen parts.
    pub pad_sample_ms: u32,
    /// Offset of the sampling grid, ms (to sweep where the samples fall
    /// relative to a shock).
    pub sample_phase_ms: u32,
    /// Half-period of a toggling consent line or phase timer fault, µs.
    /// Default 500 µs (a 1 kHz square wave).
    pub toggle_half_period_us: u32,
}

impl PlantConfig {
    /// This configuration with the hardware at `corner` of its bands.
    #[must_use]
    pub fn at_corner(self, corner: HwCorner) -> Self {
        let (hw, latch) = match corner {
            HwCorner::Nominal => (HwTiming::NOMINAL, LatchTiming::PROPOSED),
            HwCorner::Earliest => (HwTiming::EARLIEST, LatchTiming::EARLIEST),
            HwCorner::Latest => (HwTiming::LATEST, LatchTiming::LATEST),
            HwCorner::Adverse => (
                HwTiming::ADVERSE,
                LatchTiming {
                    rearm_us: LatchTiming::LATEST.rearm_us,
                    ..LatchTiming::EARLIEST
                },
            ),
        };
        PlantConfig {
            hw,
            latch_timing: latch,
            ..self
        }
    }
}

impl Default for PlantConfig {
    fn default() -> Self {
        PlantConfig {
            waveform: WaveformConfig::default(),
            charger_watts: 30.0,
            charger_max_current_a: 0.5,
            charger_runaway_volts: 2_600.0,
            path_a_ohms: 4_000.0,
            path_b_ohms: 4_000.0,
            bleed_ohms: 50.0e6,
            charger_gated_by_discharge: true,
            charger_set_point_gain: 1.0,
            link_delay_ms: 1,
            link_drop_probability: 0.0,
            main_clock_ppm: 0,
            supervisor_clock_ppm: 0,
            main_voltage_sensor: SensorModel::ideal(),
            supervisor_voltage_sensor: SensorModel::ideal(),
            main_resistance_sensor: SensorModel::ideal(),
            supervisor_resistance_sensor: SensorModel::ideal(),
            open_circuit_reading: 10_000,
            post_shock_blanking_ms: 1_500,
            impedance_noise_ohms: 0.005,
            impedance_shock_saturation_ms: 300,
            impedance_shock_offset_ohms: 3.0,
            impedance_shock_recovery_ms: 1_500.0,
            shock_step_us: 10,
            latch_timing: LatchTiming::PROPOSED,
            sequence_latch: true,
            rail_rise_us: 1_000,
            relay_break_lead_us: 1_000,
            relay_bounce_us: 1_000,
            relay_return_us: 5_000,
            over_current_a: 80.0,
            idle_fast_a: 3.0,
            test_voltage_limit_v: 60.0,
            test_load_ohms: 200.0,
            hw: HwTiming::NOMINAL,
            firmware: Config::default(),
            battery_weak: false,
            ovp_powers_up_tripped: false,
            pad_reading_recovery_ms: 300,
            voltage_sample_ms: 1,
            pad_sample_ms: 1,
            sample_phase_ms: 0,
            toggle_half_period_us: 500,
        }
    }
}

/// A fault the simulator can switch on at a chosen moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fault {
    /// The main controller's voltage sensor reads 20 % high.
    MainVoltageSensorWrong,
    /// The supervisor's voltage sensor reads 20 % low.
    SupervisorVoltageSensorWrong,
    /// The charger cannot get above 60 % of the voltage asked for.
    ChargerWeak,
    /// The charger ignores its target and keeps charging while switched on.
    ChargerRunaway,
    /// The internal discharge load is disconnected: the capacitor only
    /// empties through its slow leakage.
    DischargeStuck,
    /// The main controller stops running (no more ticks, no more heart-signal
    /// samples). Its outputs stay as they were last set.
    MainFreeze,
    /// The wire between the controllers stops carrying messages, both ways.
    LinkDrop,
    /// The pads come off the patient (and the responder does not put them back).
    PadsOff,
    /// Someone holds the cancel control for one second.
    CancelPressed,
    /// The cancel control sticks down and stays pressed (both controllers
    /// see it). Decision D-31: this must be reported as a stuck button.
    CancelStuck,
    /// The battery is suddenly flat: the power monitor says so and the charger stops.
    BatteryDepleted,
    /// The output bridge does not fire when told to, and reports nothing.
    BridgeFailsToFire,
    /// The pad-impedance channel used for compression sensing stops
    /// delivering samples. Not in [`Fault::ALL`], so that the random
    /// campaign's choices are the same as before compression sensing; it
    /// has its own scenario and tests.
    ImpedanceChannelLost,
    // The shock-path faults below (D-59, D-61) are not in `Fault::ALL`
    // either, so the campaign's random choices stay comparable with the
    // runs before them; each has its own tests.
    /// The patient relay does not move when its coil is powered.
    RelayFailsToTransfer,
    /// The patient relay does not come back from the bridge.
    RelayFailsToReturn,
    /// The patient relay is four times slower than its model times.
    RelaySlow,
    /// The main controller's phase timer is stuck: phase 1 never ends.
    MainTimerStuck,
    /// The phase timer keeps toggling phase 1 on and off (a timer left
    /// running by a frozen core, review R-4).
    MainTimerToggling,
    /// The output is shorted: the over-current trip must end the sequence.
    OutputShort,
    /// The request line is stuck on, whatever the main controller does.
    RequestStuckOn,
    /// The permit line is stuck on, whatever the supervisor does.
    PermitStuckOn,
    /// The request line toggles while the main controller holds it up.
    RequestToggling,
    /// The permit line toggles while the supervisor holds it up.
    PermitToggling,
    // ----- design revision 0.3 hardware elements (not in `Fault::ALL`) ----
    /// One short pulse on the permit line (a glitch, or the supervisor's
    /// own power-up): the detector gives up to 7.5 ms of consent (LR §1.2).
    PermitGlitch,
    /// The lead gate (`HA_ON50`/`LB_LOCK`) is stuck "pass": a low side
    /// starts without its high-side lead (B14.5 T-4).
    LeadGateStuck,
    /// The relay's rest (aux) contact reads "left rest" all the time.
    AuxContactStuckOpen,
    /// The relay coil supply (`K_SUPPLY_EN` path) is dead.
    CoilSupplyDead,
    /// The fast idle-current comparator is stuck "tripped" (v0.3 G-3).
    IdleFastStuckTripped,
    /// The fast idle-current comparator is stuck "not tripped".
    IdleFastStuckClear,
    /// A low-side switch is shorted (fault 16, R-14).
    LowSideShorted,
    /// The low-side driver supply monitor `LSD_OK` is stuck low.
    LsdOkStuckLow,
    /// `LSD_OK` stuck high (hidden: returns to v0.2 behaviour).
    LsdOkStuckHigh,
    /// The low-side driver supply (MGJ2) fails.
    LowSideSupplyFails,
    /// The sequence hold (SW5 / `SEQ_HOLD`) is dead: withdrawal of consent
    /// during a sequence cuts it (the truncation D-59 removed).
    SeqHoldDead,
    /// The over-voltage comparator never trips.
    OvpComparatorDead,
    /// The over-voltage latch feedback is open: it trips only while the
    /// condition lasts.
    OvpLatchOpen,
    /// The reset switch Q801 is shorted: the latch cannot be set (a test
    /// injection reads 0.12 V, no trip).
    OvpResetSwitchShorted,
    /// The reset path's coupling capacitor C812 is shorted: `OVP_RST` held
    /// high holds the latch clear (level reset, not edge).
    OvpResetCouplingShorted,
    /// The latch cannot be reset at all.
    OvpResetDead,
    /// A discharge path's "held" sensor stuck reading "held" (0 = A1,
    /// 1 = A2, 2 = B1, 3 = B2).
    HoldSensorStuckHeld(u8),
    /// A "held" sensor stuck reading "not held".
    HoldSensorStuckNotHeld(u8),
    /// The charger stops delivering without saying so, until restarted
    /// (WP-E C7).
    ChargerSilentStop,
    /// The charger ignores the set point and charges until its own
    /// last-resort primary-side trip, which ends the charge with
    /// `CHG_DONE_N`.
    ChargerOvershoot,
    /// The charger's own trip fires far too early (1,500 V), with
    /// `CHG_DONE_N`: below the plan it looks like a stall but for that line.
    ChargerTripsEarly,
    /// The charger keeps charging when commanded off (only the discharge
    /// gating stops it).
    ChargerStuckOn,
    /// The main controller's voltage sensor reads 10 % low.
    MainVoltageSensorLow,
}

impl Fault {
    /// Every fault, in a fixed order.
    pub const ALL: [Fault; 12] = [
        Fault::MainVoltageSensorWrong,
        Fault::SupervisorVoltageSensorWrong,
        Fault::ChargerWeak,
        Fault::ChargerRunaway,
        Fault::DischargeStuck,
        Fault::MainFreeze,
        Fault::LinkDrop,
        Fault::PadsOff,
        Fault::CancelPressed,
        Fault::CancelStuck,
        Fault::BatteryDepleted,
        Fault::BridgeFailsToFire,
    ];

    /// A short plain-English description.
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Fault::MainVoltageSensorWrong => "main controller's voltage sensor reads 20% high",
            Fault::SupervisorVoltageSensorWrong => "supervisor's voltage sensor reads 20% low",
            Fault::ChargerWeak => "charger cannot reach its target (stalls at 60%)",
            Fault::ChargerRunaway => "charger runs away past its target",
            Fault::DischargeStuck => "internal discharge path stuck open (will not empty)",
            Fault::MainFreeze => "main controller freezes",
            Fault::LinkDrop => "message wire between the controllers drops",
            Fault::PadsOff => "pads fall off the patient",
            Fault::CancelPressed => "cancel control pressed",
            Fault::CancelStuck => "cancel control stuck down",
            Fault::BatteryDepleted => "battery suddenly depleted",
            Fault::BridgeFailsToFire => "output bridge fails to fire",
            Fault::ImpedanceChannelLost => "compression-sensing (impedance) channel stops",
            Fault::RelayFailsToTransfer => "patient relay does not move to the bridge",
            Fault::RelayFailsToReturn => "patient relay does not return to the front ends",
            Fault::RelaySlow => "patient relay four times slower than modelled",
            Fault::MainTimerStuck => "phase timer stuck in phase 1",
            Fault::MainTimerToggling => "phase timer keeps toggling",
            Fault::OutputShort => "output shorted (over-current)",
            Fault::RequestStuckOn => "request line stuck on",
            Fault::PermitStuckOn => "permit line stuck on",
            Fault::RequestToggling => "request line toggling",
            Fault::PermitToggling => "permit line toggling",
            Fault::PermitGlitch => "one short pulse on the permit line",
            Fault::LeadGateStuck => "lead gate stuck pass",
            Fault::AuxContactStuckOpen => "relay rest contact reads left-rest",
            Fault::CoilSupplyDead => "relay coil supply dead",
            Fault::IdleFastStuckTripped => "fast idle comparator stuck tripped",
            Fault::IdleFastStuckClear => "fast idle comparator stuck clear",
            Fault::LowSideShorted => "low-side switch shorted",
            Fault::LsdOkStuckLow => "LSD_OK stuck low",
            Fault::LsdOkStuckHigh => "LSD_OK stuck high",
            Fault::LowSideSupplyFails => "low-side driver supply fails",
            Fault::SeqHoldDead => "sequence hold dead",
            Fault::OvpComparatorDead => "over-voltage comparator dead",
            Fault::OvpLatchOpen => "over-voltage latch feedback open",
            Fault::OvpResetSwitchShorted => "over-voltage reset switch shorted",
            Fault::OvpResetCouplingShorted => "over-voltage reset coupling capacitor shorted",
            Fault::OvpResetDead => "over-voltage latch will not reset",
            Fault::HoldSensorStuckHeld(_) => "held sensor stuck held",
            Fault::HoldSensorStuckNotHeld(_) => "held sensor stuck not held",
            Fault::ChargerSilentStop => "charger stops silently",
            Fault::ChargerOvershoot => "charger overshoots to its own trip",
            Fault::ChargerTripsEarly => "charger's own trip fires early",
            Fault::ChargerStuckOn => "charger stuck on",
            Fault::MainVoltageSensorLow => "main controller's voltage sensor reads 10% low",
        }
    }

    /// The command-line name.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Fault::MainVoltageSensorWrong => "main-voltage-sensor",
            Fault::SupervisorVoltageSensorWrong => "supervisor-voltage-sensor",
            Fault::ChargerWeak => "charger-weak",
            Fault::ChargerRunaway => "charger-runaway",
            Fault::DischargeStuck => "discharge-stuck",
            Fault::MainFreeze => "main-freeze",
            Fault::LinkDrop => "link-drop",
            Fault::PadsOff => "pads-off",
            Fault::CancelPressed => "cancel",
            Fault::CancelStuck => "cancel-stuck",
            Fault::BatteryDepleted => "battery-depleted",
            Fault::BridgeFailsToFire => "bridge-fails",
            Fault::ImpedanceChannelLost => "impedance-lost",
            Fault::RelayFailsToTransfer => "relay-no-transfer",
            Fault::RelayFailsToReturn => "relay-no-return",
            Fault::RelaySlow => "relay-slow",
            Fault::MainTimerStuck => "timer-stuck",
            Fault::MainTimerToggling => "timer-toggling",
            Fault::OutputShort => "output-short",
            Fault::RequestStuckOn => "request-stuck",
            Fault::PermitStuckOn => "permit-stuck",
            Fault::RequestToggling => "request-toggling",
            Fault::PermitToggling => "permit-toggling",
            Fault::PermitGlitch => "permit-glitch",
            Fault::LeadGateStuck => "lead-gate-stuck",
            Fault::AuxContactStuckOpen => "aux-contact-open",
            Fault::CoilSupplyDead => "coil-supply-dead",
            Fault::IdleFastStuckTripped => "idle-fast-tripped",
            Fault::IdleFastStuckClear => "idle-fast-clear",
            Fault::LowSideShorted => "low-side-shorted",
            Fault::LsdOkStuckLow => "lsd-ok-low",
            Fault::LsdOkStuckHigh => "lsd-ok-high",
            Fault::LowSideSupplyFails => "low-side-supply-fails",
            Fault::SeqHoldDead => "seq-hold-dead",
            Fault::OvpComparatorDead => "ovp-comparator-dead",
            Fault::OvpLatchOpen => "ovp-latch-open",
            Fault::OvpResetSwitchShorted => "ovp-reset-switch-shorted",
            Fault::OvpResetCouplingShorted => "ovp-reset-coupling-shorted",
            Fault::OvpResetDead => "ovp-reset-dead",
            Fault::HoldSensorStuckHeld(_) => "held-sensor-stuck-held",
            Fault::HoldSensorStuckNotHeld(_) => "held-sensor-stuck-not-held",
            Fault::ChargerSilentStop => "charger-silent-stop",
            Fault::ChargerOvershoot => "charger-overshoot",
            Fault::ChargerTripsEarly => "charger-trips-early",
            Fault::ChargerStuckOn => "charger-stuck-on",
            Fault::MainVoltageSensorLow => "main-voltage-sensor-low",
        }
    }
}

/// A consent line, for forcing one low at a chosen instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentLine {
    /// The main controller's request.
    Request,
    /// The supervisor's permit (and test permit).
    Permit,
}

/// True if a scripted rhythm is one a shock is meant for (ground truth).
/// Any ventricular fibrillation (coarse or fine) or ventricular tachycardia.
#[must_use]
pub fn rhythm_is_shockable(r: Rhythm) -> bool {
    matches!(
        r,
        Rhythm::VentricularFibrillation { .. } | Rhythm::VentricularTachycardia { .. }
    )
}

/// A plain-English name for a rhythm.
#[must_use]
pub fn rhythm_name(r: Rhythm) -> String {
    match r {
        Rhythm::Sinus { bpm, .. } => format!("normal rhythm {bpm:.0}/min"),
        Rhythm::VentricularFibrillation { amplitude_uv } => {
            if amplitude_uv < 200.0 {
                format!("fine VF ({amplitude_uv:.0} µV)")
            } else {
                format!("VF ({amplitude_uv:.0} µV)")
            }
        }
        Rhythm::VentricularTachycardia { bpm, .. } => format!("VT {bpm:.0}/min"),
        Rhythm::Asystole => "asystole (flatline)".to_string(),
    }
}

/// How a sequence ended in the plant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockEnd {
    /// The last phase ended normally.
    Completed,
    /// The hardware time cap cut it.
    TimeCap,
    /// The over-current trip cut it.
    OverCurrent,
    /// A per-phase cap cut it (D-63).
    PhaseCap,
    /// Another hardware fault latch ended it (idle current, supply).
    HardwareFault,
    /// Only with [`PlantConfig::sequence_latch`] off (the first design): a
    /// consent was removed mid-pulse and the drive was cut there.
    ConsentCut,
}

/// One shock sequence the bridge ran, computed step by step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockRecord {
    /// When it started (the first conducting moment), simulation ms.
    pub at_ms: u32,
    /// When it started, simulation µs.
    pub start_us: u64,
    /// When it ended, simulation µs.
    pub end_us: u64,
    /// When the interlock rail rose (both consents present), simulation µs.
    pub rail_us: u64,
    /// When the main controller last raised its request, simulation µs.
    pub request_us: Option<u64>,
    /// The plan the main controller loaded.
    pub plan: ShockPlan,
    /// The true capacitor voltage when it started.
    pub capacitor_v: f64,
    /// The true pad resistance at the start, or `None` if the pads were off.
    pub patient_ohms: Option<f64>,
    /// The aed-waveform model's result for the plan from the true starting
    /// voltage: what the shock SHOULD deliver (None if the pads were off).
    pub result: Option<DischargeResult>,
    /// Energy actually delivered to the patient, integrated step by step, J.
    pub integrated_j: f64,
    /// Highest patient current actually reached, A.
    pub integrated_peak_a: f64,
    /// Highest current in the output wherever it went (into a short too), A.
    pub output_peak_a: f64,
    /// The true capacitor voltage when it ended.
    pub final_v: f64,
    /// How it ended.
    pub end: ShockEnd,
    /// The rhythm the patient actually had at the start.
    pub rhythm: Rhythm,
    /// When that rhythm started (simulation time, ms).
    pub rhythm_since_ms: u32,
    /// Whether the cancel control was pressed at the start.
    pub cancel_pressed: bool,
}

impl ShockRecord {
    /// Energy delivered to the patient, joules (integrated, model).
    #[must_use]
    pub fn delivered_j(&self) -> f64 {
        self.integrated_j
    }
    /// What the plan should have delivered from the true starting voltage,
    /// joules (aed-waveform model; 0 with the pads off).
    #[must_use]
    pub fn planned_j(&self) -> f64 {
        self.result.map_or(0.0, |r| r.patient_energy_j)
    }
    /// Highest patient current, amperes.
    #[must_use]
    pub fn peak_a(&self) -> f64 {
        self.integrated_peak_a
    }
    /// Voltage left on the capacitor after the shock.
    #[must_use]
    pub fn residual_v(&self) -> f64 {
        self.final_v
    }
    /// How long the pulse ran, first conducting moment to end, µs.
    #[must_use]
    pub fn duration_us(&self) -> u64 {
        self.end_us.saturating_sub(self.start_us)
    }
    /// From both consents present (rail up) to the energy delivered, µs.
    #[must_use]
    pub fn consent_to_delivered_us(&self) -> u64 {
        self.end_us.saturating_sub(self.rail_us)
    }
    /// From the main controller's request to the energy delivered, µs.
    #[must_use]
    pub fn request_to_delivered_us(&self) -> Option<u64> {
        self.request_us.map(|t| self.end_us.saturating_sub(t))
    }
}

/// A self-test sequence into the internal test load (D-61).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestSequenceRecord {
    /// Start, simulation µs.
    pub start_us: u64,
    /// End, simulation µs.
    pub end_us: u64,
    /// How it ended.
    pub end: ShockEnd,
    /// Energy into the test load, J.
    pub load_j: f64,
}

/// The patient relay's contacts.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Relay {
    coil: bool,
    /// µs since the coil last changed.
    since_us: u32,
    front_closed: bool,
    bridge_closed: bool,
    /// µs the contacts have been on the bridge without a break.
    bridge_stable_us: u32,
    /// Transfer time chosen when the coil was energised (None: no supply,
    /// the relay does not move).
    transfer_us: Option<u32>,
}

/// One discharge path's hold (the charge pump holding its MOSFET stack off)
/// with the v0.3 engage and release times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PathHold {
    /// The path is held off (not discharging).
    pub held: bool,
    /// µs the command has differed from `held`.
    pending_us: u32,
}

impl PathHold {
    fn step(&mut self, command: bool, engage_us: u32, release_us: u32, dt_us: u32) {
        if command == self.held {
            self.pending_us = 0;
            return;
        }
        self.pending_us = self.pending_us.saturating_add(dt_us);
        let need = if command { engage_us } else { release_us };
        if self.pending_us >= need {
            self.held = command;
            self.pending_us = 0;
        }
    }
}

/// The sheet-12 state that lives only while the rail is up (power-on reset
/// of every sequencer latch when it rises, §B13.1 step 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Sheet12 {
    k_en_since_us: Option<u64>,
    k_was_rest: bool,
    coil_cap_expired: bool,
    left_rest_since_us: Option<u64>,
    btest_since_us: Option<u64>,
    ha_on_us: u32,
    fault: bool,
    done_at_us: Option<u64>,
}

/// The sequence the timer is running.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Running {
    t_us: u32,
    record: ShockRecord,
    test: bool,
    load_j: f64,
}

/// Something physical that happened in one plant step, for the timeline.
#[derive(Debug, Clone, PartialEq)]
pub enum PlantEvent {
    /// A shock sequence ended (the record is complete).
    Shock(Box<ShockRecord>),
    /// The interlock rail rose.
    RailUp {
        /// Under a test permit.
        test: bool,
    },
    /// A sequence started (the latch was set).
    SequenceStarted,
    /// The patient relay reached the bridge and settled.
    RelayOnBridge,
    /// The patient relay is back on the front ends.
    RelayBack,
    /// The rail dropped before any sequence started.
    RailDropped,
    /// The hardware's permit-duration cap removed the rail.
    PermitCapTripped,
    /// The bridge was allowed to fire but the injected fault stopped it.
    BridgeDidNotFire,
    /// The patient's rhythm changed.
    RhythmChanged(Rhythm),
}

/// The physical world.
#[derive(Debug)]
pub struct Plant {
    /// The constants in use.
    pub config: PlantConfig,
    /// Simulation time, ms.
    pub now_ms: u32,
    /// True capacitor voltage.
    pub cap_v: f64,
    /// Highest true capacitor voltage seen.
    pub max_cap_v: f64,
    /// Main controller's hold on the discharge path.
    pub main_hold: bool,
    /// Supervisor's hold on the discharge path.
    pub supervisor_hold: bool,
    /// Charger target set by the main controller.
    pub charger_target: Option<Volts>,
    /// The shock request line, with the loaded plan.
    pub request: Option<ShockPlan>,
    /// The request line raised with no sequence loaded (self-test probes).
    pub request_bare: bool,
    /// When the request was last raised, µs.
    pub request_raised_us: Option<u64>,
    /// The supervisor's permit line.
    pub permit: bool,
    /// The supervisor's test permit line (D-61).
    pub test_permit: bool,
    /// The sequence logic (D-59).
    pub latch: Latch,
    /// For a sweep: force this consent line low from this instant (µs) on.
    pub consent_cut: Option<(ConsentLine, u64)>,
    /// A bridge test step that fails, and how (D-61 tests).
    pub bridge_switch_fault: Option<(BridgeTestStep, BridgeTestOutcome)>,
    /// What the adult/child switch's second contact reads, if it differs
    /// from [`Plant::mode`] (D-60).
    pub mode_ceiling: Option<aed_types::PatientMode>,
    /// Every self-test sequence into the internal test load.
    pub test_sequences: Vec<TestSequenceRecord>,
    /// Discharge path A failed open.
    pub path_a_open: bool,
    /// Discharge path B failed open.
    pub path_b_open: bool,
    /// True pad resistance, `None` when the pads are not on the patient.
    pub pad_ohms: Option<f64>,
    /// The cancel control is held down until this time (exclusive), if set.
    pub cancel_until_ms: Option<u32>,
    /// The adult/child switch.
    pub mode: aed_types::PatientMode,
    /// The battery as the power monitor reports it.
    pub battery: BatteryStatus,
    /// The pad set plugged into the device.
    pub cartridge: PadCartridge,
    /// The ready indicator.
    pub ready: bool,
    /// The supervisor's own fault alarm (buzzer and light).
    pub supervisor_alarm: bool,
    /// The responder is doing chest compressions now.
    pub compressing: bool,
    /// Compression artifact size, microvolts peak to peak.
    pub compression_uv: f64,
    /// Compression rate, per minute. Default 110.
    pub compression_per_min: f64,
    /// Change in pad impedance from one compression, ohms peak to peak.
    /// Default 1.5 Ω: the median of 1.1 to 1.7 Ω reported for out-of-hospital
    /// compressions through pads (Alonso et al., CinC 2013;40:803); the
    /// Paris recordings have a median of about 2.6 Ω.
    pub compression_ohms: f64,
    /// Someone is giving breaths (bag and mask) every 6 s.
    pub ventilating: bool,
    /// Change in pad impedance from one breath, ohms. Default 0.8 Ω. Model
    /// choice (no verified figure; breaths are of the same order as
    /// compressions but go in over about 1 s).
    pub ventilation_ohms: f64,
    /// Change in pad impedance from each heartbeat while the heart is
    /// beating (normal rhythm), ohms. Default 0.05 Ω: the cardiac component
    /// "rarely exceeding 100 mΩ" (Urteaga et al., Entropy 2021;23:847).
    pub cardiac_ohms: f64,
    /// Someone is pressing on the chest (a touch, not compressions) until
    /// this time, if set.
    pub touch_until_ms: Option<u32>,
    /// Change in pad impedance from such a touch, ohms. Default 1.0 Ω (a
    /// firm press; a light touch changes too little to see). Model choice.
    pub touch_ohms: f64,
    /// Ground truth: since when a responder's hands have been on the chest
    /// (compressing or touching), `None` while they are off.
    pub hands_on_since: Option<u32>,
    /// Faults switched on so far.
    pub faults: Vec<Fault>,
    /// Every prompt the main controller gave, with the time.
    pub prompts: Vec<(u32, Prompt)>,
    /// Number of metronome beats.
    pub metronome_beats: u32,
    /// Every shock the bridge fired.
    pub shocks: Vec<ShockRecord>,
    /// Status messages on the wire to the supervisor (arrival time, message).
    pub to_supervisor: VecDeque<(u32, MainStatus)>,
    /// Reports on the wire to the main controller (arrival time, report).
    pub to_main: VecDeque<(u32, SupervisorReport)>,
    /// Messages lost on the wire.
    pub messages_dropped: u32,
    /// Energy the charger has put into the capacitor, joules (model). This
    /// is what the battery pays for, before charger losses (not modelled).
    pub charger_energy_j: f64,
    /// Energy burned off in the internal discharge load, joules (model):
    /// charges emptied without a shock, what is left after each shock, and
    /// the self-test charge. This is the heat cost inside the device.
    pub dump_energy_j: f64,
    /// The patient's current rhythm.
    pub rhythm: Rhythm,
    /// When the current rhythm started.
    pub rhythm_since_ms: u32,
    /// Rhythm after the first, second, ... shock (None: unchanged).
    pub after_shock: Vec<Option<Rhythm>>,
    synth: Synth,
    rng: Xorshift32,
    imp_rng: Xorshift32,
    blank_until_ms: u32,
    compression_phase: f64,
    cardiac_phase: f64,
    touch_started_ms: u32,
    last_shock_ms: Option<u32>,
    events: Vec<PlantEvent>,
    bridge_fail_reported: bool,
    relay: Relay,
    prev_cmd_lb: bool,
    sheet12: Sheet12,
    /// The main controller's delivery sequencer (its board layer, D-59).
    pub sequencer: Option<Sequencer>,
    /// Its outputs at the last step.
    pub seq_out: SequencerOutputs,
    /// `K_SUPPLY_EN` (FW-12).
    pub coil_supply: bool,
    coil_supply_since_us: Option<u64>,
    /// Relay operations started with the coil reservoir not yet full.
    pub relay_slow_starts: u32,
    /// ms in which the coil reservoir's pre-charge overlapped a full-power
    /// capacitor charge (patient-interface capture).
    pub supply_overlap_ms: u32,
    /// ms in which that overlap browned the rails out (weak battery).
    pub brownout_ms: u32,
    /// The two discharge paths' holds.
    pub path_holds: (PathHold, PathHold),
    /// `OVP_TEST`, `OVP_RST` lines and the latch.
    pub ovp_test: bool,
    /// `OVP_RST`.
    pub ovp_reset: bool,
    ovp_reset_high_us: u32,
    ovp_reset_used: bool,
    /// The over-voltage latch (`OVP_TRIP`).
    pub ovp_latched: bool,
    /// Times the latch was reset by an `OVP_RST` pulse.
    pub ovp_resets: u32,
    /// The charger has ended a charge by its own trip (`CHG_DONE_N`).
    pub charger_done: bool,
    charger_on_since_us: Option<u64>,
    charger_off_since_us: Option<u64>,
    charger_silent: bool,
    /// Charger restarts seen (off ≥ 2 ms then on).
    pub charger_restarts: u32,
    permit_last_high_us: Option<u64>,
    test_last_high_us: Option<u64>,
    permit_glitch_at_us: Option<u64>,
    /// Battery load pulses (start ms) the board layer ran (FW-19).
    pub battery_load_pulses: Vec<u32>,
    running: Option<Running>,
    rail_since_us: Option<u64>,
    relay_back_ms: Option<u32>,
    held_v: (f64, f64),
    held_ohms: (Option<f64>, Option<f64>),
    /// The device's inside temperature and its six sensors (D-48).
    pub thermal: crate::thermal::Thermal,
    /// The electronics are on (they warm the inside); true in a rescue.
    pub electronics_on: bool,
    heat_accounted_j: (f64, f64),
}

impl Plant {
    /// A new plant: capacitor empty, pads off, `rhythm` playing.
    #[must_use]
    pub fn new(config: PlantConfig, synth: Synth, seed: u32) -> Self {
        let rhythm = synth.rhythm();
        Plant {
            config,
            now_ms: 0,
            cap_v: 0.0,
            max_cap_v: 0.0,
            main_hold: false,
            supervisor_hold: false,
            charger_target: None,
            request: None,
            request_bare: false,
            request_raised_us: None,
            permit: false,
            test_permit: false,
            latch: Latch::IDLE,
            consent_cut: None,
            bridge_switch_fault: None,
            mode_ceiling: None,
            test_sequences: Vec::new(),
            path_a_open: false,
            path_b_open: false,
            pad_ohms: None,
            cancel_until_ms: None,
            mode: aed_types::PatientMode::Adult,
            battery: BatteryStatus::Good,
            cartridge: PadCartridge::Ok,
            ready: false,
            supervisor_alarm: false,
            compressing: false,
            compression_uv: 1_500.0,
            compression_per_min: 110.0,
            compression_ohms: 1.5,
            ventilating: false,
            ventilation_ohms: 0.8,
            cardiac_ohms: 0.05,
            touch_until_ms: None,
            touch_ohms: 1.0,
            hands_on_since: None,
            faults: Vec::new(),
            prompts: Vec::new(),
            metronome_beats: 0,
            shocks: Vec::new(),
            to_supervisor: VecDeque::new(),
            to_main: VecDeque::new(),
            messages_dropped: 0,
            charger_energy_j: 0.0,
            dump_energy_j: 0.0,
            rhythm,
            rhythm_since_ms: 0,
            after_shock: Vec::new(),
            synth,
            rng: Xorshift32::new(seed ^ 0x5eed_1234),
            imp_rng: Xorshift32::new(seed ^ 0x1a2b_3c4d),
            blank_until_ms: 0,
            compression_phase: 0.0,
            cardiac_phase: 0.0,
            touch_started_ms: 0,
            last_shock_ms: None,
            events: Vec::new(),
            bridge_fail_reported: false,
            relay: Relay {
                coil: false,
                since_us: 0,
                front_closed: true,
                bridge_closed: false,
                bridge_stable_us: 0,
                transfer_us: None,
            },
            prev_cmd_lb: false,
            sheet12: Sheet12::default(),
            sequencer: None,
            seq_out: SequencerOutputs::default(),
            coil_supply: false,
            coil_supply_since_us: None,
            relay_slow_starts: 0,
            supply_overlap_ms: 0,
            brownout_ms: 0,
            path_holds: (PathHold::default(), PathHold::default()),
            ovp_test: false,
            ovp_reset: false,
            ovp_reset_high_us: 0,
            ovp_reset_used: false,
            ovp_latched: config.ovp_powers_up_tripped,
            ovp_resets: 0,
            charger_done: false,
            charger_on_since_us: None,
            charger_off_since_us: None,
            charger_silent: false,
            charger_restarts: 0,
            permit_last_high_us: None,
            test_last_high_us: None,
            permit_glitch_at_us: None,
            battery_load_pulses: Vec::new(),
            running: None,
            rail_since_us: None,
            relay_back_ms: None,
            held_v: (0.0, 0.0),
            held_ohms: (None, None),
            thermal: crate::thermal::Thermal::at(20.0),
            electronics_on: true,
            heat_accounted_j: (0.0, 0.0),
        }
    }

    /// True if this fault has been switched on.
    #[must_use]
    pub fn has_fault(&self, f: Fault) -> bool {
        self.faults.contains(&f)
    }

    /// Capacitance in farads.
    fn farads(&self) -> f64 {
        self.config.waveform.capacitance_uf * 1e-6
    }

    /// True while the discharge path is emptying the capacitor.
    #[must_use]
    pub fn discharge_active(&self) -> bool {
        discharge_path_active(self.path_holds.0.held, self.path_holds.1.held)
    }

    /// The four "held" sensors as the main controller reads them (v0.3).
    #[must_use]
    pub fn hold_readback(&self) -> HoldReadback {
        let sense = |i: u8, held: bool| {
            if self.has_fault(Fault::HoldSensorStuckHeld(i)) {
                true
            } else if self.has_fault(Fault::HoldSensorStuckNotHeld(i)) {
                false
            } else {
                held
            }
        };
        let (a, b) = (self.path_holds.0.held, self.path_holds.1.held);
        HoldReadback {
            a1: sense(0, a),
            a2: sense(1, a),
            b1: sense(2, b),
            b2: sense(3, b),
        }
    }

    /// `K_SUPPLY_EN` (FW-12): the coil reservoir starts filling.
    pub fn set_coil_supply(&mut self, on: bool) {
        if on && !self.coil_supply {
            self.coil_supply_since_us = Some(self.now_us());
        } else if !on {
            self.coil_supply_since_us = None;
        }
        self.coil_supply = on;
    }

    /// The main controller asks for a sequence (the board layer loads the
    /// delivery sequencer).
    pub fn load_sequencer(&mut self, job: SequencerJob) {
        if self.sequencer.is_some_and(|s| s.job == job) {
            return;
        }
        let timing = SequencerTiming::from_config(&self.config.firmware);
        self.sequencer = Some(Sequencer::new(job, timing));
    }

    /// The fixed plan of the dry shock-mode run (D-62 T-5).
    #[must_use]
    pub const fn dry_run_plan() -> ShockPlan {
        ShockPlan {
            charge_voltage: Volts(0),
            phase1: aed_types::Micros(1_000),
            interphase: aed_types::Micros(500),
            phase2: aed_types::Micros(1_000),
            expected_energy: aed_types::Decijoules(0),
        }
    }

    /// The board layer's battery load test (FW-19): a pulse of at most
    /// `battery_load_pulse_max_ms`, never repeated within
    /// `battery_load_min_interval_ms`. Returns the battery status.
    pub fn battery_load_test(&mut self) -> BatteryStatus {
        let gap = self.config.firmware.battery_load_min_interval_ms;
        let now = self.now_ms;
        if self
            .battery_load_pulses
            .last()
            .is_none_or(|t| now.saturating_sub(*t) >= gap)
        {
            self.battery_load_pulses.push(now);
        }
        self.battery
    }

    /// True if the charger is physically able to run now: it is switched on,
    /// the battery can power it and, unless the hardware rule is switched off
    /// for a test, the discharge path is not active.
    #[must_use]
    pub fn charger_running(&self) -> bool {
        let on = (self.charger_target.is_some() || self.has_fault(Fault::ChargerStuckOn))
            && self.battery != BatteryStatus::Depleted
            && !self.charger_done;
        if self.config.charger_gated_by_discharge {
            charger_may_run(on, self.discharge_active())
        } else {
            on
        }
    }

    /// True while the cancel control is held.
    #[must_use]
    pub fn cancel_pressed(&self) -> bool {
        self.cancel_until_ms
            .is_some_and(|until| self.now_ms < until)
    }

    /// Hold the cancel control down for `hold_ms` from now.
    pub fn press_cancel(&mut self, hold_ms: u32) {
        self.cancel_until_ms = Some(self.now_ms.saturating_add(hold_ms));
    }

    /// Change the patient's rhythm now.
    pub fn set_rhythm(&mut self, r: Rhythm) {
        self.synth.set_rhythm(r);
        self.rhythm = r;
        self.rhythm_since_ms = self.now_ms;
        self.events.push(PlantEvent::RhythmChanged(r));
    }

    /// Switch on a fault now.
    pub fn inject(&mut self, f: Fault) {
        if !self.faults.contains(&f) {
            self.faults.push(f);
        }
        match f {
            Fault::MainVoltageSensorWrong => self.config.main_voltage_sensor.gain = 1.2,
            Fault::SupervisorVoltageSensorWrong => {
                self.config.supervisor_voltage_sensor.gain = 0.8;
            }
            Fault::PadsOff => self.pad_ohms = None,
            Fault::MainVoltageSensorLow => self.config.main_voltage_sensor.gain = 0.9,
            Fault::ChargerSilentStop => self.charger_silent = true,
            Fault::PermitGlitch => self.permit_glitch_at_us = Some(self.now_us()),
            Fault::CancelPressed => self.press_cancel(1_000),
            Fault::CancelStuck => self.press_cancel(u32::MAX),
            Fault::BatteryDepleted => self.battery = BatteryStatus::Depleted,
            Fault::ChargerWeak
            | Fault::ChargerRunaway
            | Fault::DischargeStuck
            | Fault::MainFreeze
            | Fault::LinkDrop
            | Fault::BridgeFailsToFire
            | Fault::ImpedanceChannelLost
            | Fault::RelayFailsToTransfer
            | Fault::RelayFailsToReturn
            | Fault::RelaySlow
            | Fault::MainTimerStuck
            | Fault::MainTimerToggling
            | Fault::OutputShort
            | Fault::RequestStuckOn
            | Fault::PermitStuckOn
            | Fault::RequestToggling
            | Fault::PermitToggling
            | Fault::LeadGateStuck
            | Fault::AuxContactStuckOpen
            | Fault::CoilSupplyDead
            | Fault::IdleFastStuckTripped
            | Fault::IdleFastStuckClear
            | Fault::LowSideShorted
            | Fault::LsdOkStuckLow
            | Fault::LsdOkStuckHigh
            | Fault::LowSideSupplyFails
            | Fault::SeqHoldDead
            | Fault::OvpComparatorDead
            | Fault::OvpLatchOpen
            | Fault::OvpResetSwitchShorted
            | Fault::OvpResetCouplingShorted
            | Fault::OvpResetDead
            | Fault::HoldSensorStuckHeld(_)
            | Fault::HoldSensorStuckNotHeld(_)
            | Fault::ChargerOvershoot
            | Fault::ChargerTripsEarly
            | Fault::ChargerStuckOn => {}
        }
    }

    /// Clock reading for a controller with this drift.
    fn clock(&self, ppm: i32) -> aed_types::Millis {
        let t = f64::from(self.now_ms) * (1.0 + f64::from(ppm) * 1e-6);
        // Non-negative and far below u32::MAX for any sensible run; `as` saturates.
        aed_types::Millis(t.max(0.0) as u32)
    }

    /// The main controller's clock.
    #[must_use]
    pub fn main_clock(&self) -> aed_types::Millis {
        self.clock(self.config.main_clock_ppm)
    }

    /// The supervisor's clock.
    #[must_use]
    pub fn supervisor_clock(&self) -> aed_types::Millis {
        self.clock(self.config.supervisor_clock_ppm)
    }

    /// A voltage reading through a sensor, rounded and kept in range, of
    /// the true voltage now.
    #[must_use]
    pub fn read_volts(&self, sensor: SensorModel) -> Volts {
        Self::volts_of(sensor, self.cap_v)
    }

    fn volts_of(sensor: SensorModel, v: f64) -> Volts {
        let v = sensor.read(v).round().clamp(0.0, f64::from(u16::MAX));
        // Clamped into the u16 range above.
        Volts(v as u16)
    }

    /// The main controller's voltage reading (its converter's latest sample).
    #[must_use]
    pub fn main_volts(&self) -> Volts {
        Self::volts_of(self.config.main_voltage_sensor, self.held_v.0)
    }

    /// The supervisor's voltage reading (its converter's latest sample).
    #[must_use]
    pub fn supervisor_volts(&self) -> Volts {
        if self.ovp_test {
            // OVP_TEST is injected at the supervisor's sense amplifier
            // input: its own reading goes to full scale (sense-sheet capture).
            return Volts(2_600);
        }
        Self::volts_of(self.config.supervisor_voltage_sensor, self.held_v.1)
    }

    /// The true pad resistance as the front ends can see it now: `None`
    /// (open) with the pads off, while the relay has them away from the
    /// front ends, or while the front ends recover after it returns.
    #[must_use]
    pub fn front_end_ohms(&self) -> Option<f64> {
        let recovering = self
            .relay_back_ms
            .is_some_and(|t| self.now_ms < t.saturating_add(self.config.pad_reading_recovery_ms));
        if !self.relay.front_closed || recovering {
            None
        } else {
            self.pad_ohms
        }
    }

    /// True while the pads are connected to the front ends (relay at rest).
    #[must_use]
    pub fn pads_on_front_ends(&self) -> bool {
        self.relay.front_closed
    }

    /// A pad-resistance reading through a sensor, of what the front ends
    /// see now.
    #[must_use]
    pub fn read_ohms(&self, sensor: SensorModel) -> aed_types::Ohms {
        Self::ohms_of(
            sensor,
            self.front_end_ohms(),
            self.config.open_circuit_reading,
        )
    }

    fn ohms_of(sensor: SensorModel, r: Option<f64>, open: u16) -> aed_types::Ohms {
        let v = match r {
            Some(r) => sensor.read(r),
            None => f64::from(open),
        };
        let v = v.round().clamp(0.0, f64::from(u16::MAX));
        // Clamped into the u16 range above.
        aed_types::Ohms(v as u16)
    }

    /// The main controller's pad reading (latest sample).
    #[must_use]
    pub fn main_ohms(&self) -> aed_types::Ohms {
        Self::ohms_of(
            self.config.main_resistance_sensor,
            self.held_ohms.0,
            self.config.open_circuit_reading,
        )
    }

    /// The supervisor's pad reading (latest sample).
    #[must_use]
    pub fn supervisor_ohms(&self) -> aed_types::Ohms {
        Self::ohms_of(
            self.config.supervisor_resistance_sensor,
            self.held_ohms.1,
            self.config.open_circuit_reading,
        )
    }

    /// The patient relay's position sense.
    #[must_use]
    pub fn relay_position(&self) -> RelayPosition {
        if self.relay.front_closed && !self.has_fault(Fault::AuxContactStuckOpen) {
            RelayPosition::FrontEnds
        } else if self.relay.bridge_closed {
            RelayPosition::Bridge
        } else {
            RelayPosition::Between
        }
    }

    /// The test-load current a bridge test step would see now, mA (FW-14).
    #[must_use]
    pub fn test_current_ma(&self) -> u16 {
        let ri = self.config.waveform.internal_ohms;
        let a = self.cap_v / (self.config.test_load_ohms + ri);
        (a * 1000.0).round().clamp(0.0, f64::from(u16::MAX)) as u16
    }

    /// One bridge test step (D-61), see [`Plant::bridge_test_step`].
    #[must_use]
    pub fn bridge_test_outcome(&self, step: BridgeTestStep) -> BridgeTestOutcome {
        self.bridge_test_step(step)
    }

    /// One bridge test step (D-61): only with the test rail up and no
    /// sequence running.
    #[must_use]
    pub fn bridge_test_step(&self, step: BridgeTestStep) -> BridgeTestOutcome {
        if !(self.latch.status == SequenceStatus::Armed && self.latch.test) {
            return BridgeTestOutcome::Refused;
        }
        match self.bridge_switch_fault {
            Some((s, o)) if s == step => o,
            _ => BridgeTestOutcome::Pass,
        }
    }

    /// Put a message on a wire, unless the wire is down or it is randomly lost.
    fn wire_accepts(&mut self) -> bool {
        if self.has_fault(Fault::LinkDrop) {
            self.messages_dropped = self.messages_dropped.saturating_add(1);
            return false;
        }
        let p = self.config.link_drop_probability;
        if p > 0.0 && f64::from(self.rng.next_unit()) < p {
            self.messages_dropped = self.messages_dropped.saturating_add(1);
            return false;
        }
        true
    }

    /// The main controller sends a status message.
    pub fn send_to_supervisor(&mut self, status: MainStatus) {
        if self.wire_accepts() {
            let at = self.now_ms.saturating_add(self.config.link_delay_ms);
            self.to_supervisor.push_back((at, status));
        }
    }

    /// The supervisor sends a report.
    pub fn send_to_main(&mut self, report: SupervisorReport) {
        if self.wire_accepts() {
            let at = self.now_ms.saturating_add(self.config.link_delay_ms);
            self.to_main.push_back((at, report));
        }
    }

    /// The newest status message that has arrived, if any (older ones are discarded).
    pub fn receive_at_supervisor(&mut self) -> Option<MainStatus> {
        let now = self.now_ms;
        let mut latest = None;
        while self.to_supervisor.front().is_some_and(|(at, _)| *at <= now) {
            latest = self.to_supervisor.pop_front().map(|(_, m)| m);
        }
        latest
    }

    /// The newest supervisor report that has arrived, if any.
    pub fn receive_at_main(&mut self) -> Option<SupervisorReport> {
        let now = self.now_ms;
        let mut latest = None;
        while self.to_main.front().is_some_and(|(at, _)| *at <= now) {
            latest = self.to_main.pop_front().map(|(_, m)| m);
        }
        latest
    }

    /// The highest voltage the charger will drive towards right now, if it is running.
    fn charger_limit(&self) -> Option<f64> {
        let stuck = self.has_fault(Fault::ChargerStuckOn);
        let target = match self.charger_target {
            Some(t) => f64::from(t.0) * self.config.charger_set_point_gain,
            None if stuck => self.config.charger_runaway_volts,
            None => return None,
        };
        if self.battery == BatteryStatus::Depleted {
            return None;
        }
        if !self.charger_running() || self.charger_silent {
            return None;
        }
        let started = self.charger_on_since_us.is_some_and(|t| {
            self.now_us().saturating_sub(t) >= u64::from(self.config.hw.charger_start_us)
        });
        if !started && !stuck {
            return None;
        }
        if self.has_fault(Fault::ChargerOvershoot)
            || self.has_fault(Fault::ChargerTripsEarly)
            || self.has_fault(Fault::ChargerRunaway)
        {
            Some(self.config.charger_runaway_volts)
        } else if self.has_fault(Fault::ChargerWeak) {
            Some(target * 0.6)
        } else {
            Some(target)
        }
    }

    /// Simulation time now, µs.
    #[must_use]
    pub fn now_us(&self) -> u64 {
        u64::from(self.now_ms) * 1_000
    }

    /// True while anything in the shock path is moving or could move: the
    /// plant then steps in [`PlantConfig::shock_step_us`].
    fn shock_path_busy(&self) -> bool {
        self.latch.status != SequenceStatus::Idle
            || self.relay.coil
            || !self.relay.front_closed
            || self.relay.bridge_closed
            || self.request.is_some()
            || self.request_bare
            || self.permit
            || self.test_permit
            || self.has_fault(Fault::RequestStuckOn)
            || self.has_fault(Fault::PermitStuckOn)
            || self.sequencer.is_some()
            || self.ovp_test
            || self.ovp_reset
            || self
                .permit_last_high_us
                .is_some_and(|t| self.now_us().saturating_sub(t) < 20_000)
            || self
                .test_last_high_us
                .is_some_and(|t| self.now_us().saturating_sub(t) < 20_000)
            || self.permit_glitch_at_us.is_some()
    }

    /// Advance the physics by one millisecond. Returns what happened.
    pub fn step(&mut self) -> Vec<PlantEvent> {
        let start_us = self.now_us();
        self.now_ms = self.now_ms.saturating_add(1);
        let dt = 1e-3;
        let c = self.farads();

        // Charger enable edges (FW-18): a restart (off ≥ 2 ms, then on)
        // clears a silent stop; its own trip is cleared by switching off.
        let now_us = start_us;
        if self.charger_target.is_some() {
            if self.charger_on_since_us.is_none() {
                let off_long = self
                    .charger_off_since_us
                    .is_some_and(|t| now_us.saturating_sub(t) >= 2_000);
                if off_long && self.charger_silent {
                    self.charger_silent = false;
                }
                if self.charger_off_since_us.is_some() {
                    self.charger_restarts = self.charger_restarts.saturating_add(1);
                }
                self.charger_on_since_us = Some(now_us);
            }
            self.charger_off_since_us = None;
        } else {
            self.charger_on_since_us = None;
            self.charger_off_since_us.get_or_insert(now_us);
            self.charger_done = false;
        }
        let trip_v = if self.has_fault(Fault::ChargerTripsEarly) {
            1_500.0
        } else {
            self.config.hw.charger_trip_v
        };
        // The coil reservoir's pre-charge against a full-power charge.
        let filling = self.coil_supply
            && self.coil_supply_since_us.is_some_and(|t| {
                now_us.saturating_sub(t) < u64::from(self.config.hw.reservoir_fill_us)
            });
        let full_power = self.charger_limit().is_some_and(|l| self.cap_v < l - 1.0);
        if filling && full_power {
            self.supply_overlap_ms = self.supply_overlap_ms.saturating_add(1);
            if self.config.battery_weak {
                self.brownout_ms = self.brownout_ms.saturating_add(1);
            }
        }
        // Charger: constant power, current-limited at low voltage, stops at its limit.
        if let Some(limit) = self.charger_limit() {
            if self.cap_v < limit {
                let i = (self.config.charger_watts / self.cap_v.max(1.0))
                    .min(self.config.charger_max_current_a);
                let before = self.cap_v;
                self.cap_v = (self.cap_v + i * dt / c).min(limit);
                self.charger_energy_j += 0.5 * c * (self.cap_v * self.cap_v - before * before);
            }
            if self.cap_v >= trip_v {
                // The last-resort primary-side trip ends the charge (E-1).
                self.charger_done = true;
            }
        }
        // The discharge paths, the bleed, the interlock, the relay and the
        // bridge: in fine steps while the shock path is busy.
        let (n, step_us) = if self.shock_path_busy() {
            let step = self.config.shock_step_us.clamp(1, 1_000);
            (1_000 / step, step)
        } else {
            (1, 1_000)
        };
        for k in 1..=n {
            self.substep(step_us, start_us + u64::from(k * step_us));
        }
        if self.cap_v < 0.01 {
            self.cap_v = 0.0;
        }
        self.max_cap_v = self.max_cap_v.max(self.cap_v);
        self.sample_sensors();
        // Heat: what was burned in the load stays inside, and the charger's
        // own losses (thermal model, parts study §4).
        let (charged, dumped) = self.heat_accounted_j;
        let heat = (self.dump_energy_j - dumped)
            + (self.charger_energy_j - charged) * self.thermal.config.charger_loss_per_j;
        self.heat_accounted_j = (self.charger_energy_j, self.dump_energy_j);
        self.thermal.step(dt, heat, self.electronics_on);
        std::mem::take(&mut self.events)
    }

    /// The converters take a new sample on their grid.
    fn sample_sensors(&mut self) {
        let t = self.now_ms.wrapping_add(self.config.sample_phase_ms);
        if t.is_multiple_of(self.config.voltage_sample_ms.max(1)) {
            self.held_v = (self.cap_v, self.cap_v);
        }
        if t.is_multiple_of(self.config.pad_sample_ms.max(1)) {
            let r = self.front_end_ohms();
            self.held_ohms = (r, r);
        }
    }

    /// The level of a toggling line at `t_us`.
    fn toggle_high(&self, t_us: u64) -> bool {
        let half = u64::from(self.config.toggle_half_period_us.max(1));
        (t_us / half).is_multiple_of(2)
    }

    /// The consent lines as the hardware sees them at `t_us`, faults applied.
    fn lines(&self, t_us: u64) -> (bool, bool, bool) {
        let cut = |line: ConsentLine| {
            self.consent_cut
                .is_some_and(|(l, at)| l == line && t_us >= at)
        };
        let mut request = self.request.is_some() || self.request_bare;
        if self.has_fault(Fault::RequestToggling) && request {
            request = self.toggle_high(t_us);
        }
        if self.has_fault(Fault::RequestStuckOn) {
            request = true;
        }
        if cut(ConsentLine::Request) {
            request = false;
        }
        let mut permit = self.permit;
        if self.has_fault(Fault::PermitToggling) && permit {
            permit = self.toggle_high(t_us);
        }
        if self.has_fault(Fault::PermitStuckOn) {
            permit = true;
        }
        if cut(ConsentLine::Permit) {
            permit = false;
        }
        let test = self.test_permit && !cut(ConsentLine::Permit);
        (request, permit, test)
    }

    /// Advance the fast parts of the plant by `dt_us`, ending at `t_us`.
    #[allow(clippy::too_many_lines)]
    fn substep(&mut self, dt_us: u32, t_us: u64) {
        let dt = f64::from(dt_us) * 1e-6;
        let c = self.farads();
        // Discharge paths (unless stuck), then the slow leakage, always.
        self.step_ovp(dt_us);
        let hw = self.config.hw;
        let forced = self.ovp_latched;
        self.path_holds.0.step(
            self.main_hold && !forced,
            hw.hold_engage_us,
            hw.hold_release_us,
            dt_us,
        );
        self.path_holds.1.step(
            self.supervisor_hold && !forced,
            hw.hold_engage_us,
            hw.hold_release_us,
            dt_us,
        );
        let stuck = self.has_fault(Fault::DischargeStuck);
        let mut g = 0.0;
        if !self.path_holds.0.held && !stuck && !self.path_a_open {
            g += 1.0 / self.config.path_a_ohms;
        }
        if !self.path_holds.1.held && !stuck && !self.path_b_open {
            g += 1.0 / self.config.path_b_ohms;
        }
        if g > 0.0 {
            let before = self.cap_v;
            self.cap_v *= (-dt * g / c).exp();
            self.dump_energy_j += 0.5 * c * (before * before - self.cap_v * self.cap_v);
        }
        self.cap_v *= (-dt / (self.config.bleed_ohms * c)).exp();

        // The consent lines; the two permit detectors hold consent for
        // their drop-out time after the clock stops (LR §1.2).
        let (request, permit_line, test_line) = self.lines(t_us);
        let hw = self.config.hw;
        let glitch = self
            .permit_glitch_at_us
            .is_some_and(|g| t_us.saturating_sub(g) < 50);
        if permit_line || glitch {
            self.permit_last_high_us = Some(t_us);
        }
        if test_line {
            self.test_last_high_us = Some(t_us);
        }
        if self
            .permit_glitch_at_us
            .is_some_and(|g| t_us.saturating_sub(g) >= 50)
        {
            self.permit_glitch_at_us = None;
        }
        let held = |last: Option<u64>| {
            last.is_some_and(|t| t_us.saturating_sub(t) <= u64::from(hw.permit_dropout_us))
        };
        let permit = permit_line || held(self.permit_last_high_us);
        let test_permit = test_line || held(self.test_last_high_us);

        let status = self.latch.status;
        let rail_age = self.rail_since_us.map_or(0, |t| t_us.saturating_sub(t));
        let rail_up = self.latch.rail_up();
        let armed_line = rail_up && rail_age >= u64::from(self.config.rail_rise_us);
        let rdy = rail_up && rail_age >= u64::from(hw.rail_ready_us);
        let at_rest = self.relay.front_closed && !self.has_fault(Fault::AuxContactStuckOpen);

        // The main controller's board layer: the delivery sequencer, run by
        // its timer (stops if the main controller freezes).
        let frozen = self.has_fault(Fault::MainFreeze);
        // A request raised on the plant directly (bare-plant tests) loads
        // the board layer's sequencer as `MainView::request_shock` does.
        if self.sequencer.is_none() && !frozen {
            if let Some(p) = self.request {
                let job = if p.charge_voltage.0 == 0 {
                    SequencerJob::Test(p)
                } else {
                    SequencerJob::Shock(p)
                };
                self.load_sequencer(job);
            } else if self.request_bare {
                self.load_sequencer(SequencerJob::RailOnly);
            }
        }
        let main_empty = self.config.firmware.reads_empty(self.main_volts());
        if let (Some(seq), false) = (self.sequencer.as_mut(), frozen) {
            seq.step(
                SequencerLines {
                    armed: armed_line,
                    relay_at_rest: at_rest,
                    reads_empty: main_empty,
                },
                dt_us,
            );
            self.seq_out = seq.outputs();
        } else if self.sequencer.is_none() {
            self.seq_out = SequencerOutputs::default();
        }
        let mut out = self.seq_out;
        if status == SequenceStatus::Active && !self.latch.test {
            if self.has_fault(Fault::MainTimerStuck) {
                out.cmd_ha = true;
                out.cmd_lb = true;
                out.cmd_la = false;
            } else if self.has_fault(Fault::MainTimerToggling) {
                out.cmd_ha = true;
                out.cmd_lb = self.toggle_high(t_us);
            }
        }

        // Sheet 12 (§B13.3): coil enable, relay-transferred, coil window,
        // coil cap, test relay, lead, rail and idle monitors.
        let test = self.latch.test;
        let done_1ms = self
            .sheet12
            .done_at_us
            .is_some_and(|d| t_us.saturating_sub(d) >= 1_000);
        let k_en = out.relay_cmd && rdy && !test && !self.sheet12.coil_cap_expired && !done_1ms;
        match (k_en, self.sheet12.k_en_since_us) {
            (true, None) => {
                self.sheet12.k_en_since_us = Some(t_us);
                self.sheet12.k_was_rest = at_rest;
                let full = self
                    .coil_supply_since_us
                    .is_some_and(|c| t_us.saturating_sub(c) >= u64::from(hw.reservoir_fill_us));
                self.relay.transfer_us =
                    if !self.coil_supply || self.has_fault(Fault::CoilSupplyDead) {
                        None
                    } else if full {
                        Some(hw.relay_transfer_us)
                    } else {
                        self.relay_slow_starts = self.relay_slow_starts.saturating_add(1);
                        Some(hw.relay_transfer_slow_us)
                    };
            }
            (true, Some(since)) => {
                if t_us.saturating_sub(since) >= u64::from(hw.coil_cap_us) {
                    self.sheet12.coil_cap_expired = true;
                }
            }
            (false, Some(_)) => self.sheet12.k_en_since_us = None,
            (false, None) => {}
        }
        let k_en_age = self.sheet12.k_en_since_us.map(|t| t_us.saturating_sub(t));
        if at_rest {
            self.sheet12.left_rest_since_us = None;
        } else if rail_up {
            self.sheet12.left_rest_since_us.get_or_insert(t_us);
        }
        let xfer = self.sheet12.k_was_rest
            && self
                .sheet12
                .left_rest_since_us
                .is_some_and(|l| t_us.saturating_sub(l) >= u64::from(hw.xfer_us));
        let window = k_en_age.is_some_and(|a| a <= u64::from(hw.coil_window_us));
        let btest = out.btest_cmd && test && rail_up;
        if btest {
            self.sheet12.btest_since_us.get_or_insert(t_us);
        } else {
            self.sheet12.btest_since_us = None;
        }
        let btok = self
            .sheet12
            .btest_since_us
            .is_some_and(|b| t_us.saturating_sub(b) >= u64::from(hw.btok_us));
        self.sheet12.ha_on_us = if out.cmd_ha {
            self.sheet12.ha_on_us.saturating_add(dt_us)
        } else {
            0
        };
        let lead_ok = self.sheet12.ha_on_us >= 50 || self.has_fault(Fault::LeadGateStuck);
        let lsd_ok = !self.has_fault(Fault::LsdOkStuckLow)
            && (!self.has_fault(Fault::LowSideSupplyFails)
                || self.has_fault(Fault::LsdOkStuckHigh));
        // Idle current: a high side on with no low side enabled.
        let ri = self.config.waveform.internal_ohms;
        let idle_a = if self.has_fault(Fault::LowSideShorted) && (out.cmd_ha || out.cmd_hb) && rdy {
            self.cap_v / ri.max(0.1)
        } else {
            0.0
        };
        let idle_trip = rdy
            && !self.has_fault(Fault::IdleFastStuckClear)
            && (self.has_fault(Fault::IdleFastStuckTripped) || idle_a >= self.config.idle_fast_a);
        if idle_trip || (rail_up && !lsd_ok && status == SequenceStatus::Active) {
            self.sheet12.fault = true;
        }
        let start_ok = rdy
            && lsd_ok
            && lead_ok
            && if test {
                at_rest && btok
            } else {
                xfer && window && self.relay.bridge_closed
            };

        // The phase command the gates pass: phase 1 (`G_LB`) until it has
        // ended once, then only phase 2 (`G_LA`, after the gap). A second
        // `CMD_LB` is ignored (§B13.3 rule 1).
        // The start latches on the RISING edge of `CMD_LB` only, with every
        // start condition met at that edge (§B13.1 step 8): a command that
        // comes too early is refused, not delayed into a shortened pulse.
        let lb_edge = out.cmd_lb && !self.prev_cmd_lb;
        self.prev_cmd_lb = out.cmd_lb;
        if self.sequencer.is_some_and(|q| {
            matches!(
                q.state,
                aed_core::sequencer::SequencerState::Done
                    | aed_core::sequencer::SequencerState::Aborted(_)
            )
        }) && self.request.is_none()
            && !self.request_bare
        {
            self.sequencer = None;
        }
        let (drive, in_phase2) = match status {
            SequenceStatus::Armed => (lb_edge, false),
            SequenceStatus::Active if !self.latch.phase1_done => (out.cmd_lb, false),
            SequenceStatus::Active => (out.cmd_la && out.cmd_hb && !out.cmd_ha, out.cmd_la),
            _ => (false, false),
        };
        let conducting = match status {
            SequenceStatus::Active if !self.latch.phase1_done => drive && out.cmd_ha,
            SequenceStatus::Active => drive,
            _ => false,
        };
        let phases_done =
            status == SequenceStatus::Active && self.latch.phase2_us.is_some() && !out.cmd_la;
        // The load the bridge would drive this step, and the current.
        let load = self.running.and_then(|r| self.load_ohms(&r));
        let current = match load {
            Some(rl) if conducting => self.cap_v / (rl + ri),
            _ => 0.0,
        };
        let inputs = LatchInputs {
            request,
            permit,
            test_permit,
            discharge_active: !self.hold_readback().all_held(),
            below_test_voltage: self.cap_v < self.config.test_voltage_limit_v,
            relay_on_bridge: self.relay_position() == RelayPosition::Bridge,
            drive: drive && !(self.has_fault(Fault::BridgeFailsToFire) && !test),
            phases_done,
            over_current: current > self.config.over_current_a,
            in_phase2,
            hardware_fault: self.sheet12.fault,
            start_blocked: !start_ok,
        };
        if status == SequenceStatus::Armed
            && start_ok
            && drive
            && self.has_fault(Fault::BridgeFailsToFire)
            && !test
            && !self.bridge_fail_reported
        {
            self.bridge_fail_reported = true;
            self.events.push(PlantEvent::BridgeDidNotFire);
        }
        let mut next = self.latch.step(inputs, self.config.latch_timing, dt_us);
        // The first design (review R-1), or a dead sequence hold (SW5): consent
        // is continuous power; losing it mid-pulse cuts the drive there.
        let no_hold = !self.config.sequence_latch || (self.has_fault(Fault::SeqHoldDead) && !test);
        let consent_cut = no_hold
            && status == SequenceStatus::Active
            && next.status == SequenceStatus::Active
            && Latch::consent(&inputs).is_none();
        if consent_cut {
            next = Latch::IDLE;
        }
        self.latch = next;
        let plan = self
            .request
            .or_else(|| match self.sequencer.map(|q| q.job) {
                Some(SequencerJob::DryRun(p) | SequencerJob::Shock(p) | SequencerJob::Test(p)) => {
                    Some(p)
                }
                _ => None,
            });

        match (status, self.latch.status) {
            (SequenceStatus::Idle, SequenceStatus::Armed) => {
                self.rail_since_us = Some(t_us);
                self.sheet12 = Sheet12::default();
                self.events.push(PlantEvent::RailUp {
                    test: self.latch.test,
                });
            }
            (SequenceStatus::Armed, SequenceStatus::Idle) => {
                self.rail_since_us = None;
                self.events.push(PlantEvent::RailDropped);
            }
            (SequenceStatus::Armed, SequenceStatus::LockedOut) => {
                self.rail_since_us = None;
                self.events.push(PlantEvent::PermitCapTripped);
            }
            (SequenceStatus::Armed, SequenceStatus::Active) => {
                self.start_sequence(t_us, plan);
            }
            (SequenceStatus::Active, SequenceStatus::Active) => {
                // The pulse: the capacitor discharges into the load, exactly
                // exponentially over this step.
                let short = self.has_fault(Fault::OutputShort);
                if let (Some(rl), true, Some(r)) = (load, conducting, self.running.as_mut()) {
                    let tau = c * (rl + ri);
                    let before = self.cap_v;
                    self.cap_v *= (-dt / tau).exp();
                    let taken = 0.5 * c * (before * before - self.cap_v * self.cap_v);
                    let share = rl / (rl + ri);
                    if r.test {
                        r.load_j += taken * share;
                    } else if short {
                        r.record.output_peak_a = r.record.output_peak_a.max(current);
                        self.dump_energy_j += taken;
                    } else {
                        r.record.integrated_j += taken * share;
                        r.record.integrated_peak_a = r.record.integrated_peak_a.max(current);
                        r.record.output_peak_a = r.record.output_peak_a.max(current);
                    }
                }
                if let Some(r) = self.running.as_mut() {
                    r.t_us = r.t_us.saturating_add(dt_us);
                }
            }
            (SequenceStatus::Active, SequenceStatus::Ended(end)) => {
                let end = match end {
                    SequenceEnd::Completed => ShockEnd::Completed,
                    SequenceEnd::TimeCapTripped => ShockEnd::TimeCap,
                    SequenceEnd::OverCurrentTripped => ShockEnd::OverCurrent,
                    SequenceEnd::PhaseCapTripped => ShockEnd::PhaseCap,
                    SequenceEnd::HardwareFault => ShockEnd::HardwareFault,
                };
                if end == ShockEnd::OverCurrent {
                    if let Some(r) = self.running.as_mut() {
                        r.record.output_peak_a = r.record.output_peak_a.max(current);
                    }
                }
                self.finish_sequence(t_us, end);
                self.sheet12.done_at_us = Some(t_us);
            }
            (SequenceStatus::Active, SequenceStatus::Idle) if consent_cut => {
                self.finish_sequence(t_us, ShockEnd::ConsentCut);
                self.rail_since_us = None;
            }
            _ => {}
        }
        // SEQ_HOLD keeps the rail (and the coil, until done + 1 ms) up for
        // 15 ms after the end; then it falls (the latch is locked out).
        if let (SequenceStatus::Ended(_), Some(d)) = (self.latch.status, self.sheet12.done_at_us) {
            if t_us.saturating_sub(d) >= 15_000 {
                self.rail_since_us = None;
            }
        }
        let coil = k_en && self.latch.relay_coil();
        self.step_relay(coil, dt_us);
    }

    /// The hardware over-voltage latch (v0.3 C-2 … C-4, I-22): trips at its
    /// threshold on the supervisor's chain or on `OVP_TEST`; latched until
    /// a rising `OVP_RST` edge held at least the reset minimum; while set it
    /// forces both discharge paths open.
    fn step_ovp(&mut self, dt_us: u32) {
        if self.ovp_reset {
            self.ovp_reset_high_us = self.ovp_reset_high_us.saturating_add(dt_us);
        } else {
            self.ovp_reset_high_us = 0;
            self.ovp_reset_used = false;
        }
        let level_clear = self.ovp_reset && self.has_fault(Fault::OvpResetCouplingShorted);
        if self.ovp_reset
            && !self.ovp_reset_used
            && self.ovp_reset_high_us >= self.config.hw.ovp_reset_min_us
            && !self.has_fault(Fault::OvpResetDead)
        {
            self.ovp_reset_used = true;
            if self.ovp_latched {
                self.ovp_resets = self.ovp_resets.saturating_add(1);
            }
            self.ovp_latched = false;
        }
        if level_clear && !self.has_fault(Fault::OvpResetDead) {
            self.ovp_latched = false;
        }
        let blocked = self.has_fault(Fault::OvpResetSwitchShorted) || level_clear;
        let over =
            self.cap_v >= self.config.hw.ovp_trip_v && !self.has_fault(Fault::OvpComparatorDead);
        let trip = (over || self.ovp_test) && !blocked && !self.has_fault(Fault::OvpComparatorDead);
        if self.has_fault(Fault::OvpLatchOpen) {
            self.ovp_latched = trip;
        } else if trip {
            self.ovp_latched = true;
        }
    }

    /// True if the timer for this run is stuck in phase 1.
    fn timer_stuck_for(&self, r: &Running) -> bool {
        !r.test && self.has_fault(Fault::MainTimerStuck)
    }

    /// The resistance the bridge drives in this run, if anything conducts.
    fn load_ohms(&self, r: &Running) -> Option<f64> {
        if r.test {
            return Some(self.config.test_load_ohms);
        }
        if !self.relay.bridge_closed {
            return None;
        }
        if self.has_fault(Fault::OutputShort) {
            return Some(0.1);
        }
        self.pad_ohms
    }

    fn start_sequence(&mut self, t_us: u64, plan: Option<ShockPlan>) {
        let Some(plan) = plan else {
            return;
        };
        let actual = ShockPlan {
            // Clamped into the u16 range.
            charge_voltage: Volts(self.cap_v.round().clamp(0.0, f64::from(u16::MAX)) as u16),
            ..plan
        };
        let test = self.latch.test;
        let result = if test {
            None
        } else {
            self.pad_ohms
                .and_then(|r| discharge(&actual, r, &self.config.waveform))
        };
        let record = ShockRecord {
            at_ms: self.now_ms,
            start_us: t_us,
            end_us: t_us,
            rail_us: self.rail_since_us.unwrap_or(t_us),
            request_us: self.request_raised_us,
            plan,
            capacitor_v: self.cap_v,
            patient_ohms: self.pad_ohms,
            result,
            integrated_j: 0.0,
            integrated_peak_a: 0.0,
            output_peak_a: 0.0,
            final_v: self.cap_v,
            end: ShockEnd::Completed,
            rhythm: self.rhythm,
            rhythm_since_ms: self.rhythm_since_ms,
            cancel_pressed: self.cancel_pressed(),
        };
        let _ = self.timer_stuck_for(&Running {
            t_us: 0,
            record,
            test,
            load_j: 0.0,
        });
        self.running = Some(Running {
            t_us: 0,
            record,
            test,
            load_j: 0.0,
        });
        self.events.push(PlantEvent::SequenceStarted);
    }

    fn finish_sequence(&mut self, t_us: u64, end: ShockEnd) {
        let Some(r) = self.running.take() else {
            return;
        };
        if r.test {
            self.test_sequences.push(TestSequenceRecord {
                start_us: r.record.start_us,
                end_us: t_us,
                end,
                load_j: r.load_j,
            });
            return;
        }
        let record = ShockRecord {
            end_us: t_us,
            final_v: self.cap_v,
            end,
            ..r.record
        };
        self.shocks.push(record);
        self.events.push(PlantEvent::Shock(Box::new(record)));
        self.blank_until_ms = self
            .now_ms
            .saturating_add(self.config.post_shock_blanking_ms);
        self.last_shock_ms = Some(self.now_ms);
        // Only a shock that reached the patient can change the rhythm.
        if record.integrated_j > 1.0 {
            let index = self.shocks.iter().filter(|s| s.integrated_j > 1.0).count();
            if let Some(Some(next)) = index
                .checked_sub(1)
                .and_then(|i| self.after_shock.get(i))
                .copied()
            {
                self.set_rhythm(next);
            }
        }
    }

    /// The relay contacts follow the coil with their operate (chosen when
    /// the coil was energised: full reservoir, slow, or no supply), bounce
    /// and release times.
    fn step_relay(&mut self, coil: bool, dt_us: u32) {
        let slow = if self.has_fault(Fault::RelaySlow) {
            4
        } else {
            1
        };
        let no_transfer = self.has_fault(Fault::RelayFailsToTransfer);
        let no_return = self.has_fault(Fault::RelayFailsToReturn);
        let transfer = self.relay.transfer_us.map(|t| t.saturating_mul(slow));
        let break_us = transfer.map(|t| t.saturating_sub(self.config.relay_break_lead_us));
        let (bounce_us, return_us) = (
            self.config.relay_bounce_us,
            self.config.relay_return_us.saturating_mul(slow),
        );
        let r = &mut self.relay;
        if coil == r.coil {
            r.since_us = r.since_us.saturating_add(dt_us);
        } else {
            r.coil = coil;
            r.since_us = 0;
        }
        let was_front = r.front_closed;
        if r.coil {
            if let (false, Some(b), Some(t)) = (no_transfer, break_us, transfer) {
                if r.since_us >= b {
                    r.front_closed = false;
                }
                if r.since_us >= t {
                    let into = r.since_us - t;
                    // Bouncing: open and closed in turn every 250 µs.
                    r.bridge_closed = into >= bounce_us || (into / 250).is_multiple_of(2);
                }
            }
        } else if !(r.front_closed && !r.bridge_closed) && !no_return {
            // Releasing: break the bridge contact first, then make the
            // front-end contact.
            if r.since_us >= 1_000_u32.min(return_us) {
                r.bridge_closed = false;
            }
            if r.since_us >= return_us {
                r.front_closed = true;
            }
        }
        if r.bridge_closed && !r.front_closed {
            r.bridge_stable_us = r.bridge_stable_us.saturating_add(dt_us);
        } else {
            r.bridge_stable_us = 0;
        }
        let front = r.front_closed;
        let stable = r.bridge_stable_us;
        if r.bridge_closed && stable >= 500 && stable.saturating_sub(dt_us) < 500 {
            self.events.push(PlantEvent::RelayOnBridge);
        }
        if front && !was_front {
            self.relay_back_ms = Some(self.now_ms);
            self.events.push(PlantEvent::RelayBack);
        }
    }

    /// The next heart-signal sample as the pads see it (call 250 times a second).
    pub fn ecg_sample(&mut self) -> EcgSample {
        // The heart keeps beating whether or not anyone is looking.
        let heart = f64::from(self.synth.next_uv());
        if self.pad_ohms.is_none() || !self.relay.front_closed {
            // Lead-off: front ends typically rail.
            return EcgSample::MAX;
        }
        if self.now_ms < self.blank_until_ms {
            return 0;
        }
        let mut v = heart;
        if self.compressing {
            // A rounded compression artifact at about 110 per minute, with a
            // little harmonic content and rate wobble.
            let hz = self.compression_per_min / 60.0
                * (1.0 + 0.03 * f64::from(self.rng.next_unit() - 0.5));
            self.compression_phase += std::f64::consts::TAU * hz / 250.0;
            let p = self.compression_phase;
            v += self.compression_uv * 0.5 * (p.sin() + 0.25 * (2.0 * p).sin());
        }
        // Clamped into the i16 range.
        v.round()
            .clamp(f64::from(EcgSample::MIN), f64::from(EcgSample::MAX)) as EcgSample
    }

    /// True while hands are on the chest (ground truth).
    #[must_use]
    pub fn hands_on(&self) -> bool {
        self.compressing || self.touch_until_ms.is_some_and(|t| self.now_ms < t)
    }

    /// Start or stop compressions, keeping the ground truth up to date.
    pub fn set_compressing(&mut self, on: bool) {
        self.compressing = on;
        self.update_hands_on();
    }

    /// Press on the chest (a touch, not a compression) for `ms`.
    pub fn touch(&mut self, ms: u32) {
        self.touch_started_ms = self.now_ms;
        self.touch_until_ms = Some(self.now_ms.saturating_add(ms));
        self.update_hands_on();
    }

    /// Keep [`Plant::hands_on_since`] in step with the truth (call after
    /// anything that changes it, and every step for touches that end).
    pub fn update_hands_on(&mut self) {
        match (self.hands_on(), self.hands_on_since) {
            (true, None) => self.hands_on_since = Some(self.now_ms),
            (false, Some(_)) => self.hands_on_since = None,
            _ => {}
        }
    }

    /// The next pad-impedance sample for compression sensing (call 125 times
    /// a second), or `None` if the channel has failed (no sample at all).
    ///
    /// The model: the patient's pad resistance, plus each compression (a
    /// rounded push over 40 % of the cycle and release over 60 %, in step
    /// with the compression artifact in the heart signal), each breath
    /// (1 s in, 2 s out, every 6 s), each heartbeat of a beating heart (a
    /// short bump), a touch (a press held, with 0.15 s ramps), and noise,
    /// in 1 mΩ steps. Pads off read as an open circuit (out of range).
    /// After a shock the front end is saturated briefly, then reads high
    /// and recovers. SIMULATION ONLY: shapes are cartoons.
    pub fn impedance_sample(&mut self) -> Option<ImpedanceSample> {
        if self.has_fault(Fault::ImpedanceChannelLost) {
            return None;
        }
        let noise = self.config.impedance_noise_ohms * f64::from(self.imp_rng.next_gaussian());
        if !self.relay.front_closed {
            // The pads are on the bridge: the front end sees nothing it can measure.
            return Some(ImpedanceSample::Saturated);
        }
        let Some(base) = self.pad_ohms else {
            let open = f64::from(self.config.open_circuit_reading) * 1000.0;
            return Some(ImpedanceSample::Milliohms(
                open.clamp(0.0, f64::from(u32::MAX)) as u32,
            ));
        };
        let t = f64::from(self.now_ms) / 1000.0;
        let mut z = base + noise;
        if let Some(shock) = self.last_shock_ms {
            let since = self.now_ms.saturating_sub(shock);
            if since < self.config.impedance_shock_saturation_ms {
                return Some(ImpedanceSample::Saturated);
            }
            let after = f64::from(since - self.config.impedance_shock_saturation_ms);
            z += self.config.impedance_shock_offset_ohms
                * (-after / self.config.impedance_shock_recovery_ms).exp();
        }
        if self.compressing {
            let p = (self.compression_phase / std::f64::consts::TAU).rem_euclid(1.0);
            let shape = if p < 0.4 {
                0.5 * (1.0 - (std::f64::consts::PI * p / 0.4).cos())
            } else {
                0.5 * (1.0 + (std::f64::consts::PI * (p - 0.4) / 0.6).cos())
            };
            z += self.compression_ohms * shape;
        }
        if self.ventilating {
            let p = t.rem_euclid(6.0);
            let shape = if p < 1.0 {
                0.5 * (1.0 - (std::f64::consts::PI * p).cos())
            } else if p < 3.0 {
                0.5 * (1.0 + (std::f64::consts::PI * (p - 1.0) / 2.0).cos())
            } else {
                0.0
            };
            z += self.ventilation_ohms * shape;
        }
        if let Rhythm::Sinus { bpm, .. } = self.rhythm {
            self.cardiac_phase = (self.cardiac_phase + f64::from(bpm) / 60.0 / 125.0).fract();
            if self.cardiac_phase < 0.35 {
                z -= self.cardiac_ohms
                    * 0.5
                    * (1.0 - (std::f64::consts::TAU * self.cardiac_phase / 0.35).cos());
            }
        }
        if let Some(until) = self.touch_until_ms {
            let ramp = 150.0;
            let rise =
                (f64::from(self.now_ms.saturating_sub(self.touch_started_ms)) / ramp).min(1.0);
            let fall = if self.now_ms < until {
                1.0
            } else {
                (1.0 - f64::from(self.now_ms - until) / ramp).max(0.0)
            };
            z += self.touch_ohms * rise.min(fall);
        }
        let mohm = (z * 1000.0).round().clamp(0.0, f64::from(u32::MAX));
        Some(ImpedanceSample::Milliohms(mohm as u32))
    }
}
