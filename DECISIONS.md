# Decisions

This file is the source of truth. If another document disagrees with it, this file wins and the other document is wrong.

- **D-xx** are decisions. Those marked *provisional* were made by Claude on 26 November 2026 from the research so that planning could proceed; the owner has not confirmed them.
- **Q-xx** are open questions for the project owner.

## Decisions

| No. | Decision | Status | Reason |
|---|---|---|---|
| D-01 | "Rive AED" in the original request is taken to mean the **Avive Connect**. The CellAED is studied as the other very small unit. | **Confirmed by owner, 26 Nov 2026** | No product named "Rive" was found; the owner confirmed the Avive Connect was meant. |
| D-02 | The shock is a **biphasic truncated exponential** waveform adjusted to the patient's resistance. | Provisional | Best-proven shape; no human trial shows another is better for cardiac arrest; founding patents have expired. |
| D-03 | Adult energy is **fixed 150 joules** for the first prototype, with the board laid out so a 200 joule version stays possible. | **Confirmed by owner, 26 Nov 2026** | Matches Philips and Avive, which opens the "prove equivalence on the bench" approval route. |
| D-04 | Child energy is **50 joules**, selected in software. | Provisional | Matches Avive and the pocket units; avoids a separate attenuator or pad. |
| D-05 | **One pad set** for all patients, about 55–70 cm² of gel per pad, front-and-back placement in child mode. | Provisional | Meets the adult minimum area while fitting an infant. Avive's FDA approval summary and help page describe the same-pads approach, though one spec-page reading says separate child pads; to be settled by reading the FDA summary in full. Several other units (Zoll AED 3, Schiller, Stryker CR2) also use one pad set with a child button. |
| D-06 | **Fully automatic:** the device analyses and, if a shock is advised, warns, counts down and delivers it without a button press. | **Confirmed by owner, 26 Nov 2026** (changed from semi-automatic) | Owner decision. Removes the delay and hesitation of a button press. It raises the bar on the software: with no human confirming, a wrong shock decision is delivered, so the stand-clear warning, countdown, a way to cancel, and the independent second check all become essential. |
| D-07 | **Rechargeable** lithium-ion battery. | **Confirmed by owner, 26 Nov 2026** | A belt-worn unit is charged like a radio; this is the biggest single size saving. |
| D-08 | Firmware is written in **Rust**. | Provisional | Owner's standing preference; memory safety suits safety-critical code. |
| D-09 | First version has **Bluetooth only**; cellular and location are deferred. | **Confirmed by owner, 26 Nov 2026** | Internet-capable devices take on FDA cybersecurity requirements. |
| D-10 | **No circuit above 50 volts** is built or powered until a reviewed lab safety policy exists. | Provisional | The stored energy can kill the builder. See SAFETY.md. |
| D-11 | Measures that predict shock success are **recorded but never used** to decide a shock. | Provisional | The one human trial showed no benefit. |
| D-12 | The project does **not distribute assembled units, boards or kits**. | Provisional | Doing so may make the project a "manufacturer" in law and invites use on people. Needs a lawyer before it could change. |
| D-13 | The GitHub repository stays **private until the licence is in place and the owner has read the plan and safety rules**, then goes public. Buildable high-voltage design files wait for the patent review. | **Confirmed by owner, 26 Nov 2026** | The owner chooses when a high-voltage medical design goes public. |

