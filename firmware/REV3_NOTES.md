# Firmware notes for hardware design revision 0.3

Written 27 November 2026 by the firmware work (AI-assisted), for the orchestrator to merge into
`docs/FINDINGS.md` and `DECISIONS.md`. **SIMULATION ONLY.** Everything below was checked against a
hand-written model of a circuit that has not been built. Nothing here is safe or ready for use on a
person or animal. Every hardware figure quoted is SIMULATED or CALCULATED in the hardware notes named
beside it; none is measured.

## In plain language

**What changed.** The firmware now follows design revision 0.3: it waits for the shock circuit's own
timers before each step (the waits are sized to the slowest unit the design allows, not the typical
one), each half of the shock now has its own hardware time limit and the firmware's longest halves are
shorter to fit under them, the patient relay's power supply is charged up before the relay is needed,
compression readings are thrown away while the high-voltage charger runs, the charger is restarted if it
stops silently but never if it stopped itself at its own safety limit, and the self-test now uses the
real shock permit (at an empty capacitor) to check the parts only a real shock uses, plus the new
over-voltage latch and all four "path held" sensors.

**What the simulator now models.** The simulated device has each new hardware part with its tolerance
band, and can be put at four "corners": typical, every part early, every part late, and the worst
combination for a real shock. A test fails if a firmware wait is shorter than the slowest hardware
(this is how integration issue I-9 was found on paper). Every new hardware part can be made to fail,
stuck either way.

**What the tests prove (in the model).** A real shock and the full periodic self-test work at all four
corners. Each firmware wait, made shorter than the hardware's worst case, makes the test fail. Every new
hardware fault the design says the self-test should find is found; the ones the design lists as hidden
stay hidden, and a test says so. In 112 self-tests, each with one fault and the capacitor starting at
0 V, 45 V, 300 V or 1,950 V, with the pads open or on a person, no sequence ever started into the pads
with the capacitor at 50 V or more (the highest seen was 14.6 V).

