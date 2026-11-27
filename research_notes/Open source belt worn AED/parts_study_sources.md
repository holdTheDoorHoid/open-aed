# Parts study: working notes and sources (27 November 2026)

Working notes behind [docs/PARTS_STUDY.md](../../docs/PARTS_STUDY.md). This pass used about 38 searches and fetches, all on 27 November 2026. It builds on `miniaturization_hardware.md` and `hv_safety_isolation.md` and does not repeat their searches.

Tags:
- **[CALC]** is my arithmetic, with the working in PARTS_STUDY.md.
- **[EST]** is an engineering estimate, with its basis stated.
- **[BK]** is background knowledge from training that was not re-checked in this pass.
- **[UNVERIFIED]** means I looked and could not confirm it.

**Repository rule (CLAUDE.md rule 2, D-10, D-13).** These are parts ratings and sizing arithmetic only. Nothing here is a schematic, a build step or an instruction to power anything above 50 V. Nothing here is safe or ready for use on a person or animal.

**Access problems on 27 Nov 2026.** Digi-Key product pages returned HTTP 403, Ohmite's site returned HTTP 418, the TME and analog.com fetches failed (403, connection refused, timeout). So most prices and stock levels in the study are estimates, not checked listings. Search-result snippets from distributors were used where they existed.

---

## 1. Ferrocene (safety-qualified Rust compiler): what Ferrous Systems' own documents say

Fetched 27 Nov 2026.

- **Target list, Ferrocene user manual** (development-branch preview; the page says "It might be incomplete or inaccurate") — <https://public-docs.ferrocene.dev/main/user-manual/targets/index.html>
  - **Qualified:** `aarch64-unknown-none` (Armv8-A), `armv7r-none-eabihf` (Armv7-R), `thumbv7em-none-eabi` and `thumbv7em-none-eabihf` (Armv7E-M = Cortex-M4/M7), `thumbv8m.main-none-eabi` and `thumbv8m.main-none-eabihf` (Armv8-M Mainline = Cortex-M33/M35P/M55/M85), and the `x86_64-unknown-linux-gnu` host.
  - The bare-metal targets are qualified only when cross-compiled from x86-64 Linux.
  - **Only "supported", not qualified:** `thumbv6m-none-eabi` (Cortex-M0/M0+) and `thumbv8m.base-none-eabi` (Cortex-M23).
  - Not listed at all: any 32-bit RISC-V bare-metal target.
  - The page says "A subset of `core` and `alloc` is certified against IEC 61508 (SIL 2) and ISO 26262 (ASIL B) on qualified targets." It does not mention IEC 62304 for the library.
- **Release notes, Ferrocene 26.11.0** — <https://public-docs.ferrocene.dev/main/release-notes/26.11.0.html>
  - Quote: "Two new targets are now supported and qualified for safety critical use: Armv8-M Mainline bare-metal (soft-float) (`thumbv8m.main-none-eabi`), Armv8-M Mainline bare-metal (hard-float) (`thumbv8m.main-none-eabihf`)".
  - No release date on the page. The release list shows 26.11.0 as the newest stable release, with 27.02 upcoming — <https://public-docs.ferrocene.dev/main/release-notes/index.html>
- **Ferrocene 26.05.0 blog** (published 28 July 2026) — <https://ferrous-systems.com/blog/ferrocene-26-05-0/>
  - Qualified `-C target-cpu` support is "limited". For `thumbv7em-none-eabihf`, only `cortex-m4` is named; `cortex-m7` is not mentioned.
  - The blog restates that Ferrocene is TÜV SÜD-qualified "according to ISO 26262 (ASIL D), IEC 61508 (SIL 3), and IEC 62304 (Class C)".
- **Qualification plan page** (development branch) — <https://public-docs.ferrocene.dev/main/qualification/plan/index.html>
  - The material was "developed for" ISO 26262:2018 (ASIL D/TCL 3), IEC 61508:2010 (class T3), IEC 62304:2006 + AMD 1:2015, and EN 50716:2023.
  - It names no targets.
- **IEC 62304 first qualification** (January 2025) — <https://ferrous-systems.com/blog/ferrocene-achieves-iec-62304-qualification/>; <https://www.businesswire.com/news/home/20250114192138/en>
- **Resolution of the conflict in earlier notes.** As of the 26.11.0 documentation, both the Cortex-M4/M7 family (Armv7E-M) and the Cortex-M33/M55/M85 family (Armv8-M Mainline) are qualified targets. Cortex-M0/M0+ is not qualified.
  - **Caveat:** these are development-branch web pages. The signed qualification report for the exact release is the authority, and it is sold with the subscription; I did not see it.
  - Whether a specific `-C target-cpu=cortex-m33` setting is inside the qualified envelope is not stated [UNVERIFIED].

## 2. Rust HAL support

- **Embassy README** — <https://github.com/embassy-rs/embassy>
  - embassy-stm32 claims "all STM32 microcontroller families".
  - embassy-nrf covers "nRF52, nRF53, nRF54 and nRF91".
  - The other HALs cover RP2040/RP23xx, TI MSPM0, NXP MCX-A, Renesas RA and others.
  - **Bluetooth.** The `trouble` BLE host lists "nRF52, nrf54, rp2040, rp23xx and esp32 and serial controllers". `nrf-softdevice` covers nRF52 only, and embassy-stm32-wpan covers STM32WB. STM32WBA Bluetooth is **not** listed.
- The Nordic link layer used under `trouble` on nRF parts is Nordic's closed binary controller [BK]. That matters for D-24 (only `libm` on the device).
- **nRF54L15** is launched and in production per Nordic — <https://www.nordicsemi.com/Products/nRF54L15>. The detailed core and memory figures (Cortex-M33 at 128 MHz, 1.5 MB non-volatile memory, 256 KB RAM) are [BK].

## 3. Energy storage capacitor

