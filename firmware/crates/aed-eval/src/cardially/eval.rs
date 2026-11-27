// SPDX-License-Identifier: GPL-3.0-or-later
//! Running the analyser, with its frozen settings, on the Cardially
//! recordings, and the generated report.
//!
//! * **Before the first shock** (the main result): the 9 s that end
//!   [`PRE_GUARD_S`] before the report's "Shock 1 delivered" mark are fed to
//!   a fresh analyser exactly as one device analysis. The device can answer
//!   "shock" no earlier than 7 s (1 s settling, a 4 s window, three windows),
//!   so in 9 s it sees five windows and can decide at 7, 8 or 9 s; the
//!   device's own limit is 12 s (D-27), so this is an analysis cut short.
//!   Also run: all the signal the report holds before the shock (9.5-10.5 s),
//!   the dataset's own digitised text file, other amplitude scales, and, as
//!   an **artificial** variant only, the 9 s played twice.
//! * **After the first shock**: one analyser runs without resets from the
//!   shock to the next shock or the end of the report. The labels say only
//!   whether the shock succeeded, not what the rhythm was second by second,
//!   so this is description, not scoring.
//!
//! Nothing here changes the analyser or its settings.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use aed_rhythm::{reason_label, Features, Reason, Settings, WindowReport};
use aed_types::RhythmDecision;

use super::report_pdf::PdfEcg;
use super::txt::to_uniform;
use super::{Item, ShockOutcome};
use crate::condition::{quantize, Resampler};
use crate::eval::{run_segment_with, run_stream, AnalysisRun, Outcome, COUNTDOWN_S, FS};
use crate::stats::{clopper_pearson_lower, percentile};
use crate::truth::amplitude_uv;

/// The pre-shock segment ends this long before the "Shock 1 delivered" mark,
/// seconds, to keep the charge-and-shock moment out of it. (The dataset's own
/// text files end about 0.35 s before the mark.)
pub const PRE_GUARD_S: f64 = 0.5;
/// Length of the pre-shock segment, seconds (the dataset's 9 s).
pub const PRE_S: f64 = 9.0;
/// The device's analysis limit, seconds (D-27).
pub const DEVICE_LIMIT_S: f64 = 12.0;
/// Amplitude used for the "normalised" assumption: each recording scaled so
/// that its pre-shock median one-second peak-to-peak is this many µV.
pub const NORMALISED_UV: f64 = 1000.0;

/// An amplitude assumption.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Amp {
    /// As printed: 2 divisions (10 mm) per mV.
    AsPrinted,
    /// The printed scale multiplied by this factor.
    Times(f64),
    /// Each recording scaled to [`NORMALISED_UV`] median peak-to-peak.
    Normalised,
}

impl Amp {
    /// Short text for tables.
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Amp::AsPrinted => "as printed (10 mm/mV)".into(),
            Amp::Times(f) => format!("printed scale x {f}"),
            Amp::Normalised => format!("each normalised to {NORMALISED_UV:.0} µV p-p"),
        }
    }
}

/// The alternative amplitude assumptions run on the pre-shock segment.
pub const AMPS: [Amp; 6] = [
    Amp::AsPrinted,
    Amp::Times(0.25),
    Amp::Times(0.5),
    Amp::Times(2.0),
    Amp::Times(4.0),
    Amp::Normalised,
];

/// Convert a report's ECG (any rate, µV) to the analyser's 250 Hz whole
/// microvolts, times `factor`. Uses the crate's anti-aliasing resampler.
///
/// # Errors
/// If the resampler cannot be built for the rate.
pub fn to_analyser(uv: &[f64], fs_hz: u32, factor: f64) -> Result<Vec<i16>, String> {
    let fs_out = FS as u32;
    let x: Vec<f64> = uv.iter().map(|v| v * factor).collect();
    let y = if fs_hz == fs_out {
        x
    } else {
        Resampler::new_default(fs_hz, fs_out).map_err(|e| format!("{e:?}"))?.process(&x)
    };
    Ok(quantize(&y).samples)
}

fn slice(s: &[i16], a: f64, b: f64) -> &[i16] {
    let i = ((a * FS as f64).round().max(0.0) as usize).min(s.len());
    let j = ((b * FS as f64).round().max(0.0) as usize).min(s.len());
    s.get(i..j.max(i)).unwrap_or(&[])
}

/// A streaming run: (seconds from the run's start to the window's end, report).
pub type Stream = Vec<(f64, WindowReport)>;

/// What the analyser did with one pre-shock segment.
#[derive(Debug, Clone)]
pub struct PreRun {
    /// Length fed, seconds.
    pub length_s: f64,
    /// The device-style analysis.
    pub run: AnalysisRun,
    /// Every window over the same signal (no stop at the decision).
    pub stream: Stream,
    /// For a shock: seconds of signal left after the decision (at most 3).
    pub countdown_seen_s: Option<f64>,
    /// For a shock: whether any answer other than "shockable" was announced
    /// in those seconds.
    pub countdown_cancelled: Option<bool>,
}

fn pre_run(seg: &[i16], s: Settings) -> PreRun {
    let run = run_segment_with(seg, &[], s);
    let stream = run_stream(seg, s);
    let length_s = seg.len() as f64 / FS as f64;
    let (seen, cancelled) = match (run.outcome, run.decided_at_s) {
        (Outcome::Shock, Some(d)) => {
            let seen = (length_s - d).clamp(0.0, COUNTDOWN_S);
            let cancelled = stream
                .iter()
                .filter(|(t, _)| *t > d + 1e-9 && *t <= d + COUNTDOWN_S + 1e-9)
                .any(|(_, r)| matches!(r.announced, Some(a) if a != RhythmDecision::Shockable));
            (Some(seen), Some(cancelled))
        }
        _ => (None, None),
    };
    PreRun { length_s, run, stream, countdown_seen_s: seen, countdown_cancelled: cancelled }
}

