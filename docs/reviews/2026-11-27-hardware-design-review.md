# Independent review of the hardware design specification and net contract

| | |
|---|---|
| Reviewed | `docs/HARDWARE_DESIGN.md` v0.1 and `hardware/aed-main/contract.md` (both dated 27 Nov 2026), `hardware/aed-main/hierarchy.toml`, against the firmware's hardware contract (`aed-core`: `hal.rs`, `interlock.rs`, `config.rs`, `supervisor.rs`, `selftest.rs`, `link.rs`, `rescue.rs`; `aed-waveform` design table) |
| Date | 27 November 2026 |
| Reviewer | an independent reviewer who did not write the design (AI-assisted) |

> **NOT FOR CONSTRUCTION — unreviewed design.** This review covers a design that has **not been built, simulated in circuit form or measured**, and that **has not been reviewed by a qualified high-voltage or medical-device engineer**. The review itself was written with AI assistance. It finds errors on paper; it cannot show that anything is safe, and it does not replace review by a qualified high-voltage and medical-device engineer. Nothing here is an instruction to build, power or test anything, and nothing here is safe or ready for use on a person or animal (SAFETY.md, CLAUDE.md rule 2).

**Words used here.** *Single fault*: one part or one controller failing in one way. *Consent*: one of the three conditions that must all be true before the bridge gets power (the main's request, the supervisor's permit, both discharge paths held off). *Phase*: one half of the two-part shock. *Freewheel path*: a route that lets the current in a coil keep flowing safely when a switch opens, instead of producing a voltage spike. *Avalanche*: a transistor breaking down because the voltage across it exceeds its rating; some parts are rated to survive it, many are not. *Latch*: a circuit that remembers an event until it is reset.

Severity: **Blocker** = fix in the specification/contract before capture of the affected sheets starts. **Major** = fix before the design is called complete; capture can proceed. **Minor**. **Note**.

---

## Overall judgement

The specification is careful, honest about what it does not know, and most of its arithmetic is right (the list of checked claims below is long). The architecture — two independent depletion-mode discharge paths, a three-consent supply rail for the bridge, slow high-side switches with the low side doing the timing, a patient relay that is cold-switched — is a reasonable starting point. But the shock path has **six problems that would make the captured circuit either dangerous or unable to shock**, and they sit exactly where the parallel capture packages meet, so they must be settled in the contract first. The most serious are: (1) the hardware turns the supervisor's permit into a *continuous* power supply for the bridge, while the firmware's supervisor withdraws the permit as soon as its own pad and voltage readings change — which they do the moment the patient relay moves and the shock starts — so shocks would be cut off at an uncontrolled point, with no defined order for turning the switches off; (2) the 30 µH protective coil has **no freewheel path**: the document's statement that its energy "returns to the capacitor through the high-side body diodes" is wrong, so every phase end would avalanche a silicon-carbide switch that has no avalanche rating; (3) the chosen high-side transformer driver (SN6505B) has a built-in soft-start of milliseconds on enable, so the high-side switches cannot turn on in the 50–100 µs the timing assumes, and the phase-2 switch cannot turn on inside the 0.5 ms gap at all; (4) the "non-retriggerable" 25 ms one-shot is a **retriggerable** part, so the hardware time cap does not hold against a timer left toggling by a frozen controller; (5) one signal drives both the charger's CHARGE pin and its supply switch, which races the LT3751's latched under-voltage fault and could stop every charge; (6) the "both paths held" signal is an open-drain wired-AND whose *released* state means "held", so an open comparator output reads as safe-to-charge and silently defeats D-25. The design also misdescribes the LT3751 FB pin (it regulates; it does not stop), which collides with the adult operating window once tolerances are included. **Capture can start now for power (WP-A), controllers (WP-B), UI and sensing (WP-C) and the two front ends (WP-G sheets 14 and 15). Interlock and storage (WP-D), charger (WP-E), bridge (WP-F) and the patient relay sheet 13 should wait for the contract changes in R-1 to R-6 and R-12.**

---

## Findings

### R-1 — Blocker — The permit (and request) are continuous power for the bridge, but both controllers withdraw them during the shock
**Concerns:** §B8.2, §B8.4, contract §5.4 (`PERMIT_CLK`, `SHOCK_REQ`, `BRIDGE_5V`), `supervisor.rs`, `rescue.rs`.

**What is wrong.** In hardware, `BRIDGE_5V` exists only while `SHOCK_REQ`, `PERMIT_OK` and `HOLD_OK` are all present; removing any one removes drive power for all four switches *and* the patient relay coil, at any moment, including mid-phase. In firmware, the supervisor re-runs `check_permit` on every tick while the permit is active and drops it if anything fails (`supervisor.rs`: `still_ok = self.check_permit(...)`; `if expired || !still_ok { self.permit_until = None }`). Two of those checks fail *because the shock is happening*:

- **Pads.** `check_permit` requires the supervisor's own pad reading to be 25–175 Ω. That reading comes from the MAX30009 through the patient relay's **rest** contacts (§B3.1). When `RELAY_CMD` moves the relay to the bridge (≤ 10 ms before the first phase, §B8.4), the MAX30009 sees an open circuit, the next sample reads out of range, and the permit is withdrawn. Depending on the MAX30009's sample timing relative to the relay, the permit falls before the shock (every shock blocked — "Refused") or during it (truncated).
- **Voltage.** `check_permit` requires the supervisor's reading to be within the at-plan window (1 % + 12 V = 32 V at 1,950 V). In phase 1 at 25 Ω the capacitor falls with time constant R·C = 30 Ω × 100 µF = 3 ms; it leaves the 32 V window after about 3 ms × 32/1,950 ≈ 50 µs. The AD7171 converts at 125 samples/s (8 ms), so the permit is withdrawn somewhere in the first 8–16 ms of a shock that lasts 5.5–15.5 ms (DESIGN_TABLE).

The main controller has the same coupling: in `Delivering`, `armed_check` aborts on `!self.pads.on` and `!chest_still`, both fed by the ADS1292R through the same rest contacts, and `withdraw_shock_request` drops `SHOCK_REQ`.

