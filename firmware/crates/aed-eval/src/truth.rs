// SPDX-License-Identifier: GPL-3.0-or-later
//! Ground truth: what the annotations say the heart was doing, when, and
//! which AHA class that counts as.
//!
//! The mapping rules are listed in the crate README ("How annotations become
//! ground truth"). In short:
//!
//! * Rhythm-change annotations (`+` with text such as `(VF`) start a rhythm
//!   that lasts until the next one.
//! * In the Creighton database (cudb) VF/flutter episodes are marked by `[`
//!   and `]`; outside them the rhythm is usually not given, and that time goes
//!   to its own bucket, not to "normal rhythm".
//! * Signal-quality annotations (`~`) mark stretches where a channel is
//!   noisy or unreadable; those stretches are excluded for that channel.
//! * Around every rhythm change a guard margin is excluded, because the
//!   annotated change times are approximate.

use crate::wfdb::annot::code;
use crate::wfdb::Annotation;

/// Seconds excluded on each side of every rhythm change.
pub const TRANSITION_GUARD_S: f64 = 3.0;
/// Seconds excluded on each side of every noisy stretch.
pub const NOISE_GUARD_S: f64 = 1.0;
/// Seconds excluded on each side of an isolated artifact mark (`|`) or a
/// tape-slippage note (`TS`).
pub const ARTIFACT_GUARD_S: f64 = 1.0;
/// VF smaller than this (peak to peak, µV, measured as in [`amplitude_uv`])
/// is "fine VF", which the AHA asks only to be reported (UNVERIFIED, from the
/// research notes' reproduction of the AHA 1997 table).
pub const FINE_VF_UV: f64 = 200.0;
/// VT at or above this rate (beats per minute) is "rapid VT". The AHA leaves
/// the rate to the manufacturer; 150/min is this evaluation's choice, and it
/// matches the analyser's own 2.4 Hz lower frequency limit.
pub const RAPID_VT_BPM: f64 = 150.0;

/// Which database a record comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Db {
    /// Creighton University Ventricular Tachyarrhythmia Database.
    Cudb,
    /// MIT-BIH Malignant Ventricular Ectopy Database.
    Vfdb,
    /// MIT-BIH Arrhythmia Database.
    Mitdb,
}

impl Db {
    /// All three, in report order.
    pub const ALL: [Db; 3] = [Db::Cudb, Db::Vfdb, Db::Mitdb];

    /// Short name (also the directory name).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Db::Cudb => "cudb",
            Db::Vfdb => "vfdb",
            Db::Mitdb => "mitdb",
        }
    }

    /// From the short name.
    #[must_use]
    pub fn from_name(s: &str) -> Option<Db> {
        Db::ALL.into_iter().find(|d| d.name() == s)
    }
}

/// A rhythm label as found in the annotations, normalised.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Label {
    /// Ventricular fibrillation (`(VF`, `(VFIB`, or a cudb `[` … `]` episode).
    Vf,
    /// Ventricular flutter (`(VFL`).
    Vfl,
    /// Ventricular tachycardia (`(VT`).
    Vt,
    /// Normal sinus rhythm (`(N`, `(NSR`).
    Nsr,
    /// Asystole (`(ASYS`).
    Asystole,
    /// High-grade ventricular ectopic activity (`(HGEA`, vfdb only): its own bucket.
    Hgea,
    /// Another non-shockable rhythm; holds the canonical label text.
    Other(String),
    /// Creighton database time outside VF with no rhythm label.
    CudbUnlabelled,
    /// Annotated noise (`(NOISE`).
    Noise,
    /// Before the first rhythm label.
    Unlabelled,
    /// A label this evaluation does not recognise (kept for reporting).
    Unknown(String),
}

