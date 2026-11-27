* NOT FOR CONSTRUCTION — unreviewed design. Simulated results; models as stated; nothing has been built or measured.
* SPDX-License-Identifier: CERN-OHL-S-2.0
* Shared control script for stack_dynamic_*.cir. Events (stack held, bottom gate -7.4 V):
*   0-50 ms  slow rise to 1,000 V (the stack settles held); 100 ms: step 1,000 -> 2,200 V in 1 us (bounding
*            case for any fast edge reaching a HELD stack)
*   0.25 s   a shock-like fall 2,200 -> 1,200 V in 10 ms (the sharing string's RC is ~33 ms, so the
*            3.3 nF capacitors, not the resistors, share this event)
*   0.45 s   rise back to 2,200 V in 2 ms (faster than any charger can do: bounding)
* Capacitor tolerance pattern: C0G +-5 % (KEMET J tolerance); the device across the LOW capacitor gets more.
.control
set noaskquit
option reltol=1e-4
foreach cfg 1 2 4 3
  alterparam dsrc = 0
  if $cfg = 1
    alterparam dev = 1
    alterparam gatefix = 0
    set cn = "drawn_10N100D2"
  end
  if $cfg = 2
    alterparam dev = 2
    alterparam gatefix = 1
    set cn = "gate-steered_2N170D2_4.7M_string"
  end
  if $cfg = 4
    alterparam dev = 2
    alterparam gatefix = 1
    alterparam dsrc = 1
    set cn = "gate-steered_2N170D2_plus_10M_and_3.3nF_across_each_device"
  end
  if $cfg = 3
    alterparam dev = 1
    alterparam gatefix = 0
    set cn = "drawn_10N100D2_ONE_DEVICE_SHORTED(dev3)"
    alterparam dsrc = 0
    alterparam rsh3 = 0.1
  end
  foreach k 1 2 3 4 5
    alterparam ct$k = 1.05
  end
  * device 1 (bottom) and device 2 in turn get the low capacitor
  foreach lowk 1 2
    foreach k 1 2 3 4 5
      alterparam ct$k = 1.05
    end
    alterparam ct$lowk = 0.95
    reset
    tran 0.2u 0.5 0 0.5m uic
    meas tran s1 max v(vdsmax) from=99m to=110m
    meas tran s2 max v(vdsmax) from=0.24 to=0.30
    meas tran s3 max v(vdsmax) from=0.44 to=0.5
    meas tran sdc find v(vdsmax) at=0.24
    let rat = v(pr)[0]
    let pnn = v(pn)[0]
    echo "DYN N=$&pnn cfg=$cn low_cap_on_string_elem=$lowk | step 1000->2200 V in 1us: max_vds=$&s1 | settled at 2200 V: $&sdc | 2200->1200 V in 10 ms: $&s2 | 1200->2200 V in 2 ms: $&s3 | rating 25C=$&rat"
  end
  alterparam rsh3 = 1e15
end
quit
.endc
