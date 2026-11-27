# Decisions

This file is the source of truth. If another document disagrees with it, this file wins and the other document is wrong.

- **D-xx** are decisions. Those marked *provisional* were made by Claude on 26 November 2026 from the research so that planning could proceed; the owner has not confirmed them.
- **Q-xx** are open questions for the project owner.

## Decisions

| No. | Decision | Status | Reason |
|---|---|---|---|
| D-01 | "Rive AED" in the original request is taken to mean the **Avive Connect**. The CellAED is studied as the other very small unit. | **Confirmed by owner, 26 Nov 2026** | No product named "Rive" was found; the owner confirmed the Avive Connect was meant. |
| D-02 | The shock is a **biphasic truncated exponential** waveform adjusted to the patient's resistance. | Provisional | Best-proven shape; no human trial shows another is better for cardiac arrest; founding patents have expired. |
| D-03 | Adult energy is **150 joules only.** No allowance for a 200 joule version. | **Confirmed by owner, 27 Nov 2026** (changed: headroom dropped) | The model showed a 100 µF capacitor cannot deliver 200 J within the 2,000 V ceiling. Owner chose the smallest device. Matches Philips and Avive. |
| D-04 | Child energy is **50 joules**, selected in software. | Provisional | Matches Avive and the pocket units; avoids a separate attenuator or pad. |
| D-05 | **One pad set** for all patients, about 55–70 cm² of gel per pad, front-and-back placement in child mode. | Provisional | Meets the adult minimum area while fitting an infant. Avive's FDA approval summary and help page describe the same-pads approach, though one spec-page reading says separate child pads; to be settled by reading the FDA summary in full. Several other units (Zoll AED 3, Schiller, Stryker CR2) also use one pad set with a child button. |
| D-06 | **Fully automatic:** the device analyses and, if a shock is advised, warns, counts down and delivers it without a button press. | **Confirmed by owner, 26 Nov 2026** (changed from semi-automatic) | Owner decision. Removes the delay and hesitation of a button press. It raises the bar on the software: with no human confirming, a wrong shock decision is delivered, so the stand-clear warning, countdown, a way to cancel, and the independent second check all become essential. |
| D-07 | **Rechargeable** lithium-ion battery. | **Confirmed by owner, 26 Nov 2026** | A belt-worn unit is charged like a radio; this is the biggest single size saving. |
| D-08 | Firmware is written in **Rust**. | Provisional | Owner's standing preference; memory safety suits safety-critical code. |
| D-09 | **Bluetooth is left out of the first prototype.** Event records and heat history come off by cable through the dock contacts; space is left for a module later. Cellular and location remain deferred. | **Confirmed by owner, 27 Nov 2026** (changed from Bluetooth-only) | The modules with ready-made software have been discontinued, so the project would have to write and secure the module's firmware. Internet-capable devices also take on FDA cybersecurity requirements. |
| D-10 | **No circuit above 50 volts** is built or powered until a reviewed lab safety policy exists. | Provisional | The stored energy can kill the builder. See SAFETY.md. |
| D-11 | Measures that predict shock success are **recorded but never used** to decide a shock. | Provisional | The one human trial showed no benefit. |
| D-12 | The project does **not distribute assembled units, boards or kits**. | Provisional | Doing so may make the project a "manufacturer" in law and invites use on people. Needs a lawyer before it could change. |
| D-13 | **The repository is public** as of 27 November 2026, after the licence was added and the owner had read the plan and safety rules. Buildable high-voltage design files still wait for the patent review. | **Confirmed by owner, 27 Nov 2026** | Publishing early is one of the things that separated open medical projects that succeeded from those that stalled; data holders must be able to inspect the work. |

