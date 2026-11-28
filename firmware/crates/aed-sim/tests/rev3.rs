// SPDX-License-Identifier: GPL-3.0-or-later
//! Design revision 0.3 (change order FW-1 … FW-19, decisions D-62 … D-67):
//! the firmware against a plant that models the new hardware with its
//! TOLERANCE BANDS (`aed_sim::HwTiming`, `aed_core::interlock::LatchTiming`).
//!
//! What these tests prove, in the model only:
//!
//! * a legitimate shock and a legitimate self-test work at every corner of
//!   the bands, including the worst unit for a legitimate sequence;
//! * each firmware wait, made shorter than the hardware's worst case,
//!   makes a shock or a self-test fail at the corner where it matters (the
//!   class of bug integration issue I-9 found), so the tests would catch a
//!   regression of the wait;
//! * each new hardware element failing, stuck either way, is either found
//!   by the self-test or is listed as hidden (and the test says so);
//! * no self-test, under any single fault injected here, starts a sequence
//!   into the pads with the capacitor at or above 50 V.
//!
//! SIMULATION ONLY: hand-written models of a circuit that has not been
//! built. Nothing here is a claim about real hardware.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    missing_docs
)]

use std::cell::RefCell;
use std::rc::Rc;

use aed_core::interlock::LatchTiming;
use aed_core::selftest::LinkSnapshot;
use aed_core::sequencer::SequencerJob;
use aed_core::{
    Config, Event, FaultKind, MainHardware, MainStatus, RescueState, SelfTest, SelfTestFailure,
    SelfTestProgress, Supervisor, SupervisorFault, TestRequest,
};
use aed_rhythm::synth::{Rhythm, Synth};
use aed_sim::hw::{MainView, SupervisorView};
use aed_sim::{
    by_name, Action, Fault, HwCorner, Plant, PlantConfig, ShockEnd, Simulation, Trigger,
};
use aed_types::{Ohms, PatientMode, ShockPlan, ShockPlanner, Volts};
use aed_waveform::{BtePlanner, WaveformConfig};

// ----- a bare plant, driven as the main controller's board layer would ----

fn plan(mode: PatientMode, ohms: u16) -> ShockPlan {
    BtePlanner::new(WaveformConfig::default())
        .unwrap()
        .plan(mode, Ohms(ohms))
        .unwrap()
}

fn plant(config: PlantConfig) -> Plant {
    Plant::new(config, Synth::new(Rhythm::Asystole, 1), 1)
}

/// A plant charged to `volts`, both holds engaged, pads at `ohms`, the coil
/// supply raised `coil_lead_ms` earlier.
fn ready_plant(config: PlantConfig, volts: f64, ohms: f64, coil_lead_ms: u32) -> Plant {
    let mut p = plant(config);
    p.pad_ohms = Some(ohms);
    p.main_hold = true;
    p.supervisor_hold = true;
    for _ in 0..40 {
        let _ = p.step();
    }
    p.cap_v = volts;
    p.set_coil_supply(true);
    for _ in 0..coil_lead_ms {
        let _ = p.step();
    }
    p
}

/// Raise request and permit for a shock, as the rescue does, and run until
/// the hardware ends the sequence or gives up (300 ms).
fn shock(p: &mut Plant, plan: &ShockPlan) -> Option<aed_sim::ShockRecord> {
    p.request = Some(*plan);
    p.request_raised_us = Some(p.now_us());
    p.load_sequencer(SequencerJob::Shock(*plan));
    p.permit = true;
    for _ in 0..300 {
        let _ = p.step();
        if p.latch.status.locked_out() {
            break;
        }
    }
    p.request = None;
    p.sequencer = None;
    p.permit = false;
    for _ in 0..100 {
        let _ = p.step();
    }
    p.shocks.last().copied()
}

fn corner_config(corner: HwCorner, firmware: Config) -> PlantConfig {
    PlantConfig {
        firmware,
        ..PlantConfig::default()
    }
    .at_corner(corner)
}

#[test]
fn a_legitimate_shock_completes_at_every_corner_of_the_hardware_bands() {
    for corner in HwCorner::ALL {
        for (mode, ohms) in [
            (PatientMode::Adult, 25),
            (PatientMode::Adult, 175),
            (PatientMode::Child, 50),
        ] {
            let pl = plan(mode, ohms);
            let mut p = ready_plant(
                corner_config(corner, Config::default()),
                f64::from(pl.charge_voltage.0),
                f64::from(ohms),
                300,
            );
            let s = shock(&mut p, &pl)
                .unwrap_or_else(|| panic!("{corner:?} {mode:?} {ohms} Ω: no shock"));
            assert_eq!(s.end, ShockEnd::Completed, "{corner:?} {mode:?} {ohms} Ω");
            let ratio = s.delivered_j() / s.planned_j();
            assert!(
                (0.97..=1.03).contains(&ratio),
                "{corner:?}: {:.1} of {:.1} J",
                s.delivered_j(),
                s.planned_j()
            );
            assert_eq!(p.relay_slow_starts, 0);
            println!(
                "{corner:?} {mode:?} {ohms} Ω: rail to delivered {:.1} ms (SIMULATED)",
                s.consent_to_delivered_us() as f64 / 1e3
            );
        }
    }
}

