// SPDX-License-Identifier: GPL-3.0-or-later
//! The self-test sequence.
//!
//! Run at power-on by the rescue sequence, and available for periodic checks.
//! It checks, in order: the battery under load; that the supervisor is
//! answering; that the charger can raise the capacitor to a test voltage in
//! time; that the two controllers' voltage readings agree; that the internal
//! discharge empties the capacitor in time; and that the pads are present and
//! in date.
//!
//! The charge step's time limit is scaled to the test voltage
//! ([`Config::charge_time_limit_for`]): 1 s for the 400 V switch-on test, so
//! a weak charger is reported about a second into the test (finding F-7),
//! and about 11.4 s for a full-energy test at 1,950 V.
//!
//! Per SAFETY.md nothing here tests real hardware yet: it only puts the trait
//! calls in the right order with the right time limits.
//!
//! The sequence does not send heartbeats to the supervisor itself. Whoever
//! runs it must keep sending status messages (with state `SelfTest`) so that
//! the supervisor holds the discharge path open during the charge step;
//! [`crate::rescue::Rescue`] does this.

use crate::config::Config;
use crate::hal::{BatteryStatus, MainHardware, PadCartridge};
use crate::link::SupervisorReport;
use aed_types::{Millis, Volts};

/// Which check failed. Each check has its own code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfTestFailure {
    /// The battery could not carry a realistic load.
    BatteryUnderLoad,
    /// No report arrived from the supervisor in time.
    SupervisorLinkDead,
    /// The supervisor answered but did not hold the discharge path open in
    /// time, so the charger could not run (see
    /// [`crate::interlock::charger_may_run`]).
    SupervisorNotHolding,
    /// The supervisor reported a latched fault.
    SupervisorFaultLatched,
    /// The charger did not reach the test voltage in time.
    ChargeTimeout,
    /// The capacitor went over the overvoltage limit while charging.
    Overvoltage,
    /// The two controllers' voltage readings did not agree in time.
    VoltageReadingsDisagree,
    /// The internal discharge did not empty the capacitor in time.
    DischargeTimeout,
    /// No pads are plugged in. Advisory: the rescue still continues.
    PadsMissing,
    /// The pads are past their use-by date. Advisory: the rescue still continues.
    PadsExpired,
    /// The test was stopped from outside before it finished (for example
    /// the battery failed or the cancel control was found stuck). The
    /// cancel control itself does NOT stop the self-test (decision D-31).
    Aborted,
}

impl SelfTestFailure {
    /// True for problems that are reported but do not stop the device being used.
    #[must_use]
    pub const fn is_advisory(self) -> bool {
        matches!(
            self,
            SelfTestFailure::PadsMissing | SelfTestFailure::PadsExpired
        )
    }
}

/// A passed self-test, with any warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfTestPass {
    /// The battery passed but is low.
    pub battery_low: bool,
}

/// Where the self-test has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfTestProgress {
    /// Still running; call `step` again.
    Running,
    /// Finished with this result. The capacitor has been released to empty
    /// (but is not necessarily confirmed empty if the result is a failure).
    Finished(Result<SelfTestPass, SelfTestFailure>),
}

/// The latest supervisor report and when it arrived (on the main controller's clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkSnapshot {
    /// The report.
    pub report: SupervisorReport,
    /// When it arrived.
    pub received_at: Millis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Battery,
    Link { since: Millis },
    Charge { since: Millis },
    Agree { since: Millis },
    Discharge { since: Millis },
    Pads,
    Finished(Result<SelfTestPass, SelfTestFailure>),
}

/// The self-test sequencer. Non-blocking: call [`SelfTest::step`] repeatedly.
#[derive(Debug, Clone)]
pub struct SelfTest {
    config: Config,
    test_voltage: Volts,
    step: Step,
    battery_low: bool,
}

impl SelfTest {
    /// A self-test that will charge to `test_voltage` (use
    /// [`Config::self_test_voltage`] at power-on, or the full charge voltage for
    /// the periodic full-energy test of decision D-15).
    #[must_use]
    pub const fn new(config: Config, test_voltage: Volts) -> Self {
        SelfTest {
            config,
            test_voltage,
            step: Step::Battery,
            battery_low: false,
        }
    }

    /// How long the charge step may take for this test's voltage, ms.
    #[must_use]
    pub fn charge_time_limit_ms(&self) -> u32 {
        self.config.charge_time_limit_for(self.test_voltage)
    }

    /// The voltage this test charges to.
    #[must_use]
    pub const fn test_voltage(&self) -> Volts {
        self.test_voltage
    }

