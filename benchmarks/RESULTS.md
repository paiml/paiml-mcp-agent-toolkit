# Dependency Reduction Benchmarking Results

**Pattern**: Modeled after trueno-db competitive benchmarking methodology
**Spec**: `docs/specifications/dependency-reduction-benchmarking-framework.md`

## Current Baseline (v2.203.0 - Sprint 46 Phase 7: Dependency Cleanup)

### Date: 2025-11-23

#### Dependency Counts

| Configuration | Count | Delta from Default | Change from v2.202.0 |
|---------------|-------|-------------------|----------------------|
| Minimal (rust-only) | 2,055 | -699 (-25.4%) | -205 (-9.1%) |
| Default | 2,754 | baseline | -205 (-6.9%) |
| All features | 2,937 | +183 (+6.6%) | -355 (-10.8%) |

**Commands used**:
```bash
cargo tree --no-default-features --features rust-only | wc -l
cargo tree | wc -l
cargo tree --all-features | wc -l
```

#### Build Times

**Tool**: bashrs bench (v6.25.0) - Scientific rigor with warmup iterations

**Commands**:
```bash
./benchmarks/bench-build.sh  # Uses bashrs bench for all build configs
```

**Results**: JSON output in `benchmarks/results/`
- Dev build: `dev-TIMESTAMP.json`
- Release build: `release-TIMESTAMP.json`
- Minimal build: `minimal-TIMESTAMP.json`

**Status**: ✅ Framework ready - Run `./benchmarks/bench-build.sh`

#### Binary Sizes

**Measurement**: `stat --format=%s target/release/pmat`

**Status**: TBD - Add to bench-build.sh script

#### Runtime Performance

**Tool**: renacer v0.5.0 (syscall tracing + timing mode)

**Usage**:
```bash
renacer -T pmat context --output /tmp/ctx.md  # Timing mode
renacer -c pmat context --output /tmp/ctx.md  # Statistics mode
```

**Status**: ✅ Available - Use renacer for detailed profiling

## Historical Data

### Sprint 46 Phase 7 (Dependency Cleanup - 2025-11-23)

**Changes**:
- Removed 11 unused dependencies (9 regular + 2 dev dependencies)
- Dependencies identified via cargo-machete analysis
- All removals verified via manual grep analysis

**Dependencies Removed**:

*Regular dependencies (9):*
1. `arc-swap` - Lock-free atomic pointer swapping
2. `bolero` - Property-based fuzzing framework
3. `kani-verifier` - Formal verification tool
4. `memmap2` - Memory-mapped file I/O
5. `num-traits` - Numeric trait abstractions
6. `renacer` - System call tracer
7. `rkyv` - Zero-copy deserialization
8. `tokio-util` - Tokio utility library
9. `zstd` - Zstandard compression

*Dev dependencies (2):*
1. `actix-test` - Actix testing utilities
2. `mockall` - Mock object library

**Results**:
- Dependency reduction:
  - Minimal (rust-only): -205 (-9.1%)
  - Default: -205 (-6.9%)
  - All features: -355 (-10.8%)
- ✅ Build verification: `cargo check --lib` passed
- ✅ cargo-machete verification: Zero unused dependencies remaining

**Commits**:
- TBD - Remove 11 unused dependencies

### Sprint 46 Phase 6 (Tree-Sitter Removal)

**Changes**:
- Removed 5 unused tree-sitter parsers (c-sharp, java, ruby, scala, swift)
- Feature-gated mutation testing module
- Implemented O(1) hash-based build caching

**Results**:
- Dependency reduction: 6.9% for rust-only configuration
- Compilation errors resolved: 147 → 0 (100% reduction)
- Feature gates added: 9 total

**Commits**:
- `ee2618d8` - Remove 5 unused tree-sitter parsers
- `27fea2ae` - Implement O(1) hash-based caching for build artifacts
- `2c1ef107` - Gate mutation CLI handlers
- `d1d2e1bf` - Gate MCP mutation tools

### Phase 2B Complete (Feature Gating)

**Status**: ✅ Complete
- All 23 files feature-gated successfully
- 100% error reduction for `--features rust-only` build
- Zero regression in default configuration

## Recommendations

### Current (2025-11-23)

1. **Development**: Use `--features rust-only` for fast iteration
   - 23.6% fewer dependencies (2,260 vs 2,959)
   - Faster build times (estimated 15-20% reduction)

2. **Testing**: Use default features for full functionality
   - All language parsers available
   - Mutation testing enabled

3. **CI**: Consider `--features rust-only` for faster CI runs
   - Reduced dependency download time
   - Faster compilation
   - Smaller binary footprint

### Next Steps

1. ✅ Dependency counting - DONE
2. ⏳ Build time measurement - Run `make bench-build-times`
3. ⏳ Binary size measurement - Run `make bench-binary-size`
4. ⏳ Runtime benchmarks - Implement Criterion benchmarks
5. ⏳ CI integration - Add to GitHub Actions

## Methodology

All benchmarks follow the scientific methodology defined in:
- `docs/specifications/dependency-reduction-benchmarking-framework.md`
- Pattern: trueno-db competitive benchmarking
- Tools: cargo-tree, cargo-bloat, Criterion.rs, GNU time

### Measurement Protocol

1. **Environment standardization**: Clear caches, sync filesystem
2. **Multiple runs**: Minimum 3 runs, report median
3. **Regression tracking**: Compare against baseline.md
4. **CI integration**: Automated benchmarking on PRs

### Quality Gates

| Metric | Max Regression | Status |
|--------|----------------|--------|
| Dependency count | +50 | ⚠️  Warning |
| Build time | +5% | ⚠️  Warning |
| Binary size | +2 MB | ❌ Block |
| Runtime | +20% | ⚠️  Warning |

---

**Last Updated**: 2025-11-23
**Tool**: benchmarks/measure-baseline.sh
**Pattern**: trueno-db methodology
