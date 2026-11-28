# Capture notes — WP-C ui+sense (sheets 06_ui, 07_sense_main, 08_sense_sup)

> **NOT FOR CONSTRUCTION — unreviewed design.** These notes and the sheets they describe were drawn with AI
> assistance and have not been reviewed by a qualified high-voltage or medical-device engineer. Nothing here tells
> anyone to build, power or probe anything (CLAUDE.md rule 2, SAFETY.md §2a). Nothing here is safe or ready for use
> on a person or animal. All arithmetic below is on paper **[CALC]**; nothing was simulated or measured.

Captured 27 November 2026 against contract issue 2 and `docs/HARDWARE_DESIGN.md` v0.2. Source of truth: the pin
tables `hardware/aed-main/specs/06_ui.toml`, `07_sense_main.toml`, `08_sense_sup.toml` and the symbol source
`hardware/lib/src/aed_uisense.toml`. Every number below gives its source; "[UNVERIFIED]" means looked for and not
confirmed.

## Revision 0.3 (contract issue 3, change order `rev3-changes.md` WP-C C-1…C-7)

Applied 27 November 2026 against `docs/HARDWARE_DESIGN.md` v0.3 and `contract.md` issue 3. Sections 1–9 below
describe the issue-2 capture; where they disagree with this section (notably §1 and §3.4 on the over-voltage
comparator, and contract issue C-3), this section wins. Checks: `hardware/aed-main/sim/rev3-wpc/` —
`ovp_band.py` (calculation) and `ovp_latch.cir` (ngspice, hand-built models; **simulated**, nothing measured).

### R3.1 In plain language

- **The main measurement now notices a broken wire.** A very large resistor (1,000 MΩ) gently pulls the main
  controller's capacitor-voltage input up toward its 2.5 V reference. If the wire from the high-voltage board
  breaks, the reading climbs within about a quarter of a second and the "capacitor is below 60 V" signal drops,
  so the low-voltage self-test is refused. With the wire intact it adds a fixed error of well under one volt,
  which the factory calibration removes.
- **The hardware over-voltage trip now latches and sits lower.** It trips at about 2,107 V (worst case
  2,071–2,141 V over −20 to +70 °C), which is above any normal charge (top 2,008.6 V) and below the
  capacitor's 2,200 V rating. Once tripped it stays tripped, holding both discharge paths open until the
  capacitor is empty, until the supervisor sends a reset *pulse* (a reset wire stuck "on" cannot keep it
  disarmed) or its supply is lost.
- **The supervisor can test it every day.** A test wire pushes a fake "over-voltage" into the measurement while
  the capacitor is empty; the trip must fire, stay latched after the test wire is released, and clear on a reset.
- **One change from the change order.** The change order fed that fake signal in *after* the resistor that
  carries the real signal to the trip circuit. If that resistor broke open, the real trip would be dead but the
  daily test would still pass. I feed the test in one step earlier (at the input of the measurement amplifier),
  so the test now goes through the same path as a real over-voltage. Simulated both ways; see R3.5.
- **One weakness I could not close inside these sheets:** a partly-failed reset transistor (or dirt on the board)
  that leaks a little current away from the trip input raises the trip point without making the daily test fail.
  The firmware over-voltage limit still covers it, but it is a hidden fault. The design authority should decide
  how to test for it (R3.6, problem P-2).

### R3.2 Changes applied (designators)

