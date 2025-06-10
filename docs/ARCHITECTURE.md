# PAIML MCP Agent Toolkit - Architecture

## Overview

PMAT is designed as a unified binary that serves multiple protocols (CLI, MCP, HTTP) while maintaining consistent behavior and performance characteristics across all interfaces.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           User Interfaces                           │
├─────────────────┬─────────────────┬─────────────────┬──────────────┤
│       CLI       │   MCP (stdio)   │   HTTP API     │     TUI      │
└─────────────────┴─────────────────┴─────────────────┴──────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Unified Protocol Layer                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────┐ │
│  │ CLI Adapter │  │ MCP Adapter  │  │ HTTP Adapter │  │TUI Adapter│ │
│  └─────────────┘  └──────────────┘  └──────────────┘  └─────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Service Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────┐ │
│  │ AST Engine   │  │  Analyzers   │  │ Generators   │  │ Cache  │ │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤  ├────────┤ │
│  │ • Rust       │  │ • Complexity │  │ • Context    │  │• Memory│ │
│  │ • TypeScript │  │ • Churn      │  │ • Mermaid    │  │• Disk  │ │
│  │ • Python     │  │ • Dead Code  │  │ • Templates  │  │• Hybrid│ │
│  │ • C/C++      │  │ • TDG        │  │ • Reports    │  └────────┘ │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          Data Layer                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────┐ │
│  │ File System  │  │     Git      │  │   Database   │  │ Models │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Unified Protocol Layer

The protocol layer ensures consistent behavior across all interfaces:

```rust
#[async_trait]
trait ProtocolAdapter: Send + Sync {
    type Input: DeserializeOwned;
    type Output: Serialize;
    
    async fn decode(&self, raw: &[u8]) -> Result<Self::Input>;
    async fn process(&self, req: UnifiedRequest) -> Result<UnifiedResponse>;
    async fn encode(&self, resp: UnifiedResponse) -> Result<Vec<u8>>;
}
```

**Key Features:**
- Protocol-agnostic request/response handling
- Consistent error propagation
- Unified logging and metrics
- Streaming support for large payloads

### 2. Service Layer

#### AST Engine
Multi-language parsing with unified AST representation:

```rust
pub struct UnifiedAstEngine {
    parsers: HashMap<Language, Box<dyn AstParser>>,
    cache: Arc<UnifiedCacheManager>,
}

pub enum UnifiedAstNode {
    Function(FunctionNode),
    Class(ClassNode),
    Module(ModuleNode),
    // ... other node types
}
```

**Supported Languages:**
- Rust (via `syn`)
- TypeScript/JavaScript (via `swc`)
- Python (via `rustpython-parser`)
- C/C++ (via `tree-sitter`)

#### Analyzers
Pluggable analysis modules:

- **Complexity Analyzer**: Cyclomatic and cognitive complexity
- **Churn Analyzer**: Git history analysis
- **Dead Code Analyzer**: Unused code detection
- **TDG Calculator**: Technical debt gradient
- **SATD Detector**: Self-admitted technical debt
- **Provability Analyzer**: Lightweight formal verification
- **Big-O Analyzer**: Algorithmic complexity detection
- **Makefile Linter**: 50+ quality rules for Makefiles
- **Graph Metrics**: PageRank and centrality analysis
- **Name Similarity**: Semantic name search with embeddings
- **Defect Prediction**: ML-based defect probability
- **Incremental Coverage**: Coverage change tracking

#### Cache System
Multi-tier caching for performance:

```rust
pub struct UnifiedCacheManager {
    l1_cache: ThreadLocal<LruCache<CacheKey, Arc<CachedValue>>>,
    l2_cache: Arc<DashMap<CacheKey, Arc<CachedValue>>>,
    l3_cache: Arc<PersistentCache>,
}
```

**Cache Hierarchy:**
- L1: Thread-local LRU (15ns latency)
- L2: Process-wide concurrent (200ns latency)
- L3: Persistent SQLite (500μs latency)

### 3. Data Models

#### Core Models

```rust
pub struct Project {
    pub path: PathBuf,
    pub language: Language,
    pub files: Vec<SourceFile>,
    pub metadata: ProjectMetadata,
}

pub struct AnalysisResult {
    pub complexity: ComplexityMetrics,
    pub quality: QualityMetrics,
    pub dependencies: DependencyGraph,
    pub issues: Vec<Issue>,
}
```

