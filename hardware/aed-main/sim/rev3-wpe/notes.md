**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# WP-E revision 0.3 check runs (sheet 10_hv_charger)

27 November 2026, ngspice 42, written with AI assistance. Hand-built models from
`../charger-storage/models/datasheet_models.lib`; no maker's model. Nothing here is safe or ready for use on a person
or animal, and nothing here is an instruction to build, power or test anything. Discussion and conclusions:
`../../capture-notes/WP-E.md`, section "Revision 0.3".

Run: `bash run.sh` (about 1–2 minutes). Results: `out/summary.txt` (not committed).

| File | What it checks | Key results (27 Nov 2026 run) |
|---|---|---|
| `l3_trip_rev3.cir` | Arithmetic only: the primary-side trip band with RBG 876 Ω from the LT3751 rev D figures; the LT3751 inductance rules at the band top; reflected voltage at the clamp | true trip 2,115.9–2,315.9 V, 2,408.7 V with maximum overdrive (worst-case sums); Monte Carlo 2,120–2,311 V (2,396 V); 107 V above the window, 11 V under the 2,420 V surge rating at the very top |
| `charger_rev3.cir` | Charge time with a transformer efficiency factor (E-2), the v0.3 inductance tolerance and standing load 2.80 MΩ; an [EST] bound for the LT3751 one-shot clock; flyback duration at the trip top; clamp energy at the trip top | to 1,950 V: nominal 4.18 s, typical cold 5.01 s, worst 5.94 s (90 % transformer), 7.10 s at 75.6 %; clock bound at its 32 µs minimum with L +10 %: 7.08–7.14 s; flyback 3.04 µs at 2,316 V and 2.93 µs at 2,409 V (7.125 µH); clamp ≤ 37.5 µJ/cycle at 2,409 V with a cold-minimum TVS (21 µJ at 2,200 V nominal TVS) |

Limits of these runs: the transformer is a coupled-inductor model with a loss *factor*, not a transformer model; the
LT3751 is not modelled as a circuit (no maker model ngspice can run); how the LT3751's one-shot clock limits a long
cycle is **not stated in the datasheet**, so the clock corners are a pessimistic bound, not a prediction; the
SMCJ45A breakdown figures are from memory (datasheet not opened) and its cold factor is an estimate.
