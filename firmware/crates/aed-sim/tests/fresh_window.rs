// SPDX-License-Identifier: GPL-3.0-or-later
//! Decision D-47: once compression sensing confirms the chest is still, the
//! rhythm analysis starts its window on filters that ran through CPR,
//! instead of waiting a settling second after a full reset.
//!
//! SIMULATION ONLY: cartoon heart signals and compression artifact, a
//! scripted responder, the real `aed-cpr` detector on a cartoon impedance
//! channel, provisional settings. Says nothing about real patients.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    missing_docs
)]

mod common;

use aed_core::RescueState;
use aed_rhythm::synth::Rhythm;
use aed_sim::report::secs;
use aed_sim::{by_name, run, RunResult, Scenario, Simulation};

/// D-67 / FW-13 (design v0.3): compression readings are thrown away while
/// the charger runs, and with charging during analysis that covers the
/// moment compressions stop in these scenarios, so the fresh window of D-47
/// is not reached. These tests check the D-47 mechanism itself with the
/// blanking off; the safety part is also checked with it on
/// (`d67_blanking_never_shocks_a_non_shockable_rhythm_either`). See
/// firmware/REV3_NOTES.md §3.2.
fn no_blanking() -> aed_core::Config {
    aed_core::Config {
        blank_compressions_while_charging: false,
        ..aed_core::Config::default()
    }
}
use common::{assert_invariants, fail};

/// Compressions run on into the first analysis by `late_ms`, then stop.
fn late_stop(rhythm: Rhythm, uv: f64, per_min: f64, late_ms: u32, sensing: bool) -> RunResult {
    late_stop_with(rhythm, uv, per_min, late_ms, sensing, no_blanking())
}

fn late_stop_with(
    rhythm: Rhythm,
    uv: f64,
    per_min: f64,
    late_ms: u32,
    sensing: bool,
    config: aed_core::Config,
) -> RunResult {
    let mut s = Scenario::base("d47", "compressions stop late into the first analysis");
    s.rhythm = rhythm;
    s.after_shock = Vec::new();
    s.responder.reaction_ms = late_ms;
    s.responder.compression_uv = uv;
    s.responder.compression_per_min = per_min;
    s.chest_sensing = sensing;
    s.duration_ms = 30_000;
    let r = Simulation::with_config(&s, config).unwrap().run();
    assert_invariants(&r);
    r
}

#[test]
fn d67_blanking_never_shocks_a_non_shockable_rhythm_either() {
    let mut jobs = Vec::new();
    for rhythm in [Rhythm::Asystole, Rhythm::sinus(60.0), Rhythm::sinus(150.0)] {
        for per_min in [120.0, 150.0, 180.0] {
            for late in [500, 1_500, 2_250, 4_000] {
                jobs.push((rhythm, per_min, late));
            }
        }
    }
    let results = in_parallel(jobs, |&(rhythm, per_min, late)| {
        let r = late_stop_with(
            rhythm,
            5_000.0,
            per_min,
            late,
            true,
            aed_core::Config::default(),
        );
        let bad = r.entered(RescueState::Countdown) || !r.shocks.is_empty();
        if bad {
            fail(
                &r,
                &format!("{rhythm:?} {per_min}/min stopping {late} ms late: shock advised"),
            );
        }
        r.fresh_windows > 0
    });
    println!(
        "D-67 blanking on (SIMULATED): {} runs, no shock advised; {} used a fresh window",
        results.len(),
        results.iter().filter(|x| **x).count()
    );
}