    /// True while the test wants the capacitor held charged.
    #[must_use]
    pub const fn is_charging(&self) -> bool {
        matches!(self.step, Step::Charge { .. } | Step::Agree { .. })
    }

    /// Stop the test now and release the capacitor to empty.
    pub fn abort(&mut self, hw: &mut impl MainHardware) {
        hw.set_charger(None);
        hw.set_discharge_hold(false);
        self.step = Step::Finished(Err(SelfTestFailure::Aborted));
    }

    /// Advance the test. `link` is the latest supervisor report, if any.
    pub fn step(
        &mut self,
        hw: &mut impl MainHardware,
        link: Option<LinkSnapshot>,
    ) -> SelfTestProgress {
        let now = hw.now();
        let fresh_link =
            link.filter(|l| now.since(l.received_at) <= self.config.heartbeat_timeout_ms);
        let next = match self.step {
            Step::Battery => match hw.battery_under_load() {
                BatteryStatus::Depleted => Step::Finished(Err(SelfTestFailure::BatteryUnderLoad)),
                BatteryStatus::Low => {
                    self.battery_low = true;
                    Step::Link { since: now }
                }
                BatteryStatus::Good => Step::Link { since: now },
            },
            // The charger is only switched on once the supervisor reports it is
            // holding the discharge path open: the charger cannot run while the
            // discharge path is active (interlock rule), so charging earlier
            // would only waste the charge step's time limit.
            Step::Link { since } => match fresh_link {
                Some(l) if l.report.fault.is_some() => {
                    Step::Finished(Err(SelfTestFailure::SupervisorFaultLatched))
                }
                Some(l) if l.report.holding => Step::Charge { since: now },
                Some(_) if now.since(since) > self.config.self_test_link_wait_ms => {
                    Step::Finished(Err(SelfTestFailure::SupervisorNotHolding))
                }
                None if now.since(since) > self.config.self_test_link_wait_ms => {
                    Step::Finished(Err(SelfTestFailure::SupervisorLinkDead))
                }
                Some(_) | None => Step::Link { since },
            },
            Step::Charge { since } => {
                let v = hw.capacitor_voltage();
                if v > self.config.overvoltage_limit {
                    Step::Finished(Err(SelfTestFailure::Overvoltage))
                } else if self.config.reached_plan(v, self.test_voltage) {
                    Step::Agree { since: now }
                } else if now.since(since) > self.charge_time_limit_ms() {
                    Step::Finished(Err(SelfTestFailure::ChargeTimeout))
                } else {
                    Step::Charge { since }
                }
            }
            Step::Agree { since } => {
                let v = hw.capacitor_voltage();
                let agrees = fresh_link.is_some_and(|l| {
                    l.received_at >= since
                        && self.config.readings_agree(l.report.capacitor_voltage, v)
                });
                if v > self.config.overvoltage_limit {
                    Step::Finished(Err(SelfTestFailure::Overvoltage))
                } else if agrees {
                    Step::Discharge { since: now }
                } else if now.since(since) > self.config.self_test_agree_wait_ms {
                    Step::Finished(Err(SelfTestFailure::VoltageReadingsDisagree))
                } else {
                    Step::Agree { since }
                }
            }
            Step::Discharge { since } => {
                let v = hw.capacitor_voltage();
                if self.config.reads_empty(v) {
                    Step::Pads
                } else if now.since(since) > self.config.disarm_time_limit_ms {
                    Step::Finished(Err(SelfTestFailure::DischargeTimeout))
                } else {
                    Step::Discharge { since }
                }
            }
            Step::Pads => match hw.pad_cartridge() {
                PadCartridge::Ok => Step::Finished(Ok(SelfTestPass {
                    battery_low: self.battery_low,
                })),
                PadCartridge::Missing => Step::Finished(Err(SelfTestFailure::PadsMissing)),
                PadCartridge::Expired => Step::Finished(Err(SelfTestFailure::PadsExpired)),
            },
            Step::Finished(result) => Step::Finished(result),
        };
        self.step = next;

        // Drive the outputs for the step we are now in. Hold first, then
        // charge; stop charging first, then release.
        if self.is_charging() {
            hw.set_discharge_hold(true);
            hw.set_charger(Some(self.test_voltage));
        } else {
            hw.set_charger(None);
            hw.set_discharge_hold(false);
        }

        match self.step {
            Step::Finished(result) => SelfTestProgress::Finished(result),
            _ => SelfTestProgress::Running,
        }
    }
}
