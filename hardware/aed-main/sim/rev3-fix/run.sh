#!/usr/bin/env bash
# NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# J-2 / J-7 study (see notes.md). ngspice batch mode, about 5-10 minutes on 7 parallel jobs.
#   held_margin_fix.cir   J-7: "held" sensor thresholds at true temperature against the largest cut-off
#   hold_fix.cir          template; this script writes the case lists below and run_cases.sh runs them:
#     out/drawn.txt        sheet 11 + sheet 09 AS DRAWN, true temperature, settled measurement window
#     out/recommended.txt  the recommended change (notes.md), full corner matrix
#     out/alternatives.txt the other changes tried, on the cases that decided against them
# Every FIX line states the case temperature and a BAT54 probe voltage that proves it reached the models; a run
# the simulator aborted is printed as "RUN ABORTED ... void", never as a result.
set -u
cd "$(dirname "$0")"
mkdir -p out
command -v ngspice >/dev/null || { echo "ngspice not found" >&2; exit 1; }
jobs=${1:-7}

ngspice -b held_margin_fix.cir > out/held_margin_fix.log 2>&1
grep '^J7' out/held_margin_fix.log > out/held_margin.txt

AGED_DRAWN="kcap=0.7 ctrp=2.5 isl=1.57e-14 bz=4.52"     # capacitors at 70 %, CTR 250 %, max MMSZ4687, high LED VF
NEW_DRAWN="ctrp=100 isl=1.41e-13 bz=4.09"               # new sensors: CTR 10,000 %, min MMSZ4687, low LED VF
NOM_DRAWN="ctrp=5 isl=4.7e-14 bz=4.3"
AGED_REC="kcap=0.7 ctrp=2.5 isl=1.57e-14 bz=4.94 tbv=1.79e-4"   # MMSZ4688 4.47-4.94 V, +0.8 mV/K
NEW_REC="ctrp=100 isl=1.41e-13 bz=4.47 tbv=1.79e-4"
NOM_REC="ctrp=5 isl=4.7e-14 bz=4.7 tbv=1.79e-4"
HOT="tc=70 vc=-2.22 idk=20u idkovp=20u"
NOMT="tc=25 vc=-2.0"

