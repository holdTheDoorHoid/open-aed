# WP-B capture notes — controllers (sheets 04 and 05)

> **NOT FOR CONSTRUCTION — unreviewed design.** These notes and the pin tables they describe were written with AI
> assistance and have not been reviewed by a qualified electronics or medical-device engineer. Nothing here tells
> anyone to build, power or program anything (SAFETY.md §2a, CLAUDE.md rule 2). Nothing here is safe or ready for use
> on a person or animal.

Captured 27 November 2026 against contract issue 2 and `docs/HARDWARE_DESIGN.md` v0.2. Pin tables:
`hardware/aed-main/specs/04_main_mcu.toml`, `hardware/aed-main/specs/05_supervisor_mcu.toml`. Custom symbols:
`hardware/lib/src/aed_mcu.toml` (MCXA153VLH, CY15B104QN-50SXI). Build: `bash tools/build.sh` passes with zero ERC
errors; every accepted ERC finding is listed with its reason in the two specs (§13 below).

Tags as in the design document: **[CALC]** arithmetic shown, **[EST]** estimate with its basis, **[UNVERIFIED]**
looked for and not confirmed.

Revised 27 November 2026 to design revision 0.3 / contract issue 3: see the next section. Sections 1–13 below
describe the issue-2 capture; where they disagree with "Revision 0.3", the revision section wins.

---

## Revision 0.3 (contract issue 3, change order `rev3-changes.md` section WP-B)

### In plain language

Revision 0.3 gives the two controllers five new wires and nothing else changes on these two sheets.

- The **main controller** gets one new output, `K_SUPPLY_EN`, which switches on the patient relay's coil supply a
  fraction of a second before the relay is needed (so the battery is not hit with a 9 A surge). It also gets two new
  inputs, `HOLD_RB_A2_N` and `HOLD_RB_B2_N`, so that its self-test can read the new second "held" sensor on each of
  the two capacitor-emptying paths separately. Each new input has a resistor that makes a broken or unconnected wire
  read "not held".
- The **supervisor** gets two new outputs to the over-voltage alarm, which now stays tripped until it is reset:
  `OVP_RST` (a short pulse resets it) and `OVP_TEST` (holding it high forces a trip, so the self-test can prove the
  alarm works at low voltage).
- One thing the change order did not ask for: the reset wire feeds a capacitor on the other sheet, and switching a
  bare controller pin straight into a capacitor can briefly draw more current than the pin is rated for. I put a
  small 220 Ω resistor in series on the supervisor sheet. The wire's name, direction and meaning are unchanged, but
  the reset pulse now has to be at least 0.1 ms long instead of 10 µs. The design authority should confirm (P-1).
- The shock-current reading keeps its filter; only its scale (24.0 mV per amp) is written on the sheet.

Every new pin was checked in the controller datasheets: what it can do, which supply it belongs to, and what it does
while the chip is held in reset (it floats, so the resistor on the far sheet holds the line in its safe state).

### Changes applied

| # | Sheet | What | Designators and pins |
|---|---|---|---|
| B-1 | 04 | New output `K_SUPPLY_EN` (push-pull GPIO on 3V3_MAIN; default 100 kΩ to GND is on 14) | **U401 pin 123, PD7** |
| B-2 | 04 | New inputs `HOLD_RB_A2_N`, `HOLD_RB_B2_N` with 100 kΩ pull-ups to 3V3_MAIN | **U401 pin 56, PG0** and **pin 57, PG1**; **R431** (`HOLD_RB_A2_N`–3V3_MAIN), **R432** (`HOLD_RB_B2_N`–3V3_MAIN), Yageo RC0603FR-07100KL |
| B-3 | 05 | New outputs `OVP_RST`, `OVP_TEST` (push-pull GPIO on 3V3_SUP; defaults on 08) | `OVP_TEST`: **U501 pin 37, P3_13**. `OVP_RST`: **U501 pin 38, P3_12** → local `OVP_RST_S` → **R518 220 Ω** (Yageo RC0603FR-07220RL, as R513) → `OVP_RST` (departure, P-1) |
| B-4 | 04, 05 | Pin totals updated (below) | — |
| B-5 | 04 | `I_SHOCK` RC (R423 1 kΩ, C432 4.7 nF) unchanged; group note now states 24.0 mV/A, ≤ 3.20 V saturated, full scale 3.20 / 0.024 = 133 A [CALC] | R423, C432 |
| B-6 | 05 | Piezo LS501 unchanged (Q-55 open) | LS501 |
| — | 04, 05 | Header and `source` lines now cite design v0.3 and contract issue 3; one accepted ERC warning per sheet for the new labels whose far end is not yet drawn (below) | — |

The three new main-sheet nets and the two supervisor nets are added to each sheet's `global` list under exactly the
contract names. No net was renamed. `hardware/lib/src/aed_mcu.toml` needed no change (no new custom symbol).

### Pin checks against the datasheets

| Net (contract need) | Pin | Datasheet facts | Fits? |
|---|---|---|---|
| `K_SUPPLY_EN`: 3.3 V push-pull out; low = safe; must not drive during reset | PD7 (123) | DS13086 Table 27: I/O, **FT_h**, alternate functions incl. USART2_CK, OCTOSPIM_P1_IO7, SDMMC2_CMD, FMC_NE1, LPTIM4_OUT; **no "(boot)" marking**, so the bootloader never drives it. Reset: analog mode, high-Z, no pull (§3.13) → R1421 100 kΩ on 14 holds it low = coil supply off. Load on 14: R1403 1 kΩ into a FET gate plus 2 × 100 kΩ, ≤ 3.3 mA peak [CALC] | yes |
| `HOLD_RB_A2_N`, `HOLD_RB_B2_N`: inputs from open-drain outputs on 09, pulled up here; open = not held | PG0 (56), PG1 (57) | Table 27: I/O, **FT_ha** (ADC4_IN7/IN8 as extra functions), no boot role. power-supply section: "VDDIO2 is the external power supply for 14 I/Os (port G[15:2])", so **PG0/PG1 are on VDD**, not VDDIO2 (irrelevant here because VDDIO2 is tied to 3V3_MAIN, but it means these pins work even if VDDIO2 were separated). Reset: analog, high-Z → R431/R432 read "not held". Line never exceeds 3V3_MAIN (pull-up there), so FT tolerance is not relied on. EXTI0/EXTI1 are shared with PA0/PB1, so these two are polled inputs, not interrupts (the self-test reads them) | yes |
| `OVP_TEST`: 3.3 V push-pull level; low = no injection | P3_13 (37) | NXP Table 41: ALT0 GPIO (also LPUART2_CTS_B, CT1_MAT3, PWM0_X1), IO supply VDD, pad SLOW, **Default DIS** (input buffer disabled), no ISP function. Not 5 V tolerant (datasheet note: only P3_27, P3_28 are 5 V tolerant); the line is 3V3_SUP-only. 100 kΩ on 08 holds it low during reset. Load 22 kΩ + diode ≤ 0.12 mA [CALC] | yes |
| `OVP_RST`: rising edge = reset; low at rest; edge ≥ 10 µs per contract §10 | P3_12 (38) | Table 41: ALT0 GPIO (also LPUART2_RTS_B, CT1_MAT2, PWM0_X0), IO supply VDD, pad SLOW, **Default DIS**, no ISP function. Table 8: single digital output pin limit **±25 mA**. Table 11: VOL ≤ 0.5 V at 4 mA (normal drive). Driving 100 nF directly → see P-1; fitted R518 | yes, with R518 |

"Default DIS" says the input buffer is off; that the output driver is also off from reset (port direction = input)
is the usual GPIO reset value but comes from the reference manual, which I did not open **[UNVERIFIED]**. The
safe state does not depend on it being driven: every one of the four new outputs has its 100 kΩ pull-down on the far
sheet. The STM32's "all GPIOs in analog mode after reset" is DS13086 §3.13 (read for the issue-2 capture).

