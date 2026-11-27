// SPDX-License-Identifier: GPL-3.0-or-later
//! Checks against the real PhysioNet files.
//!
//! The data are not in the repository. These tests look for them in
//! `$AED_EVAL_DATA`, or else in `data/physionet` at the repository root, and
//! **skip (pass without checking) when they are absent**, printing a note.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic, missing_docs)]

use std::path::PathBuf;

use aed_eval::condition::Resampler;
use aed_eval::eval::{evaluate_record, prepare, FS};
use aed_eval::truth::{Db, Label};
use aed_eval::wfdb::{load_record, read_records_list};

fn data_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("AED_EVAL_DATA").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/physionet"),
        PathBuf::from,
    );
    if dir.join("cudb/RECORDS").is_file() && dir.join("vfdb/RECORDS").is_file() && dir.join("mitdb/RECORDS").is_file() {
        Some(dir)
    } else {
        eprintln!("SKIPPED: PhysioNet data not found at {} (set AED_EVAL_DATA)", dir.display());
        None
    }
}

#[test]
fn cudb_cu01_known_facts() {
    let Some(d) = data_dir() else { return };
    let r = load_record(&d.join("cudb"), "cu01", "atr").unwrap();
    assert_eq!(r.header.record.signal_count, 1);
    assert_eq!(r.header.record.sampling_hz, 250.0);
    assert_eq!(r.header.record.samples_per_signal, Some(127_232));
    assert_eq!(r.samples[0].len(), 127_232);
    let s = &r.header.signals[0];
    assert_eq!(s.gain, 400.0);
    assert_eq!(s.initial_value, Some(-109));
    assert_eq!(r.samples[0][0], -109, "first sample equals the header's initial value");
    assert_eq!(s.checksum, Some(-28468));
    assert_eq!(r.checks[0].checksum_ok, Some(true));
    // cu01 has one VF episode from about 214 s to the end.
    let p = prepare(Db::Cudb, &d.join("cudb"), "cu01").unwrap();
    let vf: Vec<_> = p.timeline.spans.iter().filter(|s| s.label == Label::Vf).collect();
    assert_eq!(vf.len(), 1);
    assert!((vf[0].start - 214.17).abs() < 0.05, "{}", vf[0].start);
    assert!((vf[0].end - 508.92).abs() < 0.01, "{}", vf[0].end);
}

#[test]
fn mitdb_100_known_facts_and_resampled_length() {
    let Some(d) = data_dir() else { return };
    let r = load_record(&d.join("mitdb"), "100", "atr").unwrap();
    assert_eq!(r.header.record.sampling_hz, 360.0);
    assert_eq!(r.samples.len(), 2);
    assert_eq!(r.samples[0].len(), 650_000);
    assert_eq!(r.samples[0][0], 995);
    assert_eq!(r.samples[1][0], 1011);
    assert_eq!(r.header.signals[0].checksum, Some(-22131));
    assert_eq!(r.header.signals[1].checksum, Some(20052));
    assert!(r.checks.iter().all(|c| c.checksum_ok == Some(true)));
    assert_eq!(r.header.signals[0].baseline, 1024);
    let p = prepare(Db::Mitdb, &d.join("mitdb"), "100").unwrap();
    assert_eq!(p.channels[0].samples.len(), (649_999 * 25) / 36 + 1);
    // First sample: (995 - 1024) * 1000 / 200 = -145 µV.
    assert!((i32::from(p.channels[0].samples[0]) + 145).abs() <= 2);
    // Record 100 is almost all normal rhythm.
    let nsr: f64 = p.timeline.spans.iter().filter(|s| s.label == Label::Nsr).map(|s| s.end - s.start).sum();
    assert!(nsr > 1700.0, "{nsr}");
    let _ = Resampler::new_default(360, 250).unwrap();
}

#[test]
fn vfdb_418_known_facts() {
    let Some(d) = data_dir() else { return };
    let r = load_record(&d.join("vfdb"), "418", "atr").unwrap();
    assert_eq!(r.header.record.samples_per_signal, Some(525_000));
    assert_eq!(r.samples[1].len(), 525_000);
    assert_eq!(r.samples[0][0], -128);
    assert_eq!(r.samples[1][0], -14);
    assert_eq!(r.header.signals[0].checksum, Some(1830));
    assert_eq!(r.header.signals[1].checksum, Some(-5967));
    assert!(r.checks.iter().all(|c| c.checksum_ok == Some(true) && c.initial_value_ok == Some(true)));
    // vfdb has rhythm labels only, no beats.
    assert!(r.annotations.iter().all(|a| a.code == aed_eval::wfdb::annot::code::RHYTHM));
}

#[test]
fn every_record_loads_with_matching_checksums() {
    let Some(d) = data_dir() else { return };
    let mut n = 0;
    for (db, expect) in [(Db::Cudb, 35), (Db::Vfdb, 22), (Db::Mitdb, 48)] {
        let dir = d.join(db.name());
        let list = read_records_list(&dir).unwrap();
        assert_eq!(list.len(), expect, "{}", db.name());
        for name in list {
            let r = load_record(&dir, &name, "atr").unwrap_or_else(|e| panic!("{}/{name}: {e}", db.name()));
            for c in &r.checks {
                assert_eq!(c.checksum_ok, Some(true), "{}/{name}", db.name());
                assert_eq!(c.initial_value_ok, Some(true), "{}/{name}", db.name());
            }
            n += 1;
        }
    }
    assert_eq!(n, 105);
}

#[test]
fn end_to_end_on_one_record() {
    let Some(d) = data_dir() else { return };
    let p = prepare(Db::Cudb, &d.join("cudb"), "cu01").unwrap();
    let out = evaluate_record(&p);
    assert!(!out.segments.is_empty());
    assert_eq!(out.streams.len(), 1);
    // One streaming window per second after the first 5 s.
    let expected = p.channels[0].samples.len() / FS - 4;
    assert!((out.streams[0].len() as i64 - expected as i64).abs() <= 1);
}
