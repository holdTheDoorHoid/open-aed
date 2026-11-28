# Change order for capture: design revision 0.3, contract issue 3

**NOT FOR CONSTRUCTION — unreviewed design.** Written 27 November 2026 with AI assistance by the design authority
for the paper design. Nothing here has been built or measured; simulated figures are marked [SIM] and come from
hand-built models. Nothing here is an instruction to build, power or test anything, and nothing in this project is
safe or ready for use on a person or animal.

Sources: `docs/HARDWARE_DESIGN.md` v0.3 (§V3 maps every issue to its fix), `contract.md` issue 3, the integration
issues I-1…I-25 (`00-integration.md`), the simulation notes `sim/bridge/notes.md` (**BR**), `sim/charger-storage/notes.md`
(**CS**), `sim/logic-relay/notes.md` (**LR**) and `sim/rev3/notes.md` (**R3**), and decisions D-62…D-67 (provisional).

## How to use this document (plain language)

Each of the eight work packages below lists exactly what to change on its own sheets: which part, from what to
what, which new parts and nets, and why. The packages can work in parallel: every wire that crosses between
sheets is defined in `contract.md` issue 3 (§0 lists the new and changed ones), so nobody needs to agree anything
else. Where a part number could not be confirmed it says so and gives the required ratings instead. New reference
designators follow contract §2 (the next free numbers in each block, or the sheet's overflow block NN01 + 5000).

Every new internal node gets its default resistor to its blocking state (DEP-12 of WP-D). Every new part carries
`MPN`, `Manufacturer`, `Datasheet`, `DocRef` and, where a rating is not confirmed, `Verify` (contract R10).
**Do not edit another package's sheet; do not edit `hierarchy.toml` or `00_root.toml` (no pin lists change).**

---

## WP-A power (sheets 01, 02, 03)

| # | Sheet | Change | Values / parts | Reason |
|---|---|---|---|---|
| A-1 | 01, 02 | Rename local `BAT_MID` (01) and `BAT_MID_02` (02) to the global net **`VBAT_MID`**; add it to both sheets' `global` lists. U201 MID (via 300 Ω) and CBSET (via R208 100 Ω) now join the pack mid-point | no new parts | I-1, WP-A C-1 |
| A-2 | 01 | Battery-load timer: add **D101 BAS116LT1G** in series with R111, anode at `LT_NODE`, cathode at new local node `LT_R`; R111 now `LT_R`–GND and **118 kΩ → 78.7 kΩ** 1 %; add **R117 4.7 MΩ** 1 % `LT_NODE`–GND; **R112 10 kΩ → 100 kΩ** | onsemi BAS116LT1G (already used on 12) | I-25; LR §4.3 (toggling 10–100 Hz drops from 14.5–22.7 W to 1.8–2.2 W [SIM]); R111 value [CALC: on-time ∝ R111, 255 ms × 78.7/118 = 170 ms; **not simulated**, second round] |
| A-3 | 01 | Note on the sheet: "`VBAT_SYS`: no steady resistive load below 1 MΩ from any sheet (contract §4, D-66)". Lowest drawn load is R316 + R317 = 8.0 MΩ on 03: complies | — | I-10, D-66 |
| A-4 | 01 (config record) | MAX17320 non-volatile settings: `nConfig.COMMSH = 0` (WP-A C-4); discharge over-current threshold ≥ 35 A **or** delay ≥ 1 ms so the charger eFuse's 30 A / 0.48 ms limit acts first (WP-E C6); disable one of the two cell balancers (MAX17320 or BQ25887; recommend keeping the charger's, setting name to be read from the MAX17320 datasheet) | configuration, not parts | I-19, WP-E C6, WP-A doubt 1 |
| A-5 | 03 | No change. `3V3_MAIN` "used on" now lists 01 and 02 (contract §4) | — | WP-A C-2 |

Open for WP-A (record, do not design): the MAX17320 datasheet (Rev 5, 6/20) gives the PCKP test current only as
30 µA typical and the release threshold as 1 V in one place and 1.5 V in another; the 1 MΩ rule releases down to
≈ 10 µA [CALC: 8 µA leakage + 1.5 V / (1 MΩ ∥ 8 MΩ = 0.89 MΩ) = 9.7 µA]. Ask ADI for the minimum.

## WP-B controllers (sheets 04, 05)

| # | Sheet | Change | Values / parts | Reason |
|---|---|---|---|---|
| B-1 | 04 | New output pin **`K_SUPPLY_EN`** (any spare GPIO, push-pull, reset state input/high-Z; the 100 kΩ default is on 14) | — | I-15, LR §3.2 |
| B-2 | 04 | Two new input pins **`HOLD_RB_A2_N`, `HOLD_RB_B2_N`**, each with **100 kΩ pull-up to 3V3_MAIN** on 04 (new R431, R432) | 100 kΩ 1 % 0402 | I-4; self-test proves all four sensors |
| B-3 | 05 | Two new output pins **`OVP_RST`** and **`OVP_TEST`** (spare GPIOs, push-pull on 3V3_SUP; defaults are on 08) | — | I-22; self-test T-OVP |
| B-4 | 04, 05 | Update the pin tables in WP-B §4/§5: main 70 + 3 = **73 signal pins + 6 = 79 of 114**; supervisor 22 + 2 = **24 + 4 = 28 of 52** | — | WP-B C-1 |
| B-5 | 04 | `I_SHOCK` anti-alias RC unchanged; note the new scale **24.0 mV/A** (≤ 3.20 V with the amplifier saturated, below VDDA min 3.26 V) | — | I-21, LR §5.1 |
| B-6 | 05 | Piezo LS501: unchanged (Q-55 open) | — | I-12 |

## WP-C ui + sense (sheets 06, 07, 08)

