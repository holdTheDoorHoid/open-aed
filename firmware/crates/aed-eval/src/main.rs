// SPDX-License-Identifier: GPL-3.0-or-later
//! Command-line front end: evaluate the rhythm analyser on PhysioNet data.
//!
//! ```text
//! cargo run --release -p aed-eval -- --data <dir> [--db cudb|vfdb|mitdb]... [--csv <out dir>] [--out <report.md>] [--threads N]
//! ```
//!
//! Prints the report as Markdown. DEVELOPMENT COMPUTER ONLY; NOT A MEDICAL DEVICE.

use std::path::PathBuf;
use std::process::ExitCode;

use aed_eval::report::{build_report, episodes_csv, segments_csv, Meta};
use aed_eval::run_all;
use aed_eval::truth::Db;

fn usage() {
    eprintln!("Usage: aed-eval --data <dir with cudb/ vfdb/ mitdb/> [--db cudb|vfdb|mitdb]... [--csv <out dir>] [--out <report.md>] [--threads N]");
}

/// Today's date (UTC) as YYYY-MM-DD, from the system clock.
fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = (secs / 86_400) as i64;
    // Civil-from-days (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn git(args: &[&str]) -> Option<String> {
    let dir = env!("CARGO_MANIFEST_DIR");
    let out = std::process::Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn commit_text() -> String {
    let Some(hash) = git(&["rev-parse", "--short", "HEAD"]) else {
        return "unknown (git not available)".into();
    };
    let rhythm = git(&["status", "--porcelain", "--", "../aed-rhythm", "../aed-types"]).unwrap_or_default();
    let all = git(&["status", "--porcelain"]).unwrap_or_default();
    let others = all.lines().count();
    let rhythm_note = if rhythm.is_empty() {
        "no uncommitted changes in `aed-rhythm` or `aed-types`".to_string()
    } else {
        format!("**uncommitted changes in the analyser**: {}", rhythm.lines().collect::<Vec<_>>().join("; "))
    };
    format!("`{hash}` ({rhythm_note}; {others} uncommitted path(s) elsewhere in the repository, including this new evaluation crate)")
}

fn main() -> ExitCode {
    let mut data: Option<PathBuf> = None;
    let mut csv: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut dbs: Vec<Db> = Vec::new();
    let mut threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--data" => data = args.next().map(PathBuf::from),
            "--csv" => csv = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--threads" => match args.next().and_then(|t| t.parse().ok()) {
                Some(t) => threads = t,
                None => {
                    usage();
                    return ExitCode::FAILURE;
                }
            },
            "--db" => match args.next().as_deref().and_then(Db::from_name) {
                Some(d) => dbs.push(d),
                None => {
                    usage();
                    return ExitCode::FAILURE;
                }
            },
            "-h" | "--help" => {
                usage();
                return ExitCode::SUCCESS;
            }
            _ => {
                usage();
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(data) = data else {
        usage();
        return ExitCode::FAILURE;
    };
    if dbs.is_empty() {
        dbs = Db::ALL.to_vec();
    }
    eprintln!("aed-eval: development tool, not a medical device. Evaluating {dbs:?} from {} ...", data.display());
    let (outputs, failed) = run_all(&data, &dbs, threads);
    if outputs.is_empty() {
        eprintln!("No records could be loaded.");
        for (db, r, e) in &failed {
            eprintln!("  {}/{r}: {e}", db.name());
        }
        return ExitCode::FAILURE;
    }
    let meta = Meta {
        date: today(),
        commit: commit_text(),
        data_dir: "data/physionet".into(),
        failed,
    };
    let text = build_report(&outputs, &meta);
    println!("{text}");
    if let Some(path) = out {
        if let Err(e) = std::fs::write(&path, &text) {
            eprintln!("cannot write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    }
    if let Some(dir) = csv {
        let write = || -> std::io::Result<()> {
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("segments.csv"), segments_csv(&outputs))?;
            std::fs::write(dir.join("episodes.csv"), episodes_csv(&outputs))?;
            Ok(())
        };
        if let Err(e) = write() {
            eprintln!("cannot write CSV files to {}: {e}", dir.display());
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
