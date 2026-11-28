// SPDX-License-Identifier: GPL-3.0-or-later
//! Recovering the ECG from a Cardially PDF report.
//!
//! The PDFs in the Cardially dataset are event reports printed by Philips
//! "ERECG 3.0.0.0" review software to PDF (not scans: they hold no images).
//! Each page, turned sideways (`/Rotate 90`), has three strips of 10
//! seconds. The ECG is drawn as a blue line with one vertex per sample, the
//! paper grid as grey lines, each strip inside a black box, the clock time
//! at each end of each strip, and the device's events ("Shock 1 delivered",
//! "Shock advised", ...) as labels whose position along the strip is the
//! time of the event. Every page states its scale: "2 div/mV, 5 div/sec".
//!
//! This module turns that drawing back into numbers:
//!
//! * **Time.** A strip's length on the page and the clock times printed at
//!   its two ends give points per second. The vertex spacing then gives the
//!   sampling rate (200 per second in every file seen). Each vertex is put
//!   on the nearest sample instant; vertices more than 30 % of a sample
//!   away from one are counted, and so are instants with no vertex.
//! * **Amplitude.** One "div" in time is (points per second) / (div per
//!   second); the grid is square, so the same length is one div in
//!   amplitude, and (div per mV) of them is one millivolt. On the page that
//!   is 14.17 points (5 mm) per div, 10 mm per mV, the usual ECG paper
//!   scale. Up on the turned page is towards smaller x. This is the scale
//!   the review software **states**; whether it draws the device's true
//!   amplitude is not known (see the crate README).
//! * **Events** are the labels inside each strip's box; a label's position
//!   along the strip is the time of its event (checked against the marker
//!   line drawn beside it).

use super::pdf::{interpret, read_pages, Drawing, PdfError, Text};

/// One device event read from the report.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// Seconds from the start of the first strip.
    pub t_s: f64,
    /// The label text, fragments joined.
    pub text: String,
}

/// Everything recovered from one PDF report.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfEcg {
    /// The ECG, microvolts, at [`PdfEcg::fs_hz`], all strips joined.
    pub samples_uv: Vec<f64>,
    /// For each sample: the drawn trace sits on an edge of its strip's box.
    /// The review software **clips the trace to the box** (±2 mV about the
    /// strip's centre line at the printed scale), so these samples are
    /// clipped by the report, not by the patient's heart or the device.
    pub at_edge: Vec<bool>,
    /// Half the box height, µV at the printed scale: where the clipping is.
    pub clip_uv: f64,
    /// Sampling rate found from the vertex spacing, per second.
    pub fs_hz: u32,
    /// Number of strips.
    pub strips: usize,
    /// Pages.
    pub pages: usize,
    /// Seconds per strip (from the printed clock times).
    pub strip_s: u32,
    /// The scale printed on the pages: divisions per millivolt.
    pub div_per_mv: f64,
    /// The scale printed on the pages: divisions per second.
    pub div_per_s: f64,
    /// Length of one division on the page, points (14.17 = 5 mm expected).
    pub div_pt: f64,
    /// Sample instants with no vertex (filled by straight-line interpolation).
    pub missing: usize,
    /// Vertices more than 30 % of a sample from a sample instant.
    pub off_grid: usize,
    /// Vertices that landed on an instant already filled with a different value.
    pub conflicts: usize,
    /// Strips whose start clock time is not the previous strip's end time.
    pub clock_breaks: usize,
    /// Device events, in time order.
    pub events: Vec<Event>,
}

impl PdfEcg {
    /// Time of "Shock `n` delivered", seconds from the start, if printed.
    #[must_use]
    pub fn shock_delivered_s(&self, n: u32) -> Option<f64> {
        let want = format!("Shock {n} delivered");
        self.events
            .iter()
            .find(|e| e.text.trim() == want)
            .map(|e| e.t_s)
    }

    /// Duration, seconds.
    #[must_use]
    pub fn duration_s(&self) -> f64 {
        self.samples_uv.len() as f64 / f64::from(self.fs_hz.max(1))
    }
}

