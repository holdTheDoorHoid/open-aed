# WP-A power — capture notes (sheets 01, 02, 03)

Contract issue 2, captured 27 November 2026; revised for design 0.3 / contract issue 3 the same day (section
"Revision 0.3" at the end). Pin tables: `hardware/aed-main/specs/01_power_battery.toml`,
`02_power_dock.toml`, `03_power_rails.toml`; custom symbols: `hardware/lib/src/aed_power.toml`.

> **NOT FOR CONSTRUCTION — unreviewed design.** Drawn with AI assistance from datasheets; not reviewed by a qualified
> engineer, not built, not simulated, not measured. These notes describe a drawing. They are not assembly, test or
> bring-up instructions (CLAUDE.md rule 2, SAFETY.md §2a).

Tags as in the design document: **[CALC]** worked here, **[EST]** estimate, **[BK]** background knowledge,
**[UNVERIFIED]** looked for and not confirmed.

---

## 1. What is on each sheet (plain language)

**01_power_battery — the battery and its guard.** The two-cell battery plugs into a keyed eight-pin connector (two pins
each for plus and minus because of the current, one for the middle of the pack, two for the cell temperature sensors
and one return). A single chip from Analog Devices (MAX17320) watches each cell's voltage, the current (through a
2-thousandths-of-an-ohm resistor in the minus lead) and both cell temperatures, keeps a fuel gauge, and opens two
power transistors in the plus lead if anything goes wrong. After those transistors sits the main 25 A fuse, and after
the fuse the battery bus `VBAT_SYS` that feeds everything else. The same sheet holds the "battery under load" test: two
power resistors (0.8 Ω in total) switched across the battery by a transistor, for the main controller to measure how
far the battery voltage sags. A small comparator and a capacitor turn that load off by themselves after about
0.12–0.22 seconds, even if the controller's command line is stuck on.

**02_power_dock — charging and the dock.** The device touches the dock with three flat contacts: 5 V, data and ground.
The 5 V contact goes through a surge clamp and an electronic fuse (TPS25947) that only connects it when the dock voltage
is between about 4.3 V and 5.9 V, limits the current, and never lets battery voltage flow back out to the contact. The
result is `VDOCK`, which feeds the battery charger (TI BQ25887). The charger lifts 5 V to the battery's 8.4 V, limits its
input to 2 A, balances the two cells, and stops outside roughly 1–44 °C using its own temperature sensor placed against
the cells. The data contact is only ever pulled low by the device (a transistor powered from the dock), so with no dock
present nothing on the contact is live; a receiver chip on the main controller's supply passes what it hears to the
controller. A last transistor tells the controller and the wake logic "docked" whenever dock power is present.

