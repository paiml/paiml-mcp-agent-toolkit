# TDD Implementation Guide for PMAT MCP/CLI/HTTP Interfaces

## CRITICAL: Uniform Contract Requirement

### Absolute Rule: One Contract, Three Interfaces

**Every command MUST have identical parameters across CLI, MCP, and HTTP interfaces.**

This is non-negotiable. A parameter that exists in one interface must:
1. Have the exact same name in all interfaces
2. Have the exact same type in all interfaces  
3. Have the exact same behavior in all interfaces
4. Have the exact same default values in all interfaces

### Contract Violations That Are FORBIDDEN:
- ❌ CLI has `--file` while MCP has `files` parameter
- ❌ CLI accepts multiple files via `--files` while MCP only accepts single file
- ❌ HTTP uses `source_code` while CLI uses `--source`
- ❌ CLI has `--max-complexity` while MCP has `complexity_threshold`
- ❌ Different default values across interfaces

### Current Contract Violations to Fix:
1. **Inconsistent file/path parameters**: Some commands use `--path`, others use `--project-path`, some use `--file`
2. **Inconsistent top_files parameter**: Some use `--top-files`, needs standardization
3. **Format parameter inconsistency**: Should be `--format` everywhere, not variations
4. **Timeout parameter missing**: Not all commands have timeout, but should for consistency

### Unified Command Contract Definitions

```rust
// src/contracts/mod.rs
// THIS IS THE SINGLE SOURCE OF TRUTH FOR ALL INTERFACES

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Base parameters shared by ALL analysis commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseAnalysisContract {
    /// Path to analyze - ALWAYS named 'path', never 'project_path' or 'file'
    pub path: PathBuf,
    
    /// Output format - ALWAYS available, ALWAYS same enum
    pub format: OutputFormat,
    
    /// Output file path - ALWAYS optional
    pub output: Option<PathBuf>,
    
    /// Number of top files to show - ALWAYS same name, ALWAYS optional
    pub top_files: Option<usize>,
    
    /// Include test files - ALWAYS same behavior
    pub include_tests: bool,
    
    /// Analysis timeout in seconds - ALWAYS available
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeComplexityContract {
    /// Base parameters (inherited)
    #[serde(flatten)]
    pub base: BaseAnalysisContract,
    
    /// Maximum cyclomatic complexity threshold
    pub max_cyclomatic: Option<u32>,
    
    /// Maximum cognitive complexity threshold  
    pub max_cognitive: Option<u32>,
    
    /// Maximum Halstead difficulty threshold
    pub max_halstead: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeSatdContract {
    /// Base parameters (inherited)
    #[serde(flatten)]
    pub base: BaseAnalysisContract,
    
    /// Filter by severity level
    pub severity: Option<SatdSeverity>,
    
    /// Show only critical debt items
    pub critical_only: bool,
    
    /// Use strict mode (only TODO/FIXME/HACK/BUG)
    pub strict: bool,
    
    /// Exit with error if violations found
    pub fail_on_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeDeadCodeContract {
    /// Base parameters (inherited)
    #[serde(flatten)]
    pub base: BaseAnalysisContract,
    
    /// Include unreachable code blocks
    pub include_unreachable: bool,
    
    /// Minimum dead lines to report a file
    pub min_dead_lines: usize,
    
    /// Maximum allowed dead code percentage
    pub max_percentage: f64,
    
    /// Exit with error if violations found
    pub fail_on_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeTdgContract {
    /// Base parameters (inherited)
    #[serde(flatten)]
    pub base: BaseAnalysisContract,
    
    /// TDG threshold for filtering results
    pub threshold: f64,
    
    /// Include TDG component breakdown
    pub include_components: bool,
    
    /// Show only critical files (TDG > 2.5)
    pub critical_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeLintHotspotContract {
    /// Base parameters (inherited)
    #[serde(flatten)]
    pub base: BaseAnalysisContract,
    
    /// Analyze a specific file instead of finding hotspot
    pub file: Option<PathBuf>,
    
    /// Maximum allowed defect density
    pub max_density: f64,
    
    /// Minimum confidence for automated fixes
    pub min_confidence: f64,
    
    /// Enforce quality standards
    pub enforce: bool,
    
    /// Dry run - show what would be fixed
    pub dry_run: bool,
}

// Contract validation that MUST be used by all interfaces
pub trait ContractValidation {
    fn validate(&self) -> Result<(), ContractError>;
}

impl ContractValidation for BaseAnalysisContract {
    fn validate(&self) -> Result<(), ContractError> {
        if !self.path.exists() {
            return Err(ContractError::PathNotFound(self.path.clone()));
        }
        
        if self.timeout == 0 {
            return Err(ContractError::InvalidTimeout);
        }
        
        if let Some(top_files) = self.top_files {
            if top_files > 1000 {
                return Err(ContractError::TooManyFiles(top_files));
            }
        }
        
        Ok(())
    }
}
```

