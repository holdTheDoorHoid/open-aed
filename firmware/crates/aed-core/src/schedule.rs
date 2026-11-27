// SPDX-License-Identifier: GPL-3.0-or-later
//! The periodic self-test while the device is off (decision D-15, confirmed
//! by the owner on 27 November 2026): a **full-energy test once a week** and
//! a **low-voltage check every day**, both into the internal test load, with
//! the battery tested under load. The switch-on self-test stays the quick
//! low-voltage one.
//!
//! Two parts:
//!
//! * [`TestSchedule`]: pure logic. Each time the low-power timer wakes the
//!   device it is told the time and the conditions, and answers "run the
//!   daily check", "run the full-energy test" or "nothing", and when to wake
//!   next. It is told every result, keeps the record, counts full-energy
//!   charges (capacitor wear) and decides whether the ready indicator shows
//!   "ready".
//! * [`PeriodicTestRun`]: runs one test on the hardware, using the same
//!   [`SelfTest`] sequence as switch-on, with the supervisor holding the
//!   discharge path as it does at switch-on, and afterwards makes sure the
//!   capacitor is confirmed empty.
//!
//! # The clock
//!
//! No real-time clock is assumed. The board is expected to have a low-power
//! timer in the power domain that keeps counting **seconds** while the
//! device is off, and wakes the controllers when asked. Only differences
//! between its readings are used, never the time of day, so setting a clock
//! or changing time zone cannot move a test. If the count goes backwards
//! (the timer lost power, for example the battery was taken out) or the
//! board reports that the timer restarted, the schedule treats nothing as
//! known: both tests are due at once and the device is "not ready" until
//! they pass. `u32` seconds last 136 years.
//!
//! **Jitter-free:** the next due time is counted from when the test was
//! *due*, not from when the timer happened to wake, so late wakes do not
//! accumulate into drift. Only a test that runs more than one whole
//! interval late is re-anchored to when it actually ran (so a long-missed
//! test does not cause a burst of catch-up tests).
//!
//! # Decisions (provisional, Claude, 27 November 2026; see the README)
//!
//! * **Missed because the device is in use:** put off, checked again every
//!   [`Config::deferred_recheck_s`]. When the device is switched off after
//!   use, the daily check runs at the next wake (pads and battery may have
//!   been used).
//! * **On the charging dock:** both tests still run. A device kept on its
//!   dock (D-46) must not go untested. This needs the power design to let
//!   the test load be drawn from the battery while docked, so the battery
//!   is really tested under load: a requirement on the future hardware.
//! * **Too cold or too hot** (outside [`Config::full_test_min_temp_c`] to
//!   [`Config::full_test_max_temp_c`]): only the full-energy test is put
//!   off. The daily check runs at any temperature, because a device that
//!   does not work in the cold should say so. With the heat limits of D-41
//!   and D-48 ([`crate::Standby`], through [`TestSchedule::on_wake_gated`]):
//!   the full-energy test needs both cells trusted and within 0-45 °C and
//!   the capacitor at or below 50 °C (parts study R-H4); the daily check,
//!   which also charges the capacitor (to 400 V), needs the capacitor and
//!   cells at or below 60 °C and is put off above that; there is still no
//!   cold limit on it.
//! * **Battery low or depleted (power monitor, no load):** only the
//!   full-energy test is put off, so a test does not use energy a rescue
//!   may need. The daily check still runs (it will report the battery).
//! * **A test put off for too long** (no pass for its interval plus
//!   [`Config::test_overdue_grace_s`]) is counted as missed and the device
//!   shows "not ready".
//! * **A failed test** shows "not ready" and is retried: the daily check
//!   after [`Config::failed_daily_retry_s`] (1 h), the full-energy test after
//!   [`Config::failed_full_retry_s`] (1 day, to spare the capacitor). A pass
//!   clears it (a passed full-energy test also clears a failed daily check;
//!   a passed daily check does not clear a failed full-energy test). Every
//!   failure stays in the record and in the counters, so an intermittent
//!   fault is visible at service.
//! * **The capacitor not confirmed empty** after a test (or the discharge
//!   path failing its own step) is the most serious result: "not ready",
//!   [`TestSchedule::alarm`] asks for the alarm, and **no further automatic
//!   test runs** until [`TestSchedule::service_reset`]: every test charges
//!   the capacitor, and charging one that may not empty would only add
//!   stored energy. (The permanent bleeder resistor the safety rules ask for
//!   drains it meanwhile, slowly.)
//! * **A "not ready" device still runs a rescue when switched on.** The
//!   rescue sequence is not blocked by a periodic result: it runs its own
//!   switch-on self-test; if that passes, it analyses and shocks as usual
//!   (every check before a shock still applies, and the supervisor still
//!   guards the shock path); if it fails, the device guides CPR (metronome
//!   and "continue CPR") as for any fault. Reason: a responder with a
//!   flagged device is better served by one that tries than by one that
//!   refuses, a fault may have cleared, and blocking would turn one failed
//!   test into a certain "no shock". The periodic result is written into
//!   the rescue's event record at switch-on
//!   ([`crate::Rescue::record_standby`]) but not spoken.

