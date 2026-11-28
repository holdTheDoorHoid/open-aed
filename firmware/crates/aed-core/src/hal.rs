// SPDX-License-Identifier: GPL-3.0-or-later
//! What each controller can see and do, described as two traits.
//!
//! `aed-core` never touches a pin. It asks the hardware for readings and
//! gives it commands through these two traits. The real implementations will
//! live in a board-support layer once a processor is chosen; for now the only
//! implementations are the test mocks in this crate and the simulator.
//!
//! The two traits are deliberately separate. The supervisor has its own
//! clock, its own voltage sensor, its own pad-resistance reading and its own
//! view of the cancel button. It can see nothing the main controller sees
//! except the status message the main controller sends it.
//!
//! **Discharge by default.** The storage capacitor is connected to the
//! internal discharge load unless BOTH controllers are actively holding the
//! discharge path open. Either one letting go empties the capacitor, and the
//! charger cannot run while the discharge path is active (a hardware
//! requirement). See [`crate::interlock`].

use crate::heat::{DeciC, SensorSet};
use crate::link::{MainStatus, SupervisorReport};
use crate::prompts::Prompt;
use aed_types::{Millis, Ohms, PatientMode, ShockPlan, Volts};

/// State of the battery as the power monitor reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    /// Enough charge for a full rescue.
    Good,
    /// Low: the device keeps working but says so.
    Low,
    /// Too low to charge the capacitor: a fault.
    Depleted,
}

/// State of the pad set plugged into the device (pads stay plugged in).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadCartridge {
    /// Pads present and within date.
    Ok,
    /// No pads plugged in.
    Missing,
    /// Pads plugged in but past their use-by date.
    Expired,
    /// Pads plugged in that have used up their heat allowance (D-52). The
    /// board reports this when [`crate::Standby::pads_heat_worn`] says so.
    /// Advisory in a rescue, like expired pads.
    HeatWorn,
}

/// What the shock hardware's sequence logic reports (decision D-59). Each
/// controller reads it on its own input; neither relies on the other's view.
///
/// One shock is one **sequence**: both consents present, then the
/// interlock rail rises, the patient relay moves the pads from the
/// measuring front ends to the output bridge, the main controller's timer
/// runs the phases, and at the first conducting moment the hardware latches
/// "sequence active". From then on it completes the planned pulse whatever
/// the request and permit lines do, limited only by a non-retriggerable
/// hardware time cap and the over-current trip. When it ends, drive is
/// removed, the relay returns the pads to the front ends, and the hardware
/// locks out until both consents have been fully removed: one sequence per
/// permit. The rules are written out in [`crate::interlock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceStatus {
    /// No sequence: rail down, relay at rest (pads on the front ends), not
    /// locked out.
    Idle,
    /// Both consents present and the rail up; the relay is moving or has
    /// moved the pads to the bridge; no current has flowed yet. Removing a
    /// consent now still stops everything.
    Armed,
    /// The sequence latch is set: the pulse is running and will complete
    /// whatever the consent lines do.
    Active,
    /// The sequence has ended, in this way. Drive is removed and the relay
    /// is returning (or has returned) the pads to the front ends. Locked
    /// out until both consents are removed.
    Ended(SequenceEnd),
    /// The rail was removed before any sequence started because the
    /// hardware's own limit on how long a permit may hold the rail up ran
    /// out. Locked out until both consents are removed.
    LockedOut,
}

impl SequenceStatus {
    /// True once current has flowed in this sequence (active or ended).
    #[must_use]
    pub const fn started(self) -> bool {
        matches!(self, SequenceStatus::Active | SequenceStatus::Ended(_))
    }

    /// True while the hardware refuses a new sequence until both consents
    /// have been removed.
    #[must_use]
    pub const fn locked_out(self) -> bool {
        matches!(self, SequenceStatus::Ended(_) | SequenceStatus::LockedOut)
    }
}

/// How a sequence ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceEnd {
    /// The last planned phase ended normally.
    Completed,
    /// The non-retriggerable hardware time cap expired first and cut the
    /// drive (for example a stuck phase timer).
    TimeCapTripped,
    /// The over-current trip cut the drive.
    OverCurrentTripped,
    /// A per-phase hardware cap cut the drive (D-63): phase 1 or phase 2
    /// ran longer than its own limit (for example a main controller that
    /// froze with a low side commanded on).
    PhaseCapTripped,
    /// Another hardware fault latch ended it (v0.3): idle current (a
    /// shorted low side), the bridge supply under-voltage, or the low-side
    /// driver supply monitor (`LSD_OK`).
    HardwareFault,
}

