**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# WP-H revision 0.3 check simulations (sheets 14 and 15)

27 November 2026, ngspice 42, written with AI assistance by the WP-H capture engineer. These runs check what is
drawn on `specs/14_patient_interface.toml` and `specs/15_frontend_main.toml` after change order WP-H
(`capture-notes/rev3-changes.md`, H-1…H-6). Every number is **simulated** with hand-built models (no maker's SPICE
file was downloaded). Nothing here is safe or ready for use on a person or animal, and nothing here is an
instruction to build or power anything.

Run: `bash run.sh` (about 10 minutes). Results: `out/summary.txt` (`out/` is not kept in git).

## In plain language

1. **Relay coil supply.** The relay's energy store is now filled before the shock is due, under its own switch-on
   signal from the main controller. In simulation it fills in 60–168 ms and draws at most 3.3 A from the battery
   (including about 0.6 A for the rest of the device). The 9 A surge of the old drawing is gone. With a cold, nearly
   empty battery the battery rail stays at 5.4 V or above, against 4.35 V before. The relay changes over 18.1–18.4 ms
   after the sequencer's command. With the firmware's 14 ms settling wait that gives 33.1–33.4 ms, inside the 40 ms
   limit with 6.6 ms to spare. If the supply then fails mid-shock, the relay stays held for 42.7 ms or more,
   against the 30 ms required.
2. **The new current-limit value works but sits outside the maker's chart.** The maker's chart for the current-limit
   resistor runs only up to 400 kΩ; the change order's 515 kΩ is above that. The limit is also a peak value rather
   than an average, so the converter delivers less than the change order assumed. It is still enough, because the
   store was filled earlier. This needs a check with the maker or a measurement.
3. **Two firmware timing rules matter.** The store must be switched on at least 200 ms before the relay command
   (worst case it needs 168 ms). Switched on only 50–100 ms early, the relay does not change over in time and the
   shock is refused. That is the safe direction, but it would be a refusal. Running the high-voltage charger at full
   power at the same moment on a cold, nearly empty battery pulls the battery rail to about 3.7 V. That is mostly
   the charger's own load (about 4.3 V without the relay supply), but it argues for switching the store on once
   charging has finished.
4. **Compression sensing** now has its own input resistors and reads 0.97 of the pad voltage instead of 0.17.
   Simulated noise is **4.6–6.3 mΩ** against the 10 mΩ limit (charger off). That rests on an estimated chip noise
   figure: at twice that figure the noise is 8.5–11.6 mΩ, so the limit could be missed. That needs measuring.
5. **Heart-signal channel and protection are unchanged.** Gain, noise and the residual 64 kHz carrier are the same
   as before. In both surge cases (an outside defibrillator's 5 kV pulse, and this device's own shock through a
   failed relay), every chip pin current, clamp voltage and resistor energy is the same as before. The new resistor
   and clamp each carry what the existing impedance resistor and clamp carry.

## Files

| File | What it simulates |
|---|---|
| `relay_supply_wph.cir` | sheet 14 as drawn: `K_SUPPLY_EN` pre-charge (C1401 680 nF, R1406 515 kΩ), then `K_EN` coil start; battery 8.4 / 7.4 / 6.0 V × 20 / 80 mΩ per cell; boost current-limit tolerance; charger running at the same time; hold-up after a supply failure; coil-inductance sensitivity; late `K_SUPPLY_EN`. Also R1406 340 kΩ as a comparison |
| `imp_ecg_noise_wph.cir` | sheet 15, 64 kHz small-signal: impedance sensitivity and noise, issue-2 vs revision-0.3 topology, with TVS and DPCR parasitics; ECG gain, resistor noise and carrier at the ECG inputs |
| `fe_protect_wph.cir` | sheet 15, transient: pad A's taps under the external 5 kV pulse and under this device's own shock (2,409 V bound) |

## Models (what differs from `../logic-relay/`)

