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
    // No shock had been advised yet, so the one instruction is "Attach pads"
    // (finding F-6); the charge from the analysis is burned off.
    let r = standard("pads-off-during-charge");
    assert!(r.said(Prompt::AttachPads));
    let shock_at = r.shocks.first().unwrap().at_ms;
    let charges_before_shock = r
        .timeline
        .iter()
        .filter(|e| e.t_ms < shock_at && e.text.starts_with("Charging to"))
        .count();
    assert_eq!(charges_before_shock, 2, "aborted charge, then a new one after re-attaching");
    // The aborted charge, and the one from the analysis after the shock
    // (normal rhythm), were burned off.
    assert_eq!(usize::from(r.charges_burned), r.charges_started as usize - 1);
}

#[test]
fn pads_off_during_countdown() {
    // The shock was advised and is abandoned: "Check pads." then "Attach
    // pads", never "Start CPR" in between (finding F-6).
    let r = standard("pads-off-during-countdown");
    let p: Vec<Prompt> = r.prompts.iter().map(|&(_, p)| p).collect();
    let i = p.iter().position(|&x| x == Prompt::CheckPads).unwrap();
    assert_eq!(p.get(i + 1), Some(&Prompt::AttachPads));
    let before_shock: Vec<Prompt> = r
        .prompts
        .iter()
        .take_while(|&&(t, _)| t < r.shocks.first().unwrap().at_ms)
        .map(|&(_, p)| p)
        .collect();
    assert!(!before_shock.contains(&Prompt::StartCpr));
}

#[test]
fn pads_190_throughout() {
    // D-32: pads on but above the shock range. The device analyses, asks
    // for the pads to be pressed, gives no shock and sends the responder to
    // CPR. Never silent, and never only "attach pads".
    let r = standard("pads-190-throughout");
    assert!(r.shocks.is_empty());
    let pads_on = r.pads_on_at_ms.unwrap();
    assert!(
        !r.prompts.iter().any(|&(t, p)| p == Prompt::AttachPads && t > pads_on),
        "the pads are on: never 'attach pads' after that"
    );
    assert!(r.said(Prompt::PressPadsFirmly));
    assert!(r.said(Prompt::PadContactPoorNoShock));
    assert!(r.entered(RescueState::AnalysingCharging));
    assert!(r.entered(RescueState::Cpr));
    let p: Vec<Prompt> = r.prompts.iter().map(|&(_, p)| p).collect();
    for (i, x) in p.iter().enumerate() {
        if *x == Prompt::PadContactPoorNoShock {
            assert_eq!(p.get(i + 1), Some(&Prompt::StartCpr));
        }
    }
    // Every charge was burned off and the capacitor ended empty.
    assert!(r.charges_burned >= 1);
    assert!(r.final_cap_v < f64::from(r.config.empty_voltage.0));
}

#[test]
fn pads_190_pressed_to_150() {
    let r = standard("pads-190-pressed-to-150");
    assert!(r.said(Prompt::PressPadsFirmly));
    let s = r.shocks.first().unwrap();
    assert_eq!(s.patient_ohms, Some(150.0));
    assert!(!r.said(Prompt::PadContactPoorNoShock));
}

#[test]
fn stray_cancel_at_switch_on() {
    // D-31: ignored during the self-test; the device then works normally.
    let r = standard("stray-cancel-at-switch-on");
    assert!(!r.said(Prompt::DeviceFaultContinueCpr));
    assert!(r.timeline.iter().any(|e| e.text.contains("ignored (D-31)")));
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
    // The first analysis was charging (D-29); cancel ended it before any
    // shock was advised, and that charge was burned off.
    let first_cpr = r.states.iter().find(|&&(_, s)| s == RescueState::Cpr).unwrap().0;
    assert!(!r.states.iter().any(|&(t, s)| s == RescueState::Countdown && t < first_cpr));
}

