# Phase 2: Build Performance Optimization - Minimal Default Features

**Issue**: #89 (continued from Phase 1)
**Status**: SPECIFICATION - Ready for Implementation
**Date**: 2025-11-24
**Version**: 2.205.0 → 2.206.0

## Objective

Reduce build time and duplicate dependencies by implementing **minimal default feature set**, making advanced features opt-in.

**Phase 1 Results Recap**:
- Build time: 163s → 143s (-12.3%)
- Duplicates: 166 → 157 (-5.4%)
- **Need more aggressive action** to reach targets

**Phase 2 Targets**:
- **Build time**: 143s → ~100s (-30% additional)
- **Duplicates**: 157 → <100 (-36%)
- **Core principle**: "Fast by default, powerful by opt-in"

## Root Cause Analysis (Five Whys)

### Why are there still 157 duplicate dependencies?

**Answer**: Too many features enabled by default pull in conflicting dependency versions.

**Evidence**:
```toml
# Current defaults (7 features)
default = [
    "all-languages",      # Multi-language support (heavy)
    "demo",               # Demo/visualization (heavy)
    "polyglot-ast",       # Multi-language AST (heavy)
    "tdg-explain",        # TDG explanations (moderate)
    "analytics-simd",     # SIMD analytics (heavy)
    "mutation-testing",   # Mutation testing (very heavy)
]
```

### Why do these features cause duplicates?

**Key duplicates identified** (cargo tree -d):

1. **axum**: v0.6.20 (libsql/tonic) vs v0.8.4 (direct)
   - Source: libsql v0.9.29 depends on tonic v0.11 → axum v0.6
   - Our code uses axum v0.8
   - **Impact**: 2 full copies of axum framework

2. **base64**: v0.21.7 (warp/libsql) vs v0.22.1 (arrow/hyper-util)
   - Source: warp v0.3.7 → headers → base64 v0.21
   - Arrow ecosystem uses base64 v0.22
   - **Impact**: Redundant base64 implementations

3. **hyper-rustls**: v0.26.0 vs v0.27.7
   - Source: octocrab v0.40 uses old version
   - Our direct deps use new version
   - **Impact**: Duplicate TLS implementations

4. **Many others**: axum-core, bit-set, bytes, futures-*, http-*, etc.

### Why are these heavy features in defaults?

**Historical reasons**:
- Started as comprehensive analysis tool
- Added features incrementally to defaults
- Never revisited what's "essential" vs "nice to have"

**Toyota Way Lesson**: **Muda** (waste) - Building features most users don't need.

## Phase 2 Strategy

### 1. Define Minimal Core (NEW defaults)

**Philosophy**: Fast, focused analysis for 90% of use cases.

**Proposed minimal defaults**:
```toml
default = [
    "core-languages",    # Rust + JS/TS only (NEW)
    "basic-quality",     # Complexity, dead code, SATD (NEW)
]
```

**Remove from defaults** (make opt-in):
```toml
# Heavy features → opt-in
all-languages        # 30+ languages → feature flag
demo                 # Visualization → feature flag
polyglot-ast         # Multi-language AST → feature flag
analytics-simd       # SIMD analytics → feature flag
mutation-testing     # Mutation testing → feature flag
tdg-explain          # TDG explanations → feature flag
```

### 2. Create Feature Bundles

**For user convenience**, create logical bundles:

```toml
# Minimal (default)
core-languages = ["rust", "typescript", "javascript"]
basic-quality = ["complexity", "dead-code", "satd"]

# Extended language support
extended-languages = ["python", "go", "java", "cpp"]
all-languages = ["core-languages", "extended-languages", "bash", "php", "swift", "kotlin", "ruby", "wasm"]

# Advanced analysis
advanced-analysis = ["analytics-simd", "mutation-testing", "tdg-explain"]

# Full suite (equivalent to old defaults)
full = ["all-languages", "demo", "polyglot-ast", "advanced-analysis"]
```

### 3. Dependency Cleanup

**Target specific duplicates**:

#### 3.1 axum Duplicate (v0.6 vs v0.8)
- **Root cause**: libsql → tonic v0.11 → axum v0.6
- **Solution**: Make libsql opt-in (storage feature)
- **Impact**: Eliminates axum v0.6, axum-core v0.3, tower v0.4 duplicates

#### 3.2 base64 Duplicate (v0.21 vs v0.22)
- **Root cause**: warp → headers → base64 v0.21
- **Analysis**: warp is ONLY used for demo web server
- **Solution**: Move warp to demo feature (already opt-in)
- **Impact**: Eliminates base64 v0.21, headers, http duplicates

#### 3.3 Upgrade octocrab (if possible)
- **Current**: octocrab v0.40 uses old hyper-rustls
- **Check**: Is octocrab v0.41+ available with newer deps?
- **Impact**: May eliminate hyper-rustls v0.26 duplicate

### 4. Profile Slow Dependencies

**Identify compilation hotspots**:
```bash
# Measure per-crate build time
cargo build --timings

# Focus on crates that take >10s to compile
# Examples: arrow, ruchy, trueno, wgpu
```

**Candidates for opt-in**:
- **arrow**: 15-20s compile (used for analytics-simd)
- **wgpu**: 10-15s compile (used for GPU acceleration)
- **ruchy**: 8-12s compile (WASM interpreter)

## Implementation Plan

### Step 1: Create Minimal Feature Set (Issue #91)

**Files to modify**:

1. **`server/Cargo.toml`** (line 286-291)
   ```toml
   # BEFORE (Phase 1):
   default = ["all-languages", "demo", "polyglot-ast", "tdg-explain", "analytics-simd", "mutation-testing"]

   # AFTER (Phase 2):
   default = ["core-languages", "basic-quality"]

   # Feature bundles
   core-languages = ["rust", "typescript", "javascript"]
   basic-quality = ["complexity", "dead-code", "satd"]

   extended-languages = ["python", "go", "java", "cpp"]
   all-languages = ["core-languages", "extended-languages", "bash", "php", "swift", "kotlin", "ruby", "wasm"]

   advanced-analysis = ["analytics-simd", "mutation-testing", "tdg-explain"]

   full = ["all-languages", "demo", "polyglot-ast", "advanced-analysis"]
   ```

2. **Language-specific feature gates**:
   - Move each language analyzer to feature flag
   - Example: `#[cfg(feature = "python")]` for Python analyzer

3. **Complete feature gate implementation**:
   - All advanced features properly gated
   - No compilation errors with minimal defaults
   - Helpful error messages for disabled features

### Step 2: Optimize Heavy Dependencies (Issue #92)

**Actions**:

1. **Make libsql opt-in**:
   ```toml
   [dependencies]
   libsql = { version = "0.9.29", optional = true }

   [features]
   storage = ["libsql"]
   ```

2. **Profile and optimize**:
   ```bash
   cargo build --timings
   # Analyze build-timings.html for hotspots
   ```

3. **Consider replacements**:
   - warp → axum (for demo server)
   - Investigate lighter alternatives for heavy crates

### Step 3: Regenerate workspace-hack

```bash
cargo hakari generate
cargo hakari verify
```

### Step 4: Measure Improvements

```bash
./scripts/benchmark_build.sh
# Compare against Phase 1 baseline
```

### Step 5: Version Bump

**Version**: `2.205.0` → `2.206.0`

Update in:
- `Cargo.toml` (workspace root, line 7)

## Expected Results

### Build Time Improvement

**Assumptions**:
- Minimal defaults eliminate ~40% of crates
- Heavy crates (arrow, wgpu, ruchy) are opt-in

**Projected**:
- Phase 1: 163s → 143s (-12.3%)
- Phase 2: 143s → ~100s (-30%)
- **Total**: 163s → 100s (-39% cumulative)

**Evidence basis**:
- arrow + wgpu + ruchy = ~40-50s combined
- Removing from defaults saves this time

### Duplicate Dependencies

**Projected reductions**:
- axum duplicate: -15 crates (tower, axum-core, etc.)
- base64 duplicate: -8 crates (headers, http, etc.)
- Other ecosystem duplicates: -20 crates
- **Total**: 157 → ~90 (-43%)

### Developer Experience

