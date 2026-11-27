# Capture notes — WP-C ui+sense (sheets 06_ui, 07_sense_main, 08_sense_sup)

> **NOT FOR CONSTRUCTION — unreviewed design.** These notes and the sheets they describe were drawn with AI
> assistance and have not been reviewed by a qualified high-voltage or medical-device engineer. Nothing here tells
> anyone to build, power or probe anything (CLAUDE.md rule 2, SAFETY.md §2a). Nothing here is safe or ready for use
> on a person or animal. All arithmetic below is on paper **[CALC]**; nothing was simulated or measured.

Captured 27 November 2026 against contract issue 2 and `docs/HARDWARE_DESIGN.md` v0.2. Source of truth: the pin
tables `hardware/aed-main/specs/06_ui.toml`, `07_sense_main.toml`, `08_sense_sup.toml` and the symbol source
`hardware/lib/src/aed_uisense.toml`. Every number below gives its source; "[UNVERIFIED]" means looked for and not
confirmed.

## 1. What is on each sheet (plain language)

**06_ui — user interface.** The small reflective "memory" display (Sharp LS013B7DH05) runs from the always-on
supply, so whatever it last showed ("READY" or "NOT READY") stays visible when the device is off; a 1-per-second
tick from the always-on clock keeps the liquid crystal healthy. The main controller can blank it through a tiny
always-on inverter; if the main controller is off, the inverter's input is pulled low and the display stays **on**.
A digital audio amplifier (MAX98357A) drives a 28 mm speaker for voice prompts and the compression beat; it runs
from the 5 V supply and is off by default. The cancel button is two separate sealed switches under one cap: one
wire goes only to the main controller, the other only to the supervisor; they share nothing but ground. The
adult/child slide switch has two poles: one tells the main controller the position, the other — closed only in
CHILD — feeds 5 V to the charger's child voltage ceiling, so any broken wire or a switch caught between positions
leaves the higher (adult) ceiling in place, as decision D-60 requires. The power button goes to the always-on wake
logic. Every switch line has an ESD clamp and a small resistor–capacitor filter.

**07_sense_main — the main controller's capacitor-voltage measurement.** The scaled-down capacitor voltage from
the high-voltage board (1/998.5 of the real voltage) comes in through a 100 kΩ resistor, is buffered by a precision
amplifier and measured by a 24-bit converter (TI ADS1220) against a 2.5 V reference (TI REF3425). The same
converter reads the two bridge outputs (to find a shorted switch or a stuck test relay) and the capacitor's own
temperature sensor. A precision comparator produces `LOWV_OK`, which is high only while the capacitor is below
about 55 V; the interlock uses it to allow the low-voltage self-test (D-61). Everything on this sheet runs from the
main controller's supply.

**08_sense_sup — the supervisor's independent measurement and the hardware over-voltage trip.** A second,
different chain — different resistor string (on sheet 11), different converter maker (Analog Devices AD7171),
different reference maker (Analog Devices ADR3425), a different amplifier family and a different comparator — on
the supervisor's own supply. Its comparator raises `OVP_TRIP` at about 2,160 V; that lights the optocouplers on
sheet 11 which force both discharge paths on, and it keeps them on until the capacitor has fallen to about
1,000 V. No part, supply, reference or clock is shared with sheet 07.

## 2. Parts list (WP-C)

Datasheets were read on 27 November 2026. analog.com, st.com, microchip.com, mouser.com and ckswitches.com refused
or timed out for automated access from this machine; the copy actually read is named.

