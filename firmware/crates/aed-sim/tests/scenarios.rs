// SPDX-License-Identifier: GPL-3.0-or-later
//! One test per named scenario: the declared safety outcome, every
//! invariant, and the scenario-specific checks. SIMULATION ONLY.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_core::{FaultKind, Prompt, RescueState, SupervisorFault};
use aed_sim::runner::Source;
use aed_sim::catalogue;
use common::{assert_expected, assert_invariants, fail, run_named};

fn standard(name: &str) -> aed_sim::RunResult {
    let r = run_named(name);
    assert_invariants(&r);
    assert_expected(&r);
    r
}

#[test]
fn every_catalogue_scenario_has_a_test_here() {
    // Keeps this file and the catalogue in step.
    let src = include_str!("scenarios.rs");
    for s in catalogue() {
        assert!(
            src.contains(&format!("\"{}\"", s.name)),
            "scenario {} has no test",
            s.name
        );
    }
}

#[test]
fn adult_vf() {
    let r = standard("adult-vf");
    let s = r.shocks.first().unwrap();
    assert!((s.delivered_j() - 150.0).abs() < 150.0 * 0.15);
    // After the shock: CPR, re-analysis, no shock advised, CPR.
    assert!(r.said(Prompt::ShockDelivered));
    assert!(r.said(Prompt::NoShockAdvised));
    assert_eq!(r.final_state, RescueState::Cpr);
}

#[test]
fn adult_vf_two_shocks() {
    let r = standard("adult-vf-two-shocks");
    assert!(r.shocks.iter().all(|s| (s.delivered_j() - 150.0).abs() < 22.5));
}

#[test]
fn child_vf() {
    let r = standard("child-vf");
    let s = r.shocks.first().unwrap();
    assert!((s.delivered_j() - 50.0).abs() < 50.0 * 0.15, "child energy {}", s.delivered_j());
    assert!(r.said(Prompt::ChildModeFrontBack));
}

#[test]
fn normal_rhythm() {
    let r = standard("normal-rhythm");
    assert!(r.said(Prompt::NoShockAdvised));
}

#[test]
fn asystole() {
    standard("asystole");
}

#[test]
fn vf_converts_during_countdown() {
    let r = standard("vf-converts-during-countdown");
    assert!(r.said(Prompt::RhythmChangedNoShock));
}

#[test]
fn cancel_during_countdown() {
    let r = standard("cancel-during-countdown");
    assert!(r.said(Prompt::ShockCancelled));
}

#[test]
fn pads_off_during_charge() {
    let r = standard("pads-off-during-charge");
    assert!(r.said(Prompt::CheckPads));
    assert!(r.said(Prompt::AttachPads));
    assert_eq!(r.charges_started, 2, "aborted charge, then a new one after re-attaching");
}

#[test]
fn high_resistance() {
    standard("high-resistance");
}

#[test]
fn low_resistance() {
    standard("low-resistance");
}

#[test]
fn noisy() {
    // Invariants only: whether noisy VF is shocked is reported, not asserted.
    let r = standard("noisy");
    println!("noisy VF shocks: {}", r.shocks.len());
}

#[test]
fn rapid_vt() {
    standard("rapid-vt");
}

#[test]
fn fine_vf() {
    standard("fine-vf");
}

#[test]
fn slow_hands_off() {
    // Compressions run 2.5 s into the analysis: "not sure" at first, then
    // the shock once the signal is clean (finding F-1, fixed).
    let r = standard("slow-hands-off");
    assert!(r.said(Prompt::DoNotTouchPatient));
    let shock_at = r.shocks.first().unwrap().at_ms;
    assert!(
        r.prompts
            .iter()
            .all(|&(t, p)| p != Prompt::NoShockAdvised || t > shock_at),
        "'No shock advised' before the shock"
    );
}

#[test]
fn never_stops_compressing() {
    // Every analysis is "not sure" until its time limit, then CPR: no shock,
    // and the responder is never left without an instruction.
    let r = standard("never-stops-compressing");
    assert!(r.said(Prompt::DoNotTouchPatient));
    assert!(r.said(Prompt::AnalysisInconclusive));
    assert!(!r.said(Prompt::NoShockAdvised), "'not sure' must not be reported as 'no shock advised'");
    let p: Vec<Prompt> = r.prompts.iter().map(|&(_, p)| p).collect();
    for (i, x) in p.iter().enumerate() {
        if *x == Prompt::AnalysisInconclusive {
            assert_eq!(p.get(i + 1), Some(&Prompt::StartCpr), "inconclusive must be followed by Start CPR");
        }
    }
    // Each analysis lasts the full time limit (the decision arrives on a 1 s step).
    let limit = r.config.analysis_time_limit_ms;
    for w in r.states.windows(2) {
        if let [(t0, RescueState::Analysing), (t1, next)] = w {
            assert_eq!(*next, RescueState::Cpr);
            assert!((limit..=limit + 10).contains(&(t1 - t0)), "analysis took {} ms", t1 - t0);
        }
    }
    assert!(r.entered(RescueState::Cpr));
}

