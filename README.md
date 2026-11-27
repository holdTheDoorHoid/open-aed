# Open AED

[![Firmware tests](https://github.com/holdTheDoorHoid/open-aed/actions/workflows/firmware.yml/badge.svg)](https://github.com/holdTheDoorHoid/open-aed/actions/workflows/firmware.yml)

An open source design for a small, light automated external defibrillator (AED) that a police officer, paramedic or medical professional can carry on a belt, with one set of pads that works on both adults and children.

**Status: research, plan, firmware running in simulation only, and a first circuit design on paper. No hardware exists. Nothing here has been built or tested on a device, and the circuit design is unreviewed and not for construction.**

> **This is not a medical device.** Nothing in this repository is approved, cleared or safe for use on a person or an animal. A defibrillator stores enough energy to kill the person building it. Read [SAFETY.md](SAFETY.md) before doing anything beyond reading.

## Why this project exists

Research for this project (November 2026) found no mature open source AED. The only real open design, from the University of Pisa in 2017, never had its shock circuit tested on hardware and has been dormant since. Meanwhile, pocket-sized commercial units show that a full-strength 150 joule shock fits in about 0.7 kg and 0.7 litres.

The aim is an open, well-documented, well-tested reference design built from parts anyone can buy, using established and clinically proven techniques, so that a regulated manufacturer could one day take it through approval.

## What is here

| File | What it is |
|---|---|
| [docs/PLAN.md](docs/PLAN.md) | The phased roadmap, from research to a design a manufacturer could take forward |
| [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) | The draft target specification, with the reason for each number |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | How the device is divided up, the three checks before a shock, the rescue sequence |
| [firmware/](firmware/) | Rust firmware and a simulator that runs it against a simulated patient |
| [docs/HARDWARE_DESIGN.md](docs/HARDWARE_DESIGN.md) | The circuit design, block by block, with reasons and arithmetic. **Unreviewed; not for construction.** |
| [hardware/](hardware/) | KiCad circuit diagrams generated from pin tables; see [docs/HARDWARE_WORKFLOW.md](docs/HARDWARE_WORKFLOW.md) |
| [docs/PARTS_STUDY.md](docs/PARTS_STUDY.md) | Candidate parts and the size, weight and cost budget |
| [docs/TEST_INSTRUMENT.md](docs/TEST_INSTRUMENT.md) | What the test load and measuring rig must do, and how its accuracy will be proven |
| [docs/FINDINGS.md](docs/FINDINGS.md) | Problems the simulator has found so far |
| [DECISIONS.md](DECISIONS.md) | The decision log, and the open questions waiting on the project owner. **Source of truth.** |
| [SAFETY.md](SAFETY.md) | The rules that keep builders and the public safe |
| [reports/](reports/) | The full research report |
| [research_notes/](research_notes/) | The detailed notes behind the report, one file per topic, with sources |

## The approach in one paragraph

Copy what is proven; innovate only where it is safe to. The shock itself follows the established, published 150 joule biphasic waveform whose founding patents have expired, because regulators have accepted bench proof of equivalence to that waveform in place of a new human trial. The size savings come from modern parts, a rechargeable battery and careful packaging. The first things built are the low-voltage parts and the test equipment, which carry no shock hazard. High-voltage work starts only after a written lab safety policy and a qualified reviewer are in place.

## Licence

Share-alike: if you improve it, you publish the improvement. See [LICENSE.md](LICENSE.md).

- Hardware designs: CERN Open Hardware Licence v2, Strongly Reciprocal (CERN-OHL-S-2.0)
- Firmware and software: GNU General Public License v3 or later (GPL-3.0-or-later)
- Documents and research: Creative Commons Attribution-ShareAlike 4.0 (CC-BY-SA-4.0)