| Ref | Part | Maker | Datasheet (URL as in the pin table) — copy actually read |
|---|---|---|---|
| U601 | LS013B7DH05 memory LCD (footprint: Hirose FH12-10S-0.5SH 10-way FPC) | Sharp | Sharp spec LD-26912A (10 Sep 2014), read at https://cdn.eicom.ru/media/PDF/6421265.pdf |
| U602 | MAX98357AETE+T | Analog Devices (Maxim) | https://www.analog.com/media/en/technical-documentation/data-sheets/MAX98357A-MAX98357B.pdf — read copy 19-6779 Rev 13 (7/19) at https://www.makerguides.com/wp-content/uploads/2026/01/Datasheet_MAX98357A.pdf |
| U603 | SN74AUP1G04DBVR | TI | https://www.ti.com/lit/ds/symlink/sn74aup1g04.pdf (SCES571K) |
| LS601 | AS02808MR-R 28 mm 8 Ω speaker | PUI Audio | https://api.puiaudio.com/filename/AS02808MR-R.pdf (rev D, 17 Jul 2025) |
| SW601, SW602, SW603 | KSC643J LFS sealed tactile, gold, J-lead | C&K | https://www.ckswitches.com/media/1972/ksc6.pdf — read "KSC Series Sealed Tact Switch" at mm.digikey.com (silver column only legible) |
| SW604 | JS202011JCQN DPDT slide | C&K | https://www.ckswitches.com/media/1422/js.pdf — read 29 Jun 2016 JS sheet at cdn.hackaday.io |
| D601–D605 | TPD1E10B06DPYR | TI | https://www.ti.com/lit/ds/symlink/tpd1e10b06.pdf (SLLSEB1G) |
| U701 | ADS1220IPWR | TI | https://www.ti.com/lit/ds/symlink/ads1220.pdf (SBAS501D, May 2026) |
| U702 | REF3425IDBVR | TI | https://www.ti.com/lit/pdf/SBAS804 (SBAS804G, Apr 2026) |
| U703 | LMV761MF/NOPB | TI | https://www.ti.com/lit/ds/symlink/lmv761.pdf (SNOS998I) |
| U704 | OPA333AIDBVR | TI | https://www.ti.com/lit/ds/symlink/opa333.pdf (SBOS351E) |
| U801 | AD7171BCPZ-REEL7 | Analog Devices | https://www.analog.com/media/en/technical-documentation/data-sheets/AD7171.pdf — read Rev. C at https://cdn.eicom.ru/media/PDF/448041.pdf |
| U802 | ADR3425ARJZ-R7 | Analog Devices | ADR3412/…/ADR3450 data sheet — read **Rev. B** at https://nonprod.docs.rs-online.com/1656/0900766b812d4deb.pdf (later revisions not checked) |
| U803 | TLV9031DBVR | TI | https://www.ti.com/lit/ds/symlink/tlv9031.pdf (SNOSDA3H, Nov 2025) |
| U804 | OPA388IDBVR | TI | https://www.ti.com/lit/ds/symlink/opa388.pdf (SBOS777D) |
| R, C | 0.1 % thin film (Panasonic ERA-3AE on 07, Vishay TNPW on 08), 1 % thick film (Yageo RC0603 on 06/07, Vishay CRCW on 08), C0G/X7R ceramics (Murata on 06/07, KEMET on 08) | — | maker series pages as in the pin tables; values only, not individually re-read |

Totals: **97 parts** (06: 44, 07: 30, 08: 23); **67 distinct nets** (per sheet 06: 35, 07: 21, 08: 13, GND
counted on each): **30 inter-sheet signals** (15 + 10 + 5), 7 rails (GND, 3V3_AON, 5V0_SYS, 3V3_MAIN, VREF_M,
3V3_SUP, VREF_S) and 30 local nets; **263 pin connections**, all matched by the netlist round trip. Custom symbols:
8 (`aed_uisense`).

## 3. Arithmetic for every chosen value

All [CALC]; the script that produced the bands is reproduced in the formulas below.

### 3.1 Divider ratios (strings on sheet 11, contract §8)
- Main: 40.3 kΩ / (40.2 MΩ + 40.3 kΩ) = **1/998.5**; tap source resistance 40.26 kΩ + 10 kΩ clamp resistor = 50.3 kΩ.
- Supervisor: 39.8 kΩ / (39.84 MΩ + 39.8 kΩ) = **1/1,002.0**; source 49.8 kΩ.

### 3.2 Input networks
- R701, R801 = 100 kΩ (rule R7). Worst unpowered injection: 3.6 V clamp / 110 kΩ = 33 µA (OPA333 limit ±1 mA;
  OPA388 ±10 mA). R × C = 100 k × 10 nF = 1 ms (−3 dB 159 Hz); lag at 205 V/s charging slope ≈ 0.2–0.3 V.
- R702, R802 = 1 MΩ to the chain's own rail at each follower output: an open follower output reads full scale.
- ADC filters: R703 1 k + C703 10 nF (10 µs); R803 1 k + C803 10 nF. AD7171 input current 400 nA/V → 0.78 mV at
  1.95 V across R803: proportional to the input (a gain term, removed by calibration); drift 5 µV over 45 K.
