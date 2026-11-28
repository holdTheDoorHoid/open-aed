// SPDX-License-Identifier: GPL-3.0-or-later
//! The hardware interlock and the shock-sequence logic, written down as
//! pure functions and one pure state machine.
//!
//! On the real device this is plain logic gates, not software: the third of
//! the three independent checks before a shock (ARCHITECTURE.md). These
//! functions describe what that logic must do, so that the simulator can use
//! them as its model, the tests can check every combination, and the
//! hardware designer has the requirement in one place. Nothing in the
//! firmware calls them to make a decision.
//!
//! **Consent starts a shock; it does not have to last through it (D-59).**
//!
//! 1. The interlock rail rises only with the main controller's request AND
//!    the supervisor's permit AND the discharge path held off AND no
//!    lockout ([`sequence_may_start`]). Under the self-test's test permit
//!    (D-61) the rail may also rise, but only below the hardware's test
//!    voltage limit, and the patient relay is never powered
//!    ([`Latch::relay_coil`]).
//! 2. Removing any consent before the first conducting moment drops the
//!    rail at once and nothing has happened.
//! 3. At the first conducting moment (the phase timer's first drive with
//!    the relay on the bridge) the hardware latches SEQUENCE ACTIVE. From
//!    then the request, permit and hold lines are ignored: the planned
//!    pulse completes, unless the non-retriggerable time cap expires first
//!    or the over-current trip fires. Drive is then removed. Since design
//!    revision 0.3 (D-63) each phase also has its own cap, and the other
//!    hardware fault latches (idle current, rail or low-side driver supply)
//!    end it too.
//! 4. The rail may stay up without a sequence starting for at most the
//!    permit-duration cap; then it is removed and locked out.
//! 5. After a sequence (or the permit-duration cap) the hardware is locked
//!    out until request, permit and test permit have all been absent
//!    together for the re-arm time: one sequence per permit.
//!
//! The state machine runs in microseconds so the simulator can step it
//! through a pulse.

use crate::hal::{SequenceEnd, SequenceStatus};

/// True when the discharge path is active, meaning the capacitor is being
/// emptied into the internal load.
///
/// The path is active unless BOTH controllers hold it open ("discharges by
/// default"). Either controller letting go, or losing power, empties the
/// capacitor.
#[must_use]
pub const fn discharge_path_active(main_hold: bool, supervisor_hold: bool) -> bool {
    !(main_hold && supervisor_hold)
}

/// True when a shock sequence may START (the interlock rail may rise).
///
/// Only when the main controller's shock request AND the supervisor's
/// permit are both present AND the discharge path is not active AND the
/// hardware is not locked out after an earlier sequence. Any one missing
/// blocks the start. This gates the start only: once a sequence is active
/// it completes (see [`Latch`]).
#[must_use]
pub const fn sequence_may_start(
    main_request: bool,
    supervisor_permit: bool,
    discharge_active: bool,
    locked_out: bool,
) -> bool {
    main_request && supervisor_permit && !discharge_active && !locked_out
}

/// True when the TEST rail may rise (D-61): the request AND the test permit,
/// the shock permit absent (both permits together give neither), the
/// discharge path not active, not locked out, and the capacitor below the
/// hardware's own test-voltage limit.
#[must_use]
pub const fn test_rail_may_rise(
    main_request: bool,
    test_permit: bool,
    supervisor_permit: bool,
    discharge_active: bool,
    locked_out: bool,
    below_test_voltage: bool,
) -> bool {
    main_request
        && test_permit
        && !supervisor_permit
        && !discharge_active
        && !locked_out
        && below_test_voltage
}

/// True when the capacitor charger is able to run.
///
/// **This is a requirement on the future hardware design**, not something
/// the firmware decides: the charger's power stage must be cut off in
/// hardware whenever the discharge path is active, whatever the main
/// controller is asking for. Because the discharge path is active whenever
/// EITHER controller lets go (see [`discharge_path_active`]), the
/// supervisor can always stop the charger on its own simply by letting go,
/// even if the main controller has frozen with its charger command stuck
/// on. Without this rule a frozen main controller leaves the charger
/// feeding the discharge load and the capacitor never empties (simulation
/// finding F-3).
///
/// `charger_enabled` is the main controller's charger command;
/// `discharge_active` is the output of [`discharge_path_active`].
#[must_use]
pub const fn charger_may_run(charger_enabled: bool, discharge_active: bool) -> bool {
    charger_enabled && !discharge_active
}

