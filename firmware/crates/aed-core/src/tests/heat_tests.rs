// SPDX-License-Identifier: GPL-3.0-or-later
//! Heat (D-41, D-48, D-52), capacitor wear (D-53), the one readiness result
//! and the record download (D-09), tested as pure logic. Temperatures are
//! scripted; no thermal model here (that is in aed-sim).

use crate::export::{crc32_update, write_record};
use crate::hal::BatteryStatus;
use crate::heat::{
    assess, battery_charge_check, capacitor_charge_check, flags, ChargeContext, DeciC, HeatBand,
    HeatInhibit, SensorFault, SensorSet, ServiceCheck, ServiceRefusal, TempPoint, TEMP_POINTS,
};
use crate::readiness::{DeviceReadiness, Reason, ReasonSet};
use crate::schedule::{Deferral, PeriodicOutcome, PeriodicTest};
use crate::selftest::SelfTestPass;
use crate::standby::{
    DurableRecord, DurableStore, Standby, StandbyDecision, StandbyInputs, StoreError, WearStatus,
};
use crate::{Config, ConfigError};
use std::string::String;
use std::vec::Vec;

const MIN: u32 = 60;
const HOUR: u32 = 3_600;
const DAY: u32 = 86_400;

type Sb = Standby<64>;

fn c(t: i16) -> DeciC {
    DeciC::from_c(t)
}

fn all(t: i16) -> SensorSet {
    SensorSet::uniform(c(t))
}

fn inputs(now_s: u32, temps: SensorSet) -> StandbyInputs {
    StandbyInputs {
        now_s,
        clock_restarted: false,
        in_use: false,
        on_charge: true,
        battery: BatteryStatus::Good,
        temperatures: temps,
        pad_set_id: Some(1),
    }
}

fn pass() -> PeriodicOutcome {
    PeriodicOutcome {
        result: Ok(SelfTestPass { battery_low: false }),
        capacitor_confirmed_empty: true,
        charged: true,
    }
}

/// Wake as asked from `from` until `to` with the readings `temps(now)`;
/// every test asked for passes. Returns the decisions and the time reached.
fn run(s: &mut Sb, from: u32, to: u32, temps: impl Fn(u32) -> SensorSet) -> (Vec<(u32, StandbyDecision)>, u32) {
    let mut now = from;
    let mut out = Vec::new();
    while now < to {
        let d = s.on_wake(&inputs(now, temps(now)));
        let mut next = d.next_wake_in_s;
        if let Some(t) = d.tests.run {
            s.record(now, t, &pass());
            next = s.next_wake_in_s(now);
        }
        out.push((now, d));
        now += next;
    }
    (out, now)
}

fn ready_device() -> (Sb, u32) {
    let mut s = Sb::new(Config::default()).unwrap();
    let (_, now) = run(&mut s, 0, DAY, |_| all(25));
    assert!(s.readiness(now).is_ready(), "{:?}", s.readiness(now));
    (s, now)
}

fn reasons(s: &Sb, now: u32) -> ReasonSet {
    s.readiness(now).reasons
}

// ----- trusting the sensors ---------------------------------------------------

#[test]
fn a_missing_out_of_range_or_disagreeing_sensor_is_not_trusted() {
    let cfg = Config::default();
    let mut set = all(30);
    set.set(TempPoint::Cell1, None);
    set.set(TempPoint::Pads, Some(c(120)));
    set.set(TempPoint::Board, Some(c(55)));
    let a = assess(&cfg, &set, &[false; TEMP_POINTS]);
    assert_eq!(a.fault_at(TempPoint::Cell1), Some(SensorFault::Missing));
    assert_eq!(a.fault_at(TempPoint::Pads), Some(SensorFault::OutOfRange));
    assert_eq!(a.fault_at(TempPoint::Board), Some(SensorFault::Disagrees));
    assert_eq!(a.hottest(), Some(c(30)));
    assert!(a.any_fault());
}

