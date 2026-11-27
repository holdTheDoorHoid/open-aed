// SPDX-License-Identifier: GPL-3.0-or-later
//! The device while it is "off": one wake handler that runs the heat record
//! (D-48, D-52), the periodic self-test schedule (D-15, D-49) and the
//! capacitor wear count (D-53), and gives ONE readiness result.
//!
//! NOT A MEDICAL DEVICE. Pure logic; no hardware has run it.
//!
//! # One wake, two jobs
//!
//! The low-power timer that wakes the device for its self-tests
//! ([`TestSchedule`]) also wakes it to log its temperature: every 5 minutes
//! up to 40 °C, every minute above (parts study R-H2). [`Standby::on_wake`]
//! does both and asks for the sooner of the two next wakes. The board keeps
//! calling it while the device is in use, too (logging never stops).
//!
//! # What survives power loss
//!
//! Two kinds of memory are assumed:
//!
//! * memory kept alive by the power domain while the device is "off" (the
//!   [`TestSchedule`] lives there, as before);
//! * memory that survives the battery being taken out ([`DurableStore`]):
//!   the heat record and pad heat total ([`HeatMonitor`]) and the capacitor
//!   wear count ([`CapacitorWear`]). The board saves them after every wake
//!   and every test ([`Standby::save`]). If they cannot be read back
//!   ([`Standby::restore`]), the device is "not ready" until a service
//!   action, because the pad heat, the wear and any overheat are unknown.
//!
//! The byte layout in memory, its checksum and a second copy (so that a
//! power loss in the middle of a write never loses both) are the board's
//! job; they are a requirement on the board-support layer, not decided here.

use crate::config::{Config, ConfigError};
use crate::hal::BatteryStatus;
use crate::heat::{
    battery_charge_check, capacitor_charge_check, ChargeContext, DeciC, HeatInhibit, HeatMonitor,
    SensorSet, ServiceCheck, ServiceRefusal, HEAT_LOG_CAPACITY,
};
use crate::readiness::{DeviceReadiness, Reason, ReasonSet};
use crate::schedule::{
    Deferral, PeriodicOutcome, PeriodicTest, TestGate, TestSchedule, WakeDecision, WakeInputs,
};
use crate::selftest::{SelfTestFailure, SelfTestPass};

/// Full charges of the capacitor over its life (D-53). Survives power loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapacitorWear {
    /// Full-energy periodic tests that charged the capacitor.
    pub test_charges: u32,
    /// Charges in rescues (one per analysis with charging during analysis,
    /// D-29, plus any charge after a shock was advised).
    pub rescue_charges: u32,
}

/// Where the capacitor is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WearStatus {
    /// Below the "service soon" point.
    Ok,
    /// At or above [`Config::capacitor_service_soon_percent`] of the life:
    /// advisory.
    ServiceSoon,
    /// At or above [`Config::capacitor_life_charges`]: service due. The
    /// device still shocks (see the README).
    ServiceDue,
}

impl CapacitorWear {
    /// Every full charge so far.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.test_charges.saturating_add(self.rescue_charges)
    }

    /// Where the capacitor is in its life.
    #[must_use]
    pub fn status(&self, cfg: &Config) -> WearStatus {
        let total = u64::from(self.total());
        let life = u64::from(cfg.capacitor_life_charges);
        let soon = life * u64::from(cfg.capacitor_service_soon_percent) / 100;
        if total >= life {
            WearStatus::ServiceDue
        } else if total >= soon {
            WearStatus::ServiceSoon
        } else {
            WearStatus::Ok
        }
    }
}

/// Everything that must survive the battery being taken out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DurableRecord<const N: usize = HEAT_LOG_CAPACITY> {
    /// The heat record, the heat latches and the pad heat total.
    pub heat: HeatMonitor<N>,
    /// The capacitor wear count.
    pub wear: CapacitorWear,
}

/// Why the durable memory could not be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    /// Nothing has ever been written (a new device).
    Empty,
    /// What was read back failed its checksum (both copies).
    Corrupt,
    /// The write did not complete.
    WriteFailed,
}

