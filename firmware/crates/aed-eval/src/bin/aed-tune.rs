// SPDX-License-Identifier: GPL-3.0-or-later
//! Tuning experiments on the DEVELOPMENT half of the data (see
//! `crates/aed-eval/TUNING_LOG.md`, which records what each printed).
//!
//! ```text
//! cargo run --release -p aed-eval --bin aed-tune -- --data <dir> --records crates/aed-eval/splits/development.txt <experiment>
//! ```
//!
//! Refuses to run on any list other than the development list, so the
//! held-out half cannot be tuned on by accident.
//!
//! DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE.
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use aed_eval::split::{parse_list, sha256_hex};
use aed_eval::stats::{clopper_pearson_lower, percentile};
use aed_eval::truth::TruthClass;
use aed_eval::tune::{
    collect, lopo, score, UNTUNED, verdict_with, Confirm, Lopo, Seg, Tallies, Tally, LENGTHS, OBJECTIVE,
};
use aed_rhythm::{Features, Settings};

/// SHA-256 of `splits/development.txt` (see splits/README.md).
const DEV_SHA256: &str = "597e65c49bc0114f3680c1e3f49cc8d553407606e411f6bce6a3e1714d477903";

const SHOWN: [TruthClass; 7] = [
    TruthClass::CoarseVf,
    TruthClass::Vfl,
    TruthClass::VtRateUnknown,
    TruthClass::Nsr,
    TruthClass::OtherNonShockable,
    TruthClass::Asystole,
    TruthClass::CudbUnlabelled,
];

fn pct(x: u32, n: u32) -> String {
    if n == 0 {
        "–".into()
    } else {
        format!("{:.1}%", 100.0 * f64::from(x) / f64::from(n))
    }
}

fn lcl(x: u32, n: u32) -> String {
    clopper_pearson_lower(u64::from(x), u64::from(n), 0.9).map_or("–".into(), |v| format!("{:.1}%", 100.0 * v))
}

fn median_time(t: &Tally) -> String {
    let v: Vec<f64> = t
        .t_hist
        .iter()
        .enumerate()
        .flat_map(|(s, c)| std::iter::repeat_n(s as f64, *c as usize))
        .collect();
    percentile(&v, 50.0).map_or("–".into(), |m| format!("{m:.0}"))
}

/// Print a per-class table from pooled tallies.
fn table(title: &str, by_class: &BTreeMap<TruthClass, Tally>) {
    table_r(title, by_class, None);
}

/// As `table`, with the number of patients per class.
fn table_r(title: &str, by_class: &BTreeMap<TruthClass, Tally>, recs: Option<&BTreeMap<TruthClass, usize>>) {
    println!("\n**{title}**\n");
    println!("| Class | n | patients | right at 7 s | 8 s | 10 s | 12 s (LCL 90%) | inconclusive 12 s | shocks cancelled in countdown | median decision time (s) |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for c in SHOWN {
        let Some(t) = by_class.get(&c) else { continue };
        if t.n == 0 {
            continue;
        }
        let right = |i: usize| -> u32 {
            if c.should_not_shock() {
                t.n - t.shock.get(i).copied().unwrap_or(0)
            } else {
                t.shock.get(i).copied().unwrap_or(0)
            }
        };
        let cells: Vec<String> = (0..LENGTHS.len()).map(|i| pct(right(i), t.n)).collect();
        println!(
            "| {} | {} | {} | {} | {} | {} | {} ({}) | {} | {} of {} | {} |",
            c.name(),
            t.n,
            recs.and_then(|r| r.get(&c)).map_or("–".into(), ToString::to_string),
            cells.first().cloned().unwrap_or_default(),
            cells.get(1).cloned().unwrap_or_default(),
            cells.get(2).cloned().unwrap_or_default(),
            cells.get(3).cloned().unwrap_or_default(),
            lcl(right(3), t.n),
            pct(t.inconclusive12(), t.n),
            t.cancelled,
            t.shocks12(),
            median_time(t)
        );
    }
}

fn patients_per_class(t: &Tallies) -> BTreeMap<TruthClass, usize> {
    let mut m: BTreeMap<TruthClass, usize> = BTreeMap::new();
    for ((_, c), x) in t {
        if x.n > 0 {
            *m.entry(*c).or_default() += 1;
        }
    }
    m
}

fn all_classes(t: &Tallies, only: Option<&dyn Fn(usize) -> bool>) -> BTreeMap<TruthClass, Tally> {
    let mut m: BTreeMap<TruthClass, Tally> = BTreeMap::new();
    for ((p, c), x) in t {
        if only.is_none_or(|f| f(*p)) {
            m.entry(*c).or_default().merge(x);
        }
    }
    m
}

struct Ctx {
    segs: Vec<Seg>,
    patients: Vec<String>,
}

impl Ctx {
    fn channel(&self, ch: usize) -> Vec<Seg> {
        self.segs.iter().filter(|s| s.channel == ch).cloned().collect()
    }
}

/// One named candidate.
struct Cand {
    name: String,
    s: Settings,
    c: Confirm,
}

fn run_lopo(ctx: &Ctx, title: &str, cands: &[Cand]) -> Lopo {
    run_lopo_with(ctx, title, cands, &OBJECTIVE)
}

