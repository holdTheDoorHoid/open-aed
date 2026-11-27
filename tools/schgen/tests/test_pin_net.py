"""Per-instance root net templates for multi-instance sheets."""
import unittest

from tools.schgen import project


class PinNet(unittest.TestCase):
    def test_fields(self):
        self.assertEqual(project.pin_net("{tag}_SCK", "CELL_VHF1", 1), "VHF1_SCK")
        self.assertEqual(project.pin_net("SW_TONE_{tag}", "RF_UHF3", 9), "SW_TONE_UHF3")
        self.assertEqual(project.pin_net("3V3_RF_{band}", "CELL_UHF6", 12), "3V3_RF_UHF")
        self.assertEqual(project.pin_net("RXTRIG_{band}", "CELL_VHF2", 2), "RXTRIG_VHF")
        self.assertEqual(project.pin_net("CELL_FITTED{i0}", "CELL_VHF1", 1), "CELL_FITTED0")
        self.assertEqual(project.pin_net("P{n}", "RF_VHF4", 4), "P4")

    def test_per_instance_list(self):
        uhf = [f"CELL_FITTED{k}" for k in range(6, 12)]
        self.assertEqual(project.pin_net(uhf, "CELL_UHF1", 1), "CELL_FITTED6")
        self.assertEqual(project.pin_net(uhf, "CELL_UHF6", 6), "CELL_FITTED11")
        self.assertEqual(project.pin_net(["{tag}_A", "X"], "CELL_UHF1", 1), "UHF1_A")

    def test_list_length_checked(self):
        import os
        d = os.path.expanduser("~/.cache/ws-schgen-pinlist")
        os.makedirs(os.path.join(d, "specs"), exist_ok=True)
        with open(os.path.join(d, "specs", "00_root.toml"), "w") as fh:
            fh.write('[sheet]\nname = "00_root"\ntitle = "r"\n')
        with open(os.path.join(d, "hierarchy.toml"), "w") as fh:
            fh.write('[project]\nname = "t"\ntitle = "t"\nroot_spec = "specs/00_root.toml"\n'
                     '[[sheet]]\nname = "c"\ntitle = "c"\ninstances = ["C_A1", "C_A2"]\n'
                     'pins = { F = ["N0"] }\n')
        with self.assertRaisesRegex(project.spec.SpecError, "lists 1 nets for 2 instances"):
            project.load_hierarchy(os.path.join(d, "hierarchy.toml"))

    def test_no_digits_and_no_underscore(self):
        self.assertEqual(project.pin_net("{tag}_IRQ/{band}/{n}", "CELL_TONE", 13), "TONE_IRQ/TONE/")
        self.assertEqual(project.pin_net("{inst}:{tag}:{i}", "X", 1), "X:X:1")


if __name__ == "__main__":
    unittest.main()
