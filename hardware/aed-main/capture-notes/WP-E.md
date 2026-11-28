# WP-E capture notes — sheet `10_hv_charger` (high-voltage charger)

Captured 27 November 2026 against contract issue 2 and `docs/HARDWARE_DESIGN.md` v0.2; **revised the same day to
design revision 0.3 / contract issue 3 — see the section "Revision 0.3" at the end, which supersedes §3.6, §3.7 and
§8 where they differ.**

> **NOT FOR CONSTRUCTION — unreviewed design.** Drawn with AI assistance. It has not been reviewed by a qualified
> high-voltage or medical-device engineer, and nothing in it has been built, simulated in circuit form or
> measured. This sheet creates about 2,200 V. These notes contain no assembly, test or bring-up steps and no
> transformer winding recipe (SAFETY.md §2a, CLAUDE.md rule 2). Nothing here is safe or ready for use on a
> person or animal.

Tags as in the design document: **[CALC]** arithmetic shown here; **[EST]** estimate with its basis;
**[UNVERIFIED]** not confirmed against a maker's document; **[SIM]** needs circuit simulation or measurement.
Nothing in this file is simulated.

Files: `hardware/aed-main/specs/10_hv_charger.toml` (pin tables), `hardware/lib/src/aed_charger.toml` (custom
symbols), generated `hardware/aed-main/10_hv_charger.kicad_sch`, `hardware/lib/aed_charger.kicad_sym`.

---

## 1. What is on the sheet, in plain language

This page turns the 2-cell battery (5.5–8.4 V) into about 2,000 V for the storage capacitor. An electronic fuse
(TPS25982) switches the battery onto a local supply called VTRANS only while the interlock sheet allows a charging
session; it starts gently so the battery is not jolted, and it disconnects in well under a millisecond if anything
downstream shorts. A small comparator watches VTRANS and, only once it is high enough for long enough, lets the
interlock's "charge now" signal reach the charger chip (LT3751). That chip repeatedly stores energy in a
made-to-order 1:50 transformer and releases it through two high-voltage diodes in series and a fusible resistor
into the capacitor. In adult mode (and whenever the mode switch is absent, open or undecided) the chip simply runs
until the main controller stops it or its own transformer-side voltage trip fires. In child mode a precision
resistor chain makes the chip regulate itself at about 1,280–1,370 V, so a child charge cannot go much higher even
if every piece of software fails; the page reports to the main controller which ceiling is really active. If either
discharge path is not proven held off, the interlock removes both the supply and the charge signal (D-25).

## 2. Parts

All datasheets read 27 Nov 2026. "Not opened" = the ordering code is a plausible catalogue part but its maker's
datasheet was not read in this capture; ratings marked [UNVERIFIED] and a `Verify` field is on the part.

| Ref | Part (MPN) | Maker | Datasheet (version) | Read? |
|---|---|---|---|---|
| U1001 | LT3751IUFD#PBF | Analog Devices | analog.com/media/en/technical-documentation/data-sheets/lt3751.pdf — **analog.com refused automated access** (curl HTTP/2 error 92, also over HTTP/1.1); read rev D (12/17, "3751fd", 34 pp., author field "Analog Devices") from the mirror https://static.chipdip.ru/lib/439/DOC011439824.pdf | read in full (pins, EC, operation, applications) |
| U1002 | TPS259824ONRGER | Texas Instruments | ti.com/lit/ds/symlink/tps25982.pdf, SLVSEI3D (rev. May 2026) | read (pins, ratings, EC, dVdt, ILIM, ITIMER, fault response, package outline RGE0024M) |
| U1003 | TLV3012BIDBVR | Texas Instruments | ti.com/lit/ds/symlink/tlv3012.pdf, SBOS300C (Apr 2023) | read (pins, EC B version, orderables) |
| U1004 | SN74LVC1G97DBVR | Texas Instruments | ti.com/lit/ds/symlink/sn74lvc1g97.pdf, SCES416O (Nov 2026) | read (pins, function table, Schmitt thresholds) |
| U1005 | SN74LVC1G04DBVR | Texas Instruments | ti.com/lit/ds/symlink/sn74lvc1g04.pdf, SCES214AF (Oct 2025) | read (pins, Ioff) |
| U1006 | SN74LVC1G07DBVR | Texas Instruments | ti.com/lit/ds/symlink/sn74lvc1g07.pdf, SCES296AG (Oct 2025) | read (pins) |
| Q1001, Q1002 | BSC070N10LS5ATMA1 | Infineon | infineon.com Infineon-BSC070N10LS5-DataSheet-v02_02-EN.pdf, Rev 2.2 (2023-03-08) | read (pins, ratings, RDS(on), Qg, thermal); ordering suffix not confirmed |
| Q1003, Q1004 | BSS138 | onsemi | onsemi.com bss138-d.pdf, Rev 7 (Apr 2024) | read (pins, IDSS, RDS(on)) |
| T1001 | made to order | — | `docs/HARDWARE_DESIGN.md` §B4.5 (requirement) | terminals defined by our symbol |
| D1001, D1002 | UX-F5B | Sanken | semicon.sanken-ele.co.jp/sk_content/ux-f5b_ds_en.pdf, UX-F5B-DSE Rev 1.6 (Nov 2022) | read |
| R1004 | **not selected** | — | `docs/HARDWARE_DESIGN.md` §14 requirement | no catalogue part found |
| R1001 | WSK25125L000FEA (5 mΩ, 1 %, 1 W, Kelvin) | Vishay Dale | vishay.com/docs/30108/wsk2512.pdf, rev 11-Dec-2023 | read |
| R1027–R1040 | TNPV12102M87BEEA (2.87 MΩ 0.1 % 25 ppm/K, 1210) | Vishay Draloric | vishay.com/docs/28881/tnpve3.pdf, rev 29-Apr-2026 | read (ranges, Umax 1,000 V, P70) |
| 0.1 % 0603 resistors | TNPW0603…BEEA | Vishay Dale | vishay.com/docs/28758/tnpw_e3.pdf | not opened (ordering codes per the TNPW scheme) |
| 1 % resistors | CRCW0603/1206 | Vishay Dale | vishay.com/docs/20035/dcrcwe3.pdf; HP: docs/20043 | not opened |
| F1001 | 0453015.MR (NANO2 453, 15 A, 65 V) | Littelfuse | littelfuse.com 453 series page | not opened (Littelfuse refused; distributor listings only) |
| L1001 | XAL1010-102MEB (1.0 µH) | Coilcraft | coilcraft.com XAL1010 page | not opened |
| D1003 | ES3D-E3/57T | Vishay | vishay.com/docs/88584/es3a.pdf | not opened |
| D1004 | SMCJ45A | Littelfuse | SMCJ series datasheet | not opened (figures from memory) |
| D1005 | SMBJ12A | Littelfuse | SMBJ series datasheet | not opened |
| D1006 | 1N4148WS-7-F | Diodes Inc. | diodes.com ds30098.pdf | not opened |
| C1009, C1010 | 16SEPF330M (330 µF 16 V polymer) | Panasonic | OS-CON pages | not opened — requirement stated on the part |
| Ceramics | Murata GRM… codes as in the spec | Murata | murata.com product pages | not opened |
| TP1001–TP1004 | bare pads | — | — | — |

## 3. Programming arithmetic [CALC]

### 3.1 LT3751 lockouts (datasheet: V = 1.225 V + 50 µA × R; pin threshold 1.195–1.255 V, pin current 48.5–51.5 µA; resistors 0.1 %)

| Function | R | Low / nominal / high | Against |
|---|---|---|---|
| UVLO1 (VTRANS low) | 69.8 kΩ | 1.195 + 48.5 µ × 69.73 k = **4.58** / 4.72 / 1.255 + 51.5 µ × 69.87 k = **4.85 V** | VTRANS min for RVTRANS 4.75 V |
| OVLO1 (VTRANS high) | **154 kΩ** (contract 155 k not E-series) | **8.66** / 8.93 / **9.19 V** | pack max 8.4 V: 0.26 V margin |
| UVLO2 (VCC low) | 71.5 kΩ | **4.66** / 4.80 / **4.94 V** | VCC min 4.75 V; 5V0_SYS tolerance — **see contract issue C3** |
| OVLO2 (VCC high) | **87.6 kΩ** (contract 87.5 k not E-series) | **5.44** / 5.61 / **5.77 V** | 5V0_SYS ≤ 5.1 V [EST ±2 %] |

### 3.2 Comparator-reference resistors and current limit