**What they do not prove.** That the real circuit has these timings (the bands come from hand-built
simulations); anything about the relay, which has not been chosen; that the energy figures match the
circuit simulation (this model's numbers differ by up to 7 %); anything about real compressions during
a charge.

**Discrepancies found** (details in §3): the self-test's hold probe would have failed every switch-on
with the slower v0.3 hold circuits (fixed: it now waits for the sensors); the compression blanking as
written would have blinded the chest check for the whole analysis and countdown and reset the analysis
at the end of every charge (fixed: the charger now runs only while charging or topping up, the
detector is restarted and gets 3 s to re-acquire, and a "shock" answer reached while blind waits until the chest has
been seen again); with the blanking, the "fresh window" of D-47 is no longer reached when compressions
stop during the charge (finding, §3.2); D-62 says 60 V where the design and change order say 50 V (50 V used,
more strictly); the self-test's switch checks run with an empty capacitor, so the current-based checks
of §B14.3 cannot see anything (open).

**Open items:** §4.

---

## 1. Each change-order item

| Item | What the firmware does now | Where | Test that fails without it |
|---|---|---|---|
| FW-1 | `CMD_HA` only when the relay's rest contact has read "left rest" without a break for 14 ms (restart on any sample at rest); give up 40 ms after `RELAY_CMD` | `aed-core/src/sequencer.rs` (new), `Config::relay_settle_ms`, `relay_settle_abort_ms` | `rev3.rs::a_relay_settle_wait_shorter_than_the_slowest_transfer_timer_is_refused`; `rev3_tests.rs::the_high_side_is_commanded_only_14_ms_after_the_last_sample_at_rest` |
| FW-2 | Nothing commanded until the rail has read armed for 30 ms | sequencer, `Config::rail_ready_wait_ms` | `a_test_relay_wait_shorter_than_the_slowest_btok_timer_is_refused` (15 ms case); `nothing_is_commanded_until_…` |
| FW-3 | The rail must read armed without a break for the whole 30 ms wait, so a < 10 ms armed pulse never leads to `RELAY_CMD` | sequencer, `Config::armed_min_ms` | `a_short_permit_pulse_never_leads_to_a_relay_command` |
| FW-4 | Model re-arm 60.5 ms max (band 51.24–60.34 ms); the self-test waits 75 ms from its own withdrawal before the next episode | `LatchTiming`, `Config::rearm_wait_ms`, `selftest.rs` Clear step | `the_caps_bands_are_ordered_and_above_the_firmware_maxima` (see §3.7 for why only that) |
| FW-5 | Model window 150 ms (band 131.8–161.5); `hardware_permit_cap_ms` 150; `permit_window_ms` 120 | `LatchTiming`, `config.rs` | `a_legitimate_shock_completes_at_every_corner…` (earliest window), config validation test |
| FW-6 | `sequence_start_timeout_ms` 80; the configuration now refuses one shorter than the rail wait + relay limit + 1 ms | `config.rs` (`rev3_ok`) | `the_sequence_start_timeout_covers_the_slowest_legitimate_start` |
| FW-7 | Test mode: `CMD_*` 7 ms after `BTEST_CMD` | sequencer, `Config::test_relay_wait_ms` | `a_test_relay_wait_shorter_than_the_slowest_btok_timer_is_refused` |
| FW-8 | `phase1_max` 10 ms, `phase2_max` 6.7 ms, `max_total` 17.2 ms (design table regenerated: limits only, no plan changed) | `aed-waveform/src/config.rs` | `aed-waveform` `firmware_phase_maxima_and_every_plan_fit_inside_the_hardware_caps`; `every_planned_shock_fits_inside_the_earliest_per_phase_caps` |
| FW-9 | `LatchTiming` has `phase1_cap_us` 11,370 and `phase2_cap_us` 7,730 with `EARLIEST`/`LATEST` bands; the sequence logic ends a sequence at either; new end `PhaseCapTripped` | `interlock.rs`, `hal.rs`, plant | `each_phase_has_its_own_cap`; `a_main_controller_frozen_in_phase_1_is_cut_by_the_phase_1_cap` |
| FW-10 | Over-voltage limit 2,070 V reading, both controllers | `config.rs` | `the_caps_bands_are_ordered…` |
| FW-11 | Documented on `set_discharge_hold` (1.5 kHz ± 10 %, stop low); four-line `hold_readback`; the self-test waits ≤ 25 ms for "not held" and ≤ 35 ms (1 s at switch-on, §3.9) for "held" | `hal.rs`, `selftest.rs` | the periodic self-test at every corner (the hold probe failed at nominal before the fix, §3.1) |
| FW-12 | New `set_coil_supply`. Rescue: raised once the charge has reached the plan with the charger off (coordinator's revision), kept through top-ups, forced up 500 ms before the end of the countdown at the latest; down on withdrawal, abort or the end. Self-test: up from the interlock steps, ≥ 200 ms before any relay command | `rescue.rs`, `selftest.rs`, `hal.rs` | `the_relay_refuses_without_the_coil_supply_lead`; `a_rescue_never_overlaps_the_coil_pre_charge_with_a_full_power_charge` |
| FW-13 | Impedance samples discarded while the charger is commanded on and 100 ms after; the channel-failure timer is paused across the gap; the charger is on only until the charge has settled at the charger's own set point (reading in the at-plan window, no rise for 100 ms), then again in a short burst as soon as the reading sags 1 V below where it stopped; a "shock" answer reached while blind does not count (§3.2) | `rescue.rs` (`ChargeControl`, `chest_blanked`) | `compression_readings_are_blanked_while_charging_without_failing_the_channel` |
| FW-14 | `Config::shock_current_uv_per_a` 24,000 and `current_present_ma` 100; the simulated board layer reports the test-load current | `config.rs`, `aed-sim/src/hw.rs` | `the_self_test_current_counts_as_present_at_the_test_voltage` |
| FW-15 | Self-test steps T-1 (real permit alone, 20 ms), T-2/T-3 (rail, relay move and return), T-4 (start without lead refused), and in the periodic tests T-3b (coil cap), T-5 (dry run, only with both front ends reading open), T-6 (over-voltage latch), T-8 (phase-1 and phase-2 caps), T-9 (overall cap, long gap); T-7 in the discharge-path steps | `selftest.rs`, `supervisor.rs`, `hal.rs::Probe` | `the_periodic_self_test_with_the_real_permit_steps_passes_at_every_corner`; `each_new_hardware_element_failing_is_found…`; `the_per_phase_caps_are_checked_against_their_bands` |
| FW-16 | The supervisor gives the REAL permit in a self-test only with both readings "empty" (below 50 V even with the sensor's full allowed error, i.e. a reading ≤ about 37 V), the charger reported off (new `MainStatus::charger_on`), and for T-5 its own pads reading open; ≤ 150 ms; withdrawn at once if any condition fails before the start | `supervisor.rs::decide_real_test_permit` | `the_supervisor_withdraws_a_real_self_test_permit_when_the_charger_runs_away`; `no_self_test_starts_a_sequence_into_the_pads…` |
| FW-17 | After a real `OVP_TRIP`: fault latched (`OvpTripped`), reset pulse (2 ms, ≥ 1 ms; hardware minimum about 100 µs through the 220 Ω resistor) only once both readings are empty, recorded; plus the coordinator's three additions (§2) | `supervisor.rs::run_ovp` | `a_real_over_voltage_trip_is_latched_and_reset_only_once_both_readings_are_empty`; `the_over_voltage_reset_needs_a_pulse_longer_than_the_hardware_minimum` |
| FW-18 | Below the at-plan window with the charger on, no rise of 2 V for 200 ms (after a 15 ms start time) → off for 2 ms, on again; never with the reading inside the window (a charger at its set point with a sensor reading low is not stalled), never if `CHG_DONE_N` (§2.4) | `rescue.rs::ChargeControl` | `a_charger_that_stops_silently_is_restarted_and_the_shock_still_goes` |
| FW-19 | `Config::battery_load_pulse_max_ms` 100, `battery_load_min_interval_ms` 1,000; the simulated board layer never pulses twice within 1 s | `config.rs`, `aed-sim/src/plant.rs` | `the_battery_load_pulse_is_not_repeated_within_a_second` |
| FW-20 | Not done: configuration records, not crates | — | — |

## 2. Additions asked for by the coordinator during the work

1. **Over-voltage test injection reads full scale (sense-sheet capture).** The plant's supervisor
   reading goes to 2,600 V while `OVP_TEST` is high. The supervisor suspends its voltage checks on its
   own reading (over-voltage, disagreement, charged-too-long) only while its T-6 step is running
   (`Supervisor::ovp_test_running`); the heartbeat check continues. Every stage of the step has its own
   time limit (2 ms, 50 ms, 2 ms, 30 ms, 2 ms, 2 ms), so the suspension cannot exceed about 90 ms; the
   test measured 50–100 ms. The step starts only with both readings "empty". Tests:
   `the_voltage_check_suspension_lasts_only_for_the_over_voltage_test_step` (after the step a real
   disagreement is latched as before; a second request is not a second test) and
   `the_over_voltage_test_is_entered_only_with_both_readings_empty` (300 V: `OVP_TEST` never rises).
2. **Reset held high for 50 ms before the injection.** Modelled fault `OvpResetCouplingShorted` (the
   reset then holds the latch clear): found as `OvpTestFailed`.
3. **Latch powered up tripped.** Reset at switch-on once both readings are empty; if it will not reset,
   `OvpTripped` is latched and the device is not ready. The main controller's "all four held" check
   after the link allows the link wait (1 s), not 35 ms, because the tripped latch holds both paths open
   until the supervisor has reset it. Test:
   `a_latch_that_powers_up_tripped_is_reset_at_switch_on_or_leaves_the_device_not_ready`.
4. **The charger stopping itself (`CHG_DONE_N`).** New HAL readback `charger_done`. While charging, the
   rescue checks it before anything else: if set, the charger is switched off, the capacitor emptied,
   fault `ChargerStoppedItself` (not ready, recorded), and FW-18's restart is never reached. The
   self-test's charge step fails the same way. **How the two cases are told apart:** by that line alone.
   A silent stop leaves `CHG_DONE_N` high; the charger's own last-resort trip sets it. **When they
   cannot be told apart:** if the `CHG_DONE_N` line itself has failed "not done" and the charger's trip
   then fires (two faults). If the trip fired in its normal band (2,116–2,316 V), both readings are
   above the 2,070 V firmware limit and the over-voltage fault empties the capacitor before any restart
   (the restart also never fires with the reading at or above the plan). If the trip fired early, below
   the plan, the restart would refill to below the plan, repeatedly, until the 12 s charge time limit
   ends it as a fault: no voltage above normal, but a wasted charge. Tests:
   `a_charger_that_ends_the_charge_by_its_own_trip_is_never_restarted` (an early trip below the plan,
   and an overshoot to the trip with the main sensor reading 10 % low).
5. **Coil-supply pre-charge not during a full-power charge (patient-interface capture).** See FW-12.
   The plant counts any millisecond in which the reservoir is filling while the charger delivers full
   power, and with a weak battery counts it as a brown-out. **Where the 200 ms cannot be met:** none in a
   rescue: the countdown (3 s) follows the end of the charge, and the supply is forced up 500 ms before
   its end at the latest. It could not be met only with a countdown shorter than about 0.7 s (D-30 says
   3 s; the configuration does not check this — open item). The forced raise accepts a top-up overlap,
   which the coordinator said is tolerable.
6. **Reset pulse minimum 100 µs (220 Ω on `OVP_RST`).** Firmware pulse 2 ms (≥ 1 ms whatever the
   1 ms tick does); the plant needs the line high for the minimum before the latch clears.

## 3. Discrepancies and findings

### 3.1 The hold probe of the self-test (new finding, fixed)

With the v0.3 hold circuits a path takes up to 22.2 ms to read "not held" and up to 32.9 ms to read
"held" again (rev3 notes §2). The D-61 self-test released the main controller's hold and then
immediately watched for the rail for 10 ms, then re-held and expected the rail within 10 ms. In the
simulation **every switch-on self-test failed at the typical unit** (`TestRailDidNotRise`), and with a
fast release it could also have shown a false "interlock passed without hold". Fixed: the request is
raised only once both path-A sensors read "not held", and again only once all four read "held".

### 3.2 Compression blanking as written blinds the whole analysis (fixed, needs a decision)

FW-13 blanks "while `CHG_RUN` is high". The firmware used to leave the charger commanded on, holding
the capacitor at the plan, for the whole analysis, countdown and delivery. Blanking on that basis would
have made the chest check rely on the heart signal alone from the start of the analysis to the shock,
not only for the 2–6 s of the charge that D-67 describes. The firmware now commands the charger only
until the charge has settled at the charger's own set point (reading in the at-plan window, no rise for
100 ms), and again in a short burst as soon as the reading sags 1 V below where it stopped. (A first
version stopped the charger when the main controller's reading reached the plan; that put the main
sensor's error into the capacitor voltage, and with in-spec sensors at opposite edges the supervisor
then refused the permit: simulated, `findings.rs::in_spec_sensor_errors_do_not_disable_the_device`. A
second version topped up only after a sag of half the at-plan window; that let a capacitor charged at
the edge of the window drift out of it during the countdown and fault: simulated,
`voltage_sensors.rs::delivered_energy_at_the_edges_of_the_allowed_voltage_window`. **The at-plan window
was sized for sensor error alone and has no room for droop**, so the charger must hold the voltage to
about 1 V.)

With the simulator's capacitor leakage (50 MΩ, about 0.4 V/s at 1,950 V) a top-up burst comes every
2–3 s and blanks the compression readings for about 0.1 s each: the chest check works for over 90 % of
the countdown. **With the v0.3 standing load (2.80 MΩ total, change order S-12, droop about 7 V/s) a
burst would be needed every 0.15 s or so, so the blanking would cover practically the whole time the
capacitor is held charged, analysis and countdown included.** D-67's stated consequence ("once charging
ends the chest check works again before the countdown") then does not hold. Not simulated (the plant's
leakage is still 50 MΩ; open item). This needs the owner's answer to Q-58: options (b) (pause the charger
at each reading) or (c) (a charger locked to the measurement), or a lower standing load, or a wider
at-plan window (which costs delivered-energy accuracy, F-9).