| # | Sheet | Change | Values / parts | Reason |
|---|---|---|---|---|
| C-1 | 07 | Add **R718 1 GΩ** 1 %, 1206, `VREF_M` → `VCAPM_IN`. Part: Vishay **CRHV1206AF1G00FKE5** (CRHV datasheet doc 68002 rev 18-Apr-17: 1 GΩ in 1206, 1,500 V; read by a research agent 27 Nov 2026) | +0.22 V reading offset at 0–50 V, −0.07 V at 1,950 V, removed by calibration [SIM, LR §5.2] | I-23: an open tap wire now drives `LOWV_OK` low in 0.16–0.37 s instead of never |
| C-2 | 08 | OVP comparator re-centred and **latching**: R805 36.5 kΩ → **100 kΩ** 0.1 % (`VCAPS_BUF`–`OVP_SENSE`); R806 stays 100 kΩ 0.1 % (`OVP_TRIP`–`OVP_SENSE`); R807 9.09 kΩ → **21.5 kΩ** 0.1 % (`VREF_S`–`OVP_REF`); R808 15.6 kΩ unchanged | OVP_REF = 2.5 V × 15.6 / 37.1 = 1.0512 V; trip at tap = 2 × 1.0512 = 2.1024 V = **2,107 V** (ratio 1:1,002.0) [CALC]; band ≈ **2,073–2,142 V** [CS Q4 band 2,076–2,145 V for 2,110 V nominal, shifted −3 V]; latched for any tap ≥ 0 V while `OVP_TRIP` ≥ 2.1 V [SIM CS Q4] | I-2, I-22 |
| C-3 | 08 | `OVP_RST` reset: **C812 100 nF** X7R (`OVP_RST`–`OVP_RG`), **R809 100 kΩ** (`OVP_RG`–GND), **Q801 BSS138** (G `OVP_RG`, S GND, D `OVP_RSD`), **R810 1 kΩ** (`OVP_RSD`–`OVP_SENSE`), **R811 100 kΩ** `OVP_RST`–GND (contract default) | edge-only reset; a reset line stuck high does not hold the latch clear [SIM CS Q4] | I-22 |
| C-4 | 08 | `OVP_TEST` injection: **R812 22 kΩ** (`OVP_TEST`–`OVP_TI`), **D801 BAS116LT1G** (A `OVP_TI`, K `OVP_SENSE`), **R813 100 kΩ** `OVP_TEST`–GND | test high: OVP_SENSE = (3.3 − 0.6) × 50 k / 72 k = 1.88 V > 1.05 V → trips at 0 V capacitor; with Q801 shorted: 2.7 × 0.98 k / 22.98 k = 0.12 V → no trip, so the self-test sees it [CALC]; test low: BAS116 reverse leakage ≤ 5 nA (25 °C) × 50 kΩ = 0.25 mV = 0.5 V of capacitor reading [CALC; 80 nA hot bound → 8 V] | closes CS's latent "reset switch shorted" fault; self-test T-OVP |
| C-5 | 08 | Sheet notes: release is now by `OVP_RST` edge or `3V3_SUP` loss only; the 2,160 V / "releases near 1,000 V" text is obsolete | — | — |
| C-6 | 06 | No change (speaker Q-55 open). Remove 06 from the `3V3_MAIN` users (contract §4) | — | WP-C C-4 |
| C-7 | 07 | Main string stays HVCR2512 as drawn; the thin-film alternative (life drift ≤ 0.92 % instead of ≤ 0.59 %) waits for owner question 4 (HARDWARE_DESIGN §17) | — | I-16; LR §5.3 |

## WP-D shock logic (sheets 09, 12)

### Sheet 09

| # | Change | Values / parts | Reason |
|---|---|---|---|
| D-1 | Permit and test-permit detector bleeds **R928, R929, R939, R940 71.5 kΩ → 64.9 kΩ** 0.1 % 25 ppm/K thin film | hold ≥ 4.07 ms (need ≥ 3), drop-out ≤ 7.50 ms (need ≤ 8) [SIM LR §1.2] | I-9; drop-out was 8.28 ms |
| D-2 | `HOLD_A_HELD_N`, `HOLD_B_HELD_N` pull-ups **R945, R946 10 kΩ → 47 kΩ** (R947, R948 1 MΩ backups stay) | Darlington sink needed 0.10 mA instead of 0.46 mA; dark current 20 µA (85 °C) × 44.9 kΩ = 0.9 V drop, still "not held" [CALC] | cold aged-sensor margin [SIM R3 §2] |
| D-3 | New inputs **`HOLD_A_HELD2_N`, `HOLD_B_HELD2_N`**: pull-ups **R973, R974 47 kΩ** and backups **R975, R976 1 MΩ** to 5V0_SYS | as D-2 | I-4 |
| D-4 | `HOLD_OK` over four lines: U914 (SN74LVC1G02) now `HOLDA_OK` = NOR(`HOLD_A_HELD_N`, `HOLD_A_HELD2_N`); new **U933 SN74LVC1G02** `HOLDB_OK` = NOR(`HOLD_B_HELD_N`, `HOLD_B_HELD2_N`); new **U934 SN74LVC1G08** `HOLD_OK` = `HOLDA_OK` AND `HOLDB_OK`; defaults **R977 100 kΩ** `HOLDA_OK`–GND, **R978 100 kΩ** `HOLDB_OK`–GND (R904 stays on `HOLD_OK`). All on 5V0_SYS; the Darlington low level ≤ 1.0 V meets LVC's V_IL 1.35 V at 4.5 V [TI SCES213Y as used by WP-D] | — | I-4 |
| D-5 | New open-drain readbacks: **U935 SN74LVC2G07** inputs `HOLD_A_HELD2_N`, `HOLD_B_HELD2_N`, outputs **`HOLD_RB_A2_N`, `HOLD_RB_B2_N`** (pull-ups on 04) | — | self-test of all four sensors |
| D-6 | Window-start race: insert **R979 10 kΩ** (`CHAIN`–`CHAIN_RC`), **C960 10 nF C0G** (`CHAIN_RC`–GND) and **U936 SN74LVC1G17** (`CHAIN_RC` → `CHAIN_D`, on 5V0_SYS) and clock U928 from `CHAIN_D` instead of `CHAIN`; **R980 100 kΩ** `CHAIN_D`–GND | 100 µs delay ≫ the 15.7 µs re-arm release [CALC; **not simulated**, second round] | LR §1.4 race B |

