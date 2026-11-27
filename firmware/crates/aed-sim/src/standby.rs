// SPDX-License-Identifier: GPL-3.0-or-later
//! The device between rescues: weeks or months of periodic self-tests
//! (decision D-15) and temperature logging (D-48, D-52) in a few seconds of
//! computing.
//!
//! SIMULATION ONLY. Time between wakes is skipped, not simulated: the real
//! `aed_core::Standby` (the self-test schedule, the heat record and the
//! capacitor wear count) is woken when it asks, with a scripted environment
//! (in use, on the dock, the weather, battery, a charger that weakens with
//! age, a stuck discharge path, sensor faults, the timer restarting, the
//! battery taken out). The inside of the device follows the weather through
//! the one-node thermal model in [`crate::thermal`], integrated in steps of
//! at most a minute between wakes. Each test it asks for is run on a fresh
//! simulated device (the same [`Plant`] as the rescue simulations) with the
//! real `aed_core::PeriodicTestRun` and the real `aed_core::Supervisor`,
//! millisecond by millisecond, for the few seconds the test takes; the heat
//! the test burns inside is added to the thermal model. What the timer does
//! while the device sleeps, battery self-discharge, ageing and the dock's
//! power path are not modelled.
//!
//! Invariants checked at every wake (reported in
//! [`StandbyResult::violations`]): no test charges the capacitor outside
//! its limits (true temperatures); the dock never charges the battery
//! outside 0-45 °C; the device never shows "ready" while it is above 60 °C
//! or has been within the last 30 minutes; nothing in the durable record
//! (heat peak, overheat latches, pad heat, wear) is lost when the battery
//! is taken out; and the heat record never loses the hottest reading.

use std::cell::RefCell;
use std::rc::Rc;

use aed_core::schedule::LogEntry;
use aed_core::{
    BatteryStatus, Config, DeciC, DurableRecord, DurableStore, PeriodicOutcome, PeriodicTest,
    PeriodicTestRun, Readiness, Standby, StandbyDecision, StandbyInputs, StoreError, Supervisor,
    TempPoint, TestCounters, TestSchedule,
};
use aed_rhythm::synth::{Rhythm, Synth};
use aed_types::ShockPlanner;
use aed_waveform::BtePlanner;

use crate::hw::{MainView, SupervisorView};
use crate::plant::{Fault, Plant, PlantConfig};
use crate::thermal::{TempSensorFault, Thermal};
use crate::runner::MAIN_TICK_MS;

/// Seconds in a day.
pub const DAY_S: u32 = 86_400;

/// Something that changes in the device's surroundings, from a time on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Change {
    /// The device is switched on (in use) until this timer reading, seconds.
    InUseUntil(u32),
    /// On (true) or off (false) the charging dock.
    Docked(bool),
    /// The surroundings AND the inside of the device jump to this
    /// temperature at once, °C (a scripted test condition, no lag).
    Temperature(i8),
    /// The surroundings change to this, °C; the inside follows with the
    /// thermal model's lag.
    Ambient(f64),
    /// A temperature sensor fails (or is repaired).
    TempSensor(TempPoint, TempSensorFault),
    /// A temperature sensor reads this many °C off the truth.
    TempOffset(TempPoint, f64),
    /// A different pad set is fitted (its identity).
    PadSet(u32),
    /// The battery is taken out and put back: the volatile memory is lost,
    /// the durable record is restored and the timer restarts at 0.
    PowerLoss,
    /// The power monitor's view of the battery.
    Battery(BatteryStatus),
    /// The charger's power, watts (the model default is 30 W; an ageing
    /// charger delivers less).
    ChargerWatts(f64),
    /// Inject a plant fault into every test from now on.
    Fault(Fault),
    /// Remove every injected fault from now on (repaired).
    Repaired,
    /// The low-power timer restarts at this reading (battery taken out).
    TimerRestartsAt(u32),
}