- OUT taps: 100 kΩ + 10 nF (1 ms). ADS1220 bypass-mode input current (typical curves only) ≈ 20 nA → 2 mV → 2 V of
  output-node voltage: irrelevant to "near 0 V" vs "capacitor voltage".
- NTC: 10.0 kΩ 0.1 % from VREF_M; AIN3 = 2.5 V × R_NTC/(R_NTC + 10 k): 2.18 V at −20 °C (≈ 68 kΩ), 1.25 V at
  25 °C, 0.42 V at 70 °C (≈ 2 kΩ). R711 1 k + C708 100 nF.

### 3.3 LOWV_OK (07)
- Threshold node: VREF_M × 1.00 k / (46.4 k + 1.00 k) = 52.74 mV; Thevenin 978.9 Ω; hysteresis R707 1.62 MΩ.
- Output high (OK): trips to "not OK" at tap = (52.74 mV × 1.62 M + 3.3 V × 978.9) / (1.62 M + 978.9) = 54.70 mV =
  **54.6 V**. Output low: returns to OK at 52.71 mV = **52.6 V**. Hysteresis 2.0 V.
- Band (worst case of: LMV761 Vos ±1 mV over temperature; follower Vos + bias 0.16 mV; resistors 0.1 % + 25 ppm/K ×
  45 K; REF3425 0.1 %; 3V3_MAIN ±6 %; main string ratio **uncalibrated ±1.5 %**): **50.4 V – 57.0 V**.
  Requirement (D-61, §B14): OK only below 60 V → **fits, 3 V margin**. The supervisor refuses a test permit above
  50 V reading on its own chain, so the two limits sit next to each other (see doubt 3).

### 3.4 OVP_TRIP (08)
- IN− = 2.5 V × 15.6 k / 24.69 k = 1.5796 V. IN+ = tap via R805 36.5 k, feedback R806 100 k from the output.
- Trip (output low): tap = 1.5796 × (1 + 0.365) = 2.1561 V = **2,160 V**. Release (output high, VOH ≈ 3.15 V):
  2.1561 − 3.15 × 0.365 = 1.006 V = **≈ 1,008 V**.
- Band, all terms (TLV9031 Vos ±2 mV over temperature; ADR3425 ±0.1 % + 8 ppm/K × 45 K + 30 ppm + 70 ppm
  hysteresis; resistors 0.1 % + 25 ppm/K × 45 K; follower 0.07 mV; supervisor string ±0.2 % tolerance, ±0.225 %
  temperature, ±0.25 % life): **2,132 – 2,189 V**. With the design table's own assumption set (no string
  temperature/life terms): 2,142 – 2,179 V. Design §B4.4 table: 2,138 – 2,182 V.
- Checks: bottom 2,132 V is 123 V above the top of the firmware at-plan window (2,008.6 V) → never stops a normal
  charge; top 2,189 V is **11 V below the 2,200 V rating** (design said 18 V). Release 919 – 1,098 V (depends on
  VOH, i.e. 3V3_SUP and the LED load on 11).

### 3.5 User interface
- I²S series 4.7 kΩ (contract): 4.7 k × 3 pF = 14 ns ≪ 325 ns BCLK period.
- SD_MODE: 3.3 V × R_PD/(R_PD + 634 k), R_PD = 92–108 kΩ internal → 0.38–0.50 V (GPIO 3.0–3.4 V), between B0
  (0.355 V max) and B1 (0.65 V min) → mono mix. Injection with 5V0_SYS off ≤ 5 µA.
- Gain +6 dB (GAIN_SLOT to VDD): 0 dBFS → 2.1 + 6 = 8.1 dBV = 2.54 V rms → 0.81 W into 8 Ω (0.95 W at −15 %
  impedance): inside the speaker's 1.0 W rating. SPL = 80 ± 3 dB (1 m, 1 W, PUI) + 10·log(0.81) = **79 ± 3 dB at
  1 m**. 85 dB at 1 m would need 10^(5/10) = 3.2 W.
- Cancel: (10 k + 1 k) × 100 nF = 1.1 ms release, 1 k × 100 nF = 0.1 ms press. Power button: 101 k × 10 nF =
  1 ms. Mode lines: 11 k × 10 nF = 0.1 ms.
- MODE_CHILD_HW: 5.0 V × 100 k/(100 k + 1 k) = 4.95 V; a shorted wire draws 5 mA through R622.
- DISP: 3.3 V / 220 Ω = 15 mA peak into 0.1 µF; τ = 22 µs.