impl Label {
    /// Text for tables.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Label::Vf => "VF".into(),
            Label::Vfl => "VFL".into(),
            Label::Vt => "VT".into(),
            Label::Nsr => "NSR".into(),
            Label::Asystole => "ASYS".into(),
            Label::Hgea => "HGEA".into(),
            Label::Other(s) => s.clone(),
            Label::CudbUnlabelled => "cudb-unlabelled".into(),
            Label::Noise => "NOISE".into(),
            Label::Unlabelled => "unlabelled".into(),
            Label::Unknown(s) => format!("unknown:{s}"),
        }
    }

    /// Whether time with this label can be scored at all.
    #[must_use]
    pub fn scoreable(&self) -> bool {
        !matches!(self, Label::Noise | Label::Unlabelled | Label::Unknown(_))
    }

    /// Whether this is a ventricular rhythm an AED is meant to shock (before
    /// amplitude and rate are considered).
    #[must_use]
    pub fn is_ventricular_shock_candidate(&self) -> bool {
        matches!(self, Label::Vf | Label::Vfl | Label::Vt)
    }
}

/// Map the text of a rhythm annotation (`aux` of a `+`) to a label.
#[must_use]
pub fn label_from_aux(aux: &str) -> Label {
    let t = aux.trim().trim_start_matches('(').trim();
    match t {
        "VF" | "VFIB" => Label::Vf,
        "VFL" => Label::Vfl,
        "VT" => Label::Vt,
        "N" | "NSR" => Label::Nsr,
        "ASYS" => Label::Asystole,
        "HGEA" => Label::Hgea,
        "NOISE" => Label::Noise,
        "AFIB" | "AF" => Label::Other("AFIB".into()),
        "AFL" => Label::Other("AFL".into()),
        "SVTA" => Label::Other("SVTA".into()),
        "SBR" => Label::Other("SBR".into()),
        "BI" => Label::Other("BI (1st-degree block)".into()),
        "BII" => Label::Other("BII (2nd-degree block)".into()),
        "NOD" => Label::Other("NOD (junctional)".into()),
        "P" | "PM" => Label::Other("PACED".into()),
        "B" => Label::Other("B (ventricular bigeminy)".into()),
        "AB" => Label::Other("AB (atrial bigeminy)".into()),
        "T" => Label::Other("T (ventricular trigeminy)".into()),
        "IVR" => Label::Other("IVR (idioventricular)".into()),
        "VER" => Label::Other("VER (ventricular escape)".into()),
        "PREX" => Label::Other("PREX (pre-excitation)".into()),
        other => Label::Unknown(other.chars().take(16).collect()),
    }
}

/// The class a stretch of signal is scored as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TruthClass {
    /// VF at least [`FINE_VF_UV`] peak to peak. AHA goal: sensitivity ≥ 90 %.
    CoarseVf,
    /// VF below [`FINE_VF_UV`]. AHA: report only.
    FineVf,
    /// Ventricular flutter. Reported on its own (not folded into coarse VF).
    Vfl,
    /// VT with a rate from beat annotations at or above [`RAPID_VT_BPM`]. AHA goal: sensitivity ≥ 75 %.
    RapidVt,
    /// VT with a rate below [`RAPID_VT_BPM`]. AHA: report only.
    SlowVt,
    /// VT whose rate cannot be measured from the annotations (vfdb has no beat labels).
    VtRateUnknown,
    /// Normal sinus rhythm. AHA goal: specificity ≥ 99 %.
    Nsr,
    /// Asystole. AHA goal: specificity ≥ 95 %.
    Asystole,
    /// Other non-shockable rhythms. AHA goal: specificity ≥ 95 %.
    OtherNonShockable,
    /// Creighton time outside VF without a rhythm label: reported on its own
    /// (it can include VT, which the database does not label consistently).
    CudbUnlabelled,
    /// High-grade ventricular ectopic activity: reported on its own.
    Hgea,
}

impl TruthClass {
    /// All classes in report order.
    pub const ALL: [TruthClass; 11] = [
        TruthClass::CoarseVf,
        TruthClass::FineVf,
        TruthClass::Vfl,
        TruthClass::RapidVt,
        TruthClass::SlowVt,
        TruthClass::VtRateUnknown,
        TruthClass::Nsr,
        TruthClass::Asystole,
        TruthClass::OtherNonShockable,
        TruthClass::CudbUnlabelled,
        TruthClass::Hgea,
    ];

