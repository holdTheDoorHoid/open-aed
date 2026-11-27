// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests of the smaller parts: interlock, self-test sequencer, event record,
//! configuration and prompts.

use super::mock::{Board, MainView};
use crate::config::{Config, ConfigError};
use crate::events::{Event, EventRecord};
use crate::hal::{BatteryStatus, PadCartridge};
use crate::interlock::{bridge_may_fire, discharge_path_active};
use crate::link::{SupervisorFault, SupervisorReport};
use crate::prompts::Prompt;
use crate::selftest::{LinkSnapshot, SelfTest, SelfTestFailure, SelfTestPass, SelfTestProgress};
use aed_types::{Millis, Volts};
use std::cell::Cell;
use std::rc::Rc;
use std::vec::Vec;

// ----- interlock -------------------------------------------------------------------

#[test]
fn interlock_fires_only_with_request_and_permit_and_discharge_off() {
    for request in [false, true] {
        for permit in [false, true] {
            for discharging in [false, true] {
                let fires = bridge_may_fire(request, permit, discharging);
                assert_eq!(
                    fires,
                    request && permit && !discharging,
                    "request {request} permit {permit} discharging {discharging}"
                );
            }
        }
    }
}

#[test]
fn interlock_never_passes_without_both_signals() {
    for discharging in [false, true] {
        assert!(!bridge_may_fire(true, false, discharging));
        assert!(!bridge_may_fire(false, true, discharging));
        assert!(!bridge_may_fire(false, false, discharging));
    }
}

#[test]
fn discharge_path_is_active_unless_both_controllers_hold() {
    assert!(discharge_path_active(false, false));
    assert!(discharge_path_active(true, false));
    assert!(discharge_path_active(false, true));
    assert!(!discharge_path_active(true, true));
}

// ----- self-test sequencer ---------------------------------------------------------

/// How the fake supervisor behaves during a stand-alone self-test.
#[derive(Clone, Copy)]
struct FakeLink {
    alive: bool,
    offset: i32,
    fault: Option<SupervisorFault>,
}

const GOOD_LINK: FakeLink = FakeLink {
    alive: true,
    offset: 0,
    fault: None,
};

/// Run a self-test on a bare board, with the supervisor side faked.
/// Returns the result and how long it took.
fn run_self_test(
    board: &mut Board,
    test_voltage: Volts,
    link: FakeLink,
) -> (Result<SelfTestPass, SelfTestFailure>, u32, f32) {
    let mut st = SelfTest::new(Config::default(), test_voltage);
    let start = board.now;
    let mut peak = 0.0f32;
    for seq in 0..100_000u32 {
        board.now += 2;
        board.sup_hold = true; // the supervisor holds while main claims SelfTest
        board.step(2);
        peak = peak.max(board.cap_v);
        let snapshot = link.alive.then(|| LinkSnapshot {
            report: SupervisorReport {
                sequence: seq,
                fault: link.fault,
                permit: false,
                holding: true,
                capacitor_voltage: Volts(
                    u16::try_from((board.cap_v.round() as i32 + link.offset).max(0)).unwrap(),
                ),
            },
            received_at: Millis(board.now),
        });
        if let SelfTestProgress::Finished(r) = st.step(&mut MainView(board), snapshot) {
            return (r, board.now - start, peak);
        }
    }
    panic!("self-test never finished");
}

fn board() -> Board {
    Board::new(Rc::new(Cell::new(None)))
}

#[test]
fn self_test_passes_on_good_hardware_and_leaves_capacitor_released() {
    let mut b = board();
    let (r, took, peak) = run_self_test(&mut b, Volts(400), GOOD_LINK);
    assert_eq!(r, Ok(SelfTestPass { battery_low: false }));
    assert!((350.0..=410.0).contains(&peak), "peak {peak} V");
    assert!(took < 5_000);
    assert!(!b.main_hold);
    assert!(b.charger.is_none());
    assert!(b.cap_v < 50.0);
}

#[test]
fn self_test_can_charge_to_full_energy_voltage() {
    // The periodic full-energy self-test of decision D-15.
    let mut b = board();
    let (r, _, peak) = run_self_test(&mut b, Volts(1800), GOOD_LINK);
    assert!(r.is_ok());
    assert!(peak > 1_740.0);
}

#[test]
fn self_test_reports_low_battery_as_a_warning() {
    let mut b = board();
    b.battery_under_load = BatteryStatus::Low;
    let (r, _, _) = run_self_test(&mut b, Volts(400), GOOD_LINK);
    assert_eq!(r, Ok(SelfTestPass { battery_low: true }));
}

