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
//! * [`interlock`]: the hardware gate (the third check) and the shock
//!   sequence latch (D-59), written down as pure functions for the
//!   simulator to model and the hardware designer to check against.
//! * [`selftest`]: the self-test sequence.
//! * [`schedule`]: the periodic self-tests while the device is off
//!   (decision D-15): when to run them, the record, and the ready indicator.
//! * [`heat`]: temperature sensing, the heat states, the charging limits,
//!   the pad heat allowance and the heat record (decisions D-41, D-48,
//!   D-51, D-52).
//! * [`standby`]: the device while "off": one wake handler for the heat
//!   record and the self-test schedule, the capacitor wear count (D-53),
//!   and what must survive power loss.
//! * [`readiness`]: the one readiness result, with every reason.
//! * [`export`]: the record download by cable through the dock (D-09).
//! * [`prompts`]: every spoken or displayed message.
//! * [`events`]: the fixed-size event record.
//! * [`link`]: the two messages the controllers exchange.
#![no_std]

#[cfg(test)]
extern crate std;

pub mod config;
pub mod events;
pub mod export;
pub mod hal;
pub mod heat;
pub mod interlock;
pub mod link;
pub mod prompts;
pub mod readiness;
pub mod rescue;
pub mod schedule;
pub mod selftest;
pub mod standby;
pub mod supervisor;

pub use config::{Config, ConfigError};
pub use events::{Event, EventRecord, TimedEvent};
pub use hal::{
    BatteryStatus, BridgeTestOutcome, BridgeTestResult, BridgeTestStep, MainHardware,
    PadCartridge, RelayPosition, SequenceEnd, SequenceStatus, SupervisorHardware,
};
pub use link::{MainStatus, ShockCheck, SupervisorFault, SupervisorReport, TestRequest};
pub use export::{crc32_update, write_record, EXPORT_FORMAT_VERSION};
pub use heat::{
    Assessment, ChargeContext, DeciC, HeatBand, HeatInhibit, HeatMonitor, SensorFault, SensorSet,
    ServiceCheck, ServiceRefusal, TempPoint,
};
pub use prompts::Prompt;
pub use readiness::{DeviceReadiness, Reason, ReasonSet};
pub use rescue::{AbortReason, Counters, FaultKind, Rescue, RescueState, ShockOutcome};
pub use schedule::{
    Deferral, NotReadyReason, PeriodicOutcome, PeriodicTest, PeriodicTestRun, Readiness,
    TestCounters, TestGate, TestSchedule, WakeDecision, WakeInputs,
};
pub use standby::{
    CapacitorWear, DurableRecord, DurableStore, Standby, StandbyDecision, StandbyInputs,
    StoreError, WearStatus,
};
pub use selftest::{SelfTest, SelfTestFailure, SelfTestPass, SelfTestProgress};
pub use supervisor::{
    PermitRefusal, PermitWithdrawal, ReleaseReason, Supervisor, TempIgnored, TestPermitRefusal,
};

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    missing_docs
)]
mod tests;
