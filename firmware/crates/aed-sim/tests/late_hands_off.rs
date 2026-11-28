// SPDX-License-Identifier: GPL-3.0-or-later
//! Finding F-1 (fixed): compressions that run on into an analysis.
//!
//! "Not sure" no longer ends an analysis, so a responder who stops late
//! still gets a shock, as long as the clean signal arrives before the
//! analysis time limit. These tests measure how late is still fine and
//! check the reverse cases stay safe. SIMULATION ONLY: cartoon heart
//! signals, a scripted responder, provisional settings.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_core::{Prompt, RescueState};
use aed_rhythm::synth::Rhythm;
use aed_sim::report::secs;
use aed_sim::{run, RunResult, Scenario};
use common::{assert_invariants, fail};

/// Late stops checked, ms after "Analysing" (the responder's reaction time).
const LATE_STOPS_MS: [u32; 10] = [
    500, 1_000, 1_500, 2_000, 2_500, 3_000, 3_500, 4_000, 5_000, 6_000,
];

/// The late stop the design must tolerate (brief for F-1): 3 s.
const REQUIRED_TOLERANCE_MS: u32 = 3_000;

fn late_stop(rhythm: Rhythm, late_ms: u32) -> RunResult {
    let mut s = Scenario::base(
        "late-stop",
        "compressions stop late into the first analysis",
    );
    s.rhythm = rhythm;
    s.after_shock = Vec::new();
    s.responder.reaction_ms = late_ms;
    // Long enough for the first analysis, the shock and the start of CPR,
    // short of the second analysis.
    s.duration_ms = 40_000;
    let r = run(&s).unwrap();
    assert_invariants(&r);
    r
}

/// True if the first analysis ended in a shock (charging started straight from it).
fn first_analysis_shocked(r: &RunResult) -> bool {
    !r.shocks.is_empty() && r.states.iter().filter(|&&(_, s)| s.is_analysing()).count() == 1
}

#[test]
fn vf_is_shocked_from_the_first_analysis_when_compressions_stop_up_to_3_s_late() {
    println!("Late stop of compressions after 'Analysing' (SIMULATED, provisional settings):");
    println!("cell = analysis time (pads/'Analysing' to 'shock advised') / hands-off pause (compressions stop to shock)");
    print!("| VF size µV |");
    for late in LATE_STOPS_MS {
        print!(" {} |", secs(late));
    }
    println!();
    for amp in [400.0_f32, 800.0, 1_500.0] {
        let mut row = format!("| {amp:.0} |");
        for late in LATE_STOPS_MS {
            let r = late_stop(Rhythm::VentricularFibrillation { amplitude_uv: amp }, late);
            match r.timings.first() {
                Some(t) if first_analysis_shocked(&r) => {
                    row.push_str(&format!(
                        " {} / {} |",
                        secs(t.analysis_ms),
                        t.hands_off_ms.map_or_else(|| "-".to_string(), secs)
                    ));
                    assert!(t.analysis_ms <= r.config.analysis_time_limit_ms);
                }
                _ => {
                    row.push_str(" no shock |");
                    if late <= REQUIRED_TOLERANCE_MS {
                        fail(
                            &r,
                            &format!(
                                "{amp} µV VF not shocked with compressions stopping {late} ms late"
                            ),
                        );
                    }
                }
            }
        }
        println!("{row}");
    }
}

#[test]
fn normal_rhythm_with_compressions_running_into_the_analysis_is_never_shocked() {
    for late in [500, 2_000, 4_000, 8_000, 15_000] {
        let r = late_stop(Rhythm::sinus(80.0), late);
        // The capacitor charges during every analysis (D-29); what matters
        // is that no shock is ever advised or given.
        if r.entered(RescueState::Countdown) || !r.shocks.is_empty() {
            fail(
                &r,
                &format!("normal rhythm charged or shocked (late stop {late} ms)"),
            );
        }
        if !r.entered(RescueState::Cpr) {
            fail(&r, "never got to CPR");
        }
    }
}

#[test]
fn asystole_with_compressions_running_into_the_analysis_still_gets_to_cpr_promptly() {
    for late in [500, 2_000, 4_000, 8_000] {
        let r = late_stop(Rhythm::Asystole, late);
        if r.entered(RescueState::Countdown) || !r.shocks.is_empty() {
            fail(
                &r,
                &format!("asystole charged or shocked (late stop {late} ms)"),
            );
        }
        let analysing_at = r
            .states
            .iter()
            .find(|&&(_, s)| s.is_analysing())
            .map(|&(t, _)| t)
            .unwrap();
        let cpr_at = r
            .prompts
            .iter()
            .find(|&&(t, p)| p == Prompt::StartCpr && t >= analysing_at)
            .map(|&(t, _)| t)
            .unwrap_or_else(|| fail(&r, "never told to start CPR"));
        let took = cpr_at - analysing_at;
        println!(
            "asystole, compressions stop {} late: 'Start CPR' after {} (SIMULATED)",
            secs(late),
            secs(took)
        );
        // Never longer than the analysis limit (plus one analyser step).
        assert!(
            took <= r.config.analysis_time_limit_ms + 1_000,
            "took {took} ms"
        );
    }
}