#[test]
fn two_sensors_that_disagree_are_both_distrusted() {
    let cfg = Config::default();
    let mut set = SensorSet::default();
    set.set(TempPoint::Cell1, Some(c(20)));
    set.set(TempPoint::Cell2, Some(c(40)));
    let a = assess(&cfg, &set, &[false; TEMP_POINTS]);
    assert_eq!(a.fault_at(TempPoint::Cell1), Some(SensorFault::Disagrees));
    assert_eq!(a.fault_at(TempPoint::Cell2), Some(SensorFault::Disagrees));
    assert_eq!(a.hottest(), None);
}

#[test]
fn a_sensor_unchanged_for_an_hour_while_the_others_moved_is_stuck_and_not_ready() {
    let (mut s, now) = ready_device();
    // The others warm from 25 to 35 °C over two hours; the board sensor
    // stays at exactly 25.0.
    let (_, end) = run(&mut s, now, now + 2 * HOUR, |t| {
        let warm = i16::try_from((t - now) / 720).unwrap().min(10);
        let mut set = all(25 + warm);
        set.set(TempPoint::Board, Some(c(25)));
        set
    });
    assert!(reasons(&s, end).contains(Reason::TempSensorFault));
    assert!(!s.readiness(end).is_ready());
}

#[test]
fn outside_a_rescue_an_unknown_temperature_counts_as_too_hot_for_charging() {
    let cfg = Config::default();
    let mut set = all(25);
    set.set(TempPoint::Capacitor, None);
    set.set(TempPoint::Supervisor, None);
    let a = assess(&cfg, &set, &[false; TEMP_POINTS]);
    assert_eq!(
        capacitor_charge_check(&cfg, &a, ChargeContext::Standby, false),
        Err(HeatInhibit::Unknown)
    );
    let mut set = all(25);
    set.set(TempPoint::Cell2, None);
    let a = assess(&cfg, &set, &[false; TEMP_POINTS]);
    assert_eq!(battery_charge_check(&cfg, &a, true), Err(HeatInhibit::Unknown));
}

#[test]
fn in_a_rescue_a_failed_sensor_takes_the_hottest_trusted_reading_plus_the_margin() {
    let cfg = Config::default();
    let mut set = all(50);
    set.set(TempPoint::Capacitor, None);
    set.set(TempPoint::Supervisor, None);
    let a = assess(&cfg, &set, &[false; TEMP_POINTS]);
    // 50 + 5 = 55: allowed (the limit is 60).
    assert_eq!(capacitor_charge_check(&cfg, &a, ChargeContext::Rescue, false), Ok(()));
    let mut set = all(57);
    set.set(TempPoint::Capacitor, None);
    set.set(TempPoint::Supervisor, None);
    let a = assess(&cfg, &set, &[false; TEMP_POINTS]);
    // 57 + 5 = 62: held back.
    assert_eq!(
        capacitor_charge_check(&cfg, &a, ChargeContext::Rescue, false),
        Err(HeatInhibit::CapacitorHot)
    );
}

#[test]
fn in_a_rescue_no_trusted_reading_at_all_does_not_block_charging() {
    let cfg = Config::default();
    let a = assess(&cfg, &SensorSet::default(), &[false; TEMP_POINTS]);
    assert_eq!(capacitor_charge_check(&cfg, &a, ChargeContext::Rescue, false), Ok(()));
    assert_eq!(
        capacitor_charge_check(&cfg, &a, ChargeContext::Standby, false),
        Err(HeatInhibit::Unknown)
    );
}

