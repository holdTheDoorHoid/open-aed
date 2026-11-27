// SPDX-License-Identifier: GPL-3.0-or-later
//! Evaluation of the Open AED rhythm analyser on recorded heart signals.
//!
//! DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE. See SAFETY.md.
//!
//! This crate reads three public PhysioNet databases (cudb, vfdb, mitdb),
//! converts their signals to what the analyser expects (whole microvolts at
//! 250 samples per second), works out from the expert annotations what the
//! heart was doing at each moment, runs the analyser the way the device
//! would, and reports how often it got the answer right against the American
//! Heart Association's 1997 performance goals.
//!
//! * [`wfdb`]: readers for the PhysioNet file formats (written here; no
//!   outside libraries). Every file is treated as hostile.
//! * [`condition`]: units and sampling-rate conversion.
//! * [`truth`]: annotations to ground truth.
//! * [`eval`]: running the analyser.
//! * [`stats`]: confidence limits.
//! * [`report`]: the Markdown report and CSV files.
//! * [`run_all`]: everything, for a data directory.
//!
//! The data are not part of the repository; see the crate README.

pub mod condition;
pub mod eval;
pub mod report;
pub mod stats;
pub mod truth;
pub mod wfdb;

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use eval::{evaluate_record, prepare, RecordOutput};
use truth::Db;

/// Records that could not be loaded: database, record, error text.
pub type Failures = Vec<(Db, String, String)>;

/// Load and evaluate every record of the given databases under `data_dir`
/// (which holds `cudb/`, `vfdb/`, `mitdb/`), using up to `threads` threads.
/// Databases whose directory or RECORDS file is missing are skipped and
/// listed as failures. Results are in database then RECORDS order.
#[must_use]
pub fn run_all(data_dir: &Path, dbs: &[Db], threads: usize) -> (Vec<RecordOutput>, Failures) {
    let mut jobs: Vec<(Db, String)> = Vec::new();
    let mut failed: Failures = Vec::new();
    for db in dbs {
        match wfdb::read_records_list(&data_dir.join(db.name())) {
            Ok(list) => jobs.extend(list.into_iter().map(|r| (*db, r))),
            Err(e) => failed.push((*db, "RECORDS".into(), e.to_string())),
        }
    }
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, Result<RecordOutput, String>)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((db, name)) = jobs.get(i) else { break };
                let r = prepare(*db, &data_dir.join(db.name()), name)
                    .map(|p| evaluate_record(&p))
                    .map_err(|e| e.to_string());
                if let Ok(mut v) = results.lock() {
                    v.push((i, r));
                }
            });
        }
    });
    let mut v = results.into_inner().unwrap_or_default();
    v.sort_by_key(|(i, _)| *i);
    let mut outputs = Vec::new();
    for (i, r) in v {
        match r {
            Ok(o) => outputs.push(o),
            Err(e) => {
                if let Some((db, name)) = jobs.get(i) {
                    failed.push((*db, name.clone(), e));
                }
            }
        }
    }
    (outputs, failed)
}
