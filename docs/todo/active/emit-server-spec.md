# Hybrid Incremental Cache-Aware Emit Server Specification

**Version**: 1.0.0  
**Status**: Draft  
**Authors**: PAIML Engineering Team  
**Last Updated**: 2025-06-06

## Executive Summary

The Hybrid Incremental Cache-Aware Emit Server delivers sub-5ms defect metric emissions for real-time code quality feedback. By leveraging the existing PAIML MCP Agent Toolkit infrastructure, this server provides zero-copy, cache-coherent analysis deltas to IDE clients via lock-free ring buffers.

### Key Performance Targets
- **Edit-to-emit latency**: < 5ms (P99)
- **Memory overhead**: < 64MB resident set size
- **CPU overhead**: < 2% single core utilization
- **Throughput**: 10,000 edits/second sustained

## 1. Architecture Overview

### 1.1 Component Topology

```rust
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Text Editor   │────▶│  Emit Server     │────▶│   LSP Client    │
│  (VS Code/Vim)  │     │  (Rust Process)  │     │  (Extension)    │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         │                       │                         │
         │                       ▼                         │
         │              ┌──────────────────┐              │
         └─────────────▶│  Shared Memory   │◀─────────────┘
                        │  (Ring Buffer)   │
                        └──────────────────┘
```

### 1.2 Core Data Flow

1. **Edit Event** → TextEdit struct with span and content
2. **AST Delta** → Incremental tree-sitter parse on affected span
3. **Metric Computation** → Vectorized TDG/complexity on changed nodes
4. **Threshold Gate** → Filter based on DeepContextConfig thresholds
5. **Ring Buffer Write** → Zero-copy append to memory-mapped queue
6. **LSP Emission** → Client translates payload to diagnostics

## 2. Component Specifications

### 2.1 DefectEmitServer Core

```rust
pub struct DefectEmitServer {
    // AST parsing engine with incremental capabilities
    ast_engine: Arc<UnifiedAstEngine>,
    
    // Multi-layer cache (L1: content hash, L2: persistent)
    cache: Arc<UnifiedCacheManager>,
    
    // Lock-free SPSC ring buffer (single producer, single consumer)
    emit_buffer: RingBuffer<DefectPayload, 1024>,
    
    // Concurrent file state tracking
    incremental_state: DashMap<PathBuf, FileState>,
    
    // Analyzer pool for language dispatch
    analyzers: AnalyzerPool,
    
    // Metrics collector for performance monitoring
    metrics: Arc<EmitServerMetrics>,
}

struct FileState {
    // Last parsed AST root
    ast_root: Arc<AstNode>,
    
    // Version counter for optimistic concurrency
    version: AtomicU64,
    
    // Cached complexity metrics
    complexity_cache: FxHashMap<NodeId, (u16, u16)>,
    
    // Dead code bitset (hierarchical)
    dead_symbols: HierarchicalBitSet,
    
    // Last emission timestamp
    last_emit: Instant,
}
```

**Implementation Checklist:**
- [ ] Define DefectEmitServer struct with all fields
- [ ] Implement FileState with atomic version counter
- [ ] Add constructor with dependency injection
- [ ] Implement Drop trait for clean shutdown
- [ ] Add metrics collection hooks

### 2.2 DefectPayload Structure

```rust
#[repr(C, align(64))]  // Cache-line aligned
#[derive(Copy, Clone, Debug)]
pub struct DefectPayload {
    // File identifier (xxHash3 of canonical path)
    file_hash: u64,
    
    // Technical Debt Gradient score [0.0, 3.0]
    tdg_score: f32,
    
    // Complexity metrics (cyclomatic, cognitive)
    complexity: (u16, u16),
    
    // Count of potentially dead symbols
    dead_symbols: u32,
    
    // Monotonic timestamp (nanos since process start)
    timestamp: u64,
    
    // Severity flags (bit 0: error, bit 1: warning)
    severity_flags: u8,
    
    // Reserved for future use (maintains alignment)
    _padding: [u8; 7],
}

// Compile-time size assertion
const _: () = assert!(std::mem::size_of::<DefectPayload>() == 64);
```

**Implementation Checklist:**
- [ ] Define DefectPayload with exact 64-byte layout
- [ ] Add compile-time size assertion
- [ ] Implement Debug trait for logging
- [ ] Add serialization for wire protocol
- [ ] Create builder pattern for construction

### 2.3 Ring Buffer Implementation