/// A strip box on a page: x from `x0` to `x1` (amplitude), y from `y0` to
/// `y1` (time), page coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
struct BoxXY {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
}

/// A vertex this close to a box edge, in points, is "on the edge" (clipped).
/// Coordinates are written to 0.03 pt; the clipped vertices sit exactly on
/// the low edge or 0.03 pt beyond the high one.
const EDGE_PT: f64 = 0.05;

fn strip_boxes(d: &Drawing) -> Vec<BoxXY> {
    let mut v: Vec<BoxXY> =
        d.strokes
            .iter()
            .filter(|s| !s.colour.is_blue() && s.points.len() == 5)
            .filter_map(|s| {
                let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
                for (x, y) in &s.points {
                    x0 = x0.min(*x);
                    x1 = x1.max(*x);
                    y0 = y0.min(*y);
                    y1 = y1.max(*y);
                }
                // Axis-aligned: every vertex on an edge.
                let on_edge = s.points.iter().all(|(x, y)| {
                    ((x - x0).abs() < 0.05 || (x - x1).abs() < 0.05)
                        && ((y - y0).abs() < 0.05 || (y - y1).abs() < 0.05)
                });
                (on_edge && y1 - y0 > 500.0 && (50.0..300.0).contains(&(x1 - x0)))
                    .then_some(BoxXY { x0, x1, y0, y1 })
            })
            .collect();
    v.sort_by(|a, b| a.x0.total_cmp(&b.x0));
    v.dedup_by(|a, b| (a.x0 - b.x0).abs() < 1.0);
    v
}

/// Seconds since midnight for "hh.mm.ss" or "hh:mm:ss".
#[must_use]
pub fn clock_seconds(s: &str) -> Option<u32> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(['.', ':']).collect();
    let [h, m, sec] = parts.as_slice() else {
        return None;
    };
    if [h, m, sec]
        .iter()
        .any(|p| p.len() != 2 || !p.bytes().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    let (h, m, sec): (u32, u32, u32) = (h.parse().ok()?, m.parse().ok()?, sec.parse().ok()?);
    (h < 24 && m < 60 && sec < 60).then_some(h * 3600 + m * 60 + sec)
}

/// "2 div/mV, 5 div/sec" -> (2, 5).
#[must_use]
pub fn parse_scale(s: &str) -> Option<(f64, f64)> {
    let s = s.trim();
    let (a, b) = s.split_once(',')?;
    let mv = a
        .trim()
        .strip_suffix("div/mV")?
        .trim()
        .parse::<f64>()
        .ok()?;
    let sec = b
        .trim()
        .strip_suffix("div/sec")?
        .trim()
        .parse::<f64>()
        .ok()?;
    (mv.is_finite() && sec.is_finite() && mv > 0.0 && sec > 0.0).then_some((mv, sec))
}

/// Join label fragments that belong together ("A" + "rmed"): sorted along
/// the strip, a fragment continues the previous one when it is on the same
/// line (same x) and starts within the previous one's estimated length.
fn join_fragments(mut t: Vec<Text>) -> Vec<Text> {
    t.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    let mut out: Vec<Text> = Vec::new();
    for f in t {
        if let Some(prev) = out.last_mut() {
            let est_len = prev.text.chars().count() as f64 * prev.size * 0.65 + 2.0;
            if (prev.x - f.x).abs() < 0.5 && f.y >= prev.y && f.y - prev.y <= est_len {
                prev.text.push_str(&f.text);
                continue;
            }
        }
        out.push(f);
    }
    out
}

struct PageStrips {
    clock_start: Vec<Option<u32>>,
    clock_end: Vec<Option<u32>>,
    samples: Vec<Vec<Option<f64>>>,
    edges: Vec<Vec<bool>>,
    events: Vec<Vec<(f64, String)>>,
    off_grid: usize,
    conflicts: usize,
}

