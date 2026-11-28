**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# WP-G revision 0.3 check: sheet 13 as drawn

27 November 2026, ngspice 42, written with AI assistance by the WP-G capture engineer. These runs check the
revision-0.3 additions to sheet `13_hv_bridge` **as they are drawn in `specs/13_hv_bridge.toml`**, against the
second-round list of `capture-notes/rev3-changes.md` (S-1, S-2, S-3) and the WP-G task. Every number is
**simulated** with hand-built models, not makers' models. Nothing here is safe or ready for use on a person or
animal, and nothing here is an instruction to build or power anything.

Run: `bash run.sh` (`JOBS=6` by default; `ONLY=<regex>` for a subset; `COLLECT_ONLY=1` re-collects without
running). 127 cases plus 20 replays (S-3b), about 1.5–2 hours on a shared 8-core machine. Results:
`out/s1_shorted_lowside.txt`, `out/s2_oc_turnoff.txt`, `out/s3_false_trip.txt`, `out/s3b_replay.txt`,
`out/s4_lsd_monitor.txt`, one `RESULT` line per case (`out/` is not committed, as in `../bridge`).
A case the solver could not finish is retried with other integration settings (the line then says which); a case
still unfinished is marked `INCOMPLETE`.

## What is modelled (and how it differs from `../bridge`)

| Item | Here | Source / confidence |
|---|---|---|
| Power stage | `hbridge3` in `wpg_common.inc`: `../bridge` `hbridge` plus the **local film string** C1347 + C1348 (0.15 µF each, ESR 8 mΩ, ESL 32 nH typ + `lfl` lead/loop inductance 10–60 nH [EST: the -F part has 41 mm wire leads]) with R1370/R1371 10 MΩ; snubber ESL 5 nH in S-3 only [EST]; a **shunt sense-loop inductance** `lsh` 0.3 nH (0–1 nH swept) [EST — Vishay publishes none]; output node capacitance 50 pF each [EST] | CDE 940C catalogue table (read); switch and diode models as `../bridge` (hand-built from the datasheets, low confidence below 10 µs) |
| High-side islands | **circuit level** (`hsisland`): ±18.5 V × BRIDGE_5V/5 V carrier, 0.19 µs dead time, 2 × 10 Ω, bridge rectifier, C1315, R1328, 20 V clamp, D1318 + 100 Ω, 2 × 47 nF, BC807 PNP, **carrier-loss detector as drawn** (BAV70, 4.7 k/47 pF/4.7 k, MMBT3904 + BAT54, 10 k from the gate node, BZX84C15, 2N7002 + 100 Ω); 5 pF interwinding per secondary end | same part models as `../rev3/carrier_loss.cir` [EST generic fits]; transformer output resistance [EST WP-G] |
| Fast idle path | as drawn: R1365 100 Ω, C1345 100 pF, **R1364 10 kΩ pedestal**, D1332, OPA838 macro (300 MHz GBW, ±250 V/µs, output floor 0.15 V [0.1 typ / 0.2 max], bias 1.5 µA both inputs), R1366 1.00 kΩ / R1367 100 Ω, TLV3501 as a threshold at the rising switching point + 10 ns, R1368 6.73 kΩ / R1369 1.00 kΩ / C1346 | OPA838 SBOS867E (Aug 2025) and TLV3501 SBOS321E (Apr 2016), both read; macro built here. The run places the rising threshold at `ith` = 2.54 / 3.0 / 3.5 / 3.87 A (the calculated band, capture notes R3.3) by setting the comparator offset |
| Slow idle path | INA240 (0.4 µs) + 1 k/100 pF, 4.96 A, + 0.1 µs logic — kept, ORed on sheet 12 | as `../bridge` |
| Sheet 12 | WP-D D-7/D-8 as specified (not yet drawn): either idle detector sets the fault latch while both low sides have been off ≥ 9 µs; the high-side kill latch removes the carrier enable after `tlog` = 60 ns (≈ 45 ns of gates per D-7 + ≈ 15 ns sheet-13 /CLR and AND gates) | rev3-changes.md D-7 [CALC there] |
| LSD_OK | divider 100 k / 12.1 k + 100 pF (1.08 µs), threshold at `vtrip` 16.45 / 17.35 V, TLV7031 delay `tlsd` 3 µs (datasheet, 100 mV overdrive) or 10 µs; LSD_OK low ends the sequence through sheet 12 (D-9) | TLV7031 SLVSE13J (Nov 2024), read |
| Capacitor | 100 µF + 0.13 Ω, held at v0 through 10 kΩ (the 20 MΩ balancing string would pull a 1 GΩ feed, as in `../bridge`, down to 38 V) | — |

