// SPDX-License-Identifier: GPL-3.0-or-later
//! Every design number for the shock, in one place.

use aed_types::{Micros, Ohms, PatientMode, Volts, MAX_PATIENT_OHMS, MIN_PATIENT_OHMS};

use crate::planner::plan_with;

/// All the design numbers that shape the shock.
///
/// The [`Default`] is the project's starting design point. Several values
/// are assumptions or placeholders; each field says which. None of them
/// has been checked against the defibrillator standard (IEC 60601-2-4),
/// which the project has not bought yet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveformConfig {
    /// Size of the storage capacitor, in microfarads.
    ///
    /// Default 100 µF: the design point of D-02 and the Philips/Avive class
    /// of device (research note `waveforms_energy_pads.md`, Q4).
    pub capacitance_uf: f64,

    /// Resistance of the device's own discharge path (capacitor, switches,
    /// wiring, pad leads), in ohms. Energy spent here heats the device
    /// instead of reaching the patient.
    ///
    /// Default 5 Ω. ASSUMPTION: no part has been chosen, so this is a
    /// guess. It matters most for low-resistance patients: at 25 Ω, one
    /// sixth of the energy taken from the capacitor stays in the device.
    pub internal_ohms: f64,

    /// Charge voltage for adult mode. Fixed, so charging can start before
    /// the pad resistance is finally known. See [`WaveformConfig::default`]
    /// for why 1,950 V.
    pub adult_charge: Volts,

    /// Charge voltage for child mode. Fixed, for the same reason. See
    /// [`WaveformConfig::default`] for why 1,150 V.
    pub child_charge: Volts,

    /// Highest voltage the capacitor may ever be charged to.
    ///
    /// Default 2,000 V: the top of the 1.7–2.0 kV range in
    /// docs/REQUIREMENTS.md, and what the 3.3 kV-class output switches in
    /// `miniaturization_hardware.md` (section 3) are sized around.
    pub max_charge: Volts,

    /// Shortest allowed first phase. Default 3 ms (ARCHITECTURE.md starting
    /// assumption, about the cardiac chronaxie; not from the standard).
    pub phase1_min: Micros,

    /// Longest allowed first phase. Default 12 ms (ARCHITECTURE.md
    /// starting assumption).
    pub phase1_max: Micros,

    /// Second phase length as a fraction of the first. Default 2/3
    /// (ARCHITECTURE.md starting assumption; to be replaced by a match to
    /// the published Philips waveform).
    pub phase2_ratio: f64,

    /// Shortest allowed second phase. Default 2 ms (2/3 of the 3 ms
    /// minimum first phase).
    pub phase2_min: Micros,

    /// Longest allowed second phase. Default 8 ms (2/3 of the 12 ms
    /// maximum first phase).
    pub phase2_max: Micros,

    /// Gap between the phases, while the output switches change over and
    /// no current flows. Default 0.5 ms. ASSUMPTION: depends on the switch
    /// hardware, which is not chosen.
    pub interphase: Micros,

    /// Longest allowed whole shock (phase 1 + gap + phase 2). Default
    /// 20 ms (ARCHITECTURE.md starting assumption).
    pub max_total: Micros,

    /// Highest current the device may push through the patient at any
    /// moment, in amperes. This is what the output switches must survive.
    ///
    /// Default 80 A. PLACEHOLDER: the research notes estimate about 78 A
    /// worst case for a 1,950 V design with no internal resistance, and the
    /// pulse ratings of the candidate 3.3 kV switches were not retrieved
    /// (`miniaturization_hardware.md`, section 3). Replace with the chosen
    /// switch's pulse rating, derated.
    pub max_peak_current_amps: f64,

    /// How far the delivered energy may be from the target and still count
    /// as on target, as a fraction (0.15 means ±15%).
    ///
    /// UNVERIFIED PLACEHOLDER: the real accuracy requirement is in
    /// IEC 60601-2-4, which the project has not bought yet.
    pub energy_tolerance: f64,
}