/// The four "held" sensors of the two discharge paths (design v0.3, change
/// I-4): two per path, each on its own line, read back by the main
/// controller (`HOLD_RB_A_N`, `HOLD_RB_A2_N`, `HOLD_RB_B_N`,
/// `HOLD_RB_B2_N`). `true` means that sensor reads "held" (path open, not
/// discharging). The hardware's `HOLD_OK` needs all four. The two sensors
/// of one path can disagree (one has failed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HoldReadback {
    /// Path A (the main controller's), first sensor.
    pub a1: bool,
    /// Path A, second sensor.
    pub a2: bool,
    /// Path B (the supervisor's), first sensor.
    pub b1: bool,
    /// Path B, second sensor.
    pub b2: bool,
}

impl HoldReadback {
    /// All four read "held".
    #[must_use]
    pub const fn all_held(self) -> bool {
        self.a1 && self.a2 && self.b1 && self.b2
    }
    /// Both sensors of path A read "held".
    #[must_use]
    pub const fn a_held(self) -> bool {
        self.a1 && self.a2
    }
    /// Both sensors of path A read "not held".
    #[must_use]
    pub const fn a_released(self) -> bool {
        !self.a1 && !self.a2
    }
    /// Both sensors of path B read "held".
    #[must_use]
    pub const fn b_held(self) -> bool {
        self.b1 && self.b2
    }
    /// Both sensors of path B read "not held".
    #[must_use]
    pub const fn b_released(self) -> bool {
        !self.b1 && !self.b2
    }
    /// None of the four reads "held".
    #[must_use]
    pub const fn none_held(self) -> bool {
        self.a_released() && self.b_released()
    }
}

/// A self-test request that is not an ordinary sequence (D-62, HARDWARE_DESIGN
/// §B14.5), run by the delivery sequencer under the REAL permit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// T-3: move the patient relay to the bridge and hold it there for
    /// this many ms; command no switch.
    RelayMove {
        /// How long `RELAY_CMD` is held, ms.
        hold_ms: u32,
    },
    /// T-4: move the relay, then command a low side WITHOUT its high-side
    /// lead. The hardware must refuse the start.
    StartWithoutLead,
    /// T-5: a full shock-mode sequence (1 ms, 0.5 ms gap, 1 ms) with the
    /// pads read open by both controllers, only while the main
    /// controller's own reading counts as empty.
    DryRun,
}

/// Where the patient relay's position sense says the pads are connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayPosition {
    /// At rest: pads on the heart-signal and pad-resistance front ends.
    /// The only position in which pad and heart readings mean anything.
    FrontEnds,
    /// Pads on the output bridge.
    Bridge,
    /// Neither contact confirmed: moving, bouncing, or a sense fault.
    Between,
}

/// One step of the bridge switch test (D-61), run only under a test permit,
/// at low voltage, into the device's internal test load (never the pads:
/// the patient relay cannot move under a test permit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeTestStep {
    /// Low-side switch of leg A.
    LowSideA,
    /// Low-side switch of leg B.
    LowSideB,
    /// High-side switch of leg A.
    HighSideA,
    /// High-side switch of leg B.
    HighSideB,
}

impl BridgeTestStep {
    /// Every step, in the order the self-test runs them.
    pub const ALL: [BridgeTestStep; 4] = [
        BridgeTestStep::LowSideA,
        BridgeTestStep::LowSideB,
        BridgeTestStep::HighSideA,
        BridgeTestStep::HighSideB,
    ];
}

/// What one bridge test step found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeTestOutcome {
    /// The switch turned on and off as commanded.
    Pass,
    /// Commanded on, it did not conduct (failed open, or its drive failed).
    DoesNotTurnOn,
    /// Commanded off, it still conducted (shorted, or its drive is stuck on).
    DoesNotTurnOff,
    /// The hardware could not run the step: the test rail was not up (no
    /// test permit, capacitor above the test limit) or a sequence was
    /// running.
    Refused,
}

/// The result of one bridge test step, as the board layer measures it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeTestResult {
    /// Pass or the kind of failure.
    pub outcome: BridgeTestOutcome,
    /// The board layer's measurement for the record (for example the
    /// test-load current in milliamperes). Its meaning is set by the board
    /// layer; the firmware only records it.
    pub measurement: u16,
}

