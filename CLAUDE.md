# CLAUDE.md

## System Architecture Overview

This document serves as the operational guide for the paiml-mcp-agent-toolkit (pmat), a unified protocol implementation supporting CLI, MCP, and HTTP interfaces through a single binary architecture.

**Core Design Principle**: Protocol-agnostic service layer with deterministic behavior across all interfaces.
- Jidoka (自働化): Build quality in through proper error handling and verification (Never use TODO or leave unfinished code)
- Genchi Genbutsu (現地現物): Go and see the actual root causes instead of statistical approximations
- Hansei (反省): Focus on fixing existing broken functionality rather than adding new features
- Kaizen - Continuous Improvement

## Zero Tolerance Quality Standards

### ABSOLUTE RULES - NO EXCEPTIONS
1. **ZERO SATD**: No TODO, FIXME, HACK, XXX, or placeholder implementations allowed
2. **ZERO High Complexity**: No function may exceed cyclomatic complexity of 20
3. **ZERO Known Defects**: All code must be fully functional before committing
4. **ZERO Incomplete Features**: Only merge complete, tested, documented features

### Implementation Standards
- **No Placeholders**: Every function must have a complete implementation
- **No Temporary Code**: All code is production-ready or it doesn't exist
- **No Workarounds**: Fix the root cause, not the symptom
- **No Technical Debt**: Pay as you go - fix issues immediately

### Quality Gates
```bash
# These must ALL pass before ANY commit:
make lint          # Zero warnings
make test-fast     # 100% pass rate
pmat analyze satd  # Zero SATD items
pmat analyze complexity --max-cyclomatic 20  # No violations
```

If you cannot implement something completely, do not implement it at all.

## QA V2 Framework Integration

### Validation Pipeline
The system now incorporates a comprehensive QA v2 framework with 755+ tests validating:

```bash
# QA validation commands (run automatically in CI/CD)
make lint          # Zero tolerance for linting warnings
make format        # Consistent code formatting 
make test-fast     # 755+ tests with maximum parallelism
make release       # Optimized binary generation
pmat context       # Self-analysis for validation
```

### Test Categories
- **Environment Variable Integration**: 21 tests with global mutex pattern
- **CLI Structure Validation**: 18 tests for command hierarchy
- **Argument Parsing**: 28 tests for type coercion and edge cases
- **Code Smell Detection**: 22 tests for comprehensive quality analysis
- **Interface Consistency**: Triple-interface testing (CLI, MCP, HTTP)
- **Performance Validation**: Sub-10ms response time guarantees
- **C/C++ AST Integration**: 747 tests including new language support
- **Provability Analysis**: Formal verification testing with confidence scoring

## C/C++ AST Integration & Provability Analysis

### C/C++ Language Support
The system now includes comprehensive C/C++ language analysis capabilities:

```rust
// Toyota Way implementation - proper name extraction without shortcuts
impl CAstStrategy {
    fn extract_name_from_node(node: &UnifiedAstNode, content: &str) -> Option<String> {
        let start = node.source_range.start as usize;
        let end = node.source_range.end as usize;
        let source_text = &content[start..end];
        
        // Parse actual source text for accurate names
        match &node.kind {
            AstKind::Function(_) => Self::extract_function_name(source_text),
            AstKind::Type(_) => Self::extract_type_name(source_text),
            _ => None,
        }
    }
}
```

**Key Features:**
- Tree-sitter based parsing for C and C++
- Proper name extraction from source ranges
- Support for C-specific constructs (goto, labels, macros)
- C++ specific features (classes, templates, operator overloads)
- Byte position to line number conversion
- .gitignore respect for build artifacts

### Provability Analysis Framework
Lightweight formal verification system providing:

```rust
pub struct LightweightProvabilityAnalyzer {
    property_cache: DashMap<FunctionId, Vec<VerifiedProperty>>,
    confidence_threshold: f64,
    analysis_timeout: Duration,
}

// Property domain analysis with lattice structures
pub enum PropertyType {
    Nullability(NullabilityLattice),
    AliasAnalysis(AliasLattice),
    // Additional property types...
}
```

**Capabilities:**
- Property domain analysis with confidence scoring
- Incremental analysis with efficient caching
- Integration with deep context analysis
- Quality gates with automated verification
- Nullability lattice and alias analysis

## Dynamic Context Analysis Protocol

### Initialization Sequence

```bash
# Bootstrap analysis environment - extract current system state
# Performance: ~0.3s for 10K LOC codebase
rg --json -A20 '^## Complexity Hotspots' deep_context.md | \
  jq -r '.data.lines.text' | \
  grep -E '^\|.*\|.*\|.*\|' | \
  awk -F'|' '$4 > 30 {print $2, $4, $5}' | \
  sort -k2 -nr

# Verify architectural invariants
find . -name "*.rs" -type f | \
  xargs grep -l "trait ProtocolAdapter" | \
  wc -l  # Expected: 3 (one per protocol)
```

