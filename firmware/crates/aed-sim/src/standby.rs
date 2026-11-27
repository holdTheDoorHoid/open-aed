// SPDX-License-Identifier: GPL-3.0-or-later
//! The device between rescues: weeks of periodic self-tests (decision
//! D-15) in a few seconds of computing.
//!
//! SIMULATION ONLY. Time between wakes is skipped, not simulated: the
//! schedule (`aed_core::TestSchedule`) is woken when it asks, with a
//! scripted environment (in use, on the dock, temperature, battery, a
//! charger that weakens with age, a stuck discharge path, the timer
//! restarting). Each test it asks for is run on a fresh simulated device
//! (the same [`Plant`] as the rescue simulations) with the real
//! `aed_core::PeriodicTestRun` and the real `aed_core::Supervisor`, millisecond by
//! millisecond, for the few seconds the test takes. What the timer does
//! while the device sleeps, battery self-discharge, ageing and the dock's
//! power path are not modelled.

use std::cell::RefCell;
use std::rc::Rc;

use aed_core::schedule::LogEntry;
use aed_core::{
    BatteryStatus, Config, PeriodicOutcome, PeriodicTest, PeriodicTestRun, Readiness, Supervisor,
    TestCounters, TestSchedule, WakeInputs,
};
use aed_rhythm::synth::{Rhythm, Synth};
use aed_types::ShockPlanner;
use aed_waveform::BtePlanner;

use crate::hw::{MainView, SupervisorView};
use crate::plant::{Fault, Plant, PlantConfig};
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
    /// Temperature inside the device, °C.
    Temperature(i8),
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
}

impl StandbyScenario {
    /// `days` quiet days on the default configuration.
    #[must_use]
    pub fn quiet(days: u32) -> Self {
        StandbyScenario {
            duration_s: days.saturating_mul(DAY_S),
            changes: Vec::new(),
            config: Config::default(),
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
    /// The ready indicator after each wake: (timer seconds, readiness).
    pub readiness: Vec<(u32, Readiness)>,
    /// Every wake: (timer seconds, what the schedule decided).
    pub wakes: Vec<(u32, aed_core::WakeDecision)>,
    /// The schedule's counters at the end.
    pub counters: TestCounters,
    /// The schedule's record at the end.
    pub log: Vec<LogEntry>,
    /// The schedule itself, at the end.
    pub schedule: TestSchedule,
}

impl StandbyResult {
    /// The readiness at or just before timer reading `at_s`.
    #[must_use]
    pub fn readiness_at(&self, at_s: u32) -> Option<Readiness> {
        self.readiness.iter().rev().find(|(t, _)| *t <= at_s).map(|(_, r)| *r)
    }
}

#[derive(Debug, Clone)]
struct Environment {
    in_use_until: Option<u32>,
    docked: bool,
    temperature_c: i8,
    battery: BatteryStatus,
    plant: PlantConfig,
    faults: Vec<Fault>,
    timer_offset: i64,
    restarted: bool,
}

/// Run one periodic test on a fresh simulated device. The run is bounded
/// (30 s); a test that has not finished by then counts as aborted with the
/// capacitor not confirmed empty.
fn run_one(
    config: Config,
    supervisor: Supervisor,
    test: PeriodicTest,
    env: &Environment,
) -> (PeriodicOutcome, f64, f64, u32) {
    let mut supervisor = supervisor;
    let planner = BtePlanner::new(env.plant.waveform).unwrap_or_default();
    let full_voltage = planner.charge_voltage(aed_types::PatientMode::Adult);
    let mut plant = Plant::new(env.plant, Synth::new(Rhythm::Asystole, 1), 1);
    for f in &env.faults {
        plant.inject(*f);
    }
    plant.battery = env.battery;
    let plant = Rc::new(RefCell::new(plant));
    let mut main = MainView::new(Rc::clone(&plant));
    let mut sup_view = SupervisorView::new(Rc::clone(&plant));
    let mut run = PeriodicTestRun::new(config, test, full_voltage);
    let mut highest = 0.0_f64;
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
                let final_v = plant.borrow().cap_v;
                return (o, highest, final_v, now);
            }
        }
        supervisor.tick(&mut sup_view);
    }
    let final_v = plant.borrow().cap_v;
    (
        PeriodicOutcome {
            result: Err(aed_core::SelfTestFailure::Aborted),
            capacitor_confirmed_empty: false,
            charged: true,
        },
        highest,
        final_v,
        30_000,
    )
}

/// Run a standby scenario.
///
/// # Errors
/// If the configuration is refused.
pub fn run(s: &StandbyScenario) -> Result<StandbyResult, aed_core::ConfigError> {
    // A fresh supervisor (just woken) for every test.
    let supervisor = Supervisor::new(s.config)?;
    let mut schedule = TestSchedule::new(s.config);
    let mut env = Environment {
        in_use_until: None,
        docked: true,
        temperature_c: 20,
        battery: BatteryStatus::Good,
        plant: PlantConfig::default(),
        faults: Vec::new(),
        timer_offset: 0,
        restarted: false,
    };
    let mut changes = s.changes.clone();
    changes.sort_by_key(|c| c.0);
    let mut next_change = 0;
    let mut tests = Vec::new();
    let mut readiness = Vec::new();
    let mut wakes = Vec::new();
    let mut t: u32 = 0;
    while t < s.duration_s {
        while let Some((at, c)) = changes.get(next_change).copied().filter(|(at, _)| *at <= t) {
            next_change += 1;
            match c {
                Change::InUseUntil(until) => env.in_use_until = Some(until),
                Change::Docked(d) => env.docked = d,
                Change::Temperature(c) => env.temperature_c = c,
                Change::Battery(b) => env.battery = b,
                Change::ChargerWatts(w) => env.plant.charger_watts = w,
                Change::Fault(f) => env.faults.push(f),
                Change::Repaired => env.faults.clear(),
                Change::TimerRestartsAt(r) => {
                    env.timer_offset = i64::from(r) - i64::from(at);
                    env.restarted = true;
                }
            }
        }
        // What the low-power timer reads (it may have restarted).
        let timer = u32::try_from((i64::from(t) + env.timer_offset).max(0)).unwrap_or(u32::MAX);
        let in_use = env.in_use_until.is_some_and(|u| t < u);
        let d = schedule.on_wake(&WakeInputs {
            now_s: timer,
            clock_restarted: std::mem::take(&mut env.restarted),
            in_use,
            on_charge: env.docked,
            temperature_c: env.temperature_c,
            battery: env.battery,
        });
        wakes.push((t, d));
        let mut next = d.next_wake_in_s;
        if let Some(test) = d.run {
            let (outcome, highest_v, final_v, took_ms) = run_one(s.config, supervisor.clone(), test, &env);
            schedule.record(timer, test, &outcome);
            tests.push(TestRun {
                at_s: t,
                test,
                outcome,
                highest_v,
                final_v,
                took_ms,
            });
            next = schedule.next_wake_in_s(timer);
        }
        readiness.push((t, schedule.readiness(timer)));
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
    Ok(StandbyResult {
        tests,
        readiness,
        wakes,
        counters: schedule.counters(),
        log: schedule.log().copied().collect(),
        schedule,
    })
}
