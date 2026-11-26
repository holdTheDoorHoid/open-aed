# Miniaturizing an AED with off-the-shelf parts: electronics and component review (late 2026)

Scope note: this pass used about 25 search/fetch calls. The strongest primary data is CDE's defibrillator film capacitor table, Chemi-Con's PH photoflash capacitor ratings, TDK's MKP defibrillator brochure, two Avive patents, SiC MOSFET distributor and datasheet listings, and the published specs of commercial AEDs. Many questions in the brief, including creepage tables, MIL-STD-810, cold-temperature battery curves, safety MCUs and ICD capacitors, were **not** researched to primary sources in this pass. They are listed under Gaps rather than filled from memory. Anything marked **[CALC]** is my own arithmetic, with the working shown. Anything marked **[EST]** is my own engineering estimate or assumption and has no source.

---

## 1. Energy-storage capacitor (the dominant size driver)

### Takeaway
Commercial AED film capacitors store only about **0.75–1.26 J/cm³ at their rated voltage**, which works out to roughly 0.9 J/cm³ when run at about 1.95 kV. A 200 J-class film capacitor is therefore about **280–520 cm³** on its own, already about a third to half the volume of the whole Avive Connect. A series-parallel bank of 330 V photoflash aluminium electrolytics stores about **2.15 J/cm³ per can at its rating**, roughly 2x the film figure. It is the most plausible off-the-shelf route to a sub-200 cm³ bank, but it brings cold, hot-car, aging and string-balancing penalties.

