```markdown
# QuickCheck Integration for Invariant Verification

## Abstract

This specification defines the integration of property-based testing via QuickCheck into the PAIML MCP Agent Toolkit, targeting critical invariants in the symbolic reasoning layer. The focus is on verifying determinism guarantees, cache coherence, and protocol equivalence—properties that are difficult to validate through example-based testing but essential for hybrid AI correctness.

## Motivation

The toolkit's value proposition rests on **deterministic symbolic analysis** augmenting probabilistic LLMs. A single non-deterministic bug undermines the entire architecture. Traditional unit tests verify specific cases; QuickCheck verifies algebraic properties across the entire input space, with particular strength in finding edge cases in concurrent systems and cache invalidation logic.

## Architecture

### 1. Dependency Integration

```toml
# server/Cargo.toml
[dev-dependencies]
quickcheck = "1.0"
quickcheck_macros = "1.0"
proptest = "1.4"  # Superior shrinking for complex structures

[profile.test]
opt-level = 3  # Property tests need performance
```

### 2. Custom Arbitrary Implementations

```rust
// server/src/testing/arbitrary.rs
use quickcheck::{Arbitrary, Gen};
use crate::models::unified_ast::*;

impl Arbitrary for AstNode {
    fn arbitrary(g: &mut Gen) -> Self {
        // Bounded recursion to prevent stack overflow
        let depth = g.size() / 10;
        if depth == 0 {
            return AstNode::Leaf(String::arbitrary(g));
        }
        
        g.set_size(depth - 1);
        match g.gen_range(0..7) {
            0 => AstNode::Function {
                name: arbitrary_identifier(g),
                params: arbitrary_bounded_vec(g, 0, 10),
                body: Box::new(AstNode::arbitrary(g)),
                complexity: g.gen_range(1..50),
            },
            1 => AstNode::Class {
                name: arbitrary_identifier(g),
                fields: arbitrary_bounded_vec(g, 0, 20),
                methods: arbitrary_bounded_vec(g, 0, 30),
            },
            2 => AstNode::Import {
                module: arbitrary_module_path(g),
                items: arbitrary_import_list(g),
            },
            // ... remaining variants
        }
    }
    
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            AstNode::Function { body, .. } => {
                Box::new(std::iter::once((**body).clone()))
            }
            AstNode::Class { methods, .. } => {
                Box::new(methods.iter().cloned())
            }
            _ => Box::new(std::iter::empty()),
        }
    }
}

// Domain-specific generators
fn arbitrary_identifier(g: &mut Gen) -> String {
    let prefixes = ["_", "test_", "impl_", "__"];
    let base = ['a'..='z', 'A'..='Z']
        .iter()
        .flat_map(|r| *r)
        .choose(g)
        .unwrap();
    
    if g.gen_bool(0.1) {  // 10% chance of edge case
        format!("{}{}{}", 
            prefixes.choose(g).unwrap(),
            base,
            g.gen_range(0..1000))
    } else {
        format!("{}{}", base, g.gen_range(0..100))
    }
}
```

### 3. Core Invariant Properties

```rust
// server/src/testing/properties/determinism.rs
use quickcheck_macros::quickcheck;

#[quickcheck]
fn prop_ast_parsing_deterministic(content: Vec<u8>) -> TestResult {
    // Filter out invalid UTF-8
    let Ok(code) = String::from_utf8(content) else {
        return TestResult::discard();
    };
    
    // Parse twice
    let ast1 = parse_rust_content(&code);
    let ast2 = parse_rust_content(&code);
    
    match (ast1, ast2) {
        (Ok(a1), Ok(a2)) => TestResult::from_bool(a1 == a2),
        (Err(e1), Err(e2)) => TestResult::from_bool(e1.to_string() == e2.to_string()),
        _ => TestResult::failed(),
    }
}

#[quickcheck]
fn prop_complexity_metrics_deterministic(ast: AstNode) -> bool {
    let metrics1 = compute_complexity(&ast);
    let metrics2 = compute_complexity(&ast);
    
    metrics1.cyclomatic == metrics2.cyclomatic &&
    metrics1.cognitive == metrics2.cognitive &&
    (metrics1.halstead_effort - metrics2.halstead_effort).abs() < f64::EPSILON
}

#[quickcheck]
fn prop_mermaid_generation_stable(dag: DependencyGraph) -> bool {
    // Critical: Mermaid output must be byte-identical for CI/CD
    let mermaid1 = generate_mermaid_deterministic(&dag);
    let mermaid2 = generate_mermaid_deterministic(&dag);
    
    mermaid1 == mermaid2
}
```

### 4. Cache Coherence Properties

```rust
// server/src/testing/properties/cache.rs

