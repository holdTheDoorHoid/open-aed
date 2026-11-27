**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# Output bridge simulations (sheets 12 and 13)

27 November 2026. ngspice 42. Scope: the output bridge (`hardware/aed-main/specs/13_hv_bridge.toml`, sheet 12's
shock-logic behaviour as described in `capture-notes/WP-D.md`). Every number below is **simulated** with the models
described in "Models"; nothing has been built or measured, and nothing here is safe or ready for use on a person
or animal. These are not build instructions.

## The short version (plain language)

The output bridge is four high-voltage switches that let the charged capacitor out through the pads, first one
way and then the other. We simulated it for four questions.

1. **A broken (shorted) bottom switch is the most serious problem we found.** When the sequence starts, the top
   switch on the same side is turned on slowly on purpose. If the bottom switch below it has failed short, the
   top switch is half-on with about 2,000 V across it, and the current is limited only by the switch itself.
   As drawn, the protection notices within about 5 µs but then waits another 20–36 µs before telling the top
   switch to turn off, and the top switch's turn-off circuit itself starts late. In the simulation the top switch
   absorbs 0.35 J (diagonal case, 1,950 V, 25 Ω patient) to 12 J (same-side case). The second figure is far beyond
   anything the part can survive; the first is uncertain because the maker publishes no heating data that fast.
   Three changes together bring it down to about 11–13 mJ and an estimated 95–100 °C, inside the part's
   150 °C rating: **turn the top switches off at once on a current fault, add a faster current detector, and
   empty the top switch's gate supply as soon as its drive signal stops.**
2. **Switching off a short circuit** happens at 107–112 A as drawn, just above the switch's 107 A pulse rating,
   and the voltage on the switch rises to 2,750 V at the top capacitor voltage with a short cable, or **above the
   3,300 V rating with a 0.5–1 µH cable** to the off-board capacitor. Lowering the trip to 80 A and adding a
   local film capacitor next to the bridge keeps the current at or below about 97 A and the voltage at about
   2,430–2,510 V. Making the existing snubber bigger makes things worse.
3. **Normal shocks** come out as designed: 149–154 J adult and 50–54 J child, peak currents as the design table
   says, switch heating modest (at most about 98 °C). **If the firmware fails** and only the 25 ms limit ends the
   pulse, the patient can receive up to 180 J (189 J at 2,000 V) instead of 150 J. No single time limit can fix
   this, because a 175 Ω patient legitimately needs a long pulse while a 50–75 Ω patient gets too much energy in
   the same time. Separate per-phase time limits cut the worst case to about 170 J; holding it near 150 J needs
   the device to measure the delivered energy independently of the main firmware.
4. **When the bridge's supply collapses** during a shock, the bottom switch always turns off first and no
   combination we tried turns two switches of one side on together. One weakness: if only the small supply for
   the bottom switches fails, nothing on sheet 12 notices, and the bottom switch's gate sags slowly so it runs
   partly-on for up to about 0.6 ms before its driver gives up. Adding a monitor on that supply closes the gap.

The recommended changes are listed under "Recommended circuit changes" with parts and values.

## How to rerun

`bash run.sh` (optionally `JOBS=8`, or `ONLY=<pattern>` for a subset). Each netlist also runs alone with its
default values (`ngspice -b q1_shorted_lowside.cir`). Results: `out/q1_shorted_lowside.txt`,
`out/q2_oc_turnoff.txt`, `out/q3_shock_energy.txt`, `out/q4_supply_collapse.txt` (one `RESULT` line per case;
`INCOMPLETE`/`FAILED` would mark a run the solver did not finish — the final run had none). The full set is
290 cases and takes about 25 minutes on 6 cores. Shared circuit: `bridge_common.inc`.

## Models, provenance and confidence