| # | Sheet | Done | Notes |
|---|---|---|---|
| C-1 | 07 | **R718** 1 GΩ 1 %, 1206, Vishay CRHV1206AF1G00FKE5, `VREF_M` → `VCAPM_IN` | as ordered |
| C-2 | 08 | **R805** 36.5 k → **100 k** 0.1 % (TNPW0603100KBEEA); **R806** 100 k unchanged; **R807** 9.09 k → **21.5 k** 0.1 % (TNPW060321K5BEEA); **R808** 15.6 k unchanged | as ordered |
| C-3 | 08 | **C812** 100 nF X7R (`OVP_RST`–`OVP_RG`), **R809** 100 k (`OVP_RG`–GND), **Q801** BSS138 (G `OVP_RG`, S GND, D `OVP_RSD`), **R810** 1 k (`OVP_RSD`–`OVP_SENSE`), **R811** 100 k (`OVP_RST`–GND) | as ordered |
| C-4 | 08 | **R813** 100 k (`OVP_TEST`–GND); **R812 4.7 k** (`OVP_TEST`–`OVP_TI`); **D801** BAS116LT1G (A `OVP_TI`, K **`VCAPS_IN`**) | **departure**: injection node `VCAPS_IN` instead of `OVP_SENSE`, R812 4.7 k instead of 22 k (R3.5) |
| C-5 | 08 | sheet comment, group notes and part descriptions rewritten: release only by an `OVP_RST` edge or `3V3_SUP` loss; the 2,160 V / "releases near 1,000 V" text removed | |
| C-6 | 06 | nothing to change: sheet 06 has used only `3V3_AON`, `5V0_SYS` and `GND` since its issue-2 capture; contract §4 already lists 06 off `3V3_MAIN` | |
| C-7 | 07 | no change (HVCR2512 string stays; owner question 4) | |

New global nets on 08: `OVP_RST`, `OVP_TEST` (contract §5.4 names, active high, default resistors R811, R813).
New local nets: `OVP_RG`, `OVP_RSD`, `OVP_TI`. Part counts: 06 44, 07 31 (+1), 08 31 (+8). D801 uses WP-D's
symbol `aed_logic:BAS116LT1G` (library rule: defined once; not duplicated in `aed_uisense`). Symbol library
`aed_uisense.toml` unchanged (Q801 is the stock `Transistor_FET:BSS138`, pins 1 G / 2 S / 3 D as onsemi CASE 318).

### R3.3 Arithmetic

**R718 (07).** Open tap: C701 10 nF charges from VREF_M through 1 GΩ, τ = 10 s; `LOWV_OK` falls at 54.7 mV
(§3.3): t = 10 s × ln(2.5 / (2.5 − 0.0547)) = **0.22 s** [CALC] (LR §5.2 simulated 0.16–0.37 s over the
comparator band). Intact wire: 2.5 nA × (R701 100 k + 50.3 k tap source on 11) = **+0.38 mV = +0.38 V** of
reading at 0 V, +0.08 V at 1,950 V [CALC]; LR §5.2's simulation gives +0.22 / −0.07 V with its own bias-current
assumption — both are a fixed offset/gain term removed by calibration. Voltage across R718 ≤ 2.5 V (rating
1,500 V); TCR 100 ppm/K × 45 K changes the offset by < 0.002 V. Failures: R718 open = issue-2 behaviour (open tap
not detected; latent); R718 short = reading 2,500 V, refused (detected).

**Over-voltage threshold (08).** OVP_REF = 2.5 V × 15.6 k / (21.5 k + 15.6 k) = **1.05121 V**. Output low
before the trip, Rin = Rf: trip at VCAPS_BUF = 2 × 1.05121 = 2.10243 V → × 1,002.0 = **2,106.6 V** nominal.
Latched: with `OVP_TRIP` = VOH, OVP_SENSE = (VCAPS_BUF + VOH)/2 ≥ VOH/2 = 1.6 V > 1.05 V for any tap ≥ 0 V; VOH
need only exceed 2 × 1.0512 + 2 mV = 2.105 V.

**Trip band, worst case, −20…+70 °C** (`ovp_band.py`; every term at its limit at once; 200,000-sample uniform Monte
Carlo alongside):

