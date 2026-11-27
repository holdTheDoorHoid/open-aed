# Simulation findings

27 November 2026. Found by running the three firmware parts together in the simulator (`firmware/crates/aed-sim`). Each has a test in `firmware/crates/aed-sim/tests/findings.rs` that is parked (marked "ignored") and fails today for the reason given. A finding is closed when its test passes and is un-parked.

**All of this is simulation with provisional settings. It says nothing about a real device.**

| No. | Finding | Seriousness | Status |
|---|---|---|---|
| F-1 | **Compressions that run on into an analysis can prevent a shock altogether.** If the responder keeps compressing for more than about 1.5–2 seconds after "Analysing", the first look at the signal contains movement. The analysis answers "not sure", the rescue sequence takes that as final, says "No shock advised" and starts two minutes of CPR. The same can happen every cycle. | **High** | **Fixed 27 Nov 2026.** "Not sure" no longer ends an analysis; the device repeats "do not touch the patient" and keeps analysing up to the 12 s limit. In simulation a shock still follows when compressions stop up to 6 s late. |
| F-2 | **The two voltage sensors are not allowed enough disagreement.** The 50 volt tolerance works only if each sensor is accurate to about 1%. Ordinary sensor error makes the supervisor declare a fault and disable the device for the rest of the rescue. | High | **Fixed 27 Nov 2026**, with one new problem: see F-9. Each sensor is now allowed 2% + 15 V of error. |
| F-3 | **A frozen main controller leaves the charger running.** The supervisor correctly empties the capacitor and no shock fires, but the charger keeps feeding the discharge load, holding the capacitor near 245 volts, and the device goes silent. Needs a hardware rule: the charger is cut off whenever the discharge path is open. | High | **Fixed in the model 27 Nov 2026.** The charger cannot run while the discharge path is open, and the supervisor has its own alarm. Both are now requirements on the future hardware. |
| F-4 | **A rhythm change in the last 4 seconds before the shock is not seen.** The analysis looks at a 4-second stretch of signal, so a heart that recovers one second before delivery is still shocked. | Medium | **Open.** Re-measured with the 3 s countdown: a recovery to normal rhythm in the last 2 seconds, or a change to flatline in the last 3.25 seconds, is missed. That blind time is now a quarter to a third of the whole hands-off pause. |
| F-5 | The supervisor's record shows a misleading "permit refused" after a good shock when messages are slow. | Low | **Fixed 27 Nov 2026.** |
| F-6 | After pads come off during charging, the device says "Start CPR" and "Attach pads" at the same instant. | Low | **Fixed 27 Nov 2026.** The device now says "Check pads" then "Attach pads". |
| F-7 | A weak charger makes the device silent for 12 seconds at switch-on before it reports the fault. | Medium | **Fixed 27 Nov 2026.** A weak charger is reported about 1 s after switch-on. |
| F-8 | Pads that are on but read just over 175 ohms get only "Attach pads", forever. See Q-16. | Medium | **Fixed 27 Nov 2026** by wider pad sensing and a "Press pads firmly" prompt (D-32). |
| F-9 | **Child mode: in-tolerance sensors can allow a shock about 21% over target.** If the charger regulates high while both sensors read low, all checks pass at up to 1,227 volts true, which into a 25–35 ohm child delivers about 60 J instead of 50 J. The placeholder tolerance is 15%. Ways out: more accurate sensors (about 1% + 7 V), or a change to the child waveform at low resistance. The adult worst case is +13.6%, close to the limit. | High | **Fixed 27 Nov 2026** by requiring sensors accurate to 1% + 7 V (D-28). Worst case is now +14.2% against a 15% placeholder limit, which is a thin margin. |
| F-10 | **On real recordings the rhythm analysis misses too much coarse ventricular fibrillation.** Untuned it caught 61.9% against a 90% target. After tuning on half the patients it caught **71.1% on the other, held-back half** (58.3% untuned on those same patients). Still well short. It rarely advises a shock wrongly, though slightly more often than before tuning. Full report: `firmware/crates/aed-eval/REPORT.md`. | **High** | **Open.** Improved 27 Nov 2026, target not reached. Further gains need more and better data, not more tuning of this data. |
| F-11 | **About a third of correct shock decisions on real fibrillation would be cancelled during the 3-second countdown.** The rescue sequence abandons a shock on any single "not shockable" or "not sure" answer while armed, and real fibrillation flickers between answers. On the held-back half only 46.8% of fibrillation analyses would end in a delivered shock. Relaxing the rule (ignoring one doubtful second) was tried and rejected because it made finding F-4 worse: a shock 2.5 s after the heart had recovered. | **High** | Open (owner and clinician decision, Q-21) |
| F-12 | **Very fast chest compressions over a flatline are read as a shockable rhythm.** Made-up compressions at 150 a minute with no heart rhythm underneath were called shockable 20 times out of 20 (never at 140 a minute or slower). They look like rapid ventricular tachycardia. The heart signal alone cannot tell them apart; a separate signal that senses compressions, such as the resistance between the pads, is needed. | **High** | Open (needs pad-resistance sensing) |

