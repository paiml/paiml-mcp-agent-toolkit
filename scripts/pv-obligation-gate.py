#!/usr/bin/env python3
"""Every contract must be readable by pv, and its obligations must bind to code.

Three checks, all blocking, all inside the required `provable ladder` job.

1. `pv validate` passes on every contract.

2. No test-bearing obligation hides under `falsification:`. pv reads
   `falsification_tests:`; under the other key it reports zero, so the audit it
   is supposed to perform silently covers nothing. Thirteen contracts did this,
   hiding 135 of 179 obligations. Entries carrying `action:` and no `test:` are
   left alone — those are alert thresholds, not falsification tests, and forcing
   them into the schema would fabricate tests that do not exist.

3. Every `applies_to` RESOLVES, and where it names a function in a contract that
   declares a `proved_type`, that function's file must reference the type.

   Check 3 is the one that closes the gap this whole exercise is about. Thirteen
   Lean theorems proved the TDG grade order while `check_tdg_grade_gate` — named
   by two obligations in that very contract — still carried its own
   `["A","B","C","D","F"]` table and never mentioned `Grade`. The contract was
   green, the proofs were sound, and the gate was blind to 1,719 violations.
   A proof that the code ignores is decoration.

   `applies_to` may name an equation of the same contract instead of a function;
   `macs-ladder-kernel-v1.yaml` does exactly that with `ord_monotone`, which is
   legitimate and must not be reported.
"""

import glob
import re
import subprocess
import sys

import yaml

CONTRACTS = sorted(f for f in glob.glob("contracts/*.yaml") if not f.endswith("binding.yaml"))


def _fn_files(name: str) -> list[str]:
    out = subprocess.run(
        ["git", "grep", "-l", "-E", rf"fn {re.escape(name)}\b", "--", "src/"],
        capture_output=True, text=True,
    )
    return [p for p in out.stdout.split() if p]


def check_validate(path: str) -> list[str]:
    if subprocess.run(["pv", "validate", path], capture_output=True).returncode != 0:
        return [f"{path}: pv validate failed"]
    return []


def check_visible(path: str, doc: dict) -> list[str]:
    hidden = [e for e in (doc.get("falsification") or [])
              if isinstance(e, dict) and "test" in e]
    if not hidden:
        return []
    return [f"{path}: {len(hidden)} test-bearing obligation(s) under `falsification:`, "
            f"which pv cannot read — use `falsification_tests:`"]


def check_bindings(path: str, doc: dict) -> list[str]:
    equations = set((doc.get("equations") or {}).keys())
    proved = (doc.get("metadata") or {}).get("proved_type")
    problems = []
    for ob in doc.get("proof_obligations") or []:
        target = ob.get("applies_to")
        if not target or target == "all" or target in equations:
            continue
        files = _fn_files(target)
        if not files:
            problems.append(f"{path}: applies_to {target!r} names neither an equation "
                            f"of this contract nor any `fn` under src/")
            continue
        if proved and not any(
            re.search(rf"\b{re.escape(proved)}\b", open(f, encoding="utf-8", errors="replace").read())
            for f in files
        ):
            problems.append(
                f"{path}: applies_to {target!r} is proved against {proved!r}, but none of "
                f"{files} mentions it — the proof does not reach the code it names"
            )
    return problems


def main() -> int:
    problems = []
    for path in CONTRACTS:
        doc = yaml.safe_load(open(path, encoding="utf-8")) or {}
        problems += check_validate(path)
        problems += check_visible(path, doc)
        problems += check_bindings(path, doc)
    for p in problems:
        print(f"::error::{p}")
    print(f"pv obligation gate: {len(problems)} problem(s) over {len(CONTRACTS)} contracts")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
