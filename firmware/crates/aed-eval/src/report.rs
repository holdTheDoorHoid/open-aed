// SPDX-License-Identifier: GPL-3.0-or-later
//! Turning results into the Markdown report and CSV files.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use aed_rhythm::{reason_label, Features, Reason, Settings, WindowReport};
use aed_types::RhythmDecision;

use crate::eval::{Outcome, RecordOutput, ScoredWindow, SegmentResult, COUNTDOWN_S, DEVICE_VARIANT_S, VARIANTS_S};
use crate::stats::{clopper_pearson_lower, median, percentile, wilson_lower};
use crate::truth::{
    Db, Label, TruthClass, ARTIFACT_GUARD_S, FINE_VF_UV, NOISE_GUARD_S, RAPID_VT_BPM,
    TRANSITION_GUARD_S,
};

/// Confidence level used throughout (AHA 1997: one-sided 90 %).
pub const LEVEL: f64 = 0.90;

/// The AHA 1997 goal for a class, as reproduced in the project's research
/// notes (UNVERIFIED against the original paper): (goal, 90 % lower limit
/// listed, minimum test samples).
#[must_use]
pub fn aha_goal(class: TruthClass) -> Option<(f64, f64, u64)> {
    match class {
        TruthClass::CoarseVf => Some((0.90, 0.87, 200)),
        TruthClass::RapidVt => Some((0.75, 0.67, 50)),
        TruthClass::Nsr => Some((0.99, 0.97, 100)),
        TruthClass::OtherNonShockable => Some((0.95, 0.88, 30)),
        TruthClass::Asystole => Some((0.95, 0.92, 100)),
        _ => None,
    }
}

/// Minimum samples the AHA table asks for in the report-only classes (UNVERIFIED).
#[must_use]
pub fn aha_report_only_min(class: TruthClass) -> Option<u64> {
    match class {
        TruthClass::FineVf | TruthClass::SlowVt => Some(25),
        _ => None,
    }
}

/// Counts of outcomes for a group of segments.
#[derive(Debug, Clone, Default)]
pub struct Tally {
    /// Segments.
    pub n: u64,
    /// Ending in a shock.
    pub shock: u64,
    /// Ending in "no shock".
    pub no_shock: u64,
    /// Ending inconclusive.
    pub inconclusive: u64,
    /// Records contributing.
    pub records: BTreeSet<String>,
}

impl Tally {
    fn add(&mut self, s: &SegmentResult) {
        self.n += 1;
        match s.run.outcome {
            Outcome::Shock => self.shock += 1,
            Outcome::NoShock => self.no_shock += 1,
            Outcome::Inconclusive => self.inconclusive += 1,
        }
        self.records.insert(format!("{}/{}", s.db.name(), s.record));
    }

    /// Successes for the class's metric: shocks for shockable classes,
    /// no-shock plus inconclusive otherwise.
    #[must_use]
    pub fn successes(&self, class: TruthClass) -> u64 {
        if class.should_shock() || class.report_only() {
            self.shock
        } else {
            self.no_shock + self.inconclusive
        }
    }
}

fn pct(x: u64, n: u64) -> String {
    if n == 0 {
        "–".into()
    } else {
        format!("{:.1}%", 100.0 * x as f64 / n as f64)
    }
}

fn pctf(x: f64) -> String {
    format!("{:.1}%", 100.0 * x)
}

fn opt_pct(x: Option<f64>) -> String {
    x.map_or_else(|| "–".into(), pctf)
}

/// Verdict text against an AHA goal.
fn verdict(class: TruthClass, t: &Tally) -> String {
    let Some((goal, lcl_goal, min_n)) = aha_goal(class) else {
        return "report only".into();
    };
    if t.n == 0 {
        return "no data".into();
    }
    let x = t.successes(class);
    let p = x as f64 / t.n as f64;
    let lcl = clopper_pearson_lower(x, t.n, LEVEL).unwrap_or(0.0);
    let ok = p >= goal && lcl >= lcl_goal;
    let mut s = if ok { "meets".to_string() } else { "**FAILS**".to_string() };
    if t.n < min_n {
        let _ = write!(s, " (n<{min_n})");
    }
    s
}

/// One group of segments to tabulate.
struct Group<'a> {
    title: String,
    segs: Vec<&'a SegmentResult>,
}

fn class_table(out: &mut String, g: &Group<'_>) {
    let mut by: BTreeMap<TruthClass, Tally> = BTreeMap::new();
    let mut total = Tally::default();
    for s in &g.segs {
        by.entry(s.class).or_default().add(s);
        total.add(s);
    }
    let _ = writeln!(out, "**{}** ({} segments from {} records)\n", g.title, total.n, total.records.len());
    let _ = writeln!(
        out,
        "| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for class in TruthClass::ALL {
        let Some(t) = by.get(&class) else { continue };
        let x = t.successes(class);
        let metric = if class.should_shock() {
            "sensitivity"
        } else if class.should_not_shock() {
            "specificity"
        } else {
            "shock rate"
        };
        let (lcl, wl) = if class.report_only() {
            ("–".to_string(), "–".to_string())
        } else {
            (
                opt_pct(clopper_pearson_lower(x, t.n, LEVEL)),
                opt_pct(wilson_lower(x, t.n, LEVEL)),
            )
        };
        let goal = aha_goal(class).map_or_else(
            || "report only".to_string(),
            |(gl, l, _)| format!("≥{:.0}% ({:.0}%)", gl * 100.0, l * 100.0),
        );
        let min_n = aha_goal(class)
            .map(|(_, _, n)| n)
            .or_else(|| aha_report_only_min(class))
            .map_or_else(|| "–".to_string(), |n| n.to_string());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            class.name(),
            metric,
            t.n,
            t.records.len(),
            t.shock,
            t.no_shock,
            t.inconclusive,
            pct(x, t.n),
            lcl,
            wl,
            goal,
            min_n,
            verdict(class, t)
        );
    }
    // VF regardless of amplitude, and VT regardless of rate.
    for (name, classes, shock_metric) in [
        ("All VF, any amplitude", &[TruthClass::CoarseVf, TruthClass::FineVf][..], true),
        (
            "All VT, no rate criterion",
            &[TruthClass::RapidVt, TruthClass::SlowVt, TruthClass::VtRateUnknown][..],
            true,
        ),
    ] {
        let mut t = Tally::default();
        for s in g.segs.iter().filter(|s| classes.contains(&s.class)) {
            t.add(s);
        }
        if t.n > 0 && shock_metric {
            let _ = writeln!(
                out,
                "| *{}* | sensitivity | {} | {} | {} | {} | {} | {} | {} | {} | – | – | – |",
                name,
                t.n,
                t.records.len(),
                t.shock,
                t.no_shock,
                t.inconclusive,
                pct(t.shock, t.n),
                opt_pct(clopper_pearson_lower(t.shock, t.n, LEVEL)),
                opt_pct(wilson_lower(t.shock, t.n, LEVEL)),
            );
        }
    }
    let _ = writeln!(
        out,
        "\nInconclusive (\"unable to analyse\") over all {} segments: **{}**.\n",
        total.n,
        pct(total.inconclusive, total.n)
    );
}

fn other_breakdown(out: &mut String, segs: &[&SegmentResult]) {
    let mut by: BTreeMap<String, Tally> = BTreeMap::new();
    for s in segs.iter().filter(|s| {
        matches!(
            s.class,
            TruthClass::OtherNonShockable | TruthClass::Nsr | TruthClass::Asystole
        )
    }) {
        by.entry(s.label.text()).or_default().add(s);
    }
    let _ = writeln!(out, "| Rhythm label | n | records | shock (false positive) | no shock | inconcl. | specificity | 90% LCL exact |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|");
    for (label, t) in &by {
        let x = t.no_shock + t.inconclusive;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            label,
            t.n,
            t.records.len(),
            t.shock,
            t.no_shock,
            t.inconclusive,
            pct(x, t.n),
            opt_pct(clopper_pearson_lower(x, t.n, LEVEL))
        );
    }
    let _ = writeln!(out);
}

/// A test on one window's measurements, made stricter by a fraction.
pub type ConditionTest = Box<dyn Fn(&Features, f32) -> bool>;

/// One analyser condition that must hold for a window to be called shockable.
pub struct Condition {
    /// Short name.
    pub key: &'static str,
    /// The rule as a reader sees it.
    pub rule: String,
    /// True if the window passes (is on the VF side), with the threshold made
    /// stricter by the fraction `k` (0 = the current setting).
    pub passes: ConditionTest,
}

