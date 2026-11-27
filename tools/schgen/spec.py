"""Sheet specification: load and validate the TOML files in ``hardware/<board>/specs``.

A spec is a pin table, written so a reviewer who does not program can check it
against the datasheet. See ``docs/17-kicad-workflow.md`` for the format; the
short version::

    [sheet]
    name = "07_gnss"                 # file stem and sheet name in the root
    title = "GNSS receiver"          # title block
    source = "docs/07-timing.md §5.1"

    [nets]
    power  = ["GND", "3V3_GNSS"]     # drawn as power symbols (global)
    global = ["PPS_GNSS"]            # global labels: signals that leave the sheet
    hierarchical = []                # hierarchical labels (multi-instance sheets)
    power_flags = ["3V3_GNSS"]       # PWR_FLAG while the rail's source sheet is a placeholder

    [[group]]
    id = "rx"
    title = "Receiver"
    note = "Why this group looks the way it does."

    [[part]]
    ref = "U7101"
    group = "rx"
    lib_id = "ws_timing:NEO-F10T"
    value = "NEO-F10T-00B"
    footprint = "RF_GPS:ublox_NEO"
    mpn = "NEO-F10T-00B"
    manufacturer = "u-blox"
    datasheet = "https://..."
    dnp = false
    variant = "Fitted in every build"
    pins = '''
    # pin  name-in-symbol  net
      1    SAFEBOOT_N      GNSS_SAFEBOOT_N
      2    Reserved        NC          # NC = explicit no-connect flag
    '''

Two-pin parts may use ``between = ["NET_A", "NET_B"]`` (pin 1, pin 2) instead of a
pin table. Every pin of the symbol must appear exactly once.
"""

from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass, field

NC = "NC"

NET_RE = re.compile(r"^[A-Za-z0-9_+\-./~{}\[\]]+$")
REF_RE = re.compile(r"^[A-Z#]+[A-Z_]*[0-9]+$")


class SpecError(Exception):
    pass


@dataclass
class PinMap:
    number: str
    name: str  # name the spec expects the symbol to give this pin ("~" = unnamed)
    net: str  # NC for an explicit no-connect
    comment: str = ""


@dataclass
class Part:
    ref: str
    lib_id: str
    value: str
    footprint: str
    group: str
    pins: list[PinMap]
    mpn: str = ""
    manufacturer: str = ""
    datasheet: str = ""
    description: str = ""
    dnp: bool = False
    variant: str = ""
    footprint_todo: bool = False  # footprint named but not yet in any library
    fields: dict[str, str] = field(default_factory=dict)
    rotation: int | None = None  # override automatic orientation
    new_row: bool = False  # start a new row inside the group with this part
    units: list[int] = field(default_factory=list)  # multi-unit symbol: units drawn on this sheet (empty = all)


@dataclass
class Group:
    id: str
    title: str
    note: str = ""


@dataclass
class AcceptedErc:
    type: str
    match: str
    reason: str
    until_sheet: str = ""  # acceptance expires once this sheet has a spec (its driver is captured)


@dataclass
class Sheet:
    name: str
    title: str
    source: str = ""
    revision: str = "A-draft"
    date: str = "2026-11-22"
    company: str = ""
    comments: list[str] = field(default_factory=list)
    power: list[str] = field(default_factory=list)
    global_nets: list[str] = field(default_factory=list)
    hier_nets: list[str] = field(default_factory=list)
    power_flags: list[str] = field(default_factory=list)
    flag_nodes: list[str] = field(default_factory=list)  # local supply nodes fed via ferrite/0R: permanent PWR_FLAG
    single_ok: list[str] = field(default_factory=list)
    groups: list[Group] = field(default_factory=list)
    parts: list[Part] = field(default_factory=list)
    erc_accepted: list[AcceptedErc] = field(default_factory=list)
    path: str = ""

    # ------------------------------------------------------------------ queries
    def net_scope(self, net: str) -> str:
        if net in self.power:
            return "power"
        if net in self.global_nets:
            return "global"
        if net in self.hier_nets:
            return "hier"
        return "local"

    def nets(self) -> dict[str, list[tuple[str, str]]]:
        """net name -> [(ref, pin number)] as captured in the spec (NC excluded)."""
        out: dict[str, list[tuple[str, str]]] = {}
        for p in self.parts:
            for pm in p.pins:
                if pm.net == NC:
                    continue
                out.setdefault(pm.net, []).append((p.ref, pm.number))
        return out


