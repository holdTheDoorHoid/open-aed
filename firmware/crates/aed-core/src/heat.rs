// SPDX-License-Identifier: GPL-3.0-or-later
//! Heat: the temperature sensors, the heat states, the charging limits, the
//! pad heat allowance and the heat record (decisions D-41, D-48, D-51, D-52).
//!
//! NOT A MEDICAL DEVICE. Pure logic: the readings arrive from the board; no
//! sensor has been chosen and nothing here has run on hardware.
//!
//! # Where temperature is measured
//!
//! Six points (parts study §8A, R-H1): each of the two battery cells, the
//! capacitor body, next to the stored pads (the pads themselves cannot be
//! measured), the main board, and a second sensor on the capacitor body that
//! only the supervisor reads ([`TempPoint::Supervisor`]), so that a failed
//! main controller cannot hide an overheat. Readings are in tenths of a
//! degree ([`DeciC`]).
//!
//! # A sensor that cannot be trusted
//!
//! [`assess`] checks every reading. A reading is not used when:
//!
//! * it is missing (the sensor did not answer);
//! * it is outside the range a sensor can read, −40 to +100 °C (R-H1): a
//!   broken wire or a shorted sensor reads at one end;
//! * it has not changed at all for [`Config::temp_stuck_wakes`] wakes while
//!   another sensor moved by at least [`Config::temp_stuck_min_change`]
//!   (standby only; a real sensor always flickers by a tenth);
//! * it differs from the middle (median) of the other readings by more than
//!   [`Config::temp_max_gradient`] (provisional 15 °C): the inside of a box
//!   this size does not hold a gradient like that for long, so one of the
//!   sensors is wrong. With only two usable readings that disagree, both are
//!   distrusted, because there is no way to tell which is right.
//!
//! **Fail safe, outside a rescue:** a point with no trusted reading counts as
//! too hot for charging the battery or running a test, and any distrusted
//! sensor makes the device "not ready" (reason
//! [`crate::readiness::Reason::TempSensorFault`]).
//!
//! **In a rescue** a single failed sensor must not block every shock
//! (SAFETY.md §4: no single fault may block an advised shock). So in a
//! rescue a point with no trusted reading takes the hottest trusted reading
//! anywhere in the device plus [`Config::failed_sensor_margin`] (5 °C); and
//! if no sensor at all can be trusted, the device does not hold the shock
//! back for heat. It records that it could not tell. This mirrors D-38
//! (compression sensing that fails is recorded, not allowed to stop shocks).
//!
//! # The heat states (D-48)
//!
//! Driven by the hottest trusted reading, with hysteresis
//! ([`Config::heat_hysteresis`], 2 °C) so a reading sitting on a line does
//! not flap:
//!
//! * **Normal**, up to 40 °C.
//! * **Warm**, 40 to 60 °C: still ready. The time spent continuously above
//!   50 °C is counted; after 8 hours the device is "not ready" until it has
//!   cooled and passed a self-test.
//! * **Too hot**, above 60 °C: "not ready" at once. Ready again only once
//!   every sensor has read below 45 °C for 30 minutes AND a self-test has
//!   passed after that.
//! * **Service needed**, any trusted reading above 70 °C: latched. Cleared
//!   only by a service action ([`HeatMonitor::service_cleared`]) that
//!   reports a full self-test and a battery health check passed; new pads are
//!   recommended until a different pad set is fitted.
//!
//! The cold side: D-48 sets only one cold limit, battery charging at 0 °C
//! or above. No other cold limit is invented here; −20 °C is the operating
//! minimum for use, not a charging limit.

use crate::config::Config;

/// A temperature in tenths of a degree Celsius (215 is 21.5 °C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeciC(pub i16);

impl DeciC {
    /// Whole degrees Celsius.
    #[must_use]
    pub const fn from_c(c: i16) -> Self {
        DeciC(c.saturating_mul(10))
    }

    /// `self` plus `other`, saturating.
    #[must_use]
    pub const fn plus(self, other: DeciC) -> Self {
        DeciC(self.0.saturating_add(other.0))
    }

    /// `self` minus `other`, saturating.
    #[must_use]
    pub const fn minus(self, other: DeciC) -> Self {
        DeciC(self.0.saturating_sub(other.0))
    }
}

/// Where a temperature is measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempPoint {
    /// Battery cell 1 (the fuel gauge's own input).
    Cell1,
    /// Battery cell 2.
    Cell2,
    /// The storage capacitor's body (main controller's sensor).
    Capacitor,
    /// Next to the stored pads (the pads' own temperature cannot be read).
    Pads,
    /// The main board (the processor's own sensor is enough).
    Board,
    /// The supervisor's own sensor, also on the capacitor body, read only by
    /// the supervisor through its own converter.
    Supervisor,
}

/// How many temperature points there are.
pub const TEMP_POINTS: usize = 6;

impl TempPoint {
    /// Every point, in the order used by [`SensorSet`].
    pub const ALL: [TempPoint; TEMP_POINTS] = [
        TempPoint::Cell1,
        TempPoint::Cell2,
        TempPoint::Capacitor,
        TempPoint::Pads,
        TempPoint::Board,
        TempPoint::Supervisor,
    ];

