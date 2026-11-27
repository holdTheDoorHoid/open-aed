// SPDX-License-Identifier: GPL-3.0-or-later
//! Integration problems found by running the three firmware parts together.
//!
//! Each `#[ignore]`d test states what SHOULD happen and FAILS today. They are
//! ignored so the suite stays green while still listing them; run them with
//! `cargo test -p aed-sim --test findings -- --ignored --nocapture`.
//! The fixes belong in other crates (or in hardware design decisions), not here.
//! A finding's test is un-parked (no longer ignored) once it is fixed: F-1,
//! F-2 and F-3 are fixed; their wider tests are in `late_hands_off.rs`,
//! `voltage_sensors.rs` and `frozen_main.rs`.
//! SIMULATION ONLY.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_core::{PermitRefusal, Prompt, RescueState};
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
        fail(&r, "VF patient never shocked because compressions stopped 2.5 s late");
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
            row.push_str(if r.shocks.is_empty() { " no |" } else { " yes |" });
        }
        println!("{row}");
    }
}

/// Finding 2 (FIXED). Two healthy voltage sensors with opposite 2 % gain
/// errors (+2 %/+10 V and −2 %/−10 V) differ by about 98 V at 1,950 V. The
/// old single 50 V `voltage_tolerance` made the supervisor latch
/// `VoltageDisagreement`. The agreement budget now grows with the voltage
/// (`Config::agree_percent` + `Config::agree_abs_volts`, per sensor, twice
/// that between the two), separate from the "at the planned voltage" window.
#[test]
fn two_percent_sensor_errors_do_not_disable_the_device() {
    let mut s = by_name("adult-vf").unwrap();
    s.plant.main_voltage_sensor.gain = 1.02;
    s.plant.main_voltage_sensor.offset = 10.0;
    s.plant.supervisor_voltage_sensor.gain = 0.98;
    s.plant.supervisor_voltage_sensor.offset = -10.0;
    let r = run_scenario(&s);
    if r.shocks.len() != 1 {
        fail(&r, "sensor errors of ±2 % made the device fault");
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
    assert!(s.plant.charger_gated_by_discharge, "the hardware rule is the default");
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
    let told = r.supervisor_alarm_at.is_some_and(|t| t >= frozen_at && t - frozen_at <= grace);
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
            after_ms: 4_000,
        },
        Action::SetRhythm(Rhythm::sinus(75.0)),
    ));
    s.duration_ms = 40_000;
    let r = run_scenario(&s);
    if !r.violations_of(Rule::FiredOnNonShockableRhythm).is_empty() {
        fail(&r, "shock fired into a normal rhythm that started 1 s before delivery");
    }
}

/// Finding 5. With a slow message wire (40 ms), after a successful shock
/// the supervisor still sees "Delivering" in the delayed status and records
/// "permit refused: already used" in its event record. Harmless to the
/// shock, but the record then reads as if a permit had been refused.
#[test]
#[ignore = "FINDING: supervisor records a spurious 'permit refused (already used)' after a good shock on a slow link (aed-core supervisor.rs permit decision)"]
fn no_spurious_refusal_after_a_good_shock() {
    let r = run_scenario(&by_name("slow-link").unwrap());
    assert_eq!(r.shocks.len(), 1);
    if r.supervisor_refusals.contains(&PermitRefusal::AlreadyUsed) {
        fail(&r, "spurious refusal recorded");
    }
}

/// Finding 6. When the pads come off while charging, the device says
/// "Check pads.", then "Start CPR." and, in the same instant, "Attach pads
/// to bare chest." Two contradictory instructions at once.
#[test]
#[ignore = "FINDING: 'Start CPR' immediately followed by 'Attach pads' after a pads-off abort (aed-core rescue.rs abort -> disarm -> Cpr -> AttachPads)"]
fn pads_off_abort_gives_one_clear_instruction() {
    let r = run_scenario(&by_name("pads-off-during-charge").unwrap());
    let both_at_once = r.prompts.windows(2).any(|w| {
        matches!(w, [(t1, Prompt::StartCpr), (t2, Prompt::AttachPads)] if t2.saturating_sub(*t1) < 1_000)
    });
    if both_at_once {
        fail(&r, "'Start CPR' and 'Attach pads' given together");
    }
}

