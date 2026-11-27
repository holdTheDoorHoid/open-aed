"""Symbol-library access: load .kicad_sym files, flatten derived symbols, list pins.

Coordinates returned here are *library* coordinates in millimetres (y up), exactly
as written in the .kicad_sym file. :mod:`placement` converts them to schematic
coordinates (y down).
"""

from __future__ import annotations

import copy
import glob
import os
from dataclasses import dataclass, field

from . import sexpr
from .sexpr import Sym, find, find_all

STOCK_SYMBOL_GLOB = (
    "/home/hoid/.local/share/flatpak/runtime/org.kicad.KiCad.Library.Symbols/"
    "x86_64/stable/active/files/symbols"
)
STOCK_FOOTPRINT_DIR = (
    "/home/hoid/.local/share/flatpak/runtime/org.kicad.KiCad.Library.Footprints/"
    "x86_64/stable/active/files/footprints"
)


def stock_symbol_dir() -> str:
    env = os.environ.get("WS_KICAD_SYMBOL_DIR")
    if env:
        return env
    hits = glob.glob(STOCK_SYMBOL_GLOB)
    if not hits:
        raise FileNotFoundError(
            "KiCad stock symbol library not found; set WS_KICAD_SYMBOL_DIR "
            f"(looked for {STOCK_SYMBOL_GLOB})"
        )
    return hits[0]


def stock_footprint_dir() -> str:
    return os.environ.get("WS_KICAD_FOOTPRINT_DIR", STOCK_FOOTPRINT_DIR)


@dataclass
class Pin:
    number: str
    name: str
    etype: str  # power_in, input, passive ...
    x: float  # connection point, library coords (mm, y up)
    y: float
    angle: int  # direction from connection point toward the body (deg, CCW)
    length: float
    hidden: bool
    unit: int


@dataclass
class LibSymbol:
    lib: str
    name: str
    node: list  # flattened symbol node, top-level name = bare symbol name
    pins: list[Pin] = field(default_factory=list)
    is_power: bool = False
    units: int = 1
    bbox: tuple[float, float, float, float] = (0, 0, 0, 0)  # xmin, ymin, xmax, ymax (lib coords)
    unit_bbox: dict = field(default_factory=dict)  # unit -> bbox (unit 0 graphics included in every unit)
    properties: dict[str, str] = field(default_factory=dict)

    @property
    def lib_id(self) -> str:
        return f"{self.lib}:{self.name}"

    def unit_of(self, number: str) -> int:
        """Unit that carries this pin (pins common to all units are placed with unit 1)."""
        return self.pin_by_number()[number].unit or 1

    def pin_by_number(self) -> dict[str, Pin]:
        out: dict[str, Pin] = {}
        for p in self.pins:
            if p.number in out:
                raise ValueError(f"{self.lib_id}: duplicate pin number {p.number}")
            out[p.number] = p
        return out

    def embedded(self) -> list:
        """Node for the schematic's lib_symbols section (name gets the lib prefix)."""
        node = copy.deepcopy(self.node)
        node[1] = self.lib_id
        return node


def _prop_map(node: list) -> dict[str, str]:
    return {p[1]: p[2] for p in find_all(node, "property") if len(p) >= 3}


class SymbolLibraries:
    """Resolves ``lib:name`` ids against a nickname → file mapping."""

    def __init__(self, libs: dict[str, str]):
        self.libs = dict(libs)
        self._files: dict[str, list] = {}
        self._cache: dict[str, LibSymbol] = {}

    @classmethod
    def default(cls, project_lib_dir: str | None = None) -> "SymbolLibraries":
        libs: dict[str, str] = {}
        sdir = stock_symbol_dir()
        for path in glob.glob(os.path.join(sdir, "*.kicad_sym")):
            libs[os.path.splitext(os.path.basename(path))[0]] = path
        if project_lib_dir:
            for path in glob.glob(os.path.join(project_lib_dir, "*.kicad_sym")):
                libs[os.path.splitext(os.path.basename(path))[0]] = path
        return cls(libs)

    def _file(self, lib: str) -> list:
        if lib not in self._files:
            if lib not in self.libs:
                raise KeyError(f"unknown symbol library '{lib}'")
            with open(self.libs[lib], encoding="utf-8") as fh:
                self._files[lib] = sexpr.parse(fh.read())
        return self._files[lib]

    def _raw(self, lib: str, name: str) -> list:
        for s in find_all(self._file(lib), "symbol"):
            if s[1] == name:
                return s
        raise KeyError(f"symbol '{name}' not found in library '{lib}'")

    def _flatten(self, lib: str, name: str, depth: int = 0) -> list:
        if depth > 8:
            raise ValueError(f"{lib}:{name}: 'extends' chain too deep")
        raw = self._raw(lib, name)
        ext = find(raw, "extends")
        if not ext:
            return copy.deepcopy(raw)
        parent_name = ext[1]
        base = self._flatten(lib, parent_name, depth + 1)
        child_props = {p[1]: p for p in find_all(raw, "property")}
        out: list = [Sym("symbol"), name]
        seen_props = set()
        for item in base[2:]:
            if isinstance(item, list) and item and item[0] == "property":
                pname = item[1]
                seen_props.add(pname)
                out.append(copy.deepcopy(child_props.get(pname, item)))
            elif isinstance(item, list) and item and item[0] == "symbol":
                sub = copy.deepcopy(item)
                old = sub[1]
                if old.startswith(parent_name + "_"):
                    sub[1] = name + old[len(parent_name):]
                out.append(sub)
            elif isinstance(item, list) and item and item[0] == "embedded_fonts":
                continue
            else:
                out.append(copy.deepcopy(item))
        # properties that only the child defines go after the inherited ones
        insert_at = max(i for i, x in enumerate(out) if isinstance(x, list) and x and x[0] == "property") + 1
        for pname, p in child_props.items():
            if pname not in seen_props:
                out.insert(insert_at, copy.deepcopy(p))
                insert_at += 1
        out.append([Sym("embedded_fonts"), Sym("no")])
        return out

    def get(self, lib_id: str) -> LibSymbol:
        if lib_id in self._cache:
            return self._cache[lib_id]
        if ":" not in lib_id:
            raise ValueError(f"lib_id '{lib_id}' must be 'library:symbol'")
        lib, name = lib_id.split(":", 1)
        node = self._flatten(lib, name)
        sym = analyse(lib, name, node)
        self._cache[lib_id] = sym
        return sym