fn run_lopo_with(ctx: &Ctx, title: &str, cands: &[Cand], objective: &aed_eval::tune::Objective) -> Lopo {
    let tallies: Vec<Tallies> = cands.iter().map(|k| score(&ctx.segs, &verdict_with(k.s), &k.c)).collect();
    let reference = score(&ctx.segs, &verdict_with(UNTUNED), &Confirm::UNTUNED);
    let l = lopo(&tallies, &reference, ctx.patients.len(), objective);
    println!("\n### {title}\n");
    println!("{} candidates, leave-one-patient-out over {} patients.", cands.len(), ctx.patients.len());
    let mut chosen: Vec<_> = l.chosen.iter().collect();
    chosen.sort_by(|a, b| b.1.cmp(a.1));
    let names: Vec<String> = chosen
        .iter()
        .map(|(i, n)| format!("{} ×{n}", cands.get(**i).map_or("?", |c| c.name.as_str())))
        .collect();
    println!("Chosen in the folds: {}.", names.join("; "));
    if let Some(i) = l.full_choice {
        println!("Chosen on all development patients: **{}**.", cands.get(i).map_or("?", |c| c.name.as_str()));
        if let Some(t) = tallies.get(i) {
            table_r("Resubstitution (chosen and scored on all development data; optimistic)", &all_classes(t, None), Some(&patients_per_class(t)));
        }
    } else {
        println!("No candidate met the constraints on all development patients.");
    }
    // Cross-validated, both channels and per channel.
    for (title, ch) in [("both channels", None), ("channel 0", Some(0_usize)), ("channel 1", Some(1))] {
        let mut by: Tallies = Tallies::new();
        for (p, pick) in l.per_patient.iter().enumerate() {
            let Some(k) = pick.and_then(|i| cands.get(i)) else { continue };
            let segs: Vec<Seg> = ctx.segs.iter().filter(|s| s.patient == p && ch.is_none_or(|c| s.channel == c)).cloned().collect();
            for ((pp, c), x) in score(&segs, &verdict_with(k.s), &k.c) {
                by.entry((pp, c)).or_default().merge(&x);
            }
        }
        table_r(
            &format!("Cross-validated, {title} (each patient scored with the choice made without it)"),
            &all_classes(&by, None),
            Some(&patients_per_class(&by)),
        );
    }
    l
}

fn untuned() -> Cand {
    Cand { name: "untuned".into(), s: UNTUNED, c: Confirm::UNTUNED }
}

fn exp_baseline(ctx: &Ctx) {
    let k = untuned();
    let t = score(&ctx.segs, &verdict_with(k.s), &k.c);
    table("Untuned analyser, development data, both channels", &all_classes(&t, None));
    for ch in [0, 1] {
        let segs = ctx.channel(ch);
        let t = score(&segs, &verdict_with(k.s), &k.c);
        table(&format!("Untuned analyser, development data, channel {ch}"), &all_classes(&t, None));
    }
}

/// Quantiles of a feature by class over all stored windows of the first
/// 12 s of each segment.
fn exp_features(ctx: &Ctx) {
    type Getter = fn(&Features) -> f32;
    let feats: [(&str, Getter); 13] = [
        ("peak-to-peak", |f| f.peak_to_peak_uv),
        ("median 1-s peak-to-peak", |f| f.second_pp_median_uv),
        ("rms", |f| f.rms_uv),
        ("noise ratio", |f| f.noise_ratio),
        ("low-band fraction", |f| f.low_band_fraction),
        ("TCSC", |f| f.threshold_crossing_fraction),
        ("count", |f| f.bandpass_count_fraction),
        ("leakage", |f| f.vf_leakage),
        ("dominant Hz", |f| f.dominant_frequency_hz),
        ("concentration", |f| f.spectral_concentration),
        ("clipped fraction", |f| f.clipped_fraction),
        ("saturated fraction", |f| f.saturated_fraction),
        ("count (VF-side windows only)", |f| f.bandpass_count_fraction),
    ];
    for (name, g) in feats {
        println!("\n**{name}**\n\n| Class | windows | 1% | 5% | 25% | 50% | 75% | 95% | 99% |\n|---|---|---|---|---|---|---|---|---|");
        for c in SHOWN {
            let v: Vec<f64> = ctx
                .segs
                .iter()
                .filter(|s| s.class == c)
                .flat_map(|s| s.wins.iter().take(8))
                .map(|w| f64::from(g(&w.f)))
                .collect();
            if v.is_empty() {
                continue;
            }
            let q: Vec<String> = [1.0, 5.0, 25.0, 50.0, 75.0, 95.0, 99.0]
                .iter()
                .map(|p| percentile(&v, *p).map_or("–".into(), |x| format!("{x:.3}")))
                .collect();
            println!("| {} | {} | {} |", c.name(), v.len(), q.join(" | "));
        }
    }
}

/// E1: the band-pass count threshold alone.
fn exp_count(ctx: &Ctx) {
    let mut cands = Vec::new();
    // Strictest first (ties go to the earlier candidate).
    for i in (8..=18).rev() {
        let v = f32::from(i as u8) / 100.0;
        cands.push(Cand { name: format!("count > {v:.2}"), s: Settings { count_vf_min: v, ..UNTUNED }, c: Confirm::UNTUNED });
    }
    let _ = run_lopo(ctx, "E1: band-pass count threshold only (everything else untuned)", &cands);
}

