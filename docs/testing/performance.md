# Performance Testing Guide

*Reference: SPECIFICATION.md Section 30*

## Overview

PMAT maintains strict performance standards through comprehensive benchmarking and load testing, ensuring sub-second response times for most operations.

## Performance Targets

From SPECIFICATION.md Section 1.4:

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Startup Latency | 127ms cold, 4ms hot | ~150ms cold | ⚠️ |
| Analysis Throughput | 487K LOC/s | ~400K LOC/s | ⚠️ |
| Memory Usage | 47MB base RSS | 45MB | ✅ |
| Cache Hit Rate | L1 97%, L2 89% | L1 95%, L2 85% | ⚠️ |

## Benchmark Suites

### 1. Micro-benchmarks

Located in `benches/`:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench complexity

# Generate flamegraph
cargo bench --bench analysis -- --profile-time=10
```

Key benchmarks:
- **Complexity analysis**: Per-function analysis speed
- **SATD detection**: Pattern matching performance
- **AST parsing**: Tree-sitter parser throughput
- **Cache operations**: Lookup and insertion speed

### 2. Load Testing

Using `examples/quality_gate_perf.rs`:

```bash
# Test with 1000 files
cargo run --release --example quality_gate_perf -- --files 1000

# Test with concurrent requests
cargo run --release --example load_test_mcp -- --concurrent 100
```

### 3. Memory Profiling

```bash
# Heap profiling with heaptrack
heaptrack ./target/release/pmat analyze complexity src/

# Memory usage over time
/usr/bin/time -v ./target/release/pmat quality-gate

# Valgrind for leak detection
valgrind --leak-check=full ./target/release/pmat
```

## Performance Optimization

### Current Optimizations

1. **Compile-time Tool Registry**: 130x faster MCP initialization
2. **SIMD Operations**: 2.7x speedup for pattern matching
3. **Lock-free Structures**: DashSet for concurrent access
4. **Lazy Evaluation**: On-demand AST parsing
5. **Memory Pooling**: Reuse allocations for AST nodes

### Profiling Tools

```bash
# CPU profiling with perf
perf record -g ./target/release/pmat analyze complexity
perf report

# Flamegraph generation
cargo flamegraph --bin pmat -- analyze complexity src/

# Criterion benchmarks with plots
cargo criterion
```

## Continuous Performance Monitoring

### CI Integration

Performance tests run in CI:
- **PR checks**: Quick benchmarks vs baseline
- **Nightly**: Full benchmark suite
- **Weekly**: Load tests with large codebases

### Regression Detection

```yaml
# .github/workflows/benchmark.yml
- name: Run benchmarks
  run: cargo bench --bench analysis -- --save-baseline pr-${{ github.event.number }}
  
- name: Compare with main
  run: cargo bench --bench analysis -- --baseline main
```

## Writing Performance Tests

### Benchmark Structure

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_complexity(c: &mut Criterion) {
    let code = include_str!("../fixtures/complex.rs");
    
    c.bench_function("analyze_complexity", |b| {
        b.iter(|| {
            let result = analyze_complexity(black_box(code));
            black_box(result);
        })
    });
}

criterion_group!(benches, benchmark_complexity);
criterion_main!(benches);
```

### Load Test Pattern

```rust
#[tokio::test]
async fn test_concurrent_load() {
    let handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                let start = Instant::now();
                analyze_file(&format!("file_{}.rs", i)).await;
                start.elapsed()
            })
        })
        .collect();
    
    let durations = futures::future::join_all(handles).await;
    let p99 = calculate_percentile(&durations, 99.0);
    assert!(p99 < Duration::from_millis(500));
}
```

## Performance Best Practices

1. **Measure First**: Profile before optimizing
2. **Set Targets**: Define acceptable performance
3. **Test Realistically**: Use production-like data
4. **Monitor Continuously**: Track trends over time
5. **Document Changes**: Note optimization rationale

## Related Documentation

- [Property-Based Testing](./property-based.md)
- [Integration Testing](./integration.md)
- [SPECIFICATION.md Section 30](../SPECIFICATION.md#30-performance-testing)
- [Memory Management](../SPECIFICATION.md#26-memory-management)