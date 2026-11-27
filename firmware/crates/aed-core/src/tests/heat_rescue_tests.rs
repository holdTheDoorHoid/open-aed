// SPDX-License-Identifier: GPL-3.0-or-later
//! Heat in a rescue (D-41, D-48, D-51) and the adult/child switch during an
//! analysis (D-34), on the mock board with both controllers. Temperatures
//! are scripted on the mock; nothing is measured.

use super::mock::{Bench, FakePlanner};
use crate::hal::PadCartridge;
use crate::heat::{DeciC, HeatInhibit, SensorSet, TempPoint};
use crate::prompts::Prompt;
use crate::rescue::{AbortReason, RescueState};
use crate::supervisor::TempIgnored;
use crate::{Config, Event};
use aed_types::{PatientMode, RhythmDecision, Volts};
use std::vec::Vec;

use RescueState as S;

const NON: RhythmDecision = RhythmDecision::NonShockable;

fn short_cpr() -> Config {
    Config {
        cpr_period_ms: 15_000,
        ..Config::default()
    }
}

fn count(b: &Bench, pred: impl Fn(&Event) -> bool) -> usize {
    b.rescue.events().iter().filter(|e| pred(&e.event)).count()
}

/// Run `ms`, failing if the charger is ever asked to run.
fn run_never_charging(b: &mut Bench, ms: u32) {
    let end = b.board.now + ms;
    while b.board.now < end {
        b.step();
        assert!(
            b.board.charger.is_none(),
            "charger on at {} ms in {:?}",
            b.board.now,
            b.state()
        );
    }
}

// ----- too hot at switch-on (D-51) -----------------------------------------------

#[test]
fn too_hot_at_switch_on_skips_the_self_test_charge_says_so_analyses_and_never_charges() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.set_all_temps(63);
    run_never_charging(&mut b, 40_000);
    assert_eq!(b.rescue.self_test_result(), Some(Ok(())));
    assert!(b.rescue.self_test_charge_skipped());
    assert!(b.has_event(|e| *e == Event::SelfTestChargeSkippedForHeat));
    assert!(b.has_event(|e| *e == Event::HeatChargeInhibit(HeatInhibit::CapacitorHot)));
    let p = b.board.prompts_only();
    assert!(p.contains(&Prompt::DeviceTooHot));
    // The analysis ran (without charging) and said shock; the device said
    // so, sent the responder to CPR at once and analysed again.
    let i = p.iter().position(|x| *x == Prompt::ShockAdvisedTooHot).unwrap();
    assert_eq!(p.get(i + 1), Some(&Prompt::StartCpr));
    assert!(count(&b, |e| *e == Event::ShockHeldForHeat) >= 2);
    assert!(!b.state_sequence().contains(&S::AnalysingCharging));
    assert!(!b.state_sequence().contains(&S::Fault));
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.board.cap_v < 1.0);
    b.assert_no_violations();
}

#[test]
fn a_too_hot_device_never_charges_during_cpr_and_once_cooled_charges_and_shocks_as_normal() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.set_all_temps(63);
    run_never_charging(&mut b, 20_000);
    assert!(b.run_until_state(S::Cpr, 20_000));
    // It cools during CPR (to 40 °C, under the 55 °C pre-charge limit):
    // still no charge until the next analysis.
    b.board.set_all_temps(40);
    run_never_charging(&mut b, 2_000);
    assert!(b.has_event(|e| *e == Event::HeatChargeAllowed));
    assert!(b.run_until_state(S::AnalysingCharging, 20_000));
    assert!(b.run_until(30_000, |b| b.shocks_fired() == 1));
    assert!(b.rescue.counters().shocks_held_for_heat >= 1);
    b.assert_no_violations();
}

#[test]
fn within_one_degree_of_the_limit_a_device_that_was_held_back_still_waits() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.set_all_temps(61);
    b.run_ms(3_000);
    b.board.temps = SensorSet::uniform(DeciC(595));
    b.board.sup_temp = Some(DeciC(595));
    run_never_charging(&mut b, 30_000);
    assert_eq!(b.rescue.heat_inhibit(), Some(HeatInhibit::CapacitorHot));
}

