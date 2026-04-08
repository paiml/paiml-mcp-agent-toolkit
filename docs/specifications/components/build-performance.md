# Build Performance

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 3

## Targets

| Metric | Target | Current |
|--------|--------|---------|
| Clean build | <90s | 91s |
| Incremental build | <30s | 26s |
| pmat crate | <70s | 67.4s |
| aprender | <12s | 10.9s |

## Feature Gates

Minimal default features for fast builds with opt-in heavy dependencies:

```toml
[features]
default = ["core-languages", "viz", "http-client"]
core-languages = []          # Rust, Python, TypeScript, JavaScript
extended-languages = []      # Go, JVM, C++, CUDA, etc.
viz = []                     # Terminal visualization
full = ["core-languages", "extended-languages", "viz"]
```

## Optimization Phases

### Phase 1: Quick Wins (Completed)

- `default-features = false` on all dependencies
- Remove unused feature flags (serde, tokio features)
- mold linker available (linking isn't bottleneck)

### Phase 2: Dependency Reduction

Scientific approach to dependency removal:

1. **Measure baseline**: `cargo build --timings`
2. **Identify candidates**: Sort deps by compile time contribution
3. **Evaluate alternatives**: Batuta stack first (aprender vs linfa, trueno vs nalgebra)
4. **A/B test**: Compare build times with/without each dep
5. **Validate**: Full test suite must pass

### Phase 3: Compilation Strategies

- Parallel compilation maximization (codegen-units)
- Profile-guided optimization for release builds
- Shared compilation cache (sccache) for CI

## Dependency Policy

### Sovereign Stack: Aprender Monorepo Migration (crates.io)

Migrate from deprecated standalone crates to unified `aprender-*` monorepo
namespace on crates.io. All at version 0.29. Shared dep tree reduces
transitive duplicates. See [self-enforcement.md](self-enforcement.md) Phase 8.

| Deprecated | Aprender Monorepo | Purpose |
|-----------|-------------------|---------|
| `batuta-common` 0.1 | `aprender-common` 0.29 | Shared utilities |
| `trueno` 0.17 | `aprender-compute` 0.29 | SIMD/GPU compute |
| `trueno-graph` 0.1.17 | `aprender-graph` 0.29 | CSR graph, PageRank |
| `trueno-db` 0.3.16 | `aprender-db` 0.29 | Columnar storage |
| `trueno-rag` 0.2.4 | `aprender-rag` 0.29 | RAG pipeline |
| `trueno-viz` 0.2.3 | `aprender-viz` 0.29 | Terminal visualization |
| `trueno-zram-core` 0.3.1 | `aprender-zram-core` 0.29 | SIMD compression |
| `provable-contracts-macros` | `aprender-contracts-macros` 0.29 | Contract macros |
| `org-intel-plugin` 0.3.4 | `aprender-orchestrate` 0.29 | GitHub org analysis |

**Benefits**: Unified version (0.29), shared dependency tree (fewer
transitive duplicates), single release cadence, deprecated crates
are thin re-export shims.

### Sovereign Stack Priority

| External Dep | Batuta Alternative | Action |
|-------------|-------------------|--------|
| nalgebra | trueno | Replace |
| linfa | aprender | Replace |
| petgraph | trueno-graph | Replace |
| polars | trueno-db | Replace |
| rand | Keep | Foundational |
| rayon | Keep | Foundational |
| roaring | Keep | No equivalent |

### Benchmarking Framework

```bash
# Measure dependency impact
cargo build --timings 2>&1 | grep "Compiling"
# Count total dependencies
cargo tree | wc -l
```

## Runtime Benchmarking & Profiling

**Contract**: `contracts/benchmarking-v1.yaml`

### Performance Budgets (Measured 2026-04-08)

| Operation | Budget | Measured | Status |
|-----------|--------|----------|--------|
| Query (warm, semantic) | <500ms | **179ms** | PASS |
| Query (warm, literal) | <500ms | **209ms** | PASS |
| Query (warm, regex) | <500ms | **178ms** | PASS |
| Index build (4282 files) | <60s | **31s** | PASS |
| RPS fast-mode score | <5s | **1.1s** | PASS |
| Comply check (96 checks) | <30s | **10.4s** | PASS |
| Query (cold, no workspace) | <5s | **0.18s** | PASS |

### Benchmark Command

```bash
# Quick benchmark (runs all critical paths, ~60s)
make bench-quick

# Full benchmark with profiling
make benchmark

# Profile memory (dhat-rs)
cargo run --example dhat_memory_profile -- build
cargo run --example dhat_memory_profile -- query
```

### Regression Detection

Baselines stored in `.pmat-metrics/bench-baseline.json`. Each `make bench-quick`
compares against baseline and flags regressions >20%.

```json
{
  "query_semantic_ms": 179,
  "query_literal_ms": 209,
  "query_regex_ms": 178,
  "index_build_s": 31,
  "rps_fast_s": 1.1,
  "comply_s": 10.4,
  "measured_at": "2026-04-08"
}
```

### Profiling Tools

| Tool | Purpose | Command |
|------|---------|---------|
| dhat-rs | Heap allocation profiling | `cargo run --example dhat_memory_profile` |
| cargo-flamegraph | CPU flamegraph | `cargo flamegraph -- query "test"` |
| `--timings` | Build time per crate | `cargo build --timings` |
| tokio-console | Async task profiling | `RUSTFLAGS="--cfg tokio_unstable"` |

### Memory Baselines (dhat-rs, measured)

| Path | Total Allocs | Peak | Status |
|------|-------------|------|--------|
| Index build | 187 MB | 73.8 MB | Optimized (was 775 MB) |
| Query | 34 MB | 19.8 MB | Lean |
| Deep context | 8.7 GB | 104 MB | Bottleneck: syn parse |

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Feature gates and dependencies |
| `Makefile` | Build targets with timing |
| `.cargo/config.toml` | Linker and compiler settings |
| `contracts/benchmarking-v1.yaml` | Performance budget contract |
| `examples/dhat_memory_profile.rs` | Heap allocation profiler |
| `benches/` | Criterion benchmarks (10 suites) |
| `.pmat-metrics/bench-baseline.json` | Regression baseline |

## References

- Consolidated from: build-performance-optimization-v1.0, build-performance-phase2,
  phase1-build-perf-progress, dependency-reduction-benchmarking-framework,
  reduce-dependencies-maintain-functionality-speedup-compile-testing-spec,
  scientifically-remove-dependencies-time-improve-compile-speed-test-speed
