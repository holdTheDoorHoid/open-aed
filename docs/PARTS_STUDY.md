# Parts study and size, weight and power budget

| Version | Date | What changed |
|---|---|---|
| 0.1 | 27 Nov 2026 | First paper study of the parts, with a size, weight, cost and power roll-up. |
| 0.2 | 27 Nov 2026 | Re-done after the owner's decisions D-15, D-39 and D-41 to D-46: 2.2 kV capacitor with 85 °C storage, two cells, separate Bluetooth module, dock contacts, pads counted in the 700 g. Capacitor requirements sheet (Appendix A) and other made-to-order requirement lists (Appendix B) added. |
| **0.3** | **27 Nov 2026** | After D-41 was revised (capacitor storage lowered from 85 °C to about 70 °C) and D-48 set the ruggedness goal. **New:** heat limits of every temperature-sensitive part from makers' documents, published parked-car temperatures, and a proposed storage and heat envelope with three options (§8A). Capacitor re-sized for 70 °C storage, roll-up re-done, levers re-ranked, Appendix A moved to 70 °C, owner questions updated. **Finding:** dropping 85 °C storage saves almost no weight by this study's own method; version 0.2 overstated the saving (§1). |
| **0.4** | **27 Nov 2026** | After the owner's answers recorded as D-09, D-41, D-43, D-48 and D-51 to D-53. Capacitor re-sized for **60 °C working and 2,500 full charges together**, with a limit on how far the two can be combined (§1). **Bluetooth removed** from the first prototype, its space kept empty; records come off through **data contacts in the dock** (§5, §7). **Vehicle pouch** added as an accessory with a performance requirement (§8A). Roll-up, decision table, levers, owner questions and Appendix A updated. **Result:** lower-risk set about **832 g**, smallest set about **653 g**, both with pads. |

This is a paper study from datasheets, maker manuals, maker pages, distributor listings and published studies, checked on 27 November 2026. **No part has been bought, measured or tested. No hardware exists.** Working notes and every source link are in [research_notes/Open source belt worn AED/parts_study_sources.md](../research_notes/Open%20source%20belt%20worn%20AED/parts_study_sources.md). Decision numbers (D-xx) and open questions (Q-xx) refer to [DECISIONS.md](../DECISIONS.md).

> **What this document is not.** It names kinds of parts, specific part numbers and their published ratings, and does sizing arithmetic. The appendices are *requirement sheets for parts to be bought*: what the part must do and what proof to ask the maker for. They say nothing about how to make the part. The heat section states what the device must *do* (requirements), not how its circuits do it. This document contains no schematics, wiring, board layouts, winding details or assembly steps, and nothing in it is an instruction to build or power anything above 50 volts (D-10, SAFETY.md). Nothing here is safe or ready for use on a person or animal.

How to read the tags:
- **[CALC]** is arithmetic done for this study. The working is shown.
- **[EST]** is an engineering estimate. Its basis is stated.
- **[BK]** is background knowledge from training that was not re-checked this time. Treat it as something to check, not a citation.
- **[UNVERIFIED]** means I looked and could not confirm it.
- **"Checked 27 Nov 2026"** means the listing or page was seen that day.

Most distributor pages (Digi-Key, Ohmite, TME) refused automated access on the day. Most prices below are therefore **estimates**, not quotes. They must be re-checked by hand before anything is ordered. Several makers' PDF manuals could not be opened by the research tools; where a figure comes from a search-result extract of a maker's document rather than the document itself, it says so.

---

## What changed in version 0.4

| Decision (27 Nov 2026) | What it means for this study | Where |
|---|---|---|
| D-41: about 100 µF, ≥ 2,200 V, about 70 °C storage, **charged only up to 60 °C**, **≥ 2,500 full charges** | Capacitor re-sized for both changes at once; the combined saving is capped by what makers can show (§1). Appendix A rewritten to these figures | §1, Appendix A |
| D-53: count full charges; service as the capacitor nears 2,500; heavy-duty capacitor later | Charge counter added to the requirements; heavy-duty variant kept in Appendix A as a later option | §1, §8A, Appendix A |
| D-09 and D-45: no Bluetooth in the first prototype; records by cable through the dock contacts; space for a module later | Module removed from the parts set; its volume reserved as empty space; data contacts added to the dock requirements | §5, §7, §9 |
| D-48: envelope option A plus an insulated pouch kept in the vehicle | Envelope fixed; the pouch gets a performance requirement and candidate product classes | §8A |
| D-51: a device switched on while too hot guides CPR and analyses, but does not charge until back inside the limits | Written into the charging requirements | §8A |
| D-52: total pad time above 50 °C; new pads after about 7 days | Written into the logging requirements | §8A |
| D-43: 700 g with pads is the goal for the finished design; a first prototype of 780–900 g is accepted | Roll-up judged against both | §9 |

Version 0.3's changes (heat limits of every part, parked-car data, the envelope options, the 85 °C correction) are kept in place and marked where they have since been decided.

---

## Summary for the owner

- **Where the weight now lands (with pads, against 700 g and 0.7 L):**
  - **Lower-risk set, expected capacitor: about 0.81 L and 832 g.** That is **132 g (19%) and 0.11 L (16%) over** the finished-design goal, and inside the 780–900 g you accepted for a first prototype (D-43). Version 0.3 said 903 g.
  - **Smallest plausible set, low capacitor: about 0.59 L and 653 g**, **47 g and 0.11 L under** the goal. With the expected capacitor it is **714 g, 14 g over**.
  - **Plainly:** the first prototype should land around 830 g. 700 g for the finished design is now within reach if most of the levers that need no further decision work out (§9), and does not depend on the most optimistic capacitor any more.
- **What bought the weight: the capacitor decisions (D-41, D-53).** Charging only up to 60 °C and sizing for 2,500 full charges together take the expected capacitor from **210 cm³ / 300 g to about 170 cm³ / 240 g**, saving about **71 g** on the finished device.
  - **The two changes do not simply stack.** Each lets the maker stress the plastic film a little harder. Taken at face value the arithmetic would allow a film about 11–15% denser than the densest defibrillator capacitor any maker publishes at this voltage. The study does not count on that: the expected case is capped at the densest part on sale (General Atomics, 1.66 J/cc). Anything beyond that is the low case and is up to the makers to show.
  - **Estimate: about 145 / 170 / 210 cm³ and 185 / 240 / 315 g** (low / expected / high). Moderately sure of the volume, less sure of the mass (§1).
  - **Life:** on paper the expected part lasts about 3,900 full charges at the 60 °C limit [CALC], against the 2,500 required. A typical user does about 1,220 in ten years. A pool device used heavily would reach 2,500 in about six years, and D-53's charge counter will call for service in time.
- **Bluetooth is out of the first prototype (D-09).** About 1 g removed. **2 cm³ is kept empty** where the module and its antenna clearance would go. Records and heat history come off through **one or two extra data contacts** next to the charging contacts in the dock (about 1 cm³ and 1 g).
- **Heat (D-48, D-51, D-52).** Ready up to 60 °C for up to 8 hours at a time, "not ready" from 60 to 70 °C, "service needed" above 70 °C. Battery charged only between 0 and 45 °C, capacitor only up to 60 °C. A device switched on while too hot guides CPR and analyses but does not charge until it is back inside the limits. Pad time above 50 °C is totalled and new pads are called for after about 7 days.
- **The vehicle pouch** is an accessory, not carried. It must **at least triple** how long the device takes to heat: moved into a 60 °C car, the device must take at least 52 minutes to make half the rise, with a target of an hour [CALC, §8A, H8]. Suitable product classes are insulated medicine and insulin travel cases, and foil-faced closed-cell foam pouches. About $10–30 [EST].
- **What still sets the heat limits** (unchanged from version 0.3): the pads (35–40 °C long term, about 60 °C for short periods) and the lithium-ion cells (about 60 °C). No option survives a dashboard in direct sun.

---

## Terms used once and then assumed

| Term | Meaning |
|---|---|
| Capacitor | The part that stores the shock energy, like a fast battery that empties in milliseconds. |
| µF (microfarad) | The size of a capacitor. Stored energy = ½ × capacitance × voltage². |
| J/cm³ or J/cc (energy density) | Joules stored per cubic centimetre. Higher means a smaller capacitor. Makers quote it **at the part's rated voltage**. |
| Film capacitor | Plastic film (polypropylene) rolled with metal coating. What hospital and public-access AEDs use. Long-lived and stable. |
| Electrolytic capacitor | Aluminium foil in liquid. Denser, but limited to about 450 V per part, so many must be connected end to end to reach 2,000 V. Ages and dries out. |
| Photoflash capacitor | An electrolytic built for camera flashes, about 330 V, rated for rapid discharge. |
| Series / parallel | Parts connected end to end (series) share the voltage and carry the same current. Parts side by side (parallel) share the current. |
| Balancing | Keeping parts in series sharing the voltage evenly. Battery cells in series need it too. |
| Rated voltage | The highest voltage the maker allows continuously. **Derating** is running below it to make the part last. 1,950 V on a 2,200 V part is 89% of rating. |
| Made to order | Built by a catalogue maker to a customer's specification, not picked from stock (D-42). |
| Self-healing | A small fault in a film capacitor burns away the thin metal around it and isolates itself; the part loses a tiny amount of capacitance instead of failing. |
| Pressure interrupter | A safety feature in some capacitors that disconnects the part if gas from internal faults swells the case. |
| ESR / ESL | Internal resistance and internal inductance of a capacitor. Lower is better; both waste or delay the shock a little. |
| Dielectric absorption | The few volts a capacitor recovers on its own after being emptied. |
| Flyback charger | The circuit that turns the battery's few volts into about 2,000 V, using a transformer. |
| Transformer | Two coils on a magnetic core that step voltage up or down. |
| SiC MOSFET / IGBT / thyristor (SCR) | Three kinds of electronic switch. A SiC MOSFET or IGBT can be switched off at will. A thyristor, once on, stays on until the current stops. |
| Gate drive | The small control signal that turns a switch on. In the output bridge it must be insulated from up to 2,000 V. |
| Isolation, working voltage | How much voltage an insulating barrier can stand for its whole life (not just in a brief test). |
| Pulse / surge rating | How much current a part can take for a brief moment. Only useful if the datasheet says how brief. |
| Front end (AFE) | The chip that measures the tiny heart signal and the pads' resistance. |
| MCU | Microcontroller: a small computer on one chip. |
| HAL | The Rust code library that lets firmware use a particular MCU's pins and peripherals. |
| Pre-certified module / modular approval | A small radio board whose maker has already passed the radio tests. A product that follows the maker's conditions can reuse the approval instead of re-testing the radio. |
| Packing factor | The volume of the parts divided by the inside volume of the box. Real products are about 0.4–0.6. |
| dB SPL | Loudness. +10 dB sounds about twice as loud. |
| IP65 | Dust-tight, and protected against water jets. |
| Storage temperature | The range a part may sit in, switched off or idle, without damage. Makers often give a long-term range and a wider short-term range with a time limit. |
| Working (operating) temperature | The range in which a part may be used, for example charged and discharged. |
| Excursion | A limited time outside the normal storage range, such as a parked car on a hot afternoon. |
| Thermal time constant | How quickly the inside of the device follows the air around it. After one time constant it has made about two-thirds of the change; after three, about 95%. |
| LFP | Lithium iron phosphate, a lithium-ion chemistry that is more heat-tolerant and heavier for its energy than the usual high-drain cells. |
| JEITA rules | An industry practice for slowing or stopping battery charging when the cell is too cold or too hot. |

---

## The design point this study sizes against

From [DESIGN_TABLE.md](../firmware/crates/aed-waveform/DESIGN_TABLE.md) (model, not measurement), with the D-41 rating:

| Item | Adult | Child | Source |
|---|---:|---:|---|
| Capacitor | 100 µF | 100 µF | D-23, D-41 |
| Capacitor rating | at least 2,200 V; storage about 70 °C; charged only up to 60 °C; at least 2,500 full charges | | D-41, D-53 |
| Charge voltage | 1,950 V (worst-case true 2,008 V) | 1,150 V | D-23; FINDINGS.md |
| Stored energy | 190.1 J | 66.1 J | DESIGN_TABLE |
| Peak current (25 Ω) | 65.0 A | 38.3 A | DESIGN_TABLE |
| Longest shock | 15.5 ms | 14.0 ms | DESIGN_TABLE |
| Energy left after shock | up to 35.8 J | up to 14.7 J | DESIGN_TABLE |
| Charge time | ≤ 7 s, during the analysis | | D-29; REQUIREMENTS |
| Sensor accuracy | 1% + 7 V, two channels | | D-28 |

---

## 1. Energy storage capacitor

### What it must do

- Store at least **190 J at 1,950 V**. Rated **at least 2,200 V** (D-41), which is 10% above the 2,008 V worst case and 13% above the nominal 1,950 V [CALC].
- Give up 65 A in a few milliseconds.
- Survive the D-15 self-test schedule and the D-29 charge on every analysis, over a 10-year life.
- Survive **storage at about 70 °C**, be charged at up to **60 °C** (the device will not charge it hotter: D-41, D-48, D-51), and last at least **2,500 full charges** (D-41, D-53), in a device working at up to 50 °C outside.

### Catalogue and quote-only parts found in version 0.1

Kept for reference. Checked 27 Nov 2026. "Stored at 1,950 V" is [CALC] ½CV². None of these meets D-41 as listed: the compact film parts are rated 2.0 kV; the electrolytic options are not pulse parts.

