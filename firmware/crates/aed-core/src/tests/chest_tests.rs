// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for compression sensing through the pads (decision D-36; finding
//! F-12) in the rescue sequence. Run on the mock board with the real
//! supervisor alongside; the chest channel carries a scripted state that a
//! fake sensor reports. Every name states the rule it checks.

use super::mock::{BoardChest, Bench};
use crate::prompts::Prompt;
use crate::rescue::{AbortReason, RescueState};
use crate::{Config, ConfigError, Event};
use aed_types::{ChestState, ImpedanceSignal, Millis, RhythmDecision};
use std::vec::Vec;

use RescueState as S;

fn reach(b: &mut Bench, state: RescueState, max_ms: u32) {
    assert!(
        b.run_until_state(state, max_ms),
        "never reached {state:?}; states {:?}",
        b.state_sequence()
    );
}

fn prompts_between(b: &Bench, from: u32, to: u32) -> Vec<Prompt> {
    b.board
        .prompts
        .iter()
        .filter(|&&(at, _)| at >= from && at < to)
        .map(|&(_, p)| p)
        .collect()
}

fn count_events(b: &Bench, pred: impl Fn(&Event) -> bool) -> usize {
    b.rescue.events().iter().filter(|e| pred(&e.event)).count()
}

fn chest_bench() -> Bench {
    Bench::with_chest(Config::default())
}

fn now(b: &Bench) -> Millis {
    Millis(b.board.now)
}

// ----- analysis gating ---------------------------------------------------------------

#[test]
fn compressions_throughout_the_analysis_never_lead_to_a_countdown_even_on_a_shockable_signal() {
    // Finding F-12 at the level of the rescue sequence: the heart signal says
    // "shockable" (as compressions at 150/min over a flatline did), but the
    // chest is being compressed the whole time. The analysis is never fed,
    // so it ends inconclusive at its limit: no countdown, no shock.
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::Shockable;
    b.board.chest = BoardChest::Compressing;
    b.run_to_analysing();
    let start = b.board.now;
    reach(&mut b, S::Cpr, 20_000);
    assert!(!b.state_sequence().contains(&S::Countdown));
    assert!(b.has_event(|e| *e == Event::AnalysisInconclusive || *e == Event::AnalysisTimedOut));
    let took = b.board.now - start;
    assert!((12_000..=12_100).contains(&took), "the time limit still holds: {took} ms");
    assert_eq!(b.shocks_fired(), 0);
    assert!(b.board.requests.is_empty());
    b.assert_no_violations();
}

#[test]
fn unknown_chest_state_holds_the_analysis_like_compressions() {
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::Shockable;
    b.board.chest = BoardChest::Unknown;
    b.run_to_analysing();
    reach(&mut b, S::Cpr, 20_000);
    assert!(!b.state_sequence().contains(&S::Countdown));
    assert_eq!(b.shocks_fired(), 0);
    b.assert_no_violations();
}

#[test]
fn the_analysis_is_reset_when_the_chest_stops_being_still_and_restarts_clean_when_hands_come_off() {
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::Indeterminate;
    b.run_to_analysing();
    // 2.5 s of a still chest ("not sure"), then compressions for 2 s.
    b.run_ms(2_500);
    let resets_before = b.resets.get();
    b.board.chest = BoardChest::Compressing;
    b.run_ms(20);
    assert_eq!(b.resets.get(), resets_before + 1, "analysis reset at once");
    assert_eq!(b.rescue.counters().analyses_held, 1);
    assert!(b.has_event(|e| *e == Event::AnalysisHeld(ChestState::Compressing)));
    b.run_ms(2_000);
    assert!(b.rescue.last_decision().is_none(), "nothing fed while compressing");
    b.board.rhythm = RhythmDecision::Shockable;
    b.board.chest = BoardChest::HandsOff;
    let off_at = b.board.now;
    reach(&mut b, S::Countdown, 15_000);
    // The fake analyser needs 1 s of samples for its first answer, counted
    // from when the chest was still again.
    assert!(b.board.now - off_at >= 1_000);
    b.assert_no_violations();
}

