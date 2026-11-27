// SPDX-License-Identifier: GPL-3.0-or-later
//! The rescue sequence: the main controller's state machine.
//!
//! This follows the diagram in ARCHITECTURE.md, "The rescue sequence", with
//! charging during analysis (owner decision D-29):
//!
//! ```text
//! SelfTest -> AttachPads -> AnalysingCharging --(shock, charged)--------> Countdown -> Delivering -> Cpr
//!                              |        \--(shock, still charging)-> Charging -/
//!                              +-> Cpr (no shock / not sure to the limit / cancel)
//!                              +-> AttachPads (pads off)
//! Cpr -> AnalysingCharging after the CPR period; Cpr -> AttachPads if pads come off.
//! Charging / Countdown / Delivering -> Cpr if the shock is abandoned
//!                                      (-> AttachPads if the pads came off).
//! Any fault: -> Fault ("continue CPR").
//! ```
//!
//! (With [`Config::charge_during_analysis`] switched off the device uses
//! `Analysing` instead of `AnalysingCharging` and charges only after a shock
//! is advised, as in the first design.)
//!
//! **Emptying runs in the background.** Whenever the device leaves a state
//! in which it holds the capacitor charged (and on any fault), it lets go of
//! the discharge path, which empties the capacitor into the internal load,
//! and goes at once to its next state and instruction ("No shock advised.
//! Start CPR.", "Shock delivered. Start CPR."). It does not wait for the
//! capacitor to empty first: a responder is never left waiting on the
//! device. Meanwhile it checks, every tick, that its own voltage reading
//! (and the supervisor's) shows empty within [`Config::disarm_time_limit_ms`].
//! Until it does, the device holds nothing and starts no new charge. If its
//! own reading is not empty by the limit, that is the most serious fault
//! ("Serious device fault. Do not touch the device. Continue CPR."), from
//! whatever state the device has got to. The capacitor empties by itself
//! when let go (hardware rule, discharge by default), so not waiting does
//! not leave it charged; it only changes when the responder is told.
//!
//! The rules it keeps (each has a test that names it):
//!
//! * Only a `Shockable` decision can lead to a countdown and a shock.
//!   `NonShockable` ends the analysis with "No shock advised" and CPR. "Not
//!   sure" (`Indeterminate`) does not end it: the device repeats "Do not
//!   touch the patient" and keeps analysing until it gets `Shockable` or
//!   `NonShockable`, or the analysis time limit runs out ("inconclusive":
//!   no shock, CPR).
//! * The capacitor charges while the analysis runs (D-29), but only after
//!   the self-test has passed, only once the supervisor reports that it is
//!   holding the discharge path, and only once any earlier charge is
//!   confirmed empty. A charge that is not used is burned off in the
//!   internal load and counted ([`Counters::charges_burned`]).
//! * Analysis keeps running while charging and counting down. After a shock
//!   is advised, any decision other than `Shockable`, pads coming off,
//!   cancel, or a change of the adult/child switch abandons the shock: the
//!   capacitor is emptied and the device goes to CPR. (A switch change
//!   during the analysis itself empties the capacitor and recharges for the
//!   new mode without stopping the analysis.)
//! * At the moment of delivery the latest decision must be `Shockable` and no
//!   older than [`Config::confirmation_max_age_ms`], and no other decision may
//!   have arrived since the shock was advised.
//! * The shock plan is made from a fresh pad-resistance reading taken just
//!   before delivery. If no plan can be made, no shock. Pads that are on but
//!   read above the shock range (175 to 250 Ω, D-32) are analysed as normal
//!   and the responder is asked to press them firmly; at delivery, if they
//!   still read above the shock range: "Poor pad contact. No shock." and CPR.
//! * The shock is requested once. Its outcome is recorded. Afterwards the
//!   rhythm analysis is reset, the remaining charge is emptied and checked,
//!   and the device goes to CPR.
//! * Cancel works in every state except the switch-on self-test, where it
//!   is ignored (D-31). A cancel that reads pressed without a break for
//!   [`Config::cancel_stuck_ms`] is a stuck-button fault with its own prompt.
//! * Any fault empties the capacitor and ends in the Fault state, whose prompt
//!   tells the responder to continue CPR. If the capacitor cannot be
//!   confirmed empty, that is its own, more serious fault.
//! * Every state except AttachPads and Fault has a time limit.

