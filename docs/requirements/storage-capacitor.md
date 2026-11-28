# Requirements sheet: storage capacitor C1101 (made to order)

Version 0.1, 27 November 2026. Decisions D-41, D-42, D-53 (confirmed) and **D-65 (provisional, awaiting owner
confirmation)**. Written with AI assistance.

> **NOT FOR CONSTRUCTION — unreviewed design.** A list of what a part to be bought must do and what proof to ask its
> maker for. It says nothing about how to make it, and nothing here is an instruction to build or power anything.
> This part stores about 190 joules at about 2,000 volts, far above what stops a heart (SAFETY.md §2).

## In plain language

The full requirements for this capacitor are in `docs/PARTS_STUDY.md` **Appendix A** (version 0.4). This sheet
repeats the headline figures and records **two additions** that design revision 0.3 now relies on:

1. **A bleed resistor inside the capacitor's case (10 MΩ).** If the wires between the capacitor and the board ever
   came apart, the capacitor would otherwise keep its charge indefinitely, because every discharge path is on the
   board. With the internal resistor it falls below 60 V in about an hour on its own (I-5). This is a "the charge
   does not stay forever" measure, **not** a "safe to touch" measure: an hour is far too long, and a capacitor can
   recover some charge after being emptied.
2. **A surge rating: at least 2,420 V for at least 1 second, at least 100 times over its life.** The device has
   three independent ways to stop charging before 2,200 V. One of them, inside the charger chip, cannot be made
   accurate enough to sit between a normal charge (up to 2,008.6 V) and 2,200 V, so it has been moved above the
   normal range as a last resort (I-2). If the other two had both failed, the capacitor could briefly see up to
   about 2,409 V before that last stop acts.

Where this sheet and Appendix A disagree, this sheet is newer.

## 1. Headline requirements (unchanged; detail in PARTS_STUDY Appendix A)

| Item | Requirement | Source |
|---|---|---|
| Type | Metallised polypropylene film, self-healing, defibrillator pulse duty | Appendix A1 |
| Capacitance | 100 µF ± 5 % | D-23, D-41 |
| Rated DC voltage | ≥ 2,200 V continuous, −20 to +60 °C | D-41 |
| Test voltage | 2,420 V for 10 s on every part | Appendix A1 |
| Normal pulse | 65.7 A peak (1,950 V, 25 Ω), ≤ 15.5 ms per shock (simulated, `sim/bridge/notes.md` Q3) | Appendix A1 (65 A) |
| Life | ≥ 2,500 full-energy cycles at 50 °C ambient, 60 °C at the part | D-41, D-53 |
| Storage | −40 to +70 °C | D-41, D-48 |

## 2. Changed or added in revision 0.3

| Item | Requirement | Basis |
|---|---|---|
| **Internal bleed resistor** (replaces "none, unless agreed") | **10 MΩ ± 5 %**, built from **at least four series elements, each rated ≥ 1 kV** so that one shorted element leaves ≥ 7.5 MΩ and the chain still withstands **≥ 2,420 V**; stable over the working temperature range and life (drift ≤ 5 %); dissipation 0.40 W at 2,008.6 V, 0.48 W at 2,200 V [CALC: V² / 10 MΩ], to be accounted for in the maker's thermal rating at 60 °C | `sim/charger-storage/notes.md` Q5 (D-65): 2,200 → 60 V in 63 min with the capacitor alone (simulated) |
| Effect of the bleed on the capacitor's self-discharge figure | Insulation-resistance requirements of Appendix A1 apply to the capacitor element **without** the bleed; the maker states both figures | Appendix A1 "leakage" |
| **Surge rating** | **≥ 2,420 V for ≥ 1 s, ≥ 100 events over life**, at up to 60 °C, without loss of capacitance beyond end-of-life limits or self-healing damage; and the maker states the highest voltage the part survives for 1 s once | `sim/charger-storage/notes.md` Q4: the charger chip's last-resort stop, moved to RBG 876 Ω, acts at 2,116–2,316 V (2,409 V with the comparator's maximum overdrive) (D-65) |
| Fault discharge current | ≥ 10 discharges into a shorted output at the device's highest hardware-limited voltage: **≈ 460 A** peak [CALC: 2,142 V (top of the over-voltage comparator's band, HARDWARE_DESIGN §B4.4 v0.3) / 4.7 Ω internal resistance = 456 A] | replaces Appendix A1's "about 400 A" (HARDWARE_DESIGN §14 already asked for 470 A) |
| Lead and terminal construction | Terminals that cannot come apart without a tool; the maker states the pull strength | I-5: the internal bleed is a backup, not a reason to accept a loose terminal |

## 3. Evidence to request (in addition to Appendix A6)

1. Bleed: resistance at −20, 25 and 60 °C; element count and each element's voltage rating; 1,000 h at 60 °C and
   2,200 V drift.
2. Surge: test report of ≥ 100 events at 2,420 V for 1 s at 60 °C, with capacitance and dissipation factor before
   and after; and the single-event survival voltage.
3. Fault discharge at ≈ 460 A, ≥ 10 events.
