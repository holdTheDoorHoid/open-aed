// SPDX-License-Identifier: GPL-3.0-or-later
//! Every timing and limit the rescue sequence, the supervisor and the
//! self-test use, kept in one place.
//!
//! ARCHITECTURE.md requires that timings live "in one place in the firmware,
//! to be set from the current resuscitation guidelines and the purchased
//! standard, not scattered through the code". This is that place.
//!
//! **Every default below is a provisional starting value.** None has yet been
//! set from the resuscitation guidelines or from the defibrillator standard
//! (IEC 60601-2-4), which has not been purchased and read. Each must be
//! reviewed and replaced with a sourced figure before anything relies on it.

use aed_types::Volts;

/// All timings and limits, in milliseconds and volts.
///
/// Both controllers are given a copy of the same `Config`. They do not share
/// it at run time: each keeps its own copy, as they would on two separate chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// How long one period of CPR lasts before the device stops to analyse again.
    /// Provisional: 120 s, the two-minute cycle in current resuscitation guidelines.
    pub cpr_period_ms: u32,
    /// Compression metronome rate, in beats per minute. Must be 100 to 120.
    /// Provisional: 110, the middle of the guideline range.
    pub metronome_per_min: u16,
    /// Length of the "stand clear" countdown before an automatic shock
    /// ("Stand clear. Three, two, one."). The supervisor also requires the
    /// main controller to have been counting down for at least this long, by
    /// the supervisor's own clock, before it gives a permit.
    /// Owner decision D-30: 3 s (users are trained responders). To be checked
    /// against the defibrillator standard once it is bought.
    pub countdown_ms: u32,
    /// Charge during analysis (owner decision D-29). When true, the capacitor
    /// starts charging as soon as an analysis starts (state
    /// [`crate::RescueState::AnalysingCharging`]), so that a "shock advised"
    /// can go straight to the countdown without a charging wait. If no shock
    /// is advised the charge is emptied into the internal load. When false,
    /// charging starts only once a shock is advised (the first design).
    /// Default: true.
    pub charge_during_analysis: bool,
    /// Longest the capacitor may take to charge to the shock voltage, counted
    /// from the moment charging is asked for, before it counts as a fault.
    /// With charging during analysis this runs from the start of the
    /// analysis. Provisional: 12 s (the requirement is 8 s or less; this
    /// leaves margin).
    pub charge_time_limit_ms: u32,
    /// The charge voltage that [`Config::charge_time_limit_ms`] is sized for.
    /// Shorter charges get a proportionally shorter limit (see
    /// [`Config::charge_time_limit_for`]), so that a weak charger shows up
    /// quickly in the 400 V switch-on self-test (finding F-7) instead of
    /// after 12 s of silence. Provisional: 2,000 V, the highest voltage the
    /// capacitor may ever be charged to (aed-waveform `max_charge`).
    pub charge_limit_reference_voltage: Volts,
    /// The shortest charge time limit for any charge, however small, ms.
    /// Provisional: 1 s. The model charger takes about 0.27 s to reach the
    /// 400 V self-test voltage (it is current-limited at low voltage), so 1 s
    /// is between three and four times that.
    pub min_charge_time_limit_ms: u32,
    /// Longest the capacitor may stay charged before it must be emptied.
    /// The main controller counts from the moment charging is asked for; the
    /// supervisor counts, on its own clock, for as long as its own reading
    /// is not empty. Each empties the capacitor on its own at this limit.
    ///
    /// Provisional: 25 s. The longest a normal shock cycle keeps the
    /// capacitor charged, with charging during analysis (D-29) and the 3 s
    /// countdown (D-30), is:
    ///
    /// * a "shock" answer can arrive as late as the 12 s analysis limit
    ///   (the capacitor charged early, about 6.3 s in for an adult), or the
    ///   charge can take up to its 12 s limit: at most 12 s either way;
    /// * plus the 3 s countdown, plus up to 1 s waiting for delivery: 16 s
    ///   for the main controller;
    /// * plus up to the 3 s emptying limit before the supervisor's own
    ///   reading shows empty (about 0.8 s in the model), and a few
    ///   heartbeats of message delay: about 19.3 s for the supervisor, or
    ///   19.7 s if its clock runs 2 % slow.
    ///
    /// That is shorter than before D-29/D-30 (12 + 5 + 1 + 3 = 21 s), so the
    /// 25 s limit still has about 5 s in hand and is not changed.
    /// [`Config::validate`] refuses a value that does not exceed the sum.
    pub max_charged_ms: u32,
    /// Longest emptying the capacitor may take. If the main controller's
    /// reading does not show empty by then, that is the most serious fault.
    /// Emptying runs in the background (see [`crate::rescue`]): the device
    /// gives its next instruction at once and does not wait for it, but
    /// starts no new charge until the capacitor is confirmed empty.
    /// Provisional: 3 s (the model empties 1,950 V in about 0.8 s).
    pub disarm_time_limit_ms: u32,
    /// Below this voltage the capacitor counts as empty. Provisional: 50 V,
    /// matching the project's "no circuit above 50 volts" line (D-10).
    ///
    /// A controller only counts the capacitor as empty when its reading is
    /// below this voltage by at least its own sensor's allowed error (see
    /// [`Config::reads_empty`]), so a sensor that reads low cannot report an
    /// empty capacitor that is really above this voltage.
    pub empty_voltage: Volts,
    /// How close one controller's own voltage reading must be to the planned
    /// voltage for the capacitor to count as charged, and to stay armed:
    /// within this percentage of the planned voltage plus
    /// [`Config::at_plan_abs_volts`]. Used by the main controller (charged,
    /// and drifting off target during the countdown), by the supervisor (its
    /// reading against the plan before a permit) and by the self-test.
    ///
    /// It must be at least the sensor error allowed by
    /// [`Config::agree_percent`] and [`Config::agree_abs_volts`], or an
    /// in-tolerance sensor would cause false faults. Together with that
    /// sensor error it also bounds how far the true voltage can be from the
    /// plan when a shock fires, which sets the delivered energy (finding
    /// F-9). Provisional: 1 % (see `at_plan_abs_volts` for the reasoning).
    pub at_plan_percent: u8,
    /// The fixed part of the "at the planned voltage" window, volts.
    ///
    /// Provisional: 12 V, the same as one sensor's allowance (the narrowest
    /// [`Config::validate`] accepts). With 1 % this gives 32 V at 1,950 V
    /// (adult), 24 V at 1,150 V (child) and 16 V at 400 V (self-test).
    ///
    /// Why so narrow: the window, widened by the sensor's own error, is how
    /// far the TRUE voltage may be from the plan when a shock fires. At the
    /// child top edge (charger regulating high, both sensors reading 1 % +
    /// 7 V low) the true voltage can be (1,150 + 24 + 7) / 0.99 = 1,193 V,
    /// which the model puts at about +14 % over 50 J into 25 Ω, just inside
    /// the ±15 % placeholder tolerance. A 17 V fixed part (the old "10 V more
    /// than the sensor" margin) would allow 1,198 V, right on the edge.
    /// The 5 V above the 7 V sensor error covers rounding and a charger that
    /// regulates up to about 5 V away from its set point; a real charger
    /// must be specified to that, or this must be revisited.
    pub at_plan_abs_volts: Volts,
    /// The error each of the two independent voltage sensors is allowed,
    /// as a percentage of the reading (plus [`Config::agree_abs_volts`]).
    ///
    /// "Each sensor may be off by `agree_percent` % + `agree_abs_volts` V."
    /// The supervisor counts its reading and the main controller's as agreeing
    /// while they differ by no more than twice that allowance (taken at the
    /// higher reading), so two sensors with opposite errors inside their
    /// allowance always agree. A bigger difference, lasting longer than
    /// [`Config::disagreement_persist_ms`], is a fault.
    ///
    /// Provisional: 1 % + 12 V, from owner decision D-28: each real sensor
    /// must be accurate to about ±1 % ± 7 V. The arithmetic, per sensor:
    ///
    /// * the sensor itself: 1 % of the reading + 7 V;
    /// * rounding of each reading to a whole volt: up to 1 V;
    /// * the main controller's claimed reading arriving slightly old (it is
    ///   read when the status message is sent): 4 V.
    ///
    /// 7 + 1 + 4 = 12 V. Two sensors may then differ by 2 % + 24 V (64 V at
    /// 1,950 V, 47 V at 1,150 V, 32 V at 400 V); two in-spec sensors with
    /// opposite errors differ by at most 2 % + 14 V, leaving 10 V.
    ///
    /// While the capacitor is charging fast a heartbeat-old claim can lag by
    /// far more than 4 V (about 15,000 / V volts in the model: 15 V at
    /// 1,000 V). That is covered by [`Config::disagreement_persist_ms`],
    /// not by this allowance: the readings agree again each time a fresh
    /// status message arrives, which resets the persistence timer. On a
    /// slow message wire (tens of ms of delay) with sensors at opposite
    /// ends of their accuracy this margin is thin; see the aed-core README.
    /// Not from any part's datasheet: no sensor is chosen.
    pub agree_percent: u8,
    /// The fixed part of each sensor's allowed error, volts. Provisional:
    /// 12 V (7 V sensor + 1 V rounding + 4 V stale reading; see
    /// [`Config::agree_percent`]).
    pub agree_abs_volts: Volts,
    /// Above this voltage the capacitor is over-charged and is emptied at once.
    /// Provisional: 2,200 V (the design point is about 1,700 to 2,000 V).
    pub overvoltage_limit: Volts,
    /// How often the main controller sends its status (heartbeat) message to the
    /// supervisor, and the supervisor its report back. Provisional: 50 ms.
    pub heartbeat_period_ms: u32,
    /// If no new heartbeat arrives within this time, the other controller counts
    /// as stopped. Provisional: 250 ms (five missed heartbeats).
    pub heartbeat_timeout_ms: u32,
    /// How long one supervisor permit lasts. Provisional: 200 ms.
    pub permit_window_ms: u32,
    /// How long the supervisor tolerates its voltage reading disagreeing with the
    /// main controller's claim before it empties the capacitor and latches a fault.
    /// A short allowance is needed because the claim is up to one heartbeat old.
    /// Provisional: 500 ms.
    pub disagreement_persist_ms: u32,
    /// A change in pad contact (on to off, or off to on) must last this long
    /// before the device acts on it. The shock plan itself always uses a fresh,
    /// un-smoothed reading. Provisional: 200 ms.
    pub pad_debounce_ms: u32,
    /// Longest an analysis may take. "Not sure" answers do not end an analysis
    /// (movement is the usual cause, so the device keeps looking); if neither
    /// "shock" nor "no shock" has arrived by this limit the analysis is
    /// inconclusive and the device goes to CPR without a shock.
    ///
    /// Provisional: 12 s. The rhythm analysis needs about 6 to 7 s of clean
    /// signal (1 s settling, a 4 s window, 3 agreeing windows), so with 12 s
    /// a shock can still be advised if compressions stop up to about 5 s
    /// after "Analysing" (simulated: 6.0 s late for 400 µV VF, 7.0 s for
    /// 800 µV, 7.5 s for 1,500 µV; the requirement was at least 3 s). The
    /// cost: when the signal never becomes clean (fine VF, interference, a
    /// responder who keeps going), the hands-off pause before "Start CPR"
    /// lasts the whole 12 s. A longer limit tolerates later stops but makes
    /// that pause longer.
    pub analysis_time_limit_ms: u32,
    /// While analysing, after a "not sure" answer the device repeats "Do not
    /// touch the patient", at most once per this interval. Provisional: 3 s.
    pub hands_off_reminder_ms: u32,
    /// At the moment of delivery the latest "shockable" decision must be no older
    /// than this. If it is older, no shock is given. Must be longer than the
    /// interval between the rhythm analyser's decisions. Provisional: 5 s.
    pub confirmation_max_age_ms: u32,
    /// After the shock is requested, how long the main controller waits for the
    /// hardware to report it delivered before giving up. Provisional: 1 s.
    pub delivery_timeout_ms: u32,
    /// Voltage the power-on self-test charges to. A full-energy test (D-15) is
    /// left to the periodic self-test, so that switching on in an emergency is
    /// not delayed by a full charge. Provisional: 400 V.
    pub self_test_voltage: Volts,
    /// How long the self-test waits to hear from the supervisor. Provisional: 1 s.
    pub self_test_link_wait_ms: u32,
    /// How long the self-test waits for the two voltage readings to agree.
    /// Provisional: 500 ms.
    pub self_test_agree_wait_ms: u32,
    /// How often the "attach pads" prompt is repeated while waiting. Provisional: 15 s.
    pub attach_pads_repeat_ms: u32,
    /// How often "Press pads firmly" is repeated during CPR while the pads
    /// are on but read above the shock range (decision D-32). It is also
    /// said once as each analysis starts. Never during the hands-off and
    /// stand-clear phases, which would tell the responder to touch the
    /// patient. Provisional: 15 s, the same as "attach pads".
    pub press_pads_repeat_ms: u32,
    /// The cancel control counts as stuck once it has read pressed without a
    /// break for this long (decision D-31). A stuck cancel would block every
    /// shock, so it is a fault with its own prompt rather than a silent
    /// "cancel". Both controllers apply the rule to their own view of the
    /// button.
    ///
    /// Provisional: 10 s. A deliberate press lasts a second or two, and
    /// someone holding cancel through a whole 3 s countdown has already
    /// stopped that shock; 10 s is well beyond both. It is also shorter than
    /// the time from switch-on to a first shock with the pads already on
    /// (self-test about 0.6 s, analysis about 7 s, countdown 3 s), so a
    /// button stuck from switch-on is reported before a first shock would
    /// have been due. A rule of "pressed from switch-on to the end of the
    /// self-test" alone was not used: the self-test takes well under a
    /// second, so it would turn a brief stray press, the very thing D-31
    /// protects against, into a permanent fault.
    pub cancel_stuck_ms: u32,
    /// How often the fault prompt is repeated in the Fault state. Provisional: 30 s.
    pub fault_reminder_ms: u32,
    /// Use compression sensing through the pads (decision D-36), if the
    /// device has it. When true:
    ///
    /// * the rhythm analysis is fed only while the chest is still ("hands
    ///   off"); whenever it is not (compressing, or cannot tell) the
    ///   analysis is reset, and the "Do not touch the patient" reminder is
    ///   driven by that, not by a "not sure" answer;
    /// * a shock being prepared is held back if the chest is not still
    ///   (see [`Config::hands_on_reanalyses`]);
    /// * compressions, pauses and time on chest are recorded during CPR.
    ///
    /// When false (or with no sensor fitted) the device behaves exactly as
    /// before compression sensing: heart signal only. Default: true. The
    /// switch exists so that the effect can be measured.
    pub chest_sensing: bool,
    /// If no impedance sample arrives for this long, the chest counts as
    /// "cannot tell", ms. Provisional: 100 ms (about 12 samples).
    pub impedance_stale_ms: u32,
    /// A compression-sensing channel that has been missing, saturated, out of
    /// range or stuck for this long while the pads read as on counts as
    /// failed, ms. The device then records it and carries on from the heart
    /// signal alone (as with [`Config::chest_sensing`] off) rather than
    /// faulting: a failed sensing channel must not stop every shock (no
    /// single fault may block an advised shock, SAFETY.md). Until then
    /// "cannot tell" holds the analysis. Provisional: 5 s, longer than the
    /// time the front end is saturated after a shock.
    pub chest_channel_fail_ms: u32,
    /// A failed channel that has been good again for this long is used
    /// again, ms. Provisional: 3 s.
    pub chest_channel_recover_ms: u32,
    /// During CPR, say "Continue CPR." if compression sensing has seen the
    /// chest still, with no compressions, for this long (and again after the
    /// same time), ms. 0 switches it off.
    ///
    /// Default: on, 15 s. Guidelines ask for pauses under 10 s (UNVERIFIED
    /// against the current guideline text). In the development half of the
    /// Paris recordings the pauses between runs of compressions (breaths,
    /// analyses, shocks) had a median of 3 s and 90 % were under 12 s
    /// (aed-cpr-eval, after-the-fact reference), so 15 s is past nearly all
    /// normal pauses, and a responder who has stopped for that long has most
    /// likely stopped for good (tired, confused, distracted). The metronome keeps running regardless. Nothing is said
    /// while sensing "cannot tell", and no "faster/slower" prompts are given:
    /// the evidence for real-time feedback prompts is weak and too many
    /// prompts is a known risk (research notes on CPR feedback).
    pub no_compression_prompt_ms: u32,
    /// During CPR, pauses between compressions at least this long are
    /// recorded in the event record, ms. Provisional: 2 s.
    pub pause_record_ms: u32,
    /// How many times in one analysis cycle a shock held back because the
    /// chest was not still leads to a fresh analysis instead of CPR.
    /// Provisional: 1 (see the aed-core README for why).
    pub hands_on_reanalyses: u8,
    /// Periodic self-test (decision D-15, confirmed by the owner): the
    /// low-voltage check runs every this many seconds. 86,400 s: once a day.
    /// Seconds, on the low-power timer that keeps running while the device
    /// is off (see [`crate::schedule`]).
    pub daily_test_interval_s: u32,
    /// Periodic self-test (D-15): the full-energy test into the internal
    /// load runs every this many seconds. 604,800 s: once a week. A passed
    /// full-energy test also counts as that day's low-voltage check.
    pub full_test_interval_s: u32,
    /// A periodic test that has not passed for its interval plus this many
    /// seconds is overdue, and the device shows "not ready". Provisional:
    /// 86,400 s (one day), so one missed day is tolerated but two are not.
    pub test_overdue_grace_s: u32,
    /// After a failed low-voltage check, try it again after this many
    /// seconds. Provisional: 3,600 s (one hour): a fault that clears (for
    /// example a cold battery warming up) is seen to clear within the hour.
    pub failed_daily_retry_s: u32,
    /// After a failed full-energy test, try it again after this many
    /// seconds. Provisional: 86,400 s (one day). Not sooner, because every
    /// full-energy test wears the capacitor (the reason D-15 chose weekly).
    pub failed_full_retry_s: u32,
    /// While a due test is put off (device in use, or for the full-energy
    /// test: too cold or hot, or battery not good), wake again after this
    /// many seconds to see whether it can run. Provisional: 3,600 s.
    pub deferred_recheck_s: u32,
    /// The full-energy test runs only between these temperatures, degrees
    /// Celsius (inside the device). Provisional: 0 to 45 °C, a cautious
    /// range for charging the capacitor at full energy from lithium-ion
    /// cells; UNVERIFIED, to be set from the chosen cells' and capacitor's
    /// data sheets. The daily low-voltage check runs at any temperature: if
    /// the device does not work in the cold, that is what it should report.
    pub full_test_min_temp_c: i8,
    /// See [`Config::full_test_min_temp_c`].
    pub full_test_max_temp_c: i8,
}

