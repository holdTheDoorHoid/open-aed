// SPDX-License-Identifier: GPL-3.0-or-later
//! Just enough of PDF to read the vector drawings in the Cardially reports.
//!
//! This is not a general PDF reader. It finds the page objects, joins and
//! decompresses each page's content streams, and interprets the drawing
//! operators far enough to return every stroked line (with its colour and
//! position on the page) and every piece of text (with its position). The
//! files are untrusted: sizes are capped, every number is checked, anything
//! not understood is an error, and nothing in a file is ever executed.

use std::collections::BTreeMap;

use super::inflate::{inflate_zlib, InflateError};

/// Largest PDF file accepted, bytes.
pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
/// Largest decompressed content of one page, bytes.
pub const MAX_CONTENT_BYTES: usize = 8 * 1024 * 1024;
/// Most points kept from one page.
pub const MAX_POINTS: usize = 2_000_000;

/// Why a file could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfError {
    /// Not a PDF, or larger than [`MAX_FILE_BYTES`].
    NotPdf(String),
    /// Something the reader does not support or cannot make sense of.
    Unsupported(String),
    /// A compressed stream could not be decompressed.
    Inflate(InflateError),
}

impl core::fmt::Display for PdfError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PdfError::NotPdf(w) => write!(f, "not a readable PDF: {w}"),
            PdfError::Unsupported(w) => write!(f, "unsupported PDF content: {w}"),
            PdfError::Inflate(e) => write!(f, "PDF stream: {e}"),
        }
    }
}

impl std::error::Error for PdfError {}

fn unsupported<T>(w: impl Into<String>) -> Result<T, PdfError> {
    Err(PdfError::Unsupported(w.into()))
}

/// One page: its rotation (degrees) and its decompressed content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// The `/Rotate` value (0 if absent).
    pub rotate: i32,
    /// All content streams of the page, joined with a newline.
    pub content: Vec<u8>,
}

const fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\0')
}

