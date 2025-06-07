# Complexity Analysis Specification

## Executive Summary

This specification defines a zero-overhead complexity analysis system for the PAIML MCP Agent Toolkit that leverages existing AST infrastructure to detect code smells without increasing binary size beyond 2%. The system achieves sub-millisecond analysis per file through strategic caching and visitor pattern reuse.

## Design Constraints

### Binary Size Budget
- **Target**: < 2% increase (~300KB on 15MB baseline)
- **Strategy**: Reuse existing AST visitors, no new parser dependencies
- **Measurement**: Track via `cargo bloat --release --crates`

### Performance Requirements
- **File analysis**: < 1ms per KLOC
- **Cache hit latency**: < 100μs
- **Memory overhead**: < 50MB for 10K file project

## Architecture

### Core Components

```rust
// Zero-allocation complexity visitor leveraging existing AST
pub struct ComplexityVisitor<'a> {
    complexity: &'a mut u32,
    nesting_level: u8,
}

// Reuse existing FileContext, add complexity field
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContext {
    // ... existing fields ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity_metrics: Option<ComplexityMetrics>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ComplexityMetrics {
    pub cyclomatic: u16,    // McCabe complexity
    pub cognitive: u16,     // Cognitive complexity (Sonar)
    pub nesting_max: u8,    // Max nesting depth
    pub lines: u16,         // Logical lines of code
}
```

### Integration with Existing Visitors

Extend current AST visitors without new allocations:

```rust
// In ast_typescript.rs
impl Visit for TypeScriptVisitor {
    fn visit_if_stmt(&mut self, node: &IfStmt) {
        // Existing logic...
        
        // Piggyback complexity calculation
        if let Some(ref mut metrics) = self.complexity_metrics {
            metrics.cyclomatic += 1;
            metrics.cognitive += 1 + self.nesting_level as u16;
        }
        
        self.nesting_level += 1;
        node.visit_children_with(self);
        self.nesting_level -= 1;
    }
}
```

## Complexity Calculation Algorithms

### Cyclomatic Complexity (McCabe)

Traditional graph-theoretic approach: `M = E - N + 2P`

Incremental calculation during AST traversal:
- +1 for each: `if`, `for`, `while`, `case`, `catch`
- +1 for each: `&&`, `||` in boolean expressions
- +1 for ternary operators

### Cognitive Complexity (Sonar)

Accounts for human comprehension difficulty:
- +1 for control flow breaks
- +nesting_level for nested structures
- +1 for each level of nesting beyond 1

```rust
fn calculate_cognitive_increment(node_type: &NodeType, nesting: u8) -> u16 {
    match node_type {
        NodeType::If => 1 + nesting.saturating_sub(1) as u16,
        NodeType::Loop => 1 + nesting as u16,
        NodeType::Catch => 1,
        NodeType::LogicalOp if nesting > 0 => 1,
        _ => 0,
    }
}
```

## Cache Strategy

### Three-Tier Caching

1. **L1: In-Memory LRU** (via existing ContentCache)
    - Key: `complexity:{file_hash}:{git_head}`
    - TTL: Until file modification
    - Size: 10K entries

2. **L2: Memoized AST Pass**
    - Piggyback on existing AST analysis
    - Zero additional I/O cost
    - Complexity calculated during initial parse

3. **L3: Git Object Store** (optional)
    - Store in `.git/paiml-cache/complexity/`
    - Key: blob SHA-1
    - Persistent across branches

### Cache Key Design

```rust
fn compute_cache_key(path: &Path, content: &[u8]) -> String {
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(content);
    hasher.write(path.as_os_str().as_encoded_bytes());
    format!("cx:{:x}", hasher.finish())
}
```

## Rule Engine

### Threshold Configuration

```toml
# Embedded in binary via include_str!
[complexity.thresholds]
cyclomatic_warn = 10
cyclomatic_error = 20
cognitive_warn = 15
cognitive_error = 30
nesting_max = 5
method_length = 50
```

### Rule Implementation

```rust
pub trait ComplexityRule: Send + Sync {
    fn evaluate(&self, metrics: &ComplexityMetrics, ctx: &AstItem) -> Option<Violation>;
    
    // Inline threshold checking for performance
    #[inline(always)]
    fn exceeds_threshold(&self, value: u16, threshold: u16) -> bool {
        value > threshold
    }
}

// Example: Zero-allocation rule
pub struct CyclomaticComplexityRule {
    warn_threshold: u16,
    error_threshold: u16,
}

impl ComplexityRule for CyclomaticComplexityRule {
    fn evaluate(&self, metrics: &ComplexityMetrics, ctx: &AstItem) -> Option<Violation> {
        if self.exceeds_threshold(metrics.cyclomatic, self.error_threshold) {
            Some(Violation::Error { 
                rule: "cyclomatic-complexity",
                value: metrics.cyclomatic,
                location: ctx.line_number(),
            })
        } else if self.exceeds_threshold(metrics.cyclomatic, self.warn_threshold) {
            Some(Violation::Warning { /* ... */ })
        } else {
            None
        }
    }
}
```

## Performance Optimizations

### 1. Streaming Analysis

Process files in chunks to maintain constant memory usage:

```rust
pub async fn analyze_project_complexity(path: &Path) -> Result<ComplexityReport> {
    let (tx, rx) = mpsc::channel(100);
    
    // Producer: walk files
    let producer = async {
        let walker = WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !is_hidden(e));
            
        for entry in walker {
            if let Ok(entry) = entry {
                if is_source_file(&entry) {
                    tx.send(entry.path().to_owned()).await?;
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    };
    
    // Consumer: analyze with bounded parallelism
    let consumer = async {
        let semaphore = Arc::new(Semaphore::new(num_cpus::get()));
        let cache = Arc::new(SessionCacheManager::new());
        
        rx.map(|path| {
            let cache = cache.clone();
            let semaphore = semaphore.clone();
            
            async move {
                let _permit = semaphore.acquire().await?;
                cache.get_or_compute_complexity(&path, || {
                    Box::pin(analyze_file_complexity(&path))
                }).await
            }
        })
        .buffer_unordered(100)
        .try_collect().await
    };
    
    let (_, results) = tokio::try_join!(producer, consumer)?;
    Ok(aggregate_results(results))
}
```

### 2. SIMD Aggregation

For large codebases, use SIMD for metric aggregation:

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

unsafe fn sum_complexities_simd(metrics: &[ComplexityMetrics]) -> u32 {
    let mut sum = _mm256_setzero_si256();
    
    for chunk in metrics.chunks_exact(16) {
        let values = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
        sum = _mm256_add_epi16(sum, values);
    }
    
    // Horizontal sum
    let sum128 = _mm_add_epi16(
        _mm256_extracti128_si256(sum, 0),
        _mm256_extracti128_si256(sum, 1)
    );
    // ... continue reduction
}
```

### 3. Incremental Analysis

Track file modifications via inotify/FSEvents:

```rust
pub struct IncrementalAnalyzer {
    file_states: DashMap<PathBuf, FileState>,
    watcher: RecommendedWatcher,
}

struct FileState {
    last_modified: SystemTime,
    content_hash: u64,
    metrics: ComplexityMetrics,
}
```

## CLI Integration

```bash
# Analyze current directory
paiml-mcp-agent-toolkit analyze complexity

# With custom thresholds
paiml-mcp-agent-toolkit analyze complexity --max-cyclomatic 15 --max-cognitive 20

# Output formats
paiml-mcp-agent-toolkit analyze complexity --format json
paiml-mcp-agent-toolkit analyze complexity --format sarif  # For IDE integration

# Watch mode for continuous analysis
paiml-mcp-agent-toolkit analyze complexity --watch
```

## MCP Tool Integration

```json
{
  "name": "analyze_complexity",
  "description": "Analyze code complexity and detect maintainability issues",
  "inputSchema": {
    "type": "object",
    "properties": {
      "project_path": { "type": "string" },
      "include_patterns": { "type": "array", "items": { "type": "string" } },
      "thresholds": {
        "type": "object",
        "properties": {
          "cyclomatic": { "type": "integer" },
          "cognitive": { "type": "integer" }
        }
      }
    }
  }
}
```

## Output Schema

```rust
#[derive(Serialize, Deserialize)]
pub struct ComplexityReport {
    pub summary: ComplexitySummary,
    pub violations: Vec<Violation>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub metrics_by_file: HashMap<PathBuf, FileComplexityMetrics>,
}

#[derive(Serialize, Deserialize)]
pub struct ComplexitySummary {
    pub total_files: usize,
    pub total_functions: usize,
    pub avg_cyclomatic: f32,
    pub avg_cognitive: f32,
    pub p90_cyclomatic: u16,  // 90th percentile
    pub p90_cognitive: u16,
    pub technical_debt_hours: f32,  // Estimated refactoring time
}
```

## Benchmarks

Target performance on reference codebases:

| Project | Files | SLOC | Analysis Time | Memory | Cache Hit Rate |
|---------|-------|------|---------------|---------|----------------|
| tokio | 450 | 95K | 47ms | 12MB | 98% |
| rustc | 5.2K | 680K | 580ms | 78MB | 95% |
| chromium | 35K | 9.8M | 4.2s | 450MB | 92% |

## Implementation Phases

### Phase 1: Core Metrics (1 week)
- Extend existing AST visitors
- Implement McCabe complexity
- Basic CLI output

### Phase 2: Caching & Performance (1 week)
- Integrate with SessionCacheManager
- Implement incremental analysis
- Add SIMD optimizations

### Phase 3: Advanced Rules (1 week)
- Cognitive complexity
- Method length analysis
- Cross-file coupling metrics

### Phase 4: Integration (3 days)
- MCP tool wrapper
- SARIF output for IDEs
- Watch mode

## Testing Strategy

```rust
#[test]
fn test_complexity_calculation() {
    let code = r#"
        fn example(x: i32) -> i32 {
            if x > 0 {                    // +1
                for i in 0..x {           // +1
                    if i % 2 == 0 {       // +1
                        return i;
                    }
                }
            } else if x < 0 {             // +1
                return -x;
            }
            0
        }
    "#;
    
    let metrics = analyze_rust_snippet(code).unwrap();
    assert_eq!(metrics.cyclomatic, 5);
    assert_eq!(metrics.cognitive, 7); // includes nesting penalty
}
```

## Security Considerations

- No arbitrary code execution during analysis
- Bounded memory usage via streaming
- Safe handling of malformed source files
- No network access required

## Future Extensions

1. **Language Server Protocol**: Real-time complexity hints
2. **Git Hook Integration**: Pre-commit complexity gates
3. **Trend Analysis**: Track complexity over time
4. **Refactoring Suggestions**: ML-based improvement hints