// SPDX-License-Identifier: GPL-3.0-or-later
//! Every number the rhythm analysis depends on, in one place.
//!
//! **These are starting values, not validated settings.** Each one has to be
//! tuned and then proven on recorded heart signals (first the public PhysioNet
//! databases, then real cardiac-arrest recordings). Values marked
//! "UNVERIFIED" were recalled from memory of the literature and have not been
//! checked against the original paper. Values marked "TUNED ON SYNTHETIC"
//! were picked so that the made-up test signals in [`crate::synth`] come out
//! right; that says nothing about real patients.

use aed_types::ECG_SAMPLE_RATE_HZ;

// ---------------------------------------------------------------------------
// Timing
// ---------------------------------------------------------------------------

/// Samples per second, as a whole number (250, from `aed-types`).
pub const SAMPLES_PER_SECOND: usize = ECG_SAMPLE_RATE_HZ as usize;

/// Samples per second, as a decimal number for the maths.
pub const SAMPLE_RATE_HZ: f32 = ECG_SAMPLE_RATE_HZ as f32;

/// Length of signal looked at for each verdict, in seconds.
/// Source: Amann et al. 2005 report the VF filter on 4 s and 8 s windows;
/// Figuera et al. 2016 found window length made little difference. Starting value.
pub const WINDOW_SECONDS: usize = 4;

/// How often a new verdict is produced, in seconds. Windows overlap.
pub const STEP_SECONDS: usize = 1;

/// Window length in samples.
pub const WINDOW_SAMPLES: usize = WINDOW_SECONDS * SAMPLES_PER_SECOND;

/// Step length in samples.
pub const STEP_SAMPLES: usize = STEP_SECONDS * SAMPLES_PER_SECOND;

/// Number of steps that make up one window (window must be a whole number of steps).
pub const STEPS_PER_WINDOW: usize = WINDOW_SECONDS / STEP_SECONDS;

/// Samples thrown away after start or reset while the filters settle (1 s). Own choice.
pub const WARMUP_SAMPLES: usize = SAMPLES_PER_SECOND;

/// How many windows in a row must say "shockable" before the analyser
/// reports `Shockable`. With 1 s steps, 3 windows cover 6 s of signal.
/// Own choice (safety pattern chosen by the project), not from a standard.
pub const CONFIRM_WINDOWS: u8 = 3;

// ---------------------------------------------------------------------------
// Pre-filtering
// ---------------------------------------------------------------------------

/// Lowest frequency kept, Hz. Removes breathing and electrode drift.
/// Common AED analysis band is roughly 1-30 Hz (UNVERIFIED, general practice).
pub const HIGH_PASS_HZ: f32 = 1.0;

/// Highest frequency kept, Hz.
pub const LOW_PASS_HZ: f32 = 30.0;

/// Mains-hum frequencies removed (Europe 50 Hz, North America 60 Hz).
pub const MAINS_HZ: [f32; 2] = [50.0, 60.0];

/// Sharpness of the mains-hum notches (higher is narrower). Own choice.
pub const NOTCH_Q: f32 = 8.0;

/// Signal above this frequency, Hz, is treated as noise (muscle, electrical)
/// for the signal-quality check. Own choice.
pub const NOISE_BAND_HZ: f32 = 40.0;

// ---------------------------------------------------------------------------
// Amplitude and signal-quality checks
// ---------------------------------------------------------------------------

/// Peak-to-peak below this many microvolts counts as flatline (asystole) and is
/// never shocked. Source: the AHA 1997 recommendations (Kerber et al.) as
/// commonly reproduced; UNVERIFIED against the original text.
pub const ASYSTOLE_MAX_PP_UV: f32 = 100.0;

/// VF smaller than this (peak-to-peak, microvolts) is "fine VF". The AHA 1997
/// table puts fine VF at about < 200 µV and asks only that performance on it be
/// reported (UNVERIFIED). This analyser does not shock fine VF: it answers
/// `Indeterminate`. That is a deliberate lean toward not shocking, to revisit.
pub const FINE_VF_MAX_PP_UV: f32 = 200.0;

/// Peak-to-peak above this is not believable as a heart signal at the pads
/// (movement, lead problem), so the answer is `Indeterminate`.
/// Own choice; UNVERIFIED.
pub const MAX_PLAUSIBLE_PP_UV: f32 = 8_000.0;