#### Metrics Models

```rust
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub halstead: HalsteadMetrics,
    pub maintainability_index: f64,
}

pub struct QualityMetrics {
    pub test_coverage: f64,
    pub documentation_coverage: f64,
    pub type_coverage: f64,
    pub tdg_score: f64,
}
```

## Performance Architecture

### Concurrency Model

```rust
// CPU-bound work: Rayon thread pool
let pool = rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .build()?;

// I/O-bound work: Tokio runtime
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .enable_all()
    .build()?;
```

### Memory Management

- **Arena Allocation**: For AST nodes
- **String Interning**: For identifiers
- **Zero-Copy Parsing**: Where possible
- **Streaming Processing**: For large files

### Optimization Strategies

1. **Incremental Analysis**: Only re-analyze changed files
2. **Parallel Processing**: Use all available cores
3. **Smart Caching**: Content-based invalidation
4. **Lazy Evaluation**: Compute metrics on demand

## Protocol Specifications

### CLI Protocol

Command structure follows a hierarchical pattern:

```
pmat [global-options] <command> [subcommand] [options]
```

**Example:**
```bash
pmat --verbose analyze complexity --top-files 10
```

### MCP Protocol

JSON-RPC 2.0 over stdio:

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "analyze_complexity",
    "arguments": { "path": "." }
  },
  "id": "1"
}
```

### HTTP API

RESTful design with JSON payloads:

```
GET  /api/v1/health
GET  /api/v1/analyze/complexity?path=.&top_files=10
POST /api/v1/analyze/deep-context
GET  /api/v1/templates
POST /api/v1/generate
```

## Security Architecture

### Input Validation

```rust
pub struct ValidatedPath(PathBuf);

impl ValidatedPath {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // Prevent path traversal
        if path.components().any(|c| c == Component::ParentDir) {
            return Err(Error::InvalidPath);
        }
        
        // Ensure within allowed directories
        if !is_allowed_path(path) {
            return Err(Error::Unauthorized);
        }
        
        Ok(Self(path.to_owned()))
    }
}
```

### Sandboxing

- File system access limited to project directory
- No network access during analysis
- Resource limits (CPU, memory, time)
- No arbitrary code execution

## Extension Points

### Custom Analyzers

```rust
#[async_trait]
pub trait Analyzer: Send + Sync {
    fn name(&self) -> &str;
    fn supported_languages(&self) -> &[Language];
    async fn analyze(&self, ast: &UnifiedAst) -> Result<AnalysisResult>;
}

// Register custom analyzer
engine.register_analyzer(Box::new(MyCustomAnalyzer::new()));
```

### Custom Rules

```rust
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn severity(&self) -> Severity;
    fn check(&self, context: &RuleContext) -> Vec<Violation>;
}
```

### Output Formatters

```rust
pub trait Formatter: Send + Sync {
    fn format(&self, result: &AnalysisResult) -> Result<String>;
    fn mime_type(&self) -> &str;
}
```

## Error Handling

### Error Hierarchy

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Analysis error: {0}")]
    Analysis(String),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
}
```

### Error Recovery

- Partial results on failure
- Graceful degradation
- Clear error messages
- Suggested fixes

## Testing Architecture

### Test Categories

1. **Unit Tests**: Component isolation
2. **Integration Tests**: Component interaction
3. **E2E Tests**: Full workflow validation
4. **Property Tests**: Invariant checking
5. **Fuzzing**: Security and robustness

### Test Infrastructure

```rust
// Test builders for complex scenarios
let project = ProjectBuilder::new()
    .add_file("src/main.rs", MAIN_CONTENT)
    .add_file("src/lib.rs", LIB_CONTENT)
    .with_git_history(commits)
    .build();

let result = analyzer.analyze(&project).await?;
assert_eq!(result.complexity.cyclomatic, 10);
```

## Deployment Architecture

### Binary Distribution

Single static binary with embedded assets:

```
pmat (15MB)
├── Main executable
├── Embedded templates (compressed)
├── Embedded web assets (minified)
└── Embedded documentation
```

### Platform Support

- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64) [planned]

### Installation Methods

1. **Direct Download**: GitHub releases
2. **Package Managers**: brew, apt, yum [planned]
3. **Cargo**: `cargo install pmat`
4. **Docker**: `docker run paiml/pmat`

