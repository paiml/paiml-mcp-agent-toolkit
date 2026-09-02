# Claude Code Configuration

## CRITICAL: Sovereign AI Dependency Policy (80/20 Batuta Stack)

**MANDATORY: Minimize external dependencies - use batuta stack first**

PMAT follows the Sovereign AI philosophy: **80% batuta stack, 20% external deps maximum**.

Before adding ANY external dependency for math, algorithms, data science, ML, or compute:
1. **CHECK BATUTA STACK FIRST** - See if sovereign tools already provide the functionality
2. **BUILD IF CLOSE** - If batuta stack is 70%+ there, extend it rather than adding external dep
3. **EXTERNAL ONLY AS LAST RESORT** - Document why batuta stack couldn't work

### Batuta Stack (Sovereign AI Tools)

| Crate | Purpose | Use Instead Of |
|-------|---------|----------------|
| `aprender` | ML, stats, graph algorithms, text similarity | nalgebra, linfa, smartcore |
| `trueno` | SIMD/GPU compute, matrix ops | ndarray, nalgebra |
| `trueno-graph` | Graph database, PageRank, Louvain | petgraph, graph |
| `trueno-db` | Columnar storage, analytics | polars, datafusion |
| `trueno-rag` | RAG pipeline, vector search | qdrant, milvus |
| `trueno-viz` | Terminal visualization | plotters, textplots |
| `trueno-zram-core` | SIMD compression | lz4, zstd |
| `renacer` | Golden tracing, chaos testing | proptest chaos |
| `certeza` | Quality validation | custom scripts |
| `bashrs` | Bash/Makefile linting | shellcheck |
| `probar` | Property-based testing | quickcheck |
| `pmcp` | MCP protocol SDK | custom MCP |
| `presentar-core` | TUI framework | ratatui |

### Dependencies Requiring Review

| External Dep | Status | Batuta Alternative |
|--------------|--------|-------------------|
| `roaring` | Keep | Specialized bitmap (no batuta equivalent yet) |
| `rand` | Keep | Foundational (may add to trueno later) |
| `rayon` | Keep | Foundational parallel iterator |

(`nalgebra-sparse` was removed — see the note at `Cargo.toml:251`. Do not re-add it.)

Before adding ANY new dependency: check batuta stack first (`pmat query "YourFeature" --limit 5` in aprender), build if close, document if external required.

---

## CRITICAL: Code Search Policy

**NEVER use grep/glob for code search. ALWAYS use `pmat query`.**

| Task | Command |
|------|---------|
| Find functions by intent | `pmat query "error handling" --limit 10` |
| Find high-quality examples | `pmat query "serialize" --min-grade A` |
| Find simple implementations | `pmat query "cache" --max-complexity 10` |
| Find important functions | `pmat query "dispatch" --rank-by pagerank` |
| Find with fault patterns | `pmat query "unwrap" --faults --exclude-tests` |
| Cross-project search | `pmat query "simd" --include-project ../trueno` |
| Include source code | `pmat query "tokenize" --include-source` |
| Search by commit intent | `pmat query "fix memory leak" -G` |
| Find volatile hot code | `pmat query "cache" --churn` |
| Find code clones | `pmat query "serialize" --duplicates` |
| Find repetitive patterns | `pmat query "handler" --entropy` |
| Full enrichment | `pmat query "dispatch" --churn --duplicates --entropy --faults -G` |
| Regex search (like rg -e) | `pmat query --regex "fn\s+handle_\w+" --limit 10` |
| Literal string search (like rg -F) | `pmat query --literal "unwrap()" --limit 10` |
| Exclude pattern (like grep -v) | `pmat query "handler" --exclude "test"` |
| Exclude files by glob | `pmat query "cache" --exclude-file "tests"` |
| Case-insensitive search | `pmat query "Error" -i` |
| Files with matches (like rg -l) | `pmat query "handler" --files-with-matches` |
| Count matches per file (like rg -c) | `pmat query "unwrap" --count` |
| Context lines (like grep -C) | `pmat query "panic" -A 3 -B 2` |

### Search Mode Flags

