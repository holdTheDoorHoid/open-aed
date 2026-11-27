# Hardware design specification

| Version | Date | What changed |
|---|---|---|
| 0.1 | 27 Nov 2026 | First design specification, written before schematic capture: topology and parts for every block, the arithmetic, the sheet plan (`hardware/aed-main/hierarchy.toml`) and the net contract (`hardware/aed-main/contract.md`). |
| **0.2** | **27 Nov 2026** | **Revised after the independent review** (`docs/reviews/2026-11-27-hardware-design-review.md`, findings R-1 to R-31) and decisions **D-59** (consent starts a shock; hardware finishes it), **D-60** (mode-switch disagreement → adult ceiling, read back) and **D-61** (low-voltage test permit). Main changes: a hardware **shock sequencer** with a non-retriggerable latched time cap, a sequence hold that keeps the bridge powered to the end of the planned pulse, a maximum permit window, defined turn-off order and supply-collapse behaviour (§B13); a **freewheel diode** for the series coil (§B6.5); the SN6505B replaced by a **gated push-pull high-side drive** with no soft-start (§B6.3); the charger's supply switch and CHARGE pin **split**, regulation by the CHARGE pin, FB used only as a child ceiling, a consistent **threshold stack-up** (§B4); "held" now **actively proven** by an optocoupler that needs the negative gate bias (§B5.4); four devices per discharge stack, an active release, gate clamps, hold referenced to the stack; **test-permit mode** (§B14); a patient relay coil supply with on-time cap, hold-up and a position contact (§B7); two rectifiers in series and an eFuse that isolates charger faults (§B4); the bridge logic moved to its own sheet; **16 sheets, 8 capture work packages**; a "Firmware requirements from hardware" section (§F); a "Response to review" section (§R). |

> **NOT FOR CONSTRUCTION — unreviewed design.** This document was written with AI assistance. **It has not been reviewed by a qualified high-voltage or medical-device engineer, and nothing in it has been built, simulated in circuit form or measured.** It has had one independent paper review, itself AI-assisted. It may contain errors that would make a built circuit dangerous or useless. The storage capacitor in this design holds about 190 joules at about 2,000 volts, far above the level that stops a heart, and keeps it after the power is off. This is a design on paper for review and criticism (SAFETY.md §2a, D-13). It contains no assembly steps and no bring-up procedures, and it does not tell anyone to build or power anything (CLAUDE.md rule 2, D-10, D-18). Nothing here is safe or ready for use on a person or animal.

How to read the tags:
- **[CALC]** arithmetic done for this document; the working is shown.
- **[EST]** an engineering estimate; its basis is stated.
- **[BK]** background knowledge not re-checked against a source this time.
- **[UNVERIFIED]** looked for and not confirmed, or depends on a datasheet that could not be opened.
- **[SIM]** can only be settled by circuit simulation or, later, measurement (§16 lists them all).
- **"read 27 Nov 2026"** = datasheet text read that day; **"page only"** = product page, listing or search extract.

Decision numbers (D-xx) and questions (Q-xx) refer to [DECISIONS.md](../DECISIONS.md). Review findings are R-xx.

---

## Plain-language summary

**What this is.** The parts study chose *kinds* of parts. This document turns them into a circuit design that several engineers can draw at the same time. Version 0.2 fixes the problems an independent review found (§R): six that would have made the device dangerous or unable to shock, and ten serious ones.

**How a shock now works (D-59).** Before a shock, three things must all be true: the main controller asks for it, the supervisor permits it, and both discharge paths are held off. Each of the controllers' signals is a constantly changing signal that only running software can make. When all three are true a supply rail for the output stage comes up, a relay moves the pads from the heart-signal electronics to the output stage, and the main controller's timer runs the two-phase shock. **From the first moment current flows, plain hardware takes over:** it keeps the output stage powered until the planned pulse ends, whatever the two controllers do next, but never for longer than a fixed hardware limit of about 25 thousandths of a second, which nothing can extend. Then it turns the switches off in a fixed order, returns the pads, and refuses another shock until both controllers have withdrawn their signals. This matters because during a shock the pad reading must go "open" and the capacitor voltage must fall; if those changes could withdraw permission, the pulse would be cut off at a random moment. Cancel, a stopped controller, overheating and over-voltage can still stop a shock **before** it starts; once it has started they cannot, by design, and the worst extra energy that can then reach the patient is worked out in §B13.6.

**The big choices.**

1. **Charging the capacitor.** A transformer-based "flyback" charger (Analog Devices LT3751) lifts the 6–8 V battery to 1,950 V in 4–6½ s. The software turns it on and off at the target; behind that, **three separate hardware limits** keep the capacitor below its 2,200 V rating, and a fourth (D-25) **cuts the charger's power** whenever either discharge path is not proven held off. In child mode a further hardware limit keeps the capacitor below about 1,380 V even if all software fails.
2. **Emptying the capacitor by default.** Two parallel paths, each made of transistors that **conduct unless actively held off**. The main controller alone holds path A off; the supervisor alone holds path B. If either stops, its path empties the capacitor within about 2 seconds. "Held off" is now proven by a light-coupled sensor that can only light up if the holding voltage is really there, so a broken sensor reads "not held" (the review found the first version did the opposite).
3. **Delivering the shock.** Four 3,300-volt silicon-carbide transistors. The two "bottom" ones do the precise timing with catalogue drivers. The two "top" ones float up to 2,000 V and are switched slowly through small made-to-order transformers; a buzzing signal on the transformer keeps them on, silence turns them off. The first version used a transformer driver chip with a built-in multi-millisecond "soft start" that made the timing impossible; it is replaced by a simple gated driver that starts on the first cycle.
4. **A self-test that reaches the shock path (D-61).** A "test permit" lets the device exercise the interlock and all four output switches **only while the capacitor is below about 60 V, enforced by a hardware comparator**, and **only into an internal test resistor: the hardware makes it impossible for the patient relay to move in test mode.** This lets the device find a shorted switch, a stuck relay or a faulty interlock before a rescue, not during one.
5. **Protecting the heart-signal electronics.** At rest a relay connects the pads to the heart-signal electronics; only an energised relay connects them to the output stage. The relay now has its own coil supply, a hardware limit on how long its coil can be on, enough stored energy that it cannot drop out in the middle of a shock, and a contact that tells the main controller where it is.
6. **Two controllers that cannot fail together.** ST STM32U585 (main) and NXP MCX A153 (supervisor), different makers (D-39), each with its own regulator, crystal, capacitor-voltage measurement, pad-resistance measurement, capacitor temperature sensor, cancel contact and alarm (supervisor).

**What it costs.** The fixes add parts: a 22-chip discrete logic sheet for the shock sequence, an eighth discharge transistor pair, optocouplers, a second rectifier, a freewheel diode, a relay coil supply. Estimated **+45–80 g** over the parts study's lower-risk set (§14), which puts that set at about **880–910 g**, at or just over the top of the 780–900 g accepted for a first prototype (D-43). That is listed as a strained decision (§18).

**What is still unknown.** The same top risks as before (switch pulse ratings, the made-to-order transformers, the patient relay, charger interference with compression sensing, spacing at 2 kV), plus several timing and transient questions that **only circuit simulation can settle** — §16 lists each one precisely.

**One board or two.** Still expected to be two: a low-voltage board (sheets 01–09) and a high-voltage/patient board (sheets 10–16).

---

## Contents

- §0 Requirements baseline; departures from the parts study
- §B1 Power · §B2 Controllers · §B3 Front ends · §B4 Charger · §B5 Storage and discharge · §B6 Output bridge · §B7 Patient relay and pad interface · §B8 Interlock · §B9 Sensing · §B10 User interface · §B11 Durable memory and timekeeping · §B12 Creepage and partitioning · **§B13 The shock sequence (D-59)** · **§B14 Test-permit mode (D-61)**
- §11 Rail tree · §12 Power budget · §13 Single-fault table · §14 Made-to-order parts and weight · §15 Test points · §16 Open technical questions (including everything that needs simulation) · §17 Questions for the owner · §18 Strained decisions · §19 Not verified · §20 Sources
- **§F Firmware requirements from hardware** · **§R Response to review**

---

## §0. Requirements baseline

The whole of DECISIONS.md is the baseline. The ones that shape the circuit most:

| Decision | What it requires of the circuit | Where |
|---|---|---|
| D-02, D-23 | Biphasic truncated exponential from one 100 µF capacitor at 1,950 V adult / 1,150 V child; phases 3–12 ms, 0.5 ms gap; about 5 Ω internal resistance assumed | §B5, §B6, §B13 |
| D-03, D-04 | 150 J adult, 50 J child (stored 190.1 / 66.1 J) | §B4 |
| D-05, D-57 | One pad set; the pad connector reports which pad set is fitted | §B7 |
| D-06, D-19 | Fully automatic: no shock button | §B8, §B13 |
| D-07, D-44 | Rechargeable, two cells | §B1 |
| D-09, D-45, D-46 | No radio; sealed dock contacts for charging and records | §B1.4 |
| D-15, D-49 | Weekly full-energy and daily low-voltage self-test into the internal load; battery under load | §B1.3, §B5, §B14 |
| D-25 | Charger cut off in hardware whenever the discharge path is active | §B4.6, §B8 |
| D-26 | Supervisor's own alarm | §B10.2 |
| D-28 | Two voltage sensors, each 1 % + 7 V | §B9.1 |
| D-29 | Charge during analysis, ≤ 7 s | §B4 |
| D-32, D-36 | Pads sensed to about 250 Ω; compression through pad resistance | §B3 |
| D-39 | Two Cortex-M33 controllers from different makers | §B2 |
| D-41, D-53 | Made-to-order capacitor, 100 µF, ≥ 2,200 V | §B5 |
| D-42 | Made-to-order parts get a published specification | §14 |
| D-48, D-51, D-54–D-56 | Six temperature points, one read only by the supervisor; charge limits | §B9.2, §B11 |
| **D-59** | **Consent starts a shock; hardware completes it under a non-extendable cap; lockout until consent is removed** | **§B13** |
| **D-60** | **Mode-switch contacts disagree → hardware ceiling = adult; readback to the main** | **§B4.4, §B10.4** |
| **D-61** | **Low-voltage test permit exercises the interlock and output switches** | **§B14** |

### Departures from the parts study (version 0.4)

| Parts study | This design | Why |
|---|---|---|
| Charger 44–50 W in, 7.6 A average / 15 A peak | 53 W in, **9.2 A average / 21 A peak** at a cold 5.8 V pack [CALC §B4.2] | LT3751 boundary-mode arithmetic for ≤ 7 s |
| One discharge path, three resistors | **Two independent paths**, each **four** switches and three resistors | No single open part may leave the capacitor charged; four devices keep margin with one shorted (R-22) |
| Patient isolation by two switches in series | A **patient relay** (pads at rest on the front ends) | Charging during analysis (D-29); a shorted high-side switch would otherwise put 2 kV on a pad during CPR (§B7.1) |
| ADS1292R, MAX30009 as upgrade | ADS1292R for the main; **MAX30009 for the supervisor's own reading** | hal.rs requires the supervisor's own pad measurement |
| Appendix B2 gate isolation on all four switches | Made-to-order transformers on the **two high-side switches only** | Low-side drivers' barriers see only volts |
| Voltage-chain budget 0.87 % | 0.92 % (main, thick film) / 0.51 % (supervisor, thin film) | Re-done with chosen parts |

---

## §B1. Power

### B1.1 Pack, protection and fuel gauge (sheet `01_power_battery`)

Two 18650 cells in series (D-44; Molicel P28A class), midpoint tap, an NTC on each cell, **ADI MAX17320** protector and fuel gauge with external protection MOSFETs, main fuse **F101 25 A** time-lag to `VBAT_SYS`.

| Item | Part | Rating vs need | Source |
|---|---|---|---|
| Cells | Molicel INR-18650-P28A × 2, made into a pack by a pack assembler with both NTCs and a keyed connector | 20.2 Wh | parts study §5 |
| Protector + gauge | **ADI MAX17320** (2S–4S, protector with external N-FETs, programmable over-current and short-circuit thresholds and delays, SHA-256, self-discharge detection) | −40 to +85 °C [BK] | analog.com/en/products/max17320.html (page only) |
| Protection MOSFETs | 2 × 30 V N-channel, ≤ 2 mΩ, ≥ 40 A pulse (capture choice) | 9.2 A average, 21 A peak drawn through the charger's bulk capacitors (§B4.3) | — |
| F101 | 25 A time-lag, ≥ 32 V | backup to the protector | [UNVERIFIED part] |

**Protection coordination (R-16).** The order in which protections act on a fault downstream of `VBAT_SYS` must be: **the faulty load's own limiter first, the pack protector never** (except for a short on `VBAT_SYS` itself). So:
- The charger's supply is fed through an **eFuse with a fast current limit** (TPS25982, §B4.3) set at about 13 A; a shorted primary switch or an inrush is limited there in microseconds and the eFuse latches off.
- Each regulator has its own small fuse (§B1.6).
- The MAX17320's discharge over-current and short-circuit thresholds are set **above** what any limited branch can draw: over-current ≥ 30 A with a millisecond delay, short-circuit ≥ 60 A with ≥ 100 µs delay [EST; set from the datasheet by WP-A], and its fault response set to **auto-retry**, not latch, so a transient does not leave the device dead [UNVERIFIED that the part offers this; WP-A to confirm].
- **Residual risk:** a hard short on `VBAT_SYS` itself (upstream of every branch fuse) trips the pack protector and everything goes dark, including the supervisor's alarm. Listed in §13 (fault 40) and §17.

**Reverse and short protection.** Keyed internal connector (no series reverse MOSFET: it would cost 0.2–0.4 W at 21 A peaks for a fault the connector prevents [EST]); short circuits per the coordination above.

### B1.2 Cell temperatures

Cell NTCs → MAX17320 thermistor inputs; the main reads them over I²C (`TempPoint::Cell1/Cell2`).

### B1.3 Battery-under-load test (`MainHardware::battery_under_load`)

A pulse-rated **0.82 Ω** resistor switched by an N-MOSFET (`BAT_LOAD_EN`). [CALC] 10.2 A (86 W) at 8.4 V, 8.8 A at 7.2 V — the order of the charger's 9.2 A. 100 ms puts 6.3–8.6 J into the resistor. A **hardware RC time limit** turns the load off after ≤ 250 ms even if `BAT_LOAD_EN` sticks. Resistor: ≥ 25 J single pulse, one pulse an hour. (Using the charger as the load was rejected: boundary-mode power is low at low capacitor voltage.)

### B1.4 Dock contacts and battery charger (sheet `02_power_dock`)

Flat contacts on the device, springs in the dock: `DOCK_5V`, `DOCK_GND`, `DOCK_DATA` (Q-47 option (a) assumed: the dock is the reader; `DOCK_D2` reserved for option (b)).

| Function | Part | Why |
|---|---|---|
| Input protection | TI **TPS25947** eFuse (2.7–23 V, over-voltage cut-off, reverse-current blocking [BK]) after a 6.0 V TVS | faulty dock; reverse blocking keeps battery voltage off the contacts |
| Charger | TI **BQ25887** 2-cell boost charger, 2 A, cell balancing, JEITA thermistor input, −40 to +85 °C | 2S from 5 V |
| Charge window | BQ25887 TS network for a 0–45 °C hard stop [UNVERIFIED that TS reaches 45 °C; else an external comparator gates CE] | D-48 |
| Data | open-drain driver powered from `VDOCK`, receiver = Ioff Schmitt buffer on `3V3_MAIN`; **pull-up in the dock** | contact dead when undocked |
| Dock detect | `DOCK_DET_N` switched by `VDOCK` | wakes the device |

### B1.5 Battery charging limits

Three layers stop charging outside 0–45 °C: BQ25887 TS (hardware), MAX17320 charge-temperature protection (hardware), firmware via `CHGR_CE_N`, which defaults to *enabled* so a docked, switched-off device still charges.

### B1.6 Rails and their independence (sheet `03_power_rails`)

| Rail | Part | Input | Peak / typical | Notes |
|---|---|---|---|---|
| `3V3_MAIN` | TI **TPS629210** buck (3–17 V, 1 A [BK]) | VBAT_SYS via own 1 A fuse | 80 / 25 mA | main domain |
| `3V3_SUP` | **separate** TPS629210 | VBAT_SYS via own 1 A fuse | 60 / 15 mA | supervisor domain only |
| `5V0_SYS` | TI **TPS62130** buck (3–17 V, 3 A [BK]) | VBAT_SYS via own 3 A fuse | 1.2 A / 50 mA | audio, LT3751 VCC, interlock logic, `BRIDGE_5V` source |
| `3V3_AON` | TI **TPS70933** LDO (to 30 V, ≈ 1 µA quiescent [BK]) | VBAT_SYS | 5 mA / 10 µA | wake timer, ready latch, display |
| `BRIDGE_5V` | output of the interlock (§B8) | 5V0_SYS | 0.6 A for ≤ 150 ms | only while a shock or test is allowed |
| `BRIDGE_3V3` | LDO on sheet 12 with **≥ 2 ms hold-up** | BRIDGE_5V | 60 mA | bridge logic and driver input sides (§B13.5) |
| `1V8_SUP` | LDO on sheet 16 | 3V3_SUP | 5 mA | MAX30009 |