    /// Position in [`SensorSet::readings`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            TempPoint::Cell1 => 0,
            TempPoint::Cell2 => 1,
            TempPoint::Capacitor => 2,
            TempPoint::Pads => 3,
            TempPoint::Board => 4,
            TempPoint::Supervisor => 5,
        }
    }

    /// Short name used in the exported record.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            TempPoint::Cell1 => "cell1",
            TempPoint::Cell2 => "cell2",
            TempPoint::Capacitor => "capacitor",
            TempPoint::Pads => "pads",
            TempPoint::Board => "board",
            TempPoint::Supervisor => "supervisor",
        }
    }
}

/// One reading from every point. `None`: the sensor did not answer.
///
/// The main controller's own hardware fills the first five; the
/// supervisor's reading reaches the main controller in the supervisor's
/// report (in a rescue) or from the board (in standby).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SensorSet {
    /// Readings in [`TempPoint::ALL`] order.
    pub readings: [Option<DeciC>; TEMP_POINTS],
}

impl SensorSet {
    /// Every point reads `t` (for tests and the simulator).
    #[must_use]
    pub const fn uniform(t: DeciC) -> Self {
        SensorSet {
            readings: [Some(t); TEMP_POINTS],
        }
    }

    /// The reading at `p`.
    #[must_use]
    pub fn get(&self, p: TempPoint) -> Option<DeciC> {
        self.readings.get(p.index()).copied().flatten()
    }

    /// Set the reading at `p`.
    pub fn set(&mut self, p: TempPoint, t: Option<DeciC>) {
        if let Some(slot) = self.readings.get_mut(p.index()) {
            *slot = t;
        }
    }
}

/// Why a reading is not trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFault {
    /// No answer from the sensor.
    Missing,
    /// Outside the range a sensor can read (−40 to +100 °C by default).
    OutOfRange,
    /// Unchanged for many wakes while the others moved.
    Stuck,
    /// Too far from the other readings.
    Disagrees,
}

/// The result of checking one set of readings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Assessment {
    /// The trusted readings (`None` where not trusted or missing).
    pub valid: [Option<DeciC>; TEMP_POINTS],
    /// Why each untrusted reading is not used.
    pub faults: [Option<SensorFault>; TEMP_POINTS],
}

impl Assessment {
    /// The trusted reading at `p`.
    #[must_use]
    pub fn valid_at(&self, p: TempPoint) -> Option<DeciC> {
        self.valid.get(p.index()).copied().flatten()
    }

    /// The fault at `p`, if its reading is not trusted.
    #[must_use]
    pub fn fault_at(&self, p: TempPoint) -> Option<SensorFault> {
        self.faults.get(p.index()).copied().flatten()
    }

    /// The hottest trusted reading.
    #[must_use]
    pub fn hottest(&self) -> Option<DeciC> {
        self.valid.iter().flatten().copied().max()
    }

    /// The coolest trusted reading.
    #[must_use]
    pub fn coolest(&self) -> Option<DeciC> {
        self.valid.iter().flatten().copied().min()
    }

    /// True if any reading is not trusted.
    #[must_use]
    pub fn any_fault(&self) -> bool {
        self.faults.iter().any(Option::is_some)
    }

    /// The temperature to use at `p`. Its trusted reading; or, in a rescue
    /// (`substitute`), the hottest trusted reading anywhere plus `margin`;
    /// otherwise unknown.
    #[must_use]
    pub fn effective(&self, p: TempPoint, substitute: bool, margin: DeciC) -> Option<DeciC> {
        self.valid_at(p)
            .or_else(|| self.hottest().filter(|_| substitute).map(|t| t.plus(margin)))
    }

    /// The capacitor's temperature: the hotter of its two sensors (the main
    /// controller's and the supervisor's).
    #[must_use]
    pub fn capacitor(&self, substitute: bool, margin: DeciC) -> Option<DeciC> {
        let main = self.valid_at(TempPoint::Capacitor);
        let sup = self.valid_at(TempPoint::Supervisor);
        match (main, sup) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => self.effective(TempPoint::Capacitor, substitute, margin),
        }
    }

    /// The hotter cell. Unknown if either is unknown (outside a rescue).
    #[must_use]
    pub fn cells_hottest(&self, substitute: bool, margin: DeciC) -> Option<DeciC> {
        let a = self.effective(TempPoint::Cell1, substitute, margin)?;
        let b = self.effective(TempPoint::Cell2, substitute, margin)?;
        Some(a.max(b))
    }

    /// The colder cell (trusted readings only; unknown if either is unknown).
    #[must_use]
    pub fn cells_coolest(&self) -> Option<DeciC> {
        let a = self.valid_at(TempPoint::Cell1)?;
        let b = self.valid_at(TempPoint::Cell2)?;
        Some(a.min(b))
    }
}

