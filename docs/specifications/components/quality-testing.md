# Quality & Testing

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 1

## TDG (Technical Debt Gradient)

### Scoring Model

TDG computes a composite score per function:

```
TDG = w_complexity * C + w_churn * H + w_coverage * (1 - V) + w_duplication * D
```

Where:
- **C** = cyclomatic complexity (normalized 0-1)
- **H** = churn score from git history (commits/timeframe)
- **V** = line coverage ratio
- **D** = duplication ratio from MinHash/LSH

### Grade Thresholds

| Grade | Score Range | Action Required |
|-------|-------------|-----------------|
| A | 0.0 - 0.2 | None |
| B | 0.2 - 0.4 | Monitor |
| C | 0.4 - 0.6 | Plan refactoring |
| D | 0.6 - 0.8 | Refactor next sprint |
| F | 0.8 - 1.0 | Immediate action |

### Provable-Contract Coverage Gate (CB-1400)

**A file cannot achieve grade A or A+ without provable-contract coverage.**

| Has Contract Coverage | Max Possible Grade |
|-----------------------|-------------------|
| Yes | A+ (95-100) |
| No | A- (85-89) — hard cap |

This enforces contract-first design as a hard requirement for the highest
quality tier. A "perfect" file (100/100 on all metrics) without a corresponding
entry in `binding.yaml` will be graded A-, not A+. The cap is applied in
`TdgScore::calculate_total()` after all metric components are computed.

**Contract coverage detected via (any of):**
1. **YAML binding** — `binding.yaml` with `status: implemented` and `module_path` matching the file
2. **Inline Rust contracts** — `#[requires(`, `#[ensures(`, `contract_pre_*` macros in source
3. **Regex contracts** — `regex:` or `regex_invariants:` in the function's contract YAML
4. **Refinement types** — `type_enforcement:` with `validated_types:` wrapping the function's types
5. **Lean theorems** — `lean_theorem:` referencing a proved theorem for the function
6. **Kani harnesses** — `kani_harness:` referencing a verified harness for the function

A file qualifies for A/A+ if ANY of its functions have contract coverage
through any of these mechanisms.

**Three contract expression languages:**

| Language | Example | L3 Verification | L4 Verification | L5 Verification |
|----------|---------|-----------------|-----------------|-----------------|
| Rust expressions | `x.iter().all(\|v\| v.is_finite())` | `debug_assert!` | Kani bounded check | Lean theorem |
| Regex patterns | `regex: '^(PMAT\|GH)-\d+$'` | `Regex::is_match()` | Kani bounded regex | Lean language containment |
| Refinement types | `refinement: "self.len() > 0"` | `Result` constructor | Kani/proptest | Lean type class proof |

**Rationale:** You cannot claim "excellent" quality without provable guarantees.
Metrics measure observable properties (complexity, duplication, coverage), but
only contracts prove behavioral correctness. This aligns with Meyer (1988)
Design by Contract, Vazou et al. (2014) Liquid Haskell refinement types,
and Li et al. (2025, arXiv:2510.12047) on formal contracts for LLM-generated code.

### TDG Enforcement (CB-200)

