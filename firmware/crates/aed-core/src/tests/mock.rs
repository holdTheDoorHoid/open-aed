// SPDX-License-Identifier: GPL-3.0-or-later
//! Test doubles: a simulated board (capacitor, interlock, message wires) that
//! both controllers talk to through their own trait, plus fake rhythm
//! analysis and shock planning.
//!
//! The capacitor model is deliberately crude: it charges at constant power
//! (1,800 V in 8 s), empties exponentially through the internal load, and
//! keeps 20% of its voltage after a shock. It exists to exercise the logic, not to predict
//! the real circuit.

use crate::hal::{
    BatteryStatus, BridgeTestOutcome, BridgeTestResult, BridgeTestStep, MainHardware, PadCartridge,
    RelayPosition, SequenceEnd, SequenceStatus, SupervisorHardware,
};
use crate::heat::{DeciC, SensorSet, TempPoint};
use crate::interlock::{charger_may_run, discharge_path_active, Latch, LatchInputs, LatchTiming};
use crate::link::{MainStatus, SupervisorReport};
use crate::prompts::Prompt;
use crate::rescue::{Rescue, RescueState};
use crate::supervisor::Supervisor;
use crate::Config;
use aed_types::{
    ChestEvent, ChestSensor, ChestState, EcgSample, ImpedanceSample, ImpedanceSignal, Micros,
    Millis, Ohms, PatientMode, PlanError, RhythmAnalyzer, RhythmDecision, ShockPlan, ShockPlanner,
    Volts, MAX_PATIENT_OHMS, MIN_PATIENT_OHMS,
};
use std::cell::Cell;
use std::format;
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

// ----- fake rhythm analysis ---------------------------------------------------

/// Heart-signal sample values that carry the rhythm the fake analyser will report.
pub fn sample_for(r: RhythmDecision) -> EcgSample {
    match r {
        RhythmDecision::Shockable => 1000,
        RhythmDecision::NonShockable => 0,
        RhythmDecision::Indeterminate => -1000,
    }
}

fn decode(s: EcgSample) -> RhythmDecision {
    if s > 500 {
        RhythmDecision::Shockable
    } else if s < -500 {
        RhythmDecision::Indeterminate
    } else {
        RhythmDecision::NonShockable
    }
}

/// Emits a decision every `every` samples, equal to the rhythm in the latest sample.
pub struct FakeAnalyzer {
    pub every: u32,
    count: u32,
    pub last_emitted: Rc<Cell<Option<RhythmDecision>>>,
    pub resets: Rc<Cell<u32>>,
    /// Fresh windows started on running filters (D-47).
    pub restarts: Rc<Cell<u32>>,
    /// Samples passed through the filters only, not analysed (D-47).
    pub tracked: Rc<Cell<u32>>,
}

impl RhythmAnalyzer for FakeAnalyzer {
    fn push(&mut self, sample: EcgSample) -> Option<RhythmDecision> {
        self.count += 1;
        if self.count >= self.every {
            self.count = 0;
            let d = decode(sample);
            self.last_emitted.set(Some(d));
            Some(d)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.count = 0;
        self.last_emitted.set(None);
        self.resets.set(self.resets.get() + 1);
    }

    fn track(&mut self, _sample: EcgSample) {
        self.tracked.set(self.tracked.get() + 1);
    }

    fn restart_window(&mut self) {
        // Like a reset for the fake (it has no filters), but counted apart.
        self.count = 0;
        self.last_emitted.set(None);
        self.restarts.set(self.restarts.get() + 1);
    }
}

// ----- fake compression sensing ----------------------------------------------------

/// What the simulated pad-impedance channel carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardChest {
    /// Still chest.
    HandsOff,
    /// Compressions (one compression event every 545 ms).
    Compressing,
    /// Movement the sensor cannot classify.
    Unknown,
    /// Hands off, but the signal is still settling (not `Good`).
    HandsOffSettling,
    /// Front end saturated (channel problem).
    Saturated,
    /// Reading out of range (channel problem).
    OutOfRange,
}

/// Impedance samples that carry the chest state the fake sensor will report.
pub fn impedance_for(c: BoardChest) -> ImpedanceSample {
    match c {
        BoardChest::HandsOff => ImpedanceSample::Milliohms(80_000),
        BoardChest::Compressing => ImpedanceSample::Milliohms(81_000),
        BoardChest::Unknown => ImpedanceSample::Milliohms(82_000),
        BoardChest::HandsOffSettling => ImpedanceSample::Milliohms(83_000),
        BoardChest::OutOfRange => ImpedanceSample::Milliohms(300_000),
        BoardChest::Saturated => ImpedanceSample::Saturated,
    }
}