Junction temperature is given by **both estimates** of `../bridge/notes.md`: *upper* = the model's Zθ extrapolated
as √t below 10 µs; *lower* = adiabatic, SiC 2.21 J/cm³K over an **assumed** 0.2 cm² active area and a depth of
30 µm + √(D·t) (D = 1.7 cm²/s), t = the time the switch dissipates more than 10 % of its peak power. Starting
junction 55 °C.

Solver notes: two model details stalled ngspice in the same-side case and were changed (comparator delay as two RC
stages instead of a transmission line; comparator modelled at its rising switching point instead of a regenerative
Schmitt). Neither changes a threshold or a delay by more than ≈ 1 ns. The fast path's ground is taken at the
shunt's HV_RTN Kelvin terminal (layout requirement L-1); referred to a remote ground, the 25 nH return stray put
tens of volts of bounce on its input during the low-side turn-on, which no real comparator could ride out.

## S-1 — one low-side switch shorted (`s1_shorted_lowside.cir`)

QHA commanded on at the start of a sequence (its slow turn-on), QLB shorted (D, diagonal, design fault 18) or QLA
shorted (S, same side). Everything on sheet 13 as drawn, circuit-level island with the carrier-loss detector.
`ith` is where the fast path's rising switching point sits inside its calculated band (2.54–3.87 A).

**Energy in QHA and junction temperature by both estimates** (simulated; start 55 °C; limit 150 °C):

| Case | ith 2.54 A | ith 3.0 A | ith 3.5 A | ith 3.87 A (top of band) |
|---|---|---|---|---|
| D 1,950 V, 25 Ω | 11.9 mJ, 96 / 60 °C | 14.4 mJ, 103 / 61 °C | 17.1 mJ, 111 / 62 °C | 17.9 mJ, 114 / 63 °C |
| D 1,950 V, 50 / 100 / 175 Ω | 11.4 / 10.5 / 9.3 mJ | 13.6 / 12.4 / 10.7 mJ | 16.1 / 13.7 / 11.1 mJ | 16.3 / 13.7 / 11.1 mJ, ≤ 107 / 62 °C |
| D 2,200 V, 25 Ω | 13.9 mJ, 103 / 61 °C | 17.0 mJ, 112 / 62 °C | 20.2 mJ, 122 / 64 °C | 23.0 mJ, 130 / 65 °C |
| D 2,200 V, 50 / 100 / 175 Ω | 13.5 / 12.5 / 11.1 mJ | 16.1 / 14.7 / 12.8 mJ | 19.1 / 16.9 / 13.9 mJ | 20.7 / 17.4 / 13.9 mJ, ≤ 121 / 64 °C |
| S 1,950 V | 12.8 mJ, 99 / 61 °C | 15.6 mJ, 107 / 62 °C | 18.8 mJ, 117 / 63 °C | 21.3 mJ, 125 / 64 °C |
| **S 2,200 V** | 14.9 mJ, 106 / 61 °C | 18.5 mJ, 118 / 63 °C | 22.5 mJ, 130 / 65 °C | **25.4 mJ, 139 / 66 °C** |

