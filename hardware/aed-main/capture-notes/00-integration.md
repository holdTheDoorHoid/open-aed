# Integration notes: first capture of aed-main

**NOT FOR CONSTRUCTION — unreviewed design.** Drawn with AI assistance. Not simulated, not reviewed by a qualified engineer, never built.

27 November 2026. All eight work packages were captured in parallel from contract issue 2 and merged.

## Result of the merge

| Check | Result |
|---|---|
| Sheets captured | 16 of 16 |
| Parts | 1,362 |
| Nets | 719 |
| Pin connections matching the pin tables | 3,669 of 3,669 |
| KiCad rule check errors | 0 |
| Warnings not accepted | 0 (21 accepted: 20 footprints still to be drawn, 1 unused pin tied to ground) |
| Inter-sheet nets | 127; all 122 signal nets appear on at least two sheets; the 5 on one sheet are local rails |
| Renames needed at merge | none |

Every package built cleanly on its own, and the merged design built cleanly with one warning to accept. This says the sheets are wired together consistently. It does **not** say the circuit works.

## Open issues to resolve in the next design revision

Each package's notes (`WP-A.md` to `WP-H.md`) give the detail. Most serious first.

### Would stop the device working, or leave a hazard

| No. | Issue | From | Proposed change |
|---|---|---|---|
| I-1 | **The battery charger is not connected to the midpoint between the two cells.** The contract has no wire for it, so the charger cannot regulate each cell as drawn. | WP-A C-1 | Add a `VBAT_MID` net from sheet 01 to sheet 02. |
| I-2 | **One over-voltage limit cannot be set to fit.** The transformer-side trip's tolerance band (about 1,988–2,197 V, up to 2,284 V) is wider than the gap between a normal adult charge (2,008.6 V) and the capacitor rating (2,200 V). | WP-E C1 | Re-centre it and state that the independent comparator is the only limit below 2,200 V; or find a tighter method. |
| I-3 | **Four transistors per discharge path are not enough.** One shorted device puts half the capacitor voltage (975 V at a normal charge) on its neighbour, above its cold rating. | WP-F F-1 | Five per path, or higher-voltage parts. |
| I-4 | **A shorted reference diode makes a discharge path read "held" at almost any level,** and no self-test can see it. | WP-F F-2 | Two elements in series; redo the threshold margins. |
| I-5 | **The off-board capacitor has no drain of its own** if its connector opens. | WP-F C-14 | Require a bleed resistor inside the capacitor from its maker. |
| I-6 | **The patient relay as specified does not exist as a catalogue part.** No relay found publishes a rating for the shock current or has a position-sensing contact. | WP-H | Treat as made-to-order; write its requirements sheet; ask makers. |
| I-7 | **Several faults in the shock logic stay hidden until the first real shock and then block every shock,** because the self-test only ever uses the test permit. | WP-D CI-4 | Add self-test steps that exercise the real permit path and the relay at low voltage. Needs an owner decision. |
| I-8 | **With a shorted low-side switch, the opposite switch may overheat** (estimated 130–305 °C) before protection acts; and turn-off current at the over-current trip is above the switch's published limit. | WP-G F-1, F-2 | Needs simulation and the maker's data; may need a faster detector or a different arrangement. |
| I-9 | **Firmware waits equal the hardware's minimum times,** so a real unit would sometimes refuse. | WP-D CI-1 | Lengthen four firmware waits (13 ms, 70 ms, 5 ms, 30 ms). |
| I-10 | **A short on the battery wiring leaves the device dark until docked,** and any steady load on that wiring stops the protector recovering. | WP-A C-3 | Rule: no resistive load under 100 kΩ on the battery rail; consider an independent cell for the alarm. |

### Performance short of the target

| No. | Issue | From |
|---|---|---|
| I-11 | Charge time is 7.4–8.0 s at the worst tolerance corners, against 7 s. | WP-E |
| I-12 | The speaker gives about 79 dB at 1 m, against 85 dB; this size cannot do more. The supervisor's alarm sounder is also probably too quiet. | WP-C, WP-B |
| I-13 | The 25 ms time cap limits a pulse's length but hardly its energy: up to about 179 J against 150 J planned. | firmware |
| I-14 | The heart-signal chip's measuring current is limited to 100 µA, so compression sensing has less noise margin than the design assumed (about 5.6 mΩ against a 10 mΩ limit, before charger interference). | WP-H |
| I-15 | Relay transfer takes an estimated 30–35 ms against a 40 ms firmware limit; its supply draws about 8 A briefly where 2 A was budgeted. | WP-H |
| I-16 | The main voltage chain meets 1% + 7 V only if its resistors drift under about 0.6% over life; no maker publishes that figure. | WP-C |
| I-17 | Hold and release timing of the discharge paths is marginal at −20 °C with the slowest allowed clock. | WP-F |

### Settings and smaller changes

| No. | Issue | From |
|---|---|---|
| I-18 | Run the supervisor's pad-resistance chip at 96 kHz, not 128 kHz, to keep a stray tone out of the heart signal. | WP-H |
| I-19 | The battery protector must be configured not to shut down when its data lines are low. | WP-A C-4 |
| I-20 | The supervisor's pad-resistance chip uses 1.8 V logic; a level shifter was added on its sheet. | WP-B C-3, WP-H |
| I-21 | The current-sense signal has almost no headroom at the main controller's input. | WP-B C-4 |
| I-22 | The over-voltage comparator does not latch; it releases near 1,000 V. A true latch needs a new reset wire. | WP-C C-5 |
| I-23 | If the main voltage tap wire opens, the "below 60 V" signal reads true at any voltage. The supervisor's own check covers it. | WP-C C-3 |
| I-24 | The hold pump needs four stages, and the release circuit was redesigned, relative to the specification. | WP-F |
| I-25 | The battery load test's time limit does not cover firmware toggling its control line rapidly. | WP-A |

## What cannot be settled on paper

A circuit simulator is not installed on the design computer. These need simulation or measurement: turn-off voltage spikes and snubber values; the order in which switches turn off when the bridge supply collapses; how the discharge transistors share voltage while switching; the consent detectors' timing margins; the relay supply's inrush; the charger's primary clamp; charger interference in the pad-resistance channel.

## Unverified items

Each package's notes list them. Common themes: several makers' sites refused automated access, so some datasheets were read from mirror copies or not at all; custom symbols are marked `PinoutVerified = NO` until a person checks them against the datasheet; 20 footprints are named but not drawn; many passive part numbers were composed from the makers' ordering schemes and not checked for stock.