/// Check a set of readings (see the module notes). `stuck` marks readings
/// the caller's history shows to be stuck (standby only).
#[must_use]
pub fn assess(cfg: &Config, set: &SensorSet, stuck: &[bool; TEMP_POINTS]) -> Assessment {
    let mut a = Assessment::default();
    for ((raw, is_stuck), (valid, fault)) in set
        .readings
        .iter()
        .zip(stuck.iter())
        .zip(a.valid.iter_mut().zip(a.faults.iter_mut()))
    {
        match raw {
            None => *fault = Some(SensorFault::Missing),
            Some(t) if *t < cfg.temp_plausible_min || *t > cfg.temp_plausible_max => {
                *fault = Some(SensorFault::OutOfRange);
            }
            Some(_) if *is_stuck => *fault = Some(SensorFault::Stuck),
            Some(t) => *valid = Some(*t),
        }
    }
    // Disagreement: compare each usable reading with the median of all.
    let mut sorted = [0i32; TEMP_POINTS];
    let mut n = 0usize;
    for t in a.valid.iter().flatten() {
        if let Some(slot) = sorted.get_mut(n) {
            *slot = i32::from(t.0);
            n += 1;
        }
    }
    let used = sorted.get_mut(..n).unwrap_or(&mut []);
    used.sort_unstable();
    let gradient = i32::from(cfg.temp_max_gradient.0);
    let reject: [bool; TEMP_POINTS] = if n == 2 {
        let spread = used.last().copied().unwrap_or(0) - used.first().copied().unwrap_or(0);
        [spread > gradient; TEMP_POINTS]
    } else if n >= 3 {
        let mid = n / 2;
        let median = if n % 2 == 1 {
            used.get(mid).copied().unwrap_or(0)
        } else {
            (used.get(mid - 1).copied().unwrap_or(0) + used.get(mid).copied().unwrap_or(0)) / 2
        };
        let mut r = [false; TEMP_POINTS];
        for (flag, v) in r.iter_mut().zip(a.valid.iter()) {
            *flag = v.is_some_and(|t| (i32::from(t.0) - median).abs() > gradient);
        }
        r
    } else {
        [false; TEMP_POINTS]
    };
    for ((valid, fault), rej) in a.valid.iter_mut().zip(a.faults.iter_mut()).zip(reject) {
        if rej && valid.is_some() {
            *valid = None;
            *fault = Some(SensorFault::Disagrees);
        }
    }
    a
}

/// Why charging is held back for temperature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatInhibit {
    /// The capacitor is above its working limit (60 °C, D-41).
    CapacitorHot,
    /// A cell is above the limit for charging the capacitor from it (60 °C,
    /// D-48), or for battery charging above 45 °C.
    CellsHot,
    /// A cell is below 0 °C (battery charging only).
    CellsCold,
    /// The temperature that decides it is not known (outside a rescue
    /// only: unknown counts as too hot).
    Unknown,
    /// The supervisor's own reading says too hot (a rescue).
    SupervisorHot,
}

/// Who is asking to charge the capacitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeContext {
    /// A rescue: an untrusted sensor is replaced by the hottest trusted
    /// reading plus a margin, and nothing at all known does not block.
    Rescue,
    /// A periodic test: unknown counts as too hot.
    Standby,
}

/// May the capacitor be charged (for a shock, the switch-on test or a
/// periodic test)? Only with the capacitor at or below
/// [`Config::capacitor_charge_max`] and the cells at or below
/// [`Config::cell_discharge_max`] (60 °C each). `was_inhibited` applies the
/// hysteresis: once held back, the temperature must fall
/// [`Config::charge_limit_hysteresis`] below the limit.
///
/// # Errors
/// Why charging must wait.
pub fn capacitor_charge_check(
    cfg: &Config,
    a: &Assessment,
    ctx: ChargeContext,
    was_inhibited: bool,
) -> Result<(), HeatInhibit> {
    let rescue = ctx == ChargeContext::Rescue;
    if rescue && a.hottest().is_none() {
        // Nothing trusted at all: do not let a sensor failure block a shock.
        return Ok(());
    }
    let margin = cfg.failed_sensor_margin;
    let hyst = if was_inhibited { cfg.charge_limit_hysteresis } else { DeciC(0) };
    let cap = a.capacitor(rescue, margin).ok_or(HeatInhibit::Unknown)?;
    if cap > cfg.capacitor_charge_max.minus(hyst) {
        return Err(HeatInhibit::CapacitorHot);
    }
    let cells = a.cells_hottest(rescue, margin).ok_or(HeatInhibit::Unknown)?;
    if cells > cfg.cell_discharge_max.minus(hyst) {
        return Err(HeatInhibit::CellsHot);
    }
    Ok(())
}

/// May the battery be charged on the dock? Only with both cells trusted and
/// between [`Config::battery_charge_min`] and [`Config::battery_charge_max`]
/// (0 to 45 °C, D-48). Unknown is "no". `was_allowed` applies the
/// hysteresis: once stopped, the cells must come
/// [`Config::charge_limit_hysteresis`] inside the range.
///
/// # Errors
/// Why the battery must not be charged.
pub fn battery_charge_check(cfg: &Config, a: &Assessment, was_allowed: bool) -> Result<(), HeatInhibit> {
    let hyst = if was_allowed { DeciC(0) } else { cfg.charge_limit_hysteresis };
    let hot = a.cells_hottest(false, DeciC(0)).ok_or(HeatInhibit::Unknown)?;
    let cold = a.cells_coolest().ok_or(HeatInhibit::Unknown)?;
    if hot > cfg.battery_charge_max.minus(hyst) {
        return Err(HeatInhibit::CellsHot);
    }
    if cold < cfg.battery_charge_min.plus(hyst) {
        return Err(HeatInhibit::CellsCold);
    }
    Ok(())
}