#[test]
fn becoming_too_hot_during_an_analysis_drops_the_charge_and_keeps_analysing() {
    // Slow decisions: the first comes 6 s into the analysis.
    let mut b = Bench::with_all(Config::default(), FakePlanner::default(), 250 * 6);
    b.run_to_analysing();
    b.run_ms(2_000);
    assert!(b.board.cap_v > 200.0, "charging during the analysis");
    b.board.set_all_temps(62);
    assert!(b.run_until_state(S::Cpr, 20_000));
    let seq = b.state_sequence();
    let i = seq.iter().position(|&s| s == S::AnalysingCharging).unwrap();
    assert_eq!(seq.get(i + 1), Some(&S::Analysing), "{seq:?}");
    assert_eq!(seq.get(i + 2), Some(&S::Cpr), "{seq:?}");
    assert_eq!(b.rescue.counters().analyses, 1, "the analysis was not restarted");
    assert!(b.has_event(|e| *e == Event::ShockHeldForHeat));
    assert_eq!(b.rescue.counters().charges_burned, 1);
    assert_eq!(b.shocks_fired(), 0);
    b.run_ms(3_000);
    assert!(b.board.cap_v < 50.0);
    b.assert_no_violations();
}

// ----- warm: charge only once a shock is advised ---------------------------------

#[test]
fn above_55_c_the_device_charges_only_after_a_shock_is_advised() {
    let mut b = Bench::new();
    b.board.set_all_temps(57);
    assert!(b.run_until(60_000, |b| b.shocks_fired() == 1));
    let seq = b.state_sequence();
    assert!(seq.contains(&S::Analysing) && !seq.contains(&S::AnalysingCharging), "{seq:?}");
    assert!(seq.contains(&S::Charging));
    assert!(b.has_event(|e| *e == Event::PrechargeSkippedWarm));
    assert!(b.board.prompts_only().contains(&Prompt::ShockAdvisedCharging));
    b.assert_no_violations();
}

#[test]
fn above_55_c_a_no_shock_analysis_burns_no_charge() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.rhythm = NON;
    b.board.set_all_temps(57);
    b.run_ms(60_000);
    assert!(b.rescue.counters().analyses >= 3);
    assert_eq!(b.rescue.counters().charges_burned, 0);
    assert_eq!(b.rescue.counters().capacitor_charges, 0);
}

// ----- sensors that fail must not block shocks -----------------------------------

#[test]
fn in_a_rescue_one_failed_temperature_sensor_does_not_block_shocks() {
    let mut b = Bench::new();
    b.board.temps.set(TempPoint::Capacitor, None);
    assert!(b.run_until(60_000, |b| b.shocks_fired() == 1));
    assert!(b.has_event(|e| matches!(e, Event::TemperatureSensorFault(TempPoint::Capacitor, _))));
}

#[test]
fn in_a_rescue_all_temperature_sensors_failed_still_shocks_and_records_it() {
    let mut b = Bench::new();
    b.board.temps = SensorSet::default();
    b.board.sup_temp = None;
    assert!(b.run_until(60_000, |b| b.shocks_fired() == 1));
    assert!(b.has_event(|e| *e == Event::TemperaturesUnknown));
    assert!(b.sup.events().iter().any(|e| e.event == Event::SupervisorTempIgnored(TempIgnored::SensorFailed)));
}

#[test]
fn a_supervisor_temperature_sensor_stuck_high_does_not_block_shocks() {
    let mut b = Bench::new();
    b.board.sup_temp = Some(DeciC::from_c(80));
    assert!(b.run_until(60_000, |b| b.shocks_fired() == 1));
    assert!(b
        .sup
        .events()
        .iter()
        .any(|e| e.event == Event::SupervisorTempIgnored(TempIgnored::DisagreesWithMain)));
    assert!(!b.sup.too_hot());
    // The main controller distrusts the supervisor's reading too.
    assert_eq!(b.rescue.heat_inhibit(), None);
}

// ----- the supervisor's own backstop ---------------------------------------------

#[test]
fn supervisor_too_hot_during_the_countdown_stops_the_shock_with_its_own_prompt_and_cpr_not_a_fault() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.board.temps = SensorSet::uniform(DeciC::from_c(58));
    b.board.sup_temp = Some(DeciC::from_c(67));
    assert!(b.run_until_state(S::Cpr, 5_000));
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.sup.too_hot());
    assert!(!b.sup.holding());
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::TooHot)));
    assert!(b.board.prompts_only().contains(&Prompt::TooHotNoShock));
    assert!(b.sup.events().iter().any(|e| e.event == Event::SupervisorTooHot(DeciC::from_c(67))));
    assert!(!b.state_sequence().contains(&S::Fault));
    assert!(!b.board.sup_alarm, "heat is not a supervisor fault");
    b.run_ms(3_000);
    assert!(b.board.cap_v < 50.0);
}