/// E2: count, leakage and dominant-frequency lower limit together.
fn exp_thresholds(ctx: &Ctx) {
    let mut cands = Vec::new();
    for count in [0.16_f32, 0.14, 0.12, 0.10] {
        for leak in [0.625_f32, 0.65, 0.70] {
            for df in [2.4_f32, 2.0, 1.6] {
                cands.push(Cand {
                    name: format!("count > {count:.2}, leakage < {leak:.3}, DF > {df:.1} Hz"),
                    s: Settings { count_vf_min: count, leakage_vf_max: leak, dominant_vf_min_hz: df, ..UNTUNED },
                    c: Confirm::UNTUNED,
                });
            }
        }
    }
    let _ = run_lopo(ctx, "E2: count x leakage x dominant-frequency limit (confirmation untuned)", &cands);
}

/// Look at every asystole segment (there are few).
fn exp_asystole(ctx: &Ctx) {
    let v = verdict_with(UNTUNED);
    println!("\n| record | ch | start | GT amp | outcome | per-window: pp / med1s-pp / rms / noise / low / TCSC / count / leak / DF / verdict |\n|---|---|---|---|---|---|");
    for s in ctx.segs.iter().filter(|s| s.class == TruthClass::Asystole) {
        let p = aed_eval::tune::replay(s, &v, &Confirm::UNTUNED);
        let w: Vec<String> = s
            .wins
            .iter()
            .take(8)
            .map(|w| {
                let f = &w.f;
                format!(
                    "{:.0}/{:.0}/{:.0}/{:.2}/{:.2}/{:.2}/{:.2}/{:.2}/{:.1}/{}",
                    f.peak_to_peak_uv,
                    f.second_pp_median_uv,
                    f.rms_uv,
                    f.noise_ratio,
                    f.low_band_fraction,
                    f.threshold_crossing_fraction,
                    f.bandpass_count_fraction,
                    f.vf_leakage,
                    f.dominant_frequency_hz,
                    aed_rhythm::reason_label(v(f).reason).chars().take(8).collect::<String>()
                )
            })
            .collect();
        println!("| {} | {} | {:.0} | {:.0} | {:?} | {} |", s.record, s.channel, s.start_s, s.amplitude_uv, p.decision, w.join(" ; "));
    }
}

/// Flatline measure: which windows each candidate rule would call asystole.
fn exp_flatline(ctx: &Ctx) {
    let wins = |c: TruthClass| -> Vec<(&Seg, &aed_eval::tune::Win)> {
        ctx.segs.iter().filter(|s| s.class == c).flat_map(|s| s.wins.iter().take(8).map(move |w| (s, w))).collect()
    };
    type Rule = Box<dyn Fn(&Features) -> bool>;
    let mut rules: Vec<(String, Rule)> = vec![("filtered peak-to-peak < 100 µV (untuned)".into(), Box::new(|f: &Features| f.peak_to_peak_uv < 100.0))];
    for t in [120.0_f32, 150.0] {
        rules.push((format!("median 1-s pp < {t:.0} µV"), Box::new(move |f: &Features| f.second_pp_median_uv < t)));
    }
    for t in [20.0_f32, 25.0, 30.0, 35.0, 40.0, 50.0] {
        rules.push((format!("rms < {t:.0} µV"), Box::new(move |f: &Features| f.rms_uv < t)));
    }
    for (t, l) in [(25.0_f32, 0.5_f32), (25.0, 0.6), (30.0, 0.6), (30.0, 0.7), (40.0, 0.7), (40.0, 0.75), (50.0, 0.75)] {
        rules.push((format!("rms < {t:.0} µV and leakage > {l:.2}"), Box::new(move |f: &Features| f.rms_uv < t && f.vf_leakage > l)));
    }
    let classes = [TruthClass::Asystole, TruthClass::CoarseVf, TruthClass::Vfl, TruthClass::VtRateUnknown, TruthClass::Nsr, TruthClass::OtherNonShockable];
    println!("\n| Rule | {} | shockable-class windows flagged (records) |", classes.iter().map(|c| c.name()).collect::<Vec<_>>().join(" | "));
    println!("|---|{}---|", "---|".repeat(classes.len()));
    for (name, r) in &rules {
        let mut cells = Vec::new();
        let mut bad: BTreeMap<String, u32> = BTreeMap::new();
        for c in classes {
            let w = wins(c);
            let n = w.iter().filter(|(_, w)| r(&w.f)).count();
            if c.should_shock() || c == TruthClass::VtRateUnknown {
                for (s, w) in &w {
                    if r(&w.f) {
                        *bad.entry(format!("{}/{}", s.record, s.channel)).or_default() += 1;
                    }
                }
            }
            cells.push(format!("{:.1}% of {}", 100.0 * n as f64 / w.len().max(1) as f64, w.len()));
        }
        let b: Vec<String> = bad.iter().map(|(k, v)| format!("{k}:{v}")).collect();
        println!("| {name} | {} | {} |", cells.join(" | "), b.join(" "));
    }
    // Per asystole patient, rms quantiles.
    println!("\n| Asystole record/ch | windows | rms 10% | 50% | 90% | med1s-pp 50% | leakage 50% |\n|---|---|---|---|---|---|---|");
    let mut by: BTreeMap<String, Vec<Features>> = BTreeMap::new();
    for (s, w) in wins(TruthClass::Asystole) {
        by.entry(format!("{}/{}", s.record, s.channel)).or_default().push(w.f);
    }
    for (k, v) in by {
        let q = |g: fn(&Features) -> f32, p: f64| {
            let x: Vec<f64> = v.iter().map(|f| f64::from(g(f))).collect();
            percentile(&x, p).map_or("–".into(), |y| format!("{y:.0}"))
        };
        let lk: Vec<f64> = v.iter().map(|f| f64::from(f.vf_leakage)).collect();
        println!("| {k} | {} | {} | {} | {} | {} | {:.2} |", v.len(), q(|f| f.rms_uv, 10.0), q(|f| f.rms_uv, 50.0), q(|f| f.rms_uv, 90.0), q(|f| f.second_pp_median_uv, 50.0), percentile(&lk, 50.0).unwrap_or(0.0));
    }
    // Synthetic fine VF and asystole, for comparison.
    use aed_rhythm::synth::{Rhythm, Synth};
    println!("\n| Synthetic | rms range | leakage range |\n|---|---|---|");
    for (name, r) in [("fine VF 120 µV", Rhythm::fine_vf()), ("asystole", Rhythm::Asystole), ("VF 200 µV", Rhythm::VentricularFibrillation { amplitude_uv: 200.0 })] {
        let (mut lo, mut hi, mut llo, mut lhi) = (f32::MAX, 0.0_f32, f32::MAX, 0.0_f32);
        for seed in 1..=10 {
            let x: Vec<i16> = Synth::new(r, seed).take(250 * 12).collect();
            for w in aed_eval::tune::windows_of(&x, &[]) {
                lo = lo.min(w.f.rms_uv);
                hi = hi.max(w.f.rms_uv);
                llo = llo.min(w.f.vf_leakage);
                lhi = lhi.max(w.f.vf_leakage);
            }
        }
        println!("| {name} | {lo:.0}–{hi:.0} | {llo:.2}–{lhi:.2} |");
    }
}