/// Memory that survives power loss, holding a [`DurableRecord`]. The board
/// implements it (layout, checksum, two alternating copies).
pub trait DurableStore<const N: usize> {
    /// Write the record.
    ///
    /// # Errors
    /// If the write did not complete.
    fn save(&mut self, record: &DurableRecord<N>) -> Result<(), StoreError>;
    /// Read the record back into `into`.
    ///
    /// # Errors
    /// [`StoreError::Empty`] on a new device, [`StoreError::Corrupt`] if
    /// neither copy passes its checksum.
    fn load(&mut self, into: &mut DurableRecord<N>) -> Result<(), StoreError>;
}

/// What the board tells [`Standby::on_wake`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandbyInputs {
    /// The low-power timer, seconds.
    pub now_s: u32,
    /// The board knows the timer restarted since the last wake.
    pub clock_restarted: bool,
    /// The device is switched on.
    pub in_use: bool,
    /// The device is on its charging dock.
    pub on_charge: bool,
    /// The power monitor's view of the battery, without load.
    pub battery: BatteryStatus,
    /// Every temperature reading, the supervisor's included.
    pub temperatures: SensorSet,
    /// The identity the pad connector reports (`None`: no pads, or the
    /// hardware cannot tell). A different identity starts a new pad heat
    /// total.
    pub pad_set_id: Option<u32>,
}

/// What [`Standby::on_wake`] decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandbyDecision {
    /// The schedule's decision (test to run, test put off, next wake).
    pub tests: WakeDecision,
    /// Wake again within this many seconds (the sooner of the schedule's
    /// and the heat record's).
    pub next_wake_in_s: u32,
    /// The dock may charge the battery (cells trusted and within 0-45 °C).
    /// The power hardware must obey it.
    pub battery_charging_allowed: bool,
    /// Why the battery may not be charged, if it may not.
    pub battery_inhibit: Option<HeatInhibit>,
    /// Why the capacitor may not be charged for a test, if it may not.
    pub capacitor_inhibit: Option<HeatInhibit>,
    /// The ready indicator.
    pub readiness: DeviceReadiness,
    /// Sound the alarm (the capacitor was not confirmed empty after a test).
    pub alarm: bool,
    /// A gap in the heat record was found at this wake.
    pub heat_gap_found: bool,
}

/// The device between rescues. See the module notes.
#[derive(Debug, Clone)]
pub struct Standby<const N: usize = HEAT_LOG_CAPACITY> {
    config: Config,
    schedule: TestSchedule,
    durable: DurableRecord<N>,
    capacitor_gate_inhibit: Option<HeatInhibit>,
    battery_allowed: bool,
}

