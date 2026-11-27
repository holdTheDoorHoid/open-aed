# Patient-wise split

`development.txt` and `held_out.txt` divide the 105 recordings (104 patients) of cudb, vfdb and mitdb into two halves, made on 27 November 2026 before any tuning:

- **development** (53 records): every tuning step, threshold choice and logic change used only these, with leave-one-patient-out cross-validation inside them;
- **held-out** (52 records): run once, with the settings frozen, at the end.

## How the split was made

The rule is in `src/split.rs` and is mechanical; rerun it with

```
cargo run --release -p aed-eval -- --data <path to data/physionet> --make-split crates/aed-eval/splits
```

1. Whole patients are split, never parts of a recording. MIT-BIH Arrhythmia records 201 and 202 are the same man (the database's own `mitdbdir/intro.htm`), so they are one unit. No other same-patient records are documented in the files supplied with the three databases; every other record is its own unit. (That two databases contain the same person cannot be ruled out from those files.)
2. Units are grouped by database and by annotation content: annotated ventricular fibrillation, ventricular flutter only, or neither.
3. Within each group the units are sorted by name and shuffled with a fixed-seed generator (xorshift32, seed 20261127, the date). The seed was chosen before the split was made, and the split was made once.
4. The first half (rounded up) of each shuffled group is development, the rest held-out.

Only the annotations' content was used. The analyser's results were not.

| Database | Group | development | held-out |
|---|---|---|---|
| cudb | VF | 17 | 16 |
| cudb | neither | 1 | 1 |
| vfdb | VF | 2 (424, 430) | 2 (422, 426) |
| vfdb | flutter only | 2 (419, 609) | 2 (418, 429) |
| vfdb | neither | 7 | 7 |
| mitdb | flutter only | 1 (207) | 0 |
| mitdb | neither | 23 records | 24 records (23 patients: 201 and 202 together) |
| **all** | | **53** | **52** |

A limit on how clean the held-out half is: the untuned analyser had already been run on all 105 records before the split (`REPORT.md` at commit 9cf11b6), and its aggregate results and per-record outliers had been read. The split did not use them, but the person tuning had seen, for example, which records produced false shocks and which VF records were missed. The held-out result is therefore a fairer test than a resubstitution figure, but not a fully blind one.

By chance, five of the six vfdb records that contain annotated asystole (424, 427, 430, 602, 607) fell into development and one (423) into held-out, so the held-out half says almost nothing about asystole.

## File hashes (SHA-256)

```
597e65c49bc0114f3680c1e3f49cc8d553407606e411f6bce6a3e1714d477903  development.txt
3e5c70316b41bdaacc76471a7a775b3b41c99a95dbbba182fb47e13a44d26601  held_out.txt
```

`tests/split.rs` checks these hashes, that the halves are disjoint and cover every record, and (when the data are present) that rerunning the rule gives the same files.
