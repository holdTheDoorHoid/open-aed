# Tuning log: rhythm analyser on real recordings

27 November 2026. Every candidate tried on the **development** half, with its cross-validated (CV) score, and the single run on the **held-out** half. SIMULATION AND PUBLIC-DATABASE RESULTS ONLY; nothing here says the analyser is safe or ready for use on a person.

Everything below is reproduced by

```
cargo run --release -p aed-eval --bin aed-tune -- --data <data/physionet> --records crates/aed-eval/splits/development.txt \
  baseline count thresholds checks flatline clip confirm linear lowband final final-no-hold e5 synth-cpr frozen
```

`aed-tune` refuses any record list other than the development list (it checks the SHA-256).

## Rules of the tuning (fixed before the first candidate)

- **Data:** development half only, 53 records / 52 patients (`splits/development.txt`, SHA-256 `597e65c4…d477903`; see `splits/README.md`). Both channels used. 9,825 scored 12 s segments.
- **Unit of scoring:** what the device would do in one analysis: a fresh analyser on 12 s of one rhythm; shock / no shock / unable to analyse, and when. Also: whether a shock would be **cancelled during the 3 s stand-clear countdown** (the rescue sequence, aed-core, abandons a shock on any answer other than "shockable"), measured by carrying on into the next 3 s of the same rhythm. This was not measured in the first report (F-10).
- **Choosing:** leave-one-patient-out. For each patient, the candidate is chosen on the other 51 patients, and only that choice is scored on the patient left out. The pooled left-out results are the **CV** figures. "Resubstitution" (chosen and scored on everyone) is optimistic and is shown only for comparison.
- **Constraints on the training patients in every fold** (a candidate that breaks one cannot be chosen): false shocks on normal sinus rhythm ≤ 0.2 %; on other non-shockable rhythms ≤ 0.5 %; on asystole, no more than the untuned analyser gives on the same patients (the development set has only 4 patients with scored asystole and one doubtful stretch, so a rate limit is meaningless; this keeps asystole from getting worse).
- **Objective:** coarse-VF sensitivity at 12 s, plus 0.1 × the share of VF shocks not cancelled in the countdown, plus 0.1 × sensitivity at 8 s (the last two only break near-ties). Exact ties go to the candidate with fewer false shocks, then to the stricter one (candidates are listed strictest first).
- **No neural network, no flexible model.** A linear score was tested (L-1) and rejected.

## Step 1: the split (before anything else)

Made by `aed-eval --make-split` with the rule in `src/split.rs` (fixed-seed shuffle within strata of database × annotated VF / flutter only / neither; mitdb 201+202 one unit). Counts and hashes in `splits/README.md`. Development: 53 records; held-out: 52.

## T-0. Untuned analyser on the development half (the starting point)

Both channels, 12 s analysis, resubstitution (nothing was chosen).

| Class | n | patients | right at 7 s | 8 s | 10 s | 12 s (LCL 90 %) | unable to analyse | shocks cancelled in countdown |
|---|---|---|---|---|---|---|---|---|
| Coarse VF | 269 | 14 | 53.9 % | 58.7 % | 66.9 % | 71.4 % (67.5 %) | 16.7 % | **60 of 192** |
| Ventricular flutter | 52 | 5 | 44.2 % | 48.1 % | 55.8 % | 67.3 % (57.6 %) | 17.3 % | 7 of 35 |
| VT, rate unknown | 578 | 8 | 23.5 % | 25.4 % | 32.2 % | 34.8 % | 44.6 % | 62 of 201 |
| Normal sinus rhythm | 6,073 | 30 | 100.0 % | 100.0 % | 100.0 % | 100.0 % (99.9 %) | 4.0 % | – |
| Other non-shockable | 2,225 | 22 | 100.0 % | 100.0 % | 100.0 % | 100.0 % (99.9 %) | 7.7 % | – |
| Asystole | 112 | 4 | 93.8 % | 93.8 % | 93.8 % | 93.8 % (89.7 %) | 42.0 % | – |

