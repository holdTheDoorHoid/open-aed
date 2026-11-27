# Rhythm analyser on the Cardially out-of-hospital VF recordings (generated)

- **Generated:** 2026-11-27 by `aed-cardially`, commit `ec5564a` (no uncommitted changes in `aed-rhythm` or `aed-types`), analyser settings `Settings::DEFAULT` (frozen, untuned on these data).
- **Data:** `data/mendeley/cardially (Mendeley Data wpr5nzyn2z version 1)` (zip SHA-256 `a3bce9bdb07a752f5d055a0ad742e227dade961b258518824069cb7c15fca735`).
- DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE. Aggregates only; per-recording results are in the CSV, which is not committed.

## 1. Reading the files

| | count |
|---|---|
| PDF reports found | 260 |
| … labelled ROEA | 56 |
| … labelled noROEA | 195 |
| … labelled indeterminable | 9 |
| PDF reports read | 260 |
| identical ECG to an earlier report (duplicates, left out) | 9 |
| sampling rate of the drawn trace | 200/s: 260 |
| 10 s strips per report | 3: 3, 5: 1, 6: 254, 7: 2 |
| printed scale | 2 div/mV, 5 div/s, div = 14.17 pt: 260 |
| samples missing / vertices off the sample grid / conflicting / clock breaks between strips (all reports) | 0 / 0 / 0 / 0 |
| text files paired by name and read | 258 |
| text files with no PDF of the same name / PDFs with no text file | 2 / 2 |


**Evaluated: 251 distinct recordings** (labelled ROEA 51, noROEA 191, indeterminable 9).

Shock 1 mark, seconds from the report's start: min 10.000, median 10.450, max 10.991. Post-shock signal analysed (to the next shock or the report's end): median 49.376 s, min 15.480 s. Recordings with a second shock inside the report: 45.

Pre-shock VF amplitude as printed (median one-second peak-to-peak, drift removed; the measure used for PhysioNet): 5 % 193, 25 % 390, median 606, 75 % 858, 95 % 1219 µV. Below 200 µV (AHA "fine VF"): 14 recordings.

**The report clips the drawn trace** at the edges of each strip's box, ±2001 µV about the strip's centre line at the printed scale (all reports). Share of the pre-shock 9 s clipped this way: none in 248 recordings, more than 1 % in 0, more than 5 % in 0; largest 0.9 %. The clipped samples are flat in the report but no longer exactly flat after conversion to 250/s, so the analyser's flat-top check cannot see them.

Text file versus PDF trace (249 pairs; best alignment at 50/s): correlation 5 % 0.947, median 0.982; text/PDF amplitude ratio 5 % 0.868, median 0.995, 95 % 1.076; text ends this many seconds before the shock mark: 5 % 0.191, median 0.311, 95 % 0.411.

Text files at a clearly different amplitude from their PDF (ratio outside 0.7–1.3): 4; text files that do not match their PDF (correlation ≤ 0.85): 1.

## 2. Before the first shock (all recordings are VF by the dataset's definition)

Device-style analysis: a fresh analyser fed the segment; outcome is its first final answer; no final answer by the end of the segment = "inconclusive". The earliest possible "shock" is at 7 s.

| Segment | n | shock | no shock | inconclusive | shock decided at 7 / 8 / 9 / later s |
|---|---|---|---|---|---|
| **9 s before the shock, as printed (headline)** | 251 | 61.4 % (154 of 251; lower 90 % limit 57.2 %) | 15.5 % | 23.1 % | 108 / 31 / 15 / 0 |
| … labelled ROEA | 51 | 78.4 % (40 of 51; lower 90 % limit 69.3 %) | 5.9 % | 15.7 % | 33 / 6 / 1 / 0 |
| … labelled noROEA | 191 | 57.6 % (110 of 191; lower 90 % limit 52.7 %) | 17.8 % | 24.6 % | 74 / 23 / 13 / 0 |
| … labelled indeterminable | 9 | 44.4 % (4 of 9; lower 90 % limit 21.0 %) | 22.2 % | 33.3 % | 1 / 2 / 1 / 0 |
| … amplitude ≥ 200 µV (coarse VF) | 237 | 63.7 % (151 of 237; lower 90 % limit 59.4 %) | 13.9 % | 22.4 % | 107 / 30 / 14 / 0 |
| all signal before the shock (9.5–10.5 s) | 251 | 60.6 % (152 of 251; lower 90 % limit 56.4 %) | 19.1 % | 20.3 % | 111 / 29 / 10 / 2 |
| the dataset's digitised text file (9 s, mV as given) | 249 | 59.4 % (148 of 249; lower 90 % limit 55.2 %) | 16.9 % | 23.7 % | 119 / 21 / 8 / 0 |

