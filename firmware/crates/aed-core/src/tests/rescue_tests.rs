// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests of the main controller's rescue sequence, run on the mock board with
//! the real supervisor alongside. Every name states the rule it checks.

use super::mock::{Bench, FakePlanner};
use crate::hal::{BatteryStatus, PadCartridge};
use crate::link::SupervisorFault;
use crate::prompts::Prompt;
use crate::rescue::{AbortReason, FaultKind, RescueState, ShockOutcome};
use crate::selftest::SelfTestFailure;
use crate::{Config, Event};
use aed_types::{PatientMode, PlanError, RhythmDecision};
use std::vec;
use std::vec::Vec;

use RescueState as S;

const SHOCKABLE: RhythmDecision = RhythmDecision::Shockable;
const NON: RhythmDecision = RhythmDecision::NonShockable;
const NOT_SURE: RhythmDecision = RhythmDecision::Indeterminate;

/// Run until `state`, panicking with the state history if it never comes.
fn reach(b: &mut Bench, state: RescueState, max_ms: u32) {
    assert!(
        b.run_until_state(state, max_ms),
        "never reached {state:?}; states {:?}",
        b.state_sequence()
    );
}

/// After an abort or fault, the capacitor is empty and both holds are off.
fn assert_capacitor_emptied(b: &Bench) {
    assert!(
        b.board.cap_v < 50.0,
        "capacitor still at {} V",
        b.board.cap_v
    );
    assert!(!b.board.main_hold, "main still holding");
    assert!(b.board.charger.is_none(), "charger still on");
}

// ----- the full rescue ------------------------------------------------------------

#[test]
fn full_rescue_vf_shock_cpr_normal_rhythm_cpr_exact_states_and_prompts() {
    let mut b = Bench::new();
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Cpr, 60_000);
    // The patient's rhythm returns to normal during CPR.
    b.board.rhythm = NON;
    b.run_ms(120_000);
    reach(&mut b, S::Cpr, 30_000);
    b.run_ms(1_000);

    assert_eq!(
        b.state_sequence(),
        vec![
            S::SelfTest,
            S::AttachPads, // self-test passed; pads already on
            S::Analysing,
            S::Charging,
            S::Countdown,
            S::Delivering,
            S::Disarming,
            S::Cpr,
            S::Analysing,
            S::Cpr,
        ]
    );
    let prompts: Vec<Prompt> = b
        .board
        .prompts_only()
        .into_iter()
        .filter(|p| *p != Prompt::AttachPads)
        .collect();
    assert_eq!(
        prompts,
        vec![
            Prompt::AnalysingDoNotTouch,
            Prompt::ShockAdvisedCharging,
            Prompt::StandClear,
            Prompt::Countdown(5),
            Prompt::Countdown(4),
            Prompt::Countdown(3),
            Prompt::Countdown(2),
            Prompt::Countdown(1),
            Prompt::ShockDelivered,
            Prompt::StartCpr,
            Prompt::StopCprAnalysing,
            Prompt::NoShockAdvised,
            Prompt::StartCpr,
        ]
    );
    // The pads were on from the start, so "attach pads" was never needed.
    assert!(!b.board.prompts_only().contains(&Prompt::AttachPads));
    assert_eq!(b.shocks_fired(), 1);
    assert_eq!(b.board.requests.len(), 1);
    assert_eq!(b.rescue.counters().shocks_delivered, 1);
    assert_eq!(b.rescue.counters().cpr_periods, 2);
    assert_eq!(b.rescue.counters().analyses, 2);
    assert!(b.has_event(|e| *e == Event::Shock(ShockOutcome::Delivered)));
    assert!(b.has_event(|e| *e == Event::SelfTestPassed));
    b.assert_no_violations();
}

#[test]
fn shock_then_cpr_then_reanalyse_and_shock_again_if_still_shockable() {
    let mut b = Bench::new();
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Cpr, 60_000);
    let cpr_start = b.board.now;
    reach(&mut b, S::Analysing, 130_000);
    let cpr_length = b.board.now - cpr_start;
    assert!(
        (120_000..=120_010).contains(&cpr_length),
        "CPR lasted {cpr_length} ms"
    );
    reach(&mut b, S::Cpr, 60_000);
    assert_eq!(b.shocks_fired(), 2);
    assert_eq!(b.rescue.counters().shocks_delivered, 2);
    b.assert_no_violations();
}

#[test]
fn analyzer_is_reset_after_delivery() {
    let mut b = Bench::new();
    reach(&mut b, S::Delivering, 60_000);
    let before = b.resets.get();
    reach(&mut b, S::Disarming, 2_000);
    assert_eq!(b.resets.get(), before + 1);
    assert_eq!(b.rescue.last_decision(), None);
}

