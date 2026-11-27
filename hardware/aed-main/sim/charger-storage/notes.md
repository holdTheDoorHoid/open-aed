**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# Charger and storage/discharge simulations (sheets 10_hv_charger and 11_hv_storage)

27 November 2026. First circuit simulation of this design (ngspice 42). Every number below is
**[SIM]** unless marked otherwise, and every simulation uses models that were **built by hand from
datasheet figures**, not makers' models (see "Models" at the end). Nothing here is an instruction to
build, power or test anything (SAFETY.md §2a).

## In plain language

1. **The discharge paths have a weakness that adding a fifth transistor does not fix.** Each path is a
   chain of transistors that share the capacitor's voltage. If the second transistor from the bottom
   fails as a short circuit, the way the chain is wired makes the bottom transistor carry almost the
   whole voltage (about 2,140 V at 2,200 V) — twice its rating, whether the chain has four or five
   transistors, and even with 1,700 V transistors. Earlier notes expected "half the voltage". A changed
   wiring of the transistors' control connections (a diode in each, plus a resistor and capacitor across
   each transistor) and 1,700 V transistors bring the simulated worst case back inside the rating with
   margin. That change is proposed below; it needs a second check by a reviewer.
2. **The "held" sensor's single reference part is a real single point of weakness (I-4 confirmed).** If
   it shorts, the sensor says "held" when the transistor is in fact half on. Putting two reference
   parts in series does not fit the voltage available. Two complete, independent sensors per path, each
   read separately, does fit, but it needs a new wire to the interlock sheet.
3. **The hold-and-release circuit is not dependable at −20 °C with the slowest clock** if the small
   "pilot" transistor happens to be at the high end of its tolerance: the path then never holds (it fails
   safe — the device just cannot charge — but it would be unusable in the cold). Release is slow if the
   controller happens to stop its clock in the "high" state. Two cheap changes: stop the clock "low" in
   firmware, and choose a pilot part with a narrower, lower switching voltage.
4. **Emptying the capacitor works as designed:** about 1.6 s from 2,200 V to under 60 V through one path,
   0.8 s through both; each dump resistor takes about 80 J per dump, a fifth of its rating.
5. **Charging is faster than the earlier hand estimate** — 5.3 s to 1,950 V in the worst case, against
   the 7 s target — but only because the transformer's own losses are not in the model (the transformer is
   made to order and has no data yet). If the transformer wastes more than about a quarter of the energy,
   the 7 s target is missed. The switch never sees more than about 79 V (rated 100 V).
6. **The three over-voltage limits can be made to fit** by moving them: the firmware limit down to a
   2,070 V reading, the hardware comparator down to 2,110 V and made to latch, and the charger chip's own
   limit moved *above* the normal range as a last resort (which then relies on the capacitor maker
   confirming it can take 2,420 V briefly).
7. **A 10 MΩ bleed resistor inside the capacitor** would empty it to under 60 V in about an hour if it
   were ever disconnected, at a cost of 0.4 W and a few hundredths of a second of charge time.

## How to run

`bash run.sh` (all, about 1–2 hours) or `bash run.sh quick` (skips the two long Monte Carlo files).
Full logs and `out/summary.txt` are written to `out/` (not committed). Every netlist ran to completion
in batch mode on 2026-11-27; the remaining `Error:` lines in some logs are measurements that are
*meant* not to occur (for example "never released") and are reported as `-1` or blank in the result
line, never as a number.

---

## Q1 (I-3, I-4): discharge-stack voltage sharing and the "held" sensor

Netlists: `stack_static_n4.cir`, `stack_static_n5.cir` (with `stack_static.ctl`), `stack_dynamic_n4.cir`,
`stack_dynamic_n5.cir` (with `stack_dynamic.ctl`), `held_detector.cir`.

Device ratings used:
- IXTT10N100D2: V_DSX 1,000 V (T_J 25–150 °C), ≈ 940 V at −20 °C (normalised 0.94, Fig. 11) —
  Littelfuse/IXYS DS100326A(4/17) "Preliminary Technical Information", as read in WP-F §3.1.
