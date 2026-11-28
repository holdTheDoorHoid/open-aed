# Hardware design specification

| Version | Date | What changed |
|---|---|---|
| 0.1 | 27 Nov 2026 | First design specification, written before schematic capture: topology and parts for every block, the arithmetic, the sheet plan (`hardware/aed-main/hierarchy.toml`) and the net contract (`hardware/aed-main/contract.md`). |
| **0.2** | **27 Nov 2026** | **Revised after the independent review** (`docs/reviews/2026-11-27-hardware-design-review.md`, findings R-1 to R-31) and decisions **D-59** (consent starts a shock; hardware finishes it), **D-60** (mode-switch disagreement → adult ceiling, read back) and **D-61** (low-voltage test permit). Main changes: a hardware **shock sequencer** with a non-retriggerable latched time cap, a sequence hold that keeps the bridge powered to the end of the planned pulse, a maximum permit window, defined turn-off order and supply-collapse behaviour (§B13); a **freewheel diode** for the series coil (§B6.5); the SN6505B replaced by a **gated push-pull high-side drive** with no soft-start (§B6.3); the charger's supply switch and CHARGE pin **split**, regulation by the CHARGE pin, FB used only as a child ceiling, a consistent **threshold stack-up** (§B4); "held" now **actively proven** by an optocoupler that needs the negative gate bias (§B5.4); four devices per discharge stack, an active release, gate clamps, hold referenced to the stack; **test-permit mode** (§B14); a patient relay coil supply with on-time cap, hold-up and a position contact (§B7); two rectifiers in series and an eFuse that isolates charger faults (§B4); the bridge logic moved to its own sheet; **16 sheets, 8 capture work packages**; a "Firmware requirements from hardware" section (§F); a "Response to review" section (§R). |
| **0.3** | **27 Nov 2026** | **Revised after the first schematic capture (25 integration issues I-1…I-25, `hardware/aed-main/capture-notes/00-integration.md`) and the first circuit simulations** (`hardware/aed-main/sim/bridge`, `charger-storage`, `logic-relay`, plus the check runs in `sim/rev3`), with provisional decisions **D-62 to D-67**. Main changes: discharge stacks rebuilt on **1,700 V devices with gate steering and a resistor and capacitor across each device** (the v0.2 "727 V with one device shorted" was wrong: 2,141 V simulated); **two independent "held" sensors per path**; hold pump, release and sensor re-valued for −20 °C; **over-voltage limits re-placed** (firmware 2,070 V reading, comparator 2,107 V and **latching**, charger-chip trip moved above the normal range as a last resort); **high sides turned off at once on a current fault**, a **fast 3 A idle-current path**, a **carrier-loss detector** on each high-side island, over-current trip **80 A**, a **local film capacitor**, a **low-side supply monitor**; **per-phase hardware time caps** (D-63); **relay-coil reservoir filled before the shock** (`K_SUPPLY_EN`); **real-permit self-test steps** (D-62); compression sensing given its own resistors and blanked while charging (D-67); patient relay and capacitor requirement sheets in `docs/requirements/` (D-64, D-65); contract issue 3 (10 new nets). §V3 maps every issue and simulation finding to its resolution. |

> **NOT FOR CONSTRUCTION — unreviewed design.** This document was written with AI assistance. **It has not been reviewed by a qualified high-voltage or medical-device engineer, and nothing in it has been built or measured.** It has had one independent paper review, itself AI-assisted, and parts of it have been simulated (ngspice, with models built by hand from datasheet figures because the makers' models could not be obtained or used); simulated figures are tagged [SIM-done] with their source and are no better than those models. It may contain errors that would make a built circuit dangerous or useless. The storage capacitor in this design holds about 190 joules at about 2,000 volts, far above the level that stops a heart, and keeps it after the power is off. This is a design on paper for review and criticism (SAFETY.md §2a, D-13). It contains no assembly steps and no bring-up procedures, and it does not tell anyone to build or power anything (CLAUDE.md rule 2, D-10, D-18). Nothing here is safe or ready for use on a person or animal.

How to read the tags:
- **[CALC]** arithmetic done for this document; the working is shown.
- **[EST]** an engineering estimate; its basis is stated.
- **[BK]** background knowledge not re-checked against a source this time.
- **[UNVERIFIED]** looked for and not confirmed, or depends on a datasheet that could not be opened.
- **[SIM]** can only be settled by circuit simulation or, later, measurement (§16 lists them all).
- **[SIM-done]** simulated (ngspice 42, hand-built models) in the study named beside it: BR = `sim/bridge/notes.md`, CS = `sim/charger-storage/notes.md`, LR = `sim/logic-relay/notes.md`, R3 = `sim/rev3/notes.md` (all under `hardware/aed-main/`).
- **"read 27 Nov 2026"** = datasheet text read that day; **"page only"** = product page, listing or search extract.

Decision numbers (D-xx) and questions (Q-xx) refer to [DECISIONS.md](../DECISIONS.md). Review findings are R-xx.

---

## Plain-language summary

**What this is.** The parts study chose *kinds* of parts. This document turns them into a circuit design that several engineers can draw at the same time. Version 0.2 fixed the problems an independent review found (§R). **Version 0.3 fixes what the first drawing of the circuit and the first computer simulations of it found** (§V3): 25 issues from the drawing and about forty findings from the simulations. Most were fixable with part changes. A few were real design mistakes: the discharge paths could put twice their rating on one transistor after a single failure; a failed bottom output switch could destroy the top one before protection acted; a frozen controller could let up to about 180 J through instead of 150 J. All three are fixed on paper and checked again in simulation. What is still open is listed in §V3.3 and §16.

**How a shock now works (D-59).** Before a shock, three things must all be true: the main controller asks for it, the supervisor permits it, and both discharge paths are held off. Each of the controllers' signals is a constantly changing signal that only running software can make. When all three are true a supply rail for the output stage comes up, a relay moves the pads from the heart-signal electronics to the output stage, and the main controller's timer runs the two-phase shock. **From the first moment current flows, plain hardware takes over:** it keeps the output stage powered until the planned pulse ends, whatever the two controllers do next, but never for longer than a fixed hardware limit of about 25 thousandths of a second, which nothing can extend. Then it turns the switches off in a fixed order, returns the pads, and refuses another shock until both controllers have withdrawn their signals. This matters because during a shock the pad reading must go "open" and the capacitor voltage must fall; if those changes could withdraw permission, the pulse would be cut off at a random moment. Cancel, a stopped controller, overheating and over-voltage can still stop a shock **before** it starts; once it has started they cannot, by design, and the worst extra energy that can then reach the patient is worked out in §B13.6.

**The big choices.**

1. **Charging the capacitor.** A transformer-based "flyback" charger (Analog Devices LT3751) lifts the 6–8 V battery to 1,950 V in 3.3–5.3 s in simulation, about 5.9 s at the worst corner once the made-to-order transformer's own losses are included (it must be at least 90 % efficient). The software turns it on and off at the target; behind that, **a firmware limit and an independent hardware comparator** keep the capacitor below its 2,200 V rating, the comparator now **latches** so it empties the capacitor completely, a **last-resort stop inside the charger chip** sits above the normal range, and a fourth layer (D-25) **cuts the charger's power** whenever either discharge path is not proven held off. In child mode a further hardware limit keeps the capacitor below about 1,380 V even if all software fails.
2. **Emptying the capacitor by default.** Two parallel paths, each made of four 1,700-volt transistors that **conduct unless actively held off**. The main controller alone holds path A off; the supervisor alone holds path B. If either stops, its path empties the capacitor within about 2 seconds. "Held off" is proven by **two** independent light-coupled sensors per path, each of which can only light up if the holding voltage is really there, so a broken sensor reads "not held" and one failed sensor cannot fake "held".
3. **Delivering the shock.** Four 3,300-volt silicon-carbide transistors. The two "bottom" ones do the precise timing with catalogue drivers. The two "top" ones float up to 2,000 V and are switched slowly through small made-to-order transformers; a buzzing signal on the transformer keeps them on, silence turns them off. If a bottom switch has failed short, a fast detector now notices current within about a tenth of a microsecond and turns the top switches off at once, and a small detector on each top switch's floating supply empties it as soon as the buzzing stops.
4. **A self-test that reaches the shock path (D-61, D-62).** A "test permit" lets the device exercise the interlock and all four output switches **only while the capacitor is below about 60 V, enforced by a hardware comparator**, and **only into an internal test resistor: the hardware makes it impossible for the patient relay to move in test mode.** Version 0.3 adds (provisionally, D-62) steps that use the **real** permit and move the patient relay while both controllers have measured the capacitor below 60 V, because some parts used only by a real shock could otherwise fail unseen and then block every shock (§B14.5).
5. **Protecting the heart-signal electronics.** At rest a relay connects the pads to the heart-signal electronics; only an energised relay connects them to the output stage. The relay now has its own coil supply, a hardware limit on how long its coil can be on, enough stored energy that it cannot drop out in the middle of a shock, and a contact that tells the main controller where it is.
6. **Two controllers that cannot fail together.** ST STM32U585 (main) and NXP MCX A153 (supervisor), different makers (D-39), each with its own regulator, crystal, capacitor-voltage measurement, pad-resistance measurement, capacitor temperature sensor, cancel contact and alarm (supervisor).

**What it costs.** The v0.2 fixes added parts: a 22-chip discrete logic sheet for the shock sequence, an eighth discharge transistor pair, optocouplers, a second rectifier, a freewheel diode, a relay coil supply: **+45–80 g**. Version 0.3 adds about **+30–60 g** more [EST], almost all of it the local film capacitor of the output stage whose size is not yet known (§14). The lower-risk set is now about **910–970 g** [EST], over the 900 g accepted for a first prototype (D-43, Q-56). That is listed as a strained decision (§18).

**What is still unknown.** The same top risks as before (switch pulse ratings — Microchip publishes no turn-off or short-circuit rating —, the made-to-order transformers, the patient relay, spacing at 2 kV); charger interference with compression sensing is now known to be too large to design out, so readings are discarded while charging (D-67, Q-58). Simulation settled most timing questions; §16 lists what it could not settle and the second round of simulations.

**One board or two.** Still expected to be two: a low-voltage board (sheets 01–09) and a high-voltage/patient board (sheets 10–16).

---

## Contents

- §0 Requirements baseline; departures from the parts study
- §B1 Power · §B2 Controllers · §B3 Front ends · §B4 Charger · §B5 Storage and discharge · §B6 Output bridge · §B7 Patient relay and pad interface · §B8 Interlock · §B9 Sensing · §B10 User interface · §B11 Durable memory and timekeeping · §B12 Creepage and partitioning · **§B13 The shock sequence (D-59, D-63)** · **§B14 Test-permit mode and self-test (D-61, D-62)**
- §11 Rail tree · §12 Power budget · §13 Single-fault table · §14 Made-to-order parts and weight · §15 Test points · §16 Open technical questions (including everything that needs simulation) · §17 Questions for the owner · §18 Strained decisions · §19 Not verified · §20 Sources
- **§F Firmware requirements from hardware** · **§R Response to review** · **§V3 Revision 0.3: every integration issue and simulation finding, and its resolution**

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
| D-62 (prov.) | Self-test may use the real permit and move the patient relay below 60 V | §B14.5 |
| D-63 (prov.) | Per-phase hardware time caps; firmware phase maxima 10 / 6.7 ms | §B13.3, §B13.6 |
| D-64, D-65 (prov.) | Relay and capacitor made to order with published requirement sheets (internal bleed, surge rating) | §B7.2, §B5.1, §14 |
| D-66 (prov.) | No steady load below 1 MΩ on the battery rail | §B1.1 |
| D-67 (prov.) | Compression sensing: own sense resistors; readings discarded while charging | §B3.4 |

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
- The MAX17320's discharge over-current and short-circuit thresholds are set **above** what any limited branch can draw: over-current ≥ 35 A **or** a delay ≥ 1 ms (v0.3: the charger eFuse can pass up to ≈ 30 A for 0.48 ms before it trips, WP-E C6), short-circuit ≥ 60 A with ≥ 100 µs delay (WP-A set 65 A / 192 µs).
- **v0.3 correction (WP-A C-3, LR §4):** the MAX17320 has no timed auto-retry. It releases a discharge fault when the load is removed: it sources a test current into PCKP (30 µA typical; **no minimum published**, MAX17320 datasheet Rev 5) and releases when PCKP rises above 1 V (1.5 V in another place of the same datasheet). That works only if the battery rail is nearly unloaded. **Rule (D-66): no steady resistive load below 1 MΩ on `VBAT_SYS` from any sheet.** The lowest drawn load is 8.0 MΩ (U304 enable divider). Simulated (LR §4.2): with 30 µA, loads below ≈ 45 kΩ block release; with 15 µA even 100 kΩ blocks it. With the 1 MΩ rule release works down to ≈ 10 µA [CALC: 8 µA modelled leakage + 1.5 V / (1 MΩ ∥ 8 MΩ) = 9.7 µA]. Settling the minimum needs ADI's figure.
- **Residual risk (Q-52, open):** a hard short on `VBAT_SYS` itself (upstream of every branch fuse) trips the pack protector and everything goes dark, including the supervisor's alarm, until the short is removed or the device is docked. Listed in §13 (fault 40). No independent alarm cell is added (D-66).
- **Mid-point (v0.3, I-1):** the pack mid-point is a contract net `VBAT_MID` from 01 to the dock charger on 02 (MID and CBSET), so the charger regulates each cell. Only one of the two balancers (MAX17320, BQ25887) is enabled.

**Reverse and short protection.** Keyed internal connector (no series reverse MOSFET: it would cost 0.2–0.4 W at 21 A peaks for a fault the connector prevents [EST]); short circuits per the coordination above.

### B1.2 Cell temperatures

Cell NTCs → MAX17320 thermistor inputs; the main reads them over I²C (`TempPoint::Cell1/Cell2`).

### B1.3 Battery-under-load test (`MainHardware::battery_under_load`)

A pulse-rated **0.80 Ω** load (two 0.40 Ω resistors, WP-A) switched by an N-MOSFET (`BAT_LOAD_EN`). [CALC] 10.4 A (86 W) at 8.4 V. A **hardware RC time limit** turns the load off after 124–220 ms [CALC, WP-A] even if `BAT_LOAD_EN` sticks. **v0.3 (I-25):** as drawn, a line toggled at 10–100 Hz restarted the limit every cycle and kept 14.5–22.7 W in the resistors [SIM-done LR §4.3]; a BAS116 in series with the timing resistor (now 78.7 kΩ), a 4.7 MΩ recharge resistor and a 100 kΩ input resistor cut that to 1.8–2.2 W [SIM-done for the diode and resistors; the 78.7 kΩ value is CALC, second round]. (Using the charger as the load was rejected: boundary-mode power is low at low capacitor voltage.)

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
| v0.3 additions | `K_SUPPLY_EN`, `HOLD_RB_A2_N`, `HOLD_RB_B2_N` | 3 |
| **Total** | | **73 + 6 = 79** of 114 GPIO on LQFP144 (v0.2 said "76 + 6 = 82 of about 110": the rows summed to 70; 114 verified by WP-B from DS13086) |

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
| v0.3 additions | `OVP_RST`, `OVP_TEST` | 2 |
| **Total** | | **28** of 52 GPIO-capable pins on LQFP64 (WP-B, NXP Table 41) |

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

**v0.3 correction (I-14).** The v0.2 figures (0.34 mA excitation, ≈ 1 mΩ noise) were wrong: the ADS1292R limits its modulation current to 100 µA (WP-H), and the ECG filter capacitor C1505 shared the impedance input nodes and swallowed ≈ 85 % of the 64 kHz signal. Simulated noise as drawn: **25–34 mΩ RMS** against < 10 mΩ [SIM-done LR §6]. **Fix (D-67):** the ECG branch gets its own 51 kΩ DPCR sense resistors (R1528, R1529), leaving R1507/R1508 for the impedance branch: **4.6–6.2 mΩ RMS** [SIM-done LR §6]. **Charger interference** is a different matter: a charger harmonic within 10 Hz of the carrier gives 0.46 Ω per V·pF at 175 Ω; holding 10 mΩ needs V × C ≤ 0.022 V·pF, i.e. ≤ 0.2 fF for a 100 V harmonic, which no layout can achieve [SIM-done LR §6]. So **impedance readings are discarded while `CHG_RUN` is high and for 100 ms after** (D-67, F-13); the consequence for CPR feedback during charge-during-analysis is owner question Q-58.

**Cross-loading (R-17).** The supervisor's drive taps (≈ 4.2 kΩ across the pads at 64 kHz) make the main read ≈ 6 % low at 250 Ω, and vice versa (≈ 3 kΩ at 128 kHz). Requirement: both chips' drive outputs are **high-impedance when idle**, and **each channel is calibrated with the other chip in a fixed, specified state** (firmware: the other chip running its normal carrier). The supervisor's permit range is widened by this calibration uncertainty in firmware (§F).

### B3.5 Pad presence

Plugged in: the connector's presence loop. On a patient: impedance in range.

### B3.6 Supervisor's own measurement

**ADI MAX30009** (16 Hz–500 kHz drive, 20-bit, SPI, 25-ball WLP), carrier **96 kHz** (v0.3, I-18: at 128 kHz it aliased into the ADS1292R's ECG channel, whose modulator samples at 128 kHz, WP-H §3.3; 96 kHz's odd harmonics stay ≥ 32 kHz from every odd harmonic of 64 kHz, so the two demodulators still reject each other). 1.8 V logic through a level shifter on sheet 16 (I-20). Temperature range [UNVERIFIED]; fallback ADI AD5940 (LFCSP).

### B3.7 Charger noise (a known risk)

The boundary-mode charger's frequency sweeps from a few kHz to ≈ 43 kHz and its harmonics cross both carriers during every charge. v0.3: simulation shows the resulting error is orders of magnitude above the compression signal for any plausible stray coupling (§B3.4), so the impedance channels are **blanked while charging** (D-67). Pausing the CHARGE pin for each reading (≈ 20 % slower charging) or a synchronised fixed-frequency charger remain options for the owner (Q-58). The coupling itself is still **measurement only** (§16).

---

## §B4. High-voltage charger (sheet `10_hv_charger`)

### B4.1 Topology

**ADI LT3751** flyback capacitor charger, **charge mode** (FB grounded) for adult, **FB regulation** used only as the **child ceiling**, primary-side output trip, made-to-order 1:50 transformer. (LT3751 datasheet rev D read 27 Nov 2026 from a distributor mirror.)

### B4.2 Power stage sizing [CALC, confirmed by the review; v0.3: charge time simulated]

| Quantity | Value |
|---|---|
| Turns ratio | 1:50 |
| Peak primary current | **21.2 A** (5.0 mΩ sense, 106 mV typical / 100–112 mV) |
| Primary inductance | **7.5 µH ± 10 %** (window 6.1–8.6 µH) |
| Charge time to 1,950 V (v0.3, simulated, **transformer core and AC winding losses not modelled**) | 7.2 V, 25 °C: **3.74 s**; 8.4 V: 3.32 s; 5.8 V cold: 4.46 s; child 1,150 V: 1.39–2.11 s [SIM-done CS Q3] |
| **Worst corner (v0.3)** | 5.5 V, 100 mV sense, R_sense +1 %, L_pri −10 %, hot switch, slow diodes, C +5 %: **5.29 s** [SIM-done CS Q3]; circuit efficiency 0.87. **With a transformer ≥ 90 % efficient ≈ 5.9 s**; 7 s (D-29) is missed only if the transformer loses more than ≈ 24 % (overall efficiency < 0.66). v0.2's 7.2–7.8 s assumed a lumped efficiency of 0.60–0.65 for everything; the simulation shows the circuit itself loses ≈ 13 %, so the answer now rests on the transformer, whose requirement gets an efficiency figure (§B4.5). Board standing load 2.80 MΩ (§B5.6) adds ≈ 0.1 s [EST, scaled from CS Q3's +0.03 s for 0.4 W] |
| Battery current at full charge | 9.2 A average, 53 W in (5.8 V) |
| Primary RMS / switch peak | 11.4 A (12.4 A at 5.5 V and the maximum current limit, WP-E C8) / drain peak **72–79 V** for 0.5–3 % leakage, with the SMCJ45A modelled at its quoted 73–75 V at 22 A [SIM-done CS Q3; clamp figure UNVERIFIED]; clamp 0.31–2.27 W |
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
| RVOUT / RBG / RDCM / RVTRANS | 40.2 kΩ / **876 Ω, 0.1 % (v0.3; was 931 Ω)** / 18.2 kΩ / 40.2 kΩ, all 0.1 % | primary-side trip **2,116–2,316 V** true, 2,409 V with the comparator's maximum overdrive [SIM-done CS Q4]: a last-resort layer above the normal range (§B4.4) |
| FB network | top 10 × 4.02 MΩ **0.1 % HV thin film**, bottom **36.9 kΩ 0.1 %**; an N-MOSFET from FB to ground, **on by default** (adult = FB grounded = charge mode); `MODE_CHILD_HW` turns it off → child regulation | §B4.4 |
| Primary clamp | RCD/TVS at ≈ 75 V | |

