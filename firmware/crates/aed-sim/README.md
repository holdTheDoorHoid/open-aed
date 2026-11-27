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
| `pads-off-during-charge` | A pad comes off while charging; the responder re-attaches it |
| `high-resistance`, `low-resistance` | 175 Ω and 25 Ω, the ends of the range |
| `noisy` | VF with noise, mains hum and baseline wander |
| `rapid-vt` | Rapid ventricular tachycardia |
| `fine-vf` | Small VF, which the analysis deliberately answers "not sure" |
| `slow-hands-off` | The responder takes 2.5 s to stop compressions |
| `clock-drift` | Controllers' clocks 2 % apart |
| `slow-link` | 40 ms message delay, 5 % of messages lost |
| `sensor-tolerance` | The two voltage sensors read 1 % high and 1 % low |
| `fault-<name>` | One per injectable fault, listed below |

Injectable faults (each can be switched on at any time or a set time after the device enters a given state): main controller's voltage sensor wrong; supervisor's voltage sensor wrong; charger cannot reach its target; charger runs away past its target; discharge path stuck (will not empty); main controller freezes; message wire drops; pads fall off; cancel pressed; battery depleted; output bridge fails to fire. Sensors can also be given gain and offset errors or a stuck reading, each controller's clock a drift, and the message wire a delay and a loss rate.

## What it models

- **The device.** A 100 µF capacitor (from the waveform design), a charger delivering a constant 30 W (a full adult charge takes about 6 s), a 2,000 Ω internal discharge load that empties the capacitor unless *both* controllers hold it open (the `aed-core` interlock rule), a slow leak, and the output bridge, which fires only when the interlock rule allows. The shock itself is the `aed-waveform` model, started from the voltage actually on the capacitor and delivered into the patient's actual resistance; whatever charge is left stays on the capacitor.
- **Two separate controllers.** The real `Rescue` (with the real rhythm analysis and shock planner) and the real `Supervisor`. Each sees the device only through its own sensors and clock, and they talk only through a message wire.
- **The patient.** A scripted rhythm, pad resistance (or pads off), optional interference, compression artifact while the responder is doing CPR.

## Safety rules checked in every run, at every 1 ms step

1. The output bridge never fires unless the patient's **true** rhythm (from the script, not from the analysis) is VF or VT at that moment.
2. It never fires with the pads off or the true resistance outside 25–175 Ω.
3. It never fires while cancel is pressed, or while the main controller is not in its "delivering" state.
4. The true capacitor voltage never goes more than 50 V above the overvoltage limit.
5. If the device ends in its Fault state, the capacitor is empty, unless the discharge path itself is stuck; in that case the device must report the more serious "could not empty" fault.
6. The capacitor is not left charged at the end unless a shock is being prepared.
7. The responder is never left without an instruction: any purely informational message ("Shock delivered.", "No shock advised.") is followed by an instruction within 4 s, and the device speaks within 8 s of switching on.

Rule 1 cannot hold for a rhythm change in the last few seconds before the shock: the analysis looks at a 4-second window. The random campaign counts those cases separately (see "Known problems").

Other tests check that the modelled shock energy is within ±15 % of the target at 25, 50, 80, 125 and 175 Ω in both modes, and run a seeded random campaign of 300 scenarios (random rhythm, changes, resistance including just out of range, mode, responder, clocks, and one random fault at a random time in about half of them).

## Timings (simulated, provisional settings)

From `cargo run -p aed-sim -- timings`. These come from provisional settings in `aed-core/src/config.rs`, `aed-rhythm/src/settings.rs` and this crate's `PlantConfig`; none is from the guidelines or the standard yet.

| | Adult VF | Child VF |
|---|---:|---:|
| Pads on to first shock | 18.2 s | 14.2 s |
| Analysis (pads on to "shock advised") | 7.0 s | 7.0 s |
| Charge | 6.0 s | 2.0 s |
| Countdown | 5.0 s | 5.0 s |
| Hands off the chest before the shock | 17.3 s | 13.3 s |

For context only (from the project's research notes): trained EMTs reach about 51 s from recognition to shock, and each extra 5 s of pause before a shock is associated with lower survival. The simulated pre-shock pause is long because the device analyses for 7 s (1 s settling, a 4 s window, then 3 agreeing windows), only then starts charging (charging during analysis is a planned later optimisation), and counts down for 5 s. The same pause repeats before every later shock.

## Known problems found (integration findings)

Each has an `#[ignore]`d test in `tests/findings.rs` that states the right behaviour and fails today. The fixes belong in other crates or in design decisions, not here.

1. Compressions that continue more than about 1.5 s into an analysis make the first answer "not sure", and the rescue sequence acts on that first answer: "No shock advised" on a patient in VF, every time.
2. Two healthy voltage sensors with opposite 2 % errors trip the supervisor's 50 V disagreement limit and disable the device.
3. A frozen main controller leaves the charger switched on; the supervisor empties the capacitor into the discharge load, but the charger keeps feeding it (about 245 V indefinitely). The device also goes silent, because the supervisor has no way to speak.
4. A rhythm change in the last ~4 s before the shock is not seen.
5. The supervisor records a spurious "permit refused" after a good shock when the message wire is slow.
6. After a pads-off abort the device says "Start CPR." and "Attach pads" at the same moment.
7. A failing charger at switch-on keeps the device silent for about 12 s, because the 400 V self-test charge waits as long as a full charge.
8. Pads that are on but read just outside 25–175 Ω get only "Attach pads" forever.
