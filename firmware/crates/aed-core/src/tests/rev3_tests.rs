// SPDX-License-Identifier: GPL-3.0-or-later
//! Unit tests of the design revision 0.3 firmware rules that do not need the
//! time-resolved plant: the delivery sequencer's waits (FW-1, FW-2, FW-3,
//! FW-7), the per-phase caps in the sequence logic (FW-9, D-63) and the
//! configuration (FW-4 … FW-10). The plant-level tests are in
//! `aed-sim/tests/rev3.rs`.

use crate::config::{Config, ConfigError};
use crate::hal::{SequenceEnd, SequenceStatus};
use crate::interlock::{Latch, LatchInputs, LatchTiming};
use crate::sequencer::{Sequencer, SequencerJob, SequencerLines, SequencerState, SequencerTiming};
use aed_types::{Decijoules, Micros, ShockPlan, Volts};

const DT: u32 = 10;

fn plan() -> ShockPlan {
    ShockPlan {
        charge_voltage: Volts(1_950),
        phase1: Micros(5_000),
        interphase: Micros(500),
        phase2: Micros(3_000),
        expected_energy: Decijoules(1_500),
    }
}

fn lines(armed: bool, at_rest: bool) -> SequencerLines {
    SequencerLines {
        armed,
        relay_at_rest: at_rest,
        reads_empty: true,
    }
}

fn run(seq: &mut Sequencer, us: u32, l: SequencerLines) {
    for _ in 0..us / DT {
        seq.step(l, DT);
    }
}

#[test]
fn nothing_is_commanded_until_the_rail_has_read_armed_for_the_whole_rail_ready_wait() {
    let t = SequencerTiming::from_config(&super::cfg());
    let mut s = Sequencer::new(SequencerJob::Shock(plan()), t);
    run(&mut s, 29_990, lines(true, true));
    assert!(!s.outputs().relay_cmd, "FW-2: not before 30 ms");
    run(&mut s, 20, lines(true, true));
    assert!(s.outputs().relay_cmd);
    // FW-3: a short armed pulse (≤ 7.5 ms of spurious consent) and a break
    // restart the count.
    let mut s = Sequencer::new(SequencerJob::Shock(plan()), t);
    run(&mut s, 7_500, lines(true, true));
    run(&mut s, 10, lines(false, true));
    run(&mut s, 25_000, lines(true, true));
    assert!(
        !s.outputs().relay_cmd,
        "a broken armed reading restarts the wait"
    );
}

#[test]
fn the_high_side_is_commanded_only_14_ms_after_the_last_sample_at_rest() {
    let t = SequencerTiming::from_config(&super::cfg());
    let mut s = Sequencer::new(SequencerJob::Shock(plan()), t);
    run(&mut s, 30_000, lines(true, true));
    run(&mut s, 2_000, lines(true, true)); // relay still at rest
    run(&mut s, 10_000, lines(true, false)); // left rest
    run(&mut s, 50, lines(true, true)); // a bounce back to rest
    run(&mut s, 13_900, lines(true, false));
    assert!(
        !s.outputs().cmd_ha,
        "FW-1: the count restarts on any sample at rest"
    );
    run(&mut s, 200, lines(true, false));
    assert!(s.outputs().cmd_ha && !s.outputs().cmd_lb, "lead");
    run(&mut s, 300, lines(true, false));
    assert!(s.outputs().cmd_lb, "phase 1 after the lead");
}

#[test]
fn the_start_is_given_up_40_ms_after_the_relay_command() {
    let t = SequencerTiming::from_config(&super::cfg());
    let mut s = Sequencer::new(SequencerJob::Shock(plan()), t);
    run(&mut s, 30_000, lines(true, true));
    run(&mut s, 30_000, lines(true, false));
    // Settled after 14 ms: fine. A relay that leaves rest late is not.
    let mut late = Sequencer::new(SequencerJob::Shock(plan()), t);
    run(&mut late, 30_000, lines(true, true));
    run(&mut late, 30_000, lines(true, true));
    run(&mut late, 20_000, lines(true, false));
    assert!(matches!(late.state, SequencerState::Aborted(_)));
    assert!(late.outputs() == Default::default());
    assert!(s.committed());
}

#[test]
fn in_test_mode_the_switches_wait_7_ms_after_the_test_relay() {
    let t = SequencerTiming::from_config(&super::cfg());
    let mut s = Sequencer::new(SequencerJob::Test(plan()), t);
    run(&mut s, 30_000, lines(true, true));
    assert!(s.outputs().btest_cmd && !s.outputs().relay_cmd);
    run(&mut s, 6_990, lines(true, true));
    assert!(!s.outputs().cmd_ha, "FW-7");
    run(&mut s, 20, lines(true, true));
    assert!(s.outputs().cmd_ha);
}

