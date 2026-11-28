// SPDX-License-Identifier: GPL-3.0-or-later
//! Tools for tuning the analyser on the **development** half of the data
//! without fooling ourselves.
//!
//! DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE.
//!
//! * [`collect`] runs a fresh analyser (as at the start of every analysis)
//!   over each scored 12 s segment, carrying on for up to 3 s more (the
//!   stand-clear countdown) where the same rhythm continues, and keeps every
//!   window's measurements. Measurements do not depend on the thresholds,
//!   so this is done once.
//! * [`replay`] applies a candidate rule set (thresholds plus confirmation
//!   rule) to the stored measurements exactly as the analyser would, and
//!   says what the device would have done.
//! * [`lopo`] chooses among candidates by **leave-one-patient-out**
//!   cross-validation: for each patient in turn, the candidate is chosen on
//!   all the other patients, and only that choice is scored on the patient
//!   left out. The pooled left-out results are an honest estimate of how
//!   the *choosing procedure* does on a new patient; the resubstitution
//!   result (choosing and scoring on everyone) is optimistic.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use aed_rhythm::{classify_with, Analyzer, Features, Reason, Settings, WindowVerdict};
use aed_types::{RhythmAnalyzer, RhythmDecision};

use crate::eval::{prepare, FS};
use crate::truth::{
    amplitude_uv, beat_rate_bpm, classify_truth, clean_runs, Db, Label, TruthClass,
};

/// Analysis limit, seconds (decision D-27).
pub const LIMIT_S: u8 = 12;
/// Stand-clear countdown after "shock advised", seconds.
pub const COUNTDOWN_S: u8 = 3;
/// Time of the first window's end from the start of an analysis, seconds
/// (1 s settling + 4 s window).
pub const FIRST_WINDOW_S: u8 = 5;

/// One window's stored measurements.
#[derive(Debug, Clone, Copy)]
pub struct Win {
    /// The analyser's measurements.
    pub f: Features,
    /// Share of the window's raw samples that were at the original
    /// recorder's limits (ground truth for clipping).
    pub railed: f32,
    /// What the analyser, with the device's settings, announced for it.
    pub announced: Option<RhythmDecision>,
}

/// One scored segment with its windows.
#[derive(Debug, Clone)]
pub struct Seg {
    /// Patient index (records of the same patient share it).
    pub patient: usize,
    /// Database.
    pub db: Db,
    /// Record.
    pub record: String,
    /// Channel.
    pub channel: usize,
    /// Scored class.
    pub class: TruthClass,
    /// Annotated label.
    pub label: Label,
    /// Start, seconds into the record.
    pub start_s: f64,
    /// Ground-truth amplitude, µV.
    pub amplitude_uv: f64,
    /// Windows ending 5, 6, 7 ... s after the segment start (up to 15 s).
    pub wins: Vec<Win>,
}

/// Patient key: database plus record, with the documented same-patient
/// records merged.
#[must_use]
pub fn patient_key(db: Db, record: &str) -> String {
    for (d, names) in crate::split::SAME_PATIENT {
        if *d == db.name() && names.contains(&record) {
            return format!("{}/{}", db.name(), names.join("+"));
        }
    }
    format!("{}/{record}", db.name())
}

/// Collect every scored 12 s segment of the listed records.
///
/// # Errors
/// A message for the first record that cannot be loaded.
pub fn collect(
    data_dir: &Path,
    records: &[(Db, String)],
    threads: usize,
) -> Result<(Vec<Seg>, Vec<String>), String> {
    let patients: Vec<String> = {
        let mut v: Vec<String> = records.iter().map(|(d, r)| patient_key(*d, r)).collect();
        v.sort();
        v.dedup();
        v
    };
    let next = AtomicUsize::new(0);
    type Done = Vec<(usize, Result<Vec<Seg>, String>)>;
    let out: Mutex<Done> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((db, name)) = records.get(i) else {
                    break;
                };
                let pid = patients
                    .iter()
                    .position(|p| *p == patient_key(*db, name))
                    .unwrap_or(0);
                let r = collect_record(data_dir, *db, name, pid);
                if let Ok(mut v) = out.lock() {
                    v.push((i, r));
                }
            });
        }
    });
    let mut v = out.into_inner().unwrap_or_default();
    v.sort_by_key(|(i, _)| *i);
    let mut segs = Vec::new();
    for (_, r) in v {
        segs.extend(r?);
    }
    Ok((segs, patients))
}