| Term | Assumption and source | Effect on trip |
|---|---|---:|
| ADR3425 | 0.1 % initial + 8 ppm/K × 45 K + life 500 ppm **assumed** (30 ppm/1000 h at 50 °C stated) + 70 ppm hysteresis (Rev. B, issue-2 notes) | ±4.1 V |
| R807, R808 | TNPW 0.1 % + 25 ppm/K × 45 K + 0.05 % life **assumed**, each independent | ±3.2 V each |
| R805, R806 | same | ±2.8 V each |
| TLV9031 Vos | ±2 mV, −40…125 °C (SNOSDA3H §5.7) | ±4.0 V |
| TLV9031 VOL feeding R806 | ≤ 1 mV at ≤ 20 µA [EST from 175 mV at 4 mA] | −1.0 V |
| Leakage on OVP_SENSE | Q801 off-state ≤ 100 nA [EST; datasheet: 100 nA at 30 V, 25 °C] + TLV9031 IB 5 pA typ | 0 … +10.0 V |
| Follower | OPA388 Vos + IB × source (issue-2 §4) | ±0.1 V |
| Supervisor string ratio | 0.2 % + 0.225 % temperature + 0.25 % life (CS Q4) | ±14.2 V |
| **Band** | without / with the 100 nA leakage estimate | **2,071.5 – 2,141.1 V / 2,071.5 – 2,151.3 V** |
| Monte Carlo (with leakage) | 0.1 % … 99.9 % | 2,085 – 2,137 V |

Checks against the design (§B4.4): bottom **62.9 V above** the top of a normal adult charge (2,008.6 V) — no
hardware layer stops a normal charge; top **58.9 V below** the 2,200 V rating (48.7 V with the leakage estimate)
— confirmed. The change order's 2,073–2,142 V agrees within 2 V. Note the band **overlaps** the firmware limit L1
(2,043–2,098 V): on some units the latch, not the firmware, acts first (harmless: both dump; a latched trip then
needs an `OVP_RST`). Sheet-08 parts alone: trip at VCAPS_BUF 2.0815–2.1326 V.

**Reset.** Gate pulse 3.3 V decaying with 100 k × 100 nF = 10 ms; Q801 on for 10 ms × ln(3.3 / VGS(th)) =
7.9–14 ms (VGS(th) 0.8–1.5 V at 1 mA, BSS138/D Rev. 7); OVP_SENSE during reset = 3.22 V × 1 k / 101 k = 32 mV
< 1.05 V. A stuck-high line: gate below 0.8 V after 14 ms, so it cannot hold the latch clear. Falling edge:
gate ≈ −3.3 V (VGS ±20 V).

**Test (as drawn).** OVP_TEST high: VCAPS_IN = (V_OH − VF) × 149.8 k / 154.5 k. With 3V3_SUP 3.2–3.4 V
[UNVERIFIED, WP-A] and VF 0.45–0.70 V at 15–20 µA [EST]: **2.42–2.86 V**; even at the datasheet's VF ceiling
(0.90 V, which is specified at 1 mA) and 3.2 V: 2.23 V — above the sheet-08 band top 2.133 V. Rise time
(4.7 k ∥ 149.8 k) × 10 nF = 46 µs (T-6 wants ≤ 1 ms). Test low: D801 reverse-biased by ≤ 2.2 V; leakage
≤ 5 nA (25 °C, 75 V) × 150 k = 0.75 mV = **0.75 V** of supervisor reading; at the 150 °C ceiling (80 nA) 12 V,
which would still leave the supervisor chain inside D-28 at 1,950 V (13.0 + 12 = 25 V of 26.5 V) [CALC].

### R3.4 Datasheets consulted

| Part | Document | Opened? |
|---|---|---|
| BAS116LT1G (D801) | onsemi BAS116LT1/D **Rev. 14, Oct 2026**: pin 1 anode, 3 cathode, 2 NC (CASE 318 style 8); VR 75 V; IR ≤ 5 nA (75 V, 25 °C), ≤ 80 nA (150 °C); VF ≤ 0.90 V at 1 mA | yes, onsemi.com |
| BSS138 (Q801) | onsemi BSS138/D **Rev. 7, Apr 2024**: pins 1 G, 2 S, 3 D; VGS(th) 0.8–1.5 V (1 mA), −2 mV/K; VGSS ±20 V; IDSS ≤ 0.5 µA (50 V), ≤ 100 nA (30 V), ≤ 5 µA (50 V, 125 °C) | yes, onsemi.com |
| CRHV1206 (R718) | Vishay CRHV, document 68002, **revision 12-Nov-2024** (the change order cites 18-Apr-17): 1206 1,500 V; 2 M–1 G at 1–20 %; TCR 100 ppm/K "reference only"; global ordering-code key | yes, vishay.com |
| TLV9031 (U803) | TI SNOSDA3H, **Nov 2025**: Vos ±2 mV (−40…125 °C), IB 5 pA typ, **no internal hysteresis**, VOL/VOH ≤ 175 mV at 4 mA; **rev. H deleted the power-on-reset feature** | yes, ti.com |
| ADR3425, OPA388, TNPW, CRCW, KEMET X7R | as issue-2 notes (§2) | not re-opened |
| 3V3_SUP tolerance, MCXA153 VOH | — | **unverified** |