#[test]
fn residual_charge_is_emptied_and_verified_after_shock() {
    let mut b = Bench::new();
    reach(&mut b, S::Disarming, 60_000);
    assert_eq!(b.shocks_fired(), 1);
    assert!(
        b.board.cap_v > 50.0,
        "model keeps residual charge after a shock"
    );
    reach(&mut b, S::Cpr, 5_000);
    assert!(b.board.cap_v < 50.0);
    let events: Vec<Event> = b.rescue.events().iter().map(|e| e.event).collect();
    let shock = events
        .iter()
        .position(|e| *e == Event::Shock(ShockOutcome::Delivered))
        .unwrap();
    let emptied = events
        .iter()
        .position(|e| matches!(e, Event::CapacitorEmptied(_)))
        .unwrap();
    assert!(emptied > shock);
}

#[test]
fn shock_is_requested_once_per_shock() {
    let mut b = Bench::new();
    reach(&mut b, S::Cpr, 60_000);
    assert_eq!(b.board.requests.len(), 1);
    b.assert_no_violations();
}

// ----- the decision rules ----------------------------------------------------------

#[test]
fn indeterminate_never_charges() {
    let mut b = Bench::new();
    b.board.rhythm = NOT_SURE;
    reach(&mut b, S::Cpr, 60_000);
    b.run_ms(300_000);
    assert!(!b.state_sequence().contains(&S::Charging));
    assert_eq!(b.board.requests.len(), 0);
    assert_eq!(b.shocks_fired(), 0);
    // "Not sure" for ever ends each analysis as inconclusive, never as a shock.
    assert!(b.board.prompts_only().contains(&Prompt::AnalysisInconclusive));
    assert!(!b.board.prompts_only().contains(&Prompt::NoShockAdvised));
}

#[test]
fn non_shockable_goes_to_cpr_without_charging() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    assert_eq!(
        b.state_sequence(),
        vec![S::SelfTest, S::AttachPads, S::Analysing, S::Cpr]
    );
    assert_eq!(b.board.requests.len(), 0);
    assert!(b.board.cap_v < 1.0);
}

#[test]
fn only_shockable_can_start_charging() {
    for rhythm in [SHOCKABLE, NON, NOT_SURE] {
        let mut b = Bench::new();
        b.board.rhythm = rhythm;
        b.run_ms(30_000);
        let charged = b.state_sequence().contains(&S::Charging);
        assert_eq!(charged, rhythm == SHOCKABLE, "rhythm {rhythm:?}");
    }
}

#[test]
fn not_sure_wins_over_shockable_within_one_analysis() {
    // A shockable and a not-sure decision arrive before the controller acts:
    // the not-sure one must win.
    let mut b = Bench::new();
    b.run_to_analysing();
    let now = aed_types::Millis(b.board.now);
    for _ in 0..250 {
        b.rescue
            .on_ecg_sample(super::mock::sample_for(SHOCKABLE), now);
    }
    for _ in 0..250 {
        b.rescue
            .on_ecg_sample(super::mock::sample_for(NOT_SURE), now);
    }
    b.board.rhythm = NOT_SURE;
    b.run_ms(100);
    // "Not sure" is not final, so the analysis carries on; it must not charge.
    assert_eq!(b.state(), S::Analysing);
    let max = b.config.analysis_time_limit_ms + 1_000;
    reach(&mut b, S::Cpr, max);
    assert!(!b.state_sequence().contains(&S::Charging));
    assert_eq!(b.board.requests.len(), 0);
}

/// Run until the fake analyser has emitted `n` more decisions.
fn run_decisions(b: &mut Bench, n: usize) {
    let start = b
        .rescue
        .events()
        .iter()
        .filter(|e| matches!(e.event, Event::Decision(_)))
        .count();
    let reached = b.run_until(30_000, |b| {
        b.rescue
            .events()
            .iter()
            .filter(|e| matches!(e.event, Event::Decision(_)))
            .count()
            >= start + n
    });
    assert!(reached, "analyser stopped deciding");
}

fn count_prompt(b: &Bench, p: Prompt) -> usize {
    b.board.prompts_only().iter().filter(|&&x| x == p).count()
}

#[test]
fn not_sure_during_analysis_is_not_final_and_a_later_shockable_charges() {
    // Scripted analyser: not sure, not sure, then shockable.
    let mut b = Bench::new();
    b.board.rhythm = NOT_SURE;
    b.run_to_analysing();
    run_decisions(&mut b, 2);
    b.run_ms(10);
    assert_eq!(b.state(), S::Analysing, "two 'not sure' answers must not end the analysis");
    assert!(!b.board.prompts_only().contains(&Prompt::NoShockAdvised));
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Charging, 3_000);
    assert!(!b.state_sequence().contains(&S::Cpr));
}

