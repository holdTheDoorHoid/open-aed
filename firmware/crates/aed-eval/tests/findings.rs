// SPDX-License-Identifier: GPL-3.0-or-later
//! Behaviours of the analyser found while evaluating it on recorded data,
//! pinned down as tests so they are visible and can be revisited. They
//! assert the analyser's CURRENT behaviour; they are not claims that the
//! behaviour is right. Synthetic inputs only, so they run without the data.
#![allow(clippy::unwrap_used, clippy::indexing_slicing, missing_docs)]

use aed_rhythm::{Analyzer, Reason};
use aed_types::RhythmAnalyzer;

const FS: usize = 250;

fn last_reason(samples: &[i16]) -> (Reason, f32) {
    let mut a = Analyzer::new();
    for s in samples {
        let _ = a.push(*s);
    }
    let r = a.last_report().unwrap();
    (r.verdict.reason, r.features.saturated_fraction)
}

/// The saturation rule looks for samples beyond ±32 000 µV. A recorder that
/// clips at its own, smaller limit (the PhysioNet recorders clip at about ±5
/// mV) is invisible to it: a signal flattened at ±5 000 µV for long
/// stretches is reported with a saturated fraction of zero.
#[test]
fn recorder_clipping_below_32000_uv_is_not_seen_as_saturation() {
    let s: Vec<i16> = (0..8 * FS)
        .map(|i| {
            let v = 9000.0 * (2.0 * core::f64::consts::PI * 1.2 * i as f64 / FS as f64).sin();
            v.clamp(-5000.0, 5000.0) as i16
        })
        .collect();
    let (_, sat) = last_reason(&s);
    assert_eq!(sat, 0.0);
}

/// A flat line with a little 4 Hz ripple of 150 µV peak to peak (the size of
/// the baseline in vfdb's annotated asystole) is above the 100 µV flatline
/// limit, so the flatline rule does not fire; the window goes on to the VF
/// methods. On the real recordings the flatline rule fired for about 1 % of
/// annotated-asystole windows.
#[test]
fn small_ripple_defeats_the_flatline_rule() {
    let s: Vec<i16> = (0..8 * FS)
        .map(|i| (75.0 * (2.0 * core::f64::consts::PI * 4.0 * i as f64 / FS as f64).sin()) as i16)
        .collect();
    let (reason, _) = last_reason(&s);
    assert_ne!(reason, Reason::Asystole);
}