def parse_pin_table(text: str, where: str) -> list[PinMap]:
    rows: list[PinMap] = []
    for lineno, raw in enumerate(text.splitlines(), 1):
        line, _, comment = raw.partition("#")
        line = line.strip()
        if not line:
            continue
        cols = line.split()
        if len(cols) != 3:
            raise SpecError(
                f"{where}: pin table line {lineno} must be 'number name net', got {raw.strip()!r}"
            )
        num, name, net = cols
        rows.append(PinMap(number=num, name=name, net=net, comment=comment.strip()))
    return rows


def _req(d: dict, key: str, where: str):
    if key not in d:
        raise SpecError(f"{where}: missing required key '{key}'")
    return d[key]


def load(path: str) -> Sheet:
    with open(path, "rb") as fh:
        data = tomllib.load(fh)
    return from_dict(data, path)


def from_dict(data: dict, path: str = "<spec>") -> Sheet:
    sh = _req(data, "sheet", path)
    nets = data.get("nets", {})
    sheet = Sheet(
        name=_req(sh, "name", path),
        title=_req(sh, "title", path),
        source=sh.get("source", ""),
        revision=sh.get("revision", "A-draft"),
        date=sh.get("date", "2026-11-22"),
        company=sh.get("company", ""),
        comments=list(sh.get("comments", [])),
        power=list(nets.get("power", [])),
        global_nets=list(nets.get("global", [])),
        hier_nets=list(nets.get("hierarchical", [])),
        power_flags=list(nets.get("power_flags", [])),
        flag_nodes=list(nets.get("flag_nodes", [])),
        single_ok=list(nets.get("single_ok", [])),
        path=path,
    )
    for g in data.get("group", []):
        sheet.groups.append(Group(id=_req(g, "id", path), title=_req(g, "title", path), note=g.get("note", "").strip()))
    for e in data.get("erc_accepted", []):
        sheet.erc_accepted.append(
            AcceptedErc(type=_req(e, "type", path), match=e.get("match", ""), reason=_req(e, "reason", path),
                        until_sheet=e.get("until_sheet", ""))
        )
    known = {
        "ref", "lib_id", "value", "footprint", "group", "pins", "between", "mpn", "manufacturer",
        "datasheet", "description", "dnp", "variant", "footprint_todo", "fields", "rotation", "new_row", "units",
    }
    for raw in data.get("part", []):
        ref = _req(raw, "ref", path)
        where = f"{path}: part {ref}"
        unknown = set(raw) - known
        if unknown:
            raise SpecError(f"{where}: unknown key(s) {sorted(unknown)}")
        if "pins" in raw and "between" in raw:
            raise SpecError(f"{where}: give either 'pins' or 'between', not both")
        if "between" in raw:
            b = raw["between"]
            if len(b) != 2:
                raise SpecError(f"{where}: 'between' needs exactly two nets")
            pins = [PinMap("1", "*", b[0]), PinMap("2", "*", b[1])]
        else:
            pins = parse_pin_table(_req(raw, "pins", where), where)
        units = raw.get("units", [])
        if not isinstance(units, list) or not all(isinstance(u, int) and not isinstance(u, bool) and u >= 1
                                                   for u in units):
            raise SpecError(f"{where}: 'units' must be a list of unit numbers (1, 2, ...)")
        if len(set(units)) != len(units):
            raise SpecError(f"{where}: 'units' lists a unit twice")
        sheet.parts.append(
            Part(
                ref=ref,
                lib_id=_req(raw, "lib_id", where),
                value=str(_req(raw, "value", where)),
                footprint=raw.get("footprint", ""),
                group=_req(raw, "group", where),
                pins=pins,
                mpn=raw.get("mpn", ""),
                manufacturer=raw.get("manufacturer", ""),
                datasheet=raw.get("datasheet", ""),
                description=raw.get("description", ""),
                dnp=bool(raw.get("dnp", False)),
                variant=raw.get("variant", ""),
                footprint_todo=bool(raw.get("footprint_todo", False)),
                fields={str(k): str(v) for k, v in raw.get("fields", {}).items()},
                rotation=raw.get("rotation"),
                new_row=bool(raw.get("new_row", False)),
                units=sorted(units),
            )
        )
    validate_structure(sheet)
    return sheet


