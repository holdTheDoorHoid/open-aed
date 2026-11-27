"""End-to-end: a tiny hierarchical project (one sheet used twice, hierarchical labels) through kicad-cli.

Skipped when KiCad is not installed. The project is written under ~/.cache because the KiCad flatpak
cannot see /tmp.
"""

import os
import shutil
import unittest

from tools.schgen import kicad, netcheck, project, spec, symlib
from tools.schgen.tests.util import TLIB, stock_power_lib

HIER = """
[project]
name = "itest"
title = "Integration test"
root_spec = "specs/00_root.toml"
lib_dir = "lib"

[[sheet]]
name = "cell"
title = "Cell"
doc = "test"
instances = ["CELL_A", "CELL_B"]
ref_step = 100
pins = { IN = "SIG_{inst}", OUT = "BUS" }

[[sheet]]
name = "empty"
title = "Placeholder"
doc = "nothing yet"

[[net_class]]
name = "Default"
"""

ROOT = """
[sheet]
name = "00_root"
title = "root"

[nets]
global = ["SIG_CELL_A", "SIG_CELL_B", "BUS"]
power = ["3V3", "GND"]
power_flags = ["3V3", "GND"]

[[group]]
id = "g"
title = "Root parts"

[[part]]
ref = "R900"
group = "g"
lib_id = "T:R"
value = "1k"
between = ["SIG_CELL_A", "BUS"]

[[part]]
ref = "R901"
group = "g"
lib_id = "T:R"
value = "1k"
between = ["SIG_CELL_B", "BUS"]

[[part]]
ref = "R902"
group = "g"
lib_id = "T:R"
value = "10k"
between = ["3V3", "GND"]
"""

CELL = """
[sheet]
name = "cell"
title = "cell"

[nets]
power = ["3V3", "GND"]
hierarchical = ["IN", "OUT"]

[[group]]
id = "g"
title = "Cell"

[[part]]
ref = "U1"
group = "g"
lib_id = "T:BUF"
value = "BUF"
pins = '''
1 A   IN
2 Y   MID
3 VCC 3V3
4 GND GND
'''

[[part]]
ref = "R1"
group = "g"
lib_id = "T:R"
value = "100"
between = ["MID", "OUT"]
"""


@unittest.skipUnless(stock_power_lib() and kicad.available(), "KiCad 10 (flatpak) not available")
class TestIntegration(unittest.TestCase):
    def setUp(self):
        self.dir = os.path.expanduser("~/.cache/ws-schgen-itest")
        shutil.rmtree(self.dir, ignore_errors=True)
        os.makedirs(os.path.join(self.dir, "specs"))
        os.makedirs(os.path.join(self.dir, "lib"))
        for name, text in (("hierarchy.toml", HIER), ("specs/00_root.toml", ROOT), ("specs/cell.toml", CELL)):
            with open(os.path.join(self.dir, name), "w") as fh:
                fh.write(text)

    def test_hierarchical_round_trip(self):
        hier = project.load_hierarchy(os.path.join(self.dir, "hierarchy.toml"))
        libs = symlib.SymbolLibraries({"T": TLIB, "power": stock_power_lib()})
        sheets = project.generate(hier, libs, log=lambda m: None)
        project.write_project_file(hier)
        project.write_lib_tables(hier)

        rep = kicad.erc(hier.root_file, os.path.join(self.dir, "erc.json"))
        errors = [v for v in kicad.erc_violations(rep) if v["severity"] == "error"]
        self.assertEqual(errors, [], errors)

        text = kicad.netlist(hier.root_file, os.path.join(self.dir, "itest.net"))
        exp = netcheck.expected_for(hier.root_spec, None, 0)
        for e in hier.sheets:
            for i, inst in enumerate(e.instances):
                hm = {k: project.pin_net(v, inst, i + 1) for k, v in e.pins.items()}
                netcheck.expected_for(sheets[e.name], inst, i * e.ref_step, hm, exp)
        self.assertEqual(netcheck.compare(exp, text), [])
        # the two instances got distinct references and their own local nets
        nets, refs = netcheck.parse_netlist(text)
        self.assertTrue({"U1", "U101", "R1", "R101", "R900", "R901"} <= refs)
        self.assertEqual(nets["/CELL_A/MID"], {("U1", "2"), ("R1", "1")})
        self.assertEqual(nets["/CELL_B/MID"], {("U101", "2"), ("R101", "1")})
        self.assertEqual(nets["BUS"], {("R1", "2"), ("R101", "2"), ("R900", "2"), ("R901", "2")})

        # and a deliberately wrong spec is caught by the round trip
        sheets["cell"].parts[1].pins[1].net = "MID"
        exp2 = netcheck.expected_for(hier.root_spec, None, 0)
        for e in hier.sheets:
            for i, inst in enumerate(e.instances):
                hm = {k: project.pin_net(v, inst, i + 1) for k, v in e.pins.items()}
                netcheck.expected_for(sheets[e.name], inst, i * e.ref_step, hm, exp2)
        self.assertNotEqual(netcheck.compare(exp2, text), [])

    def test_flag_in_multi_instance_sheet_rejected(self):
        with open(os.path.join(self.dir, "specs/cell.toml"), "w") as fh:
            fh.write(CELL.replace('power = ["3V3", "GND"]', 'power = ["3V3", "GND"]\npower_flags = ["3V3"]'))
        hier = project.load_hierarchy(os.path.join(self.dir, "hierarchy.toml"))
        libs = symlib.SymbolLibraries({"T": TLIB, "power": stock_power_lib()})
        with self.assertRaises(spec.SpecError) as cm:
            project.generate(hier, libs, log=lambda m: None)
        self.assertIn("3V3", str(cm.exception))

    def test_spec_error_reported(self):
        bad = CELL.replace("1 A   IN", "1 B   IN")
        with open(os.path.join(self.dir, "specs/cell.toml"), "w") as fh:
            fh.write(bad)
        hier = project.load_hierarchy(os.path.join(self.dir, "hierarchy.toml"))
        libs = symlib.SymbolLibraries({"T": TLIB, "power": stock_power_lib()})
        with self.assertRaises(spec.SpecError):
            project.generate(hier, libs, log=lambda m: None)


if __name__ == "__main__":
    unittest.main()
