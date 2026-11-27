# Compression sensing on real recordings: evaluation report

27 November 2026. Tool: `crates/aed-cpr-eval`. Detector: `crates/aed-cpr` (default settings).

**NOT A MEDICAL DEVICE.** This measures one piece of software on recordings made by another manufacturer's defibrillators. It says nothing about this project's hardware, which does not exist.

## In one paragraph

On 45 held-out cardiac arrests (25.6 hours of pad-impedance recordings) the detector said "hands off" during **0.51 %** of the time that compressions were being given (159 s out of 31,457 s), found 96.4 % of individual compressions, and measured the rate to within 1 per minute (median). It noticed compressions starting within 0.33 s (median; 95 % within 0.78 s) and declared hands off 1.05 s (median) after they stopped. The misses are not spread evenly: in **6 of 45 held-out cases** there was at least one stretch of 2 s or more of missed compressions, the longest **13.7 s**, in a patient whose compressions changed the impedance by only about 0.15 to 0.25 Ω, below the detector's 0.3 Ω threshold. That is the main weakness found.

## The data

| | |
|---|---|
| Name | Insufflated Volumes and Electrical Impedance for Cardiac Arrest Resuscitations |
| Holder | Brigade de Sapeurs-Pompiers de Paris (Paris Fire Brigade) |
| Where | Mendeley Data, https://data.mendeley.com/datasets/34gffrm87w/1 |
| Licence | **CC BY 4.0** (Creative Commons Attribution 4.0 International), as stated by the repository's own record (checked through its public API on 27 Nov 2026) |
| File | `data.zip`, 96,632,878 bytes, SHA-256 `b0f6138595d3e94ac1f65379b8d5180a3a67c670e9ff51b982be5ca66b28aada` (matches the repository's published hash) |
| Kept at | `data/mendeley/paris-impedance/` at the repository root; git-ignored, never committed |

**Required citation** (CC BY 4.0 requires credit, a link to the licence, and a note of any changes; this project changed nothing in the files):

> Tassart B, Menant E. Insufflated Volumes and Electrical Impedance for Cardiac Arrest Resuscitations. Mendeley Data, version 1, 2024. doi:10.17632/34gffrm87w.1. Licence: CC BY 4.0, https://creativecommons.org/licenses/by/4.0/

Companion paper: Menant E, Tassart B, Meillier C, Lemoine F, Petermann A, Jost D, Jouven X. Registration of transthoracic impedance signal and ventilation volume data in out-of-hospital cardiac arrest. *Data in Brief* 2025;59:111345. doi:10.1016/j.dib.2025.111345 (read through PubMed Central).

### What it actually contains

- **101 cases** (adult out-of-hospital cardiac arrests, Paris, May to November 2023), one folder each. The paper reports 129 enrolled and 28 excluded for missing data.
- **Impedance** (`Impedance_signal.csv`): time and "TTI (Ohms)", **125 samples per second**, from Schiller DGT7 defibrillators (the paper calls them Defigard Touch 7). The values are the **change** in impedance around the baseline, not the whole impedance: the signal swings around zero. Steps of 12.2 mΩ. Clipped at about ±24 to 25 Ω (values beyond ±23.8 Ω are treated as clipped here). Stretches of exactly zero mark periods without a signal, for example before the pads go on.
- **Analysis markers** (`Rhythm_analysis.csv`): the times at which the defibrillator started each rhythm analysis.
- **Breaths** (`Ventilation_volumes.csv`): one row per breath from a separate ventilation monitor (EOLife), with volumes, on the monitor's own clock. The paper lined the two clocks up afterwards (89 of 101 cases), but **the offsets are not published**, so these times cannot be used to locate breaths in the impedance signal.
- **Not in it:** no heart signal (ECG), **no compression labels or compression times**, no absolute pad impedance, no shock markers, no patient details, no outcome. The paper found compression periods with an algorithm, but those times are not in the files.

So the dataset gives real impedance during real CPR, which is what was needed, but **no ground truth for compressions**. That had to be built (below).

### Untrusted input

The files were downloaded into a new empty folder, the archive listing was checked (only CSV files and one spreadsheet) before extraction, and nothing in them was executed. The reader treats every file as hostile: lines are capped at 4 KB, files at 200 MB, every number must parse and be finite and in range, and the case lists accept only names of the form `intervention_NNN`. Tests cover malformed input.

## Method

### Fixed split, before tuning

Before any detector setting was chosen, the cases were split by a hash of the case name into **56 development** and **45 held-out** cases (`splits/`, with the lists' own hashes; the tool refuses to run if a list has changed). Settings were chosen on the development half only. The held-out half was scored **once**, with every setting frozen. What had been seen before the split is listed in `splits/README.md` (one summary line per case and one plot of a case that turned out to be held out).

### The reference ("ground truth")

Because the dataset has no compression labels, the reference is itself an algorithm (`src/reference.rs`), built to be different from the detector and to have advantages the device can never have:

- it sees the whole recording, so it filters without delay (forward and backward) and judges each peak by what comes after it;
- it uses **half the detector's size threshold** (0.15 Ω), so that compressions too small for the detector still count, and show up as misses;
- it calls something compressions only when at least four peaks come in a row at 50 to 240 per minute, each gap within 0.6 to 1.6 times the recent gaps, ignoring "shoulders" smaller than 30 % of the peaks around them;
- 2 s around clipping, missing data and signal-free stretches is not scored.

It was checked by eye on plots of a sample of development windows. Where CPR is clean (regular compressions, clear pauses) it matches what the eye sees. Its known faults: in messy, double-peaked compression signals it breaks one long run into several (so some genuine compressions are counted as "hands off" in the reference, which makes the detector's "false compressing" figure look worse than it is); and, before the 50-per-minute floor was added, a regular slow wave of large breaths was once strung into a "run". **A reference built by an algorithm can share blind spots with the detector**, because both see only the impedance; real labels (from video, an accelerometer or a CPR-feedback device) would be better.

### Scoring

The detector is run sample by sample exactly as on the device, with the recorded change added to an 80 Ω baseline (any baseline in range gives the same result) and clipped or missing samples fed as "saturated". Measures:

- **Missed compressions** (the safety measure): share of reference compression time, from 1 s after the start of each run, during which the detector said "hands off". "Unknown" does not count as missed, because the device treats it like compressions.
- **Hands off taken for compressions**: share of reference hands-off time, from 2.5 s after the end of each run, during which the detector said "compressing" or "unknown" (the cost: an analysis would be held).
- **Per compression**: a reference compression counts as found if the detector reported a compression from 0.15 s before to 0.5 s after its peak.
- **Rate**: detector rate against the reference rate in 5 s windows inside runs.
- **Latency**: from the start of a run (half an interval before its first peak) to the first answer other than "hands off", and to "compressing"; from the end of a run to "hands off". Only boundaries with 3 s of clean hands-off signal on the other side.

## Results

### Held-out half (45 cases, 25.6 h; scored once, settings frozen)

| Measure | Result |
|---|---|
| Reference compression time | 31,457 s |
| **Detector said "hands off" during compressions (missed)** | **159 s = 0.51 %** |
| ... said "compressing" / "unknown" | 97.32 % / 2.17 % |
| **Longest single missed stretch** | **13.7 s** (intervention_019) |
| Cases with any missed stretch of 2 s or more | 6 of 45 (012: 4.0 s, 019: 13.7 s, 020: 2.6 s, 024: 2.8 s, 044: 3.9 s, 068: 4.4 s) |
| Reference hands-off time | 53,922 s |
| ... detector said "compressing" / "unknown" | 1.23 % / 1.84 % |
| Compressions found (sensitivity) | 61,144 of 63,398 = 96.4 % |
| Detector compressions matching a reference compression (PPV) | 91.4 % |
| Rate error, median / 95th percentile / 99th | 0.94 / 4.35 / 10.3 per minute |
| **Onset: run start → not "hands off"** | median 0.33 s, 90th 0.63 s, 95th 0.78 s, 99th 1.43 s, max 5.35 s (780 onsets) |
| Onset: run start → "compressing" | median 0.90 s, 95th 1.56 s, max 3.84 s |
| **Offset: run end → "hands off"** | median 1.05 s, 90th 1.81 s, 95th 2.72 s, 99th 4.30 s, max 11.8 s (1,243 offsets) |
| Reference compression sizes | median 2.58 Ω; 1.0 % below 0.3 Ω, 3.4 % below 0.5 Ω |
| At the defibrillator's own analysis-start markers | hands off 325, compressing 13, unknown 12 |

### Development half (56 cases, 29.6 h; the settings were chosen here)

Missed 0.34 % (140 of 41,193 s), longest 4.4 s; 6 of 56 cases with a stretch of 2 s or more; hands off taken for compressing 1.64 % / unknown 3.72 %; 96.5 % of compressions found, PPV 89.9 %; rate error median 0.94 per minute; onset median 0.33 s (95th 0.77 s); offset median 1.07 s (95th 2.58 s). Reference compression sizes: median 2.62 Ω, 0.75 % below 0.3 Ω. Pauses between runs of compressions (breaths, analyses, shocks): median 3.05 s, 90 % under 11.9 s, 5.1 % of 15 s or more (used to set the "Continue CPR" prompt, aed-core `Config::no_compression_prompt_ms`).

Settings compared on this half: high-pass corner 0.3, 0.5 and 1.0 Hz, second and fourth order; threshold 0.2, 0.25, 0.3 and 0.4 Ω; quiet time 0.8, 1.0 and 1.2 s; longest rise 0.5, 0.6 and 0.7 s. Missed compression time ranged from about 0.19 % to 0.36 % across reasonable choices; the chosen settings (0.5 Hz second order, 0.3 Ω, 1.0 s, 0.6 s, plus a 0.35 s limit on the steep part of a rise) traded a little of that for fewer breaths read as movement and a shorter time to "hands off". The 0.3 Ω threshold was fixed from published figures, not from this data (on this half, 0.2 or 0.25 Ω changed the missed time little and called more hands-off time "unknown" or "compressing").

The held-out result is worse than development (0.51 % against 0.34 % missed; longest miss 13.7 s against 4.4 s), as expected after tuning.

### The outliers (inspected by plotting after the held-out score was final; nothing was changed)

- **intervention_019** (13.5 % of its compression time missed; the 13.7 s stretch). Between pairs of large breaths about 15 s apart (the 30-compressions-then-2-breaths pattern) there are small regular ripples at about 110 per minute of only **0.15 to 0.25 Ω**. These are almost certainly real compressions: the timing fits 30:2. They are below the detector's 0.3 Ω threshold, so the detector said "hands off" while the chest was being compressed. This is the dangerous direction.
- **intervention_044** (21 % of only 170 s): small irregular bumps of 0.1 to 0.3 Ω; whether these are compressions cannot be told from the impedance alone.
- **intervention_012, 068, 020, 024**: shorter misses (2.6 to 4.4 s) at the edges of runs and in small-amplitude stretches.
- **intervention_005** (3.1 % missed, 54 % "unknown"): a short case whose compressions the detector mostly called "unknown" (still safe: unknown holds an analysis).
- On the development half, the longest misses (intervention_049, 009, 070) were of the same kind: small regular ripples that might be shallow compressions or the heart's own beat, and in 070 a slow regular wave of large breaths.

### What this means for the device

- Compression sensing works on most real CPR from this service: 99.5 % of compression time was correctly "not hands off".
- **It is not enough on its own.** Some patients' compressions are close to, or below, the size of the heart's own impedance change. For them the detector cannot tell, and says "hands off". Lowering the 0.3 Ω threshold would catch more of them but would bring the heart's own beat (up to about 0.1 Ω, Urteaga et al., Entropy 2021;23:847) closer to being read as compressions. The held-out data is spent, so this cannot be re-measured cleanly here. **This is why the heart-signal rule for compression artifact in `aed-rhythm` is kept as a second line of defence.**
- The cost: the detector needs about 1 s of stillness before it says "hands off" (median 1.05 s here). In the simulator this adds about 1.8 s to the hands-off pause before each shock.

## Limits of this evidence

- **One device model, one service, one city.** Schiller DGT7 defibrillators, Paris Fire Brigade, 2023. Pad type, pad position, the impedance front end's filtering and its excitation are not stated; this project's pads and electronics will differ. The recorded impedance is already baseline-removed and filtered by the Schiller device in ways that are not documented.
- **No real labels.** The reference is an algorithm checked by eye on a sample, not annotated compressions. It undercounts compressions in messy signals and cannot settle whether small regular ripples are compressions or heartbeats.
- **No heart signal**, so the effect on the rhythm analysis (finding F-12) is shown only in simulation, not on this data.
- **No absolute impedance**, so the detector's checks of the accepted range, of a frozen reading and of saturation were exercised only with the recordings' clipping and signal-free stretches.
- **No shock markers**, so the behaviour just after a shock was not measured.
- Adults only. No children (the device's child mode uses front-and-back pads, which will change the impedance signal).
- Mechanical CPR devices, if any were used, are not identified.
- The development half was used for several rounds of choices, so its figures are optimistic.

## How to reproduce

From `firmware/`, with the dataset unpacked so that `data/mendeley/paris-impedance/data/` (at the repository root) holds the case folders, or with `AED_CPR_DATA` pointing at that folder:

```
cargo run --release -p aed-cpr-eval -- describe      # what the development half holds
cargo run --release -p aed-cpr-eval -- dev           # score the development half
cargo run --release -p aed-cpr-eval -- heldout       # score the held-out half (frozen settings only)
```

`--marks DIR` writes, per case, the reference runs and the detector's answers as plain text for plotting. The tests (`cargo test -p aed-cpr-eval`) skip the real-data check when the data is absent.
