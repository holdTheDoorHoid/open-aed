// SPDX-License-Identifier: GPL-3.0-or-later
//! Running the analyser on recorded signals, two ways.
//!
//! * **Segment level** ([`run_segment`]): the way the device decides. A fresh
//!   analyser (as after the reset at the start of every analysis) is fed
//!   exactly one analysis worth of signal. The device outcome is the first
//!   final answer: `Shockable` means a shock, `NonShockable` means no shock,
//!   and if neither has come by the time limit the analysis is inconclusive
//!   (no shock). "Not sure" answers do not end the analysis (decision D-27).
//! * **Streaming** ([`run_stream`]): one analyser runs over the whole record
//!   without resets, giving a decision every second. This shows behaviour at
//!   rhythm changes: how long after VF starts it says "shock", and how long
//!   "shock" lasts after VF ends.

use std::path::Path;

use aed_rhythm::{Analyzer, Settings, WindowReport};
use aed_types::{RhythmAnalyzer, RhythmDecision, ECG_SAMPLE_RATE_HZ};

use crate::condition::{quantize, to_microvolts, Resampler};
use crate::truth::{
    amplitude_uv, beat_rate_bpm, build_timeline, estimated_rate_bpm, classify_truth, clean_runs, CleanRun, Db, Label,
    Timeline, TruthClass,
};
use crate::wfdb::{adc_rails, load_record, LoadError, SignalCheck};

/// Samples per second fed to the analyser.
pub const FS: usize = ECG_SAMPLE_RATE_HZ as usize;

/// Analysis lengths evaluated, in seconds: the shortest analysis that can
/// end in "shock" (1 s settling + 4 s window + 2 more windows to make 3 in a
/// row = 7 s), the 8 s analysis-time requirement mentioned in the
/// configuration, and the device's actual 12 s limit (decision D-27).
pub const VARIANTS_S: [u32; 3] = [7, 8, 12];

/// The device-realistic variant used for the headline table.
pub const DEVICE_VARIANT_S: u32 = 12;

/// The signal of one channel, ready for the analyser.
#[derive(Debug, Clone)]
pub struct Channel {
    /// Whole microvolts at 250 Hz.
    pub samples: Vec<i16>,
    /// True where the original recorder was at the end of its range (clipped).
    pub railed: Vec<bool>,
    /// Samples clipped to fit `i16` during conversion.
    pub clipped_i16: u64,
    /// Samples at the recorder's ADC limits (original rate).
    pub rail_samples: u64,
    /// Lead name from the header.
    pub lead: String,
    /// Gain used, ADC units per mV.
    pub gain: f64,
    /// True if the header gave no usable gain.
    pub gain_is_default: bool,
}

/// One record, prepared.
#[derive(Debug, Clone)]
pub struct Prepared {
    /// Database.
    pub db: Db,
    /// Record name.
    pub name: String,
    /// Original sampling rate, Hz.
    pub fs_original: f64,
    /// Samples per signal in the original file.
    pub n_original: u64,
    /// The channels.
    pub channels: Vec<Channel>,
    /// Ground truth.
    pub timeline: Timeline,
    /// Header-versus-data checks.
    pub checks: Vec<SignalCheck>,
}

