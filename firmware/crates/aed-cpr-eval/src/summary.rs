// SPDX-License-Identifier: GPL-3.0-or-later
//! Totals over many cases, printed as plain text tables.

use std::fmt::Write as _;

use aed_types::ChestState;

use crate::score::{percentile, CaseScore};

/// Totals over a set of cases.
#[derive(Debug, Clone, Default)]
pub struct Totals {
    /// Cases scored.
    pub cases: usize,
    /// Scored seconds.
    pub valid_s: f64,
    /// Reference compression seconds (after the onset allowance).
    pub ref_comp_s: f64,
    /// Of which the detector said hands off.
    pub missed_s: f64,
    /// Of which compressing.
    pub comp_s: f64,
    /// Of which unknown.
    pub comp_unknown_s: f64,
    /// Reference hands-off seconds (after the offset allowance).
    pub ref_off_s: f64,
    /// Of which the detector said compressing.
    pub off_comp_s: f64,
    /// Of which unknown.
    pub off_unknown_s: f64,
    /// Reference compressions.
    pub ref_peaks: usize,
    /// Found.
    pub found: usize,
    /// Detector compressions.
    pub det_events: usize,
    /// Matched.
    pub matched: usize,
    /// All rate errors.
    pub rate_errors: Vec<f64>,
    /// All onset latencies.
    pub onset: Vec<f64>,
    /// All onset-to-compressing latencies.
    pub onset_comp: Vec<f64>,
    /// All offset latencies.
    pub offset: Vec<f64>,
    /// All reference peak sizes.
    pub sizes: Vec<f64>,
    /// Analysis markers by detector answer: hands off, compressing, unknown.
    pub analyses: [usize; 3],
    /// All pauses between reference runs (under 60 s), seconds.
    pub pauses: Vec<f64>,
    /// Longest single miss in any case, seconds, and the case.
    pub longest_miss: (f64, String),
}

/// Add up the case scores.
#[must_use]
pub fn totals(scores: &[CaseScore]) -> Totals {
    let mut t = Totals::default();
    for c in scores {
        t.cases += 1;
        t.valid_s += c.valid_s;
        t.ref_comp_s += c.ref_comp_s;
        t.missed_s += c.missed_s;
        t.comp_s += c.comp_as_comp_s;
        t.comp_unknown_s += c.comp_as_unknown_s;
        t.ref_off_s += c.ref_off_s;
        t.off_comp_s += c.off_as_comp_s;
        t.off_unknown_s += c.off_as_unknown_s;
        t.ref_peaks += c.ref_peaks;
        t.found += c.ref_peaks_found;
        t.det_events += c.det_events;
        t.matched += c.det_events_matched;
        t.rate_errors.extend(&c.rate_errors);
        t.onset.extend(&c.onset_s);
        t.onset_comp.extend(&c.onset_comp_s);
        t.offset.extend(&c.offset_s);
        t.sizes.extend(&c.peak_sizes);
        t.pauses.extend(&c.pauses_s);
        for &(_, st) in &c.analyses {
            let k = match st {
                ChestState::HandsOff => 0,
                ChestState::Compressing => 1,
                ChestState::Unknown => 2,
            };
            t.analyses[k] += 1;
        }
        if c.longest_miss_s > t.longest_miss.0 {
            t.longest_miss = (c.longest_miss_s, c.name.clone());
        }
    }
    t
}

fn pct(a: f64, b: f64) -> f64 {
    if b > 0.0 {
        100.0 * a / b
    } else {
        0.0
    }
}

fn dist(name: &str, v: &[f64], out: &mut String) {
    let p = |q| percentile(v, q).map_or("-".to_string(), |x| format!("{x:.2}"));
    let _ = writeln!(
        out,
        "  {name:<38} n={:<6} median {:>6}  90th {:>6}  95th {:>6}  99th {:>6}  max {:>6}",
        v.len(),
        p(50.0),
        p(90.0),
        p(95.0),
        p(99.0),
        p(100.0)
    );
}

