// SPDX-License-Identifier: GPL-3.0-or-later
//! The shock sequence (D-59): the hardware rules as truth tables, and how
//! each controller follows the sequence. Consent starts a shock; once the
//! hardware has latched it, the pulse completes, and the readings that the
//! shock itself disturbs (pads open, voltage falling) never withdraw
//! consent.

use super::mock::{Bench, BoardChest};
use crate::events::Event;
use crate::hal::{BridgeTestOutcome, BridgeTestStep, RelayPosition, SequenceEnd, SequenceStatus};
use crate::interlock::{test_rail_may_rise, Latch, LatchInputs, LatchTiming};
use crate::link::ShockCheck;
use crate::rescue::{AbortReason, FaultKind, RescueState, ShockOutcome};
use crate::Config;
use aed_types::RhythmDecision;
use std::vec::Vec;

const T: LatchTiming = LatchTiming::PROPOSED;
const DT: u32 = 10;

/// Every combination of the five consent-side inputs.
fn consent_inputs() -> Vec<LatchInputs> {
    let mut v = Vec::new();
    for bits in 0u8..32 {
        v.push(LatchInputs {
            request: bits & 1 != 0,
            permit: bits & 2 != 0,
            test_permit: bits & 4 != 0,
            discharge_active: bits & 8 != 0,
            below_test_voltage: bits & 16 != 0,
            ..LatchInputs::default()
        });
    }
    v
}

/// The consent rule written out independently of the code: a shock rail
/// with request, permit, no test permit, both holds; a test rail with
/// request, test permit, no shock permit, both holds, below the test limit.
fn expected_consent(i: &LatchInputs) -> Option<bool> {
    let held = !i.discharge_active;
    if i.request && i.permit && !i.test_permit && held {
        Some(false)
    } else if i.request && i.test_permit && !i.permit && held && i.below_test_voltage {
        Some(true)
    } else {
        None
    }
}

fn latch(status: SequenceStatus, test: bool) -> Latch {
    Latch {
        status,
        test,
        in_status_us: 0,
        consent_absent_us: 0,
        phase1_done: false,
        phase2_us: None,
    }
}

// ----- truth tables (the requirements the circuit is drawn to) ----------------

#[test]
fn test_rail_truth_table() {
    for bits in 0u8..64 {
        let b = |n: u8| bits & (1 << n) != 0;
        let (req, test, permit, dis, locked, below) = (b(0), b(1), b(2), b(3), b(4), b(5));
        assert_eq!(
            test_rail_may_rise(req, test, permit, dis, locked, below),
            req && test && !permit && !dis && !locked && below,
            "bits {bits:06b}"
        );
    }
}

#[test]
fn idle_rises_only_with_full_consent_truth_table() {
    for i in consent_inputs() {
        let next = Latch::IDLE.step(i, T, DT);
        match expected_consent(&i) {
            Some(test) => {
                assert_eq!(next.status, SequenceStatus::Armed, "{i:?}");
                assert_eq!(next.test, test, "{i:?}");
            }
            None => assert_eq!(next.status, SequenceStatus::Idle, "{i:?}"),
        }
        // Drive, relay and timer inputs never raise the rail on their own.
        let pushed = LatchInputs {
            drive: true,
            relay_on_bridge: true,
            phases_done: true,
            over_current: true,
            ..i
        };
        assert_eq!(
            Latch::IDLE.step(pushed, T, DT).status == SequenceStatus::Armed,
            expected_consent(&i).is_some(),
            "{i:?}"
        );
    }
}

#[test]
fn armed_drops_at_once_when_any_consent_is_withdrawn_truth_table() {
    for test in [false, true] {
        for i in consent_inputs() {
            for (drive, relay) in [(false, false), (true, false), (false, true), (true, true)] {
                let i = LatchInputs {
                    drive,
                    relay_on_bridge: relay,
                    ..i
                };
                let next = latch(SequenceStatus::Armed, test).step(i, T, DT);
                let keeps = expected_consent(&i) == Some(test);
                let starts = keeps && drive && (test || relay);
                let expect = if !keeps {
                    SequenceStatus::Idle
                } else if starts {
                    SequenceStatus::Active
                } else {
                    SequenceStatus::Armed
                };
                assert_eq!(next.status, expect, "test {test} {i:?}");
            }
        }
    }
}

