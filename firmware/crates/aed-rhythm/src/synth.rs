// SPDX-License-Identifier: GPL-3.0-or-later
//! Made-up heart signals for testing and for the simulator.
//!
//! **These are cartoons of heart rhythms, not recordings.** They are simple
//! enough to reason about and repeatable (the same seed always gives the
//! same signal), which is what tests need. Passing on them proves the code
//! runs and behaves sensibly; it proves nothing about real patients.
//!
//! Use [`Synth::new`] with a [`Rhythm`] and a seed, optionally add noise,
//! baseline wander, mains hum or chest-compression artifact, then call
//! [`Synth::next_sample`] (or use it as an iterator) to get samples at
//! 250 per second in microvolts.

use crate::settings::SAMPLE_RATE_HZ;
use aed_types::EcgSample;
use core::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;
const DT: f32 = 1.0 / SAMPLE_RATE_HZ;

/// A tiny repeatable random-number generator (xorshift32, Marsaglia 2003).
/// Not suitable for anything secret; fine for test signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xorshift32 {
    state: u32,
}

impl Xorshift32 {
    /// Start from `seed`. A seed of zero is replaced, because zero would repeat forever.
    #[must_use]
    pub const fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9 } else { seed },
        }
    }

    /// Next raw 32-bit number.
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Next number evenly spread between 0 and 1.
    pub fn next_unit(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Next number roughly bell-shaped around 0 with spread (standard deviation) 1.
    /// Sum of four even numbers, rescaled; good enough for test noise.
    pub fn next_gaussian(&mut self) -> f32 {
        let s = self.next_unit() + self.next_unit() + self.next_unit() + self.next_unit();
        // Four unit-uniforms: mean 2, variance 4/12.
        (s - 2.0) * 1.732_050_8
    }
}

/// Which heart rhythm to make.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rhythm {
    /// Normal beats (a P-QRS-T shape) at a heart rate in beats per minute.
    /// `amplitude_uv` is the height of the main spike (R wave).
    Sinus {
        /// Heart rate, beats per minute.
        bpm: f32,
        /// R-wave height, microvolts.
        amplitude_uv: f32,
    },
    /// Ventricular fibrillation: a chaotic wobble made of a few drifting
    /// 3-7 Hz waves with a slowly changing size.
    VentricularFibrillation {
        /// Approximate peak-to-peak size, microvolts.
        amplitude_uv: f32,
    },
    /// Rapid, regular ventricular tachycardia: wide, rounded complexes.
    VentricularTachycardia {
        /// Rate, beats per minute (150-250 is the range of interest).
        bpm: f32,
        /// Approximate peak-to-peak size, microvolts.
        amplitude_uv: f32,
    },
    /// Flatline: nothing but a little noise from the electronics.
    Asystole,
}

impl Rhythm {
    /// Normal rhythm at `bpm` with a 1 mV R wave.
    #[must_use]
    pub const fn sinus(bpm: f32) -> Self {
        Rhythm::Sinus {
            bpm,
            amplitude_uv: 1000.0,
        }
    }

    /// Coarse VF, about 800 µV peak to peak.
    #[must_use]
    pub const fn coarse_vf() -> Self {
        Rhythm::VentricularFibrillation {
            amplitude_uv: 800.0,
        }
    }

    /// Fine VF, about 120 µV peak to peak (below the 200 µV coarse/fine line).
    #[must_use]
    pub const fn fine_vf() -> Self {
        Rhythm::VentricularFibrillation {
            amplitude_uv: 120.0,
        }
    }

    /// Rapid VT at `bpm`, about 1 mV peak to peak.
    #[must_use]
    pub const fn rapid_vt(bpm: f32) -> Self {
        Rhythm::VentricularTachycardia {
            bpm,
            amplitude_uv: 1200.0,
        }
    }
}

