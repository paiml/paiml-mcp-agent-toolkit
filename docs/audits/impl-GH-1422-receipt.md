# Implementation receipt — GH-1422 (ONT-001 row ONT-11, R-21, falsifier F-31)

Issue #1422: *"ONT-11: pmat maps pv verdicts in ci / gate, exposes comply rule ids, one
monotone armed_gates rule — no ontology types (ONT-001 R-21)"*. Scope as the issue states it:

> - (a) The `ci / gate` wrapper maps pv exit 2 (`Unknown`) to non-arming RED and surfaces the `decline:` line.
> - (b) `pmat comply --list-rules --format json` exposes rule ids, so contracts can `resolves: comply-rule`.
> - (c) One fleet-wide comply rule, `armed_gates` monotone.
> - No contract type, Σ, shape logic or `aprender_contracts` import enters this repo. A rule that *forces* arming is refused.

The row's current text (paiml/infra `docs/specifications/paiml-ontology.md`, ONT-11, v4.15)
refines this, and the row is what was built. Where the issue and the row differ, the row wins:
the list is `pmat comply check --list --format json` (the probe calls that spelling), and (a)
lands in CB-1201, the rule `ci / gate` runs.

## What changed

| Part | Where | Behaviour |
|---|---|---|
| (a) | `check_pv_enforcement.rs` (`run_pv_lint`, `classify_pv_lint`) | pv exit 2 gives `Fail` carrying pv's `decline:` line verbatim. Exit 3 gives `Fail` carrying the `error:` line. pv absent while `contracts/` exists gives `Fail` `decline: pv not found`, where it used to be a silent pass. Exit 0/1 keep the JSON + `pv_lint_is_error` policy, and no `contracts/` is still `Skip`. `comply check` exits 1 on any `Fail`. |
| (b) | `check.rs` (`RuleDecl`, `declared_rules`, `print_rule_list`), `misc_commands_comply.rs` (`--list`) | The group registry `--checks` validates against now holds `(id, title)` pairs. `--list --format json` prints `[{id, group, name}]` from it (174 rules), and text format prints one tab-separated row per rule. No second list exists: `every_emitted_rule_carries_its_declared_title` fails when a group emits a title other than the one it declares, and `every_declared_rule_has_one_title` fails on an empty title or a duplicate id. |
| (c) | `check_armed_gates.rs` (CB-2118 `contracts-armed-gates-monotone`), default severity Error | Compares `armed_gates[]` / `armed_shapes[]` in `contracts/lint-baseline.json` at HEAD to the merge-base (`GITHUB_BASE_REF`, else origin/HEAD, origin/master, origin/main, master, main). Any dropped entry gives `Fail` naming each. Removing the `armed_shapes` key is not a drop, because absent means every shape is armed (ONT-001 §3.9). No `contracts/` gives `Skip` `decline: no contracts/`, and a repo that never armed gives `Pass`. The rule never requires arming. |
| F-31 | `ont11_f31_no_source_file_uses_the_contracts_library` + audit arm 13 | RED if any `src/**/*.rs` contains `aprender_contracts::`. `aprender-contracts` stays a dev-dependency only. |

## Evidence

- `scripts/ont11-audit.sh` runs 13 arms against the binary:
  - On this branch's debug build: **13/13 green**.
  - On the installed 3.41.1 (pre-change): **10 RED**. Arms 4, 5 and 13 are controls and stay green on both.
  - `--self-test` is RED against a no-op pmat.
- The row's probe, verbatim, on this branch: `rc=1`, the CB-1201 jq is `true`, and the `--list` jq is `true`.
- Lib tests pass (25): `pv_lint_verdict_tests` (6), `armed_gates_tests` (10, including a git-fixture merge-base arm and F-31), and `tests_select_groups` (including the two new drift tests).
- `docs/status/comply-enforcement-ledger.md` was regenerated with `pmat comply ledger --write`. It gains the CB-2118 row.

## Not done here

- **PVL-001 EV-15** is unbound (EV-14, infra#961, is still open). Nothing here needs it: pmat reads only pv's exit code and its `decline:` / `error:` line.
- CB-1201's reported severity is still set by `.pmat.yaml` (unconfigured means Warning). The exit code does not depend on it: any `Fail` makes `comply check` exit 1.