/// Read one report.
///
/// # Errors
/// [`PdfError`] if the file cannot be parsed or does not look like the
/// Philips report this reader was written for (no strips, no scale, an
/// unexpected rotation, inconsistent sampling or clock times).
pub fn read_report(file: &[u8]) -> Result<PdfEcg, PdfError> {
    let pages = read_pages(file)?;
    let bad = |w: &str| PdfError::Unsupported(w.to_string());
    let mut scale: Option<(f64, f64)> = None;
    let mut fs: Option<u32> = None;
    let mut strip_s: Option<u32> = None;
    let mut parsed: Vec<PageStrips> = Vec::new();
    let mut div_pt = None;
    let mut clip_uv: Option<f64> = None;
    for page in &pages {
        if page.rotate != 90 {
            return Err(bad("page rotation is not 90 degrees"));
        }
        let d = interpret(&page.content)?;
        let boxes = strip_boxes(&d);
        if boxes.is_empty() {
            // A page with no ECG strips (seen: a trailing page with only the
            // header and footer). Nothing to read on it.
            continue;
        }
        let page_scale = d
            .texts
            .iter()
            .find_map(|t| parse_scale(&t.text))
            .ok_or_else(|| bad("no 'div/mV, div/sec' scale"))?;
        if scale.is_some_and(|s| s != page_scale) {
            return Err(bad("pages state different scales"));
        }
        scale = Some(page_scale);
        let (div_per_mv, div_per_s) = page_scale;
        let joined = join_fragments(d.texts.clone());
        let mut ps = PageStrips {
            clock_start: Vec::new(),
            clock_end: Vec::new(),
            samples: Vec::new(),
            edges: Vec::new(),
            events: Vec::new(),
            off_grid: 0,
            conflicts: 0,
        };
        for b in &boxes {
            // Clock labels sit just past the box's high-x edge, one at each end.
            let clocks: Vec<(f64, u32)> = d
                .texts
                .iter()
                .filter(|t| t.x > b.x1 && t.x < b.x1 + 20.0 && t.y > b.y0 - 5.0 && t.y < b.y1 + 5.0)
                .filter_map(|t| clock_seconds(&t.text).map(|c| (t.y, c)))
                .collect();
            let first = clocks
                .iter()
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|c| c.1);
            let lastc = clocks
                .iter()
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .map(|c| c.1);
            let (Some(c0), Some(c1)) = (first, lastc) else {
                return Err(bad("strip without clock times at both ends"));
            };
            let secs = (c1 + 86_400 - c0) % 86_400;
            if !(1..=60).contains(&secs) {
                return Err(bad("strip clock times implausible"));
            }
            if strip_s.is_some_and(|s| s != secs) {
                return Err(bad("strips of different lengths"));
            }
            strip_s = Some(secs);
            let pts_per_s = (b.y1 - b.y0) / f64::from(secs);
            let this_div = pts_per_s / div_per_s;
            div_pt = Some(this_div);
            let pts_per_mv = this_div * div_per_mv;

            // The trace: blue subpaths starting inside this box.
            let trace: Vec<&Vec<(f64, f64)>> = d
                .strokes
                .iter()
                .filter(|s| s.colour.is_blue())
                .filter(|s| {
                    s.points.first().is_some_and(|(x, y)| {
                        *x >= b.x0 - 1.0 && *x <= b.x1 + 1.0 && *y >= b.y0 - 1.0 && *y <= b.y1 + 1.0
                    })
                })
                .map(|s| &s.points)
                .collect();
            // Sampling rate from the median vertex spacing along time.
            let mut steps: Vec<f64> = trace
                .iter()
                .flat_map(|p| {
                    p.windows(2)
                        .filter_map(|w| w.get(1).zip(w.first()).map(|(b2, a)| b2.1 - a.1))
                })
                .filter(|d| *d > 0.0)
                .collect();
            steps.sort_by(f64::total_cmp);
            let med = *steps
                .get(steps.len() / 2)
                .ok_or_else(|| bad("strip without trace"))?;
            let this_fs = (pts_per_s / med / 10.0).round() as u32 * 10; // to the nearest 10 per second
            if fs.is_some_and(|f| f != this_fs) || !(50..=2000).contains(&this_fs) {
                return Err(bad("inconsistent or implausible sampling rate"));
            }
            fs = Some(this_fs);
            let n = (secs * this_fs) as usize;
            let dt_pt = pts_per_s / f64::from(this_fs);
            let xc = 0.5 * (b.x0 + b.x1);
            let mut s: Vec<Option<f64>> = vec![None; n];
            let mut edge: Vec<bool> = vec![false; n];
            clip_uv = Some(0.5 * (b.x1 - b.x0) / pts_per_mv * 1000.0);
            for p in &trace {
                for (x, y) in p.iter() {
                    let pos = (y - b.y0) / dt_pt;
                    let i = pos.round();
                    if i < 0.0 || i >= n as f64 {
                        continue; // past the strip's end (drawn squashed there)
                    }
                    if (pos - i).abs() > 0.3 {
                        ps.off_grid += 1;
                    }
                    // Up on the turned page is towards smaller x.
                    let uv = -(x - xc) / pts_per_mv * 1000.0;
                    if (x - b.x0).abs() < EDGE_PT || (x - b.x1).abs() < EDGE_PT {
                        if let Some(e) = edge.get_mut(i as usize) {
                            *e = true;
                        }
                    }
                    if let Some(slot) = s.get_mut(i as usize) {
                        match slot {
                            Some(v) if (*v - uv).abs() > 1e-6 => ps.conflicts += 1,
                            Some(_) => {}
                            None => *slot = Some(uv),
                        }
                    }
                }
            }
            let ev: Vec<(f64, String)> = joined
                .iter()
                .filter(|t| t.x >= b.x0 && t.x <= b.x1 && t.y >= b.y0 - 1.0 && t.y <= b.y1 + 1.0)
                .filter(|t| clock_seconds(&t.text).is_none() && !t.text.trim().is_empty())
                .map(|t| ((t.y - b.y0) / pts_per_s, t.text.trim().to_string()))
                .collect();
            ps.clock_start.push(Some(c0));
            ps.clock_end.push(Some(c1));
            ps.samples.push(s);
            ps.edges.push(edge);
            ps.events.push(ev);
        }
        parsed.push(ps);
    }
    let (Some((div_per_mv, div_per_s)), Some(fs), Some(strip_s), Some(div_pt)) =
        (scale, fs, strip_s, div_pt)
    else {
        return Err(bad("nothing recovered"));
    };
    // Order pages by their first clock time, relative to the first page in the file.
    let base = parsed
        .first()
        .and_then(|p| p.clock_start.first().copied().flatten())
        .unwrap_or(0);
    parsed.sort_by_key(|p| {
        p.clock_start
            .first()
            .copied()
            .flatten()
            .map_or(u32::MAX, |c| (c + 86_400 - base) % 86_400)
    });
    let mut samples: Vec<Option<f64>> = Vec::new();
    let mut at_edge: Vec<bool> = Vec::new();
    let mut events = Vec::new();
    let mut strips = 0;
    let mut clock_breaks = 0;
    let mut prev_end: Option<u32> = None;
    let (mut off_grid, mut conflicts) = (0, 0);
    for p in parsed {
        off_grid += p.off_grid;
        conflicts += p.conflicts;
        for (k, s) in p.samples.into_iter().enumerate() {
            let start = p.clock_start.get(k).copied().flatten();
            if prev_end.is_some() && start != prev_end {
                clock_breaks += 1;
            }
            prev_end = p.clock_end.get(k).copied().flatten();
            let t0 = (strips * strip_s as usize) as f64;
            for (t, text) in p.events.get(k).cloned().unwrap_or_default() {
                events.push(Event { t_s: t0 + t, text });
            }
            samples.extend(s);
            at_edge.extend(p.edges.get(k).cloned().unwrap_or_default());
            strips += 1;
        }
    }
    events.sort_by(|a, b| a.t_s.total_cmp(&b.t_s));
    let missing = samples.iter().filter(|v| v.is_none()).count();
    if missing * 10 > samples.len() {
        return Err(bad("more than 10 % of samples missing"));
    }
    let samples_uv = fill_gaps(&samples);
    at_edge.resize(samples_uv.len(), false);
    Ok(PdfEcg {
        samples_uv,
        at_edge,
        clip_uv: clip_uv.unwrap_or(f64::NAN),
        fs_hz: fs,
        strips,
        pages: pages.len(),
        strip_s,
        div_per_mv,
        div_per_s,
        div_pt,
        missing,
        off_grid,
        conflicts,
        clock_breaks,
        events,
    })
}

