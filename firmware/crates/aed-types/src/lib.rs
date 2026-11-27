// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared vocabulary for the Open AED firmware.
//!
//! This crate is the contract between the other crates. It holds units,
//! the small enums every part needs, and the traits that let the rescue
//! sequence (`aed-core`) use rhythm analysis, compression sensing and shock planning without
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
    ///
    /// This is the **full** reset: the signal-cleaning filters start again
    /// from nothing and must settle before any signal is analysed.
    fn reset(&mut self);

    /// Pass one sample through the signal-cleaning filters **without
    /// analysing it**, so that the filters are already settled when a fresh
    /// analysis window is started with [`RhythmAnalyzer::restart_window`]
    /// (decision D-47). The rescue sequence calls this, with the pads on,
    /// for every sample it does not pass to [`RhythmAnalyzer::push`]: during
    /// CPR and while compressions hold an analysis back.
    ///
    /// The default does nothing, which is always safe: an analyser that
    /// does not track keeps the default `restart_window`, which is a full
    /// reset.
    fn track(&mut self, sample: EcgSample) {
        let _ = sample;
    }

    /// Start a fresh analysis window **now**, keeping the signal-cleaning
    /// filters' state (decision D-47). Everything about the previous window
    /// (signal, measurements, confirmation history) is forgotten, exactly
    /// as after [`RhythmAnalyzer::reset`]; only the filters carry on. An
    /// implementation must still wait for whatever settling its filters
    /// need since the last full reset, plus any guard it needs against the
    /// tail of what came before.
    ///
    /// The rescue sequence calls this only when compression sensing
    /// confirms the chest is still (a good impedance signal and
    /// `HandsOff`); in every other case it calls `reset`.
    ///
    /// The default falls back to [`RhythmAnalyzer::reset`], so an analyser
    /// that does not implement it behaves exactly as before.
    fn restart_window(&mut self) {
        self.reset();
    }
}

// ---------------------------------------------------------------------------
// Chest-compression sensing through the pads (decisions D-14 and D-36)
// ---------------------------------------------------------------------------

/// Pad-resistance (impedance) sampling rate for compression sensing, in
/// samples per second.
///
/// 125 per second: one sample every 8 ms. A compression at the fastest rate
/// worth caring about (about 200 per minute, 3.3 Hz) is then about 38
/// samples long, and the one open recording set the detector was checked on
/// (Paris Fire Brigade, Schiller defibrillators) is sampled at exactly this
/// rate, so it can be fed in without resampling. Not from the standard.
pub const IMPEDANCE_SAMPLE_RATE_HZ: u32 = 125;

/// One pad-impedance sample from the front end, for compression sensing.
///
/// This is the small, fast measurement taken with a high-frequency test
/// current between the pads (decision D-36), not the heart signal. Its value
/// is the whole resistance between the pads, not just the change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpedanceSample {
    /// A good reading, in milliohms (80 Ω is `Milliohms(80_000)`).
    Milliohms(u32),
    /// The front end's input was at the end of its range (for example during
    /// and just after a shock, or with a pad lifting): the value means nothing.
    Saturated,
}

/// What compression sensing says about the patient's chest right now.
///
/// `Unknown` must be treated like `Compressing` wherever a shock decision
/// depends on it: if the device cannot tell, it must not assume that hands
/// are off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestState {
    /// No compressions and no other movement: hands are off the chest.
    HandsOff,
    /// Regular chest compressions are being given.
    Compressing,
    /// Cannot tell: the signal is missing, saturated, settling, noisy, or
    /// shows movement that is not (yet) a clear run of compressions.
    Unknown,
}

/// Why the impedance signal cannot be read, if it cannot. `Good` does not
/// mean hands are off, only that the signal itself is usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpedanceSignal {
    /// Usable.
    Good,
    /// No samples are arriving (reported by the rescue sequence, which
    /// watches the time of the last sample).
    Missing,
    /// The front end is saturated.
    Saturated,
    /// The resistance is outside the 25 to 250 Ω pad-sensing range.
    OutOfRange,
    /// The reading has not changed at all for too long: a real front end
    /// always shows a little noise, so a frozen value means a dead channel.
    Stuck,
    /// Recovering after one of the above; the filters are settling.
    Settling,
    /// Fast electrical noise far above the normal level.
    Noisy,
}

impl ImpedanceSignal {
    /// True for the conditions that point at a broken or disconnected
    /// channel rather than at something the responder is doing.
    #[must_use]
    pub const fn is_channel_problem(self) -> bool {
        matches!(
            self,
            ImpedanceSignal::Missing
                | ImpedanceSignal::Saturated
                | ImpedanceSignal::OutOfRange
                | ImpedanceSignal::Stuck
        )
    }
}

/// Something compression sensing noticed, for the event record and the CPR
/// measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestEvent {
    /// One chest compression was detected.
    Compression {
        /// Its size, peak to peak after filtering, in milliohms.
        amplitude_mohm: u32,
        /// Time since the previous detected compression, ms (`u32::MAX` if
        /// this is the first since the detector was reset). A long gap is a
        /// pause in CPR.
        since_previous_ms: u32,
    },
}

/// Compression sensing as the rescue sequence sees it (decision D-36).
///
/// The rescue sequence feeds every pad-impedance sample in (at
/// [`IMPEDANCE_SAMPLE_RATE_HZ`]) and reads [`ChestSensor::state`] to decide
/// whether a rhythm analysis is looking at a still chest. Like
/// [`RhythmAnalyzer`], this keeps the rescue sequence independent of how
/// compressions are detected.
pub trait ChestSensor {
    /// False if this device has no compression sensing at all, in which case
    /// the rescue sequence behaves exactly as it did without it (heart signal
    /// only). Defaults to true.
    fn fitted(&self) -> bool {
        true
    }

    /// Feed one sample. Returns an event when one happens.
    fn push(&mut self, sample: ImpedanceSample) -> Option<ChestEvent>;

    /// The current chest state.
    fn state(&self) -> ChestState;

    /// Whether the signal is usable, and if not, why.
    fn signal(&self) -> ImpedanceSignal;

    /// Compression rate over the last few compressions, per minute, while
    /// compressions are being given.
    fn rate_per_min(&self) -> Option<u16>;

    /// Milliseconds since the last detected compression (0 while
    /// compressing; counts from the last `reset` if there has been none).
    fn ms_since_compression(&self) -> u32;

    /// Share of the recent rolling period (see the implementation) during
    /// which compressions were being given, in tenths of a percent.
    fn time_on_chest_permille(&self) -> u16;

    /// Forget everything (for example when the pads are reattached).
    fn reset(&mut self);
}

/// A [`ChestSensor`] for a device without compression sensing. It reports
/// `fitted() == false`, so the rescue sequence ignores it entirely.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoChestSensor;

impl ChestSensor for NoChestSensor {
    fn fitted(&self) -> bool {
        false
    }
    fn push(&mut self, _sample: ImpedanceSample) -> Option<ChestEvent> {
        None
    }
    fn state(&self) -> ChestState {
        ChestState::Unknown
    }
    fn signal(&self) -> ImpedanceSignal {
        ImpedanceSignal::Missing
    }
    fn rate_per_min(&self) -> Option<u16> {
        None
    }
    fn ms_since_compression(&self) -> u32 {
        0
    }
    fn time_on_chest_permille(&self) -> u16 {
        0
    }
    fn reset(&mut self) {}
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
