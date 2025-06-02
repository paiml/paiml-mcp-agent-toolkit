# System Architecture

## Memory Safety Invariants

### 1. **Template Lifetime Management**

All templates are `'static` with compile-time embedding:

```rust
const TEMPLATES: &[(&str, &str)] = &[
    ("makefile/rust/cli", include_str!("../templates/makefile/rust/cli.hbs")),
    ("makefile/deno/cli", include_str!("../templates/makefile/deno/cli.hbs")),
    ("makefile/python-uv/cli", include_str!("../templates/makefile/python-uv/cli.hbs")),
    // Compiler enforces 'static lifetime
];
```

### 2. **Concurrency Model**

The system employs **fearless concurrency** patterns:

```rust
pub struct Server {
    // Shared immutable state
    templates: Arc<TemplateStore>,
    
    // Per-connection state (no sharing)
    connections: DashMap<ConnectionId, Connection>,
    
    // Global mutable state with fine-grained locking
    cache: Arc<CacheHierarchy>,
}

// Safe concurrent access via type system
impl Server {
    pub fn handle_request(&self, conn_id: ConnectionId, req: Request) {
        // No data races possible - enforced at compile time
        let conn = self.connections.get(&conn_id).unwrap();
        let template = self.templates.get(&req.template_uri);
        // ...
    }
}
```

## Zero-Copy I/O Pipeline

```
stdin → AlignedBuffer → serde_json → Request
                                        ↓
stdout ← io::Write ← Response ← Template
```

### Buffer Management

Pre-allocated buffer pools with RAII cleanup:

```rust
struct BufferPool {
    free: SegQueue<Box<[u8; 64 * 1024]>>,
    allocated: AtomicUsize,
}

impl BufferPool {
    fn acquire(&self) -> BufferGuard {
        let buffer = self.free.pop()
            .unwrap_or_else(|| Box::new([0u8; 64 * 1024]));
        self.allocated.fetch_add(1, Ordering::Relaxed);
        BufferGuard { buffer, pool: self }
    }
}

struct BufferGuard<'a> {
    buffer: Box<[u8; 64 * 1024]>,
    pool: &'a BufferPool,
}

impl<'a> Drop for BufferGuard<'a> {
    fn drop(&mut self) {
        self.pool.allocated.fetch_sub(1, Ordering::Relaxed);
        self.pool.free.push(std::mem::take(&mut self.buffer));
    }
}
```

## Component Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                      CLI Interface                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Parser    │  │  Validator  │  │  Executor   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    MCP Protocol Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  JSON-RPC   │  │   Router    │  │   Handler   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Service Layer                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  Template   │  │     AST     │  │  Demo       │        │
│  │  Service    │  │  Analysis   │  │  Engine     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Deep Context│  │     Git     │  │ Repository  │        │
│  │  Analyzer   │  │  Analysis   │  │  Manager    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Storage Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  Embedded   │  │    Cache    │  │  Persistent │        │
│  │  Templates  │  │  Hierarchy  │  │    Cache    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

### Service Dependencies

```rust
// Dependency injection via constructor
pub struct StatelessTemplateServer {
    renderer: TemplateRenderer,
    cache: Arc<CacheHierarchy>,
    analyzer: Arc<AstAnalyzer>,
}

impl StatelessTemplateServer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            renderer: TemplateRenderer::new()?,
            cache: Arc::new(CacheHierarchy::new()),
            analyzer: Arc::new(AstAnalyzer::new()),
        })
    }
}
```

## Template System Architecture

### Template Storage

Embedded templates with metadata:

```rust
#[derive(Debug, Clone)]
pub struct TemplateResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub toolchain: Toolchain,
    pub category: Category,
    pub parameters: Vec<ParameterSpec>,
    pub content_key: String,
}

lazy_static! {
    static ref TEMPLATE_METADATA: HashMap<&'static str, TemplateResource> = {
        let mut map = HashMap::new();
        // Load metadata from embedded JSON
        for entry in EMBEDDED_METADATA {
            map.insert(entry.uri, entry.to_resource());
        }
        map
    };
}
```

