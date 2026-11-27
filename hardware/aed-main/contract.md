# aed-main capture — shared net contract

**NOT FOR CONSTRUCTION — unreviewed design.** Written 2026-11-27, before any sheet is captured, so that sheets
captured in parallel meet on the same names. Design source: [`docs/HARDWARE_DESIGN.md`](../../docs/HARDWARE_DESIGN.md)
(section numbers §Bn below refer to it). Where this file and the design document disagree on a **name**, on **which
sheet holds a part** or on **which sheet fits a resistor**, this file wins; where they disagree on the **design**,
capture the conservative reading and log a question in your hand-over note. Firmware contract: the traits in
`firmware/crates/aed-core/src/hal.rs` and the functions in `interlock.rs` (table §7).

Nothing in this file is an instruction to build or power anything (CLAUDE.md rule 2, SAFETY.md §2a).

---

## 1. Work packages, sheets, owners

Seven capture work packages (WP). One engineer (or agent) per package. A package edits only its own spec files,
its own symbol-library source file and nothing else. `hierarchy.toml`, `specs/00_root.toml` and this contract are
edited only by the integrator; a package that needs a change there reports it.

| WP | Sheets (spec files it owns) | Symbol library source it owns | Size [EST, parts] |
|---|---|---|---|
| **WP-A power** | `01_power_battery`, `02_power_dock`, `03_power_rails` | `hardware/lib/src/aed_power.toml` | ≈ 110 |
| **WP-B controllers** | `04_main_mcu`, `05_supervisor_mcu` | `hardware/lib/src/aed_mcu.toml` | ≈ 110 |
| **WP-C ui+sense** | `06_ui`, `07_sense_main`, `08_sense_sup` | `hardware/lib/src/aed_uisense.toml` | ≈ 90 |
| **WP-D interlock+storage** | `09_interlock`, `11_hv_storage` | `hardware/lib/src/aed_interlock.toml` | ≈ 120 |
| **WP-E charger** | `10_hv_charger` | `hardware/lib/src/aed_charger.toml` | ≈ 60 |
| **WP-F bridge** | `12_hv_bridge` | `hardware/lib/src/aed_bridge.toml` | ≈ 120 |
| **WP-G patient+front ends** | `13_patient_interface`, `14_frontend_main`, `15_frontend_sup` | `hardware/lib/src/aed_patient.toml` | ≈ 110 |

