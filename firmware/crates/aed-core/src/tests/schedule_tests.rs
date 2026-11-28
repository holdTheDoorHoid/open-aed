// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for the periodic self-test schedule (decision D-15): the pure
//! schedule logic over simulated weeks, and one periodic test run on the
//! mock board with the real supervisor. Every name states the rule it
//! checks. Mock hardware only.

use super::mock::{Bench, Board, MainView, SupView};
use crate::hal::BatteryStatus;
use crate::schedule::{
    Deferral, LoggedTest, NotReadyReason, PeriodicOutcome, PeriodicTest, PeriodicTestRun,
    Readiness, TestSchedule, WakeInputs,
};
use crate::selftest::{SelfTestFailure, SelfTestPass};
use crate::supervisor::Supervisor;
use crate::{Config, ConfigError, Event, RescueState};
use aed_types::Volts;
use std::cell::Cell;
use std::rc::Rc;
use std::vec::Vec;

const HOUR: u32 = 3_600;
const DAY: u32 = 86_400;
const WEEK: u32 = 7 * DAY;

fn quiet(now_s: u32) -> WakeInputs {
    WakeInputs {
        now_s,
        clock_restarted: false,
        in_use: false,
        on_charge: false,
        temperature_c: 20,
        battery: BatteryStatus::Good,
    }
}

fn pass() -> PeriodicOutcome {
    PeriodicOutcome {
        result: Ok(SelfTestPass { battery_low: false }),
        capacitor_confirmed_empty: true,
        charged: true,
    }
}

fn fail(f: SelfTestFailure) -> PeriodicOutcome {
    PeriodicOutcome {
        result: Err(f),
        capacitor_confirmed_empty: true,
        charged: true,
    }
}

/// Wake every `step` seconds (plus `jitter(i)`) from `from` to `to`; run
/// whatever the schedule asks with the outcome `outcome(test, now)`, and
/// return (time, test) for every test run.
fn run_wakes(
    s: &mut TestSchedule,
    from: u32,
    to: u32,
    step: u32,
    mut inputs: impl FnMut(u32) -> WakeInputs,
    mut outcome: impl FnMut(PeriodicTest, u32) -> PeriodicOutcome,
) -> Vec<(u32, PeriodicTest)> {
    let mut ran = Vec::new();
    let mut t = from;
    while t < to {
        let d = s.on_wake(&inputs(t));
        if let Some(test) = d.run {
            s.record(t, test, &outcome(test, t));
            ran.push((t, test));
        }
        t += step;
    }
    ran
}

#[test]
fn default_schedule_is_full_energy_weekly_and_low_voltage_daily() {
    let c = super::cfg();
    assert_eq!(c.daily_test_interval_s, DAY);
    assert_eq!(c.full_test_interval_s, WEEK);
    assert_eq!(c.validate(), Ok(()));
}

#[test]
fn the_first_wake_runs_the_full_energy_test_which_also_counts_as_the_daily_check() {
    let mut s = TestSchedule::new(super::cfg());
    assert_eq!(
        s.readiness(0),
        Readiness::NotReady(NotReadyReason::NotYetTested)
    );
    let d = s.on_wake(&quiet(100));
    assert_eq!(d.run, Some(PeriodicTest::Full));
    s.record(100, PeriodicTest::Full, &pass());
    assert_eq!(s.readiness(101), Readiness::Ready);
    assert_eq!(s.daily_due_s(), 100 + DAY);
    assert_eq!(s.full_due_s(), 100 + WEEK);
    assert_eq!(s.on_wake(&quiet(200)).run, None);
}

#[test]
fn over_eight_quiet_weeks_one_full_test_a_week_and_a_daily_check_on_the_other_days() {
    let mut s = TestSchedule::new(super::cfg());
    let ran = run_wakes(&mut s, 0, 8 * WEEK, HOUR, quiet, |_, _| pass());
    let full = ran.iter().filter(|r| r.1 == PeriodicTest::Full).count();
    let daily = ran.iter().filter(|r| r.1 == PeriodicTest::Daily).count();
    assert_eq!(full, 8);
    assert_eq!(daily, 8 * 6);
    assert_eq!(s.counters().full_energy_cycles, 8);
    // Never two tests within a day, never more than a day apart.
    for w in ran.windows(2) {
        let gap = w[1].0 - w[0].0;
        assert_eq!(gap, DAY, "{w:?}");
    }
    assert_eq!(s.readiness(8 * WEEK), Readiness::Ready);
}