#[test]
fn clock_drift() {
    standard("clock-drift");
}

#[test]
fn slow_link() {
    let r = standard("slow-link");
    println!("slow-link shocks: {}", r.shocks.len());
}

#[test]
fn sensor_tolerance() {
    standard("sensor-tolerance");
}

#[test]
fn fault_main_voltage_sensor() {
    let r = standard("fault-main-voltage-sensor");
    assert!(r.supervisor_fault.is_some() || r.final_fault.is_some());
}

#[test]
fn fault_supervisor_voltage_sensor() {
    let r = standard("fault-supervisor-voltage-sensor");
    assert_eq!(r.supervisor_fault, Some(SupervisorFault::VoltageDisagreement));
}

#[test]
fn fault_charger_weak() {
    let r = standard("fault-charger-weak");
    assert_eq!(r.final_fault, Some(FaultKind::ChargeTimeout));
}

#[test]
fn fault_charger_runaway() {
    let r = standard("fault-charger-runaway");
    let limit = f64::from(r.config.overvoltage_limit.0);
    assert!(r.max_cap_v <= limit + aed_sim::runner::OVERVOLTAGE_MARGIN_V);
}

#[test]
fn fault_discharge_stuck() {
    let r = standard("fault-discharge-stuck");
    // The shock still goes; the residual cannot be emptied, which is the serious fault.
    assert_eq!(r.final_fault, Some(FaultKind::CapacitorNotEmptied));
    assert_eq!(r.last_prompt(), Some(Prompt::SeriousFaultContinueCpr));
    assert!(r.final_cap_v >= f64::from(r.config.empty_voltage.0));
}

#[test]
fn fault_main_freeze() {
    // Finding F-3 (fixed): the charger cannot run while the discharge path
    // is active, so the supervisor letting go empties the capacitor; and the
    // supervisor's own alarm tells the responder.
    let r = standard("fault-main-freeze");
    assert_eq!(r.supervisor_fault, Some(SupervisorFault::HeartbeatLost));
    assert!(r.final_cap_v < f64::from(r.config.empty_voltage.0));
    assert!(r.supervisor_alarm_at.is_some(), "supervisor alarm never came on");
    // The supervisor lets go within the heartbeat timeout (+ one heartbeat period).
    let frozen_at = r.injected.first().unwrap().0;
    let released = r
        .timeline
        .iter()
        .find(|e| {
            e.t_ms >= frozen_at
                && e.source == Source::Supervisor
                && e.text.contains("Lets go of the discharge path")
        })
        .unwrap_or_else(|| fail(&r, "supervisor never let go"));
    let limit = r.config.heartbeat_timeout_ms + r.config.heartbeat_period_ms;
    assert!(
        released.t_ms - frozen_at <= limit,
        "released after {} ms",
        released.t_ms - frozen_at
    );
}

#[test]
fn fault_link_drop() {
    let r = standard("fault-link-drop");
    assert_eq!(r.final_fault, Some(FaultKind::SupervisorLinkLost));
}

#[test]
fn fault_pads_off() {
    let r = standard("fault-pads-off");
    assert_eq!(r.final_state, RescueState::AttachPads);
}

#[test]
fn fault_cancel() {
    let r = standard("fault-cancel");
    assert!(r.said(Prompt::AnalysisCancelled));
    assert_eq!(r.charges_started, 0);
}

#[test]
fn fault_battery_depleted() {
    let r = standard("fault-battery-depleted");
    assert_eq!(r.final_fault, Some(FaultKind::BatteryDepleted));
}

#[test]
fn fault_bridge_fails() {
    let r = standard("fault-bridge-fails");
    assert!(r.said(Prompt::ShockNotDelivered));
    let empty = f64::from(r.config.empty_voltage.0);
    assert!(r.final_cap_v < empty);
}

#[test]
fn simulation_is_deterministic() {
    let a = run_named("noisy");
    let b = run_named("noisy");
    assert_eq!(a.timeline, b.timeline);
}
