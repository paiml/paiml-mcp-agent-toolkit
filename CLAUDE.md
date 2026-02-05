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
| `nalgebra-sparse` | Review | `aprender::primitives` sparse matrices |
| `roaring` | Keep | Specialized bitmap (no batuta equivalent yet) |
| `rand` | Keep | Foundational (may add to trueno later) |
| `rayon` | Keep | Foundational parallel iterator |

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

### Enrichment Flags

- **`-G` / `--git-history`** — Fuse git commit history via RRF. Finds code by intent via commit message semantic search.
- **`--churn`** — Git volatility metrics (90-day window). Hot files (>50% churn) flagged.
- **`--duplicates`** — Code clone detection via MinHash + LSH.
- **`--entropy`** — Pattern diversity metrics. Low (<30%) = boilerplate; high (>80%) = unique.
- **`--faults`** — Batuta fault pattern annotations (unwrap, panic, unsafe, etc.).

### When grep IS acceptable
- Searching for a specific string literal you know exactly (e.g., an error message)
- Searching non-code files (TOML, YAML, Markdown)
- Quick one-off during debugging when you need exact line matches

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

- Runs critical chapters in parallel (Ch 5, 7, 13, 14), completes in <30 seconds
- Chapter 13 (Multi-Language) is CRITICAL - must always pass
- If tests fail, fix code OR update book tests. Apply Andon Cord: STOP if quality issues found

---

## CRITICAL: pmat-book Push Enforcement Policy

**MANDATORY: Book updates MUST be pushed with code changes**

- **Pre-Commit Hook**: Warns about unpushed pmat-book commits (non-blocking)
- **Pre-Push Hook**: **BLOCKS `git push`** until all pmat-book commits are pushed first

**Workflow**: Update pmat-book → push to main (deploys GitHub Pages) → push code.

**crates.io Release**: Ensure all pmat-book changes pushed, `make validate-book` passes, and GitHub Pages deployment completed before `cargo publish`.

---

## CRITICAL: O(1) Quality Gates (Phase 2 - Active)

**AUTOMATIC ENFORCEMENT: Pre-commit hooks validate metrics in <30ms**

Spec: `docs/specifications/quick-test-build-O(1)-checking.md`

Metric recording during development (`make lint/test-fast/coverage/release`) populates `.pmat-metrics/`. Pre-commit validates against thresholds in `.pmat-metrics.toml`:

- **lint**: ≤30s | **test-fast**: ≤5min | **coverage**: ≤10min | **binary size**: ≤50MB | **dependencies**: ≤3,000

Staleness: Metrics older than 7 days trigger warnings. Emergency bypass: `git commit --no-verify`.

---

## CRITICAL: Documentation Accuracy Enforcement (Zero Hallucinations)

**MANDATORY FOR README.md, CLAUDE.md, GEMINI.md, AGENT.md:**

```bash
# Step 1: Generate deep context
pmat context --output deep_context.md --format llm-optimized

# Step 2: Validate documentation accuracy
pmat validate-readme \
    --targets README.md CLAUDE.md GEMINI.md AGENT.md \
    --deep-context deep_context.md \
    --fail-on-contradiction --verbose
```

Validates: capability claims against AST, file path references, function/module references, external URLs (404 detection). Uses Semantic Entropy (Nature 2024) and MIND framework (IJCAI 2025).

Enforced by pre-commit hook, CI/CD pipeline, and `pmat quality-gate --checks docs-accuracy`. Spec: `docs/specifications/documentation-accuracy-enforcement.md`

---

## Bash/Makefile Quality Enforcement with bashrs

**MANDATORY: All bash scripts and Makefiles must pass bashrs linting.**

bashrs (PAIML) lints for SC2086/SC2046/SC2116, DET003 (non-determinism), IDEM002 (idempotency), SEC008 (security).

```bash
bashrs lint scripts/install.sh
bashrs lint Makefile
```

Installation: `pmat hooks install --tdg-enforcement`. Bug reports: https://github.com/paiml/bashrs/issues

---

## Coverage Tool Policy

**Use `cargo llvm-cov` exclusively. NEVER use cargo-tarpaulin.**

---

## Test Coverage

**Total: ~94 tests ignored** (82 in server/src, remainder in server/tests). Tests marked `#[ignore]` for stable coverage metrics.

| Category | Count | Status |
|----------|-------|--------|
| Language-Specific (Kotlin, WASM) | 4 | Ignored |
| Language Regression (C, WASM, Bash, C++, PHP, Swift) | 6 | All PASSING (Sprint 42), ignored for concurrency |
| Infrastructure (memory, TDG, profiler, dashboard) | 8 | Ignored |
| Binary/E2E Integration | 8 | Require pmat binary |
| Annotation TDD | 7 | Require pmat binary |
| Unified Quality Framework | 14 | Property tests, ignored |
| Language Detection | 5 | Need fixes |
| Enhanced Naming | 6 | Require implementation |
| Unified Context | 4 | Require implementation |
| TypeScript/JavaScript | 3 | Need implementation |
| Real-World/Performance | 5 | Need proper setup |
| Timeout Integration | 3 | Require binary |
| Ruchy Parser | 10 | RED tests for ruchy-ast feature |
| CLI/Quality | 3 | Various |

