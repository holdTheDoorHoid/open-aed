// SPDX-License-Identifier: GPL-3.0-or-later
//! Turning a run into readable text: the timeline, the summary block and
//! the timing table.

use std::fmt::Write as _;

use crate::runner::{RunResult, ShockTiming};

/// Time as minutes:seconds.tenths, e.g. `2:03.4`.
#[must_use]
pub fn clock(ms: u32) -> String {
    let tenths = ms / 100;
    format!("{}:{:02}.{}", tenths / 600, (tenths / 10) % 60, tenths % 10)
}

/// Seconds with one decimal, e.g. `6.3 s`.
#[must_use]
pub fn secs(ms: u32) -> String {
    format!("{:.1} s", f64::from(ms) / 1000.0)
}

/// The whole timeline, one line per entry.
#[must_use]
pub fn timeline(r: &RunResult) -> String {
    let mut out = String::new();
    for e in &r.timeline {
        let _ = writeln!(out, "{:>8}  {}  {}", clock(e.t_ms), e.source.label(), e.text);
    }
    out
}

/// The summary block.
#[must_use]
pub fn summary(r: &RunResult) -> String {
    let mut out = String::new();
    let s = &r.scenario;
    let _ = writeln!(out, "SUMMARY: {}  (SIMULATION ONLY; provisional settings)", s.name);
    let _ = writeln!(out, "  Scenario:            {}", s.description);
    let _ = writeln!(out, "  Simulated time:      {}", secs(s.duration_ms));
    match r.pads_on_to_first_shock_ms() {
        Some(ms) => {
            let _ = writeln!(out, "  Pads on -> 1st shock: {}", secs(ms));
        }
        None => {
            let _ = writeln!(out, "  Pads on -> 1st shock: no shock");
        }
    }
    let _ = writeln!(
        out,
        "  Shocks delivered:    {}   (charges started: {})",
        r.shocks.len(),
        r.charges_started
    );
    for (i, sh) in r.shocks.iter().enumerate() {
        let _ = writeln!(
            out,
            "    #{} at {}: {:.1} J into {} (target {:.0} J), peak {:.1} A, from {:.0} V, {:.0} V left (model)",
            i + 1,
            clock(sh.at_ms),
            sh.delivered_j(),
            sh.patient_ohms
                .map_or_else(|| "open pads".to_string(), |o| format!("{o:.0} Ω")),
            f64::from(s.mode.target_energy().0) / 10.0,
            sh.peak_a(),
            sh.capacitor_v,
            sh.residual_v()
        );
    }
    for t in &r.timings {
        let _ = writeln!(
            out,
            "    #{} timing: analysis {}, charge {}, countdown {}, permit wait {}, hands-off before shock {}",
            t.number,
            secs(t.analysis_ms),
            secs(t.charge_ms),
            secs(t.countdown_ms),
            secs(t.permit_wait_ms),
            t.hands_off_ms.map_or_else(|| "n/a".to_string(), secs)
        );
    }
    let _ = writeln!(
        out,
        "  Final state:         {:?}{}",
        r.final_state,
        r.final_fault
            .map_or_else(String::new, |f| format!(" (fault {f:?})"))
    );
    if let Some(f) = r.supervisor_fault {
        let _ = writeln!(out, "  Supervisor fault:    {f:?}");
    }
    let _ = writeln!(
        out,
        "  Last thing said:     {}",
        r.last_prompt()
            .map_or_else(|| "(nothing)".to_string(), |p| format!("\"{}\"", p.text()))
    );
    let _ = writeln!(
        out,
        "  Capacitor:           highest {:.0} V, at end {:.0} V (true)",
        r.max_cap_v, r.final_cap_v
    );
    if r.injected.is_empty() {
        let _ = writeln!(out, "  Faults injected:     none");
    }
    for (t, f) in &r.injected {
        let _ = writeln!(out, "  Fault injected:      {} at {}", f.describe(), clock(*t));
    }
    if !r.supervisor_refusals.is_empty() {
        let reasons: Vec<&str> = r
            .supervisor_refusals
            .iter()
            .map(|x| crate::runner::refusal_text(*x))
            .collect();
        let _ = writeln!(out, "  Supervisor refused:  {}", reasons.join("; "));
    }
    if r.messages_dropped > 0 {
        let _ = writeln!(out, "  Messages lost:       {}", r.messages_dropped);
    }
    if r.violations.is_empty() {
        let _ = writeln!(out, "  Safety invariants:   all held");
    } else {
        let _ = writeln!(out, "  SAFETY INVARIANTS BROKEN: {}", r.violations.len());
        for v in &r.violations {
            let _ = writeln!(out, "    at {}: {} ({})", clock(v.t_ms), v.rule.describe(), v.detail);
        }
    }
    out
}

/// One row of the timing table.
#[must_use]
pub fn timing_row(name: &str, pads_to_shock: Option<u32>, t: &ShockTiming) -> String {
    format!(
        "| {name} | {} | {} | {} | {} | {} | {} | {} |",
        t.number,
        pads_to_shock.map_or_else(|| "-".to_string(), secs),
        secs(t.analysis_ms),
        secs(t.charge_ms),
        secs(t.countdown_ms),
        secs(t.permit_wait_ms),
        t.hands_off_ms.map_or_else(|| "-".to_string(), secs)
    )
}

/// A timing table (markdown) for a set of runs.
#[must_use]
pub fn timing_table(runs: &[RunResult]) -> String {
    let mut out = String::from(
        "| Scenario | Shock | Pads on -> shock | Analysis | Charge | Countdown | Permit wait | Hands-off before shock |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for r in runs {
        for t in &r.timings {
            let pads = if t.number == 1 {
                r.pads_on_to_first_shock_ms()
            } else {
                None
            };
            let _ = writeln!(out, "{}", timing_row(&r.scenario.name, pads, t));
        }
    }
    out
}