**Library rule.** A custom symbol is defined once, in the library of the package that first needs it. If two
packages need the same custom part (for example the 74LVC1G17 Ioff buffer if it is not in KiCad's stock library),
the package listed first in this table owns it and the other uses it by `lib_id`; announce it in the hand-over note.
Prefer KiCad stock symbols (`Device:R`, `Device:C`, `power:*`, stock logic and regulator symbols) wherever the stock
symbol's pin names and numbers match the datasheet; check them like a custom one.

## 2. Reference designators

Sheet NN owns references **NN01–NN99 for every prefix** (`R1201`, `C1201`, `U1201`, `Q1201`, `D1201`, `TP1201`
…). For sheets 01–09 that is 101–199 … 901–999. **Overflow block:** NN01 + 5000 (sheet 12 overflow is
`R6201–R6299`); sheet 01's overflow is 5101–5199. Root: `H1–H9` (mounting holes), `NT1–NT9` (reserved). The net
tie joining `HV_RTN` to `GND` is `NT1101` on sheet 11. Design-document names (QHA, K1301, T1001 …) go in each part's
`DocRef` field.

Fixed references (named in the design document; keep them):

| Ref | Part | Sheet |
|---|---|---|
| U101 | MAX17320 | 01 |
| F101 | main fuse 25 A | 01 |
| U201 | BQ25887 | 02 |
| U301, U302 | TPS629210 (`3V3_MAIN`, `3V3_SUP`) | 03 |
| U303 | TPS62130 (`5V0_SYS`) | 03 |
| U304 | TPS70933 (`3V3_AON`) | 03 |
| U305 | RV-3028-C7 | 03 |
| U401 | STM32U585VIT6 | 04 |
| U501 | MCXA153VLH | 05 |
| U701 / U702 | ADS1220 / REF3425 | 07 |
| U801 / U802 | AD7171 / ADR3425 | 08 |
| U1001, Q1001, T1001, D1001, F1001 | LT3751, primary switch, transformer, output diode, VTRANS fuse | 10 |
| C1101 (off-board, connector/terminals J1101), NT1101 | storage capacitor, HV_RTN–GND tie | 11 |
| Q1101–Q1103 / Q1104–Q1106 | discharge stack A / stack B | 11 |
| R1101–R1103 / R1104–R1106 | dump resistors path A / path B | 11 |
| Q1201 QHA, Q1202 QLA, Q1203 QHB, Q1204 QLB | SiC bridge switches | 12 |
| R1201 (R_S), L1201 (L_S), R1202 (shunt) | series element, shunt | 12 |
| T1201, T1202 | high-side gate transformers | 12 |
| K1201, R1203 | bridge test relay and 2 kΩ test load | 12 |
| K1301, J1301 | patient relay, pad connector | 13 |
| U1401 | ADS1292R | 14 |
| U1501 | MAX30009 | 15 |

## 3. Naming rules

- **R1** Upper case, `A–Z 0–9 _`. Active-low nets end in `_N`. No net name starts with a digit except the rails
  listed in §4.
- **R2** Prefix `HV_` **only** for nets galvanically connected to the storage capacitor or the charger secondary
  (they get net class HV / HV_PULSE by pattern). Prefix `PT_` **only** for patient/applied-part nets. `HSA_` / `HSB_`
  **only** for the floating high-side gate-drive islands of QHA / QHB (class HV_FLOAT). **No other net may contain
  the substrings `HV_`, `PT_`, `HSA_` or `HSB_`**, because the net-class patterns match substrings (`*HV_*`).
- **R3** Local nets (one sheet only) need no contract entry; name them by function (`LT_FB`, `STACKA_G1`).
  Local high-voltage nets still follow R2 (`HV_FB_MID`, `HV_STA_D2`).
- **R4** Analog taps carry the suffix `_TAP` and are ≤ 2.6 V at full scale.
- **R5** Dynamic consent signals end in `_CLK` (§5.4).
- **R6** Every rail is a power symbol (`[nets] power`). Every inter-sheet signal is a global label
  (`[nets] global`). There are no multi-instance sheets and no hierarchical labels in this project.
- **R7** A signal from a rail that stays powered into a controller (or chip) that may be unpowered enters the
  reader through **≥ 100 kΩ series resistance or a buffer with power-off (Ioff) protection powered from the
  reader's rail**, on the reader's sheet. Nothing may back-power an unpowered controller.
- **R8** **Default resistors.** Each inter-sheet signal has exactly one default (pull) resistor, on the sheet named
  in §5. Open-drain outputs are pulled up **on the reader's sheet, to the reader's own rail** (so the reader never
  sees a voltage from a rail it does not share). No other sheet adds a pull on that net. `tools/xcheck.py` lists
  pulls per net; more than one is a contract violation unless §5 says "two by design".
- **R9** No test point on any `HV_*`, `HV_FLOAT` or `PT_*` net.
- **R10** Every part carries `MPN`, `Manufacturer`, `Datasheet` and, where the design document names it, `DocRef`.
  Parts with `[UNVERIFIED]` ratings carry a field `Verify = "<what>"`.

## 4. Rails

| Rail | Made on | Source | Used on | Notes |
|---|---|---|---|---|
| `GND` | 01 | pack negative | all | system ground; joined to `HV_RTN` only at NT1101 (11) |
| `HV_RTN` | 11 | NT1101 | 10, 11, 12 | capacitor negative, charger secondary return, bridge shunt return |
| `VBAT_PACK` | 01 | MAX17320 protection FETs | 01, 02 | BQ25887 charges into it |
| `VBAT_SYS` | 01 | after F101 | 01, 03, 10 | 21 A peak to the charger |
| `VDOCK` | 02 | after TPS25947 | 02 only | power symbol, local use |
| `3V3_MAIN` | 03 | U301 | 04, 06, 07, 09, 14 | main controller domain |
| `3V3_SUP` | 03 | U302 | 05, 08, 09, 15 | supervisor domain; **never used by a main-domain part** |
| `5V0_SYS` | 03 | U303 (enable `EN_5V0`) | 06, 09, 10, 11 | loss = safe (design §B1.6); the bridge takes its power only through `BRIDGE_5V` |
| `3V3_AON` | 03 | U304 | 03, 06 | always on |
| `BRIDGE_5V` | 09 | third series load switch | 12, 13 | exists only while a shock is allowed |
| `1V8_SUP` | 15 | LDO from `3V3_SUP` | 15 only | |
| `VREF_M` | 07 | REF3425 | 07 only | |
| `VREF_S` | 08 | ADR3425 | 08 only | |

Local rails made and used on one sheet (`VTRANS`, `BRIDGE_3V3`, the low-side `+20/−5 V`, the hold pumps' negative
nodes, `AVDD_AFE`) are local power nets of that sheet (`flag_nodes` where fed through a ferrite or 0 Ω).
A sheet using a rail made on a sheet that is still a placeholder accepts the ERC "power input not driven" with
`until_sheet = "<maker sheet>"`; **no package adds a PWR_FLAG to a rail made by another package.**

## 5. Inter-sheet signals

Columns: **Net** · **Driver → loads** · **Domain** (rail of the driver's output) · **Active** · **Default / unpowered
state and why it is safe** · **Default resistor (sheet, value, to)** · **Implements** (HAL method or decision).
`M` = main controller, `S` = supervisor.

### 5.1 Power control and always-on domain

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `PWR_BTN_N` | 06 (button to GND) → 03, 04 | 3V3_AON | low | high (not pressed) | **03**: 100 kΩ to 3V3_AON. 04 reads through R7 | switch-on (§B1.7, owner question 1) |
| `MAIN_KEEP` | 04 → 03 | 3V3_MAIN | high | low: main rail off unless another wake source | **03**: 100 kΩ to GND | power latch |
| `SUP_KEEP` | 05 → 03 | 3V3_SUP | high | low | **03**: 100 kΩ to GND | power latch |
| `SUP_WAKE` | 04 → 03 | 3V3_MAIN | high | low: supervisor stays off during standby wakes | **03**: 100 kΩ to GND | scheduled self-tests power the supervisor |
| `EN_5V0` | 04 → 03 | 3V3_MAIN | high | low: 5V0_SYS off (safe, §B1.6) | **03**: 100 kΩ to GND | — |
| `RTC_INT_N` | 03 (RV-3028 INT, open drain) → 03, 04 | 3V3_AON | low | high | **03**: 100 kΩ to 3V3_AON (exception to R8: the wake logic on 03 is the primary reader); 04 reads through R7 | wake timer (schedule.rs, standby.rs) |
| `RTC_CLKOUT` | 03 → 06 | 3V3_AON | 1 Hz square | running whenever the pack is connected | none (push-pull) | memory-LCD inversion; ready LED flash |
| `READY_D` | 04 → 03 | 3V3_MAIN | high = ready | low | **03**: 1 MΩ to GND | `MainHardware::set_ready_indicator` |
| `READY_CLK` | 04 → 03 | 3V3_MAIN | rising edge latches | low (no edge) | **03**: 1 MΩ to GND | same |
| `I2C_M_SCL`, `I2C_M_SDA` | 04 ↔ 01 (MAX17320), 02 (BQ25887), 03 (RV-3028) | 3V3_MAIN | — | low when main off (devices on other rails see a quiet bus) | **04**: 4.7 kΩ to 3V3_MAIN, one pair | `battery`, `battery_under_load`, `temperatures` (cells), wake timer |

### 5.2 Battery and dock

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `GAUGE_ALRT_N` | 01 (open drain) → 04 | — | low | high | **04**: 100 kΩ to 3V3_MAIN | `battery` |
| `BAT_LOAD_EN` | 04 → 01 | 3V3_MAIN | high | low: load off; **01 also limits on-time to ≤ 250 ms in hardware** | **01**: 100 kΩ to GND | `battery_under_load` (D-15) |
| `CHGR_INT_N` | 02 (open drain) → 04 | — | low | high | **04**: 100 kΩ to 3V3_MAIN | battery charging status |
| `CHGR_CE_N` | 04 → 02 | 3V3_MAIN | low = charge enabled | **low: charging allowed** (hardware temperature limits still act) | **02**: 100 kΩ to GND | D-48 charge window (firmware layer) |
| `DOCK_DET_N` | 02 (open drain, switched by VDOCK) → 03, 04 | — | low = docked | high | **03**: 100 kΩ to 3V3_AON; 04 reads through R7 | wake on dock |
| `DOCK_TX` | 04 → 02 | 3V3_MAIN | UART, idle high | high (idle) once 3V3_MAIN is up; with the main off the line is low and the contact shows a "break", which the dock ignores. 02's open-drain stage is powered from VDOCK, so it never drives the contact when undocked | **04**: 100 kΩ to 3V3_MAIN | records out (D-09) |
| `DOCK_RX` | 02 (Ioff buffer on 3V3_MAIN) → 04 | 3V3_MAIN | UART | — | none (push-pull) | records/configuration in |
| *(reserved)* `DOCK_D2` | — | — | — | — | — | second data contact if Q-47 option (b) is chosen; do not use the name for anything else |

### 5.3 Supervisor link

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `LINK_M2S` | 04 (UART TX) → 05 | 3V3_MAIN | UART, 1 Mbit/s | low = "break" = no messages = main stopped | **05**: 100 kΩ to GND, plus 1 kΩ series and an Ioff Schmitt buffer on 3V3_SUP | `send_status` / `receive_status` (link.rs) |
| `LINK_S2M` | 05 (UART TX) → 04 | 3V3_SUP | UART | low = no reports | **04**: 100 kΩ to GND, 1 kΩ series, Ioff buffer on 3V3_MAIN | `send_report` / `supervisor_report` |

### 5.4 Interlock (§B8) — the shock path

Dynamic (`_CLK`) nets: a ≥ 1 kHz square wave produced by code that stops when the controller's control loop stops
(not a free-running timer). A static level, either one, means "no consent".

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `HOLD_M_CLK` | 04 → 09 | 3V3_MAIN | toggling = hold | static → path A conducts | **09**: 100 kΩ to GND | `MainHardware::set_discharge_hold`; `interlock::discharge_path_active` |
| `HOLD_S_CLK` | 05 → 09 | 3V3_SUP | toggling = hold | static → path B conducts | **09**: 100 kΩ to GND | `SupervisorHardware::set_discharge_hold` |
| `HOLD_A_DRV` | 09 (buffer **powered from 3V3_MAIN**) → 11 (pump A) | 3V3_MAIN | square wave | no wave → stack A on | **11**: 100 kΩ to GND at the pump input | same |
| `HOLD_B_DRV` | 09 (buffer **powered from 3V3_SUP**) → 11 (pump B) | 3V3_SUP | square wave | no wave → stack B on | **11**: 100 kΩ to GND | same |
| `HOLD_OK` | 11 (two open-drain comparators, wired-AND) → 09 | 5V0_SYS | high = both stacks measured off | low (either stack on, or 5V0_SYS off) | **09**: 10 kΩ to 5V0_SYS (the only pull) | NOT `discharge_path_active`; input to `bridge_may_fire` and `charger_may_run` |
| `HOLD_OK_RB_N` | 09 (open drain) → 04 | — | low = held | high | **04**: 100 kΩ to 3V3_MAIN | self-test "SupervisorNotHolding" diagnosis |
| `OVP_TRIP` | 08 (comparator, push-pull on 3V3_SUP) → 11, 05 | 3V3_SUP | high = over-voltage | low (but supervisor unpowered already means path B on) | **11**: 100 kΩ to GND; 05 reads through R7 | hardware over-voltage (§B4.6, §B5.5) |
| `SHOCK_REQ` | 04 → 09, 05 | 3V3_MAIN | high | low: no request | **09**: 100 kΩ to GND; 05 reads through R7 (monitor only) | `request_shock` / `withdraw_shock_request`; `bridge_may_fire(main_request…)` |
| `PERMIT_CLK` | 05 → 09 | 3V3_SUP | toggling = permit | static → no permit | **09**: 100 kΩ to GND | `SupervisorHardware::set_permit`; `bridge_may_fire(…supervisor_permit…)` |
| `BRIDGE_ARMED_N` | 09 (open drain, low while BRIDGE_5V is up) → 04 | — | low | high | **04**: 100 kΩ to 3V3_MAIN | `shock_status` (Refused vs Pending) |
| `CHG_EN` | 04 → 09 | 3V3_MAIN | high | low: charger off | **09**: 100 kΩ to GND | `set_charger`; `charger_may_run(charger_enabled…)` |
| `CHG_GATED` | 09 (AND gate on 5V0_SYS) → 10 | 5V0_SYS | high | low: LT3751 CHARGE low **and** VTRANS switch open | **10**: 100 kΩ to GND | `charger_may_run` (D-25) |

Local to 09 (no contract entry needed, but these names are reserved for the test points): `PERMIT_OK`,
`SHOCK_REQ_BUF`, `BR5V_AFTER_REQ`, `BR5V_AFTER_PERMIT`.

### 5.5 Charger

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `CHG_DONE_N` | 10 (LT3751 DONE, open collector) → 04 | — | low = target reached or fault | high | **04**: 100 kΩ to 3V3_MAIN | `set_charger` status |
| `CHG_FAULT_N` | 10 (LT3751 FAULT, open collector) → 04 | — | low = supply out of range | high | **04**: 100 kΩ to 3V3_MAIN | charge fault |
| `MODE_ADULT_HW` | 06 (switch pole B: closes to 5V0_SYS in ADULT) → 10 | 5V0_SYS | high = adult ceiling | low = **child ceiling (1,221 V)** | **10**: 100 kΩ to GND | hardware mode ceiling (§B4.4) |
| `HV_CAP_P` | 10 (output diode) → 11, 12 | HV | — | — | — (bleeder on 11) | the capacitor's positive terminal |

### 5.6 Storage and sensing

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `VCAP_M_TAP` | 11 (main string tap, clamped on 11) → 07 | analog ≤ 2.6 V | 1:998 of VCAP | 0 V | none (the divider is the default) | `MainHardware::capacitor_voltage` |
| `VCAP_S_TAP` | 11 (supervisor string tap, clamped on 11) → 08 | analog | 1:1,000 | 0 V | none | `SupervisorHardware::capacitor_voltage`; OVP |
| `NTC_CAP_M` | 11 (connector J1102 pin, NTC to GND) → 07 | analog | — | — | **07**: bias resistor to VREF_M | `temperatures` (Capacitor) |
| `NTC_CAP_S` | 11 (J1102, second NTC to GND) → 05 | analog | — | — | **05**: bias resistor to the supervisor's ADC reference | `SupervisorHardware::temperature` (D-56) |
| `HVM_SCLK`, `HVM_MOSI`, `HVM_CS_N` | 04 → 07 | 3V3_MAIN | SPI | CS high | **07**: 100 kΩ pull-up on `HVM_CS_N` to 3V3_MAIN | main chain |
| `HVM_MISO`, `HVM_DRDY_N` | 07 → 04 | 3V3_MAIN | SPI / low = data | — | none (ADS1220 drives both) | main chain |
| `HVS_SCLK`, `HVS_PDRST_N` | 05 → 08 | 3V3_SUP | serial / low = power-down | PDRST low = ADC held reset until firmware | **08**: 100 kΩ to GND on `HVS_PDRST_N` | supervisor chain |
| `HVS_DOUT` | 08 → 05 | 3V3_SUP | DOUT/RDY | — | none | supervisor chain |

### 5.7 Bridge and patient interface

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `CMD_HA`, `CMD_LA`, `CMD_HB`, `CMD_LB` | 04 (one advanced timer) → 12 | 3V3_MAIN | high = switch on | low: all off (and no drive power without BRIDGE_5V) | **12**: 100 kΩ to GND each; inputs through Ioff logic on BRIDGE_3V3 | `request_shock(plan)` phase timing |
| `RELAY_CMD` | 04 → 13 | 3V3_MAIN | high = pads to bridge | low: pads to front ends | **13**: 100 kΩ to GND | `request_shock` |
| `BTEST_CMD` | 04 → 12 | 3V3_MAIN | high = test load across outputs | low: open | **12**: 100 kΩ to GND | bridge self-test (§B6.6) |
| `I_SHOCK` | 12 (INA240 on BRIDGE_3V3) → 04 | analog 0–3.2 V | 40 mV/A | 0 V when unpowered | **04**: anti-alias RC; no pull | `shock_status` |
| `OC_TRIP_N` | 12 (latch, open drain) → 04 | — | low = over-current latched | high | **04**: 100 kΩ to 3V3_MAIN | `shock_status` (Fault) |
| `OUT_A_TAP`, `OUT_B_TAP` | 12 (output pull-down dividers, clamped on 12) → 07 | analog | 1:1,000 | 0 V | none | shorted-switch detection; output voltage |
| `HV_OUT_A`, `HV_OUT_B` | 12 → 13 | HV | — | pulled to HV_RTN on 12 (1 MΩ dividers) | **12** (the dividers) | the shock path |
| `PT_FE_A`, `PT_FE_B` | 13 (relay rest contacts) → 14, 15 | PATIENT | — | — | none on 13; each front end has its own bias network ≥ 10 MΩ | `pad_resistance` (both), ECG |
| `PAD_PRESENT_N` | 13 (connector loop to GND) → 04 | — | low = pad plug fitted | high | **04**: 100 kΩ to 3V3_MAIN | `pad_cartridge` (Missing) |
| `PAD_ID_1W` | 04 ↔ 13 (1-Wire EEPROM in the pad plug) | 3V3_MAIN | 1-Wire | idle high | **04**: 2.2 kΩ to 3V3_MAIN (1-Wire master pull-up); series resistor and TVS on 13 | `pad_cartridge` (Expired), pad-set identity (D-57) |
| `NTC_PADS` | 13 (NTC to GND) → 04 | analog | — | — | **04**: bias resistor to the main's ADC reference | `temperatures` (Pads) |

### 5.8 Front ends and user interface

| Net | Driver → loads | Domain | Active | Default / unpowered | Default resistor | Implements |
|---|---|---|---|---|---|---|
| `AFE_SCLK`, `AFE_MOSI`, `AFE_CS_N`, `AFE_START` | 04 → 14 | 3V3_MAIN | SPI; START high = convert | CS high, START low | **14**: 100 kΩ up on `AFE_CS_N`, 100 kΩ down on `AFE_START` | ECG and impedance streams (`on_ecg_sample`, `on_impedance_sample`), `pad_resistance` |
| `AFE_RESET_N` | 04 → 14 | 3V3_MAIN | low = reset / power-down | **low: held in reset until firmware** | **14**: 100 kΩ to GND | same |
| `AFE_MISO`, `AFE_DRDY_N` | 14 → 04 | 3V3_MAIN | SPI / low = data ready | — | none | same |
| `SBZ_SCLK`, `SBZ_MOSI`, `SBZ_CS_N` | 05 → 15 | 3V3_SUP | SPI | CS high | **15**: 100 kΩ up on `SBZ_CS_N` to 3V3_SUP (MAX30009 VDDIO = 3V3_SUP) | `SupervisorHardware::pad_resistance` |
| `SBZ_MISO` | 15 → 05 | 3V3_SUP | SPI | — | none | same |
| `SBZ_INT_N` | 15 (open drain) → 05 | — | low | high | **05**: 100 kΩ to 3V3_SUP | same |
| `LCD_SCLK`, `LCD_SI`, `LCD_SCS` | 04 → 06 | 3V3_MAIN into a 3V3_AON part | SPI | low (display keeps its image) | **06**: 1 MΩ to GND each (R7: display inputs defined while main is off) | `prompt` (screen) |
| `LCD_DISP` | 04 (open drain) → 06 | — | low = blank | **high: display on**, showing the last image | **06**: 1 MΩ to 3V3_AON | ready shown while off (§B10.3) |
| `I2S_BCLK`, `I2S_LRCLK`, `I2S_SD` | 04 → 06 | 3V3_MAIN | I²S | — | none | `prompt` (speech), `metronome_beat` |
| `AMP_SD_N` | 04 → 06 | 3V3_MAIN | low = amplifier off | low | **06**: 100 kΩ to GND | same |
| `CANCEL_M_N` | 06 (contact 1 to GND) → 04 | — | low = pressed | high | **04**: 10 kΩ to 3V3_MAIN | `MainHardware::cancel_pressed` (D-31) |
| `CANCEL_S_N` | 06 (contact 2 to GND) → 05 | — | low = pressed | high | **05**: 10 kΩ to 3V3_SUP | `SupervisorHardware::cancel_pressed` |
| `MODE_ADULT_N`, `MODE_CHILD_N` | 06 (switch pole A to GND) → 04 | — | low = that position | both high = "between positions" (a fault) | **04**: 10 kΩ to 3V3_MAIN each | `patient_mode` |

**Two by design** (R8 exceptions): none besides `RTC_INT_N` (noted above). The cancel contacts share only `GND`.

## 6. Board-split nets (if layout uses a low-voltage board and a high-voltage board)

Every net with a driver or load on both groups (sheets 01–09 vs 10–15) crosses the board-to-board connector:

- Power: `VBAT_SYS` (21 A peak — separate high-current contacts), `GND` (several contacts, plus the high-current return),
  `5V0_SYS`, `3V3_MAIN`, `3V3_SUP`, `BRIDGE_5V`.
- Interlock: `HOLD_A_DRV`, `HOLD_B_DRV`, `HOLD_OK`, `OVP_TRIP`, `CHG_GATED`, `MODE_ADULT_HW`.
- Charger: `CHG_DONE_N`, `CHG_FAULT_N`.
- Sensing: `VCAP_M_TAP`, `VCAP_S_TAP`, `NTC_CAP_M`, `NTC_CAP_S`, `OUT_A_TAP`, `OUT_B_TAP`, `I_SHOCK`.
- Bridge and patient: `CMD_HA`, `CMD_LA`, `CMD_HB`, `CMD_LB`, `RELAY_CMD`, `BTEST_CMD`, `OC_TRIP_N`,
  `PAD_PRESENT_N`, `PAD_ID_1W`, `NTC_PADS`.
- Front ends: all `AFE_*` and `SBZ_*`.

**No `HV_*`, `HV_FLOAT` or `PT_*` net crosses the connector.** The front ends are on the high-voltage/patient
board so that `PT_FE_A/B` stay there.

## 7. HAL methods → nets

### `MainHardware`

| Method | Nets / parts |
|---|---|
| `now` | U401 crystals (local) |
| `pad_resistance` | `PT_FE_A/B` via K1301 rest contacts → U1401 channel 1; `AFE_*` |
| `capacitor_voltage` | main divider string (11) → `VCAP_M_TAP` → U701/U702; `HVM_*` |
| `set_charger(target)` | `CHG_EN` (→ `CHG_GATED`); closed loop on `capacitor_voltage`; `MODE_ADULT_HW` sets the hardware ceiling; `CHG_DONE_N`, `CHG_FAULT_N` |
| `set_discharge_hold` | `HOLD_M_CLK` → `HOLD_A_DRV` → stack A; `HOLD_OK_RB_N` readback |
| `request_shock(plan)` | `SHOCK_REQ`; then `RELAY_CMD`; then `CMD_HA/LB` (phase 1), `CMD_HB/LA` (phase 2) from one timer |
| `withdraw_shock_request` | `SHOCK_REQ`, all `CMD_*`, `RELAY_CMD` low |
| `shock_status` | `BRIDGE_ARMED_N`, `I_SHOCK`, `OC_TRIP_N` (+ both voltage readings) |
| `cancel_pressed` | `CANCEL_M_N` |
| `patient_mode` | `MODE_ADULT_N`, `MODE_CHILD_N` |
| `battery` | `I2C_M_*` → U101; `GAUGE_ALRT_N` |
| `battery_under_load` | `BAT_LOAD_EN` + U101 readings |
| `pad_cartridge` | `PAD_PRESENT_N`, `PAD_ID_1W` |
| `temperatures` | Cell1/Cell2: `I2C_M_*` → U101; Capacitor: `NTC_CAP_M`; Pads: `NTC_PADS`; Board: U401 internal |
| `prompt` | `I2S_*`, `AMP_SD_N`; `LCD_*` |
| `metronome_beat` | `I2S_*` |
| `set_ready_indicator` | `READY_D`, `READY_CLK` |
| `send_status` / `supervisor_report` | `LINK_M2S` / `LINK_S2M` |
| board callbacks `on_ecg_sample`, `on_impedance_sample` | `AFE_*` (DRDY at 250/s) |
| standby: wake timer, durable store | `RTC_INT_N`, `I2C_M_*` (U305); F-RAM on U401's local SPI; `MAIN_KEEP`, `SUP_WAKE` |

### `SupervisorHardware`

| Method | Nets / parts |
|---|---|
| `now` | U501 crystal (local) |
| `capacitor_voltage` | supervisor divider string (11) → `VCAP_S_TAP` → U801/U802; `HVS_*` |
| `pad_resistance` | `PT_FE_A/B` → U1501; `SBZ_*` |
| `cancel_pressed` | `CANCEL_S_N` |
| `temperature` | `NTC_CAP_S` |
| `set_permit` | `PERMIT_CLK` |
| `set_discharge_hold` | `HOLD_S_CLK` → `HOLD_B_DRV` → stack B |
| `receive_status` / `send_report` | `LINK_M2S` / `LINK_S2M` |
| `set_fault_alarm` | piezo and fault light, local to 05 |
| (monitors) | `SHOCK_REQ`, `OVP_TRIP` |

### `interlock.rs` → hardware

| Function | Realised by |
|---|---|
| `discharge_path_active(main_hold, supervisor_hold)` | stack A on unless `HOLD_A_DRV` toggling; stack B on unless `HOLD_B_DRV` toggling (11); `HOLD_OK` = both measured off |
| `bridge_may_fire(main_request, supervisor_permit, discharge_active)` | three series load switches on 09: `SHOCK_REQ` · `PERMIT_OK` · `HOLD_OK` → `BRIDGE_5V` |
| `charger_may_run(charger_enabled, discharge_active)` | `CHG_GATED = CHG_EN AND HOLD_OK` (09) → LT3751 CHARGE and the VTRANS switch (10) |

## 8. Parts already chosen (do not re-decide)

| Sheet | Parts |
|---|---|
| 01 | MAX17320 (+ 2 protection N-FETs, sense resistor per its datasheet), F101 25 A time-lag, 0.82 Ω pulse load + N-FET + RC time limit, pack connector (keyed; part is a capture decision) |
| 02 | TPS25947 eFuse, 6.0 V TVS, BQ25887, dock contact pads (3: `DOCK_5V`, `DOCK_GND`, `DOCK_DATA`), open-drain data driver (powered from VDOCK side) + 74LVC1G17 receiver, VDOCK-switched dock-detect MOSFET |
| 03 | TPS629210 × 2 (separate fuses), TPS62130, TPS70933, RV-3028-C7, SN74AUP1G74 ready latch, green ready LED + pulse network, wake-OR logic |
| 04 | STM32U585VIT6, 16 MHz and 32.768 kHz crystals, W25Q128JV, CY15B104QN, Ioff receivers for `LINK_S2M`, `PWR_BTN_N`, `RTC_INT_N`, `DOCK_DET_N`, SWD pads |
| 05 | MCXA153VLH, own crystal, piezo transducer + dual buffer, red fault LED, Ioff receivers for `LINK_M2S`, `SHOCK_REQ`, `OVP_TRIP`, SWD pads |
| 06 | Sharp LS013B7DH05, MAX98357A + 28 mm sealed speaker, two sealed tactile switches (cancel), two-pole changeover slide switch (mode), sealed tactile switch (power) |
| 07 | ADS1220, REF3425, NTC bias network |
| 08 | AD7171, ADR3425, zero-drift buffer on the tap, TLV3012-class comparator with reference (2,160 V trip) |
| 09 | 74LVC-family consent buffers (main ones on 3V3_MAIN, supervisor ones on 3V3_SUP), permit diode-pump detector + Schmitt buffer, AND gate for `CHG_GATED`, 3 × TPS22918 in series, open-drain readback buffer |
| 10 | LT3751 (I grade), BSC070N10LS5, 5 mΩ sense, T1001, UX-F5B, FB divider (40.2 MΩ / 40.2 kΩ + 57.6 kΩ switched), RVOUT/RVTRANS 40.2 kΩ, RDCM 18.2 kΩ, RBG 909 Ω, UVLO/OVLO resistors (design §B4.3), LM74502 + N-FET, F1001 15 A, primary clamp, input LC filter and bulk capacitors |
| 11 | C1101 terminals, J1102 NTC connector, 6 × IXTT10N100D2, 6 × 1.5 kΩ Ohmite EY, balancing strings 3 × 10 MΩ per stack, two inverting charge pumps, two `HOLD_OK` comparators, two OVP shunt N-FETs, bleeder 8 × 2.49 MΩ, main string 10 × 4.02 MΩ HVCR2512 + 2 × 80.6 kΩ, supervisor string 12 × 3.32 MΩ TNPV1206 + 2 × 79.6 kΩ, tap clamps, NT1101 |
| 12 | 4 × MSC080SMA330B4N (alternative G2R120MT33J), R1201 3.9 Ω ceramic composition, L1201, R1202 2 mΩ shunt, INA240A1, TLV3201 + 74LVC1G74 latch, 74LVC1G123 window, lockout logic, 2 × SN6505B + T1201/T1202 + rectifiers + 100 nF gate capacitors + PNP turn-off, 2 × UCC5350MC, MGJ2D052005SC, BRIDGE_3V3 LDO, output dividers 4 × 249 kΩ + bottom leg, K1201 D-HR reed relay + R1203 2 kΩ ceramic |
| 13 | K1301 (G17-class DPDT, qualification part), J1301 (made to order), pad-line series resistors and TVS, pad-area NTC |
| 14 | ADS1292R, per pad: DPCR2512 51 kΩ sense, 2 × 4.7 nF 3 kV C0G in series + DPCR 1 kΩ drive, clamp diodes + TVS, input filters, AVDD filter |
| 15 | MAX30009, `1V8_SUP` LDO, per pad: DPCR 51 kΩ sense, 2 × 4.7 nF 3 kV + DPCR 1 kΩ drive, clamps |

## 9. Decisions left to each capture engineer

Decide, write down why in the sheet's group notes, and list in the hand-over note.

- **WP-A:** protection MOSFETs and MAX17320 sense resistor and thresholds; pack connector; exact fuse parts; inductors and capacitors of the three bucks; the wake-OR logic implementation (diodes vs gates) provided the power button enables both controller regulators without any logic in between; the RC values of the battery-load time limit; BQ25887 TS network values for a 0–45 °C hard window (or an added comparator if the TS thresholds cannot reach 45 °C).
- **WP-B:** pin assignment within the constraints of §2.2/§2.3 of the design (the four bridge commands on one advanced timer; `I_SHOCK` and `NTC_PADS` on ADC pins; UARTs for the link and the dock); supervisor crystal frequency; decoupling; boot-pin straps. If the 100-pin STM32U585 package has fewer usable I/O than counted, report it, do not drop a net.
- **WP-C:** speaker and piezo parts (≥ 85 dB targets); NTC bias values; buffer and comparator parts on 08 with the 2,160 V threshold from the actual divider ratio; anti-alias filters.
- **WP-D:** charge-pump topology and values meeting "bottom gate ≤ −7.0 V at 25 µA from a 3.3 V clock" (or 5 V buffer for pump A); `HOLD_OK` comparator and level-shift network (threshold −6.0 V); speed-up capacitors on the balancing strings; permit detector time constant (collapse within ≤ 2 ms of the clock stopping); high-voltage chip resistor parts for the bleeder.
- **WP-E:** primary clamp design; input filter; parallel second primary MOSFET if needed; layout notes for the RVOUT/RDCM nodes (datasheet: no planes under them).
- **WP-F:** lockout delay network (≥ 150 µs) and window one-shot (25 ms) values; over-current threshold resistor network (90 A); rectifier and PNP turn-off parts on the high-side islands; whether R1201 is one part or two in parallel.
- **WP-G:** clamp diode and TVS parts; front-end filter values; bias networks; whether K1301's coil needs a 12 V boost from `BRIDGE_5V` (if the qualified relay has a 12 V coil); the 1-Wire series resistor and TVS.

## 10. Cross-sheet behaviour each side must check and write down

- **WP-D with WP-B:** the hold and permit clocks' frequency, duty and the maximum gap firmware may leave without
  causing a release (pump hold-up) — and that the clocks come from code, not a free-running timer.
- **WP-D with WP-E:** `CHG_GATED` low must stop both the LT3751 (CHARGE < 0.3 V) and VTRANS within 1 ms.
- **WP-F with WP-B:** the order of `RELAY_CMD`, `CMD_HA`, `CMD_LB`, gap, `CMD_HB`, `CMD_LA` and the timing margins
  (design §B8.4); what the lockout does to a command sequence that violates it.
- **WP-F with WP-G:** relay operate/release times against the bridge sequence; `HV_OUT_*` pull-downs on 12 only.
- **WP-G with WP-B:** ADS1292R modulation setting (64 kHz) and MAX30009 carrier (128 kHz).
- **Everyone:** what your sheet's outputs do when your rail is off and the reader's rail is on (R7), and the
  reverse.