New finding at this step: **31 % of the untuned analyser's correct VF shocks (60 of 192) would have been abandoned during the countdown**, almost always because a single window in the next 3 s said "not sure". Counting only delivered shocks, the untuned analyser's coarse-VF figure on these patients is 49 %, not 71 %.

## T-2. Clipping check (flat tops)

The old saturation rule (±32,000 µV) never fires on these recorders, which clip near ±5 mV. New measurement: share of raw samples in runs of ≥ 10 identical values at the window's highest or lowest value (window span ≥ 500 µV). Scored against the samples known to sit at the original recorders' limits:

| Rule | windows truly clipped (>1 % at the limit) caught | windows with no clipping flagged | coarse-VF windows flagged |
|---|---|---|---|
| flat tops > 0 | 90 of 102 | 14 of 78,359 | 78 of 2,152 |
| **flat tops > 1 %** (chosen) | 89 of 102 | 14 of 78,359 | 75 of 2,152 |
| flat tops > 2 % | 76 of 102 | 6 of 78,359 | 63 of 2,152 |
| flat tops > 5 % | 57 of 102 | 2 of 78,359 | 46 of 2,152 |

Chosen: 1 %, the same as the existing saturation limit. Not tuned for sensitivity: the recordings hold no labelled artefacts, so data can only show the rule's cost (clipped VF in cudb), not its benefit. On whole analyses the cost was nil at 12 s (71.4 % with or without), slightly lower at 7 s (53.9 → 53.2 %). The front-end range stays a setting (`SATURATION_UV`).

## T-3. Flatline (asystole) measure

Real asystole after the analyser's filters is not flat: recorder noise of 100–200 µV peak to peak. Share of windows each rule would call flatline:

| Rule | Asystole (896 windows) | Coarse VF | Flutter | VT | NSR | Other |
|---|---|---|---|---|---|---|
| peak-to-peak < 100 µV (untuned) | 1.0 % | 0 | 0 | 0 | 0.2 % | 0 |
| median 1-s peak-to-peak < 120 µV | 40.3 % | 0 | 0 | 0.1 % | 2.0 % | 0 |
| rms < 20 µV | 10.8 % | 0 | 0 | 0 | 1.1 % | 0 |
| rms < 25 µV | 37.1 % | 0 | 0 | 0.1 % (4 windows, vfdb 427 ch 1) | 2.3 % | 0 |
| rms < 30 µV | 48.9 % | 0 | 0 | 0.4 % | 3.4 % | 0 |
| rms < 40 µV | 60.5 % | 0 | 0 | 2.1 % | 4.8 % | 0.4 % |
| rms < 50 µV | 64.8 % | 0 | 0 | 3.9 % | 8.4 % | 1.7 % |
| **rms < 25 µV and leakage > 0.6** (chosen) | 37.1 % | 0 | 0 | 0.1 % | 1.2 % | 0 |

Rule for the choice: the largest rms limit that flags no coarse-VF or flutter window and at most 0.1 % of VT windows. (The rule was written down while looking at this table, not before it; with four patients there was no honest way to do better.) The leakage guard (> 0.6) keeps small *regular* waves (made-up fine VF has leakage 0.11–0.38 and rms 18–33 µV) out of the flatline rule, so they stay "fine VF, not sure", not "flatline". Real asystole windows have leakage ≈ 0.85. Kept alongside the old peak-to-peak rule.

**Evidence is thin:** 4 development patients have scored asystole; nearly all the windows the rule catches are one patient (vfdb 427, mostly channel 1). Two of the four (424, 607) carry "asystole" labels on signals of 300–900 µV that look like slow waves or VF, not a flat line. Cross-validation was not attempted with so few patients. Effect on whole analyses: asystole "unable to analyse" 42.0 % → 21.4 %, median time to answer 7 → 5 s; no change in sensitivity or false shocks.

