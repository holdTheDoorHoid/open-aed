// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared vocabulary for the Open AED firmware.
//!
//! This crate is the contract between the other crates. It holds units,
//! the small enums every part needs, and the two traits that let the rescue
//! sequence (`aed-core`) use rhythm analysis and shock planning without
//! depending on how they are implemented.
//!
//! NOT A MEDICAL DEVICE. See SAFETY.md at the repository root.
#![no_std]

/// Heart-signal sampling rate used throughout the firmware, in samples per second.
pub const ECG_SAMPLE_RATE_HZ: u32 = 250;

/// One heart-signal sample, in microvolts.
pub type EcgSample = i16;

/// Electrical potential in volts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Volts(pub u16);

/// Resistance in ohms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ohms(pub u16);

/// Energy in tenths of a joule (so 150 J is `Decijoules(1500)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Decijoules(pub u16);

/// A duration in microseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Micros(pub u32);

/// A point in time: milliseconds since the device was switched on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Millis(pub u32);

impl Millis {
    /// Milliseconds elapsed since `earlier`, or zero if `earlier` is in the future.
    #[must_use]
    pub fn since(self, earlier: Millis) -> u32 {
        self.0.saturating_sub(earlier.0)
    }
}

/// Lowest patient resistance the device must shock into (defibrillator
/// standard test range). This is also the bottom of the pad-sensing range:
/// anything lower is treated as pads shorted or touching, not as a patient.
pub const MIN_PATIENT_OHMS: Ohms = Ohms(25);
/// Highest patient resistance the device will shock into: the top of the
/// **shock range** (25 to 175 Ω). The shock planner refuses anything above
/// it and the supervisor gives no permit above it. Unchanged by D-32.
pub const MAX_PATIENT_OHMS: Ohms = Ohms(175);
/// Highest pad resistance that still counts as "pads on a patient": the top
/// of the **pad-sensing range** (25 to 250 Ω, decision D-32).
///
/// Between 175 and 250 Ω the pads are taken to be on the chest but making
/// poor contact (dry gel, hair, not pressed down). The device then analyses
/// as normal and asks for the pads to be pressed firmly, but shocks only if
/// the resistance has come down into the shock range by the moment of
/// delivery. Provisional: 250 Ω, from the owner's decision; other devices
/// are reported to accept 180 to 230 Ω (not checked against the standard).
pub const PAD_SENSE_MAX_OHMS: Ohms = Ohms(250);

/// True if a pad resistance is in the shock range (25 to 175 Ω): the
/// device may shock into it.
#[must_use]
pub const fn in_shock_range(ohms: Ohms) -> bool {
    ohms.0 >= MIN_PATIENT_OHMS.0 && ohms.0 <= MAX_PATIENT_OHMS.0
}

/// True if a pad resistance means the pads are on a patient (25 to 250 Ω),
/// whether or not it is low enough to shock into (see [`in_shock_range`]).
#[must_use]
pub const fn pads_sensed(ohms: Ohms) -> bool {
    ohms.0 >= MIN_PATIENT_OHMS.0 && ohms.0 <= PAD_SENSE_MAX_OHMS.0
}

/// Which patient the device is set for (decision D-04: child mode is a switch, not a different pad).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientMode {
    /// Adult: 150 J target, and only 150 J (D-03: no higher-energy adult
    /// setting is planned or allowed for).
    Adult,
    /// Child: 50 J target (D-04).
    Child,
}

impl PatientMode {
    /// Target delivered energy for this mode.
    #[must_use]
    pub const fn target_energy(self) -> Decijoules {
        match self {
            PatientMode::Adult => Decijoules(1500),
            PatientMode::Child => Decijoules(500),
        }
    }
}

/// The answer from rhythm analysis.
///
/// `Indeterminate` is a first-class answer and must never lead to a shock.
/// How the rescue sequence acts on each answer is described on
/// [`RhythmAnalyzer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmDecision {
    /// A rhythm a shock can correct (ventricular fibrillation or rapid ventricular tachycardia).
    Shockable,
    /// A rhythm a shock must not be given for (normal rhythm, flatline, others).
    NonShockable,
    /// The analysis cannot tell (noise, movement, conflicting methods).
    Indeterminate,
}

/// The plan for one shock: how high to charge and how long each phase lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShockPlan {
    /// Voltage the storage capacitor is charged to before the shock.
    pub charge_voltage: Volts,
    /// Length of the first (forward) phase.
    pub phase1: Micros,
    /// Gap between the phases, with no current flowing.
    pub interphase: Micros,
    /// Length of the second (reversed) phase.
    pub phase2: Micros,
    /// Energy the plan expects to deliver to the patient (model figure).
    pub expected_energy: Decijoules,
}

/// Why a shock plan could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    /// Pad resistance is below the range for a patient (pads shorted or touching).
    ImpedanceTooLow,
    /// Pad resistance is above the range for a patient (pads off, dry, or not connected).
    ImpedanceTooHigh,
}

/// Rhythm analysis as the rescue sequence sees it.
///
/// # How the rescue sequence treats each answer
///
/// **While analysing** (before any shock is advised; the capacitor is
/// normally already charging, decision D-29):
///
/// * `Shockable` advises a shock: straight to the countdown if the capacitor
///   is charged, otherwise once it is.
/// * `NonShockable` is final: "No shock advised", then CPR.
/// * `Indeterminate` ("not sure") is **not** final. Movement, such as
///   compressions that went on into the analysis, is the usual cause, so the
///   device asks again for hands off and keeps analysing. Only `Shockable`
///   or `NonShockable` ends the analysis early. If neither arrives within
///   the analysis time limit, the analysis is "inconclusive": no shock, and
///   the responder is told to start CPR.
/// * If `Indeterminate` and `Shockable` arrive together (before the device
///   has acted on either), "not sure" wins: no charging on that answer.
///
/// **Once a shock is advised** (waiting for the charge, counting down,
/// delivering): only `Shockable` keeps the shock going. Any other answer,
/// `Indeterminate` included, abandons the shock at once and empties the
/// capacitor, even if later answers say `Shockable` again.
///
/// **During CPR** answers are not used: the signal is full of compressions.
pub trait RhythmAnalyzer {
    /// Feed one heart-signal sample (at [`ECG_SAMPLE_RATE_HZ`]).
    ///
    /// Returns `Some(decision)` each time a new decision is ready, otherwise `None`.
    ///
    /// `None` means "still analysing": not enough signal yet, or a shockable
    /// rhythm has been seen but is not yet confirmed. Every `Some` is an
    /// answer the rescue sequence may act on at once.
    ///
    /// Once decisions start, an implementation must produce one at least every
    /// two seconds while its answer is unchanged, because the rescue sequence
    /// refuses to shock on a confirmation older than a few seconds.
    fn push(&mut self, sample: EcgSample) -> Option<RhythmDecision>;

    /// Forget everything seen so far (for example after a shock, or when pads are reattached).
    fn reset(&mut self);
}

/// Shock planning as the rescue sequence sees it.
pub trait ShockPlanner {
    /// The voltage the capacitor is charged to for this mode. Fixed per mode,
    /// so charging can begin before the resistance is finally known.
    fn charge_voltage(&self, mode: PatientMode) -> Volts;

    /// Make the plan for a patient of the measured resistance.
    ///
    /// # Errors
    /// Returns [`PlanError`] if the resistance is outside the patient range.
    fn plan(&self, mode: PatientMode, impedance: Ohms) -> Result<ShockPlan, PlanError>;
}
