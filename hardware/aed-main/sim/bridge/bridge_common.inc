* NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
* SPDX-License-Identifier: CERN-OHL-S-2.0
*
* Shared sub-circuits for the sheet 13 / sheet 12 bridge simulations.
* Topology and values from hardware/aed-main/specs/13_hv_bridge.toml (27 Nov 2026) and capture notes WP-G.md:
*   HV_CAP_P -R_S 3.9 Ohm- -L_S 30 uH- HV_BUS ; D_FW = D1302 + D1301 in series, anode HV_BUS, cathode HV_CAP_P ;
*   snubber HV_BUS - 2 x 10 nF X7R in series (effective csn, 2.5-3.5 nF at 2 kV [EST WP-G]) - 10 Ohm - HV_RTN ;
*   Q1301 QHA HV_BUS->HV_OUT_A, Q1302 QLA HV_OUT_A->LS_SRC, Q1303 QHB, Q1304 QLB ; shunt R1302 2 mOhm LS_SRC->HV_RTN ;
*   output dividers 997 kOhm to GND.
* Stray inductance lst is split: half in the bus between the snubber/D_FW node and the switch drains, half in the
* return between the shunt and the snubber's HV_RTN end. lcap is the cable to the off-board capacitor.

.include models/msc080sma330b4n_beh.lib
.include models/gc05mps33j_beh.lib

*---------------------------------------------------------------------------------------------------
* High-side gated push-pull drive (sheet 13 HSA/HSB island), behavioural, from WP-G section 6.2:
*   carrier on -> rectified source 18.5 V x (BRIDGE_5V/5 V) [tracks the 5 V rail, WP-G 6.2] behind 20 Ohm
*   [EST WP-G 6.3]; rectified node RECT 1 nF + 2.2 kOhm bleed ; gate path 1N4148W (0.7 V) + 100 Ohm into
*   2 x 47 nF C0G + the MOSFET ; 10 kOhm bleed ; PNP BC807-40 emitter follower, 22 Ohm + rpnp (6.7-18 Ohm,
*   WP-G), base 470 Ohm to RECT: modelled as I = max(0, Vg - Vrect - 0.65)/(22 + rpnp).
*   'on' is a 0/1 V logic node ALREADY including the 1.3-3.8 us synchronisation delay (set by the caller).
.subckt hsdrv on b5v gate src params: rpnp=12 cg=94n rdis=1e9
Bsrc src rect I = min(max(V(on),0),1) * 0.05*ln(1+exp((18.5*V(b5v)/5 - V(rect,src))/0.05))/20
Crect rect src 1n
Rrect rect src 2.2k
* OPTION (absent as drawn, rdis = 1e9): a discharge path that empties RECT through rdis as soon as the carrier
* stops (stands for a carrier-loss detector on the floating side; see notes.md, question 1)
Bdis rect src I = (1 - min(max(V(on),0),1)) * V(rect,src)/rdis
Dg rect gx d1n4148
Rg gx gate 100
Cg gate src {cg}
Rbl gate src 10k
Bpnp gate src I = 0.02*ln(1+exp((V(gate,rect) - 0.65)/0.02))/(22 + rpnp)
.model d1n4148 d (is=2.5n n=1.75 rs=0.6 cjo=2p tt=4n)
.ends hsdrv

*---------------------------------------------------------------------------------------------------
* Low-side driver UCC5350MC, behavioural from the datasheet (SLLSER8J) figures used in WP-G 6.5:
*   prop delay td (65 ns typ / 100 ns max), output 10 A peak (2.5 Ohm equivalent from 25 V), UVLO2 falling
*   10.3-11 V (uvf), rising 12-13 V (uvr), output low under UVLO; turn-on 220 Ohm, turn-off 33 Ohm +
*   1N4148W in parallel with the 220 Ohm, 10 kOhm gate-source. Miller clamp not modelled (gate is already
*   pulled to -5 V through 33 Ohm). Rails: vpos = +20 V, vneg = -5 V relative to the Kelvin source 'ks'.
*   'in' is a 0/1 V logic node relative to circuit ground (BRIDGE_3V3 domain, ground bounce ignored).
.subckt lsdrv in vpos vneg gate ks params: td=65n uvf=10.65 uvr=12.5
Bin ind 0 V = V(in)
* (logic inputs enter linearly, clipped 0..1: hard if-then-else or steep tanh steps stalled the solver;
*  callers give every logic edge a 20 ns ramp)
Tdl ind 0 indl 0 Z0=50 TD={td}
Rtl indl 0 50
Buv uvo 0 V = 0.5*(1+tanh(4*(V(vpos,vneg) - (uvr - (uvr-uvf)*V(uvq)))))
Ruv uvo uvq 1k
Cuv uvq 0 10p
Bout o ks V = V(vneg,ks) + V(vpos,vneg)*min(max(V(indl),0),1)*V(uvq)
Ro o o2 2.5
Ron o2 gate 220
Roff o2 oo 33
Doff gate oo d1n4148l
Rgs gate ks 10k
.model d1n4148l d (is=2.5n n=1.75 rs=0.6 cjo=2p tt=4n)
.ends lsdrv