/// Everything computed for one recording.
#[derive(Debug, Clone)]
pub struct RecordResult {
    /// `class/id`.
    pub key: String,
    /// Outcome label.
    pub outcome: ShockOutcome,
    /// Identical to this earlier recording, if so.
    pub duplicate_of: Option<String>,
    /// Why it was not evaluated, if it was not.
    pub skipped: Option<String>,
    /// Report facts.
    pub fs_hz: u32,
    /// Seconds from the report's start to "Shock 1 delivered".
    pub shock1_s: f64,
    /// Seconds from "Shock 1 delivered" to the next "Shock n delivered", if any.
    pub next_shock_after_s: Option<f64>,
    /// Pre-shock amplitude, median one-second peak-to-peak, µV (as printed).
    pub amplitude_uv: f64,
    /// The main run: 9 s before the shock, as printed.
    pub pre9: Option<PreRun>,
    /// All the report's signal before the shock.
    pub pre_all: Option<PreRun>,
    /// Device outcome under each of [`AMPS`] (9 s).
    pub by_amp: Vec<(Amp, Outcome)>,
    /// The dataset's text file, 9 s, as given in mV.
    pub txt: Option<PreRun>,
    /// Correlation and amplitude ratio (text / PDF) at the best alignment, and that alignment's end, seconds before the shock mark.
    pub txt_match: Option<(f64, f64, f64)>,
    /// ARTIFICIAL: the 9 s played twice (12 s analysis + countdown).
    pub looped: Option<AnalysisRun>,
    /// After the shock: one continuous run from the shock mark.
    pub post: Stream,
    /// Length of the post-shock part analysed, seconds.
    pub post_s: f64,
    /// Largest |value| in the first second after the shock, µV, and median one-second p-p 1-2 s after.
    pub post_transient: Option<(f64, f64)>,
    /// Device events after the shock: (seconds after the shock, text).
    pub post_events: Vec<(f64, String)>,
    /// Share of the pre-shock 9 s whose drawn trace sits on the strip box's
    /// edge (clipped by the report at ±2 mV about the strip centre).
    pub pre9_clipped: f64,
    /// The same for the first 5 s after the shock.
    pub post5_clipped: f64,
}

/// Share of samples from `a` to `b` seconds that the report clipped.
fn edge_share(p: &PdfEcg, a: f64, b: f64) -> f64 {
    let f = f64::from(p.fs_hz);
    let i = ((a * f).round().max(0.0) as usize).min(p.at_edge.len());
    let j = ((b * f).round().max(0.0) as usize).min(p.at_edge.len());
    let s = p.at_edge.get(i..j.max(i)).unwrap_or(&[]);
    if s.is_empty() {
        0.0
    } else {
        s.iter().filter(|e| **e).count() as f64 / s.len() as f64
    }
}

fn best_match(pdf: &[f64], fs: u32, txt: &[f64], txt_fs: u32, shock_s: f64) -> Option<(f64, f64, f64)> {
    // Compare at 50 per second: the text file has fewer than 150 points per second.
    let dec = |x: &[f64], f: u32| -> Vec<f64> { x.iter().step_by((f / 50).max(1) as usize).copied().collect() };
    let a = dec(pdf, fs);
    let b = dec(txt, txt_fs);
    let n = b.len();
    if n < 100 || a.len() < n {
        return None;
    }
    let mb = b.iter().sum::<f64>() / n as f64;
    let bb: Vec<f64> = b.iter().map(|v| v - mb).collect();
    let eb: f64 = bb.iter().map(|v| v * v).sum();
    let mut best: Option<(f64, f64, f64)> = None;
    for off in 0..=a.len() - n {
        let seg = a.get(off..off + n)?;
        let ma = seg.iter().sum::<f64>() / n as f64;
        let (mut num, mut ea) = (0.0, 0.0);
        for (x, y) in seg.iter().zip(bb.iter()) {
            num += (x - ma) * y;
            ea += (x - ma) * (x - ma);
        }
        if ea <= 0.0 || eb <= 0.0 {
            continue;
        }
        let r = num / (ea * eb).sqrt();
        if best.is_none_or(|(br, _, _)| r > br) {
            let end_s = (off + n) as f64 / 50.0;
            best = Some((r, (eb / ea).sqrt(), shock_s - end_s));
        }
    }
    best
}

/// Evaluate one recording with `settings` (the device uses `Settings::DEFAULT`).
#[must_use]
pub fn evaluate(item: &Item, settings: Settings) -> RecordResult {
    let mut r = RecordResult {
        key: item.key(),
        outcome: item.outcome,
        duplicate_of: item.duplicate_of.clone(),
        skipped: None,
        fs_hz: 0,
        shock1_s: f64::NAN,
        next_shock_after_s: None,
        amplitude_uv: f64::NAN,
        pre9: None,
        pre_all: None,
        by_amp: Vec::new(),
        txt: None,
        txt_match: None,
        looped: None,
        post: Vec::new(),
        post_s: 0.0,
        post_transient: None,
        post_events: Vec::new(),
        pre9_clipped: f64::NAN,
        post5_clipped: f64::NAN,
    };
    let p: &PdfEcg = match &item.pdf {
        Ok(p) => p,
        Err(e) => {
            r.skipped = Some(format!("PDF not readable: {e}"));
            return r;
        }
    };
    r.fs_hz = p.fs_hz;
    let Some(shock) = p.shock_delivered_s(1) else {
        r.skipped = Some("no 'Shock 1 delivered' in the report".into());
        return r;
    };
    r.shock1_s = shock;
    let next = (2..10).find_map(|n| p.shock_delivered_s(n)).filter(|t| *t > shock);
    r.next_shock_after_s = next.map(|t| t - shock);
    let Ok(sig) = to_analyser(&p.samples_uv, p.fs_hz, 1.0) else {
        r.skipped = Some("cannot resample".into());
        return r;
    };
    let end = shock - PRE_GUARD_S;
    if end < PRE_S {
        r.skipped = Some(format!("only {end:.2} s before the shock"));
        return r;
    }
    let seg9 = slice(&sig, end - PRE_S, end);
    r.amplitude_uv = amplitude_uv(seg9, FS).unwrap_or(f64::NAN);
    r.pre9_clipped = edge_share(p, end - PRE_S, end);
    r.post5_clipped = edge_share(p, shock, shock + 5.0);
    r.pre9 = Some(pre_run(seg9, settings));
    r.pre_all = Some(pre_run(slice(&sig, 0.0, end), settings));
    for amp in AMPS {
        let factor = match amp {
            Amp::AsPrinted => 1.0,
            Amp::Times(f) => f,
            Amp::Normalised => {
                if r.amplitude_uv > 0.0 {
                    NORMALISED_UV / r.amplitude_uv
                } else {
                    1.0
                }
            }
        };
        if let Ok(s) = to_analyser(&p.samples_uv, p.fs_hz, factor) {
            let seg = slice(&s, end - PRE_S, end);
            r.by_amp.push((amp, run_segment_with(seg, &[], settings).outcome));
        }
    }
    // The dataset's own digitisation (millivolts as given).
    if let Some(Ok(points)) = &item.txt {
        let mv = to_uniform(points, FS as u32, PRE_S);
        let uv: Vec<f64> = mv.iter().map(|v| v * 1000.0).collect();
        let q = quantize(&uv).samples;
        r.txt = Some(pre_run(&q, settings));
        let pre_uv: Vec<f64> = p.samples_uv.iter().take((shock * f64::from(p.fs_hz)) as usize).copied().collect();
        r.txt_match = best_match(&pre_uv, p.fs_hz, &uv, FS as u32, shock);
    }
    // ARTIFICIAL: the 9 s twice, analysed for the device's 12 s, then a 3 s countdown.
    let mut looped: Vec<i16> = seg9.to_vec();
    looped.extend_from_slice(seg9);
    let limit = (DEVICE_LIMIT_S * FS as f64) as usize;
    let (a, b) = looped.split_at(limit.min(looped.len()));
    r.looped = Some(run_segment_with(a, b, settings));
    // After the shock.
    let post_end = next.unwrap_or(p.duration_s()).min(p.duration_s());
    let post = slice(&sig, shock, post_end);
    r.post_s = post.len() as f64 / FS as f64;
    r.post = run_stream(post, settings);
    let first = slice(&sig, shock, shock + 1.0);
    let max_abs = first.iter().map(|v| f64::from(*v).abs()).fold(0.0, f64::max);
    let pp12 = amplitude_uv(slice(&sig, shock + 1.0, shock + 2.0), FS).unwrap_or(f64::NAN);
    if !first.is_empty() {
        r.post_transient = Some((max_abs, pp12));
    }
    r.post_events = p
        .events
        .iter()
        .filter(|e| e.t_s > shock + 1e-6 && e.t_s < post_end + 1e-6)
        .map(|e| (e.t_s - shock, e.text.clone()))
        .collect();
    r
}

