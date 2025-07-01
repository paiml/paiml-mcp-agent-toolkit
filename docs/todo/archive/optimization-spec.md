# Performance Optimization Specification

## Overview
This document tracks all identified performance optimization opportunities in the paiml-mcp-agent-toolkit codebase. Each item represents a specific optimization that can improve compilation time, runtime performance, or memory usage.

## Critical Issues Found
1. **Regex Compilation in Loops** - `src/cli/mod.rs` compiles regex for every line (1M+ times)
2. **42 HashMap/HashSet without capacity** - Unnecessary allocations in hot paths
3. **Multiple O(n²) nested loops** - Can be optimized to O(n log n) or O(n)
4. **String concatenation in loops** - Missing pre-allocation causing repeated memory allocation

## Quick Wins (Do These First!)
- [✅] Fix regex compilation in `src/cli/mod.rs` - **Est. 20-30% improvement**
- [✅] Pre-allocate HashMaps in protocol handling - **Est. 5-10% improvement**  
- [✅] Pre-allocate strings in report generation - **Est. 10-15% improvement**

## Status Legend
- [ ] Not started
- [🔄] In progress
- [✅] Completed
- [❌] Blocked or not feasible

## 1. Memory Allocation Optimizations

### 1.1 Vector Pre-allocation
- [✅] `src/handlers/tools.rs` - Pre-allocate vectors in validation functions
- [✅] `src/testing/vectorized_correctness.rs` - Pre-allocate test result vectors
- [✅] `src/testing/simd_validators.rs` - Pre-allocate error/warning collections
- [ ] `src/services/file_discovery.rs` - Pre-allocate file list vectors
- [ ] `src/services/ranking.rs` - Pre-allocate ranking result vectors
- [ ] `src/services/dag_builder.rs` - Pre-allocate node/edge collections
- [ ] `src/services/complexity.rs` - Pre-allocate metric collections
- [ ] `src/cli/handlers/analysis.rs` - Pre-allocate analysis results

### 1.2 String Buffer Optimization
- [✅] `src/handlers/tools.rs` - Pre-allocate string buffers for formatting
- [✅] `src/demo/runner.rs` - Pre-allocate output strings
- [✅] `src/services/deep_context.rs` - Pre-allocate context strings
- [ ] `src/services/mermaid_generator.rs` - Optimize Mermaid diagram generation
- [ ] `src/services/renderer.rs` - Pre-allocate template rendering buffers
- [ ] `src/services/enhanced_reporting.rs` - Optimize report generation
- [ ] `src/cli/formatting_helpers.rs` - Pre-allocate formatting buffers

### 1.3 HashMap/HashSet Capacity
- [✅] `src/testing/properties.rs` - Pre-size hash collections
- [✅] `src/unified_protocol/test_harness.rs` - Pre-size protocol maps
- [ ] `src/services/symbol_table.rs` - Pre-size symbol tables based on file count
- [ ] `src/services/cache/*.rs` - Pre-size cache structures
- [ ] `src/models/dag.rs` - Pre-size graph adjacency lists

## 2. Algorithmic Complexity Improvements

### 2.1 O(n²) → O(n log n) Optimizations
- [ ] `src/services/duplicate_detector.rs` - Replace nested loops with hash-based detection
- [ ] `src/services/name_similarity_analysis.rs` - Use sorted comparisons
- [ ] `src/services/ast_based_dependency_analyzer.rs` - Optimize dependency resolution
- [ ] `src/cli/analysis/name_similarity.rs` - Improve similarity calculations

### 2.2 O(n) → O(1) Lookups
- [ ] `src/services/git_analysis.rs` - Cache git object lookups
- [ ] `src/services/project_meta_detector.rs` - Index project metadata
- [ ] `src/services/file_classifier.rs` - Use lookup tables for file types

### 2.3 Memoization Opportunities
- [ ] `src/services/complexity_patterns.rs` - Memoize pattern matching
- [ ] `src/services/ast_rust.rs` - Cache AST parsing results
- [ ] `src/services/ast_typescript.rs` - Cache TypeScript AST results
- [ ] `src/services/verified_complexity.rs` - Memoize complexity calculations