#[test]
fn do_not_touch_reminder_follows_the_chest_state_not_the_not_sure_answers() {
    // With compression sensing: "not sure" from the heart signal on a still
    // chest does not trigger "Do not touch the patient"...
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::Indeterminate;
    b.run_to_analysing();
    let start = b.board.now;
    reach(&mut b, S::Cpr, 20_000);
    assert!(!prompts_between(&b, start, b.board.now).contains(&Prompt::DoNotTouchPatient));
    assert!(b.has_event(|e| *e == Event::AnalysisInconclusive));
    // ...but a chest that is not still does, at most every 3 s, starting 3 s
    // after the hands-off instruction that opened the analysis.
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::Shockable;
    b.board.chest = BoardChest::Compressing;
    b.run_to_analysing();
    let start = b.board.now;
    reach(&mut b, S::Cpr, 20_000);
    let times: Vec<u32> = b
        .board
        .prompts
        .iter()
        .filter(|&&(at, p)| at >= start && p == Prompt::DoNotTouchPatient)
        .map(|&(at, _)| at - start)
        .collect();
    assert!(times.len() >= 3, "reminders every 3 s: {times:?}");
    assert!(times[0] >= 3_000);
    assert!(times.windows(2).all(|w| w[1] - w[0] >= 3_000));
}

#[test]
fn without_compression_sensing_not_sure_still_drives_the_reminder() {
    // The earlier behaviour, unchanged, when sensing is switched off.
    let config = Config {
        chest_sensing: false,
        ..Config::default()
    };
    let mut b = Bench::with_chest(config);
    b.board.rhythm = RhythmDecision::Indeterminate;
    b.board.chest = BoardChest::Compressing;
    b.run_to_analysing();
    let start = b.board.now;
    reach(&mut b, S::Cpr, 20_000);
    assert!(prompts_between(&b, start, b.board.now).contains(&Prompt::DoNotTouchPatient));
}

#[test]
fn chest_sensing_switched_off_falls_back_to_heart_signal_only() {
    // The F-12 situation with sensing off: the heart signal says shockable
    // while the chest is compressed, and the device counts down (the old
    // behaviour, kept so the effect is measurable).
    let config = Config {
        chest_sensing: false,
        ..Config::default()
    };
    let mut b = Bench::with_chest(config);
    b.board.rhythm = RhythmDecision::Shockable;
    b.board.chest = BoardChest::Compressing;
    reach(&mut b, S::Countdown, 30_000);
    assert!(!b.rescue.chest_sensing_active());
    assert_eq!(b.rescue.chest_state(now(&b)), None);
}

#[test]
fn a_device_without_compression_sensing_reports_no_chest_state() {
    let b = Bench::new();
    assert!(!b.rescue.chest_sensing_active());
    assert_eq!(b.rescue.chest_state(Millis(0)), None);
}

// ----- after "shock advised" ---------------------------------------------------------

#[test]
fn compressions_during_the_countdown_hold_the_shock_back_and_analyse_again_once() {
    let mut b = chest_bench();
    reach(&mut b, S::Countdown, 30_000);
    let analyses = b.rescue.counters().analyses;
    b.run_ms(1_000);
    let t = b.board.now;
    b.board.chest = BoardChest::Compressing;
    b.run_ms(20);
    assert!(b.state().is_analysing(), "back to analysing: {:?}", b.state());
    assert!(b.board.request.is_none());
    assert_eq!(b.rescue.counters().hands_on_aborts, 1);
    assert_eq!(b.rescue.counters().analyses, analyses + 1);
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::HandsOnChest)));
    let said = prompts_between(&b, t + 1, b.board.now + 1);
    assert_eq!(said, [Prompt::MovementShockDelayed, Prompt::AnalysingDoNotTouch]);
    assert!(b.rescue.emptying() || b.board.cap_v < 50.0, "the charge is emptied first");
    // Hands off again: the new analysis advises the shock, which is given.
    b.run_ms(1_500);
    b.board.chest = BoardChest::HandsOff;
    reach(&mut b, S::Cpr, 30_000);
    assert_eq!(b.shocks_fired(), 1);
    b.assert_no_violations();
}

#[test]
fn a_second_hands_on_abort_in_the_same_cycle_goes_to_cpr_without_a_shock() {
    let mut b = chest_bench();
    reach(&mut b, S::Countdown, 30_000);
    b.board.chest = BoardChest::Compressing;
    b.run_ms(20);
    b.board.chest = BoardChest::HandsOff;
    reach(&mut b, S::Countdown, 30_000);
    let t = b.board.now + 1;
    b.board.chest = BoardChest::Unknown;
    reach(&mut b, S::Cpr, 1_000);
    assert_eq!(b.shocks_fired(), 0);
    let said = prompts_between(&b, t, b.board.now + 1);
    assert_eq!(said, [Prompt::MovementNoShock, Prompt::StartCpr]);
    assert_eq!(b.rescue.counters().hands_on_aborts, 2);
    b.assert_no_violations();
}

