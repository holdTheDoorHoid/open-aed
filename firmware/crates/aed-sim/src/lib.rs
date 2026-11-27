// SPDX-License-Identifier: GPL-3.0-or-later
//! A simulator that runs the real Open AED firmware parts together on an
//! ordinary computer, against a simulated device and patient.
//!
//! SIMULATION ONLY. NOT A MEDICAL DEVICE. See SAFETY.md at the repository
//! root and this crate's README for what the simulation does not model.
//! Passing here shows the firmware parts fit together and the safety rules
//! hold in the model; it shows nothing about a real device.
//!
//! The pieces:
//!
//! * [`plant`]: the physical world (capacitor, charger, discharge load,
//!   bridge, patient, pads, battery, message wire) and fault injection.
//! * [`hw`]: the main controller's and the supervisor's separate views of
//!   it, each with its own sensors and clock.
//! * [`scenario`]: scripted scenarios and the named catalogue.
//! * [`runner`]: runs `aed_core::Rescue` (with the real `aed_rhythm`
//!   analyser and `aed_waveform` planner) and `aed_core::Supervisor`
//!   against the plant, checks the safety invariants at every step and
//!   records a timeline.
//! * [`report`]: readable timelines, summaries and timing tables.
//! * [`campaign`]: the seeded random campaign.
//! * [`standby`]: the device between rescues: weeks or months of periodic
//!   self-tests (decision D-15) and temperature logging (D-48), skipping
//!   the time between wakes.
//! * [`thermal`]: a one-node thermal model of the device and its six
//!   temperature sensors, with injectable sensor faults.
//! * [`record`]: the host-side reader and pretty-printer for the record the
//!   device sends down the dock cable (D-09).

pub mod campaign;
pub mod hw;
pub mod plant;
pub mod report;
pub mod runner;
pub mod scenario;
pub mod standby;
pub mod thermal;
pub mod record;

pub use plant::{
    ConsentLine, Fault, Plant, PlantConfig, SensorModel, ShockEnd, ShockRecord, TestSequenceRecord,
};
pub use runner::{run, Rule, RunResult, ShockTiming, Simulation, Violation};
pub use scenario::{by_name, catalogue, Action, Expect, Noise, Responder, Scenario, Trigger};