### Sheet 12

| # | Change | Values / parts | Reason |
|---|---|---|---|
| D-7 | **High sides off at once on an over-current or idle fault (HSK latch).** New **U1263 SN74LVC1G32** `OCI` = `OC_F` OR `IDLE_F`; **U1264 SN74LVC1G00** `HSK_PRE_N` = NAND(`OCI`, `RDY`); **U1265 SN74LVC1G74DCTR** (PRE_N `HSK_PRE_N`, CLR_N `RST_N`, D GND, CLK GND, Q_N `HSK_N`); rename U1222/U1223 outputs to `G_HA_P`/`G_HB_P`; new **U1266, U1267 SN74LVC1G08**: `G_HA_I` = `G_HA_P` AND `HSK_N`, `G_HB_I` = `G_HB_P` AND `HSK_N` (R1252/R1253 and the lockout timers stay on `G_HA_I`/`G_HB_I`); **R6209 100 kΩ** `HSK_N`–GND | path `OC_DET_N`/`IDLE_*` → `G_H*` ≈ 5 gates × ≤ 6.5 ns + 1 kΩ ≈ 45 ns [CALC]. Planned ends, cap expiry and rail collapse still use the 20–36 µs `RUN_D20` delay (the low side must open first) | I-8, BR change 1 |
| D-8 | **Fast idle input.** New **U1268 SN74LVC1G14** `IDLE_FAST_N` → `IDLE_FST`; **U1269 SN74LVC1G32** `IDLE_ANY` = `IDLE_RAW` OR `IDLE_FST`; `IDLE_ANY` replaces `IDLE_RAW` at **U1246 pin 1** and **U1236 pin 3**; **R6210 100 kΩ** `IDLE_FAST_N`–GND (contract default), **R6211 100 kΩ** `IDLE_ANY`–BRIDGE_3V3 (dead gate reads fault) | — | I-8, BR change 2 |
| D-9 | **`LSD_OK` input.** **R6212 100 kΩ** `LSD_OK`–GND (contract default); new **U1270 SN74LVC1G08** `RAIL_OK` = `BR5_OK` AND `LSD_OK`; `RAIL_OK` replaces `BR5_OK` at **U1237 pin 6** (start) and **U1247 pin 1** (`SEQ_UV`); **R6213 100 kΩ** `RAIL_OK`–GND | a failed MGJ2 ends the sequence like a `BRIDGE_5V` collapse, ≈ 5 µs [BR Q4] | BR change 9 |
| D-10 | **Phase-1 cap.** New **U1271 TLV7031DBVR** (IN+ `P1_TH`, IN− `P1_RC`, OUT `P1C_N`); **R6214 113 kΩ** 0.1 % 25 ppm/K thin film `ACT`–`P1_RC`; **C1282 100 nF C0G ±2 % 1206** `P1_RC`–GND; **D1216 BAS116LT1G** A `P1_RC`, K `ACT`; **R6215 57.6 kΩ** 0.1 % `BRIDGE_3V3`–`P1_TH`; **R6216 100 kΩ** 0.1 % `P1_TH`–GND; **R6217 100 kΩ** `P1C_N`–GND | threshold 100/157.6 = 0.6345; t = 113 k × 100 nF × ln(1/(1 − 0.6345)) = 11.37 ms nominal; band −5.4/+4.3 % (the simulated spread of the identical 25 ms cap, LR §2) = **10.75–11.85 ms** [CALC] | I-13, D-63 |
| D-11 | **Phase-2 cap.** New **U1272 SN74LVC1G74DCTR** `P2S` latch (CLK `G_LA_I`, D `BRIDGE_3V3`, PRE_N `BRIDGE_3V3`, CLR_N `RST_N`, Q `P2S`); **R6218 100 kΩ** `P2S`–BRIDGE_3V3 (dead latch: timer runs from rail-up and refuses); new **U1273 TLV7031DBVR** (IN+ `P2_TH`, IN− `P2_RC`, OUT `P2C_N`); **R6219 76.8 kΩ** 0.1 % `P2S`–`P2_RC`; **C1283 100 nF C0G ±2 %** `P2_RC`–GND; **D1217 BAS116LT1G** A `P2_RC`, K `P2S`; **R6220 57.6 kΩ**, **R6221 100 kΩ** 0.1 % (`P2_TH`); **R6222 100 kΩ** `P2C_N`–GND | t = 76.8 k × 100 nF × 1.0065 = 7.73 ms nominal, **7.31–8.06 ms** [CALC] | I-13, D-63 |
| D-12 | **Cap faults into the latch.** New **U1274 SN74LVC1G02** `P1F` = NOR(`P1C_N`, `P1D`); **U1275 SN74LVC1G02** `P2F` = NOR(`P2C_N`, `DONE`); **U1276 SN74LVC1G32** `PCF` = `P1F` OR `P2F`; **U1252 SN74LVC1G32 → SN74LVC1G332DBVR**, `TRIG` = `TRIG_A` OR `CAP_F` OR `PCF`; **R6223 100 kΩ** `PCF`–BRIDGE_3V3 | phase-1 cap acts only before `P1D`; phase-2 cap only before `DONE`, so a normal shock never sets a fault | D-63 |
| D-13 | 25 ms overall cap **unchanged** (R1288 249 kΩ, R1289 31.6 kΩ, R1290 26.1 kΩ). The shortened 18 ms option of LR §2 is not taken | per-phase caps make it a backstop only | — |
| D-14 | **XFER bounce.** **R6224 10 kΩ** `K_REST_N`–`K_REST_F` and **C1284 1 nF C0G** `K_REST_F`–GND; U1210 pin 1 now `K_REST_F`; new **Q1201 2N7002** (G `K_AT_REST`, S GND, D `XF_DIS`) with **R6225 10 Ω** `XF_DIS`–`XF_RC` (empties C1253 fully while the contact is at rest) | XFER rises 11.0–11.3 ms after the last release for bounces ≥ 10 µs [SIM LR §1.4] | I-9 |
| D-15 | **`K_REST_N` must be seen at rest when the coil starts** (R11, WP-H C-6). New **U1277 SN74LVC1G74DCTR** (CLK `K_EN_I`, D `K_AT_REST`, PRE_N `BRIDGE_3V3`, CLR_N `RST_N`, Q `K_WAS_REST`); **U1238 SN74LVC1G08 → SN74LVC1G11DBVR**: `XF_SRC` = `K_LEFT` AND `K_EN_I` AND `K_WAS_REST`; **R6226 100 kΩ** `K_WAS_REST`–GND | an open aux contact can no longer look like a transfer | WP-H C-6 |
| D-16 | **BTOK.** R1280 36 kΩ → **49.9 kΩ** 0.1 % thin film; **U1241 SN74LVC1G17 → TLV7031DBVR** (IN+ `BT_RC`, IN− `BT_TH`, OUT `BTOK`); **R6227 56.2 kΩ** 0.1 % `BRIDGE_3V3`–`BT_TH`; **R6228 100 kΩ** 0.1 % `BT_TH`–GND | BTOK 4.74–5.52 ms ≥ test-relay operate 3 ms [SIM LR §1.3] | I-9 |
| D-17 | Update the sheet notes: `K_EN` now drives only the coil driver on 14; status `BR_FAULT_N` also reports per-phase caps and `LSD_OK` | — | — |