**Previously "Known Failing" 14 tests**: ALL NOW PASSING (verified October 19, 2025). Service layer (6), defect report (5), E2E binary (3 - correctly ignored, require binary).

Re-enable by removing `#[ignore]` when fixed. Always work on master - no branching.

---

## PMAT Five Whys Root Cause Analysis (Toyota Way)

**Command**: `pmat five-whys` (aliases: `why`, `debug-whys`) | **Status**: Production-ready

Evidence-based root cause analysis using Toyota Way Five Whys. **This is the ONLY acceptable debugging method.**

```bash
pmat five-whys "Stack overflow in parser"              # Basic (5 iterations)
pmat why "Memory leak in cache" --depth 3              # Short alias
pmat five-whys "Test failures" --format json -o out.json  # JSON output
pmat five-whys "Perf regression" --format markdown --auto-analyze
```

Options: `--depth <1-10>`, `--format <text|json|markdown>`, `--output <FILE>`, `--path <PATH>`, `--context <FILE>`, `--auto-analyze`

Evidence sources: complexity (25%), TDG (25%), SATD (20%), git churn (20%), dead code (10%). Spec: `docs/specifications/pmat-debug-five-whys.md`

---

## Rust Project Score v1.1

**Command**: `pmat rust-project-score` (alias: `rust-score`) | **Status**: Production-ready

106-point scoring across 6 categories based on 15 peer-reviewed papers (2022-2025).

```bash
pmat rust-project-score                    # Fast mode (~2-3 min, skips clippy/mutation/build)
pmat rust-project-score --full             # Full mode (~10-15 min, all checks)
pmat rust-project-score --format json -o score.json  # CI/CD
pmat rust-project-score --failures-only    # Show only failures
```

**Categories**: Rust Tooling (25pts), Code Quality (26pts), Testing (20pts), Documentation (15pts), Performance (10pts), Dependency Health (12pts).

Spec: `docs/specifications/rust-project-score-v1.1-update.md` | Location: `server/src/services/rust_project_score/`

---

## CRITICAL: Renacer Golden Tracing

**MANDATORY for**: Transpilers, distributed systems, multi-process workflows, cross-language integrations.

```bash
renacer capture --scenario transpile_rust_to_js  # Capture golden trace
renacer validate --all                            # Validate before commits
```

Config: `renacer.toml` in project root. Always validate golden traces before completing work.

---

## trueno-graph O(1) Context and TDG Integration

**STATUS**: ACTIVE (NOT feature-gated, used in production)
**Spec**: `docs/specifications/trueno-o1-context-tdg-integration.md`

trueno-graph provides CSR graph database for O(1) symbol lookups and PageRank-based importance scoring.

**Integrations**:
1. **Context Generation** (`context.rs:565-572`, `context_graph.rs`): Every `analyze_project_with_cache()` builds a ProjectContextGraph. 8/8 tests passing.
2. **TDG Analysis** (`tdg/tdg_graph.rs`): TdgGraph provides O(1) function dependency tracking with PageRank criticality. 7/7 tests passing.

**Architecture**: Dual storage pattern - HashMap (O(1) lookups) + CSR graph (PageRank) + bidirectional NodeId mapping.

**Key insight**: CSR graphs only track nodes with edges; `num_nodes()` returns node_map.len() not graph.num_nodes().

---

## DETERMINISTIC Agent Instructions

Follow instructions in **`docs/agent-instructions/`** for deterministic fixes:

1. **`pmat-work-ux-fixes.md`** - Fuzzy ID matching, status display, quality gates, short IDs
2. **`pmat-work-quality-principles.md`** - Five Whys, Renacer tracing, Rust project requirements, commit metadata

Workflow: Read instruction doc → apply fixes in priority order → test each fix → commit atomically.

---

## Stack Documentation Search

```bash
batuta oracle --rag-index                    # Index all stack docs (once)
batuta oracle --rag "your question here"     # Search across stack
batuta oracle --rag-stats                    # Check index status
```

Auto-updates via post-commit hooks. Manual freshness check: `ora-fresh`. Force reindex: `batuta oracle --rag-index --force`

---

## Compliance (CB-130)

CB-130 validates agent context adoption via `pmat comply check`:
- Index exists at `.pmat/context.idx` and is fresh (< 24 hours)
- CLAUDE.md contains required patterns (`pmat query`, `NEVER use grep`, `--faults`)
- Index auto-built on first `pmat query`