/// Clipping: the flat-top detector against the recorder-limit ground truth.
fn exp_clip(ctx: &Ctx) {
    let all: Vec<(&Seg, &aed_eval::tune::Win)> = ctx.segs.iter().flat_map(|s| s.wins.iter().take(8).map(move |w| (s, w))).collect();
    println!("\n| Detector rule | truly clipped windows (>1% at recorder limit) caught | windows with no recorder clipping flagged | coarse VF windows flagged |\n|---|---|---|---|");
    for thr in [0.0_f32, 0.005, 0.01, 0.02, 0.05] {
        let truly: Vec<_> = all.iter().filter(|(_, w)| w.railed > 0.01).collect();
        let clean: Vec<_> = all.iter().filter(|(_, w)| w.railed == 0.0).collect();
        let caught = truly.iter().filter(|(_, w)| w.f.clipped_fraction > thr).count();
        let fa = clean.iter().filter(|(_, w)| w.f.clipped_fraction > thr).count();
        let vf: Vec<_> = all.iter().filter(|(s, _)| s.class == TruthClass::CoarseVf).collect();
        let vff = vf.iter().filter(|(_, w)| w.f.clipped_fraction > thr).count();
        println!("| clipped fraction > {thr} | {caught} of {} | {fa} of {} | {vff} of {} |", truly.len(), clean.len(), vf.len());
    }
    let mut by: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    for (s, w) in &all {
        let e = by.entry(format!("{}/{} {}", s.record, s.channel, s.class.name())).or_default();
        if w.railed > 0.01 {
            e.0 += 1;
        }
        if w.f.clipped_fraction > 0.01 {
            e.1 += 1;
        }
        e.2 += 1;
    }
    println!("\n| record/ch class | windows >1% at recorder limit | windows detector >1% | windows |\n|---|---|---|---|");
    for (k, v) in by.iter().filter(|(_, v)| v.0 > 0 || v.1 > 0) {
        println!("| {k} | {} | {} | {} |", v.0, v.1, v.2);
    }
}

/// The untuned thresholds plus the two repaired checks (T-2 clipping, T-3 flatline).
const B1: Settings = Settings {
    max_clipped_fraction: 0.01,
    asystole_max_rms_uv: 25.0,
    asystole_min_leakage: 0.6,
    ..UNTUNED
};

fn exp_checks(ctx: &Ctx) {
    for (name, s) in [
        ("untuned", UNTUNED),
        ("+ clipping check (flat tops > 1 %)", Settings { max_clipped_fraction: 0.01, ..UNTUNED }),
        ("+ clipping check (flat tops > 5 %)", Settings { max_clipped_fraction: 0.05, ..UNTUNED }),
        ("+ flatline rms rule", Settings { asystole_max_rms_uv: 25.0, asystole_min_leakage: 0.6, ..UNTUNED }),
        ("B1 = both (1 %)", B1),
    ] {
        let t = score(&ctx.segs, &verdict_with(s), &Confirm::UNTUNED);
        table(name, &all_classes(&t, None));
    }
}

/// E3: confirmation rules, jointly with the count and DF thresholds.
fn exp_confirm(ctx: &Ctx) {
    let rules = [
        ("3 of 3", Confirm::UNTUNED),
        ("3 of 4", Confirm { k: 3, n: 4, ..Confirm::UNTUNED }),
        ("3 of 5", Confirm { k: 3, n: 5, ..Confirm::UNTUNED }),
        ("2 of 2", Confirm { k: 2, n: 2, ..Confirm::UNTUNED }),
        ("2 of 3", Confirm { k: 2, n: 3, ..Confirm::UNTUNED }),
    ];
    let mut cands = Vec::new();
    for count in [0.18_f32, 0.16, 0.14, 0.12] {
        for df in [2.4_f32, 1.6] {
            for (rn, r) in rules {
                for m in [1_u8, 2] {
                    for hold in [false, true] {
                        cands.push(Cand {
                            name: format!("count > {count:.2}, DF > {df:.1}, {rn}, no-shock after {m}, hold {hold}"),
                            s: Settings { count_vf_min: count, dominant_vf_min_hz: df, ..B1 },
                            c: Confirm { nonshock_m: m, hold_on_disagreement: hold, ..r },
                        });
                    }
                }
            }
        }
    }
    let _ = run_lopo(ctx, "E3: confirmation rule x count x DF (on B1)", &cands);
    let loose = aed_eval::tune::Objective { asys_slack: 1000, ..OBJECTIVE };
    let _ = run_lopo_with(ctx, "E3b: same, WITHOUT the asystole constraint (what-if only)", &cands, &loose);
}