impl Default for Config {
    /// The provisional starting values listed on each field.
    fn default() -> Self {
        Config {
            cpr_period_ms: 120_000,
            metronome_per_min: 110,
            countdown_ms: 3_000,
            charge_during_analysis: true,
            charge_time_limit_ms: 12_000,
            charge_limit_reference_voltage: Volts(2_000),
            min_charge_time_limit_ms: 1_000,
            max_charged_ms: 25_000,
            disarm_time_limit_ms: 3_000,
            empty_voltage: Volts(50),
            at_plan_percent: 1,
            at_plan_abs_volts: Volts(12),
            agree_percent: 1,
            agree_abs_volts: Volts(12),
            overvoltage_limit: Volts(2_200),
            heartbeat_period_ms: 50,
            heartbeat_timeout_ms: 250,
            permit_window_ms: 200,
            disagreement_persist_ms: 500,
            pad_debounce_ms: 200,
            analysis_time_limit_ms: 12_000,
            hands_off_reminder_ms: 3_000,
            confirmation_max_age_ms: 5_000,
            delivery_timeout_ms: 1_000,
            self_test_voltage: Volts(400),
            self_test_link_wait_ms: 1_000,
            self_test_agree_wait_ms: 500,
            attach_pads_repeat_ms: 15_000,
            press_pads_repeat_ms: 15_000,
            cancel_stuck_ms: 10_000,
            fault_reminder_ms: 30_000,
            chest_sensing: true,
            impedance_stale_ms: 100,
            chest_channel_fail_ms: 5_000,
            chest_channel_recover_ms: 3_000,
            no_compression_prompt_ms: 15_000,
            pause_record_ms: 2_000,
            hands_on_reanalyses: 1,
            daily_test_interval_s: 86_400,
            full_test_interval_s: 604_800,
            test_overdue_grace_s: 86_400,
            failed_daily_retry_s: 3_600,
            failed_full_retry_s: 86_400,
            deferred_recheck_s: 3_600,
            full_test_min_temp_c: 0,
            full_test_max_temp_c: 45,
        }
    }
}