use crate::config::Config;
use crate::hal::{BatteryStatus, MainHardware};
use crate::link::MainStatus;
use crate::rescue::RescueState;
use crate::selftest::{LinkSnapshot, SelfTest, SelfTestFailure, SelfTestPass, SelfTestProgress};
use aed_types::{Millis, Volts};

/// Which periodic test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodicTest {
    /// The low-voltage check (the switch-on self-test's voltage), daily.
    Daily,
    /// The full-energy test (the adult shock voltage), weekly.
    Full,
}

/// What the board tells the schedule at each wake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakeInputs {
    /// The low-power timer, seconds. Only differences are used.
    pub now_s: u32,
    /// The board knows the timer restarted since the last wake (for
    /// example after the battery was taken out).
    pub clock_restarted: bool,
    /// The device is switched on (a rescue, or service).
    pub in_use: bool,
    /// The device is on its charging dock. Recorded; not a reason to put a
    /// test off (see the module notes).
    pub on_charge: bool,
    /// Temperature inside the device, degrees Celsius.
    pub temperature_c: i8,
    /// The power monitor's view of the battery, without load.
    pub battery: BatteryStatus,
}

/// Why a due test was put off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deferral {
    /// The device is in use.
    InUse,
    /// Outside the full-energy test's temperature range.
    Temperature,
    /// The battery is not good enough for a full-energy test.
    Battery,
}

/// The schedule's answer to a wake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakeDecision {
    /// The test to run now, if any. Report its outcome with
    /// [`TestSchedule::record`].
    pub run: Option<PeriodicTest>,
    /// A due test that was put off, and why.
    pub deferred: Option<(PeriodicTest, Deferral)>,
    /// When to wake next, seconds from now (at least 1).
    pub next_wake_in_s: u32,
}

/// Which tests the temperature allows now (decided by the caller; see
/// [`crate::Standby`] for the heat limits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestGate {
    /// The full-energy test may charge the capacitor to full energy.
    pub full_allowed: bool,
    /// The daily check may charge the capacitor to the low test voltage.
    pub daily_allowed: bool,
}

/// The outcome of one periodic test on the hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodicOutcome {
    /// The self-test sequence's result.
    pub result: Result<SelfTestPass, SelfTestFailure>,
    /// The capacitor was confirmed empty afterwards (main controller's own
    /// reading, with sensor error allowed for).
    pub capacitor_confirmed_empty: bool,
    /// The test got as far as charging the capacitor (for the wear count).
    pub charged: bool,
}

/// Why the device is not ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotReadyReason {
    /// The capacitor could not be confirmed empty after a test. The most
    /// serious: the alarm should sound.
    CapacitorNotConfirmedEmpty,
    /// The latest test of this kind failed at this check.
    TestFailed(PeriodicTest, SelfTestFailure),
    /// This test has not passed for longer than its interval plus the
    /// grace time.
    Overdue(PeriodicTest),
    /// No full-energy test (or no daily check) has passed since the timer
    /// started counting.
    NotYetTested,
    /// The pads are missing or past their use-by date.
    Pads(SelfTestFailure),
    /// The last test found the battery low.
    BatteryLow,
}

