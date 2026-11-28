# NOT FOR CONSTRUCTION — unreviewed design. Calculated results; tolerances as stated; nothing has been built or measured.
# SPDX-License-Identifier: CERN-OHL-S-2.0
#
# ovp_band.py — worst-case and Monte Carlo trip band of the latching OVP comparator on sheet 08 (revision 0.3,
# WP-C C-2), -20 to +70 C (45 K either side of 25 C), from the parts actually drawn. Python 3 standard library.
# Every tolerance and its source is listed in RNG below and in ../../capture-notes/WP-C.md "Revision 0.3".
#
# Trip condition (output low before the trip, VOL at the output):
#   OVP_SENSE = OVP_REF + Vos
#   VCAPS_BUF = (OVP_REF + Vos) x (1 + R805/R806) - VOL x R805/R806 + I_leak x R805
#   capacitor = (VCAPS_BUF - follower error) x 1,002.0 x (1 + string ratio error)
import itertools
import random

R807, R808, R805, R806 = 21.5e3, 15.6e3, 100e3, 100e3
N = 1002.0                     # supervisor string ratio, contract §5.6 issue 3 (WP-C C-1)
K = 45.0                       # kelvin from 25 C to -20 C or +70 C

def trip(vr_e, r807e, r808e, r805e, r806e, vos, vol, ileak, fol, string):
    vref = 2.5 * (1 + vr_e) * R808 * (1 + r808e) / (R807 * (1 + r807e) + R808 * (1 + r808e))
    a, b = R805 * (1 + r805e), R806 * (1 + r806e)
    vbuf = (vref + vos) * (1 + a / b) - vol * a / b + ileak * a
    return (vbuf - fol) * N * (1 + string)

VR = 0.001 + 8e-6 * K + 500e-6 + 70e-6   # ADR3425: 0.1 % initial, 8 ppm/K max, life 500 ppm ASSUMED (30 ppm/1000 h
                                         # at 50 C stated, Rev. B), thermal hysteresis 70 ppm
RR = 0.001 + 25e-6 * K + 500e-6          # TNPW 0.1 %, 25 ppm/K, life 0.05 % ASSUMED; each resistor independent
RNG = {
    "vref":   (-VR, VR),
    "R807":   (-RR, RR),
    "R808":   (-RR, RR),
    "R805":   (-RR, RR),
    "R806":   (-RR, RR),
    "Vos":    (-2e-3, 2e-3),            # TLV9031 -40..125 C, SNOSDA3H §5.7
    "VOL":    (0.0, 1e-3),              # output low sinking <= 20 uA [EST from 175 mV at 4 mA]
    "Ileak":  (0.0, 100e-9),            # OVP_SENSE to GND: Q801 off-state [EST, 100 nA at 30 V 25 C is the
                                        # nearest datasheet figure] + TLV9031 IB 5 pA
    "follow": (-0.07e-3, 0.07e-3),      # OPA388 Vos + IB x source (SBOS777D), from the issue-2 notes
    "string": (-0.00675, 0.00675),      # supervisor string: 0.2 % tolerance, 0.225 % temperature, 0.25 % life (CS Q4)
}
KEYS = list(RNG)

def corners(rng):
    vals = [trip(*c) for c in itertools.product(*[rng[k] for k in KEYS])]
    return min(vals), max(vals)

nom = trip(*([0.0] * len(KEYS)))
print("RESULT nominal trip %.1f V (OVP_REF %.5f V)" % (nom, 2.5 * R808 / (R807 + R808)))
lo, hi = corners(RNG)
print("RESULT worst case %.1f - %.1f V" % (lo, hi))
no_leak = dict(RNG, Ileak=(0.0, 0.0))
print("RESULT worst case without the 100 nA leakage term %.1f - %.1f V" % corners(no_leak))
only08 = dict(RNG, string=(0.0, 0.0))
lo8, hi8 = corners(only08)
print("RESULT sheet-08 parts only, as VCAPS_BUF volts: %.4f - %.4f V" % (lo8 / N, hi8 / N))
random.seed(1)
s = sorted(trip(*[random.uniform(*RNG[k]) for k in KEYS]) for _ in range(200000))
print("RESULT Monte Carlo (uniform, 200,000): 0.1 %% %.1f V, 99.9 %% %.1f V, extremes %.1f / %.1f V"
      % (s[200], s[-200], s[0], s[-1]))
print("RESULT margins: bottom above the firmware window top 2,008.6 V by %.1f V; bottom above the L1 firmware "
      "limit's bottom 2,043 V by %.1f V (the two bands overlap above that: 2,072-2,098 V); top below the "
      "2,200 V rating by %.1f V" % (lo - 2008.6, lo - 2043.0, 2200.0 - hi))
for k in KEYS:
    c = [0.0] * len(KEYS)
    i = KEYS.index(k)
    c[i] = RNG[k][0]
    a = trip(*c)
    c[i] = RNG[k][1]
    b = trip(*c)
    print("TERM %-7s %+6.1f %+6.1f V" % (k, a - nom, b - nom))
