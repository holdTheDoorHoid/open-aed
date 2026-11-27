// SPDX-License-Identifier: GPL-3.0-or-later
//! The self-test sequence.
//!
//! Run at power-on by the rescue sequence, and by the periodic tests. It
//! checks, in order:
//!
//! 1. the battery under load;
//! 2. that the supervisor is answering and holding its discharge path;
//! 3. (periodic tests only) that the adult/child switch's two contacts agree
//!    (D-60);
//! 4. **the interlock and the output stage, with no high voltage** (D-61,
//!    review R-13), using the supervisor's bounded test permit, with the
//!    capacitor empty and the patient relay at rest:
//!    * the relay's position sense reads "front ends" and the sequence
//!      logic is idle;
//!    * the request alone does not raise the interlock rail;
//!    * the test permit alone does not raise it;
//!    * request and test permit without the main controller's hold do not
//!      raise it;
//!    * request and test permit with both holds DO raise it, and the
//!      patient relay does not move;
//!    * each of the four bridge switches turns on and off (one step each,
//!      measured by the board layer into the internal test load);
//!    * a short test sequence latches and completes, and a second request
//!      within the same permit is locked out (one sequence per permit);
//!    * the lockout clears once both consents are removed;
//!    * (periodic tests only) the hardware time cap cuts a test sequence
//!      planned longer than the cap, and the hardware permit-duration cap
//!      drops a rail that is held up with no sequence;
//! 5. that the charger raises the capacitor to the test voltage in time;
//! 6. that the two controllers' voltage readings agree;
//! 7. that each discharge path, on its own, empties the capacitor at its
//!    own rate (path A: the main controller lets go; path B: the supervisor
//!    lets go when asked). A path that has failed open shows as slow;
//! 8. that both together empty it in time;
//! 9. that the pads are present and in date.
//!
//! **Why the caps are left to the periodic tests.** Everything in step 4
//! except the two cap checks costs a few tens of milliseconds and finds a
//! single fault that would otherwise stay hidden until a rescue (a shorted
//! consent switch, a stuck bridge switch, a relay that does not rest). The
//! two caps only matter if something else has already failed (a stuck
//! phase timer, a supervisor that holds its permit), and the permit-cap
//! check alone needs a quarter of a second, so they run in the daily and
//! weekly tests, where nobody is waiting. The per-path discharge check
//! adds about 0.1 s at switch-on (the two 100 ms windows also empty the
//! capacitor part of the way, so the final emptying is shorter); it runs at
//! every test because a failed path is otherwise invisible (review R-6,
//! R-9).
//!
//! The charge step's time limit is scaled to the test voltage
//! ([`Config::charge_time_limit_for`]): 1 s for the 400 V switch-on test, so
//! a weak charger is reported about a second into the test (finding F-7),
//! and about 11.4 s for a full-energy test at 1,950 V.
//!
//! Per SAFETY.md nothing here tests real hardware yet: it only puts the trait
//! calls in the right order with the right time limits.
//!
//! The sequence does not send heartbeats to the supervisor itself. Whoever
//! runs it must keep sending status messages (with state `SelfTest` and
//! [`SelfTest::test_request`]) so that the supervisor holds the discharge
//! path open during the charge step and gives the test permit when asked;
//! [`crate::rescue::Rescue`] and [`crate::schedule::PeriodicTestRun`] do
//! this, and send a message at once whenever the test request changes.

use crate::config::Config;
use crate::hal::{
    BatteryStatus, BridgeTestOutcome, BridgeTestStep, MainHardware, PadCartridge, RelayPosition,
    SequenceEnd, SequenceStatus,
};
use crate::link::{SupervisorReport, TestRequest};
use aed_types::{Decijoules, Micros, Millis, ShockPlan, Volts};