## 3. Compilation Time Optimizations

### 3.1 Inline Hints for Hot Functions
- [✅] `src/services/dead_code_analyzer.rs` - Analysis functions
- [✅] `src/services/git_clone.rs` - Clone operations
- [✅] `src/services/ast_typescript_dispatch.rs` - Dispatch functions
- [✅] `src/services/mermaid_generator.rs` - Graph generation functions
- [ ] `src/services/deep_context_orchestrator.rs` - Orchestration methods
- [ ] `src/services/ranking.rs` - Ranking calculations
- [ ] `src/handlers/resources.rs` - Resource handling

### 3.2 Generic Specialization
- [ ] `src/models/unified_ast.rs` - Specialize for common AST node types
- [ ] `src/services/cache/unified.rs` - Specialize cache for common types
- [ ] `src/testing/arbitrary.rs` - Specialize generators

### 3.3 Const Functions
- [ ] `src/models/complexity_bound.rs` - Make bound checks const
- [ ] `src/services/file_classifier.rs` - Const file type detection
- [ ] `src/utils/helpers.rs` - Const utility functions

## 4. I/O and Async Optimizations

### 4.1 Buffered I/O
- [ ] `src/services/file_discovery.rs` - Buffer file system operations
- [ ] `src/services/git_clone.rs` - Buffer git operations
- [ ] `src/services/template_service.rs` - Buffer template reads

### 4.2 Parallel Processing
- [ ] `src/services/ast_based_dependency_analyzer.rs` - Parallel file analysis
- [ ] `src/services/complexity.rs` - Parallel complexity calculation
- [ ] `src/services/duplicate_detector.rs` - Parallel duplicate detection
- [ ] `src/cli/handlers/analysis.rs` - Parallel analysis execution

### 4.3 Async Optimization
- [ ] `src/handlers/tools.rs` - Batch async operations
- [ ] `src/demo/server.rs` - Optimize request handling
- [ ] `src/services/deep_context.rs` - Concurrent context building

## 5. Cache Optimization

### 5.1 Cache Strategy
- [ ] `src/services/cache/manager.rs` - Implement LRU eviction
- [ ] `src/services/cache/persistent.rs` - Optimize SQLite queries
- [ ] `src/services/cache/content_cache.rs` - Implement content deduplication

### 5.2 Cache Key Optimization
- [ ] Use faster hashing (xxHash or wyhash instead of default hasher)
- [ ] Implement hierarchical caching for nested structures
- [ ] Add cache warming for frequently accessed data

## 6. Data Structure Optimizations

### 6.1 Replace Heavy Structures
- [ ] `src/models/dag.rs` - Use petgraph's StableGraph for better performance
- [ ] `src/services/symbol_table.rs` - Use FxHashMap for faster lookups
- [ ] `src/services/ast_strategies.rs` - Use SmallVec for small collections

### 6.2 Avoid Unnecessary Cloning
- [ ] `src/services/deep_context.rs` - Use references where possible
- [ ] `src/models/unified_ast.rs` - Implement Copy for small types
- [ ] `src/services/ranking.rs` - Pass by reference

### 6.3 String Interning
- [ ] `src/services/symbol_table.rs` - Intern symbol names
- [ ] `src/models/unified_ast.rs` - Intern common AST strings
- [ ] `src/services/file_classifier.rs` - Intern file extensions

## 7. Build Configuration Optimizations

### 7.1 Profile-Guided Optimization
- [ ] Set up PGO build pipeline
- [ ] Create representative workload for profiling
- [ ] Measure PGO impact on performance

### 7.2 Link-Time Optimization
- [ ] Enable LTO for release builds
- [ ] Test thin vs fat LTO trade-offs
- [ ] Measure binary size impact

### 7.3 Codegen Options
- [ ] Experiment with codegen-units settings
- [ ] Test different opt-level configurations
- [ ] Enable CPU-specific optimizations

## 8. Algorithm-Specific Optimizations