/// What the ready indicator should show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    /// Every periodic test is passed and recent.
    Ready,
    /// Not ready, for this reason (the most serious one).
    NotReady(NotReadyReason),
}

/// Running totals, for the event record and service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TestCounters {
    /// Daily checks that passed (switch-on self-tests included).
    pub daily_passed: u32,
    /// Daily checks that failed (switch-on self-tests included).
    pub daily_failed: u32,
    /// Full-energy tests that passed.
    pub full_passed: u32,
    /// Full-energy tests that failed.
    pub full_failed: u32,
    /// Times the capacitor was charged towards full energy by a periodic
    /// test, whatever the result: the capacitor's wear from testing.
    pub full_energy_cycles: u32,
    /// Charges during rescues, reported with
    /// [`TestSchedule::add_rescue_charges`]: the rest of the wear.
    pub rescue_charges: u32,
    /// Wakes at which a due test was put off.
    pub deferrals: u32,
    /// Daily checks that went overdue (counted once each time).
    pub missed_daily: u32,
    /// Full-energy tests that went overdue (counted once each time).
    pub missed_full: u32,
    /// Tests handed out but never reported (for example the device was
    /// switched on in the middle of one).
    pub interrupted: u32,
    /// Times the timer was found to have restarted.
    pub clock_restarts: u32,
    /// Tests run while on the charging dock.
    pub tests_on_charge: u32,
}

/// What a log entry is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoggedTest {
    /// A periodic test.
    Periodic(PeriodicTest),
    /// The switch-on self-test of a rescue.
    PowerOn,
}

/// One entry in the test record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogEntry {
    /// Timer reading, seconds.
    pub at_s: u32,
    /// Which test.
    pub test: LoggedTest,
    /// Its result (`Ok` includes passes with a low battery).
    pub result: Result<(), SelfTestFailure>,
    /// The capacitor was confirmed empty afterwards.
    pub capacitor_confirmed_empty: bool,
}

/// How many results the record keeps (the newest).
pub const TEST_LOG_CAPACITY: usize = 32;

/// The periodic self-test schedule and record. Pure logic: no hardware,
/// no clock of its own. The board keeps it in memory that survives
/// power-off.
#[derive(Debug, Clone)]
pub struct TestSchedule {
    config: Config,
    started_at: Option<u32>,
    last_now: Option<u32>,
    was_in_use: bool,
    daily_due: u32,
    full_due: u32,
    last_daily_ok: Option<u32>,
    last_full_ok: Option<u32>,
    daily_failure: Option<SelfTestFailure>,
    full_failure: Option<SelfTestFailure>,
    pads_problem: Option<SelfTestFailure>,
    battery_low: bool,
    capacitor_unconfirmed: bool,
    locked_out: bool,
    pending: Option<PeriodicTest>,
    daily_missed_for: Option<u32>,
    full_missed_for: Option<u32>,
    counters: TestCounters,
    log: [Option<LogEntry>; TEST_LOG_CAPACITY],
    log_next: usize,
}

/// The next due time after a pass at `now` of a test that was due at
/// `due`: one interval after the due time, unless that is already past, in
/// which case one interval after now.
fn next_due(due: u32, now: u32, interval: u32) -> u32 {
    let on_grid = due.saturating_add(interval);
    if now < on_grid {
        on_grid
    } else {
        now.saturating_add(interval)
    }
}

