# WP-G capture notes — sheet `13_hv_bridge` (output bridge power stage)

27 November 2026. Contract issue 2; **revision 0.3 (contract issue 3) at the end of this file**. Spec: `hardware/aed-main/specs/13_hv_bridge.toml`; symbols:
`hardware/lib/src/aed_bridge.toml`.

> **NOT FOR CONSTRUCTION — unreviewed design.** This sheet connects a capacitor charged to about 2,000 V to a
> patient. It was drawn with AI assistance and has not been reviewed by a qualified high-voltage or medical-device
> engineer. Nothing here was built, simulated in circuit form or measured. Every number below is a calculation
> from datasheets [CALC], an estimate [EST], or a value that only circuit simulation can settle [SIM]. Nothing
> here is an instruction to build, power or test anything, and nothing here is safe or ready for use on a person
> or animal (SAFETY.md §2a, CLAUDE.md rule 2).

## 1. What is on this sheet, in plain language

This page is the part of the defibrillator that actually lets the stored charge out through the pads. Four
high-voltage silicon-carbide switches are arranged as an "H": for the first half of the shock the top-left and
bottom-right switches are on, for the second half the top-right and bottom-left, so the current reverses through
the patient. The top switches float up and down with the outputs, so they are driven through two small
transformers that carry a 400 kHz signal across a 2,200 V insulation gap; the transformer output is rectified to
make about 18 V for the switch's gate, and a small transistor empties the gate in about 10 µs when the signal
stops. The bottom switches are driven by two isolated driver chips from a small isolated +20/−5 V supply. In
series with everything sits a 3.9 Ω resistor and a 30 µH coil that limit how fast and how high the current can
rise in a fault; two diodes in series give the coil's current somewhere safe to go when a switch opens. A
2 mΩ resistor in the return measures the current: a fast comparator flags 90 A (a short), a second flags 5 A
when no bottom switch should be on (a shorted bottom switch), and an amplifier sends the waveform to the main
controller. Each output has a high-value resistor chain that both pulls it to 0 V at rest and gives the
controllers a scaled-down copy of its voltage. Finally there is an internal 200 Ω test load, connected only by a
reed relay that can close only in the low-voltage test mode. Every switch is held off by a resistor when all
power is gone; nothing on this page decides to shock — sheet 12 (the sequencer) gives the four on/off commands.

## 2. Parts (166 on the sheet) and datasheets

All datasheets read on **27 November 2026**. Where the manufacturer's server refused automated download, the copy
used is named.

| Ref | Part | MPN | Datasheet (as read) |
|---|---|---|---|
| Q1301–Q1304 (QHA, QLA, QHB, QLB) | 3.3 kV SiC MOSFET, TO-247-4 notched | Microchip MSC080SMA330B4N | DS00005914A (2025); Microchip's server returned an HTML page, copy read from RS: https://docs.rs-online.com/a067/A700000015733917.pdf |
| R1301 (R_S) | 3.9 Ω ceramic composition, 2 W, 80 J | Ohmite OY39GKE | OW/OX/OY series, DigiKey-hosted: https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/7182/ow-ox-oy-series.pdf |
| L1301 (L_S) | 30 µH coil, made to order | — | requirement: docs/HARDWARE_DESIGN.md §B6.5, §14 (+ §6.4 below) |
| D1301, D1302 (D_FW) | 3.3 kV 5 A SiC Schottky MPS, TO-263-7 | GeneSiC/Navitas GC05MPS33J | https://www.genesicsemi.com/sic-schottky-mps/GC05MPS33J/GC05MPS33J.pdf (Rev 22/Apr) |
| C1301, C1302 | 10 nF 3 kV X7R MLCC 2225 | KYOCERA AVX 2225HC103KAT2A | ratings from distributor listings (element14, IBS); only an old AVX catalogue found: https://www.farnell.com/datasheets/36581.pdf — **[UNVERIFIED]** |
| R1304 | 10 Ω ceramic composition (snubber) | Ohmite OY100KE | as R1301 |
| R1302 | 2 mΩ 1 % 1 W 4-terminal shunt | Vishay WSK2512R0020FEA | https://www.vishay.com/docs/30108/wsk2512.pdf (rev 11-Dec-2023) |
| D1303 | 200 V 3 A rectifier, SMC (shunt bypass) | Vishay S3D-E3/57T | https://www.vishay.com/docs/88713/s3a.pdf (rev 21-May-2024) |
| U1301, U1302 | isolated gate driver, Miller clamp | TI UCC5350MCDR | https://www.ti.com/lit/ds/symlink/ucc5350.pdf (SLLSER8J, Aug 2024) |
| U1303 | isolated DC/DC 5 V → +20/−5 V 2 W | Murata MGJ2D052005SC | Murata's server refused; copy of KDC_MGJ2_F01 from https://datasheet.chipsmall.com/MGJ2D122005SC/484428.pdf (revision date not printed — may be old) |
| U1304 | CMOS 555 timer | TI TLC555IDR | https://www.ti.com/lit/ds/symlink/tlc555.pdf (SLFS043K, Jan 2026) |
| U1305 | inverter | TI SN74LVC1G04DBVR | https://www.ti.com/lit/ds/symlink/sn74lvc1g04.pdf (SCES214AF, Oct 2025) |
| U1306–U1308 | D flip-flop | TI SN74LVC1G74DCUR | https://www.ti.com/lit/ds/symlink/sn74lvc1g74.pdf (SCES794G, Sep 2021) |
| U1309–U1312 | 3-input AND | TI SN74LVC1G11DBVR | https://www.ti.com/lit/ds/symlink/sn74lvc1g11.pdf (SCES487J, Nov 2026) |
| U1313, U1315 | 4.5 ns comparator, push-pull | TI TLV3501AIDBVR | https://www.ti.com/lit/ds/symlink/tlv3501.pdf (SBOS321E, Apr 2016) |
| U1314 | current-sense amplifier, gain 20 | TI INA240A1DR | https://www.ti.com/lit/ds/symlink/ina240.pdf (SBOS662C, Dec 2021) |
| T1301, T1302 | gate transformer, made to order | — | requirement: §14 + §6.3 below |
| Q1305, Q1308–Q1311 | 30 V N-MOSFET SOT-23 | Diodes DMN3042L-7 | https://www.diodes.com/assets/Datasheets/DMN3042L.pdf (DS37539 Rev 4-2) |
| Q1306, Q1307 | PNP 45 V 500 mA | onsemi BC807-40LT1G | https://www.onsemi.com/download/data-sheet/pdf/bc807-16lt1-d.pdf (Rev 18) |
| D1306, D1307, D1310, D1313–D1316, D1318, D1320–D1323, D1325 | 100 V switching diode SOD-123 | Diodes 1N4148W-7-F | https://www.diodes.com/assets/Datasheets/ds30086.pdf (Rev 32-2) |
| D1304, D1305, D1308, D1309, D1311, D1312, D1317, D1319, D1324, D1326–D1330 | zeners 3.6/6.8/20/22 V | onsemi BZX84CxxLT1G | https://www.onsemi.com/download/data-sheet/pdf/bzx84c2v4lt1-d.pdf (Rev 24) |
| D1331 | dual Schottky (series) | onsemi BAT54SLT1G | https://www.onsemi.com/download/data-sheet/pdf/bat54slt1-d.pdf |
| R1305–R1308, R1310–R1313 | 249 kΩ 1 % 2512 HV, 3 kV working | KOA HV733ATTE2493F | https://www.koaspeer.com/pdfs/HV73.pdf |
| K1301 | 10 kV reed relay SPNO, 5 V coil | Sensata Cynergy3 DAR70510-HR | D-HR series (Rev 09/24/24), RS-hosted: https://docs.rs-online.com/8c09/0900766b816395c1.pdf |
| R1303 | 200 Ω ceramic composition (test load) | Ohmite OY201KE | as R1301 |
| C1316, C1341, C1318, C1342 | 47 nF C0G 50 V 1206 | TDK C3216C0G1H473J160AA | **not read** — orderable code [UNVERIFIED] |
| other R, C | generic 0603/0805/1206/1210 | — | — |
| TP1301–TP1305 | test pads | — | — |

Datasheets were saved only to a scratch directory outside the repository.

## 3. Ratings against stresses (high-voltage and pulse positions)

| Ref | Stress | Rating | Margin / remark |
|---|---|---|---|
| Q1301–Q1304 VDS | ≤ 2,200 V (capacitor max; OVP band) + the D_FW forward drop | VDSS 3,300 V | 1.5×. Spikes from wiring inductance at turn-off [SIM] |
| Q130x VGS | low side +20.8 V max (MGJ2 at 10 % load, +4 %), −5.4 V; high side ≤ 20.6 V (20 V clamp − diode) | +23/−10 V; transient +25/−12 V | clamps D1308/D1311 (21.5–24 V) and D1309/D1312 (−7.1…−7.9 V) |
| Q130x current | 65.7 A peak planned; 115–126 A at the OC turn-off | ID 43 A (Tc 25), IDM 107 A (pulse, Tj-limited) | planned pulse is thermally fine (§4); **turn-off above IDM: finding F-2** |
| R1301 R_S | 24.1 J per adult shock at 25 Ω, 3.4 J at 175 Ω [CALC, model 4.7 Ω internal]; 25–26 J in the frozen-low-side fault; 256 V peak across | 80 J single impulse; 400 V working, 20 kV pulse; "70 J / 35 s, 100 pulses" (OY pulse tolerance) | 3.3× on energy. Hard-short fault (§7) puts ≈ 157 J in it → it may crack open, which stops the discharge (acceptable failure direction) |
| L1301 | 65.7 A for 3–12 ms; up to 465 A (hard short); ≈ 2 kV across it in < 1 µs at every phase start | made to order: ≥ 450 A ≤ 20 % loss | turn-to-turn insulation requirement added (§6.4) |
| D1301, D1302 | reverse: up to 2,200 V (bus pulled low); forward: 24 A (phase-1 end, 25 Ω) … 126 A (OC trip), τ 7.6 µs, twice per shock | VRRM 3,300 V each; IF,MAX 250 A (10 µs, non-repetitive); IF,SM 50 A (10 ms); dV/dt 200 V/ns; ∫i²dt 12 A²s | 0.06 A²s at 126 A [CALC]. **Repetitive** rating at tens of A for µs not stated → [UNVERIFIED]. 465 A (switch fusing open in a hard short) exceeds IF,MAX |
| C1301, C1302 | 1,100 V each at 2,200 V bus (series pair); one shorted: 2,200 V | 3,000 V | X7R loses a large part of its capacitance at 1–2 kV [UNVERIFIED]; dv/dt and pulse current not stated |
| R1304 | ½·C·V² ≈ 6 mJ per discharge (5 nF at 2 kV) through 10 Ω, ≈ 200 A peak if the bus is shorted | 80 J; 20 kV pulse | ample |
| R1302 shunt | 12 mJ per adult shock; ≈ 0.4 J in a 465 A, ≈ 2 ms hard-short dump | 1 W; pulse capability only via Vishay's online tool | [UNVERIFIED] pulse rating |
| D1303 | ≤ 1 V reverse at rest; forward only if the shunt opens (then the OC trip ends the pulse in < 1 µs) | 200 V, IFSM 100 A | |
| R1305–R1313 | 550 V each (2.2 kV / 4); 733 V with one shorted; 1.2 W each while the output is at full voltage | 3,000 V working, 1 W, overload 2.5 × rated V for 5 s | power exceeded only in the "high side shorted, capacitor charged" fault for seconds (§7) |
| K1301 | 2,200 V across open contacts; ≤ 0.3 A carry at ≤ 60 V; cold switched | 10 kV contacts, 17 kV contact/coil, carry 4 A | **operating −20…+70 °C** (narrower than the device) |
| R1303 | ≤ 0.18 J, 17.6 W peak at 60 V; ≈ 190 J if K1301 is welded during a real shock | 80 J | known gap (fault 31) |
| T1301, T1302 | 0–2,200 V between windings, steps of ≈ 2 kV in ≈ 1 µs (≈ 2–14 V/ns) | made to order (§14) | dv/dt × ≤ 10 pF = ≤ 140 mA displacement current into the primary ground |
| U1301, U1302 | barrier sees only LS_SRC–GND bounce (shunt drop + inductance, < 1 V normally) | 3 kVrms, VIORM 990 Vpk, CMTI ≥ 100 kV/µs | if the shunt opened without D1303 the barrier would see kV — hence D1303 |
| U1303 | as U1301 | 5.2 kV DC 1 s test; 200 Vrms reinforced working (UL) | |

## 4. SiC switch: thermal and safe operating area [CALC]

