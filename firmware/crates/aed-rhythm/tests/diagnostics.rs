// SPDX-License-Identifier: GPL-3.0-or-later
//! Prints feature ranges per synthetic rhythm (run with `--nocapture`).
//! Used to see how far each class sits from each threshold.

mod common;

use aed_rhythm::reason_label;
use aed_rhythm::synth::{Rhythm, Synth};

fn show(name: &str, synth: Synth) {
    let reports = common::run(synth, 20);
    let f = |g: fn(&aed_rhythm::Features) -> f32| {
        let (lo, hi) = reports.iter().map(|r| g(&r.features)).fold(
            (f32::MAX, f32::MIN),
            |(lo, hi), v| (lo.min(v), hi.max(v)),
        );
        format!("{lo:.2}-{hi:.2}")
    };
    let mut reasons: Vec<&str> = reports.iter().map(|r| reason_label(r.verdict.reason)).collect();
    reasons.dedup();
    println!(
        "{name:<26} pp {:<15} tcsc {:<10} cnt {:<10} leak {:<10} df {:<11} conc {:<10} low {:<10} noise {:<10} {:?}",
        f(|x| x.peak_to_peak_uv),
        f(|x| x.threshold_crossing_fraction),
        f(|x| x.bandpass_count_fraction),
        f(|x| x.vf_leakage),
        f(|x| x.dominant_frequency_hz),
        f(|x| x.spectral_concentration),
        f(|x| x.low_band_fraction),
        f(|x| x.noise_ratio),
        reasons
    );
}

#[test]
fn print_feature_ranges() {
    for bpm in [40.0, 60.0, 90.0, 120.0, 150.0, 180.0] {
        show(&format!("sinus {bpm}"), Synth::new(Rhythm::sinus(bpm), 11));
    }
    for seed in [1, 2, 3] {
        show(&format!("coarse VF seed {seed}"), Synth::new(Rhythm::coarse_vf(), seed));
    }
    show("fine VF", Synth::new(Rhythm::fine_vf(), 1));
    for bpm in [150.0, 180.0, 220.0, 250.0] {
        show(&format!("VT {bpm}"), Synth::new(Rhythm::rapid_vt(bpm), 5));
    }
    show("asystole", Synth::new(Rhythm::Asystole, 1));
    show("sinus 70 + noise 100", Synth::new(Rhythm::sinus(70.0), 1).with_noise(100.0));
    show("sinus 70 + CPR", Synth::new(Rhythm::sinus(70.0), 1).with_compressions(1500.0, 110.0));
    show("asystole + CPR", Synth::new(Rhythm::Asystole, 1).with_compressions(1500.0, 110.0));
    show("VF + CPR", Synth::new(Rhythm::coarse_vf(), 1).with_compressions(1500.0, 110.0));
    show("sinus 70 + wander + mains", Synth::new(Rhythm::sinus(70.0), 1).with_baseline_wander(500.0).with_mains(300.0, 60.0));
}