## Monitoring and Observability

### Metrics

- Request latency (p50, p90, p99)
- Cache hit rates
- Memory usage
- Analysis throughput

### Logging

Structured logging with contextual information:

```rust
info!(
    target: "analysis",
    file = %path.display(),
    duration_ms = elapsed.as_millis(),
    "Completed complexity analysis"
);
```

### Tracing

OpenTelemetry-compatible tracing:

```rust
#[instrument(skip(ast))]
async fn analyze_complexity(ast: &UnifiedAst) -> Result<ComplexityMetrics> {
    // Analysis implementation
}
```

## Major Feature Architectures

### Refactor Engine

The refactor engine uses a multi-stage pipeline for safe code transformation:

```rust
pub struct RefactorEngine {
    analyzer: UnifiedRefactorAnalyzer,
    transformer: TransformationEngine,
    validator: RefactorValidator,
    checkpointer: CheckpointManager,
}

pub struct RefactorPipeline {
    stages: Vec<Box<dyn RefactorStage>>,
    rollback_manager: RollbackManager,
    test_runner: TestRunner,
}
```

**Pipeline Stages:**
1. **Analysis**: Identify refactoring opportunities
2. **Planning**: Generate transformation plan
3. **Validation**: Ensure semantic preservation
4. **Execution**: Apply transformations
5. **Verification**: Run tests and validate

### Provability Analysis System

Lightweight formal verification using abstract interpretation:

```rust
pub struct ProvabilityAnalyzer {
    property_domains: HashMap<PropertyType, Box<dyn PropertyDomain>>,
    inference_engine: InferenceEngine,
    cache: PropertyCache,
}

pub trait PropertyDomain {
    fn analyze(&self, node: &AstNode) -> PropertyResult;
    fn merge(&self, a: &Property, b: &Property) -> Property;
    fn widen(&self, a: &Property, b: &Property) -> Property;
}
```

**Supported Properties:**
- Nullability lattice
- Alias analysis
- Range intervals
- Initialization states
- Taint propagation

### Big-O Complexity Detection

Pattern-based algorithmic complexity analysis:

```rust
pub struct BigOAnalyzer {
    pattern_matcher: PatternMatcher,
    loop_analyzer: LoopComplexityAnalyzer,
    recursion_detector: RecursionAnalyzer,
    evidence_collector: EvidenceCollector,
}

pub enum ComplexityClass {
    Constant,
    Logarithmic,
    Linear,
    Linearithmic,
    Quadratic,
    Cubic,
    Exponential,
}
```

### Makefile Linting Engine

Rule-based linting with auto-fix capabilities:

```rust
pub struct MakefileLinter {
    rule_engine: RuleEngine,
    parser: MakefileParser,
    auto_fixer: AutoFixer,
}
```

## Testing Architecture

### Distributed Test Strategy

The project employs a stratified test architecture to achieve sub-linear compilation scaling and maximize parallel execution. Tests are organized into 5 independent binaries:

```
tests/
├── unit/core.rs           # <10s - Core logic, zero I/O
├── integration/
│   ├── services.rs        # <30s - Service integration
│   └── protocols.rs       # <45s - Protocol validation
├── e2e/system.rs          # <120s - Full workflows
└── performance/regression.rs  # Performance baselines
```

**Key Benefits:**
- 65-80% faster build times through parallel compilation
- <1s feedback for unit tests
- 4x throughput via parallel execution
- Selective instrumentation for coverage

### Test Organization

```rust
pub struct MakefileLinter {
    parser: MakefileParser,
    rules: Vec<Box<dyn LintRule>>,
    fixer: AutoFixer,
    config: LintConfig,
}

pub trait LintRule {
    fn check(&self, ast: &MakefileAst) -> Vec<LintIssue>;
    fn fix(&self, issue: &LintIssue) -> Option<Fix>;
}
```

## Future Architecture Considerations

1. **Plugin System**: Dynamic loading of analyzers
2. **Distributed Analysis**: Cluster support for large codebases
3. **Real-time Monitoring**: Continuous analysis daemon
4. **Cloud Integration**: SaaS offering
5. **IDE Integration**: Language Server Protocol
6. **GPU Acceleration**: For ML-based analysis
7. **Incremental Compilation**: Faster re-analysis