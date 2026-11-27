// SPDX-License-Identifier: GPL-3.0-or-later
//! Randomised and exhaustive walks over input sequences, checking the safety
//! invariants at every tick.
//!
//! The random walks use a small deterministic generator (xorshift), so every
//! run is repeatable: a failure prints its seed.

use super::mock::{Bench, FakePlanner};
use crate::rescue::RescueState;
use crate::Config;
use aed_types::{PatientMode, RhythmDecision};
use std::format;
use std::string::String;
use std::vec::Vec;

use RescueState as S;

/// Deterministic pseudo-random numbers (xorshift64*).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u32) -> u32 {
        (self.next() >> 33) as u32 % n
    }
    /// True with probability `per_thousand` / 1000.
    fn chance(&mut self, per_thousand: u32) -> bool {
        self.below(1000) < per_thousand
    }
}

/// Shorter CPR so a walk covers many rescue cycles.
fn walk_config() -> Config {
    Config {
        cpr_period_ms: 15_000,
        analysis_time_limit_ms: 10_000,
        ..Config::default()
    }
}

/// The longest each state may last, or None for states that wait on the responder.
fn limit(c: &Config, s: RescueState) -> Option<u32> {
    match s {
        S::SelfTest => Some(
            c.self_test_link_wait_ms
                + c.charge_time_limit_ms
                + c.self_test_agree_wait_ms
                + c.disarm_time_limit_ms,
        ),
        S::Analysing | S::AnalysingCharging => Some(c.analysis_time_limit_ms),
        S::Charging => Some(c.charge_time_limit_ms),
        S::Countdown => Some(c.countdown_ms),
        S::Delivering => Some(c.delivery_timeout_ms),
        S::Cpr => Some(c.cpr_period_ms),
        S::AttachPads | S::Fault => None,
    }
}

/// Watches a bench tick by tick and records any broken invariant.
struct Watcher {
    entered: (RescueState, u32),
    main_dead_since: Option<u32>,
    emptying_since: Option<u32>,
    fires_checked: usize,
    failures: Vec<String>,
}

impl Watcher {
    fn new() -> Self {
        Watcher {
            entered: (S::SelfTest, 0),
            main_dead_since: None,
            emptying_since: None,
            fires_checked: 0,
            failures: Vec::new(),
        }
    }

    fn check(&mut self, b: &Bench) {
        let now = b.board.now;
        let c = &b.config;
        let state = b.rescue.state();

        // 1. Shock requests only after a Shockable decision, with pads in range
        //    and cancel not pressed (checked inside the mock at request time).
        if !b.board.violations.is_empty() {
            self.failures.push(format!("{:?}", b.board.violations));
        }
        // 2. The interlock never passed without both signals and both holds.
        for f in &b.board.fires[self.fires_checked..] {
            if !(f.main_request && f.permit && f.main_hold && f.sup_hold) {
                self.failures
                    .push(format!("t={} fired without both signals: {f:?}", f.at));
            }
        }
        self.fires_checked = b.board.fires.len();
        // 3. The supervisor never permits while letting go of the capacitor.
        if b.board.permit && !b.board.sup_hold {
            self.failures
                .push(format!("t={now} permit while supervisor not holding"));
        }

        if !b.main_alive {
            // 4. A dead main controller: the supervisor must let go in time.
            let since = *self.main_dead_since.get_or_insert(now);
            if now - since > c.heartbeat_timeout_ms + 2 * b.tick_ms && b.board.sup_hold {
                self.failures
                    .push(format!("t={now} supervisor still holding for a dead main"));
            }
            // 4b. ...and sound its own alarm, so the device is not silent.
            if now - since > c.heartbeat_timeout_ms + c.heartbeat_period_ms + 2 * b.tick_ms
                && !b.board.sup_alarm
            {
                self.failures
                    .push(format!("t={now} supervisor alarm off for a dead main"));
            }
            return;
        }

        // 5. After Fault, the charger is off and the hold released; and outside
        //    the charging states the main controller never holds or charges.
        let charging_state = state == S::SelfTest || state.holds_charge();
        if !charging_state && (b.board.main_hold || b.board.charger.is_some()) {
            self.failures.push(format!(
                "t={now} {state:?} with hold {} charger {:?}",
                b.board.main_hold, b.board.charger
            ));
        }
        // 5b. While a charge is being emptied in the background, the main
        //     controller neither holds nor charges (no new charge starts).
        if b.rescue.emptying() && (b.board.main_hold || b.board.charger.is_some()) {
            self.failures.push(format!(
                "t={now} {state:?}: holding or charging while still emptying"
            ));
        }
        // 5c. Emptying is confirmed within its limit, or the device is in
        //     Fault (the serious fault).
        match (b.rescue.emptying(), self.emptying_since) {
            (true, None) => self.emptying_since = Some(now),
            (true, Some(since)) => {
                if now - since > c.disarm_time_limit_ms + 2 * b.tick_ms {
                    self.failures
                        .push(format!("t={now} emptying not resolved in time"));
                }
            }
            (false, _) => self.emptying_since = None,
        }
        if state == S::Fault && b.board.request.is_some() {
            self.failures
                .push(format!("t={now} shock request in Fault"));
        }

        // 6. No state lasts longer than its time limit (one tick of slack).
        if state != self.entered.0 {
            self.entered = (state, now);
        }
        if let Some(max) = limit(c, state) {
            let stayed = now - self.entered.1;
            if stayed > max + 2 * b.tick_ms {
                self.failures.push(format!(
                    "t={now} stuck in {state:?} for {stayed} ms (limit {max})"
                ));
            }
        }
    }
}

