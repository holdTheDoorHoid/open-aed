// SPDX-License-Identifier: GPL-3.0-or-later
//! Chest-compression sensing from the resistance between the pads
//! (decisions D-14 and D-36).
//!
//! NOT A MEDICAL DEVICE. See SAFETY.md at the repository root. This code has
//! run only in tests, in the simulator and on recordings from another
//! manufacturer's defibrillators; it has never run on this project's
//! hardware, which does not exist.
//!
//! # Why
//!
//! The heart signal alone cannot tell fast chest compressions from a
//! shockable rhythm (finding F-12). Every compression squeezes the chest and
//! changes the resistance between the pads by about an ohm. Measuring that
//! resistance many times a second shows when compressions are being given,
//! with no extra sensor. The same signal gives the compression rate, the
//! pauses and the share of time spent compressing (D-14).
//!
//! # What it gives
//!
//! [`CompressionDetector`] implements [`aed_types::ChestSensor`]:
//!
//! * a fast three-way answer, [`aed_types::ChestState`]: `HandsOff`,
//!   `Compressing`, or `Unknown`. It **fails safe**: whenever it cannot tell
//!   (no signal, saturated, out of range, stuck, settling, noisy, or
//!   movement that is not yet a clear run of compressions) it says
//!   `Unknown`, never `HandsOff`. The rescue sequence treats `Unknown` like
//!   `Compressing` for every shock decision.
//! * an event for each compression, with its size and the time since the
//!   previous one (so pauses can be recorded);
//! * the compression rate over the last five compressions;
//! * the share of the last minute spent compressing ("time on chest").
//!
//! # Input
//!
//! Pad impedance in milliohms, 125 samples a second
//! ([`aed_types::IMPEDANCE_SAMPLE_RATE_HZ`]), or "saturated". The whole
//! resistance, not just the change: the detector checks that it stays in
//! the 25 to 250 Ω pad-sensing range. What it assumes about the electronics
//! is written down on [`settings::FrontEndAssumptions`].
//!
//! # Latency and hysteresis
//!
//! Design bounds (measured figures, synthetic and real, are in the tests and
//! in `crates/aed-cpr-eval/REPORT.md`):
//!
//! * **Hands off → not hands off** (`Unknown` or `Compressing`): a
//!   compression is judged at its peak, once the signal has turned back by
//!   0.15 Ω. So the first compression of at least 0.3 Ω is seen within its
//!   own rise plus a few tens of milliseconds: about 0.3 s at 110 a minute
//!   (synthetic), median 0.33 s and 95th percentile 0.78 s on held-out real
//!   recordings. If the first compression is lost in the filter's start-up,
//!   the second is seen: at most one compression interval more. The rise
//!   may take at most [`Settings::max_rise_ms`] (600 ms).
//! * **Not hands off → hands off:** no fast movement for
//!   [`Settings::quiet_min_ms`] (1 s) after the last peak or trough, and
//!   1.6 compression intervals since the last compression while a run is
//!   under way (about 1.6 s at 60 a minute); plus [`Settings::settle_ms`]
//!   (1 s) if the signal was lost. Real recordings: median 1.05 s, 95th
//!   percentile 2.7 s after the reference end of compressions.
//! * **→ `Compressing`:** at the second compression of a run, about one
//!   compression interval after the first.
//! * **Signal lost → `Unknown`:** at the next sample (8 ms) for saturation
//!   or out of range; after [`Settings::stuck_ms`] (0.5 s) for a frozen
//!   reading. The rescue sequence adds its own check for samples that stop
//!   arriving.
//!
//! Hysteresis: getting into `Compressing` takes two compressions; getting
//! out of every not-hands-off state takes a full quiet period.
//!
//! # What else moves the impedance, and what the detector does with it
//!
//! * **Breaths (ventilations):** large (around an ohm) but slow: a breath
//!   goes in over about a second. A swing whose steep part (10 % to 90 %)
//!   takes longer than [`Settings::max_steep_ms`] (0.35 s), or whose whole
//!   rise takes longer than 0.6 s, is neither a compression nor movement.
//!   Breaths of up to 1 Ω leave the answer `HandsOff` in the synthetic
//!   tests; 2 Ω breaths are sometimes taken as movement (`Unknown` 15 to
//!   25 % of the time, never `Compressing`). A run of breaths is never
//!   `Compressing`: compressions in a run must be under 1.5 s apart.
//! * **Moving or carrying the patient, pressing the pads:** fast, irregular
//!   changes. Any fast change of 0.3 Ω or more gives `Unknown` for a second.
//!   That is the intended answer: the heart signal is spoiled by the same
//!   movement, and a responder pressing the pads is touching the patient.
//! * **The device's own shock:** saturates the front end: `Unknown`, then a
//!   1 s settling time after the signal returns.
//! * **The charger:** its switching noise is not modelled; the front end
//!   must keep it out (see [`settings::FrontEndAssumptions`]). Fast noise
//!   above a limit gives `Unknown`.
//! * **The heart itself:** a beating heart changes the pad impedance a
//!   little with each beat, by tens of milliohms and "rarely exceeding
//!   100 mΩ" (Urteaga et al., Entropy 2021;23:847). The 300 mΩ threshold is
//!   three times that, so the heart is not read as compressions, however
//!   fast it beats. Compressions are typically 1.1 to 1.7 Ω (Alonso et al.,
//!   Computing in Cardiology 2013;40:803); median 2.6 Ω in the Paris
//!   recordings. The tests feed a heart at 0.1 Ω and 0.2 Ω, up to 220 beats
//!   a minute.
//!
//! # Known weakness, found on real recordings
//!
//! Some patients' compressions change the impedance by only 0.15 to 0.25 Ω.
//! They are below the threshold and are MISSED: the answer is `HandsOff`
//! while compressions are being given. In one held-out case this lasted
//! 13.7 s. Lowering the threshold towards the heart's own 0.1 Ω would trade
//! this for the heart being read as compressions; it was not done (the
//! held-out data is spent) and is an open question in the evaluation report.
//! This is why the heart-signal rule for compression artifact in
//! `aed-rhythm` is kept as a second line of defence.
//!
//! What it cannot do: a hand resting still on the chest, or a very light
//! touch, changes the impedance too little to see. Compression sensing is
//! not a "someone is touching the patient" detector.
#![no_std]

#[cfg(test)]
extern crate std;

pub mod detector;
pub mod settings;

pub use detector::CompressionDetector;
pub use settings::{FrontEndAssumptions, Settings, SettingsError};

/// Milliseconds per impedance sample (8 ms at 125 per second).
pub const SAMPLE_MS: u32 = 1000 / aed_types::IMPEDANCE_SAMPLE_RATE_HZ;

/// Longest rolling period for time on chest the detector can store, seconds.
pub const MAX_ROLLING_SECONDS: usize = 120;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    missing_docs
)]
mod tests;