fn collect_record(data_dir: &Path, db: Db, name: &str, patient: usize) -> Result<Vec<Seg>, String> {
    let p = prepare(db, &data_dir.join(db.name()), name)
        .map_err(|e| format!("{}/{name}: {e}", db.name()))?;
    let mut segs = Vec::new();
    let len = f64::from(LIMIT_S);
    for (ch, chan) in p.channels.iter().enumerate() {
        for run in clean_runs(&p.timeline, ch) {
            let mut start = run.start;
            while start + len <= run.end {
                let a = ((start * FS as f64).round().max(0.0) as usize).min(chan.samples.len());
                let b12 = (a + LIMIT_S as usize * FS).min(chan.samples.len());
                let Some(s12) = chan.samples.get(a..b12) else {
                    break;
                };
                if s12.len() + 1 < LIMIT_S as usize * FS {
                    break;
                }
                let amplitude = amplitude_uv(s12, FS).unwrap_or(0.0);
                let rate = beat_rate_bpm(&p.timeline.beats, start, start + len);
                if let Some(class) = classify_truth(&run.label, Some(amplitude), rate) {
                    let end_s = (start + f64::from(LIMIT_S + COUNTDOWN_S)).min(run.end);
                    let b = ((end_s * FS as f64).round() as usize).clamp(a, chan.samples.len());
                    let samples = chan.samples.get(a..b).unwrap_or(&[]);
                    let railed = chan.railed.get(a..b).unwrap_or(&[]);
                    segs.push(Seg {
                        patient,
                        db,
                        record: name.to_string(),
                        channel: ch,
                        class,
                        label: run.label.clone(),
                        start_s: start,
                        amplitude_uv: amplitude,
                        wins: windows_of(samples, railed),
                    });
                }
                start += len;
            }
        }
    }
    Ok(segs)
}

/// Every window a fresh analyser produces on `samples`, with the railed share.
#[must_use]
pub fn windows_of(samples: &[i16], railed: &[bool]) -> Vec<Win> {
    let mut a = Analyzer::new();
    let mut out = Vec::new();
    let mut seen = 0;
    for (i, s) in samples.iter().enumerate() {
        let _ = a.push(*s);
        if let Some(r) = a.last_report() {
            if r.window_number != seen {
                seen = r.window_number;
                let lo = (i + 1).saturating_sub(4 * FS);
                let w = railed.get(lo..=i).unwrap_or(&[]);
                let n = w.iter().filter(|x| **x).count();
                out.push(Win {
                    f: r.features,
                    railed: n as f32 / w.len().max(1) as f32,
                    announced: r.announced,
                });
            }
        }
    }
    out
}

/// The analyser's thresholds as they were before any tuning on real data
/// (commit 9cf11b6), with the two checks added since switched off.
pub const UNTUNED: Settings = Settings {
    max_saturated_fraction: 0.01,
    max_clipped_fraction: 1.0,
    max_plausible_pp_uv: 8_000.0,
    asystole_max_uv: 100.0,
    asystole_max_rms_uv: 0.0,
    asystole_min_leakage: 1.0,
    max_noise_ratio: 0.15,
    max_low_band_fraction: 0.5,
    fine_vf_max_pp_uv: 200.0,
    tcsc_vf_min: 0.45,
    count_vf_min: 0.18,
    leakage_vf_max: 0.625,
    dominant_vf_min_hz: 2.4,
    dominant_vf_max_hz: 9.0,
    concentration_vf_min: 0.4,
    confirm_windows: 3,
    confirm_span: 3,
};

