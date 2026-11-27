// SPDX-License-Identifier: GPL-3.0-or-later
//! The event record: a fixed-size list of what happened and when, for the
//! hand-over to paramedics and for review afterwards.
//!
//! It never grows: when it is full, the oldest entry is overwritten and an
//! overflow counter goes up, so a reviewer can see that entries were lost.

use crate::link::SupervisorFault;
use crate::rescue::{AbortReason, FaultKind, RescueState, ShockOutcome};
use crate::selftest::SelfTestFailure;
use crate::supervisor::{PermitRefusal, ReleaseReason};
use aed_types::{Millis, Ohms, PatientMode, PlanError, RhythmDecision, ShockPlan, Volts};

/// How many events the main controller keeps.
pub const EVENT_CAPACITY: usize = 256;
/// How many events the supervisor keeps (it records far less).
pub const SUPERVISOR_EVENT_CAPACITY: usize = 64;

/// One thing that happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The controller started.
    PowerOn,
    /// The rescue moved from one state to another.
    StateChanged {
        /// State left.
        from: RescueState,
        /// State entered.
        to: RescueState,
    },
    /// The rhythm analysis gave a decision.
    Decision(RhythmDecision),
    /// No decision arrived within the analysis time limit (treated as "not sure").
    AnalysisTimedOut,
    /// Pad contact became good (after the debounce time).
    PadsOn,
    /// Pad contact was lost (after the debounce time).
    PadsOff,
    /// Pad resistance measured just before a shock.
    PadResistance(Ohms),
    /// The adult/child switch was moved.
    PatientModeChanged(PatientMode),
    /// Charging started towards this voltage.
    ChargeStarted(Volts),
    /// The capacitor reached the planned voltage.
    Charged(Volts),
    /// The shock plan made just before delivery.
    Plan(ShockPlan),
    /// The planner refused to make a plan.
    PlanRefused(PlanError),
    /// The shock request line was raised.
    ShockRequested,
    /// What happened to a requested shock.
    Shock(ShockOutcome),
    /// A shock being prepared was abandoned.
    ShockAborted(AbortReason),
    /// The capacitor was confirmed empty.
    CapacitorEmptied(Volts),
    /// A fault was found.
    Fault(FaultKind),
    /// The cancel control was pressed.
    CancelPressed,
    /// The battery became low.
    BatteryLow,
    /// The self-test passed.
    SelfTestPassed,
    /// The self-test failed (or gave an advisory warning) at this check.
    SelfTestFailed(SelfTestFailure),
    /// The supervisor gave a shock permit (recorded by the supervisor; the main
    /// controller records it from the supervisor's report).
    SupervisorPermitted,
    /// The supervisor refused a permit.
    SupervisorRefused(PermitRefusal),
    /// The supervisor let go of the discharge path, emptying the capacitor.
    SupervisorReleasedHold(ReleaseReason),
    /// The supervisor latched a fault.
    SupervisorFaultLatched(SupervisorFault),
}

/// An event with the time it happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedEvent {
    /// When, on the clock of the controller that recorded it.
    pub at: Millis,
    /// What.
    pub event: Event,
}

/// A fixed-capacity ring of events. Holds the newest `N`; older ones are
/// overwritten and counted.
#[derive(Debug, Clone)]
pub struct EventRecord<const N: usize> {
    slots: [Option<TimedEvent>; N],
    next: usize,
    len: usize,
    overwritten: u32,
}

impl<const N: usize> Default for EventRecord<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> EventRecord<N> {
    /// An empty record.
    #[must_use]
    pub const fn new() -> Self {
        EventRecord {
            slots: [None; N],
            next: 0,
            len: 0,
            overwritten: 0,
        }
    }

    /// Add an event, overwriting the oldest if full.
    pub fn push(&mut self, at: Millis, event: Event) {
        let Some(slot) = self.slots.get_mut(self.next) else {
            // Only possible when N is zero: nothing can be kept.
            self.overwritten = self.overwritten.saturating_add(1);
            return;
        };
        *slot = Some(TimedEvent { at, event });
        self.next = self.next.saturating_add(1).checked_rem(N).unwrap_or(0);
        if self.len < N {
            self.len = self.len.saturating_add(1);
        } else {
            self.overwritten = self.overwritten.saturating_add(1);
        }
    }

    /// How many events are held now.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True if nothing has been recorded (or kept).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// How many events were lost because the record was full.
    #[must_use]
    pub const fn overwritten(&self) -> u32 {
        self.overwritten
    }

    /// The events held, oldest first.
    #[must_use]
    pub fn iter(&self) -> EventIter<'_, N> {
        // Index of the oldest entry: `len` places behind `next`, wrapping.
        let start = self
            .next
            .saturating_add(N.saturating_sub(self.len))
            .checked_rem(N)
            .unwrap_or(0);
        EventIter {
            record: self,
            start,
            taken: 0,
        }
    }
}

/// Walks an [`EventRecord`] from oldest to newest.
#[derive(Debug)]
pub struct EventIter<'a, const N: usize> {
    record: &'a EventRecord<N>,
    start: usize,
    taken: usize,
}

impl<const N: usize> Iterator for EventIter<'_, N> {
    type Item = TimedEvent;

    fn next(&mut self) -> Option<TimedEvent> {
        if self.taken >= self.record.len {
            return None;
        }
        let index = self.start.saturating_add(self.taken).checked_rem(N)?;
        self.taken = self.taken.saturating_add(1);
        self.record.slots.get(index).copied().flatten()
    }
}
