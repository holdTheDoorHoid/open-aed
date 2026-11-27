#!/usr/bin/env bash
# Regenerate every KiCad sheet from its spec and run all checks.
#   1. unit tests of the generator (python -m unittest)
#   2. integration test: a tiny two-instance hierarchical project through kicad-cli (ERC + netlist)
#   3. project symbol libraries, all sheets, root sheets, .kicad_pro, lib tables
#   4. ERC (errors fail; warnings must be accepted in a spec), netlist round trip, PDF export
#   5. PCB: created once with the pcbnew API if missing, then DRC against the accepted-type list in
#      tools/schgen/build.py. Parts are put on the boards by tools/pcbgen/populate.py, run by hand, never here.
# Exits non-zero on any failure. Needs KiCad 10 as the user flatpak org.kicad.KiCad.
set -euo pipefail
cd "$(dirname "$0")/.."
echo "== unit tests"
python3 -m unittest discover -s tools/schgen/tests -t . "$@"

echo "== build and checks"
python3 -m tools.schgen.build