### Parameter Validation

Type-safe parameter handling:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Boolean,
    Enum(Vec<String>),
}

pub struct ParameterValidator {
    specs: Vec<ParameterSpec>,
}

impl ParameterValidator {
    pub fn validate(&self, params: &Map<String, Value>) -> Result<(), ValidationError> {
        // Type checking
        for spec in &self.specs {
            if let Some(value) = params.get(&spec.name) {
                self.validate_type(&spec.param_type, value)?;
            } else if spec.required {
                return Err(ValidationError::MissingRequired(spec.name.clone()));
            }
        }
        Ok(())
    }
}
```

## AST Analysis Architecture

### Language-Specific Analyzers

```rust
pub trait AstAnalyzer: Send + Sync {
    async fn analyze_file(&self, path: &Path) -> Result<FileAnalysis>;
    fn supported_extensions(&self) -> &[&str];
}

pub struct RustAnalyzer {
    parser: syn::Parser,
    cache: Arc<AstCache>,
}

pub struct TypeScriptAnalyzer {
    parser: swc::Parser,
    cache: Arc<AstCache>,
}

pub struct PythonAnalyzer {
    parser: rustpython_parser::Parser,
    cache: Arc<AstCache>,
}
```

### Complexity Calculation

Visitor pattern for AST traversal:

```rust
pub struct ComplexityVisitor {
    cyclomatic: u16,
    cognitive: u16,
    nesting_depth: u16,
    max_nesting: u16,
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        self.cyclomatic += 1;
        self.cognitive += self.nesting_depth;
        self.nesting_depth += 1;
        self.max_nesting = self.max_nesting.max(self.nesting_depth);
        
        visit::visit_expr_if(self, node);
        
        self.nesting_depth -= 1;
    }
}
```

### File Ranking System

Generic ranking engine with pluggable metrics:

```rust
pub trait FileRanker: Send + Sync {
    type Metric: PartialOrd + Clone + Send + Sync;
    
    fn compute_score(&self, file_path: &Path) -> Self::Metric;
    fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String;
    fn ranking_type(&self) -> &'static str;
}

pub struct RankingEngine<R: FileRanker> {
    ranker: R,
    cache: Arc<RwLock<HashMap<String, R::Metric>>>,
}

impl<R: FileRanker> RankingEngine<R> {
    pub async fn rank_files(&self, files: &[PathBuf], limit: usize) -> Vec<(String, R::Metric)> {
        // Parallel computation with caching
        let scores: Vec<_> = files
            .par_iter()
            .filter_map(|f| self.compute_with_cache(f))
            .collect();
        
        // Sort and apply limit
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        scores.truncate(limit);
        scores
    }
}
```

#### Built-in Ranking Metrics

- **ComplexityRanker**: Composite scoring based on cyclomatic, cognitive, and function count
- **ChurnScore**: Git-based change frequency analysis
- **DuplicationScore**: Code clone detection metrics
- **Vectorized Ranking**: SIMD-optimized for large datasets (>1024 files)

## Unified Demo System Architecture

### Demo Engine Design

Multi-modal demonstration system with deep context integration:

```rust
pub struct DemoEngine {
    analyzer: Arc<DeepContextAnalyzer>,
    repository_manager: Arc<RepositoryManager>,
    graph_reducer: Arc<AdaptiveGraphReducer>,
    metrics_cache: Arc<Mutex<HashMap<String, DemoAnalysis>>>,
}

impl DemoEngine {
    pub async fn analyze(&self, source: DemoSource) -> Result<DemoAnalysis> {
        // 1. Repository acquisition (local or remote)
        let repo_path = self.repository_manager.acquire(source.clone()).await?;
        
        // 2. Parallel analysis pipeline using DeepContext
        let deep_context = self.analyzer.analyze_project(&repo_path).await?;
        
        // 3. Graph reduction and visualization prep
        let visualization = self.graph_reducer.reduce(&deep_context.analyses.dependency_graph).await?;
        
        // 4. Extract metrics and generate insights
        let metrics = AnalysisMetrics::from_deep_context(&deep_context);
        let insights = self.generate_insights(&deep_context).await;
        
        Ok(DemoAnalysis { repository, metrics, visualization, timings, insights })
    }
}
```

### Repository Management System

Git repository cloning and workspace management:

```rust
pub struct RepositoryManager {
    workspace: TempWorkspace,
}