**A loss of `5V0_SYS` is a safe failure:** the charger and bridge lose their supply; the interlock's "held" inputs lose their pull-ups and read "not held"; the hold-pump buffers run from the **controllers'** rails, not from `5V0_SYS`, so they are unaffected and the paths behave as their controllers command. The patient relay coil does not hang on `5V0_SYS` (§B7.2).

`5V0_SYS` must stay ≥ 4.9 V down to a pack voltage of 5.2 V so the LT3751's VCC lock-out (4.8 V, §B4.3) does not trip: TPS62130 in 100 % duty drops about 0.5 A × 0.2 Ω ≈ 0.1 V [EST] (R-20).

### B1.7 Power-on and wake logic

| Enable | Turned on by (hardware OR) |
|---|---|
| `3V3_MAIN` | power button **or** wake-timer interrupt latch **or** dock present **or** `MAIN_KEEP` |
| `3V3_SUP` | power button **or** `SUP_WAKE` (main, for scheduled self-tests) **or** `SUP_KEEP` |
| `5V0_SYS` | `EN_5V0` (main; default off) |

The power button enables both controller regulators directly, with no logic between (D-26). How the device is switched on in an emergency is owner question 1 (§17).

### B1.8 Always-on domain

`3V3_AON` powers only the RV-3028-C7 wake timer, the ready latch and LED flasher, the memory display and its blanking buffer, and the wake-line pull-ups. Signals from it into a controller go through ≥ 100 kΩ or an Ioff buffer (contract rule R7).

---

## §B2. Controllers

### B2.1 Choice

| Role | Part | Why |
|---|---|---|
| Main | **ST STM32U585ZIT6** (Cortex-M33 160 MHz, 2 MB flash, **LQFP144**) | Qualified Rust target (D-39); enough advanced timers; **the 144-pin package** gives headroom after the review added nine signals (v0.1 chose LQFP100, which is now full) [UNVERIFIED: I/O count per package; DS13086 not opened] |
| Supervisor | **NXP MCXA153VLH** (Cortex-M33 96 MHz, LQFP64, −40 to +125 °C, 12-bit ADC with a 16-bit mode; NXP datasheet, page only) | Different maker (D-39) |

Both run an independent hardware watchdog (≤ 50 ms timeout, own oscillator), whose reset puts every pin in its reset state, which the contract's default resistors make safe.

### B2.2 Main controller pin budget

| Group | Nets (contract names) | Pins |
|---|---|---:|
| Front end | `AFE_SCLK, AFE_MOSI, AFE_MISO, AFE_CS_N, AFE_DRDY_N, AFE_START, AFE_RESET_N` | 7 |
| Voltage ADC | `HVM_SCLK, HVM_MOSI, HVM_MISO, HVM_CS_N, HVM_DRDY_N` | 5 |
| Flash + F-RAM (local) | — | 5 |
| Display | `LCD_SCLK, LCD_SI, LCD_SCS, LCD_BLANK` | 4 |
| I²C | `I2C_M_SCL, I2C_M_SDA` | 2 |
| Audio | `I2S_BCLK, I2S_LRCLK, I2S_SD, AMP_SD_N` | 4 |
| Link | `LINK_M2S, LINK_S2M` | 2 |
| Dock | `DOCK_TX, DOCK_RX, DOCK_DET_N` | 3 |
| Battery | `GAUGE_ALRT_N, BAT_LOAD_EN, CHGR_INT_N, CHGR_CE_N` | 4 |
| Charger | `CHG_SESSION, CHG_RUN, CHG_DONE_N, CHG_FAULT_N, MODE_HW_RB_N` | 5 |
| Interlock | `HOLD_M_CLK, SHOCK_REQ, BRIDGE_ARMED_N, HOLD_RB_A_N, HOLD_RB_B_N` | 5 |
| Bridge | `CMD_HA, CMD_LA, CMD_HB, CMD_LB` (one advanced timer), `RELAY_CMD, BTEST_CMD, I_SHOCK` (ADC), `SEQ_ACT_M_N, SEQ_DONE_M_N, BR_FAULT_N, K_REST_N` | 11 |
| User interface | `CANCEL_M_N, MODE_ADULT_N, MODE_CHILD_N, PWR_BTN_N` | 4 |
| Ready latch | `READY_D, READY_CLK` | 2 |
| Power | `MAIN_KEEP, SUP_WAKE, EN_5V0, RTC_INT_N` | 4 |
| Pads | `PAD_PRESENT_N, PAD_ID_1W, NTC_PADS` (ADC) | 3 |
| Debug + clocks | SWDIO, SWCLK, HSE ×2, LSE ×2 | 6 |
| **Total** | | **76 + 6 = 82** of about 110 I/O on LQFP144 [UNVERIFIED] |

### B2.3 Supervisor pin budget

| Group | Nets | Pins |
|---|---|---:|
| Consents | `HOLD_S_CLK, PERMIT_CLK, TEST_PERMIT_CLK` | 3 |
| Status in | `SEQ_ACT_S_N, SEQ_DONE_S_N, OVP_TRIP` | 3 |
| Cancel | `CANCEL_S_N` | 1 |
| Link | `LINK_M2S, LINK_S2M` | 2 |
| Pad-impedance front end | `SBZ_SCLK, SBZ_MOSI, SBZ_MISO, SBZ_CS_N, SBZ_INT_N` | 5 |
| Voltage ADC | `HVS_SCLK, HVS_DOUT, HVS_PDRST_N` | 3 |
| Capacitor NTC (ADC) | `NTC_CAP_S` | 1 |
| Power | `SUP_KEEP` | 1 |
| Alarm (local) | piezo A/B, fault LED | 3 |
| Debug + crystal | SWDIO, SWCLK, XTAL ×2 | 4 |
| **Total** | | **26** of about 50 I/O on LQFP64 [UNVERIFIED] |

`SHOCK_REQ` is no longer read by the supervisor (R-31): the supervisor learns the sequence state from `SEQ_ACT_S_N`/`SEQ_DONE_S_N`, which have a HAL purpose (§F).

### B2.4 Clocks

Main: 16 MHz crystal (core and bridge timer) + 32.768 kHz crystal. Supervisor: **its own crystal** (frequency per the MCX A153 oscillator range, WP-B). Neither depends on the wake timer.

### B2.5 Debug and programming

Separate SWD pads per controller, reachable only with the enclosure open; nothing on the dock can program either; no debug path joins the two controllers.

### B2.6 The message link

UART pair `LINK_M2S`, `LINK_S2M`, 1 Mbit/s. Each receiver: Ioff Schmitt buffer on its own rail, 1 kΩ series, 100 kΩ pull-down — an unpowered sender reads as a continuous "break" (no messages), which the 250 ms heartbeat timeout treats as stopped.

### B2.7 What a stuck or dead controller does to each shared line

Rule: the default resistor makes the safe state when the driver is unpowered, in reset or floating; **consent lines are dynamic**, so a stuck pin of either level is "no consent".

| Line | Driver | Default (sheet) | Unpowered / reset / stuck gives |
|---|---|---|---|
| `HOLD_M_CLK`, `HOLD_S_CLK` | M / S | 100 kΩ pull-down (09) | no clock → that path conducts within ≤ 10 ms (§B5.4) |
| `PERMIT_CLK`, `TEST_PERMIT_CLK` | S | 100 kΩ pull-down (09) | no permit, no test permit |
| `SHOCK_REQ` | M | 100 kΩ pull-down (09) | no request |
| `CHG_SESSION`, `CHG_RUN` | M | 100 kΩ pull-downs (09) | charger supply off, CHARGE low |
| `CMD_*`, `RELAY_CMD`, `BTEST_CMD` | M | 100 kΩ pull-downs (12) | no switch, relay, or test relay command; and the sequencer refuses anything out of order (§B13) |
| `EN_5V0` | M | 100 kΩ pull-down (03) | 5 V off: everything shock-related unpowered |
| Main frozen **with its timer still toggling `CMD_*`** (R-4) | — | — | the sequencer allows **one** phase-1 and **one** phase-2 interval per rail cycle and a 25 ms latched cap; no permit is given because the supervisor sees the heartbeat stop |

---

## §B3. Heart-signal and pad-impedance front ends (sheets 15, 16)

### B3.1 Topology

```
 pads ── patient relay K1401 (at rest: pads → PT_FE_A/B) ──┬── main front end (15): ADS1292R
                                                           └── supervisor front end (16): MAX30009
```

Each front end has its own taps on `PT_FE_A`/`PT_FE_B`: per pad a **sense tap** (TT Electronics **DPCR2512 51 kΩ** defibrillator pulse resistor) and a **drive tap** (two **4.7 nF 3 kV C0G** capacitors in series, 2.35 nF / 6 kV, plus a pulse-rated **1 kΩ** resistor). The capacitor-coupled drive is what lets a 3 V chip push 0.3 mA into the patient through defibrillation protection (impedance 1.06 kΩ at 64 kHz; an external 5 kV pulse pushes only 12 µC through it [CALC]).

**Drive-tap resistor (R-28).** A 1 kΩ DPCR may not exist (only 20 kΩ and 51 kΩ found). Requirement instead: 1 kΩ, **≥ 1.5 kV for 10 µs** single-pulse withstand, thick-film anti-surge class (e.g. Vishay CRCW-HP or Bourns CMP class) [UNVERIFIED part]; WP-H names the part with its pulse curve.

### B3.2 Defibrillation protection

External 5 kV / 32 µF / 500 µH test pulse (IEC 60601-1 8.5.5, from MEDTEQ via research notes [UNVERIFIED]): sense resistor 98 mA peak, ≈ 1 J; drive tap ≈ 1.3 A for ≈ 10 µs; energy-reduction share ≈ 4 J of ≈ 360 J [CALC, unchanged from v0.1, confirmed by the review].

**Clamps (R-28).** Diodes "to the analog rails" would push the test current into regulators that cannot sink it. Revised: each tap is clamped **to analog ground by a bidirectional TVS whose breakdown is below the chip's rail** (for example 3.3 V class for the 3 V ADS1292R inputs), with low-capacitance steering diodes only if the TVS capacitance spoils the measurement; and each front-end supply rail carries its own **shunt clamp** (TVS) rated for the worst injected current (98 mA × 2 sense taps). No gas tubes across the pads (they would divert an external shock). The 51 kΩ resistors are service items.

### B3.3 Disconnect during the shock, recovery

The patient relay disconnects the front ends for the whole sequence (§B13). After return, ADS1292R runs at gain 3 (±0.8 V) during recovery; the real recovery time is to be measured against IEC 60601-2-4 201.105 [UNVERIFIED].

### B3.4 Main impedance measurement (D-36)

**TI ADS1292R** (24-bit, internal respiration modulation/demodulation on channel 1, −40 to +85 °C). Channel 2 = ECG, 250/s; channel 1 = impedance, decimated to 125/s. Carrier **64 kHz** [UNVERIFIED option], above the charger's ≈ 43 kHz top fundamental.

[CALC, unchanged] 0.34 mA RMS excitation (assuming ≈ 3 V p-p drive [UNVERIFIED]); 102 µV per 0.3 Ω; ≈ 1 mΩ RMS noise (0.5–10 Hz); ≈ 0.3 mΩ per step — against the aed-cpr requirement of < 10 mΩ RMS and ≤ 2 mΩ per step.

**Cross-loading (R-17).** The supervisor's drive taps (≈ 4.2 kΩ across the pads at 64 kHz) make the main read ≈ 6 % low at 250 Ω, and vice versa (≈ 3 kΩ at 128 kHz). Requirement: both chips' drive outputs are **high-impedance when idle**, and **each channel is calibrated with the other chip in a fixed, specified state** (firmware: the other chip running its normal carrier). The supervisor's permit range is widened by this calibration uncertainty in firmware (§F).

### B3.5 Pad presence

Plugged in: the connector's presence loop. On a patient: impedance in range.

### B3.6 Supervisor's own measurement

**ADI MAX30009** (16 Hz–500 kHz drive, 20-bit, SPI, 25-ball WLP), carrier **128 kHz** (an even multiple of a 64 kHz square wave, whose spectrum has only odd harmonics, so the demodulators reject each other — confirmed by the review). Temperature range [UNVERIFIED]; fallback ADI AD5940 (LFCSP).

### B3.7 Charger noise (a known risk)

Unchanged in substance: the boundary-mode charger's frequency sweeps from a few kHz to ≈ 43 kHz and its harmonics cross both carriers during every charge. Measures: carriers above the fundamental; star return at NT1101; VTRANS LC filter and eFuse soft start; physical separation; firmware knows when charging and can pause the CHARGE pin (≈ 20 % slower charging); fallback: a synchronised fixed-frequency charger. **Measurement only** (§16).

---

## §B4. High-voltage charger (sheet `10_hv_charger`)

### B4.1 Topology

**ADI LT3751** flyback capacitor charger, **charge mode** (FB grounded) for adult, **FB regulation** used only as the **child ceiling**, primary-side output trip, made-to-order 1:50 transformer. (LT3751 datasheet rev D read 27 Nov 2026 from a distributor mirror.)

### B4.2 Power stage sizing [CALC, unchanged; confirmed by the review]

| Quantity | Value |
|---|---|
| Turns ratio | 1:50 |
| Peak primary current | **21.2 A** (5.0 mΩ sense, 106 mV typical / 100–112 mV) |
| Primary inductance | **7.5 µH ± 10 %** (window 6.1–8.6 µH) |
| Charge time to 1,950 V | 5.8 V, η 0.75: 5.35 s; 5.5 V, η 0.65: 6.43 s; 8.4 V, η 0.75: 4.07 s; child 1,150 V: 2.16 s |
| **Corner cases (R-19)** | 5.5 V, η 0.65, C = 105 µF, current limit at its 100 mV minimum (20.0 A): **7.2 s**; at η 0.60: **7.8 s**. The "≤ 7 s" target (D-29) is met at nominal and typical-cold conditions, **not at every tolerance corner**; the firmware's 12 s limit is not reached |
| Battery current at full charge | 9.2 A average, 53 W in (5.8 V) |
| Primary RMS / switch peak | 11.4 A / 52 V + spike |
| Switching frequency at full charge | 32 kHz (5.8 V) – 43 kHz (8.4 V) |
| Rectifier reverse voltage | 2,111 V (trip) + 50 × 8.4 V = **2,531 V** + ringing |

### B4.3 Parts

| Ref | Part | Notes |
|---|---|---|
| U1001 | **LT3751IUFD** (I grade, −40 to +125 °C) | VCC = `5V0_SYS`; LVGATE + HVGATE tied (VCC ≤ 8 V) |
| Q1001 | **Infineon BSC070N10LS5** (100 V logic-level), one or two in parallel | RDS(on) at 4.5 V [UNVERIFIED] |
| R1001 | 5.0 mΩ, 2 W, Kelvin | |
| T1001 | made-to-order transformer (§B4.5) | |
| **D1001, D1002** | **two Sanken UX-F5B in series** (8 kV, 0.35 A, trr ≤ 150 ns each) | **R-15:** either diode alone blocks the full 2,531 V (3.2× margin), so one shorted diode leaves the capacitor isolated from the secondary |
| **R1004** | **HV fusible resistor, ≈ 10 Ω**, in series with the rectifiers: opens within 10 ms at ≥ 5 J, withstands ≥ 3 kV across the open element | **R-15:** limits and then breaks the discharge into the secondary if both diodes fail [UNVERIFIED part] |
| **U1002** | **TI TPS25982** eFuse (2.7–24 V, ≈ 15 A class, adjustable fast current limit, dV/dt soft start, latch-off variant [BK, UNVERIFIED]) on `VBAT_SYS` → `VTRANS` | **R-5, R-16:** the VTRANS switch is now a current-limited, soft-started eFuse (limit ≈ 13 A; the 21 A peaks come from the bulk capacitors behind it) |
| F1001 | 15 A time-lag fuse in series with U1002 | backup if U1002 fails short |
| U1003 | comparator, `VTRANS_GOOD` > 5.0 V (with hysteresis), plus a ≥ 1 ms delay | **R-5:** CHARGE may rise only after VTRANS is above the LT3751's UVLO1 |
| Bulk, filter | 4 × 47 µF X7R + 2 × 330 µF polymer after U1002, input LC filter | ripple ≈ 6.7 A RMS |
| UVLO/OVLO | VTRANS UVLO1 **4.7 V** (69.8 kΩ), OVLO1 9.0 V (155 kΩ); **VCC UVLO2 4.8 V (71.5 kΩ)** (R-20: above the 4.75 V minimum), OVLO2 5.6 V (87.5 kΩ) | VUVLO = 1.225 V + 50 µA·R |
| RVOUT / RBG / RDCM / RVTRANS | 40.2 kΩ / **931 Ω, 0.1 %** / 18.2 kΩ / 40.2 kΩ, all 0.1 % | primary-side trip **2,111 V nominal** (§B4.4) |
| FB network | top 10 × 4.02 MΩ **0.1 % HV thin film**, bottom **36.9 kΩ 0.1 %**; an N-MOSFET from FB to ground, **on by default** (adult = FB grounded = charge mode); `MODE_CHILD_HW` turns it off → child regulation | §B4.4 |
| Primary clamp | RCD/TVS at ≈ 75 V | |

