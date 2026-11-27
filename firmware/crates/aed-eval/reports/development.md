# Rhythm analyser on recorded heart rhythms

- **Date (UTC):** 2026-11-27
- **Analyser evaluated:** `aed-rhythm` at git commit `b9a79e7` (**uncommitted changes in the analyser**: M firmware/crates/aed-rhythm/src/analyzer.rs;  M firmware/crates/aed-rhythm/src/features.rs;  M firmware/crates/aed-rhythm/src/lib.rs;  M firmware/crates/aed-rhythm/src/settings.rs; 14 uncommitted path(s) elsewhere in the repository)
- **Data:** PhysioNet cudb, vfdb, mitdb (public Holter / monitoring recordings), read from `data/physionet`
- **Records evaluated:** 53
- **Restricted to:** 53 records listed in `crates/aed-eval/splits/development.txt` (SHA-256 `597e65c49bc0114f3680c1e3f49cc8d553407606e411f6bce6a3e1714d477903`)

**Development half, frozen settings, resubstitution.** The settings were chosen on these same records (with leave-one-patient-out cross-validation; see TUNING_LOG.md), so these figures are optimistic. The cross-validated figures are in TUNING_LOG.md and REPORT.md.

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
| cudb | 18 | 18 | 250 | 18/18 | 18/18 | 19053 (0.8%) | 0 | 400 | none |
| vfdb | 11 | 22 | 250 | 22/22 | 22/22 | 0 (0.0%) | 0 | 200 | none |
| mitdb | 24 | 48 | 360 | 48/48 | 48/48 | 707 (0.0%) | 0 | 200 | none |

Every record listed in each database's RECORDS file loaded without error.

"Samples at recorder limit" are values at the very end of the original recorder's range: the recording itself was clipped there (the original analogue chain saturated). The analyser cannot see this: its own saturation rule looks for values beyond ±32 000 µV, which these recordings (range about ±5 mV for cudb and mitdb, ±10 mV for vfdb) can never reach. No sample needed clipping to fit the analyser's 16-bit input.

## Headline: device-realistic analysis (12 s limit)

Each cell: result (exact 90 % lower limit, segments, records); ✗ = misses the AHA goal or its lower limit. Inconclusive = share of all scored segments ending "unable to analyse".

| Group | Coarse VF sensitivity (AHA ≥90%, LCL 87%) | Rapid VT sensitivity (≥75%, 67%) | NSR specificity (≥99%, 97%) | Other non-shockable specificity (≥95%, 88%) | Asystole specificity (≥95%, 92%) | Inconclusive |
|---|---|---|---|---|---|---|
| cudb channel 0 | 59.1% (LCL 52.0%, n=93, 12 rec) ✗ | no data | no data | 100.0% (LCL 90.9%, n=24, 1 rec) | no data | 16.6% (n=589) |
| vfdb channel 0 | 84.1% (LCL 77.9%, n=88, 2 rec) ✗ | no data | 100.0% (LCL 99.6%, n=640, 8 rec) | 100.0% (LCL 99.6%, n=519, 8 rec) | 96.4% (LCL 90.8%, n=56, 4 rec) ✗ | 17.9% (n=1628) |
| mitdb channel 0 | no data | no data | 100.0% (LCL 99.9%, n=2446, 22 rec) | 100.0% (LCL 99.6%, n=594, 13 rec) | no data | 1.9% (n=3053) |
| **pooled, channel 0** | 71.3% (LCL 66.5%, n=181, 14 rec) ✗ | no data | 100.0% (LCL 99.9%, n=3086, 30 rec) | 100.0% (LCL 99.8%, n=1137, 22 rec) | 96.4% (LCL 90.8%, n=56, 4 rec) ✗ | 8.5% (n=5270) |
| vfdb channel 1 | 87.5% (LCL 81.7%, n=88, 2 rec) ✗ | no data | 99.8% (LCL 99.4%, n=640, 8 rec) | 100.0% (LCL 99.6%, n=519, 8 rec) | 91.1% (LCL 84.1%, n=56, 4 rec) ✗ | 13.9% (n=1628) |
| mitdb channel 1 | no data | no data | 100.0% (LCL 99.9%, n=2347, 22 rec) | 100.0% (LCL 99.6%, n=569, 13 rec) | no data | 4.1% (n=2927) |
| **pooled, channel 1** | 87.5% (LCL 81.7%, n=88, 2 rec) ✗ | no data | 100.0% (LCL 99.9%, n=2987, 30 rec) | 100.0% (LCL 99.8%, n=1088, 21 rec) | 91.1% (LCL 84.1%, n=56, 4 rec) ✗ | 7.6% (n=4555) |

## Time to decision, and shocks lost in the countdown (12 s analysis)

Every second of analysis is hands-off time. "Decided by" = share of segments with a final answer (shock or no shock) by that time after the analysis started; the rest are still analysing. After "shock advised" the rescue sequence counts down for 3 s and abandons the shock on any answer other than `Shockable` (aed-core); "cancelled in countdown" counts shocks the analyser itself would have called off that way, measured by carrying on into the signal that follows (where the same rhythm continues). "Delivered" = shocks not cancelled, over all segments; a shock whose countdown could not be checked counts as delivered.

| Group | Class | n | decided by 7 s | 8 s | 10 s | 12 s | median time to answer (s) | 90th pct (s) | shocks | cancelled in countdown | countdown not checkable | delivered |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| channel 0 | Coarse VF | 181 | 64.6% | 72.4% | 78.5% | 82.3% | 7 | 9 | 129 | 29 | 0 | 55.2% |
| channel 0 | Ventricular flutter | 26 | 50.0% | 57.7% | 80.8% | 88.5% | 7 | 10 | 20 | 4 | 0 | 61.5% |
| channel 0 | VT, rate unknown | 289 | 42.9% | 49.8% | 55.0% | 59.2% | 7 | 10 | 140 | 33 | 0 | 37.0% |
| channel 0 | Normal sinus rhythm | 3086 | 95.7% | 95.9% | 96.8% | 96.9% | 5 | 5 | 0 | 0 | 0 | 0.0% |
| channel 0 | Other non-shockable | 1137 | 89.7% | 90.1% | 90.9% | 91.5% | 5 | 6 | 0 | 0 | 0 | 0.0% |
| channel 0 | Asystole | 56 | 50.0% | 53.6% | 53.6% | 57.1% | 5 | 8 | 2 | 2 | 0 | 0.0% |
| channel 1 | Coarse VF | 88 | 75.0% | 81.8% | 87.5% | 90.9% | 7 | 8 | 77 | 17 | 0 | 68.2% |
| channel 1 | Ventricular flutter | 26 | 69.2% | 73.1% | 84.6% | 92.3% | 7 | 10 | 23 | 5 | 0 | 69.2% |
| channel 1 | VT, rate unknown | 289 | 41.5% | 47.1% | 54.3% | 57.8% | 7 | 10 | 108 | 37 | 0 | 24.6% |
| channel 1 | Normal sinus rhythm | 2987 | 93.2% | 93.7% | 94.6% | 95.3% | 5 | 5 | 1 | 0 | 0 | 0.0% |
| channel 1 | Other non-shockable | 1088 | 83.5% | 87.7% | 90.4% | 93.5% | 5 | 8 | 0 | 0 | 0 | 0.0% |
| channel 1 | Asystole | 56 | 87.5% | 87.5% | 92.9% | 94.6% | 5 | 7 | 5 | 1 | 0 | 7.1% |

## Segment-level results

Report-only buckets show the share that ended in a shock. "cudb non-VF, rhythm not given" is cudb time outside the annotated VF episodes for which no rhythm label exists; it may contain VT (cudb is a tachyarrhythmia database) and is therefore not counted as a non-shockable class.

### 12 s analysis

No "Rapid VT" row appears: the only VT with beat annotations (mitdb, a few cudb labels) comes in runs too short to hold a whole segment plus the 3 s guard margins, and the few that fit are slow. vfdb's VT has no beat annotations, so its rate is unknown. The rapid-VT goal therefore cannot be tested on these databases with annotation-based rates; see the rate-estimate table in the failure analysis for a rough split.

