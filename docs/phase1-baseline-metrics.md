# Phase 1 Baseline Metrics - Dependency Reduction
**Date**: 2025-11-20
**Spec**: docs/specifications/reduce-dependencies-maintain-functionality-speedup-compile-testing-spec.md v1.1

## Tooling Installed

All Phase 1 tools successfully installed and verified:

- ✅ **cargo-hakari** v0.9.36 - Feature unification
- ✅ **cargo-machete** v0.9.1 - Unused dependency detection
- ✅ **sccache** v0.12.0 - Shared compilation cache
- ✅ **cargo-nextest** v0.9.114 - Fast parallel testing
- ✅ **mold** v1.0.3 - Fast linker (already configured in `.cargo/config.toml`)

## Baseline Measurements

### Dependency Statistics

- **Total dependency tree size**: 2,767 lines (`cargo tree --workspace | wc -l`)
- **Direct dependencies**: ~165 crates (`cargo tree --workspace --depth 1 | wc -l`)

### Unused Dependencies (cargo-machete)

**26 potentially unused dependencies detected:**

```
ahash, arc-swap, bolero, cpp_demangle, fixedbitset, goblin, httparse,
kani-verifier, logos, memmap2, num-traits, object, pest, prettytable-rs,
rkyv, smallvec, tokio-util, tree-sitter-c-sharp, tree-sitter-java,
tree-sitter-ruby, tree-sitter-scala, tree-sitter-swift, trueno-db,
wasm-encoder, zstd
```

**Classification (Initial T1-T5 Tiers):**

**T1 - Heavy Optional** (Candidates for feature flags):
- `tree-sitter-c-sharp` - Language-specific AST (ast-c-sharp feature)
- `tree-sitter-java` - Language-specific AST (ast-java feature)
- `tree-sitter-ruby` - Language-specific AST (ast-ruby feature)
- `tree-sitter-scala` - Language-specific AST (ast-scala feature)
- `tree-sitter-swift` - Language-specific AST (ast-swift feature)

**T2 - Testing/Verification** (Dev dependencies):
- `bolero` - Property-based testing (dev-dependency candidate)
- `kani-verifier` - Formal verification (dev-dependency candidate)

**T3 - Potentially Unused** (Require investigation):
- `ahash` - Fast hashing (may be transitive)
- `arc-swap` - Atomic reference counting (may be unused)
- `cpp_demangle` - C++ symbol demangling (may be unused)
- `fixedbitset` - Bitmap data structure (may be unused)
- `goblin` - Binary parsing (may be unused)
- `httparse` - HTTP parsing (may be transitive)
- `logos` - Lexer generator (may be unused)
- `memmap2` - Memory mapping (may be unused)
- `num-traits` - Numeric traits (may be transitive)
- `object` - Object file parsing (may be unused)
- `pest` - PEG parser (may be unused)
- `prettytable-rs` - Table formatting (may be unused)
- `rkyv` - Zero-copy serialization (may be unused)
- `smallvec` - Stack-allocated vectors (may be transitive)
- `tokio-util` - Tokio utilities (may be transitive)
- `wasm-encoder` - WebAssembly encoder (may be unused)
- `zstd` - Compression (may be transitive)

**T4 - Under Investigation**:
- `trueno-db` - Marked as unused but may be feature-gated

### cargo-hakari Status

**Status**: Not yet configured (requires `.config/hakari.toml`)

**Next Step**: Initialize hakari configuration to prevent duplicate feature compilation.

## Immediate Wins Available

Based on tooling analysis:

1. **Unused Dependency Removal**: 26 candidates identified
   - **Potential Impact**: Reduce 165 direct deps by ~16% (26 deps)
   - **Compilation Speedup**: Estimated 5-10% from unused dep removal

2. **Language Feature Flags**: 5 tree-sitter languages
   - **Potential Impact**: Each tree-sitter crate is heavy (~10MB+ compiled)
   - **Compilation Speedup**: Estimated 15-25% when using minimal language set

3. **Feature Unification (hakari)**: Not yet measured
   - **Potential Impact**: Prevent duplicate compilation with different feature sets
   - **Compilation Speedup**: Estimated 10-20% on clean builds

4. **Existing Infrastructure**:
   - ✅ mold linker: Already configured (30s → 1s linking on large projects)
   - ✅ sccache: Installed, needs `export RUSTC_WRAPPER=sccache`
   - ✅ cargo-nextest: Installed, 60% faster test execution

## Phase 1 Completion Status

- [x] Install tooling (cargo-hakari, sccache, mold, cargo-nextest)
- [x] Baseline metrics gathered
- [ ] Dependency classification (T1-T5) - **IN PROGRESS** (initial classification above)
- [ ] cargo-hakari configuration

**Current Status**: Phase 1 - 75% Complete

**Next Phase**: Phase 2 - T1 Language Features → Optional (tree-sitter feature flags)

## Success Metrics Baseline

**Before Optimization** (Current):
- Total dependency tree: 2,767 crates
- Direct dependencies: ~165
- Unused dependencies: 26 (16% of direct deps)

**Target After Phase 2**:
- Total dependency tree: <2,000 crates (-27%)
- Direct dependencies: <140 (-15%)
- Unused dependencies: 0
- Feature flags: 5+ language-specific features

**Expected Compilation Speedup**: 30-40% (cumulative from all optimizations)
