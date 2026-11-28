**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# WP-F revision 0.3: sheet 11_hv_storage simulated as drawn

27 November 2026, ngspice 42, written with AI assistance by the WP-F capture engineer. These netlists are the
discharge paths of `hardware/aed-main/specs/11_hv_storage.toml` **as drawn after revision 0.3** (part by part, with
reference designators in the comments), not the change order's description of them. Models are hand-built from
datasheet figures (`../charger-storage/models/datasheet_models.lib` plus `models/wpf_models.lib`); no maker model
exists for the depletion MOSFETs. Nothing here is an instruction to build or power anything.

**Results, margins and what they mean are in `hardware/aed-main/capture-notes/WP-F.md`, section "Revision 0.3".**
This file only says how to run the netlists and what to watch out for.

Run: `bash run.sh` (30–60 minutes on an idle machine; most of it is `stack_static_wpf`). Logs and
`out/summary.txt` are written to `out/` (not committed).

| Netlist | What it checks |
|---|---|
| `stack_static_wpf.cir` (+ generated `stack_static_wpf.ctl`) | DC sharing of one stack: nominal at −20 / 25 / 70 °C; steering-diode leakage sweep; Monte Carlo without and with a shorted device at 1,950 and 2,200 V; **every part of the stack open and short** with tolerance spread; the change order's single 4.7 MΩ string for comparison |
| `stack_dynamic_wpf.cir` (+ generated `stack_dynamic_wpf.ctl`) | bounding fast events (1 µs step, 10 ms fall, 2 ms rise) with worst C0G tolerance patterns, Monte Carlo, each device shorted |
| `stack_shock_wpf.cir` | realistic charge, shock (2,200 → 1,200 V in 10 ms) and 1 s recharge, each device shorted |
| `discharge_wpf.cir` | emptying the capacitor through one and both paths, release edge, release failed, a shorted device; power and energy in every resistor type |
| `held_threshold_wpf.cir` | both held sensors' thresholds at three corners; one zener shorted / open |
| `hold_held_wpf.cir` | hold pump, release and both sensors, −20…+70 °C, clock 1.35–1.65 kHz, pilot 0.5 / 2.0 V, zener faults, clock-amplitude sensitivity |
| `gen_ctl.py` | writes the two `.ctl` files (see below) |

## Simulator pitfalls found while doing this (each produced plausible-looking wrong numbers)

1. **`option temp` set before `reset` is lost: the run is at 27 °C.** `reset` returns the circuit to the netlist
   temperature (checked with one BAT54 at 1 mA: 0.323 V after "−20 °C" + `reset`, the same as at 27 °C; 0.263 V with
   `option temp=70` given after `reset`). Here the temperature is set as `option temp=$tc` *after* `reset`.
   `../rev3/hold_pump_rev3.cir`, `../rev3/held_sensor_rev3.cir` and the `../charger-storage` hold/held netlists set
   it before `reset`, so their "−20 °C" and "70 °C" corners ran the semiconductors at 27 °C.
2. **A failed operating point is silently replaced by an unsettled pseudo-transient.** Several fault cases of the
   steered stack do not converge in `op`; ngspice then reports a "transient op" state that is not a DC solution
   (one showed 185 mA flowing into capacitors). Every static case here is solved as a DC sweep of the capacitor
   voltage from 0 V (continuation).
3. **Loop variables did not always reach `alterparam`.** In this work, a `foreach` variable used as an `alterparam`
   value inside nested loops, and a `foreach` variable in an `if` test in an included control file, produced
   "alterparam fs1=" / "missing if condition" errors and the *unaltered* circuit ran (cause not isolated). The
   control scripts are therefore generated with every alteration written out literally (`gen_ctl.py`).
4. **Transient steps.** With a 0.5 ms step limit the steered stack diverged to megavolts in some runs; with a 20 µs
   limit (and Gear integration for the long discharge runs) the results are stable and agree with the DC solutions.
5. A leakage current source with a kink at 0 V (`max(v,0)`) made the transient diverge; the GP02-40 leakage is
   continued linearly through 0 V.
6. In `discharge_wpf.cir` the run starts with `uic` (all stack nodes at 0 V and the full voltage applied in one
   step); peaks and energies are measured from the release at 0.2 s so that this artefact is excluded.