#[test]
fn a_shock_sequence_starts_only_with_the_relay_on_the_bridge() {
    let consent = LatchInputs {
        request: true,
        permit: true,
        drive: true,
        ..LatchInputs::default()
    };
    let armed = latch(SequenceStatus::Armed, false);
    assert_eq!(armed.step(consent, T, DT).status, SequenceStatus::Armed);
    let on_bridge = LatchInputs {
        relay_on_bridge: true,
        ..consent
    };
    assert_eq!(armed.step(on_bridge, T, DT).status, SequenceStatus::Active);
}

#[test]
fn once_active_the_consent_lines_are_ignored_truth_table() {
    // D-59: whatever the request, permit, test permit and holds do, an
    // active sequence carries on until its phases end.
    for test in [false, true] {
        for i in consent_inputs() {
            let running = LatchInputs { drive: true, ..i };
            let next = latch(SequenceStatus::Active, test).step(running, T, DT);
            assert_eq!(next.status, SequenceStatus::Active, "test {test} {i:?}");
            let done = LatchInputs {
                phases_done: true,
                ..i
            };
            let next = latch(SequenceStatus::Active, test).step(done, T, DT);
            assert_eq!(
                next.status,
                SequenceStatus::Ended(SequenceEnd::Completed),
                "test {test} {i:?}"
            );
        }
    }
}

#[test]
fn over_current_comes_before_the_cap_which_comes_before_completion() {
    for (oc, cap, done) in [
        (true, true, true),
        (true, false, false),
        (false, true, true),
        (false, true, false),
        (false, false, true),
    ] {
        let l = Latch {
            in_status_us: if cap { T.time_cap_us } else { 0 },
            // Past phase 1, so the overall cap (not the phase-1 cap) applies.
            phase1_done: true,
            ..latch(SequenceStatus::Active, false)
        };
        let i = LatchInputs {
            over_current: oc,
            phases_done: done,
            ..LatchInputs::default()
        };
        let expect = if oc {
            SequenceEnd::OverCurrentTripped
        } else if cap {
            SequenceEnd::TimeCapTripped
        } else {
            SequenceEnd::Completed
        };
        assert_eq!(l.step(i, T, DT).status, SequenceStatus::Ended(expect));
    }
}

#[test]
fn the_time_cap_is_not_retriggerable() {
    // A timer that keeps toggling its drive (review R-4) does not extend
    // the cap: it is measured from the first conducting moment.
    let mut l = latch(SequenceStatus::Active, false);
    let mut t = 0u32;
    while l.status == SequenceStatus::Active {
        let drive = (t / 500).is_multiple_of(2);
        l = l.step(
            LatchInputs {
                drive,
                request: drive,
                permit: !drive,
                ..LatchInputs::default()
            },
            T,
            DT,
        );
        t += DT;
        assert!(t <= T.time_cap_us + DT, "ran {t} µs");
    }
    assert_eq!(l.status, SequenceStatus::Ended(SequenceEnd::TimeCapTripped));
    assert!(t >= T.time_cap_us, "cut after {t} µs");
}

#[test]
fn the_permit_duration_cap_locks_out_a_rail_held_without_a_sequence() {
    for test in [false, true] {
        let i = LatchInputs {
            request: true,
            permit: !test,
            test_permit: test,
            below_test_voltage: true,
            ..LatchInputs::default()
        };
        let mut l = Latch::IDLE.step(i, T, DT);
        assert_eq!(l.status, SequenceStatus::Armed);
        let mut t = 0;
        while l.status == SequenceStatus::Armed {
            l = l.step(i, T, DT);
            t += DT;
        }
        assert_eq!(l.status, SequenceStatus::LockedOut, "test {test}");
        assert!(
            (T.permit_cap_us - DT..=T.permit_cap_us).contains(&t),
            "{t} µs"
        );
    }
}

