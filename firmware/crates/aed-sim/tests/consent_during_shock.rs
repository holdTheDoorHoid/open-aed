// SPDX-License-Identifier: GPL-3.0-or-later
//! Review finding R-1 and decision D-59: the shock is a process in time.
//!
//! These tests run the time-resolved plant (10 µs steps through the pulse):
//!
//! 1. the demonstration that the consent behaviour of the firmware before
//!    D-59 fails against a realistic plant (the relay moves the pads off the
//!    front ends before the pulse, so both pad readings go open, and the
//!    capacitor voltage leaves the "at plan" window within microseconds),
//!    and that the current firmware delivers;
//! 2. a consent line withdrawn at every instant across the whole sequence:
//!    the energy delivered is either nothing or the full plan, never
//!    anything in between, never more;
//! 3. every injected shock-path fault, with the worst-case energy.
//!
//! SIMULATION ONLY: a model of the firmware and of a circuit that has not
//! been built. The figures say nothing about real hardware.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    missing_docs
)]

use aed_core::{RescueState, SequenceEnd, SequenceStatus, ShockOutcome};
use aed_rhythm::synth::{Rhythm, Synth};
use aed_sim::plant::{ConsentLine, ShockEnd};
use aed_sim::{by_name, Fault, Plant, PlantConfig, RunResult, Scenario, ShockRecord, Simulation};
use aed_types::{Ohms, PatientMode, ShockPlan, ShockPlanner};
use aed_waveform::{BtePlanner, WaveformConfig};

// ----- a bare plant: the hardware alone, driven line by line ------------------

fn plan(mode: PatientMode, ohms: u16) -> ShockPlan {
    BtePlanner::new(WaveformConfig::default())
        .unwrap()
        .plan(mode, Ohms(ohms))
        .unwrap()
}

/// A plant charged to the plan's voltage, both holds on, pads on at `ohms`.
fn charged_plant(config: PlantConfig, ohms: f64, plan: &ShockPlan) -> Plant {
    let mut p = Plant::new(
        config,
        Synth::new(
            Rhythm::VentricularFibrillation {
                amplitude_uv: 800.0,
            },
            1,
        ),
        1,
    );
    p.pad_ohms = Some(ohms);
    p.main_hold = true;
    p.supervisor_hold = true;
    // Design v0.3: the holds take up to 35 ms to engage (FW-11) and the
    // coil supply must be up 200 ms before the relay is commanded (FW-12).
    p.set_coil_supply(true);
    for _ in 0..250 {
        let _ = p.step();
    }
    p.cap_v = f64::from(plan.charge_voltage.0);
    p
}

/// Since design v0.3 a sequence starts about 66 ms after the rail rises
/// (30 ms rail-ready wait, 15.9 ms relay transfer, 14 ms settle; simulated
/// band, aed-sim `HwTiming`), so consent is held this much longer.
const START_MS: u32 = 70;

/// Raise request and permit together, hold them for `hold_ms`, then drop
/// both and let the plant settle. Returns the shock record, if any, and
/// when the rail rose (µs).
fn raise_consent(p: &mut Plant, plan: &ShockPlan, hold_ms: u32) -> Option<ShockRecord> {
    p.request = Some(*plan);
    p.request_raised_us = Some(p.now_us());
    p.permit = true;
    for _ in 0..hold_ms + START_MS {
        let _ = p.step();
    }
    p.request = None;
    p.permit = false;
    for _ in 0..100 {
        let _ = p.step();
    }
    p.shocks.last().copied()
}

fn one_shock(
    config: PlantConfig,
    ohms: u16,
    cut: Option<(ConsentLine, u64)>,
) -> (Option<ShockRecord>, u64) {
    let pl = plan(PatientMode::Adult, ohms);
    let mut p = charged_plant(config, f64::from(ohms), &pl);
    let t0 = p.now_us();
    p.consent_cut = cut.map(|(l, at)| (l, t0 + at));
    (raise_consent(&mut p, &pl, 60), t0)
}

