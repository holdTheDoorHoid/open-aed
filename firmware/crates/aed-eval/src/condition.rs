// SPDX-License-Identifier: GPL-3.0-or-later
//! Turning recorded samples into what the analyser expects: whole
//! microvolts as `i16`, at 250 samples per second.
//!
//! * **Units.** Each recorded value is converted with its own signal's gain
//!   (ADC units per millivolt) and baseline: `µV = (value − baseline) × 1000 / gain`.
//!   Values beyond the `i16` range are clipped to it and **counted**.
//!   (On these databases the recorder's own range is about ±5 to ±10 mV, so
//!   the `i16` limit of ±32.7 mV is never reached; what does happen is that
//!   the original recorder clipped at its own limits. Those samples are
//!   counted separately as "ADC rail" samples.)
//! * **Rate.** The MIT-BIH Arrhythmia Database is sampled at 360 Hz. It is
//!   converted to 250 Hz with a windowed-sinc resampler: each output sample
//!   is a weighted sum of 96 input samples around its exact position, with
//!   weights from an ideal low-pass at 112 Hz shaped by a Kaiser window.
//!   That keeps everything up to about 100 Hz unchanged and removes what
//!   would otherwise fold back below 125 Hz (aliasing). Tested with sine
//!   sweeps below.

use core::f64::consts::PI;

/// Result of converting one signal to the analyser's input form.
#[derive(Debug, Clone, PartialEq)]
pub struct Quantized {
    /// Whole microvolts, clipped to the `i16` range.
    pub samples: Vec<i16>,
    /// How many samples had to be clipped to fit `i16`.
    pub clipped: u64,
}

/// Convert raw ADC values to microvolts (floating point, not yet rounded).
///
/// A gain that is zero, negative or not finite gives an empty result: the
/// caller must treat that signal as unusable rather than guess a scale.
#[must_use]
pub fn to_microvolts(raw: &[i16], gain_adu_per_mv: f64, baseline: i64) -> Vec<f64> {
    if !(gain_adu_per_mv.is_finite() && gain_adu_per_mv > 0.0) {
        return Vec::new();
    }
    let scale = 1000.0 / gain_adu_per_mv;
    raw.iter()
        .map(|v| (f64::from(*v) - baseline as f64) * scale)
        .collect()
}

/// Round to whole microvolts and clip to the `i16` range, counting clips.
/// Not-a-number values become 0 and are counted as clipped.
#[must_use]
pub fn quantize(uv: &[f64]) -> Quantized {
    let mut clipped = 0_u64;
    let samples = uv
        .iter()
        .map(|v| {
            if !v.is_finite() {
                clipped += 1;
                return 0;
            }
            let r = v.round();
            if r > f64::from(i16::MAX) {
                clipped += 1;
                i16::MAX
            } else if r < f64::from(i16::MIN) {
                clipped += 1;
                i16::MIN
            } else {
                r as i16
            }
        })
        .collect();
    Quantized { samples, clipped }
}

/// Zero-order modified Bessel function of the first kind (series), for the Kaiser window.
fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let half = x / 2.0;
    for k in 1..60 {
        let kf = f64::from(k);
        term *= (half / kf) * (half / kf);
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// A rational-ratio windowed-sinc resampler (polyphase).
#[derive(Debug, Clone)]
pub struct Resampler {
    /// Output samples per `down` input samples.
    up: u32,
    /// Input samples per `up` output samples.
    down: u32,
    /// Taps on each side of the output position.
    half_taps: usize,
    /// One set of `2 * half_taps` weights per phase (`up` phases), flattened.
    weights: Vec<f64>,
}

/// Errors building a resampler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResampleError {
    /// A rate was zero.
    ZeroRate,
    /// The cut-off frequency is not below both Nyquist frequencies.
    BadCutoff,
}

impl Resampler {
    /// Taps on each side used by [`Resampler::new_default`].
    pub const DEFAULT_HALF_TAPS: usize = 48;
    /// Kaiser window shape used by [`Resampler::new_default`] (about 80 dB stop band).
    pub const DEFAULT_BETA: f64 = 7.857;

