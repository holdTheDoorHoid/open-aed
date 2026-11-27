// SPDX-License-Identifier: GPL-3.0-or-later
//! The record download, by cable through the dock contacts (decision D-09:
//! no Bluetooth in the first prototype; a radio is a later option, D-45).
//!
//! NOT A MEDICAL DEVICE. Format only; no dock or cable exists.
//!
//! # Format, version 1
//!
//! Plain ASCII text, one item per line, lines ending in `\n`, so that it
//! can be read with nothing but a text editor or a few lines of any
//! language's standard library. Temperatures are in tenths of a degree
//! Celsius (`_dc`), times in seconds of the low-power timer (`_s`) or
//! milliseconds of the main controller's clock since switch-on (events).
//! `-` means "none" or "unknown".
//!
//! ```text
//! OPEN-AED-RECORD
//! version 1
//! timer_s <now>
//! ready <1|0>
//! reasons <name,name,...|->            names from Reason::name
//! wear test_charges=<n> rescue_charges=<n> total=<n> life=<n> status=<ok|service-soon|service-due>
//! tests daily_passed=<n> daily_failed=<n> full_passed=<n> full_failed=<n> missed_daily=<n> missed_full=<n> deferrals=<n> interrupted=<n> clock_restarts=<n> tests_on_charge=<n>
//! heat band=<normal|warm|too-hot> hottest_dc=<t|-> service_needed=<0|1> cool_causes=<n> gap_pending=<0|1> sensor_fault=<0|1> record_lost=<0|1>
//! pads set=<id|-> hot_s=<n> allowance_s=<n> replace_after_overheat=<0|1>
//! lifetime max_dc=<t|-> min_dc=<t|-> raw_max_dc=<t|-> above40_s=<n> above50_s=<n> above60_s=<n> too_hot_events=<n> service_events=<n> gaps=<n> gap_s=<n> gaps_unknown=<n> sensor_fault_wakes=<n> pad_sets=<n> log_overwritten=<n>
//! point <name> max_dc=<t|-> min_dc=<t|->           one line per sensor point
//! h <start_s> <max_dc|-> <min_dc|-> <wakes> <flags>  one line per heat record period, oldest first
//! event <at_ms> <text>                             the last rescue's event record, oldest first
//! end
//! crc32 <8 hex digits>
//! ```
//!
//! `flags` on an `h` line is the decimal sum of [`crate::heat::flags`].
//! The event text is the Rust `Debug` form of [`crate::Event`]; its variant
//! and field names are part of the format, so renaming one means a new
//! version number. The `crc32` is CRC-32 (the IEEE 802.3 polynomial, as in
//! zip and PNG) of every byte before the `crc32` line. A reader must
//! refuse a record whose checksum or version it does not recognise.

use core::fmt::{self, Write};

use crate::events::EventRecord;
use crate::heat::{DeciC, HeatBand, TempPoint};
use crate::standby::{Standby, WearStatus};

/// The version this firmware writes.
pub const EXPORT_FORMAT_VERSION: u16 = 1;

/// The first line of every record.
pub const EXPORT_MAGIC: &str = "OPEN-AED-RECORD";

/// Add `bytes` to a running CRC-32 (IEEE 802.3, reflected, initial and
/// final value `0xFFFF_FFFF`). Start with `0`.
#[must_use]
pub fn crc32_update(crc: u32, bytes: &[u8]) -> u32 {
    let mut c = !crc;
    for b in bytes {
        c ^= u32::from(*b);
        for _ in 0..8 {
            let mask = (c & 1).wrapping_neg();
            c = (c >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !c
}

/// Passes text on and keeps the checksum of everything written.
struct CrcWriter<'a, W: Write> {
    out: &'a mut W,
    crc: u32,
}

impl<W: Write> Write for CrcWriter<'_, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.crc = crc32_update(self.crc, s.as_bytes());
        self.out.write_str(s)
    }
}

/// A temperature or `-`.
struct T(Option<DeciC>);

impl fmt::Display for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(t) => write!(f, "{}", t.0),
            None => f.write_str("-"),
        }
    }
}

const fn band_name(b: HeatBand) -> &'static str {
    match b {
        HeatBand::Normal => "normal",
        HeatBand::Warm => "warm",
        HeatBand::TooHot => "too-hot",
    }
}