### Deep Context Utilization

The `deep_context.md` file contains pre-computed AST analysis with O(1) lookup characteristics:

```bash
# High-risk component identification (defect probability > 50%)
rg "Defect Probability: [5-9]\d\.\d+%" deep_context.md | \
  cut -d: -f1 | xargs -I{} basename {} .md | \
  sort -u > high_risk_components.txt

# Complexity density analysis - identify refactoring targets
rg "Cognitive.*[3-9]\d+" deep_context.md | \
  sed 's/.*\///' | cut -d: -f1 | \
  sort | uniq -c | sort -nr | head -10
```

## Architectural Invariants

### Workspace Topology

```
workspace/
├── Cargo.toml          # Workspace manifest (single source of truth)
├── server/             # Unified binary crate
│   ├── src/
│   │   ├── services/   # Stateful business logic (60% complexity mass)
│   │   ├── unified_protocol/  # Protocol adapters (thin translation layer)
│   │   └── handlers/   # Request routing (stateless, <5% complexity)
│   └── build.rs        # Asset compression pipeline (zstd level 19)
└── target/release/     # Single 15MB binary serving all protocols
```

### Protocol Unification Architecture

```rust
// Invariant: All protocols converge through this trait
// Performance: <100μs overhead per request
trait ProtocolAdapter: Send + Sync + 'static {
    type Input: DeserializeOwned + Debug;
    type Output: Serialize + Debug;
    type Context: Send + Default;

    async fn decode(&self, raw: &[u8]) -> Result<Self::Input>;
    async fn process(&self, req: UnifiedRequest) -> Result<UnifiedResponse>;
    async fn encode(&self, resp: UnifiedResponse) -> Result<Vec<u8>>;
}

// Concrete implementations maintain protocol semantics
impl ProtocolAdapter for McpAdapter {
    // JSON-RPC 2.0 compliance with stdio transport
}

impl ProtocolAdapter for HttpAdapter {
    // REST semantics with HTTP/2 support
}

impl ProtocolAdapter for CliAdapter {
    // POSIX-compliant argument parsing
}
```

### Concurrency Model

- **Runtime**: Tokio multi-threaded (work-stealing scheduler, NUMA-aware)
- **Shared State**: `Arc<RwLock<T>>` for services, `DashMap` for caches
- **CPU-Bound**: Rayon thread pool (size = physical cores)
- **I/O Model**: epoll-based async with 64KB buffer pools

## Performance Engineering

### Graph Intelligence System

**Problem**: Mermaid renderer fails at >500 edges; typical codebases generate 2000+ edges.

**Solution**: PageRank-based graph reduction with architectural significance preservation.

```rust
// Implementation achieves 5x speedup with 90% significance retention
pub fn reduce_graph(graph: &DiGraph, target_nodes: usize) -> DiGraph {
    // PageRank computation: O(E * iterations)
    let scores = pagerank(&graph, 0.85, 10);
    
    // Top-K selection with safety margin
    let threshold = scores.values()
        .sorted_by(|a, b| b.partial_cmp(a).unwrap())
        .nth(target_nodes.min(graph.node_count() * 4 / 5))
        .copied()
        .unwrap_or(0.0);
    
    // Subgraph extraction preserving connectedness
    graph.filter_map(
        |idx, _| if scores[idx] >= threshold { Some(()) } else { None },
        |_, edge| Some(edge.clone())
    )
}
```

**Metrics**:
- Before: 4.247s analysis, 2000+ edges, rendering failures
- After: 0.823s analysis, <400 edges, 100% render success
- Memory: 12MB → 3MB working set reduction

### Cache Hierarchy

```rust
// L1: Thread-local (p50: 15ns, p99: 50ns)
thread_local! {
    static AST_CACHE: RefCell<LruCache<Blake3Hash, Arc<ParsedAst>>> = 
        RefCell::new(LruCache::new(NonZeroUsize::new(128).unwrap()));
}

// L2: Process-wide (p50: 200ns, p99: 1μs)
static SHARED_CACHE: Lazy<DashMap<CacheKey, Arc<CachedValue>>> = 
    Lazy::new(|| DashMap::with_capacity_and_hasher(4096, Blake3Hasher));

// L3: Persistent (p50: 500μs, p99: 2ms)
// SQLite with WAL mode, mmap, and prepared statements
struct PersistentCache {
    conn: Arc<Mutex<Connection>>,
    stmts: DashMap<&'static str, Statement>,
}

// Cache key design ensures deterministic invalidation
#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    content_hash: [u8; 32],  // Blake3 hash
    mtime_ns: u64,           // Nanosecond precision
    path_hash: [u8; 32],     // Canonical path hash
}
```

## Complexity Management

### Hotspot Detection