**Datasheet facts used (DS00005914A):** VDSS 3,300 V; RDS(on) 85 mΩ typ / 106 mΩ max at VGS 20 V, 30 A, 25 °C,
90 mΩ typ at 18 V; RDS(on) temperature factor (Fig 1-5) 1.00 at 25 °C, 1.17 at 50, 1.40 at 75, 1.65 at 100,
1.90 at 125, 2.2 at 150 °C; RθJC 0.22 typ / 0.28 max °C/W; Tj max **150 °C**; IDM 107 A; transient thermal
impedance (Fig 1-18, single pulse, read from a 220-dpi render, ±15 %): 0.016 °C/W at 10 µs, 0.039 at 100 µs,
0.10 at 1 ms, 0.155 at 3 ms, 0.217 at 10 ms, 0.27 at 30 ms, 0.28 plateau. The curve stops at ≈ 7 µs (below
0.01 °C/W). Forward SOA (Fig 1-17): lines for 10 µs, 100 µs, 1 ms and 10 ms at Tc = 25 °C; reading them with the
Zθ values gives ≈ 7.6 kW at 10 µs, ≈ 1.2 kW at 1 ms, ≈ 0.55 kW at 10 ms (constant-power lines). No avalanche
rating, no short-circuit rating, no reverse-bias (turn-off) SOA.

**Method.** Junction temperature by superposition of the Zθ curve (stepwise power, 5 µs steps), with RDS(on)
updated from the instantaneous Tj (worst-case: maximum 25 °C value × the typical temperature factor). Current
I(t) = V/(R_patient + 4.7 Ω) · e^(−t/τ), τ = (R_patient + 4.7 Ω) × 100 µF (design §B6.5 internal resistance).
Case temperature taken as the starting junction temperature: in milliseconds the TO-247 tab does not warm
measurably (≈ 1 J per shock into ≈ 6 g). **Starting Tj 55 °C** (warm enclosure worn on the body [ASSUMPTION]).

| Case | Switch | Peak A | Energy in switch | Tj max | vs 150 °C |
|---|---|---:|---:|---:|---|
| Adult 25 Ω, phase 1 3 ms (DESIGN_TABLE row) | QHA, QLB | 65.7 | 0.94 J | **102 °C** | 48 K margin |
| same, high side only reaching VGS ≈ 17 V (RDS +25 % [EST from Fig 1-2]) | QHA | 65.7 | 1.27 J | **119 °C** | 31 K |
| Adult 25 Ω, phase 1 lengthened to 12 ms (config limit) | QHA, QLB | 65.7 | 1.07 J | 102 °C | the tail adds little |
| Adult 25 Ω phase 2 | QHB, QLA | 23.9 | 0.08 J | 60 °C | |
| Adult 175 Ω, phase 1 9.0 ms | QHA, QLB | 10.9 | 0.09 J | 57 °C | |
| Child 25 Ω | QHA, QLB | 38.7 | 0.27 J | 68 °C | |
| **Fault**: low side frozen on to the 28.8 ms cap, 1,950 V, 25 Ω (§B13.6) | QHA, QLB | 65.7 | 1.07 J | 102 °C | |
| **Fault**: same at 2,000 V, high side at 17 V | QHA | 67.3 | 1.55 J | **124 °C** | 26 K |
| **Fault**: same at 175 Ω | QHA, QLB | 10.9 | 0.13 J | 57 °C | |
| At Tj start 25 °C, adult 25 Ω | QHA, QLB | 65.7 | 0.72 J | 61 °C | |

So the **planned pulse and the "frozen low side" fault stay below 150 °C** on paper, with 26–48 K of margin
depending on the gate voltage the high side really reaches. The peak conduction power (458–600 W, decaying with
a 1.5 ms time constant) lies inside the 1 ms SOA line and below the 10 ms line on average.

**Not covered (finding F-1): fault 18, a shorted low side.** With QLB shorted, QHA's slow turn-on (by design)
passes through its linear region with up to 2 kV across it. Simulated on paper with a square-law transfer
characteristic fitted to Fig 1-2 (Id ≈ 0.92·(VGS − 3.1 V)² A, hot threshold): the idle-current latch fires at
5 A and the PNP then turns QHA off. Energy in QHA and Tj, starting at 55 °C:

| Patient | Detection delay / PNP τ | Peak A | Energy | Tj (Zθ below 10 µs extrapolated as √t) |
|---|---|---:|---:|---:|
| 25 Ω | 0.7 µs / 3.0 µs | 8.9 | 25 mJ | ≈ 130 °C |
| 25 Ω | 1.5 µs / 3.4 µs | 14 | 49 mJ | ≈ 186 °C |
| 25 Ω | 3.0 µs / 4.7 µs | 26 | 122 mJ | ≈ 305 °C |
| 175 Ω | 1.5–3.0 µs | 10.8 | 16–19 mJ | ≈ 92 °C |

The detection delay is INA240 bandwidth (400 kHz) + comparator + sheet 12's latch + the carrier stopping; it is
probably 0.7–1.5 µs. At low patient resistance QHA may exceed 150 °C and could fail short, which would complete
the R-14 cascade (both switches of a diagonal shorted, relay still closed: monophasic discharge of up to
≈ 160–178 J, §B13.6 figures). The datasheet gives nothing below ≈ 7 µs and no linear-mode rating, so this can
only be settled by Microchip or by low-voltage pulse measurements. **Not fixable on this sheet without changing
the drive concept** (the slow high side is deliberate). Mitigations to consider: a faster idle path (a dedicated
comparator straight on the shunt, ≈ 10 mV threshold, with a low-offset part), or a hardware rule on sheet 12 that
holds the relay at rest until the high side has been on for its lead time without current.

**Finding F-2: turn-off current at the over-current trip.** In a hard output short the current rises at
V/L = 2,182 V / 30 µH ≈ 73 A/µs. Threshold 82–98 A (§8) + ≈ 0.35–0.4 µs to the start of the low side's
turn-off (100 ns filter, 5 ns comparator, ≈ 20–50 ns latch on 12, ≤ 100 ns UCC5350, ≈ 50–100 ns gate discharge
to the plateau) → **≈ 108–126 A at turn-off**, at ≈ 2.2 kV. IDM is 107 A and no turn-off SOA is published.
The turn-off itself is short (≈ 0.14 µs with the 33 Ω off resistor), ≈ 15–20 mJ [EST]. The design document's
"≈ 130 A for ≈ 2 µs [UNVERIFIED survival]" (fault 19) stands; it needs Microchip's statement.

## 5. False turn-on margin, redone with minimum threshold [CALC/EST]

- VGS(th) (VGS = VDS, 3 mA): **1.9 V min**, 3.0 typ, 5.0 max at 25 °C. Fig 1-16 (typical): 3.0 V at 25 °C,
  2.75 V at 55 °C, 2.33 V at 150 °C (factor 0.78). Applying the typical slope to the minimum part [EST — the
  datasheet gives no minimum over temperature]: **1.74 V at 55 °C, 1.48 V at 150 °C.**
- Coupled rise on an OFF high side when the opposite low side pulls its output from ≈ 2 kV to 0:
  ΔVGS = QGD / (C_added + CGS) = 51 nC / (94 nF + 3.46 nF) = **0.52 V** (QGD typical; no maximum published —
  with +30 % [EST] 0.68 V).
- Worst case: 0.68 V against 1.48 V (a switch that has just conducted and is hot) → **0.80 V margin, ratio 2.2**;
  at the more realistic 55 °C, 1.06 V margin. The design document's 0.49 V vs 1.9 V used typical/25 °C values.
- One of the two 47 nF gate capacitors open: 51 / 50.5 = 1.0 V passive; the PNP (Q1306/Q1307) then conducts as
  an active clamp: Miller current ≈ 51 nC / 1 µs ≈ 50 mA → VGS ≈ VBE (≈ 0.65 V) + 50 mA × 22 Ω + base-path drop
  ≈ 1.4 V [EST] — **no margin hot**, ≈ 0.3 V at 55 °C. Both open: limited only by the PNP clamp (same ≈ 1.4 V).
  This is why the 100 nF of §B6.3 is split into two parts (§9). A false turn-on makes a leg short that the OC
  latch ends (no shock delivered, fault reported), not an extra shock.
- Low sides: held at −5 V by the UCC5350's Miller clamp (≥ 5 A sink below VEE2 + 2.1 V) — margin ≥ 6.5 V.

## 6. Gate drive and timing [CALC unless marked]

### 6.1 Carrier (both high sides)

TLC555 astable, RA 12.4 kΩ, RB 2.74 kΩ, C 100 pF C0G: t_high = 0.693·(RA+RB)·C = 1.049 µs, t_low =
0.693·RB·C = 0.190 µs, f = 805 kHz [CALC]; with 1 % resistors, 5 % capacitor and the 555's internal delays
(≈ 50–150 ns at 3.3 V [EST]) the band is ≈ 650–900 kHz, so the transformer sees **≈ 325–450 kHz**. The /2
flip-flop gives exactly 50 % duty. Each primary FET conducts while its phase AND OSC are high: ≈ 1.05 µs every
2.5 µs, with the 0.19 µs OSC-low time as dead time at every phase change, so the two halves never conduct
together. G_H* is synchronised on the PH1 rising edge (whole cycles only — equal volt-seconds, no core walk);
G_H* low clears the gate at once.

### 6.2 High side (QHA, QHB)

Source: rectified secondary 18–20 V (§14), minus the 1N4148W gate diode (0.7–0.86 V) → 17.3–19.3 V, clamped at
18.8–21.2 V by the 20 V zener. Gate: 100 Ω + 2 × 47 nF C0G + the MOSFET (≈ 9 nF equivalent to 15 V incl. Miller
charge) ≈ 103 nF.

| Event | Nominal | Worst | Requirement (§B6.3) |
|---|---:|---:|---|
| G_H* high → first rectified pulse | ≤ 2.5 µs sync + ≈ 1.3 µs | 3.8 µs | — |
| gate 0 → 15 V (τ = 10.8 µs nominal; worst 13.1 µs, 17.3 V source, +20 Ω source impedance [EST]) | 18.5 µs | 26.4 µs | — |
| **turn-on total** | **≈ 22 µs** | **≤ 30 µs** | ≤ 40 µs ✓ |
| G_H* low → HSA_RECT collapses enough for the PNP (τ 2.2 µs, 1.3 V) | 0.16 µs | 0.3 µs | — |
| gate 18 V → < 2 V via PNP (τ = (22 Ω + 6.7–18 Ω)·97.5 nF = 2.8–4.0 µs) | 7.1 µs | 10.3 µs | ≈ 10 µs |
| gate → < 1 V | 10.6 µs | 15.4 µs | ≤ 20 µs ✓ |
| PNP failed open: 10 kΩ bleed, τ 0.98 ms, 18 → 1.5 V | 2.4 ms | — | backup only (R-23) |

Sheet 12's lockouts (≥ 150 µs between a high side off and the same leg's low side on; ≥ 50 µs high-side lead)
leave ≥ 10× margin on the turn-off and ≥ 1.6× on the turn-on.

