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
//! SIMULATION ONLY. Every electrical figure is a model figure from simple
//! arithmetic. See the crate README for what is not modelled.

use std::collections::VecDeque;

use aed_core::interlock::{bridge_may_fire, charger_may_run, discharge_path_active};
use aed_core::{BatteryStatus, MainStatus, PadCartridge, Prompt, ShockStatus, SupervisorReport};
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
    /// Resistance of the internal discharge load, ohms. Default 2,000 Ω:
    /// with 100 µF that empties the capacitor with a 0.2 s time constant,
    /// so 1,950 V falls below 50 V in about 0.75 s. Model choice.
    pub dump_ohms: f64,
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
}

impl Default for PlantConfig {
    fn default() -> Self {
        PlantConfig {
            waveform: WaveformConfig::default(),
            charger_watts: 30.0,
            charger_max_current_a: 0.5,
            charger_runaway_volts: 2_600.0,
            dump_ohms: 2_000.0,
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
        }
    }
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

/// One shock the bridge actually fired, with its modelled result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockRecord {
    /// When it fired (simulation time, ms).
    pub at_ms: u32,
    /// The plan the main controller loaded.
    pub plan: ShockPlan,
    /// The true capacitor voltage when it fired.
    pub capacitor_v: f64,
    /// The true pad resistance, or `None` if the pads were off.
    pub patient_ohms: Option<f64>,
    /// The waveform model's result (None if the pads were off: no current flowed).
    pub result: Option<DischargeResult>,
    /// The rhythm the patient actually had at that moment.
    pub rhythm: Rhythm,
    /// When that rhythm started (simulation time, ms).
    pub rhythm_since_ms: u32,
    /// Whether the cancel control was pressed at that moment.
    pub cancel_pressed: bool,
}

impl ShockRecord {
    /// Modelled energy delivered to the patient, joules.
    #[must_use]
    pub fn delivered_j(&self) -> f64 {
        self.result.map_or(0.0, |r| r.patient_energy_j)
    }
    /// Modelled peak current, amperes.
    #[must_use]
    pub fn peak_a(&self) -> f64 {
        self.result.map_or(0.0, |r| r.peak_current_a)
    }
    /// Voltage left on the capacitor after the shock.
    #[must_use]
    pub fn residual_v(&self) -> f64 {
        self.result.map_or(self.capacitor_v, |r| r.final_capacitor_v)
    }
}