| Part | Model used | Source | Confidence |
|---|---|---|---|
| Q1301–Q1304 MSC080SMA330B4N | **behavioural electro-thermal model built here** (`models/msc080sma330b4n_beh.lib`) | Microchip's SPICE library refused automated download (HTTP 403, 27 Nov 2026). Built from datasheet DS00005914A (RS copy): square-law fit to Fig 1-2 (25 °C) and Fig 1-4 (150 °C), K = 1.03 A/V², Vt 4.06 V → 2.77 V; RDS(on) 106 mΩ max with the Fig 1-5 temperature factor; Ciss/Crss/QGD/Coss from Table 1-4 and Fig 1-7/1-8; body diode from Table 1-5; Foster thermal network fitted to the Fig 1-18 single-pulse Zθ (as read in WP-G §4) | Channel at VDS ≤ 35 V: good (checked: 3.9/16/37/66/104 A at 6/8/10/12/14 V vs datasheet ≈ 3/16/38/65/100 A). Saturation current at kilovolts: **not published**; the model gives ≈ 230 A at VGS 20 V, a sensitivity run with mobility degradation (`th = 0.06`, ≈ 170 A) is included. **No avalanche, short-circuit, turn-off-SOA or failure model exists** — the datasheet publishes none |
| Thermal below 10 µs | Zθ extrapolated as √t below the last printed point (0.0051 °C/W at 1 µs) | WP-G's reading of Fig 1-18 | **Low.** See "Junction temperature: two estimates" below |
| D1301, D1302 GC05MPS33J | behavioural diode built here (`models/gc05mps33j_beh.lib`) | Maker's SPICE zip returned HTTP 500. Datasheet Rev 22/Apr: VBI 0.97 V, RDIFF 0.24 Ω (Fig 10), C 337 pF @ 1 V / 14 pF @ 1.7 kV | Medium; the linear RDIFF overstates the drop at > 50 A |
| U1301, U1302 UCC5350MC | behavioural driver in `bridge_common.inc` (65/100 ns delay, 2.5 Ω, UVLO 10.65/12.5 V, 220 Ω on, 33 Ω + 1N4148W off) | datasheet figures from WP-G §6.5. TI's model (SLLM367) **was** downloaded and runs in ngspice (checked: 52 ns delay, clean ±20/−5 V) but its licence reads "All rights reserved" with no redistribution grant, so it is kept out of the repository (listed in this directory's `.gitignore`) and not used by the netlists | Medium-high |
| High-side drive (T1301/T1302, rectifier, PNP) | behavioural: carrier = 18.5 V × BRIDGE_5V/5 V behind 20 Ω, RECT 1 nF ∥ 2.2 kΩ, 1N4148W + 100 Ω, 2 × 47 nF, PNP as I = (Vg − Vrect − 0.65)/34 Ω | WP-G §6.2 (transformer is made to order; its 20 Ω output resistance is an estimate) | Medium |
| Sheet 12 timing | behavioural: idle path INA240 (τ 0.4 µs) + 1 k/100 pF + 5 A, OC path 100 Ω/1 nF + 89.7 A, latch + delays (`tdl`, RUN_D20 20.2–36.4 µs), rail-collapse 50 k/100 pF + 3 µs at 4.5 V | WP-D §3.5, §3.6, §4; WP-G §8 | Medium |
| Wiring | stray `lst` (half in the bus, half in the return), cable to the off-board capacitor `lcap`, patient leads 1.5 µH split, relay contacts 2 × 0.05 Ω, capacitor ESR 0.13 Ω, L_S winding 0.08 Ω | design §B6.5 [EST]; strays swept | Parameters, swept |

Starting junction temperature 55 °C everywhere (WP-G assumption). The solver occasionally stops on these stiff
circuits ("timestep too small"); `run.sh` then retries with other integration settings and the RESULT line says
which (`[solver: …]`). A few early runs that stopped part-way silently reused old numbers; they were found by a
completion check now built into every netlist (`t_end … of …`), and all reported numbers come from complete runs.

### Junction temperature: two estimates

The datasheet's thermal curve stops at about 7 µs. Extrapolating it as √t (what the netlists do) implies the heat
is spread over an effective area of only ≈ 0.08 cm² — pessimistic. A physical lower estimate treats the die as
heating adiabatically in the top ≈ 30 µm plus the heat-diffusion depth (≈ 40 µm in 10 µs in SiC), over an
**assumed** 0.2 cm² active area [ASSUMPTION — die size not published]: about **1.3 mJ/K for 0.2 µs events and
3–4 mJ/K for 5–30 µs events**. The two estimates differ by 2–6×. Where they disagree about crossing 150 °C, the
answer is "unknown until Microchip supplies Zθ below 10 µs or a pulse measurement is made". Tables give the
datasheet-extrapolated value (upper estimate); the text adds the lower estimate where it matters. **Values above
about 200 °C are outside every published figure and the model's validity (its threshold voltage keeps falling
with temperature, so it runs away): read them as "destroyed".**

---

## Question 1 — one low-side switch shorted (I-8, WP-G F-1, WP-D CI-10)