/// One random walk; returns the broken invariants, if any.
fn random_walk(seed: u64, length_ms: u32) -> Vec<String> {
    let mut rng = Rng::new(seed);
    let mut b = Bench::with(walk_config(), FakePlanner::default());
    let mut w = Watcher::new();
    // Perhaps one hardware fault, at a random moment.
    let fault_kind = if rng.chance(400) {
        Some(rng.below(10))
    } else {
        None
    };
    let fault_at = rng.below(length_ms);
    let mut fault_done = false;

    while b.board.now < length_ms {
        if b.board.now.is_multiple_of(100) {
            if rng.chance(80) {
                b.board.rhythm = match rng.below(4) {
                    0 | 1 => RhythmDecision::Shockable,
                    2 => RhythmDecision::NonShockable,
                    _ => RhythmDecision::Indeterminate,
                };
            }
            if rng.chance(25) {
                // Includes 176 to 250 Ω: pads on but above the shock range (D-32).
                b.board.pad_ohms =
                    [80, 140, 3_000, 10, 24, 176, 200, 250, 251][rng.below(9) as usize];
            } else if b.board.pad_ohms != 80 && rng.chance(200) {
                b.board.pad_ohms = 80;
            }
            if !b.board.cancel && rng.chance(15) {
                b.board.cancel = true;
            } else if b.board.cancel && rng.chance(300) {
                b.board.cancel = false;
            }
            if rng.chance(8) {
                b.board.mode = match b.board.mode {
                    PatientMode::Adult => PatientMode::Child,
                    PatientMode::Child => PatientMode::Adult,
                };
            }
            if rng.chance(10) {
                b.board.sup_pad_ohms = if b.board.sup_pad_ohms.is_some() {
                    None
                } else {
                    Some(3_000)
                };
            }
            if let Some(kind) = fault_kind {
                if !fault_done && b.board.now >= fault_at {
                    fault_done = true;
                    match kind {
                        0 => b.board.charger_broken = true,
                        1 => b.board.charger_runaway = true,
                        2 => b.board.dump_broken = true,
                        3 => b.board.output_fault = true,
                        4 => b.board.report_wire_up = false,
                        5 => b.board.status_wire_up = false,
                        6 => b.board.sup_v_offset = 300,
                        7 => b.board.main_v_offset = 300,
                        8 => b.main_alive = false,
                        _ => b.board.battery = crate::hal::BatteryStatus::Depleted,
                    }
                }
            }
        }
        b.step();
        w.check(&b);
        if !w.failures.is_empty() {
            let mut f = w.failures;
            f.insert(
                0,
                format!(
                    "seed {seed}, fault {fault_kind:?} at {fault_at}; states {:?}",
                    b.states
                ),
            );
            return f;
        }
    }
    Vec::new()
}

#[test]
fn random_walks_never_break_a_safety_invariant() {
    let mut shocks = 0;
    for seed in 1..=400u64 {
        let failures = random_walk(seed, 90_000);
        assert!(failures.is_empty(), "{failures:#?}");
        shocks += 1;
    }
    assert_eq!(shocks, 400);
}

