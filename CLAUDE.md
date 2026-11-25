# Claude Code Configuration

Long-form rationale, measurements and incident notes: `docs/agent-rules-history.md` (anchors cited below).

## CRITICAL: Sovereign AI Dependency Policy (80/20 Batuta Stack)

**MANDATORY: Minimize external dependencies - use batuta stack first.** 80% batuta stack, 20% external deps maximum.

Before adding ANY external dependency (math, algorithms, data science, ML, compute):
1. **CHECK BATUTA STACK FIRST** - `pmat query "YourFeature" --limit 5` in aprender
2. **BUILD IF CLOSE** - batuta 70%+ there: extend it
3. **EXTERNAL ONLY AS LAST RESORT** - document why batuta could not work

Use instead: `aprender` (ML, stats, graph; not nalgebra/linfa/smartcore), `trueno` (SIMD/GPU; not ndarray), `trueno-graph` (not petgraph), `trueno-db` (not polars/datafusion), `trueno-rag` (not qdrant/milvus), `trueno-viz` (not plotters), `trueno-zram-core` (not lz4/zstd), `renacer`, `certeza`, `bashrs` (not shellcheck), `probar` (not quickcheck), `pmcp`, `presentar-core` (not ratatui).

Reviewed externals, keep: `roaring`, `rand`, `rayon`. `nalgebra-sparse` removed (`Cargo.toml:251`): do not re-add.

---

## CRITICAL: Code Search Policy

**NEVER use grep/glob for code search. ALWAYS use `pmat query`.** Always add `--faults` when auditing for unwrap/panic/unsafe.

```bash
pmat query "error handling" --limit 10            # by intent
pmat query "unwrap" --faults --exclude-tests      # fault patterns
pmat query "dispatch" --churn --duplicates --entropy --faults -G   # full enrichment
pmat query --regex "fn\s+handle_\w+" --limit 10   # like rg -e
pmat query --literal "unwrap()" --limit 10        # like rg -F
```


Flags: `--exclude`, `--exclude-file`, `-i`, `--files-with-matches`, `--count`, `-A/-B/-C`, `-G`, `--churn`, `--duplicates`, `--entropy`, `--faults` (details: history). MCP tools: `pmat_query_code`, `pmat_get_function`, `pmat_find_similar`, `pmat_index_stats`.

**MANDATORY coverage work: `pmat query --coverage-gaps`. NEVER `make coverage` or raw `cargo llvm-cov` output.**

```bash
pmat query --coverage-gaps --limit 30 --exclude-tests
pmat query --coverage-gaps --rank-by impact --limit 20
pmat query "parse" --coverage --uncovered-only --limit 10
```

Workflow: run `--coverage-gaps`; pick highest impact (missed_lines * pagerank / complexity); read with `pmat query "function_name" --include-source --limit 1`; write tests for uncovered lines; re-run to verify.

**CRITICAL: exploring code to write tests: use `pmat query --include-source`, NOT `Read`/`cat`/`grep`.**

grep is acceptable only for non-code files (TOML, YAML, Markdown) and quick one-off line matches while debugging.


---

## CRITICAL: pmat-book Validation and Push Policy (Toyota Way - Jidoka)

**MANDATORY BEFORE ANY RELEASE OR VERSION BUMP:** `make validate-book`

- Critical chapters in parallel (Ch 05, 07, 13, 14; `scripts/validate-pmat-book.sh:28`), 60s timeout each. Chapter 13 (Multi-Language) must always pass.
- Book checkout at `/home/noah/src/pmat-book`; override `PMAT_BOOK_DIR=<path>`. Absent dir = the script skips, not fails: green proves nothing there.
- Tests fail: fix code OR update book tests. Andon Cord: STOP on quality issues.
- **MANDATORY: book updates MUST be pushed with code changes.** Enforced only by hooks from `bash scripts/install-git-hooks.sh`: pre-commit warns (`scripts/install-git-hooks.sh:128`); pre-push **BLOCKS `git push`** until pmat-book commits are pushed (`scripts/install-git-hooks.sh:238`). `pmat hooks install` hooks have neither check: grep `.git/hooks/pre-push` for `pmat-book`.
- Workflow: update pmat-book, push to main (deploys Pages), push code. Before `cargo publish`: book pushed, `make validate-book` passes, Pages deployed.

---

## CRITICAL: Pre-Commit Verification (`pmat verify`)