#[test]
fn not_sure_during_analysis_repeats_hands_off_prompt_at_most_every_reminder_interval() {
    let mut b = Bench::new();
    b.board.rhythm = NOT_SURE;
    b.run_to_analysing();
    let entered = b.board.now;
    run_decisions(&mut b, 1);
    b.run_ms(10);
    // The first "not sure" re-issues the hands-off instruction at once.
    assert_eq!(count_prompt(&b, Prompt::DoNotTouchPatient), 1);
    let max = b.config.analysis_time_limit_ms + 1_000;
    reach(&mut b, S::Cpr, max);
    let times: Vec<u32> = b
        .board
        .prompts
        .iter()
        .filter(|&&(t, p)| p == Prompt::DoNotTouchPatient && t >= entered)
        .map(|&(t, _)| t)
        .collect();
    assert!(times.len() >= 2, "reminder should repeat during a long analysis: {times:?}");
    for w in times.windows(2) {
        assert!(
            w[1] - w[0] >= b.config.hands_off_reminder_ms,
            "reminders too close: {times:?}"
        );
    }
}

#[test]
fn not_sure_for_the_whole_analysis_goes_to_cpr_at_the_limit_as_inconclusive() {
    let mut b = Bench::new();
    b.board.rhythm = NOT_SURE;
    b.run_to_analysing();
    let start = b.board.now;
    reach(&mut b, S::Cpr, 60_000);
    let took = b.board.now - start;
    let limit = b.config.analysis_time_limit_ms;
    assert!((limit..=limit + 10).contains(&took), "analysis lasted {took} ms");
    assert!(b.has_event(|e| *e == Event::AnalysisInconclusive));
    assert!(!b.has_event(|e| *e == Event::AnalysisTimedOut));
    // The responder is told, then told to start CPR, at the same moment.
    let p = b.board.prompts_only();
    let i = p.iter().position(|&x| x == Prompt::AnalysisInconclusive).unwrap();
    assert_eq!(p.get(i + 1), Some(&Prompt::StartCpr));
    assert!(!b.state_sequence().contains(&S::Charging));
    assert_eq!(b.shocks_fired(), 0);
}

#[test]
fn not_sure_then_non_shockable_goes_to_cpr_at_once() {
    let mut b = Bench::new();
    b.board.rhythm = NOT_SURE;
    b.run_to_analysing();
    run_decisions(&mut b, 1);
    b.board.rhythm = NON;
    run_decisions(&mut b, 1);
    b.run_ms(4);
    assert_eq!(b.state(), S::Cpr);
    assert!(b.board.prompts_only().contains(&Prompt::NoShockAdvised));
    assert!(!b.has_event(|e| *e == Event::AnalysisInconclusive));
    assert_eq!(b.board.requests.len(), 0);
}

#[test]
fn not_sure_during_countdown_still_aborts_the_shock() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.run_ms(1_000);
    b.board.rhythm = NOT_SURE;
    let seen = b.run_until(3_000, |b| b.board.last_decision.get() == Some(NOT_SURE));
    assert!(seen);
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Cpr, 10_000);
    assert_eq!(b.shocks_fired(), 0);
    assert_eq!(b.board.requests.len(), 0);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::RhythmChanged)));
    assert_capacitor_emptied(&b);
}

#[test]
fn analysis_timeout_counts_as_not_sure_and_goes_to_cpr() {
    // Analyser that never decides within the time limit.
    let mut b = Bench::with_all(Config::default(), FakePlanner::default(), 1_000_000);
    b.board.rhythm = SHOCKABLE;
    b.run_to_analysing();
    let start = b.board.now;
    reach(&mut b, S::Cpr, 20_000);
    let took = b.board.now - start;
    assert!(
        (12_000..=12_010).contains(&took),
        "analysis lasted {took} ms"
    );
    assert!(b.has_event(|e| *e == Event::AnalysisTimedOut));
    assert!(!b.state_sequence().contains(&S::Charging));
}

#[test]
fn analysis_keeps_running_during_charging_and_countdown() {
    let mut b = Bench::new();
    reach(&mut b, S::Delivering, 60_000);
    let decisions_while_armed = b
        .rescue
        .events()
        .iter()
        .skip_while(|e| !matches!(e.event, Event::ChargeStarted(_)))
        .filter(|e| matches!(e.event, Event::Decision(_)))
        .count();
    // Roughly one per second over about 13 s of charging and countdown.
    assert!(decisions_while_armed >= 10, "only {decisions_while_armed}");
}

#[test]
fn no_shock_when_rhythm_turns_non_shockable_during_countdown() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.run_ms(1_500);
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 10_000);
    assert_eq!(b.shocks_fired(), 0);
    assert_eq!(b.board.requests.len(), 0);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::RhythmChanged)));
    assert!(b
        .board
        .prompts_only()
        .contains(&Prompt::RhythmChangedNoShock));
    assert_capacitor_emptied(&b);
}

#[test]
fn no_shock_when_rhythm_turns_indeterminate_during_charging() {
    let mut b = Bench::new();
    reach(&mut b, S::Charging, 60_000);
    b.run_ms(2_000);
    b.board.rhythm = NOT_SURE;
    reach(&mut b, S::Cpr, 10_000);
    assert!(!b.state_sequence().contains(&S::Countdown));
    assert_eq!(b.shocks_fired(), 0);
    assert_capacitor_emptied(&b);
}