/// Load and prepare one record.
///
/// # Errors
/// [`LoadError`] if any file is unreadable or malformed, or the sampling
/// rate cannot be converted to 250 Hz.
pub fn prepare(db: Db, dir: &Path, name: &str) -> Result<Prepared, LoadError> {
    let rec = load_record(dir, name, "atr")?;
    let fs = rec.header.record.sampling_hz;
    let n_original = rec.samples.first().map_or(0, |s| s.len() as u64);
    let fs_int = fs.round() as u32;
    let resampler = if (fs - f64::from(ECG_SAMPLE_RATE_HZ)).abs() < 1e-9 {
        None
    } else if (fs - f64::from(fs_int)).abs() < 1e-9 && fs_int > 0 {
        Some(
            Resampler::new_default(fs_int, ECG_SAMPLE_RATE_HZ)
                .map_err(|e| LoadError::Signal(crate::wfdb::signal::SignalError::Unsupported(format!("{e:?}"))))?,
        )
    } else {
        return Err(LoadError::Signal(crate::wfdb::signal::SignalError::Unsupported(
            format!("sampling rate {fs}"),
        )));
    };
    let mut channels = Vec::new();
    for (spec, raw) in rec.header.signals.iter().zip(rec.samples.iter()) {
        let (lo, hi) = adc_rails(spec);
        let raw_railed: Vec<bool> = raw
            .iter()
            .map(|v| i64::from(*v) <= lo || i64::from(*v) >= hi)
            .collect();
        let rail_samples = raw_railed.iter().filter(|r| **r).count() as u64;
        let uv = to_microvolts(raw, spec.gain, spec.baseline);
        let (uv, railed) = match &resampler {
            None => (uv, raw_railed),
            Some(r) => {
                let out = r.process(&uv);
                let railed = (0..out.len())
                    .map(|m| {
                        let c = r.nearest_input(m);
                        (c.saturating_sub(1)..=c + 1)
                            .any(|i| raw_railed.get(i).copied().unwrap_or(false))
                    })
                    .collect();
                (out, railed)
            }
        };
        let q = quantize(&uv);
        channels.push(Channel {
            samples: q.samples,
            railed,
            clipped_i16: q.clipped,
            rail_samples,
            lead: spec.description.clone(),
            gain: spec.gain,
            gain_is_default: spec.gain_is_default,
        });
    }
    let timeline = build_timeline(db, &rec.annotations, fs, n_original, channels.len());
    Ok(Prepared {
        db,
        name: name.to_string(),
        fs_original: fs,
        n_original,
        channels,
        timeline,
        checks: rec.checks,
    })
}

/// What the device would do at the end of an analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome {
    /// `Shockable` announced: a shock would follow.
    Shock,
    /// `NonShockable` announced first: no shock.
    NoShock,
    /// Neither by the time limit: "unable to analyse", no shock.
    Inconclusive,
}

impl Outcome {
    /// Short text.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Outcome::Shock => "shock",
            Outcome::NoShock => "no-shock",
            Outcome::Inconclusive => "inconclusive",
        }
    }
}

/// The stand-clear countdown after "shock advised", seconds (aed-core
/// configuration). During it the rescue sequence abandons the shock on any
/// answer other than `Shockable`.
pub const COUNTDOWN_S: f64 = 3.0;

/// The result of one simulated analysis.
#[derive(Debug, Clone)]
pub struct AnalysisRun {
    /// For a shock: `Some(true)` if the analyser announced anything other
    /// than `Shockable` in the [`COUNTDOWN_S`] after the shock was advised
    /// (the rescue sequence would then abandon the shock), `Some(false)` if
    /// not, `None` if the recording does not continue long enough in the same
    /// rhythm to tell.
    pub cancelled_in_countdown: Option<bool>,
    /// The device outcome.
    pub outcome: Outcome,
    /// Seconds from the start of the analysis to the final answer, if any.
    pub decided_at_s: Option<f64>,
    /// Every window the analyser looked at, in order.
    pub windows: Vec<WindowReport>,
    /// Index in `windows` of the window that gave the final answer.
    pub decisive: Option<usize>,
}

/// Feed a fresh analyser `samples` (one analysis) and apply the device's
/// rule, with the device's settings. No countdown check.
#[must_use]
pub fn run_segment(samples: &[i16]) -> AnalysisRun {
    run_segment_with(samples, &[], Settings::DEFAULT)
}