/// Which check failed. Each check has its own code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfTestFailure {
    /// The battery could not carry a realistic load.
    BatteryUnderLoad,
    /// No report arrived from the supervisor in time.
    SupervisorLinkDead,
    /// The supervisor answered but did not hold the discharge path open in
    /// time, so the charger could not run (see
    /// [`crate::interlock::charger_may_run`]).
    SupervisorNotHolding,
    /// The supervisor reported a latched fault.
    SupervisorFaultLatched,
    /// The adult/child switch's two contacts disagree (D-60). Periodic
    /// tests only: the device shows "not ready". In a rescue the device
    /// records it and carries on (the hardware ceiling falls back to the
    /// adult value; see [`crate::rescue`]).
    ModeSwitchContactsDisagree,
    /// The patient relay's position sense did not read "front ends" at rest.
    RelayNotAtFrontEnds,
    /// The sequence logic was not idle at rest (a lockout that never
    /// cleared, or a consent line stuck on).
    SequenceNotIdle,
    /// The interlock rail rose with the request alone.
    InterlockPassedRequestAlone,
    /// The supervisor did not give the test permit when asked.
    TestPermitRefused,
    /// The interlock rail rose with the test permit alone.
    InterlockPassedPermitAlone,
    /// The interlock rail rose with request and test permit while the main
    /// controller's discharge path was not held off.
    InterlockPassedWithoutHold,
    /// Request and test permit with both holds did not raise the rail.
    TestRailDidNotRise,
    /// The patient relay moved while only the test permit was given.
    RelayMovedUnderTestPermit,
    /// A bridge switch did not turn on or off as commanded (or the board
    /// layer could not run the step).
    BridgeSwitch(BridgeTestStep, BridgeTestOutcome),
    /// A short test sequence did not latch and complete.
    SequenceLatchFailed,
    /// A second request within the same permit was not locked out.
    LockoutFailed,
    /// The lockout did not clear once both consents were removed.
    LockoutDidNotClear,
    /// A test sequence planned longer than the hardware time cap was not cut
    /// by it (periodic tests).
    TimeCapDidNotTrip,
    /// The hardware time cap cut a test sequence more than 5 ms before its
    /// nominal time: it would cut real shocks short (periodic tests).
    TimeCapTooShort,
    /// A rail held up with no sequence was not dropped by the hardware's
    /// permit-duration cap (periodic tests).
    PermitCapDidNotTrip,
    /// The charger did not reach the test voltage in time.
    ChargeTimeout,
    /// The capacitor went over the overvoltage limit while charging.
    Overvoltage,
    /// The two controllers' voltage readings did not agree in time.
    VoltageReadingsDisagree,
    /// Discharge path A (the main controller's) alone emptied the capacitor
    /// too slowly: failed open.
    DischargePathASlow,
    /// The supervisor did not let go of path B when asked.
    PathBNotReleased,
    /// Discharge path B (the supervisor's) alone emptied the capacitor too
    /// slowly: failed open.
    DischargePathBSlow,
    /// The internal discharge did not empty the capacitor in time.
    DischargeTimeout,
    /// No pads are plugged in. Advisory: the rescue still continues.
    PadsMissing,
    /// The pads are past their use-by date. Advisory: the rescue still continues.
    PadsExpired,
    /// The pads have used up their heat allowance (D-52). Advisory, like
    /// expired pads: the rescue still continues.
    PadsHeatWorn,
    /// The test was stopped from outside before it finished (for example
    /// the battery failed or the cancel control was found stuck). The
    /// cancel control itself does NOT stop the self-test (decision D-31).
    Aborted,
}

impl SelfTestFailure {
    /// True for problems that are reported but do not stop the device being used.
    #[must_use]
    pub const fn is_advisory(self) -> bool {
        matches!(
            self,
            SelfTestFailure::PadsMissing
                | SelfTestFailure::PadsExpired
                | SelfTestFailure::PadsHeatWorn
        )
    }
}

/// A passed self-test, with any warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfTestPass {
    /// The battery passed but is low.
    pub battery_low: bool,
}