### B4.4 How charging is controlled, and the threshold stack-up

**Control split (R-5).**
- `VTRANS_EN` (from the interlock) = `CHG_SESSION` AND `HOLD_OK` AND NOT `TEST_MODE`. It enables the eFuse, whose dV/dt soft start limits inrush into the bulk capacitors. **The firmware does not toggle the session for regulation.**
- `CHARGE_EN` (from the interlock) = `CHG_RUN` AND `HOLD_OK` AND NOT `TEST_MODE`; on sheet 10 it is ANDed with `VTRANS_GOOD` (after its delay) to drive the LT3751 **CHARGE** pin. CHARGE therefore never rises before VTRANS is above UVLO1, so the LT3751 cannot latch its under-voltage fault at start.
- If the LT3751 does latch FAULT (supply out of range), firmware lowers `CHG_RUN` for ≥ 1 ms and raises it again (the datasheet's "toggling the CHARGE pin reactivates switching").
- Both `VTRANS_EN` and `CHARGE_EN` fall immediately when `HOLD_OK` falls (D-25 unchanged).

**Regulation at the target.** The LT3751 in charge mode runs until CHARGE falls or the primary-side trip fires. The main controller regulates by switching **`CHG_RUN`** (CHARGE pin) on its own voltage reading: off at the target, on again 3 V below it. Overshoot per step is ≈ 0.2 V (205 V/s at full charge × 1 ms loop) and per switching cycle ≈ 0.009 V; droop through the v0.3 standing load of 2.80 MΩ (§B5.6) is ≈ 7.1 V/s at 2,000 V [CALC: 2,000 V / 280 s], so the charger restarts about every 0.4 s while holding. **The resulting tolerance of the true voltage is the sensor's own (1 % + 7 V) plus 3 V hysteresis**, which is what config.rs's at-plan window assumes.

**The FB pin (R-7).** The LT3751's FB pin does not "stop" at a threshold: below 1.16 V (1.12–1.20 V) it is in charge mode; between that and 1.34 V (1.29–1.38 V) it **regulates FB to 1.22 V (1.19–1.25 V)** with reduced current; above 1.34 V it stops switching. FB bias current is up to 300 nA *out of* the pin (datasheet, read). No adult FB setting can keep both its charge-to-regulation hand-over above the operating window (≈ 2,008 V true) and its regulated maximum below 2,200 V (ratio needed 1.16, available 1.096 — the review's arithmetic, confirmed). So:
- **Adult (and default):** FB grounded by the MOSFET → pure charge mode. The ceilings are the primary-side trip and the over-voltage comparator.
- **Child:** FB divider active, ratio k = 1 + 40.2 MΩ / 36.9 kΩ = **1,090** → hand-over ≥ 1,207 V, regulated 1,283–1,377 V, stop ≤ 1,521 V (table below).
- **D-60:** the MOSFET is **on (adult) by default**; only the mode switch's second pole, closed in CHILD, turns it off. Contact open, wire broken or switch between positions → **adult ceiling**. The actual state is read back on `MODE_HW_RB_N` (low = child ceiling active). The main compares it with `MODE_ADULT_N`/`MODE_CHILD_N`; disagreement → recorded, "not ready", and in a rescue the firmware still sets the dose by the position it trusts per D-60.

**Threshold stack-up, v0.3.** v0.2's table was wrong in two places that the capture (WP-E C1, WP-C §3.4) and the simulation (CS Q4, 200,000-sample Monte Carlo and worst-case sums, which agree within a few volts) both found: the primary-side trip's band is ≈ 200 V wide before overdrive (comparator ± 2.55 %, turns ratio ± 1 %, **secondary drops 19–48 V**, which v0.2 took as ≈ 5 V), so at 931 Ω its low edge (1,988 V) fell *inside* the normal window; and the over-voltage comparator's band, with the supervisor string's temperature and life terms included, is 2,127–2,193 V, only 7 V under the rating. **No setting of the primary-side trip can fit between 2,008.6 V and 2,200 V** (its band is wider than the 191 V gap). The layers are therefore re-placed as the simulation recommends; I checked its arithmetic for the comparator (below) and adopt its bands.

| Threshold | Adult (true volts) | Child (true volts) | Source |
|---|---|---|---|
| Firmware target (reading) | 1,950 | 1,150 | D-23 |
| **Top of firmware at-plan window, true** = (target + 12 + 7)/0.99 | **2,008.6** | **1,192.4** | config.rs, D-28 |
| Child FB: charge→regulation hand-over (lowest) | — | **1,206** | v0.2 [CALC], unchanged |
| Child FB: regulated point / no-load stop | — | 1,283 – 1,377 / ≤ 1,521 | v0.2 [CALC], unchanged |
| **L1 firmware over-voltage limit, both controllers** (reading **2,070 V**, was 2,200 in config.rs and 2,150 recommended) | **2,043 – 2,098** | same | [SIM-done CS Q4] |
| **L2 over-voltage comparator** (sheet 08), **2,107 V nominal, latching** (Rin = Rf = 100 kΩ 0.1 %, reference 2.5 V × 15.6 k / 37.1 k = 1.0512 V, trip at tap = 2 × 1.0512 V, ratio 1:1,002.0) | **2,073 – 2,142** | same | [CALC here: CS Q4's band 2,076–2,145 V for 2,110 V nominal, shifted by the E192 divider's −3 V] |
| **L3 primary-side trip** (RBG 876 Ω): last resort | **2,116 – 2,316**; 2,409 with maximum overdrive | same (above the child FB) | [SIM-done CS Q4] |
| **Capacitor rating** (D-41, D-65) | **2,200 V continuous; 2,420 V for 10 s on every part; surge ≥ 2,420 V for ≥ 1 s, ≥ 100 events (new requirement, D-65)** | same | docs/requirements/storage-capacitor.md |

Checks: (1) **no hardware layer can stop a normal charge**: L2's lowest value is 64 V above the window, L3's 107 V; the firmware limit L1 is 34 V above. (2) **With any single control layer failed, the capacitor stays below 2,200 V**: L2's top is 2,142 V; L1's top is 2,098 V. (3) Only with **both** L1 and L2 failed does the capacitor rely on L3 and the new surge rating (≤ 2,316 V, 2,409 V with overdrive). **L2 now latches** (I-22): once tripped it holds `OVP_TRIP` high for any capacitor voltage, so both discharge paths are forced on until the supervisor resets it with an `OVP_RST` edge (or `3V3_SUP` is lost); v0.2's comparator released near 1,000 V, so it never emptied the capacitor as §B4.6 claimed. A shorted reset switch (OVP can never trip) is now found by the self-test step T-OVP (§B14.5).

Child-mode energy at the FB regulator's top (1,377 V): ½ × 100 µF × 1,377² = 94.8 J stored, against 66.1 J planned [CALC]; the supervisor refuses a permit outside the at-plan window, so this is a two-fault case.

### B4.5 Transformer T1001 (made to order; refines Appendix B1)