/// How each analysis starts (decision D-47).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// A fresh analyser: the first second of each analysis settles the
    /// filters and is not analysed. What the device did before D-47, and
    /// still does after the pads go on, after a shock, and whenever
    /// compression sensing does not confirm the chest still.
    Cold,
    /// The filters have been running on the signal before the analysis
    /// ([`WARM_PRE_ROLL_S`] seconds of it are passed through them first),
    /// and the window starts at the analysis start after only the short
    /// guard. What the device now does after CPR once compression sensing
    /// confirms the chest still, with the device's guard
    /// ([`Start::DEVICE_WARM`]); other guards are for measurement only.
    Warm {
        /// Samples discarded after the window restart (the device uses
        /// `aed_rhythm::settings::FRESH_WINDOW_GUARD_SAMPLES`).
        guard_samples: usize,
    },
}

impl Start {
    /// A warm start exactly as the device does it.
    pub const DEVICE_WARM: Start = Start::Warm {
        guard_samples: aed_rhythm::settings::FRESH_WINDOW_GUARD_SAMPLES,
    };
}

/// Seconds of signal before each analysis that a [`Start::Warm`] analysis
/// passes through the filters first. 2 s: twice the analyser's own full
/// settling time. Where the recording has less before the analysis, the
/// analyser itself waits for whatever is left of its settling.
pub const WARM_PRE_ROLL_S: f64 = 2.0;

/// Feed a fresh analyser with `settings` the analysis `samples` and apply
/// the device's rule. If it ends in a shock, keep feeding (the rest of
/// `samples`, then `after`, the signal that follows the analysis) for the
/// [`COUNTDOWN_S`] stand-clear countdown and record whether the analyser
/// would have cancelled the shock.
#[must_use]
pub fn run_segment_with(samples: &[i16], after: &[i16], settings: Settings) -> AnalysisRun {
    run_segment_from(&[], Start::Cold, samples, after, settings)
}

/// As [`run_segment_with`], but for a [`Start::Warm`] start with a
/// non-empty `pre`, `pre` is first passed through the analyser's filters
/// only (`RhythmAnalyzer::track`) and the analysis window is then started
/// afresh with the start's guard (decision D-47). A cold start, or an empty
/// `pre`, is exactly [`run_segment_with`].
#[must_use]
pub fn run_segment_from(pre: &[i16], start: Start, samples: &[i16], after: &[i16], settings: Settings) -> AnalysisRun {
    let mut a = Analyzer::with_settings(settings);
    if let Start::Warm { guard_samples } = start {
        if !pre.is_empty() {
            for s in pre {
                a.track(*s);
            }
            a.restart_window_with_guard(guard_samples);
        }
    }
    let mut windows: Vec<WindowReport> = Vec::new();
    let mut seen = 0_u32;
    for (i, s) in samples.iter().enumerate() {
        let d = a.push(*s);
        if let Some(r) = a.last_report() {
            if r.window_number != seen {
                seen = r.window_number;
                windows.push(*r);
            }
        }
        match d {
            Some(RhythmDecision::Shockable) => {
                let need = (COUNTDOWN_S * FS as f64).round() as usize;
                let rest = samples.get(i + 1..).unwrap_or(&[]).iter().chain(after.iter());
                let mut fed = 0_usize;
                let mut cancelled = false;
                for x in rest.take(need) {
                    fed += 1;
                    if matches!(a.push(*x), Some(d) if d != RhythmDecision::Shockable) {
                        cancelled = true;
                        break;
                    }
                }
                return AnalysisRun {
                    outcome: Outcome::Shock,
                    decided_at_s: Some((i + 1) as f64 / FS as f64),
                    decisive: windows.len().checked_sub(1),
                    windows,
                    cancelled_in_countdown: if cancelled {
                        Some(true)
                    } else if fed >= need {
                        Some(false)
                    } else {
                        None
                    },
                };
            }
            Some(RhythmDecision::NonShockable) => {
                return AnalysisRun {
                    outcome: Outcome::NoShock,
                    decided_at_s: Some((i + 1) as f64 / FS as f64),
                    decisive: windows.len().checked_sub(1),
                    windows,
                    cancelled_in_countdown: None,
                }
            }
            _ => {}
        }
    }
    AnalysisRun {
        outcome: Outcome::Inconclusive,
        decided_at_s: None,
        windows,
        decisive: None,
        cancelled_in_countdown: None,
    }
}

