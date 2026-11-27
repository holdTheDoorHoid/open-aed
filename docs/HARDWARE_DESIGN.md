# Hardware design specification

| Version | Date | What changed |
|---|---|---|
| 0.1 | 27 Nov 2026 | First design specification, written before schematic capture: topology and parts for every block, the arithmetic, the sheet plan (`hardware/aed-main/hierarchy.toml`) and the net contract (`hardware/aed-main/contract.md`). |

> **NOT FOR CONSTRUCTION — unreviewed design.** This document was written with AI assistance. **It has not been reviewed by a qualified high-voltage or medical-device engineer, and nothing in it has been built, simulated in circuit form or measured.** It may contain errors that would make a built circuit dangerous or useless. The storage capacitor in this design holds about 190 joules at about 2,000 volts, far above the level that stops a heart, and keeps it after the power is off. This is a design on paper for review and criticism (SAFETY.md §2a, D-13). It contains no assembly steps and no bring-up procedures, and it does not tell anyone to build or power anything (CLAUDE.md rule 2, D-10, D-18). Nothing here is safe or ready for use on a person or animal.

How to read the tags (as in the parts study):
- **[CALC]** arithmetic done for this document; the working is shown.
- **[EST]** an engineering estimate; its basis is stated.
- **[BK]** background knowledge that was not re-checked against a source this time. Treat it as something to check.
- **[UNVERIFIED]** looked for and not confirmed, or depends on a datasheet that could not be opened.
- **"read 27 Nov 2026"** means the datasheet text itself was read that day; **"page only"** means only a product page, distributor listing or search extract was seen.

Decision numbers (D-xx) and questions (Q-xx) refer to [DECISIONS.md](../DECISIONS.md). Where this document departs from the [parts study](PARTS_STUDY.md) (version 0.4), it says so and why.

---

## Plain-language summary

**What this is.** The parts study chose *kinds* of parts. This document turns them into a circuit design that several engineers can draw at the same time: what each block of the device does, which parts it uses, why, what the numbers are, and what is still unknown. The companion files are the list of drawing sheets (`hardware/aed-main/hierarchy.toml`) and the "contract" that says what every wire between sheets is called and who draws what (`hardware/aed-main/contract.md`).

**The big choices, and why.**

