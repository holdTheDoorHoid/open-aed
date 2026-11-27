// SPDX-License-Identifier: GPL-3.0-or-later
//! The dataset's own digitised waveforms (`dataset/txt/<class>/<id>.txt`).
//!
//! Each file is plain text, two columns separated by spaces, Windows line
//! endings: time in seconds (0 to about 9) and amplitude (millivolts by
//! all appearances; the dataset does not say). The companion article says
//! they were digitised with the commercial program FindGraph, so the points
//! are **not equally spaced**: the time steps vary, some instants repeat
//! (vertical strokes of the trace), and there are gaps of up to about 80 ms.
//! The amplitude moves in steps of about 0.019 mV.

/// Most lines accepted in one file.
pub const MAX_LINES: usize = 100_000;
/// Largest file accepted, bytes.
pub const MAX_BYTES: usize = 4 * 1024 * 1024;

/// Why a file was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxtError {
    /// Too large.
    TooLarge,
    /// A line that is not two finite numbers (line number from 1).
    BadLine(usize),
    /// Time goes backwards (line number from 1).
    TimeBackwards(usize),
    /// Fewer than two points, or times outside 0 to 60 s.
    Implausible,
}

impl core::fmt::Display for TxtError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TxtError::TooLarge => write!(f, "file too large"),
            TxtError::BadLine(n) => write!(f, "line {n} is not two numbers"),
            TxtError::TimeBackwards(n) => write!(f, "time goes backwards at line {n}"),
            TxtError::Implausible => write!(f, "too few points or times out of range"),
        }
    }
}

impl std::error::Error for TxtError {}

/// Parse a digitised waveform: (time s, value) pairs in file order.
///
/// # Errors
/// [`TxtError`] for anything that is not two finite numbers per line, time
/// going backwards, or an implausible file.
pub fn parse_txt(bytes: &[u8]) -> Result<Vec<(f64, f64)>, TxtError> {
    if bytes.len() > MAX_BYTES {
        return Err(TxtError::TooLarge);
    }
    let text = String::from_utf8_lossy(bytes);
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if out.len() >= MAX_LINES {
            return Err(TxtError::TooLarge);
        }
        let mut it = line.split_whitespace();
        let (Some(a), Some(b), None) = (it.next(), it.next(), it.next()) else {
            return Err(TxtError::BadLine(n + 1));
        };
        let (Ok(t), Ok(v)) = (a.parse::<f64>(), b.parse::<f64>()) else {
            return Err(TxtError::BadLine(n + 1));
        };
        if !(t.is_finite() && v.is_finite()) {
            return Err(TxtError::BadLine(n + 1));
        }
        if out.last().is_some_and(|(p, _): &(f64, f64)| t < *p) {
            return Err(TxtError::TimeBackwards(n + 1));
        }
        out.push((t, v));
    }
    let ok = out.len() >= 2 && out.iter().all(|(t, _)| (0.0..=60.0).contains(t));
    if ok {
        Ok(out)
    } else {
        Err(TxtError::Implausible)
    }
}

/// Points with the same time merged into one (the mean of their values).
#[must_use]
pub fn merge_repeated_times(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64, u32)> = Vec::new();
    for &(t, v) in points {
        match out.last_mut() {
            Some((pt, sum, n)) if *pt == t => {
                *sum += v;
                *n += 1;
            }
            _ => out.push((t, v, 1)),
        }
    }
    out.into_iter().map(|(t, s, n)| (t, s / f64::from(n))).collect()
}

/// Put irregular points on a regular grid of `fs` per second, from time 0,
/// by straight-line interpolation between neighbours (the first or last
/// value outside the points' range). Straight lines between points up to
/// 80 ms apart act as a crude low-pass filter: detail between the
/// digitiser's points is lost and cannot be recovered.
#[must_use]
pub fn to_uniform(points: &[(f64, f64)], fs: u32, duration_s: f64) -> Vec<f64> {
    let p = merge_repeated_times(points);
    let n = (duration_s * f64::from(fs)).round().max(0.0) as usize;
    let mut out = Vec::with_capacity(n);
    let mut k = 0;
    for i in 0..n {
        let t = i as f64 / f64::from(fs);
        while p.get(k + 1).is_some_and(|(tk, _)| *tk <= t) {
            k += 1;
        }
        let v = match (p.get(k), p.get(k + 1)) {
            (Some(&(t0, v0)), Some(&(t1, v1))) if t >= t0 && t1 > t0 => v0 + (v1 - v0) * (t - t0) / (t1 - t0),
            (Some(&(_, v0)), _) => v0,
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
    fn parses_the_dataset_layout() {
        let f = b"     0.01186      -0.2331\r\n     0.02698      -0.1572\r\n     0.02698   4.441e-016\r\n\r\n";
        let p = parse_txt(f).unwrap();
        assert_eq!(p.len(), 3);
        assert_eq!(p[0], (0.01186, -0.2331));
        assert!(p[2].1.abs() < 1e-15);
    }

    #[test]
    fn refuses_bad_files() {
        assert_eq!(parse_txt(b"0.1 0.2 0.3\n0.2 0.1\n"), Err(TxtError::BadLine(1)));
        assert_eq!(parse_txt(b"0.1 x\n"), Err(TxtError::BadLine(1)));
        assert_eq!(parse_txt(b"0.1 NaN\n0.2 1\n"), Err(TxtError::BadLine(1)));
        assert_eq!(parse_txt(b"0.2 1\n0.1 1\n"), Err(TxtError::TimeBackwards(2)));
        assert_eq!(parse_txt(b"0.2 1\n"), Err(TxtError::Implausible));
        assert_eq!(parse_txt(b"0.2 1\n99 1\n"), Err(TxtError::Implausible));
        assert_eq!(parse_txt(b""), Err(TxtError::Implausible));
    }

    #[test]
    fn repeated_times_are_averaged_and_grid_is_interpolated() {
        let p = [(0.0, 0.0), (0.01, 1.0), (0.01, 3.0), (0.03, 0.0)];
        assert_eq!(merge_repeated_times(&p), vec![(0.0, 0.0), (0.01, 2.0), (0.03, 0.0)]);
        let u = to_uniform(&p, 200, 0.04);
        // 0, 5, 10, ... 35 ms: up to 2 at 10 ms, down to 0 at 30 ms, then held.
        let want = [0.0, 1.0, 2.0, 1.5, 1.0, 0.5, 0.0, 0.0];
        assert_eq!(u.len(), 8);
        for (a, b) in u.iter().zip(want.iter()) {
            assert!((a - b).abs() < 1e-9, "{u:?}");
        }
    }
}
