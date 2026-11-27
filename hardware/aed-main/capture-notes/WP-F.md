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