- **General Atomics Series DQ defibrillator capacitors** (metallized film, drawn metal cans, "peak currents up to 300 A") — <https://www.ga.com/capacitors/series-dq-defibrillator-capacitors>
  - 320DQ4130: 130 µF, 2.0 kV, 1.66 J/cc, 2,500 cycles, 33 × 55 × 99 mm
  - 323DQ4115B: 115 µF, 2.3 kV, 1.04 J/cc, 10,000 cycles, 49 × 74 × 95 mm
  - 323DQ4150A: 150 µF, 2.3 kV, 1.35 J/cc, 5,000 cycles, 49 × 74 × 95 mm
  - 30954/30955: 100 µF, 3.0 kV
  - Not on the page: weight, temperature ratings, datasheets and prices. Contact is by enquiry only.
  - GA asks to be contacted for uses other than "low rep rate (~0.1 Hz), non-oscillatory millisecond pulse discharge".
- **TDK B32365 datasheet** (September 2020) — <https://www.tdk-electronics.tdk.com/inf/20/50/ds/DBL_B32365_201014_final.pdf>
  - Metallized polypropylene, PU or epoxy fill, 32–200 µF, up to 5,500 V DC, 16–529 J
  - Tstg −20/+70 °C; Tmin −20 °C; **Tmax +55 °C**; climatic category 40/93/56
  - Life expectancy up to 10,000 cycles, "for the temperature of 25 °C"; for other temperatures, a lifetime curve is referred to but not included
  - tan δ < 0.010 at 120 Hz; test voltage 1.1 × VR
  - Dimensions: "upon request", so there is no catalogue size table. Made in Brazil per the label code.
  - The datasheet warns of "self-recharging phenomena".
- **CRE (Wuxi CRE New Energy) DMJ-PC defibrillator film** — <https://www.cre-elec.com/custom-made-film-capacitor-for-defibrillator-product/>
  - 125 µF ±10%
  - "2000V.DC max. 10min" short-time rating; permanent rating 900 V DC
  - 250 Ws stored; **weight 0.3 kg**
  - Operating −25 to 60 °C; **storage −40 to 85 °C**; ≥10,000 discharge cycles
  - A second part: 200 µF, 2,200 V, operating −25 to 70 °C, storage −40 to 85 °C, ≥2,500 cycles into 10 Ω
  - The page's short description conflicts with these (it gives −45 to +70 °C)
  - No dimensions or prices. This is the **only published mass for a defibrillator-class film capacitor found**.
- **CDE Type Q**: the product pages give no temperature, cycles, weight or part table — <https://cde.com/capacitors/film-paper-film-capacitors/power-film-dc-applications/defibrillator-capacitors>. The press material says "as high as 2 J/cc" — <https://cde.com/news/2020/4/q-series>. The part table is in the 2012 catalog already summarised in `miniaturization_hardware.md`.
- **Electrolytic density proxy.** Farnell lists Rubycon 200KXW330MEFC18X30 (330 µF, 200 V, 18 × 30 mm) at 0.012 kg — <https://uk.farnell.com/rubycon/200kxw330mefc18x30/cap-330-f-200v-20/dp/8126356>. [CALC] 12 g / 7.63 cm³ = 1.57 g/cm³.
- **105 °C 450 V electrolytics, distributor-stocked**
  - Rubycon 450MXK220MEFCSN22X40: 220 µF, 450 V, 22 × 40 mm snap-in, 3,000 h at 105 °C. Digi-Key.ca showed 20 in stock, CAD 12.34 each at qty 1 and about CAD 5.67 at 1,000 (search snippet) — <https://www.digikey.ca/en/products/detail/rubycon/450MXK220MEFCSN22X40/5023796>
  - Würth 861141483007: 22 × 53 mm, 97 in stock, CAD 13.51 at qty 1 — <https://www.digikey.ca/en/products/detail/würth-elektronik/861141483007/5147789>
- **Photoflash parts at distributors.** Chemi-Con EPH-331 (PH series) was not found at Digi-Key, Mouser or LCSC. It appears to be catalogue or special-order only [UNVERIFIED]. Catalogue: <https://www.chemi-con.co.jp/products/relatedfiles/capacitor/catalog/PH-LL-e.PDF>

## 4. Output switches

- **Microchip MSC080SMA330B4N datasheet** DS00005914A, rev A, 04/2025 (RS copy) — <https://docs.rs-online.com/a067/A700000015733917.pdf>
  - VDSS 3,300 V; ID 43 A at Tc 25 °C and 27 A at 100 °C
  - **IDM 107 A**, footnote: "Repetitive rating: pulse width and case temperature are limited by the maximum junction temperature." **No pulse width is stated.**
  - RθJC 0.22 typ / 0.28 max °C/W; TJ max 150 °C
  - RDS(on) 85 typ / 106 max mΩ at VGS 20 V, ID 30 A (pulse test < 380 µs)
  - VGS +23/−10 V; body diode VSD 4.0 V at 30 A
  - The transient thermal impedance curve (Fig. 1-18) exists, but its values could not be read from the text extraction
  - TO-247 notched 4-lead
- **Littelfuse/IXYS CS20-22moF1 thyristor**: VDRM = VRRM 2,200 V, IT(AV) 18 A, **ITSM 200 A**, ISOPLUS i4-PAC. Stocked at Farnell, Future and LCSC — <https://www.mouser.com/datasheet/2/240/CS20-22moF1-1548499.pdf>; <https://www.lcsc.com/product-detail/C5359323.html>. The ITSM pulse condition was not read; it is normally a 10 ms half-sine [BK].
- **Littelfuse/IXYS IXEL40N400 IGBT**: 4,000 V, 90 A (Digi-Key title), ISOPLUS i5-Pak — <https://www.digikey.co.uk/en/products/detail/ixys/IXEL40N400/1651852>
  - ICM 170 A "limited by TJ" (2006 sheet) or 250 A at 1 ms, VGE 20 V (2009 preliminary) — <https://www.littelfuse.com/~/media/electronics/datasheets/discrete_igbts/littelfuse_discrete_igbts_npt_ixel40n400_datasheet.pdf.pdf>
  - Distributor listings quote 400 A, which conflicts with both datasheets.
- Earlier notes cover the TI UCC21750 (2,121 V DC VIOWM), the Murata MGJ series, the Cynergy3 D-HR reed relays and the Physio-Control SCR+IGBT bridge (US5824017).

## 5. Discharge load resistors

