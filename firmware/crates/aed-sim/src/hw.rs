// SPDX-License-Identifier: GPL-3.0-or-later
//! The two controllers' views of the shared plant.
//!
//! [`MainView`] implements [`MainHardware`] and [`SupervisorView`]
//! implements [`SupervisorHardware`]. Each reads the plant through its OWN
//! sensors and its OWN clock (see [`crate::plant::PlantConfig`]); they share
//! nothing but the physics and the message wire. The plant is shared through
//! `Rc<RefCell<..>>`; each trait call borrows it only for the length of
//! that call.

use std::cell::RefCell;
use std::rc::Rc;

use aed_core::sequencer::SequencerJob;
use aed_core::{
    BatteryStatus, BridgeTestResult, BridgeTestStep, DeciC, HoldReadback, MainHardware, MainStatus,
    PadCartridge, Probe, Prompt, RelayPosition, SensorSet, SequenceStatus, SupervisorHardware,
    SupervisorReport, TempPoint,
};
use aed_types::{Millis, Ohms, PatientMode, ShockPlan, Volts};

use crate::plant::Plant;

/// The plant, shared between the two views and the simulation runner.
pub type SharedPlant = Rc<RefCell<Plant>>;

/// What the main controller can see and do.
#[derive(Debug, Clone)]
pub struct MainView {
    plant: SharedPlant,
}

impl MainView {
    /// A view of `plant` through the main controller's sensors.
    #[must_use]
    pub fn new(plant: SharedPlant) -> Self {
        MainView { plant }
    }
}

impl MainHardware for MainView {
    fn now(&self) -> Millis {
        self.plant.borrow().main_clock()
    }
    fn pad_resistance(&mut self) -> Ohms {
        self.plant.borrow().main_ohms()
    }
    fn capacitor_voltage(&mut self) -> Volts {
        self.plant.borrow().main_volts()
    }
    fn set_charger(&mut self, target: Option<Volts>) {
        self.plant.borrow_mut().charger_target = target;
    }
    fn set_discharge_hold(&mut self, hold: bool) {
        self.plant.borrow_mut().main_hold = hold;
    }
    fn request_shock(&mut self, plan: Option<&ShockPlan>) {
        let mut p = self.plant.borrow_mut();
        let was_up = p.request.is_some() || p.request_bare;
        // The board layer loads its delivery sequencer: a self-test plan
        // (no charge voltage) runs in test mode, a rescue plan in shock mode.
        let job = match plan {
            Some(plan) => {
                p.request = Some(*plan);
                if plan.charge_voltage.0 == 0 {
                    SequencerJob::Test(*plan)
                } else {
                    SequencerJob::Shock(*plan)
                }
            }
            None => {
                p.request_bare = true;
                SequencerJob::RailOnly
            }
        };
        p.load_sequencer(job);
        if !was_up {
            let now = p.now_us();
            p.request_raised_us = Some(now);
        }
    }
    fn request_probe(&mut self, probe: Probe) {
        let mut p = self.plant.borrow_mut();
        let was_up = p.request.is_some() || p.request_bare;
        p.request_bare = true;
        let job = match probe {
            Probe::RelayMove { hold_ms } => SequencerJob::RelayMove {
                hold_us: hold_ms.saturating_mul(1_000),
            },
            Probe::StartWithoutLead => SequencerJob::StartWithoutLead,
            Probe::DryRun => SequencerJob::DryRun(crate::plant::Plant::dry_run_plan()),
        };
        p.load_sequencer(job);
        if !was_up {
            let now = p.now_us();
            p.request_raised_us = Some(now);
        }
    }
    fn withdraw_shock_request(&mut self) {
        let mut p = self.plant.borrow_mut();
        p.request = None;
        p.request_bare = false;
        // Lowering the request does not stop the main controller's phase
        // timer once it has commanded phase 1 (D-59: the hardware
        // completes the sequence); the board layer drops it once done.
        if !p.sequencer.is_some_and(|s| s.committed()) {
            p.sequencer = None;
        }
    }
    fn hold_readback(&mut self) -> HoldReadback {
        self.plant.borrow().hold_readback()
    }
    fn set_coil_supply(&mut self, on: bool) {
        let mut p = self.plant.borrow_mut();
        p.set_coil_supply(on);
    }
    fn charger_done(&mut self) -> bool {
        self.plant.borrow().charger_done
    }
    fn sequence_status(&mut self) -> SequenceStatus {
        self.plant.borrow().latch.status
    }
    fn patient_relay(&mut self) -> RelayPosition {
        self.plant.borrow().relay_position()
    }
    fn bridge_test_step(&mut self, step: BridgeTestStep) -> BridgeTestResult {
        let p = self.plant.borrow();
        BridgeTestResult {
            outcome: p.bridge_test_step(step),
            // FW-14: the board layer's test-load current, mA.
            measurement: p.test_current_ma(),
        }
    }
    fn cancel_pressed(&mut self) -> bool {
        self.plant.borrow().cancel_pressed()
    }
    fn patient_mode(&mut self) -> PatientMode {
        self.plant.borrow().mode
    }
    fn patient_mode_ceiling(&mut self) -> PatientMode {
        let p = self.plant.borrow();
        p.mode_ceiling.unwrap_or(p.mode)
    }
    fn battery(&mut self) -> BatteryStatus {
        self.plant.borrow().battery
    }
    fn battery_under_load(&mut self) -> BatteryStatus {
        self.plant.borrow_mut().battery_load_test()
    }
    fn pad_cartridge(&mut self) -> PadCartridge {
        self.plant.borrow().cartridge
    }
    fn temperatures(&mut self) -> SensorSet {
        let mut s = self.plant.borrow().thermal.sensor_set();
        s.set(TempPoint::Supervisor, None);
        s
    }
    fn prompt(&mut self, prompt: Prompt) {
        let mut p = self.plant.borrow_mut();
        let now = p.now_ms;
        p.prompts.push((now, prompt));
    }
    fn metronome_beat(&mut self) {
        let mut p = self.plant.borrow_mut();
        p.metronome_beats = p.metronome_beats.saturating_add(1);
    }
    fn set_ready_indicator(&mut self, ready: bool) {
        self.plant.borrow_mut().ready = ready;
    }
    fn send_status(&mut self, status: &MainStatus) {
        self.plant.borrow_mut().send_to_supervisor(*status);
    }
    fn supervisor_report(&mut self) -> Option<SupervisorReport> {
        self.plant.borrow_mut().receive_at_main()
    }
}

