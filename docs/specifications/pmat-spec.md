# PMAT Mono-Spec v1.0

> Single-source specification for the PMAT (PAIML MCP Agent Toolkit) project.
> Each component links to a detailed sub-spec in `components/` (max 500 lines each).

## Table of Contents

| # | Component | Sub-Spec | Status |
|---|-----------|----------|--------|
| 1 | [Quality & Testing](#1-quality--testing) | [quality-testing.md](components/quality-testing.md) | Active |
| 2 | [Quality Gates](#2-quality-gates) | [quality-gates.md](components/quality-gates.md) | Active |
| 3 | [Build Performance](#3-build-performance) | [build-performance.md](components/build-performance.md) | Active |
| 4 | [Language Support](#4-language-support) | [language-support.md](components/language-support.md) | Active |
| 5 | [Semantic Search & Indexing](#5-semantic-search--indexing) | [semantic-search.md](components/semantic-search.md) | Active |
| 6 | [Context & Analysis](#6-context--analysis) | [context-analysis.md](components/context-analysis.md) | Active |
| 7 | [Graph & Metrics](#7-graph--metrics) | [graph-metrics.md](components/graph-metrics.md) | Active |
| 8 | [Database & Storage](#8-database--storage) | [database-storage.md](components/database-storage.md) | Active |
| 9 | [ML & Analytics](#9-ml--analytics) | [ml-analytics.md](components/ml-analytics.md) | Active |
| 10 | [Agent Integration](#10-agent-integration) | [agent-integration.md](components/agent-integration.md) | Active |
| 11 | [MCP & Protocols](#11-mcp--protocols) | [mcp-protocol.md](components/mcp-protocol.md) | Active |
| 12 | [CLI & HTTP API](#12-cli--http-api) | [cli-api.md](components/cli-api.md) | Active |
| 13 | [Code Quality & Analysis](#13-code-quality--analysis) | [code-quality.md](components/code-quality.md) | Active |
| 14 | [Work Management](#14-work-management) | [work-management.md](components/work-management.md) | Active |
| 15 | [Documentation](#15-documentation) | [documentation.md](components/documentation.md) | Active |
| 16 | [Repository Health](#16-repository-health) | [repo-health.md](components/repo-health.md) | Active |
| 17 | [WASM](#17-wasm) | [wasm.md](components/wasm.md) | Active |
| 18 | [Infrastructure](#18-infrastructure) | [infrastructure.md](components/infrastructure.md) | Active |
| 19 | [Memory Profiling](#19-memory-profiling) | [memory-profiling.md](components/memory-profiling.md) | Active |
| 20 | [SWE-CI & Evolution](#20-swe-ci--evolution) | [swe-ci-evolution.md](components/swe-ci-evolution.md) | Active |

---

## 1. Quality & Testing

**Sub-spec**: [components/quality-testing.md](components/quality-testing.md)

TDG (Technical Debt Gradient) scoring, test coverage (95% minimum via `cargo llvm-cov`),
mutation testing with AST fuzzing, and TDD methodology for CLI/MCP/HTTP interfaces.

**Key metrics**: TDG grade A-F, coverage %, mutation survival rate.

**Consolidated from**: tdg-specification, tdg-simplified-spec, tdg-enhanced-score, tdg-explain-mode,
tdg-enforcement-system, transactional-hashed-tdg-spec, COVERAGE, 80-20-to-95,
make-coverage-just-works, pmat-coverage-improve-command, mutant-fuzz-ast-testing, tdd-mcp-implementation.

---

## 2. Quality Gates

**Sub-spec**: [components/quality-gates.md](components/quality-gates.md)

O(1) quality gate enforcement via metric caching with <30ms pre-commit validation.
Phase 3.2 trueno-graph integration for symbol lookups. Phase 4 predictive ML gates.

**Key metrics**: lint <=30s, test-fast <=5min, coverage <=10min, binary <=50MB, deps <=3000.

**Consolidated from**: quick-test-build-O(1)-checking, O1-quality-gates-phase-3.2-trueno-graph,
o1-quality-gates-phase3.2-trueno-graph, o1-quality-gates-phase4-predictive, quality-gate-specification.

---

## 3. Build Performance

**Sub-spec**: [components/build-performance.md](components/build-performance.md)

Multi-phase build optimization: compiler flags, feature gates, dependency reduction.
Clean build target: <90s. Incremental: <30s. Minimal default features.

**Consolidated from**: build-performance-optimization-v1.0, build-performance-phase2,
phase1-build-perf-progress, dependency-reduction-benchmarking-framework,
reduce-dependencies-maintain-functionality-speedup-compile-testing-spec,
scientifically-remove-dependencies-time-improve-compile-speed-test-speed.

---

## 4. Language Support

**Sub-spec**: [components/language-support.md](components/language-support.md)

Multi-language analysis: Rust (primary), Python, TypeScript/JavaScript, Go, C/C++/CUDA,
JVM (Java/Kotlin), CLR (C#), functional (Haskell, Erlang/Elixir, R, Julia),
shell (bash/zsh), Ruchy, Lean 4. Each with AST parsing and complexity metrics.

**Consolidated from**: go-language-support, jvm-clr-language-support,
functional-scientific-language-support, shell-support-spec, enhanced-ruchy-support,
first-class-ruchy-spec, known-defects-languages-spec, lean-and-provable-contracts,
improved-cpp-pmat-query, cuda-simd-tdg, improve-language-mlops-support.

---

## 5. Semantic Search & Indexing

**Sub-spec**: [components/semantic-search.md](components/semantic-search.md)

`pmat query` semantic code search with TF-IDF, BM25 (FTS5), and embedding models.
SQLite + FTS5 backend for O(1) lookups. Git history correlation via RRF.
Enrichment flags: --churn, --duplicates, --entropy, --faults, -G.

**Consolidated from**: semantic-search-pmat-mcp-vector-db, semantic-search-feature,
index-v2-sqlite-fts5, git-commit-correlation-spec, git-history-rag-integration,
falsify-rag, pmat-query-raw-search-fallback.

---

## 6. Context & Analysis

**Sub-spec**: [components/context-analysis.md](components/context-analysis.md)

Deep context analysis with AST parsing, file discovery, and project structure analysis.
RAG-powered context generation. Two-phase execution: AST first, then parallel phases.
Arc<ProjectContext> reuse to avoid redundant syn parsing.

**Consolidated from**: current-deep-context-design-profiling, improve-context,
trueno-o1-context-tdg-integration, stack-visualization-diagnostics-reporting.

---

## 7. Graph & Metrics

**Sub-spec**: [components/graph-metrics.md](components/graph-metrics.md)

DAG construction (call graph, import graph, inheritance). PageRank scoring.
Graph descriptive statistics: centrality, community detection. Interactive visualization.

**Consolidated from**: graph-descriptive-statistics-v2, integrating-graph-visualizations-spec.

---

## 8. Database & Storage

**Sub-spec**: [components/database-storage.md](components/database-storage.md)

Trueno-DB columnar storage. SQLite + FTS5 for function index.
CSR graph database for O(1) lookups. LZ4 compressed caching.

**Consolidated from**: trueno-db-integration, trueno-db-integration-v2,
trueno-db-integration-review-response, trueno-integration-spec.

---

## 9. ML & Analytics

**Sub-spec**: [components/ml-analytics.md](components/ml-analytics.md)

Aprender ML library (sovereign stack, replaces linfa/nalgebra).
Model serialization via Realizar. TF-IDF embeddings for commit search.

**Consolidated from**: aprender-ml-integration, integrate-ml-trueno-latest-spec,
integrate-ml-trueno-a3-summary, model-serialization-request-spec-aprender,
model-serialization-manifest, model-serialization-realizar-integration, ml-model-serialization-spec.

---

## 10. Agent Integration

**Sub-spec**: [components/agent-integration.md](components/agent-integration.md)

Claude Agent SDK integration. AGENTS.md protocol bridging. Multi-agent workflows
with Actix actor framework. Claude Code skills integration.

**Consolidated from**: agents, claude-agent-integration, claude-code-agent-mode-spec,
claude-skills-spec-v1, claude-sub-agents-spec-actix.

---

## 11. MCP & Protocols

**Sub-spec**: [components/mcp-protocol.md](components/mcp-protocol.md)

MCP (Model Context Protocol) server implementation. Tool registration and validation.
Registry publishing. Acceptance testing with mock servers.

**Consolidated from**: mcp-specification, mcp-acceptance-testing, publish-mcp-registry.

---

## 12. CLI & HTTP API

**Sub-spec**: [components/cli-api.md](components/cli-api.md)

CLI command structure (clap). HTTP API with Actix-web.
Unified --help generation across CLI, MCP, and HTTP.
Acceptance testing for both interfaces.

**Consolidated from**: cli-specification, http-api-specification, cli-acceptance-testing,
http-api-acceptance-testing, unified-cli-mcp-help-integration.

---

## 13. Code Quality & Analysis

**Sub-spec**: [components/code-quality.md](components/code-quality.md)

Automated clippy fix with confidence scoring. Five Whys root cause analysis (Toyota Way).
Popper falsifiability scoring. Entropy/similarity detection.
Design-by-Contract with assertion generation. Mutation testing enhancement.

**Consolidated from**: auto-clippy-fix-guide, pmat-debug-five-whys,
popper-nullification-100point-score, entropy, entropy-spec,
enhance-pmat-mutation-spec, learn-from-rust-giants-spec, dbc, pmat-improve-safety.

---

## 14. Work Management

**Sub-spec**: [components/work-management.md](components/work-management.md)

`pmat work` contract-based quality enforcement with Popperian falsification.
Ticket tracking, roadmap management, and quality gate integration.

**Consolidated from**: enhance-pmat-work, enhance-pmat-work-spec,
improve-pmat-work, master-plan-pmat-work-system, roadmap-todo-quality-gate-spec.

---

## 15. Documentation

**Sub-spec**: [components/documentation.md](components/documentation.md)

Documentation accuracy enforcement with contradiction detection (Semantic Entropy).
CLI/MCP documentation enforcement. Link validation with 404 detection.

**Consolidated from**: CLI_MCP_DOCUMENTATION_ENFORCEMENT, documentation-accuracy-enforcement,
documentation-accuracy-enforcement-toyota-way-addendum, doc-validate.

---

## 16. Repository Health

**Sub-spec**: [components/repo-health.md](components/repo-health.md)

Rust project score (10 categories). Repository health scoring.
File health (max-lines enforcement). `pmat comply` quality checks (90+ checks).

**Consolidated from**: rust-project-score, rust-project-score-v1.1-update,
current-rust-project-score-implementation-v1, repo-score-spec, repo-score-adjust,
max-lines, PMAT_COMPLETE_UNIFIED_SPEC, demo-and-book-scoring, improve-pmat-comply,
cookbook-scoring-spec.

---

## 17. WASM

**Sub-spec**: [components/wasm.md](components/wasm.md)

WebAssembly analysis: bytecode parsing, resource metrics, control flow analysis.
Deep WASM inspection. Presentar pure-WASM visualization conversion.

**Consolidated from**: wasm-analysis-spec, wasm-quality-assurance,
deep-wasm, deep-wasm-phase2-plan, convert-demo-vis-to-presentar-pure-WASM.

---

## 18. Infrastructure

**Sub-spec**: [components/infrastructure.md](components/infrastructure.md)

Pre-commit hooks with quality gate enforcement. Enhanced hook runner ecosystem.
Makefile linter (bashrs). Project scaffolding. Session recording (.pmat format).
Oracle RAG knowledge system. Red team mode. Prompt system.

**Consolidated from**: pre-commit-hooks-spec, enhance-runner-ecosystem,
Makefile-lint-enhance, scaffold-maintain-spec, contract-refactoring-plan,
unified-contract-by-design, pmat-recording-format, pmat-oracle-specification,
red-team-mode-spec, prompt-spec, compute-brick-support,
kaizen-round-4-file-caching-plan, tracing-bug-discovery-tdg-git-expansion-spec,
universal-reporting-data-science-ascii-viz, qdd-tool-specification,
learning-system-ideas, refactoring-specification, unified-quality-driven-development-tool.

---

## 19. Memory Profiling

**Sub-spec**: [components/memory-profiling.md](components/memory-profiling.md)

Heap allocation profiling with dhat-rs. Peak memory tracking. Allocation hotspot
detection. Memory regression gates for CI. Required for production Rust projects.

**Comply check**: CB-140 penalizes repos without memory profiling infrastructure.

---

## 20. SWE-CI & Evolution

**Sub-spec**: [components/swe-ci-evolution.md](components/swe-ci-evolution.md)

Evolution-based code quality evaluation inspired by SWE-CI (arxiv:2603.03823).
EvoScore metric: future-weighted mean of normalized change across CI iterations.
Architect-programmer dual-agent protocol for requirement-driven development.

**Key formula**: `e = [sum(gamma^i * a(c_i))] / [sum(gamma^i)]` where gamma >= 1.

**Comply check**: CB-142 computes EvoScore from git history and CI results.

---

## Scoring Systems Evaluation

Eight scoring systems exist. Evaluated for: actionability (does it tell you what to fix?),
signal quality (does it correlate with real defects?), and cost (runtime + maintenance).

| Score | Granularity | Actionability | Signal | Cost | Verdict |
|-------|-------------|---------------|--------|------|---------|
| TDG | Per-file | **High** — pinpoints files | Complexity + churn + coverage + duplication | O(n) files, seconds | **Keep: core metric** |
| Rust Project Score | Per-project | **High** — category breakdown with recommendations | 10 categories, 274 pts | Minutes (runs tools) | **Keep: project dashboard** |
| Popper Score | Per-project | **Medium** — flags missing infra | File existence checks mostly | Seconds (file scans) | **Simplify** (see below) |
| Muda Waste | Per-project | **Medium** — 5 waste categories | Over/Wait/Inv/Proc/Def | Seconds | **Keep: lean signal** |
| EvoScore | Per-project | **Low** — says "regressing" not "where" | Test pass trajectory | Requires historical data | **Keep: trend only** |
| Comply Checks | Per-finding | **High** — exact file:line | 90+ pattern-specific checks | Seconds to minutes | **Keep: core enforcement** |
| Coverage % | Per-function | **High** — exact uncovered lines | Direct test gap signal | Minutes (llvm-cov) | **Keep: core metric** |
| Five Whys | Per-issue | **High** — root cause chain | Evidence-weighted hypotheses | Seconds | **Keep: debugging tool** |

### Assessment Notes

**TDG (A-tier)**: Most valuable metric. Per-file granularity means developers know exactly
what to fix. Composite formula (complexity + churn + coverage + duplication) captures
multiple debt dimensions. O(1) cached lookups. Grade gate (CB-200) is the most actionable
quality enforcement. No changes needed.

**Rust Project Score (A-tier)**: Valuable as project-level dashboard. 10 categories give
balanced view. Recommendations are directly actionable ("run cargo clippy --fix"). The 274
point scale is oddly specific — consider normalizing to percentage-only for communication.
Scores Rust Tooling at 130 points (47% of total) which overweights CI/CD relative to code
quality. Category weights should be reviewed.

**Comply Checks (A-tier)**: Most actionable enforcement. File:line precision. CB-120 series
(OIP Tarantula) catches real bugs (serde panics, NaN comparisons). CB-500+ language checks
provide concrete per-violation feedback. No changes needed.

**Coverage (A-tier)**: Direct signal — uncovered lines are provably untested. `pmat query
--coverage-gaps` ranks by impact score (missed_lines * pagerank / complexity). No changes.

**Muda Waste (B-tier)**: Useful lean signal. Five waste categories (overproduction, waiting,
inventory, over-processing, defects) map to real project problems. Current score of 36.3/100
is actionable. Keep as-is.

**Five Whys (B-tier)**: Valuable debugging tool but evidence sources overlap with TDG
(complexity 25% + TDG 25% = 50% redundant with TDG). Would benefit from incorporating
EvoScore trajectory as evidence. Keep but note overlap.

**Popper Score (C-tier, simplify)**: Mostly checks file existence (LICENSE, benches/,
Cargo.lock, CI config). 87.5/100 tells you infrastructure is present but says nothing about
code quality. Overlap with Rust Project Score (which checks the same infrastructure plus
code metrics). **Recommendation**: Fold Popper checks into Rust Project Score as a
"Reproducibility & Transparency" category rather than maintaining as separate command.

**EvoScore (C-tier, invest)**: Promising concept but currently non-functional (no data
pipeline). When activated, it answers a question no other metric answers: "is the project
improving?" However, per-project granularity limits actionability. **Recommendation**:
Implement `pmat test --record` and per-function EvoScore to increase granularity.

## Architectural Principles

1. **Sovereign AI (80/20 Batuta Stack)**: Prefer batuta stack (aprender, trueno, renacer, certeza) over external deps
2. **Toyota Way**: Jidoka (stop-the-line quality), Five Whys, Kaizen continuous improvement
3. **Popperian Falsification**: Quality claims must be falsifiable and evidence-based
4. **O(1) Operations**: Metric caching, hash-based validation, CSR graph lookups
5. **Mono-Spec Enforcement**: This document is the single source of truth (CB-140 comply check)

## Compliance Checks (pmat comply)

| Check | Name | Description |
|-------|------|-------------|
| CB-140 | Mono-Spec Structure | Validates pmat-spec.md exists, TOC links resolve, components <500 lines |
| CB-141 | Memory Profiling | Penalizes repos without dhat-rs or equivalent heap profiling |
| CB-142 | SWE-CI EvoScore | Computes evolution score from git history + CI pass rates |

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-09 | Initial mono-spec consolidation from 124 individual specs |