Then the turn-off order is undefined. When `BRIDGE_5V` collapses: the low-side drivers' input side (on `BRIDGE_3V3`, an LDO) drops out at roughly 3.4 V of `BRIDGE_5V`; the UCC5350's output goes low when its input side is under-voltage [UNVERIFIED for this variant]; the SN6505B keeps switching down to its 1.7 V under-voltage threshold (SN6505B datasheet, read 27 Nov 2026), and its rectified gate voltage sags in proportion. So the high-side gate droops from about 18 V towards 10 V while current may still flow, the over-current comparator and latch lose power at the same time, and the low side turning off first happens by accident of thresholds, not by design. If a high-side switch turns off first under current, it does so slowly (100 nF gate capacitor, about 20 µs) at up to 2 kV × 65 A — hundreds of millijoules in the die — and see R-2 for where the coil's energy goes.

The simulator does not show this because `plant.rs` fires the whole shock instantly when `bridge_may_fire` is first true.

**Suggested change.** Decide, and record in DECISIONS.md, what a consent means once a shock has started. The reviewer's recommendation: **consents gate the start only.** The bridge sheet latches "sequence started" on the first low-side enable; from then the sequence runs to its end under the hardware window (R-4), and consent removal is honoured only between shocks. Then (a) change `supervisor.rs` so that, once a permit is given, only cancel, a latched fault or a stale heartbeat withdraw it — not the pad or at-plan checks, which were satisfied at grant time; (b) change `rescue.rs` so `Delivering` does not abort on pad/chest readings after the relay command; (c) independently of (a)–(b), add a `BRIDGE_5V` supervisor on sheet 12 that **removes the low-side commands first** (at, say, `BRIDGE_5V` < 4.5 V) and holds the high side on until the current has decayed, and keep the over-current latch alive through that interval. If the owner prefers "consent removal aborts mid-shock", the hardware must still guarantee the low-side-first order and the firmware must not remove consent on readings that the shock itself disturbs. Add both cases to the single-fault table.

### R-2 — Blocker — No freewheel path for the 30 µH series coil; the stated energy path is wrong
**Concerns:** §B6.1, §B6.5 ("Energy stored in L_S ... returns to the capacitor through the high-side body diodes"), sheet 12.

**What is wrong.** Follow phase 1 at its end. Current flows `HV_CAP_P → R_S → L_S → HV_BUS → QHA → HV_OUT_A → patient → HV_OUT_B → QLB → shunt → HV_RTN`. QLB opens. The patient-loop current can circulate through QHB's body diode (from `HV_OUT_B` up to `HV_BUS`) and back through QHA — but that loop does **not** contain L_S. L_S's current enters `HV_BUS` and needs a path back to the capacitor's negative side; with QLA and QLB both off there is none. The high-side body diodes point *into* `HV_BUS`, the same direction as L_S's current, so they cannot return it to `HV_CAP_P`. The coil therefore drives `HV_BUS` and both outputs upward until something breaks down: the off low-side switches. With a ~30 ns SiC turn-off, L·di/dt at 24 A (end of phase 1 at 25 Ω: 65 A × (1 − 0.63 tilt)) is 30 µH × 24 A / 30 ns ≈ 24 kV unclamped, i.e. the 3.3 kV parts avalanche.

[CALC] Energy into avalanche ≈ ½·L·I² × V_clamp/(V_clamp − V_cap): end of phase 1 at 25 Ω, ½ × 30 µH × 24² = 8.6 mJ × 3.8/(3.8 − 0.72) ≈ **11 mJ, on every shock, twice**. At the 120 A over-current trip (§B6.5): ½ × 30 µH × 120² = 0.22 J × 3.8/(3.8 − 2.0) ≈ **0.46 J**. The MSC080SMA330B4N datasheet (DS00005914A, read 27 Nov 2026) gives **no avalanche energy rating and no short-circuit rating** at all, so this is outside anything the maker guarantees. During the clamp, `HV_BUS`, `HV_OUT_A/B` and the relay and pad conductors swing to the breakdown voltage (≈ 3.5–4 kV, [EST]), above the 2,200 V working voltage specified for the gate transformers and the spacing placeholders.

**Suggested change.** Add a freewheel diode across the series pair R_S + L_S (anode at `HV_BUS`, cathode at `HV_CAP_P`), so the coil current circulates through R_S and decays with τ = 30 µH / 3.95 Ω ≈ 7.6 µs. Rating: reverse voltage ≥ 3.3 kV (it sees up to the full capacitor voltage if `HV_BUS` is pulled to ground by a short), pulse current ≥ 450 A for microseconds, fast recovery. Consider an RC snubber from `HV_BUS` to `HV_RTN` for the switch and wiring inductance as well. Correct the sentence in §B6.5. This must be in the spec before WP-F captures because it adds a high-voltage part and possibly a net. A circuit simulation of each phase end, the 90 A trip and the R-1 consent-loss case is the only way to settle the remaining spike sizes.

### R-3 — Blocker — The SN6505B's soft-start makes the high-side timing impossible
**Concerns:** §B6.3 (high-side drive "on/off within about 50 µs"), §B6.4, §B8.4.

**What is wrong.** The SN6505B datasheet (TI, read 27 Nov 2026) says that "upon power up or when EN pin transitions from Low to High, the gate drive of the output power-MOSFET is gradually increased"; its soft-start time is 1–8 ms and its start delay 3.5–18 ms (measured with a 40 µF load; the ramp of the internal gate drive applies whatever the load). Enabling the carrier with the high-side command therefore turns QHA on over milliseconds, not 50 µs. Consequences: (a) with the planned 100 µs lead, QHA is still half-on when QLB starts phase 1, so it carries up to 65 A in its linear region with hundreds of volts across it; (b) **QHB cannot be turned on inside the 0.5 ms interphase gap**, and cannot be turned on earlier because QHB on while QLB conducts is a leg-B short. So the specified driver cannot produce the biphasic waveform. Frequency (424 kHz typical, 363–517 kHz) is as stated.

**Suggested change.** Pick a transformer drive with a fast, defined enable — for example keep the push-pull oscillator running whenever `BRIDGE_5V` exists and gate the transformer primary with discrete switches driven by the command, or use a driver without soft-start — and re-derive the high-side on/off times (rectifier, 100 nF gate capacitor, PNP turn-off) in simulation. Then set the high-side lead time and the 150 µs lockout from those results. Record any change to the 0.5 ms gap (D-02 assumption, DESIGN_TABLE) as an owner decision.