/// What the supervisor can see and do.
#[derive(Debug, Clone)]
pub struct SupervisorView {
    plant: SharedPlant,
}

impl SupervisorView {
    /// A view of `plant` through the supervisor's own sensors.
    #[must_use]
    pub fn new(plant: SharedPlant) -> Self {
        SupervisorView { plant }
    }
}

impl SupervisorHardware for SupervisorView {
    fn now(&self) -> Millis {
        self.plant.borrow().supervisor_clock()
    }
    fn capacitor_voltage(&mut self) -> Volts {
        self.plant.borrow().supervisor_volts()
    }
    fn pad_resistance(&mut self) -> Ohms {
        self.plant.borrow().supervisor_ohms()
    }
    fn cancel_pressed(&mut self) -> bool {
        self.plant.borrow().cancel_pressed()
    }
    fn temperature(&mut self) -> Option<DeciC> {
        self.plant.borrow().thermal.reading(TempPoint::Supervisor)
    }
    fn set_permit(&mut self, permit: bool) {
        self.plant.borrow_mut().permit = permit;
    }
    fn set_test_permit(&mut self, on: bool) {
        self.plant.borrow_mut().test_permit = on;
    }
    fn set_ovp_test(&mut self, on: bool) {
        self.plant.borrow_mut().ovp_test = on;
    }
    fn set_ovp_reset(&mut self, high: bool) {
        self.plant.borrow_mut().ovp_reset = high;
    }
    fn ovp_tripped(&mut self) -> bool {
        self.plant.borrow().ovp_latched
    }
    fn sequence_status(&mut self) -> SequenceStatus {
        self.plant.borrow().latch.status
    }
    fn set_discharge_hold(&mut self, hold: bool) {
        self.plant.borrow_mut().supervisor_hold = hold;
    }
    fn receive_status(&mut self) -> Option<MainStatus> {
        self.plant.borrow_mut().receive_at_supervisor()
    }
    fn send_report(&mut self, report: &SupervisorReport) {
        self.plant.borrow_mut().send_to_main(*report);
    }
    fn set_fault_alarm(&mut self, on: bool) {
        self.plant.borrow_mut().supervisor_alarm = on;
    }
}
