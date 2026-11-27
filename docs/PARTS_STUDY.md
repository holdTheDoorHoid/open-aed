# Parts study and size, weight and power budget

Version 0.2, 27 November 2026. Revised the same day as version 0.1, after the owner answered the questions version 0.1 raised (decisions D-15, D-39 and D-41 to D-46). This is a paper study from datasheets, maker pages and distributor listings, checked on 27 November 2026. **No part has been bought, measured or tested. No hardware exists.** Working notes and every source link are in [research_notes/Open source belt worn AED/parts_study_sources.md](../research_notes/Open%20source%20belt%20worn%20AED/parts_study_sources.md). Decision numbers (D-xx) refer to [DECISIONS.md](../DECISIONS.md).

> **What this document is not.** It names kinds of parts, specific part numbers and their published ratings, and does sizing arithmetic. The appendices are *requirement sheets for parts to be bought*: what the part must do and what proof to ask the maker for. They say nothing about how to make the part. This document contains no schematics, wiring, board layouts, winding details or assembly steps, and nothing in it is an instruction to build or power anything above 50 volts (D-10, SAFETY.md). Nothing here is safe or ready for use on a person or animal.

How to read the tags:
- **[CALC]** is arithmetic done for this study. The working is shown.
- **[EST]** is an engineering estimate. Its basis is stated.
- **[BK]** is background knowledge from training that was not re-checked this time. Treat it as something to check, not a citation.
- **[UNVERIFIED]** means I looked and could not confirm it.
- **"Checked 27 Nov 2026"** means the listing or page was seen that day.

Most distributor pages (Digi-Key, Ohmite, TME) refused automated access on the day. Most prices below are therefore **estimates**, not quotes. They must be re-checked by hand before anything is ordered.

---

## What changed in version 0.2

| Decision | What it means for this study | Where |
|---|---|---|
| D-41: made-to-order capacitor, ≥ 2,200 V, 85 °C storage, about 100 µF | The capacitor is re-sized for those ratings, with a low, expected and high estimate. A specification sheet for it is drafted. | §1, Appendix A |
| D-42: catalogue parts where they exist, made-to-order with a published specification where they do not | Requirement lists for the other made-to-order parts. | Appendix B |
| D-43: the 700 g includes the pads | The roll-up is judged with pads in. | §9 |
| D-44: two battery cells | Wired end to end ("in series"); cold-weather arithmetic redone. | §5 |
| D-45: Bluetooth on a separate pre-certified module | Module candidates with their published approvals. | §7 |
| D-46: sealed contacts and a dock | Replaces the USB-C socket. | §5 |
| D-39: two Cortex-M33-class processors from different makers | Confirms the version 0.1 recommendation. | §7 |
| D-15: full-energy self-test weekly, low-voltage check daily | Capacitor and discharge-load wear and standby energy recomputed over a 10-year life. | §1, §4, §9 |

The version 0.1 roll-up is kept in §9 as "before the 27 November decisions", so the cost of each decision can be seen.

---

## Summary for the owner

- **The 700 g / 0.7 L target is not met by the set of parts that follows your decisions.**
  - With the expected capacitor, that set comes to about **0.89 L and 900 g with pads**. That is about **200 g (29%) and 0.19 L (27%) over**.
  - The smallest plausible set that still follows every decision comes to about **0.63 L and 697 g with pads**. It meets the target, but only by 3 g, and only if the capacitor comes in at the *low* end of its estimate. With the expected capacitor the smallest set is about 784 g.
  - **Plainly: a 700 g device is possible on paper only if almost every part lands at its optimistic value.** A first prototype is more likely to weigh 780–900 g. For comparison, the St John Pocket AED P3 is under 700 g with pads and the Schiller FRED easyport plus is 780 g.
- **What the decisions cost.** Against version 0.1, the decisions add about **70 cm³ and 68 g** to the lower-risk set and about **62 cm³ and 53 g** to the smallest set. Almost all of it is the heavier capacitor (D-41) and, in the smallest set, the second cell (D-44). Bluetooth on its own module, two M33 processors and dock contacts cost only a few grams each. The table is in §9.
- **The capacitor (D-41).**
  - A made-to-order film capacitor of 100 µF, rated 2,200 V, with 85 °C storage and 65 °C working, is estimated at **170 / 210 / 270 cm³ and 225 / 300 / 410 g** (low / expected / high). Version 0.1 budgeted 180 cm³ and 250 g.
  - Confidence: moderate on volume (it is scaled from a published family of real parts, including two rated 2.2 kV), low on mass (only one maker publishes a mass, and none publishes a temperature-versus-life curve).
  - Its draft specification sheet is Appendix A. It asks for **at least 5,000 full-energy cycles at 50 °C**, from a stated usage profile.
- **New problem found: the capacitor is not the only part that dislikes a hot car.** Lithium-ion cells and pad gel are both usually limited to about 50–60 °C storage [BK]. An 85 °C capacitor does not on its own make the device survive a dashboard. This is a new owner question.
- **Two cells, in series.** Series halves the current in the wiring, switches and charger. The cold arithmetic says two high-drain 18650 cells **just** manage a full-speed charge at −20 °C from a nearly flat battery; a single cell does not. If the cold tests disappoint, two 21700 cells would add about 50 g more.
- **Bluetooth module.** Three current pre-certified modules fit: Ezurio BL54L15, Raytac AN54L15Q (both Nordic nRF54L15) and Silicon Labs BGM220S. Several older u-blox modules that ran a ready-made serial firmware are being withdrawn. The module adds about 1 g and under 1 cm³.
- **Self-test (D-15).** Over 10 years the self-test now accounts for about 520 full-energy cycles, against 600 (typical user) to 3,500 (heavy user) from rescues. Daily low-voltage checks cause practically no wear. Standby including self-tests uses about 0.03–0.04 Wh a day.

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

---

## The design point this study sizes against

From [DESIGN_TABLE.md](../firmware/crates/aed-waveform/DESIGN_TABLE.md) (model, not measurement), with the D-41 rating:

| Item | Adult | Child | Source |
|---|---:|---:|---|
| Capacitor | 100 µF | 100 µF | D-23, D-41 |
| Capacitor rating | at least 2,200 V | | D-41 |
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
- Survive storage at 85 °C in a parked car (D-41), and work in a device at up to 50 °C outside.

### Catalogue and quote-only parts found in version 0.1

Kept for reference. Checked 27 Nov 2026. "Stored at 1,950 V" is [CALC] ½CV². None of these meets D-41 as listed: the compact film parts are rated 2.0 kV and none has a trustworthy 85 °C storage rating; the electrolytic options are not pulse parts.

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

General Atomics (GA) also lists two biphasic parts **rated 2.2 kV**, both 196 µF: 322DQ4196B (1.20 J/cc, 5,000 cycles, 49 × 74 × 129 mm) and 30920 (1.05 J/cc, 10,000 cycles, 49 × 74 × 146 mm). They are about twice the capacitance needed and twice the size. They matter here because they show what a 2.2 kV rating costs in density. CRE lists a 200 µF, 2,200 V part with 85 °C storage; it publishes no size or mass for it.

