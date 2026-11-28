// SPDX-License-Identifier: GPL-3.0-or-later
//! Decision D-47: starting a fresh analysis window on filters that are
//! already running, the moment compression sensing says the chest is still.
//!
//! SYNTHETIC SIGNALS ONLY. These tests measure the analyser's own filters
//! and its answers on made-up rhythms with made-up compression artifact.
//! They say nothing about real patients.
//!
//! What is checked:
//!
//! 1. How long the cleaning filters take, after a large compression
//!    artifact stops abruptly, before what is left of it falls below a
//!    stated level (`measure_filter_settling_after_compression_artifact_stops`).
//!    The filters are linear, so the leftover does not depend on the heart
//!    signal underneath: it is measured on the artifact alone.
//! 2. That with the device's guard, artifact ending just before the window
//!    never produces "shockable" over a normal rhythm (40-180/min) or a
//!    flatline, for any artifact size, rate (60-180/min) or the point in the
//!    compression at which it stops
//!    (`artifact_ending_just_before_a_fresh_window_never_gives_shockable_on_a_non_shockable_rhythm`).
//!    The chest sensor is assumed to say "hands off" at the very instant the
//!    artifact stops: real sensing takes about 1 s longer (median 1.05 s on
//!    the Paris recordings), which only helps.
//! 3. That VF is still detected, 0.75 s sooner than after a full reset (the 1 s warm-up less the 0.25 s guard)
//!    (`vf_is_still_detected_after_a_fresh_window_and_sooner_than_after_a_full_reset`).
//! 4. That the guard and the remaining settling after a full reset are
//!    honoured (`a_fresh_window_soon_after_a_full_reset_still_waits_for_the_full_settling`).

use aed_rhythm::filter::Preprocessor;
use aed_rhythm::settings::{FRESH_WINDOW_GUARD_SAMPLES, WARMUP_SAMPLES};
use aed_rhythm::synth::{Rhythm, Synth};
use aed_rhythm::Analyzer;
use aed_types::{EcgSample, RhythmAnalyzer, RhythmDecision};
use core::f32::consts::TAU;

const FS: f32 = 250.0;

/// Shapes of compression artifact. Real artifacts differ from patient to
/// patient; these span a smooth wave and a sharp one-sided pulse.
#[derive(Debug, Clone, Copy)]
enum Shape {
    /// The simulator's shape (`aed-sim` plant): sin + 0.25 sin 2x.
    Sim,
    /// `aed-rhythm`'s `synth` shape: sin + 0.35 sin 2x + 0.1 sin 3x.
    Synth,
    /// A one-sided bump during the downstroke only (40 % of each cycle),
    /// with a mean offset: harder on the high-pass filter.
    Pulse,
}

const SHAPES: [Shape; 3] = [Shape::Sim, Shape::Synth, Shape::Pulse];

/// Artifact value at `phase` (radians), scaled to `pp_uv` peak to peak
/// (approximately).
fn artifact(shape: Shape, phase: f32, pp_uv: f32) -> f32 {
    match shape {
        Shape::Sim => 0.5 * pp_uv * (libm::sinf(phase) + 0.25 * libm::sinf(2.0 * phase)) / 1.1,
        Shape::Synth => {
            0.5 * pp_uv
                * (libm::sinf(phase)
                    + 0.35 * libm::sinf(2.0 * phase)
                    + 0.1 * libm::sinf(3.0 * phase))
                / 1.3
        }
        Shape::Pulse => {
            let f = phase / TAU;
            if f < 0.4 {
                pp_uv * 0.5 * (1.0 - libm::cosf(TAU * f / 0.4))
            } else {
                0.0
            }
        }
    }
}