#[test]
fn supervisor_holds_charging_back_above_65_c_and_lets_it_resume_2_c_below() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.rhythm = NON;
    b.board.set_all_temps(66);
    b.run_ms(3_000);
    assert!(b.sup.too_hot());
    b.board.set_all_temps(64);
    b.run_ms(500);
    assert!(b.sup.too_hot(), "inside the hysteresis");
    b.board.set_all_temps(62);
    b.run_ms(500);
    assert!(!b.sup.too_hot());
}

// ----- pads and wear ----------------------------------------------------------------

#[test]
fn pads_heat_worn_is_advisory_in_a_rescue_like_expired_pads() {
    let mut b = Bench::new();
    b.board.cartridge = PadCartridge::HeatWorn;
    assert!(b.run_until(60_000, |b| b.shocks_fired() == 1));
    assert!(b.board.prompts_only().contains(&Prompt::ReplacePadsHeat));
}

#[test]
fn each_arming_counts_one_capacitor_charge() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.rhythm = NON;
    b.run_ms(70_000);
    let c = b.rescue.counters();
    let finished = c.analyses - u16::from(b.state().is_analysing());
    assert!(finished >= 3);
    assert!(c.capacitor_charges == finished || c.capacitor_charges == finished + 1);
    assert_eq!(c.charges_burned, finished);
}

// ----- D-34: the adult/child switch during an analysis ----------------------------

#[test]
fn d34_mode_switch_moved_during_analysis_recharges_for_the_new_mode_and_keeps_analysing() {
    let mut b = Bench::with_all(Config::default(), FakePlanner::default(), 250 * 6);
    b.run_to_analysing();
    b.run_ms(3_000);
    b.board.mode = PatientMode::Child;
    assert!(b.run_until(30_000, |b| b.shocks_fired() == 1));
    let seq = b.state_sequence();
    let first_cpr = seq.iter().position(|&s| s == S::Cpr);
    let first_delivery = seq.iter().position(|&s| s == S::Delivering);
    assert!(first_cpr.is_none() || first_delivery < first_cpr, "{seq:?}");
    assert_eq!(b.rescue.counters().analyses, 1, "the analysis carried on");
    assert!(b.has_event(|e| *e == Event::ChargeStarted(Volts(1000))));
    assert_eq!(b.board.requests[0].plan.charge_voltage, Volts(1000));
    assert_eq!(b.rescue.counters().capacitor_charges, 2, "adult charge, then child charge");
}

#[test]
fn d34_mode_switch_moved_after_shock_advised_cancels_the_shock() {
    // While still charging after "shock advised" (fast decisions)...
    let mut b = Bench::new();
    assert!(b.run_until_state(S::Charging, 30_000));
    b.board.mode = PatientMode::Child;
    assert!(b.run_until_state(S::Cpr, 5_000));
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::ModeChanged)));
    // ... and during the countdown.
    let mut b = Bench::new();
    b.run_to_countdown();
    b.board.mode = PatientMode::Child;
    assert!(b.run_until_state(S::Cpr, 5_000));
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::ModeChanged)));
    let fired: Vec<_> = b.board.fires.iter().collect();
    assert!(fired.is_empty());
}

// ----- a device that was above 70 °C and has cooled ---------------------------------

#[test]
fn a_device_that_was_above_70_c_and_has_cooled_still_shocks_if_its_switch_on_self_test_passes() {
    use crate::readiness::{DeviceReadiness, Reason, ReasonSet};
    let mut b = Bench::new();
    let mut reasons = ReasonSet::EMPTY;
    reasons.insert(Reason::ServiceNeededOverheat);
    reasons.insert(Reason::PadsReplaceAfterOverheat);
    b.rescue
        .record_standby_readiness(aed_types::Millis(0), DeviceReadiness { reasons }, 300, 0);
    assert!(b.run_until(60_000, |b| b.shocks_fired() == 1));
    assert!(b.has_event(|e| matches!(e, Event::StandbyReadiness { .. })));
    // Nothing about "not ready" is said aloud (D-50).
    assert!(!b.board.prompts_only().contains(&Prompt::DeviceTooHot));
}