/// May the capacitor be charged at the START of an analysis (D-29), before
/// any shock is advised? Only at or below [`Config::precharge_max`]
/// (provisional 55 °C). Above it the device charges only once a shock is
/// advised: every analysis that ends without a shock burns a whole charge
/// inside the device (parts study §4), so a long no-shock rescue on a hot
/// day would otherwise heat the device towards its 60 °C limit. Unknown
/// (nothing trusted) allows it, as in a rescue.
#[must_use]
pub fn precharge_allowed(cfg: &Config, a: &Assessment, was_allowed: bool) -> bool {
    let hyst = if was_allowed { DeciC(0) } else { cfg.charge_limit_hysteresis };
    let limit = cfg.precharge_max.minus(hyst);
    let margin = cfg.failed_sensor_margin;
    if a.hottest().is_none() {
        return true;
    }
    let cap = a.capacitor(true, margin);
    let cells = a.cells_hottest(true, margin);
    cap.is_none_or(|t| t <= limit) && cells.is_none_or(|t| t <= limit)
}

/// The capacitor-charge decision with its hysteresis state, for a rescue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChargeGate {
    inhibit: Option<HeatInhibit>,
}

impl ChargeGate {
    /// Re-evaluate. Returns the inhibit now in force, if any.
    pub fn update(&mut self, cfg: &Config, a: &Assessment, ctx: ChargeContext) -> Option<HeatInhibit> {
        self.inhibit = capacitor_charge_check(cfg, a, ctx, self.inhibit.is_some()).err();
        self.inhibit
    }

    /// The inhibit in force, if any.
    #[must_use]
    pub const fn inhibit(&self) -> Option<HeatInhibit> {
        self.inhibit
    }
}

// ----- the heat record ------------------------------------------------------

/// Flags on a heat log entry.
pub mod flags {
    /// Above 60 °C (too hot) at some wake in this period.
    pub const TOO_HOT: u8 = 1;
    /// Above 70 °C (service needed) at some wake in this period.
    pub const SERVICE: u8 = 2;
    /// A gap in the record (missed wakes or the timer restarted) just
    /// before this period: temperature unknown in between.
    pub const GAP_BEFORE: u8 = 4;
    /// At least one sensor was not trusted in this period.
    pub const SENSOR_FAULT: u8 = 8;
    /// The timer restarted before this period: times before it are on an
    /// older count.
    pub const CLOCK_RESTART: u8 = 16;
    /// The warm spell above 50 °C passed its 8 hour limit in this period.
    pub const WARM_TOO_LONG: u8 = 32;
}

/// One period of the heat record (one hour by default).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeatLogEntry {
    /// Timer reading at the first wake of the period, seconds.
    pub start_s: u32,
    /// Hottest trusted reading at any wake in the period (`None` if none).
    pub max: Option<DeciC>,
    /// Coolest trusted reading at any wake in the period.
    pub min: Option<DeciC>,
    /// Wakes in the period.
    pub wakes: u16,
    /// See [`flags`].
    pub flags: u8,
}

/// Default number of entries kept: 2,160, which is 90 days at one entry an
/// hour. About 12 bytes each, so about 26 KB of memory that survives power
/// loss (a board requirement: FRAM-class memory, written at every wake).
pub const HEAT_LOG_CAPACITY: usize = 2_160;

/// A fixed-size heat record, newest entries kept. An entry that is
/// overwritten has already been counted into [`HeatLifetime`], so the
/// highest and lowest temperatures ever seen are never lost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatLog<const N: usize> {
    slots: [Option<HeatLogEntry>; N],
    next: usize,
    len: usize,
    overwritten: u32,
}

impl<const N: usize> Default for HeatLog<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> HeatLog<N> {
    /// An empty record.
    #[must_use]
    pub const fn new() -> Self {
        HeatLog {
            slots: [None; N],
            next: 0,
            len: 0,
            overwritten: 0,
        }
    }

    fn push(&mut self, e: HeatLogEntry) {
        let Some(slot) = self.slots.get_mut(self.next) else {
            self.overwritten = self.overwritten.saturating_add(1);
            return;
        };
        if slot.is_some() {
            self.overwritten = self.overwritten.saturating_add(1);
        }
        *slot = Some(e);
        self.next = self.next.saturating_add(1).checked_rem(N).unwrap_or(0);
        if self.len < N {
            self.len += 1;
        }
    }

    /// Entries kept.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True if nothing is kept.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Entries overwritten because the record was full (their extremes are
    /// in [`HeatLifetime`]).
    #[must_use]
    pub const fn overwritten(&self) -> u32 {
        self.overwritten
    }

    /// The entries, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &HeatLogEntry> {
        let (newer, older) = self
            .slots
            .split_at_checked(self.next)
            .unwrap_or((self.slots.as_slice(), &[]));
        older.iter().chain(newer.iter()).filter_map(Option::as_ref)
    }
}

