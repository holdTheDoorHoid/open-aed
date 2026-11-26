# Open source and published defibrillator prior art (as of Nov 2026)

Scope note: research done 2026-11-26 with web search, page fetches, Google Patents pages and the GitHub API. About 30 tool calls. Each claim below has a source. When a claim comes from my general knowledge and was not checked, it is marked **[unverified]**. Status labels:
- **VERIFIED WORKING**: hardware delivered energy into a load or animal, with published measurements.
- **CLAIMED**: the authors say it works, but no measurement was found.
- **SIMULATION / CONCEPT**: schematics, simulations or a paper design only.

Headline finding: **no mature open-source AED exists.** There is one serious open design, the University of Pisa "OAED" (HardwareX 2017). Its high-voltage board was never experimentally validated, and its repository has been dormant since July 2017. Everything else found is a student paper design, a patent, or a closed commercial product.

---

## Q1. Open-source AED / defibrillator hardware projects

### Takeaway
There is exactly one substantive open AED hardware project: CentroEPiaggio's OAED, under GPL-3.0 per its GitHub repo. It covers KiCad hardware, PSoC firmware, a flyback charger, an IGBT H-bridge and an ADS1298 front end. The shock path was only simulated: about 200 J biphasic, charging in about 12 s. GitHub, GitLab-adjacent and Hackaday.io searches found no other open defibrillator hardware. The only other hits are student course papers (simulation) and closed commercial low-cost devices (Jeevtronics, India).

