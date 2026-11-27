#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# Runs every simulation in this directory with ngspice (batch mode) and writes the full logs and a
# one-line-per-result summary into out/. Total run time on the design computer: roughly 1-2 hours,
# dominated by the Monte Carlo runs in stack_static_n4/n5.
# Usage: bash run.sh            (all)
#        bash run.sh quick      (everything except the two long static-sharing Monte Carlo runs)
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }

nets="held_detector hold_pump discharge charger ovp_limits bleed stack_dynamic_n4 stack_dynamic_n5"
if [ "${1:-}" != "quick" ]; then
  nets="$nets stack_static_n4 stack_static_n5"
fi

: > out/summary.txt
fail=0
for n in $nets; do
  echo "== $n" | tee -a out/summary.txt
  start=$(date +%s)
  if ngspice -b "$n.cir" > "out/$n.log" 2>&1; then
    status=ok
  else
    status="ngspice exit code $?"
    fail=1
  fi
  # result lines are the ones the control scripts echo with a leading keyword
  grep -E '^(HELD|LOAD|PUMP|DUMP|CORNER|RESULT|LEAK|L1|L2|L3|LATCH|BLEED|DYN|STACK|CONFIG|CASE)' "out/$n.log" >> out/summary.txt
  # convergence or measurement problems are reported, not hidden
  nerr=$(grep -ciE 'error|timestep too small|aborted' "out/$n.log" || true)
  echo "   $n: $status, $(( $(date +%s) - start )) s, lines mentioning error/abort: $nerr (see out/$n.log)" | tee -a out/summary.txt
done
exit $fail
