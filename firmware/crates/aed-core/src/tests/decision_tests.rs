// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for the owner decisions of 27 November 2026: charging during
//! analysis (D-29), the 3 s countdown (D-30), cancel during the self-test
//! and the stuck button (D-31), the wider pad-sensing range (D-32), and the
//! findings fixed with them (F-5, F-6, F-7). Run on the mock board with the
//! real supervisor alongside. Every name states the rule it checks.

use super::mock::{Bench, FakePlanner};
use crate::link::SupervisorFault;
use crate::prompts::Prompt;
use crate::rescue::{AbortReason, FaultKind, RescueState};
use crate::{Config, Event};
use aed_types::{PatientMode, PlanError, RhythmDecision, Volts};
use std::vec::Vec;

use RescueState as S;

const SHOCKABLE: RhythmDecision = RhythmDecision::Shockable;
const NON: RhythmDecision = RhythmDecision::NonShockable;

fn reach(b: &mut Bench, state: RescueState, max_ms: u32) {
    assert!(
        b.run_until_state(state, max_ms),
        "never reached {state:?}; states {:?}",
        b.state_sequence()
    );
}

fn count_events(b: &Bench, pred: impl Fn(&Event) -> bool) -> usize {
    b.rescue.events().iter().filter(|e| pred(&e.event)).count()
}

/// Prompts given at or after `t`.
fn prompts_from(b: &Bench, t: u32) -> Vec<Prompt> {
    b.board
        .prompts
        .iter()
        .filter(|&&(at, _)| at >= t)
        .map(|&(_, p)| p)
        .collect()
}

/// A shorter CPR period so a test covers several analyses.
fn short_cpr() -> Config {
    Config {
        cpr_period_ms: 15_000,
        ..super::cfg()
    }
}

// ----- D-29: charging during analysis ---------------------------------------------

#[test]
fn shockable_when_already_charged_goes_straight_to_the_countdown() {
    // The fake analyser's first answer comes 9 s into the analysis; the mock
    // charge takes 8 s, so the capacitor is already charged.
    let mut b = Bench::with_all(super::cfg(), FakePlanner::default(), 250 * 9);
    b.run_to_analysing();
    let start = b.board.now;
    reach(&mut b, S::Cpr, 30_000);
    let seq = b.state_sequence();
    let i = seq.iter().position(|&s| s == S::AnalysingCharging).unwrap();
    assert_eq!(
        &seq[i..i + 4],
        &[S::AnalysingCharging, S::Countdown, S::Delivering, S::Cpr]
    );
    let p = prompts_from(&b, start);
    let advised = p.iter().position(|&x| x == Prompt::ShockAdvised).unwrap();
    assert_eq!(
        &p[advised..advised + 5],
        &[
            Prompt::ShockAdvised,
            Prompt::StandClear,
            Prompt::Countdown(3),
            Prompt::Countdown(2),
            Prompt::Countdown(1),
        ]
    );
    assert!(!p.contains(&Prompt::ShockAdvisedCharging));
    assert_eq!(b.shocks_fired(), 1);
    // Shock about 9 s (answer) + 3 s (countdown) after the analysis began.
    let fired = b.board.fires[0].at - start;
    assert!((12_000..=12_100).contains(&fired), "fired {fired} ms in");
    assert_eq!(b.rescue.counters().charges_burned, 0);
    b.assert_no_violations();
}

#[test]
fn shockable_while_still_charging_waits_for_the_charge_then_counts_down() {
    let mut b = Bench::new();
    b.run_to_analysing();
    reach(&mut b, S::Charging, 5_000);
    assert!(b
        .board
        .prompts_only()
        .contains(&Prompt::ShockAdvisedCharging));
    reach(&mut b, S::Countdown, 10_000);
    // The countdown starts the moment the charge is reached.
    let charged_at = b
        .rescue
        .events()
        .iter()
        .find(|e| matches!(e.event, Event::Charged(_)))
        .unwrap()
        .at
        .0;
    assert!(
        b.board.now - charged_at <= 4,
        "countdown {} ms after charged",
        b.board.now - charged_at
    );
    reach(&mut b, S::Cpr, 10_000);
    assert_eq!(b.shocks_fired(), 1);
}