### B4.4 How charging is controlled, and the threshold stack-up

**Control split (R-5).**
- `VTRANS_EN` (from the interlock) = `CHG_SESSION` AND `HOLD_OK` AND NOT `TEST_MODE`. It enables the eFuse, whose dV/dt soft start limits inrush into the bulk capacitors. **The firmware does not toggle the session for regulation.**
- `CHARGE_EN` (from the interlock) = `CHG_RUN` AND `HOLD_OK` AND NOT `TEST_MODE`; on sheet 10 it is ANDed with `VTRANS_GOOD` (after its delay) to drive the LT3751 **CHARGE** pin. CHARGE therefore never rises before VTRANS is above UVLO1, so the LT3751 cannot latch its under-voltage fault at start.
- If the LT3751 does latch FAULT (supply out of range), firmware lowers `CHG_RUN` for ≥ 1 ms and raises it again (the datasheet's "toggling the CHARGE pin reactivates switching").
- Both `VTRANS_EN` and `CHARGE_EN` fall immediately when `HOLD_OK` falls (D-25 unchanged).

**Regulation at the target.** The LT3751 in charge mode runs until CHARGE falls or the primary-side trip fires. The main controller regulates by switching **`CHG_RUN`** (CHARGE pin) on its own voltage reading: off at the target, on again 3 V below it. Overshoot per step is ≈ 0.2 V (205 V/s at full charge × 1 ms loop) and per switching cycle ≈ 0.009 V; droop through the 6 MΩ bleed is ≈ 3.4 V/s [CALC], so the charger restarts about once a second while holding. **The resulting tolerance of the true voltage is the sensor's own (1 % + 7 V) plus 3 V hysteresis**, which is what config.rs's at-plan window assumes.

**The FB pin (R-7).** The LT3751's FB pin does not "stop" at a threshold: below 1.16 V (1.12–1.20 V) it is in charge mode; between that and 1.34 V (1.29–1.38 V) it **regulates FB to 1.22 V (1.19–1.25 V)** with reduced current; above 1.34 V it stops switching. FB bias current is up to 300 nA *out of* the pin (datasheet, read). No adult FB setting can keep both its charge-to-regulation hand-over above the operating window (≈ 2,008 V true) and its regulated maximum below 2,200 V (ratio needed 1.16, available 1.096 — the review's arithmetic, confirmed). So:
- **Adult (and default):** FB grounded by the MOSFET → pure charge mode. The ceilings are the primary-side trip and the over-voltage comparator.
- **Child:** FB divider active, ratio k = 1 + 40.2 MΩ / 36.9 kΩ = **1,090** → hand-over ≥ 1,207 V, regulated 1,283–1,377 V, stop ≤ 1,521 V (table below).
- **D-60:** the MOSFET is **on (adult) by default**; only the mode switch's second pole, closed in CHILD, turns it off. Contact open, wire broken or switch between positions → **adult ceiling**. The actual state is read back on `MODE_HW_RB_N` (low = child ceiling active). The main compares it with `MODE_ADULT_N`/`MODE_CHILD_N`; disagreement → recorded, "not ready", and in a rescue the firmware still sets the dose by the position it trusts per D-60.

**Threshold stack-up [CALC]** (sensor error 1 % + 7 V from D-28; at-plan window 1 % + 12 V from config.rs; resistor ratios ±0.2 % (0.1 % parts); FB bias error up to 0.9 % (300 nA × 36.9 kΩ = 11 mV); turns ratio ±1 %; RVOUT comparator 0.955–1.005 V (datasheet)):

| Threshold | Adult (true volts) | Child (true volts) |
|---|---|---|
| Firmware target (reading) | 1,950 | 1,150 |
| **Top of firmware at-plan window, true** = (target + window + 7)/0.99 | **2,008.6** | **1,192.4** |
| Child FB: charge→regulation hand-over (lowest) = (1.12 − 0.011) × 1,090 × 0.998 | — | **1,206** (14 V above the window) |
| Child FB: regulated point (range) = 1.19…1.25 × 1,090 × (0.989…1.011) | — | **1,283 – 1,377** |
| Child FB: no-load stop (highest) = 1.38 × 1,090 × 1.011 | — | **1,521** |
| Primary-side trip RVOUT: 0.98·N·RVOUT/RBG − V_D (nominal 2,111); band ±3.7 % (comparator ±2.5 %, N ±1 %, ratio ±0.2 %) | **2,033 – 2,189** (24 V above the window) | same (above child FB) |
| **Over-voltage comparator** on the supervisor chain, 2,160 V nominal, ±1.0 % (thin-film ratio after calibration 0.2 %, drift 0.3 %, reference 0.1 %, comparator offset and hysteresis 0.4 %) | **2,138 – 2,182** | same |
| Firmware over-voltage limit (both controllers; config.rs 2,200 V reading; **recommended 2,150 V**, R-30) | reading 2,150 → true ≤ 2,179 | same |
| **Capacitor rating** (D-41, Appendix A) | **2,200 V continuous; 2,420 V for 10 s (test voltage on every part)** | same |

Checks, both directions: (1) **no hardware layer can stop a normal charge**: every hardware threshold's lowest value is above the top of the firmware window (adult 24 V margin to RVOUT, 129 V to OVP; child 14 V margin to the FB hand-over); (2) **with any single control layer failed, the capacitor stays below 2,200 V**: adult worst case is the OVP comparator's top, 2,182 V; child worst case the FB regulator's top, 1,377 V. (3) The primary-side trip's top (2,189 V) is below the rating even if the OVP comparator has failed; only if both fail does the capacitor depend on its 2,420 V test margin. **The OVP comparator is the only hardware layer whose whole band is below the rating and that empties the capacitor** (R-7).

Child-mode energy at the FB regulator's top (1,377 V): ½ × 100 µF × 1,377² = 94.8 J stored, against 66.1 J planned [CALC]; the supervisor refuses a permit outside the at-plan window, so this is a two-fault case.

### B4.5 Transformer T1001 (made to order; refines Appendix B1)

Unchanged from v0.1: 1:50 ± 1 %; 7.5 µH ± 10 %; ≤ 10 % inductance loss at 25 A, 70 °C; 11.4 A RMS for 7 s, 14 charges at 132 s, 20 in 1,300 s; leakage ≤ 1 % of LPRI; secondary self-capacitance ≤ 15 pF; working insulation ≥ 2,200 V DC for 10 years; test ≥ 4.4 kV DC 60 s; partial-discharge inception ≥ 3.3 kV after humidity soak; −20 to +70 °C; class B; ≤ 15 cm³, ≤ 30 g. **Turns-ratio tolerance ±1 % is now load-bearing** (it sets the primary-side trip band).

### B4.6 Summary of charger protections

| Layer | Acts on | Software needed? |
|---|---|---|
| Firmware regulation (main) | CHARGE pin at the target | yes |
| Firmware over-voltage (both controllers) | release hold → dump | yes |
| Child FB regulator | regulates ≤ 1,377 V in child mode | no |
| Primary-side trip | stops each charge at 2,033–2,189 V | no |
| Over-voltage comparator → both stacks forced on → `HOLD_OK` false → `VTRANS_EN`, `CHARGE_EN` low | empties the capacitor at 2,138–2,182 V | no |
| D-25 gate (`HOLD_OK`) | removes CHARGE and VTRANS whenever either path is not proven held | no |
| eFuse U1002 | limits and latches off a charger short or inrush | no |

---

## §B5. Energy storage, discharge by default (sheet `11_hv_storage`)

### B5.1 Capacitor connection

C1101 (Appendix A) off-board on HV terminals J1101; two case NTCs on a separate LV connector J1102 (one per controller, D-56). **The capacitor negative is `HV_RTN`, joined to `GND` at one star point by NT1101 and NT1102 — two parallel net ties built as separate wide bonds** so one open bond does not separate them (R-9). Everything that must keep working if they did separate (the hold pumps, the hold sensing, the over-voltage force) is coupled by capacitors or light, not by a DC connection to `GND` (§B5.4).

### B5.2 Why depletion-mode stacks

Unchanged reasoning (v0.1): a normally-closed relay would make onto 2 kV at ≈ 18× its switching rating, needs coil power to hold open, and has a spring; enhancement MOSFETs need power to conduct; a bridge-based dump shares the shock path's failure modes. **Depletion-mode MOSFETs conduct with no power at all.**

### B5.3 Two paths

```
HV_CAP_P ─┬─ R1101–R1103 (3 × 1.5 kΩ) ─ stack A: Q1101–Q1104 (4 × 1,000 V depletion) ─ HV_RTN   held off only by the MAIN
          ├─ R1104–R1106 (3 × 1.5 kΩ) ─ stack B: Q1105–Q1108                          ─ HV_RTN   held off only by the SUPERVISOR
          ├─ bleeder 8 × 2.49 MΩ ─────────────────────────────────────────────────── HV_RTN   always
          ├─ main sense string (40.2 MΩ) ─ VCAP_M_TAP ─ bottom leg ───────────────── HV_RTN
          └─ supervisor sense string (39.8 MΩ) ─ VCAP_S_TAP ─ bottom leg ────────── HV_RTN
```

Dump timing [CALC, unchanged]: both paths 2.25 kΩ, τ = 0.225 s, 0.89 s to 38 V; one path 4.5 kΩ, τ = 0.45 s, 1.79 s (limit 3 s). A failed path is found by the τ fit on every dump, and each path is also exercised **alone** in every self-test discharge by releasing the holds one at a time (§F).

### B5.4 Holding a stack off, and proving it (R-6, R-9, R-10, R-22)

**Devices.** Four **IXTT10N100D2** per stack (1,000 V; RDS(on) 1.5 Ω; VGS(off) −2.5 to −4.5 V; IDSS 10 µA at 25 °C / 250 µA at 125 °C at −5 V gate; Ciss ≈ 5.3 nF; Qg ≈ 200 nC — datasheet copies, confirmed by the review). A smaller-die D2-series part (e.g. IXTY1R6N100D2, DPAK) would suffice for the 0.45 A dump current [UNVERIFIED that it exists with the same ratings].

