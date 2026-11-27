# aed-rhythm

Heart-rhythm analysis for the Open AED firmware: it looks at the heart signal and answers **shockable**, **not shockable** or **cannot tell**.

NOT A MEDICAL DEVICE. See [SAFETY.md](../../../SAFETY.md).

## What it does, in plain terms

1. **Cleans the signal.** Removes slow drift (below 1 Hz), mains hum (50 and 60 Hz) and fast noise (above 30 Hz).
2. **Looks at the last 4 seconds, once a second.**
3. **Checks the signal is usable.** If the input is clipped, absurdly large, or full of high-frequency noise, the answer is "cannot tell". If it looks like chest compressions, the answer is "cannot tell".
4. **Checks for flatline.** Under 100 microvolts peak to peak is asystole: "not shockable".
5. **Asks two independent families of methods.** One looks at the signal over time (how much of the time it is busy), the other at its frequencies (how regular and wave-like it is). Both are classic, published methods. Both must say "shockable" for that window to count as shockable; if they disagree, the answer is "cannot tell".
6. **Waits for confirmation.** "Shockable" is reported only after 3 shockable windows in a row (about 6 seconds of signal).

Small VF (under 200 microvolts, "fine VF") is answered "cannot tell", not "shockable". That is a deliberate lean toward not shocking and should be revisited with clinical advice.

Every window's measurements and verdict can be read out for the event record.

## What has been shown, and what has not

**Shown (simulated only):** on made-up signals from the `synth` module, the code runs, never crashes, never answers "shockable" for normal rhythm at 40-180 beats per minute, flatline, heavy noise, clipped input or compressions over a normal rhythm, and does answer "shockable" for made-up VF and rapid VT. After VF turns into a normal rhythm it stops answering "shockable" within a few seconds.

**Not shown:** anything about real patients. The made-up signals are cartoons. The thresholds were partly chosen so the cartoons come out right. Real testing has to come next, first on the public PhysioNet heart-rhythm databases and then on real cardiac-arrest recordings (decision D-20). The American Heart Association's 1997 goals, at least 90 % detection of coarse VF, 75 % of rapid VT, 99 % correct "no shock" on normal rhythm and 95 % on flatline and other rhythms, are **unproven** for this code. Published work (Amann et al. 2005) found that each of these classic methods on its own falls well short of those goals on real recordings; combining them is the reason for the two-family design, but whether that is enough is unknown.

## Where the numbers live

All thresholds are in `src/settings.rs`, each with its source or marked "UNVERIFIED" or "TUNED ON SYNTHETIC".

## Running the tests

From `firmware/`:

```
cargo test -p aed-rhythm -- --nocapture
```

The `synthetic_confusion_table` test prints a table of outcomes per made-up rhythm class.