#[test]
fn charger_is_only_commanded_once_the_supervisor_holds_at_every_analysis_start() {
    // Pads already on at switch-on (first analysis straight after the
    // self-test), and pads attached later; shockable and not shockable;
    // several CPR periods. At every tick of every run: the main controller
    // never asks for the charger unless the supervisor is holding.
    for (pads_late, rhythm) in [
        (false, NON),
        (true, NON),
        (false, SHOCKABLE),
        (true, SHOCKABLE),
    ] {
        let mut b = Bench::with(short_cpr(), FakePlanner::default());
        b.board.rhythm = rhythm;
        if pads_late {
            b.board.pad_ohms = 5_000;
        }
        while b.board.now < 90_000 {
            if pads_late && b.board.now == 4_000 {
                b.board.pad_ohms = 80;
            }
            b.step();
            if b.board.charger.is_some() {
                assert!(
                    b.board.sup_hold,
                    "t={} {:?}: charger asked for while the supervisor is not holding",
                    b.board.now,
                    b.state()
                );
            }
        }
        let analyses = b.rescue.counters().analyses;
        assert!(analyses >= 4, "only {analyses} analyses");
        // Every analysis started a charge (and only after the self-test).
        let starts = count_events(&b, |e| *e == Event::ChargingDuringAnalysis);
        assert_eq!(starts, usize::from(analyses));
        b.assert_no_violations();
    }
}

#[test]
fn precharge_never_starts_before_the_self_test_has_passed() {
    // Pads on at switch-on: the first analysis follows the self-test at once.
    let mut b = Bench::new();
    let limit = f32::from(b.config.self_test_voltage.0)
        + f32::from(b.config.at_plan_allowance(b.config.self_test_voltage).0);
    while b.rescue.self_test_result().is_none() {
        b.step();
        assert!(
            !b.has_event(|e| matches!(e, Event::ChargeStarted(_))),
            "charging for a shock started during the self-test"
        );
        assert!(
            b.board.cap_v <= limit,
            "{} V during the self-test",
            b.board.cap_v
        );
        assert!(b.board.now < 10_000);
    }
    b.run_ms(2_000);
    let passed = b
        .rescue
        .events()
        .iter()
        .find(|e| e.event == Event::SelfTestPassed)
        .unwrap()
        .at;
    let first_charge = b
        .rescue
        .events()
        .iter()
        .find(|e| matches!(e.event, Event::ChargeStarted(_)))
        .unwrap()
        .at;
    assert!(first_charge >= passed);

    // A failed self-test: no charge for a shock, ever.
    let mut b = Bench::new();
    b.board.battery_under_load = crate::hal::BatteryStatus::Depleted;
    b.run_ms(60_000);
    assert_eq!(b.state(), S::Fault);
    assert!(!b.has_event(|e| matches!(e, Event::ChargeStarted(_))));
    assert!(b.board.cap_v < 1.0);
}

#[test]
fn no_new_charge_starts_until_the_previous_charge_is_confirmed_empty() {
    // A pad comes off 4 s into a charging analysis and is back at once, so
    // the next analysis starts while the first charge is still emptying.
    let mut b = Bench::with_all(super::cfg(), FakePlanner::default(), 2_500);
    b.run_to_analysing();
    b.run_ms(4_000);
    let charged_v = b.board.cap_v;
    assert!(charged_v > 1_000.0);
    b.board.pad_ohms = 3_000;
    let mut back_on = None;
    let mut last_v = b.board.cap_v;
    let mut emptied_before_recharge = false;
    let mut seen_emptying = false;
    let mut reentered_while_emptying = false;
    while b.board.now < 40_000 {
        if back_on.is_none() && b.state() == S::AttachPads {
            back_on = Some(b.board.now + 2);
        }
        if back_on == Some(b.board.now) {
            b.board.pad_ohms = 80;
        }
        let was_emptying = b.rescue.emptying();
        b.step();
        seen_emptying |= b.rescue.emptying();
        reentered_while_emptying |= b.rescue.emptying() && b.state().is_analysing();
        if b.rescue.emptying() {
            assert!(b.board.charger.is_none(), "charger on while emptying");
            assert!(!b.board.main_hold, "holding while emptying");
            if was_emptying {
                assert!(
                    b.board.cap_v <= last_v + 0.01,
                    "voltage rose while emptying"
                );
            }
        } else if seen_emptying && b.board.charger.is_some() && !emptied_before_recharge {
            emptied_before_recharge = b.has_event(|e| matches!(e, Event::CapacitorEmptied(_)));
            assert!(
                emptied_before_recharge,
                "recharged before the capacitor was confirmed empty"
            );
        }
        last_v = b.board.cap_v;
    }
    assert!(emptied_before_recharge, "never charged again");
    assert!(
        reentered_while_emptying,
        "the test must start an analysis while still emptying"
    );
    assert_eq!(b.rescue.counters().charges_burned, 1);
    b.assert_no_violations();
}

