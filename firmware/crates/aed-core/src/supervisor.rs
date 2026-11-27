// SPDX-License-Identifier: GPL-3.0-or-later
//! The supervisor: the second, independent check before any shock.
//!
//! It runs on a separate, simpler chip in the real design. It shares nothing
//! with the main controller ([`crate::rescue::Rescue`]): it sees only its own
//! sensors (through [`SupervisorHardware`]) and the status messages the main
//! controller sends it.
//!
//! It does two jobs (ARCHITECTURE.md, "Three independent checks before a shock"):
//!
//! 1. **Permit.** It raises the permit line only when ALL of these hold:
//!    the main controller's heartbeat is fresh; the main controller says it
//!    is delivering; it has said "counting down" continuously for at least the
//!    countdown time, measured on the supervisor's own clock; the supervisor's
//!    own voltage reading agrees with the main controller's claim and with the
//!    planned voltage, and is under the overvoltage limit; its own pad
//!    reading is in the patient range; cancel is not pressed. The permit lasts
//!    only a short window and is given at most once per arming.
//! 2. **Emptying.** It holds the discharge path open only while the main
//!    controller says it is charging or armed. It lets go, which empties the
//!    capacitor, on its own if: the heartbeat stops; the capacitor stays
//!    charged too long; the voltage goes over the limit; its reading and the
//!    main controller's claim disagree; or cancel is pressed. The first four
//!    are faults and are latched until power-off.
//!
//! It does NOT second-guess the heart rhythm (an open design question in
//! ARCHITECTURE.md).

use crate::config::{within, Config, ConfigError};
use crate::events::{Event, EventRecord, SUPERVISOR_EVENT_CAPACITY};
use crate::hal::SupervisorHardware;
use crate::link::{MainStatus, SupervisorFault, SupervisorReport};
use crate::rescue::RescueState;
use aed_types::{Millis, Ohms, Volts, MAX_PATIENT_OHMS, MIN_PATIENT_OHMS};

/// Why the supervisor refused a permit. Recorded in its event record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermitRefusal {
    /// A fault is latched.
    Faulted,
    /// The main controller's heartbeat is not fresh.
    HeartbeatStale,
    /// A permit was already given during this arming.
    AlreadyUsed,
    /// The cancel control is pressed.
    CancelPressed,
    /// The main controller has not been counting down for the full countdown
    /// time on the supervisor's clock (or skipped the countdown).
    CountdownTooShort,
    /// The supervisor is not holding the discharge path open.
    NotHolding,
    /// The supervisor's voltage reading is over the limit.
    Overvoltage,
    /// The supervisor's voltage reading disagrees with the main controller's claim.
    VoltageDisagreesWithMain,
    /// The supervisor's voltage reading is not at the planned charge voltage.
    VoltageNotAtPlan,
    /// The supervisor's own pad reading is outside the patient range.
    PadsOutOfRange,
}

/// Why the supervisor let go of the discharge path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseReason {
    /// The main controller no longer says it is charging or armed (normal).
    MainNotArmed,
    /// The main controller's heartbeat went stale.
    HeartbeatStale,
    /// The cancel control was pressed.
    Cancel,
    /// A fault was latched.
    Fault(SupervisorFault),
}

/// The supervisor controller.
#[derive(Debug, Clone)]
pub struct Supervisor {
    config: Config,
    started: bool,
    last_status: Option<MainStatus>,
    last_status_at: Option<Millis>,
    countdown_since: Option<Millis>,
    permit_used: bool,
    last_refusal: Option<PermitRefusal>,
    permit_until: Option<Millis>,
    holding: bool,
    fault: Option<SupervisorFault>,
    charged_since: Option<Millis>,
    disagree_since: Option<Millis>,
    report_sequence: u32,
    last_report_at: Option<Millis>,
    last_report_flags: (bool, bool, Option<SupervisorFault>),
    events: EventRecord<SUPERVISOR_EVENT_CAPACITY>,
}

/// True for the states in which the main controller may want the capacitor charged.
const fn holdable(state: RescueState) -> bool {
    matches!(
        state,
        RescueState::SelfTest
            | RescueState::Charging
            | RescueState::Countdown
            | RescueState::Delivering
    )
}

