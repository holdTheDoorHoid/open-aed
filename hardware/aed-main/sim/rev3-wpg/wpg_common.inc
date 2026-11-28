* NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
* SPDX-License-Identifier: CERN-OHL-S-2.0
*
* wpg_common.inc — shared sub-circuits for the WP-G revision 0.3 check of sheet 13_hv_bridge AS DRAWN
* (hardware/aed-main/specs/13_hv_bridge.toml, revision 0.3, 27 Nov 2026). Re-uses the hand-built models of
* ../bridge (SiC switch, SiC diode, UCC5350 behaviour) and ../charger-storage/models/datasheet_models.lib
* (BAT54, ZENER). Nothing here is a maker's model; nothing here is an instruction to build anything.
*
*  hbridge3  power stage of ../bridge/bridge_common.inc 'hbridge' plus, as drawn in rev 0.3:
*            - shunt R1302 with an equivalent sense-loop inductance lsh (Kelvin pair SHUNT_KP = lss,
*              SHUNT_KN = kn; WSK2512 sense-loop inductance not published -> swept 0-1 nH [EST]);
*            - local film string C1347 + C1348 (940C30P15K-F, 0.15 uF each, ESR 8 mOhm and ESL 32 nH typ,
*              CDE 940C catalogue) in series through HV_CAP_FM (node fm), balancing R1370/R1371 10 MOhm,
*              from HV_CAP_P (capp, board side of the cable) to HV_RTN. Extra lead/loop inductance lfl per
*              capacitor (the -F part has 41 mm insulated stranded leads) [EST, swept]. cf1/cf2 = each
*              capacitance (tolerance +-10 %, K), cfail: 0 healthy, 1 = C1347 shorted, 2 = string open.
*  hsisland  circuit-level high-side gate island (HSA or HSB) WITH the rev-0.3 carrier-loss detector, as
*            ../rev3/carrier_loss.cir (same part models), driven by a gated +-vamp carrier.
*  fastpath  IDLE_FAST_N path as drawn: R1365/C1345/R1364 pedestal/D1332, OPA838 x 11 (R1366/R1367),
*            TLV3501 with R1368/R1369/C1346. Output 'flt' is 1 V when IDLE_FAST_N is LOW (current seen).

* (bridge_common.inc brings the SiC switch and diode models, hsdrv, lsdrv, hbridge and patient)
.include ../bridge/bridge_common.inc
.include ../charger-storage/models/datasheet_models.lib

.model D4148 d(is=2.5n n=1.75 rs=0.6 cjo=2p tt=4n bv=100)
* generic small-signal fits as ../rev3/carrier_loss.cir [EST]
.model PNPG pnp(is=2e-14 bf=300 vaf=60 br=8 rb=10 re=0.2 rc=0.5 cje=25p cjc=12p tf=0.6n tr=60n)
.model NPNG npn(is=1e-14 bf=200 vaf=100 br=4 rb=10 cje=4.5p cjc=3.5p tf=0.35n tr=200n)

