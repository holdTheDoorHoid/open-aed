# Third-party code

The medical software standard (IEC 62304) requires every outside component to be listed and justified. Nothing is added here without a reason.

| Component | Used by | Why | Runs on the device? |
|---|---|---|---|
| `libm` | `aed-waveform`, `aed-rhythm`, `aed-cpr` | Maths functions (exponential, logarithm, square root) that the Rust core library does not provide without an operating system | Yes |

`aed-sim`, `aed-eval` and `aed-cpr-eval` run only on a development computer and may use the Rust standard library. `aed-cpr-eval` uses no third-party code (it has its own CSV reader and SHA-256).