#[test]
fn late_wakes_do_not_make_the_schedule_drift() {
    // Wakes every hour, each up to 50 minutes late: the tests stay on the
    // daily grid from the first one (jitter-free), they do not creep later.
    let mut s = TestSchedule::new(super::cfg());
    let mut ran = Vec::new();
    let mut t = 0_u32;
    let mut i = 0_u32;
    while t < 4 * WEEK {
        let d = s.on_wake(&quiet(t));
        if let Some(test) = d.run {
            s.record(t, test, &pass());
            ran.push(t);
        }
        i += 1;
        t = i * HOUR + (i * 7_919) % 3_000;
    }
    for (n, at) in ran.iter().enumerate() {
        let grid = n as u32 * DAY;
        assert!(
            *at >= grid && *at < grid + HOUR,
            "test {n} at {at}, grid {grid}"
        );
    }
    assert_eq!(s.daily_due_s() % DAY, 0);
}

#[test]
fn a_test_missed_for_longer_than_its_interval_is_reanchored_without_a_burst_of_catch_up_tests() {
    let mut s = TestSchedule::new(super::cfg());
    let _ = run_wakes(&mut s, 0, DAY, HOUR, quiet, |_, _| pass());
    // No wake at all for ten days (for example a board fault), then hourly.
    let ran = run_wakes(&mut s, 11 * DAY + 600, 13 * DAY, HOUR, quiet, |_, _| pass());
    let times: Vec<u32> = ran.iter().map(|r| r.0).collect();
    assert_eq!(
        times,
        [11 * DAY + 600, 12 * DAY + 600],
        "one test, then a day later"
    );
    assert_eq!(
        ran[0].1,
        PeriodicTest::Full,
        "the overdue full-energy test runs first"
    );
    assert_eq!(s.counters().missed_daily, 1);
    assert_eq!(s.counters().missed_full, 1);
}

#[test]
fn in_use_puts_tests_off_and_the_daily_check_runs_at_the_first_wake_after_use() {
    let mut s = TestSchedule::new(super::cfg());
    let _ = run_wakes(&mut s, 0, DAY / 2, HOUR, quiet, |_, _| pass());
    // In use from 23 h to 26 h: the daily check due at 24 h is put off.
    let mut deferred = 0;
    for h in 23..26 {
        let d = s.on_wake(&WakeInputs {
            in_use: true,
            ..quiet(h * HOUR)
        });
        assert_eq!(d.run, None);
        if d.deferred == Some((PeriodicTest::Daily, Deferral::InUse)) {
            deferred += 1;
            assert!(d.next_wake_in_s <= HOUR);
        }
    }
    assert_eq!(deferred, 2);
    let d = s.on_wake(&quiet(26 * HOUR));
    assert_eq!(d.run, Some(PeriodicTest::Daily));
    s.record(26 * HOUR, PeriodicTest::Daily, &pass());
    assert_eq!(s.readiness(26 * HOUR), Readiness::Ready);
    // A use that ends early in the day also gets a check at once.
    let mut s2 = s.clone();
    let _ = s2.on_wake(&WakeInputs {
        in_use: true,
        ..quiet(30 * HOUR)
    });
    assert_eq!(s2.on_wake(&quiet(31 * HOUR)).run, Some(PeriodicTest::Daily));
}

