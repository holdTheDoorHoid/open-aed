// SPDX-License-Identifier: GPL-3.0-or-later
//! Runs the rhythm analyser, untuned, on the Cardially out-of-hospital
//! cardiac-arrest recordings (see `crates/aed-eval/README.md`).
//!
//! ```text
//! cargo run --release -p aed-eval --bin aed-cardially -- [--data <dir>] [--out <report.md>] [--csv <records.csv>] [--threads N] [--survey]
//! ```
//!
//! `--data` defaults to `data/mendeley/cardially` at the repository root.
//! `--survey` prints, for every file, only what the reader recovered (no signal).
//! The CSV holds one row per recording and must not be committed: write it
//! under `data/` or outside the repository.
//!
//! DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE.

use std::path::PathBuf;
use std::process::ExitCode;

use aed_eval::cardially::eval::{build_report, evaluate, records_csv, Meta};
use aed_eval::cardially::{dataset_dir, load, unpaired};
use aed_eval::split::sha256_hex;
use aed_rhythm::Settings;

fn usage() {
    eprintln!("Usage: aed-cardially [--data <dir>] [--out <report.md>] [--csv <records.csv>] [--threads N] [--survey]");
}

fn git(args: &[&str]) -> Option<String> {
    let dir = env!("CARGO_MANIFEST_DIR");
    let out = std::process::Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn commit_text() -> String {
    let Some(hash) = git(&["rev-parse", "--short", "HEAD"]) else {
        return "unknown (git not available)".into();
    };
    let rhythm = git(&["status", "--porcelain", "--", "../aed-rhythm", "../aed-types"]).unwrap_or_default();
    if rhythm.is_empty() {
        format!("`{hash}` (no uncommitted changes in `aed-rhythm` or `aed-types`)")
    } else {
        format!("`{hash}` (**uncommitted changes in the analyser**: {})", rhythm.lines().collect::<Vec<_>>().join("; "))
    }
}

fn today() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let days = (secs / 86_400) as i64;
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

fn main() -> ExitCode {
    let mut data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/mendeley/cardially");
    let mut out: Option<PathBuf> = None;
    let mut csv: Option<PathBuf> = None;
    let mut threads = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let mut survey = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--data" => match args.next() {
                Some(v) => data = PathBuf::from(v),
                None => return bad(),
            },
            "--out" => out = args.next().map(PathBuf::from),
            "--csv" => csv = args.next().map(PathBuf::from),
            "--threads" => match args.next().and_then(|v| v.parse().ok()) {
                Some(n) => threads = n,
                None => return bad(),
            },
            "--survey" => survey = true,
            _ => return bad(),
        }
    }
    if dataset_dir(&data).is_none() {
        eprintln!("No Cardially dataset found under {} (see crates/aed-eval/README.md for how to fetch it).", data.display());
        return ExitCode::FAILURE;
    }
    let items = match load(&data, threads) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    if survey {
        println!("record,read,fs_hz,strips,pages,missing,off_grid,conflicts,clock_breaks,shock1_s,duplicate_of,events");
        for it in &items {
            match &it.pdf {
                Ok(p) => println!(
                    "{},ok,{},{},{},{},{},{},{},{},{},{}",
                    it.key(),
                    p.fs_hz,
                    p.strips,
                    p.pages,
                    p.missing,
                    p.off_grid,
                    p.conflicts,
                    p.clock_breaks,
                    p.shock_delivered_s(1).map_or(String::new(), |t| format!("{t:.2}")),
                    it.duplicate_of.as_deref().unwrap_or(""),
                    p.events.iter().map(|e| format!("{:.2}s {}", e.t_s, e.text)).collect::<Vec<_>>().join("|").replace(',', ";")
                ),
                Err(e) => println!("{},error: {},,,,,,,,,,", it.key(), e.replace(',', ";")),
            }
        }
        return ExitCode::SUCCESS;
    }
    let results: Vec<_> = {
        let next = std::sync::atomic::AtomicUsize::new(0);
        let slots: std::sync::Mutex<Vec<(usize, aed_eval::cardially::eval::RecordResult)>> = std::sync::Mutex::new(Vec::new());
        std::thread::scope(|s| {
            for _ in 0..threads.max(1) {
                s.spawn(|| loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(it) = items.get(i) else { break };
                    let r = evaluate(it, Settings::DEFAULT);
                    if let Ok(mut v) = slots.lock() {
                        v.push((i, r));
                    }
                });
            }
        });
        let mut v = slots.into_inner().unwrap_or_default();
        v.sort_by_key(|(i, _)| *i);
        v.into_iter().map(|(_, r)| r).collect()
    };
    let zip = data.join("cardially-ecg-dataset.zip");
    let meta = Meta {
        date: today(),
        commit: commit_text(),
        data: "data/mendeley/cardially (Mendeley Data wpr5nzyn2z version 1)".into(),
        zip_sha256: std::fs::read(&zip).ok().map(|b| sha256_hex(&b)),
    };
    let report = build_report(&items, &results, &unpaired(&data), &meta);
    print!("{report}");
    if let Some(p) = out {
        if let Err(e) = std::fs::write(&p, &report) {
            eprintln!("cannot write {}: {e}", p.display());
            return ExitCode::FAILURE;
        }
    }
    if let Some(p) = csv {
        if let Err(e) = std::fs::write(&p, records_csv(&results)) {
            eprintln!("cannot write {}: {e}", p.display());
            return ExitCode::FAILURE;
        }
        eprintln!("Per-recording results written to {} (do not commit it).", p.display());
    }
    ExitCode::SUCCESS
}

fn bad() -> ExitCode {
    usage();
    ExitCode::FAILURE
}