/// Everything the MAIN controller can see and do.
pub trait MainHardware {
    /// The main controller's clock.
    fn now(&self) -> Millis;
    /// Resistance between the pads right now. Pads that are off read very high.
    fn pad_resistance(&mut self) -> Ohms;
    /// Capacitor voltage on the main controller's sensor.
    fn capacitor_voltage(&mut self) -> Volts;
    /// Turn the charger on, charging (and holding) to `Some(target)`, or off with `None`.
    fn set_charger(&mut self, target: Option<Volts>);
    /// Hold the discharge path open (`true`, capacitor may stay charged) or
    /// release it (`false`, capacitor empties into the internal load).
    ///
    /// Board-layer rules from design v0.3 (change order FW-11, rev3
    /// simulation notes §2): the hold clock `HOLD_M_CLK` runs at
    /// **1.5 kHz ± 10 %** (outside about 1.35–1.65 kHz an aged sensor in the
    /// cold reads "held" only on the ripple peaks) and is always stopped
    /// **low** (stopped high, the path does not release for more than
    /// 30 ms). "Held" may take up to 35 ms after the clock starts and "not
    /// held" up to 25 ms after it stops; see [`MainHardware::hold_readback`].
    fn set_discharge_hold(&mut self, hold: bool);
    /// `CHG_DONE_N` read back: the charger chip has ended a charge by
    /// ITSELF. The firmware stops every normal charge on its own reading,
    /// so this happens only when the charger's last-resort primary-side
    /// over-voltage trip fired (2,116–2,316 V simulated, change E-1). The
    /// firmware then empties the capacitor and never restarts the charger
    /// (it would refill to that level, beyond the capacitor's short surge
    /// rating, D-65).
    fn charger_done(&mut self) -> bool;
    /// The four "held" sensors (v0.3, I-4). See [`HoldReadback`].
    fn hold_readback(&mut self) -> HoldReadback;
    /// Raise (`true`) or lower (`false`) `K_SUPPLY_EN`, the patient-relay
    /// coil supply and its reservoir (change order FW-12, I-15). It must be
    /// up at least [`crate::Config::coil_supply_lead_ms`] before the relay
    /// is commanded: without it the relay does not move, and with a
    /// reservoir not yet full it moves slowly and loads the battery.
    fn set_coil_supply(&mut self, on: bool);
    /// Raise the request line for a self-test probe of the real-permit path
    /// (D-62). The board layer's delivery sequencer runs it. Withdrawn
    /// with [`MainHardware::withdraw_shock_request`].
    fn request_probe(&mut self, probe: Probe);
    /// Raise the shock request line. With `Some(plan)` the plan's phase
    /// timings are loaded into the main controller's phase timer, which the
    /// hardware starts by itself once the interlock rail is up and the
    /// patient relay reads [`RelayPosition::Bridge`] (or, under a test
    /// permit, once the rail is up). With `None` the line is raised with no
    /// sequence loaded: the self-test uses that to probe the interlock and
    /// to hold the test rail up for the bridge switch steps. Called once per
    /// shock.
    fn request_shock(&mut self, plan: Option<&ShockPlan>);
    /// Lower the shock request line. Safe to call at any time, repeatedly.
    /// Has no effect on a sequence the hardware has already latched (D-59).
    fn withdraw_shock_request(&mut self);
    /// The hardware's sequence status, read from the bridge sheet directly
    /// (not through the supervisor).
    fn sequence_status(&mut self) -> SequenceStatus;
    /// The patient relay's position sense.
    fn patient_relay(&mut self) -> RelayPosition;
    /// Run one bridge switch test step (D-61) and report the result. Only
    /// meaningful while the test rail is up: request raised with no
    /// sequence (`request_shock(None)`) under the supervisor's test permit.
    /// Otherwise the board layer returns [`BridgeTestOutcome::Refused`].
    /// The step's drive is separate from the phase timer and must not set
    /// the sequence latch (a hardware requirement; see the crate README).
    fn bridge_test_step(&mut self, step: BridgeTestStep) -> BridgeTestResult;
    /// True while the cancel control is pressed.
    fn cancel_pressed(&mut self) -> bool;
    /// The position of the adult/child switch, from its primary contact
    /// (the one the firmware has always read).
    fn patient_mode(&mut self) -> PatientMode;
    /// The position the switch's SECOND contact reports: the one that sets
    /// the charger's hardware voltage ceiling, read back (D-60, review
    /// R-8). It should always equal [`MainHardware::patient_mode`].
    fn patient_mode_ceiling(&mut self) -> PatientMode;
    /// The battery as the power monitor sees it now.
    fn battery(&mut self) -> BatteryStatus;
    /// Put the battery under a realistic load briefly and report the result
    /// (used by the self-test).
    fn battery_under_load(&mut self) -> BatteryStatus;
    /// State of the plugged-in pad set (presence and use-by date).
    fn pad_cartridge(&mut self) -> PadCartridge;
    /// The main controller's own temperature readings (D-48): both cells,
    /// the capacitor, next to the pads and the board. The supervisor's slot
    /// ([`crate::heat::TempPoint::Supervisor`]) is left `None`; its reading
    /// arrives in the supervisor's report. `None` where a sensor did not
    /// answer.
    fn temperatures(&mut self) -> SensorSet;
    /// Speak and show a prompt.
    fn prompt(&mut self, prompt: Prompt);
    /// One beat of the CPR compression metronome (a click or tone).
    fn metronome_beat(&mut self);
    /// Set the ready / not-ready indicator.
    fn set_ready_indicator(&mut self, ready: bool);
    /// Send a status (heartbeat) message to the supervisor.
    fn send_status(&mut self, status: &MainStatus);
    /// The latest report from the supervisor not yet read, if any.
    fn supervisor_report(&mut self) -> Option<SupervisorReport>;
}

