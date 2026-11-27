// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared helpers for the integration tests.
#![allow(dead_code)]

use aed_rhythm::synth::Synth;
use aed_rhythm::{Analyzer, WindowReport};
use aed_types::{RhythmAnalyzer, RhythmDecision};

/// Feed `seconds` of `synth` into `analyzer`, returning every window report.
pub fn feed(analyzer: &mut Analyzer, synth: &mut Synth, seconds: usize) -> Vec<WindowReport> {
    // A window whose shockable verdict is still awaiting confirmation makes
    // `push` return `None`, so collect by window number, not by `push`.
    let mut out = Vec::new();
    let mut seen = analyzer.last_report().map_or(0, |r| r.window_number);
    for _ in 0..seconds * 250 {
        let announced = analyzer.push(synth.next_sample());
        if let Some(r) = analyzer.last_report() {
            if r.window_number != seen {
                seen = r.window_number;
                out.push(*r);
            }
            if let Some(d) = announced {
                assert_eq!(d, r.decision, "announced decision must match the report");
            }
        }
    }
    out
}

/// Fresh analyser, `seconds` of signal, all reports.
pub fn run(mut synth: Synth, seconds: usize) -> Vec<WindowReport> {
    let mut a = Analyzer::new();
    feed(&mut a, &mut synth, seconds)
}

/// The decision the device would act on after a fixed analysis period:
/// the last reported decision.
pub fn final_decision(reports: &[WindowReport]) -> Option<RhythmDecision> {
    reports.last().map(|r| r.decision)
}

/// True if any report in the list is `Shockable`.
pub fn ever_shockable(reports: &[WindowReport]) -> bool {
    reports.iter().any(|r| r.decision == RhythmDecision::Shockable)
}
