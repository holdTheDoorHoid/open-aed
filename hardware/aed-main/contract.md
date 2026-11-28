# aed-main capture — shared net contract (issue 3)

**NOT FOR CONSTRUCTION — unreviewed design.** Design source: [`docs/HARDWARE_DESIGN.md`](../../docs/HARDWARE_DESIGN.md)
version 0.3 (§Bn, §F, §V3 below refer to it). Where this file and the design document disagree on a **name**, on
**which sheet holds a part** or on **which sheet fits a resistor**, this file wins; where they disagree on the
**design**, capture the conservative reading and log a question. Firmware contract: `hal.rs`, `interlock.rs`,
decisions D-59 to D-67 (D-62 to D-67 provisional) and the proposed HAL additions in design §F-11 (table §7).
The change order that turns the issue-2 capture into issue 3 is `capture-notes/rev3-changes.md`.

Nothing in this file is an instruction to build or power anything (CLAUDE.md rule 2, SAFETY.md §2a).

## 0. Change log — read this first

**Issue 3 (27 Nov 2026) amends issue 2; it does not replace it.** Everything captured from issue 2 stays valid
except where this table or `capture-notes/rev3-changes.md` says otherwise. Sources: the 25 integration issues
(`capture-notes/00-integration.md`, I-1…I-25), the three simulation studies (`sim/bridge`, `sim/charger-storage`,
`sim/logic-relay`), the revision-0.3 check runs (`sim/rev3`) and decisions D-62 to D-67 (provisional).

**New inter-sheet nets (10)**

| Net | Driver → loads | Why | Section |
|---|---|---|---|
| `VBAT_MID` | 01 (pack mid-point, J101 pin 3) → 02 (BQ25887 MID via 300 Ω, CBSET via 100 Ω) | charger regulates each cell (I-1) | §5.2 |
| `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N` | 11 (second opto per stack) → 09 | two independent "held" sensors per path (I-4) | §5.4 |
| `HOLD_RB_A2_N`, `HOLD_RB_B2_N` | 09 (open drain) → 04 | the self-test proves each of the four sensors separately | §5.4 |
| `OVP_RST` | 05 → 08 | resets the now-latching over-voltage comparator; **rising edge** only (I-22) | §5.4 |
| `OVP_TEST` | 05 → 08 | self-test forces a trip at low voltage (closes the latent "reset switch shorted" fault) | §5.4 |
| `K_SUPPLY_EN` | 04 → 14 | fills the relay-coil reservoir before the shock is due; removes the 9 A surge (I-15) | §5.7 |
| `IDLE_FAST_N` | 13 → 12 | fast (≤ 0.1 µs) 3 A idle-current comparator directly on the shunt (I-8) | §5.7 |
| `LSD_OK` | 13 → 12 | low-side gate-drive supply (MGJ2 +20 V) monitor (bridge sim change 9) | §5.7 |

**Changed nets**

| Net | Issue 2 | Issue 3 |
|---|---|---|
| `HOLD_A_HELD_N`, `HOLD_B_HELD_N` | 10 kΩ pull-up; `HOLD_OK` = both low | **47 kΩ** pull-up (+ 1 MΩ backup); first of two sensors; `HOLD_OK` = **all four** lines low |
| `HOLD_RB_A_N`, `HOLD_RB_B_N` | copy of the stack's sensor | copy of sensor 1 only |
| `HOLD_M_CLK`, `HOLD_S_CLK`, `HOLD_A_DRV`, `HOLD_B_DRV` | 1–5 kHz | **1.5 kHz ± 10 %**, stopped **low** (sim/rev3 §2) |
| `OVP_TRIP` | 2,160 V nominal, non-latching, releases ≈ 1,000 V | **2,107 V nominal (2,073–2,142 V), latched** until an `OVP_RST` rising edge or loss of `3V3_SUP` (I-2, I-22) |
| `OC_DET_N` | ≥ 90 A | **≥ 80 A** (band ≈ 74–86 A) (I-8) |
| `IDLE_I_DET_N` | ≥ 5 A from `I_SHOCK` | unchanged threshold (5 A), divider rescaled for the new `I_SHOCK` gain |
| `I_SHOCK` | 26.7 mV/A | **24.0 mV/A** (R1355 13.3 kΩ); saturated amplifier ≤ 3.20 V (I-21) |
| `K_EN` | enables the coil boost, input switch and economiser | enables the **economiser (coil driver) only**; the supply is `K_SUPPLY_EN` |
| `BR_FAULT_N` | OC, idle, rail UV, 25 ms cap | adds **per-phase caps** and **`LSD_OK` low** |
| `WIN_OPEN` | 150 ms | 150 ms nominal, **131.8–161.5 ms** simulated band (logic-relay §1.3) |
| `VCAP_M_TAP` | — | 07 adds 1 GΩ to `VREF_M` so an open tap wire reads a fault (I-23) |
| `VBAT_SYS` rule | (none) | **no steady resistive load below 1 MΩ on any sheet** (D-66, I-10) |

**Removed nets:** none. Local names `BAT_MID` (01) and `BAT_MID_02` (02) become `VBAT_MID`.
Per-phase cap signals, the high-side kill latch and the carrier-loss detectors are **local** to sheets 12 and 13
(no contract net). `hierarchy.toml` and `specs/00_root.toml` carry no sheet pin lists (every inter-sheet signal is
a global label, rule R6), so neither changes for issue 3.

---

**Issue 2 (27 Nov 2026) replaces issue 1 completely. Nothing captured from issue 1 is valid; no sheet had been
captured.** Changes, after the independent review (`docs/reviews/2026-11-27-hardware-design-review.md`) and D-59–D-61:

| Area | Issue 1 | Issue 2 |
|---|---|---|
| Sheets | 15 sheets | **16 sheets**: new `12_bridge_logic` (sequencer); old 12/13/14/15 renumbered to `13_hv_bridge`, `14_patient_interface`, `15_frontend_main`, `16_frontend_sup`. **All reference blocks of sheets 12–16 changed** |
| Work packages | 7 | **8**: WP-D is now the shock logic (09 + 12); storage (11) is WP-F; bridge power stage (13) is WP-G; patient + front ends (14–16) is WP-H. Library files renamed accordingly |
| Main controller | STM32U585VIT6 (LQFP100) | **STM32U585ZIT6 (LQFP144)** |
| Interlock | three series switches; `HOLD_OK` open-drain wired-AND (released = held) | four series switches + **SW5 sequence hold**; second permit switch for the **test permit**; **150 ms latched window**; "held" proven by **optocouplers** (`HOLD_A_HELD_N`, `HOLD_B_HELD_N`, low = held, pull-ups mean "not held") |
| Charger control | `CHG_EN` → `CHG_GATED` (one signal for CHARGE and supply) | **`CHG_SESSION` → `VTRANS_EN`** (eFuse with soft start) and **`CHG_RUN` → `CHARGE_EN`** (+ local VTRANS_GOOD). `CHG_EN`, `CHG_GATED` **deleted** |
| Mode | `MODE_ADULT_HW` (closed in ADULT; default child) | **`MODE_CHILD_HW`** (closed in CHILD; **default adult**, D-60) + readback **`MODE_HW_RB_N`**. `MODE_ADULT_HW` **deleted** |
| Discharge stacks | 3 devices each | **4 devices each**, active release, gate TVS on every device, opto OVP force |
| Bridge | logic on the power-stage sheet; `OC_TRIP_N` | sequencer on 12; gated enables `G_HA/G_LA/G_HB/G_LB`; `BR_FAULT_N` (replaces `OC_TRIP_N`); `SEQ_ACT_M_N`, `SEQ_DONE_M_N`, `SEQ_ACT_S_N`, `SEQ_DONE_S_N`, `SEQ_HOLD`, `WIN_OPEN`, `TEST_MODE`, `OC_DET_N`, `IDLE_I_DET_N`, `BTEST_EN`, `K_EN`; rail `BRIDGE_3V3` now inter-sheet |
| High-side drive | SN6505B | **gated push-pull** from a 400 kHz clock (no soft start) |
| Freewheel | none | **D1301** across R_S + L_S |
| Rectifier | one diode | **two in series + fusible resistor** |
| Patient relay | coil from `BRIDGE_5V` | coil from a 12 V boost on `VBAT_SYS` (enable `K_EN`), hold-up, 90 ms cap; **aux contact `K_REST_N`** |
| Test permit | — | **`TEST_PERMIT_CLK`** (S → 09), **`LOWV_OK`** (07 → 09) |
| Readbacks | `HOLD_OK_RB_N`, `OC_TRIP_N` | `HOLD_RB_A_N`, `HOLD_RB_B_N`, `BR_FAULT_N` |
| Display | `LCD_DISP` open drain | **`LCD_BLANK`** (active high) into an AON inverter |
| Supervisor monitors | `SHOCK_REQ` read by S | **removed** (R-31); S reads `SEQ_ACT_S_N`, `SEQ_DONE_S_N`, `OVP_TRIP` |
| `I_SHOCK` | 40 mV/A | **26.7 mV/A**, INA240 on `BRIDGE_5V` |
| Net classes | `HV_RTN` in HV_PULSE | **`RTN_PULSE`** class for `HV_RTN`, `LS_SRC` |

## 1. Work packages, sheets, owners

One engineer (or agent) per package. A package edits only its own spec files and its own symbol-library source.
`hierarchy.toml`, `specs/00_root.toml` and this contract are edited only by the integrator.

| WP | Sheets | Symbol library source | Size [EST, parts] |
|---|---|---|---|
| **WP-A power** | `01_power_battery`, `02_power_dock`, `03_power_rails` | `hardware/lib/src/aed_power.toml` | ≈ 115 |
| **WP-B controllers** | `04_main_mcu`, `05_supervisor_mcu` | `hardware/lib/src/aed_mcu.toml` | ≈ 120 |
| **WP-C ui+sense** | `06_ui`, `07_sense_main`, `08_sense_sup` | `hardware/lib/src/aed_uisense.toml` | ≈ 95 |
| **WP-D shock logic** | `09_interlock`, `12_bridge_logic` | `hardware/lib/src/aed_logic.toml` | ≈ 130 |
| **WP-E charger** | `10_hv_charger` | `hardware/lib/src/aed_charger.toml` | ≈ 75 |
| **WP-F storage** | `11_hv_storage` | `hardware/lib/src/aed_storage.toml` | ≈ 100 |
| **WP-G bridge power** | `13_hv_bridge` | `hardware/lib/src/aed_bridge.toml` | ≈ 100 |
| **WP-H patient+front ends** | `14_patient_interface`, `15_frontend_main`, `16_frontend_sup` | `hardware/lib/src/aed_patient.toml` | ≈ 120 |

WP-D owns both logic sheets because the interlock (09) and the sequencer (12) are one function split across the
two boards; its nets with WP-G (13) are few and listed in §5.7.

**Library rule.** A custom symbol is defined once, in the library of the first package in this table that needs
it; others use it by `lib_id` and say so in their hand-over note. Prefer stock KiCad symbols where their pins match
the datasheet.

## 2. Reference designators

Sheet NN owns **NN01–NN99 for every prefix** (sheets 01–09: 101–999). **Overflow block = NN01 + 5000** (sheet 12 →
6201–6299; sheet 16 → 6601–6699). Root: `H1–H9`, `NT1–NT9` reserved. The `HV_RTN`–`GND` ties are **NT1101 and NT1102**
on sheet 11. Design names go in `DocRef`.

| Ref | Part | Sheet |
|---|---|---|
| U101, F101 | MAX17320; main fuse 25 A | 01 |
| U201, U202 | BQ25887; TPS25947 | 02 |
| U301, U302 / U303 / U304 / U305 | TPS629210 (`3V3_MAIN`, `3V3_SUP`) / TPS62130 / TPS70933 / RV-3028-C7 | 03 |
| U401 | STM32U585ZIT6 | 04 |
| U501 | MCXA153VLH | 05 |
| U601, U602 | LS013B7DH05; MAX98357A | 06 |
| U701 / U702 / U703 | ADS1220 / REF3425 / `LOWV_OK` comparator | 07 |
| U801 / U802 / U803 | AD7171 / ADR3425 / OVP comparator | 08 |
| U901–U906 | TPS22918: U901 SW1 (request), U902 SW2a (permit), U903 SW2b (test permit), U904 SW3 (held), U905 SW4 (window), U906 SW5 (sequence hold) | 09 |
| U1001, Q1001, T1001, D1001, D1002, R1004, U1002, F1001, U1003 | LT3751, primary switch, transformer, rectifiers, fusible resistor, TPS25982 eFuse, backup fuse, VTRANS_GOOD comparator | 10 |
| J1101, J1102, NT1101, NT1102 | capacitor terminals, NTC connector, star ties | 11 |
| Q1101–Q1104 / Q1105–Q1108 | stack A / stack B (bottom device first) | 11 |
| R1101–R1103 / R1104–R1106 | dump resistors A / B | 11 |
| U1101, U1102 / U1103, U1104 | hold-sense optos A, B (sensor 1) / OVP-force optos A, B | 11 |
| U1105, U1106 | hold-sense optos A, B (sensor 2, issue 3) | 11 |
| Q1112, Q1113 | release pilots (MMBFJ270, issue 3) | 11 |
| Q1109, Q1110 | release switches A, B | 11 |
| U1201… | sequencer logic | 12 |
| Q1301 QHA, Q1302 QLA, Q1303 QHB, Q1304 QLB | SiC switches | 13 |
| R1301 (R_S), L1301 (L_S), D1301 (D_FW), R1302 (shunt) | series element, freewheel diode, shunt | 13 |
| T1301, T1302; U1301, U1302; U1303 | high-side gate transformers; UCC5350MC × 2; MGJ2D052005SC | 13 |
| K1301, R1303 | bridge test relay; 200 Ω test load | 13 |
| U1316 / U1317, U1318 / Q1312–Q1315 / C1347–C1349 | `LSD_OK` monitor / fast idle preamp + comparator / carrier-loss detectors / local film capacitor (issue 3) | 13 |
| K1401, J1401 | patient relay (**made-to-order placeholder symbol**, D-64, docs/requirements/patient-relay.md); pad connector | 14 |
| U1501 | ADS1292R | 15 |
| U1601 | MAX30009 | 16 |

## 3. Naming rules