impl Default for WaveformConfig {
    /// The starting design point.
    ///
    /// # Why the charge voltages are what they are (model arithmetic)
    ///
    /// Symbols: C = 100 µF, internal resistance 5 Ω. Stored energy is
    /// ½·C·V². The share of the energy that reaches the patient is
    /// R_patient / (R_patient + 5). If the capacitor discharges for a total
    /// conducting time T (phase 1 + phase 2), the fraction of its energy
    /// that leaves it is 1 − e^(−2T / (C·R_total)).
    ///
    /// The longest allowed conducting time: 20 ms total minus 0.5 ms gap
    /// leaves 19.5 ms, i.e. phase 1 = 11.7 ms and phase 2 = 7.8 ms.
    ///
    /// **Adult (150 J target).** Three limits bracket the voltage:
    ///
    /// 1. Reach 150 J at 175 Ω within 19.5 ms. R_total = 180 Ω,
    ///    time constant 18 ms, so at most 1 − e^(−39/18) = 88.5% of the
    ///    stored energy leaves, and 175/180 = 97.2% of that reaches the
    ///    patient. Stored energy must be at least 150 / (0.885 × 0.972)
    ///    = 174.3 J, so V ≥ 1,867 V.
    /// 2. Reach 150 J at 25 Ω at all. Only 25/30 = 83.3% of the energy can
    ///    ever reach the patient here, however long the shock, so stored
    ///    energy must be at least 150 / 0.833 = 180 J, so V ≥ 1,897 V.
    ///    (This, not the 175 Ω case, is the binding lower limit, and it
    ///    exists only because of the assumed 5 Ω internal resistance.)
    /// 3. Stay within limits at 25 Ω: peak current V / 30 Ω must not
    ///    exceed 80 A (V ≤ 2,400 V), and V must not exceed the 2,000 V
    ///    ceiling. Overshoot at the shortest allowed shock (3 ms + 2 ms):
    ///    80.4% of stored energy reaches the patient, which stays inside
    ///    +15% up to V = 2,072 V, so the 2,000 V ceiling binds.
    ///
    /// The window is 1,897–2,000 V. 1,950 V sits in the middle: 2.8% above
    /// the floor (room for the internal resistance to be a little higher
    /// than guessed) and 2.5% below the ceiling. It stores 190 J and gives
    /// 65 A peak into 25 Ω.
    ///
    /// **Child (50 J target).** The same three limits:
    ///
    /// 1. 175 Ω: stored ≥ 50 / (0.885 × 0.972) = 58.1 J, so V ≥ 1,078 V.
    /// 2. 25 Ω reachable at all: stored ≥ 50 / 0.833 = 60 J, so
    ///    V ≥ 1,095 V.
    /// 3. 25 Ω overshoot at the shortest allowed shock: 0.804 × stored must
    ///    stay within 50 J + 15% = 57.5 J, so stored ≤ 71.6 J, V ≤ 1,197 V.
    ///    Here, unlike adult mode, the minimum phase length is what limits
    ///    the voltage from above.
    ///
    /// The window is 1,095–1,197 V; 1,150 V is near its middle. It stores
    /// 66 J and gives 38 A peak into 25 Ω. At 25 Ω the shock is then
    /// already slightly over 50 J at the shortest allowed length (+6% to
    /// +7% from 25 to about 35 Ω); the design table shows it.
    fn default() -> Self {
        Self {
            capacitance_uf: 100.0,
            internal_ohms: 5.0,
            adult_charge: Volts(1950),
            child_charge: Volts(1150),
            max_charge: Volts(2000),
            phase1_min: Micros(3_000),
            phase1_max: Micros(12_000),
            phase2_ratio: 2.0 / 3.0,
            phase2_min: Micros(2_000),
            phase2_max: Micros(8_000),
            interphase: Micros(500),
            max_total: Micros(20_000),
            max_peak_current_amps: 80.0,
            energy_tolerance: 0.15,
        }
    }
}

/// Why a [`WaveformConfig`] was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// The capacitor size is zero, negative or not a number.
    CapacitanceInvalid,
    /// The internal resistance is negative or not a number.
    InternalResistanceInvalid,
    /// A mode's charge voltage is zero.
    ChargeVoltageZero,
    /// A mode's charge voltage is above the allowed maximum.
    ChargeVoltageAboveMax,
    /// The peak current limit is zero, negative or not a number.
    PeakCurrentLimitInvalid,
    /// A mode's charge voltage would push more than the allowed peak
    /// current through the lowest patient resistance (25 Ω).
    PeakCurrentTooHigh,
    /// The phase 1 limits are zero or the wrong way round.
    Phase1LimitsInvalid,
    /// The phase 2 ratio is zero, negative or not a number.
    Phase2RatioInvalid,
    /// The phase 2 limits are zero or the wrong way round.
    Phase2LimitsInvalid,
    /// Even the shortest allowed shock does not fit in the total-duration cap.
    TotalTooShort,
    /// The energy tolerance is not between 0 and 1 (exclusive).
    ToleranceInvalid,
    /// With these numbers the planner would give a shorter shock to a
    /// higher-resistance patient somewhere in 25–175 Ω.
    NotMonotonic,
}