#[quickcheck]
fn prop_content_addressed_cache_coherent(
    files: Vec<(PathBuf, String)>,
    ops: Vec<CacheOp>
) -> bool {
    let cache = ContentAddressedCache::new();
    let mut shadow = HashMap::new();  // Reference implementation
    
    for op in ops {
        match op {
            CacheOp::Insert(idx) => {
                if let Some((path, content)) = files.get(idx % files.len()) {
                    let hash = blake3::hash(content.as_bytes());
                    cache.insert(hash, parse_file(path, content));
                    shadow.insert(hash, content.clone());
                }
            }
            CacheOp::Get(idx) => {
                if let Some((_, content)) = files.get(idx % files.len()) {
                    let hash = blake3::hash(content.as_bytes());
                    let cached = cache.get(&hash);
                    let expected = shadow.get(&hash).map(|c| parse_file("", c));
                    
                    match (cached, expected) {
                        (Some(c), Some(e)) => {
                            if c != e { return false; }
                        }
                        (None, None) => {}
                        _ => return false,
                    }
                }
            }
            CacheOp::Evict(size) => {
                cache.evict_to_size(size);
                // Shadow doesn't model eviction, so we skip verification
            }
        }
    }
    
    true
}

#[derive(Clone, Debug, Arbitrary)]
enum CacheOp {
    Insert(usize),
    Get(usize),
    Evict(usize),
}
```

### 5. Protocol Equivalence Properties

```rust
// server/src/testing/properties/protocol.rs

#[quickcheck]
fn prop_protocol_equivalence(request: AnalysisRequest) -> TestResult {
    // All three protocols must produce identical analysis results
    
    // Skip if request contains paths that don't exist
    if !request.project_path.exists() {
        return TestResult::discard();
    }
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let cli_result = rt.block_on(async {
        execute_cli_analysis(&request).await
    });
    
    let http_result = rt.block_on(async {
        let server = start_http_server(0).await;
        let result = http_client_request(&server.addr, &request).await;
        server.shutdown().await;
        result
    });
    
    let mcp_result = rt.block_on(async {
        execute_mcp_analysis(&request).await
    });
    
    // Normalize results (remove timestamps, sort arrays)
    let cli_norm = normalize_analysis_result(cli_result?);
    let http_norm = normalize_analysis_result(http_result?);
    let mcp_norm = normalize_analysis_result(mcp_result?);
    
    TestResult::from_bool(
        cli_norm == http_norm && 
        http_norm == mcp_norm
    )
}
```

### 6. Concurrency Properties

```rust
// server/src/testing/properties/concurrency.rs

#[quickcheck]
fn prop_parallel_analysis_deterministic(
    files: Vec<FileContent>,
    thread_count: u8
) -> bool {
    let thread_count = (thread_count % 16) + 1;  // 1-16 threads
    
    // Sequential baseline
    let sequential_results: Vec<_> = files.iter()
        .map(|f| analyze_file(f))
        .collect();
    
    // Parallel execution
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count as usize)
        .build()
        .unwrap();
    
    let parallel_results: Vec<_> = pool.install(|| {
        files.par_iter()
            .map(|f| analyze_file(f))
            .collect()
    });
    
    sequential_results == parallel_results
}

#[quickcheck]
fn prop_cache_concurrent_access_safe(
    ops: Vec<(ThreadId, CacheOp)>
) -> bool {
    let cache = Arc::new(SessionCacheManager::new());
    let barrier = Arc::new(Barrier::new(ops.len()));
    
    let handles: Vec<_> = ops.into_iter()
        .map(|(tid, op)| {
            let cache = cache.clone();
            let barrier = barrier.clone();
            
            std::thread::spawn(move || {
                barrier.wait();  // Thundering herd
                
                match op {
                    CacheOp::Insert(key, value) => {
                        cache.insert(key, value);
                    }
                    CacheOp::Get(key) => {
                        let _ = cache.get(&key);
                    }
                }
            })
        })
        .collect();
    
    // If any thread panics, the test fails
    handles.into_iter().all(|h| h.join().is_ok())
}
```

### 7. Regression Properties

```rust
// server/src/testing/properties/regression.rs