/// A repeatable synthetic heart-signal source. See the module notes.
#[derive(Debug, Clone)]
pub struct Synth {
    rhythm: Rhythm,
    rng: Xorshift32,
    // Sinus
    since_beat_s: f32,
    rr_s: f32,
    // VF
    vf_phase: [f32; 3],
    vf_hz: [f32; 3],
    vf_target_hz: [f32; 3],
    vf_envelope_phase: f32,
    vf_retarget_count: u32,
    // VT
    vt_phase: f32,
    // Extras
    noise_rms_uv: f32,
    wander_uv: f32,
    wander_phase: f32,
    mains_uv: f32,
    mains_hz: f32,
    mains_phase: f32,
    compression_uv: f32,
    compression_hz: f32,
    compression_phase: f32,
    compression_rate_now: f32,
    offset_uv: f32,
}

impl Synth {
    /// Make a source for `rhythm`. Different seeds give different but
    /// repeatable variations.
    #[must_use]
    pub fn new(rhythm: Rhythm, seed: u32) -> Self {
        let mut rng = Xorshift32::new(seed);
        let mut vf_hz = [0.0; 3];
        let mut vf_phase = [0.0; 3];
        for (hz, ph) in vf_hz.iter_mut().zip(vf_phase.iter_mut()) {
            *hz = 3.0 + 4.0 * rng.next_unit();
            *ph = TWO_PI * rng.next_unit();
        }
        let envelope = TWO_PI * rng.next_unit();
        let mut s = Self {
            rhythm,
            rng,
            since_beat_s: 0.0,
            rr_s: 1.0,
            vf_phase,
            vf_hz,
            vf_target_hz: vf_hz,
            vf_envelope_phase: envelope,
            vf_retarget_count: 0,
            vt_phase: 0.0,
            noise_rms_uv: 0.0,
            wander_uv: 0.0,
            wander_phase: 0.0,
            mains_uv: 0.0,
            mains_hz: 50.0,
            mains_phase: 0.0,
            compression_uv: 0.0,
            compression_hz: 110.0 / 60.0,
            compression_phase: 0.0,
            compression_rate_now: 110.0 / 60.0,
            offset_uv: 0.0,
        };
        s.rr_s = s.next_rr();
        // Start partway into a beat so different seeds are not all aligned.
        s.since_beat_s = s.rr_s * s.rng.next_unit();
        s
    }

    /// Add random noise with the given spread (standard deviation), microvolts.
    #[must_use]
    pub fn with_noise(mut self, rms_uv: f32) -> Self {
        self.noise_rms_uv = rms_uv;
        self
    }

    /// Add slow baseline wander (0.25 Hz, as from breathing), microvolts peak.
    #[must_use]
    pub fn with_baseline_wander(mut self, amplitude_uv: f32) -> Self {
        self.wander_uv = amplitude_uv;
        self
    }

    /// Add mains hum at `hz` (50 or 60), microvolts peak.
    #[must_use]
    pub fn with_mains(mut self, amplitude_uv: f32, hz: f32) -> Self {
        self.mains_uv = amplitude_uv;
        self.mains_hz = hz;
        self
    }

    /// Add chest-compression artifact: a large, rounded, roughly regular wave
    /// at about `per_minute` compressions per minute (rate wobbles by a few
    /// percent), `amplitude_uv` microvolts peak to peak.
    #[must_use]
    pub fn with_compressions(mut self, amplitude_uv: f32, per_minute: f32) -> Self {
        self.compression_uv = amplitude_uv;
        self.compression_hz = per_minute / 60.0;
        self.compression_rate_now = self.compression_hz;
        self
    }

    /// Add a constant offset (as from electrode polarisation), microvolts.
    #[must_use]
    pub fn with_offset(mut self, offset_uv: f32) -> Self {
        self.offset_uv = offset_uv;
        self
    }

    /// Switch to a different rhythm from the next sample on (the extras stay).
    pub fn set_rhythm(&mut self, rhythm: Rhythm) {
        self.rhythm = rhythm;
        self.since_beat_s = 0.0;
        self.rr_s = self.next_rr();
    }