fn in_parallel<T: Send + 'static, R: Send + 'static>(jobs: Vec<T>, f: fn(&T) -> R) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let jobs = std::sync::Arc::new(std::sync::Mutex::new(
        jobs.into_iter().enumerate().collect::<Vec<_>>(),
    ));
    let out = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let jobs = jobs.clone();
            let out = out.clone();
            std::thread::spawn(move || loop {
                let job = jobs.lock().unwrap().pop();
                let Some((i, job)) = job else { break };
                let r = f(&job);
                out.lock().unwrap().push((i, r));
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let mut v = std::mem::take(&mut *out.lock().unwrap());
    v.sort_by_key(|x| x.0);
    v.into_iter().map(|x| x.1).collect()
}

#[test]
fn d47_compressions_ending_just_before_the_window_never_shock_a_non_shockable_rhythm() {
    // Normal rhythm 40-180/min and flatline; artifact 0.5 to 10 mV at
    // 60-180 compressions a minute; compressions stopping at 250 ms steps
    // (so at different points of a compression) from 0.5 to 2.25 s after
    // "Analysing".
    let mut jobs = Vec::new();
    let mut rhythms: Vec<Rhythm> = (40..=180)
        .step_by(20)
        .map(|b| Rhythm::sinus(b as f32))
        .collect();
    rhythms.push(Rhythm::Asystole);
    for rhythm in rhythms {
        for uv in [500.0, 1_500.0, 5_000.0, 10_000.0] {
            for per_min in [60.0, 90.0, 120.0, 150.0, 180.0] {
                for late in [500, 750, 1_000, 1_250, 1_500, 1_750, 2_000, 2_250] {
                    jobs.push((rhythm, uv, per_min, late));
                }
            }
        }
    }
    let n = jobs.len();
    let results = in_parallel(jobs, |&(rhythm, uv, per_min, late)| {
        let r = late_stop(rhythm, uv, per_min, late, true);
        let bad = r.entered(RescueState::Countdown)
            || r.entered(RescueState::Charging)
            || !r.shocks.is_empty();
        let fresh = r.fresh_windows > 0;
        if bad {
            fail(
                &r,
                &format!("{rhythm:?} {uv} µV {per_min}/min stopping {late} ms late: shock advised"),
            );
        }
        fresh
    });
    let fresh = results.iter().filter(|x| **x).count();
    println!("D-47 sweep (SIMULATED): {n} runs of non-shockable rhythm with compressions running into the analysis, none advised a shock; {fresh} used a fresh window");
    assert!(
        fresh * 10 >= n * 9,
        "the fresh window must actually be used in these runs"
    );
}

#[test]
fn d47_vf_is_still_shocked_and_the_hands_off_pause_is_shorter() {
    let with = common::run_named("adult-vf");
    if let Some(t) = with.timings.first() {
        println!(
            "adult-vf with the D-67 blanking (SIMULATED): hands-off before the shock {}, fresh window {}",
            secs(t.hands_off_ms.unwrap_or(0)),
            with.fresh_windows > 0
        );
    } else {
        fail(&with, "no shock with the blanking on");
    }
    let r = Simulation::with_config(&by_name("adult-vf").unwrap(), no_blanking())
        .unwrap()
        .run();
    let t = r.timings.first().unwrap_or_else(|| fail(&r, "no shock"));
    let hands_off = t.hands_off_ms.unwrap();
    println!(
        "adult-vf with D-47 (SIMULATED): analysis {}, hands-off before the shock {} (before D-47: 8.8 s and 11.1 s)",
        secs(t.analysis_ms),
        secs(hands_off)
    );
    assert!((r.fresh_windows > 0));
    // 0.75 s of the settling second recovered (1 s less the 0.25 s guard).
    assert!(
        (10_000..=10_500).contains(&hands_off),
        "hands-off {hands_off} ms"
    );
}

#[test]
fn d47_changes_nothing_without_compression_sensing() {
    // Without sensing the device must behave exactly as before D-47: every
    // analysis starts from a full reset.
    let mut s = by_name("adult-vf").unwrap();
    s.chest_sensing = false;
    let r = run(&s).unwrap();
    assert_invariants(&r);
    assert!(!(r.fresh_windows > 0));
    let t = r.timings.first().unwrap_or_else(|| fail(&r, "no shock"));
    println!(
        "adult-vf without compression sensing (SIMULATED): analysis {}, hands-off {}",
        secs(t.analysis_ms),
        t.hands_off_ms.map_or_else(|| "-".to_string(), secs)
    );
    // The figures from before compression sensing (README timing table).
    assert!(
        (6_900..=7_100).contains(&t.analysis_ms),
        "analysis {} ms",
        t.analysis_ms
    );
    let hands_off = t.hands_off_ms.unwrap();
    assert!(
        (9_200..=9_400).contains(&hands_off),
        "hands-off {hands_off} ms"
    );
}

#[test]
fn d47_vf_after_late_compressions_is_still_shocked_with_sensing_at_every_rate() {
    let jobs: Vec<(f64, u32)> = [60.0, 120.0, 180.0]
        .into_iter()
        .flat_map(|p| [500_u32, 1_500, 3_000].into_iter().map(move |l| (p, l)))
        .collect();
    let shocked = in_parallel(jobs, |&(per_min, late)| {
        let r = late_stop(Rhythm::coarse_vf(), 1_500.0, per_min, late, true);
        !r.shocks.is_empty()
    });
    assert!(shocked.iter().all(|x| *x), "VF not shocked: {shocked:?}");
}