/// The conditions, in the order `classify` applies them, for the given settings.
#[must_use]
pub fn conditions(st: &Settings) -> Vec<Condition> {
    let s = *st;
    vec![
        Condition {
            key: "not saturated",
            rule: format!("saturated fraction ≤ {}", s.max_saturated_fraction),
            passes: Box::new(move |f, _| f.saturated_fraction <= s.max_saturated_fraction),
        },
        Condition {
            key: "not clipped (flat tops)",
            rule: format!("clipped fraction ≤ {}", s.max_clipped_fraction),
            passes: Box::new(move |f, _| f.clipped_fraction <= s.max_clipped_fraction),
        },
        Condition {
            key: "plausible size",
            rule: format!("peak-to-peak ≤ {} µV", s.max_plausible_pp_uv),
            passes: Box::new(move |f, _| f.peak_to_peak_uv <= s.max_plausible_pp_uv),
        },
        Condition {
            key: "not flatline",
            rule: format!(
                "peak-to-peak ≥ {} µV, and not (rms < {} µV with leakage > {})",
                s.asystole_max_uv, s.asystole_max_rms_uv, s.asystole_min_leakage
            ),
            passes: Box::new(move |f, k| {
                f.peak_to_peak_uv >= s.asystole_max_uv * (1.0 + k)
                    && !(f.rms_uv < s.asystole_max_rms_uv * (1.0 + k) && f.vf_leakage > s.asystole_min_leakage)
            }),
        },
        Condition {
            key: "not noisy",
            rule: format!("noise ratio ≤ {}", s.max_noise_ratio),
            passes: Box::new(move |f, k| f.noise_ratio <= s.max_noise_ratio * (1.0 - k)),
        },
        Condition {
            key: "not low-band (compression)",
            rule: format!("low-band fraction ≤ {}", s.max_low_band_fraction),
            passes: Box::new(move |f, k| f.low_band_fraction <= s.max_low_band_fraction * (1.0 - k)),
        },
        Condition {
            key: "TCSC",
            rule: format!("threshold-crossing fraction > {}", s.tcsc_vf_min),
            passes: Box::new(move |f, k| f.threshold_crossing_fraction > s.tcsc_vf_min * (1.0 + k)),
        },
        Condition {
            key: "band-pass count",
            rule: format!("band-pass count > {}", s.count_vf_min),
            passes: Box::new(move |f, k| f.bandpass_count_fraction > s.count_vf_min * (1.0 + k)),
        },
        Condition {
            key: "VF-filter leakage",
            rule: format!("leakage < {}", s.leakage_vf_max),
            passes: Box::new(move |f, k| f.vf_leakage < s.leakage_vf_max * (1.0 - k)),
        },
        Condition {
            key: "dominant freq. low limit",
            rule: format!("dominant frequency > {} Hz", s.dominant_vf_min_hz),
            passes: Box::new(move |f, k| f.dominant_frequency_hz > s.dominant_vf_min_hz * (1.0 + k)),
        },
        Condition {
            key: "dominant freq. high limit",
            rule: format!("dominant frequency < {} Hz", s.dominant_vf_max_hz),
            passes: Box::new(move |f, k| f.dominant_frequency_hz < s.dominant_vf_max_hz * (1.0 - k)),
        },
        Condition {
            key: "spectral concentration",
            rule: format!("concentration > {}", s.concentration_vf_min),
            passes: Box::new(move |f, k| f.spectral_concentration > s.concentration_vf_min * (1.0 + k)),
        },
        Condition {
            key: "not fine VF",
            rule: format!("peak-to-peak ≥ {} µV", s.fine_vf_max_pp_uv),
            passes: Box::new(move |f, k| f.peak_to_peak_uv >= s.fine_vf_max_pp_uv * (1.0 + k)),
        },
    ]
}

fn failing(conds: &[Condition], f: &Features) -> Vec<usize> {
    conds
        .iter()
        .enumerate()
        .filter(|(_, c)| !(c.passes)(f, 0.0))
        .map(|(i, _)| i)
        .collect()
}

/// A feature of a window, for the distribution tables.
struct FeatureCol {
    name: &'static str,
    get: fn(&Features) -> f32,
    rule: String,
    vf_side: Box<dyn Fn(f32) -> bool>,
}

fn feature_cols(st: &Settings) -> Vec<FeatureCol> {
    let s = *st;
    vec![
        FeatureCol {
            name: "peak-to-peak (µV, after the analyser's filter)",
            get: |f| f.peak_to_peak_uv,
            rule: format!("≥ {} for shock, < {} = flatline, > {} = implausible", s.fine_vf_max_pp_uv, s.asystole_max_uv, s.max_plausible_pp_uv),
            vf_side: Box::new(move |v| (s.fine_vf_max_pp_uv..=s.max_plausible_pp_uv).contains(&v)),
        },
        FeatureCol {
            name: "root-mean-square (µV, after the analyser's filter)",
            get: |f| f.rms_uv,
            rule: format!("≥ {} (or leakage ≤ {}) to escape the flatline rule", s.asystole_max_rms_uv, s.asystole_min_leakage),
            vf_side: Box::new(move |v| v >= s.asystole_max_rms_uv),
        },
        FeatureCol {
            name: "noise ratio",
            get: |f| f.noise_ratio,
            rule: format!("≤ {}", s.max_noise_ratio),
            vf_side: Box::new(move |v| v <= s.max_noise_ratio),
        },
        FeatureCol {
            name: "low-band fraction",
            get: |f| f.low_band_fraction,
            rule: format!("≤ {}", s.max_low_band_fraction),
            vf_side: Box::new(move |v| v <= s.max_low_band_fraction),
        },
        FeatureCol {
            name: "threshold-crossing fraction (TCSC)",
            get: |f| f.threshold_crossing_fraction,
            rule: format!("> {}", s.tcsc_vf_min),
            vf_side: Box::new(move |v| v > s.tcsc_vf_min),
        },
        FeatureCol {
            name: "band-pass count",
            get: |f| f.bandpass_count_fraction,
            rule: format!("> {}", s.count_vf_min),
            vf_side: Box::new(move |v| v > s.count_vf_min),
        },
        FeatureCol {
            name: "VF-filter leakage",
            get: |f| f.vf_leakage,
            rule: format!("< {}", s.leakage_vf_max),
            vf_side: Box::new(move |v| v < s.leakage_vf_max),
        },
        FeatureCol {
            name: "dominant frequency (Hz)",
            get: |f| f.dominant_frequency_hz,
            rule: format!("> {} and < {}", s.dominant_vf_min_hz, s.dominant_vf_max_hz),
            vf_side: Box::new(move |v| v > s.dominant_vf_min_hz && v < s.dominant_vf_max_hz),
        },
        FeatureCol {
            name: "spectral concentration",
            get: |f| f.spectral_concentration,
            rule: format!("> {}", s.concentration_vf_min),
            vf_side: Box::new(move |v| v > s.concentration_vf_min),
        },
        FeatureCol {
            name: "saturated fraction (front-end limit)",
            get: |f| f.saturated_fraction,
            rule: format!("≤ {}", s.max_saturated_fraction),
            vf_side: Box::new(move |v| v <= s.max_saturated_fraction),
        },
        FeatureCol {
            name: "clipped fraction (flat tops)",
            get: |f| f.clipped_fraction,
            rule: format!("≤ {}", s.max_clipped_fraction),
            vf_side: Box::new(move |v| v <= s.max_clipped_fraction),
        },
    ]
}

const REASONS: [Reason; 11] = [
    Reason::InvalidMeasurement,
    Reason::Saturated,
    Reason::Clipped,
    Reason::AmplitudeImplausible,
    Reason::Asystole,
    Reason::Noisy,
    Reason::PossibleArtifact,
    Reason::FineVf,
    Reason::MethodsDisagree,
    Reason::BothMethodsShockable,
    Reason::NeitherMethodShockable,
];

fn features_short(f: &Features) -> String {
    format!(
        "pp {:.0} µV, noise {:.2}, low-band {:.2}, TCSC {:.2}, count {:.2}, leak {:.2}, dom {:.2} Hz, conc {:.2}",
        f.peak_to_peak_uv,
        f.noise_ratio,
        f.low_band_fraction,
        f.threshold_crossing_fraction,
        f.bandpass_count_fraction,
        f.vf_leakage,
        f.dominant_frequency_hz,
        f.spectral_concentration
    )
}

/// Metadata for the report header.
#[derive(Debug, Clone)]
pub struct Meta {
    /// Date of the run (YYYY-MM-DD).
    pub date: String,
    /// Git commit of the analyser, with a note on uncommitted changes.
    pub commit: String,
    /// Data directory.
    pub data_dir: String,
    /// Records that could not be loaded, with the error.
    pub failed: Vec<(Db, String, String)>,
    /// If the run was restricted to a record list: which list.
    pub records: Option<String>,
    /// What kind of run this is, in a sentence or two (printed under the title).
    pub kind: String,
    /// The analyser settings used.
    pub settings: Settings,
}

fn seg_group<'a>(all: &'a [SegmentResult], variant: u32, title: String, pick: impl Fn(&SegmentResult) -> bool) -> Group<'a> {
    Group {
        title,
        segs: all.iter().filter(|s| s.variant_s == variant && pick(s)).collect(),
    }
}

fn dbs_present(outputs: &[RecordOutput]) -> Vec<Db> {
    Db::ALL
        .into_iter()
        .filter(|d| outputs.iter().any(|o| o.db == *d))
        .collect()
}

fn max_channels(outputs: &[RecordOutput], db: Db) -> usize {
    outputs
        .iter()
        .filter(|o| o.db == db)
        .map(|o| o.channel_info.len())
        .max()
        .unwrap_or(0)
}

