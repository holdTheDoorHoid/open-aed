// SPDX-License-Identifier: GPL-3.0-or-later
//! Reader for the Paris Fire Brigade impedance and ventilation dataset
//! (Mendeley Data, DOI 10.17632/34gffrm87w.1, CC BY 4.0).
//!
//! The files are untrusted input. They are read as plain text, every number
//! is checked, sizes are capped, and anything odd is reported as an error
//! for that case rather than trusted. Nothing in them is executed.
//!
//! Layout (from the dataset's own `Read_me/Dictionnary_of_datas.xlsx` and
//! the files themselves):
//!
//! * `data/intervention_NNN/Impedance_signal.csv`: `Time (s);TTI (Ohms)`,
//!   one row every 8 ms (125 per second). The impedance is the CHANGE
//!   around the baseline (it swings around zero), in ohms, in steps of
//!   about 12.2 mΩ, clipped at about ±25 Ω. Stretches of exactly 0 are
//!   periods without a signal (for example before the pads go on).
//! * `data/intervention_NNN/Ventilation_volumes.csv`:
//!   `Cycle number;Time (s);Vi (mL);Vt (mL)`, one row per breath, the time
//!   as `0 days HH:MM:SS` on the ventilation monitor's own clock. The
//!   dataset's paper (Menant et al., Data in Brief 2025;59:111345) says the
//!   two clocks had to be lined up afterwards and 12 of 101 cases could not
//!   be; the offsets are not in the dataset. These times are therefore NOT
//!   used for scoring.
//! * `data/Rhythm_analysis.csv`: `id;List of analysis (s)`, the times at
//!   which the defibrillator started each rhythm analysis, on the impedance
//!   clock.

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

/// Samples per second in the impedance files.
pub const RATE_HZ: f64 = 125.0;
/// The recordings' clipping level, ohms: values at or beyond this (either
/// sign) are treated as saturated. Measured on the development half: the
/// largest positive value is exactly 25.0, the most negative values are
/// −23.8 to −25.0.
pub const CLIP_OHMS: f64 = 23.8;
/// Largest file accepted, bytes (the largest real file is about 25 MB).
const MAX_FILE_BYTES: u64 = 200 * 1024 * 1024;
/// Longest line accepted, bytes.
const MAX_LINE: usize = 4096;
/// Longest recording accepted, samples (about 6 hours).
const MAX_SAMPLES: usize = 3_000_000;

/// One impedance sample from the recording.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sample {
    /// A reading, ohms of change around the baseline.
    Ohms(f64),
    /// At the clipping level.
    Clipped,
    /// No row for this moment (a gap in the time column).
    Missing,
}

/// One case (one cardiac arrest).
#[derive(Debug, Clone)]
pub struct Case {
    /// Folder name, for example `intervention_004`.
    pub name: String,
    /// Case number (4 for `intervention_004`).
    pub id: u32,
    /// Impedance, one entry per 8 ms from time 0.
    pub samples: Vec<Sample>,
    /// Times the defibrillator started a rhythm analysis, seconds.
    pub analyses_s: Vec<f64>,
    /// Breath times from the ventilation monitor (its own clock), seconds.
    pub breaths_s: Vec<f64>,
    /// Rows whose time did not fall on the 8 ms grid or went backwards.
    pub irregular_rows: usize,
}

impl Case {
    /// Length, seconds.
    #[must_use]
    pub fn seconds(&self) -> f64 {
        self.samples.len() as f64 / RATE_HZ
    }
}

/// A problem reading the data.
#[derive(Debug)]
pub enum DataError {
    /// A file could not be opened or read.
    Io(PathBuf, std::io::Error),
    /// A file is larger than the cap.
    TooLarge(PathBuf),
    /// A line could not be understood.
    Parse(PathBuf, usize, String),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Io(p, e) => write!(f, "{}: {e}", p.display()),
            DataError::TooLarge(p) => write!(f, "{}: file too large", p.display()),
            DataError::Parse(p, line, what) => write!(f, "{}:{line}: {what}", p.display()),
        }
    }
}