### Contract Mapping Rules

```rust
// src/contracts/mapping.rs
// Maps between CLI args, MCP params, and HTTP bodies

use clap::Args;
use serde_json::Value;

/// CLI to Contract mapping
impl From<AnalyzeComplexityArgs> for AnalyzeComplexityContract {
    fn from(args: AnalyzeComplexityArgs) -> Self {
        Self {
            base: BaseAnalysisContract {
                path: args.path,
                format: args.format,
                output: args.output,
                top_files: args.top_files,
                include_tests: args.include_tests,
                timeout: args.timeout,
            },
            max_cyclomatic: args.max_cyclomatic,
            max_cognitive: args.max_cognitive,
            max_halstead: args.max_halstead,
        }
    }
}

/// MCP params to Contract mapping
impl TryFrom<Value> for AnalyzeComplexityContract {
    type Error = ContractError;
    
    fn try_from(params: Value) -> Result<Self, Self::Error> {
        // EXACT same parameter names as CLI
        Ok(Self {
            base: BaseAnalysisContract {
                path: params["path"].as_str()
                    .ok_or(ContractError::MissingParam("path"))?
                    .into(),
                format: params["format"].as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
                output: params["output"].as_str().map(PathBuf::from),
                top_files: params["top_files"].as_u64().map(|n| n as usize),
                include_tests: params["include_tests"].as_bool().unwrap_or(false),
                timeout: params["timeout"].as_u64().unwrap_or(60),
            },
            max_cyclomatic: params["max_cyclomatic"].as_u64().map(|n| n as u32),
            max_cognitive: params["max_cognitive"].as_u64().map(|n| n as u32),
            max_halstead: params["max_halstead"].as_f64(),
        })
    }
}
```

### Contract Enforcement Tests

