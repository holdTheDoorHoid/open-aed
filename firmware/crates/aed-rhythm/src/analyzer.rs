// SPDX-License-Identifier: GPL-3.0-or-later
//! The analyser: takes one sample at a time, keeps the last few seconds, and
//! produces a decision once per step.

use crate::features::{classify_with, measure, Features, Reason, WindowVerdict};
use crate::filter::Preprocessor;
use crate::settings::{
    Settings, CLIP_MIN_SPAN_UV, CLIP_RUN_SAMPLES, FRESH_WINDOW_GUARD_SAMPLES, SATURATION_UV,
    STEPS_PER_WINDOW, STEP_SAMPLES, WARMUP_SAMPLES, WINDOW_SAMPLES,
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
    /// How many of the last [`Settings::confirm_span`] windows (including
    /// this one) said shockable.
    pub recent_shockable: u8,
    /// What the analyser decided for this window. Equals the window
    /// verdict, except that a shockable window is recorded here as
    /// `Indeterminate` until the confirmation rule is met. While that
    /// confirmation is pending, `push` returns `None` (still analysing)
    /// instead of announcing a decision.
    pub decision: RhythmDecision,
    /// What `push` returned for this window: `None` means "still analysing".
    pub announced: Option<RhythmDecision>,
}

/// The confirmation rule, kept apart from the signal processing so that it
/// can be replayed on stored window verdicts (by the evaluation tools) and
/// gives exactly what the analyser gives.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Confirmer {
    /// Bit 0 = latest window was shockable, bit 1 = the one before, ...
    history: u8,
    consecutive: u8,
}

/// What [`Confirmer::step`] decided for one window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Confirmed {
    /// The decision recorded for the window.
    pub decision: RhythmDecision,
    /// What is announced to the rescue sequence (`None` = still analysing).
    pub announced: Option<RhythmDecision>,
    /// Shockable windows in a row, including this one.
    pub consecutive: u8,
    /// Shockable windows among the last `confirm_span`, including this one.
    pub recent: u8,
}

impl Confirmer {
    /// Take one window verdict.
    pub fn step(&mut self, verdict: &WindowVerdict, s: &Settings) -> Confirmed {
        let shockable = verdict.decision == RhythmDecision::Shockable;
        self.history = (self.history << 1) | u8::from(shockable);
        self.consecutive = if shockable {
            self.consecutive.saturating_add(1)
        } else {
            0
        };
        let span = s.confirm_span.clamp(1, 8);
        let mask = if span >= 8 {
            u8::MAX
        } else {
            (1_u8 << span) - 1
        };
        let recent = (self.history & mask).count_ones() as u8;
        let need = s.confirm_windows.max(1);
        let (decision, announced) = if shockable {
            if recent >= need {
                (RhythmDecision::Shockable, Some(RhythmDecision::Shockable))
            } else {
                // A shockable window that is not yet confirmed is "still
                // analysing", not an answer. Reporting it as Indeterminate
                // would make the rescue sequence repeat its hands-off prompt
                // for no reason, and would abandon a shock already being
                // prepared (while armed, any Indeterminate stops the shock).
                (RhythmDecision::Indeterminate, None)
            }
        } else {
            (verdict.decision, Some(verdict.decision))
        };
        Confirmed {
            decision,
            announced,
            consecutive: self.consecutive,
            recent,
        }
    }
}

/// Running counts for one step-sized block of the window.
#[derive(Debug, Clone, Copy, Default)]
struct BlockStats {
    saturated: u32,
    noise_energy: f32,
}