| Item | Model | Source | Confidence |
|---|---|---|---|
| TPS61089 current limit | **peak switch current** = 1,030,000 / R_ILIM (2.0 A at 515 kΩ). The average input current at the limit comes from the constant off-time control (CCM: Ipk − ΔI/2; DCM: triangle average). Tolerance −0.8 A / +0.8 A | TI SLVSD38C (Aug 2021) §8.3.5 Eq. 2, §9.2.2.3 ("can be 0.8 A lower"), EC table 7.3–8.1–8.9 A at 127 kΩ; Figure 7-3 graphs only 100–400 kΩ | moderate; **515 kΩ is outside the characterised range** |
| TPS61089 soft start | 4 ms | SLVSD38C §8.3.2, EC tSS 2/4/6 ms | high |
| DRV110 | UVLO 4.6 V; start-up delay ≤ 50 µs (not modelled); 0.25 mA standby while the reservoir is charged | TI SLVSBA8G | high |
| Relay coil and mechanics | 12 Ω (15 Ω hot), 60 mH [EST] (also 30 and 120 mH); pick-up 0.67 A; contacts transfer 10 ms after pick-up; release below 0.42 A | `docs/requirements/patient-relay.md` §4; no relay exists | low |
| Battery, rails, Q1401 | as `../logic-relay/relay_supply.cir` | LR §3 | low to moderate |
| HV charger load | 9.2 A constant during the pre-charge (case C only) | HARDWARE_DESIGN §B4.2 (9.2 A average at full charge) | moderate as a bound |
| CDSOD323-T03C | anti-series zeners, BV 4.0 V, RS 3 Ω (≈ 7.7 V at 1 A, above the 7 V datasheet maximum), 3 pF | Bourns REV 04/23 (via WP-H §2) | moderate; pessimistic for pin currents |
| DPCR body capacitance | 0.2 pF per 2512 | [EST] | low; negligible in the results |
| ADS1292R pins | diodes to AVDD (stiff 3.3 V) and GND; 5 pF input | [EST]; SBAS502C gives abs max only | moderate |
| ADS1292R impedance-channel noise | 0.13 µV/√Hz (and 2× as a what-if) | WP-H §3.4 estimate from SBAS502C | low to moderate |

## 1. Relay coil supply (`relay_supply_wph.cir`)

