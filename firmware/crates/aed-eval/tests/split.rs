// SPDX-License-Identifier: GPL-3.0-or-later
//! The committed patient-wise split must not change silently.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic, missing_docs)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use aed_eval::split::{list_text, make_split, parse_list, sha256_hex, survey};

const DEV_SHA256: &str = "597e65c49bc0114f3680c1e3f49cc8d553407606e411f6bce6a3e1714d477903";
const HELD_OUT_SHA256: &str = "3e5c70316b41bdaacc76471a7a775b3b41c99a95dbbba182fb47e13a44d26601";

fn read(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("splits").join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn committed_split_is_unchanged_disjoint_and_complete() {
    let dev = read("development.txt");
    let held = read("held_out.txt");
    assert_eq!(sha256_hex(dev.as_bytes()), DEV_SHA256, "development.txt changed");
    assert_eq!(sha256_hex(held.as_bytes()), HELD_OUT_SHA256, "held_out.txt changed");
    let d: BTreeSet<_> = parse_list(&dev).unwrap().into_iter().collect();
    let h: BTreeSet<_> = parse_list(&held).unwrap().into_iter().collect();
    assert!(d.is_disjoint(&h));
    assert_eq!(d.len() + h.len(), 105);
    let side = |r: &str| d.iter().any(|(db, n)| db.name() == "mitdb" && n == r);
    assert_eq!(side("201"), side("202"), "same patient must be on one side");
}

#[test]
fn rerunning_the_rule_reproduces_the_files() {
    let dir = std::env::var_os("AED_EVAL_DATA").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/physionet"),
        PathBuf::from,
    );
    if !dir.join("mitdb/RECORDS").is_file() {
        eprintln!("SKIPPED: PhysioNet data not found at {}", dir.display());
        return;
    }
    let s = make_split(&survey(&dir).unwrap());
    assert_eq!(
        list_text("Development half: all tuning uses only these records.", &s.development),
        read("development.txt")
    );
    assert_eq!(
        list_text("Held-out half: run once, with frozen settings, at the end.", &s.held_out),
        read("held_out.txt")
    );
}
