# WP-D capture notes — shock logic (sheets `09_interlock` and `12_bridge_logic`)

**NOT FOR CONSTRUCTION — unreviewed design.** These notes explain how two circuit pages were drawn. They were
written with AI assistance on 27 November 2026 and have not been reviewed by a qualified high-voltage or
medical-device engineer. Nothing here is an instruction to build, power or test anything (SAFETY.md §2a,
CLAUDE.md rule 2). Nothing here is safe or ready for use on a person or animal. Every number marked [CALC] is
hand or script arithmetic from datasheet limits, not a measurement and not a SPICE simulation; [SIM-B] marks a
behavioural model run by a small script (described in §10), not a circuit simulation.

Sources: `docs/HARDWARE_DESIGN.md` v0.2 §B5.4, §B7.2, §B8, §B13, §B14, §13, §F-1/F-4/F-8; `hardware/aed-main/contract.md`
issue 2; DECISIONS.md D-25, D-59, D-60, D-61; `firmware/crates/aed-core/src/interlock.rs`; review
`docs/reviews/2026-11-27-hardware-design-review.md` (R-1, R-4, R-6, R-11, R-13, R-23). Pin tables:
`hardware/aed-main/specs/09_interlock.toml`, `hardware/aed-main/specs/12_bridge_logic.toml`; custom symbols:
`hardware/lib/src/aed_logic.toml`.

**Revision 0.3 (27 November 2026).** Sections 1–10 describe the first (issue-2) capture and stay as its record.
The changes of design revision 0.3 / contract issue 3 are in **§11 Revision 0.3** at the end; where §11 and an
earlier section disagree, §11 is current.

---

## 1. What is on each sheet (plain language)

### 1.1 Sheet 09 — the hardware interlock (170 parts)

This page decides whether the output stage may have power at all, and whether the charger may run. Power for the
output stage (`BRIDGE_5V`) comes from the 5 V rail through **four electronic switches in a row**; each switch is
closed by a different "yes": the main controller's request (SW1), the supervisor's permit or its low-voltage test
permit (SW2a or SW2b), proof that **both** discharge paths are really held off (SW3), and a **150 ms time window**
(SW4). Any one "no" opens the row. A fifth switch (SW5) sits beside the row: it is closed only by the shock
sequencer on sheet 12 once a shock has actually started, so that a shock that has begun is finished by hardware
even if a controller withdraws its "yes" (decision D-59).

The supervisor's permits are not simple on/off wires. They are square waves of 1–5 kHz that only running code
produces. Each is turned back into a "yes" by a small detector: a filter that ignores fast signals (a pin left on a
clock), a two-phase charge pump that only fills when edges keep arriving, and a comparator. If the edges stop, the
"yes" disappears within about 7 ms; ordinary gaps of up to 1 ms in the code's timing do not disturb it.

"Held" means the light-emitting diode inside each stack's optocoupler on sheet 11 is actually lit by the negative
bias that keeps the stack off. Only two lit LEDs give `HOLD_OK`; a broken wire, an unpowered part or a missing
resistor all read "not held". The charger is enabled only with `HOLD_OK` (decision D-25), through two separate
chains of gates for the charger's supply and its CHARGE pin.

The page also makes the **test mode** of D-61 (test permit, capacitor below 60 V, and no real permit) and the
150 ms window, which starts when all consents are present, cannot be restarted by a flickering consent, and
re-opens only after every consent has been absent for at least 50 ms. Two buffers pass the controllers' hold clocks
to the discharge-path pumps on sheet 11, each on its own controller's supply. Open-drain outputs tell the main
controller whether the bridge rail is up and whether each path is held.

### 1.2 Sheet 12 — the shock sequencer (265 parts)

This page sits next to the output stage and is powered only while `BRIDGE_5V` exists, through a small regulator
with a 220 µF reserve (`BRIDGE_3V3`). Each time the rail comes up, every memory element starts cleared, so the
page allows **one shock sequence per rail cycle**.

It passes the main controller's four switch commands to the output stage only in the allowed order: high side on,
at least ~50 µs later the opposite low side (phase 1), a pause, then the other pair (phase 2). It refuses to start
unless the rail has been ready for 20 ms, the 150 ms window is open, the rail is above 4.5 V, nothing is faulty,
and either (shock mode) the patient relay has left its rest position for at least 10 ms and was energised less
than ~49 ms ago, or (test mode) the patient relay is at rest and the internal test-load relay has been closed for
at least 2 ms. The start is the **edge** of the first low-side command; from that instant a **latched 25 ms cap**
runs that nothing can restart or stretch. Over-current, current with no low side on, a collapsing rail or the cap
end the sequence through a fault latch. Every ending — planned or not — turns the switches off in a fixed order:
low sides at once, high sides about 25 µs later, the patient-relay coil about 1 ms later, and the hold on
`BRIDGE_5V` about 15 ms later (never later than ~40 ms after the start). The patient-relay coil is cut by hardware
~92 ms after it was first energised and can never be energised in test mode. Both controllers get open-drain
"sequence active", "sequence done" and (main) "ended by a fault" signals.

Every internal signal on both pages has a resistor that pulls it towards the state that blocks a shock, so a dead
or unsoldered gate output refuses rather than permits.

---

## 2. Part list

Datasheets were downloaded on **2026-11-27** from the manufacturer's own site (TI `www.ti.com/lit/ds/symlink/…`,
onsemi `www.onsemi.com/download/data-sheet/pdf/…`, Vishay `www.vishay.com/docs/…`). Nexperia refused automated
access (HTTP 403), so the diodes and the small MOSFET were chosen from onsemi, whose datasheets were read. KEMET's
site returned a certificate error, so the hold-up capacitor's datasheet was **not opened** (marked `Verify`).
Generic passives (thick-film resistors, X7R decoupling) have no single datasheet; value, tolerance and rating are
in each part's description. Precision resistors and C0G capacitors name an example series whose exact part codes
were **not checked** (§5).

| Part | Datasheet (revision read) | Pins checked against | Properties the safety argument uses (from the datasheet) |
|---|---|---|---|
| TPS22918DBVR (U901–U906) | SLVSD76C, Jul 2017 | §5 pin table, DBV | VIN 1–5.5 V; ON VIH 1 V / VIL 0.5 V, ON may be 0–5.5 V with no stated dependence on VIN, ON leakage ≤ 0.1 µA; "ON cannot be left floating" (8.3.1); QOD floating = disabled (Table 4); rise SR = 0.55·CT + 30 µs/V (Eq. 3); RON ≤ 79 mΩ to 105 °C; 2 A continuous, 2.5 A pulse |
| SN74LVC1G17DBVR | SCES351Y, Oct 2025 | DBV figure | VT+ 1.48–1.92 V, VT− 0.89–1.20 V, ΔVT ≥ 0.51 V at 3.0 V; 2.19–2.74 / 1.51–1.97 V at 4.5 V; Ioff ≤ ±10 µA; II ≤ ±5 µA; tpd ≤ 5.5 ns at 3.3 V |
| SN74LVC1G14DBVR | SCES218AA, Oct 2025 | DBV figure | Schmitt inverter, same family thresholds; Ioff |
| SN74LVC2G17DBVR / SN74LVC2G14DBVR | SCES381O / SCES200P, Nov 2026 | DBV figure (2G14 table has the GND/2A descriptions swapped; the figure is used) | dual Schmitt, Ioff |
| SN74LVC1G08 / 1G11 / 1G32 / 1G332 / 1G02 / 1G27 / 1G00 / 1G10 DBVR | SCES217AA, SCES487J, SCES219W, SCES489F, SCES213Y, SCES488F, SCES212AC, SCES486F | DBV figures | Ioff ≤ ±10 µA (partial power-down specified) |
| SN74LVC2G74DCTR | SCES203Q, Sep 2021 | §5 table, DCT | D flip-flop, PRE/CLR async; PRE and CLR both low → Q = Q̄ = H (nonstable); **no power-up state stated**; Ioff |
| SN74LVC1G07 / 2G07 DBVR | SCES296AG / SCES308M | DBV figures | open-drain, output tolerant to 5.5 V, Ioff |
| SN74LV1T08 / 1T32 / 1T02 / 1T04 DBVR | SCLS739F, SCLS741D, SCLS740D, SCLS738E | Fig. 5-1, DBV | at VCC 5 V: VIH ≤ 2.03 V, VIL ≥ 0.8 V (TTL levels for 3.3 V inputs); inputs 5.5 V tolerant with **negative clamp only** (§8.3.3); outputs have clamps to VCC; **Ioff is not specified** |
| TLV7031DBVR / TLV7041DBVR | SLVSE13J, Nov 2024 | Table 4-1 ("North-West" pinout) | VOS ≤ ±8 mV, VHYS 3–25 mV, IB 2 pA typ; inputs fault tolerant (no diode to VCC, may be driven with VCC = 0); **POR: TLV703x output held LOW, TLV704x output high-Z, while supply < 1.6 V**; tPD 3 µs |
| TPS3840PL30DBVR | SNVSB03E, May 2026 | Table 5-1 (PL column) | VIT− 3.0 V ±1.5 %, hysteresis 75–125 mV; push-pull active-low; VPOR ≤ 300 mV (output defined low from 0.3 V); tSTRT 100–350 µs; RCT 350–650 kΩ |
| TLV75533PDBVR | SBVS320D, Sep 2024 | Table 4-1 DBV | VOUT ±1.5 % (−40…125 °C); VDO ≤ 238 mV at 500 mA; active pull-down 120 Ω; reverse current warning (§7.1.4) |
| BAV99LT1G (D901–D904) | onsemi BAV99LT1/D Rev. 14, Sep 2026 | p. 1 pin diagram | pin 1 anode, pin 2 cathode, pin 3 common; IR ≤ 1 µA at 100 V; VF ≤ 715 mV at 1 mA |
| BAS116LT1G | onsemi BAS116LT1/D Rev. 14, Oct 2026 | p. 1, style 8 | pin 1 anode, pin 3 cathode, pin 2 NC; IR ≤ 5 nA at 75 V (25 °C), ≤ 80 nA at 150 °C; VF ≤ 0.9 V at 1 mA |
| 2N7002LT1G (Q901) | onsemi NDS7002A/D | SOT-23 pin list | 1 gate, 2 source, 3 drain; VGS(th) 1.0–2.5 V |
| SS14-E3/61T (D1201) | Vishay doc 88746, 23-Apr-2020 | SMA | 40 V, 1 A, VF ≤ 0.50 V at 1 A |
| T520D227M010ATE015 (C1201) | KEMET T52x series — **not opened** | — | 220 µF ±20 % 10 V polymer tantalum [UNVERIFIED] |

Full list with every reference (generated from the pin tables):

### 09_interlock — 170 parts, 88 nets (20 inter-sheet signals, 5 rails)

| Refs | Qty | Part | Notes |
|---|---:|---|---|
| U901–U906 | 6 | TPS22918DBVR | SW1, SW2a, SW2b, SW3, SW4, SW5 (contract §2) |
| U907, U908, U909 | 3 | SN74LVC1G17DBVR | `REQ_B` (3V3_MAIN); `HOLD_A_DRV` (3V3_MAIN); `HOLD_B_DRV` (3V3_SUP) |
| U910, U912 | 2 | SN74LVC2G14DBVR | permit / test-permit squarers (3V3_SUP) |
| U911, U913, U929 | 3 | TLV7031DBVR | `PERMIT_OK`, `TEST_OK` (3V3_SUP); `WIN_OPEN_I` (5V0_SYS) |
| U932 | 1 | TLV7041DBVR | `REARM_N` (open drain) |
| U914 | 1 | SN74LVC1G02DBVR | `HOLD_OK` |
| U915 | 1 | SN74LVC2G07DBVR | `HOLD_RB_A_N`, `HOLD_RB_B_N` |
| U916, U920–U925 | 7 | SN74LV1T08DBVR | `TM_A`, `VS_A`, `VTRANS_I`, `CR_A`, `CHARGE_I`, `SW5_ON`, `CH_A` |
| U917, U919 | 2 | SN74LV1T04DBVR | `PERMIT_N`, `TM_N` |
| U918, U927 | 2 | SN74LVC1G08DBVR | `TEST_MODE_I`, `CHAIN` |
| U926, U930 | 2 | SN74LV1T32DBVR | `CH_B`, `PERM_ANY` |
| U931 | 1 | SN74LV1T02DBVR | `ANY_N` |
| U928 | 1 | SN74LVC2G74DCTR | `WSTART` |
| Q901 | 1 | 2N7002LT1G | `BRIDGE_ARMED_N` |
| D901–D904 | 4 | BAV99LT1G | pump diodes |
| D905, D906 | 2 | BAS116LT1G | window / re-arm RC discharge |
| C901–C906 | 6 | 1 nF X7R 50 V 0603 | CT of U901–U906 |
| C907; C908–C913 | 1; 6 | 10 µF X7R 16 V 0805; 1 µF X7R 16 V 0603 | 5V0_SYS bulk; switch VIN/VOUT capacitors |
| C914, C917, C920, C923, C929, C932, C938–C953, C955–C957, C959 | 26 | 100 nF X7R 16 V 0603 | decoupling, one per IC |
| C915, C918, C921, C930 / C916, C919, C922, C931 | 4 / 4 | 3.9 nF / 390 pF C0G 50 V | two-pole low-pass |
| C924–C927, C933–C936 | 8 | 220 nF C0G 50 V 1210 | pump coupling, two in series per phase |
| C928, C937, C954, C958 | 4 | 100 nF C0G 50 V 1206 | detector hold; window RC; re-arm RC |
| R928–R932, R939–R943, R963–R965, R969–R971 | 16 | thin film 0.1 %, 25 ppm/K | detector bleeds (71.5 k) and thresholds (464 k, 100 k); window 1.47 M, 56.2 k, 100 k; re-arm 549 k, 56.2 k, 100 k |
| all other R9xx | 56 | thick film 1 %, 100 ppm/K | defaults, pulls (incl. the 1 MΩ backups R947, R948), filters, bleeds, series 1 k |

