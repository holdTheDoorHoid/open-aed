// SPDX-License-Identifier: GPL-3.0-or-later
//! The two messages the controllers exchange.
//!
//! The main controller and the supervisor share no memory. The only things
//! that pass between them are these two small messages: the main
//! controller's status (which doubles as its heartbeat) and the supervisor's
//! report back. On real hardware they would travel over a wire between two
//! chips; here they are plain values handed to the hardware traits.

use crate::rescue::RescueState;
use aed_types::Volts;

/// The main controller's status message. It is sent every heartbeat period and
/// whenever the state changes. The supervisor trusts none of the numbers in it
/// on their own: it compares them with its own readings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainStatus {
    /// Counts up by one with every message. A number that stops changing means
    /// the main controller has stopped (the heartbeat).
    pub sequence: u32,
    /// The state the main controller says it is in.
    pub state: RescueState,
    /// The capacitor voltage as the main controller reads it.
    pub capacitor_voltage: Volts,
    /// The voltage the main controller is charging to, or zero if none.
    pub planned_voltage: Volts,
    /// How long the main controller says the capacitor has been armed (since
    /// charging began), in milliseconds; zero when not armed.
    pub armed_for_ms: u32,
}

/// A fault the supervisor has latched. Once latched it stays until power-off:
/// the supervisor holds the capacitor empty and will not give a permit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisorFault {
    /// The main controller's heartbeat stopped after it had started.
    HeartbeatLost,
    /// The capacitor stayed charged longer than the limit.
    ChargedTooLong,
    /// The capacitor went above the overvoltage limit.
    Overvoltage,
    /// The supervisor's own voltage reading and the main controller's claim
    /// disagreed for longer than allowed.
    VoltageDisagreement,
    /// The supervisor's own view of the cancel control read pressed without
    /// a break for [`crate::Config::cancel_stuck_ms`] (D-31). A stuck cancel
    /// would block every permit, silently, so it is a fault.
    CancelStuck,
}

/// The supervisor's report back to the main controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupervisorReport {
    /// Counts up by one with every report (the supervisor's heartbeat).
    pub sequence: u32,
    /// The latched fault, if any.
    pub fault: Option<SupervisorFault>,
    /// True while the supervisor is giving a shock permit.
    pub permit: bool,
    /// True while the supervisor is holding the discharge path open
    /// (allowing the capacitor to stay charged).
    pub holding: bool,
    /// The capacitor voltage as the supervisor reads it, on its own sensor.
    pub capacitor_voltage: Volts,
}