- **R1** `A–Z 0–9 _`; active low ends in `_N`.
- **R2** `HV_` only for nets galvanically on the capacitor or charger secondary; `PT_` only for patient nets;
  `HSA_`/`HSB_` only for the high-side gate islands of QHA/QHB. **No other net may contain `HV_`, `PT_`, `HSA_`,
  `HSB_`, `LS_SRC` or `RTN`** (net-class patterns match substrings).
- **R3** Local nets need no entry; local HV nets still follow R2.
- **R4** Analog taps end `_TAP`, ≤ 2.6 V full scale.
- **R5** Dynamic consents end `_CLK` (design §B8.1: 1–5 kHz, code-generated).
- **R6** Rails are power symbols; inter-sheet signals are global labels; no hierarchical labels.
- **R7** A signal entering a reader that may be unpowered enters through ≥ 100 kΩ or an Ioff buffer on the reader's
  rail, on the reader's sheet.
- **R8** One default resistor per net, on the sheet named in §5. **Open-drain outputs are pulled up on the reader's
  sheet to the reader's rail.** If a net has two readers on different rails, the one named in §5 pulls; the other
  reads through an Ioff buffer with no pull.
- **R9** No test point on `HV_*`, `HV_FLOAT` or `PT_*` nets.
- **R10** Every part: `MPN`, `Manufacturer`, `Datasheet`, `DocRef` where named; `Verify = "<what>"` for any
  `[UNVERIFIED]` rating.
- **R11** Any signal whose open, unpowered or unsoldered state could be read as "safe to charge", "held",
  "permitted" or "sequence allowed" is a contract violation: report it.

## 4. Rails

| Rail | Made on | Source | Used on | Notes |
|---|---|---|---|---|
| `GND` | 01 | pack negative | all | joined to `HV_RTN` only at NT1101 ∥ NT1102 (11) |
| `HV_RTN` | 11 | NT1101 ∥ NT1102 | 10, 11, 13 | class `RTN_PULSE` |
| `VBAT_PACK` | 01 | MAX17320 FETs | 01, 02 | |
| `VBAT_SYS` | 01 | after F101 | 01, 03, 10, 14 | 14: relay coil boost. **Issue 3 rule (D-66): no steady resistive load below 1 MΩ** on this rail from any sheet (lowest today: 8.0 MΩ, U304 enable divider), so the MAX17320 can release a discharge fault once a short is removed |
| `VDOCK` | 02 | TPS25947 | 02 | |
| `3V3_MAIN` | 03 | U301 | 01, 02, 04, 07, 09, 15 | main domain (issue 3: 01 battery-load comparator and 02 `DOCK_RX` buffer added; 06 removed — WP-A C-2, WP-C C-4) |
| `3V3_SUP` | 03 | U302 | 05, 08, 09, 11, 16 | 11: OVP-force opto LEDs only. **Never used by a main-domain part** |
| `5V0_SYS` | 03 | U303 (`EN_5V0`) | 06, 09, 10 | loss is safe |
| `3V3_AON` | 03 | U304 | 03, 06 | |
| `BRIDGE_5V` | 09 | SW1–SW4 or SW5 | 12, 13 | only while a shock or test is allowed |
| `BRIDGE_3V3` | 12 | LDO + ≥ 2 ms hold-up | 12, 13 | 13: UCC5350 input sides, gate-drive clock |
| `1V8_SUP` | 16 | LDO from 3V3_SUP | 16 | |
| `VREF_M` / `VREF_S` | 07 / 08 | references | own sheet | |

Local rails (one sheet): `VTRANS`, the low-side `+20/−5 V` (`LSD_P20V`, `LSD_N5V`, monitored by `LSD_OK`), the pump nodes, `AVDD_AFE`, `K_VCOIL` (pre-charged under `K_SUPPLY_EN`).
A sheet using a rail made on a placeholder sheet accepts the ERC with `until_sheet`; **no package flags another
package's rail.**

## 5. Inter-sheet signals

Columns: Net · Driver → loads · Domain · Active · Default / unpowered state · Default resistor (sheet, value) ·
Implements. M = main, S = supervisor.

### 5.1 Power control and always-on

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `PWR_BTN_N` | 06 → 03, 04 | 3V3_AON | low | high | **03**: 100 kΩ to 3V3_AON; 04 via R7 | switch-on |
| `MAIN_KEEP` | 04 → 03 | 3V3_MAIN | high | low | **03**: 100 kΩ to GND | power latch |
| `SUP_KEEP` | 05 → 03 | 3V3_SUP | high | low | **03**: 100 kΩ to GND | power latch |
| `SUP_WAKE` | 04 → 03 | 3V3_MAIN | high | low | **03**: 100 kΩ to GND | self-tests power S |
| `EN_5V0` | 04 → 03 | 3V3_MAIN | high | low (5V0_SYS off: safe) | **03**: 100 kΩ to GND | F-2 ordering |
| `RTC_INT_N` | 03 (open drain) → 03, 04 | 3V3_AON | low | high | **03**: 100 kΩ to 3V3_AON (R8 exception: 03 is the primary reader); 04 via R7 | wake timer |
| `RTC_CLKOUT` | 03 → 06 | 3V3_AON | 1 Hz | running | none | LCD inversion, ready flash |
| `READY_D`, `READY_CLK` | 04 → 03 | 3V3_MAIN | high / rising edge | low | **03**: 1 MΩ to GND each | `set_ready_indicator` |
| `I2C_M_SCL`, `I2C_M_SDA` | 04 ↔ 01, 02, 03 | 3V3_MAIN | — | low with M off | **04**: 4.7 kΩ to 3V3_MAIN | gauge, charger, RTC |
| `DOCK_DET_N` | 02 (open drain) → 03, 04 | — | low = docked | high | **03**: 100 kΩ to 3V3_AON; 04 via R7 | wake on dock |

### 5.2 Battery and dock

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `GAUGE_ALRT_N` | 01 (OD) → 04 | — | low | high | **04**: 100 kΩ to 3V3_MAIN | `battery` |
| `BAT_LOAD_EN` | 04 → 01 | 3V3_MAIN | high | low; 01 limits on-time ≤ 250 ms | **01**: 100 kΩ to GND | `battery_under_load` |
| `CHGR_INT_N` | 02 (OD) → 04 | — | low | high | **04**: 100 kΩ to 3V3_MAIN | charging status |
| `CHGR_CE_N` | 04 → 02 | 3V3_MAIN | low = charge | low (charging allowed, hardware limits act) | **02**: 100 kΩ to GND | D-48 |
| `DOCK_TX` | 04 → 02 | 3V3_MAIN | UART idle high | high once M is up; 02's stage is powered from VDOCK | **04**: 100 kΩ to 3V3_MAIN | records out |
| `DOCK_RX` | 02 (Ioff buffer on 3V3_MAIN) → 04 | 3V3_MAIN | UART | — | none | records in |
| *(reserved)* `DOCK_D2` | — | — | — | — | — | Q-47 option (b) only |
| `VBAT_MID` (issue 3) | 01 (pack mid-point, J101 pin 3; also U101 cell input) → 02 (U201 MID via 300 Ω, CBSET via 100 Ω) | analog, 2.5–4.3 V to GND (one cell) | — | follows the cells; with the pack unplugged 02's pins see 0 V through their resistors | none | BQ25887 per-cell regulation (I-1). Class `PWR_HI` by pattern; ≤ 42 mA balancing current, so the 3 mm track is not needed (layout may override) |