A side effect of the F-1 fix: when the signal never becomes clean (very weak fibrillation, interference, a responder who never stops compressing), each analysis now lasts the full 12 seconds before the device says "Unable to analyse. No shock. Start CPR."

## Timing in simulation

| Step | First version | Now (charging during analysis, 3 s countdown) |
|---|---|---|
| Analysis | 7.0 s | 7.0 s |
| Waiting for charge after "shock advised" | 6.0 s adult, 2.0 s child | 0 s |
| Stand-clear countdown | 5.0 s | 3.0 s |
| **Hands off the chest before the shock** | **about 17 s** | **about 9 s** |
| Pads on to first shock | 18.2 s adult, 14.2 s child | 10.2 s |

## What charging during analysis costs

- **Energy and heat.** Every analysis that ends without a shock burns 150–190 J in the internal load. A rescue with a normal rhythm burned about 307 J in under three minutes, against 8 J before. The internal load and the battery must be sized for this.
- **High voltage is present more often:** for 7–12 seconds in every cycle, not only when a shock is advised.
- **The analysis now always runs with the charger switching.** Electrical noise from the charger getting into the heart signal is not modelled at all. This must be measured on real hardware before the analysis can be trusted.
- **Charger and sensor faults are now found at the first analysis**, so they end the rescue in the fault state even for patients who would never have needed a shock. The device still guides CPR but stops analysing.
- A charger weaker than about 27 W in this model would not finish within the 7-second analysis.

## New risks noted

- The adult worst-case true voltage is 2,008 V, 8 V over the 2,000 V design ceiling.
- "Press pads firmly" is immediately followed by "Analysing. Do not touch." Whether responders handle that pairing needs testing with people.
- A stuck-cancel fault lasts until the device is switched off.
- In the random campaign, shocks within 4 seconds of the rhythm ending rose from 6 to 16 in 5,000 runs. This is finding F-4 and the rise is not fully explained.

## Other observations

- An output stage that silently fails to fire gives "Shock not delivered" and a retry every two minutes; it is never declared a fault.
- On noisy borderline signals the analysis can waver between "shockable" and "not sure", and a single "not sure" during the countdown cancels the shock.
- A broken discharge load is not noticed until the device next needs to empty a fully charged capacitor.
- The rhythm analysis has only been run on made-up signals. It has not been tested on fast non-shockable rhythms, which are common in children.

## First test on real recordings (27 November 2026)

The rhythm analysis was run, with no adjustment, on three public databases of real heart recordings (PhysioNet: 35 + 22 + 48 recordings). Figures are for a 12-second analysis on the first recording channel.