/// True for the states that make up one "arming" (charging through delivery).
const fn armed(state: RescueState) -> bool {
    matches!(
        state,
        RescueState::Charging | RescueState::Countdown | RescueState::Delivering
    )
}

/// True if a resistance is in the range the device must shock into.
fn in_patient_range(ohms: Ohms) -> bool {
    ohms >= MIN_PATIENT_OHMS && ohms <= MAX_PATIENT_OHMS
}

impl Supervisor {
    /// A new supervisor, holding nothing and permitting nothing.
    ///
    /// # Errors
    /// Refuses a configuration whose numbers contradict each other.
    pub fn new(config: Config) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Supervisor {
            config,
            started: false,
            last_status: None,
            last_status_at: None,
            countdown_since: None,
            permit_used: false,
            last_refusal: None,
            permit_until: None,
            holding: false,
            fault: None,
            charged_since: None,
            disagree_since: None,
            report_sequence: 0,
            last_report_at: None,
            last_report_flags: (false, false, None),
            events: EventRecord::new(),
        })
    }

    /// The latched fault, if any.
    #[must_use]
    pub const fn fault(&self) -> Option<SupervisorFault> {
        self.fault
    }

    /// True while a permit is being given.
    #[must_use]
    pub const fn permit_active(&self) -> bool {
        self.permit_until.is_some()
    }

    /// True while the supervisor holds the discharge path open.
    #[must_use]
    pub const fn holding(&self) -> bool {
        self.holding
    }

    /// The supervisor's own event record.
    #[must_use]
    pub const fn events(&self) -> &EventRecord<SUPERVISOR_EVENT_CAPACITY> {
        &self.events
    }

    /// Run one step. Call frequently (every few milliseconds).
    pub fn tick(&mut self, hw: &mut impl SupervisorHardware) {
        let now = hw.now();
        if !self.started {
            self.started = true;
            self.events.push(now, Event::PowerOn);
        }

        if let Some(status) = hw.receive_status() {
            let is_new = self.last_status.map(|s| s.sequence) != Some(status.sequence);
            if is_new {
                self.on_status(status, now);
            }
        }

        let fresh = self
            .last_status_at
            .is_some_and(|t| now.since(t) <= self.config.heartbeat_timeout_ms);
        let voltage = hw.capacitor_voltage();
        let pads = hw.pad_resistance();
        let cancel = hw.cancel_pressed();

        self.check_latched_faults(now, fresh, voltage);

        // Decide whether to hold the discharge path open.
        let claim = self.last_status.map(|s| s.state);
        let hold = self.fault.is_none() && fresh && claim.is_some_and(holdable) && !cancel;
        if self.holding && !hold {
            let reason = if let Some(f) = self.fault {
                ReleaseReason::Fault(f)
            } else if cancel {
                ReleaseReason::Cancel
            } else if !fresh {
                ReleaseReason::HeartbeatStale
            } else {
                ReleaseReason::MainNotArmed
            };
            self.events.push(now, Event::SupervisorReleasedHold(reason));
        }
        self.holding = hold;
        if cancel && claim.is_some_and(armed) {
            // Cancel spends this arming's permit: no shock until the main
            // controller has left the armed states and armed again.
            self.permit_used = true;
        }

        // Decide the permit.
        if self.permit_until.is_some() {
            let expired = self.permit_until.is_some_and(|until| now >= until);
            let still_ok = self.check_permit(now, fresh, voltage, pads, cancel).is_ok();
            if expired || !still_ok {
                self.permit_until = None;
            }
        } else if fresh && claim == Some(RescueState::Delivering) {
            let result = if self.permit_used {
                Err(PermitRefusal::AlreadyUsed)
            } else {
                self.check_permit(now, fresh, voltage, pads, cancel)
            };
            match result {
                Ok(()) => {
                    self.permit_until =
                        Some(Millis(now.0.saturating_add(self.config.permit_window_ms)));
                    self.permit_used = true;
                    self.events.push(now, Event::SupervisorPermitted);
                }
                Err(reason) => {
                    if self.last_refusal != Some(reason) {
                        self.last_refusal = Some(reason);
                        self.events.push(now, Event::SupervisorRefused(reason));
                    }
                }
            }
        }

        // Lower the permit before letting go of the hold.
        hw.set_permit(self.permit_until.is_some());
        hw.set_discharge_hold(self.holding);

        self.send_report_if_due(hw, now, voltage);
    }

    fn on_status(&mut self, status: MainStatus, now: Millis) {
        if !armed(status.state) {
            // The arming (if any) is over: the next one gets a fresh permit.
            self.permit_used = false;
            self.last_refusal = None;
            self.countdown_since = None;
        } else if status.state == RescueState::Countdown {
            if self.countdown_since.is_none() {
                self.countdown_since = Some(now);
            }
        } else if status.state == RescueState::Charging {
            self.countdown_since = None;
        }
        self.last_status = Some(status);
        self.last_status_at = Some(now);
    }

    fn latch(&mut self, now: Millis, fault: SupervisorFault) {
        if self.fault.is_none() {
            self.fault = Some(fault);
            self.events.push(now, Event::SupervisorFaultLatched(fault));
        }
    }

    fn check_latched_faults(&mut self, now: Millis, fresh: bool, voltage: Volts) {
        if self.last_status_at.is_some() && !fresh {
            self.latch(now, SupervisorFault::HeartbeatLost);
        }
        if voltage > self.config.overvoltage_limit {
            self.latch(now, SupervisorFault::Overvoltage);
        }
        if voltage >= self.config.empty_voltage {
            let since = *self.charged_since.get_or_insert(now);
            if now.since(since) > self.config.max_charged_ms {
                self.latch(now, SupervisorFault::ChargedTooLong);
            }
        } else {
            self.charged_since = None;
        }
        // Compare readings only while the capacitor is meant to be held (it
        // changes slowly then). While emptying, it falls too fast for a
        // heartbeat-old claim to be comparable.
        let claim = self.last_status.map(|s| s.capacitor_voltage);
        match claim {
            Some(claimed) if self.holding && fresh => {
                if within(voltage, claimed, self.config.voltage_tolerance) {
                    self.disagree_since = None;
                } else {
                    let since = *self.disagree_since.get_or_insert(now);
                    if now.since(since) > self.config.disagreement_persist_ms {
                        self.latch(now, SupervisorFault::VoltageDisagreement);
                    }
                }
            }
            _ => self.disagree_since = None,
        }
    }

    fn check_permit(
        &self,
        now: Millis,
        fresh: bool,
        voltage: Volts,
        pads: Ohms,
        cancel: bool,
    ) -> Result<(), PermitRefusal> {
        if self.fault.is_some() {
            return Err(PermitRefusal::Faulted);
        }
        let Some(status) = self.last_status.filter(|_| fresh) else {
            return Err(PermitRefusal::HeartbeatStale);
        };
        if cancel {
            return Err(PermitRefusal::CancelPressed);
        }
        let counted = self
            .countdown_since
            .is_some_and(|since| now.since(since) >= self.config.countdown_ms);
        if !counted {
            return Err(PermitRefusal::CountdownTooShort);
        }
        if !self.holding {
            return Err(PermitRefusal::NotHolding);
        }
        if voltage > self.config.overvoltage_limit {
            return Err(PermitRefusal::Overvoltage);
        }
        if !within(
            voltage,
            status.capacitor_voltage,
            self.config.voltage_tolerance,
        ) {
            return Err(PermitRefusal::VoltageDisagreesWithMain);
        }
        if status.planned_voltage.0 == 0
            || !within(
                voltage,
                status.planned_voltage,
                self.config.voltage_tolerance,
            )
        {
            return Err(PermitRefusal::VoltageNotAtPlan);
        }
        if !in_patient_range(pads) {
            return Err(PermitRefusal::PadsOutOfRange);
        }
        Ok(())
    }

    fn send_report_if_due(&mut self, hw: &mut impl SupervisorHardware, now: Millis, v: Volts) {
        let flags = (self.permit_until.is_some(), self.holding, self.fault);
        let due = self
            .last_report_at
            .is_none_or(|t| now.since(t) >= self.config.heartbeat_period_ms);
        if due || flags != self.last_report_flags {
            self.report_sequence = self.report_sequence.wrapping_add(1);
            hw.send_report(&SupervisorReport {
                sequence: self.report_sequence,
                fault: self.fault,
                permit: flags.0,
                holding: flags.1,
                capacitor_voltage: v,
            });
            self.last_report_at = Some(now);
            self.last_report_flags = flags;
        }
    }
}