/// Headline row figures.
fn headline_cell(segs: &[&SegmentResult], class: TruthClass) -> String {
    let mut t = Tally::default();
    for s in segs.iter().filter(|s| s.class == class) {
        t.add(s);
    }
    if t.n == 0 {
        return "no data".into();
    }
    let x = t.successes(class);
    let lcl = clopper_pearson_lower(x, t.n, LEVEL).unwrap_or(0.0);
    let mark = match aha_goal(class) {
        Some((g, l, _)) if x as f64 / t.n as f64 >= g && lcl >= l => "",
        Some(_) => " ✗",
        None => "",
    };
    format!("{} (LCL {}, n={}, {} rec){}", pct(x, t.n), pctf(lcl), t.n, t.records.len(), mark)
}

/// Build the whole Markdown report.
#[must_use]
pub fn build_report(outputs: &[RecordOutput], meta: &Meta) -> String {
    let mut out = String::new();
    let segs: Vec<SegmentResult> = outputs.iter().flat_map(|o| o.segments.iter().cloned()).collect();
    header(&mut out, meta, outputs);
    data_checks(&mut out, outputs, meta);
    headline(&mut out, outputs, &segs);
    timing_section(&mut out, &segs);
    segment_tables(&mut out, outputs, &segs);
    per_record(&mut out, outputs, &segs);
    streaming(&mut out, outputs);
    failure_analysis(&mut out, outputs, &segs, &meta.settings);
    amplitude_section(&mut out, outputs, &meta.settings);
    closing(&mut out);
    out
}

fn header(out: &mut String, meta: &Meta, outputs: &[RecordOutput]) {
    let _ = writeln!(out, "# Rhythm analyser on recorded heart rhythms\n");
    let _ = writeln!(out, "- **Date (UTC):** {}", meta.date);
    let _ = writeln!(out, "- **Analyser evaluated:** `aed-rhythm` at git commit {}", meta.commit);
    let _ = writeln!(out, "- **Data:** PhysioNet cudb, vfdb, mitdb (public Holter / monitoring recordings), read from `{}`", meta.data_dir);
    let _ = writeln!(out, "- **Records evaluated:** {}", outputs.len());
    if let Some(r) = &meta.records {
        let _ = writeln!(out, "- **Restricted to:** {r}");
    }
    let _ = writeln!(out, "\n{}\n", meta.kind);
    let _ = writeln!(out, "**What these numbers are not.** They are not a claim about a real device. The recordings are:\n");
    let _ = writeln!(out, "- few patients (35 + 22 + 47), so thousands of segments are really a few dozen independent people; the confidence limits below treat segments as independent and are therefore too optimistic;");
    let _ = writeln!(out, "- in-hospital monitoring and ambulatory (Holter) recordings from the 1970s-80s, not out-of-hospital cardiac arrest recorded through defibrillator pads;");
    let _ = writeln!(out, "- free of CPR artefact; no children; with recorder bandwidths and leads unlike pad ECG;");
    let _ = writeln!(out, "- of uncertain amplitude calibration (cudb's gain is \"nominal\" only; see the amplitude section), which matters because the analyser has absolute microvolt rules;");
    let _ = writeln!(out, "- annotated with varying care: mitdb by two cardiologists beat by beat; vfdb with rhythm labels only; cudb with \"preliminary\" labels, approximate VF onsets, and every beat labelled normal even when it is not.");
    let _ = writeln!(out, "\nPublic-database results are known to overstate field performance (research notes, section 2). Nothing here is safe or ready for use on a person. See SAFETY.md.\n");
    let _ = writeln!(out, "## How it was scored\n");
    let _ = writeln!(out, "- **Device-realistic analysis.** For each segment a fresh analyser (as after the reset at the start of every analysis) is fed one analysis worth of signal. The first final answer decides: `Shockable` → shock; `NonShockable` → no shock; neither by the time limit → *inconclusive* (\"unable to analyse\", no shock). \"Not sure\" does not end an analysis (D-27). Three analysis lengths: **7 s** (the shortest that can end in a shock: 1 s settling + 4 s window + 2 more windows, since at least 3 windows must say shockable), **8 s** (the analysis-time requirement mentioned in the configuration), and **12 s** (the device's actual limit, D-27; the headline).");
    let _ = writeln!(out, "- **Scoring of inconclusive.** For sensitivity it is a miss. For specificity it is a correct \"no shock\". It is also counted on its own everywhere, because a device that says \"not sure\" to everything would score perfect specificity and be useless.");
    let _ = writeln!(out, "- **Segments** are consecutive, non-overlapping, and only taken where the ground truth is one class throughout, at least {TRANSITION_GUARD_S} s from any annotated rhythm change, at least {NOISE_GUARD_S} s from any stretch marked noisy for that channel, and at least {ARTIFACT_GUARD_S} s from isolated-artifact or tape-slippage marks.");
    let _ = writeln!(out, "- **Channels.** Channel 0 and every other channel are scored separately; nothing is picked per record. Pooled tables are shown per channel.");
    let _ = writeln!(out, "- **Coarse vs fine VF** uses an amplitude measured independently of the analyser: median over 1-second pieces of the peak-to-peak value after subtracting baseline drift (a 2-second triangular moving average); fine VF is below {FINE_VF_UV} µV. **Rapid VT** is VT with a rate, from the beat annotations in the segment, of at least {RAPID_VT_BPM}/min (own choice; the AHA leaves the rate to the manufacturer). vfdb has no beat annotations, so its VT goes to \"VT, rate unknown\"; VT is also reported with no rate criterion.");
    let _ = writeln!(out, "- **Confidence limits** are one-sided 90 % lower limits, exact (Clopper–Pearson), with Wilson alongside. The AHA goals, lower limits and minimum sample sizes (marked *) are from the research notes' reproduction of the AHA 1997 table and are UNVERIFIED against the original. \"meets\" needs both the result and the lower limit at or above the AHA figures.\n");
}

fn data_checks(out: &mut String, outputs: &[RecordOutput], meta: &Meta) {
    let _ = writeln!(out, "## Data checks\n");
    let _ = writeln!(out, "| Database | records | channels | rate (Hz) | checksums OK | first sample = header | samples at recorder limit | clipped to i16 | gains (ADC/mV) | unknown labels |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|");
    for db in dbs_present(outputs) {
        let recs: Vec<&RecordOutput> = outputs.iter().filter(|o| o.db == db).collect();
        let checks: Vec<_> = recs.iter().flat_map(|o| o.checks.iter()).collect();
        let ck_ok = checks.iter().filter(|c| c.checksum_ok == Some(true)).count();
        let iv_ok = checks.iter().filter(|c| c.initial_value_ok == Some(true)).count();
        let total: u64 = recs.iter().map(|o| o.n_original * o.channel_info.len() as u64).sum();
        let rail: u64 = recs.iter().flat_map(|o| o.channel_info.iter().map(|c| c.4)).sum();
        let clip: u64 = recs.iter().flat_map(|o| o.channel_info.iter().map(|c| c.3)).sum();
        let gains: BTreeSet<String> = recs
            .iter()
            .flat_map(|o| o.channel_info.iter().map(|c| format!("{}{}", c.1, if c.2 { " (default)" } else { "" })))
            .collect();
        let unknown: BTreeSet<String> = recs.iter().flat_map(|o| o.timeline.unknown_labels.iter().cloned()).collect();
        let fs: BTreeSet<String> = recs.iter().map(|o| format!("{}", o.fs_original)).collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {}/{} | {}/{} | {} ({}) | {} | {} | {} |",
            db.name(),
            recs.len(),
            checks.len(),
            fs.into_iter().collect::<Vec<_>>().join(", "),
            ck_ok,
            checks.len(),
            iv_ok,
            checks.len(),
            rail,
            pct(rail, total),
            clip,
            gains.into_iter().collect::<Vec<_>>().join(", "),
            if unknown.is_empty() { "none".to_string() } else { unknown.into_iter().collect::<Vec<_>>().join(", ") }
        );
    }
    let _ = writeln!(out);
    if meta.failed.is_empty() {
        let _ = writeln!(out, "Every record listed in each database's RECORDS file loaded without error.\n");
    } else {
        let _ = writeln!(out, "Records that could not be loaded (left out of every table):\n");
        for (db, r, e) in &meta.failed {
            let _ = writeln!(out, "- {}/{}: {}", db.name(), r, e);
        }
        let _ = writeln!(out);
    }
    let marks: usize = outputs.iter().map(|o| o.timeline.noise_marks_with_text).sum();
    let _ = writeln!(out, "\"Samples at recorder limit\" are values at the very end of the original recorder's range: the recording itself was clipped there (the original analogue chain saturated). The analyser cannot see this: its own saturation rule looks for values beyond ±32 000 µV, which these recordings (range about ±5 mV for cudb and mitdb, ±10 mV for vfdb) can never reach. No sample needed clipping to fit the analyser's 16-bit input.\n");
    if marks > 0 {
        let _ = writeln!(out, "{marks} signal-quality (`~`) marks in cudb carry the text `(N`; the text is ignored and only the quality change is used.\n");
    }
}