/// A standby scenario.
#[derive(Debug, Clone)]
pub struct StandbyScenario {
    /// How long, seconds of timer time.
    pub duration_s: u32,
    /// Changes, in time order: (timer seconds, change).
    pub changes: Vec<(u32, Change)>,
    /// The firmware configuration.
    pub config: Config,
    /// The surroundings over time, °C (timer seconds in). `None`: only the
    /// scripted changes. When set it overrides [`Change::Ambient`].
    pub weather: Option<fn(u32) -> f64>,
}

impl StandbyScenario {
    /// `days` quiet days on the default configuration.
    #[must_use]
    pub fn quiet(days: u32) -> Self {
        StandbyScenario {
            duration_s: days.saturating_mul(DAY_S),
            changes: Vec::new(),
            config: Config::default(),
            weather: None,
        }
    }
}

/// One test that ran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestRun {
    /// Timer reading when it ran, seconds.
    pub at_s: u32,
    /// Which test.
    pub test: PeriodicTest,
    /// Its outcome, as the firmware saw it.
    pub outcome: PeriodicOutcome,
    /// Highest TRUE capacitor voltage during it (model).
    pub highest_v: f64,
    /// TRUE capacitor voltage when it finished (model).
    pub final_v: f64,
    /// Simulated milliseconds the test took.
    pub took_ms: u32,
}

/// Everything a standby run produced.
#[derive(Debug, Clone)]
pub struct StandbyResult {
    /// Every test that ran.
    pub tests: Vec<TestRun>,
    /// The self-test schedule's own ready indicator after each wake (the
    /// self-test reasons only): (simulation seconds, readiness).
    pub readiness: Vec<(u32, Readiness)>,
    /// Every wake: (simulation seconds, what the schedule decided).
    pub wakes: Vec<(u32, aed_core::WakeDecision)>,
    /// Every wake: (simulation seconds, the whole standby decision, with
    /// the one readiness result and the battery-charging decision).
    pub decisions: Vec<(u32, StandbyDecision)>,
    /// The true inside temperature at each wake: (simulation seconds, °C).
    pub temperatures: Vec<(u32, f64)>,
    /// The schedule's counters at the end.
    pub counters: TestCounters,
    /// The schedule's record at the end.
    pub log: Vec<LogEntry>,
    /// The schedule itself, at the end.
    pub schedule: TestSchedule,
    /// The whole standby state at the end (heat record, wear).
    pub standby: Standby<{ aed_core::heat::HEAT_LOG_CAPACITY }>,
    /// Broken invariants (see the module notes). Empty when all held.
    pub violations: Vec<String>,
    /// Hottest trusted reading seen at any wake.
    pub hottest_seen: Option<DeciC>,
    /// Times the battery was taken out.
    pub power_losses: u32,
}

impl StandbyResult {
    /// The schedule's readiness at or just before simulation second `at_s`.
    #[must_use]
    pub fn readiness_at(&self, at_s: u32) -> Option<Readiness> {
        self.readiness.iter().rev().find(|(t, _)| *t <= at_s).map(|(_, r)| *r)
    }

    /// The whole standby decision at or just before simulation second `at_s`.
    #[must_use]
    pub fn decision_at(&self, at_s: u32) -> Option<StandbyDecision> {
        self.decisions.iter().rev().find(|(t, _)| *t <= at_s).map(|(_, d)| *d)
    }
}

#[derive(Debug, Clone)]
struct Environment {
    in_use_until: Option<u32>,
    docked: bool,
    battery: BatteryStatus,
    plant: PlantConfig,
    faults: Vec<Fault>,
    timer_offset: i64,
    restarted: bool,
    thermal: Thermal,
    pad_set: u32,
}

/// The durable memory, kept by the simulation across a power loss.
#[derive(Debug, Default)]
struct SimStore {
    saved: Option<Box<DurableRecord>>,
}