#[test]
fn the_full_energy_test_is_put_off_when_too_cold_too_hot_or_the_battery_is_not_good_but_the_daily_check_still_runs(
) {
    for (temp, battery, why) in [
        (-10, BatteryStatus::Good, Deferral::Temperature),
        (60, BatteryStatus::Good, Deferral::Temperature),
        (20, BatteryStatus::Low, Deferral::Battery),
        (20, BatteryStatus::Depleted, Deferral::Battery),
    ] {
        let mut s = TestSchedule::new(super::cfg());
        let w = WakeInputs {
            temperature_c: temp,
            battery,
            ..quiet(0)
        };
        let d = s.on_wake(&w);
        assert_eq!(d.deferred, Some((PeriodicTest::Full, why)));
        assert_eq!(
            d.run,
            Some(PeriodicTest::Daily),
            "daily check runs at {temp} °C, battery {battery:?}"
        );
        assert!(d.next_wake_in_s <= HOUR);
        s.record(0, PeriodicTest::Daily, &pass());
        // Without a full-energy test ever passed the device is not ready.
        assert_eq!(
            s.readiness(10),
            Readiness::NotReady(NotReadyReason::NotYetTested)
        );
        // Once conditions allow, it runs.
        assert_eq!(s.on_wake(&quiet(HOUR)).run, Some(PeriodicTest::Full));
    }
}

#[test]
fn a_test_put_off_past_its_grace_time_is_counted_missed_and_the_device_shows_not_ready() {
    let c = super::cfg();
    let mut s = TestSchedule::new(c);
    let _ = run_wakes(&mut s, 0, HOUR, HOUR, quiet, |_, _| pass());
    // A hot summer: the full-energy test cannot run for three weeks.
    let hot = |t| WakeInputs {
        temperature_c: 55,
        ..quiet(t)
    };
    let _ = run_wakes(&mut s, HOUR, 3 * WEEK, HOUR, hot, |_, _| pass());
    assert_eq!(s.counters().missed_full, 1, "counted once, not every wake");
    assert_eq!(
        s.counters().missed_daily,
        0,
        "the daily checks kept running"
    );
    assert_eq!(
        s.readiness(3 * WEEK),
        Readiness::NotReady(NotReadyReason::Overdue(PeriodicTest::Full))
    );
    // Still ready just inside the grace time.
    let mut s = TestSchedule::new(c);
    let _ = run_wakes(&mut s, 0, HOUR, HOUR, quiet, |_, _| pass());
    let edge = WEEK + c.test_overdue_grace_s;
    let _ = run_wakes(&mut s, HOUR, edge, HOUR, hot, |_, _| pass());
    assert_eq!(s.readiness(edge), Readiness::Ready);
    assert_eq!(
        s.readiness(edge + 1),
        Readiness::NotReady(NotReadyReason::Overdue(PeriodicTest::Full))
    );
}

#[test]
fn on_the_charging_dock_the_tests_still_run() {
    let mut s = TestSchedule::new(super::cfg());
    let docked = |t| WakeInputs {
        on_charge: true,
        ..quiet(t)
    };
    let ran = run_wakes(&mut s, 0, 2 * WEEK, HOUR, docked, |_, _| pass());
    assert_eq!(ran.len(), 14);
    assert_eq!(s.counters().tests_on_charge, 14);
}

#[test]
fn a_failed_full_energy_test_shows_not_ready_is_retried_after_a_day_and_a_pass_clears_it() {
    let mut s = TestSchedule::new(super::cfg());
    let ran = run_wakes(&mut s, 0, 3 * DAY, HOUR, quiet, |test, t| {
        if test == PeriodicTest::Full && t < DAY {
            fail(SelfTestFailure::ChargeTimeout)
        } else {
            pass()
        }
    });
    let full: Vec<u32> = ran
        .iter()
        .filter(|r| r.1 == PeriodicTest::Full)
        .map(|r| r.0)
        .collect();
    assert_eq!(full, [0, DAY], "retried after a day, not sooner");
    // The failed full-energy test did not count as the daily check: that
    // ran at the next wake.
    assert_eq!(ran[1], (HOUR, PeriodicTest::Daily));
    assert_eq!(s.counters().full_failed, 1);
    assert_eq!(s.counters().full_passed, 1);
    assert_eq!(s.readiness(3 * DAY), Readiness::Ready);
    // While it was failed:
    let mut s = TestSchedule::new(super::cfg());
    let _ = s.on_wake(&quiet(0));
    s.record(0, PeriodicTest::Full, &fail(SelfTestFailure::ChargeTimeout));
    assert_eq!(
        s.readiness(1),
        Readiness::NotReady(NotReadyReason::TestFailed(
            PeriodicTest::Full,
            SelfTestFailure::ChargeTimeout
        ))
    );
    // The failure stays in the record.
    assert!(s
        .log()
        .any(|e| e.result == Err(SelfTestFailure::ChargeTimeout)
            && e.test == LoggedTest::Periodic(PeriodicTest::Full)));
}

