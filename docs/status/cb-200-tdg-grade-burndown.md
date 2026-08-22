# CB-200 burn-down ledger — what the below-A definitions actually are

**Purpose.** CB-200 is becoming a ratchet on its own measured baseline. A ratchet
that reports only a number is how 1,905 violations accumulated unseen in the first
place. This document is the debt itself, itemised, so that "the baseline is just
hiding debt" has an answer other than a promise.

**Measured, not asserted.** Every number below was produced by querying
`.pmat/context.db` read-only and, where the gate's own arithmetic was involved, by
re-executing that arithmetic. Where a step could not be reproduced exactly, it says so.

## Reproduction stamp

```text
repo HEAD at measurement   2cfd7dfdcbc53fcfc08e61cf1f22b416bc7df7a3  (2026-08-22 14:35:44 +0200)
.pmat/context.db mtime     2026-08-22 10:05:16 +0200
.pmat/context.db sha256    4fa83a23463bf94c61b8d9b7880a26f062f3e144c8f143b7cd1656b540653d9c
metadata.built_at          2026-08-21T13:26:15Z          <-- see section 1
functions rows             23,451 across 2,626 files
index freshness            STALE: 18 .rs files under src/ are newer than the db
```

`pmat comply check` was **not** run (it saturates the machine). The gate's scope was
reproduced by reading
`src/cli/handlers/comply_handlers/check_handlers/check_tdg_grade.rs` and re-issuing
its query.

## 0. Reproducing CB-200's scope, and the three places it could not be matched exactly

