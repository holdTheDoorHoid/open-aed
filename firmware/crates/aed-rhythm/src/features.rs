// SPDX-License-Identifier: GPL-3.0-or-later
//! The measurements taken on each window, and the rule that turns them into a
//! verdict for that window.
//!
//! Two independent "families" of method each cast a vote:
//!
//! * **Time domain**: how much of the time the signal spends near its peaks
//!   (threshold-crossing sample count) and how much of the time a 14 Hz band
//!   of it is busy (band-pass count). VF is busy all the time; a normal heart
//!   is busy only during each short beat.
//! * **Frequency domain**: how well the signal is cancelled by a copy of
//!   itself shifted by half a period (VF filter leakage), and whether its
//!   power is concentrated around one frequency in the VF/VT range (spectrum).
//!
//! A window is only called shockable when both families vote for it.

use crate::filter::{Biquad, Kind};
use crate::settings::{
    ARTIFACT_BAND_MAX_HZ, ASYSTOLE_MAX_PP_UV, CONCENTRATION_VF_MIN, COUNT_CENTRE_HZ, COUNT_LEVEL,
    COUNT_Q, COUNT_VF_MIN, DOMINANT_SEARCH_MAX_HZ, DOMINANT_VF_MAX_HZ, DOMINANT_VF_MIN_HZ,
    FINE_VF_MAX_PP_UV, LEAKAGE_VF_MAX, MAX_LOW_BAND_FRACTION, MAX_NOISE_RATIO,
    MAX_PLAUSIBLE_PP_UV, MAX_SATURATED_FRACTION, SAMPLES_PER_SECOND, SAMPLE_RATE_HZ,
    SPECTRUM_BINS, SPECTRUM_MIN_HZ, SPECTRUM_STEP_HZ, TCSC_LEVEL, TCSC_VF_MIN, WINDOW_SAMPLES,
};
use aed_types::RhythmDecision;
use core::f32::consts::PI;

/// Everything measured on one window. All values are kept so they can be logged.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Features {
    /// Largest minus smallest value of the cleaned signal, microvolts.
    pub peak_to_peak_uv: f32,
    /// Share of raw samples that were at the input's limit (0 to 1).
    pub saturated_fraction: f32,
    /// High-frequency noise energy divided by analysis-band energy.
    pub noise_ratio: f32,
    /// Threshold-crossing sample count: share of samples above 20 % of each second's peak (0 to 1).
    pub threshold_crossing_fraction: f32,
    /// Band-pass count: share of samples where the 14 Hz band is above half its peak (0 to 1).
    pub bandpass_count_fraction: f32,
    /// VF filter leakage (0 = perfectly regular wave, about 1 = spiky beats).
    pub vf_leakage: f32,
    /// Strongest frequency between 1 and 10 Hz.
    pub dominant_frequency_hz: f32,
    /// Share of power near the strongest frequency (0 to 1).
    pub spectral_concentration: f32,
    /// Share of power below the compression-artifact limit (0 to 1).
    pub low_band_fraction: f32,
}

/// Why a window got its verdict, in words a log reader can follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// A measurement came out as not-a-number or infinite.
    InvalidMeasurement,
    /// Too much of the signal was clipped by the input electronics.
    Saturated,
    /// The signal is far larger than a heart signal can be.
    AmplitudeImplausible,
    /// The signal is almost flat: asystole.
    Asystole,
    /// Too much high-frequency noise to judge.
    Noisy,
    /// Looks like chest compressions or movement.
    PossibleArtifact,
    /// Both families say VF, but it is too small (fine VF); not shocked.
    FineVf,
    /// One family says shockable and the other does not.
    MethodsDisagree,
    /// Both families say shockable.
    BothMethodsShockable,
    /// Neither family says shockable.
    NeitherMethodShockable,
}

/// The verdict for one window, before the "several windows in a row" rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowVerdict {
    /// The answer for this window on its own.
    pub decision: RhythmDecision,
    /// Why.
    pub reason: Reason,
    /// Whether the time-domain family voted shockable.
    pub time_domain_vote: bool,
    /// Whether the frequency-domain family voted shockable.
    pub frequency_domain_vote: bool,
}

