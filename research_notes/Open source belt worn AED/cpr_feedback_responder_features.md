# Smart AED features beyond the shock decision, and human factors for trained responders (gap-filling pass)

Scope note: this pass leaves out shock-advisory algorithms, deep learning for shock decisions, analysis during compressions, AMSA, ECG datasets, pediatric rhythm analysis and software recalls, because an earlier researcher covered them.

Verification convention: **[V]** means I checked it this session against a search result, abstract snippet or primary document. **[BG]** means background knowledge I did not verify this session. Treat [BG] items as leads to check, not as citations. Evidence grades: RCT, cluster-RCT, meta-analysis (MA), observational (OBS), manikin/bench (BENCH), manufacturer claim (MFR).

Search budget: about 25 searches plus a few fetches. PMC and PubMed pages blocked automated fetch, so many numbers below come from abstracts and search snippets, not full texts. The ILCOR 2025 BLS CoSTR PDF was read in full text.

---

## 1. Real-time CPR feedback: what it does for CPR quality and survival, guideline position, sensing methods

### Takeaway
Real-time feedback reliably improves CPR process measures (rate, depth, compression fraction), but RCT evidence for survival is mixed. The large prehospital cluster-RCT (Hostler 2011) found no survival effect. One large in-hospital RCT (Goharani 2019) found a big effect, and meta-analyses disagree. ILCOR's current position (2020 wording, kept in the 2025 CoSTR) is a **weak recommendation with very low certainty**, and only as part of a quality-improvement programme. Using the device in isolation is recommended *against*. For a belt-worn AED the low-cost win is impedance-derived compression rate and pause tracking, which needs no extra sensor. Measuring depth needs a puck (accelerometer, force sensor or field-induction), and depth on soft surfaces needs a second reference sensor.

### Cited findings

