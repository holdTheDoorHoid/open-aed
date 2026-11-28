#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# WP-F revision 0.3 check runs of sheet 11_hv_storage AS DRAWN (ngspice 42, batch mode). Logs in out/, one-line
# results in out/summary.txt. Roughly 30-60 minutes on an idle 8-core machine, dominated by stack_static_wpf.
#   python3 gen_ctl.py          writes the two generated control scripts (already committed)
#   stack_static_wpf.cir        static sharing: nominal, hot diode leakage, Monte Carlo, every single part fault
#   stack_dynamic_wpf.cir       dynamic sharing, BOUNDING events: 1 us step, shock-like fall, 2 ms rise; C0G patterns;
#                               Monte Carlo; one device shorted
#   stack_shock_wpf.cir         realistic charge, shock and 1 s recharge with each device shorted
#   discharge_wpf.cir           discharge times, release edge, resistor pulse power and energy
#   held_threshold_wpf.cir      the two held sensors' thresholds at three corners, zener shorted / open
#   hold_held_wpf.cir           hold pump, release and both held sensors across -20..+70 C, zener faults
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }
python3 gen_ctl.py
: > out/summary.txt
for n in held_threshold_wpf hold_held_wpf discharge_wpf stack_dynamic_wpf stack_shock_wpf stack_static_wpf; do
  echo "== $n" | tee -a out/summary.txt
  start=$(date +%s)
  ngspice -b "$n.cir" > "out/$n.log" 2>&1 || echo "   ngspice exit code $? for $n" | tee -a out/summary.txt
  grep -E '^(STATIC|DYN|SHOCK|DUMP|PULSE|HELD|HOLD)' "out/$n.log" >> out/summary.txt
  echo "   $n: $(( $(date +%s) - start )) s; lines mentioning error: $(grep -ci error "out/$n.log") (measurements that are meant not to occur report -1)" | tee -a out/summary.txt
done
echo "done: out/summary.txt"