### What D-41's capacitor will cost in size and weight

#### Scaling law used

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

4. **Heat.** Two separate things:
   - **Storage at 85 °C, with no voltage.** Polypropylene film capacitors are commonly rated to 85 °C or 105 °C, with voltage derating of about 1.0–1.25% per °C only *above* 85 °C (Panasonic and other film datasheets, checked 27 Nov 2026). So unpowered storage at 85 °C is at the edge of the film's ordinary range, not beyond it. The risk lies in the filling, the case and slow shrinkage of the film, which a maker must prove by test (Appendix A). **Cost in density: small [EST].**
   - **Working at 65 °C instead of 55 °C.** The defibrillator parts found are rated to 55 °C working (TDK) or 60 °C (CRE). A device at 50 °C outside warms by about 4 K in a long rescue (§4), so the capacitor needs about 65 °C working to have margin. Film capacitor life is generally taken to halve for every 10 °C rise (Digi-Key technical forum) [EST that this holds for pulse duty]. Recovering a factor of two in life costs about **19% of energy density** [CALC: stress × 0.5^(1/6.6) = 0.90; density × 0.90² = 0.81].

5. **Box versus active volume.** GA's quoted J/cc is 15–18% higher than its parts' outside-box figures [CALC: e.g. 320DQ4130 stores 260 J rated in a 179.7 cm³ box = 1.45 J/cc, against 1.66 quoted]. The estimates below add **18%** for the can.

6. **Mass.** Only one defibrillator film capacitor has a published mass (CRE, 300 g for a 250 J part). This study uses **1.3–1.5 g/cm³** [EST: polypropylene film is about 0.9 g/cm³; an aluminium can and resin fill raise it], and checks against CRE's 0.83 J per gram [CALC: 250 J / 300 g].

#### Low, expected and high estimate [CALC from the scaling above; EST inputs as marked]

