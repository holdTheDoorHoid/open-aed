"""Multi-unit parts whose units are drawn on different sheets (part key ``units``)."""
import os
import shutil
import unittest

from tools.schgen import kicad, netcheck, placement, project, spec, symlib
from tools.schgen.tests.util import TLIB, libs, sheet, stock_power_lib

# T:DUAL: unit 1 = pins 1, 6; unit 2 = pins 3, 4; unit 3 = pins 5 (VCC), 2 (GND)
U_A = {"ref": "U5", "group": "g", "lib_id": "T:DUAL", "value": "DUAL", "units": [1, 3],
       "pins": "1 ~ IN_A\n6 ~ MID\n5 VCC 3V3\n2 GND GND"}
U_B = {"ref": "U5", "group": "g", "lib_id": "T:DUAL", "value": "DUAL", "units": [2],
       "pins": "3 ~ MID\n4 ~ OUT_B"}
NETS_A = {"power": ["3V3", "GND"], "global": ["MID"], "single_ok": ["IN_A"]}
NETS_B = {"global": ["MID"], "single_ok": ["OUT_B"]}


class TestSplitSpec(unittest.TestCase):
    def test_only_listed_units_need_pins(self):
        sh = sheet([U_A], nets=NETS_A)
        self.assertEqual(spec.check_against_symbols(sh, libs()), [])
        self.assertEqual(sh.parts[0].units, [1, 3])

    def test_missing_pin_of_listed_unit(self):
        sh = sheet([dict(U_A, pins="1 ~ IN_A\n5 VCC 3V3\n2 GND GND")], nets={"power": ["3V3", "GND"], "single_ok": ["IN_A"]})
        errs = "\n".join(spec.check_against_symbols(sh, libs()))
        self.assertIn("not in the spec", errs)
        self.assertIn("'6'", errs)

    def test_pin_of_other_unit_rejected(self):
        sh = sheet([dict(U_A, pins="1 ~ IN_A\n6 ~ MID\n5 VCC 3V3\n2 GND GND\n3 ~ MID")], nets=NETS_A)
        errs = "\n".join(spec.check_against_symbols(sh, libs()))
        self.assertIn("belong to units not drawn", errs)

    def test_unit_out_of_range_and_bad_key(self):
        sh = sheet([dict(U_A, units=[1, 3, 4])], nets=NETS_A)
        self.assertIn("do not exist", "\n".join(spec.check_against_symbols(sh, libs())))
        with self.assertRaises(spec.SpecError):
            sheet([dict(U_A, units=[1, 1])], nets=NETS_A)
        with self.assertRaises(spec.SpecError):
            sheet([dict(U_A, units="1")], nets=NETS_A)

    def test_cross_sheet_rules(self):
        a = sheet([U_A], nets=NETS_A, name="a")
        b = sheet([U_B], nets=NETS_B, name="b")
        self.assertEqual(spec.check_split_parts([(a, 1), (b, 1)], libs()), [])
        # a unit on no sheet
        self.assertIn("drawn on no sheet", "\n".join(spec.check_split_parts([(a, 1)], libs())))
        # the same unit twice
        b2 = sheet([dict(U_B, units=[2, 3], pins="3 ~ MID\n4 ~ OUT_B\n5 VCC 3V3\n2 GND GND")],
                   nets={"power": ["3V3", "GND"], "global": ["MID"], "single_ok": ["OUT_B"]}, name="b2")
        self.assertIn("unit 3 drawn on both", "\n".join(spec.check_split_parts([(a, 1), (b2, 1)], libs())))
        # different part data
        b3 = sheet([dict(U_B, value="OTHER")], nets=NETS_B, name="b3")
        self.assertIn("disagree on ['value']", "\n".join(spec.check_split_parts([(a, 1), (b3, 1)], libs())))
        # multi-instance sheet
        self.assertIn("multi-instance", "\n".join(spec.check_split_parts([(a, 1), (b, 2)], libs())))
        # the same reference on two sheets without units
        r1 = sheet([{"ref": "R7", "group": "g", "lib_id": "T:R", "value": "1k", "between": ["X", "Y"]}], nets={"single_ok": ["X", "Y"]}, name="r1")
        r2 = sheet([{"ref": "R7", "group": "g", "lib_id": "T:R", "value": "1k", "between": ["X", "Y"]}], nets={"single_ok": ["X", "Y"]}, name="r2")
        self.assertIn("more than one sheet", "\n".join(spec.check_split_parts([(r1, 1), (r2, 1)], libs())))

    def test_layout_draws_only_listed_units(self):
        lay = placement.layout(sheet([U_A], nets=NETS_A), libs(need_power=True))
        self.assertEqual(sorted(pp.unit for pp in lay.parts), [1, 3])
        lay = placement.layout(sheet([U_B], nets=NETS_B), libs())
        self.assertEqual([pp.unit for pp in lay.parts], [2])
        self.assertEqual(set(lay.parts[0].pins), {"3", "4"})