#[test]
fn capacitor_charging_only_at_or_below_60_c_capacitor_and_cells_with_hysteresis() {
    let cfg = Config::default();
    let check = |set: SensorSet, was: bool| {
        capacitor_charge_check(&cfg, &assess(&cfg, &set, &[false; TEMP_POINTS]), ChargeContext::Rescue, was)
    };
    assert_eq!(check(all(60), false), Ok(()));
    let mut hot_cap = all(55);
    hot_cap.set(TempPoint::Capacitor, Some(DeciC(601)));
    assert_eq!(check(hot_cap, false), Err(HeatInhibit::CapacitorHot));
    let mut hot_cell = all(55);
    hot_cell.set(TempPoint::Cell2, Some(DeciC(601)));
    assert_eq!(check(hot_cell, false), Err(HeatInhibit::CellsHot));
    // Once held back: 59.5 °C is still held, 59.0 °C is allowed.
    assert_eq!(check(SensorSet::uniform(DeciC(595)), true), Err(HeatInhibit::CapacitorHot));
    assert_eq!(check(SensorSet::uniform(DeciC(590)), true), Ok(()));
}

#[test]
fn capacitor_charging_has_no_cold_limit() {
    let cfg = Config::default();
    let a = assess(&cfg, &all(-20), &[false; TEMP_POINTS]);
    assert_eq!(capacitor_charge_check(&cfg, &a, ChargeContext::Standby, false), Ok(()));
}

#[test]
fn battery_charging_only_between_0_and_45_c_and_never_when_unknown() {
    let cfg = Config::default();
    let check = |t: i16, was: bool| {
        battery_charge_check(&cfg, &assess(&cfg, &all(t), &[false; TEMP_POINTS]), was)
    };
    assert_eq!(check(0, true), Ok(()));
    assert_eq!(check(45, true), Ok(()));
    assert_eq!(check(-1, true), Err(HeatInhibit::CellsCold));
    assert_eq!(check(46, true), Err(HeatInhibit::CellsHot));
    // Hysteresis: once stopped, 44.5 and 0.5 °C still wait; 44 and 1 °C resume.
    let half = |dc: i16| battery_charge_check(&cfg, &assess(&cfg, &SensorSet::uniform(DeciC(dc)), &[false; TEMP_POINTS]), false);
    assert_eq!(half(445), Err(HeatInhibit::CellsHot));
    assert_eq!(half(440), Ok(()));
    assert_eq!(half(5), Err(HeatInhibit::CellsCold));
    assert_eq!(half(10), Ok(()));
}

#[test]
fn dock_charging_is_inhibited_when_cold_and_when_hot() {
    let (mut s, now) = ready_device();
    let d = s.on_wake(&inputs(now, all(-5)));
    assert!(!d.battery_charging_allowed);
    assert_eq!(d.battery_inhibit, Some(HeatInhibit::CellsCold));
    let d = s.on_wake(&inputs(now + 300, all(48)));
    assert!(!d.battery_charging_allowed);
    assert_eq!(d.battery_inhibit, Some(HeatInhibit::CellsHot));
    let d = s.on_wake(&inputs(now + 600, all(25)));
    assert!(d.battery_charging_allowed);
}

// ----- heat states ----------------------------------------------------------------

#[test]
fn heat_bands_follow_the_hottest_reading_with_hysteresis() {
    let (mut s, mut now) = ready_device();
    let mut band_at = |s: &mut Sb, t_dc: i16| {
        now += MIN;
        s.on_wake(&inputs(now, SensorSet::uniform(DeciC(t_dc))));
        s.heat().band()
    };
    assert_eq!(band_at(&mut s, 400), HeatBand::Normal);
    assert_eq!(band_at(&mut s, 401), HeatBand::Warm);
    assert_eq!(band_at(&mut s, 390), HeatBand::Warm, "inside the 2 °C hysteresis");
    assert_eq!(band_at(&mut s, 380), HeatBand::Normal);
    assert_eq!(band_at(&mut s, 601), HeatBand::TooHot);
    assert_eq!(band_at(&mut s, 590), HeatBand::TooHot, "inside the hysteresis");
    assert_eq!(band_at(&mut s, 580), HeatBand::Warm);
}

#[test]
fn warm_up_to_60_c_is_still_ready() {
    let (mut s, now) = ready_device();
    let (_, end) = run(&mut s, now, now + 2 * HOUR, |_| all(58));
    assert!(s.readiness(end).is_ready(), "{:?}", s.readiness(end));
    assert_eq!(s.heat().band(), HeatBand::Warm);
}

