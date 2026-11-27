# Rhythm analyser on recorded heart rhythms

- **Date (UTC):** 2026-11-27
- **Analyser evaluated:** `aed-rhythm` at git commit `b9a79e7` (**uncommitted changes in the analyser**: M firmware/crates/aed-rhythm/src/analyzer.rs;  M firmware/crates/aed-rhythm/src/features.rs;  M firmware/crates/aed-rhythm/src/lib.rs;  M firmware/crates/aed-rhythm/src/settings.rs; 16 uncommitted path(s) elsewhere in the repository)
- **Data:** PhysioNet cudb, vfdb, mitdb (public Holter / monitoring recordings), read from `data/physionet`
- **Records evaluated:** 52
- **Restricted to:** 52 records listed in `crates/aed-eval/splits/held_out.txt` (SHA-256 `3e5c70316b41bdaacc76471a7a775b3b41c99a95dbbba182fb47e13a44d26601`)

**Held-out half, frozen settings, single run.** These records were not used for any tuning. The settings were frozen before this run (TUNING_LOG.md). The untuned analyser had been run on all records, including these, before the split was made, so this is not a fully blind test.

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
| cudb | 17 | 17 | 250 | 17/17 | 17/17 | 47032 (2.2%) | 0 | 400 | none |
| vfdb | 11 | 22 | 250 | 22/22 | 22/22 | 0 (0.0%) | 0 | 200 | none |
| mitdb | 24 | 48 | 360 | 48/48 | 48/48 | 1173 (0.0%) | 0 | 200 | none |

Every record listed in each database's RECORDS file loaded without error.

"Samples at recorder limit" are values at the very end of the original recorder's range: the recording itself was clipped there (the original analogue chain saturated). The analyser cannot see this: its own saturation rule looks for values beyond ±32 000 µV, which these recordings (range about ±5 mV for cudb and mitdb, ±10 mV for vfdb) can never reach. No sample needed clipping to fit the analyser's 16-bit input.

10 signal-quality (`~`) marks in cudb carry the text `(N`; the text is ignored and only the quality change is used.

## Headline: device-realistic analysis (12 s limit)

Each cell: result (exact 90 % lower limit, segments, records); ✗ = misses the AHA goal or its lower limit. Inconclusive = share of all scored segments ending "unable to analyse".

| Group | Coarse VF sensitivity (AHA ≥90%, LCL 87%) | Rapid VT sensitivity (≥75%, 67%) | NSR specificity (≥99%, 97%) | Other non-shockable specificity (≥95%, 88%) | Asystole specificity (≥95%, 92%) | Inconclusive |
|---|---|---|---|---|---|---|
| cudb channel 0 | 64.8% (LCL 58.9%, n=128, 14 rec) ✗ | no data | 100.0% (LCL 93.5%, n=34, 3 rec) ✗ | 40.0% (LCL 11.2%, n=5, 1 rec) ✗ | no data | 21.5% (n=483) |
| vfdb channel 0 | 80.0% (LCL 73.5%, n=90, 2 rec) ✗ | no data | 99.3% (LCL 98.6%, n=677, 7 rec) | 99.4% (LCL 98.6%, n=477, 5 rec) | no data | 11.4% (n=1394) |
| mitdb channel 0 | no data | no data | 100.0% (LCL 99.9%, n=2296, 20 rec) | 100.0% (LCL 99.7%, n=839, 11 rec) | no data | 0.3% (n=3135) |
| **pooled, channel 0** | 71.1% (LCL 66.8%, n=218, 16 rec) ✗ | no data | 99.8% (LCL 99.7%, n=3007, 30 rec) | 99.5% (LCL 99.2%, n=1321, 17 rec) | no data | 5.4% (n=5012) |
| vfdb channel 1 | 80.9% (LCL 74.5%, n=89, 2 rec) ✗ | no data | 97.3% (LCL 96.4%, n=677, 7 rec) ✗ | 99.6% (LCL 98.9%, n=477, 5 rec) | no data | 10.3% (n=1394) |
| mitdb channel 1 | no data | no data | 100.0% (LCL 99.9%, n=2198, 20 rec) | 100.0% (LCL 99.7%, n=841, 11 rec) | no data | 0.2% (n=3039) |
| **pooled, channel 1** | 80.9% (LCL 74.5%, n=89, 2 rec) ✗ | no data | 99.4% (LCL 99.1%, n=2875, 27 rec) | 99.8% (LCL 99.6%, n=1318, 16 rec) | no data | 3.4% (n=4433) |

## Time to decision, and shocks lost in the countdown (12 s analysis)

Every second of analysis is hands-off time. "Decided by" = share of segments with a final answer (shock or no shock) by that time after the analysis started; the rest are still analysing. After "shock advised" the rescue sequence counts down for 3 s and abandons the shock on any answer other than `Shockable` (aed-core); "cancelled in countdown" counts shocks the analyser itself would have called off that way, measured by carrying on into the signal that follows (where the same rhythm continues). "Delivered" = shocks not cancelled, over all segments; a shock whose countdown could not be checked counts as delivered.

| Group | Class | n | decided by 7 s | 8 s | 10 s | 12 s | median time to answer (s) | 90th pct (s) | shocks | cancelled in countdown | countdown not checkable | delivered |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| channel 0 | Coarse VF | 218 | 53.7% | 64.2% | 75.2% | 79.8% | 7 | 9 | 155 | 53 | 1 | 46.8% |
| channel 0 | Ventricular flutter | 6 | 100.0% | 100.0% | 100.0% | 100.0% | 7 | 7 | 6 | 0 | 0 | 100.0% |
| channel 0 | VT, rate unknown | 100 | 68.0% | 75.0% | 88.0% | 91.0% | 7 | 9 | 78 | 20 | 0 | 58.0% |
| channel 0 | Normal sinus rhythm | 3007 | 92.6% | 93.4% | 94.1% | 95.1% | 5 | 5 | 5 | 4 | 0 | 0.0% |
| channel 0 | Other non-shockable | 1321 | 99.0% | 99.5% | 99.6% | 99.8% | 5 | 5 | 6 | 3 | 0 | 0.2% |
| channel 1 | Coarse VF | 89 | 73.0% | 83.1% | 92.1% | 93.3% | 7 | 9 | 72 | 18 | 0 | 60.7% |
| channel 1 | Ventricular flutter | 6 | 100.0% | 100.0% | 100.0% | 100.0% | 7 | 7 | 6 | 0 | 0 | 100.0% |
| channel 1 | VT, rate unknown | 100 | 71.0% | 79.0% | 83.0% | 90.0% | 7 | 9 | 76 | 21 | 0 | 55.0% |
| channel 1 | Normal sinus rhythm | 2875 | 92.7% | 93.5% | 94.6% | 95.5% | 5 | 5 | 18 | 13 | 0 | 0.2% |
| channel 1 | Other non-shockable | 1318 | 97.6% | 99.0% | 99.5% | 99.6% | 5 | 5 | 2 | 0 | 0 | 0.2% |

## Segment-level results

Report-only buckets show the share that ended in a shock. "cudb non-VF, rhythm not given" is cudb time outside the annotated VF episodes for which no rhythm label exists; it may contain VT (cudb is a tachyarrhythmia database) and is therefore not counted as a non-shockable class.

### 12 s analysis

No "Rapid VT" row appears: the only VT with beat annotations (mitdb, a few cudb labels) comes in runs too short to hold a whole segment plus the 3 s guard margins, and the few that fit are slow. vfdb's VT has no beat annotations, so its rate is unknown. The rapid-VT goal therefore cannot be tested on these databases with annotation-based rates; see the rate-estimate table in the failure analysis for a rough split.