/// The per-case table and the totals, as text.
#[must_use]
pub fn render(title: &str, scores: &[CaseScore]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "== {title} ==\n");
    let _ = writeln!(
        out,
        "{:<18} {:>7} {:>8} {:>7} {:>7} {:>7} {:>8} {:>7} {:>7} {:>6} {:>6} {:>7}",
        "case", "scored", "ref-comp", "MISSED", "unk", "longest", "ref-off", "off:cmp", "off:unk", "sens", "ppv", "rateErr"
    );
    for c in scores {
        let abs: Vec<f64> = c.rate_errors.iter().map(|e| e.abs()).collect();
        let _ = writeln!(
            out,
            "{:<18} {:>6.0}s {:>7.0}s {:>6.2}% {:>6.1}% {:>6.1}s {:>7.0}s {:>6.2}% {:>6.1}% {:>5.1}% {:>5.1}% {:>7}",
            c.name,
            c.valid_s,
            c.ref_comp_s,
            pct(c.missed_s, c.ref_comp_s),
            pct(c.comp_as_unknown_s, c.ref_comp_s),
            c.longest_miss_s,
            c.ref_off_s,
            pct(c.off_as_comp_s, c.ref_off_s),
            pct(c.off_as_unknown_s, c.ref_off_s),
            pct(c.ref_peaks_found as f64, c.ref_peaks as f64),
            pct(c.det_events_matched as f64, c.det_events as f64),
            percentile(&abs, 50.0).map_or("-".to_string(), |x| format!("{x:.1}")),
        );
    }
    let t = totals(scores);
    let _ = writeln!(out, "\nTotals over {} cases ({:.1} h scored):", t.cases, t.valid_s / 3600.0);
    let _ = writeln!(
        out,
        "  Reference compressions: {:.0} s. Detector said HANDS OFF (missed): {:.0} s = {:.3}%; compressing {:.2}%; unknown {:.2}%",
        t.ref_comp_s,
        t.missed_s,
        pct(t.missed_s, t.ref_comp_s),
        pct(t.comp_s, t.ref_comp_s),
        pct(t.comp_unknown_s, t.ref_comp_s)
    );
    let _ = writeln!(out, "  Longest single miss: {:.2} s ({})", t.longest_miss.0, t.longest_miss.1);
    let _ = writeln!(
        out,
        "  Reference hands off: {:.0} s. Detector said compressing {:.2}%, unknown {:.2}%",
        t.ref_off_s,
        pct(t.off_comp_s, t.ref_off_s),
        pct(t.off_unknown_s, t.ref_off_s)
    );
    let _ = writeln!(
        out,
        "  Per compression: {} reference, {} found ({:.2}%); {} detected, {} matched (PPV {:.2}%)",
        t.ref_peaks,
        t.found,
        pct(t.found as f64, t.ref_peaks as f64),
        t.det_events,
        t.matched,
        pct(t.matched as f64, t.det_events as f64)
    );
    let small3 = t.sizes.iter().filter(|&&s| s < 0.3).count();
    let small5 = t.sizes.iter().filter(|&&s| s < 0.5).count();
    let _ = writeln!(
        out,
        "  Reference compression sizes: {:.2}% below 0.3 ohm, {:.2}% below 0.5 ohm",
        pct(small3 as f64, t.sizes.len() as f64),
        pct(small5 as f64, t.sizes.len() as f64)
    );
    dist("reference compression size (ohm)", &t.sizes, &mut out);
    let abs: Vec<f64> = t.rate_errors.iter().map(|e| e.abs()).collect();
    dist("rate error |detector - reference| /min", &abs, &mut out);
    dist("onset: run start -> not hands off (s)", &t.onset, &mut out);
    dist("onset: run start -> compressing (s)", &t.onset_comp, &mut out);
    dist("offset: run end -> hands off (s)", &t.offset, &mut out);
    dist("reference pauses between runs (s)", &t.pauses, &mut out);
    let over15 = t.pauses.iter().filter(|&&p| p >= 15.0).count();
    let _ = writeln!(
        out,
        "  Reference pauses of 15 s or more: {} of {} ({:.1}%)",
        over15,
        t.pauses.len(),
        pct(over15 as f64, t.pauses.len() as f64)
    );
    let _ = writeln!(
        out,
        "  At the defibrillator's own analysis markers: hands off {}, compressing {}, unknown {}",
        t.analyses[0], t.analyses[1], t.analyses[2]
    );
    out
}
