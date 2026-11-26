# High-voltage safety, isolation and control electronics for a compact AED (gap-filling pass, Nov 2026)

Scope note. This pass used 25 web searches and about 12 fetches. It fills gaps left by `miniaturization_hardware.md`, so energy-storage capacitors, flyback chargers, 3.3 kV SiC availability and batteries are not repeated here.

How to read the tags:
- **[CALC]** is my own arithmetic, with the working shown.
- **[BK]** is background knowledge from training that was not re-verified in this pass. Treat it as a pointer to check, not as a citation.
- Everything else carries an inline source.

Several primary documents could not be read and are listed under Gaps: the paywalled IEC/IPC standard text, the NFPA 70E text, and the DOE-HDBK-1092 capacitor section. The Jewel SSED, the EFCOG ground-stick best practice, the SLAC threshold summary and the Cynergy3 datasheet were downloaded as PDFs and read in full text.

**Repository rule (CLAUDE.md rule 2, D-10).** Nothing in these notes is an instruction to build or power any circuit above 50 V. The sizing figures for loads, bleeders and ground sticks are research input for the lab safety policy and design files. They are not build steps, and nothing here is safe or ready for use on a person or animal.

Design point used for all calculations:
- 100 µF capacitor at 2.0 kV, which stores **200 J** [CALC: ½·100e-6·2000² = 200 J]
- Peak current about 80 A into 25 Ω

---

## 1. Creepage, clearance and insulation (IEC 60601-1 ed. 3.2, IEC 60601-2-4, IPC-2221 fallback)

### Takeaway
The free secondary sources only reproduce IEC 60601-1 Table 12 (creepage/clearance per MOPP) up to **566 V DC / 400 V rms**. Values at 2–5 kV must be read from the purchased standard. Treat them as large, probably ≥10–20 mm of creepage for 2 MOPP at 2 kV [BK, unverified]. In practice that pushes a compact design toward solid insulation: potting, insulating barriers and buried inner-layer PCB isolation. Air and surface spacing alone will not fit.

For defibrillation-proof applied parts (the ECG/impedance front end that shares the pads with the shock), the key published figures are:
- a 5 kV, 32 µF, 500 µH test pulse
- clearance widened from 2.5 mm to 4.0 mm, matching 4.0 mm creepage
- an energy-reduction test with a 25 mH inductor

