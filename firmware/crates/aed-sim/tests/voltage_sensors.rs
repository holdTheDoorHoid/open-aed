// SPDX-License-Identifier: GPL-3.0-or-later
//! Finding F-2 (fixed): the two independent voltage sensors and how much
//! they may disagree, and what the allowed voltage window means for the
//! delivered energy (finding F-9). Since owner decision D-28 the sensors
//! are assumed accurate to ±1 % ± 7 V. SIMULATION ONLY: model sensors,
//! model shock energy, provisional settings.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_core::{Config, RescueState};
use aed_sim::{by_name, run, RunResult, Scenario, SensorModel};
use aed_types::{PatientMode, ShockPlanner};
use aed_waveform::{BtePlanner, WaveformConfig};
use common::{assert_invariants, fail};

/// The sensor accuracy the default `Config` assumes (D-28): ±1 % ± 7 V.
const GAIN_ERROR: f64 = 0.01;
const OFFSET_ERROR: f64 = 7.0;

fn sensor(sign: f64) -> SensorModel {
    SensorModel {
        gain: 1.0 + sign * GAIN_ERROR,
        offset: sign * OFFSET_ERROR,
        stuck_at: None,
    }
}

fn rescue(name: &str) -> Scenario {
    let mut s = by_name(name).unwrap();
    s.duration_ms = 40_000;
    s
}

fn assert_one_clean_shock(r: &RunResult, what: &str) {
    assert_invariants(r);
    if r.shocks.len() != 1 || r.final_fault.is_some() || r.supervisor_fault.is_some() {
        fail(r, &format!("{what}: expected one shock and no fault"));
    }
}

#[test]
fn supervisor_does_not_fault_with_opposite_sensor_errors_inside_the_budget() {
    // Adult (1,950 V) and child (1,150 V) rescues, each starting with the
    // 400 V power-on self-test, with the two sensors at opposite ends of
    // their assumed accuracy, both ways round.
    for name in ["adult-vf", "child-vf"] {
        for sign in [1.0, -1.0] {
            let mut s = rescue(name);
            s.plant.main_voltage_sensor = sensor(sign);
            s.plant.supervisor_voltage_sensor = sensor(-sign);
            let r = run(&s).unwrap();
            assert_one_clean_shock(&r, &format!("{name}, main sensor sign {sign}"));
        }
    }
}

#[test]
fn device_faults_when_one_sensor_is_outside_the_budget() {
    // Outside the D-28 budget: one sensor 5 % or 8 % off, the other perfect.
    for name in ["adult-vf", "child-vf"] {
        for (main_gain, sup_gain) in [
            (1.05, 1.0),
            (0.95, 1.0),
            (1.0, 1.05),
            (1.0, 0.95),
            (1.08, 1.0),
            (0.92, 1.0),
            (1.0, 1.08),
            (1.0, 0.92),
        ] {
            let mut s = rescue(name);
            s.plant.main_voltage_sensor.gain = main_gain;
            s.plant.supervisor_voltage_sensor.gain = sup_gain;
            let r = run(&s).unwrap();
            // A main sensor reading low never sees the 400 V self-test charge
            // as reached; since the F-7 fix the self-test reports that after
            // about 1 s, so the device is no longer silent at switch-on.
            assert_invariants(&r);
            if !r.shocks.is_empty() || r.final_state != RescueState::Fault {
                fail(&r, &format!("{name}: main gain {main_gain}, supervisor gain {sup_gain}: expected a fault and no shock"));
            }
            println!(
                "{name}, main gain {main_gain}, supervisor gain {sup_gain}: fault {:?}, supervisor {:?}",
                r.final_fault, r.supervisor_fault
            );
        }
    }
}

#[test]
fn a_stuck_voltage_sensor_is_caught() {
    for (main_stuck, stuck_at) in [(true, 0.0), (false, 0.0), (true, 1_950.0), (false, 1_950.0)] {
        let mut s = rescue("adult-vf");
        let stuck = SensorModel {
            stuck_at: Some(stuck_at),
            ..SensorModel::ideal()
        };
        if main_stuck {
            s.plant.main_voltage_sensor = stuck;
        } else {
            s.plant.supervisor_voltage_sensor = stuck;
        }
        let r = run(&s).unwrap();
        assert_invariants(&r);
        if !r.shocks.is_empty() || r.final_state != RescueState::Fault {
            fail(
                &r,
                &format!("sensor stuck at {stuck_at} V (main: {main_stuck}) not caught"),
            );
        }
        println!(
            "stuck at {stuck_at} V (main {main_stuck}): fault {:?}",
            r.final_fault
        );
    }
}

