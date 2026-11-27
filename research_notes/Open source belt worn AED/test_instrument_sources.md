# Test instrument: working notes and sources (27 Nov 2026)

Working notes behind [docs/TEST_INSTRUMENT.md](../../docs/TEST_INSTRUMENT.md). About 25 searches and fetches. Tags:

- **[CALC]** my arithmetic, working shown in TEST_INSTRUMENT.md.
- **[EST]** engineering estimate; the basis is stated.
- **[UNVERIFIED]** seen only in a secondary source, a reseller listing or a search summary, or not checked at all.
- Prices are reseller **asking prices** seen on 27 Nov 2026, not quotes or sale prices. They move.

**Repository rule (CLAUDE.md rule 2, D-10).** Nothing here is an instruction to build or power a circuit above 50 V. Component families are listed as research input for a future design review, not as a parts list.

## 1. Commercial defibrillator analysers

### Fluke Biomedical Impulse 6000D / 7000DP (primary source read in full)

From the manufacturer's manual supplement, issue 5 (4/13), "Defibrillator Analyzer Specifications", [7000dp_umeng0105.pdf](https://www.flukebiomedical.com/sites/default/files/resources/7000dp_umeng0105.pdf). Read as text after download.

| Item | Figure |
|---|---|
| Waveshapes | Lown, Edmark, trapezoidal, DC biphasic, AC pulsed biphasic |
| Energy range | 0.1 to 600 J, autoranged |
| Energy accuracy | ±(1 % of reading + 0.1 J) from 0.1 to 360 J; same, "typical", from 360 to 600 J |
| Energy accuracy, *pulsed* biphasic (chopped waveform, not BTE) | ±(1.5 % of reading + 0.3 J) |
| Load | 50 Ω, ±1 %, non-inductive (< 2 µH; the "µ" is lost in the PDF text, the brochure says µH) |
| Pulse trigger level | 20 V |
| Pulse width | 1.0 to 50.0 ms, ±0.1 ms |
| Voltage | 20 to 5,000 V, ±(1 % of reading + 2 V) |
| Current | 0.4 to 100.0 A, ±(1 % of reading + 0.1 A) |
| Tilt | 1 to 99 %, ±1 digit |
| Interphase delay | 0.1 to 9.9 ms, ±0.1 ms |
| Sample rate | 250 kHz (4 µs per sample) |
| Maximum average power | 12 W, "equivalent to 10 defib pulses of 360 J every 5 minutes" |
| Scope output | isolated; autorange 2000:1, 400:1, 80:1; waveform playback, 50 Ω, amplitude ±5 %; includes 0.4 ms before the detected pulse |
| Charge time measurement | 0.1 to 100 s, ±0.05 s typical |
| AED test ECG waves | normal sinus 10–300 BPM, VF coarse and fine, monomorphic VT 120–300 BPM, five polymorphic VT types, asystole |
| ECG on the defibrillator input load | Lead II amplitude, limited to ±4 mV |
| Caution | do not use adhesive pads on the analyser; use the maker's electrode adapters |

Older figures in `hv_safety_isolation.md` §8 (energy range 0.1–360 J in an older catalogue) are superseded by the supplement: 600 J range, accuracy specified to 360 J and "typical" above.

Impulse 7010 selectable load: 25–200 Ω in 25 Ω steps, 5,000 V maximum (from `hv_safety_isolation.md` §8; the 7010 datasheet URL returned 404 on 27 Nov 2026).