**Netlist** `q1_shorted_lowside.cir`. QHA is commanded on at the start of a sequence (its deliberately slow
turn-on, ≈ 22 µs to 15 V). Two cases:

- **D (diagonal, design fault 18):** QLB shorted (0.05 Ω). Current flows QHA → patient → QLB.
- **S (same side, shoot-through):** QLA shorted. Current flows QHA → QLA straight across the capacitor, limited
  only by R_S, L_S and QHA. **This case is not in the WP-G failure table** (the Q1302 row only discusses QHB);
  it occurs at every phase-1 start if QLA has failed short since the last self-test.

What happens as drawn: QHA's gate rises at ≈ 1.6 V/µs; QHA starts conducting ≈ 4 µs after the carrier starts,
the idle comparator (via the INA240) sets the fault latch at **≈ 4.9–5.2 µs**, but sheet 12 drops `G_HA` only
after RUN_D20 (**20.2–36.4 µs**). Meanwhile QHA turns fully on. In case D the current then becomes a normal
phase-1 current through the patient; in case S it rises at V/L_S until QHA saturates (**≈ 240 A**, far above
IDM 107 A) with ≈ 2 kV across it. A second, previously unnoticed delay: when the carrier stops, the PNP cannot
start emptying the gate until `HSA_RECT` (1 nF ∥ 2.2 kΩ, τ 2.2 µs) has fallen below the gate voltage. With the
gate only part-way up (5–6 V) that takes ≈ 2.7 µs, during which the gate keeps **rising**. (WP-G §6.2's 0.16–0.3 µs
figure assumed a fully charged gate.)

**Results (selected; full set in `out/q1_shorted_lowside.txt`). Energy in QHA and peak junction temperature
(datasheet-extrapolated).** `tdl` = extra detection/logic delay swept 0.1–2 µs; stray inductance 20–200 nH changed
nothing visible (L_S dominates).

| Arrangement | D, 1,950 V, 25 Ω | D, 2,200 V, 25 Ω | D, 1,950 / 2,200 V, 175 Ω | S, 1,950 V | S, 2,200 V |
|---|---|---|---|---|---|
| **As drawn** (any `tdl`, RUN_D20 20.2–36.4 µs) | 0.35 J, 474–489 °C, 65.7 A | 1.9–2.4 J, > 1,000 °C | 17 / 23 mJ, 84 / 94 °C | 11 J, 241 A — destroyed | 12.5 J, 246 A — destroyed |
| 1: drop `G_H*` at once on OC/idle fault | 0.13–0.34 J (tdl 0.1–2 µs), 297–521 °C | 0.22–2.9 J | 19–34 mJ, 94–123 °C | 11 J (runaway) | 12.4 J (runaway) |
| 1 + 2: plus a direct shunt comparator, 3 A, 0.1 µs | 45 mJ, 157 °C | 62 mJ, 192 °C | — | 49 mJ, 169 °C | 80 mJ, 231 °C |
| 1 + 2 at 2 A | 26 mJ, 119 °C | 33 mJ, 135 °C | — | 26 mJ, 118 °C | 36 mJ, 142 °C |
| 1 + 3: immediate off + fast RECT discharge (INA240 path kept, tdl 0.1 µs) | 32 mJ, 149 °C | 41 mJ, 171 °C | — | 36 mJ, 162 °C | 46 mJ, 191 °C |
| **1 + 2 + 3, 3 A** | **10.9 mJ, 93 °C, 4.9 A** | **12.3 mJ, 99 °C** | **7.5 / 9.5 mJ, 80 / 87 °C** | **10.9 mJ, 94 °C** | **12.3 mJ, 100 °C** |
| 1 + 2 + 3, 5 A | 19.6 mJ, 118 °C | 25 mJ, 134 °C | — | 21 mJ, 124 °C | 25 mJ, 137 °C |

Sensitivity: with the lower saturation current (`th = 0.06`) the 1 + 2 + 3 results move by < 3 %. Halving the
added gate capacitance (2 × 47 nF → 47 nF) makes things worse (51–148 mJ) because QHA turns on faster than the
detector can follow, and it halves the dv/dt immunity margin of WP-G §5 — not recommended.

**Lower estimate** (adiabatic, see above): as drawn D at 1,950 V, 0.35 J → +80–115 K (≈ 135–170 °C), i.e. the
as-drawn diagonal case at nominal voltage may or may not survive; at 2,200 V (2.4 J) and in case S (11–12 J) the
switch is destroyed under either estimate. With changes 1 + 2 + 3 both estimates stay below 150 °C.

