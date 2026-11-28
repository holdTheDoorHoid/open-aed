// SPDX-License-Identifier: GPL-3.0-or-later
//! The rescue sequence: the main controller's state machine.
//!
//! This follows the diagram in ARCHITECTURE.md, "The rescue sequence", with
//! charging during analysis (owner decision D-29):
//!
//! ```text
//! SelfTest -> AttachPads -> AnalysingCharging --(shock, charged)--------> Countdown -> Delivering -> Cpr
//!                              |        \--(shock, still charging)-> Charging -/
//!                              +-> Cpr (no shock / not sure to the limit / cancel)
//!                              +-> AttachPads (pads off)
//! Cpr -> AnalysingCharging after the CPR period; Cpr -> AttachPads if pads come off.
//! Charging / Countdown / Delivering -> Cpr if the shock is abandoned
//!                                      (-> AttachPads if the pads came off).
//! Any fault: -> Fault ("continue CPR").
//! ```
//!
//! (With [`Config::charge_during_analysis`] switched off the device uses
//! `Analysing` instead of `AnalysingCharging` and charges only after a shock
//! is advised, as in the first design.)
//!
//! **Emptying runs in the background.** Whenever the device leaves a state
//! in which it holds the capacitor charged (and on any fault), it lets go of
//! the discharge path, which empties the capacitor into the internal load,
//! and goes at once to its next state and instruction ("No shock advised.
//! Start CPR.", "Shock delivered. Start CPR."). It does not wait for the
//! capacitor to empty first: a responder is never left waiting on the
//! device. Meanwhile it checks, every tick, that its own voltage reading
//! (and the supervisor's) shows empty within [`Config::disarm_time_limit_ms`].
//! Until it does, the device holds nothing and starts no new charge. If its
//! own reading is not empty by the limit, that is the most serious fault
//! ("Serious device fault. Do not touch the device. Continue CPR."), from
//! whatever state the device has got to. The capacitor empties by itself
//! when let go (hardware rule, discharge by default), so not waiting does
//! not leave it charged; it only changes when the responder is told.
//!
//! The rules it keeps (each has a test that names it):
//!
//! * Only a `Shockable` decision can lead to a countdown and a shock.
//!   `NonShockable` ends the analysis with "No shock advised" and CPR. "Not
//!   sure" (`Indeterminate`) does not end it: the device repeats "Do not
//!   touch the patient" and keeps analysing until it gets `Shockable` or
//!   `NonShockable`, or the analysis time limit runs out ("inconclusive":
//!   no shock, CPR).
//! * The capacitor charges while the analysis runs (D-29), but only after
//!   the self-test has passed, only once the supervisor reports that it is
//!   holding the discharge path, and only once any earlier charge is
//!   confirmed empty. A charge that is not used is burned off in the
//!   internal load and counted ([`Counters::charges_burned`]).
//! * Analysis keeps running while charging and counting down. After a shock
//!   is advised, any decision other than `Shockable`, pads coming off,
//!   cancel, or a change of the adult/child switch abandons the shock: the
//!   capacitor is emptied and the device goes to CPR. (A switch change
//!   during the analysis itself empties the capacitor and recharges for the
//!   new mode without stopping the analysis.)
//! * At the moment of delivery the latest decision must be `Shockable` and no
//!   older than [`Config::confirmation_max_age_ms`], and no other decision may
//!   have arrived since the shock was advised.
//! * The shock plan is made from a fresh pad-resistance reading taken just
//!   before delivery. If no plan can be made, no shock. Pads that are on but
//!   read above the shock range (175 to 250 Ω, D-32) are analysed as normal
//!   and the responder is asked to press them firmly; at delivery, if they
//!   still read above the shock range: "Poor pad contact. No shock." and CPR.
//! * The shock is requested once, only with the hardware's sequence logic
//!   idle (otherwise a consent line is stuck or a lockout never cleared: a
//!   fault). **Consent starts the shock; it need not last through it
//!   (D-59).** Pad resistance, the plan and the "still shockable"
//!   confirmation are checked at the request. From the request on, pad,
//!   heart and impedance readings are not believed (the relay is about to
//!   take the pads off the front ends), and the device follows the
//!   hardware's sequence status with a time limit for each stage: the
//!   permit and the rail ([`Config::delivery_timeout_ms`]), the relay and
//!   the start ([`Config::sequence_start_timeout_ms`]), the end
//!   ([`Config::sequence_end_timeout_ms`]). Before the start it still
//!   withdraws for cancel, over-voltage, the supervisor's heat limit, a
//!   mode change and, while the rail is not yet up, sensed chest movement
//!   (D-37); a stale link or a supervisor fault withdraws it through the
//!   fault path. Once the hardware reports the sequence active it does not
//!   try. Each outcome has its own event ([`ShockOutcome`]): delivered;
//!   not delivered, refused by the interlock, timed out, relay did not
//!   transfer, not started (all "Shock not delivered", CPR); time cap
//!   tripped, over-current tripped, implausible residual, no end (all
//!   faults). Afterwards the rhythm analysis is reset, the remaining charge
//!   is emptied and checked, and the device goes to CPR.
//! * After any request, in the background: the relay must be back on the
//!   front ends within [`Config::relay_return_timeout_ms`] (otherwise a
//!   fault), then readings are ignored for [`Config::front_end_recovery_ms`]
//!   while the front ends recover; pad contact is neither lost nor regained
//!   meanwhile, and afterwards the rhythm analysis and compression sensing
//!   start from scratch. Pads that still read off then lead to the usual
//!   "Attach pads" (recorded). A sequence that latched just as the request
//!   was withdrawn completes by design and is recorded and counted.
//! * The adult/child switch's two contacts are compared all the time
//!   (D-60). If they disagree for [`Config::mode_contacts_disagree_ms`] the
//!   device records it, shows "not ready" (ready indicator off) and carries
//!   on with the mode from the primary contact: the hardware ceiling falls
//!   back to the adult value, so shocks in both modes are still possible,
//!   and the child dose is still set by the main controller and checked by
//!   the supervisor. Nothing is said aloud: the responder cannot fix it.
//! * Cancel works in every state except the switch-on self-test, where it
//!   is ignored (D-31). A cancel that reads pressed without a break for
//!   [`Config::cancel_stuck_ms`] is a stuck-button fault with its own prompt.
//! * Any fault empties the capacitor and ends in the Fault state, whose prompt
//!   tells the responder to continue CPR. If the capacitor cannot be
//!   confirmed empty, that is its own, more serious fault.
//! * Every state except AttachPads and Fault has a time limit.
//!
//! **Compression sensing** (decision D-36, [`Config::chest_sensing`]): when a
//! [`ChestSensor`] is fitted and switched on,
//!
//! * the rhythm analysis is fed only while the chest is still (`HandsOff`).
//!   The moment it is not (`Compressing`, or `Unknown`: cannot tell), the
//!   analysis is reset and fed nothing until hands are off again, so every
//!   decision comes from a window of still-chest signal (apart from at most
//!   the sensor's onset latency, about 0.3 s typically). "Not sure" is still
//!   not final, but the "Do not touch the patient" reminder now follows the
//!   chest state, not the "not sure" answers. The analysis time limit is
//!   unchanged: still nothing clear by then is "inconclusive", CPR.
//! * once a shock is advised (charging, counting down, delivering), a chest
//!   that is not still holds the shock back at once: "Movement detected.
//!   Shock delayed.", the charge is emptied and the rhythm analysed again
//!   from the start, at most [`Config::hands_on_reanalyses`] times per cycle;
//!   after that "Movement detected. No shock." and CPR.
//! * a channel that stays missing, saturated, out of range or stuck (with
//!   the pads on) for [`Config::chest_channel_fail_ms`] is recorded as failed
//!   and the device carries on from the heart signal alone until it recovers.
//! * during CPR, compressions, pauses and time on chest are recorded, and
//!   "Continue CPR." is said after [`Config::no_compression_prompt_ms`]
//!   without compressions.
//!
//! **Heat** (decisions D-41, D-48, D-51; see [`crate::heat`]). Every tick
//! the main controller reads its own temperature sensors (and the
//! supervisor's reading from its report):
//!
//! * The capacitor is charged only with the capacitor and both cells at or
//!   below 60 °C. Too hot at switch-on: the self-test checks everything but
//!   the charge ("Device too hot. Shocks delayed until it cools."); every
//!   analysis runs without charging; a "shock" answer gives "Shock advised.
//!   Device too hot to charge." and "Start CPR.", and the device analyses
//!   again after the CPR period. Once it has cooled 1 °C below the limit it
//!   charges during the analysis and shocks as usual. It never charges
//!   during CPR (see the README for why).
//! * Above 55 °C it does not charge during the analysis (D-29) but only once
//!   a shock is advised, so that "no shock" analyses stop burning charges
//!   inside a hot device.
//! * The limit is checked when a charge would start and throughout an
//!   analysis; once a shock is advised the charge carries on to the shock,
//!   and only the supervisor's own backstop (65 °C) stops it ("Device too
//!   hot. No shock.", CPR).
//! * A sensor that cannot be trusted is replaced by the hottest trusted
//!   reading plus 5 °C; if none can be trusted, heat does not hold the shock
//!   back (recorded): a sensor failure must not block every shock.

use crate::config::{Config, ConfigError};
use crate::events::{
    CompressionEntry, CompressionLog, Event, EventRecord, COMPRESSION_LOG_CAPACITY, EVENT_CAPACITY,
};
use crate::hal::{BatteryStatus, MainHardware, RelayPosition, SequenceEnd, SequenceStatus};
use crate::heat::{self, ChargeContext, ChargeGate, DeciC, TempPoint, TEMP_POINTS};
use crate::link::{MainStatus, ShockCheck, SupervisorFault, SupervisorReport, TestRequest};
use crate::prompts::Prompt;
use crate::selftest::{LinkSnapshot, SelfTest, SelfTestFailure, SelfTestProgress};
use aed_types::{
    pads_sensed, ChestEvent, ChestSensor, ChestState, EcgSample, ImpedanceSample, ImpedanceSignal,
    Millis, NoChestSensor, PatientMode, PlanError, RhythmAnalyzer, RhythmDecision, ShockPlan,
    ShockPlanner, Volts, MAX_PATIENT_OHMS,
};

/// The states of the rescue sequence (ARCHITECTURE.md diagram).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RescueState {
    /// Checking the device after switch-on.
    SelfTest,
    /// Waiting for the pads to be on a patient.
    AttachPads,
    /// Analysing the heart rhythm, without charging; nobody may touch the
    /// patient. Used only when [`Config::charge_during_analysis`] is off.
    Analysing,
    /// Analysing the heart rhythm while the capacitor charges, or is held
    /// charged, in case a shock is advised (decision D-29); nobody may touch
    /// the patient. No shock can be given from this state: a shock needs the
    /// countdown first, and the supervisor gives no permit here.
    AnalysingCharging,
    /// A shock is advised; waiting for the capacitor to finish charging.
    Charging,
    /// Charged; counting down aloud before the automatic shock.
    Countdown,
    /// The shock has been requested; following the hardware's sequence
    /// status until it reports the sequence ended (D-59).
    Delivering,
    /// CPR period, with the compression metronome running.
    Cpr,
    /// The device has a fault and will not shock. Prompts continue CPR.
    Fault,
}

impl RescueState {
    /// True for the two analysing states.
    #[must_use]
    pub const fn is_analysing(self) -> bool {
        matches!(
            self,
            RescueState::Analysing | RescueState::AnalysingCharging
        )
    }

    /// True for the states in which the main controller holds the
    /// capacitor charged (or charging): charging during analysis, charging,
    /// counting down and delivering.
    #[must_use]
    pub const fn holds_charge(self) -> bool {
        matches!(
            self,
            RescueState::AnalysingCharging
                | RescueState::Charging
                | RescueState::Countdown
                | RescueState::Delivering
        )
    }
}

/// Why a shock being prepared was abandoned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortReason {
    /// The cancel control was pressed.
    Cancelled,
    /// The analysis gave a decision other than `Shockable`.
    RhythmChanged,
    /// Pad contact was lost.
    PadsOff,
    /// The adult/child switch was moved.
    ModeChanged,
    /// The planner could not make a plan for the measured pad resistance.
    /// `ImpedanceTooHigh` with the pads still on (175 to 250 Ω) is the
    /// "poor pad contact" case of D-32.
    PlanRefused(PlanError),
    /// No recent enough `Shockable` decision at the moment of delivery.
    ConfirmationStale,
    /// Compression sensing (D-36) said the chest was being compressed, or
    /// could not tell, after the shock was advised.
    HandsOnChest,
    /// The supervisor's own temperature reading went over its limit while
    /// the shock was being prepared (D-41, D-48).
    TooHot,
}

