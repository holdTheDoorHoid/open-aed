# Capture notes — WP-F storage (sheet `11_hv_storage`)

> **NOT FOR CONSTRUCTION — unreviewed design.** HIGH VOLTAGE (up to 2,200 V). This page describes a circuit
> diagram for review. It has not been built, simulated in SPICE or measured, and it has not been reviewed by a
> qualified high-voltage or medical-device engineer. Nothing here tells anyone to build, power, probe or test
> anything (SAFETY.md §2a, CLAUDE.md rule 2), and nothing here is safe or ready for use on a person or animal.

Captured 27 November 2026 against contract issue 2 and `docs/HARDWARE_DESIGN.md` v0.2. Pin table:
`hardware/aed-main/specs/11_hv_storage.toml`; custom symbols: `hardware/lib/src/aed_storage.toml`.
Labels used below: **[DS]** read from the maker's datasheet (named in the part list), **[CALC]** my arithmetic,
**[SIM]** my own *behavioural* time-step model (idealised diodes and switches, written for this capture; **not
SPICE, not measured**), **[EST]** estimate, **[UNVERIFIED]** not confirmed.

## 1. What is on this sheet, in plain language

This page is the part of the device that holds the energy and guarantees it can always be let out. The big
capacitor itself sits off the board on two terminals (J1101). Across it are, permanently, a slow "bleeder"
resistor chain and two separate measuring chains (one read by each controller). Also across it are two
**discharge paths**, A and B. Each path is three power resistors in series with a stack of four special
transistors that **conduct unless they are actively held off** — so with no power at all, the capacitor empties
through them. Path A is held off only by the main controller's clock signal, path B only by the supervisor's.
Each clock drives a small "charge pump" that makes a negative voltage to hold its stack off; when the clock
stops, a release circuit dumps that voltage quickly and the stack conducts. A light-coupled sensor (an
optocoupler) on each stack tells the interlock "held" **only** while the hold voltage is really present — a dead,
missing or unplugged sensor reads "not held", which blocks charging. A second optocoupler per stack lets the
supervisor's hardware over-voltage comparator force both stacks on regardless of what either controller is doing.
The capacitor's negative side (`HV_RTN`) joins the electronics ground at one point through two separate copper
bonds.

## 2. Part list

182 parts, 115 nets (395 pin connections), all on this sheet. Datasheets were read on 27 Nov 2026. Littelfuse,
Infineon, Vishay, Toshiba, Ohmite and Nexperia refuse automated downloads; I read their own PDFs through the
built-in browser on the maker's site (text extracted in the page), except where noted.