/// Where coarse-VF windows and segments fail under one candidate.
fn misses(ctx: &Ctx, s: Settings, c: Confirm) {
    let v = verdict_with(s);
    let mut reasons: BTreeMap<&str, u32> = BTreeMap::new();
    let mut fails: BTreeMap<&str, u32> = BTreeMap::new();
    let mut n = 0;
    for seg in ctx.segs.iter().filter(|x| x.class == TruthClass::CoarseVf) {
        for w in seg.wins.iter().take(8) {
            n += 1;
            let f = &w.f;
            *reasons.entry(aed_rhythm::reason_label(v(f).reason)).or_default() += 1;
            let checks: [(&str, bool); 9] = [
                ("TCSC", f.threshold_crossing_fraction > s.tcsc_vf_min),
                ("count", f.bandpass_count_fraction > s.count_vf_min),
                ("leakage", f.vf_leakage < s.leakage_vf_max),
                ("DF low", f.dominant_frequency_hz > s.dominant_vf_min_hz),
                ("DF high", f.dominant_frequency_hz < s.dominant_vf_max_hz),
                ("concentration", f.spectral_concentration > s.concentration_vf_min),
                ("low band", f.low_band_fraction <= s.max_low_band_fraction),
                ("size", f.peak_to_peak_uv <= s.max_plausible_pp_uv),
                ("clip", f.clipped_fraction <= s.max_clipped_fraction),
            ];
            for (k, ok) in checks {
                if !ok {
                    *fails.entry(k).or_default() += 1;
                }
            }
        }
    }
    println!("\nCoarse-VF windows ({n}) by verdict: {reasons:?}\nConditions failed (any): {fails:?}");
    let mut by: BTreeMap<String, Tally> = BTreeMap::new();
    for seg in ctx.segs.iter().filter(|x| x.class == TruthClass::CoarseVf) {
        by.entry(format!("{}/{}", seg.record, seg.channel)).or_default().add(&aed_eval::tune::replay(seg, &v, &c));
    }
    let rows: Vec<String> = by.iter().map(|(k, t)| format!("{k} {}/{}", t.shocks12(), t.n)).collect();
    println!("Per record (shocks/segments, 12 s): {}", rows.join(", "));
}

fn exp_misses(ctx: &Ctx) {
    println!("\n#### untuned");
    misses(ctx, UNTUNED, Confirm::UNTUNED);
    println!("\n#### E3 choice");
    misses(ctx, Settings { count_vf_min: 0.16, dominant_vf_min_hz: 1.6, ..B1 }, Confirm { k: 3, n: 5, nonshock_m: 1, hold_on_disagreement: true });
}