#[test]
fn lockout_clears_only_when_every_consent_is_absent_for_the_rearm_time_truth_table() {
    for start in [
        SequenceStatus::Ended(SequenceEnd::Completed),
        SequenceStatus::Ended(SequenceEnd::TimeCapTripped),
        SequenceStatus::Ended(SequenceEnd::OverCurrentTripped),
        SequenceStatus::LockedOut,
    ] {
        for i in consent_inputs() {
            // A second request within the same permit (or any line still
            // up) is locked out however long it lasts.
            let mut l = latch(start, false);
            let any = i.request || i.permit || i.test_permit;
            for _ in 0..(T.rearm_us / DT + 10) {
                l = l.step(i, T, DT);
            }
            if any {
                assert_eq!(l.status, start, "{start:?} {i:?}");
            } else {
                assert_eq!(l.status, SequenceStatus::Idle, "{start:?} {i:?}");
            }
        }
        // A blip of consent restarts the re-arm count.
        let mut l = latch(start, false);
        let off = LatchInputs::default();
        for _ in 0..(T.rearm_us / DT - 1) {
            l = l.step(off, T, DT);
        }
        l = l.step(
            LatchInputs {
                permit: true,
                ..off
            },
            T,
            DT,
        );
        l = l.step(off, T, DT);
        assert_eq!(l.status, start, "the count must restart after a blip");
    }
}

#[test]
fn the_relay_coil_is_powered_only_by_a_real_rail() {
    for status in [
        SequenceStatus::Idle,
        SequenceStatus::Armed,
        SequenceStatus::Active,
        SequenceStatus::Ended(SequenceEnd::Completed),
        SequenceStatus::LockedOut,
    ] {
        for test in [false, true] {
            let l = latch(status, test);
            let rail = matches!(status, SequenceStatus::Armed | SequenceStatus::Active);
            assert_eq!(l.rail_up(), rail);
            assert_eq!(l.relay_coil(), rail && !test, "{status:?} test {test}");
        }
    }
}

// ----- the main controller and the supervisor against the sequence --------------

/// Run to the moment the main controller raises its request.
fn to_delivering(b: &mut Bench) {
    b.run_to_countdown();
    assert!(
        b.run_until_state(RescueState::Delivering, 5_000),
        "{:?}",
        b.state_sequence()
    );
}

fn outcome(b: &Bench) -> Option<ShockOutcome> {
    b.rescue.events().iter().find_map(|e| match e.event {
        Event::Shock(o) => Some(o),
        _ => None,
    })
}

#[test]
fn pads_reading_open_while_the_relay_is_on_the_bridge_does_not_withdraw_consent() {
    // Review R-1: both controllers' pad readings go open the moment the
    // relay moves (the mock reads 10,000 Ω while the relay is away), and
    // the voltage falls through the pulse. The shock is still delivered,
    // with the full plan, and neither controller withdrew before the end.
    let mut b = Bench::new();
    b.board.relay_transfer_us = 8_000;
    to_delivering(&mut b);
    assert!(b.run_until(200, |b| b.board.relay == RelayPosition::Bridge));
    assert!(b.board.permit, "permit withdrawn when the pads read open");
    assert!(
        b.board.request.is_some(),
        "request withdrawn when the pads read open"
    );
    assert!(b.run_until_state(RescueState::Cpr, 1_000));
    assert_eq!(b.shocks_fired(), 1);
    assert_eq!(outcome(&b), Some(ShockOutcome::Delivered));
    assert!(!b
        .sup
        .events()
        .iter()
        .any(|e| matches!(e.event, Event::SupervisorPermitWithdrawn(_))));
    b.run_ms(200);
    assert_eq!(b.sup.shock_check(), Some(ShockCheck::Delivered));
    assert!(b.has_event(|e| *e == Event::SupervisorShockCheckReported(ShockCheck::Delivered)));
    b.assert_no_violations();
}

