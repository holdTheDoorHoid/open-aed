# Firmware

Rust firmware for the Open AED, at the simulation stage. **No hardware exists. Nothing here has run on a device.** See [SAFETY.md](../SAFETY.md).

| Part | What it does |
|---|---|
| `crates/aed-types` | Shared vocabulary |
| `crates/aed-rhythm` | Decides shockable / not shockable / not sure |
| `crates/aed-waveform` | Plans the shock and models the discharge |
| `crates/aed-core` | The rescue sequence, the supervisor's rules, self-test, prompts, event record |
| `crates/aed-sim` | Simulated device and patient; runs everything together |

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

Known problems found in simulation are listed in [docs/FINDINGS.md](../docs/FINDINGS.md).