#[test]
fn above_60_c_is_not_ready_until_below_45_c_for_30_minutes_and_a_self_test_passes() {
    let (mut s, now) = ready_device();
    let (_, t1) = run(&mut s, now, now + HOUR, |_| all(63));
    assert!(reasons(&s, t1).contains(Reason::TooHotNow));
    assert!(reasons(&s, t1).contains(Reason::WasTooHot));
    // Cools to 50 °C: no longer too hot now, still "was too hot".
    let (_, t2) = run(&mut s, t1, t1 + HOUR, |_| all(50));
    let r = reasons(&s, t2);
    assert!(!r.contains(Reason::TooHotNow) && r.contains(Reason::WasTooHot));
    // Below 45 °C: 29 minutes is not enough...
    let (log, t3) = run(&mut s, t2, t2 + 29 * MIN, |_| all(30));
    assert!(log.iter().all(|(_, d)| d.tests.run.is_none()), "no re-test before 30 min");
    assert!(reasons(&s, t3).contains(Reason::WasTooHot));
    // ... after 30 minutes the daily check is made due at once, it passes,
    // and the device is ready.
    let (log, t4) = run(&mut s, t3, t3 + 10 * MIN, |_| all(30));
    let retest = log.iter().find(|(_, d)| d.tests.run.is_some()).expect("a re-test runs");
    assert!(retest.0 >= t2 + 30 * MIN);
    assert!(s.readiness(t4).is_ready(), "{:?}", s.readiness(t4));
}

#[test]
fn a_test_passed_before_the_device_cooled_does_not_clear_was_too_hot() {
    let (mut s, now) = ready_device();
    run(&mut s, now, now + 30 * MIN, |_| all(63));
    let t = now + HOUR;
    s.on_wake(&inputs(t, all(50)));
    s.record(t, PeriodicTest::Daily, &pass());
    assert!(reasons(&s, t).contains(Reason::WasTooHot));
}

#[test]
fn too_hot_puts_both_tests_off_and_the_full_test_needs_cells_0_to_45_and_capacitor_at_most_50() {
    let (mut s, now) = ready_device();
    // Make both due.
    let t = now + 8 * DAY;
    let d = s.on_wake(&inputs(t, all(62)));
    assert_eq!(d.tests.run, None);
    assert_eq!(d.tests.deferred.map(|x| x.1), Some(Deferral::Temperature));
    assert_eq!(d.capacitor_inhibit, Some(HeatInhibit::CapacitorHot));
    // At 48 °C the daily check may charge to 400 V, the full test may not.
    let d = s.on_wake(&inputs(t + MIN, all(48)));
    assert_eq!(d.tests.run, Some(PeriodicTest::Daily));
    assert_eq!(d.tests.deferred, Some((PeriodicTest::Full, Deferral::Temperature)));
    // Cells at 40, capacitor at 52: the full test still waits (R-H4: 50 °C).
    let mut set = all(40);
    set.set(TempPoint::Capacitor, Some(c(52)));
    set.set(TempPoint::Supervisor, Some(c(52)));
    let mut s2 = Sb::new(Config::default()).unwrap();
    let d = s2.on_wake(&inputs(0, set));
    assert_eq!(d.tests.deferred, Some((PeriodicTest::Full, Deferral::Temperature)));
    // Cold (−5 °C): the full test waits, the daily check runs (no cold limit).
    let mut s3 = Sb::new(Config::default()).unwrap();
    let d = s3.on_wake(&inputs(0, all(-5)));
    assert_eq!(d.tests.run, Some(PeriodicTest::Daily));
    assert_eq!(d.tests.deferred, Some((PeriodicTest::Full, Deferral::Temperature)));
}