### 12_bridge_logic — 265 parts, 117 nets (23 inter-sheet signals, 3 rails)

| Refs | Qty | Part | Notes |
|---|---:|---|---|
| U1201 | 1 | TLV75533PDBVR | `BRIDGE_3V3` |
| U1203 | 1 | TPS3840PL30DBVR | `RST_N` |
| U1202, U1204, U1219, U1239, U1248, U1249, U1254, U1255 | 8 | TLV7031DBVR | `BR5_OK`, `RDY`, `RUN_D15M`, `XFER`, `CAPC_N`, `HOLDCAP_N`, `KWIN_N`, `KCAP_N` |
| U1205–U1208 | 4 | SN74LVC2G17DBVR | command / window / test-mode input buffers |
| U1209, U1210 | 2 | SN74LVC2G14DBVR | `OC_F`/`IDLE_RAW`; `K_AT_REST`/`K_LEFT` |
| U1211–U1215 | 5 | SN74LVC2G74DCTR | `ACT`, `P1D`, `DONE`, `FLT`, `KON` |
| U1216, U1250, U1258 | 3 | SN74LVC1G02DBVR | `RUN_L`, `CAP_F`, `K_EN_I` |
| U1217, U1218, U1226, U1228, U1241 | 5 | SN74LVC1G17DBVR | `RUN_D20`, `RUN_D1M`, `HA_ON50`, `HB_ON50`, `BTOK` |
| U1224, U1225, U1227, U1229 | 4 | SN74LVC1G14DBVR | `LA_OFF10`, `LB_OFF10`, `HA_OFF150`, `HB_OFF150` |
| U1220, U1238, U1240, U1244, U1256, U1259 | 6 | SN74LVC1G08DBVR | `RUN_H`, `XF_SRC`, `SHOCK_OK`, `START_C`, `K_RUN`, `BTEST_EN_I` |
| U1221–U1223, U1230–U1235, U1237, U1242, U1245, U1246 | 13 | SN74LVC1G11DBVR | `SEQ_HOLD_I`, `G_HA_I`, `G_HB_I`, `LB_LOCK`, `LA_LOCK`, `LB_EN`, `LA_EN`, `G_LB_I`, `G_LA_I`, `START_B`, `TEST_OK12`, `START_OK`, `IDLE_F` |
| U1236, U1247 | 2 | SN74LVC1G27DBVR | `HEALTH_A`, `SEQ_UV` |
| U1243, U1252 / U1251 / U1253 / U1257 | 2/1/1/1 | 1G32 / 1G332 / 1G00 / 1G10 | `MODE_OK`, `TRIG` / `TRIG_A` / `FLT_PRE_N` / `KA_N` |
| U1260, U1261, U1262 | 3 | SN74LVC2G07 ×2, SN74LVC1G07 | status to main, to supervisor; `BR_FAULT_N` |
| D1201 | 1 | SS14-E3/61T | hold-up isolation |
| D1202–D1215 | 14 | BAS116LT1G | timer RC fast charge/discharge |
| C1201 | 1 | 220 µF 10 V polymer tantalum | hold-up |
| C1202–C1204, C1281 | 4 | 1 µF / 10 µF X7R | LDO in/out, bulk |
| C1205; C1222; C1232, C1234; C1236, C1240; C1238, C1242 | 1;1;2;2;2 | 100 pF; 2.7 nF; 1.5 nF; 6.8 nF; 27 nF C0G | UV filter; `RUN_D20`; 10 µs; 50 µs; 150 µs |
| C1208, C1224, C1226, C1253, C1256, C1264, C1271 | 7 | 100 nF C0G 1206 (C1264, C1271 ±2 %) | timers |
| 61 × 100 nF X7R | 61 | decoupling, one per IC | |
| R1201–R1204, R1207–R1209, R1246–R1248, R1275–R1277, R1288–R1291, R1298, R1299, R6201, R6202 | 21 | thin film 0.1 %, 25 ppm/K | rail sense, timers and thresholds |
| all other R12xx / R62xx | 86 | thick film 1 % | defaults (contract), pulls, timer 10 k–36 k, series 1 k |

Reference blocks: 09 uses 901–999 (U901–U906 as the contract fixes); 12 uses 1201–1299 and the overflow block
6201–6208 for resistors (contract §2).

---

## 3. Arithmetic and tolerance bands

Tolerance assumptions used everywhere [EST, stated so a reviewer can change them]: thick-film resistors ±2 %
(1 % + 100 ppm/K × 45 K + 0.5 % life); thin-film ±0.35 % (0.1 % + 25 ppm/K × 45 K + 0.1 % life); C0G ±5.5 %
(±5 % + 30 ppm/K × 45 K) or ±2.2 % where ±2 % parts are specified; temperature −20…+65 °C (device range, §0 of the
design); LVC thresholds from the 3.0 V and 4.5 V rows of SCES351Y Table 5-1 expressed as fractions of VCC
(VT+ 0.487–0.640·VCC, VT− 0.297–0.438·VCC at 3.3 V; VT+ 0.482–0.609·VCC, VT− 0.336–0.438·VCC at 5 V); LVC input
leakage ±5 µA (worst-case II) applied to every Schmitt timer node (so those use R ≤ 36 kΩ); TLV7031 offset ±8 mV
and hysteresis up to 25 mV; comparator timers are ratiometric (RC charged from a logic output at VCC, threshold
from a divider of the same rail), so supply tolerance cancels.

### 3.1 Clock filters (hold drives and both detectors) [CALC + SIM-B]

Two poles, 4.7 kΩ/3.9 nF and 47 kΩ/390 pF (τ = 18.3 µs each, 8.7 kHz), 1 MΩ to GND at the Schmitt input.

| Case | Result | Requirement |
|---|---|---|
| 1 kHz and 5 kHz squares, duty 30/50/70 %, nominal and slow corner (R +2 %, C +5.5 %) | every edge passed (40 toggles in 20 periods); worst node swing at 5 kHz 30 %: peak 2.54 V vs VT+ max 2.11 V; 70 %: trough 0.60 V vs VT− min 0.98 V | 1–5 kHz pass |
| 100 kHz, duty 10/50/90 %, fast corner | 13–36 mV p-p at the Schmitt input, no toggles | ≥ 100 kHz rejected; LVC ΔVT ≥ 0.51 V |
| DC | static | none |
| one filter capacitor open (single pole left) | 100 kHz ripple ≈ 0.37 V p-p < 0.51 V: still rejected (thin margin) | — |

### 3.2 Consent detectors (permit, test permit) [SIM-B]

Two-phase pump (both clock edges pump), 470 Ω + 2 × 220 nF C0G in series per phase, BAV99, hold 100 nF C0G,
bleed 2 × 71.5 kΩ (35.75 kΩ), comparator threshold 0.301 × 3V3_SUP (2 × 464 k ∥ over 100 k).
Corners: 3V3_SUP 3.2/3.4 V, diode drop 0.45/0.75 V, capacitors ±5.5 %, resistors ±2 %, threshold ±1.2 %, offset
±8 mV, hysteresis 3/25 mV; each metric at its own worst corner.

| Metric | Worst case | Requirement (§B8.1) |
|---|---|---|
| Output valid after the first edges, 1 kHz, duty 30/50/70 % | ≤ 0.04 ms (+ ≈ 0.05 ms filter) | ≤ 3 ms |
| Stays valid after the last edge (1 kHz steady state) | ≥ 3.30 ms | ≥ 3 ms (3 × the 1 ms firmware gap) |
| One 0.8 ms extra gap in a 2 kHz clock | no drop | — |
| Invalid after the last edge (from 5 kHz) | ≤ 7.32 ms | ≤ 8 ms |
| A single transition of the clock line | **does** give a valid output for roughly 3–7 ms (peak 1.51 V vs 0.99 V threshold) | not required by §B8.1; consequence in §4.3 |
| One bleed resistor open | hold ≥ 7.1 ms, collapse ≤ 14.8 ms | degraded, bounded |
| One series coupling capacitor short (220 nF instead of 110 nF) | hold ≥ 5.5 ms, collapse ≤ 11.2 ms | degraded, bounded; DC still blocked |
| One threshold top resistor open (threshold 0.177 × 3V3_SUP) | hold ≥ 7.0 ms, collapse ≤ 11.8 ms; DC still gives nothing | degraded, bounded |

The ratio between "must hold" (3 ms) and "must collapse" (8 ms) is only 2.67, and the spread of the diode drop and
supply moves the hold level by ±17 %; the design meets both with ≈ 10 % and ≈ 9 % margin. **This is the tightest
analogue design on the two sheets and needs a SPICE check with real BAV99 curves (§8 doubt 1).**

### 3.3 Timers

| Timer | Circuit | Band [CALC] | Requirement / firmware | Verdict |
|---|---|---|---|---|
| Permit window (09) | `WSTART` latch → 1.47 MΩ / 100 nF C0G → TLV7031 at 0.640 × 5V0_SYS | **138.4–162.7 ms** (nominal 150.3) | start must happen inside it: firmware's latest start ≈ consent + 64 ms (F-4: armed ≤ 10, +20, coil ≤ 6 late (§3.6), relay ≤ 15, +13 proposed, +0.1) → 74 ms margin; F-8 (f) needs the rail gone by 200 ms: 162.7 + rail fall ≤ 16 ms (1 kΩ bleed, ≤ 10 µF on 13 assumed) = 179 ms | met |
| Re-arm (09) | `ANY_N` → 549 kΩ / 100 nF → TLV7041 at 0.640 | **51.7–60.8 ms** | ≥ 50 ms (§B8.2) | met; firmware must wait ≥ 61 ms → propose ≥ 70 ms (CI-1) |
| Rail ready `RDY` (12) | `RST_N` → 221 kΩ / 100 nF → TLV7031 at 0.640 | **20.6–24.7 ms** after reset release | ≥ 20 ms (§B13.1 step 4); earliest firmware start ≈ armed + 36 ms | met |
| Reset release (12) | TPS3840PL30, CT open | VIT+ ≤ 3.17 V, then 0.10–0.35 ms | before RDY | met |
| 25 ms cap (12) | `ACT` latch → 249 kΩ / 100 nF C0G ±2 % → TLV7031 at 0.6341 | **23.6–26.5 ms** (nominal 25.0) | 21.3–28.8 ms; must not cut a planned shock (≤ 20 ms + 0.5 ms gap) | met (≥ 3.1 ms margin) |
| SEQ_HOLD backstop (12) | same RC, second threshold 0.7996 | **37.7–42.5 ms** after start | must not end the hold before the planned end + 15 ms: latest planned DONE 20.5 ms + 16.2 = 36.7 ms | met (1.0 ms margin; if it ever cut first, the turn-off order has already finished) |
| Relay transferred `XFER` (12) | `K_LEFT·K_EN` → 110 kΩ / 100 nF → TLV7031 at 0.640 | **10.3–12.3 ms** | ≥ 10 ms (§B13.3 rule 5) | met; firmware's "+10 ms" (F-4) is too short → propose ≥ 13 ms (CI-1) |
| Coil start window `KWIN_N` (12) | `KON` latch → 953 kΩ / 100 nF C0G ±2 % → TLV7031 at 0.400 | **45.7–51.8 ms** after first K_EN | ≤ 60 ms (§B7.2) and ≥ firmware's 40.1 ms | met (see §3.7) |
| Coil cap `KCAP_N` (12) | same RC at 0.620 | **87.0–97.6 ms** | ≥ latest sequence end + coil delay = 51.8 + 26.5 + 1.63 = 80.0 ms; ≤ 100 ms (G17) | met (7 ms / 2.4 ms margins) |
| End → high sides off `RUN_D20` (12) | 10 kΩ / 2.7 nF C0G, LVC1G17 | **20.2–36.4 µs** | 20 µs (§B13.5 step 2), ≥ low-side off ≈ 1 µs + L_S decay 7.6 µs | met |
| End → coil off `RUN_D1M` (12) | 12 kΩ / 100 nF C0G, LVC1G17 | **0.89–1.63 ms** | 1 ms (step 3), after high sides off (≤ 0.04 + 0.02 ms) | met |
| End → hold released `RUN_D15M` (12) | 150 kΩ / 100 nF → TLV7031 at 0.372 (decay) | **13.5–16.2 ms** | 15 ms (step 4), after coil off | met: 36 µs < 0.89 ms < 13.5 ms at every corner |
| Leg lockout 10 µs `LA_OFF10`, `LB_OFF10` (12) | 10 kΩ / 1.5 nF C0G, LVC1G14 | **11.2–20.2 µs** (6.6–14.6 µs after a sub-µs glitch) | ≥ low-side turn-off ≈ 1 µs; firmware gives 10 µs | met |
| Lead `HA_ON50`, `HB_ON50` (12) | 10 kΩ / 6.8 nF C0G, LVC1G17 | **41.2–76.8 µs** (31–65 µs if re-enabled < 1 µs after a drop) | ≥ high-side turn-on worst 40 µs (§B6.3); < firmware lead 100 µs | met (restart case below 40 µs: §4.3) |
| Lockout 150 µs `HA_OFF150`, `HB_OFF150` (12) | 10 kΩ / 27 nF C0G, LVC1G14 | **202–364 µs** (118–263 µs after a high-side pulse shorter than ~1 ms) | ≥ 150 µs (5 × 30 µs turn-off); < 490 µs (phase 2 at T1 + 0.5 ms) | met for real phases; short-pulse case 118 µs = 3.9 × worst turn-off (§5 A-7) |
| Test relay closed `BTOK` (12) | 36 kΩ / 100 nF C0G, LVC1G17 | **2.07–4.39 ms** | ≥ 2 ms (§B13.3 rule 9) | met; firmware's "2 ms" (F-8.3) too short → propose ≥ 5 ms (CI-1) |

### 3.4 `BRIDGE_3V3` hold-up and inrush [CALC]

- Reservoir voltage at the start of a collapse ≥ `BRIDGE_5V` min − SS14 drop = (4.90 V `5V0_SYS` min (design §R-20)
  − 4 × 79 mΩ × 0.6 A) − 0.35 V = 4.71 − 0.35 = **4.36 V**.
