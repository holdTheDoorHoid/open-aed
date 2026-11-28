// SPDX-License-Identifier: GPL-3.0-or-later
//! Every number the rhythm analysis depends on, in one place.
//!
//! **These are starting values, not validated settings.** Each one has to be
//! tuned and then proven on recorded heart signals (first the public PhysioNet
//! databases, then real cardiac-arrest recordings). Values marked
//! "UNVERIFIED" were recalled from memory of the literature and have not been
//! checked against the original paper. Values marked "TUNED ON SYNTHETIC"
//! were picked so that the made-up test signals in [`crate::synth`] come out
//! right; that says nothing about real patients. Values marked "TUNED ON
//! REAL DATA" were chosen on the development half of the public PhysioNet
//! recordings with leave-one-patient-out cross-validation and then tested
//! once on the held-out half (`crates/aed-eval/TUNING_LOG.md`, REPORT.md);
//! they are fitted to hospital and Holter recordings of a few dozen
//! patients, not to cardiac arrests through pads.

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

/// Samples thrown away when a fresh analysis window is started on filters
/// that are already running (decision D-47, `RhythmAnalyzer::restart_window`):
/// 62 samples, about a quarter of a second. Own choice, from measurement.
///
/// Why the full [`WARMUP_SAMPLES`] second is not needed then: the warm-up
/// exists for the filters to settle after they start from nothing (the
/// 1 Hz high-pass is already primed on its first sample, so that is mostly
/// the notches and the low-pass, and any electrode offset step). When the
/// rescue sequence keeps the filters running through CPR, they are already
/// settled; what is left is the tail of the compression artifact in the
/// filters. Measured on made-up artifact (3 shapes, 60-180 per minute,
/// stopped abruptly at 8 points of a compression;
/// `tests/fresh_window.rs`): the leftover falls below 10 % of the
/// artifact's size after 0.45 s, 3 % after 0.74 s and 1 % after 0.87 s;
/// its largest peak to peak in a window starting at the stop is about the
/// artifact's own size, and about 14 % of it in a window starting 0.25 s
/// later. Over 13,200 runs of normal rhythm (40-180/min) and flatline with
/// artifacts of 0.5 to 20 mV ending exactly as the window starts, no
/// window sequence ever reached "shockable", with this guard or with none;
/// VF and rapid VT were still detected in every run. The quarter second is
/// therefore a margin, not a measured necessity: it keeps the abrupt end of
/// the artifact (its sharpest, most broadband part) out of the first
/// window, at a cost of 0.25 s. In the device the chest sensor's own quiet
/// period (1 s without a compression, `aed-cpr`) comes on top.
///
/// After a full reset the remaining warm-up still applies in full: a fresh
/// window waits for whichever is longer, this guard or what is left of the
/// one-second warm-up.
pub const FRESH_WINDOW_GUARD_SAMPLES: usize = SAMPLES_PER_SECOND / 4;

/// Confirmation: "shockable" is reported once the current window is
/// shockable and at least this many of the last [`CONFIRM_SPAN`] windows
/// (including the current one) were. With 1 s steps the earliest
/// confirmation is still 3 windows in a row (7 s after the analysis starts).
///
/// Before tuning on real data this was 3 windows **in a row**. On real VF a
/// single "not sure" window among shockable ones is common (the two method
/// families disagree on about a quarter of real VF windows), so "in a row"
/// threw away many analyses. TUNED ON REAL DATA (development half,
/// leave-one-patient-out cross-validation, 2026-11-27; TUNING_LOG.md E4c):
/// 3 of the last 5 was chosen in 51 of 53 folds.
///
/// Nothing else about how decisions are announced changed: every window
/// that is not shockable is still announced at once ("not shockable" ends
/// the analysis; "not sure" keeps it going), so a rhythm change still
/// cancels a shock being prepared as quickly as before (finding F-4). A
/// variant that let one "not sure" window pass during the countdown was
/// tried and rejected: it let the simulator deliver a shock 2.5 s after VF
/// had turned into a normal rhythm (TUNING_LOG.md E4, E4c).
pub const CONFIRM_WINDOWS: u8 = 3;

/// See [`CONFIRM_WINDOWS`]. TUNED ON REAL DATA (E4c).
pub const CONFIRM_SPAN: u8 = 5;

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

