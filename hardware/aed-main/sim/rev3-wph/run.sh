#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# Runs the WP-H revision 0.3 check simulations (ngspice, batch mode). Logs in out/, one-line results in
# out/summary.txt. About 10 minutes in total, most of it relay_supply_wph.cir.
#   relay_supply_wph.cir     sheet 14: coil-supply pre-charge (K_SUPPLY_EN), coil start (K_EN), hold-up
#   imp_ecg_noise_wph.cir    sheet 15: impedance noise with R1528/R1529; ECG gain and noise
#   fe_protect_wph.cir       sheet 15: defibrillation protection with the new ECG taps (external 5 kV, own shock)
# Usage: bash run.sh            (all)
#        bash run.sh fe_protect_wph   (one)
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }
if [ "$#" -gt 0 ]; then nets=("$@"); else nets=(imp_ecg_noise_wph fe_protect_wph relay_supply_wph); fi
: > out/summary.txt
for n in "${nets[@]}"; do
  f="${n%.cir}"
  echo "== $f" | tee -a out/summary.txt
  ngspice -b "$f.cir" > "out/$f.log" 2>&1 || echo "   ngspice exit code $? for $f" | tee -a out/summary.txt
  grep '^RESULT' "out/$f.log" >> out/summary.txt
done
echo "done: out/summary.txt"