/// Reports whatever chest state the latest sample carries. Counts a
/// compression every 68 samples (545 ms) while compressing.
pub struct FakeChest {
    pub fitted: bool,
    state: ChestState,
    signal: ImpedanceSignal,
    samples_compressing: u32,
    samples_since_compression: u32,
    pub resets: Rc<Cell<u32>>,
}

impl FakeChest {
    pub fn new(fitted: bool) -> Self {
        FakeChest {
            fitted,
            state: ChestState::Unknown,
            signal: ImpedanceSignal::Missing,
            samples_compressing: 0,
            samples_since_compression: 0,
            resets: Rc::new(Cell::new(0)),
        }
    }
}

impl ChestSensor for FakeChest {
    fn fitted(&self) -> bool {
        self.fitted
    }
    fn push(&mut self, sample: ImpedanceSample) -> Option<ChestEvent> {
        let (state, signal) = match sample {
            ImpedanceSample::Saturated => (ChestState::Unknown, ImpedanceSignal::Saturated),
            ImpedanceSample::Milliohms(80_000) => (ChestState::HandsOff, ImpedanceSignal::Good),
            ImpedanceSample::Milliohms(81_000) => (ChestState::Compressing, ImpedanceSignal::Good),
            ImpedanceSample::Milliohms(83_000) => (ChestState::HandsOff, ImpedanceSignal::Settling),
            ImpedanceSample::Milliohms(300_000) => {
                (ChestState::Unknown, ImpedanceSignal::OutOfRange)
            }
            ImpedanceSample::Milliohms(_) => (ChestState::Unknown, ImpedanceSignal::Good),
        };
        self.state = state;
        self.signal = signal;
        self.samples_since_compression = self.samples_since_compression.saturating_add(1);
        if state == ChestState::Compressing {
            self.samples_compressing += 1;
            if self.samples_compressing.is_multiple_of(68) {
                let since = self.samples_since_compression * 8;
                self.samples_since_compression = 0;
                return Some(ChestEvent::Compression {
                    amplitude_mohm: 1_000,
                    since_previous_ms: since,
                });
            }
        } else {
            self.samples_compressing = 0;
        }
        None
    }
    fn state(&self) -> ChestState {
        self.state
    }
    fn signal(&self) -> ImpedanceSignal {
        self.signal
    }
    fn rate_per_min(&self) -> Option<u16> {
        (self.state == ChestState::Compressing).then_some(110)
    }
    fn ms_since_compression(&self) -> u32 {
        self.samples_since_compression * 8
    }
    fn time_on_chest_permille(&self) -> u16 {
        0
    }
    fn reset(&mut self) {
        self.state = ChestState::Unknown;
        self.signal = ImpedanceSignal::Missing;
        self.samples_compressing = 0;
        self.samples_since_compression = 0;
        self.resets.set(self.resets.get() + 1);
    }
}

/// Charges to a fixed voltage per mode; refuses resistances outside 25..175 ohms.
#[derive(Clone, Copy)]
pub struct FakePlanner {
    pub adult_v: u16,
    pub child_v: u16,
    /// If set, plans report this charge voltage instead (to test mismatch handling).
    pub plan_voltage_override: Option<u16>,
}

impl Default for FakePlanner {
    fn default() -> Self {
        FakePlanner {
            adult_v: 1800,
            child_v: 1000,
            plan_voltage_override: None,
        }
    }
}

impl ShockPlanner for FakePlanner {
    fn charge_voltage(&self, mode: PatientMode) -> Volts {
        match mode {
            PatientMode::Adult => Volts(self.adult_v),
            PatientMode::Child => Volts(self.child_v),
        }
    }

    fn plan(&self, mode: PatientMode, impedance: Ohms) -> Result<ShockPlan, PlanError> {
        if impedance < MIN_PATIENT_OHMS {
            return Err(PlanError::ImpedanceTooLow);
        }
        if impedance > MAX_PATIENT_OHMS {
            return Err(PlanError::ImpedanceTooHigh);
        }
        let p1 = 3000 + u32::from(impedance.0 - 25) * 60;
        Ok(ShockPlan {
            charge_voltage: self
                .plan_voltage_override
                .map_or(self.charge_voltage(mode), Volts),
            phase1: Micros(p1),
            interphase: Micros(400),
            phase2: Micros(p1 * 2 / 3),
            expected_energy: mode.target_energy(),
        })
    }
}

// ----- the simulated board ------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct RequestRecord {
    pub at: u32,
    pub plan: ShockPlan,
}

#[derive(Debug, Clone, Copy)]
pub struct FireRecord {
    pub at: u32,
    pub voltage: f32,
    pub main_request: bool,
    pub permit: bool,
    pub main_hold: bool,
    pub sup_hold: bool,
}

