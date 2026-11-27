# Firmware

Rust firmware for the Open AED, at the simulation stage. **No hardware exists. Nothing here has run on a device.** See [SAFETY.md](../SAFETY.md).

| Part | What it does |
|---|---|
| `crates/aed-types` | Shared vocabulary |
| `crates/aed-rhythm` | Decides shockable / not shockable / not sure |
| `crates/aed-waveform` | Plans the shock and models the discharge |
| `crates/aed-cpr` | Senses chest compressions from the resistance between the pads (D-14, D-36): hands off / compressing / cannot tell, rate, pauses, time on chest |
| `crates/aed-core` | The rescue sequence, the supervisor's rules, self-test (at switch-on, and the daily and weekly periodic tests of D-15), heat limits and the temperature record (D-48, D-51, D-52), capacitor wear (D-53), the one readiness result, prompts, event record, and the record download by cable through the dock (D-09) |
| `crates/aed-eval` | Scores the rhythm analysis against public recordings of real heart rhythms (data not included) |
| `crates/aed-cpr-eval` | Scores compression sensing against real pad-impedance recordings (Paris Fire Brigade; data not included). Results: `crates/aed-cpr-eval/REPORT.md` |
| `crates/aed-sim` | Simulated device and patient, with a simple thermal model; runs everything together; reads the record download back (`read-record`) |

Run from this folder:

```bash
cargo test --workspace
```

```bash
cargo run -p aed-sim -- --list
```

```bash
cargo run -p aed-sim -- adult-vf
```

There is no radio code in the firmware: records come off by cable through the dock contacts (D-09). A Bluetooth module is a later option (D-45).

Known problems found in simulation are listed in [docs/FINDINGS.md](../docs/FINDINGS.md).