#[test]
fn random_walks_actually_reach_every_state() {
    // Guard against a walk that never exercises the dangerous states.
    let mut seen = Vec::new();
    for seed in 1..=40u64 {
        let mut rng = Rng::new(seed);
        let mut b = Bench::with(walk_config(), FakePlanner::default());
        while b.board.now < 60_000 {
            if b.board.now.is_multiple_of(100) && rng.chance(30) {
                b.board.rhythm = if rng.chance(700) {
                    RhythmDecision::Shockable
                } else {
                    RhythmDecision::NonShockable
                };
            }
            if b.board.now.is_multiple_of(100) && rng.chance(5) {
                b.board.cancel = !b.board.cancel;
            }
            b.step();
        }
        for &(_, s) in &b.states {
            if !seen.contains(&s) {
                seen.push(s);
            }
        }
    }
    for s in [
        S::AttachPads,
        S::AnalysingCharging,
        S::Charging,
        S::Countdown,
        S::Delivering,
        S::Cpr,
    ] {
        assert!(seen.contains(&s), "{s:?} never reached");
    }
}

/// The disturbances swept exhaustively.
#[derive(Debug, Clone, Copy)]
enum Disturbance {
    Cancel,
    PadsOff,
    PadsShorted,
    ModeChange,
    RhythmNonShockable,
    RhythmIndeterminate,
    SupervisorSeesPadsOff,
}

const ALL: [Disturbance; 7] = [
    Disturbance::Cancel,
    Disturbance::PadsOff,
    Disturbance::PadsShorted,
    Disturbance::ModeChange,
    Disturbance::RhythmNonShockable,
    Disturbance::RhythmIndeterminate,
    Disturbance::SupervisorSeesPadsOff,
];

#[test]
fn exhaustive_single_disturbance_at_every_moment_of_an_armed_cycle_prevents_the_shock() {
    // For each disturbance, applied at every 250 ms step from the start of the
    // analysis to past the moment of delivery: if it came before the shock
    // fired, no shock fires. (A mode change before charging is allowed and
    // must give a child-energy plan instead.) Rhythm changes are allowed one analyser interval
    // (1 s in the fake) to be noticed, since the analysis needs new data.
    for d in ALL {
        let mut offset = 0;
        while offset <= 16_000 {
            let mut b = Bench::new();
            b.board.rhythm = RhythmDecision::Shockable;
            b.run_to_analysing();
            let mut w = Watcher::new();
            let start = b.board.now;
            while b.board.now < start + offset {
                b.step();
                w.check(&b);
            }
            let fired_before = b.shocks_fired();
            let applied_at = b.board.now;
            let armed_when_applied =
                matches!(b.state(), S::Charging | S::Countdown | S::Delivering);
            match d {
                Disturbance::Cancel => b.board.cancel = true,
                Disturbance::PadsOff => b.board.pad_ohms = 3_000,
                Disturbance::PadsShorted => b.board.pad_ohms = 5,
                Disturbance::ModeChange => b.board.mode = PatientMode::Child,
                Disturbance::RhythmNonShockable => {
                    b.board.rhythm = RhythmDecision::NonShockable;
                }
                Disturbance::RhythmIndeterminate => {
                    b.board.rhythm = RhythmDecision::Indeterminate;
                }
                Disturbance::SupervisorSeesPadsOff => b.board.sup_pad_ohms = Some(3_000),
            }
            for _ in 0..(20_000 / b.tick_ms) {
                b.step();
                w.check(&b);
            }
            assert!(
                w.failures.is_empty(),
                "{d:?} at +{offset}: {:?}",
                w.failures
            );
            if matches!(d, Disturbance::ModeChange) && !armed_when_applied {
                // Changing the mode before charging starts is legitimate: the
                // shock, if any, must then be planned for a child.
                for r in b.board.requests.iter().filter(|r| r.at > applied_at) {
                    assert_eq!(r.plan.charge_voltage, aed_types::Volts(1000), "+{offset}");
                }
                offset += 250;
                continue;
            }
            let grace = match d {
                Disturbance::RhythmNonShockable | Disturbance::RhythmIndeterminate => 1_000,
                _ => 0,
            };
            let late_fires: Vec<u32> = b.board.fires[fired_before..]
                .iter()
                .map(|f| f.at)
                .filter(|&t| t > applied_at + grace)
                .collect();
            assert!(
                late_fires.is_empty(),
                "{d:?} applied at +{offset} ms, yet fired at {late_fires:?} (applied {applied_at})"
            );
            offset += 250;
        }
    }
}