HIER = """
[project]
name = "stest"
title = "Split units test"
root_spec = "specs/00_root.toml"
lib_dir = "lib"

[[sheet]]
name = "sa"
title = "A"
doc = "test"

[[sheet]]
name = "sb"
title = "B"
doc = "test"

[[net_class]]
name = "Default"
"""

ROOT = """
[sheet]
name = "00_root"
title = "root"

[nets]
power = ["3V3", "GND"]
global = ["IN_A", "OUT_B"]
power_flags = ["3V3", "GND"]

[[group]]
id = "g"
title = "Root parts"

[[part]]
ref = "R900"
group = "g"
lib_id = "T:R"
value = "1k"
between = ["IN_A", "OUT_B"]

[[part]]
ref = "R901"
group = "g"
lib_id = "T:R"
value = "10k"
between = ["3V3", "GND"]
"""

SA = """
[sheet]
name = "sa"
title = "A"

[nets]
power = ["3V3", "GND"]
global = ["IN_A", "MID"]

[[group]]
id = "g"
title = "A"

[[part]]
ref = "U5"
group = "g"
lib_id = "T:DUAL"
value = "DUAL"
units = [1, 3]
pins = '''
1 ~   IN_A
6 ~   MID
5 VCC 3V3
2 GND GND
'''
"""

SB = """
[sheet]
name = "sb"
title = "B"

[nets]
global = ["MID", "OUT_B"]

[[group]]
id = "g"
title = "B"

[[part]]
ref = "U5"
group = "g"
lib_id = "T:DUAL"
value = "DUAL"
units = [2]
pins = '''
3 ~   MID
4 ~   OUT_B
'''
"""


@unittest.skipUnless(stock_power_lib() and kicad.available(), "KiCad 10 (flatpak) not available")
class TestSplitIntegration(unittest.TestCase):
    def setUp(self):
        self.dir = os.path.expanduser("~/.cache/ws-schgen-splittest")
        shutil.rmtree(self.dir, ignore_errors=True)
        os.makedirs(os.path.join(self.dir, "specs"))
        os.makedirs(os.path.join(self.dir, "lib"))
        for name, text in (("hierarchy.toml", HIER), ("specs/00_root.toml", ROOT), ("specs/sa.toml", SA),
                           ("specs/sb.toml", SB)):
            with open(os.path.join(self.dir, name), "w") as fh:
                fh.write(text)

    def _libs(self):
        return symlib.SymbolLibraries({"T": TLIB, "power": stock_power_lib()})

    def test_split_part_round_trip(self):
        hier = project.load_hierarchy(os.path.join(self.dir, "hierarchy.toml"))
        sheets = project.generate(hier, self._libs(), log=lambda m: None)
        project.write_project_file(hier)
        project.write_lib_tables(hier)
        rep = kicad.erc(hier.root_file, os.path.join(self.dir, "erc.json"))
        errors = [v for v in kicad.erc_violations(rep) if v["severity"] == "error"]
        self.assertEqual(errors, [], errors)
        text = kicad.netlist(hier.root_file, os.path.join(self.dir, "stest.net"))
        exp = netcheck.expected_for(hier.root_spec, None, 0)
        for e in hier.sheets:
            netcheck.expected_for(sheets[e.name], e.instances[0], 0, {}, exp)
        self.assertEqual(netcheck.compare(exp, text), [])
        nets, refs = netcheck.parse_netlist(text)
        self.assertIn("U5", refs)
        self.assertEqual(nets["MID"], {("U5", "6"), ("U5", "3")})
        self.assertEqual(nets["OUT_B"], {("U5", "4"), ("R900", "2")})

    def test_generate_rejects_uncovered_unit(self):
        with open(os.path.join(self.dir, "specs/sb.toml"), "w") as fh:
            fh.write(SB.replace("ref = \"U5\"", "ref = \"U6\""))
        hier = project.load_hierarchy(os.path.join(self.dir, "hierarchy.toml"))
        with self.assertRaises(spec.SpecError) as cm:
            project.generate(hier, self._libs(), log=lambda m: None)
        self.assertIn("drawn on no sheet", str(cm.exception))


if __name__ == "__main__":
    unittest.main()
