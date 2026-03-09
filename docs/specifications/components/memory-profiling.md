# Memory Profiling

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 19

## Overview

Heap allocation profiling is required for production Rust projects. This spec defines
the tooling, methodology, and compliance enforcement for memory profiling.

## dhat-rs Integration

### Setup

```rust
// In profiling binary (e.g., examples/dhat_profile.rs)
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::builder().testing().build();
    // ... run workload ...
    // dhat auto-saves profile on drop
}
```

### Cargo.toml

```toml
[dev-dependencies]
dhat = "0.3"

[[example]]
name = "dhat_profile"
path = "examples/dhat_profile.rs"
```

### Running Profiles

```bash
cargo run --example dhat_profile --release 2>&1 | tee dhat-output.txt
```

## Key Metrics

| Metric | Description | Target |
|--------|-------------|--------|
| Total allocations | Cumulative heap bytes allocated | Project-specific baseline |
| Peak memory | Maximum live bytes at any point | <500 MB for analysis tools |
| Allocation count | Number of malloc/free calls | Minimize (reuse buffers) |
| Allocation hotspots | Top functions by bytes allocated | Document top-10 |

## Optimization Patterns

### 1. Clone Elimination

Replace `data.clone()` with borrows or `Arc<T>`:
```rust
// Before: clones entire ProjectContext (~1 GB)
let ctx = (*shared_context).clone();

// After: borrow from Arc
let ctx: &ProjectContext = shared_context.as_ref();
```

### 2. In-Place Mutation

Replace clone-and-modify with ownership transfer:
```rust
// Before: clones graph, modifies copy
pub fn add_scores(graph: &DependencyGraph) -> DependencyGraph {
    let mut result = graph.clone();
    // modify result
    result
}

// After: takes ownership, mutates in-place
pub fn add_scores(mut graph: DependencyGraph) -> DependencyGraph {
    // modify graph directly
    graph
}
```

### 3. Process-Global Caching

Use DashMap for cross-task deduplication:
```rust
static CACHE: LazyLock<DashMap<PathBuf, Metrics>> = LazyLock::new(DashMap::new);

async fn analyze(path: &Path) -> Metrics {
    if let Some(cached) = CACHE.get(path) {
        return cached.clone();
    }
    let result = expensive_analysis(path).await;
    CACHE.insert(path.to_path_buf(), result.clone());
    result
}
```

### 4. Two-Phase Execution

Run dependent phases first to populate caches:
```
Phase 1 (sequential): AST analysis -> populates DashMap caches
Phase 2 (parallel):   Complexity, Provability, DAG, etc. -> read from caches
```

### 5. Test File Exclusion

Skip test files from analysis phases to reduce parsing:
```rust
fn is_test_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with("_tests.rs") || name.ends_with("_test.rs")
        || name.starts_with("test_")
        || path.components().any(|c| c.as_os_str() == "tests")
}
```

## Memory Regression Gates

### CI Integration

```yaml
# In CI pipeline
- name: Memory regression check
  run: |
    cargo run --example dhat_profile --release 2>&1 | tee dhat.txt
    peak=$(grep "At t-gmax" dhat.txt | grep -o '[0-9,]*' | head -1 | tr -d ',')
    if [ "$peak" -gt "$MEMORY_THRESHOLD" ]; then
      echo "FAIL: Peak memory ${peak} exceeds threshold ${MEMORY_THRESHOLD}"
      exit 1
    fi
```

### Baseline Tracking

Store baselines in `.pmat-metrics/memory-baseline.json`:
```json
{
  "total_allocations_bytes": 2680000000,
  "peak_memory_bytes": 145000000,
  "allocation_count": 4200000,
  "measured_at": "2026-03-09T00:00:00Z",
  "commit": "249fdd88"
}
```

## Comply Check: CB-141

### Detection

CB-141 checks for memory profiling infrastructure:

1. **dhat dependency**: `Cargo.toml` contains `dhat` in `[dev-dependencies]`
2. **Profile binary**: `examples/dhat_profile.rs` or similar exists
3. **Baseline file**: `.pmat-metrics/memory-baseline.json` exists

### Scoring

| Condition | Score Impact |
|-----------|-------------|
| No dhat dependency | -5 points |
| No profile binary | -3 points |
| No baseline file | -2 points |
| All present | 0 (no penalty) |

### Configuration

```yaml
# .pmat.yaml
comply:
  checks:
    cb-141:
      enabled: true
      severity: warning
      options:
        profiler: "dhat"  # or "jemalloc", "heaptrack"
        baseline_path: ".pmat-metrics/memory-baseline.json"
```

## Case Study: PMAT Deep Context (PR #264)

### Before Optimization

| Metric | Value |
|--------|-------|
| Total allocations | 8.7 GB |
| Peak memory | 194 MB |
| Runtime | 332s |
| Top hotspot | syn parsing (81%) |

### After Optimization (6 fixes)

| Metric | Value | Reduction |
|--------|-------|-----------|
| Total allocations | 2.68 GB | -69% |
| Peak memory | 145 MB | -25% |
| Runtime | 175s | -47% |

### Optimizations Applied

1. DashMap caches for complexity metrics (-1.2 GB)
2. Two-phase execution model (-0.8 GB)
3. Arc<ProjectContext> reuse (-2.0 GB)
4. Clone elimination in result integration (-0.5 GB)
5. In-place PageRank scoring (-49 MB peak)
6. Test file exclusion (-0.5 GB)

## References

- dhat-rs: https://docs.rs/dhat
- PMAT PR #264: Memory profiling and optimization