#[test]
fn a_full_sequence_delivers_the_plan_as_the_waveform_model_predicts() {
    println!("\nTime-resolved shock, adult plan, model figures (SIMULATION ONLY):");
    println!("| Ω | plan J | delivered J | pulse ms | consent to delivered ms | peak A |");
    println!("|---:|---:|---:|---:|---:|---:|");
    for ohms in [25u16, 50, 80, 125, 175] {
        let (s, t0) = one_shock(PlantConfig::default(), ohms, None);
        let s = s.expect("no shock");
        assert_eq!(s.end, ShockEnd::Completed);
        let err = (s.delivered_j() - s.planned_j()).abs() / s.planned_j();
        assert!(
            err < 0.002,
            "{ohms} Ω: {} J vs {} J",
            s.delivered_j(),
            s.planned_j()
        );
        println!(
            "| {ohms} | {:.1} | {:.1} | {:.2} | {:.2} | {:.1} |",
            s.planned_j(),
            s.delivered_j(),
            s.duration_us() as f64 / 1e3,
            s.end_us.saturating_sub(t0) as f64 / 1e3,
            s.peak_a()
        );
    }
}

/// Sweep a consent line cut across the whole sequence (from before the
/// request to after the end) in `step_us` steps.
fn sweep(config: PlantConfig, ohms: u16, line: ConsentLine, step_us: u64) -> (u32, u32, u32, f64) {
    let (full, _) = one_shock(config, ohms, None);
    let full = full.unwrap();
    let planned = full.delivered_j();
    let end = full.end_us - (full.rail_us.saturating_sub(5_000));
    let (mut none, mut whole, mut partial) = (0, 0, 0);
    let mut most = 0.0_f64;
    let mut at = 0;
    while at <= end + 5_000 {
        let (s, _) = one_shock(config, ohms, Some((line, at)));
        let j = s.map_or(0.0, |s| s.delivered_j());
        most = most.max(j);
        if j < 0.01 {
            none += 1;
        } else if (j - planned).abs() <= 0.001 * planned {
            whole += 1;
        } else {
            partial += 1;
        }
        at += step_us;
    }
    (none, whole, partial, most / planned)
}

#[test]
fn consent_withdrawn_at_any_instant_gives_nothing_or_the_whole_plan_never_part_and_never_more() {
    println!("\nConsent withdrawn at every instant (SIMULATION ONLY):");
    println!("| line | Ω | step µs | runs: nothing | whole plan | part | most / plan |");
    println!("|---|---:|---:|---:|---:|---:|---:|");
    for line in [ConsentLine::Request, ConsentLine::Permit] {
        for ohms in [25u16, 80, 175] {
            // 250 µs steps (was 50 µs): the sequence now lasts about 75 ms
            // from the rail; the pulse itself still gets 20–60 samples.
            let (none, whole, partial, most) = sweep(PlantConfig::default(), ohms, line, 250);
            println!("| {line:?} | {ohms} | 250 | {none} | {whole} | {partial} | {most:.4} |");
            assert_eq!(partial, 0, "{line:?} {ohms} Ω: part of a shock delivered");
            assert!(none > 0 && whole > 0);
            assert!(most <= 1.0005, "{line:?} {ohms} Ω: more than planned");
        }
    }
}

#[test]
fn the_first_design_cut_shocks_short_the_latched_design_does_not() {
    // The same sweep with the hardware of the first design (consent is
    // continuous power, review R-1): withdrawals during the pulse deliver
    // part of a shock.
    let old = PlantConfig {
        sequence_latch: false,
        ..PlantConfig::default()
    };
    let (none, whole, partial, _) = sweep(old, 80, ConsentLine::Permit, 50);
    // (50 µs steps here: the pulse must be sampled finely to show the cut.)
    println!("\nFirst design, permit withdrawn at every instant, 80 Ω: nothing {none}, whole {whole}, PART {partial} (SIMULATION ONLY)");
    assert!(partial > 100, "the old hardware should cut shocks short");
    let _ = whole;
}