| Refs (A / B) | Qty | Part | Maker, MPN | Datasheet (read 27 Nov 2026) |
|---|---:|---|---|---|
| J1101 | 1 | capacitor terminals, 2 pins | **TBD** (depends on the made-to-order capacitor's terminations) | PARTS_STUDY Appendix A |
| J1102 | 1 | case-NTC connector, 4 pins | JST B4B-PH-K-S(LF)(SN) | jst-mfg.com ePH.pdf |
| NT1101, NT1102 | 2 | net ties HV_RTN–GND | copper | — |
| Q1101–Q1104 / Q1105–Q1108 | 8 | depletion MOSFET 1,000 V | Littelfuse/IXYS **IXTT10N100D2** | littelfuse.com assetdocs (DS100326A(4/17), "Preliminary") — Littelfuse's own site via browser |
| R1101–R1103 / R1104–R1106 | 6 | dump resistor 1.5 kΩ 10 % ceramic composition | Ohmite **EY152KE** | Ohmite catalogue page (ohmite.com/catalog/ey-series); the linked PDF on ohmite.com returns an HTML page, so the series sheet was read from the DirectIndustry copy of Ohmite's catalogue |
| R1107–R1110 / R1111–R1114 | 8 | sharing string 10 MΩ 1 % 2512 | Stackpole **HVCR2512FTD10M0** | seielect.com sei-hvcr.pdf (rev 9/22/2025) |
| C1101–C1104 / C1105–C1108 | 8 | speed-up 3.3 nF 2 kV C0G 2220 | KEMET **C2220C332JGGACTU** | KEMET C0G HV SMD (C1009, 12/16/2024) |
| R1115–R1117 / R1118–R1120 | 6 | upper-gate 100 kΩ 2512 HV | Stackpole **RVC2512FT100K** | sei-rvc.pdf |
| R1121–R1123 / R1124–R1126 | 6 | upper gate-source 10 MΩ 0603 | Yageo RC0603FR-0710ML | Yageo RC series |
| D1101–D1104 / D1105–D1108 | 8 | gate TVS 15 V bidirectional | Littelfuse **SMF15CA** | littelfuse.com SMF series (Revised 11/02/23), via browser |
| C1111–C1114, C1115–C1117 / C1118–C1121, C1122–C1124 | 14 | pump and hold 2.2 µF 25 V X7R 0805 | KEMET C0805C225K3RACTU | KEMET X7R SMD **[UNVERIFIED that this exact code is stocked]** |
| D1109–D1112 / D1113–D1116 | 8 | pump Schottky pair | Diodes Inc. **BAT54S-7-F** | diodes.com DS11005 Rev 35-2 |
| C1109 / C1110 | 2 | reservoir 3.3 µF 25 V X7R 1206 | KEMET C1206C335K3RACTU | KEMET X7R **[UNVERIFIED code]** |
| R1127+R1128 / R1129+R1130 | 4 | gate-source 680 k + 330 k (1.01 MΩ, 1:3 test tap) | Yageo RC0603 | Yageo |
| R1131 / R1132 | 2 | `HOLD_*_DRV` default 100 k to GND | Yageo RC0603 | Yageo |
| R1133 / R1134 | 2 | pump clock series 10 Ω | Yageo RC0603 | Yageo |
| Q1109 / Q1110 | 2 | release switch N-MOSFET | onsemi **BSS138** | onsemi bss138-d.pdf |
| Q1112 / Q1113 | 2 | release pilot P-channel JFET (normally on) | onsemi **MMBFJ177LT1G** | onsemi mmbfj177lt1-d.pdf (Rev 9, Oct 2024) |
| R1188 / R1189 | 2 | release MOSFET gate-source 1 MΩ | Yageo RC0603 | Yageo |
| R1135 / R1136 | 2 | pilot-bias bleed 1.8 MΩ | Yageo RC0603 | Yageo |
| R1190, R1191 / R1192, R1193 | 4 | pilot-pump inner-node bleeds 2.2 MΩ | Yageo RC0603 | Yageo |
| C1125–C1130 / C1131–C1136 | 12 | pilot pump 3 × 10 nF, 2 × 4.7 nF, 1 × 2.2 nF C0G 50 V 0603 | KEMET C0603C103/472/222J5GACTU | KEMET C0G |
| D1117–D1119 / D1120–D1122 | 6 | pilot-pump low-leakage diode pair | Diodes Inc. **BAV199-7-F** | diodes.com BAV199 (DS30232 Rev 12-2) |
| U1101 / U1102 | 2 | **held** sensor, photo-Darlington | Toshiba **TLP187(TPL,E** | toshiba TLP187_datasheet_en_20260727 (Rev 6.0), via browser |
| R1137 / R1139 | 2 | held string series 3.9 kΩ | Yageo RC0603 | Yageo |
| D1125 / D1126 | 2 | held threshold zener 4.7 V (low-current) | onsemi **MMSZ4688T1G** | onsemi mmsz4678t1-d.pdf |
| R1138 / R1140 | 2 | LED shunt 22 kΩ | Yageo RC0603 | Yageo |
| U1103 / U1104 | 2 | over-voltage **force** opto | Toshiba TLP187(TPL,E | as above |
| R1141 / R1142, R1143, Q1111 | 4 | OVP LED resistors 240 Ω, `OVP_TRIP` default 100 k, LED switch BSS138 | Yageo; onsemi BSS138 | as above |
| R1144–R1151 | 8 | bleeder 2.49 MΩ 1 % 2512 | Stackpole **RVC2512FT2M49** | sei-rvc.pdf |
| R1152–R1161 | 10 | main string 4.02 MΩ 2512 | Stackpole **HVCR2512FTD4M02** (**not a standard value — see C-2**) | sei-hvcr.pdf |
| R1162, R1163 | 2 | main bottom leg 80.6 kΩ 0.1 % 1206 | Panasonic ERA-8AEB8062V | Panasonic ERA-xA |
| R1164, D1123, TP1101 | 3 | main tap 10 kΩ + clamp + test point | Yageo; TI **TPD1E01B04DPYR** | ti.com tpd1e01b04.pdf |
| R1165–R1184 | 20 | supervisor string 1.99 MΩ 0.1 % 25 ppm 1206 | Vishay **TNPV12061M99BEEA** | vishay.com doc 28881 (29-Apr-2026), via browser |
| R1185, R1186 | 2 | supervisor bottom leg 79.6 kΩ 0.1 % | Vishay TNPW120679K6BEEA | Vishay TNPW e3 |
| R1187, D1124, TP1102 | 3 | supervisor tap 10 kΩ + clamp + test point | as main | |
| TP1103–TP1106 | 4 | test points `HOLD_A_HELD_N`, `HOLD_B_HELD_N`, `STKA_GTP`, `STKB_GTP` | pads | §15 |

The test relay K1301 and the 200 Ω test load R1303 are assigned to sheet 13 by the contract (§2, §8) and are
**not** on this sheet.

## 3. The depletion stack: datasheet facts and checks

### 3.1 IXTT10N100D2 [DS, DS100326A(4/17), "Preliminary Technical Information"]

VDSX 1,000 V (TJ 25–150 °C); VGSX ±20 V continuous, VGSM ±30 V transient; PD 695 W (TC 25 °C); TJ −55…150 °C;
RthJC 0.18 K/W; weight 4 g (TO-268). VGS(off) **−2.5 to −4.5 V** (VDS 25 V, ID 1 mA). IDSX(off) ≤ 10 µA at VDS =
1,000 V, VGS = −5 V, 25 °C; ≤ 250 µA at 125 °C. RDS(on) ≤ 1.5 Ω at VGS = 0, ID = 5 A; ID(on) ≥ 10 A at VGS = 0,
VDS = 25 V. gfs 11/17 S. Ciss 5,320 pF, Coss 300 pF, Crss 70 pF (VGS −10 V, VDS 25 V). Qg 200 nC, Qgs 19 nC, Qgd 98 nC.
Body diode VSD ≤ 1.3 V at 10 A. **SOA specification: VDS 800 V, ID 0.22 A, TC 75 °C, tp 5 s (176 W).** Fig. 11
(normalised VGS(off) and BVDSX vs TJ, read from the curve): VGS(off) ≈ 0.93 at −20 °C, 1.00 at 25 °C, ≈ 1.05 at
60 °C; BVDSX ≈ 0.94 at −20 °C. Fig. 9: the cut-off moves more negative when hot. Fig. 16 (FBSOA, TC 75 °C) has a
**DC line**: power-limited (≈ 400 W) up to ≈ 300 V, then falling to ≈ 0.13–0.15 A at 1,000 V. Fig. 17: ZthJC ≈ 0.045
K/W at 1 ms, ≈ 0.10 at 10 ms, ≈ 0.16 at 100 ms, 0.18 at ≥ 1 s (curve readings).

Pinout: TO-268 terminals 1 Gate, 2 and 4 (tab) Drain, 3 Source. KiCad's TO-268-2 footprint numbers the tab pad 2
and has no centre-lead pad, so symbol pin 2 = drain = tab.

### 3.2 Cut-off margin [CALC]

Worst cut-off magnitude over −20…+60 °C: 4.5 V × 1.05 = **4.74 V** (hot); smallest: 2.5 V × 0.93 = **2.33 V**
(cold). Hold bias at the worst simulated corner −7.4 V [SIM] → **2.66 V beyond the worst cut-off**. The "held"
threshold (§5) sits at 5.49–7.00 V: at least **0.75 V beyond the worst cut-off**, so the sensor cannot read
"held" while the bottom device is at or above its cut-off; and at least **0.40 V inside** the worst hold level with
an end-of-life opto (1.0 V with a new one).

### 3.3 Static sharing [CALC]

String 4 × 10 MΩ from the stack top to `HV_RTN`, taps T1–T3. The gate of upper device k goes through 100 kΩ to tap
T(k−1); the device then sets its own source about |VGS(off)| above its gate, i.e. each upper device is a **source
follower of its tap**. Node voltages are fixed by the string, not by leakage: each device blocks V/4 (545 V at the
OVP comparator's top, 2,182 V) ± the cut-off spread (≤ 2.2 V). The stack's off-state leakage (≤ 10 µA at 25 °C;
≈ 30 µA at 60 °C by log interpolation **[EST]**; less at −7 V than at the −5 V test point) simply flows through the
followers; the only current the taps must supply is the gate network's (10 MΩ gate-source: 0.35 µA at 3.5 V; TVS
leakage ≤ 1 µA at 15 V, far less at 3.5 V) → tap error ≤ 0.35 µA × 10 MΩ ≈ 3.5 V of 545 V. String current 55 µA,
30 mW per 2512 element (rated 2 W, 3,000 V). The strings and the stack leakage form an extra, always-present
bleed path from `HV_CAP_P` (through the dump resistors) — they help, they cannot keep a hazardous voltage, and they
touch no node of the held sensing (which is only the bottom gate and `HV_RTN`).

**Finding F-1 (R-22 is not fixed by going to four devices).** With tap-referenced followers a **shorted** device
does not redistribute its share over three devices as the design says (727 V). If device k shorts, the device
below it (k−1) — or, for the top device, the one below the top — now blocks two tap intervals: **V/2 = 1,091 V at
2,182 V, 1,004 V at 2,008 V, 975 V at a normal 1,950 V charge**, against 1,000 V at 25 °C and ≈ 940 V at −20 °C.
A shorted string resistor does the same (it shorts one tap interval). The overstressed device most likely breaks
down and then fails short; the stack then conducts (the capacitor still empties) but the bottom gate is still
biased, so the sensor keeps reading "held" while the path is conducting: no hazard, but every charge then runs
into a conducting 4.5 kΩ path (charge time-out; τ fit). **Proposed fix (contract issue C-1):** five devices per
stack (shorted neighbour 2/5 × 2,182 = 873 V; at −20 °C 93 % of 940 V), or 1,500–1,700 V parts of the same D2
family (Littelfuse lists D2 depletion parts "up to 1,700 V"; part not checked), or a mid-stack voltage monitor.
I did not change the device count because the contract fixes eight IXTT10N100D2 and the reference block.

### 3.4 Dynamic sharing and the gate network [CALC/EST]

- 3.3 nF / 2 kV C0G across each string resistor = 11 × Coss(25 V) and far more than Coss at hundreds of volts.
  The contract §8 says 1 nF; §9 says "≥ 10 × Coss" — 1 nF is only 3.3 × the 300 pF datasheet value, so I chose
  3.3 nF (contract issue C-10). 545 V on a 2,000 V part; 2220 case so the pad gap (≈ 3.5–4 mm) exceeds the
  placeholder in-string spacing rule 0.6 + 0.005 × 545 = 3.3 mm [UNVERIFIED rule].
- **Release (turn-on):** the bottom device's gate rises; it starts to conduct, its drain falls a little, the
  follower above sees its VGS rise and conducts more, and so on: the upper devices stay followers of their taps
  while the taps (resistive + capacitive divider of the falling stack voltage) fall together, so each device keeps
  ≈ V_stack/4 throughout the transition. When fully on, each upper gate is pulled above its source and clamped by
  its TVS at ≈ 17–18.5 V (SMF15CA VBR 16.7–18.5 V, VC 24.4 V at 8.2 A; VGSX ±20 V, VGSM ±30 V); the 100 kΩ limits
  the clamp current. Every device conducts at VGS ≥ 0 (RDS(on) ≤ 1.5 Ω).
- **Re-hold (turn-off):** in normal use the hold is applied before charging (nothing to turn off). If a hold
  returns during a dump the bottom device turns off into the remaining current (≤ 0.485 A) and the followers
  re-take their tap shares; transient sharing is set by the 3.3 nF capacitors. **[SIM needed with the maker's
  model]**
- **Every device has a defined gate bias under any single open:** bottom gate — 1.01 MΩ to `HV_RTN`, TVS, release
  switch (three independent paths); upper gates — 100 kΩ to the tap **and** 10 MΩ gate-source (capture addition):
  if the 100 kΩ opens, VGS → 0 and the device conducts (= the "shorted device" case, F-1); if the 10 MΩ opens, the
  100 kΩ still defines it; if the TVS opens, only the clamp is lost (fault-case gate over-voltage possible).
- Upper-gate resistor stress: a few volts normally (follower); a fault that collapses the devices below puts up to
  the device's own share (545 V, 727 V with F-1) across it for < 1 ms (the 3.3 nF taps discharge through it,
  τ ≈ 0.7 ms, ≈ 2–9 mJ). RVC2512 100 kΩ: 316 V working, 790 V overload (2.5 × √(P·R)) [DS]: **pulse rating
  [UNVERIFIED]**.

### 3.5 Safe operating area and energy during a release [CALC/EST]

The stack turns on into up to 2,182 V through 4.5 kΩ. Linear-region power of the whole stack is at most
V²/(4R) = 2,182² / 18,000 = **265 W**, at a stack voltage of V/2 → **66 W per device at 273 V, 0.24 A**.

- **Normal release** (release switch working): the bottom gate crosses the ≈ 0.8 V linear window (Fig. 9:
  ≈ 0.5 A at 0.8 V above cut-off) in ≲ 1–2 ms [EST from the reservoir discharge rate]: ≤ 265 W × 2 ms ≈ 0.5 J for
  the stack, **≈ 0.13 J per device**; junction rise ≈ ZthJC(2 ms) × 66 W ≈ 0.06 × 66 ≈ 4 K. FBSOA (TC 75 °C) at 273 V:
  the 1 ms and 10 ms lines are far above 0.24 A. **Large margin.**
- **Release circuit failed** (gate decays only through 1.01 MΩ, τ = 3.3 s): crossing the linear window takes
  ≈ 3.3 s × 0.8/4 ≈ 0.7 s. Energy into the stack is bounded by the stored energy: in the V/2 regime about half the
  energy goes to the stack, ≈ 119 J at 2,182 V → **≈ 30 J per device over ≈ 0.5–1 s** at ≤ 66 W. The DC FBSOA line
  at 273 V is power-limited (≈ 400 W at TC 75 °C), and the guaranteed SOA point is 176 W for 5 s at 800 V, so
  66 W per device is inside the curves **provided the case stays ≤ 75 °C**. Thermal: ZthJC(1 s) × 66 W ≈ 12 K plus
  case heating ≈ 30 J / (≈ 1.6 J/K for a 4 g TO-268 with its copper) ≈ 19 K **[EST]**. Plausible on paper; this is
  a single-fault case and path B normally dumps the capacitor in parallel within ≈ 1 s. **Needs SPICE with the
  IXYS model and a PCB thermal model** (design §16 [SIM] item 4).
- **Fully on**, the "decaying 0.4–1 A for hundreds of ms": ≤ 0.485 A per path, VDS ≈ 0.7 V, 0.35 W per device,
  ∫I²R dt = 0.485² × 1.5 Ω × 0.225 s ≈ **0.08 J per device per dump** — trivial; the datasheet SOA is not the limit.
- One shorted device (F-1): the overstressed neighbour is in avalanche/breakdown, not linear operation; no SOA
  rating covers it (no avalanche rating given). Listed in the failure table.

## 4. Hold pumps, release, re-hold (stack A; B identical)

### 4.1 Why four stages (departure D-1) [CALC, SIM]

A single clock (`HOLD_A_DRV`, 0/3.3 V) can drive only a single-phase multiplier: each stage adds at most
Vclk − 2·VF. Three stages from 3.1 V with cold Schottky drops (BAT54S VF ≤ 0.32 V at 1 mA at 25 °C [DS]; ≈ 0.40 V
at −20 °C [EST, +2 mV/K]): 3 × 3.1 − 6 × 0.40 = **6.9 V unloaded** — the contract's "−7.0 V or more negative at
1.1 mA" cannot be met at the cold corner even with no load. My model of the contract's own 3-stage arrangement with
2.2 µF and 1.1 mA gives −6.1 V nominal [SIM]. A two-phase Dickson would need a complementary clock net that the
contract does not have. I therefore captured **four stages** (4 × BAT54S, 4 pump + 3 hold capacitors of 2.2 µF,
plus the 3.3 µF reservoir) **and** cut the load from ≈ 1.1 mA to ≈ 0.3–0.9 mA by using a photo-Darlington sensor
(§5).

Pump connections (stage k): pump capacitor from the clock to P_k; BAT54S pin 1 (A) = H_k, pin 3 (COM) = P_k, pin 2
(K) = H_(k−1) (H_0 = `HV_RTN`); hold capacitor H_k to `HV_RTN`; H_4 = the bottom gate `STKA_G` with the 3.3 µF
reservoir. All stage capacitors are referenced to `HV_RTN` at the stack; the pump capacitors are the only path from
the `GND`-referenced clock, so the pump keeps working if one star tie opens (R-9).

### 4.2 Simulated hold level [SIM — behavioural model, not SPICE]

Model: clock source 30–40 Ω + 10 Ω series, capacitors derated to 70 % at the hot/cold corners (X7R DC bias),
diode VF corners as above, Schottky leakage 1 µA/V at 60 °C [EST], held string as §5, 1.01 MΩ, release circuit as
§4.3.

| Corner | Hold level | Ripple | Clock current rms / peak |
|---|---|---|---|
| 25 °C, 3.3 V, 2 kHz | −8.7 V | 0.15 V | 6.5 / 12 mA |
| −20 °C, 3.1 V, 1 kHz | −7.4 V | 0.24 V | 4.3 / 11 mA |
| −20 °C, 3.1 V, 2 kHz | −7.7 V | 0.11 V | 4.2 / 8 mA |
| 60 °C, 3.2 V, 1 kHz | −7.8 V | 0.27 V | 6.2 / 16 mA |
| 60 °C, 3.2 V, 5 kHz | −8.5 V | 0.07 V | 6.0 / 8 mA |
| 3.5 V clock, fresh parts | −9.1 V | 0.32 V | 8.1 / 21 mA |

Rows 1, 2, 4 and 6 were re-run with the final release circuit and agree within 0.05 V; rows 3 and 5 and the ripple
and clock-current columns come from an earlier run with a 4.7 kΩ/10 kΩ held string (load within ≈ 0.1 mA of the
final one). The over-voltage-force result (§6) was also run before the release circuit was finalised; the release
circuit does not take part in it.

Maximum unloaded gate: 4 × 3.5 − 8 × 0.27 ≈ 11.8 V < 15 V TVS stand-off and ≪ VGSX 20 V. Capacitor DC stress ≤ 13 V
(25 V parts).

### 4.3 Release (departure D-3) [SIM]

The design's release — a depletion BSS126 across gate and source, held off by a small negative bias pumped from the
same clock and referenced to the gate node — was simulated first (BSS126, then the 30 Ω BSS139 of the same family).
Two physical problems appeared: (1) at 1 kHz a bias pump fast enough to collapse in a few ms cannot hold the
release device off against the BSS126/BSS139 threshold spread (−2.7…−1.6 V; hot −2.8 V); (2) once the gate node
starts to rise, the static pump capacitors see that rise as a clock edge and **pump the release device back off**
(the release stalled for > 40 ms). Captured instead:

- **Pilot Q1112, MMBFJ177 P-channel JFET — normally ON with no power** (IDSS 1.5–20 mA, VGS(off) **0.8–2.5 V**,
  rDS(on) ≤ 300 Ω, VDG −25 V, VGS 25 V [DS]). Source `HV_RTN`, drain `RLSA_NG`, gate `RLSA_JB`.
- Its gate bias comes from a **positive 3-stage BAV199 pump referenced to `HV_RTN`** (fixed), so the moving gate node
  cannot pump it. 10 nF pump capacitors, 4.7 nF inner hold capacitors each bled by 2.2 MΩ, 2.2 nF output with
  1.8 MΩ (τ = 4 ms). The inner bleeds are needed because a positive multi-stage pump stopped *low* otherwise keeps
  its output near the inner stage's voltage.
- **Switch Q1109, BSS138** (drain `HV_RTN`, source `STKA_G`, gate `RLSA_NG`, 1 MΩ gate-source R1188): when the pilot
  conducts it puts the whole hold voltage (7–9 V) on the BSS138 gate, which then empties the reservoir and any charge
  left in the pump at 3.5–6 Ω. The energy that turns it on is the charge it removes, so it needs no supply.
- **What releases the stack if the release circuit fails:** the 1.01 MΩ gate-source chain (τ = 3.3 s → conducting
  within ≈ 0.7–1.6 s) and, independently, the other stack. Pilot or BSS138 shorted → the stack can never be held
  (fails safe, found at switch-on).

Simulated times after the last clock edge (gate above −2.3 V = conducting even for the coldest device):

| Corner | Clock stopped | Gate > −2.3 V | "Not held" (LED < 0.19 mA) |
|---|---|---|---|
| 25 °C, 2 kHz, pilot VGS(off) 1.5 V | low | 13.9 ms | 11.4 ms |
| −20 °C, 1 kHz, pilot 2.5 V | low | 1.4 ms | 1.2 ms |
| 60 °C, 1 kHz, pilot 2.5 V | high | 16.4 ms | 9.0 ms |
| 3.5 V clock, pilot 0.8 V (slowest) | low | 21 ms | 17 ms |
| 3.5 V clock, pilot 0.8 V | high | > 30 ms (end of run) | 16 ms |

**The contract's "conducting ≤ 10 ms" and "not held ≤ 5 ms after the last edge" are not met at all corners** (C-7).
Neither is a safety limit in itself: the disarm limit is 3 s and the dump takes ≈ 0.9–1.8 s; the charger cut-off
(D-25) follows "not held" within ≈ 1–17 ms. A second weakness: with a pilot at the top of its VGS(off) spread
(2.5 V), at −20 °C and a 1 kHz clock the pilot bias only just exceeds VGS(off) (3.3 V at the end of a cycle), and a
re-hold after a release did not complete within the 30 ms simulated. That fails safe (stack conducts, "not held",
no charging) but is an availability risk at the 1 kHz end of the accepted band (C-7, doubt 2).

### 4.4 Re-hold and power-up [SIM]

From fully discharged pump capacitors (power-up) "held" appears 25–60 ms after the first edge; after a release
(inner capacitors still charged) 0.5–27 ms. The contract's "held within ≤ 5 ms of the first edge" is met only at
favourable corners (C-7). Firmware F-2 already waits for `HOLD_RB_*_N` before charging.

## 5. "Held" sensing (departure D-2) [DS, CALC]

String per stack: `HV_RTN` → 3.9 kΩ → MMSZ4688 4.7 V zener (4.47–4.94 V at 50 µA; IR ≤ 10 µA at 3 V) → TLP187 LED
(22 kΩ across it) → `STKA_G`. TLP187: photo-**Darlington**, CTR ≥ 1,000 % at IF 1 mA, VCE 1 V; saturated CTR ≥ 500 %
at 10 mA; VCE(sat) ≤ 1.0 V at IC 10 mA / IF 1 mA; dark current ≤ 0.2 µA at 200 V (25 °C), ≤ 20 µA at 85 °C; VCEO
300 V; −55…110 °C [DS]. The interlock sheet pulls `HOLD_A_HELD_N` up with 10 kΩ to `5V0_SYS`: reading "held" needs
the Darlington to sink ≈ 0.46 mA.

- **Threshold** (gate magnitude at which "held" appears) [CALC]: |Vg| = VZ + V_LED + 3.9 kΩ × (I_LED + V_LED/22 kΩ).
  Lowest (zener 4.47 V, LED knee 0.85 V, unspecified-maximum CTR assumed 10,000 % so I_LED = 4.6 µA): **5.49 V**.
  Highest with a new opto (CTR derated to 500 % at low IF, I_LED 92 µA, zener 4.99 V, LED 1.10 V cold): **6.64 V**;
  end-of-life opto (CTR halved, I_LED 184 µA): **7.00 V**.
- Against the cut-off (4.74 V worst): ≥ 0.75 V margin — the sensor cannot read "held" while the bottom device
  conducts. Against the worst hold (−7.4 V): LED current 0.29 mA at the worst zener/LED corner, **3.1 × the new-opto
  need and 1.55 × the aged-opto need**.
- **An aged opto fails to "not held"**: less CTR means less sink current, `HOLD_A_HELD_N` rises, the interlock
  blocks charging. Dark current (≤ 20 µA at 85 °C) pulls the line down by ≤ 0.2 V through 10 kΩ — 17 × below what
  would read "held".
- The 22 kΩ shunt keeps zener leakage (≤ 10 µA → 0.22 V) and any sub-threshold current below the LED's knee; without
  it, a high-CTR Darlington could read "held" from leakage at |Vg| ≈ 4 V, which is a **conducting** stack.
- Open, unsoldered, unpowered (`5V0_SYS` off) or missing opto, open string, open pull-up: all read "not held" (R11).
  Contract R-6 satisfied. Remaining fail-to-"held" fault: Darlington C–E short (fault 9; found by F-7).
- `HOLD_*_HELD_N` low level is up to ≈ 0.8–1.0 V (Darlington): sheet 09 must read it with V_IL ≥ 1.2 V (C-9).

## 6. Over-voltage force (R-10) [DS, CALC, SIM]

`OVP_TRIP` (0/3.3 V, 100 kΩ pull-down here) drives the gate of Q1111 (BSS138, VGS(th) 0.8–1.5 V; RDS(on) specified
only at 4.5 V and 10 V — **[UNVERIFIED] at 3.1 V**, ample for 17 mA). Q1111 sinks two separate LED branches from
`3V3_SUP` through 240 Ω each: (3.3 − 1.15 − 0.1)/240 = **8.5 mA**, ≥ 7 mA cold. Each TLP187 Darlington (collector
`HV_RTN`, emitter `STKx_G`) can then sink ≥ 35 mA (saturated CTR ≥ 500 %), against the running pump's short-circuit
current of ≈ 10–15 mA [SIM]. With a 20 mA sink model the gate is pulled to −0.34…−0.38 V and passes −2.3 V within
**1.1–2.4 ms** at every corner [SIM], while both controllers keep clocking; the held LED goes dark → "not held".
The comparator sees only a MOSFET gate and 100 kΩ (contract issue C-5 explains why the LEDs are fed from `3V3_SUP`).

## 7. Dump resistors [DS, CALC]

Ohmite EY152KE: 1.5 kΩ ± 10 %, 2.5 W at 25 °C, **400 J single impulse**, max working **550 V**, max overload 1,100 V
(2.5 × rated or this, 5 s), max pulse 25 kV, −40…110 °C, TCR −1,200 ± 300 ppm/K; pulse-tolerance test 200 V, 112 J,
10,000 cycles, 26 s rest, ±2 % [DS, Ohmite catalogue as reproduced by DirectIndustry].

| | Both paths | Path A or B alone |
|---|---|---|
| Path resistance | 2 × 4.5 kΩ in parallel = 2.25 kΩ (+ ≈ 3 Ω of RDS(on)) | 4.5 kΩ (+ 6 Ω) |
| τ (100 µF) | 0.225 s | 0.451 s |
| To 50 V from 2,008 / 2,182 V | 0.83 / 0.85 s | 1.66 / 1.70 s |
| To 38 V (§B5.3 "empty") from 2,008 / 2,182 V | 0.89 / 0.91 s | 1.79 / 1.83 s |
| Worst tolerance (+10 % R, +5 % C), one path, 2,182 V → 38 V | — | 2.11 s (limit 3 s) |
| Energy per resistor (1,950 / 2,008 / 2,182 V) | 31.7 / 33.6 / 39.7 J | 63.4 / 67.2 / 79.4 J |
| Peak voltage per resistor | 650 / 669 / 727 V | same |
| Peak power per resistor | 282 / 299 / 353 W | same |
| Peak current per path | 0.43 / 0.45 / 0.485 A | same |

Energy ≤ 20 % of the 400 J single-impulse rating. Repetition: one path alone every 120 s → 79 J/120 s = **0.66 W
average per resistor**; at 60 °C, if the 2.5 W rating derates linearly to 0 at 110 °C (derating curve not seen
**[UNVERIFIED]**), ≈ 1.5 W. Ohmite's own pulse test (112 J every 26 s, 10,000 cycles) is harsher in energy and rate
than our 5,000 × ≤ 80 J at ≥ 120 s, but at 200 V. **Voltage:** 669–727 V exceeds the 550 V *working* rating for the
first ≈ 0.13 s of a one-path dump (0.06 s with both paths) — inside the 1,100 V / 5 s overload rating and the 25 kV
pulse rating, but Ohmite must confirm repeated exposure (R-field `Verify`). Fallback in the parts study (885SP,
1,000 V) would need a contract change. Body dimensions of the EY 2.5 W were not readable (catalogue image too small):
footprint DIN0617 at 25.4 mm pitch **[UNVERIFIED]**.

## 8. Permanent bleeder [CALC]

8 × 2.49 MΩ = 19.92 MΩ (Stackpole RVC2512: 3,000 V working, power-limited √(1 W × 2.49 MΩ) = 1,578 V; 4,000 V
overload). Per element at 2,182 V: **273 V, 30 mW**. Always-connected total: bleeder 19.92 MΩ ∥ main string 40.24 MΩ
∥ supervisor string 39.84 MΩ ∥ two sharing strings 40.0045 MΩ each = **6.66 MΩ**, τ = 666 s:

- 2,008 V → 50 V: 666 × ln(2,008/50) = **41.0 min** (2,182 V: 41.9 min). The design's ≈ 41 min is confirmed.
- Bleeder alone: 1,992 s × ln(40.2) = **2.04 h** (2,182 V: 2.09 h).
- Loss while charged: 2,008²/6.66 MΩ = **0.61 W** (0.72 W at 2,182 V). The stacks' off-state leakage adds to the
  bleed (not counted).

## 9. Sense strings [DS, CALC]

| | Main (`VCAP_M_TAP` → 07) | Supervisor (`VCAP_S_TAP` → 08) |
|---|---|---|
| String | 10 × 4.02 MΩ HVCR2512 (thick film, VCR −2 ppm/V, 100 ppm/K, 1 %) = 40.2 MΩ | 20 × 1.99 MΩ TNPV1206 (thin film, < 1 ppm/V, 25 ppm/K, 0.1 %) = 39.8 MΩ |
| Per element at 2,182 V | 218 V, 12 mW (rating 3,000 V, 2 W) | 109 V, 6 mW (rating 700 V, 0.52 W) |
| Voltage-coefficient error at full scale | −2 ppm/V × 218 V = −0.044 % | < 0.011 % |
| Bottom leg | 2 × 80.6 kΩ 0.1 % (Panasonic ERA-8A) ∥ = 40.3 kΩ | 2 × 79.6 kΩ 0.1 % (Vishay TNPW) ∥ = 39.8 kΩ |
| Ratio | **998.5** (contract 1:998) | **1,001.0** (contract 1:1,000 — use 1,001) |
| Tap at 2,182 V / 60 V | 2.185 V / 60.1 mV | 2.180 V / 59.9 mV |
| Clamp | 10 kΩ + TPD1E01B04 to `GND` | same |
| Clamp leakage error | ≤ 10 nA × ≈ 50 kΩ = 0.5 mV (0.02 %) | same |

Where the legs are: both bottom legs and both clamps are on this sheet, as the contract (§8 sheet 11) assigns; the
bottom legs return to `HV_RTN` (the capacitor's own negative), the clamps to `GND` (R-21). The two strings share no
part, maker, value or node except `HV_CAP_P` and `HV_RTN`; the bleeder is a third maker's series. The clamp's
breakdown is 6.4 V typical, not 3.6 V: a 3.6 V zener leaks too much (MMSZ4685: up to 7.5 µA at 2 V → 0.4 V error);
the input is current-limited to ≤ 55 µA by the 40 MΩ string, so 07/08's own input resistor (R7) carries it (C-11).

## 10. HV_RTN–GND ties and net classes

NT1101 ∥ NT1102 (`Device:NetTie_2`, 2 mm SMD pads — the layout must make them two separate wide bonds carrying the
65 A shock return and ≈ 450 A fault pulse). Everything that must keep working if they separate is AC- or
light-coupled. Net classes: every node that can sit at hundreds of volts is named `HV_*` (61 nets: stack nodes, taps,
upper gates, dump-resistor junctions, bleeder and string nodes) → class HV; `HV_CAP_P` → HV_PULSE; `HV_RTN` →
RTN_PULSE. The bottom-gate, pump, release, held-string and tap-divider nodes (`STK*`, `PMP*`, `RLS*`, `HLD*`,
`VCAP*_DIV`, `OVP_*`) are galvanically on the capacitor through the stacks but stay within ≈ 20 V of `HV_RTN` (bounded
by the gate TVS); I named them without `HV_` so they fall in Default class — otherwise they would get 12.5 mm
clearance from the ground they are tied to (the R-18 problem again). Integrator to confirm (C-13).

## 11. Failure table (every part; stack A shown, stack B identical with its own references)

"Empties" = how the capacitor still empties. "Detect" = which check finds it. ST = switch-on/daily self-test,
F-7 = hold released one at a time with τ fit, charge = charge time-out/τ during charging.

| Part (A / B refs) | Fault | Capacitor still empties? | Reads "held" while conducting? / "not held" forever? | Detected by |
|---|---|---|---|---|
| J1101 | open | Capacitor disconnected from everything (also from the charger and bridge): its own stored energy stays on it (only its internal leakage) — **hazard to anyone opening the device** | no / charging never reaches voltage | charge (no voltage rise); listed as a mechanical-integrity item (doubt 5) |
| J1101 | short | capacitor shorted (fault 42) | — | charge |
| J1102 | open / short | unaffected (NTC only) | no | NTC reading out of range (07/05) |
| NT1101 or NT1102 | open | yes (other tie) | no | none (latent; layout inspection) |
| both ties | open (2 faults) | both stacks still conduct when released (sources on `HV_RTN`); sensors opto-isolated | taps read relative to wrong ground | readings disagree |
| R1144–R1151 bleeder | open | yes, through both paths (bleeder is backup only) | no | not detected (latent; 41 min → ≈ 80 min backup) |
| bleeder | short (one) | yes; 8 → 7 elements, 312 V each | no | not detected (latent) |
| R1152–R1161 main string | open | yes | main reads 0 | main/supervisor disagreement → S latches |
| main string | short (one) | yes | main reads 10 % low (fault 15) | disagreement |
| R1162/R1163 | open (one) | yes | main reads ×2 (fault 16) | disagreement |
| R1164 | open | yes | main reads 0 | disagreement |
| D1123 | short | yes | main reads 0 | disagreement |
| D1123 | open | yes | no clamp (latent) | none |
| R1165–R1186 supervisor string | as main | yes | S reads 0 / ≈ 5 % low / ×2 (fault 16a: OVP trips at ≈ 1,090 V) | disagreement; OVP dumps |
| R1187, D1124 | as main | yes | as main | disagreement |
| TP1101–TP1106 | short to neighbour (handling) | yes | could pull a tap/`HELD_N` | readings |
| **R1101–R1103** dump A | open | yes, path B (fault 8) | no | F-7 (τ with A alone ≫) |
| dump A | short (one) | yes, faster (3 kΩ path, 1,091 V per remaining resistor) | no | F-7 τ shorter |
| **Q1101** (bottom) | short D–S | yes: whole stack A conducts (followers cascade) | **yes — gate still biased, reads "held" while A conducts** | charge (charger into 4.5 kΩ: ≈ 420 V max) and F-7 τ |
| Q1101 | open | yes, path B (fault 7) | no | F-7 |
| Q1101 | G–S short | yes, A conducts | no ("not held") | ST (cannot charge) |
| Q1101 | G–D short | gate tied to its drain → device conducts; TVS clamps gate node to ≤ 18 V | "not held" | ST |
| **Q1102–Q1104** (upper) | short D–S | yes, but **F-1**: neighbour at V/2 (> rating) on the next full charge → likely fails short → A conducts | **"held" while A conducts** after the second failure | latent until the cascade; then charge/F-7 |
| Q1102–Q1104 | open | yes, path B | no | F-7 |
| R1107–R1110 sharing string | open (one) | yes; that follower loses its tap reference → its gate goes via 10 MΩ to its source → conducts (= shorted device, F-1) | as above | latent (F-1) |
| sharing string | short (one) | yes; one tap interval collapses → F-1 | as above | latent |
| C1101–C1104 | short | as string short (F-1) | as above | latent |
| C1101–C1104 | open | dynamic sharing lost on one interval | no | not detected **[SIM needed]** |
| R1115–R1117 upper gate | open | yes; VGS → 0 via 10 MΩ → device conducts (F-1) | as above | latent |
| R1115–R1117 | short | gate on tap directly; TVS takes clamp current from the tap (≤ 55 µA) | no | none |
| R1121–R1123 | open | none (100 kΩ defines gate) | no | none |
| R1121–R1123 | short | VGS = 0 → device conducts (F-1) | as above | latent |
| D1101 (bottom TVS) | short | yes, A conducts | "not held" | ST |
| D1101 | open | yes | no; gate over-voltage protection lost (latent) | none |
| D1102–D1104 | short | device conducts (F-1) | as above | latent |
| D1102–D1104 | open | release still works; upper gate may exceed ±30 V at turn-on **[SIM]** → gate rupture → short/open | as above | latent |
| R1131 (`HOLD_A_DRV` 100 k) | open | yes; no effect while driven; floating input if 09's buffer is dead (input caps keep it near 0; no clock = no bias) | no | none (latent) |
| R1131 | short | clock shorted → no bias → A conducts | "not held" | ST |
| R1133 (10 Ω) | open | yes, A conducts (no clock) | "not held" | ST |
| R1133 | short | none (higher peak buffer current) | no | none |
| C1111–C1114 pump caps | open | yes; one stage dead → bias ≈ −5.5 to −6 V: may read "not held" (fails safe) or hold with less margin | possible "not held" forever on A | ST / F-7 |
| C1111–C1114 | short | clock shorted into a stage → bias collapses → A conducts | "not held" | ST |
| C1115–C1117 hold caps | open | bias lower (more ripple) | possible "not held" | ST |
| C1115–C1117 | short | stage shorted to `HV_RTN` → bias ≈ −2 to −5 V | "not held" | ST |
| D1109–D1112 BAT54S | open | bias collapses → A conducts | "not held" | ST |
| D1109–D1112 | short | bias collapses → A conducts | "not held" | ST |
| C1109 reservoir | open | yes; hold-up ≈ 0 ms, ripple ≫ → "held" flickers | possibly "not held" | ST (flicker) |
| C1109 | short | A conducts | "not held" | ST |
| R1127/R1128 gate-source | open | yes; release circuit still empties the gate; only the slow backup path is lost (fault 10a) | no | F-7 only if the release also fails |
| R1127/R1128 | short | R1128 short: tap to `HV_RTN`, 680 k remains; R1127 short: 330 k load (+20 µA) | no | none |
| TP1105 | — | — | — | — |
| **Q1109** BSS138 release | open | yes, but slowly: 1.01 MΩ (≈ 0.7–1.6 s) — path B meanwhile (fault 10a) | no | F-7 (late start of A's decay) |
| Q1109 | short D–S | A conducts | "not held" | ST |
| Q1109 | G–S short | release lost (as open) | no | F-7 |
| **Q1112** MMBFJ177 pilot | open (no conduction) | release lost (as Q1109 open) | no | F-7 |
| Q1112 | short / always on | Q1109 always on → A conducts | "not held" | ST |
| R1188 (1 MΩ G–S of Q1109) | open | pilot leakage (≤ 1 nA) could slowly turn Q1109 on → A partly conducts | "not held" | ST |
| R1188 | short | Q1109 never turns on, but the pilot itself then conducts 1.5–20 mA straight from `HV_RTN` into the gate node: a slower release (3.3 µF × 7 V / 1.5 mA ≈ 15 ms worst) | no | F-7 timing |
| R1135 pilot bleed / R1190–R1191 inner bleeds | open | yes; release slower (pilot bias held by stored charge, tens of ms to seconds) | no | F-7 timing (late) |
| R1135, R1190–R1191 | short | pilot bias collapses → pilot on → Q1109 on → A conducts | "not held" | ST |
| C1125–C1130 pilot-pump caps | open | pilot not biased → A conducts (fails safe), or (C1130 open) bias ripple large → A partly conducts | "not held" | ST |
| C1125–C1130 | short | pilot bias collapses → A conducts | "not held" | ST |
| D1117–D1119 BAV199 | open / short | pilot bias collapses → A conducts | "not held" | ST |
| **U1101** held opto | Darlington C–E short | yes (sensor only) | **reads "held" always** (fault 9) | F-7: release A alone, `HOLD_RB_A_N` must go high |
| U1101 | LED open / output open / not fitted / aged | yes | "not held" forever on A → no charging | ST |
| R1137 / D1125 / R1138 | open | yes | "not held" | ST |
| R1137 | short | yes; LED current ≈ 1 mA more; threshold ≈ 5.3 V (still > 4.74 V cut-off) | no | none |
| D1125 zener | short | **threshold drops to ≈ 1.2 V: reads "held" while the gate is far above cut-off** — a conducting stack A could read "held" | **yes** | **not detected by F-7 when the release works** (the gate collapses fully, LED goes dark) — see doubt 4 |
| R1138 | short | LED shorted → "not held" | "not held" | ST |
| U1103 OVP opto | Darlington C–E short | A conducts | "not held" | ST |
| U1103 | LED open or output open | yes; OVP force lost on A only (other stack's force remains) | no | not detected (latent; no OVP test) |
| R1141 | open / short | force lost on A / LED overcurrent only during a trip (13 mA) | no | latent |
| Q1111 OVP switch | open | OVP force lost on both stacks (one layer; primary trip and firmware remain, fault 33) | no | latent |
| Q1111 | short | both forced on → no charging | "not held" both | ST |
| R1143 (`OVP_TRIP` 100 k) | open | none while 08 drives; if 08 is unpowered, Q1111's gate floats (could force — fails safe) | no | none |
| `3V3_SUP` lost | — | S dead → path B conducts; OVP force unavailable (nothing to force: B already on) | — | M link loss |

**Unsafe outcomes and what I did.**
- *Fail-to-"held" through a conducting bottom device (Q1101 short, upper-device cascade from F-1).* Inherent to
  sensing the gate, not the current. The capacitor still empties; the charger feeds 4.5 kΩ (≈ 420 V at 40 W), charge
  time-out and τ fit detect it. Not fixable on this sheet without a current sensor in each stack; recorded.
- *Zener short → "held" at almost any negative bias.* This is new with the low-current zener design and **is a
  latent single fault that F-7 cannot see** while the release works. With a second fault (release slow or failed,
  main frozen with charger on) the charger could run while stack A is only partly off. Within this sheet, a fix is a
  second series element so one short cannot remove the threshold (e.g. two 2.4 V zeners in series, or zener +
  silicon diode chain); I did **not** add it in this capture because it moves the threshold band and needs the same
  margin study — **recorded as finding F-2 and top doubt 4**.
- *J1101 open.* The capacitor keeps its charge with nothing on the board able to drain it: the off-board capacitor
  needs its own bleed or a terminal design that cannot open (doubt 5, Appendix A says "internal bleed resistor: none,
  unless agreed" — propose agreeing one).

## 12. Departures from the design specification

| # | Design / contract | Captured | Why |
|---|---|---|---|
| D-1 | 2 × three-stage pumps, −7.4 V at 1.1 mA | **four stages**, 2.2 µF | a single-clock 3-stage pump is ≤ 6.9 V unloaded at −20 °C (§4.1) |
| D-2 | hold-sense opto "VOS618A class, CTR at 1 mA", LED ≈ 1 mA, 1.5 kΩ | **TLP187 photo-Darlington**, 3.9 kΩ + 22 kΩ LED shunt, LED 0.3–0.9 mA | the pump cannot supply 1.1 mA at −7 V (§4.1); VOS618A needs ≈ 0.5 mA LED for 0.46 mA out new and fails to "not held" early in life |
| D-3 | release: BSS126-class depletion switch with clock-derived bias | **P-JFET pilot (normally on) + BSS138**, positive 3-stage pilot pump referenced to `HV_RTN` | the gate-referenced bias either would not stay off at 1 kHz or was pumped back off by the rising gate (§4.3) |
| D-4 | supervisor string 12 × 3.32 MΩ TNPV1206 | **20 × 1.99 MΩ TNPV1206** (ratio 1:1,001) | TNPV1206 range ends at 2.0 MΩ [DS 28881] |
| D-5 | speed-up 1 nF 1 kV C0G | **3.3 nF 2 kV C0G 2220** | ≥ 10 × Coss (300 pF) per contract §9; pad gap for 545 V |
| D-6 | `OVP_TRIP` drives the opto LEDs through a resistor | `OVP_TRIP` → BSS138 → two LED branches from `3V3_SUP` | 17 mA from the comparator output is unreasonable; contract §4 already lists `3V3_SUP` on 11 for these LEDs |
| D-7 | gate-source 1 MΩ | 680 k + 330 k = 1.01 MΩ | §15 1:3 test tap without extra load |
| D-8 | — | 10 MΩ gate-source on each upper device | defined gate bias if a 100 kΩ opens |
| D-9 | tap clamp "3.6 V zener/TVS" | TPD1E01B04 (VRWM 3.6 V, breakdown 6.4 V, ≤ 10 nA) | a 3.6 V zener's leakage would cost up to 0.4 V of a 2.2 V tap |
| D-10 | — | 10 Ω clock series resistor per pump | limits buffer peak current (8–21 mA [SIM]) |
| D-11 | §B5.4 "≤ 10 ms conducting; hold-up 3.5 ms" | 1.4 ms to > 30 ms; not-held 1.2–17 ms [SIM] | see §4.3 |

## 13. Contract issues (proposed changes; I changed no net names)

- **C-1 (major) R-22 not fixed.** Tap-referenced followers put a shorted device's share on one neighbour: V/2 =
  1,091 V at 2,182 V (975 V at 1,950 V, above the ≈ 940 V cold rating). *Proposal:* five devices per stack (refs
  Q1101–Q1105 / Q1106–Q1110, release switches renumbered), or higher-voltage D2 parts, and correct §B5.4/§13
  fault 6 ("727 V" is wrong).
- **C-2 HVCR2512 4.02 MΩ is not a standard value** (HVCR list 10 M–100 M, rev 9/22/2025; "other values may be
  available"). Captured as the contract says, flagged `Verify`. *Proposal:* confirm with Stackpole, or use Stackpole
  HVC2512 (precision HV chip, 1 M–100 M at 0.1 %, 3,000 V, ±25 ppm) at 4.02 MΩ, or 4 × 10 MΩ HVCR2512 with a
  re-computed bottom leg (WP-C's thresholds follow).
- **C-3 Supervisor ratio** becomes 1:1,001 (D-4). WP-C (08: OVP 2,160 V, AD7171 scaling) must use 1,001.0.
- **C-4 Pump requirement** "−7.0 V at 1.1 mA from a 2 kHz 3.3 V clock" with three stages is not achievable with one
  clock phase. *Proposal:* "four stages; ≤ −7.4 V at −20 °C, 3.1 V, 1 kHz at the sheet's real load (≤ 0.9 mA)", or
  add a complementary clock net `HOLD_A_DRV_N`/`HOLD_B_DRV_N` (two-phase Dickson: 3 stages would then give
  ≈ −8.7 V).
- **C-5 OVP LED drive:** contract §5.4 says `OVP_TRIP` → LEDs via resistor, §4 says `3V3_SUP` feeds the LEDs on 11.
  Captured: `3V3_SUP` + BSS138 gated by `OVP_TRIP`. *Proposal:* state this in §5.4 ("loads: one MOSFET gate + 100 k").
- **C-6 Hold-drive filter position:** the 8 kHz two-pole low-pass (§9 WP-D, design §B5.4) must sit **before** the
  hold buffer on 09; the buffer output must drive the pump directly (≤ 40 Ω, 0/3.3 V square, ≥ 10 mA rms, 20 mA
  peaks). A filter after the buffer would starve the pump.
- **C-7 Timing numbers (§10 WP-D with WP-F):** simulated "held" 0.5–27 ms after the first edge (25–60 ms from a cold
  pump), "not held" 1.2–17 ms after the last edge, conducting 1.4 to > 30 ms. *Proposal:* raise the minimum hold-clock
  frequency to 2 kHz (F-1), and restate the limits as "held ≤ 30 ms, not held ≤ 20 ms, conducting ≤ 50 ms" — or
  ask for SPICE before fixing numbers. None is a safety limit (dump 0.9–1.8 s, disarm 3 s).
- **C-8 Release switch:** contract §8 "BSS126 class with clock-derived bias" → JFET pilot + BSS138 (D-3). Q1112/Q1113
  are new references inside the sheet's block.
- **C-9 `HOLD_*_HELD_N` low level** is ≤ 1.0 V (Darlington VCE(sat)); 09's reader needs V_IL ≥ 1.2 V at 5 V
  (74HC/AHC/LVC at 5 V all qualify; 3.3 V logic does not).
- **C-10 Speed-up capacitor:** §8 says 1 nF, §9 says ≥ 10 × Coss (= 3 nF): use 3.3 nF (D-5).
- **C-11 Tap clamp level:** the clamp breaks down at ≈ 6.4 V; 07/08 must keep their ≥ 100 kΩ input resistors (R7) so
  ≤ 55 µA into a 3.3 V ADC pin is harmless.
- **C-12 `HV_CAP_P` default resistor** "(bleeder on 11)": satisfied by the bleeder.
- **C-13 Net classes:** low-voltage nodes on the capacitor side are named without `HV_` (Default class); confirm
  with the integrator that the layout keeps them away from the `HV_*` nets by the HV rule.
- **C-14 Off-board capacitor bleed:** Appendix A says "internal bleed resistor: none, unless agreed". J1101 open
  leaves the capacitor charged with no drain at all. *Proposal:* agree an internal bleeder (or a terminal construction
  that cannot open) with the capacitor maker.

## 14. Unverified pins, ratings and assumptions

- **Pins:** BAT54S and BAV199 — Diodes Inc. drawings show the diode arrangement but no pin numbers; standard SOT-23
  top-view numbering assumed (1 = anode of the lower-left diode, 2 = cathode of the lower-right, 3 = common). MMBFJ177
  1 D / 2 S / 3 G and BSS138 1 G / 2 S / 3 D read from onsemi's text. IXTT10N100D2 tab = pin 2 by the KiCad footprint's
  numbering (datasheet calls it 4). TLP187 1 A / 3 K / 4 E / 6 C from the datasheet text. `PinoutVerified` reads "NO"
  for both custom symbols until a person checks them.
- **Footprints:** J1101 has no part or footprint (`footprint_todo`); TPD1E01B04 (TI X1SON "DPY") placed on the stock
  0402 diode footprint; EY 2.5 W body length; TO-268 tab pad numbering.
- **Ratings not stated by the datasheets:** EY repeated 727 V for 0.13 s (550 V working); EY derating curve;
  RVC2512 100 kΩ short-pulse voltage; BSS138 RDS(on) at 3.1 V; TLP187 CTR below IF = 1 mA and its maximum CTR
  (none specified); CTR ageing (not specified by Toshiba — I assumed −50 % at end of life); IXTT10N100D2 avalanche
  (none given) and its "Preliminary" status; KEMET X7R part codes (series checked, exact codes not); HVCR 4.02 MΩ
  availability; MMBFJ177 VGS(off) temperature drift; BAV199 leakage at 60 °C at 3–10 V (only 25 °C/75 V and
  150 °C/75 V given).
- **Assumptions:** clock source ≤ 40 Ω; 3V3 rails 3.1–3.5 V at the pump; X7R at 70 % of nominal under bias and
  temperature; Schottky VF +2 mV/K cold; capacitor-case temperatures as the device's −20…+60 °C.
- **All timing and pump numbers are [SIM] from a behavioural model I wrote for this capture** (time-step, exponential
  diodes with series resistance, square-law JFET, linear MOSFETs). It has not been compared with SPICE or measurement.

## 15. Things a reviewer should check first (my own top five doubts)

1. **F-1: shorted-device sharing.** I believe the design's 727 V is wrong for this topology and the neighbour sees
   V/2. If I am right, four 1,000 V devices are not enough at 2,182 V (C-1).
2. **The release and re-hold timing at the 1 kHz end and with pilot JFETs at either end of their 0.8–2.5 V spread**
   (§4.3). My model says it works but slowly, and marginally at −20 °C/1 kHz. This wants SPICE with real models,
   and probably a contract decision on the clock frequency (C-7).
3. **The hold pump is now four stages from one clock** and still only −7.4 V at the cold/1 kHz corner; the held
   threshold band (5.49–7.00 V) leaves 0.4 V with an end-of-life opto there. Check the CTR-ageing assumption and the
   X7R derating.
4. **F-2: a shorted threshold zener** makes the held sensor read "held" at almost any bias and F-7 cannot see it.
   Consider two series threshold elements (and redo §5).
5. **The dump resistors' voltage** (727 V vs 550 V working) and **J1101/the off-board capacitor's own bleed** —
   both need the makers (Ohmite; the capacitor maker).

## 16. Totals and build

182 parts (93 resistors, 36 capacitors, 26 diodes, 13 transistors, 4 optocouplers, 2 connectors, 2 net ties, 6 test
points); 115 nets, 395 pin connections; 10 inter-sheet global nets (`HV_CAP_P`, `HOLD_A_DRV`, `HOLD_B_DRV`,
`HOLD_A_HELD_N`, `HOLD_B_HELD_N`, `OVP_TRIP`, `VCAP_M_TAP`, `VCAP_S_TAP`, `NTC_CAP_M`, `NTC_CAP_S`) and three rails
(`GND`, `HV_RTN` — made here, flagged — and `3V3_SUP`). `bash tools/build.sh`: 0 ERC errors; accepted until the
driver sheets are captured: `GND` and `3V3_SUP` not driven (01, 03), `NTC_CAP_M`/`NTC_CAP_S` single-ended (07, 05);
J1101 footprint to be made; netlist round trip matches the pin table exactly.

Not done: SPICE of the stack, pump and release; PCB thermal estimate for the TO-268s; the five-device alternative;
a two-element threshold (F-2); a test-point check against the as-yet-uncaptured readers.

---

## Revision 0.3 (27 November 2026)

> **NOT FOR CONSTRUCTION — unreviewed design.** Every figure marked [SIM] below is a simulation with hand-built models
> (ngspice 42, `hardware/aed-main/sim/rev3-wpf/`), not a measurement. Nothing here has been built, and nothing here
> is safe or ready for use on a person or animal.

Applied from `capture-notes/rev3-changes.md` section WP-F (F-1…F-12), against `docs/HARDWARE_DESIGN.md` v0.3 and
`contract.md` issue 3. Branch `worktree-agent-a3965dc4adad4d619`.

### R3.0 In plain language

The two discharge paths now use stronger 1,700-volt transistors, and each transistor's control wire has a one-way
valve (a high-voltage diode) so that one failed transistor cannot drag its neighbours' share of the voltage onto a
single part. Each path also has two independent "held" sensors on separate wires, so one failed sensor part can
no longer make the path look held when it is not.

I drew all of that, then simulated the circuit exactly as drawn, part by part, including every single part failing
open or short. Three things came out that the change order did not expect:

1. **One part of the change order had to be changed.** The voltage-sharing chain was a single 4.7 MΩ resistor per
   step. If the bottom one of those ever broke open, the new valves would let the whole capacitor voltage
   (≈ 2,170 V) land on one transistor rated about 1,670–1,700 V [SIM]. I made each step **two 10 MΩ resistors side
   by side** (5.0 MΩ): if one breaks the other still holds the step, and the worst case is then well inside the
   rating. It also solves a parts problem: no high-voltage 4.7 MΩ chip resistor could be confirmed, while the 10 MΩ
   part is already on this sheet. Eight extra resistors, change W-1.
2. **The cold-weather "held" check still does not work in my simulation unless the hold clock is at least about
   3.2 V.** The earlier check runs that said it worked at −20 °C were, it turns out, run at room temperature for
   the semiconductor parts (a quirk of the simulator, explained in R3.7). At a true −20 °C with the 3.1 V clock the
   design assumes, the hold voltage is about 0.4–0.5 V too weak and the sensors flicker. This fails safe (the
   device refuses to charge in the cold) but it would make the device unusable; it needs the design authority and
   the interlock-sheet owner, because the fix is the clock amplitude or the pump, not a value on this sheet.
3. **A shorted reference diode in a "held" sensor is now caught**, better than the change order says: it overloads
   the hold pump, the other sensor reads "not held", and the device refuses to charge — visible at the next
   self-test instead of hiding.

With change W-1, every single-part failure I simulated keeps every transistor inside its rating in steady state and
through a realistic charge–shock–recharge sequence, with the smallest margin about 0.19 of the rating (one cold
Monte Carlo run with a shorted sharing-string element; 0.24 for every other fault) [SIM]. The one exception is a
deliberately impossible test (a 1 µs voltage jump followed by a 2 ms recharge with a transistor already shorted),
reported in R3.3. Discharge still works by default with any one part failed.

### R3.1 Changes applied (designators)

| Change | Parts | What |
|---|---|---|
| F-1 | Q1101–Q1108 | IXTT10N100D2 → **IXTT2N170D2** (new symbol `aed_storage:IXTT2N170D2`, pins 1 G / 2 D = tab / 3 S read from DS100418B page 2; the old symbol is removed) |
| F-2 | **D1127–D1132** GP02-40 (DO-41, 12.7 mm pitch footprint) | gate steering diode in each upper feed; R1115–R1120 now end on new nets `HV_A_GF2..4` / `HV_B_GF2..4`; anode `HV_x_GFk`, cathode `HV_x_Gk` |
| F-3 | **R6101–R6106** 22 MΩ CRHV2512 | across each steering diode |
| F-4 | R1121–R1126 | 10 MΩ 0603 → **100 MΩ CRHV2512** |
| F-5 + **W-1** | R1107–R1114 + **R6115–R6122** | sharing string: each of the four elements is **2 × 10 MΩ HVCR2512 in parallel** (5.0 MΩ) instead of one 4.7 MΩ — departure, see R3.8 |
| F-6 | **R6107–R6114** 10 MΩ HVCR2512, **C1137–C1144** 3.3 nF 2 kV C0G 2220 | drain–source across every device |
| F-7 | C1101–C1108 | kept; MPN aligned with the new parts to **C2220Y332JGGACAUTO** (the only 3,300 pF 2 kV C0G 2220 code found in a listing) |
| F-8 | D1112 / D1116 pin 1 → `PMPA_H4` / `PMPB_H4`; **R1194 / R1195** 470 Ω | pump output resistor |
| F-9 | Q1112, Q1113 | MMBFJ177 → **MMBFJ270** (stock P-JFET symbol, 1 D / 2 S / 3 G — see R3.5) |
| F-10 | D1125, D1126 → **MMSZ4687T1G**; **R1196 / D1133 / R1197 / U1105 / TP1107** (A), **R1198 / D1134 / R1199 / U1106 / TP1108** (B) | second held sensor per stack on new globals **`HOLD_A_HELD2_N`**, **`HOLD_B_HELD2_N`** |
| F-11 | group notes | pump requirement, release ≤ 25 ms stopped low, 1.5 kHz; "727 V" removed (dump-resistor text now 733 V at 2,200 V) |
| F-12 | — | capacitor bleed and surge rating stay requirements on C1101 (`docs/requirements/storage-capacitor.md`); nothing drawn |

New references: D1127–D1134, C1137–C1144, U1105–U1106, R1194–R1199, TP1107–TP1108, overflow R6101–R6122.
Sheet total 230 parts (was 182). The sheet keeps its NOT FOR CONSTRUCTION notice.

### R3.2 Static sharing, as drawn [SIM `stack_static_wpf.cir`]

Stack A as drawn (B is identical). Each case is a DC solution at 2,200 V (1,950 V where stated). Temperature
factors from IXYS DS100418B Fig. 12: **cold** = −20 °C, \|V_GS(off)\| × 0.89, device leakage ≤ 6.5 µA, steering-diode
leakage 0.5 µA [EST], rating **1,666 V**; **hot** = 70 °C, × 1.11, ≤ 96 µA, steering-diode leakage **50 µA** (its
100 °C maximum, a deliberate bound), rating **1,700 V**. Monte Carlo: resistors ± 2 %, V_GS(off) anywhere in
−2…−4 V, device leakage 2–100 % of its maximum, diode leakage 0–100 % of the corner value; 100 runs for the
no-fault and shorted-device cases, 12 runs for every other fault. **Margin = 1 − worst / rating.**

| Case (2,200 V unless stated) | Cold: worst device V (margin) | Hot: worst device V (margin) |
|---|---|---|
| No fault (Monte Carlo 100) | 566 V (0.66) | 640 V (0.62) |
| Any one device shorted (Monte Carlo 100, random position) | 1,168 V (0.30) | 1,267 V (0.25) |
| Device shorted, by position (Q1101 short: the whole path conducts): Q1101 / Q1102 / Q1103 / Q1104 | 3 / 1,166 / 910 / 808 V (worst 0.30) | 3 / 1,246 / 1,051 / 929 V (worst 0.27) |
| Any one device open | 578 V (0.65) | 646 V (0.62) |
| String element shorted (R110x ‖ R611x), by position 1 / 2 / 3 / 4 | 655 / 747 / 1,353 / 748 V (worst 0.19) | 733 / 780 / 855 / 814 V (worst 0.50) |
| One resistor of a string pair open | 895 V (0.46) | 896 V (0.47) |
| String capacitor C1101–C1104 shorted | 753 V (0.55) | 886 V (0.48) |
| Per-device 10 MΩ R6107–R6110 shorted | 1,165 V (0.30) | 1,274 V (0.25) |
| Per-device 10 MΩ open | 1,219 V (0.27) | 1,007 V (0.41) |
| Per-device 3.3 nF C1137–C1140 shorted | 1,175 V (0.29) | 1,277 V (0.25) |
| 100 kΩ feed R1115–R1117 shorted | 568 V (0.66) | 625 V (0.63) |
| 100 kΩ feed open | 1,107 V (0.34) | 1,107 V (0.35) |
| Steering diode D1127–D1129 shorted | 565 V (0.66) | 614 V (0.64) |
| Steering diode open | 565 V (0.66) | 625 V (0.63) |
| 22 MΩ bypass R6101–R6103 shorted | 567 V (0.66) | 622 V (0.63) |
| 22 MΩ bypass open | 1,104 V (0.34) | 648 V (0.62) |
| 100 MΩ gate–source R1121–R1123 shorted | 1,168 V (0.30) | 1,277 V (0.25) |
| 100 MΩ gate–source open | 566 V (0.66) | 619 V (0.64) |
| Gate TVS D1101–D1104 shorted | 1,171 V (0.30) | 1,284 V (0.24) |
| Gate TVS open | 563 V (0.66) | 618 V (0.64) |
| *Change order's single 4.7 MΩ element open: R1107 / R1108 / R1109 / R1110 (nominal parts)* | 2,156 / 1,257 / 1,022 / 873 V (worst -0.29) | 2,175 / 1,226 / 1,233 / 883 V (worst -0.28) |

What the table says, in plain terms:
- Healthy stack: every device 553 V with nominal parts, ≤ 566 V cold / 640 V hot with tolerance spread.
- **Any one device shorted: ≤ 1,168 V cold / 1,267 V hot (margins 0.30 / 0.25)** — the design authority's "≈ V/2" holds; the hot figure is set by the
  steering diodes' leakage (next table).
- **The worst single fault is a shorted sharing-string element (R1109 ‖ R6117, or its capacitor C1103): 1,353 V**
  in one cold Monte Carlo run (735 V with nominal parts). In that run all three upper devices were pinched hard off
  (gates at the 15 V clamp) and the bottom device took the per-device resistors' share; I did not find a mechanism
  that makes this unique to the cold corner, and it may be a second DC solution reached by the sweep. Margin 0.19.
- Open per-device resistor across the bottom device (R6107): 1,219 V cold (0.27).
- **The change order's single 4.7 MΩ element, open:** **2,156 V cold / 2,175 V hot on the bottom device** with R1107 open (1.29 / 1.28 × the rating), 1,257 V with R1108 open. That is why W-1.

Steering-diode leakage, hot, device 2 shorted, nominal parts [SIM]:

| GP02-40 leakage | 0 | 5 µA (25 °C max) | 50 µA (100 °C max) | 200 µA |
|---|---|---|---|---|
| Bottom device | 1,161 V | 1,172 V | 1,278 V | 1,631 V |

About 2.2 V per µA. The datasheet's typical curve gives ≈ 1.3–3 µA at 100 °C, so the 50 µA row is pessimistic
by roughly an order of magnitude at 70 °C; the 200 µA row shows where it would stop being acceptable.

Stress on the other parts in the worst static fault cases [SIM]: steering diode reverse ≤ 548 V (rated
4,000 V); 22 MΩ bypass the same voltage, ≤ 14 mW; 100 kΩ feed ≤ 7 V (it no longer sees the fault voltage);
upper gate–source ≤ 18 V (TVS clamp, so the 100 MΩ dissipates nothing); string resistor ≤ 892 V per element
(one of a pair open) — all within the HVCR/CRHV 3,000 V working ratings.

### R3.3 Dynamic sharing [SIM `stack_dynamic_wpf.cir`, `stack_shock_wpf.cir`]

Bounding events (much faster than anything that can reach a held stack, as in sim/charger-storage): 1 µs step
1,000 → 2,200 V; 2,200 → 1,200 V in 10 ms (shock-like); 1,200 → 2,200 V in 2 ms. Worst C0G pattern = the string
capacitor and the per-device capacitor of one position at −5 %, all others +5 %.

| Case | Cold (margin) | Hot (margin) |
|---|---|---|
| Healthy, worst C0G pattern, any event | 646 V (0.61) | 1,033 V (0.39) — 1 µs step |
| Healthy, Monte Carlo 40 runs (all 3.3 nF ± 5 %, cut-off, leakage) | 814 V (0.51) | 1,044 V (0.39) |
| Device 2 shorted, 1 µs step | 1,633 V (0.02) | 1,615 V (0.05) |
| Device 2 shorted, 10 ms fall / 2 ms rise straight after that step | 1,600 / **2,087 V (fails)** | 1,280 V |
| Device 3 / 4 shorted, any event | 1,021 / 802 V | 1,061 / 844 V |

**Realistic sequence** (`stack_shock_wpf.cir`: charge to 2,200 V in 1 s, shock 2,200 → 1,200 V in 10 ms, recharge
in 1 s, worst C0G pattern): healthy ≤ 554 V; device 2 shorted ≤ 1,161 V cold / 1,280 V hot; device 3 ≤ 1,023 V;
device 4 ≤ 836 V — never above the static value. The 2,087 V case needs a 1 µs step and a 2 ms recharge within half
a second of each other with a device already shorted; the charger cannot do either. Reported in R3.7 (item 8), not
acted on. Release edge (turn-on from 2,200 V): ≤ 179 V on any device (`discharge_wpf.cir`).

### R3.4 Hold, release and "held" readback across −20…+70 °C [SIM `held_threshold_wpf.cir`, `hold_held_wpf.cir`]

Thresholds (gate magnitude at which a line falls below 1.2 V), 47 kΩ + 1 MΩ pull-ups on sheet 09, semiconductors at
the stated temperature:

| Corner | Threshold, each sensor | Margin over the 2N170D2's worst cut-off at 70 °C (4.44 V) | One zener shorted | One zener open |
|---|---|---|---|---|
| Low (70 °C, 4.09 V zener, CTR 10,000 %) | **4.72 V** | **0.28 V** | that sensor 0.71 V; HOLD_OK still needs 4.72 V | that line never low (4.10 V) → never "held" |
| Nominal (25 °C) | 5.47 V | 1.03 V | 1.15 V; HOLD_OK 5.47 V | never "held" |
| High (−20 °C, 4.52 V, aged CTR 250 %) | **5.98 V** | — | 1.40 V; HOLD_OK 5.98 V | never "held" |

Hold pump, release and both lines (clock stopped **low**; "held" = both lines < 1.2 V; requirement HELD_N ≤ 1.0 V
while held; "conducting" = gate above the smallest cut-off at that temperature, 1.78 / 2.0 / 2.22 V):

| Case | Held after power-up | Hold, worst point | Highest HELD_N while held | Conducting after stop | "Not held" after stop |
|---|---|---|---|---|---|
| −20 °C, aged, 3.1 V, 1.35 / 1.5 / 1.65 kHz, pilot 2.0 V | 34 / 35 / 56 ms | −5.59 / −5.52 / −4.30 V | **4.92 / 4.97 / 4.99 V — fails** | 1.3–1.4 ms | 1.3 ms |
| −20 °C, **new** sensors (CTR 500 %), 3.1 V, 1.5 kHz | 33 ms | −5.52 V | **4.95 V — fails** | 1.3 ms | 1.3 ms |
| −20 °C, aged, 3.1 V, 1.5 kHz, pilot 0.5 V | 31 ms | −5.87 V | **2.88 V — fails** | 7.4 ms | 6.9 ms |
| −20 °C, aged, **3.2 V**, 1.5 / 1.35 kHz | 29 / 30 ms | −6.04 / −5.91 V | **0.78 V** / 2.19 V (fails) | 1.7 ms | 1.6 ms |
| −20 °C, aged, **3.3 V**, 1.5 kHz | 27 ms | −6.23 V | **0.63 V** | 1.9 ms | 1.9 ms |
| −20 °C, aged, **2.9 V**, 1.5 kHz | never (pilot not pinched off; gate −1.25 V) | | | | |
| 25 °C, nominal, 3.3 V, 1.5 kHz, pilot 1.2 V | 27 ms | −5.87 V | 0.59 V | 4.0 ms | 4.0 ms |
| 70 °C, new sensors, 3.2 V, 1.35 kHz, pilot 0.5 V | 18 ms | −6.04 V | 0.55 V | **20.9 ms** | 18.3 ms |
| 70 °C, aged, 3.1 V, 1.65 kHz, pilot 2.0 V | 24 ms | −6.33 V | 0.60 V | 2.1 ms | 2.1 ms |
| 25 °C, fresh, 3.5 V, 1.65 kHz, pilot 0.5 V | 19 ms | −6.33 V | 0.55 V | **22.0 ms** | 19.0 ms |
| −20 °C, sensor 2 zener **shorted** | never "held" (gate sags to −3.92 V) | | sensor 1 line 4.99 V | | |
| −20 °C, sensor 2 zener **open** | never "held" | −5.57 V | sensor 2 line 4.99 V | | |
| 70 °C, sensor 2 zener **shorted** | never "held" (gate −4.43 V) | | sensor 1 line 4.95 V | | |

- **Release ≤ 25 ms with the clock stopped low: met** (worst 22.0 ms conducting, 19.0 ms "not held").
- **Held at −20 °C, 3.1 V, aged sensors: not met** in this model (problem 6 in R3.7). The aged-cold threshold
  (5.98 V) is 0.46 V above the hold level (−5.52 V). At 3.2 V it is met only near 1.5 kHz; at 3.3 V it is met.
- **Each held sensor with its zener shorted or open: fails to "not held" and is visible at the next self-test.**
- The 1 ms clock gap never released a held path where the path held (HELD_N ≤ 0.61 V during it).

### R3.4a Discharge and resistor pulse stress [SIM `discharge_wpf.cir`]

From 2,200 V to below 60 V (always-connected loads included, the capacitor's own 10 MΩ left out):

| Case | Time | Peak current per path | Dump resistor (each): peak power / energy | Bottom device: peak / energy |
|---|---|---|---|---|
| One path, nominal | 1.63 s | 0.49 A | 354 W / 80.0 J | 63 W / 0.35 J |
| One path, slowest (R + 10 %, cold TCR, C + 5 %) | 1.98 s | 0.42 A | 304 W / 84.0 J | 59 W / 0.32 J |
| One path, fastest (R − 10 %, hot, C − 5 %) | 1.32 s | 0.57 A | 415 W / 75.9 J | 72 W / 0.39 J |
| Both paths, nominal / slowest | 0.82 / 0.99 s | 0.49 / 0.42 A | 354 / 304 W, 40 / 42 J | 0.18 J |
| One path, R − 10 % with C + 5 % (most energy per resistor) | 1.54 s | 0.54 A | 391 W / 83.9 J | 70 W / 0.41 J |
| One path, device 2 shorted | 1.63 s | 0.49 A | 355 W / 80.2 J | 96 W / 0.35 J |
| One path, **release failed** (gate decays through 1.01 MΩ only) | **4.68 s** | 0.23 A | 78 W / 44.6 J | 41 W / **26.6 J** |

Limit 3 s: met by one path alone except when its release has also failed (then path B, held by the other
controller, must release too — as in v0.2). Dump resistors: ≤ 84 J of the EY's 400 J single-impulse rating; 733 V
across each at the start against 550 V working (unchanged open question for Ohmite). Every high-value resistor
dissipates microwatts to milliwatts during a dump: R6107 (10 MΩ per device) ≤ 0.14 W peak (with a shorted
neighbour), R1107 (string) ≤ 43 mW, R6101 (22 MΩ) ≤ 11 mW, R1115 (100 kΩ) ≤ 0.18 W for microseconds (≤ 14 µJ),
R1121 (100 MΩ) ≤ 3 µW — negligible against the 1–2 W ratings; no pulse rating is published to compare with.
Standing loss per stack at 2,200 V: string 0.24 W + per-device resistors 0.12 W.
### R3.5 Datasheets consulted, and what each says about the items the design authority could not verify

| Part | Document (as opened 27 Nov 2026) | What was checked | Finding |
|---|---|---|---|
| IXTT2N170D2 | IXYS **DS100418B(02/12)**, from the datasheets.com mirror (no Littelfuse-hosted copy found) | ratings, pinout, **Fig. 12** (normalised BV_DSX and V_GS(off) vs T_J) | V_DSX 1,700 V (T_J 25–150 °C); BV_DSX ≥ 1,700 V; V_GS(off) −2.0…−4.0 V; I_DSX(off) ≤ 25 µA / 500 µA (125 °C); R_DS(on) ≤ 6.5 Ω; C_iss 3,650 pF; terminals 1 G, 2/4 D (tab), 3 S — **pinout confirmed**. **Fig. 12 (typical curves, my reading):** BV_DSX ≈ 0.98 at −20 °C (≈ 0.985 at −40 °C) → **1,666 V cold**, *better* than the assumed 0.94 / 1,598 V; but \|V_GS(off)\| ≈ 0.89 at −20 °C and **≈ 1.11 at 70 °C**, *worse* than the assumed 1.06 → worst cut-off magnitude **4.44 V** at 70 °C, not 4.24 V. No avalanche rating. FBSOA point 204 W at 1,700 V / 120 mA / 5 s (T_C 75 °C) |
| GP02-40 | Vishay **doc 88635, revision 09-Aug-2022** | ratings, Fig. 4 (reverse leakage vs % V_RRM) | V_RRM 4,000 V; I_R ≤ 5 µA (25 °C) / ≤ 50 µA (100 °C) at rated V; typical (Fig. 4) ≈ 1.3–3 µA at 100 °C, almost flat with voltage; C_J 3 pF; DO-41. **Marked "Not for New Designs"** — see R3.8 |
| CRHV2512 (22 MΩ, 100 MΩ) | Vishay **doc 68002, revision 12-Nov-2024** | range, voltage, VCR | 2512: 10 MΩ–1 GΩ (1 %), 3,000 V max working (or √(P·R)), 1 W at 70 °C, VCR 10 ppm/V, TCR ±100 ppm/K. **No pulse/surge rating stated** (only "short time overload < 0.5 % ΔR", level not given). Ordering codes built from the document's scheme, not seen in a listing |
| HVCR2512 (10 MΩ) | Stackpole SEI-HVCR — **not re-opened** (vendor URL returns a 404 page; distributor copy timed out) | — | 3,000 V working and 10 MΩ in the standard list, as read for the v0.2 capture [UNVERIFIED this session] |
| 4.7 MΩ HV 2512 (change order F-5) | CRHV 68002 (above); HVCR list as read for v0.2 | is the value orderable? | **Not found**: HVCR list starts at 10 MΩ, CRHV2512 at 10 MΩ; CRHV1210 4M70 exists (1,750 V) but its 3.2 mm body is short for 733–880 V. Resolved by W-1 (2 × 10 MΩ) |
| C2220Y332JGGACAUTO | **DigiKey listing** only; KEMET C1035 datasheet **not opened** (expired TLS certificate on content.kemet.com) | rating, construction | 3,300 pF ± 5 %, 2,000 V, C0G, 2220, floating-electrode, flexible termination, AEC-Q200, ≤ 2.7 mm thick [distributor, UNVERIFIED]. C0G capacitance change with DC voltage is negligible for class-I ceramics (general property, **not checked in KEMET's data**). The previously drawn C2220C332JGGACTU was **not** found in any listing |
| MMBFJ270 | onsemi **MMBFJ270/D Rev 2, May 2022** | ratings, pinout | V_DG −30 V, V_GS 30 V, V_GS(off) 0.5–2.0 V, I_DSS 2–15 mA. **Pin numbers are not printed**: the package drawing puts D and S on the two-lead side and G alone, consistent with 1 D / 2 S / 3 G (as the MMBFJ177) [human check needed] |
| MMSZ4687T1G | onsemi **MMSZ4678T1/D Rev 13, Aug 2021** | V_Z, leakage, **temperature coefficient** | 4.09 / 4.30 / 4.52 V at 50 µA; I_R ≤ 4 µA at 2 V; **Fig. 1: typical TC ≈ 0 mV/K at 4.3 V** (≈ +0.8 mV/K for the old 4.7 V part), so ngspice ignoring the zener `tbv1` no longer matters |
| TLP187, BSS138, BAT54S, BAV199, SMF15CA, EY152KE | unchanged from v0.2 (datasheets as listed in §2–§7) | — | — |

**Creepage at 2,200 V (note only; no layout).** Voltages across each part are the simulated worst cases of R3.2/R3.3.
- TO-268 (IXTT2N170D2): drain is the tab; gate and source are the two outer leads (lead pitch 5.45 mm, from the
  standard TO-268 outline; the TO-268 dimension table was not read in DS100418B). Tab-to-source sees ≈ 550 V
  normally and up to ≈ 1.3 kV after a single fault (static) or 1.6 kV transiently. The package's own tab-to-lead
  distance (a few mm) is below plain-board creepage for 1.3–1.6 kV; coating or potting will be needed. Gate–source
  ≤ 18.5 V (TVS).
- DO-41 (GP02-40) on 12.7 mm pitch: anode–cathode normally < 1 V, up to the steering voltage after a device short
  (≈ 0.45–0.6 kV [SIM]); ≈ 10 mm between pads.
- 2512 chips (HVCR/CRHV): ≈ 4.5 mm between terminations [EST]; up to 880 V (string pair, one open) and
  ≈ 1.3 kV (per-device 10 MΩ across a device after a neighbour's short) → coating needed for the latter.
- 2220 C0G: ≈ 4.5 mm terminal gap [EST]; up to ≈ 1.3–1.6 kV after a fault → coating (surface arc-over is a known
  limit of HV MLCCs above ≈ 1 kV; KEMET guidance not read).
- The whole stack (top to `HV_RTN`) is 2,200 V: group distances between the top device, dump resistors and
  low-voltage nodes are a layout requirement (§B12), not settled here.
### R3.6 Part-failure table for the new and changed parts (stack A shown; stack B identical with its own references)

"Discharges by default?" = does the path still empty the capacitor when nothing holds it (no clock, no power)?
"Visible?" = which check finds the fault: **ST** = switch-on / daily self-test (the path cannot be held, or a
held line reads "not held"), **T-7** = per-sensor read-back during the self-test discharge (each of the four HELD
lines proven separately, §B14.5), **τ** = discharge time-constant fit when each path is released alone,
**charge** = charge time-out. Device voltages are the simulated worst case at 2,200 V across −20…+70 °C with
tolerance spread (static, R3.2), against 1,666 V (cold) / 1,700 V (hot).

| Part (A / B) | Fault | Discharges by default? | Worst device voltage | Visible? |
|---|---|---|---|---|
| Q1101–Q1104 (IXTT2N170D2) | any one D–S short | yes (that device conducts; with no hold all conduct) | see R3.2 table: ≤ 1,168 V cold / 1,267 V hot (Q1102 worst; a Q1101 short makes the whole path conduct) | no, latent while the other three share; a second short is then a cascade risk. τ unchanged (4.5 kΩ dominates). **Latent** |
| Q1101–Q1104 | open | path A no; path B yes (fault 7) | ≤ 646 V | τ (path A alone does not discharge) |
| Q1101 | G–S short | yes, A conducts | — | ST (cannot hold A) |
| R1107–R1110 ‖ R6115–R6118 (string pair) | one resistor of a pair open | yes | ≤ 896 V | **latent** (that tap shifts by ≈ 1/5 of the voltage) |
| string pair | element shorted | yes | ≤ 1,353 V (cold, one run; 855 V hot) | latent |
| *as change order: single 4.7 MΩ* | *open* | *yes* | ***2,156–2,175 V — above the rating*** | *latent; the device then breaks down* |
| C1101–C1104 | short | yes | as element short, ≤ 886 V | latent |
| C1101–C1104 | open | yes | dynamic only: see R3.3 | latent |
| R6107–R6110 (per-device 10 MΩ) | open | yes | ≤ 1,219 V | latent |
| R6107–R6110 | short | yes (that device bypassed) | ≤ 1,274 V | latent (as a device short) |
| C1137–C1140 (per-device 3.3 nF) | short | yes | ≤ 1,277 V | latent (as a device short) |
| C1137–C1140 | open | yes | static unaffected; fast edges see the device's own capacitance (R3.3) | latent |
| R1115–R1117 (100 kΩ feed) | open | yes | ≤ 1,107 V (gate then held only by 100 MΩ to its source → device conducts, as a short) | latent |
| R1115–R1117 | short | yes | ≤ 625 V | none needed (feed resistance only) |
| D1127–D1129 (GP02-40) | open | yes | ≤ 625 V (22 MΩ still feeds the gate) | latent |
| D1127–D1129 | short | yes | ≤ 614 V; a **second** fault (a device short above it) then drags the tap as in v0.2 (≈ 2.1 kV, CS cfg 1) | latent |
| R6101–R6103 (22 MΩ bypass) | open | yes | ≤ 1,104 V | latent |
| R6101–R6103 | short | yes | ≤ 622 V (diode bypassed: the v0.2 feed) | latent |
| R1121–R1123 (100 MΩ G–S) | open | yes | ≤ 619 V | none |
| R1121–R1123 | short | yes (device conducts at VGS = 0) | ≤ 1,277 V (= a device short) | latent |
| D1101–D1104 (gate TVS) | short | yes (device conducts) | ≤ 1,284 V; bottom TVS short: A conducts | upper: latent; D1101: ST |
| D1101–D1104 | open | yes | ≤ 618 V, but the gate–source voltage then reached 73 V in a hot run (V_GSM ±30 V): gate rupture likely, after which the device is a short (≤ 1,284 V) | latent (clamp lost) |
| R1194 (470 Ω pump output) | open | yes: no bias reaches the gate, A conducts | — | ST ("not held") |
| R1194 | short | yes | — | none; the cold power-up lock-out of sim/rev3 §2 can return at > 1.6 kHz (fails to "not held") |
| Q1112 (MMBFJ270 pilot) | open / never conducts | yes, slowly: release only through R1127 + R1128 (1.01 MΩ), path B meanwhile | — | τ / late "not held" in T-7 |
| Q1112 | always on | yes, A conducts | — | ST |
| D1125 / D1133 (MMSZ4687) | **short** | yes | — | **ST: the shorted sensor's LED pulls ≈ 1.2 mA from the pump, the gate sags to −3.9 V (cold) / −4.4 V (hot) and the other sensor reads "not held"** [SIM R3.5]. Fails safe and visible (better than the change order's "the other sensor decides") |
| D1125 / D1133 | open | yes | — | ST: that line never reads "held" |
| R1137 / R1196 (3.9 kΩ) | open | yes | — | ST (that line "not held") |
| R1137 / R1196 | short | yes | — | not visible; threshold ≈ 0.2–0.4 V lower on that sensor only (other sensor unchanged) |
| R1138 / R1197 (22 kΩ shunt) | open | yes | — | not visible; that sensor's threshold falls ≈ 0.3 V and leakage can reach the LED — the other sensor still decides |
| R1138 / R1197 | short | yes | — | ST (that line "not held") |
| U1101 / U1105 | Darlington C–E short | yes | — | T-7 (that line stays low when A is released alone) |
| U1101 / U1105 | LED or output open, not fitted, aged | yes | — | ST ("not held") |
| TP1107 / TP1108 | short to neighbour | yes | — | ST / T-7 |
| `HOLD_A_HELD2_N` wire | open | yes | — | ST (47 kΩ pull-up on 09 reads "not held") |

No single fault of a new part stops a path from discharging by default. Every single fault that raises a device
above its simulated fault-free level is latent (not seen by a self-test); with W-1 none of them exceeds the rating
in this model (worst margin in R3.2). The zener short, previously the latent fault F-2/I-4, is now visible.
### R3.7 Problems with the change order and the evidence behind it (for the design authority)

1. **F-5 single 4.7 MΩ string elements make a single open resistor an overstress** [SIM R3.2]. With the gate
   feeds steered, an open bottom element (R1107) lets tap T1 float up to the stack top; every upper device then
   conducts and the bottom device is asked to block **2,156–2,175 V** (cold rating 1,666 V; ratio 1.29). An
   open R1108 gives 1,257 V. This is the same class of fault as I-3 (a tap losing its reference) and was not in the
   CS Q1 case list (string *short* only). **Departure W-1** (below) fixes it on this sheet.
2. **F-5 part not available as specified**: no 4.7 MΩ high-voltage 2512 chip was found (HVCR list and CRHV2512
   both start at 10 MΩ). W-1 uses the existing HVCR2512 10 MΩ.
3. **F-2 GP02-40 is "Not for New Designs"** (Vishay 88635 rev 09-Aug-2022). It is drawn as ordered, with a
   `Verify` field. A replacement must be a 2–4 kV, ≤ 0.25 A rectifier whose **hot reverse leakage stays below
   ≈ 50 µA at 70 °C**: device-short sharing degrades about 2.2 V per µA of steering-diode leakage (R3.2 table:
   1,161 / 1,172 / 1,278 / 1,631 V at 0 / 5 / 50 / 200 µA, hot, device 2 shorted).
4. **F-1 temperature factors**: Fig. 12 gives BV_DSX ≈ 0.98 at −20 °C (1,666 V, better than assumed) but
   \|V_GS(off)\| ≈ 1.11 at 70 °C (worse than the 1.06 assumed). With a true 70 °C sensor model the lowest held
   threshold is **4.72 V** [SIM R3.4], so the margin over the worst cut-off (4.44 V) is **0.28 V**, not the 0.71 V
   of sim/rev3 §1. Still positive, but thin; it rests on typical-curve readings.
5. **The −20 °C results in `sim/rev3` and `sim/charger-storage` were not at −20 °C for the semiconductors.** In
   ngspice 42 `reset` returns the circuit to the netlist temperature (27 °C), and those netlists set `option temp`
   before `reset`. Checked with one BAT54 at 1 mA: 0.323 V after "−20 °C" + `reset` (the same as 27 °C); 0.263 V
   with `option temp=70` given after `reset`. My netlists set `option temp=$tc` after `reset`.
6. **With that corrected, the aged cold case fails** [SIM R3.4]: at −20 °C, 3.1 V clock, capacitors at 70 %,
   both sensors read "held" only on the ripple peaks (HELD_N swings to 4.9 V) at 1.35, 1.5 and 1.65 kHz — with
   aged *and* with new sensors (CTR 500 %). Hold worst point −5.5 V against an aged-cold threshold of 5.98 V. At a
   3.2 V clock it holds at 1.5 kHz (HELD_N ≤ 0.78 V) but not at 1.35 kHz; at 3.3 V it holds; at 2.9 V the release
   pilot is never pinched off and the path never holds. Fails safe (no charging) but unusable in the cold. Not
   fixable by a value on this sheet: bigger pump capacitors (4.7 µF) or a 220 Ω output resistor bring back the
   power-up lock-out; a 47 kΩ LED shunt only lowers the swing to 4.4 V. **Needs a decision** (raise and specify
   the hold-clock amplitude at the pump ≥ 3.2 V over temperature on sheet 09, or redesign the pump), then a cold
   measurement. This is the "unsettled" item; my answer is that it is not met in this model.
7. **F-10's description of a shorted zener is incomplete** (in the safe direction): the other sensor does not
   simply "decide" — the shorted string loads the pump (≈ 1.2 mA), the gate sags to −3.9 V (cold) / −4.4 V (hot)
   and the path reads "not held", so the fault is visible at the next self-test. At 70 °C the sagged gate (−4.43 V)
   sits at the worst cut-off (4.44 V): the bottom device may conduct slightly, but the path reads "not held" and
   the charger is not enabled.
8. **Dynamic bounding case with a shorted device**: a 1 µs 1,000 → 2,200 V step on a stack with device 2 shorted
   puts 1,633 V on the bottom device (margin 0.02); with the network not yet settled, the following
   bounding events reach 2,087 V (above the rating). These steps are far faster than anything that can reach a held stack (the
   charger needs seconds); the realistic shock-and-recharge sequence (`stack_shock_wpf.cir`) never exceeds the
   static value (≤ 1,280 V). Reported, not acted on.
9. Smaller points: the 100 kΩ gate-feed resistors R1115–R1120 now see < 1 V normally and ≤ 7 V in a fault
   (the diode takes the stress), so their v0.2 pulse-rating `Verify` is withdrawn; R1121–R1126 (100 MΩ) see ≤ 18 V
   while the TVS is intact, so a high-voltage chip is not needed there electrically — kept as ordered (surface
   leakage at 100 MΩ favours the larger body).

### R3.8 Departures from the change order

| # | Change order | Drawn | Why |
|---|---|---|---|
| W-1 | F-5: R1107–R1114 4.7 MΩ HV chip | **each element 2 × 10 MΩ HVCR2512 in parallel** (R1107–R1114 + new **R6115–R6122**), 5.0 MΩ per element, 20 MΩ per string | one open 4.7 MΩ element puts 2,156–2,175 V on one device [SIM]; with a pair, one open resistor gives ≤ 896 V. Also no 4.7 MΩ HV 2512 part was found. Costs 0.02 W more standing loss per stack at 2,200 V (string 0.24 W instead of 0.26 W: lower, since 20 MΩ > 18.8 MΩ) and slightly softer sharing (≤ +54 V in the device-short Monte Carlo against the 4.7 MΩ string) |
| W-2 | F-7: keep C1101–C1108 (C2220C332JGGACTU) | MPN changed to **C2220Y332JGGACAUTO** (same value, voltage, dielectric, case) for all 16 3.3 nF parts | the old code was not found in any listing; the AUTO code is the change order's own candidate for C1137–C1144 |
| W-3 | F-2: "GP02-40" | drawn as **GP02-40-E3/54** (ordering form of the datasheet's example code) with a `Verify` flag | part is "Not for New Designs" (problem 3) |

### R3.9 What still needs measurement or better models

- The hold level at a true −20 °C with the real clock buffer, pump capacitors and diodes (problem 6); the 1.35–1.65
  kHz window; the clock amplitude actually delivered at the pump input over temperature.
- IXTT2N170D2 V_GS(off) spread at 70 °C (Fig. 12 is a typical curve), BV_DSX at −20 °C (typical curve), and its
  behaviour in avalanche (no rating) — every single-fault margin here is to the *rating*, not to breakdown.
- Steering-diode reverse leakage at 70 °C for whichever part replaces GP02-40 (≤ 50 µA assumed as a bound).
- HVCR/CRHV pulse capability (not published) — simulated pulse dissipation is tiny (R3.3), but surge ratings matter
  for a dump into a shorted stack and for ESD/handling.
- KEMET C2220Y332JGGACAUTO datasheet: floating-electrode rating, surface arc-over guidance, ageing (C0G: none).
- Creepage/coating at 1.3–1.6 kV across single parts after a fault (R3.5).
- Thermal: the "release failed" case puts ≈ 27 J into the bottom device over ≈ 4.5 s at ≤ 41 W peak
  [SIM]; needs the maker's SOA at mid voltages and a board thermal model.
- All models are hand-built; no maker SPICE model exists for the depletion devices. The simulator needed
  continuation (DC sweeps) and a 20 µs step limit to give converged answers; an unsettled pseudo-transient fallback
  and a kinked leakage source both produced plausible-looking wrong numbers during this work (fixed; see R3.2).

### R3.10 Build, cross-check and files

- `bash tools/build.sh`: **BUILD OK** — 0 ERC errors; sheet 11 now 230 parts; netlist round trip matches the pin
  table. Nothing new needed an accepted ERC warning.
- `python3 tools/xcheck.py`: `HOLD_A_HELD2_N` and `HOLD_B_HELD2_N` are listed as **single-sheet nets** — expected:
  their far end (pull-ups and the four-line AND on 09_interlock, WP-D) is not yet drawn in this worktree, where sheet
  09 is still at issue 2 (its `HOLD_x_HELD_N` pull-ups still read 10 kΩ). Names are as contract issue 3 §5.4; not
  renamed.
- Files changed: `hardware/aed-main/specs/11_hv_storage.toml`, `hardware/lib/src/aed_storage.toml` (IXTT2N170D2
  replaces IXTT10N100D2), the generated `hardware/lib/aed_storage.kicad_sym`, `hardware/aed-main/11_hv_storage.kicad_sch`
  and the regenerated PDF, this file, and the new folder `hardware/aed-main/sim/rev3-wpf/` (netlists, models,
  `gen_ctl.py`, `run.sh`, `notes.md`).
- Simulation bookkeeping: on a heavily loaded machine the 70 °C half of `stack_static_wpf` was run as two parallel
  processes with the same cases (the second half with Monte Carlo seed 20261130); the committed `stack_static_wpf.ctl`
  runs everything in one process with seed 20261128, so a re-run reproduces the cases and the same order of
  results, not every last volt of the hot fault rows.
