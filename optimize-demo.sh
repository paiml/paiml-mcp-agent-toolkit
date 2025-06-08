#!/bin/bash
DOC="docs/todo/converge-peak-benchmark-spec.md"

# Get current baseline
echo "📊 Getting baseline performance..."
BASELINE=$(cd server && cargo bench --bench performance 2>/dev/null | grep "time:" | head -2)

cat << EOF > $DOC
# Performance Optimization Checklist

## Current Baseline
$BASELINE

## Optimizations Applied
- [x] Criterion benchmarks configured
- [x] Performance baseline established
- [ ] Replace HashMap with FxHashMap
- [ ] Add rayon for parallel processing
- [ ] Use SmallVec for small collections
- [ ] Add inline hints to hot functions
- [ ] Enable LTO in release builds
- [ ] Profile-guided optimization

## Optimization Opportunities

### 1. FxHashMap (rustc-hash)
Replace \`std::collections::HashMap\` with \`rustc_hash::FxHashMap\` for:
- 2-3x faster hashing for small keys
- Better cache locality
- Lower memory overhead

Example:
\`\`\`rust
use rustc_hash::FxHashMap;
// Before: HashMap<String, String>
// After: FxHashMap<String, String>
\`\`\`

### 2. Parallel Processing (rayon)
Use rayon's parallel iterators for file processing:
\`\`\`rust
use rayon::prelude::*;
files.par_iter()
    .map(|file| analyze_file(file))
    .collect()
\`\`\`

### 3. SmallVec for Small Collections
Eliminate heap allocations for small collections:
\`\`\`rust
use smallvec::SmallVec;
// Before: Vec<String>
// After: SmallVec<[String; 8]>
\`\`\`

### 4. Inline Hints
Add to frequently called small functions:
\`\`\`rust
#[inline(always)]
pub fn analyze_rust_file(path: &Path) -> Result<FileContext> {
    // Hot path code
}
\`\`\`

### 5. Link Time Optimization
Add to Cargo.toml:
\`\`\`toml
[profile.release]
lto = true
codegen-units = 1
\`\`\`

## Benchmark Commands

1. **Run benchmarks**: \`cargo criterion --bench performance\`
2. **Generate flamegraph**: \`cargo flamegraph --bench performance\`
3. **Profile specific function**: \`perf record -g target/release/pmat analyze complexity .\`

## Expected Improvements

| Operation | Current | Target | Speedup |
|-----------|---------|--------|---------|
| analyze_rust_file | 844 µs | <100 µs | 8x |
| analyze_project | 9.5 ms | <1 ms | 10x |
| Memory usage | Baseline | -50% | 2x |

## Next Steps
1. Add rustc-hash dependency
2. Implement parallel file processing
3. Profile with flamegraph to identify remaining bottlenecks
EOF

echo "✅ Optimization plan created in $DOC"