#[test]
fn unknown_chest_while_charging_after_the_advice_holds_the_shock_back() {
    // Charging only after the advice (D-29 switched off), so the Charging
    // state lasts the whole charge.
    let config = Config {
        charge_during_analysis: false,
        ..Config::default()
    };
    let mut b = Bench::with_chest(config);
    reach(&mut b, S::Charging, 30_000);
    b.run_ms(500);
    b.board.chest = BoardChest::Unknown;
    b.run_ms(20);
    assert!(b.state().is_analysing());
    assert!(b.has_event(|e| *e == Event::ShockAborted(AbortReason::HandsOnChest)));
    b.assert_no_violations();
}

#[test]
fn compressions_while_delivering_withdraw_the_request() {
    let mut b = chest_bench();
    reach(&mut b, S::Countdown, 30_000);
    // The supervisor's own pad reading is out of range, so it gives no
    // permit and the request stays pending.
    b.board.sup_pad_ohms = Some(200);
    reach(&mut b, S::Delivering, 5_000);
    b.board.chest = BoardChest::Compressing;
    b.run_ms(10);
    assert!(b.board.request.is_none());
    assert!(b.has_event(|e| *e
        == Event::Shock(crate::ShockOutcome::Withdrawn(AbortReason::HandsOnChest))));
    assert_eq!(b.shocks_fired(), 0);
}

#[test]
fn no_shock_is_ever_requested_while_the_chest_channel_is_not_still() {
    // A sweep: compressions or movement start at every 250 ms of a whole
    // shock cycle. The board flags any request made while its chest
    // channel is not "hands off" as a violation.
    // A request already raised in the same instant can still fire: the
    // mock's sensing latency is one sample plus one tick (10 ms).
    for start_ms in (0..14_000u32).step_by(250) {
        for kind in [BoardChest::Compressing, BoardChest::Unknown, BoardChest::Saturated] {
            let mut b = chest_bench();
            b.run_to_analysing();
            b.run_ms(start_ms);
            let t = b.board.now;
            b.board.chest = kind;
            b.run_ms(3_000);
            assert!(
                b.board.requests.iter().all(|r| r.at < t + 10),
                "shock requested after {kind:?} at {start_ms} ms"
            );
            assert!(
                b.board.fires.iter().all(|f| f.at < t + 10),
                "shock fired after {kind:?} at {start_ms} ms"
            );
            b.assert_no_violations();
        }
    }
}

// ----- a failed impedance channel ---------------------------------------------------

#[test]
fn missing_impedance_samples_hold_the_analysis_then_degrade_to_heart_signal_only_with_an_event() {
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::Shockable;
    b.run_to_analysing();
    b.board.impedance_up = false;
    b.run_ms(150);
    assert_eq!(b.rescue.chest_state(now(&b)), Some(ChestState::Unknown));
    assert_eq!(b.rescue.chest_signal(now(&b)), ImpedanceSignal::Missing);
    // Held until the channel counts as failed (5 s), then heart signal only.
    let t = b.board.now;
    reach(&mut b, S::Countdown, 20_000);
    assert!(b.board.now - t >= 4_800, "held while the channel might recover");
    assert!(b.has_event(|e| *e == Event::ChestSensingFailed(ImpedanceSignal::Missing)));
    assert!(!b.rescue.chest_sensing_active());
    reach(&mut b, S::Cpr, 10_000);
    assert_eq!(b.shocks_fired(), 1);
}

#[test]
fn a_saturated_or_out_of_range_channel_with_pads_on_also_degrades_and_a_good_channel_is_used_again() {
    for bad in [BoardChest::Saturated, BoardChest::OutOfRange] {
        let mut b = chest_bench();
        b.board.rhythm = RhythmDecision::NonShockable;
        b.run_to_analysing();
        b.board.chest = bad;
        b.run_ms(5_100);
        assert!(!b.rescue.chest_sensing_active(), "{bad:?}");
        b.board.chest = BoardChest::HandsOff;
        b.run_ms(3_100);
        assert!(b.rescue.chest_sensing_active(), "{bad:?}: used again");
        assert!(b.has_event(|e| *e == Event::ChestSensingRestored));
    }
}