impl TestSchedule {
    /// A schedule that has never woken. The first wake makes both tests due
    /// at once.
    #[must_use]
    pub const fn new(config: Config) -> Self {
        TestSchedule {
            config,
            started_at: None,
            last_now: None,
            was_in_use: false,
            daily_due: 0,
            full_due: 0,
            last_daily_ok: None,
            last_full_ok: None,
            daily_failure: None,
            full_failure: None,
            pads_problem: None,
            battery_low: false,
            capacitor_unconfirmed: false,
            locked_out: false,
            pending: None,
            daily_missed_for: None,
            full_missed_for: None,
            counters: TestCounters {
                daily_passed: 0,
                daily_failed: 0,
                full_passed: 0,
                full_failed: 0,
                full_energy_cycles: 0,
                rescue_charges: 0,
                deferrals: 0,
                missed_daily: 0,
                missed_full: 0,
                interrupted: 0,
                clock_restarts: 0,
                tests_on_charge: 0,
            },
            log: [None; TEST_LOG_CAPACITY],
            log_next: 0,
        }
    }

    /// The running totals.
    #[must_use]
    pub const fn counters(&self) -> TestCounters {
        self.counters
    }

    /// The test record, oldest first.
    pub fn log(&self) -> impl Iterator<Item = &LogEntry> {
        let (newer, older) = self
            .log
            .split_at_checked(self.log_next)
            .unwrap_or((self.log.as_slice(), &[]));
        older.iter().chain(newer.iter()).filter_map(Option::as_ref)
    }

    /// When the daily check is next due, timer seconds.
    #[must_use]
    pub const fn daily_due_s(&self) -> u32 {
        self.daily_due
    }

    /// When the full-energy test is next due, timer seconds.
    #[must_use]
    pub const fn full_due_s(&self) -> u32 {
        self.full_due
    }

    /// The board should sound the alarm (the capacitor was not confirmed
    /// empty after a test).
    #[must_use]
    pub const fn alarm(&self) -> bool {
        self.capacitor_unconfirmed
    }

    fn push_log(&mut self, entry: LogEntry) {
        if let Some(slot) = self.log.get_mut(self.log_next) {
            *slot = Some(entry);
        }
        self.log_next = (self.log_next + 1) % TEST_LOG_CAPACITY;
    }

    /// Forget every time: the timer restarted.
    fn restart_clock(&mut self, now: u32) {
        self.counters.clock_restarts = self.counters.clock_restarts.saturating_add(1);
        self.started_at = Some(now);
        self.daily_due = now;
        self.full_due = now;
        self.last_daily_ok = None;
        self.last_full_ok = None;
        self.daily_missed_for = None;
        self.full_missed_for = None;
    }

    /// The time by which `test` must have passed again.
    fn deadline(&self, test: PeriodicTest) -> u32 {
        let (last, interval) = match test {
            PeriodicTest::Daily => (self.last_daily_ok, self.config.daily_test_interval_s),
            PeriodicTest::Full => (self.last_full_ok, self.config.full_test_interval_s),
        };
        last.or(self.started_at)
            .unwrap_or(0)
            .saturating_add(interval)
            .saturating_add(self.config.test_overdue_grace_s)
    }

    fn count_missed(&mut self, now: u32) {
        for test in [PeriodicTest::Daily, PeriodicTest::Full] {
            let deadline = self.deadline(test);
            let (counted, missed) = match test {
                PeriodicTest::Daily => (&mut self.daily_missed_for, &mut self.counters.missed_daily),
                PeriodicTest::Full => (&mut self.full_missed_for, &mut self.counters.missed_full),
            };
            if now > deadline && *counted != Some(deadline) {
                *counted = Some(deadline);
                *missed = missed.saturating_add(1);
            }
        }
    }

    /// The low-power timer woke the device: decide what to do. The
    /// full-energy test's temperature window is judged from
    /// [`WakeInputs::temperature_c`] alone and the daily check runs at any
    /// temperature. [`crate::Standby`] uses [`TestSchedule::on_wake_gated`]
    /// instead, with the heat limits of D-41 and D-48.
    pub fn on_wake(&mut self, w: &WakeInputs) -> WakeDecision {
        let cfg = &self.config;
        let full_allowed =
            w.temperature_c >= cfg.full_test_min_temp_c && w.temperature_c <= cfg.full_test_max_temp_c;
        self.on_wake_gated(
            w,
            TestGate {
                full_allowed,
                daily_allowed: true,
            },
        )
    }