#[test]
fn rhythm_that_flickers_non_shockable_once_still_aborts() {
    // One non-shockable decision during the countdown is enough, even if
    // later decisions are shockable again.
    let mut b = Bench::new();
    b.run_to_countdown();
    b.board.rhythm = NON;
    let flicker = b.run_until(3_000, |b| b.board.last_decision.get() == Some(NON));
    assert!(flicker);
    b.board.rhythm = SHOCKABLE;
    b.run_ms(8_000);
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::RhythmChanged)));
}

#[test]
fn stale_confirmation_at_delivery_gives_no_shock() {
    // Analyser decides only every 15 s, so at the end of the countdown the
    // last "shockable" is about 13 s old: older than the 5 s allowed.
    let config = Config {
        analysis_time_limit_ms: 20_000,
        ..Config::default()
    };
    let mut b = Bench::with_all(config, FakePlanner::default(), 250 * 15);
    b.board.rhythm = SHOCKABLE;
    reach(&mut b, S::Cpr, 60_000);
    assert!(b.state_sequence().contains(&S::Countdown));
    assert_eq!(b.shocks_fired(), 0);
    assert_eq!(b.board.requests.len(), 0);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::ConfirmationStale)));
}

#[test]
fn decisions_during_cpr_are_not_used() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    let decisions_before = b
        .rescue
        .events()
        .iter()
        .filter(|e| matches!(e.event, Event::Decision(_)))
        .count();
    b.board.rhythm = SHOCKABLE;
    b.run_ms(60_000);
    let decisions_after = b
        .rescue
        .events()
        .iter()
        .filter(|e| matches!(e.event, Event::Decision(_)))
        .count();
    assert_eq!(decisions_before, decisions_after);
    assert_eq!(b.state(), S::Cpr);
}

// ----- the shock plan --------------------------------------------------------------

#[test]
fn plan_is_made_from_resistance_measured_just_before_delivery() {
    let mut b = Bench::new();
    b.board.pad_ohms = 60;
    b.run_to_countdown();
    b.run_ms(4_000);
    b.board.pad_ohms = 150;
    reach(&mut b, S::Cpr, 10_000);
    let plan = b.board.requests[0].plan;
    // The fake planner sets phase 1 = 3000 + (ohms - 25) * 60 microseconds.
    assert_eq!(plan.phase1.0, 3000 + (150 - 25) * 60);
    assert!(b.has_event(|e| *e == Event::PadResistance(aed_types::Ohms(150))));
}

#[test]
fn plan_error_at_delivery_disarms_without_shock() {
    let mut b = Bench::new();
    b.run_to_countdown();
    // Just before the end: too quick for the debounced pad monitor to notice,
    // but the fresh reading used for the plan sees it.
    b.run_ms(4_900);
    b.board.pad_ohms = 10;
    reach(&mut b, S::Cpr, 10_000);
    assert_eq!(b.shocks_fired(), 0);
    assert_eq!(b.board.requests.len(), 0);
    assert!(b.has_event(
        |e| *e == Event::ShockAborted(AbortReason::PlanRefused(PlanError::ImpedanceTooLow))
    ));
    assert_capacitor_emptied(&b);
}

#[test]
fn plan_voltage_that_does_not_match_the_charge_is_a_fault() {
    let planner = FakePlanner {
        plan_voltage_override: Some(1500),
        ..FakePlanner::default()
    };
    let mut b = Bench::with(Config::default(), planner);
    reach(&mut b, S::Fault, 60_000);
    assert_eq!(b.rescue.fault(), Some(FaultKind::PlanMismatch));
    assert_eq!(b.shocks_fired(), 0);
    assert_capacitor_emptied(&b);
}

#[test]
fn planner_charge_voltage_above_overvoltage_limit_is_a_fault() {
    let planner = FakePlanner {
        adult_v: 2_500,
        ..FakePlanner::default()
    };
    let mut b = Bench::with(Config::default(), planner);
    reach(&mut b, S::Fault, 60_000);
    assert_eq!(b.rescue.fault(), Some(FaultKind::PlanMismatch));
    assert!(!b.state_sequence().contains(&S::Charging));
}

// ----- pads ------------------------------------------------------------------------

#[test]
fn pads_removed_during_countdown_aborts() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.run_ms(2_000);
    b.board.pad_ohms = 2_000;
    b.run_ms(3_500);
    assert_eq!(b.shocks_fired(), 0);
    assert_eq!(b.board.requests.len(), 0);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::PadsOff)));
    assert!(b.board.prompts_only().contains(&Prompt::CheckPads));
    assert_capacitor_emptied(&b);
}

#[test]
fn pads_removed_during_charging_aborts() {
    let mut b = Bench::new();
    reach(&mut b, S::Charging, 60_000);
    b.run_ms(1_000);
    b.board.pad_ohms = 2_000;
    b.run_ms(2_000);
    assert!(!b.state_sequence().contains(&S::Countdown));
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::PadsOff)));
    assert_capacitor_emptied(&b);
}

