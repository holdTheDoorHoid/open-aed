// SPDX-License-Identifier: GPL-3.0-or-later
//! The detector's settings, and the assumptions it makes about the
//! electronics that measure pad impedance.
//!
//! Every value is provisional. Each says where it came from: a published
//! figure (with the source), the development half of the one open recording
//! set (Paris Fire Brigade, see `crates/aed-cpr-eval/REPORT.md`), or a
//! design choice.

/// What the detector assumes about the impedance front end. These are
/// requirements on the future hardware, written down so they can be checked
/// when a part is chosen. The detector does not read them at run time except
/// where a [`Settings`] field says so.
///
/// * **Excitation:** a small sine-wave test current between the pads, about
///   20 to 32 kHz and tens of microamps. Published AEDs use 32 kHz, 3 mA
///   peak to peak (Philips HeartStart MRx, Alonso et al., Computing in
///   Cardiology 2013;40:803) and 20 kHz, 34 µA (Berve et al., J Clin Med
///   2019;8:724). The current must stay inside the patient auxiliary
///   current limit of IEC 60601-1 at that frequency (standard not yet read).
/// * **Resolution:** 2 mΩ per step or finer. The detector's smallest
///   compression is 300 mΩ peak to peak, so 2 mΩ steps lose nothing. For
///   comparison: Philips MRx 0.74 mΩ per step (Alonso 2013), Schiller Fred
///   Easy 29 mΩ (Neyton et al., CinC 2015), and the Paris recordings
///   12.2 mΩ (measured from the data).
/// * **Range:** at least 25 to 250 Ω of baseline (the pad-sensing range of
///   decision D-32), plus about ±25 Ω of change around it, without
///   saturating. Outside it the front end must report
///   [`aed_types::ImpedanceSample::Saturated`] rather than a wrong number.
/// * **Noise:** below about 10 mΩ RMS between 0.5 and 10 Hz with the pads on
///   a still patient. A noise-free channel is NOT acceptable either: the
///   detector treats a value that does not change at all for
///   [`Settings::stuck_ms`] as a dead channel, so the front end must show at
///   least a step or two of noise. (In the Paris recordings no good stretch
///   held one value for more than 1 s.)
/// * **Bandwidth:** flat from 0 to at least 10 Hz; the detector filters
///   itself.
/// * **After a shock:** saturated (or reported as such) for as short a time
///   as possible, then back to within the range. The detector waits
///   [`Settings::settle_ms`] after the last saturated sample.
/// * **Charger noise:** the capacitor charger runs during every analysis
///   (decision D-29). Its switching noise must not reach this channel above
///   the noise level. Not modelled anywhere yet; must be measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontEndAssumptions;