/// Totals over the life of the device. Never overwritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeatLifetime {
    /// Hottest trusted reading ever.
    pub max: Option<DeciC>,
    /// Coolest trusted reading ever.
    pub min: Option<DeciC>,
    /// Hottest trusted reading per point.
    pub point_max: [Option<DeciC>; TEMP_POINTS],
    /// Coolest trusted reading per point.
    pub point_min: [Option<DeciC>; TEMP_POINTS],
    /// Hottest reading ever, trusted or not (for service).
    pub raw_max: Option<DeciC>,
    /// Seconds with the hottest reading above 40 °C.
    pub above_normal_s: u32,
    /// Seconds above 50 °C.
    pub above_warm_spell_s: u32,
    /// Seconds above 60 °C.
    pub above_too_hot_s: u32,
    /// Times the device became too hot.
    pub too_hot_events: u32,
    /// Times a reading above 70 °C was seen.
    pub service_events: u32,
    /// Gaps in the record.
    pub gaps: u32,
    /// Total length of the gaps whose length is known, seconds.
    pub gap_s: u32,
    /// Gaps of unknown length (the timer restarted).
    pub gaps_unknown_length: u32,
    /// Wakes at which at least one sensor was not trusted.
    pub sensor_fault_wakes: u32,
    /// Pad sets fitted (changes seen).
    pub pad_sets: u32,
}

/// The broad temperature band now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatBand {
    /// Up to 40 °C.
    Normal,
    /// 40 to 60 °C: still ready.
    Warm,
    /// Above 60 °C: not ready.
    TooHot,
}

/// Why the device must cool and pass a self-test before it is ready.
pub mod cool_causes {
    /// It was above 60 °C.
    pub const WAS_TOO_HOT: u8 = 1;
    /// It spent more than 8 hours above 50 °C at a stretch.
    pub const WARM_TOO_LONG: u8 = 2;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct StuckTrack {
    last: Option<i16>,
    wakes: u32,
    others_min: i16,
    others_max: i16,
}

/// What the heat monitor asks of the caller after a wake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeatWake {
    /// Wake again within this many seconds (R-H2: every 5 minutes up to
    /// 40 °C, every minute above).
    pub next_wake_in_s: u32,
    /// Make a self-test due now (the device has cooled after being too hot,
    /// or a gap was found).
    pub retest_now: bool,
    /// A gap in the record was found at this wake.
    pub gap_found: bool,
    /// The checked readings.
    pub assessment: Assessment,
}

/// What a service technician reports when clearing "service needed".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceCheck {
    /// A full-energy self-test passed.
    pub full_self_test_passed: bool,
    /// The battery health check passed (internal resistance against the
    /// cells' own earlier value, and a 24-48 hour self-discharge check;
    /// parts study R-H3).
    pub battery_health_passed: bool,
}

/// Why a service action was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRefusal {
    /// The full-energy self-test has not passed.
    FullTestNotPassed,
    /// The battery health check has not passed.
    BatteryHealthNotPassed,
    /// The device is not below 45 °C now (or its temperature is unknown).
    NotCooled,
}

/// The heat state, the pad heat total and the heat record. Pure logic, no
/// clock of its own: woken by the board with the low-power timer reading
/// (seconds). Everything in it belongs in memory that survives power loss
/// (see [`crate::standby::DurableStore`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatMonitor<const N: usize = HEAT_LOG_CAPACITY> {
    band: HeatBand,
    hottest_now: Option<DeciC>,
    faults_now: bool,
    last_wake_s: Option<u32>,
    asked_interval_s: u32,
    restored: bool,
    warm_since: Option<u32>,
    cool_causes: u8,
    cool_below_since: Option<u32>,
    cooled_at: Option<u32>,
    gap_since: Option<u32>,
    service_needed: bool,
    pads_after_overheat: bool,
    record_lost: bool,
    stuck: [StuckTrack; TEMP_POINTS],
    pad_set: Option<u32>,
    pad_hot_s: u32,
    pad_was_hot: bool,
    lifetime: HeatLifetime,
    current: Option<HeatLogEntry>,
    next_flags: u8,
    log: HeatLog<N>,
}

impl<const N: usize> Default for HeatMonitor<N> {
    fn default() -> Self {
        Self::new()
    }
}

fn opt_max(a: Option<DeciC>, b: DeciC) -> Option<DeciC> {
    Some(a.map_or(b, |x| x.max(b)))
}

fn opt_min(a: Option<DeciC>, b: DeciC) -> Option<DeciC> {
    Some(a.map_or(b, |x| x.min(b)))
}