#[test]
fn a_stuck_phase_timer_is_limited_by_the_hardware_time_cap() {
    // Since D-63 the phase-1 cap (11.37 ms nominal) ends it, before the
    // 25 ms overall cap.
    println!("\nPhase timer stuck in phase 1 (worst case), adult plan (SIMULATION ONLY):");
    println!("| Ω | plan J | with the phase-1 cap J | ratio | pulse ms | without any cap J |");
    println!("|---:|---:|---:|---:|---:|---:|");
    for ohms in [25u16, 50, 80, 125, 175] {
        let pl = plan(PatientMode::Adult, ohms);
        let mut p = charged_plant(PlantConfig::default(), f64::from(ohms), &pl);
        p.inject(Fault::MainTimerStuck);
        let s = raise_consent(&mut p, &pl, 80).unwrap();
        assert_eq!(s.end, ShockEnd::PhaseCap);
        assert!(s.duration_us() <= 11_380, "{} µs", s.duration_us());
        assert!(s.delivered_j() <= 0.5 * 100e-6 * 1950.0 * 1950.0);
        let uncapped = PlantConfig {
            latch_timing: aed_core::interlock::LatchTiming {
                time_cap_us: u32::MAX,
                phase1_cap_us: u32::MAX,
                phase2_cap_us: u32::MAX,
                ..aed_core::interlock::LatchTiming::PROPOSED
            },
            ..PlantConfig::default()
        };
        let mut q = charged_plant(uncapped, f64::from(ohms), &pl);
        q.inject(Fault::MainTimerStuck);
        let v0 = q.cap_v;
        let _ = raise_consent(&mut q, &pl, 400);
        // With no cap the sequence never ends: what has left the capacitor
        // went to the patient and the internal resistance in proportion.
        let share = f64::from(ohms) / (f64::from(ohms) + q.config.waveform.internal_ohms);
        let u = 0.5 * 100e-6 * (v0 * v0 - q.cap_v * q.cap_v) * share;
        println!(
            "| {ohms} | {:.1} | {:.1} | {:.2} | {:.1} | {:.1} |",
            s.planned_j(),
            s.delivered_j(),
            s.delivered_j() / s.planned_j(),
            s.duration_us() as f64 / 1e3,
            u
        );
    }
}

#[test]
fn a_toggling_phase_timer_cannot_hold_the_window_open() {
    // Review R-4: a retriggerable window would never close. The latch's cap
    // is measured from the first conducting moment.
    let pl = plan(PatientMode::Adult, 80);
    let mut p = charged_plant(PlantConfig::default(), 80.0, &pl);
    p.inject(Fault::MainTimerToggling);
    let s = raise_consent(&mut p, &pl, 200).unwrap();
    assert_eq!(s.end, ShockEnd::TimeCap);
    assert!(s.duration_us() <= 25_010);
    assert_eq!(p.shocks.len(), 1, "the lockout stops a second sequence");
    println!("\nToggling timer at 80 Ω: {:.1} J against {:.1} J planned, cut at {:.1} ms (SIMULATION ONLY)",
        s.delivered_j(), s.planned_j(), s.duration_us() as f64 / 1e3);
}

#[test]
fn over_current_is_tripped_by_the_hardware() {
    let pl = plan(PatientMode::Adult, 80);
    let mut p = charged_plant(PlantConfig::default(), 80.0, &pl);
    p.inject(Fault::OutputShort);
    let s = raise_consent(&mut p, &pl, 80).unwrap();
    assert_eq!(s.end, ShockEnd::OverCurrent);
    assert!(
        s.delivered_j() < 0.01,
        "into the patient: {} J",
        s.delivered_j()
    );
    assert!(s.duration_us() <= 20);
    println!("\nShorted output: tripped after {} µs at {:.0} A (model has no wiring inductance), {:.2} J into the patient (SIMULATION ONLY)",
        s.duration_us(), s.output_peak_a, s.delivered_j());
}

