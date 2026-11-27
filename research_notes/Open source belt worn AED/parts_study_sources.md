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
