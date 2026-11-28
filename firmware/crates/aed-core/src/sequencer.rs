// SPDX-License-Identifier: GPL-3.0-or-later
//! The main controller's delivery sequencer: the timer-driven board-layer
//! code that, once the interlock rail is up, commands the patient relay,
//! the bridge test relay and the four bridge switches in the order and with
//! the waits that the shock logic on sheet 12 requires (HARDWARE_DESIGN.md
//! v0.3 §B13.1 steps 5–12).
//!
//! It is a pure state machine stepped in microseconds, so the simulator can
//! run it against a plant whose hardware timings sit anywhere inside their
//! tolerance bands. The waits (change order FW-1, FW-2, FW-3, FW-7) come
//! from [`Config`] and are sized to cover the hardware's WORST case, not
//! its nominal: a wait shorter than the hardware's slowest unit makes the
//! hardware refuse the start, which the simulator tests catch.
//!
//! * **FW-3** — a `BRIDGE_ARMED_N` low shorter than
//!   [`Config::armed_min_ms`] (10 ms) is not "armed": the permit detector
//!   can give up to 7.5 ms of consent from a single edge or the
//!   supervisor's own power-up (logic-relay notes §1.2, simulated). Here the
//!   rail must read armed WITHOUT A BREAK for the whole rail-ready wait, so
//!   such a pulse never leads to `RELAY_CMD`.
//! * **FW-2** — nothing (`RELAY_CMD`, `BTEST_CMD`, any `CMD_*`) is
//!   commanded until [`Config::rail_ready_wait_ms`] (30 ms) after the rail
//!   reads armed: `RAIL_READY` comes 21.56–26.67 ms after it (simulated).
//! * **FW-1** — `CMD_HA` only once the relay's rest contact has read "left
//!   rest" continuously for [`Config::relay_settle_ms`] (14 ms), restarting
//!   on any sample at rest (contact bounce); abort if that has not happened
//!   [`Config::relay_settle_abort_ms`] (40 ms) after `RELAY_CMD`. The
//!   hardware's own transfer timer is 10.42–12.17 ms (simulated).
//! * **FW-7** — in test mode, `CMD_*` only [`Config::test_relay_wait_ms`]
//!   (7 ms) after `BTEST_CMD`; the hardware's test-relay timer is
//!   4.74–5.52 ms (simulated, with change D-16).
//!
//! Once phase 1 has been commanded the rail is ignored (D-59: the hardware
//! completes the sequence).

use crate::config::Config;
use aed_types::ShockPlan;

/// The waits, in microseconds (see the module notes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequencerTiming {
    /// The rail must read armed without a break for this long before
    /// anything is commanded (FW-2, covering FW-3).
    pub rail_ready_wait_us: u32,
    /// Relay settled: rest contact left without a break for this long (FW-1).
    pub relay_settle_us: u32,
    /// Give up if the relay has not settled this long after `RELAY_CMD`.
    pub relay_abort_us: u32,
    /// Test relay settled this long after `BTEST_CMD` (FW-7).
    pub test_relay_wait_us: u32,
    /// `CMD_HA` (or `CMD_HB`) leads its low side by this long (hardware
    /// needs ≥ 50 µs, §B13.3 rule 3).
    pub lead_us: u32,
    /// Changeover steps in the gap: high side off, other high side on
    /// (hardware needs ≥ 10 µs after the low side, §B13.3 rule 2).
    pub changeover_us: u32,
}

impl SequencerTiming {
    /// The waits from the firmware configuration.
    #[must_use]
    pub const fn from_config(c: &Config) -> Self {
        SequencerTiming {
            rail_ready_wait_us: c.rail_ready_wait_ms.saturating_mul(1_000),
            relay_settle_us: c.relay_settle_ms.saturating_mul(1_000),
            relay_abort_us: c.relay_settle_abort_ms.saturating_mul(1_000),
            test_relay_wait_us: c.test_relay_wait_ms.saturating_mul(1_000),
            lead_us: 200,
            changeover_us: 20,
        }
    }
}

/// What the sequencer has been asked to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequencerJob {
    /// Rail only: command nothing (self-test probes of the interlock).
    RailOnly,
    /// A rescue shock: relay to the bridge, then both phases.
    Shock(ShockPlan),
    /// A shock-mode sequence in the self-test (D-62 step T-5, pads open):
    /// as [`SequencerJob::Shock`], but only while the main controller's own
    /// reading says the capacitor is empty.
    DryRun(ShockPlan),
    /// A test-mode sequence into the internal load (D-61): test relay,
    /// then both phases. The patient relay is not commanded.
    Test(ShockPlan),
    /// Move the patient relay to the bridge and hold it for this long,
    /// commanding no switch (D-62 steps T-3, T-3b).
    RelayMove {
        /// How long `RELAY_CMD` is held, µs.
        hold_us: u32,
    },
    /// Move the relay, then command a low side WITHOUT its high-side lead
    /// (D-62 step T-4): the hardware must refuse the start.
    StartWithoutLead,
}