#[test]
fn fault_cancel_stuck() {
    // D-31: a cancel stuck down is a fault with its own prompt, from both
    // controllers' own views of the button.
    let r = standard("fault-cancel-stuck");
    assert!(matches!(
        r.final_fault,
        Some(FaultKind::CancelStuck | FaultKind::Supervisor(SupervisorFault::CancelStuck))
    ));
    assert_eq!(r.last_prompt(), Some(Prompt::CancelStuckContinueCpr));
    assert_eq!(r.supervisor_fault, Some(SupervisorFault::CancelStuck));
    let stuck_from = r.injected.first().unwrap().0;
    assert!(r.shocks.iter().all(|s| s.at_ms < stuck_from));
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

#[test]
fn charger_is_never_asked_for_before_the_supervisor_holds() {
    // D-29 with the hardware rule D-25: at every analysis start (the first
    // straight after the self-test or after the pads go on, and after each
    // CPR period), the main controller asks for the charger only once the
    // supervisor holds the discharge path, including on a slow message wire
    // and with drifting clocks.
    for name in ["adult-vf", "adult-vf-two-shocks", "child-vf", "normal-rhythm", "slow-link", "clock-drift"] {
        let s = aed_sim::by_name(name).unwrap();
        let mut sim = aed_sim::Simulation::new(&s).unwrap();
        let mut asked = 0;
        while sim.plant().borrow().now_ms < s.duration_ms {
            sim.step();
            let p = sim.plant().borrow();
            let charging_for_a_shock = p.charger_target.is_some_and(|v| v.0 > 1_000);
            if charging_for_a_shock {
                asked += 1;
                assert!(
                    p.supervisor_hold,
                    "{name}: t={} charger asked for while the supervisor is not holding",
                    p.now_ms
                );
            }
        }
        assert!(asked > 0, "{name}: never charged");
    }
}

// ----- compression sensing (D-36, finding F-12) -------------------------------------

/// The same scenario with compression sensing switched off in the firmware.
fn without_sensing(name: &str) -> aed_sim::RunResult {
    let mut s = aed_sim::by_name(name).unwrap();
    s.chest_sensing = false;
    common::run_scenario(&s)
}

#[test]
fn f12_compressions_150_over_flatline() {
    // With compression sensing: never charged for a shock, never shocked.
    let r = standard("f12-compressions-150-over-flatline");
    assert!(r.said(Prompt::AnalysisInconclusive));
    assert!(r.compressions_counted > 100);
    // Without it (the old behaviour): compressions at 150/min over a
    // flatline are taken for a shockable rhythm and the flatline is shocked.
    let old = without_sensing("f12-compressions-150-over-flatline");
    assert!(
        !old.shocks.is_empty(),
        "expected the heart-signal-only device to shock (finding F-12)"
    );
    assert_eq!(
        old.violations_of(aed_sim::Rule::FiredOnNonShockableRhythm).len(),
        old.shocks.len()
    );
}

#[test]
fn f12_compressions_150_over_normal_rhythm() {
    let r = standard("f12-compressions-150-over-normal-rhythm");
    assert!(r.shocks.is_empty());
    let old = without_sensing("f12-compressions-150-over-normal-rhythm");
    println!("without sensing: {} shock(s) on a normal rhythm", old.shocks.len());
}

#[test]
fn stops_very_late() {
    // The responder stops 3.5 s after being told: the analysis waits for
    // hands off, and the shock still comes in the first cycle. (About 4 s is
    // the limit now; see tests/late_hands_off.rs.)
    let r = standard("stops-very-late");
    assert_eq!(r.shocks.len(), 1);
    let first_cpr = r.states.iter().find(|&&(_, s)| s == RescueState::Cpr).map(|&(t, _)| t);
    assert!(r.shocks.first().unwrap().at_ms <= first_cpr.unwrap_or(u32::MAX), "shock in the first cycle");
    assert!(r.hands_on_at_fire.is_empty());
}

#[test]
fn touch_during_countdown() {
    // A firm touch during the countdown holds the shock back; the rhythm is
    // analysed again and the shock given once hands are off.
    let r = standard("touch-during-countdown");
    assert!(r.said(Prompt::MovementShockDelayed));
    assert_eq!(r.hands_on_aborts, 1);
    assert!(r.hands_on_at_fire.is_empty(), "no shock while touching");
    // Without sensing the touch is invisible to the device.
    let old = without_sensing("touch-during-countdown");
    println!("without sensing: shocks fired with hands on after {:?} ms", old.hands_on_at_fire);
}

#[test]
fn ventilations_during_analysis() {
    // Breaths every 6 s (no compressions) must not block the analysis.
    let r = standard("ventilations-during-analysis");
    let t = r.timings.first().unwrap();
    assert!(t.analysis_ms <= 10_000, "analysis took {} ms", t.analysis_ms);
}

#[test]
fn strong_pulse() {
    // A strong heartbeat in the impedance is not read as compressions.
    let r = standard("strong-pulse");
    let held: Vec<_> = r
        .timeline
        .iter()
        .filter(|e| e.text.starts_with("Chest being compressed"))
        .collect();
    assert!(held.is_empty(), "{held:?}");
}

#[test]
fn fast_rhythm_with_pulse() {
    let r = standard("fast-rhythm-with-pulse");
    assert!(r.shocks.is_empty());
    assert!(!r.timeline.iter().any(|e| e.text.starts_with("Chest being compressed")));
}

#[test]
fn small_compressions() {
    // Documented weakness (real recordings): compressions of 0.2 ohm are
    // below the detector's 0.3 ohm threshold and are not seen. The device
    // then behaves as without sensing during them (the heart-signal checks
    // still apply); this test records it rather than hiding it.
    let r = standard("small-compressions");
    println!("compressions counted: {}", r.compressions_counted);
    assert!(r.compressions_counted < 10);
}

#[test]
fn fault_impedance_lost() {
    // The compression-sensing channel dies at switch-on: the analysis is
    // held for 5 s, then the device carries on from the heart signal alone
    // and the shock is given.
    let r = standard("fault-impedance-lost");
    assert!(r
        .timeline
        .iter()
        .any(|e| e.text.starts_with("Compression sensing FAILED")));
}
