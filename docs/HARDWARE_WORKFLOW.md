# How the circuit diagrams are made and checked

Version 0.1, 27 November 2026. Decision D-58. Adapted from the owner's wildlife-station project, which uses the same
tools.

> **NOT FOR CONSTRUCTION — unreviewed design.** The circuit diagrams in `hardware/` are published for review
> (D-13). They have not been reviewed by a qualified high-voltage or medical-device engineer. Nothing here tells
> anyone to build or power a circuit (SAFETY.md §2a).

## 1. In plain language

The circuit diagrams are not drawn by hand. Each page ("sheet") is first written as a **pin table**: a text file
that says, part by part, "pin 3 of this chip goes to the wire called `SHOCK_REQ`". A small program reads those
tables and draws the KiCad page from them: it places the parts in titled boxes, puts a short wire and a name tag
on every pin, and fills in the title block. Every page's title block carries the line
**"NOT FOR CONSTRUCTION — unreviewed design"**; the build fails if any page is missing it.

Then the program asks KiCad itself to check the result:

1. **KiCad's electrical rules check (ERC).** Any error stops the build.
2. **A round trip.** KiCad exports its own list of what is connected to what (the netlist). The program compares
   every connection in it with the pin tables. One pin on the wrong wire stops the build.
3. **Name check against the symbol.** Each pin-table row gives the pin *number* and the pin *name* from the
   datasheet. If KiCad's symbol has a different name on that number, the build stops. This catches the
   commonest drawing mistake: the right name on the wrong pin.

Why this way: a reviewer can check a pin table against a datasheet line by line, and every change is a one-line
edit that shows clearly in the project history. **Until the schematic is frozen, the pin tables are the source of
truth. Never edit a generated `.kicad_sch` file by hand** — the next build overwrites it.

Today every page is a **placeholder**: a box with the block's name, its design-document section and which capture
work package will draw it. The pages fill in as the work packages in `hardware/aed-main/contract.md` are captured.

## 2. Where things are

| Path | What it is |
|---|---|
| `docs/HARDWARE_DESIGN.md` | The design: every block, its parts, its arithmetic, what is unknown |
| `hardware/aed-main/contract.md` | The agreement that lets several people draw pages at once: who draws which page, reference numbers, the name of every wire that crosses between pages, and which page fits each default resistor |
| `hardware/aed-main/hierarchy.toml` | The list of pages in order, the title-block notice, and the net classes (wire categories such as "high voltage" and "patient") |
| `hardware/aed-main/specs/*.toml` | The pin tables. `00_root.toml` is the front page; one file per captured page |
| `hardware/lib/src/aed_*.toml` | Pin lists for parts KiCad does not have, one file per work package; the build turns each into `hardware/lib/aed_*.kicad_sym` |
| `hardware/aed-main/aed-main.kicad_pro`, `*.kicad_sch`, `sym-lib-table`, `fp-lib-table` | The generated KiCad project (committed, so it can be opened without running anything) |
| `hardware/aed-main/pdf/aed-main.pdf` | The whole schematic as a PDF |
| `tools/build.sh` | One command that runs everything below |
| `tools/schgen/` | The generator (Python 3, standard library only) and its tests |
| `tools/xcheck.py` | A report (never fails the build) that lists every cross-page wire, its pages and its default resistors, to compare with the contract |
| `tools/libcheck.py` | Lint for the project's own symbol libraries |
| `build/` (not committed) | Check reports from the last build |

## 3. Opening the project in KiCad (owner)

KiCad 10 is installed as a user flatpak.

- Start KiCad from the desktop menu (or `flatpak run org.kicad.KiCad`), choose **File → Open Project** and open
  `hardware/aed-main/aed-main.kicad_pro`. The front page lists the blocks; double-click a box to go into that page.
- **Just to read it:** open `hardware/aed-main/pdf/aed-main.pdf`.
- Parts drawn with a red cross are "do not fit" in the default build. Every part carries hidden fields (`MPN`,
  `Manufacturer`, `Datasheet`, `DocRef` …); hover over it or open *Symbol Properties* to see them.
- If you look and change something in the KiCad window, **do not save it**: the next build overwrites it. Change
  the pin table instead, or ask whoever owns that page.

## 4. Regenerating and checking

From the repository root:

```
bash tools/build.sh                         # tests, then generate and check everything (about 15 s today)
python3 -m tools.schgen.build --no-pdf      # skip the PDF
python3 -m tools.schgen.build --no-kicad    # generate only, without KiCad's checks
python3 tools/xcheck.py                     # cross-page wire report against the contract
```

What makes the build fail:

| Check | Fails when |
|---|---|
| Unit tests | any generator test fails (`python3 -m unittest discover -s tools/schgen/tests -t .`) |
| Pin-table validation | unknown keys, duplicate references, a pin listed twice, a wire with only one pin on a page, a wire declared in two categories |
| Symbol cross-check | a symbol pin missing from the table (every pin must be listed; `NC` for unused), a pin the symbol does not have, a pin name that differs from the symbol |
| Notice | the `NOT FOR CONSTRUCTION` notice is missing from `hierarchy.toml` or from any generated page |
| Rail flags | the same supply rail is flagged as "made here" on two pages |
| Footprints | a part's footprint is in no library (unless marked as a footprint still to be made) |
| ERC | any error, or any warning not accepted with a written reason in a pin table |
| Netlist round trip | any connection differs from the pin tables |
| PDF | KiCad cannot plot the schematic |
| Symbol library lint | a project symbol lacks its datasheet, maker, part number or pin-check field, or has duplicate or off-grid pins |

Printed-board layout and its checks are not part of this build yet; the layout file will be created when layout
starts.

## 5. The pin-table format (what a reviewer reads)

```toml
[sheet]
name = "09_interlock"
title = "Hardware interlock"
source = "docs/HARDWARE_DESIGN.md §B8"

[nets]
power  = ["GND", "5V0_SYS", "3V3_MAIN", "3V3_SUP"]   # supply rails, drawn as power symbols
global = ["SHOCK_REQ", "PERMIT_CLK", "HOLD_OK"]      # wires that leave this page (names from contract.md)

[[group]]
id = "bridge_power"
title = "Bridge drive power: request AND permit AND hold"
note = "Why this box looks the way it does."

[[part]]
ref = "U901"
group = "bridge_power"
lib_id = "aed_interlock:TPS22918"
value = "TPS22918DBVR"
footprint = "Package_TO_SOT_SMD:SOT-23-6"
mpn = "TPS22918DBVR"
manufacturer = "Texas Instruments"
datasheet = "https://www.ti.com/lit/ds/symlink/tps22918.pdf"
pins = """
# pin  name-in-symbol  net
  1    VIN             5V0_SYS
  3    ON              SHOCK_REQ_BUF
"""
```

(An illustration of the format only; the real page has not been captured.) Two-pin parts can say
`between = ["NET_A", "NET_B"]`. `NC` in the net column draws a no-connect flag. References follow the contract's
numbering: page NN uses NN01–NN99 (page 9 uses 901–999).

## 6. Custom symbols

A part KiCad does not have is described in the owning work package's library file
`hardware/lib/src/aed_<package>.toml` as a pin list copied from the datasheet table, with the datasheet link and
table named. Its "pinout verified" field reads "NO — drafted from datasheet, needs human check" until a person has
checked it against the PDF and changed it.

## 7. Schematic freeze

When the design has been reviewed and the pages stop changing weekly, the KiCad files become the source, the
generator is retired, and from then on people edit in KiCad. That moment is an owner decision.
