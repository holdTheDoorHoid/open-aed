import unittest

from tools.schgen import netcheck, placement, sexpr, writer
from tools.schgen.tests.test_placement import make
from tools.schgen.tests.util import libs


class TestWriter(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.L = libs(need_power=True)

    def build(self):
        sh = make()
        lay = placement.layout(sh, self.L)
        ru = writer.uid("root")
        return sh, writer.build(sh, lay, self.L, [writer.Instance("/" + ru)], "proj", "Project", ru, is_root=True)

    def test_deterministic(self):
        a = sexpr.dumps(self.build()[1])
        b = sexpr.dumps(self.build()[1])
        self.assertEqual(a, b)

    def test_structure(self):
        sh, node = self.build()
        self.assertEqual(node[0], "kicad_sch")
        self.assertEqual(sexpr.find(node, "version")[1], writer.FILE_VERSION)
        libs_ = [s[1] for s in sexpr.find_all(sexpr.find(node, "lib_symbols"), "symbol")]
        self.assertIn("T:BUF", libs_)
        self.assertIn("power:PWR_FLAG", libs_)
        syms = sexpr.find_all(node, "symbol")
        refs = {}
        for s in syms:
            r = [p[2] for p in sexpr.find_all(s, "property") if p[1] == "Reference"][0]
            refs.setdefault(r, []).append(int(sexpr.find(s, "unit")[1]))
        self.assertEqual(sorted(refs["U99"]), [1, 2, 3])
        uuids = [sexpr.find(s, "uuid")[1] for s in syms]
        self.assertEqual(len(uuids), len(set(uuids)))
        tb = sexpr.find(node, "title_block")
        self.assertEqual(sexpr.find(tb, "date")[1], "2026-11-22")

    def test_dnp_flag(self):
        sh = make()
        sh.parts[0].dnp = True
        lay = placement.layout(sh, self.L)
        node = writer.build(sh, lay, self.L, [writer.Instance("/x")], "p", "P", "u")
        dnps = {[p[2] for p in sexpr.find_all(s, "property") if p[1] == "Reference"][0]: sexpr.find(s, "dnp")[1]
                for s in sexpr.find_all(node, "symbol")}
        self.assertEqual(dnps["U1"], "yes")
        self.assertEqual(dnps["U2"], "no")

    def test_offset_ref(self):
        self.assertEqual(writer.offset_ref("R7101", 0), "R7101")
        self.assertEqual(writer.offset_ref("R101", 300), "R401")
        self.assertEqual(writer.offset_ref("TP5", 100), "TP105")


NETLIST = """(export (version "E")
 (components (comp (ref "U1")) (comp (ref "R1")))
 (nets
  (net (code "1") (name "/A") (node (ref "U1") (pin "1")) (node (ref "R1") (pin "1")))
  (net (code "2") (name "GND") (node (ref "U1") (pin "4")) (node (ref "R1") (pin "2")))
  (net (code "3") (name "unconnected-(U1-Pad2)") (node (ref "U1") (pin "2")))))"""


class TestNetcheck(unittest.TestCase):
    def exp(self):
        e = netcheck.Expected()
        e.add("/A", ("U1", "1"))
        e.add("/A", ("R1", "1"))
        e.add("GND", ("U1", "4"))
        e.add("GND", ("R1", "2"))
        e.nc.add(("U1", "2"))
        e.refs |= {"U1", "R1"}
        return e

    def test_match(self):
        self.assertEqual(netcheck.compare(self.exp(), NETLIST), [])

    def test_membership_difference(self):
        e = self.exp()
        e.add("/A", ("U1", "3"))
        errs = netcheck.compare(e, NETLIST)
        self.assertTrue(any("membership differs" in x and "U1.3" in x for x in errs), errs)

    def test_unexpected_net_and_component(self):
        e = self.exp()
        e.nc.clear()
        e.refs.discard("R1")
        errs = "\n".join(netcheck.compare(e, NETLIST))
        self.assertIn("unconnected-(U1-Pad2)", errs)
        self.assertIn("not in any spec", errs)

    def test_renamed_net(self):
        e = self.exp()
        e.nets["/B"] = e.nets.pop("/A")
        errs = "\n".join(netcheck.compare(e, NETLIST))
        self.assertIn("net /B: not in netlist", errs)


if __name__ == "__main__":
    unittest.main()
