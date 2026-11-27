// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests of the shock model, the planner and the design table.
//!
//! These check the arithmetic is consistent with itself and with the design
//! limits. They are tests of a MODEL; they say nothing about real hardware.

use aed_types::{
    Micros, Ohms, PatientMode, PlanError, ShockPlan, ShockPlanner, Volts, MAX_PATIENT_OHMS,
    MIN_PATIENT_OHMS,
};
use aed_waveform::{
    design_row, design_table, discharge, write_report, BtePlanner, ConfigError, DischargeResult,
    WaveformConfig,
};

const MODES: [PatientMode; 2] = [PatientMode::Adult, PatientMode::Child];

fn all_ohms() -> impl Iterator<Item = u16> {
    MIN_PATIENT_OHMS.0..=MAX_PATIENT_OHMS.0
}

fn planned(mode: PatientMode, ohms: u16) -> Result<(ShockPlan, DischargeResult), String> {
    let cfg = WaveformConfig::default();
    let plan = BtePlanner::default()
        .plan(mode, Ohms(ohms))
        .map_err(|e| format!("{mode:?} {ohms} Ω: {e:?}"))?;
    let result = discharge(&plan, f64::from(ohms), &cfg).ok_or("model refused")?;
    Ok((plan, result))
}

/// Composite Simpson's rule over [a, b] with n (even) intervals.
fn simpson(f: impl Fn(f64) -> f64, a: f64, b: f64, n: u32) -> f64 {
    let h = (b - a) / f64::from(n);
    let mut sum = f(a) + f(b);
    for k in 1..n {
        let w = if k % 2 == 1 { 4.0 } else { 2.0 };
        sum += w * f(a + h * f64::from(k));
    }
    sum * h / 3.0
}

#[test]
fn default_config_is_valid() {
    assert_eq!(WaveformConfig::default().validate(), Ok(()));
    assert_eq!(
        BtePlanner::new(WaveformConfig::default()).map(|p| *p.config()),
        Ok(WaveformConfig::default())
    );
}

#[test]
fn energy_is_conserved() -> Result<(), String> {
    for mode in MODES {
        for ohms in all_ohms() {
            let (_, r) = planned(mode, ohms)?;
            let sum = r.patient_energy_j + r.internal_energy_j + r.residual_energy_j;
            assert!(
                (sum - r.stored_energy_j).abs() < 1e-9 * r.stored_energy_j,
                "{mode:?} {ohms} Ω: {sum} vs {}",
                r.stored_energy_j
            );
            let phases = r.patient_energy_phase1_j + r.patient_energy_phase2_j;
            assert!((phases - r.patient_energy_j).abs() < 1e-12 * r.stored_energy_j);
        }
    }
    Ok(())
}

#[test]
fn analytic_energy_matches_numerical_integration() -> Result<(), String> {
    for mode in MODES {
        for ohms in [25_u16, 50, 80, 125, 175] {
            let (_, r) = planned(mode, ohms)?;
            let power = |t: f64| {
                let i = r.current_at(t);
                i * i * r.patient_ohms
            };
            // Integrate each phase separately: the current jumps at the edges.
            let p2_start = r.phase1_s + r.interphase_s;
            let e1 = simpson(power, 0.0, r.phase1_s * (1.0 - 1e-12), 4000);
            let e2 = simpson(
                power,
                p2_start,
                (p2_start + r.phase2_s) * (1.0 - 1e-12),
                4000,
            );
            let gap = simpson(power, r.phase1_s, p2_start * (1.0 - 1e-12), 100);
            assert!((e1 - r.patient_energy_phase1_j).abs() < 1e-6 * r.patient_energy_j);
            assert!((e2 - r.patient_energy_phase2_j).abs() < 1e-6 * r.patient_energy_j);
            assert!(gap.abs() < 1e-12, "current flowed in the gap");

            // Charge moved, the same way.
            let q1 = simpson(|t| r.current_at(t), 0.0, r.phase1_s * (1.0 - 1e-12), 4000);
            let q2 = simpson(
                |t| r.current_at(t),
                p2_start,
                (p2_start + r.phase2_s) * (1.0 - 1e-12),
                4000,
            );
            assert!((q1 - r.phase1_charge_c).abs() < 1e-6 * r.phase1_charge_c);
            assert!((q2 + r.phase2_charge_c).abs() < 1e-6 * r.phase2_charge_c);
            assert!((r.phase1_average_current_a - r.phase1_charge_c / r.phase1_s).abs() < 1e-9);
        }
    }
    Ok(())
}