*---------------------------------------------------------------------------------------------------
.subckt hbridge3 capi rtn oa ob gha ghb gla glb lss kn tjha tjla tjhb tjlb bus isn capp fm
+ params: rs=3.9 ls=30u lst=50n lcap=200n csn=3n rsn=10 sla=1e9 slb=1e9 tc=55 th=0 vt25=4.06
+ lsh=0.3n cf1=0.15u cf2=0.15u lfl=10n cfail=0 lsn=1p
Rcesr capi c1 0.13
Lcab c1 capp {lcap}
Rrs capp n1 {rs}
Lls n1 n2 {ls}
Rlsw n2 bus 0.08
Vfw bus fw0 0
D1302 fw0 fwm gc05mps33j
D1301 fwm fwk gc05mps33j
Lfw fwk capp 20n
* snubber; lsn = its ESL (S-3 uses 5 nH [EST, two 2225 MLCCs + track]; S-1/S-2 as ../bridge, none)
Csn bus sn0 {csn}
Lsn sn0 sn1 {lsn}
Rsn sn1 rtn {rsn}
* local film string C1347 (capp-fm) + C1348 (fm-rtn); ESL 32 nH + lfl each, ESR 8 mOhm each
Vcf capp f0 0
Lf1 f0 f1 {32n+lfl}
Rf1 f1 f2 {cfail==2 ? 1e9 : 8m}
Cf1 f2 fm {cf1}
Rshort1 f2 fm {cfail==1 ? 1m : 1e12}
R1370 capp fm 10meg
Lf2 fm f3 {32n+lfl}
Rf2 f3 f4 8m
Cf2 f4 rtn {cf2}
R1371 fm rtn 10meg
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
Loa oai oa 10n
Lob obi ob 10n
* output node capacitance to HV_RTN (tracks, relay contacts, divider) [EST 50 pF]
Coa oa rtn 50p
Cob ob rtn 50p
* shunt R1302: 2 mOhm + sense-loop inductance lsh between the Kelvin pair (lss = SHUNT_KP, kn = SHUNT_KN)
Rsh lss shm 2m
Lsh shm kn {lsh}
* (1 ohm across the sense-loop inductance: damping only, corner >= 500 MHz; keeps the solver out of trouble)
Rlsh shm kn 1
Dsh lss kn ds3d
Vsh kn rl 0
Lret rl rtn {lst/2}
Hsh isn rtn Vsh 1
.model ds3d d (is=1e-9 n=1.8 rs=0.02 cjo=50p)
Rdiva oa rtn 997k
Rdivb ob rtn 997k
.ends hbridge3

*---------------------------------------------------------------------------------------------------
* High-side island, circuit level. 'on' = 0/1 V carrier enable (already synchronised by the caller);
* the carrier stops within one 20 ns edge when 'on' falls (U1307 /CLR -> AND gates). Secondary +-vamp x
* b5v/5 V, 0.19 us dead time, 2 x 10 Ohm output resistance [EST WP-G 6.3]. ks = island reference (HSx_KS =
* HV_OUT_x); gate = QHx gate pin. det = 1 detector fitted (rev 0.3), 0 = removed / failed inert.
* Probe outputs: rect (HSx_RECT), pnpi (PNP collector current as V = A), clg (detector gate HSx_CLG).
.subckt hsisland on b5v gate ks rect pnpi clg params: det=1 vth=1.8 vamp=18.5 cg=47n
Vsq sq ks PULSE(0 1 0.19u 20n 20n 1.02u 2.5u)
Vsq2 sq2 ks PULSE(0 1 1.44u 20n 20n 1.02u 2.5u)
Bemf ea eb V = {vamp}*V(b5v)/5*min(max(V(on),0),1)*(V(sq,ks)-V(sq2,ks))
R1s ea s1 10
R2s eb s2 10
Rflt eb ks 100meg
* transformer interwinding capacitance to the primary (circuit ground), <= 10 pF (WP-G 6.3 requirement)
Ciw1 s1 0 5p
Ciw2 s2 0 5p
* D1313-D1316 bridge
D1313 ks s1 D4148
D1314 s1 rect D4148
D1315 ks s2 D4148
D1316 s2 rect D4148
C1315 rect ks 1n
R1328 rect ks 2.2k
XD1317 ks rect ZENER params: bvz=20
D1318 rect gr D4148
R1329 gr gate 100
C1316 gate ks {cg}
C1341 gate ks {cg}
R1332 gate ks 10k
XD1319 ks gate ZENER params: bvz=22
R1330 b rect 470
Q1306 qc b ee PNPG
Vpc qc ks 0
R1331 gate ee 22
Hpc pnpi ks Vpc 1
* carrier-loss detector (D1333 BAV70, R1373-R1377, C1350, Q1312, D1334, D1335, Q1313)
D1333a s1 clr D4148
D1333b s2 clr D4148
R1373 clr clf 4.7k
C1350 clf ks 47p
R1374 clf ks 4.7k
R1375 clf clb 10k
Q1312 clg clb ks NPNG
D1334 clb clg BAT54
R1376 gate clg 10k
XD1335 ks clg ZENER params: bvz=15
R1377 rect cld {det ? 100 : 1e12}
M1313 cld clg ks ks N7002
.model N7002 nmos(level=1 vto={vth} kp=0.08 rd=1 rs=1 cgso=20p cgdo=4p lambda=0.02)
.ends hsisland