/// Measure one window.
///
/// `window` holds the cleaned signal, oldest sample first. It is **modified**
/// (a taper is applied for the spectrum), so pass a scratch copy.
/// `saturated_samples` and `noise_energy` come from the analyser's running counts.
#[must_use]
pub fn measure(
    window: &mut [f32; WINDOW_SAMPLES],
    saturated_samples: u32,
    noise_energy: f32,
) -> Features {
    let x = window.as_slice();
    let (min, max) = x
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let band_energy: f32 = x.iter().map(|v| v * v).sum();
    let noise_ratio = if band_energy > 1e-6 {
        noise_energy / band_energy
    } else {
        0.0
    };
    let threshold_crossing_fraction = threshold_crossing_fraction(x);
    let bandpass_count_fraction = bandpass_count_fraction(x);
    let vf_leakage = vf_leakage(x);
    let spectral = spectral_features(window);
    Features {
        peak_to_peak_uv: max - min,
        saturated_fraction: saturated_samples as f32 / WINDOW_SAMPLES as f32,
        noise_ratio,
        threshold_crossing_fraction,
        bandpass_count_fraction,
        vf_leakage,
        dominant_frequency_hz: spectral.dominant_hz,
        spectral_concentration: spectral.concentration,
        low_band_fraction: spectral.low_band_fraction,
    }
}

/// Threshold-crossing sample count (after Arafat et al.): in each one-second
/// piece, the share of samples whose size exceeds [`TCSC_LEVEL`] times that
/// piece's largest size. Averaged over the window.
#[must_use]
pub fn threshold_crossing_fraction(x: &[f32]) -> f32 {
    average_over_seconds(x, |chunk| {
        let peak = chunk.iter().fold(0.0_f32, |m, v| m.max(libm::fabsf(*v)));
        if peak <= 0.0 {
            return 0.0;
        }
        let level = TCSC_LEVEL * peak;
        let above = chunk.iter().filter(|v| libm::fabsf(**v) > level).count();
        above as f32 / chunk.len() as f32
    })
}

/// Band-pass count (in the style of Jekova's "Count1"): pass the window
/// through a band-pass around [`COUNT_CENTRE_HZ`], then in each second count
/// the share of samples above [`COUNT_LEVEL`] times that second's peak.
#[must_use]
pub fn bandpass_count_fraction(x: &[f32]) -> f32 {
    // Pass 1: peak of the band-passed signal in each second.
    let mut peaks = [0.0_f32; WINDOW_SAMPLES / SAMPLES_PER_SECOND];
    let mut bp = Biquad::new(Kind::BandPass, COUNT_CENTRE_HZ, COUNT_Q, SAMPLE_RATE_HZ);
    for (chunk, peak) in x.chunks(SAMPLES_PER_SECOND).zip(peaks.iter_mut()) {
        for v in chunk {
            *peak = peak.max(libm::fabsf(bp.process(*v)));
        }
    }
    // Pass 2: run the same filter again from scratch and count.
    let mut bp = Biquad::new(Kind::BandPass, COUNT_CENTRE_HZ, COUNT_Q, SAMPLE_RATE_HZ);
    let mut total = 0.0_f32;
    let mut pieces = 0.0_f32;
    for (chunk, peak) in x.chunks(SAMPLES_PER_SECOND).zip(peaks.iter()) {
        let level = COUNT_LEVEL * *peak;
        let above = chunk
            .iter()
            .filter(|v| libm::fabsf(bp.process(**v)) > level)
            .count();
        if *peak > 0.0 {
            total += above as f32 / chunk.len() as f32;
        }
        pieces += 1.0;
    }
    if pieces > 0.0 {
        total / pieces
    } else {
        0.0
    }
}

/// VF filter leakage (Kuo & Dillman 1978, as described by Amann et al. 2005).
///
/// Estimate the signal's half-period `N` from its average size and average
/// slope, add the signal to a copy of itself shifted by `N` samples, and see
/// how much is left. A regular wave like VF nearly cancels (small leakage);
/// spiky normal beats do not (leakage near 1). Returns 1 if it cannot be
/// computed, which counts as "not VF".
#[must_use]
pub fn vf_leakage(x: &[f32]) -> f32 {
    let sum_abs: f32 = x.iter().map(|v| libm::fabsf(*v)).sum();
    let sum_diff: f32 = x
        .iter()
        .zip(x.iter().skip(1))
        .map(|(a, b)| libm::fabsf(b - a))
        .sum();
    if sum_diff <= 0.0 {
        return 1.0;
    }
    let half_period = libm::floorf(PI * sum_abs / sum_diff + 0.5);
    if !(half_period >= 1.0 && half_period < x.len() as f32) {
        return 1.0;
    }
    let n = half_period as usize;
    let (num, den) = x
        .iter()
        .skip(n)
        .zip(x.iter())
        .fold((0.0_f32, 0.0_f32), |(num, den), (now, past)| {
            (
                num + libm::fabsf(now + past),
                den + libm::fabsf(*now) + libm::fabsf(*past),
            )
        });
    if den > 0.0 {
        num / den
    } else {
        1.0
    }
}

