**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# WP-F revision 0.3: sheet 11_hv_storage simulated as drawn

## Correction, 28 November 2026 (simulation audit)

These netlists set the temperature correctly (after `reset`), but two other errors of the kind this folder's own
"pitfalls" list warns about were found, and they change results quoted in `capture-notes/WP-F.md` "Revision 0.3"
and in `capture-notes/01-rev3-integration.md` (J-2). That capture note is outside `sim/` and was not edited; this
section supersedes the figures named below.

**1. `hold_held_wpf.cir` read the hold while the cold pump was still starting.** "Held", the hold level and HELD_N
were measured at 30–40 ms after power-up, but at −20 °C the pump reaches a steady "held" only at 31–42 ms (at 30 ms
the gate is still on its way down). The "fails" verdicts at −20 °C were start-up being read as chatter. The netlist
now measures at 70–100 ms, with the 1 ms gap at 100 ms, the stop at 150 ms and the restart at 190 ms (40 ms
windows). Proof: the same cases, same models, same temperature, only the window moved:

| Case (R3.4 table) | Was: hold worst / highest HELD_N while held | Corrected |
|---|---|---|
| −20 °C, aged, 3.1 V, 1.35 / 1.5 / 1.65 kHz, pilot 2.0 V | −5.59 / −5.52 / −4.30 V; 4.92 / 4.97 / 4.99 V — **fails** | **−6.70 / −6.73 / −6.73 V; 0.58 V — holds** (steady "held" after 33.9 / 34.5 / 41.7 ms) |
| −20 °C, new sensors, 3.1 V, 1.5 kHz | −5.52 V; 4.95 V — fails | −6.73 V; 0.56 V — holds |
| −20 °C, aged, 3.1 V, 1.5 kHz, pilot 0.5 V | −5.87 V; 2.88 V — fails | −6.74 V; 0.58 V — holds |
| −20 °C, aged, 3.2 V, 1.5 / 1.35 kHz | −6.04 / −5.91 V; 0.78 / 2.19 V | −6.85 / −6.82 V; 0.58 V |
| −20 °C, aged, 3.3 V, 1.5 kHz | −6.23 V; 0.63 V | −6.96 V; 0.57 V |
| −20 °C, aged, 2.9 V | never held | never held (unchanged: power-up lock-out) |
| 25 °C nominal, 3.3 V | −5.87 V; 0.59 V | −7.02 V; 0.56 V |
| 70 °C, new sensors, 3.2 V, 1.35 kHz, pilot 0.5 V | −6.04 V; conducting 20.9 ms after stop | −6.66 V; **21.7 ms** |
| 70 °C, aged, 3.1 V, 1.65 kHz | −6.33 V | −7.02 V |
| 25 °C, fresh, 3.5 V, 1.65 kHz, pilot 0.5 V | −6.33 V; conducting **22.0 ms**, "not held" 19.0 ms | −7.22 V; **23.2 ms**, 20.4 ms |
| −20 °C, LED shunt 47 k (not drawn) | HELD_N 4.42 V | 0.58 V |
| −20 °C, sensor 2 zener shorted | never held, gate −3.92 V | never held, gate −4.54 V (unchanged verdict: visible) |
| −20 °C, sensor 2 zener open | never held, −5.57 V | never held, −7.26 V (unchanged verdict) |
| **70 °C, sensor 2 zener shorted** | **never held** (gate −4.43 V): "visible" | **held after 35 ms** (gate −4.91 V; HELD_N 0.57 / 0.55 V): **latent when hot** |

Conclusions that no longer hold: R3.0 item 2 / R3.7 problem 6 / J-2 "the cold held check does not work unless the
clock is at least about 3.2 V … at a true −20 °C with the 3.1 V clock the hold is about 0.4–0.5 V too weak": at
3.1 V the path holds with 0.72 V over the aged-cold threshold (5.98 V). What *is* true: it stops holding at all
between 3.1 and 3.0 V (lock-out), and from cold it needs up to 41.7 ms to read "held" (firmware waits 35 ms).
R3.6 "a shorted zener is now caught … visible at the next self-test": only in the cold; at 70 °C it is latent (the
other sensor still requires the real bias, so it stays safe). "Release ≤ 25 ms with the clock stopped low: met
(worst 22.0 ms)": 23.2 ms here, and 27.3 ms at a hotter fresh corner in `../rev3-fix` — not met there. The J-2
study and a proposed change are in `../rev3-fix/notes.md`.

**2. The worst static single fault (1,352.8 V, margin 0.19) came from a failed operating point.** In
`stack_static_wpf.ctl`, case `fs3_short` (string element R1109 ‖ R6117 shorted), cold, run 7 of 12: the DC sweep's
first point did not converge and ngspice fell back to a "transient op" (pitfall 2 below; the log shows it for this
run only). The 11 other runs gave 733–747 V. `fs3_outlier_check.cir` (new) re-solves run 7 with its exact random
values: DC sweeps land on 704, 1,353 or 1,392 V depending only on the step size or the source definition, with path
currents of 0.8–118 mA (a held stack draws 0.22 mA); charging the capacitor physically (0 → 2,200 V in 1 s, held
3 s) gives **735 V** on the worst device with 0.22 mA. The 1,353 V state is a solution the real stack does not
reach.

| Result | Was | Corrected |
|---|---|---|
| String element shorted, position 3, cold, Monte Carlo worst | 1,353 V (margin 0.19) | **747 V (0.55)** |
| "String element shorted, by position 1/2/3/4", cold | 655 / 747 / 1,353 / 748 V (worst 0.19) | 655 / 747 / 747 / 748 V (worst 0.55) |
| R3.0 "smallest margin about 0.19 (one cold Monte Carlo run …); 0.24 for every other fault" | 0.19 | **0.24** (gate TVS shorted, hot, 1,284 V); cold worst 0.27 (per-device 10 MΩ open, 1,219 V) |
| R3.6 string pair element shorted | ≤ 1,353 V (cold, one run) | ≤ 855 V (hot) / 748 V (cold) |

One other run used the same fallback (`fsc2_short`, capacitor C1102 shorted, cold); its worst, 748.7 V, is in line
with the other runs and is kept, but it has not been re-solved physically [unverified].

Also corrected outside this folder by this audit: `../charger-storage` and `../rev3` (pitfall 1 below — now fixed
there), and the zener model's ignored `tbv1` (the shared library now uses `TCV`; these netlists pass no drift, so
nothing here changes). Not affected: `held_threshold_wpf.cir` (correct; its 4.72 V and 0.28 V margin stand, J-7),
`stack_dynamic_wpf.cir`, `stack_shock_wpf.cir`, `discharge_wpf.cir` (corners are stated factors; re-run, identical).

---

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
