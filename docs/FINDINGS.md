# Simulation findings

27 November 2026. Found by running the three firmware parts together in the simulator (`firmware/crates/aed-sim`). Each has a test in `firmware/crates/aed-sim/tests/findings.rs` that is parked (marked "ignored") and fails today for the reason given. A finding is closed when its test passes and is un-parked.

**All of this is simulation with provisional settings. It says nothing about a real device.**

| No. | Finding | Seriousness | Status |
|---|---|---|---|
| F-1 | **Compressions that run on into an analysis can prevent a shock altogether.** If the responder keeps compressing for more than about 1.5–2 seconds after "Analysing", the first look at the signal contains movement. The analysis answers "not sure", the rescue sequence takes that as final, says "No shock advised" and starts two minutes of CPR. The same can happen every cycle. | **High** | **Fixed 27 Nov 2026.** "Not sure" no longer ends an analysis; the device repeats "do not touch the patient" and keeps analysing up to the 12 s limit. In simulation a shock still follows when compressions stop up to 6 s late. |
| F-2 | **The two voltage sensors are not allowed enough disagreement.** The 50 volt tolerance works only if each sensor is accurate to about 1%. Ordinary sensor error makes the supervisor declare a fault and disable the device for the rest of the rescue. | High | **Fixed 27 Nov 2026**, with one new problem: see F-9. Each sensor is now allowed 2% + 15 V of error. |
| F-3 | **A frozen main controller leaves the charger running.** The supervisor correctly empties the capacitor and no shock fires, but the charger keeps feeding the discharge load, holding the capacitor near 245 volts, and the device goes silent. Needs a hardware rule: the charger is cut off whenever the discharge path is open. | High | **Fixed in the model 27 Nov 2026.** The charger cannot run while the discharge path is open, and the supervisor has its own alarm. Both are now requirements on the future hardware. |
| F-4 | **A rhythm change in the last 4 seconds before the shock is not seen.** The analysis looks at a 4-second stretch of signal, so a heart that recovers one second before delivery is still shocked. | Medium | Open (design decision) |
| F-5 | The supervisor's record shows a misleading "permit refused" after a good shock when messages are slow. | Low | Open |
| F-6 | After pads come off during charging, the device says "Start CPR" and "Attach pads" at the same instant. | Low | Open |
| F-7 | A weak charger makes the device silent for 12 seconds at switch-on before it reports the fault. | Medium | Open |
| F-8 | Pads that are on but read just over 175 ohms get only "Attach pads", forever. See Q-16. | Medium | Open (owner question) |
| F-9 | **Child mode: in-tolerance sensors can allow a shock about 21% over target.** If the charger regulates high while both sensors read low, all checks pass at up to 1,227 volts true, which into a 25–35 ohm child delivers about 60 J instead of 50 J. The placeholder tolerance is 15%. Ways out: more accurate sensors (about 1% + 7 V), or a change to the child waveform at low resistance. The adult worst case is +13.6%, close to the limit. | High | Open (input to the sensor specification) |

A side effect of the F-1 fix: when the signal never becomes clean (very weak fibrillation, interference, a responder who never stops compressing), each analysis now lasts the full 12 seconds before the device says "Unable to analyse. No shock. Start CPR."

## Timing in simulation

| Step | Time |
|---|---|
| Analysis | 7.0 s |
| Charging (adult) | 6.0 s |
| Stand-clear countdown | 5.0 s |
| **Hands off the chest before the shock** | **about 17 s** |
| Pads on to first shock | 18.2 s (adult), 14.2 s (child) |

The research found each extra 5 seconds of pause before a shock is associated with lower survival, so 17 seconds is too long. See Q-13 and Q-14.

## Other observations

- An output stage that silently fails to fire gives "Shock not delivered" and a retry every two minutes; it is never declared a fault.
- On noisy borderline signals the analysis can waver between "shockable" and "not sure", and a single "not sure" during the countdown cancels the shock.
- A broken discharge load is not noticed until the device next needs to empty a fully charged capacitor.
- The rhythm analysis has only been run on made-up signals. It has not been tested on fast non-shockable rhythms, which are common in children.