### 5.3 Link

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `LINK_M2S` | 04 → 05 | 3V3_MAIN | UART 1 Mbit/s | low = no messages | **05**: 100 kΩ to GND + 1 kΩ series + Ioff Schmitt on 3V3_SUP | link.rs |
| `LINK_S2M` | 05 → 04 | 3V3_SUP | UART | low | **04**: same on 3V3_MAIN | link.rs |

### 5.4 Consent and interlock (sheet 09)

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `HOLD_M_CLK` | 04 → 09 | 3V3_MAIN | **1.5 kHz ± 10 %** = hold (issue 3; was 1–5 kHz); always stopped **low** | static → path A conducts ≤ 25 ms (sim/rev3 §2; was ≤ 10 ms) | **09**: 100 kΩ to GND | `MainHardware::set_discharge_hold`; `discharge_path_active` |
| `HOLD_S_CLK` | 05 → 09 | 3V3_SUP | **1.5 kHz ± 10 %** = hold, stopped **low** | static → path B conducts ≤ 25 ms | **09**: 100 kΩ to GND | `SupervisorHardware::set_discharge_hold` |
| `HOLD_A_DRV` | 09 (buffer on **3V3_MAIN**) → 11 | 3V3_MAIN | square wave | none → stack A on | **11**: 100 kΩ to GND at the pump input | same |
| `HOLD_B_DRV` | 09 (buffer on **3V3_SUP**) → 11 | 3V3_SUP | square wave | none → stack B on | **11**: 100 kΩ to GND | same |
| `HOLD_A_HELD_N`, `HOLD_B_HELD_N` | 11 (opto 1 transistor to GND) → 09 | 5V0_SYS | **low = held** (LED lit by the gate bias); low level ≤ 1.0 V (Darlington) — 09 reads with V_IL ≥ 1.2 V | **high = not held** (opto dark, open or unpowered) | **09**: **47 kΩ** to 5V0_SYS each, plus 1 MΩ backup (issue 3) | input to `HOLD_OK` (local on 09) = **all four** HELD lines low |
| `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N` (issue 3) | 11 (opto 2 transistor to GND) → 09 | 5V0_SYS | as above | as above | **09**: 47 kΩ to 5V0_SYS each, plus 1 MΩ backup | second independent sensor per stack (I-4) |
| `HOLD_RB_A_N`, `HOLD_RB_B_N` | 09 (open-drain copies of sensor 1) → 04 | — | low = held | high | **04**: 100 kΩ to 3V3_MAIN | F-7 hold-sensor test (proposed `hold_readback`) |
| `HOLD_RB_A2_N`, `HOLD_RB_B2_N` (issue 3) | 09 (open-drain copies of sensor 2) → 04 | — | low = held | high | **04**: 100 kΩ to 3V3_MAIN | F-7: each of the four sensors proven separately |
| `OVP_TRIP` | 08 (push-pull on 3V3_SUP) → 11 (gate of Q1111, which switches the two opto LEDs from 3V3_SUP), 05 | 3V3_SUP | high = over-voltage; **latched (issue 3)**: trips at 2,107 V nominal (2,073–2,142 V), stays high for any capacitor voltage until an `OVP_RST` rising edge or loss of 3V3_SUP | low | **11**: 100 kΩ to GND; 05 via R7 | hardware OV force; proposed `overvoltage_tripped` |
| `OVP_RST` (issue 3) | 05 → 08 | 3V3_SUP | **rising edge** = clear the latch (AC-coupled on 08: 100 nF + 100 kΩ into a BSS138; a line stuck high cannot hold the latch clear). Active-high: not `_N` (R1) | low | **08**: 100 kΩ to GND | proposed `SupervisorHardware::reset_overvoltage` |
| `OVP_TEST` (issue 3) | 05 → 08 | 3V3_SUP | high = inject a trip into `OVP_SENSE` (through a BAS116 and 22 kΩ on 08) | low (no injection; the diode isolates the line) | **08**: 100 kΩ to GND | proposed `SupervisorHardware::set_overvoltage_test`; self-test step T-OVP (design §B14.5) |
| `SHOCK_REQ` | 04 → 09 | 3V3_MAIN | high | low | **09**: 100 kΩ to GND | `request_shock` / `withdraw_shock_request` (start only, D-59) |
| `PERMIT_CLK` | 05 → 09 | 3V3_SUP | 1–5 kHz = permit | static → none | **09**: 100 kΩ to GND | `SupervisorHardware::set_permit` (start only) |
| `TEST_PERMIT_CLK` | 05 → 09 | 3V3_SUP | 1–5 kHz = test permit | static → none | **09**: 100 kΩ to GND | proposed `set_test_permit` (D-61) |
| `LOWV_OK` | 07 (comparator, push-pull on 3V3_MAIN) → 09 | 3V3_MAIN | high = capacitor < 60 V | low (no test mode) | **09**: 100 kΩ to GND | D-61 hardware voltage limit |
| `BRIDGE_ARMED_N` | 09 (OD, low while BRIDGE_5V up) → 04 | — | low | high | **04**: 100 kΩ to 3V3_MAIN | `shock_status` / `sequence_status` |
| `WIN_OPEN` | 09 → 12 | 5V0_SYS logic → 12 via Ioff buffer | high = within 150 ms window | low | **12**: 100 kΩ to GND (Ioff buffer on BRIDGE_3V3) | D-59 point 3 |
| `TEST_MODE` | 09 → 12 | same | high = test mode | low (shock mode rules) | **12**: 100 kΩ to GND | D-61 |
| `SEQ_HOLD` | 12 → 09 | BRIDGE_3V3 | high = sequence running | low | **09**: 100 kΩ to GND | D-59 completion (SW5) |

### 5.5 Charger

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `CHG_SESSION` | 04 → 09 | 3V3_MAIN | high | low | **09**: 100 kΩ to GND | `set_charger` (session) |
| `CHG_RUN` | 04 → 09 | 3V3_MAIN | high | low | **09**: 100 kΩ to GND | `set_charger` (regulation, F-3) |
| `VTRANS_EN` | 09 → 10 | 5V0_SYS | high = eFuse on | low | **10**: 100 kΩ to GND | `charger_may_run` (supply) |
| `CHARGE_EN` | 09 → 10 | 5V0_SYS | high | low (CHARGE < 0.3 V) | **10**: 100 kΩ to GND | `charger_may_run` (CHARGE pin, after local VTRANS_GOOD) |
| `CHG_DONE_N`, `CHG_FAULT_N` | 10 (open collector) → 04 | — | low | high | **04**: 100 kΩ to 3V3_MAIN each | charge status; F-3 fault retry |
| `MODE_CHILD_HW` | 06 (pole B closes to 5V0_SYS in CHILD) → 10 | 5V0_SYS | high = child ceiling | **low = adult ceiling** (D-60) | **10**: 100 kΩ to GND | D-60 |
| `MODE_HW_RB_N` | 10 (OD, from the FB-switch gate drive) → 04 | — | low = child ceiling active | high | **04**: 100 kΩ to 3V3_MAIN | D-60 readback (proposed `mode_hw_ceiling`) |
| `HV_CAP_P` | 10 (after D1001, D1002, R1004) → 11, 13 | HV | — | — | (bleeder on 11) | capacitor + |

### 5.6 Storage and sensing