#[test]
fn a_realistic_front_end_recovery_does_not_declare_the_pads_off() {
    // The pads read open for 1.4 s after the relay returns (front ends
    // recovering). The device must not call that "pads off".
    let mut b = Bench::new();
    b.board.front_end_recovery_ms = 1_400;
    to_delivering(&mut b);
    assert!(b.run_until_state(RescueState::Cpr, 1_000));
    b.run_ms(3_000);
    assert_eq!(b.state(), RescueState::Cpr);
    assert!(!b.has_event(|e| *e == Event::PadsOff));
    assert!(b.has_event(|e| *e == Event::FrontEndsRecovered));
    assert!(!b.has_event(|e| *e == Event::PadsNotBackAfterShock));
}

#[test]
fn front_ends_that_do_not_recover_in_time_lead_to_attach_pads_not_a_fault() {
    let mut b = Bench::new();
    b.board.front_end_recovery_ms = 5_000;
    to_delivering(&mut b);
    assert!(b.run_until_state(RescueState::Cpr, 1_000));
    assert!(b.run_until_state(RescueState::AttachPads, 3_000));
    assert!(b.has_event(|e| *e == Event::PadsNotBackAfterShock));
    assert_eq!(b.rescue.fault(), None);
}

#[test]
fn recovery_time_is_configurable() {
    let config = Config {
        front_end_recovery_ms: 3_000,
        ..super::cfg()
    };
    let mut b = Bench::with(config, super::mock::FakePlanner::default());
    b.board.front_end_recovery_ms = 2_800;
    to_delivering(&mut b);
    assert!(b.run_until_state(RescueState::Cpr, 1_000));
    b.run_ms(5_000);
    assert_eq!(b.state(), RescueState::Cpr);
    assert!(!b.has_event(|e| *e == Event::PadsOff));
}

#[test]
fn cancel_before_the_sequence_starts_withdraws_and_after_it_starts_does_nothing() {
    // Before: the rail is up, the relay still moving.
    let mut b = Bench::new();
    b.board.relay_transfer_us = 20_000;
    to_delivering(&mut b);
    assert!(b.run_until(100, |b| b.board.latch.status == SequenceStatus::Armed));
    b.board.cancel = true;
    b.run_ms(100);
    b.board.cancel = false;
    assert_eq!(b.shocks_fired(), 0);
    assert_eq!(
        outcome(&b),
        Some(ShockOutcome::Withdrawn(AbortReason::Cancelled))
    );
    // After: the sequence is active.
    let mut b = Bench::new();
    to_delivering(&mut b);
    assert!(b.run_until(100, |b| b.board.latch.status == SequenceStatus::Active));
    b.board.cancel = true;
    b.run_ms(100);
    b.board.cancel = false;
    assert_eq!(b.shocks_fired(), 1);
    assert_eq!(outcome(&b), Some(ShockOutcome::Delivered));
}

#[test]
fn chest_movement_before_the_rail_rises_still_withdraws_the_request() {
    // D-37 holds up to the moment the pads leave the front ends.
    let mut b = Bench::with_chest(super::cfg());
    b.board.relay_transfer_us = 4_000;
    b.board.rhythm = RhythmDecision::Shockable;
    assert!(b.run_until_state(RescueState::Countdown, 60_000));
    // The supervisor refuses (its own pad reading out of range), so the
    // request waits with the rail down.
    b.board.sup_pad_ohms = Some(300);
    assert!(
        b.run_until_state(RescueState::Delivering, 5_000),
        "{:?} {:?}",
        b.state_sequence(),
        b.rescue.fault()
    );
    b.board.set_chest(BoardChest::Compressing);
    b.run_ms(100);
    assert_eq!(b.shocks_fired(), 0);
    assert_eq!(
        outcome(&b),
        Some(ShockOutcome::Withdrawn(AbortReason::HandsOnChest))
    );
}

type Case = (
    &'static str,
    fn(&mut Bench),
    ShockOutcome,
    Option<FaultKind>,
);