```rust
pub struct RingBuffer<T, const N: usize> {
    // Memory-mapped region (2 * N * size_of::<T>())
    mmap: MmapMut,
    
    // Write position (only incremented by producer)
    write_pos: AtomicUsize,
    
    // Read position (only incremented by consumer)
    read_pos: AtomicUsize,
    
    // Phantom data for type safety
    _phantom: PhantomData<T>,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub fn new(path: &Path) -> io::Result<Self> {
        // Create file with size 2 * N * size_of::<T>()
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        let size = 2 * N * std::mem::size_of::<T>();
        file.set_len(size as u64)?;
        
        // Memory map with double mapping for seamless wrap
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        Ok(Self {
            mmap,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            _phantom: PhantomData,
        })
    }
    
    pub fn push(&self, item: T) -> bool {
        let write = self.write_pos.load(Ordering::Acquire);
        let read = self.read_pos.load(Ordering::Relaxed);
        
        // Check if buffer is full
        if (write + 1) % N == read {
            return false;
        }
        
        // Write item at current position
        unsafe {
            let ptr = self.mmap.as_ptr() as *mut T;
            ptr.add(write).write(item);
        }
        
        // Update write position
        self.write_pos.store((write + 1) % N, Ordering::Release);
        true
    }
}
```

**Implementation Checklist:**
- [ ] Implement RingBuffer with memory mapping
- [ ] Add push method with overflow detection
- [ ] Add batch_push for multiple items
- [ ] Implement consumer-side pop method
- [ ] Add performance benchmarks
- [ ] Create unit tests for wraparound

### 2.4 Incremental AST Analysis

```rust
impl DefectEmitServer {
    pub async fn on_edit(&self, edit: TextEdit) -> Result<(), EmitError> {
        // 1. Retrieve cached AST
        let file_id = FileId::from_path(&edit.path);
        let mut state = self.incremental_state
            .get_mut(&edit.path)
            .ok_or(EmitError::FileNotTracked)?;
        
        // 2. Compute AST delta using tree-sitter
        let old_tree = &state.ast_root;
        let new_tree = self.ast_engine
            .parse_incremental(&edit.content, old_tree, &edit.changes)
            .await?;
        
        // 3. Find affected nodes
        let affected_nodes = self.compute_affected_nodes(
            &old_tree,
            &new_tree,
            &edit.changes
        );
        
        // 4. Recompute metrics for affected nodes only
        let mut metrics_changed = false;
        for node in affected_nodes {
            let old_complexity = state.complexity_cache.get(&node.id);
            let new_complexity = self.compute_node_complexity(&node);
            
            if old_complexity != Some(&new_complexity) {
                state.complexity_cache.insert(node.id, new_complexity);
                metrics_changed = true;
            }
        }
        
        // 5. Update state
        state.ast_root = Arc::new(new_tree);
        state.version.fetch_add(1, Ordering::SeqCst);
        
        // 6. Emit if metrics changed and threshold crossed
        if metrics_changed {
            self.maybe_emit_payload(&state, &edit.path).await?;
        }
        
        Ok(())
    }
    
    fn compute_affected_nodes(
        &self,
        old_tree: &AstNode,
        new_tree: &AstNode,
        changes: &[InputEdit]
    ) -> Vec<AstNode> {
        // Use tree-sitter's changed_ranges API
        let mut affected = Vec::new();
        
        for change in changes {
            // Find all nodes intersecting the change range
            let cursor = new_tree.walk();
            cursor.goto_byte(change.start_byte);
            
            while cursor.node().end_byte() <= change.new_end_byte {
                if cursor.node().has_changes() {
                    affected.push(cursor.node().clone());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        affected
    }
}
```

**Implementation Checklist:**
- [ ] Implement on_edit method with error handling
- [ ] Add compute_affected_nodes using tree-sitter
- [ ] Implement incremental complexity calculation
- [ ] Add dead code delta computation
- [ ] Create TDG score update logic
- [ ] Add comprehensive error types

## 3. Language-Specific Analyzers

### 3.1 Analyzer Pool Architecture

```rust
pub struct AnalyzerPool {
    rust: Arc<RustAnalyzer>,
    typescript: Arc<TypeScriptAnalyzer>,
    python: Arc<PythonAnalyzer>,
    c_cpp: Arc<CCppAnalyzer>,
    // ... 9 more languages
}

pub trait LanguageAnalyzer: Send + Sync {
    fn compute_complexity(&self, node: &AstNode) -> (u16, u16);
    fn detect_dead_code(&self, node: &AstNode, ctx: &AnalysisContext) -> BitVec;
    fn compute_tdg_components(&self, node: &AstNode) -> TdgComponents;
}
```

