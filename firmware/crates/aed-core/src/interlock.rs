// SPDX-License-Identifier: GPL-3.0-or-later
//! The hardware interlock, written down as three pure functions.
//!
//! On the real device this is plain logic gates, not software: the third of
//! the three independent checks before a shock (ARCHITECTURE.md). These
//! functions describe what that logic must do, so that the simulator can use
//! them as its model and the tests can check every combination. Nothing in
//! the firmware calls them to make a decision.

/// True when the discharge path is active, meaning the capacitor is being
/// emptied into the internal load.
///
/// The path is active unless BOTH controllers hold it open ("discharges by
/// default"). Either controller letting go, or losing power, empties the
/// capacitor.
#[must_use]
pub const fn discharge_path_active(main_hold: bool, supervisor_hold: bool) -> bool {
    !(main_hold && supervisor_hold)
}

/// True when the output bridge may fire.
///
/// It fires only when the main controller's shock request AND the
/// supervisor's permit are both present AND the discharge path is not active.
/// Any one missing blocks the shock.
#[must_use]
pub const fn bridge_may_fire(
    main_request: bool,
    supervisor_permit: bool,
    discharge_active: bool,
) -> bool {
    main_request && supervisor_permit && !discharge_active
}

/// True when the capacitor charger is able to run.
///
/// **This is a requirement on the future hardware design**, not something
/// the firmware decides: the charger's power stage must be cut off in
/// hardware whenever the discharge path is active, whatever the main
/// controller is asking for. Because the discharge path is active whenever
/// EITHER controller lets go (see [`discharge_path_active`]), the
/// supervisor can always stop the charger on its own simply by letting go,
/// even if the main controller has frozen with its charger command stuck
/// on. Without this rule a frozen main controller leaves the charger
/// feeding the discharge load and the capacitor never empties (simulation
/// finding F-3).
///
/// `charger_enabled` is the main controller's charger command;
/// `discharge_active` is the output of [`discharge_path_active`].
#[must_use]
pub const fn charger_may_run(charger_enabled: bool, discharge_active: bool) -> bool {
    charger_enabled && !discharge_active
}