/// Compression artifact at `per_min`, stopping after `stop_after_s` seconds
/// at a chosen fraction of a compression cycle: the sample stream, as
/// microvolts, of artifact followed by `after_s` seconds of nothing.
fn artifact_stream(
    shape: Shape,
    pp_uv: f32,
    per_min: f32,
    stop_fraction: f32,
    after_s: f32,
) -> (Vec<f32>, usize) {
    let hz = per_min / 60.0;
    // Whole cycles for about 6 s, then the chosen fraction of one more.
    let cycles = libm::ceilf(6.0 * hz);
    let n_art = ((cycles + stop_fraction) / hz * FS) as usize;
    let mut v = Vec::with_capacity(n_art + (after_s * FS) as usize);
    for i in 0..n_art {
        let phase = (TAU * hz * i as f32 / FS) % TAU;
        v.push(artifact(shape, phase, pp_uv));
    }
    v.extend(std::iter::repeat_n(0.0, (after_s * FS) as usize));
    (v, n_art)
}

const STOP_FRACTIONS: [f32; 8] = [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875];
const RATES: [f32; 5] = [60.0, 90.0, 120.0, 150.0, 180.0];

/// The largest leftover of the artifact, after the filters, in the 4 s
/// window that starts `guard_ms` after the artifact stops, as a fraction of
/// the artifact's peak to peak; worst case over shapes, rates and stopping
/// points. Also: the leftover's own peak to peak in that window, per mV of
/// artifact.
fn worst_leftover(guard_ms: u32) -> (f32, f32) {
    let mut worst_peak = 0.0_f32;
    let mut worst_pp = 0.0_f32;
    for shape in SHAPES {
        for rate in RATES {
            for frac in STOP_FRACTIONS {
                let (s, stop) = artifact_stream(shape, 1000.0, rate, frac, 6.0);
                let mut pre = Preprocessor::new();
                let start = stop + (guard_ms as usize * 250 / 1000);
                let end = start + 1000;
                let (mut lo, mut hi, mut peak) = (f32::MAX, f32::MIN, 0.0_f32);
                for (i, x) in s.iter().enumerate() {
                    let y = pre.process(*x).signal_uv;
                    if i >= start && i < end {
                        lo = lo.min(y);
                        hi = hi.max(y);
                        peak = peak.max(y.abs());
                    }
                }
                worst_peak = worst_peak.max(peak / 1000.0);
                worst_pp = worst_pp.max(hi - lo);
            }
        }
    }
    (worst_peak, worst_pp)
}

/// Time after the artifact stops until the leftover stays below `fraction`
/// of the artifact's peak to peak, worst case, in milliseconds.
fn settle_ms(fraction: f32) -> u32 {
    let mut worst = 0_u32;
    for shape in SHAPES {
        for rate in RATES {
            for frac in STOP_FRACTIONS {
                let (s, stop) = artifact_stream(shape, 1000.0, rate, frac, 6.0);
                let mut pre = Preprocessor::new();
                let mut last_above = stop;
                for (i, x) in s.iter().enumerate() {
                    let y = pre.process(*x).signal_uv;
                    if i >= stop && y.abs() > fraction * 1000.0 {
                        last_above = i;
                    }
                }
                worst = worst.max(((last_above - stop) * 4) as u32);
            }
        }
    }
    worst
}

#[test]
fn measure_filter_settling_after_compression_artifact_stops() {
    println!("Leftover compression artifact after the analyser's filters (1 Hz high-pass, 50/60 Hz notches,");
    println!(
        "30 Hz low-pass), running continuously, after the artifact stops abruptly (SYNTHETIC)."
    );
    println!("Worst case over 3 shapes x 5 rates (60-180/min) x 8 stopping points:");
    for f in [0.10, 0.03, 0.01, 0.003] {
        println!(
            "  below {:>4.1} % of the artifact's size after {:>4} ms",
            f * 100.0,
            settle_ms(f)
        );
    }
    println!(
        "Largest leftover in the 4 s window starting 'guard' after the stop, per 1 mV of artifact:"
    );
    println!("| guard ms | peak µV | peak to peak µV | for a 10 mV artifact, peak to peak µV |");
    let mut previous = f32::MAX;
    for g in [0, 100, 200, 250, 300, 400, 500, 750, 1000] {
        let (peak, pp) = worst_leftover(g);
        println!(
            "| {g} | {:.1} | {:.1} | {:.0} |",
            peak * 1000.0,
            pp,
            pp * 10.0
        );
        assert!(
            pp <= previous + 1e-3,
            "the leftover must shrink as the guard grows"
        );
        previous = pp;
    }
    // The 1 Hz Butterworth high-pass has a time constant of about
    // 1 / (0.707 * 2 pi * 1 Hz) = 0.225 s. Within 1 s the leftover must be
    // below 3 % of the artifact (the full warm-up was never shorter).
    assert!(settle_ms(0.03) <= 1000);
}

