# aed-sim

A simulator that runs the real firmware parts together on an ordinary computer, against a simulated device and a simulated patient. It is the first place the three firmware parts (`aed-core`, `aed-rhythm`, `aed-waveform`) run as one system.

> **SIMULATION ONLY. NOT A MEDICAL DEVICE.** Passing here shows that the firmware parts fit together and that the safety rules hold *inside this model*. It says **nothing** about a real device: no hardware exists, the heart signals are cartoons, and the electronics are a few lines of arithmetic. Nothing here is safe or ready for use on a person or animal. See [SAFETY.md](../../../SAFETY.md).

## What it does NOT model

Read this before reading any result.

- **Real heart signals.** The patient's heart signal comes from the made-up generators in `aed-rhythm`'s `synth` module. Real cardiac-arrest recordings are messier and are not used here.
- **Real electronics.** The capacitor, charger, discharge load and output bridge are simple equations (constant-power charging, exponential emptying, the `aed-waveform` shock model). No switch voltage drops, wiring inductance, capacitor dielectric absorption ("charge creeping back"), component tolerances, heating or ageing.
- **Switch behaviour.** The output bridge fires instantly and perfectly when the interlock allows it. Real switches can fail half-on, ring, or leak.
- **Noise coupling.** The shock does not disturb the controllers, the sensors or the message wire. The heart-signal input is simply blanked for 1.5 s after a shock.
- **Processor timing.** The controllers tick at exact intervals (main every 2 ms, supervisor every 1 ms). No interrupt latency, jitter, stack limits, watchdog, or the time the rhythm analysis takes to compute.
- **People.** The responder is a script: stops compressions a fixed time after being told, restarts a fixed time after "Start CPR", may re-attach pads. Real people hesitate, talk, touch the patient.
- **Patient physiology.** Rhythms change only when the script says so (or after a shock, if the script says that shock works). Pad resistance is fixed unless scripted.

## How to run it

From `firmware/`:

```
cargo run -p aed-sim -- --list                 # the scenarios
cargo run -p aed-sim -- adult-vf               # one scenario: timeline + summary
cargo run -p aed-sim -- timings                # timing table for every scenario
cargo run -p aed-sim -- campaign 300 1         # random campaign: 300 runs from seed 1
cargo run -p aed-sim -- random 154             # replay one campaign run by its seed
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
| `fault-<name>` | One per injectable fault, listed below |

Injectable faults (each can be switched on at any time or a set time after the device enters a given state): main controller's voltage sensor wrong; supervisor's voltage sensor wrong; charger cannot reach its target; charger runs away past its target; discharge path stuck (will not empty); main controller freezes; message wire drops; pads fall off; cancel pressed (1 s); cancel stuck down (D-31); battery depleted; output bridge fails to fire. Sensors can also be given gain and offset errors or a stuck reading, the charger a regulation error (it charges to a multiple of the voltage asked for), each controller's clock a drift, and the message wire a delay and a loss rate. The responder can be made to never stop compressions, and to press the pads down when told ("Press pads firmly"), which changes the pad resistance to a scripted value.

## What it models

- **The device.** A 100 µF capacitor (from the waveform design), a charger delivering a constant 30 W (a full adult charge takes about 6 s), a 2,000 Ω internal discharge load that empties the capacitor unless *both* controllers hold it open (the `aed-core` interlock rule), a slow leak, and the output bridge, which fires only when the interlock rule allows. The charger cannot run while the discharge path is active (the `aed-core` rule `charger_may_run`, a requirement on the future hardware); a switch (`charger_gated_by_discharge`) turns that rule off so a test can show what happens without it. The supervisor's own fault alarm (buzzer and light) appears in the timeline as the supervisor's alarm. The shock itself is the `aed-waveform` model, started from the voltage actually on the capacitor and delivered into the patient's actual resistance; whatever charge is left stays on the capacitor. The plant adds up the energy the charger puts into the capacitor and the energy burned off in the internal load, so each run's summary shows the battery and heat cost of charging during analysis (D-29): charges burned off without a shock, charger output and energy burned off inside, in joules (model figures; charger losses are not modelled).
- **Two separate controllers.** The real `Rescue` (with the real rhythm analysis and shock planner) and the real `Supervisor`. Each sees the device only through its own sensors and clock, and they talk only through a message wire.
- **The patient.** A scripted rhythm, pad resistance (or pads off), optional interference, compression artifact while the responder is doing CPR.

## Safety rules checked in every run, at every 1 ms step

1. The output bridge never fires unless the patient's **true** rhythm (from the script, not from the analysis) is VF or VT at that moment.
2. It never fires with the pads off or the true resistance outside the 25–175 Ω shock range (the wider 25–250 Ω pad-sensing range of D-32 does not count).
3. It never fires while cancel is pressed, or while the main controller is not in its "delivering" state.
4. The true capacitor voltage never goes more than 50 V above the overvoltage limit.
5. If the device ends in its Fault state, the capacitor is empty (or still emptying in the background within the main controller's own time limit), unless the discharge path itself is stuck; in that case the device must report the more serious "could not empty" fault.
6. The capacitor is not left charged at the end unless a shock is being prepared (which, with charging during analysis, includes an analysis under way) or it is still emptying in the background.
7. The responder is never left without an instruction: any purely informational message ("Shock delivered.", "No shock advised.", "Unable to analyse. No shock.", "Poor pad contact. No shock.") is followed by an instruction within 4 s, and the device speaks within 8 s of switching on. The supervisor's fault alarm counts as an instruction ("continue CPR"; the simulated responder restarts compressions when it sounds). If the main controller freezes, the supervisor's alarm must come on within 4 s; there is no exemption for a frozen main controller.

Rule 1 cannot hold for a rhythm change in the last few seconds before the shock: the analysis looks at a 4-second window. The random campaign counts those cases separately (see "Known problems").

Other tests check that the modelled shock energy is within ±15 % of the target at 25, 50, 80, 125 and 175 Ω in both modes, measure the delivered energy at the edges of the voltage window the checks allow (`tests/voltage_sensors.rs`; known problem 9, now closed), report the battery and heat cost of charging during analysis per rescue against the earlier design (`tests/energy_and_timing.rs`), check that a frozen main controller in charging, countdown or delivering leaves the capacitor empty, the charger off and the supervisor's alarm on (`tests/frozen_main.rs`), sweep how late compressions may stop (`tests/late_hands_off.rs`), and run a seeded random campaign of 300 scenarios (random rhythm, changes, resistance including just out of range, mode, responder, clocks, and one random fault at a random time in about half of them).

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

For context only (from the project's research notes): trained EMTs reach about 51 s from recognition to shock, and each extra 5 s of pause before a shock is associated with lower survival. The pause is now the 7 s analysis (1 s settling, a 4 s window, then 3 agreeing windows) plus the 3 s countdown; the charge is hidden inside the analysis. A charger weaker than about 27 W in this model (184 J to reach the charged reading, in under 7 s) would no longer finish within the analysis and part of the wait would come back. The same pause repeats before every later shock.

The cost (simulated, model energies, `tests/energy_and_timing.rs`): every analysis that ends without a shock burns a charge off inside the device, about 150 J when "no shock" comes at 5 s into the charge and 190 J for a full adult charge. A 170 s rescue with a normal rhythm now burns about 307 J (before: 8 J, the self-test only); a one-shock adult VF rescue about 188 J (before: 39 J). In the 5,000-run campaign: 0.7 charges burned per run, 108 J per run on average.

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
