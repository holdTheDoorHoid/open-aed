// SPDX-License-Identifier: GPL-3.0-or-later
//! Heat (D-41, D-48, D-51, D-52) and capacitor wear (D-53) on the simulated
//! device: rescues millisecond by millisecond, and months of standby with
//! the time between wakes skipped.
//!
//! SIMULATION ONLY: one-node thermal model with parts-study figures
//! (`src/thermal.rs`), scripted weather, cartoon patient. Says nothing about
//! a real device.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, missing_docs)]

use aed_core::{Config, HeatBand, Prompt, Reason, RescueState, TempPoint};
use aed_rhythm::synth::Rhythm;
use aed_sim::standby::{run as run_standby, Change, StandbyResult, StandbyScenario, DAY_S};
use aed_sim::thermal::TempSensorFault;
use aed_sim::{by_name, run, Action, Expect, Rule, Scenario, Simulation, Trigger};

const HOUR: u32 = 3_600;
const MIN: u32 = 60;

fn no_violations(r: &StandbyResult) {
    assert!(r.violations.is_empty(), "{:#?}", r.violations);
}

fn reasons_at(r: &StandbyResult, t: u32) -> Vec<Reason> {
    r.decision_at(t).unwrap().readiness.reasons.iter().collect()
}

fn ready_at(r: &StandbyResult, t: u32) -> bool {
    r.decision_at(t).unwrap().readiness.is_ready()
}

// ----- rescues -----------------------------------------------------------------------

#[test]
fn a_rescue_started_while_too_hot_guides_cpr_at_once_analyses_without_charging_then_shocks_once_cooled() {
    let r = run(&by_name("hot-device-rescue").unwrap()).unwrap();
    assert!(r.violations.is_empty(), "{:#?}", r.violations);
    assert_eq!(r.shocks.len(), 1);
    assert!(r.said(Prompt::DeviceTooHot));
    assert!(r.said(Prompt::ShockAdvisedTooHot));
    let held = r.prompts.iter().position(|(_, p)| *p == Prompt::ShockAdvisedTooHot).unwrap();
    assert_eq!(r.prompts[held + 1].1, Prompt::StartCpr, "CPR at once, never silent");
    assert!(r.shocks_held_for_heat >= 1);
    // No charge of any kind while it was too hot: the first charge is the
    // one for the shock.
    assert_eq!(r.charges_started, 2);
    let first_shock_s = f64::from(r.shocks[0].at_ms) / 1000.0;
    println!(
        "hot-device-rescue (SIMULATED): switched on at 63 °C in 25 °C air; {} shock(s) held for heat; \
         first shock at {first_shock_s:.0} s; device at {:.1} °C at the end",
        r.shocks_held_for_heat, r.final_interior_c
    );
}

#[test]
fn a_warm_device_charges_only_once_a_shock_is_advised() {
    let r = run(&by_name("warm-device-rescue").unwrap()).unwrap();
    assert!(r.violations.is_empty(), "{:#?}", r.violations);
    assert_eq!(r.shocks.len(), 1);
    assert!(r.said(Prompt::ShockAdvisedCharging));
    assert!(r.entered(RescueState::Charging));
    assert!(!r.entered(RescueState::AnalysingCharging));
}

#[test]
fn a_failed_main_temperature_sensor_or_a_supervisor_sensor_stuck_high_does_not_block_the_shock() {
    for name in ["temp-sensor-failed-main", "temp-sensor-stuck-supervisor"] {
        let r = run(&by_name(name).unwrap()).unwrap();
        assert!(r.violations.is_empty(), "{name}: {:#?}", r.violations);
        assert_eq!(r.shocks.len(), 1, "{name}");
    }
}

