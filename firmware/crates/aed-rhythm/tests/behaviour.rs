// SPDX-License-Identifier: GPL-3.0-or-later
//! Behaviour of the analyser on SYNTHETIC rhythms.
//!
//! These tests show the code behaves sensibly on cartoon signals. They are
//! not evidence of clinical accuracy. See the crate README.

mod common;

use aed_rhythm::settings::{CONFIRM_WINDOWS, WINDOW_SECONDS};
use aed_rhythm::synth::{Rhythm, Synth};
use aed_rhythm::{reason_label, Analyzer, Reason};
use aed_types::{RhythmAnalyzer, RhythmDecision};
use common::{ever_shockable, feed, final_decision, run};

/// Length of one simulated analysis, seconds (a typical AED analysis pause is
/// around 5-10 s; UNVERIFIED, varies by device).
const ANALYSIS_SECONDS: usize = 10;

const SEEDS: [u32; 5] = [1, 2, 3, 42, 9001];

#[derive(Default)]
struct Row {
    shock: u32,
    no_shock: u32,
    indeterminate: u32,
    ever_shock: u32,
}

impl Row {
    fn add(&mut self, reports: &[aed_rhythm::WindowReport]) {
        match final_decision(reports) {
            Some(RhythmDecision::Shockable) => self.shock += 1,
            Some(RhythmDecision::NonShockable) => self.no_shock += 1,
            _ => self.indeterminate += 1,
        }
        if ever_shockable(reports) {
            self.ever_shock += 1;
        }
    }
    fn total(&self) -> u32 {
        self.shock + self.no_shock + self.indeterminate
    }
}

fn sinus_cases() -> Vec<Synth> {
    let mut v = Vec::new();
    for bpm in (40..=180).step_by(10) {
        for amp in [300.0, 1000.0, 2500.0] {
            for seed in SEEDS {
                v.push(Synth::new(
                    Rhythm::Sinus { bpm: bpm as f32, amplitude_uv: amp },
                    seed,
                ));
            }
        }
    }
    v
}

fn coarse_vf_cases() -> Vec<Synth> {
    let mut v = Vec::new();
    for amp in [300.0, 500.0, 800.0, 1500.0, 3000.0] {
        for seed in 1..=10 {
            v.push(Synth::new(Rhythm::VentricularFibrillation { amplitude_uv: amp }, seed));
        }
    }
    v
}

fn vt_cases() -> Vec<Synth> {
    let mut v = Vec::new();
    for bpm in (150..=250).step_by(10) {
        for amp in [600.0, 1200.0, 2500.0] {
            v.push(Synth::new(
                Rhythm::VentricularTachycardia { bpm: bpm as f32, amplitude_uv: amp },
                bpm,
            ));
        }
    }
    v
}

fn asystole_cases() -> Vec<Synth> {
    let mut v: Vec<Synth> = (1..=20).map(|s| Synth::new(Rhythm::Asystole, s)).collect();
    v.extend(SEEDS.map(|s| Synth::new(Rhythm::Asystole, s).with_baseline_wander(300.0)));
    v.extend(SEEDS.map(|s| Synth::new(Rhythm::Asystole, s).with_mains(200.0, 50.0)));
    v
}

fn fine_vf_cases() -> Vec<Synth> {
    (1..=10).map(|s| Synth::new(Rhythm::fine_vf(), s)).collect()
}

fn noisy_cases() -> Vec<Synth> {
    let mut v = Vec::new();
    for seed in SEEDS {
        for rhythm in [Rhythm::sinus(60.0), Rhythm::sinus(150.0), Rhythm::Asystole] {
            v.push(Synth::new(rhythm, seed).with_noise(150.0));
            v.push(Synth::new(rhythm, seed).with_noise(400.0));
        }
    }
    v
}

fn saturated_cases() -> Vec<Synth> {
    let mut v = Vec::new();
    for seed in SEEDS {
        v.push(Synth::new(Rhythm::Sinus { bpm: 70.0, amplitude_uv: 60_000.0 }, seed));
        v.push(Synth::new(Rhythm::coarse_vf(), seed).with_offset(40_000.0));
        v.push(Synth::new(Rhythm::VentricularFibrillation { amplitude_uv: 90_000.0 }, seed));
    }
    v
}

fn compression_cases() -> Vec<Synth> {
    let mut v = Vec::new();
    for seed in SEEDS {
        for bpm in [50.0, 80.0, 120.0] {
            for (amp, rate) in [(800.0, 100.0), (1500.0, 110.0), (3000.0, 120.0)] {
                v.push(Synth::new(Rhythm::sinus(bpm), seed).with_compressions(amp, rate));
            }
        }
    }
    v
}

fn s_rhythm_kind(label: &str) -> &str {
    label.split([' ', '{']).next().unwrap_or("")
}