### 8.1 Graph Algorithms
- [ ] `src/services/dag_builder.rs` - Use Tarjan's algorithm for cycle detection
- [ ] `src/services/ranking.rs` - Implement PageRank with sparse matrices
- [ ] `src/services/ast_based_dependency_analyzer.rs` - Use topological sort

### 8.2 String Matching
- [ ] `src/services/duplicate_detector.rs` - Use rolling hash for similarity
- [ ] `src/services/satd_detector.rs` - Use Aho-Corasick for pattern matching
- [ ] `src/cli/analysis/name_similarity.rs` - Use edit distance with early termination

### 8.3 Numerical Computations
- [ ] `src/services/defect_probability.rs` - Use SIMD for vector operations
- [ ] `src/services/tdg_calculator.rs` - Vectorize gradient calculations
- [ ] `src/testing/simd_validators.rs` - Optimize validation loops

## 9. Memory Layout Optimizations

### 9.1 Struct Packing
- [ ] Analyze struct layouts with `#[repr(C)]` for cache efficiency
- [ ] Reorder fields to minimize padding
- [ ] Use bitfields where appropriate

### 9.2 Arena Allocation
- [ ] `src/models/unified_ast.rs` - Use arena for AST nodes
- [ ] `src/services/symbol_table.rs` - Arena for symbol storage
- [ ] `src/services/dag_builder.rs` - Arena for graph nodes

## 10. Specific Unoptimized Code Paths

### 10.1 Hot Paths Identified by Profiling
```rust
// src/services/mermaid_generator.rs:generate_mermaid_graph()
// Currently: O(n²) edge generation
// TODO: Pre-sort nodes and use binary search
```

```rust
// src/services/deep_context.rs:build_context()
// Currently: Multiple passes over file list
// TODO: Single pass with streaming processing
```

```rust
// src/services/ranking.rs:calculate_rankings()
// Currently: Sorts entire list multiple times
// TODO: Use partial sorting with heap
```

### 10.2 Memory Allocation Hotspots
```rust
// src/services/enhanced_reporting.rs:generate_report()
// Currently: Concatenates strings in loop
// TODO: Use String::with_capacity() and push_str()
```

```rust
// src/services/ast_based_dependency_analyzer.rs:analyze_dependencies()
// Currently: Creates many temporary vectors
// TODO: Reuse buffers across iterations
```

### 10.3 Inefficient Algorithms
```rust
// src/services/duplicate_detector.rs:find_duplicates()
// Currently: Compares all pairs O(n²)
// TODO: Use hash-based fingerprinting O(n)
```

### 10.4 Critical Performance Issues Found

#### Regex Compilation in Loops
- [✅] **CRITICAL**: `src/cli/mod.rs:9089,9138,9155,9285` - Regex compiled inside loop for every line
```rust
// Current (BAD):
for pattern in &[...] {
    if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(trimmed) {
        // Process matches
    }
}

// TODO: Use lazy_static or once_cell:
lazy_static! {
    static ref PATTERNS: Vec<Regex> = vec![
        Regex::new(r"pattern1").unwrap(),
        Regex::new(r"pattern2").unwrap(),
    ];
}
```

#### HashMap/HashSet Without Capacity (42 instances)
- [ ] `src/tests/churn.rs:38` - `HashMap::new()` without capacity
- [ ] `src/tests/code_smell_comprehensive_tests.rs:252-253` - Multiple collections
- [ ] `src/unified_protocol/service.rs:1097` - Protocol params map
- [ ] `src/tests/diagnose_tests.rs:121` - Error patterns map

#### Nested Loops (O(n²) complexity)
- [ ] `src/handlers/tools.rs:1360-1361` - Files × Functions iteration
- [ ] `src/handlers/tools.rs:1801-1802` - File metrics × Items iteration
- [ ] `src/cli/symbol_table_helpers.rs:92-93` - AST contexts × Items iteration

#### Unnecessary Cloning
- [✅] `src/services/satd_detector.rs:249` - Cloning regex strings
```rust
// Current:
let regex_strings: Vec<String> = patterns.iter().map(|p| p.regex.clone()).collect();
// TODO:
let regex_strings: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
```

