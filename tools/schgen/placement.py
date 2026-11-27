"""Placement: turn a validated sheet spec into positioned schematic items.

Conventions
-----------
* Schematic coordinates are millimetres, x right, y **down** (KiCad's sheet frame).
* Every symbol origin is snapped to the 1.27 mm (50 mil) grid, so pins drawn on
  the usual 1.27/2.54 mm library grid land on grid too.
* Each connected pin gets a 2.54 mm wire stub pointing away from the body and a
  label (local / global / hierarchical) or power symbol at the stub's free end.
  Nothing is connected by long wires: connectivity is by name only.
* Parts are flowed left-to-right inside their functional group; groups are
  flowed across the page in reading order and framed with a title box.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field

from .spec import NC, Part, Sheet

GRID = 1.27
STUB = 2.54
CHAR_W = 1.05  # estimated advance of one 1.27 mm character (KiCad stroke font)
TEXT_H = 1.27
GROUP_MAX_W = 270.0
GROUP_PAD = 5.08
PART_GAP_X = 5.08
PART_GAP_Y = 5.08
GROUP_GAP = 10.16
TITLE_H = 6.35
PAGE_MARGIN = 20.32
NOTE_FONT = 1.27
NOTE_CHAR_W = 1.12  # measured ~1.05 mm per character at 1.27 mm; keep margin so KiCad never re-wraps

PAPERS = [  # name, width, height (landscape, mm)
    ("A4", 297.0, 210.0),
    ("A3", 420.0, 297.0),
    ("A2", 594.0, 420.0),
    ("A1", 841.0, 594.0),
    ("A0", 1189.0, 841.0),
]

TWO_PIN_HORIZONTAL = {"R", "C", "L", "FerriteBead", "FerriteBead_Small", "R_Small", "C_Small", "L_Small",
                      "C_Polarized", "D_TVS", "D", "D_Schottky", "D_TVS_Small", "R_Shunt"}


def snap(v: float, g: float = GRID) -> float:
    return round(round(v / g) * g, 4)


def rnd(v: float) -> float:
    return round(v + 0.0, 4)


def transform(lx: float, ly: float, rot: int) -> tuple[float, float]:
    """Library point (y up) → offset from the symbol origin on the sheet (y down)."""
    r = math.radians(rot)
    x = lx * math.cos(r) - ly * math.sin(r)
    y = lx * math.sin(r) + ly * math.cos(r)
    return rnd(x), rnd(-y)


def outward(angle: int, rot: int) -> tuple[int, int]:
    """Unit vector (sheet frame) pointing from the pin's connection point away from the body."""
    a = math.radians((angle + 180 + rot) % 360)
    return int(round(math.cos(a))), int(round(-math.sin(a)))


def text_w(s: str) -> float:
    return len(s) * CHAR_W


# ---------------------------------------------------------------------- items
@dataclass
class Wire:
    x1: float
    y1: float
    x2: float
    y2: float


@dataclass
class Label:
    kind: str  # local | global | hier
    net: str
    x: float
    y: float
    angle: int  # 0 = text to the right of the anchor, 180 = to the left
    shape: str = "passive"


@dataclass
class PowerSym:
    lib_id: str
    net: str
    x: float
    y: float
    rot: int
    ref: str = ""
    is_flag: bool = False
    dx: int = 0  # direction the symbol points (away from the pin it hangs on)
    dy: int = -1


@dataclass
class NoConnect:
    x: float
    y: float


@dataclass
class PlacedPart:
    part: Part
    lib_id: str
    x: float
    y: float
    rot: int
    ref_at: tuple[float, float, int]
    val_at: tuple[float, float, int]
    pins: dict[str, tuple[float, float]]  # pin number -> absolute connection point
    unit: int = 1


@dataclass
class TextBox:
    text: str
    x: float
    y: float
    w: float
    h: float
    size: float = 1.27
    bold: bool = False
    border: bool = True


