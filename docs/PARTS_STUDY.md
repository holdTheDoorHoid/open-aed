# Parts study and size, weight and power budget

Version 0.1, 27 November 2026. This is a paper study from datasheets, maker pages and distributor listings, checked on 27 November 2026. **No part has been bought, measured or tested. No hardware exists.** Working notes and every source link are in [research_notes/Open source belt worn AED/parts_study_sources.md](../research_notes/Open%20source%20belt%20worn%20AED/parts_study_sources.md). Decision numbers (D-xx) refer to [DECISIONS.md](../DECISIONS.md).

> **What this document is not.** It names kinds of parts, specific part numbers and their published ratings, and does sizing arithmetic. It contains no schematics, wiring, board layouts or assembly steps, and nothing in it is an instruction to build or power anything above 50 volts (D-10, SAFETY.md). Nothing here is safe or ready for use on a person or animal.

How to read the tags:
- **[CALC]** is arithmetic done for this study. The working is shown.
- **[EST]** is an engineering estimate. Its basis is stated.
- **[BK]** is background knowledge from training that was not re-checked this time. Treat it as something to check, not a citation.
- **[UNVERIFIED]** means I looked and could not confirm it.
- **"Checked 27 Nov 2026"** means the listing or page was seen that day.

Most distributor pages (Digi-Key, Ohmite, TME) refused automated access on the day. Most prices below are therefore **estimates**, not quotes. They must be re-checked by hand before anything is ordered.

---

## Summary for the owner

- **The capacitor decides everything.**
  - The design point (100 µF at 1,950 V, about 190 J stored) needs a film capacitor of about 140–280 cm³ weighing about 200–400 g.
  - The only published mass found for a capacitor of this class is **300 g** (a 125 µF, 2,000 V part).
  - No defibrillator capacitor of any make was found in stock at a mainstream distributor. They are all sold by quote.
- **Temperature.**
  - Film capacitors: the best-documented one, TDK's, is rated **55 °C working and 70 °C storage**. One Chinese maker claims 85 °C storage.
  - No capacitor found survives a dashboard (70–100 °C).
  - A device at the top of its 50 °C operating range, running a long rescue, could push its own capacitor past 55 °C. That arithmetic is in section 4.
- **Size and weight.** **The 700 g / 0.7 L target looks reachable but tight.** The estimates run from about 0.52–0.57 L and 560–640 g (smallest plausible set) to about 0.76–0.82 L and 730–830 g (lower-risk set); the lower figure in each pair excludes the pads, the higher includes them. **The 500 g / 0.5 L stretch target does not look reachable** with a capacitor that can be bought. The volume nearly gets there; the mass does not.
- **Rust compiler.** Ferrous Systems' own documents now list both Cortex-M families (M4/M7 and M33/M55/M85) as **qualified** targets for the safety-qualified Rust compiler, Ferrocene. The newer family was added in release 26.11.0. The small Cortex-M0+ chips are not qualified. That settles the disagreement in the earlier notes, with caveats in section 7.
- **Charging during analysis (D-29).** It costs about **1 Wh of battery and 3.5 kJ of heat per 30-minute no-shock rescue**. The battery shrugs this off. The heat and the extra capacitor wear are real but manageable (sections 1 and 4).

---

## Terms used once and then assumed

| Term | Meaning |
|---|---|
| Capacitor | The part that stores the shock energy, like a fast battery that empties in milliseconds. |
| µF (microfarad) | The size of a capacitor. Stored energy = ½ × capacitance × voltage². |
| J/cm³ (energy density) | Joules stored per cubic centimetre. Higher means a smaller capacitor. |
| Film capacitor | Plastic film (polypropylene) rolled with metal coating. What hospital and public-access AEDs use. Long-lived and stable. |
| Electrolytic capacitor | Aluminium foil in liquid. Denser, but limited to about 450 V per part, so many must be connected end to end ("in series") to reach 2,000 V. Ages and dries out. |
| Photoflash capacitor | An electrolytic built for camera flashes, about 330 V, rated for rapid discharge. |
| Series string / balancing | Parts connected end to end share the voltage. Unless helped by "balancing" resistors, one part can take more than its share and fail. |
| Derating | Running a part below its rating to make it last. For example, 1,950 V on a 2,300 V part is 85% of rating. |
| Flyback charger | The circuit that turns the battery's few volts into about 2,000 V, using a transformer. |
| Transformer | Two coils on a magnetic core. The ratio of turns sets how much the voltage is stepped up. |
| SiC MOSFET / IGBT / thyristor (SCR) | Three kinds of electronic switch. A SiC MOSFET or IGBT can be switched off at will. A thyristor, once on, stays on until the current stops, but takes big surges cheaply. |
| Gate drive | The small control signal that turns a switch on. In a bridge it must be insulated from up to 2,000 V. |
| Isolation, working voltage | How much voltage an insulating barrier can stand for its whole life (not just in a brief test). |
| Pulse / surge rating | How much current a part can take for a brief moment. Only useful if the datasheet says how brief. |
| Front end (AFE) | The chip that measures the tiny heart signal and the pads' resistance. |
| Impedance | Electrical resistance measured with a small alternating current. Here it means the pads' resistance. |
| MCU | Microcontroller: a small computer on one chip. |
| HAL | The Rust code library that lets firmware use a particular MCU's pins and peripherals. |
| Packing factor | The volume of the parts divided by the inside volume of the box. Real products are about 0.4–0.6. |
| dB SPL | Loudness. +10 dB sounds about twice as loud. |
| IP65 | Dust-tight, and protected against water jets. |

---

## The design point this study sizes against

From [DESIGN_TABLE.md](../firmware/crates/aed-waveform/DESIGN_TABLE.md) (model, not measurement):

| Item | Adult | Child | Source |
|---|---:|---:|---|
| Capacitor | 100 µF | 100 µF | D-23 |
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

- Store at least **190 J at 1,950 V**, with the voltage rating above the 2,008 V worst case.
- Give up 65 A in a few milliseconds.
- Survive the duty of D-29 and D-15.
- Live in a patrol car.

### Candidates

Checked 27 Nov 2026. "Stored at 1,950 V" is [CALC] ½CV².

