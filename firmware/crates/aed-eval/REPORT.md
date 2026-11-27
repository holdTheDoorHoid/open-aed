# Rhythm analyser on recorded heart rhythms: tuning and held-out test

- **Date:** 27 November 2026 (the first, untuned run was also on 27 November 2026).
- **Analyser:** `aed-rhythm` in the working tree on top of commit `9cf11b6` (not committed), settings `Settings::DEFAULT` as frozen in `TUNING_LOG.md`.
- **Data:** PhysioNet cudb, vfdb, mitdb (105 public hospital-monitor and Holter recordings, 104 patients), split by patient into a development half (53 records) and a held-out half (52 records); see `splits/README.md`.
- **Full generated reports:** `reports/baseline_untuned_9cf11b6.md` (the first run, as committed), `reports/baseline_untuned_all.md` (the same, regenerated with the new countdown and timing tables), `reports/development_untuned.md`, `reports/development.md`, `reports/held_out_untuned.md`, `reports/held_out.md`. Every candidate tried, with cross-validated scores: `TUNING_LOG.md`.

> **What these databases cannot show.** They are hospital and Holter recordings from a few dozen adults in the 1970s–80s, not cardiac arrests recorded through defibrillator pads. They cannot establish clinical performance, and nothing here says the analyser is safe or ready for use on a person. **Untested:** rapid VT with a known rate (no usable data), fine VF (one segment in all three databases), asystole beyond 4 patients (and none at all in the held-out half), chest-compression artefact on real signals (none of these recordings has CPR), children, and out-of-hospital pad recordings. The confidence limits below treat segments as independent; they come from a few dozen patients, so the real uncertainty is larger.

## In short

- **The 90 % coarse-VF goal was not reached.** On the held-out half the tuned analyser caught **71.1 %** of coarse VF on channel 0 (lower limit 66.8 %, 218 segments, 16 patients), against 58.3 % for the untuned analyser on the same patients. On channel 1 (2 patients only) 80.9 %.
- **The cost in specificity is small but real.** On the same held-out patients, false shocks on normal rhythm rose from 2 to 5 (channel 0) and from 10 to 18 (channel 1), and on other rhythms from 3 to 6 (channel 0). Specificity is still 99.4–99.8 % for normal rhythm (lower limits 99.1–99.7 %) and 99.5–99.8 % for other rhythms. All 31 held-out false shocks come from three records that already caused the untuned analyser's false shocks: vfdb 418 (stretches labelled "normal" between flutter bursts), cudb cu09 (atrial fibrillation at 154–163/min) and vfdb 426 (supraventricular tachycardia).
- **A problem the first report did not measure: shocks lost in the stand-clear countdown.** After "shock advised" the device counts down for 3 s and abandons the shock if the analyser says anything but "shockable". On real VF a single doubtful second is common, so about a third of correct shock decisions would be abandoned there. Counting only shocks that would actually be delivered, held-out coarse VF goes from 37.2 % (untuned) to 46.8 % (tuned) on channel 0. A rule that let one doubtful second pass was tried and rejected, because in the simulator it let a shock through 2.5 s after VF had turned into a normal rhythm (finding F-4). This is a decision for the project owner, not something the analyser can settle alone.
- **Decisions come sooner.** On the held-out half a correct shock decision was reached within 8 s in 64 % of coarse-VF analyses (untuned 53 %); normal rhythm is still answered at 5 s.
- **What changed:** the band-pass count threshold (0.18 → 0.16), the confirmation rule (3 in a row → 3 of the last 5), the size limit (8 → 12 mV), a new clipping check that works on real recorders, and a new flatline test that recognises real asystole. Everything else is unchanged, including the "compression" rule, which cannot be relaxed safely from the ECG alone.

## 1. Untuned baseline (as committed at 9cf11b6, all 105 records)

The first run, before any tuning. Figures as in `reports/baseline_untuned_9cf11b6.md`; `aed-eval --untuned` reproduces them exactly. 12 s analysis unless stated.

