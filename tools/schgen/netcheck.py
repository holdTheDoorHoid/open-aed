"""Round-trip check: compare KiCad's exported netlist with what the specs say.

Every net is compared by *membership* (the set of ref.pin nodes) and by name.
Local nets are expected as ``/<sheet instance>/<net>`` (``/<net>`` on the root),
global and power nets by their bare name, hierarchical nets by the root-level
name they are wired to. Pins the spec marks NC must come back unconnected.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from . import sexpr
from .sexpr import find, find_all
from .spec import NC, Sheet
from .writer import offset_ref


@dataclass
class Expected:
    nets: dict[str, set[tuple[str, str]]] = field(default_factory=dict)
    nc: set[tuple[str, str]] = field(default_factory=set)
    refs: set[str] = field(default_factory=set)

    def add(self, name: str, node: tuple[str, str]) -> None:
        self.nets.setdefault(name, set()).add(node)


def expected_for(sheet: Sheet, instance_name: str | None, ref_offset: int,
                 hier_map: dict[str, str] | None = None, exp: Expected | None = None) -> Expected:
    """Add one sheet instance's nets to *exp*. ``instance_name`` None = root sheet."""
    exp = exp or Expected()
    prefix = "/" if instance_name is None else f"/{instance_name}/"
    for p in sheet.parts:
        ref = offset_ref(p.ref, ref_offset)
        exp.refs.add(ref)
        for pm in p.pins:
            node = (ref, pm.number)
            if pm.net == NC:
                exp.nc.add(node)
                continue
            scope = sheet.net_scope(pm.net)
            if scope in ("power", "global"):
                name = pm.net
            elif scope == "hier":
                if not hier_map or pm.net not in hier_map:
                    raise ValueError(f"{sheet.name}: hierarchical net {pm.net} has no root mapping")
                name = hier_map[pm.net]
            else:
                name = prefix + pm.net
            exp.add(name, node)
    return exp


def parse_netlist(text: str) -> tuple[dict[str, set[tuple[str, str]]], set[str]]:
    root = sexpr.parse(text)
    nets: dict[str, set[tuple[str, str]]] = {}
    for net in find_all(find(root, "nets"), "net"):
        name = find(net, "name")[1]
        nodes = set()
        for n in find_all(net, "node"):
            nodes.add((find(n, "ref")[1], find(n, "pin")[1]))
        nets[name] = nodes
    refs = {find(c, "ref")[1] for c in find_all(find(root, "components"), "comp")}
    return nets, refs


def compare(exp: Expected, netlist_text: str) -> list[str]:
    actual, refs = parse_netlist(netlist_text)
    errs: list[str] = []
    missing_refs = exp.refs - refs
    extra_refs = refs - exp.refs
    if missing_refs:
        errs.append(f"components in the specs but not in the netlist: {sorted(missing_refs)}")
    if extra_refs:
        errs.append(f"components in the netlist but not in any spec: {sorted(extra_refs)}")

    node_to_actual: dict[tuple[str, str], str] = {}
    for name, nodes in actual.items():
        for nd in nodes:
            node_to_actual[nd] = name

    for name, nodes in sorted(exp.nets.items()):
        if name not in actual:
            # find where the members went
            where = sorted({node_to_actual.get(nd, "<absent>") for nd in nodes})
            errs.append(f"net {name}: not in netlist (its pins are on {where})")
            continue
        got = actual[name]
        if got != nodes:
            miss = sorted(nodes - got)
            extra = sorted(got - nodes)
            msg = f"net {name}: membership differs"
            if miss:
                msg += f"; missing {[f'{r}.{p}' for r, p in miss]}"
            if extra:
                msg += f"; unexpected {[f'{r}.{p}' for r, p in extra]}"
            errs.append(msg)
    for name, nodes in sorted(actual.items()):
        if name in exp.nets:
            continue
        if len(nodes) == 1 and next(iter(nodes)) in exp.nc:
            continue  # an NC pin, reported by KiCad as unconnected-(...)
        errs.append(f"net {name} in the netlist is not in any spec: {sorted(nodes)}")
    for nd in sorted(exp.nc):
        name = node_to_actual.get(nd)
        if name is not None and len(actual[name]) > 1:
            errs.append(f"{nd[0]}.{nd[1]} is NC in the spec but connected to {name}")
    return errs
