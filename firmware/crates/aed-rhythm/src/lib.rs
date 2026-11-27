// SPDX-License-Identifier: GPL-3.0-or-later
//! Heart-rhythm analysis for the Open AED firmware.
//!
//! NOT A MEDICAL DEVICE. See SAFETY.md at the repository root.
//!
//! # What it does
//!
//! [`Analyzer`] receives the heart signal one sample at a time (250 per
//! second, microvolts) and, once a second, answers `Shockable`,
//! `NonShockable` or `Indeterminate` for the last 4 seconds.
//!
//! 1. **Cleaning** ([`filter`]): removes drift below 1 Hz, mains hum at 50
//!    and 60 Hz, and everything above 30 Hz.
//! 2. **Measuring** ([`features`]): on each 4-second window it measures
//!    signal quality, size, and four classical rhythm features from the
//!    published literature, grouped into two independent families
//!    (time domain and frequency domain).
//! 3. **Deciding** ([`features::classify`]): bad signal gives
//!    `Indeterminate`; a near-flat signal gives `NonShockable` (asystole);
//!    `Shockable` needs **both** families to agree; if they disagree the
//!    answer is `Indeterminate`.
//! 4. **Confirming** ([`Analyzer`]): `Shockable` is only reported after
//!    [`settings::CONFIRM_WINDOWS`] shockable windows in a row. Until then a
//!    shockable window is reported as `Indeterminate`.
//!
//! Every window's measurements and verdict are available from
//! [`Analyzer::last_report`] for logging.
//!
//! # What has been shown, and what has not
//!
//! The tests run this code against the made-up signals in [`synth`]. That
//! shows the code runs, does not crash, and behaves sensibly on cartoon
//! rhythms. **It says nothing about clinical accuracy.** The thresholds in
//! [`settings`] are starting values, several tuned only on those cartoons.
//! Real validation, first on the PhysioNet databases and then on real
//! cardiac-arrest recordings, has not been done. The AHA 1997 performance
//! goals (at least 90 % sensitivity for coarse VF, 75 % for rapid VT, 99 %
//! specificity for normal rhythm, 95 % for asystole and other rhythms) are
//! **unproven** for this code.
//!
//! # Cost on a microcontroller (estimate, not measured)
//!
//! Per window (once a second): about 120 frequency lines x 1000 samples of
//! multiply-add for the spectrum (~120 000 operations) plus a few passes over
//! the window for the other features, and about 1000 cosine evaluations for
//! the taper. On a Cortex-M4F at ~100 MHz that should be a few milliseconds.
//! Memory: about 8.5 KB, fixed. Not measured on hardware.
#![no_std]

pub mod analyzer;
pub mod features;
pub mod filter;
pub mod settings;
pub mod synth;

pub use analyzer::{reason_label, Analyzer, WindowReport};
pub use features::{classify, measure, Features, Reason, WindowVerdict};
