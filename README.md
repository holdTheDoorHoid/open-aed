# Open AED

An open source design for a small, light automated external defibrillator (AED) that a police officer, paramedic or medical professional can carry on a belt, with one set of pads that works on both adults and children.

**Status: planning and research. No hardware exists. Nothing here has been built or tested.**

> **This is not a medical device.** Nothing in this repository is approved, cleared or safe for use on a person or an animal. A defibrillator stores enough energy to kill the person building it. Read [SAFETY.md](SAFETY.md) before doing anything beyond reading.

## Why this project exists

Research for this project (November 2026) found no mature open source AED. The only real open design, from the University of Pisa in 2017, never had its shock circuit tested on hardware and has been dormant since. Meanwhile, pocket-sized commercial units show that a full-strength 150 joule shock fits in about 0.7 kg and 0.7 litres.

The aim is an open, well-documented, well-tested reference design built from parts anyone can buy, using established and clinically proven techniques, so that a regulated manufacturer could one day take it through approval.

## What is here

| File | What it is |
|---|---|
| [docs/PLAN.md](docs/PLAN.md) | The phased roadmap, from research to a design a manufacturer could take forward |
| [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) | The draft target specification, with the reason for each number |
| [DECISIONS.md](DECISIONS.md) | The decision log, and the open questions waiting on the project owner. **Source of truth.** |
| [SAFETY.md](SAFETY.md) | The rules that keep builders and the public safe |
| [reports/](reports/) | The full research report |
| [research_notes/](research_notes/) | The detailed notes behind the report, one file per topic, with sources |

## The approach in one paragraph

Copy what is proven; innovate only where it is safe to. The shock itself follows the established, published 150 joule biphasic waveform whose founding patents have expired, because regulators have accepted bench proof of equivalence to that waveform in place of a new human trial. The size savings come from modern parts, a rechargeable battery and careful packaging. The first things built are the low-voltage parts and the test equipment, which carry no shock hazard. High-voltage work starts only after a written lab safety policy and a qualified reviewer are in place.

## Licence

Not yet chosen; see question Q-03 in [DECISIONS.md](DECISIONS.md). Until a licence is added, all rights are reserved.
