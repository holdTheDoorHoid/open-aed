// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests of the supervisor on its own. The test plays the main controller by
//! hand: it writes status messages and drives the main side's hold and
//! charger lines directly, so it can tell the supervisor lies.

use super::mock::{Board, SupView};
use crate::events::Event;
use crate::interlock::bridge_may_fire;
use crate::link::{MainStatus, SupervisorFault};
use crate::rescue::RescueState;
use crate::supervisor::{PermitRefusal, ReleaseReason, Supervisor};
use crate::Config;
use aed_types::Volts;
use std::cell::Cell;
use std::rc::Rc;

use RescueState as S;

const PLANNED: u16 = 1800;

/// A board with only the supervisor attached; the test is the main controller.
struct SupBench {
    board: Board,
    sup: Supervisor,
    seq: u32,
    config: Config,
    /// The state the fake main controller claims in its heartbeats.
    claim: Option<S>,
    /// Voltage the fake main controller claims (None = its true reading).
    claim_v: Option<u16>,
    planned: u16,
}

impl SupBench {
    fn new() -> Self {
        let config = Config::default();
        SupBench {
            board: Board::new(Rc::new(Cell::new(None))),
            sup: Supervisor::new(config).unwrap(),
            seq: 0,
            config,
            claim: None,
            claim_v: None,
            planned: PLANNED,
        }
    }

    /// Fake main sends a heartbeat now.
    fn heartbeat(&mut self) {
        let Some(state) = self.claim else { return };
        self.seq += 1;
        let v = self.claim_v.unwrap_or(self.board.cap_v.round() as u16);
        let armed = matches!(state, S::Charging | S::Countdown | S::Delivering);
        self.board.status_inbox = Some(MainStatus {
            sequence: self.seq,
            state,
            capacitor_voltage: Volts(v),
            planned_voltage: Volts(if armed { self.planned } else { 0 }),
            armed_for_ms: 0,
        });
    }

    /// Advance 2 ms: physics, a heartbeat every 50 ms, supervisor tick.
    fn step(&mut self) {
        self.board.now += 2;
        self.board.step(2);
        if self.board.now.is_multiple_of(50) {
            self.heartbeat();
        }
        self.sup.tick(&mut SupView(&mut self.board));
    }

    fn run_ms(&mut self, ms: u32) {
        let end = self.board.now + ms;
        while self.board.now < end {
            self.step();
        }
    }

    /// Change the claimed state and send a heartbeat at once (as the real main
    /// controller does on every state change).
    fn claim(&mut self, s: S) {
        self.claim = Some(s);
        self.heartbeat();
    }

    /// Fake main: hold on, charge to the planned voltage, count down for the
    /// full countdown time. Leaves it at the end of the countdown.
    fn arm_and_count_down(&mut self) {
        self.board.main_hold = true;
        self.board.charger = Some(Volts(self.planned));
        self.claim(S::Charging);
        self.run_ms(9_000);
        assert!(
            self.board.cap_v > 1_750.0,
            "did not charge: {}",
            self.board.cap_v
        );
        self.claim(S::Countdown);
        self.run_ms(self.config.countdown_ms);
    }

    fn refused(&self, r: PermitRefusal) -> bool {
        self.sup
            .events()
            .iter()
            .any(|e| e.event == Event::SupervisorRefused(r))
    }
}

#[test]
fn supervisor_does_not_hold_before_hearing_main() {
    let mut s = SupBench::new();
    s.run_ms(1_000);
    assert!(!s.board.sup_hold);
    assert!(!s.board.permit);
    assert_eq!(
        s.sup.fault(),
        None,
        "silence before first contact is not a fault"
    );
}

#[test]
fn supervisor_holds_only_while_main_claims_charging_or_armed() {
    let mut s = SupBench::new();
    for (state, expect) in [
        (S::AttachPads, false),
        (S::Analysing, false),
        (S::SelfTest, true),
        (S::AnalysingCharging, true),
        (S::Charging, true),
        (S::Countdown, true),
        (S::Delivering, true),
        (S::Cpr, false),
        (S::Fault, false),
    ] {
        s.claim(state);
        s.run_ms(100);
        assert_eq!(s.board.sup_hold, expect, "{state:?}");
    }
}

