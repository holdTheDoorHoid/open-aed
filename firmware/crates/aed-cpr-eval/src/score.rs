// SPDX-License-Identifier: GPL-3.0-or-later
//! Runs the device's detector over a recording the way the device would
//! (sample by sample, no look-ahead) and compares it with the reference.

use aed_cpr::{CompressionDetector, Settings};
use aed_types::{ChestEvent, ChestSensor, ChestState, ImpedanceSample};

use crate::data::{Case, Sample, RATE_HZ};
use crate::reference::Reference;

/// The baseline added to the recordings' change-only impedance so that the
/// detector sees a whole resistance, as on the device. 80 Ω: a typical
/// adult (the recordings do not say; any value in 25 to 250 Ω gives the
/// same result, because the detector removes the baseline).
pub const BASELINE_OHMS: f64 = 80.0;

/// Allowance after a reference run starts before a "hands off" answer
/// counts as a miss, seconds. A compression's rise must be seen first; the
/// onset latencies themselves are reported separately.
pub const ONSET_ALLOWANCE_S: f64 = 1.0;
/// Allowance after a reference run ends before a "compressing" or
/// "unknown" answer counts against the detector, seconds (the quiet period
/// plus margin); the hands-off latencies are reported separately.
pub const OFFSET_ALLOWANCE_S: f64 = 2.5;
/// A reference compression counts as found if the detector reported a
/// compression from this long before its peak ...
pub const MATCH_BEFORE_S: f64 = 0.15;
/// ... to this long after it (the detector reports a peak once the signal
/// has fallen from it).
pub const MATCH_AFTER_S: f64 = 0.5;

/// What the detector did on one case.
#[derive(Debug, Clone)]
pub struct DetectorRun {
    /// State after each sample.
    pub states: Vec<ChestState>,
    /// Compression events: (sample at which reported, size in ohms).
    pub events: Vec<(usize, f64)>,
    /// Reported rate after each sample (0 if none).
    pub rates: Vec<u16>,
}

/// Run the detector over a case.
#[must_use]
pub fn run_detector(case: &Case, settings: Settings) -> Option<DetectorRun> {
    let mut det = CompressionDetector::new(settings).ok()?;
    let n = case.samples.len();
    let mut states = Vec::with_capacity(n);
    let mut rates = Vec::with_capacity(n);
    let mut events = Vec::new();
    for (i, s) in case.samples.iter().enumerate() {
        let sample = match s {
            Sample::Ohms(z) => {
                ImpedanceSample::Milliohms(((BASELINE_OHMS + z) * 1000.0).round().max(0.0) as u32)
            }
            Sample::Clipped | Sample::Missing => ImpedanceSample::Saturated,
        };
        if let Some(ChestEvent::Compression { amplitude_mohm, .. }) = det.push(sample) {
            events.push((i, f64::from(amplitude_mohm) / 1000.0));
        }
        states.push(det.state());
        rates.push(det.rate_per_min().unwrap_or(0));
    }
    Some(DetectorRun {
        states,
        events,
        rates,
    })
}

/// Results for one case.
#[derive(Debug, Clone, Default)]
pub struct CaseScore {
    /// Case name.
    pub name: String,
    /// Seconds that could be scored.
    pub valid_s: f64,
    /// Seconds of reference compressions (after the onset allowance).
    pub ref_comp_s: f64,
    /// ... during which the detector said hands off: MISSED.
    pub missed_s: f64,
    /// ... during which it said compressing.
    pub comp_as_comp_s: f64,
    /// ... during which it said unknown.
    pub comp_as_unknown_s: f64,
    /// Longest single stretch of "hands off" inside reference compressions, seconds.
    pub longest_miss_s: f64,
    /// Seconds of reference hands-off (after the offset allowance).
    pub ref_off_s: f64,
    /// ... during which the detector said compressing.
    pub off_as_comp_s: f64,
    /// ... during which it said unknown.
    pub off_as_unknown_s: f64,
    /// Reference compressions, found by the detector, and their sizes.
    pub ref_peaks: usize,
    /// Reference compressions matched by a detector compression.
    pub ref_peaks_found: usize,
    /// Detector compressions.
    pub det_events: usize,
    /// Detector compressions matching a reference compression.
    pub det_events_matched: usize,
    /// Rate errors, per minute, in 5 s windows inside reference runs.
    pub rate_errors: Vec<f64>,
    /// Onset latencies (run start to "not hands off"), seconds.
    pub onset_s: Vec<f64>,
    /// Onset latencies to "compressing", seconds.
    pub onset_comp_s: Vec<f64>,
    /// Offset latencies (run end to "hands off"), seconds.
    pub offset_s: Vec<f64>,
    /// Reference peak sizes, ohms.
    pub peak_sizes: Vec<f64>,
    /// Analysis markers checked, and the answer at each.
    pub analyses: Vec<(f64, ChestState)>,
}

fn s(n: usize) -> f64 {
    n as f64 / RATE_HZ
}