#[test]
fn pads_off_during_analysis_goes_to_attach_pads() {
    let mut b = Bench::with_all(Config::default(), FakePlanner::default(), 2_500);
    b.run_to_analysing();
    b.board.pad_ohms = 2_000;
    reach(&mut b, S::AttachPads, 1_000);
    assert!(b.board.prompts_only().contains(&Prompt::AttachPads));
}

#[test]
fn pads_off_during_cpr_goes_to_attach_pads_and_reattaching_reanalyses() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    b.board.pad_ohms = 2_000;
    reach(&mut b, S::AttachPads, 1_000);
    b.board.pad_ohms = 90;
    reach(&mut b, S::Analysing, 1_000);
    assert_eq!(
        b.board.prompts_only().last(),
        Some(&Prompt::AnalysingDoNotTouch)
    );
}

#[test]
fn pad_glitch_shorter_than_debounce_is_ignored_during_analysis() {
    let mut b = Bench::with_all(Config::default(), FakePlanner::default(), 2_500);
    b.run_to_analysing();
    b.board.pad_ohms = 2_000;
    b.run_ms(100);
    b.board.pad_ohms = 80;
    b.run_ms(500);
    assert_eq!(b.state(), S::Analysing);
}

#[test]
fn no_pads_at_switch_on_prompts_attach_pads_and_waits() {
    let mut b = Bench::new();
    b.board.pad_ohms = 5_000;
    reach(&mut b, S::AttachPads, 30_000);
    b.run_ms(60_000);
    assert_eq!(b.state(), S::AttachPads);
    let attach = b
        .board
        .prompts_only()
        .iter()
        .filter(|p| **p == Prompt::AttachPads)
        .count();
    assert!(
        attach >= 4,
        "attach-pads prompt repeated only {attach} times"
    );
}

// ----- cancel ----------------------------------------------------------------------

#[test]
fn cancel_during_countdown_empties_capacitor() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.run_ms(2_000);
    b.board.cancel = true;
    reach(&mut b, S::Cpr, 5_000);
    b.board.cancel = false;
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::Cancelled)));
    assert!(b.has_event(|e| *e == Event::CancelPressed));
    assert!(b.board.prompts_only().contains(&Prompt::ShockCancelled));
    assert_capacitor_emptied(&b);
}

#[test]
fn cancel_during_charging_empties_capacitor() {
    let mut b = Bench::new();
    reach(&mut b, S::Charging, 60_000);
    b.run_ms(3_000);
    b.board.cancel = true;
    reach(&mut b, S::Cpr, 5_000);
    assert_eq!(b.shocks_fired(), 0);
    assert_capacitor_emptied(&b);
}

#[test]
fn cancel_during_analysis_goes_to_cpr_without_charging() {
    let mut b = Bench::with_all(Config::default(), FakePlanner::default(), 2_500);
    b.run_to_analysing();
    b.board.cancel = true;
    reach(&mut b, S::Cpr, 1_000);
    b.board.cancel = false;
    assert!(!b.state_sequence().contains(&S::Charging));
    assert!(b.board.prompts_only().contains(&Prompt::AnalysisCancelled));
}

#[test]
fn cancel_during_delivering_withdraws_the_request() {
    let mut b = Bench::new();
    // The supervisor sees the pads as off, so no permit comes and the main
    // controller is left waiting in Delivering.
    b.board.sup_pad_ohms = Some(3_000);
    reach(&mut b, S::Delivering, 60_000);
    b.board.cancel = true;
    reach(&mut b, S::Cpr, 5_000);
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.board.request.is_none());
    assert!(b.has_event(|e| *e == Event::Shock(ShockOutcome::Withdrawn(AbortReason::Cancelled))));
}

#[test]
fn cancel_during_self_test_empties_and_faults_with_continue_cpr() {
    let mut b = Bench::new();
    // Into the charge step of the self-test.
    assert!(b.run_until(5_000, |b| b.board.cap_v > 200.0));
    b.board.cancel = true;
    reach(&mut b, S::Fault, 5_000);
    assert_eq!(
        b.rescue.fault(),
        Some(FaultKind::SelfTest(SelfTestFailure::Cancelled))
    );
    assert_eq!(
        b.board.prompts_only().last(),
        Some(&Prompt::DeviceFaultContinueCpr)
    );
    assert_capacitor_emptied(&b);
}