| Channel | Coarse VF (LCL 90 %; n; records) | at 8 s | at 7 s | Normal sinus rhythm | Other non-shockable | Asystole | Unable to analyse |
|---|---|---|---|---|---|---|---|
| 0 | 61.9 % (58.6 %; 399; 30) | 47.1 % (44.4 %; 614) | 40.5 % (38.0 %; 702) | 100.0 % (99.9 %; 6,093; 60), 2 false shocks | 99.9 % (99.7 %; 2,458; 39), 3 | 96.4 % (90.8 %; 56; 4), 2 | 7.4 % |
| 1 | 78.5 % (74.1 %; 177; 4) | 62.5 % (58.5 %; 272) | 54.2 % (50.4 %; 312) | 99.8 % (99.7 %; 5,862; 57), 11 | 100.0 % (99.8 %; 2,406; 37), 1 | 91.1 % (84.1 %; 56; 4), 5 | 5.8 % |

Shocks the untuned analyser would have lost in the countdown (all records): channel 0, 88 of 247 coarse-VF shocks; channel 1, 40 of 139.

The aggregate results and the per-record outliers of this run (all 105 records) were known before the split was made. The split did not use them, but the person tuning had seen, for example, which records gave false shocks and which VF records were missed. **The held-out half is therefore a fairer test than the development figures, but not a fully blind one.**

## 2. Development half (53 records, 52 patients): where the tuning happened

### 2a. Cross-validated (the honest estimate for the development half)

Leave-one-patient-out: each patient scored with the settings the procedure chose without that patient. From `aed-tune` (`TUNING_LOG.md`, E4c). Analysis lengths are the first 7, 8, 10 and 12 s of each 12 s segment.

| Class | Channel | n (patients) | untuned, 12 s | CV 7 s | CV 8 s | CV 10 s | **CV 12 s (LCL 90 %)** | shocks cancelled in countdown (CV) |
|---|---|---|---|---|---|---|---|---|
| Coarse VF | 0 | 181 (14) | 66.3 % | 56.9 % | 63.0 % | 69.1 % | **71.3 % (66.5 %)** | 31 of 129 (untuned 42 of 120) |
| Coarse VF | 1 | 88 (2) | 81.8 % | 72.7 % | 79.5 % | 83.0 % | **87.5 % (81.7 %)** | 17 of 77 (untuned 18 of 72) |
| Normal sinus rhythm | 0 | 3,086 (30) | 100.0 % | | | | 100.0 % (99.9 %), 0 false shocks | |
| Normal sinus rhythm | 1 | 2,987 (30) | 100.0 % | | | | 100.0 % (99.9 %), 1 | |
| Other non-shockable | 0 | 1,137 (22) | 100.0 % | | | | 100.0 % (99.8 %), 0 | |
| Other non-shockable | 1 | 1,088 (21) | 100.0 % | | | | 100.0 % (99.8 %), 0 | |
| Asystole | 0 | 56 (4) | 96.4 % | | | | 91.1 % (84.1 %), 5 | |
| Asystole | 1 | 56 (4) | 91.1 % | | | | 89.3 % (82.0 %), 6 | |

Asystole is worse under cross-validation than untuned: all 11 false shocks are in one stretch of vfdb 607 labelled asystole but showing a regular 4–4.5 Hz wave (the first report already doubted this label). When that patient is the one left out, the procedure picks a more lenient setting and it shocks that stretch more often. **On the development data the AHA asystole goal (95 %) is not met, and the frozen analyser only matches the untuned one there (93.8 %, resubstitution).**

### 2b. Resubstitution (chosen and scored on the same records: **optimistic**)

From `reports/development.md` (`aed-eval`, its own segment tiling for each analysis length).

| Channel | Coarse VF (LCL; n; records) | at 8 s | at 7 s | Normal sinus rhythm | Other non-shockable | Asystole | Unable to analyse |
|---|---|---|---|---|---|---|---|
| 0 | 71.3 % (66.5 %; 181; 14) | 60.4 % (56.4 %; 278) | 55.5 % (51.7 %; 319) | 100.0 % (99.9 %; 3,086), 0 | 100.0 % (99.8 %; 1,137), 0 | 96.4 % (90.8 %; 56; 4), 2 | 8.5 % |
| 1 | 87.5 % (81.7 %; 88; 2) | 70.9 % (65.3 %; 134) | 69.5 % (64.2 %; 154) | 100.0 % (99.9 %; 2,987), 1 | 100.0 % (99.8 %; 1,088), 0 | 91.1 % (84.1 %; 56; 4), 5 | 7.6 % |

