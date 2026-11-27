"""Generate project symbol libraries (.kicad_sym) from readable TOML pin lists.

Source files live in ``hardware/lib/src/<lib>.toml``; output goes to
``hardware/lib/<lib>.kicad_sym``. Every symbol gets the fields the project
requires of a custom symbol: Datasheet, MPN, Manufacturer and PinoutVerified.
"""

from __future__ import annotations

import math
import tomllib

from .sexpr import S, dumps

PIN_TYPES = {"input", "output", "bidirectional", "tri_state", "passive", "free", "unspecified", "power_in",
             "power_out", "open_collector", "open_emitter", "no_connect"}
SIDES = ("left", "right", "top", "bottom")
REQUIRED_FIELDS = ("Datasheet", "MPN", "Manufacturer", "PinoutVerified")
PITCH = 2.54
PIN_LEN = 2.54


class SymbolSpecError(Exception):
    pass


def _r(v: float) -> float:
    return round(v, 4)


def _ceil_grid(v: float) -> float:
    return math.ceil(v / PITCH - 1e-9) * PITCH


def _prop(name, value, x, y, hide=False, justify=None):
    node = [S("property"), name, value, [S("at"), x, y, 0]]
    if hide:
        node.append([S("hide"), S("yes")])
    eff = [S("effects"), [S("font"), [S("size"), 1.27, 1.27]]]
    if justify:
        eff.append([S("justify"), S(justify)])
    node += [[S("show_name"), S("no")], [S("do_not_autoplace"), S("no")], eff]
    return node


def parse_pins(text: str, where: str) -> list[tuple[str, str, str, str]]:
    rows = []
    for raw in text.splitlines():
        line = raw.partition("#")[0].strip()
        if not line:
            continue
        cols = line.split()
        if len(cols) != 4:
            raise SymbolSpecError(f"{where}: pin line must be 'number name type side': {raw.strip()!r}")
        num, name, ptype, side = cols
        if ptype not in PIN_TYPES:
            raise SymbolSpecError(f"{where}: pin {num}: unknown type {ptype}")
        if side not in SIDES:
            raise SymbolSpecError(f"{where}: pin {num}: side must be one of {SIDES}")
        rows.append((num, name, ptype, side))
    nums = [r[0] for r in rows]
    if len(set(nums)) != len(nums):
        raise SymbolSpecError(f"{where}: duplicate pin numbers")
    return rows