// ---------------------------------------------------------------------------
// Rules
// ---------------------------------------------------------------------------

/// Every rule that keeps a window from being "shockable", by name.
#[must_use]
pub fn failed_rules(f: &Features, s: &Settings) -> Vec<&'static str> {
    let mut v = Vec::new();
    if f.saturated_fraction > s.max_saturated_fraction {
        v.push("saturated");
    }
    if f.clipped_fraction > s.max_clipped_fraction {
        v.push("clipped (flat tops)");
    }
    if f.peak_to_peak_uv > s.max_plausible_pp_uv {
        v.push("size implausible");
    }
    if f.peak_to_peak_uv < s.asystole_max_uv || (f.rms_uv < s.asystole_max_rms_uv && f.vf_leakage > s.asystole_min_leakage) {
        v.push("flatline");
    }
    if f.noise_ratio > s.max_noise_ratio {
        v.push("noise");
    }
    if f.low_band_fraction > s.max_low_band_fraction {
        v.push("low band (compression rule)");
    }
    if f.threshold_crossing_fraction <= s.tcsc_vf_min {
        v.push("TCSC");
    }
    if f.bandpass_count_fraction <= s.count_vf_min {
        v.push("band-pass count");
    }
    if f.vf_leakage >= s.leakage_vf_max {
        v.push("VF-filter leakage");
    }
    if f.dominant_frequency_hz <= s.dominant_vf_min_hz {
        v.push("dominant freq <= 2.4 Hz");
    }
    if f.dominant_frequency_hz >= s.dominant_vf_max_hz {
        v.push("dominant freq >= 9 Hz");
    }
    if f.spectral_concentration <= s.concentration_vf_min {
        v.push("concentration");
    }
    if f.peak_to_peak_uv < s.fine_vf_max_pp_uv {
        v.push("fine VF (< 200 µV)");
    }
    v
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

fn pct(k: usize, n: usize) -> String {
    if n == 0 {
        "–".into()
    } else {
        format!("{:.1} %", 100.0 * k as f64 / n as f64)
    }
}

fn with_lcl(k: usize, n: usize) -> String {
    match clopper_pearson_lower(k as u64, n as u64, 0.90) {
        Some(l) => format!("{} ({k} of {n}; lower 90 % limit {:.1} %)", pct(k, n), 100.0 * l),
        None => "no data".into(),
    }
}

fn q(v: &[f64], p: f64) -> String {
    percentile(v, p).map_or("–".into(), |x| if x.abs() >= 100.0 { format!("{x:.0}") } else { format!("{x:.3}") })
}

/// Facts about the run, for the report heading.
#[derive(Debug, Clone, Default)]
pub struct Meta {
    /// UTC date.
    pub date: String,
    /// Commit (`git rev-parse --short HEAD`) or "unknown".
    pub commit: String,
    /// Data folder.
    pub data: String,
    /// SHA-256 of the downloaded zip, if known.
    pub zip_sha256: Option<String>,
}

fn decided(o: Outcome) -> &'static str {
    o.text()
}