v0.3 additions (I-11, WP-E C2, C8): **efficiency ≥ 90 % (core + AC winding loss ≤ 10 % of throughput) at 20 A peak, 30–45 kHz**; inductance **7.5 µH +10 %/−5 %** (the LT3751's minimum on-time sensing needs ≥ 6.92 µH at 2,284 V); RMS **12.4 A**; secondary winding resistance stated. Otherwise unchanged from v0.1: 1:50 ± 1 %; ≤ 10 % inductance loss at 25 A, 70 °C; 11.4 A RMS for 7 s, 14 charges at 132 s, 20 in 1,300 s; leakage ≤ 1 % of LPRI; secondary self-capacitance ≤ 15 pF; working insulation ≥ 2,200 V DC for 10 years; test ≥ 4.4 kV DC 60 s; partial-discharge inception ≥ 3.3 kV after humidity soak; −20 to +70 °C; class B; ≤ 15 cm³, ≤ 30 g. **Turns-ratio tolerance ±1 % is now load-bearing** (it sets the primary-side trip band).

### B4.6 Summary of charger protections

| Layer | Acts on | Software needed? |
|---|---|---|
| Firmware regulation (main) | CHARGE pin at the target | yes |
| Firmware over-voltage (both controllers) | release hold → dump | yes |
| Child FB regulator | regulates ≤ 1,377 V in child mode | no |
| Over-voltage comparator (latching) → both stacks forced on → `HOLD_OK` false → `VTRANS_EN`, `CHARGE_EN` low | trips at 2,073–2,142 V and empties the capacitor completely | no |
| Primary-side trip (last resort) | stops a charge at 2,116–2,316 V (2,409 V with overdrive) | no |
| D-25 gate (`HOLD_OK`) | removes CHARGE and VTRANS whenever either path is not proven held | no |
| eFuse U1002 | limits and latches off a charger short or inrush | no |

---

## §B5. Energy storage, discharge by default (sheet `11_hv_storage`)

### B5.1 Capacitor connection

C1101 (Appendix A) off-board on HV terminals J1101; two case NTCs on a separate LV connector J1102 (one per controller, D-56). **The capacitor negative is `HV_RTN`, joined to `GND` at one star point by NT1101 and NT1102 — two parallel net ties built as separate wide bonds** so one open bond does not separate them (R-9). Everything that must keep working if they did separate (the hold pumps, the hold sensing, the over-voltage force) is coupled by capacitors or light, not by a DC connection to `GND` (§B5.4). **v0.3 (I-5, D-65):** the capacitor carries its own **10 MΩ bleed inside its case**, so an open J1101 no longer leaves it charged indefinitely (2,200 → 60 V in 63 min with the capacitor alone [SIM-done CS Q5]; a "no stored charge forever" measure, not "safe to touch").

### B5.2 Why depletion-mode stacks

Unchanged reasoning (v0.1): a normally-closed relay would make onto 2 kV at ≈ 18× its switching rating, needs coil power to hold open, and has a spring; enhancement MOSFETs need power to conduct; a bridge-based dump shares the shock path's failure modes. **Depletion-mode MOSFETs conduct with no power at all.**

### B5.3 Two paths

```
HV_CAP_P ─┬─ R1101–R1103 (3 × 1.5 kΩ) ─ stack A: Q1101–Q1104 (4 × 1,700 V depletion) ─ HV_RTN   held off only by the MAIN
          ├─ R1104–R1106 (3 × 1.5 kΩ) ─ stack B: Q1105–Q1108                          ─ HV_RTN   held off only by the SUPERVISOR
          ├─ bleeder 8 × 2.49 MΩ ─────────────────────────────────────────────────── HV_RTN   always
          ├─ main sense string (40.2 MΩ) ─ VCAP_M_TAP ─ bottom leg ───────────────── HV_RTN
          └─ supervisor sense string (39.8 MΩ) ─ VCAP_S_TAP ─ bottom leg ────────── HV_RTN
   (inside the capacitor: 10 MΩ bleed, D-65)
```

Dump timing [SIM-done CS Q2], from 2,200 V to below 60 V: one path **1.62 s** nominal (1.31–1.97 s at the resistor, temperature and capacitance corners), both paths **0.81–0.99 s** (limit 3 s); peak 0.42–0.57 A per path; each dump resistor takes 77–85 J and up to 419 W peak (Ohmite EY 400 J single impulse). Each resistor starts at 733 V, above its 550 V working rating for ≈ 0.13 s per dump — Ohmite must confirm [UNVERIFIED]. The IXTT2N170D2's 6.5 Ω on-resistance adds 26 Ω to a 4.5 kΩ path (+0.4 %). A failed path is found by the τ fit on every dump, and each path is exercised **alone** in every self-test discharge by releasing the holds one at a time (§F).

### B5.4 Holding a stack off, and proving it (R-6, R-9, R-10, R-22; v0.3: I-3, I-4, I-17, I-24)

**What v0.2 got wrong (I-3).** v0.2 claimed "with one device shorted: 727 V". The simulation (CS Q1) shows that with the drawn gate network a short of the *second* device puts **2,141 V** (cold, at 2,200 V) on the bottom device: the transistor above holds the shorted node near its own tap, the shorted device's 15 V gate clamp then conducts into the tap below, and that current (milliamps, against a 55 µA string) drags the string up. I checked the mechanism against the drawn topology and agree: v0.2's analysis assumed the gate network draws no current. **A fifth device does not help (2,116 V), nor do 1,700 V devices alone (2,138 V)**, so WP-F's and I-3's proposed fix is overturned.

**Devices.** Four **Littelfuse/IXYS IXTT2N170D2** per stack (1,700 V; V_GS(off) −2.0 to −4.0 V; I_DSX(off) ≤ 25 µA at 25 °C, ≤ 500 µA at 125 °C; R_DS(on) ≤ 6.5 Ω; TO-268 like the 10N100D2; DS100418B as read in CS). Cold derating **assumed** 0.94 (1,598 V at −20 °C): Fig 12 not read [UNVERIFIED]. No catalogue depletion MOSFET above 1,700 V was found (CS).

**Static and dynamic sharing (configuration "cfg 5" of CS, adopted).**
- Sharing string **4 × 4.7 MΩ** (was 4 × 10 MΩ); the 3.3 nF C0G capacitors across each string resistor stay.
- Each upper gate is fed from its tap through 100 kΩ and a **steering diode** (Vishay GP02-40, 4 kV, anode at the tap side) **bypassed by 22 MΩ**; the gate–source resistor is **100 MΩ** (was 10 MΩ). The diode stops a shorted device's clamp current from dragging its tap.
- **Across each device, drain to source: 10 MΩ and 3.3 nF / 2 kV C0G.** Essential, not optional: with the gate fed through a diode a leaky device's source can float, and without these one Monte Carlo run reached 1,759 V (CS).
- Simulated [SIM-done CS Q1]: nominal 546–554 V per device at 2,200 V; **any single shorted device ≤ 1,174 V across 100 Monte Carlo runs (0.73 of the assumed cold rating)**; deterministic device-2 short 1,158 V; leakage mismatch hot 704 V; fast 1,000 → 2,200 V step 626 V; release edge 643 V.
- **Not settled:** the GP02-40's reverse leakage (≤ 5 µA at 25 °C, ≤ 50 µA at 100 °C at its 4 kV rating, Vishay doc 88635 Rev 09-Aug-2022) was not in the model; the five-device variant's unexplained 1,464 V Monte Carlo outlier is not investigated because five devices are not adopted (second round S-8, S-9).

**Gate clamps.** A bidirectional 15 V TVS across gate and source of every device, as v0.2.

**Hold bias (bottom device).** Clock `HOLD_A_DRV` / `HOLD_B_DRV` from each controller's own rail, **1.5 kHz ± 10 % (v0.3; was 1–5 kHz), always stopped low**, through the 8 kHz two-pole low-pass at the buffer *input* (WP-D DEP-1). **Four-stage** single-clock pump (WP-F), 2.2 µF stages, 3.3 µF reservoir, 1.01 MΩ gate–source, and **new: 470 Ω between the last pump stage and the gate** (R1194/R1195). Hold level at the cold corner (−20 °C, 3.1 V clock, capacitors at 70 %): **−5.9 to −6.1 V** worst point of the ripple [SIM-done R3 §2]. v0.2's −7.35 V (and WP-F's −7.4 V from a behavioural model) were optimistic; v0.3 is designed to the lower figure. I-24: the simulation found 3, 4 and 5 stages give about the same loaded hold (CS Q2); the fourth stage is kept because it is drawn and costs nothing to keep.

**Active release (pilot JFET + BSS138, WP-F).** Pilot **MMBFJ270** (onsemi MMBFJ270/D Rev 2: V_GS(off) 0.5–2.0 V; was MMBFJ177, 0.8–2.5 V). With a 2.5 V pilot the path never held at −20 °C (CS Q2). v0.3's own check (R3 §2) found a second trap: with a pilot at 2.0 V the release switch can lock the pump into a short at power-up whenever the clock is faster than ≈ 1.6 kHz in the cold; the 470 Ω pump resistor removes it (a resistor in the release drain or a stronger pilot pump does not). Simulated with the clock **stopped low**: gate past the 2N170D2's cold cut-off **1.4–22 ms**, "not held" 1.4–19 ms after the last edge (slowest with the pilot at its 0.5 V minimum and fresh parts); stopped **high**, not released within 30 ms (the 1.01 MΩ still releases it in 0.7–1.6 s): the firmware rule "stop low" is essential (F-1). v0.2's "conducting within ≤ 10 ms" is withdrawn; firmware allows 25 ms. "Held" after power-up 19–33 ms.

**Proving "held": two sensors per stack (I-4).** Each sensor: 3.9 kΩ + **MMSZ4687 4.3 V** zener (was 4.7 V; onsemi MMSZ4678T1/D Rev 13: 4.09–4.52 V at 50 µA) + TLP187 photo-Darlington LED ∥ 22 kΩ, from the stack source to the bottom gate; each Darlington on **its own line** to sheet 09 (`HOLD_x_HELD_N`, `HOLD_x_HELD2_N`), each pulled up with **47 kΩ** (was 10 kΩ); `HOLD_OK` needs **all four** lines low. Thresholds (gate magnitude at which a sensor reads "held"): **4.95 / 5.62 / 6.12 V** at the low / nominal / aged-cold corners [SIM-done R3 §1] — 0.71 V above the IXTT2N170D2's worst cut-off (4.24 V at 70 °C), so the stack cannot read "held" while conducting; this threshold is **valid only with the 1,700 V devices** (the 10N100D2's 4.77 V would leave 0.18 V). At the cold corner with aged sensors both lines read solidly "held" (HELD_N ≤ 0.76 V) for clocks of 1.35–1.65 kHz, and chatter at 1.0 and 2.0 kHz [SIM-done R3 §2]. A shorted zener makes one sensor read "held" at ≈ 1 V; the other still needs ≥ 4.95 V. Two series zeners were rejected: the intact threshold (≈ 9.5 V) is above anything the pump delivers (CS Q1); two Darlingtons in series on one line give a 1.10–1.13 V low level against a 1.2 V input threshold (CS). Each sensor is read back separately (`HOLD_RB_x_N`, `HOLD_RB_x2_N`), so the self-test proves all four (§B14.5 T-7). Open, unpowered or missing opto, open pull-up: "not held" (R11). Remaining fail-to-"held" fault: a shorted Darlington, now caught by T-7 and needing a second fault to matter.

**Open item, measurement only:** the hold level at −20 °C and the 1.35–1.65 kHz window rest on hand-built models (BAT54 drops at datasheet maxima, X7R at 70 %; the zener temperature coefficient is ignored by ngspice 42); they must be measured at −20 °C.

**Over-voltage force (R-10).** Unchanged: `OVP_TRIP` turns on Q1111, which feeds both OVP-force opto LEDs from `3V3_SUP` (8.5 mA); each Darlington pulls its gate to ≈ −0.35 V and the stack conducts within 1.1–2.4 ms [SIM WP-F §6]. v0.3: because L2 now latches, the force persists until reset.

**Removed (R-9):** the v0.1 option of driving pump A from a 5 V buffer on `5V0_SYS`.

### B5.5 Stresses

| Item | Value | Rating | Source |
|---|---|---|---|
| Peak dump current per path | 0.42–0.57 A | ≥ 2 A (2N170D2 [BK]) | [SIM-done CS Q2] |
| Volts per device held, 2,200 V | 546–554 V | 1,700 V (1,598 V cold, assumed) | [SIM-done CS Q1] |
| Volts per device, one device shorted | ≤ 1,174 V | 1,598 V cold | [SIM-done CS Q1, Monte Carlo] |
| Energy per dump resistor, 2,200 V dump | 77–85 J one path; 40–42 J both | Ohmite EY 400 J [UNVERIFIED repetition] | [SIM-done CS Q2] |
| Peak power per resistor | 309–419 W | — | [SIM-done CS Q2] |
| Voltage per resistor at the start | 733 V for ≈ 0.13 s | 550 V working, 1,100 V 5 s overload | [SIM-done CS Q2; Ohmite to confirm] |
| Release failed, one path alone | 4.4–5.4 s; 13–27 J in one transistor at ≤ 268 W | FBSOA at mid voltages **not settled** | [SIM-done CS Q2] |
| Repetition | 0.61 W average per resistor at one dump per 132 s; 1.24 W at 20 dumps in 1,300 s | 2.5 W at 25 °C (derating curve not seen) | [CALC CS] |

### B5.6 Permanent bleeder and standing load

8 × 2.49 MΩ HV chip resistors (19.9 MΩ), its own string. Everything always connected in v0.3 [CALC]: each stack's sharing string (18.8 MΩ) in parallel with its per-device resistors (40 MΩ) = 12.8 MΩ, two stacks 6.40 MΩ; bleeder 19.9 MΩ; sense strings 40.2 and 39.8 MΩ → **3.90 MΩ on the board**; with the capacitor's internal 10 MΩ (D-65) **2.80 MΩ**. τ = 2.80 MΩ × 100 µF = **280 s**; 2,008.6 V → 50 V in ≈ **17 min** [CALC: 280 s × ln(40.2)]; standing loss while charged **1.44 W** at 2,008.6 V (v0.2: 6.65 MΩ, 0.61 W, 41 min) — ≈ 3 % of the charger's output, cost ≈ 0.1 s of charge time [EST].

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

**v0.3, simulated (BR Q3):** normal shocks heat the switches modestly (QHA ≤ 98 °C at 25 Ω, from the datasheet's Zθ). **Microchip publishes no turn-off (RBSOA), short-circuit or avalanche rating and no Zθ below ≈ 7 µs**, so whether the part survives interrupting ≈ 100 A at 2.4–2.7 kV, and how much microsecond energy it absorbs in its linear region, is **not settled** by simulation (BR "what simulation cannot settle"; §16 rank 1). The simulations use a behavioural electro-thermal model built from DS00005914A.

### B6.3 Gate drive (R-3: SN6505B replaced)

| Switches | Drive | Timing |
|---|---|---|
| QLA, QLB (low side, reference `LS_SRC`) | TI **UCC5350MC** isolated driver with Miller clamp; output side **+20/−5 V** from Murata **MGJ2D052005SC** on `BRIDGE_5V`; input side on `BRIDGE_3V3` (with hold-up) | ≈ 1 µs on/off; gate resistor sets ≈ 1 V/ns at the drain |
| QHA, QHB (high side, floating) | **Gated push-pull transformer drive (new):** a free-running 400 kHz two-phase clock (flip-flop divider from an oscillator on sheet 13, running whenever `BRIDGE_3V3` is up) is ANDed with the gated command (`G_HA` / `G_HB` from sheet 12) and drives a dual N-MOSFET push-pull stage into the centre-tapped primary of the made-to-order transformer T1301/T1302 (centre tap on `BRIDGE_5V`). **No soft-start: energy transfer starts on the first clock half-cycle after the command.** Secondary: full-wave rectifier → 18 V zener-limited → 100 Ω → gate, with **100 nF added gate-source capacitance**, an active **PNP turn-off** (base driven by the collapse of the rectified voltage), and a **10 kΩ gate-source bleed** as backup (R-23) | below |

The flip-flop guarantees equal half-cycles, so the transformer is not driven into saturation; gating is applied to both phases at once on a clock edge.

**v0.3 additions (I-8; BR changes 1–3).**
- **High sides off at once on a current fault.** v0.2 (and the capture, WP-D CI-10) kept the high sides on for 20–36 µs after an over-current or idle-current fault. With a shorted low side that let the opposite high side turn fully on: as drawn it absorbed 0.35 J (diagonal case, 1,950 V, 25 Ω) to **11–12 J at ≈ 240 A** in the same-side case (shorted QLA under QHA at the start of phase 1, a case missing from WP-G's failure table) [SIM-done BR Q1]: destroyed. Sheet 12 now has a **high-side kill latch**, set by the over-current or either idle comparator (after `RDY`) and cleared only by power-on reset, that removes `G_HA`/`G_HB` within ≈ 45 ns [CALC]. Planned ends, cap expiry and rail collapse keep the 20–36 µs delay so that the low side always opens first.
- **A fast idle-current path** directly on the shunt: OPA838 (gain 11, 300 MHz GBW, ≤ ±220 µV offset −40…85 °C, TI SBOS867E) and a TLV3501 at **3.02 A** (band 2.6–3.5 A [CALC]), ≈ 30 ns, as `IDLE_FAST_N`, ORed on sheet 12 with the slower INA240-derived 5 A comparator (kept for redundancy).
- **A carrier-loss detector on each high-side island.** When the carrier stops, the rectified node (1 nF ∥ 2.2 kΩ) used to hold the PNP turn-off back. A second, small rectifier (BAV70) charges 47 pF across 4.7 kΩ (τ ≈ 0.2 µs); while the carrier runs it keeps an MMBT3904 on, which holds a 2N7002 off; when the carrier stops, the NPN turns off and the 2N7002 — its gate fed from the **gate node** through 10 kΩ, clamped at 15 V — empties the rectified node through 100 Ω. Simulated (R3 §3): the detector's node stays ≤ 0.41 V through every 0.19 µs dead time (2N7002 threshold ≥ 1.0 V); after a stop the rectified node is below 1 V in **1.1–1.3 µs** (8.8 µs without) and the gate below 2 V in **3.9–7.0 µs** (7.2–11.5 µs without). (A first version powered from the rectified node stalled at the 2N7002's threshold.)
- **Together (changes 1 + 2 + 3)** the shorted-low-side cases put **10.9–12.3 mJ** into QHA, 93–100 °C on the pessimistic thermal estimate, in both the diagonal and the same-side case at 1,950 and 2,200 V [SIM-done BR Q1, with an ideal 100 Ω switch standing for the detector]. **Disagreement recorded:** R3's circuit-level island model starts the PNP within ≈ 1 µs even as drawn, where BR's behavioural PNP needed ≈ 2.7 µs; both are hand-built. The BR result is the more pessimistic and was used to decide; second-round S-1 re-runs BR Q1 with the circuit-level detector.

**Timing budget [EST from the RC values; v0.3 turn-off simulated as above]:**

| Event | Time |
|---|---|
| High-side turn-on: first rectified pulse ≈ 1.3 µs after `G_H*`; 105 nF through 100 Ω from ≈ 19 V (τ ≈ 10.5 µs) to 15 V | **≈ 20 µs** (≤ 40 µs with tolerances) |
| High-side turn-off: carrier stops; detector empties the rectified node; PNP discharges ≈ 100 nF through 22 Ω | gate < 2 V in **3.9–7.0 µs**, < 1 V in 8–12 µs [SIM-done R3 §3] |
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
- **Wiring inductance** between D_FW, the bulk of the capacitor connection and the switches: an **RC snubber `HV_BUS`–`HV_RTN`** (2 × 10 nF X7R in series + 10 Ω, as captured) close to the switches. **v0.3 (BR Q2):** the turn-off spike is set mainly by the **cable to the off-board capacitor**: interrupting a short at 2,200 V gave 2,670–2,890 V, and **3.2–3.7 kV — over the 3.3 kV rating — with 0.5–1 µH of cable** [SIM-done]. A bigger snubber made it worse (47 nF discharged 112–211 A through the switches at every phase start). Fix: a **local film capacitor `HV_CAP_P`–`HV_RTN`** at D1301's cathode, two parts in series so one short cannot short the main capacitor: 470 nF simulated gave 2,497–2,511 V with 0.5 µH; a 1 µF ≥ 2.2 kV part could not be verified, so v0.3 specifies **2 × Cornell Dubilier 940C30P15K-F (0.15 µF, 3,000 V DC each) in series = 75 nF**, each with a 10 MΩ balancing resistor; **75 nF is below the 100 nF simulated** and must be confirmed (second round S-2). No RCD clamp.
- **D_FW is two GC05MPS33J in series** (D1301 + D1302, WP-G), as captured.
- **Every turn-off case:** normal phase end, over-current trip, idle-current trip, cap expiry and rail collapse all end with the **low side opening first** (§B13.5), so all use the same D_FW path. A high side opening under current happens only in fault cases (a shorted low side); its energy goes into that high side's die, which is stated in §13.
- **Interrupted current at the over-current trip (v0.3):** 92–112 A as drawn (90 A trip), above the switch's 107 A pulse rating; with the trip lowered to **80 A** (band 74–86 A) **≤ 97 A** and 2,430 V with the local film capacitor [SIM-done BR Q2]. Coil energy at 97 A: ½ × 30 µH × 97² ≈ 0.14 J into R_S and D_FW [CALC].

**Internal resistance [CALC].** Other than R_S: capacitor ESR 0.13 Ω; two switches hot 2 × 0.15 Ω; shunt 0.002 Ω; relay contacts 2 × 0.05 Ω [UNVERIFIED]; L_S winding ≈ 0.08 Ω (below); leads/connector/pads ≈ 0.2 Ω [EST] → ≈ **0.81 Ω**. **R_S = 3.9 Ω** → total ≈ **4.7 Ω** (the model's 5 Ω, D-23). Hard short: 2,182 / 4.7 ≈ **465 A**; with the coil the current never gets there before the trip (≈ 130 A peak).

**L_S (R-26, re-derived).** 30 µH, ≥ 450 A without losing more than 20 % of its inductance. Air-core one-layer 55 turns on 20 mm × 30 mm (v0.1) does **not** fit with ≤ 50 mΩ of wire (needs ≈ 88 mm of length — the review is right). Options [CALC by Wheeler's multi-layer formula, EST sizes]: **air-core, two layers, ≈ 43 turns of 1.0 mm² on a 33 mm former, ≈ 35 mm long: ≈ 30 µH, ≈ 75 mΩ, ≈ 35 cm³, ≈ 40 g** (copper ≈ 4.5 m × 8.9 g/m); or **gapped powder-core toroid** (sizing for 450 A without saturation left to the maker): **≈ 10–15 cm³** [EST]. The made-to-order specification (§14) states performance, not construction. The 75 mΩ is inside the internal-resistance budget above.

### B6.6 Outputs, current sense, test load

- **Output dividers** (also the pull-downs): 4 × 249 kΩ + bottom leg per output → `OUT_A_TAP`, `OUT_B_TAP` (1:1,000). Idle outputs sit at ≤ ≈ 35 V (35 µA leakage × 1 MΩ, R-25); a shorted high side reads the full capacitor voltage. **Tap clamps go to `GND` (zener/TVS at the tap), not to a reader's rail** (R-21).
- **Shunt** 2 mΩ (pulse-rated) `LS_SRC` → `HV_RTN`.
- **Fast over-current comparator** directly across the shunt (100 Ω / 1 nF, TLV3501), threshold **80 A** (v0.3: R1351 196 kΩ; 3.3 V × 10.0/206 = 160.2 mV = 80.1 A [CALC]; band ≈ 74–86 A) → `OC_DET_N` to sheet 12. The highest planned peak is 67.4 A (2,000 V, 25 Ω) [SIM-done BR Q3]; at the firmware over-voltage limit's top (2,098 V) ≈ 70.7 A [CALC, ∝ V].
- **Idle-current comparators** → sheet 12 (current flowing when no low side is enabled = a shorted low side, R-14): the INA240-derived one at **5 A** (`IDLE_I_DET_N`, R1358 267 kΩ for the new scale) and the **fast 3 A** one on the shunt (`IDLE_FAST_N`, §B6.3).
- **Waveform amplifier (R-24):** TI **INA240A1** (gain 20) **powered from `BRIDGE_5V`**, followed by a 13.3 k / 20.0 k divider → `I_SHOCK` = **24.0 mV/A** (v0.3; was 26.7 mV/A). The headroom problem was not the shock current but the saturated amplifier: as drawn it could drive the main controller's ADC pin to 3.55 V, 0.29 V above VDDA; now ≤ 3.20 V [SIM-done LR §5.1]. Self-test current 0.29 A reads 6.7 mV; firmware treats ≥ 0.1 A as "current present".
- **Low-side supply monitor (v0.3, BR change 9):** a TLV7031 compares the MGJ2's +20 V rail (through 100 k / 12.1 k) with 0.554 × `BRIDGE_3V3`: **`LSD_OK` high above 16.9 V** (16.5–17.3 V [CALC]). If the MGJ2 alone failed, the UCC5350 kept driving until its own 10.3–11 V lockout and QLB sagged through its linear region for up to ≈ 0.6 ms at up to 50 A, 158–181 °C [SIM-done BR Q4]; sheet 12 now treats `LSD_OK` low like a `BRIDGE_5V` collapse (≈ 5 µs). MGJ2 output at the lowest `BRIDGE_5V` ≈ 18.8 V [EST, output assumed proportional to input; regulation not read].
- **Bridge test load:** cold-switched Cynergy3 **D-HR n/o reed relay** K1301 (10 kV stand-off, 5 V coil [part code UNVERIFIED]) and a **200 Ω** ceramic resistor R1303 across the outputs. Enabled **only in test mode** (§B14). At ≤ 60 V test voltage the current is ≤ 0.3 A (≥ 8 mV on `I_SHOCK`) and the stored energy ≤ 0.18 J. If K1301 were welded during a real shock, R1303 would take 2 kV / 200 Ω = 10 A, ≈ 20 kW for up to 15 ms (≈ 300 J worth of rating needed — it would fail); a welded K1301 is therefore detected in every self-test (outputs joined at rest, §B14.4) and the firmware refuses to shock.

---

## §B7. Patient relay and pad interface (sheet `14_patient_interface`)

### B7.1 Why a relay

Unchanged: with charging during analysis, the outputs sit beside a charged capacitor while the front ends measure; a shorted high-side switch would put up to 2 kV on a pad, through the patient, during CPR (≈ 2 mA through the 1 MΩ pull-down path [CALC]); bridge leakage would reach the front ends. The relay keeps the pads on the front ends at rest and adds a third series element.

### B7.2 The relay and its coil supply (R-12)

**K1401: DPDT HV relay, cold-switched.** At rest: pads → `PT_FE_A/B`. Energised: pads → `HV_OUT_A/B`. Reference candidate **Gigavac G17** (2C, designed as a defibrillator patient relay; coil 12 V, 12 Ω ± 10 % ≈ 12 W, intermittent 100 ms maximum on, 10 % duty — Gigavac datasheet, search extract, figures confirmed by the review). Its contact pulse rating, size and stand-off are **[UNVERIFIED]**. **v0.3 (I-6, D-64): no catalogue relay found publishes a pulse rating or has a position contact linked to both poles, so K1401 is a made-to-order part**; its requirements are in `docs/requirements/patient-relay.md`, and the drawing keeps a clearly marked placeholder symbol.

**Coil supply (new):**
- A **12 V boost on sheet 14, fed from `VBAT_SYS`** (not from `BRIDGE_5V`, which cannot supply ≈ 2.8 A). **v0.3 (I-15):** as drawn, `K_EN` started the input switch, the boost and the coil together; filling the 6.6 mF reservoir drew **8.7–9.9 A for 15–33 ms** and, with a cold nearly empty battery, pulled `VBAT_SYS` to 4.35 V, `5V0_SYS` to 4.24 V and `BRIDGE_5V` below the sequencer's rail-collapse trip; the relay then took up to 31.8 ms to transfer [SIM-done LR §3.1]. Now the **reservoir is filled beforehand**: the input switch and boost are enabled by a separate **`K_SUPPLY_EN`** from the main controller, raised when charging for a shock starts (≥ 0.2 s before `RELAY_CMD`), with the switch's slew capacitor 680 nF and the boost limit ≈ 2.0 A (R1406 515 kΩ): battery peak **2.5–2.6 A**, reservoir at 11 V after 74–99 ms [SIM-done LR §3.2]. `K_EN` from the sequencer (which requires `BRIDGE_5V`, shock mode, `RELAY_CMD`, rail ready and the coil timer not expired, §B13) now enables only the economiser that drives the coil, and its fall opens the coil drive directly (CI-8). **Transfer 15.9 ms after `K_EN` in every battery case** [SIM-done LR §3.2, relay mechanics 10 ms EST].
- **Economiser:** full 12 V for ≤ 20 ms (pull-in), then hold at ≈ 50 % (≈ 3 W) [EST; hold level from the relay maker].
- **Hold-up:** a reservoir on the coil supply so that if the boost or `VBAT_SYS` fails mid-sequence, **the relay stays closed for ≥ 30 ms**. v0.2's 1.7 mF assumed a constant 3 W; WP-H's capture with the pull-in phase included uses **6.6 mF** (2 × 3,300 µF), 48–60 ms of hold-up [CALC WP-H §3.5].
- **On-time cap and start window (as captured, WP-D DEP-6):** one RC with two thresholds: start window **43.4–51.2 ms** and coil cap **83.7–96.4 ms** after the coil is first energised [SIM-done LR §1.3]; the latest sequence end plus coil delay is ≈ 80 ms, so every sequence ends before the coil cap.
- **Fast release:** zener + diode suppression (≈ 24 V) instead of a plain diode, since release time is in the timing budget.
- **Budget (v0.3):** ≈ 2.0 A for ≈ 0.1 s while the reservoir fills, then the economiser's hold current; battery peak 2.5–2.6 A including the other loads [SIM-done LR §3.2].

**Position sense (new).** **The relay must have a force-guided low-voltage auxiliary contact** (a third, mechanically linked changeover or a normally-closed contact closed only at rest) → `K_REST_N` (low = at rest). Read by the main (pull-up on 04) and by the sequencer (Ioff buffer on 12, no pull). If the qualified relay has no such contact, this becomes a made-to-order requirement (§14). The sequencer refuses to start a shock sequence unless the relay has **left** rest for ≥ 10 ms, and refuses a test sequence unless the relay **is** at rest. **v0.3:** a contact bounce while the armature lands let the 10 ms count restart from a few hundred millivolts (shock allowed 4.1–8.4 ms after the last bounce); a 10 kΩ + 1 nF filter and a transistor that empties the timer while the contact is at rest restore **11.0–11.3 ms** [SIM-done LR §1.4]; and the sequencer must have seen the contact **at rest when `K_EN` rose**, so an open or unsoldered contact can never look like a transfer (WP-H C-6).

**Requirements:** `docs/requirements/patient-relay.md` (v0.3): ≥ 70 A peak exponential, ≤ 15.5 ms per shock, ≤ 20 ms in a firmware failure; ≥ 120 A for 10 µs (output short ended by the 80 A trip); 20 shocks in 1,300 s; ≥ 5 kV across open contacts, between poles and to coil and position contact; ≤ 50 mΩ per pole; operate ≤ 10 ms and release ≤ 15 ms at −20…+65 °C; position-contact bounce ≤ 1 ms; ≥ 50,000 mechanical operations (the self-test now moves it, D-62).

### B7.3 Pad connector, identity, presence

J1401 made to order: `PT_PAD_A`, `PT_PAD_B` (HV) and three separated LV contacts `PAD_ID` (1-Wire), `PAD_ID_RTN`, `PAD_PRESENT`. 1-Wire EEPROM in the pad plug (ADI **DS2431** class: 64-bit unique ID = `pad_set_id`, D-57). Series resistor and TVS on each LV pad line. Pad-area NTC `NTC_PADS` on this sheet.

### B7.4 Leakage

At rest the patient touches only the front ends (DC blocked in the drive taps, nanoamps in the sense taps); auxiliary current ≈ 0.34 mA RMS at 64 kHz plus the supervisor's carrier, against an IEC limit of several mA at those frequencies [UNVERIFIED limit]. The bridge reaches the patient only while the relay is energised (≤ 96.4 ms, hardware-capped).

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
| `HOLD_A_HELD_N`, `HOLD_B_HELD_N`, `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N` (v0.3) | 11 | opto, low = held | 47 kΩ pull-ups (+ 1 MΩ backups) to `5V0_SYS` on 09; `HOLD_OK` = all four low |
| `LOWV_OK` | 07 | static, high = capacitor < 60 V | pull-down on 09 |
| `SEQ_HOLD` | 12 | static, high = sequence running | pull-down on 09 |
| `CHG_SESSION`, `CHG_RUN` | main | static | Ioff buffers on `3V3_MAIN` |

**Dynamic detector specification (each `_CLK`):**
- **Accepted band (permit and test-permit detectors):** 1–5 kHz square (nominal 2 kHz). *The hold clocks run at 1.5 kHz ± 10 % (v0.3, §B5.4).* Two-pole RC low-pass (corner ≈ 8 kHz) then AC coupling (corner ≈ 200 Hz) then a diode pump into a hold capacitor, then a Schmitt buffer.
- **Rejection:** at ≥ 100 kHz (a pin left on a clock or free-running timer output) the pump sees < 1 % amplitude → no output [CALC: two poles at 8 kHz attenuate 100 kHz by ≈ (100/8)² ≈ 156×]; DC gives none.
- **Rise:** output valid ≤ 3 ms after the first edge.
- **Hold-up:** stays valid **≥ 3 ms** after the last edge (3 × the firmware's maximum permitted gap of 1 ms).
- **Collapse:** output invalid **≤ 8 ms** after the last edge.
- **v0.3 (LR §1.2):** as captured, the collapse took up to **8.28 ms** at the hot corner (the pump's coupling capacitors enlarge the hold node while it decays); with the four hold bleeds at **64.9 kΩ** (was 71.5 kΩ): hold ≥ 4.07 ms, collapse ≤ 7.50 ms [SIM-done]. A line that changes level once, a single pulse longer than ≈ 50 µs, or the supervisor powering up gives **up to 7.5 ms of permit that should not exist**; this cannot be designed out with values (one pump stroke must charge the hold node above threshold). It cannot reach the output stage, because the sequencer's rail-ready delay is ≥ 20.7 ms; firmware ignores `BRIDGE_ARMED_N` lows shorter than 10 ms (F-4).
- Supply from the consenting controller's rail, so an unpowered controller gives no output.
- **Firmware rule (§F):** square wave 1–5 kHz from code that stops when the control loop stops; no gap between edges longer than 1 ms; never a free-running timer.

### B8.2 Bridge power

```
5V0_SYS ─ SW1 (EN = SHOCK_REQ) ─ SW2 (EN = PERMIT_OK, or TEST_OK·LOWV_OK·NOT PERMIT_OK) ─ SW3 (EN = HOLD_OK) ─ SW4 (EN = WIN_OPEN) ─┬─ BRIDGE_5V
5V0_SYS ─ SW5 (EN = SEQ_HOLD · NOT TEST_MODE) ─────────────────────────────────────────────────────────────────────────────────┘
```

Six TI **TPS22918** load switches (5.5 V, 2 A, quick output discharge [BK; confirmed by the review]); every EN has a 100 kΩ pull-down. SW2 is two switches in parallel (SW2a permit, SW2b test permit), so each can be proven to block (§B14). SW5 is the D-59 sequence hold (§B13.2).

**Maximum permit window (R-4, R-11).** `WIN_OPEN` is high from power-up until a latch `WIN_EXPIRED` is set **150 ms** (131.8–161.5 ms simulated, LR §1.3) after the rising edge of the consent chain (SW1–SW3 all on); the timer is an RC charged from that edge into a comparator whose output **sets a latch** — non-retriggerable. The latch clears only after **both** `SHOCK_REQ` and the permit/test detectors have been absent **continuously for ≥ 50 ms** (51.2–60.3 ms simulated), so a toggling consent cannot restart the window. **v0.3 (LR §1.4 race B):** the window-start latch's clock is delayed 100 µs (10 kΩ, 10 nF, Schmitt buffer), because a consent-chain edge within 15.7 µs of re-arm was otherwise lost and the window never started for that attempt [CALC; second round S-5]. A shock sequence must **start** while `WIN_OPEN` is high; once started, SW5 carries the rail to the end (§B13).

**`TEST_MODE`** = `TEST_OK` AND `LOWV_OK` AND NOT `PERMIT_OK` (§B14) → to 12.

**Charger gates:** `VTRANS_EN` = `CHG_SESSION` · `HOLD_OK` · NOT `TEST_MODE`; `CHARGE_EN` = `CHG_RUN` · `HOLD_OK` · NOT `TEST_MODE` → to 10.

**Readbacks to the main:** `BRIDGE_ARMED_N` (open drain, low while `BRIDGE_5V` present), `HOLD_RB_A_N`, `HOLD_RB_B_N`, and (v0.3) `HOLD_RB_A2_N`, `HOLD_RB_B2_N`: open-drain copies of each of the four held sensors. `HOLD_OK` = all four sensor lines low.

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
| Extra comparator | **`LOWV_OK`**: capacitor < **60 V** (tap < 60 mV; comparator offset ≤ 1 mV → ±2 %) on `3V3_MAIN`, push-pull, high = low voltage (new, §B14) | **`OVP_TRIP`** at **2,107 V, latching** (v0.3; reset by `OVP_RST`, test injection `OVP_TEST`) |

**Bottom-leg fault (R-27 correction).** One open resistor in a two-resistor bottom leg **doubles** the tap voltage (the reading doubles), it does not halve it. On the main chain the reading disagrees with the supervisor's → the supervisor latches. On the supervisor chain the OVP comparator then trips at ≈ 1,054 V true, so every adult charge is dumped: detected, but a "no shock" fault (§13). The clamp keeps the tap below 3.6 V either way.

**Open tap wire (v0.3, I-23).** As drawn, an open `VCAP_M_TAP` read "below 60 V" for ever. A **1 GΩ resistor (R718, Vishay CRHV1206AF1G00FKE5) from `VREF_M` to the follower input** drives an open tap's reading up: `LOWV_OK` falls within 0.16–0.37 s; the intact chain gains a fixed +0.22 V offset at 0–50 V that calibration removes [SIM-done LR §5.2].

**Drift (I-16).** The main chain meets 1 % + 7 V only if its HVCR2512 string drifts ≤ 0.59 % over life [SIM-done LR §5.3, confirming WP-C]; a thin-film string would allow ≈ 0.92 %. Still owner question 4 (§17).

**Error budget [CALC, unchanged]:** main 0.92 % + 1 V; supervisor 0.51 % + 1 V; allowance at 1,950 V 1.36 %. Life drift dominates the main chain (owner question 4). Exact ratios as captured: 1:998.5 (main), 1:1,002.0 (supervisor) (WP-C C-1).

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

I²C: pull-ups to `3V3_MAIN` on sheet 04; with the main off the bus is held low. **Confirmed (WP-A C-4, I-19):** with `nConfig.COMMSH = 1` the MAX17320 shuts down (FETs off, device dark) when SCL and SDA are held low; it must be configured `COMMSH = 0`.

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
| 0a | main | (v0.3) `K_SUPPLY_EN` high when charging for the shock starts, ≥ 0.2 s before step 5: the coil reservoir fills at ≈ 2 A | — |
| 1 | main | `SHOCK_REQ` high | — |
| 2 | supervisor | `PERMIT_CLK` running (its checks done at grant) | — |
| 3 | 09 | SW1–SW4 close → `BRIDGE_5V` up → `BRIDGE_ARMED_N` low; 150 ms window starts | `HOLD_OK`, `WIN_OPEN` |
| 4 | 12 | Power-on reset of every sequencer latch; `RAIL_READY` 21.6–26.7 ms after `BRIDGE_ARMED_N` [SIM-done LR §1.3] | `BRIDGE_3V3` up |
| 5 | main | `RELAY_CMD` high **30 ms after `BRIDGE_ARMED_N`** (v0.3) → 12 raises `K_EN` (only after `RAIL_READY`) → economiser on → K1401 moves (15.9 ms simulated); coil timer starts | shock mode (not `TEST_MODE`) |
| 6 | 12 | waits for `K_REST_N` high (relay left rest; seen at rest when `K_EN` rose) + 10 ms (11.0–12.2 ms); the main waits **14 ms after the last low sample** | aux contact |
| 7 | main | `CMD_HA` high (lead ≥ 100 µs) | 12 passes `G_HA` (leg B low side off) |
| 8 | main | `CMD_LB` high → **phase 1 starts**; 12 passes `G_LB` only if: `RAIL_READY`, `WIN_OPEN`, `BRIDGE_5V` ≥ 4.5 V **and `LSD_OK`**, relay transferred ≥ 10 ms and coil start window open, `G_HA` on ≥ 50 µs, state = idle, no fault. **At this edge 12 sets `SEQ_ACTIVE` (latched) and raises `SEQ_HOLD`; the 25 ms cap and the phase-1 cap start** | — |
| 9 | main | `CMD_LB` low (end of phase 1, T1) → state = gap | — |
| 10 | main | `CMD_HA` low, `CMD_HB` high (≥ 10 µs after LB low) | hardware lockouts |
| 11 | main | `CMD_LA` high at T1 + 0.5 ms → **phase 2**; 12 passes `G_LA` only in state = gap and after `G_HA` off ≥ 150 µs and `G_HB` on ≥ 50 µs; **the phase-2 cap starts at the first `G_LA` edge** | — |
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
4. **Caps (v0.3, D-63).** **Overall cap 25 ms** from `SEQ_ACTIVE` (23.8–26.2 ms simulated, LR §2), as v0.2: an RC started by a latch into a comparator, **non-retriggerable and not extendable**. **Per-phase caps, new:** phase 1 — an RC from the `SEQ_ACTIVE` latch (113 kΩ, 100 nF C0G ± 2 %, threshold 0.6345 × `BRIDGE_3V3`): **11.37 ms nominal, 10.75–11.85 ms** [CALC: 113 k × 100 n × ln(1/0.3655) = 11.37 ms; band = the −5.4/+4.3 % spread simulated for the identical 25 ms cap, LR §2]; it acts only until phase 1 is latched done. Phase 2 — an RC from a latch set by the first `G_LA` edge (76.8 kΩ, same C and threshold): **7.73 ms nominal, 7.31–8.06 ms**; it acts only until `DONE`. Either sets the fault latch (`BR_FAULT_N`) and the normal turn-off order. The firmware's longest phases drop to **10.0 ms and 6.7 ms** (aed-waveform; the design table's longest are 9.01 and 6.01 ms), so the caps' lowest values leave ≥ 0.6 ms margin. Why per phase: no single cap holds the energy near plan, because a 50 Ω patient reaches 150 J in ≈ 5.5 ms of uninterrupted pulse while a 175 Ω patient legitimately needs 9 ms of phase 1 (BR Q3); a cap on voltage drop or charge cannot do it either (BR Q3). The overall cap now only matters if a per-phase cap has failed; the self-test exercises it separately (§B14.5 T-9). **(The v0.1 part, SN74LVC1G123, is retriggerable — R-4 is right.)**
5. **Start window:** a sequence may start only while `WIN_OPEN`, `RAIL_READY`, and the relay has been transferred ≥ 10 ms and energised ≤ 60 ms.
6. **Fault latches** (cleared only by power-on reset): over-current (`OC_DET_N`, **80 A**); **idle current** (`IDLE_I_DET_N` > 5 A **or `IDLE_FAST_N` > 3 A** while no low-side enable is on — a shorted low side, R-14); overall or per-phase cap expired; `BRIDGE_5V` under-voltage (< 4.5 V, 4.39–4.60 V) **or `LSD_OK` low** during a sequence. Any of them → `BR_FAULT_N` low, `SEQ_DONE`, turn-off order. **v0.3: an over-current or idle fault also sets the high-side kill latch, which removes both high-side enables at once** (§B6.3); the other faults keep the low-side-first order.
7. **Relay coil:** `K_EN` = `RELAY_CMD` · `RAIL_READY` · NOT `TEST_MODE` · NOT coil-cap-expired · NOT (done + 1 ms) (WP-D DEP-7). Coil cap 83.7–96.4 ms, latched. `K_EN` drives only the economiser; the coil supply is `K_SUPPLY_EN` (§B7.2).
8. **Test relay:** `BTEST_EN` = `BTEST_CMD` · `TEST_MODE`.
9. **Mode checks:** in shock mode `G_L*` require the relay transferred (and seen at rest when `K_EN` rose); in test mode they require the relay **at rest** and the test relay closed ≥ 3.5 ms (v0.3: the drawn timer could expire at 1.88 ms, before the test relay's 3 ms operate time; now 4.74–5.52 ms with a TLV7031, LR §1.3).

### B13.4 What ends a pulse

| End | Trigger | Turn-off |
|---|---|---|
| Planned | `CMD_LA` falls (phase 2 end) | §B13.5 |
| Phase-1 cap | 10.75–11.85 ms after the phase-1 start | §B13.5; `BR_FAULT_N` |
| Phase-2 cap | 7.31–8.06 ms after the phase-2 start | §B13.5; `BR_FAULT_N` |
| Overall cap | 23.8–26.2 ms after start (backstop) | §B13.5; `BR_FAULT_N` |
| Over-current | ≥ 80 A (74–86 A) on the shunt | low sides off within ≈ 65 ns of the comparator; **high sides off at once** (≈ 45 ns); `BR_FAULT_N` |
| Idle current | ≥ 3 A (fast) or ≥ 5 A (INA path) with no low side enabled | **high sides off at once**; `BR_FAULT_N` |
| Rail collapse | `BRIDGE_5V` < 4.5 V or `LSD_OK` low | §B13.5 using `BRIDGE_3V3` hold-up; `BR_FAULT_N` |

### B13.5 Turn-off order and supply collapse

**Order, every case:**
1. `t0`: both low-side enables low → UCC5350 outputs to −5 V, Miller clamp on; current stops within ≈ 1 µs; L_S freewheels through D_FW (§B6.5).
2. `t0 + 20 µs`: both high-side enables low → carriers stop → high sides off (zero current) within ≈ 10 µs.
3. `t0 + 1 ms`: relay coil off → contacts open with no current (≈ 5–15 ms).
4. `t0 + 15 ms`: `SEQ_HOLD` low → rail falls if consent is gone.

**If `BRIDGE_5V` fails mid-phase (any cause):** an under-voltage detector on sheet 12 (4.5 V) triggers step 1 at once. The logic, the UCC5350 input sides and the over-current latch run from **`BRIDGE_3V3`, which has ≥ 2 ms of hold-up** (an LDO with a reservoir sized for the logic's ≈ 60 mA [EST: ≈ 220 µF from 5 V to the LDO's dropout]), so they stay alive long enough to turn the low sides off first. The UCC5350 output sides run from the MGJ2's output capacitors and, if those sag, their own under-voltage lockout pulls the gates low — also "off". The high-side gates are held by their 100 nF capacitors — **so the high sides stay on until after the low sides are off**, which is the safe order. The relay coil has its own ≥ 30 ms hold-up from a different supply (`VBAT_SYS`), so it does not open under current. **[SIM-done BR Q4]:** in every collapse case simulated (BRIDGE_5V in 1 µs, 100 µs, 2 ms; logic supply lost) the low side opened first, 1.1–193 µs after the event, with ≤ 0.15 µs in its linear region and no shoot-through (off-switch gates never above −4.2 V / +0.03 V). The one gap, the MGJ2 failing alone (undetected, QLB partly on for ≈ 0.5 ms, 158–181 °C), is closed by `LSD_OK` (v0.3).

### B13.6 Worst-case energy if consent is withdrawn at any instant

| Withdrawal arrives | Delivered |
|---|---|
| Before `SEQ_ACTIVE` (any time up to step 8) | **0 J** (rail removed; relay returns cold) |
| After `SEQ_ACTIVE` | **the planned pulse**, unchanged (149–154 J adult, 50–54 J child simulated, BR Q3a) — withdrawal has no effect by design |
| After `SEQ_ACTIVE`, **plus** a fault that keeps a low side on (frozen timer output high), **as v0.2 (25 ms cap only)** | **up to 180.0 J** monophasic at 1,950 V (worst load ≈ 100–125 Ω, not 175 Ω as v0.2 said), **≈ 189 J at 2,000 V** [SIM-done BR Q3b]; v0.2's 177 J at 175 Ω is confirmed for that load |
| Same fault, **v0.3 per-phase caps** (D-63) | **≤ 170.9 J at 1,950 V** (50 Ω, phase 1 stuck); **≤ 181.4 J at 2,009 V**, the top of the at-plan window [SIM-done R3 §4, caps at the top of their bands]. Below ≈ 75 Ω the capacitor's own energy is the limit, so no time cap does better |
| Child mode, same fault, v0.3 | ≈ 60 J at 1,150 V, ≈ 64 J at the top of the child window (1,192 V) [CALC, single-RC model: 66.1 J × (1 − e^(−2 × 11.85/5.48)) × 50/54.8 = 59.5 J; × (1,192/1,150)²], against 50 J |

Holding the frozen-controller case near 150 J would need a **delivered-energy limit independent of the main firmware** (patient voltage × current, integrated, tripping the fault latch at ≈ 160 J, in hardware or by the supervisor). It is **not adopted** in v0.3: owner question Q-57 states the trade-off. The fault row is a single fault (frozen timer) after consent was correctly given; it delivers at most the stored energy, monophasic, ≈ 14 % over plan at the nominal charge.

### B13.7 Status for both controllers

| Net | To | Meaning |
|---|---|---|
| `BRIDGE_ARMED_N` | main | rail up |
| `SEQ_ACT_M_N`, `SEQ_ACT_S_N` | main, supervisor | sequence started (current has been commanded): **consent checks are over** |
| `SEQ_DONE_M_N`, `SEQ_DONE_S_N` | main, supervisor | sequence ended (planned or by a latch) |
| `BR_FAULT_N` | main | ended by a fault latch (over-current, idle current, overall or per-phase cap, rail under-voltage, `LSD_OK`) |
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
| Sequencer order and caps (v0.3) | A long phase-1 command (15 ms) must end at the phase-1 cap; a long phase-2 command at the phase-2 cap; a long gap at the overall cap (§B14.5 T-8, T-9); a second `CMD_LB` must be ignored |

### B14.4 Test energy

At 60 V: ½ × 100 µF × 60² = **0.18 J**; peak current 60 / (200 + 4.7) = **0.29 A** (`I_SHOCK` ≈ 7.0 mV at 24.0 mV/A, ≈ 35 counts of a 14-bit 3.3 V ADC; 6.7 mV at the low-gain corner, LR §5.1) [CALC]. Firmware runs the test at ≈ 40–50 V.

---

### B14.5 Self-test steps with the real permit (v0.3, D-62 provisional; I-7)

**Why.** The test permit (D-61) cannot reach the parts that only a real shock uses: in test mode the patient relay cannot move and the sequence hold (SW5) is disabled. The capture (WP-D CI-4) listed single faults there that stay hidden until the first rescue shock and then **block every shock**: the real permit's detector or SW2a dead; the coil enable path, its timers, the relay-transfer timer, the position-contact or test-mode buffer stuck; SW5 / `SEQ_HOLD` dead (which would bring back the truncation risk D-59 removed).

**Conditions for every step below (D-62).** Both controllers have each measured the capacitor **below 50 V** on their own chain (firmware), the charger is off (`CHG_SESSION`, `CHG_RUN` low), the supervisor grants the real permit for ≤ 150 ms per step (F-16), and the bridge switches are commanded only in T-5, which needs both front ends to read the pads open. At < 60 V the capacitor holds ≤ 0.18 J [CALC: ½ × 100 µF × 60²]. Hardware support: `HOLD_RB_A2_N`, `HOLD_RB_B2_N`, `OVP_TEST`, `OVP_RST` (contract issue 3); everything else uses existing nets.

| Step | What the firmware does | Must see | Newly reveals (previously hidden) | When |
|---|---|---|---|---|
| T-1 real permit alone | S runs `PERMIT_CLK`; M no `SHOCK_REQ` | `BRIDGE_ARMED_N` stays high 20 ms | SW1 shorted under the real permit's path (was tested only with the test permit) | switch-on, daily |
| T-2 real permit + request | M raises `K_SUPPLY_EN`, then `SHOCK_REQ`; S `PERMIT_CLK` | `BRIDGE_ARMED_N` low within 10 ms and steady ≥ 10 ms | permit detector dead or drifted; SW2a open; their buffers (CI-4) | switch-on, daily |
| T-3 relay move | 30 ms after armed: `RELAY_CMD`; no `CMD_*` | `K_REST_N` high within 40 ms (transfer time logged; expect ≈ 16 ms); low again within 50 ms of `RELAY_CMD` low | `K_EN` path (`RAIL_READY` gate, coil-window and coil-cap stuck closed, `TM` buffer stuck high), `K_SUPPLY_EN`, input switch, boost, reservoir, economiser, coil, relay mechanism, position contact and its `K_WAS_REST` latch; slow relay | switch-on, daily |
| T-3b coil cap (separate permit) | as T-3, `RELAY_CMD` held 110 ms | `K_REST_N` back to rest 83.7–96.4 ms after `K_EN` (plus release time) | coil cap stuck "never expires" | weekly |
| T-4 refused start | with the relay transferred: `CMD_LB` edge **without** `CMD_HA` | `SEQ_ACT_M_N` stays high; `I_SHOCK` ≈ 0 | lead gate (`HA_ON50`/`LB_LOCK`) stuck in shock mode | daily |
| T-5 dry shock-mode run | only docked, both front ends reading the pads **open**: full shock-mode sequence (1 ms + 0.5 ms + 1 ms); M and S withdraw consent as soon as `SEQ_ACT` goes low | `SEQ_ACT`, `SEQ_DONE` low, `BR_FAULT_N` high, `I_SHOCK` ≈ 0 (open pads); `BRIDGE_ARMED_N` stays low until 13.5–16.2 ms after the end, then high; `K_REST_N` back at rest | **SW5 / `SEQ_HOLD` dead** (truncation), shock-mode start conditions (`XFER`, `MODE_OK`, `SHOCK_OK`), end-of-sequence coil-off | weekly (docked) |
| T-6 T-OVP | S raises `OVP_TEST` with both hold clocks running; then lowers it; then pulses `OVP_RST` | `OVP_TRIP` high within 1 ms; all four `HOLD_RB` lines high within 25 ms; `OVP_TRIP` **stays** high after `OVP_TEST` falls; low after the `OVP_RST` edge; holds return within 35 ms | OVP comparator stuck "no trip" (v0.2 fault 33); latch feedback open; **reset switch shorted** (CS's latent fault); Q1111 or either force opto open | daily |
| T-7 four sensors | in the discharge-path step, release M's hold, then S's | both A lines (then both B lines) high within 25 ms; the other pair stays low | a shorted Darlington in any one of the four sensors (fault 9) | every self-test |
| T-8 per-phase caps | test mode, 40–50 V into the 200 Ω load: `CMD_LB` held 15 ms; then (new permit) phase 1 = 1 ms and `CMD_LA` held 12 ms | current ends 10.75–11.85 ms after the phase-1 start, then 7.31–8.06 ms after the phase-2 start, each with `BR_FAULT_N` low | a per-phase timer that never expires or has drifted | daily |
| T-9 overall cap | test mode: phase 1 = 1 ms, then wait in the gap (`CMD_HB` on, no `CMD_LA`) for 30 ms | `BR_FAULT_N` low 23.8–26.2 ms after the start; a later `CMD_LA` refused | the overall cap (otherwise masked by the per-phase caps) | weekly |

**Still hidden after v0.3** (each needs a second fault to matter unless stated):
- Over-current comparator (80 A) and both idle comparators (3 A and 5 A) failing "no trip": the test current is ≈ 0.3 A. A test injection into their inputs (WP-G F-6) would reveal them; not designed (§16).
- The high-side kill latch stuck "clear" (triggered only by those comparators).
- A carrier-loss detector failed open (slower fault turn-off only); failed short is found, because its high side then never turns on in the bridge test.
- `LSD_OK` stuck high (the MGJ2-only failure returns to v0.2 behaviour).
- In the discharge stacks: a shorted device, a shorted or open steering diode, an open per-device resistor or capacitor (they matter only when a device has also shorted).
- One of the two output rectifiers shorted; an open freewheel diode; a shorted R_S or L_S (WP-G F-3, F-4).
- A single pole welded at rest while the other transfers, **unless** the made-to-order relay's position contact is linked to both poles (required in docs/requirements/patient-relay.md).

## §11. Rail tree

```
Cells (2S, 5.5–8.4 V)
└─ MAX17320 FETs ─ VBAT_PACK ─ F101 25 A ─ VBAT_SYS ─┬─ TPS25982 eFuse (VTRANS_EN, ≈13 A, soft start) ─ F1001 ─ VTRANS ─ LT3751 / T1001 ─ D1001+D1002 ─ R1004 ─ HV_CAP_P
   (BQ25887 charges VBAT_PACK from VDOCK)            ├─ 0.82 Ω load (BAT_LOAD_EN, ≤ 250 ms)
                                                     ├─ 12 V coil supply (K_SUPPLY_EN) + 6.6 mF reservoir ─ economiser (K_EN) ─ K1401 coil
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
| Coil supply input (v0.3) | ≈ 2.0 A for ≈ 0.1 s while the reservoir fills (battery peak 2.5–2.6 A with other loads, SIM-done LR §3.2), then the economiser's hold | per shock | 0 |
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
| 1a | M freezes with its **timer still toggling** `CMD_*` (R-4) | No permit (as 1). If it froze after `SEQ_ACTIVE`, the sequencer allows one phase 1 and one phase 2, each under its own cap (v0.3), and the latched 25 ms cap | H (sequencer, caps) | ≤ planned pulse, or ≤ 177 J monophasic if a low side stays commanded on (§B13.6) |
| 2 | M freezes with `HOLD_M_CLK` stuck (either level), charger on | No clock → path A conducts within ≤ 10 ms → `HOLD_OK` false → `CHARGE_EN` and `VTRANS_EN` low (D-25) | H | Dump; S alarm |
| 3 | S freezes, `PERMIT_CLK` stuck | Detector collapses ≤ 8 ms | H | No permit (failure to shock); M reports supervisor lost |
| 4 | S hangs while a **free-running** timer drives `PERMIT_CLK` | If its frequency is outside 1–5 kHz the detector rejects it; inside the band it would look like a permit | Firmware rule (code-generated clock) + S watchdog + M sees heartbeat stop and does not request | Needs M to request as well: two faults. Residual: documented (§16) |
| 5 | M's rhythm analysis wrongly says "shock" | S does not judge rhythm | Two-method agreement in M | **Not caught by S or H** (accuracy limit, unchanged) |
| 6 | Stack device (any of four) fails short | **v0.3:** with gate steering and per-device R/C the others take ≤ 1,174 V (1,700 V parts, 1,598 V assumed cold) [SIM-done CS]. *v0.2's "≤ 727 V" was wrong: as drawn 2,141 V, destroying the bottom device* | Latent, safe | No effect on function |
| 6a | Steering diode, its bypass, or a per-device resistor/capacitor fails (v0.3) | Sharing degrades only if a device has also shorted | Latent | Two faults to matter |
| 7 | Stack A fails open (e.g. bottom device) | Path B still dumps | ST: holds released one at a time (τ); every dump τ fit | Detected at the next self-test discharge |
| 8 | One dump resistor opens | That path open | as 7 | as 7 |
| 9 | One hold-sense opto's **transistor shorted** (reads "held" always) | **v0.3:** the other sensor of that stack still has to read "held", so `HOLD_OK` is still proven | ST T-7: each of the four readbacks must follow its hold | Detected at the next self-test; two faults to matter |
| 9a | Hold-sense opto open / unsoldered / pull-up open (R-6) | Reads "not held" | H (fails safe) | Cannot charge; switch-on self-test fault |
| 10 | Hold pump fails (open capacitor or diode) | No bias → stack conducts; sensor "not held" | H | Cannot charge; detected at switch-on |
| 10a | Release switch fails open, or gate resistor open | Open release switch: the gate decays through 1.01 MΩ × 3.3 µF, so that path starts conducting 0.7–1.6 s late; alone it takes 4.4–5.4 s to empty the capacitor and puts 13–27 J into one transistor [SIM-done CS Q2; SOA not settled]; open gate resistor: the release switch still empties the gate | ST τ check shows a late start of that path's decay | Detected; the other path dumps meanwhile |
| 10b | Hold clock left stopped **high** (firmware fault) | Path not released within 30 ms; 1.01 MΩ releases it in 0.7–1.6 s [SIM-done R3] | Other path; firmware rule F-1 | Slow dump of one path |
| 11 | NT1101 opens | NT1102 keeps the star (two bonds); pumps and sensors are AC/opto-coupled anyway (R-9) | — | No effect; both opening is a second fault |
| 12 | FB divider top opens (child) | FB reads 0 → charge mode | Primary trip, OVP comparator, firmware | Contained by the latching OVP comparator (≤ 2,142 V), firmware and the last-resort trip |
| 13 | Charger primary MOSFET fails short | Battery short through the primary | **eFuse current limit → latch off** before the pack protector (R-16) | Charger dead (charge time-out); rest of device and S alarm unaffected |
| 14 | One output diode fails short (R-15) | The other diode blocks the full voltage | — | No effect; latent (found only by inspection/measurement) |
| 14a | Both output diodes short (second fault) | Capacitor discharges into the secondary | R1004 fusible resistor opens | Charger dead; transformer protected |
| 15 | Main chain reads 10 % low (string resistor shorted) | M would charge high | S disagreement (2 % + 24 V, 500 ms) → S releases, latches | Contained |
| 16 | Main chain bottom leg resistor opens (R-27) | Reading **doubles** | S disagreement → latch | No shock, detected |
| 16a | Supervisor chain bottom leg opens | S reading doubles; OVP trips at ≈ 1,080 V true | OVP dump; disagreement | Every adult charge dumped: no shock, detected |
| 17 | QHA (high side) shorted | `OUT_A_TAP` = capacitor voltage | M refuses to shock; ST bridge test | No shock; fault |
| 18 | QLB (low side) shorted (R-14), diagonal case | Test sequence shows current in the lead → found before any rescue. If it fails between tests: as v0.2 drawn, QHA turned fully on for 20–36 µs: 0.35 J (1,950 V) to 2.4 J (2,200 V) in QHA [SIM-done BR Q1]. **v0.3:** fast 3 A idle path + immediate high-side kill + carrier-loss detector: **≈ 11–12 mJ, 93–99 °C** (pessimistic estimate) | ST; H idle latches | Pulse truncated at the start; no shock; `BR_FAULT_N` |
| 18a | **QLA shorted, same side (v0.3, missing in v0.2 and WP-G)** | At phase 1 start QHA turns on straight across the capacitor: as drawn ≈ 240 A, 11–12 J in QHA, destroyed [SIM-done BR Q1]; v0.3: **10.9–12.3 mJ, 94–100 °C** | ST bridge test; H idle latches + kill | No shock; `BR_FAULT_N`; R_S may crack if the fix failed |
| 19 | Sequencer lockout gate fails (allows HA + LA) | Leg short | Over-current latch (80 A) and high-side kill; coil energy → D_FW (R-2) | Pulse lost; interrupted current ≤ 97 A [SIM-done BR Q2; survival UNVERIFIED: no RBSOA published] |
| 20 | A cap timer fails (never expires) | Only matters with a stuck low side (two faults); with one per-phase cap failed the other caps still bound the energy (≤ 180 J with the 25 ms cap alone) | ST T-8, T-9 | Latent; caught daily/weekly |
| 20a | High-side kill latch stuck clear, or a fast idle comparator "no trip" (v0.3) | Shorted-low-side energy reverts towards the v0.2 figures (row 18) | Not testable at 0.3 A (§B14.5) | Latent; two faults |
| 20b | Carrier-loss detector open / shorted (v0.3) | Open: fault turn-off ≈ 3–5 µs slower. Shorted (2N7002): rectified node loaded by 100 Ω, high side never on | Open: latent; shorted: ST bridge test | Open: two faults; shorted: no shock, detected |
| 21 | One interlock series switch shorted | Two consents remain | ST blocking tests (§B14.3) | Caught daily |
| 22 | SW5 (sequence hold) shorted | Rail held up without consent; sequencer lockout blocks any second sequence; a new first sequence still needs `WIN_OPEN`, relay transfer, and the main's commands | ST: rail must fall when consent is removed (`BRIDGE_ARMED_N`) | Caught daily; no energy without M commanding |
| 23 | Window latch fails open (window never expires) | Permit can hold the rail indefinitely | ST: a test permit held 200 ms must lose the rail | Caught daily |
| 24 | `BRIDGE_5V` collapses mid-phase | §B13.5: low sides off first on `BRIDGE_3V3` hold-up; high sides held by their gate capacitors; relay held by its own hold-up | H | Pulse truncated; safe order confirmed [SIM-done BR Q4] |
| 24a | MGJ2 low-side supply fails alone (v0.3) | As v0.2: undetected, QLB partly on ≈ 0.5 ms, 158–181 °C [SIM-done BR Q4]. v0.3: `LSD_OK` ends the sequence ≈ 5 µs after the rail falls below 16.9 V | H | Pulse truncated; `BR_FAULT_N`. `LSD_OK` stuck high is latent |
| 25 | `BRIDGE_3V3` hold-up capacitor open + rail collapse (two faults) | Order no longer guaranteed | — | Listed |
| 26 | High-side PNP turn-off fails open (R-23) | QHA still ≈ 10 V at phase 2 start → leg A short | Over-current latch; ST bridge test (current when QLA turns on after QHA "off") | Phase 2 lost; caught daily |
| 27 | Patient relay welded in the shock position | Front ends disconnected; `K_REST_N` reads "not at rest" | M checks `K_REST_N` before every analysis | "Attach pads"/fault; no shock |
| 28 | Patient relay fails to transfer (coil, boost, aux contact never opens) | Sequencer refuses to start (relay not transferred) | H | Refused; detected |
| 29 | Relay: one pole welded at rest while the other transfers | If the auxiliary contact is linked to the moving pole the sequencer starts; one pad is still on its front end, so the only current path is through that front end's protection taps (51 kΩ sense, 2.35 nF drive): ≈ 0 A on `I_SHOCK`, ≈ 2 kV across the taps (like the external-defibrillator test) | `I_SHOCK` ≈ 0 → Fault; a force-guided contact linked to **both** poles would catch it before the shock [UNVERIFIED relay construction] | No energy delivered; front-end stress; open item (§16) |
| 30 | Coil supply fails mid-sequence | Hold-up 48–60 ms (6.6 mF, WP-H) > 26.2 ms cap | H | Relay opens only after the sequence |
| 30a | `K_SUPPLY_EN` stuck low / high (v0.3) | Low: reservoir empty, relay too slow → start refused (≤ 40 ms abort); high: boost always on, battery drain, no movement without `K_EN` | ST T-3 (low); standby-current check (high) | Refused or drain; detected |
| 30b | Relay position contact open (v0.3, R11) | Looks "not at rest"; the sequencer requires it to have been at rest when `K_EN` rose, so it never counts as transferred | M (F-6), H | Refused; detected at switch-on |
| 31 | Bridge test relay K1301 welded | Outputs joined through 200 Ω at all times | ST (outputs follow each other) | M refuses to shock; if missed, a real shock would destroy R1303 (≈ 20 kW): listed |
| 32 | `LOWV_OK` comparator stuck "low voltage" | Test mode possible at high voltage only if S also gives a test permit, which S's firmware refuses above 50 V on its own chain | S firmware + H | Two faults needed; patient relay still blocked in test mode |
| 33 | OVP comparator fails "no trip", or its reset switch shorted (v0.3) | One over-voltage layer lost; firmware (≤ 2,098 V) and the last-resort primary trip (≤ 2,316 V, 2,409 V with overdrive, relying on the capacitor's surge rating D-65) remain | **ST T-6 (v0.3)** | Detected daily |
| 33a | `OVP_TEST` stuck high or `OVP_RST` stuck (v0.3) | Test stuck high: OVP permanently tripped, no charging; reset stuck high: edge-only, the latch still works [SIM-done CS Q4] | ST T-6; charge refused | Detected |
| 34 | Over-current comparator fails "no trip" | Leg short (another fault) not latched fast | Caps, idle-current latches | Two faults; not testable at 0.3 A |
| 35 | Mode switch pole B open (D-60) | Ceiling = adult in child position | `MODE_HW_RB_N` ≠ pole A → "not ready", recorded | Child dose still set by firmware and checked by S |
| 36 | FB MOSFET fails short | Adult ceiling always, also in child mode | Not seen: `MODE_HW_RB_N` reads the gate drive, not the FET | Child dose still set by firmware and checked by S; listed (§16 rank 9) |
| 37 | Cancel contact to M stuck | D-31 stuck rule; S's contact disagrees | M, S | Fault |
| 38 | `3V3_MAIN` fails | M dead; path A conducts | S heartbeat; S alarm | No shock, loud |
| 39 | `3V3_SUP` fails | S dead; path B conducts; no permit | M link loss | No shock; M speaks the fault |
| 40 | Hard short on `VBAT_SYS` upstream of the branch fuses | Pack protector trips; **everything dark, including the S alarm**, until the short is removed (then release needs no load < 1 MΩ, D-66) or the device is docked | — | **Silent failure** (Q-52, open) |
| 41 | `5V0_SYS` fails | No charger, no bridge rail; held-inputs read "not held" | M, S | No shock; S alarm and M prompts available |
| 42 | Storage capacitor shorts internally | Charger into a short; eFuse limits | Charge time-out | No shock; fault |
| 43 | Battery removed while charged | Both hold pumps stop (rails die) → both paths conduct; bleeder as backup | H | Empties in ≈ 1 s |
| 44 | Shorted relay coil driver (coil on without `K_EN`) | Relay energised at rest → pads on bridge outputs (pulled to `HV_RTN`); front ends disconnected | `K_REST_N` "not at rest" → M fault | No energy (bridge off); detected |

Gaps carried to §16: 4 (in-band free-running permit), 20a and 34 (current comparators not testable at low voltage), 29 (relay single-pole weld detection, now a relay requirement), 31 (welded test relay between self-tests), 36 (FB MOSFET short not read back), 40 (silent dark device).

---

## §14. Made-to-order and qualification parts, weight

| Ref | Part | Specification |
|---|---|---|
| C1101 | Storage capacitor | **docs/requirements/storage-capacitor.md** (v0.3, D-65): Appendix A plus a **10 MΩ internal bleed** and a **surge rating ≥ 2,420 V for ≥ 1 s, ≥ 100 events**; fault current ≈ 460 A |
| T1001 | Charger transformer 1:50, 7.5 µH +10/−5 % | §B4.5, **now with an efficiency requirement ≥ 90 %** |
| T1301, T1302 | High-side gate transformers | push-pull primary at 5 V, **400 kHz**, centre-tapped; rectified secondary 18–20 V at ≥ 30 mA; ≥ 2,200 V DC working for 10 years (target 2,750 V), ≥ 5 kV DC 60 s, partial-discharge inception ≥ 3.3 kV after humidity soak; inter-winding capacitance ≤ 10 pF; −20 to +85 °C; ≤ 2 cm³ |
| L1301 | Series coil 30 µH | ≥ 450 A with ≤ 20 % inductance loss; ≤ 80 mΩ; 2,200 V insulation to surroundings; air-core ≈ 35 cm³ or powder core ≈ 10–15 cm³ |
| K1401 | Patient relay | **docs/requirements/patient-relay.md** (v0.3, D-64): made to order; placeholder symbol on sheet 14 |
| C1347, C1348 (v0.3) | Local film capacitor, 2 × 0.15 µF 3 kV in series | catalogue part (CDE 940C30P15K-F, catalogue read by a research agent; size and mass **not read**) |
| J1401 | Pad connector | Appendix B3 + three LV contacts |
| — | Pad set with DS2431-class ID | Appendix B4 |
| — | Battery pack assembly; cancel cap | §B1.1, §B10 |
| R1004 | HV fusible resistor | ≈ 10 Ω, opens within 10 ms at ≥ 5 J, ≥ 3 kV across the open element (catalogue part likely; not found) |

**v0.3 additions [EST]:** local film capacitors 2 × 940C (size unread; 20–50 g assumed for two pulse film capacitors of this rating) — the largest uncertainty; second sensors, steering diodes, per-device R/C, detectors, comparators ≈ +5–10 g; heavier 2N170D2 devices: same package. **Total v0.3 ≈ +30–60 g → lower-risk set ≈ 910–970 g** (Q-56).

**Weight against the parts study's lower-risk set (v0.2) [EST]:** second stack pair and extra two devices, pumps, optos, release switches +20–25 g; L1301 +15–40 g (powder vs air core); D1301, D1002, R1004, eFuse +4 g; patient-relay coil supply with 1.7 mF hold-up +6–10 g; sequencer logic (≈ 22 small ICs) +3 g; K1401 heavier than budgeted +5–15 g; gate transformers instead of four isolated supplies −5 g. **Total ≈ +45–80 g → lower-risk set ≈ 880–910 g with pads** (parts study 832 g). Owner question 7.

---

## §15. Test points

All on low-voltage nets; **none on `HV_*`, `HV_FLOAT` or `PT_*`** (contract R9).

| Test point | Net | Sheet |
|---|---|---|
| Capacitor voltage, both chains | `VCAP_M_TAP`, `VCAP_S_TAP` | 11 |
| Output voltages | `OUT_A_TAP`, `OUT_B_TAP` | 13 |
| Shock current | `I_SHOCK` (24.0 mV/A, v0.3) | 13 |
| Consents and window | `SHOCK_REQ`, `PERMIT_OK`, `TEST_OK`, `TEST_MODE`, `WIN_OPEN`, `BRIDGE_5V` | 09 |
| Sequence | `SEQ_ACTIVE`, `SEQ_DONE`, `BR_FAULT`, the four gated enables `G_*`, cap timer node | 12 |
| Relay | `K_EN`, `K_REST_N`, coil supply | 12, 14 |
| Holds | `HOLD_A_HELD_N`, `HOLD_B_HELD_N`; each stack's bottom-gate bias through a 1:3 divider (local, reads ≈ −2.5 V when held) | 11 |
| Charger | `VTRANS_EN`, `CHARGE_EN`, VTRANS_GOOD (local), `CHG_DONE_N` | 10 |
| Faults | `OVP_TRIP`, `LOWV_OK`, `OC_DET_N`, `IDLE_I_DET_N`, `IDLE_FAST_N`, `LSD_OK` (v0.3) | 07, 08, 13 |
| Holds (v0.3) | `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N` | 11 |
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

**Simulation status (v0.3).** v0.2's eight [SIM] items were run (BR, CS, LR) and the answers are folded into the sections above: turn-off spikes and snubber (§B6.5), high-side timing (§B6.3), supply-collapse order (§B13.5), stack sharing and release (§B5.4), pump and detectors (§B5.4, §B8.1), SiC heating per shock (§B6.2). Not run: LT3751 child-mode regulation (item 6). Charger interference (item 8) was bounded by simulation and turned out to need blanking (D-67).

**Still open after v0.3, and what settles each:**

| # | Open item | What settles it |
|---|---|---|
| O-1 | SiC switch: interrupting ≈ 100 A at 2.4–2.7 kV; µs linear-region energy (no RBSOA, short-circuit, avalanche or sub-7 µs Zθ published) | Microchip data; later, low-voltage-scaled pulse measurement by a qualified lab |
| O-2 | Hold-pump level and the 1.35–1.65 kHz window at −20 °C with aged optos (models hand-built) | Measurement at −20 °C (low voltage only) |
| O-3 | IXTT2N170D2 cold breakdown factor (Fig 12 unread); GP02-40 leakage in the stack | Read DS100418B Fig 12; second round S-8 |
| O-4 | The five-device stack's 1,464 V Monte Carlo outlier | Second round S-9 (diagnostic only: five devices not adopted) |
| O-5 | MAX17320 minimum PCKP test current (only 30 µA typical published; release threshold 1 V or 1.5 V) | ADI statement; the 1 MΩ rule covers down to ≈ 10 µA |
| O-6 | Transformer losses (decide whether 7 s is met), leakage inductance, SMCJ45A clamp at 20 A | Maker data against the ≥ 90 % requirement |
| O-7 | Local film capacitor: 75 nF vs the 100 nF simulated; size and mass of the 940C parts | Second round S-2; CDE datasheet |
| O-8 | Charger-to-pad coupling | Measurement (blanking makes it non-critical) |
| O-9 | The carrier-loss detector and fast comparator in the full fault simulation; PNP start 1 µs (R3) vs 2.7 µs (BR) | Second round S-1 |
| O-10 | Current comparators (80 A, 5 A, 3 A) and the high-side kill latch cannot be exercised by the 0.3 A self-test | Design a test injection (WP-G F-6) in a later revision |
| O-11 | LT3751 VOUT-comparator overdrive for 3–5 µs pulses (only 2 µs published) | ADI |
| O-12 | Dump-resistor and stack temperatures under repeated dumps; release-failed 27 J in one device | Thermal data; second round S-11 |
| O-13 | The patient relay itself (no part exists) | Maker quotation against docs/requirements/patient-relay.md |
| O-14 | Battery DC resistance at −20 °C and low charge (relay-supply sag) | Cell data / measurement |
| O-15 | MGJ2 output regulation at 4.7 V input (sets the `LSD_OK` margin, ≈ 1.5 V) | Murata datasheet |
| O-16 | WP-E C3 (LT3751 UVLO2 band against `5V0_SYS`), C4, C6 (eFuse rated from −15 °C) — outside I-1…I-25, carried | WP-E follow-up |

**Second-round simulation list:** `hardware/aed-main/capture-notes/rev3-changes.md` (S-1 … S-15).

## §17. Questions for the owner

1. **Emergency switch-on:** sealed button (assumed), pad-cover pull, or holster switch?
2. **Q-47:** one data contact (assumed) or two?
3. **Second discharge path:** keep (recommended)?
4. **Voltage chains:** thin film on both sides? Accept recalibration at service?
5. **Display size:** 1.26 in (assumed; keeps READY visible while off) or larger (needs a 5 V always-on supply)?
6. **Internal resistance 5 Ω (D-23):** keep R_S = 3.9 Ω?
7. **Weight (Q-56):** v0.2 added ≈ 45–80 g and v0.3 ≈ 30–60 g more; the lower-risk prototype is now ≈ 910–970 g [EST], over the 900 g accepted (D-43). Accept, or look for savings?
8. **Bleeder:** v0.3: ≈ 17 min to below 50 V with everything connected (2.80 MΩ including the capacitor's own 10 MΩ); acceptable as the last resort?
9. **Silent failure on a battery-bus short (Q-52, open):** accept, or add a small independent cell for the supervisor's alarm?
10. **Q-50:** confirm D-59, D-60, D-61 (in particular: once current starts, cancel cannot stop the shock; worst case with a single fault now ≈ 171 J at the nominal charge with D-63).
11. **Charge time:** v0.3 simulation gives ≤ 5.3 s before transformer losses; ≤ 7 s holds if the transformer is ≥ 76 % efficient (requirement ≥ 90 %). No owner action unless the maker cannot meet it.
12. **Q-53/D-63, Q-54/D-62, D-64–D-67:** confirm the provisional decisions taken for v0.3.
13. **Q-57:** add an independent delivered-energy limiter (≈ 160 J worst case instead of ≈ 171 J, at the cost of parts and new failure modes)?
14. **Q-58:** accept that compression readings are discarded while charging (blind for ≈ 2–6 s during charge-during-analysis)?
15. **Q-55:** speaker loudness — no v0.3 change.

---

## §18. Decisions in DECISIONS.md that are strained

| Decision | Strain |
|---|---|
| D-29 with D-36/D-37 | Compression readings are discarded while charging (D-67): blind for ≈ 2–6 s during charge-during-analysis (Q-58) |
| D-44 / Q-39 | 9.2 A average / 21 A peak from a cold pack |
| **D-43** | Now ≈ 910–970 g lower-risk set [EST]: over the 900 g prototype limit (Q-56) |
| D-41 / Appendix A | Fault current ≈ 460 A, not 400 A; v0.3 adds an internal bleed and a surge rating the design now relies on (D-65) |
| D-28 | Main chain life drift |
| D-29 "≤ 7 s" | Met in simulation (≤ 5.3 s) **if** the made-to-order transformer is ≥ 76 % efficient (requirement ≥ 90 %) |
| D-59 / D-63 | A frozen controller after the start can still deliver ≈ 171 J (≈ 181 J at the window top) against 150 J; an energy limiter is Q-57 |
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
- **Anything about real behaviour**: nothing is measured. v0.3's simulations use hand-built models from datasheet figures (no maker model could be obtained or redistributed); their limits are in each study's notes.
- v0.3 parts confirmed by a research agent on 27 Nov 2026 from makers' PDFs: OPA838 (SBOS867E), MMBFJ270 (MMBFJ270/D Rev 2), GP02-40 (Vishay 88635), MMSZ4687 (MMSZ4678T1/D Rev 13), SN74LVC1G74 (SCES794G), TLV3501 (SBOS321E), CRHV1206 1 GΩ (Vishay 68002), MAX17320 Rev 5 (PCKP 30 µA typical only). From listings or snippets only [UNVERIFIED]: HVCR2512 100 MΩ and 4.7/22 MΩ codes, KEMET C2220Y332JGGACAUTO, CDE 940C30P15K-F dimensions. Not checked: IXTT2N170D2 Fig 12 and pin-out; MMBFJ270 production status; OPA838 orderable code.

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
| v0.3 simulations | `hardware/aed-main/sim/bridge/notes.md`, `sim/charger-storage/notes.md`, `sim/logic-relay/notes.md`, `sim/rev3/notes.md` | run 27 Nov 2026 |
| OPA838 | TI SBOS867E | read (agent) |
| MMBFJ270 | onsemi MMBFJ270/D Rev 2 (May 2022) | read (agent) |
| GP02-40 | Vishay doc 88635 Rev 09-Aug-2022 | read (agent) |
| MMSZ4687 | onsemi MMSZ4678T1/D Rev 13 (Aug 2021) | read (agent) |
| CRHV 1 GΩ | Vishay doc 68002 rev 18-Apr-17 | read (agent) |
| 940C film capacitors | Cornell Dubilier 940C catalogue (cde.com) | read (agent); size of 940C30P15K-F not read |
| IXTT2N170D2 | Littelfuse/IXYS DS100418B (02/12), mirror | read in CS |

---

## §F. Firmware requirements from hardware

Everything the firmware must do, or stop doing, for this circuit to work. Written for the parallel firmware change (D-59 points 1–5). "M" = main, "S" = supervisor.

**F-1 Consent signals.**
1. `PERMIT_CLK`, `TEST_PERMIT_CLK`: square waves **1–5 kHz (nominal 2 kHz), duty 30–70 %**; `HOLD_M_CLK`, `HOLD_S_CLK`: **1.5 kHz ± 10 %** (v0.3, item 4 below), produced by code that stops when the control loop stops (a GPIO toggled by the loop, or a timer in one-pulse mode re-armed by the loop); **no gap between edges longer than 1 ms**; **never** a free-running timer, PWM or DMA ring.
2. Watchdogs on both controllers, ≤ 50 ms.
3. A static level on any of these lines means "no consent".
4. **v0.3:** `HOLD_M_CLK` and `HOLD_S_CLK` run at **1.5 kHz ± 10 %** (not anywhere in 1–5 kHz: outside 1.35–1.65 kHz an aged sensor can chatter in the cold, R3 §2) and are **always stopped low**. Allow 25 ms for "not held" after stopping and 35 ms for "held" after starting from cold. The permit clocks keep the 1–5 kHz band.

**F-2 Power-up ordering.** `EN_5V0` at least 10 ms before any consent clock, `CHG_SESSION`, `RELAY_CMD`, `BTEST_CMD` or I²S activity. I²S lines low whenever `EN_5V0` is low.

**F-3 Charging.**
1. Raise `CHG_SESSION` ≥ 5 ms before `CHG_RUN` (eFuse soft start, VTRANS_GOOD delay).
2. Regulate by switching **`CHG_RUN` only**: off at the target reading, on again 3 V below. Do not toggle `CHG_SESSION` for regulation.
3. `CHG_FAULT_N` low → `CHG_RUN` low ≥ 1 ms, then high; count retries; give up and report after 3.
4. Lower `CHG_SESSION` when not charging.
5. **Firmware over-voltage limit 2,070 V** reading (v0.3; was 2,150 V recommended and 2,200 V in config.rs), both controllers.
6. Remember that a child charge cannot exceed ≈ 1,377 V in hardware, that the latching comparator acts at 2,073–2,142 V and that the primary trip is a last resort at 2,116–2,316 V.
7. **v0.3:** if the capacitor does not rise for ≥ 200 ms with `CHG_RUN` high, lower `CHG_SESSION` and `CHG_RUN` for ≥ 2 ms and restart; allow ≥ 15 ms after `CHG_SESSION` before expecting a rise (WP-E C7).
8. **v0.3 (D-67):** discard impedance (compression) samples while `CHG_RUN` is high and for 100 ms after; do not count these gaps towards `chest_channel_fail_ms`.

**F-4 The shock (D-59).** The ordering of §B13.1:
1. Checks (pads in range, at plan, countdown, not cancelled, temperature) are made **before** consent: M before `SHOCK_REQ`; S before starting `PERMIT_CLK`.
2. M (v0.3 timings, LR §1.1): `K_SUPPLY_EN` ≥ 200 ms earlier (at charge start) → `SHOCK_REQ` → wait `BRIDGE_ARMED_N` low (≤ 10 ms; **ignore lows shorter than 10 ms**) → wait **30 ms** → `RELAY_CMD` → wait until `K_REST_N` has read high continuously for **14 ms** (restart on any low sample; ≤ 40 ms total from `RELAY_CMD`, else abort) → `CMD_HA` → 100 µs → `CMD_LB` for phase 1 (from the plan) → `CMD_LB` low → 10 µs → `CMD_HA` low, `CMD_HB` high → `CMD_LA` at 0.5 ms after `CMD_LB` low, for phase 2 → `CMD_LA` low → 10 µs → `CMD_HB` low → `RELAY_CMD` low → withdraw `SHOCK_REQ`. All inside the 150 ms window.
3. **S, once the permit is running:** withdraw it only for cancel, heartbeat loss, a latched fault, over-temperature or over-voltage — **not** because its pad reading goes open or its voltage reading leaves the at-plan window. (Change to supervisor.rs.)
4. **M, in `Delivering`:** do not abort on pad or chest readings after `RELAY_CMD` (change to rescue.rs).
5. **Once `SEQ_ACT_*_N` is low**, both controllers treat consent checks as over: withdrawal has no effect, and the expected changes (pads open, voltage falling) are not faults. Until `SEQ_DONE_*_N`.
6. After `SEQ_DONE_*_N`: withdraw consent; consent must be absent **≥ 75 ms** before the next attempt (v0.3; hardware re-arm 51.2–60.3 ms plus detector drop-out ≤ 8.3 ms). Lower `K_SUPPLY_EN` on withdrawal or abort.
6a. **v0.3 (D-63):** phases never longer than **10.0 ms and 6.7 ms** (aed-waveform `phase1_max`, `phase2_max`; `max_total` 17.2 ms). The hardware caps (10.75–11.85 ms, 7.31–8.06 ms) end anything longer as a fault.
7. Status: **Delivered** = `SEQ_DONE` and `BR_FAULT_N` high and `I_SHOCK` shows both phases of the planned sign, size and length; **Refused** = `BRIDGE_ARMED_N` never low, or `SEQ_ACT` never low within the window; **Fault** = `BR_FAULT_N` low, or current wrong, or relay never transferred.
8. Model the same semantics in `interlock.rs`/`aed-sim` (`plant.rs` currently fires the whole shock instantly when `bridge_may_fire` is first true).

**F-5 Mode (D-60).** Compare `MODE_ADULT_N`/`MODE_CHILD_N` with `MODE_HW_RB_N` at switch-on and on every change; mismatch → record, "not ready"; in a rescue set the dose per D-60.

**F-6 Relay.** `K_REST_N` must read "at rest" at switch-on and before every analysis; otherwise fault (relay welded or coil driver shorted).

**F-7 Discharge paths and hold sensors.** In every self-test discharge release the holds one at a time, alternating which goes first: with only path A conducting τ ≈ 0.45 s and `HOLD_RB_B_N` high; with only path B the reverse; both together τ ≈ 0.225 s. Fit τ after every dump; τ ≥ 0.4 s on a both-paths dump = one path failed.

**F-8 Test permit (D-61)** — at switch-on before the first charge, and in the daily test:
1. S gives a test permit only if its own reading is < 50 V, M reports a test state, and for ≤ 150 ms.
2. **Blocking tests** (capacitor < 50 V, both holds on unless stated), each checking `BRIDGE_ARMED_N`: (a) test permit, no `SHOCK_REQ` → must stay high; (b) `SHOCK_REQ`, no permit → high; (c) `SHOCK_REQ` + test permit, M's hold released → high; (d) same, S's hold released → high; (e) `SHOCK_REQ` + test permit + both holds → low; (f) keep (e) for 200 ms → must go high (window expiry).
3. **Bridge test** (capacitor charged to 40–50 V, charger stopped): test permit + `SHOCK_REQ` → wait 30 ms after armed → `BTEST_CMD` → **7 ms** (v0.3; 2 ms was shorter than the test relay's operate time) → the sequence with short phases (e.g. 1 ms each): `I_SHOCK` must be zero during the `CMD_HA` lead; present in phase 1 and 2 with opposite signs; zero after each phase. `K_REST_N` must stay "at rest" throughout. Then the cap tests T-8 and T-9 of §B14.5 (v0.3); a second `CMD_LB` must be ignored.
4. **Test-relay weld check:** with the test relay commanded open, `CMD_HA` alone → `OUT_B_TAP` must stay near 0.
5. Record every result; any failure → "not ready"; a failure at switch-on → no shocks in this rescue (CPR guidance), per D-50.
6. **v0.3 (D-62):** the real-permit steps T-1 … T-7 of §B14.5, only while both controllers read the capacitor < 50 V; the supervisor grants the real permit for ≤ 150 ms per step and only on those conditions. After a real over-voltage trip, pulse `OVP_RST` only once both readings are < 50 V, record it, and stay "not ready" until a self-test passes.

**F-9 Cross-loading.** Calibrate each impedance channel with the other chip running its normal carrier; widen S's permit range by its calibration uncertainty.

**F-10 Logging and memory.** F-RAM written once per standby wake; I²C idle low with the main off — configure the MAX17320 so that does not shut its interface down [UNVERIFIED need].

**F-11 Proposed HAL additions** (for the firmware team to decide): `MainHardware::sequence_status()` (from `BRIDGE_ARMED_N`, `SEQ_ACT_M_N`, `SEQ_DONE_M_N`, `BR_FAULT_N`); `MainHardware::relay_at_rest()`; `MainHardware::mode_hw_ceiling()`; `MainHardware::hold_readback()`; `MainHardware::run_test_sequence(...)` or equivalent primitives; the charger split (`set_charger` implemented with `CHG_SESSION`/`CHG_RUN`); `SupervisorHardware::set_test_permit(bool)`; `SupervisorHardware::sequence_status()` (`SEQ_ACT_S_N`, `SEQ_DONE_S_N`); `SupervisorHardware::overvoltage_tripped()` (`OVP_TRIP`); v0.3: `MainHardware::set_coil_supply()` (`K_SUPPLY_EN`), `hold_readback()` over four sensors, `SupervisorHardware::reset_overvoltage()` (`OVP_RST`), `set_overvoltage_test()` (`OVP_TEST`).

**F-12 Simulator parameters (v0.3).** Dump 2,250 Ω (4,500 Ω one path; +26 Ω with the 2N170D2), total standing load ≈ 2.80 MΩ (3.90 MΩ on the board, 10 MΩ in the capacitor), relay transfer ≈ 16 ms + 14 ms wait, overall cap 25 ms (23.8–26.2), phase caps 11.37 / 7.73 ms (10.75–11.85 / 7.31–8.06), permit window 150 ms (131.8–161.5), re-arm 51.2–60.3 ms. The firmware constant table is in `hardware/aed-main/capture-notes/rev3-changes.md` (FW-1 … FW-20).

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

---

## §V3. Revision 0.3: every integration issue and simulation finding, and its resolution

**In plain language.** Drawing the circuit raised 25 issues; simulating it produced about forty findings, several of which overturned earlier hand calculations. This section lists each one and says what happened to it: **fixed** (with where), **fixed but needing a second simulation or a maker's figure**, or **open** (with what would settle it). Where a simulation and an earlier hand analysis disagree, the reasoning is checked and the choice explained, not simply the newer number taken. Change order for capture: `hardware/aed-main/capture-notes/rev3-changes.md`; nets: `hardware/aed-main/contract.md` issue 3.

### V3.1 Integration issues (capture-notes/00-integration.md)

| No. | Issue | Resolution | Status | Where |
|---|---|---|---|---|
| I-1 | Charger not connected to the cell mid-point | New net `VBAT_MID` 01 → 02; one balancer only | Fixed | §B1.1; WP-A A-1 |
| I-2 | One over-voltage limit cannot fit the window | Layers re-placed: firmware 2,070 V reading; comparator 2,107 V latching; primary trip a last resort (RBG 876 Ω) relying on a capacitor surge rating (D-65) | Fixed (sim-checked bands) | §B4.4 |
| I-3 | Four transistors not enough with one shorted; fix "five" | **Five does not help** (CS): 1,700 V devices + gate steering + per-device 10 MΩ / 3.3 nF: ≤ 1,174 V | Fixed; steering-diode leakage to re-simulate (S-8) | §B5.4; WP-F F-1…F-6 |
| I-4 | Shorted reference diode reads "held"; fix "two zeners" | **Two zeners do not fit** (CS): two complete sensors per stack on separate lines, read back separately | Fixed | §B5.4; WP-D D-3…D-5, WP-F F-10 |
| I-5 | Off-board capacitor has no drain of its own | 10 MΩ internal bleed required of the maker (D-65) | Fixed (requirement) | §B5.1; requirements/storage-capacitor.md |
| I-6 | Patient relay does not exist as a catalogue part | Made to order (D-64); requirements sheet; placeholder symbol | Open (maker) | §B7.2; requirements/patient-relay.md |
| I-7 | Faults hidden until the first real shock | Real-permit self-test steps T-1…T-7 (D-62) | Fixed (provisional decision) | §B14.5 |
| I-8 | Shorted low side overheats the high side; turn-off current above rating | Immediate high-side kill, fast 3 A idle path, carrier-loss detector, 80 A trip, local film capacitor | Fixed in simulation (11–13 mJ); re-run with real detector circuit (S-1, S-2); switch ratings unpublished (O-1) | §B6.3, §B6.5, §B6.6 |
| I-9 | Firmware waits equal hardware minimums | 14 / 75 / 7 / 30 ms waits; detector bleeds 64.9 kΩ; BTOK on a comparator; bounce filter | Fixed | §B8.1, §B13.3, §F |
| I-10 | Battery-wiring short leaves the device dark; loads block recovery | Rule: no load below 1 MΩ (D-66); alarm cell not added | Rule fixed; dark device **open (Q-52)**; ADI minimum current open (O-5) | §B1.1 |
| I-11 | Charge time 7.4–8.0 s at corners | Simulated ≤ 5.3 s before transformer losses; transformer requirement ≥ 90 % efficient | Fixed if the transformer meets it (O-6) | §B4.2, §B4.5 |
| I-12 | Speaker ≈ 79 dB against 85 dB; alarm probably too quiet | No change | **Open (Q-55)** | — |
| I-13 | 25 ms cap limits length, not energy (≈ 179 J) | Per-phase caps (D-63): ≤ 171 J at 1,950 V [SIM]; energy limiter not adopted | Fixed as far as caps can; **Q-57 open** | §B13.3, §B13.6 |
| I-14 | Compression sensing noise margin | Own sense resistors: 4.6–6.2 mΩ [SIM]; blank while charging (D-67) | Fixed; **Q-58 open** | §B3.4 |
| I-15 | Relay transfer 30–35 ms; supply 8 A | Pre-charged reservoir via `K_SUPPLY_EN`: 2.5–2.6 A, 15.9 ms [SIM] | Fixed; real relay data needed (O-13) | §B7.2 |
| I-16 | Main chain drift ≤ 0.6 % unpublished | Confirmed 0.59 % [SIM]; thin-film string recommended | **Open (owner question 4, §17)** | §B9.1 |
| I-17 | Hold/release marginal at −20 °C | Pilot MMBFJ270, 470 Ω pump resistor, 4.3 V zeners, 47 kΩ pull-ups, 1.5 kHz clock, stop low | Fixed in simulation (R3); measurement needed (O-2) | §B5.4 |
| I-18 | Supervisor's chip at 96 kHz, not 128 kHz | Adopted (WP-H reasoning checked: 128 kHz aliases into the ECG modulator) | Fixed | §B3.6 |
| I-19 | MAX17320 must not shut down with I²C low | `COMMSH = 0` | Fixed (configuration) | §B11 |
| I-20 | 1.8 V logic on the supervisor's chip | Level shifter on 16 accepted | Fixed (capture) | §B3.6 |
| I-21 | `I_SHOCK` headroom | Real problem was the saturated amplifier: R1355 13.3 kΩ (24.0 mV/A, ≤ 3.20 V) | Fixed [SIM] | §B6.6 |
| I-22 | OVP comparator does not latch | Latching comparator + `OVP_RST` (edge) + `OVP_TEST` | Fixed [SIM for the latch; test injection CALC] | §B4.4 |
| I-23 | Open tap wire reads "below 60 V" | 1 GΩ to `VREF_M` | Fixed [SIM] | §B9.1 |
| I-24 | Four-stage pump; release redesigned | Kept; simulation shows stage count makes little difference | Accepted | §B5.4 |
| I-25 | Load test defeated by a toggling line | Diode + 4.7 MΩ + 100 kΩ; R111 78.7 kΩ | Fixed; R111 value to simulate (S-6) | §B1.3 |

### V3.2 Simulation findings and where the evidence disagreed

| Finding | Earlier statement | Simulated | Adopted, and why |
|---|---|---|---|
| CS Q1: device-2 short | §B5.4: 727 V; WP-F: V/2 | 2,141 V (cold, 2,200 V) | **Simulation.** I traced the mechanism in the drawn topology (clamp current drags the tap); the hand analyses assumed no gate current. |
| CS Q1: five devices | I-3: fixes it | 2,116 V | **Simulation**, same mechanism. Five devices with the fix: one unexplained 1,464 V outlier; not adopted, outlier left as O-4. |
| CS Q1: two zeners | I-4: fixes it | intact threshold 9.5 V > pump | **Simulation**; arithmetic agrees (two 3.9 V zeners + LED + drops). |
| CS Q2: hold level | WP-F −7.4 V (behavioural); v0.2 −7.35 V | −5.5 … −6.1 V | **The lower figure is designed to** (safe direction for availability); measurement decides (O-2). |
| CS Q2: release ≤ 10 ms | v0.2 | up to 21 ms (stopped high > 30 ms) | **Simulation**; firmware stop-low rule and 25 ms allowance. R3 then found the power-up lock-out that a 2.0 V pilot still allows above ≈ 1.6 kHz and fixed it (470 Ω). |
| CS Q3: charge time | v0.2 7.2–7.8 s; WP-E 7.4–8.0 s | 5.29 s worst, no transformer loss | **Neither alone**: the hand figures lumped all losses at 60–65 %; the simulation leaves out the transformer. The transformer requirement now carries the efficiency. |
| CS Q4: OVP bands | v0.2 trip 2,033–2,189, OVP 2,138–2,182 | trip 1,988–2,178 (2,265), OVP 2,127–2,193 | **Simulation**: it includes secondary drops and life/temperature terms the v0.2 table left out (WP-E and WP-C reached the same by hand). |
| BR Q1: shorted low side | WP-G F-1: 25–122 mJ (high side off at once) | 0.35–12 J as drawn (high side on 20–36 µs) | **Simulation**: WP-G modelled a turn-off the logic did not have (WP-D CI-10). Fixed by changes 1–3. |
| BR Q1: PNP start | WP-G §6.2: 0.16–0.3 µs | 2.7 µs (behavioural) | R3's circuit-level model gives ≈ 1 µs. **Both pessimistic-model results were used to decide** (fix adopted); S-1 settles. |
| BR Q2: turn-off spike | v0.2: "[SIM]" | 2.7–2.9 kV; 3.2–3.7 kV with 0.5–1 µH cable | Simulation; local film capacitor + 80 A trip. |
| BR Q3: cap energy | v0.2: 177 J at 175 Ω | 180 J at 125 Ω; 189 J at 2,000 V | Simulation (the worst load is 100–125 Ω); per-phase caps. |
| BR Q4: collapse order | v0.2: "[SIM]"; WP-G doubt 2 | low side first in every case; MGJ2-only gap | Confirmed; `LSD_OK` added. |
| LR §1: detector drop-out | WP-D 7.32 ms | 8.28 ms | **Simulation** (coupling capacitors enlarge the hold node, not in WP-D's hand model); 64.9 kΩ. |
| LR §1.3: BTOK | WP-D 2.07–4.39 ms | 1.88–4.58 ms | Simulation (LVC input leakage); comparator-based timer. |
| LR §1.4: bounce, window race | WP-D: XFER guarantees 10 ms | 4.1–8.4 ms after a bounce | Simulation; filter + reset transistor; window-start delay. |
| LR §3: relay supply | WP-H ≈ 8 A, 30–35 ms; v0.2 2 A | 8.7–9.9 A, transfer up to 31.8 ms + 14 ms wait | Simulation confirms WP-H's estimate and shows the brown-out; pre-charge. |
| LR §4: battery release | WP-A: 33 kΩ, rule 100 kΩ | 45 kΩ at 30 µA; 100 kΩ fails at 15 µA | Rule tightened to 1 MΩ because ADI publishes no minimum. |
| LR §5: I_SHOCK headroom | WP-B: 0 V headroom | shock 0.55 V headroom; saturated amplifier 0.29 V over | Simulation; R1355. |
| LR §6: impedance noise | v0.2 1 mΩ; WP-H 5.6 mΩ | 25–34 mΩ as drawn | Simulation (shared filter capacitor); separate resistors; charger interference → blanking. |

### V3.3 Where v0.3 departs from a simulation recommendation

1. **Over-voltage comparator reference:** 21.5 kΩ (E192) gives 2,107 V, not 2,110 V; band shifted −3 V.
2. **Reset net named `OVP_RST`, not `OVP_RST_N`:** the simulated reset is an active-high edge; the contract's naming rule R1 reserves `_N` for active-low.
3. **Battery rule 1 MΩ, not 100 kΩ:** ADI publishes only a typical test current; 1 MΩ costs nothing (lowest drawn load 8 MΩ).
4. **Per-phase cap values:** 11.37 / 7.73 ms nominal with firmware maxima 10.0 / 6.7 ms, rather than option C's 10.5 / 7.05 ms, whose low ends met the firmware maxima with no margin; worst case 170.9 J (option C: 170.2 J). The 25 ms cap is kept, not shortened to 18 ms.
5. **Hold circuit:** CS recommended only a ≤ 2.0 V pilot and a stop-low rule; R3 showed that is not enough at 1.6–2 kHz and adds a 470 Ω pump resistor, 4.3 V zeners, 47 kΩ pull-ups and a fixed 1.5 kHz clock.
6. **Local film capacitor:** 2 × 0.15 µF 3 kV (75 nF) instead of 2 × 1 µF ≥ 2.2 kV: no 1 µF part above 2 kV could be verified; 75 nF must be re-simulated (S-2).
7. **Fast idle path on its own net** (`IDLE_FAST_N`) rather than OR-ed on sheet 13, so the INA240 path stays independent up to the logic; OPA838 + TLV3501 chosen (the simulation left the part open).
8. **Carrier-loss detector powered from the gate node**, not the rectified node (the latter stalled, R3 §3).
9. **`OVP_TEST` injection added** (beyond CS) to close the latent "reset switch shorted" fault.
10. **Not adopted:** the delivered-energy limiter (owner, Q-57); the RCD clamp (adds ≈ 20–90 V of margin only); R1001 4.7 mΩ (raises peak current towards the transformer and eFuse limits).

### V3.4 What is still open

Listed with what settles each in §16 (O-1 … O-16) and the owner questions in §17 (Q-52, Q-55 … Q-58, owner question 4). Second-round simulations: `capture-notes/rev3-changes.md` S-1 … S-15.
