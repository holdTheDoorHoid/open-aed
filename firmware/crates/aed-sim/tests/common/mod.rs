// SPDX-License-Identifier: GPL-3.0-or-later
//! Helpers shared by the integration tests.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use aed_core::{Prompt, RescueState};
use aed_sim::report::{summary, timeline};
use aed_sim::{by_name, run, Expect, Rule, RunResult, Scenario};

/// Run a scenario, printing its summary (visible with --nocapture).
pub fn run_scenario(s: &Scenario) -> RunResult {
    let r = run(s).expect("configuration accepted");
    println!("{}", summary(&r));
    r
}

/// Run a named scenario.
pub fn run_named(name: &str) -> RunResult {
    run_scenario(&by_name(name).unwrap_or_else(|| panic!("no scenario {name}")))
}

/// Fail with the whole timeline, so a failure can be read without re-running.
pub fn fail(r: &RunResult, why: &str) -> ! {
    panic!("{why}\n\n{}\n{}", timeline(r), summary(r));
}

/// Assert no invariant was broken, apart from the listed rules.
pub fn assert_invariants_except(r: &RunResult, allowed: &[Rule]) {
    let bad: Vec<_> = r
        .violations
        .iter()
        .filter(|v| !allowed.contains(&v.rule))
        .collect();
    if !bad.is_empty() {
        fail(r, &format!("safety invariants broken: {bad:?}"));
    }
}

/// Assert every invariant held.
pub fn assert_invariants(r: &RunResult) {
    assert_invariants_except(r, &[]);
}

/// Check the scenario's declared safety outcome.
pub fn assert_expected(r: &RunResult) {
    let empty = f64::from(r.config.empty_voltage.0);
    let ok = |cond: bool, what: &str| {
        if !cond {
            fail(r, what);
        }
    };
    match r.scenario.expect {
        Expect::Shocks(n) => {
            ok(r.shocks.len() == n, &format!("expected {n} shock(s)"));
            ok(r.final_fault.is_none(), "expected no fault");
        }
        Expect::NeverCharges => {
            ok(r.charges_started == 0, "must never start charging for a shock");
            ok(r.shocks.is_empty(), "must never shock");
            // Only the self-test's small charge (400 V) is allowed.
            let self_test = f64::from(r.config.self_test_voltage.0);
            ok(r.max_cap_v <= self_test, "capacitor charged above the self-test voltage");
        }
        Expect::AbortedNoShock => {
            ok(r.shocks.is_empty(), "must not shock");
            ok(r.charges_started >= 1, "a shock should have been prepared");
            ok(r.entered(RescueState::Disarming), "must go through Disarming");
            ok(r.final_state != RescueState::Fault, "abort is not a fault");
            ok(r.final_cap_v < empty, "capacitor must end empty");
        }
        Expect::FaultNoShock => {
            ok(r.shocks.is_empty(), "must not shock");
            ok(r.final_state == RescueState::Fault, "must end in Fault");
            ok(r.final_cap_v < empty, "capacitor must end empty");
            ok(
                r.last_prompt() == Some(Prompt::DeviceFaultContinueCpr),
                "last prompt must be 'Device fault. Continue CPR.'",
            );
        }
        Expect::EndsInFault => ok(r.final_state == RescueState::Fault, "must end in Fault"),
        Expect::NoShock => ok(r.shocks.is_empty(), "must not shock"),
        Expect::InvariantsOnly => {}
    }
}