/// The confirmation and announcement rule being evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Confirm {
    /// `Shockable` when the current window is shockable and at least `k` of
    /// the last `n` windows were.
    pub k: u8,
    /// See `k`.
    pub n: u8,
    /// Before a shock has been confirmed, `NonShockable` is announced only
    /// after this many non-shockable windows in a row (1 = at once, as
    /// before); until then the analyser says "still analysing".
    pub nonshock_m: u8,
    /// After a shock has been confirmed, a single window on which the two
    /// families disagree, straight after a `Shockable` announcement, is
    /// "still analysing" (no announcement) instead of `Indeterminate`,
    /// provided at least k−1 of the last n windows were shockable. A second
    /// such window in a row is announced `Indeterminate` (which cancels).
    pub hold_on_disagreement: bool,
}

impl Confirm {
    /// The untuned rule: 3 in a row, every other verdict announced at once.
    pub const UNTUNED: Confirm = Confirm {
        k: 3,
        n: 3,
        nonshock_m: 1,
        hold_on_disagreement: false,
    };

    /// The rule a [`Settings`] describes (as the analyser applies it).
    #[must_use]
    pub const fn of(s: &Settings) -> Confirm {
        Confirm {
            k: s.confirm_windows,
            n: s.confirm_span,
            nonshock_m: 1,
            hold_on_disagreement: false,
        }
    }
}

/// Check that replaying the device's settings gives, window by window,
/// exactly what the real analyser announced. Returns the number of windows
/// compared and the number that differ.
#[must_use]
pub fn check_replay(segs: &[Seg]) -> (usize, usize) {
    let s = Settings::DEFAULT;
    let c = Confirm::of(&s);
    let (mut n, mut bad) = (0, 0);
    for seg in segs {
        let mut st = ConfirmState::default();
        for w in &seg.wins {
            n += 1;
            if st.step(&classify_with(&w.f, &s), &c) != w.announced {
                bad += 1;
            }
        }
    }
    (n, bad)
}

/// Replay state of [`Confirm`].
#[derive(Debug, Clone, Copy, Default)]
struct ConfirmState {
    history: u8,
    nonshock_run: u8,
    confirmed: bool,
    last_was_shockable: bool,
}

impl ConfirmState {
    fn step(&mut self, v: &WindowVerdict, c: &Confirm) -> Option<RhythmDecision> {
        let shockable = v.decision == RhythmDecision::Shockable;
        self.history = (self.history << 1) | u8::from(shockable);
        let mask = if c.n >= 8 {
            u8::MAX
        } else {
            (1_u8 << c.n.max(1)) - 1
        };
        let recent = (self.history & mask).count_ones() as u8;
        self.nonshock_run = if v.decision == RhythmDecision::NonShockable {
            self.nonshock_run.saturating_add(1)
        } else {
            0
        };
        let out = match v.decision {
            RhythmDecision::Shockable if recent >= c.k.max(1) => {
                self.confirmed = true;
                Some(RhythmDecision::Shockable)
            }
            RhythmDecision::Shockable => None,
            RhythmDecision::NonShockable => {
                if self.confirmed || self.nonshock_run >= c.nonshock_m.max(1) {
                    Some(RhythmDecision::NonShockable)
                } else {
                    None
                }
            }
            RhythmDecision::Indeterminate => {
                if self.last_was_shockable
                    && c.hold_on_disagreement
                    && v.reason == Reason::MethodsDisagree
                    && recent + 1 >= c.k
                {
                    None
                } else {
                    Some(RhythmDecision::Indeterminate)
                }
            }
        };
        self.last_was_shockable = out == Some(RhythmDecision::Shockable);
        out
    }
}

/// What the device would have done on one segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Played {
    /// First final answer and when (seconds from the start of the
    /// analysis), if one came within the 12 s limit.
    pub decision: Option<(RhythmDecision, u8)>,
    /// For a shock: `Some(true)` if a non-shockable answer came during the
    /// 3 s countdown (the rescue sequence then abandons the shock),
    /// `Some(false)` if not, `None` if the recording does not continue long
    /// enough in the same rhythm to tell.
    pub cancelled: Option<bool>,
}