def build_symbol(d: dict) -> list:
    name = d["name"]
    where = f"symbol {name}"
    for k in ("datasheet", "mpn", "manufacturer", "pinout_source", "footprint", "reference"):
        if not d.get(k):
            raise SymbolSpecError(f"{where}: missing '{k}'")
    pins = parse_pins(d["pins"], where)
    by_side = {s: [p for p in pins if p[3] == s] for s in SIDES}
    side_pitch = {"left": PITCH, "right": PITCH, "top": d.get("top_pitch", 7.62), "bottom": d.get("bottom_pitch", PITCH)}
    cw = 1.27 * 0.75
    lw = max([len(p[1]) for p in by_side["left"]] + [0]) * cw
    rw = max([len(p[1]) for p in by_side["right"]] + [0]) * cw
    span_t = (len(by_side["top"]) - 1) * side_pitch["top"] if by_side["top"] else 0
    span_b = (len(by_side["bottom"]) - 1) * side_pitch["bottom"] if by_side["bottom"] else 0
    width = max(_ceil_grid(lw + rw + 2.54), _ceil_grid(max(span_t, span_b) + 5.08), d.get("min_width", 10.16))
    if (width / PITCH) % 2:
        width += PITCH  # keep the half-width on the grid
    rows = max(len(by_side["left"]), len(by_side["right"]), 1)
    band_t = _ceil_grid(max([len(p[1]) for p in by_side["top"]] + [0]) * cw + 1.27) if by_side["top"] else 0.0
    band_b = _ceil_grid(max([len(p[1]) for p in by_side["bottom"]] + [0]) * cw + 1.27) if by_side["bottom"] else 0.0
    mid = (rows + 1) * PITCH
    # side pins sit in the middle band; top/bottom pin names get their own bands
    half = _ceil_grid(mid / 2 + max(band_t, band_b))
    height = 2 * half
    hw, hh = width / 2, height / 2
    y_mid = 0.0

    graphics = [S("symbol"), f"{name}_0_1",
                [S("rectangle"), [S("start"), _r(-hw), _r(hh)], [S("end"), _r(hw), _r(-hh)],
                 [S("stroke"), [S("width"), 0.254], [S("type"), S("default")]],
                 [S("fill"), [S("type"), S("background")]]]]
    unit = [S("symbol"), f"{name}_1_1"]

    def pin(num, pname, ptype, x, y, ang):
        return [S("pin"), S(ptype), S("line"), [S("at"), _r(x), _r(y), ang], [S("length"), PIN_LEN],
                [S("name"), pname, [S("effects"), [S("font"), [S("size"), 1.27, 1.27]]]],
                [S("number"), num, [S("effects"), [S("font"), [S("size"), 1.27, 1.27]]]]]

    for side in SIDES:
        plist = by_side[side]
        n = len(plist)
        for i, (num, pname, ptype, _) in enumerate(plist):
            pitch = side_pitch[side]
            if side in ("left", "right"):
                y = _r(y_mid + (n - 1) / 2 * PITCH - i * PITCH)
                y = round(y / 1.27) * 1.27
                x = -hw - PIN_LEN if side == "left" else hw + PIN_LEN
                unit.append(pin(num, pname, ptype, x, y, 0 if side == "left" else 180))
            else:
                x = _r(-(n - 1) / 2 * pitch + i * pitch)
                x = round(x / 1.27) * 1.27
                y = hh + PIN_LEN if side == "top" else -hh - PIN_LEN
                unit.append(pin(num, pname, ptype, x, y, 270 if side == "top" else 90))

    verified = d.get("pinout_verified") or f"NO - drafted from datasheet {d['pinout_source']}, needs human check"
    props = [
        _prop("Reference", d["reference"], _r(-hw), _r(hh + 1.27), justify="left"),
        _prop("Value", d.get("value", name), _r(-hw), _r(-hh - 1.27), justify="left"),
        _prop("Footprint", d["footprint"], 0, 0, hide=True),
        _prop("Datasheet", d["datasheet"], 0, 0, hide=True),
        _prop("Description", d.get("description", ""), 0, 0, hide=True),
        _prop("MPN", d["mpn"], 0, 0, hide=True),
        _prop("Manufacturer", d["manufacturer"], 0, 0, hide=True),
        _prop("PinoutVerified", verified, 0, 0, hide=True),
        # same convention as tools/libcheck.py (sibling library work): pad count of the package
        _prop("PackagePins", str(d.get("package_pins", len(pins))), 0, 0, hide=True),
    ]
    if d.get("footprint_verified"):
        props.append(_prop("FootprintVerified", d["footprint_verified"], 0, 0, hide=True))
    if d.get("keywords"):
        props.append(_prop("ki_keywords", d["keywords"], 0, 0, hide=True))
    node = [S("symbol"), name, [S("pin_names"), [S("offset"), 1.016]], [S("exclude_from_sim"), S("no")],
            [S("in_bom"), S("yes")], [S("on_board"), S("yes")], [S("in_pos_files"), S("yes")],
            [S("duplicate_pin_numbers_are_jumpers"), S("no")], *props, graphics, unit,
            [S("embedded_fonts"), S("no")]]
    return node


def build_library(path_toml: str) -> tuple[str, list]:
    with open(path_toml, "rb") as fh:
        data = tomllib.load(fh)
    lib = data["library"]["name"]
    out = [S("kicad_symbol_lib"), [S("version"), S("20251024")], [S("generator"), "ws_symgen"],
           [S("generator_version"), "1.0"]]
    names = set()
    for d in data.get("symbol", []):
        if d["name"] in names:
            raise SymbolSpecError(f"{path_toml}: duplicate symbol {d['name']}")
        names.add(d["name"])
        out.append(build_symbol(d))
    return lib, out


def write_library(path_toml: str, out_path: str) -> str:
    lib, node = build_library(path_toml)
    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write(dumps(node) + "\n")
    return lib


def lint_library_fields(libs, lib_name: str) -> list[str]:
    """Every symbol of a project library must carry the required fields, non-empty."""
    errs = []
    from .sexpr import find_all
    root = libs._file(lib_name)
    for s in find_all(root, "symbol"):
        props = {p[1]: p[2] for p in find_all(s, "property")}
        for f in REQUIRED_FIELDS:
            if not props.get(f):
                errs.append(f"{lib_name}:{s[1]}: missing or empty field {f}")
    return errs