/// Finding 7. The power-on self-test charges to only 400 V, but waits up to
/// `Config::charge_time_limit_ms` (12 s, sized for a full 1,950 V charge)
/// before declaring the charger failed. A good charger needs about 0.1 s
/// for 400 V. With a weak charger the device says NOTHING for about 12 s
/// after switch-on (no "attach pads", no fault), then reports the fault.
#[test]
#[ignore = "FINDING: self-test charge step uses the full-charge time limit; weak charger means ~12 s of silence at switch-on (aed-core selftest.rs Step::Charge)"]
fn weak_charger_is_reported_promptly_at_switch_on() {
    let mut s = by_name("fault-charger-weak").unwrap();
    s.script = vec![(Trigger::At(0), Action::Fault(aed_sim::Fault::ChargerWeak))];
    let r = run_scenario(&s);
    let first = r.prompts.first().map_or(u32::MAX, |&(t, _)| t);
    if first > 3_000 {
        fail(&r, &format!("first prompt only at {first} ms"));
    }
}

/// Finding 8. Pads that are on the chest but read just outside 25–175 Ω
/// (here 190 Ω, e.g. poor contact or a hairy chest; other devices accept up
/// to 180–230 Ω) are treated exactly like pads not attached: the device
/// never analyses and repeats "Attach pads to bare chest." every 15 s, an
/// instruction the responder has already followed. It never says "Check
/// pads" and never tells them to continue CPR.
#[test]
#[ignore = "FINDING: attached pads just out of range get only 'Attach pads' forever (aed-core rescue.rs PadMonitor / tick_attach_pads)"]
fn attached_pads_out_of_range_get_a_useful_instruction() {
    let mut s = by_name("adult-vf").unwrap();
    s.pad_ohms = 190.0;
    s.duration_ms = 60_000;
    let r = run_scenario(&s);
    let useful = r
        .prompts
        .iter()
        .any(|&(_, p)| matches!(p, Prompt::CheckPads | Prompt::StartCpr));
    if !useful {
        fail(&r, "only 'Attach pads' for 60 s with pads on at 190 ohms");
    }
}

/// New finding from the F-2 work (SIMULATION, MODEL energies). With
/// sensors accurate to ±2 % ± 10 V (what the default `Config` assumes),
/// the "at the planned voltage" checks cannot stop a shock from a true
/// voltage up to about 1,227 V in child mode (plan 1,150 V): a charger that
/// regulates high, with both sensors reading low by their full allowance,
/// still passes every check. At 25 to 35 Ω that delivers about 60.5 J,
/// +21 % over the 50 J target, outside the ±15 % energy tolerance (at 25 Ω
/// the planner is already at its shortest phase and +6 % on target). Adult
/// mode and the bottom edge in child mode stay inside. The window cannot be
/// narrowed without false faults from in-spec sensors, so this is an input
/// to the sensor specification (about ±1 % ± 7 V would be needed) or to the
/// child-mode waveform. Full table: `voltage_sensors.rs`,
/// `delivered_energy_at_the_edges_of_the_allowed_voltage_window`.
#[test]
#[ignore = "FINDING: ±2 % ± 10 V sensors allow a child shock about +21 % over target at 25-35 Ω (sensor accuracy vs energy tolerance; aed-core config.rs at_plan_*, aed-waveform child charge)"]
fn child_energy_at_the_top_of_the_voltage_window_is_within_tolerance() {
    let config = aed_core::Config::default();
    let plan = 1_150.0;
    let a = f64::from(config.at_plan_allowance(aed_types::Volts(1_150)).0);
    let top = (plan + a + 10.0) / 0.98 - 1.0;
    let mut s = by_name("child-vf").unwrap();
    s.duration_ms = 40_000;
    s.pad_ohms = 25.0;
    s.plant.charger_set_point_gain = top / plan;
    let low = aed_sim::SensorModel {
        gain: 0.98,
        offset: -10.0,
        stuck_at: None,
    };
    s.plant.main_voltage_sensor = low;
    s.plant.supervisor_voltage_sensor = low;
    let r = run_scenario(&s);
    let j = r.shocks.first().map_or(0.0, aed_sim::ShockRecord::delivered_j);
    if !(42.5..=57.5).contains(&j) {
        fail(&r, &format!("child shock from {top:.0} V into 25 Ω delivered {j:.1} J (target 50 J ±15 %)"));
    }
}