```rust
// tests/contract_uniformity.rs
// These tests MUST pass for every command

#[test]
fn test_analyze_complexity_contract_uniformity() {
    let contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: PathBuf::from("."),
            format: OutputFormat::Json,
            output: Some(PathBuf::from("output.json")),
            top_files: Some(10),
            include_tests: false,
            timeout: 60,
        },
        max_cyclomatic: Some(20),
        max_cognitive: Some(15),
        max_halstead: Some(10.0),
    };
    
    // CLI must accept these exact parameters
    let cli_result = parse_cli_args(&[
        "analyze", "complexity",
        "--path", ".",
        "--format", "json",
        "--output", "output.json",
        "--top-files", "10",
        "--no-include-tests",
        "--timeout", "60",
        "--max-cyclomatic", "20",
        "--max-cognitive", "15",
        "--max-halstead", "10.0"
    ]);
    assert_eq!(cli_result.unwrap(), contract);
    
    // MCP must accept these exact parameters
    let mcp_params = json!({
        "path": ".",
        "format": "json",
        "output": "output.json",
        "top_files": 10,
        "include_tests": false,
        "timeout": 60,
        "max_cyclomatic": 20,
        "max_cognitive": 15,
        "max_halstead": 10.0
    });
    let mcp_result = parse_mcp_params("analyze_complexity", mcp_params);
    assert_eq!(mcp_result.unwrap(), contract);
    
    // HTTP must accept these exact parameters
    let http_body = json!({
        "path": ".",
        "format": "json",
        "output": "output.json",
        "top_files": 10,
        "include_tests": false,
        "timeout": 60,
        "max_cyclomatic": 20,
        "max_cognitive": 15,
        "max_halstead": 10.0
    });
    let http_result = parse_http_request("/analyze/complexity", http_body);
    assert_eq!(http_result.unwrap(), contract);
}

#[test]
fn test_all_commands_have_uniform_contracts() {
    // This test ensures every command follows the uniform contract pattern
    let commands = vec![
        "analyze_complexity",
        "analyze_satd", 
        "analyze_dead_code",
        "analyze_tdg",
        "analyze_lint_hotspot",
        "analyze_duplicates",
        "analyze_churn",
        "quality_gate",
        "refactor_auto",
    ];
    
    for command in commands {
        // Verify CLI command exists with exact parameter names
        assert!(cli_has_command(command));
        
        // Verify MCP tool exists with exact parameter names  
        assert!(mcp_has_tool(command));
        
        // Verify HTTP endpoint exists with exact parameter names
        assert!(http_has_endpoint(command));
        
        // Verify all three use the same contract type
        assert_contracts_match(command);
    }
}
```

## Core Architecture

### 1. Protocol Trait Definition

```rust
// src/protocol/mod.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;

#[async_trait]
pub trait ProtocolHandler: Send + Sync + 'static {
    type Input: for<'de> Deserialize<'de> + Send;
    type Output: Serialize + Send;
    type Error: StdError + Send + Sync;
    
    async fn decode(&self, raw: &[u8]) -> Result<Self::Input, Self::Error>;
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
    async fn encode(&self, output: Self::Output) -> Result<Vec<u8>, Self::Error>;
    
    // Quality gate enforcement
    async fn validate_input(&self, input: &Self::Input) -> Result<(), Self::Error> {
        Ok(())
    }
    
    async fn validate_output(&self, output: &Self::Output) -> Result<(), Self::Error> {
        Ok(())
    }
}

// Common request/response types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedRequest {
    pub method: String,
    pub params: serde_json::Value,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<ErrorInfo>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

### 2. Test-First Core Implementation

```rust
// src/core/analyzer.rs
// Start with pure business logic, no protocol concerns

#[derive(Debug, Clone, PartialEq)]
pub struct ComplexityResult {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub lines: u32,
    pub functions: Vec<FunctionMetrics>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionMetrics {
    pub name: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub line_start: u32,
    pub line_end: u32,
}

// Pure function - easily testable
pub fn analyze_complexity(source: &str) -> Result<ComplexityResult, AnalysisError> {
    // Implementation here
    Ok(ComplexityResult {
        cyclomatic: 1,
        cognitive: 0,
        lines: source.lines().count() as u32,
        functions: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_source() {
        let result = analyze_complexity("").unwrap();
        assert_eq!(result.cyclomatic, 1);
        assert_eq!(result.lines, 0);
    }
    
    #[test]
    fn test_simple_function() {
        let source = r#"
            fn main() {
                println!("Hello");
            }
        "#;
        let result = analyze_complexity(source).unwrap();
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "main");
    }
    
    #[test]
    fn test_complex_branching() {
        let source = r#"
            fn process(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 { 20 } else { 10 }
                } else {
                    0
                }
            }
        "#;
        let result = analyze_complexity(source).unwrap();
        assert!(result.cyclomatic > 3);
    }
}
```

### 3. MCP Protocol Implementation

```rust
// src/protocol/mcp.rs
use super::{ProtocolHandler, UnifiedRequest, UnifiedResponse};
use async_trait::async_trait;
use pmcp::{Server, ToolHandler, ToolResult};

pub struct McpHandler {
    server: Arc<Server>,
    analyzer: Arc<dyn Analyzer>,
}

impl McpHandler {
    pub fn new(analyzer: Arc<dyn Analyzer>) -> Self {
        let server = Server::builder()
            .name("pmat")
            .version(env!("CARGO_PKG_VERSION"))
            .build();
            
        Self { server, analyzer }
    }
}

#[async_trait]
impl ProtocolHandler for McpHandler {
    type Input = UnifiedRequest;
    type Output = UnifiedResponse;
    type Error = McpError;
    