A second effect: when the blanking ends, the compression detector reports "cannot tell" until it has
new samples, which reset the analysis at the end of every charge (seen in simulation: "unable to
analyse"). Fixed by restarting the detector on the first sample after the gap (otherwise its filters read the
step across the gap as compressions: seen in simulation, "chest being compressed" for 5 s at the second
analysis) and treating "cannot tell" as "not sensed" for up to 3 s after a blanking gap, unless the
detector reports compressions.

**Effect on F-12:** while the capacitor charges, fast compressions over a flatline are judged on the
heart signal alone, as D-67 accepted (Q-58). In simulation that let a "shock" answer through
(compressions at 150/min never stopping: the device counted down, then aborted when the chest was seen
again). Fixed: a "shock" answer reached while the chest was blanked does not count; once the
compression sensing has looked at the chest again, compressions reset the analysis as before and with
hands off the NEXT "shock" answer is acted on. (Holding the blind answer instead moved the start of the
countdown 0.5 s off the analysis's one-second rhythm; in `vf-converts-during-countdown` a shock then went
2.56 s after the heart had recovered. Finding F-4 is that sensitive to where the countdown starts
relative to the analysis answers; worth stating in F-4.) With compression
sensing on, the device therefore no longer says "shock advised" while still charging (state `Charging`):
the advice comes when the charge has ended, which costs no time because the countdown waited for the
charge anyway.

