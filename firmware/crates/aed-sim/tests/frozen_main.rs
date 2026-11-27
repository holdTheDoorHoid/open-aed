// SPDX-License-Identifier: GPL-3.0-or-later
//! Finding F-3 (fixed): a main controller that freezes while the capacitor
//! is charged, charging or about to fire.
//!
//! Two design changes cover it: the charger cannot run while the discharge
//! path is active (`aed_core::interlock::charger_may_run`, a requirement on
//! the future hardware), and the supervisor has its own fault alarm.
//! SIMULATION ONLY.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_core::RescueState;
use aed_sim::{by_name, run, Action, Fault, RunResult, Scenario, Simulation, Trigger};
use common::{assert_invariants, fail};

fn freeze_at(state: RescueState, after_ms: u32, extra: &[Fault]) -> Scenario {
    let mut s = by_name("adult-vf").unwrap();
    s.name = format!("freeze-in-{state:?}");
    for &f in extra {
        s.script.push((Trigger::At(0), Action::Fault(f)));
    }
    s.script.push((
        Trigger::AfterState {
            state,
            nth: 1,
            after_ms,
        },
        Action::Fault(Fault::MainFreeze),
    ));
    s.duration_ms = 60_000;
    s
}

/// Runs the scenario step by step and checks, from the freeze on: the
/// capacitor falls below the empty voltage within the heartbeat timeout
/// plus the time the discharge load needs; the charger is physically off
/// from then on; the supervisor's alarm comes on; and nothing fires.
fn check_frozen(s: &Scenario) -> RunResult {
    let mut sim = Simulation::new(s).unwrap();
    let config = aed_core::Config::default();
    let empty = f64::from(config.empty_voltage.0);
    // Heartbeat timeout, one heartbeat period, then 1,950 V through 2 kΩ and
    // 100 µF to below 50 V (about 0.75 s): allow 1.5 s for that part.
    let limit = config.heartbeat_timeout_ms + config.heartbeat_period_ms + 1_500;
    let mut frozen_at = None;
    let mut emptied_at = None;
    let mut shocks_at_freeze = 0;
    while sim.plant().borrow().now_ms < s.duration_ms {
        sim.step();
        let p = sim.plant().borrow();
        if frozen_at.is_none() && p.has_fault(Fault::MainFreeze) {
            frozen_at = Some(p.now_ms);
            shocks_at_freeze = p.shocks.len();
        }
        if let Some(t) = frozen_at {
            if emptied_at.is_none() && p.cap_v < empty {
                emptied_at = Some(p.now_ms);
            }
            if p.now_ms > t + limit {
                assert!(p.cap_v < empty, "{}: {:.0} V {} ms after the freeze", s.name, p.cap_v, p.now_ms - t);
                assert!(!p.charger_running(), "{}: charger still running", s.name);
                assert!(p.supervisor_alarm, "{}: supervisor alarm off", s.name);
            }
        }
    }
    let r = sim.run();
    let frozen_at = frozen_at.unwrap_or_else(|| fail(&r, "the freeze never happened"));
    if r.shocks.len() != shocks_at_freeze {
        fail(&r, "a shock fired after the main controller froze");
    }
    let emptied = emptied_at.unwrap_or_else(|| fail(&r, "capacitor never emptied"));
    println!(
        "{}: frozen at {} ms, capacitor below {empty:.0} V {} ms later, alarm at {:?} ms (SIMULATED)",
        s.name,
        frozen_at,
        emptied - frozen_at,
        r.supervisor_alarm_at
    );
    assert_invariants(&r);
    r
}

#[test]
fn frozen_main_while_charging_empties_the_capacitor_stops_the_charger_and_sounds_the_alarm() {
    let r = check_frozen(&freeze_at(RescueState::Charging, 3_000, &[]));
    assert!(r.shocks.is_empty());
}

#[test]
fn frozen_main_during_countdown_empties_the_capacitor_stops_the_charger_and_sounds_the_alarm() {
    let r = check_frozen(&freeze_at(RescueState::Countdown, 2_000, &[]));
    assert!(r.shocks.is_empty());
}

#[test]
fn frozen_main_while_delivering_empties_the_capacitor_and_sounds_the_alarm() {
    // To freeze in Delivering before the shock, the bridge must not fire at
    // once: here it fails to fire, so the request stays raised and pending
    // when the main controller freezes.
    let r = check_frozen(&freeze_at(RescueState::Delivering, 0, &[Fault::BridgeFailsToFire]));
    assert!(r.shocks.is_empty());
}

#[test]
fn without_the_charger_rule_a_frozen_main_leaves_the_capacitor_charged() {
    // The evidence for the hardware rule: switch it off and finding F-3 returns.
    let mut s = by_name("fault-main-freeze").unwrap();
    s.plant.charger_gated_by_discharge = false;
    let r = run(&s).unwrap();
    assert!(
        r.final_cap_v > 200.0,
        "expected the charger to hold the capacitor up, got {:.0} V",
        r.final_cap_v
    );
}