| | Low | Expected | High |
|---|---|---|---|
| Basis | GA's densest film (1.66 J/cc) meets D-41 with no temperature penalty, because the maker's standard film already holds 85 °C storage and 65 °C working (as CRE claims for its parts) | GA's densest film, less 19% for 65 °C working: **1.35 J/cc**. This happens to equal GA's real 2.3 kV, 5,000-cycle grade | GA's real 2.2 kV, 10,000-cycle grade (**1.05 J/cc**), in case the hot-working life needs it |
| Active volume (242 J rated ÷ density) | 146 cm³ | 180 cm³ | 230 cm³ |
| **Boxed volume** (+18%) | **about 170 cm³** | **about 210 cm³** | **about 270 cm³** |
| **Mass** | 1.3 g/cm³: **about 225 g** | 1.4 g/cm³: **about 300 g** (CRE's 0.83 J/g gives 290 g) | 1.5 g/cm³: **about 410 g** |
| Change from version 0.1's 180 cm³ / 250 g budget | −10 cm³ / −25 g | **+30 cm³ / +50 g** | +90 cm³ / +160 g |
| Change from version 0.1's smallest-set 150 cm³ / 210 g | +20 cm³ / +15 g | +60 cm³ / +90 g | +120 cm³ / +200 g |
| Life at 1,950 V | about 5,500 cycles at 25 °C [CALC: 2,500 × 2.22], less when hot | about 5,500 cycles at the working limit [CALC] | over 10,000 |

**How confident.** Moderate on volume: the scaling is anchored on five real parts from one maker, two of them at 2.2 kV, and the expected case equals a real GA density. Low on mass: one maker's figure, and no maker publishes a mass for a 2.2 kV part. Lowest on the temperature penalty: no defibrillator capacitor maker publishes a life-versus-temperature curve; TDK refers to one that is not in its datasheet. **The range will only narrow when makers quote against Appendix A.**

**Shape.** A box of about 33 × 60 × 105 mm holds the expected 210 cm³ [CALC: 207.9 cm³]. That is the same footprint as the GA 320DQ4130, a little longer.

**Price [EST].** Version 0.1 estimated $150–400 for a sample and $80–200 at qty 100. A 2.2 kV rating and an 85 °C qualification test add perhaps 20–50% [EST, from custom-part norms]: **$200–600 for samples, $100–250 at qty 100**, plus a one-off qualification charge of perhaps $1,000–3,000 [EST].

### Temperature: what still does not survive a parked car

- **Cabin air, 47–70 °C, and dashboard, 70–100 °C.** D-41's capacitor is specified for storage to 85 °C. That covers cabin air with margin and the cooler end of a dashboard. **Nothing covers 100 °C.**
- **New finding: the battery and the pads are now the weak links.**
  - High-drain lithium-ion cells are usually rated for storage to about 60 °C for short periods, and their makers advise much cooler for long storage [BK; P28A storage range not confirmed]. Hours at 85 °C would cost permanent capacity and could swell a cell [BK]. Charging is blocked above about 45–60 °C anyway.
  - Defibrillator pad gel dries faster when hot. Commercial pads are typically labelled for storage up to about 43–50 °C [BK; not checked in this pass].
  - So an 85 °C capacitor protects the most expensive part, but **the device as a whole is still limited to roughly 50–60 °C storage by its battery and pads**. A "not on the dashboard" rule is still needed. See the new owner question in §11.
- **What heat does to electrolytics.** Kept from version 0.1: only the 105 °C electrolytics have a rating above 70 °C, and they are the heaviest and least suited to pulse duty. Not recommended.

### Operating heat inside the device

- At a 50 °C ambient, a long no-shock rescue warms the inside of the box by about 4 K (§4), more near the discharge resistors.
- Appendix A therefore asks for **65 °C working**. That gives about 10 K of margin over the 54–55 °C the capacitor would reach.

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

- **Requirement:** at least **5,000 full-energy cycles at 1,950 V and 50 °C ambient** (the heavy profile plus about 20%), and 100 h of cumulative holding at 2,008 V. The expected-case part above meets it on paper.
- **For comparison:** version 0.1's twice-daily full-energy policy would have used about 7,300 cycles in 10 years on self-tests alone [CALC: 730 × 10], forcing the high, 10,000-cycle case. D-15 is what keeps the capacitor at the expected size.

### Self-discharge, recovery and failure

- **Film** holds charge well and needs no re-forming. It recovers a little voltage after being emptied: TDK warns of "self-recharging phenomena". It usually fails gently, by self-healing and slow capacitance loss [BK]. Appendix A asks the maker to *prove* that.
- **Electrolytics** leak more, recover more charge after emptying, can fail short or vent, and need balancing in series.

### Recommendation

- **Order a single film capacitor to the specification in Appendix A** from defibrillator-capacitor makers: GA, CDE/Knowles, TDK, and CRE as a low-cost check. Ask each for a quote against the same sheet.
- **Budget 210 cm³ and 300 g** (the expected case), and carry the low and high cases in the roll-up.
- Keep the 105 °C electrolytic bank only as a heat fallback, not for size.

### What must be tested to confirm it

Later, under the lab safety policy. Appendix A lists the proof to ask the maker for; the project's own checks would be:
- Capacitance, leakage and internal loss before and after 1,000 h at 85 °C, unpowered.
- 1,000 full-energy cycles at 50 °C ambient, measuring capacitance loss and any rise in internal loss.
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

Unchanged from version 0.1 except that the gate-drive isolation now has a requirement list (Appendix B).

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
- **After 30 minutes the inside is about 3.8 K above ambient**: about 54 °C at a 50 °C ambient. Appendix A's 65 °C working rating covers this with about 10 K margin. A heavier device (§9) warms slightly more slowly.

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
- If cold tests show the 18650s sag too far, the firmware can slow the charge when the battery is cold and low, so the shock follows a few seconds after the analysis. That is a fallback, not a plan; see the new owner question in §11.
- Charging the *battery itself* is a different matter: the cells must not be charged below 0 °C (P28A rating: charge 0–60 °C). A device docked in a freezing car will not recharge until it warms.

### Charges per battery charge [CALC]

2 × P28A: 2 × 2.8 Ah × 3.6 V = 20.2 Wh. New, room temperature: 20.2 × 0.9 / 0.0703 Wh per charge ≈ **259 charges**. Cold and aged (about 60% usable): ≈ **172**. Far above the 30-shock requirement.

### Protection and fuel gauge for a 2-cell pack

A 2-cell pack needs: a protector that can cut the pack off for over-voltage, under-voltage, over-current, short circuit and temperature on *each* cell; a fuel gauge that the self-test can trust under load; and cell balancing. Candidates, checked 27 Nov 2026:

| Part | What it is | Status |
|---|---|---|
| **ADI MAX17320** | 2-to-4-cell fuel gauge with protector, programmable thresholds, internal self-discharge detection, SHA-256 battery authentication | **Recommended for new designs** (ADI page) |
| TI BQ28Z610 / BQ28Z620 | 1-to-2-cell gauge with protection and cell balancing; the BQ28Z620 is TI's newer version | BQ28Z620 **ACTIVE**; TI flags a newer version of BQ28Z610 |
| TI BQ25887 (charger) | 2-cell boost charger from a 5 V input, 2 A, integrated cell balancing, battery thermistor with JEITA temperature rules, −40 to 85 °C | Listed on TI's site; stock not confirmed |

Implications of 2 cells:
- The protector's switches must carry about 7.6 A average and 15 A peaks while charging the capacitor, with low loss [CALC, §2]. Half the single-cell current.
- The gauge's per-cell readings let the self-test catch **one weak cell**, which a single pack voltage would hide. That serves the "battery tested under load" part of D-15.
- The MAX17320's self-discharge detection and authentication would let the device refuse a counterfeit or damaged pack. The recorded fatal-failure history in REQUIREMENTS makes that worth having.
- The pack (and the cells) need UN 38.3 transport testing; a device shipped with its battery inside goes as UN 3481 [BK].
- High-drain cells are typically rated about 500 cycles to 80% capacity [BK]. Charged daily, plan on battery replacement every 2–3 years.

### Charging through sealed dock contacts (D-46)

- **How it works, in requirement terms.** The dock takes ordinary USB-C power and presents it to the device through two or three exposed contacts (power, return, and optionally a "docked" sense). The device steps it up to charge the pack. Data goes over Bluetooth, so the dock carries no data.
- **Parts.** Sealed spring-contact connectors are catalogue items: Glenair's CB series pogo-pin connectors (its test report found no water inside an unmated panel receptacle), ATTEND Technology's IP67 waterproof pogo connectors, and Yokowo's IP67/IP68/IP69K range. One magnetic pogo connector datasheet warns that the *mated* joint is not waterproof; that does not matter here, because charging happens in the dock, not in the rain.
- **Which side gets the springs.** Putting the springs in the dock and plain flat contact pads on the device leaves the device with no moving parts and no holes, which is the easier way to hold IP65 [EST].
- **Requirements on the device side:**
  - The contacts must be **dead** when the device is off the dock: no battery voltage on them, so a wet glove or a key across them shorts nothing.
  - Withstand reversed or excess dock voltage, and static discharge, without damage.
  - Corrosion-resistant plating; sweat and road salt are the realistic exposure [EST].
- **Size and mass, device side [EST]:** about 2 cm³ and 5 g including alignment magnets and input protection, against about 1 cm³ and 2 g for a USB-C socket and its seal. **D-46 costs about 1 cm³ and 3 g**, plus a dock that is not carried (about $50–150 for prototypes, $15–30 at qty 100 [EST]).

### Candidates

| | A. 1 × Molicel P45B 21700 | **B. 2 × Molicel P28A 18650 in series** | C. 2 × P45B 21700 in series | D. 2-cell high-rate pouch pack |
|---|---|---|---|---|
| Fits D-44 | **no** | **yes** | yes | yes |
| Energy | 16.2 Wh | 20.2 Wh [CALC: 2 × 2.8 Ah × 3.6 V] | 32.4 Wh | about 11–16 Wh [EST] |
| Current per cell, 44–50 W charge, cold | 15–21 A | 8–9 A | 8 A | 8–9 A |
| −20 °C, 10% charge (table above) | fails in the pessimistic case | **just works** | works with margin | unknown |
| Mass of cells | 70 g | **92 g** (46 g typical each, datasheet copy) | 140 g | about 60–90 g [EST] |
| Pack with gauge, protection, charger, holder | about 75 g / 28 cm³ | **about 97 g / 38 cm³** | about 148 g / 56 cm³ [EST] | about 70–100 g [EST] |
| Price, cells [EST] | about $6–8 | about $10 / $7 for two | about $12–16 | about $15–30 |
| Main risk | Voltage collapse at −20 °C | Thin cold margin | Mass | Quality and traceability |

### Recommendation

**Option B (2 × P28A-class 18650 in series)** for both sets, with **MAX17320**-class protection and gauge and a **BQ25887**-class charger, charged through sealed dock contacts. Option C is the fallback if cold tests fail, at about 50 g and 18 cm³ more.

### Test

Charge time to 1,950 V and lowest cell voltage, from 10% charge, at −20 °C after a 4-hour cold soak, for packs B and C.

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

## 7. Controllers and Bluetooth

### Processing need

About 120,000 multiply-adds per second and about 10.5 KB of RAM for the analysis: under 1% of a 100 MHz Cortex-M with a floating-point unit [CALC]. The event record (1 KB/s, 1.8 MB per 30 minutes) calls for an external flash chip of 16 MB or more [CALC; e.g. Winbond W25Q128 class, BK].

### Ferrocene qualification (unchanged from version 0.1)

Ferrous Systems' documents list `thumbv8m.main-none-eabi(hf)` (Cortex-M33/M55/M85) and `thumbv7em` (Cortex-M4/M7) as **qualified** targets, the former since release 26.11.0. Cortex-M0/M0+ is only "supported". Caveats: the pages are development-branch previews; the signed qualification report was not seen; whether `-C target-cpu=cortex-m33` itself is in the qualified envelope is [UNVERIFIED]. Source links are in the research notes.

### Main controller and supervisor (D-39 confirmed)

| Role | Choice | Why |
|---|---|---|
| Main controller | **ST STM32U585-class** (Cortex-M33, 160 MHz, FPU), Rust via embassy-stm32 | Qualified target; no radio code on it (D-24, D-45) |
| Supervisor | **NXP MCX A-series** (Cortex-M33), Rust; HAL listed in the Embassy README | Qualified target; different maker, silicon and tools from the main controller |

Cost of D-39 against a Cortex-M0+ supervisor: **about 0 g and 0 cm³**, about $1–3 more per unit [EST]. Lifecycle of both families is [UNVERIFIED this pass].

### Bluetooth module (D-45)

**What it must do.** Handle all Bluetooth, on its own processor, and talk to the main controller over a simple serial link (a two-wire UART). It never has any path to the shock circuit. The main controller treats everything arriving from it as untrusted input.

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

**Warning about ready-made serial firmware.** The modules that used to come with a ready-made "AT-command" serial firmware are being withdrawn: u-blox issued an end-of-life notice for NINA-B31x on 16 February 2026, and the ANNA-B112-00B has been replaced. Plan for **project-written module firmware** that does only three things: advertise readiness, hand over event records, and accept configuration that the main controller then checks.

**What "modular approval" does and does not cover.** The module's radio approval can be reused only if the product follows the grant's conditions (antenna, layout keep-out, power). The finished device still needs its own emissions testing and the medical electromagnetic-compatibility standard (IEC 60601-1-2) [BK].

**Recommendation:** **A (Ezurio BL54L15)** for its published approval list and same-family Rust tooling; **C (BGM220S)** as the smallest, with a confirmed FCC grant.

**Size, mass and power [EST]:** module 0.2–0.3 cm³ and about 1 g, plus about 1–2 cm³ of board kept clear for the antenna. Advertising every 1–2 s averages roughly 10–30 µA at 3 V (30–90 µW) [BK], already inside the standby figure in §9. **D-45 costs about 1–2 cm³ and 1–2 g** over a single-chip radio.

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

- **Verdict:** achievable with the right resistor class, but only just in the worst case over life. Drift is the largest term; board contamination is the hidden one (at 10 GΩ leakage it alone is 0.4%).
- **The two channels should use different resistor and reference families**, so they cannot drift together and still agree.
- **Test:** both chains against a calibrated HV meter at 200, 1,150 and 1,950 V; at −20, +25 and +70 °C; and after 1,000 h at 85 °C and 85% humidity unpowered.

---

## 8. User interface and the rest

| Item | Target and basis | Candidate class | Size / mass [EST] | Risk |
|---|---|---|---|---|
| Speaker + amplifier | ≥ 85 dB(A) at 1 m [EST basis: roadside traffic about 70–80 dB(A) [BK], plus about 10 dB] | Sealed 20–28 mm micro-speaker with vent membrane; 2–3 W class-D amplifier [BK] | 5 cm³ / 6 g | Loudness versus sealing |
| Supervisor's own alarm (D-26) | ≥ 85 dB at 10 cm, driven only by the supervisor | Piezo transducer [BK] | 1 cm³ / 2 g | None significant |
| Display | Readable in sunlight, tiny power | Reflective "memory LCD" (Sharp LS013B7DH05 / LS027B7DH01 class) [BK, lifecycle UNVERIFIED] | 3–6 cm³ / 4–10 g | Slow at −20 °C [BK] |
| Cancel control | Usable with gloves; 15 mm or larger; IP67 | Sealed tactile switch under a guarded cap [BK] | 1 cm³ / 2 g | Stuck-button logic in D-31 |
| Enclosure | IP65; 1.5 m drop; −20 to +50 °C | Glass-filled nylon or PC/ABS, 2–2.5 mm walls, rubber bumper, one gasket; HV section potted or behind barriers | 100–150 g | **The 300 g capacitor must be held so a drop does not tear its leads** (Appendix A asks for a shock rating) |
| Pads and connector | Pre-connected; pad connector made to order (Appendix B) | Commercial pad pairs for fit and impedance tests only | Pads 80–100 g and 50–60 cm³ [EST; a Philips HeartStart adult set, 85 cm² conductor area and 1.2 m lead, is listed at 82 g packaged singly, 106 g per set in a 5-pack] | Pads are regulated as strictly as the device; storage heat (§1) |
| Charging contacts (D-46) | Sealed, dead when undocked | Flat contact pads on the device, spring contacts in the dock | 2 cm³ / 5 g | Corrosion; see §5 |

---

## 9. Roll-up: volume, mass, cost and power

### Assumptions

- **Packing factor 0.5** (decision set) or **0.6** (smallest set) [EST; real handhelds run 0.4–0.6, and 2 kV spacing pushes toward the lower end].
- The shell is added to the inside volume. When the inside grows, the shell's volume and mass grow with its surface area, that is with (inside volume)^⅔ [CALC].
- Pads are included in the verdict (D-43).

### Before the 27 November decisions (version 0.1, kept for comparison)

| Block | Lower-risk: cm³ | Lower-risk: g | Smallest: cm³ | Smallest: g |
|---|---:|---:|---:|---:|
| Capacitor | 180 | 250 | 150 | 210 |
| Charger | 20 | 35 | 15 | 25 |
| Bridge + gate drive + reed relays | 35 | 50 | 25 | 30 |
| Discharge load | 10 | 25 | 10 | 21 |
| Battery + gauge + charger | 38 (2 × 18650) | 97 | 28 (1 × 21700) | 75 |
| Front end + protection | 20 | 25 | 15 | 20 |
| Controllers, Bluetooth, flash, sensing | 10 | 12 | 8 | 8 |
| Speaker, alarm, display, button, LEDs | 12 | 20 | 8 | 15 |
| Boards, wiring, fasteners | (in packing) | 45 | (in packing) | 35 |
| HV potting / barriers | (in packing) | 35 | (in packing) | 25 |
| **Parts subtotal** | **325** | **594** | **259** | **464** |
| Inside volume | 650 | | 432 | |
| Shell and gasket | +110 | 140 | +90 | 100 |
| **Device without pads** | **0.76 L** | **734 g** | **0.52 L** | **564 g** |
| Pads | +60 | +100 | +50 | +80 |
| **Device with pads** | **0.82 L** | **834 g** | **0.57 L** | **644 g** |

### After the 27 November decisions [CALC from the blocks; block values EST as in §§1–8]

**(a) Decision set**: every decision followed, the lower-risk choice wherever a choice remains, and the **expected** capacitor. **(b) Smallest plausible set**: every decision followed, the smallest credible choice everywhere, and the **low** capacitor.

| Block | (a) cm³ | (a) g | (b) cm³ | (b) g | What changed |
|---|---:|---:|---:|---:|---|
| Capacitor (D-41) | 210 | 300 | 170 | 225 | §1 expected / low |
| Charger | 20 | 35 | 15 | 25 | — |
| Bridge + gate drive + reed relays | 35 | 50 | 25 | 30 | — |
| Discharge load | 10 | 25 | 10 | 21 | — |
| Battery, gauge, charger, dock contacts (D-44, D-46) | 39 | 100 | 39 | 100 | (b) now 2 × 18650; both + contacts |
| Front end + protection | 20 | 25 | 15 | 20 | — |
| Controllers, Bluetooth module, flash, sensing (D-39, D-45) | 11 | 13 | 10 | 10 | antenna keep-out; module in (b) |
| Speaker, alarm, display, button, LEDs | 12 | 20 | 8 | 15 | — |
| Boards, wiring, fasteners | (in packing) | 50 | (in packing) | 38 | + capacitor mount for drop |
| HV potting / barriers | (in packing) | 35 | (in packing) | 25 | — |
| **Parts subtotal** | **357** | **653** | **292** | **509** | |
| Inside volume = parts ÷ packing factor | 714 | | 487 | | ÷ 0.5; ÷ 0.6 |
| Shell and gasket (scaled with surface) | +117 | 149 | +97 | 108 | ×1.065; ×1.083 |
| **Device without pads** | **0.83 L** | **802 g** | **0.58 L** | **617 g** | |
| Pads, pre-connected | +60 | +100 | +50 | +80 | unchanged |
| **Device with pads (D-43)** | **0.89 L** | **902 g** | **0.63 L** | **697 g** | |

**Sensitivity of the result to the capacitor and battery [CALC]:**

| Variant | With pads |
|---|---|
| (a) with the **low** capacitor | 0.80 L, 816 g |
| (a) with the **high** capacitor | 1.02 L, 1,028 g |
| (a) with 2 × 21700 cells instead of 2 × 18650 | 0.93 L, 955 g |
| (b) with the **expected** capacitor | 0.71 L, 784 g |

### What each decision cost [CALC]

Each decision applied alone to the version 0.1 set, carried through the packing factor and shell. Grams and cm³ of the finished device with pads.

| Decision | Lower-risk set | Smallest set | Why |
|---|---:|---:|---|
| **D-41** capacitor 2.2 kV, 85 °C storage, 65 °C working | **+67 cm³, +63 g** | **+38 cm³, +23 g** | larger, heavier capacitor (expected / low case) and a stronger mount |
| **D-44** two cells | 0 | **+19 cm³, +25 g** | the lower-risk set already had two cells; the smallest set had one |
| **D-45** separate Bluetooth module | +2 cm³, +1 g | +4 cm³, +3 g | module and antenna keep-out |
| **D-46** dock contacts instead of USB-C | +2 cm³, +3 g | +2 cm³, +3 g | contacts, alignment magnets, input protection |
| **D-39** two Cortex-M33s | about 0 | about 0 | costs a few dollars, not grams |
| **D-15** weekly full, daily low-voltage self-test | 0 | 0 | *saves* about +60 cm³ / +110 g of capacitor that twice-daily tests would have needed (the high case) |
| D-42 made-to-order parts allowed | 0 | 0 | no size effect; one-off design charges (cost table) |
| **Total of the decisions** | **+71 cm³, +68 g** | **+62 cm³, +53 g** | sum matches the tables above within rounding |
| **D-43** pads counted in the target | +60 cm³, +100 g against the target | +50 cm³, +80 g | bookkeeping: the pads were always carried, now they count |

### Against targets and commercial units

| | Volume | Mass | Energy |
|---|---:|---:|---|
| Target (REQUIREMENTS, D-43) | 0.7 L | 700 g with pads | 150 J |
| Stretch target | 0.5 L | 500 g | 150 J |
| **(a) Decision set** | **0.89 L** | **902 g** | 150 J |
| **(b) Smallest plausible** | **0.63 L** | **697 g** | 150 J |
| Version 0.1 lower-risk / smallest | 0.82 / 0.57 L | 834 / 644 g | 150 J |
| St John Pocket AED P3 | 0.68 L | < 700 g incl. pads | 150 J |
| Schiller FRED easyport plus | 0.99 L | 780 g | up to 200 J |
| Avive Connect | 1.19–1.51 L | 953 g | 150 J |
| CellAED | 0.31 L | ~300 g | ~75–85 J |

**Plainly:**
- **700 g with pads: not met by the decision set.** It is **202 g over (29%)** and **0.19 L over (27%)**. It would weigh a little less than the Avive Connect (953 g) and more than the Schiller (780 g).
- **The smallest set meets it by 3 g and 0.07 L.** That has no margin: it needs the low capacitor *and* every other block at its smallest. With the expected capacitor it is 784 g, 84 g over.
- **500 g / 0.5 L: not reachable.** The capacitor, battery and shell alone are about 430 g in the smallest set.
- The honest expectation for a first prototype is **780–900 g**. Reaching 700 g is a goal for a refined second design, through the levers below.

### Levers that could close the gap, ranked by grams saved

Starting from the decision set (902 g, 0.89 L). Savings are for the finished device [CALC/EST as marked].

| Rank | Lever | Saves | Cost in risk or function | Needs a decision changed? |
|---:|---|---:|---|---|
| 1 | **Capacitor at the low end**: a maker shows GA-class density (about 1.66 J/cc) at 2.2 kV *with* 85 °C storage and 65 °C working | **about 86 g, 89 cm³** [CALC] | None in function; it depends on what makers can prove. Ask every maker for density and mass against Appendix A | No |
| 2 | **Thinner, ribbed shell** (about 110 g instead of 149 g) | about 40 g [EST] | 1.5 m drop and IP65 harder to pass | No |
| 3 | **Smaller-block choices everywhere else** (surface-mount SiC bridge, lighter boards, potting, charger, front end, user interface: the smallest-set values) | about 70 g, 60 cm³ [CALC: block differences] | Surface-mount switches' pulse rating unproven; less insulation margin; quieter speaker | No |
| 4 | **Tighter packing** (0.6 instead of 0.5) with insulating barriers or potting rather than air gaps | about 25 g, 125 cm³ [CALC] | Depends on the spacing the standard requires at 2 kV (not yet bought); potting adds some mass back | No |
| 5 | **Lighter pads** (smaller pouch, shorter lead) | about 20 g [EST] | Lead must still reach for front-and-back child placement; pads are a regulated part | No |
| 6 | **Capacitor working rating 55–60 °C instead of 65 °C**, with the device pausing to cool if it is ever too hot to charge | about 40–60 g [CALC: removes part of the 19% density penalty] | A long rescue on a very hot day might need a pause. Storage stays 85 °C | Specification change within D-41; **owner question** |
| 7 | **Return to a 70 °C-storage capacitor** (TDK class) with a "never on the dashboard" rule | about 60 g [CALC: expected case back to about 180 cm³ / 250 g] | Reverses D-41's reason | **Revisits D-41**; owner question |
| 8 | **One cell, accepting slower charging in the cold** | about 25 g, 19 cm³ [CALC] | Charge may outlast the analysis below about 0 °C; the arithmetic in §5 says a nearly flat single cell fails at −20 °C | **Revisits D-44**; owner question |
| 9 | **Smaller capacitor** (for example 80 µF) | about 60 g [EST: 20% less capacitor] | Leaves the Philips-equivalent design point that D-41 kept for the bench-equivalence approval route; the waveform model would need re-running | **Revisits D-41 and D-23**; owner question |
| 10 | **Stop charging during analysis** (smaller discharge load) | about 10–15 g [EST] | Adds about 6 s of hands-off time per cycle | **Revisits D-29**; not recommended |

Levers 1–5 together (about 240 g) would bring the decision set to about 660–700 g. That is essentially the smallest set: **700 g needs nearly all of them to succeed.** Levers 6–10 each need the owner; they are listed as questions in §11, not assumed.

### Cost estimate [EST]

All prices are estimates; see the access note at the top.

| | Qty 1 (prototype) | Qty 100, per unit |
|---|---:|---:|
| Capacitor (made to order, 2.2 kV, 85 °C) | $200–600 (sample, quote) | $100–250 |
| Charger incl. transformer samples | $100–200 | $25–40 |
| Bridge, gate drive, relays | $350–550 | $200–330 |
| Discharge load | $20–40 | $15–25 |
| Battery (2 cells), gauge, charger | $45–70 | $22–35 |
| Dock contacts, device side | $5–15 | $2–5 |
| Front end + protection | $60–100 | $30–50 |
| Controllers, Bluetooth module, flash, sensing chains | $60–100 | $35–60 |
| User interface | $50–80 | $25–40 |
| Boards (HV-rated, 4-layer), small batch | $300–800 per batch | $20–40 |
| Enclosure (3D print or machined; tooling later) | $300–1,500 | $15–40 (after $10–40k tooling) |
| Pads (commercial replacements, for fit and impedance only) | $50–80 | — |
| **Parts for one first prototype** | **about $1,540–4,140** | **about $490–920 per unit** |
| One-off design and qualification charges for made-to-order parts (capacitor, transformer, gate-drive isolation, pad connector) | $3,000–10,000 | — |
| Charging dock (not carried) | $50–150 | $15–30 |

Test equipment and engineering time are **not** included.

### Power budget [EST unless marked]

| State | Draw | Notes |
|---|---:|---|
| Standby (worn, ready, Bluetooth module advertising every 1–2 s) | 0.5–1 mW | Supervisor awake periodically, gauge, regulators, module |
| Weekly full-energy self-test (D-15), averaged | 0.42 mW | [CALC: 253 J ÷ 7 days] |
| Daily low-voltage check (D-15), averaged | about 0.17 mW | [CALC: about 15 J a day: about 2 J into the capacitor at 10% voltage, plus electronics awake and a battery load test, EST] |
| **Standby including self-tests** | **1.1–1.6 mW: 0.027–0.039 Wh a day** | [CALC]. Lithium-ion self-discharge (about 2–3% a month [BK]) is of the same order |
| Rescue, not charging | 0.4–0.6 W | |
| Charging | 44–50 W for 7 s | §2 |
| Rescue average with D-29 | about 2.5–2.8 W | 30 min ≈ 1.3–1.4 Wh |

Off the dock, a full 20 Wh pack would cover standby and self-tests for many months and still hold enough for several long rescues [CALC: 0.04 Wh a day plus self-discharge, against 1.3 Wh per long rescue]. The device should still go on the dock every shift.

---

## 10. Ranked risks and unknowns this study leaves open

| Rank | Risk or unknown | Why it matters | What would resolve it (later, under the lab safety policy) |
|---:|---|---|---|
| 1 | **Capacitor: density, mass and hot life at 2.2 kV / 85 °C** | Sets about a third of the mass; spread between low and high cases is 185 g | Quotes from 3–4 makers against Appendix A, with their test evidence; then the project's own 85 °C storage and 50 °C cycling checks |
| 2 | **Battery and pads in a hot car** (new) | D-41 protects the capacitor, but cells and pad gel are typically limited to about 50–60 °C storage [BK] | Read the chosen cells' and pads' storage ratings; decide a storage rule (owner question) |
| 3 | **The 700 g target** | The decision set is 29% over | The levers in §9; maker quotes for the capacitor |
| 4 | **Front end under the charger and after the shock** | The analysis runs during charging (D-29) | Measure ECG and impedance noise with the charger running, first at ≤ 50 V output |
| 5 | **Switch pulse ratings at milliseconds** | No datasheet states them | Single 10 ms, 65 A pulses into 25 Ω; then 20 shocks in 1,300 s |
| 6 | **Gate-drive insulation at 2.2 kV working** | No catalogue part qualifies | Appendix B; partial-discharge and humidity tests on the delivered parts |
| 7 | **Battery in the cold** | 2 × 18650 just manages −20 °C in the pessimistic case | Charge time and minimum cell voltage at −20 °C from 10% charge, packs B and C |
| 8 | **Spacing required by the standard at 2 kV** | Could push the packing factor below 0.5 | Buy IEC 60601-1 and 60601-2-4 |
| 9 | **Voltage sensors over temperature and life** | D-28 is tight against drift | Both chains against a calibrated reference at three voltages and three temperatures |
| 10 | **Internal heating in a long rescue at 50 °C** | Capacitor at about 54 °C against a 65 °C rating | Thermocouples during 14 dumps at 132 s spacing in a 50 °C chamber |
| 11 | **Discharge-resistor repeat rating and 5,000-pulse life** | The EY part is marketed for infrequent pulses | Maker's statement; then 100 dumps at 132 s spacing |
| 12 | **1.5 m drop with a 300 g capacitor** | Lead or mount failure | Drop test with a mass-equivalent dummy capacitor (no HV) |
| 13 | **Bluetooth module approval conditions** | Reusing the approval depends on following the grant's conditions | Read the FCC grant and integration guide of the chosen module |
| 14 | **Ferrocene scope detail** | CPU option and library certification scope | Ask Ferrous Systems for the 26.11.0 qualification report |

---

## 11. Questions for the owner

### Answered on 27 November 2026

| Version 0.1 question | Answer | Recorded as |
|---|---|---|
| 1. Car heat | Made-to-order capacitor with 85 °C storage | D-41 (Q-23) |
| 2. Off-the-shelf parts only | Catalogue where possible; made to order with a published specification where not | D-42 (Q-24) |
| 3. Voltage margin | 2,200 V parts, keep 100 µF | D-41 (Q-25) |
| 4. Self-test frequency | Weekly full energy, daily low-voltage | D-15 (Q-26) |
| 5. Cold-weather charging | Two cells | D-44 (Q-27) |
| 6. Bluetooth code | Separate pre-certified module | D-45 (Q-28) |
| 7. Supervisor technology | Cortex-M33 class, different maker | D-39 (Q-29) |
| 8. Does 700 g include pads? | Yes | D-43 (Q-30) |
| 10. IP65 and charging | Sealed contacts and a dock | D-46 (Q-31) |

### Still open from version 0.1

These two were not among the questions answered on 27 November; they are not in DECISIONS.md.

- **A. Sensor accuracy over life (D-28).** 1% + 7 V holds at manufacture with good parts, but resistor drift uses about a third of the allowance over years. Accept periodic recalibration at service, or specify costlier low-drift dividers?
- **B. MAX30001G's 0–70 °C range** rules out the most capable compression-sensing chip against the −20 °C target. Confirm that the full temperature range comes first?

### New questions raised by this revision

- **C. The 700 g target.** The set that follows your decisions comes to about 900 g with pads; 700 g needs almost every part at its optimistic value. Which do you prefer?
  - (a) keep 700 g as the goal for a refined design, and accept that the first prototype will be about 800–900 g;
  - (b) revisit one of the decisions below to buy grams now;
  - (c) set a different target.
- **D. Capacitor working temperature.** Specify the capacitor for **65 °C working** (expected about 300 g), or **55–60 °C working** with the device pausing to cool if it is too hot to charge (saves perhaps 40–60 g)? Storage stays 85 °C either way.
- **E. Hot cars beyond the capacitor.** The battery and pads are typically limited to about 50–60 °C storage [BK], so an 85 °C capacitor does not on its own make the device dashboard-proof. Options: (a) a "never leave on the dashboard" rule anyway; (b) look for cells and pads rated higher (unknown cost); (c) an insulated carry case for the car. Which?
- **F. Battery size in the cold.** Two 18650 cells just manage a full-speed charge at −20 °C when nearly flat. If the cold tests confirm that margin is too thin: move to two 21700 cells (about +50 g), or accept a slower charge (a few seconds' wait for the shock) when the battery is both cold and low?
- **G. How hard will one device be used?** The capacitor life requirement (5,000 cycles) assumes up to 25 rescues a year, as for a unit shared across shifts. If devices are personal issue and used a few times a year, the requirement could relax to about 2,500 cycles, which may allow a smaller capacitor. Which is closer to how agencies will use it?
- **H. Publishing the made-to-order specifications.** D-42 says each made-to-order part gets a published specification. The sheets in Appendices A and B are requirements only (what the part must do and what proof to ask for), not designs. Confirm they may be published now in the public repository, or hold them for the patent review (D-13)?
- **I. Bluetooth module firmware.** The ready-made serial firmwares are being withdrawn. Accept that the project writes the module's small firmware itself (in Rust where the tools allow; the radio's lowest layer is always the chip maker's closed code), running only on the module, outside D-24?
- **J. Decisions that could be revisited for weight** (listed so they are visible, not recommended): one cell instead of two (D-44, about 25 g); a 70 °C-storage capacitor with a dashboard rule (D-41, about 60 g); a smaller capacitor than 100 µF (D-41/D-23, about 60 g, leaves the Philips-equivalent design point). Do you want any of these studied further?

---

## What I could not verify

- **Capacitor:** any mass or temperature rating for a 2.2 kV defibrillator film capacitor; a life-versus-temperature curve from any maker; whether GA's catalogue ratings are at 25 °C or at the working limit; dimensions of the CRE 200 µF / 2,200 V part. The voltage exponent (6.6) is fitted to one maker's family; published values run from 5 to 13.
- **Battery:** the P28A's and P45B's internal resistance at −20 °C (the multiplier of 3–5 is background knowledge); the P28A's storage temperature range; whether the datasheet copies are official.
- **Bluetooth:** the FCC ID of the Ezurio BL54L15 and the Raytac AN54L15Q (only the BGM220S grant was seen); the BGM220S temperature range and serial co-processor mode; module lifecycles beyond the makers' pages.
- **2-cell parts:** BQ25887 stock; the BQ28Z610's end-of-life status.
- **Dock contacts:** a specific part number with a published IP rating for the device side.
- **Pads:** mass of an unpackaged pad pair; storage temperature limits of commercial pads.
- **Carried over from version 0.1:** prices and stock (distributor pages blocked automated access); switch pulse ratings at shock length; ADS1292R respiration resolution; ADAS1000 lifecycle; base MAX30001 temperature range; STM32U585, NXP MCX-A and Sharp memory LCD lifecycles; Ferrocene's `cortex-m33` CPU scope; spacing at 2 kV (standard not bought).

---

## Appendix A. Specification sheet: made-to-order energy-storage capacitor

**Status: draft requirements for a part to be purchased (D-41, D-42). Not a design.** It says what the part must do and what proof the maker must supply. How the maker achieves it is the maker's business. Figures marked [EST] are this study's targets and may move after maker feedback; figures marked [CALC] follow from the working in §1.

### A1. Electrical

| Item | Requirement | Basis |
|---|---|---|
| Type | Metallized polypropylene film, self-healing, for biphasic defibrillator pulse discharge | §1; what all compact defibrillators use |
| Capacitance | **100 µF ± 5%** at 25 °C, 1 kHz or 120 Hz. Each part's measured value reported on its label or test record | D-23, D-41. The firmware can use each part's measured value |
| Rated DC voltage (VR) | **≥ 2,200 V** continuous over the working temperature range | D-41; 10% above the 2,008 V worst case [CALC] |
| Working voltage | 1,950 V nominal (adult); 1,150 V (child); worst case 2,008 V | D-23; FINDINGS.md |
| Test voltage | 1.1 × VR (2,420 V) for 10 s, terminal to terminal, on every part | TDK's stated test practice for B32365 |
| Terminal-to-case insulation | ≥ 5 kV DC for 60 s, no breakdown [EST]; partial-discharge inception stated by the maker | [EST, to be checked against IEC 60601-1 once bought] |
| Charge-hold | Hold 2,008 V for at least 10 minutes in one event without damage; **≥ 100 h cumulative** at 2,008 V over life | [CALC: up to 34 h in the heavy profile, ×3]; CRE quotes a "max 10 min" style rating |
| Peak discharge current, normal | **65 A** peak, falling exponentially, phase 1 then phase 2, total up to 15.5 ms per shock | DESIGN_TABLE (25 Ω patient) |
| Peak discharge current, fault | Survive **≥ 10** discharges at 2,008 V into a shorted output limited only by the device's own internal resistance (about 5 Ω): **about 400 A** peak | [CALC: 2,008 V / 5 Ω = 402 A]; GA's DQ series states "peak currents up to 300 A", so this must be confirmed |
| Current and voltage reversal | In normal use the **capacitor's** current does not reverse: the output bridge reverses the *patient* current, not the capacitor's. Voltage reversal at the capacitor ≤ 10% of the pre-discharge voltage in fault cases (ringing into a shorted output) | Biphasic truncated exponential (D-02); [EST] for the fault figure |
| Repetition | Up to **20 full-energy discharges at about 65 s spacing**, then one every 2 minutes indefinitely, at 50 °C ambient | IEC 60601-2-4 style "20 shocks in 1,300 s" check; D-29 cycle of 132 s |
| Internal loss (ESR) | Dissipation factor ≤ 0.010 at 120 Hz (ESR about 0.13 Ω at 100 µF) at 25 °C; ≤ 2× initial at end of life | [CALC: ESR = 0.01 / (2π × 120 × 100 µF) = 0.133 Ω]; TDK B32365 states tan δ < 0.010 |
| Internal inductance (ESL) | ≤ 200 nH [EST] | Irrelevant to millisecond pulses above this level; keeps fault ringing low |
| Leakage / self-discharge | Insulation resistance × capacitance ≥ 10,000 s at 25 °C and ≥ 1,000 s at 65 °C [EST] (for 100 µF: ≥ 100 MΩ and ≥ 10 MΩ) | Keeps leakage below the voltage-sensing divider's current; a charge is held for under a minute normally |
| Voltage recovery (dielectric absorption) | ≤ 0.2% of the pre-discharge voltage (about 4 V) 1 minute after discharge to below 10 V and opening the circuit [EST] | SAFETY.md: recovered charge is a hazard; polypropylene is typically far below this [BK] |
| Internal bleed resistor | None, unless agreed | The device has its own discharge path and backup bleeder |

### A2. Life

Usage profile (from §1), over a **10-year** service life:

| Use | Cycles |
|---|---:|
| Weekly full-energy self-test (52 × 10) | 520 |
| Rescue charges, heavy user (25 rescues a year × 14 charges × 10) | 3,500 |
| Factory, acceptance and service | 100 |
| **Full-energy cycles, total** | **4,120** |
| Daily low-voltage checks (365 × 10), at no more than about 10% of full voltage | 3,650 |

Requirements:
- **≥ 5,000 full-energy cycles** (charge to 2,008 V, hold up to 30 s, discharge as A1) **at 50 °C ambient**. [CALC: 4,120 × 1.2, rounded]
- ≥ 4,000 low-voltage cycles (negligible wear expected; stated for completeness).
- End-of-life criteria: capacitance loss ≤ 5% (the conventional pulse-capacitor end-of-life point), dissipation factor ≤ 2× initial, no open circuit, no short circuit, no case deformation.
- The maker states the cycle life at 25 °C, 50 °C and 65 °C, and the model it uses to scale between them.
- Shelf life ≥ 10 years at 25 °C without re-forming.

### A3. Environmental

| Item | Requirement | Basis |
|---|---|---|
| Working temperature | **−20 to +65 °C** ambient at the part | 50 °C device rating + about 4 K internal rise (§4) + margin |
| Storage temperature, unpowered | **−40 to +85 °C** | D-41 |
| Hot storage over life | ≥ 4,000 h cumulative at 85 °C, unpowered, within end-of-life limits | [CALC/EST: 10 summers × 100 days × 4 h] |
| Temperature cycling | ≥ 1,000 cycles −40 / +85 °C, unpowered, within limits | [EST: about one hot-cold swing a day over three years of car storage] |
| Humidity | Climatic category 40/93/56 or better | TDK B32365's category; the device itself is sealed to IP65 |
| Mechanical shock | Survive the device's 1.5 m drop onto concrete: maker to state a shock rating; target ≥ 500 g, 1 ms half-sine, 3 axes, both directions [EST; final level from IEC 60601-1-12 once bought] | REQUIREMENTS drop target; [UNVERIFIED level] |
| Vibration | Vehicle and carried use: maker to state random-vibration rating; target to IEC 60601-1-12 emergency-services levels once bought [UNVERIFIED] | Patrol car, belt-worn |

### A4. Safety

- **Failure mode:** the maker states the end-of-life and fault failure modes. Required: failure by gradual capacitance loss or **open circuit**, not a short circuit, and **no case rupture, flame or ejection of material** when driven to failure at 1.3 × VR [EST test level].
- **Self-healing:** required; the maker states its clearing behaviour and how capacitance loss is limited.
- **Pressure interrupter or equivalent:** the maker states whether the part has one, or demonstrates by test that internal faults cannot burst the case.
- **Materials:** case and fill flame-retardant (UL 94 V-0 or equivalent); RoHS and REACH declarations.
- **Leads:** insulated flexible leads rated ≥ 2 × VR, with strain relief and a stated pull strength. (TDK's standard lead is 15 kV DC, 150 °C silicone.)
- **Marking and traceability:** part number, capacitance, VR, date code and serial number; lot records kept for the life of the device. A medical-device customer will need this.

### A5. Mechanical envelope target

| Item | Target | Stretch |
|---|---|---|
| Volume | ≤ 210 cm³ | ≤ 175 cm³ |
| Mass | ≤ 300 g | ≤ 230 g |
| Shape | Rectangular or oval prism, about 33 × 60 × 105 mm or smaller; maker may propose | GA 320DQ4130 footprint (33 × 55 × 99 mm) |
| Mounting | Suitable for clamping or potting; maker states allowable clamping force | — |

### A6. Test evidence to request from the maker

1. Datasheet and outline drawing with **measured mass and dimensions**.
2. Life test report: cycles to the A2 end-of-life criteria at 25 °C and 50 °C (and 65 °C if available), with the discharge conditions used.
3. Unpowered storage report: capacitance, dissipation factor and insulation resistance before and after ≥ 1,000 h at 85 °C.
4. Temperature cycling and damp-heat results.
5. Fault-discharge (shorted-output) test results at the A1 fault current.
6. Destructive end-of-life test showing the failure mode (A4).
7. Dielectric-absorption (voltage recovery) measurement.
8. Mechanical shock and vibration results.
9. Quality system certificate (ISO 13485 preferred, ISO 9001 minimum), a commitment to notify product changes, and a statement of how long the part will be made.
10. Ten qualification samples with individual test records.

---

## Appendix B. Requirement lists for the other made-to-order parts

Requirements only, as in Appendix A. No construction details.

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
- **Temperature:** working −20 to +65 °C; storage −40 to +85 °C (to match Appendix A).
- **Mass target:** ≤ 10 g device side [EST].
- **Evidence to request:** dielectric test, current pulse test, contact-resistance stability under vibration, ingress test, cycling test.

### B4. Pads (noted, not specified here)

D-42 also lists the pads as made to order. Pads are a regulated product in their own right. A separate study should set gel area (55–70 cm² each, D-05), lead length for front-and-back child placement, shelf life, and the storage temperature (see question E).
