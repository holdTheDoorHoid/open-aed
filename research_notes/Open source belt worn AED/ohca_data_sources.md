# Out-of-hospital cardiac arrest (OHCA) defibrillator-pad recordings: who holds them and how to get them

Research date: 2026-11-27. About 35 web searches and page fetches were used. Every fact has a source URL beside it. **[UNVERIFIED]** marks things taken from memory or from a search-engine summary that I could not open and check. "Not found" means I searched and found nothing.

Several primary pages could not be fetched. These were PubMed Central (it showed a captcha), DOAJ, HHS.gov (it returned 403), and eCFR (it redirected). Where that happened, the fact comes from a mirror or from search-result text and is marked as such.

## Bottom line

- **Only one open set of real OHCA ECGs was found:** "Cardially" on Mendeley Data. It holds 260 pre-shock VF episodes. It is a small VF-only set, and its licence is inconsistent between two sources. It has no non-shockable rhythms, no CPR and no impedance, so it cannot train or validate a shock/no-shock decision by itself.
- **Open data with CPR periods and impedance exists, but has no ECG.** The Paris Fire Brigade set on Mendeley (CC BY 4.0) holds impedance and ventilation volumes.
- **The big US trial datasets (ROC on BioLINCC) are free, but probably tabular only.** BioLINCC lists them as "Study Datasets Only", and no waveform files are mentioned. A request needs an IRB letter and a contract signed by an institutional official other than the PI. An unaffiliated individual cannot meet that without a partner institution.
- **Everyone who actually holds annotated pad recordings shares them only through collaboration or on request.** That includes the Basque/Stavanger groups, Seattle/King County, Paris Fire Brigade with Schiller, Dallas, and the manufacturers. None of them publishes a data-access office or form for this.
- **The realistic route is to partner with a university investigator.** That investigator's IRB issues a not-human-subjects or exempt determination, and their institution signs the data use agreement. The best local candidate is the Penn Center for Resuscitation Science. Penn lists an "Avive AED Collaboration" and CARES on its About page.

## Ranked table

