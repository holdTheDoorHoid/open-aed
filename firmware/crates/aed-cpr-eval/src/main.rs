// SPDX-License-Identifier: GPL-3.0-or-later
//! Command line for the compression-sensing evaluation.
//!
//! ```text
//! aed-cpr-eval describe                 what the data holds (development half)
//! aed-cpr-eval dev [key=value ...]      score the development half (settings may be changed)
//! aed-cpr-eval heldout                  score the held-out half with the default settings
//! ```
//!
//! Options: `--data DIR` (default: see `data::default_root`), `--marks DIR`
//! (write the reference and the detector's answers per case, for plotting).
// A development-computer tool: an out-of-range index is a bug in the tool.
#![allow(clippy::indexing_slicing)]

use std::path::PathBuf;
use std::process::ExitCode;

use aed_cpr::Settings;
use aed_cpr_eval::data::{default_root, load, Case, Sample, RATE_HZ};
use aed_cpr_eval::reference::{build, RefSettings};
use aed_cpr_eval::score::{run_detector, score, CaseScore};
use aed_cpr_eval::split::Half;
use aed_cpr_eval::summary::render;
use aed_types::ChestState;

fn set(settings: &mut Settings, kv: &str) -> Result<(), String> {
    let (k, v) = kv.split_once('=').ok_or(format!("expected key=value, got {kv}"))?;
    let u = || v.parse::<u32>().map_err(|e| e.to_string());
    let f = || v.parse::<f32>().map_err(|e| e.to_string());
    match k {
        "min_compression_mohm" => settings.min_compression_mohm = u()?,
        "highpass_hz" => settings.highpass_hz = f()?,
        "highpass_fourth_order" => settings.highpass_fourth_order = v == "true",
        "lowpass_hz" => settings.lowpass_hz = f()?,
        "max_rise_ms" => settings.max_rise_ms = u()?,
        "max_steep_ms" => settings.max_steep_ms = u()?,
        "min_rise_ms" => settings.min_rise_ms = u()?,
        "quiet_min_ms" => settings.quiet_min_ms = u()?,
        "settle_ms" => settings.settle_ms = u()?,
        "stuck_ms" => settings.stuck_ms = u()?,
        "noise_max_mohm" => settings.noise_max_mohm = u()?,
        "max_interval_ms" => settings.max_interval_ms = u()?,
        _ => return Err(format!("unknown setting {k}")),
    }
    Ok(())
}

fn write_marks(dir: &PathBuf, case: &Case, states: &[ChestState], reference: &aed_cpr_eval::reference::Reference) {
    let mut text = String::new();
    for r in &reference.runs {
        text.push_str(&format!("{:.3} {:.3} C\n", r.start as f64 / RATE_HZ, r.end as f64 / RATE_HZ));
    }
    let mut i = 0;
    while i < states.len() {
        let st = states[i];
        let mut j = i;
        while j < states.len() && states[j] == st {
            j += 1;
        }
        let kind = match st {
            ChestState::Compressing => Some('U'),
            ChestState::Unknown => Some('K'),
            ChestState::HandsOff => None,
        };
        if let Some(k) = kind {
            text.push_str(&format!("{:.3} {:.3} {k}\n", i as f64 / RATE_HZ, j as f64 / RATE_HZ));
        }
        i = j;
    }
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(dir.join(format!("{}.marks", case.name)), text);
}

fn describe(cases: &[Case]) {
    println!("{:<18} {:>8} {:>8} {:>8} {:>8} {:>9} {:>8}", "case", "minutes", "clipped", "missing", "zeros", "analyses", "breaths");
    let mut extremes: Vec<f64> = Vec::new();
    for c in cases {
        let clipped = c.samples.iter().filter(|s| matches!(s, Sample::Clipped)).count();
        let missing = c.samples.iter().filter(|s| matches!(s, Sample::Missing)).count();
        let zeros = c.samples.iter().filter(|s| matches!(s, Sample::Ohms(z) if *z == 0.0)).count();
        for s in &c.samples {
            if let Sample::Ohms(z) = s {
                if z.abs() > 20.0 {
                    extremes.push(*z);
                }
            }
        }
        let n = c.samples.len() as f64;
        println!(
            "{:<18} {:>8.1} {:>7.2}% {:>7.2}% {:>7.1}% {:>9} {:>8}",
            c.name,
            c.seconds() / 60.0,
            100.0 * clipped as f64 / n,
            100.0 * missing as f64 / n,
            100.0 * zeros as f64 / n,
            c.analyses_s.len(),
            c.breaths_s.len()
        );
    }
    let total_h: f64 = cases.iter().map(Case::seconds).sum::<f64>() / 3600.0;
    println!("\n{} cases, {total_h:.1} hours", cases.len());
    extremes.sort_by(f64::total_cmp);
    let hi = extremes.iter().rev().take(3).collect::<Vec<_>>();
    let lo = extremes.iter().take(3).collect::<Vec<_>>();
    println!("values beyond ±20 ohm (below the clip level): lowest {lo:?}, highest {hi:?}");
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        eprintln!("usage: aed-cpr-eval describe|dev|heldout [--data DIR] [--marks DIR] [key=value ...]");
        return ExitCode::FAILURE;
    };
    let mut root = default_root();
    let mut marks: Option<PathBuf> = None;
    let mut settings = Settings::default();
    let mut rest: Vec<String> = args.collect();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--data" if i + 1 < rest.len() => {
                root = PathBuf::from(rest.remove(i + 1));
                rest.remove(i);
            }
            "--marks" if i + 1 < rest.len() => {
                marks = Some(PathBuf::from(rest.remove(i + 1)));
                rest.remove(i);
            }
            _ => i += 1,
        }
    }
    let half = match cmd.as_str() {
        "describe" | "dev" => Half::Development,
        "heldout" => {
            if !rest.is_empty() {
                eprintln!("the held-out half is scored with the frozen default settings only");
                return ExitCode::FAILURE;
            }
            Half::HeldOut
        }
        _ => {
            eprintln!("unknown command {cmd}");
            return ExitCode::FAILURE;
        }
    };
    for kv in &rest {
        if let Err(e) = set(&mut settings, kv) {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    }
    if let Err(e) = settings.validate() {
        eprintln!("settings refused: {e:?}");
        return ExitCode::FAILURE;
    }
    let names = match half.cases() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let loaded = match load(&root, &names) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot read the data at {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    let mut cases = Vec::new();
    for c in loaded {
        match c {
            Ok(c) => cases.push(c),
            Err(e) => eprintln!("skipped: {e}"),
        }
    }
    if cmd == "describe" {
        describe(&cases);
        return ExitCode::SUCCESS;
    }
    let rs = RefSettings::default();
    let scores: Vec<CaseScore> = std::thread::scope(|scope| {
        let handles: Vec<_> = cases
            .iter()
            .map(|c| {
                let marks = marks.clone();
                scope.spawn(move || {
                    let reference = build(c, &rs);
                    let det = run_detector(c, settings)?;
                    if let Some(dir) = &marks {
                        write_marks(dir, c, &det.states, &reference);
                    }
                    Some(score(c, &reference, &det))
                })
            })
            .collect();
        handles.into_iter().filter_map(|h| h.join().ok().flatten()).collect()
    });
    let title = match half {
        Half::Development => format!("development half, settings {settings:?}"),
        Half::HeldOut => "HELD-OUT half, frozen default settings".to_string(),
    };
    print!("{}", render(&title, &scores));
    ExitCode::SUCCESS
}