/// The hardware's fixed timings, in microseconds, for ONE unit.
///
/// The real figures have tolerance bands (component tolerance, temperature,
/// leakage). [`LatchTiming::PROPOSED`] is the nominal unit;
/// [`LatchTiming::EARLIEST`] and [`LatchTiming::LATEST`] are the ends of each
/// band, so a simulation can check that the firmware works with every unit
/// the design allows, not just the nominal one. Sources: HARDWARE_DESIGN.md
/// v0.3 §B13.3 and the simulation notes `hardware/aed-main/sim/logic-relay/
/// notes.md` §1.3 and §2 (simulated) and `sim/rev3/notes.md` §4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatchTiming {
    /// The non-retriggerable overall time cap from the first conducting
    /// moment. 25 ms nominal, 23.77–26.22 ms simulated (LR §2). Since v0.3
    /// a backstop behind the per-phase caps.
    pub time_cap_us: u32,
    /// The phase-1 cap (D-63, change order FW-9): from the first conducting
    /// moment until phase 1 has ended. 11.37 ms nominal, 10.75–11.85 ms
    /// (calculated band, §B13.3 rule 4).
    pub phase1_cap_us: u32,
    /// The phase-2 cap (D-63, FW-9): from the first phase-2 conducting
    /// moment until the sequence is done. 7.73 ms nominal, 7.31–8.06 ms.
    pub phase2_cap_us: u32,
    /// Longest the rail may stay up before a sequence starts (the 150 ms
    /// window on sheet 09). 150 ms nominal, 131.8–161.5 ms simulated
    /// (LR §1.3); FW-5 changed the model from 250 ms.
    pub permit_cap_us: u32,
    /// How long request, permit and test permit must all be absent together
    /// before a lockout clears (the re-arm comparator). 56.28 ms nominal,
    /// 51.24–60.34 ms simulated (LR §1.3); FW-4 changed the model from
    /// 20 ms. The firmware waits longer still (75 ms,
    /// [`crate::Config::rearm_wait_ms`]) because the permit detector also
    /// holds consent for up to 7.5 ms after the supervisor stops its clock.
    pub rearm_us: u32,
}

impl LatchTiming {
    /// The nominal unit.
    pub const PROPOSED: LatchTiming = LatchTiming {
        time_cap_us: 25_000,
        phase1_cap_us: 11_370,
        phase2_cap_us: 7_730,
        permit_cap_us: 150_000,
        rearm_us: 56_280,
    };
    /// Every figure at the low end of its band.
    pub const EARLIEST: LatchTiming = LatchTiming {
        time_cap_us: 23_770,
        phase1_cap_us: 10_750,
        phase2_cap_us: 7_310,
        permit_cap_us: 131_800,
        rearm_us: 51_240,
    };
    /// Every figure at the high end of its band.
    pub const LATEST: LatchTiming = LatchTiming {
        time_cap_us: 26_220,
        phase1_cap_us: 11_850,
        phase2_cap_us: 8_060,
        permit_cap_us: 161_500,
        rearm_us: 60_340,
    };
}

/// Everything the sequence logic sees at one instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LatchInputs {
    /// The main controller's request line.
    pub request: bool,
    /// The supervisor's shock permit line.
    pub permit: bool,
    /// The supervisor's test permit line (D-61).
    pub test_permit: bool,
    /// The discharge path is active ([`discharge_path_active`]).
    pub discharge_active: bool,
    /// The hardware's comparator says the capacitor is below the
    /// test-voltage limit.
    pub below_test_voltage: bool,
    /// The patient relay's position sense reads "bridge".
    pub relay_on_bridge: bool,
    /// The phase timer is commanding a conducting phase now.
    pub drive: bool,
    /// The phase timer has finished its last phase.
    pub phases_done: bool,
    /// The over-current comparator is tripped.
    pub over_current: bool,
    /// The phase timer is in phase 2 (after the gap). The phase-2 cap runs
    /// from the first conducting moment with this set (D-63).
    pub in_phase2: bool,
    /// A hardware fault latch other than over-current fired: idle current
    /// (either comparator), `BRIDGE_5V` under-voltage or `LSD_OK` low
    /// (v0.3, §B13.3 rule 6). During a sequence it ends it; before one it
    /// blocks the start.
    pub hardware_fault: bool,
    /// A start condition of sheet 12 other than consent is not met (rail
    /// ready, relay transferred, coil window, lead, mode checks): a
    /// conducting command does not latch the sequence.
    pub start_blocked: bool,
}

/// The sequence latch: what the hardware remembers between instants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Latch {
    /// What the hardware reports ([`crate::hal::SequenceStatus`]).
    pub status: SequenceStatus,
    /// The current rail episode is under a test permit.
    pub test: bool,
    /// Microseconds in the current status.
    pub in_status_us: u32,
    /// Microseconds that request, permit and test permit have all been
    /// absent together (only counted while locked out).
    pub consent_absent_us: u32,
    /// Phase 1 has ended in this sequence (its cap no longer applies).
    pub phase1_done: bool,
    /// Microseconds since phase 2 first conducted, if it has.
    pub phase2_us: Option<u32>,
}

impl Default for Latch {
    fn default() -> Self {
        Self::IDLE
    }
}

impl Latch {
    /// At rest.
    pub const IDLE: Latch = Latch {
        status: SequenceStatus::Idle,
        test: false,
        in_status_us: 0,
        consent_absent_us: 0,
        phase1_done: false,
        phase2_us: None,
    };

