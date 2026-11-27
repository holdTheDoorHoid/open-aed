#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# Runs every bridge simulation (questions 1-4) and writes the measurements to out/<netlist>.txt.
# Each netlist runs on its own with its default .param values (ngspice -b <file>.cir). This script makes one
# copy per case under out/cases/ with the .param values changed, runs the copies in parallel (JOBS, default 4)
# and collects the RESULT lines. A case whose simulation stopped early is marked INCOMPLETE; one that printed
# nothing is marked FAILED. Simulation only: nothing here describes building or powering anything.
set -euo pipefail
cd "$(dirname "$0")"
OUT=out
CASES=$OUT/cases
JOBS=${JOBS:-4}
mkdir -p "$CASES"
# (case files are removed per question just before that question is run; see "run")
: > "$OUT/joblist"

# mk <netlist.cir> <label> "<param=value ...>" [tstop tmax]
mk() {
  local src=$1 label=$2 params=$3 tstop=${4:-} tmax=${5:-}
  local f="$CASES/${src%.cir}__${label}.cir"
  rm -f "$f"; cp "$src" "$f"
  local kv k v
  for kv in $params; do
    k=${kv%%=*}; v=${kv#*=}
    grep -Eq "^\.param.* $k=" "$f" || { echo "run.sh: no .param $k in $src" >&2; exit 1; }
    sed -i -E "/^\.param/ s/ $k=[^ ]+/ $k=$v/" "$f"
  done
  sed -i "s|CASE_LABEL|$label $params|; s|^\.include bridge_common.inc|.include $PWD/bridge_common.inc|" "$f"
  if [ -n "$tstop" ]; then
    sed -i -E "s/^tran ([^ ]+) [^ ]+ ([^ ]+) [^ ]+$/tran \1 $tstop \2 $tmax/; s/^let tstop_req = .*/let tstop_req = $tstop/" "$f"
  fi
  echo "$f" >> "$OUT/joblist"
}

########## Question 1: one low-side switch shorted (I-8, F-1, CI-10) ##########
Q=q1_shorted_lowside.cir
SH=0.05
for v0 in 1950 2200; do for r in 25 175; do for tdl in 0.1u 0.5u 1u 2u; do for th in 20.2u 36.4u; do
  mk $Q "asdrawn_D_V${v0}_R${r}_tdl${tdl}_thoff${th}" "v0=$v0 rp=$r sla=1e9 slb=$SH tdl=$tdl thoff=$th"
done; done; done; done
for v0 in 1950 2200; do for tdl in 0.1u 0.5u 1u 2u; do
  mk $Q "asdrawn_S_V${v0}_tdl${tdl}" "v0=$v0 rp=25 sla=$SH slb=1e9 tdl=$tdl thoff=20.2u"
done; done
for lst in 20n 100n 200n; do
  mk $Q "asdrawn_D_lst${lst}" "v0=2200 rp=25 sla=1e9 slb=$SH tdl=1u thoff=20.2u lst=$lst"
  mk $Q "asdrawn_S_lst${lst}" "v0=2200 rp=25 sla=$SH slb=1e9 tdl=1u thoff=20.2u lst=$lst"
done
mk $Q "asdrawn_D_th0.06" "v0=2200 rp=25 sla=1e9 slb=$SH tdl=1u thoff=20.2u th=0.06"
mk $Q "asdrawn_S_th0.06" "v0=2200 rp=25 sla=$SH slb=1e9 tdl=1u thoff=20.2u th=0.06"
for sc in D S; do
  if [ $sc = D ]; then a=1e9; b=$SH; else a=$SH; b=1e9; fi
  for v0 in 1950 2200; do
    for r in 25 175; do for tdl in 0.1u 0.5u 1u 2u; do
      [ $sc = S ] && [ $r = 175 ] && continue
      mk $Q "fix1_Hoff_${sc}_V${v0}_R${r}_tdl${tdl}" "v0=$v0 rp=$r sla=$a slb=$b tdl=$tdl thoff=50n"
    done; done
    for thr in 2 3 5; do
      mk $Q "fix12_fastcomp_${sc}_V${v0}_thr${thr}" "v0=$v0 rp=25 sla=$a slb=$b tdl=0.1u thoff=50n ifilt=1n ithr=$thr"
      mk $Q "fix12_fastcomp_th0.06_${sc}_V${v0}_thr${thr}" "v0=$v0 rp=25 sla=$a slb=$b tdl=0.1u thoff=50n ifilt=1n ithr=$thr th=0.06"
    done
    mk $Q "fix12_cg47n_${sc}_V${v0}_thr3" "v0=$v0 rp=25 sla=$a slb=$b tdl=0.1u thoff=50n ifilt=1n ithr=3 cg=47n"
    for tdl in 0.1u 1u; do mk $Q "fix13_rdis100_${sc}_V${v0}_tdl${tdl}" "v0=$v0 rp=25 sla=$a slb=$b tdl=$tdl thoff=50n rdis=100"; done
    for thr in 3 5; do
      mk $Q "fix123_${sc}_V${v0}_thr${thr}" "v0=$v0 rp=25 sla=$a slb=$b tdl=0.1u thoff=50n ifilt=1n ithr=$thr rdis=100"
      mk $Q "fix123_th0.06_${sc}_V${v0}_thr${thr}" "v0=$v0 rp=25 sla=$a slb=$b tdl=0.1u thoff=50n ifilt=1n ithr=$thr rdis=100 th=0.06"
    done
    mk $Q "fix123_R175_${sc}_V${v0}_thr3" "v0=$v0 rp=175 sla=$a slb=$b tdl=0.1u thoff=50n ifilt=1n ithr=3 rdis=100"

  done
done

########## Question 2: turn-off at the over-current trip (F-2) ##########
Q=q2_oc_turnoff.cir
N="cloc=1p ccl=1p rcl=10k"
for th in 82 89.7 98; do for lp in 0 1.5u; do for td in 65n 100n; do
  mk $Q "asdrawn_trip${th}_lpat${lp}_tdrv${td}" "v0=2200 lpat=$lp octhr=$th tdrv=$td lst=50n lcap=200n csn=3n rsn=10 $N"
done; done; done
for lst in 20n 50n 100n 200n; do for lp in 0 1.5u; do
  mk $Q "asdrawn_lst${lst}_lpat${lp}" "v0=2200 lpat=$lp octhr=98 lst=$lst lcap=200n csn=3n rsn=10 $N"
done; done
for lc in 50n 500n 1u; do mk $Q "asdrawn_lcap${lc}" "v0=2200 lpat=0 octhr=98 lst=100n lcap=$lc csn=3n rsn=10 $N"; done
for cs in 10n 47n; do for rs in 10 3.3 1; do
  mk $Q "snubberRC_${cs}_${rs}ohm" "v0=2200 lpat=0 octhr=98 lst=200n lcap=500n csn=$cs rsn=$rs $N"
done; done
for cl in 100n 470n; do mk $Q "localfilm_${cl}" "v0=2200 lpat=0 octhr=98 lst=200n lcap=500n csn=3n rsn=10 cloc=$cl ccl=1p rcl=10k"; done
for cc in 100n 470n; do mk $Q "RCDclamp_${cc}" "v0=2200 lpat=0 octhr=98 lst=200n lcap=500n csn=3n rsn=10 cloc=1p ccl=$cc rcl=10k"; done
mk $Q "localfilm470n_RCD470n_lcap1u" "v0=2200 lpat=0 octhr=98 lst=200n lcap=1u csn=3n rsn=10 cloc=470n ccl=470n rcl=10k"
mk $Q "localfilm470n_RCD470n_lst200n_lcap500n" "v0=2200 lpat=0 octhr=98 lst=200n lcap=500n csn=3n rsn=10 cloc=470n ccl=470n rcl=10k"
for th in 74 80 86; do mk $Q "trip80_${th}" "v0=2200 lpat=0 octhr=$th lst=50n lcap=200n csn=3n rsn=10 $N"; done
mk $Q "trip80_86_fixes" "v0=2200 lpat=0 octhr=86 lst=200n lcap=500n csn=3n rsn=10 cloc=470n ccl=470n rcl=10k"

########## Question 3: shock energy, normal and with only the time cap (I-13) ##########
Q=q3_shock_energy.cir
# adult / child timings from DESIGN_TABLE.md
while read -r mode v0 r t1 t2; do
  mk $Q "normal_${mode}_R${r}" "v0=$v0 rp=$r t1=$t1 t2=$t2 p2en=1"
done <<'CASES'
adult 1950 25 3e-3 2e-3
adult 1950 50 3.339e-3 2.226e-3
adult 1950 75 4.422e-3 2.948e-3
adult 1950 125 6.699e-3 4.466e-3
adult 1950 175 9.011e-3 6.007e-3
child 1150 25 3e-3 2e-3
child 1150 50 3e-3 2e-3
child 1150 75 3.943e-3 2.629e-3
child 1150 125 6.020e-3 4.013e-3
child 1150 175 8.121e-3 5.414e-3
CASES
declare -A T1=( [25]=3e-3 [50]=3.339e-3 [75]=4.422e-3 [125]=6.699e-3 [175]=9.011e-3 )
for v0 in 1950 2000; do for r in 25 50 75 125 175; do
  for cap in 21.3e-3 25e-3 28.8e-3; do
    mk $Q "asdrawn_P1frozen_cap${cap}_V${v0}_R${r}" "v0=$v0 rp=$r t1=$cap t2=1e-3 p2en=0"
  done
  t2=$(awk -v a="${T1[$r]}" 'BEGIN{printf "%.5g", 28.8e-3-a-0.5e-3}')
  mk $Q "asdrawn_P2frozen_cap28.8e-3_V${v0}_R${r}" "v0=$v0 rp=$r t1=${T1[$r]} t2=$t2 p2en=1"
  for opt in "A 16.2e-3 10.8e-3" "B 13.3e-3 8.8e-3" "C 11.1e-3 7.4e-3"; do
    set -- $opt
    mk $Q "option$1_P1frozen_p1cap$2_V${v0}_R${r}" "v0=$v0 rp=$r t1=$2 t2=1e-3 p2en=0"
    mk $Q "option$1_P2frozen_p2cap$3_V${v0}_R${r}" "v0=$v0 rp=$r t1=${T1[$r]} t2=$3 p2en=1"
  done
done; done

########## Question 4: supply collapse during a pulse ##########
Q=q4_supply_collapse.cir
for r in 25 175; do
  t1=3e-3; [ $r = 175 ] && t1=9.011e-3
  for tf in 1u 100u; do mk $Q "B5V_collapse_tfall${tf}_R${r}" "mode=1 rp=$r t1=$t1 tfall=$tf" 1.42m 50n; done
  mk $Q "B5V_collapse_tfall2m_R${r}" "mode=1 rp=$r t1=$t1 tfall=2m" 4m 200n
  mk $Q "logic_3V3_lost_R${r}" "mode=3 rp=$r t1=$t1" 1.42m 50n
  mk $Q "MGJ2_alone_100us_R${r}" "mode=2 rp=$r t1=$t1 tlsf=100u" 1.6m 100n
  mk $Q "MGJ2_alone_1ms_R${r}" "mode=2 rp=$r t1=$t1 tlsf=1m" 3.3m 200n
done
mk $Q "MGJ2_alone_10ms_R175" "mode=2 rp=175 t1=9.011e-3 tlsf=10m" 9.4m 500n

########## run ##########
# ONLY=<regex> restricts the run to matching cases (for checking a subset)
if [ -n "${ONLY:-}" ]; then grep -E "$ONLY" "$OUT/joblist" > "$OUT/joblist.tmp" || true; mv "$OUT/joblist.tmp" "$OUT/joblist"; fi
# remove old outputs only for the cases about to run
while read -r f; do rm -f "${f%.cir}".alt*.cir "${f%.cir}".alt*.cir.log "$f.log"; done < "$OUT/joblist"

# A case that stops early (the solver's "timestep too small") is retried with other integration settings;
# the RESULT line then says which settings gave the answer.
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
echo "run.sh: $(wc -l < "$OUT/joblist") cases, $JOBS at a time"
xargs -P "$JOBS" -n 1 "$OUT/runcase.sh" < "$OUT/joblist"
########## collect ##########
for q in q1_shorted_lowside q2_oc_turnoff q3_shock_energy q4_supply_collapse; do
  grep -q "/${q}__" "$OUT/joblist" || continue   # not run this time: keep its previous results
  {
    echo "# $q - SIMULATED (ngspice $(ngspice -v 2>/dev/null | grep -o 'ngspice-[0-9]*' | head -1)), $(date -u +%Y-%m-%d). Models: see models/ and notes.md."
    { grep "/${q}__" "$OUT/joblist" || true; } | while read -r f; do
      line=$(grep -h '^RESULT' "$f.log" 2>/dev/null | head -1 || true)
      if [ -z "$line" ]; then
        echo "FAILED $(basename "$f" .cir) (no result; see $f.log)"
      else
        echo "$line" | awk '{ s=$0; n=split(s,a,"t_end="); split(a[n],b," of "); if ((b[1]+0) < 0.999*(b[2]+0)) print "INCOMPLETE " s; else print s }'
      fi
    done
  } > "$OUT/$q.txt"
  echo "wrote $OUT/$q.txt ($(grep -c "^RESULT" "$OUT/$q.txt" || true) ok, $(grep -c -E "^(INCOMPLETE|FAILED)" "$OUT/$q.txt" || true) not ok)"
done