#[test]
fn every_planned_shock_fits_inside_the_earliest_per_phase_caps() {
    // D-63, FW-8: the lowest cap values (10.75 / 7.31 / 23.77 ms) never cut
    // a planned pulse, for every patient resistance and both modes.
    for mode in [PatientMode::Adult, PatientMode::Child] {
        for ohms in (25..=175).step_by(25) {
            let pl = plan(mode, ohms);
            let mut p = ready_plant(
                corner_config(HwCorner::Earliest, Config::default()),
                f64::from(pl.charge_voltage.0),
                f64::from(ohms),
                300,
            );
            let s = shock(&mut p, &pl).unwrap();
            assert_eq!(s.end, ShockEnd::Completed, "{mode:?} {ohms} Ω");
        }
    }
}

/// The firmware waits of FW-1, FW-2, FW-6 and FW-7, each shortened below the
/// hardware's worst case: the start must fail at that corner. With the
/// change-order value it passes (test above). This is the class of bug I-9
/// found: a test that only used nominal hardware would not see it.
#[test]
fn a_relay_settle_wait_shorter_than_the_slowest_transfer_timer_is_refused() {
    let short = Config {
        relay_settle_ms: 10, // the hardware's XFER is up to 12.17 ms
        ..Config::default()
    };
    let pl = plan(PatientMode::Adult, 80);
    let mut p = ready_plant(corner_config(HwCorner::Latest, short), 1_950.0, 80.0, 300);
    assert!(shock(&mut p, &pl).is_none(), "start must be refused");
    let mut p = ready_plant(
        corner_config(HwCorner::Latest, Config::default()),
        1_950.0,
        80.0,
        300,
    );
    assert!(shock(&mut p, &pl).is_some());
}

#[test]
fn a_test_relay_wait_shorter_than_the_slowest_btok_timer_is_refused() {
    let test_plan = Plant::dry_run_plan();
    let run = |firmware: Config| {
        let mut p = ready_plant(corner_config(HwCorner::Latest, firmware), 0.0, 80.0, 0);
        p.request = Some(test_plan);
        p.load_sequencer(SequencerJob::Test(test_plan));
        p.test_permit = true;
        for _ in 0..120 {
            let _ = p.step();
        }
        p.test_sequences.len()
    };
    assert_eq!(run(Config::default()), 1);
    // BTOK is up to 5.52 ms: 4 ms is too short.
    assert_eq!(
        run(Config {
            test_relay_wait_ms: 4,
            ..Config::default()
        }),
        0
    );
    // RAIL_READY is up to 26.67 ms: commanding the switches at 15 + 7 ms
    // after the rail is too early.
    assert_eq!(
        run(Config {
            rail_ready_wait_ms: 15,
            ..Config::default()
        }),
        0
    );
}

#[test]
fn the_sequence_start_timeout_covers_the_slowest_legitimate_start() {
    // FW-6: the adverse unit starts about 59 ms after the rail; the old
    // 50 ms timeout would give up on it (the configuration now refuses it:
    // it must exceed the 30 ms rail-ready wait plus the 40 ms relay limit),
    // 80 ms does not.
    let old = Config {
        sequence_start_timeout_ms: 50,
        ..Config::default()
    };
    assert!(old.validate().is_err());
    {
        let (timeout, shocks) = (80, 1);
        let s = by_name("adult-vf").unwrap();
        let config = Config {
            sequence_start_timeout_ms: timeout,
            ..Config::default()
        };
        let mut scenario = s.clone();
        scenario.plant = scenario.plant.at_corner(HwCorner::Adverse);
        scenario.duration_ms = 40_000;
        let r = Simulation::with_config(&scenario, config).unwrap().run();
        assert_eq!(r.shocks.len(), shocks, "timeout {timeout} ms");
    }
}