/// Where the self-test has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfTestProgress {
    /// Still running; call `step` again.
    Running,
    /// Finished with this result. The capacitor has been released to empty
    /// (but is not necessarily confirmed empty if the result is a failure).
    Finished(Result<SelfTestPass, SelfTestFailure>),
}

/// The latest supervisor report and when it arrived (on the main controller's clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkSnapshot {
    /// The report.
    pub report: SupervisorReport,
    /// When it arrived.
    pub received_at: Millis,
}

/// What a test-permit episode is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Episode {
    /// Permit alone, hold probe, test rail, switches, latch and lockout.
    Interlock,
    /// The hardware time cap.
    TimeCap,
    /// The hardware permit-duration cap.
    PermitCap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Battery,
    Link { since: Millis },
    ModeContacts { since: Millis },
    Relay,
    RequestAlone { since: Millis },
    AskPermit { since: Millis, episode: Episode },
    PermitAlone { since: Millis },
    HoldProbe { since: Millis },
    TestRail { since: Millis },
    Switch { index: usize },
    RailDown { since: Millis },
    LatchRun { since: Millis },
    Lockout { since: Millis, reraised: bool },
    Clear { since: Millis, after: Episode },
    TimeCap { since: Millis, active_at: Option<Millis> },
    PermitCap { since: Millis, armed_at: Option<Millis> },
    Charge { since: Millis },
    Agree { since: Millis },
    PathA { since: Millis, v0: Volts },
    PathBWait { since: Millis },
    PathB { since: Millis, v0: Volts },
    Discharge { since: Millis },
    Pads,
    Finished(Result<SelfTestPass, SelfTestFailure>),
}

/// What the request line should carry in a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Request {
    Off,
    NoSequence,
    Short,
    Long,
}

/// The self-test sequencer. Non-blocking: call [`SelfTest::step`] repeatedly.
#[derive(Debug, Clone)]
pub struct SelfTest {
    config: Config,
    test_voltage: Volts,
    step: Step,
    battery_low: bool,
    skip_charge: bool,
    heat_skip_allowed: bool,
    charge_skipped: bool,
    periodic: bool,
    raised: Request,
    path_failure: Option<SelfTestFailure>,
    interlock_ms: u32,
    interlock_started: Option<Millis>,
}

impl SelfTest {
    /// A self-test that will charge to `test_voltage` (use
    /// [`Config::self_test_voltage`] at power-on, or the full charge voltage for
    /// the periodic full-energy test of decision D-15). This is the
    /// switch-on form; [`SelfTest::periodic`] adds the periodic-only checks.
    #[must_use]
    pub const fn new(config: Config, test_voltage: Volts) -> Self {
        SelfTest {
            config,
            test_voltage,
            step: Step::Battery,
            battery_low: false,
            skip_charge: false,
            heat_skip_allowed: false,
            charge_skipped: false,
            periodic: false,
            raised: Request::Off,
            path_failure: None,
            interlock_ms: 0,
            interlock_started: None,
        }
    }

    /// The periodic (daily or weekly) form: also checks the adult/child
    /// switch's two contacts (D-60) and the two hardware caps (D-61).
    #[must_use]
    pub const fn periodic(config: Config, test_voltage: Volts) -> Self {
        let mut t = Self::new(config, test_voltage);
        t.periodic = true;
        t
    }

    /// The switch-on self-test of a device that is too hot to charge its
    /// capacitor (D-48, D-51): skip the charge, agreement and discharge
    /// steps (they charge the capacitor) and check everything else. Has an
    /// effect only before the charge step starts. Also lets a supervisor
    /// report that it is holding charging back for heat lead to the same
    /// skip instead of a "supervisor not holding" failure. Only the rescue
    /// uses it; a periodic test that cannot charge must not count as passed.
    ///
    /// The interlock steps are skipped with the charge: they need the
    /// supervisor holding its discharge path, which a supervisor that is
    /// too hot does not do. They run at the next switch-on or periodic
    /// test once the device has cooled.
    pub fn skip_charge_for_heat(&mut self) {
        self.heat_skip_allowed = true;
        if matches!(self.step, Step::Battery | Step::Link { .. }) {
            self.skip_charge = true;
        }
    }