#[test]
fn more_than_8_hours_above_50_c_is_not_ready_until_cooled_and_retested() {
    let (mut s, now) = ready_device();
    let (_, t1) = run(&mut s, now, now + 7 * HOUR, |_| all(55));
    assert!(s.readiness(t1).is_ready(), "7 h at 55 °C is still ready");
    let (_, t2) = run(&mut s, t1, t1 + 90 * MIN, |_| all(55));
    assert!(reasons(&s, t2).contains(Reason::WarmSpellTooLong));
    assert!(!s.readiness(t2).is_ready());
    let (_, t3) = run(&mut s, t2, t2 + HOUR, |_| all(30));
    assert!(s.readiness(t3).is_ready(), "{:?}", s.readiness(t3));
}

#[test]
fn a_warm_spell_broken_by_cooling_starts_again() {
    let (mut s, now) = ready_device();
    let (_, t1) = run(&mut s, now, now + 6 * HOUR, |_| all(55));
    let (_, t2) = run(&mut s, t1, t1 + HOUR, |_| all(40));
    let (_, t3) = run(&mut s, t2, t2 + 6 * HOUR, |_| all(55));
    assert!(s.readiness(t3).is_ready(), "{:?}", s.readiness(t3));
}

#[test]
fn a_reading_above_70_c_latches_service_needed_until_a_service_action() {
    let (mut s, now) = ready_device();
    let (_, t1) = run(&mut s, now, now + 20 * MIN, |_| all(72));
    let r = reasons(&s, t1);
    assert!(r.contains(Reason::ServiceNeededOverheat) && r.contains(Reason::PadsReplaceAfterOverheat));
    // Cooling and passing tests does not clear it.
    let (_, t2) = run(&mut s, t1, t1 + 2 * DAY, |_| all(25));
    let r = reasons(&s, t2);
    assert!(r.contains(Reason::ServiceNeededOverheat), "{r:?}");
    assert!(!r.contains(Reason::WasTooHot), "cooled and re-tested");
    // Service: full test and battery health passed.
    let ok = ServiceCheck {
        full_self_test_passed: true,
        battery_health_passed: true,
    };
    assert_eq!(s.service_cleared(t2, ok), Ok(()));
    let r = reasons(&s, t2);
    assert!(!r.contains(Reason::ServiceNeededOverheat));
    // New pads are still recommended until a new pad set is fitted.
    assert!(r.contains(Reason::PadsReplaceAfterOverheat));
    let mut i = inputs(t2 + MIN, all(25));
    i.pad_set_id = Some(2);
    s.on_wake(&i);
    assert!(!reasons(&s, t2 + MIN).contains(Reason::PadsReplaceAfterOverheat));
}

#[test]
fn service_action_is_refused_without_full_test_battery_health_or_cooling() {
    let (mut s, now) = ready_device();
    run(&mut s, now, now + 20 * MIN, |_| all(72));
    let t = now + 20 * MIN;
    let check = |full, battery| ServiceCheck {
        full_self_test_passed: full,
        battery_health_passed: battery,
    };
    assert_eq!(s.service_cleared(t, check(false, true)), Err(ServiceRefusal::FullTestNotPassed));
    assert_eq!(s.service_cleared(t, check(true, false)), Err(ServiceRefusal::BatteryHealthNotPassed));
    assert_eq!(s.service_cleared(t, check(true, true)), Err(ServiceRefusal::NotCooled));
    assert!(s.heat().service_needed());
}

// ----- the record ---------------------------------------------------------------

#[test]
fn heat_wakes_every_5_minutes_up_to_40_c_and_every_minute_above() {
    let (mut s, now) = ready_device();
    let d = s.on_wake(&inputs(now, all(30)));
    assert_eq!(d.next_wake_in_s, 300);
    let d = s.on_wake(&inputs(now + 300, all(45)));
    assert_eq!(d.next_wake_in_s, 60);
}