**cudb channel 0** (589 segments from 16 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 93 | 12 | 55 | 18 | 20 | 59.1% | 52.0% | 52.5% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Other non-shockable | specificity | 24 | 1 | 0 | 24 | 0 | 100.0% | 90.9% | 93.6% | ≥95% (88%) | 30 | meets (n<30) |
| cudb non-VF, rhythm not given | shock rate | 472 | 16 | 1 | 393 | 78 | 0.2% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 93 | 12 | 55 | 18 | 20 | 59.1% | 52.0% | 52.5% | – | – | – |

Inconclusive ("unable to analyse") over all 589 segments: **16.6%**.

**vfdb channel 0** (1628 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 88 | 2 | 74 | 2 | 12 | 84.1% | 77.9% | 78.5% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 19 | 4 | 14 | 3 | 2 | 73.7% | 56.6% | 59.2% | report only | – | report only |
| VT, rate unknown | shock rate | 289 | 8 | 140 | 31 | 118 | 48.4% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 640 | 8 | 0 | 595 | 45 | 100.0% | 99.6% | 99.7% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 2 | 30 | 24 | 96.4% | 90.8% | 91.7% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 519 | 8 | 0 | 429 | 90 | 100.0% | 99.6% | 99.7% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 17 | 2 | 1 | 15 | 1 | 5.9% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 88 | 2 | 74 | 2 | 12 | 84.1% | 77.9% | 78.5% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 289 | 8 | 140 | 31 | 118 | 48.4% | 44.5% | 44.7% | – | – | – |

Inconclusive ("unable to analyse") over all 1628 segments: **17.9%**.

**vfdb channel 1** (1628 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 88 | 2 | 77 | 3 | 8 | 87.5% | 81.7% | 82.3% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 19 | 4 | 16 | 1 | 2 | 84.2% | 68.1% | 70.8% | report only | – | report only |
| VT, rate unknown | shock rate | 289 | 8 | 108 | 59 | 122 | 37.4% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 640 | 8 | 1 | 565 | 74 | 99.8% | 99.4% | 99.5% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 5 | 48 | 3 | 91.1% | 84.1% | 84.9% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 519 | 8 | 0 | 503 | 16 | 100.0% | 99.6% | 99.7% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 17 | 2 | 0 | 16 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 88 | 2 | 77 | 3 | 8 | 87.5% | 81.7% | 82.3% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 289 | 8 | 108 | 59 | 122 | 37.4% | 33.6% | 33.8% | – | – | – |

Inconclusive ("unable to analyse") over all 1628 segments: **13.9%**.

**mitdb channel 0** (3053 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 7 | 1 | 6 | 0 | 1 | 85.7% | 54.7% | 62.2% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 6 | 1 | 0 | 6 | 0 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 2446 | 22 | 0 | 2395 | 51 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 594 | 13 | 0 | 587 | 7 | 100.0% | 99.6% | 99.7% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 6 | 1 | 0 | 6 | 0 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 3053 segments: **1.9%**.

**mitdb channel 1** (2927 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 7 | 1 | 7 | 0 | 0 | 100.0% | 72.0% | 81.0% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 4 | 1 | 0 | 4 | 0 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 2347 | 22 | 0 | 2282 | 65 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 569 | 13 | 0 | 514 | 55 | 100.0% | 99.6% | 99.7% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 4 | 1 | 0 | 4 | 0 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 2927 segments: **4.1%**.

**Pooled, channel 0** (5270 segments from 51 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 181 | 14 | 129 | 20 | 32 | 71.3% | 66.5% | 66.8% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 26 | 5 | 20 | 3 | 3 | 76.9% | 63.0% | 64.9% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 6 | 1 | 0 | 6 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 289 | 8 | 140 | 31 | 118 | 48.4% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 3086 | 30 | 0 | 2990 | 96 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 2 | 30 | 24 | 96.4% | 90.8% | 91.7% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 1137 | 22 | 0 | 1040 | 97 | 100.0% | 99.8% | 99.9% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 472 | 16 | 1 | 393 | 78 | 0.2% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 17 | 2 | 1 | 15 | 1 | 5.9% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 181 | 14 | 129 | 20 | 32 | 71.3% | 66.5% | 66.8% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 295 | 9 | 140 | 37 | 118 | 47.5% | 43.6% | 43.8% | – | – | – |

Inconclusive ("unable to analyse") over all 5270 segments: **8.5%**.

**Pooled, channel 1 (vfdb + mitdb)** (4555 segments from 35 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 88 | 2 | 77 | 3 | 8 | 87.5% | 81.7% | 82.3% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 26 | 5 | 23 | 1 | 2 | 88.5% | 76.1% | 78.1% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 4 | 1 | 0 | 4 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 289 | 8 | 108 | 59 | 122 | 37.4% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 2987 | 30 | 1 | 2847 | 139 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 56 | 4 | 5 | 48 | 3 | 91.1% | 84.1% | 84.9% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 1088 | 21 | 0 | 1017 | 71 | 100.0% | 99.8% | 99.8% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 17 | 2 | 0 | 16 | 1 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 88 | 2 | 77 | 3 | 8 | 87.5% | 81.7% | 82.3% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 293 | 9 | 108 | 63 | 122 | 36.9% | 33.2% | 33.3% | – | – | – |

Inconclusive ("unable to analyse") over all 4555 segments: **7.6%**.

**Non-shockable rhythms by label, pooled channel 0, 12 s**

| Rhythm label | n | records | shock (false positive) | no shock | inconcl. | specificity | 90% LCL exact |
|---|---|---|---|---|---|---|---|
| AB (atrial bigeminy) | 5 | 1 | 0 | 5 | 0 | 100.0% | 63.1% |
| AFIB | 114 | 4 | 0 | 38 | 76 | 100.0% | 98.0% |
| AFL | 12 | 1 | 0 | 12 | 0 | 100.0% | 82.5% |
| ASYS | 56 | 4 | 2 | 30 | 24 | 96.4% | 90.8% |
| B (ventricular bigeminy) | 124 | 7 | 0 | 110 | 14 | 100.0% | 98.2% |
| BI (1st-degree block) | 27 | 1 | 0 | 27 | 0 | 100.0% | 91.8% |
| BII (2nd-degree block) | 52 | 1 | 0 | 52 | 0 | 100.0% | 95.7% |
| IVR (idioventricular) | 8 | 1 | 0 | 8 | 0 | 100.0% | 75.0% |
| NOD (junctional) | 140 | 2 | 0 | 137 | 3 | 100.0% | 98.4% |
| NSR | 3086 | 30 | 0 | 2990 | 96 | 100.0% | 99.9% |
| PACED | 507 | 4 | 0 | 507 | 0 | 100.0% | 99.5% |
| PREX (pre-excitation) | 16 | 1 | 0 | 16 | 0 | 100.0% | 86.6% |
| SBR | 1 | 1 | 0 | 1 | 0 | 100.0% | 10.0% |
| SVTA | 120 | 4 | 0 | 116 | 4 | 100.0% | 98.1% |
| T (ventricular trigeminy) | 9 | 1 | 0 | 9 | 0 | 100.0% | 77.4% |
| VER (ventricular escape) | 2 | 1 | 0 | 2 | 0 | 100.0% | 31.6% |

**Non-shockable rhythms by label, pooled channel 1, 12 s**

| Rhythm label | n | records | shock (false positive) | no shock | inconcl. | specificity | 90% LCL exact |
|---|---|---|---|---|---|---|---|
| AB (atrial bigeminy) | 5 | 1 | 0 | 5 | 0 | 100.0% | 63.1% |
| AFIB | 90 | 3 | 0 | 90 | 0 | 100.0% | 97.5% |
| AFL | 12 | 1 | 0 | 12 | 0 | 100.0% | 82.5% |
| ASYS | 56 | 4 | 5 | 48 | 3 | 91.1% | 84.1% |
| B (ventricular bigeminy) | 100 | 7 | 0 | 87 | 13 | 100.0% | 97.7% |
| BI (1st-degree block) | 27 | 1 | 0 | 27 | 0 | 100.0% | 91.8% |
| BII (2nd-degree block) | 52 | 1 | 0 | 52 | 0 | 100.0% | 95.7% |
| IVR (idioventricular) | 8 | 1 | 0 | 6 | 2 | 100.0% | 75.0% |
| NOD (junctional) | 140 | 2 | 0 | 136 | 4 | 100.0% | 98.4% |
| NSR | 2987 | 30 | 1 | 2847 | 139 | 100.0% | 99.9% |
| PACED | 507 | 4 | 0 | 457 | 50 | 100.0% | 99.5% |
| PREX (pre-excitation) | 15 | 1 | 0 | 15 | 0 | 100.0% | 85.8% |
| SBR | 1 | 1 | 0 | 1 | 0 | 100.0% | 10.0% |
| SVTA | 120 | 4 | 0 | 119 | 1 | 100.0% | 98.1% |
| T (ventricular trigeminy) | 9 | 1 | 0 | 8 | 1 | 100.0% | 77.4% |
| VER (ventricular escape) | 2 | 1 | 0 | 2 | 0 | 100.0% | 31.6% |

### 8 s analysis

**cudb channel 0** (898 segments from 16 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 144 | 13 | 72 | 23 | 49 | 50.0% | 44.3% | 44.7% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Other non-shockable | specificity | 36 | 1 | 0 | 36 | 0 | 100.0% | 93.8% | 95.6% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 718 | 16 | 2 | 570 | 146 | 0.3% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 144 | 13 | 72 | 23 | 49 | 50.0% | 44.3% | 44.7% | – | – | – |

Inconclusive ("unable to analyse") over all 898 segments: **21.7%**.

**vfdb channel 0** (2462 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 134 | 2 | 96 | 3 | 35 | 71.6% | 66.1% | 66.4% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 30 | 4 | 17 | 2 | 11 | 56.7% | 43.4% | 45.0% | report only | – | report only |
| VT, rate unknown | shock rate | 437 | 8 | 184 | 38 | 215 | 42.1% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 964 | 8 | 0 | 869 | 95 | 100.0% | 99.8% | 99.8% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 86 | 4 | 3 | 44 | 39 | 96.5% | 92.4% | 93.0% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 785 | 8 | 0 | 629 | 156 | 100.0% | 99.7% | 99.8% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 26 | 2 | 1 | 24 | 1 | 3.8% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 134 | 2 | 96 | 3 | 35 | 71.6% | 66.1% | 66.4% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 437 | 8 | 184 | 38 | 215 | 42.1% | 39.0% | 39.1% | – | – | – |

Inconclusive ("unable to analyse") over all 2462 segments: **22.4%**.

**mitdb channel 0** (4672 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 11 | 1 | 7 | 1 | 3 | 63.6% | 40.1% | 44.4% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 10 | 1 | 0 | 10 | 0 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 3722 | 22 | 0 | 3623 | 99 | 100.0% | 99.9% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 929 | 13 | 0 | 912 | 17 | 100.0% | 99.8% | 99.8% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 10 | 1 | 0 | 10 | 0 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 4672 segments: **2.5%**.

**Pooled, channel 0** (8032 segments from 51 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 278 | 15 | 168 | 26 | 84 | 60.4% | 56.4% | 56.6% | ≥90% (87%) | 200 | **FAILS** |
| Ventricular flutter | sensitivity | 41 | 5 | 24 | 3 | 14 | 58.5% | 47.3% | 48.5% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 10 | 1 | 0 | 10 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 437 | 8 | 184 | 38 | 215 | 42.1% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 4686 | 30 | 0 | 4492 | 194 | 100.0% | 100.0% | 100.0% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 86 | 4 | 3 | 44 | 39 | 96.5% | 92.4% | 93.0% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 1750 | 22 | 0 | 1577 | 173 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 718 | 16 | 2 | 570 | 146 | 0.3% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 26 | 2 | 1 | 24 | 1 | 3.8% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 278 | 15 | 168 | 26 | 84 | 60.4% | 56.4% | 56.6% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 447 | 9 | 184 | 48 | 215 | 41.2% | 38.1% | 38.2% | – | – | – |

Inconclusive ("unable to analyse") over all 8032 segments: **10.8%**.

**Pooled, channel 1 (vfdb + mitdb)** (6946 segments from 35 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 134 | 2 | 95 | 4 | 35 | 70.9% | 65.3% | 65.6% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 41 | 5 | 29 | 2 | 10 | 70.7% | 59.8% | 61.0% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 8 | 1 | 0 | 8 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 437 | 8 | 139 | 67 | 231 | 31.8% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 4536 | 30 | 0 | 4251 | 285 | 100.0% | 99.9% | 100.0% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 86 | 4 | 9 | 66 | 11 | 89.5% | 84.0% | 84.5% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 1678 | 21 | 0 | 1454 | 224 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 26 | 2 | 0 | 22 | 4 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 134 | 2 | 95 | 4 | 35 | 70.9% | 65.3% | 65.6% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 445 | 9 | 139 | 75 | 231 | 31.2% | 28.4% | 28.5% | – | – | – |

Inconclusive ("unable to analyse") over all 6946 segments: **11.5%**.

### 7 s analysis

**cudb channel 0** (1027 segments from 16 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 165 | 13 | 75 | 19 | 71 | 45.5% | 40.2% | 40.6% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Other non-shockable | specificity | 41 | 1 | 0 | 41 | 0 | 100.0% | 94.5% | 96.1% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 821 | 16 | 1 | 635 | 185 | 0.1% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 165 | 13 | 75 | 19 | 71 | 45.5% | 40.2% | 40.6% | – | – | – |

Inconclusive ("unable to analyse") over all 1027 segments: **24.9%**.

**vfdb channel 0** (2823 segments from 11 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 154 | 2 | 102 | 3 | 49 | 66.2% | 60.9% | 61.2% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 35 | 4 | 17 | 4 | 14 | 48.6% | 36.6% | 38.1% | report only | – | report only |
| VT, rate unknown | shock rate | 502 | 8 | 195 | 40 | 267 | 38.8% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 1104 | 8 | 0 | 987 | 117 | 100.0% | 99.8% | 99.9% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 98 | 4 | 3 | 42 | 53 | 96.9% | 93.3% | 93.8% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 900 | 8 | 0 | 718 | 182 | 100.0% | 99.7% | 99.8% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 30 | 2 | 1 | 25 | 4 | 3.3% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 154 | 2 | 102 | 3 | 49 | 66.2% | 60.9% | 61.2% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 502 | 8 | 195 | 40 | 267 | 38.8% | 36.0% | 36.1% | – | – | – |

Inconclusive ("unable to analyse") over all 2823 segments: **24.3%**.

**mitdb channel 0** (5365 segments from 24 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ventricular flutter | sensitivity | 14 | 1 | 6 | 1 | 7 | 42.9% | 24.3% | 27.6% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 11 | 1 | 0 | 9 | 2 | 0.0% | – | – | report only | 25 | report only |
| Normal sinus rhythm | specificity | 4267 | 22 | 0 | 4142 | 125 | 100.0% | 99.9% | 100.0% | ≥99% (97%) | 100 | meets |
| Other non-shockable | specificity | 1073 | 13 | 0 | 1042 | 31 | 100.0% | 99.8% | 99.8% | ≥95% (88%) | 30 | meets |
| *All VT, no rate criterion* | sensitivity | 11 | 1 | 0 | 9 | 2 | 0.0% | 0.0% | 0.0% | – | – | – |

Inconclusive ("unable to analyse") over all 5365 segments: **3.1%**.

**Pooled, channel 0** (9215 segments from 51 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 319 | 15 | 177 | 22 | 120 | 55.5% | 51.7% | 51.9% | ≥90% (87%) | 200 | **FAILS** |
| Ventricular flutter | sensitivity | 49 | 5 | 23 | 5 | 21 | 46.9% | 37.0% | 38.1% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 11 | 1 | 0 | 9 | 2 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 502 | 8 | 195 | 40 | 267 | 38.8% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 5371 | 30 | 0 | 5129 | 242 | 100.0% | 100.0% | 100.0% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 98 | 4 | 3 | 42 | 53 | 96.9% | 93.3% | 93.8% | ≥95% (92%) | 100 | meets (n<100) |
| Other non-shockable | specificity | 2014 | 22 | 0 | 1801 | 213 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| cudb non-VF, rhythm not given | shock rate | 821 | 16 | 1 | 635 | 185 | 0.1% | – | – | report only | – | report only |
| HGEA (vfdb) | shock rate | 30 | 2 | 1 | 25 | 4 | 3.3% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 319 | 15 | 177 | 22 | 120 | 55.5% | 51.7% | 51.9% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 513 | 9 | 195 | 49 | 269 | 38.0% | 35.2% | 35.3% | – | – | – |

Inconclusive ("unable to analyse") over all 9215 segments: **12.0%**.

**Pooled, channel 1 (vfdb + mitdb)** (7980 segments from 35 records)

| Class | Metric | n | records | shock | no shock | inconcl. | result | 90% LCL exact | 90% LCL Wilson | AHA goal (LCL) | AHA min n* | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | sensitivity | 154 | 2 | 107 | 4 | 43 | 69.5% | 64.2% | 64.5% | ≥90% (87%) | 200 | **FAILS** (n<200) |
| Ventricular flutter | sensitivity | 49 | 5 | 31 | 3 | 15 | 63.3% | 53.1% | 54.1% | report only | – | report only |
| Slow VT (<150/min) | shock rate | 8 | 1 | 0 | 8 | 0 | 0.0% | – | – | report only | 25 | report only |
| VT, rate unknown | shock rate | 502 | 8 | 135 | 66 | 301 | 26.9% | – | – | report only | – | report only |
| Normal sinus rhythm | specificity | 5205 | 30 | 2 | 4856 | 347 | 100.0% | 99.9% | 99.9% | ≥99% (97%) | 100 | meets |
| Asystole | specificity | 98 | 4 | 9 | 73 | 16 | 90.8% | 85.9% | 86.4% | ≥95% (92%) | 100 | **FAILS** (n<100) |
| Other non-shockable | specificity | 1934 | 21 | 0 | 1595 | 339 | 100.0% | 99.9% | 99.9% | ≥95% (88%) | 30 | meets |
| HGEA (vfdb) | shock rate | 30 | 2 | 0 | 20 | 10 | 0.0% | – | – | report only | – | report only |
| *All VF, any amplitude* | sensitivity | 154 | 2 | 107 | 4 | 43 | 69.5% | 64.2% | 64.5% | – | – | – |
| *All VT, no rate criterion* | sensitivity | 510 | 9 | 135 | 74 | 301 | 26.5% | 23.9% | 24.0% | – | – | – |

Inconclusive ("unable to analyse") over all 7980 segments: **13.4%**.

## Per record (12 s analysis, every channel)

So that one bad patient is visible. VF = coarse VF; "VT/VFL shock" = shocks / segments over VT of any rate and flutter; FP = false shocks on NSR or other non-shockable rhythms. Flags: **VF<50%** (≥5 coarse-VF segments and under half shocked), **FP** (any false shock on an AHA non-shockable class), **INC>50%** (over half of all segments inconclusive).

| Record | ch | lead | coarse VF n | VF Se | VT/VFL shock | NSR n | NSR Sp | other n | other Sp | FP | inconcl. | flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cudb/cu03 | 0 | ECG | 3 | 100.0% | – | 0 | – | 0 | – | 0 | 79.5% | **INC>50%** |
| cudb/cu05 | 0 | ECG | 6 | 100.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu06 | 0 | ECG | 8 | 87.5% | – | 0 | – | 0 | – | 0 | 5.6% |  |
| cudb/cu10 | 0 | ECG | 15 | 66.7% | – | 0 | – | 0 | – | 0 | 2.5% |  |
| cudb/cu12 | 0 | ECG | 15 | 40.0% | – | 0 | – | 0 | – | 0 | 28.2% | **VF<50%** |
| cudb/cu13 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 5.9% |  |
| cudb/cu14 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 15.0% |  |
| cudb/cu18 | 0 | ECG | 1 | 100.0% | – | 0 | – | 24 | 100.0% | 0 | 0.0% |  |
| cudb/cu19 | 0 | ECG | 2 | 100.0% | – | 0 | – | 0 | – | 0 | 45.7% |  |
| cudb/cu20 | 0 | ECG | 21 | 9.5% | – | 0 | – | 0 | – | 0 | 37.5% | **VF<50%** |
| cudb/cu22 | 0 | ECG | 8 | 100.0% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu24 | 0 | ECG | 5 | 60.0% | – | 0 | – | 0 | – | 0 | 5.9% |  |
| cudb/cu28 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 25.0% |  |
| cudb/cu31 | 0 | ECG | 0 | – | – | 0 | – | 0 | – | 0 | 2.5% |  |
| cudb/cu33 | 0 | ECG | 6 | 83.3% | – | 0 | – | 0 | – | 0 | 0.0% |  |
| cudb/cu34 | 0 | ECG | 3 | 66.7% | – | 0 | – | 0 | – | 0 | 3.1% |  |
| vfdb/419 | 0 | ECG | 0 | – | 8/9 | 79 | 100.0% | 0 | – | 0 | 46.6% |  |
| vfdb/419 | 1 | ECG | 0 | – | 8/9 | 79 | 98.7% | 0 | – | 1 | 54.5% | **FP** **INC>50%** |
| vfdb/420 | 0 | ECG | 0 | – | 16/20 | 108 | 100.0% | 0 | – | 0 | 3.1% |  |
| vfdb/420 | 1 | ECG | 0 | – | 14/20 | 108 | 100.0% | 0 | – | 0 | 1.6% |  |
| vfdb/424 | 0 | ECG | 8 | 100.0% | 1/4 | 121 | 100.0% | 10 | 100.0% | 0 | 2.1% |  |
| vfdb/424 | 1 | ECG | 8 | 100.0% | 3/4 | 121 | 100.0% | 10 | 100.0% | 0 | 5.6% |  |
| vfdb/425 | 0 | ECG | 0 | – | 1/1 | 112 | 100.0% | 25 | 100.0% | 0 | 10.1% |  |
| vfdb/425 | 1 | ECG | 0 | – | 1/1 | 112 | 100.0% | 25 | 100.0% | 0 | 11.6% |  |
| vfdb/427 | 0 | ECG | 0 | – | 51/73 | 53 | 100.0% | 46 | 100.0% | 0 | 19.8% |  |
| vfdb/427 | 1 | ECG | 0 | – | 25/73 | 53 | 100.0% | 46 | 100.0% | 0 | 20.3% |  |
| vfdb/430 | 0 | ECG | 80 | 82.5% | 24/30 | 0 | – | 31 | 100.0% | 0 | 12.9% |  |
| vfdb/430 | 1 | ECG | 80 | 86.2% | 24/30 | 0 | – | 31 | 100.0% | 0 | 10.9% |  |
| vfdb/602 | 0 | ECG | 0 | – | 2/31 | 3 | 100.0% | 117 | 100.0% | 0 | 5.3% |  |
| vfdb/602 | 1 | ECG | 0 | – | 8/31 | 3 | 100.0% | 117 | 100.0% | 0 | 6.6% |  |
| vfdb/607 | 0 | ECG | 0 | – | 8/10 | 0 | – | 156 | 98.7% | 2 | 4.8% | **FP** |
| vfdb/607 | 1 | ECG | 0 | – | 8/10 | 0 | – | 156 | 96.8% | 5 | 2.4% | **FP** |
| vfdb/609 | 0 | ECG | 0 | – | 1/1 | 69 | 100.0% | 84 | 100.0% | 0 | 46.1% |  |
| vfdb/609 | 1 | ECG | 0 | – | 1/1 | 69 | 100.0% | 84 | 100.0% | 0 | 0.0% |  |
| vfdb/611 | 0 | ECG | 0 | – | 0/74 | 0 | – | 99 | 100.0% | 0 | 42.8% |  |
| vfdb/611 | 1 | ECG | 0 | – | 0/74 | 0 | – | 99 | 100.0% | 0 | 42.8% |  |
| vfdb/614 | 0 | ECG | 0 | – | 42/55 | 95 | 100.0% | 7 | 100.0% | 0 | 7.0% |  |
| vfdb/614 | 1 | ECG | 0 | – | 32/55 | 95 | 100.0% | 7 | 100.0% | 0 | 8.3% |  |
| mitdb/100 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/100 | 1 | V5 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/101 | 0 | MLII | 0 | – | – | 147 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/101 | 1 | V1 | 0 | – | – | 147 | 100.0% | 0 | – | 0 | 44.2% |  |
| mitdb/102 | 0 | V5 | 0 | – | – | 5 | 100.0% | 141 | 100.0% | 0 | 0.0% |  |
| mitdb/102 | 1 | V2 | 0 | – | – | 5 | 100.0% | 141 | 100.0% | 0 | 3.4% |  |
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
| mitdb/117 | 0 | MLII | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/117 | 1 | V2 | 0 | – | – | 148 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/123 | 0 | MLII | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/123 | 1 | V5 | 0 | – | – | 149 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/200 | 0 | MLII | 0 | – | – | 38 | 100.0% | 26 | 100.0% | 0 | 0.0% |  |
| mitdb/200 | 1 | V1 | 0 | – | – | 19 | 100.0% | 22 | 100.0% | 0 | 0.0% |  |
| mitdb/207 | 0 | MLII | 0 | – | 6/7 | 101 | 100.0% | 21 | 100.0% | 0 | 45.7% |  |
| mitdb/207 | 1 | V1 | 0 | – | 7/7 | 103 | 100.0% | 21 | 100.0% | 0 | 3.1% |  |
| mitdb/208 | 0 | MLII | 0 | – | – | 88 | 100.0% | 9 | 100.0% | 0 | 0.0% |  |
| mitdb/208 | 1 | V1 | 0 | – | – | 74 | 100.0% | 9 | 100.0% | 0 | 1.2% |  |
| mitdb/212 | 0 | MLII | 0 | – | – | 146 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/212 | 1 | V1 | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/217 | 0 | MLII | 0 | – | – | 0 | – | 105 | 100.0% | 0 | 0.0% |  |
| mitdb/217 | 1 | V1 | 0 | – | – | 0 | – | 105 | 100.0% | 0 | 38.1% |  |
| mitdb/220 | 0 | MLII | 0 | – | – | 144 | 100.0% | 0 | – | 0 | 0.0% |  |
| mitdb/220 | 1 | V1 | 0 | – | – | 143 | 100.0% | 0 | – | 0 | 0.0% |  |
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
| mitdb/234 | 0 | MLII | 0 | – | – | 146 | 100.0% | 1 | 100.0% | 0 | 0.0% |  |
| mitdb/234 | 1 | V1 | 0 | – | – | 138 | 100.0% | 1 | 100.0% | 0 | 0.0% |  |

## Continuous (streaming) run

One analyser runs over each whole record and channel without resets, deciding once a second. Each per-second decision is scored against the truth when its last 6 s (the 4 s window plus the two earlier windows needed for confirmation) lie inside one clean run (same guard margins as above). The decision scored is what the analyser reported for that window (`Shockable` only once confirmed; an unconfirmed shockable window counts as "not sure").

| Database | ch | Class | seconds | Shockable | NonShockable | Not sure |
|---|---|---|---|---|---|---|
| cudb | 0 | Coarse VF | 1124 | 46.4% | 8.6% | 44.9% |
| cudb | 0 | Other non-shockable | 282 | 0.0% | 100.0% | 0.0% |
| cudb | 0 | cudb non-VF, rhythm not given | 5693 | 0.1% | 71.6% | 28.3% |
| vfdb | 0 | Coarse VF | 1064 | 69.7% | 0.8% | 29.5% |
| vfdb | 0 | Ventricular flutter | 230 | 55.7% | 5.2% | 39.1% |
| vfdb | 0 | VT, rate unknown | 3486 | 42.1% | 4.6% | 53.2% |
| vfdb | 0 | Normal sinus rhythm | 7658 | 0.1% | 87.0% | 12.9% |
| vfdb | 0 | Asystole | 677 | 2.2% | 34.9% | 62.9% |
| vfdb | 0 | Other non-shockable | 6234 | 0.0% | 77.1% | 22.9% |
| vfdb | 0 | HGEA (vfdb) | 195 | 1.0% | 79.0% | 20.0% |
| vfdb | 1 | Coarse VF | 1064 | 70.9% | 0.7% | 28.5% |
| vfdb | 1 | Ventricular flutter | 230 | 63.5% | 5.2% | 31.3% |
| vfdb | 1 | VT, rate unknown | 3486 | 30.2% | 7.7% | 62.0% |
| vfdb | 1 | Normal sinus rhythm | 7658 | 0.2% | 79.3% | 20.5% |
| vfdb | 1 | Asystole | 677 | 9.9% | 61.9% | 28.2% |
| vfdb | 1 | Other non-shockable | 6234 | 0.0% | 92.1% | 7.9% |
| vfdb | 1 | HGEA (vfdb) | 195 | 0.0% | 40.5% | 59.5% |
| mitdb | 0 | Ventricular flutter | 88 | 64.8% | 2.3% | 33.0% |
| mitdb | 0 | Slow VT (<150/min) | 70 | 0.0% | 78.6% | 21.4% |
| mitdb | 0 | Normal sinus rhythm | 29360 | 0.0% | 95.7% | 4.3% |
| mitdb | 0 | Other non-shockable | 7202 | 0.0% | 90.8% | 9.2% |
| mitdb | 1 | Ventricular flutter | 88 | 80.7% | 1.1% | 18.2% |
| mitdb | 1 | Slow VT (<150/min) | 53 | 0.0% | 100.0% | 0.0% |
| mitdb | 1 | Normal sinus rhythm | 28200 | 0.0% | 93.3% | 6.7% |
| mitdb | 1 | Other non-shockable | 6927 | 0.0% | 52.6% | 47.4% |

### Detection delay after VF / flutter onset

Measured from the annotated onset (cudb onsets are only approximate, per its documentation) to the first confirmed `Shockable` before the episode ends. The fastest possible is 7 s after the window first lies wholly in VF, i.e. about 6–7 s after onset. Episodes shorter than 7 s are counted apart. A delay under 6 s is faster than the analyser can confirm VF from scratch: it means the windows before the annotated onset already looked shockable to it (in vfdb record 418 the "NSR" between flutter bursts does). Noise marks are not removed here.

| Database | ch | label | episodes ≥7 s | detected | not detected | already shockable at onset | delay (detected) | of which under 6 s | episodes <7 s |
|---|---|---|---|---|---|---|---|---|---|
| cudb | 0 | VF | 20 | 19 | 1 | 0 | median 6.6 s, 90th pct 15.3 s, max 33.4 s | 8 | 0 |
| vfdb | 0 | VF | 7 | 7 | 0 | 0 | median 4.8 s, 90th pct 6.5 s, max 6.5 s | 4 | 0 |
| vfdb | 0 | VFL | 6 | 6 | 0 | 0 | median 5.0 s, 90th pct 7.5 s, max 8.8 s | 4 | 30 |
| vfdb | 1 | VF | 7 | 7 | 0 | 0 | median 4.8 s, 90th pct 6.2 s, max 6.5 s | 5 | 0 |
| vfdb | 1 | VFL | 6 | 6 | 0 | 0 | median 4.7 s, 90th pct 7.3 s, max 7.8 s | 4 | 30 |
| mitdb | 0 | VFL | 4 | 4 | 0 | 0 | median 6.6 s, 90th pct 7.7 s, max 7.9 s | 1 | 2 |
| mitdb | 1 | VFL | 4 | 4 | 0 | 0 | median 6.8 s, 90th pct 8.7 s, max 8.9 s | 2 | 2 |

### How long "shock" persists after a shockable rhythm ends

For VF, flutter and VT episodes followed by at least 10 s of a non-shockable (or cudb unlabelled) rhythm: the time from the annotated end to the first decision that is not `Shockable`. Zero means the analyser was not saying `Shockable` at the end. In cudb the end of VF is often a defibrillation shock, so the following seconds are shock artefact.

| Database | ch | episodes | shockable at end | persistence (those) | false shock 6–16 s after the end |
|---|---|---|---|---|---|
| cudb | 0 | 12 | 3 | median 0.6 s, 90th pct 0.9 s, max 0.9 s | 0 |
| vfdb | 0 | 17 | 3 | median 1.0 s, 90th pct 2.1 s, max 2.4 s | 2 |
| vfdb | 1 | 17 | 3 | median 3.6 s, 90th pct 7.7 s, max 8.7 s | 3 |
| mitdb | 0 | 12 | 0 | – | 0 |
| mitdb | 1 | 12 | 1 | median 0.9 s, 90th pct 0.9 s, max 0.9 s | 0 |

### Share of VF/flutter episodes reported `Shockable`, by seconds from onset (channel 0, episodes ≥ 20 s)

| Database | episodes | -5 s | +0 s | +2 s | +4 s | +5 s | +6 s | +7 s | +8 s | +9 s | +10 s | +12 s | +15 s | +20 s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cudb | 15 | 0.0% | 0.0% | 0.0% | 6.7% | 26.7% | 40.0% | 46.7% | 53.3% | 66.7% | 73.3% | 66.7% | 66.7% | 53.3% |
| vfdb | 13 | 15.4% | 0.0% | 0.0% | 7.7% | 53.8% | 61.5% | 84.6% | 84.6% | 84.6% | 84.6% | 76.9% | 61.5% | 61.5% |
| mitdb | 1 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 100.0% | 100.0% | 0.0% | 0.0% |

## Failure analysis

Nothing below was used to change the analyser. Where a threshold is "moved", it is a what-if count only.

### Which rule gave each window's verdict (streaming windows, all databases and channels)

| Class | windows | invalid measurement | saturated | clipped (flat tops) | amplitude implausible | asystole | noisy | possible artifact | fine VF (not shocked) | methods disagree | both methods shockable | neither method shockable |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.0% | 0.0% | 3.3% | 0.0% | 0.0% | 0.1% | 9.2% | 0.0% | 16.7% | 67.3% | 3.4% |
| Ventricular flutter | 636 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 3.5% | 0.0% | 22.2% | 70.1% | 4.2% |
| Slow VT (<150/min) | 123 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 12.2% | 0.0% | 87.8% |
| VT, rate unknown | 6972 | 0.0% | 0.0% | 0.0% | 0.1% | 0.1% | 0.0% | 36.1% | 0.0% | 17.4% | 40.2% | 6.1% |
| Normal sinus rhythm | 72876 | 0.0% | 0.0% | 0.0% | 0.0% | 1.2% | 0.1% | 4.6% | 0.6% | 2.5% | 0.1% | 90.9% |
| Asystole | 1354 | 0.0% | 0.0% | 0.0% | 0.0% | 36.6% | 0.7% | 27.8% | 0.0% | 15.6% | 7.5% | 11.7% |
| Other non-shockable | 26879 | 0.0% | 0.0% | 0.0% | 0.1% | 0.0% | 0.0% | 15.0% | 0.0% | 6.7% | 0.0% | 78.2% |
| cudb non-VF, rhythm not given | 5693 | 0.0% | 0.0% | 0.6% | 0.0% | 0.2% | 0.1% | 18.8% | 0.0% | 8.6% | 0.3% | 71.4% |
| HGEA (vfdb) | 390 | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 21.0% | 0.0% | 17.4% | 1.8% | 59.7% |

Exclusion rules by database (share of all scored streaming windows, any class):

| Database | windows | saturated | implausible size | flatline | noisy | low-band (compression) | fine VF |
|---|---|---|---|---|---|---|---|
| cudb | 7099 | 0.0% | 0.0% | 0.2% | 0.1% | 17.8% | 0.0% |
| vfdb | 39088 | 0.0% | 0.1% | 1.5% | 0.1% | 18.7% | 0.0% |
| mitdb | 71988 | 0.0% | 0.0% | 1.1% | 0.0% | 4.5% | 0.6% |

### Feature distributions on real data versus the thresholds

Streaming windows, all databases and channels, by true class. "VF side" = share of windows on the shockable side of that feature's threshold(s). A feature only helps if VF windows are mostly on the VF side and non-shockable windows mostly are not.

**peak-to-peak (µV, after the analyser's filter)** (shockable side: ≥ 200 for shock, < 100 = flatline, > 12000 = implausible)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 483.705 | 716.886 | 1153.978 | 2496.144 | 6462.030 | 99.2% |
| Ventricular flutter | 636 | 426.187 | 701.328 | 1072.890 | 1405.950 | 2209.298 | 100.0% |
| Slow VT (<150/min) | 123 | 1656.322 | 1694.509 | 1868.272 | 2185.313 | 2703.410 | 100.0% |
| VT, rate unknown | 6972 | 288.362 | 774.470 | 3655.444 | 5975.269 | 8169.795 | 98.6% |
| Normal sinus rhythm | 72876 | 310.115 | 870.358 | 1462.017 | 2105.989 | 3617.046 | 96.6% |
| Asystole | 1354 | 109.429 | 134.088 | 183.491 | 552.357 | 829.862 | 44.1% |
| Other non-shockable | 26879 | 986.445 | 1880.656 | 2923.290 | 4346.015 | 5759.296 | 99.9% |
| cudb non-VF, rhythm not given | 5693 | 1152.500 | 2224.153 | 2785.952 | 3314.663 | 6379.011 | 99.6% |
| HGEA (vfdb) | 390 | 571.116 | 1067.117 | 4822.362 | 5957.045 | 6991.697 | 100.0% |

**root-mean-square (µV, after the analyser's filter)** (shockable side: ≥ 25 (or leakage ≤ 0.6) to escape the flatline rule)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 100.448 | 154.896 | 239.827 | 516.330 | 1052.476 | 100.0% |
| Ventricular flutter | 636 | 93.735 | 141.014 | 222.688 | 310.319 | 492.797 | 100.0% |
| Slow VT (<150/min) | 123 | 283.709 | 306.303 | 363.643 | 421.861 | 492.625 | 100.0% |
| VT, rate unknown | 6972 | 55.649 | 172.156 | 733.602 | 1163.540 | 2069.946 | 99.9% |
| Normal sinus rhythm | 72876 | 41.172 | 114.031 | 192.183 | 292.339 | 479.631 | 97.7% |
| Asystole | 1354 | 18.941 | 22.215 | 30.454 | 85.100 | 159.669 | 63.4% |
| Other non-shockable | 26879 | 136.196 | 274.166 | 476.706 | 724.415 | 1090.602 | 100.0% |
| cudb non-VF, rhythm not given | 5693 | 169.442 | 370.473 | 440.971 | 564.028 | 827.670 | 99.8% |
| HGEA (vfdb) | 390 | 108.553 | 205.776 | 633.963 | 781.448 | 994.724 | 100.0% |

**noise ratio** (shockable side: ≤ 0.15)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.000 | 0.001 | 0.002 | 0.004 | 0.011 | 99.9% |
| Ventricular flutter | 636 | 0.000 | 0.001 | 0.002 | 0.006 | 0.010 | 100.0% |
| Slow VT (<150/min) | 123 | 0.002 | 0.002 | 0.002 | 0.002 | 0.004 | 100.0% |
| VT, rate unknown | 6972 | 0.000 | 0.001 | 0.001 | 0.005 | 0.031 | 100.0% |
| Normal sinus rhythm | 72876 | 0.002 | 0.006 | 0.016 | 0.033 | 0.068 | 99.5% |
| Asystole | 1354 | 0.004 | 0.013 | 0.103 | 0.179 | 0.244 | 65.3% |
| Other non-shockable | 26879 | 0.001 | 0.002 | 0.004 | 0.009 | 0.032 | 100.0% |
| cudb non-VF, rhythm not given | 5693 | 0.000 | 0.002 | 0.004 | 0.007 | 0.016 | 99.9% |
| HGEA (vfdb) | 390 | 0.001 | 0.002 | 0.003 | 0.004 | 0.006 | 100.0% |

**low-band fraction** (shockable side: ≤ 0.5)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.007 | 0.031 | 0.086 | 0.216 | 0.690 | 89.5% |
| Ventricular flutter | 636 | 0.010 | 0.034 | 0.065 | 0.147 | 0.450 | 96.5% |
| Slow VT (<150/min) | 123 | 0.118 | 0.148 | 0.190 | 0.326 | 0.374 | 100.0% |
| VT, rate unknown | 6972 | 0.006 | 0.049 | 0.250 | 0.581 | 0.668 | 63.9% |
| Normal sinus rhythm | 72876 | 0.027 | 0.080 | 0.166 | 0.271 | 0.489 | 95.4% |
| Asystole | 1354 | 0.053 | 0.149 | 0.272 | 0.574 | 0.786 | 71.3% |
| Other non-shockable | 26879 | 0.062 | 0.175 | 0.295 | 0.433 | 0.613 | 84.9% |
| cudb non-VF, rhythm not given | 5693 | 0.105 | 0.157 | 0.236 | 0.456 | 0.639 | 80.7% |
| HGEA (vfdb) | 390 | 0.138 | 0.232 | 0.347 | 0.476 | 0.603 | 79.0% |

**threshold-crossing fraction (TCSC)** (shockable side: > 0.45)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.559 | 0.648 | 0.701 | 0.752 | 0.815 | 98.1% |
| Ventricular flutter | 636 | 0.577 | 0.671 | 0.729 | 0.770 | 0.815 | 100.0% |
| Slow VT (<150/min) | 123 | 0.422 | 0.454 | 0.494 | 0.741 | 0.785 | 78.0% |
| VT, rate unknown | 6972 | 0.527 | 0.660 | 0.690 | 0.732 | 0.817 | 96.9% |
| Normal sinus rhythm | 72876 | 0.062 | 0.119 | 0.240 | 0.360 | 0.581 | 13.5% |
| Asystole | 1354 | 0.447 | 0.548 | 0.590 | 0.629 | 0.683 | 94.6% |
| Other non-shockable | 26879 | 0.144 | 0.256 | 0.371 | 0.542 | 0.633 | 36.5% |
| cudb non-VF, rhythm not given | 5693 | 0.169 | 0.268 | 0.387 | 0.549 | 0.725 | 39.2% |
| HGEA (vfdb) | 390 | 0.281 | 0.392 | 0.493 | 0.646 | 0.726 | 59.0% |

**band-pass count** (shockable side: > 0.16)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.104 | 0.170 | 0.224 | 0.277 | 0.346 | 78.3% |
| Ventricular flutter | 636 | 0.129 | 0.178 | 0.220 | 0.265 | 0.335 | 84.6% |
| Slow VT (<150/min) | 123 | 0.051 | 0.058 | 0.070 | 0.083 | 0.094 | 0.0% |
| VT, rate unknown | 6972 | 0.058 | 0.174 | 0.215 | 0.255 | 0.348 | 80.8% |
| Normal sinus rhythm | 72876 | 0.028 | 0.039 | 0.050 | 0.064 | 0.105 | 1.2% |
| Asystole | 1354 | 0.091 | 0.130 | 0.160 | 0.190 | 0.231 | 49.9% |
| Other non-shockable | 26879 | 0.031 | 0.052 | 0.067 | 0.087 | 0.119 | 0.1% |
| cudb non-VF, rhythm not given | 5693 | 0.040 | 0.052 | 0.062 | 0.100 | 0.185 | 11.7% |
| HGEA (vfdb) | 390 | 0.051 | 0.070 | 0.087 | 0.114 | 0.156 | 3.8% |

**VF-filter leakage** (shockable side: < 0.625)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.249 | 0.362 | 0.468 | 0.577 | 0.686 | 86.7% |
| Ventricular flutter | 636 | 0.240 | 0.333 | 0.439 | 0.578 | 0.713 | 82.9% |
| Slow VT (<150/min) | 123 | 0.620 | 0.638 | 0.686 | 0.710 | 0.751 | 12.2% |
| VT, rate unknown | 6972 | 0.233 | 0.397 | 0.502 | 0.561 | 0.758 | 85.2% |
| Normal sinus rhythm | 72876 | 0.628 | 0.732 | 0.788 | 0.830 | 0.860 | 4.6% |
| Asystole | 1354 | 0.577 | 0.796 | 0.840 | 0.874 | 0.901 | 8.3% |
| Other non-shockable | 26879 | 0.594 | 0.650 | 0.722 | 0.766 | 0.826 | 15.6% |
| cudb non-VF, rhythm not given | 5693 | 0.571 | 0.642 | 0.727 | 0.788 | 0.854 | 19.1% |
| HGEA (vfdb) | 390 | 0.542 | 0.616 | 0.655 | 0.708 | 0.773 | 29.7% |

**dominant frequency (Hz)** (shockable side: > 2.4 and < 9)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 1.250 | 2.750 | 3.500 | 4.000 | 5.500 | 86.5% |
| Ventricular flutter | 636 | 1.500 | 3.250 | 3.500 | 3.750 | 4.250 | 92.5% |
| Slow VT (<150/min) | 123 | 1.750 | 1.750 | 2.500 | 2.750 | 2.750 | 57.7% |
| VT, rate unknown | 6972 | 1.750 | 2.250 | 2.750 | 3.500 | 4.500 | 57.4% |
| Normal sinus rhythm | 72876 | 1.000 | 1.750 | 2.750 | 3.500 | 7.750 | 53.5% |
| Asystole | 1354 | 1.000 | 1.250 | 2.750 | 4.250 | 5.000 | 52.1% |
| Other non-shockable | 26879 | 1.250 | 1.750 | 2.250 | 2.500 | 4.750 | 40.5% |
| cudb non-VF, rhythm not given | 5693 | 1.250 | 1.500 | 2.250 | 3.250 | 5.500 | 46.4% |
| HGEA (vfdb) | 390 | 1.250 | 2.000 | 2.250 | 3.000 | 3.500 | 45.9% |

**spectral concentration** (shockable side: > 0.4)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.408 | 0.621 | 0.804 | 0.885 | 0.948 | 95.3% |
| Ventricular flutter | 636 | 0.427 | 0.699 | 0.837 | 0.914 | 0.952 | 96.5% |
| Slow VT (<150/min) | 123 | 0.259 | 0.311 | 0.471 | 0.493 | 0.510 | 56.9% |
| VT, rate unknown | 6972 | 0.431 | 0.686 | 0.800 | 0.877 | 0.958 | 96.3% |
| Normal sinus rhythm | 72876 | 0.119 | 0.196 | 0.280 | 0.386 | 0.578 | 23.4% |
| Asystole | 1354 | 0.216 | 0.380 | 0.492 | 0.614 | 0.789 | 71.4% |
| Other non-shockable | 26879 | 0.182 | 0.321 | 0.396 | 0.467 | 0.559 | 48.2% |
| cudb non-VF, rhythm not given | 5693 | 0.140 | 0.252 | 0.404 | 0.514 | 0.670 | 51.1% |
| HGEA (vfdb) | 390 | 0.236 | 0.430 | 0.569 | 0.647 | 0.759 | 78.2% |

**saturated fraction (front-end limit)** (shockable side: ≤ 0.01)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Ventricular flutter | 636 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Slow VT (<150/min) | 123 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| VT, rate unknown | 6972 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Normal sinus rhythm | 72876 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Asystole | 1354 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Other non-shockable | 26879 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| cudb non-VF, rhythm not given | 5693 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| HGEA (vfdb) | 390 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |

**clipped fraction (flat tops)** (shockable side: ≤ 0.01)

| Class | windows | 5% | 25% | median | 75% | 95% | VF side |
|---|---|---|---|---|---|---|---|
| Coarse VF | 3252 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 96.7% |
| Ventricular flutter | 636 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Slow VT (<150/min) | 123 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| VT, rate unknown | 6972 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Normal sinus rhythm | 72876 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Asystole | 1354 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| Other non-shockable | 26879 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |
| cudb non-VF, rhythm not given | 5693 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 99.4% |
| HGEA (vfdb) | 390 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0% |

### Missed coarse VF (12 s analyses, all channels)

63 of 269 coarse-VF segments did not end in a shock: 23 ended "no shock", 40 inconclusive.

How the missed ones ended (the verdict of the window that ended the analysis, or for inconclusive ones, the most common verdict among its windows):

| Ended by | segments |
|---|---|
| clipped (flat tops) | 5 |
| methods disagree | 15 |
| neither method shockable | 23 |
| possible artifact | 20 |

Which conditions failed, over 394 windows of missed coarse-VF segments (5.3% passed all of them). "Only blocker" = the window failed that condition and no other, so that single threshold alone kept it from being shockable.

| Condition | rule | failed (any) | only blocker |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% | 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 0.01 | 10.2% | 1.3% |
| plausible size | peak-to-peak ≤ 12000 µV | 3.3% | 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 25 µV with leakage > 0.6) | 0.0% | 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.0% | 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 47.2% | 1.3% |
| TCSC | threshold-crossing fraction > 0.45 | 4.1% | 0.0% |
| band-pass count | band-pass count > 0.16 | 56.9% | 18.5% |
| VF-filter leakage | leakage < 0.625 | 41.1% | 14.7% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 50.8% | 1.3% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.3% | 0.0% |
| spectral concentration | concentration > 0.4 | 9.9% | 0.0% |
| not fine VF | peak-to-peak ≥ 200 µV | 0.0% | 0.0% |

Which conditions failed, over 3252 all coarse-VF streaming windows (67.3% passed all of them). "Only blocker" = the window failed that condition and no other, so that single threshold alone kept it from being shockable.

| Condition | rule | failed (any) | only blocker |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% | 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 0.01 | 3.3% | 0.5% |
| plausible size | peak-to-peak ≤ 12000 µV | 0.8% | 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 25 µV with leakage > 0.6) | 0.0% | 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.1% | 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 10.5% | 0.2% |
| TCSC | threshold-crossing fraction > 0.45 | 1.9% | 0.0% |
| band-pass count | band-pass count > 0.16 | 21.7% | 10.2% |
| VF-filter leakage | leakage < 0.625 | 13.3% | 4.4% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 13.4% | 0.6% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.1% | 0.0% |
| spectral concentration | concentration > 0.4 | 4.7% | 0.4% |
| not fine VF | peak-to-peak ≥ 200 µV | 0.0% | 0.0% |

### False shocks on non-shockable rhythms (12 s analyses, all channels)

Worst rhythm classes by false-shock rate:

| Rhythm | false shocks | segments | rate |
|---|---|---|---|
| ASYS | 7 | 112 | 6.2% |
| NSR | 1 | 6073 | 0.0% |
| AB (atrial bigeminy) | 0 | 10 | 0.0% |
| AFIB | 0 | 204 | 0.0% |
| AFL | 0 | 24 | 0.0% |
| B (ventricular bigeminy) | 0 | 224 | 0.0% |
| BI (1st-degree block) | 0 | 54 | 0.0% |
| BII (2nd-degree block) | 0 | 104 | 0.0% |
| IVR (idioventricular) | 0 | 16 | 0.0% |
| NOD (junctional) | 0 | 280 | 0.0% |
| PACED | 0 | 1014 | 0.0% |
| PREX (pre-excitation) | 0 | 31 | 0.0% |
| SBR | 0 | 2 | 0.0% |
| SVTA | 0 | 240 | 0.0% |
| T (ventricular trigeminy) | 0 | 18 | 0.0% |
| VER (ventricular escape) | 0 | 4 | 0.0% |

Every false shock passed all 13 conditions in at least 3 of the last 5 windows (that is the only way to reach `Shockable`), so "which rule fired" is always "both method families said shockable". The useful question is which condition was closest to stopping it. For each condition: the share of the confirming windows of every false shock that would have been stopped had that one threshold been 10 % / 20 % stricter, against the share of correctly shocked coarse-VF confirming windows that the same change would also stop (the cost). What-if counts only; nothing was changed.

| Condition | rule | false-shock windows stopped at 10% / 20% stricter (of 24) | coarse-VF windows also stopped at 10% / 20% (of 618) |
|---|---|---|---|
| not saturated | saturated fraction ≤ 0.01 | 0.0% / 0.0% | 0.0% / 0.0% |
| not clipped (flat tops) | clipped fraction ≤ 0.01 | 0.0% / 0.0% | 0.0% / 0.0% |
| plausible size | peak-to-peak ≤ 12000 µV | 0.0% / 0.0% | 0.0% / 0.0% |
| not flatline | peak-to-peak ≥ 100 µV, and not (rms < 25 µV with leakage > 0.6) | 0.0% / 0.0% | 0.0% / 0.0% |
| not noisy | noise ratio ≤ 0.15 | 0.0% / 0.0% | 0.0% / 0.0% |
| not low-band (compression) | low-band fraction ≤ 0.5 | 8.3% / 12.5% | 0.2% / 0.3% |
| TCSC | threshold-crossing fraction > 0.45 | 0.0% / 0.0% | 0.0% / 0.0% |
| band-pass count | band-pass count > 0.16 | 0.0% / 8.3% | 5.8% / 15.5% |
| VF-filter leakage | leakage < 0.625 | 45.8% / 83.3% | 14.9% / 29.8% |
| dominant freq. low limit | dominant frequency > 2.4 Hz | 0.0% / 0.0% | 3.6% / 16.2% |
| dominant freq. high limit | dominant frequency < 9 Hz | 0.0% / 0.0% | 0.0% / 0.0% |
| spectral concentration | concentration > 0.4 | 0.0% / 0.0% | 1.0% / 3.4% |
| not fine VF | peak-to-peak ≥ 200 µV | 25.0% / 25.0% | 0.0% / 0.0% |

How often each condition already lets a non-shockable window through (streaming NSR + other non-shockable windows, n = 99755): a condition passed by nearly every non-shockable window does no work.

| Condition | non-shockable windows on the VF side | coarse-VF windows on the VF side |
|---|---|---|
| not saturated | 100.0% | 100.0% |
| not clipped (flat tops) | 100.0% | 96.7% |
| plausible size | 100.0% | 99.2% |
| not flatline | 99.1% | 100.0% |
| not noisy | 99.6% | 99.9% |
| not low-band (compression) | 92.5% | 89.5% |
| TCSC | 19.7% | 98.1% |
| band-pass count | 0.9% | 78.3% |
| VF-filter leakage | 7.6% | 86.7% |
| dominant freq. low limit | 52.0% | 86.6% |
| dominant freq. high limit | 98.0% | 99.9% |
| spectral concentration | 30.1% | 95.3% |
| not fine VF | 97.5% | 100.0% |

False shocks by record and channel (8 in all):

| Record | false shocks | on rhythm(s) |
|---|---|---|
| vfdb/607 ch1 | 5 | ASYS |
| vfdb/607 ch0 | 2 | ASYS |
| vfdb/419 ch1 | 1 | NSR |

### VT without beat annotations, split by a rough rate estimated from the signal

vfdb gives VT episodes but no beats, so the AHA rate criterion cannot be applied from the annotations. As a rough guide only, each segment's rate was estimated from the signal's strongest repetition (autocorrelation, 60–300/min). This helper is UNVERIFIED and can be off by a factor of two; it is not used for any AHA row above. 12 s analyses, both channels counted.

| Estimated rate | segments | records | shock | no shock | inconcl. | shock rate |
|---|---|---|---|---|---|---|
| ≥ 150/min (would be rapid VT) | 326 | 7 | 229 | 56 | 41 | 70.2% |
| < 150/min (would be slow VT) | 215 | 7 | 10 | 22 | 183 | 4.7% |
| no clear rate | 37 | 3 | 9 | 12 | 16 | 24.3% |

### Every false shock (8 in all, both channels)

Normal rhythm first, then asystole, then by how fast the shock came. Features are those of the window that confirmed the shock.

| Record | ch | start (s) | true rhythm | amplitude (µV) | rate | outcome | at (s) | window verdict | features |
|---|---|---|---|---|---|---|---|---|---|
| vfdb/419 | 1 | 3.1 | NSR | 742 | – | shock | 9 | both methods shockable | pp 702 µV, noise 0.00, low-band 0.48, TCSC 0.75, count 0.18, leak 0.58, dom 3.00 Hz, conc 0.56 |
| vfdb/607 | 0 | 1764.5 | ASYS | 227 | – | shock | 7 | both methods shockable | pp 201 µV, noise 0.07, low-band 0.06, TCSC 0.65, count 0.22, leak 0.62, dom 4.50 Hz, conc 0.81 |
| vfdb/607 | 0 | 1776.5 | ASYS | 227 | – | shock | 7 | both methods shockable | pp 216 µV, noise 0.07, low-band 0.02, TCSC 0.68, count 0.23, leak 0.61, dom 4.00 Hz, conc 0.84 |
| vfdb/607 | 1 | 1716.5 | ASYS | 321 | – | shock | 7 | both methods shockable | pp 389 µV, noise 0.05, low-band 0.09, TCSC 0.68, count 0.27, leak 0.61, dom 4.25 Hz, conc 0.58 |
| vfdb/607 | 1 | 1728.5 | ASYS | 373 | – | shock | 7 | both methods shockable | pp 468 µV, noise 0.02, low-band 0.08, TCSC 0.68, count 0.28, leak 0.50, dom 4.25 Hz, conc 0.79 |
| vfdb/607 | 1 | 1740.5 | ASYS | 313 | – | shock | 7 | both methods shockable | pp 348 µV, noise 0.04, low-band 0.03, TCSC 0.64, count 0.20, leak 0.52, dom 4.75 Hz, conc 0.88 |
| vfdb/607 | 1 | 1764.5 | ASYS | 374 | – | shock | 7 | both methods shockable | pp 367 µV, noise 0.03, low-band 0.05, TCSC 0.70, count 0.24, leak 0.50, dom 4.50 Hz, conc 0.83 |
| vfdb/607 | 1 | 1776.5 | ASYS | 379 | – | shock | 7 | both methods shockable | pp 394 µV, noise 0.03, low-band 0.01, TCSC 0.72, count 0.28, leak 0.51, dom 4.00 Hz, conc 0.86 |

### The 20 worst missed VF segments

Largest VF first (a missed large VF is the worst kind). Features are those of the last window looked at.

| Record | ch | start (s) | true rhythm | amplitude (µV) | rate | outcome | at (s) | window verdict | features |
|---|---|---|---|---|---|---|---|---|---|
| cudb/cu12 | 0 | 312.3 | VF | 10187 | – | no-shock | 5 | neither method shockable | pp 11352 µV, noise 0.12, low-band 0.02, TCSC 0.37, count 0.17, leak 0.71, dom 9.25 Hz, conc 0.42 |
| cudb/cu12 | 0 | 372.3 | VF | 7041 | – | inconclusive | – | clipped (flat tops) | pp 11043 µV, noise 0.01, low-band 0.46, TCSC 0.37, count 0.09, leak 0.79, dom 1.25 Hz, conc 0.32 |
| cudb/cu12 | 0 | 384.3 | VF | 6815 | – | no-shock | 8 | neither method shockable | pp 8986 µV, noise 0.02, low-band 0.21, TCSC 0.37, count 0.05, leak 0.79, dom 1.00 Hz, conc 0.12 |
| cudb/cu34 | 0 | 403.0 | VF | 6339 | – | inconclusive | – | clipped (flat tops) | pp 8512 µV, noise 0.01, low-band 0.03, TCSC 0.64, count 0.13, leak 0.50, dom 3.75 Hz, conc 0.85 |
| cudb/cu12 | 0 | 288.3 | VF | 5880 | – | no-shock | 6 | neither method shockable | pp 6116 µV, noise 0.02, low-band 0.03, TCSC 0.66, count 0.15, leak 0.73, dom 3.25 Hz, conc 0.59 |
| cudb/cu12 | 0 | 348.3 | VF | 5535 | – | no-shock | 5 | neither method shockable | pp 6031 µV, noise 0.02, low-band 0.16, TCSC 0.48, count 0.08, leak 0.73, dom 3.75 Hz, conc 0.68 |
| cudb/cu10 | 0 | 487.5 | VF | 4613 | – | inconclusive | – | possible artifact | pp 7315 µV, noise 0.00, low-band 0.51, TCSC 0.56, count 0.18, leak 0.59, dom 1.25 Hz, conc 0.45 |
| cudb/cu12 | 0 | 336.3 | VF | 4367 | – | no-shock | 5 | neither method shockable | pp 8282 µV, noise 0.08, low-band 0.10, TCSC 0.25, count 0.04, leak 0.79, dom 4.00 Hz, conc 0.47 |
| cudb/cu20 | 0 | 331.2 | VF | 4106 | – | inconclusive | – | possible artifact | pp 3917 µV, noise 0.00, low-band 0.93, TCSC 0.77, count 0.11, leak 0.54, dom 1.00 Hz, conc 0.67 |
| cudb/cu20 | 0 | 307.2 | VF | 3806 | – | inconclusive | – | possible artifact | pp 3277 µV, noise 0.00, low-band 0.95, TCSC 0.78, count 0.18, leak 0.49, dom 1.00 Hz, conc 0.38 |
| cudb/cu06 | 0 | 281.6 | VF | 3655 | – | no-shock | 8 | neither method shockable | pp 5639 µV, noise 0.00, low-band 0.48, TCSC 0.65, count 0.08, leak 0.64, dom 2.25 Hz, conc 0.69 |
| cudb/cu20 | 0 | 319.2 | VF | 3536 | – | inconclusive | – | possible artifact | pp 4132 µV, noise 0.00, low-band 0.90, TCSC 0.81, count 0.19, leak 0.48, dom 1.00 Hz, conc 0.66 |
| cudb/cu12 | 0 | 360.3 | VF | 3500 | – | no-shock | 5 | neither method shockable | pp 3672 µV, noise 0.02, low-band 0.12, TCSC 0.49, count 0.10, leak 0.72, dom 4.25 Hz, conc 0.37 |
| cudb/cu20 | 0 | 271.2 | VF | 3471 | – | inconclusive | – | methods disagree | pp 3278 µV, noise 0.00, low-band 0.40, TCSC 0.60, count 0.25, leak 0.66, dom 4.50 Hz, conc 0.52 |
| cudb/cu20 | 0 | 391.2 | VF | 3313 | – | inconclusive | – | possible artifact | pp 3547 µV, noise 0.00, low-band 0.64, TCSC 0.52, count 0.08, leak 0.65, dom 2.25 Hz, conc 0.67 |
| cudb/cu20 | 0 | 487.2 | VF | 3296 | – | inconclusive | – | possible artifact | pp 3186 µV, noise 0.00, low-band 0.56, TCSC 0.51, count 0.12, leak 0.57, dom 2.25 Hz, conc 0.79 |
| cudb/cu12 | 0 | 396.3 | VF | 3115 | – | no-shock | 5 | neither method shockable | pp 11357 µV, noise 0.01, low-band 0.46, TCSC 0.53, count 0.14, leak 0.80, dom 1.50 Hz, conc 0.37 |
| cudb/cu20 | 0 | 427.2 | VF | 3106 | – | no-shock | 11 | neither method shockable | pp 6399 µV, noise 0.01, low-band 0.43, TCSC 0.40, count 0.06, leak 0.54, dom 2.00 Hz, conc 0.64 |
| cudb/cu20 | 0 | 439.2 | VF | 2971 | – | inconclusive | – | possible artifact | pp 2693 µV, noise 0.00, low-band 0.63, TCSC 0.65, count 0.11, leak 0.66, dom 1.00 Hz, conc 0.40 |
| cudb/cu20 | 0 | 403.2 | VF | 2756 | – | inconclusive | – | possible artifact | pp 4108 µV, noise 0.00, low-band 0.71, TCSC 0.65, count 0.12, leak 0.58, dom 2.25 Hz, conc 0.68 |

## Amplitude calibration and the microvolt rules

The analyser has three absolute-size rules: under 100 µV peak to peak is flatline (no shock); under 200 µV VF is "fine" (not shocked); over 12000 µV is implausible (not sure). They only mean something if the recording's microvolts are right. cudb's gain of 400 ADC units/mV is described by PhysioNet as nominal (10 mV full scale "nominal"; cu01 was replayed from a Holter tape). Nothing here was rescaled.

Per database, channel 0: the median over records of each record's median ground-truth amplitude (1-s peak to peak, drift removed) in clean streaming windows, and how VF windows fall against the analyser's own size rules (its peak-to-peak is measured after its 1–30 Hz filter, so it reads lower than the ground-truth figure).

| Database | non-VF amplitude (median of record medians) | VF amplitude (median of record medians) | VF windows | analyser pp < 200 µV (fine-VF rule) | analyser pp < 100 µV (flatline rule) | ground truth < 200 µV |
|---|---|---|---|---|---|---|
| cudb | 2605 µV (16 records) | 2797 µV (13 records) | 1124 | 0.0% | 0.0% | 0.0% |
| vfdb | 1274 µV (11 records) | 602 µV (2 records) | 1064 | 0.0% | 0.0% | 0.0% |
| mitdb | 1999 µV (24 records) | – | 0 | – | – | – |

Per cudb record, channel 0 (to show whether the nominal calibration is consistent between records):

| Record | non-VF amplitude (median, µV) | VF amplitude (median, µV) | VF windows called fine VF | VF windows called flatline |
|---|---|---|---|---|
| cu03 | 1549 | 1373 | 0.0% | 0.0% |
| cu05 | 2634 | 3473 | 0.0% | 0.0% |
| cu06 | 3262 | 2386 | 0.0% | 0.0% |
| cu08 | – | – | – | – |
| cu10 | 2555 | 2237 | 0.0% | 0.0% |
| cu12 | 3585 | 3508 | 0.0% | 0.0% |
| cu13 | 2576 | – | – | – |
| cu14 | 3235 | – | – | – |
| cu18 | 3103 | 3936 | 0.0% | 0.0% |
| cu19 | 2409 | 4666 | 0.0% | 0.0% |
| cu20 | 2846 | 2797 | 0.0% | 0.0% |
| cu22 | 2807 | 2207 | 0.0% | 0.0% |
| cu24 | 2486 | 2455 | 0.0% | 0.0% |
| cu28 | 1801 | – | – | – |
| cu31 | 3551 | 3372 | 0.0% | 0.0% |
| cu33 | 2303 | 2164 | 0.0% | 0.0% |
| cu34 | 1512 | 5962 | 0.0% | 0.0% |
| cu35 | – | – | – | – |

## Reproducing this report

```
cargo run --release -p aed-eval -- --data <path to data/physionet> --csv <out dir>
```

The data are not in the repository; see `firmware/crates/aed-eval/README.md` for where to get them, their licences and the citations they ask for.