    /// Allow the skip of [`SelfTest::skip_charge_for_heat`] if the
    /// supervisor reports it is too hot, without skipping on the main
    /// controller's own readings.
    pub fn allow_heat_skip(&mut self) {
        self.heat_skip_allowed = true;
    }

    /// True if the charge steps were skipped for heat: the charger, the
    /// voltage agreement and the discharge path were not tested.
    #[must_use]
    pub const fn charge_skipped(&self) -> bool {
        self.charge_skipped
    }

    /// How long the charge step may take for this test's voltage, ms.
    #[must_use]
    pub fn charge_time_limit_ms(&self) -> u32 {
        self.config.charge_time_limit_for(self.test_voltage)
    }

    /// The voltage this test charges to.
    #[must_use]
    pub const fn test_voltage(&self) -> Volts {
        self.test_voltage
    }

    /// True while the test wants the capacitor charged to the test voltage
    /// (the charge and agreement steps).
    #[must_use]
    pub const fn is_charging(&self) -> bool {
        matches!(self.step, Step::Charge { .. } | Step::Agree { .. })
    }

    /// How long the interlock steps took (milliseconds on the main
    /// controller's clock), once they have finished.
    #[must_use]
    pub const fn interlock_ms(&self) -> u32 {
        self.interlock_ms
    }

    /// What the test asks of the supervisor now; put it in every status
    /// message, and send one at once when it changes.
    #[must_use]
    pub const fn test_request(&self) -> TestRequest {
        match self.step {
            Step::AskPermit { .. }
            | Step::PermitAlone { .. }
            | Step::HoldProbe { .. }
            | Step::TestRail { .. }
            | Step::Switch { .. }
            | Step::RailDown { .. }
            | Step::LatchRun { .. }
            | Step::Lockout { .. }
            | Step::TimeCap { .. }
            | Step::PermitCap { .. } => TestRequest::TestPermit,
            // In the final emptying too, so that both paths empty the
            // capacitor together (the supervisor would otherwise hold path
            // B for as long as the main controller reports "self-test").
            Step::PathBWait { .. } | Step::PathB { .. } | Step::Discharge { .. } => {
                TestRequest::ReleasePathB
            }
            _ => TestRequest::None,
        }
    }

    /// Stop the test now and release the capacitor to empty.
    pub fn abort(&mut self, hw: &mut impl MainHardware) {
        hw.set_charger(None);
        hw.withdraw_shock_request();
        self.raised = Request::Off;
        hw.set_discharge_hold(false);
        self.step = Step::Finished(Err(SelfTestFailure::Aborted));
    }

    /// The test sequence plan: no charge, phase 1 of `phase1_us`, the
    /// normal gap and a 1 ms phase 2, into the internal test load.
    const fn test_plan(phase1_us: u32) -> ShockPlan {
        ShockPlan {
            charge_voltage: Volts(0),
            phase1: Micros(phase1_us),
            interphase: Micros(500),
            phase2: Micros(1_000),
            expected_energy: Decijoules(0),
        }
    }

    /// After the battery and the link: the next check.
    const fn after_link(&self, now: Millis) -> Step {
        if self.periodic {
            Step::ModeContacts { since: now }
        } else {
            Step::Relay
        }
    }

    /// After a cleared episode: the next episode, or the charge.
    fn after_episode(&mut self, done: Episode, now: Millis) -> Step {
        match (done, self.periodic) {
            (Episode::Interlock, true) => Step::AskPermit {
                since: now,
                episode: Episode::TimeCap,
            },
            (Episode::TimeCap, _) => Step::AskPermit {
                since: now,
                episode: Episode::PermitCap,
            },
            _ => {
                self.interlock_ms = self
                    .interlock_started
                    .map_or(0, |t| now.since(t));
                Step::Charge { since: now }
            }
        }
    }

