import unittest

from tools.schgen import sexpr
from tools.schgen.sexpr import S, Sym


class TestParse(unittest.TestCase):
    def test_atoms_and_strings_are_distinguished(self):
        node = sexpr.parse('(property "Reference" "U1" (hide yes) (at 1.27 -2 90))')
        self.assertIsInstance(node[0], Sym)
        self.assertNotIsInstance(node[1], Sym)
        self.assertEqual(node[1], "Reference")
        self.assertIsInstance(node[3][1], Sym)  # yes
        self.assertEqual(node[4], [Sym("at"), Sym("1.27"), Sym("-2"), Sym("90")])

    def test_escapes(self):
        node = sexpr.parse(r'(text "a \"quoted\" word\nnext \\ line")')
        self.assertEqual(node[1], 'a "quoted" word\nnext \\ line')

    def test_empty_string_and_nested(self):
        node = sexpr.parse('(a "" (b (c)))')
        self.assertEqual(node, [S("a"), "", [S("b"), [S("c")]]])

    def test_unbalanced(self):
        with self.assertRaises(ValueError):
            sexpr.parse("(a (b)")
        with self.assertRaises(ValueError):
            sexpr.parse("(a))")

    def test_parse_all_multiple(self):
        self.assertEqual(len(sexpr.parse_all("(a) (b) (c)")), 3)

    def test_unicode(self):
        node = sexpr.parse('(title "Motus — §5.2 ±0.1 ppm µA")')
        self.assertEqual(node[1], "Motus — §5.2 ±0.1 ppm µA")


class TestDump(unittest.TestCase):
    def test_round_trip(self):
        src = '(kicad_sch (version 20260306) (lib_symbols (symbol "Device:R" (pin passive line (at 0 3.81 270) (name "~" (effects (font (size 1.27 1.27))))))) (wire (pts (xy 1 2) (xy 3 4))) (label "A\\"B" (at 0 0 0)))'
        node = sexpr.parse(src)
        again = sexpr.parse(sexpr.dumps(node))
        self.assertEqual(node, again)

    def test_strings_quoted_atoms_not(self):
        out = sexpr.dumps([S("hide"), S("yes"), "yes"])
        self.assertEqual(out, '(hide yes "yes")')

    def test_numbers(self):
        self.assertEqual(sexpr.dumps([S("at"), 1.0, -0.0, 2.54000001, 3]), "(at 1 0 2.54 3)")

    def test_newline_escaped(self):
        self.assertEqual(sexpr.dumps(["a\nb"]), '("a\\nb")')

    def test_xy_runs_on_one_line(self):
        out = sexpr.dumps([S("pts"), [S("xy"), 1, 2], [S("xy"), 3, 4]])
        self.assertEqual(out.splitlines()[1].strip(), "(xy 1 2) (xy 3 4)")

    def test_rejects_unknown_types(self):
        with self.assertRaises(TypeError):
            sexpr.dumps([S("a"), object()])


class TestFind(unittest.TestCase):
    def test_find(self):
        node = sexpr.parse('(sym (property "A" "1") (property "B" "2") (pin 1))')
        self.assertEqual(len(sexpr.find_all(node, "property")), 2)
        self.assertEqual(sexpr.find(node, "pin"), [S("pin"), S("1")])
        self.assertIsNone(sexpr.find(node, "nothing"))
        self.assertEqual(sexpr.head(node), "sym")


if __name__ == "__main__":
    unittest.main()