| D-14 | First version includes **compression rate, pause timing and a metronome** measured through the pads; compression-depth sensing is deferred. | Provisional | The first three need no extra hardware. Depth needs an added sensor and the survival evidence for it is mixed. |
| D-15 | The self-test **charges to full energy into an internal test load** and tests the battery under load. | Provisional | Battery, pad and charging faults account for most recorded fatal AED failures. |
| D-16 | **End goal: a partner-ready design.** The project builds and tests an open design good enough for a regulated manufacturer to take through approval. It does not seek approval itself. | **Confirmed by owner, 26 Nov 2026** | Owner decision. Matches the pattern that has worked for other open medical devices. |
| D-17 | **Licence: share-alike.** CERN-OHL-S v2 for hardware, GPL v3 (or later) for firmware and software, CC BY-SA 4.0 for documents and research. | **Confirmed by owner, 26 Nov 2026** | Owner decision. Anyone who improves the design must publish the improvement. |
| D-18 | **High-voltage work waits for an experienced engineer.** No hands-on high-voltage building until someone with professional high-voltage experience joins the project. | **Confirmed by owner, 26 Nov 2026** | Owner decision. Phase 1 is hazard-free and comes first anyway. |
| D-19 | **No manual mode.** The device is fully automatic only. | **Confirmed by owner, 26 Nov 2026** | Owner decision (see D-06). Keeps the screen small and simple. |
| D-20 | **Data partnerships are parked.** Rhythm software is developed on public data first; outreach for real cardiac-arrest recordings resumes once it works on those. | **Confirmed by owner, 26 Nov 2026** | Owner decision. Still required before Phase 5. |
| D-21 | **Budget: small and self-funded, with grant applications** for the larger costs. | **Confirmed by owner, 26 Nov 2026** | Owner decision. |
| D-22 | **Firmware is hardware-agnostic.** Everything that decides anything runs and is tested on an ordinary computer; the processor is chosen later and connected through a thin layer. | Provisional | Keeps the processor choice open while the Rust compiler qualification question is unresolved, and makes the safety logic testable now. |
| D-23 | **Model design point: 100 µF charged to 1,950 V (adult) or 1,150 V (child)**, voltage fixed per mode, phase lengths chosen from the measured resistance. | Provisional, model only | Delivers within 7% of target across 25–175 ohms in the model. Depends on an assumed 5 ohm internal resistance. |
| D-24 | **Only third-party code on the device is `libm`** (maths functions). Additions are listed in `firmware/DEPENDENCIES.md` with a reason. | Provisional | The medical software standard requires every outside component to be justified and tracked. |

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

## New questions from the first firmware and simulation work (27 November 2026)

**Q-12. The 200 joule headroom does not fit.** D-03 says "fixed 150 J, with room for 200 J later". The model shows a 100 µF capacitor at the 2,000 volt ceiling stores exactly 200 J, so it cannot deliver 200 J to a patient. A 200 J version needs a capacitor about a third larger or a higher voltage with higher-rated parts throughout. Options: (a) accept 150 J only and drop the headroom; (b) size the capacitor space and voltage ratings for 200 J now, at a size cost; (c) decide after the bench prototype.

**Q-13. Charge while analysing?** In simulation the responder's hands are off the chest for about 17 seconds before each shock: 7 s analysing, 6 s charging, 5 s countdown. Starting the charge during analysis would remove most of the 6 s. The cost: the capacitor is charged more often when no shock follows, and that energy is then burned off internally.

**Q-14. How long should the stand-clear countdown be?** It is 5 seconds as a placeholder. Shorter saves hands-off time; longer gives people more warning on a fully automatic device.

**Q-15. Cancel pressed during the switch-on self-test** currently counts as a failed self-test and disables shocks for that session. Should a stray press at switch-on be ignored instead?

**Q-16. Pads on, but resistance just above the 175 ohm limit.** Today the device only repeats "attach pads" and never analyses. Some commercial units accept up to 180–230 ohms. Widen the range, or add a distinct "check pads, continue CPR" behaviour?

## Things that need a professional, not a guess

- **Patent attorney:** freedom to operate, especially against Avive's 30+ patents on its compact unit and the Philips patent on analysing during CPR (to about 2036).
- **Regulatory consultant:** whether publishing buildable files, running responder trials with a trainer unit, or using the name "AED" creates obligations.
- **Lawyer:** product liability for publishers; a licence disclaimer does not bind an injured patient. Pennsylvania's Good Samaritan AED law protects users and owners, not designers.