| D-14 | First version includes **compression rate, pause timing and a metronome** measured through the pads; compression-depth sensing is deferred. | Provisional | The first three need no extra hardware. Depth needs an added sensor and the survival evidence for it is mixed. |
| D-15 | **Self-test: full energy once a week, a low-voltage check every day**, both into the internal test load, with the battery tested under load. | **Confirmed by owner, 27 Nov 2026** (frequency set) | Battery, pad and charging faults account for most recorded fatal AED failures; twice-daily full-energy tests would wear out the capacitor in about three years. |
| D-16 | **End goal: a partner-ready design.** The project builds and tests an open design good enough for a regulated manufacturer to take through approval. It does not seek approval itself. | **Confirmed by owner, 26 Nov 2026** | Owner decision. Matches the pattern that has worked for other open medical devices. |
| D-17 | **Licence: share-alike.** CERN-OHL-S v2 for hardware, GPL v3 (or later) for firmware and software, CC BY-SA 4.0 for documents and research. | **Confirmed by owner, 26 Nov 2026** | Owner decision. Anyone who improves the design must publish the improvement. |
| D-18 | **High-voltage work waits for an experienced engineer.** No hands-on high-voltage building until someone with professional high-voltage experience joins the project. | **Confirmed by owner, 26 Nov 2026** | Owner decision. Phase 1 is hazard-free and comes first anyway. |
| D-19 | **No manual mode.** The device is fully automatic only. | **Confirmed by owner, 26 Nov 2026** | Owner decision (see D-06). Keeps the screen small and simple. |
| D-20 | **Data partnerships resumed.** Outreach letters are drafted in `docs/outreach/`; the owner sends them. Preferred arrangement: the holder runs the project's evaluation tool and returns only scores. | **Confirmed by owner, 27 Nov 2026** (reverses the earlier "parked") | Public data has taken the rhythm analysis as far as it can (finding F-10). |
| D-21 | **Budget: small and self-funded, with grant applications** for the larger costs. | **Confirmed by owner, 26 Nov 2026** | Owner decision. |
| D-22 | **Firmware is hardware-agnostic.** Everything that decides anything runs and is tested on an ordinary computer; the processor is chosen later and connected through a thin layer. | Provisional | Keeps the processor choice open while the Rust compiler qualification question is unresolved, and makes the safety logic testable now. |
| D-23 | **Model design point: 100 µF charged to 1,950 V (adult) or 1,150 V (child)**, voltage fixed per mode, phase lengths chosen from the measured resistance. | Provisional, model only | Delivers within 7% of target across 25–175 ohms in the model. Depends on an assumed 5 ohm internal resistance. |
| D-24 | **Only third-party code on the device is `libm`** (maths functions). Additions are listed in `firmware/DEPENDENCIES.md` with a reason. | Provisional | The medical software standard requires every outside component to be justified and tracked. |
| D-25 | **The charger is cut off whenever the discharge path is open**, by hardware. | Provisional | Simulation finding F-3: otherwise a frozen main controller leaves the charger running. |
| D-26 | **The supervisor has its own alarm output**, which also sounds if the main controller never starts within one second of switch-on. | **Confirmed by owner, 27 Nov 2026** | Finding F-3: a device whose main controller has died must not be silent. |
| D-27 | **"Not sure" does not end an analysis**; the device keeps analysing to a 12 second limit. | Provisional | Finding F-1: otherwise compressions running into an analysis could prevent a shock every cycle. |
| D-28 | **Voltage sensors must be accurate to about 1% + 7 V.** The agreement allowance and the charged window are set from that. | **Confirmed by owner, 27 Nov 2026** (changed from 2% + 10 V) | Finding F-9: with looser sensors a child shock into a low-resistance patient could overshoot by about 21%. |
| D-29 | **Charge during analysis.** The capacitor starts charging when an analysis starts, so a shock can follow the decision without a charging wait. If no shock is advised the charge is burned off inside the device. | **Confirmed by owner, 27 Nov 2026** | Cuts roughly 6 s from the hands-off pause before a shock. Costs battery and heat on every no-shock cycle. |
| D-30 | **Stand-clear countdown is 3 seconds.** | **Confirmed by owner, 27 Nov 2026** | Users are trained responders; saves 2 s of hands-off time. To be checked against the purchased standard. |
| D-31 | **Cancel is ignored during the switch-on self-test.** A button held down throughout is reported as a stuck-button fault. | **Confirmed by owner, 27 Nov 2026** | A stray press while drawing the device must not disable it. |
| D-32 | **Pads are sensed up to about 250 ohms.** Above the 175 ohm shock range the device still analyses and says "press pads firmly", but shocks only once the resistance is in range; otherwise CPR continues. Widening the shock range itself is to be studied once the standard is bought. | **Confirmed by owner, 27 Nov 2026** | Finding F-8: pads that are on but read slightly high must not leave the responder with only "attach pads". |
| D-33 | **Emptying the capacitor runs in the background.** The responder gets the next instruction at once; the device confirms empty, will not recharge until it is, and faults if it cannot confirm. | Provisional | Waiting for the capacitor to empty delayed the responder by up to a second on every cycle for no safety gain. |
| D-34 | **Moving the adult/child switch during an analysis** recharges for the new mode without stopping the analysis. After a shock is advised, a switch change still cancels it. | **Confirmed by owner, 27 Nov 2026** | A switch moved as pads go on is most likely a correction; sending the patient to two minutes of CPR first would cost more. |
| D-35 | **Rhythm-analysis tuning on public data uses a fixed patient split**, recorded in `firmware/crates/aed-eval/splits/`. The held-back half was scored once on 27 Nov 2026 and is now spent as a clean test. | Provisional | So results are not flattered by tuning and testing on the same patients. |
| D-36 | **The device will sense compressions through the pads' resistance** and use that, not the heart signal alone, to tell compressions from a rhythm. | Provisional | Finding F-12, and the research finding that pad resistance gives compression rate with no extra sensor. |
| D-37 | **A shock is never delivered while compressions are sensed.** Sensed movement after "shock advised" holds the shock back; the device re-analyses once, then falls back to CPR. | Provisional (Q-33) | One touch should not cost a patient two minutes, but a loop with neither CPR nor shocks must be impossible. |
| D-38 | **If compression sensing fails, the device continues on the heart signal alone**, records it and flags it at the next self-test. It says nothing aloud. | **Confirmed by owner, 27 Nov 2026** | A single failure must not block every shock. The cost is that finding F-12 returns while sensing is down. |
| D-39 | **Both controllers are Cortex-M33-class processors from different makers.** | **Confirmed by owner, 27 Nov 2026** | The safety-qualified Rust compiler covers this family (vendor documents, 27 Nov 2026); a smaller supervisor chip would fall outside the qualification. |
| D-40 | **Device recordings and evaluation data are never committed**; evaluation tools skip cleanly without them. The Cardially dataset is never used for tuning. | Provisional | Licences, privacy, and keeping a clean external test. |
| D-41 | **The storage capacitor is made to order: about 100 µF, at least 2,200 V, about 70 °C storage, charged only up to 60 °C, and good for at least 2,500 full charges.** | **Confirmed by owner, 27 Nov 2026** (revised: 85 °C lowered; 60 °C working and 2,500 charges set) | Owner: as rugged as feasible without blowing the weight budget. An 85 °C storage rating would not have made the device safe in a hot car, because the battery and pad gel are limited to about 60 °C. **Correction:** an earlier note here said the 85 °C rating cost about 60 g. The reworked parts study (version 0.3) found that it cost about nothing; the weight comes from the capacitor's working temperature and voltage rating, not its storage rating. Lowering it does widen the choice of makers. |
| D-42 | **Catalogue parts wherever they exist; made-to-order parts where they do not**, each with a published specification so anyone can order the same part. | **Confirmed by owner, 27 Nov 2026** (relaxes "off-the-shelf only") | The capacitor, charging transformer, switch drive, pad connector and pads cannot be bought from a catalogue. |
| D-43 | **The 700 g target includes the pads, and is the goal for the finished design.** A first prototype of 780–900 g is accepted. | **Confirmed by owner, 27 Nov 2026** | What the responder carries is what counts. Parts study version 0.4 puts the lower-risk set at about 832 g with pads and the smallest plausible set at about 653 g (714 g with the expected capacitor). The 700 g goal no longer depends on the most optimistic capacitor estimate. |
| D-44 | **Two battery cells.** | **Confirmed by owner, 27 Nov 2026** | Full-speed charging at −20 °C; about 20 g heavier. |
| D-45 | **When Bluetooth is added it will be a separate pre-certified module** with its own processor. | **Confirmed by owner, 27 Nov 2026** (deferred by D-09) | Keeps radio code off the processor that decides to shock (D-24). |
| D-46 | **Charging is through sealed contacts and a dock**, not a USB socket. | **Confirmed by owner, 27 Nov 2026** | Sealing to IP65. |
| D-47 | **Once the chest is confirmed still, the rhythm analysis starts after a quarter-second guard** instead of a full settling second; its filters keep running through CPR. The guard stays. | **Confirmed by owner, 27 Nov 2026**; implemented | Recovers 0.8 s of the pause added by compression sensing (finding F-14). |
| D-48 | **Ruggedness goal: carried all day, and left in a vehicle for short periods**, with an insulated pouch kept in the vehicle. Not designed to live on a dashboard. The device measures and records its own temperature at all times, including when off. Limits: normal up to 40 °C; stays ready up to 60 °C for up to 8 hours at a time; "not ready" above 60 °C until it has cooled and passed a self-test; "service needed" above 70 °C. Battery charging only between 0 and 45 °C; capacitor charging only up to 60 °C. | **Confirmed by owner, 27 Nov 2026** (figures to be confirmed against final part datasheets) | Owner decision. Ruggedness is traded to protect the weight target, not the other way round. |
| D-49 | **Self-test schedule details:** tests run on the charging dock; only the full-energy test waits for cold, heat or a low battery; a test missed by more than a day shows "not ready"; a failed daily check is retried after an hour and a failed weekly test after a day; if the capacitor cannot be confirmed empty after a test the device alarms and runs no more automatic tests until serviced. | **Confirmed by owner, 27 Nov 2026** | Chosen conservatively by Claude while implementing D-15. |
| D-50 | **A device showing "not ready" still attempts a rescue.** Its switch-on self-test decides: if that passes it shocks with every usual check; if not it guides CPR. It does not announce "not ready" aloud during the rescue. | **Confirmed by owner, 27 Nov 2026** | A responder holding a "not ready" device must still get whatever help it can safely give. |
| D-51 | **A device switched on while too hot guides CPR and analyses, but will not charge until its temperature is back inside the limits.** Then it charges and shocks as normal. | **Confirmed by owner, 27 Nov 2026** | Charging a hot battery or capacitor risks damage and the wrong energy; the responder still gets CPR guidance at once. |
| D-52 | **The device adds up the time its pads spend above 50 °C and calls for new pads** after about 7 days in total. The figure comes from the pad maker. | **Confirmed by owner, 27 Nov 2026** | Pad makers allow only short spells at 60 °C; heat-damaged pads would otherwise go unnoticed. |
| D-53 | **The device counts full charges and calls for service as the capacitor approaches its 2,500-charge life.** A heavy-duty capacitor for very busy services is noted as a later option. | **Confirmed by owner, 27 Nov 2026** | Sizing for typical use saves about 71 g. |

