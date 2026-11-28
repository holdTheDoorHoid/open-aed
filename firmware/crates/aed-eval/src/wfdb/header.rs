// SPDX-License-Identifier: GPL-3.0-or-later
//! Reader for WFDB header (`.hea`) files.
//!
//! A header is a small text file. Its first non-comment line (the "record
//! line") gives the record name, the number of signals, the sampling rate and
//! the number of samples. One "signal line" follows per signal, giving the
//! data file, its storage format, the gain (ADC units per millivolt) with an
//! optional baseline and units, the ADC resolution, the ADC zero, the first
//! sample value, a checksum of all samples, the block size and a
//! description. Lines starting with `#` are comments.
//!
//! The file is treated as hostile: every field is checked, lengths are
//! limited, and anything unexpected is an error rather than a guess.

use core::fmt;

/// Longest header file accepted, in bytes. Real headers are a few hundred bytes.
pub const MAX_HEADER_BYTES: usize = 64 * 1024;

/// Most signals accepted in one record.
pub const MAX_SIGNALS: usize = 64;

/// Gain used by WFDB when the header gives none or gives zero (ADC units per mV).
pub const DEFAULT_GAIN: f64 = 200.0;

/// Sampling rate used by WFDB when the header gives none (Hz).
pub const DEFAULT_SAMPLING_HZ: f64 = 250.0;

/// Something wrong with a header file.
#[derive(Debug, Clone, PartialEq)]
pub enum HeaderError {
    /// The file is larger than [`MAX_HEADER_BYTES`].
    TooLarge(usize),
    /// The file is not plain ASCII/UTF-8 text.
    NotText,
    /// There is no record line.
    MissingRecordLine,
    /// A field could not be read; says which one and on which line.
    BadField {
        /// Line number, counting from 1.
        line: usize,
        /// Name of the field.
        field: &'static str,
        /// The text found.
        text: String,
    },
    /// A required field is absent.
    MissingField {
        /// Line number, counting from 1.
        line: usize,
        /// Name of the field.
        field: &'static str,
    },
    /// Multi-segment records are not supported.
    MultiSegment,
    /// The number of signals is zero or above [`MAX_SIGNALS`].
    BadSignalCount(usize),
    /// Fewer signal lines than the record line promised.
    MissingSignalLines {
        /// Signals promised by the record line.
        expected: usize,
        /// Signal lines found.
        found: usize,
    },
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeaderError::TooLarge(n) => write!(f, "header is {n} bytes, larger than allowed"),
            HeaderError::NotText => write!(f, "header is not text"),
            HeaderError::MissingRecordLine => write!(f, "header has no record line"),
            HeaderError::BadField { line, field, text } => {
                write!(f, "header line {line}: cannot read {field} from {text:?}")
            }
            HeaderError::MissingField { line, field } => {
                write!(f, "header line {line}: {field} is missing")
            }
            HeaderError::MultiSegment => write!(f, "multi-segment records are not supported"),
            HeaderError::BadSignalCount(n) => write!(f, "unsupported number of signals: {n}"),
            HeaderError::MissingSignalLines { expected, found } => {
                write!(
                    f,
                    "header promises {expected} signals but has {found} signal lines"
                )
            }
        }
    }
}

impl std::error::Error for HeaderError {}

/// The record line.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordLine {
    /// Record name.
    pub name: String,
    /// Number of signals.
    pub signal_count: usize,
    /// Samples per second per signal.
    pub sampling_hz: f64,
    /// Samples per signal, if given.
    pub samples_per_signal: Option<u64>,
}

