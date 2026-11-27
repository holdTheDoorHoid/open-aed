// SPDX-License-Identifier: GPL-3.0-or-later
//! The electrical model of one biphasic truncated exponential shock.

use aed_types::ShockPlan;
use libm::exp;

use crate::config::WaveformConfig;

/// Everything the model says about one shock into one patient.
///
/// All figures are MODEL figures (see the crate documentation for what the
/// model ignores). Units are plain SI: volts, amperes, joules, coulombs,
/// seconds, ohms. Currents are signed: positive in phase 1, negative in
/// phase 2, because the output bridge reverses the connection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DischargeResult {
    /// Voltage the capacitor started at.
    pub charge_voltage_v: f64,
    /// Patient resistance the shock was modelled into.
    pub patient_ohms: f64,
    /// Patient plus internal resistance: the whole loop.
    pub total_ohms: f64,
    /// How quickly the capacitor empties into this loop (R·C). After one
    /// time constant the voltage has fallen to about 37% of where it began.
    pub time_constant_s: f64,
    /// Length of phase 1.
    pub phase1_s: f64,
    /// Length of the gap between phases (no current).
    pub interphase_s: f64,
    /// Length of phase 2.
    pub phase2_s: f64,

    /// Energy in the capacitor before the shock (½·C·V²).
    pub stored_energy_j: f64,
    /// Highest current through the patient: the very start of phase 1.
    pub peak_current_a: f64,
    /// Current at the moment phase 1 is cut off.
    pub phase1_end_current_a: f64,
    /// Current at the start of phase 2 (negative: reversed). Its size
    /// equals the phase 1 end current, because the capacitor holds its
    /// voltage through the gap in this model.
    pub phase2_start_current_a: f64,
    /// Current at the moment phase 2 is cut off (negative).
    pub phase2_end_current_a: f64,
    /// Average current over phase 1 (charge moved divided by duration).
    pub phase1_average_current_a: f64,
    /// Electric charge moved through the patient in phase 1.
    pub phase1_charge_c: f64,
    /// Electric charge moved through the patient in phase 2 (size only).
    pub phase2_charge_c: f64,
    /// Fraction by which the current fell during phase 1 ("tilt"):
    /// 0.6 means it ended at 40% of where it started.
    pub phase1_tilt: f64,
    /// Fraction by which the current fell during phase 2.
    pub phase2_tilt: f64,
    /// Voltage left on the capacitor when the shock ends.
    pub final_capacitor_v: f64,

    /// Energy delivered to the patient in phase 1.
    pub patient_energy_phase1_j: f64,
    /// Energy delivered to the patient in phase 2.
    pub patient_energy_phase2_j: f64,
    /// Energy delivered to the patient in total. This is the "dose".
    pub patient_energy_j: f64,
    /// Energy turned to heat in the device's own discharge path.
    pub internal_energy_j: f64,
    /// Energy still in the capacitor at the end, which the device must dump
    /// into its internal discharge resistor afterwards.
    pub residual_energy_j: f64,
}

/// Model one shock.
///
/// `plan` gives the charge voltage and phase lengths, `patient_ohms` the
/// patient's resistance, and `config` the capacitor size and the device's
/// internal resistance.
///
/// Returns `None` if the numbers cannot describe a real circuit: capacitance
/// not above zero, a negative or non-finite resistance, or a loop with no
/// resistance at all.
#[must_use]
pub fn discharge(
    plan: &ShockPlan,
    patient_ohms: f64,
    config: &WaveformConfig,
) -> Option<DischargeResult> {
    let c = config.capacitance_uf * 1e-6;
    let ri = config.internal_ohms;
    let total = ri + patient_ohms;
    let usable = c.is_finite()
        && c > 0.0
        && ri.is_finite()
        && ri >= 0.0
        && patient_ohms.is_finite()
        && patient_ohms >= 0.0
        && total > 0.0;
    if !usable {
        return None;
    }

    let tau = c * total;
    let v0 = f64::from(plan.charge_voltage.0);
    let t1 = f64::from(plan.phase1.0) * 1e-6;
    let gap = f64::from(plan.interphase.0) * 1e-6;
    let t2 = f64::from(plan.phase2.0) * 1e-6;

    // Capacitor voltage at the end of phase 1 and at the end of phase 2.
    let v1 = v0 * exp(-t1 / tau);
    let v2 = v1 * exp(-t2 / tau);

    let half_c = 0.5 * c;
    let stored = half_c * v0 * v0;
    let from_cap1 = half_c * (v0 * v0 - v1 * v1);
    let from_cap2 = half_c * (v1 * v1 - v2 * v2);
    // Same current flows through both resistances, so energy divides in
    // proportion to resistance.
    let patient_share = patient_ohms / total;
    let patient1 = from_cap1 * patient_share;
    let patient2 = from_cap2 * patient_share;

    let q1 = c * (v0 - v1);
    let q2 = c * (v1 - v2);

    Some(DischargeResult {
        charge_voltage_v: v0,
        patient_ohms,
        total_ohms: total,
        time_constant_s: tau,
        phase1_s: t1,
        interphase_s: gap,
        phase2_s: t2,
        stored_energy_j: stored,
        peak_current_a: v0 / total,
        phase1_end_current_a: v1 / total,
        phase2_start_current_a: -v1 / total,
        phase2_end_current_a: -v2 / total,
        phase1_average_current_a: if t1 > 0.0 { q1 / t1 } else { 0.0 },
        phase1_charge_c: q1,
        phase2_charge_c: q2,
        phase1_tilt: if v0 > 0.0 { 1.0 - v1 / v0 } else { 0.0 },
        phase2_tilt: if v1 > 0.0 { 1.0 - v2 / v1 } else { 0.0 },
        final_capacitor_v: v2,
        patient_energy_phase1_j: patient1,
        patient_energy_phase2_j: patient2,
        patient_energy_j: patient1 + patient2,
        internal_energy_j: (from_cap1 + from_cap2) - (patient1 + patient2),
        residual_energy_j: half_c * v2 * v2,
    })
}

impl DischargeResult {
    /// Patient current, in amperes, at `t_s` seconds after the shock starts.
    ///
    /// Positive in phase 1, zero in the gap, negative in phase 2, zero
    /// before the start and after the end. Each phase includes its start
    /// instant and excludes its end instant. For the simulator to plot or
    /// integrate.
    #[must_use]
    pub fn current_at(&self, t_s: f64) -> f64 {
        let phase2_start = self.phase1_s + self.interphase_s;
        let end = phase2_start + self.phase2_s;
        // `t_s.is_nan()` is spelled out so a NaN time gives zero current.
        if t_s.is_nan() || t_s < 0.0 || t_s >= end {
            0.0
        } else if t_s < self.phase1_s {
            self.peak_current_a * exp(-t_s / self.time_constant_s)
        } else if t_s < phase2_start {
            0.0
        } else {
            self.phase2_start_current_a * exp(-(t_s - phase2_start) / self.time_constant_s)
        }
    }

    /// Total length of the shock, gap included, in seconds.
    #[must_use]
    pub fn total_s(&self) -> f64 {
        self.phase1_s + self.interphase_s + self.phase2_s
    }
}