    /// True while the interlock rail is up (drive power available).
    #[must_use]
    pub const fn rail_up(&self) -> bool {
        matches!(self.status, SequenceStatus::Armed | SequenceStatus::Active)
    }

    /// True while the patient relay coil is powered: rail up in a real
    /// (not test) episode. Under a test permit the relay cannot move.
    #[must_use]
    pub const fn relay_coil(&self) -> bool {
        self.rail_up() && !self.test
    }

    /// True if the timer's drive reaches the switches now. Only while the
    /// sequence is active; before that the first drive is what latches it.
    #[must_use]
    pub const fn drive_passes(&self) -> bool {
        matches!(self.status, SequenceStatus::Active)
    }

    const fn to(self, status: SequenceStatus, test: bool) -> Latch {
        Latch {
            status,
            test,
            in_status_us: 0,
            consent_absent_us: 0,
            phase1_done: false,
            phase2_us: None,
        }
    }

    const fn stay(self, dt_us: u32) -> Latch {
        Latch {
            in_status_us: self.in_status_us.saturating_add(dt_us),
            ..self
        }
    }

    /// The consent for a new rail episode, if any: `Some(false)` for a
    /// shock, `Some(true)` for a test, `None` for no consent.
    #[must_use]
    pub const fn consent(i: &LatchInputs) -> Option<bool> {
        if sequence_may_start(
            i.request,
            i.permit && !i.test_permit,
            i.discharge_active,
            false,
        ) {
            Some(false)
        } else if test_rail_may_rise(
            i.request,
            i.test_permit,
            i.permit,
            i.discharge_active,
            false,
            i.below_test_voltage,
        ) {
            Some(true)
        } else {
            None
        }
    }

    /// Advance by `dt_us` microseconds with inputs `i`.
    #[must_use]
    pub const fn step(self, i: LatchInputs, t: LatchTiming, dt_us: u32) -> Latch {
        match self.status {
            SequenceStatus::Idle => match Self::consent(&i) {
                Some(test) => self.to(SequenceStatus::Armed, test),
                None => self.stay(dt_us),
            },
            SequenceStatus::Armed => {
                // Consent must stay exactly as it was raised (a shock rail
                // does not turn into a test rail or the other way round).
                let still = matches!(Self::consent(&i), Some(test) if test == self.test);
                if !still {
                    // Withdrawn before the first conducting moment: nothing
                    // has happened, the rail drops.
                    self.to(SequenceStatus::Idle, false)
                } else if i.drive
                    && (self.test || i.relay_on_bridge)
                    && !i.start_blocked
                    && !i.hardware_fault
                {
                    self.to(SequenceStatus::Active, self.test)
                } else if self.in_status_us.saturating_add(dt_us) >= t.permit_cap_us {
                    self.to(SequenceStatus::LockedOut, self.test)
                } else {
                    self.stay(dt_us)
                }
            }
            SequenceStatus::Active => {
                // The consent lines are not looked at here (D-59).
                let elapsed = self.in_status_us.saturating_add(dt_us);
                let phase1_done = self.phase1_done || i.in_phase2 || !i.drive;
                let phase2_us = match self.phase2_us {
                    Some(us) => Some(us.saturating_add(dt_us)),
                    None if i.in_phase2 && i.drive => Some(0),
                    None => None,
                };
                let phase2_over = match phase2_us {
                    Some(us) => us > t.phase2_cap_us,
                    None => false,
                };
                if i.over_current {
                    self.to(
                        SequenceStatus::Ended(SequenceEnd::OverCurrentTripped),
                        self.test,
                    )
                } else if i.hardware_fault {
                    self.to(SequenceStatus::Ended(SequenceEnd::HardwareFault), self.test)
                } else if (!self.phase1_done && elapsed > t.phase1_cap_us) || phase2_over {
                    self.to(
                        SequenceStatus::Ended(SequenceEnd::PhaseCapTripped),
                        self.test,
                    )
                } else if elapsed > t.time_cap_us {
                    self.to(
                        SequenceStatus::Ended(SequenceEnd::TimeCapTripped),
                        self.test,
                    )
                } else if i.phases_done {
                    self.to(SequenceStatus::Ended(SequenceEnd::Completed), self.test)
                } else {
                    Latch {
                        in_status_us: elapsed,
                        phase1_done,
                        phase2_us,
                        ..self
                    }
                }
            }
            SequenceStatus::Ended(_) | SequenceStatus::LockedOut => {
                if i.request || i.permit || i.test_permit {
                    Latch {
                        consent_absent_us: 0,
                        ..self.stay(dt_us)
                    }
                } else {
                    let absent = self.consent_absent_us.saturating_add(dt_us);
                    if absent >= t.rearm_us {
                        Latch::IDLE
                    } else {
                        Latch {
                            consent_absent_us: absent,
                            ..self.stay(dt_us)
                        }
                    }
                }
            }
        }
    }
}