- End of useful hold-up: LDO input 3.40 V (3.30 V out + ≤ 0.1 V dropout at 60 mA; SBVS320D gives 238 mV at
  500 mA).
- Effective capacitance ≥ 220 µF × 0.8 (tolerance) × 0.9 (temperature/ageing, EST) = 158 µF.
- Load: **60 mA** for sheets 12 + 13 (design estimate §B13.5; sheet 12's own logic is < 2 mA: 62 LVC/TLV parts at
  ≤ 10 µA static plus 33 µA per enabled 100 kΩ pull-down) — **WP-G must confirm sheet 13's share is ≤ 55 mA.**
- t = 158 µF × (4.36 − 3.40) V / 60 mA = **2.5 ms** ≥ 2 ms required.
- Inrush when the rail comes up: the last switch to close slews at 0.55 × 1000 + 30 = 580 µs/V → ≈ 2.5–2.9 ms for
  5 V; 221 µF × 5 V / 2.5 ms = **0.44 A** + loads, inside the TPS22918's 2 A.

### 3.5 Rail-collapse detector (4.5 V) [CALC]

`BR5_OK` = (`BRIDGE_5V` × 0.5) > (0.6831 × `BRIDGE_3V3`). Falling trip = (0.6831 × 3.3 × (1 ± 0.0007) × (1 ± 0.015)
± 8 mV − hysteresis/2) / (0.5 ± 0.0005) = **4.39–4.60 V** (nominal 4.51 V). Normal `BRIDGE_5V` ≥ 4.71 V: 0.11 V
margin. Sense filter 50 kΩ × 100 pF = 5 µs; comparator 3 µs: the low sides are released within ≈ 10 µs of the rail
crossing 4.5 V, while `BRIDGE_3V3` holds for ≥ 2.5 ms.

### 3.6 Logic levels and delays at the domain crossings

| Crossing | Levels | Result |
|---|---|---|
| 3.3 V signals (REQ_B, PERMIT_OK, TEST_OK, LOWV_OK, CHG_*, SEQ_HOLD) into 5V0_SYS logic | SN74LV1T at 5 V: VIH ≤ 2.03 V, VIL ≥ 0.8 V; 3.3 V sources give ≥ 2.8 V | valid; LVC at 5 V would need 3.5–3.85 V (not used for these) |
| 3.3 V and 5 V signals into TPS22918 ON | VIH 1 V, VIL 0.5 V | valid; 100 kΩ pull-down × 0.1 µA leakage = 10 mV |
| 3V3_MAIN commands into 12 (BRIDGE_3V3) | LVC2G17 at 3.3 V: VT+ ≤ 2.11 V | valid |
| 5V0_SYS outputs `WIN_OPEN`, `TEST_MODE` into 12 | LVC inputs 5.5 V tolerant at any VCC | valid; 1 kΩ / 100 kΩ = 99 % level |
| Over-current path `OC_DET_N` → `G_L*` low | 2G14 → 1G332 → 1G32 → 1G00 → 2G74 PRE→Q → 1G02 → 1G11 → 1G11 → 1 kΩ: 8 gates × ≤ 6.5 ns + 1 kΩ × ~10 pF | **≤ 65 ns** vs ≤ 0.5 µs (contract §10) |
| Charger enables off after `HOLD_OK` or `TEST_MODE` changes | 2 gates + 1 kΩ | ≤ 50 ns vs ≤ 1 ms (contract §10 WP-D with WP-E) |
| `HOLD_*_DRV` | LVC1G17: VOH ≥ 2.3 V at 24 mA, 2 kHz from the controller's own rail | WP-F's pump input (contract §10) |

**Coil versus rail-ready.** `K_EN` waits for `RDY` (added to §B13.3 rule 7, §6 DEP-7), so a `RELAY_CMD` issued at
`BRIDGE_ARMED_N` + 20 ms may be held up to ≈ 5.6 ms (RDY ≤ armed + 0.5 + 0.35 + 24.7 ms). With a relay that
operates within 15 ms (§B7.2) the firmware still sees `K_REST_N` high within 20.6 ms of `RELAY_CMD`, and starts by
`RELAY_CMD` + 33.7 ms ≤ 40 ms (F-4).

### 3.7 Why the coil numbers differ from "≤ 60 ms" and "90 ms"

With the spec's nominal values the margins do not close: a start at 60 ms plus the cap's top tolerance (28.8 ms)
plus the coil-off delay (1 ms) is 89.8 ms, against a 90 ms coil cap that itself has a tolerance. Using **one RC with
two thresholds** (ratio set by 0.1 % resistors only) and centring them at 48.7 ms and 92.2 ms gives: latest
sequence end + coil delay = 51.8 + 26.5 + 1.63 = 80.0 ms < 87.0 ms (earliest coil cut), and 97.6 ms < 100 ms
(G17 on-time rating). Both still satisfy the spec's words (start window ≤ 60 ms; coil cap inside the 100 ms rating).

---

## 4. Proof

### 4.1 Truth tables of the captured circuit against `interlock.rs` [SIM-B, exhaustive]

A script read the **captured** gate and switch network of `09_interlock.toml` and evaluated all 1,024 combinations
of `SHOCK_REQ`, `PERMIT_OK`, `TEST_OK`, `LOWV_OK`, `HOLD_A_HELD_N`, `HOLD_B_HELD_N`, `WIN_OPEN_I`, `SEQ_HOLD`,
`CHG_SESSION`, `CHG_RUN` (detector outputs and the window comparator taken as inputs; their timing is in §3).
Result: **0 mismatches** against the equations below, and for every row with `SEQ_HOLD` low, the window open and no
test permit, the captured circuit equals `interlock.rs` exactly.

Let *held* = both `HOLD_*_HELD_N` low (both opto LEDs lit), *tm* = `TEST_OK`·`LOWV_OK`·¬`PERMIT_OK`.

**`discharge_path_active(m, s) = ¬(m ∧ s)`** — realised on sheet 11 by the pumps that only *m* (`HOLD_A_DRV`) and
*s* (`HOLD_B_DRV`) can drive; 09 contributes the two buffers, each on its own controller's rail, and reads the result
back as *held*. Hardware is stricter: an open opto, wire or pull-up reads "not held" (*held* false) even if the
clocks run.

| m | s | stack A | stack B | *held* (`HOLD_OK`) | `discharge_path_active` |
|---|---|---|---|---|---|
| 0 | 0 | conducts | conducts | 0 | true |
| 0 | 1 | conducts | off | 0 | true |
| 1 | 0 | off | conducts | 0 | true |
| 1 | 1 | off | off | 1 | false |

**`bridge_may_fire(r, p, d) = r ∧ p ∧ ¬d`** — at the START (D-59):

| `SHOCK_REQ` | permit | test permit | `LOWV_OK` | *held* | window | `SEQ_HOLD` | `BRIDGE_5V` | interlock.rs |
|---|---|---|---|---|---|---|---|---|
| 0 | x | x | x | x | x | 0 | off | false |
| 1 | 0 | 0 | x | x | x | 0 | off | false |
| 1 | x | x | x | 0 | x | 0 | off | false |
| 1 | 1 | x | x | 1 | 0 | 0 | off | true — hardware stricter: window expired |
| 1 | 1 | x | x | 1 | 1 | 0 | **on** (SW2a) | true |
| 1 | 0 | 1 | 1 | 1 | 1 | 0 | **on** (SW2b, test mode) | false — **D-61 departure**: test mode at < 60 V only, patient relay and charger blocked |
| 1 | 0 | 1 | 0 | 1 | 1 | 0 | off | false |
| x | x | x | x | x | x | 1, ¬tm | **on** (SW5) | D-59: the started sequence completes |
| x | x | x | x | x | x | 1, tm | as rows above | SW5 is never honoured in test mode (§B13.2) |

**`charger_may_run(e, d) = e ∧ ¬d`**:

| `CHG_SESSION` / `CHG_RUN` | *held* | *tm* | `VTRANS_EN` / `CHARGE_EN` | interlock.rs |
|---|---|---|---|---|
| 0 | x | x | 0 | false |
| 1 | 0 | x | 0 | false |
| 1 | 1 | 1 | 0 | true — hardware stricter: no charging in test mode |
| 1 | 1 | 0 | **1** | true |

### 4.2 Sequencer (sheet 12): states and scenario results [SIM-B]

State: (`ACT`, `P1D`, `DONE`, `FLT`) — idle (0,0,0,0) → phase 1 (1,0,0,0, `G_LB` follows `CMD_LB`) → gap / phase 2
(1,1,0,0, `G_LA` follows `CMD_LA`) → done (1,1,1,0) or fault (x,x,x,1). Only `RST_N` (rail cycle) returns to idle.

| From | Event | Condition | To / action |
|---|---|---|---|
| idle | `CMD_LB` rising edge | `START_OK` = no OC, no idle current, no `FLT`, `RDY`, `WIN`, `BR5_OK`, lead and lockout, and (shock: `XFER`·`KWIN_N`) or (test: `TM`·relay at rest·`BTOK`) | phase 1, cap starts; else stays idle (refused) |
| idle | `CMD_LA` | any | ignored (`G_LA` needs `ACT` and `P1D`) |
| phase 1 | `G_LB` low for 9–20 µs | — | gap (`P1D`) |
| phase 1 | further `CMD_LB` edges | — | ignored (`ACT` holds itself; `G_LB` needs ¬`P1D` after phase 1) |
| gap | `CMD_LA` | `HB_ON50`, `HA_OFF150` | phase 2 (`G_LA`) |
| gap/phase 2 | `G_LA` low for 9–20 µs | — | done; turn-off order |
| any active | OC, idle current, rail < 4.5 V, cap expiry before done | `RDY` | fault latch; turn-off order; `BR_FAULT_N` low |
| any | rail cycle | — | idle |

The **captured** netlist of sheet 12 was run through a behavioural time-step model (§10): ideal gates, nominal
thresholds, RC nodes integrated, the patient relay modelled as "aux contact leaves rest 8 ms after `K_EN`". Results:

| Scenario | Observed (model) | Rule |
|---|---|---|
| Normal shock (F-4 order: phase 1 6 ms, gap 0.5 ms, phase 2 4 ms) | `RDY` 22.9 ms; `K_EN` at `RELAY_CMD`; `XFER` 11.2 ms after the contact moved; start at the `CMD_LB` edge; `G_HB` 15 µs after `G_LB` fell; `G_LA` at T1 + 0.5 ms; `DONE` 15 µs after `G_LA` fell; cap expiry after `DONE` (no fault); `SEQ_HOLD` released 14.8 ms after `DONE` | §B13.1 |
| Main frozen after the start, `CMD_LB`/`CMD_LA` toggling at 1 kHz, both high-side commands on | exactly one phase-1 pulse (0.3 ms); phase 2 blocked by the 150 µs lockout because `G_HA` stays on; ends at the cap (25.04 ms) with `BR_FAULT_N`, high sides off 27 µs later | R-4, rule 1 (see the note below this table) |
| `CMD_LB` stuck high after the start | phase 1 lasts until the cap at 25.04 ms; `G_LB` off, `G_HA` 27 µs later, `K_EN` 1.2 ms later, `SEQ_HOLD` 14.8 ms later | rule 4, §B13.5 |
| `RUN_L` forced high **and** `CMD_LB` stuck high (two faults) | `G_LB` still falls at the cap (25.04 ms) through `CAPC_N` in `LB_LOCK`; `FLT` and `BR_FAULT_N` set; `G_HA` stays enabled (no current without a low side) | Note 1 of §4.3 |
| `CMD_LA` before phase 1 | ignored, then a normal phase 1 | rule 1 |
| `CMD_LB` 20 µs after `CMD_HA` (lead not met) | refused | rule 3 |
| `CMD_LB` 5 ms after `RELAY_CMD` (relay not transferred 10 ms) | refused | rule 5 |
| `CMD_LB` 55 ms after `RELAY_CMD` | refused (start window); coil cut at 92.2 ms | rules 5, 7 |
| Window closed before the start | refused | rule 5 |
| Over-current 2 ms into phase 1 | `FLT` at once, `G_LB` off within the model's step, `G_HA` +27 µs, `K_EN` +1.2 ms, `SEQ_HOLD` +14.8 ms | rule 6 |
| `BRIDGE_5V` falls to 4.0 V 2 ms into phase 1 | `BR5_OK` low 3 µs later (filter), `FLT`, same order | §B13.5 |
| Shorted low side: idle current as soon as `G_HA` rises | `FLT` 20 µs into the lead (model's comparator delay), `G_HA` off 27 µs later, start refused, `BR_FAULT_N` | R-14 |
| Test sequence (test mode, `BTEST_CMD`, relay at rest) | full two-phase sequence, `K_EN` never high | rules 8, 9; §B14.2 |
| Test mode with `RELAY_CMD` high | `K_EN` stays low | rule 7 |

**A bug this model found, fixed before commit:** in the first capture the `SEQ_ACTIVE` latch was an ordinary
D flip-flop clocked by every `CMD_LB` edge, so a frozen timer's *next* edge (sampling `START_OK` = 0 because a high
side was now on) **cleared** `SEQ_ACTIVE` mid-sequence. The latch now holds itself (its Q̄ drives its own PRE̅), and
the frozen-timer scenario was re-run (§4.2 table above shows the corrected behaviour). The same review added
`CAPC_N` directly into both low-side lock gates (Note 1 of §4.3), `ACT` to the phase-2 enable (so a stuck `P1D` alone cannot open phase 2), `RDY` to `K_EN` (so the coil timer's
start latch cannot miss the first coil edge during reset), and an independent 40 ms backstop on `SEQ_HOLD`.

### 4.3 Per-part failure table

Columns: **Open** = the part (or its output) open, unsoldered or high-impedance; **Short** = the part shorted, or
its output stuck at the opposite state, or an internal pin-to-pin short where relevant; **No supply** = its supply
pin open or its rail absent. "Consent" means anything that could read as held, permitted, safe to charge or
sequence allowed. **Detected by** names the self-test of §F that finds it (F-7 hold sensors, F-8 (a)–(f) blocking
tests, F-8.3 bridge test), or "latent". Faults found only at a rescue are flagged ⚠.

