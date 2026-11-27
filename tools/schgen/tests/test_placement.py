import itertools
import unittest

from tools.schgen import placement
from tools.schgen.placement import outward, snap, transform
from tools.schgen.tests.util import libs, sheet

GRID = 1.27


def on_grid(v):
    return abs(v / GRID - round(v / GRID)) < 1e-6


def parts(n=6):
    out = []
    for i in range(n):
        out.append({"ref": f"U{i+1}", "group": "g", "lib_id": "T:BUF", "value": "BUF",
                    "pins": f"1 A N{i}\n2 Y N{i+1}\n3 VCC 3V3\n4 GND GND"})
    out.append({"ref": "R1", "group": "h", "lib_id": "T:R", "value": "1k", "between": ["N0", f"N{n}"]})
    out.append({"ref": "U99", "group": "h", "lib_id": "T:DUAL", "value": "DUAL",
                "pins": "1 ~ N0\n6 ~ NC\n3 ~ N1\n4 ~ GLOB\n5 VCC 3V3\n2 GND GND"})
    return out


def make(n=6):
    return sheet(parts(n), nets={"power": ["3V3", "GND"], "global": ["GLOB"], "power_flags": ["3V3"]},
                 groups=[{"id": "g", "title": "Group G", "note": "word " * 200}, {"id": "h", "title": "H"}])


class TestTransform(unittest.TestCase):
    def test_rotations(self):
        self.assertEqual(transform(1, 2, 0), (1, -2))
        self.assertEqual(transform(1, 2, 90), (-2, -1))
        self.assertEqual(transform(1, 2, 180), (-1, 2))
        self.assertEqual(transform(1, 2, 270), (2, 1))

    def test_outward(self):
        # a pin at the left of a body points right (angle 0): the stub goes left
        self.assertEqual(outward(0, 0), (-1, 0))
        self.assertEqual(outward(180, 0), (1, 0))
        self.assertEqual(outward(270, 0), (0, -1))  # top pin: stub goes up (y down on the sheet)
        self.assertEqual(outward(90, 0), (0, 1))
        self.assertEqual(outward(270, 90), (-1, 0))  # resistor rotated 90: pin 1 on the left

    def test_snap(self):
        self.assertEqual(snap(1.3), 1.27)
        self.assertEqual(snap(-0.6), -0.0)


class TestLayout(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.L = libs(need_power=True)
        cls.sh = make()
        cls.lay = placement.layout(cls.sh, cls.L)

    def test_everything_on_grid(self):
        lay = self.lay
        pts = [(p.x, p.y) for p in lay.parts]
        pts += [xy for p in lay.parts for xy in p.pins.values()]
        pts += [(w.x1, w.y1) for w in lay.wires] + [(w.x2, w.y2) for w in lay.wires]
        pts += [(l.x, l.y) for l in lay.labels] + [(p.x, p.y) for p in lay.power]
        pts += [(n.x, n.y) for n in lay.noconnects]
        bad = [p for p in pts if not (on_grid(p[0]) and on_grid(p[1]))]
        self.assertEqual(bad, [])

    def test_every_connected_pin_has_stub_and_label(self):
        ends = {(w.x2, w.y2) for w in self.lay.wires}
        starts = {(w.x1, w.y1) for w in self.lay.wires}
        anchors = {(l.x, l.y) for l in self.lay.labels} | {(p.x, p.y) for p in self.lay.power}
        for pp in self.lay.parts:
            for pm in pp.part.pins:
                if pm.number not in pp.pins:
                    continue
                xy = pp.pins[pm.number]
                if pm.net == "NC":
                    self.assertIn(xy, {(n.x, n.y) for n in self.lay.noconnects})
                else:
                    self.assertIn(xy, starts, f"{pp.part.ref}.{pm.number} has no stub")
        for e in ends:
            self.assertIn(e, anchors | starts)

    def test_stubs_are_short(self):
        for w in self.lay.wires:
            self.assertLessEqual(abs(w.x2 - w.x1) + abs(w.y2 - w.y1), 5.08 + 1e-6)

    def test_multi_unit_placed_per_unit(self):
        units = sorted(p.unit for p in self.lay.parts if p.part.ref == "U99")
        self.assertEqual(units, [1, 2, 3])

    def test_label_kinds(self):
        kinds = {(l.net, l.kind) for l in self.lay.labels}
        self.assertIn(("GLOB", "global"), kinds)
        self.assertIn(("N1", "local"), kinds)
        self.assertTrue(all(p.net in ("3V3", "GND") for p in self.lay.power))
        self.assertEqual(sum(1 for p in self.lay.power if p.is_flag), 1)

    def test_parts_do_not_collide(self):
        origins = [(p.x, p.y) for p in self.lay.parts]
        self.assertEqual(len(origins), len(set(origins)))
        pins = [xy for p in self.lay.parts for xy in p.pins.values()]
        self.assertEqual(len(pins), len(set(pins)), "two pins share a sheet coordinate (would short)")

    def test_groups_do_not_overlap(self):
        for a, b in itertools.combinations(self.lay.frames, 2):
            overlap = a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]
            self.assertFalse(overlap, (a, b))

    def test_page_holds_frames(self):
        for f in self.lay.frames:
            self.assertLessEqual(f[2], self.lay.width)
            self.assertLessEqual(f[3], self.lay.height)

    def test_new_row(self):
        p = parts(3)
        p[2]["new_row"] = True
        sh = sheet(p, nets={"power": ["3V3", "GND"], "global": ["GLOB"]},
                   groups=[{"id": "g", "title": "G"}, {"id": "h", "title": "H"}])
        lay = placement.layout(sh, self.L)
        ys = {pp.part.ref: pp.y for pp in lay.parts}
        self.assertEqual(ys["U1"], ys["U2"])
        self.assertGreater(ys["U3"], ys["U2"])


class TestPack(unittest.TestCase):
    def test_no_overlap_and_width(self):
        sizes = [(100, 50), (100, 200), (100, 30), (250, 40), (60, 60), (60, 10)]
        pos, h = placement.pack(sizes, 420)
        rects = [(x, y, x + w, y + hh) for (x, y), (w, hh) in zip(pos, sizes)]
        for a, b in itertools.combinations(rects, 2):
            self.assertFalse(a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3], (a, b))
        for r in rects:
            self.assertLessEqual(r[2], 420 - placement.PAGE_MARGIN + 1e-6)
        self.assertGreater(h, max(r[3] for r in rects))


if __name__ == "__main__":
    unittest.main()