- [ ] `src/services/mermaid_property_tests.rs:70` - Cloning node IDs
```rust
// Current:
let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
// TODO:
let node_ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
```

#### File I/O Without Buffering
- [✅] `src/services/ranking.rs:707,710,741,830,844` - Direct file writes in tests
```rust
// Current:
let mut f1 = File::create(&file1).unwrap();
writeln!(f1, "fn small() {{}}").unwrap();

// TODO:
let f1 = File::create(&file1).unwrap();
let mut writer = BufWriter::new(f1);
writeln!(writer, "fn small() {{}}").unwrap();
```

## 11. Monitoring and Validation

### 11.1 Performance Regression Tests
- [ ] Add criterion benchmarks for critical paths
- [ ] Set up CI performance monitoring
- [ ] Create performance dashboard

### 11.2 Memory Usage Tracking
- [ ] Add memory profiling to test suite
- [ ] Track allocation patterns
- [ ] Monitor peak memory usage

### 11.3 Optimization Validation
- [ ] Verify optimizations don't break functionality
- [ ] Measure actual performance improvements
- [ ] Document optimization trade-offs

## Priority Ranking

### High Priority (Do First)
1. Memory pre-allocation for hot paths
2. O(n²) algorithm improvements
3. String buffer optimization in report generation
4. Cache implementation improvements
5. Parallel processing for analysis

### Medium Priority
1. Inline hints for remaining hot functions
2. HashMap pre-sizing
3. Async operation batching
4. SIMD optimizations
5. Build configuration tuning

### Low Priority
1. String interning
2. Arena allocation
3. Struct packing
4. Const functions
5. Generic specialization

## Estimated Impact

| Optimization Category | Estimated Improvement | Effort | Risk |
|--------------------- |----------------------|--------|------|
| Memory Pre-allocation | 10-20% | Low | Low |
| Algorithm Complexity | 20-50% | High | Medium |
| Build Configuration | 15-30% | Low | Low |
| Parallel Processing | 30-60% | Medium | Medium |
| Cache Optimization | 20-40% | Medium | Low |
| SIMD/Vectorization | 10-30% | High | Medium |

## 12. Most Impactful Optimizations (Quick Wins)

### 12.1 Regex Compilation Fix (Est. 20-30% improvement)
The regex compilation in loops is the most critical issue. In `src/cli/mod.rs`, patterns are compiled for every line of every file.

**Impact**: For a 10,000 line codebase with 100 files, this creates 1,000,000 regex compilations instead of ~10.

**Fix**:
```rust
// Add at module level:
lazy_static! {
    static ref TODO_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"TODO|FIXME|HACK|XXX").unwrap(),
        // ... other patterns
    ];
}
```

### 12.2 HashMap Pre-allocation (Est. 5-10% improvement)
42 instances of HashMap/HashSet created without capacity when size is predictable.

**Fix Priority**:
1. `src/unified_protocol/service.rs:1097` - Protocol handling (hot path)
2. `src/services/git_analysis.rs` - Git operations
3. `src/handlers/tools.rs` - Tool handling

### 12.3 String Buffer Pre-allocation (Est. 10-15% improvement)
Many report generation functions build strings incrementally without pre-allocation.

**Fix Priority**:
1. `src/services/deep_context.rs` - Context reports can be 10KB+
2. `src/services/enhanced_reporting.rs` - Full analysis reports
3. `src/handlers/tools.rs` - Response formatting

## Next Steps

1. **Immediate**: Fix regex compilation issue (1 hour effort, huge impact)
2. **This Week**: Pre-allocate collections in hot paths
3. **This Month**: Implement parallel processing for analysis
4. Run profiling to identify actual hot paths
5. Benchmark each optimization
6. Create regression tests
7. Document performance improvements

## Success Metrics

- [ ] Reduce clean build time by 50%
- [ ] Reduce incremental build time by 70%
- [ ] Reduce memory usage by 30%
- [ ] Improve analysis performance by 40%
- [ ] Maintain or improve code readability