#[test]
fn supervisor_permits_when_every_condition_holds() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(s.board.permit);
    assert!(s
        .sup
        .events()
        .iter()
        .any(|e| e.event == Event::SupervisorPermitted));
}

#[test]
fn supervisor_refuses_permit_when_voltage_reading_disagrees() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    // Main claims a voltage 200 V away from what the supervisor reads.
    s.claim_v = Some(1_600);
    s.claim(S::Delivering);
    s.step();
    assert!(!s.board.permit);
    assert!(s.refused(PermitRefusal::VoltageDisagreesWithMain));
}

#[test]
fn supervisor_refuses_permit_when_voltage_not_at_plan() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.planned = 1_500; // main claims it planned 1,500 V but the capacitor is at 1,800 V
    s.claim(S::Delivering);
    s.step();
    assert!(!s.board.permit);
    assert!(s.refused(PermitRefusal::VoltageNotAtPlan));
}

#[test]
fn supervisor_refuses_permit_before_full_countdown_on_its_own_clock() {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.claim(S::Charging);
    s.run_ms(9_000);
    s.claim(S::Countdown);
    s.run_ms(s.config.countdown_ms - 500);
    s.claim(S::Delivering);
    s.step();
    assert!(!s.board.permit);
    assert!(s.refused(PermitRefusal::CountdownTooShort));
    // Once the full countdown has passed on its own clock, it permits.
    s.run_ms(600);
    assert!(s
        .sup
        .events()
        .iter()
        .any(|e| e.event == Event::SupervisorPermitted));
}

#[test]
fn supervisor_refuses_permit_when_countdown_was_skipped() {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.claim(S::Charging);
    s.run_ms(9_000);
    s.claim(S::Delivering); // straight from charging, no countdown
    s.run_ms(10_000);
    assert!(s
        .sup
        .events()
        .iter()
        .all(|e| e.event != Event::SupervisorPermitted));
    assert!(s.refused(PermitRefusal::CountdownTooShort));
}

#[test]
fn supervisor_refuses_permit_when_own_pad_reading_out_of_range() {
    let mut s = SupBench::new();
    s.board.sup_pad_ohms = Some(400);
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(!s.board.permit);
    assert!(s.refused(PermitRefusal::PadsOutOfRange));
}

#[test]
fn supervisor_refuses_permit_when_cancel_pressed() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.board.cancel = true;
    s.claim(S::Delivering);
    s.step();
    assert!(!s.board.permit);
    assert!(!s.board.sup_hold);
}

#[test]
fn supervisor_refuses_permit_when_heartbeat_stale() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.claim = None; // heartbeats stop after the Delivering claim
    s.board.status_inbox = None;
    // Within the timeout a permit was possible; check that after the timeout
    // there is none and the capacitor is released.
    s.run_ms(s.config.heartbeat_timeout_ms + 10);
    assert!(!s.board.permit);
    assert!(!s.board.sup_hold);
}

#[test]
fn supervisor_permit_lasts_only_a_short_window() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(s.board.permit);
    s.run_ms(s.config.permit_window_ms + 4);
    assert!(!s.board.permit);
}

#[test]
fn permit_is_single_use() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(s.board.permit);
    s.run_ms(s.config.permit_window_ms + 4);
    assert!(!s.board.permit);
    // Main keeps claiming Delivering (no delivery happened): no second permit.
    s.run_ms(2_000);
    assert!(!s.board.permit);
    assert!(s.refused(PermitRefusal::AlreadyUsed));
    let permits = s
        .sup
        .events()
        .iter()
        .filter(|e| e.event == Event::SupervisorPermitted)
        .count();
    assert_eq!(permits, 1);
}

#[test]
fn permit_is_available_again_after_a_new_arming() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(s.board.permit);
    // Main disarms, goes to CPR, then arms again.
    s.board.main_hold = false;
    s.board.charger = None;
    s.claim(S::Cpr);
    s.run_ms(1_000);
    s.claim(S::Cpr);
    s.run_ms(1_000);
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(s.board.permit);
}