/// Window-level potential of a linear score over a plain-threshold rule:
/// at the same share of non-shockable windows called shockable, how many
/// coarse-VF windows does each catch? Leave-one-patient-out for the score.
fn exp_linear(ctx: &Ctx) {
    let base = Settings { count_vf_min: 0.16, dominant_vf_min_hz: 1.6, ..B1 };
    let v = verdict_with(base);
    // Windows that reach the family votes (not stopped by quality checks).
    let mut rows: Vec<(usize, bool, Features, bool)> = Vec::new();
    for s in &ctx.segs {
        let y = match s.class {
            TruthClass::CoarseVf => true,
            c if c.should_not_shock() => false,
            _ => continue,
        };
        for w in s.wins.iter().take(8) {
            let r = v(&w.f);
            use aed_rhythm::Reason as R;
            if matches!(r.reason, R::BothMethodsShockable | R::NeitherMethodShockable | R::MethodsDisagree) {
                rows.push((s.patient, y, w.f, r.decision == aed_types::RhythmDecision::Shockable));
            }
        }
    }
    let n_vf = rows.iter().filter(|r| r.1).count();
    let n_ns = rows.len() - n_vf;
    let rule_tp = rows.iter().filter(|r| r.1 && r.3).count();
    let rule_fp = rows.iter().filter(|r| !r.1 && r.3).count();
    println!("\nWindows reaching the family votes: {n_vf} coarse VF, {n_ns} non-shockable. Threshold rule (E3 choice): {rule_tp} VF ({:.1}%), {rule_fp} non-shockable ({:.3}%).", 100.0 * rule_tp as f64 / n_vf as f64, 100.0 * rule_fp as f64 / n_ns as f64);
    type Fx = fn(&Features) -> Vec<f64>;
    let sets: [(&str, Fx); 4] = [
        ("time family: TCSC, count", |f| vec![1.0, f64::from(f.threshold_crossing_fraction), f64::from(f.bandpass_count_fraction)]),
        ("frequency family: leakage, concentration", |f| vec![1.0, f64::from(f.vf_leakage), f64::from(f.spectral_concentration)]),
        ("both families: TCSC, count, leakage, concentration", |f| vec![1.0, f64::from(f.threshold_crossing_fraction), f64::from(f.bandpass_count_fraction), f64::from(f.vf_leakage), f64::from(f.spectral_concentration)]),
        ("all six incl. low band and DF", |f| vec![1.0, f64::from(f.threshold_crossing_fraction), f64::from(f.bandpass_count_fraction), f64::from(f.vf_leakage), f64::from(f.spectral_concentration), f64::from(f.low_band_fraction), f64::from(f.dominant_frequency_hz.min(9.0))]),
    ];
    for (name, fx) in sets {
        // LOPO: score each patient's windows with a model fitted without it.
        let mut scored: Vec<(bool, f64)> = Vec::new();
        for p in 0..ctx.patients.len() {
            let train: Vec<&(usize, bool, Features, bool)> = rows.iter().filter(|r| r.0 != p).collect();
            if train.len() == rows.len() {
                continue;
            }
            let x: Vec<Vec<f64>> = train.iter().map(|r| fx(&r.2)).collect();
            let y: Vec<bool> = train.iter().map(|r| r.1).collect();
            let nv = y.iter().filter(|b| **b).count() as f64;
            let nn = y.len() as f64 - nv;
            let w: Vec<f64> = y.iter().map(|b| if *b { 0.5 / nv } else { 0.5 / nn }).map(|x| x * 1000.0).collect();
            let Some(beta) = aed_eval::tune::logistic(&x, &y, &w, 1e-3) else { continue };
            for r in rows.iter().filter(|r| r.0 == p) {
                let z: f64 = fx(&r.2).iter().zip(&beta).map(|(a, b)| a * b).sum();
                scored.push((r.1, z));
            }
        }
        // Threshold giving the same non-shockable rate as the rule.
        let mut ns: Vec<f64> = scored.iter().filter(|s| !s.0).map(|s| s.1).collect();
        ns.sort_by(|a, b| b.total_cmp(a));
        let thr = ns.get(rule_fp).copied().unwrap_or(f64::MAX);
        let tp = scored.iter().filter(|s| s.0 && s.1 > thr).count();
        println!("- {name}: at the rule's non-shockable rate, catches {tp} VF windows ({:.1}%) (LOPO)", 100.0 * tp as f64 / n_vf as f64);
    }
}

/// The low-band ("compression") rule: can a reformulation keep rejecting
/// the made-up compression artefact while no longer rejecting real VF?
fn exp_lowband(ctx: &Ctx) {
    for (count, df) in [(0.18_f32, 2.4_f32), (0.16, 2.4), (0.12, 2.4), (0.16, 2.0), (0.16, 1.6)] {
        println!("\n#### count > {count}, DF > {df} Hz");
        lowband_with(ctx, Settings { count_vf_min: count, dominant_vf_min_hz: df, ..B1 });
    }
}