impl DurableStore<{ aed_core::heat::HEAT_LOG_CAPACITY }> for SimStore {
    fn save(&mut self, record: &DurableRecord) -> Result<(), StoreError> {
        match self.saved.as_mut() {
            Some(b) => (**b).clone_from(record),
            None => self.saved = Some(Box::new(record.clone())),
        }
        Ok(())
    }
    fn load(&mut self, into: &mut DurableRecord) -> Result<(), StoreError> {
        let saved = self.saved.as_ref().ok_or(StoreError::Empty)?;
        into.clone_from(saved);
        Ok(())
    }
}

/// Run one periodic test on a fresh simulated device. The run is bounded
/// (30 s); a test that has not finished by then counts as aborted with the
/// capacitor not confirmed empty. Returns the outcome, the highest and the
/// final true voltage, the time taken, and the heat it released inside (J).
fn run_one(
    config: Config,
    supervisor: Supervisor,
    test: PeriodicTest,
    env: &Environment,
) -> (PeriodicOutcome, f64, f64, u32, f64) {
    let mut supervisor = supervisor;
    let planner = BtePlanner::new(env.plant.waveform).unwrap_or_default();
    let full_voltage = planner.charge_voltage(aed_types::PatientMode::Adult);
    let mut plant = Plant::new(env.plant, Synth::new(Rhythm::Asystole, 1), 1);
    for f in &env.faults {
        plant.inject(*f);
    }
    plant.battery = env.battery;
    plant.thermal = env.thermal;
    let plant = Rc::new(RefCell::new(plant));
    let mut main = MainView::new(Rc::clone(&plant));
    let mut sup_view = SupervisorView::new(Rc::clone(&plant));
    let mut run = PeriodicTestRun::new(config, test, full_voltage);
    let mut highest = 0.0_f64;
    let heat = |p: &Plant| p.dump_energy_j + p.charger_energy_j * p.thermal.config.charger_loss_per_j;
    // At most 30 s: the full-energy charge limit is 11.4 s.
    for _ in 0..30_000 {
        let _ = plant.borrow_mut().step();
        let (now, v) = {
            let p = plant.borrow();
            (p.now_ms, p.cap_v)
        };
        highest = highest.max(v);
        if now.is_multiple_of(MAIN_TICK_MS) {
            if let Some(o) = run.step(&mut main) {
                let p = plant.borrow();
                return (o, highest, p.cap_v, now, heat(&p));
            }
        }
        supervisor.tick(&mut sup_view);
    }
    let p = plant.borrow();
    (
        PeriodicOutcome {
            result: Err(aed_core::SelfTestFailure::Aborted),
            capacitor_confirmed_empty: false,
            charged: true,
        },
        highest,
        p.cap_v,
        30_000,
        heat(&p),
    )
}

/// Advance the thermal model from `from` to `to` (simulation seconds), in
/// steps of at most a minute, following the weather if there is one.
fn advance_thermal(env: &mut Environment, s: &StandbyScenario, from: u32, to: u32, electronics_on: bool) {
    let mut t = from;
    while t < to {
        let step = (to - t).min(60);
        if let Some(w) = s.weather {
            env.thermal.ambient_c = w(t);
        }
        env.thermal.step(f64::from(step), 0.0, electronics_on);
        t += step;
    }
}

/// What the durable record held before a power loss, to check nothing is lost.
#[derive(Debug, Clone, Copy)]
struct DurableSnapshot {
    max: Option<DeciC>,
    too_hot_events: u32,
    service_needed: bool,
    pad_hot_s: u32,
    wear: u32,
}

fn snapshot<const N: usize>(sb: &Standby<N>) -> DurableSnapshot {
    DurableSnapshot {
        max: sb.heat().lifetime().max,
        too_hot_events: sb.heat().lifetime().too_hot_events,
        service_needed: sb.heat().service_needed(),
        pad_hot_s: sb.heat().pad_hot_s(),
        wear: sb.wear().total(),
    }
}