/// A raw sample at or beyond this size (either sign) is treated as clipped by
/// the input electronics. Depends on the front-end chip, not chosen yet.
pub const SATURATION_UV: i16 = 32_000;

/// If more than this fraction of a window is clipped, the answer is `Indeterminate`. Own choice.
pub const MAX_SATURATED_FRACTION: f32 = 0.01;

/// If the energy above [`NOISE_BAND_HZ`] is more than this fraction of the
/// energy in the analysis band, the signal is too noisy: `Indeterminate`.
/// TUNED ON SYNTHETIC.
pub const MAX_NOISE_RATIO: f32 = 0.15;

/// If more than this fraction of the analysis-band power sits below
/// [`ARTIFACT_BAND_MAX_HZ`], the window looks like chest compressions or
/// movement, and the answer is `Indeterminate`. TUNED ON SYNTHETIC.
pub const MAX_LOW_BAND_FRACTION: f32 = 0.5;

/// Upper edge of the "compression artifact" band, Hz. Guideline compression
/// rate is 100-120 per minute (1.7-2 Hz); UNVERIFIED against current guidelines.
pub const ARTIFACT_BAND_MAX_HZ: f32 = 2.4;

// ---------------------------------------------------------------------------
// Method family 1: time domain (threshold-crossing and band-pass count)
// ---------------------------------------------------------------------------

/// Level, as a fraction of each second's largest absolute value, used by the
/// threshold-crossing sample count. Source: TCSC, Arafat, Chowdhury & Hasan 2011
/// (uses 0.2); UNVERIFIED.
pub const TCSC_LEVEL: f32 = 0.2;

/// Fraction of samples above [`TCSC_LEVEL`] needed to vote VF. TUNED ON SYNTHETIC.
pub const TCSC_VF_MIN: f32 = 0.45;

/// Centre of the band-pass used by the count feature, Hz. Source: Jekova 2007
/// count features use a band-pass centred near 14 Hz; UNVERIFIED.
pub const COUNT_CENTRE_HZ: f32 = 14.0;

/// Sharpness of that band-pass. Own choice.
pub const COUNT_Q: f32 = 1.0;

/// Level, as a fraction of each second's largest band-passed value, for the count.
/// Jekova's "Count1" uses half the maximum; UNVERIFIED.
pub const COUNT_LEVEL: f32 = 0.5;

/// Fraction of samples above [`COUNT_LEVEL`] needed to vote VF. TUNED ON SYNTHETIC.
pub const COUNT_VF_MIN: f32 = 0.18;

// ---------------------------------------------------------------------------
// Method family 2: frequency domain (VF filter leakage and spectrum)
// ---------------------------------------------------------------------------

/// VF filter leakage below this votes VF. Source: Kuo & Dillman 1978, threshold
/// as reported by Amann et al. 2005; UNVERIFIED.
pub const LEAKAGE_VF_MAX: f32 = 0.625;

/// Lowest frequency in the spectrum, Hz.
pub const SPECTRUM_MIN_HZ: f32 = 1.0;

/// Spacing of spectrum lines, Hz (1 / window length).
pub const SPECTRUM_STEP_HZ: f32 = 1.0 / WINDOW_SECONDS as f32;

/// Number of spectrum lines, covering [`SPECTRUM_MIN_HZ`] to 30 Hz.
pub const SPECTRUM_BINS: usize = 117;

/// The strongest frequency is searched for only up to here, Hz.
pub const DOMINANT_SEARCH_MAX_HZ: f32 = 10.0;

/// Strongest frequency must be above this to vote VF/VT, Hz. Just above the
/// compression band, and just below rapid VT at 150/min (2.5 Hz). TUNED ON SYNTHETIC.
pub const DOMINANT_VF_MIN_HZ: f32 = 2.4;

/// Strongest frequency must be below this to vote VF/VT, Hz. VF is usually
/// described as 3-10 Hz (UNVERIFIED, general literature).
pub const DOMINANT_VF_MAX_HZ: f32 = 9.0;

/// Share of power within 0.6x-1.4x of the strongest frequency needed to vote VF.
/// Idea from the SPEC algorithm (Barro et al. 1989), simplified. TUNED ON SYNTHETIC.
pub const CONCENTRATION_VF_MIN: f32 = 0.4;
