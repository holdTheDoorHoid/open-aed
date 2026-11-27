# System architecture

Version 0.2, 27 November 2026. Decision numbers (D-xx) refer to [DECISIONS.md](../DECISIONS.md).

**Everything in this document is a design on paper and in simulation. No hardware exists. Figures marked *model* come from arithmetic, not measurement.**

## The device in blocks

```mermaid
flowchart LR
    subgraph LV["Low-voltage side"]
        BAT[Rechargeable battery] --> PWR[Power and battery monitor]
        PWR --> MAIN[Main controller]
        PWR --> SUP[Supervisor controller]
        ECG[Heart-signal and pad-resistance front end] --> MAIN
        MAIN --> UI[Speaker, screen, lights, cancel button]
        MAIN --> BLE[Bluetooth]
        MAIN --> LOG[Event record]
    end
    subgraph HV["High-voltage side"]
        CHG[Capacitor charger] --> CAP[Storage capacitor]
        CAP --> BRIDGE[Output bridge]
        CAP --> DUMP[Internal discharge load]
        BRIDGE --> RELAY[Patient isolation]
    end
    PWR --> CHG
    MAIN -- "charge / shock request" --> GATE{Hardware interlock}
    SUP -- "permit" --> GATE
    GATE --> BRIDGE
    SUP -- "independent voltage reading" --- CAP
    SUP -- "force discharge" --> DUMP
    SUP --> ALARM[Supervisor alarm]
    DUMP -. "open: charger off" .-> CHG
    RELAY --> PADS[Pads]
    PADS --> ECG
```

| Block | Job | Notes |
|---|---|---|
| Heart-signal front end | Reads the heart's electrical signal and the resistance between the pads | Off-the-shelf chip; must survive the device's own shock and recover within seconds |
| Main controller | Runs the rescue: analyses the rhythm, charges, counts down, requests the shock, speaks the prompts, keeps the record | Firmware in Rust (D-08) |
| Supervisor controller | A second, separate, much simpler processor that must agree before any shock, and can empty the capacitor on its own | Different chip, own power rail, own voltage reading |
| Hardware interlock | Plain logic, no software: the bridge can fire only when the main controller's request and the supervisor's permit are both present | The third independent check |
| Capacitor charger | Raises battery voltage to about 1,900 volts in under 8 seconds | Starts from a published reference design |
| Storage capacitor | Holds the shock energy: about 100 µF | The largest part in the device |
| Output bridge | Sends the energy to the patient in two phases, forward then reversed, and cuts it off at the planned moment | |
| Internal discharge load | Safely burns off the charge when a shock is cancelled, and stands in for a patient during self-test | Discharges by default: the capacitor is only held charged while the controllers actively hold the discharge path open |
| Patient isolation | Disconnects the high-voltage side from the pads except during the shock itself | |

## Three independent checks before a shock

The device is fully automatic (D-06), so no human presses a button. No single failure, in hardware or software, may deliver a shock that was not advised, or block one that was.

1. **Main controller.** Its rhythm analysis says "shock", the pads are attached, the capacitor is at the planned voltage, the countdown has finished, and cancel has not been pressed.
2. **Supervisor controller.** Checks independently, with its own readings: the capacitor voltage matches what the main controller claims; the pad resistance is in the range for a patient; the main controller is alive (a regular heartbeat signal); the main controller has been in the "armed" state for at least the countdown time; cancel has not been pressed. Only then does it issue a permit, and only for a short window.
3. **Hardware interlock.** The bridge drivers are powered only while both signals are present.

The supervisor also empties the capacitor by itself if: the main controller stops responding; the capacitor stays charged longer than a time limit; the voltage goes above a safe maximum; or the two controllers disagree.

Two further rules came out of simulation (finding F-3):

- **The charger cannot run while the discharge path is open.** Because either controller letting go opens the discharge path, the supervisor can always stop the charger, even if the main controller has frozen with the charger switched on. This is a requirement on the hardware, not something software can provide.
- **The supervisor has its own alarm**, a buzzer or light it alone controls. If the main controller dies it cannot speak, so the supervisor sounds the alarm whenever it has declared a fault, or if the main controller never starts.

What the supervisor does *not* do is second-guess the rhythm. A second opinion on the rhythm is built into the main controller's analysis (two different methods must agree). Whether the supervisor should also run its own simple rhythm check is an open design question.

