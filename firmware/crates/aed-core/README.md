# aed-core

The rescue sequence for the fully automatic design (D-06), the supervisor's rules, the hardware interlock logic, the self-test order, the prompts and the event record.

**This code has only ever run against test mocks.** Nothing here has controlled real hardware or been connected to a person. The "hardware" in the tests is a crude simulated board: a capacitor that charges at a fixed rate and empties through a resistor, a fake rhythm analyser and a fake shock planner. Passing tests show that the logic follows the rules below in that simulation. They do not show the device is safe. See [SAFETY.md](../../../SAFETY.md).

Every timing and limit is a provisional starting value in `src/config.rs`. None has yet been set from the resuscitation guidelines or from the defibrillator standard (IEC 60601-2-4), which has not been bought and read.

## How it is split

| Part | What it does |
|---|---|
| `config` | Every timing and limit, in one place, with the reasons |
| `hal` | Two separate lists of what each controller can see and do. The supervisor has its own clock, voltage sensor, pad reading, view of the cancel button, and its own fault alarm |
| `rescue` | The main controller's rescue sequence |
| `supervisor` | The second, independent controller. Shares nothing with the main controller except the status message |
| `interlock` | The hardware gate and the hardware rules (discharge by default; the charger cannot run while the discharge path is active), written as rules for the simulator to copy. They are plain logic on the real device, not software |
| `selftest` | The self-test order and time limits |
| `prompts` | Every spoken or displayed message |
| `events` | The fixed-size event record |
| `link` | The two messages the controllers send each other |

## Safety rules and the tests that cover them

Test names are in `src/tests/`. Rescue tests run the real supervisor alongside the main controller on the simulated board.