New refs on 12 use U1263–U1277, Q1201, C1282–C1284, D1216–D1217 and the overflow block R6209–R6228.
`BRIDGE_3V3` load grows by ≈ 15 LVC gates (≤ 10 µA each) and 2 × TLV7031 (≈ 0.3 µA each): negligible against the
60 mA hold-up budget [CALC].

## WP-E charger (sheet 10)

| # | Change | Values / parts | Reason |
|---|---|---|---|
| E-1 | **R1019 (RBG) 931 Ω → 876 Ω** 0.1 % | primary-side trip **2,116–2,316 V**, 2,409 V with the comparator's maximum overdrive [SIM CS Q4]: never below a normal charge (2,008.6 V) by ≥ 107 V; a last-resort layer that relies on the capacitor's surge rating (D-65) | I-2 |
| E-2 | T1001 made-to-order requirement (§B4.5 / §14): add **efficiency ≥ 90 % (core + winding loss ≤ 10 % of throughput) at 20 A peak, 30–45 kHz**; **L_pri 7.5 µH +10 %/−5 %** (WP-E C2); **RMS 12.4 A** (WP-E C8); secondary winding resistance stated | worst-case charge to 1,950 V ≈ 5.9 s with 90 %; 7 s is missed only below ≈ 76 % overall [SIM CS Q3] | I-11 |
| E-3 | Keep R1001 5.0 mΩ (the 4.7 mΩ option is not taken: it raises the peak to 24 A against the transformer's 25 A and the eFuse's 11.4 A minimum limit) | — | CS Q3 |
| E-4 | Sheet notes: replace the band text of R1019 (2,116 V "contract value kept") with E-1 | — | — |

Open for WP-E (outside I-1…I-25, carried): C3 (UVLO2 band vs `5V0_SYS`), C4 (FB string part), C6 (TPS25982 rated
only from −15 °C), C7 (silent charger stop: firmware rule, see the firmware table).

## WP-F storage (sheet 11)