Principles that cover many rows: every switch, gate output and comparator output that *permits* has a pull
toward "not permitted" (pull resistor or comparator power-on reset), so an open or unpowered driver blocks; every
pull resistor's short ties its node to the blocking state; every power path to `BRIDGE_5V` needs four independent
parts to agree before a sequence starts, so a single stuck-"yes" removes one consent and leaves three (fault 21
of §13), and each of those is visible to an F-8 blocking test **except** where stated.

#### Sheet 09

| Ref(s) | Function | Open | Short / stuck | No supply | Unsafe? / detected by |
|---|---|---|---|---|---|
| U901 SW1 | request switch | rail cannot come up | request no longer needed by the chain (one consent lost) | (VIN = 5V0_SYS) nothing on | short: F-8 (a); open: F-8 (e) |
| U902 SW2a | permit switch | permit path dead → **every real shock refused** | permit bypassed in the chain | — | short: F-8 (b); **open: latent ⚠ (CI-4)** — F-8 only ever uses the test permit |
| U903 SW2b | test-permit switch | test mode cannot raise the rail | permit bypassed | — | short: F-8 (b); open: F-8 (e) |
| U904 SW3 | held switch | no rail | held consent bypassed for the bridge (charger still gated) | — | short: F-8 (c)/(d); open: F-8 (e) |
| U905 SW4 | window switch | no rail | window bypassed (rail can stay while consents stay) | — | short: F-8 (f); open: F-8 (e) |
| U906 SW5 | sequence hold | a started pulse depends on consent again (R-1 behaviour; the 12 turn-off order still makes the end safe) | rail held without consent (fault 22) | — | short: F-8 rail-must-fall checks; **open: latent** (SW5 is disabled in test mode) |
| U901–U906 pin shorts | ON–GND (2–3) | — | switch off; driver overloaded | — | block, detected F-8 (e); QOD–VOUT (5–6) on U905 enables a 25 Ω discharge on `BRIDGE_5V` while SW5 holds (≈ 0.2 A, thermal) — listed, not unsafe |
| C901–C906 CT | rise time | **fast rise (≈ 135 µs): inrush ≈ 8 A into 221 µF**, above the 2.5 A pulse rating; may damage that switch (open or short) | slew pin grounded: behaviour [UNVERIFIED] | — | consequence is one switch open/short → as its row; listed |
| R901–R906 ON pull-downs | default off | none unless the driver is also dead (two faults) | ON tied low → switch off | — | short: block, F-8 (e) |
| C908–C913, R907–R910 | VIN/VOUT caps, bleeds | slower fall of an inner node / the rail (F-8 timing) | rail or node shorted → 5V0_SYS overloaded → fault 41 | — | not unsafe; detected (e) |
| Q901, R911, R912 | `BRIDGE_ARMED_N` | never "armed" → firmware reports Refused | always "armed" | — | readback only; F-8 (a)–(e) see either |
| U907, R913 | `REQ_B` buffer, default | (3V3_MAIN off, Ioff) request absent | request always → SW1 effectively shorted; re-arm never (window cannot reopen after use) | request absent | stuck: F-8 (a); open: F-8 (e) |
| U908 / U909 + filters R914–R921, C915–C919 | hold drives | no clock → that stack conducts → not held → no charge | static output → same; filter cap open: 100 kHz rejection thin (§3.1) | pump input pull-down on 11 → stack conducts | safe; detected at switch-on (cannot charge). A spontaneously oscillating buffer (not a known LVC failure mode) is the only path to a false hold: listed |
| U910–U911, D901–D902, C921–C929, R922–R932 | permit detector | no permit → **every real shock refused** ⚠ | comparator stuck high → permit always (SW2a bypass) | no permit | stuck-high: F-8 (b); **all no-permit faults latent until a rescue (CI-4)**; single bleed / threshold / coupling-cap faults: degraded timing only (§3.2); no single part gives a permit from a static input (DC blocked by two capacitors; threshold cannot reach 0 with one top resistor open) |
| U912–U913, D903–D904, C930–C938, R933–R944 | test-permit detector | no test permit → self-test fails | test permit always | none | detected by F-8 either way |
| U914, R904 | `HOLD_OK` NOR | output open → pull-down → not held → no charge, no rail | stuck high → **held always**: SW3 and both charger chains lose the held condition | not held | stuck high defeats D-25 latently; **detected by F-8 (c)/(d)** because the same `HOLD_OK` drives SW3 |
| R945 / R946 (10 k) | held pull-ups (contract) | 1 MΩ backup R947 / R948 keeps "not held" (DEP-9) | input tied high → not held → no charge | — | safe (single resistor open would otherwise float a CMOS input) |
| R947 / R948 (1 M backups) | backup | no effect | input tied high → not held | — | safe |
| U915 | readbacks | "not held" always | "held" always | released = not held | F-7 detects both |
| U916, R950 (`TM_A`) | TEST_OK·LOWV | no test mode | test mode whenever no permit → charging blocked | — | detected (cannot charge / F-8) |
| U917, R951 (`PERMIT_N`) | ¬PERMIT_OK | no test mode | test mode could coexist with a real permit | — | stuck high latent, needs a second fault (supervisor giving both) |
| U918, R903 (`TEST_MODE_I`) | test mode | no test mode | **test mode always**: SW2b on, charger blocked, SW5 disabled, patient relay blocked on 12 | — | cannot charge → detected at switch-on; no path to a patient |
| U919, R952 (`TM_N`) | ¬test mode | (pulled low) charger and SW5 blocked | charger and SW5 not blocked in test mode | — | stuck high latent; self-limiting (charging raises the voltage, `LOWV_OK` falls, test mode ends) |
| R949 (`LOWV_OK` default) | — | comparator off → no test mode | — | — | safe |
| U920–U923, R954–R959 | charger chains | that enable low → cannot charge | one chain stuck high: the other chain still requires `HOLD_OK` | — | stuck high **latent** (no readback, CI-5); open detected (cannot charge) |
| U924, R906, R960 | SW5 enable | SW5 never | SW5 always (fault 22) | — | as U906 |
| U925–U927, R961 | window start `CHAIN` | window never starts → never expires | same | — | F-8 (f) |
| U928, R962 | `WSTART` | pull-up → started → window closes | Q stuck low → never expires; high → never opens | started | F-8 (f) / (e) |
| R963, C954, D905 | window RC | never expires | C short: never expires; C open or D short: window ≈ 0 | — | F-8 (f) / (e) |
| R964, R965 | window threshold | top open: closed; bottom open: never expires | — | — | F-8 (e) / (f) |
| U929, R905 | window comparator | pull-down → closed | stuck high → never expires | POR low → closed | F-8 (f) / (e) |
| U930, U931, R967, R968 | consent-absent logic | `ANY_N` low → never re-arms | `ANY_N` high → re-armed while consent present → window never starts | — | F-8 (f) or next window refused |
| R969, C958, D906, R970, R971 | re-arm RC | never re-arms | C open: instant re-arm (a toggling consent could restart the window only when **all** consents drop together) | — | latent, low consequence |
| U932, R972 | re-arm comparator (open drain) | pull-up → never re-arms | stuck low → re-armed always → window never starts | high-Z → never re-arms | detected (e)/(f) |
| R953, R956, R959, R966 (1 k) | board-crossing series | far-side pull-down → off | no effect | — | safe; detected (cannot charge / F-8) |
| decoupling (26 × 100 nF, C907) | — | possible glitches [residual] | rail shorted → that rail lost (faults 38, 39, 41) | — | safe |

#### Sheet 12

| Ref(s) | Function | Open | Short / stuck | No supply | Unsafe? / detected by |
|---|---|---|---|---|---|
| D1201 | hold-up isolation | no `BRIDGE_3V3` → nothing runs | reservoir drained by sheet-13 loads on collapse → **turn-off order not guaranteed on a rail collapse** | — | short latent (two faults to matter: + mid-pulse collapse; = §13 fault 25 class) |
| C1201 | 220 µF | hold-up lost (latent, fault 25) | `BRIDGE_5V` overloaded → no rail | — | open latent; short detected |
| U1201, C1202–C1204 | LDO | no logic → block | 5 V on `BRIDGE_3V3` (all parts ≤ 6.5 V abs max) | — | block detected |
| U1202, R1201–R1205, C1205 | `BR5_OK` | pull-down → no start | stuck high: collapse not detected; order then relies on `RST_N` (all off together) | POR low → no start | stuck high latent ⚠ (fault 25 class) |
| U1203, R1206 | `RST_N` | pull-down → permanent reset → nothing | stuck high: no power-on clear; latch states random (analysed in §4.4) | output low | stuck high latent: worst case one refused attempt per rail cycle |
| U1204, R1207–R1210, C1208, D1202 | `RDY` | no start, no coil, no high side | stuck high: start possible before the 20 ms settle | POR low | stuck high latent, harmless (firmware starts at ≥ 35 ms) |
| R1211–R1220 | contract defaults | none (driver alive) | input held at default (no command / fault) | — | safe |
| U1205 (HA, HB) | high-side buffers | pull-downs → high sides never → no shock | one stuck high: that high side enabled whenever the rail is up (no current without a low side); phase 2 then blocked by the 150 µs lockout | no buffer output | F-8.3 (phase 2 missing) |
| U1206 (LA, LB) | low-side buffers | no shock | `LB` stuck high = frozen command: bounded by the cap (§B13.6 fault row, ≤ 177 J) | — | F-8.3 |
| U1207 (RLY, BTC) | relay buffers | no coil / no test relay | coil whenever the rail is up (≤ coil cap); test relay only in test mode | — | coil stuck: F-6 (`K_REST_N` not at rest) |
| U1208 (WIN, TM) | from 09 | no start / shock mode | `WIN` stuck high: 12's window check lost, SW4 on 09 still limits the rail; `TM` stuck high: coil blocked ⚠ | — | `TM` stuck high latent (CI-4) |
| U1209 (OC, idle) | fault inputs | pull-ups → fault → no shock | stuck "no fault": OC / idle latch lost | — | latent (§13 fault 34; F-8 cannot make 90 A) |
| U1210 (`K_REST_N`) | relay position | pull-downs: never transferred, never at rest → no start | `K_LEFT` stuck high: transfer check reduced to `K_EN` only | — | latent ⚠ (CI-4) |
| U1211 `ACT` | sequence active | pull-down → no sequence | stuck high: the cap runs from rail-up and ends everything ≤ 26.5 ms later, before the firmware's first command (≥ 35 ms) | — | detected F-8.3 (refused) |
| U1212 `P1D` | phase-1 done | no phase 1 / no phase 2 | stuck high: phase 1 blocked; phase 2 still needs `ACT` | — | F-8.3 |
| U1213 `DONE` | planned end | pull-up → done → nothing passes | stuck low: planned end not latched → ends at the cap as a fault | — | F-8.3 (`BR_FAULT_N` after a normal test) |
| U1214 `FLT` | fault latch | pull-up → fault → nothing | stuck low: OC/idle/UV/cap protections lost | — | **F-8.3 overlong-phase test** |
| U1215 `KON` | coil-timer start | pull-up → timer runs from rail-up | stuck low: start window never closes, coil cap never acts (coil still ends with the rail and at `RUN_D1M`) | — | latent ⚠ (CI-4) |
| U1216 `RUN_L` | end | pull-down → ended | stuck high: planned and fault endings no longer reach the gates through `RUN_L`; the cap still removes both low sides **directly** (`CAPC_N` in `LB_LOCK`/`LA_LOCK`), `SEQ_HOLD` ends at ≤ 42.5 ms (independent backstop) | — | latent, bounded by the cap; see §4.3 note 1 |
| U1217 / U1218 / U1219 + RCs | 20 µs / 1 ms / 15 ms | (pull-downs) high sides blocked / coil off / hold released | 20 µs: C open → high sides off together with low sides; C short → high sides never; 15 ms: stuck high → hold until the 40 ms backstop | POR / Ioff → blocked | latent order faults; block faults detected F-8.3 |
| U1220 `RUN_H` | high sides allowed | high sides never | stuck high: high sides stay after the end (low sides are off) | — | F-8.3 |
| U1221 `SEQ_HOLD` | hold | no hold (R-1 behaviour) | hold always while the rail is up (= fault 22) | — | latent / F-8 rail-must-fall in shock mode only ⚠ |
| U1222, U1223 | `G_HA`, `G_HB` | high side never | high side always (no current without a low side); 150 µs lockout blocks the opposite phase | — | F-8.3 |
| U1224–U1229 + R/C/D | lockouts and leads | (pull-downs) lockout kept / lead never | C short on an OFF timer → that lockout permanently satisfied; C open → lockout/lead ≈ 0; D short → ≈ 0 | — | **latent**: needs a firmware timing error as the second fault (§13 fault 19) — CI-6 proposes a test |
| U1230–U1233 | `LB_LOCK`, `LA_LOCK`, `LB_EN`, `LA_EN` | phase blocked | stuck high: lead/lockout or state check lost for that phase | — | latent (second fault needed); F-8.3 order checks |
| U1234, U1235 | `G_LB`, `G_LA` | phase never | **stuck high: low side on whenever the rail is up**: current starts with the high-side lead; OC latch ends it if ≥ 90 A, otherwise the main's `CMD_H*` falling (planned phase length) or the cap ends it | — | detected by F-8.3 (current during the lead), as §13 fault 18 |
| U1236–U1245 | start conditions | (pull-downs) start refused | one condition group bypassed (e.g. `MODE_OK` stuck high: start without relay transferred → current into the front ends, fault 29 class) | — | latent ⚠ for shock-mode conditions (CI-4); test-mode ones detected |
| U1239 + RC | `XFER` | refused ⚠ | transfer check lost | — | latent ⚠ |
| U1246, U1247, U1250–U1253, R1286–R1297 | fault triggers | (pull-ups / pull-down) fault → refused | one trigger stuck "no fault": that protection lost | — | latent (OC/idle/UV); cap trigger detected by F-8.3 |
| U1248, U1249, R1288–R1293, C1264, D1214 | cap and hold backstop | R open: never expires; C open: expires at once | C short: never expires | POR low → expired / hold released | never-expires detected by F-8.3 overlong-phase test |
| U1254, U1255, R1298, R6201–R6204, C1271, D1215 | coil window, coil cap | (pull-downs) window closed / coil cut ⚠ | stuck high: no window / no coil cap (coil still bounded by the rail, ≤ ~165 ms) | POR low → closed | latent ⚠ (CI-4) |
| U1256–U1258 | `K_EN` | coil never ⚠ | coil on whenever `RELAY_CMD`… or always (stuck high) | — | stuck high: F-6 (`K_REST_N`); open: latent ⚠ |
| U1259 | `BTEST_EN` | test relay never | test relay closed in shock mode → R1303 destroyed in a real shock | — | F-8.4 weld check sees a closed test relay |
| U1260–U1262 | status | released = "not active / not done / no fault" | stuck low = "active/done/fault" | released | F-8.3 status checks |
| all 1 k series (R1251–R1253, R1270, R1271, R6207, R6208) | board crossings | far-side pull-down → off | no effect | — | safe |
| all pull resistors | defaults | no effect unless the driver is also dead | node tied to the blocking state | — | safe by construction |
| decoupling (61 × 100 nF, C1281) | — | glitches [residual] | `BRIDGE_3V3` shorted → no sequence; `BRIDGE_5V` sagging → collapse path | — | safe |