/// A long rescue with no shockable rhythm (every analysis burns a charge
/// inside the device, D-29) on a hot day: the device warms. Numbers are
/// printed for the report (SIMULATED).
fn long_no_shock(ambient: f64, start: f64, minutes: u32, config: Config, vf_from_min: Option<u32>) -> aed_sim::RunResult {
    let mut s = Scenario::base("long-no-shock", "long asystole rescue on a hot day");
    s.rhythm = Rhythm::Asystole;
    s.after_shock = vec![];
    s.ambient_c = ambient;
    s.interior_c = start;
    s.duration_ms = minutes * 60_000;
    s.expect = Expect::InvariantsOnly;
    if let Some(m) = vf_from_min {
        s.script.push((Trigger::At(m * 60_000), Action::SetRhythm(Rhythm::coarse_vf())));
        s.after_shock = vec![Some(Rhythm::Asystole)];
    }
    Simulation::with_config(&s, config).unwrap().run()
}

#[test]
fn a_long_no_shock_rescue_on_a_50_c_day_stays_below_the_60_c_limit_and_is_never_silent() {
    let with_rule = long_no_shock(50.0, 50.0, 30, Config::default(), None);
    let without = long_no_shock(
        50.0,
        50.0,
        30,
        Config {
            precharge_max: aed_core::DeciC::from_c(60),
            ..Config::default()
        },
        None,
    );
    for r in [&with_rule, &without] {
        assert!(r.violations.is_empty(), "{:#?}", r.violations);
        assert!(r.peak_interior_c < 60.0);
        assert_eq!(r.shocks.len(), 0);
    }
    println!(
        "30 min asystole at 50 °C (SIMULATED, one-node model): with the 55 °C pre-charge rule: \
         {} charges burned, peak {:.1} °C; charging during every analysis: {} charges burned, peak {:.1} °C",
        with_rule.charges_burned, with_rule.peak_interior_c, without.charges_burned, without.peak_interior_c
    );
}

#[test]
fn a_long_rescue_in_a_57_c_car_reaches_the_limit_only_without_the_precharge_rule_and_vf_is_then_not_silent() {
    // Without the 55 °C rule the burned charges take the device to 60 °C;
    // VF at minute 20 then gets "Shock advised. Device too hot to charge."
    let without = long_no_shock(
        57.0,
        58.0,
        26,
        Config {
            precharge_max: aed_core::DeciC::from_c(60),
            ..Config::default()
        },
        Some(20),
    );
    assert!(without.violations_of(Rule::ChargedWhenTooHot).is_empty());
    assert!(without.violations_of(Rule::LeftWithoutInstruction).is_empty());
    // With the rule it burns no charges above 55 °C and shocks the VF.
    let with_rule = long_no_shock(57.0, 58.0, 26, Config::default(), Some(20));
    assert!(with_rule.violations.is_empty(), "{:#?}", with_rule.violations);
    assert!(with_rule.peak_interior_c < 60.0);
    assert_eq!(with_rule.shocks.len(), 1, "VF at minute 20 is shocked");
    println!(
        "26 min in a 57 °C car, device at 58 °C, asystole then VF at 20 min (SIMULATED): without the \
         pre-charge rule peak {:.1} °C, {} charges burned, {} shock(s) held for heat, {} shock(s); with it \
         peak {:.1} °C, {} charges burned, {} held, {} shock(s)",
        without.peak_interior_c,
        without.charges_burned,
        without.shocks_held_for_heat,
        without.shocks.len(),
        with_rule.peak_interior_c,
        with_rule.charges_burned,
        with_rule.shocks_held_for_heat,
        with_rule.shocks.len()
    );
    if without.peak_interior_c > 60.0 && without.shocks.is_empty() {
        assert!(without.said(Prompt::ShockAdvisedTooHot), "never silent about a held shock");
    }
}

// ----- standby: weather -----------------------------------------------------------------

/// 25 °C, except a hot afternoon in a car on day 2: the cabin rises to 66 °C
/// from 12:00 to 17:00 (parts study H4: 60-65 °C at the peak of a hot day).
fn car_afternoon(t: u32) -> f64 {
    if (2 * DAY_S + 12 * HOUR..2 * DAY_S + 17 * HOUR).contains(&t) {
        66.0
    } else {
        25.0
    }
}