**MANDATORY for agents: run `pmat verify` before every commit** (format, complexity, satd, clippy, tests; fail-fast). The pre-commit hook and `pmat quality-gate` miss clippy + tests.

```bash
pmat verify --format json          # ok:true safe to commit; ok:null = no verdict (see not_measured[])
pmat verify --fix                  # auto-apply cargo fmt / clippy --fix first
pmat verify --skip clippy,tests    # fast inner loop
```

Loop: `edit -> pmat verify --format json -> fix on red -> commit on green`. Spec: `docs/specifications/pmat-verify-autonomous-preflight.md`; `docs/agent-instructions/autonomous-verify-loop.md`.

---

## Build-Budget Metrics (`.pmat-metrics.toml`) — RECORDED, NOT ENFORCED

**Do not treat `[thresholds]` in `.pmat-metrics.toml` as gates.** Budgets: lint <=150s, test-fast <=360s, coverage <=600s, binary <=50MB, deps <=3,000.
- Only `make coverage` and `make build-release` call `scripts/record-metric.sh`. Binary size is enforced by a hardcoded value in `src/tests/binary_size.rs:40`.
- Really read: `[exclude]`, `[entropy]`, `provability_min`, `entropy_min_diversity` (by `pmat quality-gate`; files in history).
- To learn what enforces what, run `pmat comply coherence` (`--format json`): FIRING, VIOLATED or VACUOUS per threshold. Bindings: `.pmat-ratchet.toml` `[coherence.binding.*]`; correct claims there.
- Real pre-commit gate set is `pmat verify`. Emergency bypass: `git commit --no-verify`.

Why: nothing reads most keys (0 FIRING, 1 VIOLATED, 16 VACUOUS at `331017130`). History: docs/agent-rules-history.md#build-budget-metrics

### `.pmat-ratchet.toml` — the numbers that CAN fail a build (CB-2102)

- BASELINES asserted by comply rule CB-2102 and `--lib` test `the_committed_ratchet_holds_at_head`; also holds `[coherence.binding.*]` for CB-2101.
- A metric measuring **0 against a baseline above 0** is UNMEASURABLE, not passed; set `zero_is_reachable = true` when the zero is real. An unmeasurable metric FAILS.
- Each entry carries the command that reproduces its baseline; the gate RUNS it.

```bash
pmat comply ratchet            # judge baselines; non-zero on regression
pmat comply ratchet --lower    # rewrite baselines the tree already beat
```

- A metric may only get better; raising a baseline needs a `justification` on that entry. Engine `src/services/metrics_ratchet/`; contract `contracts/comply-ratchet-v1.yaml`.

Why: `max_unwrap_calls = 100` vs 20,390 measured, green build. History: docs/agent-rules-history.md#pmat-ratchettoml

---

## CRITICAL: Documentation Accuracy Enforcement (Zero Hallucinations)

**MANDATORY FOR README.md, CLAUDE.md, GEMINI.md, AGENT.md:**

```bash
pmat context --output deep_context.md --format llm-optimized      # ~25s
pmat validate-readme \
    --targets README.md CLAUDE.md \
    --deep-context deep_context.md \
    --fail-on-contradiction --verbose                             # --deep-context REQUIRED
```

- **No docs-accuracy check on `pmat quality-gate`** (`--checks` accepts only dead-code, complexity, coverage, sections, provability, satd, entropy, security, duplicates, all). `pmat validate-readme` is the whole surface.
- Runs only in the pre-commit hook from `bash scripts/install-git-hooks.sh` (`scripts/install-git-hooks.sh:98`, blocking on contradictions); in no `.github/workflows/` job, and not in `pmat hooks install` hooks.
- `validate-readme` can skip refs (a clean run is a floor). After editing this file run both, expected output: nothing:

```bash
grep -oE '`[A-Za-z0-9_./()-]+\.(md|rs|sh|toml)(:[0-9]+)?`' CLAUDE.md \
  | tr -d '`' | sed 's/:[0-9]*$//' | sort -u \
  | while read -r p; do test -e "$p" || echo "DEAD PATH: $p"; done
{ sed -e ':a' -e '/\\$/{N;s/\\\n//;ba}' CLAUDE.md | grep -oE '^pmat [^#|>]+'
  grep -oE '`pmat [^`]+`' CLAUDE.md | tr -d '`'
} | sed 's/[[:space:]]*$//' | grep -vE '[][^…\\]' | sort -u \
  | while read -r c; do eval "$c --help" >/dev/null 2>&1 || echo "BAD COMMAND: $c"; done