/// One run: a full reset, `track` through artifact plus rhythm (the
/// compressions), `restart_window` with `guard` the instant the artifact
/// stops, then the rhythm alone for `analyse_s`. Returns every announced
/// decision with its time after the restart, in seconds.
#[allow(clippy::too_many_arguments)]
fn run_after_artifact(
    rhythm: Rhythm,
    seed: u32,
    shape: Shape,
    pp_uv: f32,
    per_min: f32,
    stop_fraction: f32,
    guard: usize,
    analyse_s: f32,
) -> Vec<(f32, RhythmDecision)> {
    let (art, stop) = artifact_stream(shape, pp_uv, per_min, stop_fraction, analyse_s);
    let mut heart = Synth::new(rhythm, seed).with_noise(10.0);
    let mut a = Analyzer::new();
    a.reset();
    let mut out = Vec::new();
    for (i, x) in art.iter().enumerate() {
        let v = (heart.next_uv() + x).clamp(-32_000.0, 32_000.0) as EcgSample;
        if i < stop {
            a.track(v);
        } else {
            if i == stop {
                a.restart_window_with_guard(guard);
            }
            if let Some(d) = a.push(v) {
                out.push(((i - stop + 1) as f32 / FS, d));
            }
        }
    }
    out
}

/// Artifact sizes swept, microvolts peak to peak. The simulator uses
/// 1.5 mV; published CPR artifact in pad ECG reaches several millivolts
/// (UNVERIFIED general figure); 20 mV is beyond anything plausible.
const AMPLITUDES: [f32; 5] = [500.0, 1500.0, 4000.0, 10_000.0, 20_000.0];

fn non_shockable_rhythms() -> Vec<Rhythm> {
    let mut v: Vec<Rhythm> = (40..=180)
        .step_by(20)
        .map(|bpm| Rhythm::sinus(bpm as f32))
        .collect();
    v.push(Rhythm::Sinus {
        bpm: 170.0,
        amplitude_uv: 2500.0,
    });
    v.push(Rhythm::Sinus {
        bpm: 50.0,
        amplitude_uv: 300.0,
    });
    v.push(Rhythm::Asystole);
    v
}

/// Shockable answers on non-shockable rhythms after artifact, and how the
/// first answer was distributed, for one guard.
fn sweep_non_shockable(guard: usize, seeds: &[u32]) -> (u32, u32, [u32; 3]) {
    let mut runs = 0;
    let mut shocks = 0;
    let mut first = [0_u32; 3];
    for rhythm in non_shockable_rhythms() {
        for &seed in seeds {
            for shape in SHAPES {
                for pp in AMPLITUDES {
                    for rate in RATES {
                        for frac in STOP_FRACTIONS {
                            let d = run_after_artifact(
                                rhythm, seed, shape, pp, rate, frac, guard, 10.0,
                            );
                            runs += 1;
                            if d.iter().any(|(_, d)| *d == RhythmDecision::Shockable) {
                                shocks += 1;
                                println!("  SHOCKABLE: {rhythm:?} seed {seed} {shape:?} {pp} µV {rate}/min stop at {frac}: {d:?}");
                            }
                            match d.first().map(|x| x.1) {
                                Some(RhythmDecision::NonShockable) => first[0] += 1,
                                Some(RhythmDecision::Indeterminate) => first[1] += 1,
                                _ => first[2] += 1,
                            }
                        }
                    }
                }
            }
        }
    }
    (runs, shocks, first)
}

