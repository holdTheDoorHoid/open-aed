# Second independent review of the hardware design (revision 0.3)

| | |
|---|---|
| Reviewed | `docs/HARDWARE_DESIGN.md` v0.3; `hardware/aed-main/contract.md` issue 3; the sixteen pin tables in `hardware/aed-main/specs/` (read in part: sheets 08, 09, 10, 11, 12 and 13 group notes and the parts named below); `capture-notes/01-rev3-integration.md`, `rev3-changes.md`, the WP-D, WP-F and WP-G "Revision 0.3" notes; the simulation folders under `hardware/aed-main/sim/`; `firmware/REV3_NOTES.md` and `aed-core` (`config.rs`, `rescue.rs`, `selftest.rs`, `supervisor.rs`, `interlock.rs`, `hal.rs`), `aed-sim` (`plant.rs`, `hw.rs`); the first review `docs/reviews/2026-11-27-hardware-design-review.md` |
| Date | 28 November 2026 |
| Reviewer | an independent reviewer who did not write the design (AI-assisted) |

> **NOT FOR CONSTRUCTION — unreviewed design.** This review covers a design that has **not been built or measured**. Its simulations use hand-built models. **This review was written with AI assistance. It is not a review by a qualified high-voltage or medical-device engineer and it does not satisfy SAFETY.md §2a.** It finds errors on paper; it cannot show that anything is safe. Nothing here is an instruction to build, power or test anything, and nothing here is safe or ready for use on a person or animal (SAFETY.md, CLAUDE.md rule 2).

Severity, as in the first review: **Blocker** = the design as captured cannot do its job or is unsafe; fix before anything else depends on it. **Major** = fix before the design is called complete. **Minor**.