impl RepositoryManager {
    pub async fn acquire(&self, source: DemoSource) -> Result<PathBuf> {
        match source {
            DemoSource::Local(path) => {
                if path.exists() { Ok(path) } else { Err(anyhow!("Path not found")) }
            }
            DemoSource::Remote(url) => self.clone_repository(&url).await,
            DemoSource::Cached(key) => self.workspace.get_cached(&key),
        }
    }

    async fn clone_repository(&self, url: &str) -> Result<PathBuf> {
        let temp_dir = self.workspace.create_temp()?;
        
        // Use git command with depth=1 optimization
        let output = std::process::Command::new("git")
            .args(["clone", "--depth", "1", url, &temp_dir.to_string_lossy()])
            .output()
            .context("Failed to execute git clone")?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git clone failed: {}", error));
        }
        
        Ok(temp_dir)
    }
}
```

### Adaptive Graph Reduction

Intelligent graph reduction for large codebase visualization:

```rust
pub struct AdaptiveGraphReducer;

impl AdaptiveGraphReducer {
    pub async fn reduce(&self, dag: &DependencyGraph) -> Result<VisualizationData> {
        let node_count = dag.nodes.len();
        info!("Reducing graph with {} nodes", node_count);
        
        // Simple reduction strategy with complexity scoring
        let mermaid = self.generate_mermaid(dag, node_count).await?;
        
        Ok(VisualizationData {
            mermaid,
            d3_json: None,
            complexity_map: self.build_complexity_map(dag),
            metrics: GraphMetrics {
                nodes: dag.nodes.len(),
                edges: dag.edges.len(),
                density: self.calculate_density(dag),
                modularity: 0.5, // Placeholder for future enhancement
            },
        })
    }
    
    fn calculate_density(&self, dag: &DependencyGraph) -> f64 {
        let n = dag.nodes.len() as f64;
        if n <= 1.0 { return 0.0; }
        
        let max_edges = n * (n - 1.0);
        let actual_edges = dag.edges.len() as f64;
        
        actual_edges / max_edges
    }
}
```

### Demo Analysis Data Model

Comprehensive analysis structure with insights:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoAnalysis {
    pub repository: RepositoryInfo,
    pub metrics: AnalysisMetrics,
    pub visualization: VisualizationData,
    pub timings: ExecutionTimings,
    pub insights: Vec<Insight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub complexity: Option<ComplexityReport>,
    pub churn: Option<CodeChurnAnalysis>,
    pub dag: Option<DependencyGraph>,
    pub ast_contexts: Vec<FileContext>,
    pub hotspots: Vec<CodeHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeHotspot {
    pub file: String,
    pub score: f64,
    pub complexity: u32,
    pub churn: u32,
    pub risk_factors: Vec<String>,
}
```

### Multi-Modal Interface Support

CLI, Web, MCP, and HTTP interface integration:

```rust
// CLI Demo Renderer with Unicode tables and ASCII diagrams
pub struct CliDemoRenderer {
    output: Box<dyn Write>,
}

impl CliDemoRenderer {
    pub fn render(&mut self, analysis: &DemoAnalysis) -> Result<()> {
        self.render_header(analysis)?;
        self.render_repository_info(analysis)?;
        self.render_quality_metrics(analysis)?;
        self.render_architecture_overview(analysis)?;
        self.render_insights(analysis)?;
        Ok(())
    }
    
    fn render_ascii_architecture(&mut self, mermaid: &str) -> Result<()> {
        let node_count = mermaid.matches(" --> ").count() + 1;
        
        if node_count > 10 {
            writeln!(self.output, "     ┌─────────┐")?;
            writeln!(self.output, "  ┌──│ Entry   │──┐")?;
            writeln!(self.output, "Complex architecture with {} modules", node_count)?;
        } else if node_count > 3 {
            writeln!(self.output, "┌─────┐    ┌─────┐    ┌─────┐")?;
            writeln!(self.output, "│  A  │───►│  B  │───►│  C  │")?;
            writeln!(self.output, "Modular architecture with {} components", node_count)?;
        } else {
            writeln!(self.output, "┌─────────────┐")?;
            writeln!(self.output, "│   Simple    │")?;
            writeln!(self.output, "Simple structure with {} components", node_count)?;
        }
        
        Ok(())
    }
}
```