    async fn decode(&self, raw: &[u8]) -> Result<Self::Input, Self::Error> {
        let json: serde_json::Value = serde_json::from_slice(raw)?;
        Ok(UnifiedRequest {
            method: json["method"].as_str().unwrap_or("").to_string(),
            params: json["params"].clone(),
            id: json["id"].as_str().map(String::from),
        })
    }
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        match input.method.as_str() {
            "analyze_complexity" => {
                let source = input.params["source"].as_str()
                    .ok_or_else(|| McpError::InvalidParam("source"))?;
                let result = self.analyzer.analyze_complexity(source).await?;
                Ok(UnifiedResponse {
                    result: Some(serde_json::to_value(result)?),
                    error: None,
                    id: input.id,
                })
            }
            _ => Err(McpError::MethodNotFound(input.method)),
        }
    }
    
    async fn encode(&self, output: Self::Output) -> Result<Vec<u8>, Self::Error> {
        Ok(serde_json::to_vec(&output)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_decode() {
        let handler = McpHandler::new(Arc::new(MockAnalyzer));
        let raw = r#"{"method": "analyze_complexity", "params": {"source": "fn main() {}"}, "id": "1"}"#;
        let decoded = handler.decode(raw.as_bytes()).await.unwrap();
        assert_eq!(decoded.method, "analyze_complexity");
    }
    
    #[tokio::test]
    async fn test_mcp_roundtrip() {
        let handler = McpHandler::new(Arc::new(MockAnalyzer));
        let request = UnifiedRequest {
            method: "analyze_complexity".to_string(),
            params: json!({"source": "fn test() {}"}),
            id: Some("123".to_string()),
        };
        
        let response = handler.process(request.clone()).await.unwrap();
        assert!(response.result.is_some());
        assert_eq!(response.id, Some("123".to_string()));
    }
}
```

### 4. CLI Protocol Implementation

```rust
// src/protocol/cli.rs
use super::{ProtocolHandler, UnifiedRequest, UnifiedResponse};
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, default_value = "json")]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Commands {
    Analyze {
        #[arg(short, long)]
        file: PathBuf,
        
        #[arg(long)]
        complexity: bool,
        
        #[arg(long)]
        satd: bool,
    },
    Context {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

pub struct CliHandler {
    analyzer: Arc<dyn Analyzer>,
}

#[async_trait]
impl ProtocolHandler for CliHandler {
    type Input = CliArgs;
    type Output = UnifiedResponse;
    type Error = CliError;
    
    async fn decode(&self, raw: &[u8]) -> Result<Self::Input, Self::Error> {
        // In CLI mode, raw is typically from argv
        let args_str = std::str::from_utf8(raw)?;
        let args: Vec<String> = shell_words::split(args_str)?;
        Ok(CliArgs::parse_from(args))
    }
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        match input.command {
            Commands::Analyze { file, complexity, .. } => {
                let source = tokio::fs::read_to_string(file).await?;
                if complexity {
                    let result = self.analyzer.analyze_complexity(&source).await?;
                    Ok(UnifiedResponse {
                        result: Some(serde_json::to_value(result)?),
                        error: None,
                        id: None,
                    })
                } else {
                    Err(CliError::NoAnalysisSelected)
                }
            }
            Commands::Context { path } => {
                let context = self.analyzer.generate_context(path).await?;
                Ok(UnifiedResponse {
                    result: Some(serde_json::to_value(context)?),
                    error: None,
                    id: None,
                })
            }
        }
    }
    
    async fn encode(&self, output: Self::Output) -> Result<Vec<u8>, Self::Error> {
        let formatted = match self.format {
            OutputFormat::Json => serde_json::to_string_pretty(&output)?,
            OutputFormat::Yaml => serde_yaml::to_string(&output)?,
            OutputFormat::Table => format_as_table(&output),
        };
        Ok(formatted.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cli_parse() {
        let handler = CliHandler::new(Arc::new(MockAnalyzer));
        let raw = "analyze --file test.rs --complexity";
        let args = handler.decode(raw.as_bytes()).await.unwrap();
        match args.command {
            Commands::Analyze { complexity, .. } => assert!(complexity),
            _ => panic!("Wrong command parsed"),
        }
    }
}
```

### 5. HTTP Protocol Implementation

```rust
// src/protocol/http.rs
use axum::{Router, Json, extract::State};
use super::{ProtocolHandler, UnifiedRequest, UnifiedResponse};

pub struct HttpHandler {
    analyzer: Arc<dyn Analyzer>,
}

impl HttpHandler {
    pub fn router(analyzer: Arc<dyn Analyzer>) -> Router {
        Router::new()
            .route("/analyze", post(handle_analyze))
            .route("/context", post(handle_context))
            .route("/health", get(health_check))
            .with_state(analyzer)
    }
}

async fn handle_analyze(
    State(analyzer): State<Arc<dyn Analyzer>>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<UnifiedResponse>, HttpError> {
    let result = analyzer.analyze_complexity(&req.source).await?;
    Ok(Json(UnifiedResponse {
        result: Some(serde_json::to_value(result)?),
        error: None,
        id: req.id,
    }))
}

#[async_trait]
impl ProtocolHandler for HttpHandler {
    type Input = HttpRequest;
    type Output = HttpResponse;
    type Error = HttpError;
    
    async fn decode(&self, raw: &[u8]) -> Result<Self::Input, Self::Error> {
        // Parse HTTP request from raw bytes
        let request = hyper::Request::builder()
            .body(hyper::Body::from(raw.to_vec()))?;
        Ok(HttpRequest::from(request))
    }
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        match input.path.as_str() {
            "/analyze" => {
                let body: AnalyzeRequest = serde_json::from_slice(&input.body)?;
                let result = self.analyzer.analyze_complexity(&body.source).await?;
                Ok(HttpResponse {
                    status: 200,
                    body: serde_json::to_vec(&result)?,
                })
            }
            _ => Ok(HttpResponse {
                status: 404,
                body: b"Not Found".to_vec(),
            })
        }
    }
    
    async fn encode(&self, output: Self::Output) -> Result<Vec<u8>, Self::Error> {
        // Build HTTP response
        let response = format!(
            "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n",
            output.status,
            output.body.len()
        );
        Ok([response.as_bytes(), &output.body].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    
    #[tokio::test]
    async fn test_http_analyze_endpoint() {
        let app = HttpHandler::router(Arc::new(MockAnalyzer));
        let server = TestServer::new(app).unwrap();
        
        let response = server
            .post("/analyze")
            .json(&json!({"source": "fn main() {}"}))
            .await;
            
        assert_eq!(response.status_code(), 200);
        let body: UnifiedResponse = response.json();
        assert!(body.result.is_some());
    }
}
```

### 6. Unified Server Implementation

```rust
// src/server.rs
use tokio::sync::RwLock;

pub struct UnifiedServer {
    mcp: Arc<McpHandler>,
    cli: Arc<CliHandler>,
    http: Arc<HttpHandler>,
    state: Arc<RwLock<ServerState>>,
    config: ServerConfig,
}

pub struct ServerConfig {
    pub mcp_enabled: bool,
    pub cli_enabled: bool,
    pub http_enabled: bool,
    pub http_port: u16,
    pub quality_gates: QualityConfig,
}

impl UnifiedServer {
    pub fn new(config: ServerConfig) -> Self {
        let analyzer = Arc::new(CoreAnalyzer::new());
        
        Self {
            mcp: Arc::new(McpHandler::new(analyzer.clone())),
            cli: Arc::new(CliHandler::new(analyzer.clone())),
            http: Arc::new(HttpHandler::new(analyzer.clone())),
            state: Arc::new(RwLock::new(ServerState::default())),
            config,
        }
    }
    
    pub async fn run(self) -> Result<()> {
        let mut tasks = vec![];
        
        if self.config.mcp_enabled {
            tasks.push(tokio::spawn(self.clone().run_mcp()));
        }
        
        if self.config.http_enabled {
            tasks.push(tokio::spawn(self.clone().run_http()));
        }
        
        if self.config.cli_enabled {
            tasks.push(tokio::spawn(self.clone().run_cli()));
        }
        
        // Wait for all protocols
        futures::future::try_join_all(tasks).await?;
        Ok(())
    }
    
    async fn run_mcp(self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        
        loop {
            let mut buffer = vec![0; 4096];
            let n = stdin.read(&mut buffer).await?;
            
            if n == 0 {
                break; // EOF
            }
            
            let request = self.mcp.decode(&buffer[..n]).await?;
            
            // Apply quality gates
            self.enforce_quality_gates(&request).await?;
            
            let response = self.mcp.process(request).await?;
            let encoded = self.mcp.encode(response).await?;
            
            stdout.write_all(&encoded).await?;
            stdout.flush().await?;
        }
        
        Ok(())
    }
    
    async fn enforce_quality_gates(&self, request: &UnifiedRequest) -> Result<()> {
        if !self.config.quality_gates.enabled {
            return Ok(());
        }
        
        // Check complexity thresholds
        if let Some(source) = request.params.get("source").and_then(|s| s.as_str()) {
            let result = analyze_complexity(source)?;
            
            if result.cyclomatic > self.config.quality_gates.max_complexity {
                return Err(QualityError::ComplexityExceeded {
                    actual: result.cyclomatic,
                    threshold: self.config.quality_gates.max_complexity,
                });
            }
        }
        
        Ok(())
    }
}
```

### 7. Property-Based Testing

```rust
// src/tests/property.rs
use proptest::prelude::*;

// Generate arbitrary valid requests
impl Arbitrary for UnifiedRequest {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            prop::string::string_regex("[a-z_]+").unwrap(),
            any::<serde_json::Value>(),
            prop::option::of(any::<String>()),
        )
            .prop_map(|(method, params, id)| UnifiedRequest {
                method,
                params,
                id,
            })
            .boxed()
    }
}

proptest! {
    #[test]
    fn test_protocol_consistency(req in any::<UnifiedRequest>()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let analyzer = Arc::new(MockAnalyzer);
            
            let mcp = McpHandler::new(analyzer.clone());
            let cli = CliHandler::new(analyzer.clone());
            let http = HttpHandler::new(analyzer.clone());
            
            // All protocols should handle the same request identically
            let mcp_result = mcp.process(req.clone()).await;
            let cli_result = cli.process(req.clone()).await;
            let http_result = http.process(req.clone()).await;
            
            // Either all succeed or all fail
            match (mcp_result, cli_result, http_result) {
                (Ok(m), Ok(c), Ok(h)) => {
                    prop_assert_eq!(m.result, c.result);
                    prop_assert_eq!(c.result, h.result);
                }
                (Err(_), Err(_), Err(_)) => {
                    // All failed - this is consistent
                }
                _ => {
                    prop_assert!(false, "Inconsistent protocol behavior");
                }
            }
        });
    }
    
    #[test]
    fn test_encoding_roundtrip(response in any::<UnifiedResponse>()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let handler = McpHandler::new(Arc::new(MockAnalyzer));
            
            let encoded = handler.encode(response.clone()).await.unwrap();
            let decoded_req = handler.decode(&encoded).await;
            
            // Should either roundtrip successfully or fail consistently
            if let Ok(req) = decoded_req {
                let re_response = handler.process(req).await;
                prop_assert!(re_response.is_ok());
            }
        });
    }
}
```

### 8. Integration Testing

```rust
// src/tests/integration.rs