fn headline(out: &mut String, outputs: &[RecordOutput], segs: &[SegmentResult]) {
    let _ = writeln!(out, "## Headline: device-realistic analysis ({DEVICE_VARIANT_S} s limit)\n");
    let _ = writeln!(out, "Each cell: result (exact 90 % lower limit, segments, records); ✗ = misses the AHA goal or its lower limit. Inconclusive = share of all scored segments ending \"unable to analyse\".\n");
    let _ = writeln!(out, "| Group | Coarse VF sensitivity (AHA ≥90%, LCL 87%) | Rapid VT sensitivity (≥75%, 67%) | NSR specificity (≥99%, 97%) | Other non-shockable specificity (≥95%, 88%) | Asystole specificity (≥95%, 92%) | Inconclusive |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|");
    let mut rows: Vec<(String, Vec<&SegmentResult>)> = Vec::new();
    for db in dbs_present(outputs) {
        rows.push((
            format!("{} channel 0", db.name()),
            segs.iter().filter(|s| s.variant_s == DEVICE_VARIANT_S && s.db == db && s.channel == 0).collect(),
        ));
    }
    rows.push((
        "**pooled, channel 0**".into(),
        segs.iter().filter(|s| s.variant_s == DEVICE_VARIANT_S && s.channel == 0).collect(),
    ));
    for db in dbs_present(outputs) {
        if max_channels(outputs, db) > 1 {
            rows.push((
                format!("{} channel 1", db.name()),
                segs.iter().filter(|s| s.variant_s == DEVICE_VARIANT_S && s.db == db && s.channel == 1).collect(),
            ));
        }
    }
    rows.push((
        "**pooled, channel 1**".into(),
        segs.iter().filter(|s| s.variant_s == DEVICE_VARIANT_S && s.channel == 1).collect(),
    ));
    for (name, rs) in rows {
        if rs.is_empty() {
            continue;
        }
        let inc = rs.iter().filter(|s| s.run.outcome == Outcome::Inconclusive).count() as u64;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} (n={}) |",
            name,
            headline_cell(&rs, TruthClass::CoarseVf),
            headline_cell(&rs, TruthClass::RapidVt),
            headline_cell(&rs, TruthClass::Nsr),
            headline_cell(&rs, TruthClass::OtherNonShockable),
            headline_cell(&rs, TruthClass::Asystole),
            pct(inc, rs.len() as u64),
            rs.len()
        );
    }
    let _ = writeln!(out);
}

/// Time to decision and shocks lost in the stand-clear countdown.
fn timing_section(out: &mut String, segs: &[SegmentResult]) {
    let _ = writeln!(out, "## Time to decision, and shocks lost in the countdown ({DEVICE_VARIANT_S} s analysis)\n");
    let _ = writeln!(out, "Every second of analysis is hands-off time. \"Decided by\" = share of segments with a final answer (shock or no shock) by that time after the analysis started; the rest are still analysing. After \"shock advised\" the rescue sequence counts down for {COUNTDOWN_S} s and abandons the shock on any answer other than `Shockable` (aed-core); \"cancelled in countdown\" counts shocks the analyser itself would have called off that way, measured by carrying on into the signal that follows (where the same rhythm continues). \"Delivered\" = shocks not cancelled, over all segments; a shock whose countdown could not be checked counts as delivered.\n");
    let _ = writeln!(out, "| Group | Class | n | decided by 7 s | 8 s | 10 s | 12 s | median time to answer (s) | 90th pct (s) | shocks | cancelled in countdown | countdown not checkable | delivered |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for ch in [0_usize, 1] {
        for class in [TruthClass::CoarseVf, TruthClass::Vfl, TruthClass::VtRateUnknown, TruthClass::Nsr, TruthClass::OtherNonShockable, TruthClass::Asystole] {
            let v: Vec<&SegmentResult> = segs
                .iter()
                .filter(|s| s.variant_s == DEVICE_VARIANT_S && s.channel == ch && s.class == class)
                .collect();
            if v.is_empty() {
                continue;
            }
            let n = v.len() as u64;
            let by = |t: f64| v.iter().filter(|s| s.run.decided_at_s.is_some_and(|d| d <= t + 1e-9)).count() as u64;
            let times: Vec<f64> = v.iter().filter_map(|s| s.run.decided_at_s).collect();
            let shocks: Vec<&&SegmentResult> = v.iter().filter(|s| s.run.outcome == Outcome::Shock).collect();
            let cancelled = shocks.iter().filter(|s| s.run.cancelled_in_countdown == Some(true)).count() as u64;
            let unknown = shocks.iter().filter(|s| s.run.cancelled_in_countdown.is_none()).count() as u64;
            let _ = writeln!(
                out,
                "| channel {ch} | {} | {n} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                class.name(),
                pct(by(7.0), n),
                pct(by(8.0), n),
                pct(by(10.0), n),
                pct(by(12.0), n),
                median(&times).map_or("–".into(), |m| format!("{m:.0}")),
                percentile(&times, 90.0).map_or("–".into(), |m| format!("{m:.0}")),
                shocks.len(),
                cancelled,
                unknown,
                pct(shocks.len() as u64 - cancelled, n)
            );
        }
    }
    let _ = writeln!(out);
}

fn segment_tables(out: &mut String, outputs: &[RecordOutput], segs: &[SegmentResult]) {
    let _ = writeln!(out, "## Segment-level results\n");
    let _ = writeln!(out, "Report-only buckets show the share that ended in a shock. \"cudb non-VF, rhythm not given\" is cudb time outside the annotated VF episodes for which no rhythm label exists; it may contain VT (cudb is a tachyarrhythmia database) and is therefore not counted as a non-shockable class.\n");
    for variant in VARIANTS_S.iter().rev().copied() {
        let _ = writeln!(out, "### {variant} s analysis\n");
        if variant == DEVICE_VARIANT_S {
            let _ = writeln!(out, "No \"Rapid VT\" row appears: the only VT with beat annotations (mitdb, a few cudb labels) comes in runs too short to hold a whole segment plus the {TRANSITION_GUARD_S} s guard margins, and the few that fit are slow. vfdb's VT has no beat annotations, so its rate is unknown. The rapid-VT goal therefore cannot be tested on these databases with annotation-based rates; see the rate-estimate table in the failure analysis for a rough split.\n");
        }
        let mut groups = Vec::new();
        for db in dbs_present(outputs) {
            for ch in 0..max_channels(outputs, db) {
                if variant != DEVICE_VARIANT_S && ch > 0 {
                    continue;
                }
                groups.push(seg_group(segs, variant, format!("{} channel {}", db.name(), ch), |s| s.db == db && s.channel == ch));
            }
        }
        groups.push(seg_group(segs, variant, "Pooled, channel 0".into(), |s| s.channel == 0));
        groups.push(seg_group(segs, variant, "Pooled, channel 1 (vfdb + mitdb)".into(), |s| s.channel == 1));
        for g in &groups {
            if !g.segs.is_empty() {
                class_table(out, g);
            }
        }
        if variant == DEVICE_VARIANT_S {
            for ch in 0..2 {
                let g: Vec<&SegmentResult> = segs.iter().filter(|s| s.variant_s == variant && s.channel == ch).collect();
                if g.is_empty() {
                    continue;
                }
                let _ = writeln!(out, "**Non-shockable rhythms by label, pooled channel {ch}, {variant} s**\n");
                other_breakdown(out, &g);
            }
        }
    }
}

fn per_record(out: &mut String, outputs: &[RecordOutput], segs: &[SegmentResult]) {
    let _ = writeln!(out, "## Per record ({DEVICE_VARIANT_S} s analysis, every channel)\n");
    let _ = writeln!(out, "So that one bad patient is visible. VF = coarse VF; \"VT/VFL shock\" = shocks / segments over VT of any rate and flutter; FP = false shocks on NSR or other non-shockable rhythms. Flags: **VF<50%** (≥5 coarse-VF segments and under half shocked), **FP** (any false shock on an AHA non-shockable class), **INC>50%** (over half of all segments inconclusive).\n");
    let _ = writeln!(out, "| Record | ch | lead | coarse VF n | VF Se | VT/VFL shock | NSR n | NSR Sp | other n | other Sp | FP | inconcl. | flags |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for o in outputs {
        for (ch, info) in o.channel_info.iter().enumerate() {
            let rs: Vec<&SegmentResult> = segs
                .iter()
                .filter(|s| s.variant_s == DEVICE_VARIANT_S && s.db == o.db && s.record == o.name && s.channel == ch)
                .collect();
            if rs.is_empty() {
                continue;
            }
            let count = |f: &dyn Fn(&SegmentResult) -> bool| rs.iter().filter(|s| f(s)).count() as u64;
            let vf_n = count(&|s| s.class == TruthClass::CoarseVf);
            let vf_s = count(&|s| s.class == TruthClass::CoarseVf && s.run.outcome == Outcome::Shock);
            let vt_n = count(&|s| matches!(s.class, TruthClass::RapidVt | TruthClass::SlowVt | TruthClass::VtRateUnknown | TruthClass::Vfl));
            let vt_s = count(&|s| matches!(s.class, TruthClass::RapidVt | TruthClass::SlowVt | TruthClass::VtRateUnknown | TruthClass::Vfl) && s.run.outcome == Outcome::Shock);
            let nsr_n = count(&|s| s.class == TruthClass::Nsr);
            let nsr_fp = count(&|s| s.class == TruthClass::Nsr && s.run.outcome == Outcome::Shock);
            let oth_n = count(&|s| matches!(s.class, TruthClass::OtherNonShockable | TruthClass::Asystole));
            let oth_fp = count(&|s| matches!(s.class, TruthClass::OtherNonShockable | TruthClass::Asystole) && s.run.outcome == Outcome::Shock);
            let inc = count(&|s| s.run.outcome == Outcome::Inconclusive);
            let mut flags = Vec::new();
            if vf_n >= 5 && vf_s * 2 < vf_n {
                flags.push("**VF<50%**");
            }
            if nsr_fp + oth_fp > 0 {
                flags.push("**FP**");
            }
            if inc * 2 > rs.len() as u64 {
                flags.push("**INC>50%**");
            }
            let _ = writeln!(
                out,
                "| {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                o.db.name(),
                o.name,
                ch,
                info.0,
                vf_n,
                pct(vf_s, vf_n),
                if vt_n == 0 { "–".to_string() } else { format!("{vt_s}/{vt_n}") },
                nsr_n,
                pct(nsr_n - nsr_fp, nsr_n),
                oth_n,
                pct(oth_n - oth_fp, oth_n),
                nsr_fp + oth_fp,
                pct(inc, rs.len() as u64),
                flags.join(" ")
            );
        }
    }
    let _ = writeln!(out);
}

