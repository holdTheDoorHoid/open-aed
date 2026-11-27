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
//! 3. **Deciding** ([`features::classify`]): bad signal (at the front
//!    end's limit, flat-topped, implausibly large, noisy, or looking like
//!    compressions) gives `Indeterminate`; a near-flat or noise-like tiny
//!    signal gives `NonShockable` (asystole); `Shockable` needs **both**
//!    families to agree; if they disagree the answer is `Indeterminate`.
//! 4. **Confirming** ([`Confirmer`]): `Shockable` is only reported when the
//!    current window and at least [`settings::CONFIRM_WINDOWS`] of the last
//!    [`settings::CONFIRM_SPAN`] windows are shockable. Until then a
//!    shockable window is "still analysing" (`push` returns `None`). Every
//!    other verdict is reported at once.
//!
//! Every window's measurements and verdict are available from
//! [`Analyzer::last_report`] for logging.
//!
//! # What has been shown, and what has not
//!
//! The tests run this code against the made-up signals in [`synth`]. That
//! shows the code runs, does not crash, and behaves sensibly on cartoon
//! rhythms. Some thresholds in [`settings`] were then re-chosen on half of
//! the public PhysioNet recordings and tested once on the other half (see
//! `crates/aed-eval`): on the held-out half 71 % of coarse VF was caught,
//! against the AHA goal of 90 %. **The AHA 1997 goals are not met**, and
//! nothing here says anything about clinical accuracy: the recordings are
//! not cardiac arrests through pads, and real cardiac-arrest recordings
//! have not been tried.
//!
//! # Cost on a microcontroller (estimate, not measured)
//!
//! Per window (once a second): about 120 frequency lines x 1000 samples of
//! multiply-add for the spectrum (~120 000 operations) plus a few passes over
//! the window for the other features (the root-mean-square, the one-second
//! peak-to-peak values and the flat-top scan added in November 2026 are three
//! more passes of about 1000 simple operations each), and about 1000 cosine
//! evaluations for the taper. On a Cortex-M4F or M33 at ~100 MHz that should
//! be a few milliseconds per second of signal. Memory: about 10.5 KB, fixed
//! (two windows of decimal numbers plus, since November 2026, one window of
//! raw samples, 2 KB, for the flat-top check). Not measured on hardware.
#![no_std]

pub mod analyzer;
pub mod features;
pub mod filter;
pub mod settings;
pub mod synth;

pub use analyzer::{reason_label, Analyzer, Confirmed, Confirmer, WindowReport};
pub use features::{classify, classify_with, measure, Features, Reason, WindowVerdict};
pub use settings::Settings;