/// What happened to a requested shock (D-59). Each has its own event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockOutcome {
    /// The hardware reported the sequence completed and the main
    /// controller's own reading afterwards fell into the delivered range.
    Delivered,
    /// The sequence completed but the capacitor hardly emptied: no energy
    /// reached anyone (for example the pads came off at the last moment).
    /// "Shock not delivered", CPR.
    NotDelivered,
    /// The interlock did not let a sequence start although the supervisor
    /// had permitted: the rail never rose, a consent dropped before the
    /// first conducting moment, or the hardware's permit-duration cap ran
    /// out. "Shock not delivered", CPR.
    RefusedByInterlock,
    /// Nothing happened within the delivery time limit and no permit was
    /// seen from the supervisor. "Shock not delivered", CPR.
    TimedOut,
    /// The rail rose but the patient relay never reached the bridge within
    /// the start time limit. "Shock not delivered", CPR.
    RelayDidNotTransfer,
    /// The rail rose and the relay reached the bridge, but no sequence
    /// started within the start time limit. "Shock not delivered", CPR.
    NotStarted,
    /// The hardware time cap ended the sequence: the phase timing did not
    /// end it. A fault.
    TimeCapTripped,
    /// The over-current trip ended the sequence. A fault.
    OverCurrentTripped,
    /// A per-phase hardware cap ended the sequence (D-63). A fault.
    PhaseCapTripped,
    /// Another hardware fault latch ended it (idle current, bridge supply,
    /// `LSD_OK`; v0.3). A fault.
    HardwareFault,
    /// The sequence completed but the capacitor fell further than any plan
    /// takes it, or stopped part-way. A fault.
    ResidualImplausible,
    /// The hardware showed the sequence running for longer than the end
    /// time limit (its own time cap should have ended it). A fault.
    DidNotEnd,
    /// The request was withdrawn before the sequence started, for this
    /// reason.
    Withdrawn(AbortReason),
}

/// A fault in the main controller's view. Every fault ends in the Fault state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    /// The self-test failed at this check.
    SelfTest(SelfTestFailure),
    /// The capacitor did not charge within the limit.
    ChargeTimeout,
    /// The main controller read a voltage over the limit.
    Overvoltage,
    /// The capacitor voltage drifted away from the planned voltage (up or
    /// down) after it had charged.
    ChargeOffTarget,
    /// The capacitor was armed for longer than the limit.
    ArmedTooLong,
    /// The output stage did not deliver the shock as planned: its time cap
    /// or over-current trip ended it, the capacitor fell to a level no
    /// plan explains, or it never reported an end (see the
    /// [`ShockOutcome`] recorded with it).
    DeliveryFault,
    /// The patient relay did not return the pads to the front ends after a
    /// sequence: nothing can be analysed (D-59).
    PatientRelayStuck,
    /// The sequence logic was not idle when a shock was about to be
    /// requested: a consent line stuck on, or a lockout that never cleared.
    InterlockNotIdle,
    /// The planner's voltage does not fit the limits or does not match the charge.
    PlanMismatch,
    /// The supervisor latched this fault.
    Supervisor(SupervisorFault),
    /// The supervisor stopped reporting.
    SupervisorLinkLost,
    /// The battery cannot power a rescue.
    BatteryDepleted,
    /// The cancel control read pressed for longer than
    /// [`Config::cancel_stuck_ms`] (D-31). It would block every shock.
    CancelStuck,
    /// The capacitor could not be confirmed empty within the limit.
    /// The most serious fault: it has its own prompt.
    CapacitorNotEmptied,
    /// The charger ended a charge by itself (`CHG_DONE_N`): its
    /// last-resort over-voltage trip fired. The capacitor is emptied and
    /// the charger is never restarted (coordinator addition to FW-18).
    ChargerStoppedItself,
}

impl FaultKind {
    /// True for the fault where the capacitor may still be charged.
    #[must_use]
    pub const fn is_serious(self) -> bool {
        matches!(self, FaultKind::CapacitorNotEmptied)
    }
}

/// Running totals for the event summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counters {
    /// Shocks the hardware reported delivered.
    pub shocks_delivered: u16,
    /// Shocks advised but abandoned or not delivered.
    pub shocks_not_given: u16,
    /// Charges emptied into the internal load without a shock (every
    /// analysis that charges and ends without a shock burns one: the
    /// battery and heat cost of D-29).
    pub charges_burned: u16,
    /// Analyses started.
    pub analyses: u16,
    /// CPR periods started.
    pub cpr_periods: u16,
    /// Metronome beats given.
    pub metronome_beats: u32,
    /// Compressions detected by compression sensing (all states).
    pub compressions: u32,
    /// Pauses in compressions during CPR of at least
    /// [`Config::pause_record_ms`].
    pub cpr_pauses: u16,
    /// Times an analysis was reset because the chest stopped being still.
    pub analyses_held: u16,
    /// Shocks held back because the chest was not still after the shock
    /// was advised.
    pub hands_on_aborts: u16,
    /// "Continue CPR." prompts for lack of compressions.
    pub continue_cpr_prompts: u16,
    /// Analysis windows started on already-settled filters because
    /// compression sensing confirmed the chest still (D-47).
    pub fresh_windows: u16,
    /// Charges of the capacitor towards a shock voltage started (one per
    /// arming): the rescue's capacitor wear (D-53).
    pub capacitor_charges: u16,
    /// "Shock" answers that could not be acted on because the device was
    /// too hot to charge (D-51).
    pub shocks_held_for_heat: u16,
}

/// What compression sensing measured during the current CPR period.
#[derive(Debug, Clone, Copy, Default)]
struct CprPeriod {
    compressions: u16,
    interval_sum_ms: u32,
    intervals: u16,
    on_chest_ms: u32,
    longest_pause_ms: u32,
}

/// Debounced on/off condition (pad contact, poor pad contact).
#[derive(Debug, Clone, Copy)]
struct Debounced {
    on: bool,
    change_since: Option<Millis>,
}

impl Debounced {
    const fn off() -> Self {
        Debounced {
            on: false,
            change_since: None,
        }
    }

    /// Returns `Some(new_state)` when the raw condition has changed for long enough.
    fn update(&mut self, raw: bool, now: Millis, debounce_ms: u32) -> Option<bool> {
        if raw == self.on {
            self.change_since = None;
            return None;
        }
        let since = *self.change_since.get_or_insert(now);
        if now.since(since) >= debounce_ms {
            self.on = raw;
            self.change_since = None;
            Some(raw)
        } else {
            None
        }
    }

    /// Take a fresh reading's word for it at once (used when the reading
    /// taken for a shock plan shows the pads off before the smoothing has
    /// caught up).
    fn force(&mut self, on: bool) {
        self.on = on;
        self.change_since = None;
    }
}

/// How far the hardware sequence for the current request has got (on the
/// main controller's clock).
#[derive(Debug, Clone, Copy)]
struct Delivery {
    requested_at: Millis,
    permit_seen: bool,
    armed_at: Option<Millis>,
    started_at: Option<Millis>,
    ended: Option<(Millis, SequenceEnd)>,
}

/// Watching the hardware after the device has left `Delivering`, in the
/// background, whatever state it is in: a sequence that raced a withdrawal,
/// the relay returning, and the front ends recovering.
#[derive(Debug, Clone, Copy)]
struct SequenceWatch {
    /// When the watch's clock last restarted (leaving Delivering, or the
    /// end of a sequence that started after a withdrawal).
    since: Millis,
    /// The relay was (or may have been) moved off the front ends.
    relay_moved: bool,
    /// A sequence ran.
    ran: bool,
    /// Its end has been recorded.
    end_recorded: bool,
    /// When the relay was seen back on the front ends.
    relay_back_at: Option<Millis>,
    /// The relay fault has been raised.
    relay_fault: bool,
}

/// What ended an armed state early.
#[derive(Debug, Clone, Copy)]
enum ArmedExit {
    Abort(AbortReason),
    Fault(FaultKind),
}

/// The firmware's own charge control while armed (design v0.3).
///
/// * **Top-up only while needed (FW-13).** Compression readings are thrown
///   away while the charger runs (D-67), so the charger is commanded on only
///   until the charge has settled at the charger's own set point (the
///   reading in the at-plan window and no longer rising), and again as soon
///   as the reading sags below where it stopped (a short top-up burst).
///   Otherwise the chest check would be blind for the whole analysis and
///   countdown, not just the 2–6 s of the charge D-67 describes.
/// * **Silent stop (FW-18).** With the charger on and the reading below the
///   plan, no rise for [`Config::charger_stall_ms`] (after the
///   [`Config::charger_start_ms`] start time) means the charger stopped
///   without saying so: it is switched off for
///   [`Config::charger_restart_off_ms`] and on again.
/// * **Stopped by its own limit.** If the charger reports `CHG_DONE_N`
///   ([`MainHardware::charger_done`]) the restart above must never fire:
///   the charger ended the charge by its last-resort over-voltage trip, and
///   restarting would refill the capacitor to that level again. The two are
///   told apart by that line alone; see firmware/REV3_NOTES.md.
#[derive(Debug, Clone, Copy)]
struct ChargeControl {
    topping_up: bool,
    on_since: Option<Millis>,
    last_rise: Option<(Millis, Volts)>,
    off_until: Option<Millis>,
    restarts: u16,
    stopped_at: Option<Volts>,
}

/// After a blanking gap the compression detector has this long to give a
/// definite state again before "cannot tell" counts, ms.
const CHEST_REACQUIRE_MS: u32 = 3_000;

/// A blanking gap longer than this restarts the compression detector, ms.
const CHEST_GAP_RESET_MS: u32 = 500;

/// The charge counts as done once the reading is in the at-plan window and
/// has not risen by 2 V for this long, ms.
const CHARGE_SETTLED_MS: u32 = 100;

/// What [`ChargeControl::decide`] says to do this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChargeAction {
    On,
    Off,
    Restart,
    StoppedItself,
}

impl ChargeControl {
    const fn new() -> Self {
        ChargeControl {
            topping_up: true,
            on_since: None,
            last_rise: None,
            off_until: None,
            restarts: 0,
            stopped_at: None,
        }
    }

    fn decide(
        &mut self,
        c: &Config,
        target: Volts,
        v: Volts,
        done: bool,
        now: Millis,
    ) -> ChargeAction {
        if done {
            return ChargeAction::StoppedItself;
        }
        if let Some(until) = self.off_until {
            if now < until {
                return ChargeAction::Off;
            }
            self.off_until = None;
            self.on_since = None;
        }
        // Top up again as soon as the reading has sagged below where the
        // charger stopped (1 V, the reading's resolution): the at-plan
        // window was sized for sensor error alone and leaves no room for
        // droop (see firmware/REV3_NOTES.md §3.2).
        if let Some(stopped) = self.stopped_at {
            if v < stopped {
                self.topping_up = true;
                self.stopped_at = None;
            }
        }
        if !self.topping_up {
            self.on_since = None;
            self.last_rise = None;
            return ChargeAction::Off;
        }
        let since = *self.on_since.get_or_insert(now);
        let rise = match self.last_rise {
            Some((t, v0)) if v.0 < v0.0.saturating_add(2) => (t, v0),
            _ => (now, v),
        };
        self.last_rise = Some(rise);
        let still_for = now.since(rise.0);
        let started = now.since(since) >= c.charger_start_ms;
        if c.reached_plan(v, target) {
            // The charger regulates to its own set point (its accuracy is
            // the charger's, not this sensor's): once the reading is in the
            // at-plan window and has stopped rising, the charge is done and
            // the charger is switched off (FW-13).
            if started && still_for >= CHARGE_SETTLED_MS {
                self.topping_up = false;
                self.stopped_at = Some(v);
                self.on_since = None;
                self.last_rise = None;
                return ChargeAction::Off;
            }
            return ChargeAction::On;
        }
        // FW-18: below the window, on, no rise: a silent stop. (Never with
        // the reading in the window: a charger that has reached its set
        // point with a sensor reading low is not stalled.)
        if started && still_for >= c.charger_stall_ms {
            self.restarts = self.restarts.saturating_add(1);
            self.off_until = Some(Millis(now.0.saturating_add(c.charger_restart_off_ms)));
            self.last_rise = None;
            return ChargeAction::Restart;
        }
        ChargeAction::On
    }
}