- **Ohmite EY series**: ceramic composition, **400 J**, 46.2 × 9.4 mm, 10% tolerance. Ohmite says it is aimed at defibrillators (about 360 J pulses) and is meant "for single or infrequent pulse events" — <https://www.electronicspecifier.com/passives/400j-resistor-withstands-high-energy-impulse-events>
- **Ohmite 885SP** (800/1000 series tubular): 63.5 × 19.05 mm, 1–130 Ω, 45 W average at 40 °C, **250 J peak, 1,000 V peak**. The table alignment in the extracted text is uncertain — <https://www.mouser.com/datasheet/3/132/1/800_and_1000_series.pdf>
- **TT Electronics (Welwyn) DPCR defibrillator pulse chip resistors**: 2512 chips at 1–2 J per 10 ms pulse, two-sided, pre-qualified to IEC 60601-2-27. They suit ECG input protection, not the dump load — <https://www.tti.com/content/ttiinc/en/manufacturers/tt-electronics/products/defibrillator-pulse-chip-resistors.html>
- **Riedon**: wirewound parts handle more than 1,000 J pulses — <https://riedon.com/blog/pulse-applications-wirewound-resistors/>

## 6. Voltage-sensing resistors

- **TT Electronics**: conventional HV dividers reach 10 ppm/K, 0.2 ppm/V and 0.3% life drift at best; 25 ppm/K, 1 ppm/V and 0.5% are more typical — <https://www.ttelectronics.com/blog/rethinking-high-voltage-resistive-division/>; PCNS 2025 paper <https://passive-components.eu/wp-content/uploads/2025/09/3_3_TT-Electronics-PCNS-2025-Paper.pdf>
- **Stackpole HVCR**: 100% screened for voltage coefficient, better than −5 ppm/V (1206) and −2 ppm/V (2512). Ordinary HV chips "may have VCR up to 300 ppm and beyond" — <https://www.digikey.com/en/product-highlight/s/stackpole-electronics/hvcr-series-resistors>
- **Stackpole RVCU**: VCR 25–50 ppm, up to 3,000 V working in 2512 — <https://www.digikey.com/en/product-highlight/s/stackpole-electronics/rvcu-high-voltage-low-vcr-chip-resistors>
- **Vishay HV thin film**: VCR < 2 ppm/V — <https://www.vishay.com/docs/48547/_ig11394372-2503-infographic-smd%20hv%20resistors.pdf>

## 7. ECG and impedance front end

- **TI ADS1292R** — <https://www.ti.com/product/ADS1292R>
  - **ACTIVE**
  - 2 channels, 24-bit, 125 SPS–8 kSPS, 8 µVpp noise (150 Hz bandwidth, gain 6)
  - Integrated respiration impedance
  - 335 µW per channel; −40 to +85 °C; 4 × 4 mm VQFN or TQFP
  - No mention of defibrillation; no price shown.
- **ADI MAX30009**: bioimpedance front end, **PRODUCTION**. Drive 16 Hz–500 kHz, 16 nA–1.28 mA rms; Mouser gives "17 bits effective resolution with 1.1 Ω p-p noise" (units unclear) — <https://www.analog.com/en/products/max30009.html>; <https://www.mouser.in/en/new/analog-devices/maxim-max30009-afe/>
- **ADI MAX30001G**: **PRODUCTION**. BioZ drive 55 nA–96 µA; WLP package; **0 to +70 °C** per the datasheet snippet — <https://www.analog.com/en/products/max30001g.html>. The base MAX30001 datasheet gives BioZ 17 bits ENOB and 1.1 µVp-p noise — <https://wcm-cce.cldnet.analog.com/media/en/technical-documentation/data-sheets/max30001.pdf>. The base part's temperature range is [UNVERIFIED].
- **ADI ADAS1000**: respiration drive 46–64 kHz programmable (CN0308 reference design) — <https://www.analog.com/en/resources/reference-designs/circuits-from-the-lab/cn0308.html>. An ADI article says breathing changes thoracic impedance by about 0.2 Ω and that resolution is set by signal-to-noise ratio. Lifecycle [UNVERIFIED]; the product page fetch failed.
- **TI AFE4960**: not checked [UNVERIFIED]. Not carried into the study.

## 8. Battery

- **Molicel P45B**: a datasheet copy gives ≥3.60 Ah to 2.5 V at 4.5 A at **−20 °C**, against ≥4.15 Ah at 10 A at 23 °C — <https://www.kronium.cz/uploads/p45b-datasheet.pdf>. This is a third-party-hosted copy [UNVERIFIED that it is official].
- A forum cold-soak test at −15 °C and 7.5 A exists, but the sag figures were not retrieved — <https://endless-sphere.com/sphere/threads/bench-test-results-molicel-p45b-50a-4500mah-21700-an-extraordinary-cell.116190/>
- Earlier notes give mass about 70 g (66.7 g measured) and 45 A continuous.

## 9. Custom transformer sources (sourcing only)

- **Custom Coils**: custom HV transformers up to 60 kV, flyback types, biomedical applications — <https://www.customcoils.com/high-voltage-transformers>; <https://www.customcoils.com/flyback-transformers>
- **Agile Magnetics**: flyback prototypes through to production — <https://agilemagco.com/?p=1131>
- **Pulse Electronics**: quick-turn custom power transformers with free small-volume samples (general power, not HV-specific) — <https://electronicspecifier.com/power/custom-power-transformers-offered-on-quick-turnround>
- **Bourns**: custom transformer worksheet — <https://bourns.com/products/magnetic-products/custom-special-power-transformer-worksheet>
- An unnamed maker's "Model 4283-1200" HV flyback is marketed for portable defibrillators — <https://www.electronicdesign.com/products/transformer-suited-portables> (maker not identified [UNVERIFIED]).

## 10. Not found or not checked in this pass (carried into the study as UNVERIFIED)

- Masses of the GA, CDE and TDK film capacitors; the GA temperature ratings
- Live prices and stock for nearly every part: the distributor pages blocked fetching
- Lifecycle status of the LT3750/LT3751, the G2R120MT33J, the Cynergy3 D-HR, ADAS1000, STM32U5, NXP MCX-A, the Sharp memory LCDs, Molicel P28A and Broadcom fibre-optic links
- The ADS1292R respiration modulation frequency and resolution
- ECG recovery-time limits in IEC 60601-2-4 (standard not bought)
- Creepage at 2 kV (standard not bought; see `hv_safety_isolation.md`)
- Pad mass for a commercial pad pair

