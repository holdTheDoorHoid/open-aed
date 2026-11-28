#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# WP-E revision 0.3 check runs for sheet 10_hv_charger (ngspice, batch mode). Logs in out/, one-line results
# in out/summary.txt. About 1-2 minutes.
#   l3_trip_rev3.cir    primary-side trip band with RBG 876 ohm, LPRI rules, reflected voltage (arithmetic only)
#   charger_rev3.cir    charge time with transformer efficiency, v0.3 inductance tolerance and standing load;
#                       flyback duration at the trip top; clamp energy at the trip top
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }
: > out/summary.txt
for n in l3_trip_rev3 charger_rev3; do
  echo "== $n" | tee -a out/summary.txt
  ngspice -b "$n.cir" > "out/$n.log" 2>&1 || echo "   ngspice exit code $? for $n" | tee -a out/summary.txt
  grep -E '^(L3|MARGIN|LPRI|TVS|RECT|CORNER|RESULT|SENSE|CLAMP)' "out/$n.log" >> out/summary.txt
done
echo "done: out/summary.txt"