/// The generated Markdown report. Holds only aggregate figures: no
/// recording's signal, and no per-recording rows (those go to the CSV).
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_report(items: &[Item], res: &[RecordResult], unpaired: &(Vec<String>, Vec<String>), meta: &Meta) -> String {
    let s = Settings::DEFAULT;
    let mut o = String::new();
    let _ = writeln!(o, "# Rhythm analyser on the Cardially out-of-hospital VF recordings (generated)\n");
    let _ = writeln!(o, "- **Generated:** {} by `aed-cardially`, commit {}, analyser settings `Settings::DEFAULT` (frozen, untuned on these data).", meta.date, meta.commit);
    let _ = writeln!(o, "- **Data:** `{}`{}.", meta.data, meta.zip_sha256.as_ref().map_or(String::new(), |h| format!(" (zip SHA-256 `{h}`)")));
    let _ = writeln!(o, "- DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE. Aggregates only; per-recording results are in the CSV, which is not committed.\n");

    // ---- Reading
    let read_ok = items.iter().filter(|i| i.pdf.is_ok()).count();
    let dups = items.iter().filter(|i| i.duplicate_of.is_some()).count();
    let _ = writeln!(o, "## 1. Reading the files\n");
    let _ = writeln!(o, "| | count |\n|---|---|");
    let _ = writeln!(o, "| PDF reports found | {} |", items.len());
    for oc in ShockOutcome::ALL {
        let _ = writeln!(o, "| … labelled {} | {} |", oc.dir(), items.iter().filter(|i| i.outcome == oc).count());
    }
    let _ = writeln!(o, "| PDF reports read | {read_ok} |");
    let _ = writeln!(o, "| identical ECG to an earlier report (duplicates, left out) | {dups} |");
    let fs: BTreeMap<u32, usize> = items.iter().filter_map(|i| i.pdf.as_ref().ok()).fold(BTreeMap::new(), |mut m, p| {
        *m.entry(p.fs_hz).or_default() += 1;
        m
    });
    let _ = writeln!(o, "| sampling rate of the drawn trace | {} |", fs.iter().map(|(k, v)| format!("{k}/s: {v}")).collect::<Vec<_>>().join(", "));
    let strips: BTreeMap<usize, usize> = items.iter().filter_map(|i| i.pdf.as_ref().ok()).fold(BTreeMap::new(), |mut m, p| {
        *m.entry(p.strips).or_default() += 1;
        m
    });
    let _ = writeln!(o, "| 10 s strips per report | {} |", strips.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", "));
    let scales: BTreeMap<String, usize> = items.iter().filter_map(|i| i.pdf.as_ref().ok()).fold(BTreeMap::new(), |mut m, p| {
        *m.entry(format!("{} div/mV, {} div/s, div = {:.2} pt", p.div_per_mv, p.div_per_s, p.div_pt)).or_default() += 1;
        m
    });
    let _ = writeln!(o, "| printed scale | {} |", scales.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join("; "));
    let (miss, off, conf, brk): (usize, usize, usize, usize) = items.iter().filter_map(|i| i.pdf.as_ref().ok()).fold((0, 0, 0, 0), |a, p| {
        (a.0 + p.missing, a.1 + p.off_grid, a.2 + p.conflicts, a.3 + p.clock_breaks)
    });
    let _ = writeln!(o, "| samples missing / vertices off the sample grid / conflicting / clock breaks between strips (all reports) | {miss} / {off} / {conf} / {brk} |");
    let txt_ok = items.iter().filter(|i| matches!(i.txt, Some(Ok(_)))).count();
    let _ = writeln!(o, "| text files paired by name and read | {txt_ok} |");
    let _ = writeln!(o, "| text files with no PDF of the same name / PDFs with no text file | {} / {} |\n", unpaired.0.len(), unpaired.1.len());
    for i in items.iter().filter(|i| i.pdf.is_err()) {
        let _ = writeln!(o, "- PDF not read: {} ({})", i.key(), i.pdf.as_ref().err().map_or("", String::as_str));
    }
    let skipped: Vec<&RecordResult> = res.iter().filter(|r| r.duplicate_of.is_none() && r.skipped.is_some()).collect();
    for r in &skipped {
        let _ = writeln!(o, "- Not evaluated: {} ({})", r.key, r.skipped.as_deref().unwrap_or(""));
    }
    let ev: Vec<&RecordResult> = res.iter().filter(|r| r.duplicate_of.is_none() && r.skipped.is_none()).collect();
    let _ = writeln!(o, "\n**Evaluated: {} distinct recordings** (labelled ROEA {}, noROEA {}, indeterminable {}).\n", ev.len(),
        ev.iter().filter(|r| r.outcome == ShockOutcome::Roea).count(),
        ev.iter().filter(|r| r.outcome == ShockOutcome::NoRoea).count(),
        ev.iter().filter(|r| r.outcome == ShockOutcome::Indeterminable).count());

    let shock1: Vec<f64> = ev.iter().map(|r| r.shock1_s).collect();
    let amps: Vec<f64> = ev.iter().map(|r| r.amplitude_uv).collect();
    let _ = writeln!(o, "Shock 1 mark, seconds from the report's start: min {}, median {}, max {}. Post-shock signal analysed (to the next shock or the report's end): median {} s, min {} s. Recordings with a second shock inside the report: {}.\n",
        q(&shock1, 0.0), q(&shock1, 50.0), q(&shock1, 100.0),
        q(&ev.iter().map(|r| r.post_s).collect::<Vec<_>>(), 50.0), q(&ev.iter().map(|r| r.post_s).collect::<Vec<_>>(), 0.0),
        ev.iter().filter(|r| r.next_shock_after_s.is_some()).count());
    let fine = amps.iter().filter(|a| **a < 200.0).count();
    let _ = writeln!(o, "Pre-shock VF amplitude as printed (median one-second peak-to-peak, drift removed; the measure used for PhysioNet): 5 % {}, 25 % {}, median {}, 75 % {}, 95 % {} µV. Below 200 µV (AHA \"fine VF\"): {fine} recordings.\n",
        q(&amps, 5.0), q(&amps, 25.0), q(&amps, 50.0), q(&amps, 75.0), q(&amps, 95.0));
    let clip_uv: Vec<f64> = items.iter().filter_map(|i| i.pdf.as_ref().ok()).map(|p| p.clip_uv).collect();
    let pc: Vec<f64> = ev.iter().map(|r| r.pre9_clipped).collect();
    let _ = writeln!(o, "**The report clips the drawn trace** at the edges of each strip's box, ±{} µV about the strip's centre line at the printed scale (all reports). Share of the pre-shock 9 s clipped this way: none in {} recordings, more than 1 % in {}, more than 5 % in {}; largest {:.1} %. The clipped samples are flat in the report but no longer exactly flat after conversion to 250/s, so the analyser's flat-top check cannot see them.\n",
        q(&clip_uv, 50.0), pc.iter().filter(|x| **x == 0.0).count(), pc.iter().filter(|x| **x > 0.01).count(), pc.iter().filter(|x| **x > 0.05).count(), 100.0 * pc.iter().copied().fold(0.0, f64::max));
    let m: Vec<&(f64, f64, f64)> = ev.iter().filter_map(|r| r.txt_match.as_ref()).collect();
    if !m.is_empty() {
        let rr: Vec<f64> = m.iter().map(|x| x.0).collect();
        let ratio: Vec<f64> = m.iter().map(|x| x.1).collect();
        let endb: Vec<f64> = m.iter().map(|x| x.2).collect();
        let _ = writeln!(o, "Text file versus PDF trace ({} pairs; best alignment at 50/s): correlation 5 % {}, median {}; text/PDF amplitude ratio 5 % {}, median {}, 95 % {}; text ends this many seconds before the shock mark: 5 % {}, median {}, 95 % {}.\n",
            m.len(), q(&rr, 5.0), q(&rr, 50.0), q(&ratio, 5.0), q(&ratio, 50.0), q(&ratio, 95.0), q(&endb, 5.0), q(&endb, 50.0), q(&endb, 95.0));
        let bad_scale = m.iter().filter(|x| x.0 > 0.85 && !(0.7..1.3).contains(&x.1)).count();
        let no_match = m.iter().filter(|x| x.0 <= 0.85).count();
        let _ = writeln!(o, "Text files at a clearly different amplitude from their PDF (ratio outside 0.7–1.3): {bad_scale}; text files that do not match their PDF (correlation ≤ 0.85): {no_match}.\n");
    }

    // ---- Pre-shock
    let _ = writeln!(o, "## 2. Before the first shock (all recordings are VF by the dataset's definition)\n");
    let _ = writeln!(o, "Device-style analysis: a fresh analyser fed the segment; outcome is its first final answer; no final answer by the end of the segment = \"inconclusive\". The earliest possible \"shock\" is at 7 s.\n");
    let _ = writeln!(o, "| Segment | n | shock | no shock | inconclusive | shock decided at 7 / 8 / 9 / later s |\n|---|---|---|---|---|---|");
    let row = |o: &mut String, name: &str, runs: Vec<&PreRun>| {
        let n = runs.len();
        let k = |x: Outcome| runs.iter().filter(|p| p.run.outcome == x).count();
        let at = |a: f64, b: f64| runs.iter().filter(|p| p.run.outcome == Outcome::Shock && p.run.decided_at_s.is_some_and(|d| d > a && d <= b)).count();
        let _ = writeln!(o, "| {name} | {n} | {} | {} | {} | {} / {} / {} / {} |", with_lcl(k(Outcome::Shock), n), pct(k(Outcome::NoShock), n), pct(k(Outcome::Inconclusive), n),
            at(6.5, 7.5), at(7.5, 8.5), at(8.5, 9.5), at(9.5, 100.0));
    };
    row(&mut o, "**9 s before the shock, as printed (headline)**", ev.iter().filter_map(|r| r.pre9.as_ref()).collect());
    for oc in ShockOutcome::ALL {
        row(&mut o, &format!("… labelled {}", oc.dir()), ev.iter().filter(|r| r.outcome == oc).filter_map(|r| r.pre9.as_ref()).collect());
    }
    row(&mut o, "… amplitude ≥ 200 µV (coarse VF)", ev.iter().filter(|r| r.amplitude_uv >= 200.0).filter_map(|r| r.pre9.as_ref()).collect());
    row(&mut o, "all signal before the shock (9.5–10.5 s)", ev.iter().filter_map(|r| r.pre_all.as_ref()).collect());
    row(&mut o, "the dataset's digitised text file (9 s, mV as given)", ev.iter().filter_map(|r| r.txt.as_ref()).collect());
    let lp: Vec<&AnalysisRun> = ev.iter().filter_map(|r| r.looped.as_ref()).collect();
    let lk = lp.iter().filter(|a| a.outcome == Outcome::Shock).count();
    let lcan = lp.iter().filter(|a| a.cancelled_in_countdown == Some(true)).count();
    let _ = writeln!(o, "\n*Artificial variant, not a result:* the 9 s played twice, as one 12 s analysis: shock {} ; of those, {lcan} cancelled in the 3 s countdown. The join puts a jump in the signal 9 s in, which the analyser's filters and the 4 s windows straddling it see; windows 10–12 s and the countdown re-use the same VF, so this overstates what more real signal would show.\n", with_lcl(lk, lp.len()));

    // Window verdicts
    let wins: Vec<&WindowReport> = ev.iter().filter_map(|r| r.pre9.as_ref()).flat_map(|p| p.stream.iter().map(|(_, w)| w)).collect();
    let nw = wins.len();
    let ws = wins.iter().filter(|w| w.verdict.decision == RhythmDecision::Shockable).count();
    let wn = wins.iter().filter(|w| w.verdict.decision == RhythmDecision::NonShockable).count();
    let _ = writeln!(o, "**Shockable window verdicts** (each 4 s window on its own, before confirmation; 5 windows per 9 s segment): {} shockable, {} not shockable, {} not sure, of {nw}.\n", pct(ws, nw), pct(wn, nw), pct(nw - ws - wn, nw));
    let per_rec: Vec<usize> = ev.iter().filter_map(|r| r.pre9.as_ref()).map(|p| p.stream.iter().filter(|(_, w)| w.verdict.decision == RhythmDecision::Shockable).count()).collect();
    let _ = writeln!(o, "| shockable windows of 5 | 0 | 1 | 2 | 3 | 4 | 5 |\n|---|---|---|---|---|---|---|");
    let _ = writeln!(o, "| recordings | {} |\n", (0..=5).map(|k| per_rec.iter().filter(|x| **x == k).count().to_string()).collect::<Vec<_>>().join(" | "));
    let mut reasons: BTreeMap<&str, usize> = BTreeMap::new();
    for w in &wins {
        *reasons.entry(reason_label(w.verdict.reason)).or_default() += 1;
    }
    let _ = writeln!(o, "Window verdict reasons: {}.\n", reasons.iter().map(|(k, v)| format!("{k} {}", pct(*v, nw))).collect::<Vec<_>>().join("; "));

    // Amplitude sensitivity
    let _ = writeln!(o, "### Amplitude assumption (9 s segment)\n\n| Assumption | shock | no shock | inconclusive |\n|---|---|---|---|");
    for amp in AMPS {
        let outs: Vec<Outcome> = ev.iter().filter_map(|r| r.by_amp.iter().find(|(a, _)| *a == amp).map(|(_, o)| *o)).collect();
        let n = outs.len();
        let k = |x: Outcome| outs.iter().filter(|o| **o == x).count();
        let _ = writeln!(o, "| {} | {} | {} | {} |", amp.text(), with_lcl(k(Outcome::Shock), n), pct(k(Outcome::NoShock), n), pct(k(Outcome::Inconclusive), n));
    }

    // Countdown
    let _ = writeln!(o, "\n### The countdown (finding F-11)\n");
    for (name, runs) in [
        ("9 s segment", ev.iter().filter_map(|r| r.pre9.as_ref()).collect::<Vec<_>>()),
        ("all signal before the shock", ev.iter().filter_map(|r| r.pre_all.as_ref()).collect::<Vec<_>>()),
    ] {
        let sh: Vec<&&PreRun> = runs.iter().filter(|p| p.run.outcome == Outcome::Shock).collect();
        let none_seen = sh.iter().filter(|p| p.countdown_seen_s.is_some_and(|s| s < 0.5)).count();
        let full = sh.iter().filter(|p| p.countdown_seen_s.is_some_and(|s| s >= COUNTDOWN_S - 1e-9)).count();
        let canc = sh.iter().filter(|p| p.countdown_cancelled == Some(true)).count();
        let seen: Vec<&&&PreRun> = sh.iter().filter(|p| p.countdown_seen_s.is_some_and(|s| s >= 0.5)).collect();
        let full_canc = sh.iter().filter(|p| p.countdown_seen_s.is_some_and(|s| s >= COUNTDOWN_S - 1e-9) && p.countdown_cancelled == Some(true)).count();
        let _ = writeln!(o, "- **{name}:** {} shock decisions. Signal left after the decision: none for {none_seen}, the full 3 s for {full}. Of the {} with at least one more window to see, {canc} had an answer other than \"shockable\" in the remaining time (would have cancelled the shock); since most saw only 1–2 of the 3 countdown windows, this is a lower bound. Of the {full} with the whole countdown visible, {full_canc} would have been cancelled ({}).", sh.len(), seen.len(), pct(full_canc, full));
    }

    // Features
    let _ = writeln!(o, "\n### Per-window features on these VF recordings versus the thresholds\n");
    let _ = writeln!(o, "All windows of the 9 s segments, as printed ({nw} windows, {} recordings). \"VF side\" = share of windows on the shockable side of that rule.\n", ev.len());
    let _ = writeln!(o, "| Feature | threshold (VF side) | 5 % | 25 % | median | 75 % | 95 % | VF side |\n|---|---|---|---|---|---|---|---|");
    type Get = fn(&Features) -> f64;
    type Ok = fn(&Features, &Settings) -> bool;
    let feats: [(&str, String, Get, Ok); 12] = [
        ("peak-to-peak, after filter (µV)", format!("≥ {} and ≤ {}", s.fine_vf_max_pp_uv, s.max_plausible_pp_uv), |f| f64::from(f.peak_to_peak_uv), |f, s| f.peak_to_peak_uv >= s.fine_vf_max_pp_uv && f.peak_to_peak_uv <= s.max_plausible_pp_uv),
        ("root-mean-square (µV)", format!("≥ {} (or leakage ≤ {})", s.asystole_max_rms_uv, s.asystole_min_leakage), |f| f64::from(f.rms_uv), |f, s| !(f.rms_uv < s.asystole_max_rms_uv && f.vf_leakage > s.asystole_min_leakage)),
        ("noise ratio", format!("≤ {}", s.max_noise_ratio), |f| f64::from(f.noise_ratio), |f, s| f.noise_ratio <= s.max_noise_ratio),
        ("low-band fraction", format!("≤ {}", s.max_low_band_fraction), |f| f64::from(f.low_band_fraction), |f, s| f.low_band_fraction <= s.max_low_band_fraction),
        ("TCSC", format!("> {}", s.tcsc_vf_min), |f| f64::from(f.threshold_crossing_fraction), |f, s| f.threshold_crossing_fraction > s.tcsc_vf_min),
        ("band-pass count", format!("> {}", s.count_vf_min), |f| f64::from(f.bandpass_count_fraction), |f, s| f.bandpass_count_fraction > s.count_vf_min),
        ("VF-filter leakage", format!("< {}", s.leakage_vf_max), |f| f64::from(f.vf_leakage), |f, s| f.vf_leakage < s.leakage_vf_max),
        ("dominant frequency (Hz)", format!("> {} and < {}", s.dominant_vf_min_hz, s.dominant_vf_max_hz), |f| f64::from(f.dominant_frequency_hz), |f, s| f.dominant_frequency_hz > s.dominant_vf_min_hz && f.dominant_frequency_hz < s.dominant_vf_max_hz),
        ("spectral concentration", format!("> {}", s.concentration_vf_min), |f| f64::from(f.spectral_concentration), |f, s| f.spectral_concentration > s.concentration_vf_min),
        ("clipped fraction (flat tops)", format!("≤ {}", s.max_clipped_fraction), |f| f64::from(f.clipped_fraction), |f, s| f.clipped_fraction <= s.max_clipped_fraction),
        ("saturated fraction", format!("≤ {}", s.max_saturated_fraction), |f| f64::from(f.saturated_fraction), |f, s| f.saturated_fraction <= s.max_saturated_fraction),
        ("1-s peak-to-peak median (µV)", "(not a rule)".into(), |f| f64::from(f.second_pp_median_uv), |_, _| true),
    ];
    for (name, th, get, ok) in &feats {
        let v: Vec<f64> = wins.iter().map(|w| get(&w.features)).collect();
        let k = wins.iter().filter(|w| ok(&w.features, &s)).count();
        let _ = writeln!(o, "| {name} | {th} | {} | {} | {} | {} | {} | {} |", q(&v, 5.0), q(&v, 25.0), q(&v, 50.0), q(&v, 75.0), q(&v, 95.0), pct(k, nw));
    }
    let not_shock: Vec<&&WindowReport> = wins.iter().filter(|w| w.verdict.decision != RhythmDecision::Shockable).collect();
    let mut any: BTreeMap<&str, usize> = BTreeMap::new();
    let mut only: BTreeMap<&str, usize> = BTreeMap::new();
    for w in &not_shock {
        let f = failed_rules(&w.features, &s);
        for r in &f {
            *any.entry(r).or_default() += 1;
        }
        if let [one] = f.as_slice() {
            *only.entry(one).or_default() += 1;
        }
    }
    let mut anyv: Vec<(&str, usize)> = any.into_iter().collect();
    anyv.sort_by_key(|a| std::cmp::Reverse(a.1));
    let _ = writeln!(o, "\nRules failed by the {} windows that were not shockable (a window can fail several):\n\n| Rule | windows failing it | windows where it is the only failure |\n|---|---|---|", not_shock.len());
    for (r, k) in anyv {
        let _ = writeln!(o, "| {r} | {} | {} |", pct(k, not_shock.len()), pct(only.get(r).copied().unwrap_or(0), not_shock.len()));
    }

    // ---- Post-shock
    let _ = writeln!(o, "\n## 3. After the first shock (description, not scoring)\n");
    let _ = writeln!(o, "One analyser run without resets from the \"Shock 1 delivered\" mark to the next shock or the report's end. The labels are about the shock's outcome, not second-by-second rhythm (and \"noROEA\" includes asystole and slow rhythms, not only continuing VF), so nothing here is a sensitivity or specificity.\n");
    let bins: [(f64, f64); 5] = [(0.0, 10.0), (10.0, 20.0), (20.0, 30.0), (30.0, 45.0), (45.0, 61.0)];
    let _ = writeln!(o, "Share of window verdicts that were **shockable** / **not shockable** / not sure, by seconds after the shock (window end time):\n");
    let _ = writeln!(o, "| Label | n | {} |\n|---|---|{}", bins.iter().map(|(a, b)| format!("{a:.0}–{b:.0} s")).collect::<Vec<_>>().join(" | "), "---|".repeat(bins.len()));
    for oc in ShockOutcome::ALL {
        let rs: Vec<&&RecordResult> = ev.iter().filter(|r| r.outcome == oc).collect();
        let cells: Vec<String> = bins
            .iter()
            .map(|(a, b)| {
                let w: Vec<&WindowReport> = rs.iter().flat_map(|r| r.post.iter()).filter(|(t, _)| *t > *a && *t <= *b).map(|(_, w)| w).collect();
                let k = |d: RhythmDecision| w.iter().filter(|x| x.verdict.decision == d).count();
                format!("{} / {} / {} (n {})", pct(k(RhythmDecision::Shockable), w.len()), pct(k(RhythmDecision::NonShockable), w.len()), pct(k(RhythmDecision::Indeterminate), w.len()), w.len())
            })
            .collect();
        let _ = writeln!(o, "| {} | {} | {} |", oc.dir(), rs.len(), cells.join(" | "));
    }
    let _ = writeln!(o, "\nRecordings where the continuous run **confirmed \"shockable\"** at least once after the shock, and where it said \"not shockable\" at least once:\n\n| Label | n | confirmed shockable | not shockable at least once | Philips device itself delivered another shock in the report |\n|---|---|---|---|---|");
    for oc in ShockOutcome::ALL {
        let rs: Vec<&&RecordResult> = ev.iter().filter(|r| r.outcome == oc).collect();
        let conf = rs.iter().filter(|r| r.post.iter().any(|(_, w)| w.announced == Some(RhythmDecision::Shockable))).count();
        let ns = rs.iter().filter(|r| r.post.iter().any(|(_, w)| w.announced == Some(RhythmDecision::NonShockable))).count();
        let sh2 = rs.iter().filter(|r| r.next_shock_after_s.is_some()).count();
        let _ = writeln!(o, "| {} | {} | {} | {} | {} |", oc.dir(), rs.len(), pct(conf, rs.len()), pct(ns, rs.len()), pct(sh2, rs.len()));
    }
    let mut post_reasons: BTreeMap<&str, usize> = BTreeMap::new();
    let pw: Vec<&WindowReport> = ev.iter().flat_map(|r| r.post.iter().map(|(_, w)| w)).collect();
    for w in &pw {
        *post_reasons.entry(reason_label(w.verdict.reason)).or_default() += 1;
    }
    let _ = writeln!(o, "\nPost-shock window reasons (all labels, {} windows): {}.\n", pw.len(), post_reasons.iter().map(|(k, v)| format!("{k} {}", pct(*v, pw.len()))).collect::<Vec<_>>().join("; "));
    let pause = ev.iter().filter(|r| r.post_events.iter().any(|(_, t)| t.contains("Start pause"))).count();
    let pause_t: Vec<f64> = ev.iter().filter_map(|r| r.post_events.iter().find(|(_, t)| t.contains("Start pause")).map(|(t, _)| *t)).collect();
    let _ = writeln!(o, "The Philips device printed \"Start pause\" (its CPR pause) after the shock in {} recordings, median {} s after the shock; CPR movement is therefore likely in much of the post-shock signal, which the analyser's compression rule is meant to notice.\n", pause, q(&pause_t, 50.0));

    // Transient
    let first: Vec<(Option<Reason>, f64)> = ev
        .iter()
        .filter_map(|r| r.post.first().map(|(t, w)| (Some(w.verdict.reason), *t)))
        .collect();
    let mut fr: BTreeMap<&str, usize> = BTreeMap::new();
    for (rsn, _) in &first {
        if let Some(x) = rsn {
            *fr.entry(reason_label(*x)).or_default() += 1;
        }
    }
    let tr: Vec<(f64, f64)> = ev.iter().filter_map(|r| r.post_transient).collect();
    let maxa: Vec<f64> = tr.iter().map(|x| x.0).collect();
    let pp12: Vec<f64> = tr.iter().map(|x| x.1).collect();
    let p5: Vec<f64> = ev.iter().map(|r| r.post5_clipped).collect();
    let _ = writeln!(o, "**Shock transient.** Share of the first 5 s after the shock clipped by the report: median {:.1} %, 75 % {:.1} %, 95 % {:.1} %; recordings with any: {}.", 100.0 * percentile(&p5, 50.0).unwrap_or(f64::NAN), 100.0 * percentile(&p5, 75.0).unwrap_or(f64::NAN), 100.0 * percentile(&p5, 95.0).unwrap_or(f64::NAN), p5.iter().filter(|x| **x > 0.0).count());
    let _ = writeln!(o, "Largest |value| in the first second after the mark (after conversion to 250/s, so including the resampler's ringing at clipped edges): median {} µV, 95 % {} µV, max {} µV (pre-shock amplitude median {} µV). One-second peak-to-peak 1–2 s after: median {} µV. The first window after the shock (covering 1–5 s after it) was judged: {}. Clipping, size and saturation rules in all post-shock windows: clipped {}, size implausible {}, saturated {}.\n",
        q(&maxa, 50.0), q(&maxa, 95.0), q(&maxa, 100.0), q(&amps, 50.0), q(&pp12, 50.0),
        fr.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(", "),
        pw.iter().filter(|w| w.verdict.reason == Reason::Clipped).count(),
        pw.iter().filter(|w| w.verdict.reason == Reason::AmplitudeImplausible).count(),
        pw.iter().filter(|w| w.verdict.reason == Reason::Saturated).count());

    // Agreement with the Philips device's own post-shock decisions.
    let mut agree: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for r in &ev {
        for (t, text) in &r.post_events {
            let dev = if text.starts_with("Shock advised") {
                "Philips: shock advised"
            } else if text.starts_with("No shock advised") {
                "Philips: no shock advised"
            } else {
                continue;
            };
            // Our window ending at or just before the device's announcement.
            let ours = r.post.iter().rev().find(|(tw, _)| *tw <= *t + 1e-6).map_or("no window yet", |(_, w)| match w.verdict.decision {
                RhythmDecision::Shockable => "shockable",
                RhythmDecision::NonShockable => "not shockable",
                RhythmDecision::Indeterminate => "not sure",
            });
            *agree.entry((dev, ours)).or_default() += 1;
        }
    }
    let _ = writeln!(o, "**The Philips device's own post-shock decisions** (its labels inside the report) against this analyser's window verdict at the same moment:\n\n| Philips said | shockable | not shockable | not sure | no window yet |\n|---|---|---|---|---|");
    for dev in ["Philips: shock advised", "Philips: no shock advised"] {
        let c = |x: &str| agree.get(&(dev, x)).copied().unwrap_or(0);
        let _ = writeln!(o, "| {dev} | {} | {} | {} | {} |", c("shockable"), c("not shockable"), c("not sure"), c("no window yet"));
    }
    let _ = writeln!(o, "\n(The Philips analysis uses its own, longer, look at the signal; its announcement time is when it finished, so the comparison is approximate.)\n");
    let _ = writeln!(o, "## How it was run\n\n- Trace recovered from the PDF vector drawing at the rate it is drawn, scaled by the printed \"2 div/mV\", converted to 250/s with the crate's anti-aliasing resampler (`condition::Resampler`), rounded to whole µV.\n- Pre-shock segment: the 9 s ending {PRE_GUARD_S} s before the \"Shock 1 delivered\" mark. Device rule as in `eval::run_segment_with` (D-27; 3 s countdown).\n- Confidence limits: exact (Clopper–Pearson) one-sided 90 % lower limits, treating recordings as independent. Duplicated reports are counted once.\n- Analyser: `Settings::DEFAULT` as committed. Not tuned on these data.");
    let _ = decided(Outcome::Shock);
    o
}