**Fast by default**:
- New users get <2 min builds (vs 2m 43s)
- CI/CD runs faster
- Pre-commit hooks complete faster

**Powerful by opt-in**:
- Advanced users can enable full suite
- Clear feature documentation
- `--features full` for complete capabilities

## Testing Strategy

### 1. Compilation Tests

**Verify all feature combinations**:
```bash
# Minimal defaults
cargo check

# Individual features
cargo check --features python
cargo check --features analytics-simd

# Feature bundles
cargo check --features extended-languages
cargo check --features advanced-analysis
cargo check --features full

# No defaults
cargo check --no-default-features
```

### 2. Functional Tests

**Ensure core functionality works**:
```bash
# Build with minimal defaults
cargo build --release

# Test core commands
pmat context --path .
pmat analyze complexity --path src/
pmat quality-gate --path .
```

### 3. Integration Tests

**Verify pmat-book still passes**:
```bash
make validate-book
```

## Risk Mitigation

### Risk 1: Breaking Changes for Users

**Mitigation**:
- Provide `full` feature for backward compatibility
- Document migration in CHANGELOG.md
- Clear error messages for disabled features

**Example error message**:
```
Error: Python analysis not available.
This feature requires --features python or --features all-languages
Install with: cargo install pmat --features all-languages
```

### Risk 2: Feature Gate Complexity

**Mitigation**:
- Follow Phase 1 feature gate pattern
- Complete conditional compilation
- Comprehensive compilation tests

### Risk 3: Unexpected Duplicates

**Mitigation**:
- Run `cargo tree -d` frequently
- Use `cargo hakari verify` to catch issues
- Document any persistent duplicates

## Success Criteria

**Phase 2 is successful if**:

1. ✅ **Build time**: <100s clean build with minimal defaults
2. ✅ **Duplicates**: <100 duplicate crates
3. ✅ **Compilation**: All feature combinations compile
4. ✅ **Tests**: All tests pass with minimal defaults
5. ✅ **Book**: pmat-book validation passes
6. ✅ **Quality**: rust-project-score ≥110/134

## Next Steps (After Phase 2)

### Phase 3: Parallel Testing (Issue #93)
- Integrate cargo-nextest
- Parallel test execution
- Target: Test time 8m → 3m

### Phase 4: Pre-commit Optimization
- Optimize pre-commit hooks
- Incremental validation
- Target: Pre-commit <30s

## References

- Phase 1 Progress: `docs/specifications/phase1-build-perf-progress.md`
- Original Spec: `docs/specifications/build-performance-optimization-v1.0.md`
- GitHub Issues: #89, #91, #92
- Baseline: `.pmat-metrics/build-benchmarks/baseline_20251124_114748.txt`

## Toyota Way Principles

- **Muda** (Waste Elimination): Remove features most users don't need from defaults
- **Jidoka** (Built-in Quality): Feature gates prevent compilation errors
- **Kaizen** (Continuous Improvement): Measure, improve, measure again
- **Genchi Genbutsu** (Go and See): Profile actual build times, identify real bottlenecks

## Appendix: Current Duplicate Summary

**Total duplicates**: 157 (from Phase 1)

**Top 10 duplicate groups** (by impact):
1. axum (v0.6.20 vs v0.8.4) - web framework
2. axum-core (v0.3.4 vs v0.5.5) - web framework core
3. base64 (v0.21.7 vs v0.22.1) - encoding
4. bytes (v1.5.0 vs v1.8.0) - byte buffers
5. futures-* (multiple versions) - async runtime
6. http (v0.2.12 vs v1.2.0) - HTTP types
7. http-body (v0.4.6 vs v1.1.0) - HTTP body types
8. hyper (v0.14.31 vs v1.6.0) - HTTP client/server
9. hyper-rustls (v0.26.0 vs v0.27.7) - TLS support
10. tower (v0.4.13 vs v0.5.4) - service abstraction

**Root causes**:
- libsql using old tonic/axum
- warp using old http ecosystem
- Arrow/trueno using latest versions
- octocrab using old hyper-rustls

**Phase 2 will target these aggressively** by making heavy features opt-in.