| # | Source | What it holds | Access route | Requirements | Contact route (public) | Source URL | Confidence |
|---|---|---|---|---|---|---|---|
| 1 | **Cardially** (Univ. Brescia + Vinča Institute, Belgrade) on Mendeley Data | 260 OHCA ECGs. Each has 9 s of VF before the first shock plus 1 min after the shock. Labels: shock success 56, unsuccessful 195, indeterminable 9 (majority vote of 3 cardiologists). Includes digitized waveforms, scanned printed ECGs (PDF), extracted features and code. | Open download | None. The Mendeley page shows **CC BY 4.0**. A search summary of the Data in Brief article said **CC BY-NC-ND 4.0** [UNVERIFIED, article page could not be fetched]. Treat CC BY 4.0, the repository's own statement, as governing the data, and keep a record of the conflict. | Not needed. Contributors: Benini, Ivanović, Savardi, Kršić, Hadžievski, Baronio | https://data.mendeley.com/datasets/wpr5nzyn2z/1 ; Data in Brief 2021: https://pmc.ncbi.nlm.nih.gov/articles/PMC7753135/ ; https://www.sciencedirect.com/science/article/pii/S2352340920315158 | High that it exists and is open. Medium on licence. The ECGs appear to be **digitized from paper printouts**, so they are not native device files. |
| 2 | **Paris Fire Brigade (BSPP)**: "Insufflated Volumes and Electrical Impedance for Cardiac Arrest Resuscitations", Mendeley Data, 19 Sep 2024 | Transthoracic impedance from Schiller DGT7 AEDs, AED event markers, and ventilation volumes (EOLife flow sensor), OHCA May to Nov 2023. **ECG not listed.** Episode count not stated on the page. | Open download | **CC BY 4.0** | Authors listed: Bruno Tassart, Emma Menant | https://data.mendeley.com/datasets/34gffrm87w/1 ; companion paper https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11848761/ | High for the licence. Useful only for impedance and CPR-period work. |
| 3 | **NHLBI BioLINCC: ROC PRIMED, CCC, ALPS, CPR (PRIMED substudy), PART, Epistry** | Trial and registry datasets. PRIMED randomized about 8,700 (ITD) and about 9,900 (analyze-early vs late). CCC enrolled 23,711 in the active phase. BioLINCC lists all as **"Study Datasets Only"**. Waveform, ECG or defibrillator files are **not mentioned** on any study page I opened. | Application through BioLINCC (register, then submit a request) | Free. Needs a research plan and **IRB/EC approval or an exempt determination on letterhead**. A Research Materials Distribution Agreement (RMDA) is signed by the PI **and a separate authorized institutional official**. Registration uses an institutional email. Data use runs 12 months, renewable with current IRB. Collaborators need their own request. Data must be deleted at project end. | BioLINCC request form (login) | https://biolincc.nhlbi.nih.gov/studies/rocprimed/ ; https://biolincc.nhlbi.nih.gov/studies/roc_ccc/ ; https://biolincc.nhlbi.nih.gov/studies/roc_alps ; https://biolincc.nhlbi.nih.gov/studies/roccpr ; FAQ https://www.biolincc.nhlbi.nih.gov/faq/ | High on process. Medium-high that **no waveforms** are included; the data dictionaries were not read. |
| 4 | **Penn Center for Resuscitation Science** (Philadelphia) | Long-running cardiac arrest research centre (founded 2006). The About page links "CARES" and an "Avive AED Collaboration". No defibrillator dataset is described publicly. | Collaboration only | Would be Penn's own IRB and DUA | General: crsinfo@uphs.upenn.edu, 215-746-5660 (stated "For general questions") | https://www.med.upenn.edu/resuscitation/contact.html ; https://www.med.upenn.edu/resuscitation/about-us | High that it is active and that the contact is real. Leadership: the About page says "Under the leadership of John Greenwood, MD". Older pages list Benjamin Abella as Director. Use the general address, not a named director. |
| 5 | **CHOP: pediRES-Q collaborative** (pediatric) | Multicentre (target 50+ sites) pediatric resuscitation-quality network with CHOP as data coordinating centre. Collects **monitor/defibrillator files (ZOLL R Series)**. Mostly **in-hospital**. | Collaboration only (no public data access route found) | Not found | Not found for data requests. A study-specific protocol lists CHOP investigator addresses, but those are for that study, not data requests. Use the CHOP Academy for Resuscitation of Children pages. | https://clinicaltrials.gov/study/NCT02708134 ; https://www.chop.edu/centers-programs/academy-resuscitation-children/academy-resuscitation-children-quality-improvement | Medium. Pediatric and in-hospital, so a secondary target. |
| 6 | **Paris Fire Brigade + Schiller (Didon) + Inserm (Jouven)**: pediatric AED study | 349 paediatric OHCA cases (2010–2018), 1,990 ten-second ECG strips with impedance, from Schiller DGT7 and Fred Easy AEDs. Annotated by 2 emergency physicians plus 1 cardiologist; disagreements resolved by consensus, otherwise excluded. | On request / collaboration (**no data availability statement** in the article) | French approvals: CNIL #912309, CCTIRS #12336 | Corresponding author as printed: Emma Menant, (address as printed in the paper) | Resusc Plus 2025;22:100873, https://pmc.ncbi.nlm.nih.gov/articles/PMC11803253/ (read via Europe PMC full text) | High on content. Willingness to share unknown. |
| 7 | **University of Washington / King County EMS** (Coult, Rea, Kudenchuk, Kwok) | King County VF-OHCA registry with **defibrillator recordings (ECG + transthoracic impedance)**, 2005–2015 and 2008–2020 cohorts | Collaboration through registry governance [UNVERIFIED]. Data availability statement **not verified** (PMC blocked) | Not found | Not verified | https://pmc.ncbi.nlm.nih.gov/articles/PMC10860689 ; UW thesis https://digital.lib.washington.edu/researchworks/bitstreams/59c26831-ba9b-417b-bb1b-7594466bde2a/download | Medium on holdings, low on access |
| 8 | **Dallas–Fort Worth Center for Resuscitation Research** (via UPV/EHU papers) | OHCA defibrillator files with concurrent ECG and impedance. The Basque group used a subset (e.g. 1,921 PEA segments) in an Entropy 2021 paper. | Collaboration only [UNVERIFIED] | Not found | Not verified | Search summary of https://pmc.ncbi.nlm.nih.gov/articles/PMC8307658 | Low to medium |
| 9 | **CARES** (US registry) | De-identified Utstein-style registry outcome data. Waveforms **not offered** [UNVERIFIED; no evidence of waveforms found]. | Proposal to CARES Data Sharing Committee | NDA, IRB approval, data-review webinar; 5% data-services fee for externally funded projects; publication timelines (abstract 3 months, paper 9 months for state data) | National data: CARES epidemiologist (named on the CARES page); state data: state coordinator | https://mycares.net/sitepages/datasharecommittee.jsp ; https://mycares.net/sitepages/datasharepolicystate.jsp ; https://mycares.net/sitepages/uploads/CARES%20National%20Dataset%20Fees.pdf | Medium-high on process. Useful for context only, not algorithm training. |
| 10 | **Defibtech** (manufacturer, with UConn Chon lab) | Post-market AED log files from real OHCA. 52 CPR-artifact segments from 43 events were used in a 2026 paper, plus a 464-subject proprietary validation ECG set. | Not released. The paper has no data availability statement. | n/a | Corresponding author as printed: Jaechan Lim, (address as printed in the paper) | https://pmc.ncbi.nlm.nih.gov/articles/PMC13282725/ ; DOI 10.1109/ojcs.2026.3696482 | High on content. Shows manufacturers do hold such data. |