pub struct Board {
    pub now: u32,
    pub cap_v: f32,
    pub main_hold: bool,
    pub sup_hold: bool,
    pub charger: Option<Volts>,
    /// The request line raised with this plan loaded (a shock or a test
    /// sequence).
    pub request: Option<ShockPlan>,
    /// The request line raised with no sequence loaded (self-test probes).
    pub request_bare: bool,
    pub permit: bool,
    /// The supervisor's test permit line (D-61).
    pub test_permit: bool,
    /// The permit line is stuck on whatever the supervisor does.
    pub permit_stuck_on: bool,
    /// The request line is stuck on whatever the main controller does.
    pub request_stuck_on: bool,
    /// "Both held" reads true whatever the holds do (review R-6).
    pub hold_ok_stuck: bool,
    /// The hardware's test-voltage comparator limit, volts.
    pub test_limit_v: f32,
    /// The relay coil is (wrongly) powered by a test rail too.
    pub relay_on_test_rail: bool,
    /// The lockout latch does not work.
    pub no_lockout: bool,
    /// The sequence logic (D-59), stepped in 100 µs steps.
    pub latch: Latch,
    pub latch_timing: LatchTiming,
    /// Patient relay position sense.
    pub relay: RelayPosition,
    /// How long the relay coil has been powered or unpowered (µs).
    pub relay_coil_us: u32,
    pub relay_coil_was: bool,
    /// Relay operate and release times, µs.
    pub relay_transfer_us: u32,
    pub relay_return_us: u32,
    /// The relay never leaves the front ends.
    pub relay_stuck_front: bool,
    /// The relay never comes back from the bridge.
    pub relay_stuck_bridge: bool,
    /// After the relay is back, both pad readings read open for this long, ms.
    pub front_end_recovery_ms: u32,
    /// When the relay last arrived back on the front ends.
    pub relay_back_at: Option<u32>,
    /// Time into the running sequence, µs.
    pub seq_us: u32,
    /// The phase timer never starts (the bridge does not fire).
    pub bridge_no_start: bool,
    /// The phase timer never ends its first phase (stuck timer).
    pub timer_stuck: bool,
    /// What a sequence leaves on the capacitor, as a fraction.
    pub residual: f32,
    /// A bridge test step that fails, and how.
    pub bridge_switch_fault: Option<(BridgeTestStep, BridgeTestOutcome)>,
    /// What the second (hardware ceiling) contact of the mode switch reads,
    /// if it differs from `mode`.
    pub mode_ceiling: Option<PatientMode>,
    /// Path A (main's) or path B (supervisor's) discharge path failed open.
    pub path_a_broken: bool,
    pub path_b_broken: bool,
    /// Test sequences seen (start times).
    pub test_fires: Vec<u32>,
    /// True pad resistance.
    pub pad_ohms: u16,
    /// What the supervisor's own pad measurement sees, if different.
    pub sup_pad_ohms: Option<u16>,
    /// Error added to the main controller's voltage reading.
    pub main_v_offset: i32,
    /// Error added to the supervisor's voltage reading.
    pub sup_v_offset: i32,
    pub cancel: bool,
    /// What the supervisor sees of the cancel control, if different.
    pub sup_cancel: Option<bool>,
    pub mode: PatientMode,
    pub battery: BatteryStatus,
    pub battery_under_load: BatteryStatus,
    pub cartridge: PadCartridge,
    /// Charging speed: the average volts per millisecond of a charge to
    /// 1,800 V (in 8 s). The charger is modelled as constant power, so it is
    /// faster than this at low voltage. A runaway charger climbs at exactly
    /// this rate.
    pub charge_rate: f32,
    pub charger_broken: bool,
    /// Charger ignores its target and keeps going, and once started it
    /// also ignores the firmware's "off" (since v0.3 the firmware stops
    /// every normal charge itself, FW-13): only the discharge-path gating
    /// stops it.
    pub charger_runaway: bool,
    /// The runaway charger has been started.
    pub runaway_on: bool,
    /// Internal discharge load does not work.
    pub dump_broken: bool,
    /// The over-current trip ends every sequence.
    pub output_fault: bool,
    pub status_wire_up: bool,
    pub report_wire_up: bool,
    pub status_inbox: Option<MainStatus>,
    pub report_inbox: Option<SupervisorReport>,
    pub statuses: Vec<MainStatus>,
    pub prompts: Vec<(u32, Prompt)>,
    pub beats: Vec<u32>,
    pub ready: bool,
    /// The supervisor's own fault alarm.
    pub sup_alarm: bool,
    pub requests: Vec<RequestRecord>,
    pub fires: Vec<FireRecord>,
    pub violations: Vec<String>,
    /// The rhythm the heart signal carries.
    pub rhythm: RhythmDecision,
    /// The fake analyser's latest emitted decision (shared with it).
    pub last_decision: Rc<Cell<Option<RhythmDecision>>>,
    /// What the pad-impedance channel carries (compression sensing).
    pub chest: BoardChest,
    /// The impedance channel delivers samples.
    pub impedance_up: bool,
    /// Compression sensing is fitted and switched on (for the invariant
    /// "no shock request unless the chest is still").
    pub chest_sensing: bool,
    /// When `chest` last changed (set through [`Board::set_chest`]).
    pub chest_since: u32,
    /// The main controller was using compression sensing as of its last
    /// tick (it stops after a channel failure, by design).
    pub chest_in_use: bool,
    /// What the main controller's five temperature sensors read (the
    /// supervisor slot is ignored).
    pub temps: SensorSet,
    /// What the supervisor's own temperature sensor reads.
    pub sup_temp: Option<DeciC>,
}