    /// True if a fresh report, sent after `since`, shows the test permit
    /// given (`want` true) or withdrawn (`want` false).
    fn permit_reported(link: Option<LinkSnapshot>, since: Millis, want: bool) -> bool {
        link.is_some_and(|l| l.received_at > since && l.report.test_permit == want)
    }

    /// Advance the test. `link` is the latest supervisor report, if any.
    #[allow(clippy::too_many_lines)]
    pub fn step(
        &mut self,
        hw: &mut impl MainHardware,
        link: Option<LinkSnapshot>,
    ) -> SelfTestProgress {
        let now = hw.now();
        let c = self.config;
        let fresh_link = link.filter(|l| now.since(l.received_at) <= c.heartbeat_timeout_ms);
        let fail = |f| Step::Finished(Err(f));
        let next = match self.step {
            Step::Battery => match hw.battery_under_load() {
                BatteryStatus::Depleted => fail(SelfTestFailure::BatteryUnderLoad),
                BatteryStatus::Low => {
                    self.battery_low = true;
                    Step::Link { since: now }
                }
                BatteryStatus::Good => Step::Link { since: now },
            },
            // The rest only starts once the supervisor reports it is holding
            // the discharge path open: the interlock and the charger both
            // need it (interlock rules).
            Step::Link { since } => match fresh_link {
                Some(l) if l.report.fault.is_some() => fail(SelfTestFailure::SupervisorFaultLatched),
                Some(l) if self.skip_charge || (self.heat_skip_allowed && l.report.too_hot) => {
                    // The supervisor answers; charging waits for the device
                    // to cool (D-51). The pads are still checked.
                    self.charge_skipped = true;
                    Step::Pads
                }
                Some(l) if l.report.holding => self.after_link(now),
                Some(_) if now.since(since) > c.self_test_link_wait_ms => {
                    fail(SelfTestFailure::SupervisorNotHolding)
                }
                None if now.since(since) > c.self_test_link_wait_ms => {
                    fail(SelfTestFailure::SupervisorLinkDead)
                }
                Some(_) | None => Step::Link { since },
            },
            Step::ModeContacts { since } => {
                if hw.patient_mode() == hw.patient_mode_ceiling() {
                    Step::Relay
                } else if now.since(since) > c.mode_contacts_disagree_ms {
                    fail(SelfTestFailure::ModeSwitchContactsDisagree)
                } else {
                    Step::ModeContacts { since }
                }
            }
            Step::Relay => {
                self.interlock_started = Some(now);
                if hw.patient_relay() != RelayPosition::FrontEnds {
                    fail(SelfTestFailure::RelayNotAtFrontEnds)
                } else if hw.sequence_status() != SequenceStatus::Idle {
                    fail(SelfTestFailure::SequenceNotIdle)
                } else {
                    Step::RequestAlone { since: now }
                }
            }
            Step::RequestAlone { since } => {
                if hw.sequence_status() != SequenceStatus::Idle {
                    fail(SelfTestFailure::InterlockPassedRequestAlone)
                } else if now.since(since) >= c.interlock_probe_ms {
                    Step::AskPermit {
                        since: now,
                        episode: Episode::Interlock,
                    }
                } else {
                    Step::RequestAlone { since }
                }
            }
            Step::AskPermit { since, episode } => {
                if Self::permit_reported(fresh_link, since, true) {
                    match episode {
                        Episode::Interlock => Step::PermitAlone { since: now },
                        Episode::TimeCap => Step::TimeCap {
                            since: now,
                            active_at: None,
                        },
                        Episode::PermitCap => Step::PermitCap {
                            since: now,
                            armed_at: None,
                        },
                    }
                } else if now.since(since) > c.self_test_link_wait_ms {
                    fail(SelfTestFailure::TestPermitRefused)
                } else {
                    Step::AskPermit { since, episode }
                }
            }
            Step::PermitAlone { since } => {
                if hw.sequence_status() != SequenceStatus::Idle {
                    fail(SelfTestFailure::InterlockPassedPermitAlone)
                } else if now.since(since) >= c.interlock_probe_ms {
                    Step::HoldProbe { since: now }
                } else {
                    Step::PermitAlone { since }
                }
            }
            Step::HoldProbe { since } => {
                if hw.sequence_status() != SequenceStatus::Idle {
                    fail(SelfTestFailure::InterlockPassedWithoutHold)
                } else if now.since(since) >= c.interlock_probe_ms {
                    Step::TestRail { since: now }
                } else {
                    Step::HoldProbe { since }
                }
            }
            Step::TestRail { since } => {
                let status = hw.sequence_status();
                if hw.patient_relay() != RelayPosition::FrontEnds {
                    fail(SelfTestFailure::RelayMovedUnderTestPermit)
                } else if status == SequenceStatus::Armed {
                    Step::Switch { index: 0 }
                } else if now.since(since) > c.interlock_probe_ms {
                    fail(SelfTestFailure::TestRailDidNotRise)
                } else {
                    Step::TestRail { since }
                }
            }
            Step::Switch { index } => match BridgeTestStep::ALL.get(index) {
                Some(&s) => {
                    let r = hw.bridge_test_step(s);
                    if hw.patient_relay() != RelayPosition::FrontEnds {
                        fail(SelfTestFailure::RelayMovedUnderTestPermit)
                    } else if r.outcome == BridgeTestOutcome::Pass {
                        Step::Switch { index: index.saturating_add(1) }
                    } else {
                        fail(SelfTestFailure::BridgeSwitch(s, r.outcome))
                    }
                }
                None => Step::RailDown { since: now },
            },
            Step::RailDown { since } => {
                if hw.sequence_status() == SequenceStatus::Idle {
                    Step::LatchRun { since: now }
                } else if now.since(since) > c.interlock_probe_ms {
                    fail(SelfTestFailure::SequenceLatchFailed)
                } else {
                    Step::RailDown { since }
                }
            }
            Step::LatchRun { since } => match hw.sequence_status() {
                _ if hw.patient_relay() != RelayPosition::FrontEnds => {
                    fail(SelfTestFailure::RelayMovedUnderTestPermit)
                }
                SequenceStatus::Ended(SequenceEnd::Completed) => Step::Lockout {
                    since: now,
                    reraised: false,
                },
                SequenceStatus::Ended(_) | SequenceStatus::LockedOut => {
                    fail(SelfTestFailure::SequenceLatchFailed)
                }
                _ if now.since(since) > c.test_sequence_timeout_ms => {
                    fail(SelfTestFailure::SequenceLatchFailed)
                }
                _ => Step::LatchRun { since },
            },
            Step::Lockout { since, reraised } => {
                let status = hw.sequence_status();
                if !status.locked_out() {
                    fail(SelfTestFailure::LockoutFailed)
                } else if !reraised {
                    // One tick with the request withdrawn, then raise it again
                    // while the test permit is still given.
                    Step::Lockout {
                        since: now,
                        reraised: true,
                    }
                } else if now.since(since) >= c.interlock_probe_ms {
                    Step::Clear {
                        since: now,
                        after: Episode::Interlock,
                    }
                } else {
                    Step::Lockout { since, reraised }
                }
            }
            Step::Clear { since, after } => {
                if hw.sequence_status() == SequenceStatus::Idle
                    && Self::permit_reported(fresh_link, since, false)
                {
                    self.after_episode(after, now)
                } else if now.since(since) > c.self_test_link_wait_ms {
                    fail(SelfTestFailure::LockoutDidNotClear)
                } else {
                    Step::Clear { since, after }
                }
            }
            Step::TimeCap { since, active_at } => match hw.sequence_status() {
                SequenceStatus::Active => Step::TimeCap {
                    since,
                    active_at: active_at.or(Some(now)),
                },
                SequenceStatus::Ended(SequenceEnd::TimeCapTripped) => {
                    // The cap must have cut it close to its nominal time:
                    // allow two of the main controller's own sampling
                    // intervals of slack either side of what it can see.
                    let ran = active_at.map_or(0, |t| now.since(t));
                    if ran > c.hardware_time_cap_ms.saturating_add(5) {
                        fail(SelfTestFailure::TimeCapDidNotTrip)
                    } else if ran.saturating_add(5) < c.hardware_time_cap_ms {
                        fail(SelfTestFailure::TimeCapTooShort)
                    } else {
                        Step::Clear {
                            since: now,
                            after: Episode::TimeCap,
                        }
                    }
                }
                SequenceStatus::Ended(_) | SequenceStatus::LockedOut => {
                    fail(SelfTestFailure::TimeCapDidNotTrip)
                }
                _ if now.since(since) > c.test_sequence_timeout_ms.saturating_add(c.hardware_time_cap_ms) => {
                    fail(SelfTestFailure::TimeCapDidNotTrip)
                }
                _ => Step::TimeCap { since, active_at },
            },
            Step::PermitCap { since, armed_at } => {
                let limit = c.hardware_permit_cap_ms.saturating_add(c.interlock_probe_ms);
                match (hw.sequence_status(), armed_at) {
                    (SequenceStatus::Armed, _) if armed_at.is_none_or(|t| now.since(t) <= limit) => {
                        Step::PermitCap {
                            since,
                            armed_at: armed_at.or(Some(now)),
                        }
                    }
                    (SequenceStatus::LockedOut, Some(_)) => Step::Clear {
                        since: now,
                        after: Episode::PermitCap,
                    },
                    // The rail is still rising.
                    (SequenceStatus::Idle, None) if now.since(since) <= c.interlock_probe_ms => {
                        Step::PermitCap { since, armed_at }
                    }
                    // Held past the cap, dropped without the lockout, or
                    // never rose.
                    _ => fail(SelfTestFailure::PermitCapDidNotTrip),
                }
            }
            Step::Charge { since } => {
                let v = hw.capacitor_voltage();
                if v > c.overvoltage_limit {
                    fail(SelfTestFailure::Overvoltage)
                } else if c.reached_plan(v, self.test_voltage) {
                    Step::Agree { since: now }
                } else if now.since(since) > self.charge_time_limit_ms() {
                    fail(SelfTestFailure::ChargeTimeout)
                } else {
                    Step::Charge { since }
                }
            }
            Step::Agree { since } => {
                let v = hw.capacitor_voltage();
                let agrees = fresh_link.is_some_and(|l| {
                    l.received_at >= since && c.readings_agree(l.report.capacitor_voltage, v)
                });
                if v > c.overvoltage_limit {
                    fail(SelfTestFailure::Overvoltage)
                } else if agrees {
                    Step::PathA { since: now, v0: v }
                } else if now.since(since) > c.self_test_agree_wait_ms {
                    fail(SelfTestFailure::VoltageReadingsDisagree)
                } else {
                    Step::Agree { since }
                }
            }
            Step::PathA { since, v0 } => {
                if now.since(since) >= c.discharge_path_test_ms {
                    // A slow path is reported only after both paths have
                    // been let go together and the capacitor is confirmed
                    // empty (or not: that is the more serious failure).
                    if !Self::fell_enough(&c, v0, hw.capacitor_voltage()) {
                        self.path_failure = Some(SelfTestFailure::DischargePathASlow);
                    }
                    Step::PathBWait { since: now }
                } else {
                    Step::PathA { since, v0 }
                }
            }
            Step::PathBWait { since } => {
                let released = fresh_link.is_some_and(|l| l.received_at > since && !l.report.holding);
                if released {
                    Step::PathB {
                        since: now,
                        v0: hw.capacitor_voltage(),
                    }
                } else if now.since(since) > c.self_test_link_wait_ms {
                    self.path_failure.get_or_insert(SelfTestFailure::PathBNotReleased);
                    Step::Discharge { since: now }
                } else {
                    Step::PathBWait { since }
                }
            }
            Step::PathB { since, v0 } => {
                if now.since(since) >= c.discharge_path_test_ms {
                    if !Self::fell_enough(&c, v0, hw.capacitor_voltage()) {
                        self.path_failure.get_or_insert(SelfTestFailure::DischargePathBSlow);
                    }
                    Step::Discharge { since: now }
                } else {
                    Step::PathB { since, v0 }
                }
            }
            Step::Discharge { since } => {
                let v = hw.capacitor_voltage();
                if c.reads_empty(v) {
                    match self.path_failure {
                        Some(f) => fail(f),
                        None => Step::Pads,
                    }
                } else if now.since(since) > c.disarm_time_limit_ms {
                    fail(SelfTestFailure::DischargeTimeout)
                } else {
                    Step::Discharge { since }
                }
            }
            Step::Pads => match hw.pad_cartridge() {
                PadCartridge::Ok => Step::Finished(Ok(SelfTestPass {
                    battery_low: self.battery_low,
                })),
                PadCartridge::Missing => fail(SelfTestFailure::PadsMissing),
                PadCartridge::Expired => fail(SelfTestFailure::PadsExpired),
                PadCartridge::HeatWorn => fail(SelfTestFailure::PadsHeatWorn),
            },
            Step::Finished(result) => Step::Finished(result),
        };
        self.step = next;
        self.drive_outputs(hw);

        match self.step {
            Step::Finished(result) => SelfTestProgress::Finished(result),
            _ => SelfTestProgress::Running,
        }
    }

