// SPDX-License-Identifier: GPL-3.0-or-later
//! Integration problems found by running the three firmware parts together.
//!
//! Each `#[ignore]`d test states what SHOULD happen and FAILS today. They are
//! ignored so the suite stays green while still listing them; run them with
//! `cargo test -p aed-sim --test findings -- --ignored --nocapture`.
//! The fixes belong in other crates (or in hardware design decisions), not here.
//! A finding's test is un-parked (no longer ignored) once it is fixed: F-1,
//! F-2 and F-3 are fixed (their wider tests are in `late_hands_off.rs`,
//! `voltage_sensors.rs` and `frozen_main.rs`), and F-5 to F-9 were fixed
//! with the owner decisions of 27 November 2026 (D-28, D-31, D-32). Only F-4
//! is still parked; its exposure is measured by
//! `measure_how_close_to_delivery_a_rhythm_change_is_still_caught`.
//! F-12 (fast compressions over a flatline read as shockable) passes in
//! simulation with compression sensing (D-36); its wider tests are in
//! `compression_sensing.rs`.
//! SIMULATION ONLY.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_core::{Config, PermitRefusal, Prompt, RescueState};
use aed_rhythm::synth::Rhythm;
use aed_sim::runner::silence_grace_ms;
use aed_sim::{by_name, run, Action, Rule, Scenario, Trigger};
use common::{fail, run_scenario};

/// Finding 1 (FIXED). A responder who takes 2.5 s to stop compressions
/// after "Analysing. Do not touch the patient." makes the first analysis
/// window contain compression artifact, and the analyser answers
/// `Indeterminate`. `Rescue` used to act on that first answer ("No shock
/// advised", two minutes of CPR, every cycle). Now "not sure" is not final
/// while analysing: the device repeats "Do not touch the patient" and keeps
/// analysing until the signal is clean or the analysis time limit runs out.
#[test]
fn vf_is_shocked_even_if_compressions_stop_late() {
    let r = run_scenario(&by_name("slow-hands-off").unwrap());
    if r.shocks.is_empty() {
        fail(
            &r,
            "VF patient never shocked because compressions stopped 2.5 s late",
        );
    }
}

/// Measurement for finding 1: which reaction times still lead to a shock.
#[test]
fn measure_how_late_compressions_may_stop() {
    println!("Shock delivered within 40 s, by responder reaction time (SIMULATED):");
    println!("| VF size µV | 0.5 s | 0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 | 2.5 | 3.0 |");
    for amp in [400.0_f32, 800.0, 1_500.0] {
        let mut row = format!("| {amp:.0} |");
        for reaction in [500, 750, 1_000, 1_250, 1_500, 1_750, 2_000, 2_500, 3_000] {
            let mut s = Scenario::base("reaction", "reaction sweep");
            s.rhythm = Rhythm::VentricularFibrillation { amplitude_uv: amp };
            s.responder.reaction_ms = reaction;
            s.duration_ms = 40_000;
            let r = run(&s).unwrap();
            assert!(r.violations.is_empty());
            row.push_str(if r.shocks.is_empty() {
                " no |"
            } else {
                " yes |"
            });
        }
        println!("{row}");
    }
}

/// Finding 2 (FIXED). Two healthy voltage sensors with opposite errors
/// tripped the old single 50 V `voltage_tolerance`. The agreement budget now
/// grows with the voltage (`Config::agree_percent` + `Config::agree_abs_volts`,
/// per sensor, twice that between the two), separate from the "at the
/// planned voltage" window. Since D-28 the sensors are specified to
/// ±1 % ± 7 V (it was ±2 % ± 10 V when this finding was fixed), so this
/// checks opposite errors at the edge of THAT specification.
#[test]
fn in_spec_sensor_errors_do_not_disable_the_device() {
    let mut s = by_name("adult-vf").unwrap();
    s.plant.main_voltage_sensor.gain = 1.01;
    s.plant.main_voltage_sensor.offset = 7.0;
    s.plant.supervisor_voltage_sensor.gain = 0.99;
    s.plant.supervisor_voltage_sensor.offset = -7.0;
    let r = run_scenario(&s);
    if r.shocks.len() != 1 {
        fail(
            &r,
            "in-spec sensor errors (±1 % ± 7 V) made the device fault",
        );
    }
}