- **`--regex`** — Regex pattern matching against function name, signature, and source. Uses Rust regex syntax.
- **`--literal`** — Exact literal string match (no semantic ranking). Like `rg -F`.
- **`--case-sensitive`** — Force case-sensitive matching (default: smart-case like rg).
- **`-i` / `--ignore-case`** — Force case-insensitive matching.
- **`--exclude PATTERN`** — Exclude results matching content pattern (like `grep -v`).
- **`--exclude-file GLOB`** — Exclude results from files matching glob pattern.
- **`--files-with-matches`** — Output only unique file paths (like `rg -l`).
- **`--count`** — Output match count per file (like `rg -c`).
- **`-A N` / `-B N` / `-C N`** — Show N lines of context after/before/around matches.

### Enrichment Flags

- **`-G` / `--git-history`** — Fuse git commit history via RRF. Finds code by intent via commit message semantic search.
- **`--churn`** — Git volatility metrics (90-day window). Hot files (>50% churn) flagged.
- **`--duplicates`** — Code clone detection via MinHash + LSH.
- **`--entropy`** — Pattern diversity metrics. Low (<30%) = boilerplate; high (>80%) = unique.
- **`--faults`** — Batuta fault pattern annotations (unwrap, panic, unsafe, etc.).

### Coverage Gap Analysis

**MANDATORY: Use `pmat query --coverage-gaps` for coverage work. NEVER use `make coverage` or raw `cargo llvm-cov` output.**

```bash
# Find top coverage gaps ranked by uncovered lines
pmat query --coverage-gaps --limit 30 --exclude-tests

# Find coverage gaps ranked by ROI (impact score)
pmat query --coverage-gaps --rank-by impact --limit 20

# Coverage-enriched semantic search
pmat query "error handling" --coverage --limit 10

# Find only uncovered functions
pmat query "parse" --coverage --uncovered-only --limit 10
```

**Dogfooding workflow for improving coverage:**
1. Run `pmat query --coverage-gaps --limit 30 --exclude-tests` to identify targets
2. Pick functions with highest impact score (missed_lines * pagerank / complexity)
3. Use `pmat query "function_name" --include-source --limit 1` to read the function
4. Write tests targeting the uncovered lines
5. Re-run `pmat query --coverage-gaps` to verify improvement

**CRITICAL: When exploring code to write tests, use `pmat query` with `--include-source`, NOT `Read`/`cat`/`grep`.**

### When grep IS acceptable
- Searching non-code files (TOML, YAML, Markdown)
- Quick one-off during debugging when you need exact line matches
- NOTE: `pmat query --literal` and `pmat query --regex` now cover most grep/rg use cases

### MCP Tools

| Tool | Use Case |
|------|----------|
| `pmat_query_code` | Semantic search by intent |
| `pmat_get_function` | Get full function with metrics |
| `pmat_find_similar` | Find similar functions (refactoring) |
| `pmat_index_stats` | Index health and statistics |

---

## CRITICAL: pmat-book Validation Policy (Toyota Way - Jidoka)

**MANDATORY BEFORE ANY RELEASE OR VERSION BUMP:**

```bash
# Fast, parallel, fail-fast validation (recommended)
make validate-book
```

- Runs critical chapters in parallel (Ch 05, 07, 13, 14 — `scripts/validate-pmat-book.sh:28`), 60s timeout per chapter
- Chapter 13 (Multi-Language) is CRITICAL - must always pass
- The book checkout is looked for at `/home/noah/src/pmat-book`; override with `PMAT_BOOK_DIR=<path>`. If the directory is absent the script **skips** rather than fails, so a green `make validate-book` on a machine without the book proves nothing
- If tests fail, fix code OR update book tests. Apply Andon Cord: STOP if quality issues found

---

## CRITICAL: pmat-book Push Enforcement Policy

**MANDATORY: Book updates MUST be pushed with code changes**

This is enforced only by the hooks written by `bash scripts/install-git-hooks.sh`:

- **Pre-Commit Hook**: Warns about unpushed pmat-book commits (non-blocking) — `scripts/install-git-hooks.sh:128`
- **Pre-Push Hook**: **BLOCKS `git push`** until all pmat-book commits are pushed first — `scripts/install-git-hooks.sh:238`

