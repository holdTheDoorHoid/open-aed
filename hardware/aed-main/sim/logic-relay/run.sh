#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# Runs every netlist in this directory with ngspice in batch mode.
# Full logs go to out/<netlist>.log; every 'RESULT' line is collected in out/summary.txt.
# Usage: bash run.sh            (all netlists, about 10-15 minutes in total)
#        bash run.sh timers     (only timers.cir)
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }

if [ "$#" -gt 0 ]; then
  nets=("$@")
else
  nets=(timers rail_up races det_permit relay_supply battery sense imp_noise)
fi

: > out/summary.txt
for n in "${nets[@]}"; do
  f="${n%.cir}.cir"
  [ -f "$f" ] || { echo "missing $f" >&2; exit 1; }
  echo "== $f"
  start=$(date +%s)
  ngspice -b "$f" > "out/${f%.cir}.log" 2>&1 || echo "   ngspice returned an error status for $f (see out/${f%.cir}.log)"
  end=$(date +%s)
  nres=$(grep -c '^RESULT' "out/${f%.cir}.log" || true)
  nerr=$(grep -ci 'error' "out/${f%.cir}.log" || true)
  echo "   ${nres} result lines, ${nerr} lines mentioning 'error', $((end-start)) s"
  { echo "## $f"; grep '^RESULT' "out/${f%.cir}.log" || true; echo; } >> out/summary.txt
done
echo "summary: out/summary.txt"
