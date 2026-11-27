"""Cross-sheet consistency check over all specs: single-ended inter-sheet nets, pull resistors per net, rails per sheet.
Run from the repo root: python3 tools/xcheck.py [aed-main]   (report only; never fails the build).
Compare its output with hardware/aed-main/contract.md: every inter-sheet net there must appear here with the same
driver and loads, and every pull must be on the sheet the contract names."""
import glob, os, sys, difflib, collections
sys.path.insert(0, ".")
from tools.schgen import spec, project

BOARD = sys.argv[1] if len(sys.argv) > 1 else "aed-main"
sheets = [spec.load(p) for p in sorted(glob.glob(f"hardware/{BOARD}/specs/*.toml"))]
# Hierarchical labels of multi-instance sheets: one root net per instance (hierarchy.toml pins = {...}).
_hier = project.load_hierarchy(f"hardware/{BOARD}/hierarchy.toml")
hmaps = {e.name: [{k: project.pin_net(v, inst, i + 1) for k, v in e.pins.items()} for i, inst in enumerate(e.instances)]
         for e in _hier.sheets}
def roots(s, net):
    """Root-level names of a spec net: itself if global/power, one per instance if hierarchical, else none."""
    if net in s.global_nets or net in s.power: return [net]
    if net in s.hier_nets: return [m[net] for m in hmaps.get(s.name, []) if net in m]
    return []
use = collections.defaultdict(lambda: collections.defaultdict(list))   # net -> sheet -> [(ref,pin,name,part)]
scope = collections.defaultdict(set)
for s in sheets:
    for n in s.global_nets: scope[n].add("global")
    for n in s.power: scope[n].add("power")
    for n in s.hier_nets:
        for r in roots(s, n): scope[r].add("global")
    for p in s.parts:
        for pm in p.pins:
            for r in roots(s, pm.net):
                use[r][s.name].append((p.ref, pm.number, pm.name, p))

mixed = {n: sc for n, sc in scope.items() if len(sc) > 1}
print("== nets declared both power and global:", mixed or "none")

single = sorted(n for n in use if len(use[n]) == 1)
multi = sorted(n for n in use if len(use[n]) > 1)
print(f"== {len(use)} inter-sheet nets; {len(multi)} on >=2 sheets; {len(single)} on one sheet only")
allnames = sorted(use)
def norm(n): return n.replace("_N", "").replace("_", "").upper()
print("== single-sheet nets (sheet : net : near names elsewhere)")
for n in single:
    sh = next(iter(use[n]))
    near = [m for m in difflib.get_close_matches(n, [x for x in allnames if x != n], n=3, cutoff=0.82)]
    near += [m for m in allnames if m != n and norm(m) == norm(n) and m not in near]
    print(f"  {sh:20s} {n:26s} {'~ ' + ', '.join(near) if near else ''}")

print("== pulls per net (R with other end on a rail/GND), nets with more than one pull")
rails = {n for n, sc in scope.items() if "power" in sc}
pulls = collections.defaultdict(list)
for s in sheets:
    for p in s.parts:
        if p.lib_id.startswith("Device:R") and len(p.pins) == 2:
            a, b = p.pins[0].net, p.pins[1].net
            for x, y in ((a, b), (b, a)):
                if y in rails:
                    for xr in roots(s, x):
                        if xr not in rails and xr in use:
                            pulls[xr].append((s.name, p.ref, p.value, y, "DNP" if p.dnp else ""))
for n in sorted(pulls):
    if len(pulls[n]) > 1:
        print(f"  {n}: " + "; ".join(f"{sh} {r} {v}->{to} {d}".strip() for sh, r, v, to, d in pulls[n]))

# Sheets that hold a controller: a multi-sheet signal net with no endpoint on either is reported (it may be fine,
# e.g. a rail-gated logic net between the interlock and the bridge, but each one should be explained by contract.md).
OWNERS = {"aed-main": ("04_main_mcu", "05_supervisor_mcu")}.get(BOARD, ())
print("== multi-sheet signal nets with no endpoint on " + " / ".join(OWNERS))
for n in multi:
    if "power" in scope[n]: continue
    shs = sorted(use[n])
    if not any(s in OWNERS for s in shs):
        print(f"  {n:22s} {', '.join(shs)}")
print("== resistor-pack pulls overlapping a discrete pull")
packpull = collections.defaultdict(list)
for s in sheets:
    for p in s.parts:
        if "R_Pack04" in p.lib_id:
            pm = {x.number: x.net for x in p.pins}
            for a, b in (("1", "8"), ("2", "7"), ("3", "6"), ("4", "5")):
                x, y = pm.get(a), pm.get(b)
                for u, v in ((x, y), (y, x)):
                    if v in rails and u not in rails and u in use:
                        packpull[u].append((s.name, p.ref, p.value, v))
for n in sorted(packpull):
    allp = packpull[n] + [t[:4] for t in pulls.get(n, [])]
    if len(allp) > 1:
        print(f"  {n}: " + "; ".join(f"{a} {b} {c}->{d}" for a, b, c, d in allp))
print("== power rails: sheets")
for n in sorted(rails):
    print(f"  {n:16s} {', '.join(sorted(use[n])) if n in use else '(unused)'}")