- IXTT2N170D2: V_DSX 1,700 V (T_J 25–150 °C), BV_DSX ≥ 1,700 V at −5 V gate, V_GS(off) −2.0…−4.0 V,
  I_DSX(off) ≤ 25 µA (25 °C) / 500 µA (125 °C), R_DS(on) ≤ 6.5 Ω, same TO-268 package — IXYS
  DS100418B(02/12), read 2026-11-27 from a datasheets.com mirror of the IXYS PDF. Its cold derating is
  **assumed** equal to the 10N100D2's (0.94 → 1,598 V at −20 °C); its Fig. 12 curve was not read.

### Static sharing (stack held, DC) — configurations

cfg 1 = as drawn; cfg 2 = as drawn with IXTT2N170D2; cfg 3/4 = proposed gate steering with 10N100D2 /
2N170D2; cfg 5 = cfg 4 plus 10 MΩ and 3.3 nF directly across each transistor (the recommendation).
"Cold" = −20 °C (cut-off ×0.93, leakage ≈ 0.04× of 25 °C, rating ×0.94); "hot" = +70 °C (cut-off ×1.06,
leakage ≈ 4× of 25 °C [EST, log interpolation of the 25/125 °C datasheet maxima]).

| Case, capacitor at 2,200 V | cfg 1 drawn, 4 × 1 kV | 5 × 1 kV drawn | cfg 2: 4 × 1.7 kV drawn | cfg 4: 4 × 1.7 kV steered | cfg 5: 4 × 1.7 kV steered + RC |
|---|---|---|---|---|---|
| Nominal, per device | 541–559 V | 429–450 V | 542–557 V | 546–554 V | 546–554 V |
| String ±2 % worst | 575 V | — | — | — | — |
| Device 2 (just above the bottom) shorted, cold | **2,141 V** (2.28 × rating) | **2,116 V** (2.25 ×) | **2,138 V** (1.34 ×) | 1,158 V (0.72 ×) | **1,158 V (0.72 ×)** |
| Device 3 shorted, cold | 1,100 V (1.17 ×) | 1,090 V (1.16 ×) | 1,100 V (0.69 ×) | 1,073 V (0.67 ×) | 895 V (0.56 ×) |
| Top device shorted, cold | 743 V | 738 V (top: 559 V) | 744 V | 1,020 V | 799 V |
| One sharing-string resistor shorted, cold | 742 V | 561 V | 741 V | 737 V | 737 V |
| Leakage mismatch, hot (bottom device 2 % of max, upper devices at max) | 724 V (0.72 ×) | 595 V (0.60 ×) | 768 V (0.45 ×) | 1,007 V (0.59 ×) | 704 V (0.41 ×) |
| Monte Carlo, 100 runs, no fault, worst | 712 V hot | 584 V hot | 753 V hot | 917 V hot | 677 V hot |
| Monte Carlo, 100 runs, one random device shorted, worst | 2,199 V | 2,199 V | 2,199 V | **1,759 V cold (1.10 ×)** | **1,169 V cold (0.73 ×), 1,174 V hot** |

(At a normal 1,950 V charge the as-drawn device-2-short case is 1,896 V; device-3-short 975 V.)

**What the simulation shows that the paper analysis missed.** In the drawn circuit each upper
transistor's gate is tied to its tap through 100 kΩ, and a 15 V clamp (TVS) sits between gate and
source. When a transistor shorts, the transistor *above* it holds the shorted node up near its own tap,
the shorted transistor's clamp then conducts from that node into the tap below, and that current (up to
milliamps, against a 55 µA string) drags the whole string upward. The result: a short of device 2 puts
≈ 97 % of the capacitor voltage on the bottom device; a short of device 3 gives the V/2 that WP-F F-1
predicted; a short of the top device gives about V/3. **Adding a fifth device does not change the
device-2 case**, and 1,700 V devices do not survive it either. This also corrects HARDWARE_DESIGN §B5.4
("727 V with one device shorted").

