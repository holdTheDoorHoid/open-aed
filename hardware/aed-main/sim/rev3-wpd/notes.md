**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# WP-D revision 0.3: timing checks of the new shock-logic circuits

27 November 2026, ngspice 42, written with AI assistance by the WP-D capture engineer. These runs check the
timers that revision 0.3 adds to or changes on sheets 09 and 12 (change order `capture-notes/rev3-changes.md`,
WP-D D-6, D-10, D-11, D-14). Every number is **simulated** with the behavioural models of
`../logic-relay/models/behavioural.lib` (written from datasheet limits, no maker's model). Nothing here is safe or
ready for use on a person or animal, and nothing here is an instruction to build or power anything.

Run: `bash run.sh` (about a minute). Results: `out/summary.txt` (not kept in git).

## In plain language

1. **The two new per-phase time limits** land where the design wants them: phase 1 is cut 10.9–11.9 ms after it
   starts, phase 2 7.4–8.1 ms after it starts, at every tolerance and temperature corner. The firmware's longest
   planned phases are 10.0 and 6.7 ms, so a normal shock is never cut. The top of each band is a few hundredths of
   a millisecond above the figure the energy simulation used (11.85 / 8.06 ms); that moves the worst-case energy
   by well under 1 J, but the energy run should be repeated with these numbers.
2. **The window-start delay** holds the start of the 150 ms window back by 60–104 µs, comfortably longer than the
   10–16 µs during which the re-arm circuit still holds the window latch cleared, so the race found in the first
   simulation round cannot recur.
3. **The relay-contact bounce fix works** for any bounce of 15 µs or longer (transfer accepted 11.2–11.3 ms after
   the last bounce). A bounce of almost exactly the filter's own 10 µs, with the weakest discharge transistor,
   still lets the transfer be accepted after 9.0 ms instead of 10. That is a margin question for the relay
   contacts, not a path to an unwanted shock.
4. **New finding: the discharge transistor's leakage.** Its datasheet guarantees leakage only at 60 V
   (≤ 1 µA at 25 °C, ≤ 0.5 mA at 125 °C). Every 1 µA of leakage at the circuit's real 2 V delays the "relay
   transferred" signal by about 0.7–0.9 ms; at about 2.5 µA (slowest parts) to 4 µA (nominal parts) it passes the firmware's 14 ms wait and every shock would be
   refused. This fails towards "no shock", but would show up only when the device is hot, so a room-temperature
   self-test would not find it. A leakage figure at low voltage, or a part that states one, is needed.

## Results

| Netlist | Quantity | Early corner | Nominal | Late corner | Requirement |
|---|---|---|---|---|---|
| `timers_rev3.cir` | phase-1 cap, ms after the start | 10.93 | 11.43 | 11.92 | > 10.0 (firmware) + margin; change order 10.75–11.85 |
| | phase-2 cap, ms after the first phase-2 edge | 7.44 | 7.77 | 8.10 | > 6.7 (firmware); change order 7.31–8.06 |
| | `CHAIN` → `CHAIN_D`, µs | 59.6 | 79.0 | 104.1 | > `REARM_N` release |
| | `REARM_N` release after the first consent returns, µs | 9.7 | 10.1 | 10.4 | (`../logic-relay` races.cir B: 15.7 µs nominal, different start condition) |
| `xfer_bounce.cir` | `XFER` after the last release, bounce 5 / 8 µs | — | ignored (filter) | — | — |
| | same, bounce 10 µs, VGS(th) 1.0 / 2.5 V | — | 10.37 / **8.98** ms | — | ≥ 10 ms |
| | same, bounce 12 µs | — | 11.13 / 10.56 ms | — | ≥ 10 ms |
| | same, bounce 15 / 20 / 200 µs | — | 11.17–11.31 ms | — | ≥ 10 ms |
| | `XFER` after the first release vs Q1201 leakage 0 / 0.5 / 1 / 2 / 4 µA (nominal parts) | — | 11.31 / 11.65 / 12.01 / 12.81 / **14.86** ms | — | ≤ 14 ms (firmware wait); late corner without leakage 12.17 ms (LR §1.3) |

Corners: C0G ±2.2 % (±2 % parts) or ±5.5 %, thin film ±0.35 %, thick film ±2 %, TLV7031/7041 offset ±8 mV and
hysteresis 3/10/25 mV, BAS116 leakage 0 or 80 nA (its 150 °C limit used as a bound for −20…+70 °C), SN74LVC1G17 at
5 V VT+ 0.482–0.609 × VCC (SCES351Y) and input leakage ±5 µA. These are the bands of `capture-notes/WP-D.md` §3 and
`../logic-relay/timers.cir`; the parts' temperature coefficients are inside them.

**Why the phase-cap bands differ from the change order.** The change order scaled the simulated spread of the
25 ms cap (−5.4 / +4.3 %, LR §2) onto 11.37 / 7.73 ms nominal. Run directly, the low ends are later (10.93 vs
10.75 ms: the BAS116 leakage bound matters less at 113 k and 76.8 k than at 249 k) and the high ends slightly
later (11.92 vs 11.85 ms, 8.10 vs 8.06 ms: the nominal includes the comparator's 10 mV hysteresis and 3 µs
delay). The capture notes use the union, **10.75–11.92 ms and 7.31–8.10 ms**.

**Bounce at the filter edge.** A blip of exactly ≈ 10 µs is half-seen: the filter lets `K_AT_REST` go high for a
few microseconds only, not long enough for a weak 2N7002 (threshold 2.5 V, 0.8 V of gate overdrive at 3.3 V) to
empty the 100 nF fully. The count then restarts from a partly charged capacitor. The firmware's own wait (14 ms
from the last low sample, design §F) is the second barrier, but a 10 µs blip may fall between its 1 ms samples.

## Models and limits

- 2N7002: level-1 MOSFET, threshold swept over the datasheet's 1.0–2.5 V (onsemi NDS7002A/D Rev. 11, at 250 µA,
  25 °C), KP 0.08 A/V² [EST, as `../rev3/carrier_loss.cir`]; threshold temperature drift is not modelled beyond
  the swept range [EST]. Off-state leakage is a swept current source because no datasheet figure applies.
- The relay aux contact is an ideal switch with one bounce; real bounce trains are not modelled.
- The REARM release here starts from a fully charged RC with an ideal step on `ANY_N`; `../logic-relay/races.cir`
  (B) found 15.7 µs with its own start condition. The window-start delay exceeds both by ≥ 3.8×.

## Second round (not done here)

- Re-run `../rev3/per_phase_caps.sh` with the caps at 11.92 / 8.10 ms.
- Re-run `../bridge/q1_shorted_lowside.cir` with the drawn logic delay: from `IDLE_FAST_N` on sheet 12 to `G_HA`
  is 7 gates plus the 1 kΩ (≤ ≈ 60 ns to sheet 13's input threshold, ≤ ≈ 90 ns to 90 % [CALC]) on top of sheet 13's ≤ 0.1 µs, against the 0.1 µs total that the
  bridge simulation assumed for its 11–13 mJ result.
- Measure or obtain the 2N7002's drain leakage at VDS ≤ 3.3 V, 70 °C.