**Implementation Checklist:**
- [ ] Define LanguageAnalyzer trait
- [ ] Implement RustAnalyzer with syn integration
- [ ] Implement TypeScriptAnalyzer with swc
- [ ] Implement PythonAnalyzer with RustPython parser
- [ ] Add analyzer selection based on file extension
- [ ] Create analyzer benchmarks

### 3.2 Rust-Specific Implementation

```rust
impl LanguageAnalyzer for RustAnalyzer {
    fn compute_complexity(&self, node: &AstNode) -> (u16, u16) {
        let mut visitor = ComplexityVisitor::new();
        
        match node.kind() {
            "function_item" => {
                // Count branches for cyclomatic complexity
                let cyclomatic = 1 + node.children_by_field_name("body")
                    .flat_map(|b| b.descendants())
                    .filter(|n| matches!(n.kind(), 
                        "if_expression" | "match_expression" | 
                        "while_expression" | "for_expression" |
                        "loop_expression" | "return_expression" |
                        "break_expression" | "continue_expression"
                    ))
                    .count() as u16;
                
                // Cognitive complexity includes nesting depth
                let cognitive = self.compute_cognitive_complexity(node);
                
                (cyclomatic, cognitive)
            }
            _ => (0, 0),
        }
    }
}
```

**Implementation Checklist:**
- [ ] Implement cyclomatic complexity calculation
- [ ] Add cognitive complexity with nesting penalties
- [ ] Implement macro expansion handling
- [ ] Add async/await complexity adjustments
- [ ] Create test suite with known complexity values

## 4. Performance Optimizations

### 4.1 SIMD-Accelerated TDG Calculation

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn compute_tdg_vectorized(metrics: &[FileMetrics]) -> Vec<f32> {
    unsafe {
        let mut results = vec![0.0f32; metrics.len()];
        
        // Process 8 metrics at a time using AVX2
        let chunks = metrics.chunks_exact(8);
        let remainder = chunks.remainder();
        
        for (i, chunk) in chunks.enumerate() {
            // Load complexity values into SIMD registers
            let complexity = _mm256_set_ps(
                chunk[7].complexity as f32,
                chunk[6].complexity as f32,
                chunk[5].complexity as f32,
                chunk[4].complexity as f32,
                chunk[3].complexity as f32,
                chunk[2].complexity as f32,
                chunk[1].complexity as f32,
                chunk[0].complexity as f32,
            );
            
            // Load churn values
            let churn = _mm256_set_ps(
                chunk[7].churn_rate,
                chunk[6].churn_rate,
                chunk[5].churn_rate,
                chunk[4].churn_rate,
                chunk[3].churn_rate,
                chunk[2].churn_rate,
                chunk[1].churn_rate,
                chunk[0].churn_rate,
            );
            
            // TDG = 0.4 * complexity + 0.3 * churn + ...
            let weight_complexity = _mm256_set1_ps(0.4);
            let weight_churn = _mm256_set1_ps(0.3);
            
            let tdg = _mm256_fmadd_ps(complexity, weight_complexity,
                      _mm256_mul_ps(churn, weight_churn));
            
            // Store results
            _mm256_storeu_ps(&mut results[i * 8], tdg);
        }
        
        // Handle remainder with scalar code
        for (i, metric) in remainder.iter().enumerate() {
            results[chunks.len() * 8 + i] = 
                0.4 * metric.complexity as f32 + 
                0.3 * metric.churn_rate;
        }
        
        results
    }
}
```

**Implementation Checklist:**
- [ ] Implement SIMD TDG calculation for x86_64
- [ ] Add ARM NEON implementation
- [ ] Create fallback scalar implementation
- [ ] Add runtime CPU feature detection
- [ ] Benchmark against scalar version

### 4.2 Cache Warming Strategy

```rust
impl DefectEmitServer {
    pub async fn warm_cache(&self, workspace: &Path) -> Result<(), EmitError> {
        // Discover all source files
        let files = self.discover_files(workspace).await?;
        
        // Parallel cache warming with bounded concurrency
        let semaphore = Arc::new(Semaphore::new(num_cpus::get()));
        let mut tasks = Vec::new();
        
        for file in files {
            let sem = semaphore.clone();
            let engine = self.ast_engine.clone();
            let cache = self.cache.clone();
            
            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await?;
                
                // Parse and cache AST
                let content = tokio::fs::read_to_string(&file).await?;
                let ast = engine.parse_file(&file, &content).await?;
                
                // Compute and cache metrics
                let metrics = engine.analyze_ast(&ast).await?;
                cache.put_analysis(&file, metrics).await?;
                
                Ok::<(), EmitError>(())
            });
            