### Cited Findings
**CentroEPiaggio Open-Automated-External-Defibrillator (OAED), University of Pisa / UBORA (EU H2020)**
- Repository: https://github.com/CentroEPiaggio/Open-Automated-External-Defibrillator. License **GPL-3.0** (LICENSE file present). 84 stars, 23 forks. Folders: `oaed-hardware/`, `oaed-firmware/`, `Matlab scripts/`, plus `Thesis.pdf` and `Abstract.pdf` — [GitHub repo](https://github.com/CentroEPiaggio/Open-Automated-External-Defibrillator)
- Only **4 commits ever**, dated 2017-06-06 to **2017-07-12** (last commit: "Updated project description"). The 2026 "updated" timestamp on GitHub reflects metadata or star activity, not code. The open issues are spam or unanswered questions: #2 "Patient connection (Question)" from 2022, #3 "PCB FILE" from 2022, and #5 to #7, which are spam from 2025 — [GitHub API, commits and issues](https://github.com/CentroEPiaggio/Open-Automated-External-Defibrillator)
- Paper: "Open-source automated external defibrillator", *HardwareX*, Oct 2017. It describes two boards: a high-voltage board (HV-B) that delivers the shock, and a control board that detects cardiac arrest and drives the HV-B. Hardware is in KiCad, firmware in C, both under GPL. A single prototype cost about **400 EUR** — [HardwareX](https://www.hardware-x.com/article/S2468-0672(17)30035-4/fulltext); [DOAJ record](https://doaj.org/article/ce741e4142804f2fa4a53a514b2e4754)
- Architecture:
  - LPC1768 ARM MCU and an **ADS1298** biopotential front end — [ResearchGate](https://www.researchgate.net/publication/319709576_Open-source_Automated_External_Defibrillator)
  - Charger is a **flyback converter with a transformer designed in-house**
  - Output is an **H-bridge in which each switch is two series IGBTs**, chosen for low cost
  - **Three levels of isolation** between ECG sensing, charging and the H-bridge — [ResearchGate](https://www.researchgate.net/publication/319709576_Open-source_Automated_External_Defibrillator)
  - Note: the firmware folder is a Cypress PSoC Creator workspace (`Open AED.cywrk`, `.cydsn`) — [GitHub file](https://github.com/CentroEPiaggio/Open-Automated-External-Defibrillator/blob/master/oaed-firmware/Open%20AED.cywrk). This suggests a PSoC MCU in the shipped firmware, which conflicts with the "LPC1768" in the ResearchGate abstract. Unresolved.
- Maturity, from the README:
  - It says **"tests are needed for the experimental validation of HV-B"**
  - The 200 J biphasic shock in about 12 s comes **from simulations**
  - Rhythm detection showed >90% sensitivity and about 99% specificity, but only in preliminary tests on PhysioNet data
  - It began as a Master's thesis and is meant as a UBORA template and teaching tool — [README](https://github.com/CentroEPiaggio/Open-Automated-External-Defibrillator/blob/master/README.md)
  - **Status: SIMULATION / CONCEPT for the shock path.** The ECG and detection side was CLAIMED, tested on database signals only. No load test and no animal test was found.
- License conflict: the Open Source Toolkit directory lists it as **CC BY-SA 3.0**, "prototype", with "Assembly instructions published: No" and "Bill of materials published: No" — [open-source-toolkit](https://amchagas.github.io/open-source-toolkit/post/opensource_automated_external_defibrillator-_hardwarex). The repo LICENSE file says GPL-3.0.

**Student / course designs (simulation only)**
- UCSD BENG 186B (2021), Beckman, Bestmann, Gallardo, Gonzalez Mejia, Pendleton, "Bioinstrumentation Analysis: A Closer Look at the AED":
  - Circuit-level paper design: instrumentation amplifier, 159 Hz low-pass filter, a "black box" for the shock decision (pseudocode only), and a transformer and capacitor charging in 10 s
  - **Monophasic** output; a small DC test current estimates impedance, modelled as a 1 kΩ resistor
  - Virtual circuit model only. **SIMULATION / CONCEPT.** The paper itself says a biphasic design would be better — [UCSD PDF](https://isn.ucsd.edu/courses/beng186b/project/2021/Beckman_Bestmann_Gallardo_Mejia_Pendleton_Automated_external_defibrillator.pdf)
- "Cost-Effective Automated Defibrillator Design" student report on Scribd: split control board and HV board. Not peer reviewed, so low confidence — [Scribd](https://www.scribd.com/document/560527768/Report-1)
- A UIUC ECE445 course file came up in search results but was not opened — [UIUC](https://courses.physics.illinois.edu/ece445/getfile.asp?id=25176)

**Low-resource-setting devices (closed, commercial, not open source)**
- **Jeevtronics (Pune, India), SanMitra 1000 HCT**:
  - Dual-powered: mains plus a hand-crank generator, so no battery is needed; charges in about 12 s by hand-crank
  - About one quarter the price of imported units
  - Funded by India's DBT/BIRAC
  - The company holds four patents (India and US)
  - Reported 187 to over 200 units deployed in India and Africa; the figures conflict between reports
  - **Not open source** — [DBT India](https://dbtindia.gov.in/dbt-press/indian-start-makes-affordable-and-dual-powered-defibrillator-sanmitra-1000hct); [The Better India](https://www.thebetterindia.com/120607/made-in-india-defibrillator-pune/); [YourStory](https://yourstory.com/2021/06/pune-healthtech-startup-dual-powered-defibrillator/amp)

**Searches that came up empty**
- GitHub repo search (via `gh search repos`) for "defibrillator", "defibrillation", "AED defibrillator" and "shockable rhythm" found OAED as the only hardware project. Everything else was AED-location maps (e.g. openstreetmap-polska/openaedmap-frontend), games, software "defibrillator" libraries, and ML notebooks — [GitHub search](https://github.com/search?q=defibrillator&type=repositories)
- Hackaday.io has no DIY defibrillator build. Nearest: ICeeData, which sniffs telemetry from an implanted ICD and is not a defibrillator — [hackaday.io/project/11323](https://hackaday.io/project/11323-iceedata)
- Open ECG relatives on Hackaday: MobilECG went open source after a failed crowdfunding campaign — [Hackaday](https://hackaday.com/2014/02/23/mobilecg-goes-open-source)

### Inferences
- The OAED repo is the only reusable open hardware baseline. Its KiCad files, flyback and transformer design, IGBT H-bridge topology and ADS1298 front end are a starting reference, not a validated design. Its Thesis.pdf is probably the most detailed document and should be read in full.
- GPL-3.0 code and hardware means derivative firmware would have to be GPL. The CC BY-SA listing adds ambiguity. Contacting the authors (Centro E. Piaggio, Pisa) to confirm the license is worth doing before reusing files.
- Nobody has published an open design that was bench-tested into a 50 Ω load with measured energy, let alone animal-tested. A belt-worn open AED would be first on all counts.

### Gaps
- Not found: any OSHWA-certified defibrillator, any Open Source Medical Supplies defibrillator entry, any Latin American or African university open AED. Searches did not surface them; OSHWA and OSMS were not queried directly.
- The OAED thesis PDF was not read. Capacitor value, voltage and the H-bridge gate-drive details are therefore unconfirmed.
- The HardwareX full text returned HTTP 403, so measured ECG front-end figures could not be extracted.

---

## Q2. Academic papers with complete or near-complete designs

### Takeaway
Peer-reviewed, fully published AED circuit designs are essentially limited to the OAED HardwareX paper. The best public technical detail on charger, H-bridge and waveform design lives in **patents**, which describe the circuits in more detail than papers do. Several key waveform patents have now expired (see Q5).

### Cited Findings
- OAED, *HardwareX* 2017: see Q1. Flyback charger, double-IGBT H-bridge, three-level isolation, simulated 200 J biphasic — [ResearchGate](https://www.researchgate.net/publication/319709576_Open-source_Automated_External_Defibrillator)
- H-bridge circuit patents, all with full schematics, by Medtronic Physio-Control (Rockwell et al.):
  - US 5,824,017 ([USPTO PDF](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/5824017))
  - US 6,041,254, adds a protective component with both inductive and resistive properties ([USPTO PDF](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6041254))
  - US 6,175,765 ([USPTO PDF](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6175765))
  - US 6,963,773, uses SCR and IGBT switches in an integrated package. It notes that three SCR legs instead of two SCR plus two IGBT legs cut H-bridge size, weight and cost ([USPTO PDF](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6963773))
  - US 2004/0002736 A1, combines a biphasic waveform with external pacing ([Google Patents](https://patents.google.com/patent/US20040002736A1/en))
- "Simplified biphasic defibrillator circuit with make-only switching", US 8,145,300. The HV circuit needs only switch *closures* during biphasic delivery — [USPTO PDF](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/8145300)
- Zoll US 8,909,335, "Applying a rectilinear biphasic power waveform to a load" (Radzelovage, granted 2014-12-09). A boost converter and H-bridge deliver substantially constant current — [uspto.report](https://uspto.report/patent/grant/8909335)
- Rhythm-analysis literature (algorithm, not hardware):
  - Review of shock-advice algorithms — [PMC8976411](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8976411/)
  - Machine-learning shockable-rhythm detection, PLOS ONE 2016. It found VF detection harder on OHCA data than on public databases — [PMC4956226](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4956226/)
  - 2023 CNN shockable-rhythm classifier for a connected AED, AUC 0.995; test rhythms "available on request" via GitHub — [PMC10227259](https://pmc.ncbi.nlm.nih.gov/articles/PMC10227259)
  - VFPred: signal processing plus ML VF detection; notes VFDB is only about 9% VF — [arXiv 1807.02684](https://arxiv.org/pdf/1807.02684)
  - Analyzing rhythm during chest compressions (*J. Electrocardiology* 2014): compressions had to be interrupted in only 14% of cases; sensitivity 92%, specificity 99% — [PubMed 25172189](https://pubmed.ncbi.nlm.nih.gov/25172189/)
- Zoll's clinical trial of rectilinear biphasic vs damped-sine monophasic: first-shock efficacy 99% vs 93% (p=0.05), with about 60% less current — [Zoll](https://www.zoll.com/en-us/about/medical-technology/rectilinear-biphasic-technology); [Weill Cornell record](https://vivo.weill.cornell.edu/display/pubid10551711)

### Inferences
- For a belt-worn design, the patent literature is the practical "design textbook". The expired Heartstream and Zoll waveform patents (Q5) plus the expired-era H-bridge patents give fully disclosed topologies to follow.
- The SCR-based H-bridge idea (US 6,963,773) is relevant to size reduction. Its legal status was not checked.

### Gaps
- No peer-reviewed low-cost AED circuit paper from a developing-country group, with measured energy delivery, was found. Status: not found, not proven absent.
- The legal status of US 5,824,017, 6,041,254, 6,175,765, 6,963,773 and 8,145,300 was not individually checked. By filing dates, the 1990s ones are very likely expired **[unverified]**.

---

## Q3. Semiconductor vendor reference designs and app notes

### Takeaway
TI has at least one explicit AED HV capacitor-charger reference design, **PMP20899**: LM5022 flyback from 2S Li-ion (6 to 8 V) to **1500 V**. It also has an AED low-voltage rail design. Analog Devices has a well-known app note on defibrillation protection for ECG front ends. No complete vendor AED reference design (charger + H-bridge + ECG + MCU) was found.

### Cited Findings
- **TI PMP20899**: built for an AED customer. LM5022 low-cost flyback controller, 6 to 8 V 2-cell Li input, charges an HV capacitor to **1500 V** — [electronicsdatasheets mirror](https://www.electronicsdatasheets.com/manufacturers/texas-instruments/reference-designs/PMP20899) (the ti.com page was not opened directly)
- TI multi-output DC-DC for AEDs: 1.2 / 1.8 / 3.3 V rails using TPS71728 — [Arrow](https://www.arrow.com/en/reference-designs/12v-18v-33v-dc-to-dc-multi-output-power-supply-for-automatic-external-defibrillator/f0af4b688f2475390b4c816f436ae133bde48af288)
- TI PMP30162: SEPIC fast capacitor charger to 160/180 V. Not AED-specific — [electronicsdatasheets](https://www.electronicsdatasheets.com/manufacturers/texas-instruments/reference-designs/PMP30162)
- Analog Devices, "Lightning Bolts, Defibrillators, and Protection Circuitry Save Lives":
  - ECG front-end defib protection is a trade-off between protection and frequency response; the capacitance of the protection parts is critical
  - Typical stack: voltage limiters (GDT, MOV, TVS), current limiters (fuses, thermal cutouts) and rise-time reducers (resistors, inductors, ferrites)
  - An **AED can disconnect its own ECG input during the shock** because it knows when the shock is coming
  - Manufacturers recommend replacing input protection devices at least yearly because repeated shocks degrade them
  - [ADI article](https://www.analog.com/en/resources/technical-articles/lightning-bolts-defibrillators-and-protection-circuitry-save-lives.html)
- Arrow overview of defibrillator system blocks: HV discharge through IGBTs or fast MOSFETs, plus self-tests at startup and periodically in standby (battery, charger, electrodes, memory) — [Arrow article](https://www.arrow.com/en/research-and-events/articles/defibrillator-systems-a-life-saving-solution-for-cardiac-emergency-care)
- GE patent US 8,565,863 (ECG front end): protection resistors guard the front end against defibrillator pulses — [Google Patents](https://patents.google.com/patent/US8565863)

### Inferences
- PMP20899 maps almost directly onto a belt-worn design: 2S Li-ion to 1500 V flyback. It is the most reusable vendor block found. Its design files (schematic, BOM, test report) should be downloaded from ti.com.
- The ADS1298, used in OAED, is a TI part. TI's ADS129x datasheets and EVM documentation are the likely source of defib-protection input networks **[unverified; not fetched]**.

### Gaps
- No AED-specific reference design or block diagram was found from Microchip, Infineon, STMicro or Renesas; these vendors were not searched individually. TI's AED "end equipment" block-diagram page was not located; the search turned up only generic TIDU documents. Linear Technology (now ADI) photoflash/capacitor-charger parts (LT3750/LT3751 class) were not checked **[unverified that these suit AED duty]**.

---

## Q4. Teardowns and security research on commercial AEDs

### Takeaway
Public teardown detail is thin. EEVblog #909, a HeartSine Samaritan PAD, gives a part list that shows an SCR plus IGBT output stage, a MOSFET flyback primary and an ISD voice chip. No published full schematic of a commercial AED was found. Published security research concerns Zoll's *management software*, not device firmware.

### Cited Findings
- **EEVblog #909, HeartSine Samaritan PAD teardown**. Parts identified on the page: **CS20-22moF1, VS-30TPSPBF (Vishay), IRG4PH40UPBF, FQP45N15V2, ISD4004**. The page itself gives no architecture explanation; that is in the video and forum thread — [EEVblog](https://www.eevblog.com/2016/08/11/eevblog-909-heart-defibrillator-teardown/)
  - My reading of those part numbers **[unverified against datasheets in this session]**:
    - CS20-22moF1 is an IXYS 2200 V thyristor (SCR)
    - VS-30TPS is a Vishay 1600 V thyristor
    - IRG4PH40U is a 1200 V IGBT
    - FQP45N15V2 is a 150 V MOSFET, likely the flyback primary switch
    - ISD4004 is a voice record/playback chip for the spoken prompts
  - That points to an SCR/IGBT hybrid H-bridge, consistent with the Physio-Control SCR H-bridge patents above
- IEEE Spectrum articles on AED design and usability: "Idiot-Proofing the Defibrillator" and "The Shocking Truth About Defibrillators" — [Spectrum 1](https://spectrum.ieee.org/idiotproofing-the-defibrillator); [Spectrum 2](https://spectrum.ieee.org/biomedical/devices/the-shocking-truth-about-defibrillators)
- **Security: ZOLL Defibrillator Dashboard** (fleet management software, not device firmware):
  - Six vulnerabilities in versions before 2.2, one rated CVSS 9.9 (RCE)
  - They include CVE-2021-27481 (hard-coded crypto key), CVE-2021-27487 (cleartext credentials) and CVE-2021-27489 (unrestricted file upload)
  - Fixed in v2.2
  - [NVD CVE-2021-27481](https://nvd.nist.gov/vuln/detail/CVE-2021-27481); [NVD CVE-2021-27487](https://nvd.nist.gov/vuln/detail/CVE-2021-27487); [HIPAA Journal](https://www.hipaajournal.com/critical-rce-vulnerability-identified-in-zoll-defibrillator-dashboard/); [SC World](https://www.scworld.com/news/architecture/critical-flaws-in-defibrillator-management-tool-pose-account-takeover-credential-risk-for-hospitals)
- ZOLL advisories page: ZOLL assessed its products as not affected by the Nucleus RTOS vulnerabilities or Log4j — [ZOLL advisories](https://www.zoll.com/en-us/about/cybersecurity/advisories)
- Source quality warning: one search result (UAB *Inquiro*) claims AEDs step up to "almost 10,000 V". That is far above typical biphasic AED capacitor voltages, which are low thousands of volts at most; TI's AED charger design targets 1500 V. Treat that source as unreliable — [UAB Inquiro](https://www.uab.edu/inquiro/issues/past-issues/volume-11/an-introduction-to-electrical-systems-and-their-importance-to-cardiac-resuscitation-and-biomedical-innovation); compare [PMP20899](https://www.electronicsdatasheets.com/manufacturers/texas-instruments/reference-designs/PMP20899)

### Inferences
- An SCR-heavy output stage is cheaper and denser than all-IGBT. It fits a belt-worn goal, but SCRs cannot be turned off by gate, so truncating each phase needs a separate turn-off mechanism. This is an inference from general power electronics and should be confirmed against the Physio-Control patents.

### Gaps
- Not found: an iFixit AED teardown, an academic reverse-engineering paper on AED hardware, or any AED *firmware* security research (Philips HeartStart, Cardiac Science, Stryker/Physio-Control, CellAED, Avive). Searches returned nothing.

---

## Q5. Patents: expired vs. in force

### Takeaway
The core **Heartstream/Philips impedance-compensated BTE waveform family has expired**, around 2013–2016. So has **Zoll's original rectilinear-biphasic patent (US 6,096,063, expired Dec 2016)** and Zoll's early "rhythm ID in compression-corrupted ECG" patent (US 7,567,837, which lapsed for unpaid fees in 2021). Still in force:
- Philips' dual-algorithm analyze-during-CPR patent (US 10,561,853, to about 2036)
- Zoll's rectilinear power-waveform boost-converter patent (US 8,909,335, likely to about 2033) **[expiry unverified]**
- **Avive's 30+ US patents on its compact Connect AED**

Compact-AED design must clear the Avive portfolio in particular.

### Cited Findings
**Expired**
- **US 5,601,612**, "Method for applying a multiphasic waveform": Gliner, Lyster, Cole, Powers, Morgan; Heartstream, now Philips. Priority 1993-08-06. **Expired – Lifetime**; anticipated expiry 2013-08-06 — [Google Patents](https://patents.google.com/patent/US5601612A/en)
- The same family is all **Expired** per Google Patents — [Google Patents family list](https://patents.google.com/patent/US5601612A/en):
  - US 5,593,427
  - US 5,620,470 (fee related)
  - US 5,749,905
  - US 5,776,166
  - US 5,735,879
  - US 5,836,978
  - US 6,047,212, "External defibrillator capable of delivering patient impedance compensated biphasic waveforms"
- **US 6,096,063** (Zoll / ZMD; Lopin, Ayati), "Electrotherapy circuit having controlled current discharge based on patient-dependent electrical parameter". This is the rectilinear-first-phase resistor network that is shorted out step by step. Priority 1996-12-18. **Expired – Lifetime**, 2016-12-18 — [Google Patents](https://patents.google.com/patent/US6096063)
- **US 7,567,837** (Institute of Critical Care Medicine, now Zoll), "Enhanced rhythm identification in compression corrupted ECG". Priority 2003-12-19. **Expired – Fee Related**; lapsed 2021-08-30 for non-payment — [Google Patents](https://patents.google.com/patent/US7567837). Its parent is US 7,039,457 — [Google Patents](https://patents.google.com/patent/US7039457)

**Still in force, or likely**
- **US 10,561,853** (Philips), "AED with dual ECG analysis algorithms": one algorithm tolerant of CPR noise, one for hands-off periods, with continuous and scheduled CPR modes. Priority 2014-12-12. **Active**, anticipated expiry **2036-03-06** — [Google Patents](https://patents.google.com/patent/US10561853)
- **US 8,909,335** (Zoll, Radzelovage), rectilinear biphasic power waveform via boost converter and H-bridge, granted 2014-12-09 — [uspto.report](https://uspto.report/patent/grant/8909335). The related EP 2885053 A1 has a 2012-08-20 priority — [EPO](https://data.epo.org/publication-server/rest/v1.2/patents/EP2885053NWA1/document.html). Expiry is about 2033 by the 20-year rule **[unverified; status not checked]**
- Other analyze-during-CPR patents of unchecked status:
  - US 9,409,034, "analyzing cardiac rhythm during CPR" ([USPTO](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9409034))
  - US 10,335,604 and US 11,273,314, "Apparatus for monitoring a cardiac rhythm during CPR" ([USPTO 10335604](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10335604), [USPTO 11273314](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11273314))
  - US 8,509,881, adaptive piecewise stitching ([USPTO](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/8509881))
  - US 9,604,069, limb leads for artifact-free ECG ([USPTO](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9604069))
  - US 10,960,222 and US 12,138,468, "Fast identification of shockable or non-shockable rhythms" ([USPTO 10960222](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10960222), [USPTO 12138468](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/12138468))
  - US 9,308,383, defibrillation delivery decision ([USPTO](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9308383))
- **Avive Connect AED**, per Avive's virtual patent marking page:
  - **US** 10,029,109; 10,071,256; 10,112,054; 10,449,380; 10,543,376; 10,565,845; 10,621,846; 10,737,105; 10,773,091; 10,903,675; 11,077,311; 11,089,989; 11,097,121; 11,266,826; 11,305,128; 11,318,322; 11,331,471; 11,452,881; 11,607,555; 11,640,755; 11,645,899; 11,691,021; 11,839,770; 11,908,299; 12,020,548; 12,170,010; 12,194,305; 12,251,569; 12,290,696; 12,381,000; designs D893,032 and D1,028,245
  - **JP** 7257681, 7281825, 7610867
  - **EP** 3811379
  - The pad cartridge is covered specifically by US 11,266,826, 11,331,471 and 12,194,305
  - Avive markets the device as the smallest, lightest FDA-approved AED, at 2.1 lb
  - [Avive patents page](https://www.avive.com/patents); [Avive FDA approval news](https://www.avive.life/news/avive-solutions-receives-fda-approval-of-handheld-automated-external-defibrillator-aed-with-unique-capabilities)
- **CellAED (Rapid Response Revival / RRR Manufacturing / CellAED Life Saver Pty Ltd)**: the spec sheet calls it a "patented" AED with integrated hydrogel pads but gives no number — [CellAED spec sheet](https://res.cloudinary.com/hwos9pjeb/raw/upload/v1721718868/media/DOC-2104v4-BCH_Customer_Tech_Spec_Sheet-FA.pdf). A related compact-AED patent family (Shaker et al.), "compact AED with one distal electrode", appeared in search: US 11,547,863 B1, continuing from 11,433,249. Its link to CellAED is **unconfirmed** — [US11547863 PDF](https://patentimages.storage.googleapis.com/9f/a5/ec/c60fdd06b65528/US11547863.pdf); [US11433249 PDF](https://patentimages.storage.googleapis.com/07/64/69/4dc55a568c098e/US11433249.pdf)
- Other waveform patents cited by US 5,601,612, with statuses not checked:
  - US 6,208,898 and US 6,241,751 (Agilent, impedance-compensated energy delivery)
  - US 5,944,742 and US 6,411,846 (Survivalink)
  - US 6,539,255 (Cardiac Science)
  - US 6,539,258 and US 6,650,936 (Physio-Control)
  - [Google Patents](https://patents.google.com/patent/US5601612A/en)

### Inferences
- The basic impedance-compensated BTE waveform, as Heartstream/Philips used it, is free to use. Most pre-2006-priority waveform and H-bridge patents should now be expired, but each needs checking.
- The main patent risk for a *belt-worn compact* AED is the Avive and CellAED-type portfolios on form factor, pad cartridges and connectivity, and analyze-during-CPR features (Philips '853 to 2036). A freedom-to-operate review by a patent attorney is needed before any distribution. Google's statuses are assumptions, not legal conclusions.

### Gaps
- Not found: Physio-Control ADAPTIV waveform patent numbers. They were not searched separately.
- Not found: CellAED's own patent numbers. The next step is a PATENTSCOPE search on the applicants "Rapid Response Revival", "RRR Manufacturing" and "CellAED Life Saver".
- The claims of the Avive patents were not reviewed.

---

## Q6. Open rhythm-analysis software and ECG databases

### Takeaway
There is **no mature open-source shock-advisory algorithm**. Only small research repos exist: Joey5460/rhythm_analysis (no license, last push 2018), plus ML notebooks. The open validation data is PhysioNet CUDB (ODC-By) and VFDB. The **AHA database is not free**. Real OHCA defibrillator recordings are scarce in open form; the nearest is a 260-ECG Heartstart dataset on Mendeley.

### Cited Findings
**Code**
- **Joey5460/rhythm_analysis**: "Detecting VT and VF for AED". **No license**, last push 2018-05-27, 1 star; built with SWIG and Python — [GitHub](https://github.com/Joey5460/rhythm_analysis); metadata from the GitHub API
- haibaokesheng/CNN-LSTM-Network: "Shockable Rhythm Diagnosis Based on Sequential Pooling CNN-LSTM Network". No license, last updated 2022 — [GitHub](https://github.com/haibaokesheng/CNN-LSTM-Network)
- niallconeill/TFM-ECG: ML ensemble shockable-rhythm study, 2019, no license — [GitHub](https://github.com/niallconeill/TFM-ECG)
- OAED's `Matlab scripts/` and firmware include a detection algorithm, GPL-3.0, with CLAIMED >90% sensitivity and about 99% specificity on PhysioNet — [OAED README](https://github.com/CentroEPiaggio/Open-Automated-External-Defibrillator/blob/master/README.md)

**Databases**
- **CUDB**, Creighton University Ventricular Tachyarrhythmia Database: 35 eight-minute recordings with sustained VT, flutter or VF. **Open Data Commons Attribution License v1.0**. The authors say its annotations are "in no sense definitive" — [PhysioNet CUDB](https://physionet.org/content/cudb/1.0.0/)
- **VFDB**, MIT-BIH Malignant Ventricular Ectopy Database: 22 recordings of 30 min, with rhythm labels only. Highly imbalanced, about 9% VF — [PhysioNet VFDB](https://physionet.org/physiobank/database/vfdb/); [VFPred](https://arxiv.org/pdf/1807.02684)
- **I-CARE** (PhysioNet, v2.1, Dec 2023): 607 post-arrest comatose patients with OHCA and shockable-initial-rhythm flags. The ECG is post-resuscitation monitoring, *not* defibrillator capture — [PhysioNet I-CARE](https://physionet.org/content/i-care/2.1/)
- **"Cardially"** dataset (Mendeley; *Data in Brief* 2021): 260 prehospital ECGs, each a 9 s VF segment plus 1 min post-shock, labelled shock success/failure. Recorded on a Laerdal Heartstart 3000. License not confirmed — [Mendeley](https://data.mendeley.com/datasets/wpr5nzyn2z/1); [DOAJ](https://doaj.org/article/6924a34405494f24ab4f24e3bfe9cdaf)
- Validation caveat: VF detection is harder on OHCA data than on public databases, so CUDB and VFDB results overstate field performance — [PLOS ONE 2016, PMC4956226](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4956226/)

### Inferences
- An open AED project would have to write its own shock-advisory algorithm and validate it on CUDB, VFDB and MIT-BIH (open), then seek licensed OHCA data. The AHA database (ECRI, paid) is the usual one for AAMI/IEC 60601-2-4 style performance reporting **[unverified in this session]**.
- The shortage of open OHCA defibrillator recordings is probably the biggest data bottleneck. Partnering with an EMS research group (the Aramendi / Irusta group at UPV/EHU publishes heavily on this **[unverified]**) may be necessary.

### Gaps
- Not confirmed this session: the MIT-BIH Arrhythmia DB license and the Sudden Cardiac Death Holter DB on PhysioNet (both usually open), and the AHA DB's price and terms. These were not fetched.
- PhysioNet's VFDB license page was not opened.

---

## Q7. Open-source test equipment

### Takeaway
Open ECG *simulators* exist: an OSHWA-certified patient simulator, an Arduino 3-lead simulator, and a TU Delft low-resource ECG simulator. Only one "low-cost, open-source defibrillation analyzer" paper was found (2021, educational), and its files could not be confirmed. A real energy-measuring defibrillator analyzer with a 50 Ω load would effectively have to be built by the project.

### Cited Findings
- **OSHWA-certified "Johan Smets ECG Patient Simulator"** (cert. BE000003), which tests ECG cable accuracy and performance — [OSHWA](https://certification.oshwa.org/exemplar/be000003.html)
- TU Delft thesis: "Design and prototyping of an ECG simulator for medical device testing in low-resource settings" — [TU Delft repository](https://repository.tudelft.nl/record/uuid:0a0494cc-bf6d-4979-b884-4812aec8f5a0)
- *J. Med. Eng. Technol.* 2019: MATLAB-generated ECG through a sound card plus a custom output board. MATLAB source and layout are in the supplement — [informahealthcare](https://informahealthcare.com/doi/abs/10.1080/03091902.2019.1576788)
- An Arduino UNO-based ECG simulator for testing 3-lead ECG meters came up in search results; it is one of the theses listed in [UCLouvain](https://thesis.dial.uclouvain.be/bitstreams/343d9be8-3025-431e-9d34-2c1779875530/download) / [IPL](https://repositorio.ipl.pt/bitstreams/cade0fea-6f32-4814-a515-ec56b0db175d/download) (which one is uncertain)
- "Low-cost, open source defibrillation analyzer for educational purposes", 2021 IEEE conference paper. The abstract page did not render, so design-file availability is unknown — [AMiner](https://aminer.org/pub/61a8a3c15244ab9dcb6c3768/low-cost-open-source-defibrillation-analyzer-for-educational-purposes)
- TU Graz thesis (Saely): a test system that feeds ECG signals into an AED and reads back its shock decisions. That is a rhythm-analysis test rig; open status unknown — [TU Graz PDF](https://itp.tugraz.at/~karl/VF/DA_HannesSaely.pdf)
- Zoll X Series patient-simulator *app* for paramedic training. Software only, no license — [GitHub](https://github.com/snakeku/Patient-Simulator-with-Zoll-X-Defibrillator)

### Inferences
- The TU Graz-style "play a database ECG into the device and log decisions" rig is cheap to rebuild with a DAC. It would let the project replay CUDB and VFDB through the real analog front end, not just test in software.
- An energy analyzer is a 50 Ω (plus 25 to 175 Ω range) high-power, high-voltage load with a divider and a current shunt feeding a fast ADC. No open, validated design was found, so the project would need to build and calibrate one against a commercial analyzer (Fluke Impulse class) **[unverified]**.

### Gaps
- Authors, venue and file availability of the 2021 open defib-analyzer paper: not retrieved.

---

## Q8. Lessons from other open-source life-critical medical hardware

### Takeaway
Open ventilators (2020) mostly failed to reach patients. The causes were about 100 fragmented teams, "open-when-finished" secrecy, incomplete documentation, and no manufacturer to carry designs through regulation. Simpler open PPE succeeded at scale. OpenAPS and Loop succeeded through a published safety **reference design**, layered on certified hardware, plus an active user community. Loop eventually reached an FDA submission via Tidepool using observational data.

### Cited Findings
- Open ventilators:
  - "Open-when-finished" models led to a proliferation of over a hundred incompatible designs and duplicated effort
  - "A functioning design is necessary but not sufficient": it must be taken through regulatory approval by a manufacturer that understands the design decisions
  - [PMC7195895 (review of open-source ventilators)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7195895/)
- The same review found that tested, peer-reviewed systems lacked complete documentation, and the documented ones were at early stages — [PMC7195895](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7195895/)
- FSF talk "The Story and The Learnings of Open Source Ventilators":
  - About 100 teams worked largely in isolation
  - Its lessons: be part of the solution not the whole solution; stay modular; separate development from manufacturing for licensing and liability; respect FDA processes, because there are no shortcuts
  - Open PPE delivered more than 5 million pieces while ventilator projects mostly had no impact
  - [FSF PDF](https://gmg0p.fsf.org/mgoblin_media/media_entries/2495/The_Story_and_The_Learnings_of_Open_Source_Ventilators_2_copy.pdf)
- Further reading: Fraunhofer OPEN.Effect study — [Fraunhofer](https://publica-rest.fraunhofer.de/server/api/core/bitstreams/606f8b4d-85f4-4036-868e-227ea3464a34/content); IEEE Spectrum "The Great Ventilator Rush" — [Spectrum](https://spectrum.ieee.org/geek-life/hands-on/the-great-ventilator-rush)
- OpenAPS:
  - Open-sourced in late 2014; most DIY systems follow the 2015 **OpenAPS Reference Design** safety principles
  - OpenAPS documentation says its safety foundation rests on the safety features of the underlying (certified) hardware
  - [PMC6610599](https://pmc.ncbi.nlm.nih.gov/articles/PMC6610599); [OpenAPS docs](https://openaps.readthedocs.io/)
- Outcomes: early self-reported HbA1c fell from 7.1% to 6.2%, and time in range rose from 58% to 81% — [PMC5094342](https://pmc.ncbi.nlm.nih.gov/articles/PMC5094342)
- A fault-detection and redundancy paper compares DIY AID with the Boeing 737 MAX and calls the human-in-the-loop the most important "component" — [search result set, PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC7753866)
- DIY loops remain off-label and not FDA-approved. Tidepool's FDA submission for Loop relied on Loop Observational Study data — [Healio 2021](https://cm.healio.com/news/endocrinology/20210521/loop-artificial-pancreas-software-safe-effective-with-community-support); [Healio 2018](https://cm.healio.com/news/endocrinology/20181116/as-diy-loopers-reinvent-artificial-pancreas-endocrinologists-confront-safety-liability-concerns)
- Open ECG: MobilECG went open source after its crowdfunding failed — [Hackaday](https://hackaday.com/2014/02/23/mobilecg-goes-open-source). Hackaday also covered open-source electrocardiography saving lives — [Hackaday 2016](https://hackaday.com/2016/04/29/saving-lives-with-open-source-electrocardiography)

### Inferences
Lessons for an open AED:
1. Develop in the open from the start, not "open when finished".
2. Publish a safety reference design (hazard analysis, IEC 60601-2-4 mapping), as OpenAPS did.
3. Stay modular: charger, H-bridge, front end and algorithm, each separately testable.
4. Separate the open design from any regulated manufacturer early.
5. Ship test equipment and validation data alongside the device.

Unlike OpenAPS, an AED cannot lean on already-certified hardware for its safety. That makes the reference-design and test-rig work more important, not less.

### Gaps
- Tidepool Loop's FDA clearance (I believe January 2023) was not verified this session **[unverified]**.
- Glia (stethoscope, tourniquet) was not researched, so its lessons are missing.
- No source compared the success factors of open medical hardware projects systematically.
