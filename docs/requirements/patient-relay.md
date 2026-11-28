# Requirements sheet: patient relay K1401 (made to order)

Version 0.1, 27 November 2026. Decisions D-42 and **D-64 (provisional, awaiting owner confirmation)**. Written with
AI assistance from `docs/HARDWARE_DESIGN.md` v0.3 §B7.2, the capture notes `hardware/aed-main/capture-notes/WP-H.md`
§3.5 and §4, and the simulations in `hardware/aed-main/sim/logic-relay/notes.md` §1.4 and §3.

> **NOT FOR CONSTRUCTION — unreviewed design.** This is a list of what a part to be bought must do and what
> proof to ask its maker for. It says nothing about how to make it, and nothing here is an instruction to build or
> power anything. Nothing in this project is safe or ready for use on a person or animal (SAFETY.md).

## In plain language

The patient relay is a switch with moving contacts that connects the two pads either to the heart-signal
electronics (at rest, the normal state) or to the shock circuit (only while a shock is being given). It must carry
the shock current without damage, keep about 2,000 volts away from the heart-signal electronics, move quickly and
predictably, and tell the controller truthfully where its contacts are. No relay found in a catalogue publishes a
rating for the shock current or has a position contact tied to both of its switching contacts, so it has to be
made to order (integration issue I-6). Until a maker offers one, the circuit drawing uses a **placeholder symbol**
(`K1401`, value "DPDT HV relay + aux — MADE TO ORDER, PLACEHOLDER") so that the wiring can be reviewed.

The figures below come from this design's own arithmetic and simulations. Each says where. Figures marked [EST]
are targets that a maker's answer may move.

## 1. Function

