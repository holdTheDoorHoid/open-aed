**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# Revision 0.3 check simulations

27 November 2026, ngspice 42, written with AI assistance. These runs check four circuits that hardware design
revision 0.3 (`docs/HARDWARE_DESIGN.md` §V3) introduces or re-values. Every number is **simulated** with
hand-built models (the same ones as `../charger-storage/models/datasheet_models.lib` and `../bridge/`), not
makers' models. Nothing here is safe or ready for use on a person or animal, and nothing here is an instruction
to build or power anything.

Run: `bash run.sh` (about 4 minutes). Results: `out/summary.txt`.

## In plain language

1. **The discharge paths' "held" sensors (two per path now) work in the cold with old parts** once four small
   changes are made together: a lower-voltage reference diode, weaker pull-up resistors on the interlock sheet,
   a small resistor between the hold pump and the transistor it holds, and a fixed 1.5 kHz hold signal from the
   firmware. Without the small resistor, a release-circuit part at the top of its tolerance can lock the path
   "open" at power-up whenever the hold signal is faster than about 1.6 kHz in the cold.
2. **The new carrier-loss detector on each high-side switch's floating supply works as intended**: it ignores
   the normal tiny gaps in the carrier and empties the supply within about 1.2 µs of the carrier stopping
   (8.8 µs without it), so the switch's gate falls about 3–4.5 µs sooner.
3. **Per-phase time caps** cut the worst shock a frozen main controller could deliver from about 180 J to
   **171 J** at the normal 1,950 V (181 J at the top of the allowed charge window, 2,009 V).

## 1. "Held" sensor thresholds — `held_sensor_rev3.cir`

Revised sensor: 3.9 kΩ + **MMSZ4687 4.3 V** zener (onsemi MMSZ4678T1/D Rev 13: 4.09–4.52 V at 50 µA) + TLP187
LED ∥ 22 kΩ; pull-up on sheet 09 (the table's run used 22 kΩ; the adopted 47 kΩ only lowers the high corner further,
see §2). Corners as `../charger-storage/held_detector.cir`.

| Corner | Threshold, revised (gate magnitude) | Threshold, as drawn | Revised with its zener shorted | Load per sensor at −6.0 V / −7.4 V |
|---|---|---|---|---|
| Low (70 °C, min zener, CTR 10,000 %) | **4.95 V** | 5.39 V | 0.89 V | 196 / 509 µA |
| Nominal | **5.62 V** | 6.27 V | 1.25 V | 145 / 449 µA |
| High (−20 °C, max zener, aged CTR 250 %) | **6.12 V** | 6.97 V | 1.49 V | 97 / 387 µA |

The drawn row reproduces `../charger-storage/notes.md` exactly (5.39 / 6.27 / 6.97 V). The low corner keeps
**0.71 V** above the IXTT2N170D2's worst cut-off (4.0 V × 1.06 = 4.24 V, DS100418B as read in
`../charger-storage/notes.md`); it would keep only 0.18 V above the IXTT10N100D2's 4.77 V, so **this sensor
change is valid only together with the change to 1,700 V devices**. A shorted zener in one sensor reads "held"
at about 1 V; the other sensor on its own line still requires ≥ 4.95 V.

## 2. Hold pump, release and two sensors — `hold_pump_rev3.cir`

Circuit: `../charger-storage/hold_pump.cir` as drawn plus: MMBFJ270 pilot (onsemi MMBFJ270/D Rev 2:
V_GS(off) 0.5–2.0 V), two sensors as §1, **47 kΩ** pull-ups, a **470 Ω** resistor between the last pump stage
and the bottom gate (`rpo`), clock stopped **low**. Defaults: −20 °C, 3.1 V clock behind 40 Ω, capacitors at 70 %,
aged sensors, pilot at its 2.0 V maximum, 1.5 kHz. "Held" = both lines below 1.2 V.

| Case | Held after power-up | Hold, worst point | Highest HELD_N while held | Gate past −1.86 V after stop | "Not held" after stop |
|---|---|---|---|---|---|
| 1.0 kHz | 32.9 ms | −5.58 V | **4.52 V — chatters** | 1.4 ms | 1.4 ms |
| 1.35 kHz | 28.8 ms | −5.96 V | 0.76 V | 1.7 ms | 1.7 ms |
| **1.5 kHz** | 27.8 ms | −6.09 V | **0.65 V** | 1.8 ms | 1.7 ms |
| 1.65 kHz | 27.2 ms | −6.10 V | 0.65 V | 1.8 ms | 1.8 ms |
| 2.0 kHz | 32.5 ms | −5.53 V | **4.74 V — chatters** | 1.9 ms | 1.9 ms |
| X1: 1.65 kHz **without** the 470 Ω, 22 kΩ pull-ups | **never** (release locked on, gate −1.2 V) | | | | |
| X2: 2 kHz without 470 Ω, 470 Ω in the release drain instead | **never** | | | | |
| X3: 2 kHz without 470 Ω, four-stage pilot pump | **never** | | | | |
| Pilot 0.5 V (slowest release), cold, 1.5 kHz | 27.8 ms | −6.09 V | 0.65 V | 13.5 ms | 9.9 ms |
| Fresh parts, 3.5 V, 25 °C, 1.65 kHz, pilot 0.5 V | 19.2 ms | −6.33 V | 0.55 V | **22.2 ms** | **19.3 ms** |
| 70 °C, 3.2 V, 1.35 kHz, pilot 0.5 V | 20.6 ms | −5.90 V | 0.55 V | 13.6 ms | 11.3 ms |
| Nominal 25 °C, 1.5 kHz, pilot 1.2 V | 26.5 ms | −5.88 V | 0.58 V | 4.0 ms | 4.0 ms |
| 70 °C, clock stopped **high** (firmware rule broken) | 19.8 ms | −6.01 V | 0.55 V | **not within 30 ms** | 19.1 ms |
| One sensor only (comparison) | 27.2 ms | −6.21 V | 0.62 V | 1.9 ms | 1.8 ms |

