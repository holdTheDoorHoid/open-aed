# aed-sim

A simulator that runs the real firmware parts together on an ordinary computer, against a simulated device and a simulated patient. It is the first place the three firmware parts (`aed-core`, `aed-rhythm`, `aed-waveform`) run as one system.

> **SIMULATION ONLY. NOT A MEDICAL DEVICE.** Passing here shows that the firmware parts fit together and that the safety rules hold *inside this model*. It says **nothing** about a real device: no hardware exists, the heart signals are cartoons, and the electronics are a few lines of arithmetic. Nothing here is safe or ready for use on a person or animal. See [SAFETY.md](../../../SAFETY.md).

## What it does NOT model

Read this before reading any result.

- **Real heart signals.** The patient's heart signal comes from the made-up generators in `aed-rhythm`'s `synth` module. Real cardiac-arrest recordings are messier and are not used here.
- **Real electronics.** The capacitor, charger, discharge load and output bridge are simple equations (constant-power charging, exponential emptying, the `aed-waveform` shock model). No switch voltage drops, wiring inductance, capacitor dielectric absorption ("charge creeping back"), component tolerances, heating or ageing.
- **Switch behaviour.** The shock is stepped in time (10 µs steps: the rail, the patient relay's travel and bounce, the sequence latch, each phase, the caps and the lockout; see "The shock as a process in time"), but the switches themselves are perfect: no switching times, no wiring or coil inductance (so the over-current trip acts within one step, and the coil energy of review R-2 is not modelled), no half-on states, ringing or leakage. The relay and rail timings are model figures; no relay is chosen.
- **Noise coupling.** The shock does not disturb the controllers, the sensors or the message wire. The heart-signal input is simply blanked for 1.5 s after a shock.
- **Processor timing.** The controllers tick at exact intervals (main every 2 ms, supervisor every 1 ms). No interrupt latency, jitter, stack limits, watchdog, or the time the rhythm analysis takes to compute.
- **People.** The responder is a script: stops compressions a fixed time after being told, restarts a fixed time after "Start CPR", may re-attach pads, and (when scripted) presses on the chest or gives breaths. Real people hesitate, talk, touch the patient.
- **Real pad impedance.** The compression-sensing channel (decision D-36) is a cartoon: each compression is a smooth bump of a fixed size, each breath a slow bump, each heartbeat a small dip; the sizes come from published figures and from the Paris recordings (see `plant.rs`), but real signals are far messier (see `crates/aed-cpr-eval/REPORT.md`). How the front end behaves after a shock, and the charger's noise, are guesses.
- **Real heat.** The device's temperature is one number for the whole inside (`src/thermal.rs`), following the surroundings with a first-order lag (630 J/K, 0.49 W/K: about 21 minutes) and warmed by the charges burned in the internal load, the charger's losses and 0.5 W of electronics, all parts-study figures, not measurements. The capacitor, the cells, the discharge resistors and the pads are not separate; each sensor can be given an offset instead. No sunlight on the case, no pouch, no heating of the cells by the current they deliver.
- **Patient physiology.** Rhythms change only when the script says so (or after a shock, if the script says that shock works). Pad resistance is fixed unless scripted.

## How to run it

From `firmware/`:

```
cargo run -p aed-sim -- --list                 # the scenarios
cargo run -p aed-sim -- adult-vf               # one scenario: timeline + summary
cargo run -p aed-sim -- timings                # timing table for every scenario
cargo run -p aed-sim -- campaign 300 1         # random campaign: 300 runs from seed 1
cargo run -p aed-sim -- campaign 300 1 --no-chest   # the same, firmware without compression sensing
cargo run -p aed-sim -- campaign 300 1 --extended   # adds compression-sensing variations (touches, breaths, ...)
cargo run -p aed-sim -- random 154             # replay one campaign run by its seed
cargo run -p aed-sim -- record-demo > rec.txt  # a record download (D-09) from a simulated week
cargo run -p aed-sim -- read-record rec.txt    # check its checksum and summarise it
```

Add `--release` for speed (a 170-second rescue simulates in about 0.2 s; the 300-run campaign in about 4 s on 8 cores). The program exits with an error code if any safety rule was broken.

Tests:

```
cargo test -p aed-sim                                         # everything
cargo test -p aed-sim -- --nocapture                          # with printed tables
cargo test -p aed-sim --test findings -- --ignored --nocapture  # the known problems (these FAIL)
```

## Scenarios

| Name | What happens |
|---|---|
| `adult-vf` | Adult in VF at 80 Ω; one shock converts it to a normal rhythm |
| `adult-vf-two-shocks` | First shock fails, second converts |
| `child-vf` | Child mode, 60 Ω; one shock |
| `normal-rhythm` | Normal rhythm throughout; must never charge |
| `asystole` | Flatline; must never charge |
| `vf-converts-during-countdown` | VF becomes normal 0.5 s into the countdown; must abort and empty the capacitor |
| `cancel-during-countdown` | Cancel pressed during the countdown |
| `pads-off-during-charge` | A pad comes off 3 s into the analysis, while the capacitor charges; the responder re-attaches it |
| `pads-off-during-countdown` | A pad comes off during the countdown; "Check pads." then "Attach pads", never "Start CPR" in between (finding F-6) |
| `pads-190-throughout` | Pads on but reading 190 Ω (above the 175 Ω shock range) throughout: analyses, "Press pads firmly", "Poor pad contact. No shock.", CPR; never silent, never only "attach pads" (D-32) |
| `pads-190-pressed-to-150` | Pads at 190 Ω; the responder presses them when told and they read 150 Ω: shock (D-32) |
| `stray-cancel-at-switch-on` | Cancel pressed for 0.5 s during the switch-on self-test: ignored, one shock (D-31) |
| `high-resistance`, `low-resistance` | 175 Ω and 25 Ω, the ends of the range |
| `noisy` | VF with noise, mains hum and baseline wander |
| `rapid-vt` | Rapid ventricular tachycardia |
| `fine-vf` | Small VF, which the analysis deliberately answers "not sure" |
| `slow-hands-off` | The responder takes 2.5 s to stop compressions; the shock still comes (finding F-1, fixed) |
| `never-stops-compressing` | The responder never stops compressions: every analysis is "not sure" until its time limit, then "Unable to analyse. No shock." and CPR; never a shock, never silent |
| `clock-drift` | Controllers' clocks 2 % apart |
| `slow-link` | 40 ms message delay, 5 % of messages lost |
| `sensor-tolerance` | The two voltage sensors at opposite ends of the D-28 accuracy: +1 % + 7 V and −1 % − 7 V (more cases in `tests/voltage_sensors.rs`) |
| `f12-compressions-150-over-flatline` | Flatline; the responder never stops compressing, at 150/min (finding F-12). With compression sensing: never shocked. Without it (`chest_sensing: false`): the flatline is shocked |
| `f12-compressions-150-over-normal-rhythm` | The same over a normal rhythm: never shocked |
| `stops-very-late` | The responder takes 3.5 s to stop compressions; the analysis waits for hands off and the shock still comes |
| `touch-during-countdown` | A firm 0.8 s press on the chest 1.5 s into the countdown: "Movement detected. Shock delayed.", analysed again, then shocked |
| `ventilations-during-analysis` | Breaths every 6 s and no compressions from the first analysis on: the analysis is not blocked |
| `strong-pulse`, `fast-rhythm-with-pulse` | A 0.1 Ω heartbeat in the pad impedance at 75 and 160/min: never read as compressions |
| `small-compressions` | Compressions of only 0.2 Ω (seen in some real patients): below the detector's threshold and not seen (documented weakness) |
| `fault-impedance-lost` | The compression-sensing channel dies at switch-on: analysis held for 5 s, then heart signal only, shock given |
| `fault-<name>` | One per injectable fault, listed below |
| `hot-device-rescue` | Switched on at 63 °C (out of a hot car) into 25 °C air: "Device too hot. Shocks delayed until it cools.", analyses without charging, "Shock advised. Device too hot to charge." and CPR, shocks once cooled (D-51) |
| `warm-device-rescue` | VF at 57 °C inside: above the 55 °C pre-charge limit it charges only once a shock is advised |
| `temp-sensor-failed-main` | The main controller's capacitor temperature sensor dead from switch-on: still shocks |
| `temp-sensor-stuck-supervisor` | The supervisor's temperature sensor stuck at 80 °C: distrusted by both controllers, still shocks |

Injectable faults (each can be switched on at any time or a set time after the device enters a given state): main controller's voltage sensor wrong; supervisor's voltage sensor wrong; charger cannot reach its target; charger runs away past its target; discharge path stuck (will not empty); main controller freezes; message wire drops; pads fall off; cancel pressed (1 s); cancel stuck down (D-31); battery depleted; output bridge fails to fire. Sensors can also be given gain and offset errors or a stuck reading, the charger a regulation error (it charges to a multiple of the voltage asked for), each controller's clock a drift, and the message wire a delay and a loss rate. The responder can be made to never stop compressions, and to press the pads down when told ("Press pads firmly"), which changes the pad resistance to a scripted value.

## What it models

- **The device.** A 100 µF capacitor (from the waveform design), a charger delivering a constant 30 W (a full adult charge takes about 6 s), two internal discharge paths of 4,000 Ω each (path A conducts unless the main controller holds it, path B unless the supervisor does; together 2,000 Ω as before the split, one alone twice as slow), a slow leak, and the output bridge with its interlock, sequence latch and patient relay, modelled in time (see "The shock as a process in time"). The charger cannot run while the discharge path is active (the `aed-core` rule `charger_may_run`, a requirement on the future hardware); a switch (`charger_gated_by_discharge`) turns that rule off so a test can show what happens without it. The supervisor's own fault alarm (buzzer and light) appears in the timeline as the supervisor's alarm. The shock itself is integrated step by step (the same exponential as the `aed-waveform` model, so a completed sequence matches the model's prediction for the same starting voltage and resistance to within 0.2 %); whatever charge is left stays on the capacitor. The plant adds up the energy the charger puts into the capacitor and the energy burned off in the internal load, so each run's summary shows the battery and heat cost of charging during analysis (D-29): charges burned off without a shock, charger output and energy burned off inside, in joules (model figures; charger losses are not modelled).
- **Two separate controllers.** The real `Rescue` (with the real rhythm analysis and shock planner) and the real `Supervisor`. Each sees the device only through its own sensors and clock, and they talk only through a message wire.
- **The patient.** A scripted rhythm, pad resistance (or pads off), optional interference, compression artifact while the responder is doing CPR.
- **Compression sensing (D-36).** A pad-impedance channel at 125 samples a second, fed to the real `aed_cpr::CompressionDetector` inside the main controller: the patient's resistance plus each compression (1.5 Ω by default, in step with the compression artifact in the heart signal), breaths, the heartbeat of a beating heart (0.05 Ω by default), touches, noise (5 mΩ), and after a shock 0.3 s saturated then a 3 Ω offset recovering over 1.5 s (a guess). The ground truth "hands on the chest" (compressing or touching) is checked at every shock. `Scenario::chest_sensing = false` runs the firmware without compression sensing, so its effect can be measured.

## Safety rules checked in every run, at every 1 ms step

1. The output bridge never fires unless the patient's **true** rhythm (from the script, not from the analysis) is VF or VT at that moment.
2. It never fires with the pads off or the true resistance outside the 25–175 Ω shock range (the wider 25–250 Ω pad-sensing range of D-32 does not count).
3. It never fires while cancel is pressed, or while the main controller is not in its "delivering" state.
4. The true capacitor voltage never goes more than 50 V above the overvoltage limit.
5. If the device ends in its Fault state, the capacitor is empty (or still emptying in the background within the main controller's own time limit), unless the discharge path itself is stuck; in that case the device must report the more serious "could not empty" fault.
6. The capacitor is not left charged at the end unless a shock is being prepared (which, with charging during analysis, includes an analysis under way) or it is still emptying in the background.
8. With compression sensing, the bridge never fires while the responder's hands have been on the chest (ground truth) for longer than 1 s (`HANDS_ON_ALLOWANCE_MS`, the detector's design bound for seeing a compression). Every shock fired with hands on at all is reported with how long they had been on.
7. The responder is never left without an instruction: any purely informational message ("Shock delivered.", "No shock advised.", "Unable to analyse. No shock.", "Poor pad contact. No shock.") is followed by an instruction within 4 s, and the device speaks within 8 s of switching on. The supervisor's fault alarm counts as an instruction ("continue CPR"; the simulated responder restarts compressions when it sounds). If the main controller freezes, the supervisor's alarm must come on within 4 s; there is no exemption for a frozen main controller.
9. The charger never starts while the TRUE capacitor or cell temperature is above the 60 °C charging limit (D-41, D-48), unless a temperature sensor has been made faulty in the scenario (the firmware is designed to carry on then rather than block every shock).
10. Every shock sequence delivers its plan (D-59): it is never cut short by a consent withdrawn mid-pulse, never ended by the hardware time cap or over-current trip without an injected fault to explain it, and a completed sequence delivers within 1 % of what the `aed-waveform` model predicts for the same start voltage and resistance (`ShockNotAsPlanned`).

Rules 1-3 and 8 are checked at the first conducting moment of each sequence (when the hardware latches it), not at the end of the pulse.

Rule 1 cannot hold for a rhythm change in the last few seconds before the shock: the analysis looks at a 4-second window. The random campaign counts those cases separately (see "Known problems").

Other tests check that the modelled shock energy is within ±15 % of the target at 25, 50, 80, 125 and 175 Ω in both modes, measure the delivered energy at the edges of the voltage window the checks allow (`tests/voltage_sensors.rs`; known problem 9, now closed), report the battery and heat cost of charging during analysis per rescue against the earlier design (`tests/energy_and_timing.rs`), check that a frozen main controller in charging, countdown or delivering leaves the capacitor empty, the charger off and the supervisor's alarm on (`tests/frozen_main.rs`), sweep how late compressions may stop (`tests/late_hands_off.rs`), check the D-47 fresh analysis window (`tests/fresh_window.rs`), run the periodic self-test over simulated weeks (`tests/standby.rs`), and run a seeded random campaign of 300 scenarios (random rhythm, changes, resistance including just out of range, mode, responder, clocks, and one random fault at a random time in about half of them).

## Timings (simulated, provisional settings)

From `cargo run -p aed-sim -- timings`. These come from provisional settings in `aed-core/src/config.rs`, `aed-rhythm/src/settings.rs` and this crate's `PlantConfig`; none is from the guidelines or the standard yet.

With charging during analysis (D-29) and the 3 s countdown (D-30):

| | Adult VF | Child VF | Before D-29/D-30 (adult / child) |
|---|---:|---:|---:|
| Pads on to first shock | 10.2 s | 10.2 s | 18.2 s / 14.2 s |
| Analysis (pads on to "shock advised") | 7.0 s | 7.0 s | 7.0 s / 7.0 s |
| Charge (now during the analysis) | 6.1 s | 2.1 s | 6.0 s / 2.0 s (after the analysis) |
| Wait for the charge after "shock advised" | 0.0 s | 0.0 s | 6.0 s / 2.0 s |
| Countdown | 3.0 s | 3.0 s | 5.0 s / 5.0 s |
| Hands off the chest before the shock | 9.3 s | 9.3 s | 17.3 s / 13.3 s |

The table above is the device **without** compression sensing (`chest_sensing: false`), which is unchanged. With compression sensing (D-36) the analysis waits for the chest to be still, which lengthened the pause (finding F-14); decision D-47 then removed most of the settling second the restarted analysis paid (the analysis window now starts on filters that ran through CPR, after a 0.25 s guard):

| Adult and child VF, with compression sensing | Before D-47 | With D-47 |
|---|---:|---:|
| Pads on to first shock | 12.0 s | 11.2 s |
| Analysis ("Analysing" to "shock advised") | 8.8 s | 8.0 s |
| Hands off the chest before the shock | 11.1 s | 10.3 s |
| Hands off before the second shock (`adult-vf-two-shocks`) | 10.9 s | 10.1 s |
| Latest compression stop after "Analysing" that still gets a shock from the first analysis (`tests/late_hands_off.rs`, 400-1,500 µV VF) | 3.5 s | 4.0 s |
| Flatline, compressions stopping 0.5 s / 2 s / 4 s late: "Start CPR" after | 7.0 / 8.0 / 10.1 s | 6.3 / 7.2 / 9.3 s |

(Without sensing: 9.3 s, as before.) D-47 recovers 0.8 s of the 1.8 s that compression sensing added. Two side effects, measured: the rhythm-change blind spot of finding F-4 for a change to flatline grew from 3.25 to 3.5 s before the shock (the windows now fall 0.75 s earlier relative to the countdown; a change to normal rhythm is unchanged at 1.75 s); and in the 2,000-run campaign (seed 1) VF/VT runs shocked went from 433 to 437 of 581 (extended: 417 to 418 of 582), shocks within 4 s of VF/VT ending from 7 to 8 (extended: 9 to 8), shocks with hands on the chest unchanged at 7 in the extended campaign (the same seeds), and no new rule violation. `tests/fresh_window.rs` checks that compressions ending just before the window (normal rhythm 40-180/min and flatline, artifact 0.5-10 mV, 60-180/min, stopping at 250 ms steps) never lead to a shock, that VF is still shocked, and that nothing changes without sensing.

For context only (from the project's research notes): trained EMTs reach about 51 s from recognition to shock, and each extra 5 s of pause before a shock is associated with lower survival. The pause is now the 7 s analysis (1 s settling, a 4 s window, then 3 agreeing windows) plus the 3 s countdown; the charge is hidden inside the analysis. A charger weaker than about 27 W in this model (184 J to reach the charged reading, in under 7 s) would no longer finish within the analysis and part of the wait would come back. The same pause repeats before every later shock.

The cost (simulated, model energies, `tests/energy_and_timing.rs`): every analysis that ends without a shock burns a charge off inside the device, about 150 J when "no shock" comes at 5 s into the charge and 190 J for a full adult charge. A 170 s rescue with a normal rhythm now burns about 307 J (before: 8 J, the self-test only); a one-shock adult VF rescue about 188 J (before: 39 J). In the 5,000-run campaign: 0.7 charges burned per run, 108 J per run on average.

## The shock as a process in time (D-59, review R-1)

The independent review of the circuit design (finding R-1) found that the plant used to fire the whole shock in one 1 ms step, which hid a real failure: with real hardware the pads move from the measuring front ends to the bridge a few milliseconds before the pulse, so both controllers' pad readings go open, and during the pulse the capacitor voltage leaves the "at plan" window within tens of microseconds. The firmware re-checked both on every tick while consent was given, so it would have withdrawn its own consent before or during every shock. Decision D-59 (consent starts a shock; the hardware completes it) is now implemented in `aed-core` and the plant models the sequence in time:

- while anything in the shock path is moving, the plant steps in 10 µs (`PlantConfig::shock_step_us`); the rest of the simulation stays at 1 ms;
- the interlock rail rises as soon as both consents (and both holds) are present, and powers the relay coil and drivers 1 ms later; the relay's rest contact opens 2 ms after its coil is powered and the bridge contact closes at 6 ms and bounces for 1 ms; the phase timer starts once the position sense has read "bridge" for 0.5 ms; the relay returns 5 ms after the rail drops; pad readings stay open for 300 ms after it returns (all model figures, `PlantConfig`);
- the sequence latch, the 25 ms non-retriggerable time cap, the 90 A over-current trip, the 250 ms permit-duration cap and the 20 ms re-arm lockout are `aed_core::interlock::Latch`, the same pure rules the circuit is being drawn to;
- the consent lines are sampled at every step; `PlantConfig::sequence_latch = false` gives the hardware of the first design (consent is continuous power) for comparison;
- converters can sample every N ms (`voltage_sample_ms`, `pad_sample_ms`; default 1, the review quotes 8 ms for the chosen voltage converter);
- new fault injections (not in the random campaign's list, so its choices stay comparable): relay fails to transfer, fails to return, four times slow; phase timer stuck in phase 1; phase timer toggling; output shorted; request or permit stuck on; request or permit toggling; and any consent line forced low at a chosen microsecond (`Plant::consent_cut`).

Results (simulated, model figures; `tests/consent_during_shock.rs`, run with `--nocapture` for the tables):

**The old consent behaviour fails against this plant** (adult VF, 80 Ω, converters sampling every 8 ms, at each of the 8 sampling phases; the old checks are reproduced by the test harness, `Simulation::set_legacy_consent_parts`, not by the firmware):

| Consent behaviour | Hardware | Runs | Full shock | No shock | Cut short mid-pulse |
|---|---|---:|---:|---:|---:|
| Both controllers as before D-59 | first design | 8 | 0 | 8 | 0 |
| Both controllers as before D-59 | latched (D-59) | 8 | 0 | 8 | 0 |
| Only the supervisor as before D-59 | first design | 8 | 0 | 5 | 3 (79, 53, 20 J of 150 J) |
| Only the supervisor as before D-59 | latched (D-59) | 8 | 3 | 5 | 0 |
| D-59 (now) | latched (D-59) | 8 | 8 | 0 | 0 |

**Consent withdrawn at every instant** (bare plant, request or permit forced low at every 50 µs from before the rail to after the end, 25, 80 and 175 Ω): every run delivered either nothing or the whole plan (to 0.1 %), never part of it and never more. With the first design's hardware the same sweep at 80 Ω gave 164 part-shocks out of 536. Whole system (`adult-vf`, a line forced low 0 to 24 ms after the request, both lines): no shock up to 9 ms, the full 150 J from 10.5 ms, no fault either way.

**Time from consent to energy delivered** (adult plan): 14.0 ms at 25 Ω, 14.6 ms at 50 Ω, 16.8 ms at 80 Ω, 20.2 ms at 125 Ω, 24.0 ms at 175 Ω (8.5 ms of rail, relay and settling plus the pulse). From the main controller's request: 17.8 ms in `adult-vf`; median 27.6 ms and longest 91 ms over the 2,000-run campaign (slow wires and clock drift).

**Worst-case energy under each fault** (adult plan):

| Fault | Result |
|---|---|
| Phase timer stuck in phase 1 | Cut by the 25 ms cap: 158 J at 25 Ω (plan 153), 173 J at 50 Ω, 178 J at 80 Ω, 179 J at 125 Ω, 173 J at 175 Ω (plan 150 J): up to 1.19 times the plan, monophasic. Without any cap: 158 to 185 J. **The cap limits the duration, hardly the energy**: 25 ms is several time constants at most resistances, so the stored charge (190 J) is what bounds it |
| Phase timer toggling (review R-4) | Cut at 25.0 ms: 169.5 J at 80 Ω (plan 150 J); one sequence only |
| Output shorted | Over-current trip in the first 10 µs step (382 A in the model, which has no inductance), 0 J into the patient |
| Second sequence within one permit | Locked out (request toggled 20 times with the permit up): one sequence |
| Request or permit stuck on | One normal sequence (150.0 J), then locked out |
| Request or permit toggling at 1 kHz | The rail never stays up long enough for the relay: no shock (0 J) |
| Relay fails to transfer | No shock ("Shock not delivered", CPR) |
| Relay fails to return | The shock is delivered (150 J), then a fault (`PatientRelayStuck`) |
| Relay four times slow | Delivered in full |
| Rail held with no sequence | Dropped and locked out by the hardware after 250 ms |

**Before and after, the whole simulator** (same seeds):

| | Before D-59 (instantaneous shock) | After (time-resolved) |
|---|---:|---:|
| 2,000-run campaign: VF/VT runs shocked | 437 of 581 | 438 of 581 |
| Extended campaign: VF/VT runs shocked | 418 of 582 | 416 of 579 |
| Extended campaign: shocks with hands on the chest | 7 | 7 (the same seeds) |
| Shocks within 4 s of VF/VT ending (known F-4) | 8 | 8 |
| Shock sequences not completed / not as planned | (not modelled) | 0 / 0 |
| Hands off before the shock (`adult-vf`) | 10.3 s | 10.4 s |
| Switch-on self-test (`adult-vf`) | 0.79 s | 0.95 s |

The changes in the campaigns are timing shifts: the switch-on self-test is 0.17 s longer (the interlock checks and the per-path discharge check), so faults injected at fixed times land in different steps (one weak charger now hits the self-test's charge step; one voltage sensor going wrong mid-test is now caught by the path B check); the extended campaign's "no fault" set lost three runs that were shocked before for the same reason. `touch-during-countdown` now waits 0.5 s longer before its second countdown: after the hold-back the capacitor empties through path A only (the supervisor already holds path B for the new arming), which takes twice as long as both paths.

## Between rescues: the periodic self-test (D-15)

`src/standby.rs` runs the device over simulated weeks in about a second: time between wakes is skipped, not simulated. The real `aed_core::TestSchedule` decides at each wake; each test it asks for runs millisecond by millisecond on a fresh simulated device (the same plant as the rescues) with the real `PeriodicTestRun` and the real supervisor. The surroundings are scripted: in use, on the dock, temperature, battery, a charger that weakens, a plant fault, the timer restarting. Not modelled: anything that happens while asleep (self-discharge, ageing, the dock's power path).

`tests/standby.rs` (simulated): eight quiet weeks give 8 full-energy tests (each charging to about 1,950 V) and 48 daily checks (400 V), one a day, all ending with the capacitor truly empty; a charger that drops to 12 W fails the weekly test (it reaches about 1,650 V in the time limit) but not the daily check, the device shows "not ready", the test is retried daily (7 more charges, each counted) and a repair clears it; the same weak charger in a rescue passes the switch-on test, faults at the first charge and keeps guiding CPR, and once repaired the rescue shocks; a hot spell puts the weekly test off, counts it missed and shows "not ready" from the grace day on; a test due while in use runs as soon as the device is switched off; a stuck discharge path found by the daily check sounds the alarm and stops further tests; a timer restart makes both tests due at once.

## Heat (D-41, D-48, D-51, D-52, D-53)

`src/thermal.rs` is the thermal model (see "What it does NOT model"). Rescue scenarios set the surroundings and the device's starting temperature (`Scenario::ambient_c`, `interior_c`), and can change the surroundings or fail a sensor (`Action::SetAmbient`, `Action::TempSensor`); every run's summary reports the device's temperature. The standby simulation runs the real `aed_core::Standby` (self-test schedule, heat record, capacitor wear) with weather as a function of time (`StandbyScenario::weather`), sensor faults and offsets, pad changes, and the battery being taken out (the durable record is saved and restored; the timer restarts). It checks at every wake that no test charges outside its limits, the dock never charges outside 0-45 °C, the device is never "ready" while above 60 °C or within 30 minutes of it, nothing durable is lost across a power loss, and the heat record never loses the hottest reading.

`tests/heat.rs` (all SIMULATED, one-node model):

- **Hot device rescue** (63 °C into 25 °C air): two shocks held back for heat with "Shock advised. Device too hot to charge." and "Start CPR." at once; cool enough after about 2.4 minutes, first shock at about 4.7 minutes, at the next analysis (it never charges during CPR).
- **Long no-shock rescue on a 50 °C day** (30 minutes of asystole, device starting at 50 °C): 15 charges burned off inside, the device peaks at **53.9 °C**, below the 55 °C pre-charge limit, so the rule does not act and the 60 °C limit is not reached (the parts study's estimate was about 54 °C after 30 minutes).
- **The same in a 57 °C car** (device starting at 58 °C, asystole for 20 minutes then VF): charging during every analysis, 7 charges burned and the device reached **60.1 °C**; the VF was then held back three times ("Shock advised. Device too hot to charge.", CPR) and not shocked in the 6 minutes simulated. With the 55 °C pre-charge rule no charge is burned, the device peaks at **58.2 °C** and the VF is shocked at once (charging after the advice).
- **Standby:** a hot afternoon in a car (ready, not ready above 60 °C, cooled below 45 °C for 30 minutes, re-tested, ready: about 47 minutes after the cabin cooled); a dashboard at 78 °C latches "service needed" through cooling and later tests; a 9-hour warm spell at 56 °C; the battery taken out (a gap, re-test, nothing durable lost); dead and stuck temperature sensors (main and supervisor); the pad heat allowance reached and cleared by a new pad set; capacitor wear "service soon" then "service due" with the weekly test continuing; dock charging stopped in the cold and the heat; and **240 days of mixed weather** (seasons, day and night, hot car afternoons, one dashboard, two power losses, an off-dock week, a use, a sensor offset) with every invariant held: about 76,000 wakes, 252 tests, 14 too-hot episodes, 43 hours above 60 °C, the heat record wrapped (3,598 hours overwritten) with its peak (85.0 °C) kept.

`tests/record.rs`: a record written by the firmware (`aed_core::write_record`) after a simulated week with a hot afternoon, plus a simulated rescue's event record, is read back by the independent host reader (`src/record.rs`) field by field; a damaged record fails its checksum, and an unknown version is refused.

## Known problems found (integration findings)

Each open one has an `#[ignore]`d test in `tests/findings.rs` that states the right behaviour and fails today (only F-4 is still open). The fixes belong in other crates or in design decisions, not here. Fixed findings keep their test in `tests/findings.rs`, no longer ignored.

1. FIXED. Compressions that continued more than about 1.5 s into an analysis made the first answer "not sure", and the rescue sequence acted on it: "No shock advised" on a patient in VF. Now "not sure" is not final while analysing. Measurements: `tests/late_hands_off.rs` (simulated: with the 12 s analysis limit a shock still comes when compressions stop up to 6.0 s late for 400 µV VF, 7.0 s for 800 µV, 7.5 s for 1,500 µV; the hands-off pause from the moment compressions stop is now about 7.5 to 9.5 s, down from 16 to 17.5 s before D-29/D-30; the late-stop tolerance is unchanged).
2. FIXED. Two healthy voltage sensors with opposite 2 % errors tripped the supervisor's single 50 V disagreement limit. Now the agreement budget grows with the voltage and is separate from the "at the planned voltage" window. Tests: `tests/voltage_sensors.rs`.
3. FIXED (needs a hardware rule). A frozen main controller left the charger switched on (about 245 V indefinitely) and the device went silent. Now the charger cannot run while the discharge path is active, and the supervisor has its own fault alarm. Tests: `tests/frozen_main.rs`.
4. OPEN. A rhythm change shortly before the shock is not seen. Measured (`measure_how_close_to_delivery_a_rhythm_change_is_still_caught`): a change from VF to a normal rhythm is missed if it happens 2.0 s or less before the shock (before D-29/D-30: 1.75 s or less); to a flatline, 3.25 s or less (before: 3.5 s). The blind time is about the same, but the pause before the shock is now 10 s instead of 18 s, so it is about a fifth to a third of that pause instead of a tenth to a fifth. The random campaign counts these shocks separately.
5. FIXED. The supervisor recorded a spurious "permit refused: already used" after a good shock when the message wire is slow. It now ignores a "delivering" that arrives within the permit window plus a heartbeat timeout of its permit.
6. FIXED. After a pads-off abort the device said "Start CPR." and "Attach pads" at the same moment. It now goes straight to "Attach pads" (after "Check pads." if a shock had been advised).
7. FIXED. A failing charger at switch-on kept the device silent for about 12 s. The self-test charge step now has a limit scaled to its voltage (1 s at 400 V). This was also the cause of the two "silent at switch-on" runs in the earlier 5,000-run campaign (seeds 430 and 3263: a weak charger injected during the self-test).
8. FIXED by D-32. Pads on but just outside 25–175 Ω got only "Attach pads" forever. Pads now count as on up to 250 Ω (see `pads-190-throughout`).
9. FIXED by D-28. With sensors accurate to ±2 % ± 10 V a child shock could reach about 60.5 J (+21 %) at 25–35 Ω. With ±1 % ± 7 V sensors and the window set from them the top edge is 1,193 V true and the worst child case is +14.2 % at 25 Ω (model; 0.8 points inside the placeholder tolerance); every edge in both modes is inside ±15 %. Tests: `child_energy_at_the_top_of_the_voltage_window_is_within_tolerance`, `delivered_energy_at_the_edges_of_the_allowed_voltage_window`.
10. FIXED by D-59 (review R-1). The plant fired the whole shock in one step, so it could not show that both controllers withdrew consent before or during every real shock. Now time-resolved; see "The shock as a process in time". Tests: `tests/consent_during_shock.rs`.
11. FIXED. Found by the time-resolved plant with two discharge paths: after a hands-on hold-back the main controller lets go of path A and analyses again while the supervisor holds path B for the new arming; on a slow wire the falling voltage and the main controller's heartbeat-old claim disagreed for more than 500 ms and the supervisor latched a voltage-disagreement fault (extended campaign, seed 1758). The main controller now says in its status that it is emptying, and the supervisor does not compare during that, for at most the 3 s emptying limit. Tests: `emptying_through_one_path_while_the_supervisor_still_holds_is_not_a_disagreement`, `a_main_controller_that_says_it_is_emptying_is_not_compared_for_a_bounded_time`.