#[test]
fn a_daily_pass_does_not_clear_a_failed_full_energy_test() {
    let mut s = TestSchedule::new(super::cfg());
    let _ = s.on_wake(&quiet(0));
    s.record(
        0,
        PeriodicTest::Full,
        &fail(SelfTestFailure::VoltageReadingsDisagree),
    );
    let d = s.on_wake(&quiet(HOUR));
    assert_eq!(d.run, Some(PeriodicTest::Daily));
    s.record(HOUR, PeriodicTest::Daily, &pass());
    assert!(matches!(
        s.readiness(HOUR),
        Readiness::NotReady(NotReadyReason::TestFailed(PeriodicTest::Full, _))
    ));
}

#[test]
fn a_failed_daily_check_is_retried_after_an_hour_and_a_full_pass_clears_it() {
    let mut s = TestSchedule::new(super::cfg());
    let _ = run_wakes(&mut s, 0, HOUR, HOUR, quiet, |_, _| pass());
    let _ = s.on_wake(&quiet(DAY));
    s.record(
        DAY,
        PeriodicTest::Daily,
        &fail(SelfTestFailure::BatteryUnderLoad),
    );
    assert!(matches!(
        s.readiness(DAY + 1),
        Readiness::NotReady(NotReadyReason::TestFailed(PeriodicTest::Daily, _))
    ));
    assert_eq!(s.on_wake(&quiet(DAY + HOUR / 2)).run, None);
    assert_eq!(s.on_wake(&quiet(DAY + HOUR)).run, Some(PeriodicTest::Daily));
    s.record(DAY + HOUR, PeriodicTest::Daily, &pass());
    assert_eq!(s.readiness(DAY + HOUR), Readiness::Ready);
}

#[test]
fn a_capacitor_not_confirmed_empty_after_a_test_is_not_ready_with_the_alarm_and_stops_further_tests_until_service(
) {
    let mut s = TestSchedule::new(super::cfg());
    let _ = s.on_wake(&quiet(0));
    s.record(
        0,
        PeriodicTest::Full,
        &PeriodicOutcome {
            result: Err(SelfTestFailure::DischargeTimeout),
            capacitor_confirmed_empty: false,
            charged: true,
        },
    );
    assert!(s.alarm());
    assert_eq!(
        s.readiness(1),
        Readiness::NotReady(NotReadyReason::CapacitorNotConfirmedEmpty)
    );
    // No further test charges the capacitor until service.
    let ran = run_wakes(&mut s, HOUR, 3 * WEEK, HOUR, quiet, |_, _| pass());
    assert!(ran.is_empty());
    assert!(s.locked_out() && s.alarm());
    s.service_reset(3 * WEEK);
    assert_eq!(
        s.on_wake(&quiet(3 * WEEK + 1)).run,
        Some(PeriodicTest::Full)
    );
}

#[test]
fn a_timer_that_goes_backwards_or_restarts_makes_both_tests_due_and_the_device_not_ready_until_they_pass(
) {
    for restart in [true, false] {
        let mut s = TestSchedule::new(super::cfg());
        let _ = run_wakes(&mut s, 0, 3 * DAY, HOUR, quiet, |_, _| pass());
        assert_eq!(s.readiness(3 * DAY), Readiness::Ready);
        // Battery taken out: the timer starts again from a small number (or
        // the board says it restarted, with a reading that happens to be larger).
        let now = if restart { 5 * DAY } else { 50 };
        let d = s.on_wake(&WakeInputs {
            clock_restarted: restart,
            ..quiet(now)
        });
        assert_eq!(d.run, Some(PeriodicTest::Full));
        assert_eq!(
            s.readiness(now),
            Readiness::NotReady(NotReadyReason::NotYetTested)
        );
        s.record(now, PeriodicTest::Full, &pass());
        assert_eq!(s.readiness(now), Readiness::Ready);
        assert_eq!(s.counters().clock_restarts, 1);
    }
}