### Insight Generation System

AI-powered insights with confidence scoring:

```rust
impl DemoEngine {
    async fn generate_insights(&self, context: &DeepContext) -> Vec<Insight> {
        let mut insights = Vec::new();

        // Architecture insights
        if let Some(ref dag) = context.analyses.dependency_graph {
            if dag.nodes.len() > 100 {
                insights.push(Insight {
                    category: InsightCategory::Architecture,
                    title: "Large Codebase Detected".to_string(),
                    description: format!("This codebase has {} modules. Consider modularization strategies.", dag.nodes.len()),
                    impact: InsightImpact::Medium,
                    confidence: 0.9,
                });
            }
        }

        // Quality insights based on scorecard
        if context.quality_scorecard.overall_health < 70.0 {
            insights.push(Insight {
                category: InsightCategory::Quality,
                title: "Code Quality Needs Attention".to_string(),
                description: format!("Overall health score is {:.1}%. Focus on complexity reduction and technical debt.", context.quality_scorecard.overall_health),
                impact: InsightImpact::High,
                confidence: 0.85,
            });
        }

        // Maintainability insights
        if context.quality_scorecard.technical_debt_hours > 40.0 {
            insights.push(Insight {
                category: InsightCategory::Maintainability,
                title: "High Technical Debt".to_string(),
                description: format!("Estimated {:.1} hours of technical debt. Prioritize SATD resolution.", context.quality_scorecard.technical_debt_hours),
                impact: InsightImpact::High,
                confidence: 0.8,
            });
        }

        insights
    }
}
```

### Progress Tracking System

Real-time progress updates across interfaces:

```rust
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub name: String,
    pub percent: f64,
    pub message: Option<String>,
}

impl ProgressEvent {
    pub fn started(name: &str) -> Self {
        Self {
            name: name.to_string(),
            percent: 0.0,
            message: Some(format!("Starting {}", name)),
        }
    }

    pub fn completed(name: &str, percent: f64) -> Self {
        Self {
            name: name.to_string(),
            percent,
            message: Some(format!("Completed {}", name)),
        }
    }
}

impl DemoEngine {
    pub async fn analyze_with_progress(
        &self,
        source: DemoSource,
        progress_tx: mpsc::Sender<ProgressEvent>,
    ) -> Result<DemoAnalysis> {
        progress_tx.send(ProgressEvent::started("Repository Discovery")).await?;
        
        let _repo_path = self.repository_manager.acquire(source.clone()).await?;
        progress_tx.send(ProgressEvent::completed("Repository Discovery", 10.0)).await?;
        
        progress_tx.send(ProgressEvent::started("AST Analysis")).await?;
        // Continue with analysis steps, sending progress updates
        
        self.analyze(source).await
    }
}
```

## Cache Architecture

### Hierarchical Cache Design

```rust
pub struct CacheHierarchy {
    // L1: Thread-local, lock-free
    l1: Arc<DashMap<CacheKey, CacheEntry>>,
    
    // L2: Process-wide, bounded
    l2: Arc<RwLock<LruCache<CacheKey, Arc<[u8]>>>>,
    
    // L3: Persistent, unbounded
    l3: Arc<PersistentCache>,
}

impl CacheHierarchy {
    pub async fn get(&self, key: &CacheKey) -> Option<Arc<[u8]>> {
        // Try L1 first (fastest)
        if let Some(entry) = self.l1.get(key) {
            return Some(entry.data.clone());
        }
        
        // Try L2 (fast)
        if let Some(data) = self.l2.read().await.peek(key) {
            self.l1.insert(key.clone(), CacheEntry::new(data.clone()));
            return Some(data.clone());
        }
        
        // Try L3 (slower but persistent)
        if let Some(data) = self.l3.get(key).await {
            self.promote_to_l2(key, &data).await;
            return Some(data);
        }
        
        None
    }
}
```