CB-200 at HEAD does this: floor `min_tdg_grade` (no `[tdg].min_grade` in
`.pmat-gates.toml`, so `.pmat.yaml`'s `"A"` wins), so
`passing_spellings("A") == ["A+", "A"]`; then
`SELECT ... FROM functions WHERE tdg_grade NOT IN ('A+','A')`; then drop test paths
and the union of `.pmat.yaml` `tdg_exclude_paths` and `.pmat-gates.toml` `[tdg].exclude`.

```text
rows below the floor                                        2,039
  hardcoded test-path filter (/tests/, /test/, *_test.rs)  removes     0
  tests/*                          (.pmat.yaml)            removes     0
  examples/*  + examples/**                                removes    67
  benches/*   + benches/**                                 removes     6
  scripts/**                       (.pmat-gates.toml)      removes    40
  src/cli/command_dispatcher/**    (.pmat-gates.toml)      removes    21
  src/cli/command_structure.rs     (.pmat-gates.toml)      removes     0
  src/tdg/.../analyzer_impl2_heuristics_lean.rs (.pmat.yaml) removes   1
CB-200 in-scope violations                                  1,904
  distinct files                                            1,052
  maximum per file                                             12
```

**1,904, not 1,905.** The file count (1,052) and the per-file maximum (12) match the
brief exactly; the total is one lower. The difference is a single row in
`src/tdg/analyzer_ast/analyzer_impl2_heuristics_lean.rs`, which is excluded by
`.pmat.yaml`'s `tdg_exclude_paths` but by nothing in `.pmat-gates.toml`. 1,905 is the
count when only the `.pmat-gates.toml` exclusions apply. Whichever the gate actually
produces, the delta is one row and it is named.

**Where the reproduction is not exact:**

1. **Glob semantics.** `is_tdg_violation_excluded` uses the `glob` crate with
   `require_literal_separator: false`, under which `*`, `**` and `?` all cross `/`.
   For the nine patterns actually in play that reduces to prefix matching, which is
   what was implemented. Crate-specific edge cases were not exercised.
2. **Config loading.** `.pmat.yaml`'s `comply.thresholds` was read by hand rather
   than through `ComplyConfig`'s deserializer, because running the gate is banned.
   The `#[serde(default)]` on `tdg_exclude_paths` means a load failure yields an
   empty list — which is exactly the 1,904/1,905 ambiguity above.
3. **Staleness.** CB-200 would append its "index is stale" note today: 18 source
   files under `src/` are newer than the db. It reports the staleness and measures
   anyway; so does this document.

**Four of the nine exclusions, and the entire hardcoded test-path filter, remove
zero rows.** Not because they are wrong, but because the indexer never writes test
code into `functions` at all: zero rows under `tests/`, zero under `src/tests/`,
zero in any `*_tests.rs`. Test chunks are dropped upstream by `is_test_chunk`
(`src/services/agent_context/function_index/helpers_call_graph.rs:43`). Anyone
reading the exclusion list will believe it is doing work that the data says it is not.

---

## 1. Before anything else: the number is measured by a scanner the tree has already replaced

This is the most important finding in the document, and it is not a judgement call.

`.pmat/context.db` records `metadata.built_at = 2026-08-21T13:26:15Z`. The commit
`f031cdb0b` — *"fix: the complexity scanner counted closures and comments as
branches"* — landed at **2026-08-21 21:04 UTC**, about eight hours later. The `pmat`
on `PATH` is 3.32.0 at commit `8134bb373`, which is **not** a descendant of
`f031cdb0b`.

So the stored `complexity` and `tdg_grade` were computed by the *pre-fix* scanner.
That was verified two independent ways:

- The pre-fix `count_complexity` (from `git show f031cdb0b^`), re-implemented and run
  over the stored `source` text of all 1,904 in-scope rows, reproduces the stored
  `complexity` **1,904 / 1,904 exactly**. HEAD's scanner reproduces only 1,246.
- The scoring function is deterministic in `(complexity, satd_count, loc)` — the
  formula in `calculate_simple_tdg` reproduces the stored `tdg_grade` for
  **23,451 / 23,451 rows in the whole index**.

HEAD's `count_complexity` was then extracted verbatim, compiled standalone, and run
over all 23,451 stored sources. Combined with the (unchanged) scoring function:

| | stored (pre-fix scanner) | HEAD scanner, index rebuilt |
|:--|--:|--:|
| **CB-200 in-scope violations** | **1,904** | **1,685** |
| cured by the rebuild alone | — | 219 |
| newly appearing | — | 0 |
| files with a violation | 1,052 | 969 |

**A baseline frozen from today's index is 219 violations (11.5%) too high**, and it
would decay to 1,685 the first time anyone runs `pmat query` with a binary built from
HEAD — handing the ratchet 219 units of free headroom that no one earned. The
baseline must be re-derived after rebuilding the index with a HEAD binary, or the
ratchet's first act is to certify a number produced by code that no longer exists.

### 1b. …and the replacement scanner has its own blind spot, in the other direction

The 219 are not all phantoms. Attributing every one of the 1,828 decision points
that the stored index counts and HEAD's scanner does not:

| dropped points | share | reason |
|--:|--:|:--|
| 941 | 51.5% | line-leading `\|\|` — either a closure or a multi-line boolean-or |
| 518 | 28.3% | comment-only line (the old scanner read comments as code) |
| 313 | 17.1% | inline closure `\|\|` (`ok_or_else`, `unwrap_or_else`, …) |
| 53 | 2.9% | trigger inside a string literal or a comment tail |
| 3 | 0.2% | other string-blanking effects |

The first row is the problem. `has_boolean_or` treats a `||` that *opens a line* as a
closure, because "a closure's `||` follows a delimiter or another operator, or opens
the line". But a multi-line boolean condition opens continuation lines with `||` too.
Classifying every line-leading `||` in the set (960 lines, of which 941 cost a
decision point) by what the *preceding* executable line ends with — a delimiter or
operator means a closure argument split across lines, anything else means a
continued disjunction — puts **at least 905 of them in genuine boolean-or
continuations**: real branches HEAD's scanner no longer counts. The heuristic is
conservative in the wrong direction (a previous line ending `==` is filed as a
closure), so 905 is a floor. Examples it found:

```text
src/quality/gates_checks.rs:24            || line.contains("panicked")
src/mcp_integration/deep_wasm_tools_query_mapping.rs:43   || m.source_map_entry
```

The clearest single case is `src/services/spec_parser_impl.rs:401 categorize_claim`:
a twenty-clause `if a || b || c || …` chain, stored as `F` at cc 36, rescanned by
HEAD as **`A` at cc 5**. The 31 lost points are real disjunctions.

**Conclusion: neither 1,904 nor 1,685 is a count of "definitions below A" in any
sense a reader would recognise.** Both are counts of source lines matching a set of
textual triggers, and the trigger set changed yesterday. This does not argue against
the ratchet — it argues that the ratchet must pin the *measuring code*, not just the
number, exactly as `.pmat-ratchet.toml` already does by re-running each metric's
command rather than reading its recorded value.

**Everything below uses the HEAD-rescan set of 1,685**, because that is what the gate
will measure as soon as the index is rebuilt. Stored figures are given alongside
where the two differ materially.

---

## 2. The distribution

CB-200's floor is `A`, so `A-` and everything below it is a violation.

| grade | stored (1,904) | HEAD-rescan (1,685) | cumulative |
|:--|--:|--:|--:|
| A- | 1,141 (59.9%) | **1,032 (61.2%)** | 61.2% |
| B+ | 382 (20.1%) | 354 (21.0%) | 82.3% |
| B | 190 (10.0%) | 158 (9.4%) | 91.6% |
| B- | 84 (4.4%) | 70 (4.2%) | 95.8% |
| C+ | 49 (2.6%) | 31 (1.8%) | 97.6% |
| C | 21 (1.1%) | 14 (0.8%) | 98.5% |
| C- | 18 (0.9%) | 11 (0.7%) | 99.1% |
| D | 9 (0.5%) | 6 (0.4%) | 99.5% |
| F | 10 (0.5%) | 9 (0.5%) | 100.0% |

**The debt is one band deep.** 61% of it is `A-` — a single grade below the floor —
and 92% is `A-`/`B+`/`B`. Only 40 definitions in the entire repository are `C+` or
worse, and only 15 are `D` or `F`.

This is *not* an argument for moving the floor; the floor stays `A`. It is a
statement about the shape of the work: this is a very large number of very small
distances, not a small number of disasters.

### 2b. What the grade actually measures

The grade is a deterministic function of three stored columns and nothing else —
verified by reproducing all 23,451 stored grades from them:

```text
score = 100
      - min(1.5 * (cyclomatic - 1), 50)      # branches
      - min(5   * (satd - 2),       20)      # self-admitted debt
      - min((loc - 50) / 15,        30)      # size, above 50 lines
      floored at 90 when cyclomatic <= 1
grade = A+ >=95, A >=90, A- >=85, B+ >=80, B >=75, B- >=70, C+ >=65, C >=60, C- >=55, D >=50, F otherwise
```

Consequences worth stating plainly:

- `cognitive_complexity` is not a second signal. The indexer writes
  `cognitive_complexity: complexity` with the comment *"Simplified: use same as
  cyclomatic"*. It is identical for all 23,451 rows.
- **SATD contributes nothing.** Only 6 rows in the whole index have any SATD marker
  at all, and none has more than the two free ones. Zero of the 1,685 are penalised.
- **Size contributes almost nothing.** 854 of the 1,685 (50.7%) are over 50 lines,
  but only **72 (4.3%)** would reach grade A if the size penalty vanished entirely.
- Therefore **CB-200 at floor A is, to within 4%, the single predicate
  `cyclomatic complexity >= 8`**, where "cyclomatic complexity" means "number of
  source lines matching a trigger list, plus one".

The 8 is not a threshold anybody chose: `100 - 1.5*(8-1) = 89.5`, which is 0.5 below
the `A` band. And `.pmat-gates.toml` sets `max_complexity = 10` for the project's own
complexity gate — so **1,038 of the 1,685 (61.6%) sit at or below the complexity
limit the repo already declares acceptable elsewhere.** Two gates in the same tree
disagree about where "too complex" begins, and the disagreement accounts for most of
this backlog.

---

## 3. The shape: is there a hotspot?

**Claim under test: "flat, max 12 per file, no hotspot."**

### Per file — CONFIRMED, decisively

```text
                                       stored          HEAD-rescan
violations                              1,904               1,685
files                                   1,052                 969
mean per file                            1.81                1.74
median / p75 / p90 / p99 / max      1 / 2 / 3 / 7 / 12   1 / 2 / 3 / 6 / 10
Gini of per-file counts                 0.318               0.302
```

Per-file histogram (HEAD-rescan): `1:562  2:241  3:95  4:36  5:17  6:9  7:3  8:3  9:2  10:1`

- **58.0% of affected files contain exactly one violation**, carrying 33.4% of the total.
- 82.9% of affected files contain at most two, carrying 62.0% of the total.
- The **top 1% of files hold 4.3%** of the violations; the top 10% hold 25.6%.

For comparison, a Pareto-shaped backlog would put roughly 80% in the top 20% of
files. Here the top 20% hold **41.2%**. There is no file-level hotspot, and no
bounded refactor exists at file granularity. The burn-down curve is close to linear:

```text
fixing the top  10 files removes     79 (  4.7%)
fixing the top  25 files removes    162 (  9.6%)
fixing the top  50 files removes    272 ( 16.1%)
fixing the top 100 files removes    443 ( 26.3%)
fixing the top 200 files removes    709 ( 42.1%)
fixing the top 400 files removes  1,109 ( 65.8%)
```

The worst 15 files:

| n | worst | file | shape mix |
|--:|:--|:--|:--|
| 10 | B | `src/cli/handlers/work_handlers/core_handlers/handlers.rs` | nested 4, flat 3, tangled 3 |
| 9 | B | `src/cli/handlers/configuration_handlers_setters.rs` | flat 9 |
| 9 | B | `src/services/simple_deep_context/language_complexity.rs` | tangled 8, nested 1 |
| 8 | B- | `src/mcp_pmcp/tool_functions/analysis_tools.rs` | tangled 3, flat 3, nested 2 |
| 8 | C | `src/cli/handlers/comply_handlers/check_handlers/check_pv_enforcement_helpers.rs` | tangled 6, nested 2 |
| 8 | D | `src/cli/handlers/comply_handlers/check_handlers/check_contract_surfaces.rs` | tangled 4, nested 4 |
| 7 | B- | `src/cli/handlers/split_auto_handler.rs` | nested 5, tangled 2 |
| 7 | A- | `src/unified_protocol/adapters/cli_helpers.rs` | flat 7 |
| 7 | B+ | `src/services/unrun_tests/cfg.rs` | nested 4, flat 3 |
| 6 | B+ | `src/cli/handlers/comply_handlers/check_handlers/check_commit_enforcement.rs` | flat 4, tangled 2 |
| 6 | B | `src/cli/handlers/comply_handlers/check_handlers/check_commit_enforcement_p4.rs` | tangled 3, flat 2, nested 1 |
| 6 | B- | `src/cli/handlers/comply_cb_detect/rust_best_practices_extended_checks.rs` | tangled 4, nested 2 |
| 6 | B | `src/cli/handlers/comply_cb_detect/markdown_best_practices.rs` | tangled 3, nested 2, flat 1 |
| 6 | B | `src/cli/handlers/comply_cb_detect/rust_best_practices/type_safety.rs` | nested 4, tangled 2 |
| 6 | B- | `src/cli/handlers/comply_cb_detect/rust_best_practices/runtime.rs` | tangled 3, nested 3 |

### Per directory — REFUTED

The flatness is an artifact of granularity. Rolled up to at most three path
components (`src/cli/handlers/x/y.rs` -> `src/cli/handlers`, so a parent bucket holds
only the files sitting directly in it):

```text
  623 (37.0%)  cum  37.0%  src/cli/handlers
  173 (10.3%)  cum  47.2%  src/services
   52 ( 3.1%)  cum  50.3%  src/services/mutation
   44 ( 2.6%)  cum  52.9%  src/services/agent_context
   43 ( 2.6%)  cum  55.5%  src/cli
   40 ( 2.4%)  cum  57.9%  src/tdg
   36 ( 2.1%)  cum  60.0%  src/services/rust_project_score
   23 ( 1.4%)  cum  61.4%  src/services/repo_score
   21 ( 1.2%)  cum  62.6%  src/services/semantic
   20 ( 1.2%)  cum  63.8%  src/services/popper_score
                     ... 104 distinct directories in all
```

**`src/cli/handlers` alone holds 37.0% of the backlog** — 623 violations. Six
buckets hold 58%; the remaining 42% is spread over 98 more. That is a hotspot by any reasonable definition, and the
file-level statistics conceal it completely because the subtree contains 309
affected files averaging 2.02 violations each.

It is also the *worst-shaped* slice, not merely the largest. Comparing the subtree
with the other 1,062 violations (shape classes defined in section 5):

| | `src/cli/handlers` (623) | everywhere else (1,062) |
|:--|--:|--:|
| flat (no deep decision point) | 29.1% | 45.8% |
| nested, depth 2 | 33.1% | 23.4% |
| tangled, depth >= 3 | 35.0% | 25.8% |
| match with nesting | 2.9% | 5.1% |

So the subtree holds 37% of the backlog and a disproportionate share of the part of
it that is real: 218 of the 492 tangled definitions (44%) are in `src/cli/handlers`,
while only 53 of the 363 flat matches (15%) are. One team's surface area, one review,
623 violations — versus 1,062 scattered across the other 103 directories.

---

## 4. The burn-down front — top 30 by worst grade, then complexity

Columns: `cc` cyclomatic as HEAD's scanner counts it; `arms` decision points from
`=>` lines; `deep pts` decision points nested two or more brace levels below the
definition's own body; `max depth` the deepest such level.

| # | grade | cc | loc | arms | deep pts | max depth | shape | definition |
|--:|:--|--:|--:|--:|--:|--:|:--|:--|
| 1 | F | 73 | 96 | 71 | 0 | 1 | flat match | `src/cli/command_wire_names.rs:142` `classify_command` |
| 2 | F | 39 | 63 | 37 | 0 | 1 | flat match | `src/cli/command_wire_names.rs:74` `classify_analyze_command` |
| 3 | F | 39 | 139 | 37 | 0 | 1 | flat match | `src/cli/handlers/analysis_handlers/mod.rs:262` `dispatch_analyze_command` |
| 4 | F | 38 | 69 | 36 | 0 | 1 | flat match | `src/services/deep_wasm/disassembler_formatting.rs:5` `format_operator` |
| 5 | F | 35 | 198 | 4 | 20 | 4 | tangled (d>=3) | `src/cli/handlers/comply_handlers/check_handlers/check_pv_quality.rs:240` `check_codegen_fidelity` |
| 6 | F | 34 | 198 | 2 | 20 | 4 | tangled (d>=3) | `src/cli/handlers/comply_handlers/check_handlers/check_pv_quality.rs:17` `check_precondition_quality` |
| 7 | F | 33 | 101 | 31 | 0 | 1 | flat match | `src/tdg/export_html.rs:5` `score_to_html` |
| 8 | F | 32 | 239 | 10 | 14 | 4 | tangled (d>=3) | `src/cli/handlers/stack_sync_handler.rs:895` `handle_stack_sync` |
| 9 | F | 32 | 280 | 11 | 16 | 6 | tangled (d>=3) | `src/cli/handlers/qa_work_handler/impl_spec.rs:77` `handle_spec` |
| 10 | D | 35 | 44 | 33 | 0 | 1 | flat match | `src/ast/polyglot/node_kind.rs:118` `as_str` |
| 11 | D | 35 | 37 | 33 | 0 | 1 | flat match | `src/services/wasm/types_impls.rs:15` `from` |
| 12 | D | 34 | 43 | 32 | 0 | 1 | flat match | `src/ast/polyglot/node_kind.rs:72` `from_ast_item_kind` |
| 13 | D | 30 | 121 | 13 | 15 | 3 | tangled (d>=3) | `src/ast/languages/lua_visitor.rs:46` `visit_node` |
| 14 | D | 27 | 142 | 8 | 7 | 3 | tangled (d>=3) | `src/cli/handlers/comply_handlers/check_handlers/check_contract_surfaces.rs:34` `check_contract_surface_classification` |
| 15 | D | 26 | 225 | 14 | 6 | 3 | tangled (d>=3) | `src/cli/handlers/deps_audit_handlers/handler.rs:32` `handle_deps_audit` |
| 16 | C- | 31 | 42 | 29 | 0 | 1 | flat match | `src/wasm/security_matcher.rs:6` `matches` |
| 17 | C- | 28 | 43 | 25 | 23 | 3 | match + nesting | `src/services/coverage_improvement/test_generation.rs:291` `generate_strategy_for_type` |
| 18 | C- | 27 | 123 | 16 | 19 | 3 | tangled (d>=3) | `src/mcp_pmcp/tool_functions/context_tools.rs:352` `context_summary` |
| 19 | C- | 27 | 123 | 21 | 12 | 3 | match + nesting | `src/services/context_impl/persistent_analysis.rs:72` `analyze_file_by_toolchain_persistent` |
| 20 | C- | 27 | 121 | 21 | 12 | 3 | match + nesting | `src/services/context_impl/build.rs:218` `analyze_file_by_toolchain` |
| 21 | C- | 26 | 159 | 4 | 15 | 3 | tangled (d>=3) | `src/cli/handlers/comply_handlers/check_handlers/check_commit_enforcement_p8.rs:5` `generate_work_contract_yamls` |
| 22 | C- | 26 | 98 | 24 | 0 | 1 | flat match | `src/cli/handlers/work_falsification/runner.rs:178` `dispatch_falsification_test` |
| 23 | C- | 25 | 111 | 2 | 18 | 5 | tangled (d>=3) | `src/cli/handlers/comply_handlers/check_handlers/check_commit_enforcement_p2.rs:73` `check_hook_single_writer` |
| 24 | C- | 25 | 119 | 23 | 0 | 1 | flat match | `src/services/simple_deep_context/analyzer.rs:289` `analyze_file_complexity` |
| 25 | C- | 22 | 206 | 12 | 13 | 3 | tangled (d>=3) | `src/services/agent_context/function_index/build.rs:66` `build` |
| 26 | C- | 20 | 243 | 0 | 1 | 2 | nested (d=2) | `src/tdg/cuda_simd/scoring_wgpu_detection.rs:30` `detect_wgpu_memory_patterns` |
| 27 | C | 25 | 106 | 12 | 9 | 2 | nested (d=2) | `src/services/mutation_gate.rs:369` `backend_integrity` |
| 28 | C | 24 | 117 | 11 | 11 | 2 | nested (d=2) | `src/ast/languages/python_visitor.rs:31` `visit_node` |
| 29 | C | 23 | 82 | 0 | 14 | 5 | tangled (d>=3) | `src/cli/handlers/comply_handlers/check_handlers/check_pv_enforcement_helpers.rs:398` `count_contract_test_refs` |
| 30 | C | 23 | 153 | 7 | 13 | 4 | tangled (d>=3) | `src/cli/handlers/hooks_command_handlers/command_dispatch.rs:263` `handle_run` |

Two families are visible immediately, and they are not the same problem:

- **Rows 1–4, 7, 10–12, 16, 22, 24** (11 of 30) are single wide `match` expressions:
  arms account for 93–99% of their complexity and **not one has a decision point
  below the first brace level**. `classify_command` is 71 arms in cc 73; its module
  docs already say *"No catch-all arm … A new `Commands` variant must fail to compile
  here."*
- **Rows 5, 6, 8, 9, 13–15, 18, 21, 23, 25, 29, 30** (13 of 30) carry 6–20 decision
  points at depth 3–6. `check_precondition_quality` has 2 match arms and 20 deep
  decision points in 198 lines. That is the shape the gate should be catching.

The stored numbers rank these slightly differently. `src/services/spec_parser_impl.rs:401
categorize_claim` is 6th-worst today at `F`/cc 36 and vanishes entirely under HEAD's
scanner (`A`/cc 5); `handle_deps_audit` moves `F` -> `D`. Cross-reference before
opening a ticket against a stored number.

---

## 5. The honest cut: how much of this is a wide `match`?

**This is an estimate. The method is stated so it can be attacked.**

**Method.** HEAD's `count_complexity` was ported to a script and validated
**1,904 / 1,904 exact against the compiled Rust function**, so every decision point
can be attributed to the trigger and the brace depth that produced it. Two measures
per definition:

- `armshare` — the fraction of decision points coming from `=>` lines.
- `deep` — decision points at two or more brace levels below the definition's body.
  A flat `match` puts every arm at exactly one level; nesting shows up here immediately.

Where the 16,579 decision points above the base come from:

| trigger | count | share |
|:--|--:|--:|
| `=>` (match arms) | 7,075 | 42.7% |
| `if` / `else if` at line start | 5,517 | 33.3% |
| `for` / `while` / `loop` | 1,782 | 10.7% |
| `match` / `switch` head | 906 | 5.5% |
| inline ` if ` | 699 | 4.2% |
| `&&` | 410 | 2.5% |
| boolean `\|\|` | 165 | 1.0% |
| `? ` | 25 | 0.2% |

**Match arms are the single largest contributor — 42.7% of the entire backlog's
complexity.** Adding the `match` heads, 48.2% of every decision point CB-200 counts
comes from a `match`.

Classifying the 1,685 definitions:

| shape | n | share | reading |
|:--|--:|--:|:--|
| flat match (armshare >= 0.75, zero deep points) | 363 | 21.5% | a lookup table |
| flat, other triggers (zero deep points) | 298 | 17.7% | guard clauses, straight-line validation |
| nested, depth 2 | 454 | 26.9% | ordinary code |
| tangled, depth >= 3 | 492 | 29.2% | the real target |
| match with nesting | 72 | 4.3% | mixed |
| flat boolean | 6 | 0.4% | — |

**Sensitivity.** The `flat match` estimate is stable across the plausible range of
the `armshare` cut:

```text
armshare >=    deep==0    deep<=1    deep<=2
      0.60     398 (24%)  415 (25%)  441 (26%)
      0.70     376 (22%)  388 (23%)  405 (24%)
      0.75     363 (22%)  371 (22%)  383 (23%)
      0.80     360 (21%)  364 (22%)  374 (22%)
      0.90     121 ( 7%)  121 ( 7%)  125 ( 7%)
```

The collapse at 0.90 is mechanical, not a signal: the `match` head line itself is one
non-arm point, so a 9-arm match caps at 0.90. Take the estimate as **360–400
definitions, 21–24%**, and treat the 0.90 row as an artifact.

**Answer to the question asked.**

- **~363 definitions (21.5%)** are the `classify_command` shape: a wide, flat,
  total, compiler-checked `match`. They carry 3,671 decision points, 3,314 of them
  arms. Splitting them moves the number and helps no reader; the exhaustiveness check
  is doing work that no smaller function would do better.
- **~492 (29.2%)** are genuinely tangled — three or more decision points nested at
  depth 3 or deeper.
- **~667 (39.6%) have no decision point below the first brace level at all**,
  whatever the trigger. Whether a 12-arm `if`/`else if` ladder counts as "the same
  shape" as a `match` is a judgement: it is flat and readable, but unlike the `match`
  it is *not* total and *not* compiler-checked, so it is left as its own class rather
  than folded into the exempt one.

**Bounds on the estimate.** The lower bound on "cannot usefully be refactored" is the
363 flat matches. The upper bound is the 667 with zero deep points. The genuinely
tangled remainder is 492, plus some fraction of the 454 depth-2 rows. **Roughly a
fifth of this backlog (21.5%) is a metric artifact, roughly three-tenths (29.2%) is
real tangle, and the remaining half is a judgement call** — and none of the three is
a rounding error.

The estimate carries one caveat from section 1b: the ~905 boolean-or continuations
that HEAD's scanner no longer counts sat disproportionately in the `flat, other`
bucket. The flat share is, if anything, understated here.

---

## 6. Recommended ordering

Ordered by evidence-per-unit-effort, not by grade.

**Step 0 — rebuild the index and re-derive the baseline (blocking).**
Nothing below is worth doing against a number produced by a scanner that no longer
exists. Build `pmat` from HEAD, run `pmat query` to rewrite `.pmat/context.db`, and
derive the baseline from that. Expected: **1,685, not 1,904**. Committing 1,904
grants 219 units of unearned headroom.

**Step 1 — record the shape alongside the count.**
The baseline should carry the grade histogram and the flat/nested/tangled split, not
one integer. The whole failure being corrected is that a single number reported
"247 violations" while 1,719 were invisible. A count that cannot distinguish
`classify_command` from `check_precondition_quality` will re-create that failure in a
different key.

**Step 2 — the 15 definitions at `D` or `F`, and the 40 at `C+` or worse.**
Small, bounded, and the only part of the backlog where the grade is doing real work.
Of the top 30, 11 are flat matches and 3 more are matches with some nesting; those
should be *documented as such and dispositioned by an explicit, individually
justified decision* — not by a glob, which is threshold-lowering in disguise. The
other 16 are nested or tangled and are genuine refactors. Cost: tens of
definitions. Value: closes the tail permanently.

**Step 3 — decide the `match` question once, in writing, before touching 363 files.**
Either a wide exhaustive `match` is acceptable at grade A or it is not. If it is,
the decision belongs in the *scorer* — a `match` arm is not a branch a reader must
hold in their head — and that change reduces the backlog by ~20% with no code edits
and no loss of enforcement. If it is not, then `classify_command` needs splitting and
so do 362 others, and that is a 363-definition programme that must be budgeted as
one. What must not happen is 363 ad-hoc splits, each moving the number and none
improving a reader's life.

**Step 4 — `src/cli/handlers` as a single campaign (623, 37%).**
The only real concentration. 309 files, so it is not a refactor — it is a review with
one owner and one standard, and it is the only slice where a single decision moves
more than a third of the total.

**Step 5 — the cheap tail, batched, never individually.**
The distance-to-A distribution, with size held fixed:

```text
 1 branch  : 404 definitions  cum  404 (24.0%)
 2 branches: 295              cum  699 (41.5%)
 3 branches: 244              cum  943 (56.0%)
 4 branches: 152              cum 1,095 (65.0%)
 5 branches: 135              cum 1,230 (73.0%)
```

**404 definitions are one extracted branch away from grade A**, and 429 sit at
exactly cc 8 — half a point below the band. This looks like the cheapest work in the
backlog and it is the most dangerous: extracting one branch from 404 functions to
move a number produces 404 single-use helpers and a worse codebase. Batch it behind
a real motivation (a file being edited anyway, a genuine readability win) and let the
ratchet collect it as a side effect. **Never open a ticket whose body is "reduce cc
from 8 to 7".**

**Step 6 — reconcile the two complexity limits.**
`.pmat-gates.toml` says `max_complexity = 10`; CB-200 at floor A effectively says 7.
1,038 of the 1,685 (61.6%) live in the gap. Until one of those two numbers moves, 62%
of this backlog is a disagreement between two config files rather than a property of
the code.

---

## 7. What the ratchet must record for this to stay honest

1. **The baseline, re-derived after Step 0** — never transcribed, and re-derived by
   running the gate's own code, per `.pmat-ratchet.toml`'s rule that a baseline is
   the output of a command and not a number in a file.
2. **The index build stamp** the baseline was derived from. A baseline measured by
   one scanner and enforced by another is the defect this document opened with.
3. **The absolute count in every report**, passing or failing. 1,685 below-A
   definitions is the fact; "CB-200: Pass" is an inference from it, and the inference
   must never be printed without the fact.
4. **The grade histogram**, so a backlog that is getting *worse in kind* while
   holding steady in count cannot pass. Trading 30 `A-` for 3 `F` is a regression the
   integer cannot see.
5. **A justification requirement on any increase**, checked against the previous
   committed version — as `.pmat-ratchet.toml` already enforces.

None of this changes `min_tdg_grade`, adds an exclude glob, or lowers a threshold.

### Reconciling with `src/services/tdg_baseline.rs`

The ratchet implementation derives its number by calling `check_tdg_grade_gate`
through `PmatYamlConfig::load`, so it honours both exclusion sources and lands on
**1,904** — the same value this document measures independently, by a different
route, in section 0. The two agree, and the `.pmat-gates.toml`-only reading of 1,905
is the one to discard.

That agreement is about *arithmetic*, not about *truth*. Both numbers come from the
same stale index. The open item is section 1: rebuild `.pmat/context.db` with a
binary built from HEAD and re-derive, at which point the honest baseline is expected
to be **1,685**. Committing 1,904 is not wrong today — it is what the gate measures
today — but it will silently overstate the debt by 219 the moment anyone refreshes
the index, and a ratchet with 219 units of slack in it is a ratchet a regression can
hide inside. Section 1's expected value is stated here so the drop, when it comes,
is recognised as a measurement change and not booked as 219 fixes.
