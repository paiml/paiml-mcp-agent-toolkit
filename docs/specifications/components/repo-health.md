# Repository Health

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 16

## Rust Project Score v2.3

### 274-Point Scoring

```bash
pmat rust-project-score              # Fast mode (~2-3 min)
pmat rust-project-score --full       # Full mode (~10-15 min)
pmat rust-project-score --format json -o score.json
```

### Categories (10)

| Category | Points | Checks |
|----------|--------|--------|
| Rust Tooling & CI/CD | 130 | clippy, fmt, edition, MSRV, CI, audit |
| Code Quality | 26 | complexity, TDG grades, dead code |
| Testing Excellence | 20 | coverage, mutation, property tests |
| Documentation | 15 | doc-tests, README, API docs |
| Build Performance | 15 | build time, incremental, caching |
| Performance & Benchmarking | 10 | benchmarks, binary size |
| Dependency Health | 12 | audit, outdated, count |
| Known Defects | 20 | P0 defects, unwrap audit |
| GPU/SIMD Quality | 10 | CUDA-TDG, barrier safety |
| Formal Verification | 16 | Miri, Kani, Verus, specs |

### Peer-Reviewed Basis

Scoring derived from 15 peer-reviewed papers (2022-2025) on Rust project quality.

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