/// The main controller's rescue sequence.
///
/// Non-blocking: call [`Rescue::tick`] frequently (every few milliseconds),
/// [`Rescue::on_ecg_sample`] for every heart-signal sample and, if the device
/// senses compressions, [`Rescue::on_impedance_sample`] for every
/// pad-impedance sample.
#[derive(Debug)]
pub struct Rescue<A: RhythmAnalyzer, P: ShockPlanner, C: ChestSensor = NoChestSensor> {
    config: Config,
    analyzer: A,
    planner: P,
    chest: C,
    chest_last_sample: Option<Millis>,
    chest_seen: Option<ChestState>,
    chest_failed: bool,
    chest_problem_since: Option<Millis>,
    chest_good_since: Option<Millis>,
    hands_on_reanalyses_used: u8,
    continue_cpr_said_at: Option<Millis>,
    cpr_period: CprPeriod,
    compression_log: CompressionLog<COMPRESSION_LOG_CAPACITY>,
    started: bool,
    powered_on_at: Millis,
    state: RescueState,
    state_since: Millis,
    state_changed: bool,
    self_test: SelfTest,
    self_test_result: Option<Result<(), SelfTestFailure>>,
    pads: Debounced,
    pads_raw_on: bool,
    poor_contact: Debounced,
    cancel_was_pressed: bool,
    cancel_since: Option<Millis>,
    mode: PatientMode,
    mode_known: bool,
    mode_at_arming: PatientMode,
    battery_low_announced: bool,
    analysis_result: Option<RhythmDecision>,
    not_sure_seen: bool,
    hands_off_reminded_at: Option<Millis>,
    press_pads_reminded_at: Millis,
    last_decision: Option<(RhythmDecision, Millis)>,
    veto: bool,
    target: Volts,
    armed_since: Option<Millis>,
    charged_at: Option<Millis>,
    charger_was_on: bool,
    /// The charger is commanded on now (FW-13 blanking, FW-16 report).
    charger_on: bool,
    /// When it was last commanded off.
    charger_off_at: Option<Millis>,
    /// Firmware charge control (FW-13 top-up, FW-18 restart).
    charge_ctl: ChargeControl,
    /// When the coil supply was raised in this arming (FW-12).
    coil_supply_since: Option<Millis>,
    /// When the current compression-blanking gap began, if it has.
    chest_blank_since: Option<Millis>,
    /// The compression detector is restarted on the first sample after a
    /// blanking gap (its filters would read the step across the gap as
    /// movement).
    chest_resume_reset: bool,
    /// When the current blanking gap began, as the impedance samples saw it.
    chest_gap_since: Option<Millis>,
    /// The last impedance sample that arrived (and was thrown away) during
    /// a blanking gap: the channel is alive.
    chest_gap_alive_at: Option<Millis>,
    /// The detector has given a definite state since the last gap.
    chest_seen_since_gap: bool,
    shocked_this_arming: bool,
    emptying_since: Option<Millis>,
    plan: Option<ShockPlan>,
    delivery: Option<Delivery>,
    watch: Option<SequenceWatch>,
    mode_contacts_since: Option<Millis>,
    mode_contacts_bad: bool,
    last_test_request: TestRequest,
    fault: Option<FaultKind>,
    countdown_spoken: u8,
    next_beat: Millis,
    last_reminder: Millis,
    counters: Counters,
    link: Option<LinkSnapshot>,
    link_required: bool,
    status_sequence: u32,
    last_status_at: Option<Millis>,
    events: EventRecord<EVENT_CAPACITY>,
    heat_gate: ChargeGate,
    precharge_ok: bool,
    temp_faults_seen: [bool; TEMP_POINTS],
    temps_unknown_seen: bool,
    heat_announced: bool,
    main_temp_claim: Option<DeciC>,
}

impl<A: RhythmAnalyzer, P: ShockPlanner> Rescue<A, P, NoChestSensor> {
    /// A new rescue sequence without compression sensing, before switch-on.
    /// The first `tick` starts the self-test.
    ///
    /// # Errors
    /// Refuses a configuration whose numbers contradict each other.
    pub fn new(config: Config, analyzer: A, planner: P) -> Result<Self, ConfigError> {
        Self::with_chest_sensor(config, analyzer, planner, NoChestSensor)
    }
}