    /// Name for tables.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            TruthClass::CoarseVf => "Coarse VF",
            TruthClass::FineVf => "Fine VF (<200 µV)",
            TruthClass::Vfl => "Ventricular flutter",
            TruthClass::RapidVt => "Rapid VT (≥150/min)",
            TruthClass::SlowVt => "Slow VT (<150/min)",
            TruthClass::VtRateUnknown => "VT, rate unknown",
            TruthClass::Nsr => "Normal sinus rhythm",
            TruthClass::Asystole => "Asystole",
            TruthClass::OtherNonShockable => "Other non-shockable",
            TruthClass::CudbUnlabelled => "cudb non-VF, rhythm not given",
            TruthClass::Hgea => "HGEA (vfdb)",
        }
    }

    /// Short machine name for CSV files.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            TruthClass::CoarseVf => "coarse_vf",
            TruthClass::FineVf => "fine_vf",
            TruthClass::Vfl => "vfl",
            TruthClass::RapidVt => "rapid_vt",
            TruthClass::SlowVt => "slow_vt",
            TruthClass::VtRateUnknown => "vt_rate_unknown",
            TruthClass::Nsr => "nsr",
            TruthClass::Asystole => "asystole",
            TruthClass::OtherNonShockable => "other_nonshockable",
            TruthClass::CudbUnlabelled => "cudb_unlabelled",
            TruthClass::Hgea => "hgea",
        }
    }

    /// True if a shock is the right answer (scored by sensitivity).
    #[must_use]
    pub const fn should_shock(self) -> bool {
        matches!(
            self,
            TruthClass::CoarseVf | TruthClass::Vfl | TruthClass::RapidVt
        )
    }

    /// True if no shock is the right answer (scored by specificity).
    #[must_use]
    pub const fn should_not_shock(self) -> bool {
        matches!(
            self,
            TruthClass::Nsr | TruthClass::Asystole | TruthClass::OtherNonShockable
        )
    }

    /// True for the AHA "intermediate / report only" classes and the
    /// buckets kept apart because their truth is uncertain.
    #[must_use]
    pub const fn report_only(self) -> bool {
        !self.should_shock() && !self.should_not_shock()
    }
}

/// A stretch of one rhythm label, in seconds from the start of the record.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// Start, seconds.
    pub start: f64,
    /// End, seconds.
    pub end: f64,
    /// The label.
    pub label: Label,
}

/// Everything the annotations say about one record.
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    /// Rhythm spans covering the whole record, in order, merged where the label repeats.
    pub spans: Vec<Span>,
    /// Per channel: noisy or unreadable stretches (seconds).
    pub noise: Vec<Vec<(f64, f64)>>,
    /// Times of isolated artifact marks and tape-slippage notes (seconds).
    pub artifacts: Vec<f64>,
    /// Times of beat annotations (seconds).
    pub beats: Vec<f64>,
    /// Record length, seconds.
    pub duration: f64,
    /// Rhythm texts that were not recognised, with counts.
    pub unknown_labels: Vec<String>,
    /// Number of `~` annotations that carried text (only in cudb; ignored).
    pub noise_marks_with_text: usize,
}

fn push_span(spans: &mut Vec<Span>, start: f64, end: f64, label: Label) {
    if end <= start {
        return;
    }
    if let Some(last) = spans.last_mut() {
        if last.label == label && (last.end - start).abs() < 1e-9 {
            last.end = end;
            return;
        }
    }
    spans.push(Span { start, end, label });
}

