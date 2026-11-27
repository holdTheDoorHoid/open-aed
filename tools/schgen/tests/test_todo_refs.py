"""footprint_todo acceptance must use per-instance references on ref_step sheets."""
import unittest
from types import SimpleNamespace as NS

from tools.schgen import build


class TodoRefs(unittest.TestCase):
    def test_instances_shift_references(self):
        hier = NS(sheets=[NS(name="port", ref_step=100, instances=["A", "B", "C"]),
                          NS(name="single", ref_step=0, instances=["single"])])
        sheets = {"port": NS(name="port", parts=[NS(ref="U21106", footprint_todo=True), NS(ref="R21101", footprint_todo=False)]),
                  "single": NS(name="single", parts=[NS(ref="D7101", footprint_todo=True)])}
        self.assertEqual(build.todo_instance_refs(hier, sheets), {"U21106", "U21206", "U21306", "D7101"})


if __name__ == "__main__":
    unittest.main()
