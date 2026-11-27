// SPDX-License-Identifier: GPL-3.0-or-later
//! Reader for WFDB annotation files in the MIT format (`.atr`).
//!
//! The file is a sequence of 16-bit little-endian words. In each word the
//! top 6 bits are a code (`A`) and the low 10 bits a number (`I`).
//!
//! * `A` from 1 to 49 (and `A = 0` with `I` not zero): a new annotation of
//!   type `A`, `I` sample intervals after the previous one.
//! * `A = 59` (SKIP): the next 4 bytes are a 32-bit interval to add to the
//!   time, high 16 bits first, each half least significant byte first.
//! * `A = 60` (NUM), `62` (CHN): set the `num` / `chan` field of the
//!   current annotation and of the following ones.
//! * `A = 61` (SUB): set the `subtyp` field of the current annotation only.
//! * `A = 63` (AUX): `I` bytes of free text follow (padded to an even
//!   length), attached to the current annotation. Rhythm labels such as
//!   `(VF` live here.
//! * `A = 0, I = 0`: end of file.
//!
//! The modifiers follow the annotation word they apply to. Source: the WFDB
//! Applications Guide, annot(5), physionet.org/physiotools/wag/annot-5.htm.

use core::fmt;

/// Largest annotation file accepted, in bytes.
pub const MAX_ANNOTATION_BYTES: usize = 16 * 1024 * 1024;

/// Annotation type codes used here (from WFDB's `ecgcodes.h`).
pub mod code {
    /// Not a QRS (code 0).
    pub const NOTQRS: u8 = 0;
    /// Normal beat, `N`.
    pub const NORMAL: u8 = 1;
    /// Left bundle branch block beat, `L`.
    pub const LBBB: u8 = 2;
    /// Right bundle branch block beat, `R`.
    pub const RBBB: u8 = 3;
    /// Aberrated atrial premature beat, `a`.
    pub const ABERR: u8 = 4;
    /// Premature ventricular contraction, `V`.
    pub const PVC: u8 = 5;
    /// Fusion of ventricular and normal beat, `F`.
    pub const FUSION: u8 = 6;
    /// Nodal premature beat, `J`.
    pub const NPC: u8 = 7;
    /// Atrial premature beat, `A`.
    pub const APC: u8 = 8;
    /// Supraventricular premature beat, `S`.
    pub const SVPB: u8 = 9;
    /// Ventricular escape beat, `E`.
    pub const VESC: u8 = 10;
    /// Nodal escape beat, `j`.
    pub const NESC: u8 = 11;
    /// Paced beat, `/`.
    pub const PACE: u8 = 12;
    /// Unclassifiable beat, `Q`.
    pub const UNKNOWN: u8 = 13;
    /// Signal quality change / noise, `~`.
    pub const NOISE: u8 = 14;
    /// Isolated QRS-like artifact, `|`.
    pub const ARFCT: u8 = 16;
    /// Comment, `"`.
    pub const NOTE: u8 = 22;
    /// Left or right bundle branch block beat (unspecified), `B`.
    pub const BBB: u8 = 25;
    /// Rhythm change, `+` (label in the aux text).
    pub const RHYTHM: u8 = 28;
    /// Ventricular flutter wave, `!`.
    pub const FLWAV: u8 = 31;
    /// Start of ventricular flutter or fibrillation, `[`.
    pub const VFON: u8 = 32;
    /// End of ventricular flutter or fibrillation, `]`.
    pub const VFOFF: u8 = 33;
    /// Atrial escape beat, `e`.
    pub const AESC: u8 = 34;
    /// Supraventricular escape beat, `n`.
    pub const SVESC: u8 = 35;
    /// Non-conducted P wave, `x`.
    pub const NAPC: u8 = 37;
    /// Fusion of paced and normal beat, `f`.
    pub const PFUS: u8 = 38;
    /// Highest ordinary annotation code.
    pub const ACMAX: u8 = 49;
    /// Pseudo-code: long interval follows.
    pub const SKIP: u8 = 59;
    /// Pseudo-code: set `num`.
    pub const NUM: u8 = 60;
    /// Pseudo-code: set `subtyp`.
    pub const SUB: u8 = 61;
    /// Pseudo-code: set `chan`.
    pub const CHN: u8 = 62;
    /// Pseudo-code: aux text follows.
    pub const AUX: u8 = 63;

