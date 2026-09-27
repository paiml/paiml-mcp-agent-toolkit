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
lands in CB-1201, the one pv call `pmat comply check` makes.

**What this row does not do: make a required check run these rules.** The row says so itself
(v4.15, measured): CB-1201 is the one pv call `pmat comply check` makes, "and no required check
runs it — pmat's `ci.yml` `gate` scopes comply to `--checks CB-2113,CB-2115`, `quality-gate.yml`'s
unscoped 'Ladder gate' step is `continue-on-error: true` … so the `ci / gate` half is PVL EV-15's
(sovereign-ci `pv-lint`, fail-closed), not this row's". The same holds for CB-2118, which runs in
the same unscoped invocation. This PR changes what `pmat comply check` decides and exits with. It
does not change `.github/workflows/`, and a job that runs these rules without `continue-on-error`
belongs to EV-15, or to a follow-up ticket that names the workflow change.

## What changed

| Part | Where | Behaviour |
|---|---|---|
| (a) | `check_pv_enforcement.rs` (`run_pv_lint`, `classify_pv_lint`) | pv exit 2 gives `Fail` carrying pv's `decline:` line verbatim. Exit 3 gives `Fail` carrying the `error:` line. pv absent while `contracts/` exists gives `Fail` `decline: pv not found`, where it used to be a silent pass. Exit 0/1 keep the JSON + `pv_lint_is_error` policy, and no `contracts/` is still `Skip`. `comply check` exits 1 on any `Fail`. |
| (b) | `check.rs` (`RuleDecl`, `declared_rules`, `print_rule_list`), `misc_commands_comply.rs` (`--list`) | The group registry `--checks` validates against now holds `(id, title)` pairs. `--list --format json` prints `[{id, group, name}]` from it (174 rules), and text format prints one tab-separated row per rule. No second list exists: `every_emitted_rule_carries_its_declared_title` fails when a group emits a title other than the one it declares, and `every_declared_rule_has_one_title` fails on an empty title or a duplicate id. |
| (c) | `check_armed_gates.rs` (CB-2118 `contracts-armed-gates-monotone`), default severity Error | Compares `armed_gates[]` / `armed_shapes[]` in `contracts/lint-baseline.json` at HEAD to the merge-base (`GITHUB_BASE_REF`, else origin/HEAD, origin/master, origin/main, master, main). Any dropped entry gives `Fail` naming each. Removing the `armed_shapes` key from a baseline that exists is not a drop, because absent means every shape is armed (ONT-001 §3.9). Deleting the file drops every gate and shape the merge-base armed. No `contracts/` gives `Skip` `decline: no contracts/`, and a repo that never armed gives `Pass`. The rule never requires arming. |
| F-31 | `ont11_f31_no_source_file_uses_the_contracts_library` + audit arm 13 | RED if any `src/**/*.rs` contains `aprender_contracts::`. `aprender-contracts` stays a dev-dependency only. |

## Evidence

- `scripts/ont11-audit.sh` runs 14 arms against the binary:
  - On this branch's debug build: **14/14 green**.
  - On the installed 3.41.1 (pre-change): **11 RED**. Arms 4, 5 and 13 are controls and stay green on both.
  - `--self-test` is RED against a no-op pmat.
- The row's probe, verbatim, on this branch: `rc=1`, the CB-1201 jq is `true`, and the `--list` jq is `true`.
- Lib tests pass (25): `pv_lint_verdict_tests` (6), `armed_gates_tests` (10, including a git-fixture merge-base arm and F-31), and `tests_select_groups` (including the two new drift tests).
- `docs/status/comply-enforcement-ledger.md` was regenerated with `pmat comply ledger --write`. It gains the CB-2118 row.

## Not done here

- **Required-check wiring.** No workflow runs CB-1201 or CB-2118 in a required job (see above). The rules decide and exit correctly when run. Making a required check run them is EV-15's job.
- **PVL-001 EV-15** is unbound (EV-14, infra#961, is still open). Nothing here needs it: pmat reads only pv's exit code and its `decline:` / `error:` line.
- CB-1201's reported severity is still set by `.pmat.yaml` (unconfigured means Warning). The exit code does not depend on it: any `Fail` makes `comply check` exit 1.

## CI fixes after the first run on the PR

- **CB-200 (+1 below A).** `judge_armed_gates` graded A-. It is now split into `parse_armed_baseline`, `dropped_armed_entries` and the verdict. The branch then measures 1680, exactly the baseline, with the same binary that measures master at 1680. `check_pv_lint` now builds its rows through `pv_lint_row`, and maps a run through `pv_lint_verdict`. Its behaviour and its grade (A-, as on master) are unchanged.
- **CB-2115 (not this ticket's defect).** #1419 was closed as a duplicate of #1285 at 2026-09-26T18:02Z while `roadmap.yaml` still listed GH-1419 `planned`, which turns `traceability` red for every PR. `pmat work edit GH-1419 -s cancelled` records the close (#1285's fix is PR #1453), and CB-2115 then passes (126 ↔ 126).