    /// Make the daily check due now (heat asks for a re-test after the
    /// device has cooled, or after a gap in its temperature record).
    pub fn request_daily_now(&mut self, now_s: u32) {
        self.daily_due = self.daily_due.min(now_s);
    }

    /// Every self-test reason the device is not ready at `now_s` (see
    /// [`crate::readiness`]). [`TestSchedule::readiness`] gives the single
    /// most serious one.
    #[must_use]
    pub fn readiness_reasons(&self, now_s: u32) -> crate::readiness::ReasonSet {
        use crate::readiness::Reason;
        let mut r = crate::readiness::ReasonSet::EMPTY;
        if self.capacitor_unconfirmed {
            r.insert(Reason::CapacitorNotConfirmedEmpty);
        }
        if self.full_failure.is_some() || self.daily_failure.is_some() {
            r.insert(Reason::SelfTestFailed);
        }
        if self.started_at.is_none() || self.last_full_ok.is_none() || self.last_daily_ok.is_none() {
            r.insert(Reason::SelfTestNotYetRun);
        }
        if self.started_at.is_some()
            && [PeriodicTest::Full, PeriodicTest::Daily]
                .iter()
                .any(|t| now_s > self.deadline(*t))
        {
            r.insert(Reason::SelfTestOverdue);
        }
        match self.pads_problem {
            Some(SelfTestFailure::PadsMissing) => r.insert(Reason::PadsMissing),
            Some(_) => r.insert(Reason::PadsExpired),
            None => {}
        }
        if self.battery_low {
            r.insert(Reason::BatteryLow);
        }
        r
    }

    /// As [`TestSchedule::on_wake`], with the decision on temperature made
    /// by the caller: `gate.full_allowed` false puts the full-energy test
    /// off, `gate.daily_allowed` false puts the daily check off as well
    /// (both with [`Deferral::Temperature`]).
    pub fn on_wake_gated(&mut self, w: &WakeInputs, gate: TestGate) -> WakeDecision {
        let now = w.now_s;
        match self.last_now {
            None => {
                self.started_at = Some(now);
                self.daily_due = now;
                self.full_due = now;
            }
            Some(last) if w.clock_restarted || now < last => self.restart_clock(now),
            Some(_) => {}
        }
        self.last_now = Some(now);
        if self.pending.take().is_some() {
            self.counters.interrupted = self.counters.interrupted.saturating_add(1);
        }
        if self.was_in_use && !w.in_use {
            // Just switched off after use: check it today, now.
            self.daily_due = self.daily_due.min(now);
        }
        self.was_in_use = w.in_use;
        self.count_missed(now);

        let full_due = now >= self.full_due;
        let daily_due = now >= self.daily_due;
        let mut deferred = None;
        let mut run = None;
        if self.locked_out {
            // The discharge path may not work: run nothing that charges.
        } else if w.in_use {
            if full_due {
                deferred = Some((PeriodicTest::Full, Deferral::InUse));
            } else if daily_due {
                deferred = Some((PeriodicTest::Daily, Deferral::InUse));
            }
        } else {
            if full_due {
                if !gate.full_allowed || !gate.daily_allowed {
                    deferred = Some((PeriodicTest::Full, Deferral::Temperature));
                } else if w.battery != BatteryStatus::Good {
                    deferred = Some((PeriodicTest::Full, Deferral::Battery));
                } else {
                    run = Some(PeriodicTest::Full);
                }
            }
            if run.is_none() && daily_due {
                if gate.daily_allowed {
                    run = Some(PeriodicTest::Daily);
                } else if deferred.is_none() {
                    deferred = Some((PeriodicTest::Daily, Deferral::Temperature));
                }
            }
        }
        if deferred.is_some() {
            self.counters.deferrals = self.counters.deferrals.saturating_add(1);
        }
        if run.is_some() && w.on_charge {
            self.counters.tests_on_charge = self.counters.tests_on_charge.saturating_add(1);
        }
        self.pending = run;

        let mut next = self.next_wake_in_s(now);
        if deferred.is_some() {
            next = next.min(self.config.deferred_recheck_s);
        }
        WakeDecision {
            run,
            deferred,
            next_wake_in_s: next,
        }
    }

