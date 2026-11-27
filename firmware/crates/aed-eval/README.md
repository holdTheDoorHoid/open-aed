# aed-eval

Runs the rhythm analyser (`aed-rhythm`) on **recorded** heart rhythms from three public PhysioNet databases and reports how often it gives the right answer, measured against the American Heart Association's 1997 performance goals.

DEVELOPMENT COMPUTER ONLY. NOT A MEDICAL DEVICE. See [SAFETY.md](../../../SAFETY.md). Results on these databases say nothing about how a device would do on real cardiac arrests; read the limits in [REPORT.md](REPORT.md) before quoting any number.

The latest results are in [REPORT.md](REPORT.md). How the analyser was tuned on half of the recordings, and tested once on the other half, is in [TUNING_LOG.md](TUNING_LOG.md); the split itself is in [splits/](splits/README.md).

## Running it

From `firmware/`:

```
cargo run --release -p aed-eval -- --data /path/to/data/physionet
```

Options:

| Option | Meaning |
|---|---|
| `--data <dir>` | Folder holding `cudb/`, `vfdb/` and `mitdb/` (required) |
| `--db cudb` (or `vfdb`, `mitdb`) | Only this database; may be repeated |
| `--records <file>` | Only the records listed in this file (`splits/development.txt` or `splits/held_out.txt`); the report records the list's SHA-256 |
| `--untuned` | Run the analyser with its settings from before any tuning on real data (commit 9cf11b6), for comparison |
| `--kind <text>` | A sentence printed under the report's title, saying what kind of run it is |
| `--make-split <dir>` | Instead of evaluating, (re)write the patient-wise split files into `<dir>` (see [splits/README.md](splits/README.md)) |
| `--csv <dir>` | Also write `segments.csv` (every scored analysis) and `episodes.csv` (every VF / flutter / VT episode in the continuous run) |
| `--out <file>` | Also save the report to a file |
| `--threads <n>` | Records evaluated in parallel (default: all cores) |

The report is printed as Markdown. A full run takes about half a minute on 8 cores.

The tuning experiments are a second program, `aed-tune`, which runs only on the development list:

```
cargo run --release -p aed-eval --bin aed-tune -- --data /path/to/data/physionet --records crates/aed-eval/splits/development.txt frozen
```

Its experiments and what each printed are listed in [TUNING_LOG.md](TUNING_LOG.md).

## Getting the data

The recordings are **not** in the repository (the `data/` folder is ignored by git) and must never be committed. Download each database from its PhysioNet page into `data/physionet/<name>/` at the repository root:

| Database | Page | Records | Licence (as stated on the page) |
|---|---|---|---|
| Creighton University Ventricular Tachyarrhythmia Database (`cudb`) | https://physionet.org/content/cudb/1.0.0/ | 35, 1 channel, 250 Hz, about 8.5 min each | Open Data Commons Attribution License v1.0 |
| MIT-BIH Malignant Ventricular Ectopy Database (`vfdb`) | https://physionet.org/content/vfdb/1.0.0/ | 22, 2 channels, 250 Hz, 35 min each | Open Data Commons Attribution License v1.0 |
| MIT-BIH Arrhythmia Database (`mitdb`) | https://physionet.org/content/mitdb/1.0.0/ | 48, 2 channels, 360 Hz, 30 min each | Open Data Commons Attribution License v1.0 |

Each page has a "Download the ZIP file" link and a `wget` command. Check the downloaded files against each page's `SHA256SUMS.txt`.

Citations the pages ask for:

- **cudb:** Nolle FM, Badura FK, Catlett JM, Bowser RW, Sketch MH. CREI-GARD, a new concept in computerized arrhythmia monitoring systems. *Computers in Cardiology* 13:515-518 (1986).
- **vfdb:** Greenwald SD. *Development and analysis of a ventricular fibrillation detector.* M.S. thesis, MIT Dept. of Electrical Engineering and Computer Science, 1986.
- **mitdb:** Moody GB, Mark RG. The impact of the MIT-BIH Arrhythmia Database. *IEEE Eng in Med and Biol* 20(3):45-50 (May-June 2001). PMID 11446209. And the dataset: Moody G, Mark R. MIT-BIH Arrhythmia Database (version 1.0.0). PhysioNet (2005). https://doi.org/10.13026/C2F305
- **All three (PhysioNet's standard citation):** Pollard T, Moody BE, Lehman L, Gow B, Fernandes C, Xie C, Johnson A, Mark RG, Heldt T. PhysioNet as a global platform for biomedical research. *Nature Health* (2026). https://doi.org/10.1038/s44360-026-00096-z

(Licences and citations were read from the PhysioNet pages on 26 Nov 2026.)

## Tests

```
cargo test -p aed-eval
```

The unit tests use small hand-built files and synthetic signals. `tests/real_data.rs` checks known facts of the real files (sample counts, first samples against the header, checksums of every record). It looks for the data in `$AED_EVAL_DATA` or `data/physionet` at the repository root and **skips, printing "SKIPPED", when the data are absent**, so it passes on a machine without them. `tests/findings.rs` pins down two analyser behaviours found during the evaluation (see REPORT.md). `tests/split.rs` checks that the committed split files have not changed (by their SHA-256), are disjoint and cover every record, and, when the data are present, that rerunning the split rule reproduces them.

## What the tool does, in plain terms

1. **Reads the files.** PhysioNet stores each recording as a small text "header" (what is in the file and how to scale it), a packed binary sample file, and an annotation file (the experts' labels). The readers for all three are written here, with no outside libraries. Every file is treated as possibly hostile: sizes are capped, every field is checked, and a bad file produces an error message, never a crash. Each signal's checksum and first sample are compared with what the header promises.
2. **Converts the signal** to what the analyser expects: whole microvolts at 250 samples per second. Each signal is scaled with its own gain and baseline; anything that would not fit is clipped and counted. The 360-per-second MIT-BIH Arrhythmia recordings are converted to 250 per second with a proper anti-aliasing resampler (tested with sine sweeps: flat to within 0.5 % up to 90 Hz, more than 60 dB rejection of what would otherwise fold back). Nothing is rescaled to "correct" amplitudes, even where the calibration is known to be doubtful (cudb).
3. **Works out the truth** from the annotations (next section).
4. **Runs the analyser as the device would.** At the start of every analysis the device resets the analyser, then acts on its first final answer: "shock", "no shock", or, if neither comes within the time limit, "unable to analyse" (no shock). That is simulated on consecutive, non-overlapping stretches of signal for analysis lengths of 7 s (the shortest that can end in a shock), 8 s and 12 s (the device's limit, decision D-27). After a "shock", the analyser is fed the next 3 s too (the stand-clear countdown, where the same rhythm continues): the rescue sequence abandons a shock on any answer other than "shock" during the countdown, so these shocks are counted as **cancelled in the countdown**. Separately, one analyser is run over each whole recording without resets, to see how it behaves when the rhythm changes.
5. **Counts and reports**, by database, by channel, pooled, and per recording, with exact confidence limits, the AHA goals, and a failure analysis of which rules caused the mistakes.

Nothing in this crate changes the device's analyser. The first run (27 November 2026, `reports/baseline_untuned_9cf11b6.md`) was untuned and therefore clean. Since then some settings have been re-chosen on the **development** half of the recordings only, and the **held-out** half was run once with the settings frozen (TUNING_LOG.md). Results on the development half flatter the analyser; only the held-out figures are a fair test, and even they are not fully blind (see REPORT.md).

## How annotations become ground truth

The databases label rhythms in different ways. These are the rules used, and why. Anything doubtful goes to its own reported bucket or is left out, never into a class where it would make the result look better.

**Rhythm labels** (`+` annotations whose text gives the rhythm; the rhythm lasts until the next label):

| Annotation | Where | Scored as | Reason |
|---|---|---|---|
| `(VF`, `(VFIB` | vfdb, cudb | **Coarse VF** if its amplitude is at least 200 µV, else **fine VF** (report only) | AHA classes; amplitude measured independently of the analyser (below) |
| `[` … `]` (start / end of VF or flutter) | cudb | Coarse / fine VF as above | cudb's documented VF marks. cudb does not separate VF from flutter, and its onsets are "approximate" per its documentation |
| `(VFL` (ventricular flutter) | vfdb, mitdb | **Ventricular flutter**, its own row | Usually treated as shockable, but not folded into coarse VF: flutter is a large regular wave that is easier to detect and would flatter the VF figure |
| `(VT` | all | **Rapid VT** if the beat annotations in the segment give a rate of at least 150/min; **slow VT** below that (report only); **VT, rate unknown** where there are no beats (all vfdb VT) | The AHA leaves the rapid-VT rate to the manufacturer; 150/min is this evaluation's choice and matches the analyser's own 2.4 Hz limit. VT is also reported with no rate criterion |
| `(N`, `(NSR` | all | **Normal sinus rhythm** | |
| `(ASYS` | vfdb | **Asystole** | |
| `(AFIB`, `(AF` (cudb), `(AFL`, `(SVTA`, `(SBR`, `(BI`, `(BII`, `(NOD`, `(P`, `(PM`, `(B`, `(AB`, `(T`, `(IVR`, `(VER`, `(PREX` | all | **Other non-shockable**, also reported label by label | The AHA "other" class: atrial fibrillation and flutter, supraventricular tachycardia, slow sinus rhythm, heart blocks, junctional, paced, bigeminy/trigeminy (premature beats), idioventricular and escape rhythms, pre-excitation |
| `(HGEA` (high-grade ventricular ectopic activity) | vfdb | Its own bucket, report only | Not clearly shockable or non-shockable |
| `(NOISE` | vfdb | Left out (for every channel) | vfdb documents that the previous rhythm continues under the noise, but the signal is not clean |
| No label yet (before the first one) | vfdb, mitdb | Left out | Unknown rhythm |
| Outside VF with no `+` label, and after every `]` | cudb | Its own bucket, "cudb non-VF, rhythm not given", report only | cudb labels every beat "normal" even when it is not, and is a tachyarrhythmia database, so this time can contain VT. Counting it as normal rhythm would be a guess |
| Any other label text | | Left out and listed in the report | |

**Other annotations:**

| Annotation | Use |
|---|---|
| `~` signal-quality change (cudb, mitdb) | The `subtyp` field says which channels are noisy: −1 = all unreadable, bit 0 = channel 0, bit 1 = channel 1. That channel is left out until the next `~`, plus 1 s on each side. (The text `(N` that a few cudb `~` marks carry is ignored.) |
| `\|` isolated QRS-like artefact, `"` note `TS` (tape slippage) | 1 s on each side left out |
| Beat labels (`N`, `V`, `/`, `!` …) | Only their times are used, to measure heart rate (for the VT rate). cudb's beat labels are all `N`; only their positions are used |
| `[` and `]` in mitdb | Ignored: they coincide with mitdb's `(VFL` labels, which are used instead |

**Margins.** Around every rhythm change, 3 s on each side is left out, because annotated change times are approximate and the analyser looks at 4 s at a time. Only stretches with one class throughout are scored.

**Amplitude** (for coarse versus fine VF): baseline drift is removed with a 2-second triangular moving average, the signal is cut into 1-second pieces, and the median of each piece's peak-to-peak value is taken. This is independent of the analyser's own measurement (which is taken after its 1–30 Hz filter and reads lower).

**Scoring.** For sensitivity (shockable classes) only a "shock" counts; "no shock" and "unable to analyse" are misses. For specificity (non-shockable classes) "no shock" and "unable to analyse" both count as correct. "Unable to analyse" is also reported separately everywhere, because a device that answers "not sure" to everything would score perfect specificity and be useless.

## Third-party code

None. The crate depends only on `aed-types` and `aed-rhythm` and the Rust standard library. It runs only on a development computer.
