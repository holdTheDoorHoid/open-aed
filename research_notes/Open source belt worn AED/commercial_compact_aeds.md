# Commercial landscape of the smallest, lightest AEDs (as of Nov 2026)

Research date: 2026-11-26. All numbers are **manufacturer or distributor claims** unless marked otherwise. I found **no independent bench measurements** (weights, delivered energy, battery capacity) for any device covered. Volumes are my own arithmetic from the published dimensions (box volume, not displaced volume).

## Q0. What is the "Rive AED"?

### Takeaway
No product literally named "Rive" AED was found. The most likely intended device is the **Avive Connect AED** (Avive Solutions, "a-VIVE" is an easy voice mis-transcription to "Rive"). Confidence: moderate-high (~75%). A secondary possibility is Defibtech's **Lifeline/ReviveR** brand name (FDA PMA P160032), but those are not ultra-compact devices, so in the context of "small enough for a belt" Avive is the better match.

### Cited Findings
- A search for "Rive" AED returned no product of that name; the nearest names were Avive and Defibtech "Lifeline/ReviveR DDU-100 / DDU-120 / VIEW DDU-2300" — [FDA PMA P160032 via Innolitics](https://fda.innolitics.com/device/P160032)
- Avive AED FDA PMA **P210015**, approved **31 Oct 2022**, applicant Avive Solutions Inc., indicated for adults and children over 1 year — [FDA PMA database](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=P210015); [FDA "recently approved devices"](https://www.fda.gov/medical-devices/recently-approved-devices/avive-automated-external-defibrillator-aed-avive-aed-pad-cartridge-avive-aed-training-cartridge)
- GUDID listing "Avive AED with Avive Connect" (00860002508559) — [AccessGUDID](https://accessgudid.nlm.nih.gov/devices/00860002508559)

### Inferences
- If the requester actually meant CellAED ("cell" also does not sound like "Rive"), it is covered in full below. Recommend confirming with the requester; the engineering conclusions do not change much because both are covered.

### Gaps
- Could not ask the requester; no way to fully rule out a regional/obscure brand.

## Q1. Per-device fact sheet (weight, size, energy, battery, environment, regulatory, connectivity)

### Takeaway
The commercially available floor in late 2026 is roughly **300 g / 0.31 L (CellAED, single-use, ~75-85 J, company now reportedly in administration)**, then a cluster of **~0.7-0.8 kg / 0.7-1.0 L** devices that deliver full 150-200 J (St John Pocket AED P3, Schiller FRED easyport plus), then **~0.95-1.1 kg / 1.2-1.8 L** (Avive Connect, HeartSine 350P/360P/450P). Mainstream AEDs (Philips, Zoll, Stryker CR2, Defibtech) are 1.4-3 kg. A realistic belt-worn target that still delivers guideline 150 J biphasic is therefore **≤700 g and ≤0.7 L**, already demonstrated commercially.

### Summary table (manufacturer claims)

| Device | Weight | Dimensions (mm) | Box volume | Energy adult / ped | Waveform | Battery | Shocks | IP | Op. temp | Paeds handling | Connectivity | Regulatory |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| CellAED (Rapid Response Revival / RRR Manufacturing, AU) | ~300 g; 450 g with case | 196 x 93 x 17 | ~0.31 L | "~75 J" (spec-sheet/marketing) or "85 J" (Ambulance Victoria FAQ v1.3, Apr 2024); manual reportedly says "up to 150 J" — conflicting | Biphasic, multi-capacitor (each phase own capacitor) | Non-rechargeable, pre-charged; single-event device | Max 20 shocks (single event, then whole unit replaced) | not found | not found | Pads for patients >10 kg; no separate paeds mode found | Cellular (stays connected when in range) | TGA (AU), CE (EU/UK); no FDA approval found |
| St John Pocket AED, model P3 (AU, launched Jul 2026; UK "HEART Pocket" Apr 2026 likely same device) | <0.7 kg incl. battery & pads | 53 x 86 x 150 | ~0.68 L | 150 J / 50 J at 50 Ω (non-escalating implied) | BTE | LiMnO2 disposable, 12 V / 1500 mAh (18 Wh) | 130 shocks at 150 J (new, 20 °C); 30 after low-battery warning | IP65; 1.5 m drop | -15 to +50 °C; -20 °C for 1 h | Single pad set adult+child; "Child button" | 4G/LTE eSIM (paid), Bluetooth, geolocation | TGA, CE-MDR; FDA not found |
| Schiller FRED easyport plus (CH) | 780 g | 46 x 150 x 143 | ~0.99 L | "up to 200 J" | Multipulse Biowave | Rechargeable or disposable options; external charger holds 2 packs | not found | IP44 | not found | Same electrodes, press child-mode button | Wi-Fi + USB data transfer; ECG display; manual/semi/auto versions | CE; FDA not found |
| Avive Connect (Avive Solutions, US) | 953 g (2.1 lb) with pad cartridge | 143 x 160 x 66 (spec page) or x 52 (help centre) | 1.51 L (66 mm) / 1.19 L (52 mm) | 150 J / 50 J, non-escalating | BTE, 25-200 Ω | Li-ion rechargeable, not user-replaceable; 12 h to full (6 h to 85%) | ≥75 shocks at 150 J or 2.5 h operation; 10 shocks after low battery | IP54 (cartridge installed) | 0-50 °C | Separate pediatric pads, anterior-posterior placement | Cellular + Wi-Fi to REALConnect cloud; GPS tracking | FDA PMA P210015 (Oct 2022) |
| HeartSine samaritan 350P/360P/450P (Stryker) | 2.4 lb (~1.09 kg) per NFL chart; 1.1 kg not confirmed from Stryker | 200 x 184 x 48 | ~1.77 L | 150, 150, 200 J escalating / 50 J | (SCOPE biphasic; not re-verified here) | Pad-Pak: combined single-use battery + electrodes cartridge, 4-yr life | not found | IP56 | not found | Pediatric Pad-Pak (separate cartridge) | 450P CPR feedback; Wi-Fi via "connected" models | FDA PMA; CE |
| Philips HeartStart OnSite | 3.3 lb (~1.5 kg) | not found | — | 150 J / 50 J (paeds pads) | — | 4-yr battery | — | — | — | Separate infant/child pads | Cellular via attachment | FDA PMA |
| Philips HeartStart FRx | 3.5 lb (~1.6 kg) | not found | — | 150 J / 50 J | — | 4-yr | — | — | — | Infant/child key | Cellular via attachment | FDA PMA |
| Defibtech Lifeline VIEW | 3 lb (~1.36 kg) | not found | — | 150 J / 50 J | — | 4-yr | — | — | — | Paeds pads | none | FDA PMA P160032 |
| Stryker LIFEPAK CR2 | 4.5 lb (~2.0 kg) | not found | — | 150, 150, 200 J escalating / 50, 50, 50 J... (chart lists ped 50 J) | — | 4-yr | — | — | — | Child-mode button, same pads | Wi-Fi + cellular | FDA PMA |
| Zoll AED 3 | 5.5 lb (~2.5 kg) | not found | — | 120/150/200 J / 50/70/85 J | Rectilinear biphasic | 5-yr | — | — | — | Child-mode button, universal pads | Wi-Fi | FDA PMA |
| Zoll Powerheart G5 | 5.7 lb (~2.6 kg) | not found | — | 95-354 J (impedance-compensated, escalating) / 22-82 J | — | 4-yr | — | — | — | Paeds pads | Wi-Fi w/ Rescue Ready Monitor | FDA PMA |

Sources for the table rows are listed in Cited Findings below.

### Cited Findings
**Avive Connect**
- 2.1 lb (953 g) with pad cartridge; 143 x 160 x 66 mm; IP54 with cartridge; 150 J adult / 50 J pediatric; BTE non-escalating; 25-200 Ω; Li-ion rechargeable not user replaceable; charge 12 h (6 h to 85%); ≥75 shocks at 150 J or 2.5 h; 10 shocks / 20 min after low battery; standby "up to 12 months" per full charge; screen/connectivity battery 9-12 months low-power mode, 2-3 months high-power mode; 0-50 °C operating; storage -20 to 60 °C ≤14 days, 5-40 °C long term; altitude to 4,572 m; pad cartridge shelf life 3.5 years; 1.3 m leads; ≥1 h ECG/event recording; cellular + Wi-Fi + GPS; ML-based rhythm analysis ("Rhythm Recognition Detector"); adult pads anterior-lateral, paeds pads anterior-posterior — [Avive spec page](https://www.avive.com/features/aed-specifications)
- Depth conflict: help centre gives 143 x 160 x 52 mm (2.05 in) — [Avive help centre](https://help.avive.life/hc/en-us/articles/25764077740571-Technical-Specifications), vs 66 mm on the spec page above. Unresolved.
- FDA PMA P210015, approved 31 Oct 2022 — [FDA PMA database](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=P210015)
- Note: Avive needs periodic recharging (12-month standby per charge) — unlike every other device here, which uses long-life primary cells. This is a real operational trade-off for belt carry (charging discipline) vs. a benefit (rechargeable, high shock count, connectivity budget).

**CellAED**
- ~300 g, 450 g with case; L 19.6 x W 9.3 x D 1.7 cm; single-event use, max 20 shocks (IEC 60601-2-4); "uses 85 joules to deliver an equivalent shock to a traditional AED"; "regulatory-approved biphasic waveform that enables effective defibrillation at a lower energy"; non-rechargeable pre-charged battery; LED status (green/amber/none); approved shelf life 24 months; no servicing; cellular connectivity; used in Ambulance Victoria FIRST randomised trial of GoodSAM volunteer responders (~24 months) — [Ambulance Victoria FIRST trial FAQ, v1.3 Apr 2024](https://www.ambulance.vic.gov.au/sites/default/files/2025-06/AV-FIRST-Clinical-Trial-FAQ.pdf)
- Manufacturer claims ~75 J per shock via multiple capacitors (each phase has its own energy) giving "equivalent current" to conventional AEDs; "comparing different AEDs based on their energy usage is not a comparison of effectiveness" — as summarised by search from [ProTrainings CellAED whitepaper (Dec 2023)](https://www.protrainings.uk/blog/wp-content/uploads/2023/12/CellAED®-The-latest-development-in-defibrillation-technology-and-therapy.pdf) and [CellAED spec sheet](https://res.cloudinary.com/hwos9pjeb/raw/upload/v1672120877/media/DOC-2104_BCH_Customer_Tech_Spec_Sheet_v2.pdf) (spec sheets returned 401/not fully read — energy figure from search snippet, unverified)
- Energy figure conflict: 75 J (marketing) vs 85 J (AV FAQ) vs "up to 150 J" (reportedly in the instruction manual, per a critic's blog) — critic blog [adam-gent squarespace "The CellAED – What is the hype about?"](https://adam-gent-bi3h.squarespace.com/cellaed) (page now 404; content known only from search snippet — unverified)
- Pads cover patients >10 kg (22 lb) — [CellAED Device Manual via Ambulance Victoria](https://www.ambulance.vic.gov.au/sites/default/files/2025-06/CellAED-Device-Manual.pdf) (search snippet)

**St John Pocket AED (P3)**
- 5.3 x 8.6 x 15 cm; <0.7 kg incl. battery & pads; claimed "2/3 reduction in weight and 3/4 reduction in volume" vs traditional AEDs; "Patented Tiny Combine technology"; IP65; 1.5 m drop; BTE; 20-180 Ω; 150 J adult / 50 J child at 50 Ω; semi-automatic (shock button); analysis typically 8 s; charge-to-shock "typically within 17 s" from start of analysis; LiMnO2 disposable 12 V/1500 mAh; 130 shocks at 150 J (new, 20 °C), 30 shocks after low-battery; standby 3 yr (weekly self-test, LTE off) or 2 yr (daily self-test, LTE off); runtime ≤6 h; pads 5-yr standby, adult+child universal, ≥1.0 m cable, stored in rear compartment; operating -15 to +50 °C, -20 °C for 1 h; storage 0-50 °C; transport -40 to +70 °C ≤1 week; altitude -382 to 5,000 m; Bluetooth + 4G + geolocation (eSIM paid subscription); stores 24 h ECG; pacemaker pulse detection; TGA + CE-MDR; ISO 13485; IEC 60601-2-4:2010+A1:2018 and IEC 60601-1-12 (EMS environment) listed; 5-yr warranty — [St John NSW P3 data sheet (Jul 2026)](https://stjohnnsw.com.au/wp-content/uploads/2026/07/SJAA-P3-Pocket-AED-Data-Sheet_v2_FINAL.pdf)
- Note internal inconsistency in the data sheet: highlights say "4-year" pad life and "3-year battery", specs say pads 5-yr standby. Also "Child Mode Yes" but semi-automatic only.
- Launch / "Australia's smallest big thing", soft-drink-can size — [St John Ambulance Australia newsroom](https://stjohn.org.au/newsroom/st-john-pocket-aed/); [St John Vic](https://www.stjohnvic.com.au/news/new-pocket-aed-helps-bring-life-saving-support-closer-to-victorians/)
- UK "HEART Pocket", 700 g incl. battery and pads, phone-sized, EU MDR, IP65, 1.5 m drop, 8-year warranty, St John Ambulance sole UK distributor, targeted at "mobile responders in the emergency services", published 14 Apr 2026 — [Police Professional](https://policeprofessional.com/news/st-john-ambulance-launches-compact-defibrillator/)
- OEM manufacturer of the P3 is **not named** in either source (the "OBS-DE/W" pad model code hints at a Chinese OEM, but this is inference only).

**Schiller FRED easyport plus**
- 780 g; 46 x 150 x 143 mm (H x L x W); shocks "up to 200 J"; Multipulse Biowave; IP44; three versions (semi-automatic grey, automatic, manual red with ECG display); Wi-Fi + USB data transfer; child mode by button, "other electrodes are not required"; rechargeable or disposable battery, external charger takes two packs — [Schiller brochure 2.500781, Sept 2025 (German)](https://www.schiller.ch/schiller_images_pdfs/devices/monitoring_defi_cpr/fred-easyport-plus-und-trainer/2.500781_2025-09_de_fred_easyport_plus_pp_lq.pdf)
- The original (non-plus) FRED easyport is often quoted at ~490 g; **I could not verify this from a primary source** — treat as unverified.

**HeartSine 350P/360P/450P**
- 20 x 18.4 x 4.8 cm with Pad-Pak; adult 150/150/200 J, paeds 50/50/50 J; IP56 — [BHF bid spec sheet](https://www.bhf.org.uk/-/media/externally-referenced-files/ecommerce-files/defibrillators/heartsine-350p-360p-bid-specs-uk); [BHF listing](https://defibrillators.bhf.org.uk/heartsine-samaritan-pad-350p-semi-automatic-defibrillator)
- Weight 2.4 lb; 4-yr Pad-Pak life — [NFL "AEDs for Youth" comparison chart, Nov 2024](https://static.www.nfl.com/league/causes/smart-heart/AEDs-FOR-YOUTH-PROGRAM-%20AED-COMPARISON-CHART.pdf)

**Mainstream devices (weights, energies, battery life, connectivity, paeds approach)**
- From the NFL/Smart Heart Sports comparison chart (Nov 2024): weights Avive Connect 2.1 lb, Defibtech Lifeline 4.4 lb, Lifeline VIEW 3 lb, Philips OnSite 3.3 lb, FRx 3.5 lb, HeartSine 350P/360P/450P 2.4 lb, LIFEPAK CR2 4.5 lb, Zoll AED Plus 6.7 lb, Zoll AED 3 5.5 lb, Powerheart G5 5.7 lb. Energies: most 150 J adult / 50 J paeds; HeartSine 150/150/200; CR2 and Zoll 120/150/200 adult, Zoll 50/70/85 paeds; G5 95-354 J adult / 22-82 J paeds. Battery life: Avive rechargeable; Lifeline 5 or 7 yr; most others 4 yr; Zoll 5 yr. Connectivity: Avive Wi-Fi+cellular; Philips and HeartSine via attachment; CR2 Wi-Fi+cellular; Zoll AED 3 Wi-Fi. Child-mode button or key: Avive (button per chart), CR2, Zoll AED 3, Philips FRx (key); others need separate paeds pads — [NFL comparison chart PDF](https://static.www.nfl.com/league/causes/smart-heart/AEDs-FOR-YOUTH-PROGRAM-%20AED-COMPARISON-CHART.pdf)
- Note conflict: the NFL chart groups Avive with "child mode button" devices, but Avive's own spec page describes **separate pediatric pads with anterior-posterior placement**. Avive's primary page should win; the chart may be referring to the paeds pad cartridge switching modes.

**Other ultra-compact devices / startups**
- Japanese startup Onlinemaster Co. pocket AED ~ iPhone size/weight; animal testing at Toho University completed 2023; "on the cusp" of commercialisation — [JStories](https://jstories.media/pocket-aed/) (date of article and current status not verified)
- "Vivest Pocket" (Italy) described by a boating magazine as ~700 g one-hand semi-automatic AED with 2-year monitoring — [Barche a Motore](https://www.barcheamotore.com/it-takes-up-no-space-and-saves-your-life-the-pocket-defibrillator-to-have-in-your-boat/?lang=en) (secondary, unverified; may be a rebrand of the same OEM as the St John P3)
- HeartHero (US) is promoted by a UK retailer as the replacement for CellAED, availability "soon" — [Defib4Life blog, 12 Jan 2026](https://defib4life.co.uk/blogs/insights/cellaed-going-into-administration-what-does-it-mean-for-portable-aed-owners) (no specs gathered; promotional source)

### Inferences
- Energy-per-volume benchmark: St John P3 claims 130 x 150 J = 19.5 kJ of delivered therapy from an 18 Wh (64.8 kJ) LiMnO2 pack, i.e. ~30% pack-to-patient efficiency at 20 °C (manufacturer figures; real efficiency includes self-test drain over 3 yr). Avive: ≥75 x 150 J = 11.25 kJ from a rechargeable Li-ion of undisclosed capacity.
- The 0.7 L / 0.7 kg devices (P3, FRED easyport plus) are the realistic benchmark for a belt-worn full-energy AED; the 0.31 L CellAED shows what is possible only by cutting energy, shock count (20) and service life (single-use, 24 months).
- The main size drivers visible from specs: pad storage (P3 stores pads in a rear compartment; HeartSine/Avive use a cartridge that includes the pads), number of shocks (130 vs 75 vs 20), screen (Avive has a video screen; P3/CellAED use voice + LEDs), and IP/drop rating.

### Gaps
- Peak voltage, peak current, capacitor value: **not found for any device**.
- Prices: not found in primary sources in this pass (NFL chart lists "bundle pricing" without numbers).
- Philips OnSite/FRx, Zoll AED 3, Defibtech VIEW, CR2 dimensions, IP ratings, operating temps not gathered (beyond chart weights/energies) — available in each manufacturer's spec sheet.
- CellAED spec sheet (Cloudinary) returned HTTP 401; IP rating, operating temperature and waveform details not verified.

## Q2. Pediatric handling — which use a single pad set?

### Takeaway
Single universal pad set with a child button/key: St John P3 (child button), Schiller FRED easyport plus (child button, "other electrodes not required"), Zoll AED 3 (child button), Stryker CR2 (child button), Philips FRx (infant/child key). Separate paeds pads/cartridge: Avive (paeds pads, A-P placement), HeartSine (paeds Pad-Pak), Philips OnSite, Defibtech, Zoll G5. CellAED: pads for >10 kg, no paeds mode identified.

### Cited Findings
- St John P3: "Smart pads (suitable for both adults and children)"; "Child Button: Easily switch to Child Mode"; 50 J at 50 Ω — [St John P3 data sheet](https://stjohnnsw.com.au/wp-content/uploads/2026/07/SJAA-P3-Pocket-AED-Data-Sheet_v2_FINAL.pdf)
- FRED easyport plus: button changes patient mode, no other electrodes needed — [Schiller brochure](https://www.schiller.ch/schiller_images_pdfs/devices/monitoring_defi_cpr/fred-easyport-plus-und-trainer/2.500781_2025-09_de_fred_easyport_plus_pp_lq.pdf)
- Avive: pediatric pads, anterior-posterior placement; 50 J — [Avive spec page](https://www.avive.com/features/aed-specifications)
- Chart: child-mode button for Avive, CR2, Zoll AED 3; key for FRx; others need separate paeds pads (for children ≤55 lb / <8 yrs) — [NFL comparison chart](https://static.www.nfl.com/league/causes/smart-heart/AEDs-FOR-YOUTH-PROGRAM-%20AED-COMPARISON-CHART.pdf)
- CellAED pads for >10 kg patients — [CellAED manual (Ambulance Victoria)](https://www.ambulance.vic.gov.au/sites/default/files/2025-06/CellAED-Device-Manual.pdf) (snippet)

### Inferences
- For a belt-worn device, a single universal pad set + child button is the space-efficient choice (no second pad pack) and is already commercially accepted (Zoll, Stryker, Schiller, St John).

### Gaps
- How CellAED attenuates (or not) for children: not found.

## Q3. Internals of Avive and CellAED; company status; criticism

### Takeaway
No teardowns, FCC internal photos, or engineering talks were found for either in this pass. CellAED's size comes from a deliberately low-energy (~75-85 J) multi-capacitor biphasic design, single-use disposable architecture and 20-shock cap; it has drawn clinical/regulatory scrutiny (MHRA investigation 2024, FSN Nov 2024, UK withdrawal end-2024) and Rapid Response Revival reportedly entered voluntary administration at the end of 2025.

### Cited Findings
- MHRA (Oct 2024 parliamentary answer): "aware of issues raised by stakeholders regarding the CellAED device and we are investigating these concerns with the manufacturer" — [Parliamentary written question](https://streetguide.co.uk/parliament/written/1731273/5281) (search snippet)
- MHRA FOI 24-011 (2024) included the question whether MHRA can guarantee energy is sufficient after 6, 9 or 12 months — [MHRA FOI 24-011](https://assets.publishing.service.gov.uk/media/668d23d8ab5fc5929851bc14/FOI_24-011_final_redaction.pdf) (answer not read)
- MHRA Field Safety Notice 25 Nov 2024: "RRR Manufacturing Pty Ltd: CellAED", model 100-2.2-094, MHRA ref 34027855; issue not described on the listing page — [gov.uk FSN list 25 Nov 2024](https://www.gov.uk/drug-device-alerts/field-safety-notices-25-november-2024)
- Rapid Response Revival "has entered voluntary administration" (end of 2025); UK device withdrawn end of 2024 due to compliance issues; subscription replacement service will end — [Defib4Life blog, 12 Jan 2026](https://defib4life.co.uk/blogs/insights/cellaed-going-into-administration-what-does-it-mean-for-portable-aed-owners) (retailer with commercial bias; not confirmed by an official administrator notice — **unverified**)
- RCUK (Prof. Gavin Perkins) said RCUK does not endorse products and personal defibrillation "has the potential to increase access" — [RCUK response](https://www.resus.org.uk/about-us/news-and-events/professor-gavin-perkins-vice-president-resuscitation-council-uk-responds)
- Critic argues ILCOR/RCUK/ERC imply ≥150 J minimum for first shock and that the manual says "up to 150 J" — [adam-gent blog](https://adam-gent-bi3h.squarespace.com/cellaed) (page 404; snippet only). Note: guidelines actually defer to manufacturer-validated biphasic energies; the "minimum 150 J" claim is the critic's characterisation.
- April 2025 CellAED launched in France via reseller TaTonDef — [Australian Manufacturing](https://www.australianmanufacturing.com.au/australian-made-cellaed-launches-in-france-to-boost-cardiac-arrest-survival/)
- 2022 plan to raise US$37.5 M and to submit an FDA application in Q2 2022 — [Health Tech Digital](https://healthtechdigital.com/creators-of-the-worlds-first-personal-defibrillator-to-raise-usd37-5million/); no FDA approval found.
- Avive patents exist (e.g. EP3554630, EP4234004B1 "Automatic external defibrillator and method of use") — [Google Patents EP3554630A4](https://patents.google.com/patent/EP3554630A4/en); [Justia assignee list](https://patents.justia.com/assignee/avive-solutions-inc) (contents not reviewed)
- No Avive recall found in FDA results returned — [FDA search results summary](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfpma/pma.cfm?id=P210015) (absence of evidence, not confirmation)

### Inferences
- CellAED's lesson for an open design: reaching ~300 g required giving up energy headroom, shock count and serviceability, and the result struggled with regulators/market. A belt AED aimed at professionals should not undercut 150 J biphasic without its own clinical evidence.
- Avive's size comes mostly from removing the classic carry-case architecture and using a rechargeable Li-ion pack (no large primary pack), at the cost of charging discipline.

### Gaps
- FCC ID internal photos for Avive (it has cellular/Wi-Fi so an FCC ID should exist) — not searched due to budget; recommended next step via fccid.io.
- Patent claims content for Avive and CellAED multi-capacitor waveform — not reviewed.
- Peer-reviewed data on CellAED efficacy and FIRST trial results — not found.
- Official administrator notice (ASIC) for Rapid Response Revival — not found.

## Q4. Field problems and recalls of compact AEDs

### Takeaway
HeartSine (the long-time "smallest mainstream" AED) has a recall history centred on its combined battery-electrode cartridge and battery management: Class I in 2012 for 300/300P (spurious shutdown on voltage dip; 5 deaths not ruled out), and Class II actions 2023-2025 on Pad-Pak depleted cells and a component issue that may stop shocks on 350P/360P/450P. Recent Class II recalls also hit Zoll Powerheart G5 (Feb 2025) and Defibtech Lifeline VIEW AUTO (Mar 2025).

### Cited Findings
- HeartSine Samaritan 300/300P Class I recall: early battery-management software "may misinterpret a temporary drop in battery voltage as signaling a low battery, resulting in the device turning itself off"; initiated Sept 2012, terminated July 2015 — [FDA recall record](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfRes/res.cfm?id=112662); [Becker's](https://www.beckershospitalreview.com/quality/patient-safety-outcomes/fda-issues-class-i-recall-for-select-heartsine-public-access-defibrillators/)
- 2023 Pad-Pak recall Z-2143-2023: cartridges "may be rendered inoperable due to depleted battery cells" (process control) — [FDA recall record](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfRES/res.cfm?id=200366) (search summary; record ID mapping not individually verified); EU field safety notice — [Latvia ZVA FSN RA2023-3293631](https://zva.gov.lv/index.php/sites/default/files/inline-files/RA2023-3293631%20-%20FSN%20distributor.pdf)
- 2024 recall Z-1820-2024 covering 350P/360P/450P (Pad-Pak expiry instruction); 2025 California notice: component manufacturing issue may stop shocks, Class II — [CDPH HeartSine notice](https://www.cdph.ca.gov/Programs/CEH/DFDCS/CDPH%20Document%20Library/FDB/DeviceandDrugSafetyProgram/HeartSineDefibrillatorRecall.pdf); [Stryker notice FA318](https://www.stryker.com/us/en/emergency-care/product-notices/heartsine/fa318-t3.html)
- Zoll Powerheart G5 Class 2 recall initiated 13 Feb 2025; Lifeline VIEW AUTO Class 2 initiated 18 Mar 2025 — [FDA recall database results](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfRES/res.cfm?id=212406) (summary from search; individual record details not read)
- CellAED MHRA FSN Nov 2024 (see Q3).

### Inferences
- Failure modes that matter for a belt device: battery-voltage sag under charge load misread as end-of-life; primary-cell depletion in storage; combined battery+pad consumables that fail silently; component defects that block shock delivery. Self-test that loads the battery realistically, and conservative low-battery logic, are design requirements.

### Gaps
- MAUDE adverse-event analysis for Avive, HeartSine, CellAED not done.
- Details of the Zoll G5 and Lifeline VIEW 2025 recall causes not read.

## Q5. Police / first-responder carried AED programs

### Takeaway
Evidence supports police AEDs when police reliably arrive first (Miami-Dade, Rochester MN, Vienna), with negative results where police response was slow or overlapped with fast fire response. No study isolating belt-carry vs. car-mount was found, and no published size/weight requirement for officer-carried AEDs was found.

### Cited Findings
- Miami-Dade (2002): 1,900 AEDs carried by 1,900 officers; response fell from just under 8 min to under 5 min; pre-program survival 9% — [EurekAlert AHA release 2002](https://www.eurekalert.org/pub_releases/2002-08/aha-awa080702.php); [HealthDay](https://consumer.healthday.com/cardiovascular-health-information-20/heart-attack-news-357/defibrillators-and-police-a-life-saving-combination-508530.html)
- Rochester MN program started 1990 with 4 AEDs in 4 patrol cars; reported 58% survival from shockable SCA (program-level claim) — [Wikipedia: police vehicles with AEDs in North America](https://en.wikipedia.org/wiki/Police_vehicles_equipped_with_automated_external_defibrillators_in_North_America)
- Vienna: 105 patrol cars, 209 AEDs; time to first shock 6 min [IQR 2-10] with police AED vs 12 min [8-17], p<0.001 — [PMC7266310](https://pmc.ncbi.nlm.nih.gov/articles/PMC7266310)
- Negative: urban police AEDs added to fire department rapid defibrillation "not effective" — [Resuscitation 2005 via QxMD](https://read.qxmd.com/read/15964123/providing-automated-external-defibrillators-to-urban-police-officers-in-addition-to-a-fire-department-rapid-defibrillation-program-is-not-effective); Indiana suburban/rural no improvement — [PubMed 11282666](https://pubmed.ncbi.nlm.nih.gov/11282666)
- UK St John HEART Pocket marketed at "mobile responders in the emergency services" — [Police Professional, Apr 2026](https://policeprofessional.com/news/st-john-ambulance-launches-compact-defibrillator/)
- RCUK July 2026 briefing on AEDs in vehicles exists — [RCUK briefing PDF](https://www.resus.org.uk/sites/default/files/2026-07/RCUK%20briefing%20AEDs%20in%20vehicles%20_1.pdf) (not read)

### Inferences
- The benefit mechanism is minutes saved when the officer is first on scene; a belt unit adds value mainly when the officer is away from the car (foot patrol, buildings, crowds). Implied requirements from the device market: ≤0.7 kg, ≤~0.7 L, IP65+, ≥1.5 m drop, -15/-20 °C operation (car-cabin/outdoor), multi-year standby with weekly self-test, universal pads with child mode, and voice-only UI. These are inferred from current products, not from a published police specification.

### Gaps
- No published police procurement spec (weight, IP, drop, MIL-STD-810) for belt-carried AEDs found.
- Vehicle-cabin temperature exposure data and its effect on AED batteries/pads not gathered.
- RCUK 2026 vehicle briefing contents not read.