// Encode known bugs as properties to prevent regression
#[quickcheck]
fn prop_no_empty_mermaid_nodes(dag: DependencyGraph) -> bool {
    let mermaid = generate_mermaid_deterministic(&dag);
    
    // Bug #47: Empty labels caused syntax errors
    !mermaid.contains("[]") && 
    !mermaid.contains("[``]") &&
    !mermaid.contains("[\"\"]")
}

#[quickcheck]
fn prop_unicode_handling_consistent(
    content: String,
    insertions: Vec<(usize, String)>
) -> bool {
    let mut rope = ropey::Rope::from_str(&content);
    
    for (offset, text) in insertions {
        let char_idx = offset % (rope.len_chars() + 1);
        
        // Must handle grapheme clusters correctly
        if let Ok(graphemes) = unicode_segmentation::UnicodeSegmentation::graphemes(&text[..], true) {
            rope.insert(char_idx, &text);
            
            // Verify round-trip
            let extracted = rope.slice(char_idx..char_idx + text.len()).to_string();
            if extracted != text {
                return false;
            }
        }
    }
    
    true
}
```

### 8. Performance Properties

```rust
// server/src/testing/properties/performance.rs

#[quickcheck]
fn prop_complexity_analysis_linear_time(sizes: Vec<u16>) -> TestResult {
    let sizes: Vec<_> = sizes.into_iter()
        .map(|s| (s as usize % 1000) + 1)
        .take(10)
        .collect();
    
    if sizes.len() < 2 {
        return TestResult::discard();
    }
    
    let mut timings = Vec::new();
    
    for &size in &sizes {
        let ast = generate_balanced_ast(size);
        
        let start = std::time::Instant::now();
        let _ = compute_complexity(&ast);
        let elapsed = start.elapsed();
        
        timings.push((size, elapsed));
    }
    
    // Verify O(n) complexity via linear regression
    let slope = calculate_regression_slope(&timings);
    let r_squared = calculate_r_squared(&timings);
    
    // Must be linear with high correlation
    TestResult::from_bool(r_squared > 0.95)
}
```

### 9. Integration with CI

```yaml
# .github/workflows/property-tests.yml
name: Property Tests

on: [push, pull_request]

jobs:
  quickcheck:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run QuickCheck suite
        run: |
          # Increase test cases for CI
          export QUICKCHECK_TESTS=10000
          export QUICKCHECK_MAX_TESTS=100000
          cargo test --release properties:: -- --nocapture
      
      - name: Run minimization on failures
        if: failure()
        run: |
          # Re-run with shrinking enabled
          export QUICKCHECK_SHRINK=1
          cargo test --release properties:: -- --nocapture --test-threads=1
```

### 10. Measurement Framework

```rust
// server/src/testing/properties/metrics.rs

static PROPERTY_METRICS: Lazy<PropertyMetrics> = Lazy::new(|| {
    PropertyMetrics::new()
});

struct PropertyMetrics {
    executions: AtomicU64,
    failures: AtomicU64,
    discards: AtomicU64,
    shrink_steps: AtomicU64,
}

impl PropertyMetrics {
    pub fn report(&self) {
        let total = self.executions.load(Ordering::Relaxed);
        let failures = self.failures.load(Ordering::Relaxed);
        let discards = self.discards.load(Ordering::Relaxed);
        
        eprintln!("Property Test Report:");
        eprintln!("  Total executions: {}", total);
        eprintln!("  Failures: {} ({:.2}%)", failures, 
            (failures as f64 / total as f64) * 100.0);
        eprintln!("  Discard rate: {:.2}%", 
            (discards as f64 / total as f64) * 100.0);
        
        if failures > 0 {
            eprintln!("  Avg shrink steps: {:.1}", 
                self.shrink_steps.load(Ordering::Relaxed) as f64 / failures as f64);
        }
    }
}
```

## Expected Defect Detection

Based on similar integrations, QuickCheck typically uncovers:

1. **Off-by-one errors** in boundary conditions (empty collections, single elements)
2. **Unicode mishandling** in string processing (grapheme clusters vs code points)
3. **Race conditions** in concurrent cache access
4. **Floating-point comparison bugs** in metrics calculation
5. **Integer overflow** in complexity scoring with pathological ASTs
6. **Protocol divergence** under specific error conditions

## Success Metrics

- **Bug discovery rate**: >5 critical bugs in first month
- **Regression prevention**: 100% of fixed bugs encoded as properties
- **Test execution time**: <5 minutes for 10,000 cases per property
- **Shrinking effectiveness**: Average minimal case <10% of original size
- **Coverage augmentation**: 15% increase in mutation testing score
```