## 4. Error budgets against D-28 (1 % + 7 V)

Allowance: at 1,950 V = 26.5 V (1.36 %); at 1,150 V = 18.5 V (1.61 %). Worst-case linear sums over −20 to +70 °C
(45 K from 25 °C) and life; both chains assume a two-point factory calibration of the whole chain (residual 0.05 %).

**Main chain (07): HVCR2512 string + OPA333 + ADS1220 + REF3425**

| Term | Assumption and source | Error |
|---|---|---:|
| Ratio residual after calibration | parts study §7 | 0.050 % |
| String temperature mismatch | HVCR ±100 ppm/K (Stackpole family page, **page only**) vs bottom leg 25 ppm/K → 125 ppm/K × 45 K | 0.563 % |
| Voltage coefficient | HVCR < 2 ppm/V (page only) × 195 V per resistor | 0.039 % |
| String life drift | **not found** for HVCR; 0.3 % assumed (parts study) | 0.300 % |
| Board leakage | 100 GΩ across 40.2 MΩ | 0.040 % |
| ADS1220 gain drift + INL | 4 ppm/K × 45 K + 15 ppmFSR (SBAS501D) | 0.019 % |
| REF3425 | 6 ppm/K × 45 K + 35 ppm long-term + 30 ppm hysteresis (SBAS804G) | 0.034 % |
| Tap clamp leakage (sheet 11) | **≤ 10 nA required** × 40.3 kΩ | 0.40 V |
| OPA333 bias × source | 1 nA assumed over temperature × 150 kΩ | 0.15 V |
| OPA333 offset + drift; ADS1220 offset drift + input current × 1 k; quantisation | SBOS351E, SBAS501D | 0.05 V |
| **Total** | | **1.04 % + 0.61 V** |
| at 1,950 V | | **21.0 V vs 26.5 V — fits (5.5 V margin)** |
| at 1,150 V | | **12.6 V vs 18.5 V — fits (5.9 V margin)** |

**Supervisor chain (08): TNPV1206-class string + OPA388 + AD7171 + ADR3425**

| Term | Assumption and source | Error |
|---|---|---:|
| Ratio residual after calibration | parts study §7 | 0.050 % |
| String temperature mismatch | thin film 25 + 25 ppm/K × 45 K [BK for TNPV] | 0.225 % |
| Voltage coefficient | ≤ 1 ppm/V assumed [UNVERIFIED] × 163 V | 0.016 % |
| String life drift | 0.25 % assumed | 0.250 % |
| Board leakage | 100 GΩ across 39.8 MΩ | 0.040 % |
| AD7171 gain drift | 0.07 LSB/K × 45 K at 1.95 V (Rev. C, typical only) | 0.012 % |
| ADR3425 | 8 ppm/K × 45 K + 30 ppm + 70 ppm hysteresis (Rev. B) | 0.046 % |
| Tap clamp leakage (sheet 11) | ≤ 10 nA × 39.8 kΩ | 0.40 V |
| OPA388 IB × source | 0.4 nA (0–85 °C max) × 150 kΩ | 0.06 V |
| OPA388 offset; AD7171 offset drift + INL; input-current drift × 1 k; quantisation ±½ LSB (76 mV) | SBOS777D, AD7171 Rev. C | 0.10 V |
| **Total** | | **0.64 % + 0.55 V** |
| at 1,950 V | | **13.0 V vs 26.5 V — fits (13.5 V margin)** |
| at 1,150 V | | **7.9 V vs 18.5 V — fits (10.6 V margin)** |

Compared with the design's figures (main 0.92 % + 1 V, supervisor 0.51 % + 1 V): the main chain is **worse**
because HVCR's published TCR (±100 ppm/K) is larger than the 50 ppm/K mismatch the design assumed; it still fits,
but only if the HVCR life drift is ≤ ≈ 0.6 % — a number nobody has found yet. AD7171 gain/offset figures are
typical-only in its data sheet; the factory calibration absorbs the initial values, but their drift is not
guaranteed.

## 5. Unverified pins, ratings and assumptions

**Pins**
- SW604 C&K JS202011JCQN: commons = pins 2 and 5, CHILD throws = pins 1 and 4 on the same slider side — taken
  from the JS DPDT circuit diagram (2016 sheet, through-hole drawing); **not confirmed for the SMT JCQN pad
  numbering** in the stock footprint. Break-before-make not stated.
