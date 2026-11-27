"""Minimal S-expression reader/writer for KiCad files (stdlib only).

Representation
--------------
* A list ``(a b c)`` becomes a Python ``list``.
* A bare atom (``kicad_sch``, ``yes``, ``1.27``) becomes a :class:`Sym`.
* A quoted string (``"Reference"``) becomes a plain ``str``.

Keeping atoms and strings apart matters: KiCad distinguishes ``(hide yes)``
from ``(property "hide" ...)`` and a writer that re-quoted everything (or
nothing) would produce files KiCad refuses to load.
"""

from __future__ import annotations

import re

__all__ = ["Sym", "parse", "parse_all", "dumps", "find", "find_all", "head", "q", "S"]


class Sym(str):
    """A bare (unquoted) atom."""

    __slots__ = ()

    def __repr__(self) -> str:  # pragma: no cover - debugging aid
        return f"Sym({str.__repr__(self)})"


def S(text: str) -> Sym:
    """Short constructor for an atom."""
    return Sym(text)


def q(text: str) -> str:
    """Mark a value as a quoted string (identity; for readability at call sites)."""
    return str(text)


_TOKEN = re.compile(
    r"""\s*(?:
        (?P<open>\()|
        (?P<close>\))|
        "(?P<str>(?:[^"\\]|\\.)*)"|
        (?P<atom>[^\s()"]+)
    )""",
    re.VERBOSE | re.DOTALL,
)

_UNESCAPE = {"n": "\n", "t": "\t", "r": "\r", '"': '"', "\\": "\\"}


def _unescape(raw: str) -> str:
    out = []
    i = 0
    while i < len(raw):
        c = raw[i]
        if c == "\\" and i + 1 < len(raw):
            out.append(_UNESCAPE.get(raw[i + 1], raw[i + 1]))
            i += 2
        else:
            out.append(c)
            i += 1
    return "".join(out)


def parse_all(text: str) -> list:
    """Parse every top-level expression in *text*."""
    stack: list[list] = [[]]
    pos = 0
    n = len(text)
    while pos < n:
        m = _TOKEN.match(text, pos)
        if not m:
            rest = text[pos:].strip()
            if not rest:
                break
            raise ValueError(f"S-expression syntax error at offset {pos}: {text[pos:pos + 40]!r}")
        pos = m.end()
        if m.group("open"):
            stack.append([])
        elif m.group("close"):
            if len(stack) == 1:
                raise ValueError(f"unbalanced ')' at offset {pos}")
            done = stack.pop()
            stack[-1].append(done)
        elif m.group("str") is not None:
            stack[-1].append(_unescape(m.group("str")))
        else:
            stack[-1].append(Sym(m.group("atom")))
    if len(stack) != 1:
        raise ValueError("unbalanced '(' at end of input")
    return stack[0]


def parse(text: str):
    """Parse exactly one top-level expression."""
    items = parse_all(text)
    if len(items) != 1:
        raise ValueError(f"expected one top-level expression, found {len(items)}")
    return items[0]


def _quote(s: str) -> str:
    s = s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n").replace("\t", "\\t")
    return f'"{s}"'


def _fmt_num(v) -> str:
    if isinstance(v, bool):
        return "yes" if v else "no"
    if isinstance(v, int):
        return str(v)
    # float: KiCad writes up to 4 decimals, no trailing zeros
    s = f"{v:.4f}".rstrip("0").rstrip(".")
    if s in ("-0", ""):
        s = "0"
    return s


def _atom(v) -> str:
    if isinstance(v, Sym):
        return str(v)
    if isinstance(v, str):
        return _quote(v)
    if isinstance(v, (int, float)):
        return _fmt_num(v)
    raise TypeError(f"cannot serialise {type(v).__name__}: {v!r}")


def _is_flat(node: list) -> bool:
    return all(not isinstance(x, list) for x in node)


def dumps(node, indent: int = 0) -> str:
    """Serialise *node* in KiCad's tab-indented style."""
    if not isinstance(node, list):
        return _atom(node)
    pad = "\t" * indent
    if _is_flat(node):
        return pad + "(" + " ".join(_atom(x) for x in node) + ")"
    # leading atoms stay on the opening line; nested lists go one per line,
    # except runs of (xy ..) points which KiCad keeps on one line.
    parts = []
    i = 0
    while i < len(node) and not isinstance(node[i], list):
        parts.append(_atom(node[i]))
        i += 1
    lines = [pad + "(" + " ".join(parts)]
    rest = node[i:]
    if rest and all(isinstance(x, list) and x and x[0] == "xy" for x in rest):
        lines.append(pad + "\t" + " ".join(dumps(x) for x in rest))
    else:
        for x in rest:
            if isinstance(x, list):
                lines.append(dumps(x, indent + 1))
            else:
                lines.append(pad + "\t" + _atom(x))
    lines.append(pad + ")")
    return "\n".join(lines)


def head(node) -> str | None:
    """Return the first atom of a list node (its 'keyword')."""
    if isinstance(node, list) and node and isinstance(node[0], str):
        return str(node[0])
    return None


def find_all(node: list, key: str) -> list:
    """Direct children of *node* whose keyword is *key*."""
    return [c for c in node if isinstance(c, list) and c and c[0] == key]


def find(node: list, key: str, default=None):
    """First direct child of *node* whose keyword is *key*."""
    for c in node:
        if isinstance(c, list) and c and c[0] == key:
            return c
    return default
