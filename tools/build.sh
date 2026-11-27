#!/usr/bin/env bash
# Regenerate every KiCad sheet of the Open AED project from its spec and run all checks.
#   1. unit tests of the generator (python -m unittest), including a tiny hierarchical project pushed
#      through kicad-cli (ERC + netlist) when KiCad is installed
#   2. project symbol libraries (hardware/lib/src/aed_*.toml -> hardware/lib/aed_*.kicad_sym) and their lint
#   3. all sheets of hardware/aed-main (placeholder pages for sheets without a spec), root sheet, .kicad_pro,
#      lib tables; every sheet must carry the NOT FOR CONSTRUCTION notice
#   4. ERC (errors fail; warnings must be accepted in a spec), netlist round trip, PDF export
# PCB generation and DRC are out of scope until layout starts.
# Exits non-zero on any failure. Needs KiCad 10 as the user flatpak org.kicad.KiCad.
# See docs/HARDWARE_WORKFLOW.md.
set -euo pipefail
cd "$(dirname "$0")/.."
echo "== unit tests"
python3 -m unittest discover -s tools/schgen/tests -t . "$@"

echo "== build and checks"
python3 -m tools.schgen.build

echo "== symbol library lint"
python3 tools/libcheck.py