**Trials and meta-analyses**
- **Hostler D et al., BMJ 2011;342:d512 (cluster-RCT, Resuscitation Outcomes Consortium, 3 sites in the US and Canada) [V]:** 1,586 OHCA, 815 with feedback and 771 without. Compression fraction went from 64% to 66% (cluster-adjusted difference 1.9, 95% CI 0.4 to 3.4). Prehospital ROSC was 44% vs 45% (NS). Survival to discharge was 11% vs 12% (adjusted difference -1.5, 95% CI -3.9 to 0.9), and awake at discharge was 10% in both arms. **EMS muted the audible feedback in 14% of feedback cases.** — [PubMed 21296838](https://pubmed.ncbi.nlm.nih.gov/21296838/); [BMJ](https://www.bmj.com/lookup/doi/10.1136/bmj.c7108); [PMC3033623](https://pmc.ncbi.nlm.nih.gov/articles/PMC3033623)
- **Goharani R et al., J Intensive Care 2019 (RCT, NCT02845011; 8 academic hospitals, ICU IHCA; hand-held Cardio First Angel device) [V]:** 1,454 randomised and 900 analysed. Sustained ROSC was 66.7% vs 42.4%, survival to ICU discharge 59.8% vs 33.6%, and survival to hospital discharge 54% vs 28.4% (p<0.001). The benefit held in non-intubated patients but **not in intubated patients**. ROSC was the primary outcome, and survival was secondary. Caveat: the effect size is unusually large for a CPR-process intervention, and it is a single trial. — [J Intensive Care PDF, doi 10.1186/s40560-019-0357-5](https://jintensivecare.biomedcentral.com/track/pdf/10.1186/s40560-019-0357-5); [PubMed 30693086](https://pubmed.ncbi.nlm.nih.gov/30693086)
- **Bobrow BJ et al., Ann Emerg Med 2013;62(1):47-56 (before-after OBS, Mesa Fire & Medical, Arizona; ZOLL feedback) [V]:** Phase 1 ran 18 months with feedback disabled. Phase 2 ran 16 months after scenario training of 373 rescuers, with feedback on. Survival in witnessed shockable arrests went from 26.3% (15/57) to 55.6% (20/36), a difference of 29.2% (95% CI 9.4 to 49.1). The authors flag that the design is vulnerable to temporal confounding, and training and feedback are bundled together. — [PubMed 23465553](https://pubmed.ncbi.nlm.nih.gov/23465553/); [ADHS PDF](https://www.azdhs.gov/documents/preparedness/emergency-medical-services-trauma-system/save-hearts-az-registry-education/published-papers/research-article-scenario-based-training-cardiac-arrest.pdf)
- **MA, 2020 (PMID 32755666) [V, abstract-level]:** literature to April 2020, 11 studies (3 RCT, 8 non-randomised), n=4,851. It did **not** show a survival-to-discharge benefit, and heterogeneity was high. In a subgroup, **portable stand-alone devices did better than AED-integrated feedback**. — [PubMed 32755666](https://pubmed.ncbi.nlm.nih.gov/32755666/)
- **MA of RCTs of free-standing audiovisual devices in IHCA (PROSPERO CRD42020157536) [V, snippet only]:** it concluded that moderate-quality evidence suggests improved sustained ROSC and survival to ICU and hospital discharge. That result is driven heavily by Goharani 2019. I could not retrieve the pooled numbers. — [PMC7771623](https://pmc.ncbi.nlm.nih.gov/articles/PMC7771623)
- **MA, 2022 (PMC9478281) [V, snippet]:** real-time feedback did not improve ROSC or survival to discharge, and the authors call the effect "controversial". — [PMC9478281](https://pmc.ncbi.nlm.nih.gov/articles/PMC9478281)
- **MA, 2024 J Clin Med 13(19):5989 [V, snippet]:** audiovisual feedback devices significantly increased ROSC and survival, but most included studies were non-randomised. A separate review pooled 13 studies and 17,600 patients, mostly non-randomised. — [MDPI](https://www.mdpi.com/2077-0383/13/19/5989); [PMC11477487](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11477487/); [PMC10600786](https://pmc.ncbi.nlm.nih.gov/articles/PMC10600786)
- **Kramer-Johansen J et al., JAMA 2006;295:2620-8 (before-after OBS, 3 European EMS, Philips/Laerdal Q-CPR) [BG]:** feedback improved compression depth and rate and reduced hands-off time. ROSC and survival did not differ significantly. Not re-verified this session.

**Guideline position**
- **ILCOR 2025 BLS CoSTR, BLS 2511 "Feedback for CPR quality" (ScopRev 2024, literature to 18 July 2023) [V, full text]:** it keeps the 2020 treatment recommendations. "We suggest the use of real-time audiovisual feedback and prompt devices during CPR in clinical practice as part of a comprehensive quality improvement program … (weak recommendation, very low–certainty evidence)" and "We suggest against the use of real-time audiovisual feedback and prompt devices in isolation (ie, not part of a comprehensive quality improvement program) (weak recommendation, very low–certainty evidence)." The population is health professionals responding in a professional capacity. That matches this project's users. — [ILCOR BLS 2025 CoSTR full chapter, pp. 46-47](https://www.ilcor.org/uploads/BLS-2025-COSTR-Full-Chapter.pdf)
- **ILCOR 2025, BLS 2402 real-time ventilation feedback (ScopRev 2025) [V, full text]:** the evidence is insufficient to show clinical efficacy, and industry was involved in 7 included studies. Three clinical studies were found:
  - Lee 2023 RCT (Korea, n=63 vs 58): ROSC 55.5% vs 36.2% (p=0.04); survival to discharge 4.9% vs 8.6% (p=0.54).
  - Drennan 2024 before-after (Canada, n=221 vs 191): ROSC 27% vs 29% (NS), with rate and volume in target 19% vs 7%.
  - Abella 2007 IHCA cohort (impedance-based feedback): ROSC 44.6% vs 40.0%, survival 8.9% vs 9.1%.

  ILCOR also advises saying "inspiratory volume" rather than "tidal volume" for mask or airway flow measurements. — [ILCOR BLS 2025 CoSTR, pp. 48-50](https://www.ilcor.org/uploads/BLS-2025-COSTR-Full-Chapter.pdf)
- **ILCOR 2025 EIT 6404 (training) [V]:** CPR feedback devices *during training* get a **strong recommendation, moderate certainty**, for healthcare and lay providers. — [ILCOR CoSTR EIT 6404](https://costr.ilcor.org/document/cpr-feedback-device-used-in-resuscitation-training-eit-6404-tf-sr); [ILCOR Executive Summary 2025](https://www.ilcor.org/uploads/Executive-Summary-2025-COSTR.pdf)
- **AHA 2025 Part 12 (education) [V]:** recommends feedback devices during skill training for lay and professional rescuers. I could not verify the AHA 2025 adult BLS class for *clinical* real-time feedback. The AHA 2020 text had it as Class 2b, "may be reasonable" [BG]. — [AHA Part 12](https://cpr.heart.org/en/resuscitation-science/cpr-and-ecc-guidelines/resuscitation-education-science); [PubMed 41122895](https://pubmed.ncbi.nlm.nih.gov/41122895/)
- **ERC 2025 [V, partial]:** the education guideline says to use real-time CPR feedback devices to improve skill acquisition. I did not retrieve the exact clinical BLS wording. The citation is Resuscitation 2025;215 Suppl 1:110771. — [ERC 2025 Education public-comment draft](https://cprguidelines.eu/assets/other-documents/ERC-Guidelines-2025-Education-for-Resuscitation-final-for-public-comments.pdf); [ERC 2025 Executive Summary, PubMed 41117573](https://pubmed.ncbi.nlm.nih.gov/41117573/)
- **Training-only RCT review (2025) [V, snippet]:** no studies measured patient survival. — [Stirling repository](https://www.stir.ac.uk/research/hub/publication/2135800)

**Sensing methods and accuracy**
- **Single accelerometer on a soft surface overestimates depth (BENCH) [V]:**
  - On a mattress with a true depth of 4.4 cm, the CPRmeter (accelerometer) read 4.7±0.1 cm, while a pressure-sensor device read 4.4 cm.
  - An earlier manikin study found that foam-mattress compression made up about 30-40% of total measured depth. — [PubMed 26979259](https://pubmed.ncbi.nlm.nih.gov/26979259); [PMC3565146](https://pmc.ncbi.nlm.nih.gov/articles/PMC3565146)
- **Dual accelerometer, one on the chest and one on the back or mattress, with spectral analysis (BENCH, 2016) [V]:** median depth error 2.1 mm (4.4%), with mattress displacement compensated. — [PMC5143701](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5143701/)
- **Dual-sensor feedback patent family [V, existence only]:** — [USPTO 12697273](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/12697273)
- **Compression rate from transthoracic impedance (TTI) via the defib pads, no extra sensor (OBS signal study) [V]:** Ruiz de Gauna et al., PLOS ONE 2020 reported an unsigned rate error of 1.7 compressions/min and judged it accurate enough for real-time AED rate guidance. — [doi 10.1371/journal.pone.0239950](https://dx.doi.org/10.1371/journal.pone.0239950); [PMC7537815](https://pmc.ncbi.nlm.nih.gov/articles/PMC7537815)
- **Ventilation detection from TTI [V]:**
  - Compressions contaminate the impedance signal. One 2023 method removes compression artefact using force and acceleration as references and adds a signal-quality stage.
  - On a Philips MRx during mechanical CPR, TTI was unusable for ventilation detection in about 4.4% of minutes.
  - González-Otero, Aramendi et al. (CinC 2013) used 18 OHCA episodes. — [PMC10336723](https://pmc.ncbi.nlm.nih.gov/articles/PMC10336723); [CinC 2013 0807](https://www.cinc.org/archives/2013/pdf/0807.pdf); [core.ac.uk 322437357](https://core.ac.uk/download/322437357.pdf)
- **Commercial implementations [BG]:**
  - Philips Q-CPR: accelerometer plus force sensor plus impedance.
  - ZOLL Real CPR Help: accelerometer in the CPR-D-padz puck.
  - Physio-Control/Stryker TrueCPR: triaxial magnetic field induction between a chest pad and a back pad, which measures sternum-to-back distance directly. The manufacturer claims it stays accurate on soft surfaces (MFR claim).
  - Smartphone and smartwatch IMU apps exist, but I did not verify their accuracy this session.

### Inferences
- The survival evidence is too weak and too contradictory to justify much size or power on feedback *for survival alone*. The ILCOR wording ties benefit to a QI programme. **Recording and exporting CPR metrics for debriefing** is therefore part of the "feedback" value, not an optional extra.
- Impedance-only rate and compression-fraction (CCF) tracking costs nearly nothing in hardware, since impedance is already measured for shock. That makes it the best first tier.
- Depth needs extra hardware:
  - An accelerometer puck is cheap but wrong on beds and stretchers by up to about 30-40% of depth.
  - Dual accelerometers or field-induction fix this but need a back sensor, which conflicts with a compact belt-worn form.
- The Hostler finding that crews muted 14% of audible feedback argues for a **quiet visual or haptic channel and mute control** for professional users, rather than constant voice correction.

### Gaps
- I could not open the full texts of the pooled MAs (PMC blocked), so pooled RR/OR values are missing.
- AHA 2025 adult BLS class for clinical real-time feedback: not verified.
- Recent independent bench accuracy data for TrueCPR and smartwatch/IMU depth estimation: not found this session.
- I found no validated *depth* estimation from impedance alone. Only rate and pauses are validated.

---

## 2. Detecting pulse or return of circulation without a manual pulse check

### Takeaway
Research classifiers using pad ECG plus thoracic impedance reach about 92-94% balanced accuracy on retrospective OHCA data. Deep-learning ECG-only methods reach about 93.5%. ETCO2 is highly specific but insensitive. Wrist and forehead PPG detects ROSC in small studies. **I found no commercial AED that does automated pulse/ROSC detection.** It remains research-grade. Treat it as decision support ("possible ROSC, check") with no automatic action.

### Cited findings
- **Elola A et al., Entropy 2019 (OBS retrospective) [V]:** 3,914 five-second ECG segments from 279 OHCA episodes. A fully convolutional net and a CNN+RNN both reached about 93.5% balanced accuracy, more than 1.5 points better than earlier methods. Only the pad ECG was used. — [EHU repository](https://addi.ehu.eus/handle/10810/64737); [PMC7514786](https://pmc.ncbi.nlm.nih.gov/articles/PMC7514786)
- **ECG + TTI SVM, IEEE Access 2020 (Aramendi group) [V]:** 1,140 segments from 187 patients. The best 5-feature model reached about 92.6% accuracy for pulse vs no pulse. — [EHU repository](https://addi.ehu.es/handle/10810/64710); [DOAJ](https://doaj.org/article/ad5fd3c678f3439ba7474bc9efd8f0e2)
- **ROSC detection from defib signals [V, snippet]:** 94.4% sensitivity, with a median delay of 57 s after ROSC onset. — [PMC4724594](https://pmc.ncbi.nlm.nih.gov/articles/PMC4724594)
- **Impedance circulation component (ICC) for PEA prognosis and multiclass circulation states (Urteaga, Aramendi, Elola et al., Entropy 2021) [V]:** the multiclass true-PEA / pseudo-PEA / pulsed classifier also used capnography. — [PMC8307658](https://pmc.ncbi.nlm.nih.gov/articles/PMC8307658)
- **ETCO2 [V]:**
  - Tat et al. (OHCA, n=178): a 10 mmHg rise gave 33% sensitivity and 97% specificity for ROSC. A 20 mmHg rise gave 20% sensitivity and 99% specificity.
  - A waveform-based method (n=205 OHCA) reached 95.4% sensitivity and 94.9% specificity. — [UMEM pearl](https://em.umaryland.edu/educational_pearls/3056/); [PMC6581839](https://pmc.ncbi.nlm.nih.gov/articles/PMC6581839)
- **PPG [V]:**
  - Lee et al., BMJ Open 2019: smartwatch PPG at the wrist, forehead and carotid in 50 OHCA patients. At the wrist, sensitivity matched manual pulse check and specificity was higher.
  - Edgar et al. (wrist PPG, withdrawal of life support, not CPR): 23/23 sensitivity in the test set with 1 false alert.
  - Google, "Automated loss of pulse detection on a commercial smartwatch": a research publication.
  - Wijshoff (TU Eindhoven thesis 2016; patent US12414716 on a dual-probe PPG oximeter for CPR): motion artefact from compressions is the core problem. — [Hanyang PDF](https://scholarworks.bwise.kr/hanyang/bitstream/2021.sw.hanyang/147733/1/e023627.full.pdf); [EUR](https://pure.eur.nl/en/publications/automated-cardiac-arrest-detection-using-wrist-derived-photopleth/); [Google Research](https://research.google/pubs/automated-loss-of-pulse-detection-on-a-commercial-smartwatch); [Patent US12414716](https://patents.google.com/patent/US12414716); [Wijshoff thesis](https://typeset.io/pdf/on-photoplethysmography-artifact-reduction-and-applications-1tz461tmqu.pdf)

### Inferences
- The ECG + TTI pulse classifier needs **no new hardware**, because impedance is already sampled. It is the cheapest "smart" circulation feature. However, it only works during compression pauses (analysis windows), and the validation data are retrospective, from a small number of research groups.
- A PPG clip adds a cable and a sensor. ETCO2 needs a capnography line. Both are outside a belt-worn AED's natural sensor set.

### Gaps
- Commercial AED with automated pulse detection: not found.
- Seismocardiography or accelerometer pulse detection at the pads during arrest: not found this session.
- Prospective validation of the ECG/TTI pulse classifiers: not found.

---

## 3. Metronomes, prompting, time-to-shock, operating modes for trained users

### Takeaway
A metronome reliably fixes compression rate on manikins, and one field study found 62% of minutes on target with it vs 18% without. AED model and prompt design change time to first shock by up to about 50 s and perishock pause by up to about 45 s in simulation. The professional benchmark for time from recognition to shock is about 51 s median (EMTs). Evidence specific to semi-automatic vs manual override for trained users, noisy environments, and visual vs voice prompts was **not found** in this pass.

### Cited findings
- **Metronome [V]:**
  - Kern KB et al. 2010 manikin RCT, 34 firefighter/EMT pairs: target rate 90-110/min achieved in 34/34 metronome sessions vs 5/34 control sessions.
  - Field OHCA study: rate 109-111 in 62% of minutes with a metronome vs 18% without. The authors call field evidence "limited".
  - A higher metronome rate may degrade bag-mask ventilation quality. — [Arizona experts](https://experts.arizona.edu/en/publications/metronome-improves-compression-and-ventilation-rates-during-cpr-o/); [PMC10320236](https://pmc.ncbi.nlm.nih.gov/articles/PMC10320236); [PubMed 26459645](https://pubmed.ncbi.nlm.nih.gov/26459645)
- **Prompt design and device model [V]:**
  - Commercial public-access AEDs: time to first compression was 25 s without an AED and 50-148 s with AEDs, depending on model. Perishock pause ranged from 12 to 46 s.
  - With an optimised action sequence, no-flow fraction fell to 0.32-0.41 and perishock pause to 1-19 s.
  - Six Austrian AEDs, untrained users: time to first shock 78 s (95% CI 68-89) to 128 s (110-146), with device type significant (p<0.0001). — [PMC4475613](https://pmc.ncbi.nlm.nih.gov/articles/PMC4475613/); [Augsburg PDF](https://opus.bibliothek.uni-augsburg.de/opus4/files/44526/AEDHeller.pdf)
- **Professional benchmark [V]:** EMTs with AEDs in VF had a median of 51 s (IQR 43-64) from recognition to shock. The authors propose 90 s as a minimum standard. — [PubMed 12385600](https://pubmed.ncbi.nlm.nih.gov/12385600/)
- **Ongoing trials on prompts [V]:**
  - Medtronic-funded RCT of altered AED voice prompts to cut hands-off time: [ISRCTN 72257677](https://www.isrctn.com/pdf/72257677)
  - NCT07520877, AED-optimised telephone-CPR, measuring time to first shock and perishock pause: [ClinicalTrials.gov](https://clinicaltrials.gov/study/NCT07520877)
- **Pad placement (relevant to pre-connected pad design) [V, ILCOR 2025 full text]:**
  - For providers trained in manual defibrillation, anterolateral (AL) placement is the good-practice default for speed. Anteroposterior (AP) is allowed if AL is not feasible.
  - One EMS cohort found AP vs AL ROSC aOR 2.64 (1.50-4.65), with no difference in survival (aOR 1.55, 0.83-2.90). — [ILCOR BLS 2025 CoSTR pp. 52-54](https://www.ilcor.org/uploads/BLS-2025-COSTR-Full-Chapter.pdf)

### Inferences
- Prompt length and sequence are a **free, software-only** lever worth tens of seconds of hands-on time. For trained users, a terse "pro" prompt set (or prompts off, with tones only) is defensible.
- An adjustable metronome (100-120/min) costs nothing beyond a speaker or haptic driver.

### Gaps
- Not found:
  - RCTs of semi-automatic vs fully automatic vs manual override in professional hands
  - visual vs voice prompts in noise
  - ECG display usefulness on BLS-level responder AEDs
  - prompt intelligibility at siren or crowd noise levels

---

## 4. Connectivity and system integration

### Takeaway
Connectivity helps with **getting an AED to the patient** (drones, volunteer dispatch) and with **readiness monitoring**. Even these have modest or null outcome evidence in RCTs. For a device already carried by a responder, the AED-delivery benefits do not apply. What remains useful is readiness telemetry, automatic incident logging, and data hand-off. Public AED fleets are 15-21% non-ready on inspection, and that is the strongest argument for remote readiness reporting.

### Cited findings
- **Drone AED, Schierbeck S et al., Lancet Digit Health 2023 (OBS, western Sweden, about 200,000 population) [V]:**
  - Drones delivered an AED in 55 suspected OHCA. In 37 of them (67%) the drone arrived before the ambulance, with a median lead of 3 min 14 s.
  - Among 18 confirmed OHCA, the AED was attached in 6 (33%) and a shock was advised in 2. One patient survived.
  - Earlier pilot (Eur Heart J 2022): 11 deliveries (92% success), drone first in 64%, median benefit 1:52 when first. — [VGR PDF](https://www.vgregion.se/contentassets/74ed4a24d73c4f648bcc87aa22721539/schierbeck-et-al-the-lancet-dh-aed-drones-231123.pdf); [KI news](https://news.ki.se/drones-enabled-the-use-of-defibrillators-before-ambulance-arrival); [PubMed 34438449](https://pubmed.ncbi.nlm.nih.gov/34438449/)
- **Smartphone volunteer dispatch [V]:**
  - SAMBA trial (Sweden, JAMA Cardiol 2023): bystander AED attachment 13.2% vs 9.5% (difference 3.8 pp, 95% CI -0.3 to 7.9, p=0.08), not significant.
  - Danish HeartRunner RCT (ESC Congress 2026, Healio report): activation did **not** improve 30-day survival. This comes from a news report only. — [JAMA Cardiol](https://jamanetwork.com/journals/jamacardiology/fullarticle/10.1001/jamacardio.2022.4362); [PubMed 36449309](https://pubmed.ncbi.nlm.nih.gov/36449309/); [Healio 2026](https://www.healio.com/news/cardiology/20261009/community-first-responder-activation-does-not-improve-survival-after-cardiac-arrest); [NCT03835403](https://clinicaltrials.gov/study/NCT03835403)
- **Avive 4 Minute City/Community (MFR plus one paper) [V]:**
  - Program goal: a shock within 4 min of accepting an alert. In Cumberland County, PA, 911 telecommunicators dispatched a connected AED via the RapidSOS integration.
  - A *Resuscitation Plus* report (March 2025) exists, but I could not retrieve its outcome numbers.
  - A university campus deployed 73 connected units and "eliminat[ed] manual inspections". This is a manufacturer/LinkedIn claim. — [Avive 4 Minute Community](https://avive.life/4-minute-city/); [Avive Cumberland case](https://www.avive.life/customer-stories/cumberland); [RapidSOS blog](https://rapidsos.com/blog/dispatching-aeds-like-first-responders-how-avive-rapidsos-are-transforming-emergency-cardiac-response/)
- **Public AED non-readiness on inspection (OBS) [V]:**
  - Seoul 2013-2017, about 23,619 AEDs per year: 15.4% not ready to use, and 44.1% had less than 24 h usability.
  - Bornholm, Denmark: almost one fifth of registered AEDs not functional, mainly from expired electrodes (42.5% of non-functional units), failed self-tests, or access obstacles.
  - US 4-region study (about 322 devices; news summary): 21% failed at least one test phase, about 5% had dead batteries, and nearly 40% in Central Illinois had non-functional batteries or expired pads. — [PMC9232625](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9232625/); [PubMed 35618078](https://pubmed.ncbi.nlm.nih.gov/35618078/); [DotMed](https://www.dotmed.com/news/story/42586/)
- **Police AEDs [V, mixed quality]:**
  - Rochester, MN: police AEDs since 1990 under R. White at Mayo, with reported survival 54-58%. These come from advocacy and Wikipedia sources.
  - Police AEDs did not improve survival where fire first-response already carried AEDs (1998), nor in suburban/rural Indiana (PARADE).
  - Miami-Dade police AEDs cut response time by about 3 min. — [Wikipedia, police AEDs](https://en.wikipedia.org/wiki/Police_vehicles_equipped_with_automated_external_defibrillators_in_North_America); [SCA-Aware](https://sca-aware.org/sca-news/rochester-mn-achieves-58-survival-rate-thanks-to-police-aed-program)
- **PulsePoint, GoodSAM outcome data; automatic transmission of event data to receiving hospitals:** not searched or not found this session.

### Inferences
- For a responder-carried AED, the evidence-backed connectivity features are:
  1. readiness heartbeat (battery, pads, self-test result) to the agency
  2. automatic incident timestamping and location log
  3. post-event upload for QI review, which ILCOR ties feedback benefit to
- Live streaming to hospitals has no outcome evidence that I found.
- The police AED literature says the benefit depends on police arriving before EMS. The device's value lies in time-to-shock, so logging time from power-on to shock is a key metric.

### Gaps
- Published Avive outcome numbers; PulsePoint Verified Responder and GoodSAM outcomes; body-worn radio or CAD integration studies: not found.

---

## 5. Event recording and data formats

### Takeaway
Manufacturer defibrillator files are proprietary and usually need vendor software or conversion to CSV before research use. Registries use Utstein, and in the US, NEMSIS v3.5 harmonised with CARES. I found no vendor-neutral defibrillator waveform export standard in use. For an open device, publishing a documented open format plus a converter is a real differentiator.

### Cited findings
- **Proprietary files are a bottleneck [V]:** raw files often must be converted from proprietary formats to readable files, usually CSV, then de-identified and linked to Utstein-style clinical data (Resuscitation Plus, June 2024 review). — [DOAJ, Resusc Plus 2024](https://doaj.org/article/55078aab193b479892aad4fc418099bb); [PMC10960142](https://rcastoragev2.blob.core.windows.net/2fa8656fcb72fa17448f3c14dd6cf356/main.PMC10960142.pdf)
- **ZOLL [V]:** agencies must buy Code-Stat Data Review software to view recorded data. ZOLL offers an SDK, and a third-party adapter was built to ingest any manufacturer's files (JEMS). — [JEMS](https://www.jems.com/patient-care/why-can-t-we-all-get-along/)
- **NEMSIS v3.5.0 and CARES harmonised their cardiac-arrest data elements [V]:** — [NEMSIS PDF](https://nemsis.org/wp-content/uploads/2020/01/Harmonize-with-CARES.pdf)
- **Utstein core data elements [V]:** bystander CPR/AED, first monitored rhythm, time to defibrillation, and others. — [National Academies, Table F-1](https://www.nap.edu/read/21723/chapter/15)
- **Background, not verified this session [BG]:**
  - Typical AEDs record pad ECG, impedance, events (analysis result, shock energy, delivered current, prompts), CPR metrics if sensed, and optionally ambient audio. Philips, ZOLL and Physio/Stryker each use proprietary formats.
  - Open candidates include HL7 aECG (XML, FDA ECG submissions), SCP-ECG (EN 1064 / ISO 11073-91064), DICOM waveform, EDF+, and PhysioNet WFDB.
  - Under HIPAA, recorded ECG and audio tied to a patient encounter are PHI when held by a covered entity such as an EMS agency. The device maker is not automatically a covered entity. Audio of bystanders raises extra consent and wiretap-law questions that vary by US state.

### Gaps
- Not found:
  - any published defibrillator-specific open format standard
  - any study of how often recorded audio is used or consented
  - NEMSIS fields that accept device-generated CPR metrics directly

---

## 6. Self-test regimes, battery and pad readiness, field failure data

### Takeaway
Commercial AEDs run tiered automatic self-tests: daily for battery, pads and electronics; weekly for a partial HV charge; monthly for a full-energy charge into an internal load. DeLuca 2012 shows the failures that matter most happen **during charge and delivery (45%)**, and battery/power (23%) and pads/connectors (24%) dominate the causes. Self-tests missed some faults that later caused unexpected power-off.

### Cited findings
- **DeLuca LA et al., Ann Emerg Med 2012;59(2):103-111 (OBS, FDA MAUDE review, June 1993 to Oct 2008) [V, abstract and press]:**
  - 1,284 adverse events associated with fatality were reviewed, of which 1,150 were failed defibrillation attempts. Press material says these came from 40,787 AED-related reports.
  - Failure modes: 37 never powered on, 252 failed to complete rhythm analysis, and 524 failed to deliver a recommended shock.
  - 45% of failures occurred during charge and shock delivery.
  - Causes: battery/power 23.2%, pads/connectors 23.7%.
  - "Some conditions not detected by AEDs during self-test" caused unexpected power-off.
  - Inter-rater agreement on most likely cause was only 55.9%, and MAUDE reports are often incomplete. — [Arizona experts](https://experts.arizona.edu/en/publications/analysis-of-automated-external-defibrillator-device-failures-repo/); [Cardiovascular Business](https://cardiovascularbusiness.com/topics/clinical/structural-heart-disease/study-one-fifth-aed-failures-traced-battery-power-problems); [Arizona EM news](https://emergencymed.arizona.edu/news/aed-failures-linked-cardiac-arrest-deaths)
- **Self-test schedules (MFR manuals) [V]:**
  - GE Responder AED Pro: daily test of battery, pads and electronics; weekly partial HV charge; monthly full-energy charge. — [ManualsLib GE AED Pro](https://www.manualslib.com/manual/3014756/Ge-Aed-Pro.html?page=36)
  - ZOLL AED 3: daily or weekly automatic test (configurable), plus a monthly test that includes a 200 J charge/discharge. A battery-installation self-test also includes a 200 J charge/discharge. — [ZOLL AED 3 manual](https://zoll.com/-/media/product-manuals/aed3/01-en/9650-000751-01-sf_f.ashx)
  - Defibtech DDU-120: daily, weekly, monthly and quarterly self-tests. Self-test shocks are dissipated internally, and no voltage appears at the pads. — [Manualsdir DDU-120](https://www.manualsdir.com/manuals/672866/defibtech-ddu-120-series.html?page=37)
  - Lid-activated self-test patents (Heartstream, 1990s). — [US5645571A](https://patents.google.com/patent/US5645571)
- **Background, not verified [BG]:**
  - Philips HeartStart units test pad gel and integrity through the pre-connected pad cartridge's test load.
  - In 2015 FDA issued a final order requiring PMA for AEDs and accessories, citing roughly 72,000 MDRs (2005-2014) and over 100 recalls.

### Inferences
- For a belt-worn unit, the self-test design should exercise the **charge-and-deliver path** (full-energy charge into an internal load on a schedule), not just logic checks. That is where 45% of fatal failures occurred.
- It should also check the pad connector or electrode path, because pads and connectors caused about 24% of failures and expired electrodes are the top cause of field non-readiness.
- Battery state-of-health should be measured **under HV-charge load**. A resting-voltage check missed faults, per DeLuca's note on unexpected power-off.
- A body-worn device sees heat, vibration and sweat, so pad-gel drying and connector wear deserve more frequent checks than for a wall-cabinet AED.

### Gaps
- Not found:
  - published method for pad gel-dryness detection by impedance on pre-connected pads
  - battery SoH algorithms used in AEDs
  - post-2012 MAUDE failure analyses with comparable rigour

---

## 7. Safety-critical firmware practice: Rust, formal methods, IEC 62304 Class C

### Takeaway
Ferrocene (Ferrous Systems) is an open-source Rust toolchain **qualified for IEC 62304 Class C** since January 2025, as well as ISO 26262 ASIL-D and IEC 61508 SIL 4. Its qualified targets as of early 2025 were x86-64 Linux, Armv8-A bare metal and QNX. **I found no evidence that a Cortex-M (thumbv7em) target is qualified**, and **no FDA-cleared medical device publicly documented as running Rust firmware.** Formal and model-based precedents exist for pacemakers and infusion pumps.

### Cited findings
- **Ferrocene IEC 62304 Class C qualification announced 14 Jan 2025 [V]:** described as the first Rust toolchain qualified for medical device software. The qualification documents are open source (MIT/Apache-2.0). — [Ferrous Systems blog](https://ferrous-systems.com/blog/ferrocene-achieves-iec-62304-qualification/); [BusinessWire](https://www.businesswire.com/news/home/20250114192138/en); [eeNews Europe](https://www.eenewseurope.com/en/first-rust-toolchain-qualified-for-medical-devices/)
- **Qualified targets [V]:**
  - Ferrocene 24.11 and 25.02 were qualified on x86-64 Linux, Armv8-A bare metal, and QNX Neutrino 7.1.0 (x86-64, Armv8-A). Armv8-R was in preview.
  - "Qualification-ready" targets can be qualified on request.
  - Ferrous ran Rust on Cortex-R52 in March 2025.
  - Upstream Rust lists thumbv7em as Tier 2 (not Ferrocene qualification).
  - A Ferrocene 25.11.0 release was mentioned, but I did not verify its target list. — [Embedded Computing](https://embeddedcomputing.com/technology/software-and-os/ides-application-programming/ferrocene-becomes-first-rust-toolchain-to-achieve-iec-62304-qualification); [Ferrous Cortex-R52 PR PDF](https://ferrous-systems.com/pdf/rust-cortex-r52-press-release-2025-03-11.pdf); [rustc platform support](https://doc.rust-lang.org/rustc/platform-support/thumbv7em-none-eabi.html)
- **No cleared Rust medical device found [V, negative]:** the search returned only advocacy and consultancy pieces (TTP, Yalantis). — [TTP](https://ttp.com/?p=4774); [Yalantis](https://yalantis.com/blog/rust-for-medical-devices/)
- **FDA Generic Infusion Pump (GIP) project [V]:**
  - It provides FDA reference hazard analysis and safety requirements for a generic patient-controlled analgesia (PCA) pump, which manufacturers can adapt.
  - UPenn turned the GIP model into UPPAAL timed automata and generated code for a "safety-assured" GPCA implementation.
  - Kansas State built on the same work.
  - The Pacemaker Formal Methods Challenge came from McMaster and the Software Certification Consortium. — [FDA](https://www.fda.gov/medical-devices/infusion-pumps/infusion-pump-software-safety-research-fda); [UPenn GPCA](https://repository.upenn.edu/handle/20.500.14332/6530); [PMC4052948](https://pmc.ncbi.nlm.nih.gov/articles/PMC4052948); [UMN GPCA](https://crisys.cs.umn.edu/gpca.shtml)
  - [BG] The pacemaker challenge used the 2007 Boston Scientific PACEMAKER system specification.
- **IEC 62304 Class C specifics [V, secondary sources]:**
  - Detailed design (clause 5.4) is required for each unit and interface only at Class C.
  - Unit verification (5.5) is required at Class B and C, and Class C adds acceptance criteria.
  - Test procedures must themselves be evaluated.
  - SOUP items need recorded name, version and manufacturer, functional and performance requirements, required hardware and software, and evaluated known anomalies.
  - Software with no assigned class defaults to Class C. — [Wikipedia IEC 62304](https://en.wikipedia.org/wiki/IEC_62304); [ScienceSoft Class C document package](https://www.scnsoft.com/files/documents-package-for-iec-62304-safety-class-c.pdf); [MathWorks](https://www.mathworks.com/discovery/iec-62304.html)
- **[BG] What else 62304 expects at any class:**
  - software development plan
  - requirements
  - architecture
  - risk management linkage to ISO 14971
  - configuration management
  - problem resolution
  - maintenance plan
  - integration and system testing
  - release records

  IEC 62304 Amendment 1 (2015) applies. A second edition had been in drafting for years, and I did not verify its 2025-2026 status.

### Inferences
- Rust is now *qualifiable* for Class C at the compiler level, but a Cortex-M AED would need Ferrous Systems to qualify that target on request, or the team would have to justify an unqualified upstream compiler as SOUP or tool validation.
- Every crate (HAL, RTOS, `heapless` and so on) becomes SOUP with documented requirements and anomaly review. A small dependency set is therefore a regulatory cost saving, not only an engineering preference.
- The GIP and pacemaker precedents show the realistic formal-methods scope: **model the shock and therapy state machine** (timed automata or a model checker) and verify the safety invariants. Do not attempt to verify all the code.

### Gaps
- Not found:
  - Ferrocene's 2026 qualified target list
  - any public case study of a medical device in Rust
  - the IEC 62304 edition 2 status
  - formal-methods publications specific to defibrillators

---

## 8. Features specific to a responder-worn device

### Takeaway
I found **no published needs assessment or responder survey** on belt-worn AED design. The evidence that does exist points to:
- time-to-shock and prompt sequence as the main software levers
- feedback that crews will not mute, given the 14% audible-mute rate in Hostler
- readiness self-test focused on the charge and delivery path and pads
- open event export to feed QI programmes, the condition under which ILCOR supports feedback

### Cited findings
- Crews muted audible feedback in 14% of feedback-period cases. — [PubMed 21296838](https://pubmed.ncbi.nlm.nih.gov/21296838/)
- Device and prompt design change time to first compression by about 100 s and perishock pause by up to 45 s. — [PMC4475613](https://pmc.ncbi.nlm.nih.gov/articles/PMC4475613/)
- Pre-connected or expired electrodes are the leading cause of field non-readiness. — [PubMed 35618078](https://pubmed.ncbi.nlm.nih.gov/35618078/); [PMC9232625](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9232625/)
- AL pad placement is the speed-optimised default for trained providers. — [ILCOR BLS 2025 CoSTR](https://www.ilcor.org/uploads/BLS-2025-COSTR-Full-Chapter.pdf)

### Inferences (proposed ranking: evidence strength against cost)
1. **Software-only, high value:**
   - impedance-derived compression rate and compression fraction
   - pause timers and a perishock-pause countdown
   - terse professional prompt set with a metronome
   - full event log and open export

   Evidence: OBS and BENCH for process measures, ILCOR weak recommendation tied to QI. Cost: near zero.
2. **Readiness self-test** with a scheduled full-energy internal-load charge, pad-path continuity, and battery test under load, plus a readiness heartbeat. Evidence: DeLuca failure modes and field non-readiness of 15-21%. Cost: low to moderate (an internal dump load already exists in most designs).
3. **ECG+TTI pulse/ROSC advisory**, labelled "advisory only". Evidence: retrospective classifiers at about 92-94%, no commercial precedent. Cost: software only, but heavy validation burden.
4. **Depth sensing puck** (accelerometer). Evidence: improves depth on firm surfaces, survival evidence mixed. Cost: extra cable or sensor, and up to 30-40% overestimation on soft surfaces unless a second reference sensor is added.
5. **Glove-usable controls, one-handed deployment, haptic or visual cues for noise:** plausible human-factors needs, but **no published evidence was found**. Treat them as design hypotheses to test with responders.

### Gaps
- Not found:
  - responder surveys or needs assessments for wearable AEDs
  - glove-use studies of AED controls
  - prompt audibility studies in sirens or crowds
  - automatic handover report evaluations