#[test]
fn cancel_works_in_every_state() {
    // Press cancel on entry to each state of a shockable rescue. In every case
    // there must be no shock in that cycle and the capacitor must end empty.
    let states = [
        S::SelfTest,
        S::AttachPads,
        S::Analysing,
        S::Charging,
        S::Countdown,
        S::Delivering,
        S::Disarming,
        S::Cpr,
    ];
    for target in states {
        let mut b = Bench::new();
        if target == S::Delivering {
            b.board.sup_pad_ohms = Some(3_000); // hold it in Delivering
        }
        if target == S::AttachPads {
            b.board.pad_ohms = 5_000;
        }
        if target != S::SelfTest {
            reach(&mut b, target, 200_000);
        }
        let shocks_before = b.shocks_fired();
        b.board.cancel = true;
        b.run_ms(10_000);
        assert_eq!(
            b.shocks_fired(),
            shocks_before,
            "shock after cancel in {target:?}"
        );
        assert!(
            b.board.cap_v < 50.0,
            "capacitor not empty after cancel in {target:?}"
        );
        assert!(b.has_event(|e| *e == Event::CancelPressed), "{target:?}");
        b.assert_no_violations();
    }
}

// ----- patient mode ---------------------------------------------------------------

#[test]
fn patient_mode_change_during_countdown_aborts() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.run_ms(1_000);
    b.board.mode = PatientMode::Child;
    reach(&mut b, S::Cpr, 5_000);
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::ModeChanged)));
    assert!(b.has_event(|e| *e == Event::PatientModeChanged(PatientMode::Child)));
    assert!(b.board.prompts_only().contains(&Prompt::ChildModeFrontBack));
    assert_capacitor_emptied(&b);
}

#[test]
fn child_mode_charges_to_child_voltage_and_announces_front_back_placement() {
    let mut b = Bench::new();
    b.board.mode = PatientMode::Child;
    reach(&mut b, S::Cpr, 60_000);
    assert_eq!(b.shocks_fired(), 1);
    assert_eq!(
        b.board.requests[0].plan.charge_voltage,
        aed_types::Volts(1000)
    );
    let fired_at = b.board.fires[0].voltage;
    assert!(
        (950.0..=1050.0).contains(&fired_at),
        "fired at {fired_at} V"
    );
    assert!(b.board.prompts_only().contains(&Prompt::ChildModeFrontBack));
}

// ----- CPR -------------------------------------------------------------------------

#[test]
fn cpr_metronome_runs_at_configured_rate_and_cpr_lasts_its_period() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    let start = b.board.now;
    b.run_until_state(S::Analysing, 130_000);
    let beats: Vec<u32> = b
        .board
        .beats
        .iter()
        .copied()
        .filter(|&t| t >= start)
        .collect();
    let interval = Config::default().metronome_interval_ms(); // 545 ms at 110/min
    for w in beats.windows(2) {
        let gap = w[1] - w[0];
        assert!(gap.abs_diff(interval) <= 2, "beat gap {gap} ms");
    }
    // 120 s at 110 per minute is 220 beats.
    assert!((219..=221).contains(&beats.len()), "{} beats", beats.len());
    assert_eq!(b.board.now - start, 120_000);
}

#[test]
fn metronome_is_silent_outside_cpr_and_fault() {
    let mut b = Bench::new();
    reach(&mut b, S::Cpr, 60_000);
    let first_cpr = b.board.now;
    assert!(b.board.beats.iter().all(|&t| t >= first_cpr));
}

// ----- faults ----------------------------------------------------------------------

/// A scenario that should end in a fault, and the fault it should report.
struct FaultCase {
    name: &'static str,
    setup: fn(&mut Bench),
    expect: FaultKind,
}

fn fault_cases() -> Vec<FaultCase> {
    vec![
        FaultCase {
            name: "self-test battery under load",
            setup: |b| b.board.battery_under_load = BatteryStatus::Depleted,
            expect: FaultKind::SelfTest(SelfTestFailure::BatteryUnderLoad),
        },
        FaultCase {
            name: "self-test charger broken",
            setup: |b| b.board.charger_broken = true,
            expect: FaultKind::SelfTest(SelfTestFailure::ChargeTimeout),
        },
        FaultCase {
            name: "supervisor never answers",
            setup: |b| b.sup_alive = false,
            expect: FaultKind::SelfTest(SelfTestFailure::SupervisorLinkDead),
        },
        FaultCase {
            name: "charger fails during the rescue",
            setup: |b| {
                b.run_to_analysing();
                b.board.charger_broken = true;
            },
            expect: FaultKind::ChargeTimeout,
        },
        FaultCase {
            name: "charger runs away during the rescue",
            setup: |b| {
                b.run_to_analysing();
                b.board.charger_runaway = true;
            },
            // It overshoots the planned voltage before reaching the
            // overvoltage limit, so the drift check catches it first.
            expect: FaultKind::ChargeOffTarget,
        },
        FaultCase {
            name: "output stage fault",
            setup: |b| b.board.output_fault = true,
            expect: FaultKind::DeliveryFault,
        },
        FaultCase {
            name: "battery depleted mid-rescue",
            setup: |b| {
                b.run_to_analysing();
                b.board.battery = BatteryStatus::Depleted;
            },
            expect: FaultKind::BatteryDepleted,
        },
        FaultCase {
            name: "supervisor reports stop",
            setup: |b| {
                b.run_to_analysing();
                b.board.report_wire_up = false;
            },
            expect: FaultKind::SupervisorLinkLost,
        },
        FaultCase {
            name: "status messages to the supervisor stop",
            setup: |b| {
                b.run_to_analysing();
                b.board.status_wire_up = false;
            },
            expect: FaultKind::Supervisor(SupervisorFault::HeartbeatLost),
        },
        FaultCase {
            name: "the two voltage readings disagree while charged",
            setup: |b| {
                b.run_to_countdown();
                b.board.sup_v_offset = 300;
            },
            expect: FaultKind::Supervisor(SupervisorFault::VoltageDisagreement),
        },
    ]
}

