// SPDX-License-Identifier: GPL-3.0-or-later
//! Command-line front end for the Open AED simulator.
//!
//! ```text
//! cargo run -p aed-sim -- --list
//! cargo run -p aed-sim -- adult-vf
//! cargo run -p aed-sim -- timings
//! cargo run -p aed-sim -- campaign [runs] [first-seed]
//! ```
//!
//! SIMULATION ONLY. NOT A MEDICAL DEVICE.

use std::process::ExitCode;

use aed_sim::campaign::{campaign_text, run_campaign};
use aed_sim::report::{summary, timeline, timing_table};
use aed_sim::{by_name, catalogue, run};

const BANNER: &str = "Open AED simulator. SIMULATION ONLY: a model of the firmware against a \
simulated device and patient. Not a medical device; says nothing about real hardware.";

fn usage() {
    println!("{BANNER}\n");
    println!("Usage:");
    println!("  aed-sim --list                    list the scenarios");
    println!("  aed-sim <scenario>                run one scenario and print its timeline");
    println!("  aed-sim timings                   run every scenario and print the timing table");
    println!("  aed-sim campaign [runs] [seed]    random campaign (default 300 runs, seed 1)");
    println!("  aed-sim random <seed>             replay one run of the random campaign");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(first) = args.first() else {
        usage();
        return ExitCode::SUCCESS;
    };
    match first.as_str() {
        "--help" | "-h" => {
            usage();
            ExitCode::SUCCESS
        }
        "--list" => {
            for s in catalogue() {
                println!("{:<34} {}", s.name, s.description);
            }
            ExitCode::SUCCESS
        }
        "timings" => {
            let mut runs = Vec::new();
            for s in catalogue() {
                match run(&s) {
                    Ok(r) => runs.push(r),
                    Err(e) => {
                        eprintln!("{}: setup error {e:?}", s.name);
                        return ExitCode::FAILURE;
                    }
                }
            }
            println!("{BANNER}\n");
            println!("Timings per shock (simulated, provisional settings):\n");
            print!("{}", timing_table(&runs));
            ExitCode::SUCCESS
        }
        "campaign" => {
            let n = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(300);
            let seed = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(1);
            let s = run_campaign(n, seed);
            println!("{BANNER}\n");
            print!("{}", campaign_text(&s));
            if s.clean() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        name => {
            let chosen = if name == "random" {
                args.get(1)
                    .and_then(|a| a.parse().ok())
                    .map(aed_sim::campaign::random_scenario)
            } else {
                by_name(name)
            };
            let Some(s) = chosen else {
                eprintln!("Unknown scenario '{name}'. Use --list.");
                return ExitCode::FAILURE;
            };
            match run(&s) {
                Ok(r) => {
                    println!("{BANNER}\n");
                    println!("SCENARIO {}: {}\n", s.name, s.description);
                    print!("{}", timeline(&r));
                    println!();
                    print!("{}", summary(&r));
                    if r.violations.is_empty() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(e) => {
                    eprintln!("setup error {e:?}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