- RVTRANS = RVOUT = 40.2 kΩ, RDCM = 18.2 kΩ = 0.45 × 40.2 k (datasheet Table 2 row "5 to 80" for ΔVDRAIN; VTRANS
  5.3–8.4 V, ΔVDRAIN = (Vout + drops)/50 ≤ 46 V). RBG = 931 Ω → trip current 0.98/931 = 1.05 mA (datasheet: best
  accuracy 0.1–2 mA).
- Sensed trip level: Vout + Vdrops = 0.98 × N × RVOUT/RBG = 0.98 × 50 × 40,200/931 = **2,115.8 V**. With RVOUT ≠ RVTRANS
  by ±0.2 % the datasheet's second formula adds N × VTRANS × 0.002 = ±0.84 V.
- Current limit: 106 mV (100–112 mV) / 5.0 mΩ = **21.2 A** (with the 1 % resistor 19.8–22.6 A) plus the 180 ns
  propagation delay × VTRANS/LPRI = 180 ns × 8.4 V/6.75 µH = +0.22 A → **19.8–22.9 A**. Transformer requirement
  "≤ 10 % L loss at 25 A" covers it. Sense inductance error VTRANS × L_RS/L_PRI = 8.4 × 1 nH/7.5 µH = 1.1 mV (1 %).
- LPRI check (datasheet: LPRI ≥ 3 µs × VOUT/(IPK × N) so the VOUT comparator has time to see the flyback):
  3 µs × 2,200/(19.8 × 50) = **6.67 µH** against the transformer's minimum 6.75 µH — 1 % margin; at the trip's upper
  end with comparator overdrive (2,284 V) it needs 6.92 µH: **not met** (contract issue C2).

### 3.3 eFuse (TPS25982, SLVSEI3D)

- ILIM: R = 1460/(ILIM − 0.11) → 113 Ω gives **13.0 A** nominal. Datasheet tolerance rows (100 Ω: 12.85–15.99 A
  around 14.71; 182 Ω: 7.23–9.07 around 8.13) scaled to 113 Ω and 1 % resistor: **11.4–14.5 A** [CALC from
  interpolation]. Fast trip 2.1 × ILIM: ≤ 30.4 A, response 400 ns.
- Normal maximum through the eFuse = switch average current at full charge = IPK × V/(2(V + N × VT)) =
  22.9 × 2,000/(2 × (2,000 + 50 × 5.33)) = **10.1 A** (cold 5.5 V pack, maximum current limit). Margin to the lowest
  ILIM 11.4 A: 13 %. The LC filter's ringing on a load step is blanked by ITIMER (below).
- dV/dt: CdVdt = 4600/SR → 2.2 nF gives SR = 2.09 V/ms nominal; with IdVdt 2.0–6.33 µA (4.6 typ) and C ±10 %:
  **0.83–3.2 V/ms** → inrush into ≈ 722 µF effective (4 × 13 µF DC-biased ceramic [EST] + 660 µF polymer + 10 µF) =
  **0.6–2.3 A**; VTRANS at 8.4 V after 2.6–10.2 ms; at 5.3 V after 1.7–6.4 ms.
- ITIMER (blanking before the breaker acts): C × ΔV/I = 470 pF × 0.98 V/2.1 µA = **0.22 ms** (0.11–0.48 ms). Input
  filter: L 1 µH, C ≈ 722 µF → f0 = 5.9 kHz, Z0 = 37 mΩ, half-period 84 µs < 0.11 ms minimum blanking. TI recommends
  > 100 µs.
- Retry: RETRY_DLY 2.2 nF → 91.7 ms (Table 7-5), NRETRY open → 4 retries, then latched off (departure D4).
- EN: VTRANS_EN 0/5 V (abs max 7 V, recommended ≤ 6 V); on above 1.23 V; latch reset below 0.8 V.
- Thermal: 10.1 A² × 4.5 mΩ (max, hot) = 0.46 W × RθJA 34.6 K/W = 16 K rise.

### 3.4 VTRANS_GOOD (U1003 TLV3012B + U1004)

IN+ = VTRANS × Rb/(Ra + Rb) with positive feedback Rh from OUT; IN− = REF (1.242 V). Ra 309 k, Rb 100 k (0.1 %),
Rh 7.68 M (1 %). Gain g = 1 + Ra/Rb + Ra/Rh = 4.1302; hysteresis h = Ra/Rh = 0.04023.
- Rising: Vref × g = **5.13 V nominal**; band with REF 1.223–1.260 V (±1.5 %) ± drift 100 ppm/K × 45 K, offset ±9 mV
  (over temperature), internal hysteresis 2–8 mV, ratio ±0.2 %: **4.99–5.29 V**.
- Falling: Vref × g − 5V0_SYS × h = **4.93 V nominal**, band **4.76–5.07 V** (5V0_SYS 4.9–5.1 V).
- Checks: rising low edge 4.99 V > UVLO1 top 4.85 V (R-5 satisfied with 0.13 V margin). Rising top 5.29 V is below the
  unloaded pack at near-empty (≈ 6.0 V [EST]). Falling top 5.07 V is below the loaded VTRANS at a 5.5 V cold pack
  (5.5 − 10 A × ≈ 17 mΩ path [EST] = 5.33 V), margin ≈ 0.26 V less ripple (≈ 0.1 V [EST]).
- Delay: 100 k (1 %) + 22 nF (±10 %) charged from the comparator's push-pull output into U1004's Schmitt input
  (VT+ 2.16–2.74 V at 4.5 V, 2.61–3.33 V at 5.5 V; interpolated 2.39–3.04 V at 5.0 V): t = τ × ln(V/(V − VT+)) =
  **1.27–2.29 ms** (≥ 1 ms required). D1006 discharges the capacitor through the comparator output when VTRANS drops.
- AND: TI function Y = C̄·B + C·A; B (pin 1) = 0 → Y = A·C = CHARGE_EN · VTG_DLY.
- Behaviour: if VTRANS sags below 4.76–5.07 V during a charge, CHARGE falls (clean stop), and rises again 1.3–2.3 ms
  after VTRANS recovers — a CHARGE toggle, which also clears any LT3751 FAULT latch. Firmware should expect this.

### 3.5 Child ceiling (FB)

Top string 14 × 2.87 MΩ = 40.18 MΩ, bottom 36.9 kΩ, all 0.1 % (ratio ±0.2 %): k = 1 + 40.18 M/36.9 k = **1,089.9**
(design 1,090.4). FB bias current flows out of the pin, up to 300 nA, into R∥ = 36.87 kΩ: raises FB by up to 11 mV,
which only lowers the output. Two BSS138 off-state leakage (assumed ≤ 100 nA each at ≤ 1.4 V, 60 °C [EST, not
specified]) lowers FB and raises the output by up to 200 nA × 36.9 k/1.25 V = 0.6 %.

| Child threshold (true volts) | Formula | Band |
|---|---|---|
| Charge → regulation hand-over | (1.12 − 0.011) × k × 0.998 … 1.20 × k × 1.002 | **1,206 – 1,310** |
| Regulated point | (1.19 − 0.011) × k × 0.998 … 1.25 × k × 1.002 | **1,282 – 1,365** (1,374 with FET leakage) |
| No-load stop | 1.38 × k × 1.002 | **≤ 1,507** (≤ 1,516 with leakage) |

Adult (FETs on): FB = 55 µA × ≈ 3.5 Ω/2 ≈ 0.1 mV → charge mode.

### 3.6 Stack-up bands for the parts chosen [CALC; rectifier-drop model EST]

Firmware window tops from the design (§B4.4): adult **2,008.6 V**, child **1,192.4 V**.

Primary-side trip: sensed level 2,115.8 V nominal; comparator 0.955–1.005 V (±2.55 %), N ±1 %, RVOUT/RBG ±0.2 %,
RVOUT/RVTRANS mismatch ±0.84 V → sensed **2,036 – 2,197 V**. The datasheet also specifies a **VOUT comparator
overdrive of 20 mV typical, 40 mV maximum for a 2 µs pulse**; added to the threshold that raises the sensed top to
2,240 V (typ) / **2,284 V** (max). Our flyback pulse at the trip is 3.0–4.7 µs, so less overdrive is likely, but the
datasheet gives no figure for it [UNVERIFIED]. The true capacitor voltage at the trip is the sensed level minus the
drops in the secondary at the instant of peak secondary current (0.39–0.46 A): 2 × UX-F5B (VF 11 V typ / 14 V max at
0.35 A, 25 °C; modelled as 8–9 V knee + slope, +1.5 V cold, −2 V hot [EST]), R1004 (10 Ω), secondary winding
(assumed ≤ 20 Ω [EST]): **18.7 – 48.3 V**. The design assumed about 5 V.

