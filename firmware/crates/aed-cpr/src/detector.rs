// SPDX-License-Identifier: GPL-3.0-or-later
//! The streaming compression detector.
//!
//! How it works, in plain words:
//!
//! 1. **Is the signal usable?** A sample that is saturated, outside the
//!    25 to 250 Ω pad range, or that has not changed at all for
//!    [`Settings::stuck_ms`] makes the answer `Unknown` at once. After the
//!    signal comes back the detector waits [`Settings::settle_ms`] before it
//!    will say anything else.
//! 2. **Filter.** The baseline (the patient's own resistance) and slow
//!    changes such as breaths are removed with a high-pass filter; fast
//!    noise with a low-pass filter. What is left is the compression wave.
//! 3. **Movement.** Peaks and troughs of the filtered signal are tracked.
//!    A swing of at least [`Settings::min_compression_mohm`], up or down,
//!    that took no longer than [`Settings::max_rise_ms`] is movement: the
//!    answer cannot be `HandsOff` for [`Settings::quiet_min_ms`] after it.
//!    A slower swing (a breath takes about a second to go in) is not.
//! 4. **Compressions.** Each fast rise to a peak is one compression. Peaks closer
//!    together than [`Settings::min_interval_ms`] are merged.
//!    [`Settings::compressions_to_confirm`] in a row, each within the
//!    interval limits, make the answer `Compressing`.
//! 5. **Hands off** only after a quiet period with no movement at all.
//!
//! The answer is therefore never `HandsOff` while there is any doubt. The
//! worst-case latencies are stated in the crate documentation and measured
//! in the tests and the evaluation.

use crate::settings::{Settings, SettingsError};
use crate::{MAX_ROLLING_SECONDS, SAMPLE_MS};
use aed_types::{
    ChestEvent, ChestSensor, ChestState, ImpedanceSample, ImpedanceSignal,
    IMPEDANCE_SAMPLE_RATE_HZ,
};
use core::f32::consts::PI;

/// How many filtered samples the detector keeps to look back over (1.28 s).
pub(crate) const HISTORY_LEN: usize = 160;
/// Samples over which fast noise is measured (about 0.5 s).
const NOISE_LEN: usize = 63;
/// How much further back than the longest rise the detector looks to find
/// where a swing began, ms.
pub(crate) const TURN_MARGIN_MS: u32 = 400;
/// Compression intervals kept for the rate.
const INTERVALS: usize = 5;