impl Board {
    pub fn new(last_decision: Rc<Cell<Option<RhythmDecision>>>) -> Self {
        Board {
            now: 0,
            cap_v: 0.0,
            main_hold: false,
            sup_hold: false,
            charger: None,
            request: None,
            request_bare: false,
            permit: false,
            test_permit: false,
            permit_stuck_on: false,
            request_stuck_on: false,
            hold_ok_stuck: false,
            test_limit_v: 100.0,
            relay_on_test_rail: false,
            no_lockout: false,
            latch: Latch::IDLE,
            latch_timing: LatchTiming::PROPOSED,
            relay: RelayPosition::FrontEnds,
            relay_coil_us: 0,
            relay_coil_was: false,
            relay_transfer_us: 4_000,
            relay_return_us: 4_000,
            relay_stuck_front: false,
            relay_stuck_bridge: false,
            front_end_recovery_ms: 0,
            relay_back_at: None,
            seq_us: 0,
            bridge_no_start: false,
            timer_stuck: false,
            residual: 0.2,
            bridge_switch_fault: None,
            mode_ceiling: None,
            path_a_broken: false,
            path_b_broken: false,
            test_fires: Vec::new(),
            pad_ohms: 80,
            sup_pad_ohms: None,
            main_v_offset: 0,
            sup_v_offset: 0,
            cancel: false,
            sup_cancel: None,
            mode: PatientMode::Adult,
            battery: BatteryStatus::Good,
            battery_under_load: BatteryStatus::Good,
            cartridge: PadCartridge::Ok,
            charge_rate: 0.225,
            charger_broken: false,
            charger_runaway: false,
            runaway_on: false,
            dump_broken: false,
            output_fault: false,
            status_wire_up: true,
            report_wire_up: true,
            status_inbox: None,
            report_inbox: None,
            statuses: Vec::new(),
            prompts: Vec::new(),
            beats: Vec::new(),
            ready: false,
            sup_alarm: false,
            requests: Vec::new(),
            fires: Vec::new(),
            violations: Vec::new(),
            rhythm: RhythmDecision::Shockable,
            last_decision,
            chest: BoardChest::HandsOff,
            impedance_up: true,
            chest_sensing: false,
            chest_since: 0,
            chest_in_use: false,
            temps: SensorSet::uniform(DeciC::from_c(25)),
            sup_temp: Some(DeciC::from_c(25)),
        }
    }

    /// Every temperature sensor, main and supervisor, reads `c` °C.
    pub fn set_all_temps(&mut self, c: i16) {
        self.temps = SensorSet::uniform(DeciC::from_c(c));
        self.sup_temp = Some(DeciC::from_c(c));
    }

    /// Change what the chest channel carries, remembering when.
    pub fn set_chest(&mut self, c: BoardChest) {
        if c != self.chest {
            self.chest = c;
            self.chest_since = self.now;
        }
    }

    pub fn discharging(&self) -> bool {
        discharge_path_active(self.main_hold, self.sup_hold)
    }

    /// The permit line as the hardware sees it.
    pub fn permit_line(&self) -> bool {
        self.permit || self.permit_stuck_on
    }

    /// True while pad readings mean nothing: relay off the front ends, or
    /// the front ends still recovering.
    pub fn front_ends_blind(&self) -> bool {
        self.relay != RelayPosition::FrontEnds
            || self
                .relay_back_at
                .is_some_and(|t| self.now < t + self.front_end_recovery_ms)
    }