#[test]
fn a_hot_afternoon_in_a_car_goes_ready_not_ready_cooled_self_test_ready() {
    let s = StandbyScenario {
        weather: Some(car_afternoon),
        ..StandbyScenario::quiet(4)
    };
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    assert!(ready_at(&r, 2 * DAY_S + 11 * HOUR));
    // The device follows the cabin with its ~21-minute lag.
    let first_hot = r.temperatures.iter().find(|(_, c)| *c > 60.0).unwrap().0;
    assert!(!ready_at(&r, first_hot));
    assert!(reasons_at(&r, first_hot).contains(&Reason::TooHotNow));
    // After the car cools: below 45 °C for 30 minutes, a re-test, ready.
    let cooled = r
        .temperatures
        .iter()
        .find(|(t, c)| *t > 2 * DAY_S + 17 * HOUR && *c < 45.0)
        .unwrap()
        .0;
    assert!(!ready_at(&r, cooled + 20 * MIN));
    let retest = r.tests.iter().find(|t| t.at_s >= cooled + 30 * MIN).unwrap();
    assert!(retest.at_s <= cooled + 36 * MIN, "re-tested at once once cooled");
    // (The decision at a wake is made before that wake's test runs.)
    assert!(ready_at(&r, retest.at_s + 10 * MIN));
    assert_eq!(r.standby.heat().lifetime().too_hot_events, 1);
    println!(
        "hot car afternoon (SIMULATED): device above 60 °C from {:.2} h, peak {:.1} °C, below 45 °C at {:.2} h, \
         re-tested and ready at {:.2} h (day 2 hours)",
        f64::from(first_hot - 2 * DAY_S) / 3600.0,
        r.temperatures.iter().map(|(_, c)| *c).fold(0.0, f64::max),
        f64::from(cooled - 2 * DAY_S) / 3600.0,
        f64::from(retest.at_s - 2 * DAY_S) / 3600.0
    );
}

fn dashboard(t: u32) -> f64 {
    if (DAY_S + 11 * HOUR..DAY_S + 15 * HOUR).contains(&t) {
        78.0
    } else {
        22.0
    }
}

#[test]
fn over_70_c_latches_service_needed_which_cooling_and_tests_do_not_clear() {
    let s = StandbyScenario {
        weather: Some(dashboard),
        ..StandbyScenario::quiet(5)
    };
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    let end = 5 * DAY_S - 1;
    let reasons = reasons_at(&r, end);
    assert!(reasons.contains(&Reason::ServiceNeededOverheat), "{reasons:?}");
    assert!(reasons.contains(&Reason::PadsReplaceAfterOverheat));
    assert!(!reasons.contains(&Reason::WasTooHot), "it cooled and was re-tested");
    assert!(!ready_at(&r, end));
    assert!(r.standby.heat().lifetime().max.unwrap() > aed_core::DeciC::from_c(70));
}

fn warm_9_hours(t: u32) -> f64 {
    if (DAY_S..DAY_S + 9 * HOUR).contains(&t) {
        56.0
    } else {
        24.0
    }
}

#[test]
fn an_8_hour_warm_spell_makes_the_device_not_ready_until_cooled_and_retested() {
    let s = StandbyScenario {
        weather: Some(warm_9_hours),
        ..StandbyScenario::quiet(3)
    };
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    assert!(ready_at(&r, DAY_S + 6 * HOUR), "warm but ready");
    let at = DAY_S + 9 * HOUR - MIN;
    assert!(reasons_at(&r, at).contains(&Reason::WarmSpellTooLong));
    assert!(ready_at(&r, 2 * DAY_S + 12 * HOUR));
}

#[test]
fn a_log_gap_from_the_battery_taken_out_is_recorded_not_ready_until_a_self_test_passes_and_nothing_durable_is_lost() {
    let mut s = StandbyScenario {
        weather: Some(car_afternoon),
        ..StandbyScenario::quiet(5)
    };
    s.changes = vec![(3 * DAY_S + 6 * HOUR, Change::PowerLoss)];
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    let t = 3 * DAY_S + 6 * HOUR;
    let d = r.decision_at(t).unwrap();
    assert!(d.heat_gap_found);
    assert_eq!(r.power_losses, 1);
    let life = r.standby.heat().lifetime();
    assert_eq!(life.too_hot_events, 1, "the car afternoon before the power loss is kept");
    assert!(life.gaps_unknown_length >= 1);
    let test_after = r.tests.iter().find(|x| x.at_s >= t).unwrap();
    assert!(!ready_at(&r, t) || test_after.at_s == t);
    assert!(ready_at(&r, t + HOUR));
}