#[test]
fn fault_always_leads_to_continue_cpr_prompt() {
    for case in fault_cases() {
        let mut b = Bench::new();
        b.board.rhythm = SHOCKABLE;
        (case.setup)(&mut b);
        assert!(
            b.run_until_state(S::Fault, 120_000),
            "{}: never faulted; states {:?}",
            case.name,
            b.state_sequence()
        );
        b.run_ms(1_000);
        assert_eq!(b.rescue.fault(), Some(case.expect), "{}", case.name);
        assert!(
            b.board
                .prompts_only()
                .contains(&Prompt::DeviceFaultContinueCpr),
            "{}: no continue-CPR prompt",
            case.name
        );
        assert!(!b.board.ready, "{}: ready light still on", case.name);
        // Charger off, hold released, request withdrawn, capacitor empty.
        assert!(b.board.charger.is_none(), "{}", case.name);
        assert!(!b.board.main_hold, "{}", case.name);
        assert!(b.board.request.is_none(), "{}", case.name);
        assert!(b.board.cap_v < 50.0, "{}: {} V", case.name, b.board.cap_v);
        b.assert_no_violations();
    }
}

#[test]
fn overvoltage_while_armed_is_a_fault() {
    let mut b = Bench::new();
    b.run_to_analysing();
    b.board.charger_runaway = true;
    b.board.charge_rate = 1_000.0; // jumps past the limit within two ticks
    reach(&mut b, S::Fault, 30_000);
    assert_eq!(b.rescue.fault(), Some(FaultKind::Overvoltage));
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.board.charger.is_none());
    assert!(!b.board.main_hold);
}

#[test]
fn fault_state_keeps_metronome_and_reminds_to_continue_cpr() {
    let mut b = Bench::new();
    b.board.battery_under_load = BatteryStatus::Depleted;
    reach(&mut b, S::Fault, 10_000);
    let beats_before = b.board.beats.len();
    b.run_ms(61_000);
    assert!(b.board.beats.len() - beats_before >= 100);
    let reminders = b
        .board
        .prompts_only()
        .iter()
        .filter(|p| **p == Prompt::DeviceFaultContinueCpr)
        .count();
    assert!(reminders >= 3, "only {reminders} reminders");
    // Fault is final: nothing ever charges again.
    b.board.rhythm = SHOCKABLE;
    b.run_ms(60_000);
    assert_eq!(b.state(), S::Fault);
    assert!(b.board.requests.is_empty());
}

#[test]
fn capacitor_not_emptied_is_a_serious_fault_with_its_own_prompt() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.board.dump_broken = true;
    b.board.cancel = true;
    reach(&mut b, S::Fault, 10_000);
    assert_eq!(b.rescue.fault(), Some(FaultKind::CapacitorNotEmptied));
    assert!(b.has_event(|e| *e == Event::Fault(FaultKind::CapacitorNotEmptied)));
    assert!(b
        .board
        .prompts_only()
        .contains(&Prompt::SeriousFaultContinueCpr));
    assert!(!b.board.main_hold);
    assert!(b.board.charger.is_none());
    assert_eq!(b.shocks_fired(), 0);
}

#[test]
fn disarming_never_lasts_longer_than_its_limit() {
    let mut b = Bench::new();
    b.run_to_countdown();
    b.board.dump_broken = true;
    b.board.cancel = true;
    reach(&mut b, S::Disarming, 1_000);
    let start = b.board.now;
    reach(&mut b, S::Fault, 10_000);
    let took = b.board.now - start;
    assert!(
        took <= Config::default().disarm_time_limit_ms + 4,
        "{took} ms"
    );
}

#[test]
fn self_test_discharge_failure_is_the_serious_fault() {
    let mut b = Bench::new();
    b.board.dump_broken = true;
    reach(&mut b, S::Fault, 30_000);
    assert_eq!(b.rescue.fault(), Some(FaultKind::CapacitorNotEmptied));
    assert_eq!(
        b.rescue.self_test_result(),
        Some(Err(SelfTestFailure::DischargeTimeout))
    );
    assert!(b
        .board
        .prompts_only()
        .contains(&Prompt::SeriousFaultContinueCpr));
}

