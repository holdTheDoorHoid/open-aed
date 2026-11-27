// SPDX-License-Identifier: GPL-3.0-or-later
//! The seeded random campaign: many randomised scenarios with the safety
//! invariants checked at every step.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use aed_core::RescueState;
use aed_rhythm::settings::WINDOW_SECONDS;
use aed_rhythm::synth::{Rhythm, Xorshift32};
use aed_types::PatientMode;

use crate::plant::Fault;
use crate::runner::{run, Rule, RunResult};
use crate::scenario::{Action, Expect, Responder, Scenario, Trigger};

/// How long after a true rhythm change the analyser may still say
/// "shockable": its 4-second window (aed-rhythm `WINDOW_SECONDS`; the
/// aed-rhythm test `vf_to_sinus_stops_shockable_quickly` allows exactly
/// this). A shock fired inside this time after VF stopped is classified as
/// a known detection-latency limitation, not as a logic failure.
pub const DETECTION_LATENCY_MS: u32 = (WINDOW_SECONDS as u32) * 1_000;

fn pick<T: Copy>(rng: &mut Xorshift32, items: &[T]) -> Option<T> {
    let n = u32::try_from(items.len()).ok()?;
    let i = rng.next_u32().checked_rem(n)?;
    items.get(usize::try_from(i).ok()?).copied()
}

fn range(rng: &mut Xorshift32, lo: u32, hi: u32) -> u32 {
    lo.saturating_add(rng.next_u32().checked_rem(hi.saturating_sub(lo).max(1)).unwrap_or(0))
}

fn random_rhythm(rng: &mut Xorshift32) -> Rhythm {
    match range(rng, 0, 10) {
        0..=3 => Rhythm::VentricularFibrillation {
            amplitude_uv: 300.0 + 1_200.0 * rng.next_unit(),
        },
        4 => Rhythm::rapid_vt(150.0 + 80.0 * rng.next_unit()),
        5 => Rhythm::fine_vf(),
        6 => Rhythm::Asystole,
        _ => Rhythm::sinus(45.0 + 100.0 * rng.next_unit()),
    }
}

/// One random scenario for `seed`.
#[must_use]
pub fn random_scenario(seed: u32) -> Scenario {
    let mut rng = Xorshift32::new(seed.wrapping_mul(2_654_435_761).wrapping_add(7));
    let mut s = Scenario::base(&format!("random-{seed}"), "randomised");
    s.seed = seed;
    s.mode = if range(&mut rng, 0, 4) == 0 {
        PatientMode::Child
    } else {
        PatientMode::Adult
    };
    // Mostly in range, sometimes just outside it.
    s.pad_ohms = f64::from(range(&mut rng, 15, 200));
    s.pads_on_at_ms = range(&mut rng, 2_000, 15_000);
    s.rhythm = random_rhythm(&mut rng);
    s.after_shock = (0..3)
        .map(|_| {
            if range(&mut rng, 0, 2) == 0 {
                None
            } else {
                Some(random_rhythm(&mut rng))
            }
        })
        .collect();
    // 0 to 2 spontaneous rhythm changes at random times.
    for _ in 0..range(&mut rng, 0, 3) {
        let at = range(&mut rng, 5_000, 140_000);
        s.script
            .push((Trigger::At(at), Action::SetRhythm(random_rhythm(&mut rng))));
    }
    if range(&mut rng, 0, 4) == 0 {
        s.noise.rms_uv = 10.0 + 60.0 * rng.next_unit();
        s.noise.mains_uv = 200.0 * rng.next_unit();
        s.noise.wander_uv = 400.0 * rng.next_unit();
    }
    s.responder = Responder {
        reaction_ms: range(&mut rng, 300, 2_000),
        reattach_after_ms: if range(&mut rng, 0, 2) == 0 {
            Some(range(&mut rng, 3_000, 15_000))
        } else {
            None
        },
        ..Responder::default()
    };
    if range(&mut rng, 0, 5) == 0 {
        s.plant.link_delay_ms = range(&mut rng, 1, 30);
    }
    s.plant.main_clock_ppm = i32::try_from(range(&mut rng, 0, 20_001)).unwrap_or(0) - 10_000;
    s.plant.supervisor_clock_ppm = i32::try_from(range(&mut rng, 0, 20_001)).unwrap_or(0) - 10_000;
    // One random fault, at a random time, in about half the runs.
    if range(&mut rng, 0, 2) == 0 {
        if let Some(f) = pick(&mut rng, &Fault::ALL) {
            let when = if range(&mut rng, 0, 2) == 0 {
                Trigger::At(range(&mut rng, 0, 60_000))
            } else {
                let state = pick(
                    &mut rng,
                    &[
                        RescueState::Analysing,
                        RescueState::Charging,
                        RescueState::Countdown,
                        RescueState::Cpr,
                    ],
                )
                .unwrap_or(RescueState::Charging);
                Trigger::AfterState {
                    state,
                    nth: 1,
                    after_ms: range(&mut rng, 0, 6_000),
                }
            };
            s.script.push((when, Action::Fault(f)));
        }
    }
    s.duration_ms = range(&mut rng, 60_000, 160_000);
    s.expect = Expect::InvariantsOnly;
    s
}