fn tally(cases: Vec<Synth>) -> (Row, Vec<Vec<aed_rhythm::WindowReport>>) {
    tally_noting(cases, false)
}

/// Run every case; if `note_misses`, print shock-class cases that did not end Shockable.
fn tally_noting(cases: Vec<Synth>, note_misses: bool) -> (Row, Vec<Vec<aed_rhythm::WindowReport>>) {
    let mut row = Row::default();
    let mut all = Vec::new();
    for s in cases {
        let label = format!("{:?}", s.rhythm());
        let r = run(s, ANALYSIS_SECONDS);
        if let Some(last) = r.last() {
            let shock_class = matches!(
                s_rhythm_kind(&label),
                "VentricularFibrillation" | "VentricularTachycardia"
            );
            if note_misses && shock_class && last.decision != RhythmDecision::Shockable {
                println!(
                    "  note: {label} ended {:?} ({}), features {:?}",
                    last.decision,
                    reason_label(last.verdict.reason),
                    last.features
                );
            }
        }
        row.add(&r);
        all.push(r);
    }
    (row, all)
}

#[test]
fn synthetic_confusion_table() {
    let classes: Vec<(&str, Vec<Synth>, &str)> = vec![
        ("Sinus 40-180/min", sinus_cases(), "no shock"),
        ("Coarse VF", coarse_vf_cases(), "shock"),
        ("Rapid VT 150-250/min", vt_cases(), "shock"),
        ("Asystole", asystole_cases(), "no shock"),
        ("Fine VF (<200 uV)", fine_vf_cases(), "no shock (by design)"),
        ("Noisy (sinus/asystole)", noisy_cases(), "no shock"),
        ("Saturated / clipped", saturated_cases(), "no shock"),
        ("CPR artifact on sinus", compression_cases(), "no shock"),
    ];
    println!();
    println!("SYNTHETIC SIGNALS ONLY - final decision after {ANALYSIS_SECONDS} s; says nothing about clinical accuracy");
    println!(
        "{:<24} {:>5} {:>9} {:>10} {:>14} {:>11}  expected",
        "class", "runs", "Shockable", "NonShock", "Indeterminate", "ever-shock"
    );
    let mut rows = Vec::new();
    for (name, cases, expect) in classes {
        let note = name.starts_with("Coarse VF") || name.starts_with("Rapid VT");
        let (row, _) = tally_noting(cases, note);
        println!(
            "{:<24} {:>5} {:>9} {:>10} {:>14} {:>11}  {expect}",
            name,
            row.total(),
            row.shock,
            row.no_shock,
            row.indeterminate,
            row.ever_shock
        );
        rows.push((name, row));
    }
    for (name, row) in &rows {
        let should_shock = name.starts_with("Coarse VF") || name.starts_with("Rapid VT");
        if should_shock {
            // Every case must reach Shockable at some point, and at least 90 %
            // must end on it. The 90 % mirrors the AHA coarse-VF goal only in
            // shape; on synthetic data it is a smoke test, not a claim.
            assert_eq!(row.ever_shock, row.total(), "{name}: a case never became shockable");
            assert!(row.shock * 10 >= row.total() * 9, "{name}: under 90 % ended shockable");
        } else {
            assert_eq!(row.ever_shock, 0, "{name}: a Shockable decision appeared");
        }
    }
}

#[test]
fn sinus_is_never_shockable_and_usually_non_shockable() {
    let (row, all) = tally(sinus_cases());
    assert_eq!(row.ever_shock, 0);
    // Clean synthetic sinus should come out NonShockable in every window.
    for reports in &all {
        for r in reports {
            assert_eq!(
                r.decision,
                RhythmDecision::NonShockable,
                "sinus window {} gave {:?} ({})",
                r.window_number,
                r.decision,
                reason_label(r.verdict.reason)
            );
        }
    }
}

#[test]
fn asystole_is_non_shockable() {
    for s in asystole_cases() {
        for r in run(s, ANALYSIS_SECONDS) {
            assert_eq!(r.decision, RhythmDecision::NonShockable);
            assert_eq!(r.verdict.reason, Reason::Asystole);
        }
    }
}

#[test]
fn coarse_vf_becomes_shockable_within_bounded_time() {
    // Warm-up 1 s + one full window + (CONFIRM_WINDOWS - 1) more steps.
    let limit = 1 + WINDOW_SECONDS + usize::from(CONFIRM_WINDOWS) - 1;
    for seed in 1..=10 {
        let mut a = Analyzer::new();
        let mut s = Synth::new(Rhythm::coarse_vf(), seed);
        let mut first = None;
        for n in 0..(ANALYSIS_SECONDS * 250) {
            if a.push(s.next_sample()) == Some(RhythmDecision::Shockable) {
                first = Some(n / 250 + 1);
                break;
            }
        }
        let first = first.unwrap_or(usize::MAX);
        assert!(first <= limit, "seed {seed}: first shockable at {first} s, limit {limit} s");
    }
}