use crate::config::{Config, ConfigError};
use crate::events::{Event, EventRecord, EVENT_CAPACITY};
use crate::hal::{BatteryStatus, MainHardware, ShockStatus};
use crate::link::{MainStatus, SupervisorFault, SupervisorReport};
use crate::prompts::Prompt;
use crate::selftest::{LinkSnapshot, SelfTest, SelfTestFailure, SelfTestProgress};
use aed_types::{
    pads_sensed, EcgSample, Millis, PatientMode, PlanError, RhythmAnalyzer, RhythmDecision,
    ShockPlan, ShockPlanner, Volts, MAX_PATIENT_OHMS,
};

/// The states of the rescue sequence (ARCHITECTURE.md diagram).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RescueState {
    /// Checking the device after switch-on.
    SelfTest,
    /// Waiting for the pads to be on a patient.
    AttachPads,
    /// Analysing the heart rhythm, without charging; nobody may touch the
    /// patient. Used only when [`Config::charge_during_analysis`] is off.
    Analysing,
    /// Analysing the heart rhythm while the capacitor charges, or is held
    /// charged, in case a shock is advised (decision D-29); nobody may touch
    /// the patient. No shock can be given from this state: a shock needs the
    /// countdown first, and the supervisor gives no permit here.
    AnalysingCharging,
    /// A shock is advised; waiting for the capacitor to finish charging.
    Charging,
    /// Charged; counting down aloud before the automatic shock.
    Countdown,
    /// The shock has been requested; waiting for the hardware to deliver it.
    Delivering,
    /// CPR period, with the compression metronome running.
    Cpr,
    /// The device has a fault and will not shock. Prompts continue CPR.
    Fault,
}

impl RescueState {
    /// True for the two analysing states.
    #[must_use]
    pub const fn is_analysing(self) -> bool {
        matches!(
            self,
            RescueState::Analysing | RescueState::AnalysingCharging
        )
    }

    /// True for the states in which the main controller holds the
    /// capacitor charged (or charging): charging during analysis, charging,
    /// counting down and delivering.
    #[must_use]
    pub const fn holds_charge(self) -> bool {
        matches!(
            self,
            RescueState::AnalysingCharging
                | RescueState::Charging
                | RescueState::Countdown
                | RescueState::Delivering
        )
    }
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
    /// `ImpedanceTooHigh` with the pads still on (175 to 250 Ω) is the
    /// "poor pad contact" case of D-32.
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
    /// down) after it had charged.
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
    /// The cancel control read pressed for longer than
    /// [`Config::cancel_stuck_ms`] (D-31). It would block every shock.
    CancelStuck,
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
    /// Charges emptied into the internal load without a shock (every
    /// analysis that charges and ends without a shock burns one: the
    /// battery and heat cost of D-29).
    pub charges_burned: u16,
    /// Analyses started.
    pub analyses: u16,
    /// CPR periods started.
    pub cpr_periods: u16,
    /// Metronome beats given.
    pub metronome_beats: u32,
}

/// Debounced on/off condition (pad contact, poor pad contact).
#[derive(Debug, Clone, Copy)]
struct Debounced {
    on: bool,
    change_since: Option<Millis>,
}

impl Debounced {
    const fn off() -> Self {
        Debounced {
            on: false,
            change_since: None,
        }
    }