| # | Change | Values / parts | Reason |
|---|---|---|---|
| F-1 | **Q1101–Q1108 IXTT10N100D2 → IXTT2N170D2** (Littelfuse/IXYS DS100418B: 1,700 V, V_GS(off) −2.0…−4.0 V, R_DS(on) ≤ 6.5 Ω, TO-268). Pinout to verify against DS100418B (expected 1 G, 2/4 D, 3 S as the 10N100D2) | cold rating assumed 0.94 × 1,700 = 1,598 V [Fig 12 **not read**] | I-3; CS Q1 |
| F-2 | **Gate-steering diodes**: in each upper gate feed insert a diode between the 100 kΩ resistor and the gate. Stack A: R1115 now `HV_A_T1`–`HV_A_GF2`, **D1127** A `HV_A_GF2` K `HV_A_G2`; R1116 → `HV_A_GF3`, **D1128** → `HV_A_G3`; R1117 → `HV_A_GF4`, **D1129** → `HV_A_G4`. Stack B: R1118–R1120 likewise with **D1130–D1132** (`HV_B_GF2..4`). Part **Vishay GP02-40** (doc 88635 Rev 09-Aug-2022: 4,000 V, 0.25 A, I_R ≤ 5 µA at 25 °C / ≤ 50 µA at 100 °C at 4 kV) | — | I-3; CS cfg 5 |
| F-3 | **Diode bypass**: **R6101–R6106 22 MΩ** across D1127–D1132 (`HV_x_GFk`–`HV_x_Gk`), HV chip ≥ 1.5 kV working (Vishay CRHV2512 or Stackpole HVCR2512 range; 22 MΩ code **to be confirmed**) | — | CS cfg 5 |
| F-4 | **Gate–source resistors R1121–R1126 10 MΩ → 100 MΩ** HVCR2512 (100 MΩ code seen in Stackpole listings, datasheet not read: `Verify`) | — | CS cfg 5 |
| F-5 | **Sharing strings R1107–R1114 10 MΩ → 4.7 MΩ** HV chip (HVCR2512 range; code to be confirmed) | standing loss 0.26 W per stack at 2,200 V [CS] | CS cfg 5 |
| F-6 | **Per-device drain–source network**: **R6107–R6114 10 MΩ** HVCR2512 (as the old string parts) and **C1137–C1144 3.3 nF 2 kV C0G** (same part as C1101–C1108; KEMET C2220Y332JGGACAUTO is a candidate, rating seen in a distributor listing only) across each of Q1101–Q1108 (Q1101: `HV_A_M1`–`HV_RTN`; Q1102: `HV_A_M2`–`HV_A_M1`; Q1103: `HV_A_M3`–`HV_A_M2`; Q1104: `HV_A_TOP`–`HV_A_M3`; B likewise) | single shorted device ≤ 1,174 V across 100 Monte Carlo runs, 0.73 of the cold rating [SIM CS]; release-edge sharing 626 V [SIM CS dynamic] | I-3: **essential, not optional** (without it a Monte Carlo run reached 1,759 V) |
| F-7 | Keep C1101–C1108 (3.3 nF across the string resistors) | — | — |
| F-8 | **Pump output resistor**: D1112 pin 1 (and D1116 pin 1) move from `STKA_G`/`STKB_G` to new nodes `PMPA_H4`/`PMPB_H4`; add **R1194 470 Ω** `PMPA_H4`–`STKA_G` and **R1195 470 Ω** `PMPB_H4`–`STKB_G` | removes the power-up lock-out with a pilot at its 2.0 V maximum [SIM R3 §2, cases X1–X3 vs R2–R4] | I-17 |
| F-9 | **Release pilots Q1112, Q1113 MMBFJ177 → MMBFJ270** (onsemi MMBFJ270/D Rev 2, May 2022: V_GS(off) 0.5–2.0 V, I_DSS 2–15 mA, V_DG −30 V). SOT-23 pinout **to verify** against the datasheet before re-using the MMBFJ177 symbol | — | I-17; CS change 7 |
| F-10 | **Held sensors**: D1125, D1126 MMSZ4688 → **MMSZ4687T1G** (4.3 V; onsemi MMSZ4678T1/D Rev 13: 4.09–4.52 V at 50 µA). Add a **second sensor per stack** on its own line: stack A **R1196 3.9 kΩ** (`HV_RTN`–`HLDA2_ZK`), **D1133 MMSZ4687T1G** (`HLDA2_ZK`–`HLDA2_LA`), **R1197 22 kΩ** (`HLDA2_LA`–`STKA_G`), **U1105 TLP187** (anode `HLDA2_LA`, cathode `STKA_G`, collector **`HOLD_A_HELD2_N`**, emitter GND), **TP1107**; stack B **R1198, D1134, R1199, U1106, TP1108** (`HLDB2_*`, **`HOLD_B_HELD2_N`**) | thresholds 4.95 / 5.62 / 6.12 V (low / nominal / aged-cold), 0.71 V above the 2N170D2's worst cut-off [SIM R3 §1]; **valid only with F-1** | I-4, I-17 |
| F-11 | Sheet notes: pump requirement now "both sensors held, HELD_N ≤ 1.0 V, at −20 °C, 3.1 V, 1.35–1.65 kHz, aged sensors" (met [SIM R3]); release ≤ 25 ms with the clock stopped low; "727 V" text removed | — | — |
| F-12 | The off-board capacitor's internal 10 MΩ bleed and surge rating are requirements on C1101 (`docs/requirements/storage-capacitor.md`), not board parts | — | I-5, D-65 |

New refs on 11: D1127–D1134, C1137–C1144, U1105–U1106, R1194–R1199, overflow R6101–R6114, TP1107–TP1108.

## WP-G bridge power (sheet 13)