#[test]
fn a_charge_that_cannot_be_emptied_after_no_shock_advised_is_the_serious_fault() {
    // "No shock advised. Start CPR." is given at once; the capacitor then
    // fails to empty, and the device must say so within the emptying limit.
    let mut b = Bench::with_all(super::cfg(), FakePlanner::default(), 250 * 9);
    b.board.rhythm = NON;
    b.run_to_analysing();
    b.run_ms(8_500);
    assert!(b.board.cap_v > 1_700.0);
    b.board.dump_broken = true;
    reach(&mut b, S::Cpr, 5_000);
    let cpr_at = b.board.now;
    let p = prompts_from(&b, cpr_at - 2);
    assert_eq!(&p[..2], &[Prompt::NoShockAdvised, Prompt::StartCpr]);
    let max = b.config.disarm_time_limit_ms + 100;
    reach(&mut b, S::Fault, max);
    assert!(b.board.now - cpr_at <= b.config.disarm_time_limit_ms + 4);
    assert_eq!(b.rescue.fault(), Some(FaultKind::CapacitorNotEmptied));
    assert_eq!(
        b.board.prompts_only().last(),
        Some(&Prompt::SeriousFaultContinueCpr)
    );
    assert!(!b.board.main_hold && b.board.charger.is_none());
    // It does not keep re-raising the fault.
    let n = count_events(&b, |e| *e == Event::Fault(FaultKind::CapacitorNotEmptied));
    b.run_ms(10_000);
    assert_eq!(
        count_events(&b, |e| *e == Event::Fault(FaultKind::CapacitorNotEmptied)),
        n
    );
}

#[test]
fn mode_change_during_precharge_empties_and_recharges_for_the_new_mode_without_stopping_the_analysis(
) {
    let mut b = Bench::with_all(super::cfg(), FakePlanner::default(), 250 * 6);
    b.run_to_analysing();
    b.run_ms(3_000);
    assert!(b.board.cap_v > 500.0);
    b.board.mode = PatientMode::Child;
    reach(&mut b, S::Cpr, 30_000);
    // The analysis was not abandoned: straight on to the shock.
    let seq = b.state_sequence();
    let i = seq.iter().position(|&s| s == S::AnalysingCharging).unwrap();
    assert!(
        matches!(seq.get(i + 1), Some(S::Charging | S::Countdown)),
        "{seq:?}"
    );
    assert!(
        !seq[..seq.len() - 1].contains(&S::Cpr),
        "went to CPR: {seq:?}"
    );
    assert!(b.has_event(|e| *e == Event::ChargeStarted(Volts(1000))));
    assert_eq!(b.board.requests.len(), 1);
    assert_eq!(b.board.requests[0].plan.charge_voltage, Volts(1000));
    assert!((950.0..=1_050.0).contains(&b.board.fires[0].voltage));
    assert_eq!(
        b.rescue.counters().charges_burned,
        1,
        "the adult charge was burned off"
    );
    b.assert_no_violations();
}

#[test]
fn every_analysis_without_a_shock_burns_its_charge_and_counts_it() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.rhythm = NON;
    b.run_ms(70_000);
    let c = b.rescue.counters();
    assert!(c.analyses >= 4);
    // The last analysis may still be under way.
    let finished = c.analyses - u16::from(b.state().is_analysing());
    assert_eq!(c.charges_burned, finished);
    assert_eq!(
        count_events(&b, |e| *e == Event::ChargeBurnedOff),
        usize::from(c.charges_burned)
    );
}

#[test]
fn charging_during_analysis_can_be_switched_off() {
    let config = Config {
        charge_during_analysis: false,
        ..super::cfg()
    };
    let mut b = Bench::with(config, FakePlanner::default());
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Cpr, 60_000);
    assert_eq!(
        b.state_sequence(),
        std::vec![
            S::SelfTest,
            S::AttachPads,
            S::Analysing,
            S::Charging,
            S::Countdown,
            S::Delivering,
            S::Cpr
        ]
    );
    let mut b = Bench::with(config, FakePlanner::default());
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    assert!(!b.has_event(|e| matches!(e, Event::ChargeStarted(_))));
    assert!(b.board.cap_v < 1.0);
    assert_eq!(b.rescue.counters().charges_burned, 0);
}

// ----- D-30: the 3 s countdown ----------------------------------------------------