B1 = untuned + T-2 + T-3. All later experiments build on B1.

## E1. Band-pass count threshold alone (11 candidates, 0.18 … 0.08; rest untuned)

Chosen in 53 of 53 folds: **0.16**. CV: coarse VF 76.2 % (72.6 %) at 12 s; NSR 100.0 %, other 100.0 %, asystole 93.8 %. Lower thresholds catch more VF but give more asystole false shocks (vfdb 607) than the untuned analyser, which the constraint forbids.

## E2. Count × leakage limit × dominant-frequency (DF) lower limit (36 candidates; confirmation untuned)

Chosen: count 0.16, leakage 0.625 (unchanged), DF 1.6 Hz (51 of 53 folds). CV coarse VF 76.2 % (72.6 %) — no better than E1 — with asystole CV 91.1 % (10 false shocks against 7). **Lowering the DF limit gave no cross-validated gain.** See T-6 for why it was then kept at 2.4 Hz.

## E3. Confirmation rule × count × DF (160 candidates, on B1)

Rules: 3 of 3 (untuned), 3 of 4, 3 of 5, 2 of 2, 2 of 3; "no shock" announced after 1 or 2 non-shockable windows; with or without a one-window "hold" during the countdown (a single "methods disagree" window straight after a "shockable" announcement is not announced). Chosen: count 0.16, DF 1.6, 3 of 4, no-shock after 1, hold (51 of 53). CV: coarse VF 76.6 % (72.9 %), shocks cancelled in countdown 22 of 206; NSR 100.0 %, other 100.0 %, asystole 92.9 % (88.7 %).

**E3b (what-if, not used):** the same without the asystole constraint chooses count 0.14, 2 of 3, no-shock after 2: CV coarse VF **82.5 %** (79.2 %), but NSR 99.9 % (9 false shocks), other non-shockable **99.0 %** (22 false shocks), asystole 91.1 %. This is the size of the trade the constraint refuses: about 6 points of VF sensitivity for a visible loss of specificity. Not taken.

## L-1. A linear score instead of thresholds (rejected)

Window level, leave-one-patient-out logistic regression, compared at the same share of non-shockable windows called shockable as the threshold rule (0.185 %). Windows reaching the family votes: 1,865 coarse VF, 60,680 non-shockable.

| Classifier | coarse-VF windows caught at the same false rate |
|---|---|
| **thresholds, both families must agree (E3 choice)** | **79.9 %** |
| linear score, time family (TCSC, count) | 55.8 % |
| linear score, frequency family (leakage, concentration) | 56.5 % |
| linear score over both families' four features | 66.1 % |
| linear score over six features (with low band, DF) | 57.9 % |

A linear score is worse than the AND of thresholds in the far tail that matters for specificity. Plain thresholds kept; no coefficients added.

## T-6. Dominant-frequency lower limit: kept at 2.4 Hz

Real VF windows with DF ≤ 2.4 Hz (13.5 % of coarse-VF windows) look, on every feature the analyser has, like the made-up compression artefact (table in T-7). With DF > 1.6 or 2.0 Hz, twice as many made-up compression-over-asystole windows (at 140/min) were called shockable as with 2.4 Hz (16 against 8 of 1,920), while the cross-validated sensitivity gain was nil (E2, E4c gives the same 76.6 % at 2.4 Hz). Kept at 2.4 Hz.

## T-7. Low-band ("compression") rule: kept unchanged

Real coarse-VF windows with low-band fraction > 0.5 (220 windows) against made-up compressions over sinus rhythm or asystole (1,920 windows, rates 80–150/min, 500–3,000 µV), 5 %–median–95 %:

| feature | real VF, low band > 0.5 | made-up compressions |
|---|---|---|
| low-band fraction | 0.51–0.68–0.94 | 0.28–0.69–0.87 |
| dominant Hz | 1.00–1.25–2.75 | 1.25–1.75–2.25 |
| concentration | 0.34–0.59–0.86 | 0.30–0.78–0.88 |
| leakage | 0.45–0.59–0.76 | 0.35–0.59–0.81 |
| TCSC | 0.51–0.65–0.77 | 0.36–0.77–0.85 |
| count | 0.07–0.14–0.25 | 0.04–0.08–0.33 |