#[test]
fn supervisor_empties_capacitor_on_heartbeat_loss() {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.claim(S::Charging);
    s.run_ms(9_000);
    assert!(s.board.cap_v > 1_700.0);
    s.claim = None; // main stops sending; its hold stays asserted
    s.run_ms(s.config.heartbeat_timeout_ms + 400);
    assert!(!s.board.sup_hold);
    assert!(s.board.cap_v < 50.0, "{} V", s.board.cap_v);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::HeartbeatLost));
}

#[test]
fn stuck_main_in_armed_state_is_emptied_by_supervisor() {
    // The main controller is alive (heartbeats keep coming) but stuck
    // claiming Countdown forever, with the capacitor held charged.
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.claim(S::Charging);
    s.run_ms(9_000);
    s.claim(S::Countdown);
    s.run_ms(s.config.max_charged_ms);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::ChargedTooLong));
    assert!(!s.board.sup_hold);
    s.run_ms(1_000);
    assert!(s.board.cap_v < 50.0, "{} V", s.board.cap_v);
    assert!(s.sup.events().iter().any(|e| e.event
        == Event::SupervisorReleasedHold(ReleaseReason::Fault(SupervisorFault::ChargedTooLong))));
}

#[test]
fn supervisor_empties_capacitor_on_overvoltage() {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.board.charger_runaway = true;
    s.claim_v = None;
    s.claim(S::Charging);
    s.run_ms(12_000);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::Overvoltage));
    assert!(!s.board.sup_hold);
    assert!(s.board.cap_v < 2_200.0);
}

#[test]
fn supervisor_empties_capacitor_on_disagreement_that_persists() {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.claim(S::Charging);
    s.run_ms(9_000);
    s.claim(S::Countdown);
    s.claim_v = Some(900); // main now claims half the real voltage
    s.run_ms(s.config.disagreement_persist_ms - 100);
    assert!(s.board.sup_hold, "short disagreement is tolerated");
    s.run_ms(300);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::VoltageDisagreement));
    assert!(!s.board.sup_hold);
}

#[test]
fn supervisor_empties_capacitor_on_cancel() {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.claim(S::Charging);
    s.run_ms(9_000);
    s.claim(S::Countdown);
    s.run_ms(1_000);
    s.board.cancel = true;
    s.run_ms(500);
    assert!(!s.board.sup_hold);
    assert!(s.board.cap_v < 50.0);
    assert_eq!(s.sup.fault(), None, "cancel is not a fault");
}

#[test]
fn supervisor_faults_are_latched() {
    let mut s = SupBench::new();
    s.claim(S::Cpr);
    s.run_ms(200);
    s.claim = None;
    s.run_ms(500);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::HeartbeatLost));
    // Main comes back and tries a normal arming: still no hold, no permit.
    s.claim = Some(S::Cpr);
    s.run_ms(200);
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    s.claim(S::Charging);
    s.run_ms(9_000);
    s.claim(S::Countdown);
    s.run_ms(6_000);
    s.claim(S::Delivering);
    s.run_ms(500);
    assert!(!s.board.sup_hold);
    assert!(!s.board.permit);
    assert!(s.board.cap_v < 50.0);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::HeartbeatLost));
}

#[test]
fn supervisor_reports_latched_fault_back_to_main() {
    let mut s = SupBench::new();
    s.claim(S::Cpr);
    s.run_ms(200);
    s.claim = None;
    s.run_ms(500);
    let report = s.board.report_inbox.unwrap();
    assert_eq!(report.fault, Some(SupervisorFault::HeartbeatLost));
    assert!(!report.holding);
    assert!(!report.permit);
}

#[test]
fn supervisor_alone_cannot_fire() {
    // The supervisor is fooled into giving a permit by fake heartbeats, and
    // the main side's hold is on, but the main controller never raises its
    // request: the bridge must not fire.
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(s.board.permit);
    s.run_ms(500);
    assert!(s.board.fires.is_empty());
    assert!(s.board.request.is_none());
}

