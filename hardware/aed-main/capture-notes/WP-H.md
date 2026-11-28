# Capture notes — WP-H: patient relay, pad interface and the two front ends (sheets 14, 15, 16)

**NOT FOR CONSTRUCTION — unreviewed design.** Captured 27 Nov 2026 with AI assistance from
`docs/HARDWARE_DESIGN.md` v0.2 and `hardware/aed-main/contract.md` issue 2. Nothing here has been reviewed by a
qualified high-voltage or medical-device engineer, built, simulated in circuit form or measured. Everything on these
three sheets touches the patient. This file contains no assembly, test or bring-up instructions (SAFETY.md §2a).
Nothing here is safe or ready for use on a person or animal.

Tags: **[CALC]** arithmetic shown here · **[EST]** estimate, basis stated · **[BK]** background knowledge not re-checked
· **[UNVERIFIED]** looked for and not confirmed · **[SIM]** only simulation or measurement can settle it.

Files: `hardware/aed-main/specs/14_patient_interface.toml`, `15_frontend_main.toml`, `16_frontend_sup.toml`,
`hardware/lib/src/aed_patient.toml` (and the generated `.kicad_sch` / `.kicad_sym` / PDFs).

---

## 1. What is on each sheet (plain language)

**Sheet 14 — patient relay and pad connector.** The two pads plug into a made-to-order connector (J1401). From there
each pad goes to the common contact of a two-pole high-voltage relay (K1401). With the relay at rest, the pads are
connected only to the two measuring circuits (`PT_FE_A`, `PT_FE_B`); only when its coil is powered are they
connected to the shock output (`HV_OUT_A`, `HV_OUT_B`). The relay needs a 12 V coil, but the battery gives 5.5–8.4 V,
so the sheet has a small 12 V booster. The booster and everything behind it are disconnected from the battery by a
transistor switch except when the shock sequencer (sheet 12) raises `K_EN`, so it draws nothing in standby. A
coil-current controller (DRV110) gives the coil full current for 33 ms to pull it in and then about half the current
to hold it, which halves the heat; a large capacitor store keeps the relay held for well over 30 ms if the booster or
battery fails in the middle of a shock; and a clamp diode makes the relay let go quickly when `K_EN` falls. A
separate, mechanically linked low-voltage contact inside the relay closes to ground only when the relay is at rest
and reports that as `K_REST_N` (low = at rest; a broken wire reads "not at rest"). The connector's low-voltage
contacts carry the pad set's identity chip (1-Wire, `PAD_ID_1W`) and a presence loop (`PAD_PRESENT_N`), each with a
series resistor and a surge clamp. A temperature sensor near the pads (`NTC_PADS`) feeds the pad-heat record (D-52).

**Sheet 15 — main front end (ADS1292R).** One chip measures both the heart signal (channel 2) and the pad resistance
(channel 1, by passing a small 64 kHz test current through the patient and measuring the voltage it makes). Each pad
reaches the chip through two separate paths ("taps"): a **sense** tap (a defibrillator-rated 51 kΩ resistor, a surge
clamp to ground, then small resistors to the chip) and a **drive** tap (two high-voltage capacitors in series that
block any direct current, two defibrillator-rated 24 kΩ resistors in parallel, a clamp, then a 2 kΩ resistor to the
chip's test-current output). These parts are what let the chip survive the device's own shock if the relay fails,
and an outside defibrillator's 5 kV shock while the pads are on. With only two pads and no third "right-leg"
electrode, the chip's bias amplifier holds the patient at mid-supply through very large (10 MΩ) resistors.

**Sheet 16 — supervisor's front end (MAX30009).** A second, different chip from a different maker gives the
supervisor its own pad-resistance reading, with the same kind of protection on each tap (20 kΩ in the drive taps).
It runs from its own 1.8 V regulator (`1V8_SUP`, made here from `3V3_SUP`), and a small level translator lifts its
1.8 V data output to the supervisor's 3.3 V. Its test current is about 30 µA. **I recommend changing its test
frequency from 128 kHz to 96 kHz** (§3.3): at 128 kHz it would leak into the main chip's heart-signal channel.

---

## 2. Parts list

Datasheets read 27 Nov 2026 unless stated. "Formula MPN" = an orderable code built from the maker's part-numbering
scheme for a common passive; value and rating are certain, the exact catalogue line and stock were not checked.

| Ref | Part (MPN) | Maker | Key ratings used | Datasheet (as read) |
|---|---|---|---|---|
| U1501 | ADS1292RIPBSR (TQFP-32) | TI | AVDD 2.7–5.25 V, DVDD 1.7–3.6 V, −40…+85 °C, resp. 32/64 kHz, modulator ≤ 100 µA | ti.com/lit/ds/symlink/ads1292r.pdf (SBAS502C, rev. Apr 2020; addendum 31 Oct 2025: Active) |
| U1601 | MAX30009ENA+T (WLP-25) | ADI | AVDD/DVDD 1.7–2.0 V, **−40…+85 °C**, BioZ 16 Hz–500 kHz | Rev 1 (2/22) read from a third-party mirror (pdffile.jiepei.com …MAX30009_BW398032.pdf): **analog.com refused automated access**; Rev 2 (8/23) not read |
| U1602 | TPS7A2018PDBVR | TI | 1.6–6 V in, 300 mA, 7 µVrms, −40…+125 °C TJ | ti.com/lit/ds/symlink/tps7a20.pdf (SBVS338H) |
| U1603 | SN74LVC1T45DBVR | TI | dual-supply translator, Ioff | ti.com/lit/ds/symlink/sn74lvc1t45.pdf (stock KiCad symbol; datasheet not re-read [BK]) |
| U1401 | TPS61089RNRR | TI | VIN 2.7–12 V, VOUT 4.5–12.6 V, 7 A switch, no output disconnect | ti.com/lit/ds/symlink/tps61089.pdf (SLVSD38C, Aug 2021) |
| U1402 | DRV110APWR (TSSOP-14) | TI | VIN ≥ 6 V (15 V clamp), peak/keep/hold, 20 kHz, EN floats high | ti.com/lit/ds/symlink/drv110.pdf (SLVSBA8G, Mar 2018) |
| Q1401 | DMP3013SFV-7 | Diodes | −30 V, VGS ±25 V, 17 mΩ @ −4.5 V | diodes.com/assets/Datasheets/DMP3013SFV.pdf (DS40141 Rev 5-2) |
| Q1402 | 2N7002LT1G | onsemi | 60 V logic N-FET (needs only 0.1 mA) | onsemi.com 2n7002l-d.pdf [BK, not re-read] |
| Q1403 | DMN3027LFG-7 | Diodes | 30 V, **VGS ±25 V**, 26.5 mΩ @ 4.5 V, AEC-Q101 | diodes.com/assets/Datasheets/DMN3027LFG.pdf (DS38020 Rev 3-2, "Advance Information") |
| K1401 | **made to order**, basis Sensata/Gigavac G12-12Vdc + force-guided aux contact | TBD | G12: DPDT vacuum, 8 kV op / 10 kV test, 20 mΩ, 10 A carry, 15/9 ms, 12 V 60 Ω | sensata.com G12 sheet Rev 3 (5/3/18), read in a browser; G17 from Internet Archive copy of gigavac.com g17.pdf (04/02/14) |
| J1401 | **made to order** pad connector | TBD | 2 energy + 3 LV contacts | design §B7.3 |
| R1507/08, R1605/06 | DPCR2512-51KJT18 | TT Electronics | 51 k 5 %, 1.5 W, **100 pulses 5 kV peak (IEC 60601-2-27)**, −55…+155 °C | ttelectronics.com …/Datasheets/DPCR.pdf (09.26) |
| R1501–R1504 | DPCR2512-24KJT18 | TT Electronics | as above; 24 k is "other E24 value" | same |
| R1601, R1602 | DPCR2512-20KJT18 | TT Electronics | as above (standard value) | same |
| C1501–04, C1601–04 | 3640J3K00472KAT (formula MPN) | Knowles Syfer | 4.7 nF C0G 3 kV, 3640, AEC-Q200 | knowlescapacitors.com AEC-Q200 range, Issue 16 (27/05/26) |
| D1501–04, D1601–04 | CDSOD323-T03C | Bourns | bidirectional, VWM 3.3 V, VBR ≥ 4.0 V, 7 V @ 1 A, 350 W 8/20 µs, 3 pF, 5 µA | bourns.com …/cdsod323-txxc.pdf (REV 04/23) |
| D1505 | SZESD5Z3.3T1G | onsemi | VRWM 3.3 V, VBR ≥ 5.0 V, 240 W | onsemi.com esd5z2.5t1-d.pdf (Rev 16) |
| D1410–12 | TPD1E10B06DPYR | TI | bidirectional, VRWM 5.5 V, 12 pF, 6 A 8/20 µs | ti.com/lit/ds/symlink/tpd1e10b06.pdf (SLLSEB1G) |
| D1401, D1402 | SS34-E3/57T | Vishay | 40 V 3 A Schottky, IFSM 100 A | vishay.com/docs/88751/ss32.pdf |
| D1403 | SMBJ10A-E3/52 | Vishay | VBR 11.1–12.3 V, 600 W 10/1000 µs | vishay.com/docs/88392/smbj.pdf (09-Jan-2024) |
| C1410, C1411 | UHE1C332MHD | Nichicon | 3300 µF 16 V, 12.5 × 35.5 mm, −40…+105 °C, leakage ≤ 528 µA rated | nichicon.co.jp …/e-uhe.pdf |
| L1401 | XAL7030-222MEC | Coilcraft | 2.2 µH; Isat [BK, verify ≥ 9 A] | coilcraft.com xal7030.pdf (not read) |
| R1419 | WSL1206R2500FEA | Vishay Dale | 0.25 Ω 1 % 0.5 W | vishay.com/docs/30100/wsl.pdf [BK] |
| R1609 | RG1608P-101-W-T1 | Susumu | 100 Ω 0.05 % thin film | susumu.co.jp catalogue [BK] |
| TH1401 | NCP18XH103F03RB | Murata | 10 k 1 %, B25/50 3380 K 1 %, −40…+125 °C | pim.murata.com NCP18 spec |
| FB1501 | BLM18PG221SN1D | Murata | 220 Ω @ 100 MHz ferrite | [BK] |
| others | Yageo RC0603FR-07… resistors; Murata GRM… capacitors | — | value/voltage/dielectric as in the pin tables | formula MPNs [BK] |

Totals: **134 parts** (sheet 14: 51, sheet 15: 51, sheet 16: 32); **100 distinct nets** across the three sheets
(14: 40, 15: 40, 16: 26 per sheet, sharing `GND`, `PT_FE_A`, `PT_FE_B`), **353 pin connections**, all matched by the
netlist round trip. Footprints still to be made: `aed_patient:Maxim_WLP-25_2.03x2.03mm_P0.4mm`,
`aed_patient:PatientRelay_MadeToOrder`, `aed_patient:PadConnector_MadeToOrder`.

---

## 3. Arithmetic

### 3.1 Defibrillation protection, per tap

Two threats: **(a)** this device's own shock reaching a front end if the relay or its timing fails (fault 29: one
pole welded at rest) — up to **2,182 V** (OVP comparator top, design §B4.4), biphasic, up to ~20 ms (≤ 28.8 ms in
the frozen-timer fault, §B13.6), repeated ≤ 20 times in 1,300 s; **(b)** an external defibrillator with the pads on:
IEC 60601-1 8.5.5 test pulse, 32 µF at 5 kV through 500 µH, ~4.72 kV peak, 9 µs rise, 2.36 ms to half (research
notes `hv_safety_isolation.md`, MEDTEQ [UNVERIFIED against the standard]). Decay time constant ≈ 2.36 / ln 2 ≈
**3.4 ms** [CALC]. I use 5 kV as the peak for margin.