fn lowband_with(ctx: &Ctx, base: Settings) {
    use aed_rhythm::synth::{Rhythm, Synth};
    // Synthetic compression windows, wide range of rates and sizes.
    let mut cpr: Vec<(String, Features)> = Vec::new();
    for (rn, r) in [("sinus 50", Rhythm::sinus(50.0)), ("sinus 80", Rhythm::sinus(80.0)), ("sinus 120", Rhythm::sinus(120.0)), ("asystole", Rhythm::Asystole), ("VF", Rhythm::coarse_vf())] {
        for rate in [80.0_f32, 100.0, 110.0, 120.0, 140.0] {
            for amp in [500.0_f32, 800.0, 1500.0, 3000.0] {
                for seed in [1_u32, 2, 3] {
                    let x: Vec<i16> = Synth::new(r, seed).with_compressions(amp, rate).take(250 * 12).collect();
                    for w in aed_eval::tune::windows_of(&x, &[]) {
                        cpr.push((format!("{rn} + CPR {rate}/min"), w.f));
                    }
                }
            }
        }
    }
    let vf: Vec<Features> = ctx.segs.iter().filter(|s| s.class == TruthClass::CoarseVf).flat_map(|s| s.wins.iter().take(8).map(|w| w.f)).collect();
    let q = |v: &[Features], g: fn(&Features) -> f32| -> String {
        let x: Vec<f64> = v.iter().map(|f| f64::from(g(f))).collect();
        format!("{:.2}–{:.2}–{:.2}", percentile(&x, 5.0).unwrap_or(0.0), percentile(&x, 50.0).unwrap_or(0.0), percentile(&x, 95.0).unwrap_or(0.0))
    };
    let vf_low: Vec<Features> = vf.iter().filter(|f| f.low_band_fraction > 0.5).copied().collect();
    let cpr_non_vf: Vec<Features> = cpr.iter().filter(|(n, _)| !n.starts_with("VF")).map(|x| x.1).collect();
    println!("\nFeature 5%–50%–95% for real coarse-VF windows with low-band > 0.5 ({}) vs made-up compressions over sinus/asystole ({}):\n", vf_low.len(), cpr_non_vf.len());
    println!("| feature | real VF, low-band > 0.5 | made-up compressions (no VF) |\n|---|---|---|");
    type G = fn(&Features) -> f32;
    for (n, g) in [
        ("low-band fraction", (|f: &Features| f.low_band_fraction) as G),
        ("dominant Hz", |f| f.dominant_frequency_hz),
        ("concentration", |f| f.spectral_concentration),
        ("leakage", |f| f.vf_leakage),
        ("TCSC", |f| f.threshold_crossing_fraction),
        ("count", |f| f.bandpass_count_fraction),
        ("peak-to-peak µV", |f| f.peak_to_peak_uv),
    ] {
        println!("| {n} | {} | {} |", q(&vf_low, g), q(&cpr_non_vf, g));
    }
    type Rule = Box<dyn Fn(&Features) -> bool>;
    let rules: Vec<(&str, Rule)> = vec![
        ("low-band > 0.5 (current)", Box::new(|f: &Features| f.low_band_fraction > 0.5)),
        ("low-band > 0.6", Box::new(|f: &Features| f.low_band_fraction > 0.6)),
        ("low-band > 0.7", Box::new(|f: &Features| f.low_band_fraction > 0.7)),
        ("low-band > 0.5 and concentration > 0.6", Box::new(|f: &Features| f.low_band_fraction > 0.5 && f.spectral_concentration > 0.6)),
        ("low-band > 0.5 and DF < 2.4", Box::new(|f: &Features| f.low_band_fraction > 0.5 && f.dominant_frequency_hz < 2.4)),
        ("low-band > 0.3 and DF < 2.4", Box::new(|f: &Features| f.low_band_fraction > 0.3 && f.dominant_frequency_hz < 2.4)),
        ("no rule", Box::new(|_: &Features| false)),
    ];
    let shock = |f: &Features, art: &dyn Fn(&Features) -> bool| -> bool {
        let s = Settings { max_low_band_fraction: 1.0, ..base };
        !art(f) && aed_rhythm::classify_with(f, &s).decision == aed_types::RhythmDecision::Shockable
    };
    println!("\n| Formulation (thresholds as in the heading) | real coarse-VF windows shockable | made-up compression windows (no VF) shockable | made-up compression over VF shockable | NSR + other windows flagged as artefact |\n|---|---|---|---|---|");
    let ns: Vec<Features> = ctx.segs.iter().filter(|s| s.class.should_not_shock()).flat_map(|s| s.wins.iter().take(8).map(|w| w.f)).collect();
    for (n, r) in &rules {
        let a = vf.iter().filter(|f| shock(f, r.as_ref())).count();
        let b = cpr_non_vf.iter().filter(|f| shock(f, r.as_ref())).count();
        let cv: Vec<&Features> = cpr.iter().filter(|(n, _)| n.starts_with("VF")).map(|x| &x.1).collect();
        let c = cv.iter().filter(|f| shock(f, r.as_ref())).count();
        let d = ns.iter().filter(|f| r(f)).count();
        println!("| {n} | {a} of {} | {b} of {} | {c} of {} | {:.1}% |", vf.len(), cpr_non_vf.len(), cv.len(), 100.0 * d as f64 / ns.len() as f64);
        if n.contains("current") {
            let mut by: BTreeMap<&str, u32> = BTreeMap::new();
            for (k, f) in cpr.iter().filter(|(k, _)| !k.starts_with("VF")) {
                if shock(f, r.as_ref()) {
                    *by.entry(k.as_str()).or_default() += 1;
                }
            }
            println!("|   made-up compression windows shockable under the current rule, by case: {by:?} | | | | |");
        }
    }
}

/// E4: the final joint choice, with the dominant-frequency limit kept at
/// 2.4 Hz (T-6) and the low-band rule kept as it is (T-7).
fn e4_cands() -> Vec<Cand> {
    let rules = [
        ("3 of 3", 3_u8, 3_u8),
        ("3 of 4", 3, 4),
        ("3 of 5", 3, 5),
        ("2 of 3", 2, 3),
        ("2 of 2", 2, 2),
    ];
    let mut cands = Vec::new();
    for count in [0.18_f32, 0.16, 0.14, 0.12, 0.10] {
        for leak in [0.625_f32, 0.65, 0.70] {
            for size in [8_000.0_f32, 12_000.0, 16_000.0] {
                for (rn, k, n) in rules {
                    for m in [1_u8, 2] {
                        for hold in [false, true] {
                            cands.push(Cand {
                                name: format!("count > {count:.2}, leakage < {leak:.3}, size ≤ {size:.0} µV, {rn}, no-shock after {m}, hold {hold}"),
                                s: Settings { count_vf_min: count, leakage_vf_max: leak, max_plausible_pp_uv: size, ..B1 },
                                c: Confirm { k, n, nonshock_m: m, hold_on_disagreement: hold },
                            });
                        }
                    }
                }
            }
        }
    }
    cands
}

/// E5: TCSC and concentration limits, with the E4 structure fixed.
fn exp_e5(ctx: &Ctx) {
    let mut cands = Vec::new();
    for count in [0.18_f32, 0.16, 0.14] {
        for tcsc in [0.45_f32, 0.40] {
            for conc in [0.40_f32, 0.35] {
                cands.push(Cand {
                    name: format!("count > {count:.2}, TCSC > {tcsc:.2}, concentration > {conc:.2}"),
                    s: Settings { count_vf_min: count, tcsc_vf_min: tcsc, concentration_vf_min: conc, max_plausible_pp_uv: 12_000.0, ..B1 },
                    c: Confirm { k: 3, n: 5, nonshock_m: 1, hold_on_disagreement: false },
                });
            }
        }
    }
    let _ = run_lopo(ctx, "E5: TCSC and concentration limits (E4c structure fixed: 3 of 5, no hold)", &cands);
}