They overlap on every feature. Reformulations tried (count 0.16, DF 2.4 Hz): low band > 0.6 or > 0.7, low band > 0.5 with concentration > 0.6, low band > 0.5 with DF < 2.4, no rule. With DF kept at 2.4 Hz none of them changes which real VF windows are shockable by more than 7 of 2,152 (1,481 → 1,488), because the DF limit already blocks those windows. What the rule really does is turn "no shock" into "not sure, keep analysing" when the low band dominates — which is its safety job (F-1: a compression window over VF must not end the analysis with "no shock"). Loosening it would cut "not sure" on normal and other rhythms (7.7 % of windows → 1.2–3.2 %) but would let compression windows end analyses with "no shock". **Conclusion: from the ECG alone this cannot be separated; a compression signal is needed (pad impedance is the obvious one). The rule is left exactly as it was.**

Also found: made-up compressions at **150/min over asystole** are shocked in 20 of 20 analyses, by the untuned and the tuned analyser alike (at 140/min and below: 0). A regular 2.5 Hz wave is what rapid VT at 150/min looks like; only a compression reference can tell them apart. Not changed by this work; recorded as an open problem.

## E4. Final joint choice (900 candidates, DF 2.4 Hz, low band unchanged, on B1)

Count {0.18 … 0.10} × leakage {0.625, 0.65, 0.70} × size limit {8, 12, 16 mV} × confirmation {3/3, 3/4, 3/5, 2/3, 2/2} × no-shock after {1, 2} × hold {no, yes}. Chosen: count 0.16, leakage 0.625, size 12 mV, 3 of 4, no-shock after 1, **hold** (51 of 53). CV coarse VF 76.6 % (72.9 %), cancelled in countdown 22 of 206, asystole CV 90.2 %.

**The hold was then rejected for safety:** with it, the simulator (aed-sim scenario `vf-converts-during-countdown`) delivered a shock 2.5 s after VF had turned into a normal rhythm, breaking a safety invariant. Any hold lengthens the time in which a rhythm change during the countdown is missed (finding F-4).

## E4c. As E4, hold not allowed (450 candidates) — **the frozen choice**

Chosen on all development patients: **count 0.16, leakage 0.625, size 12 mV, 3 of the last 5 windows, no-shock after 1, no hold** (50 of 53 folds; the others: size 8 mV, 3 of 4, 2 of 3 with leakage 0.70, once each).

Cross-validated, 12 s analysis:

| Class | channels | n | patients | 7 s | 8 s | 10 s | 12 s (LCL 90 %) | unable to analyse | cancelled in countdown |
|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | both | 269 | 14 | 62.1 % | 68.4 % | 73.6 % | **76.6 % (72.9 %)** | 14.9 % | 48 of 206 |
| Coarse VF | 0 | 181 | 14 | 56.9 % | 63.0 % | 69.1 % | 71.3 % (66.5 %) | 17.7 % | 31 of 129 |
| Coarse VF | 1 | 88 | 2 | 72.7 % | 79.5 % | 83.0 % | 87.5 % (81.7 %) | 9.1 % | 17 of 77 |
| Ventricular flutter | both | 52 | 5 | 59.6 % | 65.4 % | 75.0 % | 82.7 % (74.1 %) | 9.6 % | 9 of 43 |
| VT, rate unknown | both | 578 | 8 | 32.2 % | 37.2 % | 41.5 % | 43.3 % | 41.2 % | 69 of 250 |
| Normal sinus rhythm | both | 6,073 | 30 | 100.0 % | 100.0 % | 100.0 % | 100.0 % (99.9 %) | 3.9 % | 1 false shock |
| Other non-shockable | both | 2,225 | 22 | 100.0 % | 100.0 % | 100.0 % | 100.0 % (99.9 %) | 7.6 % | 0 false shocks |
| Asystole | both | 112 | 4 | 92.0 % | 90.2 % | 90.2 % | **90.2 % (85.6 %)** | 21.4 % | 11 false shocks |