/// Finding 3a (FIXED by a hardware rule). If the main controller freezes
/// while charging or counting down, its last command to the charger stays
/// latched ON. Without a rule, the charger keeps pushing 30 W into the 2 kΩ
/// discharge load after the supervisor lets go, holding the capacitor at
/// about sqrt(30 W x 2,000 Ω) = 245 V. The rule
/// `aed_core::interlock::charger_may_run` (charger cut off whenever the
/// discharge path is active) is now the plant's default.
#[test]
fn frozen_main_leaves_capacitor_empty() {
    let r = run_scenario(&by_name("fault-main-freeze").unwrap());
    if !r.violations_of(Rule::CapacitorLeftCharged).is_empty() {
        fail(&r, "capacitor left charged after the main controller froze");
    }
}

/// Companion to 3a: with the charger disabled in hardware whenever the
/// discharge path is active (now the default), the capacitor empties.
#[test]
fn frozen_main_capacitor_empties_if_charger_is_gated_by_discharge_path() {
    let s = by_name("fault-main-freeze").unwrap();
    assert!(
        s.plant.charger_gated_by_discharge,
        "the hardware rule is the default"
    );
    let r = run_scenario(&s);
    assert!(r.violations.is_empty(), "{:?}", r.violations);
    assert!(r.final_cap_v < f64::from(r.config.empty_voltage.0));
}

/// Finding 3b (FIXED). A frozen main controller falls silent mid-countdown
/// (last words "Three."). The supervisor now has its own fault alarm (a
/// buzzer and light it alone drives, `SupervisorHardware::set_fault_alarm`),
/// on whenever it has a latched fault. The main controller still cannot
/// speak, so the check is the supervisor's alarm, which the never-silent
/// rule counts as "continue CPR".
#[test]
fn frozen_main_still_tells_responder_to_continue_cpr() {
    let r = run_scenario(&by_name("fault-main-freeze").unwrap());
    let frozen_at = r.main_frozen_at.unwrap();
    let grace = silence_grace_ms(&r.config);
    let told = r
        .supervisor_alarm_at
        .is_some_and(|t| t >= frozen_at && t - frozen_at <= grace);
    if !told || !r.violations_of(Rule::LeftWithoutInstruction).is_empty() {
        fail(&r, "device silent after main froze");
    }
    // The responder, told by the alarm, goes back to compressions.
    assert!(r
        .timeline
        .iter()
        .any(|e| e.t_ms > frozen_at && e.text.contains("Starts chest compressions")));
}

/// Finding 4. The analyser looks at a 4-second window, so it can keep
/// saying "shockable" for up to 4 s after VF actually stops (aed-rhythm
/// accepts this in `vf_to_sinus_stops_shockable_quickly`). If VF turns into
/// a normal rhythm 1 s before delivery, the shock goes into a normal rhythm.
/// The rule "never shock a non-VF/VT rhythm (ground truth)" therefore cannot
/// hold for changes within the detection latency; the campaign counts these.
#[test]
#[ignore = "FINDING: rhythm change in the last ~4 s before delivery is not detected (aed-rhythm window length vs aed-core countdown)"]
fn conversion_one_second_before_delivery_is_not_shocked() {
    let mut s = by_name("adult-vf").unwrap();
    s.script.push((
        Trigger::AfterState {
            state: RescueState::Countdown,
            nth: 1,
            after_ms: Config::default().countdown_ms - 1_000,
        },
        Action::SetRhythm(Rhythm::sinus(75.0)),
    ));
    s.duration_ms = 40_000;
    let r = run_scenario(&s);
    if !r.violations_of(Rule::FiredOnNonShockableRhythm).is_empty() {
        fail(
            &r,
            "shock fired into a normal rhythm that started 1 s before delivery",
        );
    }
}

/// Finding 5 (FIXED). With a slow message wire (40 ms), after a successful
/// shock the supervisor still saw "Delivering" in the delayed status and
/// recorded "permit refused: already used". It now ignores that echo for
/// the permit window plus a heartbeat timeout after its permit.
#[test]
fn no_spurious_refusal_after_a_good_shock() {
    let r = run_scenario(&by_name("slow-link").unwrap());
    assert_eq!(r.shocks.len(), 1);
    if r.supervisor_refusals.contains(&PermitRefusal::AlreadyUsed) {
        fail(&r, "spurious refusal recorded");
    }
}