const fn is_delim(b: u8) -> bool {
    matches!(b, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

/// Positions of every `N G obj` in the file: object number -> offset just
/// after `obj`. A later definition of the same number replaces an earlier one
/// (incremental updates).
fn find_objects(file: &[u8]) -> BTreeMap<u32, usize> {
    let mut out = BTreeMap::new();
    let mut i = 0;
    while let Some(off) = find(file, b"obj", i) {
        i = off + 3;
        // Must be followed by a delimiter and preceded by "<num> <gen> ".
        if file.get(off + 3).is_some_and(|b| !is_space(*b) && !is_delim(*b)) {
            continue;
        }
        let mut j = off;
        let back = |j: &mut usize, digits: bool| -> bool {
            let end = *j;
            while *j > 0 && file.get(*j - 1).is_some_and(|b| if digits { b.is_ascii_digit() } else { is_space(*b) }) {
                *j -= 1;
            }
            *j < end
        };
        if !(back(&mut j, false) && back(&mut j, true)) {
            continue;
        }
        if !back(&mut j, false) {
            continue;
        }
        let num_end = j;
        if !back(&mut j, true) {
            continue;
        }
        if j > 0 && file.get(j - 1).is_some_and(|b| !is_space(*b) && !is_delim(*b)) {
            continue;
        }
        if let Some(n) = file.get(j..num_end).and_then(|s| std::str::from_utf8(s).ok()).and_then(|s| s.parse().ok()) {
            out.insert(n, off + 3);
        }
    }
    out
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from >= hay.len() {
        return None;
    }
    hay.get(from..)?.windows(needle.len()).position(|w| w == needle).map(|p| p + from)
}

/// The dictionary text of the object starting at `pos` (up to `stream` or
/// `endobj`), and the offset of the word `stream` if the object has one.
fn object_dict(file: &[u8], pos: usize) -> Result<(&[u8], Option<usize>), PdfError> {
    let end = find(file, b"endobj", pos).unwrap_or(file.len());
    let st = find(file, b"stream", pos).filter(|s| *s < end);
    let dict_end = st.unwrap_or(end);
    let dict = file.get(pos..dict_end).ok_or_else(|| PdfError::NotPdf("object out of range".into()))?;
    Ok((dict, st))
}

/// Value text after `/Key` in a dictionary (up to the next `/`, `>>` or end).
fn dict_value<'a>(dict: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let mut i = 0;
    while let Some(p) = find(dict, key, i) {
        i = p + key.len();
        let next = dict.get(i).copied();
        if next.is_some_and(|b| !is_space(b) && !is_delim(b)) {
            continue; // a longer name, e.g. /Pages for /Page
        }
        let rest = dict.get(i..)?;
        let rest = trim_start(rest);
        if rest.first() == Some(&b'[') {
            let close = rest.iter().position(|b| *b == b']')?;
            return rest.get(..=close);
        }
        if rest.first() == Some(&b'/') {
            let end = rest.iter().skip(1).position(|b| is_space(*b) || is_delim(*b)).map_or(rest.len(), |p| p + 1);
            return rest.get(..end);
        }
        let end = rest.iter().position(|b| *b == b'/' || *b == b'>').unwrap_or(rest.len());
        return rest.get(..end);
    }
    None
}

fn trim_start(s: &[u8]) -> &[u8] {
    let n = s.iter().take_while(|b| is_space(**b)).count();
    s.get(n..).unwrap_or(&[])
}

/// The numbers `N` of every `N G R` reference in `text`.
fn references(text: &[u8]) -> Vec<u32> {
    let s = String::from_utf8_lossy(text);
    let words: Vec<&str> = s.split(|c: char| c.is_whitespace() || c == '[' || c == ']').filter(|w| !w.is_empty()).collect();
    let mut out = Vec::new();
    for w in words.windows(3) {
        if let [n, g, r] = w {
            if *r == "R" && g.parse::<u32>().is_ok() {
                if let Ok(n) = n.parse() {
                    out.push(n);
                }
            }
        }
    }
    out
}

fn integer(text: &[u8]) -> Option<i64> {
    std::str::from_utf8(text).ok()?.split_whitespace().next()?.parse().ok()
}

/// The decoded data of stream object `num`.
fn stream_data(file: &[u8], objs: &BTreeMap<u32, usize>, num: u32) -> Result<Vec<u8>, PdfError> {
    let pos = *objs.get(&num).ok_or_else(|| PdfError::NotPdf(format!("object {num} not found")))?;
    let (dict, st) = object_dict(file, pos)?;
    let st = st.ok_or_else(|| PdfError::NotPdf(format!("object {num} is not a stream")))?;
    let mut start = st + "stream".len();
    if file.get(start) == Some(&b'\r') {
        start += 1;
    }
    if file.get(start) == Some(&b'\n') {
        start += 1;
    }
    let len = match dict_value(dict, b"/Length") {
        Some(v) => {
            let refs = references(v);
            if let Some(r) = refs.first() {
                let p = *objs.get(r).ok_or_else(|| PdfError::NotPdf("length object missing".into()))?;
                let (d, _) = object_dict(file, p)?;
                integer(d)
            } else {
                integer(v)
            }
        }
        None => None,
    };
    let raw = match len.and_then(|l| usize::try_from(l).ok()) {
        Some(l) if file.get(start..start + l).is_some() => file.get(start..start + l).unwrap_or(&[]),
        _ => {
            let end = find(file, b"endstream", start).ok_or_else(|| PdfError::NotPdf("stream without end".into()))?;
            file.get(start..end).unwrap_or(&[])
        }
    };
    match dict_value(dict, b"/Filter") {
        None => Ok(raw.to_vec()),
        Some(f) if f.starts_with(b"/FlateDecode") || f == b"[/FlateDecode]" => {
            if dict_value(dict, b"/DecodeParms").is_some() {
                return unsupported("predictor on a content stream");
            }
            inflate_zlib(raw, MAX_CONTENT_BYTES).map_err(PdfError::Inflate)
        }
        Some(f) => unsupported(format!("stream filter {}", String::from_utf8_lossy(f))),
    }
}

/// Every page of the file, in the order their objects appear in the file.
///
/// # Errors
/// [`PdfError`] if the file is not a PDF, too large, or uses something this
/// reader does not support (page objects inside compressed object streams,
/// filters other than `FlateDecode`).
pub fn read_pages(file: &[u8]) -> Result<Vec<Page>, PdfError> {
    if file.len() > MAX_FILE_BYTES {
        return Err(PdfError::NotPdf("file too large".into()));
    }
    if !file.starts_with(b"%PDF-") {
        return Err(PdfError::NotPdf("no %PDF- header".into()));
    }
    let objs = find_objects(file);
    let mut by_pos: Vec<(usize, u32)> = objs.iter().map(|(n, p)| (*p, *n)).collect();
    by_pos.sort_unstable();
    let mut pages = Vec::new();
    for (pos, _num) in by_pos {
        let (dict, st) = object_dict(file, pos)?;
        if st.is_some() {
            continue;
        }
        let ty = dict_value(dict, b"/Type");
        if ty != Some(b"/Page".as_slice()) {
            continue;
        }
        let contents = dict_value(dict, b"/Contents").ok_or_else(|| PdfError::NotPdf("page without contents".into()))?;
        let mut content = Vec::new();
        for r in references(contents) {
            let data = stream_data(file, &objs, r)?;
            if content.len() + data.len() + 1 > MAX_CONTENT_BYTES {
                return unsupported("page content too large");
            }
            content.extend_from_slice(&data);
            content.push(b'\n');
        }
        let rotate = dict_value(dict, b"/Rotate").and_then(integer).unwrap_or(0);
        let rotate = i32::try_from(rotate.rem_euclid(360)).unwrap_or(0);
        pages.push(Page { rotate, content });
    }
    if pages.is_empty() {
        return unsupported("no page objects outside object streams");
    }
    Ok(pages)
}

// ---------------------------------------------------------------------------
// Content streams
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Name(Vec<u8>),
    Str(Vec<u8>),
    ArrOpen,
    ArrClose,
    Op(Vec<u8>),
}