| Net | Driver → loads | Domain | Active | Default | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `VCAP_M_TAP` | 11 (clamped to GND on 11) → 07 | analog | 1:998.5 (WP-C C-1) | 0 V | none on the net; **07 (issue 3): 1 GΩ from `VREF_M` to the follower input** so an open tap wire drives the reading up and `LOWV_OK` low within 0.4 s (I-23) | `MainHardware::capacitor_voltage`; `LOWV_OK` |
| `VCAP_S_TAP` | 11 (clamped to GND) → 08 | analog | 1:1,002.0 (WP-C C-1) | 0 V | none | `SupervisorHardware::capacitor_voltage`; OVP |
| `NTC_CAP_M` | 11 (J1102) → 07 | analog | — | — | **07**: bias to VREF_M | `temperatures` (Capacitor) |
| `NTC_CAP_S` | 11 (J1102) → 05 | analog | — | — | **05**: bias to S's ADC ref | `SupervisorHardware::temperature` |
| `HVM_SCLK`, `HVM_MOSI`, `HVM_CS_N` | 04 → 07 | 3V3_MAIN | SPI | CS high | **07**: 100 kΩ up on `HVM_CS_N` | main chain |
| `HVM_MISO`, `HVM_DRDY_N` | 07 → 04 | 3V3_MAIN | SPI | — | none | main chain |
| `HVS_SCLK`, `HVS_PDRST_N` | 05 → 08 | 3V3_SUP | serial / low = reset | PDRST low | **08**: 100 kΩ to GND on `HVS_PDRST_N` | supervisor chain |
| `HVS_DOUT` | 08 → 05 | 3V3_SUP | DOUT/RDY | — | none | supervisor chain |

### 5.7 Bridge logic (12) ↔ controllers, power stage (13) and patient relay (14)

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `CMD_HA`, `CMD_LA`, `CMD_HB`, `CMD_LB` | 04 (one advanced timer) → 12 | 3V3_MAIN | high | low | **12**: 100 kΩ to GND each; Ioff buffers on BRIDGE_3V3 | `request_shock(plan)` phases |
| `RELAY_CMD` | 04 → 12 | 3V3_MAIN | high | low | **12**: 100 kΩ to GND | `request_shock` |
| `BTEST_CMD` | 04 → 12 | 3V3_MAIN | high | low | **12**: 100 kΩ to GND | test sequence (D-61) |
| `G_HA`, `G_HB` | 12 → 13 | BRIDGE_3V3 | high = carrier on | low | **13**: 100 kΩ to GND each | gated high-side enables |
| `G_LA`, `G_LB` | 12 → 13 | BRIDGE_3V3 | high = low side on | low | **13**: 100 kΩ to GND each (at the UCC5350 inputs) | gated low-side enables |
| `BTEST_EN` | 12 → 13 | BRIDGE_3V3 | high = test relay closed | low | **13**: 100 kΩ to GND | test mode only |
| `OC_DET_N` | 13 (comparator, push-pull on BRIDGE_3V3) → 12 | BRIDGE_3V3 | low = ≥ **80 A** (issue 3; band ≈ 74–86 A) | high when working; **an open or unpowered comparator reads low = fault** | **12**: 100 kΩ to GND (see note) | over-current latch |
| `IDLE_I_DET_N` | 13 (comparator on `I_SHOCK`, push-pull) → 12 | BRIDGE_3V3 | low = ≥ 5 A | as above | **12**: 100 kΩ to GND (see note) | idle-current latch (R-14), slow path |
| `IDLE_FAST_N` (issue 3) | 13 (OPA838 ×11 preamp on the shunt Kelvin pair + TLV3501, push-pull) → 12 | BRIDGE_3V3 | low = ≥ 3 A (band ≈ 2.6–3.5 A); ≤ 0.1 µs shunt-to-latch | as above: open or unpowered reads fault | **12**: 100 kΩ to GND | idle-current latch, fast path; ORed with `IDLE_I_DET_N` on 12 (I-8) |
| `LSD_OK` (issue 3) | 13 (TLV7031 on the MGJ2 +20 V rail, push-pull) → 12 | BRIDGE_3V3 | **high = low-side supply ≥ 16.9 V** (band ≈ 16.5–17.3 V) | low = fault (open, unpowered or rail low) | **12**: 100 kΩ to GND | start condition and rail-collapse fault beside `BR5_OK` (bridge sim change 9) |
| `K_EN` | 12 → 14 | BRIDGE_3V3 | high = **coil driver (DRV110) on** (issue 3: no longer starts the boost or input switch) | low | **14**: 100 kΩ to GND | relay transfer |
| `K_SUPPLY_EN` (issue 3) | 04 → 14 | 3V3_MAIN | high = coil supply (input switch Q1401 + boost U1401) on: fills the 6.6 mF reservoir at ≈ 2 A in 74–99 ms | low (standby draw zero) | **14**: 100 kΩ to GND | proposed `MainHardware::set_coil_supply`; raised ≥ 0.2 s before `RELAY_CMD` (I-15) |
| `K_REST_N` | 14 (force-guided aux contact to GND) → 04, 12 | — | **low = at rest** | high (= "not at rest": main faults; sequencer treats as transferred only if it also saw it **low when `K_EN` rose**, issue 3, WP-H C-6) | **04**: 100 kΩ to 3V3_MAIN; 12 reads through 10 kΩ + 1 nF then an Ioff buffer, no pull | proposed `relay_at_rest`; sequencer start rule |
| `SEQ_ACT_M_N`, `SEQ_DONE_M_N` | 12 (OD) → 04 | — | low | high (not active / not done) | **04**: 100 kΩ to 3V3_MAIN each | proposed `MainHardware::sequence_status` |
| `SEQ_ACT_S_N`, `SEQ_DONE_S_N` | 12 (OD) → 05 | — | low | high | **05**: 100 kΩ to 3V3_SUP each | proposed `SupervisorHardware::sequence_status` |
| `BR_FAULT_N` | 12 (OD) → 04 | — | low = ended by a fault latch (OC, idle, rail UV, `LSD_OK` low, 25 ms cap, **phase-1 cap 10.75–11.85 ms, phase-2 cap 7.31–8.06 ms**) | high | **04**: 100 kΩ to 3V3_MAIN | `shock_status` = Fault |
| `I_SHOCK` | 13 (INA240 on BRIDGE_5V + divider 13.3 k / 20.0 k) → 04 | analog 0–3.20 V | **24.0 mV/A** (issue 3) | 0 V | **04**: anti-alias RC | `shock_status`; test sequence |
| `OUT_A_TAP`, `OUT_B_TAP` | 13 (dividers, clamped to GND on 13) → 07 | analog | 1:1,000 | ≤ 35 mV idle | none | shorted-switch and test-relay checks |
| `HV_OUT_A`, `HV_OUT_B` | 13 → 14 | HV | — | pulled to HV_RTN (1 MΩ) on 13 | **13** | shock path |
| `PT_FE_A`, `PT_FE_B` | 14 (relay rest contacts) → 15, 16 | PATIENT | — | — | none on 14 | `pad_resistance` (both), ECG |