#[test]
fn a_gap_in_the_heat_record_is_not_ready_until_a_self_test_passes_and_is_recorded() {
    let (mut s, now) = ready_device();
    let t = now + 3 * HOUR; // asked for 5 minutes, woken after 3 hours
    let d = s.on_wake(&inputs(t, all(25)));
    assert!(d.heat_gap_found);
    assert!(!d.readiness.is_ready());
    assert!(d.readiness.reasons.contains(Reason::HeatRecordGap));
    assert_eq!(d.tests.run, Some(PeriodicTest::Daily), "a re-test is made due at once");
    s.record(t, PeriodicTest::Daily, &pass());
    assert!(s.readiness(t).is_ready());
    assert_eq!(s.heat().lifetime().gaps, 1);
    assert!(s.heat().lifetime().gap_s >= 3 * HOUR);
    let flagged = s
        .heat()
        .log()
        .iter()
        .chain(s.heat().current_entry().iter())
        .any(|e| e.flags & flags::GAP_BEFORE != 0);
    assert!(flagged, "the gap is in the record");
}

#[test]
fn a_timer_restart_is_a_gap_of_unknown_length() {
    let (mut s, _) = ready_device();
    let mut i = inputs(5, all(25));
    i.clock_restarted = true;
    let d = s.on_wake(&i);
    assert!(d.heat_gap_found);
    assert_eq!(s.heat().lifetime().gaps_unknown_length, 1);
    assert!(!d.readiness.is_ready());
}

#[test]
fn heat_record_keeps_the_peak_when_it_overwrites_old_entries() {
    let (mut s, now) = ready_device();
    // A 66 °C hour, then 200 hours of 25 °C: far more than 64 entries.
    let (_, t1) = run(&mut s, now, now + HOUR, |_| all(66));
    let (_, _) = run(&mut s, t1, t1 + 200 * HOUR, |_| all(25));
    let log = s.heat().log();
    assert_eq!(log.len(), 64);
    assert!(log.overwritten() > 0);
    assert!(log.iter().all(|e| e.max.is_some_and(|m| m < c(30))), "the hot hour was overwritten");
    assert_eq!(s.heat().lifetime().max, Some(c(66)), "but its peak is kept");
    assert!(s.heat().lifetime().above_too_hot_s >= 55 * MIN);
    assert_eq!(s.heat().lifetime().too_hot_events, 1);
}

// ----- pads ----------------------------------------------------------------------

#[test]
fn pad_heat_total_counts_time_above_50_c_and_a_new_pad_set_resets_it() {
    let (mut s, now) = ready_device();
    let (_, t1) = run(&mut s, now, now + 2 * HOUR, |_| all(52));
    let hot = s.heat().pad_hot_s();
    // An interval counts if either end of it was hot (conservative): up to
    // one 5-minute interval more than the spell itself.
    assert!((2 * HOUR - 2 * MIN..=2 * HOUR + 5 * MIN).contains(&hot), "{hot}");
    let (_, t2) = run(&mut s, t1, t1 + 5 * HOUR, |_| all(45));
    assert!(s.heat().pad_hot_s() <= hot + 5 * MIN, "45 °C does not count");
    let mut i = inputs(t2, all(25));
    i.pad_set_id = Some(7);
    s.on_wake(&i);
    assert_eq!(s.heat().pad_hot_s(), 0);
    assert_eq!(s.heat().pad_set(), Some(7));
    // The same pads taken out and put back are not new pads.
    let mut i = inputs(t2 + 300, all(55));
    i.pad_set_id = None;
    s.on_wake(&i);
    let mut i = inputs(t2 + 360, all(55));
    i.pad_set_id = Some(7);
    s.on_wake(&i);
    assert!(s.heat().pad_hot_s() > 0);
}

#[test]
fn pads_past_their_heat_allowance_show_replace_pads() {
    let cfg = Config {
        pad_heat_allowance_s: 4 * HOUR,
        ..Config::default()
    };
    let mut s = Sb::new(cfg).unwrap();
    let (_, t0) = run(&mut s, 0, DAY, |_| all(25));
    // Short spells, each well under 8 hours, add up.
    let mut t = t0;
    for _ in 0..3 {
        let (_, a) = run(&mut s, t, t + 2 * HOUR, |_| all(53));
        let (_, b) = run(&mut s, a, a + 4 * HOUR, |_| all(30));
        t = b;
    }
    assert!(s.pads_heat_worn());
    assert!(reasons(&s, t).contains(Reason::PadsHeatWorn));
    assert!(!s.readiness(t).is_ready());
}