**03_power_rails — the supplies and the always-on corner.** Four regulators run from the battery bus, each behind its own
small fuse where the design asks for it: one 3.3 V regulator for the main controller, a separate identical one for the
supervisor, a 5 V regulator for the speaker, charger control and interlock (off unless the main controller switches
it on), and a tiny always-on 3.3 V regulator. The power button switches on both controller regulators directly through
two separate transistors, with no logic chip in between. The main regulator can also be switched on by the wake timer,
by docking, or kept on by the main controller itself; the supervisor regulator by the main ("wake the supervisor for a
self-test") or by the supervisor itself. The always-on corner holds the wake timer (RV-3028-C7) and the "ready" latch,
whose green light flashes for about 3 thousandths of a second each second while the device is ready, and which starts
in "not ready" whenever the battery is first connected.

---

## 2. Parts

All datasheets were read on **27 November 2026**. "Distributor copy" means the manufacturer's own server could not be
reached from this machine (Analog Devices timed out; Littelfuse, Nexperia and Murata refused automated access).

### 2.1 Active and non-generic parts

| Ref | Part number | Maker | What for | Datasheet used (read 27 Nov 2026) |
|---|---|---|---|---|
| U101 | MAX17320G20+T (I²C, no SHA, 24-TQFN 4×4) | Analog Devices | protector + gauge | ADI online datasheet pages (assets2.analog.com/cdp/onlinedatasheet/max17320-en/: pin specifications, package information, ordering information, absolute maximum ratings, detailed description, typical application circuits), opened in a browser; the PDF https://www.analog.com/media/en/technical-documentation/data-sheets/MAX17320.pdf could not be downloaded |
| Q101, Q102, Q103 | CSD17573Q5B | Texas Instruments | CHG / DIS protection FETs, load switch | https://www.ti.com/lit/ds/symlink/csd17573q5b.pdf (SLPS492B) |
| R101 | CSS2H-2512K-2L00F (2 mΩ 1 % 5 W) | Bourns | current sense | https://www.bourns.com/docs/Product-Datasheets/CSS2H-2512.pdf |
| F101 | 0456025.ER (25 A NANO2) | Littelfuse | main fuse | Littelfuse 456 series datasheet rev. 04/27/16, distributor copy https://www.farnell.com/datasheets/2282567.pdf |
| R102, R103 | PWR263S-35-R400F (0.40 Ω, D²PAK) | Bourns | load resistors | https://www.bourns.com/docs/Product-Datasheets/PWR263S-35.pdf (REV. 12/20) |
| U102 | TLV7031DBVR | Texas Instruments | load on-time comparator | https://www.ti.com/lit/ds/symlink/tlv7031.pdf (SLVSE13J) |
| J101 | 39-28-1083 (Mini-Fit Jr 5566-08A, 2×4) | Molex | pack connector | **not read** — https://www.molex.com/en-us/products/part-detail/39281083 (numbering taken from the KiCad footprint) |
| U201 | BQ25887RGER | Texas Instruments | 2S charger | https://www.ti.com/lit/ds/symlink/bq25887.pdf (SLUSD89B) |
| U202 | TPS259474ARPWR | Texas Instruments | dock eFuse | https://www.ti.com/lit/ds/symlink/tps25947.pdf (SLVSFC9C, rev. May 2026) |
| D201 | SMAJ6.0A | Bourns | dock 5 V TVS | https://www.bourns.com/docs/Product-Datasheets/SMAJ.pdf |
| D202 | TPD1E10B06DPYR | Texas Instruments | data contact ESD | https://www.ti.com/lit/ds/symlink/tpd1e10b06.pdf (SLLSEB1G) |
| U203 | SN74LV1T04DBVR | Texas Instruments | VDOCK-powered TX inverter | https://www.ti.com/lit/ds/symlink/sn74lv1t04.pdf (SCLS738E) |
| U204 | SN74LVC1G17DBVR | Texas Instruments | RX buffer (Ioff) | https://www.ti.com/lit/ds/symlink/sn74lvc1g17.pdf (SCES351Y) |
| Q201, Q202, Q305 | BSS138 | onsemi | data driver, dock detect, LED flash | https://www.onsemi.com/download/data-sheet/pdf/bss138-d.pdf |
| L201 | XAL4020-102MEB | Coilcraft | charger inductor | https://www.coilcraft.com/getmedia/6adcb47d-8b55-416c-976e-1e22e0d2848c/xal4000.pdf (doc 806, rev. 02/25/26) |
| TH201 | NCP15XH103F03RC | Murata | charger TS NTC | **not read** (Murata refused); B25/50 = 3380 K is [BK] |
| U301, U302 | TPS629210DRLR | Texas Instruments | 3V3_MAIN, 3V3_SUP bucks | https://www.ti.com/lit/ds/symlink/tps629210.pdf (SLVSGE0B) |
| U303 | TPS62130RGTR | Texas Instruments | 5V0_SYS buck | https://www.ti.com/lit/ds/symlink/tps62130.pdf (SLVSAG7F) |
| U304 | TPS70933DBVR | Texas Instruments | 3V3_AON LDO | https://www.ti.com/lit/ds/symlink/tps709.pdf (SBVS186H) |
| U305 | RV-3028-C7 | Micro Crystal | wake timer | https://www.microcrystal.com/fileadmin/Media/Products/RTC/Datasheet/RV-3028-C7.pdf (v1.3, 09.2021) and Application Manual rev. 1.4 (Nov 2021), fetched through the web-fetch service |
| U306 | SN74AUP1G74DCUR | Texas Instruments | ready latch | https://www.ti.com/lit/ds/symlink/sn74aup1g74.pdf (SCES644D) |
| F301, F302 / F303 | SF-1206S100-2 / SF-1206S300-2 | Bourns | regulator fuses 1 A / 3 A slow blow | https://www.bourns.com/docs/Product-Datasheets/SF-1206S.pdf |
| L301–L303 | XAL4020-222MEB | Coilcraft | buck inductors | as L201 |
| Q301–Q304 | BSS84 | onsemi | power-on P-MOSFETs | https://www.onsemi.com/download/data-sheet/pdf/bss84-d.pdf |
| D301, D302 | BAV70LT1G | onsemi | wake diode-OR | https://www.onsemi.com/download/data-sheet/pdf/bav70lt1-d.pdf (Rev. 15) |
| D303 | APT1608SGC | Kingbright | ready LED | https://www.kingbrightusa.com/images/catalog/SPEC/APT1608SGC.pdf (V.22B) |
| C301, C304 | C3216X7R1E475K160AC | TDK | buck input capacitors | value taken from SLVSGE0B Table 9-1 (TDK datasheet not opened) |

Temperature ranges: everything above is rated at least −40 to +85 °C (ambient or junction as the maker states it). The
MAX17320, RV-3028-C7, SN74AUP1G74, BQ25887 and APT1608SGC are the −40 to +85 °C parts; the rest go to +125 °C or more.

### 2.2 Generic passives

Resistors are 1 % thick film unless stated; capacitors X7R unless stated (C0G for C201). Voltage ratings: ≥ 25 V on
anything that touches `VBAT_*`, `VDOCK_IN` or the cell stack (8.4 V max, 6 V dock with clamp at 10.3 V); ≥ 10 V on 5 V and
3.3 V rails. Every value is listed with its reason in the pin tables.

---

## 3. Arithmetic for every chosen value

### Sheet 01
- **Sense resistor** R101 = 2 mΩ: fast over-discharge threshold limit 77.5 mV / 2 mΩ = 38.75 A (≥ 30 A needed), short-circuit
  limit 155 mV / 2 mΩ = 77.5 A (≥ 60 A needed), current register ±51.2 mV / 2 mΩ = ±25.6 A ≥ 21 A peak. Loss 9.2² × 0.002 =
  0.17 W (5 W part). [CALC]
- **Protector thresholds (non-volatile settings, not parts):** ODTH = −70 mV → 35 A, OCDLY = 5 → 70 µs + 5 × 977 µs = 4.96 ms;
  SCTH = −130 mV → 65 A, SCDLY = 2 → 70 µs + 2 × 61 µs = 192 µs. Register formulas from the nODSCTh / nODSCCfg
  descriptions. Comparator accuracy not read [UNVERIFIED].
- **Cell balancing resistors** R105, R106 = 100 Ω: CELL1 path 4.2 / (100 + 9) = 39 mA; BATTS path 4.2 / (2 × 100 + 9) = 20 mA;
  limit 100 mA (datasheet). Power 0.039² × 100 = 0.15 W → 1206 0.25 W.
- **IN filter** 10 Ω + 0.1 µF; **CP** 0.47 µF; **REG3/REG2** 0.47 µF; **CHG gate-source** 0.1 µF; **PCKP** 1 kΩ; **PFAIL** 1 kΩ;
  **AOLDO** 10 kΩ — all as the pin table states.
- **Load:** R = 2 × 0.40 = 0.80 Ω. I = 8.4 V / (0.80 + 0.009 Ω path) = 10.4 A; P = 86 W, 43 W per resistor; 100 ms → 4.3 J
  per resistor. Bourns pulse graph ≈ 8 J at 0.1 s (read off the graph image) → margin 1.9. At the 220 ms worst-case
  hardware limit: 9.5 J per resistor, past the end of the graph [UNVERIFIED].
- **On-time limit:** τ = 2.2 µF × 118 kΩ = 260 ms. Threshold while on = 3.3 × (1/100k + 1/1M) / (2/100k + 1/1M) = 3.3 ×
  11/21 = 1.73 V; t = τ × ln(21/11) = 0.647 τ = 168 ms nominal. C tolerance ±10 % and X7R temperature ±15 % (→ ×0.765 to
  ×1.265), R ±1 % → τ = 197–332 ms; divider 1 %, comparator offset ±8 mV and hysteresis → factor 0.631–0.662 →
  **t = 124–220 ms** (≤ 250 ms required, ≥ the 100 ms test). DC-bias loss of the 25 V 0805 capacitor at 3.3 V could
  shorten it by ~10 % more → ≥ 112 ms [EST].
- **Comparator input current** at the falling edge: 3.3 V / 10 kΩ = 0.33 mA ≤ 10 mA abs max.

### Sheet 02
- **eFuse UVLO:** 1.20 V × (255 + 100)/100 = 4.26 V on (4.20–4.34 V with 1.183–1.223 V), 1.09 × 3.55 = 3.87 V off.
- **eFuse OVLO:** 1.20 × (390 + 100)/100 = 5.88 V off (5.80–5.99 V), back on at 1.09 × 4.90 = 5.34 V.
- **eFuse current limit:** RILM = 3334 / I → 1.24 kΩ = 2.69 A (EC table spread ≈ ±11 % → 2.4–3.0 A).
- **eFuse slew:** CdVdt = 2000 / SR → 1.5 nF = 1.33 V/ms; ≈ 66 µF downstream → 88 mA inrush.
- **Charger input limit:** 1110 / 562 Ω = 1.98 A (KILIM spread at 1.5 A setting −7/+7 %).
- **CBSET:** 4.2 V / 100 Ω = 42 mA, 0.18 W. **MID:** 300 Ω (pin table).
- **TS window:** RT1 = 10.0 kΩ, no RT2, NTC 10 kΩ B3380 by the B-equation: R(0 °C) = 28.2 kΩ, R(45 °C) = 4.90 kΩ.
  T5 (34.375 % of REGN) at ≈ 44 °C (43.5–44.8 °C with threshold, NTC and resistor tolerances); T1 (73.25 %) at ≈ 1 °C
  (0.5–1.9 °C). The two-resistor formula (datasheet Eq 1–2) for 0 °C / 45 °C gives RT2 = −226 kΩ, i.e. impossible. [CALC]
- **Dock detect gate:** 5.0 × 100/110 = 4.55 V ≥ 1.5 V VGS(th) max.
- **Data:** R212 100 kΩ (R7) × ~3 pF input ≈ 0.3 µs, ≪ a UART bit at the dock's (unknown) baud rate.

### Sheet 03
- **TPS629210:** MODE 64.9 kΩ = Smart-CONFIG #13; VSET open = 3.3 V; accuracy −1.3/+1.1 % → 3.26–3.34 V. L 2.2 µH,
  C_OUT 2 × 22 µF (TI recommends 22 µF; 10–200 µF allowed with 2.2 µH), C_IN 4.7 µF 25 V.
- **TPS62130:** V_OUT = 0.8 × (1 + 107/20.0) = 5.08 V; with VFB 785.6–814.4 mV and 1 % resistors 4.96–5.20 V (PSM:
  4.93–5.25 V). Soft start 4.7 nF × 1.25 V / 2.5 µA = 2.35 ms. FSW high = 1.25 MHz. Dropout at 5.2 V, 1.2 A:
  (0.170 + 0.0387 + 0.023) Ω × 1.2 A = 0.28 V → 4.92 V.
- **TPS70933 EN divider:** 3.3/8.0 of VBAT_SYS → 2.27 V at 5.5 V, 3.47 V at 8.4 V (window 0.9–6.5 V); 1.05 µA.
- **EN_MAIN diode:** 3.3 V − 0.85 V (BAV70 at small current [BK]) = 2.45 V ≥ 1.03 V EN threshold.
- **Power-on clear:** 1 MΩ × 100 nF = 100 ms.
- **LED:** (3.0 − 1.9) V / 470 Ω = 2.3 mA [VF at 2 mA is EST; 2.2 V typ at 20 mA]; flash τ = 33 nF × 100 kΩ = 3.3 ms,
  on-time τ × ln(3.3 / 1.5…0.8) = 2.6–4.7 ms; average 2.3 mA × 0.003 = 7 µA → 23 µW at 3.3 V.

---

## 4. Unverified pins, ratings and assumptions

1. **MAX17320 2S cell wiring** (CELL1 = CELL2 = CELL3 joined to the mid-point, BATTS = top): read from the geometry of
   datasheet Figure 23 (positions of the dashed "short for 2S" and "short for 2S or 3S" links), not from a table.
2. **MAX17320 exposed pad (pin 25)**: not in the pin table; tied to device ground (BAT_NEG). Footprint EP size for
   outline 21-0139 not checked (KiCad `QFN-24-1EP_4x4mm_P0.5mm_EP2.7x2.7mm` assumed).
3. **MAX17320 "auto-retry"**: the part releases over-discharge / short-circuit faults on load removal (30 µA test into
   PCKP, release at > 1 V) or charger detection. There is no timed retry. Requires VBAT_SYS free of resistive loads
   below ≈ 33 kΩ.
4. **MAX17320 fast-comparator threshold accuracy** and the CP gate-drive voltage: electrical-characteristics page not
   reachable.
5. **Pack connector J101**: Molex drawing not read; MPN, terminal current (9–13 A per circuit [BK]) and pin numbering
   (taken from the KiCad footprint) are unverified.
6. **PWR263S-35 pulse energy** beyond 0.1 s, and availability of R400 in 1 % tolerance.
7. **CSD17573Q5B RDS(on) at VGS ≈ 3.2 V** (curve only).
8. **F101 nuisance-trip margin** against the charger's input ripple (fuse is very fast-acting, not time-lag).
9. **SF-1206S interrupting rating (50 A)** vs. the prospective short-circuit current of the battery bus.
10. **SMAJ6.0A symbol**: pin 1 (A1) taken as the cathode band (KiCad unidirectional TVS convention, D_SMA pad 1).
11. **NTC TH201** R-T values from the B-equation, not from Murata's table; thermal coupling to the cells.
12. **LV1T04 supply**: VDOCK may sit at up to 5.8–6.0 V before OVLO trips; the LV1T04 is recommended to 5.5 V (abs max
    7 V). Its inputs are 5 V-tolerant but no Ioff figure is given, hence the 100 kΩ series resistor (R7).
13. **TPD1E10B06** on the generic `D_0402_1005Metric` footprint (DPY land pattern not compared).
14. **RV-3028-C7 ordering string** (`RV-3028-C7 32.768kHz 1ppm-TA-QC`) composed from the ordering-code diagram.
15. **BQ25887 per-cell regulation without MID** — see contract issue C-1; the drawing is incomplete there.
16. Footprints still to be made (marked `footprint_todo`): Littelfuse NANO2 456 (F101), TPS25947 RPW (U202), dock
    contact pads (J201).

---

## 5. Departures from the design specification

| Design says | Drawn | Why |
|---|---|---|
| 0.82 Ω load (§B1.3) | 2 × 0.40 Ω = 0.80 Ω (10.4 A at 8.4 V instead of 10.2 A) | 0.82 Ω is not a value of a part with a published pulse rating; two parts halve the energy per part |
| "hardware RC time limit" | RC + nanopower comparator with hysteresis | a bare RC on a MOSFET gate would leave the 10 A switch in its linear region for tens of ms while turning off |
| F101 25 A **time-lag** | 25 A **very fast-acting** NANO2 (0456025.ER) | no surface-mount 25 A time-lag fuse found; see §4 item 8 |
| BQ25887 TS window 0–45 °C | ≈ 1–44 °C, RT1 only | the BQ25887 thresholds cannot place both ends at 0 and 45 °C with the standard divider; chose the narrower window |
| BQ25887 thermistor = pack NTC (implied) | own board NTC TH201 placed against the cells | both pack NTCs are used by the MAX17320 thermistor inputs (which switch their own pull-ups) and cannot be shared |
| 5V0_SYS = 5.0 V | 5.08 V nominal | keeps the worst case ≥ 4.9 V (§B1.6, R-20) |
| Dock data "open-drain driver powered from VDOCK" | LV1T04 inverter on VDOCK + BSS138 | a 74LVC1G07 on 5 V does not see 3.3 V as high (VIH = 0.7 × VCC) |
| MAX17320 "auto-retry" | load-removal release (the part's mechanism) | the part has no timed retry (§4 item 3) |
| Ready LED colour not stated | green | — |

---

## 6. Contract issues

**C-1 (blocking for a working charger) — the pack mid-point has no inter-sheet net.** The BQ25887 on sheet 02 regulates
each cell separately and needs the mid-point between the cells on its MID pin (through 300 Ω) and CBSET pin (through
the balancing resistor) (SLUSD89B pin table). The mid-point exists only at J101 pin 3 on sheet 01. Contract §5.2 lists
no such net, and capture packages may not add one. **Smallest conservative choice made:** on sheet 01 the mid-point
(`BAT_MID`) goes to the MAX17320; on sheet 02 MID and CBSET meet at a local node `BAT_MID_02` that connects to nothing
else, with a note on the sheet. As drawn, the charger would charge without per-cell regulation. **Proposed change:**
add `VBAT_MID` to §5.2 — "01 (J101 pin 3, pack mid-point) → 02 (U201 MID via 300 Ω, CBSET via 100 Ω); analog, cell
voltage; default: none". Both ends are WP-A; once added, rename `BAT_MID` / `BAT_MID_02` to the global net. It is not a
board-split net (01 and 02 are on the low-voltage board).

**C-2 — `3V3_MAIN` rail "used on" list.** Contract §4 lists `3V3_MAIN` as used on 04, 06, 07, 09, 15. WP-A also uses it on
**02** (the `DOCK_RX` Ioff buffer, which §5.2 itself places on 02 on 3V3_MAIN) and on **01** (the battery-load timer
comparator, so the load cannot run without the main controller's supply). **Proposed change:** add 01 and 02 to the
"Used on" column.

**C-3 — "auto-retry" (§9 WP-A).** The MAX17320 has no timed auto-retry; it releases a discharge fault when the load is
removed (> 1 V on PCKP with a 30 µA test current) or a charger is connected. That release works only if **no sheet puts
a resistive load below ≈ 33 kΩ (take 100 kΩ as the rule) on `VBAT_SYS`**. **Proposed change:** add that rule to §4
(`VBAT_SYS` notes) so WP-E (sheet 10) and WP-H (sheet 14, coil boost) respect it, and reword "auto-retry" to
"release on load removal".

**C-4 — MAX17320 communication shutdown (design §B11, F-10).** Confirmed from the datasheet: with `nConfig.COMMSH = 1`
the part shuts down (FETs off, device dark) when SCL and SDA are both held low longer than the shutdown timer — which is
the normal state of `I2C_M_*` with the main controller off. **Proposed change:** add to §5.1 (`I2C_M_*` row) and to the
firmware list: "MAX17320 nConfig.COMMSH = 0 (non-volatile), nProtCfg settings recorded in the battery configuration".

**C-5 — RTC_CLKOUT starts at 32.768 kHz.** The RV-3028-C7 delivers 32.768 kHz on CLKOUT until firmware stores the 1 Hz
setting in its EEPROM (App Manual §4.6.3). Sheet 06 (display inversion) and the LED flasher here see 32 kHz on a fresh
part. **Proposed change:** add to §5.1 (`RTC_CLKOUT` row) "1 Hz only after the one-time EEPROM setting; 32.768 kHz on a
new part" and a firmware requirement.

No contract name was changed, added or removed. R11 check for every output on WP-A sheets: `GAUGE_ALRT_N`,
`CHGR_INT_N`, `RTC_INT_N`, `DOCK_DET_N` (open drain, pulled to "inactive"); `DOCK_RX` (only meaningful when docked);
`RTC_CLKOUT` (timing only); the rails. None of them can be read as "safe to charge", "held", "permitted" or "sequence
allowed". The battery load reads "off" when open or unpowered.

---

## 7. Cross-sheet requirements other packages must respect

- `VBAT_SYS`: no resistive load below 100 kΩ (C-3). **Issue 3: tightened to 1 MΩ (D-66, contract §4).**
- Firmware (WP-B / firmware team): MAX17320 non-volatile settings (thresholds in §3, COMMSH = 0, AOLDO disabled, 2S,
  two 10 kΩ thermistors, current-sense 2 mΩ); BQ25887 charge current, 4.2 V per cell, JEITA registers; RV-3028 CLKOUT
  1 Hz stored in EEPROM; battery-load pulse ≤ 100 ms and not repeated rapidly (doubt 3).
- `EN_5V0` before any 5 V load (F-2) — unchanged; 5V0_SYS rises in 2.35 ms after EN.

---

## 8. Things a reviewer should check first (my own top five doubts)

1. **C-1, the missing mid-point net:** the charger cannot work as drawn until the contract adds `VBAT_MID`. Also check
   whether the MAX17320 and BQ25887 both balancing is wanted (one of them should probably be disabled in configuration).
2. **MAX17320 2S pin wiring and fault release:** the 2S connection was read from a figure, and the "auto-retry" the design
   wants is really "release when the load is removed" — check that the battery bus can satisfy that (all sheets on
   `VBAT_SYS`), or whether a short on the bus can leave the device dark until docked.
3. **Battery-load time limit:** it limits a stuck-high command to 124–220 ms but re-triggers on every rising edge; a
   command line toggled quickly by faulty firmware could run the 86 W load at up to about half duty and overheat the
   resistors. A non-retriggerable latch, a thermal fuse, or a firmware rule would close this.
4. **Fuses:** F101 is fast-acting instead of time-lag; the 1 A / 3 A regulator fuses have a 50 A interrupting rating on a
   bus that can deliver more; check coordination with the MAX17320 short-circuit trip (65 A after 192 µs).
5. **Load resistor pulse rating beyond 0.1 s and the TS window from an equation:** both rest on data I could not fully
   read (Bourns graph ends at 0.1 s; Murata R-T table not opened).

---

## 9. Totals

Counted from the pin tables; the netlist round trip of the build checks the same 90 nets and 391 pin connections.

| Sheet | Parts | Nets on the sheet (incl. rails) | of which local | Pin connections |
|---|---:|---:|---:|---:|
| 01_power_battery | 36 | 31 | 23 | 111 |
| 02_power_dock | 42 | 32 | 21 | 116 |
| 03_power_rails | 64 | 38 | 20 | 164 |
| **WP-A total** | **142** | **90 distinct** (64 local + 26 rails / inter-sheet signals) | 64 | **391** |

No part is DNP. Three footprints are still to be drawn (§4 item 16).

---

## 10. Build result

`bash tools/build.sh` in the WP-A worktree, 27 November 2026: 73 unit tests OK; ERC 0 errors (+1 accepted until
04_main_mcu is captured: `I2C_M_SCL` has no driver yet), 7 accepted warnings (3 footprints still to draw, 4 global labels
whose other end is on the placeholder 04_main_mcu), 0 unaccepted; **netlist round trip exact: 90 nets, 391 pin
connections**; PDF written; symbol-library lint 0 errors, 1 warning (TPS25947 footprint not drawn yet).

Tooling note (not changed): the integration test writes to a fixed directory `~/.cache/ws-schgen-itest`, so two work
packages running `tools/build.sh` at the same time can make it fail ("Schematic file does not exist"); a rerun passes.

---

## Revision 0.3 (contract issue 3), 27 November 2026

> **NOT FOR CONSTRUCTION — unreviewed design.** Paper design only. Simulated figures are marked [SIM] and come from
> behavioural models; nothing has been built or measured.

### In plain language

Four things changed on the power sheets. **(1)** The charger on sheet 02 is now wired to the middle of the battery
pack through a new shared wire, `VBAT_MID`, so it can look after each of the two cells separately. Until now it
could not. **(2)** The "battery under load" test on sheet 01 has a new diode and two resistors. Faulty firmware that
switches the test on and off quickly can no longer keep the 86 W load running and overheat it. **The change order had
one value wrong here.** It asked for a 100 kΩ resistor at the comparator input, but the comparator chip has a
built-in protection diode that undoes the fix through 100 kΩ (14.5 W was still delivered in my check run). I used
4.7 MΩ instead, and the same check run shows it working (0.77 W at most). The design authority needs to confirm that.
**(3)** Sheet 01 now states the battery-rail rule: no steady load below 1 MΩ (D-66). **(4)** The battery protector's
stored settings are now written down: communication shutdown off; over-current limits set so the charger's own fuse
acts first; and the protector's own cell balancing switched off, so that only the charger balances. Sheet 03 has no
circuit change; only its notes were updated.

### Changes applied

| # | Sheet | What was done | Designators |
|---|---|---|---|
| A-1 | 01, 02 | Renamed the local `BAT_MID` (01) and `BAT_MID_02` (02) to the contract net **`VBAT_MID`** and added it to both `global` lists. On 01 it reaches J101 pin 3 and R106 (to U101 CELL1-3); on 02 it reaches R207 300 Ω (to U201 MID) and R208 100 Ω (to U201 CBSET). Replaced the sheet notes and the old contract-issue C-1 text | J101, R106, R207, R208 |
| A-2 | 01 | New **D101 BAS116LT1G**: anode `LT_NODE`, cathode a new local node `LT_R`. **R111 118 kΩ → 78.7 kΩ** 1 %, now `LT_R`–GND. New **R117 4.7 MΩ** 1 % `LT_NODE`–GND. **R112 10 kΩ → 4.7 MΩ** (the change order says 100 kΩ; departure, see below). Box title and note rewritten | D101, R111, R112, R117 |
| A-3 | 01 | Sheet comment: "`VBAT_SYS`: no steady resistive load below 1 MΩ from any sheet (contract §4, D-66). Lowest drawn load: R316 + R317 = 8.0 MΩ on 03 (complies)". The same rule replaces the old 100 kΩ wording on 03 and in the gauge note | — |
| A-4 | 01 | Configuration record in the gauge box (see Arithmetic) | U101 (configuration) |
| A-5 | 03 | No circuit change. Comment added: `3V3_MAIN` is used on 01 and 02 as well as 04, 07, 09 and 15 (contract §4) | — |

Sheet 01 now has 38 parts (36 plus D101 and R117); 02 and 03 stay at 42 and 64. D101 uses WP-D's
`aed_logic:BAS116LT1G` symbol unchanged (pin 1 A, pin 3 K, pin 2 NC, from onsemi BAS116LT1/D Rev 14), so
`aed_power.toml` needed no new symbol.

### Arithmetic

- **R111 (A-2).** The change order's figure: with the diode fitted, 118 kΩ gave 255.5 ms [SIM logic-relay §4.3].
  On-time scales with R111, so 255.5 × 78.7 / 118 = 170 ms [CALC]. My check run (below) gave **133.6 / 174.7 /
  220.9 ms** with C108 at ×0.765 / ×1.000 / ×1.265 (10 % tolerance plus 15 % X7R change over temperature) [SIM].
  Adding the comparator offset (±8 mV on a decay margin of about 1.73 − 0.8 = 0.9 V, so about ±1 %) and 1 % resistors
  gives about **131–224 ms** [CALC]. That is under the contract's 250 ms and above the 100 ms firmware pulse (FW-19).
  The top is 4 ms above the old 220 ms band. Pulse energy at 224 ms is 43 W × 0.224 s = 9.6 J per load resistor,
  still beyond the Bourns graph (an existing doubt, unchanged).
- **R117.** 4.7 MΩ × 2.2 µF = 10.3 s, the slow return of `LT_NODE` to 0 V after a falling edge.
- **R112 (departure).** The TLV7031 inputs are diode-clamped to V− (SLVSE13J §6.4.1). After each falling edge
  `LT_NODE` sits at about −3 V, and the clamp feeds current back into C108 through R112. With the change order's
  100 kΩ that is 2.8 V / 100 kΩ = 28 µA. During each high half-cycle D101 + R111 (78.7 kΩ, a similar resistance) take
  about the same charge back out. The two balance, so the timer re-arms on every cycle. With 4.7 MΩ the return
  current is 2.8 V / 4.7 MΩ = 0.6 µA, 60 times weaker than the R111 path. Input bias is 2 pA typical and doubles every
  10 °C, so about 0.13 nA at 85 °C; across 4.7 MΩ that is 0.6 mV, negligible against the 160 mV hysteresis step
  [CALC]. The input-clamp current is at most 0.6 µA, far below the 10 mA absolute maximum.
- **Check run.** Behavioural ngspice on a scratch netlist built from `sim/logic-relay/battery.cir` and its models,
  plus an ideal input-clamp diode (IS = 1e-14) at the comparator's + input. Not committed, because `sim/` is not
  WP-A's. Figures are average resistor power over 20–30 s, after a 4 s settle [SIM]:

  | R112 | Clamp modelled | 0.2 Hz | 0.5 Hz | 1 Hz | 2 Hz | 4 Hz | 10 Hz | 100 Hz |
  |---|---|---|---|---|---|---|---|---|
  | 100 kΩ | no (as logic-relay) | — | 0 | 0 | 0 | 0 | 0 | 0 |
  | 100 kΩ | yes | — | 3.4–4.2 W | 3.8–7.9 W | ≤ 13.9 W | 10.9–14.5 W | 10.1–14.5 W | 14.5 W (25 %) |
  | 1 MΩ | yes | 1.0–1.5 W | ≤ 2.5 W | ≤ 3.2 W | ≤ 4.0 W | ≤ 5.5 W | ≤ 5.8 W | — |
  | **4.7 MΩ** | yes | **≤ 0.77 W** | **≤ 0.01 W** | **0** | **0** | **0** | **0** | — |

  Duties run: 25/50/75 % for 100 kΩ, 10/25/50 % for 1 MΩ and 4.7 MΩ. Ranges are over the duties run. The stuck-high
  on-time is the same with 100 kΩ and 4.7 MΩ (174.65 ms). Issue 2 as drawn gave 14.5–22.7 W [SIM logic-relay §4.3].
- **A-4 protector settings** (MAX17320 datasheet 19-100749 Rev 12, 7/25: nODSCTh, nODSCCfg and nBalTh tables; 2 mΩ
  sense resistor):
  - `nODSCTh.ODTH` = 3: −77.5 + 3 × 2.5 = −70 mV = **35 A** nominal. With the OC/OD/SC comparator's ±1.2 mV offset
    and ±5 % gain error (EC table) the threshold is 65.3–74.7 mV = **32.6–37.4 A**. `nODSCCfg.OCDLY` = 5: 70 µs +
    5 × 977 µs = **4.96 ms**. The rule "≥ 35 A **or** ≥ 1 ms" is met by the delay (4.96 ms is far longer than the
    charger eFuse's 30 A / 0.48 ms), not by the threshold alone (whose minimum is 32.6 A).
  - `nODSCTh.SCTH` = 5: −155 + 5 × 5 = −130 mV = 65 A; with the same errors 122.3–137.7 mV = **61.2–68.9 A**
    (≥ 60 A). `SCDLY` = 2: 70 + 2 × 61 = 192 µs (plus up to 31 µs), ≥ 100 µs.
  - `nConfig.COMMSH` = 0. Per the datasheet, with COMMSH = 1 the part shuts down when SDA and SCL are both held low.
  - `nBalTh.BALCFG` = 0. Table 21: "When set to 0, cell balancing is disabled". The BQ25887 balancer (CBSET, R208) is
    the one kept, as the change order recommends. `nBalTh.Imbalance` (a charge fault on cell imbalance, 10 mV LSB) is
    a protection, not a balancer, and may stay enabled; its value is a firmware/configuration choice.
- **`VBAT_MID` reference offset.** The mid-point is measured from the pack negative (BAT_NEG), but U201's ground is
  system GND, on the other side of R101. The offset is the R101 drop: 9.2 A × 2 mΩ = 18 mV average, 42 mV at the 21 A
  peaks, and about 4 mV while only charging [CALC]. Small against a 4.2 V cell, but real.

### Datasheets consulted this round

| Part | Document | Status |
|---|---|---|
| BAS116LT1G (D101) | onsemi BAS116LT1/D **Rev 14, October 2026** (onsemi.com PDF): VR 75 V; IR ≤ 5 nA at 75 V / 25 °C, 80 nA at 150 °C; VF ≤ 0.9 V at 1 mA; SOT-23 style 8 (1 A, 3 K) | read |
| TLV7031 (U102) | TI **SLVSE13J** (September 2017, revised November 2024): inputs diode-clamped to V−; input current ±10 mA absolute maximum; "no phase inversion" beyond the rails (§6.4.1); IB 2 pA typical; VOS ±8 mV; hysteresis | read |
| MAX17320 (U101 configuration) | ADI **19-100749 Rev 12, 7/25**, from a copy of the maker's PDF already on this machine (analog.com timed out again): nBalTh Table 21, nODSCTh Tables 22–23, nODSCCfg Table 24, comparator EC rows, nConfig.COMMSH; PCKP removal test 30 µA (typical only); release at 1 V (removal detection) and 1.5 V (SCTest SCDet) | read. **Note:** the change order and the design document cite Rev 5. Rev 12 says the same about PCKP (no minimum) |
| BQ25887 (U201 MID/CBSET) | TI SLUSD89B | **not re-opened this round**; MID and CBSET used as read for issue 2 |

### Problems with the change order (for the design authority)

1. **A-2: R112 = 100 kΩ does not work with the real comparator.** The logic-relay model of the TLV7031 has no input
   clamp, so its results ("LT_NODE sits at up to −3.3 V for seconds", 1.8–2.2 W) are optimistic. With the clamp that
   the datasheet describes, 100 kΩ gives 10–14.5 W at 4–100 Hz [SIM, WP-A check run], as bad as issue 2.
   **Drawn: 4.7 MΩ.** Second-round item S-6 should add the input clamp and settle the value.
2. **A-2 band.** 131–224 ms, a little above the issue-2 band (124–220 ms) but within the contract's 250 ms.
3. **A-4.** Once the comparator error is included, the over-discharge threshold alone is not ≥ 35 A (32.6 A minimum).
   The requirement is met by the 4.96 ms delay instead. Source revision: Rev 12 (7/25), not Rev 5.
4. **A-1.** `VBAT_MID` carries the R101 drop as an offset (up to 42 mV, see above).

### Failure modes of the new and changed parts

| Part | Fault | Effect | Detected? |
|---|---|---|---|
| D101 | open | `LT_NODE` decays only through R117, so a stuck-high `BAT_LOAD_EN` holds the load on for about 4.7 MΩ × 2.2 µF × ln(3.3/1.73) ≈ **6.7 s** [CALC] (86 W, about 290 J per resistor): the resistors overheat. Better than issue 2's open-R111 case (no limit at all), but still bad | No; only a stuck command exposes it. Firmware's ≤ 100 ms pulse (FW-19) avoids it |
| D101 | short | Back to the issue-2 circuit with 78.7 kΩ: stuck-high limit about 0.647 × 173 ms ≈ 112 ms nominal; toggling protection lost | Latent |
| R111 | open | Same as D101 open | Same as D101 open |
| R117 | open | The node returns from negative only through R112 and the clamp (similar time constant): no functional change | Latent, harmless |
| R117 | short | `LT_NODE` held at 0 V: the load never switches on | Yes: the load test shows no sag |
| R112 | open | Comparator + input floats (as in issue 2): output undefined, the load could stay on | No. Existing risk; a high-value pull-down on `LT_IN` would close it (not drawn, not in the change order) |
| R112 | short | The clamp takes the full −3.3 V step, limited only by the `BAT_LOAD_EN` driver (may briefly exceed 10 mA); toggling protection lost | Latent |
| `VBAT_MID` | open between 01 and 02 | The charger loses per-cell regulation (back to issue 2's state); the MAX17320 still sees both cells and still protects | Gauge cell voltages in firmware |
| `VBAT_MID` | short to GND / `VBAT_PACK` | Bottom or top cell shorted through R207 300 Ω / R208 100 Ω (≤ 42 mA) and R106 100 Ω | Gauge cell-imbalance and cell voltages |

### Still to be simulated or measured

- S-6 with a TLV7031 input-clamp model: the on-time band, and toggling power with R112 at 100 kΩ versus 4.7 MΩ.
  Also the BAS116 forward voltage at 10–30 µA over −20 to 85 °C (not in the datasheet table; only the Figure 2 curve).
- C108 X7R loss at 3.3 V DC bias (old [EST] about 10 %), which moves the bottom of the band toward about 120 ms.
- The MAX17320 minimum PCKP test current (from ADI; design O-5) and the comparator threshold over temperature.
- BQ25887 behaviour with the `VBAT_MID` offset during high-current discharge (it is not charging then, but MID is
  still sensed).

### Build result (revision 0.3)

`bash tools/build.sh` in this worktree, 27 November 2026, first run: 73 unit tests OK; ERC 0 errors, 21 accepted
warnings, 0 unaccepted; **netlist round trip exact: 719 nets, 3,673 pin connections**; PDF written; symbol-library
lint 0 errors. `python3 tools/xcheck.py`: `VBAT_MID` is on 01 and 02; the `3V3_MAIN` users are 01, 02, 03, 04, 07, 09
and 15, matching contract §4. No WP-A net is single-ended. The whole-design PDFs were regenerated but are left out of
this commit because they also contain every other package's sheets.