#[test]
fn the_switch_on_self_test_counts_as_the_daily_check_and_its_failure_shows_not_ready() {
    let mut s = TestSchedule::new(super::cfg());
    let _ = run_wakes(&mut s, 0, HOUR, HOUR, quiet, |_, _| pass());
    s.record_power_on(DAY - 10, Ok(SelfTestPass { battery_low: false }));
    assert_eq!(s.daily_due_s(), 2 * DAY);
    assert_eq!(s.counters().daily_passed, 1);
    s.record_power_on(DAY + 10, Err(SelfTestFailure::SupervisorLinkDead));
    assert!(matches!(
        s.readiness(DAY + 11),
        Readiness::NotReady(NotReadyReason::TestFailed(PeriodicTest::Daily, _))
    ));
    assert!(s.log().any(|e| e.test == LoggedTest::PowerOn));
}

#[test]
fn every_full_energy_test_that_charged_is_counted_as_capacitor_wear_and_rescue_charges_are_added() {
    let mut s = TestSchedule::new(super::cfg());
    let _ = s.on_wake(&quiet(0));
    s.record(0, PeriodicTest::Full, &fail(SelfTestFailure::ChargeTimeout));
    let _ = s.on_wake(&quiet(DAY));
    s.record(DAY, PeriodicTest::Full, &pass());
    let _ = s.on_wake(&quiet(2 * DAY));
    s.record(2 * DAY, PeriodicTest::Daily, &pass());
    // A full test that failed before charging (battery) is not a charge.
    let mut s2 = s.clone();
    s2.record(
        3 * DAY,
        PeriodicTest::Full,
        &PeriodicOutcome {
            charged: false,
            ..fail(SelfTestFailure::BatteryUnderLoad)
        },
    );
    assert_eq!(s.counters().full_energy_cycles, 2);
    assert_eq!(s2.counters().full_energy_cycles, 2);
    s.add_rescue_charges(3);
    assert_eq!(s.counters().rescue_charges, 3);
}

#[test]
fn a_test_that_is_never_reported_is_counted_as_interrupted_and_is_due_again() {
    let mut s = TestSchedule::new(super::cfg());
    assert_eq!(s.on_wake(&quiet(0)).run, Some(PeriodicTest::Full));
    // Switched on for a rescue mid-test: no outcome.
    assert_eq!(
        s.on_wake(&WakeInputs {
            in_use: true,
            ..quiet(60)
        })
        .run,
        None
    );
    assert_eq!(s.counters().interrupted, 1);
    assert_eq!(s.on_wake(&quiet(HOUR)).run, Some(PeriodicTest::Full));
}

#[test]
fn pads_missing_or_expired_or_a_low_battery_show_not_ready_but_count_as_a_hardware_pass() {
    let mut s = TestSchedule::new(super::cfg());
    let _ = s.on_wake(&quiet(0));
    s.record(
        0,
        PeriodicTest::Full,
        &PeriodicOutcome {
            result: Err(SelfTestFailure::PadsExpired),
            ..pass()
        },
    );
    assert_eq!(
        s.readiness(1),
        Readiness::NotReady(NotReadyReason::Pads(SelfTestFailure::PadsExpired))
    );
    assert_eq!(
        s.full_due_s(),
        WEEK,
        "hardware passed: next full test in a week"
    );
    let _ = s.on_wake(&quiet(DAY));
    s.record(
        DAY,
        PeriodicTest::Daily,
        &PeriodicOutcome {
            result: Ok(SelfTestPass { battery_low: true }),
            ..pass()
        },
    );
    assert_eq!(
        s.readiness(DAY),
        Readiness::NotReady(NotReadyReason::BatteryLow)
    );
}

#[test]
fn self_test_schedule_settings_that_contradict_each_other_are_refused() {
    let c = super::cfg();
    for bad in [
        Config {
            daily_test_interval_s: 0,
            ..c
        },
        Config {
            full_test_interval_s: HOUR,
            ..c
        },
        Config {
            failed_daily_retry_s: 2 * DAY,
            ..c
        },
        Config {
            failed_full_retry_s: 2 * WEEK,
            ..c
        },
        Config {
            deferred_recheck_s: 0,
            ..c
        },
        Config {
            full_test_min_temp_c: 50,
            ..c
        },
    ] {
        assert_eq!(bad.validate(), Err(ConfigError::SelfTestSchedule));
    }
}