| # | Change | Values / parts | Reason |
|---|---|---|---|
| G-1 | **Over-current trip 90 → 80 A**: R1351 174 kΩ → **196 kΩ** 1 % | 3.3 V × 10.0 / 206 = 160.2 mV = 80.1 A; band ≈ 74–86 A; interrupted current ≤ 97 A < I_DM 107 A [SIM BR Q2]; highest planned peak 67.4 A (2,000 V, 25 Ω) | I-8; WP-G F-2 |
| G-2 | **I_SHOCK scale**: R1355 10.0 kΩ → **13.3 kΩ** 1 % → 24.0 mV/A; **R1358 237 kΩ → 267 kΩ** (idle threshold 3.3 × 10/277 = 119.1 mV = 4.96 A) | saturated INA240 ≤ 3.20 V < VDDA min 3.26 V [SIM LR §5.1] | I-21 |
| G-3 | **Fast idle path** (new, `IDLE_FAST_N`): **R1365 100 Ω** `SHUNT_KP`–`IDF_IN`; **C1345 100 pF C0G** `IDF_IN`–`SHUNT_KN`; **D1332 BAT54S** clamp of `IDF_IN` to GND/BRIDGE_3V3 (as D1331); **U1317 TI OPA838** (SOT-23-6 DBV; orderable code to confirm) on BRIDGE_3V3, non-inverting, IN+ `IDF_IN`; **R1366 10.0 kΩ** 0.1 % `IDF_OUT`–`IDF_FB`; **R1367 1.00 kΩ** 0.1 % `IDF_FB`–`SHUNT_KN` (gain 11); **U1318 TLV3501AIDBVR** IN− `IDF_OUT`, IN+ `IDF_REF`, OUT **`IDLE_FAST_N`**; **R1368 48.7 kΩ** 0.1 % `BRIDGE_3V3`–`IDF_REF`; **R1369 1.00 kΩ** 0.1 % `IDF_REF`–`SHUNT_KN`; **C1346 100 nF** `IDF_REF`–`SHUNT_KN` | threshold 3.3 × 1.00/49.7 = 66.4 mV ÷ 11 ÷ 2 mΩ = **3.02 A**; offsets OPA838 ≤ ±220 µV (−40…85 °C, SBOS867E) × 11 and TLV3501 ≤ ±6.5 mV (SBOS321E) → ±8.9 mV → **2.6–3.5 A** [CALC]; delay ≈ 10 ns RC + ≈ 13 ns amplifier (300 MHz GBW / 11) + ≤ 7 ns comparator ≈ 30 ns [CALC] | I-8; BR change 2 |
| G-4 | **`LSD_OK` monitor** (new): **R1360 100 kΩ** 0.1 % `LSD_P20V`–`LSD_SNS`; **R1361 12.1 kΩ** 0.1 % `LSD_SNS`–GND; **C1343 100 pF C0G** `LSD_SNS`–GND; **R1362 80.6 kΩ** 0.1 % `BRIDGE_3V3`–`LSD_TH`; **R1363 100 kΩ** 0.1 % `LSD_TH`–GND; **C1344 100 nF** `LSD_TH`–GND; **U1316 TLV7031DBVR** (IN+ `LSD_SNS`, IN− `LSD_TH`, OUT **`LSD_OK`**) | trip 1.827 V × 112.1 / 12.1 = **16.9 V** (band ≈ 16.5–17.3 V: 3V3 ±1.5 %, 0.1 % parts, ±8 mV) [CALC]; MGJ2 output at the 4.71 V minimum `BRIDGE_5V` ≈ 18.8 V [EST: output ∝ input; MGJ2 regulation not read] | BR change 9 |
| G-5 | **Carrier-loss detector, island A** (new; HSB identical with the B refs): **D1333 BAV70** (anodes `HSA_S1`, `HSA_S2`, cathode `HSA_CLR`); **R1373 4.7 kΩ** `HSA_CLR`–`HSA_CLF`; **C1350 47 pF C0G** `HSA_CLF`–`HSA_KS`; **R1374 4.7 kΩ** `HSA_CLF`–`HSA_KS`; **R1375 10 kΩ** `HSA_CLF`–`HSA_CLB`; **Q1312 MMBT3904** (B `HSA_CLB`, E `HSA_KS`, C `HSA_CLG`); **D1334 BAT54** (A `HSA_CLB`, K `HSA_CLG`, Baker clamp); **R1376 10 kΩ** `HSA_G`–`HSA_CLG`; **D1335 BZX84C15** (K `HSA_CLG`, A `HSA_KS`); **Q1313 2N7002** (G `HSA_CLG`, S `HSA_KS`, D `HSA_CLD`); **R1377 100 Ω** `HSA_RECT`–`HSA_CLD`. Island B: **D1336, R1378, C1351, R1379, R1380, Q1314, D1337, R1381, D1338, Q1315, R1382** (`HSB_CL*`). All `HSA_`/`HSB_` net class `HV_FLOAT` | detector node ≤ 0.41 V while the carrier runs (2N7002 V_th ≥ 1.0 V); rectified node < 1 V 1.1–1.3 µs after the carrier stops (8.8 µs as drawn); gate < 2 V 3.9–7.0 µs (7.2–11.5 µs as drawn) [SIM R3 §3]; extra island load ≈ 3.5 mA of the ≥ 30 mA transformer rating [CALC] | I-8; BR change 3 |
| G-6 | **Local film capacitor** `HV_CAP_P`–`HV_RTN` close to D1301's cathode and R1301: **C1347, C1348 2 × Cornell Dubilier 940C30P15K-F** (0.15 µF, 3,000 V DC; CDE 940C catalogue read by a research agent; size and mass **not read**) in series through node `HV_CAP_FM`, each with a balancing **R1370, R1371 10 MΩ** HVCR2512 across it (`HV_CAP_P`–`HV_CAP_FM`, `HV_CAP_FM`–`HV_RTN`) → 75 nF. C1349 and R1372 reserved | each part alone withstands the full capacitor voltage plus the turn-off spike (one shorted part does not short the capacitor). **75 nF is below the 100 nF simulated** (2,497 V with 100 nF, 2,511 V with 470 nF) — confirm in the second round; a verified 1 µF ≥ 2.2 kV part was not found (CDE 940C 1 µF stops at 2,000 V) | I-8; BR change 5 |
| G-7 | **Keep** the RC snubber C1301/C1302/R1304 as drawn; no RCD clamp | larger snubbers made the spike worse or discharged through the switches [SIM BR Q2] | BR change 6 |
| G-8 | Sheet notes: §12 failure table gains the same-side shoot-through row (QLA shorted, QHA turning on at phase 1 start: ≈ 240 A, 11–12 J in QHA as drawn; ≈ 11–13 mJ with G-3, G-5 and D-7 [SIM BR Q1]) | — | BR "contradicts" list |

New refs on 13: U1316–U1318, Q1312–Q1315, D1332–D1338, C1343–C1351, R1360–R1382 (R1364 unused).
`BRIDGE_3V3` load on 13 rises by ≈ 4.5 mA (OPA838 ≈ 1 mA, TLV3501 ≈ 3.2 mA typical [BK]) to ≈ 25–30 mA:
inside WP-D's 55 mA allowance.

## WP-H patient + front ends (sheets 14, 15, 16)

