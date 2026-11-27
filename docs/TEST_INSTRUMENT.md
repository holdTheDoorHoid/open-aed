# The test instrument: an open defibrillator analyser

Version 0.1, 27 November 2026. A requirements-and-architecture document. Decision numbers (D-xx) refer to [DECISIONS.md](../DECISIONS.md); sources and working notes are in [research_notes/…/test_instrument_sources.md](../research_notes/Open%20source%20belt%20worn%20AED/test_instrument_sources.md).

> **Read this first.**
>
> - **Building any part of this instrument is Phase 2 work. It must not start before the lab safety policy required by [SAFETY.md](../SAFETY.md) has been written, reviewed by someone with professional high-voltage experience, and adopted (D-10, D-18).** The only part that can be done now is the analysis software (Stage 0), which touches no hardware.
> - **The test load is itself a high-voltage, high-energy apparatus.** It receives shocks of up to about 200 joules at about 2,000 volts, and stays connected to a device that may still be charged. Everything in SAFETY.md applies to it, exactly as it applies to the defibrillator.
> - This is a document of **requirements, measurement method, accuracy and plan**. It contains no schematic, no wiring, no board layout, no assembly steps and no build procedure, on purpose (CLAUDE.md rules 2 and 7, D-13). Where component families are named, they are named so that a future high-voltage engineer knows what class of part to evaluate, not as a parts list.
> - Nothing here is safe or ready for use on a person or animal. No hardware exists. Figures from the project's waveform model are **model** figures, not measurements.

How to read the tags: **[CALC]** is arithmetic with the working shown; **[EST]** is an engineering estimate with its basis stated; **[UNVERIFIED]** was seen only in a secondary source or not checked; anything else carries a source. Prices are reseller asking prices seen on 27 November 2026.

## Words used in this document

| Word | Meaning here |
|---|---|
| **Device under test (DUT)** | Whatever defibrillator is being measured: a commercial AED, or later this project's own prototype. |
| **Load** | The resistor that stands in for the patient's chest and absorbs the shock. Measured in ohms (Ω). |
| **Simulated patient** | The load plus a small signal source that makes the device "see" a heart rhythm and a pad resistance, so it will analyse and decide. |
| **Analyser** | The whole instrument: load, simulated patient, sensors, recorder and analysis software. |
| **Phase 1, phase 2, gap** | The shock is two pulses of opposite direction (biphasic) with a short pause (the interphase gap) between them. |
| **Tilt** | How much the current falls during a phase. 40 % tilt means it ends at 60 % of where it started. |
| **Peak current** | The largest current, at the very start of phase 1. |
| **Charge** | The quantity of electricity moved through the load (amperes × seconds, coulombs). |
| **Residual voltage** | What is left on the device's storage capacitor after the shock. The device dumps it internally. |
| **Sample rate** | How many readings per second the recorder takes (kS/s = thousands, MS/s = millions). |
| **Bandwidth** | How fast a change the sensor can follow. Too little bandwidth blurs the sharp edges of the pulse. |
| **Calibration** | Comparing an instrument with a better one whose accuracy is itself known, and recording the difference. |
| **Traceable** | A calibration chain that ends at a national standards laboratory (NIST in the US). |
| **Uncertainty** | An honest "plus or minus" on a measured number. "±1 % (k = 2)" means about 95 % confidence that the true value is inside ±1 %. |
| **Non-inductive** | A resistor that does not behave like a coil. Coils resist sudden changes of current and distort the pulse. |
| **Differential probe** | A probe that measures the voltage between two points without connecting either to earth. |
| **Current transformer** | A ring the current-carrying wire passes through; it gives a small voltage in proportion to the current, with no electrical contact. |
| **Shunt** | A small, precisely known resistor in the current path; the voltage across it shows the current. |
| **Droop** | A current transformer's reading slowly sags during a long pulse even when the true current is steady. |

---

## 1. Purpose, and what "verified" will mean

The project's waveform model ([`aed-waveform`](../firmware/crates/aed-waveform/DESIGN_TABLE.md)) and simulator ([`aed-sim`](../firmware/crates/aed-sim/README.md)) predict exactly what each shock should do. The analyser's job is to measure what a real shock *actually* did, with a stated uncertainty, so that the two can be compared number by number. The research found no validated open instrument of this kind ([open_source_prior_art.md](../research_notes/Open%20source%20belt%20worn%20AED/open_source_prior_art.md) Q7; [PLAN.md](PLAN.md), "What the research changed", point 5).

**"Verified" will mean:** for a stated quantity, the analyser's measurement, with its uncertainty, lies inside the requirement's tolerance, and the analyser itself has been shown fit for use by the checks in section 5. A measurement that is inside the tolerance only if its uncertainty is ignored is not "verified"; it is "indeterminate" (section 6.6).

The project has not bought IEC 60601-2-4, the defibrillator standard. Clause numbers below come from the standard's free preview and secondary sources as recorded in the research notes ([regulatory_safety_legal.md](../research_notes/Open%20source%20belt%20worn%20AED/regulatory_safety_legal.md), [hv_safety_isolation.md](../research_notes/Open%20source%20belt%20worn%20AED/hv_safety_isolation.md) §4). **All clause references are [UNVERIFIED]. Task T-1: confirm every test method, load value and tolerance below from the purchased standard (Edition 3.2, 2026-09) before any acceptance criterion is fixed.**

### 1.1 On every shock

