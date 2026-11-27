// SPDX-License-Identifier: GPL-3.0-or-later
//! Streaming filters that clean the heart signal before it is analysed.
//!
//! Each filter is a "biquad": a tiny recipe that uses the last two inputs and
//! outputs to compute the next output. They work one sample at a time and
//! need no memory beyond a few numbers. The formulas are the standard
//! "Audio EQ Cookbook" designs (Robert Bristow-Johnson).

use crate::settings::{
    HIGH_PASS_HZ, LOW_PASS_HZ, MAINS_HZ, NOISE_BAND_HZ, NOTCH_Q, SAMPLE_RATE_HZ,
};
use core::f32::consts::{FRAC_1_SQRT_2, PI};

/// One second-order filter section.
#[derive(Debug, Clone, Copy)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

/// The shape of a [`Biquad`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Keeps slow changes, removes fast ones.
    LowPass,
    /// Keeps fast changes, removes slow ones.
    HighPass,
    /// Keeps a band around the centre frequency.
    BandPass,
    /// Removes a narrow band around the centre frequency (for mains hum).
    Notch,
}

impl Biquad {
    /// Design a filter of the given kind, corner/centre frequency `hz` and
    /// sharpness `q`, for signals sampled at `sample_rate_hz`.
    #[must_use]
    pub fn new(kind: Kind, hz: f32, q: f32, sample_rate_hz: f32) -> Self {
        let w0 = 2.0 * PI * hz / sample_rate_hz;
        let cw = libm::cosf(w0);
        let alpha = libm::sinf(w0) / (2.0 * q);
        let (b0, b1, b2) = match kind {
            Kind::LowPass => ((1.0 - cw) / 2.0, 1.0 - cw, (1.0 - cw) / 2.0),
            Kind::HighPass => ((1.0 + cw) / 2.0, -(1.0 + cw), (1.0 + cw) / 2.0),
            Kind::BandPass => (alpha, 0.0, -alpha),
            Kind::Notch => (1.0, -2.0 * cw, 1.0),
        };
        let a0 = 1.0 + alpha;
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: -2.0 * cw / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Filter one sample.
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    /// Set the filter's memory as if it had been fed the steady value `x` for
    /// a long time. Used on the first sample so a large electrode offset does
    /// not cause a long settling swing.
    pub fn prime(&mut self, x: f32) {
        let dc_gain_den = 1.0 + self.a1 + self.a2;
        let y = if libm::fabsf(dc_gain_den) > 1e-9 {
            x * (self.b0 + self.b1 + self.b2) / dc_gain_den
        } else {
            0.0
        };
        self.z1 = y - self.b0 * x;
        self.z2 = self.b2 * x - self.a2 * y;
    }

    /// Forget the filter's memory.
    pub fn clear(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

/// The full cleaning chain for the analyser.
///
/// High-pass at [`HIGH_PASS_HZ`], two mains notches, then it splits: a
/// low-pass at [`LOW_PASS_HZ`] gives the signal that is analysed, and a
/// high-pass at [`NOISE_BAND_HZ`] gives a "noise" signal used only to judge
/// signal quality.
#[derive(Debug, Clone, Copy)]
pub struct Preprocessor {
    high_pass: Biquad,
    notches: [Biquad; 2],
    low_pass: Biquad,
    noise_tap: Biquad,
    primed: bool,
}

/// What [`Preprocessor::process`] gives back for one input sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cleaned {
    /// The band-limited heart signal, microvolts.
    pub signal_uv: f32,
    /// The high-frequency part, microvolts (should be small on a clean signal).
    pub noise_uv: f32,
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor {
    /// Build the chain from the values in [`crate::settings`].
    #[must_use]
    pub fn new() -> Self {
        let [m1, m2] = MAINS_HZ;
        Self {
            high_pass: Biquad::new(Kind::HighPass, HIGH_PASS_HZ, FRAC_1_SQRT_2, SAMPLE_RATE_HZ),
            notches: [
                Biquad::new(Kind::Notch, m1, NOTCH_Q, SAMPLE_RATE_HZ),
                Biquad::new(Kind::Notch, m2, NOTCH_Q, SAMPLE_RATE_HZ),
            ],
            low_pass: Biquad::new(Kind::LowPass, LOW_PASS_HZ, FRAC_1_SQRT_2, SAMPLE_RATE_HZ),
            noise_tap: Biquad::new(Kind::HighPass, NOISE_BAND_HZ, FRAC_1_SQRT_2, SAMPLE_RATE_HZ),
            primed: false,
        }
    }

    /// Clean one raw sample (microvolts).
    pub fn process(&mut self, raw_uv: f32) -> Cleaned {
        if !self.primed {
            self.high_pass.prime(raw_uv);
            self.primed = true;
        }
        let mut y = self.high_pass.process(raw_uv);
        for notch in &mut self.notches {
            y = notch.process(y);
        }
        Cleaned {
            signal_uv: self.low_pass.process(y),
            noise_uv: self.noise_tap.process(y),
        }
    }

    /// Forget everything (used by the analyser's reset).
    pub fn clear(&mut self) {
        self.high_pass.clear();
        for notch in &mut self.notches {
            notch.clear();
        }
        self.low_pass.clear();
        self.noise_tap.clear();
        self.primed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gain_at(kind: Kind, corner: f32, q: f32, test_hz: f32) -> f32 {
        let mut f = Biquad::new(kind, corner, q, SAMPLE_RATE_HZ);
        let mut peak = 0.0_f32;
        for n in 0..5000 {
            let x = libm::sinf(2.0 * PI * test_hz * n as f32 / SAMPLE_RATE_HZ);
            let y = f.process(x);
            if n > 2500 {
                peak = peak.max(libm::fabsf(y));
            }
        }
        peak
    }

    #[test]
    fn low_pass_keeps_slow_and_cuts_fast() {
        assert!(gain_at(Kind::LowPass, 30.0, FRAC_1_SQRT_2, 5.0) > 0.95);
        assert!(gain_at(Kind::LowPass, 30.0, FRAC_1_SQRT_2, 100.0) < 0.15);
    }

    #[test]
    fn high_pass_cuts_drift() {
        assert!(gain_at(Kind::HighPass, 1.0, FRAC_1_SQRT_2, 0.1) < 0.05);
        assert!(gain_at(Kind::HighPass, 1.0, FRAC_1_SQRT_2, 10.0) > 0.95);
    }

    #[test]
    fn notch_removes_mains() {
        assert!(gain_at(Kind::Notch, 50.0, NOTCH_Q, 50.0) < 0.05);
        assert!(gain_at(Kind::Notch, 60.0, NOTCH_Q, 60.0) < 0.05);
        assert!(gain_at(Kind::Notch, 50.0, NOTCH_Q, 10.0) > 0.95);
    }

    #[test]
    fn priming_removes_offset_step() {
        let mut p = Preprocessor::new();
        let mut worst = 0.0_f32;
        for _ in 0..500 {
            worst = worst.max(libm::fabsf(p.process(20_000.0).signal_uv));
        }
        assert!(worst < 1.0, "offset leaked through: {worst}");
    }
}
