# impl receipt — PMAT-688: flag-efficacy release gate RED on the 3.39.0 tree

**Ticket:** PMAT-688 (kind:code, p=high, deferred:3.40.0) · **Branch:** `PMAT-688-flag-efficacy` cut from origin/master d90a3c901 · **Orchestrator:** Fable (paiml-implement, direct routing; 1 agy teamwork lane on the plan, 3 quorum lanes on the diff) · **gate_cmd_fallback=true** (discovery: `cargo test --workspace`; `pmat verify` used as the real gate).

## What was red

`make gate-flag-efficacy-full` on 62562ccd9 (v3.39.0), report `/tmp/pmat-flag-efficacy-report.txt`:

```
summary: 584 effective, 4 refuses-honestly, 18 no-op, 4 error-out, 369 skipped
```

The gate asserts `noops.is_empty()` (flag_efficacy.rs) — 18 rows failed it. Error-outs are listed, never asserted; the ticket asked for them to be reclassified anyway because a dirty-tree refusal is documented behaviour.

## Root causes, each reproduced by hand

Every row below was reproduced on a dumped Large corpus (`PMAT_CORPUS_OUT=… cargo test --test all -- --ignored dump_corpus`) with the installed 3.39.0 binary, `--color auto` vs `--color always` for the colour rows (stdout/stderr md5 + exit), before any code was touched.

| flag | measured cause | fix |
|---|---|---|
| `list --color` | box table, no colourable element | header cells through `colors::label`, padded before colouring so the box does not move (`render_table`, extracted from `print_table`) |
| `mcp manifest --color` | two plain `println!`s | notice extracted to `manifest_notice`; hint dimmed, success green |
| `prompt comply/book/repo-image --color` | stdout IS the YAML prompt (21,724 / 25,057 / 14,166 B) | ALLOWED_NOOPS with the measured reason — an escape would corrupt the artifact |
| `roadmap todos --color`, `--include-quality-gates` | baseline failed on the corpus (no `docs/execution/roadmap.md`) with the 🔄 banner already on stdout, so the harness's "not a control" guard never fired; the flag only changed the written file (222 B vs 3,651 B) | corpus roadmap (2-task current sprint); confirmation line names the format and count, ✅ coloured |
| `validate-docs --fail-on-error` | wired (exit 1 iff broken_links > 0), corpus had no broken link | `docs/links.md` with one broken relative link |
| `popper-score --failures-only` | read only as `verbose && !failures_only` and `failures_only && gateway_passed` | the sibling display filter (repo-score `keep_category`, infra-score, rust-project-score): rows ≥ 80% hidden, text + markdown, verbose sub-scores too; recommendations stay. Corpus gains `docs/adr/` (transparency C3, +4) so one category sits above the line |
| `demo-score --failures-only` | only filtered findings, only under `--verbose` | subcategories ≥ 80% hidden in text and markdown |
| `show-metrics --failures-only` | wired, corpus had no trend store | corpus seeds `.pmat-metrics/trends/{lint,test-fast}.json` (8 daily points, rising / falling, p < 0.05) |
| `quality-gates validate --color` | plain `println!` | verdict extracted to `validation_verdict`, coloured |
| `quality-gates show --color` | prints the config as TOML/JSON | ALLOWED_NOOPS (artifact) |
| `cuda-tdg score --color` | plain one-liner | grade through the existing `grade_color`, gateway coloured |
| `cuda-tdg report --color` | default `--format markdown` renders a document | ALLOWED_NOOPS (artifact) |
| `cuda-tdg gate --color`, `--fail-on-p0` | plain text; the corpus gate already fails (gateway FAILED, 0 P0) so the policy could change nothing visible | `P0 policy:` line (and `fail_on_p0` in JSON), PASSED/FAILED coloured |
| `cuda-tdg kaizen --color` | plain text | header through `colors::header` |
| `analyze reachability/unrun-tests --write-ledger`, `--allow-dirty` (error-out ×4) | "refusing to write … from a dirty git tree … or pass --allow-dirty" (exit 1); clap "the following required arguments were not provided: --write-ledger" (exit 2, the flag `requires` it) | `is_honest_refusal` accepts a refusal that names its precondition ⇒ `Verdict::Refuses` |