/// Finding 6 (FIXED). When the pads came off while charging, the device
/// said "Check pads.", then "Start CPR." and, in the same instant, "Attach
/// pads to bare chest." Now a pads-off abort goes straight to "Attach pads"
/// (after "Check pads." if a shock had been advised): with the pads off the
/// device can neither analyse nor shock, so that is the one instruction.
#[test]
fn pads_off_abort_gives_one_clear_instruction() {
    for name in [
        "pads-off-during-charge",
        "pads-off-during-countdown",
        "fault-pads-off",
    ] {
        let r = run_scenario(&by_name(name).unwrap());
        let both_at_once = r.prompts.windows(2).any(|w| {
            matches!(w, [(t1, Prompt::StartCpr), (t2, Prompt::AttachPads)] if t2.saturating_sub(*t1) < 1_000)
        });
        if both_at_once {
            fail(&r, "'Start CPR' and 'Attach pads' given together");
        }
    }
}

/// Finding 7 (FIXED). The power-on self-test charges to only 400 V, but
/// waited up to the full 12 s charge time limit before declaring the
/// charger failed, so with a weak charger the device said nothing for about
/// 12 s. (This was also the cause of the two "silent at switch-on" runs in
/// the 5,000-run campaign: seeds 430 and 3263, both a weak charger injected
/// during the self-test.) The charge step's limit is now scaled to its
/// voltage (`Config::charge_time_limit_for`): 1 s at 400 V.
#[test]
fn weak_charger_is_reported_promptly_at_switch_on() {
    let mut s = by_name("fault-charger-weak").unwrap();
    s.script = vec![(Trigger::At(0), Action::Fault(aed_sim::Fault::ChargerWeak))];
    let r = run_scenario(&s);
    let first = r.prompts.first().map_or(u32::MAX, |&(t, _)| t);
    if first > 3_000 {
        fail(&r, &format!("first prompt only at {first} ms"));
    }
}

/// Finding 8 (FIXED by D-32). Pads that are on the chest but read just
/// outside 25–175 Ω (here 190 Ω) were treated exactly like pads not
/// attached: "Attach pads to bare chest." every 15 s, forever. Pads now
/// count as on up to 250 Ω: the device analyses, says "Press pads firmly",
/// and, if they still read above 175 Ω when the shock is due, "Poor pad
/// contact. No shock." and "Start CPR". The scenarios `pads-190-throughout`
/// and `pads-190-pressed-to-150` cover the rest.
#[test]
fn attached_pads_out_of_range_get_a_useful_instruction() {
    let mut s = by_name("adult-vf").unwrap();
    s.pad_ohms = 190.0;
    s.duration_ms = 60_000;
    let r = run_scenario(&s);
    let useful = r.prompts.iter().any(|&(_, p)| {
        matches!(
            p,
            Prompt::PressPadsFirmly | Prompt::CheckPads | Prompt::StartCpr
        )
    });
    if !useful || !r.said(Prompt::PressPadsFirmly) || !r.said(Prompt::StartCpr) {
        fail(&r, "only 'Attach pads' for 60 s with pads on at 190 ohms");
    }
}

/// Finding 9 (FIXED by D-28; SIMULATION, MODEL energies). With sensors
/// accurate to ±2 % ± 10 V, the "at the planned voltage" checks could not
/// stop a child-mode shock from a true voltage up to about 1,227 V (plan
/// 1,150 V): a charger regulating high, with both sensors reading low by
/// their full allowance, passed every check, and at 25 to 35 Ω that
/// delivered about 60.5 J, +21 % over the 50 J target. With the sensors of
/// D-28 (±1 % ± 7 V) and the at-plan window set from them (1 % + 12 V) the
/// top edge is (1,150 + 24 + 7) / 0.99 = 1,193 V. Full table:
/// `voltage_sensors.rs`, `delivered_energy_at_the_edges_of_the_allowed_voltage_window`.
#[test]
fn child_energy_at_the_top_of_the_voltage_window_is_within_tolerance() {
    let config = aed_core::Config::default();
    let plan = 1_150.0;
    let a = f64::from(config.at_plan_allowance(aed_types::Volts(1_150)).0);
    let top = (plan + a + 7.0) / 0.99 - 1.0;
    let mut s = by_name("child-vf").unwrap();
    s.duration_ms = 40_000;
    s.pad_ohms = 25.0;
    s.plant.charger_set_point_gain = top / plan;
    let low = aed_sim::SensorModel {
        gain: 0.99,
        offset: -7.0,
        stuck_at: None,
    };
    s.plant.main_voltage_sensor = low;
    s.plant.supervisor_voltage_sensor = low;
    let r = run_scenario(&s);
    let j = r
        .shocks
        .first()
        .map_or(0.0, aed_sim::ShockRecord::delivered_j);
    if !(42.5..=57.5).contains(&j) {
        fail(
            &r,
            &format!(
                "child shock from {top:.0} V into 25 Ω delivered {j:.1} J (target 50 J ±15 %)"
            ),
        );
    }
}

