import unittest

from tools.schgen import spec
from tools.schgen.tests.util import libs, sheet

BUF = {"ref": "U1", "group": "g", "lib_id": "T:BUF", "value": "BUF",
       "pins": "1 A IN_NET\n2 Y OUT\n3 VCC 3V3\n4 GND GND"}


class TestSpec(unittest.TestCase):
    def test_pin_table(self):
        rows = spec.parse_pin_table("# c\n 1 A NET_A  # comment\n\n 2 ~ NC\n", "w")
        self.assertEqual([(r.number, r.name, r.net, r.comment) for r in rows],
                         [("1", "A", "NET_A", "comment"), ("2", "~", "NC", "")])
        with self.assertRaises(spec.SpecError):
            spec.parse_pin_table("1 A", "w")

    def test_ok_and_nets(self):
        sh = sheet([BUF, {"ref": "R1", "group": "g", "lib_id": "T:R", "value": "1k", "between": ["OUT", "IN_NET"]}],
                   nets={"power": ["3V3", "GND"]})
        self.assertEqual(spec.check_against_symbols(sh, libs()), [])
        self.assertEqual(sorted(sh.nets()["OUT"]), [("R1", "1"), ("U1", "2")])
        self.assertEqual(sh.net_scope("GND"), "power")
        self.assertEqual(sh.net_scope("OUT"), "local")

    def test_name_mismatch_is_caught(self):
        bad = dict(BUF, pins="1 B IN_NET\n2 Y OUT\n3 VCC 3V3\n4 GND GND")
        sh = sheet([bad, {"ref": "R1", "group": "g", "lib_id": "T:R", "value": "1k", "between": ["OUT", "IN_NET"]}],
                   nets={"power": ["3V3", "GND"]})
        errs = spec.check_against_symbols(sh, libs())
        self.assertTrue(any("pin 1" in e and "'B'" in e and "'A'" in e for e in errs), errs)

    def test_missing_and_extra_pins(self):
        bad = dict(BUF, pins="1 A X\n2 Y X\n3 VCC 3V3\n9 Q X")
        sh = sheet([bad], nets={"power": ["3V3"]})
        errs = "\n".join(spec.check_against_symbols(sh, libs()))
        self.assertIn("not in the spec", errs)
        self.assertIn("does not have", errs)

    def test_power_pin_nc_is_error(self):
        bad = dict(BUF, pins="1 A X\n2 Y X\n3 VCC NC\n4 GND GND")
        sh = sheet([bad], nets={"power": ["GND"]})
        self.assertTrue(any("power pin marked NC" in e for e in spec.check_against_symbols(sh, libs())))

    def test_hidden_power_pin_is_error(self):
        sh = sheet([{"ref": "U1", "group": "g", "lib_id": "T:HIDDENPWR", "value": "x", "pins": "1 IN A\n2 VCC A"}])
        self.assertTrue(any("hidden power pin" in e for e in spec.check_against_symbols(sh, libs())))

    def test_single_member_local_net(self):
        with self.assertRaises(spec.SpecError) as cm:
            sheet([dict(BUF, pins="1 A LONELY\n2 Y X\n3 VCC X\n4 GND X")])
        self.assertIn("LONELY", str(cm.exception))

    def test_duplicate_ref_and_scope_conflict(self):
        with self.assertRaises(spec.SpecError):
            sheet([BUF, BUF], nets={"power": ["3V3", "GND"]})
        with self.assertRaises(spec.SpecError):
            sheet([BUF], nets={"power": ["3V3", "GND"], "global": ["GND", "OUT", "IN_NET"]})

    def test_unused_declared_net(self):
        with self.assertRaises(spec.SpecError):
            sheet([BUF], nets={"power": ["3V3", "GND", "5V"], "global": ["OUT", "IN_NET"]})

    def test_unknown_key(self):
        with self.assertRaises(spec.SpecError):
            sheet([dict(BUF, colour="red")], nets={"power": ["3V3", "GND"], "global": ["OUT", "IN_NET"]})

    def test_between_and_pins_exclusive(self):
        with self.assertRaises(spec.SpecError):
            sheet([{"ref": "R1", "group": "g", "lib_id": "T:R", "value": "1k", "between": ["A", "B"], "pins": "1 ~ A"}])


if __name__ == "__main__":
    unittest.main()