**The hooks `pmat hooks install` writes do NOT contain either check** — they run format/complexity/SATD only. Two installers, two different hook sets: check `.git/hooks/pre-push` for the string `pmat-book` before relying on this gate.

**Workflow**: Update pmat-book → push to main (deploys GitHub Pages) → push code.

**crates.io Release**: Ensure all pmat-book changes pushed, `make validate-book` passes, and GitHub Pages deployment completed before `cargo publish`.

---

## CRITICAL: Pre-Commit Verification (`pmat verify`)

**MANDATORY for agents: run `pmat verify` before every commit.** It runs the gate
set CI actually enforces — **format, complexity, satd, clippy, tests** — fail-fast,
with machine-readable output, giving a "green here ⇒ green in CI" guarantee. The
pre-commit hook and `pmat quality-gate` both miss clippy + tests; `pmat verify`
does not.

```bash
pmat verify --format json          # canonical agent check; ok:true ⇒ safe to commit; ok:null ⇒ no verdict (a stage declined — see not_measured[])
pmat verify --fix                  # auto-apply cargo fmt / clippy --fix first
pmat verify --skip clippy,tests    # fast inner loop (format+complexity+satd)
```

Canonical loop: `edit → pmat verify --format json → fix on red → commit on green`.
Spec: `docs/specifications/pmat-verify-autonomous-preflight.md`. Loop doc:
`docs/agent-instructions/autonomous-verify-loop.md`.

---

## Build-Budget Metrics (`.pmat-metrics.toml`) — RECORDED, NOT ENFORCED

**Do not treat these as gates.** The `[thresholds]` table in `.pmat-metrics.toml` is a
budget document. Grep for its keys before believing otherwise:

```bash
pmat query --literal "lint_max_ms" --limit 5                              # code
grep -n "lint_max_ms\|test_fast_max_ms\|coverage_max_ms\|deps_default_max" \
     Makefile scripts/*.sh .github/workflows/*.yml                        # non-code
```

The only code hit is a test fixture (`src/cli/analysis_utilities/quality_gate_part2f.rs:41`),
and there are no non-code hits at all — no hook, no Makefile target and no CI job reads any
timing threshold, so nothing can fail on one. Current budgets, as actually written in the file:

- **lint**: ≤150s | **test-fast**: ≤360s | **coverage**: ≤600s | **binary size**: ≤50MB | **dependencies**: ≤3,000

What *is* real:

- **Recording**: only `make coverage` and `make build-release` call `scripts/record-metric.sh`. `make lint` and `make test-fast` record nothing.
- **Binary size**: enforced by a hardcoded `50 * 1024 * 1024` in `src/tests/binary_size.rs:40` — 52,428,800, against the 50,000,000 this file declares, under a comment claiming the two are "aligned". They are 2.43 MB apart and neither reads the other.
- **`[exclude]` and `[entropy]`**: these two sections of `.pmat-metrics.toml` *are* read, by `pmat quality-gate` (`src/cli/analysis_utilities/quality_checks_part1_complexity.rs:120` and `src/cli/analysis_utilities/quality_checks_part1_entropy.rs:220`).
- **`provability_min` and `entropy_min_diversity`**: also genuinely read, by `src/cli/analysis_utilities/quality_gate_config.rs`. These are the only two `[thresholds]` keys anything parses.
- **CB-2101 now classifies all of them.** `pmat comply coherence` audits every scalar in `[thresholds]`, `[quality_gates]` and `[performance]` against a live measurement and reports each as FIRING, VIOLATED or VACUOUS. Do not re-derive the list by grepping — run it:

```bash
pmat comply coherence                 # one row per threshold, with the reason
pmat comply coherence --format json   # limit, live measurement, band, verdict
```

  At `331017130` that is **17 thresholds: 0 FIRING, 1 VIOLATED, 16 VACUOUS**. The one
  VIOLATED is `quality_gates.max_unwrap_calls = 100` against a measured 20,390. Every
  binding, and the evidence behind it, is in `.pmat-ratchet.toml` under
  `[coherence.binding.*]` — that file, not this one, is the place to correct a claim
  about what enforces what.

The real pre-commit gate set is the one described under **Pre-Commit Verification** above;
run `pmat verify`. Emergency bypass for the hooks that do exist: `git commit --no-verify`.