/// Something physical that happened in one plant step, for the timeline.
#[derive(Debug, Clone, PartialEq)]
pub enum PlantEvent {
    /// The bridge fired.
    Shock(Box<ShockRecord>),
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
    /// The supervisor's permit line.
    pub permit: bool,
    /// What the output stage reports.
    pub shock_status: ShockStatus,
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
            permit: false,
            shock_status: ShockStatus::Idle,
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
        discharge_path_active(self.main_hold, self.supervisor_hold)
    }

    /// True if the charger is physically able to run now: it is switched on,
    /// the battery can power it and, unless the hardware rule is switched off
    /// for a test, the discharge path is not active.
    #[must_use]
    pub fn charger_running(&self) -> bool {
        let on = self.charger_target.is_some() && self.battery != BatteryStatus::Depleted;
        if self.config.charger_gated_by_discharge {
            charger_may_run(on, self.discharge_active())
        } else {
            on
        }
    }

    /// True while the cancel control is held.
    #[must_use]
    pub fn cancel_pressed(&self) -> bool {
        self.cancel_until_ms.is_some_and(|until| self.now_ms < until)
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
            Fault::CancelPressed => self.press_cancel(1_000),
            Fault::CancelStuck => self.press_cancel(u32::MAX),
            Fault::BatteryDepleted => self.battery = BatteryStatus::Depleted,
            Fault::ChargerWeak
            | Fault::ChargerRunaway
            | Fault::DischargeStuck
            | Fault::MainFreeze
            | Fault::LinkDrop
            | Fault::BridgeFailsToFire
            | Fault::ImpedanceChannelLost => {}
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

    /// A voltage reading through a sensor, rounded and kept in range.
    #[must_use]
    pub fn read_volts(&self, sensor: SensorModel) -> Volts {
        let v = sensor.read(self.cap_v).round().clamp(0.0, f64::from(u16::MAX));
        // Clamped into the u16 range above.
        Volts(v as u16)
    }

    /// A pad-resistance reading through a sensor.
    #[must_use]
    pub fn read_ohms(&self, sensor: SensorModel) -> aed_types::Ohms {
        let v = match self.pad_ohms {
            Some(r) => sensor.read(r),
            None => f64::from(self.config.open_circuit_reading),
        };
        let v = v.round().clamp(0.0, f64::from(u16::MAX));
        aed_types::Ohms(v as u16)
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
        let target = f64::from(self.charger_target?.0) * self.config.charger_set_point_gain;
        if self.battery == BatteryStatus::Depleted {
            return None;
        }
        if !self.charger_running() {
            return None;
        }
        if self.has_fault(Fault::ChargerRunaway) {
            Some(self.config.charger_runaway_volts)
        } else if self.has_fault(Fault::ChargerWeak) {
            Some(target * 0.6)
        } else {
            Some(target)
        }
    }

    /// Advance the physics by one millisecond. Returns what happened.
    pub fn step(&mut self) -> Vec<PlantEvent> {
        self.now_ms = self.now_ms.saturating_add(1);
        let dt = 1e-3;
        let c = self.farads();

        // Charger: constant power, current-limited at low voltage, stops at its limit.
        if let Some(limit) = self.charger_limit() {
            if self.cap_v < limit {
                let i = (self.config.charger_watts / self.cap_v.max(1.0))
                    .min(self.config.charger_max_current_a);
                let before = self.cap_v;
                self.cap_v = (self.cap_v + i * dt / c).min(limit);
                self.charger_energy_j += 0.5 * c * (self.cap_v * self.cap_v - before * before);
            }
        }
        // Discharge path (unless stuck), then the slow leakage, always.
        if self.discharge_active() && !self.has_fault(Fault::DischargeStuck) {
            let before = self.cap_v;
            self.cap_v *= (-dt / (self.config.dump_ohms * c)).exp();
            self.dump_energy_j += 0.5 * c * (before * before - self.cap_v * self.cap_v);
        }
        self.cap_v *= (-dt / (self.config.bleed_ohms * c)).exp();
        if self.cap_v < 0.01 {
            self.cap_v = 0.0;
        }

        // The hardware interlock and the bridge.
        let may_fire = bridge_may_fire(self.request.is_some(), self.permit, self.discharge_active());
        if may_fire && self.shock_status == ShockStatus::Pending {
            if self.has_fault(Fault::BridgeFailsToFire) {
                if !self.bridge_fail_reported {
                    self.bridge_fail_reported = true;
                    self.events.push(PlantEvent::BridgeDidNotFire);
                }
            } else if let Some(plan) = self.request {
                self.fire(plan);
            }
        }
        self.max_cap_v = self.max_cap_v.max(self.cap_v);
        std::mem::take(&mut self.events)
    }

    /// Deliver the shock into the patient using the waveform model.
    fn fire(&mut self, plan: ShockPlan) {
        // The model starts from the voltage actually on the capacitor, which
        // can differ a little from the plan's nominal charge voltage.
        let actual = ShockPlan {
            charge_voltage: Volts(self.cap_v.round().clamp(0.0, f64::from(u16::MAX)) as u16),
            ..plan
        };
        let result = self
            .pad_ohms
            .and_then(|r| discharge(&actual, r, &self.config.waveform));
        let record = ShockRecord {
            at_ms: self.now_ms,
            plan,
            capacitor_v: self.cap_v,
            patient_ohms: self.pad_ohms,
            result,
            rhythm: self.rhythm,
            rhythm_since_ms: self.rhythm_since_ms,
            cancel_pressed: self.cancel_pressed(),
        };
        if let Some(r) = result {
            self.cap_v = r.final_capacitor_v;
        }
        self.shock_status = ShockStatus::Delivered;
        self.shocks.push(record);
        self.events.push(PlantEvent::Shock(Box::new(record)));
        self.blank_until_ms = self
            .now_ms
            .saturating_add(self.config.post_shock_blanking_ms);
        self.last_shock_ms = Some(self.now_ms);
        // Only a shock that reached the patient can change the rhythm.
        if result.is_some() {
            let index = self.shocks.iter().filter(|s| s.result.is_some()).count();
            if let Some(Some(next)) = index
                .checked_sub(1)
                .and_then(|i| self.after_shock.get(i))
                .copied()
            {
                self.set_rhythm(next);
            }
        }
    }

    /// The next heart-signal sample as the pads see it (call 250 times a second).
    pub fn ecg_sample(&mut self) -> EcgSample {
        // The heart keeps beating whether or not anyone is looking.
        let heart = f64::from(self.synth.next_uv());
        if self.pad_ohms.is_none() {
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
            let hz = self.compression_per_min / 60.0 * (1.0 + 0.03 * f64::from(self.rng.next_unit() - 0.5));
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
        let Some(base) = self.pad_ohms else {
            let open = f64::from(self.config.open_circuit_reading) * 1000.0;
            return Some(ImpedanceSample::Milliohms(open.clamp(0.0, f64::from(u32::MAX)) as u32));
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
            let rise = (f64::from(self.now_ms.saturating_sub(self.touch_started_ms)) / ramp).min(1.0);
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
