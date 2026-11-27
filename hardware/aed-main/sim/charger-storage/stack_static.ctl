* NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
* SPDX-License-Identifier: CERN-OHL-S-2.0
* Shared control script for stack_static_*.cir (included by each). Prints one line per case:
*   CASE <name> T=<cold|hot> V=<capacitor volts> max_vds=<V> rating=<V> ratio=<max/rating>
* cold = -20 C: cut-off x0.93, leakage 'leakm20', rating 'vratecold'; hot = +70 C: cut-off x1.06, leakage
* 'leak70', rating 'vrate' (WP-F Fig. 11 readings; leakage log-interpolated, see the netlist).
.control
set noaskquit
set rndseed = 20261127
option reltol=1e-4 gmin=1e-15
foreach cfg 1 2 3 4 5
alterparam dsrc = 0
if $cfg = 1
  alterparam dev = 1
  alterparam gatefix = 0
end
if $cfg = 2
  alterparam dev = 2
  alterparam gatefix = 0
end
if $cfg = 3
  alterparam dev = 1
  alterparam gatefix = 1
end
if $cfg = 5
  alterparam dev = 2
  alterparam gatefix = 1
  alterparam dsrc = 1
end
if $cfg = 4
  alterparam dev = 2
  alterparam gatefix = 1
end
reset
op
let tmp = v(pn)
set nn = $&tmp
let tmp = v(pr)
set r25 = $&tmp
let tmp = v(prc)
set rcold = $&tmp
let tmp = v(pstr)
set rs = $&tmp
let tmp = v(pvoffnom)
set von = $&tmp
let tmp = v(pl70)
set l70 = $&tmp
let tmp = v(plm20)
set lm20 = $&tmp
let tmp = v(pl25)
set l25 = $&tmp
foreach j 1 2 3 4 5
  alterparam vo$j = $von
  alterparam lk$j = 0.3
  alterparam rt$j = 1
  alterparam rsh$j = 1e15
  alterparam rso$j = 1e15
end
alterparam vofft = 1.0
echo "CONFIG cfg=$cfg (1 drawn 10N100D2, 2 drawn 2N170D2, 3 gate-steered 10N100D2, 4 gate-steered 2N170D2, 5 = 4 + 10 Mohm and 3.3 nF across each device)"
echo "STACK N=$nn cfg=$cfg string_element_ohm=$rs voff_nominal=$von rating_25C=$r25 rating_m20C=$rcold"

* ---------- A: nominal, typical leakage (0.3 x max at 25 C), nominal cut-off --------------------------
foreach vv 1950 2008.6 2182 2200
  alterparam vcap = $vv
  foreach k 1 2 3 4 5
    alterparam lk$k = 0.3
  end
  alterparam leakt = $l25
  reset
  op
  echo "CASE cfg=$cfg nominal_25C V=$vv max_vds=$&v(vdsmax) min_vds=$&v(vdsmin) stack+string_uA=$&v(istk) | v1=$&v(vds1) v2=$&v(vds2) v3=$&v(vds3) v4=$&v(vds4) v5=$&v(vds5)"
end

* ---------- B: worst-case string tolerance (+-2 % = 1 % initial + 1 % life [EST]) on element k -------
alterparam vcap = 2200
foreach k 1 2 3 4 5
  if $k <= $nn
    foreach j 1 2 3 4 5
      alterparam rt$j = 0.98
    end
    alterparam rt$k = 1.02
    reset
    op
    echo "CASE cfg=$cfg string_tol_elem$k T=25 V=2200 max_vds=$&v(vdsmax) min_vds=$&v(vdsmin)"
  end
end
foreach j 1 2 3 4 5
  alterparam rt$j = 1
end

* ---------- C: one device shorted, each position, cold and hot ---------------------------------------
foreach tc 0 1
  if $tc = 0
    alterparam vofft = 0.93
    alterparam leakt = $lm20
    set rat = $rcold
    set tn = cold
  else
    alterparam vofft = 1.06
    alterparam leakt = $l70
    set rat = $r25
    set tn = hot
  end
  foreach vv 1950 2008.6 2182 2200
    alterparam vcap = $vv
    foreach k 1 2 3 4 5
      if $k <= $nn
        alterparam rsh$k = 0.1
        reset
        op
        let rr = v(vdsmax)/$rat
        echo "CASE cfg=$cfg short_dev$k T=$tn V=$vv max_vds=$&v(vdsmax) rating=$rat ratio=$&rr"
        alterparam rsh$k = 1e15
      end
    end
  end
