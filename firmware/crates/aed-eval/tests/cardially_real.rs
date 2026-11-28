// SPDX-License-Identifier: GPL-3.0-or-later
//! Checks against the real Cardially files.
//!
//! The data are not in the repository. These tests look for them in
//! `$AED_CARDIALLY_DATA`, or else in `data/mendeley/cardially` at the
//! repository root, and **skip (pass without checking) when they are
//! absent**, printing a note. They check facts about the files and the
//! reader, never the analyser's score (which belongs in REPORT.md).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    missing_docs
)]

use std::path::PathBuf;

use aed_eval::cardially::eval::{evaluate, PRE_S};
use aed_eval::cardially::{dataset_dir, load, unpaired, ShockOutcome};
use aed_eval::split::sha256_hex;
use aed_rhythm::Settings;

fn data_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("AED_CARDIALLY_DATA").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/mendeley/cardially"),
        PathBuf::from,
    );
    if dataset_dir(&dir).is_some() {
        Some(dir)
    } else {
        eprintln!(
            "SKIPPED: Cardially data not found at {} (set AED_CARDIALLY_DATA)",
            dir.display()
        );
        None
    }
}

#[test]
fn every_report_is_read_completely_and_consistently() {
    let Some(d) = data_dir() else { return };
    if let Ok(zip) = std::fs::read(d.join("cardially-ecg-dataset.zip")) {
        // The SHA-256 Mendeley Data lists for version 1's only file.
        assert_eq!(
            sha256_hex(&zip),
            "a3bce9bdb07a752f5d055a0ad742e227dade961b258518824069cb7c15fca735"
        );
    }
    let items = load(&d, 4).unwrap();
    assert_eq!(items.len(), 260);
    let count = |o: ShockOutcome| items.iter().filter(|i| i.outcome == o).count();
    assert_eq!(
        (
            count(ShockOutcome::Roea),
            count(ShockOutcome::NoRoea),
            count(ShockOutcome::Indeterminable)
        ),
        (56, 195, 9)
    );
    for it in &items {
        let p = it
            .pdf
            .as_ref()
            .unwrap_or_else(|e| panic!("{}: {e}", it.key()));
        assert_eq!(p.fs_hz, 200, "{}", it.key());
        assert_eq!((p.div_per_mv, p.div_per_s), (2.0, 5.0));
        assert!(
            (p.div_pt - 14.17).abs() < 0.01,
            "{}: div {}",
            it.key(),
            p.div_pt
        );
        assert_eq!(
            p.missing + p.off_grid + p.conflicts + p.clock_breaks,
            0,
            "{}",
            it.key()
        );
        assert_eq!(p.samples_uv.len(), p.strips * 2000);
        let shock = p
            .shock_delivered_s(1)
            .unwrap_or_else(|| panic!("{}: no shock mark", it.key()));
        assert!(
            (10.0..11.0).contains(&shock),
            "{}: shock at {shock}",
            it.key()
        );
    }
    // Nine reports repeat another report's ECG exactly.
    assert_eq!(items.iter().filter(|i| i.duplicate_of.is_some()).count(), 9);
    // Two text files have no PDF of the same name, and two PDFs no text file.
    let (t, p) = unpaired(&d);
    assert_eq!((t.len(), p.len()), (2, 2));
}

#[test]
fn text_files_match_the_pdf_traces_with_five_exceptions() {
    let Some(d) = data_dir() else { return };
    let items = load(&d, 4).unwrap();
    let mut n = 0;
    let mut odd: Vec<String> = Vec::new();
    for it in items.iter().filter(|i| i.duplicate_of.is_none()) {
        let r = evaluate(it, Settings::DEFAULT);
        assert!((r.pre9.as_ref().unwrap().length_s - PRE_S).abs() < 0.01);
        let Some((corr, ratio, end_before)) = r.txt_match else {
            continue;
        };
        n += 1;
        // The dataset's digitisation is the same signal, the right way up, at
        // about the same scale, ending a few tenths of a second before the mark.
        if corr > 0.85 && (0.7..1.3).contains(&ratio) {
            assert!(
                (0.0..1.0).contains(&end_before),
                "{}: ends {end_before} s before",
                it.key()
            );
        } else {
            odd.push(it.key());
        }
    }
    assert_eq!(n, 249);
    // Four text files are at about half the PDF's amplitude; one (noROEA/27218)
    // does not match its PDF at all (correlation about 0.17).
    odd.sort();
    assert_eq!(
        odd,
        [
            "ROEA/120419",
            "noROEA/128499",
            "noROEA/23315",
            "noROEA/27218",
            "noROEA/7621"
        ]
    );
}