#[test]
fn a_sensor_fault_makes_the_device_not_ready() {
    let (mut s, now) = ready_device();
    let mut set = all(25);
    set.set(TempPoint::Supervisor, None);
    let d = s.on_wake(&inputs(now, set));
    assert!(d.readiness.reasons.contains(Reason::TempSensorFault));
    assert!(!d.readiness.is_ready());
}

// ----- survives power loss -------------------------------------------------------

#[derive(Default)]
struct Fram {
    saved: Option<DurableRecord<64>>,
    corrupt: bool,
}

impl DurableStore<64> for Fram {
    fn save(&mut self, record: &DurableRecord<64>) -> Result<(), StoreError> {
        self.saved = Some(record.clone());
        Ok(())
    }
    fn load(&mut self, into: &mut DurableRecord<64>) -> Result<(), StoreError> {
        if self.corrupt {
            return Err(StoreError::Corrupt);
        }
        let saved = self.saved.as_ref().ok_or(StoreError::Empty)?;
        into.clone_from(saved);
        Ok(())
    }
}

#[test]
fn durable_record_survives_power_loss_and_a_lost_record_is_not_ready_until_service() {
    let (mut s, now) = ready_device();
    run(&mut s, now, now + HOUR, |_| all(72));
    s.add_rescue_charges(12);
    let mut fram = Fram::default();
    s.save(&mut fram).unwrap();
    let before = s.durable().clone();
    let (mut r, err) = Sb::restore(Config::default(), &mut fram).unwrap();
    assert_eq!(err, None);
    assert_eq!(r.heat().lifetime(), before.heat.lifetime());
    assert_eq!(r.wear(), before.wear);
    assert!(r.heat().service_needed());
    assert_eq!(r.heat().pad_hot_s(), before.heat.pad_hot_s());
    // The power loss itself is a gap.
    let d = r.on_wake(&inputs(10, all(25)));
    assert!(d.heat_gap_found);

    fram.corrupt = true;
    let (lost, err) = Sb::restore(Config::default(), &mut fram).unwrap();
    assert_eq!(err, Some(StoreError::Corrupt));
    assert!(reasons(&lost, 0).contains(Reason::RecordLost));
}

// ----- capacitor wear -------------------------------------------------------------

#[test]
fn capacitor_wear_service_soon_at_90_percent_and_due_at_the_life_but_only_as_a_readiness_reason() {
    let (mut s, now) = ready_device();
    let tests = s.wear().test_charges;
    s.add_rescue_charges(2_249 - tests);
    assert_eq!(s.wear().status(s.config()), WearStatus::Ok);
    s.add_rescue_charges(1);
    assert_eq!(s.wear().total(), 2_250);
    assert_eq!(s.wear().status(s.config()), WearStatus::ServiceSoon);
    let r = s.readiness(now);
    assert!(r.reasons.contains(Reason::CapacitorServiceSoon));
    assert!(r.is_ready(), "service soon is advisory");
    s.add_rescue_charges(250);
    let r = s.readiness(now);
    assert!(r.reasons.contains(Reason::CapacitorServiceDue));
    assert!(!r.is_ready());
}

#[test]
fn the_weekly_full_energy_test_continues_after_the_capacitor_life_is_reached() {
    let (mut s, now) = ready_device();
    s.add_rescue_charges(3_000);
    let before = s.wear().test_charges;
    run(&mut s, now, now + 15 * DAY, |_| all(25));
    assert!(s.wear().test_charges >= before + 2);
}

