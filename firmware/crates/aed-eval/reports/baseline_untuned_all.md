# Rhythm analyser on recorded heart rhythms

- **Date (UTC):** 2026-11-27
- **Analyser evaluated:** `aed-rhythm` at git commit `b9a79e7` (**uncommitted changes in the analyser**: M firmware/crates/aed-rhythm/src/analyzer.rs;  M firmware/crates/aed-rhythm/src/features.rs;  M firmware/crates/aed-rhythm/src/lib.rs;  M firmware/crates/aed-rhythm/src/settings.rs; 16 uncommitted path(s) elsewhere in the repository)
- **Data:** PhysioNet cudb, vfdb, mitdb (public Holter / monitoring recordings), read from `data/physionet`
- **Records evaluated:** 105

**Untuned analyser, all 105 records, regenerated** with `aed-eval --untuned` (thresholds as at commit 9cf11b6; the clipping and flatline checks added since are switched off). Same figures as the first run (reports/baseline_untuned_9cf11b6.md); this run adds the countdown and time-to-decision tables.

**What these numbers are not.** They are not a claim about a real device. The recordings are:

- few patients (35 + 22 + 47), so thousands of segments are really a few dozen independent people; the confidence limits below treat segments as independent and are therefore too optimistic;
- in-hospital monitoring and ambulatory (Holter) recordings from the 1970s-80s, not out-of-hospital cardiac arrest recorded through defibrillator pads;
- free of CPR artefact; no children; with recorder bandwidths and leads unlike pad ECG;
- of uncertain amplitude calibration (cudb's gain is "nominal" only; see the amplitude section), which matters because the analyser has absolute microvolt rules;
- annotated with varying care: mitdb by two cardiologists beat by beat; vfdb with rhythm labels only; cudb with "preliminary" labels, approximate VF onsets, and every beat labelled normal even when it is not.

Public-database results are known to overstate field performance (research notes, section 2). Nothing here is safe or ready for use on a person. See SAFETY.md.

## How it was scored

- **Device-realistic analysis.** For each segment a fresh analyser (as after the reset at the start of every analysis) is fed one analysis worth of signal. The first final answer decides: `Shockable` → shock; `NonShockable` → no shock; neither by the time limit → *inconclusive* ("unable to analyse", no shock). "Not sure" does not end an analysis (D-27). Three analysis lengths: **7 s** (the shortest that can end in a shock: 1 s settling + 4 s window + 2 more windows, since at least 3 windows must say shockable), **8 s** (the analysis-time requirement mentioned in the configuration), and **12 s** (the device's actual limit, D-27; the headline).
- **Scoring of inconclusive.** For sensitivity it is a miss. For specificity it is a correct "no shock". It is also counted on its own everywhere, because a device that says "not sure" to everything would score perfect specificity and be useless.
- **Segments** are consecutive, non-overlapping, and only taken where the ground truth is one class throughout, at least 3 s from any annotated rhythm change, at least 1 s from any stretch marked noisy for that channel, and at least 1 s from isolated-artifact or tape-slippage marks.
- **Channels.** Channel 0 and every other channel are scored separately; nothing is picked per record. Pooled tables are shown per channel.
- **Coarse vs fine VF** uses an amplitude measured independently of the analyser: median over 1-second pieces of the peak-to-peak value after subtracting baseline drift (a 2-second triangular moving average); fine VF is below 200 µV. **Rapid VT** is VT with a rate, from the beat annotations in the segment, of at least 150/min (own choice; the AHA leaves the rate to the manufacturer). vfdb has no beat annotations, so its VT goes to "VT, rate unknown"; VT is also reported with no rate criterion.
- **Confidence limits** are one-sided 90 % lower limits, exact (Clopper–Pearson), with Wilson alongside. The AHA goals, lower limits and minimum sample sizes (marked *) are from the research notes' reproduction of the AHA 1997 table and are UNVERIFIED against the original. "meets" needs both the result and the lower limit at or above the AHA figures.

## Data checks

| Database | records | channels | rate (Hz) | checksums OK | first sample = header | samples at recorder limit | clipped to i16 | gains (ADC/mV) | unknown labels |
|---|---|---|---|---|---|---|---|---|---|
| cudb | 35 | 35 | 250 | 35/35 | 35/35 | 66085 (1.5%) | 0 | 400 | none |
| vfdb | 22 | 44 | 250 | 44/44 | 44/44 | 0 (0.0%) | 0 | 200 | none |
| mitdb | 48 | 96 | 360 | 96/96 | 96/96 | 1880 (0.0%) | 0 | 200 | none |

Every record listed in each database's RECORDS file loaded without error.

"Samples at recorder limit" are values at the very end of the original recorder's range: the recording itself was clipped there (the original analogue chain saturated). The analyser cannot see this: its own saturation rule looks for values beyond ±32 000 µV, which these recordings (range about ±5 mV for cudb and mitdb, ±10 mV for vfdb) can never reach. No sample needed clipping to fit the analyser's 16-bit input.

10 signal-quality (`~`) marks in cudb carry the text `(N`; the text is ignored and only the quality change is used.

## Headline: device-realistic analysis (12 s limit)

Each cell: result (exact 90 % lower limit, segments, records); ✗ = misses the AHA goal or its lower limit. Inconclusive = share of all scored segments ending "unable to analyse".

| Group | Coarse VF sensitivity (AHA ≥90%, LCL 87%) | Rapid VT sensitivity (≥75%, 67%) | NSR specificity (≥99%, 97%) | Other non-shockable specificity (≥95%, 88%) | Asystole specificity (≥95%, 92%) | Inconclusive |
|---|---|---|---|---|---|---|
| cudb channel 0 | 56.1% (LCL 51.6%, n=221, 26 rec) ✗ | no data | 100.0% (LCL 93.5%, n=34, 3 rec) ✗ | 93.1% (LCL 82.7%, n=29, 2 rec) ✗ | no data | 17.7% (n=1072) |
| vfdb channel 0 | 69.1% (LCL 64.2%, n=178, 4 rec) ✗ | no data | 99.8% (LCL 99.6%, n=1317, 15 rec) | 99.9% (LCL 99.6%, n=996, 13 rec) | 96.4% (LCL 90.8%, n=56, 4 rec) ✗ | 16.6% (n=3022) |
| mitdb channel 0 | no data | no data | 100.0% (LCL 100.0%, n=4742, 42 rec) | 100.0% (LCL 99.8%, n=1433, 24 rec) | no data | 1.1% (n=6188) |
| **pooled, channel 0** | 61.9% (LCL 58.6%, n=399, 30 rec) ✗ | no data | 100.0% (LCL 99.9%, n=6093, 60 rec) | 99.9% (LCL 99.7%, n=2458, 39 rec) | 96.4% (LCL 90.8%, n=56, 4 rec) ✗ | 7.4% (n=10282) |
| vfdb channel 1 | 78.5% (LCL 74.1%, n=177, 4 rec) ✗ | no data | 99.2% (LCL 98.7%, n=1317, 15 rec) | 99.9% (LCL 99.6%, n=996, 13 rec) | 91.1% (LCL 84.1%, n=56, 4 rec) ✗ | 12.6% (n=3022) |
| mitdb channel 1 | no data | no data | 100.0% (LCL 99.9%, n=4545, 42 rec) | 100.0% (LCL 99.8%, n=1410, 24 rec) | no data | 2.3% (n=5966) |
| **pooled, channel 1** | 78.5% (LCL 74.1%, n=177, 4 rec) ✗ | no data | 99.8% (LCL 99.7%, n=5862, 57 rec) | 100.0% (LCL 99.8%, n=2406, 37 rec) | 91.1% (LCL 84.1%, n=56, 4 rec) ✗ | 5.8% (n=8988) |

## Time to decision, and shocks lost in the countdown (12 s analysis)

Every second of analysis is hands-off time. "Decided by" = share of segments with a final answer (shock or no shock) by that time after the analysis started; the rest are still analysing. After "shock advised" the rescue sequence counts down for 3 s and abandons the shock on any answer other than `Shockable` (aed-core); "cancelled in countdown" counts shocks the analyser itself would have called off that way, measured by carrying on into the signal that follows (where the same rhythm continues). "Delivered" = shocks not cancelled, over all segments; a shock whose countdown could not be checked counts as delivered.

| Group | Class | n | decided by 7 s | 8 s | 10 s | 12 s | median time to answer (s) | 90th pct (s) | shocks | cancelled in countdown | countdown not checkable | delivered |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| channel 0 | Coarse VF | 399 | 50.9% | 57.9% | 69.2% | 76.4% | 7 | 10 | 247 | 88 | 1 | 39.8% |
| channel 0 | Ventricular flutter | 32 | 53.1% | 59.4% | 71.9% | 81.2% | 7 | 10 | 22 | 5 | 0 | 53.1% |
| channel 0 | VT, rate unknown | 389 | 38.6% | 44.0% | 54.8% | 62.0% | 7 | 11 | 184 | 53 | 0 | 33.7% |
| channel 0 | Normal sinus rhythm | 6093 | 94.4% | 94.9% | 95.7% | 96.2% | 5 | 5 | 2 | 2 | 0 | 0.0% |
| channel 0 | Other non-shockable | 2458 | 93.7% | 94.3% | 94.9% | 95.3% | 5 | 5 | 3 | 2 | 0 | 0.0% |
| channel 0 | Asystole | 56 | 30.4% | 35.7% | 42.9% | 46.4% | 7 | 10 | 2 | 2 | 0 | 0.0% |
| channel 1 | Coarse VF | 177 | 68.9% | 75.1% | 83.6% | 92.7% | 7 | 10 | 139 | 40 | 0 | 55.9% |
| channel 1 | Ventricular flutter | 32 | 62.5% | 62.5% | 81.2% | 90.6% | 7 | 10 | 25 | 2 | 0 | 71.9% |
| channel 1 | VT, rate unknown | 389 | 47.3% | 50.6% | 59.1% | 64.8% | 7 | 10 | 154 | 52 | 0 | 26.2% |
| channel 1 | Normal sinus rhythm | 5862 | 92.7% | 93.4% | 94.5% | 95.4% | 5 | 5 | 11 | 6 | 0 | 0.1% |
| channel 1 | Other non-shockable | 2406 | 90.6% | 93.4% | 95.1% | 96.8% | 5 | 7 | 1 | 0 | 0 | 0.0% |
| channel 1 | Asystole | 56 | 58.9% | 58.9% | 67.9% | 69.6% | 6 | 9 | 5 | 1 | 0 | 7.1% |

## Segment-level results

Report-only buckets show the share that ended in a shock. "cudb non-VF, rhythm not given" is cudb time outside the annotated VF episodes for which no rhythm label exists; it may contain VT (cudb is a tachyarrhythmia database) and is therefore not counted as a non-shockable class.

### 12 s analysis

No "Rapid VT" row appears: the only VT with beat annotations (mitdb, a few cudb labels) comes in runs too short to hold a whole segment plus the 3 s guard margins, and the few that fit are slow. vfdb's VT has no beat annotations, so its rate is unknown. The rapid-VT goal therefore cannot be tested on these databases with annotation-based rates; see the rate-estimate table in the failure analysis for a rough split.

**cudb channel 0** (1072 segments from 32 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 221 | 26 | 124 | 42 | 55 | 56.1% | 51.6% | 51.8% | ≥90% (87%) | 200 | **FAILS** |
| Normal sinus rhythm | specificity | 34 | 3 | 0 | 33 | 1 | 100.0% | 93.5% | 95.4% | ≥99% (97%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 29 | 2 | 2 | 26 | 1 | 93.1% | 82.7% | 84.5% | ≥95% (88%) | 30 | **FAILS** (n<30) |
| cudb non-VF, rhythm not given | shock rate | 788 | 32 | 1 | 654 | 133 | 0.1% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 221 | 26 | 124 | 42 | 55 | 56.1% | 51.6% | 51.8% | – | – | – |

Inconclusive ("unable to analyse") over all 1072 segments: **17.7%**.

**vfdb channel 0** (3022 segments from 22 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 178 | 4 | 123 | 16 | 39 | 69.1% | 64.2% | 64.5% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 25 | 5 | 17 | 4 | 4 | 68.0% | 53.3% | 55.3% | report only | – | report only |
| VT, rate unknown | shock rate | 389 | 15 | 184 | 57 | 148 | 47.3% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 1317 | 15 | 2 | 1142 | 173 | 99.8% | 99.6% | 99.6% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 2 | 24 | 30 | 96.4% | 90.8% | 91.7% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 996 | 13 | 1 | 888 | 107 | 99.9% | 99.6% | 99.7% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 61 | 3 | 0 | 60 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 178 | 4 | 123 | 16 | 39 | 69.1% | 64.2% | 64.5% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 389 | 15 | 184 | 57 | 148 | 47.3% | 43.9% | 44.1% | – | – | – |

Inconclusive ("unable to analyse") over all 3022 segments: **16.6%**.

**vfdb channel 1** (3022 segments from 22 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 177 | 4 | 139 | 25 | 13 | 78.5% | 74.1% | 74.3% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 1 | 0 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 25 | 5 | 18 | 4 | 3 | 72.0% | 57.4% | 59.4% | report only | – | report only |
| VT, rate unknown | shock rate | 389 | 15 | 154 | 98 | 137 | 39.6% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 1317 | 15 | 11 | 1116 | 190 | 99.2% | 98.7% | 98.8% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 5 | 34 | 17 | 91.1% | 84.1% | 84.9% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 996 | 13 | 1 | 975 | 20 | 99.9% | 99.6% | 99.7% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 61 | 3 | 0 | 59 | 2 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 178 | 4 | 139 | 26 | 13 | 78.1% | 73.6% | 73.9% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 389 | 15 | 154 | 98 | 137 | 39.6% | 36.3% | 36.5% | – | – | – |

Inconclusive ("unable to analyse") over all 3022 segments: **12.6%**.

**mitdb channel 0** (6188 segments from 48 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 7 | 1 | 5 | 0 | 2 | 71.4% | 40.4% | 47.2% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 6 | 1 | 0 | 6 | 0 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 4742 | 42 | 0 | 4682 | 60 | 100.0% | 100.0% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 1433 | 24 | 0 | 1425 | 8 | 100.0% | 99.8% | 99.9% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 6 | 1 | 0 | 6 | 0 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 6188 segments: **1.1%**.

**mitdb channel 1** (5966 segments from 48 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 7 | 1 | 7 | 0 | 0 | 100.0% | 72.0% | 81.0% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 4 | 1 | 0 | 4 | 0 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 4545 | 42 | 0 | 4465 | 80 | 100.0% | 99.9% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 1410 | 24 | 0 | 1353 | 57 | 100.0% | 99.8% | 99.9% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 4 | 1 | 0 | 4 | 0 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 5966 segments: **2.3%**.

**Pooled, channel 0** (10282 segments from 102 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 399 | 30 | 247 | 58 | 94 | 61.9% | 58.6% | 58.7% | ≥90% (87%) | 200 | **FAILS** |
| Ventricular flutter | sensitivity | 32 | 6 | 22 | 4 | 6 | 68.8% | 56.0% | 57.6% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 6 | 1 | 0 | 6 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 389 | 15 | 184 | 57 | 148 | 47.3% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 6093 | 60 | 2 | 5857 | 234 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 2 | 24 | 30 | 96.4% | 90.8% | 91.7% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 2458 | 39 | 3 | 2339 | 116 | 99.9% | 99.7% | 99.7% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 788 | 32 | 1 | 654 | 133 | 0.1% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 61 | 3 | 0 | 60 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 399 | 30 | 247 | 58 | 94 | 61.9% | 58.6% | 58.7% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 395 | 16 | 184 | 63 | 148 | 46.6% | 43.3% | 43.4% | – | – | – |

Inconclusive ("unable to analyse") over all 10282 segments: **7.4%**.

**Pooled, channel 1 (vfdb + mitdb)** (8988 segments from 70 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 177 | 4 | 139 | 25 | 13 | 78.5% | 74.1% | 74.3% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 1 | 0 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 32 | 6 | 25 | 4 | 3 | 78.1% | 66.0% | 67.5% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 4 | 1 | 0 | 4 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 389 | 15 | 154 | 98 | 137 | 39.6% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 5862 | 57 | 11 | 5581 | 270 | 99.8% | 99.7% | 99.7% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 5 | 34 | 17 | 91.1% | 84.1% | 84.9% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 2406 | 37 | 1 | 2328 | 77 | 100.0% | 99.8% | 99.9% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 61 | 3 | 0 | 59 | 2 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 178 | 4 | 139 | 26 | 13 | 78.1% | 73.6% | 73.9% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 393 | 16 | 154 | 102 | 137 | 39.2% | 35.9% | 36.1% | – | – | – |

Inconclusive ("unable to analyse") over all 8988 segments: **5.8%**.

**Non-shockable rhythms by label, pooled channel 0, 12 s**

| Rhythm label | n | records | shock (false positive) | no shock | inconcl. | specificity | 90% LCL exact |
|---|---|---|---|---|---|---|---|
| AB (atrial bigeminy) | 5 | 1 | 0 | 5 | 0 | 100.0% | 63.1% |
| AFIB | 845 | 12 | 2 | 755 | 88 | 99.8% | 99.4% |
| AFL | 29 | 3 | 0 | 29 | 0 | 100.0% | 92.4% |
| ASYS | 56 | 4 | 2 | 24 | 30 | 96.4% | 90.8% |
| B (ventricular bigeminy) | 124 | 7 | 0 | 107 | 17 | 100.0% | 98.2% |
| BI (1st-degree block) | 332 | 3 | 0 | 331 | 1 | 100.0% | 99.3% |
| BII (2nd-degree block) | 52 | 1 | 0 | 52 | 0 | 100.0% | 95.7% |
| IVR (idioventricular) | 9 | 2 | 0 | 8 | 1 | 100.0% | 77.4% |
| NOD (junctional) | 142 | 4 | 0 | 139 | 3 | 100.0% | 98.4% |
| NSR | 6093 | 60 | 2 | 5857 | 234 | 100.0% | 99.9% |
| PACED | 611 | 5 | 0 | 611 | 0 | 100.0% | 99.6% |
| PREX (pre-excitation) | 16 | 1 | 0 | 16 | 0 | 100.0% | 86.6% |
| SBR | 132 | 2 | 0 | 132 | 0 | 100.0% | 98.3% |
| SVTA | 126 | 6 | 1 | 119 | 6 | 99.2% | 96.9% |
| T (ventricular trigeminy) | 33 | 3 | 0 | 33 | 0 | 100.0% | 93.3% |
| VER (ventricular escape) | 2 | 1 | 0 | 2 | 0 | 100.0% | 31.6% |

**Non-shockable rhythms by label, pooled channel 1, 12 s**

| Rhythm label | n | records | shock (false positive) | no shock | inconcl. | specificity | 90% LCL exact |
|---|---|---|---|---|---|---|---|
| AB (atrial bigeminy) | 5 | 1 | 0 | 5 | 0 | 100.0% | 63.1% |
| AFIB | 811 | 10 | 0 | 810 | 1 | 100.0% | 99.7% |
| AFL | 36 | 3 | 0 | 35 | 1 | 100.0% | 93.8% |
| ASYS | 56 | 4 | 5 | 34 | 17 | 91.1% | 84.1% |
| B (ventricular bigeminy) | 100 | 7 | 0 | 87 | 13 | 100.0% | 97.7% |
| BI (1st-degree block) | 332 | 3 | 0 | 330 | 2 | 100.0% | 99.3% |
| BII (2nd-degree block) | 52 | 1 | 0 | 52 | 0 | 100.0% | 95.7% |
| IVR (idioventricular) | 9 | 2 | 0 | 7 | 2 | 100.0% | 77.4% |
| NOD (junctional) | 142 | 4 | 0 | 138 | 4 | 100.0% | 98.4% |
| NSR | 5862 | 57 | 11 | 5581 | 270 | 99.8% | 99.7% |
| PACED | 610 | 5 | 0 | 560 | 50 | 100.0% | 99.6% |
| PREX (pre-excitation) | 15 | 1 | 0 | 15 | 0 | 100.0% | 85.8% |
| SBR | 133 | 2 | 0 | 133 | 0 | 100.0% | 98.3% |
| SVTA | 126 | 6 | 1 | 122 | 3 | 99.2% | 96.9% |
| T (ventricular trigeminy) | 33 | 3 | 0 | 32 | 1 | 100.0% | 93.3% |
| VER (ventricular escape) | 2 | 1 | 0 | 2 | 0 | 100.0% | 31.6% |

### 8 s analysis

**cudb channel 0** (1640 segments from 32 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 341 | 27 | 140 | 47 | 154 | 41.1% | 37.5% | 37.7% | ≥90% (87%) | 200 | **FAILS** |
| Normal sinus rhythm | specificity | 52 | 3 | 0 | 45 | 7 | 100.0% | 95.7% | 96.9% | ≥99% (97%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 45 | 2 | 2 | 41 | 2 | 95.6% | 88.6% | 89.8% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 1202 | 32 | 1 | 934 | 267 | 0.1% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 341 | 27 | 140 | 47 | 154 | 41.1% | 37.5% | 37.7% | – | – | – |

Inconclusive ("unable to analyse") over all 1640 segments: **26.2%**.

**vfdb channel 0** (4606 segments from 22 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 273 | 4 | 149 | 21 | 103 | 54.6% | 50.5% | 50.7% | ≥90% (87%) | 200 | **FAILS** |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 0 | 1 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 40 | 6 | 20 | 4 | 16 | 50.0% | 38.8% | 40.1% | report only | – | report only |
| VT, rate unknown | shock rate | 594 | 16 | 218 | 68 | 308 | 36.7% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 2011 | 15 | 1 | 1661 | 349 | 100.0% | 99.8% | 99.8% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 86 | 4 | 3 | 27 | 56 | 96.5% | 92.4% | 93.0% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 1506 | 13 | 0 | 1316 | 190 | 100.0% | 99.8% | 99.9% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 95 | 3 | 0 | 93 | 2 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 274 | 4 | 149 | 21 | 104 | 54.4% | 50.3% | 50.5% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 594 | 16 | 218 | 68 | 308 | 36.7% | 34.1% | 34.2% | – | – | – |

Inconclusive ("unable to analyse") over all 4606 segments: **22.3%**.

**mitdb channel 0** (9451 segments from 48 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 11 | 1 | 4 | 1 | 6 | 36.4% | 16.9% | 20.7% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 10 | 1 | 0 | 10 | 0 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 7215 | 42 | 0 | 7089 | 126 | 100.0% | 100.0% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 2215 | 24 | 0 | 2195 | 20 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 10 | 1 | 0 | 10 | 0 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 9451 segments: **1.6%**.

**Pooled, channel 0** (15697 segments from 102 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 614 | 31 | 289 | 68 | 257 | 47.1% | 44.4% | 44.5% | ≥90% (87%) | 200 | **FAILS** |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 0 | 1 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 51 | 7 | 24 | 5 | 22 | 47.1% | 37.4% | 38.3% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 10 | 1 | 0 | 10 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 594 | 16 | 218 | 68 | 308 | 36.7% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 9278 | 60 | 1 | 8795 | 482 | 100.0% | 100.0% | 100.0% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 86 | 4 | 3 | 27 | 56 | 96.5% | 92.4% | 93.0% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 3766 | 39 | 2 | 3552 | 212 | 99.9% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 1202 | 32 | 1 | 934 | 267 | 0.1% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 95 | 3 | 0 | 93 | 2 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 615 | 31 | 289 | 68 | 258 | 47.0% | 44.3% | 44.4% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 604 | 17 | 218 | 78 | 308 | 36.1% | 33.5% | 33.6% | – | – | – |

Inconclusive ("unable to analyse") over all 15697 segments: **10.2%**.

**Pooled, channel 1 (vfdb + mitdb)** (13719 segments from 70 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 272 | 4 | 170 | 31 | 71 | 62.5% | 58.5% | 58.7% | ≥90% (87%) | 200 | **FAILS** |
| Fine VF (<200 µV) | shock rate | 2 | 1 | 0 | 1 | 1 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 51 | 7 | 32 | 3 | 16 | 62.7% | 52.8% | 53.8% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 8 | 1 | 0 | 8 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 594 | 16 | 187 | 113 | 294 | 31.5% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 8923 | 57 | 7 | 8330 | 586 | 99.9% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 86 | 4 | 9 | 42 | 35 | 89.5% | 84.0% | 84.5% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 3688 | 37 | 1 | 3422 | 265 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 95 | 3 | 0 | 76 | 19 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 274 | 4 | 170 | 32 | 72 | 62.0% | 58.0% | 58.2% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 602 | 17 | 187 | 121 | 294 | 31.1% | 28.6% | 28.7% | – | – | – |

Inconclusive ("unable to analyse") over all 13719 segments: **9.4%**.

### 7 s analysis

**cudb channel 0** (1875 segments from 32 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 388 | 27 | 141 | 44 | 203 | 36.3% | 33.1% | 33.3% | ≥90% (87%) | 200 | **FAILS** |
| Normal sinus rhythm | specificity | 61 | 3 | 0 | 49 | 12 | 100.0% | 96.3% | 97.4% | ≥99% (97%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 51 | 2 | 0 | 46 | 5 | 100.0% | 95.6% | 96.9% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 1375 | 32 | 1 | 1043 | 331 | 0.1% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 388 | 27 | 141 | 44 | 203 | 36.3% | 33.1% | 33.3% | – | – | – |

Inconclusive ("unable to analyse") over all 1875 segments: **29.4%**.

**vfdb channel 0** (5276 segments from 22 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 314 | 4 | 143 | 21 | 150 | 45.5% | 41.8% | 42.0% | ≥90% (87%) | 200 | **FAILS** |
| Ventricular flutter | sensitivity | 46 | 6 | 22 | 5 | 19 | 47.8% | 37.5% | 38.6% | report only | – | report only |
| VT, rate unknown | shock rate | 681 | 16 | 223 | 78 | 380 | 32.7% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 2302 | 15 | 1 | 1867 | 434 | 100.0% | 99.8% | 99.9% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 98 | 4 | 3 | 24 | 71 | 96.9% | 93.3% | 93.8% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 1724 | 13 | 0 | 1495 | 229 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 111 | 3 | 0 | 102 | 9 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 314 | 4 | 143 | 21 | 150 | 45.5% | 41.8% | 42.0% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 681 | 16 | 223 | 78 | 380 | 32.7% | 30.4% | 30.5% | – | – | – |

Inconclusive ("unable to analyse") over all 5276 segments: **24.5%**.

**mitdb channel 0** (10850 segments from 48 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 14 | 1 | 6 | 1 | 7 | 42.9% | 24.3% | 27.6% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 11 | 1 | 0 | 9 | 2 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 8274 | 42 | 0 | 8112 | 162 | 100.0% | 100.0% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 2551 | 24 | 0 | 2517 | 34 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 11 | 1 | 0 | 9 | 2 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 10850 segments: **1.9%**.

**Pooled, channel 0** (18001 segments from 102 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 702 | 31 | 284 | 65 | 353 | 40.5% | 38.0% | 38.1% | ≥90% (87%) | 200 | **FAILS** |
| Ventricular flutter | sensitivity | 60 | 7 | 28 | 6 | 26 | 46.7% | 37.8% | 38.6% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 11 | 1 | 0 | 9 | 2 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 681 | 16 | 223 | 78 | 380 | 32.7% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 10637 | 60 | 1 | 10028 | 608 | 100.0% | 100.0% | 100.0% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 98 | 4 | 3 | 24 | 71 | 96.9% | 93.3% | 93.8% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 4326 | 39 | 0 | 4058 | 268 | 100.0% | 99.9% | 100.0% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 1375 | 32 | 1 | 1043 | 331 | 0.1% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 111 | 3 | 0 | 102 | 9 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 702 | 31 | 284 | 65 | 353 | 40.5% | 38.0% | 38.1% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 692 | 17 | 223 | 87 | 382 | 32.2% | 29.9% | 30.0% | – | – | – |

Inconclusive ("unable to analyse") over all 18001 segments: **11.4%**.

**Pooled, channel 1 (vfdb + mitdb)** (15749 segments from 70 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 312 | 4 | 169 | 31 | 112 | 54.2% | 50.4% | 50.5% | ≥90% (87%) | 200 | **FAILS** |
| Fine VF (<200 µV) | shock rate | 2 | 1 | 0 | 1 | 1 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 60 | 7 | 35 | 4 | 21 | 58.3% | 49.2% | 50.1% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 8 | 1 | 0 | 8 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 681 | 16 | 190 | 110 | 381 | 27.9% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 10237 | 57 | 13 | 9494 | 730 | 99.9% | 99.8% | 99.8% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 98 | 4 | 9 | 41 | 48 | 90.8% | 85.9% | 86.4% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 4240 | 37 | 1 | 3840 | 399 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 111 | 3 | 0 | 81 | 30 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 314 | 4 | 169 | 32 | 113 | 53.8% | 50.0% | 50.2% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 689 | 17 | 190 | 118 | 381 | 27.6% | 25.4% | 25.4% | – | – | – |

Inconclusive ("unable to analyse") over all 15749 segments: **10.9%**.

## Per record (12 s analysis, every channel)

So that one bad patient is visible. VF = coarse VF; "VT/VFL shock" = shocks / segments over VT of any rate and flutter; FP = false shocks on NSR or other non-shockable rhythms. Flags: **VF<50%** (≥5 coarse-VF segments and under half shocked), **FP** (any false shock on an AHA non-shockable class), **INC>50%** (over half of all segments inconclusive).

| Record | ch | lead | coarse VF n | VF Se | VT/VFL shock | NSR n | NSR Sp | other n | other Sp | FP | inconcl. | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cudb/cu01 | 0 | ECG | 24 | 95.8% | – | 0 | – | 0 | – | 0 | 2.4% |  |
| cudb/cu02 | 0 | ECG | 0 | – | – | 19 | 100.0% | 0 | – | 0 | 15.2% |  |
| cudb/cu03 | 0 | ECG | 3 | 100.0% | – | 0 | – | 0 | – | 0 | 61.5% | **INC>50%** |
| cudb/cu04 | 0 | ECG | 20 | 65.0% | – | 0 | – | 0 | – | 0 | 5.7% |  |
| cudb/cu05 | 0 | ECG | 6 | 100.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu06 | 0 | ECG | 8 | 87.5% | – | 0 | – | 0 | – | 0 | 5.6% |  |
| cudb/cu07 | 0 | ECG | 26 | 50.0% | – | 0 | – | 0 | – | 0 | 40.0% |  |
| cudb/cu09 | 0 | ECG | 4 | 50.0% | – | 9 | 100.0% | 5 | 60.0% | 2 | 6.1% | **FP** |
| cudb/cu10 | 0 | ECG | 15 | 53.3% | – | 0 | – | 0 | – | 0 | 7.5% |  |
| cudb/cu11 | 0 | ECG | 10 | 30.0% | – | 0 | – | 0 | – | 0 | 77.5% | **VF<50%** **INC>50%** |
| cudb/cu12 | 0 | ECG | 15 | 40.0% | – | 0 | – | 0 | – | 0 | 30.8% | **VF<50%** |
| cudb/cu13 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu14 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 12.5% |  |
| cudb/cu15 | 0 | ECG | 8 | 0.0% | – | 0 | – | 0 | – | 0 | 58.5% | **VF<50%** **INC>50%** |
| cudb/cu16 | 0 | ECG | 7 | 100.0% | – | 6 | 100.0% | 0 | – | 0 | 0.0% |  |
| cudb/cu17 | 0 | ECG | 2 | 0.0% | – | 0 | – | 0 | – | 0 | 5.4% |  |
| cudb/cu18 | 0 | ECG | 1 | 100.0% | – | 0 | – | 24 | 100.0% | 0 | 0.0% |  |
| cudb/cu19 | 0 | ECG | 2 | 100.0% | – | 0 | – | 0 | – | 0 | 45.7% |  |
| cudb/cu20 | 0 | ECG | 21 | 4.8% | – | 0 | – | 0 | – | 0 | 40.0% | **VF<50%** |
| cudb/cu21 | 0 | ECG | 2 | 50.0% | – | 0 | – | 0 | – | 0 | 25.0% |  |
| cudb/cu22 | 0 | ECG | 8 | 100.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu23 | 0 | ECG | 8 | 62.5% | – | 0 | – | 0 | – | 0 | 7.5% |  |
| cudb/cu24 | 0 | ECG | 5 | 60.0% | – | 0 | – | 0 | – | 0 | 5.9% |  |
| cudb/cu25 | 0 | ECG | 2 | 0.0% | – | 0 | – | 0 | – | 0 | 2.8% |  |
| cudb/cu26 | 0 | ECG | 4 | 75.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu28 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 20.0% |  |
| cudb/cu29 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu30 | 0 | ECG | 8 | 12.5% | – | 0 | – | 0 | – | 0 | 33.3% | **VF<50%** |
| cudb/cu31 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 5.0% |  |
| cudb/cu32 | 0 | ECG | 3 | 33.3% | – | 0 | – | 0 | – | 0 | 23.1% |  |
| cudb/cu33 | 0 | ECG | 6 | 66.7% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu34 | 0 | ECG | 3 | 100.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| vfdb/418 | 0 | ECG | 0 | – | – | 116 | 98.3% | 0 | – | 2 | 69.8% | **FP** **INC>50%** |
| vfdb/418 | 1 | ECG | 0 | – | – | 116 | 91.4% | 0 | – | 10 | 66.4% | **FP** **INC>50%** |
| vfdb/419 | 0 | ECG | 0 | – | 5/9 | 79 | 100.0% | 0 | – | 0 | 45.5% |  |
| vfdb/419 | 1 | ECG | 0 | – | 6/9 | 79 | 98.7% | 0 | – | 1 | 54.5% | **FP** **INC>50%** |
| vfdb/420 | 0 | ECG | 0 | – | 15/20 | 108 | 100.0% | 0 | – | 0 | 3.9% |  |
| vfdb/420 | 1 | ECG | 0 | – | 13/20 | 108 | 100.0% | 0 | – | 0 | 1.6% |  |
| vfdb/421 | 0 | ECG | 0 | – | 0/12 | 84 | 100.0% | 0 | – | 0 | 2.1% |  |
| vfdb/421 | 1 | ECG | 0 | – | 8/12 | 84 | 100.0% | 0 | – | 0 | 3.1% |  |
| vfdb/422 | 0 | ECG | 40 | 37.5% | 12/15 | 87 | 100.0% | 0 | – | 0 | 18.3% | **VF<50%** |
| vfdb/422 | 1 | ECG | 40 | 72.5% | 12/15 | 87 | 100.0% | 0 | – | 0 | 1.4% |  |
| vfdb/423 | 0 | ECG | 0 | – | 30/37 | 82 | 100.0% | 1 | 100.0% | 0 | 6.7% |  |
| vfdb/423 | 1 | ECG | 0 | – | 19/37 | 82 | 100.0% | 1 | 100.0% | 0 | 5.8% |  |
| vfdb/424 | 0 | ECG | 8 | 100.0% | 1/4 | 121 | 100.0% | 10 | 100.0% | 0 | 2.1% |  |
| vfdb/424 | 1 | ECG | 8 | 100.0% | 1/4 | 121 | 100.0% | 10 | 100.0% | 0 | 4.9% |  |
| vfdb/425 | 0 | ECG | 0 | – | 1/1 | 112 | 100.0% | 25 | 100.0% | 0 | 10.1% |  |
| vfdb/425 | 1 | ECG | 0 | – | 1/1 | 112 | 100.0% | 25 | 100.0% | 0 | 11.6% |  |
| vfdb/426 | 0 | ECG | 50 | 80.0% | 1/1 | 66 | 100.0% | 4 | 75.0% | 1 | 5.0% | **FP** |
| vfdb/426 | 1 | ECG | 49 | 77.6% | 1/1 | 66 | 100.0% | 4 | 75.0% | 1 | 2.5% | **FP** |
| vfdb/427 | 0 | ECG | 0 | – | 44/73 | 53 | 100.0% | 46 | 100.0% | 0 | 23.3% |  |
| vfdb/427 | 1 | ECG | 0 | – | 21/73 | 53 | 100.0% | 46 | 100.0% | 0 | 25.6% |  |
| vfdb/428 | 0 | ECG | 0 | – | 2/2 | 0 | – | 165 | 100.0% | 0 | 0.0% |  |
| vfdb/428 | 1 | ECG | 0 | – | 2/2 | 0 | – | 165 | 100.0% | 0 | 0.6% |  |
| vfdb/429 | 0 | ECG | 0 | – | 6/6 | 0 | – | 140 | 100.0% | 0 | 0.7% |  |
| vfdb/429 | 1 | ECG | 0 | – | 6/6 | 0 | – | 140 | 100.0% | 0 | 0.7% |  |
| vfdb/430 | 0 | ECG | 80 | 75.0% | 22/30 | 0 | – | 31 | 100.0% | 0 | 14.3% |  |
| vfdb/430 | 1 | ECG | 80 | 80.0% | 21/30 | 0 | – | 31 | 100.0% | 0 | 13.6% |  |
| vfdb/602 | 0 | ECG | 0 | – | 1/31 | 3 | 100.0% | 117 | 100.0% | 0 | 5.3% |  |
| vfdb/602 | 1 | ECG | 0 | – | 2/31 | 3 | 100.0% | 117 | 100.0% | 0 | 10.6% |  |
| vfdb/605 | 0 | ECG | 0 | – | 1/1 | 0 | – | 0 | – | 0 | 0.0% |  |
| vfdb/605 | 1 | ECG | 0 | – | 1/1 | 0 | – | 0 | – | 0 | 0.0% |  |
| vfdb/607 | 0 | ECG | 0 | – | 8/10 | 0 | – | 156 | 98.7% | 2 | 4.2% | **FP** |
| vfdb/607 | 1 | ECG | 0 | – | 8/10 | 0 | – | 156 | 96.8% | 5 | 2.4% | **FP** |
| vfdb/609 | 0 | ECG | 0 | – | 1/1 | 69 | 100.0% | 84 | 100.0% | 0 | 46.1% |  |
| vfdb/609 | 1 | ECG | 0 | – | 1/1 | 69 | 100.0% | 84 | 100.0% | 0 | 0.0% |  |
| vfdb/610 | 0 | ECG | 0 | – | – | 101 | 100.0% | 0 | – | 0 | 0.0% |  |
| vfdb/610 | 1 | ECG | 0 | – | – | 101 | 100.0% | 0 | – | 0 | 0.7% |  |
| vfdb/611 | 0 | ECG | 0 | – | 0/74 | 0 | – | 99 | 100.0% | 0 | 42.8% |  |
| vfdb/611 | 1 | ECG | 0 | – | 0/74 | 0 | – | 99 | 100.0% | 0 | 42.8% |  |
| vfdb/612 | 0 | ECG | 0 | – | 24/32 | 141 | 100.0% | 0 | – | 0 | 28.9% |  |
| vfdb/612 | 1 | ECG | 0 | – | 24/32 | 141 | 100.0% | 0 | – | 0 | 26.0% |  |
| vfdb/614 | 0 | ECG | 0 | – | 27/55 | 95 | 100.0% | 7 | 100.0% | 0 | 18.5% |  |
| vfdb/614 | 1 | ECG | 0 | – | 25/55 | 95 | 100.0% | 7 | 100.0% | 0 | 7.0% |  |
| vfdb/615 | 0 | ECG | 0 | – | – | 0 | – | 167 | 100.0% | 0 | 6.6% |  |
| vfdb/615 | 1 | ECG | 0 | – | – | 0 | – | 167 | 100.0% | 0 | 0.0% |  |
| mitdb/100 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/100 | 1 | V5 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/101 | 0 | MLII | 0 | – | – | 147 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/101 | 1 | V1 | 0 | – | – | 147 | 100.0% | 0 | – | 0 | 51.0% | **INC>50%** |
| mitdb/102 | 0 | V5 | 0 | – | – | 5 | 100.0% | 141 | 100.0% | 0 | 0.0% |  |
| mitdb/102 | 1 | V2 | 0 | – | – | 5 | 100.0% | 141 | 100.0% | 0 | 3.4% |  |
| mitdb/103 | 0 | MLII | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/103 | 1 | V2 | 0 | – | – | 109 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/104 | 0 | V5 | 0 | – | – | 2 | 100.0% | 104 | 100.0% | 0 | 0.0% |  |
| mitdb/104 | 1 | V2 | 0 | – | – | 1 | 100.0% | 103 | 100.0% | 0 | 0.0% |  |
| mitdb/105 | 0 | MLII | 0 | – | – | 106 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/105 | 1 | V1 | 0 | – | – | 88 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/106 | 0 | MLII | 0 | – | – | 98 | 100.0% | 25 | 100.0% | 0 | 0.0% |  |
| mitdb/106 | 1 | V1 | 0 | – | – | 55 | 100.0% | 5 | 100.0% | 0 | 0.0% |  |
| mitdb/107 | 0 | MLII | 0 | – | – | 0 | – | 149 | 100.0% | 0 | 0.0% |  |
| mitdb/107 | 1 | V1 | 0 | – | – | 0 | – | 149 | 100.0% | 0 | 3.4% |  |
| mitdb/108 | 0 | MLII | 0 | – | – | 122 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/108 | 1 | V1 | 0 | – | – | 103 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/109 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/109 | 1 | V1 | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/111 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/111 | 1 | V1 | 0 | – | – | 147 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/112 | 0 | MLII | 0 | – | – | 147 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/112 | 1 | V1 | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/113 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/113 | 1 | V1 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/114 | 0 | V5 | 0 | – | – | 142 | 100.0% | 0 | – | 0 | 6.3% |  |
| mitdb/114 | 1 | MLII | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/115 | 0 | MLII | 0 | – | – | 146 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/115 | 1 | V1 | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/116 | 0 | MLII | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/116 | 1 | V1 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/117 | 0 | MLII | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/117 | 1 | V2 | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/118 | 0 | MLII | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/118 | 1 | V1 | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/119 | 0 | MLII | 0 | – | – | 75 | 100.0% | 3 | 100.0% | 0 | 0.0% |  |
| mitdb/119 | 1 | V1 | 0 | – | – | 73 | 100.0% | 3 | 100.0% | 0 | 0.0% |  |
| mitdb/121 | 0 | MLII | 0 | – | – | 146 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/121 | 1 | V1 | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/122 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/122 | 1 | V1 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/123 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/123 | 1 | V5 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/124 | 0 | MLII | 0 | – | – | 138 | 100.0% | 2 | 100.0% | 0 | 0.7% |  |
| mitdb/124 | 1 | V4 | 0 | – | – | 137 | 100.0% | 2 | 100.0% | 0 | 0.0% |  |
| mitdb/200 | 0 | MLII | 0 | – | – | 38 | 100.0% | 26 | 100.0% | 0 | 0.0% |  |
| mitdb/200 | 1 | V1 | 0 | – | – | 19 | 100.0% | 22 | 100.0% | 0 | 2.4% |  |
| mitdb/201 | 0 | MLII | 0 | – | – | 54 | 100.0% | 69 | 100.0% | 0 | 0.0% |  |
| mitdb/201 | 1 | V1 | 0 | – | – | 54 | 100.0% | 69 | 100.0% | 0 | 0.0% |  |
| mitdb/202 | 0 | MLII | 0 | – | – | 94 | 100.0% | 48 | 100.0% | 0 | 0.0% |  |
| mitdb/202 | 1 | V1 | 0 | – | – | 94 | 100.0% | 48 | 100.0% | 0 | 0.0% |  |
| mitdb/203 | 0 | MLII | 0 | – | – | 0 | – | 102 | 100.0% | 0 | 0.0% |  |
| mitdb/203 | 1 | V1 | 0 | – | – | 0 | – | 109 | 100.0% | 0 | 1.8% |  |
| mitdb/205 | 0 | MLII | 0 | – | – | 143 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/205 | 1 | V1 | 0 | – | – | 141 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/207 | 0 | MLII | 0 | – | 5/7 | 101 | 100.0% | 21 | 100.0% | 0 | 46.5% |  |
| mitdb/207 | 1 | V1 | 0 | – | 7/7 | 103 | 100.0% | 21 | 100.0% | 0 | 3.1% |  |
| mitdb/208 | 0 | MLII | 0 | – | – | 88 | 100.0% | 9 | 100.0% | 0 | 0.0% |  |
| mitdb/208 | 1 | V1 | 0 | – | – | 74 | 100.0% | 9 | 100.0% | 0 | 1.2% |  |
| mitdb/209 | 0 | MLII | 0 | – | – | 129 | 100.0% | 2 | 100.0% | 0 | 0.0% |  |
| mitdb/209 | 1 | V1 | 0 | – | – | 95 | 100.0% | 2 | 100.0% | 0 | 0.0% |  |
| mitdb/210 | 0 | MLII | 0 | – | – | 0 | – | 135 | 100.0% | 0 | 0.0% |  |
| mitdb/210 | 1 | V1 | 0 | – | – | 0 | – | 134 | 100.0% | 0 | 0.0% |  |
| mitdb/212 | 0 | MLII | 0 | – | – | 146 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/212 | 1 | V1 | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/213 | 0 | MLII | 0 | – | – | 127 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/213 | 1 | V1 | 0 | – | – | 127 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/214 | 0 | MLII | 0 | – | – | 128 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/214 | 1 | V1 | 0 | – | – | 128 | 100.0% | 0 | – | 0 | 3.1% |  |
| mitdb/215 | 0 | MLII | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/215 | 1 | V1 | 0 | – | – | 134 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/217 | 0 | MLII | 0 | – | – | 0 | – | 105 | 100.0% | 0 | 0.0% |  |
| mitdb/217 | 1 | V1 | 0 | – | – | 0 | – | 105 | 100.0% | 0 | 38.1% |  |
| mitdb/219 | 0 | MLII | 0 | – | – | 23 | 100.0% | 110 | 100.0% | 0 | 0.0% |  |
| mitdb/219 | 1 | V1 | 0 | – | – | 23 | 100.0% | 110 | 100.0% | 0 | 0.0% |  |
| mitdb/220 | 0 | MLII | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/220 | 1 | V1 | 0 | – | – | 143 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/221 | 0 | MLII | 0 | – | – | 0 | – | 133 | 100.0% | 0 | 0.0% |  |
| mitdb/221 | 1 | V1 | 0 | – | – | 0 | – | 129 | 100.0% | 0 | 0.0% |  |
| mitdb/222 | 0 | MLII | 0 | – | – | 55 | 100.0% | 18 | 100.0% | 0 | 0.0% |  |
| mitdb/222 | 1 | V1 | 0 | – | – | 55 | 100.0% | 18 | 100.0% | 0 | 0.0% |  |
| mitdb/223 | 0 | MLII | 0 | – | 0/6 | 106 | 100.0% | 16 | 100.0% | 0 | 0.0% |  |
| mitdb/223 | 1 | V1 | 0 | – | 0/4 | 101 | 100.0% | 16 | 100.0% | 0 | 0.0% |  |
| mitdb/228 | 0 | MLII | 0 | – | – | 80 | 100.0% | 15 | 100.0% | 0 | 0.0% |  |
| mitdb/228 | 1 | V1 | 0 | – | – | 95 | 100.0% | 15 | 100.0% | 0 | 0.0% |  |
| mitdb/230 | 0 | MLII | 0 | – | – | 45 | 100.0% | 16 | 100.0% | 0 | 0.0% |  |
| mitdb/230 | 1 | V1 | 0 | – | – | 45 | 100.0% | 15 | 100.0% | 0 | 0.0% |  |
| mitdb/231 | 0 | MLII | 0 | – | – | 85 | 100.0% | 52 | 100.0% | 0 | 0.0% |  |
| mitdb/231 | 1 | V1 | 0 | – | – | 85 | 100.0% | 52 | 100.0% | 0 | 0.0% |  |
| mitdb/232 | 0 | MLII | 0 | – | – | 0 | – | 131 | 100.0% | 0 | 0.0% |  |
| mitdb/232 | 1 | V1 | 0 | – | – | 0 | – | 132 | 100.0% | 0 | 0.0% |  |
| mitdb/233 | 0 | MLII | 0 | – | – | 113 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/233 | 1 | V1 | 0 | – | – | 113 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/234 | 0 | MLII | 0 | – | – | 146 | 100.0% | 1 | 100.0% | 0 | 0.0% |  |
| mitdb/234 | 1 | V1 | 0 | – | – | 138 | 100.0% | 1 | 100.0% | 0 | 0.0% |  |

## Continuous (streaming) run

One analyser runs over each whole record and channel without resets, deciding once a second. Each per-second decision is scored against the truth when its last 6 s (the 4 s window plus the two earlier windows needed for confirmation) lie inside one clean run (same guard margins as above). The decision scored is what the analyser reported for that window (`Shockable` only once confirmed; an unconfirmed shockable window counts as "not sure").

| Database | ch | Class | seconds | Shockable | NonShockable | Not sure |
|---|---|---|---|---|---|---|
| cudb | 0 | Coarse VF | 2636 | 37.2% | 6.0% | 56.8% |
| cudb | 0 | Normal sinus rhythm | 404 | 0.0% | 58.9% | 41.1% |
| cudb | 0 | Other non-shockable | 345 | 3.5% | 91.0% | 5.5% |
| cudb | 0 | cudb non-VF, rhythm not given | 9515 | 0.0% | 68.9% | 31.1% |
| vfdb | 0 | Coarse VF | 2166 | 45.1% | 2.4% | 52.5% |
| vfdb | 0 | Ventricular flutter | 305 | 53.4% | 5.9% | 40.7% |
| vfdb | 0 | VT, rate unknown | 4710 | 33.1% | 6.5% | 60.4% |
| vfdb | 0 | Normal sinus rhythm | 15856 | 0.1% | 76.4% | 23.5% |
| vfdb | 0 | Asystole | 677 | 1.3% | 12.4% | 86.3% |
| vfdb | 0 | Other non-shockable | 11947 | 0.0% | 83.5% | 16.5% |
| vfdb | 0 | HGEA (vfdb) | 721 | 0.0% | 76.4% | 23.6% |
| vfdb | 1 | Coarse VF | 2155 | 53.8% | 4.5% | 41.7% |
| vfdb | 1 | Fine VF (<200 µV) | 11 | 0.0% | 36.4% | 63.6% |
| vfdb | 1 | Ventricular flutter | 305 | 59.7% | 5.2% | 35.1% |
| vfdb | 1 | VT, rate unknown | 4710 | 28.4% | 9.3% | 62.3% |
| vfdb | 1 | Normal sinus rhythm | 15856 | 0.4% | 73.1% | 26.5% |
| vfdb | 1 | Asystole | 677 | 8.6% | 27.0% | 64.4% |
| vfdb | 1 | Other non-shockable | 11947 | 0.1% | 89.8% | 10.2% |
| vfdb | 1 | HGEA (vfdb) | 721 | 0.0% | 45.2% | 54.8% |
| mitdb | 0 | Ventricular flutter | 88 | 37.5% | 3.4% | 59.1% |
| mitdb | 0 | Slow VT (<150/min) | 70 | 0.0% | 78.6% | 21.4% |
| mitdb | 0 | Normal sinus rhythm | 56910 | 0.0% | 97.0% | 3.0% |
| mitdb | 0 | Other non-shockable | 17255 | 0.0% | 95.7% | 4.3% |
| mitdb | 1 | Ventricular flutter | 88 | 71.6% | 2.3% | 26.1% |
| mitdb | 1 | Slow VT (<150/min) | 53 | 0.0% | 100.0% | 0.0% |
| mitdb | 1 | Normal sinus rhythm | 54525 | 0.0% | 94.3% | 5.7% |
| mitdb | 1 | Other non-shockable | 17014 | 0.0% | 77.8% | 22.2% |

### Detection delay after VF / flutter onset

Measured from the annotated onset (cudb onsets are only approximate, per its documentation) to the first confirmed `Shockable` before the episode ends. The fastest possible is 7 s after the window first lies wholly in VF, i.e. about 6–7 s after onset. Episodes shorter than 7 s are counted apart. A delay under 6 s is faster than the analyser can confirm VF from scratch: it means the windows before the annotated onset already looked shockable to it (in vfdb record 418 the "NSR" between flutter bursts does). Noise marks are not removed here.

| Database | ch | label | episodes ≥7 s | detected | not detected | already shockable at onset | delay (detected) | of which under 6 s | episodes <7 s |
|---|---|---|---|---|---|---|---|---|---|
| cudb | 0 | VF | 47 | 42 | 5 | 0 | median 7.3 s, 90th pct 18.1 s, max 126.7 s | 16 | 0 |
| vfdb | 0 | VF | 12 | 11 | 1 | 0 | median 5.8 s, 90th pct 10.5 s, max 31.8 s | 6 | 1 |
| vfdb | 0 | VFL | 19 | 16 | 1 | 2 | median 4.8 s, 90th pct 8.0 s, max 13.9 s | 12 | 79 |
| vfdb | 1 | VF | 12 | 11 | 1 | 0 | median 6.0 s, 90th pct 8.5 s, max 9.3 s | 5 | 1 |
| vfdb | 1 | VFL | 19 | 16 | 0 | 3 | median 4.2 s, 90th pct 8.8 s, max 13.9 s | 14 | 79 |
| mitdb | 0 | VFL | 4 | 4 | 0 | 0 | median 7.5 s, 90th pct 8.0 s, max 8.1 s | 1 | 2 |
| mitdb | 1 | VFL | 4 | 4 | 0 | 0 | median 7.3 s, 90th pct 9.7 s, max 9.9 s | 2 | 2 |

### How long "shock" persists after a shockable rhythm ends

For VF, flutter and VT episodes followed by at least 10 s of a non-shockable (or cudb unlabelled) rhythm: the time from the annotated end to the first decision that is not `Shockable`. Zero means the analyser was not saying `Shockable` at the end. In cudb the end of VF is often a defibrillation shock, so the following seconds are shock artefact.

| Database | ch | episodes | shockable at end | persistence (those) | false shock 6–16 s after the end |
|---|---|---|---|---|---|
| cudb | 0 | 34 | 7 | median 0.7 s, 90th pct 0.9 s, max 0.9 s | 0 |
| vfdb | 0 | 60 | 19 | median 2.5 s, 90th pct 3.0 s, max 3.2 s | 5 |
| vfdb | 1 | 60 | 23 | median 2.6 s, 90th pct 3.4 s, max 8.7 s | 9 |
| mitdb | 0 | 46 | 0 | – | 0 |
| mitdb | 1 | 46 | 1 | median 0.9 s, 90th pct 0.9 s, max 0.9 s | 0 |

### Share of VF/flutter episodes reported `Shockable`, by seconds from onset (channel 0, episodes ≥ 20 s)

| Database | episodes | -5 s | +0 s | +2 s | +4 s | +5 s | +6 s | +7 s | +8 s | +9 s | +10 s | +12 s | +15 s | +20 s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cudb | 37 | 2.7% | 0.0% | 0.0% | 8.1% | 21.6% | 32.4% | 40.5% | 43.2% | 40.5% | 40.5% | 43.2% | 48.6% | 32.4% |
| vfdb | 18 | 11.1% | 0.0% | 0.0% | 11.1% | 33.3% | 44.4% | 44.4% | 38.9% | 50.0% | 44.4% | 44.4% | 44.4% | 55.6% |
| mitdb | 1 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 0.0% | 0.0% | 0.0% |

## Failure analysis

Nothing below was used to change the analyser. Where a threshold is "moved", it is a what-if count only.

### Which rule gave each window's verdict (streaming windows, all databases and channels)

| Class | windows | invalid measurement | saturated | clipped (flat tops) | amplitude implausible | asystole | noisy | possible artifact | fine VF (not shocked) | methods disagree | both methods shockable | neither method shockable |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.0% | 0.0% | 0.0% | 3.0% | 0.0% | 0.0% | 8.3% | 0.0% | 25.6% | 58.7% | 4.4% |
| Fine VF (<200 µV) | 11 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 18.2% | 0.0% | 9.1% | 27.3% | 9.1% | 36.4% |
| Ventricular flutter | 786 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 3.1% | 0.0% | 24.7% | 67.3% | 5.0% |
| Slow VT (<150/min) | 123 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 12.2% | 0.0% | 87.8% |
| VT, rate unknown | 9420 | 0.0% | 0.0% | 0.0% | 5.7% | 0.0% | 0.1% | 25.5% | 0.0% | 20.4% | 40.4% | 7.9% |
| Normal sinus rhythm | 143551 | 0.0% | 0.0% | 0.0% | 0.0% | 0.1% | 0.2% | 6.3% | 0.2% | 2.1% | 0.2% | 90.8% |
| Asystole | 1354 | 0.0% | 0.0% | 0.0% | 0.0% | 1.3% | 33.5% | 28.6% | 0.0% | 11.3% | 6.9% | 18.5% |
| Other non-shockable | 58508 | 0.0% | 0.0% | 0.0% | 1.0% | 0.0% | 0.0% | 7.8% | 0.0% | 4.3% | 0.1% | 86.7% |
| cudb non-VF, rhythm not given | 9515 | 0.0% | 0.0% | 0.0% | 0.2% | 0.1% | 0.0% | 20.2% | 0.0% | 10.5% | 0.1% | 68.8% |
| HGEA (vfdb) | 1442 | 0.0% | 0.0% | 0.0% | 1.5% | 0.0% | 0.0% | 21.7% | 0.0% | 15.7% | 0.2% | 60.8% |

Exclusion rules by database (share of all scored streaming windows, any class):

| Database | windows | saturated | implausible size | flatline | noisy | low-band (compression) | fine VF |
|---|---|---|---|---|---|---|---|
| cudb | 12900 | 0.0% | 1.8% | 0.1% | 0.0% | 19.3% | 0.0% |
| vfdb | 72764 | 0.0% | 1.6% | 0.0% | 0.8% | 17.4% | 0.0% |
| mitdb | 146003 | 0.0% | 0.0% | 0.1% | 0.2% | 2.8% | 0.2% |

### Feature distributions on real data versus the thresholds

Streaming windows, all databases and channels, by true class. "VF side" = share of windows on the shockable side of that feature's threshold(s). A feature only helps if VF windows are mostly on the VF side and non-shockable windows mostly are not.

**peak-to-peak (µV, after the analyser's filter)** (shockable side: ≥ 200 for shock, < 100 = flatline, > 8000 = implausible)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 446.620 | 646.175 | 1218.811 | 2199.114 | 6115.471 | 96.9% |
| Fine VF (<200 µV) | 11 | 131.995 | 162.390 | 169.566 | 194.287 | 221.161 | 18.2% |
| Ventricular flutter | 786 | 427.357 | 621.523 | 990.178 | 1342.693 | 2185.792 | 100.0% |
| Slow VT (<150/min) | 123 | 1656.322 | 1694.509 | 1868.272 | 2185.313 | 2703.410 | 100.0% |
| VT, rate unknown | 9420 | 317.644 | 718.501 | 2115.480 | 4511.925 | 8068.689 | 93.3% |
| Normal sinus rhythm | 143551 | 414.696 | 920.999 | 1573.194 | 2502.254 | 3902.993 | 98.2% |
| Asystole | 1354 | 109.429 | 134.088 | 183.491 | 552.357 | 829.862 | 44.1% |
| Other non-shockable | 58508 | 532.293 | 1134.819 | 1963.722 | 3238.874 | 6170.725 | 99.0% |
| cudb non-VF, rhythm not given | 9515 | 1706.277 | 2356.801 | 2786.119 | 3299.402 | 5775.219 | 99.6% |
| HGEA (vfdb) | 1442 | 842.406 | 4594.014 | 5559.711 | 6322.966 | 7281.264 | 98.5% |

**root-mean-square (µV, after the analyser's filter)** (shockable side: ≥ 0 (or leakage ≤ 1) to escape the flatline rule)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 86.959 | 125.466 | 251.964 | 450.239 | 1097.953 | 100.0% |
| Fine VF (<200 µV) | 11 | 24.117 | 26.533 | 29.514 | 31.340 | 35.351 | 100.0% |
| Ventricular flutter | 786 | 93.328 | 143.308 | 216.890 | 298.407 | 474.577 | 100.0% |
| Slow VT (<150/min) | 123 | 283.709 | 306.303 | 363.643 | 421.861 | 492.625 | 100.0% |
| VT, rate unknown | 9420 | 61.596 | 143.919 | 444.616 | 1015.342 | 2045.258 | 100.0% |
| Normal sinus rhythm | 143551 | 49.380 | 127.547 | 207.772 | 334.538 | 538.607 | 100.0% |
| Asystole | 1354 | 18.941 | 22.215 | 30.454 | 85.100 | 159.669 | 100.0% |
| Other non-shockable | 58508 | 76.681 | 161.120 | 259.066 | 516.605 | 941.757 | 100.0% |
| cudb non-VF, rhythm not given | 9515 | 210.371 | 356.511 | 454.657 | 586.523 | 836.131 | 100.0% |
| HGEA (vfdb) | 1442 | 175.110 | 615.689 | 748.076 | 913.106 | 1202.881 | 100.0% |

**noise ratio** (shockable side: ≤ 0.15)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.000 | 0.001 | 0.003 | 0.007 | 0.017 | 100.0% |
| Fine VF (<200 µV) | 11 | 0.081 | 0.096 | 0.109 | 0.145 | 0.181 | 81.8% |
| Ventricular flutter | 786 | 0.000 | 0.001 | 0.002 | 0.005 | 0.010 | 100.0% |
| Slow VT (<150/min) | 123 | 0.002 | 0.002 | 0.002 | 0.002 | 0.004 | 100.0% |
| VT, rate unknown | 9420 | 0.000 | 0.001 | 0.002 | 0.007 | 0.026 | 99.9% |
| Normal sinus rhythm | 143551 | 0.002 | 0.006 | 0.014 | 0.031 | 0.067 | 99.7% |
| Asystole | 1354 | 0.004 | 0.013 | 0.103 | 0.179 | 0.244 | 65.3% |
| Other non-shockable | 58508 | 0.001 | 0.003 | 0.006 | 0.011 | 0.032 | 100.0% |
| cudb non-VF, rhythm not given | 9515 | 0.000 | 0.002 | 0.004 | 0.008 | 0.018 | 100.0% |
| HGEA (vfdb) | 1442 | 0.001 | 0.002 | 0.002 | 0.004 | 0.011 | 100.0% |

**low-band fraction** (shockable side: ≤ 0.5)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.007 | 0.028 | 0.080 | 0.210 | 0.652 | 90.7% |
| Fine VF (<200 µV) | 11 | 0.036 | 0.088 | 0.145 | 0.204 | 0.232 | 100.0% |
| Ventricular flutter | 786 | 0.009 | 0.024 | 0.058 | 0.135 | 0.429 | 96.9% |
| Slow VT (<150/min) | 123 | 0.118 | 0.148 | 0.190 | 0.326 | 0.374 | 100.0% |
| VT, rate unknown | 9420 | 0.005 | 0.032 | 0.152 | 0.545 | 0.651 | 72.5% |
| Normal sinus rhythm | 143551 | 0.030 | 0.082 | 0.159 | 0.268 | 0.524 | 93.7% |
| Asystole | 1354 | 0.053 | 0.149 | 0.272 | 0.574 | 0.786 | 71.3% |
| Other non-shockable | 58508 | 0.067 | 0.146 | 0.241 | 0.372 | 0.562 | 92.1% |
| cudb non-VF, rhythm not given | 9515 | 0.066 | 0.152 | 0.237 | 0.464 | 0.672 | 79.6% |
| HGEA (vfdb) | 1442 | 0.108 | 0.229 | 0.341 | 0.479 | 0.609 | 78.2% |

**threshold-crossing fraction (TCSC)** (shockable side: > 0.45)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.554 | 0.634 | 0.678 | 0.729 | 0.806 | 98.7% |
| Fine VF (<200 µV) | 11 | 0.538 | 0.554 | 0.572 | 0.634 | 0.650 | 100.0% |
| Ventricular flutter | 786 | 0.579 | 0.684 | 0.739 | 0.781 | 0.850 | 100.0% |
| Slow VT (<150/min) | 123 | 0.422 | 0.454 | 0.494 | 0.741 | 0.785 | 78.0% |
| VT, rate unknown | 9420 | 0.536 | 0.651 | 0.689 | 0.731 | 0.814 | 97.1% |
| Normal sinus rhythm | 143551 | 0.058 | 0.125 | 0.222 | 0.350 | 0.591 | 13.2% |
| Asystole | 1354 | 0.447 | 0.548 | 0.590 | 0.629 | 0.683 | 94.6% |
| Other non-shockable | 58508 | 0.138 | 0.248 | 0.350 | 0.459 | 0.617 | 26.3% |
| cudb non-VF, rhythm not given | 9515 | 0.156 | 0.255 | 0.395 | 0.563 | 0.711 | 42.1% |
| HGEA (vfdb) | 1442 | 0.287 | 0.372 | 0.438 | 0.511 | 0.669 | 45.4% |

**band-pass count** (shockable side: > 0.18)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.109 | 0.167 | 0.210 | 0.253 | 0.325 | 68.1% |
| Fine VF (<200 µV) | 11 | 0.172 | 0.174 | 0.183 | 0.190 | 0.228 | 54.5% |
| Ventricular flutter | 786 | 0.133 | 0.186 | 0.238 | 0.288 | 0.392 | 78.1% |
| Slow VT (<150/min) | 123 | 0.051 | 0.058 | 0.070 | 0.083 | 0.094 | 0.0% |
| VT, rate unknown | 9420 | 0.079 | 0.172 | 0.213 | 0.254 | 0.351 | 70.5% |
| Normal sinus rhythm | 143551 | 0.029 | 0.041 | 0.052 | 0.067 | 0.105 | 1.9% |
| Asystole | 1354 | 0.091 | 0.130 | 0.160 | 0.190 | 0.231 | 31.4% |
| Other non-shockable | 58508 | 0.030 | 0.051 | 0.065 | 0.079 | 0.109 | 0.1% |
| cudb non-VF, rhythm not given | 9515 | 0.039 | 0.052 | 0.067 | 0.112 | 0.179 | 4.6% |
| HGEA (vfdb) | 1442 | 0.046 | 0.062 | 0.077 | 0.098 | 0.140 | 0.3% |

**VF-filter leakage** (shockable side: < 0.625)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.260 | 0.384 | 0.479 | 0.572 | 0.716 | 86.1% |
| Fine VF (<200 µV) | 11 | 0.591 | 0.653 | 0.702 | 0.762 | 0.796 | 18.2% |
| Ventricular flutter | 786 | 0.239 | 0.312 | 0.414 | 0.550 | 0.703 | 85.1% |
| Slow VT (<150/min) | 123 | 0.620 | 0.638 | 0.686 | 0.710 | 0.751 | 12.2% |
| VT, rate unknown | 9420 | 0.238 | 0.387 | 0.496 | 0.562 | 0.748 | 85.5% |
| Normal sinus rhythm | 143551 | 0.621 | 0.737 | 0.789 | 0.830 | 0.867 | 5.3% |
| Asystole | 1354 | 0.577 | 0.796 | 0.840 | 0.874 | 0.901 | 8.3% |
| Other non-shockable | 58508 | 0.607 | 0.675 | 0.733 | 0.779 | 0.838 | 9.0% |
| cudb non-VF, rhythm not given | 9515 | 0.563 | 0.642 | 0.732 | 0.790 | 0.846 | 20.3% |
| HGEA (vfdb) | 1442 | 0.534 | 0.618 | 0.673 | 0.733 | 0.787 | 28.6% |

**dominant frequency (Hz)** (shockable side: > 2.4 and < 9)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 1.250 | 2.750 | 3.750 | 4.500 | 6.000 | 85.0% |
| Fine VF (<200 µV) | 11 | 3.500 | 8.125 | 8.250 | 8.500 | 8.750 | 90.9% |
| Ventricular flutter | 786 | 2.000 | 3.250 | 3.500 | 3.750 | 4.250 | 93.1% |
| Slow VT (<150/min) | 123 | 1.750 | 1.750 | 2.500 | 2.750 | 2.750 | 57.7% |
| VT, rate unknown | 9420 | 1.750 | 2.250 | 3.000 | 4.000 | 5.250 | 66.2% |
| Normal sinus rhythm | 143551 | 1.250 | 1.750 | 2.750 | 3.500 | 7.000 | 56.5% |
| Asystole | 1354 | 1.000 | 1.250 | 2.750 | 4.250 | 5.000 | 52.1% |
| Other non-shockable | 58508 | 1.250 | 1.750 | 2.500 | 3.000 | 5.250 | 50.9% |
| cudb non-VF, rhythm not given | 9515 | 1.250 | 1.750 | 2.250 | 3.500 | 6.000 | 47.4% |
| HGEA (vfdb) | 1442 | 1.500 | 2.000 | 2.250 | 2.750 | 3.250 | 40.0% |

**spectral concentration** (shockable side: > 0.4)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.391 | 0.613 | 0.771 | 0.872 | 0.942 | 94.6% |
| Fine VF (<200 µV) | 11 | 0.321 | 0.497 | 0.556 | 0.661 | 0.765 | 90.9% |
| Ventricular flutter | 786 | 0.451 | 0.734 | 0.859 | 0.914 | 0.949 | 97.2% |
| Slow VT (<150/min) | 123 | 0.259 | 0.311 | 0.471 | 0.493 | 0.510 | 56.9% |
| VT, rate unknown | 9420 | 0.414 | 0.673 | 0.799 | 0.877 | 0.954 | 95.5% |
| Normal sinus rhythm | 143551 | 0.126 | 0.210 | 0.293 | 0.399 | 0.574 | 24.8% |
| Asystole | 1354 | 0.216 | 0.380 | 0.492 | 0.614 | 0.789 | 71.4% |
| Other non-shockable | 58508 | 0.167 | 0.280 | 0.367 | 0.439 | 0.552 | 37.9% |
| cudb non-VF, rhythm not given | 9515 | 0.144 | 0.268 | 0.431 | 0.514 | 0.651 | 55.7% |
| HGEA (vfdb) | 1442 | 0.233 | 0.407 | 0.561 | 0.653 | 0.760 | 75.8% |

**saturated fraction (front-end limit)** (shockable side: ≤ 0.01)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Fine VF (<200 µV) | 11 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Ventricular flutter | 786 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Slow VT (<150/min) | 123 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| VT, rate unknown | 9420 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Normal sinus rhythm | 143551 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Asystole | 1354 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Other non-shockable | 58508 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| cudb non-VF, rhythm not given | 9515 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| HGEA (vfdb) | 1442 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |

**clipped fraction (flat tops)** (shockable side: ≤ 1)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 6957 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Fine VF (<200 µV) | 11 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Ventricular flutter | 786 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Slow VT (<150/min) | 123 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| VT, rate unknown | 9420 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Normal sinus rhythm | 143551 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Asystole | 1354 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Other non-shockable | 58508 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| cudb non-VF, rhythm not given | 9515 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| HGEA (vfdb) | 1442 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |

### Missed coarse VF (12 s analyses, all channels)

190 of 576 coarse-VF segments did not end in a shock: 83 ended "no shock", 107 inconclusive.

How the missed ones ended (the verdict of the window that ended the analysis, or for inconclusive ones, the most common verdict among its windows):

| Ended by | segments |
|---|---|
| amplitude implausible | 12 |
| both methods shockable | 3 |
| methods disagree | 63 |
| neither method shockable | 83 |
| possible artifact | 29 |

Which conditions failed, over 1138 windows of missed coarse-VF segments (10.6% passed all of them). "Only blocker" = the window failed that condition and no other, so that single threshold alone kept it from being shockable.

| Condition | rule | failed (any) | only blocker |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% | 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 1 | 0.0% | 0.0% |
| plausible size | peak-to-peak ≤ 8000 µV | 9.6% | 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 0 µV with leakage > 1) | 0.0% | 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.1% | 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 27.4% | 0.3% |
| TCSC | threshold-crossing fraction > 0.45 | 3.6% | 0.0% |
| band-pass count | band-pass count > 0.18 | 69.1% | 36.1% |
| VF-filter leakage | leakage < 0.625 | 33.2% | 8.6% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 36.5% | 0.9% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.9% | 0.6% |
| spectral concentration | concentration > 0.4 | 9.8% | 0.3% |
| not fine VF | peak-to-peak ≥ 200 µV | 0.4% | 0.2% |

Which conditions failed, over 6957 all coarse-VF streaming windows (58.7% passed all of them). "Only blocker" = the window failed that condition and no other, so that single threshold alone kept it from being shockable.

| Condition | rule | failed (any) | only blocker |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% | 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 1 | 0.0% | 0.0% |
| plausible size | peak-to-peak ≤ 8000 µV | 3.0% | 0.2% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 0 µV with leakage > 1) | 0.0% | 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.0% | 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 9.3% | 0.1% |
| TCSC | threshold-crossing fraction > 0.45 | 1.3% | 0.0% |
| band-pass count | band-pass count > 0.18 | 31.9% | 19.0% |
| VF-filter leakage | leakage < 0.625 | 13.9% | 3.3% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 14.8% | 0.8% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.3% | 0.2% |
| spectral concentration | concentration > 0.4 | 5.4% | 0.5% |
| not fine VF | peak-to-peak ≥ 200 µV | 0.1% | 0.0% |

### False shocks on non-shockable rhythms (12 s analyses, all channels)

Worst rhythm classes by false-shock rate:

| Rhythm | false shocks | segments | rate |
|---|---|---|---|
| ASYS | 7 | 112 | 6.2% |
| SVTA | 2 | 252 | 0.8% |
| AFIB | 2 | 1656 | 0.1% |
| NSR | 13 | 11955 | 0.1% |
| AB (atrial bigeminy) | 0 | 10 | 0.0% |
| AFL | 0 | 65 | 0.0% |
| B (ventricular bigeminy) | 0 | 224 | 0.0% |
| BI (1st-degree block) | 0 | 664 | 0.0% |
| BII (2nd-degree block) | 0 | 104 | 0.0% |
| IVR (idioventricular) | 0 | 18 | 0.0% |
| NOD (junctional) | 0 | 284 | 0.0% |
| PACED | 0 | 1221 | 0.0% |
| PREX (pre-excitation) | 0 | 31 | 0.0% |
| SBR | 0 | 265 | 0.0% |
| T (ventricular trigeminy) | 0 | 66 | 0.0% |
| VER (ventricular escape) | 0 | 4 | 0.0% |

Every false shock passed all 13 conditions in at least 3 of the last 3 windows (that is the only way to reach `Shockable`), so "which rule fired" is always "both method families said shockable". The useful question is which condition was closest to stopping it. For each condition: the share of the confirming windows of every false shock that would have been stopped had that one threshold been 10 % / 20 % stricter, against the share of correctly shocked coarse-VF confirming windows that the same change would also stop (the cost). What-if counts only; nothing was changed.

| Condition | rule | false-shock windows stopped at 10% / 20% stricter (of 72) | coarse-VF windows also stopped at 10% / 20% (of 1158) |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% / 0.0% | 0.0% / 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 1 | 0.0% / 0.0% | 0.0% / 0.0% |
| plausible size | peak-to-peak ≤ 8000 µV | 0.0% / 0.0% | 0.0% / 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 0 µV with leakage > 1) | 0.0% / 0.0% | 0.0% / 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.0% / 0.0% | 0.0% / 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 4.2% / 11.1% | 0.2% / 0.3% |
| TCSC | threshold-crossing fraction > 0.45 | 0.0% / 0.0% | 0.0% / 0.0% |
| band-pass count | band-pass count > 0.18 | 19.4% / 45.8% | 12.3% / 27.7% |
| VF-filter leakage | leakage < 0.625 | 23.6% / 47.2% | 10.6% / 28.6% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 30.6% / 50.0% | 2.0% / 9.7% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.0% / 0.0% | 0.1% / 0.9% |
| spectral concentration | concentration > 0.4 | 0.0% / 0.0% | 0.8% / 2.9% |
| not fine VF | peak-to-peak ≥ 200 µV | 8.3% / 8.3% | 0.0% / 0.0% |

How often each condition already lets a non-shockable window through (streaming NSR + other non-shockable windows, n = 202059): a condition passed by nearly every non-shockable window does no work.

| Condition | non-shockable windows on the VF side | coarse-VF windows on the VF side |
|---|---|---|
| not saturated | 100.0% | 100.0% |
| not clipped (flat tops) | 100.0% | 100.0% |
| plausible size | 99.7% | 97.0% |
| not flatline | 99.9% | 100.0% |
| not noisy | 99.8% | 100.0% |
| not low-band (compression) | 93.2% | 90.7% |
| TCSC | 17.0% | 98.7% |
| band-pass count | 1.4% | 68.1% |
| VF-filter leakage | 6.4% | 86.1% |
| dominant freq. low limit | 56.0% | 85.2% |
| dominant freq. high limit | 98.9% | 99.7% |
| spectral concentration | 28.6% | 94.6% |
| not fine VF | 98.8% | 99.9% |

False shocks by record and channel (24 in all):

| Record | false shocks | on rhythm(s) |
|---|---|---|
| vfdb/418 ch1 | 10 | NSR |
| vfdb/607 ch1 | 5 | ASYS |
| cudb/cu09 ch0 | 2 | AFIB |
| vfdb/418 ch0 | 2 | NSR |
| vfdb/607 ch0 | 2 | ASYS |
| vfdb/419 ch1 | 1 | NSR |
| vfdb/426 ch0 | 1 | SVTA |
| vfdb/426 ch1 | 1 | SVTA |

### VT without beat annotations, split by a rough rate estimated from the signal

vfdb gives VT episodes but no beats, so the AHA rate criterion cannot be applied from the annotations. As a rough guide only, each segment's rate was estimated from the signal's strongest repetition (autocorrelation, 60–300/min). This helper is UNVERIFIED and can be off by a factor of two; it is not used for any AHA row above. 12 s analyses, both channels counted.

| Estimated rate | segments | records | shock | no shock | inconcl. | shock rate |
|---|---|---|---|---|---|---|
| ≥ 150/min (would be rapid VT) | 508 | 14 | 319 | 101 | 88 | 62.8% |
| < 150/min (would be slow VT) | 226 | 11 | 13 | 31 | 182 | 5.8% |
| no clear rate | 44 | 4 | 6 | 23 | 15 | 13.6% |

### Every false shock (24 in all, both channels)

Normal rhythm first, then asystole, then by how fast the shock came. Features are those of the window that confirmed the shock.

| Record | ch | start (s) | true rhythm | amplitude (µV) | rate | outcome | at (s) | window verdict | features |
|---|---|---|---|---|---|---|---|---|---|
| vfdb/418 | 0 | 657.1 | NSR | 1731 | – | shock | 7 | both methods shockable | pp 2338 µV, noise 0.00, low-band 0.17, TCSC 0.80, count 0.21, leak 0.42, dom 3.25 Hz, conc 0.87 |
| vfdb/418 | 0 | 957.2 | NSR | 1899 | – | shock | 7 | both methods shockable | pp 2393 µV, noise 0.00, low-band 0.21, TCSC 0.80, count 0.19, leak 0.40, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 1 | 657.1 | NSR | 696 | – | shock | 7 | both methods shockable | pp 901 µV, noise 0.00, low-band 0.18, TCSC 0.76, count 0.23, leak 0.45, dom 3.25 Hz, conc 0.86 |
| vfdb/418 | 1 | 805.5 | NSR | 820 | – | shock | 7 | both methods shockable | pp 1031 µV, noise 0.00, low-band 0.15, TCSC 0.70, count 0.22, leak 0.41, dom 2.75 Hz, conc 0.85 |
| vfdb/418 | 1 | 957.2 | NSR | 771 | – | shock | 7 | both methods shockable | pp 1039 µV, noise 0.00, low-band 0.21, TCSC 0.77, count 0.25, leak 0.40, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 1 | 39.1 | NSR | 639 | – | shock | 8 | both methods shockable | pp 714 µV, noise 0.00, low-band 0.43, TCSC 0.80, count 0.20, leak 0.55, dom 3.00 Hz, conc 0.59 |
| vfdb/418 | 1 | 817.5 | NSR | 727 | – | shock | 8 | both methods shockable | pp 1052 µV, noise 0.00, low-band 0.28, TCSC 0.78, count 0.25, leak 0.46, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 1 | 549.5 | NSR | 751 | – | shock | 10 | both methods shockable | pp 908 µV, noise 0.00, low-band 0.39, TCSC 0.79, count 0.20, leak 0.50, dom 2.50 Hz, conc 0.68 |
| vfdb/418 | 1 | 573.5 | NSR | 784 | – | shock | 10 | both methods shockable | pp 820 µV, noise 0.00, low-band 0.29, TCSC 0.79, count 0.24, leak 0.50, dom 3.00 Hz, conc 0.76 |
| vfdb/418 | 1 | 969.2 | NSR | 700 | – | shock | 10 | both methods shockable | pp 801 µV, noise 0.00, low-band 0.30, TCSC 0.76, count 0.25, leak 0.46, dom 3.25 Hz, conc 0.67 |
| vfdb/419 | 1 | 3.1 | NSR | 742 | – | shock | 10 | both methods shockable | pp 717 µV, noise 0.00, low-band 0.43, TCSC 0.76, count 0.19, leak 0.55, dom 3.00 Hz, conc 0.62 |
| vfdb/418 | 1 | 829.5 | NSR | 810 | – | shock | 11 | both methods shockable | pp 943 µV, noise 0.00, low-band 0.22, TCSC 0.83, count 0.24, leak 0.45, dom 3.25 Hz, conc 0.77 |
| vfdb/418 | 1 | 433.0 | NSR | 760 | – | shock | 12 | both methods shockable | pp 960 µV, noise 0.00, low-band 0.22, TCSC 0.79, count 0.21, leak 0.42, dom 3.25 Hz, conc 0.75 |
| vfdb/607 | 0 | 1764.5 | ASYS | 227 | – | shock | 7 | both methods shockable | pp 201 µV, noise 0.07, low-band 0.06, TCSC 0.65, count 0.22, leak 0.62, dom 4.50 Hz, conc 0.81 |
| vfdb/607 | 0 | 1776.5 | ASYS | 227 | – | shock | 7 | both methods shockable | pp 216 µV, noise 0.07, low-band 0.02, TCSC 0.68, count 0.23, leak 0.61, dom 4.00 Hz, conc 0.84 |
| vfdb/607 | 1 | 1716.5 | ASYS | 321 | – | shock | 7 | both methods shockable | pp 389 µV, noise 0.05, low-band 0.09, TCSC 0.68, count 0.27, leak 0.61, dom 4.25 Hz, conc 0.58 |
| vfdb/607 | 1 | 1728.5 | ASYS | 373 | – | shock | 7 | both methods shockable | pp 468 µV, noise 0.02, low-band 0.08, TCSC 0.68, count 0.28, leak 0.50, dom 4.25 Hz, conc 0.79 |
| vfdb/607 | 1 | 1740.5 | ASYS | 313 | – | shock | 7 | both methods shockable | pp 348 µV, noise 0.04, low-band 0.03, TCSC 0.64, count 0.20, leak 0.52, dom 4.75 Hz, conc 0.88 |
| vfdb/607 | 1 | 1764.5 | ASYS | 374 | – | shock | 7 | both methods shockable | pp 367 µV, noise 0.03, low-band 0.05, TCSC 0.70, count 0.24, leak 0.50, dom 4.50 Hz, conc 0.83 |
| vfdb/607 | 1 | 1776.5 | ASYS | 379 | – | shock | 7 | both methods shockable | pp 394 µV, noise 0.03, low-band 0.01, TCSC 0.72, count 0.28, leak 0.51, dom 4.00 Hz, conc 0.86 |
| cudb/cu09 | 0 | 493.8 | AFIB | 5204 | 155/min | shock | 7 | both methods shockable | pp 5628 µV, noise 0.00, low-band 0.07, TCSC 0.72, count 0.21, leak 0.48, dom 2.50 Hz, conc 0.72 |
| cudb/cu09 | 0 | 481.8 | AFIB | 5031 | 163/min | shock | 10 | both methods shockable | pp 6555 µV, noise 0.00, low-band 0.10, TCSC 0.74, count 0.25, leak 0.42, dom 2.75 Hz, conc 0.87 |
| vfdb/426 | 0 | 1842.2 | SVTA | 780 | – | shock | 10 | both methods shockable | pp 1184 µV, noise 0.00, low-band 0.10, TCSC 0.71, count 0.23, leak 0.58, dom 2.75 Hz, conc 0.70 |
| vfdb/426 | 1 | 1842.2 | SVTA | 823 | – | shock | 10 | both methods shockable | pp 1367 µV, noise 0.00, low-band 0.11, TCSC 0.70, count 0.24, leak 0.56, dom 2.75 Hz, conc 0.71 |

### The 20 worst missed VF segments

Largest VF first (a missed large VF is the worst kind). Features are those of the last window looked at.

| Record | ch | start (s) | true rhythm | amplitude (µV) | rate | outcome | at (s) | window verdict | features |
|---|---|---|---|---|---|---|---|---|---|
| cudb/cu11 | 0 | 470.2 | VF | 10411 | – | inconclusive | – | amplitude implausible | pp 17396 µV, noise 0.05, low-band 0.59, TCSC 0.54, count 0.04, leak 0.81, dom 1.25 Hz, conc 0.48 |
| cudb/cu12 | 0 | 312.3 | VF | 10187 | – | inconclusive | – | amplitude implausible | pp 10007 µV, noise 0.12, low-band 0.03, TCSC 0.57, count 0.14, leak 0.82, dom 4.00 Hz, conc 0.86 |
| cudb/cu11 | 0 | 482.2 | VF | 9216 | – | inconclusive | – | amplitude implausible | pp 12747 µV, noise 0.04, low-band 0.26, TCSC 0.55, count 0.25, leak 0.63, dom 5.50 Hz, conc 0.57 |
| cudb/cu11 | 0 | 458.2 | VF | 7631 | – | inconclusive | – | possible artifact | pp 5811 µV, noise 0.00, low-band 0.53, TCSC 0.57, count 0.13, leak 0.72, dom 1.25 Hz, conc 0.41 |
| cudb/cu11 | 0 | 446.2 | VF | 7315 | – | inconclusive | – | amplitude implausible | pp 14264 µV, noise 0.07, low-band 0.39, TCSC 0.50, count 0.23, leak 0.73, dom 1.75 Hz, conc 0.39 |
| cudb/cu12 | 0 | 372.3 | VF | 7041 | – | inconclusive | – | amplitude implausible | pp 11043 µV, noise 0.01, low-band 0.46, TCSC 0.37, count 0.09, leak 0.79, dom 1.25 Hz, conc 0.32 |
| cudb/cu12 | 0 | 384.3 | VF | 6815 | – | no-shock | 9 | neither method shockable | pp 7339 µV, noise 0.02, low-band 0.18, TCSC 0.37, count 0.04, leak 0.76, dom 3.75 Hz, conc 0.30 |
| cudb/cu11 | 0 | 422.2 | VF | 6073 | – | no-shock | 5 | neither method shockable | pp 7320 µV, noise 0.00, low-band 0.49, TCSC 0.56, count 0.12, leak 0.76, dom 1.25 Hz, conc 0.38 |
| cudb/cu12 | 0 | 288.3 | VF | 5880 | – | no-shock | 5 | neither method shockable | pp 6116 µV, noise 0.01, low-band 0.02, TCSC 0.70, count 0.16, leak 0.66, dom 3.25 Hz, conc 0.81 |
| cudb/cu12 | 0 | 348.3 | VF | 5535 | – | no-shock | 5 | neither method shockable | pp 6031 µV, noise 0.02, low-band 0.16, TCSC 0.48, count 0.08, leak 0.73, dom 3.75 Hz, conc 0.68 |
| cudb/cu30 | 0 | 66.4 | VF | 5394 | – | no-shock | 9 | neither method shockable | pp 6019 µV, noise 0.00, low-band 0.46, TCSC 0.44, count 0.07, leak 0.82, dom 1.75 Hz, conc 0.41 |
| cudb/cu30 | 0 | 78.4 | VF | 5367 | – | no-shock | 5 | neither method shockable | pp 6439 µV, noise 0.00, low-band 0.40, TCSC 0.56, count 0.10, leak 0.80, dom 1.25 Hz, conc 0.35 |
| cudb/cu10 | 0 | 487.5 | VF | 4613 | – | inconclusive | – | possible artifact | pp 7315 µV, noise 0.00, low-band 0.51, TCSC 0.56, count 0.18, leak 0.59, dom 1.25 Hz, conc 0.45 |
| cudb/cu23 | 0 | 397.4 | VF | 4485 | – | inconclusive | – | amplitude implausible | pp 13972 µV, noise 0.03, low-band 0.55, TCSC 0.43, count 0.13, leak 0.84, dom 1.50 Hz, conc 0.51 |
| cudb/cu30 | 0 | 102.4 | VF | 4462 | – | no-shock | 5 | neither method shockable | pp 6797 µV, noise 0.00, low-band 0.30, TCSC 0.66, count 0.15, leak 0.75, dom 1.75 Hz, conc 0.29 |
| cudb/cu12 | 0 | 336.3 | VF | 4367 | – | no-shock | 6 | neither method shockable | pp 6617 µV, noise 0.07, low-band 0.04, TCSC 0.40, count 0.06, leak 0.73, dom 3.25 Hz, conc 0.26 |
| cudb/cu30 | 0 | 90.4 | VF | 4181 | – | inconclusive | – | possible artifact | pp 4524 µV, noise 0.00, low-band 0.67, TCSC 0.53, count 0.10, leak 0.76, dom 1.75 Hz, conc 0.51 |
| cudb/cu20 | 0 | 331.2 | VF | 4106 | – | inconclusive | – | possible artifact | pp 3917 µV, noise 0.00, low-band 0.93, TCSC 0.77, count 0.11, leak 0.54, dom 1.00 Hz, conc 0.67 |
| cudb/cu11 | 0 | 434.2 | VF | 3909 | – | inconclusive | – | amplitude implausible | pp 9032 µV, noise 0.00, low-band 0.49, TCSC 0.54, count 0.19, leak 0.72, dom 2.00 Hz, conc 0.52 |
| cudb/cu20 | 0 | 307.2 | VF | 3806 | – | inconclusive | – | possible artifact | pp 3277 µV, noise 0.00, low-band 0.95, TCSC 0.78, count 0.18, leak 0.49, dom 1.00 Hz, conc 0.38 |

## Amplitude calibration and the microvolt rules

The analyser has three absolute-size rules: under 100 µV peak to peak is flatline (no shock); under 200 µV VF is "fine" (not shocked); over 8000 µV is implausible (not sure). They only mean something if the recording's microvolts are right. cudb's gain of 400 ADC units/mV is described by PhysioNet as nominal (10 mV full scale "nominal"; cu01 was replayed from a Holter tape). Nothing here was rescaled.

Per database, channel 0: the median over records of each record's median ground-truth amplitude (1-s peak to peak, drift removed) in clean streaming windows, and how VF windows fall against the analyser's own size rules (its peak-to-peak is measured after its 1–30 Hz filter, so it reads lower than the ground-truth figure).

| Database | non-VF amplitude (median of record medians) | VF amplitude (median of record medians) | VF windows | analyser pp < 200 µV (fine-VF rule) | analyser pp < 100 µV (flatline rule) | ground truth < 200 µV |
|---|---|---|---|---|---|---|
| cudb | 2606 µV (32 records) | 2406 µV (27 records) | 2636 | 0.0% | 0.0% | 0.0% |
| vfdb | 1274 µV (21 records) | 602 µV (4 records) | 2166 | 0.1% | 0.0% | 0.0% |
| mitdb | 1992 µV (48 records) | – | 0 | – | – | – |

Per cudb record, channel 0 (to show whether the nominal calibration is consistent between records):

| Record | non-VF amplitude (median, µV) | VF amplitude (median, µV) | VF windows called fine VF | VF windows called flatline |
|---|---|---|---|---|
| cu01 | 2579 | 1308 | 0.0% | 0.0% |
| cu02 | 1809 | – | – | – |
| cu03 | 1549 | 1373 | 0.0% | 0.0% |
| cu04 | 1886 | 1507 | 0.0% | 0.0% |
| cu05 | 2634 | 3473 | 0.0% | 0.0% |
| cu06 | 3262 | 2386 | 0.0% | 0.0% |
| cu07 | 2353 | 1386 | 0.0% | 0.0% |
| cu08 | – | – | – | – |
| cu09 | 3036 | 1758 | 0.0% | 0.0% |
| cu10 | 2555 | 2237 | 0.0% | 0.0% |
| cu11 | 3372 | 4073 | 0.0% | 0.0% |
| cu12 | 3585 | 3508 | 0.0% | 0.0% |
| cu13 | 2576 | – | – | – |
| cu14 | 3235 | – | – | – |
| cu15 | 3274 | 2406 | 0.0% | 0.0% |
| cu16 | 3149 | 4823 | 0.0% | 0.0% |
| cu17 | 1674 | 2182 | 0.0% | 0.0% |
| cu18 | 3103 | 3936 | 0.0% | 0.0% |
| cu19 | 2409 | 4666 | 0.0% | 0.0% |
| cu20 | 2846 | 2797 | 0.0% | 0.0% |
| cu21 | 3025 | 3340 | 0.0% | 0.0% |
| cu22 | 2807 | 2207 | 0.0% | 0.0% |
| cu23 | 2645 | 3073 | 0.0% | 0.0% |
| cu24 | 2486 | 2455 | 0.0% | 0.0% |
| cu25 | 1816 | 1137 | 0.0% | 0.0% |
| cu26 | 2439 | 1909 | 0.0% | 0.0% |
| cu27 | – | – | – | – |
| cu28 | 1801 | – | – | – |
| cu29 | 2827 | – | – | – |
| cu30 | 2635 | 3850 | 0.0% | 0.0% |
| cu31 | 3551 | 3372 | 0.0% | 0.0% |
| cu32 | 2282 | 1372 | 0.0% | 0.0% |
| cu33 | 2303 | 2164 | 0.0% | 0.0% |
| cu34 | 1512 | 5962 | 0.0% | 0.0% |
| cu35 | – | – | – | – |

## Reproducing this report

```
cargo run --release -p aed-eval -- --data <path to data/physionet> --csv <out dir>
```

The data are not in the repository; see `firmware/crates/aed-eval/README.md` for where to get them, their licences and the citations they ask for.