#[tokio::test]
async fn test_all_protocols_same_result() {
    let source_code = r#"
        fn complex_function(x: i32) -> i32 {
            if x > 0 {
                match x {
                    1..=10 => x * 2,
                    11..=20 => x * 3,
                    _ => x * 4,
                }
            } else {
                -x
            }
        }
    "#;
    
    let analyzer = Arc::new(CoreAnalyzer::new());
    
    // Test via MCP
    let mcp_handler = McpHandler::new(analyzer.clone());
    let mcp_req = UnifiedRequest {
        method: "analyze_complexity".to_string(),
        params: json!({"source": source_code}),
        id: Some("test".to_string()),
    };
    let mcp_response = mcp_handler.process(mcp_req).await.unwrap();
    
    // Test via CLI
    let cli_handler = CliHandler::new(analyzer.clone());
    // Create temp file for CLI
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), source_code).unwrap();
    let cli_args = CliArgs::parse_from(vec![
        "pmat",
        "analyze",
        "--file",
        temp_file.path().to_str().unwrap(),
        "--complexity",
    ]);
    let cli_response = cli_handler.process(cli_args).await.unwrap();
    
    // Test via HTTP
    let app = HttpHandler::router(analyzer.clone());
    let server = TestServer::new(app).unwrap();
    let http_response = server
        .post("/analyze")
        .json(&json!({"source": source_code}))
        .await;
    let http_body: UnifiedResponse = http_response.json();
    
    // All should produce identical results
    assert_eq!(mcp_response.result, cli_response.result);
    assert_eq!(cli_response.result, http_body.result);
    
    // Verify the actual metrics
    let result = mcp_response.result.unwrap();
    let complexity = result["cyclomatic"].as_u64().unwrap();
    assert!(complexity >= 5); // Should detect the branching complexity
}