| Layer | Adult (true V) | Child (true V) |
|---|---|---|
| Firmware target / top of at-plan window | 1,950 / **2,008.6** | 1,150 / **1,192.4** |
| Child FB hand-over (lowest) | — | **1,206** (13.6 V above the window) |
| Child FB regulated point | — | **1,282 – 1,365** |
| Child FB no-load stop (highest) | — | **1,507** |
| Primary-side trip, low / nominal / high | **1,988** / ≈ 2,088 / **2,197** (2,178 with the minimum drop credited) | same (above the child FB) |
| Primary-side trip high with max comparator overdrive | **2,284** (2,265 with minimum drop) | same |
| OVP comparator, sheet 08 (design value, not this sheet) | 2,138 – 2,182 | same |
| Capacitor rating (D-41) | 2,200 continuous; 2,420 for 10 s | same |

**Overlaps found:**
1. **Adult primary-trip low edge 1,988 V is 21 V below the window top 2,008.6 V** (all worst cases stacked: comparator
   at 0.955 V, N −1 %, high-VF diodes cold, 20 Ω winding, main sensor reading 1 % + 7 V low). On such a unit the
   hardware trip can stop an adult charge before the firmware reaches its target, and a CHARGE toggle repeats it:
   failure to shock. (Contract issue C1.)
2. **Adult primary-trip high edge 2,197 V sits at the 2,200 V rating, and 2,284 V with maximum overdrive is above it**
   (below 2,420 V). This layer only matters if firmware regulation, the firmware over-voltage limit and the OVP
   comparator have all failed. (Contract issue C1.)
3. Child: no overlap; child FB regulated top 1,365 V (design 1,377 V: the bias current only lowers the output).
4. The primary-trip band overlaps the OVP comparator band — harmless (either may act first).

### 3.7 Charge time [CALC] (t = C·V·(V + 2·N·VT)/(η·VT·IPK); VT = pack − 10 A × 17 mΩ path [EST])

| Case | to 1,950 V | to 2,008.6 V (window top) | child to 1,150 V |
|---|---|---|---|
| 8.4 V full, η 0.75, 100 µF, IPK 21.2 A | 4.13 s | 4.35 s | 1.73 s |
| 7.2 V, η 0.75 | 4.63 s | 4.87 s | 1.91 s |
| 5.8 V typical cold, η 0.75 | 5.47 s | 5.77 s | 2.20 s |
| 5.5 V cold sag (design §B4.2), η 0.65 | 6.59 s | 6.95 s | 2.64 s |
| 5.5 V, η 0.65, 105 µF, IPK 19.8 A (min) | **7.41 s** | **7.81 s** | 2.96 s |
| 5.5 V, η 0.60, 105 µF, IPK 19.8 A | **8.03 s** | **8.46 s** | 3.21 s |