#[test]
fn a_short_permit_pulse_never_leads_to_a_relay_command() {
    // FW-3: the permit detector can give up to 7.5 ms of consent from a
    // single pulse. With the request raised, the rail comes up briefly; the
    // sequencer must not command the relay.
    for (firmware, moved) in [
        (Config::default(), false),
        (
            Config {
                rail_ready_wait_ms: 3,
                armed_min_ms: 3,
                ..Config::default()
            },
            true,
        ),
    ] {
        let pl = plan(PatientMode::Adult, 80);
        let mut p = ready_plant(
            corner_config(HwCorner::Latest, firmware),
            1_950.0,
            80.0,
            300,
        );
        p.request = Some(pl);
        p.load_sequencer(SequencerJob::Shock(pl));
        p.inject(Fault::PermitGlitch);
        let mut relay_cmd = false;
        for _ in 0..60 {
            let _ = p.step();
            relay_cmd |= p.seq_out.relay_cmd;
        }
        assert_eq!(
            relay_cmd, moved,
            "rail-ready wait {}",
            firmware.rail_ready_wait_ms
        );
        assert!(p.shocks.is_empty());
    }
}

#[test]
fn the_relay_refuses_without_the_coil_supply_lead() {
    // FW-12: with the reservoir not yet full the relay transfers in up to
    // 31.8 ms and the start falls outside the firmware's 40 ms limit.
    let pl = plan(PatientMode::Adult, 80);
    let mut p = ready_plant(
        corner_config(HwCorner::Latest, Config::default()),
        1_950.0,
        80.0,
        50,
    );
    assert!(shock(&mut p, &pl).is_none());
    assert_eq!(p.relay_slow_starts, 1);
    let mut p = ready_plant(
        corner_config(HwCorner::Latest, Config::default()),
        1_950.0,
        80.0,
        200,
    );
    assert!(shock(&mut p, &pl).is_some());
    // No supply at all: the relay does not move.
    let mut p = ready_plant(
        corner_config(HwCorner::Nominal, Config::default()),
        1_950.0,
        80.0,
        300,
    );
    p.inject(Fault::CoilSupplyDead);
    assert!(shock(&mut p, &pl).is_none());
}

#[test]
fn a_rescue_never_overlaps_the_coil_pre_charge_with_a_full_power_charge() {
    // FW-12 as revised: on a weak battery the overlap browns the rails out.
    for corner in [HwCorner::Nominal, HwCorner::Adverse] {
        let mut s = by_name("adult-vf").unwrap();
        s.plant = s.plant.at_corner(corner);
        s.plant.battery_weak = true;
        s.duration_ms = 40_000;
        let mut sim = Simulation::new(&s).unwrap();
        while sim.plant().borrow().now_ms < s.duration_ms {
            sim.step();
        }
        let p = sim.plant().borrow();
        assert_eq!(p.supply_overlap_ms, 0, "{corner:?}");
        assert_eq!(p.brownout_ms, 0, "{corner:?}");
        assert_eq!(p.relay_slow_starts, 0, "{corner:?}");
        assert_eq!(p.shocks.len(), 1, "{corner:?}");
    }
    // The model does flag the overlap when it happens.
    let mut p = plant(PlantConfig {
        battery_weak: true,
        ..PlantConfig::default()
    });
    p.main_hold = true;
    p.supervisor_hold = true;
    for _ in 0..40 {
        let _ = p.step();
    }
    p.charger_target = Some(Volts(1_950));
    for _ in 0..20 {
        let _ = p.step();
    }
    p.set_coil_supply(true);
    for _ in 0..50 {
        let _ = p.step();
    }
    assert!(p.brownout_ms > 0);
}

// ----- the per-phase caps (D-63) -----------------------------------------

#[test]
fn a_main_controller_frozen_in_phase_1_is_cut_by_the_phase_1_cap() {
    for corner in [HwCorner::Earliest, HwCorner::Latest] {
        for ohms in [25_u16, 50, 100, 175] {
            let pl = plan(PatientMode::Adult, ohms);
            let mut p = ready_plant(
                corner_config(corner, Config::default()),
                1_950.0,
                f64::from(ohms),
                300,
            );
            p.inject(Fault::MainTimerStuck);
            let s = shock(&mut p, &pl).unwrap();
            assert_eq!(s.end, ShockEnd::PhaseCap, "{corner:?} {ohms} Ω");
            let cap: u32 = if corner == HwCorner::Latest {
                11_850
            } else {
                10_750
            };
            assert!(
                s.duration_us() <= u64::from(cap) + 20,
                "{corner:?} {ohms} Ω: {} µs",
                s.duration_us()
            );
            // R3 §4 (SIMULATED, ngspice): ≤ 170.9 J at 1,950 V with the caps
            // at the top of their bands. This model's figure must not exceed
            // that by more than the 5 % the two models may differ.
            assert!(
                s.delivered_j() <= 180.0,
                "{corner:?} {ohms} Ω: {:.1} J",
                s.delivered_j()
            );
            println!(
                "frozen in phase 1, {corner:?}, {ohms} Ω: {:.1} J (SIMULATED)",
                s.delivered_j()
            );
        }
    }
}