#[tokio::test]
async fn test_quality_gate_enforcement() {
    let config = ServerConfig {
        mcp_enabled: true,
        cli_enabled: false,
        http_enabled: false,
        quality_gates: QualityConfig {
            enabled: true,
            max_complexity: 5,
            max_cognitive: 10,
            allow_satd: false,
        },
    };
    
    let server = UnifiedServer::new(config);
    
    // Code that violates complexity threshold
    let complex_code = r#"
        fn too_complex(x: i32) -> i32 {
            if x > 0 {
                if x > 10 {
                    if x > 20 {
                        if x > 30 {
                            if x > 40 {
                                50
                            } else { 40 }
                        } else { 30 }
                    } else { 20 }
                } else { 10 }
            } else { 0 }
        }
    "#;
    
    let request = UnifiedRequest {
        method: "analyze_complexity".to_string(),
        params: json!({"source": complex_code}),
        id: None,
    };
    
    let result = server.enforce_quality_gates(&request).await;
    assert!(result.is_err());
    
    match result {
        Err(QualityError::ComplexityExceeded { actual, threshold }) => {
            assert!(actual > threshold);
        }
        _ => panic!("Expected ComplexityExceeded error"),
    }
}

#[tokio::test]
async fn test_concurrent_protocol_handling() {
    let server = Arc::new(UnifiedServer::new(ServerConfig::default()));
    
    let mut handles = vec![];
    
    // Spawn 100 concurrent requests across all protocols
    for i in 0..100 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let source = format!("fn test_{}() {{}}", i);
            let request = UnifiedRequest {
                method: "analyze_complexity".to_string(),
                params: json!({"source": source}),
                id: Some(format!("{}", i)),
            };
            
            // Randomly choose a protocol
            match i % 3 {
                0 => server_clone.mcp.process(request).await,
                1 => server_clone.cli.process(/* cli equivalent */).await,
                2 => server_clone.http.process(/* http equivalent */).await,
                _ => unreachable!(),
            }
        });
        handles.push(handle);
    }
    
    // All should complete successfully
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
```

### 9. Makefile Integration

```makefile
# Test execution strategy
.PHONY: test test-unit test-integration test-property test-protocols