Resubstitution (optimistic): coarse VF 76.6 % (72.9 %), cancelled 46 of 206; asystole 93.8 % (7 false shocks, all vfdb 607). The CV asystole figure is worse than resubstitution and worse than the untuned analyser (93.8 %): when vfdb 607 is the patient left out, the constraint no longer sees its doubtful stretch and a more lenient candidate is chosen. All 11 are in that one stretch. This is reported, not hidden: **on the development data the procedure does not meet the AHA asystole goal (95 %), and the frozen analyser matches but does not improve on the untuned one there.**

Delivered shocks (not cancelled in the countdown), coarse VF, resubstitution: 160 of 269 = 59.5 %, against 132 of 269 = 49.1 % untuned.

## E5. TCSC and concentration limits (12 candidates, E4c structure fixed)

First run with the E4 structure (3 of 4 with hold): TCSC 0.45 and concentration 0.40 kept in every fold. Re-run with the E4c structure: concentration 0.35 chosen in 52 folds, but only through the tie-breaker (3 fewer countdown cancellations, 43 against 46); 12 s sensitivity identical (76.6 %). Following the rule "ties toward not shocking", **0.40 was kept**.

## Synthetic compressions, whole analyses (untuned vs frozen)

Made-up compressions (80–150/min, 500–3,000 µV, 5 seeds) over sinus 50/80/120 and asystole: no shock in any analysis at ≤ 140/min with either analyser; 150/min over asystole: 20 of 20 shocked by both; 150/min over sinus 50: 1 of 20 by both. Lowering DF to 1.6 Hz (not adopted) changed none of these.

## Frozen settings

Written into `crates/aed-rhythm/src/settings.rs` with provenance: `COUNT_VF_MIN` 0.16, `MAX_PLAUSIBLE_PP_UV` 12,000, `CONFIRM_WINDOWS` 3 of `CONFIRM_SPAN` 5, `MAX_CLIPPED_FRACTION` 0.01 (new, with `CLIP_RUN_SAMPLES` 10, `CLIP_MIN_SPAN_UV` 500), `ASYSTOLE_MAX_RMS_UV` 25 with `ASYSTOLE_MIN_LEAKAGE` 0.6 (new). Unchanged: DF 2.4–9 Hz, leakage 0.625, TCSC 0.45, concentration 0.40, low band 0.5, noise 0.15, fine VF 200 µV, flatline 100 µV peak to peak.

Replay check: replaying the frozen settings on the stored measurements reproduces the real analyser's announcement on all 107,703 development windows (0 differ). `aed-eval --untuned` reproduces the original analyser's development-half tables exactly.

Source hashes of the analyser at freezing (SHA-256): `analyzer.rs` fb8ef0c2…, `features.rs` 839fa51e…, `settings.rs` b7e3528f…, `filter.rs` 597e8cb3… (unchanged), `synth.rs` 10629034… (unchanged).

## Held-out run (once)

- **When:** 2026-11-27T13:38:41Z, after everything above was written and the analyser's source hashes recorded.
- **Command:** `aed-eval --data ../data/physionet --records crates/aed-eval/splits/held_out.txt` (list SHA-256 `3e5c7031…44d26601`), frozen `Settings::DEFAULT`. Full output: `reports/held_out.md`.
- **Nothing was changed after it.** No bug was found that needed fixing; there is no second run.

12 s analysis, held-out half, against the untuned analyser on the same patients (`reports/held_out_untuned.md`):

