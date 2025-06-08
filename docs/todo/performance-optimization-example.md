# Performance Optimization Example

## Current Hot Paths (from benchmarks)

### 1. AST Analysis
- **Current**: 844.35 µs per file
- **Target**: <100 µs per file
- **Bottleneck**: HashMap allocations and string cloning

### 2. Project Analysis  
- **Current**: 9.4586 ms for small project
- **Target**: <1 ms for small project
- **Bottleneck**: Sequential file processing

## Quick Win Optimizations

### 1. Add inline hints to hot functions
```rust
#[inline(always)]
fn collect_nodes(&mut self, file: &FileContext) {
    // Hot path - called for every file
}
```

### 2. Replace HashMap with FxHashMap
```rust
use rustc_hash::FxHashMap;
// 2-3x faster for small keys
function_map: FxHashMap<String, String>,
```

### 3. Use SmallVec for small collections
```rust
use smallvec::SmallVec;
// No heap allocation for <8 items
edges: SmallVec<[Edge; 8]>,
```

### 4. Parallel file processing
```rust
use rayon::prelude::*;
project.files.par_iter()
    .map(|file| analyze_file(file))
    .collect()
```

## Measurement Commands

1. **Criterion benchmarks**: `cargo criterion --bench performance`
2. **Flame graph**: `cargo flamegraph --bench performance`
3. **Assembly output**: `cargo asm --lib --rust "services::dag_builder::DagBuilder::build_from_project"`
4. **Profile-guided optimization**: `RUSTFLAGS="-Cprofile-use=pgo.profdata" cargo build --release`

## Expected Results

| Operation | Before | After | Improvement |
|-----------|--------|-------|-------------|
| AST parse | 844 µs | 100 µs | 8.4x |
| Project scan | 9.5 ms | 1.0 ms | 9.5x |
| DAG build | 50 ms | 5 ms | 10x |
| Memory usage | 100 MB | 20 MB | 5x |