# Fast unit tests - no I/O, pure logic
test-unit:
	@echo "Running unit tests..."
	@cargo test --lib --features mock -- --test-threads=4

# Protocol-specific tests
test-mcp:
	@cargo test --test mcp_protocol -- --test-threads=1

test-cli:
	@cargo test --test cli_protocol -- --test-threads=1

test-http:
	@cargo test --test http_protocol -- --test-threads=1

test-protocols: test-mcp test-cli test-http
	@echo "✅ All protocol tests passed"

# Property-based testing
test-property:
	@echo "Running property tests..."
	@cargo test --features proptest -- --test-threads=2 --nocapture

# Integration tests - all protocols together
test-integration:
	@echo "Running integration tests..."
	@cargo test --test integration -- --test-threads=1

# Full test suite
test: test-unit test-protocols test-property test-integration
	@echo "✅ All tests passed"

# TDD workflow helper
tdd:
	@cargo watch -x "test --lib" -x "clippy -- -D warnings"

# Quality gates
quality-gate:
	@echo "Enforcing quality gates..."
	@cargo clippy -- -D warnings -W clippy::all -W clippy::pedantic
	@cargo test --all-features
	@./scripts/check-complexity.sh
	@echo "✅ Quality gates passed"
```

### 10. Continuous Testing Script

```bash
#!/bin/bash
# scripts/continuous-test.sh