| | A. GA 320DQ4130 (film) | B. CRE DMJ-PC 125 µF (film) | C. TDK B32365 (film, made to order) | D. Bank of 10 × 450 V 105 °C electrolytics | E. Bank of 28 × 330 V photoflash (Chemi-Con EPH-331) |
|---|---|---|---|---|---|
| Rating | 130 µF, 2.0 kV | 125 µF, 2,000 V "max 10 min" | 32–200 µF, to 5.5 kV, values on request | 5 in series × 2 strings, 270 µF / 450 V each [EST value] → 108 µF, 2,250 V total | 7 in series × 4 strings, 194 µF / 330 V → 111 µF, 2,310 V total |
| Stored at 1,950 V | 247 J | 238 J | as ordered | 205 J | 211 J |
| Voltage margin over 1,950 V | **2.6% (below the 2,008 V worst case)** | **2.6% (below 2,008 V)** | as ordered (2.2–2.3 kV gives 13–18%) | 15% (390 V per can, if balanced) | 18% (279 V per can) |
| Size | 33 × 55 × 99 mm = **180 cm³** | not published | "upon request" | 10 cans of about 25 × 40 mm: 196 cm³ of cans, **about 250 cm³ boxed** [CALC, EST size] | **137 cm³ of cans, about 175 cm³ boxed** (earlier notes) |
| Mass | not published; **about 230–270 g** [EST at 1.3–1.5 g/cm³, set by B's measured 300 g] | **300 g (maker's figure)** | not published | **about 310 g** [CALC: 10 × 19.6 cm³ × 1.57 g/cm³, the density of a weighed Rubycon can] | **about 205 g** [CALC: 28 × 4.9 cm³ × 1.5 g/cm³] |
| Working temperature | not published [UNVERIFIED] | −25 to 60 °C | −20 to **55 °C** | to 105 °C (general-purpose rating) | −20 to **65 °C** |
| Storage temperature | not published | **−40 to 85 °C** (maker's claim) | −20 to **70 °C** | to 105 °C, but needs re-forming | 65 °C (1,000 h unbiased test) |
| Cycle life | **2,500** | ≥ 10,000 | up to 10,000 **at 25 °C**, less when hot | not rated for pulse discharge [UNVERIFIED] | 5,000 (camera-flash test) |
| Where to buy | quote only (General Atomics, US) | quote only (China) | quote only (TDK, made in Brazil) | **Digi-Key, in stock** (similar Rubycon 220 µF part: 20 in stock) | not found at Digi-Key, Mouser or LCSC |
| Lifecycle | not stated | not stated | datasheet 2020, version 1 | active | not confirmed |
| Price, qty 1 / 100 | not published; **about $150–400 / $80–200** [EST, from typical custom film pricing] | not published; about $80–250 / $40–120 [EST] | not published; similar to A [EST] | about $10 × 10 = $100 / $60 [CALC from CAD 12.34 / 5.67 listing] + balancing parts | about $3–6 × 28 = $85–170 [EST] |
| Main risk | No voltage margin; temperatures not published | Unknown maker quality; odd "max 10 min" rating | Size unknown; 55 °C working | Not made for pulse discharge; 10 parts to balance; 31% heavier than A | Not stocked; 65 °C ceiling; 28 parts to balance |

### Temperature: which options survive a parked car

- **Cabin air, 47–70 °C.**
  - Survive *storage* at 70 °C: TDK (exactly at its limit, no margin), CRE (by its claim), the 105 °C electrolytics.
  - Do not survive: the photoflash bank (65 °C).
  - GA: unknown.
- **Dashboard, 70–100 °C. Storage at 85 °C:**
  - **No film capacitor with a trustworthy published rating.**
  - CRE claims it. The 105 °C electrolytics would survive storage.
- **What heat does to life (film).** TDK states its 10,000 cycles at 25 °C only, and refers to a lifetime curve that is not in the datasheet. Polypropylene film loses voltage strength as it warms; makers usually derate above about 70–85 °C [BK]. The Jewel wearable defibrillator qualified its capacitor at only 800 cycles at 50 °C (earlier notes).
- **What heat does to life (electrolytics).** Their life roughly doubles for every 10 °C cooler [BK]. A 3,000 h at 105 °C part would give roughly 12,000 h at 85 °C and about 34,000 h at 70 °C [CALC from that rule]. Storing them hot without voltage speeds up the loss of their insulating oxide layer, so they must be re-formed: brought up to voltage gently.
- **The plain answer.** Only electrolytics have a credible rating above 70 °C, and they are the heaviest, bulkiest and least suited to pulse duty. Every film option needs either a "do not leave in a hot car" rule or a custom part specified for 85 °C storage.

### Operating heat inside the device

This links to the discharge load in section 4.

- At a 50 °C ambient, a long no-shock rescue warms the inside of the box by about 4 K on average [CALC, section 4], and more near the discharge resistors.
- **That puts a 55 °C-rated film capacitor at or over its working limit.**
- Either the capacitor must be specified for 60–70 °C working, or the operating target must drop to about 45 °C (see owner questions).

### Cycle life against the D-29 and D-15 duty [CALC]

Usage profile, stated as an assumption [EST]:
- One full-energy self-test per shift (D-15), two shifts a day: 730 a year.
- Six rescues a year averaging 20 minutes. With D-29, every analysis charges the capacitor: one charge per 2.2-minute cycle, so about 9 charges per rescue and 54 a year.

| Self-test policy | Charges per year | Over 5 years | Against GA 2,500 | Against film rated 10,000 at 25 °C |
|---|---:|---:|---|---|
| Full energy twice a day | 784 | 3,920 | **exceeds** | 39% used (more if hot) |
| Full energy once a day | 419 | 2,095 | 84% used | 21% |
| Full energy weekly, low-voltage check daily | 106 | 530 | 21% | 5% |

**The self-test, not rescues, wears the capacitor out.** D-29 adds only about 9 charges per rescue.

### Self-discharge, recovery and failure

- **Film**
  - Holds charge well, and needs no re-forming.
  - Recovers a little voltage after being emptied: "dielectric absorption". TDK warns of "self-recharging phenomena".
  - Usually fails gently. It "self-heals" small faults and loses capacitance slowly [BK].
- **Electrolytics**
  - Leak more, and recover more charge after emptying (SAFETY.md quotes about 2 J for the flash type).
  - Can fail short or vent.
  - In a series string they need balancing resistors. Because D-29 holds the charge for only 7–12 seconds at a time, the balancing drain is negligible.

### Recommendation

- **Buy a single film capacitor made to the project's own specification** from a defibrillator-capacitor maker: GA, CDE/Knowles, TDK, or CRE as a low-cost check.
- Ask for:
  - about 100–130 µF
  - **at least 2.2 kV rated**
  - storage to 85 °C and working to 60–70 °C
  - at least 5,000 cycles at 50 °C
  - mass and dimensions on the drawing
- **Budget 180–200 cm³ and about 250 g** [EST: GA's 1.66 J/cm³ technology at a 2.2 kV rating gives about 146 cm³ for 100 µF; allow for the bigger rating].
- Keep the 105 °C electrolytic bank as the fallback for heat, not for size.

### What must be tested to confirm it

Later, under the lab safety policy:
- Capacitance, leakage and internal loss before and after 1,000 h of storage at 85 °C, without voltage.
- 1,000 full-energy cycles at 50 °C ambient. Measure capacitance loss and any rise in internal loss.
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
| Battery current, 1 cell at end of discharge, cold (about 2.8–3.0 V under load [EST]) | 44 W / 2.9 V | **about 15 A average, roughly 30 A peaks** [EST ×2 for flyback peaks] |
| Battery current, 2 cells in series (about 5.8 V cold) | 44 W / 5.8 V | **about 7.6 A average** |

### Candidate controller chips

| Part | What it is | Fit | Status, checked 27 Nov 2026 |
|---|---|---|---|
| ADI LT3751 | Flyback capacitor-charger controller; regulates the output voltage itself | Built for this job (photoflash, detonators); maker designs stop at about 450 V, so 2 kV depends on the transformer | Lifecycle [UNVERIFIED]; about $8–12 [EST] |
| ADI LT3750 | Simpler capacitor-charger controller; voltage set by turns ratio | Good for a fixed adult voltage; child mode needs separate regulation | [UNVERIFIED] |
| General-purpose flyback controller (e.g. TI LM5155/UCC28C4x class) | Flexible; needs the voltage loop built around it | More work, more control | [BK] |

### The transformer problem (sourcing only)

- No catalogue transformer reaches about 2 kV for this job; Würth's stock part for the LT3751 is 300 V.
- **It must be custom.** Firms that advertise custom HV or flyback transformers with prototypes:
  - Custom Coils (up to 60 kV, biomedical)
  - Agile Magnetics (prototype to production)
  - Pulse Electronics (quick-turn custom power transformers, free small-volume samples)
  - Bourns (custom worksheet)
- One unnamed maker's catalogue lists a "Model 4283-1200" flyback for portable defibrillators [UNVERIFIED maker].
- Typical cost [EST, from custom-magnetics norms, not quotes]: design and tooling $500–2,500; prototype samples $50–150 each; about $10–25 at qty 100.
- Typical size [EST]: a 35–50 W flyback at 100–200 kHz uses a core around 20–25 mm, so the part is about 10–20 cm³ and 15–35 g, more with thick insulation for 2 kV.
- An HV output diode rated about 3 kV or more, fast recovery, is also needed [BK].

### Alternative

Charge several lower-voltage capacitors side by side and reconnect them end to end for the shock. This avoids the 2 kV transformer but needs many more HV switches, and Avive holds patents on it (earlier notes). Not recommended before the patent review.

### Heat

About 63 J per charge [CALC], mostly in the transformer and switch. Over a 30-minute rescue that is 0.9 kJ, folded into section 4.

### Recommendation

- Controller: LT3751-class.
- Custom transformer: specify it for 35 W out, 2.2 kV output insulation, −20 to +60 °C.
- Battery: a **2-cell** pack, to halve battery current (section 5).

### Test

- Charge time to 1,950 V into a 100 µF load at −20, +25 and +50 °C, from a battery at 10% charge.
- Efficiency, and transformer temperature after 14 charges at 2.2-minute spacing.

---

## 3. Output bridge and patient isolation

### What it must do

- Block 1,950 V (2,008 V worst case) in each switch position.
- Carry 65 A peak, falling over 3–15 ms.
- Switch off on command to end each phase.

### Candidates

| | A. Four 3.3 kV SiC MOSFETs: Microchip MSC080SMA330B4N | B. Four 3.3 kV SiC in surface-mount: GeneSiC G2R120MT33J | C. Thyristors plus IGBTs (the expired Physio-Control approach): 3 × Littelfuse CS20-22moF1 + 1–2 × IXEL40N400 |
|---|---|---|---|
| Blocking voltage | 3,300 V: **69% margin** | 3,300 V: 69% | Thyristor 2,200 V: **13% margin, 10% at 2,008 V (thin)**. IGBT 4,000 V: 105% |
| Pulse current (datasheet) | IDM 107 A, "repetitive, pulse width … limited by maximum junction temperature". **No pulse width given.** 1.65× the 65 A need | body-diode pulse 116 A; drain pulse rating **not retrieved** [UNVERIFIED] | Thyristor surge ITSM 200 A (condition not read; normally a 10 ms half-sine, which matches a shock's length [BK]). IGBT ICM 250 A at 1 ms (2009 sheet) or 170 A (2006 sheet) — **the sheets disagree** |
| Heating per shock [CALC] | At 65 A, hot on-resistance about 0.15 Ω [EST, ×1.8 at 150 °C]: 634 W peak. Over a few ms, with single-pulse thermal impedance about 0.05–0.1 °C/W [EST, curve shape; values not extractable], the junction rises about 30–60 °C. **Plausible but unproven** | similar, slightly higher resistance (120 mΩ) | Thyristors are built for this |
| Package, size | TO-247-4, about 16 × 21 × 5 mm, about 6 g each [BK] | TO-263-7 surface-mount, smaller, about 2 g [BK] | ISOPLUS i4-PAC / i5-Pak, about TO-247 size |
| Count | 4 switches, 4 isolated gate drives | 4 / 4 | 3 thyristors + 1–2 IGBTs; only the IGBT leg needs switch-off drive |
| Status, 27 Nov 2026 | Datasheet rev A, April 2025; previously stocked at Digi-Key and RS | lifecycle [UNVERIFIED] | Thyristor at Farnell, Future, LCSC; IGBT at Digi-Key |
| Price, qty 1 / 100 [EST] | about $60–90 / $45–70 each | about $40–70 / $30–50 | thyristor about $15–25, IGBT about $40–60 |
| Main risk | Pulse rating at milliseconds not stated | Pulse rating not retrieved | Thyristor voltage margin too thin; can only be switched off by the IGBT |

**Flag.** None of these datasheets states a pulse current rating at the 3–15 ms pulse length a shock uses. That must be measured, or asked of the makers.

### Gate drive at 2,000 V working

**No catalogue part found qualifies.**
- TI UCC21750 isolated driver: rated 2,121 V DC working, so it would run at **92–95% of rating** at 1,950–2,008 V. Unacceptable.
- Murata gate supplies:
  - MGJ2: 800 V rms reinforced
  - MGJ6: 3 kV DC continuous, functional insulation only (not counted as a safety barrier)

Options:
1. Arrange the bridge so that no driver's barrier sees the full voltage, then use catalogue drivers.
2. Custom pulse transformers that carry signal and power together, as in the Physio-Control patent.
3. Fibre-optic signal links, where the insulation is a plastic fibre, plus a separate gate-power solution [BK; lifecycle of e.g. Broadcom "Versatile Link" parts UNVERIFIED].

Option 2 is the one with expired-patent prior art.

### Isolation relays

- Sensata/Cynergy3 D-HR vacuum reed relays: 10–15 kV stand-off, only 2–4 A carry. They suit disconnecting the heart-signal front end during the shock.
- **No relay was found with a published pulse rating for carrying the 65 A shock** (earlier notes).
- Recommended: rely on two switches in series in each current path as the patient isolation, rather than a big relay.

### Size [EST]

Four switches, drivers, gate supplies and a protective inductor-resistor: **25–40 cm³ and 40–60 g** before the extra spacing the standard requires. That spacing (creepage) needs the purchased standard; see `hv_safety_isolation.md`.

### Recommendation

- Option A: SiC, for its voltage margin and its ability to switch off anywhere.
- Gate drive by option 2 or 3.
- Ask Microchip for a pulse rating at 10 ms.

### Test

- Single 10 ms pulses of 65 A into a 25 Ω test load, first at low voltage. Measure on-voltage to estimate junction temperature.
- Then 20 pulses in 1,300 s at full voltage (the style of the IEC 60601-2-4 shock-capacity test).

---

## 4. Internal discharge load: what charging during analysis costs

### Duty

- Every no-shock analysis burns the full **190 J** (D-29).
- Every shock leaves **up to 36 J** to dump.
- Every self-test burns 190 J.

### Worst-case rescue, stated

- 30 minutes of a non-shockable rhythm.
- Analysis every 2 minutes of CPR plus up to 12 s of analysis: a 132 s cycle.
- That gives **14 full charges burned** [CALC: 1,800 s / 132 s ≈ 13.6, round up].

| Quantity | Working [CALC] | Result |
|---|---|---|
| Energy into the load | 14 × 190.1 J | **2.66 kJ** |
| Charger loss inside the box | 14 × 63 J | 0.89 kJ |
| Electronics and audio | 0.5 W × 1,800 s [EST] | 0.9 kJ |
| Total heat released inside | | **about 4.4 kJ (average 2.5 W)** |
| Battery energy for the charges | 14 × 253 J = 3.55 kJ | **0.99 Wh** (6% of one 21700 cell) |
| Same rescue without D-29 | FINDINGS.md: about 8 J dumped | — |

### Box temperature [CALC]

- Box about 150 × 86 × 50 mm: surface 494 cm².
- Still-air convection plus radiation about 10 W/m²·K [BK]: 0.49 W/K to the outside.
- Steady rise: 2.5 W / 0.49 = 5.1 K.
- Heat capacity about 0.7 kg × 900 J/kg·K = 630 J/K, so it warms with a time constant of 21 minutes.
- **After 30 minutes the inside is about 3.8 K above ambient.** At a 50 °C ambient that is about 54 °C, against a 55 °C film-capacitor limit.
- Worn against a body or lying on sunlit tarmac would be worse.

### Resistor temperature [CALC]

Recommended option below: three 400 J ceramic resistors in series, about 7 g each [EST: 3.2 cm³ × 2.2 g/cm³].
- Each takes 63 J per dump. Adiabatic rise = 63 / (7 g × 0.8 J/g·K) = **11 K per dump**.
- Average 0.48 W each. With about 40 K/W to the inside air [EST], that is a 19 K average rise.
- **Peak about 30 K above the inside air: about 84 °C at a 50 °C ambient.**
- Fine for ceramic. It must be kept away from the capacitor and the battery.

### Candidates

| | A. Ohmite EY ceramic composition, 3 in series | B. Ohmite 885SP bulk ceramic tube, 3 in series | C. Non-inductive wirewound (e.g. Riedon pulse wirewound) |
|---|---|---|---|
| Single-pulse energy | 400 J each (1,200 J total): **6× the 190 J** | 250 J each (750 J): 4× | > 1,000 J (Riedon) |
| Voltage | not found [UNVERIFIED]; 650 V each in a string of three | 1,000 V peak each: 3,000 V string | depends on part |
| Repetition | **Made "for single or infrequent pulse events"**: one pulse every 2 minutes must be confirmed with Ohmite [UNVERIFIED] | 45 W average at 40 °C: ample | depends on part |
| Size, mass | 46.2 × 9.4 mm, about 3.2 cm³ and 7 g each: **about 10 cm³ / 21 g total** [EST mass] | 63.5 × 19 mm, about 18 cm³ and 25 g each: **54 cm³ / 75 g** [EST mass] | about 10–30 cm³ [EST] |
| Status, price [EST] | new in about 2020; about $5–10 each | Mouser-hosted datasheet; about $10–20 each | various |
| Main risk | Repeat-pulse rating | Bulk | Coil inductance makes the dump ring |

### Recommendation

- Option A, if Ohmite confirms the repetition rate. Option B as the conservative fallback, costing about 45 cm³ and 55 g more.
- Either way, the D-29 cost is about **25 g, 10 cm³ and a few kelvin of internal heating** in the worst rescue, plus about 14 capacitor cycles per long rescue.

### Test

- Resistor surface temperature after 14 dumps of 190 J at 132 s spacing, inside a sealed box at 50 °C ambient.
- Capacitor case temperature in the same run.

---

## 5. Battery

### Mission energy [CALC/EST]

A 12-hour shift plus one 30-minute rescue with 14 charges.

| Item | Basis | Energy |
|---|---|---:|
| Standby, 12 h at about 0.5–1 mW (MCUs asleep, supervisor checking, Bluetooth advertising slowly) | [EST], section 9 | 0.01 Wh |
| One full-energy self-test | 253 J | 0.07 Wh |
| Rescue electronics, 0.5 W × 0.5 h | [EST] | 0.25 Wh |
| 14 charges | 3.55 kJ | 0.99 Wh |
| **Total** | | **about 1.3 Wh** |

- **Energy is not the limit; current in the cold is.**
- One Molicel P45B holds 16.2 Wh. At −20 °C the datasheet copy shows at least 3.6 of its 4.5 Ah at a gentle 4.5 A [UNVERIFIED copy]. Its behaviour at the 15 A a single-cell charger needs in the cold was not found.

### Charges per battery charge [CALC]

- New, room temperature: 16.2 Wh × 0.9 usable / 253 J per charge ≈ **207 charges**.
- Cold and aged (about 60% usable): ≈ **140**.
- REQUIREMENTS asks for 30 shocks or more. Met with large margin, even counting burned charges.

### Candidates

| | A. 1 × Molicel P45B 21700 | B. 2 × high-drain 18650 in series (Molicel P28A class) | C. 2-cell high-rate pouch pack |
|---|---|---|---|
| Energy | 16.2 Wh | about 20 Wh [UNVERIFIED cell spec] | about 11–16 Wh [EST] |
| Current per cell at a 44 W charge, cold | about 15 A average, ~30 A peaks | about 7.6 A | about 7.6 A |
| Continuous rating | 45 A (35 A per one retailer) | about 35 A [UNVERIFIED] | high, varies |
| Mass / volume | about 70 g / 25 cm³ | about 90 g / 33 cm³ [EST 45 g each] | about 60–90 g [EST] |
| Price [EST] | about $6–8 / $4 | about $10 / $7 for two | about $15–30 |
| Status | stocked at NKON (earlier notes) | [UNVERIFIED] | hobby-grade quality varies |
| Main risk | Voltage sag at −20 °C under 15–30 A | 20 g heavier | Quality and traceability |

### Supporting parts [BK; lifecycle UNVERIFIED]

- **Fuel gauge with protection:** for 2 cells, ADI MAX17320 class or TI BQ40Z50 class; for 1 cell, TI BQ27Z561 class.
- **Charger:** a TI BQ2579x-class buck-boost charger that accepts USB-C power. It must block charging below 0 °C and above about 45 °C. A device left in a cold car cannot recharge until warmed.
- **Charging connection:** an open USB-C port works against IP65. Prefer **sealed contacts on a charging dock** (radio-style), with the dock taking USB-C.
- **Shipping:** the cells and the finished pack each need UN 38.3 testing. A device shipped with its battery inside goes as UN 3481 [BK].
- **Cycle life:** high-drain Li-ion cells are typically rated about 500 cycles to 80% capacity [BK]. Charged daily, plan on replacing the battery every 2–3 years.

### Recommendation

Option B (2 cells) for the recommended set; option A for the smallest set.

### Test

Charge time to 1,950 V from each pack at 10% charge, at −20 °C after a 4-hour cold soak. Log the cell voltage minimum.

---

## 6. Heart-signal and pad-resistance front end

### What it must do

- Read the heart signal through the shock pads.
- Measure pad resistance from 25 to 250 Ω (D-32). The **compression signal is a change of a few tenths of an ohm at 1–3 Hz** (D-36).
- Survive the device's own 2,000 V shock and recover quickly.
- Work while the charger runs (D-29).

### Resolution needed [CALC]

- 0.3 Ω on 250 Ω is **0.12%** of the baseline.
- With a 48 µA measuring current, 250 Ω gives 12 mV and 0.3 Ω gives 14 µV.
- Against the MAX30001's quoted 1.1 µV p-p noise, that is a signal-to-noise ratio of about 13. **Adequate**, if the protection parts do not swamp it.

### Candidates

Status checked 27 Nov 2026.

| | A. TI ADS1292R | B. ADI MAX30001 / MAX30001G | C. ADI MAX30009 (impedance) + a separate ECG chip | D. ADI ADAS1000 |
|---|---|---|---|---|
| What it does | 2-channel 24-bit ECG with built-in respiration impedance | ECG + bio-impedance channel (17-bit effective, drive 55 nA–96 µA) | Impedance only, drive 16 Hz–500 kHz, 17-bit; very flexible | 5-electrode ECG + respiration, drive 46–64 kHz |
| Status | **ACTIVE** (TI page) | **PRODUCTION** (MAX30001G) | **PRODUCTION** | [UNVERIFIED] |
| Temperature | **−40 to +85 °C** | **MAX30001G: 0 to +70 °C — fails the −20 °C target.** Base MAX30001 [UNVERIFIED] | [UNVERIFIED] | [UNVERIFIED] |
| Noise | 8 µVpp (150 Hz bandwidth) | 1.1 µVpp (BioZ) | "1.1 Ω p-p" per Mouser (units doubtful) | ADI says about 0.2 Ω breathing changes are resolvable if noise allows |
| Size, price [EST] | 4 × 4 mm; about $10 / $7 | wafer-level package; about $10 / $7 | about $8 / $5, plus about $5 for ECG | about $25 / $18 |
| Main risk | Respiration resolution at 0.3 Ω not published [UNVERIFIED] | Temperature range | Two chips; more firmware | Lifecycle unknown; overkill |

### Problems common to all

1. **Shock survival.**
   - All need external protection: series resistors in the tens of kilohms, chip resistors built for defibrillator pulses (e.g. TT Electronics DPCR, 1–2 J per 10 ms, pre-qualified to IEC 60601-2-27), clamps.
   - Plus the HV reed relays of section 3 to disconnect them during the shock.
   - These protection parts degrade with repeated shocks (earlier notes).
2. **The protection resistors are larger than the patient.**
   - 10–50 kΩ in series dwarfs 25–250 Ω. A naive measurement would read mostly the protection.
   - Measuring pad resistance precisely needs the measuring current and the voltage reading to take separate paths, so the protection resistors drop out of the result. This is a known technique; the detail belongs in the withheld design files.
3. **The charger runs during analysis (D-29).**
   - The charger switches at tens to hundreds of kilohertz.
   - The impedance measurement also runs at tens of kilohertz, so they can interfere. Choose the impedance frequency away from the charger and its multiples, or synchronise them.
   - The charger's fast voltage swings can also couple into the pads through the transformer's insulation.
   - **No datasheet can answer this; it must be measured.**
4. **Recovery after a shock.** The time limit is set in IEC 60601-2-4 (not yet bought). No front-end datasheet quotes a recovery time with real protection fitted.

### Recommendation

- **Option A (ADS1292R)**: active, full temperature range, ECG and impedance in one chip.
- **Option C (MAX30009 for impedance)** as the upgrade if A's impedance resolution proves too coarse for compressions.

### Test

All at low voltage first.
- Impedance noise on a 25–250 Ω test resistor, with a 0.3 Ω change at 2 Hz superimposed. Measure the smallest change seen.
- The same with the charger running into a dummy load.
- Recovery time of the ECG baseline after a shock into a patient simulator (later, under the policy).

---

## 7. Controllers

### Processing need

From `aed-rhythm` and `aed-eval/REPORT.md`:
- About 120,000 multiply-adds per second of signal, and about 10.5 KB of RAM for analysis.
- [CALC] On a 100 MHz Cortex-M with a floating-point unit, that is **under 1% of the processor**.
- Any candidate below is ample.
- The event record is the larger memory need: 2 channels × 250 samples/s × 2 bytes = 1 KB/s, so 30 minutes is 1.8 MB. That calls for an external flash chip of 16 MB or more [CALC; part e.g. Winbond W25Q128 class, BK].

### Ferrocene: the qualification question, answered from Ferrous Systems' own documents

Fetched 27 Nov 2026.

- **Ferrocene user manual, target list:** <https://public-docs.ferrocene.dev/main/user-manual/targets/index.html>
  - **Qualified:** `thumbv7em-none-eabi(hf)` (Cortex-M4/M7), `thumbv8m.main-none-eabi(hf)` (Cortex-M33/M55/M85), `armv7r-none-eabihf` (Cortex-R), `aarch64-unknown-none`.
  - **Only "supported" (not qualified):** `thumbv6m-none-eabi` (Cortex-M0/M0+) and `thumbv8m.base-none-eabi` (Cortex-M23).
  - The bare-metal targets are qualified only when built on x86-64 Linux.
- **Release notes 26.11.0** (newest stable release listed): <https://public-docs.ferrocene.dev/main/release-notes/26.11.0.html>. Quote: "Two new targets are now supported and qualified for safety critical use", naming the two Armv8-M Mainline targets.
- **Ferrocene 26.05.0 blog** (28 July 2026): <https://ferrous-systems.com/blog/ferrocene-26-05-0/>
  - States the toolchain is TÜV SÜD-qualified "according to ISO 26262 (ASIL D), IEC 61508 (SIL 3), and IEC 62304 (Class C)".
  - Notes that qualified support for choosing a specific CPU within a target is limited. For `thumbv7em`, only `cortex-m4` is named; Cortex-M7 is not mentioned.
- **Caveats**
  - Both doc pages are labelled a preview from the development branch. The signed qualification report for the exact release is the authority, and I did not see it.
  - The certified subset of Rust's core library is certified to IEC 61508 SIL 2 / ISO 26262 ASIL B. The page does not list IEC 62304 for the library.
  - Whether `-C target-cpu=cortex-m33` itself is inside the qualified envelope is [UNVERIFIED].
- **What this means.**
  - The earlier conflict is resolved in favour of Cortex-M. **Both controllers should be Cortex-M33-class (or M4) parts.**
  - **A Cortex-M0+ supervisor would sit outside the qualification.**

### Main controller candidates

| | A. ST STM32U585 (Cortex-M33, 160 MHz, FPU) + separate Bluetooth module | B. Nordic nRF54L15 (Cortex-M33 with FPU, Bluetooth built in) | C. Nordic nRF52840 (Cortex-M4F, Bluetooth built in) |
|---|---|---|---|
| Ferrocene target | thumbv8m.main: **qualified** (26.11.0) | thumbv8m.main: **qualified** | thumbv7em: **qualified** (cortex-m4 named) |
| Rust HAL | embassy-stm32 ("all STM32 families") | embassy-nrf covers nRF54; `trouble` BLE host supports nRF54 | embassy-nrf and nrf-softdevice: the most mature |
| Bluetooth | In the module (e.g. a pre-certified nRF54L15 module running Nordic's own firmware) | On chip, via Nordic's **closed binary** radio controller | On chip, closed binary (SoftDevice) |
| Effect on D-24 ("only `libm`") | **Clean**: radio code lives on a separate chip | Breaks it: binary radio code on the safety processor | Breaks it |
| Status | [UNVERIFIED this pass] | in production (Nordic page) | mature [BK] |
| Price [EST] | MCU about $8–12 / $6; module about $10–15 / $7 | about $3–5 | about $5–7 |

### Supervisor candidates

It should be deliberately different and simpler.

| | A. NXP MCX A-series (Cortex-M33, different maker from the main chip) | B. A second small ST Cortex-M33 (e.g. STM32U3 class) | C. Programmable logic chip, CPLD (e.g. Lattice MachXO2) | D. A Cortex-M0+ (e.g. STM32C0, TI MSPM0) |
|---|---|---|---|---|
| Ferrocene | qualified (Armv8-M Mainline) | qualified | **not applicable: written in a hardware language (Verilog/VHDL), a new language under D-08** | **not qualified** (only "supported") |
| Rust HAL | listed in the Embassy README (NXP MCX-A) | embassy-stm32 | none | embassy-stm32 / MSPM0 |
| Independence from main | high: different maker, silicon and tools | lower: same maker | highest (Jewel uses one) | high |
| Lifecycle | [UNVERIFIED] | [UNVERIFIED] | [BK] active | [BK] |
| Price [EST] | about $2–4 | about $3–5 | about $5–10 | about $1 |

**Recommendation:** main **A** (STM32U585-class) plus a separate pre-certified Bluetooth module; supervisor **A** (NXP MCX A-series). Both run Rust on qualified targets, from two different makers.

### Voltage-sensing chains: can 1% + 7 V hold?

- At 1,950 V the allowance is 26.5 V = **1.36%**. At 1,150 V it is 18.5 V = 1.61% [CALC].
- Each chain is a high-voltage resistor divider plus an analog-to-digital converter (ADC) with its own voltage reference.
- **Resistor class matters most.** Ordinary HV chip resistors can change value with applied voltage by "300 ppm and beyond" per volt (Stackpole). Across the 200–500 V each one carries, that is several percent. Only screened low-voltage-coefficient parts will do:
  - Stackpole HVCR: better than 2 ppm/V in 2512
  - Vishay HV thin film: below 2 ppm/V
  - TT Electronics HV dividers: 0.2–1 ppm/V

**Worst-case error budget per chain** [CALC from the cited classes; ADC and reference figures BK]:

| Error source | Assumption | Error |
|---|---|---:|
| Divider ratio after one-point factory calibration | residual | 0.05% |
| Temperature tracking between top and bottom resistors | 50 ppm/K mismatch × 45 K (25 → −20 or +70 °C) | 0.23% |
| Voltage coefficient | 2 ppm/V × about 250 V per resistor (8 in the string) | 0.05% |
| Life drift | 0.3% (TT Electronics best case; 0.5% typical) | 0.30% |
| Surface leakage across the board in humidity | 40 MΩ divider against 100 GΩ leakage | 0.04% |
| ADC gain and linearity | 0.1% [BK] | 0.10% |
| Voltage reference: initial plus drift | 0.05% + 12 ppm/K × 45 K [BK, REF35 class] | 0.10% |
| **Linear sum** | | **0.87%** (plus about 2 V offset) |
| **Root-sum-square** | | **0.41%** |

- **Verdict:** 1% + 7 V is **achievable with the right resistor class, but only just in the worst case over life**. Drift is the largest term.
- Board contamination is the hidden one: at 10 GΩ leakage it alone is 0.4%. That argues for conformal coating and a lower-value divider (more standby current, but only while charged).
- **The two channels should use different resistor and reference families**, so they cannot drift together and still agree. That is a common-cause failure the supervisor's cross-check would not catch.
- **Test:** both chains against a calibrated HV meter at 200, 1,150 and 1,950 V; at −20, +25 and +70 °C; and after 1,000 h at 85 °C and 85% humidity unpowered.

---

## 8. User interface and the rest

| Item | Target and basis | Candidate class | Size / mass [EST] | Risk |
|---|---|---|---|---|
| Speaker + amplifier | **≥ 85 dB(A) at 1 m** [EST basis: roadside traffic about 70–80 dB(A) [BK]; prompts need about 10 dB above background to be understood] | Sealed micro-speaker of 20–28 mm with a water-resistant membrane, plus a 2–3 W class-D amplifier (e.g. MAX98357A / TI TAS25xx class) [BK] | 5 cm³ / 6 g | Loudness versus sealing: the speaker needs an acoustic vent membrane. Verify on a sample |
| Supervisor's own alarm (D-26) | ≥ 85 dB at 10 cm, driven only by the supervisor | Piezo transducer [BK] | 1 cm³ / 2 g | None significant |
| Display | Readable in sunlight, tiny power | Reflective "memory LCD", e.g. Sharp LS013B7DH05 (1.26") / LS027B7DH01 (2.7") class [BK, lifecycle UNVERIFIED]; or the 3-colour readiness LED alone | 3–6 cm³ / 4–10 g | Slow to respond at −20 °C [BK] |
| Cancel control | Usable with gloves; 15 mm or larger; IP67 | Sealed tactile switch (e.g. C&K KSC class, IP67) under a raised, guarded cap [BK] | 1 cm³ / 2 g | Stuck-button logic already in D-31 |
| Enclosure | IP65; 1.5 m drop onto concrete; −20 to +50 °C | Glass-filled nylon or PC/ABS shell, 2–2.5 mm walls, rubber bumper, one compressed gasket; HV section potted or behind barriers | 100–150 g | The 250 g capacitor must be held so a drop does not tear its leads. Potting adds mass |
| Pads and connector | Pre-connected (REQUIREMENTS); pad connector insulated for ≥ 5 kV | A connector of this rating is a custom or semi-custom part [UNVERIFIED]; commercial pad pairs | Pads: about 80–120 g and 60 cm³ folded [EST, no published pad mass found] | Pads are regulated as strictly as the device |
| Charging contacts | Sealed | Spring-pin contacts to a dock [BK] | 1 cm³ / 3 g | Contact corrosion |

---

## 9. Roll-up: volume, mass, cost and power

### Assumptions

- **Packing factor 0.5** (recommended set) or **0.6** (smallest set). Real handhelds run 0.4–0.6 [EST]; HV spacing pushes toward the lower end.
- Shell material is added on top of the inside volume.
- Pads are counted separately, then included, because the St John P3's "<0.7 kg" includes its pads.

| Block | Recommended: volume cm³ | Recommended: mass g | Smallest plausible: volume cm³ | Smallest plausible: mass g | Basis |
|---|---:|---:|---:|---:|---|
| Capacitor | 180 | 250 | 150 | 210 | §1; smallest = custom 100 µF / 2.2 kV at GA density |
| Charger (IC, custom transformer, switch, HV diode) | 20 | 35 | 15 | 25 | §2 [EST] |
| Bridge + gate drive + reed relays | 35 | 50 | 25 | 30 | §3 [EST]; smallest = surface-mount SiC |
| Discharge load | 10 | 25 | 10 | 21 | §4 |
| Battery + gauge + charger | 38 | 97 | 28 | 75 | §5: 2 × 18650 / 1 × 21700 |
| Front end + protection | 20 | 25 | 15 | 20 | §6 [EST] |
| Controllers, Bluetooth module, flash, sensing | 10 | 12 | 8 | 8 | §7 [EST] |
| Speaker, alarm, display, button, LEDs | 12 | 20 | 8 | 15 | §8 |
| Circuit boards, wiring, fasteners | (in packing) | 45 | (in packing) | 35 | [EST] |
| HV potting / barriers | (in packing) | 35 | (in packing) | 25 | [EST] |
| **Parts subtotal** | **325** | **594** | **259** | **464** | [CALC] |
| Inside volume = parts / packing factor | 650 | | 432 | | [CALC] |
| Shell and gasket (2–2.5 mm walls) | +110 | 140 | +90 | 100 | [EST] |
| **Device without pads** | **0.76 L** | **734 g** | **0.52 L** | **564 g** | [CALC] |
| Pads, pre-connected | +60 | +100 | +50 | +80 | §8 [EST] |
| **Device with pads** | **0.82 L** | **834 g** | **0.57 L** | **644 g** | [CALC] |

### Against targets and commercial units

| | Volume | Mass | Energy |
|---|---:|---:|---|
| Target (REQUIREMENTS) | 0.7 L | 700 g | 150 J |
| Stretch target | 0.5 L | 500 g | 150 J |
| **This study, recommended** | 0.76–0.82 L | 734–834 g | 150 J |
| **This study, smallest plausible** | 0.52–0.57 L | 564–644 g | 150 J |
| St John Pocket AED P3 | 0.68 L | < 700 g incl. pads | 150 J |
| Schiller FRED easyport plus | 0.99 L | 780 g | up to 200 J |
| Avive Connect | 1.19–1.51 L | 953 g | 150 J |
| CellAED | 0.31 L | ~300 g | ~75–85 J |

**Plainly:**
- **700 g / 0.7 L: reachable, but not with the low-risk choices.** The recommended set lands about 10–20% over. The smallest set meets it with room to spare. Reality will sit between the two, much as the St John P3 already shows is possible.
- **500 g / 0.5 L: not reachable** with a purchasable capacitor at 190 J stored. The smallest set comes close on volume (0.52 L without pads) but not on mass: the capacitor, battery and shell alone are about 385 g, before any electronics or pads.
- **The single change that buys most** is a better capacitor. Two routes:
  - **Ordering a custom film capacitor sized to the job** (100 µF at 2.2 kV rather than an off-catalogue 130 µF part). This saves about 30 cm³ and 40 g directly, and lets the whole box shrink by roughly twice that through the packing factor.
  - A maker achieving the "up to 2 J/cm³" CDE advertises would save about 85 cm³ and 110 g of capacitor alone [CALC: 190 J / 2.0 = 95 cm³ against 180 cm³].
  - Nothing else in the list moves the total by more than about 30 g.

### Cost estimate [EST]

All prices are estimates; see the access note at the top.

| | Qty 1 (prototype) | Qty 100, per unit |
|---|---:|---:|
| Capacitor | $150–400 (sample, quote) | $80–200 |
| Charger incl. transformer samples | $100–200 | $25–40 |
| Bridge, gate drive, relays | $350–550 | $200–330 |
| Discharge load | $20–40 | $15–25 |
| Battery, gauge, charger | $40–60 | $20–30 |
| Front end + protection | $60–100 | $30–50 |
| Controllers, module, flash, sensing chains | $60–100 | $35–60 |
| User interface | $50–80 | $25–40 |
| Boards (HV-rated, 4-layer), small batch | $300–800 per batch | $20–40 |
| Enclosure (3D print or machined; tooling later) | $300–1,500 | $15–40 (after $10–40k tooling) |
| Pads (commercial replacements, for fit and impedance only) | $50–80 | — |
| **Parts for one first prototype** | **about $1,500–3,800** | **about $470–860 per unit** |
| Custom transformer and capacitor design charges (one-off) | $1,000–4,000 | — |

Test equipment (defibrillator analyser, HV probes, test loads) and engineering time are **not** included. The Fluke 7000DP class analyser alone is a major grant item (earlier notes).

### Power budget [EST unless marked]

| State | Draw | Notes |
|---|---:|---|
| Standby (worn, ready, Bluetooth advertising every 1–2 s) | 0.5–1 mW (150–300 µA at 3.3 V) | Supervisor awake periodically, gauge, regulators. 12 h ≈ 0.01 Wh |
| Rescue, not charging | 0.4–0.6 W | MCU active (about 70 mW), front end (about 2 mW), audio (0.3–0.5 W average with metronome) |
| Charging | 44–50 W for 7 s | §2 |
| Rescue average with D-29 | about 2.5–2.8 W | 30 min ≈ 1.3–1.4 Wh [CALC] |

---

## 10. Ranked risks and unknowns this study leaves open

| Rank | Risk or unknown | Why it matters | What would resolve it (later, under the lab safety policy) |
|---:|---|---|---|
| 1 | **Capacitor: availability, temperature, voltage margin, mass.** No stocked part; the 2.0 kV parts sit below the 2,008 V worst case; 55 °C working rating | Sets 30–40% of mass and volume; car heat | Quotes from 3–4 makers to the project's spec. Then measure capacitance, leakage and loss after 1,000 h at 85 °C unbiased and after 1,000 full cycles at 50 °C |
| 2 | **Front end under the charger and after the shock** | The analysis runs during charging (D-29); not modelled | Measure ECG noise and impedance noise (0.3 Ω step at 2 Hz on 25–250 Ω) with the charger running, first at ≤ 50 V output; measure ECG recovery time after a shock into a patient simulator |
| 3 | **Switch pulse ratings at milliseconds** | No datasheet states the rating at shock length; thyristor margin thin | Single 10 ms, 65 A pulses into 25 Ω; estimate junction temperature from on-voltage; then 20 shocks in 1,300 s |
| 4 | **Gate-drive insulation at 2 kV working** | No catalogue part qualifies | Partial-discharge test of the chosen barrier at 1.5× working voltage, and an insulation test after humidity soak |
| 5 | **Spacing required by the standard at 2 kV** | Could push the packing factor below 0.5 and break the 0.7 L target | Buy IEC 60601-1 and 60601-2-4; lay out a spacing-only model (no circuit) to measure the volume cost |
| 6 | **Battery in the cold** | Charge must finish in 7 s at −20 °C | Charge time and minimum cell voltage at −20 °C from 10% charge, both packs |
| 7 | **Voltage sensors over temperature and life** | D-28 is tight against drift | Both chains against a calibrated reference at three voltages and three temperatures, before and after damp-heat ageing |
| 8 | **Internal heating in a long rescue at 50 °C** | Capacitor at or over 55 °C | Thermocouples on capacitor, resistors and battery during 14 dumps at 132 s spacing in a 50 °C chamber, box sealed |
| 9 | **Discharge-resistor repeat rating** | The EY part is marketed for infrequent pulses | Maker's statement; then 100 dumps at 132 s spacing, checking resistance change |
| 10 | **Custom transformer** | Efficiency, temperature rise, insulation life | Efficiency and temperature over 14 charges; insulation test at 1.5× the output voltage |
| 11 | **1.5 m drop with a 250 g capacitor** | Lead or mount failure | Drop test on 6 faces and corners with a mass-equivalent dummy capacitor (no HV) |
| 12 | **Ferrocene scope detail** | The CPU option and the library certification scope | Ask Ferrous Systems for the 26.11.0 qualification report's target and `target-cpu` list |

---

## 11. Where the parts reality strains the decisions: questions for the owner

These are questions only. DECISIONS.md and REQUIREMENTS.md are unchanged.

1. **Car heat (REQUIREMENTS "open problem").**
   - No film capacitor found has a trustworthy storage rating above 70 °C.
   - TDK's 55 °C *working* limit leaves about 1 K of margin at the 50 °C operating target during a long rescue (§4).
   - Options:
     - (a) a "never leave in a parked car or on the dashboard" rule with a thermal pouch
     - (b) pay for a custom capacitor rated 85 °C storage and 65–70 °C working
     - (c) lower the operating maximum to 45 °C
   - Which?
2. **"Off-the-shelf parts only" (REQUIREMENTS) cannot hold literally.** Defibrillator capacitors, the HV transformer, gate-drive transformers or links, the pad connector and the pads are all made to order. Accept "made to order from a catalogue maker" for these?
3. **The 2,000 V ceiling (D-23) and capacitor ratings.**
   - The compact capacitors found are rated at exactly 2.0 kV, below the model's 2,008 V worst case.
   - Either specify 2.2 kV parts (larger), or re-run the model with a larger capacitor (125–130 µF) at a lower voltage (about 1,750 V). That would give 13–15% voltage margin on the parts that exist.
   - Re-run the model at 130 µF?
4. **Full-energy self-test frequency (D-15) against capacitor life.** Twice-daily full-energy tests use up a 2,500-cycle capacitor in about 3 years. Change to **weekly full-energy plus a daily low-voltage check**?
5. **Charge time in the cold (D-29, REQUIREMENTS −20 °C).**
   - Meeting ≤ 7 s at −20 °C favours a 2-cell battery, which adds about 20 g.
   - The alternative is to accept that charging may outlast the analysis below about 0 °C. The shock then follows after a short charging wait.
   - Which?
6. **Bluetooth code and D-24 ("only `libm`").** Every Bluetooth radio needs the chip maker's closed radio code. Put Bluetooth on a **separate pre-certified module**, so the safety processor stays clean, and treat the module's firmware as outside D-24?
7. **Supervisor technology.** A programmable logic chip (as the Jewel uses) would bring in a hardware-description language, which is a new language under D-08. A Cortex-M0+ would fall outside the Ferrocene qualification. Confirm a **Cortex-M33 supervisor from a different chip maker, in Rust**?
8. **Does the 700 g target include the pads?** The St John P3's "< 0.7 kg" does. With pads the recommended set is about 830 g; without them, about 730 g.
9. **Sensor accuracy over life (D-28).** 1% + 7 V holds at manufacture with good parts, but resistor drift uses up a third of the allowance over years. Accept periodic recalibration at service, or specify costlier low-drift dividers?
10. **IP65 and charging.** An open USB-C socket conflicts with IP65. Confirm **sealed dock contacts** for charging, the way radios do it?
11. **MAX30001G's 0–70 °C range** rules it out against the −20 °C target, even though it is the most capable compression-sensing chip. Confirm that the full temperature range comes first.

---

## What I could not verify

- **Capacitors:** masses of the GA, CDE and TDK film parts (only CRE publishes one); GA temperature ratings; TDK sizes ("on request"); the CRE maker's quality system; current lifecycle of CDE Type Q; the pulse suitability of 105 °C general-purpose electrolytics.
- **Prices and stock:** almost all prices and live stock (distributor pages blocked automated access on 27 Nov 2026). Treat every price here as an estimate.
- **Switches:** pulse ratings at shock length for every switch candidate; the G2R120MT33J pulse rating; the IXEL40N400 surge figure (datasheets conflict).
- **Front end:** the ADS1292R respiration resolution and frequency; the ADAS1000 lifecycle; the base MAX30001 temperature range; ECG recovery requirements (standard not bought).
- **Battery:** cell behaviour at −20 °C under 7–15 A; whether the P45B datasheet copy is official; the P28A specifications.
- **Controllers:** lifecycle of the STM32U585, NXP MCX-A, Sharp memory LCDs and fibre-optic links; whether Ferrocene's qualification extends to the `cortex-m33` CPU option.
- **Pads and connector:** pad mass; a 5 kV pad connector part.
- **Volumes:** spacing requirements at 2 kV, which drive the packing factor.