Also: four ALLOWED_NOOPS entries the report listed as "not exercised" were verified **effective** by hand and deleted (`analyze proof-annotations --color`, `analyze deep-context --quiet`, `comply report --quiet`, `comply check --strict`); the other 13 still reproduce as no-ops and stay.

## RED → GREEN

- RED commit 27ef50ee3: 13 lib tests + 2 harness tests, all failing at that commit (`cargo test --lib -- failures_only_ gate_text_names_the_p0 … table_header_carries`: 10 FAILED; `cargo test --test all -- precondition_refusals_are_honest large_corpus_carries_the_flag_fixtures`: 2 FAILED). The four confirmation lines were extracted into pure fns in the same commit, behaviour-preserving, so the bytes could be asserted on.
- GREEN commit e7e36afcf: the same tests: `test result: ok. 20 passed` (lib, incl. 7 pre-existing `failures_only` tests of the sibling commands) and 3/3 harness.
- Discrimination beyond assertion: the popper verbose test first failed against the *fixture* (`add_sub_score` adds the sub-score's points to the category and pushed B onto the pass line); the fixture, not the filter, was corrected.

## The sweep found two more

First full sweep on e7e36afcf (`make gate-flag-efficacy-full`, 17:56–18:15Z):

```
summary: 593 effective, 8 refuses-honestly, 2 no-op, 0 error-out, 369 skipped
```

- The four deleted allow-list entries did not come back as no-ops (they were effective, as measured).
- `validate-docs --fail-on-error` stayed a no-op: the arg was `bool` with `default_value = "true"`, a switch that can never change anything; the branch binary already exited 1 on the corpus's broken link without it. Fixed as a real switch (`ArgAction::Set`, `num_args = 0..=1`, default true, `--fail-on-error false` opts out); the help advertises `[possible values: true, false]` so the sweep probes both. Verified with the freshly built binary on the dumped corpus: bare / `true` → exit 1, `false` → exit 0.
- `roadmap status --color` appeared: once the corpus carried a roadmap the command became reachable and its table went through no colour helper. Sprint and task headers now go through `colors::header`; json/csv/markdown are asserted escape-free.
- RED df0daad38 (both tests failing), GREEN 64257712a.

## Acceptance (Fable re-ran every command)

Second full sweep on 64257712a (`make gate-flag-efficacy-full`, 18:26–18:44Z, 659 s, exit 0):

```
summary: 612 effective, 8 refuses-honestly, 0 no-op, 0 error-out, 373 skipped
```

against the 3.39.0 baseline `584 effective, 4 refuses-honestly, 18 no-op, 4 error-out, 369 skipped`. The four ledger-writer rows now sit under REFUSES HONESTLY. Skips moved 369 → 373 because two commands the first sweep skipped whole as "nondeterministic baseline" (`comply report *`, `show-metrics *`) became checkable — the seeded trend store made `show-metrics` deterministic — and each now carries three per-flag "needs a value" skips while its other flags are measured. 13 allow-list entries are still "not exercised" (their commands are skipped in the sweep); each was re-verified by hand on the corpus and still reproduces as a no-op, so they stay.

Reports kept: sweep 1 and sweep 2 under the session scratchpad (`report-sweep1.txt`, `report-sweep2.txt`); the RED baseline is the report the ticket cites.

`make gate-differential` (the other gate sharing `build_corpus`): `test result: ok` (57 s, 18:45–18:53Z) — the seeded fixtures did not move any constant-leaf classification.

## Quorum

QUORUM_PLACEHOLDER

## Gate

GATE_PLACEHOLDER

IMPL-PMAT-688-RECEIPT-END