/// Replay one segment.
pub fn replay(seg: &Seg, verdict: &dyn Fn(&Features) -> WindowVerdict, c: &Confirm) -> Played {
    let mut st = ConfirmState::default();
    for (k, w) in seg.wins.iter().enumerate() {
        let t = FIRST_WINDOW_S + k as u8;
        if t > LIMIT_S {
            break;
        }
        match st.step(&verdict(&w.f), c) {
            Some(RhythmDecision::Shockable) => {
                let mut cancelled = None;
                let rest = seg.wins.get(k + 1..).unwrap_or(&[]);
                if rest.len() >= COUNTDOWN_S as usize {
                    cancelled = Some(false);
                    for w2 in rest.iter().take(COUNTDOWN_S as usize) {
                        if matches!(st.step(&verdict(&w2.f), c), Some(d) if d != RhythmDecision::Shockable)
                        {
                            cancelled = Some(true);
                            break;
                        }
                    }
                } else {
                    for w2 in rest {
                        if matches!(st.step(&verdict(&w2.f), c), Some(d) if d != RhythmDecision::Shockable)
                        {
                            cancelled = Some(true);
                            break;
                        }
                    }
                }
                return Played {
                    decision: Some((RhythmDecision::Shockable, t)),
                    cancelled,
                };
            }
            Some(RhythmDecision::NonShockable) => {
                return Played {
                    decision: Some((RhythmDecision::NonShockable, t)),
                    cancelled: None,
                };
            }
            _ => {}
        }
    }
    Played {
        decision: None,
        cancelled: None,
    }
}

/// Counts for one class (and usually one patient).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    /// Segments.
    pub n: u32,
    /// Shock by analysis length 7, 8, 10, 12 s.
    pub shock: [u32; 4],
    /// No shock by 7, 8, 10, 12 s.
    pub noshock: [u32; 4],
    /// Shocks (12 s) cancelled in the countdown.
    pub cancelled: u32,
    /// Shocks (12 s) whose countdown could not be assessed.
    pub cd_unknown: u32,
    /// Decision-time histogram (index = seconds, 0..=12), 12 s analysis.
    pub t_hist: [u32; 13],
}

/// Analysis lengths reported, seconds.
pub const LENGTHS: [u8; 4] = [7, 8, 10, 12];

impl Tally {
    /// Add one played segment.
    pub fn add(&mut self, p: &Played) {
        self.n += 1;
        if let Some((d, t)) = p.decision {
            for (i, l) in LENGTHS.iter().enumerate() {
                if t <= *l {
                    match d {
                        RhythmDecision::Shockable => {
                            if let Some(x) = self.shock.get_mut(i) {
                                *x += 1;
                            }
                        }
                        _ => {
                            if let Some(x) = self.noshock.get_mut(i) {
                                *x += 1;
                            }
                        }
                    }
                }
            }
            if let Some(h) = self.t_hist.get_mut(t as usize) {
                *h += 1;
            }
            if d == RhythmDecision::Shockable {
                match p.cancelled {
                    Some(true) => self.cancelled += 1,
                    None => self.cd_unknown += 1,
                    Some(false) => {}
                }
            }
        }
    }

    /// Merge.
    pub fn merge(&mut self, o: &Tally) {
        self.n += o.n;
        for i in 0..4 {
            if let (Some(a), Some(b)) = (self.shock.get_mut(i), o.shock.get(i)) {
                *a += b;
            }
            if let (Some(a), Some(b)) = (self.noshock.get_mut(i), o.noshock.get(i)) {
                *a += b;
            }
        }
        self.cancelled += o.cancelled;
        self.cd_unknown += o.cd_unknown;
        for (a, b) in self.t_hist.iter_mut().zip(o.t_hist.iter()) {
            *a += b;
        }
    }

    /// Shocks at the 12 s limit.
    #[must_use]
    pub fn shocks12(&self) -> u32 {
        self.shock.get(3).copied().unwrap_or(0)
    }

    /// Inconclusive at 12 s.
    #[must_use]
    pub fn inconclusive12(&self) -> u32 {
        self.n - self.shocks12() - self.noshock.get(3).copied().unwrap_or(0)
    }
}

/// Per patient, per class tallies for one candidate.
pub type Tallies = BTreeMap<(usize, TruthClass), Tally>;

/// Score a candidate on all segments.
pub fn score(segs: &[Seg], verdict: &dyn Fn(&Features) -> WindowVerdict, c: &Confirm) -> Tallies {
    let mut t = Tallies::new();
    for s in segs {
        t.entry((s.patient, s.class))
            .or_default()
            .add(&replay(s, verdict, c));
    }
    t
}