/// Why a `Config` was refused. The controllers will not start with a
/// configuration whose numbers contradict each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// The metronome rate is outside 100 to 120 per minute.
    MetronomeRateOutOfRange,
    /// A time limit that must be above zero is zero.
    ZeroTimeLimit,
    /// The longest-charged limit is not longer than the longest normal shock
    /// cycle (the longer of charging and, when charging during analysis, the
    /// analysis limit; plus countdown, delivery and emptying), so a normal
    /// shock could be cut off.
    MaxChargedTooShort,
    /// The heartbeat timeout is not longer than the heartbeat period.
    HeartbeatTimeoutTooShort,
    /// The self-test voltage is so low that a reading that counts as
    /// "charged to the test voltage" could come from a capacitor that is
    /// really empty, once sensor error is allowed for.
    SelfTestVoltageTooLow,
    /// The two sensors' allowed error is zero: two real sensors never read
    /// exactly the same, so every rescue would fault.
    SensorAllowanceZero,
    /// The two sensors' allowed error is so wide that a dead sensor reading
    /// zero would still "agree" with a capacitor charged to the self-test
    /// voltage, so the cross-check would catch nothing; or so wide that no
    /// reading could ever count as empty.
    SensorAllowanceTooWide,
    /// The "at the planned voltage" window is narrower than one sensor's
    /// allowed error, so a sensor within its allowance would cause false
    /// "charge off target" faults.
    AtPlanNarrowerThanSensorError,
    /// The self-test voltage is above the overvoltage limit.
    SelfTestVoltageTooHigh,
    /// The CPR period is not longer than the analysis time limit (sanity check).
    CprPeriodTooShort,
    /// A compression-sensing time is zero, the "Continue CPR" prompt time is
    /// shorter than 5 s (prompting sooner would talk over every breath
    /// pause), or the channel failure time is not longer than the
    /// stale-sample time.
    ChestSensingTimes,
    /// A periodic self-test time is zero, the full-energy interval is
    /// shorter than the daily one, a retry is longer than its own test's
    /// interval, or the full-energy test's temperature range is empty.
    SelfTestSchedule,
}

