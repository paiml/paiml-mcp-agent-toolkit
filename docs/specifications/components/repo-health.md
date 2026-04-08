# Repository Health

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 16

## Rust Project Score v3.0

### 289-Point Scoring

```bash
pmat rust-project-score              # Fast mode (~2-3 min)
pmat rust-project-score --full       # Full mode (~10-15 min)
pmat rust-project-score --format json -o score.json
```

### Categories (11)

| Category | Points | Checks |
|----------|--------|--------|
| Rust Tooling & CI/CD | 130 | clippy, fmt, edition, MSRV, CI, audit |
| Code Quality | 26 | complexity, TDG grades, dead code |
| Testing Excellence | 20 | coverage, mutation, property tests |
| Known Defects | 20 | P0 defects, unwrap audit |
| Documentation | 15 | doc-tests, README, API docs |
| Reproducibility | 15 | Popper falsifiability, statistical rigor |
| Formal Verification | 16 | Miri, Kani, Verus, specs |
| Build Performance | 15 | build time, incremental, caching |
| Dependency Health | 12 | audit, outdated, count |
| Performance & Benchmarking | 10 | benchmarks, binary size |
| GPU/SIMD Quality | 10 | CUDA-TDG, barrier safety |

### Real-World Assessment

**Current score: 193.5/289 (75.5%, Grade B).** Known Defects (100%), GPU/SIMD (100%),
Performance (100%) are perfect. Code Quality (46%) and Formal Verification (43%)
are weakest. Category breakdown reveals imbalance:

| Category | % of Total | Assessment |
|----------|-----------|------------|
| Rust Tooling & CI/CD | **45%** (130/289) | Overweighted. Dominates score. A project with perfect CI but bad code gets a B. |
| Code Quality | 9% (26/289) | Underweighted relative to importance. |
| Testing Excellence | 7% (20/289) | Reasonable for its scope. |
| Formal Verification | 6% (16/289) | Aspirational — most projects won't have Miri/Kani/Verus. |
| GPU/SIMD Quality | 3% (10/289) | Niche — only relevant to GPU projects. Returns 100% for non-GPU projects (free points). |

**Weight imbalance**: Rust Tooling at 130 points means clippy + fmt + edition + MSRV + CI + audit
together outweigh Code Quality + Testing + Documentation + Performance combined (71 pts).
A project passing `cargo fmt` and `cargo clippy` with CI configured scores higher than one
with 95% test coverage, low complexity, and zero dead code but no CI.

### v3.0 Rebalanced Design

Normalize to 100-point scale. Each category 10 points. Add Reproducibility (from Popper).
Drop GPU/SIMD for non-GPU projects (conditional category).

| Category | v2.3 pts | v3.0 pts | Change |
|----------|----------|----------|--------|
| Code Quality | 26 (9%) | 10 (10%) | Normalized weight |
| Testing Excellence | 20 (7%) | 10 (10%) | +3% weight |
| Rust Tooling & CI/CD | 130 (47%) | 10 (10%) | **-37% weight** (was dominating) |
| Documentation | 15 (5%) | 10 (10%) | +5% weight |
| Build Performance | 15 (5%) | 10 (10%) | +5% weight |
| Performance & Benchmarking | 10 (4%) | 10 (10%) | +6% weight |
| Dependency Health | 12 (4%) | 10 (10%) | +6% weight |
| Known Defects | 20 (7%) | 10 (10%) | +3% weight |
| Formal Verification | 16 (6%) | 10 (10%) | +4% weight |
| Reproducibility (NEW) | — | 10 (10%) | Absorbs Popper B-F |
| GPU/SIMD Quality | 10 (4%) | conditional | Only scored if GPU files present |

**Key principle**: Equal weights prevent any single category from dominating.
The normalized percentage (avg of category %) already does this in v2.3 output,
so v3.0 aligns the raw points with the display.

**Popper absorption**: Popper Categories B-F (Reproducibility, Transparency,
Statistical Rigor, Historical Integrity) become subchecks of the new
Reproducibility category. Popper Category A (Falsifiability Gateway) stays
as a standalone precondition — project must score >= 60% on falsifiability
before the full score is computed.

### Peer-Reviewed Basis

Scoring derived from 15 peer-reviewed papers (2022-2025) on Rust project quality.

### Workspace-Aware Scoring (Planned)

**Contract**: `contracts/workspace-scoring-v1.yaml`

Monorepo workspaces (e.g., aprender with 60+ crates) need per-subcrate
scoring, not just root-level. Design:

```bash
pmat rust-project-score --path ../aprender
# Output:
#   aprender (workspace): A- (87.5%)
#   ├── aprender-core: A (92%)
#   ├── aprender-compute: B+ (85%)
#   ├── aprender-graph: A- (88%)
#   └── ... (60+ crates)
```

**Discovery**: Parse `[workspace] members` from root `Cargo.toml`,
expand globs, filter to directories with `src/` or `lib.rs`.

**Per-crate scoring**: Run the 11 RPS category scorers on each
subcrate's path independently. Reuse `FileCache` across subcrates
for shared dependencies.

**Aggregate**: Workspace score = geometric mean of subcrate scores.
One failing crate drags the workspace score (same principle as
composite score geometric mean).

**O(1) caching**: Per-crate scores cached in `.pmat-metrics/workspace-scores.json`.
Invalidated by Cargo.toml mtime or `[workspace] members` change.

## Multi-Repo Quality Scoring (Sovereign Stack)

### Problem

The PAIML sovereign stack spans 5+ repos (aprender 78K functions,
trueno 9K, realizar 8K, pmat 20K, ruchy). Quality must be measured
across the fleet, not per-repo in isolation. Individual repo scores
miss cross-repo dependency health and contract coverage gaps.