≤ 7 s (D-29) is met at nominal and typical-cold conditions, **not at the tolerance corners** (worse than the
design's 7.2/7.8 s because the 0.17 V supply-path drop is now included). Firmware limit 12 s not reached. Owner
question 11 (§17) stands.

### 3.8 Pack and primary currents at full charge (2,000 V) [CALC]

| Pack | VT | Battery (eFuse) average | Primary RMS | Ripple to the bulk caps | Frequency | Input power |
|---|---|---|---|---|---|---|
| 8.4 V | 8.23 V | 8.8 A | 11.2 A | 6.9 A RMS | 42.9 kHz | 72 W |
| 5.8 V | 5.63 V | 9.3 A | 11.5 A | 6.7 A RMS | 31.0 kHz | 52 W |
| 5.5 V, IPK max | 5.33 V | **10.1 A** | **12.4 A** | **7.2 A RMS** | 27.4 kHz | 54 W |

Primary peak 22.9 A (from the bulk capacitors). Battery peak ≈ 10.1 A + filtered ripple (LC attenuation ≈ (27.4/5.9)²
≈ 21× → ≈ 0.35 A RMS) ≈ **10.6 A**. Pack protector thresholds (WP-A: OC ≥ 30 A ms, SC ≥ 60 A ≥ 100 µs) are above the
eFuse's 14.5 A / 30 A fast trip (R-16 coordination; see contract issue C6 for the margin at OC).

## 4. Stress against rating

| Part | Stress (worst) | Rating (source) | Margin / note |
|---|---|---|---|
| Q1001/Q1002 VDS | VTRANS 8.4 + reflected ≤ 45.7 + clamp overshoot: D1004 ≈ 75 V at 23 A [UNVERIFIED], +5 % hot, + D1003 forward recovery ≈ 4 V [EST] → **≈ 90 V** | 100 V; EAS 55 mJ single pulse (Infineon) | 10 %; avalanche rating as backup (leakage energy ≈ 20 µJ/cycle, but repetitive avalanche is not rated) [SIM] |
| Q1001/Q1002 current | 22.9 A peak total, 6.2 A RMS each (shared, not guaranteed) | ID,pulse 318 A; 61 A cont. at TC 100 °C | large |
| Q1001/Q1002 dissipation | 6.2² × 8.5 mΩ × 1.55 (hot) = 0.50 W + turn-off ½ × 52 V × 22.9 A × 10 ns × 43 kHz / 2 = 0.13 W each | RthJA 50 K/W (6 cm² Cu) | +32 K steady (charging is ≤ 7 s at a time) → ≈ 92 °C at 60 °C ambient; Tj 150 °C |
| Q1001/Q1002 gate | VCC 5 V (+0.5 V overshoot) | ±20 V; Qg ≤ 20 nC each at 4.5 V → 40 nC, driver 2 A → 20 ns | ok; RDS(on) specified at 4.5 V |
| R1001 | IPK² × R/3 × (V/(V + N·VT))² = 22.9² × 5 m/3 × (2,000/2,267)² = **0.68 W** | 1 W at 70 °C (WSK2512) | 68 % (design asked 2 W, departure D7) |
| D1003 | 22.9 A for ≈ 100 ns per cycle | ES3D 3 A avg, IFSM ≈ 100 A [UNVERIFIED] | — |
| D1004 | clamp energy ½ × Lleak × IPK² × Vcl/(Vcl − Vor) = ½ × 75 nH × 22.9² × 72/28 = 50 µJ/cycle → **2.2 W** near full charge | SMCJ: 1,500 W 10/1000 µs; steady-state rating [UNVERIFIED] | about 2 W for the last ≈ 1–2 s of each charge [SIM] |
| R1026 / C1023 snubber | 1 nF × 52² × 43 kHz = 0.12 W; C sees −9 to +80 V | 0.5 W (CRCW1206 HP) / 250 V C0G | ok |
| D1001, D1002 reverse | capacitor 2,420 V (test) + 50 × 9.19 V (OVLO1 top) = **2,880 V** each | 8 kV VRM; VZ ≥ 8.5 kV (Sanken) | 2.8× per diode; either alone blocks (R-15). Ringing at turn-on [SIM] |
| D1001, D1002 forward | average ≤ IPK/2N = 0.23 A (output near zero), peak 0.46 A | IF(AV) 0.35 A at TL ≤ 110 °C; repetitive peak **not stated**; IFSM 15 A (10 ms) | 65 %; ≈ 2 W per diode early in a charge (0.2 A × 10 V) [EST]; Tj max 120 °C |
| D1001, D1002 recovery | — | trr 70 ns typ, **150 ns max** | **LT3751 recommends < 100 ns**: max exceeds it (efficiency, charge time) |
| D1001, D1002 capacitance | — | **not stated** | counts against the 15 pF secondary-capacitance budget [UNVERIFIED] |
| R1004 | 0.26 A RMS (0.7 W) for ≈ 1 s early in a charge, ≈ 0.1 A later; both-diodes-short fault: 2,182 V/(10 Ω + winding) ≈ 200 A | requirement: open ≤ 10 ms at ≥ 5 J, ≥ 3 kV across the open element | **no part found** |
| R1027–R1040 | 2,200/14 = 157 V each (173 V at 2,420 V); 8.6 mW | TNPV1210: Umax 1,000 V; 0.33–0.5 W | large; creepage per element 0.6 + 0.005 × 173 = 1.5 mm (placeholder rule §B12) |
| R1017, R1018 (RVOUT, RDCM) | ≈ 2 V normally; up to 90 − 60 (pin clamp) = 30 V in a spike; 0.75 mA | 0603 thin film (TNPW0603 Umax 75 V [UNVERIFIED]); pin limit ±10 mA | ok |
| C1005–C1008 | 8.4 V (9.2 V at OVLO1) | 25 V | ok; ≈ 13 µF effective each [EST] |
| C1009, C1010 | 8.4 V; ≈ 3–3.5 A RMS each | 16 V; ripple rating **[UNVERIFIED]** | requirement written on the part |
| C1001, C1002 | VTRANS_IN spike: ½ × 1 µH × 14.5² = 105 µJ into 20 µF → +3.2 V (+ TVS 19.9 V max clamp) | 25 V | ok |
| L1001 | 10.1 A DC + 0.35 A RMS | Isat/Irms **[UNVERIFIED]** | requirement Isat ≥ 25 A, Irms ≥ 12 A |
| F1001 | 10.1 A for ≤ 7 s, repeated | 15 A (67 %) [UNVERIFIED temperature derating] | — |
| U1002 | 10.1 A, 0.46 W | 15 A continuous (recommended) | 67 % |
| U1001 | VCC 8 mA × 5 V + gate 40 nC × 5 V × 43 kHz = 49 mW | θJA 43 K/W | ok |
| T1001 | 12.4 A RMS primary for ≤ 8.5 s; 22.9 A peak | requirement 11.4 A RMS for 7 s; ≤ 10 % L loss at 25 A | **RMS 12.4 A exceeds the 11.4 A written in §B4.5** (cold pack, max IPK) — contract issue C8 |

## 5. Failure table for the charger (single part fails open or short)

Columns: **Over-charge?** = can the capacitor exceed its normal target because of this fault alone (and what stops
it). **Battery rail?** = can the fault pull down `VBAT_SYS` (and so the supervisor's alarm). **Runs when it must
not?** = can the charger run without `VTRANS_EN`/`CHARGE_EN` (D-25).

| Part | Fails | Over-charge? | Battery rail? | Runs when it must not? | Detected / result |
|---|---|---|---|---|---|
| F1001 | open | no | no | no | no charging; ChargeTimeout |
| F1001 | short | no | only if U1002 also fails (2 faults) | no | latent |
| L1001 | open | no | no | no | no charging |
| L1001 | short (turns) | no | no | no | more ripple in the eFuse; possible nuisance trip (retries) |
| D1005 | open | no | no | no | latent (spike protection lost) |
| D1005 | short | no | **VTRANS_IN shorted → F1001 blows** (fuse clears 15 A; pack protector may act first — R-16 coordination of a fuse vs the pack is not proven) | no | no charging; possible dark device (as fault 40) |
| C1001/C1002 | short | no | as D1005 short | no | as D1005 |
| U1002 | open / off | no | no | no | no charging |
| U1002 | IN–OUT short | no | no (L1001/F1001 still in series; a later primary short then reaches F1001 only) | **supply path always on**, but CHARGE is still gated by CHARGE_EN (one D-25 gate remains) | latent; VTRANS present without a session |
| U1002 | IN–GND short | no | **yes → F1001** (as D1005) | no | as D1005 |
| R1002 (ILIM) | open / short | no | no | no | ILIM 0 A (off) / ILIM-short fault (off): no charging |
| C1003 (dVdt) | open | no | no | no | fastest slew ≈ 10× inrush (≈ 7–20 A briefly) — may hit ILIM during start (current-limited start) [SIM] |
| C1004 (ITIMER) | open / short | no | no | no | open: no blanking → possible nuisance trips (retries); short: fault, off |
| C1027 (RETRY) | open / short | no | no | no | open: minimum-delay retries; short: latch-off (as the design) |
| R1005 (VTRANS_EN pull-down) | open | no | no | no (driver on 09 is push-pull; with the driver unpowered EN floats — eFuse EN has 0.1 µA leakage) [UNVERIFIED float behaviour] | latent |
| C1005–C1010 | short | no | through U1002: **eFuse limits and trips** (R-16) | no | no charging |
| C1005–C1010 | open | no | no | no | more ripple; slower |
| U1003 | output stuck high | no | no | CHARGE may rise before UVLO1 → LT3751 FAULT latch at start (R-5 race returns) | firmware retry clears it (toggle) |
| U1003 | output stuck low / unpowered | no | no | no | no charging (CHARGE held low) |
| R1006/R1007/R1008 | open/short | no | no | no | threshold moves: either never good (no charge) or good early (as U1003 stuck high) |
| C1013 | open | no | no | no | no delay; comparator alone still guards UVLO1 |
| C1013 | short | no | no | no | no charging |
| D1006 | open | no | no | no | slow reset of the delay after a sag (RC 2.2 ms) |
| U1004 | output stuck high | no | no | **CHARGE high without CHARGE_EN**, but the supply (VTRANS_EN, D-25) still gates it: with the eFuse off only the bulk's ≈ 28 mJ is available | latent until the interlock self-test; one D-25 gate remains |
| U1004 | output stuck low / open | no | no | no (R1011 holds CHARGE low) | no charging |
| R1010/R1011 | open | no | no | no | latent |
| U1001 | dead | no | no | no | no charging |
| U1001 | switching without CHARGE | no | no | needs VTRANS → gated by VTRANS_EN | latent |
| U1001 VOUT comparator dead | **primary trip lost** | no | no | layers left: firmware, firmware OV, OVP comparator |
| R1012–R1015 | open | no | no | no | UVLO open → pin sees no current → reads "low": FAULT, no charging [UNVERIFIED pin behaviour]; OVLO open → OV lockout trips → no charging [UNVERIFIED] |
| R1012–R1015 | short | no | no | no | UVLO short → lockout disabled (latent); OVLO short → pin at supply → FAULT, no charging |
| R1016 (RVTRANS) | open | no | no | no | DCM/VOUT references lost → no proper switching [UNVERIFIED] |
| R1017 (RVOUT) | open | **primary trip lost** | no | no | latent; other layers act |
| R1017 | short | no | no | no | RVOUT pin at the drain: current ≫ limit (pin damage) → trips at once → no charging |
| R1018 (RDCM) | open | no | no | no | DCM comparator never fires → 500 µs start-up timeouts → very slow charging, ChargeTimeout |
| R1019 (RBG) | short | **primary trip lost** (datasheet: RBG to ground disables the comparator) | no | no | latent |
| R1019 | open | unknown: no bias current [UNVERIFIED] | no | no | either trips at once (no charge) or trip lost |
| Q1001/Q1002 | open | no | no | no | half the switch capacity (thermal) / both open: no charging |
| Q1001/Q1002 | drain-source short | no | **primary short to GND through T1001: U1002 breaker trips in ≤ 0.48 ms (fast trip ≤ 30 A in 400 ns), 4 retries, then latched off**; F1001 backup | no | no charging; rest of the device and the supervisor's alarm unaffected (design fault 13) |
| R1022–R1025 | open | no | no | no | one FET floating/undriven (open gate resistor → that FET off; open pull-down → latent) |
| R1001 | open | no | no | no | sense lost → current limit never reached? CSP sees the source voltage → limit trips at once [UNVERIFIED] → no charge |
| R1001 | short | no | primary current unlimited until saturation → eFuse breaker (13 A average exceeded) | no | no charging; transformer stress |
| D1003/D1004 | open | no | no | no | leakage spike unclamped → Q1001/2 avalanche every cycle (EAS rated single-pulse only) → FET failure later → as Q short |
| D1003 + D1004 | short | no | VTRANS_SW tied to VTRANS (primary shorted) → no energy transfer | no | no charging |
| R1026/C1023 | open | no | no | no | more ringing, more clamp loss |
| C1023 | short | no | no | no | R1026 across the primary: overheats (≈ 8 V²/10 Ω = 6 W while switching) [EST] → open |
| T1001 | primary open | no | no | no | no charging |
| T1001 | primary–secondary insulation breakdown | 2 kV onto the primary side | possible | — | **hazard**; insulation requirement §B4.5 is the protection |
| T1001 | secondary open | no | no | no | no charging; drain sees large leakage energy → clamp |
| D1001 or D1002 | short (one) | no | no | no | latent: the other blocks the full voltage (R-15) — not detectable in normal use |
| D1001 or D1002 | open | no | no | no | no charging |
| D1001 + D1002 | both short (two faults) | capacitor discharges into the secondary | no | — | R1004 opens (design fault 14a) — part not found |
| R1004 | open | no | no | no | no charging |
| R1004 | short | no | no | no | latent (fuse function lost) |
| R1027–R1040 (one) | open | **child: FB → 0 → charge mode → child ceiling lost** (contained by primary trip ≤ 2,197/2,284 V and the OVP comparator; firmware sets the child dose) | no | no | adult unaffected; design fault 12 |
| R1027–R1040 (one) | short | child ceiling 7 % lower: hand-over ≥ 1,120 V — **below the child window top 1,192 V** → child charge may slow in regulation mode | no | no | possible slow child charge |
| R1041 | open | no (FB pulled up by the pin current → no-load stop) | no | no | child: no charging; adult: FETs hold FB at 0 |
| R1041 | short | child ceiling lost (FB = 0) | no | no | as a string open |
| C1024 | short | as R1041 short | no | no | — |
| Q1003 or Q1004 | open | no | no | no | the other FET keeps FB grounded: adult unaffected (that is why there are two) — latent |
| Q1003 or Q1004 | short | child ceiling lost (adult ceiling always; design fault 36, not read back) | no | no | child dose still set by firmware and checked by S |
| U1005 | output stuck low | no | no | no | **child ceiling in the adult position → adult charges stop at ≈ 1,300 V: no adult shock**; seen on MODE_HW_RB_N (≠ pole A) at switch-on → "not ready" |
| U1005 | output open / unpowered | no | no | no | R1043 → adult ceiling |
| U1005 | output stuck high | child ceiling lost | no | no | MODE_HW_RB_N stays high in CHILD → disagreement seen |
| R1042 | open | no | no | no | MODE_CHILD_HW floats when the contact is open → undefined ceiling (LVC input floating) — latent |
| R1043 | open | no | no | no | latent |
| U1006 | output stuck low | no | no | no | readback says "child" in ADULT → disagreement → "not ready" (false alarm) |
| U1006 | output open | no | no | no | readback always "adult" → a real child-ceiling fault in ADULT would go unseen (two faults) |

**Summary.** No single part on this sheet can, on its own, over-charge an **adult** capacitor beyond the layers on
other sheets (firmware OV, OVP comparator); several single faults remove the **primary-side trip** silently (RVOUT
open, RBG short, comparator dead). Several single faults remove the **child ceiling** (one string resistor open,
R1041 short, either FB FET short, U1005 stuck high) — contained by the adult-level layers and by firmware dosing.
No single part lets the charger run without **D-25**'s gating: the supply (eFuse, VTRANS_EN) and the CHARGE pin
(CHARGE_EN via U1004) are two independent gates, and a stuck part in either leaves the other. Faults that can pull
down `VBAT_SYS` are a short at the eFuse **input** (D1005, C1001/C1002, U1002 IN–GND) — downstream of F1001 but
upstream of the eFuse; coordination of F1001 against the pack protector is **not proven** (C6). A shorted primary
switch is limited by the eFuse (R-16). **Failure to shock:** U1005 stuck low (detected at switch-on), the
primary-trip low-edge overlap (§3.6), VTRANS_GOOD comparator stuck low, and the latched eFuse after 4 retries.

## 6. Unverified pins, ratings and assumptions

Pins:
- **TPS25982 exposed pads**: pad 25 = IN, pad 26 = GND inferred from the outline drawing geometry (the pin table says
  "Pad 1"/"Pad 2"). Must be confirmed before the footprint is drawn.
- **LT3751 QFN**: read from a mirror of rev D (analog.com refused); QFN numbering is not a rotation of the TSSOP
  (17 = RDCM, 18 = NC) — the pin table and the top-view drawing agree.
- **T1001 terminals** are defined by our symbol, not by any maker.
- Diode polarity of D1003–D1006 follows KiCad's convention (pad 1 = cathode band); maker datasheets not opened.
- BSC070N10LS5: KiCad's TDSON-8-1 footprint merges pins 5–8 and the tab into pad 5; our symbol pin 5 = drain.
- SN74LVC1G97: KiCad pin names IN1/IN0/IN2 = TI's B/A/C on pins 1/3/6 (checked against TI's pin table).

Ratings / data not confirmed: F1001 (453 series ratings, fast-acting), L1001 (XAL1010-102 Isat/Irms/DCR),
C1009/C1010 (polymer ESR, ripple, ordering code), all Murata ordering codes, D1003 (ES3D), D1004 (SMCJ45A VBR/VC and
steady-state power), D1005 (SMBJ12A), R1004 (no part), UX-F5B junction capacitance and repetitive peak current (not
stated), BSS138 off-state leakage at low VDS and 60 °C (not specified), LT3751 VOUT-comparator overdrive at 3–5 µs
pulses (only 2 µs given), LT3751 behaviour with RBG open and with UVLO/OVLO resistors open, TPS25982 operation below
TJ −15 °C (recommended range starts at −15 °C), ordering suffix ATMA1 of BSC070N10LS5, eFuse EN behaviour if
VTRANS_EN's driver is unpowered (pull-down present), DONE/FAULT open-collector outputs not back-powering the
unpowered LT3751 from the 3V3_MAIN pull-ups (abs max 24 V suggests no diode to VCC).

Assumptions [EST]: rectifier VF model; secondary winding ≤ 20 Ω; leakage inductance 1 % (75 nH); supply-path
resistance 17 mΩ; X7R 22 µF ≈ 13 µF at 8.4 V; 5V0_SYS 5.0 V ±2 %; pack ≈ 6.0 V unloaded when near empty;
efficiency 0.60–0.75 (design values).

[SIM] items: clamp and snubber values with the real transformer; LC-filter/charger interaction (filter output
impedance ≈ Q × Z0 ≈ 0.1 Ω against the converter's −V²/P ≈ −0.5 Ω input resistance: Middlebrook margin ≈ 5× [EST]);
eFuse ITIMER versus the LC step response at CHARGE restarts; child-mode regulation at 1:50 and ≈ 1,300 V (design §16
rank 6); FET current sharing at turn-off.

## 7. Departures from the design specification (and why)

| # | Design / contract | Captured | Why |
|---|---|---|---|
| D1 | FB top 10 × 4.02 MΩ 0.1 % HV thin film | **14 × 2.87 MΩ TNPV1210 0.1 % 25 ppm/K** (ratio 1,089.9 vs 1,090.4) | Vishay's high-voltage thin film (TNPV) stops at 3.01 MΩ (1210) / 2 MΩ (1206); no 4.02 MΩ 0.1 % thin-film part was found |
| D2 | OVLO1 155 k, OVLO2 87.5 k | **154 k, 87.6 k** | not E96/E192 values; thresholds move by −0.05 V / +0.005 V |
| D3 | one FB N-MOSFET | **two BSS138 in parallel (Q1003, Q1004)** | one FET failing open would otherwise put the child ceiling on every adult charge — an undetected failure to shock (the readback reads the gate drive, not the FET) |
| D4 | eFuse "latches off" | **4 automatic retries 92 ms apart, then latch-off** | a nuisance trip (e.g. input-filter ringing at a CHARGE restart) would otherwise stop all charging until firmware toggles CHG_SESSION, which F-3 does not ask for; a real short still ends latched off after ≈ 0.4 s |
| D5 | F1001 "15 A time-lag", drawn after the eFuse; LC filter after the eFuse | **0453015.MR fast-acting, upstream of the eFuse; L1001 before the eFuse** | the eFuse soft start removes inrush, so a fast fuse coordinates better (R-16); upstream it also covers an eFuse input-to-ground short; with L before the eFuse the eFuse sees only filtered current and no inductive kick at turn-off |
| D6 | VTRANS_GOOD > 5.0 V | **5.13 V rising / 4.93 V falling nominal** | the LT3751 UVLO1 band reaches 4.85 V and the comparator's own band is ±3 %: 5.0 V nominal would overlap UVLO1 |
| D7 | R1001 5.0 mΩ, 2 W | **WSK2512, 1 W** | the datasheet formula gives 0.68 W worst case; a 4-terminal 2512 is the stock Kelvin footprint. Raise if the reviewer wants the design's margin |
| D8 | bulk 4 × 47 µF X7R | **4 × 22 µF 25 V X7R** + 2 × 330 µF polymer | better DC-bias margin at 8.4 V; the polymers carry the ripple |
| D9 | "Primary clamp RCD/TVS at ≈ 75 V" | diode + SMCJ45A to VTRANS (≈ 75 V above VTRANS at 23 A) + RC snubber across the primary | datasheet recommends an RC snubber; the TVS keeps the drain ≈ 90 V |
| D10 | design net name `VTRANS_GOOD` | local nets `VTG_SNS`, `VTG_CMP`, `VTG_REF`, `VTG_DLY` | the net-class pattern `*VTRANS*` would put any net containing "VTRANS" in class PWR_HI (3 mm tracks) |
| D11 | — | added: DONE/FAULT 10 µs RC filters, CHARGE pull-down, FB filter capacitor, UVLO/OVLO pin capacitors, gate resistors and pull-downs, eFuse input TVS, four test points (§15) | datasheet recommendations and fail-safe defaults |

## 8. Contract issues (proposed changes)

- **C1 — Primary-side trip band (blocker-class for review).** With the real rectifier drops (≈ 19–48 V, not ≈ 5 V)
  and the datasheet's comparator overdrive, RBG 931 Ω gives a true trip of **1,988 – 2,197 V** (2,284 V with maximum
  overdrive). The low edge overlaps the adult window top (2,008.6 V) by 21 V; the high edge reaches the 2,200 V
  rating. No RBG value fits both: the band's width (≈ 10–12 %) exceeds 2,200/2,008.6 = 1.095. **Captured as the
  contract says (931 Ω).** Proposal: treat the primary trip as a last-resort layer and set it so it can never stop a
  normal charge — **RBG 909 Ω** (band 2,037 – 2,250 V, 2,339 V with max overdrive, all below the 2,420 V test rating),
  state in §B4.4 that the OVP comparator is the only layer below 2,200 V (as R-7 already said), update F-3 item 6, and
  ask ADI for the comparator overdrive at 3–5 µs. Also specify the secondary winding resistance in §B4.5.
- **C2 — Transformer inductance tolerance versus the LT3751 sensing time.** LPRI ≥ 3 µs × VOUT/(IPK × N) needs
  6.67 µH at 2,200 V and 6.92 µH at 2,284 V against a minimum of 6.75 µH. Proposal: §B4.5 Lpri 7.5 µH +10 %/−5 %
  (≥ 7.1 µH) or 8.2 µH ±10 %.
- **C3 — UVLO2 (71.5 kΩ) band 4.66–4.94 V versus 5V0_SYS.** The top overlaps 5V0_SYS at −2 % (4.90 V); the bottom is
  below the 4.75 V minimum VCC (R-20). LT3751 tolerance (±0.14 V) cannot fit 4.75–4.90 V. Captured as contracted.
  Proposal: WP-A sets 5V0_SYS to ≥ 5.05 V minimum (e.g. 5.15 V nominal — all 5V0_SYS loads tolerate 5.5 V [check
  per part]) and keeps 71.5 kΩ.
- **C4 — FB string part does not exist** (D1). Update contract §8 to 14 × 2.87 MΩ TNPV1210 0.1 % (or name another
  maker's 4.02 MΩ 0.1 % part).
- **C5 — Non-standard values** OVLO1 155 k → 154 k, OVLO2 87.5 k → 87.6 k (D2).
- **C6 — TPS25982 cold rating and coordination.** Recommended TJ starts at **−15 °C**; D-44 requires full-speed
  charging at −20 °C. Ask TI, or choose a −40 °C-rated eFuse/hot-swap controller. Also: the eFuse lets up to
  2.1 × ILIM ≈ 30 A flow for up to 0.48 ms before tripping — equal to the pack protector's OC threshold "≥ 30 A with
  millisecond delay"; WP-A should set OC ≥ 35 A or ≥ 1 ms. F1001 versus the pack protector for an eFuse-input short
  is not coordinated (the protector will usually act first: residual "dark device" like fault 40).
- **C7 — Firmware (F-3) additions.** (a) The charger can stop **silently**: if VTRANS sags or the eFuse trips, my
  VTRANS-good gate drops CHARGE before the LT3751's UVLO1, so neither DONE nor FAULT goes low. Add: "if the
  capacitor voltage does not rise for ≥ 200 ms with CHG_RUN high, lower CHG_SESSION and CHG_RUN for ≥ 2 ms and
  restart" (this also clears the eFuse latch). (b) F-3 item 1 "≥ 5 ms": the hardware now waits for VTRANS-good, which
  takes up to ≈ 9 ms after VTRANS_EN (soft start 1.7–6.4 ms to 5.3 V + delay 1.3–2.3 ms); firmware timeouts must
  allow ≥ 15 ms before expecting voltage to rise. (c) DONE low in adult mode means the hardware trip fired before the
  firmware target — report it. (d) There is no readback of VTRANS or VTRANS-good; consider a contract net
  `VTRANS_GOOD_RB_N` (open drain, 10 → 04) in a later issue.
- **C8 — Transformer RMS current.** §B4.5 says 11.4 A RMS; at a 5.5 V pack with the maximum current limit it is
  **12.4 A**. Update the requirement.
- **C9 — `HV_CAP_P` vs `HV_RECT`.** No change needed, but note for WP-F: the capacitor's charge current arrives
  through R1004 (10 Ω): the bleeder/sense strings on sheet 11 see HV_CAP_P directly.

## 9. Things a reviewer should check first (my top five doubts)

1. **The primary-side trip band (§3.6, C1)** — my rectifier-drop model, the meaning of the "VOUT comparator
   overdrive" spec at 3–5 µs pulses, and whether RBG should move to 909 Ω.
2. **TPS25982**: the −15 °C recommended junction minimum against D-44's −20 °C charging, the pad 25/26 = IN/GND
   pairing, circuit-breaker versus active-limiter choice, ITIMER blanking against the LC filter step, and the
   coordination margin with the pack protector (C6).
3. **The primary clamp margin**: about 90 V on a 100 V switch depends on SMCJ45A clamping figures I could not open,
   the leakage inductance (1 % assumed) and diode forward recovery — needs simulation with the real transformer.
4. **VTRANS_GOOD and UVLO2 margins**: VTRANS-good falling top 5.07 V vs ≈ 5.33 V loaded VTRANS at a cold pack
   (≈ 0.26 V margin before ripple), and the UVLO2 band against 5V0_SYS (C3). Either could stop charging in the cold.
5. **The rectifiers and R1004**: UX-F5B trr (150 ns max) exceeds the LT3751's < 100 ns recommendation and its
   capacitance is unstated — charge time and efficiency may be worse than the 0.60–0.75 assumed; and no HV fusible
   resistor rated ≥ 3 kV open-circuit was found.

## 10. Not done

- Footprints to be made (marked `footprint_todo`): TPS25982 RGE0024M (two exposed pads), T1001 (made to order),
  UX-F5B high-voltage axial land pattern (creepage across ≈ 2.9 kV), R1004 (part not chosen).
- Maker datasheets not opened for the passives, protection diodes, fuse, inductor and polymer capacitors (§2).
- No simulation of anything on this sheet.
- No layout; creepage figures are the placeholder rule of §B12.

## 11. Totals

- **93 parts**: 6 ICs (U1001–U1006), 4 transistors (Q1001–Q1004), 1 transformer, 6 diodes (D1001–D1006), 1 fuse,
  1 inductor, 43 resistors (R1001–R1043), 27 capacitors (C1001–C1027), 4 test points.
- **61 nets** on the sheet: 5 rails (`GND`, `5V0_SYS`, `VBAT_SYS`, `HV_RTN`, local rail `VTRANS`), 7 inter-sheet nets
  (`VTRANS_EN`, `CHARGE_EN`, `CHG_DONE_N`, `CHG_FAULT_N`, `MODE_CHILD_HW`, `MODE_HW_RB_N`, `HV_CAP_P`), 49 local nets
  (16 of them high voltage: `HV_SEC`, `HV_D_MID`, `HV_RECT`, `HV_FB_1`…`HV_FB_13`; plus the global `HV_CAP_P`).
  245 pin connections; netlist round trip matches exactly.
- No inter-sheet net was renamed, added or deleted.

---

## Revision 0.3

27 November 2026, against `docs/HARDWARE_DESIGN.md` v0.3, `contract.md` issue 3 and the change order
`rev3-changes.md` section WP-E (E-1…E-4). **NOT FOR CONSTRUCTION — unreviewed design.** Simulated figures are marked
[SIM] and come from hand-built models in `sim/rev3-wpe/` (new this revision) and `sim/charger-storage/` (CS); nothing
has been built or measured, and nothing here is an instruction to build, power or test anything.

### R3.1 In plain language

- **One resistor changed (R1019, 931 Ω → 876 Ω).** It sets the charger chip's own built-in voltage stop. That stop
  cannot be made accurate enough to sit between a normal charge (at most 2,008.6 V) and the capacitor's 2,200 V
  rating, and at the old value it could sometimes stop a normal adult charge early — a failure to shock. It is now a
  **last-ditch stop above the normal range**: somewhere between 2,116 and 2,316 V (2,409 V in the very worst case).
  Two other independent limits (the firmware's at 2,070 V and the latching hardware comparator at 2,073–2,142 V) act
  first; this one only matters if both have failed, and it then relies on the capacitor's new surge rating
  (≥ 2,420 V for ≥ 1 s). I re-did the arithmetic from the datasheet and got the same band as the simulation.
- **The made-to-order transformer's requirement is tighter**: it must now be at least 90 % efficient, its inductance
  may vary by +10 %/−5 % instead of ±10 %, and it must carry 12.4 A RMS.
- **Charge time, honestly:** about 4.2 s normally, about 5–6 s in the cold worst case *if* the transformer meets its
  90 % requirement — and possibly about 7.1 s (just over the 7 s target) in one cold corner that depends on a detail
  of the charger chip the datasheet does not explain. My earlier 7.4–8.0 s was too pessimistic: it guessed the
  losses of everything at 35–40 %; the simulation shows the circuit itself loses about 13 %.
- **One new problem for the design authority (P4 below):** the last-ditch stop ends one charge but does not stop the
  firmware from starting another. In the two-failure case it protects against, the firmware's own "charger seems
  stuck, restart it" rule (FW-18) would keep refilling the capacitor to the last-ditch level, for much longer than
  the 1 s the capacitor's surge rating covers. A firmware rule closes it.
- The four items left open last time: C7 is closed (it became firmware rule FW-18), C3, C4 and C6 stay open, each
  with what would settle it.

### R3.2 Changes applied to the sheet (designators)

| Change order | Designator | Before → after | Where |
|---|---|---|---|
| E-1 | **R1019** (RBG) | 931 Ω → **876 Ω 0.1 %**, `TNPW0603876RBEEA` (TNPW ordering scheme, not checked against the maker's page: `Verify` field) | spec part R1019 |
| E-2 | **T1001** | value "1:50 7.5uH +10/-5% (made to order)"; description: Lpri 7.5 µH **+10 %/−5 %**, **efficiency ≥ 90 % at 20 A peak, 30–45 kHz**, **12.4 A RMS**, **secondary winding resistance stated by the maker** (design assumes ≤ 20 Ω), turns ±1 % marked load-bearing; `Verify` records P2 and P3 | spec part T1001; library symbol `T_FLYBACK_1to50` value and description |
| E-3 | R1001 | kept 5.0 mΩ; `DocRef` records why 4.7 mΩ was not taken | spec part R1001 |
| E-4 | sheet notes | controller box text replaced (876 Ω band, last-resort layer, D-65); header comment now "≈ 2,000 V normal, up to 2,409 V last resort"; clamp note (reflected 46.7/48.5 V); rectifier note; FB string note (C4) | spec `ctrl`, `power`, `hv_out`, `fb` notes and `comments` |
| — (open items) | R1014, U1002 | `description`/`Verify` now say "OPEN (WP-E C3 / C6, O-16)" | spec |

No net was renamed, added or removed; part count stays 93; contract net names and levels unchanged.

### R3.3 Arithmetic: the primary-side trip with RBG 876 Ω [CALC; checked in `sim/rev3-wpe/l3_trip_rev3.cir`]

LT3751 rev D: Vout + V_drops = 0.98 V × N × RVOUT/RBG (sensed level); comparator 0.955 / 0.98 / 1.005 V over
temperature; overdrive 20 mV typ / 40 mV max for a 2 µs pulse. RVOUT/RBG = 40,200/876 = 45.890.

| Quantity | Formula | Result |
|---|---|---|
| Sensed, nominal | 0.98 × 50 × 45.890 | **2,248.6 V** |
| Sensed, low | 0.955 × 49.5 × 45.890 × 0.998 − 0.84 | 2,164.2 V |
| Sensed, high | 1.005 × 50.5 × 45.890 × 1.002 + 0.84 | 2,334.6 V |
| Sensed, high + 40 mV overdrive | 1.045 × 50.5 × 45.890 × 1.002 + 0.84 | 2,427.4 V |
| **True capacitor trip** (minus drops 48.3 / 33.5 / 18.7 V) | | **2,115.9 / ≈ 2,215 / 2,315.9 V; 2,408.7 V with max overdrive** |
| Monte Carlo, 200,000 uniform samples [SIM] | | 2,120–2,311 V; 2,396 V with overdrive |

This matches the change order and CS Q4 (2,116–2,316 V, 2,409 V). Checks against the design:
- **Never stops a normal charge:** low edge 2,115.9 − 2,008.6 = **107 V** above the window top (change order: ≥ 107 V). ✔
- **Above the rated voltage:** the low edge is below 2,200 V for some units and the **nominal ≈ 2,215 V is above it** —
  as intended for a last resort; the comparator (≤ 2,142 V) and firmware limit (≤ 2,098 V) sit below 2,200 V. ✔ (design text accurate)
- **Inside the required surge rating** (`docs/requirements/storage-capacitor.md`: ≥ 2,420 V for ≥ 1 s, ≥ 100 events):
  2,420 − 2,315.9 = **104 V** without overdrive, **11 V** at the very top with maximum overdrive. ✔ but thin at the top,
  and only for an event that ends within 1 s — see P4.
- Side effects checked: rectifier reverse voltage 2,408.7 + 50 × 9.19 = 2,868 V per UX-F5B (8 kV, 2.8×); FB string
  172 V per element (1,000 V part); R1004's ≥ 3 kV open-element requirement still covers it.
- **Inductance sensing rule** (datasheet: LPRI = 3 µs × VOUT/(IPK × N) "guarantees" the comparator sees the flyback;
  much higher VOUT "could result in a runaway condition"), lowest IPK 100 mV/5.05 mΩ = 19.8 A: needs **7.07 µH** at the
  2,335 V sensed top and **7.35 µH** at 2,427 V. The v0.3 minimum 7.125 µH gives a simulated flyback of **3.04 µs** at
  2,316 V true and **2.93 µs** at 2,409 V [SIM] (v0.2's 6.75 µH: 2.88 / 2.78 µs). See P2.
- **Clamp:** reflected voltage at the trip top (VOUT + drops)/N = 46.7 V (48.5 V with overdrive), was 45.7 V; the
  SMCJ45A standoff is 45 V, minimum breakdown 50.0 V [UNVERIFIED], ≈ 47.8 V cold [EST]. Simulated clamp energy at
  2,409 V with the cold-minimum TVS: 37.5 µJ/cycle against 21 µJ at 2,200 V with a nominal TVS; drain peak ≤ 76 V;
  delivered energy down 1 % [SIM]. Not a problem, recorded as P6.

### R3.4 Charge time, reconciled

| Source | What it assumed | To 1,950 V, worst corner |
|---|---|---|
| WP-E v0.2 §3.7 (mine) | lumped efficiency 0.60–0.65 for **everything** (switch, diodes, transformer), 5.5 V pack less 0.17 V path drop, 19.8 A peak, 105 µF; no simulation | **7.41–8.03 s** |
| CS Q3 [SIM] | per-cycle circuit simulation: switch, sense, diodes, snubber, clamp, path drop; **no transformer core or AC winding loss**; L −10 %; 6.66 MΩ standing load; no limit from the LT3751 clock | **5.29 s** (circuit efficiency 0.87) |
| This revision [SIM, `charger_rev3.cir`] | CS method plus a **transformer efficiency factor**, L **+10 %/−5 %**, standing load **2.80 MΩ** (S-12), and a pessimistic [EST] bound for the LT3751 one-shot clock | see below |

| Corner (5.5 V pack unless stated, 100 mV sense, R +1 %, hot switch, slow diodes, C +5 %) | to 1,150 V | to 1,950 V | to 2,008.6 V |
|---|---|---|---|
| Nominal 7.2 V, 25 °C, transformer 90 % | 1.71 s | **4.18 s** | 4.41 s |
| Typical cold 5.8 V, 90 % | 2.00 s | **5.01 s** | 5.28 s |
| Worst, L −5 %, no transformer loss (compare CS 5.29 s) | 2.11 s | 5.34 s | 5.63 s |
| **Worst, L −5 %, transformer 90 %** | 2.35 s | **5.94 s** | 6.27 s |
| Worst, L +10 %, 90 %, clock 38 µs (typical) | 2.42 s | 6.03 s | 6.36 s |
| Worst, L −5 %, transformer 75.6 % | 2.80 s | **7.10 s** | 7.50 s |
| Worst, L +10 %, 90 %, clock 32 µs (datasheet minimum), [EST] bound | 2.68 s | **7.08 s** | 7.47 s |
| Cold, highest peak current, L +10 %, 90 %, clock 32 µs, [EST] bound | 2.58 s | 7.14 s | 7.54 s |

**Honest range:** **≈ 4.2 s** (nominal) to **≈ 6.0 s** (cold worst corner) **if the transformer meets ≥ 90 %** and the
LT3751's one-shot clock does not shorten cycles; **up to ≈ 7.1 s** — just missing D-29's 7 s — if the transformer
reaches only ≈ 76 %, or if the clock limits long cycles at a cold pack the way my pessimistic bound assumes (P3).
Add ≈ 0.3–0.4 s to reach the top of the at-plan window. Both earlier numbers were "right for what they assumed":
mine over-counted the circuit losses, CS left out the transformer and the clock. Child (1,150 V): 1.7–2.8 s.

### R3.5 Open items from §8, re-examined against v0.3

| Item | v0.3 status | Result |
|---|---|---|
| **C3** UVLO2 band (4.66–4.94 V) vs `5V0_SYS` | v0.3 unchanged (71.5 kΩ, contract §8); §B1.6 now states `5V0_SYS` ≥ 4.9 V, and the datasheet's VCC minimum is 4.75 V (rev D EC table, read) | **Open.** The band (±0.14 V from the pin's ±30 mV threshold and ±3 % current) is wider than the 4.75–4.90 V gap, so no resistor on this sheet fits. Settled by either WP-A raising the `5V0_SYS` minimum to ≥ 5.0 V (TPS62130 set-point) **and** ADI confirming correct behaviour for 4.66 V ≤ VCC < 4.75 V, or the design authority accepting a cold-charging stop below 4.94 V. Marked on R1014. |
| **C4** FB string part (10 × 4.02 MΩ 0.1 % thin film is not made) | contract issue 3 §8 and design §B4.3 still say 10 × 4.02 MΩ | **Open (documentation only).** The sheet keeps 14 × 2.87 MΩ TNPV1210 (ratio 1,089.9). Settled by the contract/design adopting it, or naming a real 4.02 MΩ 0.1 % part. At the new trip top each element sees 172 V of its 1,000 V. |
| **C6** TPS25982 recommended only from TJ −15 °C | SLVSEI3D (May 2026) re-read: recommended TJ −15 to 125 °C; D-44 requires charging at −20 °C. The coordination half is closed by A-4 (pack OC ≥ 35 A or ≥ 1 ms, to be captured on 01) | **Open (cold rating).** Settled by a TI statement covering −20 °C, or by choosing a −40 °C-rated eFuse/hot-swap controller (the WP-A TPS25947 class is a candidate to check). Marked on U1002. |
| **C7** silent charger stop; firmware rules | (a) and (b) adopted as FW-18 / F-3 item 7 | **Closed** for (a), (b). (c) "`CHG_DONE_N` low in adult mode" is now more important and is restated as **P4**. (d) a `VTRANS_GOOD` readback net remains a proposal for a later contract issue (not drawn; no net added). |

### R3.6 Problems with the change order (for the design authority)

- **P1 — E-2 wording.** "7 s is missed only below ≈ 76 % overall" should read "below ≈ 76 % **transformer**
  efficiency (≈ 0.66 overall)", as §B4.2 and CS Q3 say. Confirmed by simulation: 75.6 % → 7.10 s.
- **P2 — §B4.5's reason for −5 % is computed for the old trip.** "Needs ≥ 6.92 µH at 2,284 V" is the 931 Ω band. With
  876 Ω the datasheet rule needs 7.07 µH at the no-overdrive top and 7.35 µH at the overdrive top; 7.125 µH meets the
  first by 0.7 % and misses the second by 3 % (flyback 2.93 µs, still longer than the 2 µs at which the overdrive is
  specified). **Captured as ordered (+10 %/−5 %).** Options: accept; tighten the lower limit to ≈ −2 % (hard to make);
  or raise the nominal (≈ 7.8 µH +10/−5 %), which worsens P3.
- **P3 — the LT3751 upper inductance rule was not checked by anyone.** Datasheet: LPRI < 38 µs / (IPK (1/VTRANS +
  N/VOUT)), or less than the expected output power results; the clock is 32–44 µs. At a 5.33 V cold VTRANS and 2,000 V
  this allows 6.57–7.80 µH (22.9 A) / 7.10–8.43 µH (21.2 A): the 7.5 µH nominal already exceeds it at the minimum clock,
  and the CS model does not include the clock. The datasheet does not say how the clock limits the cycle; my bound gives
  ≈ 7.1 s (R3.4). Settled by ADI applications engineering, or by running ADI's (encrypted, LTspice-only) LT3751 model.
  Not a safety issue (slower charge only), but it decides whether D-29 is met in the cold.
- **P4 — the last-resort stop can be held for longer than the surge rating covers.** In charge mode the trip sets
  DONE and ends *one* charge; nothing in hardware prevents the next. In the case L3 exists for (firmware limit and
  OVP comparator both failed — e.g. a wrong main reading), the firmware still regulates: F-3's 3 V restart, a
  VTRANS-good toggle, and especially **FW-18** ("no rise for ≥ 200 ms → drop and restart") would refill the capacitor
  to 2,116–2,409 V every few hundred milliseconds for as long as it holds, against a surge rating of ≥ 1 s per event.
  Proposal (firmware, not drawn): **in adult mode, `CHG_DONE_N` low means the last-resort trip fired — lower
  `CHG_SESSION`, do not restart, release the hold (dump), record a fault, "not ready"**; FW-18 must check
  `CHG_DONE_N` first. This uses a signal independent of the voltage reading. Alternatively state a cumulative
  duration in the capacitor's surge requirement.
- **P5 — nominal last-resort trip above the continuous rating.** ≈ 2,215 V nominal > 2,200 V. Consistent with D-65
  and §B4.4 ("last resort"), but §B4.6's wording "stops a charge at 2,116–2,316 V" should not be read as "below the
  rating". No change.
- **P6 — clamp margin at the trip top.** Reflected 46.7–48.5 V exceeds the SMCJ45A's 45 V standoff; simulated extra
  clamp loss ≤ 16 µJ/cycle, ≈ 0.7 W (benign) with the TVS figures from memory. D1004 unchanged; SMCJ datasheet to be
  read.
- **P7 — contract §8 still lists the FB string as 10 × 4.02 MΩ** (C4).

### R3.7 Datasheets consulted this revision

| Part | Document | Status |
|---|---|---|
| LT3751 | "LT3751 High Voltage Capacitor Charger Controller with Regulation", rev D (12/17, "3751fd"), copy saved earlier from the distributor mirror (analog.com refuses automated access) | **opened**: EC table (VOUT comparator, overdrive, current limit, one-shot clock, VCC range), Pin Functions RBG equations, Transformer Design (LPRI rules), RBG selection |
| TPS25982 | TI SLVSEI3D (Oct 2018, revised May 2026) | **opened**: recommended operating conditions (TJ −15…125 °C) |
| SMCJ45A | Littelfuse SMCJ series | **not opened** (figures from memory, UNVERIFIED) |
| UX-F5B, TNPV1210, BSS138, others | as §2 | not re-opened (unchanged parts) |
| TNPW0603 876 Ω | Vishay TNPW (document 28758) | **not opened**: ordering code per the series scheme |

### R3.8 Build and checks

`bash tools/build.sh`: **BUILD OK** (first run). `python3 tools/xcheck.py`: runs; sheet 10's inter-sheet nets all
reach their far sheets; the only single-sheet entry for this sheet is the local rail `VTRANS` (expected). The
whole-design PDFs were regenerated by the build but are **not committed** from this worktree (they contain every
sheet and would conflict with the seven parallel packages); regenerate at integration.

### R3.9 Still needs simulation, data or measurement

1. Transformer core + AC winding loss and secondary winding resistance (maker data) — decides the charge time and the
   secondary-drop part of the trip band.
2. LT3751 one-shot-clock behaviour on long cycles (P3) and VOUT-comparator overdrive for 2.9–3.3 µs pulses (ADI).
3. SMCJ45A breakdown and clamping figures, cold (datasheet).
4. A real LT3751 model run of the trip with the drawn values (the trip band here is arithmetic plus Monte Carlo).
5. Firmware behaviour after an unexpected `CHG_DONE_N` (P4) — a firmware/simulator test, once the rule exists.