*---------------------------------------------------------------------------------------------------
* Fast idle path as drawn. kp = SHUNT_KP, kn = SHUNT_KN, v33 = BRIDGE_3V3, gnd = GND.
* OPA838 macro [built here from SBOS867E, Aug 2025, VS = 3 V table]: single-pole integrator GBW 300 MHz
* (gm/C), slew rate +-250 V/us (minimum), output limited to vol .. 3.3 - 0.2 V (VOL 0.1 typ / 0.2 V max
* over temperature; VOH VS+ - 0.2 V min over temperature), input offset vos, bias current ib into both
* inputs (<= 3.7 uA, -40..85 C). TLV3501 [SBOS321E Apr 2016]: offset vosc (the run sets it to place the
* rising threshold at 'ith' amperes), hysteresis 6 mV typ, delay tcmp (7.5 typ / 10 ns max at 5 mV
* overdrive, 25 C; 12 ns over temperature).
.subckt fastpath kp kn v33 gnd flt aout ref params: vos=0 vosc=0 hyst=6m tcmp=10n vol=0.15 ib=1.5u
+ r1368=6.73k cin=100p
R1365 kp idf_in 100
C1345 idf_in kn {cin}
R1364 v33 idf_in 10k
D1332 gnd idf_in BAT54
Ibp idf_in gnd {ib}
Ibn idf_fb gnd {ib}
Vvos inp idf_in {vos}
* integrator: GBW = gm/(2 pi Cx), Cx 1 pF -> gm 1.885 mS; slew 250 V/us -> 0.25 mA
Gint gnd x cur = 'min(max(1.885m*(V(inp)-V(idf_fb)),-0.25m),0.25m)'
Cx x gnd 1p
Rx x gnd 1e9
Dxh x xh dideal
Vxh xh gnd 3.1
Dxl xl x dideal
Vxl xl gnd {vol}
.model dideal d(is=1e-14 n=0.05)
Bout aout gnd V = V(x,gnd)
R1366 aout idf_fb 1.00k
R1367 idf_fb kn 100
R1368 v33 ref {r1368}
R1369 ref kn 1.00k
C1346 ref kn 100n
* TLV3501: only the RISING trip matters here (sheet 12 latches it), so the comparator is a threshold at
* ref + vosc + hyst/2 (VOS is the mean of the two switching points, SBOS321E note 1), 1 ns smoothing, then the
* transport delay tcmp. (A regenerative Schmitt model stalled the solver in the same-side shorted case.)
Bc c0 gnd V = 0.5*(1+tanh(1000*(V(aout)-V(ref)-{vosc}-{hyst}/2)))
Rcq c0 cq 1k
Ccq cq gnd 1p
* delay tcmp as two RC stages (a transmission-line delay here stalled the solver); 50 % point ~ tcmp
Ecb cqb gnd cq gnd 1
Rd1 cqb d1 1k
Cd1 d1 gnd {tcmp/1.68/1k}
Ed1 d1b gnd d1 gnd 1
Rd2 d1b d2 1k
Cd2 d2 gnd {tcmp/1.68/1k}
Bfo flt gnd V = 0.5*(1+tanh(20*(V(d2,gnd)-0.5)))
.ends fastpath
