"""Regenerate every board's schematics and run all checks.

    python3 -m tools.schgen.build            (from the repo root; tools/build.sh wraps this)

Exit status is non-zero if anything fails: spec validation, pin-name cross-check,
footprint resolution, ERC errors or unaccepted ERC warnings, the netlist
round-trip, or PDF export.
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import subprocess
import sys

from . import kicad, netcheck, project, spec, symgen, symlib, writer

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BOARDS = ["main-board", "remote-head"]


class BuildError(Exception):
    pass


def _log(msg: str) -> None:
    print(msg, flush=True)


def build_symbol_libs(lib_dir: str) -> list[str]:
    made = []
    for src in sorted(glob.glob(os.path.join(lib_dir, "src", "*.toml"))):
        lib = os.path.splitext(os.path.basename(src))[0]
        symgen.write_library(src, os.path.join(lib_dir, f"{lib}.kicad_sym"))
        made.append(lib)
        _log(f"  symbols: {lib}.kicad_sym from src/{os.path.basename(src)}")
    return made


def footprint_report(hier: project.Hierarchy, sheets: dict[str, spec.Sheet]) -> tuple[list[str], list[str]]:
    """Returns (errors, todo list)."""
    fp_dirs = {os.path.splitext(os.path.basename(p))[0]: p for p in glob.glob(os.path.join(hier.lib_dir, "*.pretty"))}
    errs, todo = [], []
    for sname, sh in sheets.items():
        for p in sh.parts:
            if not p.footprint:
                errs.append(f"{sname} {p.ref}: no footprint assigned")
                continue
            exists = symlib.footprint_exists(p.footprint, fp_dirs)
            if p.footprint_todo:
                if exists:
                    errs.append(f"{sname} {p.ref}: footprint {p.footprint} exists now; drop footprint_todo")
                else:
                    todo.append(f"{p.footprint}  ({sname} {p.ref} {p.value})")
            elif not exists:
                errs.append(f"{sname} {p.ref}: footprint {p.footprint} not found (mark footprint_todo if it "
                            "is a planned project footprint)")
    return errs, sorted(set(todo))


def _accepted(v: dict, accepted: list[spec.AcceptedErc]) -> spec.AcceptedErc | None:
    text = " ".join([v["sheet"], v["description"]] + v["items"])
    for a in accepted:
        if a.type == v["type"] and (not a.match or re.search(a.match, text)):
            return a
    return None


def _items_net_names(v: dict) -> str:
    return " ".join(v["items"])


def todo_instance_refs(hier: project.Hierarchy, sheets: dict[str, spec.Sheet]) -> set[str]:
    """References of footprint_todo parts as KiCad reports them: one per sheet instance (ref_step applied)."""
    steps = {e.name: [i * e.ref_step for i in range(len(e.instances))] for e in hier.sheets}
    refs = set()
    for name, sh in sheets.items():
        for p in sh.parts:
            if p.footprint_todo:
                refs.update(writer.offset_ref(p.ref, off) for off in steps.get(sh.name, steps.get(name, [0])))
    return refs


def check_erc(hier: project.Hierarchy, sheets: dict[str, spec.Sheet], out_dir: str) -> dict:
    """ERC errors fail the build, except those an `erc_accepted` entry with `until_sheet` covers
    while that sheet is still a placeholder (the driver of the net lives there). Warnings must be
    accepted explicitly; footprint-link warnings for parts marked footprint_todo are accepted
    automatically (they are listed in the footprints-to-create report)."""
    report = kicad.erc(hier.root_file, os.path.join(out_dir, "erc.json"))
    captured = {e.name for e in hier.sheets if e.spec is not None}
    accepted = []
    expired = []
    for sh in sheets.values():
        for a in sh.erc_accepted:
            if a.until_sheet and a.until_sheet in captured:
                expired.append(a)
            else:
                accepted.append(a)
    todo_refs = todo_instance_refs(hier, sheets)
    errors, err_ok, warn_ok, warn_bad = [], [], [], []
    for v in kicad.erc_violations(report):
        a = _accepted(v, accepted)
        if v["severity"] == "error":
            if a and a.until_sheet:
                err_ok.append((v, a))
            else:
                errors.append(v)
        elif v["severity"] == "warning":
            if not a and v["type"] == "footprint_link_issues":
                m = re.search(r"Symbol (\S+)", " ".join(v["items"]))
                if m and m.group(1) in todo_refs:
                    a = spec.AcceptedErc("footprint_link_issues", "", "footprint marked footprint_todo")
            (warn_ok if a else warn_bad).append((v, a))
    summary = {"errors": len(errors), "errors_accepted_until_driver_sheet": len(err_ok),
               "warnings_accepted": len(warn_ok), "warnings_unaccepted": len(warn_bad)}
    by_type: dict[str, int] = {}
    for v, _ in warn_ok + err_ok:
        by_type[v["type"]] = by_type.get(v["type"], 0) + 1
    summary["accepted_by_type"] = by_type
    summary["accepted_detail"] = [f"{v['severity']} [{v['type']}] {v['sheet']} {'; '.join(v['items'])} -- {a.reason}"
                                  for v, a in err_ok + warn_ok]
    for v in errors:
        _log(f"    ERC ERROR [{v['type']}] {v['description']} :: {v['sheet']} :: {'; '.join(v['items'])}")
    for v, _ in warn_bad:
        _log(f"    ERC WARNING (not accepted) [{v['type']}] {v['description']} :: {v['sheet']} :: "
             f"{'; '.join(v['items'])}")
    return summary


def check_netlist(hier: project.Hierarchy, sheets: dict[str, spec.Sheet], out_dir: str) -> tuple[list[str], int, int]:
    text = kicad.netlist(hier.root_file, os.path.join(out_dir, f"{hier.name}.net"))
    exp = netcheck.expected_for(hier.root_spec, None, 0)
    for e in hier.sheets:
        sh = sheets[e.name]
        for i, inst in enumerate(e.instances):
            hm = {k: project.pin_net(v, inst, i + 1) for k, v in e.pins.items()}
            netcheck.expected_for(sh, inst, i * e.ref_step, hm, exp)
    errs = netcheck.compare(exp, text)
    return errs, len(exp.nets), sum(len(v) for v in exp.nets.values())


# ------------------------------------------------------------------ DRC policy
# The boards carry placed parts but no routing (tools/pcbgen/populate.py). Every DRC violation must match one
# entry below or the build fails; "unconnected_items" (unrouted connections) is reported, never failed, until
# routing starts. Each entry: violation type, a regex over the description and both items (None = any), the
# scope ("same_footprint" = both items belong to one footprint; "any"), and why it is acceptable now.
# docs/capture-notes/layout-start.md §DRC explains each one. Remove an entry as soon as its cause is fixed.
DRC_ALWAYS_FAIL = {"courtyards_overlap", "malformed_courtyard", "missing_courtyard", "copper_edge_clearance",
                   "duplicate_footprints", "extra_footprint", "missing_footprint"}

DRC_REPORT_CAP, DRC_UNROUTED_CAP = 199, 499  # kicad-cli 10 stops listing a violation type at these counts

DRC_ACCEPTED = [
    ("silk_overlap", None, "any",
     "reference text of a first floor plan packed at 0.25 mm courtyard gaps; silk clean-up is fine placement"),
    ("silk_over_copper", None, "any",
     "reference text over neighbouring pads in the packed floor plan; silk clean-up is fine placement"),
    ("silk_edge_clearance", None, "any",
     "reference text clipped by the board edge (edge-launch SMA, parts packed next to the head's 1 mm edge "
     "margin); moved at fine placement"),
    ("clearance", r"\(netclass '[^']+' clearance", "same_footprint",
     "pad-to-pad gap inside one package is below the net-class routing clearance (fine-pitch IC); "
     "the class clearance is a routing rule, the package pitch is fixed by the part"),
    ("clearance", r"rule '(High-current clearance|PoE high voltage clearance|"
                  r"General clearance 0\.15 mm outside the BGA area)'", "same_footprint",
     "main board: pad gap inside one package (VBAT_SYS/VSOLAR_IN pins of eFuse/charger/buck QFNs at "
     "0.2-0.375 mm, stock PowerPAK/SON pads at 0.14 mm) is below a routing clearance rule; the package pitch "
     "is fixed by the part"),
    ("solder_mask_bridge", r"Rectangle of \S+ on F\.Mask", "same_footprint",
     "a footprint's own deliberate solder-mask window over pads of different nets: the stock Hirose U.FL land "
     "pattern, and the water-sense electrodes E1701/E9301 (bare ENIG, no mask by design, docs/15 §11)"),
    ("clearance", r"rule 'ISO barrier creepage'.*(<no net>|unconnected-\()", "same_footprint",
     "creepage rule against an unconnected pad of the isolator/magnetics package itself (NC pin, mechanical pad)"),
    ("clearance", r"rule 'ISO barrier creepage'.*\] of (C1404|C1405|C1409|R1404) ", "same_footprint",
     "head finding: barrier-spanning parts by design (2 kV Y-capacitors C1404/C1405 in 1808: pad gap below "
     "3 mm, needs a creepage-rated footprint or a slot; C1409 and the 0R bond R1404 join GND and CHASSIS, "
     "which are one potential since D-119, so their gap no longer matters)"),
]


def _same_footprint(items: list[str]) -> bool:
    refs = [m.group(1) for m in (re.search(r" of (\S+)", i) for i in items) if m]
    return len(items) == 2 and len(refs) == 2 and refs[0] == refs[1]


def classify_drc(violations: list[dict], accepted=None) -> dict:
    """Split DRC violations into failing, accepted (by entry) and unrouted. Pure; unit-tested."""
    accepted = DRC_ACCEPTED if accepted is None else accepted
    failing, unrouted = [], []
    by_entry: dict[int, list[dict]] = {}
    for v in violations:
        if v["group"] == "schematic_parity":
            continue  # judged by parity_failures()
        if v["group"] == "unconnected_items" or v["type"] == "unconnected_items":
            unrouted.append(v)
            continue
        if v["type"] in DRC_ALWAYS_FAIL:
            failing.append(v)
            continue
        text = " ".join([v["description"]] + v["items"])
        for k, (typ, rx, scope, _) in enumerate(accepted):
            if typ != v["type"] or (rx and not re.search(rx, text)):
                continue
            if scope == "same_footprint" and not _same_footprint(v["items"]):
                continue
            by_entry.setdefault(k, []).append(v)
            break
        else:
            failing.append(v)
    acc = [{"type": accepted[k][0], "match": accepted[k][1], "scope": accepted[k][2], "reason": accepted[k][3],
            "count": len(vs), "example": f"{vs[0]['description']} :: {'; '.join(vs[0]['items'])}"}
           for k, vs in sorted(by_entry.items())]
    return {"failing": failing, "accepted": acc, "unrouted": len(unrouted)}


def parity_failures(violations: list[dict], allowed_missing=frozenset()) -> list[dict]:
    """Schematic-parity issues that fail the build: every one, except `missing_footprint` for a reference in
    ``allowed_missing``. Since every footprint exists (D-116), the build passes no exceptions: a symbol without
    its footprint on the board, or a footprint that differs from its symbol, fails. Pure; unit-tested."""
    out = []
    for v in violations:
        if v["group"] != "schematic_parity":
            continue
        ref = v["description"].split()[2] if v["type"] == "missing_footprint" and len(v["description"].split()) > 2 else ""
        if ref and ref in allowed_missing:
            continue
        out.append(v)
    return out


def print_drc(res: dict) -> None:
    for v in res["failing"][:40]:
        _log(f"    DRC FAIL {v['severity']} [{v['type']}] {v['description']} :: {'; '.join(v['items'])}")
    if len(res["failing"]) > 40:
        _log(f"    ... {len(res['failing']) - 40} more failing violations")
    for a in res["accepted"]:
        _log(f"    DRC accepted {a['count']:4d} x [{a['type']}] {a['scope']}: {a['reason'][:90]}")
    _log(f"    DRC failing {len(res['failing'])}, accepted {sum(a['count'] for a in res['accepted'])}, "
         f"unrouted connections {res['unrouted']} (reported only until routing starts)")
    capped = [a["type"] for a in res["accepted"] if a["count"] >= DRC_REPORT_CAP] + (
        ["unconnected_items"] if res["unrouted"] >= DRC_UNROUTED_CAP else [])
    if capped:
        _log(f"    note: kicad-cli reports at most {DRC_REPORT_CAP} violations per type ({DRC_UNROUTED_CAP} unrouted); "
             f"true counts are higher for: {', '.join(capped)}")


def build_board(board: str, libs, args, results: dict) -> list[str]:
    bdir = os.path.join(REPO, "hardware", board)
    hpath = os.path.join(bdir, "hierarchy.toml")
    if not os.path.exists(hpath):
        _log(f"[{board}] no hierarchy.toml, skipped")
        return []
    _log(f"[{board}] generating")
    hier = project.load_hierarchy(hpath)
    sheets = project.generate(hier, libs, log=_log)
    project.write_project_file(hier)
    project.write_lib_tables(hier)
    failures: list[str] = []
    res = results.setdefault(board, {})

    fp_errs, todo = footprint_report(hier, sheets)
    failures += fp_errs
    res["footprints_needing_creation"] = todo
    for e in fp_errs:
        _log(f"    FOOTPRINT: {e}")

    if args.no_kicad:
        return failures
    out_dir = os.path.join(REPO, "build", board)
    os.makedirs(out_dir, exist_ok=True)

    _log(f"[{board}] ERC")
    erc = check_erc(hier, sheets, out_dir)
    res["erc"] = erc
    _log(f"    errors {erc['errors']} (+{erc['errors_accepted_until_driver_sheet']} accepted until the driver sheet "
         f"is captured), warnings accepted {erc['warnings_accepted']}, warnings unaccepted "
         f"{erc['warnings_unaccepted']}; accepted by type {erc['accepted_by_type']}")
    if erc["errors"] or erc["warnings_unaccepted"]:
        failures.append(f"{board}: ERC has {erc['errors']} error(s) and {erc['warnings_unaccepted']} unaccepted warning(s)")

    _log(f"[{board}] netlist round trip")
    nerrs, nnets, nnodes = check_netlist(hier, sheets, out_dir)
    res["netlist"] = {"nets_checked": nnets, "pins_checked": nnodes, "mismatches": len(nerrs)}
    for e in nerrs:
        _log(f"    NETLIST: {e}")
    if nerrs:
        failures.append(f"{board}: netlist round trip found {len(nerrs)} mismatch(es)")
    else:
        _log(f"    OK: {nnets} nets, {nnodes} pin connections match the specs exactly")

    if not args.no_pdf:
        pdf_dir = os.path.join(bdir, "pdf")
        os.makedirs(pdf_dir, exist_ok=True)
        out = os.path.join(pdf_dir, f"{board}.pdf")
        kicad.pdf(hier.root_file, out)
        _log(f"[{board}] PDF {os.path.relpath(out, REPO)}")
        captured = [page for e, _, _, page in hier.instances() if e.spec is not None]
        if captured:
            out2 = os.path.join(pdf_dir, f"{board}-captured-sheets.pdf")
            kicad.pdf(hier.root_file, out2, ["1"] + captured)
            _log(f"[{board}] PDF {os.path.relpath(out2, REPO)} (root + captured sheets, pages {', '.join(captured)})")

    pcb = os.path.join(bdir, f"{board}.kicad_pcb")
    if not os.path.exists(pcb):
        # first build from a clean checkout of a board without a layout yet: create it once.
        # Never regenerated afterwards, because layout work lives in this file.
        _log(f"[{board}] creating {os.path.relpath(pcb, REPO)} (pcbnew API) and its stackup")
        r = subprocess.run(kicad.py_cmd() + [os.path.join(REPO, "tools", "pcbgen", "make_board.py"), board],
                           capture_output=True, text=True, timeout=600)
        if r.returncode != 0:
            failures.append(f"{board}: make_board.py failed:\n{r.stdout}\n{r.stderr}")
            return failures
        sys.path.insert(0, os.path.join(REPO, "tools", "pcbgen"))
        import stackup  # noqa: E402  (tools/pcbgen/stackup.py, stdlib only)
        stackup.apply(board)
    if os.path.exists(pcb):
        _log(f"[{board}] DRC")
        rep = kicad.drc(pcb, os.path.join(out_dir, "drc.json"), parity=True)
        res_drc = classify_drc(kicad.drc_violations(rep))
        par = parity_failures(kicad.drc_violations(rep))
        for v in par[:20]:
            _log(f"    PARITY FAIL [{v['type']}] {v['description']} :: {'; '.join(v['items'])}")
        _log(f"    schematic parity: {len(par)} issue(s) (footprints vs schematic; any issue fails)")
        if par:
            failures.append(f"{board}: {len(par)} schematic parity issue(s): the PCB does not match the schematic "
                            "(run python3 -m tools.pcbgen.populate)")
        res["drc"] = {"failing": len(res_drc["failing"]), "parity_issues": len(par), "unrouted_connections": res_drc["unrouted"],
                      "accepted": [{k: a[k] for k in ("type", "scope", "count", "reason")} for a in res_drc["accepted"]]}
        print_drc(res_drc)
        if res_drc["failing"]:
            failures.append(f"{board}: DRC has {len(res_drc['failing'])} violation(s) outside the accepted list")
    return failures


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--no-kicad", action="store_true", help="generate only; skip kicad-cli checks")
    ap.add_argument("--no-pdf", action="store_true", help="skip PDF export")
    ap.add_argument("--board", action="append", help="limit to these boards")
    args = ap.parse_args(argv)

    lib_dir = os.path.join(REPO, "hardware", "lib")
    failures: list[str] = []
    _log("[lib] project symbol libraries")
    made = build_symbol_libs(lib_dir)
    libs = symlib.SymbolLibraries.default(lib_dir)
    for lib in made:
        failures += symgen.lint_library_fields(libs, lib)
    results: dict = {}
    for board in args.board or BOARDS:
        try:
            failures += build_board(board, libs, args, results)
        except (spec.SpecError, symgen.SymbolSpecError, RuntimeError) as exc:
            failures.append(f"{board}: {exc}")
    os.makedirs(os.path.join(REPO, "build"), exist_ok=True)
    with open(os.path.join(REPO, "build", "results.json"), "w", encoding="utf-8") as fh:
        json.dump(results, fh, indent=2)
    if failures:
        _log("\nBUILD FAILED:")
        for f in failures:
            _log(f"  - {f}")
        return 1
    _log("\nBUILD OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