#[test]
fn a_timer_left_toggling_gets_one_short_phase_and_the_overall_cap() {
    // §B13.3 rule 1: a second CMD_LB is ignored, so a toggling timer cannot
    // repeat phase 1; the gap then runs into the overall cap.
    let pl = plan(PatientMode::Adult, 80);
    let mut p = ready_plant(PlantConfig::default(), 1_950.0, 80.0, 300);
    p.inject(Fault::MainTimerToggling);
    let s = shock(&mut p, &pl).unwrap();
    assert_eq!(s.end, ShockEnd::TimeCap);
    assert!(s.delivered_j() < 40.0, "{:.1} J", s.delivered_j());
}

// ----- the self-test against the plant: a small harness ------------------

struct Bench {
    plant: Rc<RefCell<Plant>>,
    main: MainView,
    sup: SupervisorView,
    supervisor: Supervisor,
    link: Option<LinkSnapshot>,
    seq: u32,
}

impl Bench {
    fn new(config: PlantConfig) -> Self {
        let plant = Rc::new(RefCell::new(plant(config)));
        Bench {
            main: MainView::new(plant.clone()),
            sup: SupervisorView::new(plant.clone()),
            supervisor: Supervisor::new(config.firmware).unwrap(),
            plant,
            link: None,
            seq: 0,
        }
    }

    fn tick(&mut self, request: TestRequest, charging: bool, planned: Volts) {
        let _ = self.plant.borrow_mut().step();
        if let Some(r) = self.main.supervisor_report() {
            self.link = Some(LinkSnapshot {
                report: r,
                received_at: self.main.now(),
            });
        }
        self.seq += 1;
        let v = self.main.capacitor_voltage();
        self.main.send_status(&MainStatus {
            sequence: self.seq,
            state: RescueState::SelfTest,
            capacitor_voltage: v,
            planned_voltage: planned,
            armed_for_ms: 0,
            temperature: None,
            test_request: request,
            emptying: false,
            charger_on: charging,
        });
        self.supervisor.tick(&mut self.sup);
    }

    /// Run a self-test to its end (at most 20 s).
    fn run(&mut self, st: &mut SelfTest) -> Result<(), SelfTestFailure> {
        for _ in 0..20_000 {
            if let SelfTestProgress::Finished(r) = st.step(&mut self.main, self.link) {
                return r.map(|_| ());
            }
            let planned = if st.is_charging() {
                st.test_voltage()
            } else {
                Volts(0)
            };
            self.tick(st.test_request(), st.is_charging(), planned);
        }
        panic!("self-test did not finish");
    }

    /// Hold both paths and let them engage, as the switch-on does before a test.
    fn settle(&mut self) {
        for _ in 0..200 {
            self.tick(TestRequest::None, false, Volts(0));
        }
    }
}

fn periodic(config: PlantConfig) -> (Result<(), SelfTestFailure>, Bench) {
    let mut b = Bench::new(config);
    b.settle();
    let mut st = SelfTest::periodic(config.firmware, Volts(400));
    let r = b.run(&mut st);
    (r, b)
}

#[test]
fn the_periodic_self_test_with_the_real_permit_steps_passes_at_every_corner() {
    for corner in HwCorner::ALL {
        let (r, b) = periodic(PlantConfig::default().at_corner(corner));
        assert_eq!(r, Ok(()), "{corner:?}");
        let p = b.plant.borrow();
        // T-5 ran into the open pads: one shock-mode sequence, no energy.
        assert_eq!(p.shocks.len(), 1, "{corner:?}: the dry run");
        assert!(p
            .shocks
            .iter()
            .all(|s| s.delivered_j() == 0.0 && s.capacitor_v < 50.0));
        // T-8, T-9 and the latch check: four test sequences, the three caps
        // each ended by the right cap.
        let ends: Vec<ShockEnd> = p.test_sequences.iter().map(|t| t.end).collect();
        assert_eq!(
            ends,
            [
                ShockEnd::Completed,
                ShockEnd::PhaseCap,
                ShockEnd::PhaseCap,
                ShockEnd::TimeCap
            ],
            "{corner:?}"
        );
        assert!(p.ovp_resets >= 1, "{corner:?}: T-6 reset the latch");
        assert_eq!(b.supervisor.fault(), None);
    }
}

