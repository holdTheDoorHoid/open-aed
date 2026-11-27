// SPDX-License-Identifier: GPL-3.0-or-later
//! Choosing the phase lengths for a measured pad resistance.

use aed_types::{
    Decijoules, Micros, Ohms, PatientMode, PlanError, ShockPlan, ShockPlanner, Volts,
    MAX_PATIENT_OHMS, MIN_PATIENT_OHMS,
};

use crate::config::{ConfigError, WaveformConfig};
use crate::model::discharge;

/// A limit the planner ran into, so the delivered energy is not exactly on
/// target. The design table reports these rows and by how much they miss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitHit {
    /// Even the shortest allowed shock delivers more than the target
    /// (low-resistance patients), so the shortest allowed one is used.
    ShortestAllowed,
    /// Even the longest allowed shock delivers less than the target
    /// (high-resistance patients), so the longest allowed one is used.
    LongestAllowed,
}

/// The shock planner: implements [`ShockPlanner`] for the biphasic
/// truncated exponential shock described by a [`WaveformConfig`].
///
/// How it plans: the charge voltage is fixed per mode. For the measured
/// resistance it picks the phase 1 length (to the microsecond) whose
/// modelled delivered energy is closest to the mode's target; phase 2 is
/// the configured fraction of phase 1. Phase lengths are kept inside their
/// limits and the whole shock inside the total cap, even if that means
/// missing the target.
///
/// Resistance range: exactly 25–175 Ω inclusive is accepted. There is no
/// guard band: 24 Ω is refused as too low and 176 Ω as too high. Reason:
/// the planner should not extrapolate beyond the range the standard tests
/// and the design table covers. Allowing for measurement error is a job for
/// the resistance measurement, and widening the range would be an owner
/// decision recorded in DECISIONS.md (other devices quote 20–230 Ω and
/// 25–180 Ω; see `waveforms_energy_pads.md`, Q2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BtePlanner {
    config: WaveformConfig,
}

impl BtePlanner {
    /// Make a planner, after checking the configuration.
    ///
    /// # Errors
    /// Returns the [`ConfigError`] from [`WaveformConfig::validate`].
    pub fn new(config: WaveformConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// The configuration this planner uses.
    #[must_use]
    pub const fn config(&self) -> &WaveformConfig {
        &self.config
    }
}

impl Default for BtePlanner {
    /// A planner using [`WaveformConfig::default`] (checked by a test).
    fn default() -> Self {
        Self {
            config: WaveformConfig::default(),
        }
    }
}

impl ShockPlanner for BtePlanner {
    fn charge_voltage(&self, mode: PatientMode) -> Volts {
        self.config.charge_voltage(mode)
    }

    fn plan(&self, mode: PatientMode, impedance: Ohms) -> Result<ShockPlan, PlanError> {
        plan_with(&self.config, mode, impedance).map(|p| p.plan)
    }
}

/// A plan plus which limit (if any) stopped it hitting the target.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Planned {
    pub plan: ShockPlan,
    pub limit: Option<LimitHit>,
}

/// The planning rule, usable on a configuration that has not been validated
/// (validation itself uses it to check monotonicity). Never panics.
pub(crate) fn plan_with(
    cfg: &WaveformConfig,
    mode: PatientMode,
    impedance: Ohms,
) -> Result<Planned, PlanError> {
    if impedance < MIN_PATIENT_OHMS {
        return Err(PlanError::ImpedanceTooLow);
    }
    if impedance > MAX_PATIENT_OHMS {
        return Err(PlanError::ImpedanceTooHigh);
    }
    let voltage = cfg.charge_voltage(mode);
    let patient = f64::from(impedance.0);
    let target = WaveformConfig::target_joules(mode);
    let energy = |p1: u32| delivered(cfg, voltage, patient, p1);

    let shortest = cfg.phase1_min.0;
    let longest = longest_phase1(cfg).max(shortest);

    let (phase1, limit) = if energy(shortest) >= target {
        (shortest, Some(LimitHit::ShortestAllowed))
    } else if energy(longest) <= target {
        (longest, Some(LimitHit::LongestAllowed))
    } else {
        // Delivered energy only grows as phase 1 lengthens (phase 2 follows
        // it), so halve the interval until `lo` is just under the target
        // and `hi` just over, then take whichever is closer.
        let (mut lo, mut hi) = (shortest, longest);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if energy(mid) >= target {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let closer = if target - energy(lo) <= energy(hi) - target {
            lo
        } else {
            hi
        };
        (closer, None)
    };

    Ok(Planned {
        plan: make_plan(cfg, voltage, phase1, energy(phase1)),
        limit,
    })
}

/// Build the plan record; expected energy rounded to the nearest 0.1 J.
fn make_plan(cfg: &WaveformConfig, voltage: Volts, phase1: u32, energy_j: f64) -> ShockPlan {
    // `as` saturates, so a silly figure cannot wrap around.
    let deci = libm::round(energy_j * 10.0) as u16;
    ShockPlan {
        charge_voltage: voltage,
        phase1: Micros(phase1),
        interphase: cfg.interphase,
        phase2: Micros(cfg.phase2_for(phase1)),
        expected_energy: Decijoules(deci),
    }
}

/// Modelled patient energy (J) for a given phase 1 length (µs).
fn delivered(cfg: &WaveformConfig, voltage: Volts, patient: f64, phase1: u32) -> f64 {
    let plan = make_plan(cfg, voltage, phase1, 0.0);
    discharge(&plan, patient, cfg).map_or(0.0, |r| r.patient_energy_j)
}

/// Longest phase 1 (µs) within its own limit and the total-duration cap.
fn longest_phase1(cfg: &WaveformConfig) -> u32 {
    let (mut lo, mut hi) = (cfg.phase1_min.0, cfg.phase1_max.0);
    if cfg.fits_total(hi) {
        return hi;
    }
    if hi <= lo || !cfg.fits_total(lo) {
        return lo;
    }
    // `lo` fits, `hi` does not; fitting only gets harder as phase 1 grows.
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if cfg.fits_total(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}
