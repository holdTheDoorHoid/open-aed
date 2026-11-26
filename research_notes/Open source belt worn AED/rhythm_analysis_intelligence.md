# AED Rhythm Analysis Algorithms and "Intelligent" AED Features (state of the art, emphasis 2018-2026)

Scope note: ~15 search/fetch calls, as the research brief required. Every item under "Cited Findings" was checked against a source fetched or returned in this session. Items marked **[background, unverified this session]** come from the researcher's prior knowledge of the literature. They are in Inferences or Gaps, not Cited Findings, and must be checked against the primary source before going into a design document or regulatory file. Manufacturer claims are labelled **[manufacturer claim]**.

## 1. Performance requirements (AHA 1997 / IEC 60601-2-4 / AAMI DF80) and commercial performance

### Takeaway
The de facto bar is still the 1997 AHA statement: shockable sensitivity >90% (coarse VF) and >75% (rapid VT), and non-shockable specificity >99% (normal sinus rhythm, NSR) and >95% (other non-shockable rhythms and asystole). IEC 60601-2-4 (incorporated into AAMI/ANSI DF80) uses comparable targets in clause 201.107. Specificity on NSR is the hardest constraint for a device that shocks. A new edition matters here: IEC 60601-2-4 Ed. 3.2 (2026-09) consolidates Amendment 2 (2026). The project must buy it and check the current clause text.