#[test]
fn delivered_energy_within_tolerance_everywhere() -> Result<(), String> {
    // With the default design, every whole-ohm resistance in both modes is
    // inside the (UNVERIFIED, placeholder) ±15% tolerance. If a config change
    // breaks this, list the exceptions here explicitly rather than loosening.
    let tol = WaveformConfig::default().energy_tolerance;
    for mode in MODES {
        let target = WaveformConfig::target_joules(mode);
        for ohms in all_ohms() {
            let (_, r) = planned(mode, ohms)?;
            let err = (r.patient_energy_j - target) / target;
            assert!(err.abs() <= tol, "{mode:?} {ohms} Ω: {:+.1}%", err * 100.0);
        }
    }
    Ok(())
}

#[test]
fn unclamped_rows_hit_target_closely() -> Result<(), String> {
    let cfg = WaveformConfig::default();
    for mode in MODES {
        for ohms in all_ohms() {
            let row = design_row(&cfg, mode, Ohms(ohms)).ok_or("no row")?;
            if row.limit.is_none() {
                // Microsecond steps move the energy by far less than 0.1%.
                assert!(row.error_fraction.abs() < 1e-3, "{mode:?} {ohms} Ω");
            }
        }
    }
    Ok(())
}

#[test]
fn durations_monotonic_and_within_limits() -> Result<(), String> {
    let cfg = WaveformConfig::default();
    for mode in MODES {
        let mut prev = (0_u32, 0_u32);
        for ohms in all_ohms() {
            let (plan, _) = planned(mode, ohms)?;
            let (p1, p2) = (plan.phase1.0, plan.phase2.0);
            assert!(
                p1 >= prev.0 && p2 >= prev.1,
                "{mode:?} {ohms} Ω shorter than before"
            );
            prev = (p1, p2);
            assert!((cfg.phase1_min.0..=cfg.phase1_max.0).contains(&p1));
            assert!((cfg.phase2_min.0..=cfg.phase2_max.0).contains(&p2));
            assert_eq!(plan.interphase, cfg.interphase);
            assert!(p1 + plan.interphase.0 + p2 <= cfg.max_total.0);
            assert_eq!(plan.charge_voltage, cfg.charge_voltage(mode));
            assert!(plan.charge_voltage <= cfg.max_charge);
        }
    }
    Ok(())
}

#[test]
fn peak_current_within_limit() -> Result<(), String> {
    let cfg = WaveformConfig::default();
    for mode in MODES {
        let (_, r) = planned(mode, MIN_PATIENT_OHMS.0)?;
        assert!(r.peak_current_a <= cfg.max_peak_current_amps);
    }
    Ok(())
}

#[test]
fn phase2_reverses_and_starts_where_phase1_ended() -> Result<(), String> {
    for mode in MODES {
        for ohms in all_ohms() {
            let (_, r) = planned(mode, ohms)?;
            assert!(r.peak_current_a > 0.0 && r.phase1_end_current_a > 0.0);
            assert!(r.phase2_start_current_a < 0.0 && r.phase2_end_current_a < 0.0);
            assert!((r.phase2_start_current_a + r.phase1_end_current_a).abs() < 1e-12);
            // Sampled current agrees at the edges.
            let p2 = r.phase1_s + r.interphase_s;
            assert!((r.current_at(p2) - r.phase2_start_current_a).abs() < 1e-12);
            assert!((r.current_at(0.0) - r.peak_current_a).abs() < 1e-12);
            assert!(r.current_at(-1e-6).abs() < 1e-15);
            assert!(r.current_at(r.total_s()).abs() < 1e-15);
            assert!(
                r.current_at(r.phase1_s).abs() < 1e-15,
                "gap must carry no current"
            );
            assert!(r.phase1_tilt > 0.0 && r.phase1_tilt < 1.0);
        }
    }
    Ok(())
}

#[test]
fn out_of_range_resistance_is_refused() {
    let planner = BtePlanner::default();
    for mode in MODES {
        assert_eq!(planner.plan(mode, Ohms(0)), Err(PlanError::ImpedanceTooLow));
        assert_eq!(
            planner.plan(mode, Ohms(24)),
            Err(PlanError::ImpedanceTooLow)
        );
        assert_eq!(
            planner.plan(mode, Ohms(176)),
            Err(PlanError::ImpedanceTooHigh)
        );
        assert_eq!(
            planner.plan(mode, Ohms(u16::MAX)),
            Err(PlanError::ImpedanceTooHigh)
        );
        assert!(planner.plan(mode, MIN_PATIENT_OHMS).is_ok());
        assert!(planner.plan(mode, MAX_PATIENT_OHMS).is_ok());
    }
}

#[test]
fn expected_energy_matches_model() -> Result<(), String> {
    for mode in MODES {
        for ohms in all_ohms() {
            let (plan, r) = planned(mode, ohms)?;
            let model_deci = (r.patient_energy_j * 10.0).round();
            assert!((f64::from(plan.expected_energy.0) - model_deci).abs() < 0.5);
        }
    }
    Ok(())
}