@dataclass
class Layout:
    paper: str
    width: float
    height: float
    parts: list[PlacedPart] = field(default_factory=list)
    wires: list[Wire] = field(default_factory=list)
    labels: list[Label] = field(default_factory=list)
    power: list[PowerSym] = field(default_factory=list)
    noconnects: list[NoConnect] = field(default_factory=list)
    boxes: list[TextBox] = field(default_factory=list)
    frames: list[tuple[float, float, float, float]] = field(default_factory=list)
    bottom: float = 0.0  # lowest y used by any content


# ------------------------------------------------------------ per-part geometry
@dataclass
class _Proto:
    """A part laid out around origin (0, 0) before it is moved into its group."""

    part: Part
    rot: int
    xmin: float
    ymin: float
    xmax: float
    ymax: float
    pins: dict[str, tuple[float, float, int, int]]  # num -> (x, y, dx, dy)
    ref_at: tuple[float, float, int]
    val_at: tuple[float, float, int]
    unit: int = 1


def _stub_len(sheet: Sheet, net: str, dx: int, ic: bool = False) -> float:
    """Side pins of two-pin parts on a rail get a longer stub so the upright rail symbol clears the
    part's own text; ICs get a sideways symbol on a normal stub."""
    return 5.08 if dx != 0 and not ic and sheet.net_scope(net) == "power" else STUB


def _is_ground(net: str) -> bool:
    return net.upper().startswith("GND") or net.upper() in ("VSS", "AGND", "DGND")


def _power_rot(net: str, dx: int, dy: int) -> int:
    if _is_ground(net):  # graphic points down at rotation 0
        return {(0, 1): 0, (1, 0): 90, (0, -1): 180, (-1, 0): 270}[(dx, dy)]
    return {(0, -1): 0, (-1, 0): 90, (0, 1): 180, (1, 0): 270}[(dx, dy)]


def _label_extent(sheet: Sheet, net: str, dx: int, dy: int, ic: bool = False) -> tuple[float, float, float, float]:
    """Bounding box of the label/power symbol hung on a stub end at (0,0)."""
    scope = sheet.net_scope(net)
    if scope == "power":
        w = text_w(net) + 1.0
        if ic and dx != 0:
            reach = 5.5 + w
            return (-reach, -2.0, 0, 2.0) if dx < 0 else (0, -2.0, reach, 2.0)
        # symbol drawn up (rail) or down (ground) from the stub end, value text centred on it
        return (-w / 2, -7.0, w / 2, 7.0)
    extra = 4.0 if scope in ("global", "hier") else 0.8
    w = text_w(net) + extra
    if dx < 0:
        return (-w, -TEXT_H, 0, TEXT_H)
    return (0, -TEXT_H, w, TEXT_H)  # right, and both vertical directions (text runs right)


def protos_for(part: Part, sym, sheet: Sheet) -> list[_Proto]:
    """One proto per unit of the symbol (single-unit symbols give one)."""
    if sym.units <= 1:
        return [proto(part, sym, sheet)]
    units = part.units or range(1, sym.units + 1)  # a split part draws only its units listed for this sheet
    return [proto(part, sym, sheet, unit=u) for u in units]