**Carrier stopping in mid-phase.** If G_H* falls or the carrier dies while current flows, the high side turns off
under current in ≈ 7–10 µs (a fault case already in §B13.4/§13). If only BRIDGE_5V sags (the carrier keeps
running on BRIDGE_3V3), the high-side gate *tracks* the carrier amplitude (the PNP pulls the gate down to
≈ HSA_RECT + 0.6 V); at BRIDGE_5V = 4.5 V (sheet 12's under-voltage trip) the gate is still ≈ 16.5 V — fully on —
so the low side turns off first, as §B13.5 requires. **Correction to §B13.5:** the high-side gates do *not*
simply decay through the 10 kΩ bleed on a supply collapse; the active PNP makes them follow the carrier. The order
is still right because the low side's turn-off (≈ 1 µs from the 4.5 V trip) is faster than the PNP's
(≥ 3 µs to the plateau) even for an instantaneous BRIDGE_5V collapse — a margin of a few µs that needs [SIM].

Steady drive power per side: 2.2 kΩ rectifier bleed 8.6 mA + 10 kΩ bleed 1.9 mA ≈ 11 mA at 19 V ≈ 0.21 W;
peak secondary current at turn-on 19 V / 100 Ω ≈ 190 mA for ≈ 20 µs (primary ≈ 0.8 A peak [EST, ratio ≈ 4]).

### 6.3 Gate-transformer requirements added to §14

Frequency **300–450 kHz** (not only "400 kHz"); secondary must deliver **200 mA peak for 50 µs** and ≥ 30 mA
continuous at 18–20 V rectified, with ≤ 20 Ω output resistance [EST basis of the timing above]; leakage
inductance low enough that the 22 V drain clamps (D1327–D1330) absorb the spike; pin numbering 1 P1, 2 PCT,
3 P2, 4 S1, 5 S2 (symbol `GDT_PP_400K`).

### 6.4 L1301 requirement added

Turn-to-turn and layer-to-layer insulation for the full bus step (≈ 2 kV) appearing across the winding in < 1 µs
at every phase start (the snubber and wiring keep HV_BUS up while the coil's current is zero).

### 6.5 Low side (QLA, QLB)

UCC5350MC: VCC2 − VEE2 = 25 V (13.2–33 V allowed); OUT 10 A source/sink; prop delay 65 typ / 100 ns max,
pulse-width distortion ≤ 20 ns (part-to-part mismatch not tabulated — irrelevant here, the two low sides are
never switched together); CMTI ≥ 100 kV/µs; UVLO2 rising 12–13 V, falling 10.3–11 V; VCC1 UVLO 2.4–2.8 V; IN+
pulled low internally. **Output state:** "OUT … L" with VCC1 or VCC2 below UVLO, during start-up and after
(Tables 8-2, 8-3); unbiased output actively clamped to ≈ 1.5–2 V (§8.3.4.2). This settles the §B13.5
"[UNVERIFIED for the MC variant]": the MC variant does hold its output low.

| | Value |
|---|---|
| Turn-on | (20 − 9 V plateau)/(220 + 3.7 Ω) = 49 mA; QGD 51 nC → **1.0 µs** Miller, ≈ 2 V/ns at the drain; total to 20 V ≈ 2.5–3 µs |
| Turn-off | via 33 Ω + 1N4148W (∥ 220 Ω): (9 + 5 − 0.7 V)/≈ 37 Ω ≈ 0.36 A → **0.14 µs**, ≈ 14 V/ns |
| Off hold | Miller clamp to −5 V; 10 kΩ gate–source with no supply |
| Clamps | +21.5…+24 V, −7.1…−7.9 V (anti-series zeners) |
| Recommended drive (datasheet) | dynamic data characterised "at the recommended operating VGS = 20 V/−5 V" ✓ |

The turn-off is deliberately faster than the turn-on: at the 90 A trip the switching loss grows with the
transition time, and the coil holds the current constant during it.

### 6.6 Low-side supply and supply collapse

MGJ2D052005SC: 5 V in (4.5–5.5 V), +20 V 80 mA / −5 V 40 mA, line regulation 1.2 %/%, load envelope +4 % at
10 % load to −7 % at full load, start-up 5.4 ms typ with 10 µF. **Minimum load 10 %, otherwise the output can
rise to 1.25×** — +25 V would exceed VGS max (23 V), so preloads were added (R1326 2.2 kΩ, 9.1 mA; R1327
1.2 kΩ, 4.2 mA). Total load ≈ 15 % → +20 V ≈ 20.7 V (+ line regulation with BRIDGE_5V at 4.75–5.25 V:
≈ 19.5–22 V). On BRIDGE_5V loss: 12 µF per rail with ≈ 13 mA load → VCC2−VEE2 stays above 11 V for ≈ 20 ms;
the low sides therefore stay controllable, and sheet 12 turns them off first (≈ 1 µs). Below UVLO2 the outputs
go low by themselves.

## 7. Series element, freewheel path, snubber [CALC]

Per shock (model of DESIGN_TABLE, internal resistance 4.7 Ω):

| Shock | ∫i²dt | R_S energy | Shunt energy | Patient |
|---|---:|---:|---:|---:|
| Adult 25 Ω (3 + 2 ms) | 6.18 A²s | **24.1 J** | 12 mJ | 154.5 J |
| Adult 175 Ω (9.0 + 6.0 ms) | 0.86 A²s | **3.4 J** | 2 mJ | 150.3 J |
| Child 25 Ω | 2.15 A²s | 8.4 J | 4 mJ | 53.7 J |
| Fault: low side frozen to cap, 25 Ω | 6.40 A²s | 25.0 J | — | 160 J |

R_S repetition: three adult shocks two minutes apart ≈ 72 J against an 80 J single-impulse rating and the OY
pulse test of 70 J every 35 s for 100 pulses → adequate [ASSUMPTION: pulse ratings apply to 3–12 ms pulses].
R_S voltage: 65.7 A × 3.9 Ω = 256 V (400 V working). R_S tolerance ±10 % and TCR −1,300 ppm/°C (≈ −2.6 % per
shock as it warms ≈ 20 K [EST]) move the internal resistance (D-23 assumes 5 Ω) — the firmware's energy model
must tolerate ≈ ±0.5 Ω.

Coil current at turn-off and the freewheel pulse (τ = 30 µH / 3.95 Ω = 7.6 µs):

| Turn-off | Coil current | ½·L·I² | ∫i²dt in D_FW |
|---|---:|---:|---:|
| Phase-1 end, 25 Ω | 23.9 A | 8.6 mJ | 0.002 A²s |
| Phase-2 end, 25 Ω | 12.2 A | 2.2 mJ | — |
| OC trip | ≤ 126 A | 0.24 J | 0.06 A²s |
| Switch fusing open in a hard short | 465 A | 3.2 J | 0.82 A²s |

The 465 A case exceeds the diode's IF,MAX of 250 A: in a double fault the freewheel diodes may fail short too.

**Why two freewheel diodes.** If a single D_FW fails short, R_S and L_S are bypassed: the internal resistance
drops to ≈ 0.81 Ω, the 25 Ω shock rises from 154.5 J to **180 J** (peak 75.6 A, still under the 90 A trip), and
nothing on the device can see it (the 50 V self-test current changes by 2 %). A second diode in series makes
that a double fault. Each diode alone blocks the full 2.2 kV, so no sharing resistors are needed (leakage
mismatch can put all of it on one diode: within its 3.3 kV).

**Snubber.** HV_BUS–HV_RTN: 2 × 10 nF 3 kV in series (5 nF nominal, effective ≈ 2.5–3.5 nF at 2 kV bias [EST])
+ 10 Ω. The value is a placeholder; spike size with real wiring is [SIM] (§B6.5). Two capacitors in series so
that one cracked (shorted) MLCC does not connect HV_BUS to HV_RTN through 10 Ω (that would dump the capacitor
through R_S, ≈ 157 J in R_S, on the next charge).

## 8. Current sensing [CALC]

| Detector | Signal | Threshold | Band | Response |
|---|---|---|---|---|
| Over-current → `OC_DET_N` | shunt Kelvin, 100 Ω/1 nF | 3.3 V × 10.0/184 = 179.3 mV = **89.7 A** | reference ±2.9 % (BRIDGE_3V3 ±2 % [ASSUMPTION: WP-D's LDO] + 1 % resistors), offset ±6.5 mV (±3.3 A), hysteresis 6 mV, shunt ±1 % + 250 ppm/K × 50 K → **≈ 82–98 A** | 100 ns filter + 4.5 ns ≤ 0.2 µs at 73 A/µs ✓ (≤ 0.5 µs, contract §10) |
| Idle current → `IDLE_I_DET_N` | I_SHOCK (26.67 mV/A), 1 k/100 pF | 3.3 V × 10.0/247 = 133.6 mV = **5.0 A** | reference ±2.9 %, offset ±0.24 A, divider/INA ±1.5 %, shunt ±2.3 % → **≈ 4.5–5.5 A** | INA240 400 kHz → ≈ 0.4–1 µs |
| Waveform → `I_SHOCK` | INA240A1 × 20 × 2/3 | 26.67 mV/A | gain ±0.2 % + divider 0.2 % + shunt; full scale (VS − 0.2 V) × 2/3 = 3.2 V = **120 A** at BRIDGE_5V = 5.0 V | 400 kHz |

The OC threshold is referred to the shunt's own HV_RTN Kelvin terminal (SHUNT_KN), so ground bounce between
HV_RTN and GND does not shift it; the comparator's GND sits inside its −0.2 V input range. The OC input is
clamped by BAT54S to GND/BRIDGE_3V3; the idle comparator's input is I_SHOCK (≤ 3.3 V), inside its rails.
Protection from kilovolt transients: the only path for kV to reach LS_SRC is an open shunt, which D1303 now
bypasses (§9). INA240 common-mode range −4…80 V.

Both comparator outputs are push-pull on BRIDGE_3V3; unpowered or unsoldered they are pulled low by sheet 12 =
"fault" (R11). **Caveat:** the idle detector depends on the INA240. A dead INA240 reads 0 A, which the idle
comparator takes as "no current" (a safe-looking state). It is caught because the daily test sequence requires
I_SHOCK to show the ≈ 0.25 A test current (§B14.3) — the firmware must treat "no current in the test" as a fault
of this chain too.

## 9. Output nodes, rest pull-downs and leakage [CALC]

Divider 4 × 249 kΩ + 1.00 kΩ = 997 kΩ, ratio **1:997** (contract says 1:1,000; 0.3 % different, firmware
calibrates). Tap full scale 2,200 V / 997 = 2.21 V (≤ 2.6 V, R4).

- **Idle output voltage**: high-side off-state leakage IDSS ≤ 35 µA at 3,300 V, 25 °C (typ 0.3 µA; 3.5 µA typ at
  150 °C, no maximum given hot) flows into each output and out through its 997 kΩ: ≤ 35 V on the output,
  ≤ 35 mV on the tap. The low side's own leakage (at ≤ 35 V) is negligible. A shorted high side reads the full
  capacitor voltage (≥ 1 V on the tap): ≥ 28× separation.
- **Four switches' leakage**: at rest with the capacitor charged, QHA and QHB leak into their outputs (≤ 35 µA
  each); QLA/QLB see ≤ 35 V and leak nothing significant. The pads are not connected (patient relay at rest,
  sheet 14), so **no leakage reaches the patient at rest**. During a sequence the relay connects the outputs
  only while the bridge is driving them. The extra load on the capacitor is ≤ 70 µA (≈ 0.15 W at 2 kV —
  compare 0.61 W in the bleeders, §B5.6).
- If the relay were welded in the shock position at rest (fault 27/44), the patient would sit across the two
  997 kΩ pull-downs with ≤ 35 V of leakage-driven offset each: ≤ 35 µA through the patient [CALC, worst-case
  leakage spec]. Detected by `K_REST_N` (sheet 14).
- **Resistor ratings**: 550 V per resistor (3 kV rating); 1.2 W per resistor only while an output is at full
  voltage (a few ms in a shock; seconds in the "high side shorted, capacitor charged" fault, when the firmware
  refuses to shock and dumps) — the 2.5 × rated-voltage 5 s overload test covers ≈ 6× power for 5 s.
- Tap clamp BZX84C3V6 to GND (R-21): leakage ≤ 5 µA at 1 V → ≤ 0.2 % of the 2.2 mA divider current at full
  voltage.

## 10. Bridge test path [CALC]

K1301 closes only with `BTEST_EN` = `BTEST_CMD` · `TEST_MODE` (sheet 12) and only cold (≥ 2 ms before a low side,
§B13.3 rule 9). Coil 5 V, 28 Ω (178 mA) switched by a DMN3042L (≤ 48 mΩ at VGS 2.5 V → 9 mV drop);
must-operate 3.7 V at 20 °C, +0.4 %/°C → 4.4 V at 70 °C against BRIDGE_5V ≥ 4.75 V [ASSUMPTION about the rail].
Operate 3 ms, release 2 ms with the flyback diode. Test current at 60 V: 60 / 204.7 Ω = 0.29 A (I_SHOCK
7.8 mV), 0.18 J. Contacts carry 4 A, 10 kV open.

## 11. Rail loads for WP-D (contract §10)

| Rail | Load from sheet 13 [EST] |
|---|---|
| `BRIDGE_3V3` | UCC5350 VCC1 2 × 2.4 mA max, TLV3501 2 × 5 mA max, TLC555 ≈ 0.5 mA + timing ≈ 0.3 mA, LVC logic switching the four push-pull gates (4 nC × 400 kHz × 2 per active side) ≈ 3–6 mA, threshold dividers 0.03 mA → **≈ 20–25 mA** (for the ≥ 2 ms hold-up sizing) |
| `BRIDGE_5V` | MGJ2 ≈ 90–130 mA (15 % load at 74 % efficiency + no-load), two carriers ≈ 2 × 55 mA steady, ≈ 2 × 0.8 A peak for ≈ 20 µs at turn-on, INA240 2.4 mA, K1301 178 mA (test mode only) → **≈ 0.25 A in a shock, ≈ 0.4 A in a test**, plus the turn-on peaks drawn from C1319/C1321 |

## 12. Per-part failure table

"Shock" = during a sequence; "rest" = capacitor charged, bridge unpowered. ST = daily/switch-on self-test (test
permit, §B14). OC/IDLE = the latches on sheet 12. "Blocks shocks" means a failure to shock, reported.

| Ref | Fails open | Fails short | Detected by / when |
|---|---|---|---|
| Q1301 QHA | phase 1 delivers nothing; phase 2 then runs from the full capacitor voltage: a monophasic pulse (25 Ω: ≈ 118 J in 2 ms; 175 Ω: ≈ 90 J [CALC]) instead of the biphasic 150 J; `I_SHOCK` shows it → fault reported after the shock | rest: HV_OUT_A at capacitor voltage (behind the open patient relay) → `OUT_A_TAP` high → firmware refuses to shock (fault 17). Shock (if missed): phase 2 = leg-A short → OC latch, pulse lost, ≈ 0.24 J into D_FW/R_S | OUT_A_TAP at every charge; ST |
| Q1302 QLA | phase 2 lost: monophasic phase-1-only pulse (≈ 117–139 J of the 150 J [CALC]) | rest: nothing (HV_OUT_A at 0). **Shock: if it is shorted before phase 1, QHA turning on at phase-1 start shoots straight through it across the capacitor (same-side shoot-through, BR Q1: ≈ 240 A, 11–12 J in QHA as captured in rev 0.2; rev 0.3: 12.8–25.4 mJ, 99–139 °C pessimistic [SIM rev3-wpg S-1])**; if it shorts during the gap, QHB turning on does the same on leg B → idle latch / OC | ST (current in the lead, §B14.3); IDLE (rev 0.3: fast path + kill latch) |
| Q1303 QHB | phase 2 lost (monophasic) | as QHA for leg B | OUT_B_TAP; ST |
| Q1304 QLB | phase 1 lost; phase 2 runs from the full voltage: monophasic pulse as for QHA open | **fault 18 / finding F-1**: QHA turns on into it through its linear region; idle latch ends it within µs; QHA may overheat (130–300 °C est.) and fail short → R-14 cascade, ≤ 178 J monophasic | ST between rescues; IDLE in a shock |
| R1301 R_S | no current: no shock; `I_SHOCK` 0 → fault | R_S bypassed: internal R ≈ 0.9 Ω → 25 Ω shock ≈ +17 % energy, peak 75 A; hard-short current limited only by L_S | open: ST. **short: not detected** (2 % change in the 50 V test current) — latent, **finding F-3** |
| L1301 L_S | no current: no shock | (turns shorted) di/dt limited only by wiring; OC trip at ≈ 90 A + delay with no coil → in a hard short current may reach several hundred A before turn-off → switch stress beyond F-2 | open: ST. **short: not detected** — latent (F-3) |
| D1301 | freewheel path lost: every phase end avalanches a switch (R-2) | D1302 still blocks: none | **open not detected** — latent, F-4; short: latent (double fault needed) |
| D1302 | as D1301 | as D1301 (D1301 blocks) | as D1301 |
| C1301 | snubber lost: larger spikes [SIM] | C1302 holds the bus; none | latent |
| C1302 | as C1301 | C1301 holds | latent |
| R1304 | snubber lost | snubber undamped (ringing) [SIM] | latent |
| R1302 shunt | D1303 conducts (< 1 V); OC comparator sees > 180 mV at once → OC latch → **every shock blocked**, detected (also in the ST: ≈ 0.6 V at 0.3 A trips it) | sense reads 0: OC and idle detection lost, `I_SHOCK` 0 → ST fails (no current seen) | ST |
| D1303 | none (only matters with an open shunt) | shunt bypassed: currents read 0 → ST fails | ST |
| U1301 / U1302 | input dead or VCC1/VCC2 lost: output held low → that low side never turns on → phase lost | OUT stuck high: low side on whenever its supply is up → with its own high side on = leg short (OC latch); otherwise = fault 18 behaviour | ST |
| R1319 / R1324 (100 k) | G_L* floats only if sheet 12 is also unpowered; UCC5350 IN+ has its own pull-down → low | G_L* held low: low side never on: phase lost | ST |
| C1303 / C1307 | VCC1 noise, possible UVLO glitch → output low | VCC1 short: BRIDGE_3V3 short → sheet 12 rail lost → sequence never starts | ST |
| C1304–C1306, C1308–C1310 | gate drive weaker / slower, more ringing | +20 or −5 V shorted to source or each other → MGJ2 overload (it is short-circuit protected) → no low-side drive → phases lost | ST |
| R1316 / R1321 (Rg_on 220) | low side never turns on: phase lost | fast turn-on (≈ 0.1 µs, ≈ 20 V/ns): high-side dv/dt coupling unchanged in size (charge-based), more EMI | open: ST; short: latent |
| R1317 / R1322, D1307 / D1310 (off path) | turn-off via 220 Ω only: 0.8 µs → more energy at the OC trip (≈ 50 mJ) | faster turn-on (short diode) or faster turn-off | latent |
| R1318 / R1323 (10 k) | gate can float only with the driver unpowered — the UCC5350 output clamps it to ≈ 1.5–2 V anyway | gate shorted to source: low side never on: phase lost | ST |
| D1308/D1309, D1311/D1312 | no clamp (gate still within the drive levels) | gate clamped at 0.7 V (+) or −0.7 V: low side never on (short of the 22 V part) | ST (phase missing) |
| R1320 / R1325 (2.2 Ω) | that driver's 0 V reference via the other die only (ground loop) — works, noisier | Kelvin loops joined directly | latent |
| U1303 MGJ2 | no low-side supply: drivers in UVLO, outputs low → no shock | input short: BRIDGE_5V overloaded → rail collapse → sequence ends (fault) | ST |
| C1311–C1313 | ripple, slower hold-up | rail short → as U1303 | ST |
| R1326 / R1327 (preloads) | MGJ2 can rise towards 25 V at light load: gate over-voltage (clamped by D1308/D1311 at 21.5–24 V) | rail shorted → as U1303 | open: latent (gate clamps protect) |
| U1304 TLC555 | no oscillator: no carrier → high sides never on → no shock | output stuck: divider stops → one FET held on → transformer core saturates, no carrier | ST |
| R1346, R1347, C1323, C1324, C1325 | oscillator stops or runs off-frequency (≈ ±50 %): carrier weaker or stops → slow/no high side → sequencer lead (≥ 50 µs) may start a phase with the high side half on (linear region, F-1-like) | as open | ST checks phase current; a slow high side is **not** distinguished by the ST (finding F-5) |
| U1305 inverter | divider stops → no carrier | — | ST |
| U1306 divider | stuck: one primary half driven continuously (gated by OSC) → core saturates → FET over-current; carrier lost | — | ST |
| U1307 / U1308 sync | stuck low: that high side never on; stuck high: carrier runs whenever BRIDGE_3V3 is up → that high side on as soon as the rail rises (before G_H*) — sheet 12's low-side lockouts still prevent a leg short; patient not connected unless the relay transfers | | ST (`OUT_x_TAP` rises at rail-up with no command) — firmware must check it |
| U1309–U1312 AND | output low: half-carrier → half-wave rectified, slower turn-on; both low: no carrier | output high: one primary half on continuously → saturation, FET overload | ST |
| R1348 / R1349 (G_H* pull-down) | only matters with sheet 12 unpowered — then BRIDGE_3V3 is also off | G_H* low: high side never on | ST |
| C1326–C1333 | logic noise | rail short → BRIDGE_3V3 lost → no sequence | ST |
| Q1308–Q1311 | half of the carrier lost → half-wave, ≈ 2× slower turn-on (≤ 60 µs worst — above the 50 µs lead) | primary half shorted: BRIDGE_5V across half a winding through the other FET → saturation, high current; carrier lost | ST (slow turn-on not distinguished: F-5) |
| R1338–R1345 | gate floats: pull-down keeps FET off (open 10 Ω) / FET always off (open pull-down: nothing) | FET gate tied low | ST |
| D1327–D1330 | drain spikes unclamped (30 V FETs) | primary half shorted to GND → as a shorted FET | latent / ST |
| C1319–C1322 | carrier droops during the 0.8 A turn-on peaks → slower turn-on | BRIDGE_5V short → rail lost | ST |
| T1301 / T1302 | winding open: no carrier → high side never on; **insulation breakdown**: HV_OUT potential (≤ 2.2 kV) onto the primary and BRIDGE_5V → low-voltage electronics destroyed; capacitor discharges through the fault | (see open) | **insulation breakdown is the gravest single fault on the sheet**: depends entirely on the made-to-order part's qualification (§16 rank 2) |
| D1313–D1316, D1320–D1323 (bridges) | half-wave rectification: slower turn-on (≈ 2×) | secondary shorted every other half-cycle → carrier overload, no gate voltage | ST (F-5 for the slow case) |
| C1315 / C1317 (1 nF) | more ripple on HSA_RECT; gate unaffected (diode + 94 nF) | no gate voltage: high side never on | ST |
| R1328 / R1333 (2.2 k) | RECT collapses slowly → PNP turn-off delayed by ≈ 10–100 µs; base current path lost → **PNP may not turn on at all** → bleed only (τ ≈ 1 ms) → leg short at the next low-side on (fault 26) → OC latch, phase lost | RECT shorted: no gate drive | ST checks "no current when the opposite low side turns on after a high side was turned off" (§B14.3) ✓ |
| D1317 / D1324 (20 V clamp) | no over-voltage clamp (D1319 at 22 V remains) | no gate drive | ST |
| D1318 / D1325 (gate diode) | no gate drive | RECT and gate joined: the PNP can never see RECT below the gate → **no active turn-off** (bleed only) → fault 26 | ST ✓ |
| R1329 / R1334 (100 Ω) | no gate drive | faster turn-on, larger peak current from the transformer | open ST; short latent |
| Q1306 / Q1307 PNP | fault 26 (bleed only, ≈ 2.4 ms) → leg short at phase 2 → OC latch, phase lost | gate tied to source through 22 Ω: high side never on | ST ✓ |
| R1330 / R1335 (470 Ω) | PNP never turns on: fault 26 | PNP turns on harder: fine | ST |
| R1331 / R1336 (22 Ω) | PNP cannot discharge: fault 26 | PNP peak ≈ 1.5 A (> ICM 1 A): may fail later | open ST; short latent |
| C1316, C1341 / C1318, C1342 (2 × 47 nF) | one open: turn-on ≈ 2× faster, dv/dt rise 1.0 V (PNP clamp ≈ 1.4 V) — reduced margin, possible leg short at 2 kV only → OC latch, phase lost | gate shorted: high side never on | open: **not detected** (F-5); short: ST |
| R1332 / R1337 (10 k bleed) | backup lost; PNP still turns off | gate shorted: never on | ST |
| D1319 / D1326 (22 V gate clamp) | backup clamp lost | gate shorted: never on | ST |
| D1331 BAT54S | OC input unprotected | OC_SNS clamped to GND (never trips) or to 3.3 V (trips at once → every shock blocked) | trip-at-once: ST; never-trips: **latent, two faults to matter** (needs a short as well) |
| R1350 (100 Ω) | OC comparator input floats → undefined (may read OC or not) | filter τ 0: noise trips | open: possibly latent; finding F-6 (add an OC self-test) |
| C1334 (1 nF) | no filter: noise may trip OC at switching edges → nuisance fault | OC_SNS = SHUNT_KN: OC never trips | short: **latent** (F-6) |
| R1351 (174 k) | OC_REF → 0 V: OC trips at once → every shock blocked | OC_REF = 3.3 V: never trips (≈ 1.6 kA) | short: **latent** (F-6) |
| R1352 (10.0 k) | OC_REF = 3.3 V: never trips | trips at once: blocks | open: **latent** (F-6) |
| C1335 (100 nF) | noisier threshold | OC_REF 0: trips at once | ST |
| U1313 TLV3501 | output low (pull-down on 12) = "fault": sequencer refuses to start | stuck high = "no OC": OC protection lost | stuck high: **latent** (F-6) |
| C1336 / C1340 | comparator noise | BRIDGE_3V3 short | ST |
| R1353 / R1354 (10 Ω) | INA240 input open: output to a rail — 0 (reads no current) or full scale (reads 120 A, idle latch trips at once) | none | ST |
| U1314 INA240 | output 0: `I_SHOCK` 0 and idle detection lost | — | ST ✓ (test current must be seen) |
| C1337 | noise | BRIDGE_5V short | ST |
| R1355 / R1356 | I_SHOCK 0 (top open) or ×1.5 (bottom open, up to 4.9 V into the MCU pin and the idle comparator input [EXCEEDS 3.6 V]) | ×1.5 / 0 | ST (wrong scale at 0.25 A) |
| R1357, C1338 | idle comparator input floats | idle input = 0 or no filter | latent / ST |
| R1358 / R1359, C1339 | idle threshold 0 V (trips at once → blocks) or 3.3 V (never trips) | the reverse | never-trips: **latent** (F-6) |
| U1315 TLV3501 | output low = fault (refuses) | stuck high: idle detection lost | **latent** (F-6) |
| R1305–R1308 / R1310–R1313 (divider top) | tap reads 0 always: a shorted high side would not be seen at rest (R11-like) | one shorted: tap reads 4/3 (+33 %), 733 V on the others | open: ST must see the tap follow the output in the test sequence; short: firmware sees a 33 % mismatch against the capacitor voltage during the test |
| R1309 / R1314 (bottom) | tap rises to the zener clamp (3.6 V) at > ≈ 4 V output — a false "shorted switch" reading → firmware refuses (fail safe) | tap 0 (as top open) | ST |
| D1304 / D1305 (tap clamp) | no clamp (reader protection lost) | tap 0 (as above) | latent / ST |
| K1301 | test path lost: ST bridge test fails (no test current) → device reports a fault | **welded**: outputs joined through 200 Ω at all times (fault 31): ST sees OUT_B follow OUT_A → firmware refuses; if missed, a real shock puts ≈ 10 A, ≈ 190 J into R1303 (fails open) and diverts current from the patient | ST |
| R1303 (200 Ω) | test path lost (as K1301 open) | test load 0 Ω: at 60 V the test current is limited by R_S etc. (≈ 13 A, below the 90 A trip): ST sees ≈ 45× the expected current → fault | ST |
| Q1305 | relay never closes: test fails | relay closed whenever BRIDGE_5V is up — in shock mode too → as a welded K1301 | ST (outputs follow each other) |
| R1315 (100 k) | BTEST_EN floats only with sheet 12 unpowered (BRIDGE_5V then also off) | relay never closes | ST |
| D1306 | coil flyback unclamped → Q1305 avalanche (30 V) at every release | coil shorted: relay never closes, and Q1305 shorts BRIDGE_5V to GND whenever BTEST_EN is high → rail collapse in test mode (test fails) | ST |
| TP1301–TP1305 | — | (pad shorted to something by a probe) — not a part fault | — |

**Unsafe outcomes found and what was done on this sheet:**
1. *Single freewheel diode short silently removes R_S and L_S (+17 % energy at 25 Ω, no fault current limit
   beyond the switches).* **Fixed:** D_FW is two diodes in series (D1302 + D1301).
2. *Single snubber capacitor short connects HV_BUS to HV_RTN through 10 Ω (capacitor dumped through R_S on every
   charge, device unusable).* **Fixed:** two capacitors in series.
3. *Open shunt puts kilovolts on LS_SRC (destroys both low-side drivers, the MGJ2 and the INA240; isolation
   barriers then become the only protection of the low-voltage side).* **Fixed:** D1303 bypass; the open shunt
   now trips the OC latch and blocks every shock (detected).
4. *MGJ2 at light load could raise the low-side gate drive to ≈ 25 V (VGS max 23 V).* **Fixed:** preloads
   R1326/R1327 (+ the gate clamps).
5. *One open gate capacitor halves the high side's dv/dt immunity.* **Mitigated:** 2 × 47 nF in place of one
   100 nF; the PNP clamps; remaining margin small when hot (§5).
6. *Fault 18 linear-mode stress of the high side* — **finding F-1**, not fixable on this sheet.
7. *Several "protection lost" faults of the OC and idle chains are latent* (they matter only with a second
   fault, but are invisible) — **finding F-6**: the self-test does not exercise either comparator's trip.
8. *R_S or L_S shorted is latent and raises delivered energy* — **finding F-3**.

## 13. Findings for the design owner

- **F-1** Fault 18 (shorted low side): the high side's linear-region turn-on and the idle-latch turn-off may take
  QHA past 150 °C at 25 Ω (≈ 130–300 °C est.), risking the R-14 cascade. Needs Microchip data or pulse tests;
  consider a faster idle path or a relay/lead rule on sheet 12 (§4).
- **F-2** Turn-off current at the OC trip ≈ 108–126 A at 2.2 kV, above IDM 107 A; no turn-off SOA published.
  Lowering the trip to ≈ 80 A would still clear the 65.7 A planned peak (band ≈ 74–86 A) and bring the turn-off
  current to ≈ 100–115 A — a contract/§B6.6 decision, not made here.
- **F-3** A shorted R_S or (shorted turns of) L_S is invisible to every check and raises delivered energy
  (25 Ω: 154 → ≈ 180 J). A fast-sampled I_SHOCK rise-time check in the 50 V test (L/R = 0.15 µs into 205 Ω —
  too fast for the ADC) does not help; a measurement at the start of a real shock (peak current vs expected)
  could, after the fact. Listed.
- **F-4** An open freewheel diode is latent and makes every phase end avalanche a switch (R-2 returns). Not
  detectable by the low-voltage test (≈ 0.01 mJ of coil energy at 0.3 A). Listed for §13.
- **F-5** A slow high-side turn-on (half the carrier lost, one bridge diode open, oscillator slow, one gate
  capacitor open) is not detected by the self-test as specified; with the 50 µs hardware lead (§B13.3 rule 3) a
  high side at worst ≈ 60 µs could still be in its linear region when the phase starts. Suggest the firmware
  measure, in the test sequence, the time from `CMD_H*` to the output tap reaching the capacitor voltage, and
  that sheet 12's lead be ≥ 80 µs (the firmware already uses 100 µs).
- **F-6** Neither comparator's trip is exercised by any self-test; a stuck-high comparator or a drifted threshold
  is latent. Suggest a test injection (e.g. a switched resistor into OC_SNS/IDLE_SNS from a supervisor-
  controlled line) — needs a new inter-sheet net, so it is a contract change, not made here.
- **F-7** The test relay's −20…+70 °C range and its coil's 4.4 V must-operate at 70 °C (BRIDGE_5V ≥ 4.75 V)
  leave little margin; the device's operating range should be checked against it.

## 14. Unverified pins, ratings and assumptions

**Pins**
- GC05MPS33J: datasheet labels only "NC", "A" and "Case (K)"; lead numbers 1 = NC, 2–7 = A, 8 = tab follow
  KiCad's TO-263-7_TabPin8 — **unverified**.
- DMN3042L: SOT23 G-S-D = 1-2-3 read from the standard top-view drawing; numbers not printed — **unverified**.
- WSK2512 shunt: datasheet names I1, I2 (current) and E1, E2 (sense) but gives no pad numbers; mapped to KiCad's
  R_Shunt pins 1 (I1), 4 (I2), 2 (E1), 3 (E2) and its WSK2512 footprint — **unverified**, and the footprint's
  T dimension for a 2 mΩ value is unverified.
- MSC080SMA330B4N: footprint "TO-247-4 notched" must be drawn from Fig 2-1 (custom, footprint_todo); the stock
  TO-247-4 has 2.54 mm pitch without the drain–source notch, which matters at 3.3 kV.
- TLC555 stock symbol names pins VCC and ~{RST}; TI names them VDD and RESET (same numbers).
- SN74LVC1G74 drawn with the stock 74AUC1G74 symbol (identical DCU pinout, TI SCES794G Pin Functions).
- All custom symbols carry "PinoutVerified = NO" until a person checks them.

**Ratings not stated by the datasheet (R10 `Verify` fields on the parts)**
- MSC080SMA330B4N: avalanche, short-circuit, turn-off SOA; Zθ below ≈ 7 µs; VGS(th) minimum over temperature;
  QGD maximum; IDSS maximum hot; lifecycle.
- GC05MPS33J: repetitive peak forward current for µs pulses; lifecycle (its predecessor GB05MPS33-263 is marked
  last-time-buy).
- 2225HC103KAT2A: capacitance at bias, dv/dt and pulse current; datasheet not found (only distributor listings).
- WSK2512: pulse energy (Vishay JouleWizard only).
- OY resistors: repetitive ratings for 3–12 ms pulses; footprint vs body 19 × 7 mm.
- MGJ2D052005SC: the datasheet copy is an undated distributor mirror (Murata refused automated access).
- TDK C0G 47 nF 50 V 1206 code: not checked.
- DAR70510-HR and HV733ATTE2493F part codes were composed from the makers' ordering guides.

**Assumptions**
- Starting junction temperature 55 °C; BRIDGE_3V3 accuracy ±2 %; BRIDGE_5V 4.75–5.25 V; transformer output
  resistance ≤ 20 Ω; 555 internal delay ≈ 50–150 ns; Miller-region square-law fit for fault 18; Zθ read from
  the figure ±15 %.

## 15. Departures from the design specification (§B6, contract §8)

| Spec | Captured | Why |
|---|---|---|
| D1301 single freewheel diode | D1301 + D1302 in series | single-fault: a shorted diode would remove R_S + L_S silently (§12 item 1) |
| Snubber "10 nF / 3 kV + 10 Ω" | 2 × 10 nF 3 kV X7R in series (≈ 5 nF) + 10 Ω | no 10 nF 3 kV C0G part exists in 2225 (KEMET HV C0G waterfall: ≈ 1 nF max at 3 kV); series pair for single-fault tolerance; value is [SIM] anyway |
| High side "18 V zener-limited" | 20 V zener on the rectified node (+ 22 V gate clamp) | datasheet characterises at VGS 18 V and 20 V; with an 18 V zener the gate would sit at 16–18 V after the diode and RDS(on) rises (thermal margin falls from 48 K to ≈ 31 K); 20 V keeps the gate ≤ 20.6 V < 23 V max |
| 100 nF added gate capacitance | 2 × 47 nF C0G | one open capacitor still leaves half (§5) |
| "2 × dual N-FET push-pull" | 4 × single DMN3042L | a stock symbol with verified pin names exists for the single part; function identical |
| — | sync flip-flops U1307/U1308 + AND gates | implements "gating applied to both phases at once on a clock edge" (§B6.3) and dead time |
| — | shunt bypass diode D1303 | open-shunt protection (§12 item 3) |
| — | MGJ2 preloads R1326/R1327, star resistors R1320/R1325 | minimum-load rule of the MGJ2; Kelvin-loop separation |
| Output divider "1:1,000" | 1:997 | 4 × 249 kΩ + 1.00 kΩ (nearest standard bottom value) |
| Divider bottom "pulled to HV_RTN (1 MΩ)" (contract §5.7) | bottom leg and tap clamp to **GND** | R-21 and §B6.6 put the tap clamp on GND, and the tap is read against GND on 07; GND and HV_RTN are the same node at the star (NT1101/NT1102) |
| Idle comparator source not specified | from `I_SHOCK` (INA240 output, divided) | 10 mV across the shunt is too small for a comparator with ±6.5 mV offset; caveat in §8 |
| §B13.5 "high sides decay through the 10 kΩ bleed on a supply collapse" | they track the carrier through the PNP | behaviour of the specified circuit, order still correct (§6.2) |
| §14 T1301/T1302 "400 kHz" | 300–450 kHz, 200 mA peak, ≤ 20 Ω | oscillator tolerance and turn-on current (§6.3) |

## 16. Contract issues

1. **Shunt Kelvin nets and net classes.** §B12 puts "the shunt Kelvin nets" in `RTN_PULSE`, but R2 forbids `RTN`
   and `LS_SRC` in any other net name and the class patterns are `HV_RTN` and `*LS_SRC*`. Captured as
   `SHUNT_KP` / `SHUNT_KN` in the Default class (they carry no current; a 3 mm track would be wrong for them).
   *Proposed change:* delete "the shunt Kelvin nets" from the `RTN_PULSE` row of §B12, or add an explicit rule
   for Kelvin sense nets.
2. **`HV_OUT_A/B` default "pulled to HV_RTN (1 MΩ)".** Captured to GND (see §15). *Proposed change:* "pulled to
   GND (997 kΩ divider) on 13".
3. **`OUT_A_TAP` / `OUT_B_TAP` ratio "1:1,000".** Captured 1:997. *Proposed change:* state 1:997 (or let 07 use
   the real ratio).
4. **R11 and the output taps.** An open divider top reads 0 V = "high side not shorted" — a safe-looking state.
   *Proposed change:* add to §F that the test sequence must see each tap follow its output (≈ test voltage /
   997) when its high side is on, before any rescue.
5. **Idle detector depends on the INA240** (contract does not say where the 5 A comparator takes its signal).
   *Proposed change:* say so in §5.7/§B6.6 and require the test sequence to fail on "no test current".
6. **Comparator trip self-test (F-6).** No net exists to exercise `OC_DET_N` / `IDLE_I_DET_N`. *Proposed change:*
   a supervisor-driven test-injection net (e.g. `ITEST_S`, 05 → 13, default low) in a future contract issue.
7. **`BRIDGE_3V3` / `BRIDGE_5V` loads** for WP-D: ≈ 20–25 mA and ≈ 0.25–0.4 A from this sheet (§11).
8. **§14 transformer specification** needs the frequency band and peak current (§6.3); **§14 L1301** needs the
   turn-to-turn requirement (§6.4).
9. **Fault table §13** needs rows for F-1…F-6 and the corrections in §12 (two freewheel diodes, D1303, MGJ2
   preloads).

## 17. Things a reviewer should check first (my own top five doubts)

1. **Fault 18 linear-mode stress (F-1) and turn-off at ≈ 120 A (F-2).** My numbers extrapolate a thermal curve
   below its last printed point and assume an RBSOA that is not published. If the switch cannot survive either,
   a single shorted low side can still end in a full monophasic discharge.
2. **The PNP turn-off and supply-collapse ordering (§6.2).** I argue the high-side gate tracks the carrier rather
   than decaying, and that the low side still turns off first; this is the kind of µs-level race only a circuit
   simulation can confirm.
3. **The gate transformers' insulation.** A breakdown puts up to 2.2 kV onto BRIDGE_5V and everything on it. The
   whole sheet's isolation rests on a made-to-order part that does not exist yet.
4. **Snubber and freewheel values.** The snubber capacitor (X7R at 2 kV, ≈ 2.5–3.5 nF effective) and the
   freewheel diodes' repetitive µs rating are guesses until simulation and the makers confirm; spike sizes at the
   switch terminals were not computed.
5. **Unverified pin numbering** of the SiC diode (TO-263-7), the shunt pads and the SOT-23 FETs, and the custom
   notched TO-247-4 footprint — a swapped Kelvin or gate pin on a 3.3 kV switch is not a small error.

## 18. Totals

- **Parts: 166** — R 59, C 41, D 31, U 15, Q 11, TP 5, T 2, L 1, K 1 (4 made-to-order or footprint-to-create:
  Q130x footprint, T1301/T1302, L1301, K1301 footprint).
- **Nets: 90** — 73 local, 13 inter-sheet global labels (`HV_CAP_P`, `HV_OUT_A`, `HV_OUT_B`, `G_HA`, `G_HB`,
  `G_LA`, `G_LB`, `BTEST_EN`, `OC_DET_N`, `IDLE_I_DET_N`, `I_SHOCK`, `OUT_A_TAP`, `OUT_B_TAP`), 4 rails (`GND`,
  `HV_RTN`, `BRIDGE_5V`, `BRIDGE_3V3`). 431 pin connections; netlist round trip exact.
- Build: ERC 0 errors (4 "rail not driven" accepted until sheets 01, 09, 11, 12 are captured), 8 footprint-link
  warnings for footprints still to be made (accepted automatically), netlist round trip OK.

## 19. Not done

- No circuit simulation (spikes, snubber values, ordering margins, fault 18) — [SIM] items above.
- Custom footprints (TO-247-4 notched, D-HR relay, transformers, coil) are named but not drawn.
- The MGJ2 datasheet was read from an undated mirror; the C0G 47 nF part code and the AVX 3 kV capacitor were not
  checked against manufacturer datasheets.
- Nothing in `DECISIONS.md`, the contract or the design document was changed (not in this package's scope); the
  changes proposed above are for the integrator.

---

## Revision 0.3 (contract issue 3, change order `rev3-changes.md` WP-G)

27 November 2026. Applies G-1 … G-8 to `specs/13_hv_bridge.toml` and checks the sheet **as drawn** by simulation
in `sim/rev3-wpg/` (notes there: `sim/rev3-wpg/notes.md`). Every figure is [CALC] from datasheets or [SIM] from
hand-built models; nothing was built or measured. Nothing here is an instruction to build, power or test anything,
and nothing here is safe or ready for use on a person or animal. The sheet keeps its NOT FOR CONSTRUCTION notice.

### R3.1 In plain language

Revision 0.3 adds four protections to the output bridge and changes two settings.

- **A shorted bottom switch is now caught within about 0.1 µs of the fault current reaching 3 A, and the top switch
  is told to turn off at once** instead of 20–36 µs later. A second, much faster current detector watches the
  current-measuring resistor directly; the logic sheet turns the top switches off at once; and on each top switch's
  floating gate supply a small "carrier-loss" circuit empties the supply as soon as the drive signal stops. Simulated together, the top switch that has to survive the fault absorbs
  **9–25 mJ** and stays inside its 150 °C rating by both of the project's temperature estimates in every case we
  ran — but at the very worst combination of part tolerances the pessimistic estimate reaches the rating with no
  margin left. Only the switch maker's data can say which estimate is right.
- **The short-circuit trip is lowered from 90 to 80 A,** so the switch never has to interrupt more than about
  100 A (its pulse rating is 107 A), and **a pair of film capacitors next to the bridge** holds the voltage down when
  it does: the switch saw at most **2.77 kV against its 3.3 kV rating** with up to 1 µH of cable to the main
  capacitor. Without them it would see up to 3.6 kV.
- **The small supply for the bottom switches is now watched.** If it sags below about 17 V the logic ends the
  sequence; the bottom switch is opened while its gate is still fully on, instead of drifting half-on for a
  fraction of a millisecond.
- **The new fast detector does not trip during a normal shock or while the charger runs** in the simulation, with a
  margin of 1.7 A or more against its most sensitive setting.
- Several details of the change order did not work as written and were changed on this sheet (R3.7); some
  problems need the design authority (R3.8). The most important: **none of the new protection is checked by any self-test**,
  so each part of it can fail silently and only show up when a second fault arrives.

### R3.2 Changes applied (G-1 … G-8)

| # | What | Designators | As ordered? |
|---|---|---|---|
| G-1 | Over-current trip 90 → 80 A | R1351 174 k → **196 k** (3.3 V × 10.0/206 = 160.2 mV = 80.1 A) | yes |
| G-2 | `I_SHOCK` 24.0 mV/A; idle threshold re-scaled | R1355 10.0 k → **13.3 k**; R1358 237 k → **267 k** (119.1 mV = 4.96 A) | yes |
| G-3 | Fast idle path → `IDLE_FAST_N` | R1365 100 Ω, C1345 100 pF, **R1364 10.0 kΩ pedestal (added)**, D1332 **BAT54 (single, to GND)**, U1317 OPA838IDBVR (gain 11), C1349 decoupling, R1366 **1.00 kΩ**, R1367 **100 Ω**, U1318 TLV3501AIDBVR, C6301 decoupling, R1368 **6.73 kΩ**, R1369 1.00 kΩ, C1346 100 nF | **four departures**, R3.7 P-1 … P-4 |
| G-4 | `LSD_OK` monitor | R1360 100 k, R1361 12.1 k, C1343 100 pF, R1362 80.6 k, R1363 100 k, C1344 100 nF, U1316 TLV7031DBVR, C6302 decoupling (added) | yes (decoupling added) |
| G-5 | Carrier-loss detector, both islands | A: D1333, R1373, C1350, R1374, R1375, Q1312, D1334, R1376, D1335, Q1313, R1377; B: D1336, R1378, C1351, R1379, R1380, Q1314, D1337, R1381, D1338, Q1315, R1382 | yes; R1377/R1382 in 0805 (failure table) |
| G-6 | Local film capacitor HV_CAP_P–HV_RTN, 75 nF | C1347, C1348 940C30P15K-F in series through `HV_CAP_FM`; R1370, R1371 HVCR2512FTD10M0 10 MΩ | yes (footprint to be made) |
| G-7 | RC snubber kept | C1301, C1302, R1304 unchanged | yes |
| G-8 | Same-side shoot-through row in the failure table | §12 row Q1302 (QLA) amended; see R3.5 | yes |
| — | Test points for the two new low-voltage nets | TP1306 `IDLE_FAST_N`, TP1307 `LSD_OK` | addition (§15 allows) |
| — | Notes and descriptions | group notes `bridge`, `isense`; new groups `idfast`, `lsdmon`, `film`, `cla`, `clb`; R1352, R1356, R1359, U1313, TP1303 texts | — |

Sheet now 215 parts (was 166): +49. New nets: `IDLE_FAST_N`, `LSD_OK` (global, contract issue 3), and locally
`IDF_IN`, `IDF_FB`, `IDF_OUT`, `IDF_REF`, `LSD_SNS`, `LSD_TH`, `HV_CAP_FM`, `HSA_CLR/CLF/CLB/CLG/CLD`, `HSB_CL*`.
The `NOT FOR CONSTRUCTION` notice is unchanged. New references follow the change order (R1364, the "unused" one,
is now the pedestal; C6301/C6302 from the sheet's overflow block); C1349 (reserved in the order) is U1317's
decoupling; R1372 stays unused.

### R3.3 Thresholds and ratings of the new circuits [CALC]

| Circuit | Setting | Nominal | Worst-case band | Basis |
|---|---|---|---|---|
| OC trip (G-1) | R1351 196 k / R1352 10.0 k | 160.2 mV = **80.1 A** | ≈ 74–86 A | as §8 with the new divider |
| `I_SHOCK` (G-2) | INA240 × 20 × 20.0/33.3 | **24.0 mV/A** | gain ±0.2 %, divider 0.2 %, shunt ±2.3 % | saturated ≤ (5.5 − 0.2) × 0.6006 = 3.18 V < VDDA min 3.26 V |
| slow idle (G-2) | R1358 267 k / R1359 10.0 k | 119.1 mV = **4.96 A** | ≈ 4.5–5.5 A | as §8 |
| fast idle (G-3 as drawn) | pedestal 3.3 × 100/10,100 = 32.67 mV; reference 3.3 × 1.00/7.73 = 426.9 mV = 11 × 38.81 mV | 3.07 A (reference), **3.20 A rising** | **2.54–3.87 A** | TLV3501 ±6.8 mV/11 (±0.31 A), OPA838 ±0.23 mV (±0.12 A), 0.1 % parts (±0.11 A), rail ±1.5 % (±0.05 A), shunt (±0.07 A), hysteresis +0.14 A, bias 30 µV |
| fast idle delay | 100 Ω/100 pF 10 ns + amplifier pole 6 ns + TLV3501 ≤ 12 ns | **≈ 30 ns** to `IDLE_FAST_N` | + ≈ 45 ns on sheet 12 | ≤ 0.1 µs shunt-to-sheet-12 (contract §5.7) |
| fast idle linear range | output 0.36 V at 0 A … 3.1 V | linear to **125 A** (65.7 A → 1.80 V) | VOH ≥ VS+ − 0.2 V | no saturation in any planned shock |
| `LSD_OK` (G-4) | 1.827 V × 112.1/12.1 | **16.93 V** | **16.48–17.29 V** | rail 19.5–22 V normal, ≈ 18.1 V at BRIDGE_5V 4.5 V [EST] |
| carrier-loss detector (G-5) | 47 pF / 4.7 k ∥ (4.7 k + base) | holds through the 0.19 µs dead time | RECT < 1 V 1.1–1.3 µs after stop | `../rev3` §3 [SIM] |
| local film (G-6) | 2 × 0.15 µF in series, 10 MΩ each | 75 nF (67.5–82.5 nF) | 1.1 kV each at 2.2 kV; 2.2 kV + spike on a survivor | 3,000 Vdc parts |

`BRIDGE_3V3` load added by this revision ≈ 4.3 mA typical / 7.1 mA maximum (sheet 13 total ≈ 25–32 mA).

### R3.4 Results of the revision-0.3 check [SIM] (details: `sim/rev3-wpg/notes.md`)

**(1) Shorted low side, with the real fast path, kill latch and carrier-loss detector** (energy in the surviving
high side QHA; junction pessimistic / physical estimate; start 55 °C; rating 150 °C):

| | 1,950 V | 2,200 V |
|---|---|---|
| Diagonal (QLB shorted), 25–175 Ω, threshold anywhere in its band | 9.3–17.9 mJ, 84–114 / 59–63 °C | 11.1–23.0 mJ, 89–130 / 60–65 °C |
| Same side (QLA shorted) | 12.8–21.3 mJ, 99–125 / 61–64 °C | 14.9–**25.4 mJ, 106–139** / 61–66 °C |
| Worst single sensitivity (no shunt inductance, same side, 2,200 V, top of band) | — | **29.2 mJ, 150 / 67 °C** |

Margin: ≥ 11 K (pessimistic) and ≥ 84 K (physical) in all nominal cases; **0 K pessimistic** at the worst single
sensitivity, and a few kelvin over if several worst corners coincide [not run as one case]. Confidence medium-low
(the switch's behaviour below 10 µs is not published). Against `../bridge`'s ideal-parts estimate (11–13 mJ): about
2× at the top of the threshold band.

**The 1 µs vs 2.7 µs disagreement is settled at ≈ 1 µs:** at circuit level, in this fault, the PNP starts emptying
the gate 1.06–1.08 µs after the carrier stops whether the island is modelled at circuit level or with
`../bridge`'s behavioural PNP; the gate rises by < 0.1 V, not "keeps rising". The detector's benefit is a faster
gate fall (to 2 V in 4.0–4.5 µs instead of 7.4–7.8 µs), not an earlier start. Confidence medium.

If part of the new protection has failed silently (latent; matters only with a shorted low side): fast path dead
**50–104 mJ, 194–330 °C pessimistic / 75–96 °C physical**; detector inert 32–46 mJ, 130–165 / 67–73 °C; both dead,
same side **≈ 11–12 J — destroyed**.

**(2) Turn-off at the 80 A band with the 75 nF local film:** interrupted current 85–100 A (I_DM 107 A); peak switch
voltage **2.47–2.77 kV with 0.5–1 µH of cable, ≥ 533 V (16 %) under 3.3 kV**. Film parts: ≤ 101 A (rating 216 A),
≤ 673 V/µs (rating 1,440 V/µs; 821 V/µs with one part shorted), ≤ 1,278 V each, 2,485 V on the survivor if one
shorts (3,000 V). Ringing film–cable 510–720 kHz, ±180–355 V, still ±35–145 V after 12–25 µs (lightly damped).
**With the film string open: 3.13 kV at 0.5 µH and 3.59 kV at 1 µH — over the rating** (latent fault, R3.8).
Confidence medium.

**(3) False trips:** none — not in the lead, the gap or after phase 2, at 1,950 and 2,200 V, 25–175 Ω, with the
shunt sense-loop inductance at 0.3 or 1 nH, with the charger running, with the OPA838 at its worst low swing, or
in test mode. Worst margin **≥ 1.67 A above the most sensitive threshold (2.54 A)**, in the lead (the outputs'
charging current through the shunt, ≤ 0.57 A, plus the sense-loop inductance). Confidence medium (method note in
the sim notes: replay of the shunt waveform; in-circuit numerical spikes discounted).

**(4) `LSD_OK`:** band 16.48–17.29 V [CALC]; nuisance margin ≈ 0.8 V [EST]. MGJ2-only collapse: the low side is
opened 0.7–2.1 µs after `LSD_OK` falls, with its gate still at 15.8–16.9 V; linear time **2.1 µs + 25 µs partly on →
0.1 µs**; QLB **182 → 130 °C** (pessimistic; 1 ms decay at 25 Ω); 100 µs collapse 175 → 136 °C. Rail crossing to
low side open ≈ 5–6 µs on a slow ramp. A converter that only sags to 12–15 V and stays now ends the shock
(without the monitor it was thermally harmless in the model). Confidence medium.

### R3.5 Failure table: new parts (revision 0.3)

Same conventions as §12. "Latent" = nothing on the device notices it; it matters only together with a second
fault. "Blocks" = every sequence is refused or ends in a fault (`BR_FAULT_N`), which the firmware reports: a
failure to shock, visible. Consequence figures in brackets are from S-1 [SIM] where a case was run.

**Same-side shoot-through row for §12 (G-8).** Q1302 QLA *fails short* also means: at every phase-1 start QHA turns
on above it, straight across the capacitor through R_S + L_S (≈ 240 A, 11–12 J in QHA as captured in rev 0.2, BR Q1).
With rev 0.3 the fast path, the kill latch and the carrier-loss detector end it at 12.8–25.4 mJ (99–139 °C
pessimistic, 61–66 °C physical) [SIM S-1]; the §12 row for Q1302 is amended accordingly. The mirror case (QLB shorted,
QHB at phase 2) cannot arise: a shorted QLB is caught at phase-1 start.

**Fast idle path (U1317, U1318 and their network)**

| Ref | Fails open | Fails short | Detected by / when |
|---|---|---|---|
| R1365 100 Ω | `IDF_IN` pulled to 3.3 V by R1364 → amplifier saturates → `IDLE_FAST_N` low → blocks | pedestal lost (IDF_IN tied to SHUNT_KP): fast threshold rises to ≈ 19 A → fast path useless | open: visible (start refused). **short: latent** → shorted-low-side protection falls back to the slow path [S-1 "fast path dead"] |
| R1364 10.0 kΩ (pedestal) | pedestal lost → threshold ≈ 19 A | `IDF_IN` at 3.3 V through 100 Ω → saturates → blocks (33 mA through R1365) | **open: latent** (as R1365 short); short: visible |
| C1345 100 pF | no 10 ns filter: more sensitive to edge spikes; S-3 margin falls | IDF_IN = SHUNT_KN: no signal, no pedestal → never trips | open: latent (margin only); **short: latent** |
| D1332 BAT54 | negative excursions reach the OPA838 input through 100 Ω (≤ 10 mA, its continuous limit) | IDF_IN = GND → never trips | open: latent; **short: latent** |
| U1317 OPA838 | dead or output stuck low → never trips | output stuck high → blocks | **stuck low: latent**; stuck high: visible |
| U1317 PD pin (solder open) | "pin must be driven" (SBOS867E): state undefined, may power down → output low → never trips | — | **latent** |
| C1349 | decompensated amplifier without local decoupling may ring → nuisance trips | BRIDGE_3V3 shorted → sheet 12 rail lost → nothing starts | open: visible if it trips, otherwise latent; short: visible |
| R1366 1.00 kΩ | open loop → output to a rail (high with any positive offset) → blocks | gain 1 → threshold ≈ 200 A → never trips | open: visible (most likely); **short: latent** |
| R1367 100 Ω | follower, gain 1 → never trips | open loop → saturates → blocks | **open: latent**; short: visible |
| U1318 TLV3501 | output low (12's pull-down) → blocks | output stuck high → never trips | low: visible; **stuck high: latent** |
| C6301 | comparator noise / oscillation → nuisance trips | BRIDGE_3V3 short | visible |
| R1368 6.73 kΩ | IDF_REF → 0 → trips always → blocks | IDF_REF = 3.3 V → never trips | open: visible; **short: latent** |
| R1369 1.00 kΩ | IDF_REF → 3.3 V → never trips | IDF_REF → 0 → blocks | **open: latent**; short: visible |
| C1346 100 nF | noisier threshold | IDF_REF → 0 → blocks | short: visible |

**LSD_OK monitor (U1316)**

| Ref | Fails open | Fails short | Detected by / when |
|---|---|---|---|
| R1360 100 kΩ | LSD_SNS → 0 → `LSD_OK` low → blocks | 20.7 V on U1316 IN+ (absolute limit 7 V) → part likely damaged, output state unknown | open: visible; **short: latent if the damaged output sits high** |
| R1361 12.1 kΩ | LSD_SNS pulled to the rail (0.2 mA, inside the input's fault-tolerant range is NOT guaranteed above 7 V) → reads "good" always | LSD_SNS = 0 → blocks | **open: latent**; short: visible |
| C1343 100 pF | ground-bounce filtering lost → possible nuisance trip at switching edges → sequence ends | LSD_SNS = 0 → blocks | visible (either way the device refuses) |
| R1362 80.6 kΩ | LSD_TH → 0 → always "good" | LSD_TH = 3.3 V → blocks | **open: latent**; short: visible |
| R1363 100 kΩ | LSD_TH → 3.3 V → blocks | LSD_TH = 0 → always "good" | open: visible; **short: latent** |
| C1344 100 nF | noisier threshold | LSD_TH = 0 → always "good" | **short: latent** |
| U1316 TLV7031 | output low → blocks | output stuck high → monitor lost | **stuck high: latent** → MGJ2-only failure behaves as in BR Q4 (158–181 °C) [S-4 mon = 0] |
| C6302 | noise | BRIDGE_3V3 short | visible |
| (not a part) −5 V rail | **not monitored**: if the −5 V side fails, the low sides are held off at 0 V instead of −5 V; dv/dt immunity 6.5 V → ≈ 1.5 V margin (Miller clamp still acts) | — | **latent** — raised, see R3.8 |

**Local film capacitor (C1347, C1348, R1370, R1371)**

| Ref | Fails open | Fails short | Detected by / when |
|---|---|---|---|
| C1347 / C1348 (or a lead / solder joint) | the whole 75 nF is gone: an over-current turn-off with 0.5–1 µH of cable reaches **3.2–3.7 kV, over the 3.3 kV rating** [S-2 "film open"] | the other part carries 2.2 kV + spike alone (S-2: see table) — within 3,000 V; its balancing resistor now 0.48 W | **open: latent and unsafe with a long cable** (R3.8); short: latent |
| both C1347 and C1348 short (double fault) | — | HV_CAP_P shorted to HV_RTN on the board: the capacitor (≈ 160–190 J) discharges into the short and the cable | double fault; the reason there are two in series |
| R1370 / R1371 10 MΩ | the other part's balancing resistor is then the only DC path: the unbalanced part takes up to the full DC voltage (film insulation ≥ 100,000 MΩ·µF) | the other capacitor carries the full voltage (as a shorted capacitor) | latent |

**Carrier-loss detector, per island (A refs; B identical with D1336–R1382)**

| Ref | Fails open | Fails short | Detected by / when |
|---|---|---|---|
| D1333 BAV70 (both halves) | no carrier copy → detector "sees" carrier loss all the time → Q1313 on while the carrier runs → HSA_RECT loaded by 100 Ω: gate ≈ 13 V instead of 18–19 V, R1377 ≈ 2 W while the high side is on | one half shorted: S1 or S2 drives CLR directly, the copy swings negative → Q1312 base reverse-biased via 10 k (VEB limit) | **latent** (slower, weaker high side; F-5 class: the self-test does not measure gate voltage) |
| R1373 4.7 kΩ | as D1333 open | faster copy; hold still set by C1350/R1374 | open: latent (weak gate); short: none |
| C1350 47 pF | dead time no longer bridged: Q1313 may blip on each 0.19 µs gap (10 kΩ × Ciss slows it) → small extra island load | CLF = KS → as D1333 open | latent |
| R1374 4.7 kΩ | CLF discharges only through the base: release slower → detector acts later (towards the "detector dead" result) | CLF = KS → as D1333 open | latent |
| R1375 10 kΩ | Q1312 never on → as D1333 open | more base current, none | latent |
| Q1312 MMBT3904 | as R1375 open | C–E short: CLG held low → **detector inert** | **latent** → S-1 "detector dead" result |
| D1334 BAT54 | no Baker clamp: saturation storage delays the release by ≈ 0.1–0.3 µs [EST] | B–C joined: CLG ≈ 0.7 V < 2N7002 V_th min → inert | latent |
| R1376 10 kΩ | Q1313 gate never pulled up → **inert** | CLG = gate: while the carrier runs Q1312 shorts the QHA gate → high side never turns on → phase lost (blocks); Q1312 overloaded | open: latent; short: visible (self-test phase current) |
| D1335 BZX84C15 | Q1313 gate can see up to 21 V (VGS limit 20 V) → slow damage | CLG = KS → inert | latent |
| Q1313 2N7002 | **inert** | D–S short: HSA_RECT loaded by 100 Ω permanently → as D1333 open | latent |
| R1377 100 Ω | **inert** | Q1313 empties C1315 directly (faster; 2N7002 pulse current from 1 nF only) | open: latent; short: none |

**Test points TP1306, TP1307:** a probe on them is not a part fault.

**Summary — what is now latent:** the new protection is entirely self-test-blind. A failed fast path, a failed
(inert) carrier-loss detector, an `LSD_OK` comparator stuck at "good", and an open film capacitor all look
exactly like a healthy device until a second fault (a shorted low side, an MGJ2 failure, a short at the pads with a
long capacitor cable) arrives. The existing self-test (§B14.3, D-62 T-steps) exercises none of them.

### R3.6 Datasheets consulted for revision 0.3

Opened and read in this work (27 November 2026; copies kept outside the repository):

| Part | Document | What was taken from it |
|---|---|---|
| OPA838 (U1317) | TI SBOS867E, "OPA838 1mA, 300MHz Gain Bandwidth, Voltage-Feedback Op Amp", Aug 2017, revised **Aug 2025** | DBV pinout (Fig 5-1, Table 5-1), OPA838IDBVR orderable (Package Option Addendum), 2.7–5.4 V, minimum gain 6 V/V, GBP 240 min / 300 typ MHz, SR 250 min V/µs, offset ±125 µV (25 °C) / −230…+220 µV (−40…85 °C), bias 3.7 µA max, input range VS− − 0.2 V typ (VS− over temperature) to VS+ − 1.3 V, **VOL VS− + 0.1 typ / + 0.2 V max over temperature**, VOH VS+ − 0.2 V min, overdrive recovery 50 ns, Iq ≤ 1.35 mA, PD "must be driven", enable > VS− + 1.5 V, abs max input VS± 0.5 V, **differential ±1 V**, 10 mA |
| TLV3501 (U1318; existing U1313, U1315) | TI SBOS321E, Mar 2005, revised **Apr 2016** | offset ±6.5 mV max **at 25 °C only** (drift ±5 µV/°C typ), hysteresis 6 mV typ, VOS = mean of the two switching points, input range (V−) − 0.2 to (V+) − 0.2 V, delay 4.5/6.4 ns (20 mV overdrive) and **7.5/10 ns, 12 ns over temperature, at 5 mV overdrive**, Iq 3.2 typ / 5 mA max, SHDN enabled below V+ − 1.7 V |
| TLV7031 (U1316) | TI SLVSE13J, Sep 2017, revised **Nov 2024** | offset ±8 mV, hysteresis 2–17 mV (7 typ, 25 °C), input range VEE to VCC + 0.1 V, inputs fault-tolerant to 7 V with VCC = 0, delay 3 µs at 100 mV overdrive, output held low below 1.6 V (POR), 335/900 nA, power-up 200 µs |
| 940C30P15K-F (C1347, C1348) | Cornell Dubilier "Type 940, Polypropylene Capacitors, for Pulse, Snubber" catalogue (revision date not printed; PDF metadata 2010–2021) | 0.15 µF, 3,000 Vdc / 500 Vac, D 27.0 × L 46.0 mm, lead Ø 1.2 mm, ESR 8 mΩ typ, ESL 32 nH typ, **dV/dt 1,440 V/µs, I peak 216 A**, I rms 10.1 A (100 kHz, 70 °C), full voltage to 85 °C then linearly to 50 % at 105 °C, ±10 % (K), "-F" = insulated stranded wire leads (41 mm), insulation ≥ 100,000 MΩ·µF, life 60,000 h at rated Vdc and 70 °C. **Mass not stated** |

Not opened in this work (values used as read by others, or from the earlier capture) — **unverified here**:

| Part | Used | Source of the figure |
|---|---|---|
| Stackpole HVCR2512FTD10M0 (R1370, R1371) | 3,000 V working, 10 MΩ in range | WP-F's reading of sei-hvcr.pdf rev 9/22/2025; the URL refused this work's fetch (HTTP 403). **Power rating not checked** |
| onsemi MMBT3904LT1G (Q1312/Q1314), BAV70LT1G (D1333/D1336), 2N7002LT1G (Q1313/Q1315, NDS7002A/D: V_th 1.0–2.5 V as read by WP-D for sheet 09), BAT54LT1G (D1332, D1334/D1337), BZX84C15LT1G (D1335/D1338; same family document WP-G read for the earlier capture, Rev 24) | pinouts from KiCad's stock symbols (Q_NPN_BEC, Q_NMOS_GSD, BAV70, BZX84Cxx; BAT54 drawn with the BAT54W symbol, pins A–NC–K) | **unverified**: the BAT54 SOT-23 order (1 A, 3 K) carries a `Verify` field |
| Murata MGJ2D052005SC | line regulation 1.2 %/%, load envelope (for the LSD_OK margin) | WP-G's earlier reading of an undated mirror (§2) |
| Microchip MSC080SMA330B4N, GeneSiC GC05MPS33J, TI UCC5350, INA240 | unchanged from §2 and `sim/bridge` | as before |

### R3.7 Problems with the change order, and what was changed on this sheet

| # | Item | Problem (source) | Done here |
|---|---|---|---|
| P-1 | G-3 fast path, zero-current level | With the input at 0 V the OPA838's output cannot go below **VS− + 0.1 V typ, + 0.2 V max over temperature** (SBOS867E §6.6 VOL). The ordered threshold sits at 66.4 mV, so at zero current the comparator would read "current" — `IDLE_FAST_N` permanently low and **every sequence refused** | **R1364 10.0 kΩ pedestal** from BRIDGE_3V3 into `IDF_IN`: 32.7 mV at zero current → amplifier output 0.36 V (≥ 0.16 V above the worst VOL); threshold re-set by **R1368 48.7 kΩ → 6.73 kΩ** (3.07 A falling / 3.20 A rising nominal). Pedestal and reference are both ratiometric to BRIDGE_3V3. Uses the "unused" R1364 |
| P-2 | G-3 gain resistors 10.0 k / 1.00 k | OPA838 bias current up to 3.7 µA (−40…85 °C) through Rf∥Rg = 909 Ω = **3.4 mV input error ≈ 1.7 A**, half the threshold | **R1366 1.00 kΩ, R1367 100 Ω** (gain still 11): the inverting input sees 91 Ω against 99 Ω at the non-inverting input; error ≈ 30 µV |
| P-3 | G-3 clamp "BAT54S as D1331" | the upper diode, reverse-biased by 3.3 V from BRIDGE_3V3, injects its (hot) leakage into a 99 Ω node: µA × 99 Ω is a sizeable part of a 6 mV threshold | **D1332 = single BAT54 to GND** (the positive excursion is bounded by D1303 and the OPA838's own input limits through 100 Ω) |
| P-4 | G-3 band "2.6–3.5 A" | TLV3501's ±6.5 mV is specified **at 25 °C only** (drift typ only); worst-case sum with 0.1 % parts, ±1.5 % rail, shunt ±2.3 %, OPA838 −230/+220 µV, 6 mV hysteresis: **2.54–3.87 A** | band re-stated; S-1 run at 2.54/3.0/3.5/3.87 A. Raising the gain to ≈ 31 would narrow it to ≈ 2.7–3.6 A but saturates the amplifier in every shock — not done (R3.8 item 4) |
| P-5 | G-3 delay "≤ 7 ns comparator" | TLV3501 at 5 mV overdrive: 7.5 typ / 10 ns max (25 °C), 12 ns over temperature | delay ≈ 10 ns RC + 6 ns amplifier + ≤ 12 ns ≈ 30 ns: still ≤ 0.1 µs |
| P-6 | G-4 band "16.5–17.3 V" | TLV7031 offset ±8 mV and hysteresis 2–17 mV (SLVSE13J) | 16.48–17.29 V [CALC] — agrees; decoupling C6302 added |
| P-7 | G-6 film part | 940C30P15K-F is **27 mm × 46 mm, axial, with 41 mm insulated stranded leads ("-F")**, two of them; mass not stated. Lead inductance (≈ 40–60 nH each [EST]) adds to the 32 nH ESL | simulated with 10 and 60 nH extra per part: 75 nF still holds the spike (S-2). Footprint marked to be made |
| P-8 | G-3/G-4/G-5 BRIDGE_3V3 load "≈ 4.5 mA" | OPA838 ≤ 1.35 mA, TLV3501 ≤ 5 mA (3.2 typ), pedestal 0.33 mA, references 0.45 mA, TLV7031 < 1 µA | **≈ 4.3 mA typ, 7.1 mA max** added; sheet 13 total ≈ 25–32 mA (WP-D's hold-up sizing) |
| P-9 | G-5 R1377/R1382 size | a discharge FET stuck on with the carrier running puts ≈ 2 W into the 100 Ω | drawn in 0805; still not rated for continuous operation in that fault (failure table) |
| P-10 | "BAT54" in G-5 | KiCad has no plain BAT54 (SOT-23 single) symbol | drawn with the stock BAT54W symbol (same A–NC–K pin order), MPN BAT54LT1G, SOT-23; `Verify` field |

### R3.8 Problems for the design authority (not fixable inside this sheet)

1. **No self-test reaches the new protection (and none reached the old comparators or the kill latch).** A failed
   fast path, an inert carrier-loss detector, an `LSD_OK` comparator stuck at "good" and an open film capacitor
   are all latent (R3.5), and S-1 shows what a second fault then costs (fast path dead: 50–104 mJ, 194–330 °C
   pessimistic; both dead, same side: ≈ 11–12 J, destroyed). Nothing using only the existing contract nets can
   exercise them: `BTEST_EN` and the test relay give 0.29 A at 60 V, far below every threshold, and any injection
   tied to `BTEST_EN` would latch a fault in every bridge test. **Proposed (contract change):** three supervisor
   lines `ITEST_OC`, `ITEST_IDF`, `ITEST_IDS` (05 → 13, 100 kΩ to GND on 13, active high), each through a
   BAS116 and a resistor into one comparator input (≈ 1.2 kΩ into `OC_SNS` → ≈ 200 mV ≡ 100 A; ≈ 22 kΩ into
   `IDF_IN` → ≈ 12 mV ≡ 6 A; ≈ 15 kΩ into `IDLE_SNS` → ≈ 170 mV ≡ 7 A) [CALC, not simulated], the same pattern as
   `OVP_TEST` on sheet 08. In TEST_MODE with the rail up and no command, each line alone must set the fault latch
   (`BR_FAULT_N` low); with `CMD_HA` held high at ≤ 60 V, raising one must make `OUT_A_TAP` fall although
   `CMD_HA` stays high — that proves the kill latch, the carrier stop and the PNP turn-off. A fourth injection
   (BAS116 + resistor pulling `LSD_TH` above `LSD_SNS`) would prove `LSD_OK`. Leakage of the BAS116s into the fast
   path in normal operation: ≤ 5 nA × 99 Ω, negligible. The carrier-loss detector itself would still be seen only
   by the turn-off time of `OUT_x_TAP` (gate < 2 V in ≈ 4 µs with it, ≈ 7.5 µs without), which needs a ≥ 1 MS/s
   capture on sheet 07 — capability unknown.
2. **An open film capacitor is latent and takes the switch over its rating with a long cable** (S-2: 3.13 kV at
   0.5 µH, **3.59 kV at 1 µH**). Either a cable-inductance limit (≤ 0.2 µH keeps 2.73 kV with the film open) goes
   into `docs/requirements/storage-capacitor.md`, or the film branch is made redundant (a second series pair in
   parallel — four 27 × 46 mm parts; C1349/R1372 were reserved but C1349 is now used for decoupling). Owner/
   design-authority decision on board volume.
3. **`HV_CAP_FM` falls into net class `HV` (0.5 mm track) by the `*HV_*` pattern, but it carries the film's
   ≈ 100 A ringing current.** It should be `HV_PULSE`; `hierarchy.toml` is not this package's file.
4. **Gain-31 option for the fast path (P-4).** Narrower band (worst corner ≈ 3.6 A) at the price of the amplifier
   saturating during every shock (overload recovery is specified only at 2× overdrive, 50 ns). Not taken; decision
   for the design authority if the S-1 worst corner (R3.4) is judged too thin.
5. **Standing load on the capacitor:** R1370 + R1371 add 20 MΩ (0.24 W at 2.2 kV, ≈ 0.2 W at 2 kV); CS's
   "board standing load 3.90 MΩ" (rev3-changes S-12) should include it (→ ≈ 3.3 MΩ on the board).
6. **Layout requirement L-1:** the GND pins of U1317/U1318 (and U1313/U1315) must be taken at the shunt's
   HV_RTN Kelvin terminal `SHUNT_KN`; the OPA838 tolerates only VS− − 0.2 V at its input, and a remote ground with
   25 nH of return path saw tens of volts of bounce in the model.
7. **Film–cable ringing** (S-2): 510–720 kHz, Q ≈ 20–30, ±350 V at a trip with 1 µH, and at every hard phase
   edge. Not over a rating; an RC damper (a few ohms in series with a larger film) would remove it. Needs a decision
   with point 2 because it is the same board area.
8. **`LSD_OK` ends a shock when the gate supply only sags** (S-4) — conservative, as ordered; the −5 V rail is
   not monitored (a failed −5 V side leaves the low sides held off at 0 V; dv/dt margin 6.5 V → ≈ 1.5 V [CALC]).
9. **Contract §5.7 text** for `IDLE_FAST_N` says "band ≈ 2.6–3.5 A": the drawn circuit's band is 2.54–3.87 A.

### R3.9 What only the switch maker's data or a measurement can settle

- **Which junction estimate is right** for 3–5 µs linear-mode events: Microchip's Zθ below ≈ 7 µs (or a
  low-voltage-scaled pulse measurement by a qualified lab). It decides whether the S-1 worst corner (150 °C
  pessimistic, 67 °C physical) is a real limit, and whether a silently failed fast path (194–330 °C pessimistic,
  75–96 °C physical) would destroy the switch.
- Turn-off of 86–100 A at 2.5–2.8 kV (no RBSOA published) — unchanged from `../bridge`.
- The switch's saturation current at kilovolts (sets the same-side shoot-through current; sensitivity −6 %).
- The WSK2512's sense-loop inductance (0–1 nH changed the S-1 energy by −40 %/+15 % and the S-3 margin by ≈ 0.4 A)
  and the real stray/cable inductances.
- The made-to-order gate transformers' output resistance and their behaviour when the drive stops (not modelled).
- The MGJ2's actual output at 4.5 V input and its ripple (the `LSD_OK` nuisance margin is ≈ 0.8 V).
- The 2N7002/MMBT3904/BAV70/BAT54 parts' real switching behaviour (generic models; detector timing 1.1–1.3 µs).
- The film capacitors' behaviour under the repetitive ringing (rms within the 10.1 A rating on paper).