```

Why: pre-fix revision had 8 dead paths, 1 bad command. History: docs/agent-rules-history.md#documentation-accuracy

---

## Bash/Makefile Quality Enforcement with bashrs

**Bash scripts and Makefiles should pass bashrs linting** (SC2086/SC2046/SC2116, DET003, IDEM002, SEC008).

```bash
make lint-makefile                     # what the repo runs
bashrs lint Makefile --ignore MAKE003,MAKE006,MAKE010,MAKE012,MAKE017,MAKE018
```

`make lint-makefile` ends `|| true` (`Makefile:898`): non-blocking; suppressions in `.bashrsignore`. `make lint-scripts` is deno/TypeScript, no bashrs. Install: `cargo install bashrs` (`pmat hooks install --tdg-enforcement` does not install bashrs). Bugs: https://github.com/paiml/bashrs/issues

---

## Coverage Tool Policy

**Use `cargo llvm-cov` exclusively. NEVER use cargo-tarpaulin.**

## Test Coverage

Tests marked `#[ignore]` do not run in `cargo test` and not in coverage. **Measure the count, never quote it:**

```bash
git grep -cE '^[[:space:]]*#\[ignore' -- src   | awk -F: '{n+=$2} END{print n+0}'
git grep -cE '^[[:space:]]*#\[ignore' -- tests | awk -F: '{n+=$2} END{print n+0}'
env -u RUST_MIN_STACK cargo test --lib -- --ignored --list   # authoritative, lib only
```

An `#[ignore]` is a silently unmeasured test: fix it and remove the attribute, or record why it stays next to it. Branching policy is in the global instructions (feature branch + PR); do not work on master. History: docs/agent-rules-history.md#test-coverage

---

## PMAT Five Whys Root Cause Analysis (Toyota Way)

`pmat five-whys` (aliases `why`, `debug-whys`). **This is the ONLY acceptable debugging method.**

```bash
pmat five-whys "Stack overflow in parser"              # default --depth 5
```

Options: `--depth <1-10>`, `--format <text|json|markdown>`, `--output <FILE>`, `--path <PATH>`. **`--auto-analyze` and `--context <FILE>` are accepted but do nothing** (warn "not yet implemented"): build no workflow on them. Evidence weights: history.

---

## Rust Project Score v3.0

`pmat rust-project-score` (alias `rust-score`): 289 points, 11 categories.

```bash
pmat rust-project-score                    # fast (~2-3 min)
pmat rust-project-score --full             # ~10-15 min, all checks
```

11 categories; breakdown, spec `docs/specifications/components/repo-health.md`, code `src/services/rust_project_score/`: history.

---

## CRITICAL: Renacer Golden Tracing

**MANDATORY for**: transpilers, distributed systems, multi-process workflows, cross-language integrations. Always validate golden traces before completing work. Config: `renacer.toml`.

```bash
renacer validate --generate <DIR> -- <command>    # create baseline
renacer validate --baseline <DIR> -- <command>    # compare
```


---

## trueno-graph O(1) Context and TDG Integration

ACTIVE, not feature-gated. CSR graph for O(1) symbol lookups and PageRank importance.
1. Context: `analyze_project_with_cache()` (`src/services/context_impl/visitor.rs:473`) calls `build_context_graph()`; type `src/services/context_graph.rs`.
2. TDG: `src/tdg/tdg_graph.rs` (O(1) dependency tracking + PageRank).


---

## DETERMINISTIC Agent Instructions

Follow **`docs/agent-instructions/`**: `pmat-work-ux-fixes.md` (fuzzy IDs, status display, quality gates, short IDs), `pmat-work-quality-principles.md` (Five Whys, Renacer, Rust requirements, commit metadata). Workflow: read doc, fix in priority order, test each, commit atomically.

## Stack Documentation Search

```bash
batuta oracle --rag-index                    # once
batuta oracle --rag "your question here"
batuta oracle --rag-stats                    # freshness
batuta oracle --rag-index-force              # force reindex
```

`--rag-index --force` errors; `ora-fresh` is a personal alias (use `--rag-stats`). Verified batuta 0.7.3.

## Compliance (CB-130)

`pmat comply check` validates: index at `.pmat/context.idx` fresh (<24h); CLAUDE.md contains `pmat query`, `NEVER use grep`, `--faults`; index auto-built on first `pmat query`.