| | A. GA 320DQ4130 (film) | B. CRE DMJ-PC 125 µF (film) | C. TDK B32365 (film, made to order) | D. Bank of 10 × 450 V 105 °C electrolytics | E. Bank of 28 × 330 V photoflash (Chemi-Con EPH-331) |
|---|---|---|---|---|---|
| Rating | 130 µF, 2.0 kV | 125 µF, 2,000 V "max 10 min" | 32–200 µF, to 5.5 kV, values on request | 5 in series × 2 strings, 270 µF / 450 V each [EST value] → 108 µF, 2,250 V total | 7 in series × 4 strings, 194 µF / 330 V → 111 µF, 2,310 V total |
| Stored at 1,950 V | 247 J | 238 J | as ordered | 205 J | 211 J |
| Voltage margin over 1,950 V | **2.6% (below the 2,008 V worst case)** | **2.6% (below 2,008 V)** | as ordered | 15% (if balanced) | 18% |
| Size | 33 × 55 × 99 mm = **180 cm³** | not published | "upon request" | **about 250 cm³ boxed** [CALC, EST size] | **about 175 cm³ boxed** |
| Mass | not published; about 230–270 g [EST] | **300 g (maker's figure)** | not published | about 310 g [CALC] | about 205 g [CALC] |
| Working temperature | not published [UNVERIFIED] | −25 to 60 °C | −20 to **55 °C** | to 105 °C | −20 to **65 °C** |
| Storage temperature | not published | **−40 to 85 °C** (maker's claim) | −20 to **70 °C** | to 105 °C, but needs re-forming | 65 °C |
| Cycle life | **2,500** | ≥ 10,000 | up to 10,000 **at 25 °C** | not rated for pulse discharge [UNVERIFIED] | 5,000 (camera-flash test) |
| Where to buy | quote only (General Atomics, US) | quote only (China) | quote only (TDK, made in Brazil) | Digi-Key, in stock | not found at Digi-Key, Mouser or LCSC |

General Atomics (GA) also lists two biphasic parts **rated 2.2 kV**, both 196 µF: 322DQ4196B (1.20 J/cc, 5,000 cycles, 49 × 74 × 129 mm) and 30920 (1.05 J/cc, 10,000 cycles, 49 × 74 × 146 mm). They are about twice the capacitance needed and twice the size. They matter here because they show what a 2.2 kV rating costs in density. CRE lists a 200 µF, 2,200 V part with storage −40 to 85 °C and working −25 to 70 °C, rated ≥ 2,500 cycles into 10 Ω; it publishes no size or mass for it.

**Who now meets the 70 °C storage figure (new in version 0.3).** At 85 °C only CRE claimed the storage rating. At 70 °C:
- **TDK's standard B32365 storage rating (−20 to +70 °C) now meets it**, but its +55 °C working limit still falls short of the 60 °C working requirement (D-41, Appendix A).
- **CRE** meets storage on both its parts; its 2.2 kV part's 70 °C working rating meets 60 °C working, and its published ≥ 2,500-cycle life (into 10 Ω) now matches Appendix A on paper. Its size and mass are unpublished; it is the first maker to ask.
- **GA and CDE/Knowles** publish no temperature ratings; still to be asked.
So the 70 °C decision widens the field from one maker to at least two whose *standard* storage ratings fit, which should lower qualification cost and risk. It does not, on the published evidence, make the part smaller.

### What D-41's capacitor will cost in size and weight

#### Scaling law used (unchanged from version 0.2)

1. **Size follows the energy at the *rated* voltage, not the working voltage.** A film capacitor's size is set by how hard its plastic film is stressed. At a fixed stress, the stored energy per cm³ at the rated voltage is roughly the same whatever the rating. So, at the same quality of film:

   volume ∝ ½ × C × (rated voltage)²

   For 100 µF: ½ × 100 µF × 2,200² = **242 J** "rated energy" [CALC], against 200 J at 2.0 kV. A 2.2 kV part is therefore about **21% larger** than a 2.0 kV part of the same capacitance and film quality [CALC: (2.2/2.0)² = 1.21].

2. **Lifetime trades against density.** GA's published family shows it directly. Each doubling of cycle life costs about a fifth of the energy density:

   | GA part | Rated | Density | Cycles |
   |---|---|---:|---:|
   | 320DQ4130 | 2.0 kV | 1.66 J/cc | 2,500 |
   | 323DQ4150A | 2.3 kV | 1.35 J/cc | 5,000 |
   | 322DQ4196B | 2.2 kV | 1.20 J/cc | 5,000 |
   | 323DQ4115B | 2.3 kV | 1.04 J/cc | 10,000 |
   | 30920 | 2.2 kV | 1.05 J/cc | 10,000 |

   Energy density goes as the square of the film stress. Fitting the three 2.0–2.3 kV steps gives **life ∝ (stress)^−5.3 to −6.7**, about **−6.6** for the first step [CALC: ln(5,000/2,500) / ln(√(1.66/1.35)) = 6.7; the 1.35 → 1.04 step gives 5.3]. Published guidance for film capacitor life uses exponents of 5–10 (Digi-Key technical forum), and one pulse-capacitor study measured 13. This study uses **6.6**.

3. **Running below the rating buys life.** At 1,950 V a 2,200 V part is stressed 11% less than at its rating, so it lasts about **2.2 times** its rated cycle count [CALC: (2,200/1,950)^6.6 = 2.22]. At the 2,008 V worst case the gain is 1.8.

4. **Heat (revised for version 0.3).** Two separate things, which version 0.2 kept apart in its working but merged in its lever table:
   - **Storage, with no voltage on the part.** Polypropylene film capacitors are commonly rated to 85 °C or 105 °C, with voltage derating of about 1.0–1.25% per °C only *above* 85 °C (Panasonic and other film datasheets, checked 27 Nov 2026). Unpowered storage at 70 °C is comfortably inside the film's ordinary range; TDK's and CRE's standard defibrillator parts are already rated for it. **Cost in density of 70 °C storage: none [EST]. Cost of 85 °C storage in version 0.2: "small" [EST, unchanged].** So moving from 85 °C to 70 °C storage changes the estimate by about nothing.
   - **Working temperature, with voltage on the part.** This is what costs density. The defibrillator parts found are rated to 55 °C working (TDK) or 60 °C (CRE 125 µF). A device at 50 °C outside warms by about 4 K in a long rescue (§4), so Appendix A asks for 65 °C working to keep about 10 K of margin. Film capacitor life is generally taken to halve for every 10 °C rise (Digi-Key technical forum) [EST that this holds for pulse duty]. Recovering that life costs density [CALC, using the exponent 6.6]:

     | Working rating | Life lost against 55 °C | Film stress to recover it | Density, from GA's 1.66 J/cc |
     |---|---:|---:|---:|
     | 55 °C | none | — | 1.66 J/cc |
     | 60 °C | ×0.71 (2^−0.5) | ×0.949 (2^(−0.5/6.6)) | **1.49 J/cc** (×0.90) |
     | 65 °C | ×0.5 | ×0.900 (2^(−1/6.6)) | **1.35 J/cc** (×0.81) |

5. **Box versus active volume.** GA's quoted J/cc is 15–18% higher than its parts' outside-box figures [CALC: e.g. 320DQ4130 stores 260 J rated in a 179.7 cm³ box = 1.45 J/cc, against 1.66 quoted]. The estimates below add **18%** for the can.

6. **Mass.** Only one defibrillator film capacitor has a published mass (CRE, 300 g for a 250 J part). This study uses **1.3–1.5 g/cm³** [EST: polypropylene film is about 0.9 g/cm³; an aluminium can and resin fill raise it], and checks against CRE's 0.83 J per gram [CALC: 250 J / 300 g].

#### Version 0.3 estimate: 70 °C storage, 65 °C working, 5,000 charges [CALC from the scaling above; EST inputs as marked]

Kept for comparison; superseded by the version 0.4 estimate below.

| | Low | Expected | High |
|---|---|---|---|
| Basis | GA's densest film (1.66 J/cc) meets Appendix A with no temperature penalty, because the maker's standard film already holds 65 °C working (as CRE claims 70 °C working for its 2.2 kV part) | GA's densest film, less 19% for 65 °C working: **1.35 J/cc**. This happens to equal GA's real 2.3 kV, 5,000-cycle grade | GA's real 2.2 kV, 10,000-cycle grade (**1.05 J/cc**), in case the hot-working life needs it |
| Active volume (242 J rated ÷ density) | 146 cm³ | 180 cm³ | 230 cm³ |
| **Boxed volume** (+18%) | **about 170 cm³** | **about 210 cm³** | **about 270 cm³** |
| **Mass** | 1.3 g/cm³: **about 225 g** | 1.4 g/cm³: **about 300 g** (CRE's 0.83 J/g gives 290 g) | 1.5 g/cm³: **about 410 g** |
| Change from version 0.2 (85 °C storage) | none | none | none |
| Life at 1,950 V | about 5,500 cycles at 25 °C [CALC: 2,500 × 2.22], less when hot | about 5,500 cycles at the working limit [CALC] | over 10,000 |

**What changed from 85 °C, plainly.** The numbers did not move, because the method never charged anything for storage heat. What changed is the field of suppliers (above), the qualification test (1,000 h at 70 °C instead of 85 °C, Appendix A), and the price (below). Version 0.2's "about 60 g" saving for a 70 °C part (its lever 7) assumed the working rating would also fall to TDK's 55 °C. That saving was available only by changing the **working** rating; the owner has since set 60 °C working and 2,500 charges (D-41), worked out below.

#### The version 0.3 estimate with a lower working rating, 5,000 charges [CALC]

Kept from version 0.3, where it was shown so that the working temperature could be chosen. These rows assume 5,000 charges and are **superseded** by the decided case below.

| Working rating | Expected capacitor | Device with pads, lower-risk set (§9) | What it costs in use |
|---|---|---|---|
| 65 °C (version 0.3's Appendix A) | 210 cm³ / 300 g (1.35 J/cc) | 0.89 L / 903 g | Nothing: about 10 K of margin at a 50 °C ambient |
| 60 °C | **190 cm³ / 270 g** (1.49 J/cc: 242 ÷ 1.49 × 1.18 = 191 cm³; × 1.4 = 268 g) | **0.85 L / 868 g** (−35 g) | About 6 K of margin at a 50 °C ambient. A device that is already hot (for example just out of a hot car) waits to cool before charging, per question M |
| 55 °C (TDK class) | **170 cm³ / 225 g** (1.66 J/cc; the same as the low case) | **0.80 L / 817 g** (−86 g) | About 1 K of margin at a 50 °C ambient: in a long rescue on a very hot day the device would sometimes have to pause charging to cool. Equivalent to lowering the operating target from 50 °C to about 45 °C [CALC: 55 − 4 K rise − about 5 K margin] |

#### Version 0.4: the decided capacitor (60 °C working and 2,500 charges together) [CALC; EST inputs as marked]

D-41 and D-53 set both changes at once. Version 0.3 showed each on its own (190 cm³ / 270 g for 60 °C working; 170 cm³ / 240 g for 2,500 charges) and warned that they do not simply add. Here is the combined case, by the same method.

**How the two interact.** Both act through the same thing: how hard the maker may stress the plastic film. A hotter working limit uses up life, so the film must be stressed *less*; a shorter required life frees life, so the film may be stressed *more*. With life ∝ stress^−6.6, and density ∝ stress², each factor of two in life is worth a factor of 2^(2/6.6) = 1.23 in density. The two effects therefore **multiply**: they are not two separate savings to add up.

**The arithmetic, two ways.**

| Step | Version 0.3 shortcut | Full method |
|---|---|---|
| Starting point | GA's densest film, 1.66 J/cc, taken as good at 55 °C | GA 320DQ4130: 1.66 J/cc, 2,500 cycles at its rated voltage, about **5,550 cycles at 1,950 V** [CALC: 2,500 × 2.22] |
| 60 °C instead of 55 °C | density × 0.90 → 1.49 J/cc | life × 0.71 → about **3,900 cycles at 1,950 V, 60 °C** [CALC: 5,550 × 2^−0.5 = 3,924] |
| 2,500 required instead of 5,000 | density × 1.23 | the part at 1.66 J/cc already has 1.57 × the 2,500 needed [CALC: 3,924 ÷ 2,500] |
| Density the arithmetic would allow | **1.84 J/cc** [CALC: 1.49 × 1.23] | **1.90 J/cc** [CALC: 1.66 × 1.57^(1/3.3)] |

**Why the study does not count on 1.84–1.90 J/cc: there is a ceiling set by other things.**
1. **No maker publishes it.** The densest defibrillator film part on a catalogue page is GA's 1.66 J/cc, and that is a 2.0 kV, 2,500-cycle part. CDE's press material says "as high as 2 J/cc" but gives no part, voltage or life [UNVERIFIED]. Nothing at 2.2 kV is published above 1.20 J/cc (§1 table).
2. **The voltage rating itself sets a floor on film thickness.** A 2,200 V part must pass 2,420 V on every unit (A1) and must keep enough self-healing margin to clear small faults at that voltage, at 60 °C, without the clearing spreading. A shorter required life does not relax that; it only relaxes slow wear-out. How close GA's 1.66 J/cc already sits to that floor is not published [UNVERIFIED].
3. **Self-healing and fault survival need margin.** The fault case in A1 (about 400 A into a short) and the "fail open, never short" rule (A4) both get harder as the film is pushed thinner [BK].
4. **The life model is fitted to one maker's family** (exponent 6.6; published values run 5 to 13). With an exponent of 10, the same arithmetic gives only about 1.81 J/cc [CALC: 1.66 × 1.57^(2/10)].

So the **expected** case is capped at the densest real part, 1.66 J/cc. The arithmetic says the requirement *could* allow more, but that is left as the **low** case, for makers to prove.

| | Low | Expected | High |
|---|---|---|---|
| Basis | About **2.0 J/cc**: what the arithmetic allows (1.84–1.90) and CDE's press claim, if a maker can show it at 2.2 kV | **1.66 J/cc**: GA's densest real part; meets 2,500 charges at 60 °C with about 57% spare life on paper | **1.35 J/cc**: makers decline to trade life for density and quote their standard 5,000-cycle grade (GA's real 2.3 kV part) |
| Active volume (242 J rated ÷ density) | 121 cm³ | 146 cm³ | 179 cm³ |
| **Boxed volume** (+18%) | **about 145 cm³** | **about 170 cm³** | **about 210 cm³** |
| **Mass** | 1.3 g/cm³: **about 185 g** | 1.4 g/cm³: **about 240 g** | 1.5 g/cm³: **about 315 g** |
| Change from version 0.3 (65 °C working, 5,000 charges) | −25 cm³ / −40 g | **−40 cm³ / −60 g** | −60 cm³ / −95 g |
| Life at 1,950 V, 60 °C | at least 2,500 (by construction) | about 3,900 [CALC] | about 5,500 or more |

**How sure.**
- **Volume, expected case:** moderate. It is a real part's density at 2.2 kV rating energy.
- **Low case:** low. No published part supports it.
- **Mass:** low, as before (one maker's figure).
- **The combined saving of about 71 g** on the finished device is the 2,500-charge saving alone. Adding 60 °C working on top buys nothing more in the expected case, because the cap is already reached. What it buys is spare life: about 3,900 charges against 2,500.

**Shape.** A box of about 33 × 55 × 95 mm holds the expected 170 cm³ [CALC: 172 cm³]: the footprint of the GA 320DQ4130, a little shorter.

**Price [EST].** Version 0.1 estimated $150–400 for a sample and $80–200 at qty 100. Version 0.2 added 20–50% for the 2.2 kV rating and an 85 °C test. With 70 °C storage inside two makers' standard ratings and a smaller part, perhaps 15–40% over version 0.1 [EST, from custom-part norms]: **$180–550 for samples, $95–230 at qty 100**, plus a one-off qualification charge of perhaps $800–2,500 [EST]. The shorter life test (2,500 rather than 5,000 cycles) should shorten qualification [EST].

### Temperature

The capacitor is no longer the part that decides how hot the device may get. That is set by the pads and the battery; all the heat limits, the parked-car data and the proposed envelope are in **§8A**.

- **What heat does to electrolytics.** Kept from version 0.1: only the 105 °C electrolytics have a rating above 70 °C, and they are the heaviest and least suited to pulse duty. Not recommended.

### Operating heat inside the device

- At a 50 °C ambient, a long no-shock rescue warms the inside of the box by about 4 K (§4), more near the discharge resistors.
- Appendix A asks for **60 °C working** (D-41). That gives about 6 K of margin over the 54–55 °C the capacitor would reach. A device that starts hotter (for example just out of a hot car) guides CPR and analyses, and charges once the capacitor is back at or below 60 °C (D-51).

### Cycle life over 10 years under D-15 [CALC]

Usage profile, stated as an assumption [EST]:

| Use | Typical user | Heavy user | Working |
|---|---:|---:|---|
| Weekly full-energy self-test (D-15) | 520 | 520 | 52 a year × 10 years |
| Rescues | 6 a year, 20 minutes | 25 a year, 30 minutes | assumption; a pool unit shared across shifts is the heavy case |
| Charges per rescue (one every ~2.2 min, D-29) | 10 | 14 | 20 min ÷ 2.2 min = 9.1 → 10; 30 min ÷ 2.2 min = 13.6 → 14 |
| Rescue charges over 10 years | 600 | 3,500 | 6 × 10 × 10; 25 × 14 × 10 |
| Factory, acceptance and service tests | 100 | 100 | [EST] |
| **Full-energy cycles over 10 years** | **1,220** | **4,120** | sum |
| Daily low-voltage checks (D-15) | 3,650 | 3,650 | 365 × 10 |
| Wear from the low-voltage checks, in full-cycle equivalents | about 0 | about 0 | at 10% of full voltage, one check is 0.1^6.6 ≈ 0.0000003 of a full cycle. Even at half voltage it would be 0.01 each, about 37 in total |
| Time held charged (up to about 30 s per charge) | about 10 h | about 34 h | 1,220 × 30 s; 4,120 × 30 s |

- **Requirement (D-41, D-53):** at least **2,500 full-energy cycles at 1,950 V and 50 °C ambient**, with the part at up to 60 °C [CALC: typical profile 1,220 × 2 = 2,440, rounded up], and about 60 h of cumulative holding at 2,008 V [CALC: 2,500 × 30 s = 21 h, × 3]. The expected-case part meets it with about 57% spare on paper.
- **A heavy user** (4,120 in ten years, 412 a year) reaches 2,500 in about 6 years [CALC]. The device counts full charges and calls for service before then (D-53). A **heavy-duty capacitor** (5,000 charges at 60 °C, about 190 cm³ / 270 g, +36 g on the device) is a later option for busy services (Appendix A7).
- Version 0.3's requirement was 5,000 cycles; the change to 2,500 is what saves about 71 g (§9).
- **For comparison:** version 0.1's twice-daily full-energy policy would have used about 7,300 cycles in 10 years on self-tests alone [CALC: 730 × 10], forcing the high, 10,000-cycle case. D-15 is what keeps the capacitor at the expected size.

### Self-discharge, recovery and failure

- **Film** holds charge well and needs no re-forming. It recovers a little voltage after being emptied: TDK warns of "self-recharging phenomena". It usually fails gently, by self-healing and slow capacitance loss [BK]. Appendix A asks the maker to *prove* that.
- **Electrolytics** leak more, recover more charge after emptying, can fail short or vent, and need balancing in series.

### Recommendation

- **Order a single film capacitor to the specification in Appendix A** from defibrillator-capacitor makers: GA, CDE/Knowles, TDK, and CRE as a low-cost check. Ask each for a quote against the same sheet (60 °C working, 2,500 charges), with the heavy-duty variant of A7 as an option, and for their density, mass and life model, so the low case can be confirmed or dropped.
- **Budget 170 cm³ and 240 g** (the expected case), and carry the low and high cases in the roll-up.
- Keep the 105 °C electrolytic bank only as a heat fallback, not for size.

### What must be tested to confirm it

Later, under the lab safety policy. Appendix A lists the proof to ask the maker for; the project's own checks would be:
- Capacitance, leakage and internal loss before and after 1,000 h at **70 °C**, unpowered.
- 1,000 full-energy cycles at 50 °C ambient with the part at 60 °C, measuring capacitance loss and any rise in internal loss.
- Voltage recovered 1 minute after a full dump.
- Mass and dimensions of the delivered part.

---

## 2. Capacitor charger

### Power needed [CALC]

| Step | Working | Result |
|---|---|---|
| Energy to deliver | 190.1 J in 7 s | **27.2 W out** (FINDINGS.md gives 27 W) |
| Design margin for a cold or tired battery | ×1.3 [EST] | **35 W out** |
| Efficiency of a 2 kV flyback | 70–80% [EST, typical small HV flyback] | **44–50 W in** |
| Loss as heat per full charge | 190 J × (1/0.75 − 1) | **about 63 J per charge** |
| Battery current, **2 cells in series** (D-44), about 5.8 V under load when cold | 44 W / 5.8 V | **about 7.6 A average, about 15 A peaks** [EST ×2 for flyback peaks] |
| For comparison: 1 cell, about 2.9 V | 44 W / 2.9 V | about 15 A average, about 30 A peaks |

### Candidate controller chips

| Part | What it is | Fit | Status, checked 27 Nov 2026 |
|---|---|---|---|
| ADI LT3751 | Flyback capacitor-charger controller; regulates the output voltage itself | Built for this job; maker designs stop at about 450 V, so 2 kV depends on the transformer | Lifecycle [UNVERIFIED]; about $8–12 [EST] |
| ADI LT3750 | Simpler capacitor-charger controller; voltage set by turns ratio | Good for a fixed adult voltage; child mode needs separate regulation | [UNVERIFIED] |
| General-purpose flyback controller (e.g. TI LM5155/UCC28C4x class) | Flexible; needs the voltage loop built around it | More work, more control | [BK] |

A 2-cell input (6–8.4 V) sits comfortably inside these controllers' input ranges [BK].

### The transformer (made to order)

- No catalogue transformer reaches about 2 kV for this job; Würth's stock part for the LT3751 is 300 V.
- It is made to order under D-42. **Its requirement list is in Appendix B.** Firms that advertise custom high-voltage transformers with prototypes: Custom Coils, Agile Magnetics, Pulse Electronics, Bourns. One unnamed maker lists a flyback for portable defibrillators [UNVERIFIED maker].
- Typical cost [EST, from custom-magnetics norms, not quotes]: design and tooling $500–2,500; prototype samples $50–150 each; about $10–25 at qty 100.
- Typical size [EST]: a 35–50 W part is about 10–20 cm³ and 15–35 g, more with insulation for 2.2 kV.
- A high-voltage output diode rated about 3 kV or more, fast recovery, is a catalogue part [BK].

### Alternative

Charging several lower-voltage capacitors side by side and reconnecting them end to end avoids the 2 kV transformer but needs many more switches, and Avive holds patents on it. Not recommended before the patent review.

### Heat

About 63 J per charge [CALC], mostly in the transformer and switch. Over a 30-minute rescue that is 0.9 kJ, folded into section 4.

### Recommendation

- Controller: LT3751-class.
- Transformer: made to order against Appendix B.
- Battery: 2 cells in series (D-44, §5).

### Test

- Charge time to 1,950 V into a 100 µF load at −20, +25 and +50 °C, from a battery at 10% charge.
- Efficiency, and transformer temperature after 14 charges at 2.2-minute spacing.

---

## 3. Output bridge and patient isolation

Unchanged from version 0.1 except that the gate-drive isolation has a requirement list (Appendix B).

### What it must do

- Block 1,950 V (2,008 V worst case) in each switch position.
- Carry 65 A peak, falling over 3–15 ms.
- Switch off on command to end each phase.

### Candidates

| | A. Four 3.3 kV SiC MOSFETs: Microchip MSC080SMA330B4N | B. Four 3.3 kV SiC in surface-mount: GeneSiC G2R120MT33J | C. Thyristors plus IGBTs (the expired Physio-Control approach): 3 × Littelfuse CS20-22moF1 + 1–2 × IXEL40N400 |
|---|---|---|---|
| Blocking voltage | 3,300 V: **69% margin** | 3,300 V: 69% | Thyristor 2,200 V: **13% margin, 10% at 2,008 V (thin)**. IGBT 4,000 V: 105% |
| Pulse current (datasheet) | IDM 107 A, "repetitive, pulse width … limited by maximum junction temperature". **No pulse width given.** 1.65× the 65 A need | body-diode pulse 116 A; drain pulse rating **not retrieved** [UNVERIFIED] | Thyristor surge ITSM 200 A (normally a 10 ms half-sine [BK]). IGBT ICM 250 A at 1 ms (2009 sheet) or 170 A (2006 sheet) — **the sheets disagree** |
| Heating per shock [CALC] | At 65 A, hot on-resistance about 0.15 Ω [EST]: 634 W peak; junction rise about 30–60 °C over a few ms [EST]. **Plausible but unproven** | similar | Thyristors are built for this |
| Package, size | TO-247-4, about 6 g each [BK] | TO-263-7 surface-mount, about 2 g [BK] | ISOPLUS i4-PAC / i5-Pak, about TO-247 size |
| Count | 4 switches, 4 isolated gate drives | 4 / 4 | 3 thyristors + 1–2 IGBTs |
| Price, qty 1 / 100 [EST] | about $60–90 / $45–70 each | about $40–70 / $30–50 | thyristor about $15–25, IGBT about $40–60 |
| Main risk | Pulse rating at milliseconds not stated | Pulse rating not retrieved | Thyristor voltage margin thin |

**Flag.** None of these datasheets states a pulse current rating at the 3–15 ms pulse length a shock uses. That must be measured, or asked of the makers.

### Gate drive at 2,000 V working

**No catalogue part found qualifies.** TI's UCC21750 isolated driver is rated 2,121 V DC working, so it would run at 92–95% of rating. Murata's MGJ gate supplies are 800 V rms reinforced (MGJ2) or 3 kV DC functional only (MGJ6). Options: arrange the bridge so that no driver's barrier sees the full voltage; made-to-order pulse transformers carrying signal and power (the expired Physio-Control prior art); or fibre-optic signal links plus a separate gate-power solution. **The requirement list for whichever is chosen is in Appendix B.**

### Isolation relays

- Sensata/Cynergy3 D-HR vacuum reed relays: 10–15 kV stand-off, 2–4 A carry. They suit disconnecting the heart-signal front end during the shock.
- No relay was found with a published pulse rating for carrying the 65 A shock. Rely on two switches in series in each current path as patient isolation.

### Size [EST]

Four switches, drivers, gate supplies and a protective inductor-resistor: **25–40 cm³ and 40–60 g**, before the spacing the standard requires.

### Recommendation and test

- Option A (SiC), gate drive per Appendix B, and ask Microchip for a pulse rating at 10 ms.
- Test later: single 10 ms pulses of 65 A into a 25 Ω test load, first at low voltage; then 20 pulses in 1,300 s at full voltage.

---

## 4. Internal discharge load: what charging during analysis costs

### Duty in one rescue

- Every no-shock analysis burns the full **190 J** (D-29).
- Every shock leaves **up to 36 J** to dump.
- Every weekly self-test burns 190 J (D-15).

### Worst-case rescue, stated

- 30 minutes of a non-shockable rhythm, analysis every 132 s: **14 full charges burned** [CALC: 1,800 / 132 = 13.6, round up].

| Quantity | Working [CALC] | Result |
|---|---|---|
| Energy into the load | 14 × 190.1 J | **2.66 kJ** |
| Charger loss inside the box | 14 × 63 J | 0.89 kJ |
| Electronics and audio | 0.5 W × 1,800 s [EST] | 0.9 kJ |
| Total heat released inside | | **about 4.4 kJ (average 2.5 W)** |
| Battery energy for the charges | 14 × 253 J = 3.55 kJ | **0.99 Wh** (5% of a 2 × 18650 pack) |

### Box temperature [CALC]

- Box about 150 × 86 × 50 mm: surface 494 cm²; about 10 W/m²·K to still air [BK]: 0.49 W/K.
- Steady rise 2.5 W / 0.49 = 5.1 K; time constant about 21 minutes (0.7 kg × 900 J/kg·K = 630 J/K).
- **After 30 minutes the inside is about 3.8 K above ambient**: about 54 °C at a 50 °C ambient. Appendix A's 60 °C working rating (D-41) covers this with about 6 K margin; above it the device pauses charging (D-51). A heavier device (§9) warms slightly more slowly.
- The same box constant governs how fast the device heats in a parked car (§8A): at about 0.8 kg without pads it is about 25 minutes [CALC: 0.8 kg × 900 J/kg·K ÷ 0.49 W/K = 1,470 s].

### Resistor temperature [CALC]

Three 400 J ceramic resistors in series, about 7 g each [EST]: about 11 K rise per dump, about 84 °C peak at a 50 °C ambient. Fine for ceramic; keep it away from the capacitor and the battery.

### Lifetime pulse count under D-15 [CALC]

| | Typical user | Heavy user |
|---|---:|---:|
| Weekly self-test dumps of 190 J, 10 years | 520 | 520 |
| Rescue dumps (assume every rescue charge is dumped, the worst case) | 600 | 3,500 |
| Factory and service | 100 | 100 |
| **Full-energy dumps over 10 years** | **1,220** | **4,120** |
| Daily low-voltage check dumps (about 2 J each if charged to 10% voltage [CALC: ½ × 100 µF × 200²]) | 3,650 | 3,650 |

**Requirement for the load:** at least 5,000 pulses of 190 J over life, including runs of 14 pulses at 132 s spacing, with resistance change within the maker's stated limit.

### Candidates

| | A. Ohmite EY ceramic composition, 3 in series | B. Ohmite 885SP bulk ceramic tube, 3 in series | C. Non-inductive wirewound (e.g. Riedon pulse wirewound) |
|---|---|---|---|
| Single-pulse energy | 400 J each (1,200 J total): **6× the 190 J** | 250 J each (750 J): 4× | > 1,000 J (Riedon) |
| Voltage | not found [UNVERIFIED]; 650 V each in a string of three | 1,000 V peak each: 3,000 V string | depends on part |
| Repetition | **Made "for single or infrequent pulse events"**: one pulse every 2 minutes and 5,000 over life must be confirmed with Ohmite [UNVERIFIED] | 45 W average at 40 °C: ample | depends on part |
| Size, mass | **about 10 cm³ / 21 g total** [EST mass] | **54 cm³ / 75 g** [EST mass] | about 10–30 cm³ [EST] |
| Price [EST] | about $5–10 each | about $10–20 each | various |
| Main risk | Repeat-pulse rating | Bulk | Coil inductance makes the dump ring |

### Recommendation

Option A if Ohmite confirms repetition and the 5,000-pulse life; option B as the fallback, at about 45 cm³ and 55 g more.

### Test

Resistor and capacitor case temperature after 14 dumps of 190 J at 132 s spacing in a sealed box at 50 °C ambient.

---

## 5. Battery

### Two cells, in series (D-44)

- **Series, not parallel.** Both arrangements put the same current through each cell, but series doubles the pack voltage (6–8.4 V), which **halves the current** in the protection switches, fuse, wiring and charger input. Losses there scale with the current squared, so they fall to about a quarter [CALC]. The LT3751-class charger controllers accept this input range [BK].
- What series costs: the two cells must be kept balanced, the gauge must watch each cell, and the dock's 5 V must be stepped *up* to charge an 8.4 V pack.
- D-44 was decided with "about 20 g heavier" in mind, which is **two 18650 cells** (2 × 46 g) against one 21700 (about 70 g).

### Mission energy [CALC/EST]

A 12-hour shift plus one 30-minute rescue with 14 charges: standby 0.02 Wh, rescue electronics 0.25 Wh, 14 charges 0.99 Wh: **about 1.3 Wh**. Energy is not the limit; current in the cold is.

### Cold-weather charging arithmetic at −20 °C [CALC]

The question: can the battery deliver the charger's 44–50 W at −20 °C from a nearly flat pack, without its voltage collapsing below the cells' 2.5 V cut-off?

Assumptions:
- Cell open-circuit voltage at about 10% charge: **3.4 V** [EST, typical lithium-ion curve].
- Room-temperature internal resistance: Molicel P28A about 15 mΩ typical (its datasheet copy gives 20 mΩ maximum, DC 10 A / 1 s); P45B about 10 mΩ [EST].
- At −20 °C internal resistance rises about **3 to 5 times** [BK]. The P28A datasheet copy gives at least 2.0 Ah of its 2.8 Ah at −20 °C, at an unstated current.
- Each cell must supply its share of power P. The current then follows from P = V × I − I² × R.

| Pack | Power per cell | Cold resistance per cell | Current per cell | Cell voltage under load | Verdict |
|---|---:|---:|---:|---:|---|
| 1 × P45B (one cell, for comparison) | 50 W | 30 mΩ | 17.4 A | 2.88 V | just works |
| 1 × P45B | 50 W | 50 mΩ | 21.5 A | **2.32 V** | **fails: below 2.5 V** |
| **2 × P28A in series** | 25 W | 45 mΩ | 8.3 A | 3.03 V | works |
| **2 × P28A in series** | 25 W | 75 mΩ | 9.2 A | **2.71 V** | **just works: 0.2 V above cut-off** |
| 2 × P45B in series | 25 W | 30 mΩ | 7.9 A | 3.16 V | works |
| 2 × P45B in series | 25 W | 50 mΩ | 8.4 A | 2.98 V | works with margin |

**Plainly:**
- One cell cannot be relied on for a full-speed charge at −20 °C when nearly flat. That confirms D-44.
- Two 18650s **just** manage it in the pessimistic case. Two 21700s manage it comfortably but weigh about 48 g more than two 18650s (2 × 70 g against 2 × 46 g).
- If cold tests show the 18650s sag too far, the firmware can slow the charge when the battery is cold and low, so the shock follows a few seconds after the analysis. That is a fallback, not a plan (Q-39).
- Charging the *battery itself* is a different matter: the cells must not be charged below 0 °C (P28A rating: charge 0–60 °C). A device docked in a freezing car will not recharge until it warms.

### Heat limits of the cells (new in version 0.3; detail in §8A)

- **Molicel P28A** (maker's specification, third-party copy): charge 0–60 °C, discharge −40 to 60 °C; storage for shipping **below 35 °C**, and **below 23 °C recommended** for long-term storage; shelf life 1 year. **No short-term storage maximum is published in the copy found.**
- **Class reference, Samsung INR18650-25R** (maker's specification, 2014, third-party copy): keeps at least 90% of its capacity after **1 month at up to 60 °C**, 3 months at up to 45 °C, or 1.5 years at up to 25 °C.
- So for this study's cells, **60 °C is the top of what any maker of this class of cell publishes**, and only as a one-month allowance. Between 60 and 70 °C the cells are outside every rating found.
- Heat-tolerant alternatives exist but cost weight or current (§8A, option C).

### Charges per battery charge [CALC]

2 × P28A: 2 × 2.8 Ah × 3.6 V = 20.2 Wh. New, room temperature: 20.2 × 0.9 / 0.0703 Wh per charge ≈ **259 charges**. Cold and aged (about 60% usable): ≈ **172**. Far above the 30-shock requirement.

### Protection and fuel gauge for a 2-cell pack

A 2-cell pack needs: a protector that can cut the pack off for over-voltage, under-voltage, over-current, short circuit and temperature on *each* cell; a fuel gauge that the self-test can trust under load; and cell balancing. Candidates, checked 27 Nov 2026:

| Part | What it is | Status |
|---|---|---|
| **ADI MAX17320** | 2-to-4-cell fuel gauge with protector, programmable thresholds (including over- and under-temperature), internal self-discharge detection, SHA-256 battery authentication | **Recommended for new designs** (ADI page). Temperature range not re-checked [BK: −40 to 85 °C] |
| TI BQ28Z610 / BQ28Z620 | 1-to-2-cell gauge with protection and cell balancing; the BQ28Z620 is TI's newer version | BQ28Z620 **ACTIVE**; TI flags a newer version of BQ28Z610 |
| TI BQ25887 (charger) | 2-cell boost charger from a 5 V input, 2 A, integrated cell balancing, battery thermistor with JEITA temperature rules, −40 to 85 °C | Listed on TI's site; stock not confirmed |

Implications of 2 cells:
- The protector's switches must carry about 7.6 A average and 15 A peaks while charging the capacitor, with low loss [CALC, §2]. Half the single-cell current.
- The gauge's per-cell readings let the self-test catch **one weak cell**, which a single pack voltage would hide. That serves the "battery tested under load" part of D-15.
- The MAX17320's self-discharge detection and authentication would let the device refuse a counterfeit or damaged pack. The same self-discharge detection is the battery health check that §8A requires after an over-temperature event.
- The pack (and the cells) need UN 38.3 transport testing; a device shipped with its battery inside goes as UN 3481 [BK].
- High-drain cells are typically rated about 500 cycles to 80% capacity [BK]. Charged daily, plan on battery replacement every 2–3 years; time spent hot shortens that.

### Charging through sealed dock contacts (D-46)

- **How it works, in requirement terms.** The dock takes ordinary USB-C power and presents it to the device through two or three exposed contacts (power, return, and optionally a "docked" sense). The device steps it up to charge the pack. **With no Bluetooth (D-09), the dock also carries the data**: event records, the heat history and the charge count come off through it by cable.
- **Parts.** Sealed spring-contact connectors are catalogue items: Glenair's CB series pogo-pin connectors (its test report found no water inside an unmated panel receptacle), ATTEND Technology's IP67 waterproof pogo connectors, and Yokowo's IP67/IP68/IP69K range. One magnetic pogo connector datasheet warns that the *mated* joint is not waterproof; that does not matter here, because charging happens in the dock, not in the rain.
- **Which side gets the springs.** Putting the springs in the dock and plain flat contact pads on the device leaves the device with no moving parts and no holes, which is the easier way to hold IP65 [EST].
- **Requirements on the device side:**
  - The contacts must be **dead** when the device is off the dock: no battery voltage on them, so a wet glove or a key across them shorts nothing.
  - Withstand reversed or excess dock voltage, and static discharge, without damage.
  - Corrosion-resistant plating; sweat and road salt are the realistic exposure [EST].
- **Size and mass, device side [EST]:** about 2 cm³ and 5 g including alignment magnets and input protection, against about 1 cm³ and 2 g for a USB-C socket and its seal. **D-46 costs about 1 cm³ and 3 g**, plus a dock that is not carried (about $50–150 for prototypes, $15–30 at qty 100 [EST]).
- **A dock in a vehicle** will be as hot as the cabin; the battery-charging temperature rule in §8A applies there too.

### Data through the dock contacts (D-09, new in version 0.4)

Requirements only; how the link works is for the engineer.
- **One or two extra contacts** beside the charging contacts. One contact suffices if the dock does the conversion to USB for a computer (the dock is the reader); two if the device speaks USB itself. That choice is question P (§11).
- The data contacts follow the same rules as the charging contacts: flat pads on the device, springs in the dock, **dead when undocked**, sealed to IP65, protected against static discharge, reversed or excess voltage, and a short to the power contact [EST].
- **What goes over it:** reading out event records, the heat history (D-48, D-52) and the charge count (D-53); setting the clock; loading configuration. **Nothing over it can cause a charge or a shock**; configuration is checked by the main controller before use, as version 0.3 required of the Bluetooth module.
- Records are read on the dock, so a device does not need to come off a vehicle charger to be read if that charger is a dock.
- **Size and mass [EST]:** about 1 cm³ and 1 g on the device for one or two more contact pads and their protection. A dock that reads data costs perhaps $10–20 more than a charge-only dock [EST].

### Candidates

| | A. 1 × Molicel P45B 21700 | **B. 2 × Molicel P28A 18650 in series** | C. 2 × P45B 21700 in series | D. 2-cell high-rate pouch pack |
|---|---|---|---|---|
| Fits D-44 | **no** | **yes** | yes | yes |
| Energy | 16.2 Wh | 20.2 Wh [CALC: 2 × 2.8 Ah × 3.6 V] | 32.4 Wh | about 11–16 Wh [EST] |
| Current per cell, 44–50 W charge, cold | 15–21 A | 8–9 A | 8 A | 8–9 A |
| −20 °C, 10% charge (table above) | fails in the pessimistic case | **just works** | works with margin | unknown |
| Mass of cells | 70 g | **92 g** (46 g typical each, datasheet copy) | 140 g | about 60–90 g [EST] |
| Pack with gauge, protection, charger, holder | about 75 g / 28 cm³ | **about 97 g / 38 cm³** | about 148 g / 56 cm³ [EST] | about 70–100 g [EST] |
| Storage heat | as B [UNVERIFIED for P45B] | below 35 °C shipping; no short-term maximum published; class figure 60 °C for 1 month | as B | pouch cells swell more readily when hot [BK] |
| Price, cells [EST] | about $6–8 | about $10 / $7 for two | about $12–16 | about $15–30 |
| Main risk | Voltage collapse at −20 °C | Thin cold margin | Mass | Quality and traceability |

### Recommendation

**Option B (2 × P28A-class 18650 in series)** for both sets, with **MAX17320**-class protection and gauge and a **BQ25887**-class charger, charged through sealed dock contacts. Option C is the fallback if cold tests fail, at about 50 g and 18 cm³ more (Q-39). Ask Molicel for the P28A's short-term storage rating (§8A).

### Test

Charge time to 1,950 V and lowest cell voltage, from 10% charge, at −20 °C after a 4-hour cold soak, for packs B and C. Capacity, internal resistance and thickness before and after 100 h at 60 °C at full charge, unpowered.

---

## 6. Heart-signal and pad-resistance front end

Unchanged from version 0.1.

### What it must do

- Read the heart signal through the shock pads.
- Measure pad resistance from 25 to 250 Ω (D-32). The **compression signal is a change of a few tenths of an ohm at 1–3 Hz** (D-36).
- Survive the device's own 2,000 V shock and recover quickly.
- Work while the charger runs (D-29).

### Resolution needed [CALC]

0.3 Ω on 250 Ω is 0.12% of the baseline. With a 48 µA measuring current that is a 14 µV change, against the MAX30001's quoted 1.1 µV p-p noise: a signal-to-noise ratio of about 13. **Adequate**, if the protection parts do not swamp it.

### Candidates

Status checked 27 Nov 2026.

| | A. TI ADS1292R | B. ADI MAX30001 / MAX30001G | C. ADI MAX30009 (impedance) + a separate ECG chip | D. ADI ADAS1000 |
|---|---|---|---|---|
| What it does | 2-channel 24-bit ECG with built-in respiration impedance | ECG + bio-impedance channel | Impedance only, very flexible | 5-electrode ECG + respiration |
| Status | **ACTIVE** | **PRODUCTION** (MAX30001G) | **PRODUCTION** | [UNVERIFIED] |
| Temperature | **−40 to +85 °C** | **MAX30001G: 0 to +70 °C — fails the −20 °C target** | [UNVERIFIED] | [UNVERIFIED] |
| Noise | 8 µVpp (150 Hz bandwidth) | 1.1 µVpp (BioZ) | "1.1 Ω p-p" per Mouser (units doubtful) | about 0.2 Ω changes resolvable if noise allows |
| Price [EST] | about $10 / $7 | about $10 / $7 | about $8 / $5, plus about $5 for ECG | about $25 / $18 |
| Main risk | Respiration resolution at 0.3 Ω not published | Temperature range | Two chips | Lifecycle unknown |

### Problems common to all

1. **Shock survival.** External protection is needed (series resistors in the tens of kilohms, defibrillator-rated chip resistors such as TT Electronics DPCR, clamps), plus the reed relays of section 3. These protection parts degrade with repeated shocks.
2. **The protection resistors are larger than the patient.** Measuring pad resistance precisely needs the measuring current and the voltage reading to take separate paths. The detail belongs in the withheld design files.
3. **The charger runs during analysis (D-29)** and can interfere with the impedance measurement. No datasheet can answer this; it must be measured.
4. **Recovery after a shock.** The limit is in IEC 60601-2-4 (not yet bought).

### Recommendation and test

Option A (ADS1292R), with option C as the upgrade if A's impedance resolution proves too coarse. Test at low voltage: impedance noise on a 25–250 Ω test resistor with a 0.3 Ω change at 2 Hz superimposed, with and without the charger running into a dummy load.

---

## 7. Controllers, and Bluetooth as a later option

### Processing need

About 120,000 multiply-adds per second and about 10.5 KB of RAM for the analysis: under 1% of a 100 MHz Cortex-M with a floating-point unit [CALC]. The event record (1 KB/s, 1.8 MB per 30 minutes) calls for an external flash chip of 16 MB or more [CALC; e.g. Winbond W25Q128 class, BK]. The temperature log of §8A is tiny by comparison (a few kilobytes a year [EST]).

### Ferrocene qualification (unchanged from version 0.1)

Ferrous Systems' documents list `thumbv8m.main-none-eabi(hf)` (Cortex-M33/M55/M85) and `thumbv7em` (Cortex-M4/M7) as **qualified** targets, the former since release 26.11.0. Cortex-M0/M0+ is only "supported". Caveats: the pages are development-branch previews; the signed qualification report was not seen; whether `-C target-cpu=cortex-m33` itself is in the qualified envelope is [UNVERIFIED]. Source links are in the research notes.

### Main controller and supervisor (D-39 confirmed)

| Role | Choice | Why |
|---|---|---|
| Main controller | **ST STM32U585-class** (Cortex-M33, 160 MHz, FPU), Rust via embassy-stm32 | Qualified target; no radio code on it (D-24, D-45) |
| Supervisor | **NXP MCX A-series** (Cortex-M33), Rust; HAL listed in the Embassy README | Qualified target; different maker, silicon and tools from the main controller |

Cost of D-39 against a Cortex-M0+ supervisor: **about 0 g and 0 cm³**, about $1–3 more per unit [EST]. Lifecycle of both families is [UNVERIFIED this pass].

### Bluetooth module: later option (D-09, D-45)

**Not in the first prototype (D-09).** The owner left Bluetooth out because the modules with ready-made firmware are being withdrawn, so the project would have to write and secure the module's own firmware. When it is added, it will be a separate pre-certified module (D-45). The candidates below are kept so that choice is ready.

**Space reserved now: 2 cm³, kept empty** inside the enclosure [EST: module 0.2–0.3 cm³ plus about 1–2 cm³ of antenna clearance, rounded up]. It is counted in the roll-up's volume (it costs about 4 cm³ of finished device through the packing factor) but carries no mass. The antenna needs that clearance to be near the enclosure wall and away from the battery and capacitor [BK].

**What it would have to do.** Handle all Bluetooth, on its own processor, and talk to the main controller over a simple serial link (a two-wire UART). It never has any path to the shock circuit. The main controller treats everything arriving from it as untrusted input.

**Candidates, checked 27 Nov 2026:**

| | A. Ezurio BL54L15 | B. Raytac AN54L15Q | C. Silicon Labs BGM220S |
|---|---|---|---|
| Chip inside | Nordic nRF54L15 (Cortex-M33 + RISC-V co-processor) | Nordic nRF54L15 | Silicon Labs EFR32BG22 |
| Size | 14 × 10 × 1.6 mm | 9.5 × 13.7 × 2.1 mm | **6 × 6 × 1.1 mm** |
| Temperature | −40 to +105 °C | −40 to +105 °C | [UNVERIFIED this pass] |
| Published radio approvals | Maker's page lists **FCC, ISED, EU, UKCA, RCM, MIC, KC**; Bluetooth qualified. FCC ID not shown on the page. The smaller BL54L15µ variant is listed by Digi-Key as only "working towards" approvals | Nordic's announcement says "fully pre-certified" without naming the approvals; resellers claim FCC/IC/CE/TELEC/NCC/SRRC [UNVERIFIED] | Datasheet lists **FCC, CE, ISED, MIC, KC**; **FCC ID QOQ-BGM220S2, granted 9 Sep 2020** (FCC ID listing) |
| Lifecycle | "Recommended for new design"; part numbers active | announced Nov 2024; availability date not stated | [UNVERIFIED] |
| Firmware on the module | Customer-written ("hostless"): Nordic SDK / Zephyr. Rust is possible with embassy-nrf and the `trouble` host; the radio's lowest layer stays Nordic's closed binary | as A | Silicon Labs' stack; a serial "network co-processor" mode is offered [BK; not confirmed this pass] |
| Sleep current | not stated | not stated | **1.4 µA** deep sleep with RAM kept |
| Price [EST] | $10–15 / $7–9 | $6–10 / $4–6 | $6–9 / $4–6 |

**Warning about ready-made serial firmware.** The modules that used to come with a ready-made "AT-command" serial firmware are being withdrawn: u-blox issued an end-of-life notice for NINA-B31x on 16 February 2026, and the ANNA-B112-00B has been replaced. Plan for **project-written module firmware** that does only three things: advertise readiness, hand over event records, and accept configuration that the main controller then checks (Q-40).

**What "modular approval" does and does not cover.** The module's radio approval can be reused only if the product follows the grant's conditions (antenna, layout keep-out, power). The finished device still needs its own emissions testing and the medical electromagnetic-compatibility standard (IEC 60601-1-2) [BK].

**Recommendation:** **A (Ezurio BL54L15)** for its published approval list and same-family Rust tooling; **C (BGM220S)** as the smallest, with a confirmed FCC grant.

**Size, mass and power when added [EST]:** module 0.2–0.3 cm³ and about 1 g, inside the reserved space. Advertising every 1–2 s averages roughly 10–30 µA at 3 V (30–90 µW) [BK], which would add back about 2–6% to the standby figure in §9.

### Voltage-sensing chains: can 1% + 7 V hold?

Unchanged from version 0.1.

- At 1,950 V the allowance is 26.5 V = **1.36%**. At 1,150 V it is 18.5 V = 1.61% [CALC].
- Each chain is a high-voltage resistor divider plus an analog-to-digital converter (ADC) with its own voltage reference.
- **Resistor class matters most.** Ordinary HV chip resistors can change value with applied voltage by "300 ppm and beyond" per volt (Stackpole). Only screened low-voltage-coefficient parts will do: Stackpole HVCR (better than 2 ppm/V in 2512), Vishay HV thin film (below 2 ppm/V), TT Electronics HV dividers (0.2–1 ppm/V).

**Worst-case error budget per chain** [CALC from the cited classes; ADC and reference figures BK]:

| Error source | Assumption | Error |
|---|---|---:|
| Divider ratio after one-point factory calibration | residual | 0.05% |
| Temperature tracking between top and bottom resistors | 50 ppm/K mismatch × 45 K | 0.23% |
| Voltage coefficient | 2 ppm/V × about 250 V per resistor | 0.05% |
| Life drift | 0.3% (TT Electronics best case; 0.5% typical) | 0.30% |
| Surface leakage across the board in humidity | 40 MΩ divider against 100 GΩ leakage | 0.04% |
| ADC gain and linearity | 0.1% [BK] | 0.10% |
| Voltage reference: initial plus drift | 0.05% + 12 ppm/K × 45 K [BK] | 0.10% |
| **Linear sum** | | **0.87%** (plus about 2 V offset) |
| **Root-sum-square** | | **0.41%** |

- **Verdict:** achievable with the right resistor class, but only just in the worst case over life. Drift is the largest term; board contamination is the hidden one (at 10 GΩ leakage it alone is 0.4%). Time spent hot in storage adds to drift [BK]; the temperature log of §8A will show how much a given unit has had.
- **The two channels should use different resistor and reference families**, so they cannot drift together and still agree.
- **Test:** both chains against a calibrated HV meter at 200, 1,150 and 1,950 V; at −20, +25 and +70 °C; and after 1,000 h at 85 °C and 85% humidity unpowered (a standard damp-heat test, not a car-storage figure).

---

## 8. User interface and the rest

| Item | Target and basis | Candidate class | Size / mass [EST] | Risk |
|---|---|---|---|---|
| Speaker + amplifier | ≥ 85 dB(A) at 1 m [EST basis: roadside traffic about 70–80 dB(A) [BK], plus about 10 dB] | Sealed 20–28 mm micro-speaker with vent membrane; 2–3 W class-D amplifier [BK] | 5 cm³ / 6 g | Loudness versus sealing; heat limits not checked (§8A) |
| Supervisor's own alarm (D-26) | ≥ 85 dB at 10 cm, driven only by the supervisor | Piezo transducer [BK] | 1 cm³ / 2 g | None significant |
| Display | Readable in sunlight, tiny power | Reflective "memory LCD" (Sharp LS013B7DH05 / LS027B7DH01 class): **operating −20 to +70 °C** (Sharp brochure); storage range not found [UNVERIFIED] | 3–6 cm³ / 4–10 g | Slow at −20 °C [BK]; liquid crystal can be permanently changed outside its rating (Sharp specification) |
| Cancel control | Usable with gloves; 15 mm or larger; IP67 | Sealed tactile switch under a guarded cap [BK] | 1 cm³ / 2 g | Stuck-button logic in D-31 |
| Enclosure | IP65; 1.5 m drop; −20 to +50 °C working; storage per §8A | Glass-filled nylon or PC/ABS, 2–2.5 mm walls, rubber bumper, one gasket; HV section potted or behind barriers | 100–150 g | **The 300 g capacitor must be held so a drop does not tear its leads** (Appendix A asks for a shock rating) |
| Pads and connector | Pre-connected; pad connector made to order (Appendix B) | Commercial pad pairs for fit and impedance tests only | Pads 80–100 g and 50–60 cm³ [EST; a Philips HeartStart adult set, 85 cm² conductor area and 1.2 m lead, is listed at 82 g packaged singly, 106 g per set in a 5-pack] | Pads are regulated as strictly as the device; **pads set the storage-heat limit (§8A)** |
| Charging and data contacts (D-46, D-09) | Sealed, dead when undocked; one or two data contacts for records | Flat contact pads on the device, spring contacts in the dock | 3 cm³ / 6 g | Corrosion; see §5 |
| Space for a Bluetooth module later (D-45) | Kept empty | — | 2 cm³ / 0 g | Antenna clearance must be kept free in the layout |
| Temperature sensing and log (D-48, new) | Requirements in §8A | Thermistors at cells, capacitor and pads; MCU internal sensor; passive heat-indicator label [BK that such labels exist] | about 1 cm³ / 1 g in all | Must keep logging when the device is "off" |

---

## 8A. Heat: part limits, parked cars and the envelope (new in version 0.3; envelope decided in D-48)

D-48 says the device is carried all day and left in a vehicle for short periods, not kept on a dashboard; it records its own temperature, and after overheating reports "not ready" until it has cooled and passed a self-test. This section gathers the evidence needed to turn that into numbers.

### H1. Heat limits of every temperature-sensitive part

"Long-term" and "short-term" are the makers' own words. All sources were checked 27 Nov 2026; links are in the research notes. Where the figure came from a search-result extract of the maker's document rather than the document itself, the source column says "extract".

| Part | Long-term storage | Short-term storage (time limit) | Operating | What the maker says heat does | Source |
|---|---|---|---|---|---|
| **Pads (the project's are made to order, D-42; commercial pads are the evidence)** | | | | | |
| ZOLL CPR-D-padz | **0 to 35 °C** | **−30 to 65 °C** (time limit not found [UNVERIFIED]) | 0 to 50 °C | Storing outside the recommended conditions may damage pads and batteries or shorten their useful life (ZOLL AED Plus manual) | ZOLL pad instructions for use (extract); US device database lists 0–35 °C |
| Philips SMART Pads II (FRx) | 0 to 50 °C | FRx device with battery and pads: **−20 to 60 °C for up to 1 week** (storage and shipping) | FRx: 0 to 50 °C standby and operating; −20 to 50 °C for up to 20 min after leaving 20 °C | — | Philips product page; Philips technical manual (extract, full table not seen); FRx brochure (read) |
| Philips HS1/OnSite pads (M5071A) | **10 to 43 °C** | none found | — | — | Philips product page (extract) |
| HeartSine Pad-Pak (pads and battery in one cartridge) | 0 to 50 °C (standby) | 350P/360P: −10 to 50 °C **for up to 2 days** in transport; return to 0–50 °C for 24 h before use. 450P: 0–50 °C only | 0 to 50 °C | Long-term storage outside 10–50 °C "may adversely affect the performance" (older HeartSine note) | HeartSine/Stryker data sheets (extract) |
| Stryker LIFEPAK CR2 (device with battery and electrodes) | **15 to 35 °C** (one Stryker FAQ says 15–25 °C) | **−30 to 60 °C, maximum exposure one week** | 0 to 50 °C | Long-term storage at the upper end of the range shortens battery and electrode life | Stryker EMEA data sheet and support article (extract) |
| Avive Connect (device, pads included) | **5 to 40 °C** | **−20 to 60 °C for up to 14 days** | 0 to 50 °C; battery charging 0–45 °C | Return to 0–50 °C for 24 h after being outside it | Avive help centre (extract) |
| Schiller FRED PA-1 (device) | −20 to 50 °C (storage and transport) | — | −5 to 40 °C | — | Schiller technical data (extract) |
| **Battery cells** | | | | | |
| Molicel P28A (the study's cell) | **below 35 °C** for shipping; **below 23 °C recommended** long term; shelf life 1 year | **none published** in the copy found | charge 0–60 °C; discharge −40 to 60 °C | — | Molicel specification (third-party copy) |
| Samsung INR18650-25R (same class, for reference) | 1.5 years at up to 25 °C | 3 months at up to 45 °C; **1 month at up to 60 °C** (≥ 90% capacity recovered) | charge 0–45 °C [BK]; discharge to 60 °C | Capacity lost when stored hot is partly permanent | Samsung SDI specification, 2014 (third-party copy) |
| A123 ANR26650M1B (LFP, heat-tolerant alternative) | — | storage **−40 to 70 °C** (2022 maker sheet; an older A123 sheet says −40 to 60 °C) | −30 to 60 °C; charge 0–60 °C | — | Lithium Werks data sheet (read) |
| Saft MP 174565 xtd (heat-tolerant Li-ion, listed for medical devices) | recommended +15 to 30 °C | allowable **−40 to 85 °C** | charge −30 to 85 °C; discharge −40 to 85 °C | The sheet shows the cell thickening after a year on charge at 60 °C and after 100 cycles at 85 °C | Saft data sheet (read) |
| **Battery electronics** | | | | | |
| TI BQ25887 charger | — | — | −40 to 85 °C | not limiting | TI page (version 0.2) |
| ADI MAX17320 gauge and protector | [BK: −55/−65 to 150 °C, typical for chips] | — | [BK: −40 to 85 °C] | not limiting | not re-checked [UNVERIFIED] |
| **Capacitor** | | | | | |
| Project capacitor (Appendix A) | | **70 °C** (D-41) | −20 to 60 °C working (D-41) | Film shrinkage and capacitance loss when hot; life halves per 10 °C under voltage [BK, Digi-Key forum] | D-41 |
| TDK B32365 | | −20 to 70 °C | −20 to 55 °C | — | TDK data sheet |
| CRE DMJ-PC | | −40 to 85 °C | −25 to 60 °C (125 µF) / −25 to 70 °C (200 µF, 2.2 kV) | — | CRE page |
| **The rest** | | | | | |
| Sharp memory LCD | not found [UNVERIFIED] | not found | **−20 to 70 °C** | Outside the rated range the liquid crystal can change permanently | Sharp brochure; Sharp specification on Avnet (summary) |
| Micro-speaker and piezo | [BK: −40 to 85 °C typical] | | | Standard magnets lose strength above about 80 °C [BK] | not checked [UNVERIFIED] |
| Enclosure plastics | [BK: PC/ABS softens around 95–115 °C; glass-filled nylon far higher] | | | not limiting below 85 °C | [BK] |
| Gasket (silicone) | [BK: −50 to 200 °C] | | | Heat increases permanent squash ("compression set"), which can loosen the seal over years | [BK] |
| Adhesive tape (3M VHB 5952 as an example) | **93 °C** long term (holds a load for 10,000 minutes) | higher for a few hours | | not limiting if a tape of this class is chosen | 3M technical data sheet (extract) |
| Bluetooth module, front-end chip, processors | | | −40 to 85 °C or more | not limiting | §6, §7 |

### H2. Which part sets each limit (the weakest links)

| Limit | Weakest link | Figure | Runner-up |
|---|---|---|---|
| Continuous storage (any length of time) | **Pads** | 35–40 °C (ZOLL 35, Stryker 35, Avive 40) | Battery: 23–30 °C *recommended* for long storage, 45 °C allowed for 3 months (class figure); heat shortens its 2–3-year service life rather than ending it |
| Excursion of hours to days | **Pads and battery together** | 60 °C (Philips and Stryker: 1 week; Avive: 14 days; cell class: 1 month) | ZOLL pads allow 65 °C |
| Above which damage is possible | **Battery and pads are already outside every published rating above 60–65 °C**; at 70 °C the capacitor (D-41) and display (operating limit) also reach theirs | 70 °C | — |
| Above which every part is outside its rating | All | above 70 °C | — |

**Plainly:** the 70 °C capacitor is no longer the weak link, and could not have been with an 85 °C rating either. The device as a whole is a **60 °C device for hours, and a 35–40 °C device for months**, because that is what the pads and the battery are rated for.

### H3. What commercial AEDs accept (benchmark)

| Product | Normal storage | Short-term allowance |
|---|---|---|
| Stryker LIFEPAK CR2 | 15–35 °C | −30 to 60 °C, up to 1 week |
| Philips HeartStart FRx | 0–50 °C standby | −20 to 60 °C, up to 1 week (extract) |
| Avive Connect | 5–40 °C | −20 to 60 °C, up to 14 days |
| ZOLL AED Plus | device −30 to 60 °C (spec sheet extract); its pads 0–35 °C long term, −30 to 65 °C short term | — |
| HeartSine 350P/360P | 0–50 °C | −10 to 50 °C, up to 2 days |
| Schiller FRED PA-1 | −20 to 50 °C | — |

**The market's norm is 35–50 °C for normal storage and 60 °C for up to a week.** No commercial AED found claims more than 60–65 °C. A device that matches 60 °C would be level with the best of them.

### H4. How hot parked cars get (published measurements)

| Finding | Figure | Source |
|---|---|---|
| How fast a cabin heats | Average rise of about **22 °C (40 °F) in one hour**, whatever the outside temperature between 22 and 36 °C; **80% of the rise in the first 30 minutes**; cracked windows barely help | McLaren, Null and Quinn, *Pediatrics* 2005 (Stanford release and abstract) |
| Cabin air, hot climate, 1 hour in sun | **47 °C** average (116 °F), on days above 35 °C in Tempe, Arizona | Vanos and others, *Temperature* 2018 (ASU release) |
| Cabin air, same, 1 hour in shade | **38 °C** (100 °F) | same |
| Dashboard surface, 1 hour in sun | **69 °C** average (157 °F); **89 °C** (192 °F) in one car | same |
| Dashboard surface, 1 hour in shade | **48 °C** (118 °F) | same |
| Steering wheel / seat, sun | 53 °C / 51 °C (127 / 123 °F) | same |
| Cabin above outside air | more than 20 °C above ambient in sun; a black car about 5 °C hotter than a white one; a window open 2.5 cm lowers it only about 3 °C | Dadour and others, *Forensic Science International* 2011 (abstract) |
| Hottest interior reading in a European study | **up to 89 °C** in summer (where in the car is not stated in the abstract) | Marty, Sigrist and Wyler, *Am J Forensic Med Pathol* 2001 (abstract) |
| Peak over a 12-hour day, no direct sun on the items | **62 °C** (143.6 °F) in a black sedan; the glovebox did most harm to stored adrenaline, the trunk least | Lodz pilot study on adrenaline auto-injectors, *J Allergy Clin Immunol Pract* 2018 (reports) |
| Emergency-vehicle storage compartments | up to **56.8 °C** (South Africa, a response car); −9 to **46.8 °C** over 102,524 readings (UK air ambulance service) | South African Journal of Pre-hospital Emergency Care, 2021; Essex & Herts Air Ambulance study, 2022–2023, published by Cambridge University Press (extracts) |

**Checking the earlier notes.** Version 0.2 said "cabin air 47–70 °C and dashboard 70–100 °C". **Cabin air 47 °C is confirmed** (one hour in sun in a hot climate), and a full hot day reaches about 60–65 °C (62 °C measured). **I found no source for cabin air of 70 °C** [UNVERIFIED]; the 89 °C "interior" figure does not say whether it was air or a surface. **Dashboard 70 °C is confirmed as the average after one hour in sun, and 89 °C as a measured maximum; I found no source for 100 °C** [UNVERIFIED]. Corrected figures: **cabin air about 38 °C (shade) to 47 °C (sun) after one hour in a hot climate, about 60–65 °C at the peak of a hot day; dashboard about 48 °C (shade) to 69 °C (sun) on average, 89 °C at the measured maximum.** Trunk and glovebox: **no published temperature measurement found** [UNVERIFIED]; the adrenaline study suggests the glovebox runs at least as hot as the cabin and the trunk cooler.

**How long the peak lasts.** The cabin reaches most of its rise within 30–60 minutes and then follows the sun; in a hot climate it stays near its peak through the afternoon, several hours [EST from the 80%-in-30-minutes finding and the sun's path].

**How fast the device follows.** The device's thermal time constant is about **25 minutes** in still air [CALC, §4]. So a 30-minute stop lets the inside of the device make only about 70% of the cabin's rise, a 1-hour stop about 90% [CALC: 1 − e^(−30/25) = 0.70; 1 − e^(−60/25) = 0.91]. **This lag only applies in shade**: a device lying in direct sun absorbs sunlight itself and can approach dashboard temperatures [EST].

### H5. The envelope options (version 0.3) and the decision

**Decided (D-48): option A plus the insulated pouch kept in the vehicle, which version 0.3 called option B.** The table is kept to show what was compared. Weights in it are version 0.3's; see §9 for version 0.4.

All temperatures are **measured inside the device**, by the sensors of H7. Each option has four bands.

| Band | **Option A: commercial benchmark** (no new parts) | **Option B: A plus an insulated car pouch** | **Option C: heat-tolerant parts** |
|---|---|---|---|
| 1. Normal: no limit | up to **40 °C** | up to 40 °C | up to 40 °C |
| 2. Excursion: stays ready, time logged | 40–**60 °C**, up to **8 h** at a time above 50 °C; and a lifetime allowance for the pads (D-52: about 7 days above 50 °C) | same limits as A; the pouch makes the device reach them about four times more slowly | 40–**65 °C**, same time rules |
| 3. Too hot: "not ready" | **60–70 °C**. Ready again once every sensor has been below 45 °C for 30 minutes and the daily self-test and battery load test pass | same as A | **65–70 °C**, same recovery |
| 4. Damage possible: "service needed" | **above 70 °C** | same as A | above 70 °C |
| Weakest link at the band 2 / band 3 boundary | Pads and cell (60 °C × 1 week; cell class 60 °C × 1 month) | same | Pads (ZOLL-class 65 °C) |
| Weakest link at band 4 | Capacitor (D-41) and display at 70 °C; cell and pads already outside their ratings | same | LFP cell storage, capacitor and display all 70 °C |
| Parts change | none | a pouch kept in the vehicle, not carried; about 50–100 g and $10–30 [EST]; not counted in the 700 g | 2 × LFP 26650 cells instead of 2 × 18650; pads specified to 65 °C short term |
| Weight on the belt (lower-risk set, with pads) | **903 g, 0.89 L** | **903 g, 0.89 L** | **975 g, 0.97 L** (+72 g, +0.08 L) [CALC: cells 152 g instead of 92 g; about +36 cm³ of cell volume, carried through packing and shell] |
| Other costs | — | The responder must put the device in the pouch | Cold-weather charging arithmetic must be redone for LFP (lower voltage, different cold behaviour) [UNVERIFIED]; less energy (17 Wh against 20 Wh) [CALC: 2 × 8.58 Wh] |

**Why 8 hours.** One working shift. The published allowances at 50–60 °C are 2 days (HeartSine), 1 week (Philips, Stryker) and 14 days (Avive) per event. A device in a patrol car meets this repeatedly over years, so the study takes a much shorter limit per event and adds a lifetime allowance for the pads [EST].

**Why not a dashboard option.** Surviving a dashboard in sun (69 °C on average, 89 °C measured) would need every part rated to about 90 °C. A cell exists (Saft MP xtd, to 85 °C), but each can deliver only 5 A continuously; supplying the charger's 7.6 A average and 15 A peaks would take four of them, about **464 g of cells** [CALC: 7.6 A ÷ 5 A → 2 strings × 2 in series = 4 × 116 g]. No pad maker publishes anything above 65 °C. **No option offers it.**

**About the lithium-ion safety question.** Heat in bands 3 and 4 is a damage risk, not normally a fire risk: thermal runaway of a healthy cell starts well above 100 °C [BK]. The risks that matter are:
1. **Charging a hot cell**, which ages it fastest and, in a damaged cell, can start a failure [BK]. Hence the 45 °C charging limit (H7).
2. **Using a cell that heat has already damaged** (swollen, leaking charge, higher resistance). The Saft data sheet itself shows cells thickening after long heat. Hence the battery health check after band 4, and replacement if it fails.
3. **A cell outside its discharge rating** (above 60 °C) delivering 8–9 A to charge the capacitor in a rescue. **Decided (D-51):** a device switched on while too hot guides CPR and analyses, but does not charge until it is back inside the limits.

### H6. Car situations: inside or outside each option

Device temperatures are this study's estimates from H4 and the 25-minute (option B: about 90-minute) time constant [EST]. Option B's pouch: 10 mm of foam around the device roughly quadruples the time constant [CALC: foam conductance 0.035 W/m·K × 0.05 m² ÷ 0.01 m = 0.175 W/K, in series with 0.49 W/K to air → 0.13 W/K; 720 J/K ÷ 0.13 W/K = 93 minutes].

| Situation | Device likely reaches | A | B | C |
|---|---|---|---|---|
| Worn on a belt, or indoors | 20–37 °C | Normal | Normal | Normal |
| Door pocket, footwell or under a seat, 25–30 °C day, car in sun, 1 hour | cabin about 47–52 °C (+22 °C); device about 45–50 °C | Excursion, ready | about 40 °C, normal | Excursion, ready |
| Same, 35–40 °C desert day, 1 hour | cabin about 47 °C average; device about 45 °C | Excursion, ready | about 38–40 °C | Excursion, ready |
| Same, whole afternoon (4 h or more) in desert heat | cabin about 60–65 °C at peak; device close to it | **May go "not ready"**; no damage expected | Delayed by about 1.5–2 h; may still go "not ready" | Ready up to 65 °C; "not ready" above |
| Emergency-vehicle compartment, hot day | up to about 57 °C (measured) | Excursion, ready | Excursion or normal | Excursion, ready |
| Car parked in shade, 38 °C day, even on the dashboard | cabin about 38 °C, dashboard about 48 °C | Normal or excursion | Normal | Normal or excursion |
| Glovebox | no measurement found; assume at least cabin air [UNVERIFIED] | as the cabin rows | as the cabin rows | as the cabin rows |
| Trunk | no measurement found; probably no hotter than the cabin [UNVERIFIED] | as the cabin rows | as the cabin rows | as the cabin rows |
| **Dashboard or rear shelf in direct sun, 30–40 °C day, 1 hour or more** | dashboard surface 69 °C average, 89 °C measured | **Outside: "service needed"** | **Outside** (the pouch slows it, but direct sun for hours defeats it) | **Outside** |
| Dashboard in direct sun on a mild 20 °C day | cabin about 42 °C; dashboard perhaps 50–60 °C [EST] | Excursion; possibly "not ready" | Excursion | Excursion |

**Plainly:** under every option, normal patrol-car use (door pocket, seat, footwell, compartment, shade) keeps the device ready on all but the hottest afternoons, and even then it only goes "not ready" for a while. **The dashboard or rear shelf in sun is outside every option**, as D-48 already says.

### H7. What the device must do to enforce the envelope (requirements, not circuits)

**R-H1. Where temperature is measured.**
- At each battery cell (the gauge's own temperature inputs).
- On the capacitor body.
- Next to the stored pads (the pads' temperature cannot be measured directly; this is the proxy).
- On the main board (the processor's own sensor is enough).
- At least one of these is read independently by the supervisor (D-26, D-39), so a failed main controller cannot hide an overheat.
- Accuracy ±2 °C from 0 to 80 °C; readable from −40 to +100 °C so that an over-limit event is measured, not just "off the scale" [EST].

**R-H2. Logging that never stops.**
- Sample every sensor at least every 5 minutes below 40 °C and every minute above 40 °C [EST].
- Keep, in memory that survives power loss: the highest reading per sensor; time spent in each band per event; running totals per pad set and per battery; the time of each event.
- Logging runs in **every** state the device can be in, including what the user thinks of as "off". There is no off switch that stops it.
- **If logging is ever interrupted** (battery cut off by its protector, or removed), the device treats the gap as unknown history: "not ready" until a full self-test passes, and the gap is recorded.
- **Power cost [EST]:** about 10–20 µW (a sleeping processor with its clock running, waking briefly to read the sensors): about **0.24–0.48 mWh a day**, about 1% of the 25–36 mWh-a-day standby budget (§9).
- **Independent witness:** a passive, non-electronic heat-indicator label next to the pads that changes colour permanently at 60 °C and at 70 °C, so an inspector can see an overheat the electronics missed [BK that such labels exist; no part checked].

**R-H3. The "not ready" rules** (D-48):
- Band 3 (above 60 °C) → "not ready" at once, with the reason shown ("too hot, let it cool") and recorded for the next read-out on the dock. Back to "ready" after every sensor has read below 45 °C for 30 minutes [EST] and the daily low-voltage self-test and the battery load test pass.
- Band 4 → "service needed". Back to "ready" only after: cooling as above; the full-energy self-test; a battery health check (internal resistance against the cell's own earlier value, and a 24–48 hour self-discharge check, which the MAX17320 class provides); and a pad resistance check. The record recommends replacing the pads, and replacing the battery if the health check fails.
- Band 2 beyond **8 hours** at a time above 50 °C → "not ready" until cooled and re-tested, as band 3 (D-48).
- **Pad heat total (D-52):** the device adds up the time the pad sensor spends above 50 °C over the life of each pad set, and calls for new pads after about **7 days (168 h)** in total. The figure is to be set from the pad maker's data; it is reset only when new pads are fitted (the pad-present sense of Appendix B3 can mark the change [EST]).
- **Charge count (D-53):** the device counts full-energy charges over the capacitor's life and calls for service as it nears **2,500** (warning point: question O). The count survives power loss and is read out on the dock.
- D-50 and D-51 apply: a "not ready" device switched on in an emergency still attempts the rescue; if it is too hot to charge it guides CPR and analyses, and charges and shocks once back inside the limits.

**R-H4. Charging limits.**
- **Battery charging** only between **0 and 45 °C** cell temperature, with reduced current below 10 °C [EST]. The P28A allows 0–60 °C, but heat plus full charge is the fastest way to age a cell [BK], and Avive gives 0–45 °C as its charging range. The BQ25887 charger's JEITA function provides this kind of rule.
- **Capacitor charging** only while the capacitor is at or below **60 °C** (D-41, D-48) and the cells are within their **discharge rating** (60 °C). Outside those limits, in a rescue, the device guides CPR and keeps analysing, and charges as soon as both are back inside (D-51).
- **Weekly full-energy self-test** only with the capacitor at or below 50 °C and the cells at or below 45 °C, consistent with D-49's rule that the full-energy test waits for heat [EST].

**R-H5. Reporting.** Every band change, every refusal to charge, the pad heat total and the charge count are recorded and **read out through the dock** (D-09), so a fleet manager can see which vehicles cook their devices. The device shows its own ready / not-ready state on its screen; with no Bluetooth there is no remote readiness report in the first prototype.

### H8. The vehicle pouch (D-48, new in version 0.4)

An accessory kept in the vehicle. It is **not carried** on the belt and is **not counted** in the 700 g.

**What it must do (requirements) [CALC from §4 and H6, EST where marked]:**
- **Slow the heating at least threefold.** The bare device's thermal time constant is about 25 minutes in still air. In the pouch it must be **at least 75 minutes**, with a target of 90 minutes.
- In practice, as a test: a device-sized dummy at 25 °C, in the pouch, moved into still air at 60 °C, must take **at least 52 minutes** to make half the rise (to 42.5 °C) [CALC: 75 × ln 2 = 52 min]. The target is **at least an hour** [CALC: 90 × ln 2 = 62 min].
- **What it buys:** with a 90-minute constant, a 2-hour stop in a 60 °C cabin leaves the device at about 52 °C, still ready, instead of about 60 °C [CALC: 30 + 30 × (1 − e^(−120/90)) = 52; bare device 30 + 30 × (1 − e^(−120/25)) = 60].
- **Insulation that meets it [CALC]:** about 10 mm of closed-cell foam all round (thermal conductivity about 0.035 W/m·K [BK]) gives about 0.13 W/K from device to air, and so a time constant of about 93 minutes with a 0.8 kg device. Thinner foam scales roughly in proportion.
- **Reflective outer face**, so sunlight through a window is mostly reflected [EST]. The pouch still does not make a dashboard in direct sun acceptable.
- **Quick draw:** the device comes out one-handed in about 3 seconds, with gloves [EST].
- **Visible and labelled**, so it is not mistaken for a lunch bag; and it must not block the device's screen from showing its state when the flap is open [EST].
- **The other side of insulation:** the pouch also slows *cooling*. A device that went over 60 °C stays "not ready" longer in the pouch. Taking it out of the pouch in a cool place lets it recover sooner. This belongs in the user instructions.
- **No cooling packs required.** Phase-change or evaporative inserts would extend the protection but need refreezing or re-wetting; they are optional extras, not part of the requirement [EST].

**Candidate product classes [BK; no specific product checked]:** insulated medicine and insulin travel cases; foil-faced closed-cell foam pouches sold for keeping drugs or instruments cool in vehicles; small insulated EMS drug pouches. Many such products would need a custom size. About $10–30 at prototype quantities and $5–15 at qty 100, 50–100 g [EST].

---

## 9. Roll-up: volume, mass, cost and power

### Assumptions

- **Packing factor 0.5** (lower-risk set) or **0.6** (smallest set) [EST; real handhelds run 0.4–0.6, and 2 kV spacing pushes toward the lower end].
- The shell is added to the inside volume. When the inside grows, the shell's volume and mass grow with its surface area, that is with (inside volume)^⅔, scaled from version 0.1's shell [CALC].
- Pads are included in the verdict (D-43).
- Version 0.4 follows every decision as it stands on 27 Nov 2026:
  - the capacitor of D-41 and D-53 (about 70 °C storage, 60 °C working, 2,500 charges);
  - two cells (D-44) and sealed dock contacts (D-46), now with data contacts as well (D-09);
  - **no Bluetooth module, its space reserved** (D-09, D-45);
  - temperature sensing (D-48);
  - the vehicle pouch, which is **not carried** and so is not counted.

### Versions 0.1 to 0.4 side by side [CALC]

**(a) Lower-risk set**: every decision followed, the lower-risk choice wherever a choice remains, and the **expected** capacitor. **(b) Smallest plausible set**: every decision followed, the smallest credible choice everywhere, and the **low** capacitor.

| | v0.1 (a) | v0.1 (b) | v0.2 (a) | v0.2 (b) | v0.3 (a) | v0.3 (b) | **v0.4 (a)** | **v0.4 (b)** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Capacitor, cm³ / g | 180 / 250 (2.0 kV) | 150 / 210 | 210 / 300 (2.2 kV, 85 °C storage, 65 °C working) | 170 / 225 | 210 / 300 (70 °C storage, 65 °C working) | 170 / 225 | **170 / 240** (60 °C working, 2,500 charges) | **145 / 185** |
| Parts subtotal, cm³ / g | 325 / 594 | 259 / 464 | 357 / 653 | 292 / 509 | 358 / 654 | 293 / 510 | **319 / 594** | **269 / 470** |
| Inside volume, cm³ | 650 | 432 | 714 | 487 | 716 | 488 | **638** | **448** |
| Shell and gasket, cm³ / g | +110 / 140 | +90 / 100 | +117 / 149 | +97 / 108 | +117 / 149 | +98 / 109 | **+109 / 138** | **+92 / 103** |
| Device without pads | 0.76 L / 734 g | 0.52 / 564 | 0.83 / 802 | 0.58 / 617 | 0.83 / 803 | 0.59 / 619 | **0.75 L / 732 g** | **0.54 L / 573 g** |
| Pads | +60 / 100 | +50 / 80 | +60 / 100 | +50 / 80 | +60 / 100 | +50 / 80 | +60 / 100 | +50 / 80 |
| **Device with pads** | **0.82 L / 834 g** | **0.57 L / 644 g** | **0.89 L / 902 g** | **0.63 L / 697 g** | **0.89 L / 903 g** | **0.64 L / 699 g** | **0.81 L / 832 g** | **0.59 L / 653 g** |
| Against 700 g / 0.7 L | +134 g / +0.12 L | −56 g / −0.13 L | +202 g / +0.19 L | −3 g / −0.07 L | +203 g / +0.19 L | −1 g / −0.06 L | **+132 g / +0.11 L** | **−47 g / −0.11 L** |

### Version 0.4 blocks [CALC from the blocks; block values EST as in §§1–8]

| Block | (a) cm³ | (a) g | (b) cm³ | (b) g | Change from version 0.3 |
|---|---:|---:|---:|---:|---|
| Capacitor (D-41, D-53) | 170 | 240 | 145 | 185 | 60 °C working and 2,500 charges (§1) |
| Charger | 20 | 35 | 15 | 25 | — |
| Bridge + gate drive + reed relays | 35 | 50 | 25 | 30 | — |
| Discharge load | 10 | 25 | 10 | 21 | — |
| Battery, gauge, charger, dock contacts **including data contacts** (D-44, D-46, D-09) | 40 | 101 | 40 | 101 | +1 cm³, +1 g for one or two data contacts (§5) |
| Front end + protection | 20 | 25 | 15 | 20 | — |
| Controllers, flash, voltage sensing, **empty space reserved for a Bluetooth module** (D-39, D-09, D-45) | 11 | 12 | 10 | 9 | module removed (−1 g); its 2 cm³ kept empty (§7) |
| Temperature sensors and heat label (D-48) | 1 | 1 | 1 | 1 | — |
| Speaker, alarm, display, button, LEDs | 12 | 20 | 8 | 15 | — |
| Boards, wiring, fasteners | (in packing) | 50 | (in packing) | 38 | — |
| HV potting / barriers | (in packing) | 35 | (in packing) | 25 | — |
| **Parts subtotal** | **319** | **594** | **269** | **470** | |
| Inside volume = parts ÷ packing factor | 638 | | 448 | | ÷ 0.5; ÷ 0.6 |
| Shell and gasket (scaled with surface) | +109 | 138 | +92 | 103 | |
| **Device without pads** | **0.75 L** | **732 g** | **0.54 L** | **573 g** | |
| Pads, pre-connected | +60 | +100 | +50 | +80 | |
| **Device with pads (D-43)** | **0.81 L** | **832 g** | **0.59 L** | **653 g** | |
| Vehicle pouch (D-48), not carried | — | — | — | — | about 50–100 g, kept in the vehicle [EST] |

**Low, expected and high capacitor in each set [CALC]:**

| Capacitor case (§1) | (a) with pads | (b) with pads |
|---|---|---|
| Low: 145 cm³ / 185 g | 0.75 L / 770 g | **0.59 L / 653 g** |
| Expected: 170 / 240 | **0.81 L / 832 g** | 0.64 L / 714 g |
| High: 210 / 315 | 0.90 L / 919 g | 0.71 L / 798 g |
| Heavy-duty capacitor for busy services (later option, D-53): 5,000 charges at 60 °C, about 190 / 270 | 0.85 L / 868 g (+36 g) | 0.68 L / 749 g |
| (a) with 2 × 21700 cells instead of 2 × 18650 (Q-39) | 0.85 L / about 890 g (+58 g) | — |

### What each decision cost or saved [CALC]

Each decision applied alone to the version 0.1 set, carried through the packing factor and shell. Grams and cm³ of the finished device with pads. A minus sign is a saving.

| Decision | Lower-risk set | Smallest set | Why |
|---|---:|---:|---|
| **D-41 and D-53** capacitor 2.2 kV, about 70 °C storage, **60 °C working, 2,500 charges** | **−22 cm³, −8 g** | **−10 cm³, −23 g** | The 2.2 kV rating costs size. Sizing for 60 °C working and typical use more than wins it back, because version 0.1's lower-risk part was a 2.0 kV catalogue part larger than 100 µF (130 µF). The figure includes a stronger mount (+3–5 g) |
| — in version 0.3 the same decision (65 °C working, 5,000 charges) cost | +67 cm³, +63 g | +38 cm³, +23 g | for comparison: the 60 °C and 2,500 answers are worth about **71 g** on the lower-risk set |
| **D-44** two cells | 0 | **+19 cm³, +25 g** | the lower-risk set already had two cells; the smallest set had one |
| **D-09** no Bluetooth in the first prototype, records through dock data contacts, space kept for a module | +2 cm³, 0 g | +2 cm³, 0 g | radio removed (−1 g); data contacts added (+1 g); 2 cm³ reserved empty |
| D-45 Bluetooth as a separate module | 0 now | 0 now | deferred by D-09; about +1 g when added, into the reserved space |
| **D-46** dock contacts instead of USB-C | +2 cm³, +3 g | +2 cm³, +3 g | contacts, alignment magnets, input protection |
| **D-48** temperature sensing, logging, heat label; vehicle pouch | +2 cm³, +1 g | +2 cm³, +1 g | the pouch stays in the vehicle and is not counted |
| D-51, D-52 hot-start rule, pad heat total | 0 | 0 | firmware rules on the sensors already counted |
| **D-39** two Cortex-M33s | about 0 | about 0 | costs a few dollars, not grams |
| **D-15** weekly full, daily low-voltage self-test | 0 | 0 | *saves* the larger capacitor that twice-daily tests would have needed |
| D-42 made-to-order parts allowed | 0 | 0 | no size effect; one-off design charges (cost table) |
| **Total of the decisions** | **−16 cm³, −4 g** | **+15 cm³, +6 g** | the roll-up gives −13 cm³ / −2 g and +19 cm³ / +9 g; the difference is rounding and the shell's non-linear scaling |
| **D-43** pads counted in the target | +60 cm³, +100 g against the target | +50 cm³, +80 g | bookkeeping: the pads were always carried, now they count |

**Against version 0.3, the latest answers saved [CALC]:** lower-risk set 903 → 832 g (−71 g, −0.08 L), all from the capacitor. Smallest set 699 → 653 g (−46 g, −0.05 L), because its low-case capacitor moves from 1.66 to about 2.0 J/cc (§1). Removing Bluetooth and adding data contacts cancel out in mass.

### Against targets and commercial units

| | Volume | Mass | Energy |
|---|---:|---:|---|
| Goal for the finished design (REQUIREMENTS, D-43) | 0.7 L | 700 g with pads | 150 J |
| Accepted for a first prototype (D-43) | — | 780–900 g | 150 J |
| Stretch target | 0.5 L | 500 g | 150 J |
| **(a) Lower-risk set, version 0.4** | **0.81 L** | **832 g** | 150 J |
| **(b) Smallest plausible, version 0.4** | **0.59 L** | **653 g** | 150 J |
| Version 0.3 (a) / (b) | 0.89 / 0.64 L | 903 / 699 g | 150 J |
| St John Pocket AED P3 | 0.68 L | < 700 g incl. pads | 150 J |
| Schiller FRED easyport plus | 0.99 L | 780 g | up to 200 J |
| Avive Connect | 1.19–1.51 L | 953 g | 150 J |
| CellAED | 0.31 L | ~300 g | ~75–85 J |

**Plainly:**
- **The lower-risk set misses the 700 g goal by 132 g (19%) and 0.7 L by 0.11 L (16%).** It sits inside the 780–900 g you accepted for a first prototype, close to the Schiller (780 g) and well under the Avive Connect (953 g).
- **The smallest set meets the goal with 47 g and 0.11 L to spare**, if the capacitor reaches its low case. With the expected capacitor it is **714 g, 14 g over**. So 700 g for the finished design now needs most of the no-decision levers below, but no longer every one of them at its best.
- **500 g / 0.5 L: not reachable.** The capacitor, battery and shell alone are about 390 g in the smallest set.

### Levers that could close the gap, re-ranked by grams saved

Starting from the lower-risk set (832 g, 0.81 L). Savings are for the finished device with pads [CALC from the roll-up, or EST as marked].

| Rank | Lever | Saves | Cost in risk or function | Needs an owner decision? |
|---:|---|---:|---|---|
| 1 | **Smaller-block choices everywhere else** (surface-mount SiC bridge, lighter boards and potting, smaller charger, front end and user interface: the smallest-set values) | about 70 g, 60 cm³ [CALC: block differences] | Surface-mount switches' pulse rating unproven; less insulation margin; quieter speaker | No |
| 2 | **Capacitor at the low end**: a maker shows about 2.0 J/cc at 2.2 kV, 60 °C working, 2,500 charges | **62 g, 0.06 L** | None in function; depends on what makers can prove (CDE claims "as high as 2 J/cc"; nobody publishes it at 2.2 kV) | No |
| 3 | **Smaller capacitor** (for example 80 µF) | about 58 g, 0.08 L [CALC: 80% of the expected part] | Leaves the Philips-equivalent design point kept for the bench-equivalence approval route; the waveform model would need re-running | **Yes: revisits D-41 and D-23** |
| 4 | **Thinner, ribbed shell** (about 110 g instead of 138 g) | about 28 g [EST] | 1.5 m drop and IP65 harder to pass | No |
| 5 | **One cell, accepting slower charging in the cold** | about 25 g, 19 cm³ [CALC] | The arithmetic in §5 says a nearly flat single cell fails at −20 °C | **Yes: revisits D-44** |
| 6 | **Lighter pads** (smaller pouch, shorter lead) | about 20 g [EST] | Lead must still reach for front-and-back child placement; pads are a regulated part | No |
| 7 | **Tighter packing** (0.6 instead of 0.5) with insulating barriers or potting rather than air gaps | about 16 g, 0.12 L [CALC: shell scaling] | Depends on the spacing the standard requires at 2 kV (not yet bought); potting adds some mass back | No |
| 8 | **Stop charging during analysis** (smaller discharge load) | about 10–15 g [EST] | Adds about 6 s of hands-off time per cycle | **Yes: revisits D-29**; not recommended |
| — | Capacitor working rating 55 °C instead of 60 °C | about 0 g now | The expected case is already capped at the densest part on sale (§1); a lower working rating would only add margin to an estimate makers have not yet confirmed | not worth asking |
| — | ~~60 °C working, 2,500 charges, 70 °C storage, Bluetooth out~~ | taken | D-41, D-53, D-09 | done |

**What the levers add up to.** The levers that need no decision (1, 2, 4, 6, 7; about 200 g) bring the lower-risk set to about the smallest set, **about 650 g**. A finished design at 700 g therefore has roughly 50 g of slack against those levers, which it did not have in version 0.3.

### Cost estimate [EST]

All prices are estimates; see the access note at the top.

| | Qty 1 (prototype) | Qty 100, per unit |
|---|---:|---:|
| Capacitor (made to order, 2.2 kV, 70 °C storage, 60 °C working, 2,500 charges) | $180–550 (sample, quote) | $95–230 |
| Charger incl. transformer samples | $100–200 | $25–40 |
| Bridge, gate drive, relays | $350–550 | $200–330 |
| Discharge load | $20–40 | $15–25 |
| Battery (2 cells), gauge, charger | $45–70 | $22–35 |
| Dock contacts, device side, including data contacts | $7–20 | $3–7 |
| Front end + protection | $60–100 | $30–50 |
| Controllers, flash, sensing chains (no Bluetooth module) | $50–85 | $28–51 |
| Temperature sensors and heat-indicator labels (D-48) | $5–15 | $2–5 |
| User interface | $50–80 | $25–40 |
| Boards (HV-rated, 4-layer), small batch | $300–800 per batch | $20–40 |
| Enclosure (3D print or machined; tooling later) | $300–1,500 | $15–40 (after $10–40k tooling) |
| Pads (commercial replacements, for fit and impedance only) | $50–80 | — |
| **Parts for one first prototype** | **about $1,517–4,095** | **about $481–898 per unit** |
| One-off design and qualification charges for made-to-order parts (capacitor, transformer, gate-drive isolation, pad connector) | $2,800–9,500 | — |
| Charging dock with data link to a computer (not carried) | $60–170 | $18–35 |
| Vehicle pouch (D-48; not carried) | $10–30 | $5–15 |

Test equipment and engineering time are **not** included.

### Power budget [EST unless marked]

| State | Draw | Notes |
|---|---:|---|
| Standby (worn, ready; no Bluetooth) | 0.45–0.9 mW | Supervisor awake periodically, gauge, regulators. Version 0.3 included 30–90 µW for Bluetooth advertising |
| Temperature logging (D-48), in every state | 0.01–0.02 mW | §8A, R-H2: about 0.24–0.48 mWh a day |
| Weekly full-energy self-test (D-15), averaged | 0.42 mW | [CALC: 253 J ÷ 7 days] |
| Daily low-voltage check (D-15), averaged | about 0.17 mW | [CALC: about 15 J a day: about 2 J into the capacitor at 10% voltage, plus electronics awake and a battery load test, EST] |
| **Standby including self-tests and logging** | **1.05–1.5 mW: 0.025–0.036 Wh a day** | [CALC]. Lithium-ion self-discharge (about 2–3% a month [BK], faster when hot) is of the same order |
| Record download on the dock | powered by the dock | no battery cost |
| Rescue, not charging | 0.4–0.6 W | |
| Charging | 44–50 W for 7 s | §2 |
| Rescue average with D-29 | about 2.5–2.8 W | 30 min ≈ 1.3–1.4 Wh |

Off the dock, a full 20 Wh pack would cover standby, self-tests and logging for many months and still hold enough for several long rescues [CALC: 0.04 Wh a day plus self-discharge, against 1.3 Wh per long rescue]. The device should still go on the dock every shift, which is also when its records and heat history are read off.

---

## 10. Ranked risks and unknowns this study leaves open

| Rank | Risk or unknown | Why it matters | What would resolve it (later, under the lab safety policy) |
|---:|---|---|---|
| 1 | **Capacitor: density, mass and hot-working life at 2.2 kV** | Sets about 30% of the mass; spread between low and high cases is 130 g; the low case depends on a density no maker publishes; the working-temperature penalty is this study's scaling, not a maker's curve | Quotes from 3–4 makers against Appendix A (60 °C working, 2,500 charges), with their density, mass, life model and test evidence |
| 2 | **The 700 g target** | The lower-risk set is 19% over (832 g); the smallest set meets it only with the low or expected capacitor and smallest blocks | The no-decision levers in §9; maker quotes |
| 3 | **Pads' and cell's real heat limits for this device** | They set the envelope (§8A), but the project's pads do not exist yet and Molicel publishes no short-term storage figure for the P28A | Ask Molicel; write the pad specification to the envelope; test pads and cells at 60 °C |
| 4 | **Heat-damaged battery in service** | A cell aged or swollen by heat is the realistic lithium-ion hazard | The R-H3 battery health check; capacity, resistance and thickness tests after hot storage (§5) |
| 5 | **Front end under the charger and after the shock** | The analysis runs during charging (D-29) | Measure ECG and impedance noise with the charger running, first at ≤ 50 V output |
| 6 | **Switch pulse ratings at milliseconds** | No datasheet states them | Single 10 ms, 65 A pulses into 25 Ω; then 20 shocks in 1,300 s |
| 7 | **Gate-drive insulation at 2.2 kV working** | No catalogue part qualifies | Appendix B; partial-discharge and humidity tests on the delivered parts |
| 8 | **Battery in the cold** | 2 × 18650 just manages −20 °C in the pessimistic case | Charge time and minimum cell voltage at −20 °C from 10% charge, packs B and C |
| 9 | **Spacing required by the standard at 2 kV** | Could push the packing factor below 0.5 | Buy IEC 60601-1 and 60601-2-4 |
| 10 | **How fast the device really heats in a car** | The 25-minute time constant is still-air arithmetic; sunlight on the case is not modelled | Log a dummy device (no high voltage) in a parked car beside air and dashboard sensors |
| 11 | **Voltage sensors over temperature and life** | D-28 is tight against drift | Both chains against a calibrated reference at three voltages and three temperatures |
| 12 | **Internal heating in a long rescue at 50 °C** | Capacitor at about 54 °C against its 60 °C charging limit: about 6 K of margin. If it is exceeded, the device pauses charging (D-51) | Thermocouples during 14 dumps at 132 s spacing in a 50 °C chamber |
| 13 | **Discharge-resistor repeat rating and 5,000-pulse life** | The EY part is marketed for infrequent pulses | Maker's statement; then 100 dumps at 132 s spacing |
| 14 | **1.5 m drop with a 300 g capacitor** | Lead or mount failure | Drop test with a mass-equivalent dummy capacitor (no HV) |
| 15 | **Bluetooth module approval conditions (later, D-45)** | Reusing the approval depends on following the grant's conditions, including antenna clearance in the reserved space | Read the FCC grant and integration guide of the chosen module before the enclosure is fixed |
| 16 | **Ferrocene scope detail** | CPU option and library certification scope | Ask Ferrous Systems for the 26.11.0 qualification report |

---

## 11. Questions for the owner

### Answered on 27 November 2026

| Question | Answer | Recorded as |
|---|---|---|
| Car heat (Q-23) | Made-to-order capacitor; storage first 85 °C, then lowered to about 70 °C | D-41 |
| Hot cars beyond the capacitor (version 0.2 question E; Q-37) | Left in a vehicle for short periods, not on a dashboard; the device records its own temperature | D-48 |
| K. Which heat envelope? (Q-43) | **Option A plus an insulated pouch kept in the vehicle** (version 0.3's option B): ready to 60 °C for up to 8 h at a time; "not ready" 60–70 °C; "service needed" above 70 °C; battery charging 0–45 °C | D-48 |
| L. Capacitor working temperature (Q-44) | **Charged only up to 60 °C** | D-41 |
| M. Rescue with a hot device (Q-45) | **Guides CPR and analyses; does not charge until back inside the limits**, then shocks as normal | D-51 |
| N. Pad heat allowance (Q-46) | **The device totals pad time above 50 °C and calls for new pads after about 7 days**, the figure to come from the pad maker | D-52 |
| Q-36. The 700 g target | **700 g with pads is the goal for the finished design; a first prototype of 780–900 g is accepted** | D-43 |
| Q-38. How heavily is one device used? | **Sized for 2,500 full charges** (typical use); the device counts charges and calls for service; a heavy-duty capacitor for busy services is a later option | D-41, D-53 |
| Q-40. Bluetooth module firmware | **No Bluetooth in the first prototype**; records by cable through the dock; space kept for a module | D-09 (D-45 deferred) |
| Off-the-shelf parts only (Q-24) | Catalogue where possible; made to order with a published specification where not | D-42 |
| Voltage margin (Q-25) | 2,200 V parts, keep 100 µF | D-41 |
| Self-test frequency (Q-26) | Weekly full energy, daily low-voltage | D-15 |
| Cold-weather charging (Q-27) | Two cells | D-44 |
| Supervisor technology (Q-29) | Cortex-M33 class, different maker | D-39 |
| Does 700 g include pads? (Q-30) | Yes | D-43 |
| IP65 and charging (Q-31) | Sealed contacts and a dock | D-46 |
| J. Decisions that could be revisited for weight | **None needed now**: the lower-risk set is inside the accepted prototype range | — |

### Still open

- **Q-39. Battery size in the cold (after bench tests).** Two 18650 cells just manage a full-speed charge at −20 °C when nearly flat. If the cold tests show the margin is too thin: move to two 21700 cells (lower-risk set about 890 g, +58 g, still inside the accepted prototype range), or accept a slower charge when the battery is both cold and low?
- **A. Sensor accuracy over life (D-28).** 1% + 7 V holds at manufacture with good parts, but resistor drift uses about a third of the allowance over years, and time spent hot adds drift. Accept periodic recalibration at service, or specify costlier low-drift dividers?
- **B. MAX30001G's 0–70 °C range** rules out the most capable compression-sensing chip against the −20 °C target. Confirm that the full temperature range comes first?
- **H. Publishing the made-to-order specifications.** Appendices A and B are requirements only, not designs. May they be published in the public repository now, or held for the patent review (D-13)?

### New questions raised by this revision

- **O. When should the charge counter call for service?** D-53 says "as the capacitor approaches" 2,500. A suggestion: warn at 2,250 charges (90%), and show "service needed" at 2,500 [EST]. Weekly self-tests alone use 520 in ten years, so a heavy user reaches the warning in about five and a half years [CALC: 2,250 ÷ 412 a year]. Accept, or set other figures?
- **P. How do records come off the device?** D-09 says by cable through the dock contacts. Either (a) the dock is also the reader: it turns a simple one-wire link from the device into USB for a computer, so the device needs **one** extra contact; or (b) the device speaks USB itself through **two** extra contacts, and the dock only passes them through. (a) keeps the device simpler and better sealed; (b) keeps the dock simpler. Both are a gram or less. Which do you prefer, or should this wait for an engineer?

---

## What I could not verify

- **Pads:** the time limit on ZOLL's −30 to 65 °C short-term storage; the full Philips FRx storage table (only an extract of the 1-week, −20 to 60 °C figure was seen; the PDF manuals did not open); Schiller's pad storage range; the CellAED's storage range (not searched). Several pad and AED figures come from search-result extracts of maker documents, not the documents themselves; each is marked "extract" in §8A. Mass of an unpackaged pad pair.
- **Battery:** the P28A's and P45B's short-term storage maximum (Molicel's copy gives only "below 35 °C for shipping" and "below 23 °C long term"); whether the datasheet copies are official; the cold internal-resistance multiplier (3–5, background knowledge); LFP behaviour at −20 °C for this charger; the MAX17320's temperature ranges (not re-checked this pass).
- **Cars:** no published measurement of **trunk or glovebox** temperatures was found; no source for **cabin air of 70 °C** or a **dashboard of 100 °C** (the version 0.2 notes); the full text of the Vanos 2018, Grundstein 2009, Dadour 2011 and Marty 2001 papers (only abstracts and press releases were reachable); how long cabins stay near their peak (estimated).
- **Device heating:** the 25-minute time constant and the pouch's fourfold slowing are still-air arithmetic; heating by direct sun on the case is not modelled. No specific pouch product was checked against the H8 requirement.
- **Combined capacitor (version 0.4):** whether any maker can reach the low case (about 2.0 J/cc at 2.2 kV); CDE's "as high as 2 J/cc" claim gives no part, voltage or life; how close GA's 1.66 J/cc part sits to the film-thickness floor set by the 2,420 V test and self-healing margin; whether makers will trade life for density at all.
- **Data through the dock:** no specific sealed contact part with a data rating was checked; the choice of link (question P) is open.
- **Display and the rest:** the Sharp memory LCD's storage range; speaker and piezo heat ratings; enclosure plastic and gasket figures (background knowledge only); a specific passive heat-indicator label.
- **Capacitor:** any mass or temperature rating for a 2.2 kV defibrillator film capacitor beyond CRE's; a life-versus-temperature curve from any maker; whether GA's catalogue ratings are at 25 °C or at the working limit; dimensions of the CRE 200 µF / 2,200 V part. The voltage exponent (6.6) is fitted to one maker's family; published values run from 5 to 13.
- **Carried over from version 0.2:** the FCC IDs of the Ezurio BL54L15 and Raytac AN54L15Q; the BGM220S temperature range and serial co-processor mode; BQ25887 stock; the BQ28Z610's end-of-life status; a device-side sealed contact part number; prices and stock (distributor pages blocked automated access); switch pulse ratings at shock length; ADS1292R respiration resolution; ADAS1000 lifecycle; base MAX30001 temperature range; STM32U585, NXP MCX-A and Sharp memory LCD lifecycles; Ferrocene's `cortex-m33` CPU scope; spacing at 2 kV (standard not bought).

---

## Appendix A. Specification sheet: made-to-order energy-storage capacitor

**Status: draft requirements for a part to be purchased (D-41, D-42). Not a design.** It says what the part must do and what proof the maker must supply. How the maker achieves it is the maker's business. Figures marked [EST] are this study's targets and may move after maker feedback; figures marked [CALC] follow from the working in §1. **Version 0.4: working (charging) limit 60 °C and life ≥ 2,500 full charges (D-41, D-53); storage about 70 °C; a heavy-duty variant for busy services is a later option (A7).**

### A1. Electrical

| Item | Requirement | Basis |
|---|---|---|
| Type | Metallized polypropylene film, self-healing, for biphasic defibrillator pulse discharge | §1; what all compact defibrillators use |
| Capacitance | **100 µF ± 5%** at 25 °C, 1 kHz or 120 Hz. Each part's measured value reported on its label or test record | D-23, D-41. The firmware can use each part's measured value |
| Rated DC voltage (VR) | **≥ 2,200 V** continuous over the working temperature range | D-41; 10% above the 2,008 V worst case [CALC] |
| Working voltage | 1,950 V nominal (adult); 1,150 V (child); worst case 2,008 V | D-23; FINDINGS.md |
| Test voltage | 1.1 × VR (2,420 V) for 10 s, terminal to terminal, on every part | TDK's stated test practice for B32365 |
| Terminal-to-case insulation | ≥ 5 kV DC for 60 s, no breakdown [EST]; partial-discharge inception stated by the maker | [EST, to be checked against IEC 60601-1 once bought] |
| Charge-hold | Hold 2,008 V for at least 10 minutes in one event without damage; **≥ 60 h cumulative** at 2,008 V over life | [CALC: 2,500 charges × up to 30 s = 21 h, ×3 ≈ 60 h]; CRE quotes a "max 10 min" style rating |
| Peak discharge current, normal | **65 A** peak, falling exponentially, phase 1 then phase 2, total up to 15.5 ms per shock | DESIGN_TABLE (25 Ω patient) |
| Peak discharge current, fault | Survive **≥ 10** discharges at 2,008 V into a shorted output limited only by the device's own internal resistance (about 5 Ω): **about 400 A** peak | [CALC: 2,008 V / 5 Ω = 402 A]; GA's DQ series states "peak currents up to 300 A", so this must be confirmed |
| Current and voltage reversal | In normal use the **capacitor's** current does not reverse: the output bridge reverses the *patient* current, not the capacitor's. Voltage reversal at the capacitor ≤ 10% of the pre-discharge voltage in fault cases (ringing into a shorted output) | Biphasic truncated exponential (D-02); [EST] for the fault figure |
| Repetition | Up to **20 full-energy discharges at about 65 s spacing**, then one every 2 minutes indefinitely, at 50 °C ambient | IEC 60601-2-4 style "20 shocks in 1,300 s" check; D-29 cycle of 132 s |
| Internal loss (ESR) | Dissipation factor ≤ 0.010 at 120 Hz (ESR about 0.13 Ω at 100 µF) at 25 °C; ≤ 2× initial at end of life | [CALC: ESR = 0.01 / (2π × 120 × 100 µF) = 0.133 Ω]; TDK B32365 states tan δ < 0.010 |
| Internal inductance (ESL) | ≤ 200 nH [EST] | Irrelevant to millisecond pulses above this level; keeps fault ringing low |
| Leakage / self-discharge | Insulation resistance × capacitance ≥ 10,000 s at 25 °C and ≥ 1,000 s at 60 °C [EST] (for 100 µF: ≥ 100 MΩ and ≥ 10 MΩ) | Keeps leakage below the voltage-sensing divider's current; a charge is held for under a minute normally |
| Voltage recovery (dielectric absorption) | ≤ 0.2% of the pre-discharge voltage (about 4 V) 1 minute after discharge to below 10 V and opening the circuit [EST] | SAFETY.md: recovered charge is a hazard; polypropylene is typically far below this [BK] |
| Internal bleed resistor | None, unless agreed | The device has its own discharge path and backup bleeder |

### A2. Life

Usage profile (from §1), over a **10-year** service life. The part is sized for the **typical** user (D-41, D-53); the device counts full charges and calls for service before the limit.

| Use | Typical user | Heavy user (for information) |
|---|---:|---:|
| Weekly full-energy self-test (52 × 10) | 520 | 520 |
| Rescue charges (typical: 6 rescues a year × 10 charges × 10 years; heavy: 25 × 14 × 10) | 600 | 3,500 |
| Factory, acceptance and service | 100 | 100 |
| **Full-energy cycles over 10 years** | **1,220** | **4,120** |
| Daily low-voltage checks (365 × 10), at no more than about 10% of full voltage | 3,650 | 3,650 |

Requirements:
- **≥ 2,500 full-energy cycles** (charge to 2,008 V, hold up to 30 s, discharge as A1) **at 50 °C ambient**, with the part at up to its 60 °C working limit. [CALC: 1,220 × 2 = 2,440, rounded up to 2,500. The factor of two covers users busier than typical and the spread in any life test.]
- A heavy user would reach 2,500 in about 6 years [CALC: 4,120 ÷ 10 = 412 a year; 2,500 ÷ 412 = 6.1]. That is handled by the device's charge counter (D-53), which calls for service in time, and by the heavy-duty option (A7). It is not a reason to oversize this part.
- ≥ 4,000 low-voltage cycles (negligible wear expected; stated for completeness).
- End-of-life criteria: capacitance loss ≤ 5% (the conventional pulse-capacitor end-of-life point), dissipation factor ≤ 2× initial, no open circuit, no short circuit, no case deformation.
- The maker states the cycle life at 25 °C, 50 °C and 60 °C, and the model it uses to scale between them, so that the device's charge counter can be set from the maker's figures rather than this study's.
- Shelf life ≥ 10 years at 25 °C without re-forming.

### A3. Environmental

| Item | Requirement | Basis |
|---|---|---|
| Working temperature | **−20 to +60 °C** at the part. The device does not charge the capacitor above 60 °C (D-41, D-48, D-51) | 50 °C device rating + about 4 K internal rise in a long rescue (§4) = about 54 °C, leaving about 6 K of margin |
| Storage temperature, unpowered | **−40 to +70 °C** | D-41; the device's envelope puts its absolute limit at 70 °C (D-48, §8A) |
| Hot storage over life | ≥ **2,000 h cumulative at 70 °C**, unpowered, within end-of-life limits | [EST: 10 summers × 50 days × 4 h, all assumed at the 70 °C ceiling, which is conservative because most of the time will be cooler, and more so with the vehicle pouch] |
| Behaviour above the storage limit (information only, not a pass/fail requirement) | The maker states what happens after 24 h at 85 °C unpowered: capacitance, dissipation factor, insulation resistance, any case change | So the device's "service needed" rule (§8A) knows whether a hot-car event above 70 °C ends the capacitor's life |
| Temperature cycling | ≥ 1,000 cycles −40 / +70 °C, unpowered, within limits | [EST: about one hot-cold swing a day over three years of car storage] |
| Humidity | Climatic category 40/93/56 or better | TDK B32365's category; the device itself is sealed to IP65 |
| Mechanical shock | Survive the device's 1.5 m drop onto concrete: maker to state a shock rating; target ≥ 500 g, 1 ms half-sine, 3 axes, both directions [EST; final level from IEC 60601-1-12 once bought] | REQUIREMENTS drop target; [UNVERIFIED level] |
| Vibration | Vehicle and carried use: maker to state random-vibration rating; target to IEC 60601-1-12 emergency-services levels once bought [UNVERIFIED] | Patrol car, belt-worn |

### A4. Safety

- **Failure mode:** the maker states the end-of-life and fault failure modes. Required: failure by gradual capacitance loss or **open circuit**, not a short circuit, and **no case rupture, flame or ejection of material** when driven to failure at 1.3 × VR [EST test level].
- **Self-healing:** required; the maker states its clearing behaviour and how capacitance loss is limited. **A part made denser to meet the 2,500-cycle figure must not give up self-healing margin**: the maker states the film's working stress and shows that clearing still works at 1.1 × VR, 60 °C.
- **Pressure interrupter or equivalent:** the maker states whether the part has one, or demonstrates by test that internal faults cannot burst the case.
- **Materials:** case and fill flame-retardant (UL 94 V-0 or equivalent); RoHS and REACH declarations.
- **Leads:** insulated flexible leads rated ≥ 2 × VR, with strain relief and a stated pull strength. (TDK's standard lead is 15 kV DC, 150 °C silicone.)
- **Temperature sensing:** the part must allow a temperature sensor to be held against its case (a flat area or stated location), because the device reads the capacitor's temperature (§8A, R-H1).
- **Marking and traceability:** part number, capacitance, VR, date code and serial number; lot records kept for the life of the device. A medical-device customer will need this.

### A5. Mechanical envelope target

| Item | Target | Stretch |
|---|---|---|
| Volume | ≤ 170 cm³ | ≤ 145 cm³ |
| Mass | ≤ 240 g | ≤ 185 g |
| Shape | Rectangular or oval prism, about 33 × 55 × 95 mm or smaller [CALC: 172 cm³]; maker may propose | as small as the maker can show |
| Mounting | Suitable for clamping or potting; maker states allowable clamping force | — |

The target is this study's expected case and the stretch its low case (§1). The maker's figures replace both.

### A6. Test evidence to request from the maker

1. Datasheet and outline drawing with **measured mass and dimensions**, for the standard part and, if offered, the heavy-duty part (A7).
2. Life test report: cycles to the A2 end-of-life criteria at 25 °C, 50 °C and 60 °C, with the discharge conditions used.
3. Unpowered storage report: capacitance, dissipation factor and insulation resistance before and after ≥ 1,000 h at **70 °C**; and the 24 h, 85 °C information test (A3).
4. Temperature cycling (−40 / +70 °C) and damp-heat results.
5. Fault-discharge (shorted-output) test results at the A1 fault current.
6. Destructive end-of-life test showing the failure mode (A4), and the self-healing evidence at 1.1 × VR, 60 °C.
7. Dielectric-absorption (voltage recovery) measurement.
8. Mechanical shock and vibration results.
9. Quality system certificate (ISO 13485 preferred, ISO 9001 minimum), a commitment to notify product changes, and a statement of how long the part will be made.
10. Ten qualification samples with individual test records.

### A7. Later option: heavy-duty variant for busy services (D-53)

Not needed for the first prototype. Recorded so makers can say whether they could offer it **in the same outline**, which would let a busy service fit it without a different enclosure.

| Item | Heavy-duty requirement | Basis |
|---|---|---|
| Life | **≥ 5,000 full-energy cycles** at 50 °C ambient, 60 °C working | [CALC: heavy profile 4,120 × 1.2, rounded] |
| Charge-hold | ≥ 100 h cumulative at 2,008 V | [CALC: 4,120 × 30 s ≈ 34 h, × 3] |
| Everything else | as A1, A3, A4 | — |
| Expected size | about 190 cm³ / 270 g [CALC, §1] | about +36 g on the finished device (§9) |

---

## Appendix B. Requirement lists for the other made-to-order parts

Requirements only, as in Appendix A. No construction details. Unchanged from version 0.2 except the pad connector's storage temperature (B3), which followed the capacitor's, and the pointer in B4.

### B1. Charging transformer

- **Job:** step the 2-cell battery voltage up to charge the capacitor to 1,950 V (adult) or 1,150 V (child) within 7 s.
- **Input:** 2-cell lithium-ion pack, 5.5–8.4 V under load (including cold sag, §5).
- **Output:** charges the capacitor to 2,008 V worst case; output insulation for **≥ 2,200 V working** continuously.
- **Power:** ≥ 35 W delivered for 7 s per charge [CALC, §2]; duty of 14 charges at 132 s spacing, and 20 charges in 1,300 s, at 50 °C ambient.
- **Losses:** the maker states efficiency and temperature rise at that duty; transformer losses target ≤ 5% of throughput [EST].
- **Isolation:** input to output dielectric test ≥ 4.4 kV DC for 60 s [EST: 2 × working]; partial-discharge inception ≥ 1.5 × 2,200 V [EST]; spacing to IEC 60601-1 once bought.
- **Temperature:** −20 to +70 °C ambient; insulation class B (130 °C) or better.
- **Size target:** ≤ 15 cm³ and ≤ 30 g [EST, §2].
- **Evidence to request:** efficiency and temperature-rise report, insulation and partial-discharge results, humidity-soak insulation test, samples.

### B2. Gate-drive isolation (signal and power for the output switches)

- **Channels:** one per switch that needs driving (four for an all-SiC bridge).
- **Working voltage across each barrier:** **≥ 2,200 V DC continuous for 10 years**, rated with margin (target ≥ 2,750 V working [EST: 1.25 ×]).
- **Transient immunity:** withstand the voltage swings of the shock and of the bridge switching without false turn-on; common-mode transient immunity ≥ 50 kV/µs [EST].
- **Gate power:** enough to hold a 3.3 kV SiC switch on for up to 20 ms (gate voltages per the switch datasheet, for example +20 / −5 V).
- **Timing:** turn-on and turn-off delay ≤ 1 µs, channel-to-channel mismatch ≤ 200 ns [EST]; phase ends are set in milliseconds, so this is ample.
- **Fail-safe:** loss of drive power or signal turns the switch **off**.
- **Insulation class:** whether this barrier counts as a means of patient protection under IEC 60601-1 is to be settled when the standard is bought.
- **Size target:** ≤ 3 cm³ per channel [EST].
- **Evidence to request:** partial-discharge results at 1.5 × working, insulation test after humidity soak, lifetime rating at working voltage.

### B3. Pad connector (device side and pad side)

- **Voltage:** carries the 2,008 V shock; insulation ≥ 5 kV DC between contacts and to the outside [EST, version 0.1].
- **Current:** 65 A normal peak, ≥ 100 A for 20 ms without damage [EST].
- **Contact resistance:** ≤ 50 mΩ, and **stable to within 10 mΩ** under vibration and during chest compressions, because the device reads compressions as 0.3 Ω changes in pad resistance (D-36) [CALC: one tenth of the signal].
- **Sealing:** IP65 on the device side both mated and unmated.
- **Mating cycles:** ≥ 500 [EST: pad replacement every 2–4 years plus inspections].
- **Polarised, with a pad-present sense**, so the device knows pads are connected.
- **Temperature:** working −20 to +65 °C; storage **−40 to +70 °C** (to match Appendix A; version 0.2 said 85 °C).
- **Mass target:** ≤ 10 g device side [EST].
- **Evidence to request:** dielectric test, current pulse test, contact-resistance stability under vibration, ingress test, cycling test.

### B4. Pads (noted, not specified here)

D-42 also lists the pads as made to order. Pads are a regulated product in their own right. A separate study should set gel area (55–70 cm² each, D-05), lead length for front-and-back child placement, shelf life, and the storage temperature. **The pads set the device's heat limits** (§8A): the pad specification must match the envelope the owner chooses (question K), which on commercial evidence means about 35–40 °C long term and 60 °C (option A) or 65 °C (option C) for short periods.