/// Per-recording CSV (header + one row per report, duplicates included and marked).
#[must_use]
pub fn records_csv(res: &[RecordResult]) -> String {
    let mut o = String::from("record,label,duplicate_of,skipped,shock1_s,amplitude_uv,pre9_outcome,pre9_decided_s,pre9_shockable_windows,pre9_reasons,pre9_countdown_seen_s,pre9_countdown_cancelled,preall_s,preall_outcome,txt_outcome,txt_corr,txt_amp_ratio,amp_x0.25,amp_x0.5,amp_x2,amp_x4,amp_norm,looped_outcome,looped_cancelled,pre9_clipped,post5_clipped,post_s,post_shockable_share,post_confirmed_shockable,post_events\n");
    for r in res {
        let p9 = r.pre9.as_ref();
        let amp = |a: Amp| r.by_amp.iter().find(|(x, _)| *x == a).map_or("", |(_, o)| o.text());
        let reasons = p9.map_or(String::new(), |p| p.stream.iter().map(|(_, w)| reason_label(w.verdict.reason)).collect::<Vec<_>>().join("|"));
        let post_sh = if r.post.is_empty() {
            String::new()
        } else {
            format!("{:.3}", r.post.iter().filter(|(_, w)| w.verdict.decision == RhythmDecision::Shockable).count() as f64 / r.post.len() as f64)
        };
        let _ = writeln!(
            o,
            "{},{},{},{},{:.3},{:.1},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{:.2},{},{},{}",
            r.key,
            r.outcome.dir(),
            r.duplicate_of.as_deref().unwrap_or(""),
            r.skipped.as_deref().unwrap_or("").replace(',', ";"),
            r.shock1_s,
            r.amplitude_uv,
            p9.map_or("", |p| p.run.outcome.text()),
            p9.and_then(|p| p.run.decided_at_s).map_or(String::new(), |d| format!("{d:.1}")),
            p9.map_or(String::new(), |p| p.stream.iter().filter(|(_, w)| w.verdict.decision == RhythmDecision::Shockable).count().to_string()),
            reasons,
            p9.and_then(|p| p.countdown_seen_s).map_or(String::new(), |d| format!("{d:.1}")),
            p9.and_then(|p| p.countdown_cancelled).map_or(String::new(), |c| c.to_string()),
            r.pre_all.as_ref().map_or(String::new(), |p| format!("{:.2}", p.length_s)),
            r.pre_all.as_ref().map_or("", |p| p.run.outcome.text()),
            r.txt.as_ref().map_or("", |p| p.run.outcome.text()),
            r.txt_match.map_or(String::new(), |m| format!("{:.3}", m.0)),
            r.txt_match.map_or(String::new(), |m| format!("{:.3}", m.1)),
            amp(Amp::Times(0.25)),
            amp(Amp::Times(0.5)),
            amp(Amp::Times(2.0)),
            amp(Amp::Times(4.0)),
            amp(Amp::Normalised),
            r.looped.as_ref().map_or("", |a| a.outcome.text()),
            r.looped.as_ref().and_then(|a| a.cancelled_in_countdown).map_or(String::new(), |c| c.to_string()),
            r.pre9_clipped,
            r.post5_clipped,
            r.post_s,
            post_sh,
            r.post.iter().any(|(_, w)| w.announced == Some(RhythmDecision::Shockable)),
            r.post_events.iter().map(|(t, e)| format!("{t:.1}s {e}")).collect::<Vec<_>>().join("|").replace(',', ";"),
        );
    }
    o
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::cardially::report_pdf::Event;
    use aed_rhythm::synth::{Rhythm, Synth};

    fn fake_item(pre_rhythm: Rhythm, post_rhythm: Rhythm, shock_s: f64) -> Item {
        // Build a 200 Hz record by drawing 250 Hz synthetic signal and resampling, to exercise the path.
        let mut a = Synth::new(pre_rhythm, 3);
        let mut b = Synth::new(post_rhythm, 4);
        let n250 = (60.0 * 250.0) as usize;
        let cut = (shock_s * 250.0) as usize;
        let x: Vec<f64> = (0..n250).map(|i| f64::from(if i < cut { a.next_sample() } else { b.next_sample() })).collect();
        let r = Resampler::new_default(250, 200).unwrap();
        let samples_uv = r.process(&x);
        Item {
            outcome: ShockOutcome::Roea,
            id: "test".into(),
            pdf: Ok(PdfEcg {
                at_edge: vec![false; samples_uv.len()],
                clip_uv: 2000.0,
                samples_uv,
                fs_hz: 200,
                strips: 6,
                pages: 2,
                strip_s: 10,
                div_per_mv: 2.0,
                div_per_s: 5.0,
                div_pt: 14.17,
                missing: 0,
                off_grid: 0,
                conflicts: 0,
                clock_breaks: 0,
                events: vec![
                    Event { t_s: shock_s, text: "Shock 1 delivered".into() },
                    Event { t_s: shock_s + 3.0, text: "Start pause".into() },
                ],
            }),
            txt_name: None,
            txt: None,
            duplicate_of: None,
        }
    }

    #[test]
    fn synthetic_vf_then_normal_rhythm() {
        let it = fake_item(Rhythm::coarse_vf(), Rhythm::sinus(75.0), 10.5);
        let r = evaluate(&it, Settings::DEFAULT);
        assert!(r.skipped.is_none(), "{:?}", r.skipped);
        let p = r.pre9.as_ref().unwrap();
        assert!((p.length_s - 9.0).abs() < 0.01);
        assert_eq!(p.stream.len(), 5, "windows at 5, 6, 7, 8, 9 s");
        assert_eq!(p.run.outcome, Outcome::Shock);
        assert!(p.run.decided_at_s.unwrap() >= 7.0 - 1e-9);
        assert!(r.pre_all.as_ref().unwrap().length_s > 9.9);
        assert_eq!(r.by_amp.len(), AMPS.len());
        // After the shock, normal rhythm: never confirmed shockable.
        assert!(r.post.iter().all(|(_, w)| w.announced != Some(RhythmDecision::Shockable)));
        assert!(r.post_s > 49.0);
        assert_eq!(r.post_events.len(), 1);
        let csv = records_csv(std::slice::from_ref(&r));
        assert_eq!(csv.lines().count(), 2);
        assert_eq!(csv.lines().next().unwrap().split(',').count(), csv.lines().nth(1).unwrap().split(',').count());
        let rep = build_report(&[it], &[r], &(Vec::new(), Vec::new()), &Meta::default());
        assert!(rep.contains("**Evaluated: 1 distinct recordings**"));
    }

    #[test]
    fn too_little_signal_before_the_shock_is_skipped_not_guessed() {
        let it = fake_item(Rhythm::coarse_vf(), Rhythm::sinus(75.0), 8.0);
        let r = evaluate(&it, Settings::DEFAULT);
        assert!(r.skipped.as_deref().unwrap().contains("before the shock"));
        assert!(r.pre9.is_none());
    }

    #[test]
    fn failed_rules_name_each_rule() {
        let f = Features { peak_to_peak_uv: 150.0, low_band_fraction: 0.9, dominant_frequency_hz: 2.0, ..Features::default() };
        let v = failed_rules(&f, &Settings::DEFAULT);
        assert!(v.contains(&"fine VF (< 200 µV)"));
        assert!(v.contains(&"low band (compression rule)"));
        assert!(v.contains(&"dominant freq <= 2.4 Hz"));
        assert!(!v.contains(&"flatline"));
    }
}