fn tokenize(src: &[u8]) -> Result<Vec<Tok>, PdfError> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(&b) = src.get(i) {
        if is_space(b) {
            i += 1;
        } else if b == b'%' {
            while src.get(i).is_some_and(|c| *c != b'\n' && *c != b'\r') {
                i += 1;
            }
        } else if b == b'[' {
            out.push(Tok::ArrOpen);
            i += 1;
        } else if b == b']' {
            out.push(Tok::ArrClose);
            i += 1;
        } else if b == b'(' {
            let mut depth = 1;
            let mut s = Vec::new();
            i += 1;
            loop {
                let c = *src.get(i).ok_or_else(|| PdfError::NotPdf("unterminated string".into()))?;
                i += 1;
                match c {
                    b'\\' => {
                        let e = *src.get(i).ok_or_else(|| PdfError::NotPdf("unterminated string".into()))?;
                        i += 1;
                        match e {
                            b'n' => s.push(b'\n'),
                            b'r' => s.push(b'\r'),
                            b't' => s.push(b'\t'),
                            b'b' => s.push(8),
                            b'f' => s.push(12),
                            b'0'..=b'7' => {
                                let mut v = u32::from(e - b'0');
                                for _ in 0..2 {
                                    match src.get(i) {
                                        Some(d @ b'0'..=b'7') => {
                                            v = v * 8 + u32::from(d - b'0');
                                            i += 1;
                                        }
                                        _ => break,
                                    }
                                }
                                s.push((v & 0xff) as u8);
                            }
                            b'\r' | b'\n' => {}
                            other => s.push(other),
                        }
                    }
                    b'(' => {
                        depth += 1;
                        s.push(c);
                    }
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        s.push(c);
                    }
                    _ => s.push(c),
                }
            }
            out.push(Tok::Str(s));
        } else if b == b'<' {
            if src.get(i + 1) == Some(&b'<') {
                return unsupported("dictionary in content stream");
            }
            let end = find(src, b">", i).ok_or_else(|| PdfError::NotPdf("unterminated hex string".into()))?;
            let hex: Vec<u8> = src.get(i + 1..end).unwrap_or(&[]).iter().copied().filter(|c| !is_space(*c)).collect();
            let mut s = Vec::new();
            for pair in hex.chunks(2) {
                let txt = std::str::from_utf8(pair).map_err(|_| PdfError::NotPdf("bad hex string".into()))?;
                let v = u8::from_str_radix(&format!("{txt:0<2}"), 16).map_err(|_| PdfError::NotPdf("bad hex string".into()))?;
                s.push(v);
            }
            out.push(Tok::Str(s));
            i = end + 1;
        } else if b == b'/' {
            let start = i + 1;
            i += 1;
            while src.get(i).is_some_and(|c| !is_space(*c) && !is_delim(*c)) {
                i += 1;
            }
            out.push(Tok::Name(src.get(start..i).unwrap_or(&[]).to_vec()));
        } else if is_delim(b) {
            return unsupported(format!("unexpected character {:?}", b as char));
        } else {
            let start = i;
            while src.get(i).is_some_and(|c| !is_space(*c) && !is_delim(*c)) {
                i += 1;
            }
            let word = src.get(start..i).unwrap_or(&[]);
            let num = std::str::from_utf8(word).ok().filter(|w| w.bytes().all(|c| c.is_ascii_digit() || c == b'.' || c == b'-' || c == b'+'));
            match num.and_then(|w| w.parse::<f64>().ok()) {
                Some(v) if v.is_finite() => out.push(Tok::Num(v)),
                _ => {
                    if word == b"BI" {
                        return unsupported("inline image");
                    }
                    out.push(Tok::Op(word.to_vec()));
                }
            }
        }
    }
    Ok(out)
}