            tasks.push(task);
        }
        
        // Wait for all warmup tasks
        futures::future::try_join_all(tasks).await?;
        
        Ok(())
    }
}
```

**Implementation Checklist:**
- [ ] Implement parallel cache warming
- [ ] Add progress reporting
- [ ] Implement incremental warming
- [ ] Add cache size limits
- [ ] Create warmup benchmarks

## 5. Integration Points

### 5.1 MCP Tool Registration

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EmitServerTool {
    name: String,
    description: String,
    input_schema: Value,
}

impl EmitServerTool {
    pub fn new() -> Self {
        Self {
            name: "emit_server".to_string(),
            description: "Real-time code defect emission server".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "enum": ["start", "stop", "status"]
                    },
                    "workspace": {
                        "type": "string",
                        "description": "Workspace path to monitor"
                    },
                    "config": {
                        "type": "object",
                        "properties": {
                            "thresholds": {
                                "type": "object"
                            }
                        }
                    }
                },
                "required": ["command"]
            }),
        }
    }
}
```

**Implementation Checklist:**
- [ ] Implement MCP tool interface
- [ ] Add tool discovery endpoint
- [ ] Implement start/stop commands
- [ ] Add configuration validation
- [ ] Create integration tests

### 5.2 LSP Client Protocol

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EmitServerDiagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub source: String,
    pub message: String,
    pub data: DefectPayload,
}

impl From<DefectPayload> for EmitServerDiagnostic {
    fn from(payload: DefectPayload) -> Self {
        Self {
            range: Range::default(), // Set by client
            severity: if payload.severity_flags & 1 != 0 {
                DiagnosticSeverity::Error
            } else {
                DiagnosticSeverity::Warning
            },
            code: format!("TDG{:.2}", payload.tdg_score),
            source: "paiml-emit".to_string(),
            message: format!(
                "Technical debt: {:.2}, Complexity: {:?}, Dead symbols: {}",
                payload.tdg_score,
                payload.complexity,
                payload.dead_symbols
            ),
            data: payload,
        }
    }
}
```

**Implementation Checklist:**
- [ ] Define LSP diagnostic format
- [ ] Implement payload to diagnostic conversion
- [ ] Add range computation from AST nodes
- [ ] Create diagnostic batching logic
- [ ] Add client notification protocol

## 6. Error Handling & Recovery

### 6.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("File not tracked: {0}")]
    FileNotTracked(PathBuf),
    
    #[error("Parse error: {0}")]
    ParseError(#[from] tree_sitter::Error),
    
    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),
    
    #[error("Ring buffer full")]
    BufferFull,
    
    #[error("Analysis timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),
}
```

**Implementation Checklist:**
- [ ] Define comprehensive error types
- [ ] Implement error recovery strategies
- [ ] Add error metrics collection
- [ ] Create error notification system
- [ ] Add circuit breaker for cascading failures

### 6.2 Graceful Degradation

```rust
impl DefectEmitServer {
    async fn with_timeout<T>(
        &self,
        future: impl Future<Output = T>,
        timeout: Duration,
    ) -> Result<T, EmitError> {
        match tokio::time::timeout(timeout, future).await {
            Ok(result) => Ok(result),
            Err(_) => {
                self.metrics.timeouts.fetch_add(1, Ordering::Relaxed);
                Err(EmitError::Timeout(timeout))
            }
        }
    }
    
    async fn maybe_emit_payload(
        &self,
        state: &FileState,
        path: &Path,
    ) -> Result<(), EmitError> {
        // Compute payload with timeout
        let payload = self.with_timeout(
            self.compute_payload(state, path),
            Duration::from_millis(5),
        ).await?;
        
        // Try to emit, but don't block
        if !self.emit_buffer.push(payload) {
            self.metrics.dropped_events.fetch_add(1, Ordering::Relaxed);
            // Log but don't fail
            tracing::warn!("Emit buffer full, dropping event");
        }
        
        Ok(())
    }
}
```

**Implementation Checklist:**
- [ ] Implement timeout wrapper
- [ ] Add retry logic with backoff
- [ ] Create circuit breaker
- [ ] Add fallback to batched emission
- [ ] Implement metric degradation alerts

## 7. Testing Strategy

