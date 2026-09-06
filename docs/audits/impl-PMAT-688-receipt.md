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

## Quorum (agy, `--mode plan`, width 3, review-only) on 4be6be1e8

Lanes 5a29c43b-abc7-4c34-9d44-162a797f260a, 4260e6cf-4562-4ca2-b37b-f9a8bd9eaae0, 7e26446c-345f-476f-a979-7bc5871801a9 — **3/3 needs-changes**, no lane said merge-as-is. Findings and what happened to each:

| # | finding | lanes | disposition |
|---|---|---|---|
| 1 | **BLOCKER** — the `_` arms of cuda-tdg score, gate and kaizen were now coloured, so `--format markdown` and `--format sarif` gained ANSI escapes whenever colour was on (`cuda_tdg_handlers_format_score.rs`, `cuda_tdg_handlers_gate_kaizen.rs`); the receipt's own "no machine format gained an escape" row was false | 2 + delegate's own check | **fixed fffcc8845**: colour scoped to `CudaTdgOutputFormat::Terminal`; the test asserts a Sarif config never carries an escape |
| 2 | **MAJOR** — classifying clap's "required arguments were not provided" as an honest refusal converts a sweep that never supplied the `requires` companion into a PASS | 2 of 3 (lane 1 dissented: `baseline.succeeded()` is the control) | **fixed fffcc8845**: phrase dropped; `--allow-dirty` on both ledger writers gets a `PROBE_CONTEXT` of `--write-ledger`, so the flag is probed in context (dirty tree ⇒ the control refuses ⇒ an honest skip; clean ⇒ both write); "refusing to" stays, the ledger writers name `--allow-dirty` in it |
| 3 | "a new `unwrap()` at commands_status_tests.rs:166" | 2 | **refuted by the delegate**: an unchanged context line; 0 added `unwrap(`/`panic!(` under src/ |
| 4 | the `P0 policy:` line and the todos format line are "echoes" that exist to satisfy the sweep | 2 of 3 on the P0 line | kept: `fail_on_p0` does change `passes_quality_gate` (scoring_calculation.rs:172-188); on a tree that already fails the only honest observable is to say which policy was applied, exactly as `Minimum Required` is printed. The todos line is the one place the terminal can show which document was written |
| 5 | Q4(b): no lane opened the trend-store *reader* | delegate | measured before the quorum: `metric_trends_io.rs::load` parses `Vec<MetricObservation>` from `<metric>.json`, and `show-metrics --failures-only` on the seeded corpus copy dropped the improving series by hand |

Agreed sound across all three lanes: the `--failures-only` display-filter semantics match `keep_category`; `--fail-on-error false` is a backwards-compatible `ArgAction::Set` switch with no positional to swallow; the five allow-list entries are artifact outputs; `render_table` pads before colouring; the corpus fixtures are gated to `CorpusSize::Large`. The delegate also verified every changed test file is compiled (no orphan risk).

Third full sweep after the fixes (18503f90f): `make gate-flag-efficacy-full` 19:27–19:44Z, 637 s, exit 0:

```
summary: 595 effective, 6 refuses-honestly, 0 no-op, 0 error-out, 371 skipped
```

Row for row against sweep 2: the two `--allow-dirty` flags moved from REFUSES HONESTLY to SKIPPED with the reason the probe context predicts (`probe context ["--write-ledger"]: baseline exited 1 with empty stdout; the command failed before any flag was read, so it is not a control` — the tree is dirty by then, so the control refuses), and `comply report *` / `show-metrics *` flipped back to whole-command `[nondeterministic baseline]` skips, exactly as in the 3.39.0 sweep; that removes their 21 per-flag rows (hence 612 → 595 effective) and means `show-metrics --failures-only` was measured Effective in sweep 2 only. By hand two consecutive `show-metrics` runs on the dumped corpus are byte-identical, so the flip is between the harness's two baseline runs, not in the command; filed as **PMAT-690**. No sweep since the fix has booked a no-op or an error-out.

## Gate

`pmat verify --format json` on 4be6be1e8 read `ok: false`: the `tests` stage failed on three `documentation_scorer` tests and the unrun-tests ledger drift. The ledger was regenerated with the branch binary (18503f90f; the first regeneration used the installed 3.39.0 before two tests were added). The three scorer tests fail on **every** tree right now: `score_changelog` also reads `project_path.parent()/CHANGELOG.md`, and another session left a 216 KB `/tmp/CHANGELOG.md` at 19:17 local, so a `TempDir` under `/tmp` scores it. Not this branch's file to delete; filed as **PMAT-689** (test isolation). The gate below was re-run with `TMPDIR` pointed at an empty directory so the tests measure their own fixture.

`TMPDIR=/tmp/tmpx pmat verify --format json` on 18503f90f: `ok: null` (no verdict — complexity declined: "no Rust files changed vs HEAD, so nothing was measured" on a clean tree), `stages_measured: 4`, format ✓ satd ✓ clippy ✓ tests ✓ (`cargo test --lib`: 21,255 passed, 0 failed, 154 ignored, 359 s). The same gate with the session scratchpad as TMPDIR failed two other tests on the path alone (`comprehensive_refusal_names_the_quality_score` sees digits in the temp path as a score; `test_collect_files_with_include_filter` matches a path component) — both pass under the neutral directory and neither is touched by this branch. Complexity for the branch is CI's `pmat score` / `ci / gate` to judge; PR #1209.

IMPL-PMAT-688-RECEIPT-END