### Cited Findings
**Film capacitors (metallized polypropylene) — what AEDs use**
- Cornell Dubilier (CDE) sells defibrillator film capacitors (Type Q, "MPP (Hp)" dielectric) in round or oval metal oil-filled cases or dry epoxy-filled plastic cases. They span 32–500 µF and 800–6,000 VDC and deliver "in excess of 500 joules". The catalog is dated 2012 — [CDE Defibrillator catalog](https://www.cde.com/resources/catalogs/CDEDefibrillator.pdf)
- CDE part table (inches), from the same catalog:
  - QL232EW120V: 120 µF, 2300 V, 317 J, oval metal (oil), 1.91 x 2.91 x 3.75 in
  - QL232EW195V: 195 µF, 2300 V, 516 J, 1.91 x 2.91 x 5.75 in
  - QV232EW195N: 195 µF, 2300 V, 516 J, oval plastic (dry), 1.77 x 2.75 x 5.75 in
  - QL232EW240V: 240 µF, 2300 V, 635 J, 1.97 x 3.66 x 5.88 in
  - QR172YW105V: 105 µF, 1760 V, 163 J, round metal, 1.88 dia x 4.75 in
  - QR222EW105V: 105 µF, 2200 V, 254 J, 2.13 x 4.75 in
  - QR252EW090V: 90 µF, 2500 V, 281 J, 2.63 x 4.75 in
  - QS222EW195N: 195 µF, 2200 V, 472 J, round plastic (dry), 2.57 x 4.69 in
  - QR232EW195V: 195 µF, 2300 V, 516 J, 2.38 x 5.60 in
  - Source: [CDE Defibrillator catalog](https://www.cde.com/resources/catalogs/CDEDefibrillator.pdf)
- A CDE press release describes the Q series as "up to 2 J/cc" (from the search snippet; the page now redirects to Knowles and the content would not load) — [CDE Q series news](https://cde.com/news/2020/4/q-series); [Type Q page](https://www.cde.com/new-product/type-q/). The catalog dimensions do not support 2 J/cc for the listed parts (see the CALC below), so the 2 J/cc figure is probably for newer or custom designs or counts only the winding. **Conflict noted.**
- TDK/EPCOS B32365 MKP (metallized polypropylene) defibrillator capacitors:
  - 30–200 µF, up to 5500 V DC, energy 16–529 J
  - Life expectancy up to 10,000 cycles
  - Maximum working temperature +55 °C
  - Self-healing; polyurethane or epoxy sealing; plastic or metal case; cylindrical or oval
  - Silicone leads rated 15 kV DC
  - The brochure (2020) gives no dimensions table in text form; the capacitance chart is an image
  - Source: [TDK MKP film capacitors for defibrillators product brief](https://www.tdk-electronics.tdk.com/download/2887416/9d4efc0f98fe0a32cdc4daff91cff523/film-capacitors-defi-pb.pdf)
- Patent literature sets a requirement of at least 5 J/cc for implantable defibrillators. It cites about 1 J/cc as typical for polyester film and about 4 J/cc for PVDF. These are patent claims, not product specs — [US6514276B2, metallized film capacitor for implantable defibrillator](https://patents.google.com/patent/US6514276); [US 6426861 high energy density metallized film](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6426861)

**Aluminium electrolytic photoflash capacitors**
- Chemi-Con PH series (photoflash):
  - Ratings: 300/315/330 Vdc, −20 to +65 °C
  - Leakage: I ≤ 1×C µA after 5 min at 20 °C
  - tanδ: 0.06 max at 120 Hz
  - Charge/discharge test: 5,000 cycles into 0.7–1.0 Ω or a xenon tube, with ≤±10% capacitance change afterwards
  - Shelf-life test: 1,000 h at 65 °C unbiased, then reconditioned "according to JIS C 5101-4" before measurement. This is the reforming issue.
  - Largest 330 V part: **194 µF, 12.5 x 40 mm, EPH-331E□□1K1C540S**. At 315 V: 209 µF, 12.5 x 40 mm.
  - Source: [Chemi-Con PH series catalog](https://www.chemi-con.co.jp/products/relatedfiles/capacitor/catalog/PH-LL-e.PDF)
- Rubycon FK photoflash capacitors come in 330/350/360/390 V ratings with lug terminals — [Rubycon FK datasheet (Mouser)](https://www.mouser.com/datasheet/2/977/e_FK-1878647.pdf)
- A Digi-Key forum post puts photoflash parts at about 1 J/cc and 10,000–100,000 discharge cycles. This is a forum claim, not a manufacturer figure — [Digi-Key forum](https://forum.digikey.com/t/photoflash-capacitors-vs-conventional-capacitors/43911)

**Ceramic (antiferroelectric PLZT, TDK CeraLink)**
- CeraLink capacitance density is 4.9 µF/cm³ (at DC-link voltage), against 0.7 µF/cm³ for film — [TDK CeraLink presentation](https://www.tdk-electronics.tdk.com/download/1195592/d48b20bb4b1eee698626fa5d5a57d2e0/ceralink-presentation.pdf)
- The 900 V LP type is 0.25 µF in a 10.84 x 7.85 x 4 mm body — [EPCOS/TDK press 2017](https://www.epcos.com/170323)
- The 900 V SMD 2220 parts (B58043I9563M052 / B58043E9563M052) are 0.056 µF — [TDK CeraLink 900 V article](https://www.tdk-electronics.tdk.com/en/374108/tech-library/articles/products-technologies/products-technologies/ceralink-types-with-higher-rated-voltage/2081694)

**What the small commercial AEDs use**
- CellAED: a third-party critique says it uses "separate capacitors" to supply each phase independently (a 50:50 energy split). It also says the device promotes 75 J defibrillation, against a 150 J guideline minimum. This is a secondary source, not a patent — [The CellAED critique](https://adam-gent-bi3h.squarespace.com/cellaed); [CellAED brochure](https://www.protrainings.uk/blog/wp-content/uploads/2023/12/CellAED®-The-latest-development-in-defibrillation-technology-and-therapy.pdf)
- Avive: patent US 11,607,555 says a single discharge capacitor "is typically used". An alternative embodiment charges four or five 600 V capacitors in parallel and switches them into series for discharge — [Justia US 11,607,555](https://patents.justia.com/patent/11607555)

### Inferences
- **[CALC] Film energy density from CDE box dimensions** (1 in³ = 16.387 cm³; E is the catalog joules at rated V):
  - QL232EW120V: 1.91×2.91×3.75 = 20.84 in³ = 342 cm³ → 317/342 = **0.93 J/cm³**
  - QV232EW195N (dry plastic): 459 cm³ → **1.13 J/cm³**
  - QL232EW240V: 695 cm³ → **0.91 J/cm³**
  - QR172YW105V (round): π/4·1.88²·4.75 = 13.19 in³ = 216 cm³ → **0.75 J/cm³**
  - QS222EW195N: 399 cm³ → **1.18 J/cm³**
  - QR232EW195V: 408 cm³ → **1.26 J/cm³**
- **[CALC] Derating to about 1.95 kV operation.** Energy scales with V². QL232EW120V at 1950 V stores ½·120 µF·1950² = **228 J** in 342 cm³ (48.5 × 73.9 × 95.3 mm), or 0.67 J/cm³. QR222EW105V at 1950 V stores **200 J** in 277 cm³.
- **[CALC] Photoflash density.** EPH-331 194 µF/330 V: E = ½·194e-6·330² = **10.56 J**. Cylinder volume π/4·1.25²·4.0 = 4.91 cm³, giving **2.15 J/cm³** (cylinder only, at full rating). That is about 2x the CDE film parts.
- **[CALC] A 2 kV photoflash series/parallel bank.**
  - Seven in series at 279 V each (85% of 330 V) gives 1953 V. Each can then stores ½·194e-6·279² = 7.55 J.
  - 5 strings × 7 = 35 cans: **264 J**, effective C = 5·194/7 = **139 µF**. Cylinder volume 172 cm³; about **219 cm³** boxed, using π/4 square-packing.
  - 4 strings × 7 = 28 cans: **211 J**, 137 cm³ of cylinders, about 175 cm³ boxed.
  - Balancing resistors are needed per can, which adds standby drain only while charged.
  - ESR estimate from tanδ: 0.06/(2π·120 Hz·194 µF) = 0.41 Ω per can. The string of 7 is 2.9 Ω; 5 strings in parallel are about 0.57 Ω. That is about 2% of a 25 Ω patient. ESR at the real pulse frequency will differ — **[EST]**.
- **[EST] Electrolytic penalties.** The PH series stops at +65 °C, below the cabin temperature of a parked patrol car in summer (no sourced figure found for that temperature). Film (TDK) is rated only to +55 °C working. Both need thermal management or derating for vehicle storage. Electrolytics also need reforming after long unbiased storage (see the Chemi-Con shelf-life preconditioning clause). A daily-charged, belt-worn device can run a periodic low-current "reform" self-test cycle, which a 4-year-standby public AED cannot easily do.
- **[EST] Mass.** No capacitor masses were found. If capacitor bulk density is assumed at about 1.2–1.6 g/cm³ (an unsourced assumption), a 220 cm³ electrolytic bank is about 260–350 g and a 280–340 cm³ film part about 340–540 g. At the low end, that alone uses up the "few hundred grams" target.
- **CeraLink is not a bulk-energy option.** **[CALC]** 0.25 µF at 900 V = 0.10 J in 0.34 cm³ ≈ 0.3 J/cm³, below film. It is also only 900 V, so it would need stacking.

### Gaps
- No primary datasheet with masses for CDE Type Q or TDK B32365; the TDK dimension chart is an image.
- What is inside CellAED, Avive Connect, FRED easyport and HeartSine (part numbers or teardowns): **not found**.
- ICD stacked aluminium and wet tantalum capacitors (energy density, vendors), glass, polymer-nanocomposite and supercapacitor-hybrid availability: not researched in this pass.
- Electronicon, Vishay, KEMET and AVX defibrillator film lines: not checked.
- Lifecycle status (active or NRND) of CDE Q and TDK B32365: not confirmed. The CDE catalog dates from 2012.

---

## 2. Capacitor charger

### Takeaway
Charging ~230 J in 8 s takes about 29 W out and about 36 W in **[CALC]**. ADI's LT3750/LT3751 flyback capacitor-charger controllers are the obvious off-the-shelf parts. However, every published reference design and off-the-shelf transformer (Würth WE-FB) found here targets 300–450 V, not 2 kV. There are two ways to use them: a custom high-ratio transformer with a 2 kV-rated secondary, or charge low-voltage capacitors in parallel and switch them into series (the Avive patent approach).

### Cited Findings
- The LT3751 is a high-input-voltage flyback controller that "rapidly charge[s] a large capacitor to a user-adjustable high target voltage". It comes in 20-pin QFN or TSSOP. Its listed applications are photoflash, strobes and detonators, not defibrillators — [ADI LT3751](https://www.analog.com/en/products/lt3751.html)
- In one ADI design, the output trip point is adjustable from 50 V to 450 V. The DC1322A demo board charges 400 µF to 300 V — [ADI article](https://www.analog.com/en/resources/technical-articles/dcdc-converter-capacitor-charger-takes-inputs-from-475v-to-400v.html); [DC1322A guide](https://shop.richardsonrfpd.com/docs/rfpd/dc1322a.pdf)
- On the LT3750, the target voltage is set by the flyback transformer's turns ratio plus two resistors — [ADI LT3750 article](https://www.analog.com/en/resources/technical-articles/tiny-controller-makes-it-easy-to-charge-large-capacitors.html)
- Würth WE-FB 3751 transformer for the LT3751: 3–24 V in, 300 V out, 1:10 ratio — [Würth LT3750A reference](https://www.we-online.com/en/components/icref/analog-devices/LT3750A-Flyback)
- ADI notes that the transformer secondary must withstand both positive and negative excursions, and that this is not the same as the isolation rating (via search summary of ADI material) — [ADI LT3750 page](https://www.analog.com/en/products/lt3750)
- An ADI EngineerZone thread asks whether the LT3750/51 can be used in defibrillators; the answer could not be retrieved (timeout) — [EZ thread](https://ez.analog.com/power/f/q-a/595404/lt3750-lt3751-for-medical-devices)
- Avive WO2018111688A1 describes a "valley switching flyback converter" charging the capacitor. It also describes a bank of capacitors that can be selectively included in or excluded from the active stack — [WO2018111688A1](https://patents.google.com/patent/WO2018111688A1/en)

### Inferences
- **[CALC] Power budget.** Stored 230 J ÷ 8 s = 28.8 W out. At an assumed 80% efficiency **[EST]**, that is 36 W in. Average input current is 10 A from a 1S Li-ion cell (3.6 V), 5 A from 2S, or 2.5 A from 4S. Flyback peak primary current is a multiple of the average **[EST]**, which favours 2S–4S.
- **[EST] Turns ratio.** Reaching 2 kV from 7.2 V with a flyback needs a turns ratio of very roughly 1:30–1:60, depending on duty and reflected voltage. That means a custom or semi-custom transformer with a secondary insulated for ≥2 kV, plus a ≥2.5 kV, fast-recovery output diode. No catalog 2 kV capacitor-charging transformer was found.
- **[EST] Using a photoflash bank.** Charging each series string of a photoflash bank would still need a full 2 kV charger. The alternative is a per-can charging arrangement, either the parallel-charge/series-switch topology (see §4) or a multi-winding transformer with one 300 V secondary per series level. The latter lets you use a standard LT3751 at about 300 V per winding and balances the string by construction.

### Gaps
- TI photoflash or capacitor-charger ICs, and the efficiency and charge-time data of real AED chargers: not found.
- Measured benefit of GaN or SiC on the charger primary: not found.
- LT3750/LT3751 lifecycle status: not confirmed.

---

## 3. High-voltage output bridge

### Takeaway
Each H-bridge leg must block the full capacitor voltage, so a 2 kV design needs **3.3 kV-class** switches. Distributors now stock discrete 3.3 kV SiC MOSFETs in TO-247-4 and TO-263-7 packages: Microchip MSC080SMA330B4N and MSC400SMA330B4N, GeneSiC G2R120MT33J, plus new CoolCAD and Nomis parts. Worst-case peak current at 25 Ω is about 78 A **[CALC]**, so pulsed ratings need checking against real datasheets.

### Cited Findings
- **Microchip MSC080SMA330B4N**: 3300 V, 41 A continuous, TO-247-4, stocked at Digi-Key — [Digi-Key](https://www.digikey.ca/en/products/detail/microchip-technology/MSC080SMA330B4N/24818448)
- **Microchip MSC400SMA330B4N**: 3300 V, 400 mΩ, 11 A (Tc), 131 W, TO-247-4 — [Digi-Key](https://www.digikey.com.br/en/products/detail/microchip-technology/MSC400SMA330B4N/24818411); [datasheet](https://manualmachine.com/microsemi/msc400sma330b4/24843528-datasheet)
- **GeneSiC G2R120MT33J**: 3300 V, 120 mΩ, TO-263-7 (SMD), body-diode pulse current 116 A — [datasheet (Farnell)](https://www.farnell.com/datasheets/3189227.pdf)
- New 3.3 kV listings at Digi-Key: CoolCAD CC3380354L and Nomis Power N3T080MP330 — [CoolCAD](https://www.digikey.com.mx/en/products/detail/coolcad/CC3380354L/28223313); [Nomis](https://www.digikey.com.mx/en/products/detail/nomis-power/N3T080MP330/27524843)
- **Wolfspeed C2M1000170J**: 1700 V, 1 Ω, 5.6 A continuous at 25 °C, **15 A pulsed**, TO-263-7. It is at **Last Time Buy**; the replacement is C3M0900170J — [Wolfspeed datasheet](https://assets.wolfspeed.com/uploads/2024/01/Wolfspeed_C2M1000170J_data_sheet.pdf); [Wolfspeed product page](https://www.wolfspeed.com/c2m1000170j)
- Prior art (not Avive) describes a simplified biphasic circuit with "make-only" switching and two capacitors. It notes that a large series inductor adds weight and space in a small AED — [US8145300B2](https://patents.google.com/patent/US8145300B2/en)

### Inferences
- **[CALC] Peak current at 1950 V.** 78 A into 25 Ω, 39 A into 50 Ω, 11 A into 175 Ω. This ignores internal series resistance. Real AEDs reduce the 25 Ω peak through impedance-compensated voltage or pulse width and internal resistance **[EST]**.
- The 1.7 kV C2M1000170J-class parts (15 A pulsed) are far too small for the main bridge. They fit only an energy-dump or charger-side switch.
- **[EST] Bridge size.** Four TO-263-7 3.3 kV MOSFETs, four isolated gate-driver channels with isolated bias, and a patient-isolation relay come to about 15–30 cm³ including creepage spacing.
- Truncation and phase reversal are done by switching the bridge (and dumping the residue into an internal resistor). No primary source was retrieved for specific commercial implementations.

### Gaps
- Pulsed drain current (I_DM) for MSC080SMA330B4N and G2R120MT33J: not retrieved.
- Isolated gate drivers rated for 3.3 kV working voltage, and HV patient relays (part numbers, ratings): not researched.
- How Philips, Zoll and Physio-Control actually build their bridges (IGBT vs SCR, stacking): not found in this pass.

---

## 4. Architectures that reduce size

### Takeaway
The best-documented size-reducing idea is **parallel-charge / series-discharge of lower-voltage capacitors**, from Avive's patents. It allows a low-voltage charger and smaller, denser capacitors, at the cost of many more HV switches. CellAED reportedly uses separate capacitors per phase and a lower 75 J energy, and that energy is clinically contested.

### Cited Findings
- **Avive US 11,607,555** ("Defibrillator discharge control", filed Dec 2020, issued Mar 2023): an alternative embodiment charges "four or five 600 volt capacitors" in parallel and switches them into series after charging — [Justia](https://patents.justia.com/patent/11607555)
- **Avive WO2018111688A1**:
  - A capacitor bank with a switching structure lets individual capacitors be included or excluded.
  - It notes that parallel-charge/series-discharge requires "more extensive switching and discharge control".
  - A second embodiment keeps the low-voltage capacitors in series, or in parallel, for both charge and discharge.
  - Source: [Google Patents](https://patents.google.com/patent/WO2018111688A1/en)
- US20080177341A1 switches capacitors into series/parallel combinations based on measured patient impedance — [Google Patents](https://patents.google.com/patent/US20080177341A1/en)
- CellAED: separate per-phase capacitors and a 75 J claim, against a 150 J guideline minimum (secondary critique) — [CellAED critique](https://adam-gent-bi3h.squarespace.com/cellaed)
- Schiller FRED easyport plus: energy levels 150/200/200 J (retailer listing) — [defibwarehouse via search](https://defibwarehouse.co.uk/shop/defibrillators/schiller-fred-easyport-plus-fully-automatic-defibrillator)

### Inferences
- **[EST]** Parallel-charge/series-discharge with 600 V parts would need 4 series levels for 2 kV. Each level transition needs a switch able to stand off the accumulated voltage, so the switch count and gate-isolation burden grow with the number of levels. With photoflash cans at about 300 V, there would be 7 levels.
- **[EST]** A lower-energy waveform (as CellAED reportedly uses) shrinks the capacitor roughly in proportion to energy, but the regulatory and clinical risk is large.

### Gaps
- Schiller's "pulsed/chopped" biphasic (Multipulse) mechanism, inductor-based current-controlled output, and boost-during-discharge publications: not retrieved.
- CellAED and Rapid Response Revival patents: **not found** by search.

---

## 5. Battery

### Takeaway
Energy is not the limiting factor. One **[CALC]** 200 J charge takes about 0.08 Wh from the battery, so a single 21700 high-drain Li-ion cell (Molicel P45B: 4.5 Ah, 45 A continuous, about 70 g) holds energy for well over a hundred charges. It also handles a 10 A, 1S charging load easily. Commercial AED batteries are about 17–19 Wh: HeartSine uses 4 × LiMnO2 cells at 17 g each, and the FRED easyport plus uses a 12 V / 16.8 Wh pack.

### Cited Findings
- **Molicel P45B (21700)**:
  - 4500 mAh nominal (Keeppower lists 4300 mAh minimum), 45 A continuous
  - Mass about 70 g max; one teardown measured 66.7 g
  - One retailer lists only 35 A continuous, a conflict
  - Sources: [NKON](https://www.nkon.nl/en/rechargeable/molicel-inr21700-p45b-4500mah-45a.html); [batterydesign.net tests](https://www.batterydesign.net/genuine-p45b-tests/)
- **Avive Connect**: rechargeable Li-ion, not user-replaceable. It is rated for "≥75 shocks at 150J, or 2.5 hours of continuous emergency operation" — [Avive battery specs](https://www.avive.life/battery-specs); [Avive AED specs](https://www.avive.life/aed-specs)
- **HeartSine 350P/360P Connected**: four 3 V LiMnO2 cells, 17 g each, 1550 mAh typical. Other sheets describe an 18 V LiMnO2 Pad-Pak giving about 60 shocks at 200 J — [Stryker connected datasheet](https://www.stryker.com/content/dam/stryker/ems/products/heartsine-350p/resources/heartsine-350p-360p-connected-data-sheet.pdf); [Safelincs 360P datasheet](https://safelincs.co.uk/templates_safelincs/files/datasheets/5954_HeartSine%20Samaritan%20PAD%20360P%20Datasheet.pdf)
- **Schiller FRED easyport plus**: battery 12 V / 16.8 Wh; works with primary or rechargeable batteries; about 80 shocks at maximum energy — [UGAP sheet via search](https://www.ugap.fr/images/media-wp/2023/03/rfart_3427847_ft.pdf.avif); [Schiller brochure](https://cdn.asp.events/CLIENT_Nineteen_F7A1E73F_5056_B725_6BE81DEAAC036B43/sites/ESS2025/media/libraries/exhib-brochures/143664-2.500782-2025-08-en-fred-easyport-plus-pp-lq.pdf)
- **Base FRED easyport**: non-rechargeable lithium, 5-year standby, up to 45 shocks at maximum energy (retailer) — [Safelincs](https://safelincs.co.uk/schiller-fred-easyport-defib-battery)

### Inferences
- **[CALC]** HeartSine pack: 4 × 3 V × 1.55 Ah = 18.6 Wh in 68 g of cells.
- **[CALC]** One P45B: 3.6 V × 4.5 Ah = 16.2 Wh in about 70 g. One 230 J charge at 80% efficiency = 288 J = 0.08 Wh, giving about 200 charges of theoretical energy before electronics and monitoring loads.
- **[EST]** A belt-worn device recharged like a radio could use one 18650 or 21700 high-drain cell, or a 2S small-pouch pack (about 45–90 g). A 4-year standby primary pack would be avoided. Charging Li-ion below 0 °C needs to be blocked by the charger (general knowledge; no source retrieved here).

### Gaps
- Cold-temperature (−20 °C) pulse capability curves for P45B, LiMnO2 and LiSOCl2: **not found**.
- UN38.3 / IATA shipping rules and LiFePO4 options: not researched.

---

## 6. ECG front end and control

### Takeaway
Off-the-shelf ECG front ends (TI ADS1298 family) are protected with external series resistors (10–22 kΩ) and clamps (neon or TVS). The protection parts themselves degrade with repeated shocks.

### Cited Findings
- TI's ADS1298 noise report says the main noise sources are the 10 kΩ defibrillation-protection resistors, and describes 22 kΩ and 10 kΩ current-limiting resistors between electrode and input — [TI SBAA185](https://ti.com/lit/pdf/sbaa185)
- ADI (Maxim) "Lightning Bolts, Defibrillators, and Protection Circuitry Save Lives": typical networks use series resistors, neon lamps or TVSs, and clamp diodes. It says protection devices degrade with repeated shocks and recommends replacing them at least annually — [ADI article](https://www.analog.com/en/resources/technical-articles/lightning-bolts-defibrillators-and-protection-circuitry-save-lives.html)
- TI E2E advises not relying on the internal ESD diodes, and treats 100 mA as the safe momentary input-current assumption (forum, not official) — [TI E2E ADS1298 input protection](https://e2e.ti.com/support/data-converters-group/data-converters/f/data-converters-forum/813151/ads1298-input-protection)

### Inferences
- **[EST]** In an AED the same electrodes carry both the shock and the ECG, so the ECG front end must survive a full ~2 kV shock on every use, not only an occasional one from another device. Input recovery time after a shock matters for post-shock rhythm analysis.

### Gaps
- ADS1291/1292R, AD8232, ADAS1000 and MAX30001/30003 suitability and their defibrillation-recovery specs; impedance measurement methods; safety MCUs (lockstep); audio, display and cellular/BLE/GNSS modules: **not researched in this pass**.

---

## 7. HV layout, safety and environment

### Takeaway
Little primary data was gathered. Known constraints: the film capacitor is rated to +55 °C working (TDK) and the photoflash electrolytic to +65 °C (Chemi-Con), both below likely patrol-car cabin extremes.

### Cited Findings
- TDK B32365: maximum working temperature +55 °C; terminals use 15 kV DC silicone leads — [TDK brochure](https://www.tdk-electronics.tdk.com/download/2887416/9d4efc0f98fe0a32cdc4daff91cff523/film-capacitors-defi-pb.pdf)
- Chemi-Con PH: −20 to +65 °C — [Chemi-Con PH](https://www.chemi-con.co.jp/products/relatedfiles/capacitor/catalog/PH-LL-e.PDF)

### Gaps
- IEC 60601-1 creepage and clearance values at 2–2.5 kV, IEC 60601-2-4 specifics, potting and coating guidance, MIL-STD-810 drop and vibration profiles, and IP ratings: **not retrieved**. Do not size the HV layout from this file.

---

## 8. Size and weight budget

### Takeaway
**[EST]** With off-the-shelf parts, a 200 J device plausibly lands at about **450–700 cm³ and 450–800 g**, and a 150 J device about 15–25% smaller. That is roughly half the volume of the Avive Connect. However, it is well above CellAED's reported ~300 g unless the energy is cut. The capacitor is 30–50% of both volume and mass.

### Cited Findings
- Avive Connect: 953 g with cartridge. Dimensions are 143 × 160 × 66 mm on one spec page and 143 × 160 × 52 mm on another (conflict) — [Avive specs](https://www.avive.life/aed-specs); [Avive weight FAQ](https://help.avive.life/hc/en-us/articles/25762546720795-How-much-does-the-Avive-Connect-AED-weigh)
- HeartSine 360P: about 1.1 kg and 20 × 18.4 × 4.8 cm. Other sheets give 0.7 kg or 1.5 kg (conflict) — [BHF bid spec](https://www.bhf.org.uk/-/media/externally-referenced-files/ecommerce-files/defibrillators/heartsine-350p-360p-bid-specs-uk)
- Schiller FRED easyport: 490 g (retailer); FRED easyport plus: 780 g (manufacturer) — [WMS](https://www.wms.co.uk/Equipment/Diagnostic/Defibrillators/Fred-easyport-Defibrillator/p/W11024); [Schiller](https://www.schiller.ch/en/tovary/fred-easyport-plus-p51)
- CellAED: about 300 g per the brief; no spec was retrieved in this pass.

### Inferences
**[CALC]** Volumes of the comparison devices: Avive 1,190–1,510 cm³; HeartSine 1,766 cm³.

**[EST] Budget for 200 J delivered (about 230–260 J stored):**

| Block | Volume (cm³) | Mass (g) | Basis |
|---|---|---|---|
| Storage: photoflash 5×7 bank (264 J) **or** CDE film (e.g. QL232EW120V at 1.95 kV, 228 J) | 220 **or** 342 | 260–350 **or** 400–550 | CALC above; mass assumes 1.2–1.6 g/cm³ (unsourced) |
| Charger (flyback IC, transformer, primary FET, HV diode) | 15–30 | 20–50 | EST |
| H-bridge (4× 3.3 kV SiC, drivers, isolated bias, relay, dump resistor) | 15–30 | 20–40 | EST |
| Battery (1× 21700 P45B, or 2S pouch) | 24–40 | 70–100 | P45B ~70 g |
| ECG, MCU, audio, display, radio, PCB | 30–60 | 30–60 | EST |
| Enclosure, HV spacing, potting, pad connector (~30–50% overhead) | 100–250 | 80–200 | EST |
| **Total** | **~400–750** | **~480–950** | EST |

- For 150 J (about 175–200 J stored), use 4×7 photoflash cans (211 J, about 175 cm³ boxed) or QR222EW105V at 1.95 kV (200 J, 277 cm³). That gives a total of **[EST] ~350–620 cm³, ~420–800 g**. Electrodes and pads are not included.
- **[EST]** Reaching CellAED's ~300 g at 150–200 J with off-the-shelf capacitors does not look feasible from these densities. It would need either lower energy or non-catalog capacitors.

### Gaps
- Capacitor masses, real enclosure overhead and pad mass: not found.

---

## 9. Development and test equipment

### Takeaway
The Fluke Impulse 7000DP measures 0.1–360 J into a 50 Ω default load. The 7010 accessory adds 25–200 Ω loads in 25 Ω steps, which covers the 25–175 Ω test range.

### Cited Findings
- Impulse 7000DP:
  - Energy 0.1–360 J, ±(1% reading + 0.1 J) (catalog excerpt, truncated)
  - Default load 50 Ω
  - Waveshapes: Lown, Edmark, trapezoidal, DC biphasic, AC pulsed biphasic
  - Meets IEC 60601-2-4 and AAMI DF80
  - Sources: [Fluke 7000DP spec](https://www.flukebiomedical.com/sites/default/files/resources/7000dp_gseng0007.pdf); [7010 load accessory](https://www-live.flukebiomedical.com/sites/default/files/resources/7010_eng_a_w.pdf)

### Gaps
- Rigel defibrillator analyzers, HV differential probes, bench test loads and published lab safety practice for lethal stored energy: not researched in this pass.