impl<const N: usize> HeatMonitor<N> {
    /// A new monitor (factory fresh): nothing recorded, no gap.
    #[must_use]
    pub const fn new() -> Self {
        HeatMonitor {
            band: HeatBand::Normal,
            hottest_now: None,
            faults_now: false,
            last_wake_s: None,
            asked_interval_s: 0,
            restored: false,
            warm_since: None,
            cool_causes: 0,
            cool_below_since: None,
            cooled_at: None,
            gap_since: None,
            service_needed: false,
            pads_after_overheat: false,
            record_lost: false,
            stuck: [StuckTrack {
                last: None,
                wakes: 0,
                others_min: 0,
                others_max: 0,
            }; TEMP_POINTS],
            pad_set: None,
            pad_hot_s: 0,
            pad_was_hot: false,
            lifetime: HeatLifetime {
                max: None,
                min: None,
                point_max: [None; TEMP_POINTS],
                point_min: [None; TEMP_POINTS],
                raw_max: None,
                above_normal_s: 0,
                above_warm_spell_s: 0,
                above_too_hot_s: 0,
                too_hot_events: 0,
                service_events: 0,
                gaps: 0,
                gap_s: 0,
                gaps_unknown_length: 0,
                sensor_fault_wakes: 0,
                pad_sets: 0,
            },
            current: None,
            next_flags: 0,
            log: HeatLog::new(),
        }
    }

    /// The record was restored from memory that survives power loss after
    /// the device lost power (or the main controller restarted): the time
    /// since the last wake is unknown, so the next wake records a gap.
    pub fn mark_restored(&mut self) {
        self.restored = true;
    }

    /// The durable record could not be read back (lost or corrupt): the
    /// pad heat total, the capacitor wear and any overheat are unknown.
    /// "Not ready" until a service action.
    pub fn mark_record_lost(&mut self) {
        self.record_lost = true;
        self.restored = true;
    }

    /// The band now.
    #[must_use]
    pub const fn band(&self) -> HeatBand {
        self.band
    }

    /// The hottest trusted reading at the last wake.
    #[must_use]
    pub const fn hottest_now(&self) -> Option<DeciC> {
        self.hottest_now
    }

    /// Latched: a reading above 70 °C was seen.
    #[must_use]
    pub const fn service_needed(&self) -> bool {
        self.service_needed
    }

    /// Why the device must cool and be re-tested (see [`cool_causes`]); 0
    /// if it need not.
    #[must_use]
    pub const fn cool_causes(&self) -> u8 {
        self.cool_causes
    }

    /// A gap in the record not yet followed by a passed self-test.
    #[must_use]
    pub const fn gap_pending(&self) -> bool {
        self.gap_since.is_some()
    }

    /// Seconds the current pad set has spent above 50 °C.
    #[must_use]
    pub const fn pad_hot_s(&self) -> u32 {
        self.pad_hot_s
    }

    /// The identity of the pad set the total belongs to.
    #[must_use]
    pub const fn pad_set(&self) -> Option<u32> {
        self.pad_set
    }

    /// True once the pad set has used up its heat allowance.
    #[must_use]
    pub fn pads_heat_worn(&self, cfg: &Config) -> bool {
        self.pad_hot_s >= cfg.pad_heat_allowance_s
    }

    /// New pads were recommended after an overheat and have not been
    /// replaced yet.
    #[must_use]
    pub const fn pads_after_overheat(&self) -> bool {
        self.pads_after_overheat
    }

    /// The durable record was lost.
    #[must_use]
    pub const fn record_lost(&self) -> bool {
        self.record_lost
    }

    /// At least one sensor was not trusted at the last wake (or nothing was
    /// known).
    #[must_use]
    pub const fn sensor_fault(&self) -> bool {
        self.faults_now
    }

    /// Lifetime totals.
    #[must_use]
    pub const fn lifetime(&self) -> &HeatLifetime {
        &self.lifetime
    }

    /// The record (closed periods).
    #[must_use]
    pub const fn log(&self) -> &HeatLog<N> {
        &self.log
    }

    /// The period being filled now, not yet in [`HeatMonitor::log`].
    #[must_use]
    pub const fn current_entry(&self) -> Option<HeatLogEntry> {
        self.current
    }

    /// Timer reading at the last wake.
    #[must_use]
    pub const fn last_wake_s(&self) -> Option<u32> {
        self.last_wake_s
    }

    /// Seconds above 50 °C in the current warm spell.
    #[must_use]
    pub fn warm_spell_s(&self, now_s: u32) -> u32 {
        self.warm_since.map_or(0, |s| now_s.saturating_sub(s))
    }

    fn update_stuck(&mut self, cfg: &Config, set: &SensorSet) -> [bool; TEMP_POINTS] {
        let mut out = [false; TEMP_POINTS];
        let readings = set.readings;
        for (i, (track, flag)) in self.stuck.iter_mut().zip(out.iter_mut()).enumerate() {
            let own = readings.get(i).copied().flatten();
            // The median-free reference: the hottest of the other readings.
            let other = readings
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .filter_map(|(_, r)| *r)
                .max();
            match (own, other) {
                (Some(t), Some(o)) if track.last == Some(t.0) => {
                    track.wakes = track.wakes.saturating_add(1);
                    track.others_min = track.others_min.min(o.0);
                    track.others_max = track.others_max.max(o.0);
                }
                (Some(t), Some(o)) => {
                    *track = StuckTrack {
                        last: Some(t.0),
                        wakes: 0,
                        others_min: o.0,
                        others_max: o.0,
                    };
                }
                _ => *track = StuckTrack::default(),
            }
            let moved = i32::from(track.others_max) - i32::from(track.others_min);
            *flag = track.wakes >= cfg.temp_stuck_wakes && moved >= i32::from(cfg.temp_stuck_min_change.0);
        }
        out
    }

