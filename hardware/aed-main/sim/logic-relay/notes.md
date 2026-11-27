**NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.**

# Simulation notes: safety logic timing, patient-relay supply, battery protection, sense channels

Written 27 November 2026 with AI assistance. These are the first circuit simulations of this design (ngspice 42,
batch mode). They check the drawn pin tables in `hardware/aed-main/specs/` against the timing and margin claims in
`docs/HARDWARE_DESIGN.md` v0.2 and the capture notes WP-A to WP-H. Nothing here is a measurement, an instruction to
build anything, or a claim that anything is safe or ready for use on a person or animal (SAFETY.md, CLAUDE.md rules
2-4).

## In plain language

The device's shock path is guarded by a set of small electronic timers and "consent detectors". The firmware has to
wait for those timers before taking its next step; if it waits too little, a real unit would sometimes refuse a
shock. The simulations below measured how long each timer can really take once every part is at the edge of its
tolerance and at -20 °C or +70 °C, and turned that into firmware waiting times.

What came out, most important first:

1. **The four firmware waits** (relay settled, re-arm, test relay, rail ready) are now backed by simulated ranges.
   Two of the proposed values in the capture notes are a little short; the table in §1.1 gives the new values.
2. **The relay "has settled for 10 ms" check can be fooled by contact bounce.** If the relay's position contact
   flickers once while the armature is landing, the timer does not fully restart, and the shock can be allowed
   4-8 ms after the last flicker instead of 10 ms. A two-part fix (a small input filter and a transistor that empties
   the timer) restores ≥ 11 ms in simulation.
3. **The consent detector meets its "stays on ≥ 3 ms" rule but slightly misses "goes off within 8 ms"** (8.3 ms at
   the hot corner). One resistor value change fixes it. It also confirms the known weakness that a single change of
   level on the permit line, or the supervisor simply powering up, produces a few milliseconds of "permit" that
   should not exist. That cannot be designed out by changing values; the 20 ms rail-ready delay keeps it from
   reaching the output stage.
4. **The relay supply's start-up surge is real: about 9 A for 15-33 ms**, against 2 A budgeted. With a cold, nearly
   empty battery it pulls the battery rail down to 4.4 V and the 5 V rail to 4.2 V, below what the shock logic
   accepts. The relay also then takes up to 32 ms to change over, which leaves no room inside the firmware's 40 ms
   limit. Filling the relay's energy store earlier (before the shock is due) and limiting its charging current
   removes the surge and halves the changeover time in simulation.
5. **The 25 ms shock time limit varies 23.8-26.2 ms.** It could be shortened to about 18 ms without ever cutting the
   longest planned shock in the design table (16 ms), but not to 12 ms, and only if the firmware's own 20 ms ceiling
   on shock length is lowered to match.
6. **Battery:** the "no load under 100 kΩ on the battery rail" rule is right if the protector's test current is
   really 30 µA; nothing on the drawn rail breaks it. If the true minimum test current turns out to be half that,
   100 kΩ is not enough. The battery load test can be made to run the 86 W load at 14-23 W average by a faulty
   pin that toggles (up to 23 W in simulation); a diode, one new resistor and two value changes cut that to about 2 W.
7. **Current sense:** the shock-current signal can exceed the main controller's input range when the amplifier
   saturates; one resistor change fixes it. **Open voltage-tap wire:** as drawn it can read "below 60 V" for ever; a
   1 GΩ resistor makes it read as a fault within 0.4 s. **Voltage divider drift:** the main chain fits its error
   budget only if its high-voltage resistors drift ≤ 0.59 % over life, as the capture notes said.
8. **Compression sensing is about 3 times noisier than the 10 mΩ target as drawn**, because the heart-signal filter
   capacitor sits on the same node and swallows most of the 64 kHz signal. Giving the impedance channel its own pair
   of input resistors brings it to 4.6-6.2 mΩ. Interference from the high-voltage charger, whenever one of its
   harmonics passes 64 kHz, is far larger than 10 mΩ for any plausible stray coupling: compression readings taken
   while charging cannot be trusted and must be blanked or measured.

Confidence: the timer and logic results rest on datasheet limits and are as good as those limits (moderate to
high). The relay supply, battery and interference results rest on several estimated values (battery resistance,
coil inductance, relay mechanics, stray capacitance) and show trends and orders of magnitude, not guaranteed
numbers (low to moderate). Each table says which.

## How to rerun

`bash run.sh` runs every netlist and writes `out/<netlist>.log` and `out/summary.txt` (about 10-15 minutes; `out/`
is not kept in git). `bash run.sh timers` runs one netlist. Every number in this file is a `RESULT` line in
`out/summary.txt`. Log lines "measure ... out of interval" (battery.cir, sense.cir) mean the event being timed never
happened, e.g. the protector never released or LOWV_OK never fell; that absence is the result. All eight netlists ran
to completion in about 6 minutes on 27 November 2026.

| File | What it simulates |
|---|---|
| `models/behavioural.lib` | Diode fits (BAV99, BAS116, SS14), LVC Schmitt gate, TLV7031/7041 comparator: behavioural models from datasheet figures |
| `det_permit.cir` | PERMIT_CLK → PERMIT_OK detector (09), all corners; glitches, stuck line, power-up, ripple, 100 kHz |
| `timers.cir` | XFER, BTOK, RDY, 25 ms cap, 40 ms backstop, coil window/cap, 150 ms window, 50 ms re-arm; shorter-cap options; proposed BTOK |
| `rail_up.cir` | BRIDGE_5V rise → BRIDGE_ARMED_N → BRIDGE_3V3 → RST_N → RDY |
| `races.cir` | relay-contact bounce against XFER; window re-arm release time |
| `relay_supply.cir` | coil supply inrush, VBAT_SYS / 5V0_SYS / BRIDGE_5V droop, relay changeover time; proposed pre-charge |
| `battery.cir` | MAX17320 release against loads on VBAT_SYS; load test driven by a toggling enable; proposed fix |
| `sense.cir` | I_SHOCK headroom; open voltage-tap wire; main divider drift |
| `imp_noise.cir` | impedance (compression) channel sensitivity, noise and charger interference at 64 kHz |