### Cited Findings
**Where the requirements live**
- IEC 60601-1 clause 8.5.5 covers "Defibrillation-proof applied parts". 8.5.5.1 is defibrillation protection, 8.5.5.2 is the energy reduction test, and creepage/air clearance for these parts sits separately at 8.9.1.15 — [AS/NZS IEC 60601.1:2015 listing / ToC](https://store.standards.org.au/product/as-nzs-iec-60601-1-2015)
- IEC 60601-2-4 is the particular standard for cardiac defibrillators. The current consolidated version appears as 60601-2-4:2010+AMD1:2018+AMD2:2026 (seen in an IEC preview listing; final publication status not confirmed) — [VDE preview ed3.1](https://www.vde-verlag.de/iec-normen/preview-pdf/info_iec60601-2-4{ed3.1}b.pdf); [iTeh catalog 2010 ed.](https://standards.iteh.ai/catalog/standards/iec/ff93bdf6-b5a7-4914-bff4-aa1fa7777f40/iec-60601-2-4-2010)
- **Purchase points:**
  - IEC webstore (webstore.iec.ch)
  - ANSI/AAMI ES60601-1 and ANSI/AAMI/IEC 60601-2-4 from the AAMI/ANSI webstore ([ANSI preview of 60601-2-4:2010](https://webstore.ansi.org/preview-pages/AAMI/preview_ANSI+AAMI+IEC+60601-2-4-2010.pdf))
  - national adoptions such as BS EN or AS/NZS
  - Free previews show only the table of contents.

**Table 12 (MOPP spacings), partial reproduction by TI**

| Working voltage | 1 MOPP creepage | 1 MOPP clearance | 2 MOPP creepage | 2 MOPP clearance |
|---|---|---|---|---|
| 354 V DC / 250 V rms | 4 mm | 2.5 mm | 8 mm | 5 mm |
| 566 V DC / 400 V rms (highest row reproduced) | 6 mm | 3.5 mm | 12 mm | 7 mm |

Source: [TI SLLA563 (partial Table 12 and Table 6)](https://www.ti.com/document-viewer/lit/html/SLLA563/GUID-BB8ACACE-F248-4744-B32C-5FBCDC2B049F)

**Table 6 (solid-insulation test voltages), partial reproduction by TI.** For peak working voltage 212 < U ≤ 354 V, the test voltage is 1500 V rms for 1 MOPP and 4000 V rms for 2 MOPP on mains parts, and 1500 / 3000 V rms on secondary parts. Higher rows are not reproduced — [TI SLLA563](https://www.ti.com/document-viewer/lit/html/SLLA563/GUID-BB8ACACE-F248-4744-B32C-5FBCDC2B049F)

**General MOPP figures.** MOPP needs 4 mm creepage for basic insulation and 8 mm for double (mains level) — [ADI "Don't Get Zapped"](https://www.analog.com/en/resources/technical-articles/dont-get-zapped.html). Creepage and clearance depend on material group, working voltage and pollution degree — [TDK-Lambda MOPP/MOOP note](https://emea.lambda.tdk.com/it/KB/MOPPs-and-MOOPs-V3.pdf)

**Defibrillation-proof test** (MEDTEQ description of IEC 60601-1:2005; verify against the current text)
- **Test pulse:** a 32 µF capacitor charged to **5000 V (about 400 J)**, discharged through a 500 µH inductor (≤10 Ω) into a 100 Ω load. The simulated pulse peaks at about 4724 V, with a 9 µs rise and 2.36 ms to half amplitude.
- **Polarity:** tested at both polarities.
- **Differential set-ups:** 0.5 mH/50 Ω for the function check, and 25 mH/400 Ω for energy reduction.
- **Operator-charge criterion:** Y1–Y2 < 1 V, where 1 V corresponds to 100 µC reaching the operator.
- **Spacing:** clearance for defib-proof applied parts rises from 2.5 mm to **4.0 mm**, equal to the 4.0 mm creepage.
- **Spark gaps** bridging the patient isolation should be rated above 5 kV, about 6 kV. Gaps near 3 kV broke down in testing.
- **ECG input series resistors** should be ≥10 kΩ (50 kΩ typical). 1 kΩ is borderline.
- **Type F insulation** is typically designed for 1.5 kV rms for 1 min, which the article considers more demanding than the 5 kV pulse.
- Source: [MEDTEQ defibrillator-proof testing](https://medteq.squarespace.com/article/category/Testing)

**Energy reduction test** (8.5.5.2, per MEDTEQ): 25 mH inductor with a fixed 11 Ω resistance. Nominal values are Vp = 3934 V, 176 µs rise and 3.23 ms fall. Energy is integrated as v²/R across the 100 Ω load, about 360 J with no device connected — [MEDTEQ](https://medteq.squarespace.com/article/category/Testing). A test-equipment article puts the 100 Ω measuring resistor at ±5% and says the baseline is taken without the device under test (DUT) connected — [Today's Medical Developments](https://www.todaysmedicaldevelopments.com/article/tmd-1210-iec-60601-1-energy-measurement-techniques)

**IPC-2221 as a fallback**
- Above 500 V, IPC-2221 adds a per-volt increment to the 500 V spacing — [Altium](https://resources.altium.com/p/using-an-ipc-2221-calculator-for-high-voltage-design)
- One worked example uses B1 = 0.25 mm + 0.0025 mm/V above 500 V — [Altium](https://resources.altium.com/p/using-an-ipc-2221-calculator-for-high-voltage-design)
- Other tables list +0.0025 / +0.005 / +0.005 mm/V across columns — [JLCPCB](https://jlcpcb.com/blog/pcb-voltage-clearance-guide)
- Protoexpress lists 0.00305 mm/V — [Protoexpress](https://www.protoexpress.com/blog/importance-pcb-line-spacing-creepage-clearance)
- **Conflict noted.** The difference probably reflects revision B vs C.
- smps.us notes that the IPC-2221B values "were never revised" since 1998 — [smps.us](https://smps.us/pcbtracespacing.html)

### Inferences
- **The 5 kV, 400 J defib-proof test applies to the device's own ECG front end.** In an AED the ECG input is tied to the shock pads, so the front end sees the device's own shock, not just an external one. The usual architecture (§3) is to disconnect the ECG front end with HV relays during the shock, plus series resistors of tens of kΩ and spark-gap or TVS clamps rated above 5 kV.
- **[CALC] IPC-2221 bare-board spacing at 2000 V**, using the +0.0025 mm/V (internal-layer B1-type) and +0.005 mm/V (external) increments:
  - Internal, B1-type: 0.25 + 1500·0.0025 = **4.0 mm**
  - External uncoated (B2-type, assumed base 2.5 mm at 500 V [BK]): 2.5 + 1500·0.005 = **10 mm**
  - These are functional spacings only. They are not patient-protection (MOPP) values.
- **[CALC] Margin at 2.5 kV (10% overvoltage plus ringing).** Increments scale linearly, giving about 5.25 mm internal and about 12.5 mm external.
- **[BK] Compact-product techniques**, all unverified for specific AEDs:
  - Run HV nets on inner layers (solid insulation, with minimum prepreg thickness and plies set by the standard's "thin sheet" rules)
  - Route slots under bridge switches to break surface creepage paths
  - Use polyimide/Nomex barriers between the HV section and the logic section
  - Pot the bridge and charger secondary in silicone or epoxy, which converts creepage into solid insulation
  - Conformal coating counts only if it meets the standard's coating qualification (IEC 60664-3-type)
- **Not established:** that the patient pads need a full 2 MOPP barrier from the HV section during the shock. The shock is the intended applied energy. The insulation question is mostly operator protection (enclosure, buttons, USB, charging port) and leakage when not shocking. Confirm in the 60601-2-4 clause list.

### Gaps
- IEC 60601-1 Table 12 (and Tables 13–16 for clearance) rows for 1–5 kV working voltage: **not found** in any free source. The standard must be bought.
- Whether IEC 60601-2-4 adds its own HV-circuit spacing or dielectric-test requirements (e.g. a test voltage above the charge voltage): not retrieved.
- Energy-reduction pass criterion: commonly recalled as "no more than 10% reduction" [BK]. **Not verified.** MEDTEQ does not state it.
- IPC-2221B/C Table 6-1 B4/A6 per-volt values: **conflicting** in secondary sources. The standard must be bought.

---

## 2. H-bridge implementation: switches, gate drive, stacking, dump

### Takeaway
Physio-Control's published H-bridge uses **SCRs in three legs and a series pair of 1200 V IGBTs in one leg**, for size, cost and surge reasons, with transformer-coupled gate drive. Its internal dump runs through two same-side legs, with an inductive-resistive protective element.

SiC alternatives exist:
- **Microchip MSC080SMA330B4N** (3300 V, 80 mΩ) has I_DM = **107 A** against a needed ~80 A peak. The pulse-width condition is unknown, and a defib phase lasts milliseconds, not microseconds, so a transient-thermal (Zth) check is mandatory.
- TI UCC21750-class reinforced drivers reach only **2121 V DC working (VIOWM)**, which is marginal at 2 kV.
- The Murata MGJ2T2 gate supply is reinforced only to **800 V rms working**.

### Cited Findings
**Physio-Control H-bridge (US5824017)**
- Three legs use SCRs and one leg uses two IGBTs in series. "the maximum working voltage of presently available IGBTs is not sufficient", so each IGBT is limited to 1200 V against a system maximum of 2200 V. A resistor across each IGBT (R23/R23') balances the static voltage split.
- Capacitor 150–200 µF, charged to 100–2200 V, giving 200 J and 360 J biphasic pulses, or 1–50 J low energy.
- A short circuit can produce "a brief current of up to 400 amps".
- **Gate drive:**
  - SCR gates are driven by a 10%-duty pulse train through HV transformer T11.
  - IGBT gates are driven by a 2 MHz, 25%-duty pulse train through T21, with 30 V gate bias applied only in brief pulses.
  - Control circuitry is isolated from the defibrillator circuitry by transformers.
- **Energy dump:** the residual charge is shorted through SW1 and SW4, two legs on the same side, so no separate dump circuit is needed.
- **Protective element:** component 27, inductive plus resistive, limits dI/dt and dV/dt into the SCRs.
- Source: [US5824017](https://patents.google.com/patent/US5824017)

**Related Physio-Control patents**
- One SCR or IGBT per leg in an integrated package — [US6963773](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6963773)
- Protective component that is both inductive and resistive — [US6041254](https://patents.google.com/patent/US6041254)
- IGBT gate drive with slow turn-on and fast turn-off — [US6477413](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6477413)
- Pacing variants with all-IGBT legs — [US6968230](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6968230); [US6980856](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6980856)

**Microchip MSC080SMA330B4N**
- VDSS 3300 V, ID 43 A at Tc 25 °C and 27 A at 100 °C, **IDM 107 A**, 80 mΩ, Qg 161 nC typical, TO-247-4L with Kelvin source.
- The datasheet number is DS00005914A. The IDM test condition was not visible in the excerpt.
- Sources: [RS listing](https://uk.rs-online.com/web/p/mosfets/0813825); [Microchip product page](https://www.microchip.com/en-us/product/msc080sma330b4n-mosfet); [datasheet copy (RS)](https://docs.rs-online.com/a067/A700000015733917.pdf)
- **Part-number caution.** The no-"N" MSC080SMA330B4 is listed at 41 A and 84 mΩ — [RS](https://sg.rs-online.com/web/p/mosfets/2494129)

**Other 3.3 kV SiC parts**
- Wolfspeed CPM3-3300-R050A, a bare die: pulsed ID 104 A, limited by Tvj(max) — [Wolfspeed datasheet](https://assets.wolfspeed.com/uploads/2024/01/wolfspeed_cpm3-3300-r050a_data_sheet.pdf)
- Microchip MSCSM330AM07D3NG, a 62 mm module: 600 A pulsed, far too large — [search result / Microchip datasheet mirrors](https://docs.rs-online.com/153b/A700000015831131.pdf)
- Toshiba defines I_DP as the maximum allowable peak drain current in pulsed operation, which in practice is limited by junction temperature (Tj) and depends on pulse width — [Toshiba app note](https://toshiba.semicon-storage.com/info/docget.jsp?did=148421)

**Isolated gate driver: TI UCC21750**
- **Working voltage (VIOWM):** 1500 V rms (AC, TDDB-based) and **2121 V DC**
- **Transient (VIOTM):** 8000 V peak
- **Withstand:** 5700 V rms
- **Reinforced isolation** per DIN EN IEC 60747-17
- **CMTI:** ≥150 V/ns
- **Drive and protection:** ±10 A drive, 200 ns DESAT, 4 A active Miller clamp
- **Grade:** AEC-Q100 -Q1 version, -40 to +125 °C
- Sources: [TI UCC21750 product page](https://www.ti.com/product/UCC21750); [datasheet SLUSD78C](https://ti.com/lit/pdf/SLUSD78)

**Isolated gate-drive supplies (Murata)**
- **MGJ2T2:**
  - 2 W, +18/−3 V outputs
  - **Reinforced to 800 V rms working**
  - 6 kV DC hipot
  - 2.5 pF barrier capacitance
  - CMTI >200 kV/µs
  - >8 mm creepage and clearance
  - −40 to +125 °C
  - Designed to IEC 60664-1, no medical (60601) rating
- **MGJ3:** a **3 kV continuous limit for functional isolation only**, based on partial-discharge inception per IEC 60270
- **MGJ1T:** 1 W SMD, reinforced 800 V rms, 2.2 kV DC continuous barrier
- **MGJ6:** 3 kV DC continuous barrier
- Sources: [Murata news 2026](https://www.murata.com/en-global/news/power/isolated-gate-drive-power/2026/1116); [Murata MGJ3 datasheet](https://www.murata.com/-/media/69eece91d2ef4c22be0a67c3c949d1ce.ashx); [Arrow article](https://www.arrow.com/en/research-and-events/articles/high-isolation-dc-dc-converters-enhance-the-stability-and-safety-of-motor-operation)

**Dump and pulse-load resistors (Ohmite)**
- Tubular bulk ceramic: peak energy **750 J (smallest, 885A size) to 13,000 J (6 × 1 in)**. Bulk construction makes them non-inductive — [Ohmite tubular ceramic](https://ohmite.com/tubular-ceramic-resistor)
- EY series: 400 J single impulse — [Ohmite surge](https://ohmite.com/surge-high-energy/)
- Encapsulated SP: 630 J. Encapsulated AS/BA: 5,200 J — [Ohmite encapsulated](https://ohmite.com/encapsulated-resistors)
- OU/OUM: 10 J per 10 ms — [Ohmite surge](https://ohmite.com/surge-high-energy/)
- A Series "PulsEater" ceramic composition: up to 2800 J, variant unclear — [TTI](https://www.ttiasia.com/content/ttiasia/en/manufacturers/ohmite/products/ohmite-a-series-resistor.html)
- Vishay Dale: non-inductive (Ayrton-Perry) wirewound uses two windings of 2R each, so energy per ohm = E/(4R) when checking pulse curves — [Digi-Key / Vishay article](https://www.digikey.com/en/articles/pulse-handling-capabilities-of-vishay-dale-wirewound-resistors)

### Inferences
- **Why SCRs plus one IGBT** (from US5824017 plus [BK]). SCRs are cheap, small, and rated for very high surge current (I_TSM in the hundreds of amps to kA for 10 ms) at high blocking voltage. However, they cannot be turned off by the gate. Truncating the first phase and ending the second needs one turn-off-capable leg (the IGBT pair), or diverting current to turn the SCRs off. An SCR-heavy bridge therefore needs only one or two gate-turn-off devices.
- **[CALC] SiC conduction loss per shock.** At 78 A peak into 25 Ω, assume R_DS(on),hot ≈ 2 × 80 = 160 mΩ [BK typical temperature rise]:
  - Peak conduction power per device: 78² × 0.16 = **973 W**
  - Two devices conduct per phase, so the loss fraction is 2 × 0.16 / 25 = **1.3% of delivered energy**, about **1 J per device per 150 J shock**
  - The resulting junction temperature rise depends on Zth(jc) at about 5–10 ms, which was **not retrieved**. Compare against the datasheet transient-thermal curve before trusting IDM = 107 A.
- **Why not stack lower-voltage devices.** Series stacking of 1.2/1.7 kV devices (as the IGBT pair in US5824017) needs static balancing resistors and dynamic snubbers (RC per device) [BK]. It doubles the gate-drive channels. With 3.3 kV discrete SiC now stocked, stacking is likely worse for a compact design than single 3.3 kV parts.
- **[CALC] UCC21750 margin.** 2000 V working / 2121 V VIOWM = **94% utilisation**, with no margin for overcharge or ringing. Two ways round this:
  - Reference the drivers so that no barrier sees the full capacitor voltage, e.g. drive low-side switches non-isolated from the capacitor negative and bootstrap or transformer-drive the high side.
  - Use pulse-transformer gate drive like Physio-Control. A custom transformer can be wound for the creepage you need.
- **Gate supplies are the binding constraint.** The MGJ2T2's 800 V rms reinforced rating (about 1131 V peak [CALC: 800 × √2]) is far below 2 kV working. The likely options:
  - Transformer-coupled drive that sends gate energy and signal through one custom HV-insulated pulse transformer, as US5824017 does
  - Driving high-side gates only while the bridge is idle, with the gate held up for ≤20 ms by a capacitor charged beforehand [BK idea, untested]
- **[CALC] Internal dump resistor.** Dumping 200 J at 2 kV into e.g. 100 Ω:
  - Peak current 20 A
  - Peak power 2000²/100 = **40 kW**
  - τ = 100 Ω × 100 µF = 10 ms
  - Needs a non-inductive single-pulse rating ≥200 J with margin (×2–3 for repeated dumps during self-test)
  - A small Ohmite tubular ceramic (750 J class) or a ceramic-composition part fits. Thick film and standard wirewound do not, without pulse-curve verification.

### Gaps
- IDM pulse-width condition and Zth curve for MSC080SMA330B4N, G2R120MT33J and Nomis/CoolCAD 3.3 kV parts: not retrieved. The Farnell G2R120MT33J datasheet fetch timed out.
- Specific small-package 2.5–4.5 kV SCR and IGBT part numbers with I_TSM ratings (e.g. Littelfuse/IXYS, Infineon, Semikron): **not researched**, search budget spent elsewhere.
- Isolated gate drivers or supplies with ≥3 kV DC working and medical (MOPP) ratings: **not found**. The off-the-shelf parts found top out at 2121 V DC working (driver) and 800 V rms reinforced (supply).

---

## 3. Patient isolation and output relays

### Takeaway
Vacuum HV reed relays (Sensata/Cynergy3 D-HR series, Pickering) easily handle the **voltage**: 10–15 kV stand-off and 17 kV coil-to-contact. They carry only **2–4 A**, so they suit switching the ECG and impedance front end off the pads during the shock, **not carrying the 40–80 A shock current**. A relay that carries the full shock current must be specified with a pulse-current rating, and none was found in this pass.

### Cited Findings
**Sensata/Cynergy3 D-HR series** (high insulation resistance, "suitable for … cardiac defibrillators"). Datasheet table:

| Isolation across contacts | Contact | Max switching voltage | Switching current | Carry current |
|---|---|---|---|---|
| 5 / 7.5 / 10 / 15 kV DC or AC peak | Rhodium | 1000 V | 3 A | 4 A |
| 5 kV | Tungsten | 3500 V | 2 A | 3 A |
| 7.5 kV | Tungsten | 5000 V | 2 A | 3 A |
| 10 kV | Tungsten | 7000 V | 2 A | 3 A |
| 15 kV (n/o only) | Tungsten | 10000 V | 2 A | 2 A |

All variants share:
- **Switching power:** 50 W max
- **Contact resistance:** 50 mΩ max (Rh) or 250 mΩ max (W)
- **Insulation resistance:** 1×10¹⁴ Ω
- **Capacitance across contacts:** <0.2 pF (coil to screen grounded)
- **Life:** 10⁹ operations dry, 10⁶ at 50 W
- **Coils:** 5/12/24 V
- **Timing:** operate 3 ms (n/o) or 2 ms (n/c), release 2–3 ms
- **Isolation:** contact to coil **17 kV**
- **Temperature:** −20 to +70 °C
- **Part code:** e.g. DAT71210F-HR = n/o, tungsten, 12 V coil, 10 kV isolation, flying-lead HV connection

Source: [Sensata/Cynergy3 D-HR datasheet (RS)](https://docs.rs-online.com/8c09/0900766b816395c1.pdf); [RS listing, 7 kV, 2 A, 17 kV isolation](https://uk.rs-online.com/web/p/reed-relays/1892419)

**Pickering.** HV reed relays with a 10 kV stand-off switch 7.5 kV at 50 W, operate in about 3 ms, and list defibrillators as a typical application. The source refers to Series 68 — [Elektor](https://www.elektormagazine.com/news/so-that-s-what-a-10-kv-reed-relay-looks-like)

**Older patent.** Defibrillators need fast switching at around 5 kV, which conventional relays handle poorly — [US5359307](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/5359307)

### Inferences
- **[CALC] Front-end isolation.** An n/c (or n/o) 7.5–10 kV reed in series with each ECG lead opens before the shock (2–3 ms release) and gives >10¹⁴ Ω isolation. Its 3 A carry rating is irrelevant, because ECG current is µA.
- **The bridge already isolates the patient when idle.** All four switches are off and the patient sees the bridge leakage. A separate high-current patient relay is a single-fault mitigation against a shorted bridge switch, not a functional necessity. Many designs instead rely on two independent off-switches in each current path [BK]; for example, SCR plus IGBT in series in US5824017.
- **Contact welding.** A relay that carries shock current must be closed cold, before the bridge fires. At 80 A peak a reed's 50 W switching rating is meaningless. Only the carry and pulse rating matters, and the vendor did not publish one.

### Gaps
- TE Kilovac, Gigavac and Standex/Meder HV relays: datasheets with **pulse (ms-scale) carry current** ratings were not retrieved.
- Which relays commercial AEDs use for patient switching: not found.
- Solid-state alternatives (opto-MOS at kV level, photovoltaic-driven SiC): not researched.

---

## 4. Single-fault-safe architecture and safety MCUs

### Takeaway
The published architectures converge on three things:
- **Independent hardware authority over the shock.** That can be a separate processor or CPLD, a manual switch that closes series safety switches outside processor control, or a mutual "I'm healthy" handshake that disables therapy.
- **Charge and voltage plausibility monitoring** with a charge-time limit.
- **An internal test load** used for periodic full-energy self-test and as the dump path whenever a fault or impedance anomaly appears.

The Jewel patch WCD (FDA-approved 2025) runs a **CPLD plus a separate BLE MCU**. Element Science's own safety partitioning is not published.

On safety MCUs, the TMS570 (Cortex-R lockstep) has only stale community Rust support. Ferrocene is TÜV-qualified (IEC 61508 SIL 3 toolchain; SIL 2 for a core-library subset) on Armv7E-M and Armv8-A, which covers Cortex-M4/M7 safety MCUs. That favours an STM32/NXP Cortex-M safety MCU plus an independent supervisor.

### Cited Findings
**Single-fault principle.** A 1984 TÜV guidance premise holds that no single fault should lead to a dangerous failure. It describes single-channel monitoring (ROM/RAM/CPU, time- and logic-based program monitoring) and two-channel structures with continuous comparison, but gives no defibrillator-specific implementation — [US5097830 background](https://patents.google.com/patent/US5097830)

**Redundant processors (EP0671185B1).**
- A single processor fault can cause "erroneous treatment or erroneous omission of treatment".
- Two processors exchange periodic "I'm healthy" signals.
- If one stops hearing the other beyond a threshold, it enters a state where it cannot enable treatment.
- Source: [EP0671185B1](https://data.epo.org/publication-server/rest/v1.2/patents/EP0671185NWB1/document.html)

**US6029085 (charging and safety control).**
- A manual rescue switch closes the safety switches independently of the processor, guarding against processor malfunction.
- The patent itself notes that if a safety switch fails closed, nothing else in that embodiment blocks the shock, which is an acknowledged single-point weakness.
- Source: [US6029085](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6029085)

**US5496349.** A main and an auxiliary processor share the A/D data path — [US5496349](https://patentimages.storage.googleapis.com/92/73/94/0558daaade53e9/US5496349.pdf)

**US5919212 (Survivalink watchdog).**
- **Watchdog:** about 30 h, reset by the daily self-test or lid opening, powered independently of the real-time clock (RTC) and self-test path (claim 13). On timeout it sets the maintenance indicator and alarms. It is not described as dumping energy.
- **Impedance fault:** if electrode impedance goes out of range during analysis, charge or shock, the processor discharges the HV circuit "through internal load 98".
- **Weekly self-test:** charges the HV circuit and dumps into the internal load while monitoring charge time and capacitor voltage, flagging a fault if either is outside limits.
- **Charger interlock:** the device cannot operate while a charger is connected.
- Source: [US5919212](https://patents.google.com/patent/US5919212)

**Jewel P-WCD (FDA PMA P230022, SSED)**
- **Logic:** "Complex Programmable Logic Device (CPLD software)" plus a BLE MCU.
- **Shock timing:** an 8 s extra wait after shock confirmation lets a conscious patient defer the shock.
- **HV tests:** overvoltage protection "adjustable to a value greater than 790V". HV verification covered the internal discharge circuit, TTI measurement accuracy, full-voltage charge measurement and capacitor accuracy.
- **Shock capacity:** 20 maximum-energy shocks within 1,300 s (IEC 60601-2-4 clause 201.102.3.2). At least 10 shocks after a simulated 8-day wear.
- **Field result:** ITT inappropriate-shock rate 0.357 per 100 patient-months.
- Source: [FDA SSED P230022B](https://www.accessdata.fda.gov/cdrh_docs/pdf23/P230022B.pdf)

**Rust and safety MCUs**
- **TMS570 (Hercules):**
  - Dual Cortex-R4F/R5F lockstep with lockstep-comparator self-test; HALCoGen generates C only.
  - The only Rust BSP found is a community one (paoloteti/ti-hercules-bsp), nightly, armebv7r, last updated Dec 2020.
  - The armv7r-none-eabi target is Tier 2. The cortex-ar/aarch32 crates cover the core only.
  - Sources: [ti-hercules-bsp](https://github.com/paoloteti/ti-hercules-bsp); [rustc platform support](https://doc.rust-lang.org/stable/rustc/platform-support/armv7r-none-eabi.html); [cortex-ar](https://docs.rs/crate/cortex-ar/0.3.0); [HALCoGen](https://www.ti.com/tool/HALCOGEN); [Hercules resource guide](https://software-dl.ti.com/hercules/hercules_docs/latest/hercules/)
- **Ferrocene:**
  - TÜV SÜD-qualified for ISO 26262 ASIL D, IEC 61508 SIL 3 and IEC 62304 Class C.
  - A first subset of Rust `core` was certified to IEC 61508 SIL 2 (Dec 2025).
  - Qualified targets include Armv7E-M and Armv8-A.
  - Sources: [Business Wire, Dec 2025](https://www.businesswire.com/news/home/20251203818102/en); [The Register](https://www.theregister.com/2025/12/04/rust_core_library_partly_polished/); [Ferrocene GitHub](https://www.github.com/ferrocene/ferrocene)

### Inferences
**Recommended pattern** (synthesised from the patents above plus [BK]). Three independent channels must all agree before energy reaches the patient:
1. **Main MCU** (rhythm analysis, UI): requests charge and shock.
2. **Independent supervisor** (a second small MCU or a CPLD/FPGA like Jewel's) with its own clock and its own capacitor-voltage ADC/comparator. It holds the bridge gate-drive *power* enable. It enforces:
   - a charge time-out
   - a maximum armed time, after which it auto-dumps (AEDs typically disarm after about 30 s without a shock [BK])
   - an overvoltage trip (as Jewel's adjustable OVP)
   - the "I'm healthy" handshake
3. **Hardware interlock** in the energy path. Examples:
   - the shock button wired directly into the gate-drive enable chain, as US6029085 does
   - a lid or pad-detect switch
   - a fail-safe normally-closed dump path (relay or depletion-mode switch) across the capacitor that must be actively held open, so loss of power or control dumps the energy [BK]
- **Redundant capacitor-voltage sensing.** Two independent dividers (main MCU ADC and supervisor comparator) cross-checked against charge-time plausibility. US5919212 checks charge time and voltage against limits.
- **Self-test.** Periodic full or partial charge into the internal dump resistor, which proves the charger, bridge switches and dump path. US5919212 does this weekly. Shorted-switch detection needs each leg exercised individually at low voltage [BK].
- **Failure-to-shock vs inadvertent shock pull in opposite directions.** Every interlock added against inadvertent shock is a new failure-to-shock path. EP0671185 names both failure modes, and the risk analysis (ISO 14971) has to balance them explicitly.
- **MCU choice.** For a Rust-first project:
  - **Cortex-M4/M7 safety MCUs** with safety manuals fit the Ferrocene qualification envelope (Armv7E-M): e.g. the ST STM32 families with IEC 61508/60730 self-test libraries, or NXP S32K1 (Cortex-M4F with ECC) [BK on vendor library names; not verified this pass].
  - **TMS570**, with true lockstep, means C/HALCoGen or a hand-built Rust BSP, and no Ferrocene-qualified Armv7-R target was confirmed.
  - **Infineon AURIX** (TriCore) has no mainstream Rust target found.
  - Pragmatic split: Cortex-M main MCU in Rust, plus a CPLD or small separate MCU as supervisor.

### Gaps
- Philips/Heartstream, Zoll and Avive safety architectures: no primary source found this pass.
- STM32 (e.g. ST's "SIL" families), NXP S32K3 (Cortex-M7 lockstep), Renesas RA/RX: lockstep and ECC features, safety-manual availability and Rust HAL status were **not verified**. Search budget ran out.
- Whether Ferrocene added Armv7-R or Armv8-M targets in 2026: not checked.
- Full text of US5919212 beyond the first 100k characters: not read.

---

## 5. Ruggedness: IEC 60601-1-12, MIL-STD-810, vehicle temperatures, cold batteries

### Takeaway
The IEC 60601-1-12 (EMS environment) numeric requirements (temperature range, drop height, IP) are **behind the paywall**. Free previews confirm only:
- the scope: carried to an emergency scene and used there or in transport
- clause 10.1 on mechanical strength, with a sub-clause for road-ambulance-fixed equipment and one for airborne use (10.1.4, which Jewel cited)

Commercial AED datasheets give the de-facto targets:
- 0–50 °C operation
- IP55/IP56 sealing
- MIL-STD-810G vibration
- a 1.22–1.5 m drop (Method 516.6 Procedure IV)

Parked-car cabins reach about **47–70 °C air and about 70–100 °C on the dashboard**, beyond the 55 °C rating of defibrillator film capacitors noted in the earlier notes.

### Cited Findings
**IEC 60601-1-12**
- Scope: equipment carried to the emergency scene and used there or during transport. Ed 1.0 2014 + Amd 1:2020 (consolidated 1.1). Includes a clause on environmental conditions, including transport and storage between uses, and 10.1 mechanical strength with a road-ambulance sub-clause — [ISO 59536](https://www.iso.org/standard/59536.html); [IEC webstore](https://webstore.iec.ch/publication/2589); [VDE preview ed1.1](https://www.vde-verlag.de/iec-normen/preview-pdf/info_iec60601-1-12{ed1.1}b.pdf)
- Not applicable to home-only or hospital-only equipment — [ANSI/AAMI preview](https://webstore.ansi.org/preview-pages/AAMI/preview_ANSI+AAMI+IEC+60601-1-12-2016.pdf)

**Jewel ruggedness tests.** Drop per IEC 60601-1 clause 15.3.4.1 (body-worn/hand-held drop), shock and vibration per IEC 60601-1-12 §10.1.4 (airborne), and RTCA DO-160G — [FDA SSED P230022B](https://www.accessdata.fda.gov/cdrh_docs/pdf23/P230022B.pdf)

**Commercial AED ratings**

| Product | Operating range | Sealing | Mechanical |
|---|---|---|---|
| Philips HeartStart | 0–50 °C operating/standby | IPX5 per IEC 60529 | vibration per MIL-STD-810G |
| Powerheart G5 | 0–50 °C | IP55 | MIL-STD-810G 516.6 Procedure IV, 1.22 m drop |
| Stryker law-enforcement flyer (model not named) | not given | IP56 | not given |
| Zoll AED Plus (blog source only) | not given | IP55 | 1.5 m drop |

Sources: [Philips spec sheet](https://www.documents.philips.com/assets/20200909/e7d23b651f0044cc8e06ac31011b5d20.pdf); [Powerheart G5 listing](https://www.news-medical.net/Powerheart-G5-AED-Automatic-from-Cardiac-Science); [Stryker LE flyer](https://www.stryker.com/content/dam/stryker/ems/resources/flyers/aeds-for-law-enforcement-us.pdf)

**AED storage ranges**
- One manufacturer: short-term storage −20 to 60 °C (≤14 days), long-term 5–40 °C — [AED.com](https://www.aed.com/blog/high-temperatures-putting-your-aed-at-risk)
- LIFEPAK CR2: storage 15–35 °C — [Stryker CR2 storage](https://mts.stryker.com/support/solutions/articles/60000615278-lifepak-cr2-storage)
- Resuscitation Council UK (2026): "Heat inside parked vehicles can exceed safe limits, and cold weather may require insulated cases" — [RCUK briefing, AEDs in vehicles](https://www.resus.org.uk/sites/default/files/2026-07/RCUK%20briefing%20AEDs%20in%20vehicles%20_1.pdf)

**Cabin temperatures**
- ASU/UCSD (2018, *Temperature* journal): sun-parked cars averaged 47 °C (116 °F) cabin air and 69 °C (157 °F) dashboard after 1 h. Shaded cars averaged 48 °C (118 °F) dashboard — [ASU news](https://news.asu.edu/20180516-discoveries-asu-study-hot-cars-can-hit-deadly-temperatures-within-one-hour)
- Baghdad study (secondary report): about 70 °C cabin air and a dashboard approaching 100 °C — [Gulf News](https://gulfnews.com/amp/story/best-buys%2Fwhy-a-parked-car-hits-70c-in-the-sun-1.500645677)
- 2026 sunshade test: 60 °C cabin after 90 min, surfaces >70 °C — [Carscoops](https://www.carscoops.com/2026/06/sunshade-heat-test/)

**Cold and batteries**
- An extremely cold battery takes longer to charge the AED and can falsely indicate low battery — [US5897576](https://patents.google.com/patent/US5897576)
- Panasonic CR123A (LiMnO₂) for AEDs: 8-year-equivalent cells kept about 83% capacity. A low-temperature variant under development holds ≥1.5 V under pulse at −20 °C at 50–70% depth of discharge — [Panasonic AED batteries](https://energy.panasonic.com/ap/business/information/articles/aed)
- LIFEPAK 1000 offers both LiMnO₂ (non-rechargeable) and Li-ion batteries — [Stryker LIFEPAK 1000 batteries](https://mts.stryker.com/support/solutions/articles/60000614771-lifepak-1000-battery-options)
- Jewel uses a disposable primary lithium battery unit, qualified per IEC 60086-4 — [FDA SSED](https://www.accessdata.fda.gov/cdrh_docs/pdf23/P230022B.pdf)

### Inferences
- **[CALC] Heat soak.** A belt device left on a dashboard can see about 70–100 °C on the surface and about 60–70 °C in cabin air. That exceeds:
  - TDK B32365's +55 °C maximum working temperature (from the earlier notes)
  - photoflash aluminium electrolytics' 65 °C rating (earlier notes)
  - typical Li-ion charging limits (≈45 °C [BK])
- Design consequences:
  - Specify storage to at least 70 °C
  - Inhibit charging of both the battery and the HV capacitor above a set temperature
  - Tell users explicitly not to leave the unit on the dashboard
- **Charge-time headroom from MIL-STD-810.** For a police product, cite Methods 500 (altitude), 501/502 (high/low temperature), 507 (humidity), 514 (vibration) and 516 (shock/drop, Procedure IV transit drop), with the procedure numbers stated [BK on the method list; 516.6 Proc IV confirmed via the G5 listing].
- **Cold.** Primary LiMnO₂ AED packs hold pulse voltage at −20 °C (Panasonic data). Li-ion power capability falls steeply below 0 °C [BK]. Expect longer charge time in the cold, and verify the 2-4 "20 shocks within 1,300 s"-type requirement at the cold limit.

### Gaps
- IEC 60601-1-12 numeric requirements (operating −20/+50 °C? drop height? IP22/IP34?): **not found**. The standard must be bought.
- Quantitative −20 °C pulse-power comparison of Li-ion vs LiMnO₂: **not found**.
- The Avive temperature-limit page returned 403.

---

## 6. ICD and wearable-defibrillator energy storage

### Takeaway
ICDs use 2–4 series electrolytic capacitors (wet tantalum or stacked aluminium) at roughly **2–10 J/cm³ per patent claims**, compared with about 1 J/cm³ for AED film capacitors. They are custom, hermetic, about 375–450 V per cell parts, not distributor-stocked.

The Jewel's FDA documents give the shock as 150 J then 162 J BTE, a capacitor qualified to 800 cycles at 50 °C, and an "overvoltage protection … >790 V" setting. **They do not disclose capacitor type, voltage or device volume.**

### Cited Findings
- **ICD capacitor volume:** HV capacitors can occupy about one third of ICD volume. ICDs typically use 2–4 electrolytics in series, commonly wet tantalum — [US10957493](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10957493)
- **Wet tantalum energy density:** patent claims of ≥2.0 J/cm³, about 3.0 J/cm³ in some embodiments, and 3.5–10 J/cm³ in others — [US11450486](https://patentimages.storage.googleapis.com/0d/ea/64/909311659b3880/US11450486.pdf)
- **Film target:** >5 J/cc for defib capacitors. One existing film design reached only 2 J/cc at about 375 V — [US6426864](https://patents.google.com/patent/US6426864); [US7558051](https://patents.google.com/patent/US7558051B2/en)
- **Jewel:**
  - Shocks: 150 J, then up to 4 × 162 J per salvo, ≥10 shocks before the replacement alert
  - HV capacitor qualification: 800 charge/discharge cycles at 50 °C
  - Overvoltage protection adjustable to >790 V
  - Source: [FDA SSED P230022B](https://www.accessdata.fda.gov/cdrh_docs/pdf23/P230022B.pdf)

### Inferences
- The ">790 V" OVP figure in the Jewel SSED may indicate a lower-voltage stage or capacitor string, e.g. series cells each about 400 V class. That is speculative: the SSED does not say what node the OVP watches.
- A 150 J Jewel-style unit at ICD-class 3 J/cm³ would need about 55 cm³ of capacitor [CALC: 165 J stored / 3 = 55 cm³], against about 200 cm³ at film's 0.9 J/cm³.

### Gaps
- Vishay (Evans) hybrid wet tantalum and stacked aluminium ICD capacitors: purchasability in small quantity, prices and datasheets **not researched** (budget).
- Jewel device volume and mass, and LifeVest/Kestra ASSURE capacitor technology: not found. The Jewel SSED has no dimensions.

---

## 7. Developer lab safety for defibrillator work

### Takeaway
A 200 J, 2 kV defibrillator capacitor exceeds the NFPA 70E (2021/2024) capacitor hazard threshold, **"≥400 V and >0.25 J"**, about **800-fold**. Required controls, from the NFPA 70E / EFCOG / national-lab practice reviewed:
- an engineered bleeder and dump
- permanent voltage indication
- a ground stick (soft ground above 1 kJ; current limited to <500 A in any case)
- shorting straps left on as visual proof of zero energy
- awareness that dielectric absorption can recharge electrolytics to about 10% of voltage minutes after discharge

### Cited Findings
**NFPA 70E capacitor thresholds** (2021 Article 360 / Annex R, as summarised by SLAC and EFCOG). Controls are required above any of:
- **<100 V and >100 J**
- **≥100 V and >1 J**
- **≥400 V and >0.25 J**

Above 1 kJ, soft grounding comes before hard grounding (Annex R.11.4). Above 100 kJ, specialised analysis and remote discharge are required. SLAC uses the 2024 edition — [SLAC Hazardous Energy Thresholds, 2025](https://www-group.slac.stanford.edu/esh/eshmanual/references/electricalReqHazEnergyThresholds.pdf); [EFCOG BP261 Ground Stick best practice, 2023](https://efcog.org/wp-content/uploads/Wgs/Safety%20Working%20Group/Best%20Practices/Worker%20Safety%20-%20Health/Electrical%20Safety/BP261%20Using%20Ground%20Sticks%20%28Hooks%29%20to%20Discharge%20Capacitance.pdf)

**Other thresholds**
- Berkeley Lab: capacitor threshold ≥100 V and ≥10 J — [LBL](https://electricalsafety.lbl.gov/hazardous-mean-electrical-safety/)
- PPPL: ≤10 J can be an indirect (startle) hazard; 11–49 J can cause hazardous shocks and burns — [PPPL ES manual](https://bp-pub.pppl.gov/ESHD_MANUAL/safety/es6.0.pdf)

**EFCOG BP261 controls**
- **Allowed controls:**
  - bleeder resistors
  - an actuated dump system
  - permanent voltage-indicating devices
  - portable contact meters for zero-energy verification
  - ground sticks
- **Leave ground sticks on.** They "should be left in place for all safing procedures, regardless of energy" as visual indication of the safe state, unless drain wires are approved.
- **Peak current:** keep discharge current under **500 A**, because of magnetic forces on the cable.
- **Stick resistor:** must be rated for full capacitor voltage, peak power and total energy. Choose a critically damped or over-damped discharge, because voltage reversal from ringing damages capacitors.
- **Dielectric absorption:** a recharge of **0.02% (polypropylene) up to 10% (some electrolytics)** can occur minutes after the short is removed.
- **Marx and parallel-charge/series-discharge banks:** failure analysis must cover an isolated element staying fully charged. Directly relevant to Avive-style switched stacks and photoflash banks.
- **Connections:** avoid alligator clips above 1000 J.
- **Stick testing:** dielectric test every 2 years per ASTM F711.
- **Gloves:** no dielectric glove is rated above 40 kV (class 4). Keep hands outside the restricted approach boundary.
- **Qualification:** users must be qualified electrical workers, with demonstrated proficiency.
- **Design for safing:** design bleeders, isolation and grounding points into the original design.
- **Soft-ground wait time:** T_d = −ln(V_target/V_p)·RC.
- Source: [EFCOG BP261](https://efcog.org/wp-content/uploads/Wgs/Safety%20Working%20Group/Best%20Practices/Worker%20Safety%20-%20Health/Electrical%20Safety/BP261%20Using%20Ground%20Sticks%20%28Hooks%29%20to%20Discharge%20Capacitance.pdf)

**Automatic discharge timing.** Capacitors should reach ≤50 V within the time it takes a person to reach the terminals, and never more than 5 minutes. The page does not name the governing standard — [Virginia Tech EHS capacitors](https://ehs.vt.edu/programs/occupational-safety/electrical-safety-in-research-operations/capacitors.html)

**Hard vs soft grounding** (Field-Guide-16-based training) — [JLab capacitor safety](https://www.jlab.org/sites/default/files/training/Capactitor%20Safety%20KD%20Final.pdf)
- Hard ground: an insulated stick shorts the terminals, then the terminals go to ground.
- Soft ground: an insulated stick with a series resistor discharges first, then a hard ground follows after the prescribed wait.

**DOE-HDBK-1092** (2013): the capacitor hazard classification tables formerly in this handbook now live, improved, in EFCOG Best Practice #252 — [EFCOG BP261 §6.2](https://efcog.org/wp-content/uploads/Wgs/Safety%20Working%20Group/Best%20Practices/Worker%20Safety%20-%20Health/Electrical%20Safety/BP261%20Using%20Ground%20Sticks%20%28Hooks%29%20to%20Discharge%20Capacitance.pdf); [DOE-HDBK-1092-2013 PDF](https://www.energy.gov/sites/default/files/2026-04/DOE-HDBK-1092-2013.pdf)

### Inferences
**[CALC] Hazard ratio.** 200 J / 0.25 J = **800×** the NFPA 70E threshold. Even a 1 µF charger output capacitor at 2 kV (2 J) exceeds it 8×.

**[CALC] Ground stick for 200 J / 2 kV / 100 µF**
- No soft-ground requirement, since 200 J < 1 kJ. The <500 A rule still needs R ≥ 2000/500 = **4 Ω**.
- A practical choice is R = 1 kΩ:
  - Peak current 2 A, τ = 0.1 s
  - Time to fall below 0.25 J (V < √(2·0.25/100e-6) = 70.7 V): t = 0.1·ln(2000/70.7) = **0.33 s**
  - Time to ≤50 V: 0.1·ln(40) = 0.37 s
  - Resistor must take a 200 J single pulse and a 4 kW peak at a 2 kV rating, e.g. an Ohmite tubular ceramic (750 J class, §2).

**[CALC] On-board passive bleeder** (always connected, last-resort safety). With V₀ = 2000 V, C = 100 µF, target ≤50 V (ln 40 = 3.69):

| R | τ | Time to 50 V | Power at 2 kV | Energy lost during a 10 s hold |
|---|---|---|---|---|
| 0.8 MΩ | 80 s | 295 s (meets 5 min) | 5 W | about 50 J (25% of the stored energy) |
| 2 MΩ | 200 s | 738 s ≈ 12 min | 2 W | about 20 J |
| 10 MΩ | 1000 s | 3689 s ≈ 61 min | 0.4 W | about 4 J |

- **Conclusion:** a passive bleeder alone cannot meet a 5-minute rule without wasting energy. Use an **active, fail-safe dump** (normally-closed path held open only while armed, §4) as the primary means, plus a 2–10 MΩ passive bleeder as backup.
- The bleeder string must be split across several series resistors for voltage rating, e.g. 1206 resistors are usually rated ≤200–400 V each [BK].

**[CALC] Dielectric absorption**
- A photoflash electrolytic bank recovering 10% of 2 kV = 200 V gives ½·100 µF·200² = **2 J**. That is above the 1 J (≥100 V) threshold, so it is hazardous after "discharge". Keep a shorting strap on whenever not under test.
- A polypropylene film capacitor recovers 0.02% = 0.4 V, which is negligible.

**Recommended lab policy elements** (synthesis; items marked [BK] lack a source here):
1. Treat every HV board as energised until verified with a rated meter or probe.
2. Interlocked clear-polycarbonate enclosure that cuts the charger and fires the dump when opened [BK].
3. Make the policy stage the voltage. Work below 50 V (project rule) until the policy is adopted. A later reduced-energy stage, e.g. 400 V / 10 µF = 0.8 J, still exceeds the NFPA threshold, so it needs the full controls [CALC].
4. Two-person rule whenever charging above threshold: commonly required by lab HV programs [BK; not found in the sources retrieved].
5. One hand behind the back / hands outside the restricted approach boundary (RAB) [BK; RAB from EFCOG].
6. HV differential probes rated ≥CAT/≥2.5 kV differential, never a grounded scope probe on floating nodes [BK].
7. A ground stick and a labelled shorting strap at each bench; leave the strap on when idle (EFCOG).
8. Bench isolation transformer for mains-powered chargers [BK].
9. Test loads (§8) in a closed box.
10. Never connect the device to a person. Test only into resistive loads or analysers.

### Gaps
- NFPA 70E Article 360 primary text, the DOE-HDBK-1092 capacitor chapter, and a primary source for the 5-minute/50 V rule: not read directly.
- Defibrillator **manufacturer service-manual** safety procedures (e.g. Zoll/Philips/Stryker HV discharge procedures) and biomedical equipment technician (BMET) practice guidance: **not found** this pass.
- A published two-person rule for HV labs: not found in the sources retrieved.

---

## 8. Defibrillator test loads and analyzers

### Takeaway
All commercial analyzers use a **50 Ω ±1% non-inductive load** with ±1% energy accuracy. A DIY load needs:
- 50 Ω non-inductive at **≥40 kW peak and ≥200–400 J single pulse**, for which bulk-ceramic resistors are the natural choice
- a calibrated HV divider and current measurement
- energy computed as ∫v·i dt, or ∫v²/R dt, sampled at ≥100 kS/s

### Cited Findings
**Fluke Impulse 6000D/7000DP**
- **Energy accuracy:** ±1%; load non-inductive (<2 µH) 50 Ω ±1%
- **Energy range:** 0.1–600 J per a reseller, vs 0.1–360 J in an older Fluke catalog (**conflict**)
- **Voltage:** 20–5000 V, ±(1% + 2 V)
- **Current:** 0.4–100 A, ±(1% + 0.1 A)
- **Pulse width:** 1–50 ms, ±0.1 ms
- **Power limit:** 12 W average maximum, equivalent to 10 × 360 J per 5 min
- **Scope output:** 50 Ω impedance
- **Load accessory:** the 7010 selectable load gives 25–200 Ω in 25 Ω steps, max 5000 V
- Sources: [Fluke 7000DP getting-started](https://www.flukebiomedical.com/sites/default/files/resources/7000dp_gseng0007.pdf); [Fluke defib brochure](https://www-live.flukebiomedical.com/sites/default/files/resources/defib_brochure_eng_a_w.pdf); [Fluke 7010 load](https://www-live.flukebiomedical.com/sites/default/files/resources/7010_eng_a_w.pdf); [MedicalExpo catalog](https://pdf.medicalexpo.com/pdf/fluke-biomedical/impulse-6000d-7000dp-2013/68507-99329.html)

**Rigel (Seaward) UniPulse 400**
- 50 Ω ±1% non-inductive, 0–199.9 J, ±1% of reading ±0.1 J
- FDA 510(k) K182905 cites the Impulse 7000DP as predicate
- Sources: [Seaward datasheet](https://seaward.com/us/support/medical/unipulse-400/manual/unipulse-400-datasheet-en); [FDA K182905](https://www.accessdata.fda.gov/cdrh_docs/pdf18/K182905.pdf)

**BC Group DA-2006 series**
- Cardioversion high range up to 1,000 J
- Optional 25–200 Ω variable load (DA-2006-VL)
- Accuracy figure not retrieved
- Source: [BC Group datasheet](https://assets.testequity.com/te1/Documents/pdf/bc-group/BC-Group_DA-2006-Series_Datasheet_1125.pdf)

**Datrend Phase 3**
- 0–600 J and 0–50 J ranges, ±1% ±2 LSD
- From an unofficial Scribd manual copy; verify with Datrend
- Source: [Scribd manual](https://www.scribd.com/document/719348464/Phase-3-Operators-Manual-1)

**Energy integration method.** The defib-proof test integrates v²/R across the 100 Ω load — [MEDTEQ](https://medteq.squarespace.com/article/category/Testing)

### Inferences
**[CALC] DIY 50 Ω load for 2 kV**
- Peak current 2000/50 = 40 A
- Peak power 2000²/50 = **80 kW**
- Energy up to 200 J per shock, plus repeat-shock duty (20 shocks in about 20 min ⇒ average 200·20/1300 s ≈ 3.1 W, small)
- **Component class that fits these ratings** (design input only; not a build instruction until the D-10 lab safety policy exists): one Ohmite tubular bulk-ceramic resistor (750 J+) or a few in series-parallel to share voltage. That is non-inductive by construction.
- **Avoid:** standard wirewound (inductance) and thick-film chip resistors (single-pulse limits of a few joules [BK]).
- **Check:** voltage rating per element ≥ its share of 2 kV with margin. Inductance <2 µH (Fluke's own criterion).

**[CALC] Measurement**
- **Voltage:** ≥100:1 HV divider (e.g. 10 MΩ : 100 kΩ, compensated) or a commercial HV differential probe.
- **Current:** a non-inductive coaxial shunt (e.g. 10 mΩ gives 0.8 V at 80 A) or a Rogowski/Pearson-type current transformer [BK].
- **Sampling:** phase durations are about 2–10 ms, so ≥100 kS/s gives ≥200 points per phase.
- **Energy:** E = Σ v[k]·i[k]·Δt (direct), or Σ v[k]²/R·Δt with R measured at 4-wire.

**Validation.** A DIY load and integrator should be cross-checked once against a calibrated commercial analyser. Rental or used Impulse 7000DP / UniPulse 400 units are the usual route [BK].

### Gaps
- Kanthal/HVR non-inductive pulse resistors: no data retrieved.
- Datrend Phase 3 and BC DA-2006 accuracy figures from primary datasheets: not verified.
- The Fluke 600 J vs 360 J energy-range conflict is unresolved.