#[test]
fn artifact_ending_just_before_a_fresh_window_never_gives_shockable_on_a_non_shockable_rhythm() {
    // The device's guard. `measure_guard_choices` (ignored, slow) repeats
    // this with a second seed and other guards, including none.
    let guard = FRESH_WINDOW_GUARD_SAMPLES;
    let (runs, shocks, first) = sweep_non_shockable(guard, &[1]);
    println!(
        "guard {} ms: {runs} runs, {shocks} with any 'shockable'; first answer no shock {} / not sure {} / none {} (SYNTHETIC)",
        guard * 4,
        first[0],
        first[1],
        first[2]
    );
    assert_eq!(shocks, 0, "artifact tail produced a shockable answer");
    assert_eq!(first[2], 0, "every run must reach an answer");
}

#[test]
fn vf_is_still_detected_after_a_fresh_window_and_sooner_than_after_a_full_reset() {
    let mut detected = 0;
    let mut total = 0;
    let mut sooner = 0;
    for seed in [1, 2, 3, 42, 9001] {
        for rhythm in [
            Rhythm::coarse_vf(),
            Rhythm::VentricularFibrillation {
                amplitude_uv: 400.0,
            },
            Rhythm::rapid_vt(180.0),
        ] {
            for pp in [1500.0, 10_000.0] {
                for frac in [0.0, 0.25, 0.5, 0.75] {
                    total += 1;
                    let warm = run_after_artifact(
                        rhythm,
                        seed,
                        Shape::Sim,
                        pp,
                        110.0,
                        frac,
                        FRESH_WINDOW_GUARD_SAMPLES,
                        12.0,
                    );
                    let at = warm
                        .iter()
                        .find(|(_, d)| *d == RhythmDecision::Shockable)
                        .map(|x| x.0);
                    // The same signal after a full reset at the same moment (today's behaviour).
                    let cold = run_after_artifact(
                        rhythm,
                        seed,
                        Shape::Sim,
                        pp,
                        110.0,
                        frac,
                        WARMUP_SAMPLES,
                        12.0,
                    );
                    let cold_at = cold
                        .iter()
                        .find(|(_, d)| *d == RhythmDecision::Shockable)
                        .map(|x| x.0);
                    if let Some(t) = at {
                        detected += 1;
                        if cold_at.is_some_and(|c| c - t > 0.5) {
                            sooner += 1;
                        }
                    }
                }
            }
        }
    }
    println!("VF/VT after artifact: shockable within 12 s in {detected} of {total}; sooner than after a full reset in {sooner} (SYNTHETIC)");
    assert!(detected * 10 >= total * 9, "VF must still be detected");
    assert!(
        sooner * 10 >= detected * 9,
        "the fresh window must save time"
    );
}

#[test]
fn a_fresh_window_soon_after_a_full_reset_still_waits_for_the_full_settling() {
    let mut heart = Synth::new(Rhythm::coarse_vf(), 3);
    // Full reset, then only 0.2 s through the filters before the restart:
    // the rest of the full second of settling still applies.
    let mut a = Analyzer::new();
    for _ in 0..50 {
        a.track(heart.next_sample());
    }
    a.restart_window();
    let mut first_window_at = None;
    for i in 0..250 * 8 {
        let _ = a.push(heart.next_sample());
        if first_window_at.is_none() && a.last_report().is_some() {
            first_window_at = Some(i + 1);
        }
    }
    assert_eq!(first_window_at, Some(WARMUP_SAMPLES - 50 + 1000));

    // Settled filters: the window starts after the guard only.
    let mut a = Analyzer::new();
    for _ in 0..500 {
        a.track(heart.next_sample());
    }
    a.restart_window();
    let mut first_window_at = None;
    for i in 0..250 * 8 {
        let _ = a.push(heart.next_sample());
        if first_window_at.is_none() && a.last_report().is_some() {
            first_window_at = Some(i + 1);
        }
    }
    assert_eq!(first_window_at, Some(FRESH_WINDOW_GUARD_SAMPLES + 1000));

    // A full reset after tracking forgets the filters: the full second again.
    let mut a = Analyzer::new();
    for _ in 0..500 {
        a.track(heart.next_sample());
    }
    a.reset();
    let mut first_window_at = None;
    for i in 0..250 * 8 {
        let _ = a.push(heart.next_sample());
        if first_window_at.is_none() && a.last_report().is_some() {
            first_window_at = Some(i + 1);
        }
    }
    assert_eq!(first_window_at, Some(WARMUP_SAMPLES + 1000));
}