**Answer.** As drawn, a shorted low side destroys the opposite high side in the same-side case (≈ 11–12 J,
≈ 240 A) and in the diagonal case at 2,200 V; at 1,950 V / 25 Ω the diagonal case puts ≈ 0.35 J into QHA with
an uncertain outcome. The cause is not detector speed alone: the 20–36 µs high-side delay (CI-10) dominates, and
the slow `HSA_RECT` collapse adds ≈ 2–3 µs. **No series element helps**: the existing R_S/L_S limit di/dt to
≈ 73 A/µs but the half-on switch sets the current far below that within the 4–8 µs event, and a resistor big
enough to limit a 2 kV transit to a few amperes (hundreds of ohms) would also stop the shock. The surviving
arrangement is changes 1 + 2 + 3 together (≈ 11–13 mJ, 93–100 °C upper estimate).

Note on risk: in case D a destroyed QHA that fails short completes the R-14 cascade (a monophasic discharge of up
to ≈ 160–178 J through the patient, design §B13.6). Case S does not pass current through the patient but dumps the
capacitor into R_S (WP-G expects R_S may crack).

## Question 2 — turn-off at the over-current trip (WP-G F-2)

**Netlist** `q2_oc_turnoff.cir`. Phase 1 starts into a short at the pads (0.01 Ω, with 1.5 µH of lead) or at the
board connector (no lead inductance), capacitor at **2,200 V** (worst case). The OC comparator trips, sheet 12
drops `G_LB` (≤ 65 ns), the UCC5350 adds 65–100 ns, QLB turns off through 33 Ω; the coil current moves to
D1302/D1301.

| Case (2,200 V) | Current at turn-off | Peak QLB VDS | Turn-off energy | Tj (upper est.) |
|---|---|---|---|---|
| As drawn, trip 82 / 89.7 / 98 A (band ends), lead 0–1.5 µH, driver 65–100 ns, lst 50 nH, cable 200 nH | **92–98 / 99–104 / 107–112 A** | 2,670–2,750 V | 15–18 mJ | 161–197 °C |
| As drawn, stray lst 20 → 200 nH (trip 98 A) | 109–112 A | 2,720 → 2,890 V | 18–19 mJ | 195–207 °C |
| As drawn, **cable to the capacitor 50 nH / 200 nH / 500 nH / 1 µH** | 109 A | **2,516 / 2,750 / 3,206 / 3,720 V** | 18–19 mJ | 195–207 °C |
| Bigger RC snubber 10 nF with 10 / 3.3 / 1 Ω (lst 200 nH, cable 500 nH) | 110 A | 2,590 / 2,830 / 2,980 V | 27–29 mJ | 196–250 °C |
| Bigger RC snubber 47 nF with 10 / 3.3 / 1 Ω | **112 / 162 / 211 A peak at turn-on** (snubber discharges through the switches) | 2,220–2,320 V | 50–89 mJ | 330–720 °C |
| **Local film capacitor HV_CAP_P–HV_RTN** 100 nF / 470 nF (lst 200 nH, cable 500 nH) | 110 A | **2,497 / 2,511 V** | 18 mJ | 197 °C |
| RCD clamp on HV_BUS 100 / 470 nF | 110 A | 2,477 / 2,473 V | 18 mJ | 197 °C |
| Local film 470 nF + RCD 470 nF, cable 1 µH | 110 A | 2,422 V | 18 mJ | 195 °C |
| **Trip lowered to 80 A nominal** (band 74 / 80 / 86 A) | **86 / 91 / 97 A** | 2,650–2,700 V | 14–16 mJ | 151–172 °C |
| Trip 86 A (top of the 80 A band) + local film 470 nF + RCD 470 nF, lst 200 nH, cable 500 nH | 99 A | **2,430 V** | 16 mJ | 173 °C |

The spike is set mainly by the **cable to the off-board capacitor**: when QLB opens, the current in that cable must
stop, and the only nearby path back is the 3 nF / 10 Ω snubber, whose 10 Ω alone drops ≈ 1 kV at 100 A. Board
stray inductance (20–200 nH) adds only ≈ 150 V. The turn-off energy (15–19 mJ, as WP-G estimated) is small; the
temperatures in the table are the pessimistic extrapolation — the lower estimate for a 0.2 µs, 18 mJ event is
≈ +14 K. The open question is not heat but whether the part may interrupt > 107 A at 2.4–2.7 kV at all:
**Microchip publishes no turn-off (RBSOA) or short-circuit rating**.

