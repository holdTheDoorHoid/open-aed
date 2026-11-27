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
    pub empty_voltage: Volts,
    /// How close a voltage reading must be to count as "at the planned voltage",
    /// and how closely the two controllers' readings must agree. Provisional: 50 V.
    pub voltage_tolerance: Volts,
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
    /// Longest an analysis may take. If no decision arrives in time the device
    /// treats it as "not sure" and goes to CPR. Provisional: 12 s.
    pub analysis_time_limit_ms: u32,
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
            voltage_tolerance: Volts(50),
            overvoltage_limit: Volts(2_200),
            heartbeat_period_ms: 50,
            heartbeat_timeout_ms: 250,
            permit_window_ms: 200,
            disagreement_persist_ms: 500,
            pad_debounce_ms: 200,
            analysis_time_limit_ms: 12_000,
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
    /// The "empty" voltage is not clearly below the self-test voltage.
    SelfTestVoltageTooLow,
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
        if self.self_test_voltage.0
            <= self
                .empty_voltage
                .0
                .saturating_add(self.voltage_tolerance.0)
        {
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

/// True if `a` and `b` differ by no more than `tolerance`.
#[must_use]
pub fn within(a: Volts, b: Volts, tolerance: Volts) -> bool {
    a.0.abs_diff(b.0) <= tolerance.0
}