def proto(part: Part, sym, sheet: Sheet, unit: int = 1) -> _Proto:
    if part.rotation is not None:
        rot = int(part.rotation)
    elif len(sym.pins) == 2 and sym.name in TWO_PIN_HORIZONTAL:
        rot = 90
    else:
        rot = 0
    by_num = sym.pin_by_number()
    bb = sym.unit_bbox.get(unit, sym.bbox) if sym.units > 1 else sym.bbox
    bx = [bb[0], bb[2]]
    by = [bb[1], bb[3]]
    corners = [transform(x, y, rot) for x in bx for y in by]
    xmin = min(c[0] for c in corners)
    xmax = max(c[0] for c in corners)
    ymin = min(c[1] for c in corners)
    ymax = max(c[1] for c in corners)
    body = (xmin, ymin, xmax, ymax)
    pins = {}
    for pm in part.pins:
        lp = by_num[pm.number]
        if sym.units > 1 and (lp.unit or 1) != unit:
            continue
        px, py = transform(lp.x, lp.y, rot)
        dx, dy = outward(lp.angle, rot)
        pins[pm.number] = (px, py, dx, dy)
        if pm.net == NC:
            continue
        ic = len(part.pins) > 2
        stub = _stub_len(sheet, pm.net, dx, ic)
        sx, sy = px + dx * stub, py + dy * stub
        ex = _label_extent(sheet, pm.net, dx, dy, ic)
        xmin = min(xmin, sx + ex[0])
        xmax = max(xmax, sx + ex[2])
        ymin = min(ymin, sy + ex[1])
        ymax = max(ymax, sy + ex[3])
    # reference above, value below
    if rot in (90, 270) and len(sym.pins) == 2:
        cx = (body[0] + body[2]) / 2
        ref_at = (snap(cx), snap(body[1] - 2.54), 0)
        val_at = (snap(cx), snap(body[3] + 2.54), 0)
        val_w = max(text_w(part.value), text_w(part.ref))
        xmin = min(xmin, cx - val_w / 2)
        xmax = max(xmax, cx + val_w / 2)
        ymin = min(ymin, ref_at[1] - TEXT_H)
        ymax = max(ymax, val_at[1] + TEXT_H)
    else:
        ref_at = (snap(body[0]), snap(ymin - 2.54), 0)
        val_at = (snap(body[0]), snap(ymax + 2.54), 0)
        xmax = max(xmax, body[0] + text_w(part.value), body[0] + text_w(part.ref))
        ymin = ref_at[1] - TEXT_H
        ymax = val_at[1] + TEXT_H
    return _Proto(part, rot, xmin, ymin, xmax, ymax, pins, ref_at, val_at, unit)


def _wrap(text: str, width_chars: int) -> list[str]:
    out: list[str] = []
    for para in text.split("\n"):
        words = para.split()
        if not words:
            out.append("")
            continue
        line = ""
        for w in words:
            if line and len(line) + 1 + len(w) > width_chars:
                out.append(line)
                line = w
            else:
                line = f"{line} {w}" if line else w
        out.append(line)
    return out


# ------------------------------------------------------------------- page packing
def pack(sizes: list[tuple[float, float]], page_w: float, margin: float = PAGE_MARGIN,
         gap: float = GROUP_GAP) -> tuple[list[tuple[float, float]], float]:
    """Skyline packing in reading order: each box goes to the highest free spot (then leftmost)
    that fits inside the page width. Returns positions and the page height needed."""
    placed: list[tuple[float, float, float, float]] = []
    pos = []
    for bw, bh in sizes:
        xs = sorted({margin} | {r[2] + gap for r in placed})
        best = None
        for x in xs:
            if x + bw > page_w - margin and x > margin:
                continue
            y = margin
            for (x1, y1, x2, y2) in placed:
                if x < x2 + gap - 1e-6 and x + bw + gap > x1 + 1e-6:  # columns overlap
                    y = max(y, y2 + gap)
            cand = (y, x)
            if best is None or cand < best:
                best = cand
        y, x = best
        placed.append((x, y, x + bw, y + bh))
        pos.append((x, y))
    bottom = max([r[3] for r in placed] + [margin])
    return pos, bottom + margin + 35.0  # title block clearance


