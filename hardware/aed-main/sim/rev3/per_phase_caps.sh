#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# per_phase_caps.sh — revision 0.3 per-phase hardware time caps (sheet 12), worst-case delivered energy when the
# firmware fails and only a hardware cap ends a phase (I-13, Q-53). Re-uses ../bridge/q3_shock_energy.cir
# unchanged (copied per case into out/cases/ with .param values changed, as ../bridge/run.sh does).
# Caps used = the TOP of each cap's band (calculated in docs/HARDWARE_DESIGN.md §B13.3, revision 0.3):
#   phase 1: R 113 k, C 100 nF C0G, threshold 0.6345 -> 11.37 ms nominal, band 10.75-11.85 ms
#   phase 2: R 76.8 k, same C and threshold -> 7.72 ms nominal, band 7.30-8.05 ms
# P1 frozen = phase-1 command stuck, phase 1 runs to the phase-1 cap, no phase 2.
# P2 frozen = phase 1 as planned (DESIGN_TABLE.md), phase-2 command stuck, phase 2 runs to the phase-2 cap.
# Voltages: 1,950 V (adult plan) and 2,009 V (top of the firmware's at-plan window, HARDWARE_DESIGN §B4.4).
# Usage: bash per_phase_caps.sh   (about 25 cases, a few minutes with JOBS=6)
set -euo pipefail
cd "$(dirname "$0")"
B=$(cd ../bridge && pwd)
OUT=out; CASES=$OUT/cases; JOBS=${JOBS:-6}
mkdir -p "$CASES"; rm -f "$CASES"/*.cir; : > "$OUT/per_phase_caps.txt"
P1=11.85e-3; P2=8.05e-3
declare -A T1=( [25]=3e-3 [50]=3.339e-3 [75]=4.422e-3 [100]=5.552e-3 [125]=6.699e-3 [175]=9.011e-3 )
mk() {
  local label=$1 params=$2 f="$CASES/q3__${1}.cir" kv k v
  cp "$B/q3_shock_energy.cir" "$f"
  for kv in $params; do k=${kv%%=*}; v=${kv#*=}; sed -i -E "/^\.param/ s/ $k=[^ ]+/ $k=$v/" "$f"; done
  sed -i "s|CASE_LABEL|$label $params|; s|^\.include bridge_common.inc|.include $B/bridge_common.inc|" "$f"
  echo "$f"
}
list=()
for v0 in 1950 2009; do for r in 25 50 75 100 125 175; do
  list+=("$(mk "rev3_P1frozen_p1cap${P1}_V${v0}_R${r}" "v0=$v0 rp=$r t1=$P1 t2=1e-3 p2en=0")")
  list+=("$(mk "rev3_P2frozen_p2cap${P2}_V${v0}_R${r}" "v0=$v0 rp=$r t1=${T1[$r]} t2=$P2 p2en=1")")
done; done
# Each case is run from ../bridge (model paths are relative to it). A case that stops early is retried with
# other integration settings exactly as ../bridge/run.sh does; the RESULT line then names the settings.
cat > "$OUT/runcase.sh" <<'RC'
#!/usr/bin/env bash
f=$1
complete() { grep -h '^RESULT' "$1" 2>/dev/null | head -1 | awk '{n=split($0,a,"t_end="); if (n<2) exit 1; split(a[n],b," of "); exit !((b[1]+0) >= 0.999*(b[2]+0))}'; }
ngspice -b "$f" > "$f.log" 2>&1 || true
complete "$f.log" && exit 0
i=0
for opt in "method=gear" "method=gear maxord=2 trtol=1" "method=trap reltol=2e-3" "method=gear reltol=2e-3 trtol=1"; do
  i=$((i+1)); g="${f%.cir}.alt$i.cir"
  sed "s/^\.options .*/.options $opt abstol=1e-6 vntol=1e-4 itl4=100/; s/^echo \"RESULT \(.*\)\"$/echo \"RESULT \1 [solver: $opt]\"/" "$f" > "$g"
  ngspice -b "$g" > "$g.log" 2>&1 || true
  if complete "$g.log"; then cp "$g.log" "$f.log"; exit 0; fi
done
exit 0
RC
chmod +x "$OUT/runcase.sh"
R=$(pwd)
printf '%s\n' "${list[@]}" | sed "s|^|$R/|" | (cd "$B" && xargs -P "$JOBS" -n 1 "$R/$OUT/runcase.sh")
for f in "${list[@]}"; do
  line=$(grep -h '^RESULT' "$f.log" 2>/dev/null | head -1 || true)
  if [ -z "$line" ]; then echo "FAILED $(basename "$f")"; else
    echo "$line" | awk '{ s=$0; n=split(s,a,"t_end="); split(a[n],b," of "); if ((b[1]+0) < 0.999*(b[2]+0)) print "INCOMPLETE " s; else print s }'; fi
done > "$OUT/per_phase_caps.txt"
sort "$OUT/per_phase_caps.txt" -o "$OUT/per_phase_caps.txt"
cat "$OUT/per_phase_caps.txt"