## Models and their sources

No maker's SPICE file was downloaded (so none needs a redistribution check); every model is behavioural and written
from the datasheet numbers already cited in the capture notes:

| Part | Model | Source of the numbers | Confidence |
|---|---|---|---|
| BAV99 | Shockley fit: max curve VF 0.715 V @ 1 mA, 0.85 V @ 10 mA; typical and low-drop variants [EST] | onsemi BAV99LT1/D Rev 14 (via WP-D §2) | moderate: the typical/low curves are estimates; temperature by ngspice's default silicon law |
| BAS116 | VF 0.9 V @ 1 mA fit; reverse leakage added as 0 / 5 / 20 / 80 nA | onsemi BAS116LT1/D Rev 14; 80 nA is the 150 °C limit, used as a bound | moderate |
| SN74LVC1G14/1G17/2G14 | thresholds as fractions of VCC (VT+ 0.487-0.640, VT- 0.297-0.438), 23 Ω output, input leakage ±5 µA | TI SCES351Y / SCES218AA (WP-D §3) | high for thresholds |
| TLV7031/7041 | VOS ±8 mV, hysteresis 3-25 mV, 3 µs delay, POR low below 1.6 V | TI SLVSE13J (WP-D §2) | high |
| TPS22918 | output slew 0.55 × CT[pF] + 30 µs/V, RON 79 mΩ | TI SLVSD76C Eq. 3 | moderate (CT is X7R, ±25 % assumed) |
| TPS3840PL30, TLV75533P | threshold, hysteresis, start-up delay; LDO as ideal with 0.1 V dropout and 0.5 ms soft start [EST] | TI SNVSB03E, SBVS320D | moderate |
| TPS61089, DRV110, Q1401 | average models: input current limit, efficiency 0.9 [EST], soft start 2 ms [EST]; coil current regulated to 0.857 / 0.496 A | WP-H §3.5 values | low to moderate |
| Relay K1401 | 12 Ω coil (15 Ω hot), 60 mH [EST], pick-up at 0.67 A, contacts move 10 ms after pick-up [EST] | WP-H §3.5, G17 figures | low: no relay has been chosen |
| Battery | 2 × P28A, 20 mΩ per cell at 25 °C, 80 mΩ at -20 °C [EST], 25 mΩ path [EST] | not from a datasheet | low |
| MAX17320 PCKP test current | 30 µA (and 15 µA as a what-if) | specs/01 group gauge quoting the ADI datasheet; tolerance not found | low |
| INA240A1 | gain 20 ±0.2 %, output limited at VS - 0...0.2 V | TI SBOS662C via WP-G | moderate |
| ADS1292R chip noise | 0.13 µV/√Hz | WP-H estimate from the datasheet respiration figure | low to moderate |

Tolerance bands (the same as capture-notes/WP-D.md §3 so that the two can be compared): C0G ±5.5 %, or ±2.2 % where
a ±2 % part is drawn; thin film ±0.35 %; thick film ±2 %; temperature -20 to +70 °C.

---

## 1. Consent detectors and sequencer timing (I-9)

### 1.1 Recommended firmware waits

The firmware itself is not changed here. "Tick" means the firmware's own timing granularity, assumed 1 ms.

| Wait (where it is written) | Current value | Hardware time it must cover (simulated, all corners) | Recommended | Why |
|---|---|---|---|---|
| Relay settled: F-4 "wait `K_REST_N` high + 10 ms" (HARDWARE_DESIGN F-4; no constant exists yet in `aed-core`) | 10 ms (capture notes CI-1 proposed 13 ms) | XFER 10.42-12.17 ms after the contact leaves rest (nominal 11.30) | **14 ms, counted from the last time `K_REST_N` was seen low** (restart the count on any low sample) | 12.17 ms + 1 ms tick + 0.8 ms margin. Counting from the first edge is not enough: see §1.4, contact bounce |
| Re-arm: F-4.6 "consent absent ≥ 50 ms"; `interlock.rs` `LatchTiming::PROPOSED.rearm_us` | 50 ms (F-4.6); 20,000 µs (model); CI-1 proposed 70 ms | re-arm comparator 51.24-60.34 ms after the last consent drops, plus the permit detector's own drop-out ≤ 8.28 ms after the supervisor stops its clock (§1.2) = 68.6 ms | **75 ms** (F-4.6) measured from the moment the supervisor stops `PERMIT_CLK` / `TEST_PERMIT_CLK` and the main drops `SHOCK_REQ`, whichever is later; `rearm_us` in the model = **60,500** (the hardware's own maximum) | 68.6 + 1 tick + 5.4 margin. 70 ms leaves only 1.4 ms |
| Test relay: F-8.3 "`BTEST_CMD` → 2 ms" | 2 ms (CI-1 proposed 5 ms) | BTOK 1.88-4.58 ms as drawn | **6 ms** as drawn; **7 ms** with the proposed BTOK change (§1.3: 4.74-5.52 ms) | 4.58 + 1 tick + 0.4 margin. The drawn BTOK can expire at 1.88 ms, *before* the test relay K1301 can close (3 ms operate, WP-G) |
| Rail ready: F-4 "wait `BRIDGE_ARMED_N` low → wait 20 ms → `RELAY_CMD`"; CI-1 "`CMD_LB` ≥ 30 ms after `BRIDGE_ARMED_N`" | 20 ms before `RELAY_CMD` | RDY 21.56-26.67 ms after `BRIDGE_ARMED_N` goes low (rail_up.cir) | **30 ms after `BRIDGE_ARMED_N` before `RELAY_CMD`, `BTEST_CMD` or any `CMD_*`** | 26.67 + 1 tick + 2.3 margin. `K_EN` is held until RDY, so a `RELAY_CMD` at +20 ms silently starts the coil up to 6.7 ms late and eats the 40 ms relay budget |
| (model) `LatchTiming::PROPOSED.permit_cap_us` | 250,000 | 150 ms window, 131.8-161.5 ms (§1.3) | **150,000** nominal (model the 131.8 ms minimum when checking that a sequence fits) | the drawn hardware is 150 ms, not 250 ms |
| (model) `LatchTiming::PROPOSED.time_cap_us` | 25,000 | 23.77-26.22 ms | keep 25,000; firmware must finish a sequence by **23.7 ms** after the first `CMD_LB` edge | |