### `.pmat-ratchet.toml` — the numbers that CAN fail a build (CB-2102)

Do not confuse the two files. `.pmat-metrics.toml` records budgets nothing reads.
`.pmat-ratchet.toml` records BASELINES, and two things assert them: the comply rule
CB-2102, and the `--lib` test `the_committed_ratchet_holds_at_head`. It also carries the
`[coherence.binding.*]` table CB-2101 reads, so it is the single place that says which
of the two files' numbers are enforced and by what.

A metric that measures **0 against a baseline above 0** is reported UNMEASURABLE, not
passed: a `git grep` pathspec that has rotted and a genuine zero are byte-identical at
the shell (both print `0`, both exit 1, neither writes to stderr), so the gate refuses to
guess. Declare `zero_is_reachable = true` on the metric when the zero is real.

Every entry carries the exact command that reproduces its baseline, and the gate RUNS
that command rather than reading the number — so a baseline can never quietly become a
transcription:

```bash
pmat comply ratchet            # judge the baselines; non-zero when one regressed
pmat comply ratchet --lower    # rewrite every baseline the tree has already beaten
```

A metric may only get better. Raising a baseline requires a `justification` on that entry,
checked against the previous committed version of the file. A metric that could not be
measured FAILS — "we could not measure it" must never read as "it did not regress".
Engine: `src/services/metrics_ratchet/`. Contract: `contracts/comply-ratchet-v1.yaml`.

Why the difference matters: `.pmat-metrics.toml:45` declares `max_unwrap_calls = 100`
with the inline comment `Current: 570`, in a tree that measures 20,390 by the predicate
`.pmat-ratchet.toml` pins. Three numbers, no two of which agree, and a green build
throughout, because nothing reads the key.

---

## CRITICAL: Documentation Accuracy Enforcement (Zero Hallucinations)

**MANDATORY FOR README.md, CLAUDE.md, GEMINI.md, AGENT.md:**

```bash
# Step 1: Generate deep context (~25s)
pmat context --output deep_context.md --format llm-optimized

# Step 2: Validate documentation accuracy (--deep-context is REQUIRED)
pmat validate-readme \
    --targets README.md CLAUDE.md \
    --deep-context deep_context.md \
    --fail-on-contradiction --verbose
```

Validates: capability claims against AST, file path references, function/module references, external URLs (404 detection). Uses Semantic Entropy (Nature 2024) and MIND framework (IJCAI 2025).

**There is no docs-accuracy check on `pmat quality-gate`.** Its `--checks` argument accepts
only: dead-code, complexity, coverage, sections, provability, satd, entropy, security,
duplicates, all. Passing docs-accuracy exits 2 with a clap error. `pmat validate-readme`
above is the whole enforcement surface.

**Where it actually runs**: the pre-commit hook written by `bash scripts/install-git-hooks.sh`
(`scripts/install-git-hooks.sh:98`, blocking on contradictions). It is **not** in any
`.github/workflows/` job, and the hook that `pmat hooks install` writes does not run it —
so on a fresh clone nothing checks this file until you invoke it by hand.

`validate-readme` does not extract every reference. It silently skipped a dead path whose
filename contained parentheses in this very document, so a clean run is a floor, not a
proof. Cheap belt-and-braces check for the paths cited here:

```bash
# every backtick-quoted repo path in CLAUDE.md must exist (":42" suffix allowed)
grep -oE '`[A-Za-z0-9_./()-]+\.(md|rs|sh|toml)(:[0-9]+)?`' CLAUDE.md \
  | tr -d '`' | sed 's/:[0-9]*$//' | sort -u \
  | while read -r p; do test -e "$p" || echo "DEAD PATH: $p"; done
```

And for the commands — `validate-readme` checks no command at all, which is how
`quality-gate --checks docs-accuracy` survived here for releases:

```bash
# every pmat invocation cited in CLAUDE.md must at least parse
{ sed -e ':a' -e '/\\$/{N;s/\\\n//;ba}' CLAUDE.md | grep -oE '^pmat [^#|>]+'
  grep -oE '`pmat [^`]+`' CLAUDE.md | tr -d '`'
} | sed 's/[[:space:]]*$//' | grep -vE '[][^…\\]' | sort -u \
  | while read -r c; do eval "$c --help" >/dev/null 2>&1 || echo "BAD COMMAND: $c"; done