Not used, and why: P1_8/P1_9 on the supervisor are listed in Table 41 with ISP I²C functions; a table note says the
SPI and I²C ISP interfaces are "not available in MCXA14x and MCXA15x", so they would have been usable, but P3_12/P3_13
sit next to the other sheet-08 pins (P3_9–P3_11). (The same note means the existing `SBZ_*` pins P1_0–P1_3, listed
with ISP SPI functions, have no boot role on the A153 either.) On the main, PE8/PE10/PE12 were avoided because they
are TIM1 complementary outputs next to the phase commands; the bootloader pins (PA9/PA10, PC10/PC11, PB12–PB15 …)
stay unused.

### Datasheets consulted for this revision

| Document | Revision | Status |
|---|---|---|
| ST DS13086, STM32U585xx datasheet | **Rev 10, July 2024** | opened (local copy of the PDF fetched for the issue-2 capture; Table 27 rows for PD7, PG0, PG1 and the VDDIO2 supply paragraph re-read for this revision). A fresh fetch from st.com timed out twice |
| NXP MCXAP064M096F30, MCX A13x/14x/15x data sheet | **Rev. 5.2, 26 May 2026** | opened (fetched again; Table 41 rows P3_12, P3_13, P1_8, P1_9 and notes; Table 8; Table 11) |
| NXP MCX A reference manual (GPIO/PORT reset values) | — | **not opened [UNVERIFIED]** |
| TI SN74LVC2G07 (U935 on 09, drives `HOLD_RB_*2_N`) | — | not opened by me; WP-D's part. Open-drain output, so a pull-up to 3V3_MAIN only is the contract's arrangement |

### Pin use after revision 0.3

| | Main U401 STM32U585ZIT6 | Supervisor U501 MCXA153VLH |
|---|---:|---:|
| Contract signal pins | 70 + 3 = **73** | 22 + 2 = **24** |
| Other port pins (SWD 3, crystals, boot strap; supervisor also RESET_B) | 8 | 7 |
| Port pins used / available | **81 of 114** | **31 of 52** |
| Spare | 33 | 21 |
| Parts on the sheet | 76 (+R431, R432) | 40 (+R518) |

The change order (B-4) gives "73 + 6 = 79 of 114" and "24 + 4 = 28 of 52"; those use the design document's own
count of non-signal pins (§B2.2). Counting every port pin that is not left unconnected gives the figures above; both
agree on the signal-pin counts.

Spare main pins: PC13, PC2, PC3, PA1–PA8, PE8, PE10, PE12, PB10, PB12–PB15, PA9–PA12, PA15, PC10, PC11, PD2, PB4–PB9
(several are bootloader or debug pins: see §4). Spare supervisor pins: P1_7–P1_9, P2_1, P2_2, P2_7, P2_12, P2_13,
P2_16, P3_27, P3_28, P3_30, P3_31, P3_8, P3_1, P0_3, P0_6, P0_16, P0_17, P1_5, P1_6.

### Problems found with the change order

- **P-1 (`OVP_RST` drive current; departure, needs the design authority).** On 08 (rev3 C-3) `OVP_RST` drives
  C812 100 nF directly, AC-coupled into the BSS138 gate. With nothing in series, the first instant of each edge is
  limited only by the pin's output resistance: Table 11 guarantees VOL ≤ 0.5 V at 4 mA (≈ 125 Ω at that point, and
  typically lower), which allows a peak above the **25 mA single-pin limit** of Table 8 [CALC: 3.3 V / 125 Ω = 26 mA,
  more for a stronger pin]. I added **R518 220 Ω** on 05 between the pin and the `OVP_RST` label: peak ≤ 15 mA
  [CALC]. Cost: the edge into C812 now has τ = 22 µs, so a reset pulse must be **≥ 100 µs** (4.5 τ, gate ≥ 3.2 V;
  BSS138 V_GS(th) 0.8–1.5 V [BK, onsemi datasheet not re-read by me]) instead of the contract §10 "pulse ≥ 10 µs". FW-17 should state a
  pulse of ≥ 1 ms [EST: ample margin, still ≪ the 10 ms R809·C812 decay]. Net name, direction, level and default are
  unchanged. Alternative the design authority may prefer: put the series resistor on 08 with WP-C.
- **P-2 (minor).** The change order specifies R431/R432 as 0402; I used 0603, the same part as the sensor-1 pull-ups
  R408/R409, so no new part number is introduced.
- **P-3 (to note).** "VDDA min 3.26 V" in B-5 is the simulation notes' figure for the 3.3 V regulator's low limit
  (LR §5.1), not a datasheet limit of the STM32; the ADC simply reads full scale above VREF+ = VDDA. The 3.20 V
  saturated level stays below it [SIM, LR §5.1].
- **Single-ended nets until the other sheets are revised (expected):** `K_SUPPLY_EN` (other end on 14),
  `OVP_RST`, `OVP_TEST` (08) appear on one sheet only and carry one accepted ERC `isolated_pin_label` warning each;
  `HOLD_RB_A2_N`, `HOLD_RB_B2_N` (09) are single-sheet in `tools/xcheck.py` but meet two pins here (U401 + pull-up),
  so ERC is silent. Remove the two `erc_accepted` entries once 08 and 14 are at revision 0.3.

### Still needs simulation or measurement

