// SPDX-License-Identifier: GPL-3.0-or-later
//! The seeded random campaign: a few hundred randomised scenarios with the
//! invariants checked at every step. SIMULATION ONLY.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use aed_sim::campaign::{campaign_text, random_scenario, run_campaign};

#[test]
fn random_campaign_breaks_no_invariant_outside_known_findings() {
    let s = run_campaign(300, 1);
    println!("{}", campaign_text(&s));
    assert_eq!(s.runs, 300);
    assert!(
        s.with_fault > 100,
        "about half the runs should have a fault"
    );
    assert!(
        s.clean(),
        "unexplained invariant violations (replay with `cargo run -p aed-sim -- random <seed>`): {:?}",
        s.failing_seeds
    );
}

#[test]
fn random_scenarios_are_repeatable() {
    assert_eq!(random_scenario(42), random_scenario(42));
    assert_ne!(random_scenario(42), random_scenario(43));
}