- SW601–SW603 KSC6: two terminals per contact side; the stock footprint pads are 1,1,2,2. Contact-pair grouping
  against the C&K drawing not read (ckswitches.com refused access).
- U601 LS013B7DH05: pin table read from Sharp LD-26912A; **FPC contact side / connector orientation not checked**;
  the symbol's MP pin is the connector's mounting pads, not a display terminal.
- U602 MAX98357A: read from a third-party-hosted copy of Rev 13.
- U802 ADR3425: read from Rev. B (2010-era) copy; current revision not checked.
- U801 AD7171: stock KiCad symbol matches Rev. C Table 6 (names SCK/~PDRST = datasheet SCLK/PDRST); its default
  footprint (Microchip DFN-10 EP 1.55 × 2.48 mm) not checked against ADI CP-10-9.
- TPD1E10B06 DPY on a generic 0402 land; MAX98357A TQFN land not checked against Maxim's pattern.
- All custom symbols carry `PinoutVerified = NO …` until a person checks them.

**Ratings and assumptions**
- HVCR TCR ±100 ppm/K and VCR < 2 ppm/V from distributor pages (page only); HVCR life drift not found.
- TNPV VCR ≤ 1 ppm/V and 25 ppm/K: background knowledge.
- OPA333 bias current over temperature: only typical (150 pA) is given; 1 nA assumed.
- ADS1220 input current with PGA bypassed: typical curves only; 20 nA assumed.
- AD7171 offset, gain, INL: typical only.
- LMV761: TI's page says "a newer version of this product is available" — lifecycle to be checked.
- SN74AUP1G04 output current ±20 mA: not re-read.
- KSC6 gold version: −40…+125 °C and low-current suitability from a search extract only; silver column read
  (minimum 1 mA).
- JS202011JCQN: silver contacts, 0.3 A / 6 V; contact reliability at 50–330 µA not stated; **not sealed**.
- AS02808MR-R: no ingress rating; operating −20…+60 °C (device service range to +70 °C).
- NTC_CAP_M: assumed returned to GND on sheet 11.
- Sheet 11 tap clamps: leakage ≤ 10 nA at 2.3 V, 70 °C assumed in both budgets.

## 6. Departures from the design specification, and why

| # | Departure | Why |
|---|---|---|
| D-C1 | **Speaker loudness target not met**: ≈ 79 ± 3 dB at 1 m vs 85 dB(A) (parts study §8). No 28 mm part found that gets there within its rating and the MAX98357A's 1.4 W/8 Ω | Physics of a 28 mm speaker at ≤ 1 W; needs a bigger or more sensitive speaker (owner/enclosure decision) |
| D-C2 | A **zero-drift follower added on the main chain** (U704 OPA333); the design lists none for sheet 07 | R7 needs ≥ 100 kΩ in series; the ADS1220's bypass-mode input current is not guaranteed, so 100 kΩ straight into it could cost several volts |
| D-C3 | Supervisor "zero-drift buffer" = **TI OPA388**, not an Analog Devices part | ADA4528-1 preferred for maker diversity but its data sheet could not be retrieved; OPA388 is a different family from the main's OPA333 |
| D-C4 | `LOWV_OK` nominal threshold **54.6 V** (not 60 V) | So that the whole worst-case band (50.4–57.0 V) is below 60 V |
| D-C5 | OVP **release ≈ 1,000 V** (wide hysteresis) | A non-latching comparator otherwise stops the forced dump 1–2 % below the trip and re-arms at the top of the range; see contract issue C-5 |
| D-C6 | OVP band **2,132–2,189 V** vs the design's 2,138–2,182 V | String temperature and life terms included; still inside both limits |
| D-C7 | 1 kΩ series on LCD SCLK/SI/SCS; 220 Ω between inverter and DISP | Panel input high must not exceed its own VDD; AUP output current into Sharp's recommended 0.1 µF |
| D-C8 | 1 kΩ series on the mode switch's pole B | A shorted MODE_CHILD_HW wire must not load 5V0_SYS |
| D-C9 | Follower outputs pulled up (1 MΩ) to their rails | A failed follower reads over-voltage/not-low-voltage rather than "safe" (R11 spirit) |

## 7. Contract issues