```

(The `grep -vE` drops this snippet's own regex text from its input.)

Expected output from both: nothing. Run them after editing this file. On the pre-fix
revision of this document they print 8 dead paths and 1 bad command.

---

## Bash/Makefile Quality Enforcement with bashrs

**Bash scripts and Makefiles should pass bashrs linting.**

bashrs (PAIML) lints for SC2086/SC2046/SC2116, DET003 (non-determinism), IDEM002 (idempotency), SEC008 (security).

```bash
make lint-makefile                     # what the repo actually runs
bashrs lint Makefile --ignore MAKE003,MAKE006,MAKE010,MAKE012,MAKE017,MAKE018
bashrs lint scripts/<script>.sh        # ad-hoc, for a single script
```

`make lint-makefile` ends in `|| true` (`Makefile:898`) — bashrs findings are reported but
**non-blocking**; intentional suppressions live in `.bashrsignore`. `make lint-scripts` is
deno/TypeScript, not bash, and does not invoke bashrs.

Installation: `cargo install bashrs` (`pmat hooks install --tdg-enforcement` installs pmat's
own hooks; it does not install or wire up bashrs). Bug reports: https://github.com/paiml/bashrs/issues

---

## Coverage Tool Policy

**Use `cargo llvm-cov` exclusively. NEVER use cargo-tarpaulin.**

---

## Test Coverage

Tests marked `#[ignore]` do not run in `cargo test` and therefore do not appear in coverage.
The count is large and moves every release, so **measure it, never quote it**:

```bash
git grep -cE '^[[:space:]]*#\[ignore' -- src   | awk -F: '{n+=$2} END{print n+0}'
git grep -cE '^[[:space:]]*#\[ignore' -- tests | awk -F: '{n+=$2} END{print n+0}'
env -u RUST_MIN_STACK cargo test --lib -- --ignored --list   # authoritative, lib only
```

At `fcb1eb45d` that is **341 in `src/` and 816 in `tests/`** — an earlier revision of this
section claimed "~94 total (82 in src/)" together with a 14-row breakdown by category; both
were off by more than 10x and the breakdown is not reconstructible from the tree, so it has
been deleted rather than guessed at.

An `#[ignore]` is a silently unmeasured test. When you touch one, either fix it and remove
the attribute or record why it must stay ignored next to it.

Branching policy lives in the global instructions (feature branch + PR); this file used to
say "always work on master", which contradicts them. Follow the global instructions.

---

## PMAT Five Whys Root Cause Analysis (Toyota Way)

**Command**: `pmat five-whys` (aliases: `why`, `debug-whys`) | **Status**: Production-ready

Evidence-based root cause analysis using Toyota Way Five Whys. **This is the ONLY acceptable debugging method.**

```bash
pmat five-whys "Stack overflow in parser"              # Basic (default --depth 5)
pmat why "Memory leak in cache" --depth 3              # Short alias
pmat five-whys "Test failures" --format json -o out.json  # JSON output
pmat five-whys "Perf regression" --format markdown
```

Options: `--depth <1-10>` (default 5), `--format <text|json|markdown>`, `--output <FILE>`, `--path <PATH>`.

**`--auto-analyze` and `--context <FILE>` are accepted but do nothing.** Both print
`Warning: ... is not yet implemented. Flag ignored.` and produce byte-identical output —
verified by diffing full JSON with and without. Do not build a workflow on them.

Evidence weights (v2, PMAT-510 — `calculate_confidence` in `src/services/five_whys_analyzer.rs`):
IssueLocation 35%, Complexity 25%, SATD 20%, GitChurn 15%, EvoScoreTrajectory 15%,
CoverageDelta 15%, ManualInspection 15%, DeadCode 10%. **TDG is weighted 0%** — removed as
redundant with complexity+churn. Without at least one issue-specific `IssueLocation`
evidence item the confidence score is capped (`NO_ISSUE_EVIDENCE_CEILING`), because every
other source is a repo-wide metric that is identical whatever issue you typed.