The 1 ms clock gap at 40 ms never released a held path (HELD_N ≤ 0.61 V during it).

Findings: (a) the power-up lock-out of `../charger-storage/notes.md` Q2 is a loop — the release switch shorts the
pump, the loaded clock cannot pump the pilot past its pinch-off — and a 470 Ω pump-output resistor breaks it;
a drain resistor or a stronger pilot pump do not. (b) Below about 1.3 kHz and above about 1.7 kHz an aged sensor
in the cold reads "held" only on the peaks of the ripple (HELD_N swinging to 4.5 V), so the firmware must hold the
clock at **1.5 kHz ± 10 %**. (c) Release with the clock stopped low takes up to 22 ms (pilot at its 0.5 V
minimum), longer than the 10 ms the design used to state; not a safety limit (dump 0.8–2 s, disarm 3 s), but every
firmware wait on "not held" must allow 25 ms. (d) Stopped high, the path does not release for > 30 ms: the
firmware rule "always stop low" is essential; the 1.01 MΩ gate resistor still releases it in 0.7–1.6 s.
**Not settled:** the model is hand-built (BAT54 drops at datasheet maxima, X7R derating 70 %); the window
1.35–1.65 kHz must be confirmed by measurement at −20 °C. The ZENER sub-circuit's `tbv1` is ignored by ngspice 42
(warning in the log), so zener temperature drift is in neither these results nor `../charger-storage`'s.

## 3. Carrier-loss detector — `carrier_loss.cir`

Island as drawn on sheet 13 plus the detector of `docs/HARDWARE_DESIGN.md` §B6.3 (BAV70 copy rectifier,
4.7 kΩ / 47 pF / 4.7 kΩ, MMBT3904 inverter with BAT54 Baker clamp, 10 kΩ from the **gate node**, 2N7002 with 100 Ω
across the rectified node). Secondary ±18.5 V, 400 kHz, 0.19 µs dead time, 20 Ω [EST WP-G]. Generic transistor
models [EST]; 2N7002 threshold swept 1.0–2.5 V.

| Case | Detector node while carrier runs (2N7002 must stay below its 1.0 V minimum threshold) | Rectified node < 1 V after stop | PNP starts | Gate < 2 V after stop |
|---|---|---|---|---|
| Gate fully charged, as drawn | — | 8.8 µs | 0.24 µs | 11.5 µs |
| Gate fully charged, detector (V_th 1.0 / 1.8 / 2.5 V) | 0.41 V | **1.1 / 1.2 / 1.3 µs** | 0.24 µs | **6.8 / 6.9 / 7.0 µs** |
| Gate part-way (carrier 5 µs, gate 4.7 V), as drawn | — | 4.9 µs | 0.97 µs | 7.2 µs |
| Gate part-way, detector (V_th 1.8 / 2.5 V) | 0.38 V | **1.2 / 1.3 µs** | 0.95 µs | **3.9 / 4.1 µs** |
| Low carrier (BRIDGE_5V 4.46 V), detector | 0.41 V | 1.1 µs | 0.27 µs | 6.4 µs |

A first version powered the detector from the rectified node itself; it stalled once that node fell to the
2N7002's threshold (gate < 2 V only after 9–10 µs). Powering it from the gate node fixed that.
**Disagreement to record:** in this circuit-level model the PNP starts within ≈ 1 µs even as drawn and the gate
does not keep rising after the carrier stops (4.73 → 4.79 V), whereas `../bridge/q1_shorted_lowside.cir`'s
behavioural PNP (which ignores the PNP base current pulling the rectified node down) gave ≈ 2.7 µs and a rising
gate. Both models are hand-built; the bridge result is the more pessimistic and the fault-energy figures there
(11–13 mJ with changes 1 + 2 + 3) assumed an ideal 100 Ω switch at the stop. **Second round:** re-run
`q1_shorted_lowside.cir` with this detector at circuit level.

## 4. Per-phase caps — `per_phase_caps.sh`

`../bridge/q3_shock_energy.cir` unchanged; adult; the cap at the **top** of each band (phase 1 11.85 ms, phase 2
8.05 ms; arithmetic in `docs/HARDWARE_DESIGN.md` §B13.3). Every case ran to completion (five needed the Gear
integrator; the result lines say so).

| Patient | 25 Ω | 50 Ω | 75 Ω | 100 Ω | 125 Ω | 175 Ω |
|---|---|---|---|---|---|---|
| 1,950 V, phase 1 stuck | 159.7 | **170.9** | 169.0 | 161.9 | 152.9 | 134.8 |
| 1,950 V, phase 2 stuck | 159.7 | 170.5 | 170.3 | 167.2 | 163.4 | 156.4 |
| 2,009 V, phase 1 stuck | 169.5 | **181.4** | 179.4 | 171.8 | 162.3 | 143.1 |
| 2,009 V, phase 2 stuck | 169.5 | 180.9 | 180.7 | 177.4 | 173.5 | 166.0 |
| As drawn, 25 ms cap (`../bridge/notes.md`, 1,950 V) | 159.8 | 173.1 | 177.9 | — | 180.0 | 176.5 |

Delivered joules, simulated. Planned shock 149–154 J. Worst case **170.9 J at 1,950 V**, **181.4 J at 2,009 V**
(the top of the firmware's at-plan window), against 180.0 J / ≈ 189 J (2,000 V) as drawn. Below about 75 Ω the
capacitor's own energy is the limit, so no time cap can do better there.
