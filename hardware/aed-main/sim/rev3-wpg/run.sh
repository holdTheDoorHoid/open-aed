#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# WP-G revision 0.3 check of sheet 13_hv_bridge as drawn (S-1 .. S-4, see notes.md). Same scheme as
# ../bridge/run.sh: one copy of a netlist per case under out/cases/ with .param values changed, run in parallel
# (JOBS, default 6), a case that stops early is retried with other solver settings, one RESULT line per case in
# out/<netlist>.txt (INCOMPLETE / FAILED mark runs that did not finish). ONLY=<regex> runs a subset;
# COLLECT_ONLY=1 rewrites out/<netlist>.txt from the logs already in out/cases without running anything.
# Simulation only: nothing here describes building or powering anything.
set -euo pipefail
cd "$(dirname "$0")"
OUT=out
CASES=$OUT/cases
JOBS=${JOBS:-6}
mkdir -p "$CASES"
: > "$OUT/joblist"

# mk <netlist.cir> <label> "<param=value ...>" [beh]
mk() {
  local src=$1 label=$2 params=$3 beh=${4:-}
  local f="$CASES/${src%.cir}__${label}.cir"
  echo "$f" >> "$OUT/joblist"
  [ -n "${COLLECT_ONLY:-}" ] && return 0
  rm -f "$f"; cp "$src" "$f"
  local kv k v
  for kv in $params; do
    k=${kv%%=*}; v=${kv#*=}
    grep -Eq "^\.param.* $k=" "$f" || { echo "run.sh: no .param $k in $src" >&2; exit 1; }
    sed -i -E "/^\.param/ s/ $k=[^ ]+/ $k=$v/" "$f"
    if [ "$k" = ith ]; then sed -i -E "s/^let ith_v = .*/let ith_v = $v/" "$f"; fi
  done
  sed -i "s|CASE_LABEL|$label $params|; s|^\.include wpg_common.inc|.include $PWD/wpg_common.inc|; s|WAVEFILE|$PWD/$f.wave|" "$f"
  if [ -n "$beh" ]; then
    # behavioural high-side drive of ../bridge (hsdrv) in place of the circuit-level island
    sed -i -E 's/^(XDH[AB] .* hsisland .*)$/* (replaced) \1/; s/^\*BEH //' "$f"
    sed -i 's|let pnpi = v(pnpa)-v(oa)|let pnpi = (v(gha)-v(xdha.rect)-0.65)/34|' "$f"
  fi
}

########## S-1: one low-side switch shorted, circuit-level island, real fast path ##########
Q=s1_shorted_lowside.cir
SH=0.05
for v0 in 1950 2200; do
  for ith in 2.54 3.0 3.5 3.87; do
    for r in 25 50 100 175; do mk $Q "D_V${v0}_R${r}_ith${ith}" "v0=$v0 rp=$r sla=1e9 slb=$SH ith=$ith"; done
    mk $Q "S_V${v0}_ith${ith}" "v0=$v0 rp=25 sla=$SH slb=1e9 ith=$ith"
  done
  # failures of the new protection (latent faults) on top of the shorted low side
  mk $Q "D_V${v0}_R25_fastpathDEAD" "v0=$v0 rp=25 sla=1e9 slb=$SH fast=0"
  mk $Q "S_V${v0}_fastpathDEAD" "v0=$v0 rp=25 sla=$SH slb=1e9 fast=0"
  mk $Q "D_V${v0}_R25_detectorDEAD_ith3.87" "v0=$v0 rp=25 sla=1e9 slb=$SH ith=3.87 det=0"
  mk $Q "S_V${v0}_detectorDEAD_ith3.87" "v0=$v0 rp=25 sla=$SH slb=1e9 ith=3.87 det=0"
  mk $Q "D_V${v0}_R25_bothDEAD" "v0=$v0 rp=25 sla=1e9 slb=$SH fast=0 det=0"
  mk $Q "S_V${v0}_bothDEAD" "v0=$v0 rp=25 sla=$SH slb=1e9 fast=0 det=0"
done
# sensitivities at the worst corner (2,200 V, threshold at the top of its band)
for vth in 1.0 2.5; do
  mk $Q "D_V2200_R25_ith3.87_vth${vth}" "v0=2200 rp=25 sla=1e9 slb=$SH ith=3.87 vth=$vth"
  mk $Q "S_V2200_ith3.87_vth${vth}" "v0=2200 rp=25 sla=$SH slb=1e9 ith=3.87 vth=$vth"
done
mk $Q "D_V2200_R25_ith3.87_tlog100n" "v0=2200 rp=25 sla=1e9 slb=$SH ith=3.87 tlog=100n"
mk $Q "S_V2200_ith3.87_tlog100n" "v0=2200 rp=25 sla=$SH slb=1e9 ith=3.87 tlog=100n"
mk $Q "S_V2200_ith3.87_th0.06" "v0=2200 rp=25 sla=$SH slb=1e9 ith=3.87 th=0.06"
mk $Q "S_V2200_ith3.87_lsh0" "v0=2200 rp=25 sla=$SH slb=1e9 ith=3.87 lsh=1p"
mk $Q "S_V2200_ith3.87_lsh1n" "v0=2200 rp=25 sla=$SH slb=1e9 ith=3.87 lsh=1n"
mk $Q "S_V2200_ith3.87_lowcarrier" "v0=2200 rp=25 sla=$SH slb=1e9 ith=3.87 vamp=16.5"
# the 1 us vs 2.7 us disagreement: same circuit, detector absent, circuit-level island vs behavioural hsdrv
for c in "D_V1950_R25 v0=1950 rp=25 sla=1e9 slb=$SH" "S_V2200 v0=2200 rp=25 sla=$SH slb=1e9"; do
  set -- $c; lab=$1; shift
  mk $Q "PNPstart_${lab}_island_det0" "$* ith=3.0 det=0"
  mk $Q "PNPstart_${lab}_behavioural" "$* ith=3.0 det=0" beh
done

########## S-2: turn-off at the 80 A trip band, 75 nF local film, 0.5-1 uH cable ##########
Q=s2_oc_turnoff.cir
for th in 74 80.1 86; do for lc in 500n 750n 1u; do for lfl in 10n 60n; do
  mk $Q "trip${th}_lcap${lc}_lfl${lfl}" "octhr=$th lcap=$lc lfl=$lfl lpat=0"
done; done; done
mk $Q "trip86_lcap1u_lpat1.5u" "octhr=86 lcap=1u lfl=60n lpat=1.5u"
mk $Q "trip86_lcap1u_lst200n" "octhr=86 lcap=1u lfl=60n lpat=0 lst=200n"
mk $Q "trip86_lcap1u_tdrv65n" "octhr=86 lcap=1u lfl=60n lpat=0 tdrv=65n"
mk $Q "trip86_lcap1u_film_tol" "octhr=86 lcap=1u lfl=60n lpat=0 cf1=0.135u cf2=0.165u"
mk $Q "trip86_lcap1u_C1347shorted" "octhr=86 lcap=1u lfl=60n lpat=0 cfail=1"
for lc in 500n 1u; do mk $Q "trip86_lcap${lc}_filmOPEN" "octhr=86 lcap=$lc lfl=60n lpat=0 cfail=2"; done
mk $Q "trip86_lcap200n_filmOPEN" "octhr=86 lcap=200n lfl=60n lpat=0 cfail=2"

########## S-3: no false trip of the fast path in a normal sequence or from the charger ##########
Q=s3_false_trip.cir
for v0 in 1950 2200; do for r in 25 50 100 175; do for lsh in 0.3n 1n; do
  mk $Q "V${v0}_R${r}_lsh${lsh}" "v0=$v0 rp=$r ith=2.54 lsh=$lsh"
done; done; done
mk $Q "V1950_R25_charger" "v0=1950 rp=25 ith=2.54 chg=1"
mk $Q "V2200_R175_charger" "v0=2200 rp=175 ith=2.54 chg=1"
mk $Q "V1950_R25_VOLmax_vos+230u" "v0=1950 rp=25 ith=2.54 vol=0.2 vos=230u"
mk $Q "testmode_60V_200ohm" "v0=60 rp=200 ith=2.54"

########## S-4: +20 V low-side supply monitor (LSD_OK) ##########
Q=s4_lsd_monitor.cir
for r in 25 175; do
  t1=3m; [ $r = 175 ] && t1=9.011m
  for tf in 100u 1m; do for mon in 0 1; do
    mk $Q "MGJ2_collapse_${tf}_R${r}_mon${mon}" "mode=2 rp=$r t1=$t1 tlsf=$tf mon=$mon"
  done; done
done
mk $Q "MGJ2_collapse_1m_R25_trip17.35" "mode=2 rp=25 tlsf=1m vtrip=17.35"
mk $Q "MGJ2_collapse_1m_R25_tlsd10u" "mode=2 rp=25 tlsf=1m tlsd=10u"
for k in 0.75 0.6; do for mon in 0 1; do
  mk $Q "MGJ2_sag_to_${k}_R25_mon${mon}" "mode=4 rp=25 tlsf=100u kend=$k mon=$mon"
done; done
mk $Q "reference_no_event_R25" "mode=4 rp=25 tlsf=100u kend=1 mon=1"

########## run ##########
if [ -n "${ONLY:-}" ]; then grep -E "$ONLY" "$OUT/joblist" > "$OUT/joblist.tmp" || true; mv "$OUT/joblist.tmp" "$OUT/joblist"; fi
# COLLECT_ONLY=1 only re-collects the RESULT lines of cases already run
if [ -z "${COLLECT_ONLY:-}" ]; then
while read -r f; do rm -f "${f%.cir}".alt*.cir "${f%.cir}".alt*.cir.log "$f.log"; done < "$OUT/joblist"
cat > "$OUT/runcase.sh" <<'RC'
#!/usr/bin/env bash
f=$1
complete() { grep -h '^RESULT' "$1" 2>/dev/null | head -1 | awk '{n=split($0,a,"t_end="); if (n<2) exit 1; split(a[n],b," of "); exit !((b[1]+0) >= 0.999*(b[2]+0))}'; }
ngspice -b "$f" > "$f.log" 2>&1 || true
complete "$f.log" && exit 0
i=0
for opt in "method=gear reltol=2e-3 trtol=1 rshunt=1e10 gmin=1e-10" "method=gear" "method=trap reltol=2e-3" "method=gear maxord=2 trtol=1 rshunt=1e10" "method=trap reltol=1e-3 rshunt=1e10 gmin=1e-10"; do
  i=$((i+1)); g="${f%.cir}.alt$i.cir"
  sed "s/^\.options .*/.options $opt abstol=1e-6 vntol=1e-4 itl4=200/; s/^echo \"RESULT \(.*\)\"$/echo \"RESULT \1 [solver: $opt]\"/" "$f" > "$g"
  ngspice -b "$g" > "$g.log" 2>&1 || true
  if complete "$g.log"; then cp "$g.log" "$f.log"; exit 0; fi
done
exit 0
RC
chmod +x "$OUT/runcase.sh"
echo "run.sh: $(wc -l < "$OUT/joblist") cases, $JOBS at a time"
xargs -P "$JOBS" -n 1 "$OUT/runcase.sh" < "$OUT/joblist"
fi
########## S-3b: replay each S-3 shunt waveform into the fast path alone (tight tolerances) ##########
: > "$OUT/replaylist"
{ grep "/s3_false_trip__" "$OUT/joblist" || true; } | while read -r f; do
  [ -s "$f.wave" ] || continue
  g="${f%.cir}.replay.cir"; w="${f%.cir}.pwl"
  if [ -z "${COLLECT_ONLY:-}" ]; then
    awk 'BEGIN{print "Vk kp 0 PWL("; last=-1} NF>=2 && $1+0>last {printf "+ %s %s\n", $1, $2; last=$1+0} END{print "+ )"}' "$f.wave" > "$w"
    ith=$(grep -oE '^\.param v0=[^ ]+ rp=[^ ]+ ith=[^ ]+' "$f" | grep -oE 'ith=[^ ]+' | cut -d= -f2)
    lab=$(grep -oE '^echo "RESULT [^|]*' "$f" | sed 's/^echo "RESULT //; s/ *$//')
    sed -e "s|^\.include wpg_common.inc|.include $PWD/wpg_common.inc|; s|WAVEINC|$PWD/$w|; s|CASE_LABEL|$lab|" \
        -e "s/^\.param ith=[^ ]*/.param ith=$ith/; s/^let ith_v = .*/let ith_v = $ith/" \
        -e "s/TSTOP_S/1.0e-3/; s/TSTOP/1.0m/" s3b_replay.cir > "$g"
  fi
  echo "$g" >> "$OUT/replaylist"
done
if [ -z "${COLLECT_ONLY:-}" ] && [ -s "$OUT/replaylist" ]; then
  echo "run.sh: replaying $(wc -l < "$OUT/replaylist") S-3 waveforms"
  xargs -P "$JOBS" -I{} sh -c 'ngspice -b "{}" > "{}.log" 2>&1 || true' < "$OUT/replaylist"
fi
if [ -s "$OUT/replaylist" ]; then
  { echo "# s3b_replay - SIMULATED (ngspice), $(date -u +%Y-%m-%d). Fast path alone, driven by the S-3 shunt waveforms."
    while read -r g; do grep -h '^RESULT' "$g.log" 2>/dev/null | head -1 || echo "FAILED $(basename "$g")"; done < "$OUT/replaylist"
  } > "$OUT/s3b_replay.txt"
  echo "wrote $OUT/s3b_replay.txt"
fi
for q in s1_shorted_lowside s2_oc_turnoff s3_false_trip s4_lsd_monitor; do
  grep -q "/${q}__" "$OUT/joblist" || continue
  {
    echo "# $q - SIMULATED (ngspice $(ngspice -v 2>/dev/null | grep -o 'ngspice-[0-9]*' | head -1)), $(date -u +%Y-%m-%d). Models: wpg_common.inc, ../bridge/models, ../charger-storage/models."
    { grep "/${q}__" "$OUT/joblist" || true; } | while read -r f; do
      line=$(grep -h '^RESULT' "$f.log" 2>/dev/null | head -1 || true)
      if [ -z "$line" ]; then echo "FAILED $(basename "$f" .cir) (no result; see $f.log)"
      else echo "$line" | awk '{ s=$0; n=split(s,a,"t_end="); split(a[n],b," of "); if ((b[1]+0) < 0.999*(b[2]+0)) print "INCOMPLETE " s; else print s }'; fi
    done
  } > "$OUT/$q.txt"
  echo "wrote $OUT/$q.txt ($(grep -c "^RESULT" "$OUT/$q.txt" || true) ok, $(grep -c -E "^(INCOMPLETE|FAILED)" "$OUT/$q.txt" || true) not ok)"
done