1. The `OVP_RST` edge through R518 into C812/R809 and the BSS138 (WP-C's S-10 netlist): minimum pulse length that
   clears the latch at −20 °C with the BSS138's highest threshold.
2. The pin's real short-circuit/edge current (not in the NXP datasheet; Table 11 gives only guaranteed points).
3. `K_SUPPLY_EN` timing ≥ 0.2 s before `RELAY_CMD` is a firmware rule (FW-12); nothing on 04 enforces it.

### Build

`bash tools/build.sh`: **BUILD OK** (ERC 0 errors, 24 warnings all accepted with reasons; netlist round trip 725
nets, 3,680 pin connections match). `python3 tools/xcheck.py` lists the five new nets as single-sheet, as expected.

---

## 1. What is on each sheet

**Sheet 04 — main controller.** The ST STM32U585ZIT6 (U401), the chip that runs the rescue: it listens to the heart
signal, charges the capacitor, asks for the shock and times its two phases. Around it: a 16 MHz crystal for its main
clock and the shock timer, a 32.768 kHz crystal for its own clock-keeping, a reset capacitor, a resistor that makes
it start from its own program memory, and six programming pads (nothing fitted). Two memories share one wire bundle:
a 16 MB flash chip for the event record and a 4 Mbit F-RAM for the small store that must survive every power loss.
Four small buffer chips let signals from parts that stay powered when the main controller is off (the always-on
wake lines and the supervisor's message line) reach the controller without leaking power into it. Thirty resistors
set the default state of every line the contract makes this sheet responsible for: "not held", "not armed", "no
fault", "relay not at rest" and so on whenever the far end is open, unpowered or unsoldered. Two small filters feed
the shock-current and pad-temperature readings into the analog inputs.

**Sheet 05 — supervisor controller.** The NXP MCXA153VLH (U501), a different chip from a different maker, powered
only from the supervisor's own regulator. It has its own 16 MHz crystal (from a different crystal maker than the
main's), its own programming pads, its own reset and start-up resistors, two buffers on the lines that come from
other parts (the main's message line and the over-voltage alarm), its own bias resistor for the capacitor
temperature sensor, the pull-ups the contract gives it, and its own alarm: a piezo sounder driven from both ends by a
two-channel buffer, and a red fault light. Nothing on this page is powered from the main controller's supply.

---

## 2. Parts

Datasheets opened on 27 Nov 2026 unless marked "not opened". Where a manufacturer site refused automated download the
source used is named.

| Ref | Part | Maker / MPN | Datasheet (opened version) |
|---|---|---|---|
| U401 | Main controller, LQFP144, LDO version, −40…+85 °C | ST STM32U585ZIT6 | https://www.st.com/resource/en/datasheet/stm32u585zi.pdf — DS13086 **Rev 10, July 2024**, opened (st.com; first attempts timed out, a browser user-agent worked). KiCad stock symbol `MCU_ST_STM32U5:STM32U585ZITx` used |
| U402 | 128 Mbit SPI NOR flash, SOIC-8 208 mil, −40…+85 °C | Winbond W25Q128JVSIQ | https://www.winbond.com/resource-files/W25Q128JV%20RevM%2012242024%20Plus.pdf — **Rev M, 24 Dec 2024**, opened. Stock symbol `Memory_Flash:W25Q128JVS` |
| U403 | 4 Mbit SPI F-RAM, 8-SOIC 208 mil (EIAJ), −40…+85 °C | Infineon CY15B104QN-50SXI | Infineon 002-19436 **Rev \*O, 2023-06-05** (file name v12_00), opened (infineon.com). Custom symbol |
| U404–U407, U502, U503 | Single Schmitt buffer with Ioff, SOT-23-5, −40…+125 °C | TI SN74LVC1G17DBVR | https://www.ti.com/lit/ds/symlink/sn74lvc1g17.pdf — **SCES351Y, Oct 2025**, opened |
| U504 | Dual Schmitt buffer with Ioff, ±24 mA, SOT-23-6, −40…+125 °C | TI SN74LVC2G17DBVR | https://www.ti.com/lit/ds/symlink/sn74lvc2g17.pdf — **SCES381O, Nov 2026**, opened |
| U501 | Supervisor controller, LQFP64, TA −40…+125 °C | NXP MCXA153VLH | https://www.nxp.com/docs/en/data-sheet/MCXAP064M096F30.pdf — **Rev 5.2, 26 May 2026**, opened (nxp.com refused curl; fetched through the web-fetch tool, which saved the PDF). Custom symbol |
| Y401 | 16 MHz crystal, CL 10 pF, ESR ≤ 70 Ω, C0 ≤ 3 pF, ±10 ppm / ±20 ppm −40…+85 °C, 3.2 × 2.5 mm | Abracon ABM8-16.000MHZ-10-D1X-T | https://abracon.com/Resonators/abm8.pdf — revised 07-29-20, opened. Ordering code **built from the maker's option guide** |
| Y402 | 32.768 kHz crystal, CL 7 pF, ESR ≤ 70 kΩ, −40…+85 °C, 3.2 × 1.5 mm | Abracon ABS07-32.768KHZ-7-T | https://abracon.com/Resonators/ABS07.pdf — revised 08-10-22, opened |
| Y501 | 16 MHz crystal, CL 10 pF, ESR ≤ 80 Ω, C0 ≤ 5 pF, ±25 ppm / ±30 ppm −40…+85 °C, 3.2 × 2.5 mm | ECS ECS-160-10-33-AGN-TR (ECX-32) | https://ecsxtal.com/store/pdf/ECX-32.pdf — Rev. 2017, opened. Ordering code **built from the maker's numbering guide**; availability not confirmed |
| LS501 | Piezo sounder 4 kHz, 12 × 12 mm SMD (**provisional, WP-C's choice**) | Murata PKLCS1212E4001-R1 | https://www.murata.com/en-us/products/productdetail?partno=PKLCS1212E4001-R1 — **not opened** (Murata's PDF endpoint returned HTML) |
| D501 | Red LED 0805 | Kingbright APT2012SURCK | https://www.kingbrightusa.com/images/catalog/SPEC/APT2012SURCK.pdf — **not opened** |
| FB401, FB501 | Ferrite bead 220 Ω @ 100 MHz, 0603 | Murata BLM18PG221SN1D | Murata product page — **not opened** |
| J401, J501 | SWD pads for Tag-Connect TC2030-IDC-NL (footprint only, DNP) | Tag-Connect | **not opened**; pinout = KiCad stock symbol |
| R (1 %) | Thick film 0603 | Yageo RC0603FR-07…L | https://www.yageo.com/upload/media/product/products/datasheet/rchip/PYu-RC_Group_51_RoHS_L_12.pdf — **not opened** (generic series) |
| R424, R506 | 10.0 kΩ 0.1 % 25 ppm/K thin film 0603 | Panasonic ERA-3AEB103V | Panasonic ERA-xA series PDF — **not opened** |
| C | MLCC: 100 nF 50 V X7R 0603, 1 µF 16 V X7R 0603, 4.7 µF 10 V X7R 0805 (VDDIO2), 4.7 µF 16 V X7R 1206 (VCAP), 10 µF 10 V X7R 0805, C0G 0402 (4.7 pF, 10 pF, 12 pF, 4.7 nF), 10 nF X7R 0402 | Murata GRM series | Murata product pages — **not opened**; the MPN suffixes of the 4.7 µF, 10 µF and C0G values are built from the GRM numbering scheme and carry a `Verify` field |

**Machine-readable aid (not a manufacturer source):** the alternate-function map of the STM32U585ZI was first pulled
from the embassy-rs `stm32-data-generated` JSON (derived from ST's CubeMX database), then **every alternate function
used was checked by hand against DS13086 Tables 28/29**, and every pin number and name of the stock KiCad symbol was
checked by script against DS13086 Table 27 (135 of 144 rows matched automatically; the other 9 — PC14, PC15, PH0,
PH1, VREF−, VREF+, PA13, PA14, PB3 — have wrapped names in the PDF text and were checked by eye).

---

## 3. Available pins: verified against the exact packages

| | Design §B2 said | Verified | Used here | Spare |
|---|---|---|---|---|
| STM32U585ZIT6 (LQFP144, no SMPS) | "about 110 I/O" [UNVERIFIED] | **114 GPIOs** (DS13086 Table 2 "GPIOs (without SMPS / with SMPS)" 114/111; KiCad symbol and Table 27 agree: 144 pins − 9 VDD − 11 VSS − VBAT − VDDA − VSSA − VREF+ − VREF− − VCAP − 2 VDDIO2 − VDDUSB − NRST = 114) | **78** port pins: 70 signal pins + SWDIO, SWCLK, SWO + 2 HSE + 2 LSE + PH3-BOOT0 (strap); plus NRST | **36** (incl. PA15/PB4 with debug pull-ups and PC13 backup-domain pin) |
| MCXA153VLH (LQFP64) | "about 50 I/O" [UNVERIFIED] | **52 GPIO-capable pins** (NXP Table 1 ordering information: 52; Table 41 lists 52 port pins: 64 − 4 VDD − 4 VSS − VDD_ANA − VDD_USB − USB0_DM − USB0_DP). Of these, RESET_B, the two crystal pins and the two SWD pins are used for their reset functions, leaving 47 general pins (P3_29 must be high at reset: ISP strap) | **29** port pins: 22 signal pins + SWDIO, SWCLK, SWO + RESET_B + 2 crystal + ISPMODE_N (strap) | **23** GPIO (+ USB0_DP/DM unused) |

The design's own sum in §B2.2 is off: its group rows add up to **70** signal pins, so the total is 70 + 6 = **76**,
not "76 + 6 = 82" (contract issue C-1). The supervisor's 22 + 4 = 26 is correct.

---

## 4. Main controller pin assignment (U401 STM32U585ZIT6)

Pin numbers: DS13086 Table 27, LQFP144 (non-SMPS) column. Mux: alternate-function number from Tables 28/29 (`-` =
plain GPIO, `analog` = analog function from Table 27's additional-functions column). Reset state: DS13086 §3.13
("after reset, all GPIOs are in analog mode") and Table 27 note 5 (debug pins).

| Pin | Name | Function used | Mux | Net | Reset state | Note |
|---:|---|---|---|---|---|---|
| 1 | PE2 | GPIO out | - | `AMP_SD_N` | analog, high-Z, no pull | 100k to GND on 06 |
| 2 | PE3 | GPIO in, EXTI3 | - | `CHGR_INT_N` | analog, high-Z, no pull | 100k up here |
| 3 | PE4 | SAI1_FS_A | AF13 | `I2S_LRCLK` | analog, high-Z, no pull | MAX98357A (06, 4.7k series there) |
| 4 | PE5 | SAI1_SCK_A | AF13 | `I2S_BCLK` | analog, high-Z, no pull | MAX98357A (06) |
| 5 | PE6 | SAI1_SD_A | AF13 | `I2S_SD` | analog, high-Z, no pull | MAX98357A (06) |
| 8 | PC14 | OSC32_IN | analog | `LSE_IN` | analog (oscillator off until enabled) | Y402 |
| 9 | PC15 | OSC32_OUT | analog | `LSE_OUT` | analog (oscillator off until enabled) | Y402 |
| 10 | PF0 | GPIO out | - | `CHG_SESSION` | analog, high-Z, no pull | 100k to GND on 09 |
| 11 | PF1 | GPIO out | - | `CHG_RUN` | analog, high-Z, no pull | 100k to GND on 09 |
| 12 | PF2 | GPIO in | - | `CHG_DONE_N` | analog, high-Z, no pull | 100k up here |
| 13 | PF3 | GPIO in | - | `CHG_FAULT_N` | analog, high-Z, no pull | 100k up here |
| 14 | PF4 | GPIO in | - | `MODE_HW_RB_N` | analog, high-Z, no pull | 100k up here |
| 15 | PF5 | GPIO in | - | `HOLD_RB_A_N` | analog, high-Z, no pull | 100k up here |
| 18 | PF6 | GPIO in | - | `HOLD_RB_B_N` | analog, high-Z, no pull | 100k up here |
| 19 | PF7 | GPIO in | - | `MODE_CHILD_N` | analog, high-Z, no pull | 10k up here |
| 20 | PF8 | GPIO in, EXTI8 | - | `SEQ_ACT_M_N` | analog, high-Z, no pull | 100k up here |
| 21 | PF9 | GPIO in, EXTI9 | - | `SEQ_DONE_M_N` | analog, high-Z, no pull | 100k up here |
| 22 | PF10 | GPIO in, EXTI10 | - | `K_REST_N` | analog, high-Z, no pull | 100k up here |
| 23 | PH0 | OSC_IN | analog | `HSE_IN` | analog (oscillator off until enabled) | Y401 |
| 24 | PH1 | OSC_OUT | analog | `HSE_OUT` | analog (oscillator off until enabled) | Y401 |
| 25 | NRST | reset | - | `NRST_M` | input, 30-50 kOhm pull-up | 100 nF (C421) + SWD |
| 26 | PC0 | ADC1_IN1 (fast channel) | analog | `ISHOCK_ADC` | analog, high-Z, no pull | I_SHOCK after R423/C432 anti-alias |
| 27 | PC1 | ADC1_IN2 (fast channel) | analog | `NTC_PADS_ADC` | analog, high-Z, no pull | NTC_PADS after R425/C433 |
| 34 | PA0 | GPIO in, EXTI0 / WKUP1 | - | `RTC_INT_M_N` | analog, high-Z, no pull | from U406 (Ioff receiver of RTC_INT_N) |
| 44 | PC4 | GPIO in, EXTI4 | - | `HVM_DRDY_N` | analog, high-Z, no pull | ADS1220 DRDY (07) |
| 45 | PC5 | GPIO in, EXTI5 / WKUP5 | - | `DOCK_DET_M_N` | analog, high-Z, no pull | from U407 (Ioff receiver of DOCK_DET_N) |
| 46 | PB0 | GPIO out | - | `CHGR_CE_N` | analog, high-Z, no pull | contract 5.2; default from 100k to GND on 02 |
| 47 | PB1 | GPIO in, EXTI1 | - | `GAUGE_ALRT_N` | analog, high-Z, no pull | contract 5.2; 100k up here |
| 48 | PB2 | GPIO in, EXTI2 / WKUP1 (alt.) | - | `PWR_BTN_M_N` | analog, high-Z, no pull | from U405 (Ioff receiver of PWR_BTN_N) |
| 49 | PF11 | GPIO in, EXTI11 | - | `BRIDGE_ARMED_N` | analog, high-Z, no pull | 100k up here |
| 50 | PF12 | GPIO in, EXTI12 | - | `PAD_PRESENT_N` | analog, high-Z, no pull | 100k up here |
| 53 | PF13 | GPIO in | - | `MODE_ADULT_N` | analog, high-Z, no pull | 10k up here |
| 54 | PF14 | I2C4_SCL | AF4 | `I2C_M_SCL` | analog, high-Z, no pull | 4.7k up here |
| 55 | PF15 | I2C4_SDA | AF4 | `I2C_M_SDA` | analog, high-Z, no pull | 4.7k up here |
| 58 | PE7 | GPIO in, EXTI7 / WKUP6 | - | `CANCEL_M_N` | analog, high-Z, no pull | 10k up here |
| 60 | PE9 | TIM1_CH1 | AF1 | `CMD_HA` | analog, high-Z, no pull | advanced timer TIM1; 100k to GND on 12 |
| 64 | PE11 | TIM1_CH2 | AF1 | `CMD_LB` | analog, high-Z, no pull | phase 1 low side |
| 66 | PE13 | TIM1_CH3 | AF1 | `CMD_HB` | analog, high-Z, no pull |  |
| 67 | PE14 | TIM1_CH4 | AF1 | `CMD_LA` | analog, high-Z, no pull | phase 2 low side |
| 68 | PE15 | TIM1_BKIN (break input, active low) + EXTI15 | AF1 | `BR_FAULT_N` | analog, high-Z, no pull | forces TIM1 outputs inactive on a sequencer fault |
| 77 | PD8 | USART3_TX (synchronous master) | AF7 | `LCD_SI` | analog, high-Z, no pull | memory LCD data (06) |
| 78 | PD9 | GPIO out | - | `LCD_SCS` | analog, high-Z, no pull | memory LCD chip select, active high (06; 1M to GND there) |
| 79 | PD10 | USART3_CK (synchronous master) | AF7 | `LCD_SCLK` | analog, high-Z, no pull | memory LCD clock (06) |
| 80 | PD11 | GPIO out | - | `LCD_BLANK` | analog, high-Z, no pull | high = blank (06; 1M to GND there) |
| 81 | PD12 | GPIO out toggled by the control loop (TIM4_CH1 available for one-pulse mode) | AF2 optional | `HOLD_M_CLK` | analog, high-Z, no pull | F-1: never free-running |
| 82 | PD13 | GPIO out | - | `SHOCK_REQ` | analog, high-Z, no pull | 100k to GND on 09 |
| 85 | PD14 | GPIO out | - | `RELAY_CMD` | analog, high-Z, no pull | 100k to GND on 12 |
| 86 | PD15 | GPIO out | - | `BTEST_CMD` | analog, high-Z, no pull | 100k to GND on 12 |
| 87 | PG2 | SPI1_SCK | AF5 | `AFE_SCLK` | analog, high-Z, no pull | ADS1292R (15) |
| 88 | PG3 | SPI1_MISO | AF5 | `AFE_MISO` | analog, high-Z, no pull |  |
| 89 | PG4 | SPI1_MOSI | AF5 | `AFE_MOSI` | analog, high-Z, no pull |  |
| 90 | PG5 | GPIO out (software chip select) | - | `AFE_CS_N` | analog, high-Z, no pull | 100k up on 15 |
| 91 | PG6 | GPIO in, EXTI6 | - | `AFE_DRDY_N` | analog, high-Z, no pull |  |
| 92 | PG7 | LPUART1_TX | AF8 | `DOCK_TX` | analog, high-Z, no pull | 100k up here |
| 93 | PG8 | LPUART1_RX | AF8 | `DOCK_RX` | analog, high-Z, no pull | from Ioff buffer on 02 |
| 96 | PC6 | GPIO out | - | `EN_5V0` | analog, high-Z, no pull | contract 5.1; 100k to GND on 03 (5 V off) |
| 97 | PC7 | GPIO out | - | `BAT_LOAD_EN` | analog, high-Z, no pull | contract 5.2; 100k to GND on 01 |
| 98 | PC8 | GPIO out | - | `MAIN_KEEP` | analog, high-Z, no pull | contract 5.1; 100k to GND on 03 |
| 99 | PC9 | GPIO out | - | `SUP_WAKE` | analog, high-Z, no pull | contract 5.1; 100k to GND on 03 |
| 105 | PA13 | SWDIO (JTMS/SWDIO) | AF0 | `SWDIO_M` | AF0 SWDIO, internal pull-up | debug; reset state AF0 with internal pull-up |
| 109 | PA14 | SWCLK (JTCK/SWCLK) | AF0 | `SWCLK_M` | AF0 SWCLK, internal pull-down | debug; reset state AF0 with internal pull-down |
| 113 | PC12 | UART5_TX half-duplex open drain (1-Wire) | AF8 | `PAD_ID_1W` | analog, high-Z, no pull | contract 5.8; 2.2k up here |
| 114 | PD0 | GPIO out (SPI2 chip select by software) | - | `HVM_CS_N` | analog, high-Z, no pull | 100k up on 07 |
| 115 | PD1 | SPI2_SCK | AF5 | `HVM_SCLK` | analog, high-Z, no pull | ADS1220 (07) |
| 117 | PD3 | SPI2_MISO | AF5 | `HVM_MISO` | analog, high-Z, no pull | ADS1220 DOUT (07) |
| 118 | PD4 | SPI2_MOSI | AF5 | `HVM_MOSI` | analog, high-Z, no pull | ADS1220 DIN (07) |
| 119 | PD5 | USART2_TX | AF7 | `LINK_M2S` | analog, high-Z, no pull | link 1 Mbit/s to 05 |
| 122 | PD6 | USART2_RX | AF7 | `LINK_S2M_RX` | analog, high-Z, no pull | from U404 (Ioff Schmitt receiver of LINK_S2M) |
| 124 | PG9 | SPI3_SCK | AF6 | `MEM_SCK` | analog, high-Z, no pull | flash + F-RAM |
| 125 | PG10 | SPI3_MISO | AF6 | `MEM_MISO` | analog, high-Z, no pull |  |
| 126 | PG11 | SPI3_MOSI | AF6 | `MEM_MOSI` | analog, high-Z, no pull |  |
| 127 | PG12 | GPIO out | - | `FLASH_CS_N` | analog, high-Z, no pull | 10k up (R427) |
| 128 | PG13 | GPIO out | - | `FRAM_CS_N` | analog, high-Z, no pull | 10k up (R428) |
| 129 | PG14 | GPIO out | - | `AFE_START` | analog, high-Z, no pull | 100k down on 15 |
| 132 | PG15 | GPIO out | - | `AFE_RESET_N` | analog, high-Z, no pull | 100k down on 15 |
| 133 | PB3 | TRACESWO (JTDO) | AF0 | `SWO_M` | AF0 JTDO/SWO (no pull) | debug trace out |
| 138 | PH3 | BOOT0 (sampled at reset) | - | `BOOT0_M` | BOOT0 sampled at reset; analog afterwards | 10k to GND (R426) |
| 141 | PE0 | GPIO out | - | `READY_D` | analog, high-Z, no pull | 1M to GND on 03 |
| 142 | PE1 | GPIO out | - | `READY_CLK` | analog, high-Z, no pull | 1M to GND on 03 |

Unused (NC): PC13 (7), PC2 (28), PC3 (29), PA1 (35), PA2 (36), PA3 (37), PA4 (40), PA5 (41), PA6 (42), PA7 (43), PG0 (56), PG1 (57), PE8 (59), PE10 (63), PE12 (65), PB10 (69), PB12 (73), PB13 (74), PB14 (75), PB15 (76), PA8 (100), PA9 (101), PA10 (102), PA11 (103), PA12 (104), PA15 (110), PC10 (111), PC11 (112), PD2 (116), PD7 (123), PB4 (134), PB5 (135), PB6 (136), PB7 (137), PB8 (139), PB9 (140)

**Choices that matter for safety:**
- **Phase commands on one advanced timer:** TIM1 CH1–CH4 on PE9/PE11/PE13/PE14, all AF1 (DS13086 Table 28). Phase
  1 = CH1 (`CMD_HA`) + CH2 (`CMD_LB`); phase 2 = CH3 (`CMD_HB`) + CH4 (`CMD_LA`). Resolution [CALC]: TIM1 is 16-bit
  on a 160 MHz kernel clock; with prescaler 64 (2.5 MHz, 0.4 µs per count) one counter period is 65,536 × 0.4 µs =
  26.2 ms, which covers the whole sequence (lead 0.1 ms + phase 1 ≤ 12 ms + gap 0.5 ms + phase 2, config ≤ 20 ms)
  with 0.4 µs steps — 25 times finer than the shortest hardware lockout (10 µs). (Phase-length limits from DECISIONS
  D-02/D-23 as quoted in design §0; config.rs ≤ 20 ms from §B13.3.)
- **`BR_FAULT_N` on TIM1's break input** (PE15, AF1): when the sequencer reports a fault latch, TIM1 forces its four
  outputs inactive in hardware without waiting for code. This costs no extra pin (the net had to reach the main
  anyway) and adds no contract change.
- **No safety output on a bootloader pin.** DS13086 Table 27 marks the system-bootloader pins "(boot)": USART1
  PA9/PA10, USART2 PA2/PA3, USART3 PC10/PC11, SPI1 PA4–PA7, SPI2 PB12–PB15, SPI3 PG9/PG10/PG12/PB5, I2C1 PB6/PB7,
  I2C2 PB10, I2C3 PC0/PC1, FDCAN1 PB8/PB9, USB PA11/PA12. If the chip ever started its bootloader (BOOT0 high), a
  UART transmit pin would idle **high**: none of the shock, consent, charger, relay or power-control outputs is on one
  of them. The only overlaps are inputs (`ISHOCK_ADC`, `NTC_PADS_ADC` on I2C3 open-drain pins) and the memory bus
  (SPI3 slave: its MISO drives only while `FLASH_CS_N` is low, which R427 prevents). The I²C bus was deliberately put
  on I2C4 (PF14/PF15), not the bootloader's FDCAN pins PB8/PB9.
- **UCPD dead-battery pull-downs** (Table 27 note 4: a high level on PB5/PB14 can switch 5.1 kΩ pull-downs onto
  PA15/PB15): none of these four pins is used.
- **Debug pins with reset pulls** (PA13 up, PA14 down, PA15 up, PB4 up, PB3): only SWD uses them; PA15 and PB4 are
  unused.
- **Consent clock** `HOLD_M_CLK` is a plain GPIO (PD12) that the control loop toggles (F-1). TIM4_CH1 is on the same
  pin if firmware prefers a one-pulse timer re-armed by the loop; never a free-running timer.
- **Wake-capable inputs:** `RTC_INT_N` (via U406 → PA0, WKUP1), `PWR_BTN_N` (via U405 → PB2), `DOCK_DET_N` (via U407
  → PC5, WKUP5), `CANCEL_M_N` (PE7, WKUP6). In this design the main is woken by its regulator being switched on
  (§B1.7), so wake pins only matter for Stop modes; each interrupt source has its own EXTI line number (0, 1, 2, 3,
  4, 5, 6, 7, 8, 9, 10, 11, 12, 15).
- **ADC inputs:** `I_SHOCK` → PC0 (ADC1_IN1), `NTC_PADS` → PC1 (ADC1_IN2): ADC1 fast channels (Table 27 additional
  functions). VREF+ is tied to VDDA (3.3 V) because `I_SHOCK` spans 0–3.2 V; the 2.5 V internal reference would clip it.
- **Buses that do not collide:** SPI1 = front end (PG2–PG4), SPI2 = voltage ADC (PD1/PD3/PD4), SPI3 = flash + F-RAM
  (PG9–PG11), USART3 synchronous = memory LCD (PD8/PD10), USART2 = link (PD5/PD6), LPUART1 = dock (PG7/PG8), UART5
  half-duplex = pad 1-Wire (PC12), I2C4 = gauge/charger/RTC (PF14/PF15), SAI1 block A = amplifier (PE4–PE6), TIM1 =
  phases, TIM4 (optional) = hold clock. Port G is on VDDIO2, tied to 3V3_MAIN.
- **5 V tolerance:** every signal on this sheet is a 3.3 V signal; the FT pins' tolerance is not relied on.

---

## 5. Supervisor pin assignment (U501 MCXA153VLH)

Pin numbers: NXP Table 41, LQFP64 column. Mux: ALT number from Table 41's "Pinmux Assignment" column. Reset state:
Table 41 "Default" column ("DIS" = input buffer disabled, no pull) and Table 42 (JTAG/SWD pins default to the debug
function with pull-up, SWCLK with pull-down).

| Pin | Name | Function used | Mux | Net | Reset state | Note |
|---:|---|---|---|---|---|---|
| 4 | P1_10 | ADC0_A8 | analog | `NTC_CAP_S_ADC` | input buffer disabled, no pull ("DIS") | NTC_CAP_S after R507/C512 |
| 5 | P1_11 | GPIO in, WUU0_IN11 | ALT0 | `OVP_TRIP_S` | input buffer disabled, no pull ("DIS") | from U503 (Ioff receiver of OVP_TRIP) |
| 6 | P1_12 | GPIO in, WUU0_IN12 | ALT0 | `CANCEL_S_N` | input buffer disabled, no pull ("DIS") | 10k up here |
| 7 | P1_13 | GPIO out | ALT0 | `SUP_KEEP` | input buffer disabled, no pull ("DIS") | 100k to GND on 03 |
| 8 | P1_29/RESET_B | RESET_B | ALT1 (reset default) | `RESET_S_N` | ALT1 RESET_B (RST pad) | 10k up + 10 nF + SWD |
| 9 | P1_30/XTAL48M | XTAL48M (system oscillator) | analog | `XTAL_S` | input buffer disabled (analog) | Y501 |
| 10 | P1_31/EXTAL48M | EXTAL48M (system oscillator) | analog | `EXTAL_S` | input buffer disabled (analog) | Y501 |
| 14 | P2_0 | GPIO in, WUU0_IN18 | ALT0 | `SEQ_ACT_S_N` | input buffer disabled, no pull ("DIS") | 100k up here |
| 17 | P2_3 | GPIO in, WUU0_IN19 | ALT0 | `SEQ_DONE_S_N` | input buffer disabled, no pull ("DIS") | 100k up here |
| 18 | P2_4 | GPIO out toggled by the control loop (CT1_MAT0 available) | ALT0 (ALT5) | `HOLD_S_CLK` | input buffer disabled, no pull ("DIS") | F-1; 100k to GND on 09 |
| 19 | P2_5 | GPIO out toggled by the control loop (CT1_MAT1 available) | ALT0 (ALT5) | `PERMIT_CLK` | input buffer disabled, no pull ("DIS") | F-1; 100k to GND on 09 |
| 20 | P2_6 | GPIO out toggled by the control loop (CT1_MAT2 available) | ALT0 (ALT5) | `TEST_PERMIT_CLK` | input buffer disabled, no pull ("DIS") | F-1; 100k to GND on 09 |
| 32 | P3_29/ISPMODE_N | ISPMODE_N (boot strap) | ALT1 (reset default) | `ISPMODE_S_N` | ALT1 ISPMODE_N, sampled at reset | 10k up (R503): boot from flash |
| 35 | P3_15 | LPUART2_TXD | ALT2 | `LINK_S2M` | input buffer disabled, no pull ("DIS") | link to 04 |
| 36 | P3_14 | LPUART2_RXD, WUU0_IN25 | ALT2 | `LINK_M2S_RX` | input buffer disabled, no pull ("DIS") | from U502 (Ioff Schmitt receiver of LINK_M2S) |
| 39 | P3_11 | GPIO out | ALT0 | `HVS_PDRST_N` | input buffer disabled, no pull ("DIS") | 100k to GND on 08 |
| 40 | P3_10 | LPSPI1_SCK | ALT2 | `HVS_SCLK` | input buffer disabled, no pull ("DIS") | AD7171 (08) |
| 41 | P3_9 | LPSPI1_SDI (GPIO interrupt for DOUT/RDY) | ALT2 | `HVS_DOUT` | input buffer disabled, no pull ("DIS") | AD7171 (08) |
| 43 | P3_7 | PWM0_B0 | ALT5 | `PIEZO_B_DRV` | input buffer disabled, no pull ("DIS") | to U504 channel 2 (complementary) |
| 44 | P3_6 | PWM0_A0 | ALT5 | `PIEZO_A_DRV` | input buffer disabled, no pull ("DIS") | to U504 channel 1 |
| 46 | P3_0 | GPIO out (high-drive pad) | ALT0 | `FAULT_LED_S` | input buffer disabled, no pull ("DIS") | red fault light via R513 |
| 49 | P0_0/SWDIO | SWDIO | ALT1 (reset default) | `SWDIO_S` | ALT1 SWDIO, pull-up | debug; reset default ALT1 with pull-up |
| 50 | P0_1/SWCLK | SWCLK | ALT1 (reset default) | `SWCLK_S` | ALT1 SWCLK, pull-down | debug; reset default ALT1 with pull-down |
| 51 | P0_2/SWO | SWO | ALT1 (reset default) | `SWO_S` | ALT1 TDO/SWO, pull-up | debug trace out |
| 56 | P1_0 | LPSPI0_SDO | ALT2 | `SBZ_MOSI` | input buffer disabled, no pull ("DIS") | MAX30009 (16) |
| 57 | P1_1 | LPSPI0_SCK | ALT2 | `SBZ_SCLK` | input buffer disabled, no pull ("DIS") |  |
| 58 | P1_2 | LPSPI0_SDI | ALT2 | `SBZ_MISO` | input buffer disabled, no pull ("DIS") |  |
| 59 | P1_3 | LPSPI0_PCS0 (or GPIO) | ALT2 | `SBZ_CS_N` | input buffer disabled, no pull ("DIS") | 100k up on 16 |
| 62 | P1_4 | GPIO in, WUU0_IN8 | ALT0 | `SBZ_INT_N` | input buffer disabled, no pull ("DIS") | 100k up here |

Unused (NC): P1_7 (1), P1_8 (2), P1_9 (3), P2_1 (15), P2_2 (16), P2_7 (21), P2_12 (22), P2_13 (23), P2_16 (24), USB0_DM (26), USB0_DP (27), P3_31 (30), P3_30 (31), P3_28 (33), P3_27 (34), P3_13 (37), P3_12 (38), P3_8 (42), P3_1 (45), P0_3 (52), P0_6 (53), P0_16 (54), P0_17 (55), P1_5 (63), P1_6 (64)

**Choices that matter for safety:**
- **Consent clocks** `HOLD_S_CLK`, `PERMIT_CLK`, `TEST_PERMIT_CLK` on P2_4/P2_5/P2_6: plain GPIOs toggled by the
  control loop (F-1); CTIMER1 match outputs 0/1/2 (ALT5) are on the same pins for a one-pulse variant. At reset the
  pins are high-impedance, so sheet 09's 100 kΩ pull-downs make them static low = no consent.
- **No safety output on a boot or debug pin.** The ISP-mode pin P3_29 is held high by R503; P0_2/P0_3 (the ROM's
  UART ISP pins) carry only SWO and nothing.
- **Wake-capable inputs (WUU0):** `SEQ_ACT_S_N` (IN18), `SEQ_DONE_S_N` (IN19), `OVP_TRIP` via U503 (IN11),
  `CANCEL_S_N` (IN12), `SBZ_INT_N` (IN8), link receiver (IN25).
- **Peripheral instances:** LPSPI0 = pad-impedance chip on 16 (P1_0–P1_3), LPSPI1 = AD7171 on 08 (P3_9 SDI, P3_10
  SCK; the AD7171's DOUT/RDY falling edge is caught as a GPIO interrupt on the same pin before the read), LPUART2 =
  link (P3_14/P3_15), FlexPWM0 A0/B0 = sounder (P3_6/P3_7), ADC0 channel A8 = NTC (P1_10).
- **Not 5 V tolerant:** VDIO max = VDD + 0.3 V (Table 8); only P3_27/P3_28 are 5 V tolerant and they are unused.
  Every input from another domain enters through an Ioff buffer (U502, U503) or is an open-drain/contact line pulled
  up to 3V3_SUP on this sheet.

---

## 6. How neither controller can power the other (back-powering)

The two controllers sit on separate regulators (`3V3_MAIN`, `3V3_SUP`), and either may be off while the other runs
(§B2.7). A powered chip must not feed the other's rail through an input pin's protection diode. On these two sheets:

| Path | Powered side | Unpowered side | Why no current flows into the dead rail |
|---|---|---|---|
| `LINK_M2S` | main TX (PD5) idles high | supervisor off | The line ends on sheet 05 in R504 (100 kΩ to GND) and R505 (1 kΩ) into U502, an SN74LVC1G17 on 3V3_SUP whose input is specified for partial power-down (Ioff ≤ ±10 µA at VCC = 0, SCES351Y). U501's pin is behind U502's output, which Ioff also makes high-impedance. Load on the main: 33 µA into R504 |
| `LINK_M2S` | supervisor on | main off | R504 holds the line low = continuous break; nothing on 05 drives the line |
| `LINK_S2M` | supervisor TX (P3_15) | main off | Mirror image: R406 + R407 + U404 on 3V3_MAIN |
| `LINK_S2M` | main on | supervisor off | R406 holds it low; U501's pin sees 0 V |
| `OVP_TRIP` into U501 | — | — | Through U503 (Ioff); the comparator on 08 shares 3V3_SUP anyway |
| `PWR_BTN_N`, `RTC_INT_N`, `DOCK_DET_N` (pulled up to 3V3_AON on 03) | always-on rail | main off | Only U405–U407 inputs (Ioff) touch these lines; U401 sees only the buffer outputs |
| Open-drain status/readback lines into either controller | — | — | Pulled up on the reader's own rail; an open-drain driver cannot source current |
| SWD | — | — | Separate pads J401/J501, no shared conductor |

---

## 7. Arithmetic

All [CALC] unless marked.

**Main HSE load capacitors (C422, C423 = 10 pF).** CL = C1·C2/(C1+C2) + Cstray. Cstray [EST]: STM32 OSC pins about
5 pF each → 2.5 pF in series, + ~2 pF board = 4.5 pF. CL = 10·10/20 + 4.5 = **9.5 pF** against the crystal's 10 pF
(−0.5 pF; pulling ≈ +5 ppm at ~10 ppm/pF [EST], inside the ±10 ppm tolerance budget). Oscillator margin:
gm_crit = 4·ESR·(2πf)²·(C0+CL)² = 4 · 70 Ω · (2π·16 MHz)² · (3 pF + 10 pF)² = 4 · 70 · 1.011×10¹⁶ · 1.69×10⁻²²
= **0.48 mA/V**, below Gmcritmax = 1.5 mA/V (DS13086 Table 80).

**Main LSE load capacitors (C424, C425 = 4.7 pF).** CL = 4.7/2 + Cstray, Cstray = CS_PARA 3–5 pF (Table 81, depends
on silicon revision) + ~1 pF board = 4–6 pF → **6.4–8.4 pF** against 7 pF. gm_crit = 4 · 70 kΩ · (2π·32,768 Hz)² ·
(2 pF + 7 pF)² = 4 · 7×10⁴ · 4.24×10¹⁰ · 8.1×10⁻²³ = **0.96 µA/V** → firmware must set LSEDRV = "medium high"
(Gmcritmax 1.7 µA/V; "medium low" at 0.75 µA/V is too weak).

**Supervisor crystal load capacitors (C508, C509 = 12 pF).** Cstray = EXTAL/XTAL parasitics 2 pF typ each (Table 23)
→ ~1 pF in series, + ~2 pF board = ~3 pF [EST]; worst case with the 8/10 pF maxima ≈ 4.4 + 2 = 6.4 pF. CL = 6 + 3 =
**9 pF typ** (up to 12.4 pF worst case) against 10 pF. The crystal's ESR ≤ 80 Ω equals Table 24's 16 MHz example
(Rm 80 Ω, Cload 8 pF); its C0 maximum (5 pF) is higher than Table 24's examples (1–2 pF) — reviewer item.

**VCAP (C420).** DS13086 §5.1.6: 4.7 µF ±20 % (3.76–5.64 µF), ESR < 20 mΩ at 3 MHz, ≥ 10 V. An 0805 10 V X7R keeps
only ≈ 60–70 % of nominal at a few volts of bias [EST, typical DC-bias curves] — below the limit — so C420 is a
**1206 16 V X7R** (GRM31CR71C475KA01L), which at the ≈ 1.2 V core voltage on VCAP keeps ≈ 90 % [EST] ≈ 4.2 µF.
Not checked against Murata's DC-bias curve.

**I_SHOCK anti-alias filter (R423 1 kΩ, C432 4.7 nF C0G).** fc = 1/(2π · 1 kΩ · 4.7 nF) = **33.9 kHz**; at a
≥ 100 kS/s sampling rate the Nyquist frequency is ≥ 50 kHz. Full scale: 3.2 V / 26.7 mV/A = 120 A. Settling: τ = 4.7 µs.
Charge-sharing error with the ADC sample capacitor (≈ 5 pF [EST]) = 5 pF / 4.7 nF ≈ 0.1 %.

**NTC bias (R424, R506 = 10.0 kΩ 0.1 %).** With a 10 kΩ NTC at 25 °C, V = VREF · 10/(10+10) = VREF/2, maximum
sensitivity at 25 °C; ratiometric to the ADC reference, so the regulator's tolerance cancels. Bias current ≤ 3.3 V /
10 kΩ = 330 µA (only while the controller is powered). Filter 1 kΩ + 100 nF: τ = 0.1 ms; the 100 nF supplies the
sample charge (error ≈ 5 pF/100 nF = 0.005 %).

**Pull-up currents.** 100 kΩ to 3.3 V: 33 µA when the line is held low. 10 kΩ (contacts): 330 µA while pressed.
I²C 4.7 kΩ: 0.7 mA sink. 1-Wire 2.2 kΩ: 1.5 mA sink.

**Fault light (R513 = 220 Ω).** I = (3.3 V − 1.9 V [EST red AlGaInP Vf]) / 220 Ω = **6.4 mA**, inside the P3_0
high-drive pad's limit (ID ≤ 25 mA per pin, Table 8).

**Sounder drive.** Bridge drive from one 3.3 V rail gives **6.6 V peak-to-peak** across the element. R516/R517 33 Ω:
an element of ≈ 10–20 nF [EST] charges with τ = 66 Ω · 20 nF = 1.3 µs, far shorter than the 125 µs half-period at
4 kHz; peak current ≤ 3.3 V / (66 Ω + buffer ≈ 2 × 12 Ω) ≈ 37 mA for < 1 µs, within the buffer's ±24 mA continuous /
±50 mA absolute output rating [EST: absolute maximum not re-read].

**Reset networks.** Main NRST: internal 30–50 kΩ + 100 nF → τ ≈ 4 ms on power-up (the internal POR holds the core in
reset meanwhile). Supervisor RESET_B: 10 kΩ + 10 nF → τ = 0.1 ms.

**Supply decoupling.** Main: 9 × 100 nF (one per VDD pin) + 10 µF; VDDIO2 2 × 100 nF + 4.7 µF; VDDA and VREF+
100 nF + 1 µF each; VDDUSB, VBAT 100 nF each (all from DS13086 Figure 24). Supervisor: 4 × 100 nF + 10 µF, VDD_ANA
100 nF + 1 µF — **[EST], the NXP datasheet gives no decoupling figure.**

---

## 8. Unverified pins, ratings and assumptions

1. **MCXA153VLH custom symbol** — drafted by script from Table 41's text extraction; four rows (P1_7, P1_13, P3_11,
   P3_15) needed manual reading of wrapped cells. `PinoutVerified = NO` until a person checks it against the PDF.
2. **CY15B104QN-50SXI custom symbol** — from Figure 2 and Table 1; `PinoutVerified = NO`.
3. **Crystal pad assignment** of Y401 (ABM8: pads 1/3 crystal, 2/4 case) taken from the outline drawing, which is a
   graphic; the ECX-32 table states it in text (1 and 3 crystal, 2 and 4 ground).
4. **Crystal ordering codes** Y401 (ABM8-16.000MHZ-10-D1X-T) and Y501 (ECS-160-10-33-AGN-TR) were assembled from the
   makers' ordering guides, not found as catalogue listings.
5. **Murata MLCC suffixes** for 4.7 µF/10 µF X7R and the C0G values (fields `Verify` on each part).
6. **Not opened:** piezo, LED, ferrite bead, Tag-Connect, Yageo and Panasonic resistor datasheets.
7. **Sounder loudness:** the D-26 target (≥ 85 dB at 10 cm, design §B10 [EST]) is **not** shown to be met by a
   12 mm piezo at 6.6 V p-p (contract issue C-2).
8. **MCXA153 boot behaviour:** I assumed that ISPMODE_N high at reset is the only strap needed to run from flash. The
   reference manual's boot chapter was not opened.
9. **MCXA153 decoupling** values are common practice, not from NXP.
10. **PB2 as a WKUP pin** (WKUP1 alternative) comes from the stm32-data map; DS13086's wrapped Table 27 cell was not
    read for it. Not needed for function (EXTI2 is enough).
11. **AN2606 (bootloader) not opened**; the bootloader pin list used is the "(boot)" marking in DS13086 Table 27.
12. **VCAP capacitance under DC bias** (§7) — estimated from typical curves, not from the Murata curve for the part.
13. **MAX30009 logic level** on the `SBZ_*` lines (contract issue C-3).

---

## 9. Departures from the design specification

| Design said | Captured | Why |
|---|---|---|
| §B2.2 pin total 82 of about 110 | 76 (70 + 6) of 114 | arithmetic corrected; package verified |
| Memory LCD on an SPI (implied) | USART3 in synchronous master mode (CK + TX) + GPIO chip select | the U585 has three SPIs and they are taken by the front end, the voltage ADC and the memories; a transmit-only display works on a synchronous USART |
| Not stated | `BR_FAULT_N` also drives TIM1's break input | a sequencer fault stops the phase commands in hardware; no new net |
| §B2.4 "supervisor crystal frequency per the MCX A153 range" | 16 MHz, CL 10 pF, different maker from the main's | inside Table 23 (8–50 MHz); Table 24 example |
| Not stated | VBAT and VDDUSB tied to 3V3_MAIN; VDDA/VREF+ through a ferrite bead | DS13086 §3.9.1 prefers tying unused supplies to VDD; the RV-3028 on 03 keeps time |
| §B10 "piezo driven differentially by two supervisor pins" | through the contract's dual buffer U504 with 33 Ω per side | buffer as listed in contract §8; series resistors limit the element's charging current |
| §B10 red fault light | LED driven directly from a high-drive pin (P3_0) | 6.4 mA is within the pin's rating; no transistor needed |

No inter-sheet net was renamed, added or deleted; all 65 contract nets of sheet 04 and all 19 of sheet 05 are
present (checked by script against contract §5).

---

## 10. Contract issues

- **C-1 (minor, design document):** §B2.2's total reads "76 + 6 = 82 of about 110"; its rows sum to 70, so it should
  read "70 + 6 = 76 of 114". The supervisor's "about 50" is 52. **Proposed change:** correct both numbers.
- **C-2 (sounder part and loudness):** contract §8 puts the piezo on sheet 05, but §9 gives the speaker **and piezo**
  part choice to WP-C. I captured a provisional part (Murata PKLCS1212E4001-R1) so the sheet is complete. A small
  piezo driven at 6.6 V p-p is unlikely to reach the ≥ 85 dB at 10 cm of §B10 [EST: this class of part is typically
  specified near 65–75 dB at 10 cm]. **Proposed change:** WP-C chooses the sounder **and** says what drive it needs
  (higher-voltage bridge, or a transducer with an integrated driver on 3V3_SUP); WP-B then updates LS501 and U504.
  Until then LS501 carries a `Verify` field marking it provisional.
- **C-3 (cross-sheet, `SBZ_*` levels):** the MAX30009 on 16 runs from 1V8_SUP. If its logic interface runs at 1.8 V,
  `SBZ_MISO` (≤ 1.8 V high) does not reach U501's VIH (0.7 × 3.3 V = 2.31 V, Table 9), and the contract's 100 kΩ
  pull-up of `SBZ_INT_N` **to 3V3_SUP** (on 05) would hold a 1.8 V part's pin above its rail. I did not open the
  MAX30009 datasheet. **Proposed change:** WP-H confirms the MAX30009's I/O supply; if it is 1.8 V, either power its
  I/O from 3V3_SUP (if the part allows) or add level translation on 16 and move `SBZ_INT_N`'s pull-up target.
- **C-4 (`I_SHOCK` headroom):** full scale 3.2 V against an ADC reference that is the 3.3 V regulator itself (needed,
  since the 2.5 V internal reference would clip). A −3 % regulator leaves 0 V headroom. A failed-open divider leg on
  13 puts up to 5 V on the line; R423 limits the injected current to ≈ 1.7 mA. **Proposed change:** WP-G scales
  `I_SHOCK` to ≤ 3.0 V full scale and clamps it on 13 (or the contract adds a clamp on 04).
- **C-5 (none blocking):** every net of contract §5 that names 04 or 05 is captured with its default resistor; I found
  no R11 violation on these two sheets. Every open-drain/contact line read here reads the non-permissive state when
  open (§4, §5 tables).

---

## 11. Firmware requirements arising from this capture (for design §F)

1. **Main option bytes:** program nSWBOOT0 = 0 and nBOOT0 = 1 so that BOOT0 (and a lifted R426) cannot start the
   system bootloader.
2. **LSE drive:** LSEDRV = medium high (§7).
3. **TIM1:** prescaler 64 or equivalent; break input enabled, active low, on `BR_FAULT_N`; outputs' idle and
   break states low.
4. **I/O state:** I²S lines low whenever `EN_5V0` is low (already F-2); the memory chip selects are driven high
   before SPI3 is enabled.
5. **Supervisor:** keep P3_29 (ISPMODE_N) as an input; consent clocks from the control loop only (F-1).

---

## 12. Things a reviewer should check first (my top five doubts)

1. **MCXA153VLH pin table** against NXP Table 41 (custom symbol drafted from a text extraction of a rotated, wrapped
   table) — especially pins 1, 7, 35, 39 and the mux numbers used.
2. **VCAP capacitor** (C420): effective capacitance and ESR at the core voltage against DS13086 §5.1.6, and the
   other MLCC MPNs built from the Murata scheme.
3. **Sounder loudness and driver** (C-2): the D-26 alarm may be too quiet as captured.
4. **`SBZ_*` logic levels** with the MAX30009 (C-3).
5. **Bootloader and option-byte reasoning** (§4): that no safety output can be driven high if the main ever runs its
   bootloader, which rests on DS13086's "(boot)" marks, not on AN2606.

---

## 13. Totals and build result

| | Sheet 04 | Sheet 05 |
|---|---:|---:|
| Parts | **74** (7 U, 30 R, 33 C, 2 Y, 1 FB, 1 J DNP) | **39** (4 U, 16 R, 14 C, 1 Y, 1 FB, 1 J DNP, 1 LS, 1 D) |
| Nets on the sheet | 92 (65 inter-sheet + 2 rails + 25 local) | 42 (19 inter-sheet + 2 rails + 21 local) |
| No-connect pins | 41 | 27 |

Both sheets together: 113 parts, 131 distinct nets, 408 pin connections — the netlist round trip matches the pin
tables exactly. ERC: 0 errors; 7 errors accepted **until the driver sheet is captured** (GND → 01, 3V3_MAIN/3V3_SUP
→ 03, Ioff-receiver inputs → 03 and 08); 54 `isolated_pin_label` warnings accepted, each naming the placeholder sheet
that holds the other end (they disappear at the merge).

**Generator observation (tools/ not modified):** KiCad's `power_pin_not_driven` item text names only the power
symbol's reference (`#PWR4000`), not the net, so an acceptance that must expire at a different sheet for GND than for
3V3_MAIN has to match the generated `#PWR` number. Those numbers follow part order, so adding a part before U401 would
silently move them. A `net` field on `erc_accepted` would be more robust.

**Generator observation 2:** the KiCad integration test writes to a fixed directory (`~/.cache/ws-schgen-itest`), so
two worktrees running `tools/build.sh` at the same moment can make each other's test fail ("Schematic file does not
exist"). It happened once during this capture; the rerun passed. A per-process temporary directory would avoid it.