/// Finding 4, measured (SIMULATION). How close to the moment of delivery a
/// change from VF to a normal rhythm (or to a flatline) can happen and still
/// stop the shock, before (5 s countdown, charging only after "shock
/// advised") and after (D-29 charging during analysis, D-30 3 s countdown).
/// Not a pass/fail test: it prints the table for the findings document. It
/// checks only that the safety rule holds when the change is 5 s or more
/// before delivery.
#[test]
fn measure_how_close_to_delivery_a_rhythm_change_is_still_caught() {
    let before = Config {
        countdown_ms: 5_000,
        charge_during_analysis: false,
        ..Config::default()
    };
    let after = Config::default();
    println!("Rhythm change at a time before the planned shock (SIMULATED):");
    for (to_name, to) in [
        ("normal rhythm 75/min", Rhythm::sinus(75.0)),
        ("flatline", Rhythm::Asystole),
    ] {
        for (name, config) in [
            ("before (5 s countdown, charge after advice)", before),
            ("after (D-29 + D-30)", after),
        ] {
            // When the shock fires with no change, for this configuration.
            let mut base = by_name("adult-vf").unwrap();
            base.duration_ms = 40_000;
            let r = aed_sim::Simulation::with_config(&base, config)
                .unwrap()
                .run();
            let fire = r.shocks.first().unwrap().at_ms;
            let analysing = r.states.iter().find(|&&(_, s)| s.is_analysing()).unwrap().0;
            let mut missed = Vec::new();
            for before_ms in (250..=8_000).step_by(250) {
                let mut s = base.clone();
                s.script
                    .push((Trigger::At(fire - before_ms), Action::SetRhythm(to)));
                let r = aed_sim::Simulation::with_config(&s, config).unwrap().run();
                if !r.violations_of(Rule::FiredOnNonShockableRhythm).is_empty() {
                    missed.push(before_ms);
                    assert!(
                        before_ms < 5_000,
                        "{name}: a change {before_ms} ms before delivery was missed"
                    );
                }
            }
            let caught_from = missed.iter().max().map_or(250, |m| m + 250);
            println!(
                "VF to {to_name}, {name}: missed when {:.2} s or less before the shock, always caught from {:.2} s; \
                 hands-off from 'Analysing' to the shock {:.1} s, so {:.0} % of that pause is blind",
                f64::from(caught_from.saturating_sub(250)) / 1000.0,
                f64::from(caught_from) / 1000.0,
                f64::from(fire - analysing) / 1000.0,
                100.0 * f64::from(caught_from) / f64::from(fire - analysing)
            );
        }
    }
}

/// Finding 12 (closed IN SIMULATION by compression sensing, D-36).
/// Compressions at 150/min over a flatline, from a responder who never
/// stops, were read by the heart-signal analysis as a shockable rhythm. With
/// compression sensing the analysis is never fed while compressions are
/// sensed: no countdown, no shock. Without it the flatline is shocked.
#[test]
fn finding_12_fast_compressions_over_a_flatline_are_never_shocked() {
    let r = run_scenario(&by_name("f12-compressions-150-over-flatline").unwrap());
    if !r.shocks.is_empty() || r.entered(RescueState::Countdown) {
        fail(
            &r,
            "compressions over a flatline led to a countdown or a shock",
        );
    }
    let mut old = by_name("f12-compressions-150-over-flatline").unwrap();
    old.chest_sensing = false;
    let old = run(&old).unwrap();
    assert!(!old
        .violations_of(Rule::FiredOnNonShockableRhythm)
        .is_empty());
}