    /// Resampler from `from_hz` to `to_hz` (whole numbers), with a low-pass
    /// cut-off at `cutoff_hz` (−6 dB point), `half_taps` taps on each side
    /// and Kaiser shape `beta`.
    ///
    /// # Errors
    /// [`ResampleError`] for a zero rate or a cut-off at or above either Nyquist frequency.
    pub fn new(
        from_hz: u32,
        to_hz: u32,
        cutoff_hz: f64,
        half_taps: usize,
        beta: f64,
    ) -> Result<Self, ResampleError> {
        if from_hz == 0 || to_hz == 0 {
            return Err(ResampleError::ZeroRate);
        }
        let nyq = f64::from(from_hz.min(to_hz)) / 2.0;
        if !(cutoff_hz > 0.0 && cutoff_hz < nyq) {
            return Err(ResampleError::BadCutoff);
        }
        let g = gcd(from_hz, to_hz);
        let up = to_hz / g;
        let down = from_hz / g;
        let fc = cutoff_hz / f64::from(from_hz); // cycles per input sample
        let i0_beta = bessel_i0(beta);
        let span = half_taps as f64;
        let mut weights = Vec::with_capacity(up as usize * 2 * half_taps);
        for phase in 0..up {
            // Output position lies `frac` input samples after a whole input sample.
            let frac = f64::from(phase) / f64::from(up);
            let start = weights.len();
            for k in 0..2 * half_taps {
                // Input index offset relative to floor(position): from -(half_taps-1) to +half_taps.
                let offset = k as f64 - (span - 1.0);
                let t = offset - frac;
                let x = 2.0 * fc * t;
                let sinc = if x.abs() < 1e-12 { 1.0 } else { (PI * x).sin() / (PI * x) };
                let r = t / span;
                let w = if r.abs() >= 1.0 {
                    0.0
                } else {
                    bessel_i0(beta * (1.0 - r * r).sqrt()) / i0_beta
                };
                weights.push(2.0 * fc * sinc * w);
            }
            // Normalise each phase to unit gain at zero frequency.
            let sum: f64 = weights.iter().skip(start).sum();
            if sum.abs() > 1e-12 {
                for v in weights.iter_mut().skip(start) {
                    *v /= sum;
                }
            }
        }
        Ok(Self {
            up,
            down,
            half_taps,
            weights,
        })
    }

    /// The resampler used for the MIT-BIH Arrhythmia Database: 360 Hz to
    /// 250 Hz, cut-off 112 Hz, 96 taps, Kaiser beta 7.857.
    ///
    /// # Errors
    /// Never in practice (the constants are valid); kept as a `Result` so no panic is possible.
    pub fn new_default(from_hz: u32, to_hz: u32) -> Result<Self, ResampleError> {
        let nyq = f64::from(from_hz.min(to_hz)) / 2.0;
        Self::new(
            from_hz,
            to_hz,
            nyq * 0.896,
            Self::DEFAULT_HALF_TAPS,
            Self::DEFAULT_BETA,
        )
    }

    /// Resample a whole signal. Output sample `m` sits at input time
    /// `m × down / up`, so the first samples line up exactly. Beyond the ends
    /// the first and last input values are repeated.
    #[must_use]
    pub fn process(&self, input: &[f64]) -> Vec<f64> {
        let (Some(&first), Some(&last)) = (input.first(), input.last()) else {
            return Vec::new();
        };
        let n_in = input.len() as u64;
        let up = u64::from(self.up);
        let down = u64::from(self.down);
        // Number of output samples whose position is inside the input.
        let n_out = ((n_in - 1) * up) / down + 1;
        let taps = 2 * self.half_taps;
        let mut out = Vec::with_capacity(n_out as usize);
        for m in 0..n_out {
            let num = m * down;
            let base = (num / up) as i64;
            let phase = (num % up) as usize;
            let Some(w) = self.weights.get(phase * taps..(phase + 1) * taps) else {
                out.push(0.0);
                continue;
            };
            let first_idx = base - (self.half_taps as i64 - 1);
            let mut acc = 0.0;
            for (k, wk) in w.iter().enumerate() {
                let idx = first_idx + k as i64;
                let v = if idx < 0 {
                    first
                } else {
                    input.get(idx as usize).copied().unwrap_or(last)
                };
                acc += wk * v;
            }
            out.push(acc);
        }
        out
    }