## Owner questions

All eleven were answered on 26 November 2026.

| No. | Question | Answer | Recorded as |
|---|---|---|---|
| Q-01 | Was the Avive Connect meant? | Yes | D-01 |
| Q-02 | Public or private? | Private until the licence is in and the owner has read the plan, then public | D-13 |
| Q-03 | Which licence? | Share-alike | D-17 |
| Q-04 | End goal? | Partner-ready design | D-16 |
| Q-05 | Who does high-voltage work? | Recruit an experienced engineer first | D-18 |
| Q-06 | Rechargeable or long-standby? | Rechargeable | D-07 |
| Q-07 | 150 J fixed or step up to 200 J? | Fixed 150 J, room for 200 J later | D-03 |
| Q-08 | How connected? | Bluetooth only | D-09 |
| Q-09 | Data partners? | Parked for now | D-20 |
| Q-10 | Manual mode? | No. Fully automatic | D-06, D-19 |
| Q-11 | Budget? | Small self-funded, seek grants | D-21 |

Decisions D-02, D-04, D-05, D-08, D-10, D-11, D-12, D-14 and D-15 remain provisional: they were made by Claude from the research and the owner has not been asked about them directly.

## Questions from the first firmware and simulation work

All seven were answered on 27 November 2026.

| No. | Question | Answer | Recorded as |
|---|---|---|---|
| Q-12 | 200 J headroom does not fit; what to do? | 150 J only | D-03 |
| Q-13 | Charge while analysing? | Yes | D-29 |
| Q-14 | Countdown length? | 3 seconds | D-30 |
| Q-15 | Cancel during switch-on self-test? | Ignore it; detect a stuck button | D-31 |
| Q-16 | Pads on but resistance just over 175 ohms? | Sense wider and prompt now; study a wider shock range later | D-32 |
| Q-17 | Child shock overshoot? | Require more accurate sensors | D-28 |
| Q-18 | Supervisor alarm when the main controller never starts? | Keep it | D-26 |