### 7.1 Unit Test Suite

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_payload_size() {
        assert_eq!(std::mem::size_of::<DefectPayload>(), 64);
        assert_eq!(std::mem::align_of::<DefectPayload>(), 64);
    }
    
    #[test]
    fn test_ring_buffer_wraparound() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ring.buf");
        let mut ring = RingBuffer::<u64, 4>::new(&path).unwrap();
        
        // Fill buffer
        assert!(ring.push(1));
        assert!(ring.push(2));
        assert!(ring.push(3));
        assert!(!ring.push(4)); // Should fail - buffer full
        
        // Consumer reads one
        assert_eq!(ring.pop(), Some(1));
        
        // Now we can push again
        assert!(ring.push(4));
    }
    
    proptest! {
        #[test]
        fn test_tdg_calculation_bounds(
            complexity in 0u16..100,
            churn in 0.0f32..10.0,
            dead_ratio in 0.0f32..1.0,
        ) {
            let tdg = calculate_tdg(complexity, churn, dead_ratio);
            prop_assert!(tdg >= 0.0 && tdg <= 3.0);
        }
    }
}
```

**Implementation Checklist:**
- [ ] Create unit tests for each component
- [ ] Add property-based tests for invariants
- [ ] Implement fuzzing for parser edge cases
- [ ] Add regression tests for fixed bugs
- [ ] Create performance regression tests

### 7.2 Integration Test Harness

```rust
#[tokio::test]
async fn test_end_to_end_emission() {
    // Setup test environment
    let temp_dir = tempdir().unwrap();
    let workspace = temp_dir.path();
    
    // Create test file
    let test_file = workspace.join("test.rs");
    fs::write(&test_file, "fn main() { println!(\"Hello\"); }").unwrap();
    
    // Start emit server
    let server = DefectEmitServer::new(workspace).await.unwrap();
    server.warm_cache(workspace).await.unwrap();
    
    // Create consumer
    let consumer = RingBufferConsumer::new(&server.emit_buffer);
    
    // Simulate edit
    let edit = TextEdit {
        path: test_file.clone(),
        changes: vec![InputEdit {
            start_byte: 12,
            old_end_byte: 12,
            new_end_byte: 40,
            start_position: Point { row: 0, column: 12 },
            old_end_position: Point { row: 0, column: 12 },
            new_end_position: Point { row: 0, column: 40 },
        }],
        content: "fn main() { for i in 0..10 { println!(\"{}\", i); } }".to_string(),
    };
    
    // Process edit
    server.on_edit(edit).await.unwrap();
    
    // Verify emission
    let payload = consumer.read_timeout(Duration::from_secs(1)).unwrap();
    assert!(payload.complexity.0 > 1); // Cyclomatic increased
    assert!(payload.tdg_score > 0.0);
}
```

**Implementation Checklist:**
- [ ] Create integration test framework
- [ ] Add multi-language test cases
- [ ] Implement stress tests
- [ ] Add latency verification tests
- [ ] Create memory leak tests

### 7.3 Benchmark Suite

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_incremental_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_parse");
    
    // Setup
    let engine = UnifiedAstEngine::new();
    let content = include_str!("../fixtures/large_file.rs");
    let tree = engine.parse(content).unwrap();
    
    group.bench_function("small_edit", |b| {
        b.iter(|| {
            let edit = InputEdit {
                start_byte: 1000,
                old_end_byte: 1001,
                new_end_byte: 1002,
                // ...
            };
            engine.parse_incremental(black_box(content), &tree, &[edit])
        });
    });
    
    group.bench_function("large_edit", |b| {
        b.iter(|| {
            let edit = InputEdit {
                start_byte: 0,
                old_end_byte: 5000,
                new_end_byte: 6000,
                // ...
            };
            engine.parse_incremental(black_box(content), &tree, &[edit])
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_incremental_parse);
criterion_main!(benches);
```

**Implementation Checklist:**
- [ ] Create parsing benchmarks
- [ ] Add complexity calculation benchmarks
- [ ] Implement ring buffer throughput tests
- [ ] Add end-to-end latency benchmarks
- [ ] Create memory allocation profiling

## 8. Deployment & Operations

### 8.1 Configuration Schema

```toml
[emit_server]
# Performance tuning
max_latency_ms = 5
ring_buffer_size = 1024
cache_size_mb = 64

# Thresholds (inherit from DeepContextConfig)
[emit_server.thresholds]
cyclomatic_complexity_warn = 10
cyclomatic_complexity_error = 20
cognitive_complexity_warn = 15
cognitive_complexity_error = 30
tdg_warn = 1.5
tdg_error = 2.0

# Language-specific overrides
[emit_server.languages.rust]
enabled = true
complexity_multiplier = 1.0

[emit_server.languages.typescript]
enabled = true
complexity_multiplier = 1.2  # TypeScript tends to be more complex
```