#[test]
fn each_way_a_sequence_can_go_wrong_has_its_own_outcome() {
    let cases: Vec<Case> = std::vec![
        (
            "relay stays at rest",
            |b| b.board.relay_stuck_front = true,
            ShockOutcome::RelayDidNotTransfer,
            None
        ),
        (
            "bridge never starts",
            |b| b.board.bridge_no_start = true,
            ShockOutcome::NotStarted,
            None
        ),
        // D-63: a stuck phase timer is now cut by the phase-1 cap.
        (
            "timer stuck: cap",
            |b| b.board.timer_stuck = true,
            ShockOutcome::PhaseCapTripped,
            Some(FaultKind::DeliveryFault)
        ),
        (
            "over-current",
            |b| b.board.output_fault = true,
            ShockOutcome::OverCurrentTripped,
            Some(FaultKind::DeliveryFault)
        ),
        (
            "capacitor did not fall",
            |b| b.board.residual = 1.0,
            ShockOutcome::NotDelivered,
            None
        ),
        (
            "fell to nothing",
            |b| b.board.residual = 0.01,
            ShockOutcome::ResidualImplausible,
            Some(FaultKind::DeliveryFault)
        ),
        (
            "supervisor refuses (its own pad reading out of range)",
            |b| b.board.sup_pad_ohms = Some(300),
            ShockOutcome::TimedOut,
            None
        ),
        (
            "permit cap of the hardware is shorter than the start",
            |b| {
                b.board.relay_transfer_us = 30_000;
                b.board.latch_timing.permit_cap_us = 20_000;
            },
            ShockOutcome::RefusedByInterlock,
            None
        ),
    ];
    let mut seen = Vec::new();
    for (name, setup, expect, fault) in cases {
        let mut b = Bench::new();
        b.run_to_countdown();
        setup(&mut b);
        assert!(
            b.run_until(5_000, |b| outcome(b).is_some()),
            "{name}: no outcome"
        );
        assert_eq!(outcome(&b), Some(expect), "{name}");
        b.run_ms(2_000);
        if let Some(f) = fault {
            assert_eq!(b.rescue.fault(), Some(f), "{name}");
        } else {
            assert_eq!(b.rescue.fault(), None, "{name}");
            assert_ne!(b.state(), RescueState::Fault, "{name}");
        }
        if expect != ShockOutcome::TimedOut {
            assert!(!seen.contains(&expect), "{name}");
        }
        seen.push(expect);
    }
}

#[test]
fn a_relay_that_does_not_return_is_a_fault() {
    let mut b = Bench::new();
    b.board.relay_stuck_bridge = true;
    to_delivering(&mut b);
    assert!(b.run_until(2_000, |b| b.state() == RescueState::Fault));
    assert_eq!(b.rescue.fault(), Some(FaultKind::PatientRelayStuck));
    // The shock itself was delivered and recorded before the fault.
    assert_eq!(outcome(&b), Some(ShockOutcome::Delivered));
}

#[test]
fn a_permit_line_stuck_on_is_found_at_switch_on() {
    // A shorted permit path: the request alone raises the rail.
    let mut b = Bench::new();
    b.board.permit_stuck_on = true;
    assert!(b.run_until(3_000, |b| b.rescue.self_test_result().is_some()));
    assert_eq!(
        b.rescue.self_test_result(),
        Some(Err(
            crate::selftest::SelfTestFailure::InterlockPassedRequestAlone
        ))
    );
}

#[test]
fn a_lockout_that_never_clears_is_a_fault_at_the_next_request_not_a_false_delivery() {
    // The permit line sticks on after the switch-on self-test: after the
    // first shock the hardware stays locked out, and the second request
    // finds the sequence logic not idle. It must not read the stale
    // "ended" status as a second delivered shock.
    let mut b = Bench::new();
    to_delivering(&mut b);
    b.board.permit_stuck_on = true;
    assert!(b.run_until_state(RescueState::Cpr, 1_000));
    assert_eq!(b.rescue.counters().shocks_delivered, 1);
    b.board.rhythm = RhythmDecision::Shockable;
    assert!(b.run_until(200_000, |b| b.state() == RescueState::Fault));
    assert_eq!(b.rescue.fault(), Some(FaultKind::InterlockNotIdle));
    assert_eq!(b.rescue.counters().shocks_delivered, 1);
    assert_eq!(b.shocks_fired(), 1);
}

// ----- the supervisor's test permit and the interlock self-test (D-61) ----------