### R3.5 Part-failure table (latch, reset, test; one failure at a time)

"Detected by T-6" means the self-test step of design §B14.5 fails. Simulated cases (`ovp_latch.cir`) are marked
[SIM]; the rest [CALC]. "Change order" = injection at OVP_SENSE through 22 k.

| Part, failure | What it does (as drawn) | Hides a trip / holds the limit off? | Change order arrangement |
|---|---|---|---|
| R805 open | real tap cannot reach the comparator | limit lost, **detected by T-6** (injection now passes through R805) [SIM] | **hidden**: test trips and latches, real trip never [SIM] |
| R805 short | node follows the buffer: trips at ≈ 1,054 V, cannot latch | no; every charge dumped, T-6 "held" step fails | same |
| R806 open | no latch; trips at ≈ 1,054 V | no (fail-safe, detected) | same |
| R806 short | output low holds OVP_SENSE low | limit lost, detected by T-6 | same |
| R807 open / R808 short / C810 short | OVP_REF ≈ 0: trips at any charge | no (fail-safe) | same |
| R807 short / R808 open | OVP_REF 2.5 V: trip at tap 5 V, never | limit lost, detected by T-6 (test reaches ≤ 2.86 V) | same (test reaches 1.88 V) |
| U803 output stuck low / high | no trip / permanent trip | low: detected by T-6; high: fail-safe | same |
| U804 or R801/C801 fault | reading wrong | detected by main-vs-supervisor comparison (issue 2) | same |
| Q801 drain–source short | OVP_SENSE held at ≈ 0 | limit lost, **detected by T-6** [SIM] | same |
| **Q801 partial leak (e.g. 1 MΩ D–S), or board contamination on OVP_SENSE** | trip raised by 1.05 V × 100 k / R_leak: 1 MΩ → **2,211 V** [SIM] | **hidden: T-6 passes** [SIM]; anything above ≈ 100 kΩ passes the "held" step | **same, hidden** [SIM] |
| Q801 gate–source short, R809 short, R810 open, C812 open | reset impossible | no; latch stays set → no charge, detected by T-6 reset step [SIM C812 open] | same |
| Q801 gate–drain short | diode-connected FET + 101 k on OVP_SENSE: trip ≥ 3.1 V tap | limit lost; detected by T-6 (test reaches ≤ 2.86 V) [CALC] | **may pass T-6** (strong injection, hold step marginal) |
| C812 short or R809 open | reset becomes level-sensitive | latent (needs a stuck-high `OVP_RST` too); **detected only if T-6 holds `OVP_RST` high ≥ 50 ms before raising `OVP_TEST`** [SIM C812 short] | same |
| R811 open | `OVP_RST` floats while the supervisor pin is high-impedance | latent; a stray edge only resets (re-trips if still over-voltage) | same |
| D801 short | VCAPS_IN loaded by 4.7 k: reading ≈ 3 % of true, no real trip | limit lost, **detected at once** by main-vs-supervisor comparison [SIM] | detected by T-6 "held" step when `OVP_TEST` is driven low [SIM] |
| D801 open, R812 open, R813 short | no injection | no; T-6 fails (false alarm, safe) | same |
| R812 short | stronger injection | latent, harmless | same |
| R813 open | `OVP_TEST` floats when the supervisor pin is high-impedance | spurious trip at worst (fail-safe) | same |
| D801 reverse leakage grown | supervisor reading error | detected by comparison if large | trip raised (leak on OVP_SENSE), hidden |