1. **Charging the capacitor.** A small transformer-based circuit (a "flyback charger", built around Analog Devices' LT3751 controller) lifts the two-cell battery's 6–8 volts to 1,950 volts in about 4–6½ seconds [CALC]. Its transformer has to be made to order. The charger is stopped by **three separate hardware limits** that do not need any software: a fixed ceiling inside the charger, a second ceiling measured a different way, and an independent over-voltage switch that empties the capacitor. A fourth hardware rule (D-25) **physically removes the charger's power** whenever the discharge path is not held off.

2. **Emptying the capacitor by default.** The capacitor is connected to an internal load resistor through electronic switches that **conduct unless they are actively held off** (depletion-mode transistors). There are **two such paths side by side**. The main controller alone keeps path A off; the supervisor alone keeps path B off. If either controller stops, loses power or lets go, its path conducts and the capacitor empties within about 2 seconds [CALC]. Each controller has to keep "pumping" a signal to hold its path off, so a controller that freezes with a pin stuck cannot keep the energy in. A permanent resistor bleeds the capacitor down too, as a last resort (about 40 minutes to below 50 V [CALC]). A relay was considered and rejected: relays rated for 2,000 V are not rated to switch onto a charged capacitor, and a relay's spring is a moving part.

3. **Delivering the shock.** Four 3,300-volt silicon-carbide transistors form the "bridge" that sends current through the patient one way and then the other. The hard problem the parts study found was **how to switch the two "top" transistors, which float up to 2,000 volts**. No catalogue driver is rated for it. The design sidesteps most of it: the two "bottom" transistors, which sit near ground, do all the precise timing, using ordinary catalogue drivers. The top transistors only need to be turned on a moment *before* current flows and off *after* it stops, so they can be slow. They are driven through **small made-to-order transformers** that carry both the signal and the power across the gap. Take away the transformer's buzz and the transistor turns off: no stored energy, nothing to stay stuck on.

4. **The third check: plain hardware.** The bridge's drive power comes from a supply rail that exists only while **three things are all true at once**: the main controller requests the shock, the supervisor permits it, and both discharge paths are held off. It is three electronic switches in a row. Remove any one condition and the bridge cannot be driven at all. The supervisor's permit must be a continuously changing signal from running code, so a stuck pin cannot give a permit.

5. **Protecting the heart-signal electronics.** At rest, a relay connects the pads to the heart-signal electronics; only when that relay is energised for a shock does it connect them to the bridge instead. The relay's coil is powered from the same interlock rail, so it cannot move unless a shock is allowed. Behind the relay there is a second layer of protection (defibrillator-rated resistors and high-voltage capacitors) that must survive an **external** 5,000-volt defibrillator test pulse.

6. **Two controllers that cannot fail together.** The main controller (ST STM32U585) and the supervisor (NXP MCX A153) are from different makers (D-39), each with **its own voltage regulator, its own crystal, its own capacitor-voltage measurement with its own reference, its own pad-resistance measurement, its own temperature sensor on the capacitor and its own view of the cancel button**. The supervisor has its own alarm sounder and light (D-26).

**What is still unknown — the most important.** Whether the 3,300-volt transistors survive millisecond pulses (no maker publishes it); whether made-to-order transformers can hold 2,200 volts for ten years in a box this small; whether any relay is rated to carry the shock current; whether the charger, which runs during every analysis (D-29), disturbs the compression sensing; how much spacing the standard demands at 2,000 volts (not yet bought); and the parts study's capacitor questions. Section 15 ranks all of them.

**Weight.** Two independent discharge paths, a protective coil in the shock path and a heavier patient relay add an estimated **25–45 g** over the parts study's lower-risk set [EST, §14]. The first prototype stays inside the accepted 780–900 g (D-43) on that estimate; the 700 g finished-design goal gets harder.

**One board or two.** The sheets are grouped so the layout engineer can choose. I expect **two boards**: a low-voltage board (power, controllers, display, sound, measurements, interlock) and a high-voltage board (charger, capacitor connection, discharge paths, bridge, patient relay, pad connector and the two front ends), joined by a connector whose every wire is listed in the contract (§6 there).

---

## Contents

- §0 Requirements baseline and where this design departs from the parts study
- §B1 Power: battery, dock, charger, rails, always-on domain, load test, protection
- §B2 Main controller and supervisor: pin budgets, clocks, debug, link, safe states
- §B3 Heart-signal and pad-impedance front ends
- §B4 High-voltage charger
- §B5 Energy storage, discharge by default, dump load, bleeder
- §B6 Output bridge and gate drive
- §B7 Patient isolation and pad interface
- §B8 Hardware interlock
- §B9 Sensing: capacitor voltage, temperature, current
- §B10 User interface
- §B11 Durable memory and timekeeping
- §B12 Creepage, clearance and partitioning
- §11 Rail tree · §12 Power and standby budget · §13 Single-fault table for the shock path · §14 Made-to-order and qualification parts, weight delta · §15 Test points · §16 Ranked open technical questions · §17 Questions for the owner · §18 Decisions that are strained · §19 What I could not verify · §20 Sources

---

## §0. Requirements baseline

The whole of DECISIONS.md is the baseline. The ones that shape the circuit most:

| Decision | What it requires of the circuit | Where |
|---|---|---|
| D-02, D-23 | Biphasic truncated exponential from one 100 µF capacitor at 1,950 V adult / 1,150 V child; phases 3–12 ms, 0.5 ms gap; about 5 Ω internal resistance assumed | §B5, §B6 |
| D-03, D-04 | 150 J adult, 50 J child (stored 190.1 / 66.1 J) | §B4 |
| D-05, D-57 | One pad set; the pad connector reports which pad set is fitted | §B7 |
| D-06, D-19 | Fully automatic: no shock button, so the three checks carry the whole load | §B8 |
| D-07, D-44 | Rechargeable, two cells | §B1 |
| D-09, D-45, D-46 | No radio; sealed dock contacts for charging and records; no USB socket | §B1.4 |
| D-15, D-49 | Weekly full-energy and daily low-voltage self-test into the internal load; battery tested under load | §B1.3, §B5 |
| D-25 | Charger cut off in hardware whenever the discharge path is active | §B4.6, §B8 |
| D-26 | Supervisor's own alarm, works with the main controller dead | §B10.2 |
| D-28 | Two voltage sensors, each 1 % + 7 V | §B9.1 |
| D-29 | Charge during analysis, ≤ 7 s; burn the charge off if no shock | §B4, §B5 |
| D-32, D-36 | Pads sensed to about 250 Ω; compression sensed through pad resistance (0.3 Ω threshold, 2 mΩ steps, < 10 mΩ RMS noise) | §B3 |
| D-39 | Two Cortex-M33-class controllers from different makers | §B2 |
| D-41, D-53 | Made-to-order capacitor, 100 µF, ≥ 2,200 V, charged only to 60 °C | §B5 |
| D-42 | Catalogue parts where they exist; made-to-order parts get a published specification | §14 |
| D-48, D-51, D-54–D-56 | Six temperature points, one read only by the supervisor; logging in every state; charge limits by temperature | §B9.2, §B11 |

The firmware's hardware contract is satisfied signal for signal; the method-by-method mapping is table 7 of the contract. The three pure functions in `firmware/crates/aed-core/src/interlock.rs` are shown equal to the hardware in §B8.3.

### Where this design departs from the parts study (version 0.4)

| Parts study | This design | Why |
|---|---|---|
| Charger power 44–50 W in, 7.6 A average / 15 A peak from the battery (§2) | 53 W in, **9.2 A average / 21 A peak** at a cold 5.8 V pack [CALC, §B4.2] | The LT3751's boundary-mode arithmetic, worked from its datasheet, needs 21 A peaks to finish in under 7 s with 65–75 % efficiency. Q-39 margin is thinner than the study assumed (§18). |
| One discharge path, three ceramic resistors (§4) | **Two independent paths**, each with its own switch stack and its own three resistors | No single open resistor or switch may leave the capacitor charged (§B5.3). About +21 g [EST]. |
| "Rely on two switches in series in each current path as patient isolation" (§3) | A **patient relay** between bridge and pads, which also connects the front ends at rest | Charging during analysis (D-29) means the bridge outputs sit beside a charged capacitor while the front end is measuring; without a relay the pads would ride on bridge leakage (§B7.1). The relay has no catalogue pulse rating (§16). |
| Front end: ADS1292R, MAX30009 as an upgrade (§6) | **ADS1292R for the main controller; MAX30009 for the supervisor's own pad-resistance reading** | The firmware's `SupervisorHardware::pad_resistance` is "the supervisor's own measurement" (hal.rs). A second, independent chip from a different maker gives it one (§B3.6). |
| Gate drive: an Appendix B2 part on all four switches | Made-to-order transformers on the **two high-side switches only**; catalogue isolated drivers on the low side | Low-side switches never see more than a few volts across their driver barrier (§B6.3). |
| Voltage-chain error budget 0.87 % worst case (§7) | 0.99 % (main, thick-film HVCR string) and 0.47 % (supervisor, thin film) worst case at manufacture plus life [CALC, §B9.1] | Re-done with the chosen parts; HVCR's life drift and temperature tracking are larger than the study's best-case figures. |

---

## §B1. Power

### B1.1 Pack, protection and fuel gauge (sheet `01_power_battery`)

**Topology.** Two 18650 cells in series (D-44; parts study option B, Molicel P28A class), a cell-midpoint tap for per-cell monitoring, one NTC thermistor on each cell, and the **ADI MAX17320** protector and fuel gauge driving external protection MOSFETs. After the protector, a main fuse (F101) feeds the system bus `VBAT_SYS`.

| Item | Part | Rating vs need | Source |
|---|---|---|---|
| Cells | Molicel INR-18650-P28A × 2 (made into a pack by a pack assembler, with welded tabs, both NTCs and a keyed connector) | 2 × 2.8 Ah, 20.2 Wh [CALC, parts study §5] | parts study §5 |
| Protector + gauge | ADI MAX17320 (2S–4S, ModelGauge m5, protector with external N-FETs, SHA-256 authentication, self-discharge detection) | Protects against over/under-voltage, over-current, short circuit and over/under-temperature per cell; −40 to +85 °C [BK] | analog.com/en/products/max17320.html (page only, 27 Nov 2026) |
| Protection MOSFETs | 2 × low-resistance 30 V N-channel (capture engineer chooses: ≤ 2 mΩ each, PowerPAK/SuperSO8, rated ≥ 40 A pulse) | Carry 9.2 A average, 21 A peak while charging the capacitor [CALC §B4.2] | — |
| Main fuse F101 | 25 A time-lag surface-mount fuse, ≥ 32 V | Above the 9.2 A average plus rail load with margin for the 21 A ripple peaks, which the charger's input capacitors carry (§B4.3) | part number left to capture [UNVERIFIED] |

**Why this way.** The gauge's per-cell readings let the self-test catch one weak cell (parts study §5). Two protection layers act on overheating while charging: the dock charger's own thermistor rule (B1.4) and the MAX17320's programmable charge-temperature limit, so firmware is a third, not the only, layer.

**Reverse and short protection.** The pack is internal and its connector is keyed, so reverse insertion is designed out mechanically; no series reverse-polarity MOSFET is fitted, because at 21 A peaks it would cost about 0.2–0.4 W of loss for a fault the connector already prevents [EST]. Short circuits are cut by the MAX17320's short-circuit protection (µs response [BK]) and, as a backup that needs no electronics, by F101. Each regulator has its own small fuse (§B1.6) so a shorted regulator does not take out the others.

### B1.2 Cell temperatures

Cell 1 and cell 2 NTCs go to the MAX17320's thermistor inputs; the main controller reads them over I²C (`TempPoint::Cell1/Cell2`, D-48).

### B1.3 Battery-under-load test path (`MainHardware::battery_under_load`)

A dedicated load: a pulse-rated **0.82 Ω** resistor switched by an N-MOSFET (`BAT_LOAD_EN` from the main controller). [CALC] At 8.4 V the load draws 10.2 A (86 W); at 7.2 V, 8.8 A — the same order as the charger's 9.2 A average, which is the realistic load (D-15). A 100 ms test puts 6.3–8.6 J into the resistor [CALC: 86 W × 0.1 s]. The gauge measures voltage and current during the pulse, giving each cell's internal resistance.

A **hardware time limit** is required on this sheet: the MOSFET gate is fed through an RC network so that the load turns off by itself after at most 250 ms even if `BAT_LOAD_EN` sticks high [requirement; values left to capture]. Resistor requirement: ≥ 25 J single pulse, one pulse an hour, 50 °C ambient.

**Alternative considered:** using the charger itself as the load. Rejected: the boundary-mode charger draws little power at low capacitor voltage (§B4.2), so a short test into an empty capacitor is not a realistic load, and it would add a full capacitor cycle to every daily check.

### B1.4 Dock contacts and battery charger (sheet `02_power_dock`)

**Contacts (D-46).** Flat gold contact pads on the device, springs in the dock: `DOCK_5V`, `DOCK_GND`, and one data contact `DOCK_DATA` (**parts study question P / Q-47 option (a): the dock is the reader**; §17). Requirements from the parts study §5: dead when undocked, survive reversed or excess dock voltage and static discharge, corrosion-resistant plating.

| Function | Part | Why |
|---|---|---|
| Input protection | TI **TPS25947** eFuse (2.7–23 V, adjustable over-voltage cut-off, integrated reverse-current blocking [BK]) after a 6.0 V TVS | Reverse and over-voltage from a faulty dock; reverse blocking keeps battery voltage off the contacts |
| Charger | TI **BQ25887** 2-cell boost charger from 5 V, 2 A, integrated cell balancing, battery thermistor input with JEITA, −40 to +85 °C (parts study §5) | A 2S pack needs a boost from the dock's 5 V |
| Charge temperature window | BQ25887 TS network set so charging stops below 0 °C and above 45 °C (D-48) in hardware | [UNVERIFIED: the BQ25887's TS thresholds must be checked to allow a 45 °C hard stop; if not, an external comparator gates CE] |
| Data contact | Open-drain N-MOSFET driven from the main controller's UART transmit (half-duplex), receiver through a series resistor and a Schmitt buffer with power-off protection (74LVC1G17) powered from `3V3_MAIN`; the **pull-up lives in the dock** | With no pull-up on the device side, the contact carries no voltage when undocked, so a wet glove across contacts shorts nothing |
| Dock detect | `DOCK_DET_N` from a small MOSFET switched by `VDOCK` | Wakes the device on the dock (§B1.7) |

**Why the dock is the reader (assumed).** One contact instead of two, and the device does not need a USB stack. The dock converts the single-wire UART to USB for a computer. If the owner chooses option (b) in Q-47, the contract reserves the second contact's net names (contract §5).

### B1.5 Charging limits under hardware and firmware control

Three layers stop the battery charging outside 0–45 °C: (1) BQ25887 TS thermistor (hardware); (2) MAX17320 charge-temperature protection (hardware, separate sensor channel); (3) firmware via `CHGR_CE_N`. **`CHGR_CE_N` defaults to "charge enabled"** (pull-down on the dock sheet), because a device left on the dock while switched off must still charge; the hardware layers alone enforce the temperature window then.

### B1.6 Rails and their independence (sheet `03_power_rails`)

The requirement: **the main controller and the supervisor must have independent regulators, so one failure cannot take out both.** Each is a separate buck regulator with its own input fuse from `VBAT_SYS`, its own inductor and its own enable logic.

| Rail | Part | Input | Load (peak / typical) | Notes |
|---|---|---|---|---|
| `3V3_MAIN` | TI **TPS629210** buck (3–17 V in, 1 A, 4 µA quiescent [BK]) | VBAT_SYS via its own 1 A fuse | ~80 mA / 25 mA [EST] | Main controller, flash, FRAM, ADS1292R (through a filter), ADS1220, main consent buffers |
| `3V3_SUP` | a **second, separate TPS629210** | VBAT_SYS via its own 1 A fuse | ~60 mA / 15 mA [EST] | Supervisor, AD7171, ADR3425, MAX30009 (through `1V8_SUP`), supervisor consent buffers, alarm |
| `5V0_SYS` | TI **TPS62130** buck (3–17 V in, 3 A [BK]) | VBAT_SYS via its own 3 A fuse | ~1.5 A / 50 mA [EST] | Audio amplifier, LT3751 supply, interlock logic, gate-drive power (through `BRIDGE_5V`), relay coils |
| `3V3_AON` | TI **TPS70933** LDO (up to 30 V in, about 1 µA quiescent [BK]) | VBAT_SYS | ~5 mA peak (display refresh) / ~10 µA | Always-on: wake timer, ready latch and indicator, memory display, power-on logic |
| `BRIDGE_5V` | output of the interlock (§B8) | 5V0_SYS | ~0.8 A for ≤ 200 ms | Exists only while a shock is allowed |
| `1V8_SUP` | small LDO on `15_frontend_sup` | 3V3_SUP | ~5 mA | MAX30009 core |

`5V0_SYS` is shared, but **a loss of `5V0_SYS` is a safe failure**: the charger and the bridge lose their supply, and the comparators that report "both discharge paths held" (`HOLD_OK`, §B5.4) are unpowered and therefore report "not held", so nothing can charge or fire. The controllers' own regulators, the hold pumps' drive buffers and the supervisor's alarm do not depend on `5V0_SYS` (§B10.2).

Why not the TI TPS62840 for the 3V3 rails (60 nA quiescent): its 6.5 V input limit is below the full 2S pack's 8.4 V [BK].

### B1.7 Power-on and wake logic (always-on domain)

| Enable | Turned on by (wired OR, hardware) |
|---|---|
| `3V3_MAIN` | power button pressed (`PWR_BTN_N`) **or** wake timer interrupt latch (`RTC_INT_N`) **or** dock present (`DOCK_DET_N`) **or** `MAIN_KEEP` from the main controller |
| `3V3_SUP` | power button pressed **or** `SUP_WAKE` from the main controller (scheduled self-tests) **or** `SUP_KEEP` from the supervisor |
| `5V0_SYS` | `EN_5V0` from the main controller (default off) |

The power button enables **both** controller regulators directly in hardware, so a failure of the wake logic or of either controller cannot stop the other one starting (D-26: the supervisor sounds its alarm if the main controller has not started within 1 s). Each controller then keeps itself on with its own KEEP line. **How the device is switched on in an emergency (a sealed button, pulling the pads, drawing it from the holster) is an owner question (§17).** The design assumes a sealed button separate from cancel.

### B1.8 Always-on domain contents

`3V3_AON` powers only: the RV-3028-C7 wake timer (§B11), a ready latch (SN74AUP1G74) and its LED flasher, the memory display (§B10.3) and the pull-ups of the wake and button lines. Everything on it is chosen for sub-microamp idle current. Signals from the always-on domain into a controller that may be unpowered go through series resistors of ≥ 100 kΩ or buffers with power-off protection (contract rule R7), so nothing back-powers an unpowered controller.

---

## §B2. Controllers

### B2.1 Choice

| Role | Part | Why | Departure |
|---|---|---|---|
| Main | **ST STM32U585VIT6** (Cortex-M33, 160 MHz, FPU, 2 MB flash, 786 KB SRAM, **LQFP100**) | Qualified Rust compiler target (D-39); 14-bit ADC; enough timers for the bridge sequence; Embassy HAL (parts study §7) | Parts study said "STM32U585-class"; the **100-pin** package is needed for the pin budget below (the 64-pin part has too few I/O) [UNVERIFIED: exact I/O count per package; DS13086 not opened] |
| Supervisor | **NXP MCXA153VLH** (Cortex-M33, 96 MHz, 128 KB flash, 32 KB SRAM, LQFP64, −40 to +125 °C) | Different maker, silicon and tools (D-39); NXP datasheet lists −40 to 125 °C and a 12-bit ADC with a 16-bit reduced-rate mode (page only, 27 Nov 2026) | — |

Both run their own **independent hardware watchdog** (STM32 IWDG on its own internal oscillator; MCX A WWDT) whose reset puts every pin into its reset state, which the default resistors of the contract turn into the safe state.

### B2.2 Main controller pin budget (against every HAL signal)

| Group | Nets (contract names) | Pins |
|---|---|---:|
| Heart-signal front end (SPI + control) | `AFE_SCLK, AFE_MOSI, AFE_MISO, AFE_CS_N, AFE_DRDY_N, AFE_START, AFE_RESET_N` | 7 |
| Capacitor-voltage ADC (SPI) | `HVM_SCLK, HVM_MOSI, HVM_MISO, HVM_CS_N, HVM_DRDY_N` | 5 |
| Event flash + FRAM (SPI, two chip selects, local) | local | 5 |
| Display | `LCD_SCLK, LCD_SI, LCD_SCS, LCD_DISP` | 4 |
| I²C (gauge, battery charger, wake timer) | `I2C_M_SCL, I2C_M_SDA` | 2 |
| Audio (I²S + shutdown) | `I2S_BCLK, I2S_LRCLK, I2S_SD, AMP_SD_N` | 4 |
| Link to supervisor | `LINK_M2S, LINK_S2M` | 2 |
| Dock | `DOCK_TX, DOCK_RX, DOCK_DET_N` | 3 |
| Battery | `GAUGE_ALRT_N, BAT_LOAD_EN, CHGR_INT_N, CHGR_CE_N` | 4 |
| Charger | `CHG_EN, CHG_DONE_N, CHG_FAULT_N` | 3 |
| Interlock | `HOLD_M_CLK, SHOCK_REQ, BRIDGE_ARMED_N, HOLD_OK_RB_N` | 4 |
| Bridge | `CMD_HA, CMD_LA, CMD_HB, CMD_LB, RELAY_CMD, BTEST_CMD, OC_TRIP_N, I_SHOCK` (analog) | 8 |
| User interface | `CANCEL_M_N, MODE_ADULT_N, MODE_CHILD_N, PWR_BTN_N` | 4 |
| Ready latch | `READY_D, READY_CLK` | 2 |
| Power | `MAIN_KEEP, SUP_WAKE, EN_5V0, RTC_INT_N` | 4 |
| Pads | `PAD_PRESENT_N, PAD_ID_1W, NTC_PADS` (analog) | 3 |
| Debug and clocks | SWDIO, SWCLK (+ NRST, BOOT0 dedicated); HSE 16 MHz crystal (2); LSE 32.768 kHz crystal (2) | 6 |
| **Total** | | **76** of about 82 I/O on LQFP100 [UNVERIFIED] |

The four bridge commands must come from **one advanced timer** (complementary-capable, e.g. TIM1/TIM8) so the phase times are set by hardware counting, not by code timing.

### B2.3 Supervisor pin budget

| Group | Nets | Pins |
|---|---|---:|
| Consent outputs | `HOLD_S_CLK, PERMIT_CLK` | 2 |
| Monitors | `SHOCK_REQ` (in), `OVP_TRIP` (in) | 2 |
| Cancel | `CANCEL_S_N` | 1 |
| Link | `LINK_M2S, LINK_S2M` | 2 |
| Pad-impedance front end (SPI + interrupt) | `SBZ_SCLK, SBZ_MOSI, SBZ_MISO, SBZ_CS_N, SBZ_INT_N` | 5 |
| Capacitor-voltage ADC (AD7171) | `HVS_SCLK, HVS_DOUT, HVS_PDRST_N` | 3 |
| Capacitor temperature (analog, own ADC) | `NTC_CAP_S` | 1 |
| Power | `SUP_KEEP` | 1 |
| Alarm (local) | piezo drive A/B, fault LED | 3 |
| Debug and clock | SWDIO, SWCLK; crystal (2) | 4 |
| **Total** | | **24** of about 50 I/O on LQFP64 [UNVERIFIED] |

### B2.4 Clocks

- Main: 16 MHz crystal (HSE) for the core and the bridge timer; 32.768 kHz crystal (LSE) for its own timekeeping while running. `MainHardware::now`.
- Supervisor: **its own crystal** (frequency per the MCX A153 datasheet's system oscillator range, capture engineer to set) so its countdown check ("armed for at least the countdown time, by the supervisor's own clock", config.rs) is independent of the main crystal. `SupervisorHardware::now`.
- Neither clock depends on the always-on wake timer; the wake timer only wakes and counts seconds (§B11).

### B2.5 Debug and programming

Each controller has its own SWD footprint (Tag-Connect style pads, no fitted connector), reachable only with the enclosure open. Nothing on the dock can program either controller; firmware loading in production is a fixture task, not a field function. **No debug or programming path connects the two controllers.**

### B2.6 The message link (link.rs)

A point-to-point UART pair, `LINK_M2S` and `LINK_S2M`, at 1 Mbit/s [EST: a 20-byte status every 50 ms is 0.4 % of that]. Each receiver has a **Schmitt buffer with power-off protection (74LVC1G17) powered from the receiver's own rail, a 1 kΩ series resistor, and a 100 kΩ pull-down** on the receiver's sheet. An unpowered or reset sender therefore reads as a continuous "break" (no messages), which the heartbeat timeout (250 ms, config.rs) treats as a stopped controller. A powered sender cannot back-power an unpowered receiver. The messages carry a sequence number (link.rs); a checksum is a firmware requirement.

### B2.7 What a stuck or dead controller does to each shared line

The rule for every safety-relevant line: **the default resistor on the line must produce the safe state when the driving controller is unpowered, in reset, or has its pin floating.** The table lists the lines that matter for the shock path; the contract (§5) gives every line.

| Line | Driver | Default resistor (sheet) | Unpowered / reset / floating gives | Safe because |
|---|---|---|---|---|
| `HOLD_M_CLK` | main | 100 kΩ pull-down (09) | no clock → path A conducts | capacitor empties; charger and bridge lose `HOLD_OK` |
| `HOLD_S_CLK` | supervisor | 100 kΩ pull-down (09) | no clock → path B conducts | same |
| `PERMIT_CLK` | supervisor | 100 kΩ pull-down (09) | no clock → no permit | bridge rail cannot exist |
| `SHOCK_REQ` | main | 100 kΩ pull-down (09) | no request | bridge rail cannot exist |
| `CHG_EN` | main | 100 kΩ pull-down (09) | charger off | — |
| `CMD_HA/LA/HB/LB` | main | 100 kΩ pull-downs (12) | all bridge switches off | and they have no drive power anyway |
| `RELAY_CMD` | main | 100 kΩ pull-down (13) | relay at rest: pads to front ends | — |
| `BTEST_CMD` | main | 100 kΩ pull-down (12) | test relay open | — |
| `EN_5V0` | main | 100 kΩ pull-down (03) | 5 V rail off | all of the above lose power: everything safe |
| `BAT_LOAD_EN` | main | 100 kΩ pull-down (01) + RC time limit | load off | — |
| A controller frozen with an output **stuck high** | — | — | static high on a CLK line = no clock = release / no permit; static `SHOCK_REQ` high alone does nothing without a permit | the dynamic encoding (§B8.2) |

---

## §B3. Heart-signal and pad-impedance front ends

### B3.1 Topology

```
 pads ── patient relay K1301 (at rest: pads → PT_FE_A/B) ──┬── main front end (sheet 14): ADS1292R
                                                           └── supervisor front end (sheet 15): MAX30009
```

Each front end has **its own protection taps** on `PT_FE_A` and `PT_FE_B`: separate resistors and capacitors, separate clamps, separate chip. The only shared elements are the pad leads, the pad connector and the relay's rest contacts.

Each pad gets four taps:

| Tap | Purpose | Series element (defibrillation protection) | Then |
|---|---|---|---|
| main sense | ECG (ADS1292R channel 2) and impedance demodulation input (channel 1) | **TT Electronics DPCR2512 51 kΩ** defibrillator pulse chip resistor | clamp diodes to the analog rails, RC filter, then split to channel 1 and channel 2 inputs |
| main drive | ADS1292R respiration modulation output (RESP_MODP / RESP_MODN) | **two 4.7 nF, 3 kV C0G capacitors in series** (2.35 nF, 6 kV) + DPCR 1 kΩ | clamp to the analog rails |
| supervisor sense | MAX30009 BIP/BIN | DPCR2512 51 kΩ | clamps, filter |
| supervisor drive | MAX30009 DRVP/DRVN | two 4.7 nF, 3 kV capacitors in series + DPCR 1 kΩ | clamps |

**Why capacitors in the drive path, not resistors.** This is the answer to the parts study's "the protection resistors are larger than the patient" problem. Sense taps carry almost no current, so a 51 kΩ resistor in them changes nothing. Drive taps must push tens to hundreds of microamps into a 25–250 Ω patient; behind tens of kilohms of resistor that is impossible from a 3 V chip. A small high-voltage capacitor has about 1 kΩ of impedance at the 64 kHz test frequency [CALC: 1/(2π × 64 kHz × 2.35 nF) = 1.06 kΩ] but blocks the millisecond-long defibrillator pulse almost entirely: an external 5 kV test pulse pushes at most Q = C × V = 2.35 nF × 5 kV = **12 µC** through it into the clamp [CALC]. It also blocks all direct current, which keeps patient direct leakage near zero.

### B3.2 Defibrillation protection: sizing

**Own shock.** During the device's own shock the relay has moved, so the front ends are disconnected from the pads (B7). The protection network is the second layer, for a relay that fails to move.

**External defibrillator (the harder case).** IEC 60601-1 8.5.5 defibrillation-proof applied parts: a 32 µF capacitor at 5 kV discharged through 500 µH into the applied part (research notes hv_safety_isolation §1; MEDTEQ description) [UNVERIFIED: clause and values from a secondary source; standard not bought]. At rest the relay connects the pads to the front ends, so the full pulse reaches the taps.

| Element | Stress [CALC] | Rating / basis |
|---|---|---|
| 51 kΩ sense resistor | peak current 5,000 V / 51 kΩ = **98 mA**; energy ≈ V²/R × t ≈ (4,724²/51 kΩ) × 2.4 ms ≈ **1.0 J** (pulse to half amplitude in 2.36 ms per MEDTEQ) | DPCR is made "to withstand medical defibrillator surges" (TT Electronics); the per-pulse energy rating was not found [UNVERIFIED]. MEDTEQ: ≥ 10 kΩ needed, 50 kΩ typical |
| Drive capacitors (2 in series) | 5 kV across 6 kV of rating; charge 12 µC | margin 20 % on voltage; C0G does not lose capacitance with voltage [BK] |
| 1 kΩ drive resistor | edge current i = C·dV/dt = 2.35 nF × 5,000 V / 9 µs ≈ **1.3 A** for about 10 µs [CALC; 9 µs rise per MEDTEQ] | DPCR 1 kΩ: pulse rating at 10 µs [UNVERIFIED] |
| Clamp after each series element | 98 mA for about 2 ms (sense), 1.3 A for about 10 µs (drive) | low-capacitance diode pair (e.g. BAV99 class) to the chip's analog rails plus a 5–6 V TVS to analog ground on each tap, so the rails themselves are protected [requirement; part to capture] |
| Energy-reduction test (8.5.5.2) | the front ends must not absorb a significant fraction of an external shock | sense: about 1 J × 4 taps; drive: about 0.1 mJ per tap [CALC]; against about 360 J (MEDTEQ). Passes on paper if the commonly quoted 10 % limit applies [UNVERIFIED] |

**No spark gaps or gas tubes across the pads.** A gas tube across `PT_FE_A`–`PT_FE_B` would fire during an external defibrillator shock and divert that shock's energy (energy-reduction test). Gas tubes to the enclosure are pointless in a battery device with no earth [EST]. The protection is therefore series-element-plus-clamp only, as the ADI and MEDTEQ material describes. The parts study's warning that protection parts degrade with repeated shocks applies: the 51 kΩ resistors are a service item to be checked by the self-test (their value shows up as a change in the ECG input's noise and in the impedance channel's calibration) [EST].

### B3.3 Input disconnect during the shock, and recovery

The patient relay (§B7) moves the pads from the front ends to the bridge about 5–10 ms before the bridge fires and back about 5–10 ms after (relay operate/release times; [UNVERIFIED] for the chosen relay). The front ends see no shock voltage while the relay is correct. After it returns, the pads carry a polarisation offset of up to a few hundred millivolts [BK]; the ADS1292R runs at gain 3 (±0.8 V input range) during recovery so it does not saturate, and firmware restores the high-pass filter state. The simulator assumes 1.5 s of flat ECG and 0.3 s of saturated impedance (aed-sim plant.rs); the real figure must be measured and compared with IEC 60601-2-4 201.105 [UNVERIFIED clause].

### B3.4 Main impedance measurement (compression sensing, D-36)

**Part:** TI **ADS1292R** (2 channels, 24 bits, internal respiration modulation and demodulation on channel 1, −40 to +85 °C, active; parts study §6). Channel 2 = ECG at 250 samples/s (aed-rhythm). Channel 1 = impedance; the board decimates to 125 samples/s (aed-cpr).

**Carrier: 64 kHz** (the ADS1292R's higher internal modulation option [UNVERIFIED: option set from datasheet]). Chosen above the charger's highest switching frequency, about 43 kHz [CALC §B4.2].

**Resolution and noise, against the aed-cpr requirements** (2 mΩ steps or finer; < 10 mΩ RMS from 0.5–10 Hz; range 25–250 Ω ± 25 Ω) [CALC; ADS1292R noise BK]:

| Quantity | Working | Result |
|---|---|---|
| Drive-path impedance | 2 × (1.06 kΩ capacitor + 1 kΩ resistor) + patient | about 4.1–4.4 kΩ |
| Excitation | about 3 V peak-to-peak square from RESP_MOD [UNVERIFIED drive level] ÷ 4.4 kΩ | about **0.7 mA p-p ≈ 0.34 mA RMS** |
| Patient auxiliary current limit at 64 kHz | IEC 60601-1, type BF: 100 µA × (f / 1 kHz), capped at 10 mA [BK, UNVERIFIED] | 6.4 mA: **19× margin** |
| Signal at 250 Ω | 0.34 mA × 250 Ω | 85 mV RMS, inside ±0.8 V at gain 3 |
| Change for 0.3 Ω compression | 0.34 mA × 0.3 Ω | **102 µV** |
| Input noise, 0.5–10 Hz | ADS1292R 8 µV p-p in 150 Hz [parts study] ≈ 1.3 µV RMS → × √(9.5/150) ≈ **0.33 µV RMS** | **≈ 1 mΩ RMS** equivalent: 10× better than required |
| Step size | 2 × 2.42 V / (3 × 2²⁴) = 96 nV → ÷ 0.34 mA | **≈ 0.3 mΩ per step**, 7× finer than required |

**Verdict on paper:** the capacitor-coupled drive turns the ADS1292R's marginal resolution (parts study risk) into an adequate one, provided the RESP output really drives about 3 V into a capacitive load [UNVERIFIED] and the **charger's interference** is small (B3.7). The excitation current depends on the drive capacitors, so the impedance reading is calibrated against known resistors per unit.

### B3.5 Pad presence

"Pads plugged in" comes from the connector's presence loop (§B7.3). "Pads on a patient" comes from the impedance: open circuit reads above the 300 Ω range limit and is reported as `ImpedanceSample::Saturated` or `Ohms` high (aed-types).

### B3.6 The supervisor's own pad-resistance measurement

`SupervisorHardware::pad_resistance` must be the supervisor's own reading (hal.rs). **Part: ADI MAX30009** bio-impedance front end (drive 16 Hz–500 kHz, dual 20-bit ADCs, SPI, 25-ball WLP; production status per ADI; parts study §6). Carrier set to **128 kHz**: an even multiple of the main's 64 kHz square-wave carrier, whose spectrum has only odd harmonics (64, 192, 320 kHz…), so the two synchronous demodulators reject each other [CALC from square-wave spectra; crosstalk UNVERIFIED]. The supervisor needs only a coarse reading (is the resistance between 25 and 175 Ω for the permit), so ±5 % is enough [EST].

**Why a second chip rather than giving the supervisor the main's reading:** a fault in the main front end or main firmware must not be able to make the supervisor believe the pads are on. Different maker (ADI vs TI), different measurement method, different carrier. **Open points:** the MAX30009's temperature range was not confirmed (a distributor field says 85 °C) [UNVERIFIED], and its WLP package needs a fine-pitch assembly process; ADI's AD5940 (LFCSP) is the fallback.

### B3.7 Isolation from the charger's switching noise (a known risk)

The charger runs during every analysis (D-29), exactly when the compression channel decides whether the chest is still (D-37). The LT3751 runs in **boundary mode, whose frequency sweeps** from a few kHz at low capacitor voltage to about 43 kHz near full charge [CALC §B4.2]. Its harmonics therefore sweep through every carrier at some point in each charge. Measures in this design:

1. Carriers above the charger fundamental (64 kHz, 128 kHz). Harmonics still cross; each crossing is brief (a few ms at the sweep rate) and lands outside the 0.5–6 Hz detection band after demodulation [EST].
2. Star return: the charger's primary current returns to the pack negative directly; the HV return joins `GND` at one net tie (`NT1101`) on the storage sheet; the front ends' analog grounds join `GND` at the front-end sheets only.
3. Input LC filter on `VTRANS` so the 21 A pulses do not modulate `VBAT_SYS`.
4. Physical: transformer and primary switch at the opposite end of the high-voltage board from the front ends; front-end supplies through ferrite and RC filters.
5. Firmware knows when charging (`CHG_EN`) and can flag impedance samples; **a "pause the charger for 20 ms every 100 ms" scheme is possible in firmware alone** (CHARGE pin) at the cost of about 20 % longer charging.
6. **If measurement shows these are not enough**, the alternative is a fixed-frequency flyback controller synchronised to the carrier (for example a TI LM5155-class controller with SYNC), which changes the charger design (§16, question T-4).

No datasheet can settle this; it is first on the list of things to measure at low voltage (parts study §10 rank 5).

---

## §B4. High-voltage charger (sheet `10_hv_charger`)

### B4.1 Topology and why

**Flyback capacitor charger, ADI LT3751, boundary mode, primary-side voltage sensing plus a secondary feedback divider** (parts study §2). Alternatives considered:

| Option | Verdict |
|---|---|
| LT3751 flyback with made-to-order 1:50 transformer | **Chosen.** Built for capacitor charging; programmable under/over-voltage lockouts on both supplies; current limit; primary-side output trip that works with no secondary components; −40 to +125 °C; read 27 Nov 2026 |
| LT3750 | Fixed target by turns ratio only; child mode would need separate regulation; no FB pin |
| General-purpose fixed-frequency flyback controller (LM5155 class) | More design work, no built-in charge termination; **kept as the fallback if charger noise disturbs the impedance channel** (B3.7) |
| Parallel-charge/series-discharge capacitor stack | Many switches; Avive patents; not before the patent review (parts study §2) |

### B4.2 Power stage sizing [CALC]

Inputs: VTRANS = 5.5–8.4 V (5.8 V cold under load, parts study §5); VOUT target 1,950 V (adult), worst case 2,008 V; C = 100 µF; charge time ≤ 7 s (D-29); efficiency 65–75 % [EST, LT3751: "most applications have overall efficiencies above 70 %"].

| Quantity | Formula (LT3751 datasheet, Applications Information) | Value |
|---|---|---|
| Turns ratio N (pri:sec) | N ≤ VOUT/VTRANS; chosen to keep the switch voltage low without huge reflected capacitance | **1:50** |
| Reflected voltage at full charge | (2,180 V + diode) / 50 | 43.6 V |
| Primary switch peak voltage | VTRANS + VOUT/N = 8.4 + 43.6 | **52 V** + leakage spike → 100 V switch, primary clamp at about 75–80 V |
| Peak primary current IPK | IPK = (2·N·VTRANS + VOUT)·C·VOUT / (η·VTRANS·t) solved for 7 s at 5.5 V, η = 0.65, with margin | **21.2 A** (sense resistor 106 mV / 5.0 mΩ) |
| Charge time, adult 1,950 V | t = (2·N·VTRANS + VOUT)·C·VOUT / (η·VTRANS·IPK) | 5.8 V, η 0.75: **5.35 s**; 5.5 V, η 0.65: **6.43 s**; 8.4 V, η 0.75: **4.07 s** |
| Charge time, child 1,150 V | same | 5.8 V, η 0.75: **2.16 s** |
| Primary inductance window | LPRI ≥ 3 µs·VOUT,TRIP / (IPK·N) = 3 µs × 2,170 / (21.2 × 50) = 6.1 µH; LPRI < 38 µs / (IPK·(1/VTRANS + N/VOUT)) at 5.5 V = 8.6 µH | **7.5 µH ± 10 %** (6.75–8.25 µH, inside the window) |
| Energy per switching cycle | ½·LPRI·IPK² | 1.69 mJ |
| Switching frequency at full charge | 1 / (t_on + t_off); t_on = LPRI·IPK/VTRANS; t_off = N·LPRI·IPK/VOUT | 5.8 V: t_on 27.4 µs, t_off 4.1 µs → **32 kHz**; 8.4 V: 18.9 + 4.1 µs → **43 kHz** (the highest) |
| Average primary (battery) current at full charge | IAVG,M = IPK·VOUT / (2·(VOUT + N·VTRANS)) | 5.8 V: **9.2 A** (53 W in) |
| Primary RMS current (switch, sense resistor) | triangular pulses at duty ≈ 0.87: IPK·√(D/3) | **11.4 A** |
| Output diode reverse voltage | VOUT,TRIP + N·VTRANS,max = 2,170 + 420 | **2,590 V** + ringing |
| Output diode average current | IPK / (2N) (highest at low VOUT) | 0.21 A |
| Secondary peak current | IPK / N | 0.42 A |
| Secondary inductance | N²·LPRI | 18.75 mH |

### B4.3 Parts

| Ref (block) | Part | Rating vs need | Source |
|---|---|---|---|
| U1001 | **ADI LT3751IUFD** (QFN-20) or LT3751IFE (TSSOP-20), I grade −40 to +125 °C | VCC 4.75–24 V; VTRANS ≥ 4.75 V; gate driver 2 A rail-to-rail (LVGATE, for VCC ≤ 8 V); current limit 106 mV/RSENSE | datasheet rev D read 27 Nov 2026 (mirror copy; ADI's own server did not respond) |
| Q1001 | **Infineon BSC070N10LS5** (100 V, logic-level OptiMOS 5, SuperSO8), or two in parallel | 100 V vs 52 V + spike: 48 % margin before the clamp; logic level because the LT3751 runs from `5V0_SYS` with LVGATE driving 5 V | Infineon product page (page only); RDS(on) at 4.5 V [UNVERIFIED] |
| R1001 | 5.0 mΩ, 2 W metal-strip current sense, Kelvin | 11.4 A RMS → 0.65 W | — |
| T1001 | **Made-to-order flyback transformer**, requirements §B4.5 | — | D-42 |
| D1001 | **Sanken UX-F5B** (8 kV, 350 mA, trr ≤ 150 ns) | 8 kV vs 2,590 V: 3.1×; 0.35 A vs 0.21 A | page only. trr is above the LT3751's "< 100 ns" advice: some efficiency loss [UNVERIFIED how much]. Alternative: SSDI SDA5000UF (5 kV, 0.5 A, 80 ns, hi-rel, page only) |
| VTRANS input | bulk: 4 × 47 µF X7R + 2 × 330 µF polymer [EST]; LC filter; fuse F1001 15 A time-lag | input ripple about 6.7 A RMS [CALC: √(11.4² − 9.2²)] | — |
| VTRANS switch | TI **LM74502** high-side driver + N-MOSFET (enable = `CHG_GATED`) | physically removes the transformer's supply when charging is not allowed (D-25) | ti.com/product/LM74502 (page only) |
| Primary clamp | RCD or TVS clamp at about 75 V | leakage spike energy ½·L_leak·IPK²: at 1 % leakage (75 nH), 17 µJ per cycle × 43 kHz = 0.7 W while charging [CALC] | capture |
| UVLO/OVLO | VTRANS: UVLO 4.9 V (RUVLO1 = (4.9 − 1.225)/50 µA = 73.5 kΩ), OVLO 9.0 V (155 kΩ); VCC: UVLO 4.6 V (67.5 kΩ), OVLO 5.6 V (87.5 kΩ) [CALC from VUVLO = 1.225 + 50 µA·R] | a pack sagging below 4.9 V stops the charger cleanly (FAULT) instead of letting the controller misbehave | datasheet |
| RVOUT / RBG / RDCM / RVTRANS | RVTRANS = RVOUT = 40.2 kΩ, RDCM = 18.2 kΩ (datasheet Table 2, VTRANS 4.75–60 V, ΔVDRAIN 5–80 V); RBG = 909 Ω | primary-side trip VOUT = 0.98·N·RVOUT/RBG − VDIODE = 0.98 × 50 × 40.2 k / 909 − 5 = **2,162 V** [CALC]; trip current 1.08 mA (datasheet: best accuracy 100 µA–2 mA) | datasheet |

### B4.4 How the target is set per mode, and stopped

Three layers, from finest to coarsest:

1. **Firmware closed loop (normal regulation).** `MainHardware::set_charger(Some(target))` is implemented by the board layer as on/off control of `CHG_EN` from the main's own voltage reading (ADS1220, §B9.1), with hysteresis. Overshoot per control step: the capacitor rises at P/(C·V) = 40 W / (100 µF × 1,950 V) ≈ **205 V/s** at full charge, so a 1 ms loop overshoots by about 0.2 V; one switching cycle adds 1.69 mJ, or ΔV = E/(C·V) ≈ **0.009 V** [CALC]. That meets config.rs's "a real charger must regulate to within about 5 V" with room to spare.
2. **Hardware mode ceiling (FB divider, no software).** A dedicated divider from `HV_CAP_P` to the LT3751's FB pin (regulates FB to 1.22 V): top string 40.2 MΩ; bottom 40.2 kΩ always, plus 57.6 kΩ switched in parallel by a MOSFET when the adult/child switch's second pole says ADULT (`MODE_ADULT_HW`). [CALC] Child: 1.22 × (1 + 40.2 M / 40.2 k) = **1,221 V**. Adult: bottom 23.7 kΩ → 1.22 × (1 + 40.2 M / 23.7 k) = **2,071 V**. So **a child-mode charge cannot exceed about 1,220 V whatever the firmware does**, and an adult charge stops near 2,070 V. The switch's default (open, broken or between positions) is the child ceiling: an adult charge then stops short and the firmware reports a charge time-out, which fails towards "no shock", not "overdose".
3. **Primary-side trip (RVOUT, no secondary components).** About 2,162 V ± 5 % [EST: depends on turns-ratio tolerance and leakage]. It works even if the FB divider fails open.

**Stopping.** `CHG_EN` low (firmware), or `CHG_GATED` low (hardware, §B4.6), drives the LT3751's CHARGE pin below 0.3 V (shutdown) **and** opens the VTRANS switch.

### B4.5 Transformer T1001: requirements for a made-to-order part (refines parts study Appendix B1)

| Item | Requirement | Basis |
|---|---|---|
| Type | Gapped flyback (coupled inductor) | boundary-mode flyback |
| Turns ratio | **1:50 ± 1 %** (primary : secondary) | §B4.2; the primary-side trip depends on it |
| Primary inductance | **7.5 µH ± 10 %** at 100 kHz, 0.1 V | §B4.2 window 6.1–8.6 µH |
| Saturation | LPRI falls by ≤ 10 % at **25 A** primary, at 70 °C | 1.2 × IPK |
| Primary RMS current | 11.4 A for 7 s, then off for ≥ 60 s; 14 charges at 132 s spacing; 20 charges in 1,300 s; 50 °C ambient | §B4.2, parts study Appendix B1 duty |
| Leakage inductance | ≤ 1 % of LPRI (≤ 75 nH) | primary clamp loss ≤ 1 W |
| Secondary self-capacitance | ≤ 15 pF | reflected N²·C = 37.5 nF; loss ½·C·V² = 30 µJ per cycle, 1.8 % [CALC] |
| Secondary voltage | +2,200 V to −450 V to the primary and core, every switching cycle; **working insulation ≥ 2,200 V DC** for 10 years | the secondary is referenced to `HV_RTN` (= GND) |
| Dielectric test | ≥ 4.4 kV DC, 60 s, primary + core to secondary [EST: 2 × working, parts study] | to be replaced by IEC 60601-1 values |
| Partial discharge | inception ≥ 3.3 kV peak (1.5 × working) after humidity soak [EST] | long-term insulation life |
| Temperature | −20 to +70 °C ambient; insulation class B (130 °C) or better | parts study B1 |
| Size target | ≤ 15 cm³, ≤ 30 g | parts study B1 |
| Evidence | measured L, N, leakage, self-capacitance; efficiency and temperature rise at the duty above; insulation and partial-discharge results before and after humidity soak; ten samples | D-42 |

### B4.6 Over-voltage protection that does not depend on software, and the D-25 gate

- **Over-voltage:** (1) FB ceiling 2,071 V / 1,221 V; (2) primary-side trip 2,162 V; (3) an **independent comparator on the supervisor's divider** (sheet 08) at **2,160 V** that forces both discharge stacks on and, through `HOLD_OK`, removes the charger's supply (§B5.5). Three mechanisms, three different sensing paths, none in software. Firmware on both controllers adds a fourth and fifth (2,200 V, config.rs).
- **D-25 gate:** `CHG_GATED = CHG_EN AND HOLD_OK` (sheet 09). `HOLD_OK` is true only when **both** discharge stacks are measured to be biased off (§B5.4). `CHG_GATED` drives both the LT3751 CHARGE pin and the VTRANS switch. This realises `interlock::charger_may_run(charger_enabled, discharge_active)` with `discharge_active = NOT HOLD_OK` (§B8.3).

---

## §B5. Energy storage, discharge by default, dump load and bleeder (sheet `11_hv_storage`)

### B5.1 Capacitor connection

The made-to-order capacitor C1101 (parts study Appendix A: 100 µF ± 5 %, ≥ 2,200 V, 60 °C working, 70 °C storage, ≥ 2,500 full charges) is off-board; its insulated flying leads land on two high-voltage terminals on the storage sheet's board area. Two NTC thermistors bonded to its case (one for the main, one for the supervisor; D-56) come back through a separate low-voltage 4-pin connector. **The capacitor's negative terminal is `HV_RTN`, joined to `GND` at exactly one point, the net tie NT1101 on this sheet.** That point is also where the charger secondary return and the bridge current shunt return meet.

### B5.2 Options for the discharge-by-default path

The path must conduct when the device is unpowered or when either controller releases its hold (hal.rs; interlock.rs `discharge_path_active`).

| | A. Normally-closed HV reed relay (e.g. Cynergy3 D-HR n/c) | **B. Depletion-mode MOSFET stack (chosen)** | C. Enhancement MOSFETs held *on* by a supply that dies with power | D. Bridge-based dump through two same-side legs (Physio-Control US5824017) |
|---|---|---|---|---|
| Conducts with no power | yes (spring) | **yes (physics of the device)** | no: needs energy to conduct when power is lost — fails the requirement | no |
| Stand-off | 5–15 kV (research notes §3) | 3 × 1,000 V = 3,000 V per stack (49 % margin at 2,008 V) | — | — |
| Making onto a charged capacitor | **Switching power rated 50 W; making 2,008 V at 0.45 A is about 900 W, 18× over** [CALC]; contact erosion and welding over ~5,000 dumps unknown | no contacts; the transition is a brief linear-mode pulse inside the device's safe operating area (B5.6) | — | very high peak current unless the protective element is large |
| Holding open | coil current for the whole hold (about 0.3–0.5 W for up to 25 s) [EST] | a few microamps of gate bias | — | — |
| Failure modes | weld closed (can't charge: detected); fail to close (dangerous, latent); contact bounce on a 1.5 m drop | **short (can't charge: detected at switch-on)**; open (rare, after over-stress: detected on every dump by the decay rate, B5.3) | — | shares failure modes with the shock path |
| Size | about 10 cm³ per relay [EST] | 3 × TO-268 per stack + resistors, about 4 cm³ per stack [EST] | — | none extra |

**Chosen: B, two stacks in parallel, each with its own resistors.**

### B5.3 The two-path arrangement

```
HV_CAP_P ──┬── R1101-R1103 (3 × 1.5 kΩ) ── stack A (3 × IXTT10N100D2) ── HV_RTN     held off by the MAIN only
           ├── R1104-R1106 (3 × 1.5 kΩ) ── stack B (3 × IXTT10N100D2) ── HV_RTN     held off by the SUPERVISOR only
           ├── bleeder 8 × 2.49 MΩ (19.9 MΩ) ─────────────────────────── HV_RTN     always
           ├── main sense string 40 MΩ ── VCAP_M_TAP ── bottom leg ────── HV_RTN     always
           └── supervisor sense string 40 MΩ ── VCAP_S_TAP ── bottom leg ─ HV_RTN     always
```

- **Path A is held off only while the main controller pumps `HOLD_M_CLK`; path B only while the supervisor pumps `HOLD_S_CLK`.** Either stopping turns its path on. That *is* `discharge_path_active(main_hold, supervisor_hold) = NOT(main_hold AND supervisor_hold)`, done in silicon rather than logic (§B8.3).
- **No single open part leaves the capacitor charged:** each path has its own switches and resistors.
- **A failed-open path is found on every dump.** Both paths: R = 2.25 kΩ, τ = 0.225 s. One path: R = 4.5 kΩ, τ = 0.45 s [CALC]. The firmware fits τ to its voltage readings after each release; a τ near 0.45 s means one path has failed. (Firmware requirement; nothing in the existing firmware does this yet.)

**Dump timing against config.rs** (`disarm_time_limit_ms` = 3 s; empty = 50 V less the sensor's own error, call it 38 V) [CALC]:
- both paths: t = τ·ln(2,008/38) = 0.225 × 3.97 = **0.89 s**
- one path: **1.79 s**, still inside 3 s.

The simulator's model uses 2,000 Ω and a 50 MΩ bleeder (aed-sim plant.rs). The equivalent figures here are 2,250 Ω (both paths) and about 6 MΩ total bleed (B5.7); the firmware team may wish to update the model (§17).

### B5.4 Hold bias: how each controller keeps its path off

- Each stack: three Littelfuse/IXYS **IXTT10N100D2** depletion-mode MOSFETs in series (1,000 V, on-current ≥ 10 A, RDS(on) 1.5 Ω, VGS(off) −2.5 to −4.5 V, IDSS 10 µA at 25 °C / 250 µA at 125 °C; datasheet copies, page only; one 2017 revision marked "preliminary" [UNVERIFIED current revision]). TO-268 surface mount.
- **Bottom device:** gate held at about **−8 to −9 V** by a diode-capacitor inverting charge pump driven by the controller's square wave (`HOLD_A_DRV`, `HOLD_B_DRV`, buffered on the interlock sheet by a buffer powered from **that controller's own rail**). Gate-to-source 100 kΩ, so the gate returns to 0 V (conducting) within about 1–3 ms of the clock stopping [EST]. Gate-source clamp zener 12 V (VGS max ±20 V [BK]).
- **Upper two devices:** self-biased from a 3 × 10 MΩ balancing string across the stack (the standard series depletion-device arrangement): when the bottom device blocks, each upper gate sits below its source and the stack shares the voltage about equally; when the bottom device conducts, the upper gates are pulled positive (clamped) and they conduct too. Speed-up capacitors across the balancing resistors keep the upper devices following the bottom one within microseconds [requirement; values from simulation].
- **Pump amplitude:** a doubler-inverter from 3.3 V gives about −5.4 V, not enough against VGS(off) of −4.5 V. The capture engineer uses a **three-stage inverting Dickson pump** from the 3.3 V clock (about −8.1 V unloaded [CALC: −(3 × 3.3 − 6 × 0.3)]) or, for path A only, drives the pump from a 5 V buffer on `5V0_SYS` gated by the 3.3 V clock. Requirement: **bottom gate ≤ −7.0 V at 25 µA load**, everywhere in −20 to +60 °C.
- **Clock:** ≥ 1 kHz square wave, produced by code that stops when the control loop stops (for example a timer in one-shot/repetition mode that the loop must re-arm at least every 2 ms), **never a free-running timer**. Firmware requirement for the board-support layer.
- **`HOLD_OK`:** one comparator per stack watches its bottom gate through a level-shift network and releases an open-drain output only when the gate is below −6.0 V (more negative than the worst-case −4.5 V cut-off, with margin). The two open-drain outputs are wired together onto `HOLD_OK` (pull-up on the interlock sheet): **high only when both stacks are measured off**.

### B5.5 Over-voltage force

`OVP_TRIP` (from the supervisor's comparator, sheet 08) drives a small N-MOSFET on each stack that shorts that stack's bottom gate to its source: both paths conduct at once, `HOLD_OK` falls, the charger loses its supply. Default pull-down on the storage sheet (no trip). This duplicates the supervisor's firmware over-voltage rule in hardware.

### B5.6 Stresses [CALC]

| Item | Normal (both paths) | One path | Rating |
|---|---|---|---|
| Peak dump current per path | 2,008 V / 4.5 kΩ = **0.45 A** | 0.45 A (the other path absent) | IXTT10N100D2 on-current ≥ 10 A: 22× |
| Voltage per device, held | 2,008 / 3 = **670 V** | — | 1,000 V: 49 % margin; a shorted device leaves 1,004 V on each remaining one, at the limit — it then avalanches and conducts: a fail-towards-dumping |
| Release transition (per bottom device) | about 670 V × 0.45 A / 2 for ≤ 3 ms ≈ **0.45 J** worst case [EST] | same | **Must be checked against the forward-bias safe-operating-area curve** [UNVERIFIED]; a faster release (smaller gate resistor) cuts it |
| Energy per resistor, full 190 J dump | 190 J / 6 = **32 J** | 190 J / 3 = **63 J** | parts study: Ohmite EY 400 J single impulse [UNVERIFIED repetition rating] |
| Peak power per resistor | (0.45 A)² × 1.5 kΩ = **300 W** | 300 W | — |
| Voltage per resistor | 670 V peak | 670 V | EY voltage rating not found [UNVERIFIED]; fallback Ohmite 885SP: 1,000 V peak, 250 J each (parts study §4) |
| Temperature rise per dump | 32 J / (7 g × 0.8 J/g·K) ≈ **5.7 K** | ≈ 11 K | ceramic; parts study §4 |

**Resistors:** Ohmite **EY-series** ceramic composition, 1.5 kΩ, three per path (parts study option A), with **885SP** as the fallback. Requirement for either: ≥ 5,000 pulses of 63 J over life, including runs of 14 at 132 s spacing, 50 °C ambient (parts study §4).

### B5.7 Permanent bleeder

8 × 2.49 MΩ high-voltage chip resistors (19.9 MΩ; 250 V each at 2,008 V), on its own string, **not shared with any divider or switch**. Total resistance from `HV_CAP_P` to `HV_RTN` with everything always connected [CALC]: bleeder 19.9 MΩ ∥ two 40 MΩ divider strings ∥ two 30 MΩ balancing strings (through the conducting-or-not stacks, which are in series with their dump resistors) ≈ **6.0 MΩ**: τ = 600 s; from 2,008 V to 50 V in τ·ln(40) ≈ **37 minutes**. With only the bleeder left: 2.0 hours. Loss while charged: 2,008²/6.0 MΩ = **0.67 W**, about 1.9 % of the charger's output; 17 J over a 25 s hold [CALC]. The research notes (§7) found no passive bleeder that meets a 5-minute rule without wasting energy; the active paths do that job, the bleeder is the last resort and the dielectric-absorption sink.

### B5.8 Self-test discharge and post-shock residual: how they are measured

Both controllers read the capacitor continuously on their own chains (§B9.1). After every release (self-test, cancelled shock, post-shock residual up to about 846 V / 35.8 J [DESIGN_TABLE]), each records the decay; the main controller fits τ (both paths / one path / none) and both check "empty within 3 s" (config.rs, D-33). The full-energy weekly test dumps 190 J the same way (D-15). Nothing reaches the pads: the patient relay is at rest (pads to front ends) throughout.

---

## §B6. Output bridge (sheet `12_hv_bridge`)

### B6.1 Arrangement

```
HV_CAP_P ── R_S 3.9 Ω ── L_S 30 µH ── HV_BUS ─┬─ QHA ── HV_OUT_A ──┬─ QLA ─┐
                                               └─ QHB ── HV_OUT_B ──┴─ QLB ─┴── LS_SRC ── shunt 2 mΩ ── HV_RTN
Phase 1: QHA on (beforehand), QLB times the phase.   Phase 2: QHB on (beforehand), QLA times the phase.
```

### B6.2 Switch technology

| | **A. 4 × 3.3 kV SiC MOSFET (chosen)** | B. SCRs + IGBTs (expired Physio-Control approach) |
|---|---|---|
| Part | **Microchip MSC080SMA330B4N** (3,300 V, 80 mΩ, IDM 107 A, TO-247-4 with Kelvin source); alternative GeneSiC/Navitas **G2R120MT33J** (3,300 V, 120 mΩ, TO-263-7 surface mount) | 3 × Littelfuse CS20-22moF1 (2,200 V) + 1–2 × IXEL40N400 |
| Voltage margin at 2,008 V | **64 %** | SCR **10 %** (thin, parts study §3) |
| Turn-off | any switch, at will | only the IGBT leg; SCRs need current to stop |
| Gate drive | 4 channels, but only the 2 low-side ones need fast timing (B6.3) | pulse-transformer firing for SCRs, isolated drive for the IGBTs |
| Pulse rating at 3–15 ms | **not published** [UNVERIFIED]: IDM 107 A is "limited by maximum junction temperature" with no pulse width | SCR surge rating is a 10 ms half-sine figure, made for this [BK] |
| Conduction loss per adult shock | 2 devices × (65 A)² × 0.15 Ω (hot) ≈ 1.3 kW peak; about 1 J per device per shock [research notes §2 CALC] | lower for SCRs at high current |

Chosen A for its voltage margin and full controllability (a stuck-on SCR cannot be turned off by its gate), and because the high-side drive problem is reduced below. The millisecond pulse rating is open question 1 (§16).

### B6.3 Gate drive and its isolation

**The key observation: only the low-side switches need precise timing.** Phase 1 runs through QHA and QLB. QHA is turned on about 100 µs *before* QLB; at that moment no current can flow (QLB is off), so QHA switches with no load. QLB then starts the phase and **ends it**, interrupting the current. QHA turns off afterwards, again with no current. Phase 2 mirrors this with QHB and QLA, and the 0.5 ms interphase gap is long enough for one high-side switch to turn off and the other to turn on. So:

| Switches | Reference | Timing need | Drive chosen |
|---|---|---|---|
| QLA, QLB (low side) | `LS_SRC`, a few volts above `HV_RTN` (= GND) at most (0.13 V at 65 A, 0.9 V at a 450 A fault across the shunt) | phase start/end to about 1 µs | **TI UCC5350MC** isolated single-channel driver with active Miller clamp (catalogue), output side powered **+20 / −5 V** from a **Murata MGJ2D052005SC** module (5 V in, +20/−5 V out, designed for SiC gate drive [BK]), both fed from `BRIDGE_5V`. The isolation barrier sees only the shunt voltage — the isolated parts are used for their split rails and Miller clamp, not for their voltage rating |
| QHA, QHB (high side) | their own sources, which float from 0 to 2,008 V with the outputs | on/off within about 50 µs, held on up to 20 ms | **Transformer-coupled carrier drive**: a TI **SN6505B** push-pull transformer driver (420 kHz [BK]) on the low-voltage side, enabled by the high-side command, drives a **made-to-order gate transformer** (T1201, T1202); the secondary is rectified to about +18 V and fed to the gate. Carrier on = gate on; carrier off = gate discharged by an active PNP turn-off within about 20 µs [EST] |

**Why transformer-coupled for the high side (alternatives considered):**

| Option | Verdict |
|---|---|
| Catalogue reinforced isolated driver (UCC21750, VIOWM 2,121 V DC) + isolated gate supply | Driver at 94 % of rating; and no catalogue gate supply is rated above 800 V rms reinforced (MGJ2T2) or 3 kV functional (MGJ6) (research notes §2). Rejected for now — but note that in this topology the high-side barrier sees ~2 kV only **during a shock (≤ 20 ms)**, because the outputs are pulled to `HV_RTN` at all other times (B6.6). If the standard counts that as a transient rather than a working voltage, catalogue drivers might qualify. Settle when the standard is bought (§16). |
| Bootstrap supply from the low side | Needs a 3 kV+ bootstrap diode *and* a separate isolated signal path: two barriers instead of one; the bootstrap capacitor can hold the switch on after the command goes away. Rejected. |
| Fibre-optic signal (Broadcom Versatile Link) + separate gate supply | Signal isolation unlimited, but power still needs a ≥ 2.2 kV barrier (none in catalogue). Kept as the fallback for the signal half. |
| Photovoltaic (opto) MOSFET drivers | Turn-on of tens of µs is fine, but isolation ratings of small photovoltaic drivers are far below 2 kV working [BK]. Rejected. |
| **Transformer carries signal and power together (Physio-Control prior art, expired)** | **Chosen.** One barrier per switch, defined by a part whose insulation can be specified to the job (D-42). No stored energy: losing the carrier, the driver or `BRIDGE_5V` turns the switch off. |

**dv/dt immunity — the shoot-through risk.** When a low-side switch turns on (phase start), its output node falls by up to 2 kV in about 2 µs (≈ 1 V/ns); the *off* high-side switch above it sees that swing across its Miller capacitance, which tries to lift its gate. Defences:
- **High side:** an added **100 nF gate-source capacitor**, so the Miller charge produces only a small gate voltage: with gate-drain charge of order 40 nC for a 3.3 kV device [UNVERIFIED], ΔVGS ≈ 40 nC / 105 nF ≈ **0.4 V**, against a threshold no lower than about 1.5 V hot [BK]. The high side can afford the big capacitor because it is deliberately slow: charging 105 nF through 100 Ω from the rectified carrier takes about 50 µs.
- **Low side:** −5 V off-bias and the UCC5350MC's active Miller clamp.
- **The high side is switched slowly on purpose** (no current flows then), which keeps dv/dt at the low side's drain low when the high side turns on.

### B6.4 Shoot-through avoidance in hardware (bridge logic powered from `BRIDGE_5V`)

All four commands come from the main controller's one advanced timer (`CMD_HA, CMD_LA, CMD_HB, CMD_LB`). On the bridge sheet, before the drivers, discrete logic (74LVC1G family with power-off protection, on a local `BRIDGE_3V3` regulator from `BRIDGE_5V`) enforces:

1. **Leg lockout:** `CMD_LA` is passed only if `CMD_HA` has been low for ≥ 150 µs (RC delay + Schmitt gate); `CMD_LB` likewise against `CMD_HB`. 150 µs exceeds the high side's worst turn-off (≈ 20 µs) with margin and fits the 0.5 ms gap. **This allows phase 1 = HA + LB and phase 2 = HB + LA, and makes HA + LA or HB + LB impossible.**
2. **Maximum window:** a non-retriggerable one-shot (74LVC1G123) started by the first low-side enable passes low-side commands for at most **25 ms**, so a stuck timer cannot hold a phase on (the longest planned shock is 15.5 ms; DESIGN_TABLE).
3. **Over-current trip:** a fast comparator (TI TLV3201) on the shunt at **90 A** sets a latch (74LVC1G74) that blocks all four commands until `BRIDGE_5V` is removed; its output `OC_TRIP_N` goes to the main controller. The latch's power-on reset clears it.

### B6.5 Series protective element and the 5 Ω internal resistance

[CALC] Fault current must be limited (parts study Appendix A1: the capacitor survives about 400 A into a short). Internal resistance other than R_S: capacitor ESR 0.13 Ω; two SiC switches hot 2 × 0.15 Ω; shunt 0.002 Ω; relay contacts 2 × 0.05 Ω [UNVERIFIED]; L_S winding 0.05 Ω; leads, connector and pad leads about 0.2 Ω [EST]; total about **0.8 Ω**. **R_S = 3.9 Ω** brings the total to about **4.7 Ω**, close to the 5 Ω the waveform model assumes (D-23), and limits a hard short to 2,008 / 4.7 ≈ **430 A**. L_S = **30 µH** limits the rise to 2,008 V / 30 µH ≈ 67 A/µs, so the 90 A trip (≈ 1.3 µs) plus about 0.5 µs of comparator, latch and driver delay leaves a peak of about **120 A** for about 2 µs [CALC] — briefly above the SiC part's 107 A IDM, so the short-circuit withstand of the chosen switch must be confirmed [UNVERIFIED].

- R_S energy per adult shock at 25 Ω: the model's 30.6 J internal loss × 3.9/4.7 ≈ **25 J**; peak power 65² × 3.9 = **16.5 kW**; 20 shocks in 1,300 s average 0.4 W. Part: ceramic composition (Ohmite EY class, 400 J) or two in parallel.
- L_S: made-to-order air-core (or non-saturating powder core) coil, 30 µH ± 20 %, ≥ 450 A without losing more than 20 % of its inductance, ≤ 50 mΩ, insulated for 2,200 V to its surroundings. Air-core estimate: about 55 turns on a 20 mm diameter × 30 mm former [CALC, Wheeler's formula], about 10 cm³ [EST].
- With L_S in circuit the shock's current rise time is L/R ≈ 30 µH / 30 Ω = 1 µs, irrelevant to a millisecond waveform. Energy stored in L_S at 65 A (0.06 J) returns to the capacitor through the high-side body diodes when a phase ends.

**Departure flag:** keeping the total near 5 Ω preserves the waveform model's assumption (D-23). Lowering R_S would deliver more of the stored energy (less heat) but raise the fault current and invalidate the model's figures; that is an owner/engineer trade (§17).

### B6.6 Output nodes, sensing and bridge self-test

- **Output pull-downs:** each output is pulled to `HV_RTN` by a 1 MΩ string (4 × 249 kΩ + a 1 kΩ bottom leg), which is also its **sense divider** (`OUT_A_TAP`, `OUT_B_TAP`, 1:1,000, to the main's ADS1220). With the bridge idle, the outputs sit near 0 V even with the capacitor charged: a high-side switch's off-state leakage (≤ 10 µA [EST]) lifts its output by ≤ 10 V. **So the high-side gate barriers see about 2 kV only during a shock.** A **shorted high-side switch shows as its output tap reading the capacitor voltage**, at every charge.
- **Current sense:** 2 mΩ pulse-rated shunt between `LS_SRC` and `HV_RTN`; TI **INA240A1** (gain 20) → `I_SHOCK` = **40 mV/A** (80 A ≈ 3.2 V, full scale for the main's ADC). Gives the delivered/fault status the HAL reports (§B9.3).
- **Bridge test load:** a cold-switched Cynergy3 **D-HR n/o reed relay** (10 kV stand-off, 5 V coil [UNVERIFIED part code, e.g. DAT70510F-HR]) connects a **2 kΩ** ceramic resistor between the two outputs on command (`BTEST_CMD`). At a low self-test voltage (≤ 100 V), firing the phase sequence into it gives ≤ 50 mA on the shunt: each switch is proved to turn on and off, and a shorted low-side switch shows as current flowing before its command. Requirement: the 2 kΩ resistor must survive a full shock diverted into it by a stuck relay (2 kV across 2 kΩ for 20 ms = **40 J**, ceramic composition). The test relay's coil is powered from `BRIDGE_5V`, so it too can move only when the interlock allows.

---

## §B7. Patient isolation and pad interface (sheet `13_patient_interface`)

### B7.1 Why a patient relay

With charging during the analysis (D-29), the capacitor is at full voltage while the front ends are measuring the pads. Without a relay, the pads would connect to the bridge outputs, which carry the off-state leakage of 3.3 kV switches (micro- to tens of microamps, rising with temperature [EST]) and any transient from the charger, straight into the heart-signal measurement and the patient. A relay that keeps the pads on the front ends at rest also gives the **front-end disconnect during the shock** with the same part, and adds a third series element against a shorted bridge switch.

### B7.2 The relay

**K1301: a double-pole changeover (DPDT) high-voltage relay.** At rest (coil off): `PT_PAD_A → PT_FE_A`, `PT_PAD_B → PT_FE_B`. Energised: `PT_PAD_A → HV_OUT_A`, `PT_PAD_B → HV_OUT_B`. Coil from `BRIDGE_5V` through a transistor commanded by `RELAY_CMD`: **the relay cannot reach the shock position unless the interlock allows a shock.** It is **cold-switched**: it moves only with the bridge off, so it never makes or breaks current.

Requirement (a qualification item, not a catalogue rating — none was found):

| Item | Requirement | Basis |
|---|---|---|
| Carry | 65 A peak exponential pulse, up to 15.5 ms per shock; ≥ 100 A for 20 ms without damage; 20 shocks in 1,300 s | DESIGN_TABLE; parts study B3 |
| Fault carry | 430 A for 5 µs (bridge short before the trip) | §B6.5 |
| Stand-off | ≥ 5 kV DC across open contacts and between poles; ≥ 5 kV contact to coil | the 5 kV external defibrillator test reaches the rest contacts |
| Contact resistance | ≤ 50 mΩ, stable within 10 mΩ | compression sensing through the rest contacts (parts study B3) |
| Coil | 5 V class (from `BRIDGE_5V`), intermittent duty ≤ 200 ms on, ≥ 10 % off | the permit window is 200 ms (config.rs) |
| Operate / release | ≤ 10 ms each | timing budget, §B8.4 |
| Life | ≥ 20,000 operations cold-switched | shocks plus bridge self-tests |
| Temperature | −20 to +65 °C | parts study B3 |

Reference candidate: **Gigavac G17** — a 2C (DPDT) relay Gigavac designed "as a patient discharge relay" for (older, sinusoidal) defibrillators, 10 A continuous carry, 12 V intermittent coil (100 ms on, 10 % duty) (Gigavac datasheet, search extract, 27 Nov 2026). Its pulse rating, size and 5 V availability are [UNVERIFIED]; a 12 V coil would need a small boost from `BRIDGE_5V` (a capture decision). Cynergy3 D-HR vacuum reeds (3 A carry) are **not** rated for the shock current (research notes §3).

### B7.3 Pad connector, pad-set identity (D-57) and presence

J1301 is made to order (parts study B3). Contacts: `PT_PAD_A`, `PT_PAD_B` (high voltage), and three low-voltage contacts kept apart from them inside the connector: `PAD_ID` (1-Wire data), `PAD_ID_RTN` (return, = GND), `PAD_PRESENT` (looped to `PAD_ID_RTN` inside the pad plug).

- **Identity:** a 1-Wire EEPROM in the pad plug (ADI **DS2431** class: 1 Kbit, factory-unique 64-bit ID [BK]) holds the pad set's serial number and use-by date; the 64-bit ID is the `pad_set_id` the firmware keeps heat history against (standby.rs `pads_replaced(Option<u32>)`).
- **Presence:** `PAD_PRESENT_N` reads low when a pad plug is fitted, independent of the EEPROM working.
- **Protection on the low-voltage pad lines:** they run beside 2 kV leads in the pad cable. Each gets a series resistor and a TVS at the connector [requirement].
- **Pad-area temperature (D-48):** an NTC next to the stored pads, on this sheet (`NTC_PADS`).

### B7.4 Leakage

- At rest, the patient connects only to the front ends: direct current is blocked by the drive capacitors, and the sense taps take nanoamps (bias network ≥ 10 MΩ [EST]). Alternating patient auxiliary current is the impedance excitation: about 0.34 mA RMS at 64 kHz (main) plus a smaller current at 128 kHz (supervisor), against an IEC limit of several milliamps at those frequencies [CALC §B3.4; UNVERIFIED limit].
- The device has no earth and no mains connection; leakage to the person holding it is set by the enclosure insulation and the dock (charging is never done on a patient; the dock contacts are dead off the dock).
- The bridge's leakage never reaches the patient except during the ≤ 40 ms the relay is energised.

---

## §B8. Hardware interlock (sheet `09_interlock`, with the stacks on `11_hv_storage`)

### B8.1 Signals

| Name | From | Encoding | Means |
|---|---|---|---|
| `SHOCK_REQ` | main | static, active high | main requests the shock |
| `PERMIT_CLK` → `PERMIT_OK` | supervisor | **dynamic**: a ≥ 1 kHz square wave from running code; a diode-pump detector (on 09, powered from `3V3_SUP`) turns it into a level that collapses within about 1 ms of the wave stopping | supervisor permits |
| `HOLD_M_CLK` → `HOLD_A_DRV` | main | dynamic | main holds path A off |
| `HOLD_S_CLK` → `HOLD_B_DRV` | supervisor | dynamic | supervisor holds path B off |
| `HOLD_OK` | storage sheet | open-drain wired-AND of two measured gate biases | both paths are held off = discharge path **not** active |
| `CHG_EN` → `CHG_GATED` | main → 09 → charger | static | charger runs |

### B8.2 Implementation

- **Consent buffers powered by the consenting controller's own rail:** the main's lines are buffered by 74LVC-family gates powered from `3V3_MAIN`; the supervisor's from `3V3_SUP` (power-off protected parts, so an unpowered buffer neither drives nor is back-powered). **Loss of a controller's supply removes its consent physically.**
- **Bridge power:** `5V0_SYS → TPS22918 (EN = SHOCK_REQ) → TPS22918 (EN = PERMIT_OK) → TPS22918 (EN = HOLD_OK) → BRIDGE_5V`. Three TI TPS22918 load switches in series (5.5 V, 2 A, quick output discharge [BK]); each has a 100 kΩ pull-down on its enable. Any one off removes the bridge's drive power, the high-side carriers, the low-side gate supply and the patient relay coil, and the quick-discharge pulls `BRIDGE_5V` down.
- **Readbacks:** `BRIDGE_ARMED_N` (open drain, low while `BRIDGE_5V` is present) and `HOLD_OK_RB_N` (open drain, low while `HOLD_OK`) to the main controller, so firmware can tell "refused" from "fault" (§B9.3) and test the interlock (§B8.5).
- **Charger gate:** `CHG_GATED = CHG_EN AND HOLD_OK` (one AND gate on `5V0_SYS`).

### B8.3 Truth tables, shown equal to interlock.rs

Let *m* = main holds (`HOLD_M_CLK` running), *s* = supervisor holds (`HOLD_S_CLK` running), *r* = `SHOCK_REQ`, *p* = `PERMIT_OK`, *e* = `CHG_EN`. Path A conducts unless *m*; path B conducts unless *s*. `HOLD_OK` = (A off) AND (B off) = *m* AND *s*.

**discharge_path_active** (interlock.rs: `!(main_hold && supervisor_hold)`)

| m | s | Path A | Path B | Discharge path active (hardware) | interlock.rs |
|---|---|---|---|---|---|
| 0 | 0 | on | on | **yes** | true |
| 0 | 1 | on | off | **yes** | true |
| 1 | 0 | off | on | **yes** | true |
| 1 | 1 | off | off | **no** | false |

**bridge_may_fire** (interlock.rs: `main_request && supervisor_permit && !discharge_active`)

| r | p | HOLD_OK | BRIDGE_5V present (hardware) | interlock.rs |
|---|---|---|---|---|
| 0 | x | x | no | false |
| x | 0 | x | no | false |
| x | x | 0 | no | false |
| 1 | 1 | 1 | **yes** | true |

Because `HOLD_OK` = NOT(discharge active), the three series switches compute exactly r AND p AND NOT(discharge_active).

**charger_may_run** (interlock.rs: `charger_enabled && !discharge_active`)

| e | HOLD_OK | CHG_GATED (CHARGE pin and VTRANS switch) | interlock.rs |
|---|---|---|---|
| 0 | x | off | false |
| 1 | 0 | off | false |
| 1 | 1 | **on** | true |

**Where the hardware is stricter than interlock.rs** (safe-side additions): `OVP_TRIP` forces both paths on and so forces "discharge active" regardless of *m* and *s*; `HOLD_OK` is *measured* (gate bias), so a pump fault reads as "discharge active" even with both clocks running. `bridge_may_fire` "true" makes drive *power* available; the main's timer commands, the leg lockout, the 25 ms window and the over-current latch then govern each switch (§B6.4).

### B8.4 Timing of one shock [EST]

Supervisor permit window 200 ms (config.rs). Inside it: `BRIDGE_5V` rises and settles (≤ 2 ms); low-side gate supply up (≤ 5 ms [UNVERIFIED MGJ2 start-up]); `RELAY_CMD` → relay closed and settled (≤ 10 ms); QHA on (≈ 0.1 ms); phase 1 (3–12 ms), gap 0.5 ms, phase 2 (2–8 ms); QHB off; relay released (≤ 10 ms); request withdrawn. Total **≤ 50 ms**, well inside 200 ms and inside the 1 s `delivery_timeout_ms`.

### B8.5 The interlock can test itself

Without any high voltage (capacitor empty, relay at rest, test relay open, all switch commands low), the main controller and supervisor can step through the combinations and watch `BRIDGE_ARMED_N`: request only → not armed; permit only → not armed; request + permit without both holds → not armed; all three → armed. This proves each series switch can **block**, which is the property that matters, and catches a shorted switch before it matters. A firmware self-test requirement (it does not exist yet).

### B8.6 The stuck-cancel line both controllers read independently

The cancel control is **two separate sealed switch contacts under one guarded cap** (§B10.4): contact 1 to `CANCEL_M_N` (pull-up to `3V3_MAIN` on the main's sheet), contact 2 to `CANCEL_S_N` (pull-up to `3V3_SUP` on the supervisor's sheet). The only shared conductor is `GND`. Each controller runs D-31's stuck-button rule on its own contact (config.rs `cancel_stuck_ms` 10 s; link.rs `SupervisorFault::CancelStuck`). A broken contact on one side shows as the two controllers disagreeing about cancel.

---

## §B9. Sensing

### B9.1 Two independent capacitor-voltage chains (D-28: 1 % + 7 V each)

| | Main chain | Supervisor chain |
|---|---|---|
| Divider string (sheet 11) | 10 × 4.02 MΩ **Stackpole HVCR2512** (thick film, screened voltage coefficient < 2 ppm/V in 2512) = 40.2 MΩ | 12 × 3.32 MΩ **Vishay TNPV1206** class high-voltage thin film (< 2 ppm/V, research notes) = 39.8 MΩ [UNVERIFIED value range] |
| Bottom leg | two 80.6 kΩ in parallel (40.3 kΩ), same family as the string, so one open bottom resistor doubles the ratio (a large disagreement, caught) instead of letting the tap rise to high voltage | two 79.6 kΩ in parallel, same family |
| Ratio | about 1:998 → 2,500 V full scale = 2.50 V | about 1:1,000 |
| Tap protection | 10 kΩ series + clamp to the reader's rail + 3.6 V TVS, on the storage sheet | same |
| ADC | **TI ADS1220** (24-bit delta-sigma, SPI, 4 inputs) on sheet 07 | **ADI AD7171** (16-bit delta-sigma, 125 samples/s, serial) on sheet 08, behind a zero-drift buffer that also feeds the over-voltage comparator |
| Reference | **TI REF3425** (2.5 V, 0.05 %, 6 ppm/°C max [BK]) | **ADI ADR3425** (2.5 V, 0.1 %, 8 ppm/°C max [BK]) |
| Other inputs | ADS1220 AIN1/AIN2 = `OUT_A_TAP`/`OUT_B_TAP`; AIN3 = capacitor NTC (main) | — |

**Different makers and technologies in every element** (thick vs thin film, TI vs ADI converter, TI vs ADI reference), so the two chains cannot drift together and still agree (parts study §7).

**Error budget, worst case over −20 to +60 °C and life, after one-point factory calibration at 25 °C** [CALC; component figures BK except where the parts study cites them]:

| Source | Main (HVCR) | Supervisor (thin film) |
|---|---:|---:|
| Calibration residual | 0.05 % | 0.05 % |
| Top-to-bottom temperature tracking × 45 K | 50 ppm/K → 0.23 % | 10 ppm/K → 0.05 % |
| Voltage coefficient (≈ 200 V per string resistor × 2 ppm/V) | 0.04 % | 0.04 % |
| Life drift (thick film 0.5 %, thin film 0.25 %) | 0.50 % | 0.25 % |
| Board surface leakage (40 MΩ against 100 GΩ) | 0.04 % | 0.04 % |
| ADC gain drift and non-linearity | 0.03 % | 0.04 % |
| Reference drift × 45 K | 0.03 % | 0.04 % |
| **Linear sum** | **0.92 %** | **0.51 %** |
| Fixed part (offset, 1 V rounding) | ≈ 1 V | ≈ 1 V |
| Allowance at 1,950 V (1 % + 7 V = 26.5 V = 1.36 %) | 0.92 % + 1 V = 19 V: **meets, 7 V spare** | 0.51 % + 1 V = 11 V: **meets** |
| Allowance at 1,150 V (1.61 %) and 400 V (2.75 %) | meets | meets |

**Life drift is the biggest term in the main chain.** That is parts study question A (recalibrate at service, or pay for low-drift dividers). This design uses a thick-film string on one side deliberately, for technology diversity; if the owner prefers, both strings can be thin film from two makers (§17).

### B9.2 Six temperature points (D-48, D-56)

| Point (`TempPoint`) | Sensor | Read by |
|---|---|---|
| `Cell1`, `Cell2` | NTCs in the pack → MAX17320 thermistor inputs | main, over I²C |
| `Capacitor` | NTC bonded to the capacitor case → ADS1220 AIN3, biased from `VREF_M` (ratiometric) | main |
| `Pads` | NTC next to the stored pads, sheet 13 → main's own ADC, biased from the main's analog supply | main |
| `Board` | STM32U585 internal sensor | main |
| `Supervisor` | **second NTC on the capacitor case**, own wire → supervisor's own ADC, biased from `3V3_SUP` | supervisor only |

NTC requirement: 10 kΩ, B-value tolerance ≤ 1 %, so ±2 °C from 0 to 80 °C (parts study R-H1) [EST]; readable −40 to +100 °C. All NTCs are low-voltage parts; the capacitor case is insulated from its terminals (Appendix A: ≥ 5 kV terminal-to-case).

### B9.3 Current sense and the shock status the HAL reports

`MainHardware::shock_status` is derived by the board layer from:
- `BRIDGE_ARMED_N` never asserted within the timeout → **Refused** (the interlock said no).
- `I_SHOCK` shows phase 1 and phase 2 current of the expected sign, size and duration (compared with the plan) → **Delivered**.
- `OC_TRIP_N` asserted, or no current with the bridge armed and the relay commanded, or current outside the plan → **Fault**.
- In addition, the capacitor voltage before and after (both chains) gives the energy taken: E = ½·C·(V₀² − V₁²), cross-checking the current integral (TEST_INSTRUMENT §5.4).

### B9.4 Answer to Q-TI-16 (test points for the test instrument)

Yes: the prototype carries low-voltage test points for the scaled capacitor voltage (both chains), both scaled output voltages, the shock current signal, and fast event markers (request, permit, armed, each switch command, relay command). No test point is placed on any high-voltage or patient net (§15).

---

## §B10. User interface (sheet `06_ui`; alarm on `05_supervisor_mcu`)

### B10.1 Speaker and amplifier

ADI **MAX98357A** I²S class-D amplifier (3.2 W into 4 Ω at 5 V, −40 to +85 °C [BK]) on `5V0_SYS`, shutdown pin `AMP_SD_N` (default off). Sealed 28 mm micro-speaker with vent membrane, ≥ 85 dB(A) at 1 m target (parts study §8) [speaker part UNVERIFIED]. Prompts and the metronome (`MainHardware::prompt`, `metronome_beat`) both play through it.

### B10.2 The supervisor's own alarm (D-26)

On the supervisor's sheet, powered only from `3V3_SUP`: an externally driven **piezo transducer**, driven differentially by two supervisor pins through a dual buffer (≈ 6.6 V peak-to-peak across the element), target ≥ 85 dB at 10 cm [EST], plus a red **fault light**. `SupervisorHardware::set_fault_alarm`. It works with the main controller, `5V0_SYS` and `3V3_MAIN` all dead.

### B10.3 Display and ready indicator that works with the main controller off

- **Display:** Sharp **LS013B7DH05** memory LCD (1.26 in, −20 to +70 °C operating, parts study §8), powered from **`3V3_AON`**. Its polarity-inversion input is driven by the wake timer's 1 Hz clock output (`RTC_CLKOUT`), so the last image — including "READY" or "NOT READY" — stays on the screen while the controllers are off, for a few microwatts [BK]. The main controller writes it when running. Its inputs are pulled down on the UI sheet so they are defined when the main is unpowered; `LCD_DISP` is pulled *up* to `3V3_AON` (display on) through 1 MΩ. A larger memory LCD (Sharp LS027B7DH01, 2.7 in) needs 5 V and so cannot hold its image from the always-on rail; choosing it means a 5 V always-on supply (§17).
- **Ready LED:** an SN74AUP1G74 latch on `3V3_AON` holds the ready state (`READY_D`, `READY_CLK` from the main; `MainHardware::set_ready_indicator`). A gate combines it with the 1 Hz clock through a short-pulse network to flash a green LED for about 3 ms each second: about 20 µW [CALC: 2 mA × 3 ms × 3.3 V].

### B10.4 Controls

- **Cancel:** two sealed tactile switches (C&K KSC-class, IP67 [UNVERIFIED part code]) under one guarded cap (made-to-order mechanical part), wired as in §B8.6.
- **Adult/child switch:** a sealed two-pole changeover slide switch. Pole A gives `MODE_ADULT_N` and `MODE_CHILD_N` to the main (both read, so "both" or "neither" is a detectable fault: `MainHardware::patient_mode`); pole B gives `MODE_ADULT_HW` (closed only in ADULT) to the charger's hardware ceiling (§B4.4).
- **Power button:** sealed tactile switch, `PWR_BTN_N` (§B1.7).

### B10.5 Dock interface

See §B1.4: `DOCK_TX`, `DOCK_RX`, `DOCK_DET_N`. Nothing arriving through the dock can charge the capacitor or shock: the dock's data reaches only the main controller's UART, and firmware treats it as untrusted (parts study §5).

---

## §B11. Durable memory and timekeeping

| Function | Part | Why |
|---|---|---|
| Durable store (`DurableStore`: heat record, pad heat totals, capacitor charge count, two copies with checksums — standby.rs) | Infineon **CY15B104QN** 4 Mbit SPI F-RAM, −40 to +85 °C [BK] | Survives battery removal with no power (D-57); written about every minute: F-RAM endurance ~10¹⁴ writes [BK] versus about 5 × 10⁶ writes in 10 years [CALC: 525,600 minutes × 10 years]. Size: 2,160 hourly records (heat.rs `HEAT_LOG_CAPACITY`) × about 16 bytes × 2 copies ≈ 70 KB [EST] against 512 KB |
| Event record (1 KB/s, 1.8 MB per 30-minute rescue; parts study §7) | Winbond **W25Q128JV** 16 MB SPI NOR flash | parts study §7 |
| Wake timer (TestSchedule's low-power seconds counter that keeps counting while "off"; wakes for logging every 1 or 5 minutes and for self-tests) | Micro Crystal **RV-3028-C7** (≈ 45 nA, 32-bit Unix-time seconds counter, countdown timer and alarm interrupt, 1 Hz clock output, −40 to +85 °C [BK]) | Counts seconds, which is exactly what schedule.rs assumes; its interrupt (`RTC_INT_N`) turns on `3V3_MAIN` through the wake logic. If the battery is removed it stops, and schedule.rs then treats nothing as known — the behaviour the firmware already expects |

The F-RAM sits on `3V3_MAIN` (it needs no power to retain). The flash and F-RAM share one SPI bus with separate chip selects, local to the main controller's sheet.

---

## §B12. Creepage, clearance and partitioning

**The spacing tables above 566 V are not available to the project**: IEC 60601-1 Table 12 is reproduced in free sources only up to 566 V DC (research notes §1), and IEC 60601-2-4 has not been bought. **Placeholder rule used here [UNVERIFIED]:**

| Between | Placeholder | Basis |
|---|---|---|
| Any `HV_*`, `HV_FLOAT` net and any low-voltage net, outer layers, uncoated | **12.5 mm** creepage and clearance | IPC-2221 external-uncoated extrapolation at 2.5 kV (2,000 V working + 25 %), research notes §1 [CALC there] |
| Same, inner layers (solid insulation) | 5.3 mm | IPC-2221 internal (B1) at 2.5 kV |
| Adjacent nodes inside one high-voltage string (divider, bleeder, stack) | 0.6 mm + 0.005 mm per volt of their difference (e.g. 1.6 mm for 200 V) | same extrapolation, scaled by the voltage actually across the gap |
| `PT_*` (applied part) and other low-voltage nets | **4.0 mm** creepage and clearance | the defibrillation-proof applied-part figure (MEDTEQ, research notes §1) |
| Everything inside potted or barrier-insulated areas | per the solid-insulation rules of the standard, once bought | compact designs depend on potting (research notes §1) |

**Net classes** in `hierarchy.toml`: `HV` (anything galvanically on the capacitor or the charger secondary), `HV_PULSE` (the 65 A shock path: `HV_CAP_P`, `HV_BUS`, `HV_OUT_*`, `HV_RTN`, `PT_PAD_*`), `HV_FLOAT` (high-side gate-drive circuits, `HSA_*`/`HSB_*`), `PATIENT` (`PT_*`), `PWR_HI` (`VBAT_*`, `VTRANS`), `PWR`, `Default`. Net-class clearance in KiCad is the larger of the two classes, so the per-string rule above becomes a custom rules file at layout.

**HV zone and the barrier.** High-voltage group sheets: 10 (charger secondary side), 11, 12, 13. Patient group: 13, 14, 15. The barrier runs:
- across the charger transformer T1001 (primary on the low-voltage side of the barrier, secondary on the high-voltage side);
- across the high-side gate transformers T1201/T1202 (the `HV_FLOAT` islands on their secondaries);
- along the low-voltage ends of the four divider strings (taps and bottom legs are low-voltage; everything above is high-voltage);
- across the discharge stacks' bottom devices (gate and source low-voltage, drain high-voltage);
- across the patient relay (contacts high-voltage/patient, coil low-voltage);
- around the pad connector's high-voltage contacts, separated from its three low-voltage contacts.

**This is not galvanic isolation.** `HV_RTN` is tied to `GND`: the barrier is spacing between potentials inside one circuit, not an insulation barrier protecting a person. Protection of the person holding the device comes from the enclosure (insulating, sealed, no exposed metal except the dock contacts, which are dead off the dock); protection of the patient comes from the applied-part design (§B7). **Whether the standard requires a means of protection between the shock circuit and the dock contacts or enclosure is an open question** (§16).

**One board or two.** Expected: **two boards**. The low-voltage board carries sheets 01–09; the high-voltage board carries 10–15. Crossing nets are listed in the contract (§6); the heaviest is `VBAT_SYS` to the charger (9 A average, 21 A peak). The sheet grouping keeps a single-board layout possible too.

---

## §11. Rail tree

```
Cells (2S, 5.5–8.4 V)
└─ MAX17320 protection FETs ─ VBAT_PACK ─ F101 25 A ─ VBAT_SYS ─┬─ F1001 15 A ─ LM74502 switch (CHG_GATED) ─ VTRANS ─ LT3751 / T1001 ─ HV_CAP_P (≤ 2,200 V)
   (BQ25887 charges VBAT_PACK from VDOCK)                       ├─ 0.82 Ω battery load (BAT_LOAD_EN, ≤ 250 ms)
                                                                ├─ fuse 1 A ─ TPS629210 #1 ─ 3V3_MAIN ─ main MCU, flash, F-RAM, ADS1292R (filtered), ADS1220, REF3425, main consent buffers
                                                                ├─ fuse 1 A ─ TPS629210 #2 ─ 3V3_SUP ─┬─ supervisor MCU, AD7171, ADR3425, OVP comparator, alarm, supervisor consent buffers
                                                                │                                     └─ LDO ─ 1V8_SUP ─ MAX30009
                                                                ├─ fuse 3 A ─ TPS62130 ─ 5V0_SYS ─┬─ MAX98357A, LT3751 VCC, interlock logic, HOLD_OK comparators
                                                                │                                 └─ 3 × TPS22918 (REQ · PERMIT · HOLD_OK) ─ BRIDGE_5V ─┬─ SN6505B × 2 ─ T1201/T1202 (high-side gates)
                                                                │                                                                                       ├─ MGJ2D052005SC ─ +20/−5 V (low-side gates)
                                                                │                                                                                       ├─ LDO ─ BRIDGE_3V3 (bridge logic, INA240)
                                                                │                                                                                       └─ patient relay and test relay coils
                                                                └─ TPS70933 ─ 3V3_AON ─ RV-3028-C7, ready latch + LED, memory LCD, wake logic
VDOCK (5 V from the dock, after TPS25947) ─ BQ25887 ─ VBAT_PACK
HV_RTN ═ NT1101 ═ GND (one point, sheet 11)
```

| Rail | Peak | Typical in a rescue | Standby |
|---|---:|---:|---:|
| VTRANS | 21 A | 9.2 A while charging (≤ 7 s per 132 s) | 0 |
| 5V0_SYS | 1.5 A [EST] | 0.15 A | off |
| BRIDGE_5V | 0.8 A for ≤ 200 ms [EST] | 0 | off |
| 3V3_MAIN | 80 mA | 25 mA | off (on for ~50 ms per wake) |
| 3V3_SUP | 60 mA | 15 mA | off |
| 3V3_AON | 5 mA | 0.1 mA | ≈ 10 µA |

---

## §12. Power and standby budget [EST unless CALC]

| Standby item (device off, worn) | Current | Power at 7.4 V |
|---|---:|---:|
| MAX17320 (hibernate) | 10–30 µA [BK] | 74–222 µW |
| BQ25887 on battery, no input | 5–20 µA [BK] | 37–148 µW |
| TPS629210 × 2, TPS62130 disabled | ≈ 3 µA | ≈ 22 µW |
| TPS70933 quiescent | ≈ 1 µA | ≈ 7 µW |
| RV-3028-C7 | 45 nA | < 1 µW |
| Memory LCD holding an image + 1 Hz inversion | — | 10–15 µW [BK] |
| Ready LED flash | — | ≈ 20 µW [CALC] |
| Logging wakes (main on ≈ 50 ms every 5 min at ≈ 15 mA, 3.3 V) | — | ≈ 8 µW [CALC: 2.5 mJ / 300 s] |
| Discharge paths, bleeder, dividers (capacitor empty) | 0 | 0 |
| **Standby subtotal** | | **≈ 0.18–0.44 mW** |
| Weekly full-energy test, averaged (parts study §9) | | 0.42 mW |
| Daily low-voltage check, averaged (parts study §9) | | ≈ 0.17 mW |
| **Total including self-tests** | | **≈ 0.77–1.03 mW** (0.018–0.025 Wh a day) |

The parts study budgeted 1.05–1.5 mW; this design sits at or under it, the gauge and charger quiescent currents being the largest unknowns [UNVERIFIED]. Rescue average with D-29 is unchanged from the parts study (about 2.5–2.8 W).

---

## §13. Single-fault table for the shock path

"Caught by" names the check that detects or contains the fault. M = main controller, S = supervisor, H = hardware interlock or other hardware.

| # | Single failure | What happens | Caught by | Result |
|---|---|---|---|---|
| 1 | M firmware freezes with `SHOCK_REQ` and timer commands stuck high | No permit: S sees the heartbeat stop (250 ms), stops `PERMIT_CLK` and `HOLD_S_CLK`; path B conducts | S; H (permit must be dynamic) | No shock; capacitor empties; S alarm |
| 2 | M freezes with `HOLD_M_CLK` pin stuck high, charger command on | Stuck high is not a clock: path A conducts within ms; `HOLD_OK` falls; charger supply cut (D-25, F-3) | H | Capacitor empties; S alarm |
| 3 | S freezes with `PERMIT_CLK` stuck (either level) | Permit detector output collapses in ≈ 1 ms | H | No permit → no shock (failure to shock), M reports supervisor lost, "continue CPR" |
| 4 | S freezes but a free-running timer keeps toggling `PERMIT_CLK` | Would look like a permit | Design rule: the clock is produced by code that stops with the loop (§B5.4, §B8.1); S watchdog resets pins | Requires a firmware discipline; flagged in §16 |
| 5 | M's rhythm analysis wrongly says "shock" | S checks voltage, pads, armed time, heartbeat, cancel — not the rhythm | Two-method agreement inside M (ARCHITECTURE.md) | **Not caught by S or H**: an accuracy limit, not a hardware fault. Stated, not solved |
| 6 | Discharge stack A fails short | Path A always on: `HOLD_OK` never true; charger never runs | Switch-on self-test (charge to 400 V fails) | No shock possible; loud fault at switch-on |
| 7 | Discharge stack A fails open | Path B still discharges | τ check on every dump (0.45 s instead of 0.225 s) | Detected at the next dump; capacitor still empties in < 2 s |
| 8 | One dump resistor opens | That path open; as 7 | τ check | As 7 |
| 9 | Hold charge pump fails (no bias) | That stack conducts; `HOLD_OK` false | H | Cannot charge; detected at switch-on |
| 10 | FB ceiling divider top opens | FB reads 0: charger would run on | Primary-side trip 2,162 V; OVP comparator 2,160 V; both controllers' 2,200 V rule; M's regulation stops it at 1,950 V first | Contained |
| 11 | Charger primary MOSFET fails short | Battery short through the transformer primary | F1001 (15 A) opens; MAX17320 short-circuit protection | Charger dead: charge time-out fault; rest of device runs, S alarm works |
| 12 | Output diode fails short | Capacitor discharges back into the secondary | LT3751 current limit; charge time-out | No shock; fault |
| 13 | Main voltage chain reads 10 % low (e.g. one string resistor shorted) | M would charge 10 % high | S's own reading disagrees beyond 2 % + 24 V for 500 ms → S releases and latches (config.rs) | Contained; no shock |
| 14 | Either chain's bottom leg opens | Tap would rise to HV | Two-resistor bottom leg: ratio doubles, reading halves → disagreement → S latches; tap clamp protects the ADC | Contained |
| 15 | QHA (high side) fails short | `HV_OUT_A` follows the capacitor | `OUT_A_TAP` reads high at the next charge: M refuses to shock; relay at rest keeps pads off the bridge | No shock; fault |
| 16 | QLB (low side) fails short | Phase 1 would start the instant QHA turns on and could not be truncated by QLB | Bridge self-test into the test load at ≤ 100 V shows current before the command; if undetected: the 25 ms hardware window, the high-side turn-off and the over-current latch end the pulse; energy up to the full stored 190 J at high patient resistance | **Over-delivery possible if the self-test has not run since the failure**; flagged §16 |
| 17 | Leg lockout logic fails (allows HA + LA) | Capacitor short through R_S + L_S | Over-current latch at 90 A within ≈ 2 µs; R_S limits to ≈ 430 A | Switches stressed above IDM for ≈ 2 µs [UNVERIFIED survival]; shock lost |
| 18 | One of the three interlock series switches fails short | That consent no longer needed; two remain | Interlock self-test (§B8.5) | Latent until the next test; two of three checks still active |
| 19 | `BRIDGE_5V` quick-discharge fails, rail lingers | Drive power decays slowly after consent is withdrawn | High-side carriers stop when commands drop; lockout logic loses power below its threshold | Contained by the commands; flagged for simulation |
| 20 | Patient relay welded in the shock position | Front ends disconnected; pads on bridge outputs (pulled to `HV_RTN`, leakage µA) | ECG flat and impedance open with pads on → "attach pads"; no shock | Failure to shock, announced. **Relay position is not sensed** (§16) |
| 21 | Patient relay fails to close | Shock into an open circuit | `I_SHOCK` zero → Fault | Failure to shock, detected |
| 22 | High-side gate transformer insulation breaks down | Up to 2 kV reaches the SN6505B and `BRIDGE_5V` during a shock | Not caught before it happens; `BRIDGE_5V` is separated from the controllers by the load switches and consent buffers [EST] | Damage inside the device; the enclosure must keep it from the user (§B12). Part qualification (§14) is the control |
| 23 | Cancel contact to M stuck closed | M sees permanent cancel | D-31 stuck rule in M; S's contact disagrees | No shock, fault (D-31) |
| 24 | `3V3_MAIN` regulator fails | M dead; path A conducts | S: heartbeat never/no longer arrives; S alarm (D-26) | No shock, loud |
| 25 | `3V3_SUP` regulator fails | S dead; path B conducts; no permit | M: link lost | No shock; M speaks the fault |
| 26 | `5V0_SYS` fails | No charger, no bridge power, path A pump unaffected (on 3V3_MAIN) but `HOLD_OK` comparators unpowered → `HOLD_OK` false | M and S | No shock; S alarm and M prompt available (both on their own rails) |
| 27 | `HV_RTN`–`GND` tie (NT1101) opens | Divider references and the bridge return float from the capacitor negative | Both chains read nonsense: disagreement or implausible values → S latches | Contained; the tie must be robust (wide copper, multiple vias) |
| 28 | Storage capacitor shorts internally | Charger into a short: no voltage rise | Charge time-out | No shock; fault |
| 29 | Over-voltage comparator fails "no trip" | Loss of one of three hardware over-voltage layers | Latent; FB ceiling and primary trip remain | Contained |

**Two gaps the table exposes, carried to §16:** fault 16 (a shorted low-side switch between self-tests could deliver more than the plan) and fault 20 (relay position not sensed).

---

## §14. Made-to-order and qualification parts

| Ref | Part | Specification | Status |
|---|---|---|---|
| C1101 | Storage capacitor, 100 µF, ≥ 2,200 V | parts study Appendix A | D-41, D-42 |
| T1001 | Charger flyback transformer, 1:50, 7.5 µH | §B4.5 (refines Appendix B1) | D-42 |
| T1201, T1202 | High-side gate-drive transformers | Below | D-42 |
| L1201 | Series protective inductor, 30 µH, ≥ 450 A | §B6.5 | new; simple coil, any magnetics maker |
| K1301 | Patient relay (DPDT, cold-switched, 65 A pulse carry, 5 kV) | §B7.2 | qualification of a catalogue relay (Gigavac G17 class) or made to order |
| J1301 | Pad connector, device and pad side | parts study Appendix B3, plus three low-voltage contacts (§B7.3) | D-42 |
| — | Pad set with 1-Wire identity EEPROM | parts study Appendix B4 | D-42 |
| — | Battery pack assembly (2 × P28A, NTCs, keyed connector) | §B1.1 | pack assembler |
| — | Cancel cap with two switch contacts | §B10.4 | mechanical |

**T1201/T1202 requirements (refines Appendix B2 for the chosen topology):** push-pull primary for the SN6505B at 5 V, 420 kHz [BK]; rectified secondary 18–20 V at ≥ 30 mA; **insulation primary-to-secondary ≥ 2,200 V DC working for 10 years** (target 2,750 V [parts study B2]), withstand ≥ 5 kV DC for 60 s [EST], partial-discharge inception ≥ 3.3 kV after humidity soak; **inter-winding capacitance ≤ 10 pF** (limits common-mode current at ≈ 1 V/ns to ≤ 10 mA); −20 to +85 °C; ≤ 2 cm³ each. Catalogue SN6505 transformers are rated about 400 V rms reinforced working [BK] — too low.

**Weight delta against the parts study's lower-risk set [EST]:** second discharge path (3 × TO-268 + 3 resistors + pump) +15–20 g; L1201 air-core +8–12 g; R1201 and bridge test parts +5 g; patient relay larger than the reed relays budgeted +5–15 g (G17 size unknown); two gate transformers instead of four isolated supplies −5 g; MAX30009 +1 g. **Total about +25–45 g**, so the lower-risk set moves from about 832 g to about **860–880 g**, inside the accepted 780–900 g (D-43) but with less room.

---

## §15. Test points

All on low-voltage nets; **none on any `HV_*`, `HV_FLOAT` or `PT_*` net** (design rule; contract R9).

| Test point | Net | Sheet | For |
|---|---|---|---|
| TP Vcap main | `VCAP_M_TAP` (1:998) | 11 (low end) | Q-TI-16 capacitor voltage, residual after shock |
| TP Vcap sup | `VCAP_S_TAP` (1:1,000) | 11 | same, second chain |
| TP Out A / Out B | `OUT_A_TAP`, `OUT_B_TAP` | 12 | output voltage during phases |
| TP I shock | `I_SHOCK` (40 mV/A) | 12 | current waveform |
| TP request / permit / armed | `SHOCK_REQ`, `PERMIT_OK` (local on 09), `BRIDGE_5V` | 09 | request-to-delivery timing |
| TP switch commands (after lockout) | the four gated commands (local on 12) | 12 | phase start/end markers |
| TP relay | `RELAY_CMD` | 13 | relay timing |
| TP hold | `HOLD_OK`, each stack's bottom-gate bias via a 1:3 divider (local on 11; reads −3 V when held) | 11 | discharge path state |
| TP charger | `CHG_GATED`, `CHG_DONE_N` | 10 | charge timing |
| TP faults | `OVP_TRIP`, `OC_TRIP_N` | 08, 12 | — |
| TP rails | every rail, and `GND` beside each | 01–03, 09 | — |
| TP front end | `AFE_DRDY_N` | 14 | sample timing |

---

## §16. Ranked open technical questions

| Rank | Question | Why it matters | What would settle it |
|---:|---|---|---|
| 1 | **Do the 3.3 kV SiC switches survive 65 A for 3–15 ms, repeatedly, and ≈ 120 A for 2 µs in a fault?** | No datasheet gives a millisecond rating or the short-circuit withstand at 2 kV | Microchip/Navitas statement and Zth curves; then single-pulse tests at low voltage (later, under the lab policy) |
| 2 | **Gate transformers T1201/T1202 at 2,200 V for 10 years in ≤ 2 cm³** | The made-to-order part the shock path depends on | Quotes and partial-discharge data from magnetics makers; and the standard's definition of working voltage for ≤ 20 ms exposures (if it counts as transient, catalogue isolated drivers may qualify) |
| 3 | **Patient relay K1301: is any relay rated to carry the shock?** | No catalogue pulse rating found; size and weight unknown | Gigavac (G17) and others' pulse data; or a made-to-order relay |
| 4 | **Charger noise in the impedance and ECG channels during D-29 charging** | D-37 relies on the impedance channel being clean while charging | Measurement with the charger running into a low-voltage dummy at ≤ 50 V output; fallback: firmware charge pausing, or a synchronised fixed-frequency charger |
| 5 | **Spacing at 2.2 kV (IEC 60601-1, 60601-2-4)** | Could change board size, potting and the high-side drive decision | Buy the standards |
| 6 | Capacitor density, mass and hot life (parts study rank 1) | 30 % of the mass | Maker quotes against Appendix A |
| 7 | **Depletion stack release: safe operating area and voltage sharing during the transition** | The discharge path is the safety default; its first microseconds are unsimulated | Circuit simulation (SPICE) with the IXYS models; then datasheet FBSOA check |
| 8 | **Shorted low-side switch between self-tests (fault 16)** and **relay position not sensed (fault 20)** | Two single faults with weak detection | Options: run the bridge self-test at every switch-on (≤ 100 V, ≈ 0.5 J); add an auxiliary contact or a sense path for K1301 |
| 9 | Front-end protection under the 5 kV external test, and DPCR pulse ratings | Applied-part compliance | DPCR datasheet energy rating; the standard |
| 10 | ADS1292R drive level into capacitive coupling; MAX30009 temperature range and package | The impedance resolution in §B3.4 assumes ≈ 3 V p-p drive | Full datasheets; low-voltage bench measurement |
| 11 | Voltage-sensor drift over life (D-28) | Main chain uses 0.92 % of its 1.36 % allowance | Owner decision on recalibration; or thin film on both chains |
| 12 | Cold battery at 9.2 A average / 21 A peak (higher than the parts study's 7.6/15 A) | Q-39 margin is thinner | Cold tests (parts study §5); or lower IPK and accept a longer cold charge |
| 13 | LT3751 at N = 1:50 and 2 kV (outside the maker's reference designs, which stop at about 450 V) | Primary-side trip accuracy, secondary capacitance effects | ADI applications engineering; the transformer maker's measured parameters |
| 14 | Output diode reverse recovery (150 ns vs < 100 ns advised) | A few % efficiency, charge time | A faster 3–5 kV diode, or measurement |
| 15 | Dump resistor repeat rating (Ohmite EY) | 5,000 pulses of 63 J | Ohmite statement |
| 16 | Whether the standard requires a means of protection between the shock circuit and the dock contacts/enclosure | `HV_RTN` = `GND` design; the dock contacts are exposed | The standard |
| 17 | Supervisor PERMIT/HOLD clocks must come from code that stops with the loop (fault 4) | A firmware discipline the hardware relies on | Board-support design review |

---

## §17. Questions for the owner

1. **How is the device switched on in an emergency?** This design assumes a sealed power button separate from cancel. Alternatives: pulling the pad cover, drawing it from the holster (a magnet switch). Affects the UI sheet only.
2. **Q-47: one data contact (dock is the reader) or two (device speaks USB)?** The design assumes one; the contract reserves names for two.
3. **Second discharge path: about +20 g.** It removes a single point of failure that would leave the capacitor charged for up to two hours. Keep it (recommended)?
4. **Both voltage chains thin film?** The main chain uses thick film (HVCR) on purpose, for technology diversity; it uses more of the D-28 allowance through life drift. Alternative: thin film from two different makers. Either way, accept periodic recalibration at service (parts study question A)?
5. **Display size.** The 1.26 in memory LCD can keep "READY / NOT READY" on screen while off from the always-on supply. A 2.7 in display needs a 5 V always-on supply (more standby power, a few µW to tens of µW).
6. **Internal resistance (D-23 depends on 5 Ω).** The design adds a 3.9 Ω protective resistor to keep the total near 5 Ω, which also limits fault current to ≈ 430 A. A smaller resistor would waste less energy as heat but raise fault current and change the waveform model. Keep 5 Ω for now?
7. **Simulator parameters.** The real dump resistance (2,250 Ω with both paths, 4,500 Ω with one) and total bleed (≈ 6 MΩ) differ from the simulator's 2,000 Ω and 50 MΩ. Should the firmware team update `aed-sim`? (Not changed here.)
8. **Bleeder: about 37 minutes to below 50 V** with everything intact, 2 hours with only the bleeder. Faster costs energy while charged (research notes §7). Acceptable as a last resort?

---

## §18. Decisions in DECISIONS.md that are strained

DECISIONS.md is not changed here. These are the decisions this design finds hard to meet, for the owner to look at.

| Decision | Strain |
|---|---|
| **D-29** (charge during analysis) with **D-36/D-37** (compression sensing decides "hands off") | The charger's sweeping switching frequency crosses every impedance carrier during every analysis. Whether that corrupts the compression signal is unknown until measured (§B3.7, §16 rank 4). If it does, the cures cost charge time (pausing) or a different charger design. |
| **D-44** (two 18650 cells) and **Q-39** | The charger needs 9.2 A average / 21 A peak from a cold pack, not the 7.6/15 A the parts study assumed. The −20 °C margin is thinner than D-44 was decided on. |
| **D-43** (700 g goal; 780–900 g prototype) | Safety additions add an estimated 25–45 g (§14). The prototype stays inside 900 g on paper; the 700 g goal loses about a third of its remaining slack. |
| **D-41** (capacitor fault figure about 400 A) | The fault current with R_S = 3.9 Ω is about 430 A — slightly above Appendix A's 400 A figure. Appendix A should say 450 A, or R_S rises to 4.3 Ω (total ≈ 5.1 Ω). |
| **D-28** (1 % + 7 V over life) | Met at manufacture with margin; life drift of the thick-film chain uses most of the allowance (§B9.1). Realistic only with recalibration or thin film on both sides. |
| **D-13** wording vs ARCHITECTURE.md | ARCHITECTURE.md ("What is deliberately not designed yet") still says HV schematics wait for the patent review and lab policy; D-13 (revised 27 Nov) publishes them as drawn. The architecture document needs a line changed (not done here: out of this task's scope). |
| **D-23** (5 Ω internal resistance assumed) | Kept true by design choice (R_S), not by necessity; see owner question 6. |
| **REQUIREMENTS "≤ 7 s"** | Met on paper at 5.5 V and 65 % efficiency with 0.6 s spare (§B4.2); a lower efficiency (60 %) would miss it. |

---

## §19. What I could not verify

- **ADI documents:** ADI's own servers timed out; the LT3751 datasheet (rev D) was read from a distributor mirror. MAX17320, MAX30009, MAX98357A, AD7171, ADR3425 and DS2431 figures are from product pages or background knowledge.
- **TI parts:** UCC5350, SN6505B, TPS629210, TPS62130, TPS70933, TPS22918, LM74502, TPS25947, TLV3201, TLV3012, INA240, ADS1220, REF3425, SN74AUP1G74 — part choices and key figures from background knowledge; product pages not opened this pass.
- **SiC switches:** millisecond pulse ratings, transient thermal impedance, gate-drain charge and short-circuit withstand for MSC080SMA330B4N and G2R120MT33J.
- **Depletion MOSFET:** IXTT10N100D2 figures from third-party datasheet copies (2017 "preliminary" revision); safe operating area at 670 V not checked.
- **Relays:** Gigavac G17 pulse rating, size and coil options; Cynergy3 D-HR 5 V-coil part code.
- **Resistors:** Ohmite EY voltage and repetition ratings (Ohmite's site refused automated access); DPCR pulse-energy rating and its 2024 lifecycle notice; Vishay TNPV value range; HVCR life drift and temperature coefficient.
- **Front ends:** ADS1292R modulation frequencies and drive level; MAX30009 temperature range; channel-to-channel crosstalk; charger interference (measurement only).
- **Controllers:** I/O counts per package for STM32U585VIT6 and MCXA153VLH; MCX A153 oscillator ranges.
- **Standards:** every IEC 60601-1 / 60601-2-4 clause, limit and spacing figure (not bought); patient auxiliary current limits.
- **Prices and stock:** none checked; all [UNVERIFIED].
- **Anything about real behaviour:** nothing here has been simulated in circuit form or measured.

---

## §20. Sources

Accessed 27 November 2026 unless stated. "Read" = document text read; "page" = product page, listing or search extract only.

| Part / topic | Source | Access |
|---|---|---|
| LT3751 | https://www.analog.com/en/products/lt3751.html ; datasheet rev D (3751fd) via https://static.chipdip.ru/lib/439/DOC011439824.pdf | read (mirror) |
| BSC070N10LS5 | https://www.infineon.com/cms/de/product/power/mosfet/n-channel/bsc070n10ls5/ | page |
| Sanken UX-F5B | https://datasheetspdf.com/datasheet/UX-F5B.html | page |
| SSDI SDA5000UF (alternative diode) | https://ssdi-power.com/sda5000uf.html | page |
| IXTH/IXTT10N100D2 | https://www.digikey.fr/en/products/detail/ixys/IXTH10N100D2/5810056 ; https://70ic.com/628584/IXTH10N100D2.pdf ; https://pdf.elecfans.com/IXYSCORPORATION/IXTH10N100D2.html | page (third-party datasheet copies) |
| MSC080SMA330B4N | https://www.microchip.com/en-us/product/msc080sma330b4n-mosfet (via research notes hv_safety_isolation §2) | page |
| G2R120MT33J | https://www.farnell.com/datasheets/3189227.pdf (via research notes miniaturization_hardware) | page |
| UCC21750 (rejected option) | https://www.ti.com/product/UCC21750 (via research notes) | page |
| Murata MGJ2/MGJ3/MGJ6 | research notes hv_safety_isolation §2 (Murata pages) | page |
| Cynergy3 D-HR reed relays | https://docs.rs-online.com/8c09/0900766b816395c1.pdf (via research notes §3) | read (in research notes) |
| Gigavac G17 | https://www.electronicsdatasheets.com/download/52f9f69ce34e245450a49294.pdf | page (search extract) |
| TT Electronics DPCR | https://www.mouser.lt/pdfDocs/DPCRDatasheet.pdf ; https://ttelectronics.com/TTElectronics/media/ProductFiles/Product-Change-Notification/PCNs-for-DSC.pdf | page |
| ADS1292R | https://www.ti.com/product/ADS1292R | page (parts study) |
| MAX30009 | https://www.analog.com/en/products/max30009.html ; https://www.mouser.in/en/new/analog-devices/maxim-max30009-afe/ | page |
| STM32U585 | https://www.st.com/en/microcontrollers-microprocessors/stm32u585ri.html (family page; VI variant chosen) | page |
| MCX A153 | https://www.nxp.com/part/MCXA153VLH ; NXP datasheet MCXAP64M96FS3 via https://www.mouser.com/datasheet/2/302/MCXAP64M96FS3-3440010.pdf | page (search extract) |
| MAX17320, BQ25887 | parts study §5 and its sources | page |
| HV resistors (HVCR, TT dividers) | https://www.digikey.com/en/product-highlight/s/stackpole-electronics/hvcr-series-resistors ; https://www.ttelectronics.com/blog/rethinking-high-voltage-resistive-division/ | page (parts study sources) |
| Defibrillation-proof test, spacing, bleeder arithmetic, Physio-Control bridge | research_notes/Open source belt worn AED/hv_safety_isolation.md §1–§4, §7 | read |
| Sharp memory LCD | https://www.mouser.com/pdfDocs/Sharp_Memory_LCD_Brochure_2015746065.PDF | page (parts study) |
| Firmware hardware contract | firmware/crates/aed-core/src/{hal,interlock,config,heat,standby,schedule,selftest,link}.rs; firmware/crates/aed-waveform/DESIGN_TABLE.md; firmware/crates/aed-cpr/src/settings.rs; firmware/crates/aed-sim/src/plant.rs | read |