#[test]
fn a_dead_capacitor_temperature_sensor_puts_the_tests_off_and_is_not_ready() {
    let mut s = StandbyScenario::quiet(3);
    s.changes = vec![
        (DAY_S, Change::TempSensor(TempPoint::Capacitor, TempSensorFault::Missing)),
        (DAY_S, Change::TempSensor(TempPoint::Supervisor, TempSensorFault::Missing)),
    ];
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    assert!(reasons_at(&r, 2 * DAY_S).contains(&Reason::TempSensorFault));
    assert!(!r.tests.iter().any(|t| t.at_s > DAY_S), "no test charges a capacitor of unknown temperature");
}

fn slow_warming(t: u32) -> f64 {
    20.0 + 15.0 * (f64::from(t % DAY_S) / f64::from(DAY_S) * std::f64::consts::TAU).sin().max(0.0)
}

#[test]
fn a_stuck_main_or_supervisor_temperature_sensor_is_found_and_shown_not_ready() {
    for point in [TempPoint::Board, TempPoint::Supervisor] {
        let mut s = StandbyScenario {
            weather: Some(slow_warming),
            ..StandbyScenario::quiet(2)
        };
        s.changes = vec![(HOUR, Change::TempSensor(point, TempSensorFault::StuckAt(20.0)))];
        let r = run_standby(&s).unwrap();
        no_violations(&r);
        let flagged = r
            .decisions
            .iter()
            .any(|(_, d)| d.readiness.reasons.contains(Reason::TempSensorFault));
        assert!(flagged, "{point:?} stuck was not found");
    }
}

fn hot_afternoons(t: u32) -> f64 {
    let h = (t % DAY_S) / HOUR;
    if (12..18).contains(&h) {
        54.0
    } else {
        26.0
    }
}

#[test]
fn pad_heat_allowance_reached_shows_replace_pads_and_new_pads_clear_it() {
    let mut s = StandbyScenario {
        weather: Some(hot_afternoons),
        config: Config {
            pad_heat_allowance_s: DAY_S,
            ..Config::default()
        },
        ..StandbyScenario::quiet(9)
    };
    s.changes = vec![(8 * DAY_S, Change::PadSet(2))];
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    let worn = r
        .decisions
        .iter()
        .find(|(_, d)| d.readiness.reasons.contains(Reason::PadsHeatWorn))
        .map(|(t, _)| *t)
        .unwrap();
    // Six hours a day above 50 °C (less the lag): about five days.
    assert!(worn > 4 * DAY_S && worn < 8 * DAY_S, "{worn}");
    assert!(!reasons_at(&r, 8 * DAY_S + HOUR).contains(&Reason::PadsHeatWorn));
    assert_eq!(r.standby.heat().pad_set(), Some(2));
}

#[test]
fn capacitor_wear_thresholds_service_soon_then_due_and_the_weekly_test_continues() {
    let s = StandbyScenario {
        config: Config {
            capacitor_life_charges: 5,
            capacitor_service_soon_percent: 80,
            ..Config::default()
        },
        ..StandbyScenario::quiet(50)
    };
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    // Weekly tests at days 0, 7, 14, 21, 28: the 4th makes 80 %, the 5th 100 %.
    let soon = reasons_at(&r, 22 * DAY_S);
    assert!(soon.contains(&Reason::CapacitorServiceSoon) && ready_at(&r, 22 * DAY_S));
    let due = reasons_at(&r, 29 * DAY_S);
    assert!(due.contains(&Reason::CapacitorServiceDue) && !ready_at(&r, 29 * DAY_S));
    assert!(r.standby.wear().test_charges >= 7, "the weekly test continues past the life");
}

