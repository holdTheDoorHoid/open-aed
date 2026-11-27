#!/usr/bin/env python3
"""Sanity checker for the project's KiCad symbol libraries (stdlib only).

Usage:
    python3 tools/libcheck.py [--fp-root DIR] [lib.kicad_sym ...]

With no file arguments it checks hardware/lib/ws_*.kicad_sym.

Per symbol it reports:
  * duplicate pin numbers (within the whole symbol, all units)
  * pins without a number
  * pin count (distinct numbers) vs the `PackagePins` field
  * missing / empty required fields
  * pins of invalid electrical type, pins off the 1.27 mm grid, and two pins
    with different names on the same spot in one unit (layout bugs)
  * if the Footprint field names a library found under --fp-root (stock
    footprints) or hardware/lib/<nick>.pretty, the footprint's distinct pad
    numbers vs the symbol's pin numbers.
Symbols that only `extends` another are checked for fields only.
Exit status 1 if any ERROR was found (WARN does not fail).
"""
import glob
import os
import re
import sys

REQUIRED = ["Reference", "Value", "Footprint", "Datasheet", "MPN",
            "Manufacturer", "PinoutVerified", "PackagePins"]
# Footprint may legitimately be empty only if the symbol says why.
TYPES = {"input", "output", "bidirectional", "tri_state", "passive", "free",
         "unspecified", "power_in", "power_out", "open_collector",
         "open_emitter", "no_connect"}
DEFAULT_FP_ROOT = ("/home/hoid/.local/share/flatpak/runtime/org.kicad.KiCad.Library."
                   "Footprints/x86_64/stable/active/files/footprints")

TOKEN = re.compile(r'"((?:[^"\\]|\\.)*)"|([()])|([^\s()"]+)')


def parse(text):
    stack, cur = [], []
    for m in TOKEN.finditer(text):
        s, paren, atom = m.groups()
        if paren == "(":
            stack.append(cur)
            cur = []
        elif paren == ")":
            done = cur
            cur = stack.pop()
            cur.append(done)
        elif s is not None:
            cur.append(("STR", s.replace('\\"', '"').replace("\\\\", "\\")))
        else:
            cur.append(atom)
    return cur[0]


def val(x):
    return x[1] if isinstance(x, tuple) else x


def children(node, head):
    return [c for c in node if isinstance(c, list) and c and c[0] == head]


def pins_of(sym):
    out = []
    for sub in children(sym, "symbol"):
        m = re.search(r"_(\d+)_(\d+)$", val(sub[1]))
        unit = int(m.group(1)) if m else 0
        for p in children(sub, "pin"):
            typ = p[1]
            at = children(p, "at")[0]
            name = val(children(p, "name")[0][1]) if children(p, "name") else ""
            num = val(children(p, "number")[0][1]) if children(p, "number") else ""
            hidden = any(isinstance(c, list) and c[:2] == ["hide", "yes"] for c in p) or "hide" in p
            out.append(dict(unit=unit, type=typ, x=float(at[1]), y=float(at[2]),
                            name=name, num=num, hidden=hidden))
    return out


def fp_pads(fp, fp_root, local_root):
    if ":" not in fp:
        return None, "no library nickname"
    nick, name = fp.split(":", 1)
    for root in (local_root, fp_root):
        path = os.path.join(root, nick + ".pretty", name + ".kicad_mod")
        if os.path.isfile(path):
            tree = parse(open(path, encoding="utf-8").read())
            nums = set()
            for pad in children(tree, "pad"):
                n = val(pad[1])
                if n != "":
                    nums.add(n)
            return nums, path
    return None, "footprint file not found (planned custom footprint?)"


def check_file(path, fp_root, local_root):
    errors = warns = 0
    tree = parse(open(path, encoding="utf-8").read())
    print("== %s" % path)
    for sym in children(tree, "symbol"):
        name = val(sym[1])
        props = {val(p[1]): val(p[2]) for p in children(sym, "property")}
        msgs = []
        for f in REQUIRED:
            if f not in props:
                msgs.append(("ERROR", "missing field %s" % f))
            elif props[f].strip() == "" and f != "Footprint":
                msgs.append(("ERROR", "empty field %s" % f))
        if props.get("Footprint", "x").strip() == "":
            msgs.append(("WARN", "Footprint field empty"))
        if children(sym, "extends"):
            pass
        else:
            pins = pins_of(sym)
            nums = {}
            for p in pins:
                if p["num"].strip() == "":
                    msgs.append(("ERROR", "pin '%s' has no number" % p["name"]))
                    continue
                nums.setdefault(p["num"], []).append(p)
                if p["type"] not in TYPES:
                    msgs.append(("ERROR", "pin %s invalid type %s" % (p["num"], p["type"])))
                for c in (p["x"], p["y"]):
                    if abs(c / 1.27 - round(c / 1.27)) > 1e-6:
                        msgs.append(("WARN", "pin %s off 1.27 mm grid" % p["num"]))
                        break
            for n, lst in nums.items():
                if len(lst) > 1:
                    msgs.append(("ERROR", "duplicate pin number %s (%s)" %
                                 (n, ", ".join(q["name"] for q in lst))))
            spots = {}
            for p in pins:
                key = (p["unit"], round(p["x"], 3), round(p["y"], 3))
                spots.setdefault(key, set()).add(p["name"])
            for key, names in spots.items():
                if len(names) > 1:
                    msgs.append(("ERROR", "unit %d: different pins share position (%g,%g): %s" %
                                 (key[0], key[1], key[2], ", ".join(sorted(names)))))
            pp = props.get("PackagePins", "")
            if pp.isdigit():
                if int(pp) != len(nums):
                    msgs.append(("ERROR", "pin count %d != PackagePins %s" % (len(nums), pp)))
            elif pp:
                msgs.append(("ERROR", "PackagePins not an integer: %r" % pp))
            fp = props.get("Footprint", "")
            if fp:
                pads, where = fp_pads(fp, fp_root, local_root)
                if pads is None:
                    msgs.append(("WARN", "footprint %s: %s" % (fp, where)))
                else:
                    sym_nums = set(nums)
                    if pads != sym_nums:
                        only_s = sorted(sym_nums - pads)
                        only_f = sorted(pads - sym_nums)
                        msgs.append(("ERROR", "footprint %s pads differ: symbol-only %s, footprint-only %s" %
                                     (fp, only_s[:12], only_f[:12])))
            info = "%d pins, %d units" % (len(nums), len({p['unit'] for p in pins}))
        if children(sym, "extends"):
            info = "extends " + val(children(sym, "extends")[0][1])
        status = "OK" if not msgs else ("FAIL" if any(m[0] == "ERROR" for m in msgs) else "WARN")
        print("  %-4s %-22s %s" % (status, name, info))
        for lvl, m in msgs:
            print("       %s: %s" % (lvl, m))
            if lvl == "ERROR":
                errors += 1
            else:
                warns += 1
    return errors, warns


def main(argv):
    fp_root = DEFAULT_FP_ROOT
    files = []
    it = iter(argv)
    for a in it:
        if a == "--fp-root":
            fp_root = next(it)
        else:
            files.append(a)
    repo = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    local_root = os.path.join(repo, "hardware", "lib")
    if not files:
        files = sorted(glob.glob(os.path.join(local_root, "ws_*.kicad_sym")))
    te = tw = 0
    for f in files:
        e, w = check_file(f, fp_root, local_root)
        te += e
        tw += w
    print("\nTOTAL: %d error(s), %d warning(s) in %d file(s)" % (te, tw, len(files)))
    return 1 if te else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