impl<const N: usize> Standby<N> {
    /// A factory-new device.
    ///
    /// # Errors
    /// Refuses a configuration whose numbers contradict each other.
    pub fn new(config: Config) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Standby {
            config,
            schedule: TestSchedule::new(config),
            durable: DurableRecord::default(),
            capacitor_gate_inhibit: None,
            battery_allowed: false,
        })
    }

    /// After a power loss: read the durable record back. The test schedule
    /// starts again ("not ready" until both tests pass, as for a timer
    /// restart) and the heat record notes a gap. If the record cannot be
    /// read, a fresh one is started and marked lost ("not ready" until a
    /// service action). Returns the store's error, if any, alongside.
    ///
    /// # Errors
    /// Refuses a configuration whose numbers contradict each other.
    pub fn restore(
        config: Config,
        store: &mut impl DurableStore<N>,
    ) -> Result<(Self, Option<StoreError>), ConfigError> {
        let mut s = Self::new(config)?;
        let err = store.load(&mut s.durable).err();
        if err.is_some() {
            s.durable = DurableRecord::default();
            s.durable.heat.mark_record_lost();
        } else {
            s.durable.heat.mark_restored();
        }
        Ok((s, err))
    }

    /// Write the durable record.
    ///
    /// # Errors
    /// If the store could not write it.
    pub fn save(&self, store: &mut impl DurableStore<N>) -> Result<(), StoreError> {
        store.save(&self.durable)
    }

    /// The configuration.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// The self-test schedule.
    #[must_use]
    pub const fn schedule(&self) -> &TestSchedule {
        &self.schedule
    }

    /// The heat record.
    #[must_use]
    pub const fn heat(&self) -> &HeatMonitor<N> {
        &self.durable.heat
    }

    /// The capacitor wear count.
    #[must_use]
    pub const fn wear(&self) -> CapacitorWear {
        self.durable.wear
    }

    /// The durable record.
    #[must_use]
    pub const fn durable(&self) -> &DurableRecord<N> {
        &self.durable
    }

    /// The low-power timer woke the device.
    pub fn on_wake(&mut self, w: &StandbyInputs) -> StandbyDecision {
        let cfg = self.config;
        let hw = self
            .durable
            .heat
            .on_wake(&cfg, w.now_s, w.clock_restarted, &w.temperatures, w.pad_set_id);
        let a = hw.assessment;
        if hw.retest_now {
            self.schedule.request_daily_now(w.now_s);
        }
        // Capacitor charging for a test: the 60 °C limits, unknown = too hot.
        self.capacitor_gate_inhibit = capacitor_charge_check(
            &cfg,
            &a,
            ChargeContext::Standby,
            self.capacitor_gate_inhibit.is_some(),
        )
        .err();
        let daily_allowed = self.capacitor_gate_inhibit.is_none();
        let cells_in_window = a.cells_hottest(false, DeciC(0)).is_some_and(|hot| {
            hot <= DeciC::from_c(i16::from(cfg.full_test_max_temp_c))
                && a.cells_coolest()
                    .is_some_and(|cold| cold >= DeciC::from_c(i16::from(cfg.full_test_min_temp_c)))
        });
        let cap_cool = a
            .capacitor(false, DeciC(0))
            .is_some_and(|t| t <= cfg.full_test_max_capacitor);
        let gate = TestGate {
            full_allowed: daily_allowed && cells_in_window && cap_cool,
            daily_allowed,
        };
        // The schedule's own temperature field is not used on this path.
        let hottest_cell = a.cells_hottest(false, DeciC(0)).map_or(i8::MAX, |t| {
            i8::try_from(t.0 / 10).unwrap_or(if t.0 < 0 { i8::MIN } else { i8::MAX })
        });
        let tests = self.schedule.on_wake_gated(
            &WakeInputs {
                now_s: w.now_s,
                clock_restarted: w.clock_restarted,
                in_use: w.in_use,
                on_charge: w.on_charge,
                temperature_c: hottest_cell,
                battery: w.battery,
            },
            gate,
        );
        // Battery charging on the dock: cells within 0-45 °C.
        let battery = battery_charge_check(&cfg, &a, self.battery_allowed);
        self.battery_allowed = battery.is_ok();
        StandbyDecision {
            tests,
            next_wake_in_s: tests.next_wake_in_s.min(hw.next_wake_in_s).max(1),
            battery_charging_allowed: self.battery_allowed,
            battery_inhibit: battery.err(),
            capacitor_inhibit: self.capacitor_gate_inhibit,
            readiness: self.readiness(w.now_s),
            alarm: self.schedule.alarm(),
            heat_gap_found: hw.gap_found,
        }
    }

    /// Seconds until the next wake after a test result was recorded at
    /// `now_s`.
    #[must_use]
    pub fn next_wake_in_s(&self, now_s: u32) -> u32 {
        let heat = if self
            .durable
            .heat
            .hottest_now()
            .is_none_or(|t| t > self.config.heat_normal_max)
        {
            self.config.heat_wake_warm_s
        } else {
            self.config.heat_wake_cool_s
        };
        self.schedule.next_wake_in_s(now_s).min(heat).max(1)
    }

    /// Record the outcome of the test [`Standby::on_wake`] handed out.
    pub fn record(&mut self, now_s: u32, test: PeriodicTest, outcome: &PeriodicOutcome) {
        self.schedule.record(now_s, test, outcome);
        if test == PeriodicTest::Full && outcome.charged {
            self.durable.wear.test_charges = self.durable.wear.test_charges.saturating_add(1);
        }
        let hardware_ok = match outcome.result {
            Ok(_) => true,
            Err(f) => f.is_advisory(),
        };
        if hardware_ok && outcome.capacitor_confirmed_empty {
            self.durable.heat.on_test_passed(now_s);
        }
    }

    /// Record a rescue's switch-on self-test (see
    /// [`TestSchedule::record_power_on`]). A pass also counts as the
    /// re-test heat asks for, unless the self-test skipped its charge for
    /// heat (`charge_skipped`): then it did not test the charger.
    pub fn record_power_on(
        &mut self,
        now_s: u32,
        result: Result<SelfTestPass, SelfTestFailure>,
        charge_skipped: bool,
    ) {
        self.schedule.record_power_on(now_s, result);
        let ok = match result {
            Ok(_) => true,
            Err(f) => f.is_advisory(),
        };
        if ok && !charge_skipped {
            self.durable.heat.on_test_passed(now_s);
        }
    }

    /// Add a rescue's capacitor charges to the wear count (persisted).
    pub fn add_rescue_charges(&mut self, charges: u32) {
        self.schedule.add_rescue_charges(charges);
        self.durable.wear.rescue_charges = self.durable.wear.rescue_charges.saturating_add(charges);
    }

    /// A service action after an overheat (or a lost record).
    ///
    /// # Errors
    /// See [`HeatMonitor::service_cleared`].
    pub fn service_cleared(&mut self, now_s: u32, check: ServiceCheck) -> Result<(), ServiceRefusal> {
        let cfg = self.config;
        self.durable.heat.service_cleared(&cfg, check)?;
        self.schedule.service_reset(now_s);
        Ok(())
    }

    /// New pads fitted on a device whose connector cannot report an identity.
    pub fn pads_replaced(&mut self, pad_set_id: Option<u32>) {
        self.durable.heat.pads_replaced(pad_set_id);
    }

    /// The capacitor was replaced in service: start a new wear count.
    pub fn capacitor_replaced(&mut self) {
        self.durable.wear = CapacitorWear::default();
    }

    /// True if the pads have used up their heat allowance. The board then
    /// reports [`crate::PadCartridge::HeatWorn`] to the rescue (advisory,
    /// like expired pads).
    #[must_use]
    pub fn pads_heat_worn(&self) -> bool {
        self.durable.heat.pads_heat_worn(&self.config)
    }

    /// The one readiness result at timer reading `now_s`.
    #[must_use]
    pub fn readiness(&self, now_s: u32) -> DeviceReadiness {
        let cfg = &self.config;
        let h = &self.durable.heat;
        let mut r: ReasonSet = self.schedule.readiness_reasons(now_s);
        let too_hot_now = h.band() == crate::heat::HeatBand::TooHot
            || h.hottest_now().is_some_and(|t| t > cfg.heat_too_hot_above);
        let flags = [
            (too_hot_now, Reason::TooHotNow),
            (h.cool_causes() & crate::heat::cool_causes::WAS_TOO_HOT != 0, Reason::WasTooHot),
            (
                h.cool_causes() & crate::heat::cool_causes::WARM_TOO_LONG != 0,
                Reason::WarmSpellTooLong,
            ),
            (h.gap_pending(), Reason::HeatRecordGap),
            (h.sensor_fault(), Reason::TempSensorFault),
            (h.service_needed(), Reason::ServiceNeededOverheat),
            (h.pads_after_overheat(), Reason::PadsReplaceAfterOverheat),
            (h.pads_heat_worn(cfg), Reason::PadsHeatWorn),
            (h.record_lost(), Reason::RecordLost),
        ];
        for (on, reason) in flags {
            if on {
                r.insert(reason);
            }
        }
        match self.durable.wear.status(cfg) {
            WearStatus::Ok => {}
            WearStatus::ServiceSoon => r.insert(Reason::CapacitorServiceSoon),
            WearStatus::ServiceDue => r.insert(Reason::CapacitorServiceDue),
        }
        DeviceReadiness { reasons: r }
    }

    /// A due test put off for temperature, if that is why (for reports).
    #[must_use]
    pub fn deferred_for_heat(d: &StandbyDecision) -> bool {
        matches!(d.tests.deferred, Some((_, Deferral::Temperature)))
    }
}