### 1.2 Permit detector (`det_permit.cir`; TEST_PERMIT detector is identical)

| Behaviour | Requirement (§B8.1) | Nominal | Earliest-drop corner | Longest-consent corner | Verdict |
|---|---|---|---|---|---|
| Valid after the first edge, 1 kHz | ≤ 3 ms | 0.07 ms | 0.09 ms | 0.06 ms | met |
| Stays valid after the last edge, 1 kHz, duty 30 % / 70 % | ≥ 3 ms | 6.53 / 6.49 ms | **4.54 / 4.49 ms** | 8.18 / 8.14 ms | met (1.5 ms margin) |
| Invalid after the last edge, 5 kHz | ≤ 8 ms | 6.63 ms | 4.63 ms | **8.28 ms** | **missed by 0.28 ms** |
| 100 kHz square wave on the line | nothing | 31 mV p-p at the Schmitt input, no output | 26 mV | 37 mV | met |
| Line goes high once and stays (stuck high, or one slow edge) | not required | 3.2 ms of consent | 2.3 ms | 4.1 ms | **consent that should not exist** (bounded) |
| Single positive glitch on the line, width 5-30 µs | — | none | none | none | rejected by the filter |
| Single positive pulse 50 µs / 100 µs / 1 ms | — | 1.0 / 3.9 / 6.1 ms | 0.3 / 2.3 / 4.5 ms | 2.4 / 5.2 / 7.5 ms | **consent that should not exist** (bounded) |
| Supervisor powers up (3V3_SUP rises in 1 ms), line held low | nothing | 3.4 ms of consent | — | 4.2 ms | **consent that should not exist** (bounded) |
| 30 mV p-p, 20 kHz ripple on 3V3_SUP (PFM bursts [EST]) | clean edge | 2 transitions (one on, one off) with hysteresis 3 mV or 25 mV | — | — | met: no chatter |

Why the drop-out is slower than the capture notes' estimate (7.3 ms): after the clock stops, the pump's coupling
capacitors are still connected to the hold node through the diodes, so the hold capacitance is effectively larger
than 100 nF while it decays. Consent margins in WP-D §3.2 were computed without this.