/// Everything the SUPERVISOR can see and do. It is a different chip with its
/// own sensors; nothing here is shared with [`MainHardware`].
pub trait SupervisorHardware {
    /// The supervisor's own clock.
    fn now(&self) -> Millis;
    /// Capacitor voltage on the supervisor's own, independent sensor.
    fn capacitor_voltage(&mut self) -> Volts;
    /// Pad resistance on the supervisor's own measurement.
    fn pad_resistance(&mut self) -> Ohms;
    /// The supervisor's own view of the cancel control.
    fn cancel_pressed(&mut self) -> bool;
    /// The supervisor's own temperature sensor on the capacitor body, read
    /// through its own converter (D-48, parts study R-H1), or `None` if it
    /// did not answer.
    fn temperature(&mut self) -> Option<DeciC>;
    /// Raise (`true`) or lower (`false`) the shock permit line.
    fn set_permit(&mut self, permit: bool);
    /// Raise (`true`) or lower (`false`) `OVP_TEST` (v0.3, I-22): injects a
    /// trip into the hardware over-voltage latch at any capacitor voltage.
    fn set_ovp_test(&mut self, on: bool);
    /// Drive `OVP_RST`. The latch is released by a RISING EDGE only; a line
    /// held high does not keep it clear.
    fn set_ovp_reset(&mut self, high: bool);
    /// `OVP_TRIP` read back: the hardware over-voltage latch is set (it
    /// then forces both discharge paths to release).
    fn ovp_tripped(&mut self) -> bool;
    /// Raise (`true`) or lower (`false`) the TEST permit line (D-61). It
    /// lets the interlock rail rise for the self-test only while the
    /// capacitor is below the hardware's own test-voltage limit, never
    /// powers the patient relay, and with the shock permit also present
    /// gives neither (a hardware requirement).
    fn set_test_permit(&mut self, on: bool);
    /// The hardware's sequence status, on the supervisor's own input.
    fn sequence_status(&mut self) -> SequenceStatus;
    /// Hold the discharge path open (`true`) or release it (`false`).
    fn set_discharge_hold(&mut self, hold: bool);
    /// The latest status message from the main controller not yet read, if any.
    fn receive_status(&mut self) -> Option<MainStatus>;
    /// Send a report back to the main controller.
    fn send_report(&mut self, report: &SupervisorReport);
    /// Switch the supervisor's own fault alarm on (`true`) or off (`false`).
    ///
    /// A dedicated buzzer and light, wired to the supervisor alone and
    /// labelled on the case (for example "Device fault. No shocks. Continue
    /// CPR."), so that a device whose main controller has died is never
    /// silent. The supervisor switches it on whenever it has a latched fault,
    /// and if the main controller has never sent a message since switch-on.
    fn set_fault_alarm(&mut self, on: bool);
}
