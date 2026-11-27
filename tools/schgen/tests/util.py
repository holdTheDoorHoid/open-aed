"""Shared test helpers."""
import glob
import os
import unittest

from tools.schgen import spec, symlib

HERE = os.path.dirname(os.path.abspath(__file__))
TLIB = os.path.join(HERE, "fixtures", "tlib.kicad_sym")


def stock_power_lib():
    try:
        d = symlib.stock_symbol_dir()
    except FileNotFoundError:
        return None
    p = os.path.join(d, "power.kicad_sym")
    return p if os.path.exists(p) else None


def libs(need_power=False):
    m = {"T": TLIB}
    pw = stock_power_lib()
    if pw:
        m["power"] = pw
    elif need_power:
        raise unittest.SkipTest("KiCad stock power library not installed")
    return symlib.SymbolLibraries(m)


def sheet(parts, nets=None, groups=None, name="t"):
    return spec.from_dict({
        "sheet": {"name": name, "title": "test"},
        "nets": nets or {},
        "group": groups or [{"id": "g", "title": "G", "note": "note"}],
        "part": parts,
    })