impl Config {
    /// Check that the numbers do not contradict each other.
    ///
    /// # Errors
    /// Returns the first problem found.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(100..=120).contains(&self.metronome_per_min) {
            return Err(ConfigError::MetronomeRateOutOfRange);
        }
        let must_be_positive = [
            self.cpr_period_ms,
            self.countdown_ms,
            self.charge_time_limit_ms,
            self.max_charged_ms,
            self.disarm_time_limit_ms,
            self.heartbeat_period_ms,
            self.heartbeat_timeout_ms,
            self.permit_window_ms,
            self.disagreement_persist_ms,
            self.analysis_time_limit_ms,
            self.hands_off_reminder_ms,
            self.confirmation_max_age_ms,
            self.delivery_timeout_ms,
            self.self_test_link_wait_ms,
            self.self_test_agree_wait_ms,
            self.attach_pads_repeat_ms,
            self.press_pads_repeat_ms,
            self.cancel_stuck_ms,
            self.min_charge_time_limit_ms,
            u32::from(self.charge_limit_reference_voltage.0),
            self.fault_reminder_ms,
        ];
        if must_be_positive.contains(&0) {
            return Err(ConfigError::ZeroTimeLimit);
        }
        // With charging during analysis the capacitor may already be
        // charged when a late "shock" answer arrives at the analysis limit.
        let before_countdown = if self.charge_during_analysis {
            self.charge_time_limit_ms.max(self.analysis_time_limit_ms)
        } else {
            self.charge_time_limit_ms
        };
        let normal_shock = before_countdown
            .saturating_add(self.countdown_ms)
            .saturating_add(self.delivery_timeout_ms)
            .saturating_add(self.disarm_time_limit_ms);
        if self.max_charged_ms <= normal_shock {
            return Err(ConfigError::MaxChargedTooShort);
        }
        if self.heartbeat_timeout_ms <= self.heartbeat_period_ms {
            return Err(ConfigError::HeartbeatTimeoutTooShort);
        }
        if self.agree_percent == 0 && self.agree_abs_volts.0 == 0 {
            return Err(ConfigError::SensorAllowanceZero);
        }
        // A dead sensor (reading 0 V) must disagree with a capacitor at the
        // self-test voltage, or the self-test's agreement check proves nothing.
        // And even a perfect reading of 0 V must count as empty.
        if self.readings_agree(Volts(0), self.self_test_voltage) || !self.reads_empty(Volts(0)) {
            return Err(ConfigError::SensorAllowanceTooWide);
        }
        if self.at_plan_percent < self.agree_percent
            || self.at_plan_abs_volts < self.agree_abs_volts
        {
            return Err(ConfigError::AtPlanNarrowerThanSensorError);
        }
        // The lowest reading that counts as charged to the self-test voltage,
        // less one sensor's allowed error, must still be above "empty".
        let lowest_charged = self
            .self_test_voltage
            .0
            .saturating_sub(self.at_plan_allowance(self.self_test_voltage).0);
        let lowest_true =
            lowest_charged.saturating_sub(self.sensor_allowance(Volts(lowest_charged)).0);
        if lowest_true <= self.empty_voltage.0 {
            return Err(ConfigError::SelfTestVoltageTooLow);
        }
        if self.self_test_voltage >= self.overvoltage_limit {
            return Err(ConfigError::SelfTestVoltageTooHigh);
        }
        if self.cpr_period_ms <= self.analysis_time_limit_ms {
            return Err(ConfigError::CprPeriodTooShort);
        }
        let prompt_ok =
            self.no_compression_prompt_ms == 0 || self.no_compression_prompt_ms >= 5_000;
        let chest_ok = self.impedance_stale_ms > 0
            && self.chest_channel_fail_ms > self.impedance_stale_ms
            && self.chest_channel_recover_ms > 0
            && self.pause_record_ms > 0
            && prompt_ok;
        if !chest_ok {
            return Err(ConfigError::ChestSensingTimes);
        }
        let schedule_ok = [
            self.daily_test_interval_s,
            self.full_test_interval_s,
            self.test_overdue_grace_s,
            self.failed_daily_retry_s,
            self.failed_full_retry_s,
            self.deferred_recheck_s,
        ]
        .iter()
        .all(|t| *t > 0)
            && self.full_test_interval_s >= self.daily_test_interval_s
            && self.failed_daily_retry_s <= self.daily_test_interval_s
            && self.failed_full_retry_s <= self.full_test_interval_s
            && self.full_test_min_temp_c < self.full_test_max_temp_c;
        if !schedule_ok {
            return Err(ConfigError::SelfTestSchedule);
        }
        Ok(())
    }

    /// The charge time limit for a charge to `target`, ms: the full
    /// [`Config::charge_time_limit_ms`] scaled by the stored energy, which
    /// grows with the square of the voltage, relative to
    /// [`Config::charge_limit_reference_voltage`], and never less than
    /// [`Config::min_charge_time_limit_ms`] nor more than the full limit.
    ///
    /// For the 400 V switch-on self-test: 12 s × (400 / 2,000)² = 0.48 s,
    /// so the 1 s floor applies (finding F-7: a weak charger is reported
    /// about 1 s into the self-test, not 12 s). For a full-energy test at
    /// 1,950 V: 12 s × 0.95 = 11.4 s.
    #[must_use]
    pub fn charge_time_limit_for(&self, target: Volts) -> u32 {
        let full = u64::from(self.charge_time_limit_ms);
        let v = u64::from(target.0);
        let reference = u64::from(self.charge_limit_reference_voltage.0);
        let scaled = full
            .saturating_mul(v.saturating_mul(v))
            .checked_div(reference.saturating_mul(reference))
            .unwrap_or(full);
        let scaled = u32::try_from(scaled).unwrap_or(self.charge_time_limit_ms);
        scaled
            .max(self.min_charge_time_limit_ms)
            .min(self.charge_time_limit_ms)
    }

    /// Time between metronome beats, in milliseconds (545 ms at 110 per minute).
    #[must_use]
    pub fn metronome_interval_ms(&self) -> u32 {
        60_000u32
            .checked_div(u32::from(self.metronome_per_min))
            .unwrap_or(600)
    }
}