/// One edge of the window of true capacitor voltages from which a shock can
/// still fire with in-spec sensors: the charger regulates wrongly (true
/// voltage `v`), and both sensors err by their full assumed accuracy towards
/// the plan, so each controller still reads "at plan".
#[derive(Debug, Clone, Copy)]
struct Edge {
    mode: PatientMode,
    high: bool,
    /// The true voltage at the edge (1 V inside it).
    true_v: f64,
}

fn window_edges(config: &Config, waveform: &WaveformConfig) -> Vec<Edge> {
    let planner = BtePlanner::new(*waveform).unwrap();
    let mut out = Vec::new();
    for mode in [PatientMode::Adult, PatientMode::Child] {
        let plan = planner.charge_voltage(mode);
        let a = f64::from(config.at_plan_allowance(plan).0);
        let p = f64::from(plan.0);
        // Reading = V x (1 - 2 %) - 10 V must be <= plan + a (high edge),
        // reading = V x (1 + 2 %) + 10 V must be >= plan - a (low edge).
        let high = (p + a + OFFSET_ERROR) / (1.0 - GAIN_ERROR) - 1.0;
        let low = (p - a - OFFSET_ERROR) / (1.0 + GAIN_ERROR) + 1.0;
        out.push(Edge {
            mode,
            high: false,
            true_v: low,
        });
        out.push(Edge {
            mode,
            high: true,
            true_v: high,
        });
    }
    out
}

/// Delivered energy (model) for a shock from the edge voltage into `ohms`,
/// from a whole simulated rescue. Returns (joules, error as a fraction).
fn energy_at_edge(edge: Edge, ohms: f64) -> (f64, f64) {
    let planner = BtePlanner::default();
    let plan = f64::from(planner.charge_voltage(edge.mode).0);
    let mut s = rescue(if edge.mode == PatientMode::Adult {
        "adult-vf"
    } else {
        "child-vf"
    });
    s.pad_ohms = ohms;
    s.plant.charger_set_point_gain = edge.true_v / plan;
    // Sensors err towards the plan: low readings when the voltage is high.
    let sign = if edge.high { -1.0 } else { 1.0 };
    s.plant.main_voltage_sensor = sensor(sign);
    s.plant.supervisor_voltage_sensor = sensor(sign);
    let r = run(&s).unwrap();
    assert_one_clean_shock(&r, &format!("{edge:?} at {ohms} Ω"));
    let shot = r.shocks.first().unwrap();
    let target = WaveformConfig::target_joules(edge.mode);
    (shot.delivered_j(), shot.delivered_j() / target - 1.0)
}

const OHMS: [f64; 6] = [25.0, 35.0, 50.0, 80.0, 125.0, 175.0];

#[test]
fn delivered_energy_at_the_edges_of_the_allowed_voltage_window() {
    let config = Config::default();
    let waveform = WaveformConfig::default();
    let tolerance = waveform.energy_tolerance;
    println!(
        "Delivered energy at the edges of the voltage window (MODEL, sensors ±{:.0} % ± {:.0} V):",
        GAIN_ERROR * 100.0,
        OFFSET_ERROR
    );
    print!("| mode | edge | true V |");
    for o in OHMS {
        print!(" {o:.0} Ω |");
    }
    println!();
    let mut outside = Vec::new();
    for edge in window_edges(&config, &waveform) {
        let mut row = format!(
            "| {:?} | {} | {:.0} |",
            edge.mode,
            if edge.high { "top" } else { "bottom" },
            edge.true_v
        );
        for ohms in OHMS {
            let (j, err) = energy_at_edge(edge, ohms);
            row.push_str(&format!(" {j:.1} J ({:+.1} %) |", err * 100.0));
            if err.abs() > tolerance {
                outside.push((edge, ohms, err));
            }
        }
        println!("{row}");
    }
    println!("outside ±{:.0} %: {outside:?}", tolerance * 100.0);
    // With the D-28 sensors (±1 % ± 7 V) and the at-plan window set from
    // them (1 % + 12 V), both modes stay within tolerance at both edges and
    // every resistance (finding F-9 closed; it is also checked on its own in
    // findings.rs).
    assert!(outside.is_empty(), "outside tolerance: {outside:?}");
}