`pmat comply` check CB-200 gates the **per-definition** TDG grades already stored
in `.pmat/context.db` — one row per function/method in the `functions` table, NOT
one row per file. The two populations differ by an order of magnitude, so a
per-file grade distribution says nothing about whether this gate passes; see
[Real-World Assessment](#real-world-assessment) below.

**Minimum grade — precedence order.** The first source that supplies a value
wins (`check_tdg_grade_gate`,
`src/cli/handlers/comply_handlers/check_handlers/check_tdg_grade.rs`):

1. `.pmat-gates.toml` → `[tdg] min_grade` — project-local override, beats everything below
2. `.pmat.yaml` → `comply.thresholds.min_tdg_grade`
3. Built-in default: **`A`** (`default_min_tdg_grade()` in
   `src/models/comply_config_defaults.rs`, pinned by the `--lib` test
   `test_default_min_tdg_grade_is_a`)

`comply.checks.cb-200.threshold` is **not** the grade knob. CB-200's default
check config carries `threshold: None` and the gate never reads that field —
setting it changes nothing. `.pmat-gates.toml` also carries `[tdg] exclude`,
which is unioned with `.pmat.yaml`'s `comply.thresholds.tdg_exclude_paths`
rather than overriding it; test files are excluded unconditionally.

The floor is matched against the eleven grade spellings `GRADE_VARIANTS`
produces (`A+` … `F`) by taking the up-set of the floor, so a floor spelled any
other way **fails** the check instead of passing vacuously over an empty
violation set.

**The gate reads the index; it never builds one** (#939). With no
`.pmat/context.db` it returns Skip / "Not measured" and refuses to write
`.pmat/context.db` or `.pmat/context.idx` into the project under audit. A stale
index (a source file newer than the db) is still a real measurement: it is
reported with the staleness named in the message, not silently repaired. Run
`pmat query "x"` to (re)build the index first. Pinned by the `--lib` tests
`cb200_never_builds_an_index_inside_the_audited_project` and
`cb200_reports_staleness_instead_of_rebuilding`.

`pmat tdg --explain` shows score decomposition.

### Real-World Assessment

**TDG is the most actionable metric in PMAT.** Per-file granularity means developers know
exactly what to fix. Include-fragment files (tree-sitter can't parse non-standalone Rust)
are auto-detected and excluded from TDG scoring. False-positive unwrap detection inside
string literals (e.g. documentation text) was fixed in v3.11.1.

**Two populations are reported, and they must not be conflated.** A per-file
distribution cannot be read as a CB-200 verdict:

| | Population | Producer | Snapshot |
|---|---|---|---|
| Per **file** | 2,305 analyzed files (2,198 include-fragments skipped) | `pmat tdg` | avg 95.6/100 — A-: 2241, B+: 28, B: 21, C+: 3, C: 1, **F: 0** |
| Per **definition** — what CB-200 gates | 23,451 definitions over 2,626 file paths | `pmat query`'s index (`.pmat/context.db`) | A+: 18801, A: 2611, A-: 1202, B+: 403, B: 209, B-: 96, C+: 55, C: 25, C-: 19, D: 12, **F: 18** |

Both rows are snapshots of this repo, not invariants — re-measure rather than
quote them. The per-definition row was taken on 2026-08-22 with:

```bash
sqlite3 .pmat/context.db \
  'SELECT tdg_grade, COUNT(*) FROM functions GROUP BY tdg_grade ORDER BY COUNT(*) DESC'
```

The per-file row is an earlier recorded `pmat tdg` run. Note the disagreement in
the last column: zero F-grade *files* alongside eighteen F-grade *definitions*.
At the built-in floor of `A` only `A+` and `A` pass, so 2,039 of those 23,451
definitions sit below the floor before the test-file and `[tdg] exclude`
filters run — which is the number CB-200 acts on.

**Versus Popper Score (87.5)**: TDG tells you "query_handler.rs has complexity 15 and 3% duplication."
Popper tells you "LICENSE exists." TDG is 10x more actionable per point.

**Versus Rust Project Score (75.5%, Grade B)**: TDG is per-file, Rust Project Score is per-project.
Both useful at different zoom levels — TDG for daily development, Rust Project Score for
quarterly project health reviews.

### Churn-Weighted TDG Priority (Implemented)

New computed field for ranking which files to fix first:

```
tdg_priority = tdg_score * (1 + churn_factor)
```

Where `churn_factor` = normalized commit count in 90-day window (0.0-1.0).

**Rationale**: A file with TDG grade C that changes 3x/week is more urgent than a
file with TDG grade C that hasn't changed in 6 months. The static file's debt is
dormant; the active file's debt compounds with every change.

**Surfacing**: `pmat query --churn --rank-by priority` sorts by churn-weighted TDG.
Aliases: "priority", "tdg-priority", "churn-priority". Implementation in
`src/services/agent_context/query/engine_query.rs`. Also add to `pmat tdg --hotspots`
output so the top-10 list reflects actual urgency, not just static severity.

**Data source**: Git log already parsed for `--churn` enrichment. No new I/O needed.
Computation is O(1) per function (multiply two cached values).

### Transactional Updates

TDG scores use BLAKE3 hashing for incremental updates:
- Only recompute changed files (git diff)
- Store in `.pmat/context.db` SQLite
- Cache invalidation on HEAD hash change

## Test Coverage

### Targets

- **Minimum**: 95% line coverage (upgraded from 85%)
- **Tool**: `cargo llvm-cov` exclusively (never cargo-tarpaulin)
- **Command**: `make coverage` (canonical invocation)

### Coverage Workflow

```bash
# Find coverage gaps (MANDATORY approach)
pmat query --coverage-gaps --limit 30 --exclude-tests

# Target highest-impact functions first
pmat query --coverage-gaps --rank-by impact --limit 20

# Verify improvement
pmat query "function_name" --coverage --include-source --limit 1
```

### Coverage Checks

- `pmat comply` validates coverage >= 95%
- `.pmat/coverage-cache.json` stores per-function line data
- Cache invalidated on HEAD hash change
- MCP files (`mcp_pmcp/`, `mcp_server/`) have `coverage(off)` — excluded

### Pareto Analysis (80/20)

Coverage improvement prioritizes:
1. Functions with most uncovered lines (highest ROI)
2. Functions with highest PageRank (most important)
3. Functions with highest complexity (most bug-prone)

Impact score: `missed_lines * pagerank / complexity`

## Mutation Testing

### AST Fuzzing

Mutation operators:
- **Arithmetic**: `+` -> `-`, `*` -> `/`
- **Boolean**: `&&` -> `||`, `!` -> identity
- **Boundary**: `<` -> `<=`, `>=` -> `>`
- **Return**: `Ok(x)` -> `Err(...)`, `Some(x)` -> `None`
- **Deletion**: Remove statement, replace with `todo!()`

### Survival Prediction

ML-based prediction of mutation survival:
- Features: complexity, coverage, churn, TDG grade
- Model: Aprender gradient boosting
- Target: >80% mutation kill rate

## TDD Implementation

### Three-Interface Pattern

Every feature must have tests across:
1. **CLI**: `cargo test --lib -- cli_handler`
2. **MCP**: `cargo test --lib -- mcp_tool`
3. **HTTP**: `cargo test --lib -- http_endpoint`

### Test Organization

- Unit tests: same file as implementation
- Integration tests: `src/tests/` directory
- Property tests: proptest (never quickcheck)
- Coverage-off: `#[cfg_attr(coverage_nightly, coverage(off))]` for test modules

## Key Files

| File | Purpose |
|------|---------|
| `src/tdg/` | TDG scoring implementation |
| `src/tdg/tdg_graph.rs` | TdgGraph with O(1) dependency tracking |
| `src/cli/handlers/comply_handlers/check_handlers/check_tdg_grade.rs` | CB-200 TDG gate |
| `Makefile` (coverage target) | Canonical coverage invocation |

## References

- Consolidated from: tdg-specification, tdg-simplified-spec, tdg-enhanced-score,
  tdg-explain-mode, tdg-enforcement-system, transactional-hashed-tdg-spec,
  COVERAGE, 80-20-to-95, make-coverage-just-works, pmat-coverage-improve-command,
  mutant-fuzz-ast-testing, tdd-mcp-implementation