/// What a campaign found.
#[derive(Debug, Clone, Default)]
pub struct CampaignSummary {
    /// Runs made.
    pub runs: usize,
    /// Runs with at least one shock.
    pub runs_with_shock: usize,
    /// Total shocks.
    pub shocks: usize,
    /// Runs that ended in the Fault state.
    pub ended_in_fault: usize,
    /// Runs with a fault injected.
    pub with_fault: usize,
    /// Count of final states.
    pub final_states: BTreeMap<String, usize>,
    /// Count of violations per rule (after classifying the known limitations below).
    pub violations: BTreeMap<String, usize>,
    /// Shocks fired on a non-shockable rhythm within the analyser's
    /// detection latency of the rhythm changing (a known limitation).
    pub late_rhythm_change_shocks: usize,
    /// Runs where the device stayed silent after switch-on (finding: the
    /// self-test charge step waits the full charge time limit).
    pub silent_at_power_on: usize,
    /// Seeds of the runs counted in `late_rhythm_change_shocks`, with details.
    pub late_seeds: Vec<(u32, String)>,
    /// Runs that started in VF/VT with no fault injected and pads in range.
    pub shockable_clean_runs: usize,
    /// Of those, how many delivered at least one shock.
    pub shockable_clean_shocked: usize,
    /// Those that were not shocked: seed and a description of the run.
    pub shockable_clean_unshocked: Vec<(u32, String)>,
    /// Seeds of runs with an unexplained violation, for replay.
    pub failing_seeds: Vec<(u32, String)>,
}

impl CampaignSummary {
    /// True if no run broke a rule outside the known, classified limitations.
    #[must_use]
    pub fn clean(&self) -> bool {
        self.failing_seeds.is_empty()
    }
}

/// Classify one run's violations into the summary.
fn tally(sum: &mut CampaignSummary, seed: u32, r: &RunResult) {
    sum.runs += 1;
    sum.shocks += r.shocks.len();
    if !r.shocks.is_empty() {
        sum.runs_with_shock += 1;
    }
    if r.final_state == RescueState::Fault {
        sum.ended_in_fault += 1;
    }
    if !r.injected.is_empty() {
        sum.with_fault += 1;
    }
    *sum
        .final_states
        .entry(format!("{:?}", r.final_state))
        .or_insert(0) += 1;
    let in_range = (25.0..=175.0).contains(&r.scenario.pad_ohms);
    if crate::plant::rhythm_is_shockable(r.scenario.rhythm) && r.injected.is_empty() && in_range {
        sum.shockable_clean_runs += 1;
        if r.shocks.is_empty() {
            let first = r.decisions.first().map(|&(_, d)| d);
            let changes = r
                .scenario
                .script
                .iter()
                .filter(|(_, a)| matches!(a, Action::SetRhythm(_)))
                .count();
            sum.shockable_clean_unshocked.push((
                seed,
                format!(
                    "{}, responder reaction {} ms, noise {:.0} µV rms, {} scripted rhythm change(s), \
                     first answer {:?}, ran {} s, pads on at {} s",
                    crate::plant::rhythm_name(r.scenario.rhythm),
                    r.scenario.responder.reaction_ms,
                    r.scenario.noise.rms_uv,
                    changes,
                    first,
                    r.scenario.duration_ms / 1000,
                    r.scenario.pads_on_at_ms / 1000
                ),
            ));
        } else {
            sum.shockable_clean_shocked += 1;
        }
    }
    for v in &r.violations {
        let late = v.rule == Rule::FiredOnNonShockableRhythm
            && v.rhythm_changed_ms_before.is_some_and(|ms| ms <= DETECTION_LATENCY_MS);
        if v.rule == Rule::SilentAtPowerOn {
            sum.silent_at_power_on += 1;
        } else if late {
            sum.late_rhythm_change_shocks += 1;
            sum.late_seeds.push((seed, v.detail.clone()));
        } else {
            *sum
                .violations
                .entry(v.rule.describe().to_string())
                .or_insert(0) += 1;
            sum.failing_seeds
                .push((seed, format!("{}: {}", v.rule.describe(), v.detail)));
        }
    }
}

