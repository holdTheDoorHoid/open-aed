"""Regenerate every board's schematics and run all checks.

    python3 -m tools.schgen.build            (from the repo root; tools/build.sh wraps this)

Exit status is non-zero if anything fails: spec validation, pin-name cross-check,
footprint resolution, ERC errors or unaccepted ERC warnings, the netlist
round-trip, or PDF export.

Open AED: one KiCad project, hardware/aed-main (docs/HARDWARE_WORKFLOW.md). PCB generation and DRC are out of
scope until layout starts; the board file is not created here.
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import sys

from . import kicad, netcheck, project, spec, symgen, symlib, writer

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BOARDS = ["aed-main"]


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


def check_notice(hier: project.Hierarchy) -> list[str]:
    """Every generated schematic file must carry the project notice (CLAUDE.md rule 2, SAFETY.md §2a).
    A project without a notice in hierarchy.toml fails too: the notice is mandatory for this repository."""
    if not hier.notice:
        return [f"{hier.name}: hierarchy.toml [project] has no `notice` (must be the NOT FOR CONSTRUCTION line)"]
    if "NOT FOR CONSTRUCTION" not in hier.notice:
        return [f"{hier.name}: [project] notice must contain 'NOT FOR CONSTRUCTION'"]
    errs = []
    files = [hier.root_file] + [os.path.join(hier.dir, e.file) for e in hier.sheets]
    for f in files:
        with open(f, encoding="utf-8") as fh:
            if hier.notice not in fh.read():
                errs.append(f"{os.path.relpath(f, REPO)}: notice missing from the title block")
    return errs


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

    notice_errs = check_notice(hier)
    failures += notice_errs
    for e in notice_errs:
        _log(f"    NOTICE: {e}")

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