#[test]
fn permit_with_request_fires_only_while_both_hold() {
    // With request and permit both present, releasing either hold blocks the shock.
    for (main_hold, sup_hold) in [(true, false), (false, true), (false, false)] {
        assert!(!bridge_may_fire(
            true,
            true,
            crate::interlock::discharge_path_active(main_hold, sup_hold)
        ));
    }
}

// ----- the supervisor's own fault alarm (finding F-3) ----------------------------

/// Drive the supervisor into `kind`, the way a real failure would.
fn provoke(kind: SupervisorFault) -> SupBench {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(PLANNED));
    match kind {
        SupervisorFault::HeartbeatLost => {
            s.claim(S::Charging);
            s.run_ms(2_000);
            s.claim = None;
            s.run_ms(s.config.heartbeat_timeout_ms + 100);
        }
        SupervisorFault::ChargedTooLong => {
            s.claim(S::Charging);
            s.run_ms(9_000);
            s.claim(S::Countdown);
            s.run_ms(s.config.max_charged_ms);
        }
        SupervisorFault::Overvoltage => {
            s.board.charger_runaway = true;
            s.claim(S::Charging);
            s.run_ms(12_000);
        }
        SupervisorFault::VoltageDisagreement => {
            s.claim(S::Charging);
            s.run_ms(9_000);
            s.claim_v = Some(900);
            s.claim(S::Countdown);
            s.run_ms(s.config.disagreement_persist_ms + 200);
        }
        SupervisorFault::CancelStuck => {
            s.claim(S::Cpr);
            s.board.cancel = true;
            s.run_ms(s.config.cancel_stuck_ms + 100);
        }
    }
    s
}

#[test]
fn supervisor_alarm_is_on_for_every_supervisor_fault_kind() {
    for kind in [
        SupervisorFault::HeartbeatLost,
        SupervisorFault::ChargedTooLong,
        SupervisorFault::Overvoltage,
        SupervisorFault::VoltageDisagreement,
        SupervisorFault::CancelStuck,
    ] {
        let mut s = provoke(kind);
        assert_eq!(s.sup.fault(), Some(kind));
        assert!(s.board.sup_alarm, "{kind:?}: alarm off");
        assert!(s.sup.alarm_active());
        // Latched: it stays on.
        s.run_ms(5_000);
        assert!(s.board.sup_alarm, "{kind:?}: alarm went off again");
    }
}

#[test]
fn supervisor_alarm_stays_off_through_a_normal_arming() {
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.run_ms(100);
    s.claim(S::Cpr);
    s.run_ms(1_000);
    assert!(!s.board.sup_alarm);
    assert_eq!(s.sup.fault(), None);
}

#[test]
fn supervisor_sounds_its_alarm_if_the_main_controller_never_starts() {
    let mut s = SupBench::new();
    s.run_ms(s.config.self_test_link_wait_ms - 100);
    assert!(!s.board.sup_alarm, "too early");
    s.run_ms(200);
    assert!(s.board.sup_alarm, "main controller silent since switch-on: alarm must sound");
    // Not a latched fault: it never held anything (see
    // supervisor_does_not_hold_before_hearing_main).
    assert_eq!(s.sup.fault(), None);
    assert!(!s.board.sup_hold);
}

// ----- charging during analysis (D-29) ---------------------------------------------

/// Fake main: hold on and charge while claiming "analysing and charging".
fn precharge(s: &mut SupBench, ms: u32) {
    s.board.main_hold = true;
    s.board.charger = Some(Volts(s.planned));
    s.claim(S::AnalysingCharging);
    s.run_ms(ms);
}

#[test]
fn supervisor_holds_and_accepts_a_charged_capacitor_during_analysis_without_a_permit() {
    let mut s = SupBench::new();
    precharge(&mut s, 12_000);
    assert!(s.board.cap_v > 1_750.0);
    assert!(s.board.sup_hold, "holds while analysing and charging");
    assert!(!s.board.permit);
    assert_eq!(s.sup.fault(), None, "a charged capacitor during analysis is not a fault");
    assert!(s
        .sup
        .events()
        .iter()
        .all(|e| e.event != Event::SupervisorPermitted));
}