#[test]
fn vf_with_moderate_interference_still_shockable() {
    for seed in SEEDS {
        let s = Synth::new(Rhythm::coarse_vf(), seed)
            .with_noise(20.0)
            .with_baseline_wander(400.0)
            .with_mains(300.0, 60.0)
            .with_offset(-5_000.0);
        assert_eq!(final_decision(&run(s, ANALYSIS_SECONDS)), Some(RhythmDecision::Shockable));
    }
}

#[test]
fn heavy_noise_and_saturation_never_shock() {
    for s in noisy_cases().into_iter().chain(saturated_cases()) {
        assert!(!ever_shockable(&run(s, ANALYSIS_SECONDS)));
    }
    for s in saturated_cases() {
        assert_eq!(final_decision(&run(s, ANALYSIS_SECONDS)), Some(RhythmDecision::Indeterminate));
    }
}

#[test]
fn compressions_over_normal_rhythm_never_shock() {
    for s in compression_cases() {
        let r = run(s, ANALYSIS_SECONDS);
        assert!(!ever_shockable(&r));
        assert!(matches!(
            final_decision(&r),
            Some(RhythmDecision::Indeterminate | RhythmDecision::NonShockable)
        ));
    }
}

#[test]
fn vf_to_sinus_stops_shockable_quickly() {
    for seed in SEEDS {
        let mut a = Analyzer::new();
        let mut s = Synth::new(Rhythm::coarse_vf(), seed);
        let before = feed(&mut a, &mut s, 12);
        assert_eq!(final_decision(&before), Some(RhythmDecision::Shockable));
        s.set_rhythm(Rhythm::sinus(80.0));
        let after = feed(&mut a, &mut s, 10);
        // Allowed: at most WINDOW_SECONDS decisions still Shockable (windows
        // that still contain mostly VF); in practice the mixed window breaks
        // agreement much sooner.
        let still = after
            .iter()
            .take_while(|r| r.decision == RhythmDecision::Shockable)
            .count();
        assert!(still <= WINDOW_SECONDS, "seed {seed}: {still} shockable windows after switch");
        assert!(
            after
                .iter()
                .skip(WINDOW_SECONDS)
                .all(|r| r.decision != RhythmDecision::Shockable),
            "seed {seed}: shockable after the window cleared"
        );
        println!("seed {seed}: {still} shockable decision(s) after VF ended");
    }
}

#[test]
fn reset_clears_state() {
    let mut a = Analyzer::new();
    let mut vf = Synth::new(Rhythm::coarse_vf(), 3);
    let r = feed(&mut a, &mut vf, 12);
    assert_eq!(final_decision(&r), Some(RhythmDecision::Shockable));

    a.reset();
    assert!(a.last_report().is_none());
    // No decision until warm-up plus one full window has passed again.
    let mut quiet = Synth::new(Rhythm::Asystole, 1);
    for _ in 0..(250 * (1 + WINDOW_SECONDS) - 1) {
        assert_eq!(a.push(quiet.next_sample()), None);
    }
    // The shockable streak must not survive the reset: fresh VF needs the
    // full confirmation count again.
    let mut a = Analyzer::new();
    feed(&mut a, &mut Synth::new(Rhythm::coarse_vf(), 3), 12);
    a.reset();
    let r = feed(&mut a, &mut Synth::new(Rhythm::coarse_vf(), 3), 1 + WINDOW_SECONDS);
    assert_eq!(r.len(), 1);
    assert_eq!(r.first().map(|x| x.window_number), Some(1));
    assert_eq!(r.first().map(|x| x.consecutive_shockable), Some(1));
    assert_eq!(final_decision(&r), Some(RhythmDecision::Indeterminate));
}

#[test]
fn one_decision_per_second_after_first_window() {
    let r = run(Synth::new(Rhythm::sinus(70.0), 1), 20);
    // 20 s minus 1 s warm-up minus first 4 s window, plus that first window.
    assert_eq!(r.len(), 20 - 1 - WINDOW_SECONDS + 1);
}

/// The rescue sequence treats any announced `Indeterminate` as "go to CPR",
/// so an unconfirmed shockable window must be "still analysing" (`None`),
/// and the first thing announced for clean coarse VF must be `Shockable`.
#[test]
fn unconfirmed_shockable_is_not_announced() {
    for seed in 1..=10 {
        let mut a = Analyzer::new();
        let mut vf = Synth::new(Rhythm::coarse_vf(), seed);
        let mut first = None;
        for _ in 0..(250 * 12) {
            if let Some(d) = a.push(vf.next_sample()) {
                first = Some(d);
                break;
            }
        }
        assert_eq!(first, Some(RhythmDecision::Shockable), "seed {seed}");
    }
}
