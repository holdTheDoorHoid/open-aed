// SPDX-License-Identifier: GPL-3.0-or-later
//! The analyser: takes one sample at a time, keeps the last few seconds, and
//! produces a decision once per step.

use crate::features::{classify, measure, Features, Reason, WindowVerdict};
use crate::filter::Preprocessor;
use crate::settings::{
    CONFIRM_WINDOWS, SATURATION_UV, STEPS_PER_WINDOW, STEP_SAMPLES, WARMUP_SAMPLES, WINDOW_SAMPLES,
};
use aed_types::{EcgSample, RhythmAnalyzer, RhythmDecision};

/// Everything about one analysed window, kept for the event record and for
/// inspection in tests and the simulator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowReport {
    /// Counts windows since start or the last reset, starting at 1.
    pub window_number: u32,
    /// The raw measurements.
    pub features: Features,
    /// The verdict for this window on its own.
    pub verdict: WindowVerdict,
    /// How many windows in a row (including this one) have said shockable.
    pub consecutive_shockable: u8,
    /// What the analyser actually reported for this window. Equals the
    /// window verdict, except that a shockable window is recorded here as
    /// `Indeterminate` until [`CONFIRM_WINDOWS`] in a row agree. While that
    /// confirmation is pending, `push` returns `None` (still analysing)
    /// instead of announcing a decision.
    pub decision: RhythmDecision,
}

/// Running counts for one step-sized block of the window.
#[derive(Debug, Clone, Copy, Default)]
struct BlockStats {
    saturated: u32,
    noise_energy: f32,
}

/// The rhythm analyser. Implements [`RhythmAnalyzer`].
///
/// Memory use is fixed: two windows of decimal numbers (about 8 KB) plus a
/// few hundred bytes. Nothing is allocated while running.
#[derive(Debug, Clone)]
pub struct Analyzer {
    pre: Preprocessor,
    ring: [f32; WINDOW_SAMPLES],
    scratch: [f32; WINDOW_SAMPLES],
    write_pos: usize,
    filled: usize,
    warmup_left: usize,
    since_step: usize,
    blocks: [BlockStats; STEPS_PER_WINDOW],
    block_pos: usize,
    consecutive_shockable: u8,
    windows: u32,
    last: Option<WindowReport>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    /// A fresh analyser with nothing seen yet.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pre: Preprocessor::new(),
            ring: [0.0; WINDOW_SAMPLES],
            scratch: [0.0; WINDOW_SAMPLES],
            write_pos: 0,
            filled: 0,
            warmup_left: WARMUP_SAMPLES,
            since_step: 0,
            blocks: [BlockStats::default(); STEPS_PER_WINDOW],
            block_pos: 0,
            consecutive_shockable: 0,
            windows: 0,
            last: None,
        }
    }

    /// The report for the most recent window, if any window has been analysed
    /// since start or the last reset.
    #[must_use]
    pub fn last_report(&self) -> Option<&WindowReport> {
        self.last.as_ref()
    }

    fn analyse(&mut self) -> WindowReport {
        // Copy the ring into time order (oldest first).
        for (dst, src) in self
            .scratch
            .iter_mut()
            .zip(self.ring.iter().cycle().skip(self.write_pos))
        {
            *dst = *src;
        }
        let (saturated, noise_energy) = self
            .blocks
            .iter()
            .fold((0_u32, 0.0_f32), |(s, n), b| (s + b.saturated, n + b.noise_energy));
        let features = measure(&mut self.scratch, saturated, noise_energy);
        let verdict = classify(&features);

        let decision = if verdict.decision == RhythmDecision::Shockable {
            self.consecutive_shockable = self.consecutive_shockable.saturating_add(1);
            if self.consecutive_shockable >= CONFIRM_WINDOWS {
                RhythmDecision::Shockable
            } else {
                RhythmDecision::Indeterminate
            }
        } else {
            self.consecutive_shockable = 0;
            verdict.decision
        };
        self.windows = self.windows.saturating_add(1);
        WindowReport {
            window_number: self.windows,
            features,
            verdict,
            consecutive_shockable: self.consecutive_shockable,
            decision,
        }
    }
}

impl RhythmAnalyzer for Analyzer {
    fn push(&mut self, sample: EcgSample) -> Option<RhythmDecision> {
        let cleaned = self.pre.process(f32::from(sample));
        if self.warmup_left > 0 {
            self.warmup_left -= 1;
            return None;
        }

        if let Some(slot) = self.ring.get_mut(self.write_pos) {
            *slot = cleaned.signal_uv;
        }
        self.write_pos = (self.write_pos + 1) % WINDOW_SAMPLES;
        self.filled = (self.filled + 1).min(WINDOW_SAMPLES);

        if let Some(block) = self.blocks.get_mut(self.block_pos) {
            if sample >= SATURATION_UV || sample <= -SATURATION_UV {
                block.saturated = block.saturated.saturating_add(1);
            }
            block.noise_energy += cleaned.noise_uv * cleaned.noise_uv;
        }

        self.since_step += 1;
        if self.since_step < STEP_SAMPLES {
            return None;
        }
        self.since_step = 0;

        let result = if self.filled >= WINDOW_SAMPLES {
            let report = self.analyse();
            self.last = Some(report);
            // A shockable window that is not yet confirmed is "still
            // analysing", not an answer. Reporting it as Indeterminate would
            // make the rescue sequence repeat its hands-off prompt for no
            // reason while analysing, and would abandon a shock already
            // being prepared (while armed, any Indeterminate stops the shock).
            let awaiting_confirmation = report.verdict.decision == RhythmDecision::Shockable
                && report.decision != RhythmDecision::Shockable;
            if awaiting_confirmation {
                None
            } else {
                Some(report.decision)
            }
        } else {
            None
        };

        // Start a fresh block, overwriting the oldest one.
        self.block_pos = (self.block_pos + 1) % STEPS_PER_WINDOW;
        if let Some(block) = self.blocks.get_mut(self.block_pos) {
            *block = BlockStats::default();
        }
        result
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

/// A short label for a [`Reason`], for printed tables and logs.
#[must_use]
pub fn reason_label(reason: Reason) -> &'static str {
    match reason {
        Reason::InvalidMeasurement => "invalid measurement",
        Reason::Saturated => "saturated",
        Reason::AmplitudeImplausible => "amplitude implausible",
        Reason::Asystole => "asystole",
        Reason::Noisy => "noisy",
        Reason::PossibleArtifact => "possible artifact",
        Reason::FineVf => "fine VF (not shocked)",
        Reason::MethodsDisagree => "methods disagree",
        Reason::BothMethodsShockable => "both methods shockable",
        Reason::NeitherMethodShockable => "neither method shockable",
    }
}
