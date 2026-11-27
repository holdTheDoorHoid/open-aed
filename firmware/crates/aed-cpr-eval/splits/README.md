# Case split for the compression-sensing evaluation

Fixed on 27 November 2026, **before** any tuning of the compression detector and before the recordings were explored in detail.

- Data: "Insufflated Volumes and Electrical Impedance for Cardiac Arrest Resuscitations", Paris Fire Brigade, Mendeley Data, DOI 10.17632/34gffrm87w.1 (CC BY 4.0). 101 cases (`intervention_001` to `intervention_101`). The recordings are not in this repository.
- Rule: a case goes to the development half if the first byte of SHA-256(`open-aed-cpr-split-v1:` + case name) is even, otherwise to the held-out half. The rule uses nothing but the case name.
- Result: 56 development cases (`development.txt`), 45 held-out cases (`heldout.txt`). `SHA256SUMS` holds the hashes of both lists so any later change is visible.
- Use: thresholds may be chosen by looking at development cases only. The held-out cases are scored once, with every setting frozen, and that score is the one reported.

What had been seen before the split (so a reader can judge how clean it is):

- One summary line per case for all 101 cases: length, sample spacing, and percentiles of the impedance values. Nothing about compressions specifically.
- One plot of the first 200 s of `intervention_001`, which the rule then put in the held-out half. It showed that the impedance channel is baseline-removed (it swings around zero), that compressions are clearly visible at roughly 2 to 3 ohms peak to peak in that case, and where an analysis pause sits relative to the analysis marker. No threshold was chosen from it.
