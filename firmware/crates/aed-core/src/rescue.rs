// SPDX-License-Identifier: GPL-3.0-or-later
//! The rescue sequence: the main controller's state machine.
//!
//! This follows the diagram in ARCHITECTURE.md, "The rescue sequence":
//!
//! ```text
//! SelfTest -> AttachPads -> Analysing -> Charging -> Countdown -> Delivering -> Cpr
//!                              |            |            |             |
//!                              +-> Cpr      +-> Disarming (empty the capacitor) -> Cpr
//!                                                        |
//!                                                        +-> Fault (if it cannot be emptied)
//! Cpr -> Analysing after the CPR period; Analysing/Cpr -> AttachPads if pads come off.
//! Any fault: Disarming -> Fault ("continue CPR").
//! ```
//!
//! The rules it keeps (each has a test that names it):
//!
//! * Only a `Shockable` decision can start charging. "Not sure"
//!   (`Indeterminate`) and `NonShockable` both lead to CPR.
//! * Analysis keeps running while charging and counting down. Any decision
//!   other than `Shockable`, pads coming off, cancel, or a change of the
//!   adult/child switch abandons the shock: the capacitor is emptied and the
//!   device goes to CPR.
//! * At the moment of delivery the latest decision must be `Shockable` and no
//!   older than [`Config::confirmation_max_age_ms`], and no other decision may
//!   have arrived since charging began.
//! * The shock plan is made from a fresh pad-resistance reading taken just
//!   before delivery. If no plan can be made, no shock.
//! * The shock is requested once. Its outcome is recorded. Afterwards the
//!   rhythm analysis is reset, the remaining charge is emptied and checked,
//!   and the device goes to CPR.
//! * Cancel works in every state.
//! * Any fault empties the capacitor and ends in the Fault state, whose prompt
//!   tells the responder to continue CPR. If the capacitor cannot be
//!   confirmed empty, that is its own, more serious fault.
//! * Every state except AttachPads and Fault has a time limit.

use crate::config::{within, Config, ConfigError};
use crate::events::{Event, EventRecord, EVENT_CAPACITY};
use crate::hal::{BatteryStatus, MainHardware, ShockStatus};
use crate::link::{MainStatus, SupervisorFault, SupervisorReport};
use crate::prompts::Prompt;
use crate::selftest::{LinkSnapshot, SelfTest, SelfTestFailure, SelfTestProgress};
use aed_types::{
    EcgSample, Millis, Ohms, PatientMode, PlanError, RhythmAnalyzer, RhythmDecision, ShockPlan,
    ShockPlanner, Volts, MAX_PATIENT_OHMS, MIN_PATIENT_OHMS,
};

/// The states of the rescue sequence (ARCHITECTURE.md diagram).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RescueState {
    /// Checking the device after switch-on.
    SelfTest,
    /// Waiting for the pads to be on a patient.
    AttachPads,
    /// Analysing the heart rhythm; nobody may touch the patient.
    Analysing,
    /// A shock is advised; charging the capacitor.
    Charging,
    /// Charged; counting down aloud before the automatic shock.
    Countdown,
    /// The shock has been requested; waiting for the hardware to deliver it.
    Delivering,
    /// Emptying the capacitor into the internal load and checking it is empty.
    Disarming,
    /// CPR period, with the compression metronome running.
    Cpr,
    /// The device has a fault and will not shock. Prompts continue CPR.
    Fault,
}

/// Why a shock being prepared was abandoned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortReason {
    /// The cancel control was pressed.
    Cancelled,
    /// The analysis gave a decision other than `Shockable`.
    RhythmChanged,
    /// Pad contact was lost.
    PadsOff,
    /// The adult/child switch was moved.
    ModeChanged,
    /// The planner could not make a plan for the measured pad resistance.
    PlanRefused(PlanError),
    /// No recent enough `Shockable` decision at the moment of delivery.
    ConfirmationStale,
}

/// What happened to a requested shock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockOutcome {
    /// The hardware delivered it.
    Delivered,
    /// The hardware interlock refused it.
    RefusedByInterlock,
    /// Nothing happened within the delivery time limit (usually: no permit
    /// from the supervisor). The request was withdrawn.
    TimedOut,
    /// The output stage reported a fault.
    Fault,
    /// The request was withdrawn before delivery, for this reason.
    Withdrawn(AbortReason),
}