#[test]
fn charge_voltage_is_fixed_per_mode() {
    let planner = BtePlanner::default();
    assert_eq!(planner.charge_voltage(PatientMode::Adult), Volts(1950));
    assert_eq!(planner.charge_voltage(PatientMode::Child), Volts(1150));
}

#[test]
fn validation_rejects_bad_configs() {
    let good = WaveformConfig::default();
    let cases: [(WaveformConfig, ConfigError); 14] = [
        (
            WaveformConfig {
                capacitance_uf: 0.0,
                ..good
            },
            ConfigError::CapacitanceInvalid,
        ),
        (
            WaveformConfig {
                capacitance_uf: f64::NAN,
                ..good
            },
            ConfigError::CapacitanceInvalid,
        ),
        (
            WaveformConfig {
                internal_ohms: -1.0,
                ..good
            },
            ConfigError::InternalResistanceInvalid,
        ),
        (
            WaveformConfig {
                child_charge: Volts(0),
                ..good
            },
            ConfigError::ChargeVoltageZero,
        ),
        (
            WaveformConfig {
                adult_charge: Volts(2100),
                ..good
            },
            ConfigError::ChargeVoltageAboveMax,
        ),
        (
            WaveformConfig {
                max_peak_current_amps: 0.0,
                ..good
            },
            ConfigError::PeakCurrentLimitInvalid,
        ),
        (
            WaveformConfig {
                max_peak_current_amps: 60.0,
                ..good
            },
            ConfigError::PeakCurrentTooHigh,
        ),
        (
            WaveformConfig {
                phase1_min: Micros(13_000),
                ..good
            },
            ConfigError::Phase1LimitsInvalid,
        ),
        (
            WaveformConfig {
                phase2_ratio: 0.0,
                ..good
            },
            ConfigError::Phase2RatioInvalid,
        ),
        (
            WaveformConfig {
                phase2_min: Micros(9_000),
                ..good
            },
            ConfigError::Phase2LimitsInvalid,
        ),
        (
            WaveformConfig {
                max_total: Micros(5_000),
                ..good
            },
            ConfigError::TotalTooShort,
        ),
        (
            WaveformConfig {
                energy_tolerance: 0.0,
                ..good
            },
            ConfigError::ToleranceInvalid,
        ),
        (
            WaveformConfig {
                energy_tolerance: 1.5,
                ..good
            },
            ConfigError::ToleranceInvalid,
        ),
        // A large internal resistance with almost no minimum phase length: at
        // the low end the planner would then give some higher resistance a
        // shorter shock than a lower one (more of the energy is lost inside the
        // device at the lowest resistances, so they need longer shocks).
        (
            WaveformConfig {
                internal_ohms: 20.0,
                adult_charge: Volts(2000),
                max_peak_current_amps: 200.0,
                phase1_min: Micros(1),
                phase2_min: Micros(1),
                ..good
            },
            ConfigError::NotMonotonic,
        ),
    ];
    for (cfg, want) in cases {
        assert_eq!(cfg.validate(), Err(want));
        assert_eq!(BtePlanner::new(cfg), Err(want));
    }
}

#[test]
fn model_refuses_impossible_circuits() {
    let plan = ShockPlan {
        charge_voltage: Volts(1000),
        phase1: Micros(5000),
        interphase: Micros(500),
        phase2: Micros(3000),
        expected_energy: aed_types::Decijoules(0),
    };
    let good = WaveformConfig::default();
    assert!(discharge(&plan, -1.0, &good).is_none());
    assert!(discharge(&plan, f64::NAN, &good).is_none());
    let no_r = WaveformConfig {
        internal_ohms: 0.0,
        ..good
    };
    assert!(discharge(&plan, 0.0, &no_r).is_none());
    assert!(discharge(
        &plan,
        50.0,
        &WaveformConfig {
            capacitance_uf: 0.0,
            ..good
        }
    )
    .is_none());
}

#[test]
fn table_covers_every_step() {
    let rows: Vec<_> = design_table(&WaveformConfig::default()).collect();
    assert_eq!(rows.len(), 2 * 31);
    assert!(rows.iter().all(|r| r.within_tolerance));
}

#[test]
fn saved_design_table_is_current() {
    let mut text = String::new();
    assert!(write_report(&WaveformConfig::default(), &mut text).is_ok());
    println!("{text}");
    let saved = include_str!("../DESIGN_TABLE.md");
    assert!(
        saved == text,
        "DESIGN_TABLE.md is out of date; regenerate it with \
         `cargo run -p aed-waveform --example design_table > crates/aed-waveform/DESIGN_TABLE.md`"
    );
}
