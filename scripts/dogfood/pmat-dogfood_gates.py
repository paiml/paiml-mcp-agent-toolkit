#!/usr/bin/env python3
"""The ONE parser of [package.metadata.dogfood].

Requires Python 3.6+ (json, sys, os only — no tomllib, no f-string-with-= ).

Two consumers read this declaration and, until #2644, each carried its own
parser: scripts/dogfood.sh read cargo-metadata JSON through an embedded python
heredoc, while scripts/check_verifier_pinning.sh scraped the TOML with
awk+grep for double-quoted strings ending in `.sh`. Two parsers of one
declaration is the one-source-two-copies class that #2640 exists to close,
sitting inside #2640's own gate (audit finding CI-3).

The divergence was not theoretical. The runner executes EVERY declared string
with `bash "$dg_path"`; the guard's scrape saw only `"...*.sh"`, so a gate
declared as `scripts/check_x` (no suffix) or with TOML literal quotes was
RUN by the release and never SCANNED for unpinned verifiers (VP-05, F2).

Output protocol — one line, or one line per gate:

    NOPKG       no package of that name in the metadata
    NODECL      no [package.metadata.dogfood] table
    BADSHAPE    `gates` is not a list, or holds a non-string / blank entry
    EMPTY       `gates = []` — a claim to have none, which is not a pass
    GATE <path> one per declared gate, in declaration order

Usage:  CRATE=<name> python3 dogfood_gates.py <cargo-metadata.json>
"""
import json
import os
import sys


def _select_package(packages, crate_name):
    """The named package, or the only one when a crate stands alone."""
    for pkg in packages:
        if pkg.get("name") == crate_name:
            return pkg
    if len(packages) == 1:
        return packages[0]
    return None


def _gate_lines(gates):
    """GATE lines for a non-empty list, or None if any entry is malformed."""
    lines = []
    for gate in gates:
        if not isinstance(gate, str) or not gate.strip():
            return None
        lines.append("GATE " + gate.strip())
    return lines


def plan(metadata, crate_name):
    pkg = _select_package(metadata.get("packages", []), crate_name)
    if pkg is None:
        return ["NOPKG"]
    declared = (pkg.get("metadata") or {}).get("dogfood")
    if not isinstance(declared, dict):
        return ["NODECL"]
    gates = declared.get("gates")
    if not isinstance(gates, list):
        return ["BADSHAPE"]
    if not gates:
        return ["EMPTY"]
    return _gate_lines(gates) or ["BADSHAPE"]


def main(argv):
    if len(argv) != 2:
        sys.stderr.write("usage: CRATE=<name> dogfood_gates.py <metadata.json>\n")
        return 2
    try:
        with open(argv[1], encoding="utf-8") as handle:
            metadata = json.load(handle)
    except (OSError, ValueError) as exc:
        sys.stderr.write("dogfood_gates: cannot read metadata: %s\n" % exc)
        return 2
    for line in plan(metadata, os.environ.get("CRATE", "")):
        print(line)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
