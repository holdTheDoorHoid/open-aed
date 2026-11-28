// SPDX-License-Identifier: GPL-3.0-or-later
//! Decoder for WFDB signal files in format 212.
//!
//! Format 212 packs two 12-bit samples into three bytes. Byte 0 holds the
//! low 8 bits of the first sample; the low 4 bits of byte 1 hold its top 4
//! bits; the high 4 bits of byte 1 hold the top 4 bits of the second sample;
//! byte 2 holds the low 8 bits of the second sample. Samples are two's
//! complement. With several signals in one file the samples are interleaved
//! (signal 0, signal 1, signal 0, ...) and the packing runs across that
//! interleaved stream.

use core::fmt;

/// Largest signal file accepted, in bytes (64 MB; the real files are under 2 MB).
pub const MAX_SIGNAL_BYTES: usize = 64 * 1024 * 1024;

/// Something wrong with a signal file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalError {
    /// The file is larger than [`MAX_SIGNAL_BYTES`].
    TooLarge(usize),
    /// The header asks for something this reader does not handle.
    Unsupported(String),
    /// The byte offset given in the header is past the end of the file.
    OffsetPastEnd,
    /// The file holds fewer samples than the header says.
    Truncated {
        /// Samples per signal the header promised.
        expected: u64,
        /// Complete frames found.
        found: u64,
    },
    /// The number of signals is zero.
    NoSignals,
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalError::TooLarge(n) => write!(f, "signal file is {n} bytes, larger than allowed"),
            SignalError::Unsupported(s) => write!(f, "unsupported signal layout: {s}"),
            SignalError::OffsetPastEnd => write!(f, "byte offset is past the end of the file"),
            SignalError::Truncated { expected, found } => {
                write!(
                    f,
                    "signal file holds {found} samples per signal, header says {expected}"
                )
            }
            SignalError::NoSignals => write!(f, "no signals"),
        }
    }
}

impl std::error::Error for SignalError {}

/// Sign-extend a 12-bit value.
fn sext12(v: u16) -> i16 {
    let v = v & 0x0fff;
    if v & 0x0800 != 0 {
        (v | 0xf000) as i16
    } else {
        v as i16
    }
}

/// Decode a whole format-212 byte stream into samples (interleaved order).
///
/// A final group of only two bytes (an odd number of samples) yields one
/// sample; a final lone byte is ignored.
#[must_use]
pub fn decode_212_stream(bytes: &[u8]) -> Vec<i16> {
    let mut out = Vec::with_capacity(bytes.len() / 3 * 2 + 1);
    let (triples, rest) = bytes.as_chunks::<3>();
    for &[b0, b1, b2] in triples {
        out.push(sext12(u16::from(b0) | (u16::from(b1 & 0x0f) << 8)));
        out.push(sext12(u16::from(b2) | (u16::from(b1 & 0xf0) << 4)));
    }
    if let [b0, b1] = *rest {
        out.push(sext12(u16::from(b0) | (u16::from(b1 & 0x0f) << 8)));
    }
    out
}

/// Decode a format-212 file holding `signal_count` interleaved signals,
/// starting at `byte_offset`, into one vector per signal.
///
/// If `expected_samples` is given, the file must hold at least that many
/// complete frames, and only that many are returned. Otherwise every complete
/// frame is returned.
///
/// # Errors
/// Returns [`SignalError`] if the file is too large, too short, or the
/// offset is past its end.
pub fn decode_212(
    bytes: &[u8],
    signal_count: usize,
    byte_offset: u64,
    expected_samples: Option<u64>,
) -> Result<Vec<Vec<i16>>, SignalError> {
    if bytes.len() > MAX_SIGNAL_BYTES {
        return Err(SignalError::TooLarge(bytes.len()));
    }
    if signal_count == 0 {
        return Err(SignalError::NoSignals);
    }
    let offset = usize::try_from(byte_offset).map_err(|_| SignalError::OffsetPastEnd)?;
    let body = bytes.get(offset..).ok_or(SignalError::OffsetPastEnd)?;
    let flat = decode_212_stream(body);
    let frames = (flat.len() / signal_count) as u64;
    let keep = match expected_samples {
        Some(n) if n > frames => {
            return Err(SignalError::Truncated {
                expected: n,
                found: frames,
            })
        }
        Some(n) => n,
        None => frames,
    };
    let keep =
        usize::try_from(keep).map_err(|_| SignalError::Unsupported("too many samples".into()))?;
    let mut out: Vec<Vec<i16>> = (0..signal_count)
        .map(|_| Vec::with_capacity(keep))
        .collect();
    for frame in flat.chunks_exact(signal_count).take(keep) {
        for (sig, v) in out.iter_mut().zip(frame.iter()) {
            sig.push(*v);
        }
    }
    Ok(out)
}