**Static voltage sharing.** A 4 × 10 MΩ string across the stack; the gate of upper device k connects through 100 kΩ to the tap at k/4 of the stack voltage. Each upper device then regulates its own source to sit about |VGS(off)| above its gate tap, so each device blocks V/4 ± about 2 V (the VGS(off) spread) — set by the string, not by leakage (the review's analysis, agreed). [CALC] At the OVP comparator's top (2,182 V): 545 V per device; **with one device shorted: 727 V** (rating 1,000 V, 27 % margin) — the v0.1 three-device stack would have had 1,091 V (R-22).

**Dynamic sharing.** 1 nF, 1 kV C0G across each string resistor (capacitive divider ≥ 10 × the devices' output capacitance [requirement; value from simulation]) so the stack shares voltage during the release transition. **[SIM]**

**Gate clamps.** A **bidirectional 15 V TVS across gate and source of every device** (eight per pair of stacks). When the stack conducts, the upper gates are pulled towards their taps (hundreds of volts above their sources); the clamp holds 15 V and the 100 kΩ gate resistor limits the current (≤ 7 mA at 727 V).

**Hold bias (bottom device), referenced to the stack's own source (`HV_RTN` at the stack).**
- **Clock:** `HOLD_A_DRV` (buffer powered by `3V3_MAIN`, driven by the main's `HOLD_M_CLK`) and `HOLD_B_DRV` (buffer on `3V3_SUP`, from `HOLD_S_CLK`). Accepted band **1–5 kHz** (nominal 2 kHz, square, 50 % ± 20 % duty); a two-pole RC low-pass (corner ≈ 8 kHz) before the pump makes a fast free-running signal (≥ 100 kHz) produce < 1 % of the pump drive, and DC produces none.
- **Pump:** three-stage inverting Dickson pump, AC-coupled (the coupling capacitors carry the clock across any difference between `GND` and `HV_RTN`), output **≈ −7.4 V at the full load** [CALC: −(3 × 3.3 − 6 × 0.3) = −8.1 V unloaded; output impedance ≈ n/(f·C) = 3/(2 kHz × 2.2 µF) = 680 Ω; 1.1 mA load → −7.35 V].
- **Load (stated honestly, R-9):** hold-sense optocoupler LED ≈ 1.0 mA (below), gate-source resistor **1 MΩ** (7 µA), release-switch bias ≈ 50 µA, TVS leakage ≤ 1 µA: **≈ 1.1 mA**.
- **Reservoir:** 3.3 µF on the gate node. Ripple at 2 kHz: 1.1 mA / (2 kHz × 3.3 µF) = **0.17 V** p-p [CALC].
- **Hold-up after the last clock edge:** from −7.35 V to the sense threshold (≈ −6.2 V) at ≈ 1.1 mA: 3.3 µF × 1.15 V / 1.1 mA ≈ **3.5 ms** [CALC] — 3.5× the firmware's maximum permitted gap of 1 ms (R-11).
- **Active release (new):** a normally-on depletion-mode small-signal MOSFET (e.g. Infineon **BSS126**, 600 V, depletion [BK]) across the bottom gate and source, held **off** by its own small negative bias derived from the same clock through a short time constant (≈ 1 ms). When the clock stops, that bias collapses in ≈ 3 ms and the release switch turns on, discharging the 3.3 µF reservoir through ≈ 0.5 kΩ (τ ≈ 1.6 ms). **The stack is conducting within ≤ 10 ms of the last clock edge** [EST; **SIM**]. This also covers an open gate-source resistor (the review's question: an open resistor alone would otherwise let the gate charge hold the stack off for a long time).
- **Release transition energy:** with the active release the bottom gate crosses its linear region in ≈ 1–2 ms; at most V²/(4R) = 2,008²/(4 × 4.5 kΩ) = 224 W for the stack, ≈ 56 W per device, ≈ 0.1 J per device [EST]. **To be checked against the FBSOA curve and simulated [SIM].**

**Proving "held" (R-6).** One **optocoupler per stack** (low-current type, CTR specified at 1 mA; e.g. Vishay VOS618A class [UNVERIFIED part]): its LED, a 4.7 V zener and 1.5 kΩ are in series from the stack's source (`HV_RTN` at the stack) to the bottom gate. **The LED can only light when the gate is more negative than ≈ −6 V** [CALC: 1.1 V LED + 4.7 V zener + small drop], i.e. when the stack is really biased off (VGS(off) is −4.5 V at worst). Its phototransistor pulls `HOLD_A_HELD_N` (or `_B_`) **low**; the pull-up is on the interlock sheet. So **"held" exists only as an LED actually lit by the hold bias**: an unsoldered, open or unpowered opto, an open pull-up path to the reader or a missing bias all read **"not held"**. The only fail-to-"held" fault is a shorted phototransistor, which the self-test catches by releasing each hold in turn and watching `HOLD_RB_A_N`/`_B_` (§F); it is a latent single fault that needs a second fault (main frozen with charger on) to matter (§13 fault 9).

**Over-voltage force (R-10).** `OVP_TRIP` (0/3.3 V on `3V3_SUP`, referenced to `GND`) drives the LEDs of **two optocouplers in series** (one per stack, through a resistor on sheet 11), whose phototransistors short each stack's bottom gate reservoir to its source (collector at source, emitter at the negative gate node). Galvanic, no level-shift problem, default (no LED current) = no force. Discharge of the reservoir through the phototransistor (≈ 10 mA) takes ≈ 3 ms [CALC: 3.3 µF × 7.4 V / 10 mA = 2.4 ms]; the release switch is not needed for this path.

**Removed (R-9):** the v0.1 option of driving pump A from a 5 V buffer on `5V0_SYS`. Both pumps run only from their own controller's 3.3 V rail.

### B5.5 Stresses [CALC]

| Item | Both paths | One path | Rating |
|---|---|---|---|
| Peak dump current per path | 0.45 A | 0.45 A | ≥ 10 A |
| Volts per device held (OVP top 2,182 V) | 545 V | — | 1,000 V; 727 V with one device shorted |
| Energy per dump resistor, 190 J dump | 32 J | 63 J | Ohmite EY 400 J [UNVERIFIED repetition; fallback 885SP 250 J, 1 kV] |
| Peak power per resistor | 300 W | 300 W | — |
| Voltage per resistor | 670 V | 670 V | EY [UNVERIFIED]; 885SP 1,000 V |

### B5.6 Permanent bleeder

8 × 2.49 MΩ HV chip resistors (19.9 MΩ), its own string. Everything always connected (bleeder 19.9 MΩ, two sense strings ≈ 40 MΩ each, the two 40 MΩ sharing strings): ≈ **6.65 MΩ** [CALC: 1/(1/19.9 + 1/40.2 + 1/39.8 + 1/40 + 1/40)], τ ≈ 665 s, 2,008 V → 50 V in ≈ **41 min**; bleeder alone 2.0 h; loss while charged 2,008²/6.65 MΩ ≈ **0.61 W** (≈ 2 % of the charger output).

### B5.7 Self-test discharge and residual: how measured

Both controllers read the capacitor throughout. The main fits τ after each release. In every self-test discharge the holds are released **one at a time** (supervisor first: path A alone must give τ ≈ 0.45 s and `HOLD_RB_B_N` must go high; then the reverse on the next test), so each path and each hold sensor is proven independently (§F).

---

## §B6. Output bridge power stage (sheet `13_hv_bridge`)

### B6.1 Arrangement

```
HV_CAP_P ─┬─ R_S 3.9 Ω ─ L_S 30 µH ─┬─ HV_BUS ─┬─ QHA ─ HV_OUT_A ─┬─ QLA ─┐
          │                          │          └─ QHB ─ HV_OUT_B ─┴─ QLB ─┴─ LS_SRC ─ shunt 2 mΩ ─ HV_RTN
          └──────── D_FW (cathode) ◄─┘ (anode at HV_BUS)       snubber RC: HV_BUS ─ HV_RTN
Phase 1: QHA on beforehand, QLB times it.   Phase 2: QHB on beforehand, QLA times it.
```

### B6.2 Switches

**Microchip MSC080SMA330B4N** (3,300 V; RDS(on) 85 mΩ typical / 106 mΩ maximum at 25 °C; IDM 107 A; **Tj max 150 °C**; leakage up to 35 µA at 3,300 V; Qgd 51 nC; TO-247-4 Kelvin source; datasheet DS00005914A as read by the reviewer, R-25). **No avalanche or short-circuit rating** — the design must never avalanche them (R-2). The datasheet does publish a forward SOA with a 10 ms line and a transient thermal impedance curve, so the millisecond pulse question can now be partly answered on paper [EST: ≈ 400 W average per conducting device over 10 ms with Zθ ≈ 0.1–0.15 °C/W gives tens of kelvin; to be computed properly from the curves]. Lifecycle: distributor listings disagree [UNVERIFIED]. Alternative: GeneSiC/Navitas G2R120MT33J.

### B6.3 Gate drive (R-3: SN6505B replaced)

| Switches | Drive | Timing |
|---|---|---|
| QLA, QLB (low side, reference `LS_SRC`) | TI **UCC5350MC** isolated driver with Miller clamp; output side **+20/−5 V** from Murata **MGJ2D052005SC** on `BRIDGE_5V`; input side on `BRIDGE_3V3` (with hold-up) | ≈ 1 µs on/off; gate resistor sets ≈ 1 V/ns at the drain |
| QHA, QHB (high side, floating) | **Gated push-pull transformer drive (new):** a free-running 400 kHz two-phase clock (flip-flop divider from an oscillator on sheet 13, running whenever `BRIDGE_3V3` is up) is ANDed with the gated command (`G_HA` / `G_HB` from sheet 12) and drives a dual N-MOSFET push-pull stage into the centre-tapped primary of the made-to-order transformer T1301/T1302 (centre tap on `BRIDGE_5V`). **No soft-start: energy transfer starts on the first clock half-cycle after the command.** Secondary: full-wave rectifier → 18 V zener-limited → 100 Ω → gate, with **100 nF added gate-source capacitance**, an active **PNP turn-off** (base driven by the collapse of the rectified voltage), and a **10 kΩ gate-source bleed** as backup (R-23) | below |

The flip-flop guarantees equal half-cycles, so the transformer is not driven into saturation; gating is applied to both phases at once on a clock edge.

**Timing budget [EST from the RC values; SIM]:**

| Event | Time |
|---|---|
| High-side turn-on: first rectified pulse ≈ 1.3 µs after `G_H*`; 105 nF through 100 Ω from ≈ 19 V (τ ≈ 10.5 µs) to 15 V | **≈ 20 µs** (≤ 40 µs with tolerances) |
| High-side turn-off: carrier stops; PNP discharges 105 nF through ≈ 10 Ω | **≈ 5–10 µs** to < 1 V (≤ 20 µs worst case) |
| Same, PNP failed open: 10 kΩ bleed, τ = 1.05 ms | 18 V → 1.5 V in ≈ 2.6 ms (backup only) |
| Low-side on/off | ≈ 1 µs |
| Lead (high side on before its phase starts) | **100 µs** (≥ 2.5 × the worst high-side turn-on) |
| Phase 1 end → `G_HA` off and `G_HB` on | +10 µs (hardware: a high-side command passes only after its own leg's low side has been off ≥ 10 µs) |
| QHA fully off | ≤ T1 + 30 µs |
| QHB fully on | ≤ T1 + 50 µs |
| Phase 2 start (`CMD_LA`), set by firmware | T1 + 500 µs (D-02 gap); hardware lets `G_LA` pass only after `G_HA` has been off ≥ **150 µs** (5× the worst turn-off) |
| Phase 2 end → `G_HB` off | +10 µs |

The 0.5 ms gap is met with ≥ 300 µs margin (QHB on at ≤ 50 µs; lockout lets QLA start at ≥ 160 µs). **No change to D-02's 0.5 ms gap is needed.**

**dv/dt immunity.** At phase 2's start QLA pulls `HV_OUT_A` from ≈ 2 kV to 0 in ≈ 2 µs; the off QHA's gate rises by Qgd / (Cgs + 100 nF) = 51 nC / 105 nF ≈ **0.49 V** [CALC, R-25 figure] against a 1.9 V minimum threshold at 25 °C (lower hot, ≈ 1.5 V [BK]) — adequate. When a high side turns on (slowly, at zero current), the low-side drain sees a slow rise; the low side holds −5 V with an active Miller clamp. QHB turns on during the gap at near-zero drain-source voltage (its output already sits at the bus through the patient loop), so it switches no voltage at all.

**A stuck-on high-side switch.** Shorted QHA: `OUT_A_TAP` reads the capacitor voltage at the next charge → the firmware refuses to shock; if missed, phase 2 (QLA) would short leg A → over-current latch (§B13.4) and coil energy into D_FW. QHA's PNP failed open: QHA still ≈ 10 V gate at 0.5 ms → QLA start = leg short → latch → phase 2 lost. Both are found earlier by the test-permit bridge test (§B14), which turns each high side off and checks that no current flows when the opposite low side then turns on into the test load.

### B6.4 Shoot-through and sequence rules

Moved to the sequencer (sheet 12, §B13.3). The power-stage sheet only receives four gated enables (`G_HA`, `G_LA`, `G_HB`, `G_LB`).

### B6.5 Series element, freewheel path (R-2) and internal resistance

**The v0.1 claim that the coil's energy "returns to the capacitor through the high-side body diodes" was wrong** (an N-channel MOSFET's body diode conducts from source to drain, i.e. from the output *into* `HV_BUS`, the same direction as the coil current; there was no return path, so each phase end would have avalanched a switch). Fix:

- **D_FW (D1301): freewheel diode across R_S + L_S, anode at `HV_BUS`, cathode at `HV_CAP_P`.** When a low side opens, L_S's current circulates HV_CAP_P → R_S → L_S → HV_BUS → D_FW → HV_CAP_P and decays with τ = L/R = 30 µH / 3.95 Ω ≈ **7.6 µs**; `HV_BUS` rises only by the diode's forward drop above `HV_CAP_P`. Part: a **3.3 kV SiC Schottky/MPS diode** (GeneSiC/Navitas 3,300 V 5 A MPS class, Mouser listing, page only [part code and surge rating UNVERIFIED]): no reverse recovery; reverse voltage up to the full capacitor voltage when `HV_BUS` is pulled low in a short (2,182 V against 3,300 V); pulse current = the coil current at turn-off (≤ 65 A normally, ≤ ≈ 130 A at the over-current trip), decaying in ≈ 8 µs.
- **The patient loop** (pad-lead inductance, ≈ 1–2 µH [EST]) freewheels through the opposite high-side body diode (e.g. at phase 1's end: `HV_OUT_B` → QHB body diode → `HV_BUS` → QHA → `HV_OUT_A` → patient) and decays in the patient's resistance.
- **Wiring inductance** between D_FW, the bulk of the capacitor connection and the switches: an **RC snubber `HV_BUS`–`HV_RTN`** (order 10 nF / 3 kV + 10 Ω [EST; values by simulation]) close to the switches.
- **Every turn-off case:** normal phase end, over-current trip, idle-current trip, cap expiry and rail collapse all end with the **low side opening first** (§B13.5), so all use the same D_FW path. A high side opening under current happens only in fault cases (a shorted low side); its energy goes into that high side's die, which is stated in §13.
- **Energy at the 90 A trip** (peak ≈ 130 A with the coil slowing the rise): ½ × 30 µH × 130² ≈ **0.25 J**, dissipated in R_S and D_FW, not in an avalanching switch [CALC].
- **Remaining spike sizes** with real layout inductance: **[SIM]**.

**Internal resistance [CALC].** Other than R_S: capacitor ESR 0.13 Ω; two switches hot 2 × 0.15 Ω; shunt 0.002 Ω; relay contacts 2 × 0.05 Ω [UNVERIFIED]; L_S winding ≈ 0.08 Ω (below); leads/connector/pads ≈ 0.2 Ω [EST] → ≈ **0.81 Ω**. **R_S = 3.9 Ω** → total ≈ **4.7 Ω** (the model's 5 Ω, D-23). Hard short: 2,182 / 4.7 ≈ **465 A**; with the coil the current never gets there before the trip (≈ 130 A peak).

**L_S (R-26, re-derived).** 30 µH, ≥ 450 A without losing more than 20 % of its inductance. Air-core one-layer 55 turns on 20 mm × 30 mm (v0.1) does **not** fit with ≤ 50 mΩ of wire (needs ≈ 88 mm of length — the review is right). Options [CALC by Wheeler's multi-layer formula, EST sizes]: **air-core, two layers, ≈ 43 turns of 1.0 mm² on a 33 mm former, ≈ 35 mm long: ≈ 30 µH, ≈ 75 mΩ, ≈ 35 cm³, ≈ 40 g** (copper ≈ 4.5 m × 8.9 g/m); or **gapped powder-core toroid** (sizing for 450 A without saturation left to the maker): **≈ 10–15 cm³** [EST]. The made-to-order specification (§14) states performance, not construction. The 75 mΩ is inside the internal-resistance budget above.

### B6.6 Outputs, current sense, test load

- **Output dividers** (also the pull-downs): 4 × 249 kΩ + bottom leg per output → `OUT_A_TAP`, `OUT_B_TAP` (1:1,000). Idle outputs sit at ≤ ≈ 35 V (35 µA leakage × 1 MΩ, R-25); a shorted high side reads the full capacitor voltage. **Tap clamps go to `GND` (zener/TVS at the tap), not to a reader's rail** (R-21).
- **Shunt** 2 mΩ (pulse-rated) `LS_SRC` → `HV_RTN`.
- **Fast over-current comparator** directly across the shunt (RC-filtered, ≤ 0.5 µs response), threshold **90 A** → `OC_DET_N` to sheet 12.
- **Idle-current comparator**, threshold **5 A** → `IDLE_I_DET_N` to sheet 12 (current flowing when no low side is enabled = a shorted low side, R-14).
- **Waveform amplifier (R-24):** TI **INA240A1** (gain 20) **powered from `BRIDGE_5V`** (output swing to ≈ 4.9 V), followed by a 2:3 divider → `I_SHOCK` = **26.7 mV/A**, **full scale ≈ 120 A at 3.2 V** — covers the 90 A trip and most of the ≈ 130 A peak.
- **Bridge test load:** cold-switched Cynergy3 **D-HR n/o reed relay** K1301 (10 kV stand-off, 5 V coil [part code UNVERIFIED]) and a **200 Ω** ceramic resistor R1303 across the outputs. Enabled **only in test mode** (§B14). At ≤ 60 V test voltage the current is ≤ 0.3 A (≥ 8 mV on `I_SHOCK`) and the stored energy ≤ 0.18 J. If K1301 were welded during a real shock, R1303 would take 2 kV / 200 Ω = 10 A, ≈ 20 kW for up to 15 ms (≈ 300 J worth of rating needed — it would fail); a welded K1301 is therefore detected in every self-test (outputs joined at rest, §B14.4) and the firmware refuses to shock.

---

## §B7. Patient relay and pad interface (sheet `14_patient_interface`)

### B7.1 Why a relay

Unchanged: with charging during analysis, the outputs sit beside a charged capacitor while the front ends measure; a shorted high-side switch would put up to 2 kV on a pad, through the patient, during CPR (≈ 2 mA through the 1 MΩ pull-down path [CALC]); bridge leakage would reach the front ends. The relay keeps the pads on the front ends at rest and adds a third series element.

### B7.2 The relay and its coil supply (R-12)

**K1401: DPDT HV relay, cold-switched.** At rest: pads → `PT_FE_A/B`. Energised: pads → `HV_OUT_A/B`. Reference candidate **Gigavac G17** (2C, designed as a defibrillator patient relay; coil 12 V, 12 Ω ± 10 % ≈ 12 W, intermittent 100 ms maximum on, 10 % duty — Gigavac datasheet, search extract, figures confirmed by the review). Its contact pulse rating, size and stand-off are **[UNVERIFIED]**; it remains a qualification item.

**Coil supply (new):**
- A **12 V boost on sheet 14, fed from `VBAT_SYS`** (not from `BRIDGE_5V`, which cannot supply ≈ 2.8 A), enabled only by `K_EN` from the sequencer (which requires `BRIDGE_5V` present, shock mode, `RELAY_CMD`, and the coil timer not expired, §B13).
- **Economiser:** full 12 V for ≤ 20 ms (pull-in), then hold at ≈ 50 % (≈ 3 W) [EST; hold level from the relay maker].
- **Hold-up:** a reservoir on the coil supply so that if the boost or `VBAT_SYS` fails mid-sequence, **the relay stays closed for ≥ 30 ms** (longer than the 25 ms sequence cap): ≈ 3 W × 30 ms = 0.09 J → ≈ 1.7 mF at 12 V allowing a fall to 6 V [CALC: ½C(12² − 6²) = 0.09 J].
- **On-time cap:** the coil is cut by hardware **90 ms** after it is energised (non-retriggerable, latched until the rail is removed), inside G17's 100 ms; the sequencer lets a sequence start only within **≤ 60 ms** of coil energisation, so a 25 ms sequence always ends before the coil cap (§B13.3).
- **Fast release:** zener + diode suppression (≈ 24 V) instead of a plain diode, since release time is in the timing budget.
- **Budget:** pull-in ≈ 12 W / 0.85 / 7.2 V ≈ 2.0 A from `VBAT_SYS` for 20 ms [CALC]; this is drawn outside the eFuse-limited charger branch and inside the pack protector's limits.

**Position sense (new).** **The relay must have a force-guided low-voltage auxiliary contact** (a third, mechanically linked changeover or a normally-closed contact closed only at rest) → `K_REST_N` (low = at rest). Read by the main (pull-up on 04) and by the sequencer (Ioff buffer on 12, no pull). If the qualified relay has no such contact, this becomes a made-to-order requirement (§14). The sequencer refuses to start a shock sequence unless the relay has **left** rest for ≥ 10 ms, and refuses a test sequence unless the relay **is** at rest.

**Requirements:** carry 65 A peak exponential, ≤ 15.5 ms; ≥ 100 A for 20 ms; 20 shocks in 1,300 s; fault 465 A for ≤ 5 µs; ≥ 5 kV across open contacts, between poles and contact to coil; contact resistance ≤ 50 mΩ, stable within 10 mΩ; operate and release ≤ 15 ms; ≥ 20,000 cold operations; −20 to +65 °C.

### B7.3 Pad connector, identity, presence

J1401 made to order: `PT_PAD_A`, `PT_PAD_B` (HV) and three separated LV contacts `PAD_ID` (1-Wire), `PAD_ID_RTN`, `PAD_PRESENT`. 1-Wire EEPROM in the pad plug (ADI **DS2431** class: 64-bit unique ID = `pad_set_id`, D-57). Series resistor and TVS on each LV pad line. Pad-area NTC `NTC_PADS` on this sheet.

### B7.4 Leakage

At rest the patient touches only the front ends (DC blocked in the drive taps, nanoamps in the sense taps); auxiliary current ≈ 0.34 mA RMS at 64 kHz plus the supervisor's carrier, against an IEC limit of several mA at those frequencies [UNVERIFIED limit]. The bridge reaches the patient only while the relay is energised (≤ 90 ms, hardware-capped).

---

## §B8. Hardware interlock (sheet `09_interlock`)

### B8.1 Signals and detectors (R-11)

| Signal | From | Encoding | Detector on 09 |
|---|---|---|---|
| `SHOCK_REQ` | main | static | Ioff buffer on `3V3_MAIN` |
| `PERMIT_CLK` → `PERMIT_OK` | supervisor | dynamic, 1–5 kHz | band-limited pump detector on `3V3_SUP` |
| `TEST_PERMIT_CLK` → `TEST_OK` | supervisor | dynamic, 1–5 kHz | identical, separate detector on `3V3_SUP` |
| `HOLD_M_CLK` → `HOLD_A_DRV` | main | dynamic | buffer on `3V3_MAIN` (pump on 11) |
| `HOLD_S_CLK` → `HOLD_B_DRV` | supervisor | dynamic | buffer on `3V3_SUP` (pump on 11) |
| `HOLD_A_HELD_N`, `HOLD_B_HELD_N` | 11 | opto, low = held | 10 kΩ pull-ups to `5V0_SYS` on 09; `HOLD_OK` = both low |
| `LOWV_OK` | 07 | static, high = capacitor < 60 V | pull-down on 09 |
| `SEQ_HOLD` | 12 | static, high = sequence running | pull-down on 09 |
| `CHG_SESSION`, `CHG_RUN` | main | static | Ioff buffers on `3V3_MAIN` |

**Dynamic detector specification (each `_CLK`):**
- **Accepted band:** 1–5 kHz square (nominal 2 kHz). Two-pole RC low-pass (corner ≈ 8 kHz) then AC coupling (corner ≈ 200 Hz) then a diode pump into a hold capacitor, then a Schmitt buffer.
- **Rejection:** at ≥ 100 kHz (a pin left on a clock or free-running timer output) the pump sees < 1 % amplitude → no output [CALC: two poles at 8 kHz attenuate 100 kHz by ≈ (100/8)² ≈ 156×]; DC gives none.
- **Rise:** output valid ≤ 3 ms after the first edge.
- **Hold-up:** stays valid **≥ 3 ms** after the last edge (3 × the firmware's maximum permitted gap of 1 ms).
- **Collapse:** output invalid **≤ 8 ms** after the last edge.
- Supply from the consenting controller's rail, so an unpowered controller gives no output.
- **Firmware rule (§F):** square wave 1–5 kHz from code that stops when the control loop stops; no gap between edges longer than 1 ms; never a free-running timer.

### B8.2 Bridge power

```
5V0_SYS ─ SW1 (EN = SHOCK_REQ) ─ SW2 (EN = PERMIT_OK, or TEST_OK·LOWV_OK·NOT PERMIT_OK) ─ SW3 (EN = HOLD_OK) ─ SW4 (EN = WIN_OPEN) ─┬─ BRIDGE_5V
5V0_SYS ─ SW5 (EN = SEQ_HOLD · NOT TEST_MODE) ─────────────────────────────────────────────────────────────────────────────────┘
```

Six TI **TPS22918** load switches (5.5 V, 2 A, quick output discharge [BK; confirmed by the review]); every EN has a 100 kΩ pull-down. SW2 is two switches in parallel (SW2a permit, SW2b test permit), so each can be proven to block (§B14). SW5 is the D-59 sequence hold (§B13.2).

**Maximum permit window (R-4, R-11; new).** `WIN_OPEN` is high from power-up until a latch `WIN_EXPIRED` is set **150 ms** after the rising edge of the consent chain (SW1–SW3 all on); the timer is an RC charged from that edge into a comparator whose output **sets a latch** — non-retriggerable. The latch clears only after **both** `SHOCK_REQ` and the permit/test detectors have been absent **continuously for ≥ 50 ms** (a second RC), so a toggling consent cannot restart the window. A shock sequence must **start** while `WIN_OPEN` is high; once started, SW5 carries the rail to the end (§B13).

**`TEST_MODE`** = `TEST_OK` AND `LOWV_OK` AND NOT `PERMIT_OK` (§B14) → to 12.

**Charger gates:** `VTRANS_EN` = `CHG_SESSION` · `HOLD_OK` · NOT `TEST_MODE`; `CHARGE_EN` = `CHG_RUN` · `HOLD_OK` · NOT `TEST_MODE` → to 10.

**Readbacks to the main:** `BRIDGE_ARMED_N` (open drain, low while `BRIDGE_5V` present), `HOLD_RB_A_N`, `HOLD_RB_B_N` (open-drain copies of each stack's held state).

### B8.3 Truth tables against interlock.rs

Let *m*, *s* = main/supervisor hold clocks running; *r* = `SHOCK_REQ`; *p* = `PERMIT_OK`; *e* = `CHG_RUN` (with `CHG_SESSION`); A_held, B_held = opto lit.

**discharge_path_active(m, s) = NOT(m AND s):** path A conducts unless *m*, path B unless *s*. Hardware `HOLD_OK` = A_held AND B_held, and A_held implies *m* (the LED lights only from the bias that only *m* produces). Same four rows as v0.1: active unless both hold. **Stricter than interlock.rs:** `OVP_TRIP` forces both paths on; a pump, sensor or opto fault reads "not held".

**bridge_may_fire(r, p, discharge_active) = r AND p AND NOT discharge_active — at the START of a sequence:**

| r | p | HOLD_OK | WIN_OPEN | BRIDGE_5V via SW1–SW4 | interlock.rs |
|---|---|---|---|---|---|
| 0 | x | x | x | off | false |
| x | 0 | x | x | off | false |
| x | x | 0 | x | off | false |
| 1 | 1 | 1 | 0 | off | true — **hardware stricter**: the window has expired |
| 1 | 1 | 1 | 1 | **on** | true |

**After the sequence has started (D-59):** SW5 keeps the rail up while `SEQ_HOLD` is high, regardless of r, p and `HOLD_OK`, until the sequence ends or its cap expires. This is a deliberate departure from a level-by-level reading of `bridge_may_fire`: **the function governs the start; the hardware completes the started pulse.** The firmware's `interlock.rs` model and the simulator need the same change (§F).

**charger_may_run(e, discharge_active) = e AND NOT discharge_active:**

| e | HOLD_OK | TEST_MODE | CHARGE_EN / VTRANS_EN | interlock.rs |
|---|---|---|---|---|
| 0 | x | x | off | false |
| 1 | 0 | x | off | false |
| 1 | 1 | 1 | off | true — **hardware stricter**: no charging in test mode |
| 1 | 1 | 0 | **on** | true |

### B8.4 Cancel

Two separate contacts under one cap: `CANCEL_M_N` (pull-up `3V3_MAIN`, sheet 04), `CANCEL_S_N` (pull-up `3V3_SUP`, sheet 05); only `GND` shared. Cancel withdraws consent **before** a sequence starts; after the start it has no effect (D-59, §B13.6).

---

## §B9. Sensing

### B9.1 Capacitor-voltage chains (D-28)

| | Main | Supervisor |
|---|---|---|
| String (sheet 11) | 10 × 4.02 MΩ Stackpole **HVCR2512** (thick film, < 2 ppm/V) | 12 × 3.32 MΩ Vishay **TNPV1206**-class thin film [UNVERIFIED values] |
| Bottom leg | 2 × 80.6 kΩ in parallel | 2 × 79.6 kΩ in parallel |
| Ratio | 1:998 | 1:1,000 |
| Tap clamp (on 11) | **10 kΩ series + 3.6 V zener/TVS to `GND`** (R-21: not to a reader's rail) | same |
| ADC / reference (sheet) | **TI ADS1220 + REF3425** (07) | **ADI AD7171 + ADR3425** (08), behind a zero-drift buffer that also feeds the OVP comparator |
| Extra comparator | **`LOWV_OK`**: capacitor < **60 V** (tap < 60 mV; comparator offset ≤ 1 mV → ±2 %) on `3V3_MAIN`, push-pull, high = low voltage (new, §B14) | **`OVP_TRIP`** at 2,160 V |

**Bottom-leg fault (R-27 correction).** One open resistor in a two-resistor bottom leg **doubles** the tap voltage (the reading doubles), it does not halve it. On the main chain the reading disagrees with the supervisor's → the supervisor latches. On the supervisor chain the OVP comparator then trips at ≈ 1,080 V true, so every adult charge is dumped: detected, but a "no shock" fault (§13). The clamp keeps the tap below 3.6 V either way.

**Error budget [CALC, unchanged]:** main 0.92 % + 1 V; supervisor 0.51 % + 1 V; allowance at 1,950 V 1.36 %. Life drift dominates the main chain (owner question 4).

### B9.2 Six temperature points

Unchanged: cells via MAX17320; `NTC_CAP_M` → ADS1220 (ratiometric to VREF_M); `NTC_PADS` → main ADC; board = main's internal sensor; `NTC_CAP_S` → supervisor's own ADC. NTC: 10 kΩ, B tolerance ≤ 1 %.

### B9.3 Shock status the HAL reports

From `BRIDGE_ARMED_N`, `SEQ_ACT_*_N`, `SEQ_DONE_*_N`, `BR_FAULT_N`, `I_SHOCK` and the two voltage readings (§B13.7).

### B9.4 Q-TI-16

Low-voltage test points on the scaled capacitor and output voltages, `I_SHOCK`, and the request/permit/armed/sequence/command/relay markers (§15).

---

## §B10. User interface (sheet 06; alarm on 05)

- **Speaker:** ADI **MAX98357A** on `5V0_SYS`; **4.7 kΩ series resistors in the three I²S lines** on sheet 06, and a firmware rule that the I²S lines are low whenever `EN_5V0` is low, so a powered main cannot back-power `5V0_SYS` through the amplifier's inputs (R-21).
- **Supervisor alarm (D-26):** piezo driven differentially by two supervisor pins, ≥ 85 dB at 10 cm [EST], and a red fault light, all on `3V3_SUP`.
- **Display:** Sharp **LS013B7DH05** on `3V3_AON`, inversion from `RTC_CLKOUT` (1 Hz) so the last image ("READY"/"NOT READY") stays while off. **Blanking (R-21):** the main drives `LCD_BLANK` (active high) into a **74AUP1G04 inverter powered from `3V3_AON`** (Ioff input, 1 MΩ pull-down) whose output is the display's DISP pin. Main off → input low → display **on**, and nothing back-powers the main.
- **Ready LED:** SN74AUP1G74 latch on `3V3_AON` (`READY_D`, `READY_CLK`), flashed ≈ 3 ms per second from `RTC_CLKOUT` (≈ 20 µW).
- **Cancel:** two sealed tactile switches under one guarded cap.
- **Adult/child switch (D-60):** two-pole changeover. Pole A: `MODE_ADULT_N` / `MODE_CHILD_N` to the main. Pole B: **closes in CHILD only** → `MODE_CHILD_HW` (to `5V0_SYS` through the contact) → sheet 10 turns the FB MOSFET off. Default (open) = adult ceiling. Sheet 10 reads back the actual state on `MODE_HW_RB_N`.
- **Power button:** `PWR_BTN_N`.

---

## §B11. Durable memory and timekeeping

| Function | Part |
|---|---|
| Durable store (standby.rs `DurableStore`, two copies with checksums) | Infineon **CY15B104QN** 4 Mbit SPI F-RAM [BK]; ≈ 70 KB used; **written once per wake: every minute while warm (> 40 °C), every 5 minutes otherwise** (standby.rs cadence; R-29: the budget below uses the 1-minute worst case) |
| Event record | Winbond **W25Q128JV** 16 MB |
| Wake timer | Micro Crystal **RV-3028-C7** (≈ 45 nA, 32-bit seconds counter, interrupt, 1 Hz output [BK]) |

I²C: pull-ups to `3V3_MAIN` on sheet 04; with the main off the bus is held low. Whether the MAX17320 enters a communication-shutdown mode with SDA/SCL low must be checked and, if so, disabled in its configuration [UNVERIFIED] (R-21).

---

## §B12. Creepage, clearance and partitioning

**Placeholder rule [UNVERIFIED]** (IEC 60601-1 Table 12 above 566 V and 60601-2-4 not available): HV to LV, outer layers uncoated 12.5 mm; inner 5.3 mm; inside one HV string 0.6 mm + 0.005 mm/V of the local difference; `PT_*` 4.0 mm; potted areas per the standard once bought.

**Net classes (R-18 corrected):**

| Class | Nets | Track | Clearance |
|---|---|---|---|
| `HV_PULSE` | `HV_CAP_P`, `HV_BUS`, `HV_OUT_*`, `PT_PAD_*` | 3.0 mm | 12.5 mm (placeholder) |
| `HV` | every other `HV_*` | 0.5 mm | 12.5 mm |
| `HV_FLOAT` | `HSA_*`, `HSB_*` (high-side islands) | 0.3 mm | 12.5 mm |
| **`RTN_PULSE` (new)** | `HV_RTN`, `LS_SRC` (sheet-local), the shunt Kelvin nets | 3.0 mm | 0.3 mm — **they sit at ground potential and carry 65 A pulses** |
| `PATIENT` | `PT_*` | 0.3 mm | 4.0 mm |
| `PWR_HI` | `VBAT_*`, `VTRANS` | 3.0 mm | 0.3 mm |
| `PWR`, `Default` | rails, logic | 0.5 / 0.15 mm | 0.2 / 0.15 mm |

**HV zone and barrier:** as v0.1 (charger transformer, high-side gate transformers, the low ends of the five HV strings, the stacks' bottom devices, the patient relay, the pad connector), **plus** the optocouplers on sheet 11 (hold sense, OVP force), whose LED and transistor sides lie in the same potential domain in normal use but provide separation if NT1101/NT1102 fail. `HV_RTN` = `GND`: spacing, not galvanic isolation; protection of the person holding the device is the enclosure; whether the standard needs a means of protection between the shock circuit and the dock contacts is open (§16).

**Boards:** low-voltage board sheets 01–09; high-voltage/patient board sheets 10–16. The bridge logic (12) sits on the HV board next to the power stage because its outputs drive the gate circuits directly; it is low-voltage logic.

---

## §B13. The shock sequence (D-59)

### B13.1 The sequence, step by step

| Step | Who | What | Hardware condition |
|---|---|---|---|
| 0 | firmware | Pads in range, capacitor at plan, countdown done, not cancelled — checked by each controller **before** consent | — |
| 1 | main | `SHOCK_REQ` high | — |
| 2 | supervisor | `PERMIT_CLK` running (its checks done at grant) | — |
| 3 | 09 | SW1–SW4 close → `BRIDGE_5V` up → `BRIDGE_ARMED_N` low; 150 ms window starts | `HOLD_OK`, `WIN_OPEN` |
| 4 | 12 | Power-on reset of every sequencer latch; `RAIL_READY` after ≥ 20 ms (MGJ2 and oscillator settled) | `BRIDGE_3V3` up |
| 5 | main | `RELAY_CMD` high → 12 raises `K_EN` → coil supply on → K1401 moves; coil timer (90 ms) starts | shock mode (not `TEST_MODE`) |
| 6 | 12 | waits for `K_REST_N` high (relay left rest) + 10 ms | aux contact |
| 7 | main | `CMD_HA` high (lead ≥ 100 µs) | 12 passes `G_HA` (leg B low side off) |
| 8 | main | `CMD_LB` high → **phase 1 starts**; 12 passes `G_LB` only if: `RAIL_READY`, `WIN_OPEN`, relay transferred ≥ 10 ms and coil on ≤ 60 ms, `G_HA` on ≥ 50 µs, state = idle, no fault. **At this edge 12 sets `SEQ_ACTIVE` (latched) and raises `SEQ_HOLD`; the 25 ms cap starts** | — |
| 9 | main | `CMD_LB` low (end of phase 1, T1) → state = gap | — |
| 10 | main | `CMD_HA` low, `CMD_HB` high (≥ 10 µs after LB low) | hardware lockouts |
| 11 | main | `CMD_LA` high at T1 + 0.5 ms → **phase 2**; 12 passes `G_LA` only in state = gap and after `G_HA` off ≥ 150 µs and `G_HB` on ≥ 50 µs | — |
| 12 | main | `CMD_LA` low → 12 sets **`SEQ_DONE`** (latched) and runs the turn-off order (§B13.5) | — |
| 13 | 12 | coil off at +1 ms; `SEQ_HOLD` released at +15 ms | — |
| 14 | both | read `SEQ_DONE`, `BR_FAULT_N`, `I_SHOCK`; withdraw consent | — |
| 15 | 09/12 | rail falls when consent is gone; the lockout persists until the rail has been off (power-on reset) **and** consent absent ≥ 50 ms (window latch) | one sequence per permit |

### B13.2 Consent starts, hardware completes

From step 8 to step 13, **SW5 holds `BRIDGE_5V` up whatever `SHOCK_REQ`, the permit and `HOLD_OK` do**. A controller that withdraws consent during the pulse — because the pads now read open, the capacitor voltage is falling, or for any other reason — has no effect on the pulse. That is the intended behaviour of D-59 and is stated here plainly: **once current has started, neither controller can stop the shock; only the sequence's own end, the hardware cap, or a hardware fault latch (over-current, idle current, rail under-voltage) end it.** Before step 8, withdrawal of any consent (cancel, heartbeat loss, over-temperature, over-voltage, a supervisor-latched fault) removes the rail and nothing is delivered.

`SEQ_HOLD` is not honoured in test mode (SW5 needs NOT `TEST_MODE`), so a test sequence always ends when its consents or `LOWV_OK` go.

### B13.3 Sequencer rules (sheet 12, discrete logic on `BRIDGE_3V3`)

1. **Order:** idle → phase 1 (`G_LB`) → gap → phase 2 (`G_LA`) → done. One phase-1 interval and one phase-2 interval per rail cycle. Any `CMD_LA` before phase 1, any second `CMD_LB`, any command after done is **ignored**. This bounds a main controller frozen with its timer still toggling (R-4).
2. **Leg lockouts:** `G_HA` blocked while `G_LA` is on or < 10 µs after; `G_HB` likewise against `G_LB`; `G_LA` blocked until `G_HA` has been off ≥ 150 µs; `G_LB` blocked until `G_HB` has been off ≥ 150 µs.
3. **Lead:** `G_LB` (`G_LA`) passes only after `G_HA` (`G_HB`) has been on ≥ 50 µs.
4. **Cap:** **25 ms** (± 15 %, i.e. 21.3–28.8 ms) from `SEQ_ACTIVE`; an RC started by the `SEQ_ACTIVE` latch into a comparator that **sets a latch** — **non-retriggerable and not extendable**: the RC's start is itself a latch that only a power-on reset clears. The longest planned shock is 15.5 ms + gap (DESIGN_TABLE) and config.rs allows ≤ 20 ms, so the cap's lowest value (21.3 ms) never cuts a planned shock. **(The v0.1 part, SN74LVC1G123, is retriggerable — R-4 is right.)**
5. **Start window:** a sequence may start only while `WIN_OPEN`, `RAIL_READY`, and the relay has been transferred ≥ 10 ms and energised ≤ 60 ms.
6. **Fault latches** (cleared only by power-on reset): over-current (`OC_DET_N`, 90 A); **idle current** (`IDLE_I_DET_N` > 5 A while no low-side enable is on — a shorted low side, R-14); cap expired; `BRIDGE_5V` under-voltage (< 4.5 V) during a sequence. Any of them → `BR_FAULT_N` low, `SEQ_DONE`, turn-off order.
7. **Relay coil:** `K_EN` = `RELAY_CMD` · NOT `TEST_MODE` · NOT coil-cap-expired · NOT (done + 1 ms). Coil cap 90 ms, latched.
8. **Test relay:** `BTEST_EN` = `BTEST_CMD` · `TEST_MODE`.
9. **Mode checks:** in shock mode `G_L*` require the relay transferred; in test mode they require the relay **at rest** and the test relay closed ≥ 2 ms.

### B13.4 What ends a pulse

| End | Trigger | Turn-off |
|---|---|---|
| Planned | `CMD_LA` falls (phase 2 end) | §B13.5 |
| Cap | 25 ms after start | §B13.5; `BR_FAULT_N` |
| Over-current | ≥ 90 A on the shunt | §B13.5 within ≈ 0.5 µs; `BR_FAULT_N` |
| Idle current | ≥ 5 A with no low side enabled | §B13.5; high sides off under current (fault case); `BR_FAULT_N` |
| Rail collapse | `BRIDGE_5V` < 4.5 V | §B13.5 using `BRIDGE_3V3` hold-up; `BR_FAULT_N` |

### B13.5 Turn-off order and supply collapse

**Order, every case:**
1. `t0`: both low-side enables low → UCC5350 outputs to −5 V, Miller clamp on; current stops within ≈ 1 µs; L_S freewheels through D_FW (§B6.5).
2. `t0 + 20 µs`: both high-side enables low → carriers stop → high sides off (zero current) within ≈ 10 µs.
3. `t0 + 1 ms`: relay coil off → contacts open with no current (≈ 5–15 ms).
4. `t0 + 15 ms`: `SEQ_HOLD` low → rail falls if consent is gone.

**If `BRIDGE_5V` fails mid-phase (any cause):** an under-voltage detector on sheet 12 (4.5 V) triggers step 1 at once. The logic, the UCC5350 input sides and the over-current latch run from **`BRIDGE_3V3`, which has ≥ 2 ms of hold-up** (an LDO with a reservoir sized for the logic's ≈ 60 mA [EST: ≈ 220 µF from 5 V to the LDO's dropout]), so they stay alive long enough to turn the low sides off first. The UCC5350 output sides run from the MGJ2's output capacitors and, if those sag, their own under-voltage lockout pulls the gates low (output low on UVLO [UNVERIFIED for the MC variant]) — also "off". The high-side gates are held by their 100 nF capacitors and decay through the 10 kΩ bleed (≥ 1 ms) — **so the high sides stay on until after the low sides are off**, which is the safe order. The relay coil has its own ≥ 30 ms hold-up from a different supply (`VBAT_SYS`), so it does not open under current. **[SIM: the exact ordering margins.]**

### B13.6 Worst-case energy if consent is withdrawn at any instant [CALC]

| Withdrawal arrives | Delivered |
|---|---|
| Before `SEQ_ACTIVE` (any time up to step 8) | **0 J** (rail removed; relay returns cold) |
| After `SEQ_ACTIVE` | **the planned pulse**, unchanged (150 J adult / 50 J child per DESIGN_TABLE ± model error) — withdrawal has no effect by design |
| After `SEQ_ACTIVE`, **plus** a fault that keeps a low side on (frozen timer output high) | phase 1 continues until the cap (≤ 28.8 ms): at 175 Ω, τ = 179.7 Ω × 100 µF = 18 ms, delivered ≈ 190.1 J × (1 − e^(−2 × 28.8/18)) × 175/179.7 ≈ **177 J monophasic**; at 25 Ω essentially all, ≈ 190.1 × 25/29.7 ≈ **160 J** — against 150 J planned |
| Child mode, same fault | ≈ 66.1 J × 0.96 × 0.97 ≈ **62 J** monophasic at 175 Ω, against 50 J |

The fault row is a single fault (frozen timer) after consent was correctly given; it delivers at most the stored energy, monophasic, ≤ 18 % over plan.

### B13.7 Status for both controllers

| Net | To | Meaning |
|---|---|---|
| `BRIDGE_ARMED_N` | main | rail up |
| `SEQ_ACT_M_N`, `SEQ_ACT_S_N` | main, supervisor | sequence started (current has been commanded): **consent checks are over** |
| `SEQ_DONE_M_N`, `SEQ_DONE_S_N` | main, supervisor | sequence ended (planned or by a latch) |
| `BR_FAULT_N` | main | ended by a fault latch (over-current, idle current, cap, rail under-voltage) |
| `I_SHOCK` | main | current waveform |

Open-drain outputs (pulled up on each reader's sheet); with the rail off they read "not active / not done / no fault".

---

## §B14. Test-permit mode (D-61)

### B14.1 What it is

A second permit from the supervisor (`TEST_PERMIT_CLK`, dynamic, its own detector) that can bring up `BRIDGE_5V` **only when the capacitor is below 60 V by a hardware comparator on the main chain** (`LOWV_OK`) and **only when the normal permit is absent**. `TEST_MODE` = `TEST_OK` · `LOWV_OK` · NOT `PERMIT_OK`.

### B14.2 What it can and cannot energise (all enforced in hardware)

| Can | Cannot |
|---|---|
| Bring up `BRIDGE_5V` (with `SHOCK_REQ`, `HOLD_OK`, `WIN_OPEN`) | Energise the patient relay (`K_EN` requires NOT `TEST_MODE`) — **the bridge can never be connected to the pads in test mode** |
| Close the bridge test relay K1301 (only in test mode) | Run with the capacitor ≥ 60 V: `LOWV_OK` falls → `TEST_MODE` falls → SW2's test switch opens → rail off; and SW5 (sequence hold) is disabled in test mode |
| Run a full sequence (both phases) into the 200 Ω test load | Charge: `CHARGE_EN` and `VTRANS_EN` are blocked in test mode (the capacitor is charged to the test voltage beforehand, then the charger stopped) |
| Exercise each series switch's ability to block | Coexist with a real permit (NOT `PERMIT_OK`) |

`LOWV_OK` is on the **main** chain, independent of the supervisor's reading which the supervisor's firmware uses to decide whether to give a test permit: one failed chain cannot enable test mode at high voltage.

### B14.3 What the self-test can now detect

| Fault | How |
|---|---|
| Fault 18: one interlock series switch shorted | **Blocking tests** with the capacitor < 60 V, watching `BRIDGE_ARMED_N`: test permit without `SHOCK_REQ` → must stay off (SW1); `SHOCK_REQ` without either permit → off (both SW2 switches); `SHOCK_REQ` + test permit with **one hold released** (each in turn) → off (SW3 and each stack's sensor); all present → on |
| Fault 16: shorted low-side switch (R-14) | Test sequence into the 200 Ω load at ≤ 60 V: current during the high-side lead (before the low-side command) → shorted low side; hardware idle-current latch at 5 A would not see 0.3 A, the firmware does on `I_SHOCK` |
| Each switch turns on and off; high-side turn-off and PNP | Phase 1 and phase 2 current present, correct sign; no current when the opposite low side turns on after a high side was turned off |
| Patient relay position / weld | `K_REST_N` must read "at rest" throughout (and before every analysis); a relay welded in the shock position reads "not at rest" |
| Bridge test relay welded | Outputs joined at rest: with only QHA on (test, relay commanded open), `OUT_B_TAP` follows `OUT_A_TAP` |
| Failed-open discharge path, failed hold sensor | Not test-permit faults: found in every self-test discharge by releasing the holds one at a time (§B5.7) |
| Sequencer order and cap | A test sequence with a deliberately long phase 1 command (> 30 ms) must end at the cap with `BR_FAULT_N`; a second `CMD_LB` must be ignored |

### B14.4 Test energy

At 60 V: ½ × 100 µF × 60² = **0.18 J**; peak current 60 / (200 + 4.7) = **0.29 A** (`I_SHOCK` ≈ 7.8 mV ≈ 39 counts of a 14-bit 3.3 V ADC) [CALC]. Firmware runs the test at ≈ 40–50 V.

---

## §11. Rail tree

```
Cells (2S, 5.5–8.4 V)
└─ MAX17320 FETs ─ VBAT_PACK ─ F101 25 A ─ VBAT_SYS ─┬─ TPS25982 eFuse (VTRANS_EN, ≈13 A, soft start) ─ F1001 ─ VTRANS ─ LT3751 / T1001 ─ D1001+D1002 ─ R1004 ─ HV_CAP_P
   (BQ25887 charges VBAT_PACK from VDOCK)            ├─ 0.82 Ω load (BAT_LOAD_EN, ≤ 250 ms)
                                                     ├─ 12 V coil boost (K_EN) + economiser + hold-up ─ K1401 coil
                                                     ├─ fuse ─ TPS629210 #1 ─ 3V3_MAIN ─ main MCU, flash, F-RAM, ADS1292R (filtered), ADS1220, REF3425, LOWV comparator, main consent buffers, HOLD_A_DRV buffer
                                                     ├─ fuse ─ TPS629210 #2 ─ 3V3_SUP ─┬─ supervisor MCU, AD7171, ADR3425, OVP comparator + OVP opto LEDs, alarm, permit/test detectors, HOLD_B_DRV buffer
                                                     │                                 └─ LDO ─ 1V8_SUP ─ MAX30009
                                                     ├─ fuse ─ TPS62130 ─ 5V0_SYS ─┬─ MAX98357A, LT3751 VCC, interlock logic, HOLD_*_HELD pull-ups
                                                     │                            ├─ SW1·SW2·SW3·SW4 (consent chain) ─┐
                                                     │                            └─ SW5 (SEQ_HOLD) ──────────────────┴─ BRIDGE_5V ─┬─ LDO + 2 ms hold-up ─ BRIDGE_3V3 (sequencer, UCC5350 inputs, oscillator)
                                                     │                                                                              ├─ push-pull stages ─ T1301/T1302 (high-side gates)
                                                     │                                                                              ├─ MGJ2D052005SC ─ +20/−5 V (low-side gates)
                                                     │                                                                              └─ INA240, K1301 coil
                                                     └─ TPS70933 ─ 3V3_AON ─ RV-3028-C7, ready latch + LED, memory LCD, blanking inverter, wake logic
HV_RTN ═ NT1101 ∥ NT1102 ═ GND (one star point, sheet 11)
```

| Rail | Peak | Typical in a rescue | Standby |
|---|---:|---:|---:|
| VTRANS | 13 A (eFuse limit; 21 A switch peaks from the bulk capacitors) | 9.2 A while charging | 0 |
| Coil boost input | 2.0 A for 20 ms, then ≈ 0.5 A for ≤ 70 ms | per shock | 0 |
| 5V0_SYS | 1.2 A [EST] | 0.15 A | off |
| BRIDGE_5V | 0.6 A for ≤ 150 ms [EST: MGJ2 ≈ 0.4 A, push-pull ≈ 0.1 A, logic] | 0 | off |
| 3V3_MAIN / 3V3_SUP | 80 / 60 mA | 25 / 15 mA | off |
| 3V3_AON | 5 mA | 0.1 mA | ≈ 10 µA |

---

## §12. Power and standby budget [EST unless CALC]

| Standby item | Power at 7.4 V |
|---|---:|
| MAX17320 hibernate (10–30 µA [BK]) | 74–222 µW |
| BQ25887 on battery (5–20 µA [BK]) | 37–148 µW |
| Disabled regulators, eFuse off (≈ 4 µA) | ≈ 30 µW |
| TPS70933 | ≈ 7 µW |
| RV-3028-C7 | < 1 µW |
| Memory LCD + inversion | 10–15 µW |
| Ready LED flash | ≈ 20 µW |
| Logging wakes, worst case once a minute (R-29): 2.5 mJ / 60 s | ≈ 42 µW |
| **Standby subtotal** | **≈ 0.22–0.49 mW** |
| Weekly full-energy test, averaged | 0.42 mW |
| Daily low-voltage check incl. test-permit bridge test (adds ≈ 0.2 J at 50 V and ≈ 1 s of electronics) | ≈ 0.18 mW |
| **Total** | **≈ 0.82–1.09 mW** (0.020–0.026 Wh a day) |

---

## §13. Single-fault table for the shock path (re-derived for v0.2)

M = main, S = supervisor, H = hardware, ST = self-test (switch-on or daily, §F).

| # | Single failure | What happens | Caught by | Result |
|---|---|---|---|---|
| 1 | M freezes, `SHOCK_REQ` and `CMD_*` static high | S sees heartbeat stop (250 ms), stops `PERMIT_CLK` and `HOLD_S_CLK`; path B conducts | S, H | No shock; dump; S alarm |
| 1a | M freezes with its **timer still toggling** `CMD_*` (R-4) | No permit (as 1). If it froze after `SEQ_ACTIVE`, the sequencer allows one phase 1 and one phase 2 and the latched 25 ms cap | H (sequencer, cap) | ≤ planned pulse, or ≤ 177 J monophasic if a low side stays commanded on (§B13.6) |
| 2 | M freezes with `HOLD_M_CLK` stuck (either level), charger on | No clock → path A conducts within ≤ 10 ms → `HOLD_OK` false → `CHARGE_EN` and `VTRANS_EN` low (D-25) | H | Dump; S alarm |
| 3 | S freezes, `PERMIT_CLK` stuck | Detector collapses ≤ 8 ms | H | No permit (failure to shock); M reports supervisor lost |
| 4 | S hangs while a **free-running** timer drives `PERMIT_CLK` | If its frequency is outside 1–5 kHz the detector rejects it; inside the band it would look like a permit | Firmware rule (code-generated clock) + S watchdog + M sees heartbeat stop and does not request | Needs M to request as well: two faults. Residual: documented (§16) |
| 5 | M's rhythm analysis wrongly says "shock" | S does not judge rhythm | Two-method agreement in M | **Not caught by S or H** (accuracy limit, unchanged) |
| 6 | Stack device (any of four) fails short | Remaining three share ≤ 727 V (rating 1,000 V) | Latent, safe | No effect on function (R-22) |
| 7 | Stack A fails open (e.g. bottom device) | Path B still dumps | ST: holds released one at a time (τ); every dump τ fit | Detected at the next self-test discharge |
| 8 | One dump resistor opens | That path open | as 7 | as 7 |
| 9 | Hold-sense opto's **transistor shorted** (reads "held" always) | Latent; matters only with main frozen holding the charger on **and** path A released — then path B still dumps and the charger is gated by the B sensor | ST: each hold released in turn, `HOLD_RB_*_N` must follow | Detected within a day |
| 9a | Hold-sense opto open / unsoldered / pull-up open (R-6) | Reads "not held" | H (fails safe) | Cannot charge; switch-on self-test fault |
| 10 | Hold pump fails (open capacitor or diode) | No bias → stack conducts; sensor "not held" | H | Cannot charge; detected at switch-on |
| 10a | Release switch fails open, or gate resistor open | Open release switch: the gate decays through 1 MΩ × 3.3 µF (τ = 3.3 s), so that path starts conducting ≈ 1.6 s late [CALC: 3.3 s × ln(7.35/4.5)]; open gate resistor: the release switch still empties the gate | ST τ check shows a late start of that path's decay | Detected; the other path dumps meanwhile |
| 11 | NT1101 opens | NT1102 keeps the star (two bonds); pumps and sensors are AC/opto-coupled anyway (R-9) | — | No effect; both opening is a second fault |
| 12 | FB divider top opens (child) | FB reads 0 → charge mode | Primary trip, OVP comparator, firmware | Contained ≤ 2,189 V |
| 13 | Charger primary MOSFET fails short | Battery short through the primary | **eFuse current limit → latch off** before the pack protector (R-16) | Charger dead (charge time-out); rest of device and S alarm unaffected |
| 14 | One output diode fails short (R-15) | The other diode blocks the full voltage | — | No effect; latent (found only by inspection/measurement) |
| 14a | Both output diodes short (second fault) | Capacitor discharges into the secondary | R1004 fusible resistor opens | Charger dead; transformer protected |
| 15 | Main chain reads 10 % low (string resistor shorted) | M would charge high | S disagreement (2 % + 24 V, 500 ms) → S releases, latches | Contained |
| 16 | Main chain bottom leg resistor opens (R-27) | Reading **doubles** | S disagreement → latch | No shock, detected |
| 16a | Supervisor chain bottom leg opens | S reading doubles; OVP trips at ≈ 1,080 V true | OVP dump; disagreement | Every adult charge dumped: no shock, detected |
| 17 | QHA (high side) shorted | `OUT_A_TAP` = capacitor voltage | M refuses to shock; ST bridge test | No shock; fault |
| 18 | QLB (low side) shorted (R-14) | Test sequence shows current in the lead → found at switch-on/daily before any rescue. If it fails between tests: phase 1 starts as QHA turns on; the **idle-current latch (5 A)** ends the sequence within µs; QHA then turns off under current (fault case, die stress); relay stays closed until after turn-off | ST; H idle latch | Pulse truncated at the start; no shock delivered; `BR_FAULT_N` |
| 19 | Sequencer lockout gate fails (allows HA + LA) | Leg short | Over-current latch (90 A); coil energy → D_FW (R-2) | Pulse lost; switches see ≈ 130 A for ≈ 2 µs [UNVERIFIED survival] |
| 20 | Cap timer fails (never expires) | Only matters with a stuck low side (two faults) | ST cap test | Latent; caught daily |
| 21 | One interlock series switch shorted | Two consents remain | ST blocking tests (§B14.3) | Caught daily |
| 22 | SW5 (sequence hold) shorted | Rail held up without consent; sequencer lockout blocks any second sequence; a new first sequence still needs `WIN_OPEN`, relay transfer, and the main's commands | ST: rail must fall when consent is removed (`BRIDGE_ARMED_N`) | Caught daily; no energy without M commanding |
| 23 | Window latch fails open (window never expires) | Permit can hold the rail indefinitely | ST: a test permit held 200 ms must lose the rail | Caught daily |
| 24 | `BRIDGE_5V` collapses mid-phase | §B13.5: low sides off first on `BRIDGE_3V3` hold-up; high sides held by their gate capacitors; relay held by its own hold-up | H | Pulse truncated; switches and relay turned off in safe order [SIM] |
| 25 | `BRIDGE_3V3` hold-up capacitor open + rail collapse (two faults) | Order no longer guaranteed | — | Listed |
| 26 | High-side PNP turn-off fails open (R-23) | QHA still ≈ 10 V at phase 2 start → leg A short | Over-current latch; ST bridge test (current when QLA turns on after QHA "off") | Phase 2 lost; caught daily |
| 27 | Patient relay welded in the shock position | Front ends disconnected; `K_REST_N` reads "not at rest" | M checks `K_REST_N` before every analysis | "Attach pads"/fault; no shock |
| 28 | Patient relay fails to transfer (coil, boost, aux contact never opens) | Sequencer refuses to start (relay not transferred) | H | Refused; detected |
| 29 | Relay: one pole welded at rest while the other transfers | If the auxiliary contact is linked to the moving pole the sequencer starts; one pad is still on its front end, so the only current path is through that front end's protection taps (51 kΩ sense, 2.35 nF drive): ≈ 0 A on `I_SHOCK`, ≈ 2 kV across the taps (like the external-defibrillator test) | `I_SHOCK` ≈ 0 → Fault; a force-guided contact linked to **both** poles would catch it before the shock [UNVERIFIED relay construction] | No energy delivered; front-end stress; open item (§16) |
| 30 | Coil supply fails mid-sequence | Hold-up ≥ 30 ms > 25 ms cap | H | Relay opens only after the sequence |
| 31 | Bridge test relay K1301 welded | Outputs joined through 200 Ω at all times | ST (outputs follow each other) | M refuses to shock; if missed, a real shock would destroy R1303 (≈ 20 kW): listed |
| 32 | `LOWV_OK` comparator stuck "low voltage" | Test mode possible at high voltage only if S also gives a test permit, which S's firmware refuses above 50 V on its own chain | S firmware + H | Two faults needed; patient relay still blocked in test mode |
| 33 | OVP comparator fails "no trip" | One over-voltage layer lost | Latent; primary trip (≤ 2,189 V) and firmware remain | Contained |
| 34 | Over-current comparator fails "no trip" | Leg short (another fault) not latched fast | Cap, idle-current latch | Two faults |
| 35 | Mode switch pole B open (D-60) | Ceiling = adult in child position | `MODE_HW_RB_N` ≠ pole A → "not ready", recorded | Child dose still set by firmware and checked by S |
| 36 | FB MOSFET fails short | Adult ceiling always, also in child mode | Not seen: `MODE_HW_RB_N` reads the gate drive, not the FET | Child dose still set by firmware and checked by S; listed (§16 rank 9) |
| 37 | Cancel contact to M stuck | D-31 stuck rule; S's contact disagrees | M, S | Fault |
| 38 | `3V3_MAIN` fails | M dead; path A conducts | S heartbeat; S alarm | No shock, loud |
| 39 | `3V3_SUP` fails | S dead; path B conducts; no permit | M link loss | No shock; M speaks the fault |
| 40 | Hard short on `VBAT_SYS` upstream of the branch fuses | Pack protector trips; **everything dark, including the S alarm** | — | **Silent failure** (owner question 9) |
| 41 | `5V0_SYS` fails | No charger, no bridge rail; held-inputs read "not held" | M, S | No shock; S alarm and M prompts available |
| 42 | Storage capacitor shorts internally | Charger into a short; eFuse limits | Charge time-out | No shock; fault |
| 43 | Battery removed while charged | Both hold pumps stop (rails die) → both paths conduct; bleeder as backup | H | Empties in ≈ 1 s |
| 44 | Shorted relay coil driver (coil on without `K_EN`) | Relay energised at rest → pads on bridge outputs (pulled to `HV_RTN`); front ends disconnected | `K_REST_N` "not at rest" → M fault | No energy (bridge off); detected |

Gaps carried to §16: 4 (in-band free-running permit), 29 (relay single-pole weld detection), 31 (welded test relay between self-tests), 36 (FB MOSFET short not read back), 40 (silent dark device).

---

## §14. Made-to-order and qualification parts, weight

| Ref | Part | Specification |
|---|---|---|
| C1101 | Storage capacitor | Appendix A; **fault current figure to read 470 A** (2,182 V / 4.7 Ω), not 400 A (§18) |
| T1001 | Charger transformer 1:50, 7.5 µH | §B4.5 |
| T1301, T1302 | High-side gate transformers | push-pull primary at 5 V, **400 kHz**, centre-tapped; rectified secondary 18–20 V at ≥ 30 mA; ≥ 2,200 V DC working for 10 years (target 2,750 V), ≥ 5 kV DC 60 s, partial-discharge inception ≥ 3.3 kV after humidity soak; inter-winding capacitance ≤ 10 pF; −20 to +85 °C; ≤ 2 cm³ |
| L1301 | Series coil 30 µH | ≥ 450 A with ≤ 20 % inductance loss; ≤ 80 mΩ; 2,200 V insulation to surroundings; air-core ≈ 35 cm³ or powder core ≈ 10–15 cm³ |
| K1401 | Patient relay | §B7.2, **including a force-guided LV auxiliary contact and a coil ≤ 12 W pull-in, ≤ 100 ms on** |
| J1401 | Pad connector | Appendix B3 + three LV contacts |
| — | Pad set with DS2431-class ID | Appendix B4 |
| — | Battery pack assembly; cancel cap | §B1.1, §B10 |
| R1004 | HV fusible resistor | ≈ 10 Ω, opens within 10 ms at ≥ 5 J, ≥ 3 kV across the open element (catalogue part likely; not found) |

**Weight against the parts study's lower-risk set [EST]:** second stack pair and extra two devices, pumps, optos, release switches +20–25 g; L1301 +15–40 g (powder vs air core); D1301, D1002, R1004, eFuse +4 g; patient-relay coil supply with 1.7 mF hold-up +6–10 g; sequencer logic (≈ 22 small ICs) +3 g; K1401 heavier than budgeted +5–15 g; gate transformers instead of four isolated supplies −5 g. **Total ≈ +45–80 g → lower-risk set ≈ 880–910 g with pads** (parts study 832 g). Owner question 7.

---

## §15. Test points

All on low-voltage nets; **none on `HV_*`, `HV_FLOAT` or `PT_*`** (contract R9).

| Test point | Net | Sheet |
|---|---|---|
| Capacitor voltage, both chains | `VCAP_M_TAP`, `VCAP_S_TAP` | 11 |
| Output voltages | `OUT_A_TAP`, `OUT_B_TAP` | 13 |
| Shock current | `I_SHOCK` (26.7 mV/A) | 13 |
| Consents and window | `SHOCK_REQ`, `PERMIT_OK`, `TEST_OK`, `TEST_MODE`, `WIN_OPEN`, `BRIDGE_5V` | 09 |
| Sequence | `SEQ_ACTIVE`, `SEQ_DONE`, `BR_FAULT`, the four gated enables `G_*`, cap timer node | 12 |
| Relay | `K_EN`, `K_REST_N`, coil supply | 12, 14 |
| Holds | `HOLD_A_HELD_N`, `HOLD_B_HELD_N`; each stack's bottom-gate bias through a 1:3 divider (local, reads ≈ −2.5 V when held) | 11 |
| Charger | `VTRANS_EN`, `CHARGE_EN`, VTRANS_GOOD (local), `CHG_DONE_N` | 10 |
| Faults | `OVP_TRIP`, `LOWV_OK`, `OC_DET_N`, `IDLE_I_DET_N` | 07, 08, 13 |
| Rails | every rail with GND beside it | 01–03, 09, 12 |

---

## §16. Open technical questions, ranked

| Rank | Question | What settles it |
|---:|---|---|
| 1 | SiC switches: 65 A for 3–15 ms repeatedly (FBSOA 10 ms line and Zθ now known to exist — compute), ≈ 130 A for 2 µs, high side turning off under current in fault 18 | Datasheet curves now; Microchip statement; later low-voltage pulse tests |
| 2 | Gate transformers T1301/T1302 at 2,200 V for 10 years in ≤ 2 cm³; whether the standard counts ≤ 20 ms exposures as working voltage | Makers' quotes and partial-discharge data; the standard |
| 3 | Patient relay: pulse carry rating, a force-guided auxiliary contact, coil, size; single-pole weld detection (fault 29) | Gigavac data or a made-to-order relay |
| 4 | Charger noise in the impedance/ECG channels during D-29 charging | **Measurement** |
| 5 | Spacing at 2.2 kV | Buy the standards |
| 6 | Capacitor density, mass, hot life; fault current 470 A | Maker quotes |
| 7 | Silent dark device on a `VBAT_SYS` short (fault 40) | Owner decision: e.g. a small independent alarm cell, or accept |
| 8 | In-band free-running permit (fault 4) | Firmware discipline; possibly a challenge-response permit (a code pattern the detector checks) — a design change if wanted |
| 9 | FB MOSFET short not read back (fault 36) | Read the FB pin's actual state (a comparator on FB) — cheap addition if wanted |
| 10 | Front-end protection under the 5 kV external test; drive-tap resistor part; DPCR ratings | Datasheets; the standard |
| 11 | ADS1292R drive level, MAX30009 range/package; cross-loading calibration | Datasheets; low-voltage measurement |
| 12 | D-28 over life | Owner decision (recalibration / thin film) |
| 13 | Cold battery at 9.2 A average (Q-39) | Cold tests |
| 14 | LT3751 at 1:50 and 2 kV; primary-trip accuracy depends on N ± 1 % | ADI applications engineering; transformer maker's data |
| 15 | Ohmite EY repeat rating; R1004 part; D1301 surge rating and part code | Makers |
| 16 | Protection between shock circuit and dock contacts | The standard |
| 17 | MAX17320 retry behaviour and communication shutdown | Datasheet |

**Only circuit simulation (or, later, measurement) can settle [SIM]:**
1. Voltage spikes at every turn-off with real layout and lead inductance, and the snubber values (§B6.5).
2. High-side on/off times with the gated push-pull drive, the rectifier and the 100 nF gate capacitor, across temperature (§B6.3).
3. The turn-off order margins when `BRIDGE_5V` collapses mid-phase, including UCC5350 behaviour on input-side loss (§B13.5).
4. Depletion-stack release: dynamic voltage sharing with the speed-up capacitors, the linear-region energy per device against the FBSOA, the active release timing (§B5.4).
5. Pump output and hold-up at the real load across −20 to +60 °C; detector rise/hold-up/collapse times (§B5.4, §B8.1).
6. LT3751 child-mode regulation at ≈ 1,300 V with a 1:50 transformer; the hand-over behaviour (§B4.4).
7. Junction temperature of the SiC switches per shock from the Zθ curve (rank 1).
8. Charger interference in the impedance channel (measurement only).

---

## §17. Questions for the owner

1. **Emergency switch-on:** sealed button (assumed), pad-cover pull, or holster switch?
2. **Q-47:** one data contact (assumed) or two?
3. **Second discharge path:** keep (recommended)?
4. **Voltage chains:** thin film on both sides? Accept recalibration at service?
5. **Display size:** 1.26 in (assumed; keeps READY visible while off) or larger (needs a 5 V always-on supply)?
6. **Internal resistance 5 Ω (D-23):** keep R_S = 3.9 Ω?
7. **Weight (new):** the review's fixes add ≈ 45–80 g; the lower-risk prototype is now ≈ 880–910 g, at or just over the 900 g accepted (D-43). Accept, or look for savings (powder-core coil, smaller-die stack devices, a lighter relay)?
8. **Bleeder:** ≈ 41 min to below 50 V acceptable as the last resort?
9. **Silent failure on a battery-bus short (new):** accept, or add a small independent cell for the supervisor's alarm?
10. **Q-50:** confirm D-59, D-60, D-61 as specified here (in particular: once current starts, cancel cannot stop the shock; worst case with a single fault ≈ 177 J monophasic).
11. **Charge time at tolerance corners (new):** 7.2–7.8 s at the worst corners against the ≤ 7 s target (D-29). Accept, or raise the current limit (larger cold battery current)?

---

## §18. Decisions in DECISIONS.md that are strained

| Decision | Strain |
|---|---|
| D-29 with D-36/D-37 | Charger interference with compression sensing — measurement only |
| D-44 / Q-39 | 9.2 A average / 21 A peak from a cold pack |
| **D-43** | Now ≈ 880–910 g lower-risk set: at or over the 900 g prototype limit |
| D-41 / Appendix A | Fault current ≈ 470 A, not 400 A |
| D-28 | Main chain life drift |
| D-29 "≤ 7 s" | Not met at tolerance corners (7.2–7.8 s) |
| D-13 vs ARCHITECTURE.md | ARCHITECTURE.md still says HV schematics wait for the patent review |
| D-23 | 5 Ω kept by design choice |
| **D-59** | **The firmware as written** (supervisor.rs re-checking pads and voltage during the permit; rescue.rs aborting `Delivering` on pad/chest readings; `interlock.rs`/`plant.rs` modelling `bridge_may_fire` level-by-level) **contradicts the decision**; the firmware change being made in parallel must follow §F |
| ARCHITECTURE.md "three independent checks" text | Describes the interlock as "bridge drivers powered only while both signals are present"; under D-59 that is true at the start only |

---

## §19. What I could not verify

- ADI documents (servers time out); LT3751 rev D read from a mirror (FB and RVOUT figures confirmed in that text).
- TI parts (UCC5350MC on input-side loss, TPS25982 limits and variants, TPS22918, TPS62130 dropout, INA240 swing, comparators, logic) — background knowledge; the review read some.
- SiC: the full SOA and Zθ numbers; short-circuit withstand; lifecycle.
- IXTT10N100D2 FBSOA; IXTY1R6N100D2 existence; BSS126 figures; optocoupler part and CTR at 1 mA.
- D1301 (3.3 kV SiC MPS) part code and surge rating; R1004 part.
- Gigavac G17 pulse rating, size, auxiliary contacts; D-HR 5 V-coil code.
- Ohmite EY; DPCR ratings and lifecycle; a 1 kΩ pulse-rated drive resistor.
- MAX30009 range/package/idle behaviour; ADS1292R drive level and frequencies.
- Microcontroller I/O counts per package.
- Every IEC clause and limit; prices and stock.
- **Anything about real behaviour**: no circuit simulation was available; see §16's [SIM] list.

---

## §20. Sources

As version 0.1 (LT3751 mirror; Infineon BSC070N10LS5 page; Sanken UX-F5B page; SSDI SDA5000UF page; IXTH/IXTT10N100D2 copies; Microchip MSC080SMA330B4N; G2R120MT33J; UCC21750; Murata MGJ; Cynergy3 D-HR; Gigavac G17 extract; TT DPCR; ADS1292R; MAX30009; STM32U585; MCX A153; MAX17320; BQ25887; HVCR; MEDTEQ via research notes; Sharp memory LCD; the firmware sources), plus, 27 Nov 2026:

| Part / topic | Source | Access |
|---|---|---|
| Independent review | docs/reviews/2026-11-27-hardware-design-review.md (which read the SN6505B, SN74LVC1G123, MSC080SMA330B4N DS00005914A, UCC5350, TPS22918 and G17 documents) | read |
| LT3751 FB pin thresholds, bias current, VOUT comparator band | LT3751 rev D, Electrical Characteristics (mirror https://static.chipdip.ru/lib/439/DOC011439824.pdf) | read |
| 3.3 kV SiC MPS diode (D1301 class) | https://www.mouser.fr/genesic-3300v-5a-sic-schottky-diode | page |
| TPS25982 eFuse | https://www.ti.com/product/TPS25982 | not opened [BK] |
| BSS126 | https://www.infineon.com/ (BSS126 product page) | not opened [BK] |

---

## §F. Firmware requirements from hardware

Everything the firmware must do, or stop doing, for this circuit to work. Written for the parallel firmware change (D-59 points 1–5). "M" = main, "S" = supervisor.

**F-1 Consent signals.**
1. `HOLD_M_CLK`, `HOLD_S_CLK`, `PERMIT_CLK`, `TEST_PERMIT_CLK`: square waves **1–5 kHz (nominal 2 kHz), duty 30–70 %**, produced by code that stops when the control loop stops (a GPIO toggled by the loop, or a timer in one-pulse mode re-armed by the loop); **no gap between edges longer than 1 ms**; **never** a free-running timer, PWM or DMA ring.
2. Watchdogs on both controllers, ≤ 50 ms.
3. A static level on any of these lines means "no consent".

**F-2 Power-up ordering.** `EN_5V0` at least 10 ms before any consent clock, `CHG_SESSION`, `RELAY_CMD`, `BTEST_CMD` or I²S activity. I²S lines low whenever `EN_5V0` is low.

**F-3 Charging.**
1. Raise `CHG_SESSION` ≥ 5 ms before `CHG_RUN` (eFuse soft start, VTRANS_GOOD delay).
2. Regulate by switching **`CHG_RUN` only**: off at the target reading, on again 3 V below. Do not toggle `CHG_SESSION` for regulation.
3. `CHG_FAULT_N` low → `CHG_RUN` low ≥ 1 ms, then high; count retries; give up and report after 3.
4. Lower `CHG_SESSION` when not charging.
5. **Firmware over-voltage limit 2,150 V** reading (R-30), both controllers.
6. Remember that a child charge cannot exceed ≈ 1,377 V in hardware and that the primary trip stops adult charges at 2,033–2,189 V.

**F-4 The shock (D-59).** The ordering of §B13.1:
1. Checks (pads in range, at plan, countdown, not cancelled, temperature) are made **before** consent: M before `SHOCK_REQ`; S before starting `PERMIT_CLK`.
2. M: `SHOCK_REQ` → wait `BRIDGE_ARMED_N` low (≤ 10 ms) → wait 20 ms → `RELAY_CMD` → wait `K_REST_N` high + 10 ms (≤ 40 ms total from `RELAY_CMD`, else abort) → `CMD_HA` → 100 µs → `CMD_LB` for phase 1 (from the plan) → `CMD_LB` low → 10 µs → `CMD_HA` low, `CMD_HB` high → `CMD_LA` at 0.5 ms after `CMD_LB` low, for phase 2 → `CMD_LA` low → 10 µs → `CMD_HB` low → `RELAY_CMD` low → withdraw `SHOCK_REQ`. All inside the 150 ms window.
3. **S, once the permit is running:** withdraw it only for cancel, heartbeat loss, a latched fault, over-temperature or over-voltage — **not** because its pad reading goes open or its voltage reading leaves the at-plan window. (Change to supervisor.rs.)
4. **M, in `Delivering`:** do not abort on pad or chest readings after `RELAY_CMD` (change to rescue.rs).
5. **Once `SEQ_ACT_*_N` is low**, both controllers treat consent checks as over: withdrawal has no effect, and the expected changes (pads open, voltage falling) are not faults. Until `SEQ_DONE_*_N`.
6. After `SEQ_DONE_*_N`: withdraw consent; consent must be absent ≥ 50 ms before the next attempt (one sequence per permit).
7. Status: **Delivered** = `SEQ_DONE` and `BR_FAULT_N` high and `I_SHOCK` shows both phases of the planned sign, size and length; **Refused** = `BRIDGE_ARMED_N` never low, or `SEQ_ACT` never low within the window; **Fault** = `BR_FAULT_N` low, or current wrong, or relay never transferred.
8. Model the same semantics in `interlock.rs`/`aed-sim` (`plant.rs` currently fires the whole shock instantly when `bridge_may_fire` is first true).

**F-5 Mode (D-60).** Compare `MODE_ADULT_N`/`MODE_CHILD_N` with `MODE_HW_RB_N` at switch-on and on every change; mismatch → record, "not ready"; in a rescue set the dose per D-60.

**F-6 Relay.** `K_REST_N` must read "at rest" at switch-on and before every analysis; otherwise fault (relay welded or coil driver shorted).

**F-7 Discharge paths and hold sensors.** In every self-test discharge release the holds one at a time, alternating which goes first: with only path A conducting τ ≈ 0.45 s and `HOLD_RB_B_N` high; with only path B the reverse; both together τ ≈ 0.225 s. Fit τ after every dump; τ ≥ 0.4 s on a both-paths dump = one path failed.

**F-8 Test permit (D-61)** — at switch-on before the first charge, and in the daily test:
1. S gives a test permit only if its own reading is < 50 V, M reports a test state, and for ≤ 150 ms.
2. **Blocking tests** (capacitor < 50 V, both holds on unless stated), each checking `BRIDGE_ARMED_N`: (a) test permit, no `SHOCK_REQ` → must stay high; (b) `SHOCK_REQ`, no permit → high; (c) `SHOCK_REQ` + test permit, M's hold released → high; (d) same, S's hold released → high; (e) `SHOCK_REQ` + test permit + both holds → low; (f) keep (e) for 200 ms → must go high (window expiry).
3. **Bridge test** (capacitor charged to 40–50 V, charger stopped): test permit + `SHOCK_REQ` → `BTEST_CMD` → 2 ms → the sequence with short phases (e.g. 1 ms each): `I_SHOCK` must be zero during the `CMD_HA` lead; present in phase 1 and 2 with opposite signs; zero after each phase. `K_REST_N` must stay "at rest" throughout. Then a deliberate overlong phase 1 command (> 30 ms) must end with `BR_FAULT_N`; a second `CMD_LB` must be ignored.
4. **Test-relay weld check:** with the test relay commanded open, `CMD_HA` alone → `OUT_B_TAP` must stay near 0.
5. Record every result; any failure → "not ready"; a failure at switch-on → no shocks in this rescue (CPR guidance), per D-50.

**F-9 Cross-loading.** Calibrate each impedance channel with the other chip running its normal carrier; widen S's permit range by its calibration uncertainty.

**F-10 Logging and memory.** F-RAM written once per standby wake; I²C idle low with the main off — configure the MAX17320 so that does not shut its interface down [UNVERIFIED need].

**F-11 Proposed HAL additions** (for the firmware team to decide): `MainHardware::sequence_status()` (from `BRIDGE_ARMED_N`, `SEQ_ACT_M_N`, `SEQ_DONE_M_N`, `BR_FAULT_N`); `MainHardware::relay_at_rest()`; `MainHardware::mode_hw_ceiling()`; `MainHardware::hold_readback()`; `MainHardware::run_test_sequence(...)` or equivalent primitives; the charger split (`set_charger` implemented with `CHG_SESSION`/`CHG_RUN`); `SupervisorHardware::set_test_permit(bool)`; `SupervisorHardware::sequence_status()` (`SEQ_ACT_S_N`, `SEQ_DONE_S_N`); `SupervisorHardware::overvoltage_tripped()` (`OVP_TRIP`).

**F-12 Simulator parameters.** Dump 2,250 Ω (4,500 Ω one path), total bleed ≈ 6.65 MΩ, relay transfer ≤ 40 ms, sequence cap 25 ms, window 150 ms.

---

## §R. Response to review

Every Blocker and Major is fixed; none is rebutted outright. Where I checked the reviewer's claim myself, it says so.

| Finding | Verdict | How (one or two lines) |
|---|---|---|
| **R-1** Consent is continuous power; firmware withdraws it mid-shock | **Fixed** | D-59 sequencer: consent starts, SW5 `SEQ_HOLD` completes; defined turn-off order, rail-UV detector and 2 ms `BRIDGE_3V3` hold-up (§B13); firmware rules F-4 |
| **R-2** No freewheel path for L_S | **Fixed** (checked: an N-MOSFET's body diode conducts source→drain, i.e. output→`HV_BUS`; the reviewer is right) | D_FW across R_S + L_S (3.3 kV SiC MPS); snubber; every turn-off is low-side-first and uses D_FW (§B6.5) |
| **R-3** SN6505B soft-start | **Fixed** (I could not open the SN6505B datasheet; accepted the reviewer's reading — the replacement removes the dependency either way) | Gated push-pull from a free-running 400 kHz clock, no soft start; timing re-derived: on ≈ 20 µs, off ≈ 10 µs, gap margin ≥ 300 µs (§B6.3) |
| **R-4** Retriggerable window | **Fixed** (agreed: the '123 is the retriggerable monostable) | Latched RC cap started from a latch, cleared only by power-on reset; one phase 1 and one phase 2 per rail cycle; 150 ms latched permit window on 09 with a 50 ms absent-consent re-arm (§B8.2, §B13.3) |
| **R-5** CHARGE/supply race; inrush | **Fixed** (checked in the LT3751 text: UVLO trips the FAULT latch, CHARGE toggle resets it) | `VTRANS_EN` (session) and `CHARGE_EN` split; CHARGE only after VTRANS_GOOD + delay; eFuse soft start; regulation by CHARGE only (§B4.4) |
| **R-6** `HOLD_OK` fails to "held" | **Fixed** | "Held" = opto LED lit by the negative bias; open/unpowered = "not held"; AND with pull-ups that mean "not held" (§B5.4) |
| **R-7** FB regulates; tolerances | **Fixed** (checked: 1.12–1.20 / 1.19–1.25 / 1.29–1.38 V, 300 nA, comparator 0.955–1.005 V) | Adult FB grounded; FB is a child-only regulator (1,283–1,377 V); primary trip lowered to 2,111 V nominal with 0.1 % parts (2,033–2,189 V); full stack-up table (§B4.4) |
| **R-8** Mode pole B blocks adult shocks | **Fixed** per D-60 | Default = adult ceiling; `MODE_HW_RB_N` readback; firmware compare (§B4.4, F-5) |
| **R-9** Pump load, release, reference, clamps | **Fixed** | Real load 1.1 mA; 3-stage pump −7.35 V; 3.3 µF reservoir, 0.17 V ripple, 3.5 ms hold-up; 1 MΩ gate resistor; active release ≤ 10 ms; AC/opto-coupled to the stack source; two net ties; TVS on all eight gates; contradiction removed (§B5.4) |
| **R-10** OVP force level shift | **Fixed** | Opto per stack driven by `OVP_TRIP`; transistor across each stack's gate reservoir (§B5.4) |
| **R-11** Detector band, hold-up, margin | **Fixed** | 1–5 kHz band, ≥ 100 kHz rejected, hold-up ≥ 3 ms vs 1 ms firmware gap, collapse ≤ 8 ms; 150 ms hardware permit maximum (§B8.1) |
| **R-12** Relay coil supply, on-time | **Fixed** | 12 V boost from `VBAT_SYS`, economiser, ≥ 30 ms hold-up, 90 ms latched cap, fast release, start window ≤ 60 ms (§B7.2) |
| **R-13** Self-tests cannot run | **Fixed** per D-61 | Test-permit mode with hardware `LOWV_OK`, patient relay blocked, test load only (§B14); firmware F-8 |
| **R-14** Shorted low side cascade | **Fixed** | Daily/switch-on bridge test finds it; hardware idle-current latch (5 A) ends a sequence that starts before its low-side command (§B13.3, fault 18) |
| **R-15** Shorted rectifier | **Fixed** | Two diodes in series, each blocking the full voltage; HV fusible resistor (§B4.3) |
| **R-16** Charger fault darkens everything | **Fixed for charger faults** | eFuse current limit and latch-off ahead of the pack protector; protector thresholds above every branch; residual `VBAT_SYS` short listed (fault 40, owner question 9) |
| R-17 Cross-loading | Fixed | Idle drive high-Z; calibrate with the other chip running (§B3.4, F-9) |
| R-18 Net classes | Fixed | `RTN_PULSE` class for `HV_RTN`, `LS_SRC` (§B12, hierarchy.toml) |
| R-19 Charge time corners | Fixed (stated) | §B4.2; owner question 11 |
| R-20 VCC UVLO | Fixed | 4.8 V; `5V0_SYS` ≥ 4.9 V at 5.2 V pack |
| R-21 Domain crossings | Fixed | `LCD_BLANK` via AON inverter; I²S series resistors + rule; tap clamps to GND; MAX17320 note; EN_5V0 ordering (F-2); push-pull/opto held signals |
| R-22 Upper device margin | Fixed | Four devices per stack |
| R-23 PNP open | Fixed | 10 kΩ bleed backup; consequence and test coverage stated (fault 26) |
| R-24 INA240 full scale | Fixed | INA240 on `BRIDGE_5V` + 2:3 divider: 120 A at 3.2 V |
| R-25 SiC datasheet corrections | Adopted | §B6.2, §B6.3 |
| R-26 Coil size | Fixed | Re-derived (§B6.5) |
| R-27 Table errors | Fixed | §13 re-derived; bottom-leg doubling (§B9.1) |
| R-28 Front-end clamps, 1 kΩ DPCR | Fixed | TVS to analog ground below the rail plus rail shunt clamps; drive resistor by requirement (§B3) |
| R-29 Logging cadence | Fixed | 42 µW in the budget |
| R-30 Firmware OV limit | Adopted | 2,150 V recommended (F-3) |
| R-31 Inputs with no HAL purpose | Fixed | `SHOCK_REQ` dropped from S; `OVP_TRIP` kept with a proposed HAL method (F-11) |
