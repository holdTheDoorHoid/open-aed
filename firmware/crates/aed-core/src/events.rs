// SPDX-License-Identifier: GPL-3.0-or-later
//! The event record: a fixed-size list of what happened and when, for the
//! hand-over to paramedics and for review afterwards.
//!
//! It never grows: when it is full, the oldest entry is overwritten and an
//! overflow counter goes up, so a reviewer can see that entries were lost.

use crate::hal::{SequenceEnd, SequenceStatus};
use crate::heat::{DeciC, HeatInhibit, SensorFault, TempPoint};
use crate::link::{ShockCheck, SupervisorFault};
use crate::readiness::ReasonSet;
use crate::rescue::{AbortReason, FaultKind, RescueState, ShockOutcome};
use crate::schedule::Readiness;
use crate::selftest::SelfTestFailure;
use crate::supervisor::{
    PermitRefusal, PermitWithdrawal, ReleaseReason, TempIgnored, TestPermitRefusal,
};
use aed_types::{
    ChestState, ImpedanceSignal, Millis, Ohms, PatientMode, PlanError, RhythmDecision, ShockPlan,
    Volts,
};

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
    /// No decision at all arrived within the analysis time limit. No shock; CPR.
    AnalysisTimedOut,
    /// The analysis time limit ran out with only "not sure" answers (usually
    /// movement or interference): the analysis was inconclusive. No shock; CPR.
    AnalysisInconclusive,
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
    /// The cancel control was pressed during the switch-on self-test and
    /// ignored, as decision D-31 requires.
    CancelIgnoredDuringSelfTest,
    /// The cancel control has read pressed without a break for
    /// [`crate::Config::cancel_stuck_ms`] and counts as stuck (D-31).
    CancelStuck,
    /// The capacitor started charging while the analysis runs (D-29). The
    /// matching [`Event::ChargeStarted`] gives the voltage.
    ChargingDuringAnalysis,
    /// Emptying the capacitor started (in the background: the device does
    /// not wait for it before its next instruction). [`Event::CapacitorEmptied`]
    /// confirms it, or the serious fault follows.
    EmptyingStarted,
    /// A charge was emptied without a shock: the energy is burned off in
    /// the internal load (the battery and heat cost of D-29).
    ChargeBurnedOff,
    /// The pads are on but read above the shock range (D-32), at this
    /// resistance.
    PadContactPoor(Ohms),
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
    /// Compression sensing (D-36): while analysing, the chest stopped being
    /// still (this state). The rhythm analysis was reset and fed nothing
    /// until hands were off again.
    AnalysisHeld(ChestState),
    /// Compression sensing confirmed the chest still (good signal, hands
    /// off), so the rhythm analysis started a fresh window on its
    /// already-running filters, without the full settling second (D-47).
    FreshAnalysisWindow,
    /// The compression-sensing channel has had this problem, with the pads
    /// on, for longer than [`crate::Config::chest_channel_fail_ms`]: the
    /// device now works from the heart signal alone, as without
    /// compression sensing.
    ChestSensingFailed(ImpedanceSignal),
    /// The compression-sensing channel has been good again for
    /// [`crate::Config::chest_channel_recover_ms`]: it is used again.
    ChestSensingRestored,
    /// During CPR, compressions restarted after a pause of this many ms
    /// (pauses of at least [`crate::Config::pause_record_ms`] only).
    CprPause(u32),
    /// End of a CPR period: what compression sensing measured during it.
    CprPeriodSummary {
        /// Compressions detected.
        compressions: u16,
        /// Average rate over compressions given in runs, per minute (0 if none).
        rate_per_min: u16,
        /// Share of the period spent compressing, tenths of a percent.
        on_chest_permille: u16,
        /// Longest pause between compressions, ms.
        longest_pause_ms: u32,
    },
    /// "Continue CPR." was said because no compressions were sensed for
    /// [`crate::Config::no_compression_prompt_ms`].
    NoCompressionsPrompt,
    /// At switch-on: what the periodic self-tests (D-15) said before this
    /// use, and the capacitor's wear count. Recorded only; it does not stop
    /// the rescue (see [`crate::schedule`]).
    StandbyRecord {
        /// The ready indicator's state before switch-on.
        readiness: Readiness,
        /// Full-energy charges by periodic tests so far.
        full_energy_cycles: u32,
        /// Charges during earlier rescues so far.
        rescue_charges: u32,
    },
    /// At switch-on: the one readiness result from standby (D-48, D-52,
    /// D-53), the capacitor's lifetime full charges and the pad heat total.
    /// Recorded only; the rescue goes ahead (D-50).
    StandbyReadiness {
        /// Every reason that applied before switch-on.
        reasons: ReasonSet,
        /// Full charges over the capacitor's life.
        capacitor_charges: u32,
        /// Seconds the pads have spent above 50 °C.
        pad_hot_s: u32,
    },
    /// Capacitor charging is held back for temperature (D-41, D-48, D-51).
    HeatChargeInhibit(HeatInhibit),
    /// Capacitor charging is allowed again (the device has cooled).
    HeatChargeAllowed,
    /// The switch-on self-test skipped its charge steps because the device
    /// was too hot to charge (D-51).
    SelfTestChargeSkippedForHeat,
    /// An analysis started without charging during it, because the device
    /// is warmer than the pre-charge limit (55 °C): it charges only if a
    /// shock is advised.
    PrechargeSkippedWarm,
    /// A shock was advised but charging is held back for temperature: no
    /// shock now, CPR, analyse again next cycle (D-51).
    ShockHeldForHeat,
    /// A temperature reading is not trusted (recorded once per point per rescue).
    TemperatureSensorFault(TempPoint, SensorFault),
    /// No temperature reading at all could be trusted: heat cannot hold the
    /// shock back (a sensor failure must not block every shock).
    TemperaturesUnknown,
    /// The supervisor holds charging back: its own reading is over its limit.
    SupervisorTooHot(DeciC),
    /// The supervisor is not acting on its own temperature reading.
    SupervisorTempIgnored(TempIgnored),

    // ----- the shock sequence (D-59) ---------------------------------------
    /// The main controller found the sequence logic not idle at the moment
    /// it was about to request a shock (a lockout that did not clear, or a
    /// consent line stuck on). No request was raised.
    SequenceNotIdleAtRequest(SequenceStatus),
    /// The main controller saw the hardware rail come up (pads moving to
    /// the bridge).
    SequenceArmed,
    /// The main controller saw the sequence latched (current flowing).
    SequenceStarted,
    /// The main controller saw the sequence end, in this way.
    SequenceEnded(SequenceEnd),
    /// A sequence started although the main controller had just withdrawn
    /// its request (the withdrawal and the hardware's latch raced): it
    /// completed by design and is recorded with how it ended.
    SequenceCompletedAfterWithdrawal(SequenceEnd),
    /// The patient relay is back on the front ends after a sequence; pad and
    /// heart readings are ignored for the recovery time from here.
    RelayReturned,
    /// The front-end recovery time after a sequence is over; pad, heart and
    /// impedance readings are used again (analysis and compression sensing
    /// restart from scratch).
    FrontEndsRecovered,
    /// After the recovery time the pads still did not read as on (they
    /// came off, or a front end did not recover). The ordinary pad-contact
    /// rule takes over ("Attach pads").
    PadsNotBackAfterShock,
    /// The supervisor's own verdict on the last sequence, as reported to the
    /// main controller.
    SupervisorShockCheckReported(ShockCheck),
    /// The adult/child switch's two contacts have disagreed for
    /// [`crate::Config::mode_contacts_disagree_ms`] (D-60). The primary
    /// contact still sets the mode; the device shows "not ready".
    ModeContactsDisagree {
        /// The primary contact (used).
        primary: PatientMode,
        /// The contact that sets the hardware voltage ceiling.
        ceiling: PatientMode,
    },
    /// The adult/child switch's two contacts agree again.
    ModeContactsAgree,
    /// The supervisor took back its permit before the sequence started.
    SupervisorPermitWithdrawn(PermitWithdrawal),
    /// The supervisor's permit window passed with no sequence started.
    SupervisorPermitUnused,
    /// The supervisor saw the sequence it permitted start.
    SupervisorSequenceStarted,
    /// The supervisor saw the sequence end, in this way.
    SupervisorSequenceEnded(SequenceEnd),
    /// The supervisor's verdict on the sequence from its own reading.
    SupervisorShockCheck(ShockCheck),
    /// The supervisor gave the self-test's test permit (D-61).
    SupervisorTestPermitted,
    /// A test permit ran its full window while still asked for.
    SupervisorTestPermitExpired,
    /// The supervisor refused a test permit.
    SupervisorTestPermitRefused(TestPermitRefusal),
    /// FW-18: the charger stopped rising below the plan with no
    /// `CHG_DONE_N`; it was switched off and on again (reading then).
    ChargerRestarted(Volts),
    /// The charger ended a charge by itself (`CHG_DONE_N`): its last-resort
    /// over-voltage trip. Emptied, not restarted (reading then).
    ChargerStoppedItself(Volts),
    /// The supervisor gave the REAL permit for a self-test step (D-62).
    SupervisorRealTestPermitted,
    /// The supervisor refused or withdrew a real self-test permit (D-62):
    /// a reading not empty, the charger on, or (dry run) its own pad
    /// reading not open.
    SupervisorRealTestPermitRefused,
    /// The over-voltage latch test (D-62 T-6) passed.
    SupervisorOvpTestPassed,
    /// The hardware over-voltage latch was reset after a real trip, once
    /// both readings were empty (FW-17).
    SupervisorOvpReset,
}

