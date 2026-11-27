// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests of the smaller parts: interlock, self-test sequencer, event record,
//! configuration and prompts.

use super::mock::{Board, MainView};
use crate::config::{Config, ConfigError};
use crate::events::{Event, EventRecord};
use crate::hal::{BatteryStatus, PadCartridge};
use crate::interlock::{bridge_may_fire, charger_may_run, discharge_path_active};
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

#[test]
fn charger_cannot_run_while_discharge_path_is_active() {
    // Truth table of the hardware rule, through every combination of the
    // two controllers' holds: the charger runs only when both hold and the
    // main controller asks for it, so either controller letting go stops it.
    for main_hold in [false, true] {
        for supervisor_hold in [false, true] {
            for enabled in [false, true] {
                let discharging = discharge_path_active(main_hold, supervisor_hold);
                assert_eq!(
                    charger_may_run(enabled, discharging),
                    enabled && main_hold && supervisor_hold,
                    "main {main_hold} supervisor {supervisor_hold} enabled {enabled}"
                );
            }
        }
    }
    assert!(!charger_may_run(true, true));
    assert!(!charger_may_run(false, false));
    assert!(charger_may_run(true, false));
}

// ----- self-test sequencer ---------------------------------------------------------

/// How the fake supervisor behaves during a stand-alone self-test.
#[derive(Clone, Copy)]
struct FakeLink {
    alive: bool,
    holding: bool,
    offset: i32,
    fault: Option<SupervisorFault>,
}