/// A fault in the main controller's view. Every fault ends in the Fault state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    /// The self-test failed at this check.
    SelfTest(SelfTestFailure),
    /// The capacitor did not charge within the limit.
    ChargeTimeout,
    /// The main controller read a voltage over the limit.
    Overvoltage,
    /// The capacitor voltage drifted away from the planned voltage (up or
    /// down) during the countdown.
    ChargeOffTarget,
    /// The capacitor was armed for longer than the limit.
    ArmedTooLong,
    /// The output stage reported a fault while delivering.
    DeliveryFault,
    /// The planner's voltage does not fit the limits or does not match the charge.
    PlanMismatch,
    /// The supervisor latched this fault.
    Supervisor(SupervisorFault),
    /// The supervisor stopped reporting.
    SupervisorLinkLost,
    /// The battery cannot power a rescue.
    BatteryDepleted,
    /// The capacitor could not be confirmed empty within the limit.
    /// The most serious fault: it has its own prompt.
    CapacitorNotEmptied,
}

impl FaultKind {
    /// True for the fault where the capacitor may still be charged.
    #[must_use]
    pub const fn is_serious(self) -> bool {
        matches!(self, FaultKind::CapacitorNotEmptied)
    }
}

/// Running totals for the event summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counters {
    /// Shocks the hardware reported delivered.
    pub shocks_delivered: u16,
    /// Shocks advised but abandoned or not delivered.
    pub shocks_not_given: u16,
    /// Analyses started.
    pub analyses: u16,
    /// CPR periods started.
    pub cpr_periods: u16,
    /// Metronome beats given.
    pub metronome_beats: u32,
}

/// Where to go once the capacitor is confirmed empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AfterDisarm {
    Cpr,
    Fault(FaultKind),
}

/// Debounced pad contact.
#[derive(Debug, Clone, Copy)]
struct PadMonitor {
    on: bool,
    change_since: Option<Millis>,
}

impl PadMonitor {
    /// Returns `Some(new_state)` when contact has changed for long enough.
    fn update(&mut self, ohms: Ohms, now: Millis, debounce_ms: u32) -> Option<bool> {
        let raw = pads_in_range(ohms);
        if raw == self.on {
            self.change_since = None;
            return None;
        }
        let since = *self.change_since.get_or_insert(now);
        if now.since(since) >= debounce_ms {
            self.on = raw;
            self.change_since = None;
            Some(raw)
        } else {
            None
        }
    }
}

/// True if a pad resistance is in the range the device must shock into.
#[must_use]
pub fn pads_in_range(ohms: Ohms) -> bool {
    ohms >= MIN_PATIENT_OHMS && ohms <= MAX_PATIENT_OHMS
}

/// What ended an armed state early.
#[derive(Debug, Clone, Copy)]
enum ArmedExit {
    Abort(AbortReason),
    Fault(FaultKind),
}

/// The main controller's rescue sequence.
///
/// Non-blocking: call [`Rescue::tick`] frequently (every few milliseconds) and
/// [`Rescue::on_ecg_sample`] for every heart-signal sample.
#[derive(Debug)]
pub struct Rescue<A: RhythmAnalyzer, P: ShockPlanner> {
    config: Config,
    analyzer: A,
    planner: P,
    started: bool,
    powered_on_at: Millis,
    state: RescueState,
    state_since: Millis,
    state_changed: bool,
    self_test: SelfTest,
    self_test_result: Option<Result<(), SelfTestFailure>>,
    pads: PadMonitor,
    cancel_was_pressed: bool,
    mode: PatientMode,
    mode_known: bool,
    mode_at_arming: PatientMode,
    battery_low_announced: bool,
    analysis_result: Option<RhythmDecision>,
    last_decision: Option<(RhythmDecision, Millis)>,
    veto: bool,
    target: Volts,
    armed_since: Option<Millis>,
    plan: Option<ShockPlan>,
    after_disarm: AfterDisarm,
    fault: Option<FaultKind>,
    countdown_spoken: u8,
    next_beat: Millis,
    last_reminder: Millis,
    counters: Counters,
    link: Option<LinkSnapshot>,
    link_required: bool,
    status_sequence: u32,
    last_status_at: Option<Millis>,
    events: EventRecord<EVENT_CAPACITY>,
}