| # | Change | Values / parts | Reason |
|---|---|---|---|
| H-1 | **Separate coil-supply enable**: R1403 (to Q1402 gate) and R1413 (boost EN) now come from **`K_SUPPLY_EN`** instead of `K_EN`; add **R1421 100 kΩ** `K_SUPPLY_EN`–GND (contract default); R1404 stays on `K_EN`; R1417 (DRV110 enable) stays on `K_EN` | `K_EN` low still opens the coil drive directly (CI-8) | I-15 |
| H-2 | **C1401 68 nF → 680 nF** X7R 50 V; **R1406 127 kΩ → 515 kΩ** (boost ILIM ≈ 2.0 A) | battery peak 2.5–2.6 A instead of 8.7–9.9 A; reservoir at 11 V 74–99 ms after `K_SUPPLY_EN`; transfer **15.9 ms** after `K_EN` in every battery case [SIM LR §3.2] | I-15 |
| H-3 | K1401: keep the placeholder symbol; set its value to "DPDT HV relay + aux — MADE TO ORDER, PLACEHOLDER (docs/requirements/patient-relay.md)" and `Verify` = "no part exists; D-64" | — | I-6, D-64 |
| H-4 | Sheet 15: **R1528, R1529 51 kΩ DPCR2512** (TT Electronics, as R1507/R1508) from `PT_FE_A`/`PT_FE_B` to new nodes `PT_ESA_CL`/`PT_ESB_CL`; R1509/R1510 move from `PT_MSA_CL`/`PT_MSB_CL` to these new nodes (R1511/R1512, the impedance inputs, stay on the old nodes); new clamps **D1506, D1507 CDSOD323-T03C** (as D1503/D1504) from `PT_ESA_CL`/`PT_ESB_CL` to GND. The `PT_` names keep the PATIENT net class | impedance noise 4.6–6.2 mΩ instead of 25–34 mΩ [SIM LR §6]; ECG filter corner unchanged | I-14 |
| H-5 | Sheet 16: no circuit change; MAX30009 carrier **96 kHz** (firmware/configuration) | stops aliasing into the ADS1292R ECG channel (WP-H §3.3) | I-18 |
| H-6 | Sheet 14 notes: coil-supply budget now ≈ 2.0 A for ≈ 0.1 s before the shock, not 8 A | — | — |

---

## Firmware (to be made by the firmware work; nothing here edits firmware)

| # | What | Value | Where it lives | Source |
|---|---|---|---|---|
| FW-1 | Relay settled before `CMD_HA` | **14 ms** after `K_REST_N` last read low (restart on any low sample); abort if not settled 40 ms after `RELAY_CMD` | `aed-core/src/rescue.rs` (delivery sequence) and the board layer; no constant exists yet — add to `aed-core/src/config.rs` | LR §1.1 |
| FW-2 | Rail-ready wait | **30 ms** after `BRIDGE_ARMED_N` low before `RELAY_CMD`, `BTEST_CMD` or any `CMD_*` | new constant in `aed-core/src/config.rs` | LR §1.1 |
| FW-3 | Ignore short armed pulses | treat `BRIDGE_ARMED_N` lows < **10 ms** as "not armed" | `aed-core/src/rescue.rs`, `selftest.rs` | LR §1.2 |
| FW-4 | Re-arm | consent absent ≥ **75 ms** between attempts; `LatchTiming::PROPOSED.rearm_us` 20,000 → **60,500** | `aed-core/src/interlock.rs` | LR §1.1 |
| FW-5 | Permit window | `LatchTiming::PROPOSED.permit_cap_us` 250,000 → **150,000**; `Config::hardware_permit_cap_ms` 250 → **150**; `Config::permit_window_ms` 200 → **120** (below the 131.8 ms hardware minimum, above the latest start ≈ 70 ms) | `interlock.rs`, `config.rs` | LR §1.3 |
| FW-6 | Sequence start timeout | `Config::sequence_start_timeout_ms` 50 → **80** (armed + 30 + ≤ 40 + 0.1 ms) | `config.rs` | FW-1, FW-2 |
| FW-7 | Test relay | **7 ms** after `BTEST_CMD` (with D-16; 6 ms without) | `aed-core/src/selftest.rs` / board layer | LR §1.1 |
| FW-8 | Phase maxima | `aed-waveform` `phase1_max` 12,000 → **10,000 µs**, `phase2_max` 8,000 → **6,700 µs**, `max_total` 20,000 → **17,200 µs** (DESIGN_TABLE longest 9.011 / 6.007 ms) | `aed-waveform/src/config.rs` | D-63 |
| FW-9 | Hardware caps in the model | add `phase1_cap_us` **11,370** (band 10,750–11,850) and `phase2_cap_us` **7,730** (7,310–8,060) to `LatchTiming`; keep `time_cap_us` 25,000 | `interlock.rs`; `aed-sim/src/plant.rs` | D-63 |
| FW-10 | Over-voltage limit | `Config::overvoltage_limit` 2,200 → **2,070 V** reading, both controllers | `config.rs` (used by `selftest.rs`, `supervisor.rs`) | CS Q4 |
| FW-11 | Hold clocks | `HOLD_M_CLK`, `HOLD_S_CLK` at **1.5 kHz ± 10 %**, always stopped **low**; any wait on "not held" ≥ **25 ms**; "held" after power-up ≤ 35 ms | board layer; document in `aed-core/src/hal.rs` (`set_discharge_hold`) | R3 §2 |
| FW-12 | Coil supply | raise `K_SUPPLY_EN` when charging for a shock starts and ≥ **200 ms** before `RELAY_CMD`; lower it on withdrawal or abort | new HAL method `set_coil_supply`; `rescue.rs` | LR §3.2 |
| FW-13 | Compression readings | discard impedance samples while `CHG_RUN` is high and for **100 ms** after [EST: 10 Hz demodulation bandwidth]; **exclude these gaps from `chest_channel_fail_ms`** (5 s) so a long charge is not recorded as a failed channel; optional: suspend charger top-ups during the 3 s countdown (droop ≈ 7.1 V/s × 3 s = 21 V, inside the ± 31.5 V at-plan window at 1,950 V — check against the hysteresis) | `rescue.rs`, `aed-cpr/src/detector.rs`, `config.rs` | D-67, LR §6 |
| FW-14 | I_SHOCK | scale **24.0 mV/A**; "current present" = ≥ 0.1 A | board layer | LR §5.1 |
| FW-15 | Self-test, new steps (D-62) | the T-steps of HARDWARE_DESIGN §B14.5: real permit alone / with request (rail must rise), relay move, refusal of an out-of-order start, dry shock-mode run with pads open, T-OVP (`OVP_TEST` → trip, latch, `OVP_RST`), four hold sensors, per-phase caps (overlong phase 1, overlong phase 2), overall cap (long gap) | `aed-core/src/selftest.rs`, `supervisor.rs` | D-62 |
| FW-16 | Real permit in self-test | supervisor grants it only with **its own reading < 50 V and the main's reported reading < 50 V**, charger off, at most 150 ms | `supervisor.rs` | D-62 |
| FW-17 | Over-voltage reset | after a real `OVP_TRIP`, pulse `OVP_RST` only once both readings are < 50 V; record; "not ready" until a self-test passes | `supervisor.rs` | I-22 |
| FW-18 | Charger silent stop | if the capacitor does not rise for ≥ 200 ms with `CHG_RUN` high, drop `CHG_SESSION` and `CHG_RUN` ≥ 2 ms and restart; allow ≥ 15 ms after `CHG_SESSION` before expecting a rise | `rescue.rs` / charger driver | WP-E C7 |
| FW-19 | Battery-load pulse | ≤ 100 ms, not repeated within 1 s | `MainHardware::battery_under_load` implementation | I-25 |
| FW-20 | Configuration records (not crates) | MAX17320 `COMMSH = 0`, thresholds (A-4); RV-3028 CLKOUT 1 Hz stored once; MAX30009 96 kHz; ADS1292R settings of WP-H C-9 | board configuration | WP-A, WP-H |