    /// Seconds from `now_s` until the next test is due (at least 1, at most
    /// one daily interval; [`Config::deferred_recheck_s`] if one is due
    /// already). Ask again after recording a result.
    #[must_use]
    pub fn next_wake_in_s(&self, now_s: u32) -> u32 {
        let next = self
            .daily_due
            .min(self.full_due)
            .saturating_sub(now_s)
            .min(self.config.daily_test_interval_s);
        if next == 0 {
            self.config.deferred_recheck_s.max(1)
        } else {
            next
        }
    }

    /// Record the outcome of the test [`TestSchedule::on_wake`] handed out.
    pub fn record(&mut self, now_s: u32, test: PeriodicTest, outcome: &PeriodicOutcome) {
        self.pending = None;
        self.push_log(LogEntry {
            at_s: now_s,
            test: LoggedTest::Periodic(test),
            result: outcome.result.map(|_| ()),
            capacitor_confirmed_empty: outcome.capacitor_confirmed_empty,
        });
        if test == PeriodicTest::Full && outcome.charged {
            self.counters.full_energy_cycles = self.counters.full_energy_cycles.saturating_add(1);
        }
        self.capacitor_unconfirmed = !outcome.capacitor_confirmed_empty;
        if self.capacitor_unconfirmed || outcome.result == Err(SelfTestFailure::DischargeTimeout) {
            self.locked_out = true;
        }
        let hardware_ok = match outcome.result {
            Ok(_) => true,
            Err(f) => f.is_advisory(),
        };
        self.note_pads_and_battery(outcome.result);
        let c = self.config;
        match (test, hardware_ok) {
            (PeriodicTest::Daily, true) => self.daily_passed(now_s),
            (PeriodicTest::Daily, false) => {
                self.counters.daily_failed = self.counters.daily_failed.saturating_add(1);
                self.daily_failure = outcome.result.err();
                self.daily_due = now_s.saturating_add(c.failed_daily_retry_s);
            }
            (PeriodicTest::Full, true) => {
                self.counters.full_passed = self.counters.full_passed.saturating_add(1);
                self.full_failure = None;
                self.last_full_ok = Some(now_s);
                self.full_due = next_due(self.full_due, now_s, c.full_test_interval_s);
                // The full-energy test includes everything the daily check does.
                self.daily_failure = None;
                self.last_daily_ok = Some(now_s);
                self.daily_due = next_due(self.daily_due.min(now_s), now_s, c.daily_test_interval_s);
            }
            (PeriodicTest::Full, false) => {
                self.counters.full_failed = self.counters.full_failed.saturating_add(1);
                self.full_failure = outcome.result.err();
                self.full_due = now_s.saturating_add(c.failed_full_retry_s);
            }
        }
    }

    fn daily_passed(&mut self, now_s: u32) {
        self.counters.daily_passed = self.counters.daily_passed.saturating_add(1);
        self.daily_failure = None;
        self.last_daily_ok = Some(now_s);
        self.daily_due = next_due(self.daily_due, now_s, self.config.daily_test_interval_s);
    }

    fn note_pads_and_battery(&mut self, result: Result<SelfTestPass, SelfTestFailure>) {
        match result {
            Ok(pass) => {
                self.pads_problem = None;
                self.battery_low = pass.battery_low;
            }
            Err(f) if f.is_advisory() => {
                self.pads_problem = Some(f);
                self.battery_low = false;
            }
            Err(_) => {}
        }
    }