/// Fill missing samples by straight lines between their neighbours (the
/// nearest known value at the ends).
fn fill_gaps(s: &[Option<f64>]) -> Vec<f64> {
    let known: Vec<(usize, f64)> = s
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
        .collect();
    let mut out = Vec::with_capacity(s.len());
    let mut k = 0;
    for i in 0..s.len() {
        while known.get(k + 1).is_some_and(|(j, _)| *j <= i) {
            k += 1;
        }
        let v = match (known.get(k), known.get(k + 1)) {
            (Some(&(a, va)), _) if a == i => va,
            (Some(&(a, va)), Some(&(b, vb))) if a < i => {
                va + (vb - va) * (i - a) as f64 / (b - a) as f64
            }
            (Some(&(_, va)), _) => va,
            (None, _) => 0.0,
        };
        out.push(v);
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn clock_and_scale_parsing() {
        assert_eq!(clock_seconds("12.29.22"), Some(12 * 3600 + 29 * 60 + 22));
        assert_eq!(clock_seconds("07:48:39"), Some(7 * 3600 + 48 * 60 + 39));
        assert_eq!(clock_seconds("24.00.00"), None);
        assert_eq!(clock_seconds("1.2.3"), None);
        assert_eq!(clock_seconds("Shock 1 delivered"), None);
        assert_eq!(parse_scale("2 div/mV, 5 div/sec"), Some((2.0, 5.0)));
        assert_eq!(parse_scale("0 div/mV, 5 div/sec"), None);
        assert_eq!(parse_scale("2 div/mV"), None);
    }

    #[test]
    fn fragments_are_joined_only_along_one_line() {
        let t = |x: f64, y: f64, s: &str| Text {
            x,
            y,
            size: 6.8,
            text: s.into(),
        };
        let j = join_fragments(vec![
            t(56.0, 347.6, "rmed"),
            t(56.0, 343.1, "A"),
            t(80.0, 343.1, "Other"),
        ]);
        let texts: Vec<&str> = j.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Armed", "Other"]);
    }

    #[test]
    fn gaps_are_filled_by_straight_lines() {
        assert_eq!(
            fill_gaps(&[None, Some(1.0), None, None, Some(4.0), None]),
            vec![1.0, 1.0, 2.0, 3.0, 4.0, 4.0]
        );
        assert!(fill_gaps(&[None, None]).iter().all(|v| *v == 0.0));
    }

    /// A one-page, one-strip report drawn the way the Philips software draws it,
    /// with a known 5 Hz sine of 1 mV peak.
    fn hand_built_report(polarity: f64) -> Vec<u8> {
        use std::fmt::Write as _;
        let mut c = String::new();
        // Box: x 42.54..155.97 (40 mm), y 42.591..751.281 (10 s at 25 mm/s).
        c.push_str("0 G .565 w q 1 0 0 1 42.54 42.591 cm 0 0 m 0 708.69 l 113.43 708.69 l 113.43 0 l 0 0 l S Q\n");
        c.push_str("/Cs6 CS 0 0 1 SCN .565 w\n");
        let xc = 0.5 * (42.54 + 155.97);
        let pts_per_mv = 708.69 / 10.0 / 5.0 * 2.0;
        // One subpath per second, 201 vertices each, as in the real files.
        for sec in 0..10 {
            let _ = write!(c, "q 1 0 0 1 0 0 cm ");
            for k in 0..=200 {
                let i = sec * 200 + k;
                let t = f64::from(i) / 200.0;
                let mv = (2.0 * std::f64::consts::PI * 5.0 * t).sin();
                let x = xc - polarity * mv * pts_per_mv;
                let y = 42.591 + t * 70.869;
                let _ = write!(c, "{x:.2} {y:.2} {} ", if k == 0 { "m" } else { "l" });
            }
            c.push_str("S Q\n");
        }
        c.push_str("BT 0 6.8 -6.8 0 162.06 42.59 Tm (12.29.22)Tj 100.2866 0 TD (12.29.32)Tj ET\n");
        c.push_str("BT 0 6.8 -6.8 0 100.0 113.46 Tm (Shock 1 d)Tj ET BT 0 6.8 -6.8 0 100.0 145.0 Tm (elivered)Tj ET\n");
        c.push_str("BT 0 6.8 -6.8 0 40.9 42.6 Tm (2 div/mV, 5 div/sec)Tj ET\n");
        let content = c.into_bytes();
        let mut z = vec![0x78, 0x01];
        for (n, chunk) in content.chunks(60_000).enumerate() {
            let last = n == content.len().div_ceil(60_000) - 1;
            z.push(u8::from(last));
            let len = chunk.len() as u16;
            z.extend_from_slice(&len.to_le_bytes());
            z.extend_from_slice(&(!len).to_le_bytes());
            z.extend_from_slice(chunk);
        }
        let (mut a, mut b) = (1_u32, 0_u32);
        for ch in &content {
            a = (a + u32::from(*ch)) % 65_521;
            b = (b + a) % 65_521;
        }
        z.extend_from_slice(&((b << 16) | a).to_be_bytes());
        let mut f =
            b"%PDF-1.5\n2 0 obj\n<</Contents 3 0 R/Rotate 90/Type/Page>>\nendobj\n".to_vec();
        f.extend_from_slice(
            format!(
                "3 0 obj\n<</Filter/FlateDecode/Length {}>>stream\n",
                z.len()
            )
            .as_bytes(),
        );
        f.extend_from_slice(&z);
        f.extend_from_slice(b"\nendstream\nendobj\n%%EOF\n");
        f
    }

    #[test]
    fn recovers_a_hand_built_trace_with_scale_time_and_events() {
        let r = read_report(&hand_built_report(1.0)).unwrap();
        assert_eq!(r.fs_hz, 200);
        assert_eq!(r.strips, 1);
        assert_eq!(r.strip_s, 10);
        assert_eq!(r.samples_uv.len(), 2000);
        assert_eq!(r.missing, 0);
        assert_eq!(r.off_grid, 0);
        assert!((r.clip_uv - 2000.6).abs() < 1.0, "{}", r.clip_uv);
        assert_eq!(r.at_edge.len(), 2000);
        assert!(!r.at_edge.iter().any(|e| *e));
        assert!((r.div_pt - 14.1738).abs() < 1e-3);
        // 1 mV sine back to within the 0.01-point rounding (about 0.35 µV) plus rounding of y.
        for (i, v) in r.samples_uv.iter().enumerate() {
            let want = 1000.0 * (2.0 * std::f64::consts::PI * 5.0 * i as f64 / 200.0).sin();
            assert!((v - want).abs() < 2.0, "sample {i}: {v} vs {want}");
        }
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].text, "Shock 1 delivered");
        assert!((r.shock_delivered_s(1).unwrap() - 1.0).abs() < 0.01);
        // Drawn the other way up, the recovered signal flips sign.
        let flipped = read_report(&hand_built_report(-1.0)).unwrap();
        assert!((flipped.samples_uv[10] + r.samples_uv[10]).abs() < 2.0);
    }

    #[test]
    fn rejects_reports_without_scale_or_strips() {
        let good = hand_built_report(1.0);
        assert!(read_report(b"%PDF-1.5\n").is_err());
        // Any truncation: an error, never a panic.
        for n in (0..good.len()).step_by(997) {
            let _ = read_report(&good[..n]);
        }
    }
}
