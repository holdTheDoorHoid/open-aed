# Integration notes: revision 0.3 capture of aed-main

**NOT FOR CONSTRUCTION — unreviewed design.** Drawn with AI assistance. Simulated figures come from hand-built part models. Not reviewed by a qualified engineer, never built, never measured.

28 November 2026. All eight work packages were redrawn in parallel from design revision 0.3 and contract issue 3, then merged. The firmware was updated in the same round.

## Result of the merge

| Check | First capture | Revision 0.3 |
|---|---|---|
| Sheets | 16 | 16 |
| Parts | 1,362 | 1,558 |
| Nets | 719 | 789 |
| Pin connections matching the pin tables | 3,669 | 4,150 of 4,150 |
| KiCad rule check errors | 0 | 0 |
| Warnings not accepted | 0 | 0 (23 accepted: 22 footprints still to be drawn, 1 unused pin tied to ground) |
| Inter-sheet nets | 127 | 137; 132 on at least two sheets; the 5 on one sheet are local rails |
| Renames needed at merge | none | none |
| Firmware tests (simulation only) | 542 | 578 reported by the firmware work; rerun at merge recorded in the commit |

A clean check says the sheets are wired together consistently. It does **not** say the circuit works.

## What the first simulations changed

Three studies (`sim/bridge`, `sim/charger-storage`, `sim/logic-relay`) ran before the revision. They overturned two fixes proposed in the first integration notes (a fifth discharge transistor does not help; two reference diodes in series do not help), found a shorted-switch case the failure table had missed, and found that compression sensing failed its noise limit as drawn. Revision 0.3 was written from their results. Each redraw package then simulated its own sheet as actually drawn (`sim/rev3-wp*`).

## Problems raised during the redraw, not yet resolved

Each package's notes (`WP-A.md` to `WP-H.md`, section "Revision 0.3") and `firmware/REV3_NOTES.md` give the detail. Most serious first. Numbering continues from the first integration notes.

### Would stop the device working, or weakens a protection

