// SPDX-License-Identifier: GPL-3.0-or-later
//! Compression sensing (decision D-36) in the whole simulated system:
//! finding F-12 across compression rates and sizes, with and without
//! sensing, and the window in which a shock can still fire after hands go
//! on the chest. SIMULATION ONLY: cartoon signals, provisional settings.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

mod common;

use aed_core::RescueState;
use aed_rhythm::synth::Rhythm;
use aed_sim::scenario::{Action, Trigger};
use aed_sim::{run, Expect, Rule, Scenario};
use common::{assert_invariants, fail};

fn never_stops(rhythm: Rhythm, per_min: f64, ohms: f64, sensing: bool) -> aed_sim::RunResult {
    let mut s = Scenario::base("f12-sweep", "responder never stops compressing");
    s.rhythm = rhythm;
    s.after_shock = Vec::new();
    s.responder.keeps_compressing = true;
    s.responder.compression_per_min = per_min;
    s.responder.compression_ohms = ohms;
    s.chest_sensing = sensing;
    s.duration_ms = 90_000;
    s.expect = Expect::InvariantsOnly;
    run(&s).unwrap()
}

#[test]
fn f12_compressions_that_never_stop_are_never_shocked_with_sensing_at_any_rate_or_size_above_the_threshold() {
    // Finding F-12: the heart signal alone took compressions at 150/min
    // over a flatline for a shockable rhythm. With compression sensing the
    // analysis is never fed while compressions are sensed, whatever the
    // heart signal looks like.
    let mut old_shocks = 0;
    let mut runs = 0;
    for rhythm in [Rhythm::Asystole, Rhythm::sinus(70.0)] {
        for per_min in [100.0, 120.0, 140.0, 150.0, 160.0, 180.0] {
            for ohms in [0.5, 1.0, 2.5] {
                let r = never_stops(rhythm, per_min, ohms, true);
                assert_invariants(&r);
                if !r.shocks.is_empty() || r.entered(RescueState::Countdown) {
                    fail(&r, &format!("{per_min}/min {ohms} Ω: counted down or shocked"));
                }
                let old = never_stops(rhythm, per_min, ohms, false);
                old_shocks += old.shocks.len();
                runs += 1;
            }
        }
    }
    println!(
        "F-12 sweep (SIMULATED): with sensing 0 shocks in {runs} runs; without sensing {old_shocks} shocks"
    );
    assert!(old_shocks > 0, "the sweep must reproduce F-12 without sensing");
}

#[test]
fn measure_how_soon_a_shock_can_still_fire_after_hands_go_on_the_chest() {
    // A firm 1 Ω press on the chest starting at every 50 ms through the
    // countdown. The press is seen and the shock held back, except when it
    // starts so close to delivery that the detector cannot see it in time.
    // The longest such "hands on at the shock" time is the exposure window.
    let mut worst = 0u32;
    let mut fired_with_hands_on = 0;
    let mut held_back = 0;
    for offset in (0..3_100u32).step_by(50) {
        let mut s = Scenario::base("touch-sweep", "press during the countdown");
        s.script.push((
            Trigger::AfterState {
                state: RescueState::Countdown,
                nth: 1,
                after_ms: offset,
            },
            Action::Touch(800),
        ));
        s.duration_ms = 60_000;
        s.expect = Expect::InvariantsOnly;
        let r = run(&s).unwrap();
        assert_invariants(&r);
        if let Some(&ms) = r.hands_on_at_fire.first() {
            fired_with_hands_on += 1;
            worst = worst.max(ms);
        }
        if r.hands_on_aborts > 0 {
            held_back += 1;
        }
    }
    println!(
        "Press during the countdown (SIMULATED): held back {held_back} times, fired with hands on {fired_with_hands_on} times, \
         hands on for at most {worst} ms at the shock"
    );
    assert!(worst <= aed_sim::runner::HANDS_ON_ALLOWANCE_MS);
    assert!(held_back > 40);
}

#[test]
fn small_compressions_below_the_threshold_are_a_known_exposure() {
    // Compressions of 0.2 ohm by a responder who never stops: invisible to
    // the detector, so the device behaves as without sensing. On fine VF
    // with compressions the heart signal may then say "shockable" and the
    // shock fires with hands on. Recorded here, not hidden; see the
    // evaluation report (some real patients' compressions are this small).
    let r = never_stops(Rhythm::coarse_vf(), 110.0, 0.2, true);
    let hands_on = r.violations_of(Rule::FiredWhileHandsOn).len();
    println!(
        "0.2 Ω compressions that never stop (SIMULATED): {} shock(s), {hands_on} with hands on beyond the allowance",
        r.shocks.len()
    );
}