**Note 1 — the ending path.** Every ending (planned or fault) reaches the low sides through `RUN_L` (U1216).
The first capture had `RUN_L` as a single point: stuck high, together with a frozen low-side command, it would have
let current run past the 25 ms cap. The cap comparator output `CAPC_N` now also enters both low-side lock gates
(`LB_LOCK`, `LA_LOCK`, U1230/U1231) directly, so the cap ends a low side through two independent paths (latch →
`RUN_L`, and comparator → lock gate). The behavioural model confirms it with `RUN_L` forced high and `CMD_LB` stuck
high: `G_LB` falls at the cap (§4.2). What remains single is the gate that drives each low-side enable
(`G_LB_I`/`G_LA_I` stuck high, same class as a shorted low-side switch, §13 fault 18) and the cap comparator itself
when it fails "never expires" (found by the F-8.3 overlong-phase test).

**Note 2 — open input pins.** An unsoldered *input* pin of a CMOS gate floats and can read either level. On 09 any
such fault affects one consent at most (one switch). On 12 an open input on one of the AND gates in the `G_L*` path
can remove one lockout or one start condition (the same latent class as the stuck-high rows above). No single open
input on 12 enables a low side by itself.

### 4.4 Power-up and power-down of every rail the logic sees

| Rail | Power-up | Power-down |
|---|---|---|
| **5V0_SYS** (09 logic, switches) | F-2 orders `EN_5V0` ≥ 10 ms before any consent. While it ramps: every ON pin is held low by its 100 kΩ (gate outputs undefined below ≈ 1.65 V are high-impedance or follow the input); consents absent → `REQ_B`, `PERMIT_OK`, `TEST_OK`, `SEQ_HOLD` low; `HOLD_OK` = NOR of two pulled-up inputs = low (or genuinely held); window comparator held low by POR until 1.6 V. No switch receives an "on" from an undefined gate because every chain needs all four. `WSTART` powers up undefined: either the window is open until the first chain edge or it is open ≤ 163 ms and then closed until re-arm — both are within the design intent | Switch inputs collapse → `BRIDGE_5V` collapses → sheet 12 runs the turn-off order on its hold-up (§13 fault 41). Charger enables: driven low as the gates lose supply, held low by the pull-downs on 10 |
| **3V3_MAIN** (U907, U908) | Ioff outputs high-impedance until powered; pull-downs hold `REQ_B` low; `HOLD_A_DRV` static | `REQ_B` falls (pull-down) → SW1 opens; hold clock stops → stack A conducts in ≤ 10 ms → `HOLD_OK` low → charger off |
| **3V3_SUP** (detectors, U909) | detector comparators held LOW by their POR below 1.6 V, then compare a hold node at 0 V | `PERMIT_OK`/`TEST_OK` low (POR, pull-downs); stack B conducts |
| **BRIDGE_5V** (12 input) | Rises over ≈ 2.5 ms; `BR5_OK` low until 4.5 V | Collapse: `BR5_OK` falls within ≈ 10 µs → `FLT` (if a sequence runs) → low sides off → high sides ≈ 25 µs → coil ≈ 1 ms; `BRIDGE_3V3` lasts ≥ 2.5 ms |
| **BRIDGE_3V3** (12 logic) | `RST_N` driven low from VDD = 0.3 V (TPS3840PL, VPOR ≤ 300 mV) until VDD > 3.1 V + ≤ 0.35 ms; every latch cleared; comparators held low by POR below 1.6 V (all of them mean "not ready / window closed / coil cut / expired / hold released"); `RDY` 20.6–24.7 ms later. `G_H*` need `RDY`; `G_L*` need `ACT` (cleared); `K_EN` needs `RDY`; `SEQ_HOLD` needs `ACT`; status outputs released. `BTEST_EN` = `BTEST_CMD`·`TEST_MODE` does not wait for `RDY` (test relay only, test mode only) | Below 3.0 V `RST_N` falls → latches clear, `RDY` falls at once (its RC empties through D1202) → every enable low; below 1.6 V comparators are forced low; LVC outputs go high-impedance and the pull-downs on 13/14 hold `G_*`, `K_EN`, `BTEST_EN` low. **No momentary enable**: every enable is an AND that includes a latch or `RDY`, all of which are cleared before the supply falls below the logic's operating minimum |

If the supervisor U1203 fails with `RST_N` stuck high, the five latches power up in random states. Worst
combinations: `ACT` = 1 → the cap ends everything ≤ 26.5 ms after rail-up (before any firmware command);
`DONE` or `FLT` = 1 → nothing passes; `KON` = 1 → the start window closes 46–52 ms after rail-up (an attempt may be
refused). None delivers energy; repeated refusals reveal the fault.

---

## 5. Unverified pins, ratings and assumptions

| # | Item | Status |
|---|---|---|
| A-1 | All pinouts in `aed_logic.toml` and every pin table | Copied from the datasheet tables named above; `PinoutVerified = NO` until a person checks them |
| A-2 | Stock KiCad `74xGxx` symbols have empty pin names | The build cannot cross-check names for those parts; datasheet names are in each pin row's comment. Pin numbers were checked against the DBV/DCT figures |
| A-3 | TPS22918 with VOUT > VIN (SW4 while SW5 holds the rail; the inner nodes then fed backwards) | Outside the recommended VOUT ≤ VIN, inside the 6 V absolute maximum; reverse-current behaviour not stated by TI [UNVERIFIED]. Consequence analysed as harmless (current flows towards 5V0_SYS only) |
| A-4 | TPS22918 ON pin driven high while its VIN is 0 (inner switches before the chain closes) | Allowed by the recommended conditions (VON 0–5.5 V, no relation to VIN stated); input structure not described [UNVERIFIED] |
| A-5 | TPS22918 behaviour with the CT pin shorted to GND | Not stated [UNVERIFIED] |
| A-6 | SN74LV1T family has no Ioff specification | Inputs have only a negative clamp (SCLS739F §8.3.3), so a 3.3 V input cannot back-feed an unpowered 5V0_SYS; outputs only drive pull-downs and inputs |
| A-7 | Lockout after a high-side pulse shorter than ≈ 1 ms is 118–263 µs, not ≥ 150 µs | The BAS116 drops ≈ 0.9 V on the fast charge; real phases are milliseconds long |
| A-8 | BAV99 forward drop at pump currents over −20…+65 °C taken as 0.45–0.75 V | Datasheet gives 715 mV max at 1 mA (25 °C); temperature coefficient assumed −2 mV/K [EST] |
| A-9 | 60 mA `BRIDGE_3V3` load (12 + 13) | Design estimate; WP-G to confirm |
| A-10 | `5V0_SYS` ≥ 4.90 V | Design §R-20 statement; WP-A to confirm (UV margin is 0.11 V) |
| A-11 | Sheet 13 puts ≤ 10 µF on `BRIDGE_5V` | Used for the F-8 (f) fall time; WP-G to confirm |
| A-12 | Part codes: KEMET T520D227M010ATE015 (datasheet not opened), C0G 100 nF ±2 % 1206, 220 nF C0G 1210, Panasonic ERA-3AEB thin-film values | Series exist [BK]; exact codes not checked; marked `Verify` where the timing depends on them |
| A-13 | SN74LVC2G74 power-up state | Not specified by TI; the design does not depend on it (cleared by `RST_N` on 12; both states acceptable on 09) |
| A-14 | Footprints | All stock KiCad footprints; DCT (SSOP-8 2.95 × 2.8 mm, 0.65 mm) chosen for the flip-flops |

---

## 6. Departures from the design specification

| # | Spec says | Captured | Why |
|---|---|---|---|
| DEP-1 | Hold-drive buffers "with 8 kHz two-pole low-pass" (contract §8), filter "before the pump" (§B5.4) | Low-pass at the **input** of the Schmitt buffer | A low-pass between buffer and pump would starve the 1.1 mA pump; before a Schmitt input it rejects ≥ 100 kHz completely (no edges at all) |
| DEP-2 | `CHG_SESSION`, `CHG_RUN` through "Ioff buffers on 3V3_MAIN" (§B8.1) | The first SN74LV1T08 of each charger chain is the buffer, on the reader's rail (R7) | A separate 3V3_MAIN buffer adds a part and a failure mode and protects nothing more |
| DEP-3 | Window started by "the rising edge of the consent chain (SW1–SW3 all on)" | Logical AND of the three switch enables (`CHAIN`) | The physical node behind SW3 is fed backwards through SW4 while SW5 holds the rail, which could mask or fake the edge |
| DEP-4 | "RC ... into a comparator whose output sets a latch" (window; same wording for the cap) | The latch is at the **start** (`WSTART`, `ACT`, `KON`) and drives the RC; the comparator output is the expiry | Equivalent and the form §B13.3 rule 4 itself uses for the cap: the RC can only rise until a power-on reset or re-arm, so the expiry cannot be undone; one latch fewer |
| DEP-5 | Detector: "diode pump into a hold capacitor, then a Schmitt buffer" | Two-phase pump (both edges), comparator with a ratiometric threshold | A Schmitt gate's threshold spread cannot meet hold ≥ 3 ms and collapse ≤ 8 ms together; pumping on both edges makes the result independent of duty cycle |
| DEP-6 | Start window "≤ 60 ms", coil cap "90 ms" | 45.7–51.8 ms and 87.0–97.6 ms on one RC | §3.7: the nominal values do not close with tolerances |
| DEP-7 | Rule 7: `K_EN` = `RELAY_CMD`·¬`TEST_MODE`·¬coil-cap·¬(done + 1 ms) | … ·`RDY` added | Otherwise a `RELAY_CMD` high at rail-up would raise `K_EN` during reset and the coil timer's start latch would miss the edge (coil uncapped) |
| DEP-8 | `SEQ_HOLD` released at end + 15 ms | … and never later than 37.7–42.5 ms after the start | Independent backstop for a stuck end-of-sequence element |
| DEP-9 | One default resistor per net (R8): 10 kΩ on `HOLD_*_HELD_N` | 10 kΩ **plus a 1 MΩ backup** each | The opto is open-collector and "dark" is normal; one open resistor must not float the input |
| DEP-10 | Rule 1 "CMD_LB low → state = gap"; rule 12 "CMD_LA low → DONE" | `P1D` and `DONE` latch 9–20 µs after the **gated** enables fall, and `DONE` only if phase 2 really passed | Uses what reached the output stage, not the command; a phase 2 that never passed ends at the cap as a fault |
| DEP-11 | RAIL_READY via the supervisor | TPS3840 for reset only; separate RC + comparator for the 20 ms | TPS3840 CT delay spreads 1.86:1 |
| DEP-12 | — | Every internal node pulled to its blocking state; 1 kΩ series on every board-crossing push-pull output; QOD unused, bleeds instead | Failure behaviour (§4.3) |
| DEP-13 | Rule 2 "`G_HA` blocked while `G_LA` is on or < 10 µs after" etc. | Lockouts measure the **gated** enables | Review R-23 |

---

## 7. Contract issues