---

# Revision pass: version 0.2 of the parts study (27 November 2026)

After the owner's decisions D-15, D-39 and D-41 to D-46. About 17 searches and fetches, all on 27 November 2026. Same tags and the same repository rule as above: ratings and sizing arithmetic only; no designs, no build steps.

## 11. Capacitor scaling at 2.2 kV and higher temperature

- **General Atomics Series DQ, full table** (re-fetched) — <https://www.ga.com/capacitors/series-dq-defibrillator-capacitors>
  - Two biphasic parts rated **2.2 kV** that were not noted in the first pass:
    - 322DQ4196B: 196 µF, 2.2 kV, 1.20 J/cc, 5,000 cycles, 49 × 74 × 129 mm
    - 30920: 196 µF, 2.2 kV, 1.05 J/cc, 10,000 cycles, 49 × 74 × 146 mm
  - 30954: 100 µF, 3.0 kV, 0.91 J/cc, 5,000 cycles, 50 × 93 × 121 mm; 30955: 100 µF, 3.0 kV, 1.00 J/cc, 5,000 cycles, 49 × 74 × 146 mm
  - Still no weights or temperature ratings on the page.
  - [CALC] GA's quoted J/cc is 1.15–1.18 × the outside-box figure for every part checked (e.g. 320DQ4130: 260 J rated in 179.7 cm³ = 1.45 J/cc box, against 1.66 quoted; 322DQ4196B: 474 J in 467.8 cm³ = 1.01, against 1.20).
  - [CALC] Life against density: 1.66 → 1.35 J/cc doubles life (2,500 → 5,000); 1.35 → 1.04 doubles it again. Since density ∝ (film stress)², this gives life ∝ stress^−6.7 and ^−5.3. The study uses −6.6.
- **Film-capacitor life model, general** — Digi-Key technical forum, "Film Capacitors" — <https://forum.digikey.com/t/film-capacitors/12832>: service life models take the ratio of rated to applied voltage to an exponent "usually between 5 and 10"; temperature acts roughly as a factor of 2 per 10 °C. Generic, not for pulse duty specifically.
- **Pulse-capacitor voltage exponent** — SLAC-PUB-14112 — <https://www.slac.stanford.edu/pubs/slacpubs/14000/slac-pub-14112.pdf>: a fitted voltage exponent of 13.1 for the film studied (search-result summary; paper not read in full). End of life commonly taken as 5% capacitance loss (search-result summary of the same set of sources) [UNVERIFIED which paper].
- **Polypropylene voltage derating above 85 °C** (search-result summaries of film datasheets): Panasonic ECWF(A) about 1.0–1.25%/°C above 85 °C — <https://www.farnell.com/datasheets/4824820.pdf>; RS film-foil polypropylene listing 1.25%/°C above 85 °C — <https://uk.rs-online.com/web/p/film-capacitors/0114581>; Vishay metallized PP rated 85 °C, operable to 105–125 °C with derating. These are general film parts, not defibrillator parts; they show 85 °C is the usual edge of polypropylene's un-derated range.
- **TDK defibrillator product brief (2020)** — <https://www.tdk-electronics.tdk.com/download/2887416/9d4efc0f98fe0a32cdc4daff91cff523/film-capacitors-defi-pb.pdf>: metallized polypropylene, self-healing, PU or epoxy sealing, plastic or metal case, life "up to 10 000 cycles", **maximum working temperature +55 °C**, climatic category 40/93/56, 16–529 J, silicone leads 22 AWG / 15 kV DC / +150 °C, cylindrical or oval. No sizes, masses or temperature-life curve.
- **CRE 200 µF / 2,200 V part**: listed on the CRE page already cited; a separate datasheet link exists but no size or mass was retrieved [UNVERIFIED].
- **Not found:** any mass, or any life-versus-temperature curve, for a ≥ 2.2 kV defibrillator film capacitor.

## 12. Battery: two cells

- **Molicel P28A** (datasheet copy on NKON and retailer listings) — <https://www.nkon.nl/en/rechargeable/li-ion/18650-size/molicel-inr18650-p28a-2800mah-35a.html>; <https://www.imrbatteries.com/content/molicel_p28a.pdf>
  - 46 g typical (listings range 45–48 g, 50 g max); 2,800 mAh; 35 A continuous.
  - DC internal resistance 20 mΩ (10 A, 1 s) and AC 20 mΩ max at 1 kHz (temperature not stated).
  - At −20 °C: ≥ 2.00 Ah to 2.5 V after charging at 23 °C (one listing; discharge current not given) [UNVERIFIED official].
  - Discharge −40 to 60 °C; charge 0 to 60 °C.
  - Storage temperature range not retrieved [UNVERIFIED].
- Cold internal-resistance multiplier (3–5× at −20 °C) is background knowledge [BK]; no cell-specific figure found.
- **ADI MAX17320** — <https://www.analog.com/en/products/max17320.html>: 2S–4S ModelGauge m5 EZ fuel gauge with protector, internal self-discharge detection, SHA-256 authentication; protection against over/under-voltage, over-current, short circuit, over/under-temperature using external FETs; **Recommended for New Designs**.
- **TI BQ28Z610** — <https://www.ti.com/product/BQ28Z610>: 1–2 series gauge with protection and cell balancing; TI's page points to the **BQ28Z620 (ACTIVE)** as the newer version.
- **TI BQ25887** — <https://www.ti.com/product/BQ25887>: 2-cell boost charger from USB/5 V input, 2 A, 1.5 MHz, integrated cell balancing, battery thermistor with JEITA profile, 93.4% efficiency at 5 V in / 7.6 V battery / 1 A; input max 6.2 V; charge voltage 6.8–9.2 V; −40 to 85 °C. Stock not confirmed.

## 13. Sealed charging contacts (dock)