#[test]
fn a_dry_run_is_abandoned_if_the_main_reading_is_not_empty() {
    let t = SequencerTiming::from_config(&super::cfg());
    let mut s = Sequencer::new(SequencerJob::DryRun(plan()), t);
    run(&mut s, 30_000, lines(true, true));
    run(&mut s, 14_000, lines(true, false));
    let charged = SequencerLines {
        reads_empty: false,
        ..lines(true, false)
    };
    run(&mut s, 100, charged);
    assert!(matches!(s.state, SequencerState::Aborted(_)));
    assert!(!s.outputs().cmd_lb);
}

#[test]
fn each_phase_has_its_own_cap() {
    let t = LatchTiming::PROPOSED;
    let active = Latch {
        status: SequenceStatus::Active,
        ..Latch::IDLE
    };
    // Phase 1 held on: cut at the phase-1 cap.
    let mut l = active;
    let mut us = 0;
    while l.status == SequenceStatus::Active {
        l = l.step(
            LatchInputs {
                drive: true,
                ..LatchInputs::default()
            },
            t,
            DT,
        );
        us += DT;
    }
    assert_eq!(
        l.status,
        SequenceStatus::Ended(SequenceEnd::PhaseCapTripped)
    );
    assert!(
        (t.phase1_cap_us..=t.phase1_cap_us + DT).contains(&us),
        "{us} µs"
    );
    // Phase 1 ends, phase 2 held on: cut at the phase-2 cap from its start.
    let mut l = active;
    for _ in 0..100 {
        l = l.step(
            LatchInputs {
                drive: true,
                ..LatchInputs::default()
            },
            t,
            DT,
        );
    }
    for _ in 0..50 {
        l = l.step(LatchInputs::default(), t, DT);
    }
    let mut p2 = 0;
    while l.status == SequenceStatus::Active {
        l = l.step(
            LatchInputs {
                drive: true,
                in_phase2: true,
                ..LatchInputs::default()
            },
            t,
            DT,
        );
        p2 += DT;
    }
    assert_eq!(
        l.status,
        SequenceStatus::Ended(SequenceEnd::PhaseCapTripped)
    );
    assert!(
        (t.phase2_cap_us..=t.phase2_cap_us + 2 * DT).contains(&p2),
        "{p2} µs"
    );
    // A hardware fault latch ends it, and blocks a start.
    let l = active.step(
        LatchInputs {
            drive: true,
            hardware_fault: true,
            ..LatchInputs::default()
        },
        t,
        DT,
    );
    assert_eq!(l.status, SequenceStatus::Ended(SequenceEnd::HardwareFault));
    let armed = Latch {
        status: SequenceStatus::Armed,
        ..Latch::IDLE
    };
    let consent = LatchInputs {
        request: true,
        permit: true,
        relay_on_bridge: true,
        drive: true,
        ..LatchInputs::default()
    };
    for blocked in [
        LatchInputs {
            start_blocked: true,
            ..consent
        },
        LatchInputs {
            hardware_fault: true,
            ..consent
        },
    ] {
        assert_eq!(armed.step(blocked, t, DT).status, SequenceStatus::Armed);
    }
    assert_eq!(armed.step(consent, t, DT).status, SequenceStatus::Active);
}

#[test]
fn the_caps_bands_are_ordered_and_above_the_firmware_maxima() {
    let (e, n, l) = (
        LatchTiming::EARLIEST,
        LatchTiming::PROPOSED,
        LatchTiming::LATEST,
    );
    for (lo, mid, hi) in [
        (e.time_cap_us, n.time_cap_us, l.time_cap_us),
        (e.phase1_cap_us, n.phase1_cap_us, l.phase1_cap_us),
        (e.phase2_cap_us, n.phase2_cap_us, l.phase2_cap_us),
        (e.permit_cap_us, n.permit_cap_us, l.permit_cap_us),
        (e.rearm_us, n.rearm_us, l.rearm_us),
    ] {
        assert!(lo <= mid && mid <= hi);
    }
    let c = Config::default();
    // FW-5: the supervisor's window ends before the hardware's earliest.
    assert!(c.permit_window_ms * 1_000 < e.permit_cap_us);
    // FW-4: the firmware's re-arm wait covers the latest re-arm plus the
    // permit detector's 7.5 ms drop-out and a tick.
    assert!(c.rearm_wait_ms * 1_000 >= l.rearm_us + 7_500 + 1_000);
    // FW-10: the firmware limit sits below the hardware latch's lowest trip.
    assert!(c.overvoltage_limit < Volts(2_073));
}

#[test]
fn the_configuration_refuses_waits_that_do_not_fit_together() {
    let bad = [
        Config {
            armed_min_ms: 40,
            ..Config::default()
        },
        Config {
            relay_settle_ms: 40,
            ..Config::default()
        },
        Config {
            sequence_start_timeout_ms: 60,
            ..Config::default()
        },
        Config {
            permit_window_ms: 60,
            ..Config::default()
        },
        Config {
            battery_load_pulse_max_ms: 2_000,
            ..Config::default()
        },
    ];
    for c in bad {
        assert_eq!(c.validate(), Err(ConfigError::ShockSequence), "{c:?}");
    }
    assert_eq!(Config::default().validate(), Ok(()));
}
