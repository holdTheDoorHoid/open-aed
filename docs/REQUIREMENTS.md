# Draft target specification

Version 0.2, 26 November 2026. Every number here is provisional and traces to the research in [research_notes/](../research_notes/). Decision numbers (D-xx) and open questions (Q-xx) refer to [DECISIONS.md](../DECISIONS.md).

## Who it is for

Trained responders: police officers, paramedics, medical professionals. They carry it on a belt or vest all shift, so it competes for space with a radio, and it rides in patrol cars in summer and winter. They have training, so prompts can be shorter than on a public-access unit.

## Size and weight

| Target | Value | Why |
|---|---|---|
| Weight, finished design | 700 g or less, **including pads** (D-43). A first prototype of 780–900 g is accepted. | Matches the smallest full-energy units on sale (St John Pocket AED P3, Schiller FRED easyport plus at 780 g) |
| Volume, first prototype goal | 0.7 litre or less | Same |
| Stretch goal | 500 g and 0.5 litre | Not reachable with parts available today (parts study version 0.3) |
| Not a goal | 300 g, like the CellAED | That device reached its size by cutting shock energy and shock count, and its maker has had regulatory trouble. Catalogue parts do not get there at full energy. |

**What the parts study found (version 0.3, 27 November 2026):** a set of parts that follows every decision with lower-risk choices comes to about 0.89 L and 903 g with pads, about 200 g over the target. The smallest plausible set comes to about 0.64 L and 699 g, and only if a capacitor maker delivers the most optimistic estimate; with the expected capacitor it is about 783 g. A first prototype is more likely to weigh 780–900 g. The 500 g stretch goal is not reachable with parts available today. See [PARTS_STUDY.md](PARTS_STUDY.md).

For comparison, the Avive Connect is 953 g and mainstream AEDs are 1.4 to 3 kg.

## The shock

| Item | Target | Why |
|---|---|---|
| Waveform | Biphasic truncated exponential, adjusted to the patient's measured resistance | The best-proven shape. No human trial shows any other biphasic shape is better for cardiac arrest. Founding patents expired around 2013. (D-02) |
| Adult energy | 150 joules | Matches Philips and Avive. The FDA accepted Avive's bench proof that its 150 J waveform matched Philips' in place of a new human trial. (D-03) |
| Child energy | 50 joules, chosen in software | Matches Avive and the pocket units. (D-04) |
| Patient resistance range | 25 to 175 ohms | The range the defibrillator standard tests |
| Storage capacitor | About 100 µF charged to 1.7–2.0 kV | Follows from the above. Below about 60 µF the shock becomes too brief to work well. |
| Charge time | 7 seconds or less from a ready battery, so the charge finishes inside the analysis (D-29) | Provisional; the standard's exact limit must be read from the purchased standard |
| Shocks per battery charge | 30 or more | Provisional  |

## Pads

- **One pad set for every patient.** Two identical pads, each with roughly 55–70 cm² of gel. That is above the adult minimum (50 cm² each) and small enough to fit an infant. (D-05)
- Child mode is a switch on the device, not a different pad or an attenuator cable.
- In child mode the device instructs front-and-back placement, as the 2025 European guideline recommends for children under 25 kg.
- Pads stay plugged in so there is nothing to connect at the scene.
- Open problem: pads expire because the gel dries out. Pad design and supply is its own workstream, and pads are regulated as strictly as the device.

## Deciding whether to shock

- Must meet the American Heart Association targets: at least 90% of coarse ventricular fibrillation caught, at least 75% of rapid ventricular tachycardia, at least 99% of normal rhythms correctly left alone, at least 95% for flatline and other non-shockable rhythms.
- Must be proven on real cardiac-arrest recordings, not only public databases. No open set of such recordings exists, so this needs a data-sharing agreement. (Q-09)
- Design pattern: a conventional, explainable detector and a small neural network must agree, and the system may answer "not sure, keep doing CPR" instead of guessing.
- The device records measures that predict shock success, but they do not influence the shock decision. The one human trial showed no benefit.
- Goal for a later version: analyse the rhythm during chest compressions, because every 5 seconds of pause before a shock lowers survival. One relevant patent runs to about 2036.

## Operation

