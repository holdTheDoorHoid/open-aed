// SPDX-License-Identifier: GPL-3.0-or-later
//! Readers for the PhysioNet WFDB file formats: headers, format-212 sample
//! files and MIT-format annotation files, plus [`load_record`], which reads
//! all three for one record and cross-checks them.
//!
//! Every file is treated as hostile input: sizes are limited, every field is
//! checked, and problems come back as errors, never as a crash.

pub mod annot;
pub mod header;
pub mod signal;

use core::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};

pub use annot::{parse_annotations, Annotation};
pub use header::{parse_header, Header, SignalSpec};

/// Anything that can go wrong loading a record.
#[derive(Debug)]
pub enum LoadError {
    /// A file could not be read.
    Io(PathBuf, std::io::Error),
    /// A file is larger than allowed.
    TooLarge(PathBuf),
    /// The header is malformed.
    Header(header::HeaderError),
    /// The sample file is malformed or unsupported.
    Signal(signal::SignalError),
    /// The annotation file is malformed.
    Annot(annot::AnnotError),
    /// The record name is not a plain name.
    BadName(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(p, e) => write!(f, "{}: {e}", p.display()),
            LoadError::TooLarge(p) => write!(f, "{}: file too large", p.display()),
            LoadError::Header(e) => write!(f, "{e}"),
            LoadError::Signal(e) => write!(f, "{e}"),
            LoadError::Annot(e) => write!(f, "{e}"),
            LoadError::BadName(n) => write!(f, "bad record name {n:?}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Read a whole file, refusing anything above `limit` bytes.
///
/// # Errors
/// I/O errors, or [`LoadError::TooLarge`].
pub fn read_limited(path: &Path, limit: usize) -> Result<Vec<u8>, LoadError> {
    let file = std::fs::File::open(path).map_err(|e| LoadError::Io(path.to_path_buf(), e))?;
    let mut buf = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|e| LoadError::Io(path.to_path_buf(), e))?;
    if buf.len() > limit {
        return Err(LoadError::TooLarge(path.to_path_buf()));
    }
    Ok(buf)
}

/// Consistency checks between a header and the samples actually decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalCheck {
    /// Whether the header's checksum matches the samples (None if the header gives none).
    pub checksum_ok: Option<bool>,
    /// Whether the first sample equals the header's initial value (None if none given).
    pub initial_value_ok: Option<bool>,
    /// Samples at the lowest or highest value the ADC can produce (by its resolution).
    pub adc_rail_samples: u64,
}

/// One record: header, raw samples per signal (ADC units), annotations.
#[derive(Debug, Clone)]
pub struct Record {
    /// The parsed header.
    pub header: Header,
    /// Raw samples, one vector per signal.
    pub samples: Vec<Vec<i16>>,
    /// Reference annotations.
    pub annotations: Vec<Annotation>,
    /// Header-versus-data checks, one per signal.
    pub checks: Vec<SignalCheck>,
}

/// Load a record `name` from `dir`, reading `<name>.hea`, the sample file the
/// header names and `<name>.<annotator>`.
///
/// Only format 212 with all signals in one file, one sample per frame and no
/// skew is supported (that covers cudb, vfdb and mitdb).
///
/// # Errors
/// [`LoadError`] for unreadable, malformed or unsupported files.
pub fn load_record(dir: &Path, name: &str, annotator: &str) -> Result<Record, LoadError> {
    let plain = |s: &str| {
        !s.is_empty()
            && s.len() <= 64
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    };
    if !plain(name) || !plain(annotator) {
        return Err(LoadError::BadName(format!("{name}.{annotator}")));
    }
    let hea = read_limited(&dir.join(format!("{name}.hea")), header::MAX_HEADER_BYTES)?;
    let header = parse_header(&hea).map_err(LoadError::Header)?;
    let first = header
        .signals
        .first()
        .ok_or(LoadError::Signal(signal::SignalError::NoSignals))?;
    for s in &header.signals {
        if s.format != 212 {
            return Err(LoadError::Signal(signal::SignalError::Unsupported(format!(
                "format {}",
                s.format
            ))));
        }
        if s.file_name != first.file_name || s.byte_offset != first.byte_offset {
            return Err(LoadError::Signal(signal::SignalError::Unsupported(
                "signals in different files".into(),
            )));
        }
        if s.samples_per_frame != 1 || s.skew != 0 {
            return Err(LoadError::Signal(signal::SignalError::Unsupported(
                "multi-frequency or skewed signals".into(),
            )));
        }
    }
    let dat = read_limited(&dir.join(&first.file_name), signal::MAX_SIGNAL_BYTES)?;
    let samples = signal::decode_212(
        &dat,
        header.record.signal_count,
        first.byte_offset,
        header.record.samples_per_signal,
    )
    .map_err(LoadError::Signal)?;
    let checks = header
        .signals
        .iter()
        .zip(samples.iter())
        .map(|(spec, s)| check_signal(spec, s))
        .collect();
    let atr = read_limited(
        &dir.join(format!("{name}.{annotator}")),
        annot::MAX_ANNOTATION_BYTES,
    )?;
    let annotations = parse_annotations(&atr).map_err(LoadError::Annot)?;
    Ok(Record {
        header,
        samples,
        annotations,
        checks,
    })
}

/// Compare a decoded signal with what its header says.
#[must_use]
pub fn check_signal(spec: &SignalSpec, samples: &[i16]) -> SignalCheck {
    let computed = signal::checksum(samples);
    let (lo, hi) = adc_rails(spec);
    SignalCheck {
        checksum_ok: spec.checksum.map(|c| signal::checksum_matches(c, computed)),
        initial_value_ok: spec
            .initial_value
            .map(|iv| samples.first().is_some_and(|f| i64::from(*f) == iv)),
        adc_rail_samples: samples
            .iter()
            .filter(|v| i64::from(**v) <= lo || i64::from(**v) >= hi)
            .count() as u64,
    }
}

/// Lowest and highest values the ADC can produce, from its resolution and
/// zero. Format 212 holds 12 bits, so that is the default range.
#[must_use]
pub fn adc_rails(spec: &SignalSpec) -> (i64, i64) {
    let bits = if spec.adc_resolution == 0 || spec.adc_resolution > 12 {
        12
    } else {
        spec.adc_resolution
    };
    let half = 1_i64 << (bits - 1);
    (spec.adc_zero - half, spec.adc_zero + half - 1)
}

/// Read a PhysioNet `RECORDS` file (one record name per line).
///
/// # Errors
/// I/O errors or a file too large.
pub fn read_records_list(dir: &Path) -> Result<Vec<String>, LoadError> {
    let bytes = read_limited(&dir.join("RECORDS"), 64 * 1024)?;
    Ok(String::from_utf8_lossy(&bytes)
        .lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty() && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
        .map(str::to_string)
        .collect())
}