(upper / lower junction estimate.) Detection 5.1–5.7 µs after the carrier starts, at 3.4–7.7 A peak; the event in
QHA lasts 2.9–3.8 µs. Margin to 150 °C: **≥ 11 K by the pessimistic estimate in every nominal case, ≥ 84 K by the
physical one**.

Sensitivities at the worst corner (S, 2,200 V, ith 3.87 A, upper / lower):

| Change | E_QHA | Tj |
|---|---|---|
| none (reference) | 25.4 mJ | 139 / 66 °C |
| shunt sense-loop inductance 0 instead of 0.3 nH | **29.2 mJ** | **150 / 67 °C** — at the rating by the pessimistic estimate |
| shunt sense-loop inductance 1 nH | 15.1 mJ | 107 / 62 °C (the inductive term reports di/dt early) |
| 2N7002 V_th 2.5 V (maximum) / 1.0 V (minimum) | 27.6 / 22.4 mJ | 144 / 131 °C |
| sheet-12 kill path 100 ns instead of 60 ns | 27.5 mJ | 145 °C |
| lower switch saturation current (th 0.06) | 23.9 mJ | 132 °C |
| low carrier (BRIDGE_5V 4.46 V) | 24.9 mJ | 136 °C |

The corners do not all coincide in one part, but they can: with no sense-loop inductance, the comparator at the top
of its band, the slowest 2N7002 and the slowest gates together, the pessimistic estimate passes 150 °C by roughly
5–10 K [extrapolated from the single sensitivities; not run as one case]. The physical estimate stays below 70 °C.
**Verdict: the three changes together hold the shorted-low-side cases inside the rating on the physical estimate
with a large margin, and on the pessimistic one in every simulated case but with zero to negative margin at a
stacked worst corner.** Which estimate is right needs Microchip's Zθ below 10 µs (capture notes R3.9).

Compared with `../bridge` (ideal fast comparator, ideal 100 Ω RECT discharge, ideal immediate gate drop): 10.9–
12.3 mJ there, 9.3–29 mJ here. The real parts cost about 2× at the top of the threshold band, mostly the
comparator offset (the threshold spread 2.54–3.87 A) and the island's finite turn-off (gate < 2 V 4.0–4.5 µs after
the carrier stops instead of ≈ 1 µs with the ideal discharge).

**If part of the new protection has failed silently** (latent; it matters only together with a shorted low side):

| Failed | D 1,950 V 25 Ω | D 2,200 V 25 Ω | S 1,950 V | S 2,200 V |
|---|---|---|---|---|
| fast path dead (slow INA240 path + kill latch + detector left) | 50 mJ, **194** / 75 °C | 65 mJ, **230** / 81 °C | 70 mJ, **252** / 83 °C | 104 mJ, **330** / 96 °C |
| carrier-loss detector inert (fast path at 3.87 A + kill latch) | 32 mJ, 130 / 67 °C | 44 mJ, **156** / 72 °C | 35 mJ, 141 / 69 °C | 46 mJ, **165** / 73 °C |
| both dead (slow path + kill latch only) | 110 mJ, **257** / 92 °C | 189 mJ, **357** / 115 °C | **10.5 J — destroyed** | **12.3 J — destroyed** |

Bold = above 150 °C on that estimate. A dead fast path puts the outcome in the zone where the two estimates
disagree (pessimistic: destroyed; physical: survives); with both dead the same-side case runs away as in
`../bridge` (11–12 J). Neither failure is visible to any self-test today (capture notes R3.5, R3.8).

### The 1 µs vs 2.7 µs disagreement (PNP turn-off start), settled at circuit level

Same circuit, same fault, detector **removed**, once with the circuit-level island and once with `../bridge`'s
behavioural `hsdrv` (`PNPstart_*` cases):

