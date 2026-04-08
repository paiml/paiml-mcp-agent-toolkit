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

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Feature gates and dependencies |
| `Makefile` | Build targets with timing |
| `.cargo/config.toml` | Linker and compiler settings |

## References

- Consolidated from: build-performance-optimization-v1.0, build-performance-phase2,
  phase1-build-perf-progress, dependency-reduction-benchmarking-framework,
  reduce-dependencies-maintain-functionality-speedup-compile-testing-spec,
  scientifically-remove-dependencies-time-improve-compile-speed-test-speed