#[test]
fn each_new_hardware_element_failing_is_found_by_the_periodic_self_test() {
    use SelfTestFailure as F;
    type Check = fn(&Result<(), SelfTestFailure>, &Bench) -> bool;
    let cases: Vec<(Fault, Check)> = vec![
        (Fault::LeadGateStuck, |r, _| {
            *r == Err(F::LeadGateDidNotRefuse)
        }),
        (Fault::SeqHoldDead, |r, _| *r == Err(F::DryRunFailed)),
        (Fault::CoilSupplyDead, |r, _| *r == Err(F::RelayDidNotMove)),
        (Fault::AuxContactStuckOpen, |r, _| {
            *r == Err(F::RelayNotAtFrontEnds)
        }),
        (Fault::IdleFastStuckTripped, |r, _| {
            *r == Err(F::SequenceLatchFailed)
        }),
        (Fault::LsdOkStuckLow, |r, _| {
            *r == Err(F::SequenceLatchFailed)
        }),
        (Fault::OvpComparatorDead, |r, b| {
            r.is_err() && b.supervisor.fault() == Some(SupervisorFault::OvpTestFailed)
        }),
        (Fault::OvpLatchOpen, |r, b| {
            r.is_err() && b.supervisor.fault() == Some(SupervisorFault::OvpTestFailed)
        }),
        (Fault::OvpResetSwitchShorted, |r, b| {
            r.is_err() && b.supervisor.fault() == Some(SupervisorFault::OvpTestFailed)
        }),
        (Fault::OvpResetCouplingShorted, |r, b| {
            r.is_err() && b.supervisor.fault() == Some(SupervisorFault::OvpTestFailed)
        }),
        (Fault::OvpResetDead, |r, b| {
            r.is_err() && b.supervisor.fault() == Some(SupervisorFault::OvpTestFailed)
        }),
    ];
    let mut all = cases;
    for i in 0..4 {
        // A sensor stuck "held" is found by whichever comes first: T-6 (all
        // four must release while the latch is tripped) or T-7.
        all.push((Fault::HoldSensorStuckHeld(i), |r, _| {
            matches!(r, Err(F::HoldSensor(_) | F::OvpHoldsDidNotFollow))
        }));
        all.push((Fault::HoldSensorStuckNotHeld(i), |r, _| {
            matches!(r, Err(F::HoldSensor(_)))
        }));
    }
    for (fault, check) in all {
        let config = PlantConfig::default().at_corner(HwCorner::Adverse);
        let mut b = Bench::new(config);
        b.plant.borrow_mut().inject(fault);
        b.settle();
        let mut st = SelfTest::periodic(config.firmware, Volts(400));
        let r = b.run(&mut st);
        assert!(
            check(&r, &b),
            "{fault:?}: {r:?}, supervisor {:?}",
            b.supervisor.fault()
        );
        println!("{}: found as {r:?}", fault.describe());
    }
}

#[test]
fn the_hidden_failures_listed_in_the_design_stay_hidden_and_say_so() {
    // HARDWARE_DESIGN.md §B14.5 "still hidden": these pass the self-test.
    // The test documents it; it is not a requirement that they stay hidden.
    // A charger that ignores its set point is masked too: since v0.3 the
    // firmware stops every charge on its own reading (FW-13), so it shows
    // only together with a main sensor that reads low (then its own trip
    // ends the charge: see the charger tests below).
    for fault in [
        Fault::IdleFastStuckClear,
        Fault::LsdOkStuckHigh,
        Fault::LowSideShorted,
        Fault::ChargerOvershoot,
    ] {
        let (r, _) = {
            let config = PlantConfig::default();
            let mut b = Bench::new(config);
            b.plant.borrow_mut().inject(fault);
            b.settle();
            let mut st = SelfTest::periodic(config.firmware, Volts(400));
            (b.run(&mut st), ())
        };
        assert_eq!(r, Ok(()), "{fault:?} (hidden at the 0 V test voltage)");
    }
}

#[test]
fn the_per_phase_caps_are_checked_against_their_bands() {
    use SelfTestFailure as F;
    let p = LatchTiming::PROPOSED;
    for (timing, want) in [
        (
            LatchTiming {
                phase1_cap_us: u32::MAX,
                ..p
            },
            F::PhaseCapDidNotTrip,
        ),
        (
            LatchTiming {
                phase2_cap_us: u32::MAX,
                ..p
            },
            F::PhaseCapDidNotTrip,
        ),
        (
            LatchTiming {
                phase1_cap_us: 6_000,
                ..p
            },
            F::PhaseCapTooShort,
        ),
        (
            LatchTiming {
                time_cap_us: u32::MAX,
                ..p
            },
            F::TimeCapDidNotTrip,
        ),
    ] {
        let config = PlantConfig {
            latch_timing: timing,
            ..PlantConfig::default()
        };
        let (r, _) = periodic(config);
        assert_eq!(r, Err(want));
    }
}

