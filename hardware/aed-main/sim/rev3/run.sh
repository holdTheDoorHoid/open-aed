#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# Runs the revision 0.3 check simulations (ngspice, batch mode). Logs in out/, one-line results in
# out/summary.txt. About 3-5 minutes in total.
#   held_sensor_rev3.cir   revised "held" sensor thresholds (DC sweep)
#   hold_pump_rev3.cir     hold pump + release + two sensors, clock 1-2 kHz, corners
#   carrier_loss.cir       carrier-loss detector on a high-side gate island
#   per_phase_caps.sh      per-phase hardware caps: worst delivered energy (re-uses ../bridge/q3_shock_energy.cir)
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }
: > out/summary.txt
for n in held_sensor_rev3 hold_pump_rev3 carrier_loss; do
  echo "== $n" | tee -a out/summary.txt
  ngspice -b "$n.cir" > "out/$n.log" 2>&1 || echo "   ngspice exit code $? for $n" | tee -a out/summary.txt
  grep -E '^(HELD|PUMP|RESULT)' "out/$n.log" >> out/summary.txt
done
echo "== per_phase_caps" | tee -a out/summary.txt
bash per_phase_caps.sh > out/per_phase_caps.log 2>&1
cat out/per_phase_caps.txt >> out/summary.txt
echo "done: out/summary.txt"