const GOOD_LINK: FakeLink = FakeLink {
    alive: true,
    holding: true,
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
        board.sup_hold = link.holding; // the supervisor holds while main claims SelfTest
        board.step(2);
        peak = peak.max(board.cap_v);
        let snapshot = link.alive.then(|| LinkSnapshot {
            report: SupervisorReport {
                sequence: seq,
                fault: link.fault,
                permit: false,
                holding: link.holding,
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
            "supervisor not holding",
            |_| {},
            FakeLink {
                holding: false,
                ..GOOD_LINK
            },
            SelfTestFailure::SupervisorNotHolding
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
    assert!(
        took <= c.charge_time_limit_for(Volts(400)) + 10,
        "charge step {took} ms"
    );
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
fn self_test_charge_step_limit_scales_with_the_test_voltage() {
    // Finding F-7: the charge step's limit follows the stored energy (the
    // square of the voltage), with a 1 s floor.
    let c = Config::default();
    assert_eq!(c.charge_time_limit_for(Volts(400)), 1_000);
    assert_eq!(c.charge_time_limit_for(Volts(0)), 1_000);
    assert_eq!(c.charge_time_limit_for(Volts(1_950)), 11_407);
    assert_eq!(c.charge_time_limit_for(Volts(2_000)), 12_000);
    assert_eq!(c.charge_time_limit_for(Volts(3_000)), 12_000, "never above the full limit");
    assert_eq!(SelfTest::new(c, Volts(400)).charge_time_limit_ms(), 1_000);
}

#[test]
fn weak_charger_is_reported_about_a_second_into_the_self_test() {
    let mut b = board();
    b.charger_broken = true;
    let (r, took, _) = run_self_test(&mut b, Volts(400), GOOD_LINK);
    assert_eq!(r, Err(SelfTestFailure::ChargeTimeout));
    assert!(took <= 1_100, "reported after {took} ms");
    // A charger that only gets part way is caught the same way.
    let mut b = board();
    b.charge_rate = 0.02; // about a tenth of the normal speed
    let (r, took, _) = run_self_test(&mut b, Volts(400), GOOD_LINK);
    assert_eq!(r, Err(SelfTestFailure::ChargeTimeout));
    assert!(took <= 1_100, "reported after {took} ms");
}

#[test]
fn only_pad_problems_are_advisory() {
    use SelfTestFailure as F;
    for f in [
        F::BatteryUnderLoad,
        F::SupervisorLinkDead,
        F::SupervisorFaultLatched,
        F::SupervisorNotHolding,
        F::ChargeTimeout,
        F::Overvoltage,
        F::VoltageReadingsDisagree,
        F::DischargeTimeout,
        F::Aborted,
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
        SelfTestProgress::Finished(Err(SelfTestFailure::Aborted))
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
    // D-30: "Stand clear. Three, two, one."
    assert_eq!(c.countdown_ms, 3_000);
    // D-29: charge during analysis.
    assert!(c.charge_during_analysis);
    assert_eq!(c.max_charged_ms, 25_000);
    // D-31: a cancel held for 10 s is stuck.
    assert_eq!(c.cancel_stuck_ms, 10_000);
    assert_eq!(c.press_pads_repeat_ms, 15_000);
    assert_eq!(c.charge_time_limit_ms, 12_000);
    assert_eq!(c.empty_voltage, Volts(50));
    assert_eq!(c.metronome_interval_ms(), 545);
    assert_eq!(c.analysis_time_limit_ms, 12_000);
    assert_eq!(c.hands_off_reminder_ms, 3_000);
    // D-28: sensors accurate to 1 % + 7 V; allowances 1 % + 12 V.
    assert_eq!((c.at_plan_percent, c.at_plan_abs_volts), (1, Volts(12)));
    assert_eq!((c.agree_percent, c.agree_abs_volts), (1, Volts(12)));
}

/// A reading from a sensor with a gain error (percent) and an offset (volts).
fn sensor(true_v: u16, percent: i32, offset: i32) -> Volts {
    let v = i32::from(true_v) * (100 + percent) / 100 + offset;
    Volts(u16::try_from(v.max(0)).unwrap())
}

#[test]
fn two_sensors_with_opposite_errors_inside_their_allowance_agree_and_read_at_plan() {
    // The accuracy the defaults assume (D-28): each sensor ±1 % ± 7 V. At
    // the adult and child charge voltages and the self-test voltage, two
    // sensors with opposite errors must agree and each must read "at plan".
    let c = Config::default();
    for v in [1_950, 1_150, 400] {
        let high = sensor(v, 1, 7);
        let low = sensor(v, -1, -7);
        assert!(c.readings_agree(high, low), "{v} V: {high:?} vs {low:?}");
        assert!(c.readings_agree(low, high));
        for r in [high, low] {
            assert!(c.is_at_plan(r, Volts(v)), "{v} V: {r:?} not at plan");
            assert!(c.reached_plan(r, Volts(v)));
        }
    }
}

#[test]
fn a_sensor_five_or_eight_percent_off_disagrees_at_the_charge_voltages() {
    // Outside the D-28 budget (two sensors may differ by 2 % + 24 V): one
    // sensor 5 % (or 8 %) off, the other perfect.
    let c = Config::default();
    for v in [1_950, 1_150] {
        for off in [5, -5, 8, -8] {
            assert!(
                !c.readings_agree(sensor(v, off, 0), Volts(v)),
                "{v} V, one sensor {off} %"
            );
        }
    }
    // A dead (or stuck-at-zero) sensor disagrees even at the self-test voltage.
    assert!(!c.readings_agree(Volts(0), Volts(400)));
}

#[test]
fn empty_is_only_reported_when_the_true_voltage_is_below_the_empty_voltage() {
    // A sensor reading low by its whole allowance must still not report
    // "empty" for a capacitor at or above 50 V.
    let c = Config::default();
    for true_v in 50..200 {
        let low = sensor(true_v, -1, -12);
        assert!(!c.reads_empty(low), "{true_v} V true reads {low:?} and counted as empty");
    }
    assert!(c.reads_empty(Volts(0)));
    assert!(c.reads_empty(Volts(30)));
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
            // 12 s (analysis or charge) + 3 s countdown + 1 s delivery + 3 s
            // emptying = 19 s: a limit of exactly that would cut off a
            // normal pre-charged shock cycle.
            Config {
                max_charged_ms: 19_000,
                ..base
            },
            ConfigError::MaxChargedTooShort,
        ),
        (
            Config {
                cancel_stuck_ms: 0,
                ..base
            },
            ConfigError::ZeroTimeLimit,
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
                self_test_voltage: Volts(70),
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
        (
            Config {
                hands_off_reminder_ms: 0,
                ..base
            },
            ConfigError::ZeroTimeLimit,
        ),
        (
            Config {
                agree_percent: 0,
                agree_abs_volts: Volts(0),
                ..base
            },
            ConfigError::SensorAllowanceZero,
        ),
        (
            // 50 % per sensor: a dead sensor would "agree" with anything.
            Config {
                agree_percent: 50,
                at_plan_percent: 50,
                ..base
            },
            ConfigError::SensorAllowanceTooWide,
        ),
        (
            // 60 V per sensor: nothing could ever read as empty (< 50 V).
            Config {
                agree_abs_volts: Volts(60),
                at_plan_abs_volts: Volts(60),
                ..base
            },
            ConfigError::SensorAllowanceTooWide,
        ),
        (
            Config {
                at_plan_percent: 0,
                ..base
            },
            ConfigError::AtPlanNarrowerThanSensorError,
        ),
        (
            Config {
                at_plan_abs_volts: Volts(10),
                ..base
            },
            ConfigError::AtPlanNarrowerThanSensorError,
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
        Prompt::DoNotTouchPatient,
        Prompt::AnalysisInconclusive,
        Prompt::ShockAdvised,
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
        Prompt::PressPadsFirmly,
        Prompt::PadContactPoorNoShock,
        Prompt::CancelStuckContinueCpr,
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
    assert!(Prompt::CancelStuckContinueCpr
        .text()
        .contains("Continue CPR"));
}