**Answer.** As drawn the switch interrupts **92–112 A** (vs IDM 107 A) and sees **2.67–2.89 kV**, rising to
**3.2–3.7 kV — over the 3.3 kV rating — if the cable to the capacitor is 0.5–1 µH**. Recommended: trip at 80 A
nominal (interrupted current ≤ 97 A) and a local film capacitor (≥ 100 nF, 470 nF preferred) at HV_CAP_P; an RCD
clamp adds little once the film capacitor is there. Do not enlarge the RC snubber. Patient-lead inductance changed
the results by < 2 %.

## Question 3 — normal shock and the 25 ms time cap (I-13)

**Netlist** `q3_shock_energy.cir`. DESIGN_TABLE timings; full bridge with all parasitics above.

**3a Normal biphasic shock (simulated)**

| Mode, load | Phases ms | Delivered J | Peak A ph1 / ph2 | Switch energy mJ QHA / QLB / QHB / QLA | Tj max °C (QHA/QLB) | R_S J |
|---|---|---|---|---|---|---|
| Adult 1,950 V, 25 Ω | 3.0 / 2.0 | **154.4** | 65.7 / 23.8 | 858 / 823 / 82 / 79 | 98 / 96 | 24.1 |
| Adult, 50 Ω | 3.34 / 2.23 | 150.7 | 35.6 / 19.3 | 336 / 324 / 74 / 73 | 69 / 83 | 11.8 |
| Adult, 75 Ω | 4.42 / 2.95 | 150.1 | 25.1 / 14.5 | 213 / 206 / 54 / 53 | 62 / 78 | 7.8 |
| Adult, 125 Ω | 6.70 / 4.47 | 149.7 | 15.5 / 9.3 | 124 / 119 / 33 / 33 | 67 / 72 | 4.7 |
| Adult, 175 Ω | 9.01 / 6.01 | 149.4 | 11.1 / 6.7 | 87 / 84 / 24 / 24 | 61 / 71 | 3.3 |
| Child 1,150 V, 25 Ω | 3.0 / 2.0 | 53.8 | 38.8 / 14.0 | 264 / 254 / 28 / 28 | 68 / 68 | 8.4 |
| Child, 50 Ω | 3.0 / 2.0 | 50.6 | 21.0 / 12.1 | 106 / 102 / 27 / 26 | 60 / 65 | 3.9 |
| Child, 75 Ω | 3.94 / 2.63 | 50.1 | 14.8 / 9.0 | 68 / 66 / 19 / 19 | 61 / 63 | 2.6 |
| Child, 125 Ω | 6.02 / 4.01 | 49.9 | 9.2 / 5.8 | 40 / 39 / 12 / 12 | 58 / 62 | 1.6 |
| Child, 175 Ω | 8.12 / 5.41 | 49.8 | 6.6 / 4.2 | 29 / 27 / 9 / 8 | 58 / 61 | 1.1 |

Agreement with the design documents: delivered energy within 0.5 % of DESIGN_TABLE (its 5 Ω internal resistance
vs ≈ 4.8 Ω here); 25 Ω peak 65.7 A, R_S 24.1 J and QHA ≈ 98 °C match WP-G §4/§7 (102 °C there, max-RDS basis).
The QLB temperature at high resistance (71–83 °C) comes from its hard turn-off at the end of phase 1, not from
conduction. No turn-off overshoot above the bus (VDS ≤ 1,949 V) at normal phase ends.

**3b Firmware failed; only a time limit ends the pulse (delivered J; adult)**

Scenario P1: the phase-1 command sticks, phase 1 runs to the limit, no phase 2. Scenario P2: phase 1 as planned,
phase 2 sticks until the limit.