### R-4 — Blocker — The "non-retriggerable" 25 ms window is a retriggerable part and has no latch
**Concerns:** §B6.4 item 2, §13 fault 1, contract §8 (sheet 12: "74LVC1G123 window").

**What is wrong.** TI describes the SN74LVC1G123 as a "single **retriggerable** monostable multivibrator" ("retriggerable for very long output pulses, up to 100 % duty cycle"; ti.com, read 27 Nov 2026). The specification says non-retriggerable. A retriggerable window is extended by every new edge, so a low-side command that keeps **toggling** is never cut off. That is a realistic single fault: an STM32 advanced timer keeps running when the processor core hangs, so "main frozen with the timer still producing PWM" is the classic stuck-timer failure. With `SHOCK_REQ` high and the permit window open, the bridge could deliver repeated phase-1 pulses for up to the full 200 ms permit (`permit_window_ms`), i.e. essentially the whole stored energy, monophasic — finding type (b). Even with a non-retriggerable part, nothing stops a *second* window after the first expires unless the expiry is latched. The window is independent of both controllers' timing but not of their being powered (it lives on `BRIDGE_3V3`), which is acceptable.

**Suggested change.** Make the window expiry set a latch (like the over-current latch) that blocks all four commands until `BRIDGE_5V` is removed; optionally also count low-side enables and refuse more than two (one per phase) per `BRIDGE_5V` cycle. Add a hardware maximum permit duration on sheet 09 (for example `BRIDGE_5V` allowed at most ≈ 250 ms per rising edge of `PERMIT_OK`, re-armed only after the permit has been absent for some time) so a supervisor whose code keeps toggling cannot hold the rail indefinitely. Add "main frozen with timer toggling" to the single-fault table.

### R-5 — Blocker — One signal drives CHARGE and the supply switch, racing the LT3751's latched fault; regulation by toggling it causes battery inrush
**Concerns:** §B4.4 "Stopping", §B4.6, contract §5.4 `CHG_GATED`, §10 "WP-D with WP-E".

**What is wrong.** The LT3751 datasheet (rev D, read 27 Nov 2026 from the same mirror) says the UVLO1 pin "trips the FAULT latch low, disabling switching. After VTRANS rises above VUVLO1, toggling the CHARGE pin reactivates switching", and "resetting either latch requires the CHARGE pin to be toggled". `CHG_GATED` raises CHARGE and closes the VTRANS switch at the same instant; VTRANS then ramps while the LM74502's FET charges about 850 µF of bulk capacitance. If CHARGE is seen high while VTRANS is still below 4.9 V, the part latches FAULT and does not start — and every later attempt repeats the same race. Depending on component values this could stop **every** charge (finding type (d)), including the switch-on self-test. Separately, §B4.4 implements normal regulation as on/off control of `CHG_EN`, which re-opens and re-closes the VTRANS switch on every regulation step, recharging the bulk capacitors from the battery each time; with no inrush limit specified that is a large current pulse that can trip the MAX17320's short-circuit protection and take down every rail (see R-16).