/// Pool tallies of one class over patients, optionally leaving one out.
#[must_use]
pub fn pooled(t: &Tallies, class: TruthClass, leave_out: Option<usize>) -> Tally {
    let mut out = Tally::default();
    for ((p, c), x) in t {
        if *c == class && Some(*p) != leave_out {
            out.merge(x);
        }
    }
    out
}

/// A candidate's training-set constraints and objective.
#[derive(Debug, Clone, Copy)]
pub struct Objective {
    /// Highest allowed false-shock rate on normal sinus rhythm (12 s).
    pub max_fs_nsr: f64,
    /// Highest allowed false-shock rate on other non-shockable rhythms.
    pub max_fs_other: f64,
    /// Extra asystole false shocks allowed beyond the untuned analyser's.
    pub asys_slack: u32,
}

/// The constraints used for every choice in this tuning (see TUNING_LOG.md).
/// Asystole has no rate limit of its own: a candidate may not give more
/// asystole false shocks than the untuned analyser gives on the same
/// training patients (the dev set holds only 4 patients with scored
/// asystole, one of them with a doubtfully labelled stretch).
pub const OBJECTIVE: Objective = Objective {
    max_fs_nsr: 0.002,
    max_fs_other: 0.005,
    asys_slack: 0,
};

fn rate(x: u32, n: u32) -> f64 {
    if n == 0 {
        0.0
    } else {
        f64::from(x) / f64::from(n)
    }
}

/// (feasible, objective value, false shocks) of a candidate's tallies, leaving one patient out.
#[must_use]
pub fn evaluate(
    t: &Tallies,
    reference: &Tallies,
    leave_out: Option<usize>,
    o: &Objective,
) -> (bool, f64, u32) {
    let vf = pooled(t, TruthClass::CoarseVf, leave_out);
    let nsr = pooled(t, TruthClass::Nsr, leave_out);
    let oth = pooled(t, TruthClass::OtherNonShockable, leave_out);
    let asys = pooled(t, TruthClass::Asystole, leave_out);
    let feasible = rate(nsr.shocks12(), nsr.n) <= o.max_fs_nsr
        && rate(oth.shocks12(), oth.n) <= o.max_fs_other
        && asys.shocks12()
            <= pooled(reference, TruthClass::Asystole, leave_out)
                .shocks12()
                .saturating_add(o.asys_slack);
    // Sensitivity at 12 s, with delivered shocks (not cancelled in the
    // countdown) and the 8 s figure as small tie-breakers.
    let delivered = vf.shocks12().saturating_sub(vf.cancelled);
    let obj = rate(vf.shocks12(), vf.n)
        + 0.1 * rate(delivered, vf.n)
        + 0.1 * rate(vf.shock.get(1).copied().unwrap_or(0), vf.n);
    let fs = nsr.shocks12() + oth.shocks12() + asys.shocks12();
    (feasible, obj, fs)
}

/// Result of leave-one-patient-out selection.
#[derive(Debug, Clone)]
pub struct Lopo {
    /// Pooled left-out tallies, per class.
    pub cv: BTreeMap<TruthClass, Tally>,
    /// How often each candidate was chosen.
    pub chosen: BTreeMap<usize, usize>,
    /// The candidate chosen on all patients (the one to freeze).
    pub full_choice: Option<usize>,
    /// For each patient, the candidate chosen without it.
    pub per_patient: Vec<Option<usize>>,
}

/// Choose the best feasible candidate on the given patients. Ties (within
/// 1e-9) go to the candidate with fewer false shocks, then to the earlier
/// one: candidates must be listed from strictest to most lenient.
#[must_use]
pub fn choose(
    cands: &[Tallies],
    reference: &Tallies,
    leave_out: Option<usize>,
    o: &Objective,
) -> Option<usize> {
    let mut best: Option<(usize, f64, u32)> = None;
    for (i, t) in cands.iter().enumerate() {
        let (ok, obj, fs) = evaluate(t, reference, leave_out, o);
        if !ok {
            continue;
        }
        let better = match best {
            None => true,
            Some((_, bo, bfs)) => obj > bo + 1e-9 || ((obj - bo).abs() <= 1e-9 && fs < bfs),
        };
        if better {
            best = Some((i, obj, fs));
        }
    }
    best.map(|b| b.0)
}