    /// True for codes WFDB counts as heart beats (its `isqrs` table).
    #[must_use]
    pub fn is_beat(c: u8) -> bool {
        matches!(c, 1..=13 | 25 | 30 | 34 | 35 | 38)
    }

    /// The one-character mnemonic WFDB prints for a code.
    #[must_use]
    pub fn mnemonic(c: u8) -> &'static str {
        match c {
            0 => "NOTQRS",
            1 => "N",
            2 => "L",
            3 => "R",
            4 => "a",
            5 => "V",
            6 => "F",
            7 => "J",
            8 => "A",
            9 => "S",
            10 => "E",
            11 => "j",
            12 => "/",
            13 => "Q",
            14 => "~",
            16 => "|",
            18 => "s",
            19 => "T",
            20 => "*",
            21 => "D",
            22 => "\"",
            23 => "=",
            24 => "p",
            25 => "B",
            26 => "^",
            27 => "t",
            28 => "+",
            29 => "u",
            30 => "?",
            31 => "!",
            32 => "[",
            33 => "]",
            34 => "e",
            35 => "n",
            36 => "@",
            37 => "x",
            38 => "f",
            39 => "(",
            40 => ")",
            41 => "r",
            _ => "?code",
        }
    }
}

/// One annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    /// Time, in samples from the start of the record.
    pub sample: u64,
    /// Type code (see [`code`]).
    pub code: u8,
    /// Subtype (0 unless a SUB word was given).
    pub subtype: i16,
    /// Channel number.
    pub chan: u16,
    /// Annotator number field.
    pub num: u16,
    /// Auxiliary text, with trailing NUL bytes removed and any non-printable
    /// byte replaced by `?`.
    pub aux: String,
}

/// Something wrong with an annotation file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotError {
    /// The file is larger than [`MAX_ANNOTATION_BYTES`].
    TooLarge(usize),
    /// The file ends in the middle of a word, a SKIP interval or aux text.
    Truncated {
        /// Byte position where the problem was found.
        at: usize,
    },
    /// A SUB or AUX word arrived before any annotation.
    ModifierWithoutAnnotation {
        /// Byte position.
        at: usize,
    },
    /// A SKIP moved the time before the start of the record, or time overflowed.
    BadTime {
        /// Byte position.
        at: usize,
    },
    /// A code between 50 and 58, which WFDB does not define.
    UnknownCode {
        /// The code.
        code: u8,
        /// Byte position.
        at: usize,
    },
}

impl fmt::Display for AnnotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnnotError::TooLarge(n) => write!(f, "annotation file is {n} bytes, larger than allowed"),
            AnnotError::Truncated { at } => write!(f, "annotation file truncated at byte {at}"),
            AnnotError::ModifierWithoutAnnotation { at } => {
                write!(f, "annotation modifier with no annotation at byte {at}")
            }
            AnnotError::BadTime { at } => write!(f, "annotation time out of range at byte {at}"),
            AnnotError::UnknownCode { code, at } => {
                write!(f, "undefined annotation code {code} at byte {at}")
            }
        }
    }
}

impl std::error::Error for AnnotError {}

fn word_at(bytes: &[u8], at: usize) -> Option<u16> {
    match bytes.get(at..at.checked_add(2)?)? {
        [lo, hi] => Some(u16::from(*lo) | (u16::from(*hi) << 8)),
        _ => None,
    }
}