/// What the board layer reads for the sequencer at each step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SequencerLines {
    /// `BRIDGE_ARMED_N` reads low (rail up).
    pub armed: bool,
    /// `K_REST_N` reads low (patient relay at rest).
    pub relay_at_rest: bool,
    /// The main controller's own capacitor reading counts as empty
    /// ([`Config::reads_empty`]); used only by [`SequencerJob::DryRun`].
    pub reads_empty: bool,
}

/// The sequencer's output lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SequencerOutputs {
    /// `RELAY_CMD`.
    pub relay_cmd: bool,
    /// `BTEST_CMD`.
    pub btest_cmd: bool,
    /// `CMD_HA`.
    pub cmd_ha: bool,
    /// `CMD_HB`.
    pub cmd_hb: bool,
    /// `CMD_LB` (phase 1).
    pub cmd_lb: bool,
    /// `CMD_LA` (phase 2).
    pub cmd_la: bool,
}

/// Why the sequencer gave up (it then commands nothing more).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequencerAbort {
    /// The relay had not settled [`Config::relay_settle_abort_ms`] after
    /// `RELAY_CMD`.
    RelayNotSettled,
    /// A dry run found the capacitor reading not empty.
    VoltageNotEmpty,
}

/// Where the sequencer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequencerState {
    /// Waiting for the rail to read armed for the whole rail-ready wait.
    WaitArmed {
        /// µs armed without a break.
        armed_us: u32,
    },
    /// `RELAY_CMD` raised; waiting for the relay to settle.
    Relay {
        /// µs since `RELAY_CMD`.
        since_cmd_us: u32,
        /// µs since the rest contact last read at rest (None: never left).
        left_for_us: Option<u32>,
    },
    /// `BTEST_CMD` raised; waiting for the test relay.
    TestRelay {
        /// µs since `BTEST_CMD`.
        us: u32,
    },
    /// The high-side lead before phase 1.
    Lead {
        /// µs since `CMD_HA`.
        us: u32,
    },
    /// Phase 1 (`CMD_LB`).
    Phase1 {
        /// µs into phase 1.
        us: u32,
    },
    /// The gap between the phases.
    Gap {
        /// µs into the gap.
        us: u32,
    },
    /// Phase 2 (`CMD_LA`).
    Phase2 {
        /// µs into phase 2.
        us: u32,
    },
    /// A low side without its lead (T-4).
    LowSideOnly {
        /// µs since `CMD_LB`.
        us: u32,
    },
    /// Finished; nothing commanded.
    Done,
    /// Gave up; nothing commanded.
    Aborted(SequencerAbort),
}

/// The delivery sequencer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sequencer {
    /// The job.
    pub job: SequencerJob,
    /// The waits.
    pub timing: SequencerTiming,
    /// Where it is.
    pub state: SequencerState,
}

impl Sequencer {
    /// A sequencer for `job`, waiting for the rail.
    #[must_use]
    pub const fn new(job: SequencerJob, timing: SequencerTiming) -> Self {
        Sequencer {
            job,
            timing,
            state: SequencerState::WaitArmed { armed_us: 0 },
        }
    }

    const fn plan(&self) -> Option<ShockPlan> {
        match self.job {
            SequencerJob::Shock(p) | SequencerJob::DryRun(p) | SequencerJob::Test(p) => Some(p),
            _ => None,
        }
    }

    /// True once phase 1 has been commanded (the hardware completes it).
    #[must_use]
    pub const fn committed(&self) -> bool {
        matches!(
            self.state,
            SequencerState::Phase1 { .. }
                | SequencerState::Gap { .. }
                | SequencerState::Phase2 { .. }
                | SequencerState::LowSideOnly { .. }
                | SequencerState::Done
        )
    }

    /// True from the gap on: the phase timer is in phase 2.
    #[must_use]
    pub const fn in_phase2(&self) -> bool {
        matches!(self.state, SequencerState::Phase2 { .. })
    }

    /// The output lines in the current state.
    #[must_use]
    pub const fn outputs(&self) -> SequencerOutputs {
        let t = self.timing;
        let off = SequencerOutputs {
            relay_cmd: false,
            btest_cmd: false,
            cmd_ha: false,
            cmd_hb: false,
            cmd_lb: false,
            cmd_la: false,
        };
        let test = matches!(self.job, SequencerJob::Test(_));
        let relay = !test;
        let base = SequencerOutputs {
            relay_cmd: relay,
            btest_cmd: test,
            ..off
        };
        match self.state {
            SequencerState::WaitArmed { .. }
            | SequencerState::Done
            | SequencerState::Aborted(_) => off,
            SequencerState::Relay { .. } => SequencerOutputs {
                relay_cmd: true,
                ..off
            },
            SequencerState::TestRelay { .. } => SequencerOutputs {
                btest_cmd: true,
                ..off
            },
            SequencerState::Lead { .. } => SequencerOutputs {
                cmd_ha: true,
                ..base
            },
            SequencerState::Phase1 { .. } => SequencerOutputs {
                cmd_ha: true,
                cmd_lb: true,
                ..base
            },
            SequencerState::Gap { us } => SequencerOutputs {
                cmd_ha: us < t.changeover_us,
                cmd_hb: us >= t.changeover_us.saturating_mul(2),
                ..base
            },
            SequencerState::Phase2 { .. } => SequencerOutputs {
                cmd_hb: true,
                cmd_la: true,
                ..base
            },
            SequencerState::LowSideOnly { .. } => SequencerOutputs {
                cmd_lb: true,
                ..base
            },
        }
    }

