// SPDX-License-Identifier: GPL-3.0-or-later
//! The "Cardially" dataset of out-of-hospital cardiac-arrest ECGs (University
//! of Brescia, Mendeley Data, DOI 10.17632/wpr5nzyn2z.1): reading it and
//! running the rhythm analyser on it, **untuned**.
//!
//! What the download holds (checked 27 November 2026, see the crate README):
//!
//! * `dataset/pdf/<class>/<id>.pdf`: event reports printed by Philips review
//!   software ("ERECG 3.0.0.0") to PDF, 60 s of single-lead ECG around the
//!   first shock, as **vector drawings** with one vertex per sample (200 per
//!   second) and a printed scale ("2 div/mV, 5 div/sec"). Read by
//!   [`report_pdf`] (with [`pdf`] and [`inflate`], written here).
//! * `dataset/txt/<class>/<id>.txt`: the authors' digitised 9 s before the
//!   first shock, irregularly spaced points. Read by [`txt`].
//! * `<class>` is the cardiologists' label of the shock outcome: `ROEA`
//!   (return of organised electrical activity), `noROEA`, `indeterminable`.
//!
//! Nothing here is tuned on these data, and nothing may be: see the README.

pub mod eval;
pub mod inflate;
pub mod pdf;
pub mod report_pdf;
pub mod txt;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use report_pdf::{read_report, PdfEcg};

/// The cardiologists' label of the first shock's outcome (folder name).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShockOutcome {
    /// `ROEA`: return of organised electrical activity (40-150/min within
    /// 1 min of the shock, lasting at least 15 s without CPR, per the article).
    Roea,
    /// `noROEA`: asystole, rate under 40/min, pulseless electrical activity,
    /// or VF/VT after the shock (per the article). Not the same as "VF went on".
    NoRoea,
    /// `indeterminable`: the cardiologists could not decide.
    Indeterminable,
}

impl ShockOutcome {
    /// All three, in report order.
    pub const ALL: [ShockOutcome; 3] = [ShockOutcome::Roea, ShockOutcome::NoRoea, ShockOutcome::Indeterminable];

    /// The folder name.
    #[must_use]
    pub const fn dir(self) -> &'static str {
        match self {
            ShockOutcome::Roea => "ROEA",
            ShockOutcome::NoRoea => "noROEA",
            ShockOutcome::Indeterminable => "indeterminable",
        }
    }
}

/// One recording of the dataset.
#[derive(Debug, Clone)]
pub struct Item {
    /// Outcome label.
    pub outcome: ShockOutcome,
    /// File name of the PDF, without `.pdf`.
    pub id: String,
    /// The recovered report, or why it could not be read.
    pub pdf: Result<PdfEcg, String>,
    /// File name of the matching `.txt` (leading zeros ignored), if any.
    pub txt_name: Option<String>,
    /// The digitised points, or why they could not be read.
    pub txt: Option<Result<Vec<(f64, f64)>, String>>,
    /// If this report's ECG is identical, sample for sample, to an earlier
    /// item's: that item's `class/id`.
    pub duplicate_of: Option<String>,
}

impl Item {
    /// `class/id`.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}/{}", self.outcome.dir(), self.id)
    }
}

/// Largest file read, bytes.
const MAX_FILE: u64 = 16 * 1024 * 1024;

/// File stems in `dir` with extension `ext`: regular files only, names of
/// letters, digits and `_` only, sorted.
fn list(dir: &Path, ext: &str) -> Vec<String> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut v: Vec<String> = rd
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(ext).map(str::to_string))
        .filter(|s| !s.is_empty() && s.len() <= 40 && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'))
        .collect();
    v.sort();
    v
}

fn read_capped(path: &Path) -> Result<Vec<u8>, String> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > MAX_FILE {
        return Err("not a regular file of acceptable size".into());
    }
    std::fs::read(path).map_err(|e| e.to_string())
}