| No. | Problem | From |
|---|---|---|
| J-1 | **Earlier "cold" and "hot" simulation results were really room temperature.** `sim/charger-storage` and `sim/rev3` set the temperature in a way the simulator undoes, so their −20 °C and 70 °C semiconductor corners ran at 27 °C. Conclusions drawn from those corners need re-running. | WP-F |
| J-2 | **The discharge paths do not hold reliably at a true −20 °C.** With a 3.1 V clock both "held" lines flicker; at 2.9 V the path never holds. Fails safe (no charge), but the device would be unusable in the cold. The fix is the clock amplitude on sheet 09 or the pump, not sheet 11. | WP-F |
| J-3 | **No self-test reaches the new bridge protection** (80 A trip, 3 A fast path, kill latch, carrier-loss detectors, gate-supply monitor). If the fast path fails silently and a switch then shorts, the surviving switch is destroyed (simulated). A proposal for three supervisor test-injection lines needs a contract change. | WP-G, WP-D |
| J-4 | **The new voltage-sharing parts drain the capacitor at about 7 V per second** (total standing load about 2.8 MΩ, was 50 MΩ in the firmware's model). Keeping the capacitor topped up means frequent charger bursts, and compression sensing is blanked during each. Blanking would cover most of the armed time. Bears on Q-58. | firmware, WP-F, WP-G |
| J-5 | **A partial leak on the over-voltage sense node can raise the trip to about 2,211 V** (above the capacitor's rating) while the self-test still passes, because the test checks that the limit fires, not at what level. | WP-C |
| J-6 | **If the local film capacitor string fails open, turn-off reaches about 3.59 kV** with 1 µH of cable, over the switch's 3.3 kV rating. Needs a cable-inductance limit or a second film pair. | WP-G |
| J-7 | **One "held" sensor margin is 0.28 V, not the 0.71 V stated.** | WP-F |
| J-8 | **Shorted-switch survival has no margin at the worst single sensitivity** (150 °C on the pessimistic estimate with no shunt inductance). | WP-G |
| J-9 | **A transistor's hot leakage (2N7002, no guaranteed figure at 3.3 V) can push the relay-transferred signal past the firmware's wait,** refusing every shock only when hot. Fails safe; a room-temperature self-test cannot see it. | WP-D |
| J-10 | **The steering diode chosen (GP02-40) is marked "not for new designs"** by its maker. A replacement needs ≤ 50 µA leakage at 70 °C. | WP-F |
| J-11 | **An open gate protection diode on a discharge transistor allows up to 73 V gate-to-source** against a ±30 V rating (one hot run). | WP-F |
| J-12 | **The battery load test can stay on if one resistor (R112) fails open.** Predates this revision. | WP-A |
| J-13 | **The charger chip's upper inductance limit was never checked;** at a cold pack the 7.5 µH nominal already exceeds it. Bears on charge time. | WP-E |

### Smaller, or documentation

| No. | Problem | From |
|---|---|---|
| J-14 | Contract issue 3 has stale text: §10 says 128 kHz (now 96 kHz) and a 10 µs reset pulse (now ≥ 1 ms, R518 added on sheet 05); §8 says a 1 kΩ drive resistor and lists a 10 × 4.02 MΩ string that does not exist; sheet 11's divider is 1:1,001.0 where the contract says 1:1,002.0. | WP-B, WP-C, WP-E, WP-H |
| J-15 | The over-voltage test now injects at the amplifier input, not the comparator input, so that an open resistor cannot disable the real limit unseen. Design §B14.5 and contract §5.4 need the new description. | WP-C |
| J-16 | D-62 says 60 V and design §B14.5 says 50 V for the self-test's "capacitor empty" condition. Firmware uses 50 V. | firmware |
| J-17 | The made-to-order relay needs a limit on coil current-rise time (a hot coil of about 100 mH or more misses the 40 ms limit). | WP-H |
| J-18 | The relay supply's current-limit resistor (515 kΩ) is outside the range the chip's data sheet graphs. | WP-H |
| J-19 | The impedance-noise margin rests on an estimated chip-noise figure; at twice the estimate it can exceed the 10 mΩ limit. | WP-H |
| J-20 | The fast kill path adds about 60 ns on sheet 12; the per-phase cap bands are slightly wider than stated (10.75–11.92 and 7.31–8.10 ms). The energy bound should be recomputed at the new tops. | WP-D |
| J-21 | A relay-contact bounce of about 10 µs still lets the transfer signal rise at 8.98 ms against the 10 ms rule. | WP-D |
| J-22 | The over-voltage latch may power up tripped (safe direction); firmware now resets it at switch-on. `OVP_SENSE` has no noise filter. | WP-C |
| J-23 | The battery protector's over-discharge threshold alone can be as low as 32.6 A; the "≥ 35 A or ≥ 1 ms" rule is met only by the delay. Its minimum test current is still unpublished. | WP-A |
| J-24 | `HV_CAP_FM` should be in the pulse net class; the fast comparator's ground must be taken at the shunt (layout notes). Single parts on sheet 11 can see 1.3–1.6 kV after a fault, so coating is likely needed. | WP-G, WP-F |
| J-25 | The firmware's switch tests run at 0 V, so current-based checks see nothing; the lockout cannot be seen on the drawn status lines; whether sequence start is edge- or level-triggered is unsettled. | firmware |
| J-26 | Sheet 12 was checked for connectivity and reasoned through; it has not been simulated as a whole. | WP-D |

## Departures from the change order that were accepted at merge

- Sheet 05: R518 220 Ω in series with `OVP_RST` (controller pin current limit).
- Sheet 01: R112 is 4.7 MΩ, not 100 kΩ (the chip's input clamp, left out of the earlier model, would have left 10–14.5 W in the load with a toggling control line).
- Sheet 08: over-voltage test injected at the amplifier input.
- Sheets 09/12: seven extra default resistors on new internal nodes.
- Sheet 11: each sharing-string element is two 10 MΩ parts in parallel (a single open part would have put about 1.3 times the rating on one transistor); one capacitor part number replaced.
- Sheet 13: four value changes in the fast idle-current path so that it works with the real amplifier.

## Unverified items

As in the first capture: several makers' sites refused automated access; custom symbols stay `PinoutVerified = NO` until a person checks them; 22 footprints are named but not drawn; several part numbers were composed from ordering schemes. Named in this round: MMBFJ270 pin numbers, the KEMET 3.3 nF 2 kV part, HVCR and CRHV resistor codes, the TLP187 dark current, the NXP port reset values.
