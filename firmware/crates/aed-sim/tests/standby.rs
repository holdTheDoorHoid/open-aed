// SPDX-License-Identifier: GPL-3.0-or-later
//! The periodic self-test (decision D-15) over simulated weeks: a
//! full-energy test once a week and a low-voltage check every day, each run
//! on the simulated device with the real firmware and supervisor.
//!
//! SIMULATION ONLY: time between wakes is skipped, the device is a few
//! lines of arithmetic, the environment is scripted. Says nothing about a
//! real device.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, missing_docs)]

mod common;

use aed_core::{BatteryStatus, FaultKind, NotReadyReason, PeriodicTest, Readiness, RescueState, SelfTestFailure};
use aed_sim::standby::{run, Change, StandbyResult, StandbyScenario, DAY_S};
use aed_sim::{by_name, Fault};

const HOUR: u32 = 3_600;

fn summary(r: &StandbyResult) {
    let full = r.tests.iter().filter(|t| t.test == PeriodicTest::Full).count();
    let daily = r.tests.len() - full;
    println!(
        "{} tests ({full} full-energy, {daily} daily); {} full-energy charges; counters {:?} (SIMULATED)",
        r.tests.len(),
        r.counters.full_energy_cycles,
        r.counters
    );
    for t in &r.tests {
        if t.outcome.result.is_err() {
            println!(
                "  day {:.2}: {:?} {:?}, highest {:.0} V, true voltage at the end {:.0} V, confirmed empty {}",
                f64::from(t.at_s) / f64::from(DAY_S),
                t.test,
                t.outcome.result,
                t.highest_v,
                t.final_v,
                t.outcome.capacitor_confirmed_empty
            );
        }
    }
}

/// Every test left the capacitor truly empty (model voltage below 50 V),
/// unless the scenario breaks the discharge path.
fn assert_all_ended_empty(r: &StandbyResult) {
    for t in &r.tests {
        assert!(t.final_v < 50.0, "test at {} s ended at {:.0} V", t.at_s, t.final_v);
        assert!(t.outcome.capacitor_confirmed_empty);
    }
}

#[test]
fn eight_quiet_weeks_on_the_dock_give_one_full_energy_test_a_week_and_a_daily_check_on_the_other_days() {
    let r = run(&StandbyScenario::quiet(56)).unwrap();
    summary(&r);
    let full: Vec<_> = r.tests.iter().filter(|t| t.test == PeriodicTest::Full).collect();
    assert_eq!(full.len(), 8);
    assert_eq!(r.tests.len(), 56, "one test a day");
    for t in &full {
        assert!(t.outcome.result.is_ok());
        assert!(t.highest_v > 1_900.0, "charged to full energy: {:.0} V", t.highest_v);
        assert!(t.took_ms < 12_000, "{} ms", t.took_ms);
    }
    for t in r.tests.iter().filter(|t| t.test == PeriodicTest::Daily) {
        assert!(t.highest_v < 450.0, "the daily check stays at the low test voltage");
    }
    assert_eq!(r.counters.full_energy_cycles, 8);
    assert_all_ended_empty(&r);
    // Ready from the first test on.
    assert!(r.readiness.iter().all(|(_, x)| *x == Readiness::Ready));
}

#[test]
fn an_ageing_charger_fails_the_weekly_test_shows_not_ready_leaves_the_capacitor_empty_and_a_repair_clears_it() {
    let mut s = StandbyScenario::quiet(28);
    // From day 10 the charger gives only 12 W: the 400 V daily check still
    // passes, a full-energy charge takes too long.
    s.changes = vec![(10 * DAY_S, Change::ChargerWatts(12.0)), (20 * DAY_S + HOUR, Change::ChargerWatts(30.0))];
    let r = run(&s).unwrap();
    summary(&r);
    let failed: Vec<_> = r.tests.iter().filter(|t| t.outcome.result.is_err()).collect();
    assert!(!failed.is_empty());
    for t in &failed {
        assert_eq!(t.test, PeriodicTest::Full);
        assert_eq!(t.outcome.result, Err(SelfTestFailure::ChargeTimeout));
        assert!(t.highest_v > 1_000.0, "it charged part way");
    }
    // First failure at the weekly test on day 14; retried once a day.
    assert_eq!(failed[0].at_s, 14 * DAY_S);
    let days: Vec<u32> = failed.iter().map(|t| t.at_s / DAY_S).collect();
    assert_eq!(days, [14, 15, 16, 17, 18, 19, 20]);
    assert_all_ended_empty(&r);
    assert!(matches!(
        r.readiness_at(14 * DAY_S),
        Some(Readiness::NotReady(NotReadyReason::TestFailed(PeriodicTest::Full, SelfTestFailure::ChargeTimeout)))
    ));
    // The daily checks were not needed (the failed full test is retried
    // daily instead), and the device stays not ready until the repair.
    assert_eq!(r.readiness_at(21 * DAY_S - 1).map(|x| x == Readiness::Ready), Some(false));
    assert_eq!(r.readiness_at(21 * DAY_S), Some(Readiness::Ready));
    // Every attempt charged the capacitor: wear is counted.
    assert_eq!(r.counters.full_energy_cycles, 3 + 7, "passes on days 0, 7 and 21, failures on days 14-20");
}

