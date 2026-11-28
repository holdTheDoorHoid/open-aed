#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# Runs the WP-D revision 0.3 timing checks (ngspice, batch mode). Logs in out/, RESULT lines in out/summary.txt.
#   timers_rev3.cir   phase-1 / phase-2 caps, window-start clock delay, re-arm release (corners)
#   xfer_bounce.cir   XFER after aux-contact bounce with the 10 us filter and Q1201; Q1201 leakage sensitivity
# Models: ../logic-relay/models/behavioural.lib. About 1 minute in total.
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }
: > out/summary.txt
for n in timers_rev3 xfer_bounce; do
  echo "== $n" | tee -a out/summary.txt
  ngspice -b "$n.cir" > "out/$n.log" 2>&1 || echo "   ngspice exit code $? for $n" | tee -a out/summary.txt
  grep '^RESULT' "out/$n.log" >> out/summary.txt
done
echo "done: out/summary.txt"
