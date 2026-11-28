// SPDX-License-Identifier: GPL-3.0-or-later
//! The host-side reader for the record the device sends down the dock
//! cable (decision D-09; format in `aed_core::export`). Standard library
//! only, so that anyone can see how little it takes to read one.
//!
//! The checksum is computed here with its own small CRC-32, independent of
//! the firmware's, so a mistake in one shows up as a mismatch.

use std::collections::BTreeMap;
use std::fmt::Write;

/// The format version this reader understands.
pub const SUPPORTED_VERSION: u16 = 1;

/// One heat record period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeatRow {
    /// Timer reading at the start of the period, seconds.
    pub start_s: u32,
    /// Hottest reading, tenths of a degree.
    pub max_dc: Option<i16>,
    /// Coolest reading, tenths of a degree.
    pub min_dc: Option<i16>,
    /// Wakes in the period.
    pub wakes: u16,
    /// Flags (see `aed_core::heat::flags`).
    pub flags: u8,
}

/// A record, read and checked.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Record {
    /// Format version.
    pub version: u16,
    /// Timer reading when written, seconds.
    pub timer_s: u32,
    /// The ready indicator.
    pub ready: bool,
    /// Every reason, by its short name.
    pub reasons: Vec<String>,
    /// `key=value` lines by their first word (`wear`, `tests`, `heat`,
    /// `pads`, `lifetime`).
    pub sections: BTreeMap<String, BTreeMap<String, String>>,
    /// Per sensor point: (name, highest, lowest), tenths of a degree.
    pub points: Vec<(String, Option<i16>, Option<i16>)>,
    /// The heat record, oldest first.
    pub heat: Vec<HeatRow>,
    /// The event record: (ms since switch-on, text).
    pub events: Vec<(u32, String)>,
    /// The checksum, as stated and checked.
    pub crc32: u32,
}

/// Why a record was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The first line is not `OPEN-AED-RECORD`.
    NotARecord,
    /// A version this reader does not know.
    UnsupportedVersion(u16),
    /// No `crc32` line at the end.
    MissingChecksum,
    /// The checksum does not match: the record was damaged.
    BadChecksum {
        /// In the record.
        stated: u32,
        /// Over what was received.
        computed: u32,
    },
    /// A line that does not fit the format (line number, from 1).
    BadLine(usize, String),
}

/// CRC-32 (IEEE 802.3), written separately from the firmware's.
#[must_use]
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (n, slot) in (0u32..).zip(table.iter_mut()) {
        let mut c = n;
        for _ in 0..8 {
            c = if c & 1 == 1 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for b in bytes {
        let i = ((crc ^ u32::from(*b)) & 0xFF) as usize;
        crc = table.get(i).copied().unwrap_or(0) ^ (crc >> 8);
    }
    !crc
}

fn temp(s: &str) -> Result<Option<i16>, ()> {
    if s == "-" {
        Ok(None)
    } else {
        s.parse().map(Some).map_err(|_| ())
    }
}

/// Read and check a record.
///
/// # Errors
/// See [`ParseError`].
pub fn parse(text: &str) -> Result<Record, ParseError> {
    if !text.starts_with("OPEN-AED-RECORD\n") {
        return Err(ParseError::NotARecord);
    }
    let at = text.rfind("\ncrc32 ").ok_or(ParseError::MissingChecksum)?;
    let body = text.get(..=at).ok_or(ParseError::MissingChecksum)?;
    let stated_text = text.get(at + 7..).unwrap_or("").trim();
    let stated = u32::from_str_radix(stated_text, 16).map_err(|_| ParseError::MissingChecksum)?;
    let computed = crc32(body.as_bytes());
    if stated != computed {
        return Err(ParseError::BadChecksum { stated, computed });
    }
    let mut r = Record {
        crc32: stated,
        ..Record::default()
    };
    let mut ended = false;
    for (n, line) in body.lines().enumerate().skip(1) {
        let bad = || ParseError::BadLine(n + 1, line.to_string());
        let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
        match word {
            "version" => {
                r.version = rest.parse().map_err(|_| bad())?;
                if r.version != SUPPORTED_VERSION {
                    return Err(ParseError::UnsupportedVersion(r.version));
                }
            }
            "timer_s" => r.timer_s = rest.parse().map_err(|_| bad())?,
            "ready" => r.ready = rest == "1",
            "reasons" => {
                if rest != "-" {
                    r.reasons = rest.split(',').map(str::to_string).collect();
                }
            }
            "wear" | "tests" | "heat" | "pads" | "lifetime" => {
                let mut m = BTreeMap::new();
                for kv in rest.split(' ') {
                    let (k, v) = kv.split_once('=').ok_or_else(bad)?;
                    m.insert(k.to_string(), v.to_string());
                }
                r.sections.insert(word.to_string(), m);
            }
            "point" => {
                let mut parts = rest.split(' ');
                let name = parts.next().ok_or_else(bad)?.to_string();
                let mut get = |key: &str| -> Result<Option<i16>, ParseError> {
                    let kv = parts.next().ok_or_else(bad)?;
                    let v = kv.strip_prefix(key).ok_or_else(bad)?;
                    temp(v).map_err(|()| bad())
                };
                let max = get("max_dc=")?;
                let min = get("min_dc=")?;
                r.points.push((name, max, min));
            }
            "h" => {
                let f: Vec<&str> = rest.split(' ').collect();
                let [start, max, min, wakes, flags] = f.as_slice() else {
                    return Err(bad());
                };
                r.heat.push(HeatRow {
                    start_s: start.parse().map_err(|_| bad())?,
                    max_dc: temp(max).map_err(|()| bad())?,
                    min_dc: temp(min).map_err(|()| bad())?,
                    wakes: wakes.parse().map_err(|_| bad())?,
                    flags: flags.parse().map_err(|_| bad())?,
                });
            }
            "event" => {
                let (at, what) = rest.split_once(' ').ok_or_else(bad)?;
                r.events
                    .push((at.parse().map_err(|_| bad())?, what.to_string()));
            }
            "end" => ended = true,
            _ => return Err(bad()),
        }
    }
    if !ended || r.version == 0 {
        return Err(ParseError::BadLine(
            0,
            "no version or no end line".to_string(),
        ));
    }
    Ok(r)
}

impl Record {
    /// A `key=value` from a section.
    #[must_use]
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections.get(section)?.get(key).map(String::as_str)
    }

    /// A number from a section.
    #[must_use]
    pub fn number(&self, section: &str, key: &str) -> Option<u64> {
        self.get(section, key)?.parse().ok()
    }
}

