// SPDX-License-Identifier: GPL-3.0-or-later
//! The rescue sequence and the safety logic around the shock.
//!
//! NOT A MEDICAL DEVICE. See SAFETY.md at the repository root. This code has
//! only ever run against test mocks; it has never controlled real hardware.
//!
//! What is here:
//!
//! * [`config`]: every timing and limit, in one place, all provisional.
//! * [`hal`]: what the main controller and the supervisor can each see and
//!   do, as two separate traits.
//! * [`rescue`]: the main controller's rescue sequence for a fully automatic
//!   defibrillator (decision D-06).
//! * [`supervisor`]: the second, independent controller that must give a
//!   permit before any shock and can empty the capacitor on its own.
//! * [`interlock`]: the hardware gate (the third check), written down as a
//!   pure function for the simulator to model.
//! * [`selftest`]: the self-test sequence.
//! * [`prompts`]: every spoken or displayed message.
//! * [`events`]: the fixed-size event record.
//! * [`link`]: the two messages the controllers exchange.
#![no_std]

#[cfg(test)]
extern crate std;

pub mod config;
pub mod events;
pub mod hal;
pub mod interlock;
pub mod link;
pub mod prompts;
pub mod rescue;
pub mod selftest;
pub mod supervisor;

pub use config::{Config, ConfigError};
pub use events::{Event, EventRecord, TimedEvent};
pub use hal::{BatteryStatus, MainHardware, PadCartridge, ShockStatus, SupervisorHardware};
pub use link::{MainStatus, SupervisorFault, SupervisorReport};
pub use prompts::Prompt;
pub use rescue::{AbortReason, Counters, FaultKind, Rescue, RescueState, ShockOutcome};
pub use selftest::{SelfTest, SelfTestFailure, SelfTestProgress};
pub use supervisor::{PermitRefusal, ReleaseReason, Supervisor};

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    missing_docs
)]
mod tests;