| Limit | V0 | 25 Ω | 50 Ω | 75 Ω | 125 Ω | 175 Ω |
|---|---|---|---|---|---|---|
| Planned shock (3a) | 1,950 | 154.4 | 150.7 | 150.1 | 149.7 | 149.4 |
| As drawn: 25 ms cap, low end 21.3 ms (P1) | 1,950 | 159.8 | 173.0 | 177.2 | 175.3 | 166.8 |
| As drawn: 25 ms cap, high end 28.8 ms (P1 / P2) | 1,950 | 159.8 / 159.8 | 173.1 / 173.1 | 177.9 / 177.8 | **180.0** / 179.8 | 176.5 / 176.0 |
| Option A: per-phase caps, ±15 % RC (P1 ≤ 16.2 ms / P2 ≤ 10.8 ms) | 1,950 | 159.8 / 159.8 | 172.6 / 172.1 | 175.0 / 174.1 | 167.2 / 169.9 | 153.7 / 163.7 |
| Option B: per-phase caps, ±5 % (P1 ≤ 13.3 / P2 ≤ 8.8 ms) | 1,950 | 159.8 / 159.7 | 171.8 / 171.1 | 171.7 / 171.6 | 158.8 / 165.5 | 142.1 / 158.6 |
| Option C: as B with config limits lowered to 10 / 6.7 ms (P1 ≤ 11.1 / P2 ≤ 7.4 ms) | 1,950 | 159.7 / 159.7 | 170.2 / 169.7 | 167.1 / 168.9 | 149.3 / 161.5 | 130.5 / 154.4 |
| As drawn, 28.8 ms (P1) | 2,000 | 168.1 | 182.1 | 187.1 | **189.3** | 185.6 |
| Option C (P1 / P2) | 2,000 | 168.0 / 167.9 | 179.0 / 178.6 | 175.8 / 177.7 | 157.1 / 169.9 | 137.3 / 162.4 |

