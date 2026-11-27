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

use aed_core::{
    BatteryStatus, BridgeTestResult, BridgeTestStep, DeciC, MainHardware, MainStatus,
    PadCartridge, Prompt, RelayPosition, SensorSet, SequenceStatus, SupervisorHardware,
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
        match plan {
            Some(plan) => p.request = Some(*plan),
            None => p.request_bare = true,
        }
        if !was_up {
            let now = p.now_us();
            p.request_raised_us = Some(now);
        }
    }
    fn withdraw_shock_request(&mut self) {
        let mut p = self.plant.borrow_mut();
        p.request = None;
        p.request_bare = false;
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
            measurement: 0,
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
        self.plant.borrow().battery
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