    /// Record a rescue's switch-on self-test (the same low-voltage test as
    /// the daily check, so a pass counts as the day's check). Call it when
    /// the device next reports to the power domain.
    pub fn record_power_on(&mut self, now_s: u32, result: Result<SelfTestPass, SelfTestFailure>) {
        self.push_log(LogEntry {
            at_s: now_s,
            test: LoggedTest::PowerOn,
            result: result.map(|_| ()),
            capacitor_confirmed_empty: !matches!(result, Err(SelfTestFailure::DischargeTimeout)),
        });
        if result == Err(SelfTestFailure::DischargeTimeout) {
            self.capacitor_unconfirmed = true;
            self.locked_out = true;
        }
        self.note_pads_and_battery(result);
        match result {
            Ok(_) => self.daily_passed(now_s),
            Err(f) if f.is_advisory() => self.daily_passed(now_s),
            Err(SelfTestFailure::Aborted) => {}
            Err(f) => {
                self.counters.daily_failed = self.counters.daily_failed.saturating_add(1);
                self.daily_failure = Some(f);
                self.daily_due = now_s.saturating_add(self.config.failed_daily_retry_s);
            }
        }
    }

    /// After service has checked the device (the discharge path in
    /// particular): clear every failure and the lock-out, and make both
    /// tests due at the next wake. The record and the counters are kept.
    pub fn service_reset(&mut self, now_s: u32) {
        self.locked_out = false;
        self.capacitor_unconfirmed = false;
        self.daily_failure = None;
        self.full_failure = None;
        self.daily_due = now_s;
        self.full_due = now_s;
    }

    /// True while automatic tests are stopped because the capacitor may not
    /// empty (see the module notes). Cleared only by
    /// [`TestSchedule::service_reset`].
    #[must_use]
    pub const fn locked_out(&self) -> bool {
        self.locked_out
    }

    /// Add the charges a rescue made (with charging during analysis, D-29,
    /// one per analysis) to the wear count.
    pub fn add_rescue_charges(&mut self, charges: u32) {
        self.counters.rescue_charges = self.counters.rescue_charges.saturating_add(charges);
    }

    /// What the ready indicator should show at timer reading `now_s`.
    #[must_use]
    pub fn readiness(&self, now_s: u32) -> Readiness {
        use NotReadyReason as R;
        let not = Readiness::NotReady;
        if self.capacitor_unconfirmed {
            return not(R::CapacitorNotConfirmedEmpty);
        }
        if let Some(f) = self.full_failure {
            return not(R::TestFailed(PeriodicTest::Full, f));
        }
        if let Some(f) = self.daily_failure {
            return not(R::TestFailed(PeriodicTest::Daily, f));
        }
        if self.started_at.is_none() {
            return not(R::NotYetTested);
        }
        for test in [PeriodicTest::Full, PeriodicTest::Daily] {
            if now_s > self.deadline(test) {
                return not(R::Overdue(test));
            }
        }
        if self.last_full_ok.is_none() || self.last_daily_ok.is_none() {
            return not(R::NotYetTested);
        }
        if let Some(f) = self.pads_problem {
            return not(R::Pads(f));
        }
        if self.battery_low {
            return not(R::BatteryLow);
        }
        Readiness::Ready
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Testing,
    Emptying {
        since: Millis,
        result: Result<SelfTestPass, SelfTestFailure>,
    },
    Done(PeriodicOutcome),
}

/// Runs one periodic test on the hardware: the switch-on [`SelfTest`]
/// sequence at the test's voltage, with status messages to the supervisor
/// (as state `SelfTest`, so it holds the discharge path during the charge),
/// then a check that the capacitor is empty. Non-blocking: call
/// [`PeriodicTestRun::step`] every few milliseconds until it returns the
/// outcome. The supervisor runs as usual alongside.
#[derive(Debug, Clone)]
pub struct PeriodicTestRun {
    config: Config,
    test: PeriodicTest,
    self_test: SelfTest,
    phase: Phase,
    charged: bool,
    link: Option<LinkSnapshot>,
    sequence: u32,
    last_status_at: Option<Millis>,
}

impl PeriodicTestRun {
    /// A test of kind `test`: the daily check charges to
    /// [`Config::self_test_voltage`], the full-energy test to `full_voltage`
    /// (the adult shock voltage, from the shock planner).
    #[must_use]
    pub const fn new(config: Config, test: PeriodicTest, full_voltage: Volts) -> Self {
        let v = match test {
            PeriodicTest::Daily => config.self_test_voltage,
            PeriodicTest::Full => full_voltage,
        };
        PeriodicTestRun {
            config,
            test,
            self_test: SelfTest::new(config, v),
            phase: Phase::Testing,
            charged: false,
            link: None,
            sequence: 0,
            last_status_at: None,
        }
    }

