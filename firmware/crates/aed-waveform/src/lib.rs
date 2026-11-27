// SPDX-License-Identifier: GPL-3.0-or-later
//! Shock planning and the electrical model of the shock.
//!
//! NOT A MEDICAL DEVICE. See SAFETY.md at the repository root. This crate is
//! design arithmetic and simulation only. Every number it produces is a
//! MODEL figure, not a measurement: no hardware exists.
//!
//! # What the shock is
//!
//! A capacitor (about 100 µF) is charged to a fixed voltage, then connected
//! to the patient through the pads. Current flows and dies away as the
//! capacitor empties (an "exponential" decay). The device cuts the current
//! off early ("truncated"), waits a fraction of a millisecond, reverses the
//! connection and lets current flow the other way for a shorter time
//! ("biphasic"). Whatever charge is left is then dumped inside the device.
//! This is the biphasic truncated exponential shape chosen in decision D-02.
//!
//! # What this crate does
//!
//! - [`WaveformConfig`] holds every design number in one place.
//! - [`discharge`] works out what a given shock does to a patient of a given
//!   resistance: currents, energies, charge, tilt ([`DischargeResult`]).
//! - [`BtePlanner`] implements the shared [`aed_types::ShockPlanner`]
//!   contract: from the measured pad resistance it picks how long each phase
//!   lasts so that the delivered energy lands on the target.
//! - [`design_table`] and [`write_report`] produce the full design table
//!   (saved in `DESIGN_TABLE.md` next to this crate).
//!
//! # The electrical model, in brief
//!
//! The capacitor, the device's own internal resistance and the patient form
//! one loop. With total resistance `R = R_internal + R_patient`, the voltage
//! falls as `V(t) = V0 · e^(−t / (R·C))` while current flows, and the
//! current is `V(t) / R`. Energy taken from the capacitor is split between
//! the patient and the device's internal resistance in proportion to their
//! resistances. During the gap between phases no current flows and the
//! capacitor holds its voltage. Ignored on purpose (they are small or
//! unknown today): switch voltage drops, wiring inductance, capacitor
//! leakage and dielectric absorption, and any change of the patient's
//! resistance during the shock.
//!
//! Arithmetic is done in 64-bit floating point through `libm`, because a
//! shock is planned once, not in a fast loop, and the extra precision keeps
//! the energy book-keeping exact enough to test.
#![no_std]

mod config;
mod model;
mod planner;
mod table;

pub use config::{ConfigError, WaveformConfig};
pub use model::{discharge, DischargeResult};
pub use planner::{BtePlanner, LimitHit};
pub use table::{design_row, design_table, write_report, DesignRow, TABLE_STEP_OHMS};