/// Build the timeline for a record.
///
/// `fs` is the record's original sampling rate (annotation times are in its
/// samples), `n_samples` its length and `n_channels` its signal count.
#[must_use]
pub fn build_timeline(
    db: Db,
    annotations: &[Annotation],
    fs: f64,
    n_samples: u64,
    n_channels: usize,
) -> Timeline {
    let duration = n_samples as f64 / fs;
    let t = |a: &Annotation| (a.sample as f64 / fs).min(duration);
    let mut unknown_labels = Vec::new();
    let mut noise_marks_with_text = 0;

    // Rhythm changes: (time, label). For cudb, '[' and ']' also drive the state.
    let mut changes: Vec<(f64, Label)> = Vec::new();
    let initial = if db == Db::Cudb {
        Label::CudbUnlabelled
    } else {
        Label::Unlabelled
    };
    let mut in_vf = false;
    for a in annotations {
        match a.code {
            code::RHYTHM => {
                let l = label_from_aux(&a.aux);
                if let Label::Unknown(s) = &l {
                    unknown_labels.push(s.clone());
                }
                if db == Db::Cudb {
                    // The `[`/`]` marks are the reference for VF in cudb; a `(VF`
                    // label outside them is still taken as VF.
                    if !in_vf {
                        changes.push((t(a), l));
                    }
                } else {
                    changes.push((t(a), l));
                }
            }
            code::VFON if db == Db::Cudb => {
                in_vf = true;
                changes.push((t(a), Label::Vf));
            }
            code::VFOFF if db == Db::Cudb => {
                in_vf = false;
                // The rhythm after a VF episode is not given (often a shock follows).
                changes.push((t(a), Label::CudbUnlabelled));
            }
            _ => {}
        }
    }
    changes.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut spans = Vec::new();
    let mut cur_t = 0.0;
    let mut cur = initial;
    for (time, label) in changes {
        push_span(&mut spans, cur_t, time, cur.clone());
        cur_t = time;
        cur = label;
    }
    push_span(&mut spans, cur_t, duration, cur);

    // Noise per channel from '~' annotations: state changes until the next '~'.
    let mut noise: Vec<Vec<(f64, f64)>> = vec![Vec::new(); n_channels];
    let mut noisy_since: Vec<Option<f64>> = vec![None; n_channels];
    for a in annotations.iter().filter(|a| a.code == code::NOISE) {
        if !a.aux.is_empty() {
            noise_marks_with_text += 1;
        }
        let time = t(a);
        for (ch, since) in noisy_since.iter_mut().enumerate() {
            let noisy = a.subtype == -1
                || (ch < 4 && (a.subtype >> ch) & 1 == 1)
                || (ch < 4 && (a.subtype >> (ch + 4)) & 1 == 1);
            match (*since, noisy) {
                (None, true) => *since = Some(time),
                (Some(s), false) => {
                    if let Some(list) = noise.get_mut(ch) {
                        list.push((s, time));
                    }
                    *since = None;
                }
                _ => {}
            }
        }
    }
    for (ch, since) in noisy_since.iter().enumerate() {
        if let (Some(s), Some(list)) = (since, noise.get_mut(ch)) {
            list.push((*s, duration));
        }
    }
    // vfdb's "(NOISE" spans apply to every channel.
    for s in spans.iter().filter(|s| s.label == Label::Noise) {
        for list in &mut noise {
            list.push((s.start, s.end));
        }
    }

    let artifacts = annotations
        .iter()
        .filter(|a| a.code == code::ARFCT || (a.code == code::NOTE && a.aux.trim() == "TS"))
        .map(t)
        .collect();
    let beats = annotations
        .iter()
        .filter(|a| code::is_beat(a.code))
        .map(t)
        .collect();
    Timeline {
        spans,
        noise,
        artifacts,
        beats,
        duration,
        unknown_labels,
        noise_marks_with_text,
    }
}

/// Remove `holes` from a list of intervals.
#[must_use]
pub fn subtract(runs: &[(f64, f64)], holes: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = runs.to_vec();
    for &(hs, he) in holes {
        let mut next = Vec::with_capacity(out.len() + 1);
        for (s, e) in out {
            if he <= s || hs >= e {
                next.push((s, e));
                continue;
            }
            if hs > s {
                next.push((s, hs));
            }
            if he < e {
                next.push((he, e));
            }
        }
        out = next;
    }
    out.retain(|(s, e)| e > s);
    out
}

/// A stretch of one channel that can be scored: one label throughout, with
/// guard margins removed and no noise or artifact marks.
#[derive(Debug, Clone, PartialEq)]
pub struct CleanRun {
    /// Start, seconds.
    pub start: f64,
    /// End, seconds.
    pub end: f64,
    /// The label.
    pub label: Label,
}

/// The clean runs of one channel.
#[must_use]
pub fn clean_runs(tl: &Timeline, channel: usize) -> Vec<CleanRun> {
    let mut holes: Vec<(f64, f64)> = tl
        .noise
        .get(channel)
        .map(|v| {
            v.iter()
                .map(|(s, e)| (s - NOISE_GUARD_S, e + NOISE_GUARD_S))
                .collect()
        })
        .unwrap_or_default();
    holes.extend(
        tl.artifacts
            .iter()
            .map(|t| (t - ARTIFACT_GUARD_S, t + ARTIFACT_GUARD_S)),
    );
    let mut out = Vec::new();
    for span in tl.spans.iter().filter(|s| s.label.scoreable()) {
        let s = span.start + TRANSITION_GUARD_S;
        let e = span.end - TRANSITION_GUARD_S;
        if e <= s {
            continue;
        }
        for (a, b) in subtract(&[(s, e)], &holes) {
            out.push(CleanRun {
                start: a,
                end: b,
                label: span.label.clone(),
            });
        }
    }
    out
}