**Note on `OC_DET_N` / `IDLE_I_DET_N` / `IDLE_FAST_N` / `LSD_OK`:** push-pull comparator outputs on `BRIDGE_3V3`. The default resistor is a
**100 kΩ pull-down on 12**, so an open or unpowered comparator reads *fault* (R11), and the sequencer refuses to
start a sequence while any of them reads low. On 12, `OC_DET_N`, `IDLE_I_DET_N` and `IDLE_FAST_N` also set the
local high-side kill latch, which removes `G_HA`/`G_HB` at once instead of after the 20–36 µs end-of-sequence
delay (issue 3, I-8).

### 5.8 Pads, front ends and user interface

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `PAD_PRESENT_N` | 14 → 04 | — | low = fitted | high | **04**: 100 kΩ to 3V3_MAIN | `pad_cartridge` |
| `PAD_ID_1W` | 04 ↔ 14 | 3V3_MAIN | 1-Wire | idle high | **04**: 2.2 kΩ to 3V3_MAIN; series R + TVS on 14 | `pad_cartridge`, D-57 |
| `NTC_PADS` | 14 → 04 | analog | — | — | **04**: bias to the ADC ref | `temperatures` (Pads) |
| `AFE_SCLK`, `AFE_MOSI`, `AFE_CS_N`, `AFE_START` | 04 → 15 | 3V3_MAIN | SPI / START | CS high, START low | **15**: 100 kΩ up on CS, down on START | ECG + impedance streams |
| `AFE_RESET_N` | 04 → 15 | 3V3_MAIN | low = reset | low | **15**: 100 kΩ to GND | same |
| `AFE_MISO`, `AFE_DRDY_N` | 15 → 04 | 3V3_MAIN | — | — | none | same |
| `SBZ_SCLK`, `SBZ_MOSI`, `SBZ_CS_N` | 05 → 16 | 3V3_SUP | SPI | CS high | **16**: 100 kΩ up on CS | `SupervisorHardware::pad_resistance` |
| `SBZ_MISO` | 16 → 05 | 3V3_SUP | — | — | none | same |
| `SBZ_INT_N` | 16 (OD) → 05 | — | low | high | **05**: 100 kΩ to 3V3_SUP | same |
| `LCD_SCLK`, `LCD_SI`, `LCD_SCS` | 04 → 06 | 3V3_MAIN into an AON part | SPI | low | **06**: 1 MΩ to GND each | `prompt` |
| `LCD_BLANK` | 04 → 06 (74AUP1G04 on 3V3_AON) | 3V3_MAIN | high = blank | **low = display on** | **06**: 1 MΩ to GND | ready shown while off |
| `I2S_BCLK`, `I2S_LRCLK`, `I2S_SD` | 04 → 06 | 3V3_MAIN | I²S | low | none; **4.7 kΩ series on 06** | `prompt`, `metronome_beat` |
| `AMP_SD_N` | 04 → 06 | 3V3_MAIN | low = off | low | **06**: 100 kΩ to GND | same |
| `CANCEL_M_N` | 06 → 04 | — | low | high | **04**: 10 kΩ to 3V3_MAIN | `MainHardware::cancel_pressed` |
| `CANCEL_S_N` | 06 → 05 | — | low | high | **05**: 10 kΩ to 3V3_SUP | `SupervisorHardware::cancel_pressed` |
| `MODE_ADULT_N`, `MODE_CHILD_N` | 06 (pole A to GND) → 04 | — | low = that position | both high = fault | **04**: 10 kΩ to 3V3_MAIN each | `patient_mode`; D-60 compare |

## 6. Board-split nets

Low-voltage board 01–09, high-voltage/patient board 10–16. Crossing:
- Power: `VBAT_SYS` (charger and coil boost — high-current contacts), `GND`, `5V0_SYS`, `3V3_MAIN`, `3V3_SUP`, `BRIDGE_5V`.
- Interlock: `HOLD_A_DRV`, `HOLD_B_DRV`, `HOLD_A_HELD_N`, `HOLD_B_HELD_N`, `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N` (issue 3), `OVP_TRIP`, `VTRANS_EN`, `CHARGE_EN`,
  `MODE_CHILD_HW`, `WIN_OPEN`, `TEST_MODE`, `SEQ_HOLD`.
- Charger: `CHG_DONE_N`, `CHG_FAULT_N`, `MODE_HW_RB_N`.
- Sensing: `VCAP_M_TAP`, `VCAP_S_TAP`, `NTC_CAP_M`, `NTC_CAP_S`, `OUT_A_TAP`, `OUT_B_TAP`, `I_SHOCK`.
- Sequencer: `CMD_*`, `RELAY_CMD`, `BTEST_CMD`, `SEQ_ACT_M_N`, `SEQ_DONE_M_N`, `SEQ_ACT_S_N`, `SEQ_DONE_S_N`,
  `BR_FAULT_N`, `K_REST_N`, `K_SUPPLY_EN` (issue 3).
- Not crossing (both ends on one board): `VBAT_MID` (01–02), `HOLD_RB_A2_N`/`HOLD_RB_B2_N` (09–04), `OVP_RST`/`OVP_TEST`
  (05–08), `IDLE_FAST_N`/`LSD_OK` (13–12).
- Pads and front ends: `PAD_PRESENT_N`, `PAD_ID_1W`, `NTC_PADS`, all `AFE_*`, `SBZ_*`.

**No `HV_*`, `HV_FLOAT` or `PT_*` net crosses.**

## 7. HAL methods → nets

### `MainHardware` (existing methods)

| Method | Nets / parts |
|---|---|
| `now` | U401 crystals |
| `pad_resistance` | `PT_FE_A/B` via K1401 rest contacts → U1501 ch. 1; `AFE_*` |
| `capacitor_voltage` | main string (11) → `VCAP_M_TAP` → U701/U702; `HVM_*` |
| `set_charger(target)` | `CHG_SESSION` → `VTRANS_EN`; `CHG_RUN` → `CHARGE_EN` (regulation loop on `capacitor_voltage`); `CHG_DONE_N`, `CHG_FAULT_N`; ceiling via `MODE_CHILD_HW` |
| `set_discharge_hold` | `HOLD_M_CLK` → `HOLD_A_DRV` → stack A |
| `request_shock(plan)` | `SHOCK_REQ`; `RELAY_CMD`; `CMD_HA` → `CMD_LB` (phase 1); `CMD_HB` → `CMD_LA` (phase 2) — sequencer rules §B13 |
| `withdraw_shock_request` | `SHOCK_REQ`, `CMD_*`, `RELAY_CMD` low (no effect after `SEQ_ACT`, D-59) |
| `shock_status` | `BRIDGE_ARMED_N`, `SEQ_ACT_M_N`, `SEQ_DONE_M_N`, `BR_FAULT_N`, `I_SHOCK` |
| `cancel_pressed` | `CANCEL_M_N` |
| `patient_mode` | `MODE_ADULT_N`, `MODE_CHILD_N` |
| `battery`, `battery_under_load` | `I2C_M_*` → U101; `GAUGE_ALRT_N`; `BAT_LOAD_EN` |
| `pad_cartridge` | `PAD_PRESENT_N`, `PAD_ID_1W` |
| `temperatures` | cells (`I2C_M_*`), `NTC_CAP_M`, `NTC_PADS`, U401 internal |
| `prompt`, `metronome_beat` | `I2S_*`, `AMP_SD_N`, `LCD_*` |
| `set_ready_indicator` | `READY_D`, `READY_CLK` |
| `send_status` / `supervisor_report` | `LINK_M2S` / `LINK_S2M` |
| board callbacks (ECG, impedance) | `AFE_*` |
| standby | `RTC_INT_N`, `I2C_M_*`, F-RAM (local), `MAIN_KEEP`, `SUP_WAKE` |

