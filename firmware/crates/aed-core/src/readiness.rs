// SPDX-License-Identifier: GPL-3.0-or-later
//! The one readiness result that drives the ready indicator: "ready", or
//! "not ready" with every reason that applies.
//!
//! It combines the periodic self-test (decisions D-15, D-49), heat (D-48,
//! D-52) and capacitor wear (D-53). A reason is either *blocking* (the
//! indicator shows "not ready") or *advisory* (recorded and exported, the
//! indicator still shows "ready"). Only "capacitor service soon" is
//! advisory.
//!
//! **"Not ready" never stops a rescue** (D-50): a device switched on in an
//! emergency runs its own switch-on self-test, and that decides whether it
//! may shock. Heat is the one exception the owner decided (D-51): a device
//! that is too hot now guides CPR and analyses but does not charge until it
//! has cooled.

/// One reason the device is not ready (or, for advisories, needs attention).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// No daily check or no full-energy test has passed since the timer
    /// started counting.
    SelfTestNotYetRun,
    /// A periodic self-test has not passed for its interval plus a day.
    SelfTestOverdue,
    /// The latest periodic self-test failed.
    SelfTestFailed,
    /// The capacitor could not be confirmed empty after a test (the alarm
    /// sounds and automatic tests stop until service).
    CapacitorNotConfirmedEmpty,
    /// No pads plugged in.
    PadsMissing,
    /// Pads past their use-by date.
    PadsExpired,
    /// The pads have spent longer above 50 °C than their allowance (D-52).
    PadsHeatWorn,
    /// The device was above 70 °C: new pads recommended until a new pad set
    /// is fitted.
    PadsReplaceAfterOverheat,
    /// The last test found the battery low.
    BatteryLow,
    /// Above 60 °C now.
    TooHotNow,
    /// Was above 60 °C: waiting to be below 45 °C for 30 minutes and then
    /// pass a self-test.
    WasTooHot,
    /// More than 8 hours at a stretch above 50 °C: as [`Reason::WasTooHot`].
    WarmSpellTooLong,
    /// A gap in the temperature record: the temperature in between is
    /// unknown. Cleared by a passed self-test.
    HeatRecordGap,
    /// A temperature sensor cannot be trusted (missing, out of range, stuck,
    /// or disagreeing with the others), or nothing is known.
    TempSensorFault,
    /// A reading above 70 °C was seen. Latched until a service action.
    ServiceNeededOverheat,
    /// The durable record (heat history, pad heat total, capacitor wear) was
    /// lost. Until a service action.
    RecordLost,
    /// The capacitor has used 90 % of its full-charge life (advisory).
    CapacitorServiceSoon,
    /// The capacitor has reached its full-charge life (D-53). Blocking for
    /// the indicator, but it never stops the device shocking.
    CapacitorServiceDue,
}

impl Reason {
    /// Every reason, in a fixed order (the bit order of [`ReasonSet`]).
    pub const ALL: [Reason; 18] = [
        Reason::SelfTestNotYetRun,
        Reason::SelfTestOverdue,
        Reason::SelfTestFailed,
        Reason::CapacitorNotConfirmedEmpty,
        Reason::PadsMissing,
        Reason::PadsExpired,
        Reason::PadsHeatWorn,
        Reason::PadsReplaceAfterOverheat,
        Reason::BatteryLow,
        Reason::TooHotNow,
        Reason::WasTooHot,
        Reason::WarmSpellTooLong,
        Reason::HeatRecordGap,
        Reason::TempSensorFault,
        Reason::ServiceNeededOverheat,
        Reason::RecordLost,
        Reason::CapacitorServiceSoon,
        Reason::CapacitorServiceDue,
    ];

    const fn bit(self) -> u32 {
        1 << (self as u32)
    }

    /// True for a reason that does not make the device "not ready".
    #[must_use]
    pub const fn is_advisory(self) -> bool {
        matches!(self, Reason::CapacitorServiceSoon)
    }

    /// Short name used in the exported record (stable: part of the format).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Reason::SelfTestNotYetRun => "self-test-not-yet-run",
            Reason::SelfTestOverdue => "self-test-overdue",
            Reason::SelfTestFailed => "self-test-failed",
            Reason::CapacitorNotConfirmedEmpty => "capacitor-not-confirmed-empty",
            Reason::PadsMissing => "pads-missing",
            Reason::PadsExpired => "pads-expired",
            Reason::PadsHeatWorn => "pads-heat-worn",
            Reason::PadsReplaceAfterOverheat => "pads-replace-after-overheat",
            Reason::BatteryLow => "battery-low",
            Reason::TooHotNow => "too-hot-now",
            Reason::WasTooHot => "was-too-hot",
            Reason::WarmSpellTooLong => "warm-spell-too-long",
            Reason::HeatRecordGap => "heat-record-gap",
            Reason::TempSensorFault => "temp-sensor-fault",
            Reason::ServiceNeededOverheat => "service-needed-overheat",
            Reason::RecordLost => "record-lost",
            Reason::CapacitorServiceSoon => "capacitor-service-soon",
            Reason::CapacitorServiceDue => "capacitor-service-due",
        }
    }
}

/// A set of reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReasonSet(u32);

impl ReasonSet {
    /// No reasons.
    pub const EMPTY: ReasonSet = ReasonSet(0);

    /// Add a reason.
    pub fn insert(&mut self, r: Reason) {
        self.0 |= r.bit();
    }

    /// Add every reason in `other`.
    pub fn extend(&mut self, other: ReasonSet) {
        self.0 |= other.0;
    }

    /// True if `r` is in the set.
    #[must_use]
    pub const fn contains(self, r: Reason) -> bool {
        self.0 & r.bit() != 0
    }

    /// True if the set is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The raw bits (bit n is `Reason::ALL[n]`).
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// The reasons, in [`Reason::ALL`] order.
    pub fn iter(self) -> impl Iterator<Item = Reason> {
        Reason::ALL.into_iter().filter(move |r| self.contains(*r))
    }
}

/// What the ready indicator should show, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeviceReadiness {
    /// Every reason that applies (blocking and advisory).
    pub reasons: ReasonSet,
}

impl DeviceReadiness {
    /// True when no blocking reason applies: the indicator shows "ready".
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.reasons.iter().all(Reason::is_advisory)
    }
}