| Case | Island model | Gate at carrier stop | Gate rise after stop | PNP starts after | Gate < 2 V after | E_QHA |
|---|---|---|---|---|---|---|
| D 1,950 V 25 Ω | circuit level | 5.18 V | +0.07 V | **1.07 µs** | 7.8 µs | 24.9 mJ |
| D 1,950 V 25 Ω | behavioural (`../bridge`) | 5.20 V | +0.09 V | **1.06 µs** | 7.7 µs | 27.2 mJ |
| S 2,200 V | circuit level | 5.09 V | +0.07 V | 1.08 µs | 7.4 µs | 30.9 mJ |
| S 2,200 V | behavioural | 5.11 V | +0.10 V | 1.08 µs | 7.6 µs | 35.4 mJ |
| with the detector (all S-1 cases) | circuit level | 5.0–5.4 V | +0.01–0.07 V | 0.83–1.09 µs | **4.0–4.5 µs** | (table above) |

**Answer: about 1 µs.** In this fault the carrier stops at a gate of ≈ 5 V; C1315 (1 nF) dumps into the 97 nF
gate within ≈ 0.1 µs, after which HSA_RECT sits only ≈ 0.7 V above the gate and the 2.2 k bleed takes it below
the PNP's start point in ≈ 0.6 µs more. Both island models agree on that here; the gate rises by < 0.1 V, not
"keeps rising". The ≈ 2.7 µs of `../bridge/notes.md` Q1 is not reproduced by either model in this circuit; it is consistent with
emptying a rectified node that starts near 18 V (τ 2.2 µs), i.e. with the 1 nF still charged well above the gate,
which is not what happens once C1315 has shared its charge with the gate.
The detector's value in this fault is therefore not an earlier PNP start but a **faster gate fall** (4.0–4.5 µs to
2 V instead of 7.4–7.8 µs), because the PNP's base is pulled down through 100 Ω instead of 2.2 kΩ.
Confidence: medium — both island models are hand-built [EST], the transformer's own decay after the drive stops
(leakage and magnetising energy) is not modelled.

Confidence for S-1 as a whole: medium-low. The switch model below 10 µs is the weak point (no Zθ, no linear-mode
rating published); the fast path ran inside the full circuit here, where S-3 found ns-wide numerical spikes on its
nodes — the 66 detection times are consistent with each other (5.1–5.7 µs) and with the inductive term of the
shunt, so the effect is judged small, but it was not separated out as in S-3b.

## S-2 — turn-off at the 80 A trip band with the 75 nF local film (`s2_oc_turnoff.cir`)

Phase 1 into a short (0.01 Ω) at the board connector (no lead inductance) unless stated, capacitor at 2,200 V,
board stray 50 nH, UCC5350 100 ns, film lead/loop inductance `lfl` 10 or 60 nH per capacitor on top of its 32 nH.