    fn close_period(&mut self) {
        if let Some(e) = self.current.take() {
            self.log.push(e);
        }
    }

    /// The low-power timer woke the device. `set` holds every reading,
    /// the supervisor's included; `pad_set_id` the identity the pad
    /// connector reports (`None`: no pads, or the board cannot tell).
    pub fn on_wake(
        &mut self,
        cfg: &Config,
        now_s: u32,
        clock_restarted: bool,
        set: &SensorSet,
        pad_set_id: Option<u32>,
    ) -> HeatWake {
        let mut retest_now = false;
        // ---- gaps ----
        let mut gap = false;
        let mut dt = 0u32;
        match self.last_wake_s {
            None if self.restored => {
                gap = true;
                self.lifetime.gaps_unknown_length = self.lifetime.gaps_unknown_length.saturating_add(1);
            }
            None => {}
            Some(last) if clock_restarted || now_s < last || self.restored => {
                gap = true;
                self.lifetime.gaps_unknown_length = self.lifetime.gaps_unknown_length.saturating_add(1);
                self.next_flags |= flags::CLOCK_RESTART;
            }
            Some(last) => {
                let since = now_s - last;
                let allowed = self.asked_interval_s.saturating_add(cfg.heat_gap_tolerance_s);
                if since > allowed {
                    gap = true;
                    self.lifetime.gap_s = self.lifetime.gap_s.saturating_add(since);
                } else {
                    dt = since;
                }
            }
        }
        self.restored = false;
        if gap {
            self.lifetime.gaps = self.lifetime.gaps.saturating_add(1);
            self.gap_since = Some(now_s);
            self.close_period();
            self.next_flags |= flags::GAP_BEFORE;
            // The cooling clock cannot run across unknown time.
            self.cool_below_since = None;
            self.cooled_at = None;
            retest_now = true;
        }

        // ---- the readings ----
        let stuck = self.update_stuck(cfg, set);
        let a = assess(cfg, set, &stuck);
        let hottest = a.hottest();
        self.faults_now = a.any_fault() || hottest.is_none();
        if self.faults_now {
            self.lifetime.sensor_fault_wakes = self.lifetime.sensor_fault_wakes.saturating_add(1);
        }
        for raw in set.readings.iter().flatten() {
            self.lifetime.raw_max = opt_max(self.lifetime.raw_max, *raw);
        }

        // ---- pad set and pad heat ----
        if let Some(id) = pad_set_id {
            match self.pad_set {
                Some(old) if old == id => {}
                Some(_) => {
                    self.new_pad_set(Some(id));
                }
                None => {
                    self.pad_set = Some(id);
                    self.lifetime.pad_sets = self.lifetime.pad_sets.saturating_add(1);
                }
            }
        }
        // The pad point's own trusted reading, or (conservatively) the
        // hottest trusted reading plus the margin; unknown counts as hot.
        let pad_t = a.effective(TempPoint::Pads, true, cfg.failed_sensor_margin);
        let pad_hot = pad_t.is_none_or(|t| t > cfg.pad_heat_above);
        if dt > 0 && (pad_hot || self.pad_was_hot) {
            self.pad_hot_s = self.pad_hot_s.saturating_add(dt);
        }
        self.pad_was_hot = pad_hot;

        // ---- bands and latches ----
        let mut entry_flags = 0u8;
        if self.faults_now {
            entry_flags |= flags::SENSOR_FAULT;
        }
        if let Some(t) = hottest {
            if t > cfg.heat_service_above {
                if !self.service_needed {
                    self.lifetime.service_events = self.lifetime.service_events.saturating_add(1);
                }
                self.service_needed = true;
                self.pads_after_overheat = true;
                entry_flags |= flags::SERVICE;
            }
            if t > cfg.heat_too_hot_above {
                if self.band != HeatBand::TooHot {
                    self.lifetime.too_hot_events = self.lifetime.too_hot_events.saturating_add(1);
                }
                self.band = HeatBand::TooHot;
                self.cool_causes |= cool_causes::WAS_TOO_HOT;
            } else if self.band == HeatBand::TooHot && t <= cfg.heat_too_hot_above.minus(cfg.heat_hysteresis) {
                self.band = HeatBand::Warm;
            }
            if self.band != HeatBand::TooHot {
                if t > cfg.heat_normal_max {
                    self.band = HeatBand::Warm;
                } else if t <= cfg.heat_normal_max.minus(cfg.heat_hysteresis) {
                    self.band = HeatBand::Normal;
                }
            }
            if self.band == HeatBand::TooHot {
                entry_flags |= flags::TOO_HOT;
            }
            // Warm spell above 50 °C.
            if t > cfg.heat_warm_spell_above {
                let since = *self.warm_since.get_or_insert(now_s);
                if now_s.saturating_sub(since) >= cfg.heat_warm_spell_limit_s
                    && self.cool_causes & cool_causes::WARM_TOO_LONG == 0
                {
                    self.cool_causes |= cool_causes::WARM_TOO_LONG;
                    entry_flags |= flags::WARM_TOO_LONG;
                }
            } else if t <= cfg.heat_warm_spell_above.minus(cfg.heat_hysteresis) {
                self.warm_since = None;
            }
            // Lifetime extremes and time in each band.
            self.lifetime.max = opt_max(self.lifetime.max, t);
            for ((mx, mn), v) in self
                .lifetime
                .point_max
                .iter_mut()
                .zip(self.lifetime.point_min.iter_mut())
                .zip(a.valid.iter())
            {
                if let Some(v) = v {
                    *mx = opt_max(*mx, *v);
                    *mn = opt_min(*mn, *v);
                }
            }
            if t > cfg.heat_normal_max {
                self.lifetime.above_normal_s = self.lifetime.above_normal_s.saturating_add(dt);
            }
            if t > cfg.heat_warm_spell_above {
                self.lifetime.above_warm_spell_s = self.lifetime.above_warm_spell_s.saturating_add(dt);
            }
            if t > cfg.heat_too_hot_above {
                self.lifetime.above_too_hot_s = self.lifetime.above_too_hot_s.saturating_add(dt);
            }
        }
        if let Some(t) = a.coolest() {
            self.lifetime.min = opt_min(self.lifetime.min, t);
        }
        self.hottest_now = hottest;

        // ---- cooling before a re-test ----
        if self.cool_causes != 0 {
            let cooled = !self.faults_now && hottest.is_some_and(|t| t < cfg.heat_cooled_below);
            if cooled {
                let since = *self.cool_below_since.get_or_insert(now_s);
                if self.cooled_at.is_none() && now_s.saturating_sub(since) >= cfg.heat_cooled_for_s {
                    self.cooled_at = Some(now_s);
                    retest_now = true;
                }
            } else {
                self.cool_below_since = None;
                self.cooled_at = None;
            }
        }

        // ---- the record ----
        let period_over = self
            .current
            .is_some_and(|e| now_s < e.start_s || now_s.saturating_sub(e.start_s) >= cfg.heat_log_period_s);
        if period_over {
            self.close_period();
        }
        let e = self.current.get_or_insert(HeatLogEntry {
            start_s: now_s,
            max: None,
            min: None,
            wakes: 0,
            flags: core::mem::take(&mut self.next_flags),
        });
        e.wakes = e.wakes.saturating_add(1);
        e.flags |= entry_flags;
        if let Some(t) = hottest {
            e.max = opt_max(e.max, t);
        }
        if let Some(t) = a.coolest() {
            e.min = opt_min(e.min, t);
        }

        // ---- next wake ----
        let warm = hottest.is_none_or(|t| t > cfg.heat_normal_max);
        let next = if warm { cfg.heat_wake_warm_s } else { cfg.heat_wake_cool_s };
        self.asked_interval_s = next;
        self.last_wake_s = Some(now_s);
        HeatWake {
            next_wake_in_s: next.max(1),
            retest_now,
            gap_found: gap,
            assessment: a,
        }
    }