#[test]
fn a_second_sequence_within_one_permit_is_locked_out() {
    let pl = plan(PatientMode::Adult, 80);
    let mut p = charged_plant(PlantConfig::default(), 80.0, &pl);
    p.request = Some(pl);
    p.permit = true;
    for _ in 0..40 + START_MS {
        let _ = p.step();
    }
    assert_eq!(p.shocks.len(), 1);
    assert_eq!(
        p.latch.status,
        SequenceStatus::Ended(SequenceEnd::Completed)
    );
    // Charge again (as a fault would) and toggle the request with the
    // permit still up.
    p.cap_v = 1_950.0;
    for _ in 0..20 {
        p.request = None;
        let _ = p.step();
        p.request = Some(pl);
        for _ in 0..30 {
            let _ = p.step();
        }
    }
    assert_eq!(p.shocks.len(), 1, "a second sequence within one permit");
    assert!(p.latch.status.locked_out());
    // Only both lines down for the re-arm time clears it.
    p.request = None;
    p.permit = false;
    // Re-arm up to 60.34 ms plus the permit detector's drop-out (v0.3).
    for _ in 0..80 {
        let _ = p.step();
    }
    assert_eq!(p.latch.status, SequenceStatus::Idle);
}

#[test]
fn stuck_and_toggling_consent_lines_never_give_a_part_or_extra_shock() {
    println!("\nStuck and toggling consent lines, 80 Ω (SIMULATION ONLY):");
    for (fault, request, permit) in [
        (Fault::RequestStuckOn, false, true),
        (Fault::PermitStuckOn, true, false),
        (Fault::RequestToggling, true, true),
        (Fault::PermitToggling, true, true),
    ] {
        let pl = plan(PatientMode::Adult, 80);
        let mut p = charged_plant(PlantConfig::default(), 80.0, &pl);
        p.inject(fault);
        p.request = if request || fault == Fault::RequestStuckOn {
            Some(pl)
        } else {
            None
        };
        p.permit = permit;
        for _ in 0..400 {
            let _ = p.step();
        }
        p.request = None;
        p.permit = false;
        for _ in 0..100 {
            let _ = p.step();
        }
        let j: Vec<f64> = p.shocks.iter().map(ShockRecord::delivered_j).collect();
        println!(
            "  {fault:?}: sequences {}, energies {j:.1?}",
            p.shocks.len()
        );
        assert!(p.shocks.len() <= 1, "{fault:?}");
        for s in &p.shocks {
            assert_eq!(s.end, ShockEnd::Completed, "{fault:?}");
            assert!(
                (s.delivered_j() - s.planned_j()).abs() < 0.002 * s.planned_j(),
                "{fault:?}"
            );
        }
        // A request line toggling at 1 kHz never holds the rail up long
        // enough for the relay to move: no shock at all. (A toggling PERMIT
        // line is what the supervisor's permit clock is: the dynamic
        // detector, modelled since v0.3, holds consent through it, so it
        // gives the whole plan once, which the checks above cover.)
        if matches!(fault, Fault::RequestToggling) {
            assert!(p.shocks.is_empty(), "{fault:?}");
        }
    }
}

#[test]
fn the_hardware_permit_cap_drops_a_rail_held_without_a_sequence() {
    let pl = plan(PatientMode::Adult, 80);
    let mut p = charged_plant(PlantConfig::default(), 80.0, &pl);
    p.inject(Fault::BridgeFailsToFire);
    p.request = Some(pl);
    p.permit = true;
    let t0 = p.now_ms;
    while p.latch.status != SequenceStatus::LockedOut && p.now_ms < t0 + 1_000 {
        let _ = p.step();
    }
    assert_eq!(p.latch.status, SequenceStatus::LockedOut);
    let held = p.now_ms - t0;
    // The v0.3 window: 150 ms nominal (FW-5).
    assert!((150..=152).contains(&held), "rail held {held} ms");
    assert!(p.shocks.is_empty());
}

// ----- whole system: both controllers against the plant ------------------------

fn short_vf() -> Scenario {
    let mut s = by_name("adult-vf").unwrap();
    s.duration_ms = 23_000;
    s
}

fn run_sim(mut sim: Simulation) -> RunResult {
    while sim.plant().borrow().now_ms < 23_000 {
        sim.step();
    }
    sim.into_result()
}