/// Results of the spectrum step.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Spectral {
    dominant_hz: f32,
    concentration: f32,
    low_band_fraction: f32,
}

/// Power spectrum from 1 to 30 Hz using the Goertzel method (one frequency at
/// a time, no large tables), after a Hann taper. Modifies `window` in place.
fn spectral_features(window: &mut [f32; WINDOW_SAMPLES]) -> Spectral {
    let len = window.len() as f32;
    for (i, v) in window.iter_mut().enumerate() {
        *v *= 0.5 - 0.5 * libm::cosf(2.0 * PI * i as f32 / len);
    }
    let mut power = [0.0_f32; SPECTRUM_BINS];
    for (k, p) in power.iter_mut().enumerate() {
        let hz = bin_hz(k);
        let coeff = 2.0 * libm::cosf(2.0 * PI * hz / SAMPLE_RATE_HZ);
        let (mut s1, mut s2) = (0.0_f32, 0.0_f32);
        for v in window.iter() {
            let s0 = v + coeff * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        *p = (s1 * s1 + s2 * s2 - coeff * s1 * s2).max(0.0);
    }
    let total: f32 = power.iter().sum();
    let (peak_bin, _) = power
        .iter()
        .enumerate()
        .filter(|(k, _)| bin_hz(*k) <= DOMINANT_SEARCH_MAX_HZ)
        .fold((0_usize, -1.0_f32), |(bk, bp), (k, p)| {
            if *p > bp {
                (k, *p)
            } else {
                (bk, bp)
            }
        });
    let dominant_hz = bin_hz(peak_bin);
    let band_power = |lo: f32, hi: f32| -> f32 {
        power
            .iter()
            .enumerate()
            .filter(|(k, _)| {
                let f = bin_hz(*k);
                f >= lo && f <= hi
            })
            .map(|(_, p)| *p)
            .sum()
    };
    if total <= 0.0 {
        return Spectral {
            dominant_hz,
            concentration: 0.0,
            low_band_fraction: 0.0,
        };
    }
    Spectral {
        dominant_hz,
        concentration: band_power(0.6 * dominant_hz, 1.4 * dominant_hz) / total,
        low_band_fraction: band_power(0.0, ARTIFACT_BAND_MAX_HZ) / total,
    }
}

fn bin_hz(k: usize) -> f32 {
    SPECTRUM_MIN_HZ + k as f32 * SPECTRUM_STEP_HZ
}

fn average_over_seconds(x: &[f32], f: impl Fn(&[f32]) -> f32) -> f32 {
    let (sum, n) = x
        .chunks(SAMPLES_PER_SECOND)
        .fold((0.0_f32, 0.0_f32), |(s, n), c| (s + f(c), n + 1.0));
    if n > 0.0 {
        sum / n
    } else {
        0.0
    }
}

/// Turn one window's measurements into a verdict for that window.
///
/// Order matters and every doubtful case leans toward not shocking:
/// bad signal first (`Indeterminate`), then flatline (`NonShockable`), then
/// the two method families. Comparisons are written so that a value exactly
/// on a threshold does not count as shockable.
#[must_use]
pub fn classify(f: &Features) -> WindowVerdict {
    let verdict = |decision, reason, t, q| WindowVerdict {
        decision,
        reason,
        time_domain_vote: t,
        frequency_domain_vote: q,
    };
    let all = [
        f.peak_to_peak_uv,
        f.saturated_fraction,
        f.noise_ratio,
        f.threshold_crossing_fraction,
        f.bandpass_count_fraction,
        f.vf_leakage,
        f.dominant_frequency_hz,
        f.spectral_concentration,
        f.low_band_fraction,
    ];
    if !all.iter().all(|v| v.is_finite()) {
        return verdict(RhythmDecision::Indeterminate, Reason::InvalidMeasurement, false, false);
    }
    if f.saturated_fraction > MAX_SATURATED_FRACTION {
        return verdict(RhythmDecision::Indeterminate, Reason::Saturated, false, false);
    }
    if f.peak_to_peak_uv > MAX_PLAUSIBLE_PP_UV {
        return verdict(RhythmDecision::Indeterminate, Reason::AmplitudeImplausible, false, false);
    }
    if f.peak_to_peak_uv < ASYSTOLE_MAX_PP_UV {
        return verdict(RhythmDecision::NonShockable, Reason::Asystole, false, false);
    }
    if f.noise_ratio > MAX_NOISE_RATIO {
        return verdict(RhythmDecision::Indeterminate, Reason::Noisy, false, false);
    }

    let time_vote = f.threshold_crossing_fraction > TCSC_VF_MIN
        && f.bandpass_count_fraction > COUNT_VF_MIN;
    let freq_vote = f.vf_leakage < LEAKAGE_VF_MAX
        && f.dominant_frequency_hz > DOMINANT_VF_MIN_HZ
        && f.dominant_frequency_hz < DOMINANT_VF_MAX_HZ
        && f.spectral_concentration > CONCENTRATION_VF_MIN;

    if f.low_band_fraction > MAX_LOW_BAND_FRACTION {
        return verdict(RhythmDecision::Indeterminate, Reason::PossibleArtifact, time_vote, freq_vote);
    }
    match (time_vote, freq_vote) {
        (true, true) if f.peak_to_peak_uv < FINE_VF_MAX_PP_UV => {
            verdict(RhythmDecision::Indeterminate, Reason::FineVf, true, true)
        }
        (true, true) => verdict(RhythmDecision::Shockable, Reason::BothMethodsShockable, true, true),
        (false, false) => verdict(
            RhythmDecision::NonShockable,
            Reason::NeitherMethodShockable,
            false,
            false,
        ),
        (t, q) => verdict(RhythmDecision::Indeterminate, Reason::MethodsDisagree, t, q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(hz: f32, amp: f32) -> [f32; WINDOW_SAMPLES] {
        let mut w = [0.0; WINDOW_SAMPLES];
        for (i, v) in w.iter_mut().enumerate() {
            *v = amp * libm::sinf(2.0 * PI * hz * i as f32 / SAMPLE_RATE_HZ);
        }
        w
    }

    #[test]
    fn leakage_is_small_for_a_sine() {
        let w = sine(5.0, 500.0);
        assert!(vf_leakage(&w) < 0.2);
    }

    #[test]
    fn dominant_frequency_found() {
        let mut w = sine(5.0, 500.0);
        let s = spectral_features(&mut w);
        assert!(libm::fabsf(s.dominant_hz - 5.0) < 0.3);
        assert!(s.concentration > 0.9);
    }

    #[test]
    fn flat_window_is_asystole_not_a_crash() {
        let mut w = [0.0; WINDOW_SAMPLES];
        let f = measure(&mut w, 0, 0.0);
        let v = classify(&f);
        assert_eq!(v.decision, RhythmDecision::NonShockable);
        assert_eq!(v.reason, Reason::Asystole);
    }

    #[test]
    fn nan_measurement_is_indeterminate() {
        let f = Features {
            vf_leakage: f32::NAN,
            peak_to_peak_uv: 1000.0,
            ..Features::default()
        };
        assert_eq!(classify(&f).decision, RhythmDecision::Indeterminate);
    }

    #[test]
    fn exactly_on_threshold_is_not_shockable() {
        let f = Features {
            peak_to_peak_uv: 1000.0,
            threshold_crossing_fraction: TCSC_VF_MIN,
            bandpass_count_fraction: 0.9,
            vf_leakage: 0.1,
            dominant_frequency_hz: 5.0,
            spectral_concentration: 0.9,
            ..Features::default()
        };
        let v = classify(&f);
        assert!(!v.time_domain_vote);
        assert_eq!(v.decision, RhythmDecision::Indeterminate);
    }

    #[test]
    fn one_family_alone_is_indeterminate() {
        let f = Features {
            peak_to_peak_uv: 1000.0,
            threshold_crossing_fraction: 0.9,
            bandpass_count_fraction: 0.9,
            vf_leakage: 0.9,
            dominant_frequency_hz: 5.0,
            spectral_concentration: 0.9,
            ..Features::default()
        };
        let v = classify(&f);
        assert_eq!(v.reason, Reason::MethodsDisagree);
        assert_eq!(v.decision, RhythmDecision::Indeterminate);
    }
}
