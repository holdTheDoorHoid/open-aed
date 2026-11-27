// SPDX-License-Identifier: GPL-3.0-or-later
//! In-crate tests. Everything here runs against mocks only: a simulated
//! board with a crude capacitor model, a fake rhythm analyser and a fake
//! shock planner. No real hardware is involved.

mod chest_tests;
mod decision_tests;
mod mock;
mod parts_tests;
mod rescue_tests;
mod supervisor_tests;
mod walk_tests;
