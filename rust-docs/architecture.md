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
│  │  Template   │  │     AST     │  │     Git     │        │
│  │  Service    │  │  Analysis   │  │  Analysis   │        │
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