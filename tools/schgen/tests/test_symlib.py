import unittest

from tools.schgen.tests.util import libs
from tools.schgen.sexpr import find, find_all


class TestSymlib(unittest.TestCase):
    def setUp(self):
        self.L = libs()

    def test_pins_and_bbox(self):
        s = self.L.get("T:BUF")
        nums = {p.number: p for p in s.pins}
        self.assertEqual(set(nums), {"1", "2", "3", "4"})
        self.assertEqual(nums["1"].name, "A")
        self.assertEqual(nums["1"].angle, 0)
        self.assertEqual(nums["3"].etype, "power_in")
        self.assertEqual(s.bbox, (-7.62, -7.62, 7.62, 7.62))
        self.assertEqual(s.units, 1)

    def test_extends_is_flattened(self):
        s = self.L.get("T:BUF2")
        self.assertEqual(len(s.pins), 4)
        self.assertEqual(s.properties["Datasheet"], "http://child")
        self.assertEqual(s.properties["MPN"], "CHILD-1")
        self.assertEqual(s.properties["Value"], "BUF2")
        subs = [x[1] for x in find_all(s.node, "symbol")]
        self.assertEqual(subs, ["BUF2_0_1", "BUF2_1_1"])
        self.assertIsNone(find(s.node, "extends"))

    def test_embedded_has_lib_prefix(self):
        e = self.L.get("T:BUF2").embedded()
        self.assertEqual(e[1], "T:BUF2")
        self.assertEqual(find_all(e, "symbol")[0][1], "BUF2_0_1")

    def test_multi_unit(self):
        s = self.L.get("T:DUAL")
        self.assertEqual(s.units, 3)
        self.assertEqual(s.unit_of("6"), 1)
        self.assertEqual(s.unit_of("4"), 2)
        self.assertEqual(s.unit_of("5"), 3)
        self.assertEqual(s.unit_bbox[3], (-2.54, -7.62, 2.54, 7.62))

    def test_hidden_pin(self):
        s = self.L.get("T:HIDDENPWR")
        self.assertTrue(s.pin_by_number()["2"].hidden)

    def test_unknown(self):
        with self.assertRaises(KeyError):
            self.L.get("T:NOPE")
        with self.assertRaises(KeyError):
            self.L.get("NOLIB:X")
        with self.assertRaises(ValueError):
            self.L.get("no-colon")


if __name__ == "__main__":
    unittest.main()