- **Glenair CB series pogo-pin connectors**, technical information: no water penetration into an unmated panel-mounted receptacle in immersion testing — <https://ww.glenair.com/cb-series-pogopin-connectors/pdf/technical-information.pdf>
- **ATTEND Technology waterproof pogo-pin connectors** (IP67, some magnetic) — <https://pdf.directindustry.com/pdf/attend-technology-inc/waterproof-pogp-pin-connectors/4596672-1117678.html>
- **Magnetic pogo connector datasheet (Farnell-hosted)**: panel part rated IP67, but the mated joint is not waterproof — <https://www.farnell.com/datasheets/4796932.pdf>
- **Yokowo S series**: IP67, IP68 and IP69K options, sealed with an O-ring between pin and housing (ConnectorSupplier e-book summary) — <https://www.connectorsupplier.com/wp-content/uploads/flipbook/19/files/basic-html/page42.html>
- No specific device-side part number with a published IP rating was settled on [UNVERIFIED].

## 14. Bluetooth modules (pre-certified, own processor)

- **Ezurio BL54L15** — <https://www.ezurio.com/bl54l15>: nRF54L15 (128 MHz Cortex-M33 + RISC-V co-processor, 1.5 MB NVM, 256 KB RAM), 14 × 10 × 1.6 mm, −40 to +105 °C, PCB trace antenna or MHF4 connector; page lists **FCC/ISED/EU/UKCA/RCM/MIC/KC** compliance and Bluetooth/802.15.4 certification; "Recommended for New Design"; customer firmware ("hostless"), nRF Connect SDK / Zephyr. No FCC ID shown on the page. The **BL54L15µ** variant is listed by Digi-Key as "working towards" these approvals — <https://www.digikey.com/en/product-highlight/e/ezurio/bl54l15u-series>
- **Raytac AN54L15Q** — Nordic news, 22 Nov 2024 — <https://www.nordicsemi.com/Nordic-news/2024/11/Raytac-AN54L15Q-multiprotocol-modules-integrate-nRF54L15-SoC>: "fully pre-certified" (approvals not named), chip / PCB / u.FL antenna variants, 9.5 × 13.7 × 2.1 mm (u.FL 9.5 × 14 × 2.1 mm), −40 to +105 °C. Reseller listings claim FCC/IC/CE/TELEC/NCC/SRRC [UNVERIFIED].
- **Silicon Labs BGM220S** — datasheet <https://www.silabs.com/documents/public/data-sheets/bgm220s-datasheet.pdf>; FCC listing <https://fccid.io/I28-BGM220S22A> (which cites original **FCC ID QOQ-BGM220S2, granted 09/09/2020**): 6 × 6 × 1.1 mm; certifications listed FCC, ISED, CE, MIC, KC; 1.40 µA in EM2 deep sleep with full RAM retention; 26 µA/MHz active. Serial co-processor (NCP) mode not confirmed in this pass [BK].
- **u-blox end-of-life notices** (modules with ready-made u-connectXpress serial firmware):
  - NINA-B31x-02B EOL notice dated 16 Feb 2026; successors NINA-B41 series — <https://content.u-blox.com/sites/default/files/documents/NINA-B31x-02B_EOL_UBXDOC-465451970-4810.pdf>
  - ANNA-B112-00B-00 replaced by ANNA-B112-02B-00 (u-connectXpress 4.0.0) — <https://shop.richardsonrfpd.com/docs/rfpd/ublox_PCN_ANNA-B112-00B-00.pdf>
  - Octopart lists NINA-B312-00B obsolete, last-time-buy 2025-12-15 (search-result summary).

## 15. Pads

- **Philips HeartStart adult pads**: 1-pack listed at package weight 0.082 kg per set; 5-pack 0.532 kg (about 106 g per set); conductor surface area 85 cm²; lead 1.2 m — <https://www.philips.ca/healthcare/product/HC989803158221/heartstart-adult-defibrillator-pads-5-pack-pads>; <https://www.philips.ca/healthcare/product/HC989803158211/heartstart-adult-defibrillator-pads-1-pack-pads>. Package weights, not pad-only mass.
- Pad storage temperature limits were not checked in this pass [UNVERIFIED].

## 16. Arithmetic done in this pass (working in PARTS_STUDY.md v0.2)

- Capacitor low / expected / high: 242 J rated energy ÷ 1.66 / 1.35 / 1.05 J/cc → 146 / 180 / 230 cm³ active → ×1.18 → about 170 / 210 / 270 cm³; × 1.3 / 1.4 / 1.5 g/cm³ → about 225 / 300 / 410 g.
- Life gain from running a 2,200 V part at 1,950 V: (2,200/1,950)^6.6 = 2.22.
- Ten-year duty: 520 weekly tests + 600 (typical) or 3,500 (heavy) rescue charges + 100 service = 1,220 or 4,120 full cycles; requirement ≥ 5,000 at 50 °C.
- Cold charging: P = V·I − I²·R per cell, V = 3.4 V at about 10% charge, R cold = 3–5 × room value. 2 × P28A at 75 mΩ: 9.2 A, 2.71 V (just above 2.5 V cut-off); 1 × P45B at 50 mΩ, 50 W: 2.32 V (fails).
- Roll-up: decision set 0.89 L / 902 g with pads; smallest set 0.63 L / 697 g; shell scaled as (inside volume)^⅔.

## 17. Not found or not checked in this pass

- Mass, temperature rating or temperature-life curve of any ≥ 2.2 kV defibrillator film capacitor.
- Cold internal resistance of the P28A or P45B; P28A storage temperature.
- FCC IDs for the Ezurio BL54L15 and Raytac AN54L15Q.
- A device-side sealed contact part number.
- Storage temperature of commercial defibrillator pads; pad-only mass.

---

# Revision pass: version 0.3 of the parts study (27 November 2026)

After D-41 was revised (capacitor storage about 70 °C) and D-48 set the ruggedness goal. About 30 searches and fetches, all on 27 November 2026. Same tags and the same repository rule as above: ratings, published measurements and sizing arithmetic only; no designs, no build steps.

**Access problems.** PubMed and PMC returned cookie or browser-check pages; several maker PDFs (ZOLL pad instructions, Philips FRx technical manual) returned 404 or could not be parsed. Where a figure below comes from a search-result extract of a maker's document rather than the document itself, it is marked **(extract)**. Those figures should be checked against the printed manuals before they are relied on.

## 18. Defibrillation pads and commercial AEDs: storage limits