- **Fully automatic** (D-06): when a shock is advised the device warns everyone to stand clear, counts down three seconds aloud and on screen (D-30), and delivers the shock itself. No shock button.
- **Hands-off target:** about 9 seconds from "stop CPR" to shock in simulation, by charging during the analysis (D-29).
- **Voltage sensing:** two independent sensors, each accurate to about 1% + 7 V (D-28).
- Because no human confirms the shock, the design must include: a loud stand-clear warning and countdown; a cancel control the responder can hit at any time; a check that the pads are still attached and the signal is still shockable right up to delivery; and the independent second check on the shock path. Whether the device can sense someone touching the patient is an open research question.
- The accuracy bar for leaving normal rhythms alone matters even more than on a semi-automatic unit.
- Short prompts for trained users, loud enough for a roadside, with a screen that can be read in sunlight and worked with gloves.
- Records the whole event for the handover and for review.

## Power

- Rechargeable lithium-ion, charged like a radio. Two cells (D-44), so charging stays fast at −20 °C; together they hold enough for a few hundred charges of the capacitor. A unit that is worn daily does not need the four-year standby battery of a wall-mounted AED, and this is the biggest single size saving. (D-07)
- Self-test must load the battery realistically. A past fatal recall came from battery logic that misjudged the charge.

## Toughness

| Item | Target |
|---|---|
| Dust and water | IP65 or better |
| Drop | 1.5 m onto concrete |
| Operating temperature | −20 to +50 °C target (commercial AEDs are rated 0 to 50 °C) |
| Storage heat | **Left in a vehicle for short periods, with an insulated pouch kept in the vehicle; not kept on a dashboard** (D-48). Long-term storage up to about 40 °C (set by the pads); stays ready up to 60 °C for up to 8 hours at a time; "not ready" above 60 °C until cooled and re-tested; "service needed" above 70 °C. The device records its own temperature at all times, adds up pad heat exposure and calls for new pads (D-52). |
| Charging temperature | Battery charging only between 0 and 45 °C. Capacitor charging only up to 60 °C. A device switched on while hotter than that guides CPR and analyses, and shocks once it has cooled (D-51). |

## Readiness and self-test

Field surveys found 15–21% of public AEDs not ready when inspected, and an analysis of 1,150 fatal failures found about a quarter caused by battery or power and about a quarter by pads and connectors. So:

- Self-test into an internal test load: full energy once a week, a low-voltage check every day (D-15).
- Battery tested under a realistic load, with cautious low-battery logic.
- Pad connection and gel condition checked through the pads' electrical resistance.
- A clear ready / not-ready indicator, and readiness reported to the agency over Bluetooth.

## CPR support

- **Included, because the existing pads can measure them with no extra sensor:** compression rate (published error under 2 per minute), pauses, time-on-chest, and a metronome. A metronome alone moved field crews from 18% to 62% of minutes on target rate.
- **Deferred:** compression depth. It needs an added sensor on the chest, and a single sensor over-reads on a mattress. The 2025 international review gives feedback during real arrests only a weak recommendation; the survival evidence is mixed.
- **Research goal, advisory only:** telling the responder a pulse may have returned, from the heart trace and pad resistance (about 93% accurate in research; no commercial AED does this).

## Software and electronics

- Firmware in Rust. (D-08) A safety-qualified Rust compiler (Ferrocene) exists and has been qualified to the medical software standard at its strictest class since January 2025. **Unresolved:** two research threads disagree on whether that qualification covers the small microcontroller family this device would use. Check with the vendor before choosing the processor.
- No single fault may cause or block a shock. Published designs converge on three independent checks before energy is released: the main processor, a separate supervisor, and a hardware interlock.
- The storage capacitor discharges itself by default whenever it is not actively being held for a shock, with a permanent backup bleed resistor as well.
- Catalogue parts wherever they exist; made-to-order parts where they do not, each with a published specification (D-42). The storage capacitor is made to order: about 100 µF, at least 2,200 V, about 70 °C storage, charged only up to 60 °C, at least 2,500 full charges (D-41, D-53).

## Connectivity

None in the first prototype (D-09). Event records and heat history come off by cable through the dock contacts. Space is left for a separate pre-certified Bluetooth module later (D-45). Cellular and location reporting are deferred: any internet-capable device takes on the FDA's cybersecurity requirements.
