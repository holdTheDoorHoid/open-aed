"""Project-level generation: hierarchy, root sheet, placeholder sheets, .kicad_pro, lib tables."""

from __future__ import annotations

import glob
import re
import json
import os
import tomllib
from dataclasses import dataclass, field

from . import placement, spec, writer
from .sexpr import S
from .writer import Instance, uid

# ----------------------------------------------------------------- hierarchy


@dataclass
class SheetEntry:
    name: str  # file stem
    title: str
    doc: str
    instances: list[str]
    ref_step: int = 0
    note: str = ""
    owner: str = ""  # capture work package (hierarchy.toml `owner`), shown on placeholder pages
    pins: dict[str, str | list[str]] = field(default_factory=dict)  # hier label -> root net template(s), see pin_net()
    spec_path: str | None = None  # set when a spec exists
    spec: spec.Sheet | None = None

    @property
    def file(self) -> str:
        return f"{self.name}.kicad_sch"


@dataclass
class Hierarchy:
    dir: str
    name: str
    title: str
    revision: str
    date: str
    root_spec: spec.Sheet
    sheets: list[SheetEntry]
    lib_dir: str
    net_classes: list[dict]
    netclass_patterns: list[dict]
    rules: dict
    root_uuid: str
    notice: str = ""  # e.g. "NOT FOR CONSTRUCTION — unreviewed design": first title-block line of every sheet
    company: str = ""

    @property
    def root_file(self) -> str:
        return os.path.join(self.dir, f"{self.name}.kicad_sch")

    def instance_uuid(self, inst: str) -> str:
        return uid(self.name, "sheet-instance", inst)

    def instances(self):
        """Yield (entry, index, instance_name, page) in page order (root is page 1)."""
        page = 2
        for e in self.sheets:
            for i, inst in enumerate(e.instances):
                yield e, i, inst, str(page)
                page += 1


def load_hierarchy(path: str) -> Hierarchy:
    base = os.path.dirname(os.path.abspath(path))
    with open(path, "rb") as fh:
        data = tomllib.load(fh)
    pj = data["project"]
    name = pj["name"]
    root_spec = spec.load(os.path.join(base, pj["root_spec"]))
    sheets = []
    for s in data.get("sheet", []):
        e = SheetEntry(
            name=s["name"], title=s["title"], doc=s.get("doc", ""), instances=list(s.get("instances", [s["name"]])),
            ref_step=int(s.get("ref_step", 0)), note=s.get("note", ""), pins=dict(s.get("pins", {})),
            owner=s.get("owner", ""),
        )
        sp = os.path.join(base, "specs", f"{e.name}.toml")
        if os.path.exists(sp):
            e.spec_path = sp
            e.spec = spec.load(sp)
            if e.spec.name != e.name:
                raise spec.SpecError(f"{sp}: sheet.name '{e.spec.name}' must equal the file stem '{e.name}'")
        for pin, tmpl in e.pins.items():
            if isinstance(tmpl, list) and len(tmpl) != len(e.instances):
                raise spec.SpecError(f"{e.name}: pins.{pin} lists {len(tmpl)} nets for {len(e.instances)} instances")
        if len(e.instances) > 1 and not e.ref_step and e.spec and e.spec.parts:
            raise spec.SpecError(f"{e.name}: multi-instance sheet needs ref_step")
        sheets.append(e)
    names = [i for e in sheets for i in e.instances]
    if len(set(names)) != len(names):
        raise spec.SpecError("duplicate sheet instance names in hierarchy")
    return Hierarchy(
        dir=base, name=name, title=pj["title"], revision=pj.get("revision", "A-draft"),
        date=pj.get("date", "2026-11-22"), root_spec=root_spec, sheets=sheets,
        lib_dir=os.path.normpath(os.path.join(base, pj.get("lib_dir", "../lib"))),
        net_classes=data.get("net_class", []), netclass_patterns=data.get("netclass_pattern", []),
        rules=data.get("rules", {}), root_uuid=uid(name, "root"),
        notice=pj.get("notice", ""), company=pj.get("company", ""),
    )