Current path: a tap from pad A to its clamp to `GND`, then from `GND` through the other pad's taps to pad B. "One-
sided" = the whole voltage across one tap (the other side's paths conduct strongly at first); "balanced" = half each.
Clamp voltage ≈ 5–7 V for 0.1–1 A (CDSOD323-T03C: 7 V max at 1 A).

**Sense taps (51 k DPCR, sheets 15 and 16):**

| | (a) own shock 2,182 V, 20 ms | (b) external 5 kV pulse |
|---|---|---|
| Peak current | 2,182 / 51 k = **43 mA** | one-sided 5,000 / 51 k = **98 mA**; balanced 49 mA |
| Peak voltage on the resistor | 2.18 kV | ≤ 5 kV (DPCR test: 5 kV peak) |
| Power / energy | 93 W × 20 ms = **1.9 J** (28.8 ms: 2.7 J) | one-sided V²τ/2R = 25e6 × 3.4e-3 / 102e3 = **0.83 J**; balanced 0.21 J |
| Rating | single-pulse curve ≈ 400 W at 10 ms, ≈ 250 W at 20 ms (read from the graph) → ≥ 2.5× | "100 pulses, 5 kV peak, ΔR ≤ 1 %" to IEC 60601-2-27 (waveform not stated in the datasheet [UNVERIFIED]); curve ≈ 1.5 kW at 1 ms vs 490 W peak |
| Clamp | 43 mA for 20 ms, ≈ 5 mJ | ≤ 98 mA for a few ms, ≈ 2 mJ |
| Chip pin current | (6.5 − 3.6) V / 10 k = 0.3 mA (ADS); 6.5 V / 20 k = 0.33 mA (MAX) | same order |

**Drive taps, main (2 × 4.7 nF in series = 2.35 nF; 2 × 24 k in parallel = 12 k; 2.0 k inner):**

| | (a) | (b) |
|---|---|---|
| Edge current into the clamp | 2,182 / 12 k = 0.18 A, τ = 12 k × 2.35 nF = 28 µs | ≤ 5,000 / 12 k = **0.42 A**, τ 28 µs |
| Energy into the resistors per voltage change | ½ C V² = ½ × 2.35 nF × 2,182² = 5.6 mJ; at phase reversal (≈ 3.3 kV swing) ≈ 13 mJ | ½ × 2.35 nF × 5,000² = 29 mJ on the rise and again on the decay, split between two 24 k → ≈ 15 mJ each |
| Peak voltage / power per 24 k | 2.18 kV / 0.2 kW | **≤ 5 kV / 1.0 kW** for ~30 µs (DPCR pulse-tested at 5 kV; single-pulse curve ≈ 5.5 kW at 0.1 ms) |
| Capacitor voltage | ≤ 1.1 kV each (two in series) | ≈ 1.25 kV balanced, ≤ 2.5 kV one-sided; one capacitor shorted (single fault): ≤ 2.5 kV balanced on the survivor — inside 3 kV, but **one-sided + one shorted = 5 kV on one 3 kV part** (residual, §8) |
| Clamp | 0.18 A × 6 V × ~30 µs ≈ 30 µJ | 0.42 A × 7 V × 30 µs ≈ 90 µJ (vs 350 W × 20 µs ≈ 7 mJ 8/20 µs rating) |
| Chip pin (RESP_MOD) | (6.5 − 3.6) / 2.0 k = **1.5 mA** for ~30 µs (abs max ±10 mA continuous, ±100 mA momentary) | same |

**Drive taps, supervisor (20 k DPCR, 10 k inner):** edge current ≤ 0.25 A, τ = 47 µs, ≈ 29 mJ in one resistor per
edge (b), ≤ 5 kV; pin current ≤ (7 V − 0) / 10 k = 0.7 mA (MAX30009 abs max ±50 mA; its pins clamp to AGND).

**Rails.** The TVS clamps cannot hold a tap below the chips' own limits (no 3.3 V TVS has VBR < 4.0 V; ADS1292R pins
are limited to AVDD + 0.3 V; MAX30009 pins to 2.2 V), so the **inner resistors are the real protection**: worst
total injection into `AVDD_AFE` ≈ 2 × 0.3 + 2 × 1.5 ≈ 3.6 mA for tens of µs (sense part for ms), which passes
through the ferrite into `3V3_MAIN` (main-domain load tens of mA when on). Into `1V8_SUP` ≤ ~1.1 mA, against the
1.8 mA guaranteed sink of R1610 + the chip's own current. `D1505` (ESD5Z3.3, VBR ≥ 5.0 V) is a backup only. With
the main domain **off** during an external shock, the ~3.6 mA would lift the unpowered `3V3_MAIN` until D1505
(≥ 5.0 V) conducts [residual, §8].

