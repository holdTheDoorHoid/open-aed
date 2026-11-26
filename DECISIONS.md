# Decisions

This file is the source of truth. If another document disagrees with it, this file wins and the other document is wrong.

- **D-xx** are decisions. Those marked *provisional* were made by Claude on 26 November 2026 from the research so that planning could proceed; the owner has not confirmed them.
- **Q-xx** are open questions for the project owner.

## Decisions

| No. | Decision | Status | Reason |
|---|---|---|---|
| D-01 | "Rive AED" in the original request is taken to mean the **Avive Connect**. The CellAED is studied as the other very small unit. | **Confirmed by owner, 26 Nov 2026** | No product named "Rive" was found; the owner confirmed the Avive Connect was meant. |
| D-02 | The shock is a **biphasic truncated exponential** waveform adjusted to the patient's resistance. | Provisional | Best-proven shape; no human trial shows another is better for cardiac arrest; founding patents have expired. |
| D-03 | Adult energy is **150 joules**. | Provisional (Q-07) | Matches Philips and Avive, which opens the "prove equivalence on the bench" approval route. |
| D-04 | Child energy is **50 joules**, selected in software. | Provisional | Matches Avive and the pocket units; avoids a separate attenuator or pad. |
| D-05 | **One pad set** for all patients, about 55–70 cm² of gel per pad, front-and-back placement in child mode. | Provisional | Meets the adult minimum area while fitting an infant. Avive's FDA approval summary and help page describe the same-pads approach, though one spec-page reading says separate child pads; to be settled by reading the FDA summary in full. Several other units (Zoll AED 3, Schiller, Stryker CR2) also use one pad set with a child button. |
| D-06 | **Semi-automatic:** the device advises, the responder presses the button. | Provisional (Q-10) | Users are trained; keeps a human in the loop. |
| D-07 | **Rechargeable** lithium-ion battery. | Provisional (Q-06) | A belt-worn unit is charged like a radio; this is the biggest single size saving. |
| D-08 | Firmware is written in **Rust**. | Provisional | Owner's standing preference; memory safety suits safety-critical code. |
| D-09 | First version has **Bluetooth only**; cellular and location are deferred. | Provisional (Q-08) | Internet-capable devices take on FDA cybersecurity requirements. |
| D-10 | **No circuit above 50 volts** is built or powered until a reviewed lab safety policy exists. | Provisional | The stored energy can kill the builder. See SAFETY.md. |
| D-11 | Measures that predict shock success are **recorded but never used** to decide a shock. | Provisional | The one human trial showed no benefit. |
| D-12 | The project does **not distribute assembled units, boards or kits**. | Provisional | Doing so may make the project a "manufacturer" in law and invites use on people. Needs a lawyer before it could change. |
| D-13 | The GitHub repository starts **private**. | Provisional (Q-02) | The owner chooses when a high-voltage medical design goes public. |

| D-14 | First version includes **compression rate, pause timing and a metronome** measured through the pads; compression-depth sensing is deferred. | Provisional | The first three need no extra hardware. Depth needs an added sensor and the survival evidence for it is mixed. |
| D-15 | The self-test **charges to full energy into an internal test load** and tests the battery under load. | Provisional | Battery, pad and charging faults account for most recorded fatal AED failures. |

## Open questions for the owner

**Q-01. Did you mean the Avive Connect?** *Answered 26 Nov 2026: yes.*

**Q-02. Name, and public or private?** The working name is `open-aed`. Recommendation: keep the name, keep the repository private until the licence is chosen and you have read the plan, then make it public. Publishing early is one of the things that separated open medical projects that succeeded from those that stalled.

**Q-03. Which licence?** Recommendation: CERN-OHL-S v2 for hardware, GPL v3 for firmware, CC BY-SA 4.0 for documents. These require anyone who improves the design to share the improvement. The trade-off: a manufacturer partner may prefer a more permissive licence (CERN-OHL-W or -P, Apache 2.0) so it can keep some changes private.

**Q-04. What is the end goal?** (a) A reference and education platform that never touches a patient; (b) a design mature enough for a regulated partner to take through approval; (c) founding a non-profit to seek approval itself. The plan assumes (b). Option (c) means millions of dollars and years.

**Q-05. Who does the high-voltage work?** It needs someone with professional high-voltage experience, a second person present, and a proper enclosure. Recommendation: nobody starts until that person is found; Phase 1 has months of hazard-free work in the meantime.

**Q-06. Rechargeable, or long-standby?** Rechargeable makes the device much smaller but it must be charged regularly, like a radio. A long-standby primary battery suits a unit that sits in a glovebox for years. The plan assumes rechargeable.

**Q-07. 150 joules fixed, or able to step up to 200?** One trial found stepping up helped on second and later shocks; manufacturers dispute how far that applies to a 150 J device. Stepping up to 200 J needs a capacitor roughly a third larger. The plan assumes fixed 150 J for the first prototype, with the electronics laid out so a 200 J version is possible.

**Q-08. How connected should it be?** Bluetooth only (assumed); or cellular with location so that dispatch can see where units are. Cellular adds size, a subscription, and regulatory cybersecurity work.

**Q-09. Data partners.** Do you have contacts at an ambulance service, hospital or university who might share cardiac-arrest recordings under an agreement? This is the main bottleneck for the rhythm software.

**Q-10. Semi-automatic only, or also a manual mode for paramedics?** A manual mode (the paramedic reads the rhythm and decides) is normal on professional monitors, and falls in a lower EU risk class, but adds a screen good enough to read a heart trace.

**Q-11. Budget.** Near-term costs are small but real: the defibrillator standard (a few hundred dollars), development boards, and later a used commercial analyser and the three advisers. Is there a budget, or should the plan favour free options and seek grants?

## Things that need a professional, not a guess

- **Patent attorney:** freedom to operate, especially against Avive's 30+ patents on its compact unit and the Philips patent on analysing during CPR (to about 2036).
- **Regulatory consultant:** whether publishing buildable files, running responder trials with a trainer unit, or using the name "AED" creates obligations.
- **Lawyer:** product liability for publishers; a licence disclaimer does not bind an injured patient. Pennsylvania's Good Samaritan AED law protects users and owners, not designers.