/// One scored analysis segment.
#[derive(Debug, Clone)]
pub struct SegmentResult {
    /// Database.
    pub db: Db,
    /// Record name.
    pub record: String,
    /// Channel number.
    pub channel: usize,
    /// Analysis length, seconds.
    pub variant_s: u32,
    /// Start time in the record, seconds.
    pub start_s: f64,
    /// Annotated rhythm.
    pub label: Label,
    /// Scored class.
    pub class: TruthClass,
    /// Ground-truth amplitude, µV peak to peak.
    pub amplitude_uv: f64,
    /// Rate from beat annotations, if any.
    pub rate_bpm: Option<f64>,
    /// Rough rate estimated from the signal (VT segments only; UNVERIFIED helper).
    pub est_rate_bpm: Option<f64>,
    /// Share of samples at the recorder's limits.
    pub railed_fraction: f64,
    /// The analysis.
    pub run: AnalysisRun,
}

/// One window of a streaming run, with the truth that applies to it.
#[derive(Debug, Clone)]
pub struct ScoredWindow {
    /// Time of the window's last sample, seconds.
    pub t_end: f64,
    /// The analyser's report.
    pub report: WindowReport,
    /// Truth class, if the last [`STREAM_TRUTH_SPAN_S`] seconds are one clean run.
    pub class: Option<TruthClass>,
    /// The label of that run.
    pub label: Option<Label>,
    /// Ground-truth amplitude of the window's 4 s, µV.
    pub amplitude_uv: f64,
}

/// How far back a streaming decision depends on the signal: the 4 s window
/// plus the two earlier windows needed for 3-in-a-row confirmation.
pub const STREAM_TRUTH_SPAN_S: f64 = 6.0;

/// Run one analyser over a whole channel without resets.
#[must_use]
pub fn run_stream(samples: &[i16], settings: Settings) -> Vec<(f64, WindowReport)> {
    let mut a = Analyzer::with_settings(settings);
    let mut out = Vec::new();
    let mut seen = 0_u32;
    for (i, s) in samples.iter().enumerate() {
        let _ = a.push(*s);
        if let Some(r) = a.last_report() {
            if r.window_number != seen {
                seen = r.window_number;
                out.push(((i + 1) as f64 / FS as f64, *r));
            }
        }
    }
    out
}

fn slice_s(samples: &[i16], start: f64, end: f64) -> &[i16] {
    let a = ((start * FS as f64).round().max(0.0) as usize).min(samples.len());
    let b = ((end * FS as f64).round().max(0.0) as usize).min(samples.len());
    samples.get(a..b.max(a)).unwrap_or(&[])
}

/// Everything computed for one record.
#[derive(Debug, Clone)]
pub struct RecordOutput {
    /// Database.
    pub db: Db,
    /// Record name.
    pub name: String,
    /// Original sampling rate.
    pub fs_original: f64,
    /// Original samples per signal.
    pub n_original: u64,
    /// Per channel: lead, gain, default-gain flag, i16 clips, rail samples.
    pub channel_info: Vec<(String, f64, bool, u64, u64)>,
    /// Header checks per channel.
    pub checks: Vec<SignalCheck>,
    /// Ground truth.
    pub timeline: Timeline,
    /// Per channel: its clean runs.
    pub runs: Vec<Vec<CleanRun>>,
    /// All segment results, every variant and channel.
    pub segments: Vec<SegmentResult>,
    /// Per channel: streaming windows.
    pub streams: Vec<Vec<ScoredWindow>>,
}