Prices [UNVERIFIED, asking prices]:
- Used 7000DP: about $2,500 to $5,500 (DOTmed and eBay listings; a refurbished unit at $5,499.99). Search summary of [DOTmed listing](https://www.dotmed.com/listing/defibrillator-tester/fluke-biomedical/impulse7000dp/5194857), [eBay](https://www.ebay.com/itm/277703214590).
- New 7000DP from a parts distributor: about $10,239; with the 7010 load about $13,536 ([PartsSource](https://www.partssource.com/parts/fluke-electronics-corp-biomedical-div/3481039)). One reseller: $6,490 marked down from $10,800 ([Sakomed](https://sakomed.com/?p=994)).
- New 6000D: about $3,400 to $7,400 ([PartsSource](https://www.partssource.com/parts/fluke-electronics-corp-biomedical-div/2811928); a TradeIndia listing).
- Used 7010 load: $850 (DOTmed) to $1,199 (eBay).

### Rigel (Seaward) UniPulse 400

- 50 Ω ±1 % non-inductive, 0–199.9 J, ±1 % of reading ±0.1 J (from `hv_safety_isolation.md` §8, [Seaward datasheet](https://seaward.com/us/support/medical/unipulse-400/manual/unipulse-400-datasheet-en)). Its 200 J ceiling is just above this project's 190 J stored adult charge and below many commercial manual defibrillators.
- New price $4,595 to $6,116 [UNVERIFIED]: [Relinkonline](https://relinkonline.com/products/rigel-medical-unipulse-400-defibrillator-analyzer), [TestEquity](https://www.testequity.com/product/20021574-UniPulse-400) and [TEquipment](https://www.tequipment.net/Rigel-Medical/UniPulse-400/Medical-Device-Safety-Testers/) at $4,810, [Zoro](https://www.zoro.com/rigel-medical-defibrillator-analyzer-with-pacer-unipulse-400/i/G114923063/) at $6,116. Rentals are listed by [Kwipped](https://www.kwipped.com/rentals/product/rigel-unipulse-400-defibrillator-analyzer/16653) (no rate seen).

### Datrend Phase 3

- 0–600 J and 0–50 J ranges, ±1 % ±2 least significant digits, from an unofficial machine-translated manual copy ([Scribd](https://www.scribd.com/document/719348464/Phase-3-Operators-Manual-1)) [UNVERIFIED].
- FDA 510(k) K062099: one fixed 50 Ω load; optional module for selectable loads 25–175 Ω; "laboratory use, outside the patient care vicinity" ([FDA K062099](https://www.accessdata.fda.gov/cdrh_docs/pdf6/K062099.pdf), via search summary).

### BC Group DA-2006 series

- Up to 1,000 J (cardioversion range); optional 25–200 Ω variable load (DA-2006-VL). Accuracy figure not retrieved ([datasheet](https://assets.testequity.com/te1/Documents/pdf/bc-group/BC-Group_DA-2006-Series_Datasheet_1125.pdf)).
- Price $3,291 ([TestEquity](https://testequity.com/product/443-594-DA-2006)) to $5,055 (PartsSource, Bluetooth version); DA-2006P (with pacer) $6,585 to $8,530 [UNVERIFIED].

### Calibration of a commercial analyser

- Fluke Biomedical quotes calibration on request; no published price. It offers ISO/IEC 17025 accredited calibration, but the accreditation page found ties the A2LA scope to its RaySafe lab, so whether the Impulse line is inside the accredited scope must be asked ([Fluke Biomedical calibration](https://flukebiomedical.com/services/calibration-repair-services)).
- An **example** third-party certificate for an Impulse 7000DP ([Fox Valley Metrology example certificate](https://www.foxvalleymetrology.com/wp-content/uploads/2026/02/Analyzer-Defibrillator-Calibration-Certification.pdf), marked "Example certificate") shows what is checked and how well:
  - Defibrillator load 50.000 Ω, limits 49.5–50.5 Ω.
  - Biphasic energy at 15, 50, 100 and 175 J, limits about ±1.5 % (e.g. 98.5–101.5 J at 100 J); expanded uncertainty (k = 2) 0.83 J at 15 J, 0.98 J at 50 J, 1.2 J at 100 J, 1.5 J at 175 J.
  - Monophasic energy at 10, 50, 100 and 360 J.
  - Scope output and ECG outputs.
  - 12-month recalibration interval; traceable to NIST; ISO/IEC 17025:2017.
  - **Finding:** the calibration lab's own uncertainty on energy is about ±1 % (k = 2). A commercial analyser is therefore a roughly ±1.5 % instrument in practice once its calibration uncertainty is included, not a ±1 % one.
- Calibration price: not found. [EST] a few hundred dollars; get a quote.

### Low-cost analyser prior art

- 2013 IEEE conference paper, "Design and construction the low-cost defibrillator analyzer" (Thailand): compared with an Impulse 6000D; ISO 17025 lab calibration gave ±1.2 J (95 %) average uncertainty; not open source ([IEEE Xplore 6687653](https://ieeexplore.ieee.org/document/6687653), via search summary) [UNVERIFIED detail].
- 2003 IEEE EMBS: a PC-based low-cost defibrillator analysis system measuring energy, peak voltage, peak current and pulse width (search summary only) [UNVERIFIED].
- The 2021 "low-cost, open source defibrillation analyzer for educational purposes" listed in `open_source_prior_art.md` Q7 was not found again in this pass. Still no validated open instrument known.

## 2. Measuring chain components

### Current transformer: Pearson Electronics model 110 (primary datasheet read)

From the [Pearson 110 datasheet](https://transientspecialists.com/cdn/shop/files/110_ds.pdf) (1999 edition):

| Item | Figure |
|---|---|
| Sensitivity | 0.1 V/A, +1 / −0 % |
| Output resistance | 50 Ω |
| Maximum peak current | 5,000 A |
| Maximum rms current | 65 A |
| Droop rate | 0.8 % per millisecond |
| Usable rise time | 20 ns |
| Current-time product | 0.5 A·s maximum with core-reset bias; 0.2 A·s typical without |
| 3 dB band | about 1 Hz to about 20 MHz |

Price: 110 at $595, 110A at $695 (one distributor, via search summary) [UNVERIFIED].

**Finding:** droop of 0.8 %/ms means the reading has sagged by about 7 % by the end of a 9 ms phase. It must be corrected in software (the droop is a known first-order effect), or a sensor that responds down to DC must be used.

### Coaxial current shunts

T&M Research / Powertek SDN series: resistance, bandwidth and energy limit are coupled; the smallest listed (SDN-001, 1 mΩ) is rated 12 J ([Powertek SDN](https://powertekuk.com/coaxial-shunt-series-sdn), search summary). The SDN-414 row was not retrieved. Tolerance appears to be chosen at order time [UNVERIFIED].

### High-voltage differential probes

- Tektronix THDP0100: 100 MHz; ±6,000 V differential at 1000×, ±600 V at 100×; new price about $4,740 to $5,680 (Transcat, TestEquity, Octopart, ValueTronics), higher at some distributors [UNVERIFIED].
- Tektronix THDP0200: 200 MHz; ±1,500 V differential at 500×. **Below the 1,950 V adult charge**, so not usable across the load for adult shocks.
- THDP series DC gain accuracy ±2 % (retailer page, via search) [UNVERIFIED against the Tektronix manual].
- Pico TA044: 70 MHz; CAT I per its manual; voltage range and accuracy not retrieved.

### Isolated or differential digitisers

- Tektronix TPS2000B: isolated, floating channels; float limit about 600 V to ground; CAT II 600 V (RS listing). New TPS2012B about $5,610 [UNVERIFIED]. 8-bit vertical resolution [BK].
- PicoScope 4444: four fully differential inputs, 12 or 14 bit, 20 MHz; 1,000 V CAT III kit about £1,749 ex VAT (RS) [UNVERIFIED]. DC accuracy not retrieved.

### Load resistors

**Ohmite 800 and 1000 series tubular bulk ceramic** (primary datasheet read, [Ohmite via Digi-Key](https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/7182/800-and-1000-series.pdf)):

| Type | Size (length × diameter) | Resistance | Avg power at 40 °C | Peak energy | Peak voltage |
|---|---|---|---|---|---|
| 885AS | 63.5 × 19.1 mm | 6–1,200 Ω | 15 W | 2,800 J | 8,000 V |
| 886SP | 127 × 19.1 mm | 1–330 Ω | 90 W | 500 J | 4,000 V |
| 886AS | 127 × 19.1 mm | 15–3,300 Ω | 30 W | 7,000 J | 20,000 V |
| 887AS | 152 × 25.4 mm | 12–3,300 Ω | 50 W | 13,000 J | 30,000 V |

- Peak energy and voltage "will depend on the resistance value. Consult factory." Energy ratings assume pulses **under 10 ms**; SP ratings can be greater for longer pulses.
- Tolerance options: **±10 % or ±20 %** only.
- Temperature coefficient: SP +0.2 to −0.08 %/°C; AS 0 to −0.08 %/°C (−800 ppm/°C).
- Voltage coefficient: up to **−1.0 % per kilovolt per inch of active length** (SP and AS).
- Short-time overload change ±5 % (SP), ±2 % (AS); load life ±5 %.
- Density 2.2–2.6 g/cm³; specific heat 0.23–0.26 cal/(g·°C) (about 1.0 J/(g·°C)).
- Operating temperature SP −55 to +350 °C, AS −55 to +230 °C.
- Bulk construction "inherently non-inductive".

**Finding:** bulk ceramic is the right class for energy and inductance, but it is **not a precision resistor**: ±10 % tolerance, resistance that falls as it heats, and resistance that changes with the voltage across it. An instrument built on it must measure current directly, not infer it from a nominal resistance.

**HVR Advanced Power Components disc and HV series** (search summaries of HVR datasheets, [HVR brochure via Hilltech](https://www.hilltech.com/pdf/Resistors/HVR_Product_brochure.pdf), [disc resistors](https://www.hilltech.com/pdf/Resistors/disc-resistors.pdf)) [UNVERIFIED detail]: TCR −0.05 to −0.15 %/°C; energy about 350 J/cm³ infrequent, 250 J/cm³ continuous; inductance "negligible (nH)".

Other families named in earlier notes (`hv_safety_isolation.md` §2): Ohmite EY (400 J), encapsulated SP (630 J) and AS/BA (5,200 J), Ohmite 500 series slabs; Vishay Dale Ayrton-Perry non-inductive wirewound (energy per ohm rules apply).

## 3. Reference AEDs

- Philips HeartStart OnSite (M5066A): new from Philips' store $1,622 to $1,841; refurbished $725 to $1,275 (Integris, AED Brands, MDMaxx, Alert First Aid); used $570 to $750 (DOTmed) [UNVERIFIED asking prices]. [Philips store](https://aedshop.philips.com/us/products/heartstart-onsite-aed.html), [Integris](https://integrisequipment.com/products/philips-onsite-aed-business-package-refurbished).
- AED rental rates seen: Zoll AED Plus from $95 per month (Kwipped) [UNVERIFIED].

## 4. IEC 60601-2-4 (not bought)

- Clause map from the official preview (in `regulatory_safety_legal.md`): 201.12 accuracy of controls and instruments (energy accuracy), 201.101 charging time, 201.105 recovery of monitor/ECG input after defibrillation (test arrangement Figures 201.105 and 201.107), 201.107 rhythm recognition detector, 201.108 electrodes, 201.4.2.101 readiness in the risk management file. 201.102.3.2 shock capacity (20 maximum-energy shocks within 1,300 s) from the Jewel FDA summary (`hv_safety_isolation.md` §4).
- A secondary case study lists delivered-energy accuracy tests at 25, 50, 75, 100, 125, 150 and 175 Ω, maximum energy at 25, 50 and 100 Ω, and other energies at 50 Ω (search summary of an [Atlantis Press paper](https://www.atlantis-press.com/article/25853886.pdf)) [UNVERIFIED].
- "±3 J or ±15 %" energy accuracy and the recovery-time limit remain [UNVERIFIED]. A third-party monitor manual says its display recovers within 10 s; that is a product claim, not the standard's limit.
- Edition 3.2 (2026-09) is current; its changes to 201.105 were not seen.

## Gaps

- Tektronix THDP0100 DC accuracy from the Tektronix manual.
- PicoScope 4444 DC accuracy.
- Pearson models with lower droop and their prices.
- Closed-loop Hall-effect current sensors (respond to DC, no droop): not researched in this pass.
- Precision high-voltage dividers able to check the D-28 voltage sensors (about ±0.3 % at 2 kV): not researched.
- Ohmite factory ratings for 25–250 Ω values and for pulses of 10–16 ms.
- Calibration prices for an analyser, a probe and a current transformer.
- Whether commercial analysers inject the ECG at a low-voltage tap of the load or by another method (not published in what was read).