// ----- the over-voltage latch (I-22, T-6, FW-17) ---------------------------

#[test]
fn the_over_voltage_reset_needs_a_pulse_longer_than_the_hardware_minimum() {
    let mut p = plant(PlantConfig::default());
    p.ovp_test = true;
    let _ = p.step();
    p.ovp_test = false;
    let _ = p.step();
    assert!(p.ovp_latched, "latched after the injection");
    // A held-high reset line does not keep it clear, and a pulse shorter
    // than the minimum does not clear it: here the minimum is raised to
    // 2 ms so that the 1 ms plant step can show it.
    p.config.hw.ovp_reset_min_us = 2_000;
    p.ovp_reset = true;
    let _ = p.step();
    p.ovp_reset = false;
    let _ = p.step();
    assert!(
        p.ovp_latched,
        "a 1 ms pulse under a 2 ms minimum must not reset"
    );
    p.ovp_reset = true;
    let _ = p.step();
    let _ = p.step();
    let _ = p.step();
    p.ovp_reset = false;
    let _ = p.step();
    assert!(!p.ovp_latched, "a 3 ms pulse resets");
    // The firmware's pulse (≥ 1 ms) is ten times the real 100 µs minimum.
    const {
        assert!(
            aed_core::supervisor::OVP_RESET_PULSE_MS * 1_000
                >= 10 * aed_sim::HwTiming::LATEST.ovp_reset_min_us
        )
    };
}

#[test]
fn a_real_over_voltage_trip_is_latched_and_reset_only_once_both_readings_are_empty() {
    let config = PlantConfig::default();
    let mut b = Bench::new(config);
    b.settle();
    b.plant.borrow_mut().cap_v = 2_130.0;
    b.plant.borrow_mut().inject(Fault::DischargeStuck);
    for _ in 0..10 {
        b.tick(TestRequest::None, false, Volts(0));
    }
    assert!(b.plant.borrow().ovp_latched);
    assert_eq!(b.supervisor.fault(), Some(SupervisorFault::Overvoltage));
    // Still high: no reset.
    for _ in 0..200 {
        b.tick(TestRequest::None, false, Volts(0));
    }
    assert!(b.plant.borrow().ovp_latched, "no reset while charged");
    assert!(
        !b.plant.borrow().hold_readback().all_held(),
        "the latch forces both paths open"
    );
    // Emptied (the discharge path returns): reset once, recorded.
    b.plant
        .borrow_mut()
        .faults
        .retain(|f| *f != Fault::DischargeStuck);
    for _ in 0..4_000 {
        b.tick(TestRequest::None, false, Volts(0));
    }
    let p = b.plant.borrow();
    assert!(!p.ovp_latched, "reset once both readings are empty");
    assert_eq!(p.ovp_resets, 1);
    assert!(b
        .supervisor
        .events()
        .iter()
        .any(|e| e.event == Event::SupervisorOvpReset));
    // The fault stays: not ready until a self-test passes after power-off.
    assert!(b.supervisor.fault().is_some());
}

#[test]
fn a_latch_that_powers_up_tripped_is_reset_at_switch_on_or_leaves_the_device_not_ready() {
    for (stuck, ok) in [(false, true), (true, false)] {
        let config = PlantConfig {
            ovp_powers_up_tripped: true,
            ..PlantConfig::default()
        };
        let mut b = Bench::new(config);
        if stuck {
            b.plant.borrow_mut().inject(Fault::OvpResetDead);
        }
        b.settle();
        let mut st = SelfTest::new(config.firmware, Volts(400));
        let r = b.run(&mut st);
        assert_eq!(r.is_ok(), ok, "reset dead {stuck}: {r:?}");
        if stuck {
            assert_eq!(b.supervisor.fault(), Some(SupervisorFault::OvpTripped));
        }
    }
}

#[test]
fn the_voltage_check_suspension_lasts_only_for_the_over_voltage_test_step() {
    let config = PlantConfig::default();
    let mut b = Bench::new(config);
    b.settle();
    let mut running_ms = 0;
    for _ in 0..300 {
        b.tick(TestRequest::OvpTest, false, Volts(0));
        if b.supervisor.ovp_test_running() {
            running_ms += 1;
        }
    }
    assert!(b.link.is_some_and(|l| l.report.ovp_test_ok));
    assert!(
        (50..=100).contains(&running_ms),
        "suspended {running_ms} ms"
    );
    assert_eq!(
        b.supervisor.fault(),
        None,
        "the full-scale reading was not acted on"
    );
    // After the step, a real disagreement is latched as before.
    b.plant
        .borrow_mut()
        .config
        .supervisor_voltage_sensor
        .stuck_at = Some(400.0);
    for _ in 0..1_000 {
        b.tick(TestRequest::OvpTest, false, Volts(0));
    }
    assert_eq!(
        b.supervisor.fault(),
        Some(SupervisorFault::VoltageDisagreement)
    );
    assert!(
        !b.supervisor.ovp_test_running(),
        "a second request is not a second test"
    );
}