**The chest check at the moment of the request (D-37).** A top-up burst at the end of the countdown
would blank the last chest check before the shock request (found by the aed-core random walks:
a request raised while the scripted chest was being compressed). Fixed: no top-up in the last second of
the countdown, and the request waits until the chest has been seen since the last blanking; if it has
not been seen within one more second the shock is held back as for movement. The countdown can
therefore last up to 4 s instead of 3 s in the worst case (not seen in the scenario runs).

**A dead impedance channel during a charge.** The failure timer is paused across a blanking gap only
if samples kept arriving (and were thrown away); a channel that delivers nothing is counted as failing
through the gap, as before. Without that, a lost channel was declared failed only after the charge,
and the first analysis ended "unable to analyse" (simulated, scenario `fault-impedance-lost`).

**Late stop of compressions (F-1):** with the blanking, a shock still follows from the first analysis
when compressions stop up to 3.0 s late at 400 µV VF, 3.5 s at 800 µV and 4.0 s at 1,500 µV
(`late_hands_off.rs`, SIMULATED), against the 3 s requirement; the analysis time is now set by the
charge (about 9 s in every case).

**Effect on D-47 and F-14 (new finding):** in the standard scenarios compressions stop during the
charge (the charge starts with the analysis). The hands-off moment is then not seen, so the analysis
does not start a "fresh window" on the running filters (D-47) and the hands-off pause gains back the
time D-47 saved. The D-47 tests now run with the blanking off to check the mechanism, plus a sweep with
it on that checks no non-shockable rhythm is shocked. Options for the owner (Q-58): accept; option (b)
of Q-58 (pause the charger briefly at each reading) would restore it.