#[test]
fn a_fresh_window_forgets_the_previous_window_and_its_confirmations() {
    let mut vf = Synth::new(Rhythm::coarse_vf(), 1);
    let mut a = Analyzer::new();
    let mut confirmed = false;
    for _ in 0..250 * 9 {
        confirmed |= a.push(vf.next_sample()) == Some(RhythmDecision::Shockable);
    }
    assert!(confirmed);
    a.restart_window();
    assert!(a.last_report().is_none());
    // Nothing is announced until a whole new window has been seen, and a
    // shockable answer needs three fresh windows again.
    let mut first_shock = None;
    for i in 0..250 * 8 {
        if a.push(vf.next_sample()) == Some(RhythmDecision::Shockable) && first_shock.is_none() {
            first_shock = Some(i + 1);
        }
    }
    assert_eq!(first_shock, Some(FRESH_WINDOW_GUARD_SAMPLES + 1000 + 500));
}

/// The evidence for the guard: the same sweeps for several guards. Slow;
/// run with `cargo test --release -p aed-rhythm --test fresh_window -- --ignored --nocapture`.
#[test]
#[ignore = "measurement, about a minute per guard in release"]
fn measure_guard_choices() {
    println!("| guard ms | non-shockable runs | any shockable | first answer no shock / not sure | asystole: median, worst s to 'no shock' | VF/VT: detected, median s to 'shock' |");
    for guard_ms in [0_usize, 100, 250, 500, 1000] {
        let guard = guard_ms * 250 / 1000;
        let (runs, shocks, first) = sweep_non_shockable(guard, &[1, 7]);
        let mut asys = Vec::new();
        for seed in [1, 7] {
            for shape in SHAPES {
                for pp in AMPLITUDES {
                    for rate in RATES {
                        for frac in STOP_FRACTIONS {
                            let d = run_after_artifact(
                                Rhythm::Asystole,
                                seed,
                                shape,
                                pp,
                                rate,
                                frac,
                                guard,
                                12.0,
                            );
                            asys.push(
                                d.iter()
                                    .find(|x| x.1 == RhythmDecision::NonShockable)
                                    .map_or(99.0, |x| x.0),
                            );
                        }
                    }
                }
            }
        }
        asys.sort_by(f32::total_cmp);
        let mut vf = Vec::new();
        let mut vf_total = 0;
        for seed in [1, 2, 3, 42, 9001] {
            for rhythm in [
                Rhythm::coarse_vf(),
                Rhythm::VentricularFibrillation {
                    amplitude_uv: 400.0,
                },
                Rhythm::rapid_vt(180.0),
            ] {
                for shape in SHAPES {
                    for pp in [1500.0, 10_000.0] {
                        for frac in [0.0, 0.25, 0.5, 0.75] {
                            vf_total += 1;
                            let d = run_after_artifact(
                                rhythm, seed, shape, pp, 110.0, frac, guard, 12.0,
                            );
                            if let Some(x) = d.iter().find(|x| x.1 == RhythmDecision::Shockable) {
                                vf.push(x.0);
                            }
                        }
                    }
                }
            }
        }
        vf.sort_by(f32::total_cmp);
        println!(
            "| {guard_ms} | {runs} | {shocks} | {} / {} | {:.2}, {:.2} | {} of {vf_total}, {:.2} |",
            first[0],
            first[1],
            asys.get(asys.len() / 2).copied().unwrap_or(0.0),
            asys.last().copied().unwrap_or(0.0),
            vf.len(),
            vf.get(vf.len() / 2).copied().unwrap_or(0.0),
        );
    }
}