| Measure | Result | Target | Verdict |
|---|---|---|---|
| Coarse ventricular fibrillation caught | 61.9% (399 segments, 30 recordings) | 90% or more | **Fails** |
| Rapid ventricular tachycardia caught | no usable data | 75% or more | Not testable |
| Normal rhythm correctly left alone | 100.0% (6,093 segments, 60 recordings) | 99% or more | Meets |
| Other non-shockable rhythms left alone | 99.9% (2,458 segments, 39 recordings) | 95% or more | Meets |
| Flatline left alone | 96.4% (56 segments, only 4 recordings) | 95% or more | Too little data; one doubtful label |
| Analyses ending "unable to analyse" | 7.4% | none set | |

What this means:

- **The analysis errs heavily on the side of not shocking.** That is the safer direction for a fully automatic device, but a device that misses four in ten shockable rhythms does not do its job.
- **Main causes of the misses,** in order: one threshold (the "band-pass count") set too high for real fibrillation; the rule that turns any disagreement between the two methods into "not sure"; a lower frequency limit; and the rule meant to spot chest compressions, which fires on 17–19% of these recordings although none contains CPR.
- **Shorter analyses do worse:** 47% at 8 seconds, 40% at 7. The 7-second analysis seen in simulation is therefore optimistic for real signals.
- **Three rules tuned on made-up signals do not work on real ones:** the clipping check never fires, the flatline check catches almost no real flatline, and the weak-fibrillation rule almost never applies.

## After tuning (27 November 2026)

The patients were split in half by a fixed rule before any tuning. Settings were adjusted using only the development half, then the held-back half was scored once with the settings frozen. Figures are for a 12-second analysis.

| Measure | Untuned, all recordings | Tuned, held-back half | Same half, untuned | Target |
|---|---|---|---|---|
| Coarse fibrillation caught, channel 0 | 61.9% | **71.1%** (218 segments, 16 patients) | 58.3% | 90% |
| Coarse fibrillation caught, channel 1 | 78.5% | 80.9% (89 segments, 2 patients) | 75.3% | 90% |
| Same, with an 8-second analysis | 47.1% | 58.0% | 44.3% | |
| Normal rhythm left alone, channel 0 | 100.0% | 99.8% (5 wrong shock decisions in 3,007) | 99.9% | 99% |
| Normal rhythm left alone, channel 1 | 99.8% | 99.4% (18 in 2,875) | 99.7% | 99% |
| Other non-shockable left alone | 99.9% | 99.5% | 99.8% | 95% |
| Flatline left alone | 96.4% | no data in this half | no data | 95% |

- **What was changed:** one threshold lowered slightly; confirmation changed from "three looks in a row" to "three of the last five"; the size limit raised; a working clipping check and a better flatline check added. A more flexible scoring model was tried and did worse. The compression rule was left alone.
- **All 31 wrong shock decisions on the held-back half come from three recordings,** the same three that caused them before tuning: one where the labelled "normal" stretches sit between bursts of flutter and look abnormal, one fast atrial fibrillation, one fast supraventricular rhythm. Most would have been cancelled during the countdown.
- **Flatline got worse on the development half** (91.1% under cross-validation against 96.4% untuned), all from one stretch of one recording whose label is doubtful. The held-back half has no flatline, so this is unresolved.
- **Limits on how clean this test is:** the untuned results for every recording, including which patients did badly, had been seen before the split was made.
- **The honest conclusion:** tuning these settings on this data is close to exhausted. Reaching 90% needs recordings from real cardiac arrests through defibrillator pads, from many patients, and a compression-sensing signal.

What it does not mean:

- These are hospital and monitor recordings from a few dozen patients, not out-of-hospital cardiac arrests through defibrillator pads. They contain no CPR movement and no children. Even a pass here would say little about a real device.
- This was a clean test only because nothing was tuned on these recordings. Any tuning on them from here on must hold whole patients back for testing.
- The American Heart Association target figures are from the research notes and have not been checked against the original paper.