/// One VF/flutter episode in a streaming run.
#[derive(Debug, Clone)]
pub struct Episode {
    /// Database.
    pub db: Db,
    /// Record.
    pub record: String,
    /// Channel.
    pub channel: usize,
    /// Label.
    pub label: Label,
    /// Onset, seconds.
    pub onset: f64,
    /// End, seconds.
    pub end: f64,
    /// Seconds from onset to the first `Shockable`, if one came before the end.
    pub delay: Option<f64>,
    /// True if the analyser already said `Shockable` at the onset.
    pub shockable_at_onset: bool,
    /// Seconds `Shockable` persisted after the end (if it was shockable at the end
    /// and a non-shockable rhythm followed).
    pub persistence: Option<f64>,
    /// What followed.
    pub next_label: Option<Label>,
    /// End of the following span, seconds.
    pub next_end: Option<f64>,
}

fn decision_at(stream: &[ScoredWindow], t: f64) -> Option<RhythmDecision> {
    stream
        .iter()
        .rev()
        .find(|w| w.t_end <= t + 1e-9)
        .filter(|w| t - w.t_end < 1.0 + 1e-9)
        .map(|w| w.report.decision)
}

/// Find the VF/flutter (and VT, for persistence) episodes in every streaming run.
#[must_use]
pub fn episodes(outputs: &[RecordOutput]) -> Vec<Episode> {
    let mut eps = Vec::new();
    for o in outputs {
        for (ch, stream) in o.streams.iter().enumerate() {
            let spans = &o.timeline.spans;
            for (i, sp) in spans.iter().enumerate() {
                if !sp.label.is_ventricular_shock_candidate() {
                    continue;
                }
                let next = spans.get(i + 1);
                let delay = stream
                    .iter()
                    .find(|w| w.t_end >= sp.start && w.t_end <= sp.end + 1e-9 && w.report.decision == RhythmDecision::Shockable)
                    .map(|w| w.t_end - sp.start);
                let at_onset = decision_at(stream, sp.start) == Some(RhythmDecision::Shockable);
                let next_nonshock = next.is_some_and(|n| {
                    matches!(n.label, Label::Nsr | Label::Other(_) | Label::Asystole | Label::CudbUnlabelled)
                        && n.end - n.start >= 10.0
                });
                let persistence = if next_nonshock && decision_at(stream, sp.end) == Some(RhythmDecision::Shockable) {
                    stream
                        .iter()
                        .find(|w| w.t_end > sp.end && w.report.decision != RhythmDecision::Shockable)
                        .map(|w| w.t_end - sp.end)
                } else if next_nonshock {
                    Some(0.0)
                } else {
                    None
                };
                eps.push(Episode {
                    db: o.db,
                    record: o.name.clone(),
                    channel: ch,
                    label: sp.label.clone(),
                    onset: sp.start,
                    end: sp.end,
                    delay,
                    shockable_at_onset: at_onset,
                    persistence,
                    next_label: next.map(|n| n.label.clone()),
                    next_end: next.map(|n| n.end),
                });
            }
        }
    }
    eps
}

fn dist(v: &[f64]) -> String {
    if v.is_empty() {
        return "–".into();
    }
    format!(
        "median {:.1} s, 90th pct {:.1} s, max {:.1} s",
        median(v).unwrap_or(0.0),
        percentile(v, 90.0).unwrap_or(0.0),
        percentile(v, 100.0).unwrap_or(0.0)
    )
}