/// `percent` % of `v` plus `abs`, in volts, rounded up and capped at the
/// largest voltage the type can hold. Never panics.
fn allowance(v: Volts, percent: u8, abs: Volts) -> Volts {
    let part = (u32::from(v.0) * u32::from(percent)).div_ceil(100);
    let total = part.saturating_add(u32::from(abs.0));
    Volts(u16::try_from(total).unwrap_or(u16::MAX))
}

impl Config {
    /// The error one voltage sensor is allowed at reading `v`:
    /// [`Config::agree_percent`] % of it plus [`Config::agree_abs_volts`].
    #[must_use]
    pub fn sensor_allowance(&self, v: Volts) -> Volts {
        allowance(v, self.agree_percent, self.agree_abs_volts)
    }

    /// True if two independent readings of the same capacitor agree: they
    /// differ by no more than two sensors' allowed error (taken at the higher
    /// reading), so two sensors with opposite errors inside their allowance
    /// always agree.
    #[must_use]
    pub fn readings_agree(&self, a: Volts, b: Volts) -> bool {
        let budget = u32::from(self.sensor_allowance(a.max(b)).0).saturating_mul(2);
        u32::from(a.0.abs_diff(b.0)) <= budget
    }

    /// How far a reading may be from the planned voltage `plan` and still
    /// count as "at the planned voltage": [`Config::at_plan_percent`] % of
    /// the plan plus [`Config::at_plan_abs_volts`].
    #[must_use]
    pub fn at_plan_allowance(&self, plan: Volts) -> Volts {
        allowance(plan, self.at_plan_percent, self.at_plan_abs_volts)
    }

    /// True if one controller's own reading counts as at the planned voltage.
    #[must_use]
    pub fn is_at_plan(&self, reading: Volts, plan: Volts) -> bool {
        within(reading, plan, self.at_plan_allowance(plan))
    }

    /// True if one controller's own reading counts as charged to `plan`
    /// (no lower than the bottom of the "at the planned voltage" window).
    #[must_use]
    pub fn reached_plan(&self, reading: Volts, plan: Volts) -> bool {
        reading.0 >= plan.0.saturating_sub(self.at_plan_allowance(plan).0)
    }

    /// True if a reading shows the capacitor empty even if the sensor reads
    /// low by its full allowed error: reading plus allowance below
    /// [`Config::empty_voltage`].
    #[must_use]
    pub fn reads_empty(&self, reading: Volts) -> bool {
        u32::from(reading.0) + u32::from(self.sensor_allowance(reading).0)
            < u32::from(self.empty_voltage.0)
    }
}

/// True if `a` and `b` differ by no more than `tolerance`.
#[must_use]
pub fn within(a: Volts, b: Volts, tolerance: Volts) -> bool {
    a.0.abs_diff(b.0) <= tolerance.0
}