**Recovery to a usable reading [EST].** Own shock: the front ends are disconnected for the whole sequence; on
return (coil off at +1 ms, release ≤ 9–15 ms) they see only the post-shock electrode polarisation. External shock:
drive-tap capacitors discharge with τ 28 µs (main) / 47 µs (supervisor); the respiration coupling capacitors
(2.2 nF, 5 MΩ bias) recover with τ ≈ 11 ms (≈ 55 ms to settle); ECG digital-filter settling is 4 conversions
(16 ms at 250 /s); electrode polarisation (hundreds of mV, seconds) dominates, hence the design's gain-3 recovery mode.
The IEC 60601-2-4 recovery requirement was not read [UNVERIFIED]; real recovery is a measurement item [SIM].

### 3.2 Patient auxiliary and leakage current [CALC]

Limits: the research notes record **no** numeric leakage limit; design §B7.4 says "several mA at those frequencies
[UNVERIFIED limit]". For orientation only, **[BK, UNVERIFIED]** IEC 60601-1 patient auxiliary current for a type CF
applied part is 10 µA DC/AC normal and 50 µA single fault (type BF 100/500 µA), with the limit rising in proportion to
frequency above 1 kHz up to 10 mA.

| Source | Normal condition | Single fault (worst) |
|---|---|---|
| Main drive taps, DC | 0 (two capacitors in series; C0G leakage ≪ 1 nA) | 0 (one capacitor shorted, the other still blocks) |
| Main drive, 64 kHz carrier | 2.42 V / 30.4 kΩ = **80 µA** peak square (≈ 72 µA RMS fundamental) | one 24 k of a pair shorted: 2.42 / ~18 kΩ ≈ 0.13 mA; inner 2 k shorted: 0.09 mA |
| Supervisor drive, 96 kHz (or 128) | 1.8 / (2 × 30 k) = **30 µA** square | 20 k shorted: 1.8 / 40 k = 45 µA |
| Sense taps, DC | both pads biased equally through 10 MΩ from RLDOUT: ≈ 0; with 300 mV electrode offset ≤ 15 nA | ADS input latched to a rail: 3.3 V / (2 × 61 k) = **27 µA**; MAX pin to rail: 1.8 V / (2 × 71 k) = 13 µA |
| MAX sense pins, DC | (1.65 − ~0.9) V / 48 MΩ ≈ 15 nA | — |
| Relay coil supply, aux contact, ID, NTC | not galvanically connected to the patient (relay insulation) | relay insulation breakdown: outside this table (HV spacing item) |

Normal total: ≈ 0.1 mA RMS at 64–128 kHz (well inside a frequency-scaled limit of 0.6–1.3 mA for CF), DC < 0.1 µA.
Single-fault DC worst 27 µA: inside 50 µA (CF) [BK limits]. The design's 0.34 mA RMS figure is not reached because
the chip cannot drive it (§3.4).

### 3.3 Two impedance chips on one pair of pads

**Mutual loading [CALC].** Seen by the main at 64 kHz, the supervisor's network across the pads is its two drive taps
(20 k + 10 k + 2.35 nF ≈ 30 kΩ each) ≈ **60 kΩ** (its sense taps, 71 kΩ into ~48 MΩ, are negligible): error at
250 Ω = 250 / 60 k = **0.4 %**. Seen by the supervisor at 128 kHz: the main's drive taps 2 × (12 k + 2 k + 0.53 kΩ)
≈ 29 kΩ, its ECG inputs ≈ 2 × 61 k + 470 pF ≈ 125 kΩ → ≈ 24 kΩ in parallel → **1.0 %** at 250 Ω, 0.7 % at
175 Ω. Both far smaller than the design's 6 % (which assumed 1 k drive resistors). The value changes when the other
chip changes state (driving / idle / unpowered) by at most these amounts, so calibrating each channel with the other
running its normal carrier (F-9) leaves a residual ≤ ~1 %.

**Demodulator rejection [CALC].** Main: square-wave demodulation at 64 kHz (odd harmonics 64, 192, 320 … kHz).
Supervisor: square-wave I/Q demodulation at its carrier F. For F = 128 kHz the review's argument holds (no product
near DC), and for F = 96 kHz too: 96 kHz has odd harmonics 96, 288, 480 …, every one ≥ 32 kHz from any odd
harmonic of 64 kHz, with either chip's clock error (ADS ±1.5 %, MAX ±2.5 %).

**A problem the review missed: 128 kHz aliases into the main ECG channel.** The ADS1292R's modulator samples at
fMOD = 128 kHz, and its digital filter's pass band repeats at every multiple of 128 kHz (SBAS502C §8.3.5.1). The
supervisor's 128 kHz carrier (≈ 7.5 mV at 250 Ω) and its harmonics at 384, 640 kHz … land at (clock difference)
≈ 0–5 kHz after sampling — a tone of uncontrolled frequency, possibly inside the 0.5–40 Hz ECG band. The ECG input
filter (470 pF with 61 k per side, 2.8 kHz) and the PGA filter (≈ 8.5 kHz) attenuate it by ≈ 46 × 15 ≈ 700 →
≈ 10 µV, which is about 10 % of a fine ventricular-fibrillation signal [EST]. **At 96 kHz** every odd harmonic is
32 × (odd multiple of 3) kHz, which samples to 32 or 96 kHz, never near DC, and the decimation filter removes it.
The main's own 64 kHz carrier samples to 64 kHz (Nyquist) and is removed likewise. 96 kHz is reachable exactly on the
MAX30009: PLL 24.576 MHz = 750 × 32.768 kHz (inside 14–28 MHz), KDIV 1, DAC_OSR 256 [CALC from datasheet Table 1
equations]. It is above the charger's ≤ 43 kHz fundamental. **Proposed change: supervisor carrier 96 kHz, not
128 kHz** (Contract issue 1).

**Conclusion.** The two chips can share the pads **with that one change**; as specified (128 kHz) they would load
each other acceptably but the supervisor would inject an in-band interference tone into the main ECG.

### 3.4 Noise and resolution

**Main impedance channel (ADS1292R, channel 1, 64 kHz).** Excitation is limited by the chip: datasheet "maximum
modulator current, internal reference: 100 µA" (SBAS502C p.8); current = VREFP / loop impedance (Eq. 10). The
design's 0.34 mA was impossible. Here 2.42 V / 30.4 kΩ = **80 µA** [CALC]. Resolution: 24 bits, ±2.42 V / gain 3 →
LSB 96 nV → 96 nV / 80 µA = **1.2 mΩ per step** (requirement ≤ 2 mΩ). Noise [EST]: the datasheet's 40 mΩ p-p
(0.05–2 Hz, 30 µA, gain 4) ≈ 6 mΩ RMS ≈ 0.18 µV RMS ≈ 0.13 µV/√Hz; plus sense-resistor thermal noise (2 × 61 kΩ →
45 nV/√Hz, ×√2 for demodulation folding = 64 nV/√Hz); total ≈ 0.145 µV/√Hz × √9.5 Hz (0.5–10 Hz) ≈ 0.45 µV RMS →
0.45 µV / 80 µA ≈ **5.6 mΩ RMS** (requirement < 10 mΩ). The 0.3 Ω compression threshold gives 24 µV, ≈ 50 × the
noise. Carrier at 250 Ω: 20 mV, far inside ±0.8 V. Margin ≈ 1.8× on noise: real but not large; charger interference
is not in this figure (§3.6, design §B3.7).

**ECG (channel 2, gain 6, 250 /s).** Chip 0.7 µV RMS (Table 3) + sense resistors 45 nV/√Hz × √65 Hz ≈ 0.36 µV →
≈ **0.8 µV RMS**; LSB 48 nV. Firmware's `EcgSample` is i16 µV (±32.7 mV): ample.

**Supervisor (MAX30009, 30 µA H-bridge).** 10 mΩ per count (1 V / (2¹⁹ × gain 10 × 2/π × 30 µA)) [CALC]. Noise
[EST]: datasheet 0.80 mΩ RMS at 1.28 mA RMS, 25 Ω, 0–62.5 Hz → × 42.7 for 30 µA ≈ 34 mΩ, plus 2 × 71 kΩ thermal
≈ 13 mΩ → ≈ **37 mΩ RMS**: ample for permit decisions at 25 / 175 Ω, not for compression sensing (not its job).
Absolute accuracy depends on the 5 % DPCR drive resistors (current set by them): factory calibration at the pad
connector required (F-9).

### 3.5 Relay coil supply and timing

**Coil.** Sized for the worst coil considered (G17 class: 12 V, 12 Ω ±10 %, pick-up ≤ 8 V, drop-out 0.5–5 V; hot
copper → up to ~15 Ω). A G12-class coil (60 Ω) draws only 0.2 A and simply runs at 100 % duty.