Result: as drawn, **no single open or short** in the latch/reset/test arrangement hides a trip or holds the
limit off undetected. A **resistive partial failure** to GND on OVP_SENSE (P-2) is not caught by T-6 in either
arrangement.

### R3.6 Problems with the change order (for the design authority)

- **P-1 (C-4 test node).** Injecting at OVP_SENSE leaves R805 open hidden (and, less likely, Q801 gate–drain
  short) [SIM]. Implemented the closest correct circuit inside sheet 08: inject at `VCAPS_IN` through 4.7 k + D801.
  Net names and levels of `OVP_TEST` unchanged (high = inject a trip). Documents to update: contract §5.4
  `OVP_TEST` row ("through a BAS116 and 22 kΩ" into `OVP_SENSE`), design §B14.5 T-6. **Firmware consequences:**
  during T-6 the supervisor's own reading goes to full scale, so T-6 must suspend the main-vs-supervisor
  comparison and the supervisor's firmware over-voltage response (holds are forced anyway); the reading also
  proves the injection reached the chain.
- **P-2 (hidden partial leak, both arrangements).** A leak R from OVP_SENSE to GND raises the trip by
  1.05 V × 100 k / R and passes T-6 for R ≳ 100 kΩ (up to ≈ +1,000 V); 1 MΩ → 2,211 V [SIM]. Covered only by
  L1 firmware (≤ 2,098 V) and L3 (≤ 2,316 V). Options: a quantitative test that ramps the injection while the
  supervisor ADC reads it (needs a controlled ramp, e.g. a slow RC on the injection) and checks the trip level; a
  second, independent comparator; or accept as a latent two-layer fault. Within-spec Q801 leakage is also the
  largest unverified term in the band (+10 V per 100 nA).
- **P-3 (T-6 order).** Propose: raise `OVP_RST` and hold it ≥ 50 ms, then raise `OVP_TEST` with `OVP_RST` still
  high (must trip), lower `OVP_TEST` (must stay latched), lower then pulse `OVP_RST` (must clear). This proves
  the reset is edge-only (catches C812 short / R809 open) [SIM].
- **P-4 (power-up).** TLV9031 SNOSDA3H rev. H deleted the power-on reset; the latch may power up set (safe
  direction). Firmware should pulse `OVP_RST` at switch-on once both readings are < 50 V.
- **P-5 (diode drop).** The change order's 0.6 V for the BAS116 is not a datasheet figure (only ≤ 0.90 V at
  1 mA is); the margins above use both.
- **P-6 (string ratio).** Sheet 11 now draws the supervisor string as 20 × 1.99 MΩ + 2 × 79.6 kΩ = 1:1,001.0
  (WP-F departure note) while contract §5.6 and this sheet use 1:1,002.0; at 1:1,001.0 the nominal trip is
  2,104.5 V (−2 V). Not changed here; integrator to settle the constant.
- **P-7 (noise).** OVP_SENSE is a 50 kΩ node with no filter capacitor; a fast disturbance can now latch (a
  nuisance trip empties the capacitor and needs a reset — no shock until then). Not changed; worth a layout and
  immunity review, and possibly a small capacitor (it would slow T-6 and the reset only microseconds).

### R3.7 Still needs simulation or measurement

- TLV9031 with the maker's model: output during 3V3_SUP ramp; behaviour on a slow ramp at the trip point.
- Q801 off-state leakage at VDS 1–3 V, −20…+70 °C (sets +0…10 V or more of the band).
- BAS116 VF at 15–20 µA and IR at 2 V over temperature; MCXA153 output high level; 3V3_SUP tolerance.
- The P-2 partial-leak case with a real test procedure; ESD/noise immunity of OVP_SENSE (P-7).
- R718 offset with real bias currents (LR +0.22 V vs hand +0.38 V).
- Build check: `bash tools/build.sh` BUILD OK (724 nets, 3,688 pin connections round-trip); `tools/xcheck.py`
  lists `OVP_RST` and `OVP_TEST` as single-sheet nets — expected until WP-B draws them on sheet 05.

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
