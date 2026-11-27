// SPDX-License-Identifier: GPL-3.0-or-later
//! Print the shock design table (a MODEL, not a measurement) as Markdown.
//!
//! Regenerate the saved copy from `firmware/` with:
//! `cargo run -p aed-waveform --example design_table > crates/aed-waveform/DESIGN_TABLE.md`

use aed_waveform::{write_report, WaveformConfig};

fn main() {
    let mut text = String::new();
    if write_report(&WaveformConfig::default(), &mut text).is_err() {
        eprintln!("could not format the design table");
        std::process::exit(1);
    }
    print!("{text}");
}