/// One signal line.
#[derive(Debug, Clone, PartialEq)]
pub struct SignalSpec {
    /// Name of the file holding the samples.
    pub file_name: String,
    /// Storage format number (for example 212).
    pub format: u16,
    /// Samples per frame (the `x` suffix); 1 if absent.
    pub samples_per_frame: u32,
    /// Skew in samples (the `:` suffix); 0 if absent.
    pub skew: i64,
    /// Byte offset of the first sample in the file (the `+` suffix); 0 if absent.
    pub byte_offset: u64,
    /// ADC units per physical unit (normally per millivolt).
    pub gain: f64,
    /// True if the header gave no gain or a zero gain, so [`DEFAULT_GAIN`] is
    /// used and the amplitude scale is not really known.
    pub gain_is_default: bool,
    /// ADC value corresponding to 0 physical units. Defaults to `adc_zero`.
    pub baseline: i64,
    /// Physical units text, "mV" if absent.
    pub units: String,
    /// ADC resolution in bits; 0 if absent.
    pub adc_resolution: u32,
    /// ADC value at the middle of the ADC range; 0 if absent.
    pub adc_zero: i64,
    /// Value of the first sample, if given.
    pub initial_value: Option<i64>,
    /// 16-bit checksum of all samples, if given.
    pub checksum: Option<i64>,
    /// Block size; 0 if absent.
    pub block_size: u64,
    /// Free-text description (often the lead name).
    pub description: String,
}

/// A parsed header.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// The record line.
    pub record: RecordLine,
    /// One entry per signal, in order.
    pub signals: Vec<SignalSpec>,
    /// Comment lines, without the leading `#`.
    pub comments: Vec<String>,
}

fn bad(line: usize, field: &'static str, text: &str) -> HeaderError {
    HeaderError::BadField {
        line,
        field,
        text: text.chars().take(80).collect(),
    }
}

fn parse_f64(line: usize, field: &'static str, text: &str) -> Result<f64, HeaderError> {
    let v: f64 = text.parse().map_err(|_| bad(line, field, text))?;
    if v.is_finite() {
        Ok(v)
    } else {
        Err(bad(line, field, text))
    }
}

fn parse_i64(line: usize, field: &'static str, text: &str) -> Result<i64, HeaderError> {
    text.parse().map_err(|_| bad(line, field, text))
}

fn parse_u64(line: usize, field: &'static str, text: &str) -> Result<u64, HeaderError> {
    text.parse().map_err(|_| bad(line, field, text))
}

/// Parse a header from its bytes.
///
/// # Errors
/// Returns [`HeaderError`] for anything that is not a well-formed
/// single-segment header.
pub fn parse_header(bytes: &[u8]) -> Result<Header, HeaderError> {
    if bytes.len() > MAX_HEADER_BYTES {
        return Err(HeaderError::TooLarge(bytes.len()));
    }
    let text = core::str::from_utf8(bytes).map_err(|_| HeaderError::NotText)?;
    let mut comments = Vec::new();
    let mut record: Option<RecordLine> = None;
    let mut signals = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if let Some(c) = trimmed.strip_prefix('#') {
            comments.push(c.trim().to_string());
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        match &record {
            None => record = Some(parse_record_line(line_no, trimmed)?),
            Some(r) => {
                if signals.len() < r.signal_count {
                    signals.push(parse_signal_line(line_no, trimmed)?);
                }
                // Extra non-comment lines after the signal lines are info
                // strings in some headers; they are ignored.
            }
        }
    }
    let record = record.ok_or(HeaderError::MissingRecordLine)?;
    if signals.len() != record.signal_count {
        return Err(HeaderError::MissingSignalLines {
            expected: record.signal_count,
            found: signals.len(),
        });
    }
    Ok(Header {
        record,
        signals,
        comments,
    })
}

fn parse_record_line(line: usize, text: &str) -> Result<RecordLine, HeaderError> {
    let mut fields = text.split_whitespace();
    let name_field = fields.next().ok_or(HeaderError::MissingField {
        line,
        field: "record name",
    })?;
    if name_field.contains('/') {
        return Err(HeaderError::MultiSegment);
    }
    if name_field.len() > 64
        || !name_field
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(bad(line, "record name", name_field));
    }
    let nsig_text = fields.next().ok_or(HeaderError::MissingField {
        line,
        field: "number of signals",
    })?;
    let signal_count = usize::try_from(parse_u64(line, "number of signals", nsig_text)?)
        .map_err(|_| bad(line, "number of signals", nsig_text))?;
    if signal_count == 0 || signal_count > MAX_SIGNALS {
        return Err(HeaderError::BadSignalCount(signal_count));
    }
    let sampling_hz = match fields.next() {
        None => DEFAULT_SAMPLING_HZ,
        Some(fs) => {
            // Sampling frequency may carry "/counter(base)"; only the first part matters here.
            let main = fs.split('/').next().unwrap_or(fs);
            let v = parse_f64(line, "sampling frequency", main)?;
            if v <= 0.0 || v > 1.0e6 {
                return Err(bad(line, "sampling frequency", fs));
            }
            v
        }
    };
    let samples_per_signal = match fields.next() {
        None => None,
        Some(n) => Some(parse_u64(line, "number of samples", n)?),
    };
    Ok(RecordLine {
        name: name_field.to_string(),
        signal_count,
        sampling_hz,
        samples_per_signal,
    })
}

