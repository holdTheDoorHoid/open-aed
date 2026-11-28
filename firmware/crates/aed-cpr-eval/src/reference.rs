// SPDX-License-Identifier: GPL-3.0-or-later
//! The reference: where the compressions really were, as best as can be
//! told after the fact.
//!
//! **The dataset has no compression labels.** Its paper (Menant et al.,
//! Data in Brief 2025;59:111345) found compression start and end times with
//! an algorithm, but those times are not in the published files. So this
//! module builds a reference the same way: by an algorithm, but one that is
//! deliberately different from the device's detector and has advantages
//! the device can never have:
//!
//! * it sees the whole recording at once, including the future, so it can
//!   filter without delay (forward and backward) and judge each peak by
//!   what comes after it;
//! * it uses a lower size threshold (0.15 Ω, half the device's 0.3 Ω), so
//!   that compressions too small for the device still count as
//!   compressions here and show up as misses;
//! * it only calls something compressions when there are at least four
//!   peaks in a row at a compression-like rate (50 to 240 per minute), each
//!   gap within 0.6 to 1.6 times the run's recent gaps, and ignores
//!   "shoulders" smaller than 30 % of the peaks around them.
//!
//! Its weaknesses are stated in the report: a reference built by an
//! algorithm can share blind spots with the detector (both rely on the
//! impedance signal alone), and it was checked by eye on a sample of
//! windows, not on every compression.

use crate::data::{Case, Sample, RATE_HZ};

/// The reference's settings.
#[derive(Debug, Clone, Copy)]
pub struct RefSettings {
    /// Smallest peak, ohms, measured as the drop on BOTH sides within
    /// `side_s`. Half the device's threshold.
    pub min_pp: f64,
    /// How far either side of a peak to look for its troughs, seconds.
    pub side_s: f64,
    /// Band-pass corners, Hz (second order each way, run forwards and backwards).
    pub low_hz: f64,
    /// Upper corner, Hz.
    pub high_hz: f64,
    /// Shortest and longest gap between compressions in a run, seconds.
    pub min_interval_s: f64,
    /// Longest gap, seconds.
    pub max_interval_s: f64,
    /// Fewest peaks that make a run.
    pub min_run: usize,
    /// A gap continues a run only if it is at least this fraction ...
    pub regular_low: f64,
    /// ... and at most this multiple of the median of the run's last four gaps.
    pub regular_high: f64,
    /// A peak smaller than this fraction of the median size of the peaks
    /// within 5 s is a shoulder, not a compression.
    pub relative_min: f64,
    /// Stretches of exactly zero longer than this are "no signal", seconds.
    pub zero_run_s: f64,
    /// Time excluded around clipping, missing samples and no-signal
    /// stretches, seconds.
    pub guard_s: f64,
}

impl Default for RefSettings {
    fn default() -> Self {
        RefSettings {
            min_pp: 0.15,
            side_s: 0.6,
            low_hz: 0.5,
            high_hz: 8.0,
            min_interval_s: 0.25,
            max_interval_s: 1.2,
            min_run: 4,
            regular_low: 0.6,
            regular_high: 1.6,
            relative_min: 0.3,
            zero_run_s: 0.5,
            guard_s: 2.0,
        }
    }
}

/// A run of reference compressions.
#[derive(Debug, Clone, Copy)]
pub struct Run {
    /// First sample of the run (half an interval before the first peak).
    pub start: usize,
    /// One past the last sample (half an interval after the last peak).
    pub end: usize,
    /// Number of compressions.
    pub count: usize,
    /// Median interval, seconds.
    pub median_interval_s: f64,
}

/// The reference for one case.
#[derive(Debug, Clone)]
pub struct Reference {
    /// Per sample: may this sample be scored at all?
    pub valid: Vec<bool>,
    /// Per sample: compressions under way (only meaningful where valid).
    pub compressing: Vec<bool>,
    /// Compression peaks: (sample, size in ohms).
    pub peaks: Vec<(usize, f64)>,
    /// Runs of compressions.
    pub runs: Vec<Run>,
}