fn open(path: &Path) -> Result<BufReader<fs::File>, DataError> {
    let meta = fs::metadata(path).map_err(|e| DataError::Io(path.to_path_buf(), e))?;
    if !meta.is_file() || meta.len() > MAX_FILE_BYTES {
        return Err(DataError::TooLarge(path.to_path_buf()));
    }
    let f = fs::File::open(path).map_err(|e| DataError::Io(path.to_path_buf(), e))?;
    Ok(BufReader::new(f))
}

/// Read lines, capped in length, skipping the header.
fn lines(path: &Path) -> Result<Vec<(usize, String)>, DataError> {
    let mut reader = open(path)?;
    let mut out = Vec::new();
    let mut buf = Vec::new();
    let mut number = 0usize;
    loop {
        buf.clear();
        let n = reader
            .by_ref()
            .take(MAX_LINE as u64 + 1)
            .read_until(b'\n', &mut buf)
            .map_err(|e| DataError::Io(path.to_path_buf(), e))?;
        if n == 0 {
            break;
        }
        number += 1;
        if buf.len() > MAX_LINE {
            return Err(DataError::Parse(path.to_path_buf(), number, "line too long".into()));
        }
        let text = String::from_utf8_lossy(&buf).trim().to_string();
        if number == 1 || text.is_empty() {
            continue;
        }
        out.push((number, text));
        if out.len() > MAX_SAMPLES {
            return Err(DataError::TooLarge(path.to_path_buf()));
        }
    }
    Ok(out)
}

fn number(path: &Path, line: usize, s: &str) -> Result<f64, DataError> {
    let v: f64 = s
        .trim()
        .parse()
        .map_err(|_| DataError::Parse(path.to_path_buf(), line, format!("not a number: {s:?}")))?;
    if v.is_finite() {
        Ok(v)
    } else {
        Err(DataError::Parse(path.to_path_buf(), line, "not finite".into()))
    }
}

/// Read one impedance file.
///
/// # Errors
/// If the file is missing, too large or malformed.
pub fn read_impedance(path: &Path) -> Result<(Vec<Sample>, usize), DataError> {
    let rows = lines(path)?;
    let mut samples: Vec<Sample> = Vec::with_capacity(rows.len());
    let mut irregular = 0usize;
    for (line, text) in rows {
        let mut parts = text.split(';');
        let (Some(t), Some(z), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(DataError::Parse(path.to_path_buf(), line, "expected two columns".into()));
        };
        let t = number(path, line, t)?;
        let z = number(path, line, z)?;
        if !(0.0..=21_600.0).contains(&t) || z.abs() > 1_000.0 {
            return Err(DataError::Parse(path.to_path_buf(), line, "value out of range".into()));
        }
        let index = (t * RATE_HZ).round();
        if (index - t * RATE_HZ).abs() > 0.1 {
            irregular += 1;
            continue;
        }
        let index = index as usize;
        if index < samples.len() {
            irregular += 1;
            continue;
        }
        if index > MAX_SAMPLES {
            return Err(DataError::TooLarge(path.to_path_buf()));
        }
        samples.resize(index, Sample::Missing);
        samples.push(if z.abs() >= CLIP_OHMS {
            Sample::Clipped
        } else {
            Sample::Ohms(z)
        });
    }
    Ok((samples, irregular))
}