fn parse_format(line: usize, text: &str) -> Result<(u16, u32, i64, u64), HeaderError> {
    // format[xspf][:skew][+offset]
    let (rest, offset) = match text.split_once('+') {
        Some((a, b)) => (a, parse_u64(line, "byte offset", b)?),
        None => (text, 0),
    };
    let (rest, skew) = match rest.split_once(':') {
        Some((a, b)) => (a, parse_i64(line, "skew", b)?),
        None => (rest, 0),
    };
    let (fmt_text, spf) = match rest.split_once('x') {
        Some((a, b)) => {
            let spf = u32::try_from(parse_u64(line, "samples per frame", b)?)
                .map_err(|_| bad(line, "samples per frame", b))?;
            (a, spf)
        }
        None => (rest, 1),
    };
    let format = u16::try_from(parse_u64(line, "format", fmt_text)?)
        .map_err(|_| bad(line, "format", fmt_text))?;
    if spf == 0 {
        return Err(bad(line, "samples per frame", text));
    }
    Ok((format, spf, skew, offset))
}

fn parse_signal_line(line: usize, text: &str) -> Result<SignalSpec, HeaderError> {
    let mut fields = text.split_whitespace();
    let file_name = fields.next().ok_or(HeaderError::MissingField {
        line,
        field: "file name",
    })?;
    if file_name.len() > 128
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.starts_with('.')
    {
        // Refuse anything that could point outside the record's directory.
        return Err(bad(line, "file name", file_name));
    }
    let fmt_text = fields.next().ok_or(HeaderError::MissingField {
        line,
        field: "format",
    })?;
    let (format, samples_per_frame, skew, byte_offset) = parse_format(line, fmt_text)?;

    let mut spec = SignalSpec {
        file_name: file_name.to_string(),
        format,
        samples_per_frame,
        skew,
        byte_offset,
        gain: DEFAULT_GAIN,
        gain_is_default: true,
        baseline: 0,
        units: "mV".to_string(),
        adc_resolution: 0,
        adc_zero: 0,
        initial_value: None,
        checksum: None,
        block_size: 0,
        description: String::new(),
    };
    let mut baseline: Option<i64> = None;

    if let Some(g) = fields.next() {
        // gain[(baseline)][/units]
        let (g, units) = match g.split_once('/') {
            Some((a, u)) => (a, Some(u)),
            None => (g, None),
        };
        let gain_text = match g.split_once('(') {
            Some((a, b)) => {
                let inner = b
                    .strip_suffix(')')
                    .ok_or_else(|| bad(line, "baseline", g))?;
                baseline = Some(parse_i64(line, "baseline", inner)?);
                a
            }
            None => g,
        };
        let gain = parse_f64(line, "gain", gain_text)?;
        if gain < 0.0 {
            return Err(bad(line, "gain", gain_text));
        }
        if gain > 0.0 {
            spec.gain = gain;
            spec.gain_is_default = false;
        }
        if let Some(u) = units {
            spec.units = u.chars().take(20).collect();
        }
    }
    if let Some(r) = fields.next() {
        spec.adc_resolution = u32::try_from(parse_u64(line, "ADC resolution", r)?)
            .map_err(|_| bad(line, "ADC resolution", r))?;
        if spec.adc_resolution > 32 {
            return Err(bad(line, "ADC resolution", r));
        }
    }
    if let Some(z) = fields.next() {
        spec.adc_zero = parse_i64(line, "ADC zero", z)?;
    }
    if let Some(iv) = fields.next() {
        spec.initial_value = Some(parse_i64(line, "initial value", iv)?);
    }
    if let Some(c) = fields.next() {
        spec.checksum = Some(parse_i64(line, "checksum", c)?);
    }
    if let Some(b) = fields.next() {
        spec.block_size = parse_u64(line, "block size", b)?;
    }
    let description: Vec<&str> = fields.collect();
    spec.description = description.join(" ").chars().take(200).collect();
    // WFDB: an absent baseline equals the ADC zero.
    spec.baseline = baseline.unwrap_or(spec.adc_zero);
    Ok(spec)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_two_signal_header_with_defaults() {
        let h = parse_header(
            b"100 2 360 650000\n100.dat 212 200 11 1024 995 -22131 0 MLII\n\
              100.dat 212 200 11 1024 1011 20052 0 V5\n# 69 M 1085 1629 x1\n",
        )
        .unwrap();
        assert_eq!(h.record.name, "100");
        assert_eq!(h.record.signal_count, 2);
        assert_eq!(h.record.sampling_hz, 360.0);
        assert_eq!(h.record.samples_per_signal, Some(650_000));
        let s = &h.signals[0];
        assert_eq!(s.format, 212);
        assert_eq!(s.gain, 200.0);
        assert!(!s.gain_is_default);
        assert_eq!(s.adc_zero, 1024);
        assert_eq!(s.baseline, 1024, "absent baseline must equal ADC zero");
        assert_eq!(s.initial_value, Some(995));
        assert_eq!(s.checksum, Some(-22131));
        assert_eq!(s.description, "MLII");
        assert_eq!(h.comments.len(), 1);
    }

    #[test]
    fn parses_baseline_units_and_format_suffixes() {
        let h =
            parse_header(b"r 1 250/1000(0) 10\nr.dat 212x2:3+512 400(-5)/uV 12 7 1 2 0 lead I\n")
                .unwrap();
        let s = &h.signals[0];
        assert_eq!(h.record.sampling_hz, 250.0);
        assert_eq!(s.samples_per_frame, 2);
        assert_eq!(s.skew, 3);
        assert_eq!(s.byte_offset, 512);
        assert_eq!(s.gain, 400.0);
        assert_eq!(s.baseline, -5);
        assert_eq!(s.units, "uV");
        assert_eq!(s.adc_zero, 7);
        assert_eq!(s.description, "lead I");
    }

    #[test]
    fn zero_or_missing_gain_is_flagged_as_default() {
        let h = parse_header(b"r 1 250 10\nr.dat 212 0\n").unwrap();
        assert!(h.signals[0].gain_is_default);
        assert_eq!(h.signals[0].gain, DEFAULT_GAIN);
        let h = parse_header(b"r 1\nr.dat 212\n").unwrap();
        assert!(h.signals[0].gain_is_default);
        assert_eq!(h.record.sampling_hz, DEFAULT_SAMPLING_HZ);
    }

    #[test]
    fn hostile_headers_are_errors_not_panics() {
        let cases: &[&[u8]] = &[
            b"",
            b"# only a comment\n",
            b"r 0 250\n",
            b"r 99999999999999999999 250\n",
            b"r 2 250 10\nr.dat 212\n",
            b"r 1 nan 10\nr.dat 212\n",
            b"r 1 -250 10\nr.dat 212\n",
            b"r 1 250 10\n../etc/passwd 212\n",
            b"r 1 250 10\nr.dat 212x0\n",
            b"r 1 250 10\nr.dat 99999999\n",
            b"r 1 250 10\nr.dat 212 -1\n",
            b"r 1 250 10\nr.dat 212 200(3\n",
            b"r/3 1 250\n",
            b"\xff\xfe\n",
            b"r 1 250 -10\nr.dat 212\n",
        ];
        for c in cases {
            assert!(
                parse_header(c).is_err(),
                "accepted {:?}",
                String::from_utf8_lossy(c)
            );
        }
        assert!(matches!(
            parse_header(&vec![b'#'; MAX_HEADER_BYTES + 1]),
            Err(HeaderError::TooLarge(_))
        ));
    }
}