| # | Problem | Smallest conservative choice made | Proposed change |
|---|---|---|---|
| **CI-1** | Firmware waits equal the hardware minimums: F-4 "+10 ms after `K_REST_N`" vs relay transferred ≥ 10 ms; F-4.6 "absent ≥ 50 ms" vs re-arm ≥ 50 ms; F-8.3 "`BTEST_CMD` → 2 ms" vs test relay ≥ 2 ms. With any tolerance the firmware would be refused | Hardware keeps the spec's minimums | F-4: wait **≥ 13 ms** after `K_REST_N` high; F-4.6: consent absent **≥ 70 ms** between attempts; F-8.3: **≥ 5 ms** after `BTEST_CMD`; F-4: `CMD_LB` not earlier than 30 ms after `BRIDGE_ARMED_N` (rail-ready ≤ 25.6 ms) |
| **CI-2** | §B7.2 start window ≤ 60 ms with a 90 ms coil cap does not close (§3.7) | 48.7 / 92.2 ms nominal | Record 45.7–51.8 ms and 87.0–97.6 ms in §B7.2 / §B13.3 |
| **CI-3** | `K_EN` rule 7 has no rail-ready term | `RDY` added (DEP-7) | Add to §B13.3 rule 7; the coil may start up to ≈ 6 ms after `RELAY_CMD` |
| **CI-4** | **The normal-permit path and the whole shock-mode start path have no self-test.** F-8 uses only the test permit, and test mode blocks `K_EN` and SW5. So these single faults are latent until the first rescue shock and then **block every shock**: permit detector or SW2a dead; coil enable, `KON`, `KWIN_N`, `KCAP_N`, `XFER`, `K_REST_N` buffer, `TM` buffer stuck high; SW5 / `SEQ_HOLD` dead (truncation risk returns) | None possible inside my sheets | Add to F-8: (g) capacitor < 50 V, **real permit** + `SHOCK_REQ`, no test permit, no bridge commands → `BRIDGE_ARMED_N` must go low (proves U910/U911/U902); (h) same, then `RELAY_CMD` with no bridge commands: `K_REST_N` must leave rest, and a `CMD_LB` edge without `CMD_HA` must be refused (proves the relay path without current). Both move the patient relay or raise the bridge rail with the capacitor below 50 V; this needs the owner's decision (D-61 extension) and a supervisor change |
| **CI-5** | The charger enables have no readback; a stuck-high gate in one chain is latent | Two independent chains (both must fail) | Optional: readback nets of `VTRANS_EN` and `CHARGE_EN` to 04 (new contract nets) |
| **CI-6** | Lockout and lead faults (§4.3) are latent (§13 fault 19) | — | Optional F-8 step at test voltage: command `CMD_HB` during `CMD_LB` and `CMD_LA` < 150 µs after `CMD_HA` low; `I_SHOCK` must stay zero |
| **CI-7** | Sheet 13's `BRIDGE_3V3` load and `BRIDGE_5V` capacitance are not in the contract | Designed for 60 mA and ≤ 10 µF | WP-G to state both; hold-up needs ≤ 55 mA on 13 |
| **CI-8** | Sheet 14's coil hold-up (≥ 30 ms if the supply fails) must not keep the coil on after `K_EN` falls, or the 92 ms coil cap and the end-of-sequence coil-off do nothing | — | WP-H: `K_EN` low must open the coil drive directly, after the reservoir |
| **CI-9** | `K_REST_N` has no pull on 12 (contract): with 3V3_MAIN off while sheet 12 is powered by SW5 it floats | Read through Ioff buffers with pull-downs on the buffer outputs only | Accept (the relay check is at the start, when the main is alive), or add a weak pull-up on 12 to `BRIDGE_3V3` (would make "contact open" read "left rest", which `XFER` already combines with `K_EN`) |
| **CI-10** | §B13.5 "idle current → high sides off under current" at +20 µs: in the idle (shorted low-side) case nothing turns the current off for 20–36 µs | Spec followed | Consider dropping `G_H*` immediately on an idle-current fault (D_FW takes the coil current either way) — WP-G to judge |
| **CI-11** | Contract §5.4 says `WIN_OPEN`/`TEST_MODE` domain "5V0_SYS logic" and pull-down "100 kΩ on 12"; both done, plus 1 kΩ series on 09 | — | Note the 1 kΩ in the contract |

No inter-sheet net was renamed, added or deleted.

---

## 8. Things a reviewer should check first (my own top five doubts)

1. **The consent detectors' timing** (§3.2) rests on a behavioural pump model with piecewise-linear diodes. The
   hold/collapse window has only ≈ 10 % margin each way. A SPICE run with the real BAV99 curves, 3V3_SUP ripple and
   the comparator's input behaviour near 1 V is needed before anyone relies on "≥ 3 ms / ≤ 8 ms".
2. **TPS22918 used outside its stated conditions**: VOUT above VIN on SW4 and the inner switches while SW5 holds
   the rail, and ON high while VIN is still 0. TI does not describe either. If the part misbehaves (for example
   turns on from VOUT, or latches), the series-switch argument changes. Ask TI or choose a switch with stated
   reverse blocking.
3. **The low-side enable gates themselves** (`G_LB_I`, `G_LA_I`, U1234/U1235) are the last single points: one
   stuck high is equivalent to a shorted low-side switch (§13 fault 18). That is accepted in the design (bridge test
   finds it daily), but it means a logic part, not only a 3.3 kV switch, sits in that row of the fault table.
4. **The self-test gap (CI-4)**: half of the shock path that only a rescue exercises. This is a design-level
   problem, not a capture problem, but it decides whether many "latent" rows in §4.3 are acceptable.
5. **Every timer uses C0G and thin-film parts whose exact codes I did not check** (A-12), and the ±2 % C0G 100 nF
   in 1206 for the cap and coil timers must exist and be stocked; with ±5 % parts the coil cap's upper bound moves
   to ≈ 101 ms, above the G17's 100 ms.

---

## 9. Totals

| Sheet | Parts | ICs | Nets | Inter-sheet signals | Rails |
|---|---:|---:|---:|---:|---:|
| 09_interlock | 170 | 32 (+ 1 MOSFET) | 88 | 20 | 5 (`GND`, `5V0_SYS`, `3V3_MAIN`, `3V3_SUP`, `BRIDGE_5V` made here) |
| 12_bridge_logic | 265 | 62 | 117 | 23 | 3 (`GND`, `BRIDGE_5V`, `BRIDGE_3V3` made here) |
| **Both** | **435** | **94** | **205** | 43 (incl. `WIN_OPEN`, `TEST_MODE`, `SEQ_HOLD` on both) | |

Custom symbols (aed_logic): TPS22918DBV, SN74LV1T08DBV, SN74LV1T32DBV, SN74LV1T02DBV, SN74LV1T04DBV,
TPS3840PL30DBV, BAV99LT1G, BAS116LT1G. The part count is well above the contract's ≈ 130 estimate, mostly because of
one decoupling capacitor per IC (26 + 61 = 87), one pull resistor per internal node (≈ 70) and discrete RC timers.

---

## 10. How the numbers were produced

Throwaway Python scripts (standard library only, not committed, not part of the build):
- **bands.py** — every RC timer's band from the corner combinations stated in §3.
- **detector.py / det2.py** — time-step model of the two-phase pump detector: ideal CMOS drive, piecewise-linear
  diodes (drop + 10 Ω), explicit Euler at 1 µs, comparator with offset and hysteresis.
- **lpf.py** — the two-pole filter plus Schmitt thresholds at 1, 5 and 100 kHz.
- **truth09.py** — reads `09_interlock.toml`, evaluates the captured gate and switch network for all 1,024 input
  combinations (§4.1).
- **sim12.py** — reads `12_bridge_logic.toml` and steps the captured netlist at 0.25 µs: gates ideal with nominal
  thresholds (Schmitt 0.55/0.37 VCC), flip-flops with asynchronous clear/preset, comparators ideal, RC nodes
  integrated from the captured R and C values, BAS116 as 0.5 V ideal clamps, the supervisor as a 0.35 ms reset,
  the relay as "aux contact moves 8 ms after `K_EN`" (§4.2).
These are behavioural models of the captured pin tables, not circuit simulations; the §16 [SIM] items of the design
remain open.

---

## 11. Revision 0.3

Applied 27 November 2026 from the change order `capture-notes/rev3-changes.md` (WP-D D-1…D-17), against
`docs/HARDWARE_DESIGN.md` v0.3 (§B8, §B13, §B14.5, §V3) and `contract.md` issue 3. Simulations: `sim/logic-relay`
(**LR**), `sim/bridge` (**BR**), `sim/rev3` (**R3**) and this package's own `sim/rev3-wpd` (**WPD**, new). [CALC] is
hand arithmetic from datasheet limits, [SIM] a simulated result with hand-built models; nothing has been built or
measured, and nothing here is safe or ready for use on a person or animal.

### 11.1 In plain language

- **Proof that the discharge paths are really held off now needs four "yes" signals instead of two.** Each path
  has two independent light sensors on sheet 11; sheet 09 reads all four on separate wires, accepts "held" only
  when all four agree, and reports each one to the main controller so the daily self-test can check every sensor
  on its own. A single sensor that fails into "held" can no longer fake a held path.
- **A fault current now switches the high-side switches off at once.** Before, an over-current or "current with no
  low side on" fault ended the sequence in the normal order, leaving the high side on for 20–36 µs, long enough to
  destroy it if a low-side switch had failed short. A new latch now cuts both high sides within about 50–60 ns of
  the fault signal. Normal endings keep the gentle order.
- **Each phase of the shock now has its own hardware time limit** (about 11 ms for phase 1, 8 ms for phase 2),
  besides the old 25 ms limit for the whole sequence. A controller that freezes with a switch command stuck on can
  now deliver at most about 171 J instead of 180 J at the normal charge (R3).
- **Smaller fixes**, each from the first simulations: the permit detectors forget a stopped clock fast enough; a
  bouncing relay contact cannot shorten the "relay has moved for 10 ms" check; a relay contact that was never
  seen at rest cannot count as "moved"; the test-relay timer no longer expires before the test relay can close;
  the 150 ms window can no longer fail to start when the two consents arrive within microseconds; a failed
  low-side gate-drive supply ends the sequence like a collapsing bridge rail.
- **What I found that the change order did not say:** (1) the small transistor that resets the relay timer has
  no guaranteed leakage at the low voltage it sees; if it leaks a few microamps when hot, every shock would be
  refused (safe direction, but a room-temperature self-test would not notice); (2) the margin by which a "not
  held" sensor line reads high is thin at the worst corner; (3) the phase-limit bands come out a few hundredths
  of a millisecond wider than the change order states; (4) seven new internal points had no default resistor and
  got one. None of these lets a shock start that should not, or keeps one going.

### 11.2 Changes applied

Sheet 09 (now 187 parts):

| # | Applied as | Designators |
|---|---|---|
| D-1 | Permit and test-permit hold bleeds 71.5 k → **64.9 k** 0.1 % thin film | R928, R929, R939, R940 |
| D-2 | `HOLD_A_HELD_N`, `HOLD_B_HELD_N` pull-ups 10 k → **47 k** (1 M backups R947, R948 kept) | R945, R946 |
| D-3 | New inputs `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N`: 47 k pull-ups, 1 M backups | R973, R974, R975, R976 |
| D-4 | U914 → `HOLDA_OK` = NOR(`HOLD_A_HELD_N`, `HOLD_A_HELD2_N`); U933 SN74LVC1G02 `HOLDB_OK`; U934 SN74LVC1G08 `HOLD_OK` = AND; 100 k pull-downs on `HOLDA_OK`, `HOLDB_OK` (R904 stays on `HOLD_OK`) | U914, U933, U934, R977, R978, C961, C962 |
| D-5 | SN74LVC2G07 open-drain readbacks `HOLD_RB_A2_N`, `HOLD_RB_B2_N` | U935, C963 |
| D-6 | `CHAIN` → 10 k → `CHAIN_RC` (10 nF C0G) → SN74LVC1G17 on 5V0_SYS → `CHAIN_D` → U928 CLK; 100 k pull-down on `CHAIN_D` | R979, C960, U936, R980, C964 |

Sheet 12 (now 328 parts):

| # | Applied as | Designators |
|---|---|---|
| D-7 | High-side kill latch: `OCI` = `OC_F` OR `IDLE_F`; `HSK_PRE_N` = NAND(`OCI`, `RDY`); SN74LVC1G74 (PRE̅ `HSK_PRE_N`, CLR̅ `RST_N`, D and CLK to GND, Q̄ = `HSK_N`); U1222/U1223 outputs renamed `G_HA_P`/`G_HB_P`; `G_HA_I` = `G_HA_P` AND `HSK_N`, `G_HB_I` likewise (R1252/R1253 and the lockout and lead timers stay on `G_H*_I`); R6209 `HSK_N`–GND | U1263–U1267, R6209, C1285–C1289; **added** R6229 (`OCI` up), R6230 (`HSK_PRE_N` down), R6231/R6232 (`G_HA_P`/`G_HB_P` down) |
| D-8 | `IDLE_FAST_N` → SN74LVC1G14 → `IDLE_FST`; `IDLE_ANY` = `IDLE_RAW` OR `IDLE_FST` at U1246 pin 1 and U1236 pin 3; R6210 (contract default), R6211 (`IDLE_ANY` up) | U1268, U1269, R6210, R6211, C1290, C1291; **added** R6233 (`IDLE_FST` up) |
| D-9 | `RAIL_OK` = `BR5_OK` AND `LSD_OK` at U1237 pin 6 and U1247 pin 1; R6212 (contract default on `LSD_OK`), R6213 | U1270, R6212, R6213, C1292 |
| D-10 | Phase-1 cap: `ACT` → 113 k → `P1_RC` (100 nF C0G ±2 %), BAS116 back to `ACT`; TLV7031 IN+ `P1_TH` (57.6 k / 100 k), IN− `P1_RC`, OUT `P1C_N`; 100 k down on `P1C_N` | U1271, R6214, C1282, D1216, R6215–R6217, C1293 |
| D-11 | Phase-2 cap: SN74LVC1G74 `P2S` (CLK `G_LA_I`, D and PRE̅ to `BRIDGE_3V3`, CLR̅ `RST_N`); R6218 `P2S` up; 76.8 k / 100 nF C0G ±2 %; BAS116; TLV7031 with 57.6 k / 100 k; 100 k down on `P2C_N` | U1272, U1273, R6218–R6222, C1283, D1217, C1294, C1295 |
| D-12 | `P1F` = NOR(`P1C_N`, `P1D`), `P2F` = NOR(`P2C_N`, `DONE`), `PCF` = OR; U1252 SN74LVC1G32 → **SN74LVC1G332**: `TRIG` = `TRIG_A` OR `CAP_F` OR `PCF`; R6223 `PCF` up | U1274–U1276, U1252, R6223, C1296–C1298; **added** R6234, R6235 (`P1F`, `P2F` up) |
| D-13 | 25 ms cap unchanged (R1288 249 k, R1289 31.6 k, R1290 26.1 k) | — |
| D-14 | `K_REST_N` → 10 k → `K_REST_F` (1 nF C0G) → U1210 pin 1; 2N7002LT1G (G `K_AT_REST`, S GND, D `XF_DIS`) + 10 Ω `XF_DIS`–`XF_RC` | R6224, C1284, Q1201, R6225 |
| D-15 | SN74LVC1G74 `K_WAS_REST` (CLK `K_EN_I`, D `K_AT_REST`, PRE̅ `BRIDGE_3V3`, CLR̅ `RST_N`); U1238 SN74LVC1G08 → **SN74LVC1G11**: `XF_SRC` = `K_LEFT` AND `K_EN_I` AND `K_WAS_REST`; R6226 down | U1277, U1238, R6226, C1299 |
| D-16 | R1280 36 k thick → **49.9 k** 0.1 % thin film; U1241 SN74LVC1G17 → **TLV7031** (IN+ `BT_RC`, IN− `BT_TH`, OUT `BTOK`); `BT_TH` from 56.2 k / 100 k thin film | R1280, U1241, R6227, R6228 |
| D-17 | Sheet notes: `K_EN` drives only the coil driver on 14; `BR_FAULT_N` also reports per-phase caps and `LSD_OK` | group notes `relay`, `status`, `faults` |