#[test]
fn countdown_says_stand_clear_three_two_one_and_lasts_three_seconds() {
    let mut b = Bench::new();
    b.run_to_countdown();
    let start = b.board.now;
    reach(&mut b, S::Delivering, 5_000);
    let took = b.board.now - start;
    assert!((3_000..=3_004).contains(&took), "countdown {took} ms");
    let p = prompts_from(&b, start - 2);
    assert_eq!(
        &p[..4],
        &[
            Prompt::StandClear,
            Prompt::Countdown(3),
            Prompt::Countdown(2),
            Prompt::Countdown(1)
        ]
    );
    assert_eq!(Prompt::StandClear.text(), "Stand clear.");
}

// ----- D-31: cancel during the self-test, and a stuck button ------------------------

#[test]
fn cancel_held_without_a_break_for_the_stuck_time_is_a_stuck_button_fault_with_its_own_prompt() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    b.board.cancel = true;
    let pressed = b.board.now;
    let max = b.config.cancel_stuck_ms + 1_000;
    reach(&mut b, S::Fault, max);
    let took = b.board.now - pressed;
    assert!(
        (b.config.cancel_stuck_ms..=b.config.cancel_stuck_ms + 10).contains(&took),
        "stuck fault after {took} ms"
    );
    assert!(matches!(
        b.rescue.fault(),
        Some(FaultKind::CancelStuck | FaultKind::Supervisor(SupervisorFault::CancelStuck))
    ));
    assert!(b.has_event(|e| *e == Event::CancelStuck));
    assert_eq!(
        b.board.prompts_only().last(),
        Some(&Prompt::CancelStuckContinueCpr)
    );
    // The supervisor applies the same rule on its own view of the button.
    b.run_ms(100);
    assert_eq!(b.sup.fault(), Some(SupervisorFault::CancelStuck));
    assert!(b.board.sup_alarm);
    assert!(b.board.cap_v < 50.0);
}

#[test]
fn cancel_held_from_switch_on_is_ignored_by_the_self_test_and_reported_as_stuck() {
    let mut b = Bench::new();
    b.board.cancel = true;
    reach(&mut b, S::Fault, 20_000);
    assert_eq!(
        b.rescue.self_test_result(),
        Some(Ok(())),
        "self-test must pass"
    );
    assert!(b.has_event(|e| *e == Event::CancelIgnoredDuringSelfTest));
    assert_eq!(b.rescue.fault(), Some(FaultKind::CancelStuck));
    let at = b.board.now;
    assert!(
        (b.config.cancel_stuck_ms..=b.config.cancel_stuck_ms + 10).contains(&at),
        "stuck fault at {at} ms"
    );
    assert_eq!(b.shocks_fired(), 0);
}

#[test]
fn a_cancel_press_shorter_than_the_stuck_time_is_not_a_fault() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    b.board.cancel = true;
    b.run_ms(b.config.cancel_stuck_ms - 500);
    b.board.cancel = false;
    b.run_ms(5_000);
    assert_eq!(b.state(), S::Cpr);
    assert_eq!(b.rescue.fault(), None);
    assert_eq!(b.sup.fault(), None);
}

// ----- D-32: pads on but above the shock range --------------------------------------

#[test]
fn pads_above_the_shock_range_count_as_on_and_are_analysed_with_press_pads_firmly() {
    let mut b = Bench::new();
    b.board.pad_ohms = 200;
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    assert!(b.state_sequence().contains(&S::AnalysingCharging));
    let p = b.board.prompts_only();
    assert!(
        !p.contains(&Prompt::AttachPads),
        "pads are on: never 'attach pads'"
    );
    let press = p
        .iter()
        .position(|&x| x == Prompt::PressPadsFirmly)
        .unwrap();
    assert_eq!(p.get(press + 1), Some(&Prompt::AnalysingDoNotTouch));
    assert!(b.has_event(|e| *e == Event::PadContactPoor(aed_types::Ohms(200))));
}

#[test]
fn pads_still_above_the_shock_range_at_delivery_give_no_shock_poor_contact_and_cpr() {
    let mut b = Bench::new();
    b.board.pad_ohms = 200;
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Countdown, 60_000);
    reach(&mut b, S::Cpr, 10_000);
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.board.requests.is_empty());
    assert!(b.has_event(
        |e| *e == Event::ShockAborted(AbortReason::PlanRefused(PlanError::ImpedanceTooHigh))
    ));
    let p = b.board.prompts_only();
    let i = p
        .iter()
        .position(|&x| x == Prompt::PadContactPoorNoShock)
        .unwrap();
    assert_eq!(p.get(i + 1), Some(&Prompt::StartCpr));
    // Emptied, and during CPR the responder is asked again to press the pads.
    assert!(b.run_until(b.config.disarm_time_limit_ms, |b| !b.rescue.emptying()));
    assert!(b.board.cap_v < 50.0);
    let cpr_at = b.board.now;
    b.run_ms(40_000);
    let reminders: Vec<u32> = b
        .board
        .prompts
        .iter()
        .filter(|&&(t, x)| x == Prompt::PressPadsFirmly && t > cpr_at)
        .map(|&(t, _)| t)
        .collect();
    assert!(reminders.len() >= 2, "{reminders:?}");
    for w in reminders.windows(2) {
        assert!(w[1] - w[0] >= b.config.press_pads_repeat_ms);
    }
    assert!(!b.board.prompts_only().contains(&Prompt::AttachPads));
}

