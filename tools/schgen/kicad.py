"""Thin wrappers around ``kicad-cli`` (KiCad 10, installed as a user flatpak)."""

from __future__ import annotations

import json
import os
import shlex
import subprocess

DEFAULT_CLI = "flatpak run --command=kicad-cli org.kicad.KiCad"
DEFAULT_PY = "flatpak run --command=python3 org.kicad.KiCad"


def cli_cmd() -> list[str]:
    return shlex.split(os.environ.get("WS_KICAD_CLI", DEFAULT_CLI))


def py_cmd() -> list[str]:
    return shlex.split(os.environ.get("WS_KICAD_PY", DEFAULT_PY))


def available() -> bool:
    try:
        r = subprocess.run(cli_cmd() + ["version"], capture_output=True, text=True, timeout=120)
    except (OSError, subprocess.TimeoutExpired):
        return False
    return r.returncode == 0


def run(args: list[str], check: bool = True) -> subprocess.CompletedProcess:
    r = subprocess.run(cli_cmd() + args, capture_output=True, text=True, timeout=900)
    if check and r.returncode not in (0,):
        raise RuntimeError(f"kicad-cli {' '.join(args)} failed ({r.returncode}):\n{r.stdout}\n{r.stderr}")
    return r


def version() -> str:
    return run(["version"]).stdout.strip()


def erc(sch: str, out_json: str) -> dict:
    # exit code is non-zero when violations exist; we judge them ourselves
    r = run(["sch", "erc", "--format", "json", "--severity-all", "--units", "mm", "-o", out_json, sch], check=False)
    if not os.path.exists(out_json):
        raise RuntimeError(f"ERC produced no report:\n{r.stdout}\n{r.stderr}")
    with open(out_json, encoding="utf-8") as fh:
        return json.load(fh)


def netlist(sch: str, out: str) -> str:
    run(["sch", "export", "netlist", "--format", "kicadsexpr", "-o", out, sch])
    with open(out, encoding="utf-8") as fh:
        return fh.read()


def pdf(sch: str, out: str, pages: list[str] | None = None) -> None:
    args = ["sch", "export", "pdf", "-o", out]
    if pages:
        args += ["--pages", ",".join(pages)]
    run(args + [sch])


def drc(pcb: str, out_json: str, parity: bool = False) -> dict:
    """DRC report; parity=True adds KiCad's schematic parity check (footprints vs the schematic, as F8 sees them)."""
    extra = ["--schematic-parity"] if parity else []
    r = run(["pcb", "drc", "--format", "json", "--severity-all", "--units", "mm", *extra, "-o", out_json, pcb],
            check=False)
    if not os.path.exists(out_json):
        raise RuntimeError(f"DRC produced no report:\n{r.stdout}\n{r.stderr}")
    with open(out_json, encoding="utf-8") as fh:
        return json.load(fh)


def erc_violations(report: dict) -> list[dict]:
    """Flatten an ERC JSON report into [{severity, type, description, sheet, items}]."""
    out = []
    for sheet in report.get("sheets", []):
        for v in sheet.get("violations", []):
            out.append({
                "severity": v.get("severity", ""),
                "type": v.get("type", ""),
                "description": v.get("description", ""),
                "sheet": sheet.get("path", ""),
                "items": [i.get("description", "") for i in v.get("items", [])],
            })
    return out


def drc_violations(report: dict) -> list[dict]:
    out = []
    for key in ("violations", "unconnected_items", "schematic_parity"):
        for v in report.get(key, []):
            out.append({
                "group": key,
                "severity": v.get("severity", ""),
                "type": v.get("type", ""),
                "description": v.get("description", ""),
                "items": [i.get("description", "") for i in v.get("items", [])],
            })
    return out