    /// Advance the physics by `dt` milliseconds.
    pub fn step(&mut self, dt: u32) {
        let dt_f = dt as f32;
        if self.discharging() {
            // Two paths side by side (path A let go by the main controller,
            // path B by the supervisor). Crude on purpose, as before the
            // split: any working path that is let go empties with a 100 ms
            // time constant; a broken path does not conduct (only the slow
            // bleed remains).
            let a = !self.main_hold && !self.path_a_broken && !self.dump_broken;
            let b = !self.sup_hold && !self.path_b_broken && !self.dump_broken;
            let tau = if a || b { 100.0 } else { 600_000.0 };
            self.cap_v *= (-dt_f / tau).exp();
            if self.cap_v < 0.5 {
                self.cap_v = 0.0;
            }
        }
        // The charger runs only while the discharge path is not active.
        if self.charger.is_some() && self.charger_runaway {
            self.runaway_on = true;
        }
        let stuck = (self.charger_runaway && self.runaway_on).then_some(Volts(u16::MAX));
        if let Some(target) = self
            .charger
            .or(stuck)
            .filter(|_| charger_may_run(true, self.discharging()))
        {
            if !self.charger_broken {
                let t = f32::from(target.0);
                if self.charger_runaway {
                    self.cap_v += self.charge_rate * dt_f;
                } else if self.cap_v < t {
                    // Constant power, as a real flyback charger: fast at low
                    // voltage (current-limited to 5 V/ms), slower near the
                    // top. 1,800 V in 8 s, 1,000 V in about 2.5 s, 400 V in
                    // about 0.4 s.
                    let rate = (self.charge_rate * 900.0 / self.cap_v.max(1.0)).min(5.0);
                    self.cap_v = (self.cap_v + rate * dt_f).min(t);
                }
            }
        }
        // The sequence logic and the relay, in 100 µs steps while anything
        // is happening; one step when everything is at rest.
        let quiet = self.latch.status == SequenceStatus::Idle
            && self.relay == RelayPosition::FrontEnds
            && !self.relay_coil_was
            && self.request.is_none()
            && !self.request_bare
            && !self.request_stuck_on
            && !self.permit_line()
            && !self.test_permit;
        if quiet {
            self.step_sequence(dt * 1_000);
        } else {
            for _ in 0..dt * 10 {
                self.step_sequence(100);
            }
        }
    }

    fn step_sequence(&mut self, dt_us: u32) {
        let plan = self.request;
        let armed = self.latch.status == SequenceStatus::Armed;
        let active = self.latch.status == SequenceStatus::Active;
        let total_us = plan.map_or(0, |p| p.phase1.0 + p.interphase.0 + p.phase2.0);
        let ready = self.latch.test || self.relay == RelayPosition::Bridge;
        // The phase timer: phase 1, the gap (no drive), phase 2 (D-63: each
        // phase has its own hardware cap, so the gap must be modelled).
        let (p1, p2_start) = plan.map_or((0, 0), |p| (p.phase1.0, p.phase1.0 + p.interphase.0));
        let in_phase2 = active && !self.timer_stuck && self.seq_us >= p2_start;
        let timing = self.timer_stuck || self.seq_us < p1 || in_phase2;
        let drive =
            !self.bridge_no_start && plan.is_some() && ((armed && ready) || (active && timing));
        let done = active && !self.timer_stuck && self.seq_us >= total_us;
        let inputs = LatchInputs {
            request: self.request.is_some() || self.request_bare || self.request_stuck_on,
            permit: self.permit_line(),
            test_permit: self.test_permit,
            discharge_active: self.discharging() && !self.hold_ok_stuck,
            below_test_voltage: self.cap_v < self.test_limit_v,
            relay_on_bridge: self.relay == RelayPosition::Bridge,
            drive,
            phases_done: done,
            over_current: active && !self.latch.test && self.output_fault && self.seq_us > 0,
            in_phase2,
            ..LatchInputs::default()
        };
        let before = self.latch.status;
        self.latch = self.latch.step(inputs, self.latch_timing, dt_us);
        if self.no_lockout && self.latch.status.locked_out() && !inputs.request {
            // A broken lockout clears as soon as the request drops, with
            // the permit still up.
            self.latch = Latch::IDLE;
        }
        match (before, self.latch.status) {
            (SequenceStatus::Armed, SequenceStatus::Active) => {
                self.seq_us = 0;
                if self.latch.test {
                    self.test_fires.push(self.now);
                } else {
                    self.fires.push(FireRecord {
                        at: self.now,
                        voltage: self.cap_v,
                        main_request: inputs.request,
                        permit: inputs.permit,
                        main_hold: self.main_hold,
                        sup_hold: self.sup_hold,
                    });
                }
            }
            (SequenceStatus::Active, SequenceStatus::Active) => {
                self.seq_us += dt_us;
            }
            (SequenceStatus::Active, SequenceStatus::Ended(end))
                if !self.latch.test && end != SequenceEnd::OverCurrentTripped =>
            {
                self.cap_v *= self.residual;
            }
            _ => {}
        }
        // The relay follows its coil, with operate and release times.
        let coil = self.latch.relay_coil() || (self.relay_on_test_rail && self.latch.rail_up());
        if coil != self.relay_coil_was {
            self.relay_coil_was = coil;
            self.relay_coil_us = 0;
        } else {
            self.relay_coil_us += dt_us;
        }
        if coil && !self.relay_stuck_front && self.relay == RelayPosition::FrontEnds {
            if self.relay_coil_us >= 1_000 {
                self.relay = RelayPosition::Between;
            }
        } else if coil
            && self.relay == RelayPosition::Between
            && self.relay_coil_us >= self.relay_transfer_us
        {
            self.relay = RelayPosition::Bridge;
        } else if !coil && self.relay != RelayPosition::FrontEnds && !self.relay_stuck_bridge {
            if self.relay_coil_us >= self.relay_return_us {
                self.relay = RelayPosition::FrontEnds;
                self.relay_back_at = Some(self.now);
            } else if self.relay_coil_us >= 1_000 {
                self.relay = RelayPosition::Between;
            }
        }
    }