fn outcome(r: &RunResult) -> Option<String> {
    r.timeline
        .iter()
        .find(|e| {
            e.text.contains("delivered")
                || e.text.contains("withdrawn")
                || e.text.contains("did not")
                || e.text.contains("Interlock")
        })
        .map(|e| e.text.clone())
}

#[test]
fn the_old_consent_behaviour_fails_against_a_realistic_plant_and_the_new_one_delivers() {
    // Converters that sample every 8 ms (the review's figure for the
    // chosen converter), with the sampling grid at each of its 8 phases.
    println!("\nR-1 demonstration, adult VF at 80 Ω, one shock expected (SIMULATION ONLY):");
    println!("| firmware consent | hardware | sample phase | shocks | energy J | how it ended |");
    println!("|---|---|---:|---:|---:|---|");
    let mut old_ok = 0;
    let mut new_ok = 0;
    let mut cut_short = 0;
    let mut refused = 0;
    let mut runs = 0;
    for (legacy, latch) in [
        ((true, true), false),
        ((true, true), true),
        ((false, true), false),
        ((false, true), true),
        ((false, false), true),
    ] {
        for phase in 0..8 {
            let mut s = short_vf();
            s.plant.voltage_sample_ms = 8;
            s.plant.pad_sample_ms = 8;
            s.plant.sample_phase_ms = phase;
            s.plant.sequence_latch = latch;
            let mut sim = Simulation::new(&s).unwrap();
            sim.set_legacy_consent_parts(legacy.0, legacy.1);
            let r = run_sim(sim);
            let j: f64 = r.shocks.iter().map(ShockRecord::delivered_j).sum();
            let full = r
                .shocks
                .iter()
                .any(|s| s.end == ShockEnd::Completed && s.delivered_j() > 140.0);
            println!(
                "| {} | {} | {phase} | {} | {j:.1} | {} |",
                match legacy {
                    (true, true) => "both before D-59",
                    (false, true) => "supervisor before D-59",
                    _ => "D-59",
                },
                if latch {
                    "latched (D-59)"
                } else {
                    "first design"
                },
                r.shocks.len(),
                r.shocks.first().map_or_else(
                    || outcome(&r).unwrap_or_default(),
                    |s| format!("{:?}", s.end)
                )
            );
            if legacy.0 || legacy.1 {
                runs += 1;
                old_ok += u32::from(full);
                cut_short += u32::from(r.shocks.iter().any(|s| s.end == ShockEnd::ConsentCut));
                refused += u32::from(r.shocks.is_empty());
            } else {
                new_ok += u32::from(full);
                assert!(full, "phase {phase}: new firmware did not deliver");
                assert!(r.violations.is_empty(), "{:?}", r.violations);
            }
        }
    }
    println!("old consent behaviour: {old_ok} of {runs} runs delivered a full shock ({refused} no shock at all, {cut_short} cut short mid-pulse); new: {new_ok} of 8");
    // The old main controller's rule alone (with both) blocks every shock;
    // the old supervisor's rule alone blocks it or, with the first design's
    // hardware, cuts it short; with the latched hardware some of those
    // complete, which is exactly why the latch alone is not enough.
    assert!(
        refused >= 16,
        "the old consent behaviour should block shocks"
    );
    // Since v0.3 the start comes 14 ms after the relay leaves rest, so the
    // old supervisor's rule (withdraw when its pads read open) now always
    // withdraws BEFORE the start: it blocks the shock rather than cutting
    // it short. Either way it fails.
    assert!(
        cut_short + refused > 0 && old_ok < runs,
        "the old supervisor rule should fail"
    );
}