// ----- one periodic test on the mock board, with the real supervisor ----------------

fn run_periodic(board: &mut Board, test: PeriodicTest) -> (PeriodicOutcome, f32) {
    let config = super::cfg();
    let mut sup = Supervisor::new(config).expect("valid config");
    let mut run = PeriodicTestRun::new(config, test, Volts(1_800));
    let mut highest = 0.0_f32;
    for _ in 0..15_000 {
        board.now += 1;
        board.step(1);
        highest = highest.max(board.cap_v);
        if board.now.is_multiple_of(2) {
            if let Some(o) = run.step(&mut MainView(board)) {
                return (o, highest);
            }
        }
        sup.tick(&mut SupView(board));
    }
    panic!("periodic test did not finish");
}

fn board() -> Board {
    Board::new(Rc::new(Cell::new(None)))
}

#[test]
fn the_weekly_test_charges_to_full_energy_with_the_supervisor_holding_and_ends_with_the_capacitor_empty(
) {
    let mut b = board();
    let (o, highest) = run_periodic(&mut b, PeriodicTest::Full);
    assert_eq!(o.result, Ok(SelfTestPass { battery_low: false }));
    assert!(o.charged && o.capacitor_confirmed_empty);
    assert!(highest > 1_750.0, "charged to full energy: {highest} V");
    assert!(b.cap_v < 50.0);
    assert!(b.statuses.iter().all(|s| s.state == RescueState::SelfTest));
}

#[test]
fn the_daily_check_charges_only_to_the_low_test_voltage() {
    let mut b = board();
    let (o, highest) = run_periodic(&mut b, PeriodicTest::Daily);
    assert!(o.result.is_ok() && o.capacitor_confirmed_empty);
    assert!(highest < 450.0, "{highest} V");
}

#[test]
fn a_failed_periodic_test_leaves_the_capacitor_empty() {
    // A charger that stalls part way: the test fails at its charge step
    // and the capacitor is confirmed empty afterwards.
    let mut b = board();
    b.charge_rate = 0.05;
    let (o, highest) = run_periodic(&mut b, PeriodicTest::Full);
    assert_eq!(o.result, Err(SelfTestFailure::ChargeTimeout));
    assert!(highest > 400.0, "it did charge part way: {highest} V");
    assert!(o.capacitor_confirmed_empty);
    assert!(b.cap_v < 50.0);
    assert!(!b.main_hold && b.charger.is_none());
    // A broken discharge load: the capacitor cannot be confirmed empty.
    let mut b = board();
    b.dump_broken = true;
    let (o, _) = run_periodic(&mut b, PeriodicTest::Daily);
    assert_eq!(o.result, Err(SelfTestFailure::DischargeTimeout));
    assert!(!o.capacitor_confirmed_empty);
}

#[test]
fn a_device_shown_not_ready_still_runs_a_rescue_and_records_the_periodic_result() {
    // The weekly test failed; the device is switched on in an emergency.
    let mut s = TestSchedule::new(super::cfg());
    let _ = s.on_wake(&quiet(0));
    s.record(0, PeriodicTest::Full, &fail(SelfTestFailure::ChargeTimeout));
    let readiness = s.readiness(10);
    assert!(matches!(readiness, Readiness::NotReady(_)));
    let mut bench = Bench::new();
    bench
        .rescue
        .record_standby(aed_types::Millis(0), readiness, s.counters());
    // The fault has cleared (the switch-on self-test passes): it analyses
    // and shocks as usual.
    bench.run_to_countdown();
    bench.run_ms(5_000);
    assert_eq!(bench.shocks_fired(), 1);
    assert!(bench.has_event(|e| matches!(
        e,
        Event::StandbyRecord {
            readiness: Readiness::NotReady(_),
            full_energy_cycles: 1,
            ..
        }
    )));
    bench.assert_no_violations();
}