fn streaming(out: &mut String, outputs: &[RecordOutput]) {
    let _ = writeln!(out, "## Continuous (streaming) run\n");
    let _ = writeln!(out, "One analyser runs over each whole record and channel without resets, deciding once a second. Each per-second decision is scored against the truth when its last {} s (the 4 s window plus the two earlier windows needed for confirmation) lie inside one clean run (same guard margins as above). The decision scored is what the analyser reported for that window (`Shockable` only once confirmed; an unconfirmed shockable window counts as \"not sure\").\n", crate::eval::STREAM_TRUTH_SPAN_S);
    let _ = writeln!(out, "| Database | ch | Class | seconds | Shockable | NonShockable | Not sure |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|");
    for db in dbs_present(outputs) {
        for ch in 0..max_channels(outputs, db) {
            let mut by: BTreeMap<TruthClass, [u64; 3]> = BTreeMap::new();
            for o in outputs.iter().filter(|o| o.db == db) {
                for w in o.streams.get(ch).into_iter().flatten() {
                    if let Some(c) = w.class {
                        let e = by.entry(c).or_default();
                        let i = match w.report.decision {
                            RhythmDecision::Shockable => 0,
                            RhythmDecision::NonShockable => 1,
                            RhythmDecision::Indeterminate => 2,
                        };
                        if let Some(v) = e.get_mut(i) {
                            *v += 1;
                        }
                    }
                }
            }
            for (c, [s, n, i]) in by {
                let tot = s + n + i;
                let _ = writeln!(out, "| {} | {} | {} | {} | {} | {} | {} |", db.name(), ch, c.name(), tot, pct(s, tot), pct(n, tot), pct(i, tot));
            }
        }
    }
    let _ = writeln!(out);

    let eps = episodes(outputs);
    let _ = writeln!(out, "### Detection delay after VF / flutter onset\n");
    let _ = writeln!(out, "Measured from the annotated onset (cudb onsets are only approximate, per its documentation) to the first confirmed `Shockable` before the episode ends. The fastest possible is 7 s after the window first lies wholly in VF, i.e. about 6–7 s after onset. Episodes shorter than 7 s are counted apart. A delay under 6 s is faster than the analyser can confirm VF from scratch: it means the windows before the annotated onset already looked shockable to it (in vfdb record 418 the \"NSR\" between flutter bursts does). Noise marks are not removed here.\n");
    let _ = writeln!(out, "| Database | ch | label | episodes ≥7 s | detected | not detected | already shockable at onset | delay (detected) | of which under 6 s | episodes <7 s |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|");
    for db in dbs_present(outputs) {
        for ch in 0..max_channels(outputs, db) {
            for label in [Label::Vf, Label::Vfl] {
                let e: Vec<&Episode> = eps.iter().filter(|e| e.db == db && e.channel == ch && e.label == label).collect();
                if e.is_empty() {
                    continue;
                }
                let long: Vec<&&Episode> = e.iter().filter(|e| e.end - e.onset >= 7.0).collect();
                let det: Vec<f64> = long.iter().filter(|e| !e.shockable_at_onset).filter_map(|e| e.delay).collect();
                let not_det = long.iter().filter(|e| e.delay.is_none()).count();
                let at_on = long.iter().filter(|e| e.shockable_at_onset).count();
                let fast = det.iter().filter(|d| **d < 6.0).count();
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    db.name(),
                    ch,
                    label.text(),
                    long.len(),
                    det.len(),
                    not_det,
                    at_on,
                    dist(&det),
                    fast,
                    e.len() - long.len()
                );
            }
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "### How long \"shock\" persists after a shockable rhythm ends\n");
    let _ = writeln!(out, "For VF, flutter and VT episodes followed by at least 10 s of a non-shockable (or cudb unlabelled) rhythm: the time from the annotated end to the first decision that is not `Shockable`. Zero means the analyser was not saying `Shockable` at the end. In cudb the end of VF is often a defibrillation shock, so the following seconds are shock artefact.\n");
    let _ = writeln!(out, "| Database | ch | episodes | shockable at end | persistence (those) | false shock 6–16 s after the end |");
    let _ = writeln!(out, "|---|---|---|---|---|---|");
    for db in dbs_present(outputs) {
        for ch in 0..max_channels(outputs, db) {
            let e: Vec<&Episode> = eps.iter().filter(|e| e.db == db && e.channel == ch && e.persistence.is_some()).collect();
            if e.is_empty() {
                continue;
            }
            let p: Vec<f64> = e.iter().filter_map(|e| e.persistence).filter(|p| *p > 0.0).collect();
            // Shockable again once the decision no longer depends on any signal
            // from before the end (6 s on), up to 16 s, within the following span.
            let relapse = e
                .iter()
                .filter(|ep| {
                    let Some(o) = outputs.iter().find(|o| o.db == ep.db && o.name == ep.record) else { return false };
                    let Some(stream) = o.streams.get(ep.channel) else { return false };
                    let limit = (ep.end + 16.0).min(ep.next_end.unwrap_or(ep.end));
                    stream.iter().any(|w| w.t_end >= ep.end + 6.0 && w.t_end <= limit && w.report.decision == RhythmDecision::Shockable)
                })
                .count();
            let _ = writeln!(out, "| {} | {} | {} | {} | {} | {} |", db.name(), ch, e.len(), p.len(), dist(&p), relapse);
        }
    }
    let _ = writeln!(out);
    // Transition profile.
    let _ = writeln!(out, "### Share of VF/flutter episodes reported `Shockable`, by seconds from onset (channel 0, episodes ≥ 20 s)\n");
    let offsets: [i32; 13] = [-5, 0, 2, 4, 5, 6, 7, 8, 9, 10, 12, 15, 20];
    let _ = write!(out, "| Database | episodes |");
    for k in offsets {
        let _ = write!(out, " {k:+} s |");
    }
    let _ = writeln!(out);
    let _ = write!(out, "|---|---|");
    for _ in offsets {
        let _ = write!(out, "---|");
    }
    let _ = writeln!(out);
    for db in dbs_present(outputs) {
        let e: Vec<&Episode> = eps
            .iter()
            .filter(|e| e.db == db && e.channel == 0 && matches!(e.label, Label::Vf | Label::Vfl) && e.end - e.onset >= 20.0)
            .collect();
        if e.is_empty() {
            continue;
        }
        let _ = write!(out, "| {} | {} |", db.name(), e.len());
        for k in offsets {
            let n = e
                .iter()
                .filter(|ep| {
                    outputs
                        .iter()
                        .find(|o| o.db == ep.db && o.name == ep.record)
                        .and_then(|o| o.streams.first())
                        .and_then(|s| decision_at(s, ep.onset + f64::from(k)))
                        == Some(RhythmDecision::Shockable)
                })
                .count();
            let _ = write!(out, " {} |", pct(n as u64, e.len() as u64));
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out);
}

fn failure_analysis(out: &mut String, outputs: &[RecordOutput], segs: &[SegmentResult], st: &Settings) {
    let conds = conditions(st);
    let _ = writeln!(out, "## Failure analysis\n");
    let _ = writeln!(out, "Nothing below was used to change the analyser. Where a threshold is \"moved\", it is a what-if count only.\n");

    // 1. Reasons per class (streaming windows, all channels).
    let _ = writeln!(out, "### Which rule gave each window's verdict (streaming windows, all databases and channels)\n");
    let _ = write!(out, "| Class | windows |");
    for r in REASONS {
        let _ = write!(out, " {} |", reason_label(r));
    }
    let _ = writeln!(out);
    let _ = write!(out, "|---|---|");
    for _ in REASONS {
        let _ = write!(out, "---|");
    }
    let _ = writeln!(out);
    let windows: Vec<&ScoredWindow> = outputs.iter().flat_map(|o| o.streams.iter().flatten()).collect();
    for class in TruthClass::ALL {
        let ws: Vec<&&ScoredWindow> = windows.iter().filter(|w| w.class == Some(class)).collect();
        if ws.is_empty() {
            continue;
        }
        let _ = write!(out, "| {} | {} |", class.name(), ws.len());
        for r in REASONS {
            let n = ws.iter().filter(|w| w.report.verdict.reason == r).count();
            let _ = write!(out, " {} |", pct(n as u64, ws.len() as u64));
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Exclusion rules by database (share of all scored streaming windows, any class):\n");
    let _ = writeln!(out, "| Database | windows | saturated | implausible size | flatline | noisy | low-band (compression) | fine VF |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|");
    for db in dbs_present(outputs) {
        let ws: Vec<&ScoredWindow> = outputs.iter().filter(|o| o.db == db).flat_map(|o| o.streams.iter().flatten()).filter(|w| w.class.is_some()).collect();
        let f = |r: Reason| pct(ws.iter().filter(|w| w.report.verdict.reason == r).count() as u64, ws.len() as u64);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            db.name(),
            ws.len(),
            f(Reason::Saturated),
            f(Reason::AmplitudeImplausible),
            f(Reason::Asystole),
            f(Reason::Noisy),
            f(Reason::PossibleArtifact),
            f(Reason::FineVf)
        );
    }
    let _ = writeln!(out);

    // 2. Feature distributions.
    let _ = writeln!(out, "### Feature distributions on real data versus the thresholds\n");
    let _ = writeln!(out, "Streaming windows, all databases and channels, by true class. \"VF side\" = share of windows on the shockable side of that feature's threshold(s). A feature only helps if VF windows are mostly on the VF side and non-shockable windows mostly are not.\n");
    for col in feature_cols(st) {
        let _ = writeln!(out, "**{}** (shockable side: {})\n", col.name, col.rule);
        let _ = writeln!(out, "| Class | windows | 5% | 25% | median | 75% | 95% | VF side |");
        let _ = writeln!(out, "|---|---|---|---|---|---|---|---|");
        for class in TruthClass::ALL {
            let vals: Vec<f64> = windows.iter().filter(|w| w.class == Some(class)).map(|w| f64::from((col.get)(&w.report.features))).collect();
            if vals.is_empty() {
                continue;
            }
            let side = windows.iter().filter(|w| w.class == Some(class)).filter(|w| (col.vf_side)((col.get)(&w.report.features))).count();
            let p = |q| percentile(&vals, q).map_or_else(|| "–".to_string(), |v| format!("{v:.3}"));
            let _ = writeln!(out, "| {} | {} | {} | {} | {} | {} | {} | {} |", class.name(), vals.len(), p(5.0), p(25.0), p(50.0), p(75.0), p(95.0), pct(side as u64, vals.len() as u64));
        }
        let _ = writeln!(out);
    }

    // 3. Missed coarse VF.
    let dev: Vec<&SegmentResult> = segs.iter().filter(|s| s.variant_s == DEVICE_VARIANT_S).collect();
    let missed: Vec<&&SegmentResult> = dev.iter().filter(|s| s.class == TruthClass::CoarseVf && s.run.outcome != Outcome::Shock).collect();
    let vf_all: Vec<&&SegmentResult> = dev.iter().filter(|s| s.class == TruthClass::CoarseVf).collect();
    let _ = writeln!(out, "### Missed coarse VF ({DEVICE_VARIANT_S} s analyses, all channels)\n");
    let _ = writeln!(out, "{} of {} coarse-VF segments did not end in a shock: {} ended \"no shock\", {} inconclusive.\n",
        missed.len(), vf_all.len(),
        missed.iter().filter(|s| s.run.outcome == Outcome::NoShock).count(),
        missed.iter().filter(|s| s.run.outcome == Outcome::Inconclusive).count());
    let _ = writeln!(out, "How the missed ones ended (the verdict of the window that ended the analysis, or for inconclusive ones, the most common verdict among its windows):\n");
    let mut end_reason: BTreeMap<&'static str, u64> = BTreeMap::new();
    for s in &missed {
        let r = match s.run.decisive.and_then(|i| s.run.windows.get(i)) {
            Some(w) => reason_label(w.verdict.reason),
            None => {
                let mut c: BTreeMap<&'static str, u64> = BTreeMap::new();
                for w in &s.run.windows {
                    *c.entry(reason_label(w.verdict.reason)).or_default() += 1;
                }
                c.into_iter().max_by_key(|(_, n)| *n).map_or("no window", |(k, _)| k)
            }
        };
        *end_reason.entry(r).or_default() += 1;
    }
    let _ = writeln!(out, "| Ended by | segments |");
    let _ = writeln!(out, "|---|---|");
    for (r, n) in &end_reason {
        let _ = writeln!(out, "| {r} | {n} |");
    }
    let _ = writeln!(out);
    blocker_table(out, &conds, &missed.iter().flat_map(|s| s.run.windows.iter()).collect::<Vec<_>>(), "windows of missed coarse-VF segments");
    let vf_windows: Vec<&WindowReport> = windows.iter().filter(|w| w.class == Some(TruthClass::CoarseVf)).map(|w| &w.report).collect();
    blocker_table(out, &conds, &vf_windows, "all coarse-VF streaming windows");

    // 4. False positives.
    let fps: Vec<&&SegmentResult> = dev.iter().filter(|s| (s.class.should_not_shock()) && s.run.outcome == Outcome::Shock).collect();
    let _ = writeln!(out, "### False shocks on non-shockable rhythms ({DEVICE_VARIANT_S} s analyses, all channels)\n");
    let mut by_label: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    for s in dev.iter().filter(|s| s.class.should_not_shock()) {
        let e = by_label.entry(s.label.text()).or_default();
        e.1 += 1;
        if s.run.outcome == Outcome::Shock {
            e.0 += 1;
        }
    }
    let mut ranked: Vec<(String, (u64, u64))> = by_label.into_iter().collect();
    ranked.sort_by(|a, b| (b.1 .0 as f64 / b.1 .1.max(1) as f64).total_cmp(&(a.1 .0 as f64 / a.1 .1.max(1) as f64)));
    let _ = writeln!(out, "Worst rhythm classes by false-shock rate:\n");
    let _ = writeln!(out, "| Rhythm | false shocks | segments | rate |");
    let _ = writeln!(out, "|---|---|---|---|");
    for (l, (fp, n)) in &ranked {
        let _ = writeln!(out, "| {l} | {fp} | {n} | {} |", pct(*fp, *n));
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Every false shock passed all {} conditions in at least {} of the last {} windows (that is the only way to reach `Shockable`), so \"which rule fired\" is always \"both method families said shockable\". The useful question is which condition was closest to stopping it. For each condition: the share of the confirming windows of every false shock that would have been stopped had that one threshold been 10 % / 20 % stricter, against the share of correctly shocked coarse-VF confirming windows that the same change would also stop (the cost). What-if counts only; nothing was changed.\n", conds.len(), st.confirm_windows, st.confirm_span);
    let span = usize::from(st.confirm_span);
    let fp_windows: Vec<&WindowReport> = fps.iter().flat_map(|s| confirming(s, span)).collect();
    let tp_windows: Vec<&WindowReport> = dev
        .iter()
        .filter(|s| s.class == TruthClass::CoarseVf && s.run.outcome == Outcome::Shock)
        .flat_map(|s| confirming(s, span))
        .collect();
    let _ = writeln!(out, "| Condition | rule | false-shock windows stopped at 10% / 20% stricter (of {}) | coarse-VF windows also stopped at 10% / 20% (of {}) |", fp_windows.len(), tp_windows.len());
    let _ = writeln!(out, "|---|---|---|---|");
    for c in &conds {
        let stop = |ws: &[&WindowReport], k: f32| ws.iter().filter(|w| !(c.passes)(&w.features, k)).count() as u64;
        let _ = writeln!(
            out,
            "| {} | {} | {} / {} | {} / {} |",
            c.key,
            c.rule,
            pct(stop(&fp_windows, 0.1), fp_windows.len() as u64),
            pct(stop(&fp_windows, 0.2), fp_windows.len() as u64),
            pct(stop(&tp_windows, 0.1), tp_windows.len() as u64),
            pct(stop(&tp_windows, 0.2), tp_windows.len() as u64)
        );
    }
    let _ = writeln!(out);
    let nsr_windows: Vec<&WindowReport> = windows.iter().filter(|w| matches!(w.class, Some(TruthClass::Nsr | TruthClass::OtherNonShockable))).map(|w| &w.report).collect();
    let _ = writeln!(out, "How often each condition already lets a non-shockable window through (streaming NSR + other non-shockable windows, n = {}): a condition passed by nearly every non-shockable window does no work.\n", nsr_windows.len());
    let _ = writeln!(out, "| Condition | non-shockable windows on the VF side | coarse-VF windows on the VF side |");
    let _ = writeln!(out, "|---|---|---|");
    for c in &conds {
        let a = nsr_windows.iter().filter(|w| (c.passes)(&w.features, 0.0)).count() as u64;
        let b = vf_windows.iter().filter(|w| (c.passes)(&w.features, 0.0)).count() as u64;
        let _ = writeln!(out, "| {} | {} | {} |", c.key, pct(a, nsr_windows.len() as u64), pct(b, vf_windows.len() as u64));
    }
    let _ = writeln!(out);

    let mut fp_rec: BTreeMap<String, (u64, BTreeSet<String>)> = BTreeMap::new();
    for s in &fps {
        let e = fp_rec.entry(format!("{}/{} ch{}", s.db.name(), s.record, s.channel)).or_default();
        e.0 += 1;
        e.1.insert(s.label.text());
    }
    let mut fp_rec: Vec<_> = fp_rec.into_iter().collect();
    fp_rec.sort_by_key(|x| std::cmp::Reverse(x.1 .0));
    let _ = writeln!(out, "False shocks by record and channel ({} in all):\n", fps.len());
    let _ = writeln!(out, "| Record | false shocks | on rhythm(s) |");
    let _ = writeln!(out, "|---|---|---|");
    for (r, (n, labels)) in &fp_rec {
        let _ = writeln!(out, "| {r} | {n} | {} |", labels.iter().cloned().collect::<Vec<_>>().join(", "));
    }
    let _ = writeln!(out);

    // VT split by an estimated rate.
    let vt: Vec<&&SegmentResult> = dev.iter().filter(|s| s.class == TruthClass::VtRateUnknown).collect();
    if !vt.is_empty() {
        let _ = writeln!(out, "### VT without beat annotations, split by a rough rate estimated from the signal\n");
        let _ = writeln!(out, "vfdb gives VT episodes but no beats, so the AHA rate criterion cannot be applied from the annotations. As a rough guide only, each segment's rate was estimated from the signal's strongest repetition (autocorrelation, 60–300/min). This helper is UNVERIFIED and can be off by a factor of two; it is not used for any AHA row above. 12 s analyses, both channels counted.\n");
        let _ = writeln!(out, "| Estimated rate | segments | records | shock | no shock | inconcl. | shock rate |");
        let _ = writeln!(out, "|---|---|---|---|---|---|---|");
        for (name, pick) in [
            ("≥ 150/min (would be rapid VT)", (|r: Option<f64>| r.is_some_and(|r| r >= RAPID_VT_BPM)) as fn(Option<f64>) -> bool),
            ("< 150/min (would be slow VT)", |r| r.is_some_and(|r| r < RAPID_VT_BPM)),
            ("no clear rate", |r: Option<f64>| r.is_none()),
        ] {
            let mut t = Tally::default();
            for s in vt.iter().filter(|s| pick(s.est_rate_bpm)) {
                t.add(s);
            }
            let _ = writeln!(out, "| {name} | {} | {} | {} | {} | {} | {} |", t.n, t.records.len(), t.shock, t.no_shock, t.inconclusive, pct(t.shock, t.n));
        }
        let _ = writeln!(out);
    }

    // 5. Worst segments.
    let mut worst_fp: Vec<&&SegmentResult> = fps.clone();
    worst_fp.sort_by(|a, b| {
        let pri = |s: &SegmentResult| match s.class {
            TruthClass::Nsr => 0,
            TruthClass::Asystole => 1,
            _ => 2,
        };
        pri(a).cmp(&pri(b)).then(a.run.decided_at_s.unwrap_or(99.0).total_cmp(&b.run.decided_at_s.unwrap_or(99.0)))
    });
    let _ = writeln!(out, "### Every false shock ({} in all, both channels)\n", worst_fp.len());
    let _ = writeln!(out, "Normal rhythm first, then asystole, then by how fast the shock came. Features are those of the window that confirmed the shock.\n");
    worst_table(out, &worst_fp.into_iter().take(200).collect::<Vec<_>>());
    let mut worst_miss: Vec<&&SegmentResult> = missed.clone();
    worst_miss.sort_by(|a, b| b.amplitude_uv.total_cmp(&a.amplitude_uv));
    let _ = writeln!(out, "### The 20 worst missed VF segments\n");
    let _ = writeln!(out, "Largest VF first (a missed large VF is the worst kind). Features are those of the last window looked at.\n");
    worst_table(out, &worst_miss.into_iter().take(20).collect::<Vec<_>>());
}

/// The shockable windows among the last `span` up to the one that
/// confirmed the shock (the windows that made the confirmation).
fn confirming(s: &SegmentResult, span: usize) -> Vec<&WindowReport> {
    match s.run.decisive {
        Some(i) => s
            .run
            .windows
            .iter()
            .take(i + 1)
            .skip((i + 1).saturating_sub(span))
            .filter(|w| w.verdict.decision == RhythmDecision::Shockable)
            .collect(),
        None => Vec::new(),
    }
}

fn blocker_table(out: &mut String, conds: &[Condition], windows: &[&WindowReport], what: &str) {
    let mut any = vec![0_u64; conds.len()];
    let mut sole = vec![0_u64; conds.len()];
    let mut passing_all = 0_u64;
    for w in windows {
        let f = failing(conds, &w.features);
        if f.is_empty() {
            passing_all += 1;
        }
        for i in &f {
            if let Some(v) = any.get_mut(*i) {
                *v += 1;
            }
        }
        if let [only] = f.as_slice() {
            if let Some(v) = sole.get_mut(*only) {
                *v += 1;
            }
        }
    }
    let n = windows.len() as u64;
    let _ = writeln!(out, "Which conditions failed, over {n} {what} ({} passed all of them). \"Only blocker\" = the window failed that condition and no other, so that single threshold alone kept it from being shockable.\n", pct(passing_all, n));
    let _ = writeln!(out, "| Condition | rule | failed (any) | only blocker |");
    let _ = writeln!(out, "|---|---|---|---|");
    for (i, c) in conds.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            c.key,
            c.rule,
            pct(any.get(i).copied().unwrap_or(0), n),
            pct(sole.get(i).copied().unwrap_or(0), n)
        );
    }
    let _ = writeln!(out);
}