| Class | channel | n (patients) | untuned | frozen (LCL 90 %) | frozen: shocks cancelled in countdown |
|---|---|---|---|---|---|
| Coarse VF | 0 | 218 (16) | 58.3 % | **71.1 % (66.8 %)** | 53 of 155 |
| Coarse VF | 1 | 89 (2) | 75.3 % | 80.9 % (74.5 %) | 18 of 72 |
| Normal sinus rhythm | 0 | 3,007 (30) | 99.9 % (2 false shocks) | 99.8 % (99.7 %), 5 false shocks | |
| Normal sinus rhythm | 1 | 2,875 (27) | 99.7 % (10) | 99.4 % (99.1 %), 18 false shocks | |
| Other non-shockable | 0 | 1,321 (17) | 99.8 % (3) | 99.5 % (99.2 %), 6 false shocks | |
| Other non-shockable | 1 | 1,318 (16) | 99.9 % (1) | 99.8 % (99.6 %), 2 false shocks | |
| Asystole | – | no scored segments | – | – | |

All 31 held-out false shocks: vfdb 418 "NSR" (5 on channel 0, 18 on channel 1), cudb cu09 atrial fibrillation at 154–163/min (3), vfdb 426 supraventricular tachycardia (3 on channel 0, 2 on channel 1). The same three records gave all the untuned analyser's held-out false shocks (16).

**The 90 % coarse-VF goal was not reached on the held-out half (71.1 % on channel 0).** Specificity got slightly worse than untuned on the same patients, by 1 to 8 more false shocks per channel and class, all in the three records above; it stays above the AHA figures for the pooled classes (normal rhythm lower limit 99.7 % / 99.1 %), but vfdb on channel 1 alone falls to 97.3 % (lower limit 96.4 %).

### Changes after the held-out run (none to behaviour)

After the run, only documentation comments and test code (`#[cfg(test)]` unit tests in `aed-rhythm/src/analyzer.rs`; the updated expectation in `aed-eval/tests/findings.rs`) were added, so the analyser's source hashes changed. To show the behaviour did not: the development-half report regenerated afterwards is identical to `reports/development.md` (apart from the commit line), and the replay check still finds 0 of 107,703 windows different. The held-out half was not run again.

## D-47 timing change: regression checks (27 November 2026) — not new clean tests