/// Run a standby scenario.
///
/// # Errors
/// If the configuration is refused.
#[allow(clippy::too_many_lines)]
pub fn run(s: &StandbyScenario) -> Result<StandbyResult, aed_core::ConfigError> {
    let cfg = s.config;
    // A fresh supervisor (just woken) for every test.
    let supervisor = Supervisor::new(cfg)?;
    let mut sb: Standby = Standby::new(cfg)?;
    let mut store = SimStore::default();
    let mut env = Environment {
        in_use_until: None,
        docked: true,
        battery: BatteryStatus::Good,
        plant: PlantConfig::default(),
        faults: Vec::new(),
        timer_offset: 0,
        restarted: false,
        thermal: Thermal::at(20.0),
        pad_set: 1,
    };
    let mut changes = s.changes.clone();
    changes.sort_by_key(|c| c.0);
    let mut next_change = 0;
    let mut tests = Vec::new();
    let mut readiness = Vec::new();
    let mut wakes = Vec::new();
    let mut decisions = Vec::new();
    let mut temperatures = Vec::new();
    let mut violations = Vec::new();
    let mut hottest_seen: Option<DeciC> = None;
    let mut power_losses = 0u32;
    let mut last_too_hot_at: Option<u32> = None;
    let mut t: u32 = 0;
    let mut last_t: u32 = 0;
    while t < s.duration_s {
        let in_use_before = env.in_use_until.is_some_and(|u| last_t < u);
        advance_thermal(&mut env, s, last_t, t, in_use_before);
        last_t = t;
        while let Some((at, c)) = changes.get(next_change).copied().filter(|(at, _)| *at <= t) {
            next_change += 1;
            match c {
                Change::InUseUntil(until) => env.in_use_until = Some(until),
                Change::Docked(d) => env.docked = d,
                Change::Temperature(c) => {
                    env.thermal.ambient_c = f64::from(c);
                    env.thermal.interior_c = f64::from(c);
                }
                Change::Ambient(c) => env.thermal.ambient_c = c,
                Change::TempSensor(p, f) => {
                    if let Some(slot) = env.thermal.faults.get_mut(p.index()) {
                        *slot = f;
                    }
                }
                Change::TempOffset(p, o) => {
                    if let Some(slot) = env.thermal.offsets.get_mut(p.index()) {
                        *slot = o;
                    }
                }
                Change::PadSet(id) => env.pad_set = id,
                Change::Battery(b) => env.battery = b,
                Change::ChargerWatts(w) => env.plant.charger_watts = w,
                Change::Fault(f) => env.faults.push(f),
                Change::Repaired => env.faults.clear(),
                Change::TimerRestartsAt(r) => {
                    env.timer_offset = i64::from(r) - i64::from(at);
                    env.restarted = true;
                }
                Change::PowerLoss => {
                    let before = snapshot(&sb);
                    let _ = sb.save(&mut store);
                    let (restored, err) = Standby::restore(cfg, &mut store)?;
                    sb = restored;
                    power_losses += 1;
                    env.timer_offset = -i64::from(at);
                    env.restarted = true;
                    let after = snapshot(&sb);
                    let lost = err.is_some()
                        || after.max < before.max
                        || after.too_hot_events < before.too_hot_events
                        || (before.service_needed && !after.service_needed)
                        || after.pad_hot_s < before.pad_hot_s
                        || after.wear < before.wear;
                    if lost {
                        violations.push(format!("at {at} s: durable record lost across power loss: {before:?} -> {after:?}"));
                    }
                }
            }
        }
        // What the low-power timer reads (it may have restarted).
        let timer = u32::try_from((i64::from(t) + env.timer_offset).max(0)).unwrap_or(u32::MAX);
        let in_use = env.in_use_until.is_some_and(|u| t < u);
        let temps = env.thermal.sensor_set();
        let d = sb.on_wake(&StandbyInputs {
            now_s: timer,
            clock_restarted: std::mem::take(&mut env.restarted),
            in_use,
            on_charge: env.docked,
            battery: env.battery,
            temperatures: temps,
            pad_set_id: Some(env.pad_set),
        });
        if let Some(h) = sb.heat().hottest_now() {
            hottest_seen = Some(hottest_seen.map_or(h, |x| x.max(h)));
        }
        // Invariants on this wake, against TRUE temperatures.
        let th = env.thermal;
        let tol = th.offsets.iter().fold(0.0_f64, |m, o| m.max(o.abs())) + 0.15;
        let healthy = th.faults.iter().all(|f| *f == TempSensorFault::Healthy);
        let hot = th.hottest_charge_part_c();
        let cells = [th.true_at(TempPoint::Cell1), th.true_at(TempPoint::Cell2)];
        if hot > 60.0 {
            last_too_hot_at = Some(t);
        }
        if healthy && d.battery_charging_allowed && cells.iter().any(|c| *c < -tol || *c > 45.0 + tol) {
            violations.push(format!("at {t} s: dock charging allowed with cells at {cells:?} °C"));
        }
        let recently_too_hot = last_too_hot_at.is_some_and(|h| t.saturating_sub(h) < cfg.heat_cooled_for_s);
        if healthy && d.readiness.is_ready() && (hot > 60.0 + tol || recently_too_hot) {
            violations.push(format!("at {t} s: shown ready at {hot:.1} °C / recently too hot"));
        }
        temperatures.push((t, th.interior_c));
        wakes.push((t, d.tests));
        decisions.push((t, d));
        let mut next = d.next_wake_in_s;
        if let Some(test) = d.tests.run {
            if healthy {
                let cap = th.true_at(TempPoint::Capacitor);
                let over = hot > 60.0 + tol
                    || (test == PeriodicTest::Full
                        && (cap > 50.0 + tol || cells.iter().any(|c| *c < -tol || *c > 45.0 + tol)));
                if over {
                    violations.push(format!("at {t} s: {test:?} test charged at {hot:.1} °C (capacitor {cap:.1}, cells {cells:?})"));
                }
            }
            let (outcome, highest_v, final_v, took_ms, heat_j) = run_one(cfg, supervisor.clone(), test, &env);
            env.thermal.step(f64::from(took_ms) / 1_000.0, heat_j, true);
            sb.record(timer, test, &outcome);
            tests.push(TestRun {
                at_s: t,
                test,
                outcome,
                highest_v,
                final_v,
                took_ms,
            });
            next = sb.next_wake_in_s(timer);
        }
        let _ = sb.save(&mut store);
        readiness.push((t, sb.schedule().readiness(timer)));
        // Wake no later than the next scripted change, and when use ends.
        let mut wake = t.saturating_add(next.max(1));
        if let Some((at, _)) = changes.get(next_change) {
            wake = wake.min(*at);
        }
        if let Some(u) = env.in_use_until.filter(|u| *u > t) {
            wake = wake.min(u);
        }
        t = wake.max(t + 1);
    }
    // The heat record never loses the hottest reading.
    let life_max = sb.heat().lifetime().max;
    if life_max != hottest_seen.map_or(life_max, |h| life_max.map(|m| m.max(h))) || (hottest_seen.is_some() && life_max.is_none()) {
        violations.push(format!("lifetime peak {life_max:?} below the hottest reading seen {hottest_seen:?}"));
    }
    if sb.heat().log().overwritten() == 0 && power_losses == 0 {
        let logged = sb.heat().log().iter().copied().chain(sb.heat().current_entry()).filter_map(|e| e.max).max();
        if logged != hottest_seen {
            violations.push(format!("heat record peak {logged:?} differs from the hottest reading seen {hottest_seen:?}"));
        }
    }
    Ok(StandbyResult {
        tests,
        readiness,
        wakes,
        decisions,
        temperatures,
        counters: sb.schedule().counters(),
        log: sb.schedule().log().copied().collect(),
        schedule: sb.schedule().clone(),
        standby: sb,
        violations,
        hottest_seen,
        power_losses,
    })
}