/// The stroke colour in force when a path was stroked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Colour {
    /// Grey level, 0 = black.
    Grey(f64),
    /// Red, green, blue (0 to 1).
    Rgb(f64, f64, f64),
    /// Anything else (CMYK, patterns).
    Other,
}

impl Colour {
    /// Pure blue: the ECG trace in the Philips reports.
    #[must_use]
    pub fn is_blue(self) -> bool {
        matches!(self, Colour::Rgb(r, g, b) if r < 0.2 && g < 0.2 && b > 0.8)
    }
}

/// One stroked subpath, in page coordinates (points, 1/72 inch).
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    /// Stroke colour.
    pub colour: Colour,
    /// Line width (as set, not transformed).
    pub width: f64,
    /// The vertices.
    pub points: Vec<(f64, f64)>,
}

/// One piece of text and where its first character starts, page coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    /// x.
    pub x: f64,
    /// y.
    pub y: f64,
    /// Font size times the text matrix scale (approximate height in points).
    pub size: f64,
    /// The bytes as Latin-1 text.
    pub text: String,
}

/// What was drawn on one page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Drawing {
    /// Every stroked subpath.
    pub strokes: Vec<Stroke>,
    /// Every text string.
    pub texts: Vec<Text>,
}

type Mat = [f64; 6];
const IDENTITY: Mat = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn mul(m: &Mat, n: &Mat) -> Mat {
    [
        m[0] * n[0] + m[1] * n[2],
        m[0] * n[1] + m[1] * n[3],
        m[2] * n[0] + m[3] * n[2],
        m[2] * n[1] + m[3] * n[3],
        m[4] * n[0] + m[5] * n[2] + n[4],
        m[4] * n[1] + m[5] * n[3] + n[5],
    ]
}