*Artificial variant, not a result:* the 9 s played twice, as one 12 s analysis: shock 66.5 % (167 of 251; lower 90 % limit 62.4 %) ; of those, 60 cancelled in the 3 s countdown. The join puts a jump in the signal 9 s in, which the analyser's filters and the 4 s windows straddling it see; windows 10–12 s and the countdown re-use the same VF, so this overstates what more real signal would show.

**Shockable window verdicts** (each 4 s window on its own, before confirmation; 5 windows per 9 s segment): 60.6 % shockable, 6.0 % not shockable, 33.4 % not sure, of 1255.

| shockable windows of 5 | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| recordings | 46 | 23 | 26 | 29 | 36 | 91 |

Window verdict reasons: both methods shockable 60.6 %; fine VF (not shocked) 0.6 %; methods disagree 24.1 %; neither method shockable 6.0 %; possible artifact 8.8 %.

### Amplitude assumption (9 s segment)

| Assumption | shock | no shock | inconclusive |
|---|---|---|---|
| as printed (10 mm/mV) | 61.4 % (154 of 251; lower 90 % limit 57.2 %) | 15.5 % | 23.1 % |
| printed scale x 0.25 | 38.6 % (97 of 251; lower 90 % limit 34.6 %) | 35.9 % | 25.5 % |
| printed scale x 0.5 | 57.8 % (145 of 251; lower 90 % limit 53.5 %) | 19.1 % | 23.1 % |
| printed scale x 2 | 62.2 % (156 of 251; lower 90 % limit 58.0 %) | 15.5 % | 22.3 % |
| printed scale x 4 | 61.8 % (155 of 251; lower 90 % limit 57.6 %) | 15.5 % | 22.7 % |
| each normalised to 1000 µV p-p | 62.2 % (156 of 251; lower 90 % limit 58.0 %) | 15.5 % | 22.3 % |

### The countdown (finding F-11)

- **9 s segment:** 154 shock decisions. Signal left after the decision: none for 15, the full 3 s for 0. Of the 139 with at least one more window to see, 23 had an answer other than "shockable" in the remaining time (would have cancelled the shock); since most saw only 1–2 of the 3 countdown windows, this is a lower bound. Of the 0 with the whole countdown visible, 0 would have been cancelled (–).
- **all signal before the shock:** 152 shock decisions. Signal left after the decision: none for 2, the full 3 s for 55. Of the 150 with at least one more window to see, 33 had an answer other than "shockable" in the remaining time (would have cancelled the shock); since most saw only 1–2 of the 3 countdown windows, this is a lower bound. Of the 55 with the whole countdown visible, 17 would have been cancelled (30.9 %).

### Per-window features on these VF recordings versus the thresholds

All windows of the 9 s segments, as printed (1255 windows, 251 recordings). "VF side" = share of windows on the shockable side of that rule.

| Feature | threshold (VF side) | 5 % | 25 % | median | 75 % | 95 % | VF side |
|---|---|---|---|---|---|---|---|
| peak-to-peak, after filter (µV) | ≥ 200 and ≤ 12000 | 256 | 493 | 778 | 1123 | 1768 | 98.8 % |
| root-mean-square (µV) | ≥ 25 (or leakage ≤ 0.6) | 45.874 | 92.820 | 146 | 211 | 330 | 100.0 % |
| noise ratio | ≤ 0.15 | 0.000 | 0.000 | 0.001 | 0.001 | 0.002 | 100.0 % |
| low-band fraction | ≤ 0.5 | 0.011 | 0.047 | 0.141 | 0.318 | 0.579 | 91.2 % |
| TCSC | > 0.45 | 0.562 | 0.618 | 0.655 | 0.688 | 0.740 | 99.6 % |
| band-pass count | > 0.16 | 0.130 | 0.171 | 0.204 | 0.238 | 0.287 | 82.4 % |
| VF-filter leakage | < 0.625 | 0.339 | 0.443 | 0.515 | 0.600 | 0.693 | 82.9 % |
| dominant frequency (Hz) | > 2.4 and < 9 | 1.500 | 2.500 | 3.500 | 4.500 | 6.250 | 77.2 % |
| spectral concentration | > 0.4 | 0.344 | 0.528 | 0.669 | 0.798 | 0.899 | 90.5 % |
| clipped fraction (flat tops) | ≤ 0.01 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0 % |
| saturated fraction | ≤ 0.01 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 100.0 % |
| 1-s peak-to-peak median (µV) | (not a rule) | 195 | 374 | 592 | 840 | 1293 | 100.0 % |

Rules failed by the 494 windows that were not shockable (a window can fail several):

