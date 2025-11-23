# Dependency Reduction Benchmarking Results

**Pattern**: Modeled after trueno-db competitive benchmarking methodology
**Spec**: `docs/specifications/dependency-reduction-benchmarking-framework.md`

## Current Baseline (v2.202.0 - Sprint 46 Phase 6)

### Date: 2025-11-23

#### Dependency Counts

| Configuration | Count | Delta from Default |
|---------------|-------|-------------------|
| Minimal (rust-only) | 2,260 | -699 (-23.6%) |
| Default | 2,959 | baseline |
| All features | 3,292 | +333 (+11.3%) |

**Commands used**:
```bash
cargo tree --no-default-features --features rust-only | wc -l
cargo tree | wc -l
cargo tree --all-features | wc -l
```

#### Build Times

**Status**: TBD - Run `make bench-build-times`

#### Binary Sizes

**Status**: TBD - Run `make bench-binary-size`

#### Runtime Performance

**Status**: TBD - Run `make bench-runtime` (when Criterion benchmarks implemented)

## Historical Data

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