fn apply(m: &Mat, x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

/// The last `K` operands, or an error naming the operator.
fn last<const K: usize>(nums: &[f64], op: &[u8]) -> Result<[f64; K], PdfError> {
    nums.len()
        .checked_sub(K)
        .and_then(|s| nums.get(s..))
        .and_then(|s| <[f64; K]>::try_from(s).ok())
        .ok_or_else(|| PdfError::Unsupported(format!("operator {} needs {K} numbers", String::from_utf8_lossy(op))))
}

fn latin1(b: &[u8]) -> String {
    b.iter().map(|c| char::from(*c)).collect()
}

/// Interpret a page's content: return its stroked lines and its text.
///
/// # Errors
/// [`PdfError`] for malformed content, a graphics-state stack imbalance
/// beyond 64 levels, or more than [`MAX_POINTS`] points.
pub fn interpret(content: &[u8]) -> Result<Drawing, PdfError> {
    let toks = tokenize(content)?;
    let mut d = Drawing::default();
    let mut ctm = IDENTITY;
    let mut colour = Colour::Grey(0.0);
    let mut width = 1.0;
    let mut stack: Vec<(Mat, Colour, f64)> = Vec::new();
    let mut path: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut tm = IDENTITY;
    let mut tlm = IDENTITY;
    let mut font_size = 1.0;
    let mut nums: Vec<f64> = Vec::new();
    let mut array: Option<Vec<Vec<u8>>> = None;
    let mut last_str: Option<Vec<u8>> = None;
    let mut points = 0_usize;
    let mut last_name: Option<Vec<u8>> = None;
    for t in toks {
        let op = match t {
            Tok::Num(v) => {
                if nums.len() >= 64 {
                    return unsupported("too many operands");
                }
                nums.push(v);
                continue;
            }
            Tok::Name(n) => {
                last_name = Some(n);
                continue;
            }
            Tok::Str(s) => {
                if let Some(a) = array.as_mut() {
                    a.push(s);
                } else {
                    last_str = Some(s);
                }
                continue;
            }
            Tok::ArrOpen => {
                array = Some(Vec::new());
                continue;
            }
            Tok::ArrClose => {
                continue;
            }
            Tok::Op(o) => o,
        };

        match op.as_slice() {
            b"q" => {
                if stack.len() >= 64 {
                    return unsupported("graphics state nested too deep");
                }
                stack.push((ctm, colour, width));
            }
            b"Q" => {
                if let Some((m, c, w)) = stack.pop() {
                    ctm = m;
                    colour = c;
                    width = w;
                }
            }
            b"cm" => {
                let m: Mat = last(&nums, &op)?;
                ctm = mul(&m, &ctm);
            }
            b"w" => [width] = last(&nums, &op)?,
            b"G" => {
                let [g] = last(&nums, &op)?;
                colour = Colour::Grey(g);
            }
            b"RG" => {
                let [r, g, b] = last(&nums, &op)?;
                colour = Colour::Rgb(r, g, b);
            }
            b"K" => colour = Colour::Other,
            b"SC" | b"SCN" => {
                colour = match nums.as_slice() {
                    [g] => Colour::Grey(*g),
                    [r, g, b] => Colour::Rgb(*r, *g, *b),
                    _ => Colour::Other,
                };
            }
            b"CS" => colour = Colour::Other,
            b"m" => {
                let [x, y] = last(&nums, &op)?;
                path.push(vec![apply(&ctm, x, y)]);
                points += 1;
            }
            b"l" | b"c" | b"v" | b"y" => {
                let [x, y] = last(&nums, &op).map_err(|_| PdfError::Unsupported("path operator without coordinates".into()))?;
                let p = apply(&ctm, x, y);
                match path.last_mut() {
                    Some(sp) => sp.push(p),
                    None => return unsupported("line before move"),
                }
                points += 1;
            }
            b"h" => {
                if let Some(sp) = path.last_mut() {
                    if let Some(first) = sp.first().copied() {
                        sp.push(first);
                    }
                }
            }
            b"re" => {
                let [x, y, w, h] = last(&nums, &op)?;
                path.push(vec![
                    apply(&ctm, x, y),
                    apply(&ctm, x + w, y),
                    apply(&ctm, x + w, y + h),
                    apply(&ctm, x, y + h),
                    apply(&ctm, x, y),
                ]);
                points += 5;
            }
            b"S" | b"s" | b"B" | b"B*" | b"b" | b"b*" => {
                for mut sp in path.drain(..) {
                    if matches!(op.as_slice(), b"s" | b"b" | b"b*") {
                        if let Some(first) = sp.first().copied() {
                            sp.push(first);
                        }
                    }
                    d.strokes.push(Stroke { colour, width, points: sp });
                }
            }
            b"f" | b"F" | b"f*" | b"n" => path.clear(),
            b"W" | b"W*" | b"g" | b"rg" | b"k" | b"cs" | b"sc" | b"scn" | b"i" | b"J" | b"j" | b"M" | b"d"
            | b"gs" | b"ri" | b"Tc" | b"Tw" | b"Tz" | b"TL" | b"Tr" | b"Ts" | b"Do" | b"sh" | b"d0" | b"d1"
            | b"BX" | b"EX" | b"MP" | b"DP" | b"BMC" | b"BDC" | b"EMC" => {}
            b"BT" => {
                tm = IDENTITY;
                tlm = IDENTITY;
            }
            b"ET" => {}
            b"Tf" => [font_size] = last(&nums, &op)?,
            b"Tm" => {
                tm = last(&nums, &op)?;
                tlm = tm;
            }
            b"Td" | b"TD" => {
                let [tx, ty] = last(&nums, &op)?;
                tlm = mul(&[1.0, 0.0, 0.0, 1.0, tx, ty], &tlm);
                tm = tlm;
            }
            b"T*" => {}
            b"Tj" | b"'" | b"\"" | b"TJ" => {
                let bytes: Vec<u8> = if op.as_slice() == b"TJ" {
                    array.take().unwrap_or_default().concat()
                } else {
                    last_str.take().unwrap_or_default()
                };
                let m = mul(&tm, &ctm);
                let size = font_size * (m[0] * m[0] + m[1] * m[1]).sqrt();
                d.texts.push(Text { x: m[4], y: m[5], size, text: latin1(&bytes) });
            }
            other => {
                return unsupported(format!("operator {}", String::from_utf8_lossy(other)));
            }
        }
        if points > MAX_POINTS {
            return unsupported("too many points");
        }
        nums.clear();
        if op.as_slice() != b"TJ" {
            array = None;
        }
        let _ = last_name.take();
    }
    Ok(d)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_handles_strings_names_and_numbers() {
        let t = tokenize(b"/Cs6 CS 0 0 1 SCN (a\\(b\\)c \\101) [(x)-3.5(y)] TJ <4142> -.5 % comment\n q").unwrap();
        assert_eq!(t[0], Tok::Name(b"Cs6".to_vec()));
        assert_eq!(t[2], Tok::Num(0.0));
        assert_eq!(t[6], Tok::Str(b"a(b)c A".to_vec()));
        assert!(t.contains(&Tok::Str(b"AB".to_vec())));
        assert!(t.contains(&Tok::Num(-0.5)));
        assert_eq!(t.last(), Some(&Tok::Op(b"q".to_vec())));
        assert!(tokenize(b"(never closed").is_err());
        assert!(tokenize(b"<< /A 1 >>").is_err());
    }

    #[test]
    fn interprets_a_hand_built_page() {
        let content = b"0 G .565 w q 1 0 0 1 10 20 cm 0 0 m 0 100 l 50 100 l 50 0 l 0 0 l S Q \
            /Cs6 CS 0 0 1 SCN q 1 0 0 1 30 20 cm 0 0 m 1 .5 l -2 1 l S Q \
            BT 0 6 -6 0 70 20 Tm (12.29.22)Tj 10 0 TD [(Sh)5(ock 1 delivered)]TJ ET";
        let d = interpret(content).unwrap();
        assert_eq!(d.strokes.len(), 2);
        assert_eq!(d.strokes[0].colour, Colour::Grey(0.0));
        assert_eq!(d.strokes[0].points[2], (60.0, 120.0));
        assert!(d.strokes[1].colour.is_blue());
        assert_eq!(d.strokes[1].points, vec![(30.0, 20.0), (31.0, 20.5), (28.0, 21.0)]);
        assert_eq!(d.texts.len(), 2);
        assert_eq!(d.texts[0].text, "12.29.22");
        assert!((d.texts[0].x - 70.0).abs() < 1e-9 && (d.texts[0].y - 20.0).abs() < 1e-9);
        assert!((d.texts[0].size - 6.0).abs() < 1e-9);
        // TD moves 10 text units along the rotated baseline: 60 points up the page.
        assert_eq!(d.texts[1].text, "Shock 1 delivered");
        assert!((d.texts[1].y - 80.0).abs() < 1e-9, "{}", d.texts[1].y);
    }

    #[test]
    fn interpreter_rejects_what_it_does_not_understand() {
        assert!(interpret(b"1 2 l").is_err()); // line before move
        assert!(interpret(b"1 2 3 cm").is_err()); // too few operands
        assert!(interpret(b"0 0 m 5 5 l frobnicate").is_err());
        assert!(interpret(b"BI /W 1 ID x EI").is_err());
    }

    fn build_pdf(content: &[u8]) -> Vec<u8> {
        let mut z = vec![0x78, 0x01, 0x01];
        let len = content.len() as u16;
        z.extend_from_slice(&len.to_le_bytes());
        z.extend_from_slice(&(!len).to_le_bytes());
        z.extend_from_slice(content);
        let (mut a, mut b) = (1_u32, 0_u32);
        for c in content {
            a = (a + u32::from(*c)) % 65_521;
            b = (b + a) % 65_521;
        }
        z.extend_from_slice(&((b << 16) | a).to_be_bytes());
        let mut f = b"%PDF-1.5\n1 0 obj\n<</Type/Pages/Kids[2 0 R]/Count 1>>\nendobj\n".to_vec();
        f.extend_from_slice(b"2 0 obj\n<</Contents 3 0 R/Rotate 90/Type/Page/Parent 1 0 R>>\nendobj\n");
        f.extend_from_slice(format!("3 0 obj\n<</Filter/FlateDecode/Length {}>>stream\r\n", z.len()).as_bytes());
        f.extend_from_slice(&z);
        f.extend_from_slice(b"\nendstream\nendobj\ntrailer\n%%EOF\n");
        f
    }

    #[test]
    fn reads_pages_of_a_hand_built_file() {
        let f = build_pdf(b"0 0 m 1 1 l S");
        let pages = read_pages(&f).unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].rotate, 90);
        assert_eq!(pages[0].content, b"0 0 m 1 1 l S\n");
        assert!(read_pages(b"not a pdf").is_err());
        // Truncated anywhere: an error or fewer pages, never a panic.
        for n in 0..f.len() {
            let _ = read_pages(&f[..n]);
        }
    }

    #[test]
    fn dictionary_lookup_does_not_confuse_page_and_pages() {
        assert_eq!(dict_value(b"<</Type/Pages/Count 1>>", b"/Type"), Some(b"/Pages".as_slice()));
        assert_eq!(dict_value(b"<</Type /Page/Rotate 90>>", b"/Type"), Some(b"/Page".as_slice()));
        assert_eq!(dict_value(b"<</Contents[18 0 R 19 0 R]>>", b"/Contents"), Some(b"[18 0 R 19 0 R]".as_slice()));
        assert_eq!(references(b"[18 0 R 19 0 R]"), vec![18, 19]);
    }
}