| Trip (band) | Cable | lfl | Interrupted I | Peak VDS QLB | Margin to 3.3 kV | Film peak I | Film dv/dt (rating 1,440 V/µs) | Each film V max | Ringing HV_CAP_P (0–2 / 6–12 / 12–25 µs) | f |
|---|---|---|---|---|---|---|---|---|---|---|
| 74 A | 0.5 µH | 10 / 60 nH | 86.0 / 85.6 A | 2,468 / 2,591 V | 832 / 709 V | 75 / 71 A | 501 / 470 | 1,188 / 1,245 V | ±176 / 71 / 34 V | 670–720 kHz |
| 74 A | 1 µH | 10 / 60 nH | 85.2 / 85.5 A | 2,579 / 2,586 V | 721 / 714 V | 96 / 99 A | 641 / 657 | 1,267 / 1,253 V | ±334 / 223 / 131 V | 510–535 kHz |
| 80 A | 0.5 µH | 10 / 60 nH | 91.8 / 91.4 A | 2,465 / 2,600 V | 835 / 700 V | 80 / 74 A | 536 / 491 | 1,194 / 1,246 V | ±187 / 75 / 36 V | 660–710 kHz |
| 80 A | 0.75 µH | 10 / 60 nH | 92.1 / 91.6 A | 2,502 / 2,634 V | 798 / 666 V | 86 / 87 A | 573 / 581 | 1,227 / 1,264 V | ±254 / 138 / 75 V | 580–610 kHz |
| 80 A | 1 µH | 10 / 60 nH | 91.0 / 91.3 A | 2,583 / 2,618 V | 717 / 682 V | 96 / 100 A | 642 / 664 | 1,268 / 1,260 V | ±336 / 224 / 131 V | 515–535 kHz |
| 86 A | 0.5 µH | 10 / 60 nH | 97.6 / 97.2 A | 2,470 / 2,592 V | 830 / 708 V | 88 / 78 A | 584 / 520 | 1,201 / 1,242 V | ±202 / 81 / 39 V | ≈ 710 kHz |
| 86 A | 0.75 µH | 10 / 60 nH | 97.7 / 97.3 A | 2,518 / 2,628 V | 782 / 672 V | 89 / 88 A | 591 / 586 | 1,230 / 1,261 V | ±259 / 140 / 76 V | 576–606 kHz |
| 86 A | 1 µH | 10 / 60 nH | 97.8 / 97.0 A | 2,587 / 2,656 V | 713 / 644 V | 97 / 101 A | 649 / 673 | 1,268 / 1,276 V | ±352 / 225 / 144 V | 515–537 kHz |
| 86 A, 1 µH, pads short (1.5 µH leads) | | 60 nH | 100.5 A | 2,669 V | 631 V | 97 A | 648 | 1,278 V | ±355 / 203 / 139 V | 514 kHz |
| 86 A, 1 µH, board stray 200 nH | | 60 nH | 98.6 A | **2,767 V** | **533 V (16 %)** | 100 A | 669 | 1,275 V | ±350 / 211 / 144 V | 515 kHz |
| 86 A, 1 µH, driver 65 ns | | 60 nH | 95.3 A | 2,639 V | 661 V | 101 A | 672 | 1,268 V | — | 513 kHz |
| 86 A, 1 µH, films −10 % / +10 % | | 60 nH | 97.2 A | 2,646 V | 654 V | 100 A | 667 | 1,273 / 1,271 V | — | 516 kHz |
| 86 A, 1 µH, **C1347 shorted** (C1348 alone, 0.15 µF) | | 60 nH | 96.8 A | 2,568 V | 732 V | 123 A | **821** | C1348 **2,485 V** (rating 3,000 V: 515 V) | ±264 / 185 / 111 V | — |
| 86 A, **film string open** (no local capacitor), cable 0.2 / 0.5 / 1 µH | | — | 97.4 / 97.4 / 97.0 A | 2,726 / 3,131 / **3,591 V** | 574 / 169 / **−291 V** | — | — | — | — | — |

Turn-off energy 14.4–16.6 mJ; QLB junction (pessimistic, 0.2 µs event) 146–174 °C, physical estimate ≈ +14 K
(as `../bridge`; unchanged question: Microchip publishes no turn-off SOA for 86–100 A at 2.5 kV).

**Answers.**
1. **75 nF is enough:** with 0.5–1 µH of cable the switch sees **2.47–2.77 kV, at least 533 V (16 %) under its
   3.3 kV rating** in every simulated case (100 nF gave 2.50 kV in `../bridge`; the 75 nF result is within
   ≈ 100 V of it at 0.5 µH). Interrupted current 85–100 A, below I_DM 107 A.
2. **The film parts are inside their catalogue ratings:** ≤ 101 A against 216 A peak, ≤ 673 V/µs against
   1,440 V/µs (821 V/µs with one part shorted), ≤ 1,278 V each in normal sharing and 2,485 V on the survivor if
   one shorts (3,000 V DC rating; 500 V AC rating not exceeded by the ±350 V ring).