#[test]
fn the_switch_on_self_test_exercises_the_interlock_without_moving_the_relay() {
    let mut b = Bench::new();
    assert!(b.run_until(3_000, |b| b.rescue.self_test_result().is_some()));
    assert_eq!(b.rescue.self_test_result(), Some(Ok(())));
    // One latched test sequence (into the test load), the relay never moved.
    assert_eq!(b.board.test_fires.len(), 1);
    assert!(b.board.fires.is_empty());
    assert!(b
        .sup
        .events()
        .iter()
        .any(|e| e.event == Event::SupervisorTestPermitted));
    assert!(
        b.board.requests.is_empty(),
        "no shock request in the self-test"
    );
}

#[test]
fn a_stuck_bridge_switch_is_found_at_switch_on() {
    for step in BridgeTestStep::ALL {
        for o in [
            BridgeTestOutcome::DoesNotTurnOn,
            BridgeTestOutcome::DoesNotTurnOff,
        ] {
            let mut b = Bench::new();
            b.board.bridge_switch_fault = Some((step, o));
            assert!(b.run_until(3_000, |b| b.rescue.self_test_result().is_some()));
            assert_eq!(
                b.rescue.self_test_result(),
                Some(Err(crate::selftest::SelfTestFailure::BridgeSwitch(step, o)))
            );
            assert_eq!(b.state(), RescueState::Fault);
        }
    }
}

#[test]
fn a_relay_that_does_not_rest_on_the_front_ends_is_found_at_switch_on() {
    let mut b = Bench::new();
    b.board.relay = RelayPosition::Between;
    b.board.relay_stuck_bridge = true;
    assert!(b.run_until(3_000, |b| b.rescue.self_test_result().is_some()));
    assert_eq!(
        b.rescue.self_test_result(),
        Some(Err(crate::selftest::SelfTestFailure::RelayNotAtFrontEnds))
    );
}

#[test]
fn a_failed_open_discharge_path_is_found_at_switch_on() {
    use crate::selftest::SelfTestFailure as F;
    for (a, expect) in [
        (true, F::DischargePathASlow),
        (false, F::DischargePathBSlow),
    ] {
        let mut b = Bench::new();
        if a {
            b.board.path_a_broken = true;
        } else {
            b.board.path_b_broken = true;
        }
        assert!(b.run_until(5_000, |b| b.rescue.self_test_result().is_some()));
        assert_eq!(b.rescue.self_test_result(), Some(Err(expect)));
        // The other path still emptied the capacitor.
        b.run_ms(3_000);
        assert!(b.board.cap_v < 50.0);
        assert_ne!(b.rescue.fault(), Some(FaultKind::CapacitorNotEmptied));
    }
}

// ----- D-60: the adult/child switch's two contacts -------------------------------

#[test]
fn mode_switch_contacts_that_disagree_are_recorded_and_shown_not_ready_and_the_primary_contact_sets_the_mode(
) {
    use aed_types::PatientMode;
    let mut b = Bench::new();
    b.board.mode = PatientMode::Child;
    b.board.mode_ceiling = Some(PatientMode::Adult);
    b.board.rhythm = RhythmDecision::Shockable;
    assert!(b.run_until(2_000, |b| b.rescue.self_test_result().is_some()));
    b.run_ms(1_000);
    assert!(b.rescue.mode_contacts_disagree());
    assert!(
        !b.board.ready,
        "ready indicator on with the contacts disagreeing"
    );
    assert!(b.has_event(|e| *e
        == Event::ModeContactsDisagree {
            primary: PatientMode::Child,
            ceiling: PatientMode::Adult
        }));
    // The rescue carries on, in the primary contact's mode (child).
    assert!(b.run_until_state(RescueState::Cpr, 60_000));
    assert_eq!(b.shocks_fired(), 1);
    assert_eq!(b.board.requests[0].plan.charge_voltage.0, 1_000);
    // A brief disagreement while the switch moves is not recorded.
    let mut b = Bench::new();
    b.run_ms(1_000);
    b.board.mode_ceiling = Some(PatientMode::Child);
    b.run_ms(100);
    b.board.mode_ceiling = None;
    b.run_ms(1_000);
    assert!(!b.has_event(|e| matches!(e, Event::ModeContactsDisagree { .. })));
    assert!(b.board.ready);
}