- **ZOLL CPR-D-padz** instructions for use (8900-0800-01, multilingual) — <https://zoll.com/-/media/product-manuals/electrodes/english/ifu-8900-0800-01-cpr-d-padz-with-kit-en-intl.ashx> (fetch returned 404; figures from search extracts of this and the English IFU): operating 0 to 50 °C; **short-term storage −30 to 65 °C**; **long-term storage 0 to 35 °C** (extract). Time limit of "short term" not found [UNVERIFIED]. US device database (GUDID) entry for a CPR-D-padz lists storage 0–35 °C — <https://accessgudid.nlm.nih.gov/devices/00847946016333> (extract).
- **ZOLL AED Plus**: device storage −30 to 60 °C, operating 0 to 50 °C (spec sheet, extract) — <https://www.healthfirst.com/wp-content/uploads/2026/08/Zoll-AED-Plus-Spec-Sheet-12.30.2024.pdf>. A ZOLL manual warns that storing outside the recommended conditions may damage the pads and/or batteries or shorten their useful life (extract).
- **Philips HeartStart FRx brochure** (read; PDF text extracted) — <https://www.documents.philips.com/doclib/enc/fetch/577774/577795/FRx_Brochure.pdf>: operating/standby **0 to 50 °C**; transient operating −20 to 50 °C for 20 minutes or less after a rapid move from 20 °C. Battery standby life is quoted "within the specified standby temperature range".
- **Philips FRx technical manual** (extract; the manual itself was not opened): "Storage/shipping (with battery and pads case): −20° to 60° C for up to 1 week". Longer-term storage line not seen [UNVERIFIED].
- **Philips SMART Pads II** (989803139261): storage 0–50 °C (Philips UK product page, extract) — <https://www.philips.co.uk/healthcare/product/HC989803139261/smart-pads-ii-pads>
- **Philips Adult SMART Pads cartridge M5071A** (HS1/OnSite/Home): storage **10–43 °C** (Philips product pages, extract) — <https://acc.philips.co.uk/healthcare/product/HCM5071A/adult-smart-pads-cartridge-pads>
- **HeartSine samaritan PAD 350P/360P** data sheets (extract) — <https://www.stryker.com/content/dam/stryker/ems/products/lifelink-central/aed-program-administrator/heartsine-literature/H009-032-340-3_350P_360P_Data_US_0820_web.pdf>: operating/standby 0 to 50 °C; **transport −10 to 50 °C for up to two days**; return to standby temperature for 24 h before use. 450P data sheet: transport 0 to 50 °C only — <https://www.stryker.com/content/dam/stryker/ems/products/lifelink-central/aed-program-administrator/heartsine-literature/H009-041-001-5_450P_Data_US_0820_web.pdf>. Older HeartSine technical note (sam300): long-term storage outside 10–50 °C "may adversely affect the performance" — <https://heartsinelive.s3.amazonaws.com/uploads/sites/14/2013/11/H009-013-005-0_Technical-Note_Temperature_sam300-sam300P.pdf>. Pad-Pak shelf life 4 years.
- **Stryker LIFEPAK CR2** EMEA data sheet (extract) — <https://www.stryker.com/content/dam/stryker/ems/products/lifepak-cr2/eu/resources/3328414_a_emea-en_lpcr2_data_sheet.pdf>: storage with battery and electrodes **−30 to +60 °C, maximum exposure one week**; long-term storage **15 to 35 °C**; support article: long-term storage at the higher end shortens battery and electrode life — <https://mts.stryker.com/support/solutions/articles/60000615278-lifepak-cr2-storage>. State-project FAQ gives long-term 15–25 °C (conflict) — <https://www.stryker.com/content/dam/stryker/ems/aed-state-project/resources/lifepak-cr2-state-project-faq.pdf>. GUDID handling −30 to 60 °C.
- **Avive Connect** (help centre, extract) — <https://help.avive.life/hc/en-us/articles/25763026137243-How-should-I-store-my-Avive-Connect-AED>; <https://help.avive.life/hc/en-us/sections/25760065907739-Device-Specifications>: operating 0 to 50 °C; **long-term storage 5 to 40 °C**; **short-term −20 to 60 °C for up to 14 days**; return to 0–50 °C for 24 h; battery charging optimum 0 to 45 °C.
- **Schiller FRED PA-1** technical data (extract): operating −5 to 40 °C; transport/storage −20 to 50 °C — <https://cdn.shopify.com/s/files/1/0834/9256/9409/files/FRED-PA-1-User-Manual_66511fa6-c3a1-4988-89ea-4eb133fa2c79.pdf>. No pad storage figure found [UNVERIFIED]; easyport plus not found.
- Not searched: CellAED storage range.

## 19. Battery cells: heat limits

- **Molicel INR18650-P28A** specification (document INR18650P28A-01, ver. 0.1, third-party copies) — <https://www.nkon.nl/amfile/file/download/file/121/product/3518>; <https://www.imrbatteries.com/content/molicel_p28a.pdf> (extract): "Storage temperature for shipping cell < 35 °C"; "Recommended temperature less than 23 °C for long term storage"; shelf life 1 year; charge 0–60 °C, discharge −40 to 60 °C. **No short-term storage maximum found.**
- **Samsung SDI INR18650-25R** specification (2014, third-party copies) — <https://www.kiwiquads.co.nz/wp-content/uploads/INR18650-25R-datasheet.pdf>; lygte-info review: storage with ≥ 90% recovery: **1.5 years at −30 to 25 °C; 3 months at −30 to 45 °C; 1 month at −30 to 60 °C**. Used as the class reference for high-drain NMC/NCA cells.
- **A123 / Lithium Werks ANR26650M1B (LFP)** data sheet, 2022 (read; text extracted) — <https://lithiumwerks.com/wp-content/uploads/2022/08/26650-Power-Cell-080322.pdf>: 3.3 V, 2.6 Ah typical (2.5 min), 8.58 Wh, **76.0 g**, 52 A continuous, 120 A 10 s pulse, operating −30 to 60 °C, charge 0 to 60 °C (< 250 mA below 0 °C), **storage −40 to 70 °C**, > 4,000 cycles at 1C. The older A123 sheet MD100113 gives storage −40 to 60 °C (conflict).
- **Saft MP 174565 xtd** data sheet (read; text extracted; Master Instruments copy) — <http://www.master-instruments.com.au/file/67746/1/Saft-MP174565XTD%20CLIP.pdf>: 4.0 Ah, 3.65 V, 14.6 Wh, **116 g**, 150 Wh/kg, **5 A maximum continuous discharge**, 4 A charge; charge −30 to +85 °C, discharge −40 to +85 °C; storage recommended +15 to +30 °C, **allowable −40 to +85 °C**; ≥ 2,700 cycles; typical applications include medical devices; ISO 13485. Thickness table shows growth from 18.65 mm (new) to 20.75 mm after 1 year float at 60 °C and 21.45 mm after 100 cycles at 85 °C.
- [CALC] Saft xtd cannot supply the charger alone: 7.6 A average ÷ 5 A per cell → two parallel strings of two = 4 cells × 116 g = 464 g.
- [BK, not re-checked] Thermal runaway of healthy lithium-ion cells starts well above 100 °C; charging hot and later use of a heat-damaged cell are the realistic hazards; MAX17320 operating −40 to 85 °C.