| Item | Requirement | Basis |
|---|---|---|
| Arrangement | Two poles, each a changeover (common = pad; rest side = heart-signal electronics; worked side = shock circuit) | HARDWARE_DESIGN §B7.1 |
| Default state | Both poles at rest with the coil unpowered | §B7.1: pads stay on the front ends when anything fails |
| Switching duty | **Cold-switched only**: the contacts never make or break current in normal use. They may be asked to open with a fault current flowing only if both the coil hold-up and the bridge's turn-off have failed (two faults) | §B13.5 turn-off order: bridge off before the coil is released |
| Position contact | One low-voltage contact, **mechanically linked to both poles** (force-guided), closed **only when both poles are fully at rest** | §B7.2; WP-H contract issue 4; fault 29 (one pole welded at rest while the other moves) |
| Position contact, circuit | Dry circuit, 3.3 V, 0.1–1 mA [CALC: 3.3 V / 100 kΩ pull-up on sheet 04 = 33 µA, plus the sequencer's input]; contact material suited to dry circuits (for example gold) | contract §5.7 `K_REST_N` |

## 2. Contacts (each pole)

| Item | Requirement | Basis |
|---|---|---|
| Normal pulse current | **≥ 70 A peak**, decaying exponentially, two phases totalling ≤ 15.5 ms per shock, polarity reversing between phases | 65.7 A at 1,950 V / 25 Ω, 67.4 A at 2,000 V (`sim/bridge/notes.md` Q3) |
| Firmware-failure pulse | ≥ 70 A peak for ≤ 20 ms (phase caps 11.85 + 8.05 ms at the top of their bands) | HARDWARE_DESIGN §B13.3 rule 4 (v0.3) |
| Fault pulse | ≥ 120 A for ≥ 10 µs (an output short ended by the 80 A over-current trip: ≤ 97 A interrupted, simulated) | `sim/bridge/notes.md` Q2 |
| Repetition | 20 shocks in 1,300 s, then one every 2 minutes; ≥ 5,000 shocks over life [EST: 2,500 full charges (D-41) × 2] | D-41, D-53 |
| Contact resistance | ≤ 50 mΩ per pole at the pulse current, stable within ± 10 mΩ over life | §B6.5 internal-resistance budget (2 × 0.05 Ω) |
| Rest-side contacts | Carry the front-end currents only (≤ 1 mA); must withstand the 5 kV external-defibrillator test pulse that reaches them through the pads (≈ 98 mA peak through the 51 kΩ sense taps) | §B3.2 |
| Weld resistance | No weld after the fault pulse above; the maker states the current at which welding can occur | fault 27, fault 29 |

## 3. Insulation

| Item | Requirement | Basis |
|---|---|---|
| Across open contacts (rest ↔ worked side, each pole) | ≥ 5 kV DC for 60 s; working ≥ 2,200 V DC | §B7.2; capacitor rating 2,200 V (D-41) |
| Between poles | ≥ 5 kV DC for 60 s | the two pads are at opposite polarity during a shock |
| Poles to coil and to the position contact | ≥ 5 kV DC for 60 s; the position contact and coil are at ground potential | `K_REST_N`, `K_VCOIL` are low-voltage nets |
| Insulation resistance | ≥ 10 GΩ at 2,500 V DC between all isolated parts, at 25 °C, 93 % RH conditioning [EST] | front-end leakage (§B7.4) |
| Creepage and clearance | stated by the maker; to be checked against IEC 60601-1 Table 12 and IEC 60601-2-4 once bought [UNVERIFIED] | §B12 |

## 4. Coil and timing

| Item | Requirement | Basis |
|---|---|---|
| Nominal coil voltage | 12 V | 12 V coil supply on sheet 14 (WP-H §3.5) |
| Pick-up (must-operate) | ≤ 8 V at 70 °C | WP-H §3.5 (economiser peak 0.857 A) |
| Drop-out (must-release) | ≥ 0.5 V and ≤ 5 V | economiser hold current 0.496 A must stay above drop-out |
| Coil resistance | ≥ 12 Ω at 25 °C (≤ 12 W at 12 V) | coil supply sized for 12 Ω, 15 Ω hot |
| Coil inductance | stated by the maker (the design assumed 60 mH [EST]) | `sim/logic-relay/notes.md` §3 |
| On-time | Full voltage for ≤ 100 ms, then hold at 25–30 % power; duty ≤ 10 % | the sequencer cuts the coil 83.7–96.4 ms after it is first energised (simulated, `sim/logic-relay` §1.3) |
| **Operate time** (coil reaching pick-up → worked contacts closed and position contact open, including bounce) | **≤ 10 ms**, −20 to +65 °C | with the pre-charged coil supply the simulated transfer is 15.9 ms from `K_EN` assuming 10 ms of mechanics [EST]; the firmware then waits 14 ms; total must stay inside 40 ms |
| Release time (coil off → rest contacts closed), with ≈ 24 V suppression | ≤ 15 ms | §B7.2; firmware `relay_return_timeout_ms` 50 ms |
| Position-contact bounce | ≤ 1 ms in total, each bounce ≥ 10 µs apart or merged | the sequencer's filter ignores < 10 µs blips and restarts its 10 ms count on any longer one (`sim/logic-relay` §1.4) |
| Behaviour under 1.5 m drop | No contact transfer longer than 10 µs at rest under the drop shock [EST] | REQUIREMENTS drop target |

## 5. Life, environment, size

| Item | Requirement | Basis |
|---|---|---|
| Mechanical life | ≥ 50,000 operations [CALC: 10 years × (365 daily tests + ≈ 1,000 switch-on tests under D-62) ≈ 13,650, plus rescues, × 3 margin] | D-15, D-62 |
| Electrical life | ≥ 5,000 shocks as in §2 | §2 |
| Operating temperature | −20 to +65 °C | §B7.2; D-48 |
| Storage temperature | −40 to +70 °C | D-48 |
| Size and mass | ≤ 15 cm³ and ≤ 30 g [EST] | weight budget (D-43, Q-56) |
| Sealing | flux- and wash-tight, or sealed | assembly cleanliness |

## 6. Evidence to request from the maker

1. Datasheet and outline drawing with measured mass, pin-out and the position contact's construction (how it is
   linked to both poles).
2. Pulse-current test report at the §2 currents and repetition, with contact resistance before and after.
3. Dielectric test report for every §3 item, and insulation resistance after humidity conditioning.
4. Operate, release and bounce times measured at −20, +25 and +65 °C, with the coil at 8 V and 12 V.
5. Coil resistance and inductance (pulled-in and released).
6. Weld test: the current at which a contact welds.
7. Life test report (mechanical and electrical).
8. Drop and vibration results.
9. Quality certificate (ISO 13485 preferred) and a commitment to notify product changes.

## 7. Candidates looked at

Gigavac G17 (designed as a defibrillator relay; coil 12 V, 12 Ω, 100 ms maximum on-time from a search extract) and
G12-class parts: neither publishes a pulse-current rating for the contacts or a position contact linked to both
poles (WP-H §4). They are starting points for a maker's quotation, not qualified parts.
