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

### Sovereign Stack Versions (v3.11.1)

| Crate | Version | Purpose |
|-------|---------|---------|
| trueno | 0.17 | SIMD/GPU compute |
| trueno-graph | 0.1.17 | CSR graph, PageRank |
| trueno-db | 0.3.15 | Columnar storage |
| trueno-rag | 0.2 | RAG pipeline |
| trueno-viz | 0.2 | Terminal visualization |
| aprender | 0.27.5 | ML, stats, text similarity |
| organizational-intelligence-plugin | 0.3.4 | GitHub org analysis |

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