/// A second-order Butterworth section in f64.
#[derive(Clone, Copy)]
struct Bq {
    b: [f64; 3],
    a: [f64; 2],
}

impl Bq {
    fn new(high: bool, hz: f64) -> Self {
        let k = (std::f64::consts::PI * hz / RATE_HZ).tan();
        let q = std::f64::consts::FRAC_1_SQRT_2;
        let norm = 1.0 / (1.0 + k / q + k * k);
        let b = if high {
            [norm, -2.0 * norm, norm]
        } else {
            let g = k * k * norm;
            [g, 2.0 * g, g]
        };
        Bq {
            b,
            a: [2.0 * (k * k - 1.0) * norm, (1.0 - k / q + k * k) * norm],
        }
    }

    fn run(&self, x: &mut [f64]) {
        let (mut z1, mut z2) = (0.0, 0.0);
        let [b0, b1, b2] = self.b;
        let [a1, a2] = self.a;
        for v in x.iter_mut() {
            let y = b0 * *v + z1;
            z1 = b1 * *v - a1 * y + z2;
            z2 = b2 * *v - a2 * y;
            *v = y;
        }
    }
}

/// Zero-phase band-pass: forwards, then backwards.
fn filtfilt(x: &mut [f64], s: &RefSettings) {
    let hp = Bq::new(true, s.low_hz);
    let lp = Bq::new(false, s.high_hz);
    if let Some(&first) = x.first() {
        for v in x.iter_mut() {
            *v -= first;
        }
    }
    hp.run(x);
    lp.run(x);
    x.reverse();
    hp.run(x);
    lp.run(x);
    x.reverse();
}