---

## Rust Project Score v3.0

**Command**: `pmat rust-project-score` (alias: `rust-score`) | **Status**: Production-ready

289-point scoring across 11 categories based on 15 peer-reviewed papers (2022-2025).

```bash
pmat rust-project-score                    # Fast mode (~2-3 min, skips clippy/mutation/build)
pmat rust-project-score --full             # Full mode (~10-15 min, all checks)
pmat rust-project-score --format json -o score.json  # CI/CD
pmat rust-project-score --failures-only    # Show only failures
```

**Categories**: Rust Tooling & CI/CD (130pts), Code Quality (26pts), Testing (20pts), Known Defects (20pts), Formal Verification (16pts), Documentation (15pts), Reproducibility (15pts), Build Performance (15pts), Dependency Health (12pts), Performance & Benchmarking (10pts), GPU/SIMD Quality (10pts).

Spec: `docs/specifications/components/repo-health.md` | Location: `src/services/rust_project_score/`

---

## CRITICAL: Renacer Golden Tracing

**MANDATORY for**: Transpilers, distributed systems, multi-process workflows, cross-language integrations.

```bash
renacer validate --generate <DIR> -- <command>    # Create a golden baseline
renacer validate --baseline <DIR> -- <command>    # Compare against it
```

renacer 0.10.2 has exactly two subcommands, `validate` and `visualize`; there is no
`renacer capture` and no `renacer validate --all` (both were documented here and both
error out). `validate` traces a command you pass after `--`; exit codes: 0=passed,
1=failed, 2=baseline not found, 3=invalid baseline, 4=command error, 5=config error.

Config: `renacer.toml` in project root. Always validate golden traces before completing work.

---

## trueno-graph O(1) Context and TDG Integration

**STATUS**: ACTIVE (NOT feature-gated, used in production)

trueno-graph provides CSR graph database for O(1) symbol lookups and PageRank-based importance scoring.

**Integrations**:
1. **Context Generation**: `analyze_project_with_cache()` (`src/services/context_impl/visitor.rs:473`) calls `build_context_graph()` at line 483, so every cached project analysis builds a `ProjectContextGraph`. Type: `src/services/context_graph.rs`; tests: `src/services/context_graph_tests.rs` plus `context_graph_coverage_{core,extended,fixtures}.rs`.
2. **TDG Analysis**: `src/tdg/tdg_graph.rs` — TdgGraph provides O(1) function dependency tracking with PageRank criticality; tests in `src/tdg/tdg_graph_tests.rs`.

**Architecture**: Dual storage pattern - HashMap (O(1) lookups) + CSR graph (PageRank) + bidirectional NodeId mapping.

**Key insight**: CSR graphs only track nodes with edges; `num_nodes()` returns node_map.len() not graph.num_nodes().

---

## DETERMINISTIC Agent Instructions

Follow instructions in **`docs/agent-instructions/`** for deterministic fixes:

1. **`docs/agent-instructions/pmat-work-ux-fixes.md`** - Fuzzy ID matching, status display, quality gates, short IDs
2. **`docs/agent-instructions/pmat-work-quality-principles.md`** - Five Whys, Renacer tracing, Rust project requirements, commit metadata

Workflow: Read instruction doc → apply fixes in priority order → test each fix → commit atomically.

---

## Stack Documentation Search

```bash
batuta oracle --rag-index                    # Index all stack docs (once)
batuta oracle --rag "your question here"     # Search across stack
batuta oracle --rag-stats                    # Check index status / freshness
batuta oracle --rag-index-force              # Force reindex (clears cache first)
```

Force reindex is the single flag `--rag-index-force`; `--rag-index --force` errors
(`unexpected argument '--force'`). `ora-fresh` is a personal shell alias, not part of this
repo or of batuta — use `batuta oracle --rag-stats` instead. Verified against batuta 0.7.3.

---

## Compliance (CB-130)

CB-130 validates agent context adoption via `pmat comply check`:
- Index exists at `.pmat/context.idx` and is fresh (< 24 hours)
- CLAUDE.md contains required patterns (`pmat query`, `NEVER use grep`, `--faults`)
- Index auto-built on first `pmat query`