def _num(v) -> float:
    return float(v)


def _is_hidden(pin: list) -> bool:
    for c in pin:
        if c == "hide":
            return True
        if isinstance(c, list) and c and c[0] == "hide" and len(c) > 1 and c[1] == "yes":
            return True
    return False


def analyse(lib: str, name: str, node: list) -> LibSymbol:
    """Extract pins, unit count and body bounding box from a flattened symbol node."""
    pins: list[Pin] = []
    xs: list[float] = []
    ys: list[float] = []
    units = set()
    per_unit: dict[int, tuple[list, list]] = {}
    for sub in find_all(node, "symbol"):
        sname = sub[1]
        try:
            unit, style = (int(t) for t in sname.rsplit("_", 2)[-2:])
        except ValueError:
            raise ValueError(f"{lib}:{name}: unexpected sub-symbol name {sname}")
        if style not in (0, 1):
            continue  # De Morgan alternate body: ignore
        if unit:
            units.add(unit)
        ux, uy = per_unit.setdefault(unit, ([], []))
        n_before = len(xs)
        for g in sub:
            if not isinstance(g, list) or not g:
                continue
            kind = g[0]
            if kind == "pin":
                at = find(g, "at")
                length = find(g, "length")
                nm = find(g, "name")
                nb = find(g, "number")
                p = Pin(
                    number=str(nb[1]),
                    name=str(nm[1]),
                    etype=str(g[1]),
                    x=_num(at[1]),
                    y=_num(at[2]),
                    angle=int(float(at[3])) % 360 if len(at) > 3 else 0,
                    length=_num(length[1]) if length else 2.54,
                    hidden=_is_hidden(g),
                    unit=unit,
                )
                pins.append(p)
                xs.append(p.x)
                ys.append(p.y)
            elif kind == "rectangle":
                s, e = find(g, "start"), find(g, "end")
                xs += [_num(s[1]), _num(e[1])]
                ys += [_num(s[2]), _num(e[2])]
            elif kind == "polyline":
                for xy in find_all(find(g, "pts") or [], "xy"):
                    xs.append(_num(xy[1]))
                    ys.append(_num(xy[2]))
            elif kind == "circle":
                c, r = find(g, "center"), find(g, "radius")
                cx, cy, rr = _num(c[1]), _num(c[2]), _num(r[1])
                xs += [cx - rr, cx + rr]
                ys += [cy - rr, cy + rr]
            elif kind == "arc":
                for k in ("start", "mid", "end"):
                    pt = find(g, k)
                    if pt:
                        xs.append(_num(pt[1]))
                        ys.append(_num(pt[2]))
        ux += xs[n_before:]
        uy += ys[n_before:]
    is_power = find(node, "power") is not None
    bbox = (min(xs), min(ys), max(xs), max(ys)) if xs else (0.0, 0.0, 0.0, 0.0)
    unit_bbox = {}
    common = per_unit.get(0, ([], []))
    for u in (sorted(units) or [1]):
        ux, uy = per_unit.get(u, ([], []))
        ax, ay = ux + common[0], uy + common[1]
        unit_bbox[u] = (min(ax), min(ay), max(ax), max(ay)) if ax else bbox
    return LibSymbol(
        lib=lib,
        name=name,
        node=node,
        pins=pins,
        is_power=is_power,
        units=max(units) if units else 1,
        bbox=bbox,
        unit_bbox=unit_bbox,
        properties=_prop_map(node),
    )


def footprint_exists(fp: str, project_fp_dirs: dict[str, str] | None = None) -> bool:
    """True if ``Lib:Name`` resolves to a .kicad_mod in the stock or project libraries."""
    if ":" not in fp:
        return False
    lib, name = fp.split(":", 1)
    if project_fp_dirs and lib in project_fp_dirs:
        return os.path.exists(os.path.join(project_fp_dirs[lib], name + ".kicad_mod"))
    return os.path.exists(os.path.join(stock_footprint_dir(), lib + ".pretty", name + ".kicad_mod"))