/// The label in force at time `t`, if any span covers it.
#[must_use]
pub fn label_at(tl: &Timeline, t: f64) -> Option<&Label> {
    tl.spans
        .iter()
        .find(|s| t >= s.start && t < s.end)
        .map(|s| &s.label)
}

/// Heart rate from beat annotations between `start` and `end` seconds, in
/// beats per minute; `None` with fewer than 3 beats.
#[must_use]
pub fn beat_rate_bpm(beats: &[f64], start: f64, end: f64) -> Option<f64> {
    let inside: Vec<f64> = beats
        .iter()
        .copied()
        .filter(|b| *b >= start && *b < end)
        .collect();
    let (Some(first), Some(last)) = (inside.first(), inside.last()) else {
        return None;
    };
    if inside.len() < 3 || last <= first {
        return None;
    }
    Some(60.0 * (inside.len() - 1) as f64 / (last - first))
}

/// Centred moving average over `2 * half + 1` samples (shorter at the ends).
fn moving_average(x: &[f64], half: usize) -> Vec<f64> {
    let mut prefix = Vec::with_capacity(x.len() + 1);
    prefix.push(0.0_f64);
    let mut acc = 0.0;
    for v in x {
        acc += *v;
        prefix.push(acc);
    }
    let n = x.len();
    (0..n)
        .map(|i| {
            let a = i.saturating_sub(half);
            let b = (i + half + 1).min(n);
            (prefix.get(b).copied().unwrap_or(0.0) - prefix.get(a).copied().unwrap_or(0.0))
                / (b - a).max(1) as f64
        })
        .collect()
}

/// Ground-truth amplitude of a stretch of signal, µV peak to peak.
///
/// Independent of the analyser: slow baseline drift is estimated with a
/// 1-second moving average applied twice (a 2-second triangular smoother,
/// which passes less than 2 % of anything at 2.5 Hz or above, so VF and
/// rapid VT are left essentially untouched) and subtracted. The result is
/// cut into 1-second pieces, each piece's largest minus smallest value is
/// taken, and the median over the pieces is returned. `None` if shorter
/// than 1 second.
#[must_use]
pub fn amplitude_uv(samples: &[i16], fs: usize) -> Option<f64> {
    if fs == 0 || samples.len() < fs {
        return None;
    }
    let x: Vec<f64> = samples.iter().map(|v| f64::from(*v)).collect();
    let drift = moving_average(&moving_average(&x, fs / 2), fs / 2);
    let detrended: Vec<f64> = x.iter().zip(drift.iter()).map(|(a, b)| a - b).collect();
    let pps: Vec<f64> = detrended
        .chunks_exact(fs)
        .map(|c| {
            let (lo, hi) = c
                .iter()
                .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
            hi - lo
        })
        .collect();
    crate::stats::median(&pps)
}

/// A rough heart-rate estimate from the signal itself, for VT where the
/// annotations give no beats (vfdb). It finds the strongest repetition
/// (autocorrelation peak) at intervals from 0.2 s to 1 s (300 to 60 per
/// minute). This is an UNVERIFIED helper, not ground truth: it can lock onto
/// half or double the true rate. `None` if nothing repeats clearly
/// (correlation below 0.3) or the stretch is under 2 seconds.
#[must_use]
pub fn estimated_rate_bpm(samples: &[i16], fs: usize) -> Option<f64> {
    if fs == 0 || samples.len() < 2 * fs {
        return None;
    }
    let n = samples.len() as f64;
    let mean = samples.iter().map(|v| f64::from(*v)).sum::<f64>() / n;
    let x: Vec<f64> = samples.iter().map(|v| f64::from(*v) - mean).collect();
    let r0: f64 = x.iter().map(|v| v * v).sum();
    if r0 <= 0.0 {
        return None;
    }
    let lo = fs / 5;
    let hi = fs;
    let mut best: Option<(usize, f64)> = None;
    for lag in lo..=hi {
        let r: f64 = x
            .iter()
            .zip(x.iter().skip(lag))
            .map(|(a, b)| a * b)
            .sum::<f64>()
            / r0;
        if best.is_none_or(|(_, br)| r > br) {
            best = Some((lag, r));
        }
    }
    let (lag, r) = best?;
    (r >= 0.3).then(|| 60.0 * fs as f64 / lag as f64)
}