    /// Map an output sample index to the nearest input sample index.
    #[must_use]
    pub fn nearest_input(&self, out_index: usize) -> usize {
        let num = out_index as u64 * u64::from(self.down);
        let up = u64::from(self.up);
        ((num + up / 2) / up) as usize
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn sine(hz: f64, fs: f64, n: usize, phase: f64) -> Vec<f64> {
        (0..n).map(|i| (2.0 * PI * hz * i as f64 / fs + phase).sin()).collect()
    }

    /// Output error versus the exact sine at the output times, ignoring the edges.
    fn check_pass(hz: f64) -> (f64, f64) {
        let r = Resampler::new_default(360, 250).unwrap();
        let x = sine(hz, 360.0, 36_000, 0.3);
        let y = r.process(&x);
        let mut worst_err = 0.0_f64;
        let mut peak = 0.0_f64;
        for (m, v) in y.iter().enumerate().skip(500).take(y.len() - 1000) {
            let exact = (2.0 * PI * hz * m as f64 / 250.0 + 0.3).sin();
            worst_err = worst_err.max((v - exact).abs());
            peak = peak.max(v.abs());
        }
        (worst_err, peak)
    }

    #[test]
    fn passband_sweep_is_flat_and_in_phase() {
        for hz in [0.5, 1.0, 3.0, 5.0, 10.0, 20.0, 30.0, 45.0, 60.0, 75.0, 90.0] {
            let (err, peak) = check_pass(hz);
            assert!(err < 0.005, "{hz} Hz: error {err}");
            assert!((peak - 1.0).abs() < 0.005, "{hz} Hz: gain {peak}");
        }
    }

    #[test]
    fn stopband_sweep_does_not_alias() {
        let r = Resampler::new_default(360, 250).unwrap();
        for hz in [126.0, 130.0, 140.0, 150.0, 160.0, 170.0, 179.0] {
            let x = sine(hz, 360.0, 36_000, 0.0);
            let y = r.process(&x);
            let peak = y
                .iter()
                .skip(500)
                .take(y.len() - 1000)
                .fold(0.0_f64, |m, v| m.max(v.abs()));
            // At least 60 dB down (would alias to 250 - hz).
            assert!(peak < 1e-3, "{hz} Hz leaks {peak}");
        }
    }

    #[test]
    fn dc_length_and_alignment() {
        let r = Resampler::new_default(360, 250).unwrap();
        let y = r.process(&vec![5.0; 3600]);
        assert_eq!(y.len(), (3599 * 25) / 36 + 1);
        assert!(y.iter().all(|v| (v - 5.0).abs() < 1e-9));
        assert_eq!(r.nearest_input(25), 36);
        assert!(r.process(&[]).is_empty());
        let one = r.process(&[2.0]);
        assert_eq!(one.len(), 1);
        assert!((one[0] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn bad_settings_are_errors() {
        assert!(Resampler::new(0, 250, 100.0, 8, 5.0).is_err());
        assert!(Resampler::new(360, 250, 125.0, 8, 5.0).is_err());
        assert!(Resampler::new(360, 250, f64::NAN, 8, 5.0).is_err());
    }

    #[test]
    fn units_and_clipping() {
        let uv = to_microvolts(&[1024, 1224, 824], 200.0, 1024);
        assert_eq!(uv, vec![0.0, 1000.0, -1000.0]);
        assert!(to_microvolts(&[1], 0.0, 0).is_empty());
        assert!(to_microvolts(&[1], f64::NAN, 0).is_empty());
        let q = quantize(&[1.4, -1.6, 40_000.0, -40_000.0, f64::NAN]);
        assert_eq!(q.samples, vec![1, -2, i16::MAX, i16::MIN, 0]);
        assert_eq!(q.clipped, 3);
    }
}