impl<A: RhythmAnalyzer, P: ShockPlanner> Rescue<A, P> {
    /// A new rescue sequence, before switch-on. The first `tick` starts the self-test.
    ///
    /// # Errors
    /// Refuses a configuration whose numbers contradict each other.
    pub fn new(config: Config, analyzer: A, planner: P) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Rescue {
            config,
            analyzer,
            planner,
            started: false,
            powered_on_at: Millis(0),
            state: RescueState::SelfTest,
            state_since: Millis(0),
            state_changed: true,
            self_test: SelfTest::new(config, config.self_test_voltage),
            self_test_result: None,
            pads: PadMonitor {
                on: false,
                change_since: None,
            },
            cancel_was_pressed: false,
            mode: PatientMode::Adult,
            mode_known: false,
            mode_at_arming: PatientMode::Adult,
            battery_low_announced: false,
            analysis_result: None,
            last_decision: None,
            veto: false,
            target: Volts(0),
            armed_since: None,
            plan: None,
            after_disarm: AfterDisarm::Cpr,
            fault: None,
            countdown_spoken: 0,
            next_beat: Millis(0),
            last_reminder: Millis(0),
            counters: Counters::default(),
            link: None,
            link_required: false,
            status_sequence: 0,
            last_status_at: None,
            events: EventRecord::new(),
        })
    }

    /// The current state.
    #[must_use]
    pub const fn state(&self) -> RescueState {
        self.state
    }

    /// The fault that put the device in the Fault state (or is about to), if any.
    #[must_use]
    pub const fn fault(&self) -> Option<FaultKind> {
        self.fault
    }

    /// The power-on self-test result, once it has finished.
    #[must_use]
    pub const fn self_test_result(&self) -> Option<Result<(), SelfTestFailure>> {
        self.self_test_result
    }

    /// Running totals (shocks, analyses, CPR periods, metronome beats).
    #[must_use]
    pub const fn counters(&self) -> Counters {
        self.counters
    }

    /// The event record.
    #[must_use]
    pub const fn events(&self) -> &EventRecord<EVENT_CAPACITY> {
        &self.events
    }

    /// The configuration in use.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// The rhythm analyser (for inspection by tests and the simulator).
    #[must_use]
    pub const fn analyzer(&self) -> &A {
        &self.analyzer
    }

    /// The latest rhythm decision and when it arrived.
    #[must_use]
    pub const fn last_decision(&self) -> Option<(RhythmDecision, Millis)> {
        self.last_decision
    }

    /// The plan for the most recent shock request, if any.
    #[must_use]
    pub const fn last_plan(&self) -> Option<ShockPlan> {
        self.plan
    }

    /// The adult/child setting last read from the switch.
    #[must_use]
    pub const fn patient_mode(&self) -> PatientMode {
        self.mode
    }

    /// Milliseconds since switch-on.
    #[must_use]
    pub fn elapsed_ms(&self, now: Millis) -> u32 {
        now.since(self.powered_on_at)
    }

    /// Milliseconds spent in the current state.
    #[must_use]
    pub fn time_in_state_ms(&self, now: Millis) -> u32 {
        now.since(self.state_since)
    }

    /// Feed one heart-signal sample. Samples are passed to the rhythm analysis
    /// only while analysing, charging, counting down or delivering; during CPR
    /// the signal is full of compression movement and is not analysed.
    pub fn on_ecg_sample(&mut self, sample: EcgSample, now: Millis) {
        let analysing = matches!(
            self.state,
            RescueState::Analysing
                | RescueState::Charging
                | RescueState::Countdown
                | RescueState::Delivering
        );
        if !analysing {
            return;
        }
        let Some(decision) = self.analyzer.push(sample) else {
            return;
        };
        self.events.push(now, Event::Decision(decision));
        self.last_decision = Some((decision, now));
        if self.state == RescueState::Analysing {
            // "Not sure" or "no shock" always wins over "shock" within one analysis.
            if self.analysis_result.is_none() || decision != RhythmDecision::Shockable {
                self.analysis_result = Some(decision);
            }
        } else if decision != RhythmDecision::Shockable {
            self.veto = true;
        }
    }

    /// Run one step. Call frequently.
    pub fn tick(&mut self, hw: &mut impl MainHardware) {
        let now = hw.now();
        if !self.started {
            self.started = true;
            self.powered_on_at = now;
            self.state_since = now;
            self.events.push(now, Event::PowerOn);
            hw.set_ready_indicator(false);
        }

        if let Some(report) = hw.supervisor_report() {
            self.on_report(report, now);
        }

        let ohms = hw.pad_resistance();
        match self.pads.update(ohms, now, self.config.pad_debounce_ms) {
            Some(true) => self.events.push(now, Event::PadsOn),
            Some(false) => self.events.push(now, Event::PadsOff),
            None => {}
        }

        let cancel = hw.cancel_pressed();
        if cancel && !self.cancel_was_pressed {
            self.events.push(now, Event::CancelPressed);
        }
        self.cancel_was_pressed = cancel;

        let mode = hw.patient_mode();
        if !self.mode_known {
            self.mode_known = true;
            self.mode = mode;
        } else if mode != self.mode {
            self.mode = mode;
            self.events.push(now, Event::PatientModeChanged(mode));
            hw.prompt(mode_prompt(mode));
        }

        match hw.battery() {
            BatteryStatus::Good => self.battery_low_announced = false,
            BatteryStatus::Low => {
                if !self.battery_low_announced {
                    self.battery_low_announced = true;
                    self.events.push(now, Event::BatteryLow);
                    hw.prompt(Prompt::LowBattery);
                }
            }
            BatteryStatus::Depleted => self.begin_fault(FaultKind::BatteryDepleted, hw, now),
        }

        if let Some(f) = self.link.and_then(|l| l.report.fault) {
            self.begin_fault(FaultKind::Supervisor(f), hw, now);
        }
        let link_stale = self
            .link
            .is_none_or(|l| now.since(l.received_at) > self.config.heartbeat_timeout_ms);
        if self.link_required && link_stale {
            self.begin_fault(FaultKind::SupervisorLinkLost, hw, now);
        }

        let voltage = hw.capacitor_voltage();
        match self.state {
            RescueState::SelfTest => self.tick_self_test(hw, now, cancel),
            RescueState::AttachPads => self.tick_attach_pads(hw, now),
            RescueState::Analysing => self.tick_analysing(hw, now, cancel),
            RescueState::Charging => self.tick_charging(hw, now, cancel, voltage),
            RescueState::Countdown => self.tick_countdown(hw, now, cancel, voltage),
            RescueState::Delivering => self.tick_delivering(hw, now, cancel, voltage),
            RescueState::Disarming => self.tick_disarming(hw, now, voltage),
            RescueState::Cpr => self.tick_cpr(hw, now),
            RescueState::Fault => self.tick_fault(hw, now),
        }

        self.apply_outputs(hw);
        self.send_status_if_due(hw, now);
    }

    // ----- state handlers -------------------------------------------------

    fn tick_self_test(&mut self, hw: &mut impl MainHardware, now: Millis, cancel: bool) {
        if cancel {
            // The device has not yet shown it can empty its capacitor safely,
            // so a cancelled self-test is treated as a failed one.
            self.self_test.abort(hw);
            self.finish_self_test_failed(SelfTestFailure::Cancelled, hw, now);
            return;
        }
        match self.self_test.step(hw, self.link) {
            SelfTestProgress::Running => {}
            SelfTestProgress::Finished(Ok(pass)) => {
                self.self_test_result = Some(Ok(()));
                self.events.push(now, Event::SelfTestPassed);
                self.self_test_passed(pass.battery_low, hw, now);
            }
            SelfTestProgress::Finished(Err(f)) if f.is_advisory() => {
                self.self_test_result = Some(Err(f));
                self.events.push(now, Event::SelfTestFailed(f));
                if f == SelfTestFailure::PadsExpired {
                    hw.prompt(Prompt::ReplacePads);
                }
                self.self_test_passed(false, hw, now);
            }
            SelfTestProgress::Finished(Err(f)) => self.finish_self_test_failed(f, hw, now),
        }
    }

    fn self_test_passed(&mut self, battery_low: bool, hw: &mut impl MainHardware, now: Millis) {
        if battery_low && !self.battery_low_announced {
            self.battery_low_announced = true;
            self.events.push(now, Event::BatteryLow);
            hw.prompt(Prompt::LowBattery);
        }
        self.link_required = true;
        hw.set_ready_indicator(true);
        if self.mode == PatientMode::Child {
            hw.prompt(Prompt::ChildModeFrontBack);
        }
        self.go(RescueState::AttachPads, hw, now);
    }

    fn finish_self_test_failed(
        &mut self,
        f: SelfTestFailure,
        hw: &mut impl MainHardware,
        now: Millis,
    ) {
        self.self_test_result = Some(Err(f));
        self.events.push(now, Event::SelfTestFailed(f));
        if f == SelfTestFailure::DischargeTimeout {
            // Emptying has already been tried and failed: straight to the serious fault.
            self.enter_fault(FaultKind::CapacitorNotEmptied, hw, now);
        } else {
            self.begin_fault(FaultKind::SelfTest(f), hw, now);
        }
    }

    fn tick_attach_pads(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if self.pads.on {
            self.go(RescueState::Analysing, hw, now);
        } else if now.since(self.last_reminder) >= self.config.attach_pads_repeat_ms {
            self.last_reminder = now;
            hw.prompt(Prompt::AttachPads);
        }
    }

    fn tick_analysing(&mut self, hw: &mut impl MainHardware, now: Millis, cancel: bool) {
        if !self.pads.on {
            self.go(RescueState::AttachPads, hw, now);
        } else if cancel {
            hw.prompt(Prompt::AnalysisCancelled);
            self.go(RescueState::Cpr, hw, now);
        } else if let Some(decision) = self.analysis_result {
            if decision == RhythmDecision::Shockable {
                self.start_charging(hw, now);
            } else {
                hw.prompt(Prompt::NoShockAdvised);
                self.go(RescueState::Cpr, hw, now);
            }
        } else if self.time_in_state_ms(now) > self.config.analysis_time_limit_ms {
            self.events.push(now, Event::AnalysisTimedOut);
            hw.prompt(Prompt::NoShockAdvised);
            self.go(RescueState::Cpr, hw, now);
        }
    }

    fn start_charging(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let target = self.planner.charge_voltage(self.mode);
        let lowest_sensible = self
            .config
            .empty_voltage
            .0
            .saturating_add(self.config.voltage_tolerance.0);
        let highest_sensible = self
            .config
            .overvoltage_limit
            .0
            .saturating_sub(self.config.voltage_tolerance.0);
        if target.0 <= lowest_sensible || target.0 >= highest_sensible {
            self.begin_fault(FaultKind::PlanMismatch, hw, now);
            return;
        }
        self.target = target;
        self.armed_since = Some(now);
        self.mode_at_arming = self.mode;
        self.veto = false;
        self.plan = None;
        self.events.push(now, Event::ChargeStarted(target));
        hw.prompt(Prompt::ShockAdvisedCharging);
        self.go(RescueState::Charging, hw, now);
    }

    /// Checks shared by Charging, Countdown and Delivering.
    fn armed_check(&self, now: Millis, cancel: bool, voltage: Volts) -> Option<ArmedExit> {
        if cancel {
            return Some(ArmedExit::Abort(AbortReason::Cancelled));
        }
        if !self.pads.on {
            return Some(ArmedExit::Abort(AbortReason::PadsOff));
        }
        if self.mode != self.mode_at_arming {
            return Some(ArmedExit::Abort(AbortReason::ModeChanged));
        }
        if self.veto {
            return Some(ArmedExit::Abort(AbortReason::RhythmChanged));
        }
        if voltage > self.config.overvoltage_limit {
            return Some(ArmedExit::Fault(FaultKind::Overvoltage));
        }
        let armed_for = self.armed_since.map_or(0, |t| now.since(t));
        if armed_for > self.config.max_charged_ms {
            return Some(ArmedExit::Fault(FaultKind::ArmedTooLong));
        }
        None
    }

    fn handle_armed_exit(&mut self, exit: ArmedExit, hw: &mut impl MainHardware, now: Millis) {
        match exit {
            ArmedExit::Abort(reason) => self.abort(reason, hw, now),
            ArmedExit::Fault(kind) => self.begin_fault(kind, hw, now),
        }
    }

    fn tick_charging(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        if let Some(exit) = self.armed_check(now, cancel, voltage) {
            self.handle_armed_exit(exit, hw, now);
        } else if voltage.0
            >= self
                .target
                .0
                .saturating_sub(self.config.voltage_tolerance.0)
        {
            self.events.push(now, Event::Charged(voltage));
            self.go(RescueState::Countdown, hw, now);
        } else if self.time_in_state_ms(now) > self.config.charge_time_limit_ms {
            self.begin_fault(FaultKind::ChargeTimeout, hw, now);
        }
    }

    fn tick_countdown(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        if let Some(exit) = self.armed_check(now, cancel, voltage) {
            self.handle_armed_exit(exit, hw, now);
            return;
        }
        if !within(voltage, self.target, self.config.voltage_tolerance) {
            self.begin_fault(FaultKind::ChargeOffTarget, hw, now);
            return;
        }
        let elapsed = self.time_in_state_ms(now);
        if elapsed >= self.config.countdown_ms {
            self.deliver(hw, now);
            return;
        }
        // Speak the number of whole seconds left, rounded up: 5, 4, 3, 2, 1.
        let remaining = self.config.countdown_ms.saturating_sub(elapsed);
        let seconds = remaining.saturating_add(999) / 1000;
        let seconds = u8::try_from(seconds).unwrap_or(u8::MAX);
        if seconds != self.countdown_spoken {
            self.countdown_spoken = seconds;
            hw.prompt(Prompt::Countdown(seconds));
        }
    }

    fn deliver(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let fresh = matches!(
            self.last_decision,
            Some((RhythmDecision::Shockable, at))
                if now.since(at) <= self.config.confirmation_max_age_ms
        );
        if !fresh || self.veto {
            self.abort(AbortReason::ConfirmationStale, hw, now);
            return;
        }
        let ohms = hw.pad_resistance();
        self.events.push(now, Event::PadResistance(ohms));
        match self.planner.plan(self.mode, ohms) {
            Err(e) => {
                self.events.push(now, Event::PlanRefused(e));
                self.abort(AbortReason::PlanRefused(e), hw, now);
            }
            Ok(plan) => {
                if !within(
                    plan.charge_voltage,
                    self.target,
                    self.config.voltage_tolerance,
                ) {
                    self.begin_fault(FaultKind::PlanMismatch, hw, now);
                    return;
                }
                self.plan = Some(plan);
                self.events.push(now, Event::Plan(plan));
                hw.request_shock(&plan);
                self.events.push(now, Event::ShockRequested);
                self.go(RescueState::Delivering, hw, now);
            }
        }
    }

    fn tick_delivering(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        match hw.shock_status() {
            ShockStatus::Delivered => {
                hw.withdraw_shock_request();
                self.events.push(now, Event::Shock(ShockOutcome::Delivered));
                self.counters.shocks_delivered = self.counters.shocks_delivered.saturating_add(1);
                self.analyzer.reset();
                self.last_decision = None;
                hw.prompt(Prompt::ShockDelivered);
                self.disarm(AfterDisarm::Cpr, hw, now);
            }
            ShockStatus::Fault => {
                hw.withdraw_shock_request();
                self.events.push(now, Event::Shock(ShockOutcome::Fault));
                self.begin_fault(FaultKind::DeliveryFault, hw, now);
            }
            ShockStatus::Refused => {
                hw.withdraw_shock_request();
                self.events
                    .push(now, Event::Shock(ShockOutcome::RefusedByInterlock));
                self.not_given();
                hw.prompt(Prompt::ShockNotDelivered);
                self.disarm(AfterDisarm::Cpr, hw, now);
            }
            ShockStatus::Pending | ShockStatus::Idle => {
                if let Some(exit) = self.armed_check(now, cancel, voltage) {
                    hw.withdraw_shock_request();
                    if let ArmedExit::Abort(reason) = exit {
                        self.events
                            .push(now, Event::Shock(ShockOutcome::Withdrawn(reason)));
                    }
                    self.handle_armed_exit(exit, hw, now);
                } else if self.time_in_state_ms(now) > self.config.delivery_timeout_ms {
                    hw.withdraw_shock_request();
                    self.events.push(now, Event::Shock(ShockOutcome::TimedOut));
                    self.not_given();
                    hw.prompt(Prompt::ShockNotDelivered);
                    self.disarm(AfterDisarm::Cpr, hw, now);
                }
            }
        }
    }

    fn tick_disarming(&mut self, hw: &mut impl MainHardware, now: Millis, voltage: Volts) {
        if voltage < self.config.empty_voltage {
            self.events.push(now, Event::CapacitorEmptied(voltage));
            match self.after_disarm {
                AfterDisarm::Cpr => self.go(RescueState::Cpr, hw, now),
                AfterDisarm::Fault(kind) => self.enter_fault(kind, hw, now),
            }
        } else if self.time_in_state_ms(now) > self.config.disarm_time_limit_ms {
            self.enter_fault(FaultKind::CapacitorNotEmptied, hw, now);
        }
    }

    fn tick_cpr(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if !self.pads.on {
            self.go(RescueState::AttachPads, hw, now);
            return;
        }
        self.metronome(hw, now);
        if self.time_in_state_ms(now) >= self.config.cpr_period_ms {
            self.go(RescueState::Analysing, hw, now);
        }
    }

    fn tick_fault(&mut self, hw: &mut impl MainHardware, now: Millis) {
        // A faulty device must never leave a responder waiting: keep the
        // metronome going and keep reminding them to continue CPR.
        self.metronome(hw, now);
        if now.since(self.last_reminder) >= self.config.fault_reminder_ms {
            self.last_reminder = now;
            hw.prompt(self.fault_prompt());
        }
    }

    // ----- transitions ----------------------------------------------------

    fn go(&mut self, to: RescueState, hw: &mut impl MainHardware, now: Millis) {
        let from = self.state;
        self.events.push(now, Event::StateChanged { from, to });
        self.state = to;
        self.state_since = now;
        self.state_changed = true;
        match to {
            RescueState::AttachPads => {
                self.last_reminder = now;
                if !self.pads.on {
                    hw.prompt(Prompt::AttachPads);
                }
            }
            RescueState::Analysing => {
                self.analyzer.reset();
                self.analysis_result = None;
                self.last_decision = None;
                self.armed_since = None;
                self.counters.analyses = self.counters.analyses.saturating_add(1);
                hw.prompt(if from == RescueState::Cpr {
                    Prompt::StopCprAnalysing
                } else {
                    Prompt::AnalysingDoNotTouch
                });
            }
            RescueState::Countdown => {
                self.countdown_spoken = 0;
                hw.prompt(Prompt::StandClear);
            }
            RescueState::Disarming => {
                hw.set_charger(None);
                hw.withdraw_shock_request();
                hw.set_discharge_hold(false);
            }
            RescueState::Cpr => {
                self.armed_since = None;
                self.next_beat = now;
                self.counters.cpr_periods = self.counters.cpr_periods.saturating_add(1);
                hw.prompt(Prompt::StartCpr);
            }
            RescueState::Fault => {
                hw.set_charger(None);
                hw.withdraw_shock_request();
                hw.set_discharge_hold(false);
                hw.set_ready_indicator(false);
                self.armed_since = None;
                self.next_beat = now;
                self.last_reminder = now;
                hw.prompt(self.fault_prompt());
            }
            RescueState::SelfTest | RescueState::Charging | RescueState::Delivering => {}
        }
    }

    fn abort(&mut self, reason: AbortReason, hw: &mut impl MainHardware, now: Millis) {
        self.events.push(now, Event::ShockAborted(reason));
        self.not_given();
        hw.prompt(match reason {
            AbortReason::Cancelled => Prompt::ShockCancelled,
            AbortReason::RhythmChanged => Prompt::RhythmChangedNoShock,
            AbortReason::PadsOff | AbortReason::PlanRefused(_) => Prompt::CheckPads,
            AbortReason::ModeChanged | AbortReason::ConfirmationStale => Prompt::ShockNotDelivered,
        });
        self.disarm(AfterDisarm::Cpr, hw, now);
    }

    fn not_given(&mut self) {
        self.counters.shocks_not_given = self.counters.shocks_not_given.saturating_add(1);
    }

    fn disarm(&mut self, after: AfterDisarm, hw: &mut impl MainHardware, now: Millis) {
        self.after_disarm = after;
        self.go(RescueState::Disarming, hw, now);
    }

    /// Start the fault path: empty the capacitor, then the Fault state.
    fn begin_fault(&mut self, kind: FaultKind, hw: &mut impl MainHardware, now: Millis) {
        match self.state {
            RescueState::Fault => {}
            RescueState::Disarming => {
                if self.after_disarm == AfterDisarm::Cpr {
                    self.events.push(now, Event::Fault(kind));
                    self.fault = Some(kind);
                    self.after_disarm = AfterDisarm::Fault(kind);
                }
            }
            _ => {
                self.events.push(now, Event::Fault(kind));
                self.fault = Some(kind);
                if self.state == RescueState::SelfTest {
                    self.self_test.abort(hw);
                }
                if self.state == RescueState::Delivering {
                    hw.withdraw_shock_request();
                }
                self.disarm(AfterDisarm::Fault(kind), hw, now);
            }
        }
    }

    fn enter_fault(&mut self, kind: FaultKind, hw: &mut impl MainHardware, now: Millis) {
        if kind.is_serious() {
            // Recorded even if another fault started the path: this one matters most.
            self.events.push(now, Event::Fault(kind));
            self.fault = Some(kind);
        } else if self.fault.is_none() {
            self.events.push(now, Event::Fault(kind));
            self.fault = Some(kind);
        }
        self.go(RescueState::Fault, hw, now);
    }

    fn fault_prompt(&self) -> Prompt {
        if self.fault.is_some_and(FaultKind::is_serious) {
            Prompt::SeriousFaultContinueCpr
        } else {
            Prompt::DeviceFaultContinueCpr
        }
    }

    // ----- outputs and messages -------------------------------------------

    fn metronome(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if now >= self.next_beat {
            hw.metronome_beat();
            self.counters.metronome_beats = self.counters.metronome_beats.saturating_add(1);
            let interval = self.config.metronome_interval_ms();
            let next = self.next_beat.0.saturating_add(interval);
            // If ticks were late, do not burst-beat to catch up.
            self.next_beat = Millis(if next <= now.0 {
                now.0.saturating_add(interval)
            } else {
                next
            });
        }
    }

    /// Re-assert every output for the current state, every tick. Hold before
    /// charging; stop charging before releasing.
    fn apply_outputs(&mut self, hw: &mut impl MainHardware) {
        match self.state {
            // The self-test sequencer drives the outputs itself.
            RescueState::SelfTest => {}
            RescueState::Charging | RescueState::Countdown => {
                hw.set_discharge_hold(true);
                hw.set_charger(Some(self.target));
            }
            RescueState::Delivering => {
                hw.set_discharge_hold(true);
                hw.set_charger(None);
            }
            RescueState::AttachPads
            | RescueState::Analysing
            | RescueState::Disarming
            | RescueState::Cpr
            | RescueState::Fault => {
                hw.set_charger(None);
                hw.withdraw_shock_request();
                hw.set_discharge_hold(false);
            }
        }
    }

    fn send_status_if_due(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let due = self
            .last_status_at
            .is_none_or(|t| now.since(t) >= self.config.heartbeat_period_ms);
        if !due && !self.state_changed {
            return;
        }
        self.state_changed = false;
        self.last_status_at = Some(now);
        self.status_sequence = self.status_sequence.wrapping_add(1);
        let armed = matches!(
            self.state,
            RescueState::Charging | RescueState::Countdown | RescueState::Delivering
        );
        let planned_voltage = if armed {
            self.target
        } else if self.state == RescueState::SelfTest && self.self_test.is_charging() {
            self.self_test.test_voltage()
        } else {
            Volts(0)
        };
        let armed_for_ms = if armed {
            self.armed_since.map_or(0, |t| now.since(t))
        } else {
            0
        };
        let capacitor_voltage = hw.capacitor_voltage();
        hw.send_status(&MainStatus {
            sequence: self.status_sequence,
            state: self.state,
            capacitor_voltage,
            planned_voltage,
            armed_for_ms,
        });
    }

    fn on_report(&mut self, report: SupervisorReport, now: Millis) {
        let previous = self.link.map(|l| l.report);
        if report.permit && !previous.is_some_and(|p| p.permit) {
            self.events.push(now, Event::SupervisorPermitted);
        }
        if let Some(f) = report.fault {
            if previous.and_then(|p| p.fault).is_none() {
                self.events.push(now, Event::SupervisorFaultLatched(f));
            }
        }
        self.link = Some(LinkSnapshot {
            report,
            received_at: now,
        });
    }
}

/// The announcement for a patient mode.
const fn mode_prompt(mode: PatientMode) -> Prompt {
    match mode {
        PatientMode::Adult => Prompt::AdultMode,
        PatientMode::Child => Prompt::ChildModeFrontBack,
    }
}