#[test]
fn a_controller_withdrawing_at_each_instant_of_a_real_rescue_gives_nothing_or_the_whole_plan() {
    // Whole system: run to the shock request, then force one consent line
    // low at an instant swept from the request to past the end.
    let mut results = Vec::new();
    for line in [ConsentLine::Request, ConsentLine::Permit] {
        for offset_us in (0..=24_000u64).step_by(1_500) {
            let s = short_vf();
            let mut sim = Simulation::new(&s).unwrap();
            while sim.rescue().state() != RescueState::Delivering {
                sim.step();
            }
            {
                let mut p = sim.plant().borrow_mut();
                let at = p.now_us() + offset_us;
                p.consent_cut = Some((line, at));
            }
            let r = run_sim(sim);
            let j: f64 = r.shocks.iter().map(ShockRecord::delivered_j).sum();
            let planned: f64 = r.shocks.iter().map(ShockRecord::planned_j).sum();
            results.push((line, offset_us, r.shocks.len(), j, planned));
            assert!(r.shocks.len() <= 1);
            assert!(
                j < 0.01 || (j - planned).abs() < 0.002 * planned,
                "{line:?} +{offset_us} µs: {j} J"
            );
            assert!(r.violations_of(aed_sim::Rule::ShockNotAsPlanned).is_empty());
            // Never a fault from a withdrawal: either delivered, or "not delivered" and CPR.
            assert_ne!(
                r.final_state,
                RescueState::Fault,
                "{line:?} +{offset_us} µs"
            );
        }
    }
    println!("\nWhole system, consent withdrawn after the request (SIMULATION ONLY):");
    for (line, off, n, j, _) in results {
        println!(
            "  {line:?} cut at +{:.1} ms: {n} shock(s), {j:.1} J",
            off as f64 / 1e3
        );
    }
}

fn outcome_event(r: &RunResult, want: &str) -> bool {
    r.timeline.iter().any(|e| e.text.contains(want))
}

#[test]
fn relay_faults_in_a_rescue() {
    // Fails to transfer: not delivered, CPR.
    let mut s = short_vf();
    // (After the switch-on self-test, which would find it: T-3, D-62.)
    s.script.push((
        aed_sim::Trigger::At(3_000),
        aed_sim::Action::Fault(Fault::RelayFailsToTransfer),
    ));
    let r = aed_sim::run(&s).unwrap();
    assert!(r.shocks.is_empty());
    assert!(outcome_event(&r, "Patient relay did not reach the bridge"));
    assert_ne!(r.final_state, RescueState::Fault);
    // Fails to return: the shock is delivered, then the relay fault.
    let mut s = short_vf();
    s.script.push((
        aed_sim::Trigger::At(2_000),
        aed_sim::Action::Fault(Fault::RelayFailsToReturn),
    ));
    let r = aed_sim::run(&s).unwrap();
    assert_eq!(r.shocks.len(), 1);
    assert_eq!(r.final_fault, Some(aed_core::FaultKind::PatientRelayStuck));
    // Slow (four times the model times, 64 ms to transfer): since v0.3 the
    // firmware gives up 40 ms after RELAY_CMD (FW-1), and the switch-on
    // self-test finds it (T-3).
    let mut s = short_vf();
    s.script.push((
        aed_sim::Trigger::At(0),
        aed_sim::Action::Fault(Fault::RelaySlow),
    ));
    let r = aed_sim::run(&s).unwrap();
    assert!(r.shocks.is_empty());
    assert_eq!(
        r.final_fault,
        Some(aed_core::FaultKind::SelfTest(
            aed_core::SelfTestFailure::RelayDidNotMove
        ))
    );
    let _ = ShockOutcome::Delivered;
}

#[test]
fn emptying_through_one_path_while_the_supervisor_still_holds_is_not_a_disagreement() {
    // Campaign seed 1758 (extended): a hands-on hold-back, then the main
    // controller analyses again while the capacitor empties through path A
    // only (the supervisor holds path B for the new arming), on a slow
    // wire. Before the main controller reported "emptying" in its status,
    // the supervisor latched a voltage disagreement here.
    let s = aed_sim::campaign::random_scenario_with(
        1758,
        aed_sim::campaign::CampaignOptions {
            extended: true,
            ..Default::default()
        },
    );
    let r = aed_sim::run(&s).unwrap();
    assert_ne!(
        r.supervisor_fault,
        Some(aed_core::SupervisorFault::VoltageDisagreement),
        "{:?}",
        r.final_fault
    );
    assert!(r.violations.is_empty(), "{:?}", r.violations);
}