impl<A: RhythmAnalyzer, P: ShockPlanner, C: ChestSensor> Rescue<A, P, C> {
    /// A new rescue sequence with compression sensing (decision D-36),
    /// before switch-on. The first `tick` starts the self-test.
    ///
    /// # Errors
    /// Refuses a configuration whose numbers contradict each other.
    pub fn with_chest_sensor(
        config: Config,
        analyzer: A,
        planner: P,
        chest: C,
    ) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Rescue {
            config,
            analyzer,
            planner,
            chest,
            chest_last_sample: None,
            chest_seen: None,
            chest_failed: false,
            chest_problem_since: None,
            chest_good_since: None,
            hands_on_reanalyses_used: 0,
            continue_cpr_said_at: None,
            cpr_period: CprPeriod::default(),
            compression_log: CompressionLog::new(),
            started: false,
            powered_on_at: Millis(0),
            state: RescueState::SelfTest,
            state_since: Millis(0),
            state_changed: true,
            self_test: SelfTest::new(config, config.self_test_voltage),
            self_test_result: None,
            pads: Debounced::off(),
            pads_raw_on: false,
            poor_contact: Debounced::off(),
            cancel_was_pressed: false,
            cancel_since: None,
            mode: PatientMode::Adult,
            mode_known: false,
            mode_at_arming: PatientMode::Adult,
            battery_low_announced: false,
            analysis_result: None,
            not_sure_seen: false,
            hands_off_reminded_at: None,
            press_pads_reminded_at: Millis(0),
            last_decision: None,
            veto: false,
            target: Volts(0),
            armed_since: None,
            charged_at: None,
            charger_was_on: false,
            charger_on: false,
            charger_off_at: None,
            charge_ctl: ChargeControl::new(),
            coil_supply_since: None,
            chest_blank_since: None,
            chest_resume_reset: false,
            chest_gap_since: None,
            chest_gap_alive_at: None,
            chest_seen_since_gap: true,
            shocked_this_arming: false,
            emptying_since: None,
            plan: None,
            delivery: None,
            watch: None,
            mode_contacts_since: None,
            mode_contacts_bad: false,
            last_test_request: TestRequest::None,
            fault: None,
            countdown_spoken: 0,
            next_beat: Millis(0),
            last_reminder: Millis(0),
            counters: Counters::default(),
            link: None,
            link_required: false,
            status_sequence: 0,
            last_status_at: None,
            events: EventRecord::new(),
            heat_gate: ChargeGate::default(),
            precharge_ok: true,
            temp_faults_seen: [false; TEMP_POINTS],
            temps_unknown_seen: false,
            heat_announced: false,
            main_temp_claim: None,
        })
    }

    /// Why capacitor charging is held back for temperature now, by the main
    /// controller's own judgement (the supervisor's reading included), if it is.
    #[must_use]
    pub const fn heat_inhibit(&self) -> Option<heat::HeatInhibit> {
        self.heat_gate.inhibit()
    }

    /// True if the switch-on self-test skipped its charge steps for heat
    /// (report it with [`crate::Standby::record_power_on`]).
    #[must_use]
    pub const fn self_test_charge_skipped(&self) -> bool {
        self.self_test.charge_skipped()
    }

    /// Write the standby readiness (D-48, D-52, D-53) into the event record
    /// at switch-on. Recorded only: the rescue goes ahead (D-50).
    pub fn record_standby_readiness(
        &mut self,
        now: Millis,
        readiness: crate::readiness::DeviceReadiness,
        capacitor_charges: u32,
        pad_hot_s: u32,
    ) {
        self.events.push(
            now,
            Event::StandbyReadiness {
                reasons: readiness.reasons,
                capacitor_charges,
                pad_hot_s,
            },
        );
    }

    /// True while the supervisor reports it is holding charging back for heat.
    fn supervisor_too_hot(&self, now: Millis) -> bool {
        self.link.is_some_and(|l| {
            l.report.too_hot && now.since(l.received_at) <= self.config.heartbeat_timeout_ms
        })
    }

    /// True while capacitor charging must wait for the device to cool.
    fn charge_inhibited(&self, now: Millis) -> bool {
        self.heat_gate.inhibit().is_some() || self.supervisor_too_hot(now)
    }

    /// Read the temperatures and update the charging limits (see the module notes).
    fn update_heat(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let mut set = hw.temperatures();
        let sup = self
            .link
            .filter(|l| now.since(l.received_at) <= self.config.heartbeat_timeout_ms)
            .and_then(|l| l.report.temperature);
        set.set(TempPoint::Supervisor, sup);
        let a = heat::assess(&self.config, &set, &[false; TEMP_POINTS]);
        for (p, seen) in TempPoint::ALL.iter().zip(self.temp_faults_seen.iter_mut()) {
            // The supervisor records its own sensor's problems.
            if *p == TempPoint::Supervisor {
                continue;
            }
            if let Some(f) = a.fault_at(*p) {
                if !*seen {
                    *seen = true;
                    self.events.push(now, Event::TemperatureSensorFault(*p, f));
                }
            }
        }
        if a.hottest().is_none() && !self.temps_unknown_seen {
            self.temps_unknown_seen = true;
            self.events.push(now, Event::TemperaturesUnknown);
        }
        let was = self.heat_gate.inhibit();
        let inhibit = self
            .heat_gate
            .update(&self.config, &a, ChargeContext::Rescue);
        if inhibit != was {
            self.events.push(
                now,
                inhibit.map_or(Event::HeatChargeAllowed, Event::HeatChargeInhibit),
            );
        }
        self.precharge_ok = heat::precharge_allowed(&self.config, &a, self.precharge_ok);
        self.main_temp_claim = [TempPoint::Cell1, TempPoint::Cell2, TempPoint::Capacitor]
            .iter()
            .filter_map(|p| a.valid_at(*p))
            .max();
    }

    /// The device became too hot to charge during an analysis that charges
    /// (D-29): let the charge go and carry on analysing without one. The
    /// analysis and its time limit are not restarted.
    fn drop_charge_keep_analysing(&mut self, hw: &mut impl MainHardware, now: Millis) {
        self.burn_charge_if_any(now);
        self.armed_since = None;
        self.charged_at = None;
        self.start_emptying(hw, now);
        self.events.push(
            now,
            Event::StateChanged {
                from: self.state,
                to: RescueState::Analysing,
            },
        );
        self.state = RescueState::Analysing;
        self.state_changed = true;
    }

    /// The current state.
    #[must_use]
    pub const fn state(&self) -> RescueState {
        self.state
    }

    /// The fault that put the device in the Fault state, if any.
    #[must_use]
    pub const fn fault(&self) -> Option<FaultKind> {
        self.fault
    }

    /// The power-on self-test result, once it has finished.
    #[must_use]
    pub const fn self_test_result(&self) -> Option<Result<(), SelfTestFailure>> {
        self.self_test_result
    }

    /// Running totals (shocks, burned charges, analyses, CPR periods, metronome beats).
    #[must_use]
    pub const fn counters(&self) -> Counters {
        self.counters
    }

    /// The event record.
    #[must_use]
    pub const fn events(&self) -> &EventRecord<EVENT_CAPACITY> {
        &self.events
    }

    /// The configuration in use.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// The rhythm analyser (for inspection by tests and the simulator).
    #[must_use]
    pub const fn analyzer(&self) -> &A {
        &self.analyzer
    }

    /// The latest rhythm decision and when it arrived.
    #[must_use]
    pub const fn last_decision(&self) -> Option<(RhythmDecision, Millis)> {
        self.last_decision
    }

    /// The plan for the most recent shock request, if any.
    #[must_use]
    pub const fn last_plan(&self) -> Option<ShockPlan> {
        self.plan
    }

    /// The adult/child setting last read from the switch.
    #[must_use]
    pub const fn patient_mode(&self) -> PatientMode {
        self.mode
    }

    /// True while the adult/child switch's two contacts disagree (D-60):
    /// the device shows "not ready" (and still shocks, with the mode from
    /// the primary contact).
    #[must_use]
    pub const fn mode_contacts_disagree(&self) -> bool {
        self.mode_contacts_bad
    }

    /// True while pad, heart and impedance readings are not believed: from
    /// a shock request until the relay is back on the front ends and the
    /// recovery time is over (D-59).
    #[must_use]
    pub fn front_ends_blind(&self, now: Millis) -> bool {
        self.state == RescueState::Delivering
            || self.watch.is_some_and(|w| {
                w.relay_moved
                    && w.relay_back_at
                        .is_none_or(|t| now.since(t) < self.config.front_end_recovery_ms)
            })
    }

    /// True while the impedance channel is disconnected or recovering:
    /// once the rail is up in `Delivering`, and in the watch afterwards.
    fn impedance_blind(&self, now: Millis) -> bool {
        (self.state == RescueState::Delivering
            && self.delivery.is_some_and(|d| d.armed_at.is_some()))
            || (self.state != RescueState::Delivering && self.front_ends_blind(now))
    }

    /// True while the capacitor is being emptied in the background and is
    /// not yet confirmed empty. No new charge starts until it is.
    #[must_use]
    pub const fn emptying(&self) -> bool {
        self.emptying_since.is_some()
    }

    /// Milliseconds since switch-on.
    #[must_use]
    pub fn elapsed_ms(&self, now: Millis) -> u32 {
        now.since(self.powered_on_at)
    }

    /// Milliseconds spent in the current state.
    #[must_use]
    pub fn time_in_state_ms(&self, now: Millis) -> u32 {
        now.since(self.state_since)
    }

    /// The compression sensor (for inspection by tests and the simulator).
    #[must_use]
    pub const fn chest_sensor(&self) -> &C {
        &self.chest
    }

    /// The compression record (one entry per detected compression).
    #[must_use]
    pub const fn compression_log(&self) -> &CompressionLog<COMPRESSION_LOG_CAPACITY> {
        &self.compression_log
    }

    /// Write the periodic self-test record (D-15) into the event record, at
    /// switch-on. The board passes what [`crate::TestSchedule`] reports.
    /// Recorded only: a device that was "not ready" still runs the rescue
    /// (its own switch-on self-test decides whether it can shock).
    pub fn record_standby(
        &mut self,
        now: Millis,
        readiness: crate::Readiness,
        counters: crate::TestCounters,
    ) {
        self.events.push(
            now,
            Event::StandbyRecord {
                readiness,
                full_energy_cycles: counters.full_energy_cycles,
                rescue_charges: counters.rescue_charges,
            },
        );
    }

    /// True if compression sensing is fitted, switched on and has not failed:
    /// the device is using it now.
    #[must_use]
    pub fn chest_sensing_active(&self) -> bool {
        self.config.chest_sensing && self.chest.fitted() && !self.chest_failed
    }

    /// The chest state the device acts on at `now`: `None` when compression
    /// sensing is not in use (heart signal only), `Unknown` if impedance
    /// samples have stopped arriving.
    #[must_use]
    pub fn chest_state(&self, now: Millis) -> Option<ChestState> {
        if !self.chest_sensing_active() || self.chest_blanked(now) {
            return None;
        }
        // After a blanking gap, until the compression detector gives a
        // definite state again (at most 3 s), the chest counts as "not
        // sensed" rather than "cannot tell": otherwise the end of every
        // charge would reset the analysis.
        if self.chest_reacquiring(now) {
            return None;
        }
        Some(if self.chest_stale(now) {
            ChestState::Unknown
        } else {
            self.chest.state()
        })
    }

    fn chest_stale(&self, now: Millis) -> bool {
        self.chest_last_sample
            .is_none_or(|t| now.since(t) > self.config.impedance_stale_ms)
    }

    /// Whether the compression-sensing signal is usable at `now`.
    #[must_use]
    pub fn chest_signal(&self, now: Millis) -> ImpedanceSignal {
        if self.chest_stale(now) {
            ImpedanceSignal::Missing
        } else {
            self.chest.signal()
        }
    }

    /// True while a shock decision may rely on the chest being still: the
    /// heart signal is analysed only then. Always true without compression
    /// sensing.
    fn chest_still(&self, now: Millis) -> bool {
        self.chest_state(now)
            .is_none_or(|s| s == ChestState::HandsOff)
    }

    /// Feed one pad-impedance sample (compression sensing, D-36), at
    /// [`aed_types::IMPEDANCE_SAMPLE_RATE_HZ`]. Ignored if the device has no
    /// compression sensing or it is switched off.
    pub fn on_impedance_sample(&mut self, sample: ImpedanceSample, now: Millis) {
        if !(self.config.chest_sensing && self.chest.fitted()) {
            return;
        }
        if self.impedance_blind(now) {
            // The pads are on the bridge, or the front end is recovering
            // from a sequence: the samples say nothing about the chest.
            return;
        }
        if self.chest_blanked(now) {
            // FW-13 (D-67): the charger's switching swamps the 64 kHz
            // measurement. Thrown away, and the time is not counted against
            // the channel (see `check_chest_channel`), since samples still
            // arrive.
            self.chest_gap_since.get_or_insert(now);
            self.chest_gap_alive_at = Some(now);
            self.chest_seen_since_gap = false;
            return;
        }
        if let Some(g) = self.chest_gap_since.take() {
            // After a long gap (a charge, not a short top-up) the detector's
            // filters would read the step across it as movement: restart it.
            if now.since(g) > CHEST_GAP_RESET_MS {
                self.chest_resume_reset = true;
            }
        }
        if self.chest_resume_reset {
            self.chest_resume_reset = false;
            self.chest.reset();
        }
        let event = self.chest.push(sample);
        self.chest_last_sample = Some(now);
        if self.chest.state() != ChestState::Unknown {
            self.chest_seen_since_gap = true;
        }
        if let Some(ChestEvent::Compression {
            amplitude_mohm,
            since_previous_ms,
        }) = event
        {
            self.on_compression(now, amplitude_mohm, since_previous_ms);
        }
        if self.state == RescueState::Cpr && self.chest.state() == ChestState::Compressing {
            let ms = 1000 / aed_types::IMPEDANCE_SAMPLE_RATE_HZ;
            self.cpr_period.on_chest_ms = self.cpr_period.on_chest_ms.saturating_add(ms);
        }
        self.refresh_chest(now);
    }

    fn on_compression(&mut self, now: Millis, amplitude_mohm: u32, since_previous_ms: u32) {
        self.counters.compressions = self.counters.compressions.saturating_add(1);
        self.compression_log.push(CompressionEntry {
            at: now,
            amplitude_mohm: u16::try_from(amplitude_mohm).unwrap_or(u16::MAX),
        });
        if self.state != RescueState::Cpr {
            return;
        }
        let p = &mut self.cpr_period;
        p.compressions = p.compressions.saturating_add(1);
        if since_previous_ms == u32::MAX {
            return;
        }
        let since_cpr_start = now.since(self.state_since);
        // A pause counts from the later of the last compression and the
        // start of this CPR period.
        let pause = since_previous_ms.min(since_cpr_start);
        if since_previous_ms <= 1_500 {
            p.interval_sum_ms = p.interval_sum_ms.saturating_add(since_previous_ms);
            p.intervals = p.intervals.saturating_add(1);
        }
        p.longest_pause_ms = p.longest_pause_ms.max(pause);
        if pause >= self.config.pause_record_ms {
            self.counters.cpr_pauses = self.counters.cpr_pauses.saturating_add(1);
            self.events.push(now, Event::CprPause(pause));
        }
    }

    /// React to a change in the chest state the device acts on.
    fn refresh_chest(&mut self, now: Millis) {
        let current = self.chest_state(now);
        if current == self.chest_seen {
            return;
        }
        let was_still = self.chest_seen.is_none_or(|s| s == ChestState::HandsOff);
        self.chest_seen = current;
        let still = current.is_none_or(|s| s == ChestState::HandsOff);
        if !was_still && still && self.state.is_analysing() {
            // Hands are off again. The analysis was reset when they went on
            // and has been fed nothing since (only the filters have run).
            // With the chest confirmed still, start the window now on the
            // settled filters (D-47); otherwise (sensing failed or switched
            // off meanwhile, or the signal is not good) start from a full
            // reset, exactly as before D-47.
            self.start_analysis_window(now);
        }
        if was_still && !still && self.state.is_analysing() {
            // Everything the analysis has seen may include the start of the
            // movement: forget it, and feed nothing until hands are off.
            self.analyzer.reset();
            self.analysis_result = None;
            self.last_decision = None;
            self.counters.analyses_held = self.counters.analyses_held.saturating_add(1);
            if let Some(s) = current {
                self.events.push(now, Event::AnalysisHeld(s));
            }
        }
    }

    /// True when compression sensing confirms the chest is still: sensing in
    /// use, the impedance signal good, and the state `HandsOff`. Only then
    /// may an analysis window start on filters that are already running,
    /// without the full settling second (decision D-47).
    fn chest_confirmed_still(&self, now: Millis) -> bool {
        self.chest_state(now) == Some(ChestState::HandsOff)
            && self.chest_signal(now) == ImpedanceSignal::Good
    }

    /// Start a fresh analysis window: on the running filters if the chest
    /// is confirmed still (D-47, recorded), otherwise with a full reset of
    /// the analysis (the behaviour before D-47).
    fn start_analysis_window(&mut self, now: Millis) {
        if self.chest_confirmed_still(now) {
            self.analyzer.restart_window();
            self.counters.fresh_windows = self.counters.fresh_windows.saturating_add(1);
            self.events.push(now, Event::FreshAnalysisWindow);
        } else {
            self.analyzer.reset();
        }
    }

    /// Watch for a failed compression-sensing channel, and for its recovery.
    fn check_chest_channel(&mut self, now: Millis) {
        if !(self.config.chest_sensing && self.chest.fitted()) {
            return;
        }
        // FW-13: a blanking gap is not a failed channel. The failure timer
        // is paused across it (its start moves on by the gap's length) and
        // the staleness clock restarts when the gap ends.
        if self.chest_blanked(now) {
            self.chest_blank_since.get_or_insert(now);
            return;
        }
        if let Some(b) = self.chest_blank_since.take() {
            // Only if samples kept arriving: a channel that delivers nothing
            // has failed whether or not the charger runs.
            let alive = self.chest_gap_alive_at.is_some_and(|t| t >= b);
            if alive {
                let gap = now.since(b);
                if let Some(p) = self.chest_problem_since {
                    self.chest_problem_since = Some(Millis(p.0.saturating_add(gap)));
                }
                self.chest_last_sample = Some(now);
            }
        }
        let signal = self.chest_signal(now);
        let problem = self.pads.on && signal.is_channel_problem();
        if problem {
            self.chest_good_since = None;
            let since = *self.chest_problem_since.get_or_insert(now);
            if !self.chest_failed && now.since(since) >= self.config.chest_channel_fail_ms {
                self.chest_failed = true;
                self.events.push(now, Event::ChestSensingFailed(signal));
            }
        } else {
            self.chest_problem_since = None;
            if signal == ImpedanceSignal::Good {
                let since = *self.chest_good_since.get_or_insert(now);
                if self.chest_failed && now.since(since) >= self.config.chest_channel_recover_ms {
                    self.chest_failed = false;
                    self.events.push(now, Event::ChestSensingRestored);
                }
            } else {
                self.chest_good_since = None;
            }
        }
        self.refresh_chest(now);
    }

    /// Feed one heart-signal sample. Samples are passed to the rhythm analysis
    /// only while analysing, charging, counting down or delivering; during CPR
    /// the signal is full of compression movement and is not analysed. With
    /// compression sensing they are passed on only while the chest is still.
    pub fn on_ecg_sample(&mut self, sample: EcgSample, now: Millis) {
        if self.front_ends_blind(now) {
            // From the shock request until the front ends have recovered,
            // the heart signal is disconnected or disturbed: nothing is
            // analysed or tracked. The decision that led to the request was
            // already confirmed at the request (D-59).
            return;
        }
        let analysing = self.state.is_analysing()
            || matches!(self.state, RescueState::Charging | RescueState::Countdown);
        if !analysing {
            if self.state == RescueState::Cpr && self.pads.on {
                // Keep the analysis filters running through CPR, so that the
                // next analysis can start without waiting for them to settle
                // (D-47). Nothing is analysed.
                self.analyzer.track(sample);
            }
            return;
        }
        self.refresh_chest(now);
        if !self.chest_still(now) {
            // Held back by compressions: the filters keep running, nothing
            // is analysed.
            self.analyzer.track(sample);
            return;
        }
        let Some(decision) = self.analyzer.push(sample) else {
            return;
        };
        self.events.push(now, Event::Decision(decision));
        self.last_decision = Some((decision, now));
        if self.state.is_analysing() {
            // Answers not yet acted on: "no shock" wins over everything, and
            // "not sure" wins over "shock".
            let pending = self.analysis_result;
            self.analysis_result = Some(match (pending, decision) {
                (Some(RhythmDecision::NonShockable), _) | (_, RhythmDecision::NonShockable) => {
                    RhythmDecision::NonShockable
                }
                (Some(RhythmDecision::Indeterminate), _) | (_, RhythmDecision::Indeterminate) => {
                    RhythmDecision::Indeterminate
                }
                (_, RhythmDecision::Shockable) => RhythmDecision::Shockable,
            });
        } else if decision != RhythmDecision::Shockable {
            self.veto = true;
        }
    }

    /// Run one step. Call frequently.
    pub fn tick(&mut self, hw: &mut impl MainHardware) {
        let now = hw.now();
        if !self.started {
            self.started = true;
            self.powered_on_at = now;
            self.state_since = now;
            self.events.push(now, Event::PowerOn);
            hw.set_ready_indicator(false);
        }

        if let Some(report) = hw.supervisor_report() {
            self.on_report(report, now);
        }
        self.watch_sequence(hw, now);

        // Pad contact is neither lost nor regained while the front ends are
        // disconnected or recovering (D-59): an open reading then says
        // nothing about the pads.
        if !self.front_ends_blind(now) {
            let ohms = hw.pad_resistance();
            match self
                .pads
                .update(pads_sensed(ohms), now, self.config.pad_debounce_ms)
            {
                Some(true) => {
                    self.events.push(now, Event::PadsOn);
                    self.chest.reset();
                }
                Some(false) => self.events.push(now, Event::PadsOff),
                None => {}
            }
            self.pads_raw_on = pads_sensed(ohms);
            let poor_raw = pads_sensed(ohms) && ohms > MAX_PATIENT_OHMS;
            if self
                .poor_contact
                .update(poor_raw, now, self.config.pad_debounce_ms)
                == Some(true)
            {
                self.events.push(now, Event::PadContactPoor(ohms));
            }
        }

        let cancel = self.read_cancel(hw, now);

        let mode = hw.patient_mode();
        if !self.mode_known {
            self.mode_known = true;
            self.mode = mode;
        } else if mode != self.mode {
            self.mode = mode;
            self.events.push(now, Event::PatientModeChanged(mode));
            hw.prompt(mode_prompt(mode));
        }
        self.check_mode_contacts(hw, now, mode);

        match hw.battery() {
            BatteryStatus::Good => self.battery_low_announced = false,
            BatteryStatus::Low => {
                if !self.battery_low_announced {
                    self.battery_low_announced = true;
                    self.events.push(now, Event::BatteryLow);
                    hw.prompt(Prompt::LowBattery);
                }
            }
            BatteryStatus::Depleted => self.begin_fault(FaultKind::BatteryDepleted, hw, now),
        }

        if let Some(f) = self.link.and_then(|l| l.report.fault) {
            self.begin_fault(FaultKind::Supervisor(f), hw, now);
        }
        let link_stale = self
            .link
            .is_none_or(|l| now.since(l.received_at) > self.config.heartbeat_timeout_ms);
        if self.link_required && link_stale {
            self.begin_fault(FaultKind::SupervisorLinkLost, hw, now);
        }

        self.check_chest_channel(now);
        self.update_heat(hw, now);

        let voltage = hw.capacitor_voltage();
        self.check_emptying(hw, now, voltage);

        match self.state {
            RescueState::SelfTest => self.tick_self_test(hw, now),
            RescueState::AttachPads => self.tick_attach_pads(hw, now),
            RescueState::Analysing => self.tick_analysing(hw, now, cancel),
            RescueState::AnalysingCharging => {
                self.tick_analysing_charging(hw, now, cancel, voltage);
            }
            RescueState::Charging => self.tick_charging(hw, now, cancel, voltage),
            RescueState::Countdown => self.tick_countdown(hw, now, cancel, voltage),
            RescueState::Delivering => self.tick_delivering(hw, now, cancel, voltage),
            RescueState::Cpr => self.tick_cpr(hw, now),
            RescueState::Fault => self.tick_fault(hw, now),
        }

        self.apply_outputs(hw);
        self.send_status_if_due(hw, now);
    }

    /// Read the cancel control: record presses, detect a stuck button, and
    /// return whether cancel is in force. During the switch-on self-test a
    /// press is recorded but ignored (D-31).
    fn read_cancel(&mut self, hw: &mut impl MainHardware, now: Millis) -> bool {
        let raw = hw.cancel_pressed();
        if raw && !self.cancel_was_pressed {
            self.events.push(
                now,
                if self.state == RescueState::SelfTest {
                    Event::CancelIgnoredDuringSelfTest
                } else {
                    Event::CancelPressed
                },
            );
        }
        self.cancel_was_pressed = raw;
        if raw {
            let since = *self.cancel_since.get_or_insert(now);
            let stuck = now.since(since) >= self.config.cancel_stuck_ms;
            if stuck
                && self.fault != Some(FaultKind::CancelStuck)
                && self.state != RescueState::Fault
            {
                self.events.push(now, Event::CancelStuck);
                self.begin_fault(FaultKind::CancelStuck, hw, now);
            }
        } else {
            self.cancel_since = None;
        }
        raw && self.state != RescueState::SelfTest
    }

    // ----- background emptying ----------------------------------------------

    /// Let go of the capacitor so it empties into the internal load, and
    /// start checking that it does. The caller moves on at once.
    fn start_emptying(&mut self, hw: &mut impl MainHardware, now: Millis) {
        hw.set_charger(None);
        hw.withdraw_shock_request();
        hw.set_discharge_hold(false);
        if self.emptying_since.is_none() {
            self.emptying_since = Some(now);
            self.events.push(now, Event::EmptyingStarted);
        }
    }

    /// Confirm a background emptying, or raise the serious fault if the
    /// capacitor is not empty in time.
    ///
    /// "Empty" needs the main controller's own reading to show empty and,
    /// so that the supervisor's own "charged too long" timer is also reset
    /// before a new charge starts, a supervisor report sent after emptying
    /// began whose reading shows empty too. If no such report comes by the
    /// time limit (supervisor link problems, which have their own faults),
    /// the main controller's own reading is enough.
    fn check_emptying(&mut self, hw: &mut impl MainHardware, now: Millis, voltage: Volts) {
        let Some(since) = self.emptying_since else {
            return;
        };
        let own_empty = self.config.reads_empty(voltage);
        let supervisor_empty = self.link.is_some_and(|l| {
            l.received_at >= since
                && now.since(l.received_at) <= self.config.heartbeat_timeout_ms
                && self.config.reads_empty(l.report.capacitor_voltage)
        });
        let overdue = now.since(since) > self.config.disarm_time_limit_ms;
        if own_empty && (supervisor_empty || overdue) {
            self.emptying_since = None;
            self.events.push(now, Event::CapacitorEmptied(voltage));
        } else if !own_empty && overdue {
            self.emptying_since = None;
            self.serious_fault(hw, now);
        }
    }

    /// The capacitor could not be confirmed empty: the most serious fault,
    /// from whatever state the device is in.
    fn serious_fault(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let kind = FaultKind::CapacitorNotEmptied;
        self.events.push(now, Event::Fault(kind));
        self.fault = Some(kind);
        if self.state == RescueState::Fault {
            self.last_reminder = now;
            hw.prompt(self.fault_prompt());
        } else {
            if self.state == RescueState::SelfTest {
                self.self_test.abort(hw);
            }
            self.go(RescueState::Fault, hw, now);
        }
        // Already tried and failed: do not start checking again.
        self.emptying_since = None;
    }

    // ----- state handlers -------------------------------------------------

    fn tick_self_test(&mut self, hw: &mut impl MainHardware, now: Millis) {
        // Cancel is ignored here (D-31): a stray press while the device is
        // drawn and switched on must not disable it.
        // Too hot to charge (D-51): test everything but the charge.
        self.self_test.allow_heat_skip();
        if self.charge_inhibited(now) {
            self.self_test.skip_charge_for_heat();
        }
        match self.self_test.step(hw, self.link) {
            SelfTestProgress::Running => {}
            SelfTestProgress::Finished(Ok(pass)) => {
                self.self_test_result = Some(Ok(()));
                self.events.push(now, Event::SelfTestPassed);
                self.self_test_passed(pass.battery_low, hw, now);
            }
            SelfTestProgress::Finished(Err(f)) if f.is_advisory() => {
                self.self_test_result = Some(Err(f));
                self.events.push(now, Event::SelfTestFailed(f));
                if f == SelfTestFailure::PadsExpired {
                    hw.prompt(Prompt::ReplacePads);
                }
                if f == SelfTestFailure::PadsHeatWorn {
                    hw.prompt(Prompt::ReplacePadsHeat);
                }
                self.self_test_passed(false, hw, now);
            }
            SelfTestProgress::Finished(Err(f)) => self.finish_self_test_failed(f, hw, now),
        }
    }

    fn self_test_passed(&mut self, battery_low: bool, hw: &mut impl MainHardware, now: Millis) {
        if battery_low && !self.battery_low_announced {
            self.battery_low_announced = true;
            self.events.push(now, Event::BatteryLow);
            hw.prompt(Prompt::LowBattery);
        }
        self.link_required = true;
        hw.set_ready_indicator(!self.mode_contacts_bad);
        if self.self_test.charge_skipped() {
            self.events.push(now, Event::SelfTestChargeSkippedForHeat);
        }
        if self.charge_inhibited(now) && !self.heat_announced {
            self.heat_announced = true;
            hw.prompt(Prompt::DeviceTooHot);
        }
        if self.mode == PatientMode::Child {
            hw.prompt(Prompt::ChildModeFrontBack);
        }
        self.go(RescueState::AttachPads, hw, now);
    }

    fn finish_self_test_failed(
        &mut self,
        f: SelfTestFailure,
        hw: &mut impl MainHardware,
        now: Millis,
    ) {
        self.self_test_result = Some(Err(f));
        self.events.push(now, Event::SelfTestFailed(f));
        if f == SelfTestFailure::DischargeTimeout {
            // Emptying has already been tried and failed: straight to the serious fault.
            self.serious_fault(hw, now);
        } else {
            self.begin_fault(FaultKind::SelfTest(f), hw, now);
        }
    }

    fn tick_attach_pads(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if self.pads.on {
            self.enter_analysis(hw, now);
        } else if now.since(self.last_reminder) >= self.config.attach_pads_repeat_ms {
            self.last_reminder = now;
            hw.prompt(Prompt::AttachPads);
        }
    }

    /// Start an analysis: with charging during analysis (D-29), set up the
    /// charge first, then enter the analysing state.
    fn enter_analysis(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let hot = self.charge_inhibited(now);
        if self.config.charge_during_analysis && !hot && !self.precharge_ok {
            // Warm (D-29 with heat): charge only if a shock is advised.
            self.events.push(now, Event::PrechargeSkippedWarm);
        }
        if self.config.charge_during_analysis && !hot && self.precharge_ok {
            if !self.begin_arming(hw, now) {
                return;
            }
            self.events.push(now, Event::ChargingDuringAnalysis);
            self.go(RescueState::AnalysingCharging, hw, now);
        } else {
            self.go(RescueState::Analysing, hw, now);
        }
    }

    /// Set the charge target for the current mode and start a new arming.
    /// Returns false (after starting the fault path) if the planner's charge
    /// voltage does not fit the limits.
    fn begin_arming(&mut self, hw: &mut impl MainHardware, now: Millis) -> bool {
        let target = self.planner.charge_voltage(self.mode);
        // The whole "at the planned voltage" window must lie above "empty"
        // and below the overvoltage limit.
        let window = self.config.at_plan_allowance(target).0;
        let lowest_sensible = self.config.empty_voltage.0.saturating_add(window);
        let highest_sensible = self.config.overvoltage_limit.0.saturating_sub(window);
        if target.0 <= lowest_sensible || target.0 >= highest_sensible {
            self.begin_fault(FaultKind::PlanMismatch, hw, now);
            return false;
        }
        self.target = target;
        self.armed_since = Some(now);
        self.charged_at = None;
        self.charger_was_on = false;
        self.charge_ctl = ChargeControl::new();
        self.shocked_this_arming = false;
        self.mode_at_arming = self.mode;
        self.veto = false;
        self.plan = None;
        self.events.push(now, Event::ChargeStarted(target));
        true
    }

    /// Analysis decisions shared by both analysing states. Returns true if
    /// the state was left.
    fn analysis_step(&mut self, hw: &mut impl MainHardware, now: Millis) -> bool {
        match self.analysis_result.take() {
            Some(RhythmDecision::Shockable) if self.chest_unseen_after_charge(now) => {
                // FW-13 / D-67: the analysis ran on the heart signal alone
                // while the charger blanked the compression readings. A
                // "shock" answer then does not count until the compression
                // sensing has looked at the chest again: compressions seen
                // reset the analysis (as always); with hands off, the NEXT
                // "shock" answer is acted on. Waiting for the next answer,
                // rather than holding this one, keeps the countdown in step
                // with the analysis, so that its last answer falls at the end
                // of the countdown (finding F-4). This keeps F-12's
                // protection for the advice itself.
            }
            Some(RhythmDecision::Shockable) => {
                self.shock_advised(hw, now);
                return true;
            }
            Some(RhythmDecision::NonShockable) => {
                hw.prompt(Prompt::NoShockAdvised);
                self.go(RescueState::Cpr, hw, now);
                return true;
            }
            Some(RhythmDecision::Indeterminate) => {
                // Not final: movement is the likely cause. Without
                // compression sensing, ask again for hands off (not too
                // often) and keep analysing. With it, the reminder follows
                // the chest state instead (below).
                self.not_sure_seen = true;
                if self.chest_state(now).is_none() {
                    self.remind_hands_off(hw, now);
                }
            }
            None => {}
        }
        if self
            .chest_state(now)
            .is_some_and(|s| s != ChestState::HandsOff)
        {
            self.remind_hands_off(hw, now);
        }
        if self.time_in_state_ms(now) > self.config.analysis_time_limit_ms {
            // Neither "shock" nor "no shock" in time. Never leave the
            // responder waiting: say so and go to CPR (which says "Start CPR").
            self.events.push(
                now,
                if self.not_sure_seen {
                    Event::AnalysisInconclusive
                } else {
                    Event::AnalysisTimedOut
                },
            );
            hw.prompt(Prompt::AnalysisInconclusive);
            self.go(RescueState::Cpr, hw, now);
            return true;
        }
        false
    }

    /// "Do not touch the patient. Still analysing.", at most once per
    /// [`Config::hands_off_reminder_ms`].
    fn remind_hands_off(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let due = self
            .hands_off_reminded_at
            .is_none_or(|t| now.since(t) >= self.config.hands_off_reminder_ms);
        if due {
            self.hands_off_reminded_at = Some(now);
            hw.prompt(Prompt::DoNotTouchPatient);
        }
    }

    /// A shock is advised: straight to the countdown if the capacitor is
    /// already charged, otherwise wait for the charge to finish.
    fn shock_advised(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if self.state == RescueState::Analysing && self.charge_inhibited(now) {
            // Too hot to charge (D-51): say so and go to CPR at once; the
            // next analysis comes after the CPR period. No charging during
            // CPR: see the README.
            self.events.push(now, Event::ShockHeldForHeat);
            self.counters.shocks_held_for_heat =
                self.counters.shocks_held_for_heat.saturating_add(1);
            self.not_given();
            hw.prompt(Prompt::ShockAdvisedTooHot);
            self.go(RescueState::Cpr, hw, now);
            return;
        }
        if self.state == RescueState::Analysing && !self.begin_arming(hw, now) {
            return;
        }
        // From now on any other answer stops the shock.
        self.veto = false;
        if self.charged_at.is_some() && self.emptying_since.is_none() {
            hw.prompt(Prompt::ShockAdvised);
            self.go(RescueState::Countdown, hw, now);
        } else {
            hw.prompt(Prompt::ShockAdvisedCharging);
            self.go(RescueState::Charging, hw, now);
        }
    }

    fn tick_analysing(&mut self, hw: &mut impl MainHardware, now: Millis, cancel: bool) {
        if !self.pads.on {
            self.go(RescueState::AttachPads, hw, now);
        } else if cancel {
            hw.prompt(Prompt::AnalysisCancelled);
            self.go(RescueState::Cpr, hw, now);
        } else {
            self.analysis_step(hw, now);
        }
    }

    fn tick_analysing_charging(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        if !self.pads.on {
            self.go(RescueState::AttachPads, hw, now);
            return;
        }
        if cancel {
            hw.prompt(Prompt::AnalysisCancelled);
            self.go(RescueState::Cpr, hw, now);
            return;
        }
        if self.charge_inhibited(now) {
            self.drop_charge_keep_analysing(hw, now);
            self.analysis_step(hw, now);
            return;
        }
        if self.mode != self.mode_at_arming {
            // The switch was moved while analysing: empty the capacitor and
            // charge again for the new mode. The analysis itself carries on.
            self.burn_charge_if_any(now);
            self.start_emptying(hw, now);
            if !self.begin_arming(hw, now) {
                return;
            }
        }
        if let Some(kind) = self.charge_check(now, voltage) {
            self.begin_fault(kind, hw, now);
            return;
        }
        self.analysis_step(hw, now);
    }

    /// Charging progress and limits while the capacitor is meant to be
    /// charging or charged (charging during analysis, and Charging).
    /// Records "charged" the first time the plan is reached.
    fn charge_check(&mut self, now: Millis, voltage: Volts) -> Option<FaultKind> {
        if voltage > self.config.overvoltage_limit {
            return Some(FaultKind::Overvoltage);
        }
        let armed_for = self.armed_since.map_or(0, |t| now.since(t));
        if armed_for > self.config.max_charged_ms {
            return Some(FaultKind::ArmedTooLong);
        }
        // Readings only count once this arming's own charge is under way
        // (not while an earlier charge is still being emptied).
        let charging = self.emptying_since.is_none() && self.charger_was_on;
        match self.charged_at {
            None => {
                if charging && self.config.reached_plan(voltage, self.target) {
                    self.charged_at = Some(now);
                    self.events.push(now, Event::Charged(voltage));
                } else if armed_for > self.config.charge_time_limit_ms {
                    return Some(FaultKind::ChargeTimeout);
                }
            }
            Some(_) => {
                if !self.config.is_at_plan(voltage, self.target) {
                    return Some(FaultKind::ChargeOffTarget);
                }
            }
        }
        None
    }

    /// Checks shared by Charging, Countdown and Delivering.
    fn armed_check(&self, now: Millis, cancel: bool, voltage: Volts) -> Option<ArmedExit> {
        if cancel {
            return Some(ArmedExit::Abort(AbortReason::Cancelled));
        }
        if !self.pads.on {
            return Some(ArmedExit::Abort(AbortReason::PadsOff));
        }
        if !self.chest_still(now) {
            // Compressions (or cannot tell) after the shock was advised:
            // never shock a chest that is being compressed (D-36). If the
            // latest pad reading also shows the pads off, that is the
            // reason (the impedance channel sees a lifting pad before the
            // smoothed pad contact does).
            return Some(ArmedExit::Abort(if self.pads_raw_on {
                AbortReason::HandsOnChest
            } else {
                AbortReason::PadsOff
            }));
        }
        if self.supervisor_too_hot(now) {
            return Some(ArmedExit::Abort(AbortReason::TooHot));
        }
        if self.mode != self.mode_at_arming {
            return Some(ArmedExit::Abort(AbortReason::ModeChanged));
        }
        if self.veto {
            return Some(ArmedExit::Abort(AbortReason::RhythmChanged));
        }
        if voltage > self.config.overvoltage_limit {
            return Some(ArmedExit::Fault(FaultKind::Overvoltage));
        }
        let armed_for = self.armed_since.map_or(0, |t| now.since(t));
        if armed_for > self.config.max_charged_ms {
            return Some(ArmedExit::Fault(FaultKind::ArmedTooLong));
        }
        None
    }

    fn handle_armed_exit(&mut self, exit: ArmedExit, hw: &mut impl MainHardware, now: Millis) {
        match exit {
            ArmedExit::Abort(reason) => self.abort(reason, hw, now),
            ArmedExit::Fault(kind) => self.begin_fault(kind, hw, now),
        }
    }

    fn tick_charging(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        if let Some(exit) = self.armed_check(now, cancel, voltage) {
            self.handle_armed_exit(exit, hw, now);
            return;
        }
        if let Some(kind) = self.charge_check(now, voltage) {
            self.begin_fault(kind, hw, now);
            return;
        }
        if self.charged_at.is_some() {
            self.go(RescueState::Countdown, hw, now);
        }
    }

    fn tick_countdown(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        if let Some(exit) = self.armed_check(now, cancel, voltage) {
            self.handle_armed_exit(exit, hw, now);
            return;
        }
        if !self.config.is_at_plan(voltage, self.target) {
            self.begin_fault(FaultKind::ChargeOffTarget, hw, now);
            return;
        }
        let elapsed = self.time_in_state_ms(now);
        if elapsed >= self.config.countdown_ms {
            // D-37 with FW-13: the chest must have been seen (not blanked by
            // the charger, not re-acquiring) when the request goes out. Top-
            // ups are stopped for the last second of the countdown, so this
            // waits at most for the detector; if it has not seen the chest
            // within a further second, the shock is held back as for
            // movement.
            if self.chest_unseen_after_charge(now) {
                if elapsed >= self.config.countdown_ms.saturating_add(1_000) {
                    self.handle_armed_exit(ArmedExit::Abort(AbortReason::HandsOnChest), hw, now);
                }
                return;
            }
            self.deliver(hw, now);
            return;
        }
        // Speak the number of whole seconds left, rounded up: 3, 2, 1.
        let remaining = self.config.countdown_ms.saturating_sub(elapsed);
        let seconds = remaining.saturating_add(999) / 1000;
        let seconds = u8::try_from(seconds).unwrap_or(u8::MAX);
        if seconds != self.countdown_spoken {
            self.countdown_spoken = seconds;
            hw.prompt(Prompt::Countdown(seconds));
        }
    }

    fn deliver(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let fresh = matches!(
            self.last_decision,
            Some((RhythmDecision::Shockable, at))
                if now.since(at) <= self.config.confirmation_max_age_ms
        );
        if !fresh || self.veto {
            self.abort(AbortReason::ConfirmationStale, hw, now);
            return;
        }
        let ohms = hw.pad_resistance();
        self.events.push(now, Event::PadResistance(ohms));
        match self.planner.plan(self.mode, ohms) {
            Err(e) => {
                self.events.push(now, Event::PlanRefused(e));
                if !pads_sensed(ohms) && self.pads.on {
                    // The fresh reading shows the pads off (or shorted)
                    // before the smoothed contact has caught up: treat it as
                    // pads off now, so the device asks for the pads at once.
                    self.pads.force(false);
                    self.events.push(now, Event::PadsOff);
                }
                self.abort(AbortReason::PlanRefused(e), hw, now);
            }
            Ok(plan) => {
                // Both are nominal numbers from the planner, not readings,
                // so they must match exactly.
                if plan.charge_voltage != self.target {
                    self.begin_fault(FaultKind::PlanMismatch, hw, now);
                    return;
                }
                self.plan = Some(plan);
                self.events.push(now, Event::Plan(plan));
                // The hardware must be at rest with our request down: if it
                // is not, a consent line is stuck on or a lockout never
                // cleared, and a request could not start a sequence of its
                // own (one sequence per permit).
                let status = hw.sequence_status();
                if status != SequenceStatus::Idle {
                    self.events
                        .push(now, Event::SequenceNotIdleAtRequest(status));
                    self.begin_fault(FaultKind::InterlockNotIdle, hw, now);
                    return;
                }
                hw.request_shock(Some(&plan));
                self.delivery = Some(Delivery {
                    requested_at: now,
                    permit_seen: false,
                    armed_at: None,
                    started_at: None,
                    ended: None,
                });
                self.events.push(now, Event::ShockRequested);
                self.go(RescueState::Delivering, hw, now);
            }
        }
    }

    /// Follow the hardware sequence (D-59). Pad and voltage were checked
    /// when the request was raised; from then on their expected changes
    /// (pads read open once the relay moves, the voltage falls) are not
    /// reasons to withdraw. Before the first conducting moment the request
    /// is still withdrawn for cancel, over-voltage, the supervisor's heat
    /// limit, a mode change and, while the pads are still on the front
    /// ends, sensed chest movement (D-37); a stale link or a supervisor
    /// fault withdraws it through the fault path. Once the sequence is
    /// active, withdrawing has no effect and is not tried.
    fn tick_delivering(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        cancel: bool,
        voltage: Volts,
    ) {
        let Some(mut d) = self.delivery else {
            // Cannot happen: Delivering is only entered with a request.
            self.begin_fault(FaultKind::DeliveryFault, hw, now);
            return;
        };
        let c = self.config;
        let status = hw.sequence_status();
        let relay = hw.patient_relay();
        if self
            .link
            .is_some_and(|l| l.report.permit && l.received_at >= d.requested_at)
        {
            d.permit_seen = true;
        }
        if status != SequenceStatus::Idle && d.armed_at.is_none() {
            d.armed_at = Some(now);
            self.events.push(now, Event::SequenceArmed);
        }
        if status.started() && d.started_at.is_none() {
            d.started_at = Some(now);
            self.events.push(now, Event::SequenceStarted);
        }
        if let SequenceStatus::Ended(end) = status {
            if d.ended.is_none() {
                d.ended = Some((now, end));
                self.events.push(now, Event::SequenceEnded(end));
                // Consent no longer matters; lowering it lets the lockout
                // clear once the supervisor lowers its permit too.
                hw.withdraw_shock_request();
            }
        }
        self.delivery = Some(d);

        if let Some((at, end)) = d.ended {
            // Judge the result on a settled reading (a converter may sample
            // only every few milliseconds). Both controllers still hold.
            if now.since(at) >= c.post_shock_settle_ms {
                self.sequence_finished(end, hw, now, voltage);
            }
            return;
        }
        match status {
            SequenceStatus::Active => {
                let running = d.started_at.map_or(0, |t| now.since(t));
                if running > c.sequence_end_timeout_ms {
                    self.delivery_fault(ShockOutcome::DidNotEnd, hw, now);
                }
            }
            SequenceStatus::LockedOut => {
                self.shock_not_delivered(ShockOutcome::RefusedByInterlock, hw, now);
            }
            SequenceStatus::Armed => {
                if let Some(exit) = self.prestart_check(now, cancel, voltage, false) {
                    self.withdraw_before_start(exit, hw, now);
                } else if d
                    .armed_at
                    .is_some_and(|t| now.since(t) > c.sequence_start_timeout_ms)
                {
                    let outcome = if relay == RelayPosition::Bridge {
                        ShockOutcome::NotStarted
                    } else {
                        ShockOutcome::RelayDidNotTransfer
                    };
                    self.shock_not_delivered(outcome, hw, now);
                }
            }
            SequenceStatus::Idle => {
                if d.armed_at.is_some() {
                    // The rail rose and fell again before any current: a
                    // consent was taken back before the start.
                    self.shock_not_delivered(ShockOutcome::RefusedByInterlock, hw, now);
                } else if let Some(exit) = self.prestart_check(now, cancel, voltage, true) {
                    self.withdraw_before_start(exit, hw, now);
                } else if now.since(d.requested_at) > c.delivery_timeout_ms {
                    let outcome = if d.permit_seen {
                        ShockOutcome::RefusedByInterlock
                    } else {
                        ShockOutcome::TimedOut
                    };
                    self.shock_not_delivered(outcome, hw, now);
                }
            }
            SequenceStatus::Ended(_) => {}
        }
    }

    /// What may still take a request back before the sequence starts.
    /// `front_ends_valid`: the rail is not up yet, so the pads are still on
    /// the front ends and the chest reading still means something.
    fn prestart_check(
        &self,
        now: Millis,
        cancel: bool,
        voltage: Volts,
        front_ends_valid: bool,
    ) -> Option<ArmedExit> {
        if cancel {
            return Some(ArmedExit::Abort(AbortReason::Cancelled));
        }
        if front_ends_valid && !self.chest_still(now) {
            // D-37: never shock a chest that is being compressed.
            return Some(ArmedExit::Abort(AbortReason::HandsOnChest));
        }
        if self.supervisor_too_hot(now) {
            return Some(ArmedExit::Abort(AbortReason::TooHot));
        }
        if self.mode != self.mode_at_arming {
            return Some(ArmedExit::Abort(AbortReason::ModeChanged));
        }
        if voltage > self.config.overvoltage_limit {
            return Some(ArmedExit::Fault(FaultKind::Overvoltage));
        }
        let armed_for = self.armed_since.map_or(0, |t| now.since(t));
        if armed_for > self.config.max_charged_ms {
            return Some(ArmedExit::Fault(FaultKind::ArmedTooLong));
        }
        None
    }

    fn withdraw_before_start(&mut self, exit: ArmedExit, hw: &mut impl MainHardware, now: Millis) {
        hw.withdraw_shock_request();
        if let ArmedExit::Abort(reason) = exit {
            self.events
                .push(now, Event::Shock(ShockOutcome::Withdrawn(reason)));
        }
        self.handle_armed_exit(exit, hw, now);
    }

    /// No shock, no fault: "Shock not delivered", CPR.
    fn shock_not_delivered(
        &mut self,
        outcome: ShockOutcome,
        hw: &mut impl MainHardware,
        now: Millis,
    ) {
        hw.withdraw_shock_request();
        self.events.push(now, Event::Shock(outcome));
        self.not_given();
        hw.prompt(Prompt::ShockNotDelivered);
        self.go(RescueState::Cpr, hw, now);
    }

    /// The output stage did not behave as planned: a fault.
    fn delivery_fault(&mut self, outcome: ShockOutcome, hw: &mut impl MainHardware, now: Millis) {
        hw.withdraw_shock_request();
        self.events.push(now, Event::Shock(outcome));
        self.begin_fault(FaultKind::DeliveryFault, hw, now);
    }

    /// The hardware reported the sequence ended and the reading has settled.
    fn sequence_finished(
        &mut self,
        end: SequenceEnd,
        hw: &mut impl MainHardware,
        now: Millis,
        voltage: Volts,
    ) {
        match end {
            SequenceEnd::TimeCapTripped => {
                self.delivery_fault(ShockOutcome::TimeCapTripped, hw, now)
            }
            SequenceEnd::OverCurrentTripped => {
                self.delivery_fault(ShockOutcome::OverCurrentTripped, hw, now);
            }
            SequenceEnd::PhaseCapTripped => {
                self.delivery_fault(ShockOutcome::PhaseCapTripped, hw, now)
            }
            SequenceEnd::HardwareFault => self.delivery_fault(ShockOutcome::HardwareFault, hw, now),
            SequenceEnd::Completed => {
                match self.config.check_shock_residual(voltage, self.target) {
                    ShockCheck::Delivered => {
                        self.events.push(now, Event::Shock(ShockOutcome::Delivered));
                        self.counters.shocks_delivered =
                            self.counters.shocks_delivered.saturating_add(1);
                        self.shocked_this_arming = true;
                        self.analyzer.reset();
                        self.last_decision = None;
                        hw.prompt(Prompt::ShockDelivered);
                        self.go(RescueState::Cpr, hw, now);
                    }
                    ShockCheck::NotDelivered => {
                        self.shock_not_delivered(ShockOutcome::NotDelivered, hw, now);
                    }
                    ShockCheck::Implausible | ShockCheck::HardwareTrip => {
                        self.delivery_fault(ShockOutcome::ResidualImplausible, hw, now);
                    }
                }
            }
        }
    }

    /// The background watch after leaving `Delivering` (see
    /// [`SequenceWatch`]).
    fn watch_sequence(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let Some(mut w) = self.watch else {
            return;
        };
        let c = self.config;
        let status = hw.sequence_status();
        let relay = hw.patient_relay();
        if relay != RelayPosition::FrontEnds || status.started() || status == SequenceStatus::Armed
        {
            w.relay_moved = true;
        }
        if !w.ran && status.started() {
            // The hardware latched just as the request was withdrawn: by
            // design it completes (D-59).
            w.ran = true;
            w.relay_back_at = None;
            self.events.push(now, Event::SequenceStarted);
        }
        if w.ran && !w.end_recorded {
            if let SequenceStatus::Ended(end) = status {
                w.end_recorded = true;
                w.since = now;
                self.events
                    .push(now, Event::SequenceCompletedAfterWithdrawal(end));
                if end == SequenceEnd::Completed {
                    self.counters.shocks_delivered =
                        self.counters.shocks_delivered.saturating_add(1);
                }
                self.analyzer.reset();
                self.last_decision = None;
                self.analysis_result = None;
            }
        }
        let running = status == SequenceStatus::Active;
        if relay == RelayPosition::FrontEnds {
            if w.relay_moved && w.relay_back_at.is_none() && !running {
                w.relay_back_at = Some(now);
                self.events.push(now, Event::RelayReturned);
            }
        } else if !running && !w.relay_fault && now.since(w.since) > c.relay_return_timeout_ms {
            w.relay_fault = true;
            if self.state == RescueState::Fault {
                self.events
                    .push(now, Event::Fault(FaultKind::PatientRelayStuck));
            } else {
                self.begin_fault(FaultKind::PatientRelayStuck, hw, now);
            }
        }
        let quiet = c
            .sequence_start_timeout_ms
            .saturating_add(c.sequence_end_timeout_ms);
        let done = match w.relay_back_at {
            Some(t) => now.since(t) >= c.front_end_recovery_ms,
            None => !w.relay_moved && now.since(w.since) > quiet,
        };
        if done {
            self.watch = None;
            if w.relay_moved {
                self.front_ends_recovered(hw, now);
            }
        } else {
            self.watch = Some(w);
        }
    }

    /// The front ends have recovered: start the rhythm analysis and
    /// compression sensing from scratch, and say if the pads did not come
    /// back.
    fn front_ends_recovered(&mut self, hw: &mut impl MainHardware, now: Millis) {
        self.events.push(now, Event::FrontEndsRecovered);
        self.analyzer.reset();
        self.chest.reset();
        self.chest_last_sample = None;
        if self.state.is_analysing() {
            self.analysis_result = None;
            self.last_decision = None;
        }
        if !pads_sensed(hw.pad_resistance()) {
            self.events.push(now, Event::PadsNotBackAfterShock);
        }
    }

    /// D-60: compare the adult/child switch's two contacts.
    fn check_mode_contacts(
        &mut self,
        hw: &mut impl MainHardware,
        now: Millis,
        primary: PatientMode,
    ) {
        let ceiling = hw.patient_mode_ceiling();
        if ceiling == primary {
            self.mode_contacts_since = None;
            if self.mode_contacts_bad {
                self.mode_contacts_bad = false;
                self.events.push(now, Event::ModeContactsAgree);
                hw.set_ready_indicator(self.ready());
            }
            return;
        }
        let since = *self.mode_contacts_since.get_or_insert(now);
        if !self.mode_contacts_bad && now.since(since) >= self.config.mode_contacts_disagree_ms {
            self.mode_contacts_bad = true;
            self.events
                .push(now, Event::ModeContactsDisagree { primary, ceiling });
            hw.set_ready_indicator(false);
        }
    }

    /// The ready indicator: on once the switch-on self-test has passed,
    /// unless the device has a fault or the mode switch's contacts disagree.
    fn ready(&self) -> bool {
        matches!(self.self_test_result, Some(Ok(())))
            && self.state != RescueState::Fault
            && !self.mode_contacts_bad
    }

    fn tick_cpr(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if !self.pads.on {
            self.end_cpr_period(now);
            self.go(RescueState::AttachPads, hw, now);
            return;
        }
        self.metronome(hw, now);
        if self.poor_contact.on
            && now.since(self.press_pads_reminded_at) >= self.config.press_pads_repeat_ms
        {
            self.press_pads_reminded_at = now;
            hw.prompt(Prompt::PressPadsFirmly);
        }
        self.prompt_if_no_compressions(hw, now);
        if self.time_in_state_ms(now) >= self.config.cpr_period_ms {
            self.end_cpr_period(now);
            self.enter_analysis(hw, now);
        }
    }

    /// During CPR: "Continue CPR." when compression sensing has seen a still
    /// chest, with no compressions, for [`Config::no_compression_prompt_ms`]
    /// (and again after the same time). Never while it cannot tell.
    fn prompt_if_no_compressions(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let limit = self.config.no_compression_prompt_ms;
        if limit == 0 || self.chest_state(now) != Some(ChestState::HandsOff) {
            return;
        }
        let quiet = self
            .chest
            .ms_since_compression()
            .min(self.time_in_state_ms(now));
        let since_said = self.continue_cpr_said_at.map_or(u32::MAX, |t| now.since(t));
        if quiet >= limit && since_said >= limit {
            self.continue_cpr_said_at = Some(now);
            self.counters.continue_cpr_prompts =
                self.counters.continue_cpr_prompts.saturating_add(1);
            self.events.push(now, Event::NoCompressionsPrompt);
            hw.prompt(Prompt::ContinueCpr);
        }
    }

    /// Record what compression sensing measured during the CPR period that
    /// is ending (nothing without compression sensing).
    fn end_cpr_period(&mut self, now: Millis) {
        if !self.chest_sensing_active() {
            return;
        }
        let p = self.cpr_period;
        let length = self.time_in_state_ms(now).max(1);
        let rate = u32::from(p.intervals)
            .saturating_mul(60_000)
            .checked_div(p.interval_sum_ms)
            .unwrap_or(0);
        let on_chest = u64::from(p.on_chest_ms) * 1000 / u64::from(length);
        // A pause still running at the end of the period counts too.
        let open_pause = self.chest.ms_since_compression().min(length);
        self.events.push(
            now,
            Event::CprPeriodSummary {
                compressions: p.compressions,
                rate_per_min: u16::try_from(rate).unwrap_or(u16::MAX),
                on_chest_permille: u16::try_from(on_chest.min(1000)).unwrap_or(1000),
                longest_pause_ms: p.longest_pause_ms.max(open_pause),
            },
        );
    }

    fn tick_fault(&mut self, hw: &mut impl MainHardware, now: Millis) {
        // A faulty device must never leave a responder waiting: keep the
        // metronome going and keep reminding them to continue CPR.
        self.metronome(hw, now);
        if now.since(self.last_reminder) >= self.config.fault_reminder_ms {
            self.last_reminder = now;
            hw.prompt(self.fault_prompt());
        }
    }

    // ----- transitions ----------------------------------------------------

    fn go(&mut self, to: RescueState, hw: &mut impl MainHardware, now: Millis) {
        let from = self.state;
        if from == RescueState::Delivering {
            // Keep watching the hardware in the background (D-59).
            if let Some(d) = self.delivery.take() {
                self.watch = Some(SequenceWatch {
                    since: d.ended.map_or(now, |(t, _)| t),
                    relay_moved: d.armed_at.is_some(),
                    ran: d.started_at.is_some() || d.ended.is_some(),
                    end_recorded: d.ended.is_some(),
                    relay_back_at: None,
                    relay_fault: false,
                });
            }
        }
        self.events.push(now, Event::StateChanged { from, to });
        self.state = to;
        self.state_since = now;
        self.state_changed = true;
        // Leaving the charge-holding states (or a self-test that may have
        // charged), and on any fault: let go, and empty in the background.
        let was_charged = from.holds_charge() || from == RescueState::SelfTest;
        if !to.holds_charge() && (was_charged || to == RescueState::Fault) {
            if from.holds_charge() {
                self.burn_charge_if_any(now);
                self.armed_since = None;
                self.charged_at = None;
            }
            if from != RescueState::SelfTest || to == RescueState::Fault {
                self.start_emptying(hw, now);
            }
        }
        match to {
            RescueState::AttachPads => {
                self.last_reminder = now;
                if !self.pads.on {
                    hw.prompt(Prompt::AttachPads);
                }
            }
            RescueState::Analysing | RescueState::AnalysingCharging => {
                // After CPR, with the chest confirmed still, the window
                // starts now on the filters that ran through CPR (D-47).
                // In every other case (pads just attached, analysing again
                // after movement held a shock back, no compression sensing,
                // or the chest not confirmed still) a full reset, as before.
                if from == RescueState::Cpr {
                    self.start_analysis_window(now);
                } else {
                    self.analyzer.reset();
                }
                self.analysis_result = None;
                self.not_sure_seen = false;
                // With compression sensing the reminder follows the chest
                // state; the instruction just given counts as the first.
                self.hands_off_reminded_at = self.chest_state(now).map(|_| now);
                if from == RescueState::Cpr || from == RescueState::AttachPads {
                    self.hands_on_reanalyses_used = 0;
                }
                self.last_decision = None;
                self.counters.analyses = self.counters.analyses.saturating_add(1);
                if self.poor_contact.on {
                    // Pads on but above the shock range (D-32): ask for them
                    // to be pressed down before the hands-off instruction.
                    self.press_pads_reminded_at = now;
                    hw.prompt(Prompt::PressPadsFirmly);
                }
                hw.prompt(if from == RescueState::Cpr {
                    Prompt::StopCprAnalysing
                } else {
                    Prompt::AnalysingDoNotTouch
                });
            }
            RescueState::Countdown => {
                self.countdown_spoken = 0;
                hw.prompt(Prompt::StandClear);
            }
            RescueState::Cpr => {
                self.next_beat = now;
                self.press_pads_reminded_at = now;
                self.cpr_period = CprPeriod::default();
                self.continue_cpr_said_at = None;
                self.counters.cpr_periods = self.counters.cpr_periods.saturating_add(1);
                hw.prompt(Prompt::StartCpr);
            }
            RescueState::Fault => {
                hw.set_charger(None);
                hw.withdraw_shock_request();
                hw.set_discharge_hold(false);
                hw.set_ready_indicator(false);
                self.next_beat = now;
                self.last_reminder = now;
                hw.prompt(self.fault_prompt());
            }
            RescueState::SelfTest | RescueState::Charging | RescueState::Delivering => {}
        }
    }

    /// Count a charge that is being emptied without a shock.
    fn burn_charge_if_any(&mut self, now: Millis) {
        if self.charger_was_on && !self.shocked_this_arming {
            self.counters.charges_burned = self.counters.charges_burned.saturating_add(1);
            self.events.push(now, Event::ChargeBurnedOff);
        }
        self.charger_was_on = false;
    }

    fn abort(&mut self, reason: AbortReason, hw: &mut impl MainHardware, now: Millis) {
        if reason == AbortReason::PadsOff && self.pads.on && !self.pads_raw_on {
            // Take the fresh reading's word for it (as for the shock plan),
            // so the one instruction is "Attach pads" (finding F-6).
            self.pads.force(false);
            self.events.push(now, Event::PadsOff);
        }
        self.events.push(now, Event::ShockAborted(reason));
        self.not_given();
        if reason == AbortReason::HandsOnChest {
            self.counters.hands_on_aborts = self.counters.hands_on_aborts.saturating_add(1);
            if self.pads.on && self.hands_on_reanalyses_used < self.config.hands_on_reanalyses {
                // Someone touched or compressed the chest after "shock
                // advised": empty the charge and analyse again from the
                // start, rather than sending a patient in VF to two minutes
                // of CPR for one touch.
                self.hands_on_reanalyses_used = self.hands_on_reanalyses_used.saturating_add(1);
                hw.prompt(Prompt::MovementShockDelayed);
                self.burn_charge_if_any(now);
                self.armed_since = None;
                self.charged_at = None;
                self.start_emptying(hw, now);
                self.enter_analysis(hw, now);
                return;
            }
        }
        hw.prompt(match reason {
            AbortReason::Cancelled => Prompt::ShockCancelled,
            AbortReason::RhythmChanged => Prompt::RhythmChangedNoShock,
            AbortReason::PlanRefused(PlanError::ImpedanceTooHigh) if self.pads.on => {
                Prompt::PadContactPoorNoShock
            }
            AbortReason::PadsOff | AbortReason::PlanRefused(_) => Prompt::CheckPads,
            AbortReason::ModeChanged | AbortReason::ConfirmationStale => Prompt::ShockNotDelivered,
            AbortReason::HandsOnChest => Prompt::MovementNoShock,
            AbortReason::TooHot => Prompt::TooHotNoShock,
        });
        // With the pads off the device can neither analyse nor shock, so the
        // one instruction is to put them back (finding F-6): straight to
        // "Attach pads", not "Start CPR" followed at once by "Attach pads".
        let next = if self.pads.on {
            RescueState::Cpr
        } else {
            RescueState::AttachPads
        };
        self.go(next, hw, now);
    }

    fn not_given(&mut self) {
        self.counters.shocks_not_given = self.counters.shocks_not_given.saturating_add(1);
    }

    /// Start the fault path: stop everything, empty the capacitor in the
    /// background, and go to the Fault state at once ("continue CPR").
    fn begin_fault(&mut self, kind: FaultKind, hw: &mut impl MainHardware, now: Millis) {
        if self.state == RescueState::Fault {
            return;
        }
        self.events.push(now, Event::Fault(kind));
        self.fault = Some(kind);
        if self.state == RescueState::SelfTest {
            self.self_test.abort(hw);
        }
        hw.withdraw_shock_request();
        self.go(RescueState::Fault, hw, now);
    }

    fn fault_prompt(&self) -> Prompt {
        match self.fault {
            Some(k) if k.is_serious() => Prompt::SeriousFaultContinueCpr,
            Some(FaultKind::CancelStuck | FaultKind::Supervisor(SupervisorFault::CancelStuck)) => {
                Prompt::CancelStuckContinueCpr
            }
            _ => Prompt::DeviceFaultContinueCpr,
        }
    }

    // ----- outputs and messages -------------------------------------------

    fn metronome(&mut self, hw: &mut impl MainHardware, now: Millis) {
        if now >= self.next_beat {
            hw.metronome_beat();
            self.counters.metronome_beats = self.counters.metronome_beats.saturating_add(1);
            let interval = self.config.metronome_interval_ms();
            let next = self.next_beat.0.saturating_add(interval);
            // If ticks were late, do not burst-beat to catch up.
            self.next_beat = Millis(if next <= now.0 {
                now.0.saturating_add(interval)
            } else {
                next
            });
        }
    }

    /// Re-assert every output for the current state, every tick. Hold before
    /// charging (and charge only once the supervisor reports it holds too);
    /// stop charging before releasing. While an earlier charge is still being
    /// emptied, hold nothing and charge nothing.
    fn apply_outputs(&mut self, hw: &mut impl MainHardware) {
        let emptying = self.emptying_since.is_some();
        match self.state {
            // The self-test sequencer drives the outputs itself.
            RescueState::SelfTest => {}
            RescueState::AnalysingCharging | RescueState::Charging | RescueState::Countdown
                if !emptying =>
            {
                hw.set_discharge_hold(true);
                // The charger cannot run until the supervisor holds too
                // (interlock rule), so ask for it only once the supervisor
                // reports that it is holding, in a report that arrived after
                // this charge was asked for (not one left over from the
                // self-test or an earlier charge).
                let since = self.armed_since;
                let supervisor_holding = self
                    .link
                    .is_some_and(|l| l.report.holding && since.is_some_and(|a| l.received_at > a));
                if supervisor_holding {
                    if !self.charger_was_on {
                        self.counters.capacitor_charges =
                            self.counters.capacitor_charges.saturating_add(1);
                    }
                    self.charger_was_on = true;
                    let v = hw.capacitor_voltage();
                    let done = hw.charger_done();
                    let now = hw.now();
                    // No top-up in the last second of the countdown: the
                    // chest check before the request must not be blanked.
                    let final_second = self.state == RescueState::Countdown
                        && self.time_in_state_ms(now).saturating_add(1_000)
                            >= self.config.countdown_ms;
                    let decision = if final_second && !done {
                        ChargeAction::Off
                    } else {
                        self.charge_ctl
                            .decide(&self.config, self.target, v, done, now)
                    };
                    match decision {
                        ChargeAction::On => self.charger(hw, Some(self.target), now),
                        ChargeAction::Off => self.charger(hw, None, now),
                        ChargeAction::Restart => {
                            self.events.push(now, Event::ChargerRestarted(v));
                            self.charger(hw, None, now);
                        }
                        ChargeAction::StoppedItself => {
                            self.charger(hw, None, now);
                            self.events.push(now, Event::ChargerStoppedItself(v));
                            self.begin_fault(FaultKind::ChargerStoppedItself, hw, now);
                            return;
                        }
                    }
                } else {
                    let now = hw.now();
                    self.charger(hw, None, now);
                }
                // FW-12 as revised by the patient-interface capture: the coil
                // supply's pre-charge is NOT overlapped with a full-power
                // charge (a cold, nearly empty battery would sag to about
                // 3.7 V). It is raised once the charge has reached the plan
                // and the charger is off, and kept up through any later
                // top-up; as a last resort it is forced up 500 ms before
                // the end of the countdown (≥ 200 ms before `RELAY_CMD`),
                // accepting a top-up overlap then.
                let now = hw.now();
                let charged = !self.charger_on && !self.charge_ctl.topping_up;
                let late = self.state == RescueState::Countdown
                    && self.time_in_state_ms(now).saturating_add(500) >= self.config.countdown_ms;
                let on = self.coil_supply_since.is_some() || charged || late;
                self.coil_supply(hw, on, now);
            }
            RescueState::Delivering if !emptying => {
                hw.set_discharge_hold(true);
                let now = hw.now();
                self.charger(hw, None, now);
                self.coil_supply(hw, true, now);
            }
            RescueState::AttachPads
            | RescueState::Analysing
            | RescueState::AnalysingCharging
            | RescueState::Charging
            | RescueState::Countdown
            | RescueState::Delivering
            | RescueState::Cpr
            | RescueState::Fault => {
                let now = hw.now();
                self.charger(hw, None, now);
                self.coil_supply(hw, false, now);
                hw.withdraw_shock_request();
                hw.set_discharge_hold(false);
            }
        }
    }

    /// Command the charger and keep the record compression blanking needs.
    fn charger(&mut self, hw: &mut impl MainHardware, target: Option<Volts>, now: Millis) {
        let on = target.is_some();
        if self.charger_on && !on {
            self.charger_off_at = Some(now);
        }
        self.charger_on = on;
        hw.set_charger(target);
    }

    /// Raise or lower `K_SUPPLY_EN` (FW-12): up from the start of the charge
    /// for a shock (so at least the countdown, 3 s, before `RELAY_CMD`;
    /// FW-12 asks for 200 ms), down on withdrawal, abort or the end.
    fn coil_supply(&mut self, hw: &mut impl MainHardware, on: bool, now: Millis) {
        if on {
            self.coil_supply_since.get_or_insert(now);
        } else {
            self.coil_supply_since = None;
        }
        hw.set_coil_supply(on);
    }

    /// True while compression sensing is in use but has not seen the chest
    /// since the charger blanked it (blanked now, or still re-acquiring).
    fn chest_unseen_after_charge(&self, now: Millis) -> bool {
        if !(self.config.chest_sensing && self.chest.fitted() && !self.chest_failed) {
            return false;
        }
        self.chest_blanked(now) || self.chest_reacquiring(now)
    }

    /// After a blanking gap, until the compression detector gives a
    /// definite state again (at most [`CHEST_REACQUIRE_MS`]), the chest
    /// counts as "not sensed" rather than "cannot tell": otherwise the end of
    /// every charge would reset the analysis.
    fn chest_reacquiring(&self, now: Millis) -> bool {
        self.config.blank_compressions_while_charging
            && !self.chest_seen_since_gap
            && self.charger_off_at.is_some_and(|t| {
                now.since(t)
                    < self
                        .config
                        .impedance_blank_after_charge_ms
                        .saturating_add(CHEST_REACQUIRE_MS)
            })
    }

    /// True while compression readings are thrown away because the charger
    /// runs or ran within [`Config::impedance_blank_after_charge_ms`]
    /// (FW-13, D-67). The chest check then relies on the heart signal
    /// alone, as when sensing has failed (D-38).
    #[must_use]
    pub fn chest_blanked(&self, now: Millis) -> bool {
        self.config.blank_compressions_while_charging
            && (self.charger_on
                || self
                    .charger_off_at
                    .is_some_and(|t| now.since(t) < self.config.impedance_blank_after_charge_ms))
    }

    fn send_status_if_due(&mut self, hw: &mut impl MainHardware, now: Millis) {
        let test_request = if self.state == RescueState::SelfTest {
            self.self_test.test_request()
        } else {
            TestRequest::None
        };
        let due = self
            .last_status_at
            .is_none_or(|t| now.since(t) >= self.config.heartbeat_period_ms);
        if !due && !self.state_changed && test_request == self.last_test_request {
            return;
        }
        self.last_test_request = test_request;
        self.state_changed = false;
        self.last_status_at = Some(now);
        self.status_sequence = self.status_sequence.wrapping_add(1);
        let armed = self.state.holds_charge();
        let planned_voltage = if armed {
            self.target
        } else if self.state == RescueState::SelfTest && self.self_test.is_charging() {
            self.self_test.test_voltage()
        } else {
            Volts(0)
        };
        let armed_for_ms = if armed {
            self.armed_since.map_or(0, |t| now.since(t))
        } else {
            0
        };
        let capacitor_voltage = hw.capacitor_voltage();
        hw.send_status(&MainStatus {
            sequence: self.status_sequence,
            state: self.state,
            capacitor_voltage,
            planned_voltage,
            armed_for_ms,
            temperature: self.main_temp_claim,
            test_request,
            emptying: self.emptying_since.is_some(),
            charger_on: self.charger_on
                || (self.state == RescueState::SelfTest && self.self_test.is_charging()),
        });
    }

    fn on_report(&mut self, report: SupervisorReport, now: Millis) {
        let previous = self.link.map(|l| l.report);
        if let Some(check) = report.shock_check {
            if previous.and_then(|p| p.shock_check) != Some(check) {
                self.events
                    .push(now, Event::SupervisorShockCheckReported(check));
            }
        }
        if report.permit && !previous.is_some_and(|p| p.permit) {
            self.events.push(now, Event::SupervisorPermitted);
        }
        if let Some(f) = report.fault {
            if previous.and_then(|p| p.fault).is_none() {
                self.events.push(now, Event::SupervisorFaultLatched(f));
            }
        }
        self.link = Some(LinkSnapshot {
            report,
            received_at: now,
        });
    }
}

/// The announcement for a patient mode.
const fn mode_prompt(mode: PatientMode) -> Prompt {
    match mode {
        PatientMode::Adult => Prompt::AdultMode,
        PatientMode::Child => Prompt::ChildModeFrontBack,
    }
}