/// The WFDB checksum: the sum of all samples of a signal, kept to 16 bits
/// (two's complement).
#[must_use]
pub fn checksum(samples: &[i16]) -> i16 {
    samples.iter().fold(0_i16, |acc, v| acc.wrapping_add(*v))
}

/// True if a header checksum (as written, possibly outside the i16 range in
/// odd files) matches the computed one.
#[must_use]
pub fn checksum_matches(header_value: i64, computed: i16) -> bool {
    // Compare modulo 2^16, as WFDB does.
    (header_value as u16 as i16) == computed
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    /// Pack samples into format 212 (the inverse of the decoder), for fixtures.
    fn encode(samples: &[i16]) -> Vec<u8> {
        let mut out = Vec::new();
        for pair in samples.chunks(2) {
            let a = pair[0] as u16 & 0x0fff;
            match pair.get(1) {
                Some(b) => {
                    let b = *b as u16 & 0x0fff;
                    out.push((a & 0xff) as u8);
                    out.push(((a >> 8) as u8) | (((b >> 8) as u8) << 4));
                    out.push((b & 0xff) as u8);
                }
                None => {
                    out.push((a & 0xff) as u8);
                    out.push((a >> 8) as u8);
                }
            }
        }
        out
    }

    #[test]
    fn decodes_hand_built_bytes() {
        // Samples 0x123 and -1 (0xFFF): bytes 0x23, 0xF1, 0xFF.
        assert_eq!(decode_212_stream(&[0x23, 0xF1, 0xFF]), vec![0x123, -1]);
        // -2048 and 2047: 0x800 and 0x7FF -> bytes 0x00, 0x78, 0xFF.
        assert_eq!(decode_212_stream(&[0x00, 0x78, 0xFF]), vec![-2048, 2047]);
    }

    #[test]
    fn round_trip_and_interleaving() {
        let s0: Vec<i16> = (0..7).map(|i| i * 300 - 1000).collect();
        let s1: Vec<i16> = (0..7).map(|i| 2047 - i * 500).collect();
        let mut inter = Vec::new();
        for (a, b) in s0.iter().zip(s1.iter()) {
            inter.push(*a);
            inter.push(*b);
        }
        let bytes = encode(&inter);
        let out = decode_212(&bytes, 2, 0, Some(7)).unwrap();
        assert_eq!(out[0], s0);
        assert_eq!(out[1], s1);
        assert_eq!(checksum(&out[0]), s0.iter().sum::<i16>());
    }

    #[test]
    fn odd_sample_count_and_offset() {
        let s: Vec<i16> = vec![5, -6, 7];
        let mut bytes = vec![0xAA; 4];
        bytes.extend(encode(&s));
        let out = decode_212(&bytes, 1, 4, Some(3)).unwrap();
        assert_eq!(out[0], s);
    }

    #[test]
    fn short_or_hostile_files_are_errors() {
        assert!(matches!(
            decode_212(&[1, 2, 3], 1, 0, Some(5)),
            Err(SignalError::Truncated { .. })
        ));
        assert_eq!(
            decode_212(&[1, 2, 3], 1, 10, None),
            Err(SignalError::OffsetPastEnd)
        );
        assert_eq!(
            decode_212(&[1, 2, 3], 0, 0, None),
            Err(SignalError::NoSignals)
        );
        assert_eq!(
            decode_212(&[], 2, 0, None).unwrap(),
            vec![Vec::<i16>::new(), Vec::new()]
        );
        assert_eq!(decode_212_stream(&[7]), Vec::<i16>::new());
        assert_eq!(
            decode_212(&[1, 2, 3], 1, u64::MAX, None),
            Err(SignalError::OffsetPastEnd)
        );
    }

    #[test]
    fn checksum_wraps_like_wfdb() {
        assert_eq!(checksum(&[30_000, 30_000]), 30_000_i16.wrapping_add(30_000));
        assert!(checksum_matches(-22131, -22131));
        assert!(checksum_matches(65535, -1));
        assert!(!checksum_matches(1, 2));
    }
}
