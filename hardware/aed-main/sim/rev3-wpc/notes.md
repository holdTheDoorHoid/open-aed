**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# WP-C revision 0.3 checks: the latching over-voltage comparator (sheet 08)

27 November 2026, ngspice 42 and Python 3, written with AI assistance. Every number is **simulated** with
hand-built models (behavioural comparator and follower; the BSS138 fit from
`../charger-storage/models/datasheet_models.lib`; a BAS116 diode fitted to its datasheet maximum forward drop), or
**calculated**. Nothing here is safe or ready for use on a person or animal, and nothing here is an instruction to
build or power anything.

Run: `bash run.sh` (about 1 minute). Results: `out/summary.txt`. Full discussion and the part-failure table:
`../../capture-notes/WP-C.md`, section "Revision 0.3".

## In plain language

1. **Where it trips.** With every part at the worst end of its tolerance from −20 to +70 °C, the comparator trips
   between **2,071 and 2,141 V** of capacitor voltage (up to 2,151 V if the reset transistor leaks as much as
   100 nA, an estimate). That is above every normal charge (2,008.6 V) and below the capacitor's 2,200 V rating.
2. **It latches and resets as intended** (simulated): it powers up untripped (ideal comparator), trips at
   2,106 V, stays tripped with the capacitor back at 0 V, clears on a reset pulse, and a reset wire stuck "on"
   does not stop the test from tripping it.
3. **Test-input position matters.** With the test fed in where the change order put it, a broken input
   resistor (R805 open) kills the real trip but the daily test still passes. Fed in at the measurement
   amplifier's input (as drawn), the same fault fails the test.
4. **Not closed:** a partial leak from the trip input to ground (for example a damaged reset transistor at
   1 MΩ) raises the trip to 2,211 V and the daily test still passes, in either arrangement.

## `ovp_band.py` — trip band (calculation)

Worst case of every term at once (2^10 corners) and a 200,000-sample uniform Monte Carlo; tolerances and sources
are in the script. Result: nominal 2,106.6 V; worst case 2,071.5–2,141.1 V (2,071.5–2,151.3 V with 100 nA of
leakage on OVP_SENSE); Monte Carlo 0.1–99.9 % 2,085–2,137 V. Largest terms: string ratio ±14.2 V, leakage
estimate +10 V, reference ±4.1 V, comparator offset ±4.0 V.

## `ovp_latch.cir` — latch, reset, test, faults (simulation)

`arr = 1` is the circuit as drawn (test into `VCAPS_IN` through 4.7 kΩ); `arr = 2` is change order C-4 (test into
`OVP_SENSE` through 22 kΩ). Columns: OVP_TRIP at power-up / during the test / after the test / after a reset;
real trip on a 0–3.0 V tap ramp; latched at 0 V; test with the reset line held high.

| Case | Test trips and latches | Reset clears | Real trip | Verdict |
|---|---|---|---|---|
| arr 1 healthy | yes | yes | 2,106 V | works; stuck-high reset still lets the test trip |
| arr 2 healthy | yes | yes | 2,106 V | works |
| arr 1, R805 open | **no** | — | none | detected by the test |
| arr 2, R805 open | **yes** | yes | **none** | **hidden** |
| arr 1, D801 short | yes | yes | none (reading ≈ 3 %) | detected by the reading comparison |
| arr 2, D801 short | trips, does not hold | — | none | detected by the test |
| arr 1, Q801 drain–source short | no | — | none | detected by the test |
| arr 1 / arr 2, Q801 1 MΩ leak | yes | yes | **2,211 V** | **hidden in both** |
| arr 1, C812 open | yes | **no** | 2,106 V | detected (reset step) |
| arr 1, C812 short | yes | yes | 2,106 V | detected only by the stuck-high-reset step |

Limits of the models: the comparator is ideal apart from a settable offset (no power-up behaviour, no
propagation delay beyond 50 Ω × 20 pF); the follower is ideal; the tap source on sheet 11 is a 149.8 kΩ resistor;
the BAS116 model is fitted to the 1 mA maximum only.