### `MainHardware` — proposed additions (design §F-11)

| Proposed method | Nets |
|---|---|
| `sequence_status()` | `BRIDGE_ARMED_N`, `SEQ_ACT_M_N`, `SEQ_DONE_M_N`, `BR_FAULT_N` |
| `relay_at_rest()` | `K_REST_N` |
| `mode_hw_ceiling()` | `MODE_HW_RB_N` |
| `hold_readback()` | `HOLD_RB_A_N`, `HOLD_RB_B_N`, `HOLD_RB_A2_N`, `HOLD_RB_B2_N` (four sensors) |
| `set_coil_supply(on)` (issue 3) | `K_SUPPLY_EN` |
| test-sequence primitives | `BTEST_CMD`, `CMD_*`, `I_SHOCK`, `OUT_A_TAP`, `OUT_B_TAP` |

### `SupervisorHardware`

| Method | Nets |
|---|---|
| `now` | U501 crystal |
| `capacitor_voltage` | supervisor string → `VCAP_S_TAP` → U801/U802; `HVS_*` |
| `pad_resistance` | `PT_FE_A/B` → U1601; `SBZ_*` |
| `cancel_pressed` | `CANCEL_S_N` |
| `temperature` | `NTC_CAP_S` |
| `set_permit` | `PERMIT_CLK` (start only, D-59) |
| `set_discharge_hold` | `HOLD_S_CLK` → `HOLD_B_DRV` → stack B |
| `receive_status` / `send_report` | `LINK_M2S` / `LINK_S2M` |
| `set_fault_alarm` | local piezo and light |
| *proposed* `set_test_permit` | `TEST_PERMIT_CLK` |
| *proposed* `sequence_status` | `SEQ_ACT_S_N`, `SEQ_DONE_S_N` |
| *proposed* `overvoltage_tripped` | `OVP_TRIP` |
| *proposed* `reset_overvoltage` (issue 3) | `OVP_RST` (one rising edge) |
| *proposed* `set_overvoltage_test(on)` (issue 3) | `OVP_TEST` |

### `interlock.rs` → hardware

| Function | Realised by |
|---|---|
| `discharge_path_active(m, s)` | stack A on unless `HOLD_A_DRV` toggles; stack B on unless `HOLD_B_DRV` toggles; `HOLD_OK` (local 09) = `HOLD_A_HELD_N`, `HOLD_A_HELD2_N`, `HOLD_B_HELD_N` and `HOLD_B_HELD2_N` all low |
| `bridge_may_fire(r, p, d)` | **at the start**: SW1 (`SHOCK_REQ`) · SW2 (`PERMIT_OK`) · SW3 (`HOLD_OK`) · SW4 (`WIN_OPEN`) → `BRIDGE_5V`; **after `SEQ_ACTIVE`**: SW5 (`SEQ_HOLD`) completes the pulse (D-59) |
| `charger_may_run(e, d)` | `VTRANS_EN` = `CHG_SESSION`·`HOLD_OK`·NOT `TEST_MODE`; `CHARGE_EN` = `CHG_RUN`·`HOLD_OK`·NOT `TEST_MODE` (09), ANDed with VTRANS_GOOD (10) |

## 8. Parts already chosen (do not re-decide)

| Sheet | Parts |
|---|---|
| 01 | MAX17320 + 2 protection N-FETs + sense resistor; F101 25 A; 0.82 Ω load + N-FET + RC limit **with BAS116 in series with R111 (78.7 kΩ), new R117 4.7 MΩ, R112 100 kΩ (issue 3, I-25)**; keyed pack connector |
| 02 | TPS25947, 6.0 V TVS, BQ25887, three dock contacts, VDOCK-powered open-drain data driver, 74LVC1G17 receiver, dock-detect MOSFET |
| 03 | TPS629210 × 2 (own fuses), TPS62130, TPS70933, RV-3028-C7, SN74AUP1G74, ready LED + pulse network, wake-OR |
| 04 | STM32U585ZIT6, 16 MHz + 32.768 kHz crystals, W25Q128JV, CY15B104QN, Ioff receivers (`LINK_S2M`, `PWR_BTN_N`, `RTC_INT_N`, `DOCK_DET_N`), SWD pads |
| 05 | MCXA153VLH, crystal, piezo + dual buffer, red LED, Ioff receivers (`LINK_M2S`, `OVP_TRIP`), NTC bias, SWD pads |
| 06 | LS013B7DH05, 74AUP1G04 (3V3_AON) for blanking, MAX98357A + 28 mm speaker, 4.7 kΩ I²S series resistors, two cancel switches, two-pole changeover mode switch (pole B closes in CHILD), power switch |
| 07 | ADS1220, REF3425, NTC bias, **`LOWV_OK` comparator** (60 V on the tap, push-pull, 3V3_MAIN), **R718 1 GΩ `VREF_M` → follower input (issue 3)** |
| 08 | AD7171, ADR3425, zero-drift buffer, OVP comparator **re-centred to 2,107 V and latching, with AC-coupled `OVP_RST` reset and diode-isolated `OVP_TEST` injection (issue 3)** |
| 09 | TPS22918 × 6 (U901–U906: SW1, SW2a, SW2b, SW3, SW4, SW5), consent buffers on their own rails, two band-limited pump detectors (permit, test permit; **bleeds 4 × 64.9 kΩ, issue 3**) on 3V3_SUP, hold-drive buffers (A on 3V3_MAIN, B on 3V3_SUP) with 8 kHz two-pole low-pass at the buffer input, `HOLD_OK` logic over **four** HELD lines with **47 kΩ** pull-ups, **window-start delay 100 µs (issue 3)**, window timer + latch + 50 ms re-arm, `TEST_MODE` logic, charger gates, open-drain readbacks |
| 10 | LT3751IUFD, BSC070N10LS5, 5 mΩ sense, T1001, **2 × UX-F5B**, **R1004 fusible**, **TPS25982 eFuse**, F1001 15 A, VTRANS_GOOD comparator + ≥ 1 ms delay, FB network (10 × 4.02 MΩ 0.1 % + 36.9 kΩ 0.1 %, N-FET default on), RVOUT/RVTRANS 40.2 kΩ, **RBG 876 Ω 0.1 % (issue 3)**, RDCM 18.2 kΩ, UVLO/OVLO (69.8 k, 155 k, **71.5 k**, 87.5 k), primary clamp, LC filter, bulk |
| 11 | J1101, J1102, NT1101 ∥ NT1102, **8 × IXTT2N170D2 (issue 3)**, 6 × 1.5 kΩ EY, 2 × (4 × **4.7 MΩ** sharing strings + 3.3 nF speed-up caps), **per device 10 MΩ + 3.3 nF drain–source, upper-gate steering diodes GP02-40 ∥ 22 MΩ, 100 MΩ gate–source (issue 3)**, 8 × 15 V bidirectional gate TVS, upper-gate 100 kΩ resistors, 2 × four-stage pumps **+ 470 Ω to the gate** + 3.3 µF reservoirs + 1.01 MΩ gate resistors, 2 × release switches (BSS138 + **MMBFJ270** pilot), **2 × 2** hold-sense optos (TLP187 + **4.3 V** zener + 3.9 kΩ ∥ 22 kΩ), 2 × OVP-force optos, bleeder 8 × 2.49 MΩ, main string 10 × 4.02 MΩ HVCR2512 + 2 × 80.6 kΩ, supervisor string 12 × 3.32 MΩ TNPV1206 + 2 × 79.6 kΩ, tap clamps (10 kΩ + 3.6 V to GND) |
| 12 | BRIDGE_3V3 LDO + ≥ 2 ms hold-up, 4.5 V rail-UV detector, power-on reset, RAIL_READY (20 ms), Ioff command buffers, sequencer state latches, lockout delays (10 µs, 150 µs, 50 µs lead), **latched 25 ms cap** (RC + comparator + latch), OC / idle / UV / cap fault latches, relay coil 90 ms latched cap and start window (≤ 60 ms), turn-off order delays (20 µs, 1 ms, 15 ms), open-drain status outputs; **issue 3: high-side kill latch, `IDLE_FAST_N` and `LSD_OK` inputs, per-phase caps (11.37 / 7.72 ms nominal), XFER bounce filter and reset, `K_REST_N`-seen-at-rest latch, BTOK on a TLV7031** |
| 13 | 4 × MSC080SMA330B4N, R1301 3.9 Ω ceramic, L1301 30 µH, **D1301 3.3 kV SiC**, snubber RC, R1302 2 mΩ, 2 × UCC5350MC, MGJ2D052005SC, 400 kHz oscillator + two-phase divider, 2 × dual N-FET push-pull + T1301/T1302 + rectifiers + 18 V zener + 100 Ω + 100 nF + PNP turn-off + 10 kΩ bleed, fast OC comparator (**80 A**), idle comparator (5 A), INA240A1 on BRIDGE_5V + **13.3 k / 20.0 k** divider, **issue 3: fast 3 A idle path (OPA838 + TLV3501), `LSD_OK` monitor, carrier-loss detector per island, local film capacitor 2 × 0.15 µF 3 kV in series (75 nF) on `HV_CAP_P`–`HV_RTN`**, output dividers (4 × 249 kΩ + bottom leg, clamp to GND), K1301 D-HR reed + R1303 200 Ω |
| 14 | K1401 (**made-to-order placeholder**, D-64), 12 V coil boost from VBAT_SYS + economiser + 6.6 mF hold-up + zener release; **issue 3: supply (Q1401, U1401) enabled by `K_SUPPLY_EN`, economiser by `K_EN`; C1401 680 nF, R1406 515 kΩ**, J1401, pad-line series R + TVS, pad NTC |
| 15 | ADS1292R, per pad: DPCR2512 51 kΩ sense **for the impedance branch plus a separate 51 kΩ DPCR (R1528, R1529) for the ECG branch (issue 3)**, 2 × 4.7 nF 3 kV C0G + 1 kΩ pulse-rated drive resistor, TVS to analog ground below the rail, rail shunt clamp, filters |
| 16 | MAX30009, 1V8_SUP LDO, taps as sheet 15 |