## The rescue sequence

```mermaid
stateDiagram-v2
    [*] --> SelfTest: switched on
    SelfTest --> AttachPads: passed
    SelfTest --> Fault: failed
    AttachPads --> Analysing: pads on a patient
    Analysing --> Charging: shock advised
    Analysing --> Cpr: no shock advised, or not sure
    Charging --> Countdown: charged
    Countdown --> Delivering: still shockable, pads on, not cancelled
    Countdown --> Disarming: rhythm changed, pads off, or cancelled
    Delivering --> Cpr: shock delivered
    Disarming --> Cpr: capacitor emptied
    Cpr --> Analysing: two minutes of CPR done
    Analysing --> AttachPads: pads came off
    Cpr --> AttachPads: pads came off
    Charging --> Disarming: fault or cancel
    Disarming --> Fault: could not empty the capacitor
    Fault --> [*]
```

Points that matter:

- **"Not sure" never leads to a shock.** The analysis may answer shockable, not shockable, or not sure. While analysing, "not sure" means keep analysing and remind the responder not to touch the patient; if the time limit passes without a clear answer the device says so and goes to CPR. Once the capacitor is charged, "not sure" cancels the shock.
- **The rhythm is checked right up to delivery.** During the countdown the analysis keeps running. If the rhythm stops being shockable, the shock is abandoned and the capacitor emptied.
- **Any fault leads to "continue CPR"**, with the capacitor emptied. A faulty device must never leave a responder waiting for it.
- **Cancel works in every state.**
- Charging while analysing, to save seconds, is a later optimisation. The first version charges only after a shock is advised.

Timings (CPR period, countdown length, time limits) are settings in one place in the firmware, to be set from the current resuscitation guidelines and the purchased standard, not scattered through the code.

## The shock, in numbers (*model*)

The design point follows D-02 and D-03: a biphasic truncated exponential shock from a single capacitor of about 100 µF.

- The capacitor is charged to a fixed voltage for the chosen mode (adult or child).
- The device measures the resistance between the pads before the shock.
- From that resistance it chooses how long each phase lasts, so that the delivered energy lands on the target: longer for high-resistance patients, shorter for low.
- Starting assumptions, to be replaced by a parameter-by-parameter match to the published Philips waveform: first phase between about 3 and 12 milliseconds; second phase about two-thirds of the first; a short gap between them; total no longer than about 20 milliseconds.

The firmware's waveform module computes the full table (charge voltage, phase times, peak current, delivered energy, for every resistance from 25 to 175 ohms, adult and child) and the tests check it. That table is the specification the high-voltage hardware will later be measured against.

## Firmware structure

The firmware is split so that everything that decides anything can be tested on an ordinary computer, with no device attached.

| Part | What it holds | Runs on |
|---|---|---|
| `aed-types` | The shared vocabulary: units, patient mode, rhythm decision, shock plan | Device and computer |
| `aed-rhythm` | Heart-rhythm analysis | Device and computer |
| `aed-waveform` | Shock planning and the electrical model of the discharge | Device and computer |
| `aed-core` | The rescue sequence, the supervisor's rules, self-test, prompts, event record. Talks to hardware only through a defined interface. | Device and computer |
| `aed-sim` | A simulated device and patient: plays heart rhythms in, models the capacitor and the patient's resistance, injects faults | Computer only |
| *(later)* board support | The thin layer that connects `aed-core` to a real processor's pins | Device only |

Rules for the firmware:

- No hardware-specific code outside the board-support layer. The processor is not chosen yet (the Rust compiler qualification question is unresolved), and this keeps that choice open.
- No dynamic memory, no unsafe code, and no code path that can crash in the parts that run on the device.
- The main controller and the supervisor are written as separate pieces that share no state, even in simulation, so the simulation can test what happens when one of them fails.
- Every safety rule in this document has a test that names it.
- Third-party code is kept to the minimum. Each addition is listed with the reason, because the medical software standard requires every outside component to be justified and tracked.

## What is deliberately not designed yet

- Which processor, which heart-signal chip, which capacitor. These wait for the open technical questions in [PLAN.md](PLAN.md).
- Circuit schematics for the high-voltage side. These wait for the patent review and the lab safety policy.
- Pads, enclosure, battery pack.