Decision D-47 (owner, 27 Nov 2026): once compression sensing confirms the chest is still, the analysis no longer spends its first second letting the filters settle. The filters now run through CPR (`RhythmAnalyzer::track`) and the window starts at the hands-off moment after a 0.25 s guard (`RhythmAnalyzer::restart_window`, `FRESH_WINDOW_GUARD_SAMPLES` = 62 samples). **No setting of the analyser was changed or re-chosen.** The guard was fixed beforehand from made-up compression artifact (`aed-rhythm/tests/fresh_window.rs`), not from these recordings, and was not changed after any run below. The evaluation tool gained `--warm-start` (the device's new start: the 2 s of recording before each analysis pass through the filters first, then the 0.25 s guard) and, for measurement only, `--warm-guard-ms N`.

These runs are **regression checks of a timing change**. The held-out half and Cardially were each run **once**, at the end, with the analyser frozen (source SHA-256 at the time: `analyzer.rs` 43ffa5e0…, `features.rs` f5168f8f… (unchanged), `settings.rs` 6afccb07…, `filter.rs` 597e8cb3… (unchanged), `synth.rs` 10629034… (unchanged)). They are not new clean tests: both had been scored before, and only the start of each analysis differs.

### Development half (allowed; nothing chosen from it)

| Run (UTC) | Start | Coarse VF ch 0 / ch 1, 12 s | False shocks, all non-shockable, both channels |
|---|---|---|---|
| 2026-11-27T15:41:13Z | cold (as before) | 71.3 % / 87.5 % | 8 |
| 2026-11-27T15:42:50Z | warm, 1,000 ms guard | 71.3 % / 87.5 % | 8 |
| 2026-11-27T15:43:07Z | warm, no guard | 71.8 % / 87.5 % | 9 |
| 2026-11-27T15:41:30Z | **warm, 250 ms guard (the device)** | **72.4 % / 84.1 %** | **12** |
| 2026-11-27T15:43:53Z | warm, 500 ms guard | 73.5 % / 86.4 % | 11 |
| 2026-11-27T15:44:10Z | warm, 750 ms guard | 72.4 % / 88.6 % | 10 |

- The cold run is identical to `reports/development.md` (apart from the title line): the cold path did not change.
- A warm start with a full second's guard is identical to a cold start: running the filters beforehand changes nothing. Every difference between warm and cold therefore comes from **where the 4-second windows fall**: a warm window starts 0.75 s earlier in the signal. Within a 12 s analysis the number of windows is the same (eight), so the result at a fixed analysis length should be the same within that alignment noise.
- Across guards of 0 to 1,000 ms the figures move by up to 2 points (channel 0) and 4.5 points (channel 1, 88 segments from 2 patients), and false shocks between 8 and 12, without a trend. The device's 250 ms guard sits at the unfavourable end for false shocks (12 against 8): one more on normal rhythm in each channel (vfdb 419 at 3.1 s, the first segment of that record, and vfdb 419 at 1,339.5 s), and one more on asystole in each channel (vfdb 424 at 1,564.5 s, labelled asystole at 802 µV, and the known doubtful stretch of vfdb 607). This is reported, not tuned away.
- Time to decision: the median "shock" on coarse VF is 6 s instead of 7 s (decisions now fall at 4.25, 5.25, … s), and normal rhythm is answered at 4 s instead of 5 s.

### Held-out half (once; regression check)

- **When:** 2026-11-27T15:53:51Z. **Command:** `aed-eval --data ../data/physionet --records crates/aed-eval/splits/held_out.txt --warm-start` (list SHA-256 `3e5c7031…44d26601`). Full output kept outside the repository.

| Class, 12 s analysis | channel | before (cold, 2026-11-27T13:38:41Z) | now (warm, the device's new start) |
|---|---|---|---|
| Coarse VF | 0 | 71.1 % (LCL 66.8 %) | 72.0 % (LCL 67.8 %) |
| Coarse VF | 1 | 80.9 % (LCL 74.5 %) | 83.1 % (LCL 76.9 %) |
| Normal sinus rhythm | 0 | 99.8 %, 5 false shocks | 99.6 % (LCL 99.4 %), 11 false shocks |
| Normal sinus rhythm | 1 | 99.4 %, 18 false shocks | 99.5 % (LCL 99.3 %), 15 false shocks |
| Other non-shockable | 0 | 99.5 %, 6 false shocks | 99.8 %, 3 false shocks |
| Other non-shockable | 1 | 99.8 %, 2 false shocks | 99.8 %, 2 false shocks |

All 31 held-out false shocks (31 before as well) are in the same three records as before: vfdb 418 (26), cudb cu09 (2), vfdb 426 (3). Coarse-VF shocks cancelled in the countdown, channel 0: 51 of 157 (before 53 of 155). The 90 % goal is still not met.

### Cardially (once; regression check)

- **When:** 2026-11-27T15:54:06Z. **Command:** `aed-cardially --warm-start` (zip SHA-256 `a3bce9bd…ca735`). The reports rarely hold 2 s before the 9 s segment, so where less than 1 s is available the segment's own first 2 s, played backwards, are the pre-roll (a mirror; the signal is continuous where the analysis starts).
- **9 s before the first shock:** "shock" 60.6 % (152 of 251; lower 90 % limit 56.4 %), against **61.4 %** (154 of 251) cold. "No shock" 16.3 % (15.5 % before), inconclusive 23.1 % (23.1 %).
- **Consequence, not an improvement:** a 9 s recording now allows "shock" at 6.25 s instead of 7 s, but still holds five windows (at 4.25 to 8.25 s instead of 5 to 9 s), so it gives the analyser no more chances than before. A recording a little shorter than 9 s would now fit one more window.