| Rule | Where it comes from | Test(s) |
|---|---|---|
| "Not sure" never leads to a shock | ARCHITECTURE.md | `indeterminate_never_charges`, `only_shockable_can_start_charging`, `not_sure_wins_over_shockable_within_one_analysis` |
| While analysing, "not sure" is not final: the device repeats "Do not touch the patient" and keeps analysing | Finding F-1 | `not_sure_during_analysis_is_not_final_and_a_later_shockable_charges`, `not_sure_then_non_shockable_goes_to_cpr_at_once`, `not_sure_during_analysis_repeats_hands_off_prompt_at_most_every_reminder_interval` |
| "Not sure" until the analysis time limit is "inconclusive": no shock, and the responder is told to start CPR | Finding F-1 | `not_sure_for_the_whole_analysis_goes_to_cpr_at_the_limit_as_inconclusive` |
| While armed, "not sure" still stops the shock | ARCHITECTURE.md, finding F-1 | `not_sure_during_countdown_still_aborts_the_shock`, `no_shock_when_rhythm_turns_indeterminate_during_charging` |
| "No shock" goes to CPR without charging | ARCHITECTURE.md | `non_shockable_goes_to_cpr_without_charging` |
| An analysis with no answer at all within the limit goes to CPR without a shock | This crate | `analysis_timeout_counts_as_not_sure_and_goes_to_cpr` |
| The rhythm is checked right up to delivery | ARCHITECTURE.md | `analysis_keeps_running_during_charging_and_countdown`, `no_shock_when_rhythm_turns_non_shockable_during_countdown`, `no_shock_when_rhythm_turns_indeterminate_during_charging`, `rhythm_that_flickers_non_shockable_once_still_aborts` |
| The "still shockable" confirmation must be recent at delivery | This crate (see below) | `stale_confirmation_at_delivery_gives_no_shock` |
| Rhythm is not analysed during CPR | This crate | `decisions_during_cpr_are_not_used` |
| Pads coming off before delivery stops the shock | ARCHITECTURE.md | `pads_removed_during_countdown_aborts`, `pads_removed_during_charging_aborts` |
| The plan uses the pad resistance measured just before delivery | Brief | `plan_is_made_from_resistance_measured_just_before_delivery` |
| No plan means no shock | Brief | `plan_error_at_delivery_disarms_without_shock`, `plan_voltage_that_does_not_match_the_charge_is_a_fault`, `planner_charge_voltage_above_overvoltage_limit_is_a_fault` |
| Cancel works in every state | ARCHITECTURE.md | `cancel_works_in_every_state`, `cancel_during_countdown_empties_capacitor`, `cancel_during_charging_empties_capacitor`, `cancel_during_analysis_goes_to_cpr_without_charging`, `cancel_during_delivering_withdraws_the_request`, `cancel_during_self_test_empties_and_faults_with_continue_cpr` |
| Changing adult/child while armed stops the shock | Brief | `patient_mode_change_during_countdown_aborts` |
| Child mode charges for a child and asks for front-and-back pads | D-04, D-05 | `child_mode_charges_to_child_voltage_and_announces_front_back_placement` |
| The shock is requested once; its outcome is recorded | Brief | `shock_is_requested_once_per_shock`, `full_rescue_vf_shock_cpr_normal_rhythm_cpr_exact_states_and_prompts` |
| After a shock: analysis reset, charge emptied and checked, then CPR | Brief | `analyzer_is_reset_after_delivery`, `residual_charge_is_emptied_and_verified_after_shock`, `shock_then_cpr_then_reanalyse_and_shock_again_if_still_shockable` |
| CPR lasts its period with a metronome, then re-analyses | ARCHITECTURE.md, D-14 | `cpr_metronome_runs_at_configured_rate_and_cpr_lasts_its_period`, `metronome_is_silent_outside_cpr_and_fault` |
| Pads off in analysis or CPR goes back to "attach pads" | ARCHITECTURE.md | `pads_off_during_analysis_goes_to_attach_pads`, `pads_off_during_cpr_goes_to_attach_pads_and_reattaching_reanalyses`, `no_pads_at_switch_on_prompts_attach_pads_and_waits`, `pad_glitch_shorter_than_debounce_is_ignored_during_analysis` |
| Any fault empties the capacitor and says "continue CPR" | ARCHITECTURE.md | `fault_always_leads_to_continue_cpr_prompt` (ten fault scenarios), `fault_state_keeps_metronome_and_reminds_to_continue_cpr`, `overvoltage_while_armed_is_a_fault`, `supervisor_latched_fault_puts_main_in_fault` |
| Capacitor not confirmed empty is its own, more serious fault | Brief | `capacitor_not_emptied_is_a_serious_fault_with_its_own_prompt`, `self_test_discharge_failure_is_the_serious_fault` |
| No state lasts longer than its limit | Brief | `charging_never_lasts_longer_than_its_limit`, `disarming_never_lasts_longer_than_its_limit`, `self_test_checks_have_time_limits`, random walks |
| Main controller alone cannot fire | ARCHITECTURE.md | `main_alone_cannot_fire`, `main_without_a_supervisor_cannot_even_charge` |
| Supervisor alone cannot fire | ARCHITECTURE.md | `supervisor_alone_cannot_fire` |
| Supervisor permits only when every condition holds | ARCHITECTURE.md | `supervisor_permits_when_every_condition_holds`, `supervisor_refuses_permit_when_voltage_reading_disagrees`, `supervisor_refuses_permit_when_voltage_not_at_plan`, `supervisor_refuses_permit_before_full_countdown_on_its_own_clock`, `supervisor_refuses_permit_when_countdown_was_skipped`, `supervisor_refuses_permit_when_own_pad_reading_out_of_range`, `supervisor_refuses_permit_when_cancel_pressed`, `supervisor_refuses_permit_when_heartbeat_stale` |
| Permit is short and single-use per arming | ARCHITECTURE.md | `supervisor_permit_lasts_only_a_short_window`, `permit_is_single_use`, `permit_is_available_again_after_a_new_arming` |
| Supervisor empties the capacitor by itself | ARCHITECTURE.md | `supervisor_empties_capacitor_on_heartbeat_loss`, `stuck_main_in_armed_state_is_emptied_by_supervisor`, `stuck_main_in_countdown_is_emptied_by_supervisor`, `supervisor_empties_capacitor_on_overvoltage`, `supervisor_empties_capacitor_on_disagreement_that_persists`, `supervisor_empties_capacitor_on_cancel` |
| Supervisor holds only when asked, and its faults are latched and reported | Brief | `supervisor_does_not_hold_before_hearing_main`, `supervisor_holds_only_while_main_claims_charging_or_armed`, `supervisor_faults_are_latched`, `supervisor_reports_latched_fault_back_to_main` |
| The supervisor has its own fault alarm, on for every latched fault, so a device whose main controller died is not silent | Finding F-3 | `supervisor_alarm_is_on_for_every_supervisor_fault_kind`, `supervisor_alarm_stays_off_through_a_normal_arming`, `supervisor_sounds_its_alarm_if_the_main_controller_never_starts`, random walks (invariant 4) |
| Two sensors with opposite errors inside their allowance agree; one sensor 8 % off, or dead, does not | Finding F-2 | `two_sensors_with_opposite_errors_inside_their_allowance_agree_and_read_at_plan`, `a_sensor_eight_percent_off_disagrees_at_the_charge_voltages` |
| "Empty" is only reported when the true voltage is below the empty voltage, even with sensor error | Finding F-2 | `empty_is_only_reported_when_the_true_voltage_is_below_the_empty_voltage` |
| Interlock: shock only with request AND permit AND discharge path off | ARCHITECTURE.md | `interlock_fires_only_with_request_and_permit_and_discharge_off`, `interlock_never_passes_without_both_signals`, `permit_with_request_fires_only_while_both_hold` |
| The capacitor discharges by default | ARCHITECTURE.md, REQUIREMENTS.md | `discharge_path_is_active_unless_both_controllers_hold` |
| The charger cannot run while the discharge path is active, so either controller letting go stops it (hardware requirement) | Finding F-3 | `charger_cannot_run_while_discharge_path_is_active` |
| Self-test: battery under load, supervisor link (and supervisor holding before the charger starts), charge, readings agree, discharge, pads | REQUIREMENTS.md, D-15 | `self_test_passes_on_good_hardware_and_leaves_capacitor_released`, `self_test_failure_codes_are_distinct_per_check`, `self_test_can_charge_to_full_energy_voltage`, `self_test_reports_low_battery_as_a_warning`, `only_pad_problems_are_advisory`, `self_test_abort_releases_capacitor`, `expired_pads_warn_but_the_rescue_continues` |
| Event record keeps the newest entries and counts losses | Brief | `event_record_keeps_order_oldest_first`, `event_record_overwrites_oldest_and_counts_overflow`, `event_record_of_zero_capacity_does_not_crash` |
| Configuration that contradicts itself is refused | Brief | `default_config_is_valid_and_matches_documented_values`, `contradictory_config_is_refused` |