def validate_structure(sheet: Sheet) -> None:
    """Checks that need no symbol library."""
    errs: list[str] = []
    group_ids = [g.id for g in sheet.groups]
    if len(set(group_ids)) != len(group_ids):
        errs.append("duplicate group id")
    refs = [p.ref for p in sheet.parts]
    dup = {r for r in refs if refs.count(r) > 1}
    if dup:
        errs.append(f"duplicate references: {sorted(dup)}")
    scopes = [set(sheet.power), set(sheet.global_nets), set(sheet.hier_nets)]
    for i in range(3):
        for j in range(i + 1, 3):
            both = scopes[i] & scopes[j]
            if both:
                errs.append(f"nets declared in two scopes: {sorted(both)}")
    for f in sheet.power_flags:
        if f not in sheet.power:
            errs.append(f"power_flags entry {f} is not a declared power net")
    for f in sheet.flag_nodes:
        if sheet.net_scope(f) != "local":
            errs.append(f"flag_nodes entry {f} must be a local net (use power_flags for rails)")
    for p in sheet.parts:
        if not REF_RE.match(p.ref):
            errs.append(f"{p.ref}: reference must be letters followed by a number")
        if p.group not in group_ids:
            errs.append(f"{p.ref}: unknown group '{p.group}'")
        nums = [pm.number for pm in p.pins]
        d = {n for n in nums if nums.count(n) > 1}
        if d:
            errs.append(f"{p.ref}: pin(s) listed twice: {sorted(d)}")
        for pm in p.pins:
            if pm.net != NC and not NET_RE.match(pm.net):
                errs.append(f"{p.ref} pin {pm.number}: illegal net name {pm.net!r}")
    used = sheet.nets()
    for n in sheet.flag_nodes:
        if n not in used:
            errs.append(f"flag_nodes entry {n} is not used by any pin")
    for n in sheet.power + sheet.global_nets + sheet.hier_nets:
        if n not in used and n not in sheet.power_flags:
            errs.append(f"declared net {n} is not used by any pin")
    for n, members in used.items():
        if sheet.net_scope(n) == "local" and len(members) < 2 and n not in sheet.single_ok:
            errs.append(
                f"local net {n} has a single member {members}; connect it, make it global, "
                "or list it in nets.single_ok"
            )
    if errs:
        raise SpecError(f"{sheet.path}:\n  " + "\n  ".join(errs))


def _norm_name(name: str) -> str:
    return "~" if name in ("", "~") else name