/// The rhythm analyser. Implements [`RhythmAnalyzer`].
///
/// Memory use is fixed: two windows of decimal numbers (about 8 KB), one
/// window of raw samples (2 KB) and a few hundred bytes. Nothing is
/// allocated while running.
#[derive(Debug, Clone)]
pub struct Analyzer {
    settings: Settings,
    pre: Preprocessor,
    ring: [f32; WINDOW_SAMPLES],
    raw: [EcgSample; WINDOW_SAMPLES],
    scratch: [f32; WINDOW_SAMPLES],
    write_pos: usize,
    filled: usize,
    warmup_left: usize,
    since_step: usize,
    blocks: [BlockStats; STEPS_PER_WINDOW],
    block_pos: usize,
    confirmer: Confirmer,
    windows: u32,
    last: Option<WindowReport>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Count raw samples that sit in flat-topped runs at the window's highest or
/// lowest value (at least [`CLIP_RUN_SAMPLES`] identical samples in a row),
/// provided the window spans at least [`CLIP_MIN_SPAN_UV`]. `samples` yields
/// the window in time order.
fn clipped_samples(samples: impl Iterator<Item = EcgSample> + Clone) -> u32 {
    let (lo, hi) = samples
        .clone()
        .fold((EcgSample::MAX, EcgSample::MIN), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        });
    if i32::from(hi) - i32::from(lo) < CLIP_MIN_SPAN_UV {
        return 0;
    }
    let mut total = 0_u32;
    let mut run = 0_u32;
    let mut run_value = 0;
    let close = |run: u32, value: EcgSample| {
        if (value == hi || value == lo) && run as usize >= CLIP_RUN_SAMPLES {
            run
        } else {
            0
        }
    };
    for v in samples {
        if run > 0 && v == run_value {
            run += 1;
        } else {
            total += close(run, run_value);
            run = 1;
            run_value = v;
        }
    }
    total + close(run, run_value)
}

impl Analyzer {
    /// A fresh analyser with the device's settings and nothing seen yet.
    #[must_use]
    pub fn new() -> Self {
        Self::with_settings(Settings::DEFAULT)
    }

    /// A fresh analyser with other settings (for evaluation tools; the
    /// device uses [`Analyzer::new`]).
    #[must_use]
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            settings,
            pre: Preprocessor::new(),
            ring: [0.0; WINDOW_SAMPLES],
            raw: [0; WINDOW_SAMPLES],
            scratch: [0.0; WINDOW_SAMPLES],
            write_pos: 0,
            filled: 0,
            warmup_left: WARMUP_SAMPLES,
            since_step: 0,
            blocks: [BlockStats::default(); STEPS_PER_WINDOW],
            block_pos: 0,
            confirmer: Confirmer::default(),
            windows: 0,
            last: None,
        }
    }

    /// Start a fresh analysis window now, keeping the filters' state, and
    /// discard the next `guard_samples` samples (or whatever is left of the
    /// settling after the last full reset, if that is longer) before the
    /// window starts filling. The device uses
    /// [`RhythmAnalyzer::restart_window`], which uses
    /// [`FRESH_WINDOW_GUARD_SAMPLES`]; this form exists so the measurement
    /// tests can show what other guards would do.
    pub fn restart_window_with_guard(&mut self, guard_samples: usize) {
        let warmup_left = self.warmup_left.max(guard_samples);
        let pre = self.pre;
        *self = Self::with_settings(self.settings);
        self.pre = pre;
        self.warmup_left = warmup_left;
    }

    /// The settings in use.
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings
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
        let (saturated, noise_energy) = self.blocks.iter().fold((0_u32, 0.0_f32), |(s, n), b| {
            (s + b.saturated, n + b.noise_energy)
        });
        let clipped = clipped_samples(
            self.raw
                .iter()
                .cycle()
                .skip(self.write_pos)
                .take(WINDOW_SAMPLES)
                .copied(),
        );
        let features = measure(&mut self.scratch, saturated, clipped, noise_energy);
        let verdict = classify_with(&features, &self.settings);
        let c = self.confirmer.step(&verdict, &self.settings);
        self.windows = self.windows.saturating_add(1);
        WindowReport {
            window_number: self.windows,
            features,
            verdict,
            consecutive_shockable: c.consecutive,
            recent_shockable: c.recent,
            decision: c.decision,
            announced: c.announced,
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
        if let Some(slot) = self.raw.get_mut(self.write_pos) {
            *slot = sample;
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
            report.announced
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
        *self = Self::with_settings(self.settings);
    }

    fn track(&mut self, sample: EcgSample) {
        // The filters run exactly as they do inside `push`; the cleaned
        // sample is thrown away. Samples tracked count toward the settling
        // needed after a full reset.
        let _ = self.pre.process(f32::from(sample));
        self.warmup_left = self.warmup_left.saturating_sub(1);
    }

    fn restart_window(&mut self) {
        self.restart_window_with_guard(FRESH_WINDOW_GUARD_SAMPLES);
    }
}