#[test]
fn supervisor_refuses_permit_straight_from_analysing_and_charging() {
    // A main controller that skips the countdown after charging during
    // analysis gets no permit.
    let mut s = SupBench::new();
    precharge(&mut s, 9_000);
    s.claim(S::Delivering);
    s.run_ms(3_000);
    assert!(s.refused(PermitRefusal::CountdownTooShort));
    assert!(s
        .sup
        .events()
        .iter()
        .all(|e| e.event != Event::SupervisorPermitted));
}

#[test]
fn supervisor_permits_after_charging_during_analysis_only_after_the_full_countdown() {
    let mut s = SupBench::new();
    precharge(&mut s, 9_000);
    s.claim(S::Countdown);
    s.run_ms(s.config.countdown_ms - 500);
    s.claim(S::Delivering);
    s.step();
    assert!(!s.board.permit);
    assert!(s.refused(PermitRefusal::CountdownTooShort));
    s.run_ms(600);
    assert!(s
        .sup
        .events()
        .iter()
        .any(|e| e.event == Event::SupervisorPermitted));
}

#[test]
fn supervisor_charged_too_long_limit_still_applies_while_analysing_and_charging() {
    let mut s = SupBench::new();
    let ms = s.config.max_charged_ms + 500;
    precharge(&mut s, ms);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::ChargedTooLong));
    assert!(!s.board.sup_hold);
}

// ----- the shock range (D-32) ------------------------------------------------------

#[test]
fn supervisor_refuses_permit_above_the_shock_range_even_inside_the_sensing_range() {
    for ohms in [176, 200, 250] {
        let mut s = SupBench::new();
        s.board.sup_pad_ohms = Some(ohms);
        s.arm_and_count_down();
        s.claim(S::Delivering);
        s.step();
        assert!(!s.board.permit, "{ohms} Ω");
        assert!(s.refused(PermitRefusal::PadsOutOfRange), "{ohms} Ω");
    }
}

// ----- cancel (D-31) ---------------------------------------------------------------

#[test]
fn supervisor_ignores_cancel_while_main_is_in_its_self_test() {
    let mut s = SupBench::new();
    s.board.main_hold = true;
    s.board.charger = Some(Volts(400));
    s.claim(S::SelfTest);
    s.run_ms(100);
    s.board.cancel = true;
    s.run_ms(500);
    assert!(s.board.sup_hold, "a press during the self-test must not stop it");
    assert!(s.board.cap_v > 350.0);
    assert_eq!(s.sup.fault(), None);
    // Outside the self-test, cancel lets go at once.
    s.claim(S::AnalysingCharging);
    s.run_ms(10);
    assert!(!s.board.sup_hold);
}

#[test]
fn supervisor_latches_a_stuck_cancel_and_sounds_its_alarm() {
    let mut s = SupBench::new();
    s.claim(S::Cpr);
    s.board.cancel = true;
    s.run_ms(s.config.cancel_stuck_ms - 100);
    assert_eq!(s.sup.fault(), None, "not yet stuck");
    s.run_ms(200);
    assert_eq!(s.sup.fault(), Some(SupervisorFault::CancelStuck));
    assert!(s.board.sup_alarm);
}

// ----- finding F-5 -----------------------------------------------------------------

#[test]
fn no_already_used_refusal_is_recorded_for_a_delayed_delivering_echo() {
    // After a good shock the main controller's "delivering" can still arrive
    // in delayed status messages for a while. That is not a second request.
    let mut s = SupBench::new();
    s.arm_and_count_down();
    s.claim(S::Delivering);
    s.step();
    assert!(s.board.permit);
    // The shock: most of the charge leaves; main lets go.
    s.board.cap_v *= 0.2;
    s.board.main_hold = false;
    s.board.charger = None;
    // The delayed "delivering" keeps arriving for 150 ms, then CPR.
    s.run_ms(150);
    s.claim(S::Cpr);
    s.run_ms(500);
    assert!(
        !s.refused(PermitRefusal::AlreadyUsed),
        "spurious 'already used' refusal recorded"
    );
}