/// Read the analysis start times for every case: `(case id, times)`.
///
/// # Errors
/// If the file is missing or malformed.
pub fn read_analyses(path: &Path) -> Result<Vec<(u32, Vec<f64>)>, DataError> {
    let mut out = Vec::new();
    for (line, text) in lines(path)? {
        let Some((id, list)) = text.split_once(';') else {
            return Err(DataError::Parse(path.to_path_buf(), line, "expected id;list".into()));
        };
        let id = number(path, line, id)?;
        if !(1.0..=10_000.0).contains(&id) || id.fract() != 0.0 {
            return Err(DataError::Parse(path.to_path_buf(), line, "bad id".into()));
        }
        let inner = list.trim().trim_start_matches('[').trim_end_matches(']');
        let mut times = Vec::new();
        for part in inner.split(',').filter(|p| !p.trim().is_empty()) {
            times.push(number(path, line, part)?);
        }
        out.push((id as u32, times));
    }
    Ok(out)
}

/// Read the breath times of one case, seconds on the monitor's clock.
///
/// # Errors
/// If the file is malformed.
pub fn read_breaths(path: &Path) -> Result<Vec<f64>, DataError> {
    let mut out = Vec::new();
    for (line, text) in lines(path)? {
        let cols: Vec<&str> = text.split(';').collect();
        let Some(time) = cols.get(1) else {
            return Err(DataError::Parse(path.to_path_buf(), line, "missing time".into()));
        };
        let hms = time.rsplit(' ').next().unwrap_or("");
        let parts: Vec<&str> = hms.split(':').collect();
        let [h, m, s] = parts.as_slice() else {
            return Err(DataError::Parse(path.to_path_buf(), line, "bad time".into()));
        };
        let secs = number(path, line, h)? * 3600.0 + number(path, line, m)? * 60.0 + number(path, line, s)?;
        out.push(secs);
    }
    Ok(out)
}

/// Case names in a split list file (one per line; blank lines ignored).
/// Only names of the form `intervention_NNN` are accepted.
///
/// # Errors
/// If the file cannot be read or holds anything else.
pub fn read_list(path: &Path) -> Result<Vec<String>, DataError> {
    let text = fs::read_to_string(path).map_err(|e| DataError::Io(path.to_path_buf(), e))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let name = line.trim();
        if name.is_empty() {
            continue;
        }
        if case_id(name).is_none() {
            return Err(DataError::Parse(path.to_path_buf(), i + 1, format!("bad case name {name:?}")));
        }
        out.push(name.to_string());
    }
    Ok(out)
}

/// `4` for `intervention_004`.
#[must_use]
pub fn case_id(name: &str) -> Option<u32> {
    let digits = name.strip_prefix("intervention_")?;
    if digits.len() != 3 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Load the named cases from the dataset's `data` folder.
///
/// # Errors
/// If the analysis list cannot be read. A case that cannot be read is
/// returned as an error in its own slot.
pub fn load(root: &Path, names: &[String]) -> Result<Vec<Result<Case, DataError>>, DataError> {
    let analyses = read_analyses(&root.join("Rhythm_analysis.csv"))?;
    Ok(names
        .iter()
        .map(|name| {
            let id = case_id(name)
                .ok_or_else(|| DataError::Parse(root.to_path_buf(), 0, format!("bad name {name}")))?;
            let dir = root.join(name);
            let (samples, irregular_rows) = read_impedance(&dir.join("Impedance_signal.csv"))?;
            let breaths_s = read_breaths(&dir.join("Ventilation_volumes.csv")).unwrap_or_default();
            let analyses_s = analyses
                .iter()
                .find(|(i, _)| *i == id)
                .map(|(_, t)| t.clone())
                .unwrap_or_default();
            Ok(Case {
                name: name.clone(),
                id,
                samples,
                analyses_s,
                breaths_s,
                irregular_rows,
            })
        })
        .collect())
}

/// Where the downloaded data is looked for: the `AED_CPR_DATA` environment
/// variable if set, otherwise `data/mendeley/paris-impedance/data` at the
/// repository root (git-ignored; never committed).
#[must_use]
pub fn default_root() -> PathBuf {
    std::env::var_os("AED_CPR_DATA").map_or_else(
        || {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../data/mendeley/paris-impedance/data")
        },
        PathBuf::from,
    )
}