set -e

echo "Starting continuous TDD cycle..."

# Watch for changes and run appropriate tests
while true; do
    inotifywait -r -e modify,create,delete src/ tests/ 2>/dev/null
    
    clear
    echo "Changes detected, running tests..."
    
    # Determine what changed
    CHANGED=$(git diff --name-only)
    
    if echo "$CHANGED" | grep -q "protocol/mcp"; then
        echo "MCP changes detected"
        cargo test --test mcp_protocol
    elif echo "$CHANGED" | grep -q "protocol/cli"; then
        echo "CLI changes detected"
        cargo test --test cli_protocol
    elif echo "$CHANGED" | grep -q "protocol/http"; then
        echo "HTTP changes detected"
        cargo test --test http_protocol
    else
        echo "Core changes detected"
        cargo test --lib
    fi
    
    # Always run quality checks
    cargo clippy -- -D warnings
    
    echo "✅ Tests passed, waiting for changes..."
done
```

## Implementation Checklist

- [ ] Define core `ProtocolHandler` trait
- [ ] Implement pure business logic with unit tests
- [ ] Add property-based tests for invariants
- [ ] Implement MCP handler with tests
- [ ] Implement CLI handler with tests
- [ ] Implement HTTP handler with tests
- [ ] Create unified server orchestration
- [ ] Add integration tests across all protocols
- [ ] Implement quality gate enforcement
- [ ] Add concurrent testing scenarios
- [ ] Create continuous testing infrastructure
- [ ] Document protocol parity requirements
- [ ] Add performance benchmarks
- [ ] Implement error handling consistency
- [ ] Add observability and metrics

## Quality Standards

### Mandatory Requirements
- Zero SATD comments
- Maximum cyclomatic complexity: 10
- Minimum test coverage: 80%
- All protocols produce identical results
- Property tests for all invariants
- Quality gates enforced before processing

### Testing Pyramid
- **Unit Tests (70%)**: Pure functions, no I/O
- **Integration Tests (20%)**: Protocol interactions
- **E2E Tests (10%)**: Full system validation

## Conclusion

This implementation provides a robust, test-driven foundation for unified MCP/CLI/HTTP interfaces. The architecture ensures protocol parity through shared core logic while maintaining clean separation of concerns. The comprehensive test suite validates both individual protocol behavior and system-wide consistency.