/// Second flatline test, for real recordings whose baseline is never truly
/// flat: root-mean-square of the cleaned window below this many microvolts,
/// together with a noise-like shape (VF filter leakage above
/// [`ASYSTOLE_MIN_LEAKAGE`]), is asystole.
///
/// TUNED ON REAL DATA (development half only, 2026-11-27; see
/// `crates/aed-eval/TUNING_LOG.md`, step T-3). Annotated asystole in the
/// development recordings has a cleaned peak-to-peak of 100-200 µV, mostly
/// recorder noise, so the 100 µV rule above caught 1 % of its windows; this
/// rule catches 37 %. 25 µV was the largest value tried that flagged no
/// coarse-VF or flutter window and under 0.1 % of VT windows. **Thin
/// evidence:** nearly all of it is one patient (vfdb 427), and only four
/// development patients have scored asystole at all. A regular wave of the
/// same size (for example fine VF) has a low leakage and is not called
/// flatline. The device's own noise floor is unknown, and this value must
/// be re-checked against it.
pub const ASYSTOLE_MAX_RMS_UV: f32 = 25.0;

/// See [`ASYSTOLE_MAX_RMS_UV`]. Real asystole windows in the development data
/// have leakage around 0.85; made-up fine VF stays below 0.4. Own choice
/// between the two, checked on the development data (T-3).
pub const ASYSTOLE_MIN_LEAKAGE: f32 = 0.6;

/// VF smaller than this (peak-to-peak, microvolts) is "fine VF". The AHA 1997
/// table puts fine VF at about < 200 µV and asks only that performance on it be
/// reported (UNVERIFIED). This analyser does not shock fine VF: it answers
/// `Indeterminate`. That is a deliberate lean toward not shocking, to revisit.
pub const FINE_VF_MAX_PP_UV: f32 = 200.0;

/// Peak-to-peak above this is not believable as a heart signal at the pads
/// (movement, lead problem), so the answer is `Indeterminate`.
///
/// Was 8,000 µV (own choice). On real recordings that rejected VF (and, in
/// the well-calibrated vfdb, VT and other rhythms over 8 mV). Part of that
/// is cudb's doubtful amplitude calibration (its gain is only "nominal";
/// its signals read about twice vfdb's), so the gain from raising it is
/// partly an artefact of that database. TUNED ON REAL DATA (development
/// half, cross-validated, TUNING_LOG.md E4c): 12,000 µV, chosen in 52 of 53
/// folds; no extra false shock. Clipping is now detected separately
/// ([`MAX_CLIPPED_FRACTION`]), which catches the largest artefacts.
/// Must be re-set for the real front end and pad recordings.
pub const MAX_PLAUSIBLE_PP_UV: f32 = 12_000.0;

/// A raw sample at or beyond this size (either sign) is treated as clipped by
/// the input electronics: the **front end's range**. Depends on the
/// front-end chip, not chosen yet; ±32,000 µV is the limit of the 16-bit
/// sample type. Clipping at any smaller, unknown limit (as in the PhysioNet
/// recorders, which clip near ±5 mV) is caught by the flat-top check
/// ([`MAX_CLIPPED_FRACTION`]) instead.
pub const SATURATION_UV: i16 = 32_000;

/// If more than this fraction of a window is clipped, the answer is `Indeterminate`. Own choice.
pub const MAX_SATURATED_FRACTION: f32 = 0.01;

/// If more than this fraction of a window's raw samples sit in flat tops
/// (runs of at least [`CLIP_RUN_SAMPLES`] identical values at the window's
/// highest or lowest value), the signal was clipped somewhere before the
/// analyser and the answer is `Indeterminate`.
///
/// Checked on the development recordings against the samples known to be at
/// the original recorders' limits (TUNING_LOG.md T-2): it flags 89 of 102
/// windows with more than 1 % of samples at the limit, and 14 of 78,359
/// windows with none. Own choice (same 1 % as [`MAX_SATURATED_FRACTION`]);
/// not tuned for sensitivity, because the recordings hold no labelled
/// artefacts for it to be tuned against. Cost on these recordings: 75 of
/// 2,152 coarse-VF windows, all in cudb, whose recorder clipped large VF.
pub const MAX_CLIPPED_FRACTION: f32 = 0.01;

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

/// Fraction of samples above [`COUNT_LEVEL`] needed to vote VF.
///
/// Was 0.18, TUNED ON SYNTHETIC; on real VF that sat in the middle of the
/// distribution (median 0.22) and was the largest single cause of missed VF
/// (finding F-10). TUNED ON REAL DATA (development half, leave-one-patient-
/// out cross-validation, 2026-11-27; TUNING_LOG.md E1-E4c): 0.16, chosen in
/// every fold. Lower values caught more VF but gave more false shocks on
/// the development data's annotated asystole than the untuned analyser,
/// which the tuning did not allow. Normal rhythm sits far lower (95 % of
/// windows below 0.11).
pub const COUNT_VF_MIN: f32 = 0.16;

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

// ---------------------------------------------------------------------------
// Clipping seen in the signal itself
// ---------------------------------------------------------------------------