Timing: `K_SUPPLY_EN` at 1 ms, `K_EN` 200 ms later (FW-12 minimum lead). Hot coil 15 Ω, 60 mH. "Total" = transfer
+ 14 ms firmware wait (FW-1) + 1 ms tick, against the 40 ms abort limit counted from `RELAY_CMD` (taking `K_EN` to
rise with `RELAY_CMD` because the firmware's 30 ms rail-ready wait, FW-2, has already passed).

### 1.1 As drawn (R1406 515 kΩ), every battery corner × current-limit tolerance (36 runs incl. §1.2)

| Quantity | Range over the 18 runs | Limit / comparison | Margin |
|---|---|---|---|
| Battery peak during the pre-charge (whole battery) | **1.71–3.33 A** (−0.8 A corner 1.7–1.8; nominal 2.4–2.6; +0.8 A 3.2–3.3) | change order: 2.5–2.6 A; H-6 "≈ 2.0 A" for the coil supply | nominal as ordered; +0.8 A corner 0.7 A above |
| Reservoir at 11 V after `K_SUPPLY_EN` | **60–168 ms** (168 ms at 6.0 V, 80 mΩ, −0.8 A) | must be < 200 ms lead | **32 ms** |
| Reservoir at the moment `K_EN` rises | ≥ 11.68 V | — | — |
| `VBAT_SYS` minimum, cold nearly empty battery (6.0 V, 80 mΩ) | **5.40 V** (+0.8 A corner), 5.52 V nominal, 5.67 V (−0.8 A) | issue-2 drawing: 4.35 V (LR §3.1) | — |
| `5V0_SYS` / `BRIDGE_5V` minimum, all cases | 5.08 / 5.00 V (regulation held: dropout begins below `VBAT_SYS` ≈ 5.2 V) | ≥ 4.9 V (charger controller) / ≥ 4.60 V (rail-collapse trip top) | 0.18 / 0.40 V; `VBAT_SYS` 0.2 V above the dropout point |
| Battery peak during the coil peak phase | 1.21–2.10 A | — | — |
| Reservoir minimum during the coil peak phase | 8.67 V (6.0 V, 80 mΩ, −0.8 A) – 11.51 V | pick-up at 15 Ω needs 10 V, reached first at ~7 ms while the reservoir is still ≥ 11 V | — |
| Contacts transfer after `K_EN` | **18.1–18.4 ms** | change order 15.9 ms (simulated with 2.5 A average, LR mode 1) | — |
| Transfer + 14 ms + 1 ms tick | **33.1–33.4 ms** | ≤ 40 ms | **6.6 ms** |

### 1.2 Comparison: R1406 340 kΩ (3.0 A, inside the characterised range)

Transfer the same (18.1 ms); reservoir full in 55–110 ms; but battery peak during the pre-charge **2.65–4.26 A**
and `VBAT_SYS` down to 5.24 V. Not adopted: it raises the battery current, which is what H-2 set out to cut, and
515 kΩ meets every criterion here. The comparison is kept because it is the fallback if TI says 515 kΩ is not
supported.

### 1.3 HV charger running during the pre-charge (case C; +0.8 A corner)

| Battery | `VBAT_SYS` min | `5V0_SYS` min | Reservoir at 11 V |
|---|---|---|---|
| 7.4 V, 20 mΩ | 6.59 V | 5.08 V | 80 ms |
| 7.4 V, 80 mΩ | 5.09 V | **4.98 V** | 116 ms |
| 6.0 V, 20 mΩ | 5.18 V | 5.07 V | 111 ms |
| **6.0 V, 80 mΩ** | **3.72 V** | **3.61 V** | 193 ms |

The cold nearly empty case collapses mainly because of the charger itself: 9.2 A × (2 × 80 mΩ + 25 mΩ) = 1.7 V →
≈ 4.3 V without the coil supply [CALC]; the coil supply adds ≈ 0.6 V. A cold battery at 6.0 V probably cannot
deliver the charger's full power at all (outside WP-H; raised). With 340 kΩ it is worse (3.62 V).

### 1.4 Hold-up: boost and its input lost 20 ms after `K_EN` (cold nearly empty battery, hot coil)

| R1406 | limit corner | Reservoir | Reservoir at the failure | Relay held after the failure |
|---|---|---|---|---|
| 515 kΩ | −0.8 A | 6.6 mF | 10.36 V | **56.8 ms** |
| 515 kΩ | −0.8 A | 5.28 mF (−20 %) | 10.11 V | **42.7 ms** |
| 515 kΩ | nominal | 6.6 / 5.28 mF | 10.84 / 10.68 V | 63.1 / 48.4 ms |
| 340 kΩ | −0.8 A | 6.6 / 5.28 mF | 10.96 / 10.83 V | 64.8 / 50.0 ms |

Requirement ≥ 30 ms (§B7.2): met, worst margin **12.7 ms**. (The 340 kΩ nominal 6.6 mF run gave no crossing within
the 69 ms window: held longer than that.)

### 1.5 Coil sensitivity (cold nearly empty battery, −0.8 A corner)

| Coil | 30 mH | 60 mH | 120 mH |
|---|---|---|---|
| 12 Ω: transfer / total | 12.9 / 27.9 ms | 15.9 / 30.9 ms | 22.1 / 37.1 ms |
| 15 Ω (hot): transfer / total | 14.1 / 29.1 ms | 18.4 / 33.4 ms | **28.4 / 43.4 ms: over 40 ms** |

The result depends on the coil's inductance, which nobody knows yet (no relay exists). A hot coil of about
100 mH or more misses the 40 ms limit. The relay requirement sheet asks the maker only to state the inductance;
it should also bound the coil's current-rise time (raised).

### 1.6 `K_SUPPLY_EN` raised late (cold nearly empty battery, −0.8 A corner)

| Lead before `K_EN` | Reservoir at `K_EN` | Transfer |
|---|---|---|
| 50 ms | 2.8 V | none within the run (refused by the 40 ms abort) |
| 100 ms | 8.4 V | none within 100 ms (refused) |
| (340 kΩ, 100 ms) | 10.3 V | 23.8 ms |

A late `K_SUPPLY_EN` makes the relay too slow and the shock is refused: the safe direction, but a refusal. The
≥ 200 ms lead of FW-12 is necessary; worst-case fill 168 ms leaves 32 ms.

## 2. Impedance and ECG channels (`imp_ecg_noise_wph.cir`), charger off

| Topology | Supervisor load | Patient | IN1 / pad voltage at 64 kHz | Resistor noise | Impedance noise at 80 / 100 µA (chip 0.13 µV/√Hz) | Same, chip 2× |
|---|---|---|---|---|---|---|
| issue 2 | 4.2 k / 60 k | 25 Ω, 175 Ω | 0.167 | 24.5 nV/√Hz | 31.1–34.0 / 24.9–27.2 mΩ | 60.7–66.3 / 48.5–53.0 mΩ |
| **revision 0.3 (drawn)** | 4.2 k | 25 Ω, 175 Ω | **0.970** | 43.9 nV/√Hz | **5.81–6.27 / 4.64–5.02 mΩ** | 10.8–11.6 / 8.6–9.3 mΩ (the 175 Ω, 100 µA line is garbled in the log by a solver message; 9.3 = 11.64 × 80/100) |
| **revision 0.3 (drawn)** | 60 k | 25 Ω, 175 Ω | 0.970 | 43.9 nV/√Hz | **5.74–5.82 / 4.59–4.65 mΩ** | 10.7–10.8 / 8.5–8.6 mΩ |

Requirement < 10 mΩ RMS (0.5–10 Hz): **met with margin 1.6× at the worst case, with the estimated chip noise**;
**not met** at 80 µA if the chip's real impedance-channel noise is twice the estimate. This reproduces LR §6
(4.6–6.2 mΩ) with the drawn parasitics added (TVS 3 pF, DPCR 0.2 pF), which change it by < 0.1 mΩ.

ECG channel (pad difference → IN2 difference, patient 75 Ω):

| | Gain 10 Hz | Gain 40 Hz | Gain 2.8 kHz (filter corner) | Resistor noise 0.5–40 Hz | 64 kHz carrier at IN2 / at the pads |
|---|---|---|---|---|---|
| issue 2 | 0.989 | 0.984 | 0.692 | 0.280 µV RMS | 0.0427 |
| revision 0.3 | 0.993 | 0.993 | 0.698 | 0.281 µV RMS | 0.0430 |

Unchanged: the filter corner is the same and the resistor noise is the same. The chip's own 0.7 µV RMS
(SBAS502C Table 3) dominates.

## 3. Defibrillation protection of the main front end (`fe_protect_wph.cir`)

Pad A's taps carry the whole event and pad B is held at ground: the one-sided case of WP-H §3.1 (conservative).

| | External 5 kV pulse, issue 2 | External, rev 0.3 | Own shock 2,409 V, 19.9 ms, issue 2 | Own shock, rev 0.3 |
|---|---|---|---|---|
| R1507 (impedance sense) peak / energy | 97 mA, 482 W / 0.83 J | same | 47 mA, 113 W / 2.26 J | same |
| **R1528 (new ECG sense)** peak / energy | — | **97 mA, 482 W / 0.83 J** | — | **47 mA, 113 W / 2.26 J** |
| Clamp node voltage (each TVS) | 5.36 V | 5.36 V | 5.02 V | 5.02 V |
| IN2P pin peak current | 0.45 mA | 0.45 mA | 0.79 mA | 0.79 mA |
| RESP_MODP pin peak current | 1.94 mA | 1.94 mA | 3.20 mA | 3.20 mA |
| Current into AVDD (pad A side) | 1.60 mA | 1.61 mA | 1.19 mA | 1.29 mA |
| Drive tap: peak / per 24 k | 0.32 A / 13 mJ | same | 0.38 A / 20 mJ | same |
| C1501 peak voltage | 2,397 V | same | 1,205 V | same |

Ratings (TT DPCR datasheet 09.26, curves as read by WP-H): 5 kV peak, 100 pulses; single pulse ≈ 1.5 kW at 1 ms
(3.1× over 482 W) and ≈ 250 W at 20 ms (2.2× over 113 W). Each TVS carries ≤ 97 mA for milliseconds, ≈ 2 mJ
(external) or ≈ 5 mJ (own shock); the TVS has no published millisecond rating [UNVERIFIED], as for D1503. The
issue-2 model here gave D1503 the same 97 mA, so the new clamp does not change any existing part's stress.

A second external pulse applies the same per-pulse figures again; DPCR's rating is for 100 pulses. When the patient
relay is transferred, the front ends are disconnected and see only the open-contact capacitance.

## 4. What contradicts the change order or earlier notes

| Document says | Simulation says |
|---|---|
| H-2 / LR §3.2: "R1406 515 kΩ (ILIM ≈ 2.0 A)", battery 2.5–2.6 A | 2.0 A is a **peak switch** current (SLVSD38C Eq. 2); with its ±0.8 A tolerance the battery peak is 1.7–3.3 A; 515 kΩ is above the graphed 100–400 kΩ |
| H-2 / LR §3.2: transfer 15.9 ms in every battery case | that run used a 2.5 A average limit (R1406 412 kΩ, LR mode 1); with the drawn 515 kΩ and the peak-limit model, 18.1–18.4 ms (hot coil); 33.4 ms total, still < 40 ms |
| H-2: reservoir at 11 V after 74–99 ms | 60–168 ms over the current-limit tolerance; 200 ms lead still sufficient |
| FW-12: raise `K_SUPPLY_EN` "when charging for a shock starts" | with the charger at full power on a cold nearly empty battery, `VBAT_SYS` falls to 3.7 V (charger alone ≈ 4.3 V) |

## 5. What cannot be settled without measurement

- TPS61089 current limit at R_ILIM 515 kΩ (outside the datasheet's graph) and its loop behaviour into 6.6 mF.
- The relay coil's inductance, pick-up current and mechanics (no relay exists, D-64).
- ADS1292R impedance-channel noise at 64 kHz in this circuit (the 0.13 µV/√Hz estimate decides the 10 mΩ margin).
- Battery DC resistance at −20 °C and low charge.
- TVS behaviour for millisecond pulses; DPCR behaviour under the real IEC test waveform.