Untuned on the same records (`reports/development_untuned.md`): channel 0 coarse VF 66.3 % at 12 s, 50.4 % at 8 s, 45.1 % at 7 s.

## 3. Held-out half (52 records, 51 patients): the single frozen run

Run once, at 2026-11-27T13:38:41Z, with the settings frozen; nothing in the analyser's behaviour was changed afterwards. `reports/held_out.md`. Compared with the untuned analyser on the same patients (`reports/held_out_untuned.md`).

| Measure (12 s unless stated) | Channel | Untuned | **Tuned, frozen** (LCL 90 %; n; records) | AHA goal (LCL) |
|---|---|---|---|---|
| Coarse VF caught | 0 | 58.3 % | **71.1 %** (66.8 %; 218; 16) | 90 % (87 %): **not met** |
| Coarse VF caught | 1 | 75.3 % | **80.9 %** (74.5 %; 89; 2) | **not met** |
| Coarse VF caught, 8 s analysis | 0 | 44.3 % | 58.0 % (54.4 %; 336; 16) | |
| Coarse VF caught, 7 s analysis | 0 | 36.6 % | 47.0 % (43.6 %; 383; 16) | |
| Coarse VF shock delivered (not cancelled in countdown) | 0 | 37.2 % | 46.8 % | |
| Coarse VF shock delivered | 1 | 50.6 % | 60.7 % | |
| Normal sinus rhythm left alone | 0 | 99.9 % (2 false shocks) | 99.8 % (99.7 %; 3,007; 30), **5 false shocks** | 99 % (97 %): met |
| Normal sinus rhythm left alone | 1 | 99.7 % (10) | 99.4 % (99.1 %; 2,875; 27), **18 false shocks** | met |
| Other non-shockable left alone | 0 | 99.8 % (3) | 99.5 % (99.2 %; 1,321; 17), **6 false shocks** | 95 % (88 %): met |
| Other non-shockable left alone | 1 | 99.9 % (1) | 99.8 % (99.6 %; 1,318; 16), **2 false shocks** | met |
| Normal sinus rhythm, 8 s / 7 s analysis | 0 | 100.0 % / 100.0 % | 99.8 % / 100.0 % | |
| Asystole | – | no scored segments | no scored segments | not testable |
| Unable to analyse (all segments) | 0 / 1 | 5.9 % / 3.3 % | 5.4 % / 3.4 % | |

By database (channel 0): cudb coarse VF 64.8 % (14 patients), vfdb 80.0 % (2 patients). vfdb normal rhythm on channel 1 alone is 97.3 % (lower limit 96.4 %), all from record 418; cudb "other" rhythms are 3 of 5 segments shocked (cu09), too few to judge.

### Every held-out false shock (31)

| Record | Channel | Rhythm label | False shocks |
|---|---|---|---|
| vfdb 418 | 0 | NSR (between flutter bursts) | 5 |
| vfdb 418 | 1 | NSR (between flutter bursts) | 18 |
| cudb cu09 | 0 | AFIB, 154–163/min | 3 |
| vfdb 426 | 0 | SVTA | 3 |
| vfdb 426 | 1 | SVTA | 2 |

Of the normal-rhythm false shocks, 4 of 5 (channel 0) and 13 of 18 (channel 1) would have been abandoned in the countdown; 1 and 5 would have been delivered. The first report already questioned whether vfdb 418's "NSR" stretches are labelled correctly; that needs a cardiologist. Each false shock, with its start time and measurements, is listed in `reports/held_out.md` ("Every false shock").

### What the held-out misses are