/// The `dataset` folder under `root`: `root/dataset`, `root/extracted/dataset`,
/// or `root` itself if it holds `pdf/`.
#[must_use]
pub fn dataset_dir(root: &Path) -> Option<PathBuf> {
    [root.join("extracted/dataset"), root.join("dataset"), root.to_path_buf()]
        .into_iter()
        .find(|d| d.join("pdf/ROEA").is_dir() && d.join("pdf/noROEA").is_dir())
}

fn stem_key(s: &str) -> &str {
    s.trim_start_matches('0')
}

/// Read every recording under `root` (see [`dataset_dir`]) using up to
/// `threads` threads. Items are in label order, then name order.
///
/// # Errors
/// If no dataset folder is found.
pub fn load(root: &Path, threads: usize) -> Result<Vec<Item>, String> {
    let d = dataset_dir(root).ok_or_else(|| format!("no Cardially dataset under {}", root.display()))?;
    let mut jobs: Vec<(ShockOutcome, String, Option<String>)> = Vec::new();
    for o in ShockOutcome::ALL {
        let txts = list(&d.join("txt").join(o.dir()), ".txt");
        for id in list(&d.join("pdf").join(o.dir()), ".pdf") {
            let t = txts.iter().find(|t| stem_key(t) == stem_key(&id)).cloned();
            jobs.push((o, id, t));
        }
    }
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, Item)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((o, id, t)) = jobs.get(i) else { break };
                let pdf = read_capped(&d.join("pdf").join(o.dir()).join(format!("{id}.pdf")))
                    .and_then(|b| read_report(&b).map_err(|e| e.to_string()));
                let txt = t.as_ref().map(|t| {
                    read_capped(&d.join("txt").join(o.dir()).join(format!("{t}.txt")))
                        .and_then(|b| txt::parse_txt(&b).map_err(|e| e.to_string()))
                });
                let item = Item { outcome: *o, id: id.clone(), pdf, txt_name: t.clone(), txt, duplicate_of: None };
                if let Ok(mut v) = results.lock() {
                    v.push((i, item));
                }
            });
        }
    });
    let mut v = results.into_inner().unwrap_or_default();
    v.sort_by_key(|(i, _)| *i);
    let mut items: Vec<Item> = v.into_iter().map(|(_, it)| it).collect();
    mark_duplicates(&mut items);
    Ok(items)
}

/// Mark reports whose ECG is identical to an earlier one's.
pub fn mark_duplicates(items: &mut [Item]) {
    let mut seen: Vec<(Vec<u64>, String)> = Vec::new();
    for it in items.iter_mut() {
        let Ok(p) = &it.pdf else { continue };
        let bits: Vec<u64> = p.samples_uv.iter().map(|v| v.to_bits()).collect();
        if let Some((_, k)) = seen.iter().find(|(b, _)| *b == bits) {
            it.duplicate_of = Some(k.clone());
        } else {
            seen.push((bits, it.key()));
        }
    }
}

/// Text files that have no PDF partner (by name, leading zeros ignored), and
/// PDFs with no text partner, as `class/name`.
#[must_use]
pub fn unpaired(root: &Path) -> (Vec<String>, Vec<String>) {
    let Some(d) = dataset_dir(root) else { return (Vec::new(), Vec::new()) };
    let (mut t_only, mut p_only) = (Vec::new(), Vec::new());
    for o in ShockOutcome::ALL {
        let txts = list(&d.join("txt").join(o.dir()), ".txt");
        let pdfs = list(&d.join("pdf").join(o.dir()), ".pdf");
        for t in &txts {
            if !pdfs.iter().any(|p| stem_key(p) == stem_key(t)) {
                t_only.push(format!("{}/{t}", o.dir()));
            }
        }
        for p in &pdfs {
            if !txts.iter().any(|t| stem_key(p) == stem_key(t)) {
                p_only.push(format!("{}/{p}", o.dir()));
            }
        }
    }
    (t_only, p_only)
}