/// The detector's settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// Lowest baseline pad impedance accepted, milliohms. Below this the
    /// pads are shorted or touching: no reading. 25 Ω, the bottom of the
    /// pad-sensing range (aed-types `MIN_PATIENT_OHMS`).
    pub baseline_min_mohm: u32,
    /// Highest baseline pad impedance accepted, milliohms. 250 Ω, the top of
    /// the pad-sensing range (decision D-32).
    pub baseline_max_mohm: u32,
    /// Corner of the high-pass filter (fourth order), Hz. It removes the
    /// baseline (the patient's own resistance) and slow drift. Breaths are
    /// NOT removed by this filter but by [`Settings::max_rise_ms`]: a breath
    /// takes about a second to go in (ventilation detectors low-pass at
    /// 0.6 Hz to separate breaths from compressions: Gonzalez-Otero et al.,
    /// CinC 2013;40:807), a compression a quarter to half of that.
    /// 0.3 Hz: chosen on the development half of the Paris recordings
    /// (0.3, 0.5 and 1.0 Hz were compared). 1.0 Hz missed slightly fewer
    /// compressions there (0.19 % against 0.28 % of compression time) but
    /// called breaths and other slow movement "compressing" twice as often
    /// and added about 0.6 s to the median time to "hands off"; the higher
    /// corner also distorts slow compressions (at 60 per minute a 0.35 Ω
    /// compression vanished in the synthetic tests).
    pub highpass_hz: f32,
    /// Fourth-order high-pass (`true`) or second-order (`false`). A
    /// fourth-order filter removes slow changes more sharply but rings, and
    /// its ringing turns a breath into fast wiggles that look like
    /// movement; see [`Settings::highpass_hz`].
    pub highpass_fourth_order: bool,
    /// Corner of the low-pass filter (second order), Hz. Removes fast noise;
    /// compressions have little above their third harmonic. Design choice: 6 Hz.
    pub lowpass_hz: f32,
    /// The smallest change, peak to peak after filtering, that counts as a
    /// compression or as movement, milliohms.
    ///
    /// This is the single most important number. Too high and real
    /// compressions are missed (the dangerous direction: the device would
    /// think hands are off). Too low and the heart's own small impedance
    /// change could look like compressions. Evidence:
    ///
    /// * Compressions: median 1.1 to 1.7 Ω peak to peak, lower quartile
    ///   0.78 Ω for shallow compressions, in 41,078 compressions from 19
    ///   out-of-hospital arrests (Alonso et al., CinC 2013;40:803).
    /// * The heart's own impedance change "rarely exceeding 100 mΩ"
    ///   (Urteaga et al., Entropy 2021;23:847); the example heartbeat in
    ///   Risdal et al. (CinC 2005;32:809) is about ±50 mΩ.
    ///
    /// 300 mΩ is three times the largest usual cardiac change and well
    /// under the lower quartile of compressions. Checked on the development
    /// half of the Paris recordings (see the evaluation report).
    pub min_compression_mohm: u32,
    /// Longest time a compression's rise may take, ms. A slower rise is a
    /// breath or a slow movement, not a compression (and not movement
    /// either). 600 ms: a compression at 50 per minute rises in about that
    /// time; 500 and 700 ms were compared on the development half (500 ms
    /// missed more compressions, 700 ms called more breaths "unknown").
    pub max_rise_ms: u32,
    /// Longest time the steep part of a compression's rise (from 10 % to
    /// 90 % of its size) may take, ms. This is what tells a compression from
    /// a breath most reliably: a rise shaped like half a sine wave spends
    /// about 60 % of its time between 10 % and 90 %, so a compression rising
    /// in 0.25 to 0.5 s takes 0.15 to 0.3 s, a breath going in over 0.7 to
    /// 1 s takes 0.4 to 0.6 s. Design choice: 350 ms.
    pub max_steep_ms: u32,
    /// Shortest time a compression's rise may take, ms. Faster is a spike.
    /// Design choice: 60 ms.
    pub min_rise_ms: u32,
    /// Shortest gap between two compressions that still counts as a run,
    /// ms. 250 ms is 240 per minute.
    pub min_interval_ms: u32,
    /// Longest gap between two compressions that still counts as one run,
    /// ms. 1,500 ms is 40 per minute.
    pub max_interval_ms: u32,
    /// How many compressions in a row, each within the interval limits,
    /// make the state `Compressing`. Before that, a single compression-sized
    /// movement makes the state `Unknown` at once (never `HandsOff`).
    pub compressions_to_confirm: u8,
    /// "Hands off" needs this long with no compression-sized movement at
    /// all, ms. Design choice: 1,000 ms, about two compressions at 110 per
    /// minute. This is added to every hands-off pause before an analysis
    /// can start, so it is kept short.
    pub quiet_min_ms: u32,
    /// While compressions are being given, "hands off" needs at least this
    /// many recent compression intervals of quiet, in tenths (16 = 1.6
    /// intervals), if that is longer than [`Settings::quiet_min_ms`]. So a
    /// slow rhythm of compressions does not flicker to "hands off" between
    /// compressions.
    pub quiet_intervals_tenths: u16,
    /// After the signal comes back (pads on, end of saturation, end of a
    /// stuck reading), wait this long for the filters to settle before
    /// saying anything but `Unknown`, ms. Design choice: 1,000 ms.
    pub settle_ms: u32,
    /// A reading that does not change at all for this long is a dead
    /// channel, ms. Design choice: 500 ms (see [`FrontEndAssumptions`]). It
    /// must be shorter than [`Settings::quiet_min_ms`]: a channel that
    /// freezes in the middle of compressions must be reported as stuck
    /// before the quiet period could make the answer `HandsOff`.
    pub stuck_ms: u32,
    /// Fast noise limit: if the RMS of the sample-to-sample curvature
    /// (second difference) over the last half second is above this, the
    /// signal is `Noisy`, milliohms. Design choice: 150 mΩ, far above
    /// what compressions produce (an 8 Ω compression at 2 Hz gives about
    /// 30 mΩ) and far above a 12 mΩ quantisation step.
    pub noise_max_mohm: u32,
    /// Length of the rolling period for the time-on-chest measure, seconds
    /// (at most [`crate::MAX_ROLLING_SECONDS`]). 60 s: the "chest compression
    /// fraction" in resuscitation research is usually reported per minute.
    pub rolling_seconds: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            baseline_min_mohm: 25_000,
            baseline_max_mohm: 250_000,
            highpass_hz: 0.5,
            highpass_fourth_order: false,
            lowpass_hz: 6.0,
            min_compression_mohm: 300,
            max_rise_ms: 600,
            max_steep_ms: 350,
            min_rise_ms: 60,
            min_interval_ms: 250,
            max_interval_ms: 1_500,
            compressions_to_confirm: 2,
            quiet_min_ms: 1_000,
            quiet_intervals_tenths: 16,
            settle_ms: 1_000,
            stuck_ms: 500,
            noise_max_mohm: 150,
            rolling_seconds: 60,
        }
    }
}