### 3.3 50 V or 60 V (D-62)

D-62 says "below 60 V"; §B14.5 and FW-16 say 50 V. The firmware uses 50 V, made stricter: both
readings must count as empty with the sensor's full allowed error (`Config::reads_empty`), so the true
voltage is below 50 V for in-spec sensors. The hardware's `LOWV_OK` (60 V) does not gate the real
permit. Recommend correcting D-62's wording.

### 3.4 The start latches on the edge of `CMD_LB`

§B13.1 step 8 says sequence-active is set "at this edge". The plant models it so: a `CMD_LB` that rises
before every start condition is met is refused. If the drawn logic is level-sensitive, a too-early
`CMD_LB` would start the pulse late, part-way through the firmware's phase-1 time, shortening phase 1
(an under-dose, not caught by any cap). Worth checking in the second-round race simulation (S-5).

### 3.5 A toggling or frozen timer (R-4, D-63): better

* Frozen with phase 1 on: cut by the phase-1 cap at 10.75–11.85 ms. This model gives at most 159.6 J at
  1,950 V (25–175 Ω), against 170.9 J in the circuit simulation (rev3 notes §4). The two models differ
  (this one is a pure exponential with a fixed internal resistance); quote the circuit figure.
* Left toggling: a second `CMD_LB` is ignored, so it gets one short phase 1 and then waits in the gap
  until the overall cap: well under 40 J in the model, against a full pulse before.

### 3.6 The self-test's switch checks run with an empty capacitor (open)

§B14.4 says the firmware runs the test sequences at 40–50 V into the 200 Ω load, so that the shock
current shows. The self-test (D-61, unchanged order) runs its interlock and switch steps before the
charge, with the capacitor near 0 V. The current-based checks of §B14.3 (a shorted low side during the
lead, phase current present) therefore see nothing; in the simulation a shorted low side is not found.
The caps and the sequence order are still checked through the status lines. Fixing it means charging to
about 45 V first and stopping the charger before the test permit; not done.

### 3.7 What the firmware can see of the lockout (open)