3. **Ringing between the film and the cable is lightly damped:** 510–720 kHz (1/(2π√(L·C)) with the cable and
   75 nF plus the leads), ±180–355 V at first and still ±35–145 V 12–25 µs later (Q ≈ 20–30; the only damping is
   the main capacitor's 0.13 Ω and the film's 2 × 8 mΩ). It happens at every hard edge, not only at a trip (each
   normal phase end interrupts 12–66 A). It does not threaten a rating in these cases, but it is a conducted-
   emission and cable-stress source, and it sits on the bus the patient current comes from. A series damper (a few
   ohms in series with a second, larger film branch) would kill it; not added — it needs a part with a pulse
   rating and a decision on board area (capture notes R3.8).
4. **An open film string is latent and unsafe with a long cable:** 3.13 kV at 0.5 µH (169 V margin) and
   **3.59 kV at 1 µH — over the 3.3 kV rating**. Nothing detects an open film capacitor or a cracked lead.
   Either the capacitor cable must be kept short (≤ 0.2 µH gives 2.73 kV even with the film open), or the film
   branch made redundant (capture notes R3.8).

## S-3 — no false trip in a normal sequence or from the charger (`s3_false_trip.cir`, `s3b_replay.cir`)

A healthy bridge runs a whole sequence (lead, phase 1, gap with QHA off and QHB on, phase 2, end; phases shortened
to 100 µs so the phase-end currents are higher than in a real 3–9 ms phase) with the patient connected. Sheet 12
arms the idle fault only while both low sides have been off for ≥ 9 µs, so the fast path must stay quiet in three
windows: **W1** the lead (QHA turns on slowly and both outputs charge from 0 V to the capacitor voltage, charging
the low sides' output capacitance *through the shunt*), **W2** the gap (QHA off, QHB on) and **W3** after phase 2.
The fast path is set at the **bottom of its band (2.54 A)**, its most sensitive.

**Method note (why two netlists).** In the full bridge circuit, the solver settings that get through the switching
events also put isolated 1–4 ns spikes of 5–25 mV on the fast path's internal nodes; at those instants the
amplifier's input moved while the shunt voltage feeding it did not, which no circuit cause can do, and a run with
the charger pulses even showed the amplifier "saturating" with 0.13 A in the shunt. These are numerical. Because the
fast path loads the shunt through 100 Ω only, each run records the shunt's Kelvin voltage and `s3b_replay.cir`
replays it into the fast path alone with tight tolerances (trap, reltol 1e-4). The replay is the result below; the
in-circuit comparator model, which filters the spikes like a real comparator would, did not trip either (its output
never passed 7 × 10⁻⁶ of a trip).

| Case | Shunt current max W1 / W2 / W3 | Fast path trips? | Margin above the 2.54 A threshold W1 / W2 / W3 |
|---|---|---|---|
| 1,950 V, 25–175 Ω, shunt sense-loop 0.3 nH | 0.54–0.55 / 0.16 / 0.01–0.03 A | no | **1.98 / 2.37 / 2.50 A** |
| 1,950 V, 25–175 Ω, 1 nH | same | no | **1.71 / 2.30 / 2.43 A** |
| 2,200 V, 25–175 Ω, 0.3 nH | 0.57 / 0.18 / 0.01–0.03 A | no | 1.96 / 2.36 / 2.50 A |
| 2,200 V, 25–175 Ω, 1 nH | same | no | **1.67** / 2.28 / 2.44 A |
| charger running (0.43 A pulses + 0.3 A recovery spikes into HV_CAP_P), 1,950 V 25 Ω / 2,200 V 175 Ω | 0.55 / 0.16 / 0.13 A | no | 1.90 / **1.83** / 1.91 A |
| OPA838 at its worst low swing (0.2 V) and +230 µV offset | as 1,950 V 25 Ω | no | 1.98 / 2.38 / 2.50 A |
| test mode, 60 V into 200 Ω | 0.13 A | no | 2.38 / 2.53 / 2.53 A |

**Answer: no false trip.** The worst point is the lead (W1), where the outputs' charging current (≤ 0.57 A in the
shunt) plus the inductive term of the shunt's sense loop make the fast path see the equivalent of ≈ 0.87 A, leaving
**≥ 1.67 A (≥ 66 % of the most sensitive threshold)**. The charger's current returns to HV_RTN without passing the
shunt; what reaches the shunt through the switches' capacitances is ≤ 0.13 A. The pedestal works: at the OPA838's
worst-case low output swing nothing changes. The slow INA240 path stays below 0.44 A in all windows.
Confidence: medium. Not modelled: the real WSK2512 sense-loop inductance (only swept 0.3–1 nH), board-level
coupling of the high-side transformers' interwinding capacitance into the Kelvin traces (5 pF per end is in the
model but layout coupling is not), the decompensated OPA838's real behaviour on a 1.1 kΩ feedback with a
comparator input, and real ground bounce between GND and SHUNT_KN (layout requirement L-1 assumed met).