impl WaveformConfig {
    /// The fixed charge voltage for a mode.
    #[must_use]
    pub const fn charge_voltage(&self, mode: PatientMode) -> Volts {
        match mode {
            PatientMode::Adult => self.adult_charge,
            PatientMode::Child => self.child_charge,
        }
    }

    /// The energy the planner aims to deliver to the patient, in joules,
    /// taken from [`PatientMode::target_energy`] (150 J adult, 50 J child).
    #[must_use]
    pub fn target_joules(mode: PatientMode) -> f64 {
        f64::from(mode.target_energy().0) / 10.0
    }

    /// Check the numbers make sense together.
    ///
    /// # Errors
    /// Returns the first problem found, as a [`ConfigError`].
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(self.capacitance_uf.is_finite() && self.capacitance_uf > 0.0) {
            return Err(ConfigError::CapacitanceInvalid);
        }
        if !(self.internal_ohms.is_finite() && self.internal_ohms >= 0.0) {
            return Err(ConfigError::InternalResistanceInvalid);
        }
        if !(self.max_peak_current_amps.is_finite() && self.max_peak_current_amps > 0.0) {
            return Err(ConfigError::PeakCurrentLimitInvalid);
        }
        for mode in [PatientMode::Adult, PatientMode::Child] {
            let v = self.charge_voltage(mode);
            if v.0 == 0 {
                return Err(ConfigError::ChargeVoltageZero);
            }
            if v > self.max_charge {
                return Err(ConfigError::ChargeVoltageAboveMax);
            }
            let peak = f64::from(v.0) / (self.internal_ohms + f64::from(MIN_PATIENT_OHMS.0));
            if peak > self.max_peak_current_amps {
                return Err(ConfigError::PeakCurrentTooHigh);
            }
        }
        if self.phase1_min.0 == 0 || self.phase1_min > self.phase1_max {
            return Err(ConfigError::Phase1LimitsInvalid);
        }
        if !(self.phase2_ratio.is_finite() && self.phase2_ratio > 0.0) {
            return Err(ConfigError::Phase2RatioInvalid);
        }
        if self.phase2_min.0 == 0 || self.phase2_min > self.phase2_max {
            return Err(ConfigError::Phase2LimitsInvalid);
        }
        if !self.fits_total(self.phase1_min.0) {
            return Err(ConfigError::TotalTooShort);
        }
        if !(self.energy_tolerance.is_finite()
            && self.energy_tolerance > 0.0
            && self.energy_tolerance < 1.0)
        {
            return Err(ConfigError::ToleranceInvalid);
        }
        self.check_monotonic()
    }

    /// Plan every whole-ohm resistance in range and check durations never
    /// shrink as resistance rises.
    fn check_monotonic(&self) -> Result<(), ConfigError> {
        for mode in [PatientMode::Adult, PatientMode::Child] {
            let mut previous = (0_u32, 0_u32);
            for r in MIN_PATIENT_OHMS.0..=MAX_PATIENT_OHMS.0 {
                let Ok(planned) = plan_with(self, mode, Ohms(r)) else {
                    return Err(ConfigError::NotMonotonic);
                };
                let now = (planned.plan.phase1.0, planned.plan.phase2.0);
                if now.0 < previous.0 || now.1 < previous.1 {
                    return Err(ConfigError::NotMonotonic);
                }
                previous = now;
            }
        }
        Ok(())
    }

    /// Phase 2 length (µs) that goes with a phase 1 length (µs): the ratio,
    /// rounded to the microsecond, then held inside the phase 2 limits.
    pub(crate) fn phase2_for(&self, phase1_us: u32) -> u32 {
        let raw = libm::round(f64::from(phase1_us) * self.phase2_ratio);
        // `as` saturates for out-of-range floats, and NaN becomes 0.
        let raw = raw as u32;
        raw.max(self.phase2_min.0).min(self.phase2_max.0)
    }

    /// Whether a shock with this phase 1 (µs) fits in the total-duration cap.
    pub(crate) fn fits_total(&self, phase1_us: u32) -> bool {
        let total = u64::from(phase1_us)
            + u64::from(self.interphase.0)
            + u64::from(self.phase2_for(phase1_us));
        total <= u64::from(self.max_total.0)
    }
}