# --------------------------------------------------------------- placeholders


def placeholder_spec(e: SheetEntry, hier: Hierarchy) -> spec.Sheet:
    note = (
        (f"{hier.notice}\n" if hier.notice else "")
        + f"PLACEHOLDER — this block is not captured yet.\n"
        f"Design source: {e.doc}.\n"
        f"Instances: {', '.join(e.instances)}.\n"
        f"{e.note}\n"
        f"When it is captured, add hardware/{os.path.basename(hier.dir)}/specs/" + e.name + ".toml and run tools/build.sh; "
        "the generator then replaces this page."
    )
    sh = spec.Sheet(name=e.name, title=e.title, source=e.doc, revision=hier.revision, date=hier.date,
                    comments=[f"Capture owner: {e.owner}"] if getattr(e, "owner", "") else [])
    sh.groups.append(spec.Group(id="placeholder", title=f"{e.name}: {e.title}", note=note))
    return sh


# ---------------------------------------------------------------- generation


def _stamp(sh: spec.Sheet, hier: Hierarchy) -> None:
    """Project-wide title-block text: the notice line and the company field."""
    if hier.notice and not sh.notice:
        sh.notice = hier.notice
    if hier.company and not sh.company:
        sh.company = hier.company


def pin_net(tmpl: str | list[str], inst: str, idx: int) -> str:
    """Root net for one sheet pin of one instance.

    Template fields: {inst} instance name (CELL_VHF1), {i} 1-based index in the instance list, {i0} 0-based index,
    {tag} the name after its first underscore (VHF1), {band} the letters of the tag (VHF), {n} its trailing digits
    (1, empty if none: CELL_TONE -> tag TONE, band TONE, n "").
    A list gives one template per instance, in instance order (for numbering that does not follow the sheet's own
    instance index, e.g. CELL_FITTED6..11 on the UHF cell sheet).
    """
    if isinstance(tmpl, list):
        tmpl = tmpl[idx - 1]
    tag = inst.split("_", 1)[1] if "_" in inst else inst
    m = re.match(r"^(.*?)(\d*)$", tag)
    return tmpl.format(inst=inst, i=idx, i0=idx - 1, tag=tag, band=m.group(1), n=m.group(2))


def _sheet_symbol(hier: Hierarchy, e: SheetEntry, inst: str, page: str, x: float, y: float,
                  w: float, h: float) -> tuple[list, list]:
    """Sheet symbol for one instance, plus the root-side wires/labels for its pins."""
    su = hier.instance_uuid(inst)
    node = [S("sheet"), [S("at"), x, y], [S("size"), w, h], [S("exclude_from_sim"), S("no")],
            [S("in_bom"), S("yes")], [S("on_board"), S("yes")], [S("dnp"), S("no")],
            [S("fields_autoplaced"), S("yes")], [S("stroke"), [S("width"), 0.1524], [S("type"), S("solid")]],
            [S("fill"), [S("color"), 0, 0, 0, 0.0]], [S("uuid"), su],
            [S("property"), "Sheetname", inst, [S("at"), x, placement.rnd(y - 0.7116), 0],
             [S("show_name"), S("no")], [S("do_not_autoplace"), S("no")],
             writer.effects(1.27, ["left", "bottom"])],
            [S("property"), "Sheetfile", e.file, [S("at"), x, placement.rnd(y + h + 0.5884), 0],
             [S("show_name"), S("no")], [S("do_not_autoplace"), S("no")],
             writer.effects(1.27, ["left", "top"])]]
    extra = []
    idx = e.instances.index(inst) + 1
    for k, (pin, tmpl) in enumerate(sorted(e.pins.items())):
        py = placement.rnd(y + 2.54 * (k + 1))
        node.append([S("pin"), pin, S("bidirectional"), [S("at"), x, py, 180],
                     [S("uuid"), uid(hier.name, "sheetpin", inst, pin)], writer.effects(1.27, ["left"])])
        net = pin_net(tmpl, inst, idx)
        extra.append([S("wire"), [S("pts"), [S("xy"), x, py], [S("xy"), placement.rnd(x - 2.54), py]],
                      [S("stroke"), [S("width"), 0], [S("type"), S("solid")]],
                      [S("uuid"), uid(hier.name, "sheetpin-wire", inst, pin)]])
        extra.append([S("global_label"), net, [S("shape"), S("passive")], [S("at"), placement.rnd(x - 2.54), py, 180],
                      [S("fields_autoplaced"), S("yes")], writer.effects(1.27, ["right"]),
                      [S("uuid"), uid(hier.name, "sheetpin-label", inst, pin)]])
    node.append([S("instances"), [S("project"), hier.name, [S("path"), f"/{hier.root_uuid}", [S("page"), page]]]])
    return node, extra