#[test]
fn supervisor_latched_fault_puts_main_in_fault() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::Cpr, 60_000);
    b.board.status_wire_up = false;
    reach(&mut b, S::Fault, 5_000);
    assert_eq!(
        b.rescue.fault(),
        Some(FaultKind::Supervisor(SupervisorFault::HeartbeatLost))
    );
    assert!(b.has_event(|e| *e == Event::SupervisorFaultLatched(SupervisorFault::HeartbeatLost)));
}

#[test]
fn charging_never_lasts_longer_than_its_limit() {
    let mut b = Bench::new();
    b.run_to_analysing();
    b.board.charger_broken = true;
    reach(&mut b, S::Charging, 10_000);
    let start = b.board.now;
    reach(&mut b, S::Disarming, 20_000);
    let took = b.board.now - start;
    assert!(
        took <= Config::default().charge_time_limit_ms + 4,
        "charging lasted {took} ms"
    );
    assert_eq!(b.rescue.fault(), Some(FaultKind::ChargeTimeout));
}

// ----- the second check, seen from the main controller ----------------------------

#[test]
fn main_alone_cannot_fire() {
    // Everything on the main side says shock, but the supervisor's own pad
    // reading is out of range, so it gives no permit.
    let mut b = Bench::new();
    b.board.sup_pad_ohms = Some(3_000);
    reach(&mut b, S::Cpr, 60_000);
    assert_eq!(b.board.requests.len(), 1, "main did request");
    assert_eq!(b.shocks_fired(), 0, "but nothing fired");
    assert!(b.has_event(|e| *e == Event::Shock(ShockOutcome::TimedOut)));
    assert!(b.board.prompts_only().contains(&Prompt::ShockNotDelivered));
    assert_capacitor_emptied(&b);
}

#[test]
fn main_without_a_supervisor_cannot_even_charge() {
    let mut b = Bench::new();
    b.sup_alive = false;
    b.run_ms(60_000);
    assert_eq!(b.state(), S::Fault);
    assert!(b.board.cap_v < 1.0);
    assert_eq!(b.shocks_fired(), 0);
}

#[test]
fn stuck_main_in_countdown_is_emptied_by_supervisor() {
    // The main controller freezes mid-countdown with its outputs still
    // asserted (hold on, charger on). The supervisor notices the heartbeat
    // stop and empties the capacitor on its own.
    let mut b = Bench::new();
    b.run_to_countdown();
    b.main_alive = false;
    b.run_ms(1_000);
    assert!(b.board.main_hold, "main froze with its hold asserted");
    assert!(!b.board.sup_hold);
    assert!(b.board.cap_v < 50.0, "{} V", b.board.cap_v);
    assert_eq!(b.sup.fault(), Some(SupervisorFault::HeartbeatLost));
    assert_eq!(b.shocks_fired(), 0);
}

// ----- other rules -----------------------------------------------------------------

#[test]
fn low_battery_is_announced_once_and_rescue_continues() {
    let mut b = Bench::new();
    b.board.battery = BatteryStatus::Low;
    reach(&mut b, S::Cpr, 60_000);
    let n = b
        .board
        .prompts_only()
        .iter()
        .filter(|p| **p == Prompt::LowBattery)
        .count();
    assert_eq!(n, 1);
    assert_eq!(b.shocks_fired(), 1);
}

#[test]
fn expired_pads_warn_but_the_rescue_continues() {
    let mut b = Bench::new();
    b.board.cartridge = PadCartridge::Expired;
    reach(&mut b, S::Cpr, 60_000);
    assert!(b.board.prompts_only().contains(&Prompt::ReplacePads));
    assert_eq!(
        b.rescue.self_test_result(),
        Some(Err(SelfTestFailure::PadsExpired))
    );
    assert_eq!(b.shocks_fired(), 1);
}

#[test]
fn ready_indicator_on_after_self_test_and_off_in_fault() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    reach(&mut b, S::AttachPads, 30_000);
    assert!(b.board.ready);
    b.board.report_wire_up = false;
    reach(&mut b, S::Fault, 5_000);
    assert!(!b.board.ready);
}

#[test]
fn status_heartbeat_is_sent_every_period_with_rising_sequence() {
    let mut b = Bench::new();
    b.board.rhythm = NON;
    b.run_ms(10_000);
    let s = &b.board.statuses;
    assert!(s.len() >= 10_000 / 50);
    for w in s.windows(2) {
        assert_eq!(w[1].sequence, w[0].sequence.wrapping_add(1));
    }
}

#[test]
fn counters_and_elapsed_time_are_kept() {
    let mut b = Bench::new();
    reach(&mut b, S::Cpr, 60_000);
    let now = aed_types::Millis(b.board.now);
    assert_eq!(b.rescue.elapsed_ms(now), b.board.now - 2);
    assert_eq!(b.rescue.time_in_state_ms(now), 0);
    let c = b.rescue.counters();
    assert_eq!((c.shocks_delivered, c.analyses, c.cpr_periods), (1, 1, 1));
    b.run_ms(10_000);
    assert!(b.rescue.counters().metronome_beats >= 18);
}