### Fleet Score: `pmat score --fleet`

```bash
pmat score --fleet ../aprender ../trueno ../realizar .
# Output:
#   PAIML Fleet Score: B+ (83.2%)
#   ├── aprender:  D  (56.4%) — 78549 fn, 523 contracts
#   ├── pmat:      A  (90.6%) — 20161 fn, 8 contracts
#   ├── realizar:  C  (65.0%) — 7838 fn, 17 contracts
#   ├── trueno:    C+ (68.0%) — 9084 fn, 0 contracts
#   └── ruchy:     B  (75.0%) — est.
#   Fleet contract coverage: 548/115K functions (0.5%)
```

### Multi-Repo Contract Schema

The aprender monorepo demonstrates the contract-at-scale pattern:

| Metric | aprender | pmat | trueno | realizar |
|--------|----------|------|--------|----------|
| .rs files | 9,313 | 2,308 | 1,566 | 2,094 |
| Functions | 78,549 | 20,161 | 9,084 | 7,838 |
| Contracts | 523 | 8 | 0 | 17 |
| Coverage | 0.7% | 28.7% | 0% | 0.2% |

**Cross-repo contract binding**: `contracts/binding.yaml` maps contract
equations to specific function signatures across crate boundaries:

```yaml
# aprender binding.yaml — cross-crate function → contract mapping
bindings:
- contract: softmax-kernel-v1.yaml
  equation: softmax
  module_path: aprender::nn::functional::softmax
  function: softmax
  status: implemented
```

### Fleet Compliance Checks

| Check | Description |
|-------|-------------|
| CB-150 | Cross-crate quality index (sovereign dep scoring) |
| CB-160 | Self-score gate (per-repo Grade A requirement) |
| CB-161 | Category penetration gate (per-repo 80% threshold) |
| CB-1202 | Contract coverage (critical keywords must have contracts) |
| CB-FLEET-001 | Fleet contract coverage > 5% (planned) |
| CB-FLEET-002 | No repo below Grade C in fleet (planned) |

### Research Basis

- Google Monorepo (CACM 2016): per-module health scoring at scale
- Meta (ICSE 2023): signal-to-noise in large-scale CI
- Build System Evolution (ICSE 2024): workspace deps reduce conflicts 34%
- SWE-CI (arXiv 2603.03823): evolution-based quality across commit history

## pmat comply (90+ Checks)

### Check Categories

| Range | Category |
|-------|----------|
| CB-050 - CB-070 | Stub/panic detection, GPU quality |
| CB-081 | Dependency count |
| CB-120 - CB-128 | OIP Tarantula (NaN, locks, serde, tests) |
| CB-130 | Agent context adoption |
| CB-140 | Mono-spec structure enforcement |
| CB-141 | Memory profiling infrastructure |
| CB-142 | SWE-CI EvoScore |
| CB-200 | TDG grade gate |
| CB-300 - CB-304 | Muda waste, reproducibility, golden trace |
| CB-400 | Shell/Makefile quality (bashrs) |
| CB-500+ | Language-specific checks |
| CB-1000+ | Model quality, Lean, provable contracts |

### Configuration

```yaml
# .pmat.yaml
comply:
  checks:
    cb-200:
      enabled: true
      severity: error
      threshold: 60.0  # Minimum TDG grade (C)
    cb-140:
      enabled: true
      severity: warning
    cb-141:
      enabled: true
      severity: warning
    cb-142:
      enabled: true
      severity: info
      options:
        gamma: 1.5
        window: 90
```

## File Health (max-lines)

### Enforcement

Files exceeding line limits are flagged:
- Source files: 500 lines recommended, 1000 max
- Spec files: 500 lines max (mono-spec enforcement)
- Test files: 1000 lines max

## Repository Score

### Health Metrics

| Metric | Weight | Source |
|--------|--------|--------|
| Test coverage | 20% | cargo llvm-cov |
| TDG distribution | 20% | pmat analyze tdg |
| Dependency health | 15% | cargo audit |
| Build performance | 15% | cargo build --timings |
| Documentation | 15% | pmat validate-readme |
| Code freshness | 15% | git log |

## Muda-to-Code Mapping (Planned)

> Tracked in PMAT-540 (repo-health enhancements).

Currently Muda waste categories are abstract project-level numbers. Improvement:
map each waste type to concrete files using existing PMAT data.

| Muda Category | Data Source | Maps To |
|---------------|-------------|---------|
| Inventory | `pmat analyze dead-code` | Files with dead functions (CB-304) |
| Over-processing | `pmat analyze complexity` | Files with complexity > 20 |
| Defects | `pmat comply check` | Files with CB-120 violations (serde panics, NaN) |
| Waiting | `cargo build --timings` | Slowest compilation units |
| Overproduction | `pmat query --duplicates` | Files with code clones |

This would change the Muda output from "Inventory: 16" to
"Inventory: 16 (132 dead items in src/services/cache/, src/workflow/)".

## Key Files

| File | Purpose |
|------|---------|
| `src/services/rust_project_score/` | Rust project scoring |
| `src/cli/handlers/comply_handlers/` | Compliance check framework |
| `src/models/comply_config_defaults.rs` | Check registration |

## References

- Consolidated from: rust-project-score, rust-project-score-v1.1-update,
  current-rust-project-score-implementation-v1, repo-score-spec, repo-score-adjust,
  max-lines, PMAT_COMPLETE_UNIFIED_SPEC, demo-and-book-scoring, improve-pmat-comply,
  cookbook-scoring-spec
