// SPDX-License-Identifier: GPL-3.0-or-later
//! The two messages the controllers exchange.
//!
//! The main controller and the supervisor share no memory. The only things
//! that pass between them are these two small messages: the main
//! controller's status (which doubles as its heartbeat) and the supervisor's
//! report back. On real hardware they would travel over a wire between two
//! chips; here they are plain values handed to the hardware traits.

use crate::heat::DeciC;
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
    /// The hottest of the main controller's own trusted readings at the
    /// capacitor and the two cells, or `None` if it has none (D-48). The
    /// supervisor uses it only as a plausibility check on its own sensor:
    /// it holds a charge back for heat on its own reading, unless that
    /// reading is far above this one (then its own sensor is the suspect,
    /// and one failed sensor must not block every shock).
    pub temperature: Option<DeciC>,
    /// What the main controller's self-test asks of the supervisor (D-61).
    /// Honoured only while `state` is [`RescueState::SelfTest`].
    pub test_request: TestRequest,
    /// True while the main controller has let go of its discharge path and
    /// is waiting for the capacitor to empty in the background. The
    /// voltage then falls fast (one path alone: a 0.4 s time constant in
    /// the model) while the claim above is up to a heartbeat plus the wire
    /// delay old, so the supervisor does not compare the two then, for at
    /// most [`crate::Config::disarm_time_limit_ms`].
    pub emptying: bool,
    /// True while the main controller has the charger commanded on. The
    /// supervisor gives the REAL permit in a self-test only with this false
    /// (D-62, change order FW-16).
    pub charger_on: bool,
}

/// What the main controller's self-test asks of the supervisor (D-61).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestRequest {
    /// Nothing.
    #[default]
    None,
    /// Give the bounded, low-voltage test permit.
    TestPermit,
    /// Let go of the discharge path (path B) for a moment, so the main
    /// controller can time path B emptying the capacitor on its own.
    ReleasePathB,
    /// Give the REAL shock permit for one self-test step (D-62, FW-16):
    /// only with both controllers' readings empty and the charger off, for
    /// at most [`crate::Config::real_test_permit_ms`].
    RealPermit,
    /// As [`TestRequest::RealPermit`], for the dry shock-mode run (D-62
    /// step T-5): also only with the supervisor's OWN pad reading open.
    RealPermitDryRun,
    /// Run the over-voltage latch test (D-62 step T-6): `OVP_TEST` up, see
    /// the trip, `OVP_TEST` down, see it stay, pulse `OVP_RST`, see it clear.
    OvpTest,
}

/// What a controller concluded about a finished shock sequence from its own
/// capacitor reading afterwards ([`crate::Config::check_shock_residual`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockCheck {
    /// The voltage fell into the range a delivered shock leaves.
    Delivered,
    /// The voltage hardly fell: no energy left the capacitor.
    NotDelivered,
    /// The voltage fell further than any planned shock can take it, or
    /// stopped part-way: something other than the plan happened.
    Implausible,
    /// The hardware reported its time cap or over-current trip.
    HardwareTrip,
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
    /// After a sequence, the capacitor voltage fell to a level no planned
    /// shock leaves, or stopped part-way (D-59).
    ShockImplausible,
    /// The hardware reported that its time cap or over-current trip ended a
    /// sequence (D-59).
    SequenceTripped,
    /// The hardware reported a sequence running while the supervisor had
    /// given no permit and had not just withdrawn one: a consent path is
    /// shorted.
    SequenceWithoutPermit,
    /// The over-voltage latch test failed (D-62 step T-6): no trip on
    /// `OVP_TEST`, the latch did not hold, or `OVP_RST` did not clear it.
    OvpTestFailed,
    /// The hardware over-voltage latch tripped outside a test (I-22). The
    /// supervisor resets it only once both readings are empty (FW-17).
    OvpTripped,
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
    /// True while the supervisor holds charging back because its own
    /// temperature reading is over its limit (D-41, D-48). Not a fault: it
    /// clears when the device cools. The main controller treats it like its
    /// own heat limit (no charge; CPR guidance).
    pub too_hot: bool,
    /// The supervisor's own temperature reading (capacitor body), if it
    /// trusts it.
    pub temperature: Option<DeciC>,
    /// True while the supervisor is giving the self-test's test permit.
    pub test_permit: bool,
    /// The supervisor's own verdict on the most recent shock sequence of
    /// this arming, from its own voltage reading, if it has one.
    pub shock_check: Option<ShockCheck>,
    /// The over-voltage latch test (D-62 step T-6) has passed since the
    /// main controller last asked for it.
    pub ovp_test_ok: bool,
}