#[test]
fn pads_that_come_into_the_shock_range_before_delivery_are_shocked() {
    let mut b = Bench::new();
    b.board.pad_ohms = 200;
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Countdown, 60_000);
    b.board.pad_ohms = 150;
    reach(&mut b, S::Cpr, 10_000);
    assert_eq!(b.shocks_fired(), 1);
    b.assert_no_violations();
}

#[test]
fn press_pads_firmly_is_never_said_while_hands_must_be_off() {
    let mut b = Bench::with(short_cpr(), FakePlanner::default());
    b.board.pad_ohms = 200;
    b.board.rhythm = SHOCKABLE;
    let mut said = 0;
    while b.board.now < 90_000 {
        let before = b.board.prompts.len();
        b.step();
        for &(_, p) in &b.board.prompts[before..] {
            if p == Prompt::PressPadsFirmly {
                said += 1;
                // Said either during CPR or as an analysis starts (the
                // hands-off instruction follows it at once).
                let ok = b.state() == S::Cpr
                    || (b.state().is_analysing()
                        && b.rescue.time_in_state_ms(aed_types::Millis(b.board.now)) == 0);
                assert!(ok, "t={} 'press pads' in {:?}", b.board.now, b.state());
            }
        }
    }
    assert!(said >= 3);
}

#[test]
fn pads_above_the_sensing_range_still_count_as_off() {
    let mut b = Bench::new();
    b.board.pad_ohms = 251;
    reach(&mut b, S::AttachPads, 30_000);
    b.run_ms(30_000);
    assert_eq!(b.state(), S::AttachPads);
    assert!(!b.board.prompts_only().contains(&Prompt::PressPadsFirmly));
}

#[test]
fn shock_range_is_unchanged_by_the_wider_sensing_range() {
    use aed_types::{in_shock_range, pads_sensed, Ohms};
    assert!(!pads_sensed(Ohms(24)) && !in_shock_range(Ohms(24)));
    assert!(pads_sensed(Ohms(25)) && in_shock_range(Ohms(25)));
    assert!(pads_sensed(Ohms(175)) && in_shock_range(Ohms(175)));
    assert!(pads_sensed(Ohms(176)) && !in_shock_range(Ohms(176)));
    assert!(pads_sensed(Ohms(250)) && !in_shock_range(Ohms(250)));
    assert!(!pads_sensed(Ohms(251)));
}

// ----- F-6: one instruction after the pads come off --------------------------------

#[test]
fn pads_off_abort_goes_straight_to_attach_pads_without_start_cpr() {
    for state in [S::Charging, S::Countdown] {
        let mut b = Bench::new();
        reach(&mut b, state, 60_000);
        b.run_ms(500);
        let at = b.board.now;
        b.board.pad_ohms = 3_000;
        reach(&mut b, S::AttachPads, 2_000);
        b.run_ms(3_000);
        let p = prompts_from(&b, at);
        assert_eq!(
            &p[..2],
            &[Prompt::CheckPads, Prompt::AttachPads],
            "{state:?}"
        );
        assert!(!p.contains(&Prompt::StartCpr), "{state:?}: {p:?}");
        assert!(!b.state_sequence().contains(&S::Cpr), "{state:?}");
        assert!(b.board.cap_v < 50.0);
    }
}

// ----- F-7: a weak charger at switch-on --------------------------------------------

#[test]
fn weak_charger_at_switch_on_is_reported_within_about_a_second() {
    let mut b = Bench::new();
    b.board.charger_broken = true;
    reach(&mut b, S::Fault, 10_000);
    let first = b.board.prompts.first().unwrap().0;
    assert!(first <= 1_500, "first prompt at {first} ms");
    assert_eq!(
        b.rescue.fault(),
        Some(FaultKind::SelfTest(crate::SelfTestFailure::ChargeTimeout))
    );
}