fn cold_then_hot(t: u32) -> f64 {
    match t / DAY_S {
        1 => -12.0,
        2 => 52.0,
        _ => 20.0,
    }
}

#[test]
fn dock_charging_is_inhibited_when_cold_and_when_hot() {
    let s = StandbyScenario {
        weather: Some(cold_then_hot),
        ..StandbyScenario::quiet(4)
    };
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    let cold = r.decision_at(DAY_S + 12 * HOUR).unwrap();
    assert!(!cold.battery_charging_allowed);
    let hot = r.decision_at(2 * DAY_S + 12 * HOUR).unwrap();
    assert!(!hot.battery_charging_allowed);
    assert!(r.decision_at(3 * DAY_S + 12 * HOUR).unwrap().battery_charging_allowed);
}

// ----- months of mixed weather ------------------------------------------------------------

/// A deterministic mix: seasons, day and night, and on some days a car
/// afternoon (most days ordinary, some warm, a few very hot, one a
/// dashboard).
fn mixed_weather(t: u32) -> f64 {
    let day = t / DAY_S;
    let hour = f64::from(t % DAY_S) / 3_600.0;
    let season = 15.0 - 15.0 * (f64::from(day) / 365.0 * std::f64::consts::TAU).cos();
    let daily = 6.0 * ((hour - 9.0) / 24.0 * std::f64::consts::TAU).sin();
    let base = season + daily;
    let hash = day.wrapping_mul(2_654_435_761) >> 24;
    let in_car = (13.0..17.0).contains(&hour);
    match hash % 23 {
        0..=2 if in_car => base + 25.0,
        3 if in_car => base + 40.0,
        _ if day == 151 && in_car => 85.0,
        _ => base,
    }
}

#[test]
fn months_of_mixed_weather_keep_every_heat_invariant() {
    let days = 240;
    let mut s = StandbyScenario {
        weather: Some(mixed_weather),
        ..StandbyScenario::quiet(days)
    };
    s.changes = vec![
        (40 * DAY_S + 3 * HOUR, Change::PowerLoss),
        (60 * DAY_S, Change::Docked(false)),
        (65 * DAY_S, Change::Docked(true)),
        (100 * DAY_S + 9 * HOUR, Change::InUseUntil(100 * DAY_S + 10 * HOUR)),
        (152 * DAY_S, Change::PowerLoss),
        (200 * DAY_S, Change::TempOffset(TempPoint::Pads, 1.5)),
    ];
    let r = run_standby(&s).unwrap();
    no_violations(&r);
    let life = r.standby.heat().lifetime();
    let not_ready = r.decisions.iter().filter(|(_, d)| !d.readiness.is_ready()).count();
    let held = r
        .decisions
        .iter()
        .filter(|(_, d)| aed_core::Standby::<2160>::deferred_for_heat(d))
        .count();
    let no_dock = r.decisions.iter().filter(|(_, d)| !d.battery_charging_allowed).count();
    println!(
        "{days} days of mixed weather (SIMULATED): {} wakes, {} tests, hottest {:?} dC, coolest {:?} dC, \
         {} too-hot episodes, {} service latch(es), {:.1} h above 60 °C, {} gaps, {} wakes not ready, \
         {} wakes with a test put off for heat, {} wakes with dock charging stopped, pad heat {:.1} h, \
         log {} entries ({} overwritten)",
        r.decisions.len(),
        r.tests.len(),
        life.max,
        life.min,
        life.too_hot_events,
        life.service_events,
        f64::from(life.above_too_hot_s) / 3_600.0,
        life.gaps,
        not_ready,
        held,
        no_dock,
        f64::from(r.standby.heat().pad_hot_s()) / 3_600.0,
        r.standby.heat().log().len(),
        r.standby.heat().log().overwritten()
    );
    assert!(life.too_hot_events >= 3, "the weather must exercise the limits");
    assert!(life.service_events >= 1);
    assert!(r.standby.heat().service_needed());
    assert_eq!(r.power_losses, 2);
    assert!(r.standby.heat().log().overwritten() > 0, "the record wrapped");
    assert_ne!(r.standby.heat().band(), HeatBand::TooHot);
}