end

* ---------- D: one sharing-string element shorted (its tap interval collapses), cold -----------------
alterparam vcap = 2200
alterparam vofft = 0.93
alterparam leakt = $lm20
foreach k 1 2 3 4 5
  if $k <= $nn
    alterparam rso$k = 0.1
    reset
    op
    let rr = v(vdsmax)/$rcold
    echo "CASE cfg=$cfg short_string$k T=cold V=2200 max_vds=$&v(vdsmax) rating=$rcold ratio=$&rr"
    alterparam rso$k = 1e15
  end
end

* ---------- E: leakage mismatch, hot: bottom device 2 % of max, device k at max -----------------------
alterparam vofft = 1.06
alterparam leakt = $l70
foreach k 2 3 4 5
  if $k <= $nn
    foreach j 1 2 3 4 5
      alterparam lk$j = 0.02
    end
    alterparam lk$k = 1
    reset
    op
    let rr = v(vdsmax)/$r25
    echo "CASE cfg=$cfg leak_mismatch_dev$k T=hot V=2200 max_vds=$&v(vdsmax) ratio=$&rr | v1=$&v(vds1) v2=$&v(vds2) v3=$&v(vds3) v4=$&v(vds4) v5=$&v(vds5)"
  end
end
foreach j 2 3 4 5
  alterparam lk$j = 1
end
alterparam lk1 = 0.02
reset
op
let rr = v(vdsmax)/$r25
echo "CASE cfg=$cfg leak_mismatch_allupper T=hot V=2200 max_vds=$&v(vdsmax) ratio=$&rr | v1=$&v(vds1) v2=$&v(vds2) v3=$&v(vds3) v4=$&v(vds4) v5=$&v(vds5)"

* ---------- F: Monte Carlo, 150 runs per corner -----------------------------------------------------
* string +-2 % uniform; cut-off uniform voff_nominal +-1 V (the datasheet range) x temperature factor;
* leakage uniform 2..100 % of max; without a fault, and with one shorted device at a random position.
foreach tc 0 1
  if $tc = 0
    alterparam vofft = 0.93
    alterparam leakt = $lm20
    set rat = $rcold
    set tn = cold
  else
    alterparam vofft = 1.06
    alterparam leakt = $l70
    set rat = $r25
    set tn = hot
  end
  foreach flt 0 1
    set worst = 0
    set run = 0
    while $run < 100
      let u = sunif(vector(16))
      foreach j 1 2 3 4 5
        let a = 1 + 0.02*u[$j]
        alterparam rt$j = $&a
        let b = 0.51 + 0.49*u[$j+5]
        alterparam lk$j = $&b
        let c = $von + 1.0*u[$j+10]
        alterparam vo$j = $&c
        alterparam rsh$j = 1e15
      end
      if $flt = 1
        let kk = floor((u[0]+1)/2*$nn) + 1
        if kk > $nn
          let kk = $nn
        end
        set kks = $&kk
        alterparam rsh$kks = 0.1
      end
      reset
      op
      let cur = v(vdsmax)
      if cur > $worst
        set wdet = "v1=$&v(vds1) v2=$&v(vds2) v3=$&v(vds3) v4=$&v(vds4) v5=$&v(vds5) t1=$&v(t1) t2=$&v(t2) m1=$&v(m1) m2=$&v(m2)"
        set worst = $&cur
      end
      let rn = $run + 1
      set run = $&rn
    end
    let rr = $worst/$rat
    echo "CASE cfg=$cfg montecarlo_100 T=$tn shorted_device=$flt V=2200 worst_max_vds=$worst rating=$rat ratio=$&rr | worst run: $wdet"
  end
end
foreach j 1 2 3 4 5
  alterparam rsh$j = 1e15
end
end
quit
.endc