fn clean_aux(raw: &[u8]) -> String {
    let end = raw.iter().rposition(|b| *b != 0).map_or(0, |p| p + 1);
    raw.iter()
        .take(end)
        .map(|b| {
            if b.is_ascii_graphic() || *b == b' ' {
                char::from(*b)
            } else {
                '?'
            }
        })
        .collect()
}

/// Parse an MIT-format annotation file.
///
/// A file that simply stops without the end-of-file word is accepted (many
/// writers omit it) as long as it stops on a word boundary.
///
/// # Errors
/// Returns [`AnnotError`] for truncated or malformed content.
pub fn parse_annotations(bytes: &[u8]) -> Result<Vec<Annotation>, AnnotError> {
    if bytes.len() > MAX_ANNOTATION_BYTES {
        return Err(AnnotError::TooLarge(bytes.len()));
    }
    let mut out: Vec<Annotation> = Vec::new();
    let mut time: i64 = 0;
    let mut chan: u16 = 0;
    let mut num: u16 = 0;
    let mut pos = 0_usize;
    while pos < bytes.len() {
        let at = pos;
        let w = word_at(bytes, pos).ok_or(AnnotError::Truncated { at })?;
        pos += 2;
        let a = (w >> 10) as u8;
        let i = w & 0x03ff;
        match a {
            0 if i == 0 => break,
            code::SKIP => {
                let hi = word_at(bytes, pos).ok_or(AnnotError::Truncated { at })?;
                let lo = word_at(bytes, pos + 2).ok_or(AnnotError::Truncated { at })?;
                pos += 4;
                let interval = ((u32::from(hi) << 16) | u32::from(lo)) as i32;
                time = time
                    .checked_add(i64::from(interval))
                    .filter(|t| *t >= 0)
                    .ok_or(AnnotError::BadTime { at })?;
            }
            code::NUM => {
                num = i;
                if let Some(last) = out.last_mut() {
                    last.num = num;
                }
            }
            code::CHN => {
                chan = i;
                if let Some(last) = out.last_mut() {
                    last.chan = chan;
                }
            }
            code::SUB => {
                let last = out
                    .last_mut()
                    .ok_or(AnnotError::ModifierWithoutAnnotation { at })?;
                // The 10-bit field is signed in WFDB's reading.
                last.subtype = if i & 0x200 != 0 { (i | 0xfc00) as i16 } else { i as i16 };
            }
            code::AUX => {
                let len = usize::from(i);
                let raw = bytes
                    .get(pos..pos + len)
                    .ok_or(AnnotError::Truncated { at })?;
                let aux = clean_aux(raw);
                pos += len + (len & 1);
                if pos > bytes.len() {
                    // The pad byte of an odd-length aux string is missing; accept.
                    pos = bytes.len();
                }
                let last = out
                    .last_mut()
                    .ok_or(AnnotError::ModifierWithoutAnnotation { at })?;
                last.aux = aux;
            }
            50..=58 => return Err(AnnotError::UnknownCode { code: a, at }),
            _ => {
                time = time
                    .checked_add(i64::from(i))
                    .ok_or(AnnotError::BadTime { at })?;
                out.push(Annotation {
                    sample: u64::try_from(time).map_err(|_| AnnotError::BadTime { at })?,
                    code: a,
                    subtype: 0,
                    chan,
                    num,
                    aux: String::new(),
                });
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn w(a: u8, i: u16) -> [u8; 2] {
        let v = (u16::from(a) << 10) | (i & 0x3ff);
        v.to_le_bytes()
    }

    fn build(words: &[&[u8]]) -> Vec<u8> {
        words.iter().flat_map(|x| x.iter().copied()).collect()
    }

    #[test]
    fn reads_beats_rhythm_aux_and_eof() {
        let bytes = build(&[
            &w(code::RHYTHM, 18),
            &w(code::AUX, 3),
            b"(N\0\0", // 3 bytes + pad... "(N" then NUL counted, pad NUL
            &w(code::NORMAL, 100),
            &w(code::PVC, 50),
            &w(code::SUB, 1),
            &w(0, 0),
            &w(code::NORMAL, 1), // after EOF: ignored
        ]);
        let a = parse_annotations(&bytes).unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(a[0].sample, 18);
        assert_eq!(a[0].code, code::RHYTHM);
        assert_eq!(a[0].aux, "(N");
        assert_eq!(a[1].sample, 118);
        assert_eq!(a[2].sample, 168);
        assert_eq!(a[2].subtype, 1);
        assert_eq!(a[1].subtype, 0);
    }

    #[test]
    fn skip_num_chn_and_negative_subtype() {
        // SKIP of 70000 samples: hi word 0x0001, lo word 0x1170.
        let bytes = build(&[
            &w(code::SKIP, 0),
            &1_u16.to_le_bytes(),
            &0x1170_u16.to_le_bytes(),
            &w(code::NOISE, 5),
            &w(code::SUB, 0x3ff),
            &w(code::CHN, 1),
            &w(code::NUM, 2),
            &w(code::NORMAL, 10),
        ]);
        let a = parse_annotations(&bytes).unwrap();
        assert_eq!(a[0].sample, 70_005);
        assert_eq!(a[0].subtype, -1);
        assert_eq!(a[0].chan, 1);
        assert_eq!(a[0].num, 2);
        assert_eq!(a[1].sample, 70_015);
        assert_eq!(a[1].chan, 1, "chan persists");
        assert_eq!(a[1].num, 2, "num persists");
    }

    #[test]
    fn odd_aux_with_pad_and_nonprintable_bytes() {
        let bytes = build(&[&w(code::RHYTHM, 1), &w(code::AUX, 5), b"(VF\x01\x00\x00", &w(code::NORMAL, 1)]);
        let a = parse_annotations(&bytes).unwrap();
        assert_eq!(a[0].aux, "(VF?");
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn hostile_files_are_errors_not_panics() {
        // Truncated word.
        assert!(parse_annotations(&[1]).is_err());
        // Truncated SKIP.
        assert!(parse_annotations(&build(&[&w(code::SKIP, 0), &[0, 0]])).is_err());
        // AUX longer than the file.
        assert!(parse_annotations(&build(&[&w(code::NORMAL, 1), &w(code::AUX, 100), b"ab"])).is_err());
        // AUX before any annotation.
        assert!(parse_annotations(&build(&[&w(code::AUX, 2), b"ab"])).is_err());
        // SUB before any annotation.
        assert!(parse_annotations(&w(code::SUB, 1)).is_err());
        // SKIP to negative time.
        assert!(parse_annotations(&build(&[
            &w(code::SKIP, 0),
            &0xffff_u16.to_le_bytes(),
            &0xff00_u16.to_le_bytes()
        ]))
        .is_err());
        // Undefined code.
        assert!(parse_annotations(&w(55, 1)).is_err());
        // Empty file is fine and empty.
        assert_eq!(parse_annotations(&[]).unwrap(), vec![]);
        // Random bytes never panic.
        let mut seed = 12345_u32;
        for _ in 0..2000 {
            let len = (seed % 64) as usize;
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
                v.push((seed >> 16) as u8);
            }
            let _ = parse_annotations(&v);
        }
    }

    #[test]
    fn mnemonics_cover_the_codes_used() {
        assert_eq!(code::mnemonic(code::VFON), "[");
        assert_eq!(code::mnemonic(code::VFOFF), "]");
        assert_eq!(code::mnemonic(code::FLWAV), "!");
        assert_eq!(code::mnemonic(code::NOISE), "~");
        assert_eq!(code::mnemonic(code::ARFCT), "|");
        assert!(code::is_beat(code::PVC));
        assert!(!code::is_beat(code::RHYTHM));
    }
}