    fn reading(v: f32, offset: i32) -> Volts {
        let r = (v.round() as i32 + offset).clamp(0, i32::from(u16::MAX));
        Volts(u16::try_from(r).unwrap_or(0))
    }

    pub fn prompts_only(&self) -> Vec<Prompt> {
        self.prompts.iter().map(|&(_, p)| p).collect()
    }
}

/// The main controller's view of the board.
pub struct MainView<'a>(pub &'a mut Board);
/// The supervisor's view of the board.
pub struct SupView<'a>(pub &'a mut Board);

impl MainHardware for MainView<'_> {
    fn now(&self) -> Millis {
        Millis(self.0.now)
    }
    fn pad_resistance(&mut self) -> Ohms {
        if self.0.front_ends_blind() {
            return Ohms(10_000);
        }
        Ohms(self.0.pad_ohms)
    }
    fn capacitor_voltage(&mut self) -> Volts {
        Board::reading(self.0.cap_v, self.0.main_v_offset)
    }
    fn set_charger(&mut self, target: Option<Volts>) {
        self.0.charger = target;
    }
    fn set_discharge_hold(&mut self, hold: bool) {
        self.0.main_hold = hold;
    }
    fn hold_readback(&mut self) -> crate::hal::HoldReadback {
        let (a, b) = (self.0.main_hold, self.0.sup_hold);
        crate::hal::HoldReadback {
            a1: a,
            a2: a,
            b1: b,
            b2: b,
        }
    }
    fn set_coil_supply(&mut self, _on: bool) {}
    fn request_probe(&mut self, _probe: crate::hal::Probe) {
        self.0.request_bare = true;
    }
    fn charger_done(&mut self) -> bool {
        false
    }
    fn request_shock(&mut self, plan: Option<&ShockPlan>) {
        let b = &mut *self.0;
        let Some(plan) = plan.copied() else {
            b.request_bare = true;
            return;
        };
        if plan.charge_voltage.0 == 0 {
            // A self-test sequence into the internal load.
            b.request = Some(plan);
            return;
        }
        // Invariant checks at the moment a request is raised.
        if b.last_decision.get() != Some(RhythmDecision::Shockable) {
            b.violations.push(format!(
                "t={} shock requested but last analyser decision was {:?}",
                b.now,
                b.last_decision.get()
            ));
        }
        if !(MIN_PATIENT_OHMS.0..=MAX_PATIENT_OHMS.0).contains(&b.pad_ohms) {
            b.violations.push(format!(
                "t={} shock requested with pads at {} ohms",
                b.now, b.pad_ohms
            ));
        }
        if b.cancel {
            b.violations
                .push(format!("t={} shock requested while cancel pressed", b.now));
        }
        // One impedance sample (8 ms) and one tick (2 ms) may pass before
        // the main controller can know of a change.
        let known = b.now.saturating_sub(b.chest_since) > 10;
        if b.chest_sensing
            && b.chest_in_use
            && b.impedance_up
            && known
            && !matches!(b.chest, BoardChest::HandsOff | BoardChest::HandsOffSettling)
        {
            b.violations.push(format!(
                "t={} shock requested while the chest channel carried {:?}",
                b.now, b.chest
            ));
        }
        if b.request.is_some() {
            b.violations.push(format!(
                "t={} shock requested twice without withdrawal",
                b.now
            ));
        }
        b.requests.push(RequestRecord { at: b.now, plan });
        b.request = Some(plan);
    }
    fn withdraw_shock_request(&mut self) {
        self.0.request = None;
        self.0.request_bare = false;
    }
    fn sequence_status(&mut self) -> SequenceStatus {
        self.0.latch.status
    }
    fn patient_relay(&mut self) -> RelayPosition {
        self.0.relay
    }
    fn bridge_test_step(&mut self, step: BridgeTestStep) -> BridgeTestResult {
        let b = &*self.0;
        let outcome = if !(b.latch.status == SequenceStatus::Armed && b.latch.test) {
            BridgeTestOutcome::Refused
        } else {
            match b.bridge_switch_fault {
                Some((s, o)) if s == step => o,
                _ => BridgeTestOutcome::Pass,
            }
        };
        BridgeTestResult {
            outcome,
            measurement: 0,
        }
    }
    fn cancel_pressed(&mut self) -> bool {
        self.0.cancel
    }
    fn patient_mode(&mut self) -> PatientMode {
        self.0.mode
    }
    fn patient_mode_ceiling(&mut self) -> PatientMode {
        self.0.mode_ceiling.unwrap_or(self.0.mode)
    }
    fn battery(&mut self) -> BatteryStatus {
        self.0.battery
    }
    fn battery_under_load(&mut self) -> BatteryStatus {
        self.0.battery_under_load
    }
    fn pad_cartridge(&mut self) -> PadCartridge {
        self.0.cartridge
    }
    fn temperatures(&mut self) -> SensorSet {
        let mut t = self.0.temps;
        t.set(TempPoint::Supervisor, None);
        t
    }
    fn prompt(&mut self, prompt: Prompt) {
        let now = self.0.now;
        self.0.prompts.push((now, prompt));
    }
    fn metronome_beat(&mut self) {
        let now = self.0.now;
        self.0.beats.push(now);
    }
    fn set_ready_indicator(&mut self, ready: bool) {
        self.0.ready = ready;
    }
    fn send_status(&mut self, status: &MainStatus) {
        self.0.statuses.push(*status);
        if self.0.status_wire_up {
            self.0.status_inbox = Some(*status);
        }
    }
    fn supervisor_report(&mut self) -> Option<SupervisorReport> {
        self.0.report_inbox.take()
    }
}