**Implementation Checklist:**
- [ ] Define configuration schema
- [ ] Add configuration validation
- [ ] Implement hot reload support
- [ ] Create configuration migration tool
- [ ] Add configuration documentation

### 8.2 Monitoring & Observability

```rust
#[derive(Debug, Default)]
pub struct EmitServerMetrics {
    // Latency histogram (microseconds)
    edit_latency: Histogram,
    parse_latency: Histogram,
    emit_latency: Histogram,
    
    // Throughput counters
    edits_processed: AtomicU64,
    payloads_emitted: AtomicU64,
    
    // Error counters
    parse_errors: AtomicU64,
    timeouts: AtomicU64,
    dropped_events: AtomicU64,
    
    // Resource usage
    memory_bytes: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl EmitServerMetrics {
    pub fn export_prometheus(&self) -> String {
        format!(
            "# HELP emit_server_edit_latency Edit processing latency\n\
             # TYPE emit_server_edit_latency histogram\n\
             emit_server_edit_latency_bucket{{le=\"1\"}} {}\n\
             emit_server_edit_latency_bucket{{le=\"5\"}} {}\n\
             emit_server_edit_latency_bucket{{le=\"10\"}} {}\n\
             emit_server_edit_latency_bucket{{le=\"+Inf\"}} {}\n",
            self.edit_latency.percentile(0.5),
            self.edit_latency.percentile(0.95),
            self.edit_latency.percentile(0.99),
            self.edit_latency.count(),
        )
    }
}
```

**Implementation Checklist:**
- [ ] Implement metrics collection
- [ ] Add Prometheus exporter
- [ ] Create Grafana dashboards
- [ ] Add distributed tracing
- [ ] Implement alerting rules

## 9. Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] DefectEmitServer struct and basic lifecycle
- [ ] Ring buffer implementation with tests
- [ ] Basic incremental parsing integration
- [ ] Simple emit logic without optimization

### Phase 2: Analysis Integration (Week 3-4)
- [ ] Language analyzer trait and dispatch
- [ ] Rust analyzer implementation
- [ ] TypeScript analyzer implementation
- [ ] Cache integration with UnifiedCacheManager

### Phase 3: Performance Optimization (Week 5-6)
- [ ] SIMD TDG calculation
- [ ] Parallel cache warming
- [ ] Lock-free optimizations
- [ ] Latency profiling and tuning

### Phase 4: Client Integration (Week 7-8)
- [ ] MCP tool registration
- [ ] LSP diagnostic protocol
- [ ] VS Code extension updates
- [ ] End-to-end testing

### Phase 5: Production Hardening (Week 9-10)
- [ ] Error recovery mechanisms
- [ ] Monitoring and alerting
- [ ] Documentation
- [ ] Performance benchmarks

## 10. Success Criteria

### Performance Metrics
- **P50 Latency**: < 2ms
- **P95 Latency**: < 4ms
- **P99 Latency**: < 5ms
- **Throughput**: > 10K edits/second
- **Memory Usage**: < 64MB RSS
- **CPU Usage**: < 2% single core

### Quality Metrics
- **Test Coverage**: > 90%
- **Benchmark Stability**: < 5% variance
- **Zero Memory Leaks**: Verified by Valgrind
- **API Compatibility**: 100% backward compatible

### User Experience
- **Setup Time**: < 30 seconds
- **First Emission**: < 100ms after edit
- **False Positive Rate**: < 5%
- **User Satisfaction**: > 90% positive feedback

## Appendix A: API Reference

[Detailed API documentation would go here]

## Appendix B: Troubleshooting Guide

[Common issues and solutions would go here]

## Appendix C: Performance Tuning Guide

[Advanced tuning parameters would go here]