/// Score one case.
#[must_use]
pub fn score(case: &Case, reference: &Reference, det: &DetectorRun) -> CaseScore {
    let n = case.samples.len();
    let mut out = CaseScore {
        name: case.name.clone(),
        ..CaseScore::default()
    };
    let onset = (ONSET_ALLOWANCE_S * RATE_HZ) as usize;
    let offset = (OFFSET_ALLOWANCE_S * RATE_HZ) as usize;
    // Distance from the start of the current reference run / since the end of the last.
    let mut in_run_for = 0usize;
    let mut off_for = usize::MAX / 2;
    let mut miss_len = 0usize;
    let mut longest_miss = 0usize;
    for i in 0..n {
        let valid = reference.valid[i];
        let comp = reference.compressing[i];
        if comp {
            in_run_for += 1;
            off_for = 0;
        } else {
            in_run_for = 0;
            off_for = off_for.saturating_add(1);
        }
        if !valid {
            miss_len = 0;
            continue;
        }
        out.valid_s += 1.0;
        let st = det.states[i];
        if comp && in_run_for > onset {
            out.ref_comp_s += 1.0;
            match st {
                ChestState::HandsOff => {
                    out.missed_s += 1.0;
                    miss_len += 1;
                    longest_miss = longest_miss.max(miss_len);
                }
                ChestState::Compressing => {
                    out.comp_as_comp_s += 1.0;
                    miss_len = 0;
                }
                ChestState::Unknown => {
                    out.comp_as_unknown_s += 1.0;
                    miss_len = 0;
                }
            }
        } else {
            miss_len = 0;
        }
        if !comp && off_for > offset {
            out.ref_off_s += 1.0;
            match st {
                ChestState::Compressing => out.off_as_comp_s += 1.0,
                ChestState::Unknown => out.off_as_unknown_s += 1.0,
                ChestState::HandsOff => {}
            }
        }
    }
    for v in [
        &mut out.valid_s,
        &mut out.ref_comp_s,
        &mut out.missed_s,
        &mut out.comp_as_comp_s,
        &mut out.comp_as_unknown_s,
        &mut out.ref_off_s,
        &mut out.off_as_comp_s,
        &mut out.off_as_unknown_s,
    ] {
        *v /= RATE_HZ;
    }
    out.longest_miss_s = s(longest_miss);

    // Per compression.
    let before = (MATCH_BEFORE_S * RATE_HZ) as usize;
    let after = (MATCH_AFTER_S * RATE_HZ) as usize;
    out.ref_peaks = reference.peaks.len();
    out.det_events = det.events.len();
    let mut used = vec![false; det.events.len()];
    let mut j0 = 0usize;
    for &(p, size) in &reference.peaks {
        out.peak_sizes.push(size);
        while j0 < det.events.len() && det.events[j0].0 + before < p {
            j0 += 1;
        }
        let mut j = j0;
        while j < det.events.len() && det.events[j].0 <= p + after {
            if !used[j] && det.events[j].0 + before >= p {
                used[j] = true;
                out.ref_peaks_found += 1;
                break;
            }
            j += 1;
        }
    }
    // Detector events that fall inside a reference run (near a reference
    // peak) are matched; the rest are extra.
    out.det_events_matched = used.iter().filter(|&&u| u).count();

    // Rate error in 5 s windows inside each reference run.
    let win = (5.0 * RATE_HZ) as usize;
    for run in &reference.runs {
        let mut a = run.start + onset + (2.0 * RATE_HZ) as usize;
        while a + win <= run.end {
            let peaks: Vec<usize> = reference
                .peaks
                .iter()
                .map(|&(p, _)| p)
                .filter(|&p| p >= a && p < a + win)
                .collect();
            if peaks.len() >= 3 && (a..a + win).all(|k| reference.valid[k]) {
                let mut gaps: Vec<usize> = peaks.windows(2).map(|w| w[1] - w[0]).collect();
                gaps.sort_unstable();
                let ref_rate = 60.0 * RATE_HZ / gaps[gaps.len() / 2] as f64;
                let det_rate = f64::from(det.rates[a + win - 1]);
                if det_rate > 0.0 {
                    out.rate_errors.push(det_rate - ref_rate);
                }
            }
            a += win;
        }
    }

    // Latencies at run boundaries with at least 3 s of scored hands-off
    // before (onset) or after (offset).
    let quiet = (3.0 * RATE_HZ) as usize;
    for (k, run) in reference.runs.iter().enumerate() {
        let prev_end = if k == 0 { 0 } else { reference.runs[k - 1].end };
        let clean_before = run.start >= prev_end + quiet
            && (run.start - quiet..run.start).all(|x| reference.valid[x])
            && reference.valid[run.start];
        if clean_before && det.states[run.start - 1] == ChestState::HandsOff {
            if let Some(d) = (run.start..run.end).find(|&x| det.states[x] != ChestState::HandsOff) {
                out.onset_s.push(s(d - run.start));
            }
            if let Some(d) = (run.start..run.end).find(|&x| det.states[x] == ChestState::Compressing) {
                out.onset_comp_s.push(s(d - run.start));
            }
        }
        let next_start = reference.runs.get(k + 1).map_or(n, |r| r.start);
        let clean_after = next_start >= run.end + quiet
            && (run.end..run.end + quiet).all(|x| x < n && reference.valid[x]);
        if clean_after {
            if let Some(d) = (run.end..next_start).find(|&x| det.states[x] == ChestState::HandsOff) {
                out.offset_s.push(s(d - run.end));
            }
        }
    }

    // The device's own analysis markers: hands should be off then.
    for &t in &case.analyses_s {
        let i = (t * RATE_HZ).round() as usize;
        if i < n && reference.valid[i] {
            out.analyses.push((t, det.states[i]));
        }
    }
    out
}

/// Percentile of a list (nearest rank); `None` if empty.
#[must_use]
pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let rank = ((p / 100.0) * (v.len() - 1) as f64).round() as usize;
    v.get(rank.min(v.len() - 1)).copied()
}
