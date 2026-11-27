// SPDX-License-Identifier: GPL-3.0-or-later
//! Test doubles: a simulated board (capacitor, interlock, message wires) that
//! both controllers talk to through their own trait, plus fake rhythm
//! analysis and shock planning.
//!
//! The capacitor model is deliberately crude: it charges at a fixed rate,
//! empties exponentially through the internal load, and keeps 20% of its
//! voltage after a shock. It exists to exercise the logic, not to predict
//! the real circuit.

use crate::hal::{BatteryStatus, MainHardware, PadCartridge, ShockStatus, SupervisorHardware};
use crate::interlock::{bridge_may_fire, charger_may_run, discharge_path_active};
use crate::link::{MainStatus, SupervisorReport};
use crate::prompts::Prompt;
use crate::rescue::{Rescue, RescueState};
use crate::supervisor::Supervisor;
use crate::Config;
use aed_types::{
    EcgSample, Micros, Millis, Ohms, PatientMode, PlanError, RhythmAnalyzer, RhythmDecision,
    ShockPlan, ShockPlanner, Volts, MAX_PATIENT_OHMS, MIN_PATIENT_OHMS,
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
    pub request: Option<ShockPlan>,
    pub permit: bool,
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
    /// Volts per millisecond (1800 V in 8 s).
    pub charge_rate: f32,
    pub charger_broken: bool,
    /// Charger ignores its target and keeps going.
    pub charger_runaway: bool,
    /// Internal discharge load does not work.
    pub dump_broken: bool,
    /// Output stage reports a fault on delivery.
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
    pub shock_status: ShockStatus,
    pub requests: Vec<RequestRecord>,
    pub fires: Vec<FireRecord>,
    pub violations: Vec<String>,
    /// The rhythm the heart signal carries.
    pub rhythm: RhythmDecision,
    /// The fake analyser's latest emitted decision (shared with it).
    pub last_decision: Rc<Cell<Option<RhythmDecision>>>,
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
            permit: false,
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
            shock_status: ShockStatus::Idle,
            requests: Vec::new(),
            fires: Vec::new(),
            violations: Vec::new(),
            rhythm: RhythmDecision::Shockable,
            last_decision,
        }
    }

    pub fn discharging(&self) -> bool {
        discharge_path_active(self.main_hold, self.sup_hold)
    }

    /// Advance the physics by `dt` milliseconds.
    pub fn step(&mut self, dt: u32) {
        let dt_f = dt as f32;
        if self.discharging() {
            let tau = if self.dump_broken { 600_000.0 } else { 100.0 };
            self.cap_v *= (-dt_f / tau).exp();
            if self.cap_v < 0.5 {
                self.cap_v = 0.0;
            }
        }
        // The charger runs only while the discharge path is not active.
        if let Some(target) = self
            .charger
            .filter(|_| charger_may_run(true, self.discharging()))
        {
            if !self.charger_broken {
                let t = f32::from(target.0);
                if self.charger_runaway {
                    self.cap_v += self.charge_rate * dt_f;
                } else if self.cap_v < t {
                    self.cap_v = (self.cap_v + self.charge_rate * dt_f).min(t);
                }
            }
        }
        // The hardware interlock.
        let discharging = self.discharging();
        if self.request.is_some()
            && self.shock_status == ShockStatus::Pending
            && bridge_may_fire(self.request.is_some(), self.permit, discharging)
        {
            self.fires.push(FireRecord {
                at: self.now,
                voltage: self.cap_v,
                main_request: self.request.is_some(),
                permit: self.permit,
                main_hold: self.main_hold,
                sup_hold: self.sup_hold,
            });
            self.cap_v *= 0.2;
            self.shock_status = if self.output_fault {
                ShockStatus::Fault
            } else {
                ShockStatus::Delivered
            };
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
    fn request_shock(&mut self, plan: &ShockPlan) {
        let b = &mut *self.0;
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
        if b.request.is_some() {
            b.violations.push(format!(
                "t={} shock requested twice without withdrawal",
                b.now
            ));
        }
        b.requests.push(RequestRecord {
            at: b.now,
            plan: *plan,
        });
        b.request = Some(*plan);
        b.shock_status = ShockStatus::Pending;
    }
    fn withdraw_shock_request(&mut self) {
        self.0.request = None;
        self.0.shock_status = ShockStatus::Idle;
    }
    fn shock_status(&mut self) -> ShockStatus {
        self.0.shock_status
    }
    fn cancel_pressed(&mut self) -> bool {
        self.0.cancel
    }
    fn patient_mode(&mut self) -> PatientMode {
        self.0.mode
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
        Ohms(self.0.sup_pad_ohms.unwrap_or(self.0.pad_ohms))
    }
    fn cancel_pressed(&mut self) -> bool {
        self.0.sup_cancel.unwrap_or(self.0.cancel)
    }
    fn set_permit(&mut self, permit: bool) {
        self.0.permit = permit;
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

pub type TestRescue = Rescue<FakeAnalyzer, FakePlanner>;

pub struct Bench {
    pub board: Board,
    pub rescue: TestRescue,
    pub sup: Supervisor,
    pub config: Config,
    pub tick_ms: u32,
    pub main_alive: bool,
    pub sup_alive: bool,
    pub resets: Rc<Cell<u32>>,
    pub states: Vec<(u32, RescueState)>,
}

impl Bench {
    pub fn new() -> Self {
        Self::with(Config::default(), FakePlanner::default())
    }

    pub fn with(config: Config, planner: FakePlanner) -> Self {
        Self::with_all(config, planner, 250)
    }

    /// `every`: how many samples between fake analyser decisions (250 = one per second).
    pub fn with_all(config: Config, planner: FakePlanner, every: u32) -> Self {
        let last = Rc::new(Cell::new(None));
        let resets = Rc::new(Cell::new(0));
        let analyzer = FakeAnalyzer {
            every,
            count: 0,
            last_emitted: last.clone(),
            resets: resets.clone(),
        };
        let rescue = Rescue::new(config, analyzer, planner).expect("valid config");
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

    /// Power on and run until the pads are being analysed (pads on, self-test passed).
    pub fn run_to_analysing(&mut self) {
        assert!(
            self.run_until_state(RescueState::Analysing, 30_000),
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