    /// Advance by `dt_us` with the lines read now.
    pub fn step(&mut self, lines: SequencerLines, dt_us: u32) {
        let t = self.timing;
        // Before anything is committed, a rail that drops (consent
        // withdrawn, permit window over) sends the sequencer back to wait:
        // it commands nothing on a rail that is not up.
        if !self.committed() && !matches!(self.state, SequencerState::Aborted(_)) && !lines.armed {
            self.state = SequencerState::WaitArmed { armed_us: 0 };
            return;
        }
        let plan = self.plan();
        self.state = match self.state {
            SequencerState::WaitArmed { armed_us } => {
                let armed_us = armed_us.saturating_add(dt_us);
                if armed_us < t.rail_ready_wait_us {
                    SequencerState::WaitArmed { armed_us }
                } else {
                    match self.job {
                        SequencerJob::RailOnly => SequencerState::WaitArmed { armed_us },
                        SequencerJob::Test(_) => SequencerState::TestRelay { us: 0 },
                        _ => SequencerState::Relay {
                            since_cmd_us: 0,
                            left_for_us: None,
                        },
                    }
                }
            }
            SequencerState::Relay {
                since_cmd_us,
                left_for_us,
            } => {
                let since_cmd_us = since_cmd_us.saturating_add(dt_us);
                // Restart the count on any sample at rest (FW-1).
                let left_for_us = if lines.relay_at_rest {
                    if left_for_us.is_some() {
                        Some(0)
                    } else {
                        None
                    }
                } else {
                    Some(left_for_us.map_or(0, |u| u.saturating_add(dt_us)))
                };
                let settled = matches!(left_for_us, Some(u) if u >= t.relay_settle_us);
                match self.job {
                    SequencerJob::RelayMove { hold_us } => {
                        if since_cmd_us >= hold_us {
                            SequencerState::Done
                        } else {
                            SequencerState::Relay {
                                since_cmd_us,
                                left_for_us,
                            }
                        }
                    }
                    SequencerJob::StartWithoutLead if settled => {
                        SequencerState::LowSideOnly { us: 0 }
                    }
                    SequencerJob::DryRun(_) if settled && !lines.reads_empty => {
                        SequencerState::Aborted(SequencerAbort::VoltageNotEmpty)
                    }
                    _ if settled => SequencerState::Lead { us: 0 },
                    _ if since_cmd_us >= t.relay_abort_us => {
                        SequencerState::Aborted(SequencerAbort::RelayNotSettled)
                    }
                    _ => SequencerState::Relay {
                        since_cmd_us,
                        left_for_us,
                    },
                }
            }
            SequencerState::TestRelay { us } => {
                let us = us.saturating_add(dt_us);
                if us >= t.test_relay_wait_us {
                    SequencerState::Lead { us: 0 }
                } else {
                    SequencerState::TestRelay { us }
                }
            }
            SequencerState::Lead { us } => {
                let us = us.saturating_add(dt_us);
                if matches!(self.job, SequencerJob::DryRun(_)) && !lines.reads_empty {
                    SequencerState::Aborted(SequencerAbort::VoltageNotEmpty)
                } else if us >= t.lead_us {
                    SequencerState::Phase1 { us: 0 }
                } else {
                    SequencerState::Lead { us }
                }
            }
            SequencerState::Phase1 { us } => {
                let us = us.saturating_add(dt_us);
                match plan {
                    Some(p) if us < p.phase1.0 => SequencerState::Phase1 { us },
                    _ => SequencerState::Gap { us: 0 },
                }
            }
            SequencerState::Gap { us } => {
                let us = us.saturating_add(dt_us);
                match plan {
                    Some(p) if us < p.interphase.0 => SequencerState::Gap { us },
                    _ => SequencerState::Phase2 { us: 0 },
                }
            }
            SequencerState::Phase2 { us } => {
                let us = us.saturating_add(dt_us);
                match plan {
                    Some(p) if us < p.phase2.0 => SequencerState::Phase2 { us },
                    _ => SequencerState::Done,
                }
            }
            SequencerState::LowSideOnly { us } => {
                let us = us.saturating_add(dt_us);
                if us < 1_000 {
                    SequencerState::LowSideOnly { us }
                } else {
                    SequencerState::Done
                }
            }
            s @ (SequencerState::Done | SequencerState::Aborted(_)) => s,
        };
    }
}
