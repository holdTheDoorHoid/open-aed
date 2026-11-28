// SPDX-License-Identifier: GPL-3.0-or-later
//! The patient-wise split of the recordings into a **development** half
//! (used for all tuning) and a **held-out** half (run once, with frozen
//! settings, at the end).
//!
//! The rule is fixed and mechanical, so anyone can check it:
//!
//! 1. **Patients, not records, are split.** MIT-BIH Arrhythmia records 201
//!    and 202 come from the same man (stated in the database's own
//!    documentation, `mitdbdir/intro.htm`), so they form one unit. No other
//!    same-patient records are documented in the files that come with the
//!    three databases; every other record is its own unit.
//! 2. **Strata.** Units are grouped by database and by what their
//!    annotations contain: annotated ventricular fibrillation (`(VF`,
//!    `(VFIB`, or cudb's `[`…`]` marks), ventricular flutter only (`(VFL`),
//!    or neither. This makes both halves get VF patients from each database
//!    that has them.
//! 3. **Shuffle.** Strata are taken in a fixed order (cudb, vfdb, mitdb;
//!    within each: VF, flutter only, neither). Within a stratum the units
//!    are sorted by name and shuffled (Fisher–Yates) with one xorshift32
//!    generator seeded with [`SPLIT_SEED`] and carried from stratum to
//!    stratum.
//! 4. **Halves.** The first ⌈n/2⌉ units of each shuffled stratum go to
//!    development, the rest to held-out.
//!
//! The seed was fixed before the split was made and the split was made
//! once. Nothing about the analyser's results was used: only the annotation
//! contents above.

use std::collections::BTreeMap;
use std::path::Path;

use crate::truth::{build_timeline, Db, Label};
use crate::wfdb::{load_record, read_records_list, LoadError};

/// The seed of the split's shuffle (the date the split was made, 2026-11-27).
pub const SPLIT_SEED: u32 = 20_261_127;

/// Records known to come from the same patient (documented in the databases).
pub const SAME_PATIENT: &[(&str, &[&str])] = &[("mitdb", &["201", "202"])];

/// What a record's annotations contain, for stratification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Content {
    /// Annotated ventricular fibrillation.
    Vf,
    /// Ventricular flutter, but no fibrillation.
    FlutterOnly,
    /// Neither.
    Neither,
}

impl Content {
    /// Short text.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Content::Vf => "VF",
            Content::FlutterOnly => "flutter only",
            Content::Neither => "no VF or flutter",
        }
    }
}

/// One record's identity and content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordInfo {
    /// Database.
    pub db: Db,
    /// Record name.
    pub name: String,
    /// What its annotations contain.
    pub content: Content,
}

/// The two halves, as `db/record` names in a fixed order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Split {
    /// Used for all tuning.
    pub development: Vec<String>,
    /// Run once at the end.
    pub held_out: Vec<String>,
}

/// xorshift32 (Marsaglia 2003), as in `aed_rhythm::synth`.
#[derive(Debug, Clone, Copy)]
struct Xorshift32(u32);

impl Xorshift32 {
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}

/// The content of one record, from its annotations.
#[must_use]
pub fn content_of(labels: &[Label]) -> Content {
    if labels.contains(&Label::Vf) {
        Content::Vf
    } else if labels.contains(&Label::Vfl) {
        Content::FlutterOnly
    } else {
        Content::Neither
    }
}

/// Read every record's annotations under `data_dir` and say what each contains.
///
/// # Errors
/// [`LoadError`] if a RECORDS list or a record cannot be read.
pub fn survey(data_dir: &Path) -> Result<Vec<RecordInfo>, LoadError> {
    let mut out = Vec::new();
    for db in Db::ALL {
        let dir = data_dir.join(db.name());
        for name in read_records_list(&dir)? {
            let rec = load_record(&dir, &name, "atr")?;
            let n = rec.samples.first().map_or(0, |s| s.len() as u64);
            let tl = build_timeline(
                db,
                &rec.annotations,
                rec.header.record.sampling_hz,
                n,
                rec.samples.len(),
            );
            let labels: Vec<Label> = tl.spans.iter().map(|s| s.label.clone()).collect();
            out.push(RecordInfo {
                db,
                name,
                content: content_of(&labels),
            });
        }
    }
    Ok(out)
}