# matrix <prefix> <low-amplitude V> <its source ohms> <high V> <its ohms> <fix params> <aged> <new> <nom> <cliff Vs>
matrix() {
  local k=$1 vl=$2 rl=$3 vhh=$4 rhh=$5 x=$6 ag=$7 nw=$8 nm=$9 cl=${10}
  for f in 1.35k 1.5k 1.65k; do for m in 0 1 2 3 4; do
    echo "${k}_cold_aged_${f}_pilot2.0_m$m|vh=$vl rclk=$rl fclk=$f vp=2.0 mode=$m $ag $x"; done; done
  for f in 1.35k 1.65k; do for m in 1 2 3 4; do
    echo "${k}_cold_aged_${f}_pilot0.5_m$m|vh=$vl rclk=$rl fclk=$f vp=0.5 mode=$m $ag $x"; done; done
  for m in 1 2 3 4; do echo "${k}_cold_fresh_1.35k_pilot0.5_m$m|vh=$vhh rclk=$rhh kcap=1.0 fclk=1.35k vp=0.5 mode=$m $nw $x"; done
  for f in 1.35k 1.65k; do echo "${k}_hot_aged_${f}_pilot2.0_m0|vh=$vl rclk=$rl fclk=$f vp=2.0 mode=0 $HOT $ag $x"; done
  for m in 1 2 3 4; do echo "${k}_hot_fresh_1.35k_pilot0.5_m$m|vh=$vhh rclk=$rhh kcap=1.0 fclk=1.35k vp=0.5 mode=$m $HOT $nw $x"; done
  for m in 1 2 3 4; do echo "${k}_hot_fresh_1.65k_pilot2.0_m$m|vh=$vhh rclk=$rhh kcap=1.0 fclk=1.65k vp=2.0 mode=$m $HOT $nw $x"; done
  for m in 0 1 2 3; do echo "${k}_nom25_1.5k_pilot1.2_m$m|vh=$vl rclk=$rl kcap=1.0 fclk=1.5k vp=1.2 mode=$m $NOMT $nm $x"; done
  for v in $cl; do for f in 1.35k 1.65k; do echo "${k}_cliff_${v}V_${f}_pilot2.0_m0|vh=$v rclk=$rl fclk=$f vp=2.0 mode=0 $ag $x"; done; done
  echo "${k}_cold_aged_2.0k_60ohm_pilot2.0_m0|vh=$vl rclk=60 fclk=2k vp=2.0 mode=0 $ag $x"
  echo "${k}_cold_aged_1.35k_duty40_pilot2.0_m0|vh=$vl rclk=$rl fclk=1.35k duty=0.4 vp=2.0 mode=0 $ag $x"
  echo "${k}_cold_aged_zener2_SHORT_m0|vh=$vl rclk=$rl fclk=1.5k vp=2.0 mode=0 z2=1 $ag $x"
  echo "${k}_cold_aged_zener2_OPEN_m0|vh=$vl rclk=$rl fclk=1.5k vp=2.0 mode=0 z2=2 $ag $x"
  echo "${k}_hot_new_zener2_SHORT_m0|vh=$vhh rclk=$rhh kcap=1.0 fclk=1.5k vp=0.5 mode=0 z2=1 $HOT $nw $x"
  echo "${k}_hot_freshpump_agedopto_OVPforce_m5|vh=$vhh rclk=$rhh kcap=1.0 fclk=1.65k vp=2.0 mode=5 $HOT ctrp=2.5 isl=1.57e-14 $x"
  echo "${k}_cold_freshpump_agedopto_OVPforce_m5|vh=$vhh rclk=$rhh kcap=1.0 fclk=1.65k vp=2.0 mode=5 ctrp=2.5 isl=1.57e-14 $x"
}
# AS DRAWN: 3V3_MAIN buffer; cold corner 3.1 V behind 40 ohm (WP-F), fresh corner 3.5 V behind 30 ohm
matrix D 3.1 40 3.5 30 "" "$AGED_DRAWN" "$NEW_DRAWN" "$NOM_DRAWN" "2.9 3.0" > out/drawn.list
# RECOMMENDED: 5 V drive (5V0_SYS through a translator; cold corner 4.75 V behind 40 ohm, fresh 5.25 V behind
# 22 ohm), R1131/R1132 1 k, R1133/R1134 22 ohm, 1 M bleeds on RLSx_P1..P3, MMSZ4688 sensor zeners
REC="rpd=1k rpb=1meg r1133=22"
matrix R 4.75 40 5.25 22 "$REC" "$AGED_REC" "$NEW_REC" "$NOM_REC" "3.5 4.0 4.25" > out/recommended.list
# ALTERNATIVES, each on the cases that decided against it
{
  # A1: 5 V drive alone (nothing else changed)
  for m in 1 2 3 4; do echo "A1_5V_only_hot_fresh_1.35k_pilot0.5_m$m|vh=5.25 rclk=22 kcap=1.0 fclk=1.35k vp=0.5 mode=$m $HOT $NEW_DRAWN"; done
  # A2: 5 V drive + R1131 1 k + 2.2 M bleeds (slower release)
  for m in 1 2 3 4; do echo "A2_5V_1k_2M2bleed_hot_fresh_1.35k_pilot0.5_m$m|vh=5.25 rclk=22 kcap=1.0 fclk=1.35k vp=0.5 mode=$m rpd=1k rpb=2.2meg $HOT $NEW_DRAWN"; done
  # A3: 5 V drive + R1131 1 k + R1135/R1190/R1191 1 M (pilot loads instead of node bleeds): low-amplitude lock-out
  for v in 3.5 4.0; do echo "A3_5V_1k_pilotloads1M_cliff_${v}V_1.65k_m0|vh=$v rclk=40 fclk=1.65k vp=2.0 mode=0 rpd=1k rjb=1meg rh=1meg $AGED_DRAWN"; done
  # A4: as recommended but R1133 raised further to 47 ohm: lock-out moves above 4.25 V
  for v in 4.25 4.75; do echo "A4_5V_R1133_47_cliff_${v}V_1.35k_m0|vh=$v rclk=40 fclk=1.35k vp=2.0 mode=0 $REC r1133=47 $AGED_DRAWN"; done
  # A7: as recommended but R1133 left at 10 ohm: works, but edge currents reach ~50 mA at 5.25 V
  for m in 1 2; do echo "A7_5V_R1133_10_hot_fresh_1.35k_pilot0.5_m$m|vh=5.25 rclk=22 kcap=1.0 fclk=1.35k vp=0.5 mode=$m rpd=1k rpb=1meg $HOT $NEW_REC"; done
  echo "A7_5V_R1133_10_cliff_4.0V_1.35k_m0|vh=4.0 rclk=40 fclk=1.35k vp=2.0 mode=0 rpd=1k rpb=1meg $AGED_REC"
  # A5: sheet 11 only - Schottky (BAT54S) pilot pump + 1 M bleeds + R1131 1 k: the 1 ms clock gap releases when hot
  echo "A5_sheet11only_BAT54S_1M_hot_aged_1.35k_pilot2.0_m0|vh=3.1 rclk=40 fclk=1.35k vp=2.0 mode=0 pds=1 rlkp=1meg rpb=1meg rpd=1k $HOT $AGED_DRAWN"
  for m in 1 2 3; do echo "A5_sheet11only_BAT54S_1M_hot_fresh_1.35k_pilot0.5_m$m|vh=3.5 rclk=30 kcap=1.0 fclk=1.35k vp=0.5 mode=$m pds=1 rlkp=1meg rpb=1meg rpd=1k $HOT $NEW_DRAWN"; done
  # A6: sheet 11 only - as A5 with 2.2 M bleeds: survives the gap, but stopped-high release > 25 ms when cold
  echo "A6_sheet11only_BAT54S_2M2_hot_aged_1.35k_pilot2.0_m0|vh=3.1 rclk=40 fclk=1.35k vp=2.0 mode=0 pds=1 rlkp=1meg rpb=2.2meg rpd=1k $HOT $AGED_DRAWN"
  for m in 1 2 3; do echo "A6_sheet11only_BAT54S_2M2_cold_fresh_1.35k_pilot0.5_m$m|vh=3.5 rclk=30 kcap=1.0 fclk=1.35k vp=0.5 mode=$m pds=1 rpb=2.2meg rpd=1k $NEW_DRAWN"; done
  for v in 2.7 2.8; do echo "A6_sheet11only_BAT54S_2M2_cliff_${v}V_1.65k_m0|vh=$v rclk=40 fclk=1.65k vp=2.0 mode=0 pds=1 rpb=2.2meg rpd=1k $AGED_DRAWN"; done
} > out/alternatives.list

for n in drawn recommended alternatives; do
  echo "== $n"
  bash run_cases.sh out/$n.list out/$n.txt "$jobs"
  echo "   $(grep -c '^FIX' out/$n.txt) results, $(grep -c -E 'ABORTED|FAILED|NOT APPLIED' out/$n.txt) void"
done
echo "done: out/held_margin.txt, out/drawn.txt, out/recommended.txt, out/alternatives.txt"