/// Run everything for one prepared record, with the analyser's `settings`
/// (the device uses `Settings::DEFAULT`). Every analysis starts cold.
#[must_use]
pub fn evaluate_record(p: &Prepared, settings: Settings) -> RecordOutput {
    evaluate_record_with(p, settings, Start::Cold)
}

/// As [`evaluate_record`], with each analysis started as `start` says.
/// For [`Start::Warm`] the pre-roll is the [`WARM_PRE_ROLL_S`] seconds of
/// the recording just before the analysis, whatever they contain (on the
/// device it would be CPR); the streaming run is the same either way.
#[must_use]
pub fn evaluate_record_with(p: &Prepared, settings: Settings, start_mode: Start) -> RecordOutput {
    let mut segments = Vec::new();
    let mut streams = Vec::new();
    let mut all_runs = Vec::new();
    for (ch, chan) in p.channels.iter().enumerate() {
        let runs = clean_runs(&p.timeline, ch);
        for variant in VARIANTS_S {
            let len = f64::from(variant);
            for run in &runs {
                let mut start = run.start;
                while start + len <= run.end {
                    let s = slice_s(&chan.samples, start, start + len);
                    if s.len() + 1 < variant as usize * FS {
                        break;
                    }
                    let amplitude = amplitude_uv(s, FS).unwrap_or(0.0);
                    let rate = beat_rate_bpm(&p.timeline.beats, start, start + len);
                    if let Some(class) = classify_truth(&run.label, Some(amplitude), rate) {
                        let a = ((start * FS as f64).round() as usize).min(chan.railed.len());
                        let b = (a + s.len()).min(chan.railed.len());
                        let railed = chan
                            .railed
                            .get(a..b)
                            .map_or(0, |r| r.iter().filter(|x| **x).count());
                        segments.push(SegmentResult {
                            db: p.db,
                            record: p.name.clone(),
                            channel: ch,
                            variant_s: variant,
                            start_s: start,
                            label: run.label.clone(),
                            class,
                            amplitude_uv: amplitude,
                            rate_bpm: rate,
                            est_rate_bpm: if run.label == Label::Vt {
                                estimated_rate_bpm(s, FS)
                            } else {
                                None
                            },
                            railed_fraction: railed as f64 / s.len().max(1) as f64,
                            run: run_segment_from(
                                match start_mode {
                                    Start::Cold => &[],
                                    Start::Warm { .. } => slice_s(&chan.samples, start - WARM_PRE_ROLL_S, start),
                                },
                                start_mode,
                                s,
                                slice_s(&chan.samples, start + len, (start + len + COUNTDOWN_S).min(run.end)),
                                settings,
                            ),
                        });
                    }
                    start += len;
                }
            }
        }
        // Streaming.
        let raw = run_stream(&chan.samples, settings);
        let mut scored = Vec::with_capacity(raw.len());
        let mut idx = 0_usize;
        for (t_end, report) in raw {
            let t0 = t_end - STREAM_TRUTH_SPAN_S;
            while runs.get(idx).is_some_and(|r| r.end < t_end) {
                idx += 1;
            }
            let run = runs.get(idx).filter(|r| r.start <= t0 && t_end <= r.end);
            let amplitude = amplitude_uv(slice_s(&chan.samples, t_end - 4.0, t_end), FS).unwrap_or(0.0);
            let (class, label) = match run {
                Some(r) => {
                    let rate = beat_rate_bpm(&p.timeline.beats, t0, t_end);
                    (classify_truth(&r.label, Some(amplitude), rate), Some(r.label.clone()))
                }
                None => (None, None),
            };
            scored.push(ScoredWindow {
                t_end,
                report,
                class,
                label,
                amplitude_uv: amplitude,
            });
        }
        streams.push(scored);
        all_runs.push(runs);
    }
    RecordOutput {
        db: p.db,
        name: p.name.clone(),
        fs_original: p.fs_original,
        n_original: p.n_original,
        channel_info: p
            .channels
            .iter()
            .map(|c| (c.lead.clone(), c.gain, c.gain_is_default, c.clipped_i16, c.rail_samples))
            .collect(),
        checks: p.checks.clone(),
        timeline: p.timeline.clone(),
        runs: all_runs,
        segments,
        streams,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn sine(hz: f64, amp: f64, secs: f64) -> Vec<i16> {
        (0..(secs * FS as f64) as usize)
            .map(|i| (amp * (2.0 * core::f64::consts::PI * hz * i as f64 / FS as f64).sin()) as i16)
            .collect()
    }

    #[test]
    fn flat_signal_ends_no_shock_at_first_window() {
        let r = run_segment(&vec![0; 12 * FS]);
        assert_eq!(r.outcome, Outcome::NoShock);
        assert_eq!(r.decided_at_s, Some(5.0), "1 s settling + 4 s window");
        assert_eq!(r.windows.len(), 1);
    }

    #[test]
    fn a_vf_like_sine_shocks_after_three_windows() {
        let r = run_segment(&sine(5.0, 500.0, 12.0));
        assert_eq!(r.outcome, Outcome::Shock, "{:?}", r.windows.last());
        assert_eq!(r.decided_at_s, Some(7.0));
    }

    #[test]
    fn a_warm_start_decides_three_quarters_of_a_second_sooner_and_without_pre_roll_is_a_cold_start() {
        let pre = sine(5.0, 500.0, 2.0);
        let seg = sine(5.0, 500.0, 12.0);
        let warm = run_segment_from(&pre, Start::DEVICE_WARM, &seg, &[], Settings::DEFAULT);
        assert_eq!(warm.outcome, Outcome::Shock);
        assert_eq!(warm.decided_at_s, Some(6.248), "62 samples of guard, not 250");
        let no_pre = run_segment_from(&[], Start::DEVICE_WARM, &seg, &[], Settings::DEFAULT);
        assert_eq!(no_pre.decided_at_s, Some(7.0));
        // A warm start with a full second's guard is a cold start.
        let slow = run_segment_from(&pre, Start::Warm { guard_samples: FS }, &seg, &[], Settings::DEFAULT);
        assert_eq!(slow.decided_at_s, Some(7.0));
    }

    #[test]
    fn countdown_is_checked_after_a_shock() {
        let vf = sine(5.0, 500.0, 12.0);
        let r = run_segment_with(&vf, &sine(5.0, 500.0, 3.0), Settings::DEFAULT);
        assert_eq!(r.outcome, Outcome::Shock);
        assert_eq!(r.cancelled_in_countdown, Some(false), "VF goes on: not cancelled");
        // A signal that turns into clipped junk right after "shock advised"
        // cancels it (a change to a flat line would not be seen this fast:
        // the 4 s window makes that blind for about 3 s, finding F-4).
        let mut stop = sine(5.0, 500.0, 7.0);
        stop.extend((0..8 * FS).map(|i| if (i / 50) % 2 == 0 { 3000 } else { -3000 }));
        let r = run_segment_with(&stop, &[], Settings::DEFAULT);
        assert_eq!(r.outcome, Outcome::Shock);
        assert_eq!(r.cancelled_in_countdown, Some(true));
        // Not enough signal after the shock to tell.
        let r = run_segment_with(&sine(5.0, 500.0, 8.0), &[], Settings::DEFAULT);
        assert_eq!(r.cancelled_in_countdown, None);
    }

    #[test]
    fn too_short_to_decide_is_inconclusive() {
        let r = run_segment(&sine(5.0, 500.0, 4.0));
        assert_eq!(r.outcome, Outcome::Inconclusive);
        assert!(r.windows.is_empty());
    }

    #[test]
    fn stream_gives_one_window_per_second() {
        let w = run_stream(&sine(5.0, 500.0, 20.0), Settings::DEFAULT);
        assert_eq!(w.len(), 16);
        assert_eq!(w[0].0, 5.0);
    }
}