**Answer.** Worst case as drawn **≈ 180 J at 1,950 V (125 Ω) and ≈ 189 J at 2,000 V**, confirming I-13
(the design note's 177 J at 175 Ω is right; the true worst load is ≈ 100–125 Ω). **No single cap value can hold
≈ 150 J:** a 50 Ω patient reaches 150 J in ≈ 5.5 ms of monophasic pulse, while a 175 Ω patient legitimately needs
9 ms of phase 1. Per-phase caps (option C) cut the worst case to ≈ 170 J at 1,950 V and ≈ 179 J at 2,000 V, mostly
helping above 100 Ω; below 25 Ω the capacitor's own energy (160 J at 25 Ω) is the limit anyway. To hold the worst
case near 150 J the device needs a **delivered-energy limit that does not depend on the main firmware**: measure
patient voltage (OUT_A_TAP − OUT_B_TAP, already on sheet 13) × I_SHOCK, integrate, and set the sheet-12 fault
latch at ≈ 160 J. That could be an analogue multiplier/integrator or the supervisor controller (sheet 05) with a
new stop line into the latch; it was **not simulated in circuit form** (an ideal limiter trivially gives 160 J) and
it is a contract change. Note that a cap on capacitor voltage drop or on charge cannot do it either: at 25 Ω the
planned shock removes more energy and charge (internal losses 31 J) than a 175 Ω fault needs to deliver 180 J.

## Question 4 — supply collapse during a pulse

**Netlist** `q4_supply_collapse.cir`. Event 1 ms into phase 1 (QHA + QLB conducting), 25 Ω and 175 Ω.

| Event | Fault latch | QLB starts blocking | Current stops | Linear-region time QLB | Energy QLB (from event) | Tj QLB (upper est.) | Forward current in the "off" switches QLA/QHB |
|---|---|---|---|---|---|---|---|
| BRIDGE_5V to 0 in 1 µs, 25 Ω | 1.0 µs | 5.1 µs | 5.2 µs | 0.09 µs | 3.9 mJ | 132 °C | ≤ 0.6 A (capacitive) |
| BRIDGE_5V to 0 in 100 µs, 25 Ω | 14 µs | 18 µs | 18 µs | 0.11 µs | 8.3 mJ (incl. conduction) | 133 °C | ≤ 0.7 A |
| BRIDGE_5V sags over 2 ms, 25 Ω | 188 µs | 193 µs | 193 µs | 0.15 µs | 64 mJ (mostly conduction) | 133 °C | ≤ 1.0 A |
| BRIDGE_3V3 (logic) lost, 25 Ω | — (driver input dies) | 1.1 µs | 1.1 µs | 0.10 µs | 2.6 mJ | 131 °C | ≤ 0.5 A |
| **MGJ2 output alone collapses in 100 µs, 25 Ω** | **none** | 57 µs | 57 µs | 0.73 µs | 30 mJ | 158 °C | ≤ 0.5 A |
| **MGJ2 output alone decays over 1 ms, 25 Ω** | **none** | 568 µs | 568 µs | 2.0 µs (plus ≈ 0.5 ms partly-on) | 232 mJ | **181 °C** | ≤ 0.4 A |
| Same events, 175 Ω | as above | 2–5,700 µs | — | ≤ 0.12 µs | 0.3–63 mJ | ≤ 62 °C | ≤ 0.3 A |

In every case **the low side turns off first**; QHA never had to block while current flowed (its gate follows the
carrier down later, as WP-G §6.2 predicted). The margin WP-G asked about is real and comfortable: the rail-collapse
detector plus the UCC5350 turn QLB off ≈ 4 µs after the latch, while the high-side PNP needs ≈ 3 µs more just to
start (see Q1). **No combination produced shoot-through**: the off switches' gates never rose above −4.2 V
(low side, Miller clamp path) or +0.03 V (high side), and their currents stayed below 1 A (capacitive).

The one gap: **the MGJ2 (low-side ±supply) failing on its own is not detected**. The UCC5350MC keeps driving until
its UVLO (10.3–11 V), so the low-side gate sags from 20 V through 9–12 V while carrying up to 50 A; at 25 Ω the
switch spends ≈ 0.5 ms increasingly resistive and the extrapolated junction temperature reaches 158–181 °C (the
millisecond part of this estimate uses the published Zθ and is credible). At 175 Ω it is harmless.

**Answer.** Turn-off order is right, no shoot-through, no long linear dwell after a BRIDGE_5V or logic collapse.
Add a low-side supply monitor (below) to cover the MGJ2-only failure.

---

## Recommended circuit changes

Numbered for reference; none has been reviewed by a qualified engineer.

1. **Sheet 12 — turn the high sides off at once on an over-current or idle-current fault** (WP-D CI-10). Keep the
   RUN_D20 delay (20.2–36.4 µs) only for planned ends, cap expiry and rail collapse. Implementation: a fault-source
   signal "OC or idle latched" (active low) as an additional input to the AND gates that form `G_HA_I`/`G_HB_I`
   (U1222, U1223; one more gate input each, or one added SN74LVC1G08 per side). The coil current is unaffected
   (it freewheels through D1302/D1301 whatever the high side does), and the low side, being faster, still opens
   first in a short.
2. **Sheet 13 — a dedicated fast idle-current comparator directly on the shunt Kelvin pair** (SHUNT_KP/SHUNT_KN),
   threshold **3 A (6.0 mV)**, total delay to the sheet-12 latch ≤ 0.1 µs, feeding `IDLE_I_DET_N` in place of (or
   OR-ed with) the INA240-derived path. The TLV3501's ±6.5 mV offset is too large for 6 mV: it needs either a
   comparator with ≤ ±1 mV offset at ≤ 50 ns, or a ≥ 10 MHz gain-of-10 preamplifier ahead of a TLV3501 (part to be
   chosen — not specified here). 2 A was better (26–36 mJ without change 3) but leaves less margin to noise and to
   the 0.29 A self-test current.
3. **Sheet 13, both high-side islands — empty `HSA_RECT`/`HSB_RECT` within ≈ 0.1–0.3 µs once the carrier stops**
   (simulated as 100 Ω across C1315/C1317 that switches in when the carrier stops). This needs a small
   carrier-loss detector on each floating island: it must ignore the 0.19 µs dead time at every half-cycle but act
   within ≈ 0.5 µs of a real stop (e.g. an N-MOSFET across C1315 whose gate is held off by a separately rectified,
   short-time-constant copy of the secondary). Circuit not designed here — design work for WP-G.
   Changes 1 + 2 + 3 together give 11–13 mJ / ≈ 93–100 °C in all shorted-low-side cases simulated.
4. **Sheet 13 — lower the over-current trip to 80 A nominal**: R1351 **174 kΩ → 196 kΩ** (1 %): 3.3 V × 10.0/206 =
   160.2 mV = **80.1 A** on the 2 mΩ shunt (band ≈ 74–86 A with WP-G's tolerances). Interrupted current then ≤ 97 A
   (< IDM 107 A). The highest planned peak is 67.4 A (25 Ω at 2,000 V), so the band's low end leaves ≈ 6.6 A of
   margin; at the OVP top (2,182 V, 25 Ω, ≈ 73.5 A) it could trip — acceptable, that charge is itself a fault.
   (Proposed in WP-G F-2; this confirms it.)
5. **Sheet 13 — add a local film capacitor from HV_CAP_P to HV_RTN** close to D1301's cathode and R1301:
   **two 1 µF, ≥ 2.2 kV DC (≥ 3 kV preferred) metallised-film capacitors in series (≈ 0.5 µF; 470 nF simulated), each with a
   ≈ 10 MΩ balancing resistor** — two in series so that one shorted capacitor does not short the main capacitor
   directly (same reasoning as C1301/C1302). This keeps the interrupted-short VDS at ≈ 2.5 kV with up to 0.5 µH of
   cable (it was 3.2 kV, over the rating at 1 µH). Part to be chosen.
6. **Keep the RC snubber (C1301/C1302 + R1304) as drawn.** Larger capacitance or smaller resistance raised the
   spike or discharged through the switches at every phase start (47 nF: 112–211 A pulses). An RCD clamp on
   HV_BUS (GC05MPS33J + ≥ 100 nF to HV_RTN, bled to HV_CAP_P) is an optional extra (≈ −20 to −90 V more).
7. **Sheet 12 — per-phase hardware time limits** in addition to the 25 ms cap: phase 1 ≤ 12.6 ms and phase 2
   ≤ 8.4 ms nominal with ±5 % timers (option B), or 10.5 / 7.05 ms if the firmware's maximum phases are lowered to
   10 / 6.7 ms (option C; the DESIGN_TABLE's longest are 9.0 / 6.0 ms). Worst case falls from 180 to ≈ 170 J.
8. **Independent delivered-energy limit** (contract change, owner decision): patient voltage × current,
   integrated, tripping the sheet-12 latch at ≈ 160 J, implemented in hardware or by the supervisor controller.
   This is the only arrangement found that holds the firmware-failure worst case near 150 J.
9. **Sheet 13/12 — monitor the low-side supply**: a comparator on the MGJ2 +20 V rail (relative to LS_SRC, which
   sits within ≈ 1 V of GND) tripping at ≈ 17 V, push-pull output into the sheet-12 fault latch beside `BR5_OK`
   (new inter-sheet net, e.g. `LSD_OK`, default low = fault). With the existing rail-collapse path this turns the
   low side off within ≈ 5 µs of an MGJ2 failure instead of letting it sag for up to ≈ 0.6 ms.

## Things that contradict or correct the existing documents

- **WP-G §12, row Q1302 (QLA shorted)** misses the same-side shoot-through at the start of phase 1 (QHA turning on
  above a shorted QLA). It is the worst case found (≈ 240 A, 11–12 J in QHA as drawn).
- **WP-G F-1** modelled the idle latch turning QHA off at once. As drawn (WP-D §5, CI-10) the high side stays
  commanded on for 20–36 µs, so QHA turns fully on; F-1's 25–122 mJ understates the drawn circuit (0.35–2.4 J).
- **WP-G §6.2** "G_H* low → HSA_RECT collapses enough for the PNP in 0.16–0.3 µs" holds for a fully charged
  gate only. For a gate part-way up (the fault case), the PNP cannot start for ≈ 2.7 µs and the gate keeps rising.
- **00-integration I-13** is confirmed; the worst load is ≈ 100–125 Ω (180 J), slightly worse than the 175 Ω
  figure in the design document (177 J); at the 2,000 V top of the allowed range it is ≈ 189 J.
- **WP-G §17 doubt 2 (supply-collapse order)**: confirmed correct with margin (low side blocks ≈ 5 µs after an
  instantaneous collapse; high side would need ≥ 8 µs). A gap remains for the MGJ2-only failure (change 9).

## What simulation cannot settle

- Whether the MSC080SMA330B4N survives ≈ 100–110 A turn-off at 2.4–2.7 kV, and how much µs-scale energy it can
  absorb in its linear region: **no RBSOA, short-circuit or avalanche rating, and no Zθ below ≈ 7 µs, are
  published**. Needs Microchip's data or low-voltage-scaled pulse measurements by a qualified lab.
- The saturation current of the switch at kilovolts (sets the same-side shoot-through current).
- The made-to-order gate transformers' real output resistance and the carrier-loss detector of change 3.
- Real stray and cable inductances, the X7R snubber capacitors' capacitance at 2 kV, and the freewheel diodes'
  repetitive surge behaviour (their linear model overstates the drop at > 50 A).
- The patient: a fixed resistor stands in for a person, whose resistance changes during a shock.