#[test]
fn full_energy_test_charges_count_as_capacitor_wear_and_persist() {
    let mut s = Sb::new(Config::default()).unwrap();
    run(&mut s, 0, 22 * DAY, |_| all(25));
    assert_eq!(s.wear().test_charges, 4);
}

// ----- one readiness --------------------------------------------------------------

#[test]
fn readiness_is_never_ready_while_a_blocking_reason_applies() {
    for r in Reason::ALL {
        let mut set = ReasonSet::EMPTY;
        set.insert(r);
        set.insert(Reason::CapacitorServiceSoon);
        let ready = DeviceReadiness { reasons: set }.is_ready();
        assert_eq!(ready, r.is_advisory(), "{r:?}");
    }
    assert!(DeviceReadiness::default().is_ready());
}

#[test]
fn readiness_lists_every_reason_at_once() {
    let (mut s, now) = ready_device();
    s.add_rescue_charges(5_000);
    run(&mut s, now, now + 10 * MIN, |_| all(65));
    let r = reasons(&s, now + 10 * MIN);
    for want in [Reason::TooHotNow, Reason::WasTooHot, Reason::CapacitorServiceDue] {
        assert!(r.contains(want), "{want:?} missing from {r:?}");
    }
}

#[test]
fn heat_config_that_contradicts_itself_is_refused() {
    let d = Config::default();
    let bad = [
        Config { heat_too_hot_above: c(75), ..d },
        Config { heat_hysteresis: DeciC(0), ..d },
        Config { battery_charge_min: c(50), ..d },
        Config { precharge_max: c(65), ..d },
        Config { supervisor_temp_limit: c(55), ..d },
        Config { heat_cooled_below: c(65), ..d },
        Config { heat_wake_warm_s: 600, ..d },
    ];
    for cfg in bad {
        assert_eq!(cfg.validate(), Err(ConfigError::HeatThresholds), "{cfg:?}");
    }
    assert_eq!(
        Config { capacitor_service_soon_percent: 0, ..d }.validate(),
        Err(ConfigError::CapacitorLife)
    );
}

#[test]
fn default_heat_config_matches_the_owner_decisions() {
    let c0 = Config::default();
    assert_eq!(c0.validate(), Ok(()));
    // D-48.
    assert_eq!(c0.heat_normal_max, c(40));
    assert_eq!(c0.heat_too_hot_above, c(60));
    assert_eq!(c0.heat_service_above, c(70));
    assert_eq!(c0.heat_warm_spell_limit_s, 8 * HOUR);
    assert_eq!((c0.battery_charge_min, c0.battery_charge_max), (c(0), c(45)));
    // D-41.
    assert_eq!(c0.capacitor_charge_max, c(60));
    // D-52, D-53.
    assert_eq!(c0.pad_heat_allowance_s, 7 * DAY);
    assert_eq!(c0.capacitor_life_charges, 2_500);
}

// ----- the record download -------------------------------------------------------

#[test]
fn crc32_matches_the_standard_check_value() {
    assert_eq!(crc32_update(0, b"123456789"), 0xCBF4_3926);
    let split = crc32_update(crc32_update(0, b"1234"), b"56789");
    assert_eq!(split, 0xCBF4_3926);
}

#[test]
fn the_record_download_has_a_version_every_reason_and_a_checksum_that_matches() {
    let (mut s, now) = ready_device();
    run(&mut s, now, now + 10 * MIN, |_| all(65));
    let mut text = String::new();
    write_record::<_, 64, 4>(&mut text, &s, now + 10 * MIN, None).unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("OPEN-AED-RECORD"));
    assert_eq!(lines.next(), Some("version 1"));
    assert!(text.contains("ready 0\n"));
    assert!(text.contains("too-hot-now"));
    let at = text.find("crc32 ").unwrap();
    let body = text.get(..at).unwrap();
    let stated = u32::from_str_radix(text.get(at + 6..at + 14).unwrap(), 16).unwrap();
    assert_eq!(crc32_update(0, body.as_bytes()), stated);
    assert!(!text.to_lowercase().contains("bluetooth"));
}