    /// Returns `Some(new_state)` when the raw condition has changed for long enough.
    fn update(&mut self, raw: bool, now: Millis, debounce_ms: u32) -> Option<bool> {
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

    /// Take a fresh reading's word for it at once (used when the reading
    /// taken for a shock plan shows the pads off before the smoothing has
    /// caught up).
    fn force(&mut self, on: bool) {
        self.on = on;
        self.change_since = None;
    }
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
    pads: Debounced,
    poor_contact: Debounced,
    cancel_was_pressed: bool,
    cancel_since: Option<Millis>,
    mode: PatientMode,
    mode_known: bool,
    mode_at_arming: PatientMode,
    battery_low_announced: bool,
    analysis_result: Option<RhythmDecision>,
    not_sure_seen: bool,
    hands_off_reminded_at: Option<Millis>,
    press_pads_reminded_at: Millis,
    last_decision: Option<(RhythmDecision, Millis)>,
    veto: bool,
    target: Volts,
    armed_since: Option<Millis>,
    charged_at: Option<Millis>,
    charger_was_on: bool,
    shocked_this_arming: bool,
    emptying_since: Option<Millis>,
    plan: Option<ShockPlan>,
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
            pads: Debounced::off(),
            poor_contact: Debounced::off(),
            cancel_was_pressed: false,
            cancel_since: None,
            mode: PatientMode::Adult,
            mode_known: false,
            mode_at_arming: PatientMode::Adult,
            battery_low_announced: false,
            analysis_result: None,
            not_sure_seen: false,
            hands_off_reminded_at: None,
            press_pads_reminded_at: Millis(0),
            last_decision: None,
            veto: false,
            target: Volts(0),
            armed_since: None,
            charged_at: None,
            charger_was_on: false,
            shocked_this_arming: false,
            emptying_since: None,
            plan: None,
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

    /// The fault that put the device in the Fault state, if any.
    #[must_use]
    pub const fn fault(&self) -> Option<FaultKind> {
        self.fault
    }

    /// The power-on self-test result, once it has finished.
    #[must_use]
    pub const fn self_test_result(&self) -> Option<Result<(), SelfTestFailure>> {
        self.self_test_result
    }

    /// Running totals (shocks, burned charges, analyses, CPR periods, metronome beats).
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

    /// True while the capacitor is being emptied in the background and is
    /// not yet confirmed empty. No new charge starts until it is.
    #[must_use]
    pub const fn emptying(&self) -> bool {
        self.emptying_since.is_some()
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
        let analysing = self.state.is_analysing()
            || matches!(
                self.state,
                RescueState::Charging | RescueState::Countdown | RescueState::Delivering
            );
        if !analysing {
            return;
        }
        let Some(decision) = self.analyzer.push(sample) else {
            return;
        };
        self.events.push(now, Event::Decision(decision));
        self.last_decision = Some((decision, now));
        if self.state.is_analysing() {
            // Answers not yet acted on: "no shock" wins over everything, and
            // "not sure" wins over "shock".
            let pending = self.analysis_result;
            self.analysis_result = Some(match (pending, decision) {
                (Some(RhythmDecision::NonShockable), _) | (_, RhythmDecision::NonShockable) => {
                    RhythmDecision::NonShockable
                }
                (Some(RhythmDecision::Indeterminate), _) | (_, RhythmDecision::Indeterminate) => {
                    RhythmDecision::Indeterminate
                }
                (_, RhythmDecision::Shockable) => RhythmDecision::Shockable,
            });
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
        match self
            .pads
            .update(pads_sensed(ohms), now, self.config.pad_debounce_ms)
        {
            Some(true) => self.events.push(now, Event::PadsOn),
            Some(false) => self.events.push(now, Event::PadsOff),
            None => {}
        }
        let poor_raw = pads_sensed(ohms) && ohms > MAX_PATIENT_OHMS;
        if self
            .poor_contact
            .update(poor_raw, now, self.config.pad_debounce_ms)
            == Some(true)
        {
            self.events.push(now, Event::PadContactPoor(ohms));
        }

        let cancel = self.read_cancel(hw, now);

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
        self.check_emptying(hw, now, voltage);

        match self.state {
            RescueState::SelfTest => self.tick_self_test(hw, now),
            RescueState::AttachPads => self.tick_attach_pads(hw, now),
            RescueState::Analysing => self.tick_analysing(hw, now, cancel),
            RescueState::AnalysingCharging => {
                self.tick_analysing_charging(hw, now, cancel, voltage);
            }
            RescueState::Charging => self.tick_charging(hw, now, cancel, voltage),
            RescueState::Countdown => self.tick_countdown(hw, now, cancel, voltage),
            RescueState::Delivering => self.tick_delivering(hw, now, cancel, voltage),
            RescueState::Cpr => self.tick_cpr(hw, now),
            RescueState::Fault => self.tick_fault(hw, now),
        }

        self.apply_outputs(hw);
        self.send_status_if_due(hw, now);
    }

    /// Read the cancel control: record presses, detect a stuck button, and
    /// return whether cancel is in force. During the switch-on self-test a
    /// press is recorded but ignored (D-31).
    fn read_cancel(&mut self, hw: &mut impl MainHardware, now: Millis) -> bool {
        let raw = hw.cancel_pressed();
        if raw && !self.cancel_was_pressed {
            self.events.push(
                now,
                if self.state == RescueState::SelfTest {
                    Event::CancelIgnoredDuringSelfTest
                } else {
                    Event::CancelPressed
                },
            );
        }
        self.cancel_was_pressed = raw;
        if raw {
            let since = *self.cancel_since.get_or_insert(now);
            let stuck = now.since(since) >= self.config.cancel_stuck_ms;
            if stuck
                && self.fault != Some(FaultKind::CancelStuck)
                && self.state != RescueState::Fault
            {
                self.events.push(now, Event::CancelStuck);
                self.begin_fault(FaultKind::CancelStuck, hw, now);
            }
        } else {
            self.cancel_since = None;
        }
        raw && self.state != RescueState::SelfTest
    }

    // ----- background emptying ----------------------------------------------

    /// Let go of the capacitor so it empties into the internal load, and
    /// start checking that it does. The caller moves on at once.
    fn start_emptying(&mut self, hw: &mut impl MainHardware, now: Millis) {
        hw.set_charger(None);
        hw.withdraw_shock_request();
        hw.set_discharge_hold(false);
        if self.emptying_since.is_none() {
            self.emptying_since = Some(now);
            self.events.push(now, Event::EmptyingStarted);
        }
    }

    /// Confirm a background emptying, or raise the serious fault if the
    /// capacitor is not empty in time.
    ///
    /// "Empty" needs the main controller's own reading to show empty and,
    /// so that the supervisor's own "charged too long" timer is also reset
    /// before a new charge starts, a supervisor report sent after emptying
    /// began whose reading shows empty too. If no such report comes by the
    /// time limit (supervisor link problems, which have their own faults),
    /// the main controller's own reading is enough.
    fn check_emptying(&mut self, hw: &mut impl MainHardware, now: Millis, voltage: Volts) {
        let Some(since) = self.emptying_since else {
            return;
        };
        let own_empty = self.config.reads_empty(voltage);
        let supervisor_empty = self.link.is_some_and(|l| {
            l.received_at >= since
                && now.since(l.received_at) <= self.config.heartbeat_timeout_ms
                && self.config.reads_empty(l.report.capacitor_voltage)
        });
        let overdue = now.since(since) > self.config.disarm_time_limit_ms;
        if own_empty && (supervisor_empty || overdue) {
            self.emptying_since = None;
            self.events.push(now, Event::CapacitorEmptied(voltage));
        } else if !own_empty && overdue {
            self.emptying_since = None;
            self.serious_fault(hw, now);
        }
    }

    /// The capacitor could not be confirmed empty: the most serious fault,
    /// from whatever state the device is in.
    fn serious_fault(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let kind = FaultKind::CapacitorNotEmptied;
        self.events.push(now, Event::Fault(kind));
        self.fault = Some(kind);
        if self.state == RescueState::Fault {
            self.last_reminder = now;
            hw.prompt(self.fault_prompt());
        } else {
            if self.state == RescueState::SelfTest {
                self.self_test.abort(hw);
            }
            self.go(RescueState::Fault, hw, now);
        }
        // Already tried and failed: do not start checking again.
        self.emptying_since = None;
    }

    // ----- state handlers -------------------------------------------------

    fn tick_self_test(&mut self, hw: &mut impl MainHardware, now: Millis) {
        // Cancel is ignored here (D-31): a stray press while the device is
        // drawn and switched on must not disable it.
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
            self.serious_fault(hw, now);
        } else {
            self.begin_fault(FaultKind::SelfTest(f), hw, now);
        }
    }

    fn tick_attach_pads(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if self.pads.on {
            self.enter_analysis(hw, now);
        } else if now.since(self.last_reminder) >= self.config.attach_pads_repeat_ms {
            self.last_reminder = now;
            hw.prompt(Prompt::AttachPads);
        }
    }

    /// Start an analysis: with charging during analysis (D-29), set up the
    /// charge first, then enter the analysing state.
    fn enter_analysis(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if self.config.charge_during_analysis {
            if !self.begin_arming(hw, now) {
                return;
            }
            self.events.push(now, Event::ChargingDuringAnalysis);
            self.go(RescueState::AnalysingCharging, hw, now);
        } else {
            self.go(RescueState::Analysing, hw, now);
        }
    }

    /// Set the charge target for the current mode and start a new arming.
    /// Returns false (after starting the fault path) if the planner's charge
    /// voltage does not fit the limits.
    fn begin_arming(&mut self, hw: &mut impl MainHardware, now: Millis) -> bool {
        let target = self.planner.charge_voltage(self.mode);
        // The whole "at the planned voltage" window must lie above "empty"
        // and below the overvoltage limit.
        let window = self.config.at_plan_allowance(target).0;
        let lowest_sensible = self.config.empty_voltage.0.saturating_add(window);
        let highest_sensible = self.config.overvoltage_limit.0.saturating_sub(window);
        if target.0 <= lowest_sensible || target.0 >= highest_sensible {
            self.begin_fault(FaultKind::PlanMismatch, hw, now);
            return false;
        }
        self.target = target;
        self.armed_since = Some(now);
        self.charged_at = None;
        self.charger_was_on = false;
        self.shocked_this_arming = false;
        self.mode_at_arming = self.mode;
        self.veto = false;
        self.plan = None;
        self.events.push(now, Event::ChargeStarted(target));
        true
    }

    /// Analysis decisions shared by both analysing states. Returns true if
    /// the state was left.
    fn analysis_step(&mut self, hw: &mut impl MainHardware, now: Millis) -> bool {
        match self.analysis_result.take() {
            Some(RhythmDecision::Shockable) => {
                self.shock_advised(hw, now);
                return true;
            }
            Some(RhythmDecision::NonShockable) => {
                hw.prompt(Prompt::NoShockAdvised);
                self.go(RescueState::Cpr, hw, now);
                return true;
            }
            Some(RhythmDecision::Indeterminate) => {
                // Not final: movement is the likely cause. Ask again for
                // hands off (not too often) and keep analysing.
                self.not_sure_seen = true;
                let due = self
                    .hands_off_reminded_at
                    .is_none_or(|t| now.since(t) >= self.config.hands_off_reminder_ms);
                if due {
                    self.hands_off_reminded_at = Some(now);
                    hw.prompt(Prompt::DoNotTouchPatient);
                }
            }
            None => {}
        }
        if self.time_in_state_ms(now) > self.config.analysis_time_limit_ms {
            // Neither "shock" nor "no shock" in time. Never leave the
            // responder waiting: say so and go to CPR (which says "Start CPR").
            self.events.push(
                now,
                if self.not_sure_seen {
                    Event::AnalysisInconclusive
                } else {
                    Event::AnalysisTimedOut
                },
            );
            hw.prompt(Prompt::AnalysisInconclusive);
            self.go(RescueState::Cpr, hw, now);
            return true;
        }
        false
    }

    /// A shock is advised: straight to the countdown if the capacitor is
    /// already charged, otherwise wait for the charge to finish.
    fn shock_advised(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if self.state == RescueState::Analysing && !self.begin_arming(hw, now) {
            return;
        }
        // From now on any other answer stops the shock.
        self.veto = false;
        if self.charged_at.is_some() && self.emptying_since.is_none() {
            hw.prompt(Prompt::ShockAdvised);
            self.go(RescueState::Countdown, hw, now);
        } else {
            hw.prompt(Prompt::ShockAdvisedCharging);
            self.go(RescueState::Charging, hw, now);
        }
    }

    fn tick_analysing(&mut self, hw: &mut impl MainHardware, now: Millis, cancel: bool) {
        if !self.pads.on {
            self.go(RescueState::AttachPads, hw, now);
        } else if cancel {
            hw.prompt(Prompt::AnalysisCancelled);
            self.go(RescueState::Cpr, hw, now);
        } else {
            self.analysis_step(hw, now);
        }
    }

    fn tick_analysing_charging(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        if !self.pads.on {
            self.go(RescueState::AttachPads, hw, now);
            return;
        }
        if cancel {
            hw.prompt(Prompt::AnalysisCancelled);
            self.go(RescueState::Cpr, hw, now);
            return;
        }
        if self.mode != self.mode_at_arming {
            // The switch was moved while analysing: empty the capacitor and
            // charge again for the new mode. The analysis itself carries on.
            self.burn_charge_if_any(now);
            self.start_emptying(hw, now);
            if !self.begin_arming(hw, now) {
                return;
            }
        }
        if let Some(kind) = self.charge_check(now, voltage) {
            self.begin_fault(kind, hw, now);
            return;
        }
        self.analysis_step(hw, now);
    }

    /// Charging progress and limits while the capacitor is meant to be
    /// charging or charged (charging during analysis, and Charging).
    /// Records "charged" the first time the plan is reached.
    fn charge_check(&mut self, now: Millis, voltage: Volts) -> Option<FaultKind> {
        if voltage > self.config.overvoltage_limit {
            return Some(FaultKind::Overvoltage);
        }
        let armed_for = self.armed_since.map_or(0, |t| now.since(t));
        if armed_for > self.config.max_charged_ms {
            return Some(FaultKind::ArmedTooLong);
        }
        // Readings only count once this arming's own charge is under way
        // (not while an earlier charge is still being emptied).
        let charging = self.emptying_since.is_none() && self.charger_was_on;
        match self.charged_at {
            None => {
                if charging && self.config.reached_plan(voltage, self.target) {
                    self.charged_at = Some(now);
                    self.events.push(now, Event::Charged(voltage));
                } else if armed_for > self.config.charge_time_limit_ms {
                    return Some(FaultKind::ChargeTimeout);
                }
            }
            Some(_) => {
                if !self.config.is_at_plan(voltage, self.target) {
                    return Some(FaultKind::ChargeOffTarget);
                }
            }
        }
        None
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
            return;
        }
        if let Some(kind) = self.charge_check(now, voltage) {
            self.begin_fault(kind, hw, now);
            return;
        }
        if self.charged_at.is_some() {
            self.go(RescueState::Countdown, hw, now);
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
        if !self.config.is_at_plan(voltage, self.target) {
            self.begin_fault(FaultKind::ChargeOffTarget, hw, now);
            return;
        }
        let elapsed = self.time_in_state_ms(now);
        if elapsed >= self.config.countdown_ms {
            self.deliver(hw, now);
            return;
        }
        // Speak the number of whole seconds left, rounded up: 3, 2, 1.
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
                if !pads_sensed(ohms) && self.pads.on {
                    // The fresh reading shows the pads off (or shorted)
                    // before the smoothed contact has caught up: treat it as
                    // pads off now, so the device asks for the pads at once.
                    self.pads.force(false);
                    self.events.push(now, Event::PadsOff);
                }
                self.abort(AbortReason::PlanRefused(e), hw, now);
            }
            Ok(plan) => {
                // Both are nominal numbers from the planner, not readings,
                // so they must match exactly.
                if plan.charge_voltage != self.target {
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
                self.shocked_this_arming = true;
                self.analyzer.reset();
                self.last_decision = None;
                hw.prompt(Prompt::ShockDelivered);
                self.go(RescueState::Cpr, hw, now);
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
                self.go(RescueState::Cpr, hw, now);
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
                    self.go(RescueState::Cpr, hw, now);
                }
            }
        }
    }

    fn tick_cpr(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if !self.pads.on {
            self.go(RescueState::AttachPads, hw, now);
            return;
        }
        self.metronome(hw, now);
        if self.poor_contact.on
            && now.since(self.press_pads_reminded_at) >= self.config.press_pads_repeat_ms
        {
            self.press_pads_reminded_at = now;
            hw.prompt(Prompt::PressPadsFirmly);
        }
        if self.time_in_state_ms(now) >= self.config.cpr_period_ms {
            self.enter_analysis(hw, now);
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
        // Leaving the charge-holding states (or a self-test that may have
        // charged), and on any fault: let go, and empty in the background.
        let was_charged = from.holds_charge() || from == RescueState::SelfTest;
        if !to.holds_charge() && (was_charged || to == RescueState::Fault) {
            if from.holds_charge() {
                self.burn_charge_if_any(now);
                self.armed_since = None;
                self.charged_at = None;
            }
            if from != RescueState::SelfTest || to == RescueState::Fault {
                self.start_emptying(hw, now);
            }
        }
        match to {
            RescueState::AttachPads => {
                self.last_reminder = now;
                if !self.pads.on {
                    hw.prompt(Prompt::AttachPads);
                }
            }
            RescueState::Analysing | RescueState::AnalysingCharging => {
                self.analyzer.reset();
                self.analysis_result = None;
                self.not_sure_seen = false;
                self.hands_off_reminded_at = None;
                self.last_decision = None;
                self.counters.analyses = self.counters.analyses.saturating_add(1);
                if self.poor_contact.on {
                    // Pads on but above the shock range (D-32): ask for them
                    // to be pressed down before the hands-off instruction.
                    self.press_pads_reminded_at = now;
                    hw.prompt(Prompt::PressPadsFirmly);
                }
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
            RescueState::Cpr => {
                self.next_beat = now;
                self.press_pads_reminded_at = now;
                self.counters.cpr_periods = self.counters.cpr_periods.saturating_add(1);
                hw.prompt(Prompt::StartCpr);
            }
            RescueState::Fault => {
                hw.set_charger(None);
                hw.withdraw_shock_request();
                hw.set_discharge_hold(false);
                hw.set_ready_indicator(false);
                self.next_beat = now;
                self.last_reminder = now;
                hw.prompt(self.fault_prompt());
            }
            RescueState::SelfTest | RescueState::Charging | RescueState::Delivering => {}
        }
    }

    /// Count a charge that is being emptied without a shock.
    fn burn_charge_if_any(&mut self, now: Millis) {
        if self.charger_was_on && !self.shocked_this_arming {
            self.counters.charges_burned = self.counters.charges_burned.saturating_add(1);
            self.events.push(now, Event::ChargeBurnedOff);
        }
        self.charger_was_on = false;
    }

    fn abort(&mut self, reason: AbortReason, hw: &mut impl MainHardware, now: Millis) {
        self.events.push(now, Event::ShockAborted(reason));
        self.not_given();
        hw.prompt(match reason {
            AbortReason::Cancelled => Prompt::ShockCancelled,
            AbortReason::RhythmChanged => Prompt::RhythmChangedNoShock,
            AbortReason::PlanRefused(PlanError::ImpedanceTooHigh) if self.pads.on => {
                Prompt::PadContactPoorNoShock
            }
            AbortReason::PadsOff | AbortReason::PlanRefused(_) => Prompt::CheckPads,
            AbortReason::ModeChanged | AbortReason::ConfirmationStale => Prompt::ShockNotDelivered,
        });
        // With the pads off the device can neither analyse nor shock, so the
        // one instruction is to put them back (finding F-6): straight to
        // "Attach pads", not "Start CPR" followed at once by "Attach pads".
        let next = if self.pads.on {
            RescueState::Cpr
        } else {
            RescueState::AttachPads
        };
        self.go(next, hw, now);
    }

    fn not_given(&mut self) {
        self.counters.shocks_not_given = self.counters.shocks_not_given.saturating_add(1);
    }

    /// Start the fault path: stop everything, empty the capacitor in the
    /// background, and go to the Fault state at once ("continue CPR").
    fn begin_fault(&mut self, kind: FaultKind, hw: &mut impl MainHardware, now: Millis) {
        if self.state == RescueState::Fault {
            return;
        }
        self.events.push(now, Event::Fault(kind));
        self.fault = Some(kind);
        if self.state == RescueState::SelfTest {
            self.self_test.abort(hw);
        }
        hw.withdraw_shock_request();
        self.go(RescueState::Fault, hw, now);
    }

    fn fault_prompt(&self) -> Prompt {
        match self.fault {
            Some(k) if k.is_serious() => Prompt::SeriousFaultContinueCpr,
            Some(FaultKind::CancelStuck | FaultKind::Supervisor(SupervisorFault::CancelStuck)) => {
                Prompt::CancelStuckContinueCpr
            }
            _ => Prompt::DeviceFaultContinueCpr,
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
    /// charging (and charge only once the supervisor reports it holds too);
    /// stop charging before releasing. While an earlier charge is still being
    /// emptied, hold nothing and charge nothing.
    fn apply_outputs(&mut self, hw: &mut impl MainHardware) {
        let emptying = self.emptying_since.is_some();
        match self.state {
            // The self-test sequencer drives the outputs itself.
            RescueState::SelfTest => {}
            RescueState::AnalysingCharging | RescueState::Charging | RescueState::Countdown
                if !emptying =>
            {
                hw.set_discharge_hold(true);
                // The charger cannot run until the supervisor holds too
                // (interlock rule), so ask for it only once the supervisor
                // reports that it is holding, in a report that arrived after
                // this charge was asked for (not one left over from the
                // self-test or an earlier charge).
                let since = self.armed_since;
                let supervisor_holding = self
                    .link
                    .is_some_and(|l| l.report.holding && since.is_some_and(|a| l.received_at > a));
                if supervisor_holding {
                    self.charger_was_on = true;
                    hw.set_charger(Some(self.target));
                } else {
                    hw.set_charger(None);
                }
            }
            RescueState::Delivering if !emptying => {
                hw.set_discharge_hold(true);
                hw.set_charger(None);
            }
            RescueState::AttachPads
            | RescueState::Analysing
            | RescueState::AnalysingCharging
            | RescueState::Charging
            | RescueState::Countdown
            | RescueState::Delivering
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
        let armed = self.state.holds_charge();
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