    /// True if the reading fell to at most the allowed share of `v0`.
    fn fell_enough(c: &Config, v0: Volts, v: Volts) -> bool {
        u32::from(v.0) * 100 <= u32::from(v0.0) * u32::from(c.discharge_path_max_remaining_percent)
    }

    /// Drive the outputs for the step we are now in. Hold first, then
    /// charge; stop charging first, then release. The request line follows
    /// the step, raised once per step that wants it.
    fn drive_outputs(&mut self, hw: &mut impl MainHardware) {
        let (hold, charge, request) = match self.step {
            Step::Charge { .. } | Step::Agree { .. } => (true, true, Request::Off),
            Step::RequestAlone { .. } | Step::TestRail { .. } | Step::Switch { .. } => {
                (true, false, Request::NoSequence)
            }
            Step::HoldProbe { .. } => (false, false, Request::NoSequence),
            Step::PermitCap { .. } => (true, false, Request::NoSequence),
            Step::LatchRun { .. } => (true, false, Request::Short),
            Step::Lockout { reraised, .. } => {
                (true, false, if reraised { Request::Short } else { Request::Off })
            }
            Step::TimeCap { .. } => (true, false, Request::Long),
            Step::Relay
            | Step::ModeContacts { .. }
            | Step::AskPermit { .. }
            | Step::PermitAlone { .. }
            | Step::RailDown { .. }
            | Step::Clear { .. }
            | Step::PathBWait { .. }
            | Step::PathB { .. } => (true, false, Request::Off),
            Step::Battery
            | Step::Link { .. }
            | Step::PathA { .. }
            | Step::Discharge { .. }
            | Step::Pads
            | Step::Finished(_) => (false, false, Request::Off),
        };
        if request != self.raised {
            hw.withdraw_shock_request();
            let cap_plus = self.config.hardware_time_cap_ms.saturating_add(15).saturating_mul(1_000);
            match request {
                Request::Off => {}
                Request::NoSequence => hw.request_shock(None),
                Request::Short => hw.request_shock(Some(&Self::test_plan(1_000))),
                Request::Long => hw.request_shock(Some(&Self::test_plan(cap_plus))),
            }
            self.raised = request;
        }
        if hold {
            hw.set_discharge_hold(true);
        }
        if charge {
            hw.set_charger(Some(self.test_voltage));
        } else {
            hw.set_charger(None);
        }
        if !hold {
            hw.set_discharge_hold(false);
        }
    }
}