    /// The rhythm currently being made.
    #[must_use]
    pub fn rhythm(&self) -> Rhythm {
        self.rhythm
    }

    /// The next sample, microvolts, clipped to the range of an [`EcgSample`].
    pub fn next_sample(&mut self) -> EcgSample {
        let v = self.next_uv();
        let clipped = v.clamp(f32::from(i16::MIN), f32::from(i16::MAX));
        // Value is already clamped into range; `as` rounds toward zero.
        clipped as i16
    }

    /// The next sample as an unclipped decimal number, microvolts.
    pub fn next_uv(&mut self) -> f32 {
        let heart = match self.rhythm {
            Rhythm::Sinus { amplitude_uv, .. } => amplitude_uv * self.sinus_step(),
            Rhythm::VentricularFibrillation { amplitude_uv } => amplitude_uv * self.vf_step(),
            Rhythm::VentricularTachycardia { bpm, amplitude_uv } => {
                amplitude_uv * self.vt_step(bpm)
            }
            Rhythm::Asystole => 0.0,
        };
        let floor_noise = if matches!(self.rhythm, Rhythm::Asystole) {
            8.0 * self.rng.next_gaussian()
        } else {
            0.0
        };
        heart + floor_noise + self.extras_step() + self.offset_uv
    }

    fn next_rr(&mut self) -> f32 {
        let bpm = match self.rhythm {
            Rhythm::Sinus { bpm, .. } => bpm,
            _ => 60.0,
        };
        let base = 60.0 / bpm.max(1.0);
        // Natural beat-to-beat variation of about +/-3 %.
        base * (1.0 + 0.06 * (self.rng.next_unit() - 0.5))
    }

    /// One sample of a P-QRS-T beat train, scaled so the R wave is 1.0.
    fn sinus_step(&mut self) -> f32 {
        self.since_beat_s += DT;
        if self.since_beat_s >= self.rr_s {
            self.since_beat_s -= self.rr_s;
            self.rr_s = self.next_rr();
        }
        let t = self.since_beat_s;
        // Current beat plus the start (P wave) of the next one.
        beat_template(t, self.rr_s) + beat_template(t - self.rr_s, self.rr_s)
    }

    /// One sample of VF, scaled to roughly 1.0 peak to peak.
    fn vf_step(&mut self) -> f32 {
        // Every second, each component picks a new 3-7 Hz frequency to drift toward.
        self.vf_retarget_count += 1;
        if self.vf_retarget_count >= 250 {
            self.vf_retarget_count = 0;
            for target in &mut self.vf_target_hz {
                *target = 3.0 + 4.0 * self.rng.next_unit();
            }
        }
        let weights = [1.0_f32, 0.6, 0.4];
        let mut sum = 0.0;
        for (((phase, hz), target), w) in self
            .vf_phase
            .iter_mut()
            .zip(self.vf_hz.iter_mut())
            .zip(self.vf_target_hz.iter())
            .zip(weights.iter())
        {
            *hz += (target - *hz) * 0.004;
            *phase = (*phase + TWO_PI * *hz * DT) % TWO_PI;
            sum += w * libm::sinf(*phase);
        }
        self.vf_envelope_phase = (self.vf_envelope_phase + TWO_PI * 0.2 * DT) % TWO_PI;
        let envelope = 0.7 + 0.3 * libm::sinf(self.vf_envelope_phase);
        // Sum of weights is 2.0; typical peaks reach about 1.4.
        0.5 * envelope * sum / 1.4
    }

    /// One sample of monomorphic VT, scaled to roughly 1.0 peak to peak.
    fn vt_step(&mut self, bpm: f32) -> f32 {
        self.vt_phase = (self.vt_phase + TWO_PI * (bpm / 60.0) * DT) % TWO_PI;
        let p = self.vt_phase;
        0.5 * (libm::sinf(p) + 0.3 * libm::sinf(2.0 * p - 0.8)) / 1.2
    }