**The proposed fix (cfg 4/5):** feed each upper gate through a high-voltage diode (anode at the tap
side) bypassed by 22 MΩ, raise the gate-source resistor to 100 MΩ, stiffen the string to 4 × 4.7 MΩ,
and — essential, see dynamic sharing — put 10 MΩ and 3.3 nF directly across each transistor. The diode
stops a shorted device's clamp from dragging the tap; the per-device resistor keeps a leaky device from
taking another's share; the per-device capacitor shares fast edges. With 1,700 V devices this keeps a
single short at about V/2 (≈ 1,160 V, 0.72 of the cold rating).
*Monte Carlo note:* without the per-device resistors (cfg 4) a cold run with a random short and random
leakage/cut-off reached 1,759 V (reproduced in two separate runs): with the gate feed steered, a leaky
device's source is free to float, so the per-device 10 MΩ is needed, not optional. With it (cfg 5) the
100-run worst is 1,169 V cold / 1,174 V hot (0.73 / 0.69 of the rating). **Five devices with the same
fix** give 936 V for the deterministic device-2 short, but one Monte Carlo run reached 1,464 V (0.92 ×):
in that run a second device was also biased on; not explained, so five devices are not recommended over
four on this evidence.

### Dynamic sharing (`stack_dynamic_*`) — stack held, worst C0G pattern (one 3.3 nF −5 %, others +5 %)

| Event | drawn 4 × 1 kV | drawn 5 × 1 kV | steered 4 × 1.7 kV, no per-device RC | steered 4 × 1.7 kV + 10 MΩ/3.3 nF per device |
|---|---|---|---|---|
| Step 1,000 → 2,200 V in 1 µs (bounding) | 605 V | 522 V | **1,338 V** | 626 V |
| Settled at 2,200 V | 559 V | 451 V | 556 V | 553 V |
| 2,200 → 1,200 V in 10 ms (shock-like) | 559 V | 451 V | 556 V | 553 V |
| 1,200 → 2,200 V in 2 ms | 579 V | 469 V | 615 V | 643 V |
| Release (turn-on) from 2,200 V, per device peak (`discharge.cir`) | 604 V | — | 1,045 V (2N170D2, drawn) | — |

The 3.3 nF capacitors on the sheet are across the *tap string*, not across the transistors, so on a
microsecond edge the transistors' own capacitances decide the sharing. In the drawn circuit that is
still acceptable (605 V); with gate steering it is not unless each transistor gets its own capacitor.

### Fewer, higher-voltage parts

No catalogue depletion MOSFET above 1,700 V was found (Littelfuse lists its D2 family "up to 1,700 V";
IXTH/IXTT2N170D2 and IXTA/IXTH1N170DHV are the 1,700 V parts). Three 1,700 V parts per path would give
733 V each when healthy but, with this follower wiring, one short still puts nearly the full voltage on
one part. Four 1,700 V parts with the proposed wiring is the arrangement recommended. The 2N170D2's
6.5 Ω on-resistance adds 26 Ω to a 4.5 kΩ path (dump time +0.4 %) and its cut-off (−2.0…−4.0 V) is
narrower and lower than the 10N100D2's, which helps the "held" margins below.

### "Held" sensor (I-4), `held_detector.cir`

Threshold = the bottom-gate magnitude at which `HOLD_x_HELD_N` falls below 1.2 V (sheet 09 must read
V_IL ≥ 1.2 V, WP-F C-9). Low corner = min zener, low LED V_F, CTR 10,000 %, +70 °C; high corner = max
zener, high LED V_F, aged CTR 250 %, −20 °C.

| Arrangement | Threshold low / nom / high | With one reference element shorted | Pump current at −7.4 V |
|---|---|---|---|
| As drawn (3.9 k + MMSZ4688 4.7 V + LED‖22 k) | 5.39 / 6.27 / 6.97 V | **0.95 / 1.46 / 1.86 V** — reads "held" while the bottom device conducts | 0.29–0.42 mA |
| Zener open | never "held" (HELD_N stays 4.8–5.0 V) — fails safe | | |
| Two 3.9 V zeners in series | 8.29 / 9.49 / 10.41 V | 4.62 / 5.47 / 6.13 V | 0.002–0.013 mA (at 7.4 V it is off) |
| Two complete sensors, transistors in series on one line | 5.50 / 7.01 / 8.30 V | 5.45 / 6.61 / 7.63 V | 0.58–0.84 mA |
| **Two complete sensors, each on its own line (proposed)** | 5.39 / 6.27 / 6.97 V each | the other sensor still needs ≥ 5.39 V | 0.58–0.84 mA |