| Rule | windows failing it | windows where it is the only failure |
|---|---|---|
| dominant freq <= 2.4 Hz | 57.7 % | 8.1 % |
| band-pass count | 44.7 % | 22.9 % |
| VF-filter leakage | 43.5 % | 8.7 % |
| concentration | 24.1 % | 1.2 % |
| low band (compression rule) | 22.3 % | 0.4 % |
| fine VF (< 200 µV) | 3.0 % | 1.4 % |
| TCSC | 1.0 % | 0.0 % |
| dominant freq >= 9 Hz | 0.2 % | 0.2 % |

## 3. After the first shock (description, not scoring)

One analyser run without resets from the "Shock 1 delivered" mark to the next shock or the report's end. The labels are about the shock's outcome, not second-by-second rhythm (and "noROEA" includes asystole and slow rhythms, not only continuing VF), so nothing here is a sensitivity or specificity.

Share of window verdicts that were **shockable** / **not shockable** / not sure, by seconds after the shock (window end time):

| Label | n | 0–10 s | 10–20 s | 20–30 s | 30–45 s | 45–61 s |
|---|---|---|---|---|---|---|
| ROEA | 51 | 1.0 % / 44.4 % / 54.6 % (n 306) | 4.7 % / 52.9 % / 42.4 % (n 510) | 5.1 % / 51.1 % / 43.8 % (n 491) | 5.0 % / 51.5 % / 43.4 % (n 714) | 11.6 % / 55.1 % / 33.3 % (n 198) |
| noROEA | 191 | 8.9 % / 39.5 % / 51.6 % (n 1146) | 15.7 % / 46.2 % / 38.1 % (n 1875) | 21.1 % / 37.2 % / 41.7 % (n 1651) | 26.0 % / 28.0 % / 46.0 % (n 2337) | 27.5 % / 21.0 % / 51.4 % (n 599) |
| indeterminable | 9 | 0.0 % / 53.7 % / 46.3 % (n 54) | 0.0 % / 71.1 % / 28.9 % (n 90) | 0.0 % / 60.0 % / 40.0 % (n 90) | 0.0 % / 62.2 % / 37.8 % (n 135) | 0.0 % / 66.7 % / 33.3 % (n 36) |

Recordings where the continuous run **confirmed "shockable"** at least once after the shock, and where it said "not shockable" at least once:

| Label | n | confirmed shockable | not shockable at least once | Philips device itself delivered another shock in the report |
|---|---|---|---|---|
| ROEA | 51 | 23.5 % | 94.1 % | 5.9 % |
| noROEA | 191 | 57.1 % | 90.1 % | 22.0 % |
| indeterminable | 9 | 0.0 % | 88.9 % | 0.0 % |

Post-shock window reasons (all labels, 10232 windows): asystole 19.4 %; both methods shockable 15.9 %; clipped (flat tops) 0.2 %; fine VF (not shocked) 0.3 %; methods disagree 12.9 %; neither method shockable 20.7 %; possible artifact 30.6 %.

The Philips device printed "Start pause" (its CPR pause) after the shock in 70 recordings, median 3.060 s after the shock; CPR movement is therefore likely in much of the post-shock signal, which the analyser's compression rule is meant to notice.

**Shock transient.** Share of the first 5 s after the shock clipped by the report: median 23.7 %, 75 % 25.0 %, 95 % 27.0 %; recordings with any: 251.
Largest |value| in the first second after the mark (after conversion to 250/s, so including the resampler's ringing at clipped edges): median 2472 µV, 95 % 2510 µV, max 2510 µV (pre-shock amplitude median 606 µV). One-second peak-to-peak 1–2 s after: median 1509 µV. The first window after the shock (covering 1–5 s after it) was judged: both methods shockable 3, clipped (flat tops) 19, methods disagree 10, neither method shockable 36, possible artifact 183. Clipping, size and saturation rules in all post-shock windows: clipped 22, size implausible 0, saturated 0.

**The Philips device's own post-shock decisions** (its labels inside the report) against this analyser's window verdict at the same moment:

| Philips said | shockable | not shockable | not sure | no window yet |
|---|---|---|---|---|
| Philips: shock advised | 34 | 5 | 25 | 0 |
| Philips: no shock advised | 1 | 103 | 46 | 0 |

(The Philips analysis uses its own, longer, look at the signal; its announcement time is when it finished, so the comparison is approximate.)

## How it was run

- Trace recovered from the PDF vector drawing at the rate it is drawn, scaled by the printed "2 div/mV", converted to 250/s with the crate's anti-aliasing resampler (`condition::Resampler`), rounded to whole µV.
- Pre-shock segment: the 9 s ending 0.5 s before the "Shock 1 delivered" mark. Device rule as in `eval::run_segment_with` (D-27; 3 s countdown).
- Confidence limits: exact (Clopper–Pearson) one-sided 90 % lower limits, treating recordings as independent. Duplicated reports are counted once.
- Analyser: `Settings::DEFAULT` as committed. Not tuned on these data.
