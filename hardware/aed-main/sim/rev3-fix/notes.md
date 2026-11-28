**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# Discharge-path hold at true temperature: J-2 and J-7 re-examined, and a proposed change

28 November 2026, ngspice 42, written with AI assistance during the simulation audit. Every number here is
**simulated** with the hand-built models of `../charger-storage/models/datasheet_models.lib` (datasheet limit
values, not makers' models). Nothing here has been built or measured, nothing here is safe or ready for use on a
person or animal, and nothing here is an instruction to build or power anything. The specs were **not** edited:
this note says what someone else should draw, and that change still needs review.

## In plain language

Each discharge path is kept switched off ("held") by a small negative voltage on its bottom transistor, made by a
little charge pump that only works while the controller sends it a clock. Stop the clock, remove a supply, or lose
the controller, and the hold must collapse so the capacitor empties by itself. Two sensors per path report "held".

1. **The cold-weather failure reported as J-2 was mostly a measuring mistake.** The earlier runs read the hold
   between 30 and 40 ms after switch-on, but in the cold the pump takes 34–42 ms to settle. Read after it has
   settled, the drawn circuit holds at −20 °C with old parts and a 3.1 V clock, with about 0.7 V to spare.
2. **But the drawn circuit has three real weaknesses at true temperature:**
   - it sits on a cliff: at a 3.0 V clock (0.1 V lower), or with a weaker clock driver (60 Ω), or at a 2 kHz
     clock, it never holds at all in the cold (it "locks out" — safe, but the device cannot charge); and from cold
     it takes up to 42 ms to read "held", against the 35 ms the firmware waits;
   - when the controller freezes with its clock pin **high**, or when the buffer's supply disappears, the hold takes
     **up to 52 ms and 85 ms** to let go, and even a normal stop takes **up to 27 ms** at one hot corner, against the
     25 ms the design allows;
   - the "held" sensor's safety margin is **0.28 V**, not 0.71 V (J-7 is confirmed).
3. **Proposed change (per path: one new chip and three new resistors, plus four value changes, on two sheets):**
   drive the pump with a 5 V clock instead of 3.3 V, make the pump input's pull-down resistor 1 kΩ instead of
   100 kΩ and its series resistor 22 Ω instead of 10 Ω, add three 1 MΩ resistors that drain the release circuit's
   own small pump, and use the 4.7 V zener in the sensors again. In simulation this holds with 2.0 V to spare at the
   cold corner, keeps holding down to a 4.0 V clock, lets go within **17 ms** in every way the clock or supply can
   be lost, and raises the sensor margin to **0.69 V**.

## What the design requires (docs/HARDWARE_DESIGN.md v0.3)

- §B5.4 / firmware rule 4: hold clock 1.5 kHz ± 10 %, stopped low; firmware allows **25 ms for "not held"** after
  stopping and **35 ms for "held"** after starting from cold. Sheet 11 note: both sensors held, HELD_N ≤ 1.0 V, at
  −20 °C, 3.1 V clock, 1.35–1.65 kHz, aged sensors; release ≤ 25 ms stopped low.
- §B15 fault 2: controller frozen with `HOLD_M_CLK` stuck **at either level** → that path conducts (the table still
  says "≤ 10 ms", the v0.2 figure that §B5.4 withdrew). Fault 10b accepts "stopped high" only as a *firmware*
  fault covered by the other path; a frozen controller can leave its pin high with no firmware fault, so this study
  holds **every** loss — stopped low, stopped high, buffer supply removed during a high or a low half-cycle — to the
  same **25 ms**.
- §B5.4: "default discharge" — no clock, no supply or no controller → the path conducts.

## How it was simulated (`hold_fix.cir`, `run.sh`, `run_cases.sh`)

Sheet 11 stack A hold circuit part for part as `../rev3-wpf/hold_held_wpf.cir` (which reads it from
`specs/11_hv_storage.toml`): R1131, R1133, C1111–C1117, D1109–D1112 BAT54S, R1194 470 Ω, C1109 3.3 µF, R1127/R1128,
D1101 SMF15CA, gate capacitance 3.65 nF, Q1109 BSS138 + R1188, Q1112 MMBFJ270 (vp = pinch-off 0.5–2.0 V), pilot
pump C1125–C1130, D1117–D1119 BAV199, R1190/R1191/R1135, both sensors with 47 k + 1 M pull-ups, and the OVP force
opto U1103 (dark, or lit with 8.5 mA in mode 5). The sheet-09 buffer U908 (SN74LVC1G17) is a source of level vh
behind rclk. Timeline: power-up at 0, hold judged at 70–100 ms (settled), 1 ms clock gap at 100 ms, fault at
130 ms, end 230 ms. Temperature is set after `reset`, and every result line prints a BAT54 probe voltage
(0.388 / 0.326 / 0.263 V at −20 / 25 / 70 °C) — a case whose probe does not match is printed as void. A run the
simulator aborts is printed as void, never as a time. "Conducting" = gate above the 2N170D2's smallest cut-off at
that temperature (−1.78 / −2.0 / −2.22 V), the slowest device to turn on.

Corners: cold = −20 °C, pump capacitors at 70 %, aged sensors (CTR 250 %, maximum zener, high LED drop); fresh =
capacitors 100 %, new sensors (CTR 10,000 %, minimum zener, low LED drop) — the slowest release; hot = 70 °C with
20 µA dark current in every opto. Clock 1.35 / 1.5 / 1.65 kHz (also 2 kHz, 60 Ω, 40 % duty). Pilot at 2.0 V (hardest
to pinch off: lock-out) and 0.5 V (slowest release). Drawn drive: 3.1 V behind 40 Ω (WP-F's cold corner) and
3.5 V behind 30 Ω (fresh). Proposed drive: 4.75 V behind 40 Ω and 5.25 V behind 22 Ω [EST: 5V0_SYS ± 5 % and the
translator's output drop; the translator's datasheet was not opened].

Results are in `out/drawn.txt`, `out/recommended.txt`, `out/alternatives.txt`, `out/held_margin.txt` after
`bash run.sh` (about 5–10 minutes).

## 1. The drawn circuit at true temperature [SIM]

| Check | Result | Against the requirement |
|---|---|---|
| Hold, −20 °C, aged, 3.1 V, 1.35 / 1.5 / 1.65 kHz | −6.70 / −6.73 / −6.73 V; HELD_N ≤ 0.58 V | met; 0.72 V over the aged-cold threshold (5.98 V) |
| Hold, 70 °C, aged, 3.1 V | −6.91 / −6.98 V | met |
| Time to a steady "held", −20 °C | 33.9 / 34.5 / **41.7 ms** | 35 ms allowance **exceeded at 1.65 kHz**; 56 ms at 40 % duty |
| 1 ms clock gap | never released | met |
| Clock 3.0 V or 2.9 V, −20 °C, aged, pilot 2.0 V | **never held** (lock-out: gate −1.25 V) | amplitude margin < 0.1 V |
| 3.1 V behind 60 Ω, 2 kHz | **never held** | |
| Release, clock stopped low (worst: 70 °C, fresh, 3.5 V, 1.35 kHz, pilot 0.5 V) | **27.3 ms** conducting, 24.9 ms "not held" | **not met** (25 ms) |
| Release, clock stopped **high** (worst: same corner / −20 °C aged pilot 0.5 V) | **51.7 / 44.6 ms** | **not met** |
| Release, buffer supply removed (high / low half-cycle; worst −20 °C aged pilot 0.5 V) | **83–85 ms** (78.8 ms hot) | **not met** |
| OVP force opto lit (clock running) | conducting in 0.97–0.99 ms | met |
| One sensor's zener shorted, −20 °C | never "held" (gate −4.54 V) | visible at self-test |
| One sensor's zener shorted, 70 °C, new sensors | **"held"** (gate −5.69 V, both lines low) | latent at 70 °C (still safe: the intact sensor needs the real bias) |
| One zener open | never "held" | visible |

Why release is slow when the clock stops high or the buffer loses power: the release transistor's pilot is biased
by its own small pump (C1125–C1130), whose charge drains only through 1.8–2.2 MΩ. With the clock held high, or the
pump input left floating on R1131's 100 kΩ, that stored charge keeps the pilot pinched off for tens of milliseconds.
Why it locks out at low amplitude: at switch-on the pilot conducts, the release transistor shorts the main pump,
the loaded clock then cannot pump the pilot past its pinch-off, and the loop never breaks.

## 2. Recommended change, for someone to draw [SIM, not reviewed]

**Sheet 09 (`specs/09_interlock.toml`, group `hold_drive`), both paths:**
- Keep U908 / U909 (SN74LVC1G17 on 3V3_MAIN / 3V3_SUP, with the input filter) as they are, but their outputs go to
  new local nets `HA_BUF` / `HB_BUF` instead of `HOLD_A_DRV` / `HOLD_B_DRV`.
- Add **U937 / U938: SN74LVC1T45DBVR** (single-bit dual-supply translator), VCCA = 3V3_MAIN / 3V3_SUP (the same
  controller rail as the Schmitt buffer before it), VCCB = **5V0_SYS**, DIR tied to VCCA (A → B), A = `HA_BUF` /
  `HB_BUF`, B = `HOLD_A_DRV` / `HOLD_B_DRV`. Decoupling: 100 nF X7R on each VCCB (C960, C961) and on VCCA if
  C917 / its B twin is not adjacent (C962, C963). Pin numbers, the B-port drive at 4.5 V (≈ 32 mA assumed) and the
  "either supply at 0 V → both ports high-impedance" behaviour are **[UNVERIFIED: TI SN74LVC1T45 datasheet not
  opened in this audit]** and must be checked before drawing; the result relies on that last property.
- Why a translator and not U908 itself on 5V0_SYS: an SN74LVC1G17 at VCC = 5 V needs up to 0.609 × VCC ≈ 3.2 V to
  switch (SCES351Y as read in `../rev3-wpd`), which a 3.3 V controller pin cannot guarantee; and the Schmitt buffer
  must stay in front, because the 100 kHz rejection of the input filter depends on its hysteresis.

**Sheet 11 (`specs/11_hv_storage.toml`), stack A (B identical):**

| Part | Drawn | Proposed |
|---|---|---|
| R1131 (A) / R1132 (B), HOLD_x_DRV pull-down at the pump input | 100 kΩ | **1 kΩ** 1 %, 0.1 W (5 mA while the clock is high: 25 mW) |
| R1133 (A) / R1134 (B), pump clock series resistor | 10 Ω | **22 Ω** (keeps the 5 V buffer's edge current ≤ 40 mA in this model; with 10 Ω it reached 48–50 mA, near the ±50 mA output limit usual for this logic family [UNVERIFIED for the translator]) |
| new R6123–R6125 (A) / R6126–R6128 (B) | — | **1 MΩ** 1 % from each pilot-pump node `RLSA_P1`, `RLSA_P2`, `RLSA_P3` (B: `RLSB_P1..P3`) to `HV_RTN` |
| D1125, D1133 (A) / D1126, D1134 (B), held-sensor zeners | MMSZ4687 4.3 V | **MMSZ4688 4.7 V** (onsemi MMSZ4678T1/D: 4.47–4.94 V at 50 µA; the part drawn before rev 0.3) |
| everything else (pump capacitors and diodes, R1194 470 Ω, pilot, sensors' resistors) | | unchanged |

**Contract (`contract.md`, issue 3 §5.4 table row `HOLD_A_DRV` / `HOLD_B_DRV`):** level changes from 3V3_MAIN to
**5V0_SYS** swing; default resistor on 11 changes from **100 kΩ to 1 kΩ** to GND; the clock rule (1.5 kHz ± 10 %,
stop low, ≤ 1 ms gap) is unchanged. `docs/HARDWARE_DESIGN.md` v0.3 changes for the design authority: §B5.4 hold
bias and its "Removed (R-9)" line (a 5 V drive returns, through a translator that keeps each path tied to its own
controller's rail); the power-rails paragraph "A loss of `5V0_SYS` is a safe failure … the hold-pump buffers run
from the controllers' rails, not from `5V0_SYS`, so they are unaffected" — with this change a loss
of 5V0_SYS releases **both** paths within 11 ms, which is the default-discharge direction and consistent with
"loss of 5V0_SYS is a safe failure", but it is a common cause the text currently excludes; §B5.4 thresholds and
margin (next section).

**What the change does.** The 5 V clock moves the hold and the pilot bias far from the lock-out cliff; the 1 kΩ
pull-down makes a dead buffer look exactly like a clock stopped low; the 1 MΩ bleeds empty the pilot pump's
coupling capacitors when the clock stops at either level (they cost little bias at 5 V, which is why the same
bleeds failed at 3.1 V in `../charger-storage`); the 4.7 V zener spends some of the extra hold on a larger
"held" margin.

### Results of the recommended change [SIM, `out/recommended.txt`]

| Check | Result | Margin |
|---|---|---|
| Hold, −20 °C, aged, 4.75 V, 1.35 / 1.5 / 1.65 kHz | −8.40 / −8.46 / −8.50 V; HELD_N ≤ 0.56 V | **2.05 V** over the aged-cold threshold 6.35 V |
| Hold, 70 °C aged / 25 °C nominal | −8.65 / −8.76 V / −8.48 V | |
| Time to a steady "held" (all corners) | 12.5–21.8 ms (cold aged 18–19 ms) | ≥ 13 ms under the 35 ms allowance |
| 1 ms clock gap (every corner, hot included) | never released, HELD_N ≤ 0.57 V | |
| Clock 4.25 / 4.0 V, −20 °C aged | −7.92 / −7.68 V, held (25.7 ms) | holds 0.75 V (16 %) below the 4.75 V corner |
| Clock 3.5 V | never held (lock-out) | the cliff is between 3.5 and 4.0 V |
| 4.75 V behind 60 Ω at 2 kHz; 40 % duty | −8.18 V; −8.41 V, held | |
| Release, clock stopped **low** (worst: 70 °C, fresh, pilot 0.5 V) | **9.3 ms** | 15.7 ms |
| Release, clock stopped **high** (worst: same) | **16.9 ms** ("not held" 16.4 ms) | 8.1 ms |
| Release, buffer supply removed in a high / low half-cycle (worst −20 °C aged, pilot 0.5 V) | **10.8 / 9.7 ms** | 14.2 ms |
| OVP force opto lit, strongest pump (fresh, 5.25 V) with an aged opto | conducting in 1.52–1.55 ms | (drawn: 0.97–0.99 ms) |
| Edge current from the buffer (sampled every ≤ 10 µs) | ≤ 35 mA (40 mA with a shorted zener) | |
| One zener open | never "held" | visible |
| One zener **shorted**, −20 °C aged | held only marginally (gate −6.41 V against the intact sensor's 6.35 V; HELD_N 0.80 V, 2.7 V during a 1 ms gap) | may or may not show at self-test: treat as **latent** |
| One zener **shorted**, 70 °C new | "held" (gate −7.77 V) | **latent** |

Release worst case over all 54 cases of the recommended set: **16.9 ms** to conducting, **16.4 ms** to "not held".
Gate never more negative than −9.8 V (the SMF15CA starts to clamp at 16.7 V; the 2N170D2's gate is rated ± 30 V).

**One thing gets worse.** With the drawn 3.3 V pump, a shorted sensor zener in the cold overloads the pump so the
path reads "not held" and the fault shows at the next self-test (WP-F R3.6). With the stronger pump the path still
holds, so that fault becomes latent — as it already is at 70 °C in the drawn circuit (§1). It stays safe: the other
sensor of the same path still reads "held" only when the gate is really past its own threshold, which is what the
two-sensor design (I-4) relies on. A self-test that releases one sensor at a time cannot see it; this is for the
design authority to accept or to cover with a sensor-current check.

## 3. J-7: the "held" sensor margin at true temperature [SIM, `held_margin_fix.cir`]

Lowest threshold (minimum zener, low LED drop, CTR 10,000 %, 20 µA dark current) against the IXTT2N170D2's largest
cut-off magnitude at the same temperature (4.0 V × DS100418B Fig. 12: 1.00 / 1.06 / 1.11 at 25 / 50 / 70 °C,
typical-curve readings):

| Sensor zener | 25 °C | 50 °C | 70 °C (worst) |
|---|---|---|---|
| MMSZ4687 4.3 V, as drawn, TC 0 | 4.89 V − 4.00 = 0.89 V | 4.80 − 4.24 = 0.56 V | **4.72 − 4.44 = 0.28 V** |
| MMSZ4687, TC −1 / +1 mV/K [EST bound; datasheet Fig. 1 typical ≈ 0] | 0.89 / 0.89 V | 0.53 / 0.58 V | **0.24 / 0.32 V** |
| **MMSZ4688 4.7 V (proposed)**, TC +0.8 mV/K | 1.27 V | 0.95 V | **0.69 V** |

Highest thresholds (maximum zener, aged CTR 250 %, −20 °C): 5.98 V (MMSZ4687), 6.35 V (MMSZ4688). So J-7 stands:
the drawn margin is 0.24–0.32 V, not the 0.71 V of `../rev3` §1 (that run was at 27 °C and used a 1.06 factor).
The 4.7 V zener is affordable only with a stronger hold: on the drawn 3.1 V pump its 6.35 V aged-cold threshold
would leave 0.35 V of hold margin.

## 4. Other changes tried [SIM, `out/alternatives.txt`]

| Change | Why not |
|---|---|
| A1: 5 V clock alone | releases slowly: 47 ms stopped low, 65 ms stopped high, supply removed not within 80 ms |
| A2: 5 V + 1 kΩ pull-down + 2.2 MΩ bleeds | stopped high 29.0 ms (> 25 ms) |
| A3: 5 V + 1 kΩ + R1135 / R1190 / R1191 lowered to 1 MΩ (instead of node bleeds) | locks out at 3.5 V, and at 4.0 V a 1 ms gap releases the path |
| A4: as recommended but R1133 47 Ω | locks out at 4.25 V: no amplitude margin |
| A7: as recommended but R1133 left at 10 Ω | works (holds at 4.0 V, release ≤ 17.1 ms) but edge current reaches 48–50 mA |
| A5: sheet 11 only — Schottky (BAT54S) pilot pump + 1 MΩ bleeds + 1 kΩ pull-down, 3.3 V clock | releases fast (≤ 16.5 ms) but at 70 °C a 1 ms clock gap releases the path (Schottky leakage bound 1 µA/V [EST]) |
| A6: as A5 with 2.2 MΩ bleeds | survives the gap and holds down to 2.7 V, but stopped high takes 28.5 ms (> 25 ms) in the cold |
| Bigger pump capacitors, a lower-resistance pump output, more stages, a lower-pinch-off pilot | not pursued: `../rev3` and `../charger-storage` (corrected) show they trade lock-out against release, and no P-JFET with a narrower low pinch-off range was found |

A sheet-11-only fix (A5/A6) is possible in principle but sits between two limits — survive a 1 ms gap when hot,
let go within 25 ms when the clock stops high in the cold — with almost no room; it is not recommended.

## 5. What this does not settle

- The translator part is chosen from memory of its family; its datasheet must be read (pinout, ±32 mA at 4.5 V,
  high impedance when either supply is off, Ioff). The 5V0_SYS tolerance (± 5 %) is an estimate.
- The MMBFJ270's pinch-off drift with temperature is not modelled (the full 0.5–2.0 V range is swept instead);
  BAT54 drops are at datasheet maxima; capacitor ageing is the 70 % estimate; the opto CTR corners are WP-F's.
- The clock driver is an ideal source behind a resistor; edge currents in this model stay ≤ 40 mA (sampled), but
  the translator's real output into a 6 µF pump load, and its current limit, must be checked.
- Static stack sharing in `../rev3-wpf` assumes a −5.9 V hold; with the proposed −8.2…−9.8 V the bottom device is
  held harder (no sharing figure gets worse from a more negative bottom gate, but it was not re-run).
- Everything here must be confirmed by measurement at −20 °C and +70 °C before it is relied on.

## 6. Where corrected results contradict what is now drawn

Corrected figures from this audit (`../charger-storage`, `../rev3`, `../rev3-wpf` Corrections and this folder)
against the pin tables and notes as drawn after revision 0.3. Nothing below was edited; each is for the sheet owner.

| Sheet / parts | What is drawn or written | Corrected [SIM] |
|---|---|---|
| 11, group notes `holdA` / `holdB`; U1101, U1102 descriptions | each sensor "reads held only below about −4.95..−6.12 V [SIM]" (from `../rev3` §1, run at 27 °C) | **−4.72 … −5.98 V** with the drawn 47 k + 1 M pull-ups; margin over the worst cut-off **0.28 V** (0.24–0.32 V with zener drift), not 0.71 V |
| 11, D1125, D1133, D1126, D1134 (MMSZ4687 4.3 V) | chosen because `../rev3` §1 gave a 0.71 V margin | that margin is 0.28 V; the 4.7 V part restores 0.69 V but needs a stronger hold (§2) |
| 11, R1194, R1195 (470 Ω) descriptions | "removes the cold power-up lock-out [SIM rev3 §2]" | removes it only for a clock ≥ 3.1 V behind ≤ 40 Ω at ≤ 1.65 kHz; at a true −20 °C the path still **never holds** at 2.0 kHz, at 3.0 V, or behind 60 Ω |
| 11, group notes, requirement "release ≤ 25 ms with the clock stopped LOW" | stated as met | **27.3 ms** at 70 °C, fresh sensors, 3.5 V, 1.35 kHz, 0.5 V pilot (23.2 ms at WP-F's own corners) — not met at that corner |
| 11, R1131, R1132 (100 kΩ pull-down) with sheet 09 U908, U909 (Ioff buffers on the controller rails) | loss of the controller's rail = "no clock" = path conducts | it does, but only after up to **85 ms** (78–85 ms with a 0.5 V pilot, 16–25 ms with a 2.0 V pilot): the pump input floats on 100 kΩ |
| 11, Q1112, Q1113 pilot pump (C1125–C1130, R1135, R1190, R1191) | release relies on the clock being stopped low (firmware rule) | a controller frozen with its pin **high** (fault 2, §B15: "either level") releases only after up to **52 ms** (45–52 ms with a 0.5 V pilot, 11–19 ms with a 2.0 V pilot) |
| 11, group notes, requirement "BOTH sensors held, HELD_N ≤ 1.0 V, at −20 °C, 3.1 V, 1.35–1.65 kHz, aged (met [SIM])" | met | **met** (HELD_N 0.58 V, after start-up) — but less than 0.1 V of clock amplitude above a lock-out |
| 09, group `hold_drive` note ("WP-F's requirement: −7.0 V at 1.1 mA from a 2 kHz 3.3 V clock") | U908 / U909 on 3V3_MAIN / 3V3_SUP | at 3.3 V / 35 Ω and 2 kHz the cold aged hold is −7.10 V (met); at the 3.1 V / 40 Ω corner and 2 kHz it locks out |

Design text (not a sheet; for the design authority), `docs/HARDWARE_DESIGN.md` v0.3: §B5.4 "hold level at the cold
corner −5.9 to −6.1 V" (as drawn it settles at −6.70 to −6.73 V); thresholds "4.95 / 5.62 / 6.12 V" and "0.71 V";
"chatter at 1.0 and 2.0 kHz" (at 1.0 kHz it holds; at 2.0 kHz it never holds); "gate past cut-off 1.4–22 ms" (up to
27.3 ms); "Held after power-up 19–33 ms" (up to 41.7 ms, beyond firmware's 35 ms); I-24 "3, 4 and 5 stages give
about the same" (corrected −6.55 / −7.28 / −7.50 V: the fourth stage helps); §B15 fault 2 "≤ 10 ms" and fault 10b
"0.7–1.6 s" (simulated 11–52 ms for a clock stuck high); and the static-sharing worst margin quoted from WP-F R3.0
(0.19 → 0.24, `../rev3-wpf` Correction).

## 7. Audit of every simulation folder (28 November 2026)

Checked for: temperature set and then undone; temperature not reaching the device models; behavioural models with
no temperature dependence presented as corners; Monte Carlo that does not vary its parameters or re-seeds; swept
parameters the circuit does not use; measurements that fail silently (including aborted runs and failed operating
points). Each suspicion was tested with a minimal run (probe diode, per-run parameter print-out, re-solve).

| Folder | Verdict | Detail |
|---|---|---|
| `bridge` | clean | temperature is a stated junction-temperature parameter of the behavioural SiC model; no Monte Carlo; `run.sh` retries aborted runs with another integrator and marks short runs INCOMPLETE |
| `charger-storage` | **errors, corrected** | temperature before `reset` (hold/held corners at 27 °C); hold read during start-up; zener drift ignored (`tbv1`). Stack Monte Carlo checked: parameters change every run, one seed, results identical on re-run |
| `logic-relay` | clean | `sense.cir` sets the temperature after `reset`; `det_permit.cir` uses per-diode `temp={tdeg}`, which was proven to follow `alterparam` + `reset` |
| `rev3` | **errors, corrected** | as `charger-storage` (§1, §2); §3, §4 have no temperature corners |
| `rev3-wpc` | clean | Python Monte Carlo seeded once; the latch netlist prints "−1 = no trip" for the C812-open case, which is already latched — `notes.md` reads it correctly (2,106 V) |
| `rev3-wpd`, `rev3-wpe`, `rev3-wpg`, `rev3-wph` | clean | temperature only through stated bands / factors; control-language Monte Carlo seeded once. `rev3-wph/fe_protect_wph.cir`: each transient's starting point uses ngspice's "transient op" fallback (4 of 4 runs); the state before the event is near zero and the events start later, so the results are probably unaffected, but this was not re-solved [unverified] |
| `rev3-wpf` | **errors, corrected** | temperature correct; hold read during start-up (J-2); the 1,353 V worst static fault came from a failed operating point |
| `rev3-fix` (this) | — | every result line carries a temperature proof and an abort check |

Simulator behaviours found on the way (ngspice 42), for anyone writing more netlists: (1) `reset` returns the
circuit to 27 °C — set `option temp` after it; (2) a circuit `.param` cannot be read from the control script before
an analysis (the read fails and the run continues at the old temperature); (3) the diode `TBV1` is ignored —
use `TCV`; (4) a failed operating point is silently replaced by a "transient op" whose continuation can land on a
non-physical state; (5) an included control file whose **first line** is `.control` has every `$` preceded by a
space treated as a comment (`if $x = 1` becomes `if`); every included `.ctl` in this repository starts with a
comment line and is not affected; (6) a current-source output node with no capacitance can abort a transient.
