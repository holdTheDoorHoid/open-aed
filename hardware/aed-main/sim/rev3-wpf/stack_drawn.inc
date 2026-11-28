* NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
* SPDX-License-Identifier: CERN-OHL-S-2.0
*
* stack_drawn.inc — ONE discharge path of 11_hv_storage as drawn after design revision 0.3 (stack A references;
* stack B identical): dump resistors R1101-R1103 (1.5 k x rm), Q1101-Q1104 IXTT2N170D2, sharing string
* R1107-R1110 || R6115-R6118 (each element 2 x 10 M = 5.0 M, WP-F change W-1) with C1101-C1104 3.3 nF, upper gate feed R1115-R1117 100 k -> GP02-40 (D1127-D1129) || 22 M
* (R6101-R6103) -> gate, R1121-R1123 100 M gate-source, SMF15CA gate TVS D1101-D1104, R6107-R6110 10 M and
* C1137-C1140 3.3 nF across each device. The bottom gate gb is driven by the caller (pump / release model).
* Parameters: tf, leakt (corner, see wpf_models.lib); vo1..vo4 cut-off at 25 C; lk1..lk4 leakage fraction;
* ilk GP02 leakage; cs1..cs4 string-capacitor and cd1..cd4 per-device-capacitor multipliers (C0G +-5 %);
* sh1..sh4 = 1 shorts that device (0.1 ohm); rm dump-resistor multiplier.
* Observables (in the sub-circuit): vd1..vd4 drain-source voltages, pr = power in ONE dump resistor (R1101),
* p6107 = power in R6107 (the per-device 10 M across the bottom device), ps1 = power in R1107 (one 10 M of the pair), pb2 = power in
* R6101 (22 M), pg2 = power in R1115 (100 k), pgs2 = power in R1121 (100 M), it = path current.
.subckt STACKWPF hvp gb rtn params: tf=1 leakt=68meg ilk=5u vo1=-3 vo2=-3 vo3=-3 vo4=-3 lk1=0.3 lk2=0.3 lk3=0.3
+ lk4=0.3 cs1=1 cs2=1 cs3=1 cs4=1 cd1=1 cd2=1 cd3=1 cd4=1 sh1=0 sh2=0 sh3=0 sh4=0 rm=1
R1101 hvp d1 {1.5k*rm}
R1102 d1 d2 {1.5k*rm}
R1103 d2 top {1.5k*rm}
XQ1101 m1 gb rtn DEPMOS params: voff={vo1*tf} rleak={leakt/lk1} kp=0.6 rd=6.0 cgs=3.57n cgd0=485p cds0=760p
RSH1 m1 rtn {sh1 ? 0.1 : 1e15}
XD1101 gb rtn SMF15CA
R6107 m1 rtn 10meg
C1137 m1 rtn {3.3n*cd1}
R1115 t1 gf2 100k
XD1127 gf2 g2 GPL params: ilk={ilk}
R6101 gf2 g2 22meg
R1121 g2 m1 100meg
XD1102 g2 m1 SMF15CA
XQ1102 m2 g2 m1 DEPMOS params: voff={vo2*tf} rleak={leakt/lk2} kp=0.6 rd=6.0 cgs=3.57n cgd0=485p cds0=760p
RSH2 m2 m1 {sh2 ? 0.1 : 1e15}
R6108 m2 m1 10meg
C1138 m2 m1 {3.3n*cd2}
R1116 t2 gf3 100k
XD1128 gf3 g3 GPL params: ilk={ilk}
R6102 gf3 g3 22meg
R1122 g3 m2 100meg
XD1103 g3 m2 SMF15CA
XQ1103 m3 g3 m2 DEPMOS params: voff={vo3*tf} rleak={leakt/lk3} kp=0.6 rd=6.0 cgs=3.57n cgd0=485p cds0=760p
RSH3 m3 m2 {sh3 ? 0.1 : 1e15}
R6109 m3 m2 10meg
C1139 m3 m2 {3.3n*cd3}
R1117 t3 gf4 100k
XD1129 gf4 g4 GPL params: ilk={ilk}
R6103 gf4 g4 22meg
R1123 g4 m3 100meg
XD1104 g4 m3 SMF15CA
XQ1104 top g4 m3 DEPMOS params: voff={vo4*tf} rleak={leakt/lk4} kp=0.6 rd=6.0 cgs=3.57n cgd0=485p cds0=760p
RSH4 top m3 {sh4 ? 0.1 : 1e15}
R6110 top m3 10meg
C1140 top m3 {3.3n*cd4}
R1107 t1 rtn 10meg
R6115 t1 rtn 10meg
R1108 t2 t1 10meg
R6116 t2 t1 10meg
R1109 t3 t2 10meg
R6117 t3 t2 10meg
R1110 top t3 10meg
R6118 top t3 10meg
C1101 t1 rtn {3.3n*cs1}
C1102 t2 t1 {3.3n*cs2}
C1103 t3 t2 {3.3n*cs3}
C1104 top t3 {3.3n*cs4}
BV1 vd1 rtn V=v(m1,rtn)
BV2 vd2 rtn V=v(m2,m1)
BV3 vd3 rtn V=v(m3,m2)
BV4 vd4 rtn V=v(top,m3)
BVM vdm rtn V=max(max(v(m1,rtn),v(m2,m1)),max(v(m3,m2),v(top,m3)))
BIT it rtn V=v(hvp,top)/(4.5k*{rm})
BPR pr rtn V=(v(hvp,top)/3)^2/(1.5k*{rm})
BP6 p6107 rtn V=v(m1,rtn)^2/10meg
BPS ps1 rtn V=v(t1,rtn)^2/10meg
BPB pb2 rtn V=v(gf2,g2)^2/22meg
BPG pg2 rtn V=v(t1,gf2)^2/100k
BPGS pgs2 rtn V=v(g2,m1)^2/100meg
* bottom-device power (W) and worst-device power (W)
BPD1 pd1 rtn V=v(m1,rtn)*v(hvp,top)/(4.5k*{rm})
BPD pdm rtn V=max(max(v(m1,rtn),v(m2,m1)),max(v(m3,m2),v(top,m3)))*v(hvp,top)/(4.5k*{rm})
.ends STACKWPF