**cudb channel 0** (483 segments from 16 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 128 | 14 | 83 | 13 | 32 | 64.8% | 58.9% | 59.3% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Normal sinus rhythm | specificity | 34 | 3 | 0 | 32 | 2 | 100.0% | 93.5% | 95.4% | ≥99% (97%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 5 | 1 | 3 | 2 | 0 | 40.0% | 11.2% | 18.0% | ≥95% (88%) | 30 | **FAILS** (n<30) |
| cudb non-VF, rhythm not given | shock rate | 316 | 16 | 0 | 246 | 70 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 128 | 14 | 83 | 13 | 32 | 64.8% | 58.9% | 59.3% | – | – | – |

Inconclusive ("unable to analyse") over all 483 segments: **21.5%**.

**vfdb channel 0** (1394 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 90 | 2 | 72 | 6 | 12 | 80.0% | 73.5% | 74.1% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 6 | 1 | 6 | 0 | 0 | 100.0% | 68.1% | 78.5% | report only | – | report only |
| VT, rate unknown | shock rate | 100 | 7 | 78 | 13 | 9 | 78.0% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 677 | 7 | 5 | 535 | 137 | 99.3% | 98.6% | 98.7% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 477 | 5 | 3 | 473 | 1 | 99.4% | 98.6% | 98.7% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 44 | 1 | 0 | 44 | 0 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 90 | 2 | 72 | 6 | 12 | 80.0% | 73.5% | 74.1% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 100 | 7 | 78 | 13 | 9 | 78.0% | 71.8% | 72.3% | – | – | – |

Inconclusive ("unable to analyse") over all 1394 segments: **11.4%**.

**vfdb channel 1** (1394 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 89 | 2 | 72 | 11 | 6 | 80.9% | 74.5% | 75.0% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 1 | 0 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 6 | 1 | 6 | 0 | 0 | 100.0% | 68.1% | 78.5% | report only | – | report only |
| VT, rate unknown | shock rate | 100 | 7 | 76 | 14 | 10 | 76.0% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 677 | 7 | 18 | 535 | 124 | 97.3% | 96.4% | 96.4% | ≥99% (97%) | 100 | **FAILS** |
| Other non-shockable | specificity | 477 | 5 | 2 | 472 | 3 | 99.6% | 98.9% | 99.0% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 44 | 1 | 0 | 43 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 90 | 2 | 72 | 12 | 6 | 80.0% | 73.5% | 74.1% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 100 | 7 | 76 | 14 | 10 | 76.0% | 69.7% | 70.1% | – | – | – |

Inconclusive ("unable to analyse") over all 1394 segments: **10.3%**.

**mitdb channel 0** (3135 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Normal sinus rhythm | specificity | 2296 | 20 | 0 | 2287 | 9 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 839 | 11 | 0 | 838 | 1 | 100.0% | 99.7% | 99.8% | ≥95% (88%) | 30 | meets |

Inconclusive ("unable to analyse") over all 3135 segments: **0.3%**.

**mitdb channel 1** (3039 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Normal sinus rhythm | specificity | 2198 | 20 | 0 | 2194 | 4 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 841 | 11 | 0 | 839 | 2 | 100.0% | 99.7% | 99.8% | ≥95% (88%) | 30 | meets |

Inconclusive ("unable to analyse") over all 3039 segments: **0.2%**.

**Pooled, channel 0** (5012 segments from 51 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 218 | 16 | 155 | 19 | 44 | 71.1% | 66.8% | 67.0% | ≥90% (87%) | 200 | **FAILS** |
| Ventricular flutter | sensitivity | 6 | 1 | 6 | 0 | 0 | 100.0% | 68.1% | 78.5% | report only | – | report only |
| VT, rate unknown | shock rate | 100 | 7 | 78 | 13 | 9 | 78.0% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 3007 | 30 | 5 | 2854 | 148 | 99.8% | 99.7% | 99.7% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 1321 | 17 | 6 | 1313 | 2 | 99.5% | 99.2% | 99.2% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 316 | 16 | 0 | 246 | 70 | 0.0% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 44 | 1 | 0 | 44 | 0 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 218 | 16 | 155 | 19 | 44 | 71.1% | 66.8% | 67.0% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 100 | 7 | 78 | 13 | 9 | 78.0% | 71.8% | 72.3% | – | – | – |

Inconclusive ("unable to analyse") over all 5012 segments: **5.4%**.

**Pooled, channel 1 (vfdb + mitdb)** (4433 segments from 35 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 89 | 2 | 72 | 11 | 6 | 80.9% | 74.5% | 75.0% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 1 | 0 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 6 | 1 | 6 | 0 | 0 | 100.0% | 68.1% | 78.5% | report only | – | report only |
| VT, rate unknown | shock rate | 100 | 7 | 76 | 14 | 10 | 76.0% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 2875 | 27 | 18 | 2729 | 128 | 99.4% | 99.1% | 99.2% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 1318 | 16 | 2 | 1311 | 5 | 99.8% | 99.6% | 99.6% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 44 | 1 | 0 | 43 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 90 | 2 | 72 | 12 | 6 | 80.0% | 73.5% | 74.1% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 100 | 7 | 76 | 14 | 10 | 76.0% | 69.7% | 70.1% | – | – | – |

Inconclusive ("unable to analyse") over all 4433 segments: **3.4%**.

**Non-shockable rhythms by label, pooled channel 0, 12 s**

| Rhythm label | n | records | shock (false positive) | no shock | inconcl. | specificity | 90% LCL exact |
|---|---|---|---|---|---|---|---|
| AFIB | 731 | 8 | 3 | 728 | 0 | 99.6% | 99.1% |
| AFL | 17 | 2 | 0 | 17 | 0 | 100.0% | 87.3% |
| BI (1st-degree block) | 305 | 2 | 0 | 304 | 1 | 100.0% | 99.2% |
| IVR (idioventricular) | 1 | 1 | 0 | 0 | 1 | 100.0% | 10.0% |
| NOD (junctional) | 2 | 2 | 0 | 2 | 0 | 100.0% | 31.6% |
| NSR | 3007 | 30 | 5 | 2854 | 148 | 99.8% | 99.7% |
| PACED | 104 | 1 | 0 | 104 | 0 | 100.0% | 97.8% |
| SBR | 131 | 1 | 0 | 131 | 0 | 100.0% | 98.3% |
| SVTA | 6 | 2 | 3 | 3 | 0 | 50.0% | 20.1% |
| T (ventricular trigeminy) | 24 | 2 | 0 | 24 | 0 | 100.0% | 90.9% |

**Non-shockable rhythms by label, pooled channel 1, 12 s**

| Rhythm label | n | records | shock (false positive) | no shock | inconcl. | specificity | 90% LCL exact |
|---|---|---|---|---|---|---|---|
| AFIB | 721 | 7 | 0 | 720 | 1 | 100.0% | 99.7% |
| AFL | 24 | 2 | 0 | 23 | 1 | 100.0% | 90.9% |
| BI (1st-degree block) | 305 | 2 | 0 | 303 | 2 | 100.0% | 99.2% |
| IVR (idioventricular) | 1 | 1 | 0 | 1 | 0 | 100.0% | 10.0% |
| NOD (junctional) | 2 | 2 | 0 | 2 | 0 | 100.0% | 31.6% |
| NSR | 2875 | 27 | 18 | 2729 | 128 | 99.4% | 99.1% |
| PACED | 103 | 1 | 0 | 103 | 0 | 100.0% | 97.8% |
| SBR | 132 | 1 | 0 | 132 | 0 | 100.0% | 98.3% |
| SVTA | 6 | 2 | 2 | 3 | 1 | 66.7% | 33.3% |
| T (ventricular trigeminy) | 24 | 2 | 0 | 24 | 0 | 100.0% | 90.9% |

### 8 s analysis

**cudb channel 0** (742 segments from 16 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 197 | 14 | 105 | 15 | 77 | 53.3% | 48.5% | 48.7% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Normal sinus rhythm | specificity | 52 | 3 | 0 | 36 | 16 | 100.0% | 95.7% | 96.9% | ≥99% (97%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 9 | 1 | 3 | 5 | 1 | 66.7% | 40.1% | 45.4% | ≥95% (88%) | 30 | **FAILS** (n<30) |
| cudb non-VF, rhythm not given | shock rate | 484 | 16 | 0 | 349 | 135 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 197 | 14 | 105 | 15 | 77 | 53.3% | 48.5% | 48.7% | – | – | – |

Inconclusive ("unable to analyse") over all 742 segments: **30.9%**.

**vfdb channel 0** (2144 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 139 | 2 | 90 | 6 | 43 | 64.7% | 59.1% | 59.4% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 0 | 1 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 10 | 2 | 8 | 0 | 2 | 80.0% | 55.0% | 60.2% | report only | – | report only |
| VT, rate unknown | shock rate | 157 | 8 | 104 | 21 | 32 | 66.2% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 1047 | 7 | 7 | 774 | 266 | 99.3% | 98.9% | 98.9% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 721 | 5 | 1 | 715 | 5 | 99.9% | 99.5% | 99.5% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 69 | 1 | 0 | 68 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 140 | 2 | 90 | 6 | 44 | 64.3% | 58.6% | 59.0% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 157 | 8 | 104 | 21 | 32 | 66.2% | 61.0% | 61.3% | – | – | – |

Inconclusive ("unable to analyse") over all 2144 segments: **16.3%**.

**mitdb channel 0** (4779 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Normal sinus rhythm | specificity | 3493 | 20 | 0 | 3466 | 27 | 100.0% | 99.9% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 1286 | 11 | 0 | 1283 | 3 | 100.0% | 99.8% | 99.9% | ≥95% (88%) | 30 | meets |

Inconclusive ("unable to analyse") over all 4779 segments: **0.6%**.

**Pooled, channel 0** (7665 segments from 51 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 336 | 16 | 195 | 21 | 120 | 58.0% | 54.4% | 54.6% | ≥90% (87%) | 200 | **FAILS** |
| Fine VF (<200 µV) | shock rate | 1 | 1 | 0 | 0 | 1 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 10 | 2 | 8 | 0 | 2 | 80.0% | 55.0% | 60.2% | report only | – | report only |
| VT, rate unknown | shock rate | 157 | 8 | 104 | 21 | 32 | 66.2% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 4592 | 30 | 7 | 4276 | 309 | 99.8% | 99.7% | 99.8% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 2016 | 17 | 4 | 2003 | 9 | 99.8% | 99.6% | 99.6% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 484 | 16 | 0 | 349 | 135 | 0.0% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 69 | 1 | 0 | 68 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 337 | 16 | 195 | 21 | 121 | 57.9% | 54.2% | 54.4% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 157 | 8 | 104 | 21 | 32 | 66.2% | 61.0% | 61.3% | – | – | – |

Inconclusive ("unable to analyse") over all 7665 segments: **7.9%**.

**Pooled, channel 1 (vfdb + mitdb)** (6773 segments from 35 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 138 | 2 | 98 | 15 | 25 | 71.0% | 65.5% | 65.8% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Fine VF (<200 µV) | shock rate | 2 | 1 | 0 | 0 | 2 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 10 | 2 | 8 | 0 | 2 | 80.0% | 55.0% | 60.2% | report only | – | report only |
| VT, rate unknown | shock rate | 157 | 8 | 104 | 15 | 38 | 66.2% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 4387 | 27 | 14 | 4082 | 291 | 99.7% | 99.5% | 99.6% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 2010 | 16 | 2 | 1982 | 26 | 99.9% | 99.7% | 99.8% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 69 | 1 | 0 | 56 | 13 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 140 | 2 | 98 | 15 | 27 | 70.0% | 64.5% | 64.8% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 157 | 8 | 104 | 15 | 38 | 66.2% | 61.0% | 61.3% | – | – | – |

Inconclusive ("unable to analyse") over all 6773 segments: **5.9%**.

### 7 s analysis

**cudb channel 0** (848 segments from 16 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 223 | 14 | 103 | 15 | 105 | 46.2% | 41.7% | 42.0% | ≥90% (87%) | 200 | **FAILS** |
| Normal sinus rhythm | specificity | 61 | 3 | 0 | 40 | 21 | 100.0% | 96.3% | 97.4% | ≥99% (97%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 10 | 1 | 2 | 5 | 3 | 80.0% | 55.0% | 60.2% | ≥95% (88%) | 30 | **FAILS** (n<30) |
| cudb non-VF, rhythm not given | shock rate | 554 | 16 | 0 | 395 | 159 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 223 | 14 | 103 | 15 | 105 | 46.2% | 41.7% | 42.0% | – | – | – |

Inconclusive ("unable to analyse") over all 848 segments: **34.0%**.

**vfdb channel 0** (2453 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 160 | 2 | 77 | 10 | 73 | 48.1% | 42.8% | 43.1% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 11 | 2 | 9 | 0 | 2 | 81.8% | 58.5% | 63.2% | report only | – | report only |
| VT, rate unknown | shock rate | 179 | 8 | 109 | 23 | 47 | 60.9% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 1198 | 7 | 2 | 861 | 335 | 99.8% | 99.6% | 99.6% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 824 | 5 | 0 | 811 | 13 | 100.0% | 99.7% | 99.8% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 81 | 1 | 0 | 76 | 5 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 160 | 2 | 77 | 10 | 73 | 48.1% | 42.8% | 43.1% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 179 | 8 | 109 | 23 | 47 | 60.9% | 55.9% | 56.1% | – | – | – |

Inconclusive ("unable to analyse") over all 2453 segments: **19.4%**.

**mitdb channel 0** (5485 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Normal sinus rhythm | specificity | 4007 | 20 | 0 | 3970 | 37 | 100.0% | 99.9% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 1478 | 11 | 0 | 1475 | 3 | 100.0% | 99.8% | 99.9% | ≥95% (88%) | 30 | meets |

Inconclusive ("unable to analyse") over all 5485 segments: **0.7%**.

**Pooled, channel 0** (8786 segments from 51 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 383 | 16 | 180 | 25 | 178 | 47.0% | 43.6% | 43.7% | ≥90% (87%) | 200 | **FAILS** |
| Ventricular flutter | sensitivity | 11 | 2 | 9 | 0 | 2 | 81.8% | 58.5% | 63.2% | report only | – | report only |
| VT, rate unknown | shock rate | 179 | 8 | 109 | 23 | 47 | 60.9% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 5266 | 30 | 2 | 4871 | 393 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 2312 | 17 | 2 | 2291 | 19 | 99.9% | 99.8% | 99.8% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 554 | 16 | 0 | 395 | 159 | 0.0% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 81 | 1 | 0 | 76 | 5 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 383 | 16 | 180 | 25 | 178 | 47.0% | 43.6% | 43.7% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 179 | 8 | 109 | 23 | 47 | 60.9% | 55.9% | 56.1% | – | – | – |

Inconclusive ("unable to analyse") over all 8786 segments: **9.1%**.

**Pooled, channel 1 (vfdb + mitdb)** (7769 segments from 35 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 158 | 2 | 96 | 18 | 44 | 60.8% | 55.4% | 55.7% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Fine VF (<200 µV) | shock rate | 2 | 1 | 0 | 0 | 2 | 0.0% | – | – | report only | 25 | report only |
| Ventricular flutter | sensitivity | 11 | 2 | 9 | 0 | 2 | 81.8% | 58.5% | 63.2% | report only | – | report only |
| VT, rate unknown | shock rate | 179 | 8 | 107 | 13 | 59 | 59.8% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 5032 | 27 | 13 | 4646 | 373 | 99.7% | 99.6% | 99.6% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 2306 | 16 | 4 | 2263 | 39 | 99.8% | 99.7% | 99.7% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 81 | 1 | 0 | 62 | 19 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 160 | 2 | 96 | 18 | 46 | 60.0% | 54.7% | 55.0% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 179 | 8 | 107 | 13 | 59 | 59.8% | 54.7% | 55.0% | – | – | – |

Inconclusive ("unable to analyse") over all 7769 segments: **6.9%**.

## Per record (12 s analysis, every channel)

So that one bad patient is visible. VF = coarse VF; "VT/VFL shock" = shocks / segments over VT of any rate and flutter; FP = false shocks on NSR or other non-shockable rhythms. Flags: **VF<50%** (≥5 coarse-VF segments and under half shocked), **FP** (any false shock on an AHA non-shockable class), **INC>50%** (over half of all segments inconclusive).

| Record | ch | lead | coarse VF n | VF Se | VT/VFL shock | NSR n | NSR Sp | other n | other Sp | FP | inconcl. | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cudb/cu01 | 0 | ECG | 24 | 100.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu02 | 0 | ECG | 0 | – | – | 19 | 100.0% | 0 | – | 0 | 24.2% |  |
| cudb/cu04 | 0 | ECG | 20 | 75.0% | – | 0 | – | 0 | – | 0 | 8.6% |  |
| cudb/cu07 | 0 | ECG | 26 | 57.7% | – | 0 | – | 0 | – | 0 | 42.5% |  |
| cudb/cu09 | 0 | ECG | 4 | 50.0% | – | 9 | 100.0% | 5 | 40.0% | 3 | 9.1% | **FP** |
| cudb/cu11 | 0 | ECG | 10 | 30.0% | – | 0 | – | 0 | – | 0 | 75.0% | **VF<50%** **INC>50%** |
| cudb/cu15 | 0 | ECG | 8 | 0.0% | – | 0 | – | 0 | – | 0 | 58.5% | **VF<50%** **INC>50%** |
| cudb/cu16 | 0 | ECG | 7 | 100.0% | – | 6 | 100.0% | 0 | – | 0 | 0.0% |  |
| cudb/cu17 | 0 | ECG | 2 | 50.0% | – | 0 | – | 0 | – | 0 | 2.7% |  |
| cudb/cu21 | 0 | ECG | 2 | 50.0% | – | 0 | – | 0 | – | 0 | 25.0% |  |
| cudb/cu23 | 0 | ECG | 8 | 62.5% | – | 0 | – | 0 | – | 0 | 10.0% |  |
| cudb/cu25 | 0 | ECG | 2 | 50.0% | – | 0 | – | 0 | – | 0 | 2.8% |  |
| cudb/cu26 | 0 | ECG | 4 | 100.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu29 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu30 | 0 | ECG | 8 | 37.5% | – | 0 | – | 0 | – | 0 | 33.3% | **VF<50%** |
| cudb/cu32 | 0 | ECG | 3 | 66.7% | – | 0 | – | 0 | – | 0 | 23.1% |  |
| vfdb/418 | 0 | ECG | 0 | – | – | 116 | 95.7% | 0 | – | 5 | 75.0% | **FP** **INC>50%** |
| vfdb/418 | 1 | ECG | 0 | – | – | 116 | 84.5% | 0 | – | 18 | 72.4% | **FP** **INC>50%** |
| vfdb/421 | 0 | ECG | 0 | – | 0/12 | 84 | 100.0% | 0 | – | 0 | 2.1% |  |
| vfdb/421 | 1 | ECG | 0 | – | 9/12 | 84 | 100.0% | 0 | – | 0 | 2.1% |  |
| vfdb/422 | 0 | ECG | 40 | 75.0% | 13/15 | 87 | 100.0% | 0 | – | 0 | 7.0% |  |
| vfdb/422 | 1 | ECG | 40 | 77.5% | 13/15 | 87 | 100.0% | 0 | – | 0 | 0.7% |  |
| vfdb/423 | 0 | ECG | 0 | – | 35/37 | 82 | 100.0% | 1 | 100.0% | 0 | 4.2% |  |
| vfdb/423 | 1 | ECG | 0 | – | 23/37 | 82 | 100.0% | 1 | 100.0% | 0 | 5.0% |  |
| vfdb/426 | 0 | ECG | 50 | 84.0% | 1/1 | 66 | 100.0% | 4 | 25.0% | 3 | 3.3% | **FP** |
| vfdb/426 | 1 | ECG | 49 | 83.7% | 1/1 | 66 | 100.0% | 4 | 50.0% | 2 | 5.0% | **FP** |
| vfdb/428 | 0 | ECG | 0 | – | 2/2 | 0 | – | 165 | 100.0% | 0 | 0.0% |  |
| vfdb/428 | 1 | ECG | 0 | – | 2/2 | 0 | – | 165 | 100.0% | 0 | 0.6% |  |
| vfdb/429 | 0 | ECG | 0 | – | 6/6 | 0 | – | 140 | 100.0% | 0 | 0.7% |  |
| vfdb/429 | 1 | ECG | 0 | – | 6/6 | 0 | – | 140 | 100.0% | 0 | 0.7% |  |
| vfdb/605 | 0 | ECG | 0 | – | 1/1 | 0 | – | 0 | – | 0 | 0.0% |  |
| vfdb/605 | 1 | ECG | 0 | – | 1/1 | 0 | – | 0 | – | 0 | 0.0% |  |
| vfdb/610 | 0 | ECG | 0 | – | – | 101 | 100.0% | 0 | – | 0 | 0.0% |  |
| vfdb/610 | 1 | ECG | 0 | – | – | 101 | 100.0% | 0 | – | 0 | 0.7% |  |
| vfdb/612 | 0 | ECG | 0 | – | 26/32 | 141 | 100.0% | 0 | – | 0 | 28.9% |  |
| vfdb/612 | 1 | ECG | 0 | – | 27/32 | 141 | 100.0% | 0 | – | 0 | 24.3% |  |
| vfdb/615 | 0 | ECG | 0 | – | – | 0 | – | 167 | 100.0% | 0 | 0.0% |  |
| vfdb/615 | 1 | ECG | 0 | – | – | 0 | – | 167 | 100.0% | 0 | 0.0% |  |
| mitdb/103 | 0 | MLII | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/103 | 1 | V2 | 0 | – | – | 109 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/104 | 0 | V5 | 0 | – | – | 2 | 100.0% | 104 | 100.0% | 0 | 0.0% |  |
| mitdb/104 | 1 | V2 | 0 | – | – | 1 | 100.0% | 103 | 100.0% | 0 | 0.0% |  |
| mitdb/105 | 0 | MLII | 0 | – | – | 106 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/105 | 1 | V1 | 0 | – | – | 88 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/114 | 0 | V5 | 0 | – | – | 142 | 100.0% | 0 | – | 0 | 6.3% |  |
| mitdb/114 | 1 | MLII | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/115 | 0 | MLII | 0 | – | – | 146 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/115 | 1 | V1 | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/116 | 0 | MLII | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/116 | 1 | V1 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/118 | 0 | MLII | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/118 | 1 | V1 | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/119 | 0 | MLII | 0 | – | – | 75 | 100.0% | 3 | 100.0% | 0 | 0.0% |  |
| mitdb/119 | 1 | V1 | 0 | – | – | 73 | 100.0% | 3 | 100.0% | 0 | 0.0% |  |
| mitdb/121 | 0 | MLII | 0 | – | – | 146 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/121 | 1 | V1 | 0 | – | – | 145 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/122 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/122 | 1 | V1 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/124 | 0 | MLII | 0 | – | – | 138 | 100.0% | 2 | 100.0% | 0 | 0.7% |  |
| mitdb/124 | 1 | V4 | 0 | – | – | 137 | 100.0% | 2 | 100.0% | 0 | 0.0% |  |
| mitdb/201 | 0 | MLII | 0 | – | – | 54 | 100.0% | 69 | 100.0% | 0 | 0.0% |  |
| mitdb/201 | 1 | V1 | 0 | – | – | 54 | 100.0% | 69 | 100.0% | 0 | 0.0% |  |
| mitdb/202 | 0 | MLII | 0 | – | – | 94 | 100.0% | 48 | 100.0% | 0 | 0.0% |  |
| mitdb/202 | 1 | V1 | 0 | – | – | 94 | 100.0% | 48 | 100.0% | 0 | 0.0% |  |
| mitdb/203 | 0 | MLII | 0 | – | – | 0 | – | 102 | 100.0% | 0 | 0.0% |  |
| mitdb/203 | 1 | V1 | 0 | – | – | 0 | – | 109 | 100.0% | 0 | 1.8% |  |
| mitdb/205 | 0 | MLII | 0 | – | – | 143 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/205 | 1 | V1 | 0 | – | – | 141 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/209 | 0 | MLII | 0 | – | – | 129 | 100.0% | 2 | 100.0% | 0 | 0.0% |  |
| mitdb/209 | 1 | V1 | 0 | – | – | 95 | 100.0% | 2 | 100.0% | 0 | 0.0% |  |
| mitdb/210 | 0 | MLII | 0 | – | – | 0 | – | 135 | 100.0% | 0 | 0.0% |  |
| mitdb/210 | 1 | V1 | 0 | – | – | 0 | – | 134 | 100.0% | 0 | 0.0% |  |
| mitdb/213 | 0 | MLII | 0 | – | – | 127 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/213 | 1 | V1 | 0 | – | – | 127 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/214 | 0 | MLII | 0 | – | – | 128 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/214 | 1 | V1 | 0 | – | – | 128 | 100.0% | 0 | – | 0 | 3.1% |  |
| mitdb/215 | 0 | MLII | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/215 | 1 | V1 | 0 | – | – | 134 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/219 | 0 | MLII | 0 | – | – | 23 | 100.0% | 110 | 100.0% | 0 | 0.0% |  |
| mitdb/219 | 1 | V1 | 0 | – | – | 23 | 100.0% | 110 | 100.0% | 0 | 0.0% |  |
| mitdb/221 | 0 | MLII | 0 | – | – | 0 | – | 133 | 100.0% | 0 | 0.0% |  |
| mitdb/221 | 1 | V1 | 0 | – | – | 0 | – | 129 | 100.0% | 0 | 0.0% |  |
| mitdb/232 | 0 | MLII | 0 | – | – | 0 | – | 131 | 100.0% | 0 | 0.0% |  |
| mitdb/232 | 1 | V1 | 0 | – | – | 0 | – | 132 | 100.0% | 0 | 0.0% |  |
| mitdb/233 | 0 | MLII | 0 | – | – | 113 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/233 | 1 | V1 | 0 | – | – | 113 | 100.0% | 0 | – | 0 | 0.0% |  |

## Continuous (streaming) run

One analyser runs over each whole record and channel without resets, deciding once a second. Each per-second decision is scored against the truth when its last 6 s (the 4 s window plus the two earlier windows needed for confirmation) lie inside one clean run (same guard margins as above). The decision scored is what the analyser reported for that window (`Shockable` only once confirmed; an unconfirmed shockable window counts as "not sure").

| Database | ch | Class | seconds | Shockable | NonShockable | Not sure |
|---|---|---|---|---|---|---|
| cudb | 0 | Coarse VF | 1512 | 51.1% | 2.6% | 46.4% |
| cudb | 0 | Normal sinus rhythm | 404 | 0.0% | 48.3% | 51.7% |
| cudb | 0 | Other non-shockable | 63 | 33.3% | 50.8% | 15.9% |
| cudb | 0 | cudb non-VF, rhythm not given | 3822 | 0.0% | 62.7% | 37.3% |
| vfdb | 0 | Coarse VF | 1102 | 58.3% | 2.1% | 39.7% |
| vfdb | 0 | Ventricular flutter | 75 | 93.3% | 0.0% | 6.7% |
| vfdb | 0 | VT, rate unknown | 1224 | 62.7% | 8.4% | 28.8% |
| vfdb | 0 | Normal sinus rhythm | 8198 | 0.7% | 65.7% | 33.6% |
| vfdb | 0 | Other non-shockable | 5713 | 0.2% | 95.2% | 4.6% |
| vfdb | 0 | HGEA (vfdb) | 526 | 0.2% | 75.1% | 24.7% |
| vfdb | 1 | Coarse VF | 1091 | 69.8% | 4.8% | 25.4% |
| vfdb | 1 | Fine VF (<200 µV) | 11 | 0.0% | 18.2% | 81.8% |
| vfdb | 1 | Ventricular flutter | 75 | 93.3% | 0.0% | 6.7% |
| vfdb | 1 | VT, rate unknown | 1224 | 62.4% | 3.8% | 33.8% |
| vfdb | 1 | Normal sinus rhythm | 8198 | 1.5% | 67.5% | 31.1% |
| vfdb | 1 | Other non-shockable | 5713 | 0.2% | 91.9% | 7.9% |
| vfdb | 1 | HGEA (vfdb) | 526 | 0.2% | 47.9% | 51.9% |
| mitdb | 0 | Normal sinus rhythm | 27550 | 0.0% | 98.3% | 1.7% |
| mitdb | 0 | Other non-shockable | 10053 | 0.0% | 99.2% | 0.8% |
| mitdb | 1 | Normal sinus rhythm | 26325 | 0.0% | 96.2% | 3.8% |
| mitdb | 1 | Other non-shockable | 10087 | 0.0% | 95.0% | 5.0% |

### Detection delay after VF / flutter onset

Measured from the annotated onset (cudb onsets are only approximate, per its documentation) to the first confirmed `Shockable` before the episode ends. The fastest possible is 7 s after the window first lies wholly in VF, i.e. about 6–7 s after onset. Episodes shorter than 7 s are counted apart. A delay under 6 s is faster than the analyser can confirm VF from scratch: it means the windows before the annotated onset already looked shockable to it (in vfdb record 418 the "NSR" between flutter bursts does). Noise marks are not removed here.

| Database | ch | label | episodes ≥7 s | detected | not detected | already shockable at onset | delay (detected) | of which under 6 s | episodes <7 s |
|---|---|---|---|---|---|---|---|---|---|
| cudb | 0 | VF | 27 | 24 | 3 | 0 | median 5.3 s, 90th pct 10.3 s, max 126.7 s | 17 | 0 |
| vfdb | 0 | VF | 5 | 4 | 1 | 0 | median 6.2 s, 90th pct 10.0 s, max 10.8 s | 2 | 1 |
| vfdb | 0 | VFL | 13 | 9 | 0 | 4 | median 4.4 s, 90th pct 4.8 s, max 4.8 s | 9 | 49 |
| vfdb | 1 | VF | 5 | 4 | 1 | 0 | median 5.6 s, 90th pct 7.7 s, max 8.1 s | 2 | 1 |
| vfdb | 1 | VFL | 13 | 9 | 0 | 4 | median 3.9 s, 90th pct 4.5 s, max 4.8 s | 9 | 49 |

### How long "shock" persists after a shockable rhythm ends

For VF, flutter and VT episodes followed by at least 10 s of a non-shockable (or cudb unlabelled) rhythm: the time from the annotated end to the first decision that is not `Shockable`. Zero means the analyser was not saying `Shockable` at the end. In cudb the end of VF is often a defibrillation shock, so the following seconds are shock artefact.

| Database | ch | episodes | shockable at end | persistence (those) | false shock 6–16 s after the end |
|---|---|---|---|---|---|
| cudb | 0 | 22 | 6 | median 0.7 s, 90th pct 0.8 s, max 0.8 s | 0 |
| vfdb | 0 | 43 | 22 | median 2.8 s, 90th pct 3.2 s, max 3.6 s | 5 |
| vfdb | 1 | 43 | 23 | median 2.8 s, 90th pct 3.2 s, max 3.8 s | 8 |
| mitdb | 0 | 34 | 0 | – | 0 |
| mitdb | 1 | 34 | 0 | – | 0 |

### Share of VF/flutter episodes reported `Shockable`, by seconds from onset (channel 0, episodes ≥ 20 s)

| Database | episodes | -5 s | +0 s | +2 s | +4 s | +5 s | +6 s | +7 s | +8 s | +9 s | +10 s | +12 s | +15 s | +20 s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cudb | 22 | 4.5% | 0.0% | 4.5% | 9.1% | 31.8% | 59.1% | 59.1% | 59.1% | 54.5% | 54.5% | 54.5% | 50.0% | 40.9% |
| vfdb | 5 | 0.0% | 0.0% | 0.0% | 40.0% | 60.0% | 40.0% | 40.0% | 40.0% | 60.0% | 60.0% | 60.0% | 40.0% | 80.0% |

## Failure analysis

Nothing below was used to change the analyser. Where a threshold is "moved", it is a what-if count only.

### Which rule gave each window's verdict (streaming windows, all databases and channels)

| Class | windows | invalid measurement | saturated | clipped (flat tops) | amplitude implausible | asystole | noisy | possible artifact | fine VF (not shocked) | methods disagree | both methods shockable | neither method shockable |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.0% | 0.0% | 4.0% | 0.0% | 0.0% | 0.0% | 6.4% | 0.1% | 21.1% | 65.4% | 3.1% |
| Fine VF (<200 µV) | 11 | 0.0% | 0.0% | 0.0% | 0.0% | 18.2% | 0.0% | 0.0% | 9.1% | 63.6% | 9.1% | 0.0% |
| Ventricular flutter | 150 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 1.3% | 0.0% | 5.3% | 93.3% | 0.0% |
| VT, rate unknown | 2448 | 0.0% | 0.0% | 0.0% | 0.4% | 0.2% | 0.1% | 3.1% | 0.0% | 22.3% | 68.1% | 5.9% |
| Normal sinus rhythm | 70675 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.1% | 8.0% | 0.0% | 1.6% | 0.4% | 89.9% |
| Other non-shockable | 31629 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 1.7% | 0.0% | 2.3% | 0.2% | 95.7% |
| cudb non-VF, rhythm not given | 3822 | 0.0% | 0.0% | 0.6% | 0.0% | 0.0% | 0.0% | 21.7% | 0.0% | 15.0% | 0.1% | 62.7% |
| HGEA (vfdb) | 1052 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 22.1% | 0.0% | 15.6% | 0.8% | 61.5% |

Exclusion rules by database (share of all scored streaming windows, any class):

| Database | windows | saturated | implausible size | flatline | noisy | low-band (compression) | fine VF |
|---|---|---|---|---|---|---|---|
| cudb | 5801 | 0.0% | 0.0% | 0.0% | 0.0% | 19.8% | 0.0% |
| vfdb | 33676 | 0.0% | 0.1% | 0.0% | 0.1% | 16.6% | 0.0% |
| mitdb | 74015 | 0.0% | 0.0% | 0.0% | 0.0% | 1.2% | 0.0% |

### Feature distributions on real data versus the thresholds

Streaming windows, all databases and channels, by true class. "VF side" = share of windows on the shockable side of that feature's threshold(s). A feature only helps if VF windows are mostly on the VF side and non-shockable windows mostly are not.

**peak-to-peak (µV, after the analyser's filter)** (shockable side: ≥ 200 for shock, < 100 = flatline, > 12000 = implausible)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 423.183 | 603.537 | 1368.049 | 2077.036 | 5846.219 | 97.9% |
| Fine VF (<200 µV) | 11 | 131.995 | 162.390 | 169.566 | 194.287 | 221.161 | 18.2% |
| Ventricular flutter | 150 | 491.337 | 588.467 | 820.154 | 1010.632 | 1327.917 | 100.0% |
| VT, rate unknown | 2448 | 494.178 | 681.319 | 1164.920 | 2176.612 | 4315.403 | 99.4% |
| Normal sinus rhythm | 70675 | 498.451 | 970.332 | 1786.738 | 2612.079 | 4055.822 | 100.0% |
| Other non-shockable | 31629 | 477.201 | 813.999 | 1295.469 | 2168.625 | 6261.729 | 99.9% |
| cudb non-VF, rhythm not given | 3822 | 1914.856 | 2458.447 | 2786.227 | 3272.415 | 5068.181 | 100.0% |
| HGEA (vfdb) | 1052 | 3453.102 | 4904.030 | 5753.817 | 6450.496 | 7323.161 | 100.0% |

**root-mean-square (µV, after the analyser's filter)** (shockable side: ≥ 25 (or leakage ≤ 0.6) to escape the flatline rule)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 78.251 | 112.751 | 270.631 | 414.881 | 1137.572 | 100.0% |
| Fine VF (<200 µV) | 11 | 24.117 | 26.533 | 29.514 | 31.340 | 35.351 | 81.8% |
| Ventricular flutter | 150 | 94.527 | 149.571 | 176.867 | 261.745 | 308.054 | 100.0% |
| VT, rate unknown | 2448 | 92.275 | 128.187 | 247.023 | 448.503 | 790.605 | 99.8% |
| Normal sinus rhythm | 70675 | 63.501 | 138.799 | 232.534 | 370.156 | 568.250 | 100.0% |
| Other non-shockable | 31629 | 69.551 | 130.225 | 184.049 | 280.393 | 737.245 | 100.0% |
| cudb non-VF, rhythm not given | 3822 | 213.360 | 343.463 | 498.749 | 617.879 | 850.266 | 100.0% |
| HGEA (vfdb) | 1052 | 514.599 | 656.002 | 783.690 | 945.031 | 1230.701 | 100.0% |

**noise ratio** (shockable side: ≤ 0.15)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.000 | 0.001 | 0.005 | 0.009 | 0.021 | 100.0% |
| Fine VF (<200 µV) | 11 | 0.081 | 0.096 | 0.109 | 0.145 | 0.181 | 81.8% |
| Ventricular flutter | 150 | 0.001 | 0.001 | 0.002 | 0.002 | 0.005 | 100.0% |
| VT, rate unknown | 2448 | 0.001 | 0.002 | 0.005 | 0.008 | 0.013 | 99.7% |
| Normal sinus rhythm | 70675 | 0.002 | 0.006 | 0.012 | 0.029 | 0.066 | 99.9% |
| Other non-shockable | 31629 | 0.003 | 0.005 | 0.008 | 0.013 | 0.032 | 100.0% |
| cudb non-VF, rhythm not given | 3822 | 0.000 | 0.002 | 0.004 | 0.008 | 0.022 | 100.0% |
| HGEA (vfdb) | 1052 | 0.001 | 0.002 | 0.002 | 0.004 | 0.012 | 100.0% |

**low-band fraction** (shockable side: ≤ 0.5)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.006 | 0.026 | 0.074 | 0.203 | 0.620 | 91.8% |
| Fine VF (<200 µV) | 11 | 0.036 | 0.088 | 0.145 | 0.204 | 0.232 | 100.0% |
| Ventricular flutter | 150 | 0.006 | 0.014 | 0.021 | 0.055 | 0.325 | 98.7% |
| VT, rate unknown | 2448 | 0.004 | 0.016 | 0.051 | 0.154 | 0.411 | 96.9% |
| Normal sinus rhythm | 70675 | 0.032 | 0.085 | 0.155 | 0.263 | 0.554 | 92.0% |
| Other non-shockable | 31629 | 0.071 | 0.132 | 0.217 | 0.311 | 0.434 | 98.2% |
| cudb non-VF, rhythm not given | 3822 | 0.046 | 0.139 | 0.243 | 0.492 | 0.700 | 78.0% |
| HGEA (vfdb) | 1052 | 0.105 | 0.229 | 0.338 | 0.481 | 0.611 | 77.9% |

**threshold-crossing fraction (TCSC)** (shockable side: > 0.45)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.553 | 0.625 | 0.665 | 0.708 | 0.786 | 99.2% |
| Fine VF (<200 µV) | 11 | 0.538 | 0.554 | 0.572 | 0.634 | 0.650 | 100.0% |
| Ventricular flutter | 150 | 0.638 | 0.733 | 0.817 | 0.850 | 0.868 | 100.0% |
| VT, rate unknown | 2448 | 0.554 | 0.637 | 0.681 | 0.728 | 0.806 | 97.5% |
| Normal sinus rhythm | 70675 | 0.054 | 0.129 | 0.208 | 0.338 | 0.608 | 13.0% |
| Other non-shockable | 31629 | 0.133 | 0.244 | 0.330 | 0.421 | 0.542 | 17.7% |
| cudb non-VF, rhythm not given | 3822 | 0.149 | 0.230 | 0.416 | 0.572 | 0.627 | 46.4% |
| HGEA (vfdb) | 1052 | 0.289 | 0.367 | 0.425 | 0.488 | 0.579 | 40.4% |

**band-pass count** (shockable side: > 0.16)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.114 | 0.167 | 0.202 | 0.237 | 0.287 | 78.2% |
| Fine VF (<200 µV) | 11 | 0.172 | 0.174 | 0.183 | 0.190 | 0.228 | 100.0% |
| Ventricular flutter | 150 | 0.202 | 0.270 | 0.334 | 0.382 | 0.454 | 100.0% |
| VT, rate unknown | 2448 | 0.099 | 0.166 | 0.207 | 0.252 | 0.357 | 78.1% |
| Normal sinus rhythm | 70675 | 0.030 | 0.043 | 0.053 | 0.069 | 0.106 | 3.9% |
| Other non-shockable | 31629 | 0.029 | 0.050 | 0.064 | 0.075 | 0.095 | 0.3% |
| cudb non-VF, rhythm not given | 3822 | 0.034 | 0.054 | 0.078 | 0.128 | 0.152 | 2.8% |
| HGEA (vfdb) | 1052 | 0.045 | 0.059 | 0.074 | 0.093 | 0.124 | 0.8% |

**VF-filter leakage** (shockable side: < 0.625)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.272 | 0.403 | 0.486 | 0.568 | 0.739 | 85.6% |
| Fine VF (<200 µV) | 11 | 0.591 | 0.653 | 0.702 | 0.762 | 0.796 | 18.2% |
| Ventricular flutter | 150 | 0.239 | 0.281 | 0.326 | 0.423 | 0.623 | 94.7% |
| VT, rate unknown | 2448 | 0.253 | 0.372 | 0.463 | 0.562 | 0.717 | 86.4% |
| Normal sinus rhythm | 70675 | 0.605 | 0.740 | 0.789 | 0.830 | 0.873 | 6.0% |
| Other non-shockable | 31629 | 0.633 | 0.692 | 0.743 | 0.788 | 0.844 | 3.5% |
| cudb non-VF, rhythm not given | 3822 | 0.551 | 0.640 | 0.741 | 0.794 | 0.835 | 22.0% |
| HGEA (vfdb) | 1052 | 0.533 | 0.618 | 0.683 | 0.737 | 0.791 | 28.1% |

**dominant frequency (Hz)** (shockable side: > 2.4 and < 9)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 1.500 | 3.000 | 4.000 | 5.250 | 6.500 | 83.7% |
| Fine VF (<200 µV) | 11 | 3.500 | 8.125 | 8.250 | 8.500 | 8.750 | 90.9% |
| Ventricular flutter | 150 | 3.250 | 3.500 | 3.750 | 3.750 | 3.750 | 96.0% |
| VT, rate unknown | 2448 | 1.250 | 3.750 | 4.000 | 5.000 | 5.750 | 91.3% |
| Normal sinus rhythm | 70675 | 1.250 | 1.750 | 2.750 | 3.750 | 6.500 | 59.6% |
| Other non-shockable | 31629 | 1.250 | 1.750 | 2.500 | 3.000 | 6.000 | 59.8% |
| cudb non-VF, rhythm not given | 3822 | 1.250 | 1.750 | 2.250 | 3.750 | 6.500 | 48.9% |
| HGEA (vfdb) | 1052 | 1.500 | 2.000 | 2.250 | 2.750 | 3.250 | 37.8% |

**spectral concentration** (shockable side: > 0.4)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.374 | 0.607 | 0.748 | 0.856 | 0.935 | 93.9% |
| Fine VF (<200 µV) | 11 | 0.321 | 0.497 | 0.556 | 0.661 | 0.765 | 90.9% |
| Ventricular flutter | 150 | 0.658 | 0.831 | 0.897 | 0.914 | 0.939 | 100.0% |
| VT, rate unknown | 2448 | 0.348 | 0.644 | 0.788 | 0.876 | 0.943 | 93.2% |
| Normal sinus rhythm | 70675 | 0.135 | 0.223 | 0.305 | 0.406 | 0.562 | 26.3% |
| Other non-shockable | 31629 | 0.155 | 0.260 | 0.344 | 0.412 | 0.545 | 29.1% |
| cudb non-VF, rhythm not given | 3822 | 0.150 | 0.314 | 0.462 | 0.514 | 0.633 | 62.6% |
| HGEA (vfdb) | 1052 | 0.230 | 0.400 | 0.558 | 0.657 | 0.760 | 74.9% |

**saturated fraction (front-end limit)** (shockable side: ≤ 0.01)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Fine VF (<200 µV) | 11 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Ventricular flutter | 150 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| VT, rate unknown | 2448 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Normal sinus rhythm | 70675 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Other non-shockable | 31629 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| cudb non-VF, rhythm not given | 3822 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| HGEA (vfdb) | 1052 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |

**clipped fraction (flat tops)** (shockable side: ≤ 0.01)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3705 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 96.0% |
| Fine VF (<200 µV) | 11 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Ventricular flutter | 150 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| VT, rate unknown | 2448 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Normal sinus rhythm | 70675 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Other non-shockable | 31629 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| cudb non-VF, rhythm not given | 3822 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 99.4% |
| HGEA (vfdb) | 1052 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |

### Missed coarse VF (12 s analyses, all channels)

80 of 307 coarse-VF segments did not end in a shock: 30 ended "no shock", 50 inconclusive.

How the missed ones ended (the verdict of the window that ended the analysis, or for inconclusive ones, the most common verdict among its windows):

| Ended by | segments |
|---|---|
| clipped (flat tops) | 8 |
| methods disagree | 29 |
| neither method shockable | 30 |
| possible artifact | 13 |

Which conditions failed, over 507 windows of missed coarse-VF segments (15.0% passed all of them). "Only blocker" = the window failed that condition and no other, so that single threshold alone kept it from being shockable.

| Condition | rule | failed (any) | only blocker |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% | 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 0.01 | 12.8% | 0.0% |
| plausible size | peak-to-peak ≤ 12000 µV | 7.9% | 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 25 µV with leakage > 0.6) | 0.0% | 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.0% | 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 28.0% | 0.2% |
| TCSC | threshold-crossing fraction > 0.45 | 2.8% | 0.0% |
| band-pass count | band-pass count > 0.16 | 48.5% | 21.3% |
| VF-filter leakage | leakage < 0.625 | 41.2% | 10.1% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 44.8% | 2.6% |
| dominant freq. high limit | dominant frequency < 9 Hz | 2.8% | 2.8% |
| spectral concentration | concentration > 0.4 | 11.4% | 0.6% |
| not fine VF | peak-to-peak ≥ 200 µV | 1.0% | 0.4% |

Which conditions failed, over 3705 all coarse-VF streaming windows (65.4% passed all of them). "Only blocker" = the window failed that condition and no other, so that single threshold alone kept it from being shockable.

| Condition | rule | failed (any) | only blocker |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% | 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 0.01 | 4.0% | 0.1% |
| plausible size | peak-to-peak ≤ 12000 µV | 2.0% | 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 25 µV with leakage > 0.6) | 0.0% | 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.0% | 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 8.2% | 0.0% |
| TCSC | threshold-crossing fraction > 0.45 | 0.8% | 0.0% |
| band-pass count | band-pass count > 0.16 | 21.8% | 12.0% |
| VF-filter leakage | leakage < 0.625 | 14.4% | 3.2% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 15.9% | 1.5% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.4% | 0.4% |
| spectral concentration | concentration > 0.4 | 6.1% | 0.6% |
| not fine VF | peak-to-peak ≥ 200 µV | 0.2% | 0.1% |

### False shocks on non-shockable rhythms (12 s analyses, all channels)

Worst rhythm classes by false-shock rate:

| Rhythm | false shocks | segments | rate |
|---|---|---|---|
| SVTA | 5 | 12 | 41.7% |
| NSR | 23 | 5882 | 0.4% |
| AFIB | 3 | 1452 | 0.2% |
| AFL | 0 | 41 | 0.0% |
| BI (1st-degree block) | 0 | 610 | 0.0% |
| IVR (idioventricular) | 0 | 2 | 0.0% |
| NOD (junctional) | 0 | 4 | 0.0% |
| PACED | 0 | 207 | 0.0% |
| SBR | 0 | 263 | 0.0% |
| T (ventricular trigeminy) | 0 | 48 | 0.0% |

Every false shock passed all 13 conditions in at least 3 of the last 5 windows (that is the only way to reach `Shockable`), so "which rule fired" is always "both method families said shockable". The useful question is which condition was closest to stopping it. For each condition: the share of the confirming windows of every false shock that would have been stopped had that one threshold been 10 % / 20 % stricter, against the share of correctly shocked coarse-VF confirming windows that the same change would also stop (the cost). What-if counts only; nothing was changed.

| Condition | rule | false-shock windows stopped at 10% / 20% stricter (of 93) | coarse-VF windows also stopped at 10% / 20% (of 681) |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% / 0.0% | 0.0% / 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 0.01 | 0.0% / 0.0% | 0.0% / 0.0% |
| plausible size | peak-to-peak ≤ 12000 µV | 0.0% / 0.0% | 0.0% / 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 25 µV with leakage > 0.6) | 0.0% / 0.0% | 0.0% / 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.0% / 0.0% | 0.0% / 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 4.3% / 16.1% | 0.3% / 0.7% |
| TCSC | threshold-crossing fraction > 0.45 | 0.0% / 0.0% | 0.0% / 0.1% |
| band-pass count | band-pass count > 0.16 | 23.7% / 38.7% | 12.5% / 25.4% |
| VF-filter leakage | leakage < 0.625 | 6.5% / 30.1% | 9.0% / 32.9% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 38.7% / 75.3% | 1.2% / 6.3% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.0% / 0.0% | 0.4% / 2.2% |
| spectral concentration | concentration > 0.4 | 1.1% / 1.1% | 0.7% / 2.3% |
| not fine VF | peak-to-peak ≥ 200 µV | 0.0% / 0.0% | 0.0% / 0.0% |

How often each condition already lets a non-shockable window through (streaming NSR + other non-shockable windows, n = 102304): a condition passed by nearly every non-shockable window does no work.

| Condition | non-shockable windows on the VF side | coarse-VF windows on the VF side |
|---|---|---|
| not saturated | 100.0% | 100.0% |
| not clipped (flat tops) | 100.0% | 96.0% |
| plausible size | 100.0% | 98.0% |
| not flatline | 100.0% | 100.0% |
| not noisy | 100.0% | 100.0% |
| not low-band (compression) | 93.9% | 91.8% |
| TCSC | 14.4% | 99.2% |
| band-pass count | 2.8% | 78.2% |
| VF-filter leakage | 5.2% | 85.6% |
| dominant freq. low limit | 59.8% | 84.1% |
| dominant freq. high limit | 99.8% | 99.6% |
| spectral concentration | 27.1% | 93.9% |
| not fine VF | 100.0% | 99.8% |

False shocks by record and channel (31 in all):

| Record | false shocks | on rhythm(s) |
|---|---|---|
| vfdb/418 ch1 | 18 | NSR |
| vfdb/418 ch0 | 5 | NSR |
| cudb/cu09 ch0 | 3 | AFIB |
| vfdb/426 ch0 | 3 | SVTA |
| vfdb/426 ch1 | 2 | SVTA |

### VT without beat annotations, split by a rough rate estimated from the signal

vfdb gives VT episodes but no beats, so the AHA rate criterion cannot be applied from the annotations. As a rough guide only, each segment's rate was estimated from the signal's strongest repetition (autocorrelation, 60–300/min). This helper is UNVERIFIED and can be off by a factor of two; it is not used for any AHA row above. 12 s analyses, both channels counted.

| Estimated rate | segments | records | shock | no shock | inconcl. | shock rate |
|---|---|---|---|---|---|---|
| ≥ 150/min (would be rapid VT) | 182 | 7 | 142 | 23 | 17 | 78.0% |
| < 150/min (would be slow VT) | 11 | 4 | 9 | 1 | 1 | 81.8% |
| no clear rate | 7 | 1 | 3 | 3 | 1 | 42.9% |

### Every false shock (31 in all, both channels)

Normal rhythm first, then asystole, then by how fast the shock came. Features are those of the window that confirmed the shock.

| Record | ch | start (s) | true rhythm | amplitude (µV) | rate | outcome | at (s) | window verdict | features |
|---|---|---|---|---|---|---|---|---|---|
| vfdb/418 | 0 | 657.1 | NSR | 1731 | – | shock | 7 | both methods shockable | pp 2338 µV, noise 0.00, low-band 0.17, TCSC 0.80, count 0.21, leak 0.42, dom 3.25 Hz, conc 0.87 |
| vfdb/418 | 0 | 957.2 | NSR | 1899 | – | shock | 7 | both methods shockable | pp 2393 µV, noise 0.00, low-band 0.21, TCSC 0.80, count 0.19, leak 0.40, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 0 | 981.2 | NSR | 1625 | – | shock | 7 | both methods shockable | pp 2049 µV, noise 0.00, low-band 0.42, TCSC 0.83, count 0.17, leak 0.45, dom 2.50 Hz, conc 0.71 |
| vfdb/418 | 1 | 657.1 | NSR | 696 | – | shock | 7 | both methods shockable | pp 901 µV, noise 0.00, low-band 0.18, TCSC 0.76, count 0.23, leak 0.45, dom 3.25 Hz, conc 0.86 |
| vfdb/418 | 1 | 805.5 | NSR | 820 | – | shock | 7 | both methods shockable | pp 1031 µV, noise 0.00, low-band 0.15, TCSC 0.70, count 0.22, leak 0.41, dom 2.75 Hz, conc 0.85 |
| vfdb/418 | 1 | 841.5 | NSR | 800 | – | shock | 7 | both methods shockable | pp 947 µV, noise 0.00, low-band 0.15, TCSC 0.74, count 0.17, leak 0.44, dom 3.25 Hz, conc 0.88 |
| vfdb/418 | 1 | 957.2 | NSR | 771 | – | shock | 7 | both methods shockable | pp 1039 µV, noise 0.00, low-band 0.21, TCSC 0.77, count 0.25, leak 0.40, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 1 | 981.2 | NSR | 742 | – | shock | 7 | both methods shockable | pp 871 µV, noise 0.00, low-band 0.38, TCSC 0.77, count 0.16, leak 0.48, dom 2.50 Hz, conc 0.71 |
| vfdb/418 | 1 | 993.2 | NSR | 720 | – | shock | 7 | both methods shockable | pp 945 µV, noise 0.00, low-band 0.24, TCSC 0.75, count 0.23, leak 0.47, dom 2.75 Hz, conc 0.77 |
| vfdb/418 | 0 | 445.0 | NSR | 1921 | – | shock | 8 | both methods shockable | pp 2280 µV, noise 0.00, low-band 0.46, TCSC 0.79, count 0.21, leak 0.41, dom 2.50 Hz, conc 0.86 |
| vfdb/418 | 0 | 993.2 | NSR | 1710 | – | shock | 8 | both methods shockable | pp 2463 µV, noise 0.00, low-band 0.30, TCSC 0.80, count 0.18, leak 0.46, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 1 | 39.1 | NSR | 639 | – | shock | 8 | both methods shockable | pp 714 µV, noise 0.00, low-band 0.43, TCSC 0.80, count 0.20, leak 0.55, dom 3.00 Hz, conc 0.59 |
| vfdb/418 | 1 | 445.0 | NSR | 761 | – | shock | 8 | both methods shockable | pp 959 µV, noise 0.00, low-band 0.46, TCSC 0.76, count 0.27, leak 0.41, dom 2.50 Hz, conc 0.85 |
| vfdb/418 | 1 | 817.5 | NSR | 727 | – | shock | 8 | both methods shockable | pp 1052 µV, noise 0.00, low-band 0.28, TCSC 0.78, count 0.25, leak 0.46, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 1 | 969.2 | NSR | 700 | – | shock | 8 | both methods shockable | pp 991 µV, noise 0.00, low-band 0.31, TCSC 0.74, count 0.22, leak 0.45, dom 2.50 Hz, conc 0.71 |
| vfdb/418 | 1 | 75.1 | NSR | 728 | – | shock | 9 | both methods shockable | pp 762 µV, noise 0.00, low-band 0.24, TCSC 0.81, count 0.22, leak 0.47, dom 3.00 Hz, conc 0.76 |
| vfdb/418 | 1 | 549.5 | NSR | 751 | – | shock | 9 | both methods shockable | pp 908 µV, noise 0.00, low-band 0.39, TCSC 0.82, count 0.18, leak 0.47, dom 2.50 Hz, conc 0.68 |
| vfdb/418 | 1 | 561.5 | NSR | 724 | – | shock | 9 | both methods shockable | pp 864 µV, noise 0.00, low-band 0.40, TCSC 0.76, count 0.21, leak 0.54, dom 2.50 Hz, conc 0.67 |
| vfdb/418 | 1 | 573.5 | NSR | 784 | – | shock | 10 | both methods shockable | pp 820 µV, noise 0.00, low-band 0.29, TCSC 0.79, count 0.24, leak 0.50, dom 3.00 Hz, conc 0.76 |
| vfdb/418 | 1 | 27.1 | NSR | 671 | – | shock | 11 | both methods shockable | pp 880 µV, noise 0.00, low-band 0.30, TCSC 0.76, count 0.18, leak 0.46, dom 2.75 Hz, conc 0.83 |
| vfdb/418 | 1 | 829.5 | NSR | 810 | – | shock | 11 | both methods shockable | pp 943 µV, noise 0.00, low-band 0.22, TCSC 0.83, count 0.24, leak 0.45, dom 3.25 Hz, conc 0.77 |
| vfdb/418 | 1 | 433.0 | NSR | 760 | – | shock | 12 | both methods shockable | pp 960 µV, noise 0.00, low-band 0.22, TCSC 0.79, count 0.21, leak 0.42, dom 3.25 Hz, conc 0.75 |
| vfdb/418 | 1 | 585.5 | NSR | 790 | – | shock | 12 | both methods shockable | pp 833 µV, noise 0.00, low-band 0.41, TCSC 0.76, count 0.19, leak 0.49, dom 2.50 Hz, conc 0.71 |
| cudb/cu09 | 0 | 481.8 | AFIB | 5031 | 163/min | shock | 7 | both methods shockable | pp 6787 µV, noise 0.00, low-band 0.09, TCSC 0.69, count 0.18, leak 0.46, dom 2.75 Hz, conc 0.79 |
| cudb/cu09 | 0 | 493.8 | AFIB | 5204 | 155/min | shock | 7 | both methods shockable | pp 5628 µV, noise 0.00, low-band 0.07, TCSC 0.72, count 0.21, leak 0.48, dom 2.50 Hz, conc 0.72 |
| vfdb/426 | 0 | 1842.2 | SVTA | 780 | – | shock | 8 | both methods shockable | pp 1007 µV, noise 0.00, low-band 0.29, TCSC 0.69, count 0.20, leak 0.62, dom 2.50 Hz, conc 0.60 |
| vfdb/426 | 1 | 1854.2 | SVTA | 989 | – | shock | 8 | both methods shockable | pp 1142 µV, noise 0.00, low-band 0.20, TCSC 0.72, count 0.17, leak 0.51, dom 3.00 Hz, conc 0.78 |
| vfdb/426 | 0 | 1795.5 | SVTA | 655 | – | shock | 9 | both methods shockable | pp 692 µV, noise 0.00, low-band 0.04, TCSC 0.73, count 0.17, leak 0.52, dom 2.75 Hz, conc 0.55 |
| vfdb/426 | 1 | 1842.2 | SVTA | 823 | – | shock | 10 | both methods shockable | pp 1367 µV, noise 0.00, low-band 0.11, TCSC 0.70, count 0.24, leak 0.56, dom 2.75 Hz, conc 0.71 |
| cudb/cu09 | 0 | 469.8 | AFIB | 2922 | 154/min | shock | 12 | both methods shockable | pp 3803 µV, noise 0.00, low-band 0.01, TCSC 0.78, count 0.18, leak 0.42, dom 2.75 Hz, conc 0.83 |
| vfdb/426 | 0 | 1854.2 | SVTA | 959 | – | shock | 12 | both methods shockable | pp 1225 µV, noise 0.00, low-band 0.40, TCSC 0.76, count 0.18, leak 0.51, dom 2.50 Hz, conc 0.81 |

### The 20 worst missed VF segments

Largest VF first (a missed large VF is the worst kind). Features are those of the last window looked at.

| Record | ch | start (s) | true rhythm | amplitude (µV) | rate | outcome | at (s) | window verdict | features |
|---|---|---|---|---|---|---|---|---|---|
| cudb/cu11 | 0 | 470.2 | VF | 10411 | – | inconclusive | – | clipped (flat tops) | pp 17396 µV, noise 0.05, low-band 0.59, TCSC 0.54, count 0.04, leak 0.81, dom 1.25 Hz, conc 0.48 |
| cudb/cu11 | 0 | 482.2 | VF | 9216 | – | inconclusive | – | clipped (flat tops) | pp 12747 µV, noise 0.04, low-band 0.26, TCSC 0.55, count 0.25, leak 0.63, dom 5.50 Hz, conc 0.57 |
| cudb/cu11 | 0 | 458.2 | VF | 7631 | – | inconclusive | – | possible artifact | pp 5811 µV, noise 0.00, low-band 0.53, TCSC 0.57, count 0.13, leak 0.72, dom 1.25 Hz, conc 0.41 |
| cudb/cu11 | 0 | 446.2 | VF | 7315 | – | no-shock | 5 | neither method shockable | pp 10498 µV, noise 0.07, low-band 0.29, TCSC 0.58, count 0.12, leak 0.70, dom 2.00 Hz, conc 0.30 |
| cudb/cu11 | 0 | 422.2 | VF | 6073 | – | no-shock | 5 | neither method shockable | pp 7320 µV, noise 0.00, low-band 0.49, TCSC 0.56, count 0.12, leak 0.76, dom 1.25 Hz, conc 0.38 |
| cudb/cu30 | 0 | 66.4 | VF | 5394 | – | no-shock | 9 | neither method shockable | pp 6019 µV, noise 0.00, low-band 0.46, TCSC 0.44, count 0.07, leak 0.82, dom 1.75 Hz, conc 0.41 |
| cudb/cu30 | 0 | 78.4 | VF | 5367 | – | no-shock | 6 | neither method shockable | pp 5331 µV, noise 0.01, low-band 0.37, TCSC 0.54, count 0.13, leak 0.80, dom 1.00 Hz, conc 0.16 |
| cudb/cu23 | 0 | 397.4 | VF | 4485 | – | inconclusive | – | clipped (flat tops) | pp 13972 µV, noise 0.03, low-band 0.55, TCSC 0.43, count 0.13, leak 0.84, dom 1.50 Hz, conc 0.51 |
| cudb/cu30 | 0 | 102.4 | VF | 4462 | – | inconclusive | – | possible artifact | pp 5313 µV, noise 0.00, low-band 0.66, TCSC 0.63, count 0.13, leak 0.81, dom 1.75 Hz, conc 0.62 |
| cudb/cu30 | 0 | 90.4 | VF | 4181 | – | inconclusive | – | possible artifact | pp 4524 µV, noise 0.00, low-band 0.67, TCSC 0.53, count 0.10, leak 0.76, dom 1.75 Hz, conc 0.51 |
| cudb/cu11 | 0 | 434.2 | VF | 3909 | – | inconclusive | – | methods disagree | pp 9032 µV, noise 0.00, low-band 0.49, TCSC 0.54, count 0.19, leak 0.72, dom 2.00 Hz, conc 0.52 |
| cudb/cu15 | 0 | 409.0 | VF | 3682 | – | inconclusive | – | possible artifact | pp 4107 µV, noise 0.00, low-band 0.94, TCSC 0.78, count 0.15, leak 0.30, dom 2.00 Hz, conc 0.94 |
| cudb/cu21 | 0 | 71.2 | VF | 3600 | – | inconclusive | – | both methods shockable | pp 3580 µV, noise 0.01, low-band 0.05, TCSC 0.62, count 0.22, leak 0.61, dom 4.25 Hz, conc 0.64 |
| cudb/cu15 | 0 | 421.0 | VF | 3534 | – | inconclusive | – | possible artifact | pp 3773 µV, noise 0.01, low-band 0.93, TCSC 0.80, count 0.14, leak 0.55, dom 2.25 Hz, conc 0.97 |
| cudb/cu15 | 0 | 445.0 | VF | 2767 | – | inconclusive | – | possible artifact | pp 3078 µV, noise 0.00, low-band 0.93, TCSC 0.81, count 0.24, leak 0.41, dom 2.00 Hz, conc 0.94 |
| cudb/cu23 | 0 | 409.4 | VF | 2614 | – | inconclusive | – | clipped (flat tops) | pp 20364 µV, noise 0.01, low-band 0.39, TCSC 0.29, count 0.06, leak 0.82, dom 1.00 Hz, conc 0.19 |
| cudb/cu30 | 0 | 54.4 | VF | 2468 | – | inconclusive | – | possible artifact | pp 6936 µV, noise 0.00, low-band 0.74, TCSC 0.56, count 0.12, leak 0.80, dom 1.50 Hz, conc 0.72 |
| cudb/cu23 | 0 | 421.4 | VF | 2462 | – | inconclusive | – | possible artifact | pp 2825 µV, noise 0.00, low-band 0.55, TCSC 0.52, count 0.11, leak 0.74, dom 1.75 Hz, conc 0.54 |
| cudb/cu15 | 0 | 433.0 | VF | 2412 | – | inconclusive | – | possible artifact | pp 3777 µV, noise 0.01, low-band 0.92, TCSC 0.74, count 0.13, leak 0.44, dom 2.25 Hz, conc 0.93 |
| cudb/cu15 | 0 | 457.0 | VF | 2377 | – | inconclusive | – | possible artifact | pp 2603 µV, noise 0.01, low-band 0.91, TCSC 0.74, count 0.21, leak 0.59, dom 2.00 Hz, conc 0.97 |

## Amplitude calibration and the microvolt rules

The analyser has three absolute-size rules: under 100 µV peak to peak is flatline (no shock); under 200 µV VF is "fine" (not shocked); over 12000 µV is implausible (not sure). They only mean something if the recording's microvolts are right. cudb's gain of 400 ADC units/mV is described by PhysioNet as nominal (10 mV full scale "nominal"; cu01 was replayed from a Holter tape). Nothing here was rescaled.

Per database, channel 0: the median over records of each record's median ground-truth amplitude (1-s peak to peak, drift removed) in clean streaming windows, and how VF windows fall against the analyser's own size rules (its peak-to-peak is measured after its 1–30 Hz filter, so it reads lower than the ground-truth figure).

| Database | non-VF amplitude (median of record medians) | VF amplitude (median of record medians) | VF windows | analyser pp < 200 µV (fine-VF rule) | analyser pp < 100 µV (flatline rule) | ground truth < 200 µV |
|---|---|---|---|---|---|---|
| cudb | 2607 µV (16 records) | 2046 µV (14 records) | 1512 | 0.0% | 0.0% | 0.0% |
| vfdb | 1319 µV (10 records) | 1042 µV (2 records) | 1102 | 0.2% | 0.0% | 0.0% |
| mitdb | 1992 µV (24 records) | – | 0 | – | – | – |

Per cudb record, channel 0 (to show whether the nominal calibration is consistent between records):

| Record | non-VF amplitude (median, µV) | VF amplitude (median, µV) | VF windows called fine VF | VF windows called flatline |
|---|---|---|---|---|
| cu01 | 2579 | 1308 | 0.0% | 0.0% |
| cu02 | 1809 | – | – | – |
| cu04 | 1886 | 1507 | 0.0% | 0.0% |
| cu07 | 2353 | 1386 | 0.0% | 0.0% |
| cu09 | 3036 | 1758 | 0.0% | 0.0% |
| cu11 | 3372 | 4073 | 0.0% | 0.0% |
| cu15 | 3274 | 2406 | 0.0% | 0.0% |
| cu16 | 3149 | 4823 | 0.0% | 0.0% |
| cu17 | 1674 | 2182 | 0.0% | 0.0% |
| cu21 | 3025 | 3340 | 0.0% | 0.0% |
| cu23 | 2645 | 3073 | 0.0% | 0.0% |
| cu25 | 1816 | 1137 | 0.0% | 0.0% |
| cu26 | 2439 | 1909 | 0.0% | 0.0% |
| cu27 | – | – | – | – |
| cu29 | 2827 | – | – | – |
| cu30 | 2635 | 3850 | 0.0% | 0.0% |
| cu32 | 2282 | 1372 | 0.0% | 0.0% |

## Reproducing this report

```
cargo run --release -p aed-eval -- --data <path to data/physionet> --csv <out dir>
```

The data are not in the repository; see `firmware/crates/aed-eval/README.md` for where to get them, their licences and the citations they ask for.