## 1. Openly downloadable OHCA ECG datasets

- **Cardially (Mendeley Data, DOI 10.17632/wpr5nzyn2z.1, v1 17 Feb 2020).** The holdings are as in the table.
  - Licence shown on the repository page: "CC BY 4.0". Source: https://data.mendeley.com/datasets/wpr5nzyn2z/1
  - The Data in Brief article (2021) was summarized by search as CC BY-NC-ND 4.0 [UNVERIFIED]. NC/ND would conflict with open-source use, so ask the authors to confirm in writing that CC BY 4.0 applies to the data.
  - The record says it contains "anonymized scanned printed ECGs (PDF), and digitized waveforms" (repository description). The sampling rate is not stated on the page.
  - The device was reported in a search summary as a HeartStart (Philips) defibrillator [UNVERIFIED].
  - Limitation for a shock-decision algorithm: it contains **only VF before a shock**. There are no non-shockable rhythms (asystole, PEA, organized rhythms), no CPR segments and no impedance.
- **Paris Fire Brigade impedance and ventilation (Mendeley Data, 2024, CC BY 4.0).** Holdings are as in the table; ECG is not listed. It is useful for impedance-based compression detection and ventilation detection. Sources: https://data.mendeley.com/datasets/34gffrm87w/1 and https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11848761/
- **PhysioNet.** No PhysioNet database of OHCA defibrillator-pad recordings was found.
  - The only cardiac-arrest database found is **I-CARE**, which holds post-arrest ICU EEG/ECG from comatose patients (607 patients). It is not pad data and has no arrest rhythms during resuscitation. Source: https://www.physionet.org/content/i-care/2.1/
  - I did not browse the full PhysioNet catalogue. A direct catalogue search for "defibrillator" or "impedance" is still worth doing.
