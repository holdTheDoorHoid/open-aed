// SPDX-License-Identifier: GPL-3.0-or-later
//! In-crate tests. Everything here runs against mocks only: a simulated
//! board with a crude capacitor model, a fake rhythm analyser and a fake
//! shock planner. No real hardware is involved.

mod chest_tests;
mod decision_tests;
mod heat_rescue_tests;
mod heat_tests;
mod mock;
mod parts_tests;
mod rescue_tests;
mod rev3_tests;
mod schedule_tests;
mod sequence_tests;
mod supervisor_tests;
mod walk_tests;

/// The configuration the unit tests use: the defaults, without the D-62
/// real-permit self-test steps, which the simple board model here does not
/// implement (the time-resolved simulator in aed-sim runs them).
pub(crate) fn cfg() -> crate::config::Config {
    crate::config::Config {
        self_test_real_permit_steps: false,
        ..crate::config::Config::default()
    }
}