    fn extras_step(&mut self) -> f32 {
        let mut v = 0.0;
        if self.noise_rms_uv > 0.0 {
            v += self.noise_rms_uv * self.rng.next_gaussian();
        }
        if self.wander_uv > 0.0 {
            self.wander_phase = (self.wander_phase + TWO_PI * 0.25 * DT) % TWO_PI;
            v += self.wander_uv * libm::sinf(self.wander_phase);
        }
        if self.mains_uv > 0.0 {
            self.mains_phase = (self.mains_phase + TWO_PI * self.mains_hz * DT) % TWO_PI;
            v += self.mains_uv * libm::sinf(self.mains_phase);
        }
        if self.compression_uv > 0.0 {
            let before = self.compression_phase;
            self.compression_phase =
                (self.compression_phase + TWO_PI * self.compression_rate_now * DT) % TWO_PI;
            if self.compression_phase < before {
                // New compression: rate wobbles by +/-5 %.
                self.compression_rate_now =
                    self.compression_hz * (1.0 + 0.1 * (self.rng.next_unit() - 0.5));
            }
            let p = self.compression_phase;
            let shape = libm::sinf(p) + 0.35 * libm::sinf(2.0 * p) + 0.1 * libm::sinf(3.0 * p);
            v += 0.5 * self.compression_uv * shape / 1.3;
        }
        v
    }
}

/// One P-QRS-T beat built from bell curves, R peak at `t = 0`, R height 1.0.
/// Wave timings scale with the square root of the beat interval, roughly as
/// the real QT interval does. A cartoon, not a physiological model.
fn beat_template(t: f32, rr_s: f32) -> f32 {
    let s = libm::sqrtf(rr_s.max(0.2));
    // (centre seconds, width seconds, height)
    let waves = [
        (-0.16 * s, 0.025, 0.12),  // P
        (-0.03, 0.008, -0.12),     // Q
        (0.0, 0.010, 1.0),         // R
        (0.03, 0.010, -0.25),      // S
        (0.28 * s, 0.05 * s, 0.3), // T
    ];
    waves
        .iter()
        .map(|(c, w, a)| {
            let d = (t - c) / w;
            a * libm::expf(-0.5 * d * d)
        })
        .sum()
}

impl Iterator for Synth {
    type Item = EcgSample;

    fn next(&mut self) -> Option<EcgSample> {
        Some(self.next_sample())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pp(mut s: Synth, n: usize) -> f32 {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for _ in 0..n {
            let v = s.next_uv();
            lo = lo.min(v);
            hi = hi.max(v);
        }
        hi - lo
    }

    #[test]
    fn same_seed_same_signal() {
        let a: [i16; 64] = core::array::from_fn({
            let mut s = Synth::new(Rhythm::coarse_vf(), 7).with_noise(20.0);
            move |_| s.next_sample()
        });
        let b: [i16; 64] = core::array::from_fn({
            let mut s = Synth::new(Rhythm::coarse_vf(), 7).with_noise(20.0);
            move |_| s.next_sample()
        });
        assert_eq!(a, b);
    }

    #[test]
    fn amplitudes_roughly_as_documented() {
        let vf = pp(Synth::new(Rhythm::coarse_vf(), 1), 2500);
        assert!((400.0..1200.0).contains(&vf), "coarse VF pp {vf}");
        let fine = pp(Synth::new(Rhythm::fine_vf(), 1), 2500);
        assert!((60.0..200.0).contains(&fine), "fine VF pp {fine}");
        let asys = pp(Synth::new(Rhythm::Asystole, 1), 2500);
        assert!(asys < 100.0, "asystole pp {asys}");
        let nsr = pp(Synth::new(Rhythm::sinus(70.0), 1), 2500);
        assert!((1000.0..1600.0).contains(&nsr), "sinus pp {nsr}");
    }

    #[test]
    fn huge_signals_clip_instead_of_wrapping() {
        let mut s = Synth::new(Rhythm::sinus(60.0), 3).with_offset(100_000.0);
        assert_eq!(s.next_sample(), i16::MAX);
    }
}