#[test]
fn the_over_voltage_test_is_entered_only_with_both_readings_empty() {
    let config = PlantConfig::default();
    let mut b = Bench::new(config);
    b.settle();
    b.plant.borrow_mut().cap_v = 300.0;
    b.plant.borrow_mut().inject(Fault::DischargeStuck);
    for _ in 0..300 {
        b.tick(TestRequest::OvpTest, false, Volts(0));
        assert!(
            !b.plant.borrow().ovp_test,
            "OVP_TEST raised with 300 V on the capacitor"
        );
    }
}

// ----- no energy to the pads from a self-test (D-62) ------------------------

#[test]
fn no_self_test_starts_a_sequence_into_the_pads_at_50_v_or_more_under_any_single_fault() {
    // Every single fault of the shock path, the sensors and the charger,
    // with the capacitor at several voltages when the test starts (a
    // charge left behind, for example by a stuck discharge path), the pads
    // on a person (one front end wrongly reading open) or open.
    let faults = [
        None,
        Some(Fault::MainVoltageSensorWrong),
        Some(Fault::SupervisorVoltageSensorWrong),
        Some(Fault::MainVoltageSensorLow),
        Some(Fault::ChargerStuckOn),
        Some(Fault::ChargerRunaway),
        Some(Fault::ChargerOvershoot),
        Some(Fault::DischargeStuck),
        Some(Fault::LeadGateStuck),
        Some(Fault::SeqHoldDead),
        Some(Fault::PermitStuckOn),
        Some(Fault::RequestStuckOn),
        Some(Fault::HoldSensorStuckHeld(0)),
        Some(Fault::OvpComparatorDead),
    ];
    let mut worst_v = 0.0_f64;
    let mut runs = 0;
    for fault in faults {
        for start_v in [0.0, 45.0, 300.0, 1_950.0] {
            for pads_on in [false, true] {
                let mut config = PlantConfig::default();
                if pads_on {
                    // The main controller's front end wrongly reads open.
                    config.main_resistance_sensor.stuck_at = Some(10_000.0);
                }
                let mut b = Bench::new(config);
                {
                    let mut p = b.plant.borrow_mut();
                    p.pad_ohms = pads_on.then_some(80.0);
                    p.cap_v = start_v;
                    if start_v > 0.0 {
                        p.inject(Fault::DischargeStuck);
                    }
                    if let Some(f) = fault {
                        p.inject(f);
                    }
                }
                b.settle();
                let mut st = SelfTest::periodic(config.firmware, Volts(400));
                let _ = b.run(&mut st);
                runs += 1;
                let p = b.plant.borrow();
                for s in &p.shocks {
                    worst_v = worst_v.max(s.capacitor_v);
                    assert!(
                        s.capacitor_v < 50.0 && s.delivered_j() < 0.125,
                        "{fault:?}, start {start_v} V, pads on {pads_on}: sequence from {:.0} V, {:.3} J",
                        s.capacitor_v,
                        s.delivered_j()
                    );
                }
            }
        }
    }
    println!("{runs} self-tests: highest capacitor voltage at any pad sequence {worst_v:.1} V (SIMULATED)");
}

#[test]
fn the_supervisor_withdraws_a_real_self_test_permit_when_the_charger_runs_away() {
    // A charger stuck on during a real-permit step: the supervisor's own
    // reading leaves "empty" and the permit goes before anything starts.
    let config = PlantConfig::default();
    let mut b = Bench::new(config);
    b.main.set_discharge_hold(true);
    b.settle();
    for _ in 0..50 {
        b.tick(TestRequest::RealPermit, false, Volts(0));
    }
    assert!(b.supervisor.real_test_permit_active());
    b.plant.borrow_mut().inject(Fault::ChargerStuckOn);
    for _ in 0..100 {
        b.tick(TestRequest::RealPermit, false, Volts(0));
    }
    assert!(!b.supervisor.real_test_permit_active());
    assert!(
        b.plant.borrow().cap_v > 50.0,
        "the stuck charger did charge"
    );
}

// ----- the charger: silent stop vs its own last-resort trip ----------------