**Suggested change.** Split the function in the contract: `VTRANS_EN` = `HOLD_OK` AND a charger-session enable (switch has a controlled rise); `CHARGE` = `CHG_EN` AND `HOLD_OK` AND "VTRANS good" (a comparator or delay, so CHARGE rises only after VTRANS is above UVLO1). Turning off stays immediate on both when `HOLD_OK` falls (D-25 unchanged). Regulate with the CHARGE pin (or the LT3751's own regulation mode, R-7), not by cycling the supply switch.

### R-6 — Blocker — `HOLD_OK` fails towards "held" when a comparator output is open or unpowered
**Concerns:** §B5.4 (`HOLD_OK`), contract §5.4 (`HOLD_OK`: open-drain wired-AND, 10 kΩ pull-up on 09), D-25, finding F-3.

**What is wrong.** Each stack's comparator *releases* its open-drain output when its gate is properly negative, and the pull-up then makes `HOLD_OK` high = "both held, charger and bridge may run". The released state is the **same** state as a comparator whose output pin is not soldered, whose output transistor has failed open, or whose own supply pin is open while `5V0_SYS` (and the pull-up) is present. Such a fault is invisible in normal use and leaves `CHG_GATED = CHG_EN`. Combine it with the fault D-25 exists for — main frozen with `CHG_EN` high — and the charger keeps feeding the conducting discharge path: 40 W into 2.25 kΩ holds the capacitor at about √(40 × 2,250) ≈ 300 V [CALC] until the battery is flat (finding type (c), the F-3 failure). The design's claim "a pump fault reads as discharge active" is true only for faults on the gate side, not for the comparator itself. The only detection is the interlock self-test (§B8.5), which does not exist in firmware and cannot run as specified (R-13).

**Suggested change.** Make "held" an actively driven state: each comparator drives a push-pull output high only when its gate is below the threshold, and the two are combined with an AND gate (or two series switches) whose inputs have pull-*downs*, so any open or unpowered element reads "not held". Better still, derive each stack's permission from a part that physically needs the negative gate bias to conduct. Add "HOLD_OK comparator output open" to the single-fault table and require the interlock self-test (R-13).

### R-7 — Major — The LT3751 FB pin is a regulator, not a ceiling that stops; its tolerances do not fit between the operating window and the capacitor rating; the primary-side trip can exceed 2,200 V
**Concerns:** §B4.4 item 2 and 3, §B4.6, §13 fault 10.

**What is wrong.** Datasheet (read 27 Nov 2026): below FB = 1.16 V (1.12–1.20 V) the part is in charge mode; between that threshold and 1.34 V it enters **low-noise regulation mode** and regulates FB to 1.22 V (1.19–1.25 V) with a reduced current limit (11–106 mV); above 1.34 V it stops switching. So the adult "ceiling" divider is a 2,071 V regulator whose charge-to-regulation hand-over happens at 2,071 × 1.16/1.22 ≈ **1,969 V nominal**. [CALC] With FB tolerances and ±1 % resistors (ratio ±2 %): hand-over as low as 1.12/1.22 × 2,071 × 0.98 ≈ **1,863 V**; regulated point as low as 1.19/1.22 × 2,071 × 0.98 ≈ 1,980 V, minus the FB bias current (up to 300 nA × 23.7 kΩ ≈ 7 mV, about 0.6 %) ≈ **1,968 V**; as high as 1.25/1.22 × 2,071 × 1.02 ≈ **2,164 V**. The firmware's worst-case true voltage at plan is ≈ 1,976 V (1,950 V with a sensor reading 1 % + 7 V low). So on a corner unit the top of every adult charge runs in reduced-current regulation mode (slower, unsimulated) and the charger can stall just under the needed true voltage; `rescue.rs` counts "reached plan" within the window, so this probably slows rather than blocks, but it has not been analysed. No choice of nominal fixes it: keeping the hand-over above 2,008 V and the regulated maximum below 2,200 V needs a ratio of (1.25 × 1.02)/(1.12 × 0.98) ≈ 1.16, but 2,200/2,008 is only 1.096. The primary-side trip: 0.98 × 50 × 40.2 kΩ/909 Ω − V_diode ≈ 2,162 V nominal, with the datasheet's comparator band 0.955–1.005 V (±2.5 %), turns ratio ±1 % and resistors ±1 % each → about **2,040–2,280 V**; the upper end is above the capacitor's 2,200 V rating, so in a "FB divider open" fault it protects nothing the OVP comparator does not.

**Suggested change.** Either (a) use regulation mode deliberately — the FB divider is the regulator, switched per mode, with 0.1 % high-voltage resistors, and the firmware loop becomes a check — and re-derive the at-plan arithmetic; or (b) keep the FB divider purely as an over-voltage stop by setting it so the *no-load threshold* (1.34 V, 1.29–1.38 V) sits between 2,030 V and 2,180 V, and accept that regulation mode is entered only above the operating window. Lower the primary-side trip nominal to about 2,080 V so its upper bound stays under 2,200 V. State in the spec that the OVP comparator (2,160 V on the supervisor chain: about 2,138–2,182 V with chain and comparator error) is the only hardware layer whose whole tolerance band is below the capacitor rating.

### R-8 — Major — A single fault in the mode switch's second pole blocks every adult shock, and nothing detects it before a rescue
**Concerns:** §B4.4 item 2, §B10.4, contract §5.5 `MODE_ADULT_HW`.

**What is wrong.** The firmware reads pole A (`MODE_ADULT_N`/`MODE_CHILD_N`); the charger ceiling reads pole B (`MODE_ADULT_HW`). If pole B's contact fails open, the wire breaks or the switching MOSFET fails open while pole A says ADULT, every adult charge regulates at ≈ 1,221 V (R-7: it regulates, so it sits there), `rescue.rs` raises `ChargeTimeout` after 12 s, and the device enters a fault for the rest of the rescue (finding type (d)). The daily low-voltage test (400 V) cannot see it; only a weekly full-energy test done with the switch in ADULT would. The "fails towards no shock" reasoning is right for *overdose*, but no shock in a shockable adult is also a harm.

**Suggested change.** Route `MODE_ADULT_HW` (after the switch) back to the main controller as a readback; check pole A against pole B at switch-on and on every mode change; announce a mismatch. Ask the owner what the device should do in a rescue when the poles disagree (for example: shock at whatever the hardware ceiling allows, with a spoken warning, versus refuse). Record it.

### R-9 — Major — The hold pump and gate network are specified inconsistently; the release time and `HOLD_OK` margin depend on unstated values; the hold is referenced to the wrong ground
**Concerns:** §B5.4, §B5.6, contract §9 WP-D, §13 fault 27.

**What is wrong.**
- **Load.** The requirement is "bottom gate ≤ −7.0 V at 25 µA", but the same section fits a 100 kΩ gate-source resistor, which alone draws 70–80 µA at −7 to −8 V, plus the `HOLD_OK` level-shift network and the test-point divider. A three-stage Dickson pump from 3.3 V (≈ −8.1 V unloaded) will not meet −7 V at three to four times the stated load without much larger capacitors.
- **Ripple versus release time.** At a 1 kHz clock, the gate's own capacitance (Ciss ≈ 5.3 nF, DigiKey listing; Qg ≈ 200 nC, IXYS datasheet copy) with 100 kΩ decays by τ = 0.53 ms — about 60 % in one half-period. A reservoir capacitor is therefore unavoidable: holding ripple to 0.5 V at 80 µA needs about 160 nF, and then the release time constant becomes 100 kΩ × 165 nF ≈ 16.5 ms: the gate takes about 19 ms to reach −2.5 V and tens of ms to conduct fully, not the "1–3 ms" stated. That is still well inside the 3 s disarm limit, but it changes the linear-region transition (about 0.2 J per device by the same estimate the document uses — still needs the FBSOA check it asks for).
- **`HOLD_OK` margin versus loop jitter.** With no reservoir, the gate goes from −8.1 V to the −6.0 V threshold in 0.53 ms × ln(8.1/6) ≈ 0.16 ms, so any gap in the firmware-generated clock would drop `HOLD_OK` (and the bridge rail mid-shock, R-1). With a reservoir, it takes about 5 ms. The contract asks WP-D and WP-B to agree this but gives no number.
- **Ground.** The pump and `HOLD_OK` comparator are referenced to `GND`; the stack's source is `HV_RTN`, joined to `GND` only at NT1101. If NT1101 opens (fault 27) the gate-source voltage of **both** stacks becomes undefined at once — a common-cause loss of both discharge paths, not merely "both chains read nonsense".
- **Upper devices.** When the stack conducts, each upper gate sits near its divider tap (hundreds of volts) above its source; the gate-source clamps that this needs are mentioned for the upper devices only as "(clamped)". The IXTT10N100D2's gate limit is ±20 V [BK].

**Suggested change.** Restate the pump requirement with the real load (or raise the gate resistor to about 1 MΩ and accept the slower release), specify the reservoir capacitance and the resulting release time, set the clock frequency and the longest permitted firmware gap as numbers, return each pump, gate resistor and comparator reference to its stack's source on `HV_RTN` at the stack, make NT1101 a construction that cannot open (multiple parallel bonds), and specify gate-source zeners on all six devices. Remove the contradiction between §B1.6 ("hold pumps' drive buffers do not depend on `5V0_SYS`") and §B5.4 ("for path A only ... a 5 V buffer on `5V0_SYS`").

### R-10 — Major — The over-voltage "force" MOSFET cannot be driven by a ground-referenced `OVP_TRIP`
**Concerns:** §B5.5, contract §5.4 `OVP_TRIP`.

**What is wrong.** The MOSFET must short a gate that sits at about −8 V to a source at 0 V. Oriented with its source on the −8 V node, `OVP_TRIP` at 0 V gives it +8 V gate-source — it is permanently on, so the stack can never be held (every charge blocked). Oriented with its source on 0 V, its body diode conducts from 0 V into the −8 V node and clamps the gate near −0.7 V — same result. Either naive capture fails safe but blocks every shock.

**Suggested change.** State in the contract that `OVP_TRIP` is 0/3.3 V referenced to `GND` and that sheet 11 level-shifts it to a switch referenced to the negative gate node (for example a small optocoupler or a PNP/level-shift pair), with a defined state when 3V3_SUP is off.

### R-11 — Major — The dynamic consent decoders have no specified frequency window, no hardware maximum duration, and zero timing margin against the firmware
**Concerns:** §B8.1–B8.2, §B5.4 "Clock", contract §5.4 and §10.

**What is wrong.** A diode-pump detector accepts **any** waveform above its corner frequency, including a fast oscillation (a pin left on a clock output, a free-running timer — fault 4 is acknowledged but not bounded). The permit detector "collapses within ≤ 2 ms" while the firmware may leave "at least every 2 ms" between re-arms: zero margin, so a supervisor loop that takes slightly longer (an SPI transaction, an event-log write) drops `BRIDGE_5V`, mid-shock if R-1 is not fixed. Nothing in hardware limits how long a permit can last.

**Suggested change.** Specify for each `_CLK` net: frequency band (for example 1–5 kHz), detector response to frequencies far above the band (reject, using a band-pass or an edge-counting window), hold-up time (≥ 3× the longest permitted firmware gap) and collapse time (≤ a stated bound), and the hardware maximum permit duration of R-4. Add a firmware rule to the board-support requirements for the worst-case loop time.

### R-12 — Major — The patient relay candidate's coil cannot be powered from the interlock rail as budgeted, and its on-time limit is shorter than the permit window
**Concerns:** §B7.2, §11 (BRIDGE_5V 0.8 A), §B8.2 (TPS22918, 2 A).

**What is wrong.** The Gigavac G17 datasheet (search extract, 27 Nov 2026) gives a 12 V coil of 12 Ω ±10 % — about 1 A and **12 W** — rated for intermittent duty of **100 ms maximum on, 10 % duty**. From `BRIDGE_5V` through a boost that is about 2.8 A at 85 % efficiency [CALC], above the TPS22918's 2 A, above the 0.8 A rail budget and above the 1.5 A peak budgeted for `5V0_SYS`. The permit window and the specified coil duty are 200 ms; a `RELAY_CMD` stuck high would over-drive the coil.

**Suggested change.** Budget the relay coil explicitly (energise from a stored-charge reservoir behind the interlock, or a separate switch gated by `BRIDGE_5V`), limit coil on-time in hardware to the relay's rating, and specify fast release (zener rather than plain diode suppression) because release time is in the timing budget. Re-check `5V0_SYS` and the TPS22918 chain once the relay is chosen.

### R-13 — Major — The self-tests that the fault table relies on cannot run with the firmware as written
**Concerns:** §B8.5 (interlock self-test), §B6.6 (bridge self-test at ≤ 100 V), §B5.3 (τ fit), §13 faults 7, 16 and 18; `supervisor.rs`.

**What is wrong.** Both tests need `BRIDGE_5V`, so both need a supervisor permit. `supervisor.rs` gives a permit only when the main reports `Delivering`, after a 3 s countdown, with its own pad reading at 25–175 Ω, at the planned voltage, and once per arming. A bridge self-test at ≤ 100 V with no patient fails the pad check; proving the *request* switch can block needs a permit with no request, which the supervisor never gives. So faults 16 and 18 have no detection at all today, and fault 7/8 detection (the τ fit) is also unwritten.

**Suggested change.** Define a supervisor "test permit": allowed only when the supervisor's own reading is below, say, 100 V, the main reports a test state, and bounded in time; record it as a decision; add it to the firmware and simulator. Then run the bridge self-test at every switch-on before the first charge (§16 rank 8 already suggests this), so a shorted low-side switch is found before a rescue, not after.

### R-14 — Major — A shorted low-side switch is worse than fault 16 says
**Concerns:** §13 fault 16.

**What is wrong.** With QLB shorted, phase 1 starts as soon as QHA begins to turn on, and QHA turns on slowly (R-3) through its linear region with the full shock current — enough dissipation to fail QHA short as well [EST]. With QHA and QLB both shorted and the relay closed, nothing controllable remains in the path: the capacitor discharges monophasically into the patient — at 175 Ω about 190 J × 175/180 ≈ 185 J against 150 J planned [CALC], with a decay time of about 18 ms — and the relay may then open while current is still flowing, which it is not rated to do. Phase 2 (QHB + shorted QLB) would be a leg short, stopped by the 90 A latch with the R-2 energy. The table's "if undetected" outcome understates this.

**Suggested change.** Update fault 16 with this cascade; make the per-switch-on bridge test of R-13 mandatory before the first charge; consider a current check in the first microseconds of the QHA lead (no current must flow before `CMD_LB`) that latches the bridge off.

### R-15 — Major — A shorted output diode dumps the stored energy into the charger transformer
**Concerns:** §13 fault 12, §B4.3 D1001.

**What is wrong.** With D1001 shorted and the capacitor charged, the capacitor is connected straight across the 18.75 mH secondary. The secondary core saturates at about 0.5 A [CALC: 25 A ÷ 50], after which the winding's resistance is the only limit, so up to 190 J goes into the transformer — overheating, possible insulation failure inside the device. "LT3751 current limit; charge time-out" covers only a diode that shorts while the capacitor is empty.

**Suggested change.** Use two rectifiers in series, each rated for the full reverse voltage (2,700 V at the trip ceiling), plus a fusible series resistor rated for high voltage in the secondary; update the table.

### R-16 — Major — Charger faults can take down every rail, including the supervisor's alarm
**Concerns:** §B1.1, §13 fault 11, R-5 inrush.

**What is wrong.** A shorted primary MOSFET (or an uncontrolled bulk-capacitor inrush) draws current from the pack through F1001. Whether F1001 (15 A time-lag) clears before the MAX17320's short-circuit protection opens the pack FETs has not been coordinated. If the pack protection trips first, `VBAT_SYS` collapses and with it `3V3_SUP` and the supervisor's alarm (D-26) — the table's "rest of device runs, S alarm works" is not guaranteed.

**Suggested change.** Specify the coordination (fuse I²t versus the MAX17320 short-circuit threshold and delay), limit VTRANS inrush, and say what the MAX17320 does after a trip (latch or retry).

### R-17 — Minor — The two pad-impedance chips load each other
**Concerns:** §B3.4, §B3.6.

[CALC] Seen from the main's 64 kHz drive, the supervisor's two drive taps (2.35 nF ≈ 1.06 kΩ plus 1 kΩ each) are about 4.2 kΩ across the pads; at 250 Ω that reads about 6 % low, and the shunt changes with the MAX30009's state (driving, idle, unpowered). The supervisor sees the main's network the same way (≈ 3 kΩ at 128 kHz). The supervisor's "±5 % is enough" claim is then marginal at the 175 Ω permit limit. The carrier choice does stop the two demodulators producing a low-frequency product (64 kHz square-wave harmonics are 64, 192, 320 kHz…; 128 − 64 = 192 − 128 = 64 kHz), and that holds even with independent clocks. **Change:** calibrate each channel with the other chip in a fixed, specified state; specify both chips' drive pins as high-impedance when idle.

### R-18 — Minor — Net classes: `HV_RTN` gets 12.5 mm clearance from the ground it is tied to; the 65 A shunt node is in the default class
**Concerns:** `hierarchy.toml` net-class patterns, contract R2.

`HV_RTN` matches `HV_PULSE` (12.5 mm) but sits at `GND` potential — every gate driver, stack source and shunt next to it would violate. `LS_SRC` (the shunt top, 65 A pulses) matches no pattern and gets 0.15 mm tracks. **Change:** class `HV_RTN` by current, not voltage (a new `HV_RTN` class with wide tracks and low-voltage clearance), and give `LS_SRC` a high-current class.

### R-19 — Minor — The 7 s charge time is not met at the tolerance corners
[CALC] t = C·V·(V + 2·N·V_TRANS)/(η·V_TRANS·I_PK). At 5.5 V, η = 0.65, capacitor +5 % (105 µF) and the current-limit threshold at its 100 mV minimum (I_PK = 20 A): 1.05 × 0.195 × 2,500 / (0.65 × 20 × 5.5) ≈ **7.2 s**; at η = 0.60, 7.8 s. The firmware limit is 12 s, so this does not fault, but the "≤ 7 s" requirement claim should state its corners.

### R-20 — Minor — LT3751 VCC lockout set below the part's minimum operating voltage
VCC UVLO is set to 4.6 V (§B4.3), but the datasheet gives 4.75 V as VCC's minimum operating voltage. Set it to ≥ 4.8 V; check that `5V0_SYS` stays above it when the pack sags to 5.2–5.5 V under the 9 A charger load (TPS62130 in dropout).

### R-21 — Minor — Domain crossings the contract does not cover
- `LCD_DISP`: an open-drain output of the main pulled up to `3V3_AON` on sheet 06. With the main unpowered, its pin protection diode (unless it is a 5 V-tolerant pin with no diode to VDD) back-powers the main and pulls the line low — the display blanks, defeating §B10.3. Specify an FT pin or a buffer (contract R7).
- `I2S_*` from `3V3_MAIN` into the MAX98357A on `5V0_SYS`: with `5V0_SYS` off, the amplifier's input diodes can back-power `5V0_SYS`, which also feeds the `HOLD_OK` comparators, the `CHG_GATED` gate and the load-switch chain. Add series resistors/buffers or a firmware rule.
- Tap clamps "to the reader's rail" on sheets 11 and 12: neither sheet lists `3V3_MAIN`/`3V3_SUP` in the rails table, and a clamp to an unpowered reader's rail back-powers it. Clamp with zeners/TVS to `GND` instead, or move the clamp to the reader's sheet.
- I²C held low when the main is off: check whether the MAX17320 enters a communication-shutdown mode with SDA and SCL low [UNVERIFIED].
- Power-up: the `HOLD_OK` comparators' open-drain outputs are undefined while `5V0_SYS` ramps (and R-6 makes "undefined" read as "held"). Add a firmware ordering rule (`EN_5V0` well before any consent or `CHG_EN`).

### R-22 — Minor — One shorted upper depletion device takes the other two past their rating, undetected
§B5.6 says a shorted device leaves 1,004 V on each remaining one "at the limit". At the OVP trip (2,160 V) it is 1,080 V, above the 1,000 V rating; `HOLD_OK` watches only the bottom gate, so the loss of margin is invisible. The failure direction is towards conducting (safe), but it is latent. Consider four devices per stack, or measuring the stack's mid-point.

### R-23 — Minor — The leg lockout trusts commands, not gates; the high-side turn-off relies on one PNP
If a high-side PNP fails open, the 100 nF gate capacitor holds QHA on far longer than the 150 µs lockout, and the next low-side command on that leg is a shoot-through (caught only by the 90 A latch, with R-2's energy). Add a gate-source bleed resistor (for example 10 kΩ, τ ≈ 1 ms) on each high-side gate as a backup.

### R-24 — Minor — Current sense cannot reach its stated full scale
The INA240A1 on `BRIDGE_3V3` cannot output 3.2 V (80 A) from a 3.3 V supply; it saturates near 3.1 V ≈ 77 A [BK swing]. Fault currents (90–120 A) will be clipped in the record. Use a lower gain or a different supply/reference.

### R-25 — Note — Corrections from the SiC MOSFET datasheet (DS00005914A, Microchip, 2025, read 27 Nov 2026)
- The datasheet **does** publish a forward safe-operating-area chart with a 10 ms line and a transient thermal impedance curve, so §16 rank 1 can be partly answered on paper now. [EST] At ~400 W average conduction loss per conducting device over a 10 ms pulse and Zθ of order 0.1–0.15 °C/W, the junction rises by tens of kelvin — plausible, but to be computed properly.
- Maximum junction temperature is **150 °C** (not 175 °C); RDS(on) 85 mΩ typical, 106 mΩ maximum at 25 °C.
- Off-state leakage up to **35 µA** at 3,300 V (25 °C), not ≤ 10 µA: the idle output node can sit at up to ≈ 35 V through the 1 MΩ divider (still far below the "shorted switch" signature).
- Gate-drain charge 51 nC (not ≈ 40 nC): ΔV_GS ≈ 51 nC / 105 nF ≈ 0.49 V against a 1.9 V minimum threshold at 25 °C (lower hot) — still adequate.
- No avalanche or short-circuit rating (see R-2).
- Distributor listings disagree on lifecycle (one marks a variant discontinued). Confirm status with Microchip.

### R-26 — Minor — The air-core coil does not fit the stated size at the stated resistance
[CALC] 55 turns × π × 20 mm ≈ 3.5 m of wire; for ≤ 50 mΩ that needs ≈ 1.2 mm² copper (≈ 1.4 mm diameter with insulation ≈ 1.6 mm), i.e. ≈ 88 mm of winding length in one layer, against 30 mm. A multi-layer coil changes inductance and turn-to-turn voltage. Re-derive.

### R-27 — Minor — Errors in the single-fault table text
- Fault 14: an open resistor in the two-resistor bottom leg **doubles** the tap voltage (reading doubles), not halves. On the supervisor chain it also doubles the input to the OVP comparator, which then trips at about 1,080 V true — every adult charge is dumped (detected, but a "no shock" fault, not "contained").
- Fault 12: see R-15. Fault 11: see R-16. Fault 27: see R-9. Fault 1: add the toggling-timer case (R-4).
- **Omissions:** R-1 (consent lost mid-shock), R-2 (coil energy), R-3 (high side too slow), R-5 (charger start race), R-6 (open `HOLD_OK` comparator), R-14 (cascade), R-23 (PNP open), relay one-pole weld, battery removed while charged (behaves safely as far as this review can tell; list it).

### R-28 — Minor — Front-end protection details
- The diode clamps "to the chip's analog rails" will conduct before a 5–6 V TVS on the tap does, so the 98 mA sense-tap and ≈ 1.3 A drive-tap currents of the external-defibrillator test go into `AVDD_AFE` and the MAX30009's 1.8 V rail, which regulators cannot sink. Put the TVS (or a shunt clamp) on the rails, or clamp the taps to ground below the rail.
- A search extract lists DPCR values as 20 kΩ and 51 kΩ only; a **1 kΩ DPCR may not exist** [UNVERIFIED]. Confirm, or name another pulse-rated part for the drive taps (≈ 1.3 kV across it for ≈ 10 µs).

### R-29 — Note — Standby logging cadence
§B11 says the durable store is written "about every minute"; §12's budget uses one wake every 5 minutes. At one per minute the logging term is ≈ 42 µW, not 8 µW [CALC: 2.5 mJ / 60 s]. The total stays near 1 mW.

### R-30 — Note — Firmware's overvoltage limit equals the capacitor's rating
`overvoltage_limit` is 2,200 V, the capacitor's minimum rating; with a sensor reading 1 % + 7 V low the true voltage could reach ≈ 2,230 V before firmware acts. Hardware layers below it carry the load (R-7); consider 2,150 V in firmware.

### R-31 — Note — Inputs with no HAL purpose
The supervisor reads `SHOCK_REQ` and `OVP_TRIP` (contract §7, "monitors"), but `SupervisorHardware` has no method for either and `supervisor.rs` uses neither. Either add the methods and a use, or drop the nets.

---

## Answers to the specific questions in the brief

- **Can a single fault deliver energy without both request and permit?** Not found, provided R-6 is fixed: `BRIDGE_5V` needs all three series switches, and a single shorted switch leaves two consents (fault 18). But fault 18 is latent until a self-test that cannot currently run (R-13).
- **Longer/larger shock than planned?** Yes: toggling-timer fault with a retriggerable window (R-4); shorted low-side switch cascade (R-14). The 25 ms cap is independent of both controllers' timing but not of the part choice.
- **Capacitor left charged with no path to empty it?** Main frozen with charger on plus an open `HOLD_OK` comparator (R-6) holds ≈ 300 V. NT1101 opening affects both stacks' gate reference at once (R-9). A single open pump or resistor is covered by the second path.
- **Block every shock?** Several: R-1 (permit dropped when the relay moves), R-3 (high side cannot meet the timing), R-5 (CHARGE/UVLO race), R-8 (mode pole B), R-10 (naive OVP force capture), R-12 (relay coil supply).
- **Shorted charge-pump capacitor or diode leaving the gate negative?** No mechanism found: an inverting pump makes no negative voltage without a clock, and the only other negative rail (the low-side −5 V) is on another sheet with no path. A shorted pump part removes the bias (safe). An **open** gate-source resistor would leave the gate's own charge holding the stack off for a long time; the second path and the τ fit cover it once the fit exists.
- **Stack voltage sharing and turn-on into 1,950 V.** Statically, sharing is set by the gate-tap divider, not by leakage, because each upper device regulates its own source; off-state leakage (up to 250 µA at 125 °C at −5 V gate) flows through the whole stack. The stack does not turn *off* into high voltage in normal use (hold precedes charging). At release, peak stack dissipation is V²/(4R) = 2,008²/(4 × 4,500) ≈ 224 W, about 75 W per device if shared; duration depends on R-9's reservoir. Needs SPICE with the IXYS model and the FBSOA chart.
- **Outputs pulled to ground at rest.** No interaction found with the relay or the front ends: at rest the relay isolates the pads from the outputs; the outputs sit at ≤ ≈ 35 V through the divider (R-25).
- **Sneak paths.** None found through the body diodes from `HV_BUS` to the pads when idle (they point into `HV_BUS`). Gate-transformer capacitance at 10 pF and ≈ 1 V/ns gives ≈ 10 mA common-mode for ≈ 2 µs — harmless. Measurement dividers: only the back-powering in R-21.

---

## Claims checked and found correct

Arithmetic [CALC by this reviewer]:
- Stored energy ½CV²: 190.1 J at 1,950 V, 66.1 J at 1,150 V.
- Charger: charge-time formula t = C·V·(V + 2N·V_TRANS)/(η·V_TRANS·I_PK) derived independently from boundary-mode power P = ½·I_PK·V_TRANS·V/(V + N·V_TRANS); 5.35 s, 6.43 s, 4.07 s (adult) and 2.16 s (child) reproduced. Average primary current 9.2 A and 53 W input at full charge; primary RMS 11.4 A (duty 0.87); input ripple ≈ 6.7 A RMS; reflected voltage 43.6 V; switch peak ≈ 52 V; diode reverse 2,590 V; secondary peak 0.42 A; secondary inductance 18.75 mH; per-cycle energy 1.69 mJ; switching frequency 32/43 kHz; primary-inductance window 6.1–8.6 µH with 7.5 µH ± 10 % inside it; leakage-clamp loss ≈ 0.7 W.
- LT3751 datasheet facts: current limit 106 mV typical (100–112 mV); VUVLO/OVLO = 1.225 V + 50 µA·R and the four resistor values; output-trip formula 0.98·N·RVOUT/RBG − V_DIODE and the 2,162 V nominal; bias current 0.98 V/909 Ω ≈ 1.08 mA; LVGATE for VCC ≤ 8 V; CHARGE below 0.3 V shuts down.
- FB divider arithmetic: 1,221 V and 2,071 V nominal (but see R-7 for what the pin does).
- Discharge: 2.25 kΩ / 4.5 kΩ, τ 0.225 / 0.45 s, 0.89 s / 1.79 s to 38 V; 0.45 A peak per path; 670 V per device and per resistor; 32 J / 63 J per resistor; 300 W peak per resistor.
- Bleed: 19.9 MΩ string; 6.0 MΩ total; 37 min to 50 V; 2.0 h on the bleeder alone; 0.67 W while charged.
- Divider ratios 1:998 and ≈ 1:1,000; error-budget sums 0.92 % and 0.51 % and their margins against D-28.
- Bridge: 67 A/µs with 30 µH; 90 A in ≈ 1.3 µs and ≈ 120 A peak with 0.5 µs of delay; 430 A hard-short limit; R_S 25 J per adult shock and 16.5 kW peak; 0.4 W average over 20 shocks; Wheeler's formula gives ≈ 30 µH for the stated coil (size problem in R-26); shunt 40 mV/A.
- Patient side: drive-capacitor impedance 1.06 kΩ at 64 kHz; 12 µC through the drive capacitors; 98 mA sense-tap peak; ≈ 1.3 A drive-tap edge current; ≈ 0.8–1 J per 51 kΩ resistor per external pulse; excitation ≈ 0.34 mA RMS; 102 µV for 0.3 Ω; 96 nV per step (≈ 0.3 mΩ); carrier products of 64 kHz square and 128 kHz contain no low-frequency term.
- Battery load: 10.2 A, 86 W at 8.4 V.
- Logic: the three truth tables in §B8.3 match `interlock.rs` exactly for the static case (the dynamic problems are in R-1, R-6, R-11).
- Reference-designator blocks: sheet blocks 101–999 and 1001–1599 and overflow blocks 5101–6599 do not collide; fixed designators are unique.
- Net names: no inter-sheet net other than the intended ones contains `HV_`, `PT_`, `HSA_` or `HSB_` (`HVM_`/`HVS_` do not match `*HV_*`).
- Every `MainHardware` and `SupervisorHardware` method has at least one net (contract §7); every inter-sheet net in §5 has one driver and one default resistor, apart from the noted `RTC_INT_N` exception.
- The board-split list (contract §6) matches the sheet grouping; no `HV_*` or `PT_*` net crosses.
- Power-on: every enable and consent defaults off through a pull-down; no momentary enable was found except the `HOLD_OK` ramp in R-21.
- Self-test charging: with both holds before the charger, the self-test's 400 V charge takes ≈ 0.52 s at 5.5 V, η = 0.65 — inside its 1 s limit, given R-5 is fixed.

Datasheet ratings checked by this reviewer (27 Nov 2026):
- **IXTH/IXTT10N100D2**: 1,000 V; RDS(on) 1.5 Ω; VGS(off) −2.5 to −4.5 V; leakage 10 µA (25 °C) / 250 µA (125 °C) at −5 V gate; 2017 revision marked "Preliminary" — all as stated. Ciss ≈ 5.3 nF, Qg ≈ 200 nC.
- **MSC080SMA330B4N**: 3,300 V and IDM 107 A as stated; differences in R-25.
- **UCC5350**: 100 kV/µs CMTI, 10 A peak, output supply up to 33 V, Miller-clamp option; adequate for the low side.
- **SN6505B**: 424 kHz typical as stated; soft-start problem in R-3.
- **TPS22918**: 1–5.5 V, 2 A, adjustable quick output discharge, −40 to +105 °C — as stated.
- **SN74LVC1G123**: retriggerable, has Ioff (R-4).
- **Gigavac G17**: DPDT, 12 V coil, intended as a defibrillator patient discharge relay; coil power and duty in R-12; no contact pulse or kV rating found.

---

## What could not be checked

- **No circuit simulation was available.** Settled only by simulation or measurement: the spike sizes and clamp energies of R-2 with real wiring inductance; high-side on/off times with any replacement for the SN6505B (R-3); depletion-stack release dynamics, voltage sharing during the transition and FBSOA margin (R-9); charger behaviour in the LT3751's regulation mode at 2 kV with a 1:50 transformer (R-7); charger interference in the impedance channel (§16 rank 4); the turn-off order when `BRIDGE_5V` collapses (R-1).
- **Datasheets not read:** UCC5350MC's output state with the input side unpowered and its under-voltage thresholds; MGJ2D052005SC start-up time; MAX30009 (sample timing, temperature range, drive behaviour when idle); ADS1292R respiration drive level and frequency options; MAX17320 short-circuit behaviour and any communication-shutdown mode; TLV3201, INA240 output swing, TPS62130 dropout, LM74502 inrush control; DPCR pulse ratings and value range; Ohmite EY ratings; Sanken UX-F5B forward voltage and lifecycle; the full MSC080SMA330B4N SOA numbers (the chart was seen only as extracted text).
- **The LT3751 datasheet** was read from the same distributor mirror the author used (ADI's server was not tried again); its text matched itself but was not compared with ADI's current revision.
- **Standards:** no IEC 60601-1 or 60601-2-4 clause was read. Patient leakage and auxiliary-current limits, the defibrillation-proof test circuit, energy-reduction limits and creepage at 2.2 kV remain unverified, as the design document says.
- **Board-level and mechanical questions:** creepage, potting, thermal paths and the enclosure were not reviewed.
- **Firmware changes** suggested in R-1, R-8 and R-13 were not designed or simulated here.