## Second-round simulation list

Each item names the netlist to re-run or write and the observable that settles it. New netlists go in `sim/rev3/`.

| # | Netlist | What changes | Settles |
|---|---|---|---|
| S-1 | `bridge/q1_shorted_lowside.cir` (new variant in rev3) | replace `hsdrv`'s ideal discharge with the `rev3/carrier_loss.cir` island; fast comparator 2.6 / 3.0 / 3.5 A with ≈ 30 ns; HSK path 45 ns; cases D and S at 1,950 and 2,200 V | QHA energy and Tj with the real circuits (BR's 11–13 mJ assumed ideal parts); resolves the 1 µs vs 2.7 µs PNP-start disagreement (R3 §3) |
| S-2 | `bridge/q2_oc_turnoff.cir` | trip 86 A (top of the 80 A band), local film **75 nF** (G-6), cable 0.5–1 µH | VDS stays ≈ 2.5 kV with 75 nF |
| S-3 | `bridge/q4_supply_collapse.cir` | add the `LSD_OK` monitor (16.5–17.3 V, 1.1 µs RC, 3 µs comparator) | MGJ2-only failure ends ≤ 5 µs, QLB Tj < 150 °C |
| S-4 | `logic-relay/timers.cir` | add the two phase caps (D-10, D-11) with BAS116 leakage and C0G ±2 %; BTOK on TLV7031 | 10.75–11.85 / 7.31–8.06 ms; never cuts 10.0 / 6.7 ms |
| S-5 | `logic-relay/races.cir` | window-start delay (D-6); `K_WAS_REST` latch (D-15) | race B closed; open aux contact refused |
| S-6 | `logic-relay/battery.cir` | R111 78.7 kΩ (A-2); test current 10–30 µA with the 1 MΩ rule | stuck-high on-time 124–220 ms band; release |
| S-7 | `logic-relay/sense.cir` | idle divider 267 kΩ | 4.96 A threshold with the 24.0 mV/A scale |
| S-8 | `charger-storage/stack_static_n4.cir`, `stack_dynamic_n4.cir` | cfg 5 **plus GP02-40 leakage 5–50 µA**, 100 MΩ / 22 MΩ tolerances, the 2N170D2 cold factor once Fig 12 is read | single-short worst stays ≤ 0.75 of the cold rating |
| S-9 | `charger-storage/stack_static_n5.cir` (diagnostic only) | re-run the 1,464 V Monte Carlo case with a fixed seed and dump every node | explains the outlier; five devices are not adopted either way |
| S-10 | `charger-storage/ovp_limits.cir` | R807 21.5 kΩ (2,107 V); `OVP_TEST` injection with Q801 healthy and shorted; diode leakage | band 2,073–2,142 V; the test detects a shorted Q801 |
| S-11 | `charger-storage/discharge.cir` | per-device 10 MΩ, 2N170D2; release-failed case thermal (27 J in one device) once a thermal model exists | dump time; SOA |
| S-12 | `charger-storage/charger.cir`, `bleed.cir` | board standing load 3.90 MΩ plus the capacitor's 10 MΩ (2.80 MΩ total, 1.44 W at 2,008.6 V) | charge time + ≈ 0.1 s [EST]; droop 7.1 V/s |
| S-13 | `rev3/hold_pump_rev3.cir` | re-fit the pump and sensor models to −20 °C measurements when they exist | the 1.35–1.65 kHz window |
| S-14 | `logic-relay/relay_supply.cir` | the made-to-order relay's real coil and timing | transfer ≤ 16 ms; total ≤ 30 ms |
| S-15 | new `rev3/energy_limiter.cir` (only if Q-57 is answered yes) | delivered-energy integrator into the fault latch | ≈ 160 J worst case |