/// Why a [`Settings`] was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsError {
    /// The baseline range is empty or starts at zero.
    BaselineRange,
    /// A filter corner is not between 0 and a fifth of the sample rate, or
    /// the high-pass corner is not below the low-pass corner.
    FilterCorners,
    /// The compression threshold is zero.
    ZeroThreshold,
    /// Rise or interval limits are zero or in the wrong order, or the rise
    /// window is longer than the detector can look back.
    TimeLimits,
    /// Confirmation needs at least one compression.
    ConfirmCount,
    /// The rolling period is zero or longer than the detector can store.
    RollingPeriod,
}

impl Settings {
    /// Check the numbers make sense together.
    ///
    /// # Errors
    /// Returns the first problem found.
    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.baseline_min_mohm == 0 || self.baseline_min_mohm >= self.baseline_max_mohm {
            return Err(SettingsError::BaselineRange);
        }
        let nyquist_fifth = aed_types::IMPEDANCE_SAMPLE_RATE_HZ as f32 / 5.0;
        let corners_ok = self.highpass_hz > 0.0
            && self.lowpass_hz > self.highpass_hz
            && self.lowpass_hz < nyquist_fifth;
        if !corners_ok {
            return Err(SettingsError::FilterCorners);
        }
        if self.min_compression_mohm == 0 {
            return Err(SettingsError::ZeroThreshold);
        }
        let lookback_ms = (crate::detector::HISTORY_LEN as u32 * crate::SAMPLE_MS)
            .saturating_sub(crate::detector::TURN_MARGIN_MS);
        let times_ok = self.min_rise_ms > 0
            && self.min_rise_ms < self.max_rise_ms
            && self.max_rise_ms < lookback_ms
            && self.max_steep_ms > 0
            && self.max_steep_ms <= self.max_rise_ms
            && self.min_interval_ms > 0
            && self.min_interval_ms < self.max_interval_ms
            && self.quiet_min_ms > 0
            && self.settle_ms > 0
            && self.stuck_ms > 0
            && self.stuck_ms < self.quiet_min_ms;
        if !times_ok {
            return Err(SettingsError::TimeLimits);
        }
        if self.compressions_to_confirm == 0 {
            return Err(SettingsError::ConfirmCount);
        }
        if self.rolling_seconds == 0 || usize::from(self.rolling_seconds) > crate::MAX_ROLLING_SECONDS {
            return Err(SettingsError::RollingPeriod);
        }
        Ok(())
    }
}