/// A short label for a [`Reason`], for printed tables and logs.
#[must_use]
pub fn reason_label(reason: Reason) -> &'static str {
    match reason {
        Reason::InvalidMeasurement => "invalid measurement",
        Reason::Saturated => "saturated",
        Reason::Clipped => "clipped (flat tops)",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{Rhythm, Synth, Xorshift32};

    fn verdict(decision: RhythmDecision, reason: Reason) -> WindowVerdict {
        WindowVerdict {
            decision,
            reason,
            time_domain_vote: false,
            frequency_domain_vote: false,
        }
    }

    #[test]
    fn three_of_five_confirms_through_one_doubtful_window() {
        let s = Settings::DEFAULT;
        let shock = verdict(RhythmDecision::Shockable, Reason::BothMethodsShockable);
        let doubt = verdict(RhythmDecision::Indeterminate, Reason::MethodsDisagree);
        let mut c = Confirmer::default();
        assert_eq!(c.step(&shock, &s).announced, None);
        assert_eq!(c.step(&shock, &s).announced, None);
        // The doubtful window is announced as "not sure" (it never holds a shock back silently).
        assert_eq!(
            c.step(&doubt, &s).announced,
            Some(RhythmDecision::Indeterminate)
        );
        // 3 of the last 4 now shockable, current one too: confirmed.
        assert_eq!(
            c.step(&shock, &s).announced,
            Some(RhythmDecision::Shockable)
        );
    }

    #[test]
    fn a_non_shockable_window_is_announced_at_once_even_after_confirmation() {
        let s = Settings::DEFAULT;
        let shock = verdict(RhythmDecision::Shockable, Reason::BothMethodsShockable);
        let mut c = Confirmer::default();
        for _ in 0..3 {
            let _ = c.step(&shock, &s);
        }
        let none = verdict(RhythmDecision::NonShockable, Reason::NeitherMethodShockable);
        assert_eq!(
            c.step(&none, &s).announced,
            Some(RhythmDecision::NonShockable)
        );
        let doubt = verdict(RhythmDecision::Indeterminate, Reason::MethodsDisagree);
        assert_eq!(
            c.step(&doubt, &s).announced,
            Some(RhythmDecision::Indeterminate)
        );
        assert_eq!(
            c.step(&none, &s).announced,
            Some(RhythmDecision::NonShockable)
        );
        // Only two shockable windows among the last five: not confirmed.
        assert_eq!(c.step(&shock, &s).announced, None);
    }

    #[test]
    fn flat_topped_signal_is_clipped_not_shockable() {
        let mut a = Analyzer::new();
        for i in 0..250 * 8 {
            // A 5 Hz wave of 8 mV clipped flat at ±2 mV.
            let v = 8000.0 * libm::sinf(2.0 * core::f32::consts::PI * 5.0 * i as f32 / 250.0);
            let _ = a.push(v.clamp(-2000.0, 2000.0) as EcgSample);
        }
        let r = a.last_report().copied();
        assert_eq!(r.map(|r| r.verdict.reason), Some(Reason::Clipped));
        assert!(r.is_some_and(|r| r.features.clipped_fraction > 0.5));
    }

    #[test]
    fn recorder_noise_is_flatline_but_a_small_regular_wave_is_not() {
        // Broadband noise of 40 µV rms before the analyser's 1-30 Hz filter
        // (about 20 µV after it; peak to peak over 100 µV).
        let mut rng = Xorshift32::new(5);
        let mut a = Analyzer::new();
        for _ in 0..250 * 8 {
            let _ = a.push((40.0 * rng.next_gaussian()) as EcgSample);
        }
        let r = a.last_report().copied();
        assert!(r.is_some_and(|r| r.features.peak_to_peak_uv > 100.0));
        assert_eq!(r.map(|r| r.verdict.reason), Some(Reason::Asystole));
        // Made-up fine VF of similar rms is regular (low leakage): whenever
        // it is over the old 100 µV peak-to-peak line, the rms rule must not
        // call it flatline either.
        let mut checked = 0;
        for seed in 1..=10 {
            let mut a = Analyzer::new();
            let mut vf = Synth::new(Rhythm::fine_vf(), seed);
            let mut seen = 0;
            for _ in 0..250 * 12 {
                let _ = a.push(vf.next_sample());
                if let Some(r) = a.last_report() {
                    if r.window_number != seen {
                        seen = r.window_number;
                        if r.features.peak_to_peak_uv >= 100.0 && r.features.rms_uv < 25.0 {
                            checked += 1;
                            assert_ne!(r.verdict.reason, Reason::Asystole);
                        }
                    }
                }
            }
        }
        assert!(
            checked > 0,
            "no fine-VF window fell in the range the guard is for"
        );
    }
}