80 of 307 held-out coarse-VF analyses (both channels) did not end in a shock: 30 "no shock", 50 "unable to analyse". They concentrate in a few patients: cudb cu15 (0 of 8), cu11 (3 of 10), cu30 (3 of 8), cu07 (15 of 26). In the windows of missed analyses the band-pass count was still the commonest failure (48 % of windows; the only failure in 21 %), then a dominant frequency at or below 2.4 Hz (45 %), VF-filter leakage (41 %), the low-band rule (28 %) and the new clipping check (13 %, cudb records whose recorder clipped large VF; 8 analyses ended on it).

## 4. Time to decision and the countdown, before and after

Share of analyses with a final answer by each time; held-out half, channel 0 (development half, channel 0, in brackets).

| Class | Version | by 7 s | by 8 s | by 10 s | by 12 s | median | 90th percentile |
|---|---|---|---|---|---|---|---|
| Coarse VF | untuned | 45.4 % (57.5 %) | 52.8 % (64.1 %) | 66.1 % (72.9 %) | 72.9 % (80.7 %) | 7 s | 10 s |
| Coarse VF | tuned | 53.7 % (64.6 %) | 64.2 % (72.4 %) | 75.2 % (78.5 %) | 79.8 % (82.3 %) | 7 s | 9 s |
| Normal sinus rhythm | untuned | 93.0 % | 93.7 % | 94.5 % | 95.3 % | 5 s | 5 s |
| Normal sinus rhythm | tuned | 92.6 % | 93.4 % | 94.1 % | 95.1 % | 5 s | 5 s |
| Other non-shockable | untuned | 97.5 % | 98.2 % | 98.6 % | 98.8 % | 5 s | 5 s |
| Other non-shockable | tuned | 99.0 % | 99.5 % | 99.6 % | 99.8 % | 5 s | 5 s |
| Asystole (development only) | untuned | 30.4 % | 35.7 % | 42.9 % | 46.4 % | 7 s | 10 s |
| Asystole (development only) | tuned | 50.0 % | 53.6 % | 53.6 % | 57.1 % | 5 s | 8 s |

(For VF "a final answer" includes a wrong "no shock"; the coarse-VF rows count any answer.) Shocks cancelled in the countdown, held-out, channel 0: 46 of 127 untuned, 53 of 155 tuned (about a third either way).

The earliest possible "shock" is still 7 s after the analysis starts (1 s settling, a 4 s window, 3 shockable windows). In the simulator nothing changed: the timing table of every scenario is identical (hands-off before the first shock 9.3 s in the standard adult case), and a 2,000-run random campaign gave 442 of 581 VF/VT runs shocked against 440 before, with no safety-rule violation and the same 6 known F-4 cases. After VF or flutter ends, "shock" persisted at most 3.8 s in the continuous runs, as before.

## 5. What was changed, and the evidence