/// One second-order filter section (transposed direct form II).
#[derive(Debug, Clone, Copy)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    /// Butterworth-style section by the bilinear transform: low-pass
    /// (`high == false`) or high-pass, corner `hz`, quality `q`.
    fn new(high: bool, hz: f32, q: f32) -> Self {
        let fs = IMPEDANCE_SAMPLE_RATE_HZ as f32;
        let k = libm::tanf(PI * hz / fs);
        let kk = k * k;
        let denom = 1.0 + k / q + kk;
        let norm = if denom.abs() > 1e-12 { 1.0 / denom } else { 0.0 };
        let (b0, b1, b2) = if high {
            (norm, -2.0 * norm, norm)
        } else {
            (kk * norm, 2.0 * kk * norm, kk * norm)
        };
        Biquad {
            b0,
            b1,
            b2,
            a1: 2.0 * (kk - 1.0) * norm,
            a2: (1.0 - k / q + kk) * norm,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// A section that passes its input unchanged.
    const fn identity() -> Self {
        Biquad {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    fn clear(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

/// Q factors of the two sections of a fourth-order Butterworth filter.
const BUTTERWORTH4_Q: [f32; 2] = [0.541_196_1, 1.306_563];
/// Q factor of a second-order Butterworth filter.
const BUTTERWORTH2_Q: f32 = core::f32::consts::FRAC_1_SQRT_2;

/// The compression detector. Feed it every pad-impedance sample at
/// [`IMPEDANCE_SAMPLE_RATE_HZ`] through [`ChestSensor::push`].
///
/// Fixed size, no heap: about 1.3 KB.
#[derive(Debug, Clone)]
pub struct CompressionDetector {
    settings: Settings,
    hp1: Biquad,
    hp2: Biquad,
    lp: Biquad,
    /// The first good reading after a (re)start, subtracted from every
    /// reading so the filters start from rest.
    x_ref: Option<f32>,
    /// Samples seen since the last reset.
    n: u32,
    /// Filtered signal, ohms, by sample number modulo `HISTORY_LEN`.
    history: [f32; HISTORY_LEN],
    /// Number of valid entries in `history` (up to `HISTORY_LEN`).
    filled: u32,
    /// Peak tracking: true while looking for a maximum.
    rising: bool,
    extreme: f32,
    extreme_n: u32,
    /// Fast-noise measurement.
    prev1: Option<f32>,
    prev2: Option<f32>,
    noise_sq: [f32; NOISE_LEN],
    noise_count: usize,
    noise_head: usize,
    /// Stuck-reading detection.
    last_raw: Option<u32>,
    same_count: u32,
    signal: ImpedanceSignal,
    settle_until: u32,
    last_activity_n: Option<u32>,
    /// When the state last became `HandsOff`.
    hands_off_since_n: Option<u32>,
    last_compression_n: Option<u32>,
    run_len: u8,
    intervals_ms: [u32; INTERVALS],
    interval_count: usize,
    interval_head: usize,
    state: ChestState,
    rolling: [u8; MAX_ROLLING_SECONDS],
    rolling_filled: usize,
    rolling_head: usize,
    second_count: u8,
    second_samples: u8,
    compressions_total: u32,
    last_amplitude_mohm: u32,
}

impl CompressionDetector {
    /// A detector with the given settings.
    ///
    /// # Errors
    /// Refuses settings that contradict each other.
    pub fn new(settings: Settings) -> Result<Self, SettingsError> {
        settings.validate()?;
        Ok(Self::build(settings))
    }

    fn build(settings: Settings) -> Self {
        let [q1, q2] = BUTTERWORTH4_Q;
        let (hp1, hp2) = if settings.highpass_fourth_order {
            (
                Biquad::new(true, settings.highpass_hz, q1),
                Biquad::new(true, settings.highpass_hz, q2),
            )
        } else {
            (
                Biquad::new(true, settings.highpass_hz, BUTTERWORTH2_Q),
                Biquad::identity(),
            )
        };
        CompressionDetector {
            settings,
            hp1,
            hp2,
            lp: Biquad::new(false, settings.lowpass_hz, BUTTERWORTH2_Q),
            x_ref: None,
            n: 0,
            history: [0.0; HISTORY_LEN],
            filled: 0,
            rising: true,
            extreme: 0.0,
            extreme_n: 0,
            prev1: None,
            prev2: None,
            noise_sq: [0.0; NOISE_LEN],
            noise_count: 0,
            noise_head: 0,
            last_raw: None,
            same_count: 0,
            signal: ImpedanceSignal::Settling,
            settle_until: 0,
            last_activity_n: None,
            hands_off_since_n: None,
            last_compression_n: None,
            run_len: 0,
            intervals_ms: [0; INTERVALS],
            interval_count: 0,
            interval_head: 0,
            state: ChestState::Unknown,
            rolling: [0; MAX_ROLLING_SECONDS],
            rolling_filled: 0,
            rolling_head: 0,
            second_count: 0,
            second_samples: 0,
            compressions_total: 0,
            last_amplitude_mohm: 0,
        }
    }

    /// The settings in use.
    #[must_use]
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Compressions detected since the last reset.
    #[must_use]
    pub const fn compressions(&self) -> u32 {
        self.compressions_total
    }

    /// Size of the last detected compression, milliohms peak to peak (filtered).
    #[must_use]
    pub const fn last_amplitude_mohm(&self) -> u32 {
        self.last_amplitude_mohm
    }

    /// The most recent filtered value, ohms (for diagnostics).
    #[must_use]
    pub fn filtered(&self) -> f32 {
        self.hist(self.n.wrapping_sub(1)).unwrap_or(0.0)
    }

    fn ms_to_samples(ms: u32) -> u32 {
        ms.div_ceil(SAMPLE_MS)
    }

    fn hist(&self, at: u32) -> Option<f32> {
        let age = self.n.wrapping_sub(1).wrapping_sub(at);
        if age >= self.filled {
            return None;
        }
        let slot = (at as usize) % HISTORY_LEN;
        self.history.get(slot).copied()
    }

    /// The signal is unusable: forget the filters and say `Unknown`.
    fn invalidate(&mut self, why: ImpedanceSignal) {
        self.signal = why;
        self.x_ref = None;
        self.run_len = 0;
        self.interval_count = 0;
        self.filled = 0;
        self.prev1 = None;
        self.prev2 = None;
        self.noise_count = 0;
        self.state = ChestState::Unknown;
    }

    fn restart_filters(&mut self, x: f32) {
        self.x_ref = Some(x);
        self.hp1.clear();
        self.hp2.clear();
        self.lp.clear();
        self.filled = 0;
        self.rising = true;
        self.extreme = 0.0;
        self.extreme_n = self.n;
        self.settle_until = self
            .n
            .saturating_add(Self::ms_to_samples(self.settings.settle_ms));
    }

    /// Fast-noise RMS over the last half second, ohms (None until full).
    fn noise_rms(&mut self, x: f32) -> Option<f32> {
        let (Some(p1), Some(p2)) = (self.prev1, self.prev2) else {
            self.prev2 = self.prev1;
            self.prev1 = Some(x);
            return None;
        };
        let d2 = x - 2.0 * p1 + p2;
        self.prev2 = Some(p1);
        self.prev1 = Some(x);
        if let Some(slot) = self.noise_sq.get_mut(self.noise_head) {
            *slot = d2 * d2;
        }
        self.noise_head = (self.noise_head + 1) % NOISE_LEN;
        self.noise_count = (self.noise_count + 1).min(NOISE_LEN);
        if self.noise_count < NOISE_LEN {
            return None;
        }
        let sum: f32 = self.noise_sq.iter().sum();
        Some(libm::sqrtf(sum / NOISE_LEN as f32))
    }

    fn median_interval_ms(&self) -> Option<u32> {
        let count = self.interval_count.min(INTERVALS);
        if count == 0 {
            return None;
        }
        let mut sorted = [0u32; INTERVALS];
        for (dst, src) in sorted.iter_mut().zip(self.intervals_ms.iter()).take(count) {
            *dst = *src;
        }
        let part = sorted.get_mut(..count)?;
        part.sort_unstable();
        part.get(count / 2).copied()
    }

    fn quiet_samples(&self) -> u32 {
        let base = self.settings.quiet_min_ms;
        let adaptive = if self.run_len >= 2 {
            self.median_interval_ms().map_or(0, |m| {
                m.saturating_mul(u32::from(self.settings.quiet_intervals_tenths)) / 10
            })
        } else {
            0
        };
        Self::ms_to_samples(base.max(adaptive))
    }

    /// A swing that ended at `at` (a peak if `up`, a trough if not): was it
    /// fast and big enough to be a compression or a movement?
    ///
    /// Walks back from `at` to the turning point where the swing began (the
    /// nearest trough before a peak, or peak before a trough, found with the
    /// same hysteresis as the tracking). That turning point must lie within
    /// [`Settings::max_rise_ms`]: if the walk reaches that far back without
    /// finding it, the swing took longer, which is a breath or a slow
    /// movement. Returns the size of the swing, ohms.
    fn fast_swing(&self, at: u32, up: bool) -> Option<f32> {
        let s = self.settings;
        // A compression turns back quickly too: a peak or trough confirmed
        // long after it happened is the top of a slow hump.
        let now = self.n.wrapping_sub(1);
        if now.wrapping_sub(at).saturating_mul(SAMPLE_MS) > s.max_rise_ms {
            return None;
        }
        // Look back a little further than the longest rise, to see the
        // signal turn beyond the start of the swing.
        let window = Self::ms_to_samples(s.max_rise_ms.saturating_add(TURN_MARGIN_MS));
        let threshold = s.min_compression_mohm as f32 / 1000.0;
        let h = threshold / 2.0;
        let end = self.hist(at)?;
        // Running extreme in the direction of the swing's start.
        let mut best = end;
        let mut best_at = at;
        for back in 1..=window {
            let t = at.wrapping_sub(back);
            let Some(v) = self.hist(t) else { break };
            let further = if up { v < best } else { v > best };
            if further {
                best = v;
                best_at = t;
            } else if (up && v - best >= h) || (!up && best - v >= h) {
                // Turned: the swing started at `best_at`.
                break;
            }
        }
        // If it never turned, `best_at` is the furthest point in the
        // direction of the start within the look-back: on a flat stretch
        // (hands off before the first compression) that is where the rise
        // began; on a slow slope it is far back, and the rise is too slow.
        let swing = (end - best).abs();
        // On a flat, slightly noisy stretch the lowest point can be anywhere
        // in it; the swing began at the last point close to it.
        let tolerance = threshold / 10.0;
        let mut start_at = best_at;
        for back in 1..=window {
            let t = at.wrapping_sub(back);
            if t == best_at {
                break;
            }
            let Some(v) = self.hist(t) else { break };
            if (up && v <= best + tolerance) || (!up && v >= best - tolerance) {
                start_at = t;
                break;
            }
        }
        let rise_ms = at.wrapping_sub(start_at).saturating_mul(SAMPLE_MS);
        // The steep part: from 10 % to 90 % of the swing. A compression's
        // takes about 60 % of its rise (0.1 to 0.3 s); a breath's 0.4 s or more.
        let mut t90 = None;
        let mut t10 = None;
        for back in 0..=window {
            let t = at.wrapping_sub(back);
            let Some(v) = self.hist(t) else { break };
            let done = if up { end - v } else { v - end };
            if t90.is_none() && done >= 0.1 * swing {
                t90 = Some(t);
            }
            if done >= 0.9 * swing {
                t10 = Some(t);
                break;
            }
        }
        let steep_ms = match (t90, t10) {
            (Some(a), Some(b)) => a.wrapping_sub(b).saturating_mul(SAMPLE_MS),
            _ => u32::MAX,
        };
        let fast = rise_ms >= s.min_rise_ms
            && rise_ms <= s.max_rise_ms
            && steep_ms <= s.max_steep_ms;
        (swing >= threshold && fast).then_some(swing)
    }

    /// Movement happened at sample `at` (the moment of the peak or trough,
    /// not the later moment it was confirmed), unless something later is
    /// already recorded.
    fn note_activity(&mut self, at: u32) {
        let now = self.n.wrapping_sub(1);
        let newer = self
            .last_activity_n
            .is_none_or(|prev| now.wrapping_sub(at) < now.wrapping_sub(prev));
        // A swing that ended before the chest was last declared still
        // changes nothing: the signal since then has been quiet.
        let before_quiet = self.state == ChestState::HandsOff
            && self
                .hands_off_since_n
                .is_some_and(|h| now.wrapping_sub(at) > now.wrapping_sub(h));
        if newer && !before_quiet {
            self.last_activity_n = Some(at);
        }
    }

    /// A peak was confirmed at sample `at`: count a compression if it was one.
    fn on_peak(&mut self, at: u32) -> Option<ChestEvent> {
        let s = self.settings;
        let swing = self.fast_swing(at, true)?;
        self.note_activity(at);
        let since_samples = self.last_compression_n.map(|p| at.wrapping_sub(p));
        let since_ms = since_samples.map_or(u32::MAX, |d| d.saturating_mul(SAMPLE_MS));
        if since_ms < s.min_interval_ms {
            // Too close to the previous one: part of the same compression.
            return None;
        }
        if since_ms <= s.max_interval_ms {
            self.run_len = self.run_len.saturating_add(1);
            if let Some(slot) = self.intervals_ms.get_mut(self.interval_head) {
                *slot = since_ms;
            }
            self.interval_head = (self.interval_head + 1) % INTERVALS;
            self.interval_count = (self.interval_count + 1).min(INTERVALS);
        } else {
            self.run_len = 1;
            self.interval_count = 0;
        }
        self.last_compression_n = Some(at);
        self.compressions_total = self.compressions_total.saturating_add(1);
        let amplitude = (swing * 1000.0).max(0.0) as u32;
        self.last_amplitude_mohm = amplitude;
        Some(ChestEvent::Compression {
            amplitude_mohm: amplitude,
            since_previous_ms: since_ms,
        })
    }

    /// Process one good, filtered value. Returns a compression event, if any.
    ///
    /// Peaks and troughs are tracked with hysteresis of half the threshold.
    /// When one is confirmed (the signal has turned back by that much), the
    /// swing that led to it is judged by [`Self::fast_swing`]. A fast,
    /// big-enough swing either way is movement; a fast rise to a peak is
    /// also a compression.
    fn track(&mut self, y: f32) -> Option<ChestEvent> {
        let threshold = self.settings.min_compression_mohm as f32 / 1000.0;
        let now = self.n.wrapping_sub(1);
        let h = threshold / 2.0;
        let mut event = None;
        if self.rising {
            if y > self.extreme {
                self.extreme = y;
                self.extreme_n = now;
            } else if self.extreme - y >= h {
                event = self.on_peak(self.extreme_n);
                self.rising = false;
                self.extreme = y;
                self.extreme_n = now;
            }
        } else if y < self.extreme {
            self.extreme = y;
            self.extreme_n = now;
        } else if y - self.extreme >= h {
            if self.fast_swing(self.extreme_n, false).is_some() {
                self.note_activity(self.extreme_n);
            }
            self.rising = true;
            self.extreme = y;
            self.extreme_n = now;
        }
        event
    }

    fn update_state(&mut self) {
        let now = self.n.wrapping_sub(1);
        if self.signal != ImpedanceSignal::Good {
            self.state = ChestState::Unknown;
            return;
        }
        let since_compression = self.last_compression_n.map(|p| now.wrapping_sub(p));
        let max_gap = Self::ms_to_samples(self.settings.max_interval_ms);
        if since_compression.is_none_or(|d| d > max_gap) {
            self.run_len = 0;
            self.interval_count = 0;
        }
        let quiet = self.quiet_samples();
        let compressing = self.run_len >= self.settings.compressions_to_confirm
            && since_compression.is_some_and(|d| d <= quiet);
        let quiet_min = Self::ms_to_samples(self.settings.quiet_min_ms);
        let active = self
            .last_activity_n
            .is_some_and(|a| now.wrapping_sub(a) <= quiet_min)
            || since_compression.is_some_and(|d| d <= quiet_min);
        let next = if compressing {
            ChestState::Compressing
        } else if active {
            ChestState::Unknown
        } else {
            ChestState::HandsOff
        };
        if next == ChestState::HandsOff && self.state != ChestState::HandsOff {
            self.hands_off_since_n = Some(now);
        }
        self.state = next;
    }

    fn update_rolling(&mut self) {
        if self.state == ChestState::Compressing {
            self.second_count = self.second_count.saturating_add(1);
        }
        self.second_samples = self.second_samples.saturating_add(1);
        if u32::from(self.second_samples) >= IMPEDANCE_SAMPLE_RATE_HZ {
            if let Some(slot) = self.rolling.get_mut(self.rolling_head) {
                *slot = self.second_count;
            }
            let len = usize::from(self.settings.rolling_seconds).clamp(1, MAX_ROLLING_SECONDS);
            self.rolling_head = (self.rolling_head + 1) % len;
            self.rolling_filled = (self.rolling_filled + 1).min(len);
            self.second_count = 0;
            self.second_samples = 0;
        }
    }
}

impl ChestSensor for CompressionDetector {
    fn push(&mut self, sample: ImpedanceSample) -> Option<ChestEvent> {
        self.n = self.n.wrapping_add(1);
        let s = self.settings;
        let reading = match sample {
            ImpedanceSample::Saturated => {
                self.last_raw = None;
                self.same_count = 0;
                Err(ImpedanceSignal::Saturated)
            }
            ImpedanceSample::Milliohms(m) => {
                if self.last_raw == Some(m) {
                    self.same_count = self.same_count.saturating_add(1);
                } else {
                    self.last_raw = Some(m);
                    self.same_count = 0;
                }
                if m < s.baseline_min_mohm || m > s.baseline_max_mohm {
                    Err(ImpedanceSignal::OutOfRange)
                } else if self.same_count.saturating_mul(SAMPLE_MS) >= s.stuck_ms {
                    Err(ImpedanceSignal::Stuck)
                } else {
                    Ok(m as f32 / 1000.0)
                }
            }
        };
        let x = match reading {
            Ok(x) => x,
            Err(why) => {
                self.invalidate(why);
                self.update_rolling();
                return None;
            }
        };
        if self.x_ref.is_none() {
            self.restart_filters(x);
        }
        let v = x - self.x_ref.unwrap_or(x);
        let y = self.lp.process(self.hp2.process(self.hp1.process(v)));
        let slot = (self.n.wrapping_sub(1) as usize) % HISTORY_LEN;
        if let Some(h) = self.history.get_mut(slot) {
            *h = y;
        }
        self.filled = (self.filled + 1).min(HISTORY_LEN as u32);

        let noise = self.noise_rms(x);
        let event = self.track(y);
        let settling = self.n <= self.settle_until;
        let noisy = noise.is_some_and(|r| r * 1000.0 > s.noise_max_mohm as f32);
        self.signal = if settling {
            ImpedanceSignal::Settling
        } else if noisy {
            ImpedanceSignal::Noisy
        } else {
            ImpedanceSignal::Good
        };
        self.update_state();
        self.update_rolling();
        event
    }

    fn state(&self) -> ChestState {
        self.state
    }

    fn signal(&self) -> ImpedanceSignal {
        self.signal
    }

    fn rate_per_min(&self) -> Option<u16> {
        if self.state != ChestState::Compressing {
            return None;
        }
        let m = self.median_interval_ms()?;
        u16::try_from(60_000u32.checked_div(m)?).ok()
    }

    fn ms_since_compression(&self) -> u32 {
        let now = self.n.wrapping_sub(1);
        let samples = self
            .last_compression_n
            .map_or(self.n, |p| now.wrapping_sub(p));
        samples.saturating_mul(SAMPLE_MS)
    }

    fn time_on_chest_permille(&self) -> u16 {
        let len = self.rolling_filled.min(MAX_ROLLING_SECONDS);
        let total: u32 = self
            .rolling
            .iter()
            .take(len)
            .map(|&c| u32::from(c))
            .sum::<u32>()
            .saturating_add(u32::from(self.second_count));
        let samples = (len as u32)
            .saturating_mul(IMPEDANCE_SAMPLE_RATE_HZ)
            .saturating_add(u32::from(self.second_samples));
        let permille = total
            .saturating_mul(1000)
            .checked_div(samples)
            .unwrap_or(0)
            .min(1000);
        u16::try_from(permille).unwrap_or(1000)
    }

    fn reset(&mut self) {
        *self = Self::build(self.settings);
    }
}