fn worst_table(out: &mut String, rows: &[&&SegmentResult]) {
    if rows.is_empty() {
        let _ = writeln!(out, "None.\n");
        return;
    }
    let _ = writeln!(out, "| Record | ch | start (s) | true rhythm | amplitude (µV) | rate | outcome | at (s) | window verdict | features |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|");
    for s in rows {
        let w = s.run.decisive.and_then(|i| s.run.windows.get(i)).or_else(|| s.run.windows.last());
        let _ = writeln!(
            out,
            "| {}/{} | {} | {:.1} | {} | {:.0} | {} | {} | {} | {} | {} |",
            s.db.name(),
            s.record,
            s.channel,
            s.start_s,
            s.label.text(),
            s.amplitude_uv,
            s.rate_bpm.map_or_else(|| "–".to_string(), |r| format!("{r:.0}/min")),
            s.run.outcome.text(),
            s.run.decided_at_s.map_or_else(|| "–".to_string(), |t| format!("{t:.0}")),
            w.map_or("–", |w| reason_label(w.verdict.reason)),
            w.map_or_else(|| "–".to_string(), |w| features_short(&w.features))
        );
    }
    let _ = writeln!(out);
}

fn amplitude_section(out: &mut String, outputs: &[RecordOutput], st: &Settings) {
    let (fine_max, flat_max, implausible) = (st.fine_vf_max_pp_uv, st.asystole_max_uv, st.max_plausible_pp_uv);
    let _ = writeln!(out, "## Amplitude calibration and the microvolt rules\n");
    let _ = writeln!(out, "The analyser has three absolute-size rules: under {flat_max} µV peak to peak is flatline (no shock); under {fine_max} µV VF is \"fine\" (not shocked); over {implausible} µV is implausible (not sure). They only mean something if the recording's microvolts are right. cudb's gain of 400 ADC units/mV is described by PhysioNet as nominal (10 mV full scale \"nominal\"; cu01 was replayed from a Holter tape). Nothing here was rescaled.\n");
    let _ = writeln!(out, "Per database, channel 0: the median over records of each record's median ground-truth amplitude (1-s peak to peak, drift removed) in clean streaming windows, and how VF windows fall against the analyser's own size rules (its peak-to-peak is measured after its 1–30 Hz filter, so it reads lower than the ground-truth figure).\n");
    let _ = writeln!(out, "| Database | non-VF amplitude (median of record medians) | VF amplitude (median of record medians) | VF windows | analyser pp < {fine_max} µV (fine-VF rule) | analyser pp < {flat_max} µV (flatline rule) | ground truth < {FINE_VF_UV} µV |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|");
    for db in dbs_present(outputs) {
        let mut nonvf = Vec::new();
        let mut vf = Vec::new();
        let (mut n_vf, mut fine, mut flat, mut gt_fine) = (0_u64, 0_u64, 0_u64, 0_u64);
        for o in outputs.iter().filter(|o| o.db == db) {
            let Some(stream) = o.streams.first() else { continue };
            let a: Vec<f64> = stream
                .iter()
                .filter(|w| matches!(w.class, Some(TruthClass::Nsr | TruthClass::OtherNonShockable | TruthClass::CudbUnlabelled)))
                .map(|w| w.amplitude_uv)
                .collect();
            if let Some(m) = median(&a) {
                nonvf.push(m);
            }
            let v: Vec<&ScoredWindow> = stream.iter().filter(|w| matches!(w.class, Some(TruthClass::CoarseVf | TruthClass::FineVf))).collect();
            if let Some(m) = median(&v.iter().map(|w| w.amplitude_uv).collect::<Vec<_>>()) {
                vf.push(m);
            }
            n_vf += v.len() as u64;
            fine += v.iter().filter(|w| w.report.features.peak_to_peak_uv < fine_max).count() as u64;
            flat += v.iter().filter(|w| w.report.features.peak_to_peak_uv < flat_max).count() as u64;
            gt_fine += v.iter().filter(|w| w.amplitude_uv < FINE_VF_UV).count() as u64;
        }
        let f = |v: &[f64]| median(v).map_or_else(|| "–".to_string(), |m| format!("{m:.0} µV ({} records)", v.len()));
        let _ = writeln!(out, "| {} | {} | {} | {} | {} | {} | {} |", db.name(), f(&nonvf), f(&vf), n_vf, pct(fine, n_vf), pct(flat, n_vf), pct(gt_fine, n_vf));
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Per cudb record, channel 0 (to show whether the nominal calibration is consistent between records):\n");
    let _ = writeln!(out, "| Record | non-VF amplitude (median, µV) | VF amplitude (median, µV) | VF windows called fine VF | VF windows called flatline |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for o in outputs.iter().filter(|o| o.db == Db::Cudb) {
        let Some(stream) = o.streams.first() else { continue };
        let a: Vec<f64> = stream.iter().filter(|w| w.class == Some(TruthClass::CudbUnlabelled) || matches!(w.class, Some(TruthClass::Nsr | TruthClass::OtherNonShockable))).map(|w| w.amplitude_uv).collect();
        let v: Vec<&ScoredWindow> = stream.iter().filter(|w| matches!(w.class, Some(TruthClass::CoarseVf | TruthClass::FineVf))).collect();
        let fine = v.iter().filter(|w| w.report.verdict.reason == Reason::FineVf).count() as u64;
        let flat = v.iter().filter(|w| w.report.verdict.reason == Reason::Asystole).count() as u64;
        let m = |x: Option<f64>| x.map_or_else(|| "–".to_string(), |v| format!("{v:.0}"));
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            o.name,
            m(median(&a)),
            m(median(&v.iter().map(|w| w.amplitude_uv).collect::<Vec<_>>())),
            pct(fine, v.len() as u64),
            pct(flat, v.len() as u64)
        );
    }
    let _ = writeln!(out);
}