    fn new_pad_set(&mut self, id: Option<u32>) {
        self.pad_set = id;
        self.pad_hot_s = 0;
        self.pad_was_hot = false;
        self.pads_after_overheat = false;
        self.lifetime.pad_sets = self.lifetime.pad_sets.saturating_add(1);
    }

    /// A service technician fitted new pads on a device whose connector
    /// cannot report a pad identity: start a new pad heat total.
    pub fn pads_replaced(&mut self, pad_set_id: Option<u32>) {
        self.new_pad_set(pad_set_id);
    }

    /// A self-test passed (periodic, or the switch-on test) at timer reading
    /// `now_s`. It clears "cool and re-test" if the device had cooled before
    /// it, and a gap found before it.
    pub fn on_test_passed(&mut self, now_s: u32) {
        if self.cooled_at.is_some_and(|c| now_s >= c) {
            self.cool_causes = 0;
            self.cooled_at = None;
            self.cool_below_since = None;
        }
        if self.gap_since.is_some_and(|g| now_s >= g) {
            self.gap_since = None;
        }
    }

    /// A service action clears "service needed" (and a lost record).
    ///
    /// # Errors
    /// Refused unless the full-energy self-test and the battery health check
    /// passed and the device reads below 45 °C now.
    pub fn service_cleared(&mut self, cfg: &Config, check: ServiceCheck) -> Result<(), ServiceRefusal> {
        if !check.full_self_test_passed {
            return Err(ServiceRefusal::FullTestNotPassed);
        }
        if !check.battery_health_passed {
            return Err(ServiceRefusal::BatteryHealthNotPassed);
        }
        let cool = !self.faults_now && self.hottest_now.is_some_and(|t| t < cfg.heat_cooled_below);
        if !cool {
            return Err(ServiceRefusal::NotCooled);
        }
        self.service_needed = false;
        self.record_lost = false;
        self.cool_causes = 0;
        self.cooled_at = None;
        self.cool_below_since = None;
        self.gap_since = None;
        if self.band == HeatBand::TooHot {
            self.band = HeatBand::Warm;
        }
        Ok(())
    }
}