impl SupervisorHardware for SupView<'_> {
    fn now(&self) -> Millis {
        Millis(self.0.now)
    }
    fn capacitor_voltage(&mut self) -> Volts {
        Board::reading(self.0.cap_v, self.0.sup_v_offset)
    }
    fn pad_resistance(&mut self) -> Ohms {
        if self.0.front_ends_blind() {
            return Ohms(10_000);
        }
        Ohms(self.0.sup_pad_ohms.unwrap_or(self.0.pad_ohms))
    }
    fn cancel_pressed(&mut self) -> bool {
        self.0.sup_cancel.unwrap_or(self.0.cancel)
    }
    fn temperature(&mut self) -> Option<DeciC> {
        self.0.sup_temp
    }
    fn set_permit(&mut self, permit: bool) {
        self.0.permit = permit;
    }
    fn set_test_permit(&mut self, on: bool) {
        self.0.test_permit = on;
    }
    fn set_ovp_test(&mut self, _on: bool) {}
    fn set_ovp_reset(&mut self, _high: bool) {}
    fn ovp_tripped(&mut self) -> bool {
        false
    }
    fn sequence_status(&mut self) -> SequenceStatus {
        self.0.latch.status
    }
    fn set_discharge_hold(&mut self, hold: bool) {
        self.0.sup_hold = hold;
    }
    fn receive_status(&mut self) -> Option<MainStatus> {
        self.0.status_inbox.take()
    }
    fn send_report(&mut self, report: &SupervisorReport) {
        if self.0.report_wire_up {
            self.0.report_inbox = Some(*report);
        }
    }
    fn set_fault_alarm(&mut self, on: bool) {
        self.0.sup_alarm = on;
    }
}

// ----- the bench: board + both controllers --------------------------------------

pub type TestRescue = Rescue<FakeAnalyzer, FakePlanner, FakeChest>;

pub struct Bench {
    pub board: Board,
    pub rescue: TestRescue,
    pub sup: Supervisor,
    pub config: Config,
    pub tick_ms: u32,
    pub main_alive: bool,
    pub sup_alive: bool,
    pub resets: Rc<Cell<u32>>,
    /// Analyser `restart_window` calls (D-47).
    pub restarts: Rc<Cell<u32>>,
    /// Samples passed to the analyser's `track` (D-47).
    pub tracked: Rc<Cell<u32>>,
    pub states: Vec<(u32, RescueState)>,
}

impl Bench {
    pub fn new() -> Self {
        Self::with(super::cfg(), FakePlanner::default())
    }

    pub fn with(config: Config, planner: FakePlanner) -> Self {
        Self::with_all(config, planner, 250)
    }