/// Leave-one-patient-out selection over candidates.
#[must_use]
pub fn lopo(cands: &[Tallies], reference: &Tallies, n_patients: usize, o: &Objective) -> Lopo {
    let mut cv: BTreeMap<TruthClass, Tally> = BTreeMap::new();
    let mut chosen = BTreeMap::new();
    let mut per_patient = Vec::new();
    for p in 0..n_patients {
        let pick = choose(cands, reference, Some(p), o);
        per_patient.push(pick);
        let Some(i) = pick else { continue };
        *chosen.entry(i).or_insert(0) += 1;
        if let Some(t) = cands.get(i) {
            for ((pp, c), x) in t {
                if *pp == p {
                    cv.entry(*c).or_default().merge(x);
                }
            }
        }
    }
    Lopo {
        cv,
        chosen,
        full_choice: choose(cands, reference, None, o),
        per_patient,
    }
}

/// Default verdict function for given settings.
pub fn verdict_with(s: Settings) -> impl Fn(&Features) -> WindowVerdict {
    move |f| classify_with(f, &s)
}

/// Logistic regression by Newton's method (iteratively reweighted least
/// squares) with a small ridge penalty. `x` rows must already include a
/// constant 1 if an intercept is wanted. `w` are per-row weights.
/// Returns the coefficients, or `None` if a step could not be solved.
#[must_use]
pub fn logistic(x: &[Vec<f64>], y: &[bool], w: &[f64], ridge: f64) -> Option<Vec<f64>> {
    let d = x.first()?.len();
    let mut beta = vec![0.0; d];
    for _ in 0..50 {
        let mut g = vec![0.0; d];
        let mut h = vec![vec![0.0; d]; d];
        for ((xi, yi), wi) in x.iter().zip(y).zip(w) {
            let z: f64 = xi.iter().zip(&beta).map(|(a, b)| a * b).sum();
            let p = 1.0 / (1.0 + (-z).exp());
            let r = f64::from(u8::from(*yi)) - p;
            let s = (p * (1.0 - p)).max(1e-9);
            for j in 0..d {
                let xj = *xi.get(j)?;
                *g.get_mut(j)? += wi * r * xj;
                for k in 0..d {
                    *h.get_mut(j)?.get_mut(k)? += wi * s * xj * xi.get(k)?;
                }
            }
        }
        for j in 0..d {
            *g.get_mut(j)? -= ridge * beta.get(j)?;
            *h.get_mut(j)?.get_mut(j)? += ridge;
        }
        let step = solve(h, g)?;
        let mut biggest = 0.0_f64;
        for (b, s) in beta.iter_mut().zip(&step) {
            *b += s;
            biggest = biggest.max(s.abs());
        }
        if biggest < 1e-8 {
            break;
        }
    }
    Some(beta)
}

/// Solve `a x = b` by Gaussian elimination with partial pivoting.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let piv = (col..n).max_by(|i, j| {
            let x = a.get(*i).and_then(|r| r.get(col)).map_or(0.0, |v| v.abs());
            let y = a.get(*j).and_then(|r| r.get(col)).map_or(0.0, |v| v.abs());
            x.total_cmp(&y)
        })?;
        a.swap(col, piv);
        b.swap(col, piv);
        let p = *a.get(col)?.get(col)?;
        if p.abs() < 1e-12 {
            return None;
        }
        for r in col + 1..n {
            let f = a.get(r)?.get(col)? / p;
            for c in col..n {
                let v = *a.get(col)?.get(c)?;
                *a.get_mut(r)?.get_mut(c)? -= f * v;
            }
            let bv = *b.get(col)?;
            *b.get_mut(r)? -= f * bv;
        }
    }
    let mut x = vec![0.0; n];
    for r in (0..n).rev() {
        let mut s = *b.get(r)?;
        for c in r + 1..n {
            s -= a.get(r)?.get(c)? * x.get(c)?;
        }
        *x.get_mut(r)? = s / a.get(r)?.get(r)?;
    }
    Some(x)
}