| Quantity | What it checks | Model figure it is compared with | Standard clause area [UNVERIFIED] |
|---|---|---|---|
| Delivered energy, total | The dose: 150 J adult (D-03), 50 J child (D-04), within tolerance | `patient_energy_j`; design table, e.g. 150.0 J at 50 Ω | 201.12 accuracy of controls (energy accuracy); commonly cited as ±15 % or ±3 J [UNVERIFIED]; project placeholder ±15 % |
| Energy in phase 1 and phase 2 | That the split between phases matches the design | `patient_energy_phase1_j`, `patient_energy_phase2_j` | Not separately identified |
| Peak current | Sizes the output switches; Philips argues peak current matters most | `peak_current_a`: 65.0 A at 25 Ω, 35.5 A at 50 Ω, 10.8 A at 175 Ω (adult) | Not separately identified |
| Current at the end of each phase | The truncation happened where planned | `phase1_end_current_a`, `phase2_end_current_a` | — |
| Phase 1 and phase 2 durations | The resistance compensation chose the right lengths | `phase1_s`, `phase2_s`: 3.0 to 9.0 ms and 2.0 to 6.0 ms (adult) | — |
| Interphase gap | Switch timing | `interphase_s`: 0.5 ms (assumption) | — |
| Tilt, each phase | Shape of the waveform against Philips' published shape (D-02, PLAN Phase 3) | `phase1_tilt`, `phase2_tilt`: 39–63 % (adult phase 1) | — |
| Polarity | Phase 1 goes the intended way between the two pads | Sign convention in section 6.2 | — |
| Charge delivered, each phase | Independent cross-check of the current measurement against the capacitor (section 5.4) | `phase1_charge_c`, `phase2_charge_c` | — |
| Voltage across the load | Peak and end-of-phase voltages | Model current × load resistance | — |
| Residual capacitor voltage (only where accessible: the project's own prototype, never a commercial AED) | Energy book-keeping and the internal dump | `final_capacitor_v`: up to about 846 V adult at 175 Ω [CALC: √(2 × 35.8 J / 100 µF)] | — |
| Charge time | Charger performance; D-29 needs the charge to finish inside the analysis (about 7 s) | `ShockTiming.charge_ms` in `aed-sim` (about 6.1 s adult, model) | 201.101 charging time; commonly ≤15 s [UNVERIFIED] |
| Time from shock request to delivery | The supervisor's permit and the bridge's response | `ShockTiming.permit_wait_ms` | — |
| Time from analysis start to delivery, and hands-off time | Rescue timing (D-29, D-30) | `ShockTiming.analysis_ms`, `countdown_ms`, `hands_off_ms` | AED timing limits, commonly ≤30 s from analysis start to ready [UNVERIFIED] |
| Rhythm at the moment of delivery | Finding F-4 (late rhythm change) measured on hardware rather than in simulation | `ShockRecord.rhythm`, `rhythm_since_ms` | 201.107 rhythm recognition detector |

### 1.2 Around the shock

| Quantity | What it checks | Reference | Standard clause area [UNVERIFIED] |
|---|---|---|---|
| ECG input recovery time after the device's own shock | The device can analyse again soon after a shock (PLAN Phase 4) | `aed-sim` blanks the ECG for 1.5 s (an assumption, not a requirement) | 201.105 recovery after defibrillation (test arrangement Figures 201.105/201.107); IEC 60601-1 8.5.5 defibrillation-proof applied parts |
| Pad-resistance reading accuracy at 25, 50, 75, 100, 125, 150, 175 Ω | The planner gets the right resistance, so the energy lands on target | The known load value | Not identified |
| Energy accuracy across 25–175 Ω, both modes | The whole compensation scheme | Design table rows | 201.12; a secondary source lists tests at 25, 50, 75, 100, 125, 150 and 175 Ω [UNVERIFIED] |
| Out of range: 190 Ω and 250 Ω | No shock; "press pads firmly" prompt (D-32) | `pads-190-throughout` scenario | Not identified |
| Below range and near-short | No shock | `PlanError::ImpedanceTooLow` | Not identified |
| Open circuit (pads not connected) | No shock; "attach pads" | `PlanError::ImpedanceTooHigh` | Not identified |
| Self-test discharge (D-15) | Full-energy charge and internal dump; **nothing** appears at the pads | `aed-core` self-test | 201.4.2.101 readiness (risk management file) |
| Cancelled shock | No energy at the load; capacitor emptied internally within the limit (D-33) | `cancel-during-countdown` scenario | Not identified |
| Repeated shocks | Energy stays in tolerance over a sequence | — | 201.102.3.2: 20 maximum-energy shocks within 1,300 s ([hv_safety_isolation.md](../research_notes/Open%20source%20belt%20worn%20AED/hv_safety_isolation.md) §4, from the Jewel FDA summary) |
| Device voltage sensor accuracy (prototype only) | D-28: each sensor within about 1 % + 7 V | Capacitor voltage measured independently | — |

---

## 2. The measurement method, and why

### 2.1 The method

Measure two things at the same instant, many times a second, for the whole shock:

- the **voltage across the load**, and
- the **current through the load**.

Power at each instant is voltage × current. Adding up power × time over the pulse gives the energy delivered (numerical integration). Adding up current × time gives the charge. Durations, tilt and peaks come from the shape of the recorded current.

This is how the commercial analysers and the defibrillation-proof tests in IEC 60601-1 work: the MEDTEQ description integrates v²/R across the load ([hv_safety_isolation.md](../research_notes/Open%20source%20belt%20worn%20AED/hv_safety_isolation.md) §1, §8). The Fluke Impulse samples at 250 kHz ([Fluke manual supplement](https://www.flukebiomedical.com/sites/default/files/resources/7000dp_umeng0105.pdf)).

**Why measure current directly instead of only voltage.** With a perfect resistor, voltage alone is enough: energy = ∫v²/R dt. But the resistors that can absorb these pulses are not precise (section 3.3): the bulk-ceramic family is sold at ±10 % or ±20 %, loses resistance as it heats, and changes resistance with the voltage across it. Measuring current as well means the energy figure does not depend on knowing the resistance during the pulse. The software also computes the voltage-only and current-only versions and checks that all three agree (section 6.4); disagreement is a warning that something in the chain is wrong.

### 2.2 How fast to sample, and how much bandwidth

The pulse lasts 5.5 to 15.5 ms in total (design table), but its edges are fast: the output switches turn on and off in roughly a microsecond [EST: typical for the switch classes in [hv_safety_isolation.md](../research_notes/Open%20source%20belt%20worn%20AED/hv_safety_isolation.md) §2]. Energy is lost from the sum if the edge falls between two samples. An upper bound on that error, per edge, is half a sample interval times the power at that edge [CALC].

| Load, adult | Power at the start of phase 1 [CALC: I²R, I = 1,950 V / (R + 5 Ω)] | Worst edge error at 100 kS/s | at 250 kS/s | at 1 MS/s |
|---|---:|---:|---:|---:|
| 25 Ω | 105.6 kW | 0.53 J (0.35 %) | 0.21 J (0.14 %) | 0.05 J (0.04 %) |
| 50 Ω | 62.9 kW | 0.31 J (0.21 %) | 0.13 J (0.08 %) | 0.03 J (0.02 %) |
| 175 Ω | 20.5 kW | 0.10 J (0.07 %) | 0.04 J (0.03 %) | 0.01 J (0.01 %) |

[CALC: error ≤ P × Δt/2; e.g. 105.6 kW × 5 µs = 0.53 J. The end-of-phase edges carry much less power and add little. Interpolating between samples, as the software will, does better than this bound.]

Timing has its own need. The 0.5 ms gap should be checkable to about ±10 % (±50 µs), and phase durations to better than ±1 % of 2 ms (±20 µs). One sample interval at 100 kS/s is 10 µs; at 1 MS/s it is 1 µs.

**Requirement R-ACQ-1.** Sample rate at least **250 kS/s** for development use and at least **1 MS/s** for claims; both channels sampled at the same instants (or with a known, corrected time offset).

**Bandwidth.** A sensor that can only follow slow changes behaves like a short delay plus a blur. A single-pole filter with cut-off frequency f delays by about 1/(2πf): 1.6 µs at 100 kHz, 0.16 µs at 1 MHz. Its energy error is roughly the edge power times that delay: about 0.17 J (0.11 %) at 100 kHz and 0.02 J at 1 MHz for the worst adult edge [CALC: 105.6 kW × 1.6 µs].

**Requirement R-ACQ-2.** Analogue bandwidth of each channel at least **1 MHz**, matched between the two channels. If the sensors' bandwidth is much higher than half the sample rate (the Pearson 110 current transformer is about 20 MHz), either add a filter that removes what the recorder cannot sample, or record fast enough to see the ringing (tens of MS/s) and let the software reduce it. Otherwise ringing at the edges "folds" into false readings of peak current.

**Requirement R-ACQ-3.** Resolution at least **12 bits** (development) and **14 bits** (claims) over a range set so that the largest expected signal uses more than half the range and nothing clips. Record at least 2 ms before the pulse (to measure the zero level) and at least 5 ms after it.

### 2.3 Why inductance matters

A resistor wound like a coil (ordinary wirewound) has inductance. When the device cuts the current off at the end of a phase, an inductance forces a voltage spike of L × (rate of change of current). For 50 µH and 65 A switched off in 1 µs that spike would be about 3,250 V [CALC: 50e-6 × 65 / 1e-6], on top of the 1,950 V the switch already holds. That can damage the device under test and gives a waveform no patient would produce. Commercial analysers specify less than 2 µH ([Fluke manual supplement](https://www.flukebiomedical.com/sites/default/files/resources/7000dp_umeng0105.pdf)); with 2 µH the stored magnetic energy at 65 A is only 4 mJ [CALC: ½ × 2e-6 × 65²], negligible for energy, and the spike is about 130 V [CALC].

**Requirement R-LOAD-1.** Load inductance below **2 µH** at every step, including its connections, matching the commercial analyser criterion.

### 2.4 Why resistor heating and voltage coefficient matter

The bulk-ceramic resistors that suit the energy (section 3.3) have, per the maker's datasheet ([Ohmite 800/1000 series](https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/7182/800-and-1000-series.pdf)):

- resistance that changes by up to −0.08 %/°C (type AS) or between +0.2 and −0.08 %/°C (type SP);
- resistance that falls by up to 1 % per kilovolt per inch of active length.

For one tube about 127 mm long carrying a whole 1,950 V adult shock: 1,950 V / 5 in ≈ 0.39 kV per inch, so up to about −0.4 % lower resistance at the start of the shock than at the end [CALC]. For heating, see section 3.4: about +2.4 °C per 150 J shock in a single such tube, so up to −0.2 % per shock and −3.8 % after 20 shocks with no cooling [CALC].

So "50 Ω" is a nominal label; during the pulse the load is perhaps 49.6–50 Ω, and later in a sequence lower again. With direct current measurement this does not corrupt the energy. It does matter for:

- **comparing with the model**, which must use the resistance measured just before that shock, not the nominal one; and
- **the device's own planning**, which measured its pad resistance before the shock with a small signal. The analyser should record the load resistance cold, before each shock, and the effective resistance during it (v ÷ i), so the difference can be seen.

### 2.5 Error budget: how ±1 % on energy is built up

Commercial analysers claim energy accuracy of ±1 % of reading plus 0.1 J (Fluke Impulse 7000DP, up to 360 J; Rigel UniPulse 400 ±1 % ± 0.1 J). That is ±1.6 J at 150 J and ±0.6 J at 50 J [CALC]. An example calibration certificate for an Impulse 7000DP shows the calibration lab's own expanded uncertainty is about 1.2 J at 100 J (k = 2) ([example certificate](https://www.foxvalleymetrology.com/wp-content/uploads/2026/02/Analyzer-Defibrillator-Calibration-Certification.pdf)). So in practice a commercial analyser is roughly a **±1.5 %** instrument once its own calibration is counted [EST: the specification and the lab's uncertainty combined].

For the project's own chain (v × i method), the budget below shows what ±1 % (k = 2) requires. Each line is a standard uncertainty (one standard deviation) in per cent of energy; they combine as the square root of the sum of squares.

| Contribution | How it enters the energy | "Claims" tier: what is needed | Value | "Development" tier: off-the-shelf parts, uncalibrated as a system | Value |
|---|---|---|---:|---|---:|
| Voltage channel gain (probe or divider × recorder) | Directly, 1:1 | Calibrated as a chain to ±0.25 % | 0.25 % | HV differential probe ±2 % [UNVERIFIED THDP spec] and recorder ±1 % [EST], taken as limits | 1.29 % |
| Current channel gain (transformer or shunt × recorder) | Directly, 1:1 | Calibrated to ±0.25 % | 0.25 % | Pearson 110 +1/−0 %, droop correction residual 0.2 %, recorder ±1 % [EST] | 0.68 % |
| Current transformer droop (0.8 %/ms, Pearson 110) | Up to about 7 % low by the end of a 9 ms phase if uncorrected [CALC: 0.8 × 9.0] | Corrected in software from the calibrated droop time constant | 0.1 % | Corrected | (in line above) |
| Zero offset and baseline drift | Larger where the signal is small (end of phase 2) | Pre- and post-pulse baseline subtraction | 0.1 % | Same | 0.1 % |
| Time offset (skew) between the two channels | Mostly at the edges: about edge power × skew; 105.6 kW × 1 µs = 0.1 J [CALC] | Measured and corrected to ±0.1 µs | 0.07 % | Corrected | (negligible) |
| Edge sampling (section 2.2) | At 1 MS/s ≤ 0.04 % | 1 MS/s | 0.05 % | 250 kS/s | (≤0.14 % bound, inside the totals) |
| Sample clock error | 1:1 with time; recorder clocks are typically ±50 ppm [EST] = 0.005 % | — | 0.005 % | — | 0.005 % |
| Quantisation | Random and averaged over thousands of samples | 14-bit | <0.01 % | 12-bit | <0.02 % |
| Load resistance value and drift | **Does not enter** the v × i result; enters only the v²/R cross-check | Measured cold before each shock with a 4-wire meter | — | Same | — |
| **Combined, k = 1** | | | **0.39 %** | | **1.46 %** |
| **Expanded, k = 2 (about 95 %)** | | | **±0.8 %** | | **±2.9 %** |

[CALC: claims √(0.25² + 0.25² + 0.1² + 0.05² + 0.1² + 0.07² + 0.05²) = 0.39 %, × 2 = 0.79 %. Development: voltage √(1.15² + 0.58²) = 1.29 %; current √(0.29² + 0.2² + 0.58²) = 0.68 %; total √(1.29² + 0.68² + 0.1²) = 1.46 %, × 2 = 2.9 %. A ±a % limit is converted to a standard uncertainty by dividing by √3.]

Peak current and timing are easier than energy: peak current depends only on the current channel (about ±1.4 % development [CALC: 2 × 0.68 %] and ±0.5 % claims [CALC: 2 × 0.25 %], k = 2); durations depend on the sample interval and edge shape (about ±2 µs at 1 MS/s [EST: one sample plus interpolation error]). The commercial analyser is weaker on timing than on energy: ±0.1 ms on pulse width and gap, which is ±20 % of a 0.5 ms gap. **The project's own capture, at 1 MS/s, can therefore beat the commercial analyser on timing** even when it only matches it on energy.

### 2.6 What accuracy the project actually needs

A common metrology rule is that the measuring instrument should be about four times more accurate than the tolerance it checks (a "4:1 ratio") [EST: general metrology practice; ANSI/NCSL Z540.3 frames this as a limit on false acceptance].

| What is checked | Tolerance | Needed at 4:1 | Development tier gives | Claims tier gives |
|---|---|---|---|---|
| Delivered energy (placeholder, [DESIGN_TABLE.md](../firmware/crates/aed-waveform/DESIGN_TABLE.md)) | ±15 % (placeholder, [UNVERIFIED]) | ±3.75 % | ±2.9 % ✓ | ±0.8 % ✓ |
| Energy near the edge of tolerance (model worst case: child +14.2 % at 25 Ω, 0.8 points inside, finding F-9) | 0.8 percentage points of margin | ±0.8 % or better to decide | ✗ (answer "indeterminate") | borderline |
| Device voltage sensors (D-28), at 1,950 V | 1 % + 7 V = ±26.5 V [CALC] | ±6.6 V, about ±0.34 % [CALC: 26.5 / 4] | ✗ | needs a precision DC high-voltage reference, not the pulse chain |
| Phase durations against plan | not yet set [EST: a few per cent] | tens of µs | ✓ at ≥ 250 kS/s | ✓ |
| Charge time (D-29: within the ~7 s analysis) | seconds | ±0.05 s | ✓ | ✓ |

**Two tiers, stated plainly:**

- **"Good enough for development" — about ±3 % on energy (k = 2), ±0.1 ms timing or better.** Enough to tune the hardware, see whether shocks land near target, and catch gross faults. Results near a limit come out as "indeterminate", not "pass". Reachable with a commercial analyser alone (Stage 1), or with an off-the-shelf probe chain (Stage 2) before system calibration.
- **"Good enough to make claims" — about ±1 % on energy (k = 2), traceable, with ±2 µs timing.** Needed for any statement that will go into a design history file for a regulated partner (D-16), and to decide shocks within a point or two of the tolerance. Requires each channel calibrated by an accredited lab and the whole chain cross-checked against a calibrated commercial analyser (section 5).

**The most demanding single measurement is not energy but the capacitor voltage check for D-28** (±0.34 % at 2 kV). That is a slow, steady (DC) measurement on the project's own prototype, done with a precision high-voltage divider of known ratio rather than with the pulse-capture chain. It is listed as its own requirement (R-CAP-1) and as an open question (Q-TI-7).

---

## 3. The load and the simulated patient

### 3.1 Resistance values

| Step | Purpose | Expected device behaviour |
|---|---|---|
| 25, 50, 75, 100, 125, 150, 175 Ω | The shock range (REQUIREMENTS.md); the test points a secondary source attributes to the standard [UNVERIFIED] | Shock, energy within tolerance |
| 190 Ω | Just above range (finding F-8, D-32) | Analyse; "press pads firmly"; **no shock** |
| 250 Ω | Top of the pad-sensing range (D-32) | Same |
| About 20 Ω | Just below range | **No shock** |
| Near-short, a few ohms (never a bare wire) | Pads touching or shorted | **No shock** |
| Open | Pads not connected | "Attach pads"; **no shock** |

Commercial selectable loads stop at the shock range (Fluke 7010: 25–200 Ω in 25 Ω steps, 5,000 V; Datrend: 25–175 Ω; BC Group: 25–200 Ω), so the 190 Ω, 250 Ω, sub-range and near-short cases need the project's own load or extra parts.

**Every step, including the "no shock" steps, must be rated as if the device fired into it anyway**, because the point of those tests is to catch a device that does.

### 3.2 Energy and power each step must absorb

**Per shock (adult, model)** [CALC from the design table: I₀ = 1,950 V / (R + 5 Ω), P = I₀²R; energy from the table]:

| Step | Delivered energy (model) | Peak current | Peak power into the load | Charge, phase 1 + phase 2 |
|---|---:|---:|---:|---:|
| 25 Ω | 152.8 J | 65.0 A | 105.6 kW | 0.123 + 0.035 A·s |
| 50 Ω | 150.0 J | 35.5 A | 62.9 kW | 0.089 + 0.035 A·s |
| 75 Ω | 150.0 J | 24.4 A | 44.6 kW | 0.083 + 0.035 A·s |
| 100 Ω | 150.0 J | 18.6 A | 34.5 kW | 0.080 + 0.034 A·s |
| 125 Ω | 150.0 J | 15.0 A | 28.1 kW | 0.079 + 0.034 A·s |
| 150 Ω | 150.0 J | 12.6 A | 23.7 kW | 0.078 + 0.034 A·s |
| 175 Ω | 150.0 J | 10.8 A | 20.5 kW | 0.077 + 0.034 A·s |
| 190 Ω (fault: fired anyway at 2,000 V) | up to 200 J stored | 10.3 A | 20.0 kW | — |
| 250 Ω (same) | up to 200 J | 7.8 A | 15.4 kW | — |
| Near-short, 2 Ω (same) | up to 200 J | 286 A | 163 kW | — |

Child mode peaks are lower: 38.3 A and 36.7 kW at 25 Ω [CALC].

The near-short row is the reason a "short" test must use a rated low-value resistor: with only the device's assumed 5 Ω internal resistance in the way, a true short at 1,950 V would draw about 390 A [CALC: 1,950 / 5]. That is near the 500 A limit EFCOG sets for discharge sticks because of magnetic forces on cables ([hv_safety_isolation.md](../research_notes/Open%20source%20belt%20worn%20AED/hv_safety_isolation.md) §7).

**Commercial devices.** If the load will also be used with commercial defibrillators (Stage 1 and later comparison), some reach 360 J and about 5 kV (the Fluke analyser's ranges are 600 J and 5,000 V). The owner must choose (Q-TI-3) between a load rated only for this project (about 200 J, 2 kV) and one rated like a commercial analyser (about 400 J, 5 kV).

**Requirement R-LOAD-2.** Each step rated, with margin of at least 2× [EST: usual pulse-component derating], for: the full stored energy of the most energetic device to be tested in a single pulse (≥ 400 J for this project's device at 2× margin; ≥ 800 J if commercial devices up to 360 J are included); the peak voltage (≥ 2.5 kV project only; ≥ 5 kV with commercial devices); pulse length up to 20 ms; and the peak power in the table above.

**Repeated-shock sequences** [CALC]:

| Sequence | Energy into the load | Average power |
|---|---:|---:|
| Standard's shock-capacity test: 20 maximum-energy shocks within 1,300 s [UNVERIFIED clause 201.102.3.2] | 20 × 150 J = 3,000 J | 2.3 W |
| Same at 200 J (fault margin) | 4,000 J | 3.1 W |
| Endurance: one shock per minute for an hour (PLAN Phase 3 asks for "hundreds of shocks") | 60 × 150 J = 9,000 J | 2.5 W |
| Rapid development testing: one shock every 10 s | 150 J / 10 s | 15 W |

The Fluke analyser's limit is 12 W average ("10 defib pulses of 360 J every 5 minutes"). Rapid development testing would exceed it, so the test plan must pace shocks when a commercial analyser is the load.

### 3.3 The component class

**Suitable class: bulk-ceramic (also called ceramic-composition) non-inductive pulse resistors.** The resistive material runs through the whole body, not as a film or a wire, so the energy spreads through the body and there is no coil.

From the manufacturer's datasheet ([Ohmite 800 and 1000 series](https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/7182/800-and-1000-series.pdf)), as examples of the class:

| Example type | Size | Resistance range | Peak energy (pulses < 10 ms) | Peak voltage | Average power at 40 °C |
|---|---|---|---:|---:|---:|
| 885AS | 63.5 × 19.1 mm | 6–1,200 Ω | 2,800 J | 8,000 V | 15 W |
| 886SP | 127 × 19.1 mm | 1–330 Ω | 500 J | 4,000 V | 90 W |
| 886AS | 127 × 19.1 mm | 15–3,300 Ω | 7,000 J | 20,000 V | 30 W |
| 887AS | 152 × 25.4 mm | 12–3,300 Ω | 13,000 J | 30,000 V | 50 W |

Other families seen in the research: Ohmite slab (500 series) and encapsulated (SP, AS, BA) ceramics; HVR Advanced Power Components disc and high-voltage ceramics (about 250–350 J per cm³, temperature coefficient −0.05 to −0.15 %/°C, inductance "negligible") [UNVERIFIED detail]; Ayrton-Perry non-inductive wirewound, which has much better precision but needs pulse-curve checking ([hv_safety_isolation.md](../research_notes/Open%20source%20belt%20worn%20AED/hv_safety_isolation.md) §2).

**Ratings to look for, and the catches:**

| Rating | Why | Catch found in the datasheet |
|---|---|---|
| Single-pulse energy at the actual resistance and pulse length | Survive a full dump | "Will depend on the resistance value. Consult factory." Ratings assume pulses **under 10 ms**; this device's pulses run to 15.5 ms. Needs the maker's confirmation (Q-TI-10). |
| Peak voltage | No internal breakdown | Derated 50 % with oil-resistant coating (type AS). |
| Inductance < 2 µH | R-LOAD-1 | Bulk ceramic is inherently non-inductive; the connections then dominate. |
| Tolerance | Knowing the value | Only **±10 % or ±20 %** offered. Each step must be measured, not trusted. |
| Temperature coefficient | Drift during sequences | Up to −0.08 %/°C (AS); +0.2 to −0.08 %/°C (SP). |
| Voltage coefficient | Value under high voltage differs from the value measured with a meter | Up to −1 % per kV per inch of length. |
| Stability after overload | Value shifts after many shocks | ±2 % (AS) to ±5 % (SP) after the maker's overload test. Re-measure every session. |

**Consequence (design rule, not a part choice):** the load is a **robust absorber of known, re-measured value**, not a precision standard. Precision comes from measuring voltage and current directly (section 2.1) and from re-measuring the resistance before each session and each shock.

**Avoid:** ordinary wirewound (inductance) and thick-film or chip resistors (single-pulse limits of a few joules [BK, from the research notes]).

### 3.4 Thermal limits [CALC and EST]

One 127 × 19.1 mm tube, assuming a 9.5 mm bore [EST; the datasheet does not give the bore]: volume about 27 cm³, mass about 63 g at 2.3 g/cm³, heat capacity about 1.0 J/(g·°C) (datasheet: 0.23–0.26 cal/(g·°C)).

| Case | Temperature rise if no heat escapes | Resistance change at −0.08 %/°C |
|---|---:|---:|
| One 150 J shock | +2.4 °C | −0.19 % |
| 20 shocks in 1,300 s | +48 °C (upper bound) | −3.8 % |
| One shock per minute for an hour | steady state about +16 °C [EST: 2.5 W × about 6 °C/W, the thermal resistance implied by the 30 W rating of the 886AS from 40 °C to its 230 °C limit] | about −1.3 % |

These are well inside the material's limits (230–350 °C continuous) but far outside a metrology tolerance, and above the temperature at which a surface can burn skin [EST: surfaces above roughly 50–60 °C].

**Requirements R-LOAD-3 to R-LOAD-5:**

- **R-LOAD-3.** Each load step has a temperature sensor whose reading is recorded with every shock.
- **R-LOAD-4.** The analysis records the cold resistance (4-wire, before the shock) and the effective resistance during the pulse (v ÷ i) for every shock.
- **R-LOAD-5.** A temperature limit, set by the high-voltage engineer, blocks further shocks until the load cools; the load is never touchable while hot or while any device is connected (section 7).

### 3.5 Selecting steps: arrangements in principle and their hazards

Three arrangements are possible in principle. The choice belongs to the high-voltage engineer and the lab safety policy.

| Arrangement | Advantage | Hazard it creates | Requirement |
|---|---|---|---|
| Separate fixed loads, each in its own closed module, changed by hand | Simplest; no switch in the shock path | Every change means handling connectors near a device that may be charged | Changes only with the device switched off, its capacitor proven empty and shorted (SAFETY.md §2) |
| One selector that picks a step | Fewer connections handled | A selector caught between positions is an **open circuit** (the device stays charged, section 7); contacts may weld or arc | Selector operated only with no energy present ("cold switching"); position read back by the software; firing inhibited unless the position is confirmed |
| Fixed string with switches that short out sections | Fine steps | A failed or welded switch silently changes the resistance | Resistance actually in circuit measured automatically before every shock and compared with the selected value |

**Requirement R-LOAD-6.** Whatever the arrangement, the resistance actually in circuit is confirmed (by a low-voltage measurement) before each shock, and the analysis records it.

**Requirement R-LOAD-7.** No step change, connection or disconnection is ever made while the device under test is switched on, charging or charged (SAFETY.md §2).

### 3.6 Enclosure, interlock and discharge (carried over from SAFETY.md)

- **R-ENC-1.** The load and every high-voltage connection are inside a closed, interlocked enclosure; opening it must make it impossible to fire into the load and must not leave anything charged (SAFETY.md §2: "an enclosed, interlocked test fixture").
- **R-ENC-2.** Shocks go only into the rated load (SAFETY.md §2).
- **R-ENC-3.** A permanent, high-value bleed path across the load terminals, rated for the full voltage, so that the load terminals cannot hold a charge if the main load fails open (section 7, H-2) [principle; values for the high-voltage engineer].
- **R-ENC-4.** A separate resistor-type discharge tool, a "prove it is dead" check, and shorting straps on any capacitor not in use (SAFETY.md §2) are part of the bench, not optional extras.
- **R-ENC-5.** Visible indication at the bench whether the device under test may be charged.

### 3.7 The simulated patient: heart signal and pad resistance

A defibrillator only shocks if it sees a shockable rhythm and a pad resistance in range. So the load must also behave like a patient between shocks.

**What it must present:**

- **R-SIM-1. A heart signal across the pads**, at the amplitudes a real chest produces: about 0.1 to 5 mV (the Fluke analyser injects its AED test rhythms onto the defibrillator load limited to ±4 mV). Amplitude within ±5 % (the Fluke ECG output accuracy). Sources:
  - the project's synthetic rhythms (`aed-rhythm` `synth` module, as used by `aed-sim`);
  - recorded rhythms from the public databases the project already uses (`aed-eval`: CUDB, VFDB, MITDB), played back at their recorded rate. The data files stay under `data/` and are never committed (CLAUDE.md rule 7);
  - later, compression artefact and the pad-resistance changes compressions cause (D-36).
- **R-SIM-2. The right pad resistance at the frequency the device measures it with.** Devices measure pad resistance with a small, fast alternating signal (frequency not yet chosen for this project). A non-inductive load reads the same at that frequency as at DC; the requirement is agreement within 1 % [EST] between the DC value and the value at the device's measuring frequency.
- **R-SIM-3. A shared clock.** Every rhythm sample played is time-stamped on the same clock as the shock capture, so "the rhythm changed at time *t*" and "the shock was delivered at time *t*" can be compared to the millisecond. This turns finding F-4 (the late-rhythm-change blind spot) into a hardware measurement.
- **R-SIM-4. Scripted changes.** Rhythm changes, resistance steps (via the load selector, cold) and pads-off events at scripted times, mirroring the `aed-sim` scenarios.
- **R-SIM-5. Recovery.** After a shock, the injected signal is back to normal within 0.5 s [EST], so the simulator never limits the measurement of the device's own ECG recovery time.

**The isolation problem, stated plainly.** The heart-signal source must be connected to exactly the two points that receive a 2,000-volt, 65-amp shock, while producing signals of a few thousandths of a volt accurately, and while being controlled by a computer. Any electrical path from those two points to the computer, the mains or earth is a path the shock can take: through the signal source, into the computer, to the person using it. Commercial analysers solve this internally; how is not published in what the research read.

**Requirements:**

- **R-SIM-6.** The heart-signal source survives repeated shocks at the maximum voltage the load is rated for (2.5 kV project only, 5 kV with commercial devices) with no damage and no drift beyond R-SIM-1. The IEC 60601-1 defibrillation-proof test pulse (5 kV, 32 µF, about 400 J; [hv_safety_isolation.md](../research_notes/Open%20source%20belt%20worn%20AED/hv_safety_isolation.md) §1) is a sensible reference for the rating [EST].
- **R-SIM-7.** No conductive path from the load terminals to any computer, mains supply, earth or touchable surface. Control and data cross the barrier only by a means rated for at least the test pulse (for example optical fibre, as a class).
- **R-SIM-8.** The signal source's connection must not change the load resistance seen by the device by more than 0.1 % [EST], or it would corrupt the energy measurement.
- **R-SIM-9.** Protection must be passive and always present. It must not depend on a warning from the device under test, because a commercial AED gives no such warning and a faulty prototype might not.

How these are met (injection at a low-voltage portion of the load, disconnection, clamping, self-contained battery power, or a combination) is a design decision for the high-voltage engineer, under the lab safety policy.

---

## 4. The measuring chain: three options compared

### 4.1 (a) A commercial defibrillator analyser as the reference

| Model | Energy range and accuracy | Load | Other measurements | Price (asking, 27 Nov 2026) [UNVERIFIED] |
|---|---|---|---|---|
| **Fluke Biomedical Impulse 7000DP / 6000D** ([manual supplement](https://www.flukebiomedical.com/sites/default/files/resources/7000dp_umeng0105.pdf), primary) | 0.1–600 J; ±(1 % + 0.1 J) to 360 J | 50 Ω ±1 %, < 2 µH; optional 7010 selectable load 25–200 Ω, 5 kV | Voltage ±(1 % + 2 V) to 5 kV; current ±(1 % + 0.1 A) to 100 A; pulse width and gap ±0.1 ms; tilt; 250 kHz sampling; charge time ±0.05 s; isolated scope output (±5 %); AED test rhythms (VF coarse/fine, VT, asystole, normal) | 7000DP used $2,500–5,500; new about $6,500–10,200; 6000D new about $3,400–7,400; 7010 load used $850–1,200 |
| **Rigel (Seaward) UniPulse 400** | 0–199.9 J; ±1 % ± 0.1 J | 50 Ω ±1 % | (not retrieved) | New $4,595–6,116; rentals listed |
| **Datrend Phase 3** | 0–600 J and 0–50 J; ±1 % ± 2 digits [UNVERIFIED, unofficial manual copy] | 50 Ω fixed; optional 25–175 Ω module (FDA K062099) | (not retrieved) | (not retrieved) |
| **BC Group DA-2006** | up to 1,000 J; accuracy not retrieved | optional 25–200 Ω (DA-2006-VL) | — | DA-2006 $3,300–5,050; DA-2006P $6,600–8,500 |

**Calibration service.** Fluke Biomedical quotes on request and offers ISO/IEC 17025 accredited calibration; whether the Impulse line is inside its accredited scope must be asked. Third-party labs calibrate the 7000DP: an example certificate checks the 50 Ω load (±1 %), biphasic energy at 15, 50, 100 and 175 J (limits about ±1.5 %, lab uncertainty about ±1 %), monophasic energy, ECG and scope outputs, on a 12-month cycle, traceable to NIST. No price found [EST: a few hundred dollars].

**Strengths:** trusted by hospitals and test labs; a calibration certificate makes it traceable; it already contains the AED test rhythms, so a commercial AED will analyse and shock into it with nothing else. **Weaknesses:** a closed box (its numbers cannot be inspected); ±0.1 ms timing; only a 50 Ω load built in; 12 W average power limit; the UniPulse's 200 J ceiling is close to this device's stored adult energy (190 J). The Fluke manual warns against sticking adhesive pads onto the analyser; maker-specific pad adapters are needed [EST: $100–300 each].

**It is the anchor for trust.** Every other option is proven by comparison with it.

### 4.2 (b) A general-purpose oscilloscope with a high-voltage differential probe and a current transducer

| Part (class) | Example and published figures | Price [UNVERIFIED] |
|---|---|---|
| HV differential probe | Tektronix THDP0100: ±6,000 V, 100 MHz; DC gain ±2 % [UNVERIFIED]. **Note:** the THDP0200 is only ±1,500 V, below the 1,950 V adult charge, so it cannot be used across the load for adult shocks. | THDP0100 new about $4,700–5,700 |
| Current transformer | Pearson 110: 0.1 V/A, +1/−0 %; 5,000 A peak; droop 0.8 %/ms; about 1 Hz–20 MHz; 0.2 A·s current-time product typical without core-reset bias (phase 1 at 25 Ω uses 0.123 A·s, 62 % of that [CALC]). Must be droop-corrected (section 2.5). | $595 (110), $695 (110A) |
| Coaxial shunt (alternative) | T&M Research / Powertek SDN class; energy limit and bandwidth depend on resistance (1 mΩ part rated 12 J) [UNVERIFIED]. Sits *in* the high-voltage circuit, so the recorder side must be isolated. | not retrieved |
| Current sensor that responds to DC (alternative) | Closed-loop Hall-effect class: no droop. Not researched in this pass. | — |
| Recorder | A differential or isolated-input digitiser: e.g. PicoScope 4444 (four differential inputs, 12/14-bit, 20 MHz; 1,000 V CAT III kit), or an isolated-channel oscilloscope such as Tektronix TPS2000B (floats to about 600 V; 8-bit [BK]). Neither may be connected directly across the load; the HV probe and the current transformer do the isolating. | PicoScope 4444 1,000 V kit about £1,750 ex VAT; TPS2012B about $5,600 |

Total for this chain: about **$6,000–12,000** new [CALC from the rows above, one probe plus transformer plus recorder plus calibration estimate].

**Why ordinary probes and earth-referenced oscilloscopes are dangerous here (a hazard, not a procedure).** A normal bench oscilloscope connects the outer part of every probe to mains earth. Clip that probe's earth lead to any point of the shock circuit and the shock current has a new path: through the probe lead and the oscilloscope's chassis to earth. That can destroy the probe and oscilloscope, damage the device under test, and make the oscilloscope's metal case and every connected computer live at a lethal voltage. Ordinary passive probes are also not rated for 2 kV pulses [BK]. **Requirement R-MEAS-1:** no measurement equipment connected to the high-voltage circuit may have a conductive path to earth or to anything touchable; only equipment rated for the full test voltage between its input and everything else (differential HV probes, current transformers, isolated digitisers) may be used, as decided in the lab safety policy.

**Strengths:** waveforms can be recorded and inspected sample by sample, which the commercial analyser cannot offer; 1 MS/s and better timing. **Weaknesses:** gain accuracy of off-the-shelf probes (±2 %) limits it to the development tier until it is calibrated as a system; more connections to get right.

### 4.3 (c) A purpose-built open capture unit

A small, battery-powered, isolated recorder designed by the project: two channels sampled together at ≥ 1 MS/s with 16-bit resolution, an optical-fibre link to the computer, a precision high-voltage divider and a current sensor of the classes above, temperature inputs for the load, and event-marker inputs from the device under test. Its value is that it would be the **first open, documented, validated** defibrillator analyser, which is useful to others and is what PLAN.md set out to create.

**Cost** [EST]: a few hundred to about $1,500 in parts, but substantial design and validation effort, and its high-voltage parts fall under the same patent-review rule as the device's design files (CLAUDE.md rule 7) before publication.

**It must be proven against (a) exactly like option (b).** It does not replace the commercial reference; it reduces the cost of routine measurement and makes the method open.

### 4.4 Recommendation: a staged path

| Stage | Chain | Confidence it gives |
|---|---|---|
| 0 (now) | Analysis software only, tested on synthetic captures | That the *software* computes every quantity correctly and reports honest uncertainties. Nothing about any hardware. |
| 1 (after the lab safety policy) | Commercial analyser (a), calibrated, plus a commercial AED | Traceable energy ±1.5 % (k = 2 including the analyser's calibration) at 50 Ω; timing ±0.1 ms. Reference measurements of a proven commercial waveform. |
| 2 | Project load + chain (b), validated against Stage 1 | Development tier (±3 %) immediately; claims tier (±1 %) after accredited calibration of each channel and agreement with Stage 1 per section 5; best-in-class timing (±2 µs). All resistance steps including out-of-range cases. |
| Later, optional | Open capture unit (c), validated against Stage 1 and 2 | Same as Stage 2 at lower cost per bench, and an open instrument others can use. |

---

## 5. Proving the instrument

The analyser is the evidence-maker for everything in Phase 3 onwards. If it is wrong, every later claim is wrong. So it is proven in layers, each independent of the others.

### 5.1 Layer 1: low-voltage calibration against a traceable meter

Done under the lab safety policy, at ≤ 50 V where possible:

- **Load resistance** of every step, 4-wire, with a calibrated precision meter [EST: a 6½-digit meter gives about ±0.01 %], cold, recorded with temperature.
- **Voltage channel ratio** (divider or probe plus recorder) with a calibrated DC source at several points across its range.
- **Current channel** (transformer or shunt plus recorder) with known low-voltage current pulses of several lengths, which also measures the transformer's droop time constant.
- **Channel time offset (skew)** with one sharp low-voltage edge applied to both channels at once.

**Limitation stated plainly:** a low-voltage calibration does not show the voltage coefficient of the divider or the load (they behave differently at 2 kV), nor heating. Layer 2 covers those.

### 5.2 Layer 2: comparison with a calibrated commercial analyser on real shocks

Two complementary arrangements, both under the policy:

1. **Same shock, two chains (tests the capture chain).** The commercial analyser is the load; the project's voltage and current sensors measure the same pulses entering it. Each shock gives two readings of one event, so device shot-to-shot variation cancels. The difference between the two, shock by shock, is the capture chain's error.
2. **Alternate shocks, two loads (tests the whole instrument including the project's load).** The same reference AED fires alternately into the commercial analyser and into the project's 50 Ω step. The difference in mean energy is the whole-instrument error, mixed with the device's own variation, so more shocks are needed.

For each quantity, compute the **E_n number** used in laboratory comparisons (ISO 13528): E_n = |project − reference| ÷ √(U_project² + U_reference²), where U is each side's expanded uncertainty. |E_n| ≤ 1 means the two agree within their stated uncertainties.

### 5.3 Layer 3: an independent source — a commercial AED's own figures

A commercial AED's published waveform data (energy delivered into stated resistances, phase durations, peak current; for Philips, the application note's 1,683 V and 21.0 A into 80 Ω recorded in [DESIGN_TABLE.md](../firmware/crates/aed-waveform/DESIGN_TABLE.md)) gives a check that does not depend on the commercial analyser. It is weak (manufacturers' tolerances are wide, typically the standard's ±15 % [UNVERIFIED]) but independent: if both analysers agree with each other and disagree badly with the published waveform, something shared is wrong.

### 5.4 Layer 4: physics cross-checks on the project's own prototype

When the project's own device exists (Phase 3), its storage capacitor becomes a second, independent calibrator:

- **Charge:** capacitance × (voltage before phase 1 − voltage after phase 1) must equal the charge the current channel measured in phase 1. The capacitance can be measured at low voltage with a meter.
- **Energy:** ½C(V_before² − V_after²) must equal load energy + energy lost inside the device. Combined with the measured internal resistance, this closes the energy book-keeping.
- **Three energy formulas:** ∫v·i, ∫v²/R and ∫i²R must agree within uncertainty (section 6.4).

### 5.5 Repeatability

- **R-PROVE-1.** At least 30 shocks per condition (50 Ω and 25 Ω and 175 Ω, each mode) [EST: enough to estimate a standard deviation to within about ±25 %]. Report the mean, the standard deviation and the standard deviation of the shock-by-shock difference against the reference (arrangement 1 above), which isolates the instrument from the device.

### 5.6 Acceptance criteria (proposed; to be finalised against the purchased standard)

| Criterion | Development tier | Claims tier |
|---|---|---|
| Energy agreement with the reference at 50 Ω, both modes | mean difference ≤ 2 % | \|E_n\| ≤ 1 with U_project ≤ 1 % (k = 2) |
| Energy agreement at 25 Ω and 175 Ω (via the commercial selectable load) | ≤ 3 % | \|E_n\| ≤ 1 |
| Peak current agreement | ≤ 2 % | \|E_n\| ≤ 1 |
| Phase durations and gap | within ±0.1 ms of the reference (the reference's own limit) | same, plus internal consistency ±5 µs between repeated captures of a stable source [EST] |
| Repeatability (std. dev. of difference vs reference) | ≤ 0.5 % | ≤ 0.3 % [EST] |
| Three energy formulas agree | within 1 % | within combined uncertainty |
| Physics cross-checks (prototype only) | charge within 2 % | within combined uncertainty |
| Software | Stage 0 validation suite passes (section 6.8) at the tagged version used | same |

### 5.7 How often to re-check

- **Every session:** cold 4-wire resistance of each step to be used; zero/baseline check; one "check shock" from the reference AED into the 50 Ω step, plotted on a running control chart. A result outside the chart's limits stops work until explained.
- **After any event:** overtemperature, arc or flashover, a dropped or repaired part, a probe or cable change, a software change in the analysis tool → repeat Layer 1 for the affected part and a short Layer 2 comparison.
- **Every 12 months:** full Layer 1 and Layer 2, and accredited recalibration of the commercial analyser and of each sensor (12 months is the cycle on the example certificate).

### 5.8 Records to keep

So the evidence could later support a regulated partner's design history file (D-16) [EST: under ISO 13485-style design controls; the partner's quality system decides the final form]:

- Instrument identity: every part, serial number, configuration, and the analysis software version (git commit).
- Calibration certificates and the project's own Layer 1 records, with the meter's certificate.
- **Every raw capture file, never edited or deleted**, plus the analysis report generated from it, with checksums.
- The test plan and any deviations from it; who did the test; date, temperature, humidity.
- Control charts; non-conformances and how each was resolved.
- The comparison results (E_n tables) that justify calling the instrument fit for use, signed off by the reviewer named in the lab safety policy.

Raw captures are large (about 3 MB of text per shock at 1 MS/s [CALC: 50,000 samples × 3 columns × about 20 bytes]); where they are stored is an owner question (Q-TI-5). Captures contain no patient data, but the rule that nothing under `data/` is committed still applies.

---

## 6. Software: the open analysis tool (Stage 0, buildable now)

This is the part that can be written now. It touches no hardware, has no hazard, and can be validated entirely against synthetic captures generated from the project's own waveform model. **Its results on synthetic data must always be labelled SYNTHETIC; nothing it computes from synthetic data says anything about a real shock.**

### 6.1 Place in the repository

- A new crate, `firmware/crates/aed-analyser`, **development computer only** (like `aed-sim` and `aed-eval`), using the Rust standard library.
- Depends on `aed-types` and `aed-waveform` from the workspace, so it compares against exactly the model the device firmware uses.
- **No third-party dependencies** [proposal]: the CSV reader, JSON writer and random number generator are small enough to write in the crate. If one is ever added, it goes in `firmware/DEPENDENCIES.md` with a reason.
- Workspace lints apply (`unsafe_code = "forbid"`, no `unwrap`/`expect`/`panic`). A malformed file must produce a clear error, never a crash.

### 6.2 Inputs

**(1) The capture file** (`*.capture.csv`): one shock, sampled at a fixed rate. Plain text so it can be read by anyone, in any tool, for decades.

```
# open-aed-capture: 1
# kind: measured                  (or: synthetic)
# capture_id: 2027-03-01T14-22-05Z-0007
# created_utc: 2027-03-01T14:22:05Z
# instrument: <identifier of the measuring chain>
# calibration_ref: <identifier(s) of the calibration records used>
# analysis_note: <free text>
# sample_rate_hz: 1000000
# sample_rate_u_ppm: 50
# pretrigger_samples: 5000
# polarity: v = V(pad A) - V(pad B); i positive flowing from pad A into the load
# load_nominal_ohm: 50
# load_cold_ohm: 50.059
# load_cold_u_ohm: 0.01
# load_temp_c: 22.4
# channel: v_load | unit=V | full_scale=2500 | bits=14 | gain_u_rel=0.0025 | offset_u=0.5 | bandwidth_hz=1e6 | skew_s=0 | skew_u_s=1e-7
# channel: i_load | unit=A | full_scale=100 | bits=14 | gain_u_rel=0.0025 | offset_u=0.02 | droop_tau_s=0.125 | droop_tau_u_s=0.002 | skew_s=2.1e-7 | skew_u_s=1e-7
# channel: v_cap | unit=V | ...     (optional; only for the project's own prototype)
# dut: <identifier> | firmware=<commit or version> | mode=adult
# dut_plan: charge_v=1950 | phase1_s=0.003339 | gap_s=0.0005 | phase2_s=0.002226 | target_j=150   (optional; when the device exports its plan)
# event_clock_trigger_s: 1234.567891   (time of the trigger on the session event clock)
# synthetic_truth: <every generator parameter and the seed>   (only when kind = synthetic)
v_load,i_load
-0.12,0.003
...
```

Rules: lines starting with `#` come first; the first non-comment line names the columns; one row per sample; decimal point `.`; SI units; time is implicit (row number ÷ sample rate). A `counts` form is allowed in which each channel carries `scale` and `offset` and the values are raw recorder counts. Unknown header keys are kept and reported, not fatal. The `open-aed-capture` version line is mandatory; a version the tool does not know is refused. Optional digital columns named `mark_<event>` (0 or 1) carry fast markers from the device (shock request, phase start) when available.

**(2) The session event log** (`*.events.csv`), optional: slower events over a whole test run, time-stamped on one clock with at least 100 µs resolution: `time_s,source,event,value` — for example analysis start, charge start, charged, countdown start, shock request, cancel, rhythm change (from the simulated patient), load step selected, load temperature. Charge time, request-to-delivery and rhythm-at-delivery come from this file.

**(3) Settings**: tolerances (energy ±15 % placeholder, timing, peak current), which may be given on the command line or in a small settings file, so that when the purchased standard's figures arrive they change in one place.

**(4) For Stage 1 comparisons:** a CSV of the commercial analyser's readings per shock (`capture_id, energy_j, peak_v, peak_a, phase1_ms, gap_ms, phase2_ms, tilt_pct`) to pair with the captures.

### 6.3 Outputs

- A **Markdown report** per shock or per batch, headed MEASURED or SYNTHETIC and "NOT A MEDICAL DEVICE", with every quantity of section 1.1, its expanded uncertainty (k = 2), the model's prediction for the same plan and resistance, the difference, and the verdict.
- A **machine-readable result line** (JSON) per shock, appended to a results file, for control charts and later statistics. Its field names follow the simulator's `DischargeResult` names wherever the meaning is the same (`patient_energy_j`, `peak_current_a`, `phase1_s`, `phase1_tilt`, `final_capacitor_v`, …), so measured and simulated results can be put side by side.
- An **exit code**: 0 all pass; 1 any fail; 2 any indeterminate (and no fail); 3 input error. A capture that clipped or could not be analysed is never "pass".
- Optionally, plot data (CSV of the processed traces and the fitted exponentials) for inspection in any spreadsheet.

### 6.4 Algorithms

1. **Read and check.** Parse header and samples; refuse if the sample rate, channel list or version is missing. Flag any sample at or beyond full scale as **clipped**; a clipped channel makes every quantity that depends on it invalid.
2. **Baseline correction.** Subtract the mean of the pre-trigger samples (at least 1 ms) from each channel. Measure the post-pulse baseline (starting 2 ms after the pulse ends); if it differs from the pre-trigger baseline by more than a set fraction, correct with a straight line across the pulse and add the size of the correction to the uncertainty.
3. **Droop correction** (current transformers). A transformer behaves like a first-order high-pass filter with time constant τ_d (125 ms for 0.8 %/ms). Recover the true current as measured + (1/τ_d) × running integral of measured. τ_d and its uncertainty come from the calibration record.
4. **Deskew.** Shift one channel by the calibrated time offset (interpolating between samples).
5. **Pulse detection.** Find where |current| (and |voltage|) exceed a threshold with hysteresis: the larger of 20 V across the load (the Fluke trigger level) and 5 % of the peak. Merge active stretches separated by less than 50 µs [EST] so that ringing through zero at an edge does not split a phase. Classify each stretch by the sign of its mean current. A correct shock is exactly **two phases of opposite sign**; anything else (one phase, three, same sign twice, a chatter of short pulses) is reported as an anomaly with the shape described.
6. **Edge timing.** For each phase, fit a decaying exponential to the middle part of the phase (excluding the first and last 5 % and any ringing — at least five ringing periods or 20 µs after an edge, whichever is longer) by a straight-line fit to the logarithm of |current|. Extrapolate the fit to the edges. Each edge time is where the measured trace crosses 50 % of the fitted value at that edge, interpolated between samples. Phase duration = end − start; gap = start of phase 2 − end of phase 1. Also report 10–90 % rise and fall times.
7. **Peaks and ringing.** Report peak current two ways: the **raw maximum** (includes any overshoot) and the **fitted peak** (the exponential fit at the start edge), which is what the model predicts. Report overshoot in per cent and the ringing frequency (from zero crossings after the edge) as diagnostics. Same for voltage.
8. **Integration.** Trapezoid rule. Energy per phase over the phase window widened by a guard (three rise times or 20 µs, whichever is longer) so that edge energy is not lost: E = Σ v·i·Δt. Total energy over the whole shock, gap included. Charge per phase Q = Σ |i|·Δt.
9. **Cross-checks.** Also compute E_v = Σ v²/R_cold·Δt and E_i = Σ i²·R_cold·Δt. Report the effective resistance v/i through each phase (mean and slope). Flag if the three energies disagree by more than their combined uncertainty: likely causes are named in the report (gain error in one channel, heating, voltage coefficient, inductance).
10. **Tilt and end currents.** Tilt = (fitted start − fitted end) ÷ fitted start, each phase. Polarity: sign of phase 1 against the expected convention.
11. **Residual voltage.** If a `v_cap` channel exists, the mean over 1 ms starting 1 ms after the end of phase 2. If not, an **estimate** from the load voltage and current at the end of phase 2 plus an assumed internal resistance, clearly labelled as an estimate that depends on that assumption.
12. **Inferred device parameters.** From the fitted time constants (τ = (R_load + R_internal) × C): with a known C, infer the device's real internal resistance; with v_cap, infer C. These are the numbers that replace the model's assumptions (the 5 Ω in D-23).
13. **Timing from the event log.** Charge time, shock request to delivery (also from a `mark_` column when present, which is more precise), analysis start to delivery, and the rhythm playing at the moment of delivery and since when.
14. **"No delivery expected" cases.** For cancel, self-test, out-of-range and open cases: report the largest pulse found and an upper bound on any energy delivered; "pass" means no pulse above the trigger level and an energy bound below 0.1 J [EST].

### 6.5 Uncertainty

- **Required method: numerical sensitivity (the GUM approach).** For each uncertain input (each channel's gain, offset, skew, droop time constant, sample-rate error, quantisation, baseline correction, and R_cold for the cross-checks), re-run the whole computation with that input moved by its standard uncertainty, note how much each result changes, and combine the changes as the square root of the sum of squares. Multiply by 2 for the expanded (about 95 %) uncertainty. Report the three largest contributors for each quantity, in plain words.
- **Optional check: Monte Carlo** (GUM Supplement 1): repeat the computation 1,000 times with all inputs drawn at random from their uncertainties (seeded, reproducible generator) and take the 2.5 % and 97.5 % points. Used in validation (section 6.8, test V-4) to confirm the simpler method is not misleading.

### 6.6 Comparison with the model, and the verdict

- **Against the target.** For each shock: the target energy (150 J or 50 J) and the tolerance from settings. Using a "guard band":
  - **PASS** if the measurement plus its uncertainty lies wholly inside the tolerance;
  - **FAIL** if the measurement minus its uncertainty lies wholly outside;
  - **INDETERMINATE** otherwise. In plain words: "too close to the limit for this instrument to tell."
- **Against the model for the same shock.** Run `aed_waveform::discharge` with the plan the device used (from `dut_plan` if present, otherwise reconstructed from the measured phase durations and an inferred charge voltage) and the measured cold resistance. Report each quantity's difference from the model. This is not pass/fail; it is how well the model describes the hardware, and it feeds back into `WaveformConfig` (for example the real internal resistance).
- **Against the reference (Stage 1 and 2).** Pair captures with commercial-analyser readings, report differences and E_n (section 5.2), and summarise per resistance step.

### 6.7 The synthetic capture generator

The same tool has a `synth` command that writes capture files from the model, with impairments added in a controlled way and the true values recorded in the header (`kind: synthetic`, `synthetic_truth: …`). It is how the analysis is tested today, and later it is how a real chain's expected behaviour can be simulated before it is built.

Each impairment can be switched on alone or together, with a stated size and a seed:

| Impairment | What it imitates |
|---|---|
| Finite switch edges (set rise and fall times) | Real output switches |
| Series inductance and capacitance (damped ringing at a set frequency) | Wiring and probe effects |
| Channel bandwidth (one- or two-pole low-pass at set cut-offs, different per channel) | Probe and recorder limits |
| Transformer droop (first-order high-pass, set τ_d) | Current transformer |
| Gain error and offset, per channel | Uncalibrated sensors |
| Channel time offset | Different cable and probe delays |
| Sample-clock error (ppm) and jitter | Recorder clock |
| Random noise (set rms) and mains hum | Electrical noise |
| Quantisation to N bits over a set range, with clipping | Recorder resolution and overrange |
| Load heating during the pulse and voltage coefficient | Ceramic load behaviour (section 2.4) |
| Baseline drift | Slow zero shifts |
| Fault shapes: missing phase 2, same polarity twice, chatter, a cancelled shock (no pulse), a pulse into an open circuit | Anomaly detection |

The generator simulates internally at least 10 times faster than the output sample rate before filtering and sampling, so that the impairments act on a near-continuous signal, as they would in reality.

### 6.8 Test plan (validation of the tool)

| Test | What it shows | Pass condition (proposed) |
|---|---|---|
| V-1 Ideal capture, 1 MS/s, every whole-ohm value 25–175 Ω, both modes | The arithmetic is right | Energies within 0.01 % of the model; durations within one-tenth of a sample interval |
| V-2 Out-of-range and no-shock captures (190, 250 Ω, sub-range, open, cancel, self-test) | "No delivery" is recognised | Reported as no pulse; energy bound < 0.1 J; never "pass" on a missing expected shock |
| V-3 Each impairment alone, at three sizes | Each correction works | Error after correction within the reported uncertainty |
| V-4 **Coverage test:** 200 random seeds per condition with realistic combined impairments | The reported uncertainty is honest | The true value lies inside the ±U (k = 2) interval in at least 93 % of runs (95 % target, allowing for sampling) [EST], and not in 100 % with a needlessly wide U; Monte Carlo and sensitivity methods agree within 20 % on U |
| V-5 Sample-rate sweep, 50 kS/s to 10 MS/s | Section 2.2 holds | Documented error curve; supports the 250 kS/s and 1 MS/s requirements |
| V-6 Droop: corrected vs uncorrected | Droop correction | Uncorrected error reproduces about 0.8 %/ms; corrected error < 0.1 % |
| V-7 Clipping and overrange | Bad data is caught | Every clipped capture reported invalid; never a verdict of pass |
| V-8 Malformed files (truncated, wrong version, missing channel, text in numbers, huge values) and randomly corrupted files | Robustness | Clear error and exit code 3; no crash; no panic |
| V-9 Anomalous waveforms (section 6.7 fault shapes) | Shape checking | Each anomaly named correctly |
| V-10 Round trip: generate → write → read → analyse | File format is stable | Identical results |
| V-11 Golden files: a handful of small synthetic captures committed with their expected results | Regressions are caught | Results unchanged unless the change is deliberate and recorded |
| V-12 Consistency with `aed-sim`: run a simulator scenario, synthesise the capture of each `ShockRecord`, analyse it | The tool and the simulator describe shocks the same way | Every shared field agrees within numerical tolerance |
| Later, Stage 1: real captures from the commercial analyser's scope output | First contact with hardware | Agreement with the analyser's own readings within the scope output's ±5 % (a shape check, not an accuracy check) |

The tool counts as validated for a given version when V-1 to V-12 pass; that version's git commit is recorded with every later measurement report.

---

## 7. Hazards specific to the instrument, and the requirements they create

These are inputs to the future lab safety policy (SAFETY.md §2). They are requirements only; how each is met is for the policy and the high-voltage engineer.

| No. | Hazard | Requirement | SAFETY.md link |
|---|---|---|---|
| H-1 | **Stored energy in the device under test.** It holds about 190 J at 1,950 V before a shock and up to about 36 J (846 V) after one; a cancelled or refused shock leaves the full charge, to be dumped internally. A commercial AED's internal state cannot be observed. | Treat the device as charged until proven otherwise. For the project's own prototype, an independent capacitor-voltage indication visible at the bench (R-ENC-5) and an independent dump. The analysis flags every case where a shock was expected and none arrived ("energy may still be in the device"). | §2: discharge by default, bleeder, discharge tool, prove-dead check, shorting straps |
| H-2 | **The load fails open** (cracked element, loose or burnt connection, selector between positions). The device fires into nothing: no current flows, so the capacitor stays charged, and the load terminals may hold high voltage. | Continuous low-voltage continuity monitoring of the load, interlocked so the device cannot be armed or fired unless continuity is good (R-LOAD-6); a permanent rated bleed path across the load terminals (R-ENC-3); every "no current flowed" capture treated as H-1. | §2: shocks only into a rated load |
| H-3 | **The load fails short**, or a near-short test. Current limited only by the device's internal resistance: about 390 A at 1,950 V with 5 Ω [CALC]. Magnetic forces on cables, damage to the device's switches, arcing. | "Short" tests use a rated low-value resistor, never a wire (section 3.1); every step rated for a full dump (R-LOAD-2); cables secured against movement [EST: EFCOG's 500 A guidance for discharge sticks]. | §2 |
| H-4 | **Hot load after repeated shocks.** Up to about +48 °C above room after 20 shocks with no cooling [CALC]; burns; resistance drift. | Temperature sensing and a shock-inhibit limit (R-LOAD-3, R-LOAD-5); load not touchable when hot; shock pacing within the analyser's and the load's average-power limits (12 W for the Fluke analyser). | §2: enclosed fixture |
| H-5 | **Arc or flashover at connectors** at 2 kV (5 kV with commercial devices): burns, ultraviolet flash, noise, a path to the operator. | Connectors and spacings rated with margin above the highest test voltage; no clip-on test leads in the shock path [EST: EFCOG advises against alligator clips above 1,000 J; this is below that, but the risk is the same in kind]; connections made only with the device off and proven dead (R-LOAD-7); enclosure closed during shocks (R-ENC-1); the analysis flags captures whose shape suggests arcing. | §2 |
| H-6 | **Earth-referenced measurement equipment** (section 4.2): the shock current finds a path through an oscilloscope or computer to earth; equipment and its case become live. | R-MEAS-1; R-SIM-7; the computer connects to the high-voltage side only through an isolated means rated for the test pulse; laptops on battery during shocks is a policy option [EST]. | §2 |
| H-7 | **The simulated patient's signal source back-feeds the shock** into low-voltage equipment or the computer (section 3.7). | R-SIM-6 to R-SIM-9: passive, always-present protection and isolation rated for at least the 5 kV defibrillation-proof test pulse. | §2 |
| H-8 | **Sensor overrange or saturation:** a current transformer beyond its current-time product (0.2 A·s typical without bias; 0.123 A·s used by phase 1 at 25 Ω, more with high-energy commercial devices) gives a wrong reading that looks plausible. | Margin of at least 2× on every sensor rating for the most energetic device to be tested [EST]; the analysis detects clipping and saturation shapes and refuses to give a verdict. | §5 honest reporting |
| H-9 | **Fully automatic devices fire without a button** (this project's device, D-06, and some commercial AEDs). A test can shock as soon as the device sees a shockable rhythm. | All connections complete and the enclosure closed **before** the device is switched on; the simulated patient starts on a non-shockable rhythm and changes to a shockable one only by command from outside the enclosure. | §2; §1 never on a person |
| H-10 | **Charge creeping back** after a dump (dielectric absorption): up to about 2 J on an electrolytic bank [from hv_safety_isolation.md §7]. | Shorting straps on any capacitor not under test (SAFETY.md §2); applies to the prototype, not to sealed commercial AEDs. | §2 |
| H-11 | **Wrong evidence:** the analysis tool reports pass on a bad shock, or an uncalibrated instrument is used for a claim. | Validated tool version recorded with every report (section 6.8); guard-banded verdicts with "indeterminate" (6.6); session check shock and control chart (5.7); claims only from instruments that met the claims-tier criteria (5.6). | §5 honest reporting |
| H-12 | **The analyser mistaken for a patient-safe device** or a prototype tested on a person "just to check". | The instrument's labels and documents say it is test equipment for loads only; nothing in the lab is ever connected to a person or animal. | §1 |

The lab safety policy also needs, from SAFETY.md §2: who is qualified (D-18), never working alone, emergency procedure and the nearest real AED.

---

## 8. Staged plan with rough costs

### Stage 0 — now, no hazard: the analysis software

- **Work:** write `aed-analyser` to section 6; run the validation suite V-1 to V-12; publish the report and the file format.
- **Cost:** no cash [CALC: software only, existing computer].
- **Skills:** Rust; signal processing; measurement uncertainty (GUM). All available to the project now (an agent can implement it from section 6).
- **Exit criteria:** V-1 to V-12 pass; the coverage test (V-4) shows the reported uncertainties are honest; the file format is documented in the crate's README; results labelled SYNTHETIC throughout.

### Stage 1 — after the lab safety policy is adopted: reference measurements

- **Work:** acquire (buy, rent or borrow) a commercial analyser with a current calibration certificate, and a commercial AED. Fire the commercial AED into the analyser across its loads: record energy, peak current, durations, gap and tilt at each resistance, and charge times; record the analyser's isolated scope output with an isolated recorder for shape (option (b)'s recorder only, no probes on the shock circuit); feed both into the Stage 0 tool.
- **What this alone teaches, with no project-built high-voltage hardware:** how a proven biphasic waveform actually adapts its phase lengths to resistance; its real peak currents, gap and tilt (to compare with the design table and with Philips' published figures); its charge times; how it behaves at 190 Ω, below range and on an open circuit; and how quickly its own ECG analysis resumes after a shock. It also gives the project its first real captures to test the analysis tool on.
- **It is still a high-energy activity.** A commercial AED firing into an analyser stores and delivers the same order of energy as this project's device. It falls under the lab safety policy, and whether it may start before the high-voltage engineer of D-18 joins (for example with a trained biomedical technician) is an owner decision (Q-TI-1).
- **Cost** [UNVERIFIED asking prices, section 4.1 and the sources notes]:

| Item | Used / borrowed route | New route |
|---|---:|---:|
| Commercial analyser | $2,500–5,500 (used Impulse 7000DP) or rental/loan | $4,600–10,200 (UniPulse 400 to Impulse 7000DP) |
| Selectable load 25–200 Ω (Fluke 7010) | $850–1,200 used | included in some bundles (~$13,500 total new) |
| Calibration of the analyser | a few hundred dollars [EST] | often included new |
| Commercial AED (Philips HeartStart OnSite, the waveform family the project follows) | $570–1,275 used or refurbished | $1,622–1,841 |
| AED pads/pad adapters for the analyser | $100–300 [EST] | same |
| Isolated recorder for the scope output | ~£1,750 (PicoScope 4444 1,000 V kit) or borrow | same |
| Lab safety equipment named in SAFETY.md (enclosure, discharge tool, straps, signage) | $500–2,000 [EST] | same |
| **Total** | **about $4,500–12,000** | **about $9,000–18,000** |

  A loan from a hospital clinical engineering department or a university biomedical engineering programme, which own these analysers, would cut this to the AED and safety equipment [EST].
- **Skills:** someone qualified under the policy; familiarity with defibrillator analysers (a biomedical equipment technician is the usual profession).
- **Exit criteria:** analyser calibration current; at least 30 shocks per resistance step recorded and analysed; the commercial AED's waveform tabulated against the design table; the Stage 0 tool run on real scope captures without anomalies it cannot explain.

### Stage 2 — the project's own load and capture chain, validated against Stage 1

- **Work:** design (by the high-voltage engineer, after patent review where relevant) and build the load (all steps of section 3.1), the simulated patient (3.7) and the capture chain of option (b); calibrate (5.1); compare with Stage 1 (5.2–5.5); accept to the development tier, then to the claims tier after accredited calibration.
- **Cost** [EST unless sourced]:

| Item | Range |
|---|---:|
| Load elements for about 11 steps plus spares (bulk ceramic class; price not retrieved) | $500–1,500 [EST] |
| Enclosure, interlock, step selection, connectors, temperature sensing | $500–2,000 [EST] |
| HV differential probe (Tektronix THDP0100 class), or a precision divider | $4,700–5,700 new |
| Current transformer (Pearson 110 class), plus a second as a check | $600–1,300 |
| Isolated recorder ≥ 1 MS/s, ≥ 14 bit (if not bought in Stage 1) | ~$1,200–2,500 |
| Simulated-patient signal source with isolation | $300–1,000 [EST] |
| Precision 4-wire meter (or borrow) and its calibration | $500–2,000 [EST] |
| Accredited calibration of probe and current sensor (claims tier) | $500–1,500 [EST] |
| Precision DC high-voltage reference for D-28 sensor checks | not researched (Q-TI-7) |
| **Total** | **about $9,000–18,000**, less with used probes |

- **Skills:** professional high-voltage engineer (D-18); measurement and calibration practice; mechanical design of the enclosure.
- **Exit criteria:** acceptance criteria of section 5.6 met at the development tier for all steps; claims tier at least at 50 Ω, 25 Ω and 175 Ω; records of section 5.8 complete; this then satisfies PLAN.md Phase 2's exit condition "analyser agrees with a commercial instrument within a stated error".

### Later, optional — the open capture unit (option (c))

Only if the project wants a cheaper routine instrument or an open instrument for others. Parts a few hundred to about $1,500 [EST]; validated exactly like Stage 2.

---

## 9. Open questions

### For the owner

- **Q-TI-1.** May Stage 1 (a commercial AED fired into a commercial analyser) begin once the lab safety policy is adopted, with a qualified biomedical technician, or must it wait for the high-voltage engineer of D-18? It involves no project-built high-voltage hardware but is still high-energy work.
- **Q-TI-2.** Buy, rent or borrow the commercial analyser? Buying used is cheapest over time (about $2,500–5,500 plus calibration); borrowing from a hospital or university is cheapest now and brings expert help, but depends on goodwill and their schedule.
- **Q-TI-3.** Should the project's load be rated only for this device (about 200 J, 2.5 kV) or like a commercial analyser (about 400 J, 5 kV) so it can also measure commercial defibrillators and be useful to others? The second costs more and raises the hazard level.
- **Q-TI-4.** Which commercial AED to measure? Recommendation: a Philips HeartStart model, because the project's waveform follows Philips' (D-02) and Avive's approval rested on matching it; a second make later for contrast.
- **Q-TI-5.** Where should raw capture files live? Committing them to the public repository keeps the evidence open but grows it by about 3 MB per shock; an external archive with checksums in the repository is lighter.
- **Q-TI-6.** When to pay for the claims tier (accredited calibration, about $500–1,500 [EST] a year)? Suggested: only when results are about to go into evidence for a partner; the development tier suffices until then.
- **Q-TI-7.** Is a precision DC high-voltage reference (to check the D-28 sensors to about ±0.3 % at 2 kV) worth buying, or should those sensors be checked at a calibration lab?
- **Q-TI-8.** Pursue the open capture unit (option (c)) as a contribution to the wider community, after the project's own needs are met?
- **Q-TI-9.** May measurements of a named commercial AED be published in the public repository? Probably unproblematic, but worth one question to the regulatory consultant or lawyer named in DECISIONS.md.

### For the future high-voltage engineer

- **Q-TI-10.** Load element choice: confirm with the maker the energy and voltage ratings at the actual resistance values and for pulses of 10–16 ms (the datasheet's ratings assume under 10 ms).
- **Q-TI-11.** Step selection arrangement (section 3.5), and how to make the open-circuit state impossible to fire into.
- **Q-TI-12.** The simulated patient's isolation and injection method (section 3.7).
- **Q-TI-13.** Voltage measurement: commercial differential probe or a purpose-built divider with a known voltage coefficient? Current: transformer with droop correction, a DC-responding sensor, or a coaxial shunt with an isolated recorder?
- **Q-TI-14.** The grounding and isolation scheme of the whole bench, including where (if anywhere) the high-voltage side may be tied to earth.
- **Q-TI-15.** Connector, spacing and enclosure ratings for 2.5 kV or 5 kV (Q-TI-3).
- **Q-TI-16.** Whether the project's own prototype should carry built-in test points for the capacitor voltage and fast event markers (shock request, phase starts), so the analyser can measure residual voltage and request-to-delivery directly. This is a requirement on the device design, not the instrument, and should be decided before Phase 3 layout.

### Tasks

- **T-1.** Confirm every test method, load value and tolerance from the purchased IEC 60601-2-4 (Edition 3.2) and update sections 1, 3.1 and 5.6.
- **T-2.** Implement and validate the Stage 0 tool (section 6).
- **T-3.** Get quotes: analyser calibration (and whether it is inside an accredited scope), probe and current-transformer calibration.
- **T-4.** Research the gaps in the sources notes: DC-responding current sensors, precision HV dividers, the Tektronix probe's DC accuracy from its manual, load-element prices.

## What could not be verified

- Every IEC 60601-2-4 clause number, test method and limit (standard not bought).
- The Tektronix THDP series ±2 % DC gain figure (retailer page only); PicoScope 4444 DC accuracy; BC Group DA-2006 accuracy; Datrend Phase 3 accuracy (unofficial manual copy).
- All prices are asking prices from resellers; calibration prices were not found.
- Load-element mass, bore and prices; thermal resistance in use (estimated from the power rating).
- How commercial analysers isolate their ECG injection.
- Which closed-loop current sensors and precision HV dividers would suit; not researched.
