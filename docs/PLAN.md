# Project plan

Version 0.1, 26 November 2026. Decision numbers (D-xx) and open questions (Q-xx) refer to [DECISIONS.md](../DECISIONS.md).

## The goal

A fully open, documented and tested design for a belt-worn AED, built from parts anyone can buy, that works on adults and children with one set of pads, and that a regulated manufacturer could take through approval.

## What the research changed

1. **Nothing open exists to build on.** The one open design (University of Pisa, 2017) simulated its shock circuit and never tested it. This project starts close to zero on hardware.
2. **The size target is already proven possible.** Commercial pocket units deliver 150 J at about 0.7 kg. The job is to match that openly, then shrink.
3. **Copying the proven waveform is the smart route.** The FDA approved Avive's adult shock on bench proof that it matched Philips' published waveform, with no new human trial. A novel lower-energy waveform would shrink the device further but would need far more evidence. So this project innovates in packaging, parts, software and openness, and stays conventional in the shock itself.
4. **Data is the bottleneck for the software.** There is no open collection of real cardiac-arrest recordings. Every serious rhythm-detection study uses private data.
5. **Test equipment must come first.** No validated open instrument for measuring a defibrillator's output exists either.
6. **Reaching patients needs a regulated partner.** The pattern that has worked for open medical devices is an open design upstream and a regulated organisation that takes one frozen version through approval.

## Phases

Each phase ends with a review. The next phase starts only when the exit conditions are met.

### Phase 0 — Foundation (now)

No hardware. No hazard.

- Research report and topic notes. *Done in first draft.*
- Draft requirements, decision log, safety rules. *Done in first draft.*
- Owner answers the open questions in DECISIONS.md. *Done 26 November 2026.*
- Choose the licence. *Done: share-alike (D-17).*
- Owner reads the plan and safety rules; then the repository goes public (D-13).
- Start recruiting an engineer with professional high-voltage experience (D-18); Phase 3 cannot start without one.
- Buy the current defibrillator standard (IEC 60601-2-4 edition 3.2, September 2026) and replace every "unverified" limit in the requirements with the real figure.
- Start looking for three advisers: a resuscitation clinician, a medical-device regulatory consultant, and a patent attorney.

**Exit:** owner decisions recorded; licence added; the standard's real limits are in the requirements.

### Phase 1 — Low-voltage work and software

Nothing above battery voltage. No shock hazard.

- **Heart-signal board:** an off-the-shelf ECG chip on a development board, with the protection parts that will later let it survive a shock.
- **Rhythm analysis in Rust:** first the classical, explainable detectors, measured on the public databases with patients kept separate between training and testing. Then a small neural network alongside. Publish the scores against the AHA targets, including the failures.
- **Heart-signal simulator:** plays recorded rhythms into the board so the whole chain can be tested without a person.
- **Firmware skeleton:** the state machine (pads on, analysing, charging, shock advised, CPR), prompts, event recording, and the two-controller agreement scheme, running on development boards.
- **Circuit simulation** of the charger and output stage, to size parts before buying any.
- **User-interface mock-up** to try prompt wording and screen layouts with real responders.
- Data partnerships: outreach letters drafted 27 November 2026 in `docs/outreach/` (D-20). The owner sends them. This must bear fruit before Phase 5.

**Exit:** rhythm analysis meets the AHA targets on public data with honest patient-separated testing; firmware state machine runs end to end against the simulator.

### Phase 2 — Safety policy and test rig

- Write the **lab safety policy** required by SAFETY.md and have it reviewed by someone with professional high-voltage experience.
- Build the **open defibrillator analyser:** a 50 ohm test load rated for the pulse, plus voltage and current capture that computes delivered energy. Add switchable loads from 25 to 175 ohms.
- Check the analyser against a commercial one (for example a used Fluke Impulse) and against a commercial AED's known output.
- Build the interlocked test enclosure.

**Exit:** safety policy reviewed and adopted; analyser agrees with a commercial instrument within a stated error.

### Phase 3 — High-voltage bench prototype

Large and open on purpose. Size does not matter yet; getting the waveform right and safe does.