### Cited Findings
- The AHA issued AED arrhythmia-analysis sensitivity and specificity recommendations in 1997. The 2005 guideline update did not revise these performance recommendations. The shockable classes are coarse VF and rapid VT. Source: FDA/CDRH-authored review, "Performance aspects of automated rhythm detection capabilities of AEDs", EP Lab Digest — [HMP Global](https://www.hmpgloballearningnetwork.com/site/eplab/articles/performance-aspects-automated-rhythm-detection-capabilities-aeds). The full text failed to load on fetch, so only the search-snippet content is cited here.
- Figuera et al. (2016) restate the AHA thresholds: Se >90% for shockable rhythms, Sp >95% for non-shockable rhythms, and Sp >99% for NSR — [Figuera et al., PLOS ONE 2016, doi:10.1371/journal.pone.0159654](https://pmc.ncbi.nlm.nih.gov/articles/PMC4956226).
- A secondary source quoting IEC 60601-2-4 lists coarse VF sensitivity >90% and pulseless rapid VT sensitivity >75%. The normative text was not seen. — [Saadat Shooka manual, citing IEC 60601-2-4](https://www.manualslib.com/manual/2876392/Saadat-Shooka.html?page=33)
- In IEC 60601-2-4, clause 201.107 holds the rhythm recognition detector (RRD) requirements and Table 201.102 lists the rhythm categories. The editions are Ed. 3.0 (2010), Amd 1 (2018), and the consolidated Ed. 3.2 (2026-09, including Amd 2) — [VDE preview Ed 3.0](https://www.vde-verlag.de/iec-normen/preview-pdf/info_iec60601-2-4{ed3.0}b.pdf); [IEC 60601-2-4 Amd1 2018 sample](https://cdn.standards.iteh.ai/samples/23437/3b2a0e970a474843aa54b52c5fd17e10/IEC-60601-2-4-2010-AMD1-2018.pdf); [ANSI/AAMI/IEC 60601-2-4 redline preview](https://webstore.ansi.org/preview-pages/AAMI/preview_ANSI+AAMI+IEC+60601-2-4-2010+A1-2018+RED+LINE.pdf)
- The AHA recommendation required three-expert agreement on the identity of each test rhythm. This is per a search summary of the AHA document and is not seen verbatim. — [HMP Global](https://www.hmpgloballearningnetwork.com/site/eplab/articles/performance-aspects-automated-rhythm-detection-capabilities-aeds)
- Commercial performance in the field. A prospective Amsterdam study compared a conventional AED (LIFEPAK CR2 control) with one running cprINSIGHT. The control AED reached Se 99.3% and Sp 98.7%. The cprINSIGHT AED reached Se 96% (90% lower confidence limit, LCL, 93%) and Sp 98% (LCL 97%), not significantly different from control — [Resuscitation 2021, S0300-9572(21)00009-5](https://www.resuscitationjournal.com/article/S0300-9572(21)00009-5/fulltext)
- Philips publishes AED algorithm performance in its own documentation **[manufacturer claim]**. The document surfaced but its contents were not extracted — [Philips PDF](https://www.documents.philips.com/assets/20170523/2491dcfcb6504217bf55a77c016945eb.pdf)

### Inferences
- **[background, unverified this session]** The Kerber et al. table is from "Automatic external defibrillators for public access defibrillation: recommendations for specifying and reporting arrhythmia analysis algorithm performance…", Circulation 1997;95:1677-1682, doi:10.1161/01.CIR.95.6.1677. As commonly reproduced, it reads:

  | Rhythm class | Min. test samples | Goal | 90% one-sided LCL |
  |---|---|---|---|
  | Coarse VF | 200 | Se >90% | 87% |
  | Rapid VT | 50 | Se >75% | 67% |
  | NSR | 100 | Sp >99% | 97% |
  | Other non-shockable (AF, SVT, sinus brady, heart block, idioventricular, PVCs) | 30 | Sp >95% | 88% |
  | Asystole | 100 | Sp >95% | 92% |
  | Fine VF (intermediate) | 25 | Report only | — |
  | Other VT (intermediate) | 25 | Report only | — |

  It also asks for "noise-free" recordings plus reporting of artifact effects, and states that the minimum sample counts are not enough on their own to prove performance. Fine VF is defined by amplitude (about <200 µV) and rapid VT by a manufacturer-chosen rate threshold. Verify every number against the original before using it as an acceptance criterion.
- Test against the LCL criteria, not point estimates, and use a patient-disjoint test set that is far larger than the minimum sample counts.
- Field results (cprINSIGHT control arm, Se 99.3%, Sp 98.7%) show that mature commercial algorithms beat the AHA minimums by a wide margin in real use. An open device should be benchmarked against that level, not the 1997 floor.

### Gaps
- AAMI DF80 / IEC 60601-2-4 normative sample sizes, the test-database composition rules, and any changes in Amd 2 (2026) were **not found**. These are paywalled standards and need a purchased copy.
- Manufacturer-reported Se/Sp for Zoll, Philips, Stryker, Cardiac Science, Defibtech and HeartSine (published in operator manuals) were not extracted in this session.

## 2. Classical feature-based detectors and comparative evaluations

### Takeaway
Single-feature detectors (TCI, VF filter, SPEC, CPLX, sample entropy and others) all fail the AHA targets when run on unselected ECG. Amann et al. (2005) found the best of them, SCA, reached only Se 71.2% / Sp 98.5%. Combining about 30 such features in a machine-learning classifier (Figuera 2016) reaches Se 94.7% / Sp 96.5% on OHCA data. That combined-feature approach is the strong, explainable baseline any deep-learning approach has to beat.

### Cited Findings
- Amann A, Tratnig R, Unterkofler K. "Reliability of old and new ventricular fibrillation detection algorithms for automated external defibrillators." Biomed Eng Online 2005;4:60, doi:10.1186/1475-925X-4-60, PMID 16253134 — [PMC1283146](https://pmc.ncbi.nlm.nih.gov/articles/PMC1283146/). The study used the MIT-BIH Arrhythmia DB (48 records), CUDB (35 records) and AHA DB (40 files) with no preselection, giving about 333,583 decisions per algorithm on 8 s windows. Results:

  | Algorithm | Se % | Sp % | Acc % | AUC (IROC) % |
  |---|---|---|---|---|
  | TCI (threshold crossing interval) | 75.1 | 84.4 | 83.6 | 82 |
  | ACF (autocorrelation) | 49.6 | 49.0 | 49.0 | 49 |
  | VF filter (8 s) | 18.8 | 100 | 93.0 | 87 |
  | VF filter (4 s) | 19.6 | 99.9 | 93.1 | 85 |
  | SPEC (spectral) | 29.1 | 99.9 | 93.8 | 89 |
  | CPLX (complexity measure) | 59.2 | 92.0 | 89.2 | 87 |
  | STE (signal comparison) | 50.1 | 81.7 | 79.0 | 67 |
  | MEA (mean absolute value) | 51.2 | 84.1 | 81.3 | 82 |
  | SCA (scatter-graph / phase-space, new) | 71.2 | 98.5 | 96.2 | 92 |
  | WVL1 / WVL2 (wavelet, new) | 26.7 / 72.0 | 99.7 / 88.4 | — | 80 / n.c. |

  The authors conclude that no algorithm achieves its originally published Se/Sp on arbitrary ECG. They argue that specificity matters more than sensitivity, and that QRS detectors are unsuitable for VF detection.
- Figuera C, Irusta U, Morgado E, et al. "Machine Learning Techniques for the Detection of Shockable Rhythms in Automated External Defibrillators." PLOS ONE 2016;11(7):e0159654 — [PMC4956226](https://pmc.ncbi.nlm.nih.gov/articles/PMC4956226). The study computed 30 features per segment and used classifiers with built-in feature selection. Public databases gave Se 96.6%, Sp 98.8% and balanced error rate (BER) 2.2%. OHCA data gave Se 94.7%, Sp 96.5% and BER 4.4%. OHCA needed about twice as many features (6 vs 3). Segment length made almost no difference (BER difference <0.5 points).

### Inferences
- Benchmark results on public databases overstate field performance. Both papers show a drop on unselected or OHCA data, so headline numbers from MIT-BIH/VFDB-only papers should be discounted.
- **[background, unverified this session]** Other features widely used in feature sets include bCP/bWT (bin counting), the phase-space reconstruction / time-delay method (Roig, Amann 2007), Li's SVM (Li Q, Rajagopalan C, Clifford GD, IEEE TBME 2014), the Fourier/Hilbert "leakage" measures, and the Count1/2/3 bandpass-count features (Jekova 2007). Li et al. 2014 reported cross-validated Se about 95%, Sp about 99%. Verify before citing.

### Gaps
- Amann 2007 (time-delay method) and Li 2014 numbers were not re-verified in this session.

## 3. Machine learning / deep learning shock advice, TinyML, and validation pitfalls

### Takeaway
Since 2018, CNN and CNN-LSTM shock-advice models meet AHA targets on clean ECG with 3-8 s windows. Embedded deployments are now reported: a 7 s decision took 383 ms on a 200 MHz / 512 kB AED processor, and one STM32 model reached Se 88.9% / Sp 99.6% with <3 s ECG and 0.2 s inference. Two problems persist across this literature: evaluations mix patients across train and test splits, and most rely on public Holter databases. Neither is acceptable for a safety case.

### Cited Findings
- Picon A, et al. "Mixed convolutional and long short-term memory network for the detection of lethal ventricular arrhythmia." PLOS ONE 2019;14(5):e0216756 — [PMC6527215](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6527215/). The CNN-LSTM was tested with 2, 3, 4 and 8 s segments. It met the AHA Se >90% at every length, and Sp >95% for segments longer than 2 s.
- Nguyen MT, et al. "Deep feature learning for sudden cardiac arrest detection in automated external defibrillators." Sci Rep 2018 — [PMC6249221](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6249221/). Listed only; numbers were not extracted.
- Krasteva V, et al. "Deep Neural Network Approach for Continuous ECG-Based AED Shock Advisory System During CPR." J Am Heart Assoc 2021, doi:10.1161/JAHA.120.019065 — [AHA Journals](https://www.ahajournals.org/doi/10.1161/JAHA.120.019065). Over 4-fold cross-validation on the whole dataset (with and without CPR), shock vs no-shock reached Se 95.21% and Sp 86.03%. On ECG without CPR artifact it reached Se 99.04% and Sp 95.2%.
- Jekova I, Krasteva V. "Optimization of End-to-End CNNs for Analysis of OHCA Rhythms during CPR." Sensors 2021;21(12):4105, doi:10.3390/s21124105 — [PMC8232133](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8232133/). The paper notes that few groups use real OHCA-during-CPR data because of privacy and the lack of public records.
- A sliding-window deep-learning analysis during CPR studied how hands-off time affects accuracy — [PMC10181605](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10181605/). Numbers were not extracted.
- Embedded hardware: a JAHA study of a CNN running inside a "digitally connected AED" averaged 383 ms per shock/no-shock decision on a 7 s strip, on the AED's 32-bit 200 MHz processor with 512 kB RAM. The same model took 112 ms on a server CPU for 14 s of ECG — [JAHA, doi:10.1161/JAHA.122.026974](https://www.ahajournals.org/doi/10.1161/JAHA.122.026974). The full text returned HTTP 403, so the figures come from a search summary and the authors, dataset and per-class Se/Sp were not verified.
- "Frugal AI for Automated Cardiac Defibrillation: Balancing Performance and Hardware Constraints" (Springer LNCS chapter, 2025). It detected shockable rhythms from <3 s of ECG with Se 88.9% and Sp 99.6%, with 0.2 s inference on an STM32 NUCLEO board — [Springer](https://link.springer.com/chapter/10.1007/978-3-031-94559-5_14). Note that Se is below the AHA 90% target.
- "LDIAED: A lightweight deep learning algorithm implementable on automated external defibrillators." PLOS ONE 2022, e0264405 — [PLOS](https://journals.plos.org/plosone/article/file?id=10.1371%2Fjournal.pone.0264405&type=printable). Details were not extracted.
- A VF-detection DNN on an ARM microcontroller wearable reported 20 ms inference and 5.9 mA current draw — ["Implementing Deep Neural Networks on ARM-Based Microcontrollers: Application for VF Detection"](https://ouci.dntb.gov.ua/en/works/7A56VyMw/)
- Lim J, Hicks D, Valentine M, Chon KH. "Sensor-Free Shockable Rhythm Classification During CPR Using the Existing ECG Channel of Public-Access AEDs." arXiv:2611.26393, 19 Nov 2026 (preprint, not peer reviewed) — [arXiv](https://arxiv.org/abs/2611.26393). The design is a cascade of three compact CNNs: a second network biased toward the proposed outcome checks each decision, and any disagreement returns "indeterminate". It used 8 s windows and 464 subjects from a manufacturer (data not public), over 15 train/validation partitions. At 0 dB signal-to-artifact ratio, NSR Sp was 99.20-99.43%, other non-shockable Sp 95.48-96.20%, and 1.07-1.48% of segments were indeterminate. It fails the criteria at -3 dB. A decision takes 6.68 ms single-thread on a server CPU. The 8-bit quantized model is 1.45 MiB, which is too large for most Cortex-M4 flash budgets. The authors say validation on the target device is still needed.

### Inferences
- A model of about 1 MiB or less in int8 that runs within a few hundred ms is feasible on Cortex-M7/M55-class parts, going by the 200 MHz / 512 kB AED figure. Cortex-M4 timing for a guideline-compliant shock-advice model was **not found**.
- The "biased second opinion + indeterminate" cascade (arXiv 2611.26393) is a good safety pattern for a class-C device. Pairing a deep-learning classifier with an independent classical feature detector, and allowing "no shock / continue CPR" on disagreement, is a defensible architecture.
- **[background, unverified this session]** Known validation pitfalls:
  - Segments from the same patient appearing in both train and test sets inflate results. Many MIT-BIH/VFDB/CUDB papers do this.
  - VFDB and CUDB have very few patients (22 and 35), so "thousands of segments" is really about 50 independent subjects.
  - Public databases are Holter or monitoring recordings at different sampling rates and bandwidths from defibrillation-pad ECG.
  - The AHA DB is ECRI-licensed, not free.
  - Asystole and fine VF are under-represented.

  Acharya UR et al. (Future Gener Comput Syst 2018, CNN on public databases, about 93% accuracy) is a commonly cited example of a segment-level split.

### Gaps
- No peer-reviewed shock-advice model with published Cortex-M4/M33/M55 latency, RAM and flash that also meets all AHA classes on patient-disjoint OHCA data was **found**.
- No transformer-based shock-advice paper was surfaced in this session (**not found**, not proven absent).

## 4. Rhythm analysis during chest compressions

### Takeaway
Two families exist: adaptive filters that use a compression reference signal (accelerometer, impedance, force), and reference-free CNNs. The best published results are around Se 95-99% / Sp 86-99%, depending on the dataset. The one prospective commercial evaluation (cprINSIGHT) reached a decision during compressions in only about 70% of analyses. The clinical motivation is strong: each 5 s increase in pre-shock pause lowered survival by 18%.

### Cited Findings
- Commercial, prospective: Stryker/Physio-Control cprINSIGHT (LIFEPAK CR2), Amsterdam study. cprINSIGHT decided during compressions in 70% of analyses, with Se 96% (LCL 93%) and Sp 98% (LCL 97%), not significantly different from the control AED (Se 99.3%, Sp 98.7%) — [Resuscitation 2021](https://www.resuscitationjournal.com/article/S0300-9572(21)00009-5/fulltext). An earlier AHA 2018 abstract (n=114) reported decisions in 65-74% of cases, Se 90-100% and Sp 100% — [Circulation 2018;138:Suppl 2 Abstract 18](https://www.ahajournals.org/doi/10.1161/circ.138.suppl_2.18). Stryker's white paper **[manufacturer claim]** quotes Se 95.5% and Sp 98.2% for the same study — [Stryker white paper](https://www.stryker.com/content/dam/stryker/ems/products/lifelink-central/aed-program-administrator/lifepak-cr2-literature/lpcr2_cprinsight_whitepaper_us.pdf)
- Isasi I, et al. "Rhythm Analysis during Cardiopulmonary Resuscitation Using Convolutional Neural Networks." Entropy 2020;22(6):595 — [MDPI](https://www.mdpi.com/1099-4300/22/6/595). The study used 3,319 nine-second segments recorded during compressions (586 shockable, 2,733 non-shockable).
- "A Robust Machine Learning Architecture for a Reliable ECG Rhythm Analysis during CPR" (2019) — [PubMed 31946270](https://pubmed.ncbi.nlm.nih.gov/31946270). Numbers were not extracted.
- Approaches for mechanical (piston) compressions that combine an adaptive filter with a CNN report Se 99% and Sp 95%. Multiclass (shockable / asystole / organized) accuracy is above 88% — [Overview, Electronics 2022;11(21):3593](https://mdpi.com/2079-9292/11/21/3593/html). This is a review; check the primary papers.
- CNN during CPR (Computing in Cardiology 2020): VF Se 89% and non-shockable Sp 91.7%, below AHA targets — [CinC 2020-045](https://www.cinc.org/archives/2020/pdf/CinC2020-045.pdf)
- A cascade skip-connection BiLSTM autoencoder removes CPR artifact before a shock-advice classifier, reporting overall Se 98.61% and Sp 99.49%. The CPR artifact data came from a manufacturer under agreement and cannot be redistributed — [IEEE OJCS 2026, doi:10.1109/ojcs.2026.3696482](https://doi.org/10.1109/ojcs.2026.3696482); [PMC13282725](https://pmc.ncbi.nlm.nih.gov/articles/PMC13282725/)
- Clinical benefit: Cheskes S, et al. "Perishock pause: an independent predictor of survival from out-of-hospital shockable cardiac arrest." Circulation 2011 (Resuscitation Outcomes Consortium, ROC; n=815) — [PubMed 21690495](https://pubmed.ncbi.nlm.nih.gov/21690495/). A pre-shock pause of ≥20 s vs <10 s gave adjusted OR 0.47 (95% CI 0.27-0.82) for survival to discharge. Each 5 s increase (up to 40 s) cut survival by 18%. Post-shock pause was not independently associated with survival.
- Brouwer TF, et al. Circulation 2015: each 5 s increase in the longest compression interruption was associated with lower odds of survival (OR 0.89) — [Amsterdam UMC PDF](https://pure.amsterdamumc.nl/ws/files/160786622/Association-between-chest-compression-interruptions-and-clinical-outcomes-of-ventricular-fibrillation-out-of-hospital-ca.pdf)
- A 2025-era systematic review and meta-analysis covers closed-loop and AI-guided CPR, including shock-advice and pulse-detection algorithms during compressions — [PMC13101195](https://pmc.ncbi.nlm.nih.gov/articles/PMC13101195/)

### Inferences
- **[background, unverified this session]** Zoll See-Thru CPR is an adaptive filter that uses the CPR-sensor (accelerometer) compression reference to display a filtered ECG for clinicians. Zoll documents say it is not used for automated shock advice. Zoll "Rapid Shock" (AED 3) and Philips "Quick Shock" are fast-analysis features **[manufacturer claims]**. Philips SMART CPR uses VF-waveform analysis to choose CPR-first or shock-first. Reference signals in the literature include transthoracic impedance (available from the pads at no extra cost), sternal accelerometer, and compression force/depth. Irusta/Aramendi (Univ. Basque Country) and Eilevstjønn/Eftestøl (Stavanger) are the key groups.
- For a belt-worn device, the transthoracic impedance channel already in the pads is the cheapest reference signal. A reference-free CNN (Lim 2026) avoids any extra sensor but has only been shown on 8 s windows at ≥0 dB SNR.
- The realistic design is a "decide during CPR if confident, otherwise ask for a short pause" policy. A device built this way would still need a pause in about 30% of analyses (cprINSIGHT data).

### Gaps
- Published accuracy figures for Zoll See-Thru/RapidShock and Philips SMART CPR in peer-reviewed sources were **not found** in this session.
- Head-to-head comparison of reference signals (impedance vs accelerometer vs force) for artifact removal was not extracted.

## 5. VF waveform analysis (AMSA, slope) for shock-outcome prediction

### Takeaway
AMSA (amplitude spectrum area) predicts whether a shock will succeed in observational OHCA data. The only human randomized trial (eBioMedicine 2023) was small, stopped early, and showed no benefit. Swine data are positive. The MOSAIC study (NCT04997980) is ongoing. AMSA is not yet evidence-based enough to gate shocks, but is reasonable as a logged research or display feature.

### Cited Findings
- In a retrospective analysis of 609 OHCA VF patients, AMSA before successful shocks was 15.6±0.6 mV·Hz vs 7.97±0.2 mV·Hz before failed shocks (p<0.0001) — [Resuscitation 2013, PubMed 24005007](https://pubmed.ncbi.nlm.nih.gov/24005007/)
- In a 1,617-patient validation, thresholds of 15.5 mV·Hz (predicts success) and 6.5 mV·Hz (predicts failure) were derived. Source: "Amplitude Spectrum Area to Guide Defibrillation," Circulation 2015 — [doi:10.1161/circulationaha.114.010989](https://www.ahajournals.org/doi/10.1161/circulationaha.114.010989)
- AMSA is associated with survival in out-of-hospital VF — [JACC 2014, doi:10.1016/j.jacc.2014.06.1196](https://www.jacc.org/doi/10.1016/j.jacc.2014.06.1196)
- AMSA trial (Ristagno et al., eBioMedicine 2023): a small, open-label, pseudo-randomized multicenter trial. AMSA ≥15.5 mV·Hz triggered early shock and AMSA <6.5 mV·Hz deferred the shock for CPR. It found no improvement in VF termination or ROSC, enrolled only 19 patients in the AMSA arm, and was stopped early because of COVID-19 — [eBioMedicine](https://www.thelancet.com/journals/ebiom/article/PIIS2352-3964(23)00109-3/fulltext)
- Swine: real-time AMSA-guided shock timing was more effective and reduced the number of shocks — [J Am Heart Assoc 2017, PMC5721767](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5721767/)
- MOSAIC (AMSA plus amiodarone, OHCA) protocol, ongoing — [NCT04997980](https://clinicaltrials.gov/study/NCT04997980); [protocol PMC10226588](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10226588/)
- AMSA can be estimated in real time during compressions with a 1D CNN — [PMC10157479](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10157479/)
- A public Brescia OHCA dataset holds 260 patients' 5 s pre-shock VF segments, recorded with and without compressions, with shock outcome. Its license was not confirmed — [ResearchGate: "ECG waveform dataset for predicting defibrillation outcome in OHCA patients"](https://www.researchgate.net/publication/347780865_ECG_waveform_dataset_for_predicting_defibrillation_outcome_in_out-of-hospital_cardiac_arrested_patients); SVM on similar data — [PMC12308557](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12308557/)

### Inferences
- Guidelines continue to recommend shocking any shockable rhythm. An open device should not withhold a shock on the basis of AMSA. Computing and logging AMSA, and optionally displaying it to clinicians, is low-risk and supports future research.
- No evidence was found that supports AMSA-guided energy selection.

### Gaps
- Median-slope and other waveform measures were not separately verified. MOSAIC results were **not found** and may not be published yet.

## 6. Datasets

### Takeaway
The open data is PhysioNet Holter and monitoring material: MIT-BIH Arrhythmia, VFDB, CUDB, SDDB, and NSTDB for noise. No openly licensed dataset of real AED/defibrillator recordings with CPR artifact was found. Every high-quality OHCA-during-CPR study uses private, agreement-bound EMS or manufacturer data. The AHA DB is licensed through ECRI.

### Cited Findings
- Database sizes as used by Amann 2005 — [PMC1283146](https://pmc.ncbi.nlm.nih.gov/articles/PMC1283146/):
  - MIT-BIH Arrhythmia: 48 records, 2 channels, about 1,805 s each
  - CUDB: 35 records, 1 channel, about 508 s each
  - AHA DB series 7001-8210: 40 files, 2 channels, 1,800 s each
- CPR-artifact removal studies build their test signals synthetically: clean rhythms from PhysioNet (CUDB, VFDB, SDDB, AFDB) plus separately recorded artifact. Real CPR artifact data came from a manufacturer under an agreement that forbids redistribution — [IEEE OJCS 2026 / PMC13282725](https://pmc.ncbi.nlm.nih.gov/articles/PMC13282725/)
- Few groups use real OHCA-during-CPR recordings, because of privacy and the lack of public records — [Jekova & Krasteva 2021](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8232133/)
- The Lim et al. 2026 cohort of 464 subjects was provided by a manufacturer — [arXiv 2611.26393](https://arxiv.org/abs/2611.26393)
- I-CARE (PhysioNet) holds post-arrest comatose EEG/clinical data and is not suitable for shock-advice work — [PhysioNet I-CARE v2.1](https://physionet.org/content/i-care/2.1/)
- Brescia OHCA VF pre-shock dataset (260 patients) — [ResearchGate](https://www.researchgate.net/publication/347780865_ECG_waveform_dataset_for_predicting_defibrillation_outcome_in_out-of-hospital_cardiac_arrested_patients)

### Inferences
- **[background, unverified this session]** Further database details:
  - VFDB (MIT-BIH Malignant Ventricular Ectopy): 22 half-hour 2-channel records at 250 Hz.
  - CUDB: 8.5 min at 250 Hz.
  - MIT-BIH Arrhythmia: 360 Hz.
  - SDDB: 23 Holter recordings, including VF onset.
  - NSTDB: electrode motion, baseline wander and muscle noise.
  - PhysioNet "open access" databases are mostly under ODC-By or similar permissive terms. Check each one's page.
  - The AHA DB is sold by ECRI and cannot be redistributed.
  - Pediatric arrest ECG: no open dataset known.
- Practical path: train and design on public databases, then get a data-sharing agreement with an EMS system or academic group (Univ. Basque Country / Stavanger / Oslo, Amsterdam ARREST, ROC) for patient-disjoint OHCA validation, since no open substitute exists. This is the single biggest bottleneck for an open-source AED.

### Gaps
- No open AED recording dataset with CPR artifact was **found**. Licensing of the Brescia dataset was not confirmed.

## 7. Pediatric rhythm analysis

### Takeaway
Adult-derived algorithms tested on pediatric rhythm libraries in the early 2000s showed high specificity, around 99.5-100%, including on fast SVT. VF sensitivity was 96-99%. Little newer pediatric-specific validation was found for 2018-2026.

### Cited Findings
- Cecchin F, et al. "Is arrhythmia detection by automatic external defibrillator accurate for children? Sensitivity and specificity of an AED algorithm in 696 pediatric arrhythmias." Circulation 2001;103:2483, doi:10.1161/01.cir.103.20.2483 — [AHA Journals](https://www.ahajournals.org/doi/10.1161/01.cir.103.20.2483). The device was a Heartstream/Philips FR2. Non-shockable Sp was 100% and VF Se 96%.
- Atkinson E, et al. "Specificity and sensitivity of AED rhythm analysis in infants and children." 2003 (LIFEPAK 500, adult algorithm) — [PubMed 12883506](https://pubmed.ncbi.nlm.nih.gov/12883506/). Coarse VF: 72/73 (99%). Non-shockable: 1,465/1,472 (99.5%) on 15 s samples.
- "Sensitivity and specificity of an AED algorithm designed for pediatric patients," 2007 (n=198 with pediatric-specific criteria, including SVT) — [PubMed 17765384](https://pubmed.ncbi.nlm.nih.gov/17765384/). High Se for coarse VF and rapid VT, and high Sp including SVT; exact figures were not extracted.
- Using spectral parameters to separate SVT from VT gave Sp 0.91 at Se 0.95 in validation — [ResearchGate](https://www.researchgate.net/publication/224371036_An_algorithm_to_discriminate_SVT_from_VT_in_pediatric_AED_based_on_spectral_parameters)

### Inferences
- The main pediatric risk is a rapid-VT rate threshold that infant SVT (often >220-250 bpm) can exceed. A pediatric mode, triggered by pediatric pads or a switch, should use pediatric-specific rate and morphology criteria. It also needs a separate test set, since no open pediatric dataset was found.

### Gaps
- No 2018-2026 pediatric deep-learning shock-advice validation was **found**.

## 8. Other intelligent features (CPR feedback, pulse detection, connectivity, self-test, prompting, recording)

### Takeaway
Within this session's budget, only pause-reduction evidence was verified (see section 4). The other features below are noted with background knowledge flagged and need a follow-up researcher.

### Cited Findings
- The AI-guided CPR systematic review covers pulse-detection algorithms during compressions and real-time physiological-signal CPR guidance — [PMC13101195](https://pmc.ncbi.nlm.nih.gov/articles/PMC13101195/)
- One study used 4 s accelerometry-plus-ECG snippets from compression pauses to predict circulatory state (pulse vs no pulse); the venue was not confirmed — [ResearchGate search result, via Electronics 2022 overview](https://mdpi.com/2079-9292/11/21/3593/html)
- Patents describe accelerometer-based compression depth and release feedback (Zoll) — [US10828232](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10828232)

### Inferences
- **[background, unverified this session]**
  - CPR feedback: the ROC PRIMED cluster RCT (Hostler D et al., BMJ 2011;342:d512, n≈1,586) found real-time audiovisual feedback improved CPR metrics but not ROSC or survival. ILCOR 2020 gives a weak recommendation for real-time feedback devices.
  - Pulse detection: Elola A, et al. "Deep neural networks for ECG-based pulse detection during OHCA" (Entropy 2019) used ECG plus thoracic impedance. Ruiz de Gauna/Aramendi used the impedance circulation component.
  - Dispatch-integrated AEDs: Avive Connect (LTE + GPS, "4 Minute City", FDA-approved 2022) and PulsePoint Respond / Verified Responder exist, but outcome evidence is observational.
  - Event recording: no single open standard. The major vendors use proprietary formats (Zoll .zol/.ful, Philips, Stryker CODE-STAT). An open device could use EDF+/WFDB and an annotated event log.

  Verify all of these before use.

### Gaps
- The following were **not researched in this session**:
  - Measured outcome effects of CPR feedback
  - Accuracy of impedance cardiography, PPG and accelerometer pulse detection
  - Post-shock rhythm-assessment algorithms
  - Avive/PulsePoint outcome data
  - LTE-M/GPS implementation evidence
  - Self-test standards (IEC 60601-2-4 has self-test and readiness-indicator clauses)
  - Voice-prompt human-factors studies
  - Event-recording formats

  This section needs a follow-up researcher.

## 9. Trained-responder vs lay-public device, and software safety (IEC 62304 Class C, Rust, formal methods, recalls)

### Takeaway
Software-caused AED recalls are real and severe. The Defibtech DDU-100 Class I recall (2011, 65,885 US units) was for software that could cancel a shock. Shock-advice firmware is IEC 62304 Class C. Rust and formal-methods evidence for medical firmware was not researched in this session.

### Cited Findings
- Defibtech DDU-100 series, 2011: an FDA Class I recall of 65,885 US units. Units with software 2.004 or earlier could cancel a shock, and the fix was a software upgrade — [OHS Online](https://ohsonline.com/Articles/2011/05/03/Defibtech-Recalling-65000-US-AEDs.aspx); [FDANews: "Defibtech recalls 42,000 AEDs citing software glitch"](https://fdanews.com/articles/90357-defibtech-recalls-42-000-aeds-citing-software-glitch). Source counts differ (42,000 vs 65,885), possibly separate recall events.
- Defibtech Sentry/ReviveR, 2007: a software design issue let the self-test clear a low-battery warning. The recall was terminated in 2021 — [FDA recall database res.cfm?id=50470](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfRES/res.cfm?ID=50470); [HealthDay](https://www.healthday.com/healthpro-news/public-health/fda-defibtech-lifeline-and-reviver-defibrillators-recalled-652858.html)
- Welch Allyn AED 10, 2009: a Class I recall requiring a software upgrade from ≤2.02 to 2.06.02 — [FDA recall database](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfRes/res.cfm?id=50474)
- 2019-2023 AED recalls found in this search were Class II and not software-caused. LIFEPAK 500 (2020) had connector wear that stopped the device recognising a patient was connected. HeartSine Samaritan PAD (2023) shipped with the wrong language configuration — [FDA recall database](https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfRES/res.cfm?id=180255)

### Inferences
- These recalls show that the failures that matter are not only misclassification. They include a cancelled shock, a self-test that masks a fault, and a wrong-language configuration. The hazard analysis should cover the whole flow from charging to delivering the shock, plus readiness reporting, not just the classifier.
- **[background, unverified this session]**
  - IEC 62304 Class C requires:
    - detailed design down to software units
    - unit verification with acceptance criteria
    - integration and system testing
    - SOUP (software of unknown provenance) risk analysis
    - full traceability to ISO 14971 risk controls
  - FDA's 2015 final order moved AEDs to PMA (Class III).
  - Trained-responder features seen in professional AEDs (Zoll AED Pro, LIFEPAK 1000, Philips FR3) are an ECG display, manual override / manual mode, configurable prompts and CPR intervals, shock counter and elapsed time, and rhythm-strip export.

  Verify before use.

### Gaps
- The following were **not found or not researched in this session**:
  - Published use of Rust or formal methods in AED or defibrillator firmware
  - FDA MAUDE analyses of AED software failures after 2015
  - Human-factors evidence on prompting for professional users