// ----- CPR period ------------------------------------------------------------------------

#[test]
fn compressions_pauses_and_a_period_summary_are_recorded_during_cpr() {
    let config = Config {
        cpr_period_ms: 30_000,
        ..Config::default()
    };
    let mut b = Bench::with_chest(config);
    b.board.rhythm = RhythmDecision::NonShockable;
    reach(&mut b, S::Cpr, 30_000);
    b.board.chest = BoardChest::Compressing;
    b.run_ms(10_000);
    b.board.chest = BoardChest::HandsOff;
    b.run_ms(4_000);
    b.board.chest = BoardChest::Compressing;
    reach(&mut b, S::AnalysingCharging, 30_000);
    let c = b.rescue.counters();
    assert!(c.compressions >= 40, "{} compressions", c.compressions);
    assert_eq!(c.cpr_pauses, 1);
    assert_eq!(b.rescue.compression_log().len() as u32, c.compressions);
    let summary = b.rescue.events().iter().find_map(|e| match e.event {
        Event::CprPeriodSummary {
            compressions,
            rate_per_min,
            on_chest_permille,
            longest_pause_ms,
        } => Some((compressions, rate_per_min, on_chest_permille, longest_pause_ms)),
        _ => None,
    });
    let (n, rate, on_chest, longest) = summary.expect("a summary at the end of CPR");
    assert!(n >= 40);
    assert!((105..=115).contains(&rate), "rate {rate}");
    assert!((800..=900).contains(&on_chest), "on chest {on_chest}");
    // Measured between compression events (one every 545 ms here).
    assert!((4_000..=5_200).contains(&longest), "longest pause {longest}");
    assert!(count_events(&b, |e| matches!(e, Event::CprPause(ms) if *ms >= 4_000)) == 1);
}

#[test]
fn continue_cpr_is_said_after_the_configured_time_without_compressions_and_not_when_off() {
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::NonShockable;
    reach(&mut b, S::Cpr, 30_000);
    let start = b.board.now;
    b.board.chest = BoardChest::HandsOff;
    b.run_ms(40_000);
    let times: Vec<u32> = b
        .board
        .prompts
        .iter()
        .filter(|&&(at, p)| at >= start && p == Prompt::ContinueCpr)
        .map(|&(at, _)| at - start)
        .collect();
    assert_eq!(times.len(), 2, "{times:?}");
    assert!((15_000..=15_100).contains(&times[0]));
    assert!(times[1] - times[0] >= 15_000);
    assert_eq!(b.rescue.counters().continue_cpr_prompts, 2);

    // Compressing: nothing said.
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::NonShockable;
    reach(&mut b, S::Cpr, 30_000);
    let start = b.board.now;
    b.board.chest = BoardChest::Compressing;
    b.run_ms(40_000);
    assert!(!prompts_between(&b, start, b.board.now).contains(&Prompt::ContinueCpr));

    // Cannot tell: nothing said.
    let mut b = chest_bench();
    b.board.rhythm = RhythmDecision::NonShockable;
    reach(&mut b, S::Cpr, 30_000);
    let start = b.board.now;
    b.board.chest = BoardChest::Unknown;
    b.run_ms(40_000);
    assert!(!prompts_between(&b, start, b.board.now).contains(&Prompt::ContinueCpr));

    // Switched off.
    let config = Config {
        no_compression_prompt_ms: 0,
        ..Config::default()
    };
    let mut b = Bench::with_chest(config);
    b.board.rhythm = RhythmDecision::NonShockable;
    reach(&mut b, S::Cpr, 30_000);
    let start = b.board.now;
    b.run_ms(40_000);
    assert!(!prompts_between(&b, start, b.board.now).contains(&Prompt::ContinueCpr));
}

#[test]
fn chest_sensing_settings_that_contradict_each_other_are_refused() {
    let c = Config::default();
    assert!(c.chest_sensing);
    assert_eq!(c.no_compression_prompt_ms, 15_000);
    assert_eq!(c.hands_on_reanalyses, 1);
    for bad in [
        Config { impedance_stale_ms: 0, ..c },
        Config { chest_channel_fail_ms: 50, ..c },
        Config { chest_channel_recover_ms: 0, ..c },
        Config { no_compression_prompt_ms: 2_000, ..c },
        Config { pause_record_ms: 0, ..c },
    ] {
        assert_eq!(bad.validate(), Err(ConfigError::ChestSensingTimes));
    }
}