/// Write the record (format version [`EXPORT_FORMAT_VERSION`]) to `out`, at
/// timer reading `now_s`, with the last rescue's event record if there is
/// one. `out` is whatever carries bytes to the dock contacts; no buffer is
/// needed.
///
/// # Errors
/// Whatever `out` reports.
pub fn write_record<W: Write, const N: usize, const E: usize>(
    out: &mut W,
    standby: &Standby<N>,
    now_s: u32,
    events: Option<&EventRecord<E>>,
) -> fmt::Result {
    let cfg = standby.config();
    let readiness = standby.readiness(now_s);
    let heat = standby.heat();
    let wear = standby.wear();
    let tc = standby.schedule().counters();
    let life = heat.lifetime();
    let mut w = CrcWriter { out, crc: 0 };
    writeln!(w, "{EXPORT_MAGIC}")?;
    writeln!(w, "version {EXPORT_FORMAT_VERSION}")?;
    writeln!(w, "timer_s {now_s}")?;
    writeln!(w, "ready {}", u8::from(readiness.is_ready()))?;
    w.write_str("reasons ")?;
    if readiness.reasons.is_empty() {
        w.write_str("-")?;
    }
    for (i, r) in readiness.reasons.iter().enumerate() {
        if i > 0 {
            w.write_str(",")?;
        }
        w.write_str(r.name())?;
    }
    w.write_str("\n")?;
    let status = match wear.status(cfg) {
        WearStatus::Ok => "ok",
        WearStatus::ServiceSoon => "service-soon",
        WearStatus::ServiceDue => "service-due",
    };
    writeln!(
        w,
        "wear test_charges={} rescue_charges={} total={} life={} status={status}",
        wear.test_charges,
        wear.rescue_charges,
        wear.total(),
        cfg.capacitor_life_charges
    )?;
    writeln!(
        w,
        "tests daily_passed={} daily_failed={} full_passed={} full_failed={} missed_daily={} \
         missed_full={} deferrals={} interrupted={} clock_restarts={} tests_on_charge={}",
        tc.daily_passed,
        tc.daily_failed,
        tc.full_passed,
        tc.full_failed,
        tc.missed_daily,
        tc.missed_full,
        tc.deferrals,
        tc.interrupted,
        tc.clock_restarts,
        tc.tests_on_charge
    )?;
    writeln!(
        w,
        "heat band={} hottest_dc={} service_needed={} cool_causes={} gap_pending={} sensor_fault={} record_lost={}",
        band_name(heat.band()),
        T(heat.hottest_now()),
        u8::from(heat.service_needed()),
        heat.cool_causes(),
        u8::from(heat.gap_pending()),
        u8::from(heat.sensor_fault()),
        u8::from(heat.record_lost())
    )?;
    w.write_str("pads set=")?;
    match heat.pad_set() {
        Some(id) => write!(w, "{id}")?,
        None => w.write_str("-")?,
    }
    writeln!(
        w,
        " hot_s={} allowance_s={} replace_after_overheat={}",
        heat.pad_hot_s(),
        cfg.pad_heat_allowance_s,
        u8::from(heat.pads_after_overheat())
    )?;
    writeln!(
        w,
        "lifetime max_dc={} min_dc={} raw_max_dc={} above40_s={} above50_s={} above60_s={} \
         too_hot_events={} service_events={} gaps={} gap_s={} gaps_unknown={} \
         sensor_fault_wakes={} pad_sets={} log_overwritten={}",
        T(life.max),
        T(life.min),
        T(life.raw_max),
        life.above_normal_s,
        life.above_warm_spell_s,
        life.above_too_hot_s,
        life.too_hot_events,
        life.service_events,
        life.gaps,
        life.gap_s,
        life.gaps_unknown_length,
        life.sensor_fault_wakes,
        life.pad_sets,
        heat.log().overwritten()
    )?;
    for ((p, mx), mn) in TempPoint::ALL
        .iter()
        .zip(life.point_max.iter())
        .zip(life.point_min.iter())
    {
        writeln!(w, "point {} max_dc={} min_dc={}", p.name(), T(*mx), T(*mn))?;
    }
    for e in heat.log().iter().copied().chain(heat.current_entry()) {
        writeln!(
            w,
            "h {} {} {} {} {}",
            e.start_s,
            T(e.max),
            T(e.min),
            e.wakes,
            e.flags
        )?;
    }
    if let Some(record) = events {
        for e in record.iter() {
            writeln!(w, "event {} {:?}", e.at.0, e.event)?;
        }
    }
    writeln!(w, "end")?;
    let crc = w.crc;
    writeln!(w.out, "crc32 {crc:08x}")
}
