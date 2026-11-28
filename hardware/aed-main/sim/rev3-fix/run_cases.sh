#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# Runs one list of hold_fix.cir cases. Usage: bash run_cases.sh <case-list> <output-file> [parallel jobs]
# A case list has one case per line: "<label>|<.param assignments>" (lines starting with # are skipped).
# Each case is written to out/cases/<label>.cir (CASE_PARAMS -> ".param <assignments>", CASE_LABEL -> label),
# run with ngspice in batch mode, and its FIX result line collected into <output-file> in list order.
set -u
cd "$(dirname "$0")"
list=$1; outf=$2; jobs=${3:-4}
mkdir -p out/cases
here=$PWD
run_one() {
  local label=$1 params=$2
  local f="out/cases/$label.cir"
  # the case temperature (tc=..., default -20 C as in the template) is also written into the control script
  local t; t=$(echo "$params" | grep -o "tc=[-0-9.]*" | cut -d= -f2); [ -n "$t" ] || t=-20
  sed -e "s|^CASE_PARAMS\$|.param $params|" -e "s|CASE_LABEL|$label|g" -e "s|CASE_TEMP|$t|" \
      -e "s|^\.include \.\./charger-storage/|.include $here/../charger-storage/|" hold_fix.cir > "$f"
  # pds=1 in a case: the pilot-pump diodes D1117-D1119 become BAT54 (BAT54S) instead of BAV199
  case " $params " in *" gear=1 "*) sed -i 's/^\* GEAR_OPTION.*/option method=gear/' "$f" ;; esac
  case " $params " in *" pds=1 "*) sed -i -E 's/^(D111[789][ab] .*) BAV199$/\1 BAT54/' "$f" ;; esac
  ngspice -b "$f" > "$f.log" 2>&1 || echo "ngspice exit code $? for $label" >> "$f.log"
}
n=0
while IFS='|' read -r label params; do
  case "$label" in ''|'#'*) continue ;; esac
  run_one "$label" "$params" &
  n=$((n+1))
  if [ $((n % jobs)) -eq 0 ]; then wait; fi
done < "$list"
wait
: > "$outf"
while IFS='|' read -r label params; do
  case "$label" in ''|'#'*) continue ;; esac
  line=$(grep -h '^FIX ' "out/cases/$label.cir.log" 2>/dev/null | head -1)
  if [ -z "$line" ]; then echo "FAILED $label (no result; see out/cases/$label.cir.log)" >> "$outf"
  else echo "$line  [params: $params]" >> "$outf"; fi
done < "$list"