### Cache Key Design

Content-addressable with metadata:

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub category: CacheCategory,
    pub content_hash: [u8; 32],
    pub metadata_hash: [u8; 16],
}

impl CacheKey {
    pub fn from_content(category: CacheCategory, content: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(content);
        let content_hash = hasher.finalize().into();
        
        // Metadata hash for versioning
        let metadata = format!("{:?}:{}", category, env!("CARGO_PKG_VERSION"));
        let metadata_hash = blake3::hash(metadata.as_bytes()).as_bytes()[..16]
            .try_into()
            .unwrap();
        
        Self { category, content_hash, metadata_hash }
    }
}
```

## Error Handling Strategy

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    
    #[error("Parameter validation failed: {0}")]
    ValidationError(#[from] ValidationError),
    
    #[error("AST parsing failed: {0}")]
    AstError(#[from] AstError),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

// Automatic conversion to MCP errors
impl From<AppError> for McpError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::TemplateNotFound(_) => McpError {
                code: -32000,
                message: err.to_string(),
                data: None,
            },
            AppError::ValidationError(_) => McpError {
                code: -32002,
                message: err.to_string(),
                data: None,
            },
            _ => McpError {
                code: -32603,
                message: "Internal error".to_string(),
                data: Some(json!({ "details": err.to_string() })),
            },
        }
    }
}
```

## Security Architecture

### Input Sanitization

```rust
pub fn sanitize_template_uri(uri: &str) -> Result<String, SecurityError> {
    // Prevent directory traversal
    if uri.contains("..") || uri.contains("~") {
        return Err(SecurityError::PathTraversal);
    }
    
    // Validate URI format
    let re = regex::Regex::new(r"^template://[\w-]+/[\w-]+/[\w-]+$").unwrap();
    if !re.is_match(uri) {
        return Err(SecurityError::InvalidUri);
    }
    
    Ok(uri.to_string())
}
```

### Resource Limits

```rust
pub struct ResourceLimits {
    pub max_request_size: usize,      // 1MB
    pub max_response_size: usize,     // 10MB
    pub max_concurrent_requests: u32, // 128
    pub max_template_size: usize,     // 100KB
    pub max_cache_memory: usize,      // 256MB
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_request_size: 1024 * 1024,
            max_response_size: 10 * 1024 * 1024,
            max_concurrent_requests: 128,
            max_template_size: 100 * 1024,
            max_cache_memory: 256 * 1024 * 1024,
        }
    }
}
```

## Testing Architecture

### Property-Based Testing

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn template_rendering_never_panics(
            params in prop::collection::hash_map(
                ".*",
                prop_oneof![
                    Just(json!(true)),
                    Just(json!(false)),
                    any::<String>().prop_map(|s| json!(s)),
                    any::<i64>().prop_map(|n| json!(n)),
                ],
                0..10
            )
        ) {
            let server = StatelessTemplateServer::new().unwrap();
            let uri = "template://makefile/rust/cli";
            
            // Should never panic, only return errors
            let _ = server.generate_template(uri, params);
        }
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_mcp_protocol_flow() {
    let server = Arc::new(StatelessTemplateServer::new().unwrap());
    let (client_tx, server_rx) = mpsc::channel(100);
    let (server_tx, client_rx) = mpsc::channel(100);
    
    // Spawn server task
    tokio::spawn(async move {
        handle_mcp_connection(server, server_rx, server_tx).await
    });
    
    // Client flow
    client_tx.send(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    })).await.unwrap();
    
    let response = client_rx.recv().await.unwrap();
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
}