/// Which samples may be scored.
fn valid_mask(case: &Case, s: &RefSettings) -> Vec<bool> {
    let n = case.samples.len();
    let mut bad = vec![false; n];
    // Clipped or missing.
    for (i, smp) in case.samples.iter().enumerate() {
        if !matches!(smp, Sample::Ohms(_)) {
            bad[i] = true;
        }
    }
    // Long stretches of exactly zero (no signal).
    let zero_len = (s.zero_run_s * RATE_HZ) as usize;
    let mut i = 0;
    while i < n {
        if matches!(case.samples[i], Sample::Ohms(z) if z == 0.0) {
            let mut j = i;
            while j < n && matches!(case.samples[j], Sample::Ohms(z) if z == 0.0) {
                j += 1;
            }
            if j - i >= zero_len {
                for b in &mut bad[i..j] {
                    *b = true;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    // Guard band around every bad sample.
    let guard = (s.guard_s * RATE_HZ) as usize;
    let mut valid = vec![true; n];
    let mut last_bad: Option<usize> = None;
    for i in 0..n {
        if bad[i] {
            last_bad = Some(i);
        }
        if last_bad.is_some_and(|b| i - b <= guard) {
            valid[i] = false;
        }
    }
    let mut next_bad: Option<usize> = None;
    for i in (0..n).rev() {
        if bad[i] {
            next_bad = Some(i);
        }
        if next_bad.is_some_and(|b| b - i <= guard) {
            valid[i] = false;
        }
    }
    valid
}

/// Build the reference for one case.
#[must_use]
pub fn build(case: &Case, s: &RefSettings) -> Reference {
    let n = case.samples.len();
    let valid = valid_mask(case, s);
    let mut compressing = vec![false; n];
    let mut peaks_all: Vec<(usize, f64)> = Vec::new();
    let mut runs = Vec::new();
    let side = (s.side_s * RATE_HZ) as usize;
    let local = (0.15 * RATE_HZ) as usize;
    let min_gap = (s.min_interval_s * RATE_HZ) as usize;
    let max_gap = (s.max_interval_s * RATE_HZ) as usize;

    // Each stretch of valid samples separately.
    let mut i = 0;
    while i < n {
        if !valid[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && valid[i] {
            i += 1;
        }
        let end = i;
        let mut y: Vec<f64> = case.samples[start..end]
            .iter()
            .map(|smp| match smp {
                Sample::Ohms(z) => *z,
                _ => 0.0,
            })
            .collect();
        filtfilt(&mut y, s);
        let m = y.len();
        // Peaks: the highest point within ±0.15 s, with a drop of at least
        // `min_pp` on both sides within `side_s`.
        let mut peaks: Vec<(usize, f64)> = Vec::new();
        for k in 1..m.saturating_sub(1) {
            let lo = k.saturating_sub(local);
            let hi = (k + local + 1).min(m);
            let v = y[k];
            if y[lo..hi].iter().any(|&w| w > v) || y[k - 1] >= v {
                continue;
            }
            let pre = y[k.saturating_sub(side)..k]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            let post = y[k + 1..(k + side + 1).min(m)]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            if v - pre >= s.min_pp && v - post >= s.min_pp {
                // Merge peaks closer than the shortest interval: keep the larger.
                if let Some(last) = peaks.last_mut() {
                    if k - last.0 < min_gap {
                        if v - pre > last.1 {
                            *last = (k, v - pre);
                        }
                        continue;
                    }
                }
                peaks.push((k, v - pre));
            }
        }
        // Shoulders: a "peak" much smaller than the peaks around it is a
        // bump on the way down, not a compression of its own.
        let near = (5.0 * RATE_HZ) as usize;
        let sizes: Vec<f64> = peaks.iter().map(|p| p.1).collect();
        let keep: Vec<bool> = peaks
            .iter()
            .map(|&(k, size)| {
                let mut around: Vec<f64> = peaks
                    .iter()
                    .zip(&sizes)
                    .filter(|((j, _), _)| j.abs_diff(k) <= near)
                    .map(|(_, &sz)| sz)
                    .collect();
                around.sort_by(f64::total_cmp);
                let med = around[around.len() / 2];
                size >= s.relative_min * med
            })
            .collect();
        let peaks: Vec<(usize, f64)> = peaks
            .into_iter()
            .zip(keep)
            .filter_map(|(p, k)| k.then_some(p))
            .collect();
        // Runs: peaks in a row at a compression-like rate, each gap within
        // 0.6 to 1.6 times the median of the run's last few gaps (so that
        // scattered movements are not strung together into a "run").
        let mut r = 0;
        while r < peaks.len() {
            let mut q = r + 1;
            let mut recent: Vec<usize> = Vec::new();
            while q < peaks.len() {
                let gap = peaks[q].0 - peaks[q - 1].0;
                if gap > max_gap {
                    break;
                }
                if !recent.is_empty() {
                    let mut sorted = recent.clone();
                    sorted.sort_unstable();
                    let med = sorted[sorted.len() / 2] as f64;
                    let g = gap as f64;
                    if g < s.regular_low * med || g > s.regular_high * med {
                        break;
                    }
                }
                recent.push(gap);
                if recent.len() > 4 {
                    recent.remove(0);
                }
                q += 1;
            }
            if q - r >= s.min_run {
                let mut gaps: Vec<usize> =
                    (r + 1..q).map(|j| peaks[j].0 - peaks[j - 1].0).collect();
                gaps.sort_unstable();
                let med = gaps[gaps.len() / 2];
                let a = peaks[r].0.saturating_sub(med / 2);
                let b = (peaks[q - 1].0 + med / 2 + 1).min(m);
                for c in &mut compressing[start + a..start + b] {
                    *c = true;
                }
                runs.push(Run {
                    start: start + a,
                    end: start + b,
                    count: q - r,
                    median_interval_s: med as f64 / RATE_HZ,
                });
                for p in &peaks[r..q] {
                    peaks_all.push((start + p.0, p.1));
                }
            }
            r = q;
        }
    }
    Reference {
        valid,
        compressing,
        peaks: peaks_all,
        runs,
    }
}
