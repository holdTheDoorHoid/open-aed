// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests of the reader and the scoring. The reader tests use small made-up
//! files; the real-data test skips cleanly when the recordings are absent
//! (they are never in the repository).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, missing_docs)]

use std::path::PathBuf;

use aed_cpr::Settings;
use aed_cpr_eval::data::{case_id, read_analyses, read_breaths, read_impedance, Case, Sample};
use aed_cpr_eval::reference::{build, RefSettings};
use aed_cpr_eval::score::{run_detector, score};
use aed_cpr_eval::split::Half;
use aed_types::ChestState;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aed-cpr-eval-test-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A made-up recording: 4 s quiet, 20 s of compressions at 110/min of
/// 1.5 ohm, 6 s quiet; change-only impedance like the real files.
fn synthetic_case() -> Case {
    let mut samples = Vec::new();
    for i in 0..(30 * 125) {
        let t = f64::from(i) / 125.0;
        let mut z = 0.02 * ((t * 37.0).sin());
        if (4.0..24.0).contains(&t) {
            let phase = ((t - 4.0) * 110.0 / 60.0).fract();
            z += 1.5 * 0.5 * (1.0 - (std::f64::consts::TAU * phase).cos());
        }
        samples.push(Sample::Ohms((z / 0.0122).round() * 0.0122));
    }
    Case {
        name: "synthetic".into(),
        id: 0,
        samples,
        analyses_s: vec![27.0],
        breaths_s: Vec::new(),
        irregular_rows: 0,
    }
}

#[test]
fn reader_parses_the_three_file_kinds_and_rejects_junk() {
    let dir = temp_dir("reader");
    let imp = dir.join("imp.csv");
    std::fs::write(&imp, "Time (s);TTI (Ohms)\n0.0;-0.0\n0.008;1.5\n0.016;25.0\n0.04;-24.0\n").unwrap();
    let (s, irregular) = read_impedance(&imp).unwrap();
    assert_eq!(irregular, 0);
    assert_eq!(s.len(), 6);
    assert_eq!(s[1], Sample::Ohms(1.5));
    assert_eq!(s[2], Sample::Clipped);
    assert_eq!(s[3], Sample::Missing, "a gap in the time column is missing data");
    assert_eq!(s[5], Sample::Clipped);

    let bad = dir.join("bad.csv");
    std::fs::write(&bad, "Time (s);TTI (Ohms)\n0.0;NaN\n").unwrap();
    assert!(read_impedance(&bad).is_err(), "not-a-number must be refused");
    std::fs::write(&bad, "Time (s);TTI (Ohms)\n0.0;1;2\n").unwrap();
    assert!(read_impedance(&bad).is_err(), "extra columns must be refused");
    std::fs::write(&bad, format!("h\n0.0;{}\n", "9".repeat(5000))).unwrap();
    assert!(read_impedance(&bad).is_err(), "over-long lines must be refused");

    let an = dir.join("an.csv");
    std::fs::write(&an, "id;List of analysis (s)\n1;[31.3, 165.6]\n2;[19.4]\n").unwrap();
    let a = read_analyses(&an).unwrap();
    assert_eq!(a, vec![(1, vec![31.3, 165.6]), (2, vec![19.4])]);

    let br = dir.join("br.csv");
    std::fs::write(&br, "Cycle number;Time (s);Vi (mL);Vt (mL)\n1;0 days 00:01:15;875;305.0\n").unwrap();
    assert_eq!(read_breaths(&br).unwrap(), vec![75.0]);

    assert_eq!(case_id("intervention_004"), Some(4));
    assert_eq!(case_id("intervention_4"), None);
    assert_eq!(case_id("../etc"), None);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn reference_and_detector_agree_on_a_clean_synthetic_recording() {
    let case = synthetic_case();
    let reference = build(&case, &RefSettings::default());
    assert_eq!(reference.runs.len(), 1, "one run of compressions");
    let run = reference.runs[0];
    assert!((run.start as f64 / 125.0 - 4.0).abs() < 0.5);
    assert!((run.end as f64 / 125.0 - 24.0).abs() < 0.5);
    let det = run_detector(&case, Settings::default()).unwrap();
    let s = score(&case, &reference, &det);
    assert_eq!(s.missed_s, 0.0, "no hands-off answers during the compressions");
    assert!(s.ref_peaks_found as f64 >= 0.95 * s.ref_peaks as f64);
    assert_eq!(s.analyses, vec![(27.0, ChestState::HandsOff)]);
    assert!(s.onset_s[0] < 0.6, "onset {:?}", s.onset_s);
    assert!(s.offset_s[0] < 2.0, "offset {:?}", s.offset_s);
}

#[test]
fn real_development_cases_if_the_data_is_present() {
    let root = aed_cpr_eval::data::default_root();
    if !root.join("Rhythm_analysis.csv").exists() {
        eprintln!("SKIPPED: Paris Fire Brigade data not found at {} (set AED_CPR_DATA)", root.display());
        return;
    }
    // Two development cases only: the held-out half is never touched by tests.
    let names: Vec<String> = Half::Development.cases().unwrap().into_iter().take(2).collect();
    let cases = aed_cpr_eval::data::load(&root, &names).unwrap();
    for c in cases {
        let c = c.unwrap();
        let reference = build(&c, &RefSettings::default());
        let det = run_detector(&c, Settings::default()).unwrap();
        let s = score(&c, &reference, &det);
        assert!(s.ref_comp_s > 60.0, "{}: reference found compressions", c.name);
        assert!(s.missed_s / s.ref_comp_s < 0.05, "{}: missed {:.1} s", c.name, s.missed_s);
    }
}