/// A run of at least this many identical raw samples sitting at the window's
/// highest or lowest value counts as clipped ("flat top"). 10 samples is
/// 40 ms. Own choice: long enough that the rounded top of a slow wave,
/// recorded at a few microvolts' resolution, is not mistaken for clipping.
pub const CLIP_RUN_SAMPLES: usize = 10;

/// Flat tops are only looked for when the raw window spans at least this
/// many microvolts, so a quiet, nearly flat signal (asystole) is never
/// called clipped. Own choice.
pub const CLIP_MIN_SPAN_UV: i32 = 500;

// ---------------------------------------------------------------------------
// All decision thresholds together, so they can be evaluated as a set
// ---------------------------------------------------------------------------

/// Every threshold the per-window verdict and the confirmation rule use.
///
/// The device uses [`Settings::DEFAULT`]. The struct exists so that the
/// evaluation tools can score alternative values without rebuilding; the
/// analyser itself never changes its settings while running.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// See [`MAX_SATURATED_FRACTION`]: share of samples at the front end's limit.
    pub max_saturated_fraction: f32,
    /// Share of samples in flat-topped (clipped) runs above which the window
    /// is `Indeterminate`. See [`CLIP_RUN_SAMPLES`].
    pub max_clipped_fraction: f32,
    /// See [`MAX_PLAUSIBLE_PP_UV`].
    pub max_plausible_pp_uv: f32,
    /// Flatline: peak-to-peak below this is asystole. See [`ASYSTOLE_MAX_PP_UV`].
    pub asystole_max_uv: f32,
    /// Flatline: a window whose root-mean-square is below this AND whose VF
    /// filter leakage is above [`Settings::asystole_min_leakage`] (it is
    /// noise-like, not a small regular wave) is asystole.
    /// See [`ASYSTOLE_MAX_RMS_UV`].
    pub asystole_max_rms_uv: f32,
    /// See [`Settings::asystole_max_rms_uv`] and [`ASYSTOLE_MIN_LEAKAGE`].
    pub asystole_min_leakage: f32,
    /// See [`MAX_NOISE_RATIO`].
    pub max_noise_ratio: f32,
    /// See [`MAX_LOW_BAND_FRACTION`].
    pub max_low_band_fraction: f32,
    /// See [`FINE_VF_MAX_PP_UV`].
    pub fine_vf_max_pp_uv: f32,
    /// See [`TCSC_VF_MIN`].
    pub tcsc_vf_min: f32,
    /// See [`COUNT_VF_MIN`].
    pub count_vf_min: f32,
    /// See [`LEAKAGE_VF_MAX`].
    pub leakage_vf_max: f32,
    /// See [`DOMINANT_VF_MIN_HZ`].
    pub dominant_vf_min_hz: f32,
    /// See [`DOMINANT_VF_MAX_HZ`].
    pub dominant_vf_max_hz: f32,
    /// See [`CONCENTRATION_VF_MIN`].
    pub concentration_vf_min: f32,
    /// Confirmation: `Shockable` is announced when the current window is
    /// shockable and at least this many of the last [`Settings::confirm_span`]
    /// windows (including the current one) were. See [`CONFIRM_WINDOWS`].
    pub confirm_windows: u8,
    /// Confirmation: how many recent windows [`Settings::confirm_windows`]
    /// counts over (1 to 8). See [`CONFIRM_SPAN`].
    pub confirm_span: u8,
}

impl Settings {
    /// The settings the device uses.
    pub const DEFAULT: Settings = Settings {
        max_saturated_fraction: MAX_SATURATED_FRACTION,
        max_clipped_fraction: MAX_CLIPPED_FRACTION,
        max_plausible_pp_uv: MAX_PLAUSIBLE_PP_UV,
        asystole_max_uv: ASYSTOLE_MAX_PP_UV,
        asystole_max_rms_uv: ASYSTOLE_MAX_RMS_UV,
        asystole_min_leakage: ASYSTOLE_MIN_LEAKAGE,
        max_noise_ratio: MAX_NOISE_RATIO,
        max_low_band_fraction: MAX_LOW_BAND_FRACTION,
        fine_vf_max_pp_uv: FINE_VF_MAX_PP_UV,
        tcsc_vf_min: TCSC_VF_MIN,
        count_vf_min: COUNT_VF_MIN,
        leakage_vf_max: LEAKAGE_VF_MAX,
        dominant_vf_min_hz: DOMINANT_VF_MIN_HZ,
        dominant_vf_max_hz: DOMINANT_VF_MAX_HZ,
        concentration_vf_min: CONCENTRATION_VF_MIN,
        confirm_windows: CONFIRM_WINDOWS,
        confirm_span: CONFIRM_SPAN,
    };
}

impl Default for Settings {
    fn default() -> Self {
        Self::DEFAULT
    }
}