### Invariants checked at every tick

`random_walks_never_break_a_safety_invariant` runs 400 repeatable random 90-second rescues (random rhythm changes, pad faults, cancel presses, mode changes, and in 40% of runs one hardware fault such as a dead charger, a broken discharge load, a lost message wire, a wrong voltage sensor or a frozen main controller). At every 2 ms tick it checks:

1. A shock request is only raised when the last analyser decision was "shockable", the pads were in range and cancel was not pressed.
2. The bridge only fired with request, permit and both holds present.
3. The supervisor never gives a permit while letting go of the capacitor.
4. A frozen main controller is released by the supervisor within the heartbeat timeout, and the supervisor's own fault alarm is on within one more heartbeat period.
5. In the Fault state, and in every state that is not charging, the charger is off and the main controller's hold is released.
6. No state lasts longer than its time limit.

`exhaustive_single_disturbance_at_every_moment_of_an_armed_cycle_prevents_the_shock` applies each of seven disturbances at every 250 ms from the start of analysis to after delivery, and checks that no shock follows it. Rhythm changes get one analyser interval to be noticed.

## Choices made where the design documents were not specific

- **How fresh "still shockable" must be.** At delivery the latest rhythm decision must be "shockable", no older than 5 seconds (provisional), and no other answer may have arrived since charging began. A single "not shockable" or "not sure" at any point while armed stops the shock, even if later answers say "shockable" again.
- **Cancel during the power-on self-test** counts as a failed self-test: the device has not yet shown it can empty its capacitor, so it goes to the Fault state and says "continue CPR".
- **A shock the supervisor does not permit** (the request times out) is recorded as not delivered, the capacitor is emptied and the device goes to CPR and tries again at the next analysis. It is not a fault on its own; the supervisor's own latched faults cover persistent problems.
- **Every supervisor fault is latched until power-off**, including the main controller's heartbeat stopping. A main controller that restarts mid-rescue therefore cannot shock again until the device is switched off and on.
- **Pad contact is smoothed** (200 ms) before the device acts on it, but the shock plan always uses a fresh, unsmoothed reading, and the supervisor checks its own reading.
- **Expired or missing pads** found by the self-test are a warning, not a fault, so the rescue continues.
- **"Not sure" while analysing is not final** (finding F-1). Compressions running on into the analysis are the usual cause, so the device says "Do not touch the patient. Still analysing." (at most every 3 s) and keeps looking. Only "shock" or "no shock" ends the analysis early. At the analysis time limit (12 s) with only "not sure" answers, the device says "Unable to analyse. No shock." and "Start CPR." While armed, any "not sure" still stops the shock.
- **Two voltage tolerances, not one** (finding F-2). *At the planned voltage*: a controller's own reading within 2 % + 20 V of the plan counts as charged and stays armed; both controllers check their own reading this way. *Agreement*: each sensor may be off by 2 % + 15 V, so the two readings agree while they differ by no more than twice that (at the higher reading). These defaults assume sensors accurate to ±2 % ± 10 V. "Empty" is only reported when the reading is below 50 V by a full sensor allowance (about 34 V or less). The overvoltage limit (2,200 V) stays a hard limit on each controller's own reading. The planner's charge voltage must match the charge target exactly (both are nominal numbers, not readings).
- **The charger is cut off whenever the discharge path is active** (finding F-3). This is a requirement on the future hardware, written down in `interlock::charger_may_run`. The main controller also only asks for the charger once the supervisor reports it is holding, and the self-test waits for that before its charge step.
- **The supervisor has its own fault alarm** (finding F-3): a buzzer and light it alone drives, on whenever it has a latched fault, and also if the main controller never sends a single message within 1 s of switch-on.
- **Power-on self-test charges to 400 V**, not full energy, so switching on in an emergency is not delayed. The full-energy test (D-15) is the same sequence given the full voltage, for periodic checks.
- **The Fault state is final** (as in the diagram), but keeps the metronome running and repeats "continue CPR".