# ------------------------------------------------------------------- the layout
def layout(sheet: Sheet, libs, power_ref_base: int = 1) -> Layout:
    groups: dict[str, list[_Proto]] = {g.id: [] for g in sheet.groups}
    for p in sheet.parts:
        groups[p.group].extend(protos_for(p, libs.get(p.lib_id), sheet))

    # 1. lay each group out at a local origin: parts first, then the note wrapped to the final width
    blocks = []
    for g in sheet.groups:
        protos = groups[g.id]
        widest = max([pr.xmax - pr.xmin for pr in protos] + [0.0])
        limit = max(GROUP_MAX_W, widest)
        cursor_x = cursor_y = row_h = used_w = 0.0
        placed = []
        for pr in protos:
            w, h = pr.xmax - pr.xmin, pr.ymax - pr.ymin
            if cursor_x > 0 and (cursor_x + w > limit or (pr.part.new_row and pr.unit == (pr.part.units or [1])[0])):
                cursor_x = 0.0
                cursor_y += row_h + PART_GAP_Y
                row_h = 0.0
            placed.append((pr, cursor_x - pr.xmin, cursor_y - pr.ymin))
            cursor_x += w + PART_GAP_X
            used_w = max(used_w, cursor_x - PART_GAP_X)
            row_h = max(row_h, h)
        inner_w = max(used_w, 140.0 if g.note else 80.0, text_w(g.title) * 1.6 + 5.0)
        note_lines = _wrap(g.note, max(30, int((inner_w - 2.0) / NOTE_CHAR_W))) if g.note else []
        note_h = (len(note_lines) * 1.7 * NOTE_FONT + 2.0) if note_lines else 0.0
        header = TITLE_H + note_h + 2.54
        placed = [(pr, lx, ly + header) for pr, lx, ly in placed]
        total_h = header + cursor_y + row_h if protos else TITLE_H + note_h
        blocks.append((g, inner_w, total_h, placed, note_lines, note_h))

    # 2. choose paper and flow the groups across it
    def flow(page_w: float):
        return pack([(w + 2 * GROUP_PAD, h + 2 * GROUP_PAD) for _, w, h, *_ in blocks], page_w)

    chosen = None
    for name, pw, ph in PAPERS:
        pos, need_h = flow(pw)
        widest = max([w + 2 * GROUP_PAD for _, w, *_ in blocks] + [0])
        if need_h <= ph and widest <= pw - 2 * PAGE_MARGIN:
            chosen = (name, pw, ph, pos)
            break
    if chosen is None:
        name, pw, ph = PAPERS[-1]
        pos, need_h = flow(pw)
        ph = max(ph, need_h)
        chosen = ("User", pw, snap(ph, 10.0), pos)
    paper, pw, ph, pos = chosen
    lay = Layout(paper=paper, width=pw, height=ph)

    pwr_n = power_ref_base
    for (g, w, h, placed, note_lines, note_h), (gx, gy) in zip(blocks, pos):
        ox, oy = gx + GROUP_PAD, gy + GROUP_PAD
        lay.frames.append((gx, gy, gx + w + 2 * GROUP_PAD, gy + h + 2 * GROUP_PAD))
        lay.boxes.append(TextBox(g.title, ox, oy, w, TITLE_H - 1.0, size=2.0, bold=True, border=False))
        if note_lines:
            lay.boxes.append(TextBox("\n".join(note_lines), ox, oy + TITLE_H, w, note_h, size=NOTE_FONT, border=False))
        for pr, lx, ly in placed:
            x0, y0 = snap(ox + lx), snap(oy + ly)
            abs_pins = {}
            for num, (px, py, dx, dy) in pr.pins.items():
                ax, ay = rnd(x0 + px), rnd(y0 + py)
                abs_pins[num] = (ax, ay)
            pp = PlacedPart(
                part=pr.part,
                lib_id=pr.part.lib_id,
                x=x0,
                y=y0,
                rot=pr.rot,
                ref_at=(rnd(x0 + pr.ref_at[0]), rnd(y0 + pr.ref_at[1]), pr.ref_at[2]),
                val_at=(rnd(x0 + pr.val_at[0]), rnd(y0 + pr.val_at[1]), pr.val_at[2]),
                pins=abs_pins,
                unit=pr.unit,
            )
            lay.parts.append(pp)
            for pm in pr.part.pins:
                if pm.number not in abs_pins:
                    continue  # pin belongs to another unit
                ax, ay = abs_pins[pm.number]
                if pm.net == NC:
                    lay.noconnects.append(NoConnect(ax, ay))
                    continue
                _, _, dx, dy = pr.pins[pm.number]
                ic = len(pr.part.pins) > 2
                stub = _stub_len(sheet, pm.net, dx, ic)
                sx, sy = rnd(ax + dx * stub), rnd(ay + dy * stub)
                lay.wires.append(Wire(ax, ay, sx, sy))
                scope = sheet.net_scope(pm.net)
                if scope == "power":
                    # side stubs get an upright symbol (rail arrow up, ground bar down)
                    pdx, pdy = (dx, dy) if (dx == 0 or ic) else (0, 1 if _is_ground(pm.net) else -1)
                    lay.power.append(PowerSym(_power_lib(pm.net), pm.net, sx, sy, _power_rot(pm.net, pdx, pdy),
                                              ref=f"#PWR{pwr_n:04d}", dx=pdx, dy=pdy))
                    pwr_n += 1
                else:
                    angle = 180 if dx < 0 else 0
                    lay.labels.append(Label(scope, pm.net, sx, sy, angle))

    lay.bottom = max([f[3] for f in lay.frames] + [0.0])
    # 3. temporary PWR_FLAGs, in a row under the last group
    if sheet.power_flags:
        y = snap(max(f[3] for f in lay.frames) + 12.7) if lay.frames else 40.0
        x = PAGE_MARGIN + 10.16
        lay.boxes.append(TextBox(
            "Temporary PWR_FLAGs: these rails are generated on sheets that are still placeholders. "
            "Remove each flag when its source sheet is captured.", PAGE_MARGIN, y - 10.16, 230.0, 5.0, border=False))
        for i, net in enumerate(sheet.power_flags):
            fx = snap(x + i * 22.86)
            lay.power.append(PowerSym(_power_lib(net), net, fx, y, 0, ref=f"#PWR{pwr_n:04d}"))
            pwr_n += 1
            lay.wires.append(Wire(fx, y, fx, rnd(y + 2.54)))
            lay.power.append(PowerSym("power:PWR_FLAG", net, fx, rnd(y + 2.54), 180, ref=f"#FLG{pwr_n:04d}",
                                      is_flag=True, dx=0, dy=1))
            pwr_n += 1
        lay.bottom = y + 15.0
    if sheet.flag_nodes:
        y = snap(max(lay.bottom, max([f[3] for f in lay.frames] + [0.0])) + 17.78)
        x = PAGE_MARGIN + 10.16
        lay.boxes.append(TextBox(
            "Supply nodes fed through a ferrite bead or 0 Ohm link: PWR_FLAG tells ERC they are driven on purpose "
            "(permanent, not a placeholder workaround).", PAGE_MARGIN, y - 10.16, 230.0, 5.0, border=False))
        for i, net in enumerate(sheet.flag_nodes):
            fx = snap(x + i * 25.4)
            lay.labels.append(Label("local", net, fx, y, 0))
            lay.wires.append(Wire(fx, y, fx, rnd(y + 2.54)))
            lay.power.append(PowerSym("power:PWR_FLAG", net, fx, rnd(y + 2.54), 180, ref=f"#FLG{pwr_n:04d}",
                                      is_flag=True, dx=0, dy=1))
            pwr_n += 1
        lay.bottom = y + 15.0
    if sheet.power_flags or sheet.flag_nodes:
        need = lay.bottom + 35.0
        if lay.height < need:
            for name, pw, ph in PAPERS:
                if pw >= lay.width and ph >= need:
                    lay.paper, lay.width, lay.height = name, pw, ph
                    break
            else:
                lay.paper, lay.height = "User", snap(need + 10, 10.0)
    return lay


def _power_lib(net: str) -> str:
    if net.upper() == "GND":
        return "power:GND"
    return "power:VCC"