/// The class of a stretch with label `label`, measured amplitude and beat rate.
/// `None` for labels that are not scored.
#[must_use]
pub fn classify_truth(
    label: &Label,
    amplitude: Option<f64>,
    rate_bpm: Option<f64>,
) -> Option<TruthClass> {
    Some(match label {
        Label::Vf => match amplitude {
            Some(a) if a >= FINE_VF_UV => TruthClass::CoarseVf,
            Some(_) => TruthClass::FineVf,
            None => return None,
        },
        Label::Vfl => TruthClass::Vfl,
        Label::Vt => match rate_bpm {
            Some(r) if r >= RAPID_VT_BPM => TruthClass::RapidVt,
            Some(_) => TruthClass::SlowVt,
            None => TruthClass::VtRateUnknown,
        },
        Label::Nsr => TruthClass::Nsr,
        Label::Asystole => TruthClass::Asystole,
        Label::Other(_) => TruthClass::OtherNonShockable,
        Label::CudbUnlabelled => TruthClass::CudbUnlabelled,
        Label::Hgea => TruthClass::Hgea,
        Label::Noise | Label::Unlabelled | Label::Unknown(_) => return None,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn ann(sample: u64, c: u8, sub: i16, aux: &str) -> Annotation {
        Annotation {
            sample,
            code: c,
            subtype: sub,
            chan: 0,
            num: 0,
            aux: aux.into(),
        }
    }

    #[test]
    fn rhythm_labels_become_spans() {
        let a = vec![
            ann(25, code::RHYTHM, 0, "(N"),
            ann(1000, code::RHYTHM, 0, "(VFL"),
            ann(2000, code::RHYTHM, 0, "(NOISE"),
            ann(2500, code::RHYTHM, 0, "(XYZ"),
        ];
        let tl = build_timeline(Db::Vfdb, &a, 250.0, 5000, 2);
        let labels: Vec<_> = tl.spans.iter().map(|s| s.label.clone()).collect();
        assert_eq!(
            labels,
            vec![
                Label::Unlabelled,
                Label::Nsr,
                Label::Vfl,
                Label::Noise,
                Label::Unknown("XYZ".into())
            ]
        );
        assert_eq!(tl.spans[2].start, 4.0);
        assert_eq!(
            tl.noise[1],
            vec![(8.0, 10.0)],
            "vfdb NOISE spans apply to every channel"
        );
        assert_eq!(tl.unknown_labels, vec!["XYZ".to_string()]);
    }

    #[test]
    fn cudb_vf_brackets_and_unlabelled_time() {
        let a = vec![
            ann(250, code::NORMAL, 0, ""),
            ann(2500, code::VFON, 0, ""),
            ann(2600, code::RHYTHM, 0, "(N"), // inside VF: does not end VF
            ann(5000, code::VFOFF, 0, ""),
            ann(7500, code::RHYTHM, 0, "(N"),
        ];
        let tl = build_timeline(Db::Cudb, &a, 250.0, 10_000, 1);
        let labels: Vec<_> = tl
            .spans
            .iter()
            .map(|s| (s.start, s.label.clone()))
            .collect();
        assert_eq!(
            labels,
            vec![
                (0.0, Label::CudbUnlabelled),
                (10.0, Label::Vf),
                (20.0, Label::CudbUnlabelled),
                (30.0, Label::Nsr)
            ]
        );
        assert_eq!(tl.beats, vec![1.0]);
    }

    #[test]
    fn noise_marks_per_channel() {
        let a = vec![
            ann(250, code::NOISE, 2, ""),  // channel 1 noisy
            ann(500, code::NOISE, -1, ""), // both unreadable
            ann(750, code::NOISE, 0, ""),  // clean
            ann(1000, code::NOISE, 1, ""), // channel 0 noisy to the end
        ];
        let tl = build_timeline(Db::Mitdb, &a, 250.0, 1250, 2);
        assert_eq!(tl.noise[0], vec![(2.0, 3.0), (4.0, 5.0)]);
        assert_eq!(tl.noise[1], vec![(1.0, 3.0)]);
    }

    #[test]
    fn clean_runs_apply_guards_and_holes() {
        let tl = Timeline {
            spans: vec![
                Span {
                    start: 0.0,
                    end: 30.0,
                    label: Label::Nsr,
                },
                Span {
                    start: 30.0,
                    end: 40.0,
                    label: Label::Vf,
                },
            ],
            noise: vec![vec![(10.0, 12.0)]],
            artifacts: vec![20.0],
            beats: vec![],
            duration: 40.0,
            unknown_labels: vec![],
            noise_marks_with_text: 0,
        };
        let runs = clean_runs(&tl, 0);
        let r: Vec<_> = runs.iter().map(|r| (r.start, r.end)).collect();
        assert_eq!(
            r,
            vec![(3.0, 9.0), (13.0, 19.0), (21.0, 27.0), (33.0, 37.0)]
        );
    }

    #[test]
    fn rate_amplitude_and_classes() {
        assert_eq!(beat_rate_bpm(&[0.0, 0.4, 0.8, 1.2], 0.0, 2.0), Some(150.0));
        assert_eq!(beat_rate_bpm(&[0.0, 0.4], 0.0, 2.0), None);
        // A 5 Hz sine of 300 µV amplitude (600 peak to peak) on a ramp.
        let s: Vec<i16> = (0..1000)
            .map(|i| {
                let t = i as f64 / 250.0;
                (300.0 * (2.0 * core::f64::consts::PI * 5.0 * t).sin() + 50.0 * t) as i16
            })
            .collect();
        let a = amplitude_uv(&s, 250).unwrap();
        assert!((a - 600.0).abs() < 18.0, "{a}");
        // And at 3.5 Hz, between the smoother's zeros.
        let s: Vec<i16> = (0..2500)
            .map(|i| (300.0 * (2.0 * core::f64::consts::PI * 3.5 * i as f64 / 250.0).sin()) as i16)
            .collect();
        let a = amplitude_uv(&s, 250).unwrap();
        assert!((a - 600.0).abs() < 18.0, "{a}");
        assert_eq!(
            classify_truth(&Label::Vf, Some(150.0), None),
            Some(TruthClass::FineVf)
        );
        assert_eq!(
            classify_truth(&Label::Vf, Some(250.0), None),
            Some(TruthClass::CoarseVf)
        );
        assert_eq!(
            classify_truth(&Label::Vt, None, None),
            Some(TruthClass::VtRateUnknown)
        );
        assert_eq!(
            classify_truth(&Label::Vt, None, Some(149.0)),
            Some(TruthClass::SlowVt)
        );
        assert_eq!(classify_truth(&Label::Noise, None, None), None);
        assert!(TruthClass::CoarseVf.should_shock());
        assert!(TruthClass::FineVf.report_only());
        assert!(TruthClass::CudbUnlabelled.report_only());
        assert!(TruthClass::Nsr.should_not_shock());
    }

    #[test]
    fn rate_estimate_from_signal() {
        // A 3 Hz oscillation is 180 per minute.
        let s: Vec<i16> = (0..2500)
            .map(|i| (500.0 * (2.0 * core::f64::consts::PI * 3.0 * i as f64 / 250.0).sin()) as i16)
            .collect();
        let r = estimated_rate_bpm(&s, 250).unwrap();
        assert!((r - 180.0).abs() < 3.0, "{r}");
        assert_eq!(estimated_rate_bpm(&[0; 1000], 250), None);
        assert_eq!(estimated_rate_bpm(&[1, 2, 3], 250), None);
    }

    #[test]
    fn subtract_intervals() {
        assert_eq!(
            subtract(&[(0.0, 10.0)], &[(2.0, 3.0), (5.0, 20.0)]),
            vec![(0.0, 2.0), (3.0, 5.0)]
        );
        assert_eq!(subtract(&[(0.0, 1.0)], &[(-1.0, 2.0)]), vec![]);
    }
}