Cut-off magnitude to beat: IXTT10N100D2 up to 4.5 V × 1.05 (60 °C) ≈ 4.74 V, ×1.06 at 70 °C ≈ 4.77 V;
IXTT2N170D2 up to 4.0 V × 1.06 ≈ 4.24 V.
- Two zeners in series: with one shorted the lowest threshold (4.62 V) is *below* the 10N100D2's worst
  cut-off and only 0.38 V above the 2N170D2's; and intact it needs a hold of about 10.9 V, which the pump
  does not reach (next section). Rejected.
- Two sensors with their transistors in series on one line: the low level of two Darlingtons in series
  is 1.10–1.13 V, against a 1.2 V input threshold. Rejected.
- **Two independent sensors, each read on its own line, "held" only if both are**: each keeps the drawn
  thresholds, 0.62 V above the 10N100D2's worst cut-off (1.15 V above the 2N170D2's) across
  −20…+70 °C. A shorted zener in one sensor leaves the other's full threshold; a shorted Darlington in one
  is likewise covered (this also closes WP-F fault 9). Cost: one more opto, zener, two resistors per path,
  one more line per path to sheet 09 and an AND there, and about twice the sensor current from the pump.
- **Open problem, not settled:** the high-corner threshold (6.97 V, aged opto, cold) is above the hold
  level this model gives at −20 °C / 1 kHz (−5.5 to −6.0 V worst point of the ripple, next section). If
  that is right, an old unit in the cold reads "not held" and cannot charge (safe direction, but
  unavailable). WP-F's behavioural model gave −7.4 V. Needs measurement of the pump at −20 °C.

---

## Q2: discharge, hold pump and release (I-17, I-24)

### Emptying the capacitor, `discharge.cir`

| Case (start 2,200 V) | Time to < 60 V | Peak current per path | Per dump resistor: peak power / energy | Worst transistor: peak power / energy |
|---|---|---|---|---|
| One path, nominal | 1.62 s | 0.49 A | 357 W / 80.5 J | 268 W (instant.) / 0.09 J |
| One path, slowest (R +10 %, cold +5.4 % TCR, C +5 %) | 1.97 s | 0.42 A | 309 W / 84.5 J | 232 W / 0.08 J |
| One path, fastest (R −10 %, hot −5.4 %, C −5 %) | 1.31 s | 0.57 A | 419 W / 76.5 J | 316 W / 0.10 J |
| Both paths, nominal / slowest | 0.81 / 0.99 s | 0.49 / 0.42 A | 357 / 309 W, 40 / 42 J | 268 W / 0.05 J |
| One path, release circuit failed (gate decays via 1.01 MΩ) | 4.39 s | 0.49 A | 357 W / 63 J | 268 W / **13.5 J** |
| As above with IXTT2N170D2 | 5.44 s | 0.49 A | 356 W / 45 J | 268 W / **27 J** |
| One path nominal, IXTT2N170D2 | 1.63 s | 0.49 A | 356 W / 80 J | 268 W / 0.35 J |

Ratings: Ohmite EY 400 J single impulse, 550 V working, 1,100 V overload (5 s) (WP-F §7, catalogue copy).
Each resistor starts at 733 V (one third of 2,200 V), above the 550 V working rating for ≈ 0.13 s per
dump — Ohmite must confirm, as WP-F says. The "release failed" case needs the other path too to meet
the 3 s limit; on its own it takes 4.4–5.4 s and puts 13–27 J into one transistor over about a second
at up to 268 W: inside the 10N100D2's DC line at ≤ 300 V as read by WP-F (≈ 400 W at T_C 75 °C), but
this needs a thermal model and the maker's SOA at mid voltages — **not settled**.

