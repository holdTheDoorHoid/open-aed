// SPDX-License-Identifier: GPL-3.0-or-later
//! A small DEFLATE decoder (RFC 1951) with the zlib wrapper (RFC 1950),
//! written here so that the PDF reader needs no outside library.
//!
//! The input is untrusted: every read is bounds-checked, Huffman tables are
//! validated, back-references may not point before the start of the output,
//! and the output is capped at a caller-given size. Any problem is an
//! error, never a panic.

/// Why decompression failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflateError {
    /// The zlib header is not valid (method, window size or check bits).
    BadZlibHeader,
    /// A preset dictionary is required (not supported, and not used by PDF).
    DictionaryRequired,
    /// The data ended in the middle of a block.
    Truncated,
    /// A block type of 3, a bad stored-block length, or a bad code.
    Corrupt(&'static str),
    /// The output would exceed the cap.
    TooLarge,
    /// The Adler-32 checksum at the end does not match.
    ChecksumMismatch,
}

impl core::fmt::Display for InflateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InflateError::BadZlibHeader => write!(f, "bad zlib header"),
            InflateError::DictionaryRequired => write!(f, "zlib preset dictionary not supported"),
            InflateError::Truncated => write!(f, "compressed data truncated"),
            InflateError::Corrupt(w) => write!(f, "corrupt compressed data: {w}"),
            InflateError::TooLarge => write!(f, "decompressed data larger than the limit"),
            InflateError::ChecksumMismatch => write!(f, "zlib checksum mismatch"),
        }
    }
}

impl std::error::Error for InflateError {}

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u32,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bit: 0,
        }
    }

    fn get(&mut self, n: u32) -> Result<u32, InflateError> {
        let mut v = 0_u32;
        for i in 0..n {
            let byte = *self.data.get(self.pos).ok_or(InflateError::Truncated)?;
            let b = (u32::from(byte) >> self.bit) & 1;
            v |= b << i;
            self.bit += 1;
            if self.bit == 8 {
                self.bit = 0;
                self.pos += 1;
            }
        }
        Ok(v)
    }

    fn align(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.pos += 1;
        }
    }
}