<svg viewBox="0 0 1200 800" xmlns="http://www.w3.org/2000/svg">
  <!-- Background -->
  <rect width="1200" height="800" fill="#f8f9fa"/>

  <!-- Title -->
  <text x="600" y="40" text-anchor="middle" font-family="Arial, sans-serif" font-size="24" font-weight="bold" fill="#1a1a1a">
    Defect Emit Server Architecture
  </text>

  <!-- Editor/IDE -->
  <g id="editor">
    <rect x="50" y="100" width="180" height="80" rx="8" fill="#e3f2fd" stroke="#1976d2" stroke-width="2"/>
    <text x="140" y="145" text-anchor="middle" font-family="Arial, sans-serif" font-size="16" font-weight="bold" fill="#1976d2">Text Editor/IDE</text>
    <text x="140" y="165" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#424242">TextEdit events</text>
  </g>

  <!-- Defect Emit Server Box -->
  <g id="emit-server">
    <rect x="320" y="80" width="560" height="420" rx="12" fill="#fff" stroke="#424242" stroke-width="2" stroke-dasharray="8,4"/>
    <text x="600" y="110" text-anchor="middle" font-family="Arial, sans-serif" font-size="18" font-weight="bold" fill="#424242">Defect Emit Server</text>

    <!-- Incremental Parser -->
    <rect x="340" y="140" width="200" height="60" rx="6" fill="#e8f5e9" stroke="#4caf50" stroke-width="2"/>
    <text x="440" y="165" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#2e7d32">Incremental Parser</text>
    <text x="440" y="185" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">UnifiedAstEngine</text>
    
    <!-- Cache Manager -->
    <rect x="340" y="220" width="200" height="60" rx="6" fill="#fce4ec" stroke="#e91e63" stroke-width="2"/>
    <text x="440" y="245" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#880e4f">Cache Manager</text>
    <text x="440" y="265" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">UnifiedCacheManager</text>
    
    <!-- Analyzer Dispatcher -->
    <rect x="560" y="140" width="180" height="140" rx="6" fill="#f3e5f5" stroke="#9c27b0" stroke-width="2"/>
    <text x="650" y="165" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#6a1b9a">Analyzer Dispatch</text>
    <text x="650" y="185" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">• Rust Analyzer</text>
    <text x="650" y="200" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">• TS Analyzer</text>
    <text x="650" y="215" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">• Python Analyzer</text>
    <text x="650" y="230" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">• C/C++ Analyzer</text>
    <text x="650" y="250" text-anchor="middle" font-family="Arial, sans-serif" font-size="10" fill="#757575">+ 9 more languages</text>
    
    <!-- Metric Computers -->
    <rect x="340" y="300" width="240" height="80" rx="6" fill="#fff3e0" stroke="#ff9800" stroke-width="2"/>
    <text x="460" y="325" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#e65100">Metric Computers</text>
    <text x="460" y="345" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">• TDGCalculator (vectorized)</text>
    <text x="460" y="360" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">• ComplexityVisitor</text>
    <text x="460" y="375" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">• DeadCodeAnalyzer (bitset)</text>
    
    <!-- Ring Buffer -->
    <rect x="600" y="300" width="140" height="80" rx="6" fill="#e0f2f1" stroke="#009688" stroke-width="2"/>
    <text x="670" y="325" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#00695c">Ring Buffer</text>
    <text x="670" y="345" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">Lock-free SPSC</text>
    <text x="670" y="360" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">64-byte aligned</text>
    <text x="670" y="375" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">1024 slots</text>
    
    <!-- Threshold Gate -->
    <rect x="760" y="300" width="100" height="80" rx="6" fill="#ffebee" stroke="#f44336" stroke-width="2"/>
    <text x="810" y="325" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#c62828">Threshold</text>
    <text x="810" y="345" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">Gate</text>
    <text x="810" y="365" text-anchor="middle" font-family="Arial, sans-serif" font-size="10" fill="#757575">cy > 10</text>
    
    <!-- Emit Controller -->
    <rect x="460" y="400" width="160" height="60" rx="6" fill="#e8eaf6" stroke="#3f51b5" stroke-width="2"/>
    <text x="540" y="425" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#283593">Emit Controller</text>
    <text x="540" y="445" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">< 5ms latency</text>
  </g>

  <!-- Existing Infrastructure -->
  <g id="infrastructure">
    <rect x="50" y="300" width="200" height="180" rx="8" fill="#f5f5f5" stroke="#757575" stroke-width="2"/>
    <text x="150" y="330" text-anchor="middle" font-family="Arial, sans-serif" font-size="16" font-weight="bold" fill="#424242">Existing Infrastructure</text>
    <text x="150" y="355" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#616161">• IncrementalCoverageAnalyzer</text>
    <text x="150" y="375" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#616161">• HierarchicalBitSet</text>
    <text x="150" y="395" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#616161">• AstStrategy trait</text>
    <text x="150" y="415" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#616161">• VectorizedCacheKey</text>
    <text x="150" y="435" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#616161">• 353 calibrated defects</text>
    <text x="150" y="455" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#616161">• MCP Protocol handlers</text>
  </g>

  <!-- Output Targets -->
  <g id="outputs">
    <rect x="950" y="200" width="180" height="60" rx="8" fill="#e1f5fe" stroke="#0288d1" stroke-width="2"/>
    <text x="1040" y="225" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#01579b">LSP Client</text>
    <text x="1040" y="245" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">IDE diagnostics</text>

    <rect x="950" y="280" width="180" height="60" rx="8" fill="#f9fbe7" stroke="#827717" stroke-width="2"/>
    <text x="1040" y="305" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#558b2f">Metrics Collector</text>
    <text x="1040" y="325" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">Prometheus/Grafana</text>
    
    <rect x="950" y="360" width="180" height="60" rx="8" fill="#efebe9" stroke="#5d4037" stroke-width="2"/>
    <text x="1040" y="385" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#3e2723">CI/CD Pipeline</text>
    <text x="1040" y="405" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="#424242">Quality gates</text>
  </g>

  <!-- Data Flow Arrows -->
  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
    </marker>
  </defs>

  <!-- Editor to Parser -->
  <path d="M 230 140 L 340 170" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
  <text x="285" y="150" text-anchor="middle" font-family="Arial, sans-serif" font-size="10" fill="#666">TextEdit</text>

  <!-- Parser to Cache -->
  <path d="M 440 200 L 440 220" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Parser to Analyzers -->
  <path d="M 540 170 L 560 170" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Cache to Metrics -->
  <path d="M 440 280 L 440 300" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Analyzers to Metrics -->
  <path d="M 650 280 L 580 340" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Metrics to Ring Buffer -->
  <path d="M 580 340 L 600 340" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Ring Buffer to Threshold -->
  <path d="M 740 340 L 760 340" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Threshold to Emit -->
  <path d="M 810 380 L 620 430" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Infrastructure to Components -->
  <path d="M 250 390 L 320 340" stroke="#999" stroke-width="1" stroke-dasharray="4,2"/>

  <!-- Emit to Outputs -->
  <path d="M 620 430 L 950 230" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
  <path d="M 620 430 L 950 310" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
  <path d="M 620 430 L 950 390" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>

  <!-- Performance Metrics -->
  <g id="perf-metrics">
    <rect x="50" y="550" width="300" height="80" rx="6" fill="#263238" stroke="none"/>
    <text x="200" y="575" text-anchor="middle" font-family="monospace" font-size="12" fill="#b0bec5">Performance Profile</text>
    <text x="60" y="595" font-family="monospace" font-size="11" fill="#4caf50">AST parse (cached): 2.5ms</text>
    <text x="60" y="610" font-family="monospace" font-size="11" fill="#4caf50">TDG calculation:    0.8ms (SIMD)</text>
    <text x="60" y="625" font-family="monospace" font-size="11" fill="#4caf50">Dead code scan:     1.2ms (bitset)</text>
  </g>

  <!-- Payload Structure -->
  <g id="payload">
    <rect x="400" y="550" width="400" height="80" rx="6" fill="#37474f" stroke="none"/>
    <text x="600" y="575" text-anchor="middle" font-family="monospace" font-size="12" fill="#b0bec5">DefectPayload (64 bytes, cache-aligned)</text>
    <text x="410" y="595" font-family="monospace" font-size="10" fill="#81c784">file_hash: u64    | tdg_score: f32</text>
    <text x="410" y="610" font-family="monospace" font-size="10" fill="#81c784">complexity: (u16,u16) | dead_symbols: u32</text>
    <text x="410" y="625" font-family="monospace" font-size="10" fill="#81c784">timestamp: u64    | _padding: [u8; 16]</text>
  </g>

  <!-- Key Features -->
  <g id="features">
    <rect x="850" y="550" width="300" height="80" rx="6" fill="#1a237e" stroke="none"/>
    <text x="1000" y="575" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#e8eaf6">Key Features</text>
    <text x="860" y="595" font-family="Arial, sans-serif" font-size="11" fill="#9fa8da">• Zero-copy ring buffer emission</text>
    <text x="860" y="610" font-family="Arial, sans-serif" font-size="11" fill="#9fa8da">• Language-agnostic (13 supported)</text>
    <text x="860" y="625" font-family="Arial, sans-serif" font-size="11" fill="#9fa8da">• Sub-5ms edit-to-emit latency</text>
  </g>
</svg>