fn rescue_with(fault: Fault, at: u32, extra: &[Fault]) -> (aed_sim::RunResult, u32) {
    let mut s = by_name("adult-vf").unwrap();
    s.duration_ms = 40_000;
    for f in extra {
        s.script.push((Trigger::At(0), Action::Fault(*f)));
    }
    s.script.push((
        Trigger::AfterState {
            state: RescueState::AnalysingCharging,
            nth: 1,
            after_ms: at,
        },
        Action::Fault(fault),
    ));
    let mut sim = Simulation::new(&s).unwrap();
    while sim.plant().borrow().now_ms < s.duration_ms {
        sim.step();
    }
    let restarts = sim
        .rescue()
        .events()
        .iter()
        .filter(|e| matches!(e.event, Event::ChargerRestarted(_)))
        .count() as u32;
    (sim.into_result(), restarts)
}

#[test]
fn a_charger_that_stops_silently_is_restarted_and_the_shock_still_goes() {
    let (r, restarts) = rescue_with(Fault::ChargerSilentStop, 1_500, &[]);
    assert!(restarts >= 1, "FW-18 restart");
    assert_eq!(r.shocks.len(), 1);
    assert_eq!(r.final_fault, None);
}

#[test]
fn a_charger_that_ends_the_charge_by_its_own_trip_is_never_restarted() {
    // Below the plan (the trip fires early) it looks like a stall but for
    // CHG_DONE_N; above it (the charger overshoots while the main sensor
    // reads low) the voltage is beyond the capacitor's normal range.
    for (fault, extra) in [
        (Fault::ChargerTripsEarly, vec![]),
        (Fault::ChargerOvershoot, vec![Fault::MainVoltageSensorLow]),
    ] {
        let (r, restarts) = rescue_with(fault, 0, &extra);
        assert_eq!(restarts, 0, "{fault:?}: FW-18 must not refill");
        assert!(r.shocks.is_empty(), "{fault:?}");
        assert!(r.final_fault.is_some(), "{fault:?}: not ready");
        assert!(r.final_cap_v < 50.0, "{fault:?}: emptied");
        if fault == Fault::ChargerTripsEarly {
            assert_eq!(r.final_fault, Some(FaultKind::ChargerStoppedItself));
        }
    }
}

// ----- compression blanking while charging (FW-13, D-67) ------------------

#[test]
fn compression_readings_are_blanked_while_charging_without_failing_the_channel() {
    // A slow charge (20 W, about 9.5 s) is longer than the 5 s after which a
    // silent impedance channel counts as failed: the blanking gap must not.
    let mut s = by_name("adult-vf").unwrap();
    s.plant.charger_watts = 20.0;
    s.duration_ms = 40_000;
    let mut sim = Simulation::new(&s).unwrap();
    let mut blanked_ms = 0;
    while sim.plant().borrow().now_ms < s.duration_ms {
        sim.step();
        let p = sim.plant().borrow();
        let t = p.main_clock();
        if sim.rescue().chest_blanked(t) {
            blanked_ms += 1;
            assert!(
                sim.rescue().chest_state(t).is_none(),
                "no chest state from blanked samples"
            );
        }
    }
    assert!(blanked_ms > 5_000, "blanked {blanked_ms} ms");
    assert!(
        !sim.rescue()
            .events()
            .iter()
            .any(|e| matches!(e.event, Event::ChestSensingFailed(_))),
        "the blanking gap was counted as a failed channel"
    );
    assert_eq!(sim.into_result().shocks.len(), 1);
}

// ----- the board layer (FW-14, FW-19) -------------------------------------

#[test]
fn the_battery_load_pulse_is_not_repeated_within_a_second() {
    let shared = Rc::new(RefCell::new(plant(PlantConfig::default())));
    let mut main = MainView::new(shared.clone());
    let _ = main.battery_under_load();
    for _ in 0..500 {
        let _ = shared.borrow_mut().step();
    }
    let _ = main.battery_under_load();
    assert_eq!(shared.borrow().battery_load_pulses.len(), 1);
    for _ in 0..600 {
        let _ = shared.borrow_mut().step();
    }
    let _ = main.battery_under_load();
    assert_eq!(shared.borrow().battery_load_pulses.len(), 2);
}

#[test]
fn the_self_test_current_counts_as_present_at_the_test_voltage() {
    let c = Config::default();
    let mut p = plant(PlantConfig::default());
    for v in [40.0, 50.0] {
        p.cap_v = v;
        assert!(
            p.test_current_ma() >= c.current_present_ma,
            "{v} V: {} mA",
            p.test_current_ma()
        );
    }
    p.cap_v = 0.0;
    assert!(p.test_current_ma() < c.current_present_ma);
}