/// A canonical Huffman decoding table: counts per length and symbols in order.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Self, InflateError> {
        let mut counts = [0_u16; 16];
        for &l in lengths {
            let slot = counts
                .get_mut(usize::from(l))
                .ok_or(InflateError::Corrupt("code length"))?;
            *slot += 1;
        }
        counts[0] = 0;
        // Over-subscribed sets are invalid; incomplete ones are allowed (RFC 1951 permits one-code sets).
        let mut left = 1_i32;
        for c in counts.iter().skip(1) {
            left = left * 2 - i32::from(*c);
            if left < 0 {
                return Err(InflateError::Corrupt("over-subscribed code"));
            }
        }
        let mut offs = [0_u16; 16];
        let mut acc = 0_u16;
        for (o, c) in offs.iter_mut().zip(counts.iter()) {
            *o = acc;
            acc += *c;
        }
        let mut symbols = vec![0_u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                let o = offs
                    .get_mut(usize::from(l))
                    .ok_or(InflateError::Corrupt("code length"))?;
                let slot = symbols
                    .get_mut(usize::from(*o))
                    .ok_or(InflateError::Corrupt("table"))?;
                *slot = sym as u16;
                *o += 1;
            }
        }
        Ok(Self { counts, symbols })
    }

    fn decode(&self, b: &mut Bits<'_>) -> Result<u16, InflateError> {
        let mut code = 0_i32;
        let mut first = 0_i32;
        let mut index = 0_i32;
        for &c in self.counts.iter().skip(1) {
            code |= b.get(1)? as i32;
            let count = i32::from(c);
            if code - count < first {
                return self
                    .symbols
                    .get((index + (code - first)) as usize)
                    .copied()
                    .ok_or(InflateError::Corrupt("symbol"));
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err(InflateError::Corrupt("no code matches"))
    }
}

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn codes(
    b: &mut Bits<'_>,
    out: &mut Vec<u8>,
    lit: &Huffman,
    dist: &Huffman,
    cap: usize,
) -> Result<(), InflateError> {
    loop {
        let sym = lit.decode(b)?;
        if sym < 256 {
            if out.len() >= cap {
                return Err(InflateError::TooLarge);
            }
            out.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            let i = usize::from(sym - 257);
            let base = *LEN_BASE
                .get(i)
                .ok_or(InflateError::Corrupt("length code"))?;
            let extra = *LEN_EXTRA
                .get(i)
                .ok_or(InflateError::Corrupt("length code"))?;
            let len = usize::from(base) + b.get(u32::from(extra))? as usize;
            let d = usize::from(dist.decode(b)?);
            let dbase = *DIST_BASE
                .get(d)
                .ok_or(InflateError::Corrupt("distance code"))?;
            let dextra = *DIST_EXTRA
                .get(d)
                .ok_or(InflateError::Corrupt("distance code"))?;
            let distance = usize::from(dbase) + b.get(u32::from(dextra))? as usize;
            if distance > out.len() {
                return Err(InflateError::Corrupt("distance before start"));
            }
            if out.len() + len > cap {
                return Err(InflateError::TooLarge);
            }
            let start = out.len() - distance;
            for k in 0..len {
                let v = *out.get(start + k).ok_or(InflateError::Corrupt("copy"))?;
                out.push(v);
            }
        }
    }
}

fn fixed_tables() -> Result<(Huffman, Huffman), InflateError> {
    let mut l = [0_u8; 288];
    for (i, v) in l.iter_mut().enumerate() {
        *v = match i {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }
    Ok((Huffman::new(&l)?, Huffman::new(&[5_u8; 30])?))
}

fn dynamic_tables(b: &mut Bits<'_>) -> Result<(Huffman, Huffman), InflateError> {
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let nlen = b.get(5)? as usize + 257;
    let ndist = b.get(5)? as usize + 1;
    let ncode = b.get(4)? as usize + 4;
    if nlen > 286 || ndist > 30 {
        return Err(InflateError::Corrupt("table sizes"));
    }
    let mut cl = [0_u8; 19];
    for &o in ORDER.iter().take(ncode) {
        if let Some(slot) = cl.get_mut(o) {
            *slot = b.get(3)? as u8;
        }
    }
    let clh = Huffman::new(&cl)?;
    let mut lengths = vec![0_u8; nlen + ndist];
    let mut i = 0;
    while i < nlen + ndist {
        let sym = clh.decode(b)?;
        let (value, repeat) = match sym {
            0..=15 => (sym as u8, 1),
            16 => {
                let prev = *lengths
                    .get(i.wrapping_sub(1))
                    .ok_or(InflateError::Corrupt("repeat at start"))?;
                (prev, 3 + b.get(2)? as usize)
            }
            17 => (0, 3 + b.get(3)? as usize),
            18 => (0, 11 + b.get(7)? as usize),
            _ => return Err(InflateError::Corrupt("code-length symbol")),
        };
        if i + repeat > nlen + ndist {
            return Err(InflateError::Corrupt("repeat past end"));
        }
        for v in lengths.iter_mut().skip(i).take(repeat) {
            *v = value;
        }
        i += repeat;
    }
    if lengths.get(256).copied().unwrap_or(0) == 0 {
        return Err(InflateError::Corrupt("no end-of-block code"));
    }
    let (l, d) = lengths.split_at(nlen);
    Ok((Huffman::new(l)?, Huffman::new(d)?))
}

/// Decompress raw DEFLATE data, producing at most `cap` bytes.
///
/// # Errors
/// [`InflateError`] for anything malformed or too large.
pub fn inflate_raw(data: &[u8], cap: usize) -> Result<(Vec<u8>, usize), InflateError> {
    let mut b = Bits::new(data);
    let mut out = Vec::new();
    loop {
        let last = b.get(1)?;
        match b.get(2)? {
            0 => {
                b.align();
                let p = b.pos;
                let Some(&[l0, l1, n0, n1]) = data.get(p..p + 4) else {
                    return Err(InflateError::Truncated);
                };
                let len = u16::from_le_bytes([l0, l1]);
                let nlen = u16::from_le_bytes([n0, n1]);
                if len != !nlen {
                    return Err(InflateError::Corrupt("stored length"));
                }
                let body = data
                    .get(p + 4..p + 4 + usize::from(len))
                    .ok_or(InflateError::Truncated)?;
                if out.len() + body.len() > cap {
                    return Err(InflateError::TooLarge);
                }
                out.extend_from_slice(body);
                b.pos = p + 4 + usize::from(len);
            }
            1 => {
                let (l, d) = fixed_tables()?;
                codes(&mut b, &mut out, &l, &d, cap)?;
            }
            2 => {
                let (l, d) = dynamic_tables(&mut b)?;
                codes(&mut b, &mut out, &l, &d, cap)?;
            }
            _ => return Err(InflateError::Corrupt("block type 3")),
        }
        if last == 1 {
            b.align();
            return Ok((out, b.pos));
        }
    }
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u32, 0_u32);
    for chunk in data.chunks(5552) {
        for &x in chunk {
            a += u32::from(x);
            b += a;
        }
        a %= 65_521;
        b %= 65_521;
    }
    (b << 16) | a
}

/// Decompress zlib-wrapped data (what PDF's `FlateDecode` holds), producing
/// at most `cap` bytes, and check the Adler-32 checksum (required).
///
/// # Errors
/// [`InflateError`] for a bad header, corrupt data, a wrong checksum or too
/// much output.
pub fn inflate_zlib(data: &[u8], cap: usize) -> Result<Vec<u8>, InflateError> {
    let (&cmf, &flg) = match data {
        [a, b, ..] => (a, b),
        _ => return Err(InflateError::Truncated),
    };
    if cmf & 0x0f != 8 || cmf >> 4 > 7 || (u16::from(cmf) * 256 + u16::from(flg)) % 31 != 0 {
        return Err(InflateError::BadZlibHeader);
    }
    if flg & 0x20 != 0 {
        return Err(InflateError::DictionaryRequired);
    }
    let body = data.get(2..).ok_or(InflateError::Truncated)?;
    let (out, used) = inflate_raw(body, cap)?;
    let Some(&[s0, s1, s2, s3]) = body.get(used..used + 4) else {
        return Err(InflateError::Truncated);
    };
    let want = u32::from_be_bytes([s0, s1, s2, s3]);
    if want != adler32(&out) {
        return Err(InflateError::ChecksumMismatch);
    }
    Ok(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    // zlib.compress(b"hello hello hello hello", 9): one fixed-Huffman block with a back-reference.
    const HELLO: [u8; 16] = [
        0x78, 0xda, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0xc8, 0x40, 0x27, 0x01, 0x68, 0x03, 0x08,
        0xb1,
    ];

    // zlib.compress(text, 9) where text is "{i*i % 97} {i*7 % 13} l\n" for i in 0..20:
    // one dynamic-Huffman block (generated with Python's zlib).
    const DYNAMIC: [u8; 83] = [
        0x78, 0xda, 0x1d, 0x8c, 0xb9, 0x11, 0xc0, 0x40, 0x0c, 0x02, 0x73, 0x57, 0x41, 0x09, 0xa7,
        0xe7, 0x24, 0xab, 0x27, 0xf7, 0x1f, 0x1b, 0x91, 0xb0, 0x33, 0x0c, 0xec, 0xc1, 0xc1, 0xf7,
        0x18, 0x9a, 0x99, 0x30, 0xe6, 0xe0, 0xdd, 0xa6, 0xe0, 0x84, 0x5f, 0x0c, 0x11, 0x85, 0xd8,
        0xc1, 0xc0, 0x76, 0x5e, 0x89, 0x24, 0x5e, 0x83, 0xed, 0x23, 0x70, 0x77, 0xca, 0xfb, 0x5e,
        0xb2, 0x51, 0x44, 0xbb, 0xcc, 0x2e, 0x73, 0x98, 0xd4, 0xe5, 0x72, 0xcf, 0x95, 0x3b, 0x42,
        0xee, 0x3e, 0x72, 0xff, 0xef, 0x3e, 0x19, 0xc5,
    ];

    #[test]
    fn dynamic_huffman_block() {
        let want: String = (0..20_u32)
            .map(|i| format!("{} {} l\n", i * i % 97, i * 7 % 13))
            .collect();
        assert_eq!(inflate_zlib(&DYNAMIC, 1000).unwrap(), want.as_bytes());
    }

    #[test]
    fn stored_block() {
        // zlib header, final stored block of "abc", Adler-32 of "abc".
        let mut d = vec![0x78, 0x01, 0x01, 0x03, 0x00, 0xfc, 0xff, b'a', b'b', b'c'];
        d.extend_from_slice(&adler32(b"abc").to_be_bytes());
        assert_eq!(inflate_zlib(&d, 100).unwrap(), b"abc");
    }

    #[test]
    fn fixed_huffman_with_backreference() {
        assert_eq!(
            inflate_zlib(&HELLO, 100).unwrap(),
            b"hello hello hello hello"
        );
        let mut bad = HELLO;
        bad[15] ^= 1;
        assert_eq!(inflate_zlib(&bad, 100), Err(InflateError::ChecksumMismatch));
    }

    #[test]
    fn rejects_bad_input_without_panicking() {
        assert_eq!(inflate_zlib(&[], 10), Err(InflateError::Truncated));
        assert_eq!(
            inflate_zlib(&[0x78, 0x00], 10),
            Err(InflateError::BadZlibHeader)
        );
        assert!(inflate_zlib(&HELLO[..8], 100).is_err());
        assert_eq!(
            inflate_raw(&[0x07], 10).map(|_| ()),
            Err(InflateError::Corrupt("block type 3"))
        );
        // Output cap.
        assert_eq!(
            inflate_raw(&HELLO[2..], 5).map(|_| ()),
            Err(InflateError::TooLarge)
        );
        // Every truncation and every single-byte corruption must return, not panic.
        for n in 0..DYNAMIC.len() {
            assert!(inflate_zlib(&DYNAMIC[..n], 1000).is_err());
        }
        for i in 0..DYNAMIC.len() {
            for x in [0x01, 0xff, 0x55] {
                let mut d = DYNAMIC;
                d[i] ^= x;
                let _ = inflate_zlib(&d, 1000);
            }
        }
    }
}