type SelfTestCase = (&'static str, fn(&mut Board), FakeLink, SelfTestFailure);

#[test]
fn self_test_failure_codes_are_distinct_per_check() {
    let cases: Vec<SelfTestCase> = std::vec![
        (
            "battery",
            |b| b.battery_under_load = BatteryStatus::Depleted,
            GOOD_LINK,
            SelfTestFailure::BatteryUnderLoad
        ),
        (
            "link",
            |_| {},
            FakeLink {
                alive: false,
                ..GOOD_LINK
            },
            SelfTestFailure::SupervisorLinkDead
        ),
        (
            "sup fault",
            |_| {},
            FakeLink {
                fault: Some(SupervisorFault::Overvoltage),
                ..GOOD_LINK
            },
            SelfTestFailure::SupervisorFaultLatched
        ),
        (
            "charger",
            |b| b.charger_broken = true,
            GOOD_LINK,
            SelfTestFailure::ChargeTimeout
        ),
        (
            "disagree",
            |_| {},
            FakeLink {
                offset: 200,
                ..GOOD_LINK
            },
            SelfTestFailure::VoltageReadingsDisagree
        ),
        (
            "overvoltage",
            |b| {
                b.charger_runaway = true;
                b.charge_rate = 10.0;
            },
            FakeLink {
                offset: 500,
                ..GOOD_LINK
            },
            SelfTestFailure::Overvoltage
        ),
        (
            "discharge",
            |b| b.dump_broken = true,
            GOOD_LINK,
            SelfTestFailure::DischargeTimeout
        ),
        (
            "pads missing",
            |b| b.cartridge = PadCartridge::Missing,
            GOOD_LINK,
            SelfTestFailure::PadsMissing
        ),
        (
            "pads expired",
            |b| b.cartridge = PadCartridge::Expired,
            GOOD_LINK,
            SelfTestFailure::PadsExpired
        ),
    ];
    let mut seen = Vec::new();
    for (name, setup, link, expect) in cases {
        let mut b = board();
        setup(&mut b);
        let (r, _, _) = run_self_test(&mut b, Volts(400), link);
        assert_eq!(r, Err(expect), "{name}");
        assert!(!b.main_hold, "{name}: hold left on");
        assert!(b.charger.is_none(), "{name}: charger left on");
        assert!(!seen.contains(&expect));
        seen.push(expect);
    }
}

#[test]
fn self_test_checks_have_time_limits() {
    let c = Config::default();
    let mut b = board();
    b.charger_broken = true;
    let (_, took, _) = run_self_test(&mut b, Volts(400), GOOD_LINK);
    assert!(took <= c.charge_time_limit_ms + 10, "charge step {took} ms");
    let mut b = board();
    let (_, took, _) = run_self_test(
        &mut b,
        Volts(400),
        FakeLink {
            alive: false,
            ..GOOD_LINK
        },
    );
    assert!(took <= c.self_test_link_wait_ms + 10, "link step {took} ms");
}

#[test]
fn only_pad_problems_are_advisory() {
    use SelfTestFailure as F;
    for f in [
        F::BatteryUnderLoad,
        F::SupervisorLinkDead,
        F::SupervisorFaultLatched,
        F::ChargeTimeout,
        F::Overvoltage,
        F::VoltageReadingsDisagree,
        F::DischargeTimeout,
        F::Cancelled,
    ] {
        assert!(!f.is_advisory(), "{f:?}");
    }
    assert!(F::PadsMissing.is_advisory());
    assert!(F::PadsExpired.is_advisory());
}

#[test]
fn self_test_abort_releases_capacitor() {
    let mut b = board();
    let mut st = SelfTest::new(Config::default(), Volts(400));
    b.main_hold = true;
    b.charger = Some(Volts(400));
    st.abort(&mut MainView(&mut b));
    assert!(!b.main_hold);
    assert!(b.charger.is_none());
    b.now += 2;
    assert_eq!(
        st.step(&mut MainView(&mut b), None),
        SelfTestProgress::Finished(Err(SelfTestFailure::Cancelled))
    );
}

// ----- event record ----------------------------------------------------------------

#[test]
fn event_record_keeps_order_oldest_first() {
    let mut r: EventRecord<4> = EventRecord::new();
    assert!(r.is_empty());
    for t in 0..3 {
        r.push(Millis(t), Event::PowerOn);
    }
    let times: Vec<u32> = r.iter().map(|e| e.at.0).collect();
    assert_eq!(times, std::vec![0, 1, 2]);
    assert_eq!(r.overwritten(), 0);
}

#[test]
fn event_record_overwrites_oldest_and_counts_overflow() {
    let mut r: EventRecord<4> = EventRecord::new();
    for t in 0..10 {
        r.push(Millis(t), Event::CancelPressed);
    }
    let times: Vec<u32> = r.iter().map(|e| e.at.0).collect();
    assert_eq!(times, std::vec![6, 7, 8, 9]);
    assert_eq!(r.len(), 4);
    assert_eq!(r.overwritten(), 6);
}

#[test]
fn event_record_of_zero_capacity_does_not_crash() {
    let mut r: EventRecord<0> = EventRecord::new();
    r.push(Millis(1), Event::PowerOn);
    assert_eq!(r.iter().count(), 0);
    assert_eq!(r.overwritten(), 1);
}

// ----- configuration ---------------------------------------------------------------

#[test]
fn default_config_is_valid_and_matches_documented_values() {
    let c = Config::default();
    assert_eq!(c.validate(), Ok(()));
    assert_eq!(c.cpr_period_ms, 120_000);
    assert_eq!(c.countdown_ms, 5_000);
    assert_eq!(c.charge_time_limit_ms, 12_000);
    assert_eq!(c.empty_voltage, Volts(50));
    assert_eq!(c.metronome_interval_ms(), 545);
}

#[test]
fn contradictory_config_is_refused() {
    let base = Config::default();
    let cases = [
        (
            Config {
                metronome_per_min: 90,
                ..base
            },
            ConfigError::MetronomeRateOutOfRange,
        ),
        (
            Config {
                metronome_per_min: 130,
                ..base
            },
            ConfigError::MetronomeRateOutOfRange,
        ),
        (
            Config {
                countdown_ms: 0,
                ..base
            },
            ConfigError::ZeroTimeLimit,
        ),
        (
            Config {
                max_charged_ms: 15_000,
                ..base
            },
            ConfigError::MaxChargedTooShort,
        ),
        (
            Config {
                heartbeat_timeout_ms: 50,
                ..base
            },
            ConfigError::HeartbeatTimeoutTooShort,
        ),
        (
            Config {
                self_test_voltage: Volts(80),
                ..base
            },
            ConfigError::SelfTestVoltageTooLow,
        ),
        (
            Config {
                self_test_voltage: Volts(2_300),
                ..base
            },
            ConfigError::SelfTestVoltageTooHigh,
        ),
        (
            Config {
                cpr_period_ms: 10_000,
                ..base
            },
            ConfigError::CprPeriodTooShort,
        ),
    ];
    for (c, e) in cases {
        assert_eq!(c.validate(), Err(e));
        assert!(crate::Supervisor::new(c).is_err());
    }
}

// ----- prompts ---------------------------------------------------------------------

#[test]
fn every_prompt_has_short_plain_text() {
    let all = [
        Prompt::AttachPads,
        Prompt::ChildModeFrontBack,
        Prompt::AdultMode,
        Prompt::CheckPads,
        Prompt::ReplacePads,
        Prompt::AnalysingDoNotTouch,
        Prompt::StopCprAnalysing,
        Prompt::ShockAdvisedCharging,
        Prompt::StandClear,
        Prompt::ShockDelivered,
        Prompt::ShockNotDelivered,
        Prompt::NoShockAdvised,
        Prompt::StartCpr,
        Prompt::ShockCancelled,
        Prompt::AnalysisCancelled,
        Prompt::RhythmChangedNoShock,
        Prompt::DeviceFaultContinueCpr,
        Prompt::SeriousFaultContinueCpr,
        Prompt::LowBattery,
    ];
    for p in all {
        let t = p.text();
        assert!(!t.is_empty() && t.len() <= 70, "{p:?}: {t:?}");
    }
    for n in 1..=5 {
        assert!(!Prompt::Countdown(n).text().is_empty());
    }
    assert_eq!(Prompt::Countdown(3).text(), "Three.");
}

#[test]
fn fault_prompts_tell_the_responder_to_continue_cpr() {
    assert!(Prompt::DeviceFaultContinueCpr
        .text()
        .contains("Continue CPR"));
    assert!(Prompt::SeriousFaultContinueCpr
        .text()
        .contains("Continue CPR"));
}