/// Apply the split rule (module notes) to a list of records.
#[must_use]
pub fn make_split(records: &[RecordInfo]) -> Split {
    // Units: same-patient records grouped; key = (db, unit name).
    let mut units: BTreeMap<(Db, String), Vec<&RecordInfo>> = BTreeMap::new();
    for r in records {
        let group = SAME_PATIENT
            .iter()
            .find(|(db, names)| *db == r.db.name() && names.contains(&r.name.as_str()))
            .map_or_else(|| r.name.clone(), |(_, names)| names.join("+"));
        units.entry((r.db, group)).or_default().push(r);
    }
    let mut rng = Xorshift32(SPLIT_SEED);
    let mut split = Split::default();
    for db in Db::ALL {
        for content in [Content::Vf, Content::FlutterOnly, Content::Neither] {
            // A unit's content is its "strongest" record's content.
            let mut stratum: Vec<(&String, &Vec<&RecordInfo>)> = units
                .iter()
                .filter(|((d, _), recs)| {
                    *d == db && recs.iter().map(|r| r.content).min() == Some(content)
                })
                .map(|((_, name), recs)| (name, recs))
                .collect();
            stratum.sort_by(|a, b| a.0.cmp(b.0));
            for i in (1..stratum.len()).rev() {
                let j = (rng.next() % (i as u32 + 1)) as usize;
                stratum.swap(i, j);
            }
            let dev_n = stratum.len().div_ceil(2);
            for (k, (_, recs)) in stratum.iter().enumerate() {
                let side = if k < dev_n {
                    &mut split.development
                } else {
                    &mut split.held_out
                };
                side.extend(recs.iter().map(|r| format!("{}/{}", r.db.name(), r.name)));
            }
        }
    }
    let order = |v: &mut Vec<String>| {
        v.sort_by_key(|s| {
            let (db, rec) = s.split_once('/').unwrap_or(("", s));
            (Db::from_name(db), rec.to_string())
        });
    };
    order(&mut split.development);
    order(&mut split.held_out);
    split
}

/// The text of a split list file.
#[must_use]
pub fn list_text(title: &str, names: &[String]) -> String {
    let mut s = format!(
        "# {title}\n# Patient-wise split made by `aed-eval --make-split` (rule: src/split.rs, seed {SPLIT_SEED}).\n# One record per line, database/record. Lines starting with # are ignored.\n"
    );
    for n in names {
        s.push_str(n);
        s.push('\n');
    }
    s
}

/// Parse a list file: `db/record` per line; `#` comments and blank lines ignored.
///
/// # Errors
/// A message naming the first bad line.
pub fn parse_list(text: &str) -> Result<Vec<(Db, String)>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let bad = || format!("line {}: expected database/record, got {line:?}", i + 1);
        let (db, rec) = line.split_once('/').ok_or_else(bad)?;
        let db = Db::from_name(db).ok_or_else(bad)?;
        if rec.is_empty()
            || !rec
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(bad());
        }
        out.push((db, rec.to_string()));
    }
    Ok(out)
}

/// SHA-256 of a byte string, as lowercase hex (FIPS 180-4; written here
/// because the crate uses no outside libraries). Used to record which split
/// files a report was made with.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for block in msg.as_chunks::<64>().0 {
        let mut w = [0_u32; 64];
        for (slot, word) in w.iter_mut().zip(block.as_chunks::<4>().0) {
            *slot = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let g = |k: usize| w.get(k).copied().unwrap_or(0);
            let s0 = g(i - 15).rotate_right(7) ^ g(i - 15).rotate_right(18) ^ (g(i - 15) >> 3);
            let s1 = g(i - 2).rotate_right(17) ^ g(i - 2).rotate_right(19) ^ (g(i - 2) >> 10);
            let v = g(i - 16)
                .wrapping_add(s0)
                .wrapping_add(g(i - 7))
                .wrapping_add(s1);
            if let Some(slot) = w.get_mut(i) {
                *slot = v;
            }
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for (k, wi) in K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(*k)
                .wrapping_add(*wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_values() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let long = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(
            sha256_hex(long),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    fn rec(db: Db, name: &str, content: Content) -> RecordInfo {
        RecordInfo {
            db,
            name: name.into(),
            content,
        }
    }

    #[test]
    fn same_patient_records_stay_together_and_strata_are_halved() {
        let mut recs = Vec::new();
        for i in 0..10 {
            recs.push(rec(
                Db::Cudb,
                &format!("cu{i:02}"),
                if i < 6 { Content::Vf } else { Content::Neither },
            ));
        }
        recs.push(rec(Db::Mitdb, "201", Content::Neither));
        recs.push(rec(Db::Mitdb, "202", Content::Neither));
        recs.push(rec(Db::Mitdb, "100", Content::Neither));
        let s = make_split(&recs);
        assert_eq!(s.development.len() + s.held_out.len(), recs.len());
        let dev201 = s.development.contains(&"mitdb/201".to_string());
        let dev202 = s.development.contains(&"mitdb/202".to_string());
        assert_eq!(dev201, dev202);
        let vf_dev = s
            .development
            .iter()
            .filter(|n| n.starts_with("cudb/cu0") && n.as_str() < "cudb/cu06")
            .count();
        assert_eq!(vf_dev, 3);
        // Deterministic.
        assert_eq!(make_split(&recs), s);
    }

    #[test]
    fn list_round_trip_and_bad_lines() {
        let names = vec!["cudb/cu01".to_string(), "mitdb/201".to_string()];
        let parsed = parse_list(&list_text("t", &names)).unwrap();
        assert_eq!(
            parsed,
            vec![(Db::Cudb, "cu01".into()), (Db::Mitdb, "201".into())]
        );
        assert!(parse_list("nodb/x").is_err());
        assert!(parse_list("cudb/../etc").is_err());
        assert!(parse_list("cudb").is_err());
    }
}