- **C-1 Tap ratios.** §5.6 says `VCAP_M_TAP` 1:998 and `VCAP_S_TAP` 1:1,000; the §8 resistor values give
  **1:998.5** and **1:1,002.0**. Proposed change: write the exact ratios in §5.6 (firmware calibration constants).
- **C-2 Tap clamp leakage (sheet 11) can swamp both budgets.** A 3.6 V zener at 2.2 V can leak microamperes
  [BK]; 1 µA × 40 kΩ = 40 mV = **40 V** of error, more than the whole allowance, and strongly temperature
  dependent. Proposed change to §8 (sheet 11): "tap clamp leakage ≤ 10 nA at 2.3 V and 70 °C" (WP-F to choose a
  low-leakage clamp). Both budgets here assume 10 nA (0.4 V).
- **C-3 R11: an open `VCAP_M_TAP` reads "below 60 V".** If the tap wire is open or unsoldered, the follower input is
  pulled to 0 V through the 10 nF/100 kΩ and `LOWV_OK` goes high at any capacitor voltage. Mitigation already in
  the design: the supervisor refuses a test permit above 50 V on its own chain, and the main reading disagrees with
  the supervisor's (fault 32 class). I did **not** add a pull-up at the tap: any bias current through the 50 kΩ
  source shifts the main reading. Proposed change: record this in §5.6 as an accepted R11 exception with that
  mitigation, or have sheet 11 add an open-tap detector.
- **C-4 Rails table.** §4 lists sheet 06 as a user of `3V3_MAIN`; the captured sheet 06 uses only `3V3_AON`,
  `5V0_SYS` and `GND`. Proposed change: remove 06 from the 3V3_MAIN row.
- **C-5 OVP latch.** §B4.6 says the over-voltage comparator "empties the capacitor"; a non-latching comparator
  cannot. Without a new net I chose a release near 1,000 V. Proposed decision: either accept that, or add a latch
  with a supervisor reset (would need a new inter-sheet net, e.g. `OVP_RST_N`, 05 → 08).
- **C-6 OVP_TRIP load.** TLV9031 VOH is specified to 4 mA (175 mV drop max). Two opto LEDs in series (≈ 2.4 V) from
  ≈ 3.15 V leave ≈ 0.75 V for WP-F's resistor, so the LED current is sensitive to VOH. Proposed: WP-F states the
  LED current (≤ 4 mA) and checks the opto CTR at the minimum.
- **C-7 MODE_CHILD_HW needs 5V0_SYS.** With `EN_5V0` low the line reads low (adult ceiling) even in CHILD; harmless
  (the charger has no supply then), but `MODE_HW_RB_N` is only meaningful with 5V0_SYS up. Proposed: firmware F-5
  compares only after `EN_5V0` is high (note for WP-B/firmware).
- **C-8 NTC_CAP_M return.** §5.6 does not say where the NTC's other end goes; I assume GND on sheet 11.

## 8. Things a reviewer should check first (my top five doubts)

1. **Main chain over temperature and life.** HVCR's ±100 ppm/K against a 25 ppm/K bottom leg is 0.56 % alone; the
   budget fits only if the string's life drift is ≤ ≈ 0.6 % (not found). Either make the bottom leg the same
   thick-film family so the coefficients track, or compensate with the board temperature, or use thin film on both
   chains (owner question 4).
2. **Tap clamp leakage on sheet 11** (C-2): one wrong diode turns a 7 V budget into tens of volts.
3. **LOWV_OK failure modes** (C-3) and its closeness to the supervisor's 50 V test-permit limit: band 50.4–57.0 V.
4. **OVP behaviour**: band top 11 V under the rating; non-latching; release depends on VOH and the LED load; and
   while the firmware keeps `CHG_RUN` high the charger and the forced dump will cycle (average dump power ≈ charger
   output) until a controller stops it.
5. **Mechanical parts**: the mode switch's SMT pin numbering, break-before-make and low-current contact
   reliability; the speaker's loudness, temperature and sealing; the display FPC orientation.

## 9. Not done

- No circuit simulation (follower stability with 10 nF behind 1 kΩ, comparator behaviour at the hysteresis points).
- ADI and Maxim data sheets read only from third-party copies; ADA4528-1 not obtained.
- KSC6 and JS mechanical drawings not read from C&K directly.
- Footprints for AD7171, MAX98357A and TPD1E10B06 not checked against the makers' land patterns.