    /// Which test this is.
    #[must_use]
    pub const fn test(&self) -> PeriodicTest {
        self.test
    }

    /// The voltage this test charges to.
    #[must_use]
    pub const fn test_voltage(&self) -> Volts {
        self.self_test.test_voltage()
    }

    /// Stop the test now (for example the device is being switched on for a
    /// rescue) and let go of the capacitor. The outcome, once confirmed
    /// empty, is `Aborted`.
    pub fn abort(&mut self, hw: &mut impl MainHardware) {
        self.self_test.abort(hw);
        if matches!(self.phase, Phase::Testing) {
            self.phase = Phase::Emptying {
                since: hw.now(),
                result: Err(SelfTestFailure::Aborted),
            };
        }
    }

    /// Advance the test. Returns the outcome once finished (and keeps
    /// returning it).
    pub fn step(&mut self, hw: &mut impl MainHardware) -> Option<PeriodicOutcome> {
        let now = hw.now();
        if let Some(report) = hw.supervisor_report() {
            self.link = Some(LinkSnapshot {
                report,
                received_at: now,
            });
        }
        match self.phase {
            Phase::Testing => {
                let progress = self.self_test.step(hw, self.link);
                self.charged |= self.self_test.is_charging();
                if let SelfTestProgress::Finished(result) = progress {
                    self.phase = match result {
                        // The sequence's own discharge step confirmed empty.
                        Ok(_) => self.done(result, true),
                        Err(f) if f.is_advisory() => self.done(result, true),
                        // Emptying has already been tried and failed.
                        Err(SelfTestFailure::DischargeTimeout) => self.done(result, false),
                        // Failed before its discharge step: it has let go;
                        // confirm the capacitor empties.
                        Err(_) => Phase::Emptying { since: now, result },
                    };
                }
            }
            Phase::Emptying { since, result } => {
                hw.set_charger(None);
                hw.set_discharge_hold(false);
                if self.config.reads_empty(hw.capacitor_voltage()) {
                    self.phase = self.done(result, true);
                } else if now.since(since) > self.config.disarm_time_limit_ms {
                    self.phase = self.done(result, false);
                }
            }
            Phase::Done(_) => {
                hw.set_charger(None);
                hw.set_discharge_hold(false);
            }
        }
        self.send_status(hw, now);
        match self.phase {
            Phase::Done(o) => Some(o),
            _ => None,
        }
    }

    const fn done(&self, result: Result<SelfTestPass, SelfTestFailure>, empty: bool) -> Phase {
        Phase::Done(PeriodicOutcome {
            result,
            capacitor_confirmed_empty: empty,
            charged: self.charged,
        })
    }

    fn send_status(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let due = self
            .last_status_at
            .is_none_or(|t| now.since(t) >= self.config.heartbeat_period_ms);
        if !due {
            return;
        }
        self.last_status_at = Some(now);
        self.sequence = self.sequence.wrapping_add(1);
        let charging = matches!(self.phase, Phase::Testing) && self.self_test.is_charging();
        let capacitor_voltage = hw.capacitor_voltage();
        hw.send_status(&MainStatus {
            sequence: self.sequence,
            state: RescueState::SelfTest,
            capacitor_voltage,
            planned_voltage: if charging { self.self_test.test_voltage() } else { Volts(0) },
            armed_for_ms: 0,
            // The standby gate has already checked the temperature before
            // handing out this test; the supervisor judges its own reading
            // alone (a test refused for heat is reported as failed).
            temperature: None,
        });
    }
}