fn closing(out: &mut String) {
    let _ = writeln!(out, "## Reproducing this report\n");
    let _ = writeln!(out, "```\ncargo run --release -p aed-eval -- --data <path to data/physionet> --csv <out dir>\n```\n");
    let _ = writeln!(out, "The data are not in the repository; see `firmware/crates/aed-eval/README.md` for where to get them, their licences and the citations they ask for.");
}

/// CSV of every segment (all variants).
#[must_use]
pub fn segments_csv(outputs: &[RecordOutput]) -> String {
    let mut s = String::from("db,record,channel,variant_s,start_s,label,class,amplitude_uv,rate_bpm,est_rate_bpm,railed_fraction,outcome,decided_at_s,windows,decisive_reason,pp_uv,noise_ratio,low_band,tcsc,count,leakage,dominant_hz,concentration\n");
    for o in outputs {
        for seg in &o.segments {
            let w = seg.run.decisive.and_then(|i| seg.run.windows.get(i)).or_else(|| seg.run.windows.last());
            let f = w.map(|w| w.features).unwrap_or_default();
            let _ = writeln!(
                s,
                "{},{},{},{},{:.2},\"{}\",{},{:.1},{},{},{:.4},{},{},{},\"{}\",{:.1},{:.4},{:.4},{:.4},{:.4},{:.4},{:.2},{:.4}",
                seg.db.name(),
                seg.record,
                seg.channel,
                seg.variant_s,
                seg.start_s,
                seg.label.text(),
                seg.class.key(),
                seg.amplitude_uv,
                seg.rate_bpm.map_or_else(String::new, |r| format!("{r:.1}")),
                seg.est_rate_bpm.map_or_else(String::new, |r| format!("{r:.1}")),
                seg.railed_fraction,
                seg.run.outcome.text(),
                seg.run.decided_at_s.map_or_else(String::new, |t| format!("{t:.2}")),
                seg.run.windows.len(),
                w.map_or("", |w| reason_label(w.verdict.reason)),
                f.peak_to_peak_uv,
                f.noise_ratio,
                f.low_band_fraction,
                f.threshold_crossing_fraction,
                f.bandpass_count_fraction,
                f.vf_leakage,
                f.dominant_frequency_hz,
                f.spectral_concentration
            );
        }
    }
    s
}

/// CSV of streaming VF/flutter/VT episodes.
#[must_use]
pub fn episodes_csv(outputs: &[RecordOutput]) -> String {
    let mut s = String::from("db,record,channel,label,onset_s,end_s,delay_s,shockable_at_onset,persistence_s,next_label\n");
    for e in episodes(outputs) {
        let _ = writeln!(
            s,
            "{},{},{},{},{:.2},{:.2},{},{},{},\"{}\"",
            e.db.name(),
            e.record,
            e.channel,
            e.label.text(),
            e.onset,
            e.end,
            e.delay.map_or_else(String::new, |d| format!("{d:.2}")),
            e.shockable_at_onset,
            e.persistence.map_or_else(String::new, |p| format!("{p:.2}")),
            e.next_label.map_or_else(String::new, |l| l.text())
        );
    }
    s
}
