// SPDX-License-Identifier: GPL-3.0-or-later
//! Command-line front end: evaluate the rhythm analyser on PhysioNet data.
//!
//! ```text
//! cargo run --release -p aed-eval -- --data <dir> [--db cudb|vfdb|mitdb]... [--records <list>] [--csv <out dir>] [--out <report.md>] [--threads N]
//! cargo run --release -p aed-eval -- --data <dir> --make-split <out dir>
//! ```
//!
//! Prints the report as Markdown. DEVELOPMENT COMPUTER ONLY; NOT A MEDICAL DEVICE.

use std::path::PathBuf;
use std::process::ExitCode;

use aed_eval::report::{build_report, episodes_csv, segments_csv, Meta};
use aed_eval::run_all_with;
use aed_eval::split::{list_text, make_split, parse_list, sha256_hex, survey};
use aed_eval::truth::Db;

fn usage() {
    eprintln!("Usage: aed-eval --data <dir with cudb/ vfdb/ mitdb/> [--db cudb|vfdb|mitdb]... [--records <list file>] [--untuned] [--warm-start] [--warm-guard-ms N] [--kind <text>] [--csv <out dir>] [--out <report.md>] [--threads N]");
    eprintln!("       aed-eval --data <dir> --make-split <out dir>   (writes development.txt and held_out.txt)");
}

/// Write the patient-wise split files (see `aed_eval::split`).
fn write_split(data: &std::path::Path, dir: &std::path::Path) -> ExitCode {
    let records = match survey(data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cannot read the annotations: {e}");
            return ExitCode::FAILURE;
        }
    };
    let split = make_split(&records);
    let files = [
        (
            "development.txt",
            "Development half: all tuning uses only these records.",
            &split.development,
        ),
        (
            "held_out.txt",
            "Held-out half: run once, with frozen settings, at the end.",
            &split.held_out,
        ),
    ];
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        return ExitCode::FAILURE;
    }
    for (file, title, names) in files {
        let text = list_text(title, names);
        if let Err(e) = std::fs::write(dir.join(file), &text) {
            eprintln!("cannot write {file}: {e}");
            return ExitCode::FAILURE;
        }
        println!(
            "{file}: {} records, sha256 {}",
            names.len(),
            sha256_hex(text.as_bytes())
        );
    }
    for r in &records {
        let side = if split
            .development
            .contains(&format!("{}/{}", r.db.name(), r.name))
        {
            "development"
        } else {
            "held-out"
        };
        println!(
            "  {}/{}  {:<18} {side}",
            r.db.name(),
            r.name,
            r.content.text()
        );
    }
    ExitCode::SUCCESS
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
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn commit_text() -> String {
    let Some(hash) = git(&["rev-parse", "--short", "HEAD"]) else {
        return "unknown (git not available)".into();
    };
    let rhythm = git(&[
        "status",
        "--porcelain",
        "--",
        "../aed-rhythm",
        "../aed-types",
    ])
    .unwrap_or_default();
    let all = git(&["status", "--porcelain"]).unwrap_or_default();
    let others = all.lines().count();
    let rhythm_note = if rhythm.is_empty() {
        "no uncommitted changes in `aed-rhythm` or `aed-types`".to_string()
    } else {
        format!(
            "**uncommitted changes in the analyser**: {}",
            rhythm.lines().collect::<Vec<_>>().join("; ")
        )
    };
    format!("`{hash}` ({rhythm_note}; {others} uncommitted path(s) elsewhere in the repository)")
}

/// Printed under the title of a `--warm-start` report.
fn warm_note(guard_ms: Option<usize>) -> String {
    let guard = guard_ms.map_or_else(
        || "the device's 0.25 s guard".to_string(),
        |ms| format!("a {ms} ms guard (MEASUREMENT ONLY: not the device's guard)"),
    );
    format!("**Warm start (decision D-47):** before each analysis the 2 s of recording just before it were passed through the analyser's filters, and the analysis window started after {guard} instead of the 1 s settling of a cold start.")
}

fn main() -> ExitCode {
    let mut data: Option<PathBuf> = None;
    let mut csv: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut records: Option<PathBuf> = None;
    let mut make_split_dir: Option<PathBuf> = None;
    let mut untuned = false;
    let mut warm = false;
    let mut warm_guard_ms: Option<usize> = None;
    let mut kind: Option<String> = None;
    let mut dbs: Vec<Db> = Vec::new();
    let mut threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--data" => data = args.next().map(PathBuf::from),
            "--csv" => csv = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--records" => records = args.next().map(PathBuf::from),
            "--make-split" => make_split_dir = args.next().map(PathBuf::from),
            "--untuned" => untuned = true,
            "--warm-start" => warm = true,
            // Measurement only: a warm start with another guard than the device's.
            "--warm-guard-ms" => match args.next().and_then(|t| t.parse().ok()) {
                Some(ms) => {
                    warm = true;
                    warm_guard_ms = Some(ms);
                }
                None => {
                    usage();
                    return ExitCode::FAILURE;
                }
            },
            "--kind" => kind = args.next(),
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
    if let Some(dir) = make_split_dir {
        return write_split(&data, &dir);
    }
    let only = match &records {
        None => None,
        Some(path) => match std::fs::read(path) {
            Ok(bytes) => match std::str::from_utf8(&bytes)
                .map_err(|e| e.to_string())
                .and_then(parse_list)
            {
                Ok(list) => {
                    eprintln!(
                        "restricted to {} records from {} (sha256 {})",
                        list.len(),
                        path.display(),
                        sha256_hex(&bytes)
                    );
                    Some((list, path.display().to_string(), sha256_hex(&bytes)))
                }
                Err(e) => {
                    eprintln!("bad record list {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            },
            Err(e) => {
                eprintln!("cannot read {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        },
    };
    if dbs.is_empty() {
        dbs = Db::ALL.to_vec();
    }
    eprintln!(
        "aed-eval: development tool, not a medical device. Evaluating {dbs:?} from {} ...",
        data.display()
    );
    let settings = if untuned {
        aed_eval::tune::UNTUNED
    } else {
        aed_rhythm::Settings::DEFAULT
    };
    let start = match (warm, warm_guard_ms) {
        (false, _) => aed_eval::eval::Start::Cold,
        (true, None) => aed_eval::eval::Start::DEVICE_WARM,
        (true, Some(ms)) => aed_eval::eval::Start::Warm {
            guard_samples: ms * aed_eval::eval::FS / 1000,
        },
    };
    let (outputs, failed) = run_all_with(
        &data,
        &dbs,
        threads,
        only.as_ref().map(|(l, _, _)| l.as_slice()),
        settings,
        start,
    );
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
        records: only.as_ref().map(|(l, p, h)| format!("{} records listed in `{p}` (SHA-256 `{h}`)", l.len())),
        kind: kind.map(|k| if warm { format!("{k} {}", warm_note(warm_guard_ms)) } else { k }).unwrap_or_else(|| {
            if warm {
                format!("**Analyser with its current settings** (`Settings::DEFAULT`). {}", warm_note(warm_guard_ms))
            } else if untuned {
                "**Untuned analyser** (thresholds as at commit 9cf11b6, set on made-up signals only; the clipping and flatline checks added later are switched off).".into()
            } else {
                "**Analyser with its current settings** (`Settings::DEFAULT`).".into()
            }
        }),
        settings,
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