New custom symbol: `aed_logic:SN74LVC1G74DCT` (KiCad's 74xGxx library has no 74LVC1G74; pins from TI SCES794G
Figure 5-1, same numbering as the SN74LVC2G74 already used). New local nets on 09: `HOLDA_OK`, `HOLDB_OK`,
`CHAIN_RC`, `CHAIN_D`; on 12: `OCI`, `HSK_PRE_N`, `HSK_N`, `G_HA_P`, `G_HB_P`, `IDLE_FST`, `IDLE_ANY`, `RAIL_OK`,
`P1_RC`, `P1_TH`, `P1C_N`, `P2S`, `P2_RC`, `P2_TH`, `P2C_N`, `P1F`, `P2F`, `PCF`, `K_REST_F`, `XF_DIS`,
`K_WAS_REST`, `BT_TH`. Contract nets added: `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N`, `HOLD_RB_A2_N`, `HOLD_RB_B2_N` (09),
`IDLE_FAST_N`, `LSD_OK` (12). No contract net was renamed.

### 11.3 Timing across tolerance and −20…+70 °C

Bands as §3 (C0G ±2.2 % for ±2 % parts, else ±5.5 %; thin film ±0.35 %; thick film ±2 %; TLV7031 ±8 mV,
hysteresis 3–25 mV; BAS116 leakage up to its 80 nA 150 °C limit; LVC thresholds and ±5 µA input leakage from the
datasheets' 3.0/4.5 V rows). LVC propagation maxima are the datasheets' −40…85 °C figures.

| Circuit | Result | Requirement | Source |
|---|---|---|---|
| Permit / test-permit detector, 64.9 k bleeds | hold ≥ 4.07 ms, drop-out ≤ 7.50 ms | ≥ 3 ms, ≤ 8 ms | [SIM LR §1.2] |
| Phase-1 cap, from the `CMD_LB` start edge | **10.93–11.92 ms** (nominal 11.43); with the change order's spread 10.75–11.92 ms | > firmware's 10.0 ms longest phase 1 (≥ 0.75 ms margin) | [SIM WPD] |
| Phase-2 cap, from the first `G_LA_I` edge | **7.44–8.10 ms** (nominal 7.77); union 7.31–8.10 ms | > 6.7 ms (≥ 0.61 ms margin) | [SIM WPD] |
| 25 ms cap (unchanged) | 23.77–26.22 ms | backstop; T-9 | [SIM LR §2] |
| BTOK | 4.74–5.52 ms | ≥ 3.5 ms (test relay operates in 3 ms); firmware waits 7 ms | [SIM LR §1.3] |
| Window-start delay `CHAIN` → `CHAIN_D` | **59.6–104.1 µs** | > `REARM_N` release (WPD 9.7–10.4 µs; LR 15.7 µs): ≥ 3.8× | [SIM WPD] |
| Permit window with the delay | 131.9–161.6 ms (LR band + 0.06–0.10 ms) | F-8 (f) rail gone by 200 ms: 161.6 + 16 = 178 ms | [CALC] |
| `XFER` after the last contact release, bounce ≥ 15 µs | 11.17–11.31 ms (nominal parts) | ≥ 10 ms | [SIM WPD] |
| same, bounce 10 µs, VGS(th) 1.0 / 2.5 V | 10.37 / **8.98 ms** | ≥ 10 ms — **residual**, P-3 | [SIM WPD] |
| same, bounce ≤ 8 µs | ignored by the filter (no restart) | — | [SIM WPD] |
| `XFER` vs Q1201 off-state leakage 0 / 1 / 2 / 4 µA | 11.31 / 12.01 / 12.81 / 14.86 ms (nominal parts; late corner adds ≈ 0.9 ms) | ≤ 14 ms (firmware wait) → leakage must stay ≤ ≈ 2.5 µA | [SIM WPD], P-1 |
| Over-current → high sides: U1209 → U1263 → U1264 → U1265 (PRE̅→Q̄ ≤ 5.9 ns, SCES794G) → U1266 → 1 kΩ | 5 gates ≤ 33 ns + 1 kΩ × ≤ 20 pF [EST] to sheet 13's threshold ≈ 14 ns: **≤ 47 ns** | "at once" (change order ≈ 45 ns) | [CALC] |
| Fast idle → high sides: U1268 → U1269 → U1246 → U1263 → U1264 → U1265 → U1266 → 1 kΩ | 7 gates ≤ 46 ns + 14 ns: **≤ 60 ns** (≤ ≈ 90 ns to 90 %), on top of sheet 13's ≤ 0.1 µs | BR's 11–13 mJ assumed 0.1 µs in total — P-4 | [CALC] |
| Over-current → low sides (unchanged path) | ≤ 65 ns | ≤ 0.5 µs | [CALC, §3.6] |
| `LSD_OK` low → low sides | sheet 13 ≤ 5 µs + U1270 + `SEQ_UV` path (≤ 50 ns) | ≈ 5 µs (BR change 9) | [CALC] |
| `BR5_OK` collapse path | + one gate (U1270, ≤ 6.5 ns) | ≈ 10 µs | [CALC] |
| `HOLD_OK` → charger enables | + one gate (U934) | ≤ 1 ms | [CALC] |
| `HOLD_*_HELD*_N` "held" level | Darlington ≤ 1.0 V at 0.10 mA (contract) vs VIL 0.3 × 4.5 V = 1.35 V (SCES213Y) | — | [CALC] |
| `HOLD_*_HELD*_N` "not held" level | 4.90 V − (20 µA dark [change order, 85 °C] + 2 × 5 µA LVC II) × 44.9 kΩ = **3.55 V** vs VIH 0.7 × 4.90 = 3.43 V: **0.12 V** at the worst stack; ≈ 0.5 V at +70 °C with ≈ 10 µA dark [EST] | reads "not held" | [CALC], P-2 |
| `K_REST_N` filter | 10 kΩ × 1 nF = 10 µs; LVC leakage 5 µA × 10 kΩ = 50 mV | — | [CALC] |
| Q1201 discharge | peak ≤ 3.3 V / 10 Ω = 0.33 A (≤ 0.8 A pulsed, NDS7002A/D), 0.5 µJ | — | [CALC] |
| `BRIDGE_3V3` load added | 15 ICs ≤ 10 µA each + ≈ 8 × 33 µA in pulls ≈ 0.4 mA | 60 mA budget | [CALC] |

### 11.4 Revised failure table (new and changed parts)

Columns as §4.3. **Every row ends in "no shock possible", "degraded but bounded by another hardware limit", or
needs a second independent fault; none gives an unwanted or unending shock on its own.** T-n are the real-permit
self-test steps of design §B14.5; F-8 (x) the test-permit steps of §F-8.

Sheet 09:

| Ref(s) | Function | Open | Short / stuck | No supply | Result / detected by |
|---|---|---|---|---|---|
| R928/R929/R939/R940 | detector bleeds | one open: hold ≈ 2×, collapse ≤ ≈ 15 ms (bounded, §3.2) | node held low → no permit | — | no permit: T-2 (permit), F-8 (e) (test permit); one open: degraded timing, hidden (bounded by the 20 ms rail-ready delay) |
| R945/R946/R973/R974 | 47 k pull-ups | 1 M backup keeps "not held" at room temperature; **hot, a dark opto's leakage (≥ 5 µA) can pull that line to "held"** | line high → not held → no charge | — | open: hidden at room temperature; the **other sensor of the same path** still needs the real bias (second fault needed); short: no charge, switch-on and T-7 |
| R947/R948/R975/R976 | 1 M backups | no effect | line high → not held | — | safe; short: T-7 |
| U914 / U933 | `HOLDA_OK` / `HOLDB_OK` | open → R977/R978 → not held → no rail, no charge | stuck high → that path's two sensors bypassed; the other path still required | not held | stuck high: F-8 (c)/(d); open: switch-on |
| U934 | `HOLD_OK` | R904 → not held | stuck high → held always (SW3 and both charger chains) | not held | stuck high: F-8 (c)/(d); open: switch-on |
| R977 / R978 | defaults | no effect unless the gate is also dead | not held | — | safe; switch-on |
| U935 | second-sensor readbacks | "not held" always | "held" always | released | T-7 (pattern fails either way) |
| (sheet 11) one sensor reading "held" while dark | — | — | — | — | that path still needs its other sensor; T-7 detects (the line stays low when the hold is released) |
| R979, C960 | 100 µs delay | R open: `CHAIN_RC` floats → window may never start | C short: never starts; C open or R short: delay lost → race B returns (one attempt's window may not start) | — | never starts: F-8 (f); delay lost: hidden, bounded (the attempt still needs every consent; one sequence per rail cycle under the 25 ms and per-phase caps) |
| U936, R980 | `CHAIN_D` | R980 → never starts | stuck low or high: no edge → never starts | Ioff, R980 | F-8 (f) |
| C961–C964 | decoupling | glitches [residual] | 5V0_SYS shorted (fault 41) | — | safe |

Sheet 12:

| Ref(s) | Function | Open | Short / stuck | No supply | Result / detected by |
|---|---|---|---|---|---|
| U1263, R6229 | `OCI` | R6229 → `OCI` high → kill at `RDY` → no high side | stuck low: kill latch never set (v0.2 behaviour: high sides off 20–36 µs after a fault); stuck high: no high side | (common rail) | stuck low: **hidden** (design §B14.5 list), needs a shorted low side as second fault; others: F-8.3 / T-8 |
| U1264, R6230 | `HSK_PRE_N` | R6230 → kill | stuck high: never set (hidden, as above); stuck low: no high side | — | as U1263 |
| U1265, R6209 | kill latch | output open → R6209 → killed | Q̄ stuck high: never kills (hidden); stuck low: no high side; D/CLK pins open: a stray edge could clear it after a preset → `FLT`/`RUN_D20` still end the high sides 20–36 µs later (v0.2 behaviour) | — | "killed": F-8.3 / T-8; "never": hidden, second fault needed |
| U1266 / U1267, R6231 / R6232 | kill gates | `G_H*` held low by 13's 100 k through 1 k → no high side | **stuck high: that high side on whenever the rail is up** (as v0.2 U1222 stuck high): no current without a low side; the 150 µs lockout blocks the opposite phase | — | F-8.3 / T-8 (phase 2 missing) |
| U1268 / U1269, R6233 / R6211 | fast idle input | pull-ups → fault → start refused | stuck "no fault": fast path lost (5 A INA240 path remains, v0.2 behaviour) | — | "no fault": **hidden** (test current 0.3 A < 3 A); fault: refused, detected |
| R6210 | `IDLE_FAST_N` default | no effect | fault → refused | — | detected |
| U1270, R6212, R6213 | `RAIL_OK` | R6213 → no start | stuck high: rail-collapse **and** `LSD_OK` protection lost | — | stuck high: **hidden** (as `BR5_OK` stuck high, fault-25 class, second fault needed); low: refused |
| U1271, R6214–R6217, C1282, D1216 | phase-1 cap | R6214 open, C1282 short, R6216 open, comparator stuck high: **never expires** → 25 ms cap remains (≤ 180 J at 1,950 V, v0.2 bound) | C1282 open, R6214 short, D1216 short, R6215 open, comparator stuck low: expires at once → every start ends as a fault | POR low → expired | never: **T-8**; at once: F-8.3 / T-8 (`BR_FAULT_N`); D1216 open: slow RC reset — a rail re-cycle within ≈ 30 ms may shorten the next cap (fails early) |
| U1272, R6218 | `P2S` latch | open → R6218 → timer from rail-up → expired before `RDY` → every start refused | stuck low / CLK open: phase-2 cap lost → 25 ms cap remains; stuck high: as open | — | lost: **T-8**; refused: detected |
| U1273, R6219–R6222, C1283, D1217 | phase-2 cap | as phase 1 | as phase 1 | POR low | as phase 1 (T-8 second part) |
| U1274 / U1275 / U1276, R6234 / R6235 / R6223 | `P1F`, `P2F`, `PCF` | pull-ups → fault → refused | stuck low: that cap (or both) lost → 25 ms cap remains | — | lost: T-8; refused: detected |
| U1252 (now 1G332) | `TRIG` | R1296 → fault | stuck low: every fault trigger lost; the 25 ms cap still ends both low sides directly (`CAPC_N` in `LB_LOCK`/`LA_LOCK`, §4.3 Note 1) | — | T-8, T-9 |
| R6224 | `K_REST_N` filter | `K_REST_F` floats → contact state undefined on 12 | filter lost → bounce race returns (≥ 4.1 ms, LR) | — | open: shock-mode start usually refused (T-5); a wandering input could fake "moved" — needs the main to start without its own `K_REST_N` check (second fault), and a relay at rest isolates the bridge from the pads; short: hidden, degraded margin |
| C1284 | 1 nF | filter lost (hidden, degraded) | `K_REST_F` = 0 → "at rest" always → shock mode refused | — | short: T-5; open: hidden |
| Q1201 | `XF_RC` reset | drain/source open: bounce protection lost (hidden, degraded) | D–S short: `XFER` never → refused; G–S short: `K_AT_REST` low → `K_WAS_REST` = 0 → refused; **G–D short**: `K_AT_REST` drives `XF_RC` through 10 Ω against `XF_SRC` low through D1212 → `XFER` undefined at rest, never after a real transfer | — | D–S, G–S, G–D: real attempts refused, T-5; a start with the relay at rest would need a firmware fault as well and reaches no pads; **leakage**: P-1 |
| R6225 | 10 Ω | bounce protection lost (hidden) | peak current set by RDS(on) | — | hidden, degraded |
| U1277, R6226 | `K_WAS_REST` | R6226 → never → shock refused | stuck high: WP-H C-6 protection lost (an open aux contact can look like a transfer — second fault needed, and the main checks `K_REST_N` itself) | — | low: T-5; high: **hidden** |
| U1238 (now 1G11) | `XF_SRC` | R1274 → refused | stuck high: transfer check lost (as v0.2) | — | high: **hidden** |
| U1241 (now TLV7031), R1280, R6227, R6228, C1256 | BTOK | never → test refused | stuck high or R6227 open: test-relay check lost (test mode only, ≤ 0.18 J, patient relay at rest) | POR low | refused: F-8.3; lost: hidden, harmless |
| C1285–C1299 | decoupling | glitches [residual] | `BRIDGE_3V3` shorted → no sequence | — | safe |

**Single-path note (new).** The per-phase caps reach the low sides only through `TRIG` → `FLT` → `RUN_L`; they
have no direct path into the lock gates as `CAPC_N` has. If `RUN_L` is stuck high **and** the main is frozen with a
command high (two faults), the sequence runs to the 25 ms cap, which keeps its two paths: the v0.2 bound (≤ 180 J
at 1,950 V, BR Q3b). A direct path would need `P1C_N`/`P2C_N` masked by `P1D`/`DONE` inside the lock gates; not
done (not in the change order), listed as P-6.

### 11.5 Default state of every new node

| Node | All supplies off | Rail up, `RST_N` low (sheet-12 reset) | Main or supervisor held in reset |
|---|---|---|---|
| `HOLD_*_HELD2_N` (09) | 0 V (pull-ups to a dead rail), but every reader is unpowered: `HOLDx_OK`/`HOLD_OK` held low by R977/R978/R904; readbacks released (Ioff) = "not held" on 04 | (09 has no reset) high = not held until sheet 11's optos are lit | that controller's hold clock stops → its path conducts within ≤ 25 ms → its two lines high → `HOLD_OK` low |
| `HOLDA_OK`, `HOLDB_OK`, `HOLD_OK` | low (pull-downs) | low unless all four lit | low |
| `HOLD_RB_A2_N`, `HOLD_RB_B2_N` | released → high on 04 = not held | follow the sensors | follow the sensors |
| `CHAIN_RC`, `CHAIN_D` | low (R980) | low until `CHAIN` | `CHAIN` low (no request or no permit) → low |
| `OCI` | 0 V | `OC_F`/`IDLE_F` pulled up (with sheet 13 unpowered they read fault) → `OCI` high, but `RDY` low keeps `HSK_PRE_N` high | 12 does not read the controllers here |
| `HSK_PRE_N` | 0 V | high (`RDY` low) | — |
| `HSK_N` | low (R6209) = killed | **high** (CLR̅ low → Q̄ high = not killed); harmless: `RUN_H` = `RUN_D20`·`RDY` is low | — |
| `G_HA_P`, `G_HB_P` | low | low (`RUN_H` low) | main in reset: `CMD_H*` pulled low on 12 → low |
| `IDLE_FST`, `IDLE_ANY` | 0 V | high = fault until sheet 13's fast comparator is up (start refused meanwhile) | — |
| `RAIL_OK` | low | low until `BR5_OK` and `LSD_OK` | — |
| `P1_RC`, `P2_RC` | 0 V | 0 V (`ACT`, `P2S` cleared) | — |
| `P1C_N`, `P2C_N` | low | low (POR below 1.6 V), then high (not expired) | — |
| `P1F`, `P2F`, `PCF` | 0 V | may read "fault" while the comparators are in POR; every fault trigger is gated by `RDY` (U1253), so no fault is latched | — |
| `P2S`, `K_WAS_REST` | low (P2S: 0 V; R6226) | low (cleared) | low (no `G_LA_I` / `K_EN` edge) |
| `K_REST_F` | 0 V | follows `K_REST_N`; with 3V3_MAIN off `K_REST_N` floats (CI-9) | main in reset: `RELAY_CMD` low → `K_EN` low |
| `XF_DIS` | — | Q1201 on while at rest → `XF_RC` 0 V | — |
| `BT_TH`, `BTOK` | 0 V | `BTOK` low (POR, `BT_RC` 0 V) | `BTEST_CMD` low → low |

**A controller held in reset during a running sequence.** Main: its commands fall (100 k pull-downs on 12) → the
phase ends at once, or if phase 2 never comes the 25 ms cap ends the sequence as a fault; a main **frozen** with a
command high is what the per-phase caps bound (10.75–11.92 / 7.31–8.10 ms). Supervisor: the permit disappears, SW5
holds the rail until the sequence ends (D-59); the new circuits do not depend on the supervisor.

### 11.6 CI-4 (faults hidden until the first rescue): what the v0.3 self-test reveals

| CI-4 item | Revealed by | Still hidden |
|---|---|---|
| Permit detector dead (U910/U911, D901/D902, C921–C929), SW2a (U902) open | **T-2** | detector drifted within its band (timing only) |
| SW1 shorted under the real permit | **T-1** | — |
| Coil enable path (U1256–U1258), `RDY` gate, `TM` buffer stuck high, coil driver, relay, aux contact, `K_WAS_REST` stuck low | **T-3** | — |
| `KWIN_N` / `KCAP_N` stuck refusing | **T-3** | `KWIN_N` stuck **high** (start window never closes; only late starts affected) |
| `KCAP_N` never expires, `KON` stuck low | **T-3b** (weekly) | — |
| Lead gate (`HA_ON50`, `LB_LOCK`) stuck in shock mode | **T-4** | other lockout/lead timers (CI-6, §13 fault 19) |
| SW5 / `SEQ_HOLD` dead, `XFER`/`SHOCK_OK`/`MODE_OK` stuck low, end-of-sequence coil-off | **T-5** (weekly, docked) | — |
| `XFER`, `SHOCK_OK`, `MODE_OK`, `XF_SRC`, `K_LEFT` stuck **high** (transfer check lost) | — | **hidden** (the main's own `K_REST_N` check remains: second fault needed) |
| `K_WAS_REST` stuck high | — | hidden (second fault needed) |
| Each of the four held sensors (shorted Darlington, fault 9) | **T-7** | a sensor's 47 k pull-up open (room-temperature test cannot see it; the other sensor covers) |
| Per-phase caps never expiring | **T-8** | — |
| 25 ms cap never expiring | **T-9** (weekly) | — |
| OC 80 A, idle 3 A and 5 A comparators "no trip", kill latch stuck clear, `RAIL_OK`/`BR5_OK`/`LSD_OK` stuck high | — | **hidden** (design §B14.5 list; test current ≈ 0.3 A) |
| Q1201 open or leaky, bounce-filter parts open | — | hidden: open = degraded bounce margin; leakage = hot-only refusal (P-1) |

### 11.7 Problems with the change order (for the design authority)

- **P-1 Q1201 leakage (D-14).** NDS7002A/D Rev. 11 guarantees IDSS only at 60 V: ≤ 1 µA at 25 °C, ≤ 0.5 mA at
  125 °C; nothing at ≤ 3.3 V and +70 °C. `XFER`'s charging current at threshold is only ≈ 11 µA (1.19 V / 110 kΩ),
  so ≈ 2.5 µA (slow corner) to 4 µA (nominal) pushes `XFER` past the firmware's 14 ms wait, and ≈ 11 µA stops it
  [SIM WPD]. Direction: refusal (no shock), only when hot, so a room-temperature T-5 does not see it. Captured as
  ordered, with a `Verify` field on Q1201. Options: a low-voltage leakage figure from onsemi; a part that states
  one; or a smaller `XFER` resistor with a larger capacitor (more charging current, same time constant).
- **P-2 "Not held" margin (D-2/D-3).** 47 k ∥ 1 M with the change order's 20 µA dark current (85 °C) and two LVC
  inputs at ±5 µA leaves 3.55 V against VIH 3.43 V at `5V0_SYS` = 4.90 V: 0.12 V. One sensor could then read
  "held" while dark; the other sensor of the path still decides (not a single-fault hazard), but the TLP187's dark
  current at +70 °C should be confirmed (WP-F; I did not open its datasheet).
- **P-3 Bounce at the filter's time constant (D-14).** A ≈ 10 µs re-touch with a 2.5 V-threshold 2N7002 lets
  `XFER` rise 8.98 ms after it (rule ≥ 10 ms); ≥ 15 µs is fine [SIM WPD]. Consequence: a start on still-settling
  contacts (contact wear), not an unwanted shock. A MOSFET with VGS(th) ≤ 1.5 V, or a 20 µs filter, would close it.
- **P-4 Kill-path delay versus the bridge simulation (D-7/D-8).** The fast idle path is 7 gates on sheet 12
  (≤ 60 ns to sheet 13's input threshold [CALC]) on top of sheet 13's ≤ 0.1 µs; BR's 11–13 mJ assumed 0.1 µs in
  total. Re-run BR Q1 with the drawn chain (second round S-1). The over-current path matches the change order.
- **P-5 Per-phase cap bands (D-10/D-11).** Simulated directly: 10.93–11.92 and 7.44–8.10 ms, against the change
  order's 10.75–11.85 and 7.31–8.06 ms. The contract's `BR_FAULT_N` row, design §B13.3/§B13.4 and §B14.5 T-8 should
  carry the union 10.75–11.92 / 7.31–8.10 ms; R3 §4 (run at 11.85 / 8.05 ms) should be repeated at the new tops
  (expected well under 1 J more).
- **P-6 Defaults and parts the change order did not name.** No default resistor was given for `OCI`, `HSK_PRE_N`,
  `G_HA_P`, `G_HB_P`, `IDLE_FST`, `P1F`, `P2F` (contrary to its own "every new internal node" rule): added as
  R6229–R6235, each towards "fault / no high side". Decoupling C961–C964 and C1285–C1299 numbered by me. The
  SN74LVC1G74 needed a new library symbol. The per-phase caps have one path to the low sides (single-path note).
- **P-7 Single-ended nets at this revision.** `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N` (driver on 11), `HOLD_RB_A2_N`,
  `HOLD_RB_B2_N` (reader on 04), `IDLE_FAST_N`, `LSD_OK` (driver on 13) appear only on my sheets until WP-F, WP-B
  and WP-G land issue 3; names kept exactly. The two readback labels needed an `isolated_pin_label` ERC acceptance
  on 09 without `until_sheet` (04 is already captured); remove it once 04 carries them.
- **P-8 Shared generated PDFs.** `pdf/aed-main.pdf` and `pdf/aed-main-captured-sheets.pdf` contain every sheet and
  were regenerated by my build; they should be regenerated again after all packages are merged.

### 11.8 Datasheets consulted for revision 0.3

Opened 27 November 2026 from the maker's site:

| Part | Document | Used for |
|---|---|---|
| SN74LVC1G74 | TI SCES794G (Oct 2009, rev. Sep 2021) | DCT pinout (Figure 5-1); function table (PRE̅ and CLR̅ both low → Q = Q̄ = H, nonstable); II ±5 µA; Ioff; PRE/CLR → Q tpd ≤ 5.9 ns at 3.3 V; no power-up state stated |
| 2N7002 (2N7002LT1G) | onsemi NDS7002A/D Rev. 11 (Mar 2025) | VGS(th) 1.0–2.5 V at 250 µA; IDSS ≤ 1 µA (60 V, 25 °C), ≤ 0.5 mA (60 V, 125 °C); RDS(on) ≤ 7.5 Ω at 5 V; ID pulsed 800 mA |
| SN74LVC1G02 | TI SCES213Y (rev. Aug 2026) | VIH 0.7 × VCC, VIL 0.3 × VCC at 4.5–5.5 V; II ±5 µA; Ioff |
| SN74LVC1G08 / 1G32 / 1G00 / 1G11 / 1G332 / 1G14 / 1G17 / 2G07; TLV7031 | TI SCES217AA, SCES219W, SCES212AC, SCES487J, SCES489F, SCES218AA, SCES351Y, SCES308M; SLVSE13J | current revision confirmed; pinouts as already captured on these sheets (DBV figures, §2) |
| BAS116LT1G | onsemi BAS116LT1/D Rev. 14 | as §2 (not re-opened) |

Not opened: Toshiba TLP187 (WP-F's opto; the 20 µA dark current is the change order's figure, **unverified by
me**); Murata C0G and Panasonic ERA thin-film part codes (series pages only, as A-12).

### 11.9 Still needs simulation or measurement

1. BR Q1 shorted-low-side run with the drawn kill-path delay and sheet 13's real fast comparator (P-4, S-1).
2. R3 §4 per-phase energy at the band tops 11.92 / 8.10 ms (P-5).
3. 2N7002 drain leakage at ≤ 3.3 V, +70 °C (P-1); bounce trains with real relay data (P-3, O-13).
4. TLP187 dark current at +70 °C into 47 k ∥ 1 M with two LVC loads (P-2).
5. Noise immunity of the 3 A fast idle path (sheet 13): one glitch ≥ 5 ns with no low side enabled now latches
   `FLT` and the kill latch and ends the shock (availability, not safety; WP-G). The shunt is not in either
   freewheel loop (design §B6.5), so no idle current is expected after a planned phase end.
6. A behavioural re-run of the whole sheet-12 netlist (the §10 `sim12.py` method) with the new latches; done here
   only as a connectivity check (every new net has its driver, loads and default) and by reasoning.
7. Everything still open from §8 (detector curves, TPS22918 reverse conditions).

### 11.10 Totals and checks after revision 0.3

| Sheet | Parts | Change |
|---|---:|---|
| 09_interlock | 187 | +17 |
| 12_bridge_logic | 328 | +63 |

`bash tools/build.sh` → BUILD OK (ERC 0 errors, 0 unaccepted warnings; netlist round trip 751 nets, 3,895 pin
connections match the specs). `python3 tools/xcheck.py`: the six issue-3 nets of P-7 are single-sheet, as
expected; nothing renamed.