## Open questions

**Q-19. Adult/child switch moved during analysis.** The firmware recharges for the new mode and keeps analysing (D-34). The alternative is to stop, go to CPR and analyse again in the new mode. Confirm or overrule.

**Q-20. The last-seconds blind spot (finding F-4).** A heart that recovers in the last 2 seconds before delivery is still shocked, because the analysis looks at a 4-second stretch of signal. Options: add a fast "has the rhythm just changed?" check during the countdown; shorten the analysis window (costs accuracy); or accept it, as commercial fully automatic units have a similar gap. Needs a clinician's view.

**Q-21. How strict should the countdown be? (finding F-11)** Today any single doubtful answer during the 3-second countdown cancels the shock. On real fibrillation that cancels about a third of correct shocks. Allowing one doubtful second would deliver more of them, but would also shock a heart that recovered up to 2.5 seconds earlier (finding F-4). This is a trade between missed shocks and unnecessary ones and needs a clinician's view. Not changed.

## Questions from the parts study, test instrument and compression sensing

Answered on 27 November 2026, except where marked.

| No. | Question | Answer | Recorded as |
|---|---|---|---|
| Q-23 | Parked-car heat? | Made-to-order capacitor; 85 °C first chosen, then lowered to about 70 °C | D-41, D-48 |
| Q-24 | Accept made-to-order parts? | Catalogue where possible | D-42 |
| Q-25 | Voltage margin? | 2,200 V parts, keep 100 µF | D-41 |
| Q-26 | Self-test frequency? | Weekly full, daily light | D-15 |
| Q-27 | Cold weather? | Two cells | D-44 |
| Q-28 | Bluetooth? | Separate certified module | D-45 |
| Q-29 | Supervisor processor? | Cortex-M33-class, different maker | D-39 |
| Q-30 | Does 700 g include pads? | Yes | D-43 |
| Q-31 | Charging connection? | Sealed contacts and a dock | D-46 |
| Q-32 | Extra pause from compression sensing? | Recover about a second | D-47 |
| Q-33 | Movement during the countdown? | **Open: needs a clinician, with Q-20 and Q-21.** D-37 stands meanwhile. | |
| Q-34 | Compression sensing failed? | Carry on, record only | D-38 |
| Q-35 | Reference test equipment? | **Deferred by owner**: decide when the lab safety policy exists | |

