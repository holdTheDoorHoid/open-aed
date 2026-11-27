// SPDX-License-Identifier: GPL-3.0-or-later
//! Confidence bounds and simple summaries.
//!
//! The AHA 1997 recommendations judge a detector by the **lower one-sided
//! 90 % confidence limit** of each sensitivity or specificity, not by the
//! raw percentage. That limit is computed here exactly (Clopper–Pearson,
//! from the beta distribution), with the Wilson score bound alongside as a
//! cross-check.
//!
//! Caution: both assume every segment is an independent trial. Segments
//! from the same patient are not independent, so these bounds are too
//! optimistic; the per-record tables exist to show that.

/// Natural log of the gamma function (Lanczos approximation, about 15 digits).
#[must_use]
pub fn ln_gamma(x: f64) -> f64 {
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        // Reflection formula.
        let pi = core::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let mut a = C[0];
    let t = x + G + 0.5;
    for (i, c) in C.iter().enumerate().skip(1) {
        a += c / (x + i as f64);
    }
    0.5 * (2.0 * core::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
}

/// Continued fraction for the incomplete beta function (modified Lentz).
fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    const TINY: f64 = 1e-300;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..500 {
        let m = f64::from(m);
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    h
}

/// Regularised incomplete beta function I_x(a, b), for a, b > 0 and 0 ≤ x ≤ 1.
#[must_use]
pub fn incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let ln_front = ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln();
    let front = ln_front.exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_cf(a, b, x) / a
    } else {
        1.0 - front * beta_cf(b, a, 1.0 - x) / b
    }
}

/// Exact (Clopper–Pearson) one-sided lower confidence limit for a
/// proportion of `successes` out of `n`, at confidence `level` (0.90 for the
/// AHA criterion). Returns `None` when `n` is zero.
#[must_use]
pub fn clopper_pearson_lower(successes: u64, n: u64, level: f64) -> Option<f64> {
    if n == 0 || successes > n {
        return None;
    }
    if successes == 0 {
        return Some(0.0);
    }
    let alpha = 1.0 - level;
    let a = successes as f64;
    let b = (n - successes) as f64 + 1.0;
    // Find p with I_p(a, b) = alpha by bisection (I is increasing in p).
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        if incomplete_beta(a, b, mid) < alpha {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Wilson score one-sided lower bound at the given level (uses the normal
/// quantile 1.2816 for 0.90, 1.6449 for 0.95; other levels fall back to 1.2816).
#[must_use]
pub fn wilson_lower(successes: u64, n: u64, level: f64) -> Option<f64> {
    if n == 0 || successes > n {
        return None;
    }
    let z = if (level - 0.95).abs() < 1e-9 { 1.644_853_6 } else { 1.281_551_6 };
    let nf = n as f64;
    let p = successes as f64 / nf;
    let z2 = z * z;
    let centre = p + z2 / (2.0 * nf);
    let spread = z * (p * (1.0 - p) / nf + z2 / (4.0 * nf * nf)).sqrt();
    Some(((centre - spread) / (1.0 + z2 / nf)).max(0.0))
}

/// Percentile (0–100) of a sample by linear interpolation; `None` if empty.
/// Not-a-number values are ignored.
#[must_use]
pub fn percentile(values: &[f64], pct: f64) -> Option<f64> {
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let pos = (pct / 100.0).clamp(0.0, 1.0) * (v.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    let a = *v.get(lo)?;
    let b = *v.get(hi)?;
    Some(a + (b - a) * (pos - lo as f64))
}

/// Median; `None` if empty.
#[must_use]
pub fn median(values: &[f64]) -> Option<f64> {
    percentile(values, 50.0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn ln_gamma_known_values() {
        assert!((ln_gamma(1.0)).abs() < 1e-12);
        assert!((ln_gamma(5.0) - 24.0_f64.ln()).abs() < 1e-12);
        assert!((ln_gamma(0.5) - core::f64::consts::PI.sqrt().ln()).abs() < 1e-10);
    }

    #[test]
    fn incomplete_beta_known_values() {
        // I_x(1, 1) = x; I_x(2, 1) = x^2; I_x(1, b) = 1 - (1-x)^b.
        assert!((incomplete_beta(1.0, 1.0, 0.3) - 0.3).abs() < 1e-12);
        assert!((incomplete_beta(2.0, 1.0, 0.3) - 0.09).abs() < 1e-12);
        assert!((incomplete_beta(1.0, 4.0, 0.2) - (1.0 - 0.8_f64.powi(4))).abs() < 1e-12);
    }

    #[test]
    fn clopper_pearson_matches_closed_forms() {
        // All successes: lower bound is alpha^(1/n).
        for n in [1_u64, 10, 100, 300] {
            let lo = clopper_pearson_lower(n, n, 0.90).unwrap();
            assert!((lo - 0.1_f64.powf(1.0 / n as f64)).abs() < 1e-9, "n={n}");
        }
        // One success out of n: lower bound solves 1-(1-p)^n = alpha.
        let lo = clopper_pearson_lower(1, 20, 0.90).unwrap();
        assert!((lo - (1.0 - 0.9_f64.powf(1.0 / 20.0))).abs() < 1e-9);
        assert_eq!(clopper_pearson_lower(0, 5, 0.9), Some(0.0));
        assert_eq!(clopper_pearson_lower(1, 0, 0.9), None);
        // The AHA table's lower limits for the minimum sample sizes are close
        // to what an exact bound gives at the goal rate (sanity check only).
        let lo = clopper_pearson_lower(180, 200, 0.90).unwrap();
        assert!(lo > 0.86 && lo < 0.88, "{lo}");
    }

    #[test]
    fn wilson_is_close_to_exact_for_moderate_counts() {
        let cp = clopper_pearson_lower(450, 500, 0.90).unwrap();
        let w = wilson_lower(450, 500, 0.90).unwrap();
        assert!((cp - w).abs() < 0.01);
    }

    #[test]
    fn percentiles() {
        let v = [3.0, 1.0, 2.0, f64::NAN, 4.0];
        assert_eq!(median(&v), Some(2.5));
        assert_eq!(percentile(&v, 0.0), Some(1.0));
        assert_eq!(percentile(&v, 100.0), Some(4.0));
        assert_eq!(percentile(&[], 50.0), None);
    }
}