/// How many compressions the compression record keeps (the newest).
pub const COMPRESSION_LOG_CAPACITY: usize = 512;

/// One detected compression, for the compression record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionEntry {
    /// When it was detected.
    pub at: Millis,
    /// Its size, milliohms peak to peak after filtering (capped at 65,535).
    pub amplitude_mohm: u16,
}

/// The compression record: one entry per detected compression (D-14),
/// kept apart from the main event record so that compressions (about 220
/// per CPR period) can never push safety events out of it. Fixed size; the
/// oldest entry is overwritten and counted.
#[derive(Debug, Clone)]
pub struct CompressionLog<const N: usize> {
    slots: [Option<CompressionEntry>; N],
    next: usize,
    len: usize,
    overwritten: u32,
}

impl<const N: usize> Default for CompressionLog<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> CompressionLog<N> {
    /// An empty record.
    #[must_use]
    pub const fn new() -> Self {
        CompressionLog {
            slots: [None; N],
            next: 0,
            len: 0,
            overwritten: 0,
        }
    }

    /// Add one compression.
    pub fn push(&mut self, entry: CompressionEntry) {
        let Some(slot) = self.slots.get_mut(self.next) else {
            self.overwritten = self.overwritten.saturating_add(1);
            return;
        };
        *slot = Some(entry);
        self.next = self.next.saturating_add(1).checked_rem(N).unwrap_or(0);
        if self.len < N {
            self.len = self.len.saturating_add(1);
        } else {
            self.overwritten = self.overwritten.saturating_add(1);
        }
    }

    /// Entries held.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True if no entries are held.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Entries lost to overwriting.
    #[must_use]
    pub const fn overwritten(&self) -> u32 {
        self.overwritten
    }

    /// The newest entry, if any.
    #[must_use]
    pub fn newest(&self) -> Option<CompressionEntry> {
        if self.len == 0 {
            return None;
        }
        let index = self.next.checked_add(N.saturating_sub(1))?.checked_rem(N)?;
        self.slots.get(index).copied().flatten()
    }
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