| Change | Before → after | Evidence (development half) |
|---|---|---|
| Band-pass count threshold | 0.18 → **0.16** | Chosen in every cross-validation fold (E1, E4c). Lower values caught more VF but gave more false shocks on the development half's annotated asystole than the untuned analyser, which the tuning did not allow. |
| Confirmation | 3 shockable windows in a row → **3 of the last 5** (current window shockable) | Chosen in 50 of 53 folds (E4c). Every non-shockable window is still announced at once, so rhythm changes cancel a shock as fast as before. A one-second "hold" during the countdown was better on the data but rejected for safety (simulator). "No shock" after 2 windows instead of 1 was not chosen. |
| Size limit ("implausible") | 8,000 → **12,000 µV** | Chosen in 52 of 53 folds, no extra false shock. Part of the gain comes from cudb, whose amplitude calibration is only nominal; must be re-set for the real front end. |
| Clipping check | front-end limit only (±32,000 µV, never fired) → **plus a flat-top check** (≥ 10 identical raw samples at the window's extreme, more than 1 % of the window) | Catches 89 of 102 development windows that were really clipped by the recorder, 14 false alarms in 78,359 windows. Not tuned for sensitivity. Costs some cudb VF (clipped by its recorder). The front-end range stays a setting. |
| Flatline (asystole) | peak-to-peak < 100 µV → **also** rms < 25 µV with a noise-like shape (VF-filter leakage > 0.6) | Catches 37 % of annotated-asystole windows instead of 1 %; no VF or flutter window, 4 VT windows (0.1 %). Thin evidence: nearly all from one patient (vfdb 427). A small regular wave (fine VF) is not called flatline. |
| Dominant-frequency lower limit | 2.4 Hz, **unchanged** | Lowering it gave no cross-validated gain (E2, E4c) and doubled the made-up compression windows called shockable. |
| VF-filter leakage, TCSC, concentration, noise, fine-VF limits | **unchanged** | Cross-validation kept the old values (E2, E4, E5). Fine VF (200 µV) cannot be assessed: one segment in all the data. |
| Linear score instead of thresholds | **not adopted** | At the same false rate it caught 56–66 % of VF windows against 80 % for the thresholds (L-1). |

Everything else in the analyser is as before. Settings are in `crates/aed-rhythm/src/settings.rs`, each with its provenance, and gathered in `Settings::DEFAULT`. Compute cost: three extra passes over each window (root-mean-square, one-second peak-to-peak values, flat-top scan), small next to the spectrum; memory up by 2 KB (one window of raw samples) to about 10.5 KB. Not measured on hardware.

## 6. The compression rule and the three checks that did not work on real signals

- **Compression ("low-band") rule: left exactly as it was.** On every feature the analyser has, real VF windows whose power sits below 2.4 Hz overlap the made-up compression artefact (TUNING_LOG.md, T-7). With the dominant-frequency limit kept at 2.4 Hz, the rule changes which real VF windows are shockable by only 7 in 2,152; what it does is turn "no shock" into "not sure, keep analysing" when the low band dominates, which is its safety job (finding F-1). It still makes 7.7 % of normal and other-rhythm windows "not sure". **Telling real slow VF from compressions needs a compression signal, most simply the pad impedance.** Also found: made-up compressions at 150/min over a flat line are called shockable by both the old and the new analyser (20 of 20); at that rate they look exactly like rapid VT at 150/min. Only a compression reference can separate them.
- **Clipping:** now detected from the signal (flat tops), as above.
- **Flatline:** now also recognises recorder-noise asystole, as above; the evidence is four development patients, effectively one, and none in the held-out half.
- **Size limits and cudb calibration:** the implausible-size limit was raised to 12 mV; the fine-VF limit (200 µV) was not touched, because there is no fine VF in these data to judge it by. cudb's VF reads about twice vfdb's, so both limits mean less on cudb than they appear to.

## 7. What would be needed to reach 90 %

The remaining misses are not one threshold away. Lowering thresholds further on the development half (E3b) bought about 6 points of sensitivity at the price of 9 normal-rhythm and 22 other-rhythm false shocks, and the held-out half already shows a small loss of specificity at the chosen settings. In order of expected value:

1. **Real out-of-hospital pad recordings**, many patients, split by patient: the only way to tune and test for the real device. These three databases hold 30 VF patients in all.
2. **A compression reference (pad impedance)**, so that slow, low-frequency VF need not be treated as possible CPR artefact, and so that the countdown could tolerate a doubtful second when nothing has changed at the pads.
3. **More features in a small, explainable model** (the literature's ~30-feature sets, for example Figuera et al. 2016), trained only once there are enough patients to support it; with a few dozen patients here, a learned model would overfit.
4. A second lead would help (channel 1 did better than channel 0 on the same patients) but a two-pad device has only one.
5. A project decision on the countdown rule: today any doubt in the 3 s countdown abandons the shock, which loses about a third of correct VF shocks on these recordings and protects against shocking a rhythm that has just changed (F-4).

## How it was scored

As in the first report (`reports/baseline_untuned_9cf11b6.md`, "How it was scored"), with one addition: after a "shock" the analyser is fed the next 3 s of the same rhythm, and a shock is "cancelled in the countdown" if the analyser announced anything other than "shockable" in that time. Confidence limits are one-sided 90 % lower limits (Clopper–Pearson). The AHA goals are from the project's research notes and are UNVERIFIED against the original paper.

## Reproducing this report

```
cargo run --release -p aed-eval -- --data <data/physionet> --records crates/aed-eval/splits/held_out.txt
cargo run --release -p aed-eval -- --data <data/physionet> --records crates/aed-eval/splits/development.txt
cargo run --release -p aed-eval -- --data <data/physionet> --untuned
cargo run --release -p aed-eval --bin aed-tune -- --data <data/physionet> --records crates/aed-eval/splits/development.txt final-no-hold frozen
```

The data are not in the repository; see `README.md` for where to get them, their licences and the citations they ask for.

---

## 8. External test on real out-of-hospital cardiac arrests: the Cardially dataset (27 November 2026)

*Hand-written summary, added 27 November 2026. Sections 1–7 above are unchanged. Every figure below is from the generated report `reports/cardially_untuned.md` (program `aed-cardially`, commit `ec5564a`, analyser settings exactly as frozen in `TUNING_LOG.md`), except where a PhysioNet figure is quoted from `reports/held_out.md`.*

### What was tested, and why it is a clean test

The frozen analyser was run, **with no change and no tuning**, on the only openly downloadable set of real out-of-hospital cardiac-arrest ECGs the project has found (research note `ohca_data_sources.md`): "Cardially" (University of Brescia, Mendeley Data, DOI 10.17632/wpr5nzyn2z.1). Nothing in the analyser was chosen with these recordings in view, so this is an independent test of how it detects real out-of-hospital VF. **It remains one only as long as nothing is ever tuned on it.**

What the dataset is, in short (details and evidence in the crate README):

- **251 different recordings** of adults in Brescia, 2006–2009 (260 reports, of which 9 repeat another report's ECG exactly). How many patients is not known.
- Each is a printed report from the defibrillator's review software covering 60 s around the **first shock**: about 10 s of VF before it (during the device's own analysis and charging, so no CPR) and about 50 s after it.
- The reports are vector drawings, not scans, so the ECG could be read back exactly: 200 points per second, with a printed scale of 10 mm per mV. That scale is the software's statement, not a verified calibration; the software also **clips the trace at ±2 mV**, and its filtering is unknown.
- Labels: three cardiologists' verdict on whether the shock worked (`ROEA` 51, `noROEA` 191, `indeterminable` 9 distinct recordings). They are not rhythm annotations.
- **Licence:** the data page says CC BY 4.0; the companion article is CC BY-NC-ND 4.0. Unresolved, written confirmation from the authors pending. This evaluation does not redistribute or tune on the data, which either licence allows for non-commercial research, and nothing from the dataset is in the repository.

**What it cannot show:** anything about wrong shocks (every recording is VF); the last 3 s of a 12 s analysis or a full countdown (only about 10 s before the shock); CPR artefact before the shock; children. And it is VF that the recording defibrillator itself had already called shockable, which probably makes it easier than VF in general.

### Main result: VF before the first shock

Treated as one device analysis: a fresh analyser fed the 9 s ending 0.5 s before the shock mark, the outcome being its first final answer.

| Measure | Result (n = 251 recordings) | Lower 90 % limit |
|---|---|---|
| **VF called "shock" within 9 s** | **61.4 %** (154) | 57.2 % |
| VF of at least 200 µV ("coarse", the AHA class) | 63.7 % (151 of 237) | 59.4 % |
| Answered "no shock" | 15.5 % (39) | |
| No answer within the 9 s ("unable to analyse") | 23.1 % (58) | |
| Shocks that worked (`ROEA`) / did not (`noROEA`) | 78.4 % (40 of 51) / 57.6 % (110 of 191) | 69.3 % / 52.7 % |
| All the signal before the shock (9.5–10.5 s) | 60.6 % (152) | 56.4 % |
| The dataset's own digitised version of the same 9 s | 59.4 % (148 of 249) | 55.2 % |
| AHA goal for coarse VF | 90 % | 87 % |

**The 90 % goal is far from met on real out-of-hospital VF.** The figure sits where the PhysioNet held-out figures did for similar analysis lengths (58.0 % at 8 s, 71.1 % at 12 s, channel 0, section 3), so the out-of-hospital recordings tell the same story as finding F-10 rather than a worse one. VF in shocks that later worked was caught more often; that VF was also larger (median 840 µV against 548 µV, from the per-recording CSV).

**Is 9 s enough?** No. The analyser throws away the first second, needs a full 4 s window, and then three shockable windows, so the earliest possible "shock" is 7 s after it starts. In 9 s it sees only five windows and can decide at 7, 8 or 9 s (it did so 108, 31 and 15 times). The device is allowed 12 s (D-27), so this is an analysis cut short by three windows; some of the 58 "no answer" recordings would probably have been decided in the remaining time, in either direction. An *artificial* check (the 9 s played twice to make 12 s, which puts a jump in the signal at the join and re-uses the same VF) gave 66.5 %. That is not a result; it only suggests that the missing 3 s are worth roughly 5 points, not the 30 needed.

**Window by window**, before the confirmation rule: 60.6 % of the 1,255 windows were shockable, 33.4 % "not sure" (24.1 % because the two method families disagreed, 8.8 % from the compression rule) and 6.0 % "not shockable". 91 recordings had all five windows shockable; 46 had none.

**Amplitude.** Changing the scale barely matters except when the signal is made much smaller: x 0.5 gives 57.8 %, x 2 gives 62.2 %, x 4 gives 61.8 %, each recording normalised to 1 mV peak-to-peak gives 62.2 %; only x 0.25 falls sharply (38.6 %), because the "fine VF" and flatline rules then start to act. So the uncertain calibration does not change the conclusion, unless the printed scale overstates the true size by a factor of about three or more. As printed, the median VF size (one-second peak-to-peak) is 606 µV, and 14 recordings are below 200 µV.

### Why real out-of-hospital VF is missed: same story as F-10, slower

Among the 494 windows that were not shockable, the rules that failed were:

| Rule | windows failing it | the only failure |
|---|---|---|
| strongest frequency at or below 2.4 Hz | 57.7 % | 8.1 % |
| band-pass count | 44.7 % | 22.9 % |
| VF-filter leakage | 43.5 % | 8.7 % |
| spectral concentration | 24.1 % | 1.2 % |
| compression ("low-band") rule | 22.3 % | 0.4 % |
| fine VF (< 200 µV) | 3.0 % | 1.4 % |

These are the same rules that rejected PhysioNet VF (section 3: band-pass count 48 %, frequency at or below 2.4 Hz 45 %, leakage 41 %, low band 28 %), in a slightly different order. **Out-of-hospital VF is slower**: its strongest frequency has a median of 3.5 Hz against 4.0 Hz on held-out PhysioNet VF, a quarter of windows are at or below 2.5 Hz, and only 77.2 % of windows are above the 2.4 Hz floor (83.7 % on PhysioNet). Its low-band share is about twice PhysioNet's (median 0.141 against 0.074). The band-pass count behaves just as on PhysioNet (median 0.204 against 0.202; 82.4 % of windows over the threshold). Size, noise, flatline and clipping rules hardly ever act before the shock.

The 2.4 Hz floor is there to keep chest compressions out (section 6 and finding F-12). These recordings show that the price is paid on real arrests, where the VF has had time to slow down. As section 6 already concluded, it cannot be relaxed safely without a compression signal such as pad impedance.

### The countdown (finding F-11)

The segments are too short to see a whole 3 s countdown after a 9 s analysis. Using all the signal before the shock, 55 shock decisions came early enough to see the whole countdown, and **17 of them (30.9 %) would have been abandoned** by a single doubtful second. With only part of the countdown visible, at least 33 of 150 would have been. The artificial looped version gives 60 of 167 (36 %). All three agree with F-11: about a third of correct shock decisions on real VF would be lost in the countdown under today's rule.

### After the first shock (description, not a score)

The labels say whether the shock worked, not what the rhythm was second by second. `noROEA` includes asystole and slow rhythms as well as continuing VF, and the device began a CPR pause about 3 s after the shock in 70 recordings, so much of this signal contains compressions.

- **The analyser behaves in the expected direction.** After a shock that worked, 1–12 % of windows were shockable; after one that did not, 9–28 %, rising over the minute as VF returned or persisted. Over the whole post-shock part the analyser confirmed "shockable" at least once in 57.1 % of `noROEA` recordings and 23.5 % (12 of 51) of `ROEA` ones.
- **The 12 `ROEA` recordings with a confirmed "shockable"** need a cardiologist's look. In 4 of them the recording defibrillator itself also advised or delivered another shock (VF returned). The other 8 are recordings in which the device started a CPR pause or said "no shock advised" after the shock: either VF came back briefly, or compressions were read as VF (finding F-12). This cannot be decided from these labels.
- **Against the recording defibrillator's own decisions** inside the window: when it said "no shock advised" (150 times), this analyser's window at the same moment said "not shockable" 103 times, "not sure" 46 times and "shockable" once; when it said "shock advised" (64 times), "shockable" 34, "not sure" 25, "not shockable" 5. Broad agreement, with the same leaning towards "not sure".
- **The shock transient.** For about 1.2 s after each shock the trace sits at the report's ±2 mV clipping limit (median 23.7 % of the first 5 s). The first window after the shock was mostly "not sure" from the compression rule (183 of 251), because the recovering baseline puts most of the power below 2.4 Hz; the flat-top clipping check fired in 19 first windows (22 post-shock windows in all), and the size and saturation rules never. So the transient ends in "not sure", not in a wrong answer. The analyser's flat-top check does not see the report's clipping elsewhere, because converting to 250 samples per second makes the clipped tops no longer exactly flat; a real device's own clipping would not be resampled.

### What could make these numbers untrustworthy

- **The signal is a printed report read back,** not the device's file. Unknown filtering; the 10 mm/mV scale is the software's statement; clipping at ±2 mV (negligible before the shock, large just after it). Whether 200 samples per second is the device's rate is unknown.
- **Selection:** VF that the recording defibrillator had already advised shocking, from one city and one device type in 2006–2009.
- **9 s analyses,** not the device's 12 s. The direction of the bias is towards fewer shocks.
- **Recordings, not patients:** 9 exact duplicates were found and removed; others from the same patient cannot be ruled out. The confidence limits treat recordings as independent.
- **The segment's end was chosen here** (0.5 s before the shock mark, to keep the shock moment out); the dataset's own 9 s end about 0.3 s before it. The result is the same with all the signal before the shock (60.6 %).
- **Data errors found in the dataset itself:** the nine duplicates; four of the authors' text files at about half the PDF's amplitude; one text file that does not match its PDF; two mismatched file names; the description says "scanned" but the PDFs are vector drawings; the article and the reports name the device differently. None affects the PDF-based results above except the duplicates, which were removed.

### Bugs found

None in the analyser. The data problems above were all in the dataset.

### What this suggests (recommendations, nothing implemented)

1. **Keep Cardially untouched by tuning.** It is the project's only independent out-of-hospital test. Any future setting chosen while looking at it must be recorded, and it then stops being a clean test.
2. **The 2.4 Hz frequency floor is the main loss on real arrests.** Freeing it needs the compression reference (pad impedance) already recommended in section 7; this is the strongest evidence for that so far.
3. **The two-family "disagree means not sure" rule** made a quarter of real VF windows doubtful. A combined, explainable score is worth revisiting, but only with many more patients (section 7, point 3).
4. **F-11 is confirmed on real out-of-hospital VF:** about a third of correct decisions would be lost in the countdown. The owner and clinician decision (Q-21) now has a second, independent piece of evidence.
5. **Ask the authors** for the device's own files with longer pre-shock signal (so that a full 12 s analysis and countdown can be tested), the device model, the true amplitude calibration and filtering, an explanation of the duplicates and of the patient count, and written confirmation of the data licence.
6. **Have a cardiologist look at the 12 `ROEA` recordings** where the analyser confirmed "shockable" after a successful shock (from the CSV, which is not committed).