## S-4 — the +20 V low-side supply monitor `LSD_OK` (`s4_lsd_monitor.cir`)

Event 1.0 ms into phase 1 (QHA + QLB conducting, 25 Ω: ≈ 47 A), MGJ2 +20 V side only; BRIDGE_5V stays up.
"Partly on" = QLB above 20 V while carrying > 2 A (normal conduction ≤ 13 V hot). Junction: pessimistic estimate
(the event's turn-off is a ≈ 0.2 µs pulse; physical estimate ≈ +14 K above the conduction temperature).

| Event (25 Ω unless stated) | Monitor | LSD_OK trips after | Rail / QLB VGS then | QLB blocks after | Partly on / linear | E_QLB from event | Tj QLB |
|---|---|---|---|---|---|---|---|
| none (reference: the same phase 1 from 1.3 to 3.3 ms) | fitted | — | — | — | 0 / 0 µs | 362 mJ (conduction) | 97 °C |
| MGJ2 output collapses in 100 µs | **absent** (rev 0.2) | — | — | 57 µs (driver UVLO) | 3.3 / 1.3 µs | 36 mJ | **175 °C** |
| same | fitted, trip 16.45 V | 24.9 µs | 15.6 V / 16.4 V | 25.7 µs | 0.13 / 0.13 µs | 12 mJ | 136 °C |
| MGJ2 output decays over 1 ms | **absent** | — | — | 568 µs | **24.6 / 2.1 µs** | 235 mJ | **182 °C** |
| same | fitted, trip 16.45 V (band bottom) | 210.6 µs | 16.3 V / 15.9 V | 211.5 µs | 0.12 / 0.12 µs | 71 mJ | 130 °C |
| same | fitted, trip 17.35 V (band top) | 167.2 µs | 17.2 V / 16.9 V | 168.0 µs | 0.12 / 0.12 µs | 57 mJ | 131 °C |
| same | fitted, comparator 10 µs (slow overdrive) | 217.6 µs | 16.2 V / 15.8 V | 218.3 µs | 0.09 / 0.09 µs | 73 mJ | 129 °C |
| MGJ2 sags to 75 % (15.5 V) and stays | absent | — | — | never (QLB stays ohmic) | 0 / 0 | 397 mJ | 98 °C |
| same | fitted | 86.9 µs | 16.2 V | 87.7 µs | 0.13 / 0.13 µs | 32 mJ | 132 °C — **shock ended** |
| MGJ2 sags to 60 % (12.4 V) and stays | absent | — | — | never | 0 / 0 | 445 mJ | 100 °C |
| same | fitted | 55.9 µs | 16.1 V | 56.6 µs | 0.09 / 0.09 µs | 22 mJ | 133 °C — **shock ended** |
| 175 Ω, collapse 100 µs / 1 ms | absent / fitted | — / 24.5, 207 µs | | 58 / 569 µs; 27 / 209 µs | ≤ 0.16 µs | ≤ 9 mJ | ≤ 63 °C |

**Answers.**
1. **Thresholds:** trip 16.93 V nominal; worst-case band **16.48–17.29 V** [CALC: BRIDGE_3V3 ±1.5 %, four 0.1 %
   resistors, TLV7031 offset ±8 mV and hysteresis 2–17 mV, all × 9.26]; the runs used 16.45 and 17.35 V. Normal
   rail 19.5–22 V (WP-G §6.6); lowest working rail at BRIDGE_5V 4.5 V ≈ 18.1 V [EST from the MGJ2's 1.2 %/% line
   regulation, undated datasheet copy] → **≈ 0.8 V margin** against a nuisance trip. Not simulated: MGJ2 ripple.
2. **The partly-on case of `../bridge` Q4 is closed:** with the monitor the low side is opened 0.7–2.1 µs after
   `LSD_OK` falls, while its gate is still at 15.8–16.9 V (fully on). The linear-region time falls from 2.1 µs plus
   ≈ 25 µs partly on (182 °C pessimistic) to ≈ 0.1 µs (129–136 °C, the ordinary signature of opening the low side at
   ≈ 50 A; `../bridge` Q4's BRIDGE_5V collapse gave 132 °C). Rail crossing → low side open: ≈ 5–6 µs on a slow ramp
   (1.1 µs filter lag + 3 µs comparator + gates), as the change order asked (≤ 5 µs from the comparator).
3. **A side effect to know:** a converter that only sags (to 15.5 V or 12.4 V and stays) was harmless without the
   monitor in this model (QLB stays ohmic, 98–100 °C), but with the monitor it **ends the shock**. That is the
   conservative choice the change order made (a sagging gate supply is a fault); it trades a modest thermal risk for
   a lost shock in that rare case. Recorded for the design authority (capture notes R3.8).

## Summary for the capture notes

| Question | Result [SIM] | Margin | Confidence |
|---|---|---|---|
| S-1 shorted low side, all cases, protection working | QHA 9.3–25.4 mJ; junction 84–139 °C pessimistic, 59–66 °C physical | ≥ 11 K / ≥ 84 K to 150 °C; stacked worst corner ≈ 0 K pessimistic | medium-low (switch model below 10 µs) |
| S-1 fast path failed silently | 50–104 mJ, 194–330 °C / 75–96 °C | fails on the pessimistic estimate | medium-low |
| S-1 both new detectors failed, same side | 10.5–12.3 J | destroyed | medium |
| PNP turn-off start, gate part-way | 1.06–1.08 µs both island models (detector: gate < 2 V in 4.0–4.5 µs vs 7.4–7.8 µs) | — | medium |
| S-2 turn-off at 74–86 A, 75 nF, 0.5–1 µH | 85–100 A interrupted, VDS 2.47–2.77 kV | ≥ 533 V (16 %) to 3.3 kV | medium (strays estimated) |
| S-2 film string open | 3.13 kV (0.5 µH), 3.59 kV (1 µH) | **−291 V at 1 µH** | medium |
| S-2 film ratings | ≤ 101 A, ≤ 673 V/µs (821 with one shorted), ≤ 2,485 V on a survivor | 2.1× / 1.75× / 515 V | medium (catalogue typical ESL/ESR) |
| S-3 false trips | none; fast path sees ≤ 0.87 A equivalent | ≥ 1.67 A above the 2.54 A band bottom | medium |
| S-4 LSD_OK | trips at 16.1–17.2 V on the rail; low side opened at VGS ≥ 15.8 V; linear time 2.1 µs → 0.1 µs; 182 → 130 °C | 0.8 V nuisance margin [EST] | medium (MGJ2 behaviour estimated) |

## What simulation cannot settle

As `../bridge/notes.md`, plus: the WSK2512 sense-loop inductance and the layout of the Kelvin traces (they move the
S-1 energy by −40 %/+15 % and the S-3 margin by ≈ 0.3 A); the real OPA838 and TLV3501 dynamics near their
thresholds (macro and threshold models here); the 940C film's lead inductance in the final mounting; the
transformers' behaviour when the drive stops; the MGJ2's output at low input and its ripple.