Every number below is tagged: **[SIM-R2]** = simulated by this reviewer (the files are in the reviewer's scratch directory, not in the repository; how to reproduce is stated with each one), **[CALC]** = arithmetic shown here, **[DS]** = read in a maker's datasheet (sources in §5), **[SPEC]** = read in the pin tables, **[REPO]** = a figure the project's own documents give.

---

## 1. Plain-language summary for the owner

**What I was asked to do.** Look for mistakes in the revised circuit design that nobody has found yet, especially where one sheet's circuit meets another's and where the circuit meets the firmware.

**What I found, in one sentence.** The circuit's individual pieces have been worked over carefully, but **when the firmware and the circuit are put together as drawn, the device would not deliver an adult shock**, for two separate reasons that the existing tests could not see because the computer model of the device is kinder than the real circuit.

**Reason 1: nothing in the circuit holds the capacitor at the right voltage the way the firmware assumes.** The firmware switches the charger off when the voltage "stops rising by itself", as if the charger had its own accurate stopping point. The circuit's adult charger has no such stopping point: it keeps charging until a safety limit trips. The design document says the main controller must switch it off on its own voltage reading instead. If it does, the two controllers' voltage sensors, each allowed an error of about 1 %, can disagree by more than the supervisor accepts, and the supervisor then refuses to permit the shock. I checked this in the project's own simulator: with sensors at the edges of their allowed error, no shock.

**Reason 2: the capacitor now leaks charge about eight volts a second.** Revision 0.3 added resistors that spread voltage evenly over the discharge transistors. They drain the capacitor, so the charger has to top it up several times a second. Compression sensing is switched off every time the charger runs, and the firmware refuses to act on a "shock" decision made while compression sensing was off. Result in the project's own simulator, with only the leak changed to the real value: the standard adult cardiac-arrest case gets **no shock at all**, and the child case also fails at the leak I calculated. The design's leak figure was also too low: it left out three resistor chains.

**Other important findings.** The self-test that is meant to prove the shock time limits are right accepts limits up to about two milliseconds off, so a drifted limit that would cut real shocks short can pass. If a bottom output switch fails short in the middle of a shock (the most likely moment for it to fail), the top switch has to break the full shock current and the patient relay may open while current is still flowing; nobody has worked this case out. The new latching over-voltage protection, combined with a firmware rule that keeps that fault until power-off, means one trip (including one caused by Reason 1) ends all shocks for the rest of the rescue. Both controllers run the same software rules for "the capacitor is empty" and "the readings agree", so a software mistake there is not caught by having two controllers. The main high-voltage switch and the freewheel diode may be going out of production.

**What I agree with.** The 26 known problems (J-1 to J-26) are real; I think three of them are more serious than listed (J-4 above all). The first review's findings are mostly fixed on paper; a few are re-opened (table in §4).

**What it means for you.** None of this needs a decision from you yet except one: the "how accurate must the voltage sensors be" decision (D-28) and the "compression readings while charging" question (Q-58) are now tied together and need to be decided with these findings in front of you. The fixes are on paper and are described below; most are firmware or small circuit changes.

---

## 2. Verdict

**Not ready.** Revision 0.3 is a careful sheet-by-sheet design, but two Blockers sit exactly at the circuit/firmware boundary and stop every adult shock in simulation. Neither is a deep circuit problem; both come from the firmware simulator having a better charger and a smaller leak than the circuit has, so 578 passing tests gave false comfort. Fix S-1 and S-2 (and re-run the whole firmware suite against a plant that models the real charger and the real standing load) before a fourth revision of anything else.

Counts: **2 Blocker, 7 Major, 8 Minor.**

---

## 3. Findings

### Blockers

#### S-1 — Blocker — The adult charger has no set point, but the firmware relies on one; regulating on the main reading instead makes the supervisor refuse in-spec units

**Sheets / parts / nets:** 10 (U1001 LT3751, FB MOSFET, `CHG_RUN` → `CHARGE_EN` → CHARGE pin), 07 and 08 (the two voltage chains), firmware `aed-core/src/rescue.rs` (`ChargeControl::decide`), `supervisor.rs` line 1172 (`is_at_plan`), `config.rs` (`at_plan_percent` 1, `at_plan_abs_volts` 12); simulator `aed-sim/src/plant.rs` `charger_limit()`.

**Sequence.**
1. HARDWARE_DESIGN §B4.4: in adult mode FB is grounded, so the LT3751 is in pure charge mode. It keeps switching until CHARGE falls or the primary-side trip fires at 2,116–2,316 V [REPO]. "The main controller regulates by switching `CHG_RUN` on its own voltage reading" (§B4.4, F-3.2).
2. The firmware does not do that. `ChargeControl::decide` switches the charger off only when the reading is above the bottom of the at-plan window **and has not risen by 2 V for 100 ms** (`CHARGE_SETTLED_MS`). Its own comment says "The charger regulates to its own set point (its accuracy is the charger's, not this sensor's)". `hal.rs` line 277 says the opposite ("The firmware stops every normal charge on its own reading").
3. The simulator's charger (`plant.rs` `charger_limit`) drives the **true** capacitor voltage exactly to the target (`charger_set_point_gain` 1.0). That is why the firmware works in simulation.
4. On the drawn circuit the reading never "stops rising" at the plan. It rises at about 205 V/s [REPO §B4.4] until the firmware limit (2,070 V reading), the latching comparator (2,073–2,142 V) or the primary-side trip. Each of those empties the capacitor. Firmware as written + circuit as drawn = **every adult charge overshoots and is dumped**; with S-5, the first such trip also ends the rescue.
5. Suppose instead the board layer implements `set_charger(Some(V))` by switching `CHG_RUN` on the main reading, as §B4.4 intends. Then the true voltage carries the main sensor's error (±(1 % + 7 V), D-28). The supervisor then checks **its own** reading against the plan with only ±(1 % + 12 V) = ±31.5 V at 1,950 V. Two in-spec sensors at opposite edges differ by up to 2 × 26.5 = 53 V. The supervisor refuses.

**Evidence.** [SIM-R2] The project's own scenario `adult-vf`, run in a scratch copy of the firmware with a new test file only (no source changed). To emulate regulation on the main reading I set the plant's set-point gain so that the main sensor reads exactly the plan:

| Sensors (main / supervisor) | Charger model | Result |
|---|---|---|
| +1 % +7 V / −1 % −7 V | repository (true voltage exact) | 1 shock, 149.9 J |
| same | regulated on main reading (gain 0.98655) | **no shock**; supervisor refusal `VoltageNotAtPlan`; true voltage 1,924 V |
| −1 % −7 V / +1 % +7 V | regulated on main reading | **no shock**; `VoltageNotAtPlan`; true 1,977 V |
| ±0.5 % ±3.5 V, opposite | regulated on main reading | 1 shock, 147.8 J |

So the at-plan window works only for sensors at about half their D-28 specification. REV3_NOTES §3.2 records that the firmware team saw this ("a first version stopped the charger when the main controller's reading reached the plan… the supervisor then refused") and moved to a charger set point that the circuit does not have.

**Proposed fix (needs an owner decision, because it re-opens D-28).** Pick one and change both sides together:
- (a) **Widen the supervisor's at-plan check** to two sensor allowances plus hysteresis plus droop: about 2 × 26.5 + 3 + 17 ≈ 73 V at 1,950 V [CALC; droop from S-2]. Re-do finding F-9 (child overshoot) with that window, because it lets a main sensor drift go unseen by the supervisor for longer.
- (b) **Tighten the sensors** to about ±(0.5 % + 3.5 V) each, which the half-spec row shows works. That is the thin-film-string question (owner question 4) made harder.
- (c) **Give the charger an independent set point:** a third divider and comparator on sheet 10 that ends CHARGE at the plan, used only for regulation. Neither controller's sensor error then enters the true voltage, and the supervisor's present window is right. This costs parts but keeps D-28 as it is.

In every case, change `plant.rs` so the simulated charger is regulated the way the circuit is (on the main reading, or on the new comparator with its own tolerance), and re-run the suite. Correct the contradictory comments in `rescue.rs` and `hal.rs`.

#### S-2 — Blocker — The standing drain on the capacitor plus compression blanking stops every adult shock in simulation; the drain is also larger than the design says

**Sheets / parts:** 11 (sharing strings R1107–R1114 and R6115–R6122, per-device R6107–R6114, bleeder, sense strings), 10 (FB string, 14 × 2.87 MΩ), 13 (R1370, R1371 film-capacitor balancing), the capacitor's internal 10 MΩ (D-65); firmware `rescue.rs` (top-up "as soon as the reading sags below where it stopped", 100 ms blanking after each burst, "a 'shock' answer reached while blind does not count").

**Sequence.**
1. With the capacitor held at the plan, the leak makes the reading sag 1 V in about 0.12 s [CALC: 1 V ÷ 8.3 V/s].
2. The firmware tops up after every 1 V sag and blanks compression readings while the charger runs and for 100 ms after (FW-13). So the chest is "not seen" for most of the time the capacitor is held.
3. Each "shock" decision while blind is discarded (REV3_NOTES §3.2). The analysis reaches its 12 s limit and ends "unable to analyse"; the device goes to CPR, and the same thing happens at the next analysis.

**Evidence.** [SIM-R2] Scenario `adult-vf`, ideal sensors, only `bleed_ohms` changed:

| Standing load | Adult VF (80 Ω) | Child VF (60 Ω) |
|---|---|---|
| 50 MΩ (repository) | shock at 11.3 s after pads on | shock |
| 20 MΩ | shock at 11.3 s | — |
| 10 MΩ | shock at 14.3 s (analysis 11.0 s of 12 s) | — |
| 5 MΩ | **no shock** in 170 s | — |
| 2.8 MΩ (design's figure) | **no shock**: "Shockable" every second, then `AnalysisInconclusive` at 12 s, twice | shock, analysis 12.0 s (at the limit) |
| 2.35 MΩ (this review's figure) | **no shock** | **no shock** |
| 2.35 MΩ, blanking switched off | shock, 148.6 J from 1,941 V | — |

The last row shows the blanking is the cause.

**The design's drain figure is too low.** §B5.6 gives 3.90 MΩ on the board (2.80 MΩ with the capacitor's own 10 MΩ). As drawn [SPEC]:
- each sharing-string element is two 10 MΩ parts in parallel (5 MΩ, not 4.7 MΩ): 20 MΩ per string, 13.3 MΩ per stack with the 40 MΩ per-device chain;
- the charger's FB string (14 × 2.87 MΩ = 40.18 MΩ, sheet 10, R from `HV_CAP_P` to `HV_FB_1`…) is always across the capacitor, also in adult mode where FB is grounded;
- the local film capacitor's balancing resistors R1370 + R1371 (2 × 10 MΩ = 20 MΩ, sheet 13) are across `HV_CAP_P`–`HV_RTN`.

[CALC] Conductances in µS: stacks 2 × 0.0750, bleeder 0.0502, main chain 0.0249, supervisor chain (20 × 1.99 MΩ) 0.0251, FB string 0.0249, film balance 0.0500 → 0.325 µS = **3.08 MΩ on the board, 2.35 MΩ with the capacitor's 10 MΩ**. τ = 235 s; droop **8.3 V/s at 1,950 V, 8.5 V/s at 2,000 V** (not 7.1); standing loss **1.72 W** at 2,008.6 V (not 1.44 W). The 10 MΩ / 3.3 nF per-device parts were described as "essential", and hot depletion-device leakage (IXTT2N170D2 I_DSX(off) ≤ 25 µA at 25 °C rising to ≤ 500 µA at 125 °C [DS]) adds to it, so the drain at 60 °C will be higher still; it must be measured.

**Proposed fix.** Firmware first, because the circuit cannot easily get back to 50 MΩ:
- Top up on a **fixed schedule** in short bursts (for example one burst a second, 8–10 V each, about 40–50 ms of switching), not after every 1 V. Blank only the burst plus a measured filter tail. Allow the sawtooth inside the at-plan window, which S-1's fix has to widen anyway.
- Replace "a shock answer reached while blind does not count" by "the chest must have been seen for at least X ms in the last analysis window", so a blind fraction of 10–20 % does not cancel every decision.
- Measure the real ADS1292R demodulator recovery after a burst; the 100 ms tail is an estimate (FW-13 [EST]).
- In the circuit: drop the film balancing resistors to 2 × 47 MΩ, move the FB string behind a switch that is open in adult mode (or re-value it to 200 MΩ class), and correct §B5.6, F-12 and the plant's `bleed_ohms` to the drawn value.
- Re-run every scenario with the corrected plant, and add the table above as a regression test.

J-4 should be raised to Blocker and merged with this finding.

### Majors

#### S-3 — Major — The self-test accepts per-phase caps about ±2 ms from nominal (and the overall cap ±5 ms), so it does not prove the bands D-63 relies on

**Where:** `selftest.rs` lines 1001–1040 (`Step::TimeCap`); `config.rs` `hardware_phase1_cap_ms` 11, `hardware_phase2_cap_ms` 8, `hardware_time_cap_ms` 25; §B14.5 T-8, T-9; sheet 12 phase caps (10.75–11.92 ms and 7.31–8.10 ms, J-20).

**Sequence.** T-8 accepts the phase-1 cap if the sequence ended between `nominal − slack` and `nominal + slack`, with nominal 11 ms and slack 2 ms: **9–13 ms**. For phase 2 the nominal is 8 + 2 = 10 ms from the start of a 1 ms + 0.5 ms + phase-2 test, so the phase-2 cap is accepted at **6.5–10.5 ms**. T-9 accepts the overall cap at **20–30 ms**. Measurement is also in 1 ms ticks from the first tick that sees `Active`.
- A phase-1 cap that has drifted to 9.2 ms passes the daily test and then cuts every legitimate long phase 1. The design table's longest phase 1 is 9.01 ms (175 Ω) and firmware allows up to 10 ms. The shock ends as a fault for high-resistance patients.
- A phase-1 cap at 13 ms passes and weakens D-63's frozen-controller bound. 181 J was calculated with the cap at 11.85 ms.

**Fix.** Check against the hardware bands themselves, measured from the hardware edge: phase 1 accepted 10.6–12.0 ms, phase 2 7.2–8.2 ms, overall 23.5–26.5 ms, with a timer of 0.1 ms resolution started on `SEQ_ACT_M_N`. Move the bands into `config.rs` from `LatchTiming` instead of separate integer-millisecond constants, so the two cannot drift apart. Add a test in `aed-sim` that injects a cap at 9.5 ms and expects T-8 to fail.

#### S-4 — Major — A low-side switch that fails short during a phase leaves the high side to break the full shock current, and the patient relay can then open under current; not analysed anywhere

**Sheets / parts:** 13 (QLB/QLA MSC080SMA330B4N, their UCC5350MC drivers; QHA/QHB with the carrier-loss detector and PNP turn-off), 12 (HSK kill latch, `IDLE_F`, `RUN_D1M` coil-off at +0.89–1.63 ms), 14 (K1401).

**Sequence.**
1. Phase 1 is running at 30–67 A [REPO BR Q3]. QLB fails short, or its driver output sticks high. Power switches most often fail under exactly this stress.
2. The firmware ends phase 1; `G_LB` falls; QLB keeps conducting. After `LB_OFF10` (9–20 µs) the idle comparators see current with no low side enabled → `IDLE_F` → `FLT` and the HSK latch → `G_HA` removed within about 60 ns.
3. QHA now turns off **under the full phase current at 1.0–1.6 kV**, its gate falling over 3.9–7.0 µs [REPO R3 §3]. BR Q1 simulated only the case where the low side is already shorted at the start and QHA is still turning on. [CALC, rough] 0.5 × 1,300 V × 50 A × 3 µs ≈ 0.1 J in QHA's linear region. On the datasheet-extrapolated Zθ this is far above 150 °C; on the "physical" estimate (3–4 mJ/K) it is about +25–33 K. The project's two thermal estimates disagree on whether QHA survives.
4. If QHA fails short as well, nothing in the design can stop the current. The rest of the stored energy goes monophasically into the patient (up to about the whole 190 J × R/(R+5 Ω)).
5. Meanwhile the turn-off order switches the relay coil off at +0.9–1.6 ms. K1401 then opens **with current flowing**, against its "cold-switched" requirement (patient-relay.md). Its at-rest contact closes onto a pad that may still be at kilovolts, which puts the full shock across the front-end protection taps.

**Evidence.** Reasoning from the drawn logic [SPEC 12 groups `faults`, `hsk`, `turnoff`] and the existing simulations' figures. Not simulated: no netlist has a low side that fails short mid-phase.

**Fix.**
- Simulate it (a variant of `rev3-wpg/s1_shorted_lowside.cir` with the short applied at 2, 5 and 9 ms into phase 1, 25–175 Ω).
- Add a **current-zero interlock on the relay**: `K_EN` must stay on while the shunt current is above, say, 1 A, or while `OUT_A_TAP`/`OUT_B_TAP` read above about 100 V, so the relay never opens under current. Bound it by the coil cap.
- Add this case to §13 and to the relay requirement as an abnormal-break rating. If the relay cannot be specified to break it, say so in §13.

#### S-5 — Major — The latching over-voltage comparator plus the firmware's "fault until power-off" turns any single trip into the end of the rescue

**Sheets / parts:** 08 (U803 TLV9031, R805/R806 100 kΩ, `OVP_SENSE` with no filter, J-22), 11 (Q1111 and the force optos), firmware `supervisor.rs` `run_ovp` and REV3_NOTES §3.9 ("The supervisor's over-voltage fault stays until power-off").

**Sequence.** With the capacitor at the plan, `VCAPS_BUF` ≈ 1.946 V, so `OVP_SENSE` = 0.973 V. That is **78 mV** below `OVP_REF` (1.0512 V) [CALC from SPEC 08]. U803 is a fast comparator and `OVP_SENSE` is a 50 kΩ node with no capacitor. A transient of more than 78 mV, or a real trip, does three things:
- it latches `OVP_TRIP`;
- it forces both stacks on, which empties the capacitor;
- under the firmware's policy it latches a supervisor fault until power-off, so there are no further shocks in this rescue.

Real trips are not rare either: with S-1 unfixed, every adult charge runs into L1 or L2. Revision 0.3's latch fixed the "does not empty fully" problem (I-22) and created this one.

**Fix.**
- Add an RC on `OVP_SENSE` (for example 10 kΩ / 1 nF from the R805/R806 junction, inside the latch loop), so the latch ignores sub-10 µs events.
- In a rescue, let the supervisor reset the latch once both readings are empty and the cause is understood: allow one reset per rescue, recorded, after a trip with both readings below the trip. A trip with a reading above it stays latched.
- Keep "not ready until a self-test passes" for standby.

#### S-6 — Major — The two controllers share the same software rules, so the "independent second check" is hardware-diverse only; and the real-permit self-test (D-62) has no hardware voltage gate

**Where:** `aed-core/src/config.rs` (`reads_empty`, `readings_agree`, `sensor_allowance`, `is_at_plan`) used by both `selftest.rs`/`rescue.rs` (main) and `supervisor.rs` (supervisor). Sheet 09: `LOWV_OK` gates only `TEST_MODE` (SW2b), not SW2a or the real permit. §B14.5, FW-16.

**Sequence.** D-39 makes the processors diverse; the code deciding "empty", "agree" and "at plan" is one crate compiled twice. A single wrong constant or arithmetic slip in `Config` (for example `reads_empty` or the allowance function) is a common-cause fault in both controllers. D-62's real-permit steps are the clearest exposure. Both controllers' firmware readings saying "< 50 V" is the only thing standing between a real permit, the relay moving and, in T-5, the bridge switches being commanded. `LOWV_OK`, the hardware comparator that protects test mode, is not in that path. SAFETY.md §4 asks for "an independent second check on the shock path".

**Fix.**
- Put a hardware gate on the dry run: a new sheet-12 input `DRY_RUN` (from the main controller) that, when high, requires `LOWV_OK` for `G_L*`. T-5 must raise it; a real rescue leaves it low.
- Alternatively, run T-5 in test mode with a separate relay-enable path that also requires `LOWV_OK`.
- In software: give the supervisor its own implementation of the three safety predicates (independently written, with its own tests), or at least its own constants table checked against the main's at link-up.

#### S-7 — Major — Several "survives" conclusions rest on thermal and switch models that cannot decide them; start temperature 55 °C is below the device's own ready range

**Where:** `sim/bridge/notes.md` "Models", "Junction temperature: two estimates"; `sim/rev3-wpg/notes.md` (start 55 °C; 25.4 → 29.2 mJ, 139 → 150 °C sensitivities); HARDWARE_DESIGN §B6.3, §13 rows 18, 18a, 19, 24a.

**What is wrong.**
- The SiC model is behavioural. It has no saturation current at kilovolts (230 A assumed, 170 A sensitivity), no avalanche, turn-off SOA or short-circuit model, and Zθ below 7 µs is extrapolated as √t. The two thermal estimates differ 2–6×. Every row of §13 that says a switch "survives" a microsecond event (18, 18a, 19, 24a and S-4) can be read either way, and the notes say so. The design summary ("all three are fixed on paper and checked again in simulation") and §V3 ("fixed in simulation (11–13 mJ)") claim more.
- The start junction temperature is 55 °C everywhere. D-48 lets the device stay ready at 60 °C ambient for 8 hours and charge up to 60 °C, and 20 shocks in 1,300 s heat QHA by up to about 40 K per shock [REPO BR Q3: ≤ 98 °C from 55 °C]. Starting at 65–70 °C moves the J-8 worst case from 150 °C to 160–165 °C on the pessimistic estimate.
- J-1's temperature error has been corrected in the netlist files: every `option temp` now follows `reset`. I confirmed the pitfall itself in ngspice 42 [SIM-R2: a diode at 1 mA reads 0.655 V at 27 °C, 0.578 V with `option temp=70` after `reset`, and 0.655 V again when `reset` follows `option temp`]. But HARDWARE_DESIGN v0.3 still quotes the uncorrected cold/hot figures (§B5.4 "−5.9 to −6.1 V", "1.4–22 ms", "4.95 / 5.62 / 6.12 V", "19–33 ms"). The `out/` folders are not committed, so a reader cannot tell which run produced a figure.
- The other pitfalls in `sim/rev3-wpf/notes.md` are "a failed operating point silently replaced by a pseudo-transient" and "`foreach` variables not reaching `alterparam`". These were found only in WP-F. I re-ran `logic-relay/sense.cir`: its `alterparam` loops did take effect, because the results vary with every parameter. The other folders have not been checked for either pitfall.

**Fix.**
- Re-state every "survives" row as "undetermined: needs Zθ below 10 µs from Microchip or a low-voltage-scaled pulse measurement". Use the pessimistic estimate for design decisions.
- Re-run the shorted-switch cases from 70 °C.
- Commit a short `out/summary.txt` with a run date and netlist hash for every figure HARDWARE_DESIGN quotes, and re-issue §B5.4 from the corrected runs (J-1, J-2).
- Add an assertion to every netlist that checks the convergence state, and one that checks that each `alterparam` took effect (print the parameter back).

#### S-8 — Major — The standing-load, droop and loss figures used across the design, the change order and the firmware are wrong

This is the accounting half of S-2, listed separately because several other documents carry the wrong figure: §B5.6 (3.90 / 2.80 MΩ, 1.44 W, 17 min), §B4.4 (7.1 V/s, "restarts about every 0.4 s"), F-12, change order S-12 and FW-13 (21 V over the countdown), D-65's "about an hour" (capacitor alone, still right), and J-4. As drawn: **2.35 MΩ, 8.3–8.5 V/s, 1.72 W, 14.5 min to 50 V** [CALC: 235 s × ln(2,008.6/50)]. The phrase "restarts about every 0.4 s" assumes the 3 V hysteresis of §B4.4, which the firmware does not use (it uses 1 V). With 3 V the restart interval is 0.35 s. Hot leakage of the 2N170D2 and GP02-40 (≤ 50 µA at 100 °C at 4 kV [DS]) will raise the drain further.

**Fix:** correct all of them. Make the standing load a contract quantity with a maximum, so any later sheet that adds a resistor across `HV_CAP_P` has to justify it.

#### S-9 — Major — The main output switch and the freewheel diode may be at end of life

- **MSC080SMA330B4N** (QHA, QHB, QLA, QLB): DigiKey lists it as Active and Arrow as Obsolete; RS shows a related part as discontinued [§5, rows 4–6]. The whole bridge model, the thermal estimates and the 80 A trip are fitted to this part. Tj max is 150 °C (confirmed).
- **GC05MPS33J** (D1301, D1302): search snippets indicate a last-time-buy date of 31 May 2026 and an "Obsolete" flag on some DigiKey listings (not confirmed on a Navitas page) [§5, row 23].
- **GP02-40** (J-10): its status could not be read.

**Fix:** ask Microchip and Navitas for lifecycle statements now. Name a second source for each (G2R120MT33J is already named for the switch, with no model). Re-fit the bridge model if the switch changes; the margins in S-4 and S-7 depend on it.

### Minors

#### S-10 — Minor — "Hardware completes the planned pulse" overstates D-59

The plain-language summary says hardware "keeps the output stage powered until the planned pulse ends, whatever the two controllers do next". The hardware keeps the **rail** up. The phase lengths still come from the main controller's timer outputs `CMD_LB`/`CMD_LA`.
- A main watchdog reset (≤ 50 ms, §B2.1) or a `3V3_MAIN` brown-out mid-phase pulls `CMD_*` low through their 100 kΩ defaults. That ends phase 1 early, and no phase 2 follows (the sequencer waits in the gap until the 25 ms cap): a truncated monophasic under-dose.
- A main frozen high runs to the caps.

The owner is asked to confirm D-59 (Q-50) on the basis of the summary, so the summary should say this. **Fix:** reword §B13.2 and the summary; add "main controller reset mid-phase" to §13 (result: truncated, `BR_FAULT_N` by the overall cap, under-dose).

#### S-11 — Minor — The firmware verification claims more than the plant supports

REV3_NOTES says "A real shock and the full periodic self-test work at all four corners". The plant has an exact-set-point charger (S-1), a 50 MΩ leak (S-2) and relay figures that are estimates. **Fix:** state those three limits next to the claim, and add the S-1 and S-2 tables as tests.

#### S-12 — Minor — Low-side gate supply: unregulated MGJ2 against a 23 V gate rating

The MGJ2 output is unregulated: line regulation 1.0 typ / 1.2 max %/%, load regulation given only as envelopes, 10 % minimum load [DS]. The MSC080SMA330B4N's VGS maximum is +23 V (+25 V transient) [DS]. With the preloads, sheet 13 gives the "normal rail 19.5–22 V" [SPEC], which leaves about 1 V to the gate rating at the high-input, light-load corner. The figure rests on an [EST]. **Fix:** read the MGJ2 envelope at 5.25 V input and 10–15 % load. If the top exceeds 22 V, add a 22 V clamp on `LSD_P20V`, or an over-voltage trip in the `LSD_OK` comparator pair.

#### S-13 — Minor — DC current through the patient while a high side is on before phase 1 or after a refused start

With the relay transferred and QHA on (the 100 µs lead), pad A sits at the capacitor voltage and pad B at `HV_RTN` through the 1 MΩ output divider (sheet 13 `osense`): about 2 mA DC through the patient [CALC: 1,950 V ÷ 1 MΩ]. That is normal for the 100 µs lead. If the start is refused (START_OK false at the `CMD_LB` edge), firmware can leave `CMD_HA` and the relay on until `sequence_start_timeout_ms` (80 ms). **Fix:** firmware drops `CMD_HA` within 1 ms if `SEQ_ACT_M_N` has not gone low within 0.2 ms of the `CMD_LB` edge. Add this to F-4.

#### S-14 — Minor — Comparator offset bands use a 25 °C figure

The TLV3501's ±6.5 mV maximum offset is specified at 25 °C only. Over temperature, add about ±5 µV/°C [DS]. The fast-idle band (2.6–3.5 A) and the 80 A band (74–86 A) are slightly wider than stated. **Fix:** re-state both bands. This matters for J-8, which has no margin.

#### S-15 — Minor — Depletion-device ratings misquoted

IXTT2N170D2 [DS, DS100418B]:
- "≥ 2 A" (§B5.5) is I_D(on), a pulsed test minimum at V_DS = 50 V, not a continuous rating.
- The gate rating is ±20 V continuous; ±30 V applies to transients only.

The 15 V gate TVS keeps normal operation inside both. J-11's 73 V is far outside. **Fix:** correct §B5.5. Obtain FBSOA for the release-failed case, which puts 26.6 J into one device [REPO WP-F].

#### S-16 — Minor — SW4 is reverse-biased by SW5 outside the TPS22918's recommended conditions

The TPS22918 has no reverse blocking; current flows from VOUT to VIN through the body diode, and the datasheet requires VOUT ≤ VIN [DS]. Sheet 09 accepts this deliberately (assumption A-3): SW4's VIN node (`SW_N3`) sits 0.7 V below `BRIDGE_5V` through its 10 kΩ bleed while SW5 holds the rail. **Fix:** get TI's statement on repeated reverse body-diode conduction at about 0.5 mA, or add a Schottky in series with SW4.

#### S-17 — Minor — Test-permit voltage limit disagrees with §F

`config.rs` `test_permit_max_voltage` is 60 V. F-8.1 and FW-16 say the supervisor gives a test permit only below 50 V. The hardware `LOWV_OK` (60 V) is still stricter in practice, so this is a documentation and firmware consistency fix. Make it 50 V, or change F-8.1 and D-62 together (J-16).

---

## 4. Opinion on the known problems and the first review

### 4.1 J-1 … J-26

| No. | Listed as | My view | Comment |
|---|---|---|---|
| J-1 | serious | **Agree, Major** | Netlists are corrected; HARDWARE_DESIGN figures are not. See S-7 |
| J-2 | serious | **Agree, Major** (availability at −20 °C) | Also: "held after power-up" reaches 56 ms at 1.65 kHz cold [REPO WP-F R3.4] against the firmware's `hold_engage_wait_ms` 35 ms |
| J-3 | serious | Agree, Major | Together with J-8 (no margin) and S-14, this is the weakest protection |
| J-4 | serious | **Under-rated: Blocker** | See S-2: no shock at all, not just reduced feedback |
| J-5 | serious | Agree, Major | The self-test proves "fires", not "fires at what level"; consider injecting a known level |
| J-6 | serious | Agree, Major | |
| J-7 | serious | Agree, Minor–Major | |
| J-8 | serious | Agree, Major; **worse than stated** | The 55 °C start is optimistic (S-7) |
| J-9 | serious | Agree, Minor (fail-safe) | |
| J-10 | serious | Agree, Minor | Supply chain; see S-9 for more serious parts |
| J-11 | serious | Minor | Needs a second fault (open TVS plus a transient) |
| J-12 | serious | **Major** | An 86 W load left on is a heat and battery hazard. One open resistor is a single fault |
| J-13 | serious | Agree, Major | LT3751 (rev A text read for this review): "LPRI must stay below 38 µs/[IPK(1/VTRANS + N/VOUT)]" in regulation mode, and a "runaway… overcharge" warning when VOUT exceeds the design value [DS] |
| J-14 | minor | Agree, Minor | |
| J-15 | minor | Agree, Minor | |
| J-16 | minor | Agree, Minor | See S-17 |
| J-17 | minor | Agree, Minor | |
| J-18 | minor | Agree, Minor | |
| J-19 | minor | Agree, Minor | |
| J-20 | minor | **Major together with S-3** | The bands are wider, and the self-test does not check them anyway |
| J-21 | minor | Agree, Minor | |
| J-22 | minor | **Major together with S-5** | |
| J-23 | minor | Agree, Minor | |
| J-24 | minor | Agree, Minor | Coating at 1.3–1.6 kV is likely mandatory |
| J-25 | minor | **Major** | The current-based switch checks are the only way D-61 finds a shorted low side (fault 18) before a rescue. At 0 V they find nothing, so R-14's fix is not achieved |
| J-26 | minor | **Major** | Sheet 12 is the most safety-critical logic in the device. Simulate it as a whole (all five latches, the caps, the HSK latch and the relay timers together) before trusting any timing claim |

### 4.2 R-1 … R-31 (first review)

"Confirmed" means the fix is present in the pin tables or firmware as far as I checked. It does not mean the fix works.

| No. | Status | Note |
|---|---|---|
| R-1 | **Confirmed in firmware** (`supervisor.rs` no longer re-checks pads or voltage after the start); **partly re-opened** by S-10 (summary overclaims) | |
| R-2 | Confirmed | Freewheel diode part lifecycle: S-9 |
| R-3 | Confirmed (gated push-pull) | |
| R-4 | Confirmed (latched caps; ACT self-holding) | Cap verification by self-test: S-3 |
| R-5 | Confirmed (split enables) | **Re-opened in a new form by S-1**: who regulates the adult charge |
| R-6 | Confirmed (two sensors, active-low "held" with pull-ups to "not held") | |
| R-7 | Confirmed (adult in charge mode, child FB ceiling) | Consequence: no adult set point (S-1) |
| R-8 | Confirmed (D-60 default adult) | |
| R-9 | **Re-opened** by J-1/J-2 (hold at a true −20 °C) | |
| R-10 | Confirmed | Latching side effects: S-5 |
| R-11 | Confirmed (64.9 kΩ bleeds; band) | |
| R-12 | Confirmed | Hot opening under current: S-4 |
| R-13 | **Partly re-opened** by J-25 | The tests run, but the current checks run at 0 V |
| R-14 | **Re-opened** by J-25 | |
| R-15 | Confirmed (two diodes, fusible R1004 still not a named part) | |
| R-16 | Confirmed for charger faults | The TPS25982 is rated only from TJ −15 °C [DS], against the −20 °C requirement (WP-E C6, O-16). I would list this as Major |
| R-17 | Not re-checked | |
| R-18 | Partly (J-24) | |
| R-19 | Confirmed as stated | Depends on the transformer and J-13 |
| R-20 | Open (O-16, WP-E C3) | |
| R-21 | Not fully re-checked | Debug-pin reset states on sheet 04 checked: no contract signal on a pin with a reset pull-up |
| R-22 | Confirmed in simulation | Subject to J-1 corner validity |
| R-23 | Confirmed | |
| R-24 | Confirmed (24.0 mV/A; 3.20 V) | |
| R-25 | Confirmed | Tj max 150 °C, no avalanche, SC or RBSOA data [DS] |
| R-26 | Not re-checked | |
| R-27 | Confirmed | |
| R-28 | Not re-checked in detail | |
| R-29 | Confirmed | |
| R-30 | Confirmed (2,070 V) | |
| R-31 | Confirmed | |

---

## 5. Arithmetic and values I spot-checked

**Arithmetic re-done [CALC]; all agree unless marked:**
- 190.1 J and 66.1 J stored.
- One-path dump 2,200 → 60 V in 1.62 s (τ = 0.45 s).
- Over-current trip 3.3 V × 10/206 = 160.2 mV → 80.1 A.
- Fast idle 66.4 mV ÷ 11 ÷ 2 mΩ = 3.02 A.
- Phase caps 113 kΩ × 100 nF × ln(1/0.3655) = 11.37 ms and 7.73 ms.
- OVP 2.5 × 15.6/37.1 × 2 × 1,002.0 = 2,106.7 V.
- `LSD_OK` 16.93 V.
- Test energy 0.18 J, 0.293 A.
- Child FB ratio 1,089.9.
- Window 1.47 MΩ × 100 nF × 1.022 = 150 ms.
- RDY 22.6 ms.
- `BRIDGE_3V3` hold-up 2.53 ms.
- **Standing load: does not agree** (S-8).
- **OVP margin at plan: 78 mV** (S-5).

**Datasheet values checked** (a research helper fetched the documents for this review and I read its extracts; maker PDF unless marked "mirror"):

| # | Part / value claimed | Found | Source |
|---|---|---|---|
| 1 | LT3751 FB 1.12/1.16/1.20, 1.19/1.22/1.25, 1.29/1.34/1.38 V; bias 300 nA | Confirmed (bias 300 nA max) | LT3751 rev A, octopart mirror (rev B/D not reachable) |
| 2 | LT3751 UVLO 1.225 V + 50 µA·R | Confirmed (48.5–51.5 µA) | same |
| 3 | LT3751 VCC 4.75–24 V | Confirmed | same |
| 4 | MSC080SMA330B4N RDS(on) 85/106 mΩ, IDM 107 A, VGS(th) 1.9 V min, Qgd 51 nC, IDSS 35 µA | Confirmed | DS00005914A (RS mirror) |
| 5 | same, Tj max | 150 °C | same |
| 6 | same, VGS max | +23/−10 V (+25/−12 transient) | same |
| 7 | IXTT2N170D2 1,700 V, VGS(off) −2…−4 V, 6.5 Ω, 25/500 µA | Confirmed | DS100418B (mirror) |
| 8 | IXTT2N170D2 "≥ 2 A", gate rating | **Pulsed I_D(on); VGS ±20 V continuous** | same |
| 9 | UCC5350MC UVLO falling 10.3–11 V, Miller clamp | Confirmed (on VCC2−VEE2) | TI SLLSER8J |
| 10 | TPS25982 temperature range | **TJ −15…125 °C specified** | TI SLVSEI3D |
| 11 | TPS22918 5.5 V, 2 A, QOD | Confirmed; **no reverse blocking** | TI SLVSD76C |
| 12 | TLV3501 ±6.5 mV, 4.5 ns | **At 25 °C only**; delay confirmed | TI SBOS321E |
| 13 | OPA838 300 MHz, ±220 µV | GBP confirmed; offset −230/+220 µV (−40…85 °C); minimum stable gain ≈ 6 | TI SBOS867E |
| 14 | TLV7031 3 µs, 335 nA | Confirmed (offset 8 mV max) | TI SLVSE13J |
| 15 | MMBFJ270 VGS(off) 0.5–2.0 V | Confirmed | onsemi MMBFJ270/D |
| 16 | MMSZ4687 4.09–4.52 V at 50 µA | Confirmed | onsemi MMSZ4678T1/D Rev 13 |
| 17 | TLP187 dark current 20 µA at 85 °C; CTR | Confirmed; CTR ≥ 1,000 % at 1 mA | Toshiba TLP187 Rev 6.0 |
| 18 | GP02-40 4 kV, 5/50 µA | Confirmed; status not determined | Vishay 88635 |
| 19 | ADS1292R 100 µA modulation maximum, 32/64 kHz, 128 kHz modulator | Confirmed | TI SBAS502C |
| 20 | MGJ2D052005SC regulation | **Unregulated; 10 % minimum load; 5.2 kV DC 1 s test** | Murata KDC_MGJ2_F01 (mirror) |
| 21 | INA240A1 gain 20, 2.7–5.5 V, swing VS − 0.2 V | Confirmed | TI SBOS662C |
| 22 | CDE 940C30P15K-F 0.15 µF, 3 kV | Confirmed; 27 × 46 mm each, no mass given, 216 A peak, full voltage to 85 °C | CDE 940C catalogue |
| 23 | GC05MPS33J 3.3 kV 5 A | Confirmed; IFSM 250 A at 10 µs; **lifecycle doubt** | GeneSiC Rev 22/Apr |
| 24 | BSC070N10LS5 100 V, RDS(on) at 4.5 V | 8.5 mΩ max at 20 A | Infineon Rev 2.2 |
| 25 | TPS629210 3–17 V, 1 A; TPS62130 3–17 V, 3 A, 100 % mode; TPS70933 to 30 V | Confirmed (TPS709 IQ 1.3 typ / 2.05 µA max) | TI SLVSGE0B, SLVSAG7F, SBVS186H |

Not reached (sites refused): MAX17320, MAX30009, UX-F5B, DPCR2512, HVCR2512, CRHV1206, 2N7002, BAS116, Gigavac G17.

---

## 6. What I did not review

- I did not read every line of the sixteen pin tables. I read the group notes and the parts named in the findings. I did not check sheets 01–07, 14 (beyond the coil enable), 15 and 16 pin by pin, or any footprint.
- I did not re-run the bridge, charger-storage, rev3-wpf or rev3-wpg simulations. I re-ran `logic-relay/sense.cir` and an ngspice temperature check, and ran the firmware simulator with plant parameters changed (S-1, S-2). The firmware runs used a scratch copy of the workspace with one added test file; no repository file was changed.
- Front-end defibrillation protection (R-28) and the external 5 kV test were not re-derived.
- Creepage and clearance, the IEC 60601-2-4 clauses and the patient-leakage limits were not checked: the standards are not in the repository.
- The compression and rhythm algorithms were not reviewed, except where blanking interacts with the hardware (S-2).
- Battery, dock, power-on logic and the MAX17320 configuration were not reviewed.
- Lifecycle statements (S-9) come from distributor listings and search extracts, not from the makers.

**The three things I am least sure of:**
1. **How the real board layer will implement `set_charger`** (S-1). If a hidden charger set point is intended, for example a firmware comparator interrupt outside `aed-core`, the first half of S-1 falls away. The second half (supervisor refusal with main-reading regulation) does not.
2. **Whether the 100 ms blanking tail and the "blind decisions do not count" rule are what the real device would need** (S-2). The no-shock result is in the project's own simulator with only the leak changed. A shorter measured filter tail could make it much less severe, but the firmware as written would still lose most decisions.
3. **The mid-phase low-side short** (S-4). Whether QHA survives breaking about 50 A, and whether the relay really opens under current, depends on figures that nobody publishes (sub-10 µs Zθ, relay break rating). The rough 0.1 J estimate could be several times off either way.
