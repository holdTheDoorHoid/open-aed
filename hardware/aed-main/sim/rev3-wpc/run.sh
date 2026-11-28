#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# WP-C revision 0.3 checks of the latching over-voltage comparator on sheet 08 (about 1 minute).
#   ovp_band.py     worst-case and Monte Carlo trip band from the parts drawn (calculation, Python 3)
#   ovp_latch.cir   latch, edge-only reset, test injection, eleven healthy/fault cases (ngspice, batch)
# Logs in out/, one-line results in out/summary.txt.
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }
: > out/summary.txt
echo "== ovp_band" | tee -a out/summary.txt
python3 ovp_band.py > out/ovp_band.log 2>&1
cat out/ovp_band.log >> out/summary.txt
echo "== ovp_latch" | tee -a out/summary.txt
ngspice -b ovp_latch.cir > out/ovp_latch.log 2>&1 || echo "   ngspice exit code $?" | tee -a out/summary.txt
grep -E '^RESULT' out/ovp_latch.log >> out/summary.txt
echo "done: out/summary.txt"