*---------------------------------------------------------------------------------------------------
* Power stage. Ports: capi (capacitor + inside its can), rtn (HV_RTN = 0), oa ob (HV_OUT_A/B at the board),
* gha ghb (high-side gate pins), gla glb (low-side gate pins), lss (LS_SRC = low-side Kelvin source),
* tjha tjla tjhb tjlb (junction temperatures), bus (HV_BUS), isn (shunt current, 1 V/A).
* Current probes: Vmha Vmla Vmhb Vmlb (drain currents), Vsh (shunt), Vfw (freewheel diodes).
* Fault parameters: sla / slb = resistance put across QLA / QLB (1e9 = healthy).
.subckt hbridge capi rtn oa ob gha ghb gla glb lss tjha tjla tjhb tjlb bus isn
+ params: rs=3.9 ls=30u lst=50n lcap=200n csn=3n rsn=10 sla=1e9 slb=1e9 tc=55 th=0 vt25=4.06 cloc=1p ccl=1p rcl=10k
Rcesr capi c1 0.13
Lcab c1 capp {lcap}
Rrs capp n1 {rs}
Lls n1 n2 {ls}
Rlsw n2 bus 0.08
* freewheel diodes D1302 + D1301 (anode HV_BUS, cathode HV_CAP_P)
Vfw bus fw0 0
D1302 fw0 fwm gc05mps33j
D1301 fwm fwk gc05mps33j
Lfw fwk capp 20n
* snubber
Csn bus sn1 {csn}
Rsn sn1 rtn {rsn}
* OPTIONS studied in q2 (absent as drawn: 1 pF placeholders):
*  cloc = local high-voltage film capacitor HV_CAP_P - HV_RTN on the board (0.1 ohm ESR)
*  ccl, rcl = RCD clamp: diode HV_BUS -> clamp capacitor ccl (to HV_RTN), bled back to HV_CAP_P by rcl
Cloc capp cl1 {cloc}
Rloc cl1 rtn 0.1
Dcl bus clp gc05mps33j
Ccl clp rtn {ccl}
Rcl clp capp {rcl}
* bus stray to the drains
Lbus bus drn {lst/2}
Vmha drn dha 0
Vmhb drn dhb 0
XQHA dha gha oai tjha msc080b params: tc={tc} th={th} vt25={vt25}
XQHB dhb ghb obi tjhb msc080b params: tc={tc} th={th} vt25={vt25}
Vmla oai dla 0
Vmlb obi dlb 0
XQLA dla gla lss tjla msc080b params: tc={tc} th={th} vt25={vt25}
XQLB dlb glb lss tjlb msc080b params: tc={tc} th={th} vt25={vt25}
Rsla dla lss {sla}
Rslb dlb lss {slb}
* outputs (10 nH of board track each)
Loa oai oa 10n
Lob obi ob 10n
* shunt R1302 + bypass D1303 (S3D)
Rsh lss shn 2m
Dsh lss shn ds3d
Vsh shn rl 0
Lret rl rtn {lst/2}
* shunt current as a voltage, 1 V per A (positive = conventional current LS_SRC -> HV_RTN)
Hsh isn rtn Vsh 1
.model ds3d d (is=1e-9 n=1.8 rs=0.02)
* output dividers 4 x 249 k + 1.00 k to GND
Rdiva oa rtn 997k
Rdivb ob rtn 997k
.ends hbridge

*---------------------------------------------------------------------------------------------------
* Patient path: relay contacts 2 x 0.05 Ohm [UNVERIFIED, design doc], lead inductance lpat split, patient rp.
* Current probe Vpat (positive = from A to B).
.subckt patient oa ob params: rp=50 lpat=1.5u
Rka oa p1 0.05
Lpa p1 p2 {lpat/2}
Vpat p2 p3 0
Rpat p3 p4 {rp}
Lpb p4 p5 {lpat/2}
Rkb p5 ob 0.05
.ends patient