fn deg(dc: Option<i16>) -> String {
    dc.map_or_else(
        || "unknown".to_string(),
        |t| format!("{:.1} °C", f64::from(t) / 10.0),
    )
}

fn deg_text(s: Option<&str>) -> String {
    deg(s.and_then(|v| v.parse().ok()))
}

/// A plain-language summary of a record, for a person.
#[must_use]
pub fn pretty(r: &Record) -> String {
    let mut o = String::new();
    let _ = writeln!(
        o,
        "Open AED record (format {}), timer at {} s",
        r.version, r.timer_s
    );
    let _ = writeln!(o, "Ready: {}", if r.ready { "yes" } else { "NO" });
    if r.reasons.is_empty() {
        let _ = writeln!(o, "Reasons: none");
    } else {
        let _ = writeln!(o, "Reasons:");
        for reason in &r.reasons {
            let _ = writeln!(o, "  - {}", reason.replace('-', " "));
        }
    }
    let _ = writeln!(
        o,
        "Capacitor: {} full charges of {} ({} tests, {} rescues): {}",
        r.get("wear", "total").unwrap_or("?"),
        r.get("wear", "life").unwrap_or("?"),
        r.get("wear", "test_charges").unwrap_or("?"),
        r.get("wear", "rescue_charges").unwrap_or("?"),
        r.get("wear", "status").unwrap_or("?").replace('-', " ")
    );
    let _ = writeln!(
        o,
        "Self-tests: {} daily passed, {} failed; {} full-energy passed, {} failed; {} missed",
        r.get("tests", "daily_passed").unwrap_or("?"),
        r.get("tests", "daily_failed").unwrap_or("?"),
        r.get("tests", "full_passed").unwrap_or("?"),
        r.get("tests", "full_failed").unwrap_or("?"),
        r.number("tests", "missed_daily").unwrap_or(0)
            + r.number("tests", "missed_full").unwrap_or(0)
    );
    let _ = writeln!(
        o,
        "Temperature now: {} ({}); hottest ever {}, coolest {}",
        deg_text(r.get("heat", "hottest_dc")),
        r.get("heat", "band").unwrap_or("?").replace('-', " "),
        deg_text(r.get("lifetime", "max_dc")),
        deg_text(r.get("lifetime", "min_dc"))
    );
    let hours = |k: &str| r.number("lifetime", k).unwrap_or(0) as f64 / 3_600.0;
    let _ = writeln!(
        o,
        "Time above 40 °C: {:.1} h; above 50 °C: {:.1} h; above 60 °C: {:.1} h; gaps in the record: {}",
        hours("above40_s"),
        hours("above50_s"),
        hours("above60_s"),
        r.get("lifetime", "gaps").unwrap_or("?")
    );
    let _ = writeln!(
        o,
        "Pads: {:.1} days above 50 °C of an allowance of {:.1} days",
        r.number("pads", "hot_s").unwrap_or(0) as f64 / 86_400.0,
        r.number("pads", "allowance_s").unwrap_or(0) as f64 / 86_400.0
    );
    for (name, max, min) in &r.points {
        let _ = writeln!(
            o,
            "  sensor {name:<10} highest {:>9}, lowest {:>9}",
            deg(*max),
            deg(*min)
        );
    }
    let _ = writeln!(o, "Heat record: {} periods", r.heat.len());
    for row in r.heat.iter().filter(|h| h.flags != 0) {
        let _ = writeln!(
            o,
            "  at {} s: highest {}, flags {}",
            row.start_s,
            deg(row.max_dc),
            row.flags
        );
    }
    let _ = writeln!(o, "Events: {}", r.events.len());
    for (at, what) in &r.events {
        let _ = writeln!(o, "  {:>8.1} s  {what}", f64::from(*at) / 1_000.0);
    }
    o
}