def generate(hier: Hierarchy, libs, log=print) -> dict[str, spec.Sheet]:
    """Write every sheet file and the root. Returns {file stem: Sheet} of what was written."""
    written: dict[str, spec.Sheet] = {}
    project_title = hier.title
    pages = {inst: page for _, _, inst, page in hier.instances()}

    # 0. KiCad 10 ERC reports two PWR_FLAGs on one net as "power output connected to power output",
    #    so every rail may be flagged on exactly one sheet instance in the whole project.
    flagged: dict[str, list[str]] = {}
    for n in hier.root_spec.power_flags:
        flagged.setdefault(n, []).append("root")
    for e in hier.sheets:
        if e.spec:
            for n in e.spec.power_flags:
                for inst in e.instances:
                    flagged.setdefault(n, []).append(inst)
    dup = {n: w for n, w in flagged.items() if len(w) > 1}
    if dup:
        raise spec.SpecError("rail flagged more than once (one PWR_FLAG per rail per project; a flag in a "
                             f"multi-instance sheet counts once per instance): {dup}")

    # 0b. a multi-unit part may have its units on several sheets (one reference, disjoint units, all covered)
    errs = spec.check_split_parts([(hier.root_spec, 1)] + [(e.spec, len(e.instances)) for e in hier.sheets if e.spec],
                                  libs)
    if errs:
        raise spec.SpecError("parts split across sheets:\n  " + "\n  ".join(errs))

    # 1. child sheets (one file per sheet family; instance paths for every instance)
    for idx, e in enumerate(hier.sheets):
        sh = e.spec or placeholder_spec(e, hier)
        sh.revision = sh.revision or hier.revision
        _stamp(sh, hier)
        errs = spec.check_against_symbols(sh, libs)
        if errs:
            raise spec.SpecError(f"{e.name}:\n  " + "\n  ".join(errs))
        if len(e.instances) == 1 and sh.hier_nets:
            missing = set(sh.hier_nets) - set(e.pins)
            if missing:
                raise spec.SpecError(f"{e.name}: hierarchical nets without a root mapping: {sorted(missing)}")
        lay = placement.layout(sh, libs, power_ref_base=(idx + 1) * 1000)
        insts = [Instance(f"/{hier.root_uuid}/{hier.instance_uuid(inst)}", i * e.ref_step, pages[inst])
                 for i, inst in enumerate(e.instances)]
        node = writer.build(sh, lay, libs, insts, hier.name, project_title, uid(hier.name, "file", e.name))
        writer.write(os.path.join(hier.dir, e.file), node)
        written[e.name] = sh
        log(f"  wrote {e.file} ({'spec' if e.spec else 'placeholder'}, {len(sh.parts)} parts, "
            f"{len(e.instances)} instance(s), paper {lay.paper})")

    # 2. root: its own spec content, then the sheet symbols
    rs = hier.root_spec
    _stamp(rs, hier)
    errs = spec.check_against_symbols(rs, libs)
    if errs:
        raise spec.SpecError("root:\n  " + "\n  ".join(errs))
    lay = placement.layout(rs, libs, power_ref_base=1)
    top = max(lay.bottom, 20.0) + 15.24
    x0 = placement.PAGE_MARGIN
    col_w, gap = 55.88, 12.7
    page_w = max(lay.width, 841.0)
    per_row = max(1, int((page_w - 2 * x0 + gap) // (col_w + gap)))
    items = []
    k = 0
    row_y = top
    row_h = 0.0
    for e in hier.sheets:
        for inst in e.instances:
            if k and k % per_row == 0:
                row_y += row_h + 12.7
                row_h = 0.0
            x = placement.snap(x0 + (k % per_row) * (col_w + gap) + 15.24)
            h = placement.snap(max(10.16, 2.54 * (len(e.pins) + 2)))
            sym, extra = _sheet_symbol(hier, e, inst, pages[inst], x, placement.snap(row_y), col_w - 15.24, h)
            items.append(sym)
            items += extra
            row_h = max(row_h, h + 5.08)
            k += 1
    bottom = row_y + row_h + 40.0
    for name, pw, ph in placement.PAPERS:
        if pw >= page_w and ph >= bottom:
            lay.paper, lay.width, lay.height = name, pw, ph
            break
    else:
        lay.paper, lay.width, lay.height = "User", page_w, placement.snap(bottom, 10.0)
    rs.revision = rs.revision or hier.revision
    node = writer.build(rs, lay, libs, [Instance(f"/{hier.root_uuid}", 0, "1")], hier.name, project_title,
                        hier.root_uuid, is_root=True, extra_items=items)
    writer.write(hier.root_file, node)
    written["<root>"] = rs
    log(f"  wrote {os.path.basename(hier.root_file)} (root, {len(rs.parts)} parts, {k} sheet instances)")
    return written


# ------------------------------------------------------------ project files


def write_lib_tables(hier: Hierarchy) -> None:
    rel = os.path.relpath(hier.lib_dir, hier.dir)
    syms = sorted(glob.glob(os.path.join(hier.lib_dir, "*.kicad_sym")))
    fps = sorted(glob.glob(os.path.join(hier.lib_dir, "*.pretty")))
    lines = ["(sym_lib_table", "\t(version 7)"]
    for p in syms:
        nick = os.path.splitext(os.path.basename(p))[0]
        lines.append(f'\t(lib (name "{nick}") (type "KiCad") (uri "${{KIPRJMOD}}/{rel}/{os.path.basename(p)}") '
                     f'(options "") (descr "Open AED project library"))')
    lines.append(")")
    with open(os.path.join(hier.dir, "sym-lib-table"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    lines = ["(fp_lib_table", "\t(version 7)"]
    for p in fps:
        nick = os.path.splitext(os.path.basename(p))[0]
        lines.append(f'\t(lib (name "{nick}") (type "KiCad") (uri "${{KIPRJMOD}}/{rel}/{os.path.basename(p)}") '
                     f'(options "") (descr "Open AED project footprints"))')
    lines.append(")")
    with open(os.path.join(hier.dir, "fp-lib-table"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")


def _netclass(d: dict) -> dict:
    base = {
        "bus_width": 12, "clearance": 0.15, "diff_pair_gap": 0.15, "diff_pair_via_gap": 0.25,
        "diff_pair_width": 0.15, "line_style": 0, "microvia_diameter": 0.3, "microvia_drill": 0.1,
        "pcb_color": "rgba(0, 0, 0, 0.000)", "priority": 0, "schematic_color": "rgba(0, 0, 0, 0.000)",
        "track_width": 0.15, "via_diameter": 0.6, "via_drill": 0.3, "wire_width": 6,
    }
    out = dict(base)
    for k, v in d.items():
        if k in ("note",):
            continue
        out[k] = v
    return out


def write_project_file(hier: Hierarchy) -> None:
    path = os.path.join(hier.dir, f"{hier.name}.kicad_pro")
    classes = [_netclass(c) for c in hier.net_classes]
    for i, c in enumerate(classes):
        c["priority"] = 2147483647 if c["name"] == "Default" else i
    sheets = [[hier.root_uuid, "Root"]] + [[hier.instance_uuid(inst), inst] for _, _, inst, _ in hier.instances()]
    r = hier.rules
    pro = {
        "board": {
            "design_settings": {
                "defaults": {
                    "board_outline_line_width": 0.1, "copper_line_width": 0.2, "copper_text_size_h": 1.5,
                    "copper_text_size_v": 1.5, "copper_text_thickness": 0.3, "silk_line_width": 0.15,
                    "silk_text_size_h": 1.0, "silk_text_size_v": 1.0, "silk_text_thickness": 0.15,
                    "pads": {"drill": 0.0, "height": 0.6, "width": 0.6},
                },
                "meta": {"version": 2},
                "rules": {
                    "allow_blind_buried_vias": False, "allow_microvias": False, "max_error": 0.005,
                    "min_clearance": r.get("min_clearance", 0.1),
                    "min_connection": 0.0,
                    "min_copper_edge_clearance": r.get("min_copper_edge_clearance", 0.5),
                    "min_hole_clearance": r.get("min_hole_clearance", 0.2),
                    "min_hole_to_hole": r.get("min_hole_to_hole", 0.25),
                    "min_microvia_diameter": 0.2, "min_microvia_drill": 0.1, "min_resolved_spokes": 2,
                    "min_silk_clearance": 0.0,
                    "min_text_height": r.get("min_text_height", 0.8),
                    "min_text_thickness": r.get("min_text_thickness", 0.15),
                    "min_through_hole_diameter": r.get("min_through_hole_diameter", 0.2),
                    "min_track_width": r.get("min_track_width", 0.1),
                    "min_via_annular_width": r.get("min_via_annular_width", 0.125),
                    "min_via_diameter": r.get("min_via_diameter", 0.45),
                    "solder_mask_to_copper_clearance": 0.0, "use_height_for_length_calcs": True,
                },
                "track_widths": [0.0] + r.get("track_widths", []),
                "via_dimensions": [{"diameter": 0.0, "drill": 0.0}]
                + [{"diameter": v[0], "drill": v[1]} for v in r.get("vias", [])],
                "diff_pair_dimensions": [{"gap": 0.0, "via_gap": 0.0, "width": 0.0}],
            },
            "ipc2581": {}, "layer_pairs": [], "layer_presets": [], "viewports": [],
        },
        "boards": [],
        "cvpcb": {"equivalence_files": []},
        "erc": {"erc_exclusions": [], "meta": {"version": 0}, "pin_map": [], "rule_severities": {}},
        "libraries": {"pinned_footprint_libs": [], "pinned_symbol_libs": []},
        "meta": {"filename": os.path.basename(path), "version": 3},
        "net_settings": {
            "classes": classes, "meta": {"version": 4}, "net_colors": None, "netclass_assignments": None,
            "netclass_patterns": [{"netclass": p["netclass"], "pattern": p["pattern"]} for p in hier.netclass_patterns],
        },
        "pcbnew": {"last_paths": {}, "page_layout_descr_file": ""},
        "schematic": {"annotate_start_num": 0, "connection_grid_size": 50.0, "meta": {"version": 1},
                      "net_format_name": "", "page_layout_descr_file": "", "plot_directory": "",
                      "subpart_first_id": 65, "subpart_id_separator": 0},
        "sheets": sheets,
        "text_variables": {},
    }
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(pro, fh, indent=2)
        fh.write("\n")