    /// `every`: how many samples between fake analyser decisions (250 = one per second).
    pub fn with_all(config: Config, planner: FakePlanner, every: u32) -> Self {
        Self::build(config, planner, every, false)
    }

    /// A bench whose device has compression sensing fitted (D-36). The
    /// chest channel starts "hands off".
    pub fn with_chest(config: Config) -> Self {
        let mut b = Self::build(config, FakePlanner::default(), 250, true);
        b.board.chest_sensing = config.chest_sensing;
        b
    }

    fn build(config: Config, planner: FakePlanner, every: u32, chest_fitted: bool) -> Self {
        let last = Rc::new(Cell::new(None));
        let resets = Rc::new(Cell::new(0));
        let restarts = Rc::new(Cell::new(0));
        let tracked = Rc::new(Cell::new(0));
        let analyzer = FakeAnalyzer {
            every,
            count: 0,
            last_emitted: last.clone(),
            resets: resets.clone(),
            restarts: restarts.clone(),
            tracked: tracked.clone(),
        };
        let rescue =
            Rescue::with_chest_sensor(config, analyzer, planner, FakeChest::new(chest_fitted))
                .expect("valid config");
        let sup = Supervisor::new(config).expect("valid config");
        Bench {
            board: Board::new(last),
            rescue,
            sup,
            config,
            tick_ms: 2,
            main_alive: true,
            sup_alive: true,
            resets,
            restarts,
            tracked,
            states: Vec::new(),
        }
    }

    /// One tick: physics, heart-signal samples, main controller, supervisor.
    pub fn step(&mut self) {
        let before = self.board.now;
        self.board.now += self.tick_ms;
        self.board.step(self.tick_ms);
        let now = self.board.now;
        if self.main_alive {
            // 250 samples per second: one every 4 ms.
            let due = now / 4 - before / 4;
            for _ in 0..due {
                self.rescue
                    .on_ecg_sample(sample_for(self.board.rhythm), Millis(now));
            }
            // 125 impedance samples per second: one every 8 ms.
            if self.board.impedance_up {
                let due = now / 8 - before / 8;
                for _ in 0..due {
                    self.rescue
                        .on_impedance_sample(impedance_for(self.board.chest), Millis(now));
                }
            }
            self.board.chest_in_use = self.rescue.chest_sensing_active();
            self.rescue.tick(&mut MainView(&mut self.board));
        }
        if self.sup_alive {
            self.sup.tick(&mut SupView(&mut self.board));
        }
        let s = self.rescue.state();
        if self.states.last().map(|&(_, x)| x) != Some(s) {
            self.states.push((now, s));
        }
    }

    pub fn run_ms(&mut self, ms: u32) {
        let end = self.board.now + ms;
        while self.board.now < end {
            self.step();
        }
    }

    /// Run until the predicate holds; false if `max_ms` passed first.
    pub fn run_until(&mut self, max_ms: u32, mut pred: impl FnMut(&Bench) -> bool) -> bool {
        let end = self.board.now + max_ms;
        while self.board.now < end {
            self.step();
            if pred(self) {
                return true;
            }
        }
        false
    }

    pub fn run_until_state(&mut self, s: RescueState, max_ms: u32) -> bool {
        self.run_until(max_ms, |b| b.rescue.state() == s)
    }

    pub fn state(&self) -> RescueState {
        self.rescue.state()
    }

    pub fn state_sequence(&self) -> Vec<RescueState> {
        self.states.iter().map(|&(_, s)| s).collect()
    }

    /// Power on and run until the pads are being analysed (pads on, self-test
    /// passed): `AnalysingCharging` by default, `Analysing` if charging
    /// during analysis is switched off.
    pub fn run_to_analysing(&mut self) {
        assert!(
            self.run_until(30_000, |b| b.rescue.state().is_analysing()),
            "never reached Analysing; states {:?}",
            self.state_sequence()
        );
    }

    /// Power on with a shockable rhythm and run until the countdown starts.
    pub fn run_to_countdown(&mut self) {
        self.board.rhythm = RhythmDecision::Shockable;
        assert!(
            self.run_until_state(RescueState::Countdown, 60_000),
            "never reached Countdown; states {:?}",
            self.state_sequence()
        );
    }

    pub fn shocks_fired(&self) -> usize {
        self.board.fires.len()
    }

    pub fn has_event(&self, pred: impl Fn(&crate::Event) -> bool) -> bool {
        self.rescue.events().iter().any(|e| pred(&e.event))
    }

    pub fn assert_no_violations(&self) {
        assert!(
            self.board.violations.is_empty(),
            "invariant violations: {:?}",
            self.board.violations
        );
    }
}