/// The frozen settings (Settings::DEFAULT), resubstitution and replay check.
fn exp_frozen(ctx: &Ctx) {
    let (n, bad) = aed_eval::tune::check_replay(&ctx.segs);
    println!("\nReplay check: {n} windows compared with the real analyser, {bad} differ.");
    let s = Settings::DEFAULT;
    let t = score(&ctx.segs, &verdict_with(s), &Confirm::of(&s));
    table("Frozen settings, development data, both channels (resubstitution)", &all_classes(&t, None));
    for ch in [0, 1] {
        let segs = ctx.channel(ch);
        let t = score(&segs, &verdict_with(s), &Confirm::of(&s));
        table(&format!("Frozen settings, development data, channel {ch} (resubstitution)"), &all_classes(&t, None));
    }
}

/// Made-up compressions over non-shockable rhythms, whole 12 s analyses,
/// with the untuned and the frozen analyser: how many end in "shock"?
fn exp_synth_cpr() {
    use aed_rhythm::synth::{Rhythm, Synth};
    use aed_types::{RhythmAnalyzer, RhythmDecision};
    println!("\n| Made-up case | analyses | untuned: shock | frozen: shock | frozen but DF > 1.6 Hz: shock |\n|---|---|---|---|---|");
    for (rn, r) in [("sinus 50", Rhythm::sinus(50.0)), ("sinus 80", Rhythm::sinus(80.0)), ("sinus 120", Rhythm::sinus(120.0)), ("asystole", Rhythm::Asystole)] {
        for rate in [80.0_f32, 100.0, 110.0, 120.0, 140.0, 150.0] {
            let mut n = 0;
            let mut shocks = [0; 3];
            for amp in [500.0_f32, 800.0, 1500.0, 3000.0] {
                for seed in 1..=5_u32 {
                    n += 1;
                    for (i, s) in [UNTUNED, Settings::DEFAULT, Settings { dominant_vf_min_hz: 1.6, ..Settings::DEFAULT }].into_iter().enumerate() {
                        let mut a = aed_rhythm::Analyzer::with_settings(s);
                        let mut src = Synth::new(r, seed).with_compressions(amp, rate);
                        for _ in 0..250 * 12 {
                            if a.push(src.next_sample()) == Some(RhythmDecision::Shockable) {
                                if let Some(x) = shocks.get_mut(i) {
                                    *x += 1;
                                }
                                break;
                            }
                        }
                    }
                }
            }
            println!("| {rn} + compressions {rate}/min | {n} | {} | {} | {} |", shocks[0], shocks[1], shocks[2]);
        }
    }
}

fn exp_final(ctx: &Ctx) {
    let cands = e4_cands();
    let _ = run_lopo(ctx, "E4: final joint choice (DF limit 2.4 Hz, low-band rule unchanged, on B1)", &cands);
}

/// E4c: as E4 but without the countdown hold, which broke a simulator
/// safety invariant (a shock delivered 2.5 s after VF had turned into a
/// normal rhythm; aed-sim scenario vf-converts-during-countdown).
fn exp_final_no_hold(ctx: &Ctx) {
    let cands: Vec<Cand> = e4_cands().into_iter().filter(|c| !c.c.hold_on_disagreement).collect();
    let _ = run_lopo(ctx, "E4c: as E4, countdown hold not allowed", &cands);
}

fn main() -> ExitCode {
    let mut data: Option<PathBuf> = None;
    let mut records: Option<PathBuf> = None;
    let mut threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut exps: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--data" => data = args.next().map(PathBuf::from),
            "--records" => records = args.next().map(PathBuf::from),
            "--threads" => threads = args.next().and_then(|t| t.parse().ok()).unwrap_or(threads),
            other => exps.push(other.to_string()),
        }
    }
    let (Some(data), Some(records)) = (data, records) else {
        eprintln!("usage: aed-tune --data <dir> --records <development list> <experiment>...");
        return ExitCode::FAILURE;
    };
    let Ok(bytes) = std::fs::read(&records) else {
        eprintln!("cannot read {}", records.display());
        return ExitCode::FAILURE;
    };
    if sha256_hex(&bytes) != DEV_SHA256 {
        eprintln!("refusing: {} is not the development list (SHA-256 mismatch). Tuning uses only development data.", records.display());
        return ExitCode::FAILURE;
    }
    let Ok(list) = std::str::from_utf8(&bytes).map_err(|e| e.to_string()).and_then(parse_list) else {
        eprintln!("bad list");
        return ExitCode::FAILURE;
    };
    let (segs, patients) = match collect(&data, &list, threads) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!("{} segments from {} patients", segs.len(), patients.len());
    let ctx = Ctx { segs, patients };
    for e in &exps {
        match e.as_str() {
            "baseline" => exp_baseline(&ctx),
            "features" => exp_features(&ctx),
            "count" => exp_count(&ctx),
            "checks" => exp_checks(&ctx),
            "final" => exp_final(&ctx),
            "frozen" => exp_frozen(&ctx),
            "synth-cpr" => exp_synth_cpr(),
            "final-no-hold" => exp_final_no_hold(&ctx),
            "e5" => exp_e5(&ctx),
            "lowband" => exp_lowband(&ctx),
            "linear" => exp_linear(&ctx),
            "misses" => exp_misses(&ctx),
            "confirm" => exp_confirm(&ctx),
            "asystole" => exp_asystole(&ctx),
            "flatline" => exp_flatline(&ctx),
            "clip" => exp_clip(&ctx),
            "thresholds" => exp_thresholds(&ctx),
            other => {
                let _ = run_lopo;
                eprintln!("unknown experiment {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