def check_against_symbols(sheet: Sheet, libs) -> list[str]:
    """Cross-check every pin table against the real symbol. Returns error strings."""
    errs: list[str] = []
    for p in sheet.parts:
        try:
            sym = libs.get(p.lib_id)
        except (KeyError, ValueError) as exc:
            errs.append(f"{p.ref}: {exc}")
            continue
        by_num = sym.pin_by_number()
        listed = {pm.number for pm in p.pins}
        if p.units:
            bad_units = [u for u in p.units if u > sym.units]
            if bad_units:
                errs.append(f"{p.ref}: units {bad_units} do not exist ({p.lib_id} has {sym.units} unit(s))")
            # a split part lists only the pins of the units drawn on this sheet (unit-0 pins are common: unit 1)
            other = sorted((n for n in listed if n in by_num and (by_num[n].unit or 1) not in p.units),
                           key=lambda s: (len(s), s))
            if other:
                errs.append(f"{p.ref}: pins {other} belong to units not drawn on this sheet (units = {p.units})")
            by_num = {n: lp for n, lp in by_num.items() if (lp.unit or 1) in p.units}
        missing = sorted(set(by_num) - listed, key=lambda s: (len(s), s))
        extra = sorted(listed - set(by_num), key=lambda s: (len(s), s))
        if missing:
            errs.append(f"{p.ref}: pins not in the spec (every pin must be listed, use NC): {missing}")
        if extra:
            errs.append(f"{p.ref}: spec lists pins the symbol {p.lib_id} does not have: {extra}")
        for pm in p.pins:
            lp = by_num.get(pm.number)
            if not lp:
                continue
            if pm.name != "*" and _norm_name(pm.name) != _norm_name(lp.name):
                errs.append(
                    f"{p.ref} pin {pm.number}: spec says name '{pm.name}', symbol {p.lib_id} says '{lp.name or '~'}'"
                )
            if lp.hidden and lp.etype.startswith("power"):
                errs.append(f"{p.ref} pin {pm.number}: hidden power pin in {p.lib_id} (would join a net silently)")
            if pm.net == NC and lp.etype == "power_in":
                errs.append(f"{p.ref} pin {pm.number} ({lp.name}): power pin marked NC")
    return errs


# Fields that must be identical on every sheet that draws units of one split part.
_SPLIT_SAME = ("lib_id", "value", "footprint", "mpn", "manufacturer", "datasheet", "description", "dnp", "variant",
               "footprint_todo", "fields")


def check_split_parts(placed: list[tuple[Sheet, int]], libs) -> list[str]:
    """Cross-sheet check for multi-unit parts whose units are drawn on different sheets.

    ``placed`` is [(sheet, number of instances of that sheet)]. A reference may appear on more than one sheet
    only if every occurrence names its ``units``, the unit sets are disjoint, together they cover every unit of
    the symbol, the part data is identical, and each such sheet is used exactly once (a multi-instance sheet
    would renumber its copy of the part). A part with ``units`` on one sheet only must still cover every unit.
    Returns error strings."""
    errs: list[str] = []
    occ: dict[str, list[tuple[Sheet, int, Part]]] = {}
    for sh, n_inst in placed:
        for p in sh.parts:
            occ.setdefault(p.ref, []).append((sh, n_inst, p))
    for ref, lst in sorted(occ.items()):
        split = len(lst) > 1 or any(p.units for _, _, p in lst)
        if not split:
            continue
        where = ", ".join(sh.name for sh, _, _ in lst)
        if any(not p.units for _, _, p in lst):
            errs.append(f"{ref}: reference used on more than one sheet ({where}); a part drawn on several sheets "
                        "must name its 'units' on each")
            continue
        for sh, n_inst, _ in lst:
            if n_inst != 1:
                errs.append(f"{ref}: split part on multi-instance sheet {sh.name} ({n_inst} instances)")
        first = lst[0][2]
        for sh, _, p in lst[1:]:
            diff = [k for k in _SPLIT_SAME if getattr(p, k) != getattr(first, k)]
            if diff:
                errs.append(f"{ref}: {sh.name} and {lst[0][0].name} disagree on {diff}")
        seen: dict[int, str] = {}
        for sh, _, p in lst:
            for u in p.units:
                if u in seen:
                    errs.append(f"{ref}: unit {u} drawn on both {seen[u]} and {sh.name}")
                seen.setdefault(u, sh.name)
        try:
            n_units = libs.get(first.lib_id).units
        except (KeyError, ValueError) as exc:
            errs.append(f"{ref}: {exc}")
            continue
        missing = [u for u in range(1, n_units + 1) if u not in seen]
        if missing:
            errs.append(f"{ref}: units {missing} of {first.lib_id} are drawn on no sheet ({where})")
    return errs
