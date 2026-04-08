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

### Sovereign Stack: Path Dep Migration (Planned)

Migrate from crates.io versions to local path deps for development velocity
and shared dependency trees. See [self-enforcement.md](self-enforcement.md) Phase 8.

| Crate | crates.io | Path | Purpose |
|-------|-----------|------|---------|
| aprender | 0.27.5 | `../aprender` | ML, stats, text similarity |
| trueno | 0.17 | `../trueno` | SIMD/GPU compute |
| trueno-graph | 0.1.17 | `../trueno/crates/trueno-graph` | CSR graph, PageRank |
| trueno-db | 0.3.16 | `../trueno/crates/trueno-db` | Columnar storage |
| trueno-rag | 0.2.4 | `../trueno/crates/trueno-rag` | RAG pipeline |
| trueno-viz | 0.2.3 | `../trueno/crates/trueno-viz` | Terminal visualization |
| trueno-zram-core | 0.3.1 | `../trueno/crates/trueno-zram-core` | SIMD compression |
| pmcp | 1.10 | `../pmcp` | MCP protocol SDK |
| ruchy | 4.2.1 | `../ruchy` | Parser engine |
| batuta-common | 0.1 | `../batuta-common` | Shared utilities |
| org-intel | 0.3.4 | `../organizational-intelligence` | GitHub org analysis |
| pv-macros | latest | `../provable-contracts/crates/macros` | Contract macros |

**Dual-source pattern**: Use path for development, crates.io for release:
```toml
# Development (checked in, used by default)
aprender = { path = "../aprender" }
# Release override (cargo publish uses crates.io)
# [patch.crates-io]
# aprender = { path = "../aprender" }
```

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