The HAL reports "locked out" and "ended" after the rail has dropped. The drawn status lines (§B13.7)
read "not active / not done / no fault" with the rail off, so the real firmware cannot see when the
re-arm has completed: that is why FW-4 exists. The self-test now waits 75 ms from its own withdrawal
(and for the supervisor's report of its permit down). In the simulator the HAL still shows the lockout,
so a shorter wait is not caught by a plant test, only by the configuration check. The rescue's next
shock comes minutes later.

### 3.8 T-7 "the other pair stays low"

In the path-B step the main controller's path was re-held only just before, and takes up to 32.9 ms to
read "held" again. It is checked at the end of the 100 ms step, not at 25 ms.

### 3.9 Smaller points

* `test_permits_per_self_test` 4 → 6 (the periodic test now uses five test permits); real permits per
  self-test: 6.
* T-3b holds `RELAY_CMD` 115 ms; the relay must be back 80–108 ms after the command.
* A charger that ignores its set point is now masked: the firmware stops every charge on its own
  reading. It matters only together with a main sensor that reads low; then the charger's own trip ends
  the charge (§2.4).
* The supervisor's over-voltage fault stays until power-off: stricter than "not ready until a
  self-test passes".
* Switch-on self-test is longer: the real-permit steps and the 75 ms re-arm waits add about 0.4 s in the
  model (F-7: a weak charger is now reported about 1.15 s after switch-on, was 1.1 s).
* T-5 runs only when both front ends read the pads open (docked); with pads on a person it is skipped
  and runs at the next periodic test.
* A relay four times slower than modelled (64 ms to transfer) now fails: the firmware gives up 40 ms
  after `RELAY_CMD` (FW-1) and the switch-on self-test reports it (T-3). Before, it was shocked.

### 3.10 Existing tests changed, and why

* Stuck phase timer: now ends by the phase-1 cap (`PhaseCapTripped`), not the 25 ms cap (D-63).
* Unit-test harness (aed-core): runs without the real-permit steps, which its simple board does not
  model (`tests::cfg()`); the chest-logic unit tests run without the charge blanking (their slow mock
  charger would hide what they script). Both are covered by the simulator tests instead.
* The mock's "runaway" charger now also ignores "off", or the firmware's own stop would mask it.
* A sensor offset of 300 V now trips the 2,070 V over-voltage limit first; the disagreement case uses
  100 V.
* The periodic self-test counts four test sequences (latch, phase 1, phase 2, overall).
* Bare-plant tests in `consent_during_shock.rs` hold consent 70 ms longer (the v0.3 start takes about
  66 ms from the rail) and sweep in 250 µs steps; a toggling PERMIT line now gives the whole plan once
  (the dynamic detector holds consent through it, which is what the supervisor's clock is), a toggling
  request still gives nothing.
* "Without the charger rule a frozen main leaves the capacitor charged" now freezes during the charge
  (the firmware switches the charger off once a charge has settled).
* The relay-transfer failure is injected after the switch-on self-test, which would otherwise find it.

## 4. Open items

1. Plant capacitor leakage still 50 MΩ; the v0.3 standing load is 2.80 MΩ (S-12). Re-run the at-plan
   window and the countdown with it (§3.2).
2. The switch checks at 40–50 V (§3.6).
3. Lockout not observable on the status lines (§3.7).
4. Edge or level start (§3.4), second-round S-5.
5. A configuration check that the countdown leaves ≥ 200 ms for the coil supply (§2.5).
6. All plant bands come from first-round simulations; the relay figures are estimates (no relay chosen,
   D-64).
7. FW-20 (board configuration records).

## 5. Verification

Run on 27 November 2026 in `firmware/` with the pinned toolchain (1.99.1):

* `cargo fmt --check`: clean. (It was not clean before this work, in files this work did not
  otherwise touch, e.g. `aed-core/src/heat.rs`; `cargo fmt --all` was applied to the whole firmware
  workspace. The CI workflow does not run it.)
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.
* `cargo test --workspace`: **578 passed, 0 failed, 3 ignored** (was 542 passed). New: 26 in
  `aed-sim/tests/rev3.rs`, 8 in `aed-core/src/tests/rev3_tests.rs`, 1 in `aed-waveform`, 1 in
  `aed-sim/tests/fresh_window.rs`.
* Device crates build for `thumbv8m.main-none-eabihf` (as CI).