- **Zenodo, Figshare, IEEE DataPort, Dryad, OSF.** One extended search found no OHCA pad-ECG deposit. This is "not found", not proven absent.
- **University of the Basque Country (Aramendi, Irusta, Elola, Isasi).**
  - No open dataset release was found.
  - Their papers use data from Dallas–Fort Worth, Stavanger/Oslo and others (search summary of https://pmc.ncbi.nlm.nih.gov/articles/PMC8307658 and an IEEE Access 2020 paper with 1,140 ECG and impedance segments from 187 OHCA patients: https://doaj.org/article/ad5fd3c678f3439ba7474bc9efd8f0e2).
  - The data belongs to the partner EMS systems. Any request would go to the group as collaborators.
- **Oslo/Stavanger (Eftestøl, Kramer-Johansen, Wik, Nordseth).**
  - No open dataset found.
  - A 2024 Resuscitation Plus review by Nordseth, Eftestøl, Aramendi, Kvaløy and Skogvoll covers the methodological challenges of research on defibrillator data (export, signal processing). It is useful background for writing the request. Source: https://doaj.org/article/55078aab193b479892aad4fc418099bb
- **Graz, Sofia (Krasteva, Jekova, Didon/Schiller), Amsterdam ARREST, Pittsburgh, Vienna, and the Taiwan, Korea and Japan registries.** Not searched individually because of the budget. Not found or not checked.

## 2. Resuscitation Outcomes Consortium (ROC) on BioLINCC

- **Studies listed.** PRIMED, CCC, ALPS, ROC CPR (a PRIMED substudy on real-time CPR feedback), PART, and Cardiac/Trauma Epistry. All PRIMED/CCC/ALPS pages list **"Study Datasets Only"**, and the documents offered are the data dictionary, forms, manual of operations (MOO) and protocol. Sources: https://biolincc.nhlbi.nih.gov/studies/rocprimed/ ; https://biolincc.nhlbi.nih.gov/studies/roc_ccc/ ; https://biolincc.nhlbi.nih.gov/studies/roc_alps ; https://biolincc.nhlbi.nih.gov/studies/roccpr
- **PRIMED.** The trial was enrolled under exception from informed consent. Its page shows "Commercial Use Data Restrictions: No" and "Data Restrictions Based On Area Of Research: No". CCC shows the same flags.
- **Waveforms.** No page mentions ECG waveforms or defibrillator files. ROC is known to have abstracted CPR-process measures from defibrillator files, such as compression fraction, rate and depth [UNVERIFIED; that would appear as summary variables in the data dictionaries, which I did not read]. **Expect tabular data only.** Before applying, download the ROC CPR and PRIMED data dictionaries (public PDFs on those pages) to confirm.
- **CCC is also on EGA as phs003901,** with an access note saying its access differs from typical dbGaP studies. Source: https://ega-archive.org/studies/phs003901. Not opened.
- **What a BioLINCC request requires** (FAQ, https://www.biolincc.nhlbi.nih.gov/faq/):
  - **IRB/EC documentation.** The FAQ requires "documentation of review and approval from an Institutional Review Board (IRB) or Ethics Committee (EC)". It says an IRB/EC determination that the research is exempt from review suffices for most studies. A requester with no IRB "may consider obtaining review from an independent IRB" (which may cost money).
  - **RMDA signatures.** The agreement is signed by the PI and "an individual at the institution/entity that has authority to enter into a contract on behalf of the institution", who must be someone other than the PI.
  - **Institutional email.** Registration uses an institutional email.
  - **Cost.** Data are free.
  - **Term.** 12-month term, renewable with current IRB evidence (https://www.biolincc.nhlbi.nih.gov/data-renewal/).
  - **Other conditions.** Collaborators need their own request. Changes to the aims need NHLBI approval. Data must be deleted at project end.
  - The FAQ says nothing about publishing derived models.
- **Conclusion.** An unaffiliated individual realistically cannot be the requester. A university PI, or possibly a nonprofit with a signing official plus a commercial IRB, could be. Whether BioLINCC accepts a non-academic "entity" was not verified.

## 3. Registries

- **CARES (US).**
  - Process: a proposal to the Data Sharing Committee, then an NDA, IRB approval and a data-review webinar. Feedback takes 4 weeks according to the 2020 guide, or 6–8 weeks according to the state policy page. Externally funded projects pay a 5% data-services fee.
  - Contents: de-identified registry outcome data. Physio-Control built and maintains the CARES software (per Stryker material).
  - No evidence of waveform release [UNVERIFIED that none exists].
  - Sources: https://mycares.net/sitepages/datasharecommittee.jsp ; https://mycares.net/sitepages/datasharepolicystate.jsp ; https://mycares.net/sitepages/uploads/2020/Data%20Sharing%20User%20Guide%202020.pdf
- **King County (Seattle).** The registry includes defibrillator recordings (see table row 7).
- **EuReCa, the Danish, Swedish and Norwegian registries, VACAR, PAROS and All-Japan Utstein.** Not checked within the budget. From memory these are Utstein outcome registries without waveform release [UNVERIFIED].

## 4. Manufacturers

- **No published researcher data-request programme was found** for Stryker/Physio-Control, ZOLL, Philips, Schiller, Corpuls, Nihon Kohden or Avive (search, 2026-11-27).
- **ZOLL.** A 2013 press release pledged to share patient data and offer SDKs for device data. It is old and does not describe a research data route. Source: https://www.zoll.com/news-releases/2013/01/28/medical-device-pledge-patient-data
- **Stryker/Physio-Control.** CODE-STAT is a clinical case-review tool, not a research intake.
- **Defibtech.** It uses its own post-market AED logs in academic work with UConn (table row 10). This shows the data exists at manufacturers, but nothing is released.
- **Schiller.** Didon (Schiller) co-authors the Paris Fire Brigade pediatric work using DGT7/Fred Easy recordings (table row 6). Schiller is a possible partner through BSPP/Inserm.
- **Avive.** Penn lists an "Avive AED Collaboration" (https://www.med.upenn.edu/resuscitation/about-us). The nature of that collaboration was not verified.
- **Patent filings.** A US patent on "condition-based suppression of CPR artifacts" turned up (https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/12427328). The assignee was not checked. It is relevant to freedom-to-operate, not data.

## 5. Pediatric rhythm libraries and real CPR-artifact data with a reference signal

- **Pediatric.**
  - No open pediatric arrest-rhythm database was found.
  - Requestable candidates: Paris Fire Brigade/Inserm (349 cases, 1,990 strips; row 6) and CHOP pediRES-Q (row 5, mostly in-hospital, ZOLL files).
  - The historic Atkins et al. 2008 pediatric rhythm database used rhythms of at least 9 s annotated by pediatric electrophysiologists (search summary; https://www.pubmed.gov/17765384/). Its availability was not verified.
  - A conference paper notes "no public database of ECG records compliant with the AHA statement" (search summary; https://cinc.org/archives/2013/pdf/1091.pdf).
- **CPR artifact with a reference signal.**
  - Open: Paris Fire Brigade impedance and ventilation (no ECG).
  - Held but not released: King County (ECG + impedance), Dallas (ECG + impedance), Stavanger/Oslo, Defibtech logs.
  - No open set of ECG with compression-depth or accelerometer force was found.

## 6. Philadelphia

- **Penn Center for Resuscitation Science.** Active. The general contact crsinfo@uphs.upenn.edu, 215-746-5660, is published "For general questions" (https://www.med.upenn.edu/resuscitation/contact.html).
  - The About page says "Under the leadership of John Greenwood, MD". Some faculty pages still list Benjamin Abella as Director, so leadership on the public pages is inconsistent. Address the letter to the centre, not to a person.
  - The About page links CARES and an Avive AED collaboration.
  - Penn has previously used Philadelphia Fire Department OHCA data: 4,909 cases, 2008–2012 (search summary of https://www.jems.com/patient-care/cardiac-resuscitation/study-finds-black-cardiac-arrest-victims/).
- **CHOP.** Data coordinating centre for pediRES-Q (PI listed on ClinicalTrials.gov NCT02708134: Vinay Nadkarni). Funded by the Laerdal Foundation, then by unrestricted ZOLL grants (search summary). Public data-request contact: not found.
- **Thomas Jefferson.** A 2025 retrospective review of 701 OHCA encounters from "a single urban EMS agency" (2017–2022) was published in a Cambridge journal (search summary: https://resolve-he.cambridge.org/core/product/4320A39F5C81C58E9656F1F0BCB9F001). The agency was not confirmed as PFD. No defibrillator-file work was found.
- **Temple.** Not found.
- **Philadelphia Fire Department EMS.** No public research-collaboration route found. A 2016 Philadelphia Magazine article reported that PFD had declined to publicize its CARES data (https://www.phillymag.com/2016/04/19/cardiact-arrest-philadelphia-fire-department/). Approaching PFD directly as an unaffiliated individual looks unpromising. Going through Penn is better.

## 7. Practical requirements for an unaffiliated open-source project

- **HIPAA de-identification** (45 CFR 164.514, read via https://www.law.cornell.edu/cfr/text/45/164.514). Safe Harbor requires removing these identifiers:
  - (C) "All elements of dates (except year)" for dates directly related to an individual, and ages over 89. **Device files carry timestamps.** Ask for times relative to the start of the recording.
  - (M) "Device identifiers and serial numbers". **The AED serial number in the file header must be stripped.**
  - (P) "Biometric identifiers, including finger and voice prints". **Many monitor-defibrillators record scene audio. Ask for the audio channel to be removed.**
  - (R) "Any other unique identifying number, characteristic, or code". **Strip case and incident IDs.**
  - Alternatives: Expert Determination ("the risk is very small", documented by a qualified person). Or a **Limited Data Set**, which may keep dates but needs a DUA that forbids re-identifying or contacting individuals and restricts use to research, public health or operations.
- **"Not human subjects research."** The OHRP coded-private-information guidance (2018, replacing 2008) says research using only coded data, where investigators cannot link it to individuals, can fall outside "human subjects research". OHRP recommends the **institution**, not the investigator, makes that determination. It also says collaboration with the data provider can change the analysis. Source: https://www.hhs.gov/ohrp/regulations-and-policy/guidance/research-involving-coded-private-information/index.html (via search; the page itself returned 403).
  - In practice data holders (BioLINCC, CARES) still ask for an IRB letter: exemption, or not-human-subjects determination.
  - An unaffiliated person can buy a determination from an independent or commercial IRB (BioLINCC FAQ). Whether specific commercial IRBs (e.g. WCG, Advarra) accept individuals without an institution: [UNVERIFIED].
- **Signatory problem.** DUAs and RMDAs need an institutional signing official other than the PI (BioLINCC). Realistic routes:
  - **Partner with a university investigator.** Best fit: Penn CRS. The data stays at the institution and the project receives code-reviewed results or model weights.
  - **Form or join a nonprofit** with an officer who can sign, plus a commercial IRB determination.
  - **Fund a collaborator** (e.g. UPV/EHU or Stavanger) to run your open-source code on their data and return metrics. This "code-to-data" pattern is [UNVERIFIED as offered by any specific group].
- **Open-source training and publishing model parameters.**
  - No DUA I checked explicitly addresses publishing trained model weights.
  - The closest precedent is PhysioNet. Its Credentialed Health Data Use Agreement 1.5.0 forbids sharing data access and limits use to "lawful scientific research". It **requires code used for published results to be shared openly**.
  - PhysioNet kept some models trained on restricted clinical text (Clinical-T5) behind credentialed access because of leakage risk. Sources: https://physionet.org/content/discq/view-dua/1.0 ; https://physionet.org/content/clinical-t5/
  - "Research only" clauses could be read to exclude a deployable device product. ECG-classifier weights carry far less leakage risk than text models [my judgement, not sourced].
  - **Recommendation: put the question in writing in every request.** Ask: "May trained model parameters and aggregate performance results be published under an open-source licence? May the code be open-source?" Get the answer in the DUA.
- **Commercial-use flags.** ROC datasets on BioLINCC show "Commercial Use Data Restrictions: No", which helps with a future device.

## 8. Annotation and reporting standards to ask for

- **AHA 1997 statement (Kerber et al.)** on specifying and reporting AED arrhythmia-analysis performance (search summaries; original not opened):
  - It classifies rhythms as shockable (coarse VF, rapid VT), intermediate (fine VF, other VT) and non-shockable (NSR, other organized rhythms, asystole) [the intermediate class and thresholds are UNVERIFIED].
  - Commonly cited targets: shockable sensitivity >90%, non-shockable specificity >95%, NSR >99% (secondary citation: https://pmc.ncbi.nlm.nih.gov/articles/PMC4956226 , https://www.hmpgloballearningnetwork.com/site/eplab/articles/performance-aspects-automated-rhythm-detection-capabilities-aeds).
  - The 200 µV coarse/fine VF threshold and the asystole threshold: [UNVERIFIED].
  - **Ask for labels mapped to these classes,** plus the segment length used (commonly 9–10 s strips).
- **Annotator practice seen in recent OHCA work:**
  - Cardially: 3 cardiologists, majority vote.
  - Paris pediatric: 2 emergency physicians plus 1 cardiologist, blinded, consensus required, undetermined strips excluded, with impedance shown to annotators.
  - **Ask for:** the number and specialty of annotators, the agreement statistic (kappa) or adjudication rule, whether impedance or compression markers were visible, and the CPR on/off interval annotation.
- **What to request, concretely:**
  - Native device export (e.g. ZOLL or LIFEPAK case files, or an EDF/WFDB conversion) with ECG through the pads, transthoracic impedance and the device's own event markers (analysis, shock, CPR prompts).
  - Accelerometer or compression-depth channel where present.
  - Sampling rate and gain.
  - Rhythm annotations by segment, with timestamps relative to the start of the recording.
  - Outcome of each shock.
  - Audio removed, device serial removed, dates removed.

## Not found / could not verify

- Any PhysioNet, Zenodo, Figshare, IEEE DataPort, Dryad or OSF deposit of OHCA pad recordings with ECG (searched, none found; the PhysioNet catalogue was not exhaustively browsed).
- Data availability statements for the King County (Coult et al., PMC10860689) and UPV/EHU–Dallas (PMC8307658) papers. PubMed Central was blocked by a captcha and Europe PMC returned an error.
- Whether the ROC data dictionaries contain CPR-process or waveform-derived variables. The PDFs were not read.
- Whether BioLINCC accepts a nonprofit or non-academic entity as the signing institution.
- The Cardially licence conflict (CC BY 4.0 on Mendeley vs a reported CC BY-NC-ND 4.0 in Data in Brief).
- The nature of the Penn–Avive collaboration, and who formally directs Penn CRS now.
- Any public research or data contact for CHOP pediRES-Q, Philadelphia Fire Department, Temple, or Jefferson.
- Manufacturer research data programmes (none found for any of the seven named).
- Graz, Sofia/Schiller (Krasteva, Jekova), Amsterdam ARREST, Pittsburgh (Menegazzi/Salcido), Vienna, and the Taiwan, Korea, Japan, PAROS, EuReCa, Nordic and VACAR registries: not checked within the search budget.
- The exact text of AHA 1997 (Kerber) thresholds.
- Whether commercial IRBs accept unaffiliated individuals.
