# aed-rhythm

Heart-rhythm analysis for the Open AED firmware: it looks at the heart signal and answers **shockable**, **not shockable** or **cannot tell**.

NOT A MEDICAL DEVICE. See [SAFETY.md](../../../SAFETY.md).

## What it does, in plain terms

1. **Cleans the signal.** Removes slow drift (below 1 Hz), mains hum (50 and 60 Hz) and fast noise (above 30 Hz).
2. **Looks at the last 4 seconds, once a second.**
3. **Checks the signal is usable.** If the input is at the front end's limit, has flat tops (it was clipped somewhere before the analyser), is absurdly large (over 12 mV), or is full of high-frequency noise, the answer is "cannot tell". If it looks like chest compressions, the answer is "cannot tell".
4. **Checks for flatline.** Under 100 microvolts peak to peak, or a noise-like signal under 25 microvolts root-mean-square (what real asystole looks like through a recorder), is asystole: "not shockable".
5. **Asks two independent families of methods.** One looks at the signal over time (how much of the time it is busy), the other at its frequencies (how regular and wave-like it is). Both are classic, published methods. Both must say "shockable" for that window to count as shockable; if they disagree, the answer is "cannot tell".
6. **Waits for confirmation.** "Shockable" is reported once the current window and at least 3 of the last 5 windows say shockable (at the earliest 3 windows in a row, 7 s after the analysis starts). Every window that is not shockable is still reported at once, so a change of rhythm cancels a shock being prepared as quickly as before.

Small VF (under 200 microvolts, "fine VF") is answered "cannot tell", not "shockable". That is a deliberate lean toward not shocking and should be revisited with clinical advice.

Every window's measurements and verdict can be read out for the event record.

## Starting a fresh window without the settling second (decision D-47)

A freshly reset analyser throws away its first second while its filters settle, then needs a 4-second window and three agreeing windows: "shockable" at the earliest 7 s after the start. Compression sensing (D-36) restarts the analysis each time the chest becomes still, so that second was paid again on every hands-off pause (finding F-14).

Two calls now let the rescue sequence avoid it, without changing anything else:

- **`track(sample)`** passes a sample through the filters only. The rescue sequence calls it during CPR and while compressions hold an analysis back, so the filters are always running on the real signal.
- **`restart_window()`** starts a fresh window **now**, forgetting the previous window, its measurements and its confirmations, but keeping the filters. It waits only a quarter-second guard (62 samples), or whatever is left of the full second if the filters were reset less than a second ago. The rescue sequence uses it only when compression sensing confirms the chest is still; in every other case (pads just attached, after a shock, sensing off, failed or unsure) it uses the full `reset()` exactly as before.

Why that is enough (made-up signals, `tests/fresh_window.rs`): after a compression artifact stops abruptly, what is left of it in the filters falls below 10 % of its size within 0.45 s, 3 % within 0.74 s and 1 % within 0.87 s. With artifact of 0.5 to 20 mV at 60-180 compressions a minute, stopping at any point of a compression exactly as the window starts, over normal rhythm at 40-180 beats a minute and over flatline, no run in 13,200 reached "shockable", with the guard or without it; VF and rapid VT were still detected in every run, 0.75 s sooner. The guard is therefore a margin, not a measured necessity: it keeps the sharp end of the artifact out of the first window. On real recordings (PhysioNet development half) running the filters beforehand made no difference at all; the only effect is that the windows fall 0.75 s earlier in the signal (`crates/aed-eval/TUNING_LOG.md`, "D-47 timing change"). No threshold was changed.

## What has been shown, and what has not

**Shown (simulated only):** on made-up signals from the `synth` module, the code runs, never crashes, never answers "shockable" for normal rhythm at 40-180 beats per minute, flatline, heavy noise, clipped input or compressions over a normal rhythm, and does answer "shockable" for made-up VF and rapid VT. After VF turns into a normal rhythm it stops answering "shockable" within a few seconds.

**Tested on public recordings (27 November 2026):** some settings were re-chosen on half of the PhysioNet recordings and tested once on the other half (`crates/aed-eval/REPORT.md`, `TUNING_LOG.md`). On the held-out half it caught 71 % of coarse VF (channel 0) against the 90 % goal: **the goal is not met.** Normal rhythm was left alone 99.8 % of the time. These are hospital and Holter recordings, not cardiac arrests through pads; they say nothing about clinical performance.

**Not shown:** anything about real patients or real cardiac arrests. Rapid VT with a known rate, fine VF, asystole (only four patients), movement from chest compressions on real signals, children, and out-of-hospital pad recordings are all untested. The American Heart Association's 1997 goals, at least 90 % detection of coarse VF, 75 % of rapid VT, 99 % correct "no shock" on normal rhythm and 95 % on flatline and other rhythms, are **not met or not testable** for this code. Published work (Amann et al. 2005) found that each of these classic methods on its own falls well short of those goals on real recordings; combining them is the reason for the two-family design, and on these data it is still not enough.

## Where the numbers live

All thresholds are in `src/settings.rs`, each with its source or marked "UNVERIFIED", "TUNED ON SYNTHETIC" or "TUNED ON REAL DATA" (with the tuning step it came from). The same values are gathered in `Settings::DEFAULT`, which the evaluation tools use to score alternatives; the device never changes them while running.

## Running the tests

From `firmware/`:

```
cargo test -p aed-rhythm -- --nocapture
```

The `synthetic_confusion_table` test prints a table of outcomes per made-up rhythm class. `tests/fresh_window.rs` holds the D-47 measurements and safety checks; its slow comparison of guards is ignored by default:

```
cargo test --release -p aed-rhythm --test fresh_window -- --ignored --nocapture
```
