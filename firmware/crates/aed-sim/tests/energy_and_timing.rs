// SPDX-License-Identifier: GPL-3.0-or-later
//! Delivered-shock checks across the resistance range, and the timing
//! measurements. SIMULATION ONLY: model energies, provisional settings.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_sim::report::{secs, timing_table};
use aed_sim::{catalogue, run, Scenario};
use aed_types::PatientMode;
use aed_waveform::WaveformConfig;
use common::{assert_invariants, fail};

fn one_shock(mode: PatientMode, ohms: f64) -> aed_sim::RunResult {
    let mut s = Scenario::base(&format!("energy-{mode:?}-{ohms}"), "energy check");
    s.mode = mode;
    s.pad_ohms = ohms;
    s.duration_ms = 40_000;
    let r = run(&s).unwrap();
    assert_invariants(&r);
    if r.shocks.len() != 1 {
        fail(&r, "expected exactly one shock");
    }
    r
}

fn check_mode(mode: PatientMode) {
    let target = WaveformConfig::target_joules(mode);
    let tolerance = WaveformConfig::default().energy_tolerance;
    println!("{mode:?} mode, target {target} J, tolerance ±{:.0}% (MODEL figures)", tolerance * 100.0);
    println!("| Ω | from V | delivered J | error | peak A | phase 1 ms | phase 2 ms | left V |");
    for ohms in [25.0, 50.0, 80.0, 125.0, 175.0] {
        let r = one_shock(mode, ohms);
        let s = r.shocks.first().unwrap();
        let res = s.result.unwrap();
        let err = (s.delivered_j() - target) / target;
        println!(
            "| {ohms:.0} | {:.0} | {:.1} | {:+.1}% | {:.1} | {:.2} | {:.2} | {:.0} |",
            s.capacitor_v,
            s.delivered_j(),
            err * 100.0,
            res.peak_current_a,
            res.phase1_s * 1e3,
            res.phase2_s * 1e3,
            res.final_capacitor_v
        );
        assert!(
            err.abs() <= tolerance,
            "{mode:?} at {ohms} Ω delivered {:.1} J, target {target} J",
            s.delivered_j()
        );
        assert!(res.peak_current_a <= WaveformConfig::default().max_peak_current_amps);
    }
}

#[test]
fn adult_energy_on_target_across_resistance_range() {
    check_mode(PatientMode::Adult);
}

#[test]
fn child_mode_gives_child_energy_across_resistance_range() {
    check_mode(PatientMode::Child);
}

#[test]
fn timing_measurements_are_reported() {
    let runs: Vec<_> = catalogue().iter().map(|s| run(s).unwrap()).collect();
    println!("Timings per shock (SIMULATED, provisional settings):\n");
    println!("{}", timing_table(&runs));

    let adult = runs.iter().find(|r| r.scenario.name == "adult-vf").unwrap();
    let t = adult.timings.first().unwrap();
    let pads_to_shock = adult.pads_on_to_first_shock_ms().unwrap();
    println!(
        "adult-vf: pads on -> shock {}, hands-off before shock {}",
        secs(pads_to_shock),
        t.hands_off_ms.map_or_else(|| "-".to_string(), secs)
    );
    // Loose sanity bounds only: these are measurements, not requirements.
    let cfg = adult.config;
    assert!(t.countdown_ms.abs_diff(cfg.countdown_ms) <= 50, "countdown {}", t.countdown_ms);
    assert!(t.charge_ms <= 8_000, "charge took {} ms (requirement: under 8 s)", t.charge_ms);
    assert!(t.analysis_ms <= cfg.analysis_time_limit_ms);
    assert!(pads_to_shock < 30_000);
}

/// The battery and heat cost of charging during analysis (D-29), per rescue
/// (SIMULATED, model energies): every analysis that ends without a shock
/// burns a full charge off in the internal load. Compared with charging
/// only once a shock is advised (the design before D-29; 5 s countdown).
#[test]
fn battery_and_heat_cost_of_charging_during_analysis_is_reported() {
    let before = aed_core::Config {
        countdown_ms: 5_000,
        charge_during_analysis: false,
        ..aed_core::Config::default()
    };
    let after = aed_core::Config::default();
    println!("Battery and heat cost per rescue (SIMULATED, model energies, 30 W charger):");
    println!("| scenario | length | design | analyses | charges burned | charger output J | burned off inside J |");
    for name in [
        "adult-vf",
        "child-vf",
        "adult-vf-two-shocks",
        "normal-rhythm",
        "asystole",
        "never-stops-compressing",
        "pads-190-throughout",
    ] {
        let s = aed_sim::by_name(name).unwrap();
        for (design, config) in [("before", before), ("after", after)] {
            let r = aed_sim::Simulation::with_config(&s, config).unwrap().run();
            assert_invariants(&r);
            let analyses = r.states.iter().filter(|&&(_, x)| x.is_analysing()).count();
            println!(
                "| {name} | {} | {design} | {analyses} | {} | {:.0} | {:.0} |",
                secs(s.duration_ms),
                r.charges_burned,
                r.charger_energy_j,
                r.dump_energy_j
            );
            if design == "after" && name == "normal-rhythm" {
                // One full adult charge (about 190 J) burned per analysis.
                assert_eq!(usize::from(r.charges_burned), analyses);
            }
        }
    }
}