- Capacitor charger, starting from Texas Instruments' published AED charger reference design.
- Storage capacitor: test both candidate types (film, and a bank of camera-flash capacitors).
- Output bridge that produces the two-phase pulse and cuts it off at the right moment for the measured resistance.
- Safe internal discharge for a cancelled shock.
- Measure the waveform across 25–175 ohms and compare it, parameter by parameter, with the published Philips and Avive waveforms.
- Fault testing: prove that single failures neither cause nor block a shock.

**Exit:** delivered energy and waveform shape within tolerance across the full resistance range, hundreds of shocks, with fault tests passed.

### Phase 4 — Integration and shrinking

- Join the Phase 1 and Phase 3 halves into one working unit and prove the heart-signal input recovers after a shock.
- Pad design: universal size, connector, shelf-life testing.
- Fold it into the belt-worn form: board layout with proper high-voltage spacing, enclosure, battery, speaker, screen.
- Ruggedness: water, dust, drop, vibration, heat and cold.
- Responder trials with a non-shocking trainer version: carry comfort, deployment time, prompts.

**Exit:** a belt-sized unit that meets the size, energy and toughness targets on the bench.

### Phase 5 — Proving it

- Pre-compliance testing against the safety, electromagnetic and defibrillator standards at a test lab.
- Rhythm analysis validated on real cardiac-arrest recordings under a data agreement.
- Formal usability study.
- Complete the design history: requirements, risk analysis, verification records, software documentation.

**Exit:** a frozen, documented design release with test evidence.

### Phase 6 — Path to patients

Not something a volunteer project does alone. This phase belongs to a regulated organisation: a manufacturer partner or a non-profit set up for the purpose. It includes the animal study for the child dose, the approval submission, manufacturing under a quality system, and post-market monitoring. The FDA's filing fee alone was about $145,000 for a small business in 2026, before any testing costs.

The project's job is to make Phases 0–5 good enough that such a partner wants to pick it up.

## Workstreams

| Workstream | Starts | Main risk |
|---|---|---|
| Shock electronics | Phase 1 (simulation), Phase 3 (hardware) | Builder safety; capacitor size |
| Rhythm analysis | Phase 1 | Access to real cardiac-arrest data |
| Firmware and safety architecture | Phase 1 | Discipline: documentation from day one |
| Pads | Phase 4, research from Phase 0 | Gel shelf life; regulated as strictly as the device |
| Test equipment | Phase 2 | Accuracy must be proven against a commercial instrument |
| Enclosure and human factors | Phase 1 (mock-ups), Phase 4 | Parked-car heat; glove use; noise |
| Regulatory, legal, patents | Phase 0 | Avive holds 30+ patents on its compact unit; liability of publishing and of kits |

## Known technical unknowns

These came out of the research unresolved and each needs an answer before the design depends on it.

- **Parked-car heat.** Cabin temperatures exceed the rating of the capacitors that give the smallest size.
- **Charging transformer.** No catalogue transformer reaches 2,000 volts; this is the one part that must be custom, or avoided by charging several lower-voltage capacitors side by side and connecting them end to end for the shock. Avive holds patents on that second approach.
- **Output switches.** A switch rated for 3,300 volts and about 80 amps of pulse exists in the catalogue, but its pulse rating has unknown conditions. The traditional alternative (thyristors plus one transistor) is in expired patents.
- **Spacing rules at 2,000 volts.** Free sources only give the safety standard's spacing tables up to 566 volts. The real figures need the purchased standard, and they drive how small the circuit board can be.
- **Rust compiler qualification** for the chosen processor family: sources disagree.
- **Pad gel shelf life** in a belt pouch through heat and cold.

## Top risks

1. **Someone is hurt building it.** Countered by SAFETY.md, the phase order, and the reviewed lab policy.
2. **Someone uses a prototype on a person.** Countered by clear labelling, not distributing assembled units or kits, and the trainer-only rule for field trials.
3. **Patents.** Countered by an attorney's review before any hardware design is published in buildable form.
4. **No real-world data for the rhythm software.** Countered by starting data-partnership conversations in Phase 1.
5. **The project stalls like the 2020 open ventilators.** Most of those failed because teams worked in isolation, published late and had no manufacturer to carry the design through regulators. Countered by publishing early, documenting fully, and seeking a regulated partner well before Phase 5.
