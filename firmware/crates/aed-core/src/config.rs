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
    /// Length of the "stand clear" countdown before an automatic shock.
    /// The supervisor also requires the main controller to have been counting
    /// down for at least this long, by the supervisor's own clock.
    /// Provisional: 5 s.
    pub countdown_ms: u32,
    /// Longest the capacitor may take to charge before it counts as a fault.
    /// Provisional: 12 s (the requirement is 8 s or less; this leaves margin).
    pub charge_time_limit_ms: u32,
    /// Longest the capacitor may stay charged (from the start of charging) before
    /// it must be emptied. Both controllers enforce this separately.
    /// Provisional: 25 s. Must be longer than charging + countdown + delivery.
    pub max_charged_ms: u32,
    /// Longest emptying the capacitor may take. If it is not empty by then, that
    /// is the most serious fault. Provisional: 3 s.
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
    /// plan when a shock fires, which sets the delivered energy.
    /// Provisional: 2 % (see `at_plan_abs_volts` for the reasoning).
    pub at_plan_percent: u8,
    /// The fixed part of the "at the planned voltage" window, volts.
    /// Provisional: 20 V. With 2 % this gives 59 V at 1,950 V (adult),
    /// 43 V at 1,150 V (child) and 28 V at 400 V (self-test): 10 V more than
    /// the error of a sensor accurate to ±2 % ± 10 V, the accuracy these
    /// defaults assume.
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
    /// Provisional: 2 % + 15 V. ASSUMES each real sensor is accurate to
    /// ±2 % ± 10 V; the extra 5 V per sensor covers rounding and the
    /// main controller's reading being up to one heartbeat old while the
    /// capacitor charges. Not from any part's datasheet: no sensor is chosen.
    pub agree_percent: u8,
    /// The fixed part of each sensor's allowed error, volts. Provisional: 15 V.
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
    /// How often the fault prompt is repeated in the Fault state. Provisional: 30 s.
    pub fault_reminder_ms: u32,
}

impl Default for Config {
    /// The provisional starting values listed on each field.
    fn default() -> Self {
        Config {
            cpr_period_ms: 120_000,
            metronome_per_min: 110,
            countdown_ms: 5_000,
            charge_time_limit_ms: 12_000,
            max_charged_ms: 25_000,
            disarm_time_limit_ms: 3_000,
            empty_voltage: Volts(50),
            at_plan_percent: 2,
            at_plan_abs_volts: Volts(20),
            agree_percent: 2,
            agree_abs_volts: Volts(15),
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
            fault_reminder_ms: 30_000,
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
    /// The longest-charged limit is not longer than charging + countdown + delivery,
    /// so a normal shock would be cut off.
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
            self.fault_reminder_ms,
        ];
        if must_be_positive.contains(&0) {
            return Err(ConfigError::ZeroTimeLimit);
        }
        let normal_shock = self
            .charge_time_limit_ms
            .saturating_add(self.countdown_ms)
            .saturating_add(self.delivery_timeout_ms);
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
        let lowest_true = lowest_charged.saturating_sub(self.sensor_allowance(Volts(lowest_charged)).0);
        if lowest_true <= self.empty_voltage.0 {
            return Err(ConfigError::SelfTestVoltageTooLow);
        }
        if self.self_test_voltage >= self.overvoltage_limit {
            return Err(ConfigError::SelfTestVoltageTooHigh);
        }
        if self.cpr_period_ms <= self.analysis_time_limit_ms {
            return Err(ConfigError::CprPeriodTooShort);
        }
        Ok(())
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