```bash
# Real-time complexity analysis with cognitive load weighting
COGNITIVE_THRESHOLD=25
rg '^\|[^|]+\|[^|]+\|[^|]+\|\s*(\d+)\s*\|\s*(\d+)\s*\|$' deep_context.md | \
  awk -F'|' -v t=$COGNITIVE_THRESHOLD \
    '$4 > t { printf "%-40s Cognitive: %3d Cyclomatic: %3d Ratio: %.2f\n", \
    $2, $4, $5, $4/$5 }' | \
  sort -k8 -nr
```

### Refactoring Patterns

```rust
// Pattern: Pipeline Architecture for Complex Operations
// Reduces cognitive complexity from 45 to 12
impl Service {
    pub async fn process(&self, input: Input) -> Result<Output> {
        // Composable stages with error propagation
        let pipeline = Pipeline::builder()
            .stage("validate", |i| self.validate(i))
            .stage("normalize", |i| self.normalize(i))
            .stage("analyze", |i| self.analyze(i))
            .stage("transform", |i| self.transform(i))
            .with_telemetry()
            .with_retry(3, Duration::from_millis(100))
            .build();
            
        pipeline.execute(input).await
    }
}
```

## Development Workflow

### Protocol Equivalence Testing

```bash
# Automated semantic equivalence verification
test_protocol_equivalence() {
    local test_input='{"method":"analyze_ast","params":{"path":"src/main.rs"}}'
    local canonical_output="/tmp/canonical_$$.json"
    
    # Generate canonical output via CLI
    echo "$test_input" | \
        cargo run --release -- analyze ast src/main.rs --json | \
        jq -S . > "$canonical_output"
    
    # Verify MCP protocol
    echo "$test_input" | \
        cargo run --release -- --protocol mcp | \
        jq -S .result | \
        diff -u "$canonical_output" - || return 1
    
    # Verify HTTP protocol
    curl -s -X POST localhost:3000/api/v1/analyze/ast \
        -H "Content-Type: application/json" \
        -d "$test_input" | \
        jq -S . | \
        diff -u "$canonical_output" - || return 1
    
    rm "$canonical_output"
}
```

### Performance Profiling

```bash
# CPU flame graph generation for hotspot identification
cargo build --release
perf record -F 997 -g -- ./target/release/pmat analyze dag .
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg

# Memory allocation tracking
DHAT_OUTPUT=dhat.out cargo run --release --features dhat-heap -- analyze complexity .
dh_view.js dhat.out  # Visualize allocation patterns
```

## Release Engineering

### Automated Release Pipeline

```bash
# Semantic version bumping with changelog generation
gh workflow run "Simple Release" --field version_bump=minor

# Version bump semantics:
# - patch: Bug fixes, documentation (0.18.2 → 0.18.3)
# - minor: New features, tools (0.18.2 → 0.19.0)  
# - major: Breaking changes (0.18.2 → 1.0.0)
```

### Binary Artifact Matrix

| Platform | Architecture | Binary Size | Compression |
|----------|--------------|-------------|-------------|
| Linux | x86_64 | 15.2 MB | zstd -19 |
| Linux | aarch64 | 14.8 MB | zstd -19 |
| macOS | x86_64 | 16.1 MB | zstd -19 |
| macOS | aarch64 | 15.7 MB | zstd -19 |

## Correctness Invariants

### Determinism Requirements

```rust
// File ordering: UTF-8 canonical
paths.sort_by(|a, b| a.as_os_str().cmp(b.as_os_str()));

// Hash stability: Platform-independent Blake3
let hash = blake3::Hasher::new()
    .update(content)
    .finalize();

// Time handling: UTC with second precision
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_secs();

// Numeric stability: Integer-only scoring
let score = (probability * 1000.0) as u32;  // 3 decimal places
```

### Safety Boundaries

```rust
// Input validation pipeline with compile-time guarantees
pub fn process_request<T: Validate>(raw: T) -> Result<Response> {
    let validated = raw
        .validate_structure()?      // Schema validation
        .validate_semantics()?      // Business rules
        .validate_security()?       // Path traversal, injection
        .seal();                    // Type-state transition
    
    // validated: ValidatedInput<T> - safe to process
    handle_validated_request(validated)
}
```

## Operational Guidelines

1. **Default Commands**:
  - `make test-fast` for rapid iteration (3s test suite)
  - `rg` over `grep` for 10x performance on large codebases

2. **Performance Targets**:
  - p50 latency: <10ms for AST operations
  - p99 latency: <100ms for full repository analysis
  - Memory ceiling: 500MB for 100K LOC analysis

3. **Debugging Protocol**:
   ```bash
   # Enable trace logging for specific subsystem
   RUST_LOG=paiml_mcp_agent_toolkit::services::ast=trace cargo run
   
   # Profile specific operation
   hyperfine --warmup 3 \
     'cargo run --release -- analyze complexity src/'
   ```

This architecture prioritizes deterministic correctness across protocol boundaries while maintaining sub-second response times for typical development workflows. The unified service layer ensures behavioral equivalence regardless of entry point, enabling seamless integration across diverse toolchains.