## Questions from the parts study revisions, settling and self-test work

Answered on 27 November 2026.

| No. | Question | Answer | Recorded as |
|---|---|---|---|
| Q-19 | Adult/child switch moved during analysis? | Recharge and keep analysing | D-34 |
| Q-36 | Weight target? | 700 g with pads for the finished design; heavier first prototype accepted | D-43 |
| Q-37, Q-43 | Hot cars and heat envelope? | Ready to 60 °C, "not ready" 60–70 °C, plus a vehicle pouch | D-48 |
| Q-38 | Expected usage? | 2,500 full charges; heavy-duty option later | D-41, D-53 |
| Q-40 | Bluetooth firmware? | Leave Bluetooth out of the first prototype | D-09 |
| Q-41 | Quarter-second guard? | Keep it | D-47 |
| Q-42 | "Not ready" device in a rescue? | Switch-on self-test decides; nothing said aloud | D-49, D-50 |
| Q-44 | Capacitor charging temperature? | Up to 60 °C | D-41 |
| Q-45 | Hot device in a rescue? | Guide CPR, shock once cooled | D-51 |
| Q-46 | Pad heat allowance? | Track it and call for new pads | D-52 |

## Still open

- **Q-20, Q-21, Q-33** (clinician): the last-seconds blind spot, how strict the countdown is, and movement during the countdown.
- **Q-35** (owner, later): how to obtain the reference test equipment.
- **Q-39** (after bench tests): two larger battery cells if cold-weather tests show too little margin.

- **Q-47** (owner): how records come off by cable. Either the dock is the reader (one extra contact on the device), or the device speaks USB itself through the dock (two extra contacts).
- **Q-48** (owner, with a lawyer): may the specification sheets for made-to-order parts be published now, or held for the patent review? They state what a part must do, not how to build anything, and are currently in the public parts study.
- **For the hardware engineer:** how to hold the voltage sensors to 1% + 7 V over the device's life (recalibrate at service, or costlier low-drift parts); the temperature range of the candidate pad-resistance chip.

## Things that need a professional, not a guess

- **Patent attorney:** freedom to operate, especially against Avive's 30+ patents on its compact unit and the Philips patent on analysing during CPR (to about 2036).
- **Regulatory consultant:** whether publishing buildable files, running responder trials with a trainer unit, or using the name "AED" creates obligations.
- **Lawyer:** product liability for publishers; a licence disclaimer does not bind an injured patient. Pennsylvania's Good Samaritan AED law protects users and owners, not designers.
