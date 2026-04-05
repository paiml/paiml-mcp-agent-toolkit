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
| 19 | [PV Compatibility](#19-pv-compatibility) | [pv-compatibility.md](components/pv-compatibility.md) | Active |
| 19b | [Memory Profiling](#19-memory-profiling) | [memory-profiling.md](components/memory-profiling.md) | Active |
| 20 | [SWE-CI & Evolution](#20-swe-ci--evolution) | [swe-ci-evolution.md](components/swe-ci-evolution.md) | Active |
| 21 | [Scoring Convergence & Hardening](#21-scoring-convergence--hardening) | [scoring-convergence.md](components/scoring-convergence.md) | Active |
| 22 | [Provable Contracts Integration](#22-provable-contracts-integration) | [provable-contracts.md](components/provable-contracts.md) | Active |
| 23 | [Contract Surface Types](#23-contract-surface-types) | [contract-surface-types.md](components/contract-surface-types.md) | Active |
| 24 | [Verification Backends](#24-verification-backends) | [verification-backends.md](components/verification-backends.md) | Active |
| 25 | [Commit-Level Contract Enforcement & Asset Contracts](#25-commit-level-contract-enforcement--asset-contracts) | [commit-level-contract-enforcement.md](components/commit-level-contract-enforcement.md) | Active |

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

**MANDATORY: Provable-contract-first design for ALL agents and sub-agents.**
No agent may generate code without a prior contract (YAML or contract.json).
CB-1400..1410 enforce contract existence, falsifiability, verification level,
and assume-guarantee chains for multi-agent workflows. Minimum L1 for
autonomous agents (recommended L3+). Based on Bruni et al. (2026,
arXiv:2602.22302) Agent Behavioral Contracts.

Claude Agent SDK integration. AGENTS.md protocol bridging. Multi-agent workflows
with Actix actor framework. Claude Code skills integration.

**Agent instructions**: [provable-contract-first-agents.md](../agent-instructions/provable-contract-first-agents.md)

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

## 21. Scoring Convergence & Hardening

**Sub-spec**: [components/scoring-convergence.md](components/scoring-convergence.md)

**ONE canonical command**: `pmat score` produces geometric composite (0-100) from
7 sub-scores (RPS, comply, coverage, muda, evoscore, DBC, file health). Planned
8th sub-score: PV Lint (`pv lint --format json`). `pmat comply` defaults to
`check` (no subcommand needed). Deep `pmat query --score-diagnosis` integration.
CI via `quality-gate.yml`. Pre-push hook with `pmat score --gate 60`.

**New comply checks**: CB-145 (regression), CB-146 (cross-validation), CB-147
(composite gate), CB-148 (spec-work), CB-150 (stack quality), CB-533 (stale paths),
CB-1201 (PV Lint gate), CB-1202 (contract coverage), CB-1250 (work-DBC binding).

---

## 22. Provable Contracts Integration

**Sub-spec**: [components/provable-contracts.md](components/provable-contracts.md)

Three contract expression languages: **Rust expressions** (default),
**regex patterns** (for string-producing functions), and **refinement types**
(Haskell/F# style — bad states unrepresentable). Two verification backends:
**Lean** (L5 mandatory — full theorem proving) and **Kani** (L4 — bounded
model checking). YAML contracts + `pv lint` enforce. 285+ contracts across
sovereign stack. CB-1203: annotation coverage. CB-1500..1513: Lean/Kani
verification pipeline checks.

Regex contracts use `regex:` fields in postconditions; Kani can bounded-verify
regex matching; Lean can prove language containment. Refinement type contracts
use `type_enforcement:` with `validated_types:` (newtype pattern) and
`type_class_contracts:` (algebraic laws like Invertible, Idempotent, Commutative).

---

## 23. Contract Surface Types

**Sub-spec**: [components/contract-surface-types.md](components/contract-surface-types.md)

Eight contract surface types (CLI, HTTP, MCP, Config, Library, PV Schema,
TUI Widget, WASM FFI) plus two contract classes (**regex-contract**,
**refinement-type**) with anti-leak enforcement via CB-1305 contract class
classifier (7 classes). Resolution hierarchy: org commits > batuta oracle >
arXiv > web search > chain of thought > five-whys. New checks CB-1300..1308
prevent whack-a-mole drift from provable-contracts.

**Key insight**: CB-1305 is the anti-leak gate -- classifies every contract YAML
and flags unrecognized structures instead of silently skipping them.

**Consolidated from**: provable-contracts leak analysis, contract-surface-types (new).

---

## 24. Verification Backends

**Sub-spec**: [components/verification-backends.md](components/verification-backends.md)

**Lean (L5):** Full theorem proving — mandatory for all equation contracts.
`lean_theorem:` with `status: proved`, zero `sorry` budget. Dual expression
language (`rust:` + `lean:`) verified independently. Can prove regex language
containment and refinement type soundness universally. Checks: CB-1500..1503.

**Kani (L4):** Bounded model checking via SAT/SMT. Exhaustive verification up
to configurable bounds. Verifies regex postconditions, refinement type
constructors, and type class laws (idempotency, commutativity, invertibility)
for all inputs within bound. Checks: CB-1510..1513.

---

## 25. Commit-Level Contract Enforcement & Asset Contracts

**Sub-spec**: [components/commit-level-contract-enforcement.md](components/commit-level-contract-enforcement.md)

Unifies `pmat work` contracts and `provable-contracts` YAML under a single
commit-level enforcement pipeline. Six phases:

**Phase 1: Work Item -> YAML Contract.** `pmat work start` generates both
`contract.json` (DbC v5.0) and `contracts/work/<ID>.yaml` (provable-contracts
schema). Acceptance criteria map to typed proof obligations. `pv validate` and
`pv score` become commit gates.

**Phase 2: L-Level Monotonicity Ratchet.** Verification levels (L0-L5) in
`binding.yaml` never decrease. Pre-commit reads cached levels (O(1), < 3ms)
and blocks regressions. Escape hatch: `pmat comply ratchet-override` with
audit trail, 14-day expiry. CB-1330.

**Phase 3: Asset Layout Contracts.** Non-code assets are containers with named
slots (rmedia Grid Protocol paradigm). Content never exists without a placement
contract. Seven asset types: README (CB-1320), Dockerfile (CB-1321), SVG
(CB-1322), forjar (CB-1323), mdBook (CB-1324), CHANGELOG (CB-1325), Badges
(CB-1326). All validated from O(1) cache.

**Phase 4: Differential Obligation Verification.** Only obligations whose bound
functions appear in the diff are re-checked. File-to-binding reverse index
(`.pmat/binding-index.json`) enables O(1) lookup.

**Phase 5: Assume-Guarantee Chains.** Concurrent work items declare `assumes`
and `guarantees`. Pre-commit validates chain consistency — no commit may break
another work item's assumptions.

**Phase 6: `pmat query --contracts` Enrichment.** Six new flags: `--contracts`,
`--contract-gaps`, `--min-level`, `--max-level`, `--contract-score`,
`--asset-contracts`. Contract data loaded lazily into `ContractIndex` from
cached binding index. Composes with `--churn`, `--faults`, `--duplicates`, `-G`.

**Phase 7: Hook Subsystem Consolidation.** Git history audit across 8 PAIML
repos identified 14 problem classes: 6 conflicting writers (Critical), shell
injection (Medium-High), non-deterministic timestamps, TOCTOU races, 72
`--no-verify` bypasses. Design rules: single `HookRegistry` writer (CB-1333),
atomic writes (CB-1334), deterministic content (CB-1335), shell escaping
(CB-1336), O(1) performance budget (CB-1337).

**Phase 8: Falsify Leak Remediation.** Provable-contracts git history audit
identified 7 leak classes: ghost bindings (28,206 stripped in PMAT-106),
placeholder preconditions (507 `!is_empty()` in PMAT-129), zero enforcement
(fleet avg 0.01 in PMAT-133), codegen fidelity drift, spec number inflation
(22 falsified claims), parser bugs, assertion misplacement. Design rules with
CB-1338..1343 break the whack-a-mole cycle.

**O(1) Firm Requirement:** All pre-commit checks < 45ms total from cached data.
No cold verification in the commit path. Caches populated by `pmat work
checkpoint`, `pmat comply refresh-contracts`, `pmat asset validate`.

---

## Scoring Systems Evaluation

Eight scoring systems evaluated for actionability, signal quality, and cost:

| Score | Verdict | Key Trait |
|-------|---------|-----------|
| TDG | **Keep: core** | Per-file, pinpoints hotspots |
| Rust Project Score | **Keep: dashboard** | 11 categories, 289 pts |
| Popper Score | **Absorbed into RPS** | Falsifiability gateway + Reproducibility scorer |
| Muda Waste | **Keep: lean signal** | 5 waste categories mapped to files |
| EvoScore | **Keep: trend** | Test pass trajectory over git history |
| Comply Checks | **Keep: enforcement** | 90+ pattern-specific checks |
| Coverage % | **Keep: core** | Per-function uncovered lines |
| Five Whys | **Keep: debugging** | Evidence-weighted root cause chains |

All six planned improvements completed:
1. **RPS v3.0** — 11 categories, falsifiability gateway, 289-point scale
2. **Popper absorbed** — Categories B-F → ReproducibilityScorer, A → gateway
3. **Five Whys v2** — Diversified weights (Complexity 25%, SATD 20%, Churn 15%, Evo 15%, Coverage 15%, Dead 10%)
4. **EvoScore pipeline** — `pmat test --record` writes commit test data
5. **TDG churn priority** — `--rank-by priority` = `tdg_score * (1 + churn_score)`
6. **Muda file mapping** — Waste categories → concrete file paths

## Architectural Principles

1. **Sovereign AI (80/20 Batuta Stack)**: Prefer batuta stack (aprender, trueno, renacer, certeza) over external deps
2. **Toyota Way**: Jidoka (stop-the-line quality), Five Whys, Kaizen continuous improvement
3. **Popperian Falsification**: Quality claims must be falsifiable and evidence-based
4. **O(1) Operations**: Metric caching, hash-based validation, CSR graph lookups
5. **Mono-Spec Enforcement**: This document is the single source of truth (CB-140 comply check)
6. **Contract-First Agent Design**: No agent or sub-agent may write code before writing a provable contract. Enforced by CB-1400..1410. Verification ladder (L0-L5) applies to agent tasks, not just kernels. Assume-guarantee chains (Dardik & Kang, 2509.06250) compose multi-agent contracts. See [agent-integration.md](components/agent-integration.md)
7. **Asset Layout Contracts**: Non-code assets (README, Dockerfile, SVG, configs, books) are containers with named slots — content never exists without a placement contract (rmedia Grid Protocol paradigm). Enforced by CB-1320..1326. See [commit-level-contract-enforcement.md](components/commit-level-contract-enforcement.md)

## Compliance Checks (pmat comply)

| Check | Name | Description |
|-------|------|-------------|
| CB-140 | Mono-Spec Structure | pmat-spec.md exists, TOC links resolve, components <500 lines |
| CB-141 | Memory Profiling | Penalizes repos without dhat-rs or equivalent |
| CB-142 | SWE-CI EvoScore | Evolution score from git history + CI pass rates |
| CB-1400..1410 | Agent Contracts | Contract existence, falsifiability, verification level, A/G chains. See [agent-integration.md](components/agent-integration.md) |
| CB-1320..1326 | Asset Layout | README, Dockerfile, SVG, forjar, mdBook, CHANGELOG, badges. See [commit-level-contract-enforcement.md](components/commit-level-contract-enforcement.md) |
| CB-1330..1332 | Contract Gates | L-level ratchet, work YAML validity, cache staleness |
| CB-1333..1337 | Hook Quality | Single writer, atomic writes, determinism, no injection, performance |
| CB-1338..1343 | Falsify Leaks | Ghost bindings, placeholders, enforcement penetration, codegen fidelity, spec accuracy, assertion placement |

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.2 | 2026-04-05 | CB-140 compliance: condense pmat-spec.md (543→488), component 25 (1921→395). Fix repo-health.md stale denominators (274→289). Consolidate comply check table |
| 1.1 | 2026-04-05 | Component 25: Commit-level contract enforcement (8 phases), asset layout contracts (CB-1320..1326), hook consolidation (CB-1333..1337), falsify leak remediation (CB-1338..1343), pmat query --contracts enrichment |
| 1.0 | 2026-03-09 | Initial mono-spec consolidation from 124 individual specs |