## 9. Decisions left to each capture engineer

- **WP-A:** protection FETs, MAX17320 thresholds (OC ≥ 30 A ms delay, SC ≥ 60 A ≥ 100 µs, **auto-retry**), pack connector, fuse parts, buck passives, wake-OR implementation (button enables both regulators with no logic between), load RC, BQ25887 TS network for 0–45 °C.
- **WP-B:** pin assignment (four `CMD_*` on one advanced timer; `I_SHOCK`, `NTC_PADS` on ADC pins); supervisor crystal; decoupling; straps.
- **WP-C:** speaker and piezo parts; NTC bias; buffer and comparator parts (OVP **2,107 V latching** (issue 3), LOWV 60 V from the real divider ratios); filters.
- **WP-D:** the logic family and gate-level implementation of §B8 and §B13 (discrete 74LVC/AUP with Ioff, or another discrete family — **no programmable logic** without an owner decision); detector RC values meeting §B8.1 (1–5 kHz pass, ≥ 100 kHz reject, rise ≤ 3 ms, hold-up ≥ 3 ms, collapse ≤ 8 ms); timer RCs and comparator thresholds (cap 25 ms ± 15 %, **per-phase caps per `capture-notes/rev3-changes.md` (issue 3)**, window 150 ms, re-arm 50 ms, lockouts); `BRIDGE_3V3` hold-up sizing (≥ 2 ms at the sheet's measured load).
- **WP-E:** primary clamp; LC filter; eFuse current-limit and dV/dt values; VTRANS_GOOD threshold (5.0 V, hysteresis) and delay; R1004 part.
- **WP-F:** pump capacitor values (issue 3: the requirement is now "both sensors read held, HELD_N ≤ 1.0 V, at −20 °C, 3.1 V, 1.35–1.65 kHz, aged sensors", met as captured plus the rev-3 changes in simulation); release-switch bias network (stack conducting ≤ 10 ms after the last edge); opto part (CTR at 1 mA); speed-up capacitor value (≥ 10 × device Coss); bleeder resistor parts.
- **WP-G:** gate resistors; oscillator and push-pull parts; rectifier and PNP parts; snubber values; comparator networks (90 A, 5 A); D1301 part; L1301 construction choice with the maker.
- **WP-H:** clamp and TVS parts; drive-tap resistor part (≥ 1.5 kV / 10 µs); boost converter and economiser for the chosen relay's coil; hold-up value; 1-Wire protection.

## 10. Cross-sheet behaviour each side must check and write down

- **WP-D with WP-B:** consent clocks 1–5 kHz from code, maximum gap 1 ms (F-1); the shock command order and timing (F-4); what the sequencer does with commands out of order.
- **WP-D with WP-F:** `HOLD_*_DRV` amplitude and frequency into the pumps (1.5 kHz ± 10 %, issue 3); `HOLD_*_HELD_N` and `HOLD_*_HELD2_N` timing (issue 3, simulated: held ≤ 35 ms after the first edge from cold; not held ≤ 25 ms after the last edge with the clock stopped low).
- **WP-D with WP-E:** `VTRANS_EN`, `CHARGE_EN` off within 1 ms of `HOLD_OK` loss or `TEST_MODE`.
- **WP-D with WP-G:** `G_*` timing; `OC_DET_N`/`IDLE_I_DET_N`/`IDLE_FAST_N`/`LSD_OK` polarity and response (≤ 0.5 µs for OC, ≤ 0.1 µs shunt-to-12 for `IDLE_FAST_N`, ≤ 5 µs for `LSD_OK`); `BRIDGE_3V3` load of sheet 13 for the hold-up sizing; turn-off order on rail collapse.
- **WP-D with WP-H:** `K_EN` timing; coil hold-up ≥ 30 ms; `K_REST_N` contact type; `K_EN` low must open the coil drive directly after the reservoir (CI-8).
- **WP-B with WP-H (issue 3):** `K_SUPPLY_EN` raised ≥ 0.2 s before `RELAY_CMD`, lowered on abort.
- **WP-B with WP-C (issue 3):** `OVP_RST` is an edge (a pulse ≥ 10 µs), `OVP_TEST` a level.
- **WP-H with WP-B:** ADS1292R at 64 kHz, MAX30009 at 128 kHz; idle drive outputs high-impedance.
- **Everyone:** R7 and R11 for every output on your sheet.