**Repetition.** A dump every 132 s (the transformer's design duty, 14 charges at 132 s, §B4.5): 80.5 J /
132 s = **0.61 W average per resistor**; 20 dumps in 1,300 s (65 s apart): **1.24 W** average; rating
2.5 W at 25 °C (derating curve not seen). Daily self-test dumps are negligible. The resistor's thermal
time constant is not published, so its temperature under repetition was not simulated — **not settled**.

### Hold pump and release, `hold_pump.cir`

Timeline: clock from power-up; a 1 ms clock gap at 40 ms (the firmware's allowed gap) must not release;
clock stopped at 60 ms; restarted at 90 ms. "Conducting" = gate above −2.33 V (10N100D2) / −1.86 V (2N170D2).
Corners: −20 °C, 3.1 V clock, 40 Ω source, capacitors at 70 % (as WP-F); diode drops at datasheet maxima.

| Case | Held after power-up | Hold, worst point (incl. ripple) | 1 ms gap | Conducting after clock stops | "Not held" after stop | Re-held after restart |
|---|---|---|---|---|---|---|
| Drawn, −20 °C, 1 kHz, pilot V_GS(off) 2.5 V | **never** (pilot never pinched off) | −1.2 V | — | — | — | never |
| Drawn, −20 °C, 1 kHz, pilot 2.0 V | 34 ms | −5.95 V | held | 2.2 ms | 1.4 ms | 26 ms |
| Drawn, −20 °C, 1 kHz, pilot 1.5 V | 34 ms | −5.96 V | held | 3.6 ms | 2.6 ms | 26 ms |
| Drawn, −20 °C, 1 kHz, pilot 0.8 V | 34 ms | −5.96 V | held | 7.4 ms | 5.1 ms | 26 ms |
| Drawn, 25 °C, 2 kHz, pilot 1.5 V | 26 ms | −6.69 V | held | 5.1 ms | 3.5 ms | 20 ms |
| Drawn, 70 °C, 1 kHz, pilot 1.5 V, clock stops **high** | 31 ms | −6.17 V | held | **21 ms** | 13 ms | 29 ms |
| Drawn, fresh, 3.5 V, 2 kHz, pilot 0.8 V, stops **high** | 21 ms | −7.43 V | held | **> 30 ms** (not before the restart) | 26 ms | 0.3 ms (never released) |
| Contract 3-stage pump, −20 °C, 1 kHz | 35 ms | −6.02 V | held | 3.5 ms | 2.8 ms | 30 ms |
| Contract 3-stage pump, 25 °C, 2 kHz | 23 ms | −6.80 V | held | 6.2 ms | 4.2 ms | 19 ms |
| 5-stage pump, −20 °C, 1 kHz | 38 ms | −5.48 V | held | 2.9 ms | 2.2 ms | 27 ms |
| Pilot-pump nodes bled by 1 MΩ, fresh, stops high, pilot 1.2 V | 21 ms | −7.43 V | held | 8.8 ms | 7.4 ms | 25 ms |
| as above, 70 °C, 1 kHz, pilot 1.2 V | 31 ms | −6.17 V | held | 6.3 ms | 5.3 ms | 32 ms |
| as above, −20 °C, 1 kHz, pilot 1.8 V | **never** | | | | | |

Findings:
- **Cold lock-out (I-17, worse than "marginal").** At −20 °C and 1 kHz the pilot-pump bias reaches only
  2.2–2.6 V. A pilot at the top of its 0.8–2.5 V range is never pinched off; its release transistor then
  shorts the main pump, which loads the clock and keeps the pilot bias low: the path never holds. Fails
  safe (no charging) but the device is unusable in the cold. With a pilot ≤ 2.0 V it holds in 34 ms.
- **Clock stopped high** is the slow release case (21 ms to > 30 ms) because the pilot-pump capacitors
  keep feeding the pilot's gate. With the clock stopped **low** every simulated corner releases in
  ≤ 7.4 ms and reads "not held" in ≤ 5.1 ms.
- Bleeding the pilot-pump nodes fixes "stopped high" (≤ 8.8 ms) but brings the cold lock-out down to a
  1.8 V pilot. Not recommended on its own.
- **I-24 (stage count):** in this model the loaded hold at −20 °C / 1 kHz is about the same with 3, 4 or 5
  stages (−6.0, −6.0, −5.5 V worst point): with a 40 Ω clock source and 2.2 µF capacitors the clock cannot
  move more charge per cycle, so extra stages add ripple, not voltage. This **contradicts WP-F §4.1/4.2**
  (−7.4 V and the reason for the fourth stage). Low confidence: diode drops are at datasheet maxima.
- The 1 ms clock gap was tolerated in every case that held.
- Start-up is 21–38 ms, not ≤ 5 ms (as WP-F said; firmware already waits for "held").

---

## Q3: charger (I-11), `charger.cir`

Method: on-time from the RL ramp (analytic); flyback simulated transistor-level per cycle at 14 capacitor
voltages; charge time integrated (see the file header for what is and is not modelled — chiefly, **no
transformer core loss or AC winding loss**, because the transformer is made to order and has no data).

| Corner | to 1,150 V (child) | to 1,950 V | to 2,008.6 V | Battery current at 2 kV | Circuit efficiency |
|---|---|---|---|---|---|
| 7.2 V, 25 °C, nominal parts | 1.53 s | 3.74 s | 3.94 s | 9.0 A | 0.90 |
| 8.4 V full | 1.39 s | 3.32 s | 3.49 s | 8.8 A | 0.91 |
| 5.8 V typical cold | 1.79 s | 4.46 s | 4.71 s | 9.3 A | 0.89 |
| **Worst**: 5.5 V, 100 mV sense, R_sense +1 %, L_pri −10 %, hot switch, slow and high-V_F diodes, C +5 % | 2.11 s | **5.29 s** | 5.59 s | 8.8 A | 0.87 |
| Worst with R1001 = 4.7 mΩ | 1.98 s | 4.97 s | 5.25 s | 9.3 A | 0.87 |
| Worst with R1001 = 4.5 mΩ | 1.90 s | 4.76 s | 5.02 s | 9.8 A | 0.87 |
| Worst plus a 10 MΩ bleed in the capacitor (Q5) | 2.11 s | 5.32 s | 5.61 s | 8.8 A | 0.87 |

- The worst case is under 7 s *in this model*. WP-E's 7.4–8.0 s assumed a lumped efficiency of 0.60–0.65.
  The model's circuit efficiency is 0.87; **the transformer's own losses decide the answer**: the 7 s
  target is missed only if core + AC winding losses exceed about 24 % of the throughput (overall
  efficiency below ≈ 0.66). Put a loss figure in the transformer requirement (§B4.5), e.g. ≥ 90 % at
  20 A peak, 30–45 kHz, and the worst case stays ≈ 5.9 s.
- If more margin is wanted: R1001 4.7 mΩ (peak current 21.2–24.0 A) cuts the worst case by 6 %, raises
  battery current by about 6 % and I²R heat by about 13 %, and pushes the peak to 24.0 A against the
  transformer's "≤ 10 % inductance loss at 25 A" requirement and the eFuse's 11.4 A minimum limit
  (WP-E §3.3) — both margins shrink; raise the transformer requirement to 27 A if this is taken.
  Raising L_pri does not help (in boundary mode power ≈ ½ · I_pk · V_T · η, independent of L).
- **Primary switch stress and clamp (8.4 V pack, 22.9 A peak, capacitor at 2,200 V):**

| Transformer leakage | 0.5 % | 1 % (spec limit) | 2 % | 3 % |
|---|---|---|---|---|
| Drain peak | 72 V | 76 V | 78 V | 79 V |
| Clamp (D1003 + D1004) energy per cycle / power | 8 µJ / 0.31 W | 21 µJ / 0.83 W | 37 µJ / 1.44 W | 57 µJ / 2.27 W |
| Snubber R1026 power | 0.15 W | 0.15 W | 0.12 W | 0.10 W |

  The 100 V switch keeps ≥ 21 V margin up to 3 % leakage *with the SMCJ45A modelled at its quoted
  73–75 V at 22 A* (that clamp figure is **unverified**, WP-E §4). Clamp power is below WP-E's 2.2 W
  estimate at 1 % leakage and reaches it at 3 %.

---

## Q4: the three over-voltage limits (I-2, I-22), `ovp_limits.cir`

Window: normal adult charge top 2,008.6 V (§B4.4); capacitor 2,200 V continuous, 2,420 V for 10 s (D-41).
200,000-sample Monte Carlo (uniform within each tolerance) and worst-case sums; they agree within a few volts.

| Limit | As drawn / documented | Proposed |
|---|---|---|
| L1 firmware limit, main chain (1 % + 7 V) | reading 2,150 V → 2,122–2,179 V | **reading 2,070 V → 2,043–2,098 V** (34 V above the window) |
| L2 OVP comparator (sheet 08) | 2,160 V nominal → **2,127–2,193 V** (only 7 V under the rating); releases near 1,000 V | **2,110 V nominal, latching → 2,076–2,145 V**; 2,083–2,138 V if the reference divider is a matched pair |
| L3 LT3751 primary-side trip (RBG) | 931 Ω → **1,988–2,178 V**, 2,265 V with max overdrive — low edge inside the normal window (C1 confirmed) | **RBG 876 Ω → 2,116–2,316 V, 2,409 V with max overdrive**: never below the window (107 V margin); a last-ditch limit that relies on the capacitor's 2,420 V rating |

The dominant OVP term is the supervisor string's ratio (±0.675 %: tolerance, temperature, life); then
the reference divider (±0.43 %) and the ADR3425 (±0.24 %). The primary-side trip cannot be fitted inside
2,008.6–2,200 V by any RBG: its band is ≈ 200 V wide before overdrive (comparator ±2.55 %, turns ±1 %,
secondary drops 19–48 V), wider than the 191 V window.

**Latching comparator (I-22), proposed for sheet 08** (simulated in `ovp_limits.cir`): IN− = a reference of
1.053 V (from the ADR3425 2.5 V through a divider); IN+ = OVP_SENSE with **Rin (VCAPS_BUF → OVP_SENSE) =
Rf (OVP_TRIP → OVP_SENSE) = 100 kΩ 0.1 %**. It trips at tap = 2 × 1.053 V = 2,110 V and, once OVP_TRIP is
high (≥ 2.1 V is enough), IN+ stays above the reference for any tap ≥ 0 V: latched until reset or until
3V3_SUP is removed. Reset: a new net **OVP_RST_N (05 → 08)** through 100 nF + 100 kΩ (AC coupling) into a
BSS138 pulling OVP_SENSE to GND via 1 kΩ. Simulated: power-up untripped; trips at 2,109.6 V; holds 3.15 V
with the tap back at 0 V; clears on a reset edge; with the reset line **stuck high** it still re-trips
(the coupling makes only an edge reset). Remaining latent fault: the reset BSS138 shorted (OVP can never
trip) — needs a self-test path, not designed here.

**Capacitor question for the maker:** with L3 as proposed the capacitor must survive up to 2,409 V (2,316 V
without the overdrive term) for the time the charger takes to stop, as a fault event. Ask for a stated surge
rating, e.g. ≥ 2,420 V for ≥ 1 s, ≥ 100 events over life.

---

## Q5: bleed resistor inside the capacitor (I-5), `bleed.cir`

| Internal bleed | Capacitor alone, 2,200 → 60 V | τ when on the board (with the 6.66 MΩ of board loads) | Droop at 2,000 V | Loss in the capacitor at 2,008.6 V |
|---|---|---|---|---|
| none | never (only dielectric leakage) | 699 s | 2.9 V/s | 0 |
| 4.7 MΩ | 30 min | 289 s | 6.9 V/s | 0.86 W |
| **10 MΩ** | **63 min** | 420 s | 4.8 V/s | **0.40 W** |
| 22 MΩ | 139 min | 537 s | 3.7 V/s | 0.18 W |

Recommendation: **10 MΩ inside the capacitor can** (as several series elements, e.g. 4 × 2.49 MΩ, each
rated ≥ 1 kV, total ≥ 2,420 V), added to the capacitor's made-to-order requirement (PARTS_STUDY Appendix A).
Effect on charge time: +0.03 s at the worst corner (Q3 table). While holding during analysis the charger
restarts a little more often (4.8 V/s instead of 2.9 V/s droop at 2,000 V; regulation restarts 3 V below
target). This is a "no indefinitely stored charge" measure, not a "safe to touch" one: an hour is far too
long for anyone opening a device, and dielectric absorption can bring charge back (SAFETY.md §2).

---

## Recommended changes (for the capture engineer; each needs review)

Sheet 11_hv_storage, both stacks (A shown; B identical):
1. Q1101–Q1104 (and Q1105–Q1108): **IXTT10N100D2 → IXTT2N170D2** (same TO-268 footprint; pinout to be
   checked against DS100418B: 1 gate, 2/4 drain, 3 source — same as the 10N100D2).
2. Add a gate-steering diode in each upper gate feed: R1115 (100 kΩ) → new HV diode (anode to R1115,
   cathode to HV_A_G2), e.g. Vishay GP02-40 (4 kV) **[datasheet not re-read: UNVERIFIED]**, bypassed by a
   new 22 MΩ HV resistor; same for R1116/R1117 and stack B.
3. R1121–R1123 (gate-source) 10 MΩ → **100 MΩ** (HV-rated if a fault can put a tap interval across it).
4. Sharing string R1107–R1110 10 MΩ → **4.7 MΩ** HVCR2512 (standing loss per stack 0.26 W at 2,200 V).
5. Add **10 MΩ and 3.3 nF / 2 kV C0G directly drain-to-source across each of the four transistors**.
   (Items 2–5 together are cfg 5: single short ≤ 1,174 V across Monte Carlo, ≤ 0.73 of the rating.)
6. Held sensing: **duplicate the sensor** (second 3.9 kΩ + MMSZ4688 + 22 kΩ + TLP187 per stack, same
   nodes), each Darlington on its own line; new nets HOLD_A_HELD2_N / HOLD_B_HELD2_N to sheet 09, where
   "held" = both lines low. (Sheet 09 change is outside this work package.)
7. Release pilot Q1112/Q1113: a P-JFET with V_GS(off) ≤ 2.0 V maximum (MMBFJ177's 0.8–2.5 V range is too
   wide) — **no part identified**; and **firmware: always stop HOLD_x_CLK in the low state** when releasing.

Sheet 10_hv_charger:
8. R1019 (RBG) 931 Ω → **876 Ω 0.1 %** (primary trip becomes a last-ditch limit above the window).
9. T1001 requirement: add an efficiency/loss figure (core + winding) at full power, e.g. ≥ 90 %; optional
   R1001 5 mΩ → 4.7 mΩ with the transformer saturation requirement raised to 27 A.

Sheet 08_sense_sup (outside this package, for its owner): OVP comparator re-centred to 2,110 V and made
latching (Rin = Rf = 100 kΩ, reference 1.053 V), new reset net OVP_RST_N with AC-coupled reset.
Firmware (config): over-voltage limit reading 2,150 V → 2,070 V.
Capacitor requirement (PARTS_STUDY Appendix A): internal 10 MΩ bleed; stated surge rating ≥ 2,420 V.

## What cannot be settled without measurement or better models

- Every transistor and diode here is a hand-built model; in particular the depletion MOSFETs' leakage
  at intermediate gate voltages, their behaviour in avalanche, and the 2N170D2's cold breakdown factor.
- The hold-pump level at −20 °C (this model: −5.5 to −6.0 V; WP-F: −7.4 V) and therefore whether an aged
  "held" sensor still reads in the cold.
- Transformer core and AC winding loss (decides whether 7 s is met), leakage inductance, and the SMCJ45A's
  real clamping voltage at 20 A.
- Dump-resistor and transistor temperatures under repeated dumps (no thermal data).
- The LT3751 VOUT-comparator overdrive for 3–5 µs pulses (only a 2 µs figure is published).
- The one unexplained Monte Carlo outlier of the five-device fixed stack (1,464 V).

## Models (provenance)

All in `models/datasheet_models.lib`, written for this work; no third-party model text, so there is no
licence restriction and nothing is git-ignored except `out/`.
- No maker SPICE model could be obtained: no IXYS/Littelfuse model for the D2 depletion parts was found;
  onsemi (BSS138, MMBFJ177) and Diodes Inc. (BAT54S) model links redirected or returned 404 on
  2026-11-27. The LT3751 model exists only as an encrypted LTspice model, which ngspice cannot use.
- Depletion MOSFETs: level-1 with V_TO, R_DS(on), C_iss/C_oss/C_rss and leakage fitted to the datasheet
  limit values (see the file header). Breakdown not modelled.
- BAT54, BAV199, MMSZ4688/86, SMF15CA, SMCJ45A, ES3D, UX-F5B: diode models fitted to the datasheet maxima
  quoted in WP-E/WP-F. Zener temperature coefficient ±0.94 mV/K **[EST]**.
- TLP187: LED diode + current-controlled output with CTR as a parameter and a 0.55 V saturation knee.
- BSS138, MMBFJ177: level-1 MOSFET / JFET fitted to threshold, on-resistance and I_DSS ranges.
- Charger controller: not modelled as a circuit; on-time computed, flyback simulated per cycle.
- GP02-40: from memory of the Vishay datasheet, **not re-read**.