**Can a glitch, stuck line or slow edge create consent?** Yes, in three ways, all simulated: a line that changes
level once (stuck high, or a slow edge squared by the Schmitt), a single pulse longer than about 50 µs, and the
supervisor's own power-up. Each gives at most 7.5 ms of `PERMIT_OK` (and the same on `TEST_OK`). Pulses
shorter than 30 µs and fast clocks (100 kHz) give nothing. This is inherent to a charge-pump detector that must hold
for ≥ 3 ms: one pump stroke charges the hold node above the threshold. It cannot be designed out with values alone
(raising the threshold above a single stroke's 1.3-1.6 V leaves no hold margin at the cold corner, 1.77 V). It does
not by itself reach the output stage because the sequencer's rail-ready delay is ≥ 20.7 ms after the rail rises
(§1.3), longer than any spurious consent by a factor of about 2.9 (21.6 ms against 7.5 ms); SW2a would still close and
`BRIDGE_ARMED_N` would pulse. **Firmware rule:** the main must treat a `BRIDGE_ARMED_N` low that lasts less than
10 ms as "not armed" and must not start `RELAY_CMD` from it.

Proposed hardware changes (simulated in `det_permit.cir`, tests T7/T8):

| Change | Result |
|---|---|
| R928, R929 (and R939, R940 on the test-permit detector) 71.5 kΩ → **64.9 kΩ** thin film 0.1 % | hold ≥ 4.07 ms (need ≥ 3), drop-out ≤ 7.50 ms (need ≤ 8) |

### 1.3 Every timer (`timers.cir`, `rail_up.cir`)

Times from the moment the timer's driving logic output changes. "Early" and "late" corners move every part the same
way; BAS116 leakage is taken at its 80 nA bound in the early corner.

| Timer | Requirement | Simulated band (nominal) | WP-D [CALC] band | Note |
|---|---|---|---|---|
| XFER relay transferred (12) | ≥ 10 ms | 10.42-12.17 (11.30) ms | 10.3-12.3 | met, but see §1.4 bounce |
| BTOK test relay (12) | ≥ 2 ms | **1.88**-4.58 (2.97) ms | 2.07-4.39 | **missed**: LVC input leakage (5 µA × 36 kΩ = 0.18 V) |
| BTOK proposed: R1280 → 49.9 kΩ thin film, U1241 → TLV7031 at 0.640 (56.2 k / 100 k) | ≥ 3.5 ms (test relay operate 3 ms) | 4.74-5.52 (5.13) ms | — | proposed |
| RDY after RST_N (12) | ≥ 20 ms | 20.77-24.44 (22.69) ms | 20.6-24.7 | met |
| RDY after `BRIDGE_ARMED_N` | — | 21.56-26.67 (23.92) ms | ≤ 25.6 | basis of the 30 ms wait |
| 25 ms cap (12) | 21.3-28.8 ms; must not cut ≤ 20.5 ms | 23.77-26.22 (25.14) ms | 23.6-26.5 | met |
| SEQ_HOLD backstop (12) | ≥ latest DONE + 16.2 ms = 36.7 ms | 37.96-41.96 (40.21) ms | 37.7-42.5 | met, 1.3 ms margin |
| Coil start window KWIN_N (12) | ≤ 60 ms, ≥ firmware's start | 43.40-51.20 (48.91) ms; 46.87 ms early corner with no diode leakage | 45.7-51.8 | met; latest relay start (§3) must be inside it |
| Coil cap KCAP_N (12) | ≥ latest end 79.1 ms; ≤ 100 ms | 83.65-96.38 (92.52) ms | 87.0-97.6 | met, 4.6 ms margin at the leakage bound |
| Permit window (09) | contains the latest start | 131.8-161.5 (150.6) ms; 140.3 early with no leakage | 138.4-162.7 | met (BAS116 leakage through 1.47 MΩ costs up to 8.5 ms) |
| Re-arm (09) | ≥ 50 ms | 51.24-60.34 (56.28) ms | 51.7-60.8 | met |

### 1.4 Two races found

| Race | Result | Proposed change |
|---|---|---|
| **Relay aux-contact bounce vs XFER** (`races.cir` A). The contact leaves rest, re-touches for 1-200 µs 8 ms later, then leaves for good. D1212 empties C1253 only down to the diode's knee, so the count restarts from a few hundred mV | XFER rises **4.1-8.4 ms** after the last release (rule ≥ 10 ms) | 10 kΩ + 1 nF (10 µs) on `K_REST_N` before U1210, and a 2N7002 with 10 Ω drain resistor across C1253, gate from `K_AT_REST`. Simulated: **11.0-11.3 ms** after the last release for bounces ≥ 10 µs; sub-10 µs blips are ignored. Firmware: restart its own count on every low sample of `K_REST_N` |
| **Window start lost at re-arm** (`races.cir` B). When the first consent returns, REARM_N is released only **15.7 µs** later; a `CHAIN` rising edge inside those 15.7 µs is ignored by WSTART (still cleared), so the 150 ms window does not start for that attempt and SW4 stays on until consent is withdrawn | happens only if `SHOCK_REQ` and the permit become true within ~16 µs of each other | Hardware: delay the WSTART clock by 10 kΩ / 10 nF C0G (100 µs) + SN74LVC1G17 between `CHAIN` and U928 CLK [CALC: 100 µs ≫ 15.7 µs; not simulated]. Firmware alternative: the supervisor starts `PERMIT_CLK` ≥ 1 ms after it learns the main has raised `SHOCK_REQ` |

---

## 2. The latched time cap (question 2)

Simulated spread of the drawn cap (R1288 249 kΩ thin film, C1264 100 nF C0G ±2 %, TLV7031 at 0.6341):
**23.77-26.22 ms, i.e. -5.4 % / +4.3 % about 25.14 ms nominal** (`timers.cir`). The spread is dominated by the
capacitor, the comparator offset and hysteresis, and diode leakage at its bound, not by the resistors.

Longest legitimate pulse: the aed-waveform design table's longest shock is 15.5 ms plus the 0.5 ms gap = **16.0 ms**
(`interlock.rs` comment; HARDWARE_DESIGN §B13.3 rule 4); the waveform configuration allows up to **20.0 ms**
(`aed-waveform/src/config.rs` `max_total`). The cap starts at the first `CMD_LB` edge and `DONE` latches 9-20 µs
after phase 2 ends, so a legitimate sequence needs the cap to exceed the pulse length by ≥ 0.05 ms plus margin
(1 ms used here).

| R1288 | C1264 band | Simulated cap | Never cuts 16.0 ms (+1 ms)? | Never cuts 20.0 ms? |
|---|---|---|---|---|
| 249 k (as drawn) | ±2.2 % | 23.77-26.22 ms | yes | yes (3.7 ms margin) |
| 249 k | ±1.2 % (±1 % part) | 24.01-25.96 ms | yes | yes |
| **178 k** | ±2.2 % | **17.07-18.75 ms** | **yes (1.0 ms margin)** | **no** |
| 178 k | ±1.2 % | 17.25-18.56 ms | yes | no |
| 162 k | ±2.2 % | 15.55-17.06 ms | no | no |
| 147 k | ±2.2 % | 14.13-15.48 ms | no | no |

Conclusions:
- **A cap of 12 ms is not possible** without shortening the waveform table: it would cut the 15.5 ms shocks.
- **The shortest cap that never cuts the design table's 16.0 ms is ≈ 18 ms (R1288 = 178 kΩ)** with the parts
  already drawn. Tighter capacitors (±1 %) gain only ≈ 0.2 ms; the comparator and leakage terms remain.
- **Any cap below ≈ 21.2 ms requires the firmware ceiling `max_total` (aed-waveform config, 20,000 µs) to be
  lowered to the cap's minimum minus 1 ms** (for 178 kΩ: 16,000 µs), and `LatchTiming.time_cap_us` to match.
- **The 40 ms `SEQ_HOLD` backstop shares this RC.** With R1288 = 178 kΩ it would fall to ≈ 27-30 ms, before the
  planned hold release (latest DONE + 16.2 ms ≈ 32.3 ms). Move its threshold up: R1289 31.6 kΩ → **19.6 kΩ**,
  R1290 26.1 kΩ → **38.3 kΩ** (R1291 100 kΩ unchanged) gives 0.633 / 0.876, i.e. cap ≈ 17.9 ms and backstop
  ≈ 37 ms nominal [CALC from the same RC; not simulated].
- The coil timings then gain margin (latest coil-off falls by ≈ 7 ms).

---

## 3. Patient-relay drive and supply rails (I-15, `relay_supply.cir`)

### 3.1 As drawn

Battery peak current includes ≈ 0.5 A for the other loads. "Transfer" = coil reaches pick-up + 10 ms mechanical
[EST].

| Battery (open-circuit, per-cell resistance) | Peak battery current | Above 2 A for | VBAT_SYS min | 5V0_SYS min | BRIDGE_5V min | Contacts transfer after `K_EN` |
|---|---|---|---|---|---|---|
| 8.4 V, 20 mΩ (full, warm) | 9.9 A | 15 ms | 7.76 V | 5.08 V | 5.00 V | 23.7 ms |
| 7.4 V, 20 mΩ | 8.7 A | 20 ms | 6.83 V | 5.08 V | 5.00 V | 25.3 ms |
| 7.4 V, 80 mΩ (cold) | 8.7 A | 33 ms | 5.79 V | 5.08 V | 5.00 V | 27.3 ms |
| 6.0 V, 20 mΩ (nearly empty) | 8.8 A | 33 ms | 5.43 V | 5.08 V | 5.00 V | 28.3 ms |
| **6.0 V, 80 mΩ (nearly empty, cold)** | **8.9 A** | **33 ms** | **4.35 V** | **4.24 V** | **4.16 V** | **31.8 ms** |

- The surge is the reservoir filling (Q1401 slew ≈ 1.1 V/ms into 6.6 mF, then the boost at its 8.1 A limit), as WP-H
  estimated; simulation confirms 8.7-9.9 A.
- In the cold, nearly-empty case 5V0_SYS falls below the 4.9 V that the charger controller needs (§B1.6) and
  BRIDGE_5V below the sequencer's 4.39-4.60 V rail-collapse trip. Before a sequence starts this only blocks the start
  while the sag lasts; during a sequence it would end it as a fault. BRIDGE_3V3 should still regulate (4.16 V - 0.35 V
  SS14 = 3.8 V at the LDO input, above its 3.4 V need [CALC, BRIDGE_3V3 not in this netlist]), so the sequencer stays above
  the reset threshold in this case.
- **Timing against the 40 ms firmware limit:** transfer 23.7-31.8 ms + the 14 ms wait of §1.1 = **37.7-45.8 ms**:
  over 40 ms in the cold / low cases. Against the coil start window: transfer + XFER max 12.2 ms = up to 44.0 ms,
  above the window's 43.4 ms minimum at the leakage bound (46.9 ms without). The relay mechanics are an estimate,
  but the margin is gone either way.

### 3.2 Proposed: pre-charge the reservoir, limit the boost

A reservoir that is already full when `K_EN` rises removes the surge. Nothing can usefully protect the logic rails
locally instead: holding 5V0_SYS's 0.47 A input through a 15-30 ms, 1.5 V dip would need ≈ 10 mF on VIN_5 [CALC].

| Proposed configuration (simulated) | Peak battery current | VBAT_SYS min | 5V0_SYS min | Transfer after `K_EN` |
|---|---|---|---|---|
| Reservoir pre-charged; `K_EN` starts only the DRV110 (mode 1); 8.4 → 6.0 V, 20 → 80 mΩ | 1.7-2.6 A | ≥ 5.53 V | ≥ 4.99 V | **15.9 ms** in every case |
| The pre-charge itself: C1401 68 nF → **680 nF**, R1406 127 kΩ → **515 kΩ** (ILIM ≈ 2.0 A) | 2.5-2.6 A (≈ 2.0 A for the coil supply) | ≥ 5.51 V | 5.08 V | reservoir at 11 V after 74-99 ms |

Changes this needs (sheet 14 and the contract):
1. A separate enable for Q1401 and the boost (new contract net, e.g. `K_SUPPLY_EN` from the main controller, 100 kΩ
   pull-down), raised when the shock is being prepared (at charge start, ≥ 0.2 s before `RELAY_CMD`) and lowered
   with `SHOCK_REQ` withdrawal or any abort; `K_EN` keeps enabling only the DRV110 (and so the 92 ms coil cap and
   end-of-sequence coil-off still act directly on the coil, CI-8). Standby draw stays zero.
2. C1401 68 nF → 680 nF X7R 50 V; R1406 127 kΩ → 515 kΩ (ILIM ≈ 2.0 A); the reservoir stays 6.6 mF.
3. Firmware: with the pre-charge, transfer ≈ 16 ms + 14 ms = 30 ms < 40 ms; keep 40 ms as the abort limit.
   Needs an owner decision (new net and a firmware step).

---

## 4. Battery protector recovery and the battery load test (I-10, I-25, `battery.cir`)

### 4.1 Every load on the battery rail (from the pin tables)

| Net | Part | What it is | DC resistance seen at ≈ 1 V with everything off |
|---|---|---|---|
| VBAT_PACK | R107 1 kΩ | PCKP sense resistor (the test current flows through it) | series, not a load |
| VBAT_PACK | C210 10 µF; U201 BQ25887 BAT pins | dock charger | capacitor; IC leakage only [EST ≤ 1 µA] |
| VBAT_PACK → VBAT_SYS | F101 25 A | fuse, 2 mΩ | series |
| VBAT_SYS | C110 10 µF | bulk | ∞ |
| VBAT_SYS | R102 + R103 0.80 Ω via Q103 | load test | Q103 off: ≥ 1 MΩ [EST, IDSS ≤ 1 µA] |
| VBAT_SYS | R316 4.7 MΩ + R317 3.3 MΩ | U304 EN divider | **8.0 MΩ** |
| VBAT_SYS | U304 TPS70933 IN, C312 1 µF | always-on LDO | IC leakage [EST ≤ 1 µA] |
| VIN_M / VIN_S (F301 / F302) | U301 / U302 TPS629210, C301 / C304 4.7 µF | 3V3 bucks, EN low | IC leakage [EST ≤ 1 µA each] |
| VIN_5 (F303) | U303 TPS62130, C307 10 µF, C308 100 nF | 5 V buck, EN low | IC leakage [EST ≤ 1 µA] |
| VTRANS_IN (F1001, L1001) | U1002 TPS25982, C1001 + C1002 20 µF, SMBJ12A | charger eFuse, EN low | leakage [EST ≤ 2 µA] |
| VBAT_SYS (sheet 14) | R1401 100 kΩ → K_PG → C1401 / R1402 47 kΩ → Q1402 (off); Q1401 source (off) | coil-supply switch | **no DC path** (capacitor or an off transistor in series); 147 kΩ only if Q1402 is shorted |

Every drawn load meets "no resistive load under 100 kΩ"; the lowest real DC path is the 8.0 MΩ divider. Total
sub-threshold leakage was modelled as 8 µA [EST].

### 4.2 What steady load prevents recovery

| Test current | Extra resistive load on VBAT_SYS | PCKP steady-state voltage | Releases (> 1 V)? |
|---|---|---|---|
| 30 µA | none / 1 MΩ / 100 kΩ / 68 kΩ / 47 kΩ | 2.86 / 2.81 / 2.20 / 1.51 / 1.06 V | yes |
| 30 µA | 33 kΩ / 22 kΩ / 10 kΩ | 0.75 / 0.51 / 0.27 V | **no** |
| 15 µA (what-if) | none / 1 MΩ | 2.72 / 2.63 V | yes |
| 15 µA (what-if) | 100 kΩ / 68 kΩ / 47 kΩ | 0.71 / 0.49 / 0.34 V | **no** |

Release time with the rail starting discharged (≈ 60 µF), 30 µA: 2.5 s with no extra load, 3.4 s with 100 kΩ, 4.2 s
with 68 kΩ, 7.9 s with 47 kΩ; 15 µA, no extra load: 7.7 s. (The netlist now runs 30 s; `out/battery.log` has the
rerun.)

- **With 30 µA, any steady load below ≈ 45 kΩ prevents recovery** (1 V / (30 µA - 8 µA leakage)). The 100 kΩ rule
  keeps a 2.2× margin. **The rule is confirmed only if the MAX17320's minimum test current is ≥ ≈ 20 µA**; its
  tolerance was not found. With 15 µA even 100 kΩ blocks release. Action: read the MAX17320 PCKP current limits;
  if the minimum is below 20 µA, tighten the rule to 1 V / (I_min - 8 µA) / 2.
- A battery wiring short that persists keeps the device dark whatever the loads (the protector cannot tell a short
  from a load); only removing the short or docking releases it. That part of I-10 is unchanged.

### 4.3 Load test driven by a toggling enable

| BAT_LOAD_EN | As drawn: average power in R102 + R103 / in Q103 | Proposed: same |
|---|---|---|
| stuck high (one edge) | on for 179.7 ms nominal (design band 124-220 ms) | on for 255.5 ms nominal (too long, see below) |
| 10 Hz, duty 25 % / 50 % | 14.5 / 14.9 W; Q103 0.05 / 0.29 W | 1.8 / 0 W; Q103 0.07 / 0 W |
| 100 Hz, duty 25 % / 50 % | 14.5 / **22.7 W**; Q103 0.06 / 1.5 W | 2.2 / 0 W; Q103 0.08 / 0 W |
| 10 Hz, duty 75 % (earlier run) | 0 W (the node never rises above the threshold) | — |

As drawn, a pin toggled at 10-100 Hz with duty up to 50 % keeps the 86 W load running at 14-23 W average,
continuously (1 kHz was not run: too slow to simulate; the mechanism does not depend on rate once the period is far
below the 260 ms coupling time constant). Each PWR263S is rated 35 W only on an infinite heatsink; on board copper this would
overheat them within tens of seconds [EST], and it drains the battery.

**Proposed** (simulated): put a BAS116 in series with R111 (anode at LT_NODE), so the coupling capacitor recharges
only through a new **R117 4.7 MΩ** from LT_NODE to GND (τ ≈ 10 s), and raise **R112 10 kΩ → 100 kΩ** (LT_NODE now
sits at up to -3.3 V for seconds; 33 µA into the comparator input). Simulated: toggling now gives 1.8-2.2 W average (7-10 times less) and
50 % duty gives nothing. Side effect: the diode stops the node decaying fully, so a single stuck-high pulse lasts
255 ms nominal instead of 180 ms, beyond the 220 ms limit and the resistors' graphed pulse rating. Lower R111
118 kΩ → **78.7 kΩ** to bring it back to ≈ 170 ms nominal [CALC: on-time scales with R111; not simulated]. TLV7031 behaviour with its input held below V- through 100 kΩ is not specified
[UNVERIFIED]; the output stays low (load off) in the model.

---

## 5. Sense channels (I-21, I-23, I-16; `sense.cir`)

### 5.1 I_SHOCK headroom at the main controller

| | As drawn (R1355 10.0 k / R1356 20.0 k, 26.7 mV/A) | Proposed: **R1355 13.3 kΩ** (24.0 mV/A) |
|---|---|---|
| Signal at 80 A (largest planned), high-gain corner | 2.21 V | 2.00 V |
| Signal at 98 A (over-current trip maximum) | 2.71 V (0.55 V below VDDA min 3.26 V) | 2.45 V |
| INA240 saturated (BRIDGE_5V 5.25 V, divider high) | **3.55 V: 0.29 V above VDDA min** | **3.20 V: below 3.26 V** |
| Sensitivity, low-gain corner | 25.7 mV/A | 23.1 mV/A |
| Self-test current 0.29 A | 7.4 mV | 6.7 mV |

The real headroom problem is not the shock current (98 A leaves 0.55 V) but the amplifier's saturated output, which
can drive the ADC pin above its supply through R423 1 kΩ. Proposed: **R1355 10.0 kΩ → 13.3 kΩ (1 %)**; the
idle-current comparator then needs its threshold scaled by the same 0.9 (5.0 A → 120 mV): its divider top
237 kΩ → **267 kΩ** (119 mV = 4.96 A) [CALC]. No offset change is proposed: the INA240's own output floor near 0 V
(not modelled, [UNVERIFIED] figure) matters only for the 0.29 A self-test current; firmware should treat
"≥ 0.1 A seen" as current present.

### 5.2 Open main voltage-tap wire

| | As drawn | Proposed: **R718 1 GΩ from VREF_M (2.5 V) to VCAPM_IN** |
|---|---|---|
| Wire opens with the node at 0 V, OPA333 bias current flowing out of the node (1 nA) | node stays at 0 V; **LOWV_OK stays high ("below 60 V") indefinitely** | LOWV_OK goes low after 0.37 s; node → 0.58 V (= 580 V reading) after 5 s and rising |
| Same, bias current flowing into the node | LOWV_OK low after 0.55 s (only because of the bias current's sign) | LOWV_OK low after 0.16 s |
| Normal reading error added (wire intact) | — | +0.22 V at 0-50 V, -0.07 V at 1,950 V (a fixed offset, removed by the factory calibration) |

A 1 GΩ part fails safe in both directions: open = today's behaviour; short = reads 2,500 V (refuses). Board leakage
(100 GΩ assumed) changes the result by ≈ 1 %. The resistor's availability (1 GΩ, 0805/1206) was not checked.

### 5.3 Main divider drift budget (I-16)

Allowance at 1,950 V: 26.5 V = 1.36 %. Terms other than the string's temperature and life (WP-C §4): 0.18 % +
0.61 V ≈ 0.21 %. Left for temperature + life: **1.15 %**.

| String | Temperature error (-20 / +70 °C, calibrated at 25 °C) | Largest life drift that still fits |
|---|---|---|
| HVCR2512 as drawn (±100 ppm/K vs bottom leg 25 ppm/K, opposite signs) | ±0.56 % | **0.59 %** (simulated: 0.6 % gives 1.152 %) |
| thin-film string (25 ppm/K both) | ±0.22 % | **≈ 0.92 %** (0.9 % gives 1.114 %) |

This confirms WP-C's 0.6 % figure. Since no life-drift figure exists for HVCR, the recommendation stays with WP-C
doubt 1: use a thin-film high-voltage string on the main chain too (owner question 4), which gives a 0.9 % life
allowance.

---

## 6. Compression (impedance) sensing noise (I-14, `imp_noise.cir`)

| Configuration (patient 25 / 175 Ω) | Signal at the ADS1292R per ohm | Noise, 0.5-10 Hz | Equivalent impedance noise at 80 µA / 100 µA | Requirement |
|---|---|---|---|---|
| As drawn | 13.2 / 12.3 µV per Ω per 80 µA (**≈ 0.16 of the pad voltage**) | 0.41 µV RMS | **31-34 / 25-27 mΩ RMS** | < 10 mΩ: **not met** |
| Proposed: separate 51 kΩ DPCR sense resistors for the ECG branch | ≈ 0.9 of the pad voltage | 0.44 µV RMS | **5.8-6.2 / 4.6-5.0 mΩ RMS** | met |

Why: C1505 (470 pF, the ECG differential filter) is connected through R1509/R1510 10 kΩ to the same clamp nodes
that feed the impedance inputs. At 64 kHz it is 5.3 kΩ across them, against 2 × 51 kΩ of source resistance, so it
shunts ≈ 85 % of the 64 kHz signal. WP-H's 5.6 mΩ estimate did not include it. Proposed: add **R1528, R1529 51 kΩ
DPCR** from PT_FE_A / PT_FE_B to new nodes feeding R1509 / R1510 (ECG branch), leaving R1507/R1508 for the
impedance branch only. Cost: two pulse-rated resistors; the ECG branch's filter corner is unchanged.

**Charger interference.** A charger node with a harmonic of V volts near 64 kHz, coupled through C into one pad,
gives an apparent impedance error of **0.46 Ω per V·pF at 175 Ω** (0.063 Ω per V·pF at 25 Ω), whenever that
harmonic is within the 10 Hz demodulation bandwidth of the carrier (or its odd harmonics). Keeping that below
10 mΩ needs V × C ≤ 0.022 V·pF: for a 100 V harmonic on the transformer's secondary, C ≤ 0.2 fF, which no layout
achieves. The boundary-mode charger sweeps up to ≈ 43 kHz and its harmonics cross 64 kHz several times per charge
(§B3.7), so each crossing produces a burst of artefact much larger than 10 mΩ. **The 10 mΩ limit does not hold
while the charger runs.** Options: firmware blanks compression readings while `CHG_RUN` is high (simplest); or pauses
the charger's CHARGE pin during compression measurement (≈ 20 % slower charging, §B3.7); or a fixed-frequency
charger synchronised so its harmonics land exactly on the carrier (they then demodulate to a constant, removed by the
0.5 Hz high-pass) [idea, not simulated]. The coupling itself can only be measured.

---

## 7. Recommended changes in one place

Hardware (sheet, parts, values):
1. 09: R928, R929, R939, R940 71.5 kΩ → **64.9 kΩ** thin film 0.1 % 25 ppm/K (detector drop-out ≤ 8 ms).
2. 09: **10 kΩ + 10 nF C0G + SN74LVC1G17** between `CHAIN` and U928 CLK (window-start race) [CALC].
3. 12: R1280 → **49.9 kΩ** thin film; U1241 → **TLV7031** with **56.2 kΩ / 100 kΩ** thin-film threshold from
   BRIDGE_3V3 (BTOK 4.7-5.5 ms ≥ test relay operate).
4. 12/14: **10 kΩ + 1 nF** filter on `K_REST_N` before U1210; **2N7002 + 10 Ω** across C1253 gated by `K_AT_REST`.
5. 12 (only if a shorter cap is chosen): R1288 249 kΩ → **178 kΩ**, R1289 → **19.6 kΩ**, R1290 → **38.3 kΩ**.
6. 14 + contract: separate coil-supply enable; C1401 → **680 nF**; R1406 → **515 kΩ**.
7. 01: **BAS116 in series with R111**, R111 → **78.7 kΩ** [CALC], add **R117 4.7 MΩ** LT_NODE → GND, R112 → **100 kΩ**.
8. 13: R1355 → **13.3 kΩ**; idle-threshold divider top → **267 kΩ**.
9. 07: add **R718 1 GΩ** VREF_M → VCAPM_IN.
10. 11/07: thin-film high-voltage string on the main chain (owner question 4).
11. 15: separate **51 kΩ DPCR** sense resistors for the ECG branch (R1528, R1529).
12. 01 rule: keep "no resistive load < 100 kΩ on VBAT_SYS" once the MAX17320 PCKP minimum current is confirmed
    ≥ 20 µA.

Firmware (to be made by someone else):
1. F-4: wait **14 ms** after `K_REST_N` has been continuously high (restart on any low sample); abort at 40 ms.
2. F-4: **30 ms** after `BRIDGE_ARMED_N` low before `RELAY_CMD`, `BTEST_CMD` or any `CMD_*`.
3. F-4.6: consent absent **≥ 75 ms** between attempts.
4. F-8.3: **6 ms** after `BTEST_CMD` (7 ms with hardware change 3).
5. `interlock.rs` `LatchTiming::PROPOSED`: `rearm_us` 20,000 → **60,500**; `permit_cap_us` 250,000 → **150,000**.
6. Ignore `BRIDGE_ARMED_N` lows shorter than 10 ms.
7. If the cap is shortened (hardware 5): `aed-waveform` `max_total` 20,000 → **16,000 µs** and `time_cap_us` →
   **18,000**.
8. Blank compression readings while the charger runs.

## 8. What contradicts the existing documents

| Document says | Simulation says |
|---|---|
| WP-D §3.2: detector drop-out ≤ 7.32 ms; single transition gives 3-7 ms | ≤ 8.28 ms (misses 8 ms); single transition 2.3-4.1 ms; a single 50 µs-1 ms pulse up to 7.5 ms; supervisor power-up 3.4-4.2 ms |
| WP-D §3.3: BTOK 2.07-4.39 ms ≥ 2 ms | 1.88-4.58 ms (LVC input leakage); and below the test relay's 3 ms operate time |
| WP-D §3.3: XFER guarantees "left rest ≥ 10 ms" | not after a contact bounce (4.1-8.4 ms) |
| WP-D §3.3: permit window 138.4-162.7 ms | 131.8-161.5 ms with BAS116 leakage at its bound |
| WP-H §3.5: transfer ≈ +30-35 ms with ≈ 5 ms margin to 40 ms | 23.7-31.8 ms to contact transfer, then a 14 ms wait: up to 45.8 ms; logic rails brown out with a cold, nearly empty battery |
| WP-H §3.4: impedance noise 5.6 mΩ | 25-34 mΩ as drawn (C1505 shunts the 64 kHz signal) |
| `interlock.rs`: re-arm 20 ms, permit cap 250 ms | hardware: 51-60 ms, 132-162 ms |
| WP-B C-4: "a -3 % regulator leaves 0 V headroom" | the shock signal has 0.55 V headroom; the problem is the saturated amplifier (0.29 V over) |

## 9. What cannot be settled without measurement

- The relay: coil inductance, pick-up current, mechanical operate time and aux-contact bounce (no relay is chosen).
- Battery DC resistance at -20 °C and low charge, and the MAX17320's PCKP test-current tolerance.
- Real stray coupling between the charger and the pad nets, and the charger's harmonic amplitudes near 64 kHz.
- The ADS1292R's own respiration-channel noise in this circuit (the 0.13 µV/√Hz figure is an estimate).
- BAS116 leakage at 70 °C and a few volts (the 80 nA bound is pessimistic; with 5 nA the window and coil timers lose
  < 1 ms).
- TPS22918 behaviour with VOUT above VIN, TLV7031 with an input below V-, INA240 output floor near 0 V.
- 3V3_SUP's real ripple waveform at light load.