/// Run `n` random scenarios starting from `first_seed`, spread over the
/// computer's processor cores. The result does not depend on the number of
/// cores: every run is seeded on its own and results are tallied in seed order.
#[must_use]
pub fn run_campaign(n: u32, first_seed: u32) -> CampaignSummary {
    let seeds: Vec<u32> = (0..n).map(|i| first_seed.wrapping_add(i)).collect();
    let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let chunk = seeds.len().div_ceil(workers).max(1);
    // Each worker returns (seed, outcome) pairs; only plain data crosses threads.
    let mut outcomes: Vec<(u32, Result<RunResult, String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|&seed| {
                            let r = run(&random_scenario(seed)).map_err(|e| format!("{e:?}"));
                            (seed, r)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| {
                h.join()
                    .unwrap_or_else(|_| vec![(0, Err("worker thread failed".to_string()))])
            })
            .collect()
    });
    outcomes.sort_by_key(|(seed, _)| *seed);
    let mut sum = CampaignSummary::default();
    for (seed, outcome) in outcomes {
        match outcome {
            Ok(r) => tally(&mut sum, seed, &r),
            Err(e) => sum.failing_seeds.push((seed, format!("setup or run error {e}"))),
        }
    }
    sum
}

/// The campaign summary as text.
#[must_use]
pub fn campaign_text(s: &CampaignSummary) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "RANDOM CAMPAIGN (SIMULATION ONLY)");
    let _ = writeln!(out, "  Runs:                         {}", s.runs);
    let _ = writeln!(out, "  Runs with a fault injected:   {}", s.with_fault);
    let _ = writeln!(out, "  Runs with at least one shock: {}", s.runs_with_shock);
    let _ = writeln!(out, "  Shocks in total:              {}", s.shocks);
    let _ = writeln!(out, "  Runs ending in Fault:         {}", s.ended_in_fault);
    let _ = writeln!(out, "  Final states:");
    for (k, v) in &s.final_states {
        let _ = writeln!(out, "    {k:<12} {v}");
    }
    let _ = writeln!(
        out,
        "  Known limitation, shock within {} s of VF/VT ending: {}",
        DETECTION_LATENCY_MS / 1000,
        s.late_rhythm_change_shocks
    );
    for (seed, what) in &s.late_seeds {
        let _ = writeln!(out, "    seed {seed}: {what}");
    }
    let _ = writeln!(
        out,
        "  Known finding, device silent for over 8 s after switch-on: {}",
        s.silent_at_power_on
    );
    let _ = writeln!(
        out,
        "  Started in VF/VT, no fault, pads in range: {} runs, {} shocked",
        s.shockable_clean_runs, s.shockable_clean_shocked
    );
    for (seed, what) in &s.shockable_clean_unshocked {
        let _ = writeln!(out, "    not shocked, seed {seed}: {what}");
    }
    if s.violations.is_empty() {
        let _ = writeln!(out, "  Other invariant violations:   none");
    } else {
        let _ = writeln!(out, "  OTHER INVARIANT VIOLATIONS:");
        for (k, v) in &s.violations {
            let _ = writeln!(out, "    {k}: {v}");
        }
        for (seed, what) in s.failing_seeds.iter().take(20) {
            let _ = writeln!(out, "    seed {seed}: {what}");
        }
    }
    out
}
