"""The project notice ("NOT FOR CONSTRUCTION — unreviewed design") must reach every sheet's title block."""
import os
import tempfile
import unittest
from types import SimpleNamespace as NS

from tools.schgen import build, project, spec, writer

NOTICE = "NOT FOR CONSTRUCTION — unreviewed design"


class Notice(unittest.TestCase):
    def test_stamp_sets_notice_and_company_once(self):
        sh = spec.Sheet(name="x", title="X", source="doc")
        hier = NS(notice=NOTICE, company="Open AED")
        project._stamp(sh, hier)
        project._stamp(sh, hier)
        self.assertEqual(sh.notice, NOTICE)
        self.assertEqual(sh.company, "Open AED")

    def test_title_block_puts_notice_first(self):
        sh = spec.Sheet(name="x", title="X", source="docs/HARDWARE_DESIGN.md", notice=NOTICE)
        tb = writer.title_block(sh, "P")
        comments = [c for c in tb if isinstance(c, list) and str(c[0]) == "comment"]
        self.assertEqual(comments[0][2], NOTICE)
        self.assertEqual(comments[1][2], "Source: docs/HARDWARE_DESIGN.md")

    def test_placeholder_text_carries_notice_and_owner(self):
        e = project.SheetEntry(name="12_hv_bridge", title="Bridge", doc="d", instances=["12_hv_bridge"],
                               owner="WP-F bridge")
        hier = NS(notice=NOTICE, revision="A", date="2026-11-27", dir="/x/aed-main")
        sh = project.placeholder_spec(e, hier)
        self.assertIn(NOTICE, sh.groups[0].note)
        self.assertIn("hardware/aed-main/specs/12_hv_bridge.toml", sh.groups[0].note)
        self.assertIn("Capture owner: WP-F bridge", sh.comments)

    def test_check_notice_requires_notice(self):
        with tempfile.TemporaryDirectory() as d:
            hier = NS(name="p", notice="", root_file=os.path.join(d, "p.kicad_sch"), dir=d, sheets=[])
            self.assertTrue(build.check_notice(hier))
            hier.notice = "something else"
            self.assertTrue(build.check_notice(hier))

    def test_check_notice_finds_missing_file_text(self):
        with tempfile.TemporaryDirectory() as d:
            root = os.path.join(d, "p.kicad_sch")
            with open(root, "w", encoding="utf-8") as fh:
                fh.write(f'(comment 1 "{NOTICE}")')
            child = NS(file="c.kicad_sch")
            with open(os.path.join(d, "c.kicad_sch"), "w", encoding="utf-8") as fh:
                fh.write("(no notice)")
            hier = NS(name="p", notice=NOTICE, root_file=root, dir=d, sheets=[child])
            errs = build.check_notice(hier)
            self.assertEqual(len(errs), 1)
            self.assertIn("c.kicad_sch", errs[0])


    def test_stale_sheets_only_generated_files(self):
        with tempfile.TemporaryDirectory() as d:
            def w(name, text):
                with open(os.path.join(d, name), "w", encoding="utf-8") as fh:
                    fh.write(text)
            gen = f'(kicad_sch (version 1) (generator "{writer.GENERATOR}")'
            w("p.kicad_sch", gen)
            w("keep.kicad_sch", gen)
            w("old.kicad_sch", gen)
            w("handmade.kicad_sch", '(kicad_sch (version 1) (generator "eeschema")')
            hier = NS(root_file=os.path.join(d, "p.kicad_sch"), dir=d, sheets=[NS(file="keep.kicad_sch")])
            self.assertEqual([os.path.basename(f) for f in build.stale_sheets(hier)], ["old.kicad_sch"])


if __name__ == "__main__":
    unittest.main()