## 20. Other parts

- **Sharp memory LCD**: operating −20 to +70 °C for LS013B7DH05 (Sharp brochure) — <https://www.mouser.com/pdfdocs/Sharp_Memory_LCD_Brochure_2015746065.PDF>; LS027B7DH01 −20 to +70 °C (Arrow, Future listings). Sharp's LS027B7DH01A specification on Avnet gives storage guidance only qualitatively and warns that temperatures outside the rated range can permanently change the liquid crystal (search summary) — <https://embedded.avnet.com/product/ls027b7dh01a>. Numeric storage range not found [UNVERIFIED].
- **3M VHB 5952** technical data sheet (extract) — <https://multimedia.3m.com/mws/media/2369604O/3m-vhb-tape-5952-technical-data-sheet.pdf>: long-term temperature 93 °C (200 °F), defined as holding 250 g per 0.5 in² in static shear for 10,000 minutes; short-term rating defined by 4 h conditioning. 3M says the data are typical, not for specification.
- Speaker, piezo, enclosure plastics and silicone gasket ratings: not checked; background knowledge only [BK].

## 21. Parked-car temperatures

- **McLaren C, Null J, Quinn J. Heat stress from enclosed vehicles: moderate ambient temperatures cause significant temperature rise in enclosed vehicles. Pediatrics 2005;116:e109–e112.** Abstract and Stanford release — <https://pubmed.ncbi.nlm.nih.gov/15995010/>; <https://med.stanford.edu/news/all-news/2005/07/parked-cars-get-dangerously-hot-even-on-cool-days-stanford-study-finds.html>: cabin rose on average about 40 °F (22 °C) in an hour on sunny days with highs of 72–96 °F (22–36 °C), regardless of ambient; **80% of the rise in the first 30 minutes**; cracked windows did not slow heating.
- **Vanos JK, Middel A, Poletti MN, Selover NJ. Evaluating the impact of solar radiation on pediatric heat balance within enclosed, hot vehicles. Temperature 2018.** ASU release — <https://news.asu.edu/20180516-discoveries-asu-study-hot-cars-can-hit-deadly-temperatures-within-one-hour>; full text at <https://pmc.ncbi.nlm.nih.gov/articles/PMC6204985> (blocked by a browser check). Six vehicles, Tempe, Arizona, three days above 95 °F (35 °C). After 1 h in sun: cabin air 116 °F (**46.7 °C**), dashboard 157 °F (**69.4 °C**), steering wheel 127 °F (52.8 °C), seat 123 °F (50.6 °C). One dashboard reached **192 °F (88.9 °C)** (state climatologist, quoted in Live Science / press coverage — <https://www.livescience.com/62651-how-hot-cars-get.html>). After 1 h in shade: cabin 100 °F (37.8 °C), dashboard 118 °F (47.8 °C), wheel 107 °F, seat 105 °F.
- **Dadour IR, Almanjahie I, Fowkes ND, Keady G, Vijayan K. Temperature variations in a parked vehicle. Forensic Sci Int 2011;207:205–211.** Abstract (Curtin repository) — <https://espace.curtin.edu.au/handle/20.500.11937/2661?show=full>: cabin can exceed ambient by more than 20 °C in sun; black car about 5 °C hotter than white; driver's window lowered 2.5 cm reduces cabin temperature about 3 °C.
- **Marty W, Sigrist T, Wyler D. Temperature variations in automobiles in various weather conditions. Am J Forensic Med Pathol 2001;22:215–219.** Abstract (via search extract; PubMed 11563725 blocked): summer interior temperature extremes **up to 89 °C**; whether air or surface not stated in the abstract.
- **Adrenaline auto-injector pilot study, Lodz, Poland, J Allergy Clin Immunol Pract 2018** (press reports) — <https://allergyasthmanetwork.org/news/why-you-shouldnt-keep-epinephrine-hot-car/>: black sedan, items in glovebox, trunk and cabin shelf with no direct sun, 12 h; car temperature peaked at **143.6 °F (62 °C)**; drug loss 14.3% glovebox, 13.3% cabin, 3.3% trunk. Read here only as a hint that the glovebox runs at least as hot as the cabin and the trunk cooler; no compartment temperatures were reported [UNVERIFIED].
- **Emergency-vehicle storage**: South African EMS sample, 13.1–**56.8 °C**, highest in a primary response vehicle — <https://www.journals.ac.za/sajpec/article/view/4480>; Essex & Herts Air Ambulance (2022–2023), 102,524 readings, −9 to **46.8 °C** — <https://www.cambridge.org/core/product/BE6DFFC05945DC1BDE2F83541FD898B9/core-reader>; helicopter EMS drug box (Prehospital Emergency Care 1999): summer daytime maxima averaged 31.2 °C — <https://pubmed.ncbi.nlm.nih.gov/9921737/> (all extracts).
- **Grundstein A, Meentemeyer V, Dowd J. Maximum vehicle cabin temperatures under different meteorological conditions. Int J Biometeorol 2009;53:255–261.** Cited, not read; figures not retrieved [UNVERIFIED].
- **Not found:** any controlled measurement of trunk or glovebox temperature; any source for cabin **air** at 70 °C or a dashboard at 100 °C (the version 0.2 notes).