**Economiser (DRV110).** RSENSE 0.25 Ω; IPEAK = 0.9 V × 66.67 k / 280 k / 0.25 = **0.857 A** (pick-up needs 8 V /
12 Ω = 0.67 A); IHOLD = 0.15 V × 66.67 k / 80.6 k / 0.25 = **0.496 A** (worst drop-out 5 V / 12 Ω = 0.42 A); keep
time 0.33 µF × 10⁵ = **33 ms**; PWM 20 kHz default. Hold power 0.5² × 12–15 Ω = 3–3.7 W (≈ 25–30 % of 12 W).

**Boost.** Output 1.212 × (1 + 887/100) = **11.96 V**. Peak load ≈ 0.86 A × 11.6 V + 0.3 W ≈ 10.3 W → at η ≈ 0.9:
**2.1 A** from 5.5 V, 1.6 A from 7.4 V (matches design's 2.0 A). Current limit 1.03 M / 127 k = 8.1 A (7.3–8.9 A).

**Input disconnect and inrush.** Reservoir precharge as Q1401 turns on: Miller slew ≈ (7.4 − 2.5) V / 47 k − 2.5 V /
100 k ≈ 79 µA into 68 nF → 1.2 V/ms → 6.6 mF × 1.2 V/ms ≈ **7–8 A for ~6 ms**; then the boost charges the reservoir
from ~6.6 V to 11.6 V at its current limit (~3 A net out, ~8 A in) for ≈ 10 ms. So the coil supply draws up to
~8 A from `VBAT_SYS` for ~15 ms per transfer — **above the design's 2.0 A budget**, inside the pack protector's
OC ≥ 30 A (contract issue 7) [EST; Q1401 SOA UNVERIFIED].

**Hold-up (requirement: relay held ≥ 30 ms if the boost or `VBAT_SYS` fails mid-sequence).** In the peak phase the
DRV110 is at 100 % duty and the coil is a resistor: the reservoir decays with τ = R × C; in hold, current is
regulated at 0.5 A and the coil needs V ≥ 0.5 × R + ~0.3 V. With C = 6.6 mF, starting at 11.6 V:
- boost lost at the earliest sequence start (~20 ms after `K_EN`, ≤ 13 ms of peak left), hot coil 15 Ω: τ = 99 ms →
  11.6 e^(−13/99) = 10.2 V; then hold to 7.8 V: ½ × 6.6 m × (10.2² − 7.8²) = 143 mJ / 4.05 W = 35 ms → **≈ 48 ms**;
- boost lost during hold, hot coil: ½ × 6.6 m × (11.6² − 7.8²) = 244 mJ / 4.05 W → **≈ 60 ms**;
- nominal 12 Ω coil: longer in both cases.
4.4 mF gives ≈ 30 ms in the first case (no margin); the design's 1.7 mF assumed a constant 3 W and no peak phase.

**Timeline from `K_EN` [EST, SIM].** Q1401 ramp ≈ 6 ms (the coil already sees ~6.5 V through the boost's body diode,
not enough to pull in); boost brings `K_VCOIL` to 8 V at ≈ +9–10 ms; G17-class operate 15 ms → contacts move at
≈ +20–25 ms; `K_REST_N` high; +10 ms → sequence may start at ≈ **+30–35 ms** — inside the sequencer's 60 ms start
window and the main's 40 ms abort (F-4), with only ~5 ms of margin on the latter. The 90 ms on-time cap and 60 ms
window are on sheet 12 (contract §8); this sheet obeys `K_EN` only.

**Fast release.** Coil voltage at turn-off ≈ −(0.5 + 12.3) V instead of −0.5 V. Drain peak ≈ 13.2 V (`K_VCOIL`
max: OVP 13.6 V − diode) + 12.3 V + ~1 V clamp rise + 0.5 V ≈ **27 V** vs 30 V VDS (90 %). DRV110 gate drive ≤
VIN ≤ 13.2 V vs Q1403 VGS ±25 V.

### 3.6 Charger interference (component choice only)

Carriers above the charger's ≤ 43 kHz fundamental (64 and 96 kHz); ECG input filtered at 2.8 kHz (differential RC)
and 8.5 kHz (PGA); C0G in every signal-path capacitor; star return per design. The respiration path cannot be
low-pass filtered below its carrier. Harmonics of the charger near 64 kHz (respiration) and near multiples of
128 kHz (ECG sampling) remain a **measurement item** (design §B3.7, §16 rank 4).

---

## 4. Patient relay search (result)

Full search report in this session (agent search of Sensata/Gigavac, TE Kilovac, Cynergy3, Pickering, Standex-Meder,
Jennings): **no relay publishes all three of: a carry/pulse rating for ~65 A / 15 ms (or 100 A / 20 ms, or the 465 A
fault), ≥ 5 kV standoff, and a force-guided or mirror auxiliary contact. No HV vacuum or gas relay found has any
auxiliary contact.** Every Sensata/Gigavac sheet gives only continuous carry and says "consult factory" for load
switching.

| Candidate | Form / kV | Carry, contact R | Operate / release | Coil | Notes |
|---|---|---|---|---|---|
| Gigavac G17 (design's reference) | DPDT SF6, 7.5 kV op / 17 kV test | 10 A, **1.0 Ω max** | 15 / 9 ms | 12 V 12 Ω, 100 ms max on, 10 % duty | "Heart defibrillator patient relay" for sinusoidal-waveform units; **not in Sensata's current catalogue** (likely discontinued [UNVERIFIED]); 140 g |
| **Sensata/Gigavac G12** (captured basis) | DPDT vacuum, 8 kV op / 10 kV test | 10 A, **20 mΩ** | 15 / 9 ms | 12 V 60 Ω | best DPDT fit; 71 g; −55…+125 °C |
| Gigavac GH2 ×2 | SPDT vacuum, 8 / 10 kV | 25 A, 10 mΩ | 6 / 6 ms | 12 V 80 Ω | poles not linked: each needs its own position sense |
| G2SPD / G8SPD | DPDT vacuum, 15 / 17 kV | 50 / 30 A, 12–25 mΩ | **20** / 8 ms | 12 V 60 Ω | too slow; 160 g |
| TE Kilovac KM-17 | DPDT SF6, 7.5 kV | 10 A, 0.5 Ω | 20 / 20 ms | 12 V 12 Ω | **obsolete** |
| TE Kilovac H-19 | DPDT vacuum, 20 kV | 30 A, 15 mΩ | 30 / 20 ms | 12 V 48 Ω | too slow |
| Reed relays (Cynergy3 D, Pickering, Meder HM) | SPST, 10–15 kV | 2–5 A | 2–3 ms | — | cannot carry the shock |

Captured: K1401 as a **made-to-order** DPDT on a G12-class basis with a force-guided LV aux contact linked to both
poles. What remains unverified: every pulse/fault carry figure, pole-to-pole and contact-to-coil standoff (only
"contacts and to base" is rated; coil hi-pot is 500 V RMS), contact behaviour (repulsion/welding) at 465 A, life
under shock load, and whether any maker will add the aux contact.

---

## 5. Per-part failure table (patient side)

M = main controller sees it; S = supervisor sees it; "cross-check" = firmware compares M's and S's pad readings.

| Part | Open | Short |
|---|---|---|
| K1401 contacts | NC open: one front end pair loses that pad → both readings open-circuit → "attach pads" (M, S). NO open: shock path open → `I_SHOCK` ≈ 0 → Fault | NC welded (one pole, fault 29): with the aux contact linked to both poles the relay reads "not transferred"→ sequencer refuses; if linked to one pole only, ≈ 2 kV reaches one pad's taps (= threat (a), rated §3.1) and `I_SHOCK` ≈ 0 → Fault. NO welded: pads on `HV_OUT` at rest, front ends disconnected → open reading, `K_REST_N` high → M fault (F-6); no energy (bridge off) |
| K1401 aux contact | reads "not at rest" → M faults at switch-on (F-6); sequencer would read "transferred" after 10 ms of `K_EN` (contract issue 6) — no energy can reach the patient because the HV contacts are still at rest; `I_SHOCK` ≈ 0 → Fault | reads "at rest" always → sequencer never sees transfer → refuses every shock (detected: Refused); test-mode check passes |
| K1401 coil | relay never moves → Refused (detected) | coil short → DRV110 limits current to 0.86 A peak; boost into short limited by ILIM; relay never moves → Refused |
| J1401 energy contact | open pad → readings open → "attach pads" | pad A–B short at the connector → both read ≈ 0 Ω → "pads shorted" (M, S) |
| R1410/D1410 (presence) | open → "not fitted" (safe) | D1410 short → "fitted" with no pad set: the 1-Wire ID read fails → firmware fault [needs F-requirement: presence AND valid ID] |
| R1411/D1411 (ID) | ID unreadable → firmware fault | ID line shorted → unreadable → fault |
| R1412/D1412 (`K_REST_N`) | as aux contact open | D1412 short → as aux short |
| Q1401 input switch | coil supply dead → relay never moves → Refused | supply always connected: reservoir and DRV110 sit at `VBAT_SYS`; DRV110 EN held low → no coil; **standby drain ≈ 0.25 mA + capacitor leakage, not detected** except by the gauge's standby current |
| Q1402 | Q1401 never on → Refused | Q1401 always on → as above |
| U1401 boost | no 12 V: coil at ~6.5 V from the battery, may not pull in → Refused; if it does pull in, hold works | VOUT shorted → input current-limited; relay never moves → Refused |
| C1410/C1411 reservoir | open one (3.3 mF): worst case (hot coil, boost lost early in the peak phase) ≈ 20 ms, below the 25 ms cap — the relay could open under current only if the boost ALSO fails mid-sequence (second fault); not detectable in normal use [CALC as §3.5] | short: boost into a short, `K_VCOIL` ≈ 0 → Refused; Q1401 sees ILIM current |
| D1401 | open: coil unsupplied → Refused | short: boost sees 6.6 mF directly (outside its datasheet range, loop [SIM]); reservoir can discharge back into the boost output → hold-up lost if the boost fails short |
| U1402 DRV110 | gate low → relay never moves → Refused | output stuck high → coil on whenever `K_VCOIL` is up (only while K_EN powers the supply, plus reservoir decay after) → relay may stay transferred for up to ~80 ms after `K_EN` falls; front ends disconnected, `K_REST_N` high → M fault; no energy (bridge off) |
| Q1403 | as DRV110 gate low | coil energised while the reservoir holds charge (≈ τ 80 ms after any sequence) and whenever `K_EN` is high; at rest the reservoir is 0 V → no coil; detected via `K_REST_N` after a shock (relay late to return) [design fault 44] |
| D1402/D1403 (release clamp) | open: drain voltage unclamped at turn-off → Q1403 avalanche (≥ 30 V), possible Q1403 short (above) | short: plain diode or zero clamp → slow release (≈ tens of ms), still after the sequence ends; release time check [F-requirement proposed: measure `K_REST_N` return time] |
| TH1401 | open/short → reads out of range → M flags pad temperature unknown | same |
| DPCR sense resistor (51 k) | that pad's ECG and the chip's sense lost → impedance reads open/very high → "attach pads" (detected by that chip; cross-check catches a one-chip open) | **service item**: tap becomes 10 k (main) / 20 k (MAX) to the pad; the inner resistor then takes the defib current (rating exceeded) — normal readings unchanged, **not detectable** until it fails [residual] |
| DPCR drive resistor (main pair) | one of two open: current 80 → ≈ 67 µA → main reads ≈ 17 % low; **not detectable by M alone, caught by the M–S cross-check** | one shorted: current ≈ 0.13 mA (> chip's 100 µA max), reads high; caught by cross-check; protection on that tap reduced to the inner 2 k (exceeds its rating under (b)) |
| DPCR drive resistor (MAX, 20 k) | supervisor reads open → S refuses permit (fails safe: no shock) | current 45 µA → S reads ≈ 33 % low; cross-check |
| Drive capacitor (any one) | tap open → that chip reads open → refuses / "attach pads" | other capacitor still blocks DC; series capacitance doubles, reading barely changes; **not detectable**; under (b) one-sided the survivor sees up to 5 kV vs 3 kV rating [residual] |
| Tap TVS | no clamp: the inner resistor and chip pin take the event (main drive: up to ~2 A momentary through 2 k → chip damage under (b)) — not detectable in normal use [residual] | tap shorted to GND: that pad's sense/drive collapses → reading wrong or open → detected by that chip (out of range) and the cross-check |
| Inner resistors (2 k / 10 k / 20 k) | that path open: ECG lost (flat line, firmware lead-off/flat-line check) or impedance open | pin current under defib rises (≤ 100 mA momentary through the TVS clamp voltage); normal readings unchanged; not detectable [residual] |
| R1513/R1514 (RLD 10 M) | patient DC floats on the other path → ECG offset drifts, may saturate → detected as saturation | one ECG input tied to RLDOUT through 61 k: CM fixed, differential DC ≤ offsets; ≤ 27 µA DC single-fault (§3.2) |
| C1505 (ECG filter) | more charger noise in ECG (not detectable as a fault) | ECG inputs shorted → flat ECG → firmware flat-line/lead check |
| U1501 ADS1292R | (dead) no data / DRDY → M fault | pin latched to a rail → ≤ 27 µA DC patient current (§3.2); readings wrong → cross-check |
| U1601 MAX30009 | no SPI reply → S fault, no permit | as above, ≤ 13 µA |
| U1602 LDO | `1V8_SUP` absent → no supervisor reading → no permit (fails safe) | 3.3 V on `1V8_SUP` → MAX30009 over abs max (2.2 V) → likely destroyed → no permit; patient current ≤ 3.3 V / 60 k ≈ 55 µA at the carrier only while it still drives |
| U1603 | no MISO → S fault, no permit | — |
| R1610 bleed | rail loses its guaranteed sink (only matters during an external shock) | `1V8_SUP` shorted → LDO current limit → no supervisor reading → no permit |

**HV side into a front end.** The only galvanic routes from the bridge to the front ends are through K1401: a welded
NC contact plus transfer of the other pole (fault 29), or contact-to-contact flashover inside the relay. Both are
covered by threat (a) in §3.1 (the taps survive 2,182 V for ≥ 20 ms on paper). **Relay left on the bridge side** (NO
welded or coil stuck on): front ends read open, `K_REST_N` high → M refuses analysis/shock (F-6).

---

## 6. Unverified pins, ratings and assumptions

**Pins.**
- MAX30009: from Rev 1 (2/22) via a third-party mirror; Rev 2 (8/23) not read. Figure 17 swaps DRVSJ/DRVXR balls
  against the pin table (both unconnected here). `PinoutVerified = NO` for every custom symbol.
- K1401 and J1401: project terminal numbers (no maker numbering exists).
- DMN3027LFG: pins from the 2015 "Advance Information" sheet; stock-footprint pad 5 = drain pads 5–8 + exposed pad.
- 2N7002 (stock symbol, G-S-D = 1-2-3) and SN74LVC1T45 (stock symbol) not re-read against their datasheets.
- ADS1292R pins confirmed (SBAS502C pin table). DRDY push-pull vs open drain **not stated** (treated as push-pull).

**Ratings / values.**
- DPCR pulse rating is stated only as "100 pulses, 5 kV peak, IEC 60601-2-27"; the test waveform and a joule figure
  are not given. 24 k is an "other E24 value" (availability). Lead time ~20 weeks (distributor listing).
- 4.7 nF C0G 3 kV exists only in 3640 (Knowles); MPN formula-derived; no surge rating published by any maker.
- No TVS has a published rating for ms-duration pulses (all 8/20 µs).
- ADS1292R: RESP_MOD output resistance not stated; 47 nF vs 4.7 nF CFILTER contradiction; respiration noise at
  64 kHz not specified (scaled from the 32 kHz figure); no two-electrode RLD guidance.
- MAX30009: pin state when unpowered not stated; whether the CAL-port calibration applies in H-bridge mode;
  lifecycle (ADI says production; one distributor lists ENA+ as discontinued at that distributor).
- TPS61089: output capacitance above 1,000 µF (the reservoir behind D1401) is outside the datasheet range [SIM].
- DMP3013SFV SOA during the ~7 A, ~6 ms precharge; XAL7030-222 saturation current; DMN3027LFG production status.
- Relay: everything in §4.
- Leakage limits: §3.2 values are [BK, UNVERIFIED]; the standard has not been bought.
- 1-Wire series-resistor guidance from ADI (DS9503 / app notes) could not be read.

**Assumptions.** Electrode offset ≤ 300 mV; coil hot resistance ≤ 15 Ω; relay operate 15 ms at full voltage; the
external test pulse as MEDTEQ describes it.

---

## 7. Departures from the design specification (and why)

1. **Drive-tap resistor**: not "1 kΩ pulse-rated" but **12 k (2 × 24 k DPCR in parallel) + 2.0 k** on sheet 15 and
   **20 k DPCR + 10 k** on sheet 16. Reasons: (i) the ADS1292R cannot drive more than 100 µA (a 1 k tap would ask for
   ~0.5 mA); (ii) under an external 5 kV pulse the drive resistor sees up to ~5 kV for tens of µs and no 1 k part was
   found rated above ~1.75 kV, while DPCR is pulse-tested at 5 kV (range starts at 20 k).
2. **Excitation and numbers in §B3.4**: 80 µA (main), 30 µA (supervisor), not 0.34 mA; noise ≈ 5.6 mΩ RMS and
   1.2 mΩ per step, not ≈ 1 mΩ and 0.3 mΩ. Still inside the firmware's < 10 mΩ and ≤ 2 mΩ.
3. **Tap clamps**: TVS to GND as specified, but its breakdown cannot be "below the rail" (none exists < 4.0 V); the
   inner resistors (2 k / 10 k / 20 k) limit chip pin currents instead.
4. **Rail shunt clamp**: on `AVDD_AFE` a 3.3 V TVS (backup only); on `1V8_SUP` **no TVS can work** (2.2 V abs max) →
   replaced by a 1 k bleed that guarantees the rail can sink the worst injected current.
5. **MAX30009 drive in H-bridge voltage mode** (ADI's own defibrillation-protected application) instead of current
   mode: current mode's compliance (0.2 V to AVDD − 0.2 V) cannot drive through tens of kΩ of protection.
6. **Supervisor carrier 96 kHz proposed** instead of 128 kHz (§3.3) — a register setting; no part changes.
7. **Hold-up 6.6 mF** (2 × 3300 µF) instead of ≈ 1.7 mF; **coil-supply input current up to ~8 A for ~15 ms**
   instead of 2.0 A; peak phase 33 ms (≤ 20 ms in the design) so pull-in completes while `K_VCOIL` is still rising.
   Weight: two 12.5 × 35.5 mm capacitors (~10–12 g [EST]) — at or above the design's +6–10 g for the whole coil supply.
8. **Input disconnect switch** (Q1401) added: a boost converter passes the battery to its output when off; needed for
   the zero-standby budget (design §11 "Standby 0").
9. **K1401 captured as made to order** on a G12 basis rather than "G17-class": G17 has 1 Ω contacts (would add up to
   2 Ω to the 5 Ω internal-resistance budget, D-23) and appears discontinued.
10. **1.8 V → 3.3 V translator** (U1603) for the MAX30009's data-out line, not in the contract's part list: its SDO
    is a 1.8 V output that a 3.3 V supervisor cannot read reliably.
11. Pad identity EEPROM (DS2431 class) is in the pad plug, not on this board; it is not drawn.

---

## 8. Contract issues

1. **Supervisor carrier 128 kHz (contract §10 "MAX30009 at 128 kHz", design §B3.6) aliases into the ADS1292R ECG
   channel** (its modulator samples at 128 kHz). *Proposed:* 96 kHz (PLL 24.576 MHz, KDIV 1, DAC_OSR 256); update §10
   and §B3.6, and the firmware's MAX30009 configuration (WP-B).
2. **ADS1292R drive level (design §B3.4, contract §8 sheet 15 "1 kΩ pulse-rated drive resistor").** The chip limits
   the modulator to 100 µA. *Proposed:* contract §8 sheet 15 to read "2 × 24 k DPCR in parallel + 2.0 k per drive
   tap"; sheet 16 "20 k DPCR + 10 k"; §9 WP-H "drive-tap resistor ≥ 1.5 kV / 10 µs" → "5 kV pulse-tested (DPCR)".
3. **Rail shunt clamps (§8 sheet 15, design §B3.2)** impossible as worded for `1V8_SUP`. *Proposed:* "each rail must
   sink the worst injected current (inner resistors sized for it; bleed where the load cannot)".
4. **Relay (§8 sheet 14 "G17-class with force-guided aux contact")**: no such catalogue part (§4). *Proposed:* record
   K1401 as a made-to-order item (design §14) with the requirements of §B7.2 plus "aux contact linked to both poles".
5. **Coil-supply budget (design §11 "Coil boost input 2.0 A for 20 ms")**: actual ≈ 8 A for ≈ 15 ms (reservoir
   precharge + charge) then ≈ 2 A. *Proposed:* WP-A to confirm `VBAT_SYS` tolerates the dip, or accept a slower
   charge (longer transfer, less F-4 margin).
6. **R11 and `K_REST_N`.** An open or unsoldered aux contact reads "not at rest", which the sequencer (12) treats as
   "transferred" once `K_EN` has been on ≥ 10 ms — a "sequence allowed" state produced by an open circuit. No energy
   can then reach the patient (the HV contacts are still at rest, `I_SHOCK` ≈ 0 → Fault) and F-6 catches it at
   switch-on, but it is a literal R11 case. *Proposed (WP-D):* the sequencer also requires that `K_REST_N` was **low
   when `K_EN` rose** (it must see the low → high transition), so an open contact can never look like a transfer.
7. **F-4 timing.** The main aborts if `K_REST_N` + 10 ms takes > 40 ms; my estimate is ≈ 30–35 ms (§3.5) [SIM].
   *Proposed:* WP-B/WP-D consider 50 ms (still inside the 60 ms start window and the 90 ms coil cap).
8. **`AFE_RESET_N` default low (§5.8)** holds the ADS1292R powered down; fine, noted only: firmware must release it
   and wait the datasheet's power-up time (≥ 2¹² tMOD + 18 tCLK; 1 s in Figure 44) before configuring.
9. **Firmware requirements added** (for §F): ADS1292R CONFIG2.PDB_REFBUF = 1 (internal reference; power-up default is
   external), RESP_CTRL = 0, RESP_FREQ = 1 (64 kHz), channel 1 gain 3, RLDREF_INT = 1; MAX30009 MUX_EN = 0 whenever
   not measuring (drive pins disconnected — contract §10 "idle high-impedance"), DISABLE_I2C = 1, H-bridge mode,
   96 kHz; presence must be confirmed by a valid 1-Wire ID (a shorted presence TVS reads "fitted").

---

## 9. Things a reviewer should check first (my top five doubts)

1. **The relay does not exist as specified.** Every protection and timing figure on sheet 14 rests on a made-to-order
   relay whose pulse carry, kV between poles and to the coil, aux-contact linkage and coil figures are unknown.
2. **The 96 kHz carrier argument and the 128 kHz aliasing finding** (§3.3): if I have the ADS1292R's sampling
   behaviour wrong, the contract's 128 kHz may be fine — or the 64 kHz own-carrier and charger harmonics may also
   matter more than estimated. Needs a measurement or a TI confirmation.
3. **Protection under the external 5 kV pulse with real parasitics**: DPCR's rating is "5 kV peak per IEC 60601-2-27"
   without the waveform; the TVSs have no ms rating; the 3640 capacitors have no surge rating; capacitor one-sided
   stress with one capacitor shorted reaches 5 kV on a 3 kV part. [SIM + standard]
4. **Coil supply dynamics**: precharge inrush (~8 A), Q1401 SOA, TPS61089 behaviour while charging 6.6 mF through
   D1401, and the 30–35 ms transfer estimate against the 40 ms F-4 abort. [SIM]
5. **Impedance noise margin is only ≈ 1.8×** (5.6 vs 10 mΩ RMS), scaled from a 32 kHz datasheet figure, before any
   charger interference; and the two-electrode RLD bias arrangement has no datasheet guidance.

---

## 10. Not done

- No circuit simulation (SPICE) of the protection, coil supply or relay timing; no measurement of anything.
- MAX30009 Rev 2 datasheet not read; IEC 60601-1 / -2-4 / -2-27 not read.
- Custom footprints (WLP-25, relay, connector) not drawn (marked `footprint_todo`).
- Exact catalogue availability of formula-derived passive MPNs not checked.

---

## Revision 0.3 (27 Nov 2026)

**NOT FOR CONSTRUCTION — unreviewed design.** Applies change order WP-H (`rev3-changes.md` H-1…H-6) under
`docs/HARDWARE_DESIGN.md` v0.3, contract issue 3 and `docs/requirements/patient-relay.md` (D-64). Simulations:
`hardware/aed-main/sim/rev3-wph/notes.md`. All figures below marked [SIM] come from there; they are simulated with
hand-built models, not measured. Nothing here is safe or ready for use on a person or animal. Sections §1–§10
above describe issue 2 and remain valid except where this section says otherwise.

### R3.1 In plain language

- **Relay coil supply (sheet 14).** The relay's energy store is now filled before the shock is due, switched on by
  its own new signal from the main controller (`K_SUPPLY_EN`). The sequencer's `K_EN` now only starts the coil
  driver. The store fills more gently, through a larger slow-start capacitor and a lower current limit. In
  simulation the battery's surge falls from about 9 A to at most 3.3 A. On a cold, nearly empty battery the
  battery rail stays at 5.4 V instead of falling to 4.35 V. The relay changes over 18 ms after the command, and
  33.4 ms with the firmware's settling wait, inside the 40 ms limit. If the supply fails mid-shock, the relay
  stays held for at least 42.7 ms (30 ms required).
- **Relay.** The relay symbol is now clearly marked as a placeholder for a part that does not exist yet. The
  specification a maker must meet is `docs/requirements/patient-relay.md`.
- **Heart-signal and compression inputs (sheet 15).** The heart-signal branch has its own pair of
  defibrillator-rated input resistors and clamps, so it no longer swamps the compression (impedance) signal.
  Simulated compression noise is 4.6–6.3 mΩ against the 10 mΩ limit (charger off). Heart-signal quality and every
  protection figure are unchanged, and the new parts add no meaningful path for current through the patient.
- **Supervisor's chip (sheet 16).** Its test frequency is 96 kHz. That is a setting, so no part changed.
- **What I could not settle:** the new current-limit resistor value lies outside the range in the converter
  maker's chart. The changeover time depends on a relay coil nobody has measured: a slow coil would miss the 40 ms
  limit. The compression-noise margin rests on an estimated chip-noise figure.

### R3.2 Changes applied

| # | Sheet | Change | Designators |
|---|---|---|---|
| H-1 | 14 | Coil supply enable moved to new global **`K_SUPPLY_EN`** (04 → 14): R1403 now `K_SUPPLY_EN`–`K_SUPG` (local net renamed from `K_ENG`) to Q1402's gate; R1413 now `K_SUPPLY_EN`–`K_BEN` (boost EN). New **R1421 100 kΩ** `K_SUPPLY_EN`–GND (contract default). R1404 (`K_EN` default) and R1417 (DRV110 EN) stay on `K_EN` | R1403, R1413, R1421 |
| H-2 | 14 | **C1401 68 nF → 680 nF** X7R 0603 (MPN TBD, R3.6); **R1406 127 kΩ → 515 kΩ** 1 % (RC0603FR-07515KL, formula code); L1401 `Verify` lowered to Isat ≥ 3 A; Q1401 `Verify` updated for the slower pre-charge | C1401, R1406 |
| H-3 | 14 | K1401 value "DPDT HV relay + aux — MADE TO ORDER, PLACEHOLDER (docs/requirements/patient-relay.md)", `Verify` = "no part exists; D-64"; group title and note say placeholder; library symbol description, datasheet field and MPN say placeholder | K1401, `aed_patient:PATIENT_RELAY_DPDT_AUX` |
| H-4 | 15 | New **R1528, R1529 51 kΩ DPCR2512-51KJT18** from `PT_FE_A`/`PT_FE_B` to new nodes **`PT_ESA_CL`/`PT_ESB_CL`** (PATIENT class by the `*PT_*` pattern); **R1509/R1510 moved** to those nodes; new clamps **D1506, D1507 CDSOD323-T03C** to GND. R1511/R1512 stay on `PT_MSA_CL`/`PT_MSB_CL` | R1528, R1529, D1506, D1507, R1509, R1510 |
| H-5 | 16 | Carrier 96 kHz recorded as adopted (sheet comment and note); no circuit change | — |
| H-6 | 14 | Notes: coil-supply budget ≈ 2 A for ≈ 0.1 s before the shock; the boost limit is a peak value | — |
| — | 14 | New ERC acceptance: `K_SUPPLY_EN` / Q1402 / U1401 EN not driven until 04 is at issue 3 | — |

Every sheet keeps its NOT FOR CONSTRUCTION notice (build check). Parts: **52** on 14 (+1), **55** on 15 (+4), 32 on
16. Departures from the change order: none in values or topology; C1401's MPN is left TBD (none given, none
confirmed).

### R3.3 Results with margins [SIM unless marked]

| Check | Result | Limit | Margin |
|---|---|---|---|
| Reservoir at 11 V after `K_SUPPLY_EN` | 60–168 ms | < 200 ms lead (FW-12) | 32 ms |
| Battery peak, pre-charge (whole battery, incl. ≈ 0.6 A other loads) | 1.71–3.33 A | change order 2.5–2.6 A | +0.7 A at the +0.8 A limit corner |
| `VBAT_SYS` min, cold nearly empty battery (6.0 V, 80 mΩ/cell) | 5.40 V (issue 2: 4.35 V) | `5V0_SYS` dropout begins ≈ 5.2 V | 0.2 V |
| `5V0_SYS` / `BRIDGE_5V` min, all battery cases | 5.08 / 5.00 V | 4.9 / 4.60 V | 0.18 / 0.40 V |
| Transfer after `K_EN` (hot coil, 60 mH EST) | 18.1–18.4 ms | — | — |
| Transfer + 14 ms wait + 1 ms tick | 33.1–33.4 ms | 40 ms | **6.6 ms** |
| Same with a 120 mH hot coil | 43.4 ms | 40 ms | **−3.4 ms (fails)**; coil not known |
| Hold-up after a supply loss at `K_EN` + 20 ms | 42.7–63.1 ms (−20 % reservoir included) | ≥ 30 ms | 12.7 ms |
| Charger at full power during the pre-charge, cold nearly empty battery | `VBAT_SYS` 3.72 V (charger alone ≈ 4.3 V [CALC]) | — | see R3.7 item 4 |
| Impedance noise, charger off | 4.6–6.3 mΩ RMS (issue 2: 25–34) | < 10 mΩ | 1.6× with the estimated chip noise; 8.5–11.6 mΩ if chip noise is 2× |
| ECG gain 10 Hz / resistor noise 0.5–40 Hz / 64 kHz at IN2 | 0.993 / 0.28 µV / 0.043 | unchanged (0.989 / 0.28 / 0.043) | — |
| IN2P pin, external 5 kV / own 2,409 V | 0.45 / 0.79 mA (unchanged) | ±10 mA continuous (SBAS502C) | ≥ 12× |
| R1528 energy, external / own | 0.83 J (482 W peak) / 2.26 J (113 W) | DPCR ≈ 1.5 kW at 1 ms / ≈ 250 W at 20 ms (graph, §3.1) | 3.1× / 2.2× |

### R3.4 Defibrillation protection and patient leakage for each new part

**R1528, R1529 (51 kΩ DPCR).** They see exactly what R1507/R1508 see.
- External defibrillator with the pads on (5 kV, τ 3.4 ms), one-sided: **97 mA, ≤ 5 kV, 0.83 J per pulse**
  (balanced: 49 mA, 0.21 J) [SIM; matches CALC V²τ/2R = 0.83 J].
- This device's own shock through a failed relay (normally refused by the aux contact, fault 29), bounded at
  2,409 V (charger last-resort trip, E-1) for 11.85 + 8.06 ms (per-phase caps at the top of their bands, D-63):
  **47 mA, 113 W, 2.26 J** [SIM, rectangular bound; the issue-2 figure was 1.9 J at 2,182 V].
- A second external pulse adds the same again; DPCR is pulse-tested for 100 pulses at 5 kV.

**D1506, D1507 (CDSOD323-T03C).** Clamp ≈ 5.0–5.4 V [SIM]; ≤ 97 mA for milliseconds, ≈ 2 mJ (external) / ≈ 5 mJ
(own shock). This is the same stress as D1503, and outside the 8/20 µs rating basis [UNVERIFIED, as before].
The ECG pin current behind them is unchanged: 0.45 / 0.79 mA. Energy diverted from an external defibrillator:
resistive taps per pad 2 → 3 × 51 kΩ, pad to pad 51 → 34 kΩ, ≈ 0.3 % (was ≈ 0.2 %) of the energy into a 100 Ω
patient [CALC].

**Patient leakage arithmetic [CALC].**
- *Normal condition, DC.* The TVSs only sink to GND. The only DC sources that can push current through the
  patient are RLDOUT through R1513/R1514 (≤ 3.3 V / 10 MΩ = 0.33 µA per side) and the MAX30009 sense inputs
  (≈ 15 nA). So the patient DC current stays **≤ ≈ 0.35 µA** whatever the TVS leakage (≤ 5 µA each at 3.3 V,
  Bourns). Unchanged from issue 2.
- *Normal condition, 64 kHz.* The ECG branch is a load across the pads (2 × 61 kΩ + 470 pF ≈ 127 kΩ). It takes
  ≈ 20 mV / 127 kΩ = 0.16 µA of the 80 µA excitation and adds no source: the patient carrier current is
  unchanged (≈ 80 µA peak square).
- *Single fault (an ADS1292R input latched to a rail, worst case of §3.2).* The source is still limited by its own
  61 kΩ (10 kΩ + 51 kΩ, now through R1528/R1529). The return through the other pad's ADS pin gives 3.3 V /
  122 kΩ = 27 µA, as before. The new clamp adds at most its leakage to that return, ≤ 5 µA (less in practice,
  since it sits at ≤ 1.7 V, below its 3.3 V working voltage). Bound: **≤ 27 + 2 × 5 = 37 µA** (issue 2: 32 µA),
  against 50 µA single-fault CF [BK, UNVERIFIED limit].
- *Single fault D1506 short.* The ECG node of pad A is held at GND through 51 kΩ. No source drives it except RLD
  (0.33 µA): the ECG saturates or goes flat, which is detected; no new patient current.
- *`K_SUPPLY_EN`, R1421, C1401, R1406* are not connected to the patient. The coil now sits at ≈ 12 V for up to
  ≈ 10 s before each shock (charge + countdown) instead of only during it. Leakage through the relay's required
  ≥ 10 GΩ insulation is ≤ 12 V / 10 GΩ = **1.2 nA** (requirement figure; no relay exists).

### R3.5 Failure-table entries for the new and changed parts (add to §5)

| Part | Open | Short |
|---|---|---|
| R1528/R1529 (ECG sense 51 k) | that pad's ECG input drifts to RLDOUT via 10 M → ECG lead-off/saturated → firmware lead check; impedance unaffected (now independent), so it can read "pads good" while the ECG is lost: **firmware must treat ECG lead-off as "check pads"** [proposed F-requirement] | **service item**: tap becomes R1509's 10 k → R1509 and D1506 take the defibrillation current (exceeds R1509's rating under an external pulse); normal readings unchanged → **not detectable** [residual, as R1507] |
| D1506/D1507 | no clamp: IN2 pin protected only by 61 k and the chip's own diodes (≈ 82 mA at 5 kV one-sided → chip damage likely); not detectable in use [residual, as D1503] | ECG node to GND through 51 k → ECG saturated/flat → detected; no new patient current (R3.4) |
| R1509/R1510 (moved) | ECG input lost → lead check | as before (§5 inner resistors) |
| R1421 (`K_SUPPLY_EN` default) | `K_SUPPLY_EN` floats while the main is in reset: boost EN held low by its 800 k; Q1402 gate may drift → Q1401 partly on → standby drain (reservoir and DRV110 at VBAT_SYS), not detected except by the gauge; no coil without `K_EN` | `K_SUPPLY_EN` held low → reservoir never fills → relay too slow → refused; self-test T-3 detects |
| `K_SUPPLY_EN` stuck high / low (design fault 30a) | — | high: boost always on, ≈ 1 mA drain [EST], no coil without `K_EN`; low: refused, T-3 |
| C1401 (680 nF) | Q1401 turns on fast → issue-2 surge (8.7–9.9 A, LR §3.1) at `K_SUPPLY_EN`, i.e. **before** the sequence, not during it; not detected [residual: battery dip at charge start] | Q1401 gate tied to drain → runs diode-connected (K_VIN ≈ VBAT − 2 V); slower fill; transfer time logged by T-3 |
| R1406 (515 k) | ILIM pin open: limit not stated [UNVERIFIED] → possibly the full limit → surge as C1401 open | ILIM to GND: behaviour not stated [UNVERIFIED]; surge or no start; T-3 |

### R3.6 Datasheets consulted

| Part | Document | Status |
|---|---|---|
| TPS61089 | TI SLVSD38C, Nov 2015, revised Aug 2021 | **opened this session** (Eq. 2/4, §9.2.2.3, EC table, Figure 7-3 range, REN 800 k, tSS) |
| DRV110 | TI SLVSBA8G | **opened this session** (UVLO 4.6 V, tD ≤ 50 µs, standby 200–250 µA) |
| DPCR2512 (R1528, R1529) | TT Electronics DPCR datasheet (09.26) | read for issue 2 (§2); **not reopened** this session; same part and value as R1507 |
| CDSOD323-T03C (D1506, D1507) | Bourns REV 04/23 | read for issue 2; **not reopened**; same part as D1503 |
| Yageo RC0603 (R1406, R1421) | PYu-RC_Group_51 | formula code; not reopened |
| C1401 680 nF X7R 0603 | — | **no orderable code identified**: MPN left "TBD" with `Verify` rather than invented |
| ADS1292R | TI SBAS502C | not reopened (values from §2/§3) |
| Patient relay | `docs/requirements/patient-relay.md` v0.1 | read |

### R3.7 Problems with the change order (for the design authority)

1. **H-2's current limit is a peak switch current, and 515 kΩ lies outside the datasheet's graph.** SLVSD38C
   Figure 7-3 covers 100–400 kΩ only, and the limit can be 0.8 A below or above the equation (1.2–2.8 A at
   515 kΩ). LR §3.2 modelled it as an average input current. Drawn as ordered, because it passes every check
   here. If TI does not support 515 kΩ, the fallback is 340 kΩ (inside the range): it also passes, but the
   battery peak rises to 4.3 A. Ask TI.
2. **"Transfer 15.9 ms in every battery case" was not simulated with the drawn value.** LR's mode 1 used a 2.5 A
   (412 kΩ) average limit. With 515 kΩ: 18.1–18.4 ms, 33.4 ms total. It still passes, with 6.6 ms margin.
3. **Relay coil inductance can break the 40 ms limit.** A hot coil of ≈ 100 mH or more gives > 40 ms. The relay
   requirement asks the maker only to state the inductance. Proposed for `patient-relay.md` §4: "coil current
   reaches pick-up within ≤ 12 ms from a 12 V supply at 65 °C" (≈ L/R ≤ 6 ms).
4. **FW-12 timing.** "Raise `K_SUPPLY_EN` when charging starts" overlaps the pre-charge with the HV charger. With a
   cold nearly empty battery `VBAT_SYS` falls to 3.7 V: mostly the charger's own 9.2 A (≈ 4.3 V without the coil
   supply), which is a WP-A/WP-E question. Proposed: raise `K_SUPPLY_EN` when the capacitor reaches plan (during
   the 3 s countdown, charger paused), still ≥ 200 ms before `RELAY_CMD`. Raised late (50–100 ms), the relay is
   too slow and the shock is refused (the safe direction, but a refusal).
5. **Contract issue 3 still carries stale WP-H text:** §10 "WP-H with WP-B: … MAX30009 at **128 kHz**" (design
   v0.3 and H-5 say 96 kHz), and §8 sheet 15 "**1 kΩ** pulse-rated drive resistor" (drawn since issue 2 as
   2 × 24 k DPCR + 2.0 k; issue-2 contract issue 2 above). Not edited here.
6. **No MPN for C1401 680 nF** in the change order. Left TBD with `Verify`.
7. **`K_SUPPLY_EN` is single-ended** in this worktree (xcheck: sheet 14 only): expected until sheet 04 (WP-B) is at
   issue 3. Not renamed.

### R3.8 Still needs measurement

- TPS61089 current limit at 515 kΩ and its behaviour charging 6.6 mF through D1401.
- The real relay's coil inductance, pick-up and operate times (no relay exists).
- ADS1292R impedance-channel noise at 64 kHz in this circuit (decides the 10 mΩ margin).
- Battery DC resistance cold and nearly empty; the charger's real load during the pre-charge.
- TVS behaviour for millisecond pulses; DPCR under the real IEC test waveform.
