// SPDX-License-Identifier: GPL-3.0-or-later
//! The record download by cable (D-09): the firmware writes it
//! (`aed_core::write_record`), the host reader (`aed_sim::record`) reads it
//! back. Round trips on simulated data.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, missing_docs)]

use aed_core::heat::HEAT_LOG_CAPACITY;
use aed_core::{write_record, Reason, TempPoint, EXPORT_FORMAT_VERSION};
use aed_sim::record::{crc32, parse, pretty, ParseError};
use aed_sim::standby::{run as run_standby, StandbyScenario, DAY_S};
use aed_sim::{by_name, Simulation};

fn hot_week(t: u32) -> f64 {
    if (3 * DAY_S + 13 * 3_600..3 * DAY_S + 16 * 3_600).contains(&t) {
        64.0
    } else {
        22.0
    }
}

fn sample() -> (String, aed_sim::standby::StandbyResult, usize) {
    let r = run_standby(&StandbyScenario {
        weather: Some(hot_week),
        ..StandbyScenario::quiet(6)
    })
    .unwrap();
    let mut sim = Simulation::new(&by_name("adult-vf").unwrap()).unwrap();
    for _ in 0..60_000 {
        sim.step();
    }
    let events = sim.rescue().events().clone();
    let mut text = String::new();
    write_record::<_, HEAT_LOG_CAPACITY, 256>(&mut text, &r.standby, 6 * DAY_S, Some(&events)).unwrap();
    (text, r, events.len())
}

#[test]
fn the_record_round_trips_through_the_host_reader() {
    let (text, r, n_events) = sample();
    let rec = parse(&text).unwrap();
    let sb = &r.standby;
    assert_eq!(rec.version, EXPORT_FORMAT_VERSION);
    assert_eq!(rec.timer_s, 6 * DAY_S);
    let readiness = sb.readiness(6 * DAY_S);
    assert_eq!(rec.ready, readiness.is_ready());
    let names: Vec<String> = readiness.reasons.iter().map(|x| Reason::name(x).to_string()).collect();
    assert_eq!(rec.reasons, names);
    assert_eq!(rec.number("wear", "total"), Some(u64::from(sb.wear().total())));
    assert_eq!(rec.number("tests", "daily_passed"), Some(u64::from(sb.schedule().counters().daily_passed)));
    assert_eq!(rec.number("pads", "hot_s"), Some(u64::from(sb.heat().pad_hot_s())));
    assert_eq!(rec.number("lifetime", "too_hot_events"), Some(1));
    let life = sb.heat().lifetime();
    assert_eq!(rec.get("lifetime", "max_dc").unwrap(), life.max.unwrap().0.to_string());
    assert_eq!(rec.points.len(), 6);
    assert_eq!(rec.points[TempPoint::Capacitor.index()].1, life.point_max[2].map(|t| t.0));
    let entries: Vec<_> = sb.heat().log().iter().copied().chain(sb.heat().current_entry()).collect();
    assert_eq!(rec.heat.len(), entries.len());
    for (row, e) in rec.heat.iter().zip(&entries) {
        assert_eq!((row.start_s, row.max_dc, row.min_dc, row.wakes, row.flags), (e.start_s, e.max.map(|t| t.0), e.min.map(|t| t.0), e.wakes, e.flags));
    }
    assert_eq!(rec.events.len(), n_events);
    assert!(rec.events.iter().any(|(_, e)| e.starts_with("Shock(Delivered)")));
    let p = pretty(&rec);
    assert!(p.contains("Ready:") && p.contains("hottest ever 64."));
    println!("{p}");
}

#[test]
fn a_damaged_record_is_refused() {
    let (text, _, _) = sample();
    let damaged = text.replacen("too_hot_events=1", "too_hot_events=0", 1);
    assert!(matches!(parse(&damaged), Err(ParseError::BadChecksum { .. })));
    let truncated = &text[..text.len() / 2];
    assert!(parse(truncated).is_err());
    assert_eq!(parse("hello\n"), Err(ParseError::NotARecord));
    let v2 = text.replacen("version 1", "version 2", 1);
    let at = v2.rfind("\ncrc32 ").unwrap();
    let body = &v2[..=at];
    let resealed = format!("{body}crc32 {:08x}\n", crc32(body.as_bytes()));
    assert_eq!(parse(&resealed), Err(ParseError::UnsupportedVersion(2)));
}

#[test]
fn the_host_checksum_and_the_firmware_checksum_agree() {
    for s in ["", "a", "123456789", "OPEN-AED-RECORD\nversion 1\n"] {
        assert_eq!(crc32(s.as_bytes()), aed_core::crc32_update(0, s.as_bytes()));
    }
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
}