## 22. Arithmetic done in this pass (working in PARTS_STUDY.md v0.3)

- Working-temperature density penalty (life halves per 10 K, exponent 6.6): 60 °C: 2^(−0.5/6.6) = 0.949 stress, × 0.90 density → 1.49 J/cc; 65 °C: 2^(−1/6.6) = 0.900, × 0.81 → 1.35 J/cc.
- Capacitor, 242 J rated, +18% can, 1.4 g/cm³: 65 °C → 212 cm³ / 297 g; 60 °C → 191 / 268; 55 °C → 172 / 241 at 1.4 g/cm³ (225 g at the low case's 1.3 g/cm³).
- 2,500 instead of 5,000 cycles: stress × 2^(1/6.6) = 1.111, density × 1.234: 1.35 → 1.66 J/cc (172 cm³, 241 g); with 60 °C working, 1.49 → 1.84 J/cc (155 cm³, 217 g).
- Roll-up method unchanged (parts ÷ packing factor; shell scaled as (inside / v0.1 inside)^⅔ from 110 cm³ / 140 g (a) or 90 cm³ / 100 g (b)). v0.3 adds 1 cm³ / 1 g of temperature sensing. Results with pads: (a) 0.89 L / 903 g; (b) 0.64 L / 699 g; (a) low / high capacitor 817 / 1,030 g; (b) expected 783 g; (a) 60 °C working 868 g; (a) 2,500 cycles 832 g; option C (2 × LFP 26650, +60 g and about +36 cm³ of cells) 975 g.
- Device thermal time constant: 0.8 kg × 900 J/kg·K ÷ 0.49 W/K = 1,470 s ≈ 25 min. With a 10 mm foam pouch (0.035 W/m·K, 0.05 m²): 0.175 W/K in series with 0.49 W/K = 0.13 W/K → 93 min.
- Temperature logging power [EST]: 10–20 µW → 0.24–0.48 mWh/day, about 1% of standby.

---

# Revision pass: version 0.4 of the parts study (27 November 2026)

After D-09, D-41, D-43, D-48 and D-51 to D-53. No new web searches: this pass is re-derivation from the sources above. Same tags and repository rule.

## 23. Arithmetic done in this pass (working in PARTS_STUDY.md v0.4)

- **Combined capacitor, 60 °C working and 2,500 charges, 2,200 V, 100 µF (242 J rated energy):**
  - Version 0.3 shortcut: 1.66 × 2^(−1/6.6) [60 °C] × 2^(2/6.6) [half the life] = 1.66 × 0.90 × 1.23 = **1.84 J/cc**.
  - Full method: GA 320DQ4130 1.66 J/cc, 2,500 cycles at rated voltage → × (2,200/1,950)^6.6 = 2.22 → about 5,550 cycles at 1,950 V; at 60 °C × 2^(−0.5) → 3,924; spare against 2,500 = 1.57; allowed density 1.66 × 1.57^(1/3.3) = **1.90 J/cc**. With a life exponent of 10 instead of 6.6: 1.66 × 1.57^(0.2) = 1.81 J/cc.
  - **Capped**: no published defibrillator film part exceeds 1.66 J/cc (GA, 2.0 kV); at 2.2 kV the best published is 1.20 J/cc (GA 322DQ4196B). CDE press material says "as high as 2 J/cc" with no part named (section 3 above). Expected case therefore 1.66 J/cc; low case 2.0 J/cc; high case 1.35 J/cc (GA's real 5,000-cycle grade).
  - Boxed volume (+18%) and mass: low 242/2.0 = 121 → 143 cm³, × 1.3 = 186 g (about 145 / 185); expected 242/1.66 = 146 → 172 cm³, × 1.4 = 241 g (about 170 / 240); high 242/1.35 = 179 → 212 cm³, × 1.5 = 317 g (about 210 / 315).
  - Heavy-duty later option (5,000 charges at 60 °C): version 0.3 figure 190 cm³ / 270 g.
- **Cycle-life requirement:** typical profile 520 self-tests + 600 rescue charges + 100 service = 1,220 in 10 years; × 2 = 2,440 → 2,500. Heavy profile 4,120 → 412 a year → reaches 2,500 in 6.1 years. Charge-hold 2,500 × 30 s = 21 h, × 3 ≈ 60 h.
- **Roll-up** (same method as before; Bluetooth module −1 g with its 2 cm³ kept as empty reserved volume; data contacts +1 cm³, +1 g):
  - Lower-risk set: parts 319 cm³ / 594 g → inside 638 → shell 109 cm³ / 138 g → 0.75 L / 732 g → **0.81 L / 832 g with pads**.
  - Smallest set: parts 269 / 470 → inside 448 → shell 92 / 103 → 0.54 L / 573 g → **0.59 L / 653 g with pads**.
  - Capacitor cases with pads: (a) 770 / 832 / 919 g; (b) 653 / 714 / 798 g. Heavy-duty capacitor (a) 868 g. 2 × 21700 cells (a) about 890 g.
  - D-41 + D-53 applied alone to the version 0.1 set: (a) −22 cm³, −8 g; (b) −10 cm³, −23 g.
- **Vehicle pouch requirement:** bare device time constant about 25 min; requirement ≥ 75 min (×3), target 90 min. Half-rise times: 75 × ln 2 = 52 min; 90 × ln 2 = 62 min. 2-hour stop at 60 °C from 30 °C: pouch (90 min) 30 + 30 × (1 − e^(−120/90)) = 52 °C; bare 30 + 30 × (1 − e^(−120/25)) = 60 °C. 10 mm foam at 0.035 W/m·K over 0.05 m² → 0.175 W/K, in series with 0.49 W/K → 0.13 W/K → 720 J/K ÷ 0.13 = 93 min.
- **Standby without Bluetooth:** version 0.3's 0.5–1 mW less 30–90 µW advertising → 0.45–0.9 mW; with self-tests and logging 1.05–1.5 mW, 0.025–0.036 Wh a day.

## 24. Not checked in this pass

- No pouch product was checked against the requirement; product classes are background knowledge.
- No sealed contact part with a data rating was checked for the dock data link.