#[test]
fn a_device_with_that_charger_still_guides_a_rescue_and_faults_to_cpr_and_shocks_once_repaired() {
    // Switched on in an emergency while "not ready" with the weak charger:
    // the switch-on self-test (400 V) passes, the analysis charges, the
    // charge times out, the device faults to CPR guidance. Never silent.
    let mut s = by_name("adult-vf").unwrap();
    s.plant.charger_watts = 12.0;
    let r = run_rescue(&s);
    assert!(r.shocks.is_empty());
    assert_eq!(r.final_fault, Some(FaultKind::ChargeTimeout));
    assert!(r.metronome_beats > 100, "CPR guidance continues");
    // Repaired: the same rescue shocks.
    s.plant.charger_watts = 30.0;
    let r = run_rescue(&s);
    assert_eq!(r.shocks.len(), 1);
}

fn run_rescue(s: &aed_sim::Scenario) -> aed_sim::RunResult {
    let r = aed_sim::run(s).unwrap();
    common::assert_invariants(&r);
    assert!(r.entered(RescueState::AttachPads), "switch-on self-test passed");
    r
}

#[test]
fn a_hot_spell_puts_the_weekly_test_off_counts_it_missed_shows_not_ready_and_it_runs_when_it_cools() {
    let mut s = StandbyScenario::quiet(21);
    s.changes = vec![(6 * DAY_S, Change::Temperature(55)), (16 * DAY_S + 5 * HOUR, Change::Temperature(25))];
    let r = run(&s).unwrap();
    summary(&r);
    // The full-energy test due on day 7 cannot run until it cools.
    let full: Vec<u32> = r.tests.iter().filter(|t| t.test == PeriodicTest::Full).map(|t| t.at_s).collect();
    assert_eq!(full[0], 0);
    assert_eq!(full[1], 16 * DAY_S + 5 * HOUR);
    // Daily checks kept running in the heat.
    assert!(r.tests.iter().any(|t| t.test == PeriodicTest::Daily && t.at_s / DAY_S == 10));
    assert_eq!(r.counters.missed_full, 1);
    // Due on day 7; one grace day; overdue from day 8 until it runs.
    assert_eq!(r.readiness_at(7 * DAY_S + 20 * HOUR), Some(Readiness::Ready), "inside the grace day");
    assert_eq!(
        r.readiness_at(8 * DAY_S + HOUR),
        Some(Readiness::NotReady(NotReadyReason::Overdue(PeriodicTest::Full)))
    );
    assert_eq!(
        r.readiness_at(16 * DAY_S + 4 * HOUR),
        Some(Readiness::NotReady(NotReadyReason::Overdue(PeriodicTest::Full)))
    );
    assert_eq!(r.readiness_at(16 * DAY_S + 5 * HOUR), Some(Readiness::Ready));
    assert_all_ended_empty(&r);
}

#[test]
fn a_test_due_while_the_device_is_in_use_runs_when_it_is_switched_off() {
    let mut s = StandbyScenario::quiet(4);
    s.changes = vec![(2 * DAY_S - 600, Change::InUseUntil(2 * DAY_S + 2 * HOUR))];
    let r = run(&s).unwrap();
    let day2: Vec<u32> = r.tests.iter().filter(|t| t.at_s / DAY_S == 2).map(|t| t.at_s).collect();
    assert_eq!(day2, [2 * DAY_S + 2 * HOUR], "at once after use");
    assert!(r.counters.deferrals >= 1);
}

#[test]
fn a_low_battery_puts_the_full_energy_test_off_but_not_the_daily_check() {
    let mut s = StandbyScenario::quiet(10);
    s.changes = vec![(6 * DAY_S, Change::Battery(BatteryStatus::Low)), (8 * DAY_S, Change::Battery(BatteryStatus::Good))];
    let r = run(&s).unwrap();
    let full: Vec<u32> = r.tests.iter().filter(|t| t.test == PeriodicTest::Full).map(|t| t.at_s).collect();
    assert_eq!(full, [0, 8 * DAY_S]);
    assert!(r.tests.iter().any(|t| t.test == PeriodicTest::Daily && t.at_s == 7 * DAY_S));
}

#[test]
fn a_stuck_discharge_path_found_by_the_daily_check_is_the_serious_not_ready_with_the_alarm() {
    let mut s = StandbyScenario::quiet(5);
    s.changes = vec![(3 * DAY_S - 60, Change::Fault(Fault::DischargeStuck))];
    let r = run(&s).unwrap();
    summary(&r);
    let t = r.tests.iter().find(|t| t.at_s == 3 * DAY_S).unwrap();
    assert_eq!(t.outcome.result, Err(SelfTestFailure::DischargeTimeout));
    assert!(!t.outcome.capacitor_confirmed_empty);
    assert_eq!(
        r.readiness_at(3 * DAY_S),
        Some(Readiness::NotReady(NotReadyReason::CapacitorNotConfirmedEmpty))
    );
    assert!(r.schedule.alarm());
    // No test runs again: it would charge a capacitor that may not empty.
    assert_eq!(r.tests.iter().filter(|t| t.at_s >= 3 * DAY_S).count(), 1);
    assert!(r.schedule.locked_out());
}

#[test]
fn the_timer_restarting_makes_both_tests_due_at_once() {
    let mut s = StandbyScenario::quiet(12);
    s.changes = vec![(9 * DAY_S + 3 * HOUR, Change::TimerRestartsAt(0))];
    let r = run(&s).unwrap();
    let after: Vec<_> = r.tests.iter().filter(|t| t.at_s >= 9 * DAY_S + 3 * HOUR).collect();
    assert_eq!(after[0].at_s, 9 * DAY_S + 3 * HOUR);
    assert_eq!(after[0].test, PeriodicTest::Full);
    assert_eq!(r.counters.clock_restarts, 1);
    assert_eq!(r.readiness.last().map(|x| x.1), Some(Readiness::Ready));
}
