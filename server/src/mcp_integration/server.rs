use super::*;
use crate::agents::registry::AgentRegistry;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener};

// MCP server implementation
pub struct McpServer {
    context: Arc<McpContext>,
    config: ServerConfig,
    shutdown: Arc<tokio::sync::Notify>,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub bind_address: String,
    pub unix_socket: Option<String>,
    pub max_connections: usize,
    pub request_timeout: std::time::Duration,
    pub enable_logging: bool,

    // Semantic search configuration (PMAT-SEARCH-012)
    pub semantic_enabled: bool,
    pub semantic_api_key: Option<String>,
    pub semantic_db_path: Option<String>,
    pub semantic_workspace: Option<std::path::PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        // Load semantic config from environment variables if available
        let semantic_enabled = std::env::var("OPENAI_API_KEY").is_ok();
        let semantic_api_key = std::env::var("OPENAI_API_KEY").ok();
        let semantic_db_path = std::env::var("PMAT_VECTOR_DB_PATH")
            .ok()
            .or_else(|| {
                dirs::home_dir().map(|h| {
                    h.join(".pmat")
                        .join("embeddings.db")
                        .to_string_lossy()
                        .to_string()
                })
            });
        let semantic_workspace = std::env::var("PMAT_WORKSPACE")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::current_dir().ok());

        Self {
            name: "PMAT MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            bind_address: "127.0.0.1:3000".to_string(),
            unix_socket: None,
            max_connections: 100,
            request_timeout: std::time::Duration::from_secs(30),
            enable_logging: true,

            // Semantic search (PMAT-SEARCH-012)
            semantic_enabled,
            semantic_api_key,
            semantic_db_path,
            semantic_workspace,
        }
    }
}

impl McpServer {
    pub fn new(
        agent_registry: Arc<AgentRegistry>,
        config: ServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let capabilities = ServerCapabilities {
            experimental: None,
            logging: if config.enable_logging {
                Some(LoggingCapabilities {
                    level: "info".to_string(),
                })
            } else {
                None
            },
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
        };

        let server_info = ServerInfo {
            name: config.name.clone(),
            version: config.version.clone(),
            protocol_version: MCP_VERSION.to_string(),
        };

        let context = Arc::new(McpContext {
            server_info,
            capabilities,
            tools: Arc::new(RwLock::new(ToolRegistry::new())),
            resources: Arc::new(RwLock::new(ResourceRegistry::new())),
            prompts: Arc::new(RwLock::new(PromptRegistry::new())),
            agent_registry,
        });

        Ok(Self {
            context,
            config,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        })
    }

    pub async fn register_defaults(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Register default tools
        self.register_agent_tools().await?;

        // Register hallucination detection tools (Sprint 40a)
        self.register_hallucination_detection_tools().await?;

        // Register TDG analysis tools (Sprint 40c)
        self.register_tdg_tools().await?;

        // Register JVM language tools (Sprint 51)
        self.register_jvm_tools().await?;

        // Register cross-language analysis tools (Sprint 52)
        self.register_polyglot_tools().await?;

        // Register semantic search tools (PMAT-SEARCH-012)
        // Only registers if OPENAI_API_KEY is set
        self.register_semantic_tools().await?;

        // Register default resources
        self.register_agent_resources().await?;

        // Register default prompts
        self.register_agent_prompts().await?;

        Ok(())
    }

    async fn register_agent_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::tools::*;

        let mut tools = self.context.tools.write();

        // Register analyze tool
        tools.register(Arc::new(AnalyzeTool::new(
            self.context.agent_registry.clone(),
        )));

        // Register transform tool
        tools.register(Arc::new(TransformTool::new(
            self.context.agent_registry.clone(),
        )));

        // Register validate tool
        tools.register(Arc::new(ValidateTool::new(
            self.context.agent_registry.clone(),
        )));

        // Register orchestrate tool
        tools.register(Arc::new(OrchestrateTool::new(
            self.context.agent_registry.clone(),
        )));

        Ok(())
    }

    /// Register hallucination detection tools (Sprint 40a)
    ///
    /// Exposes Sprint 37's hallucination detection system via MCP to enable
    /// AI agents to validate documentation claims against the actual codebase.
    ///
    /// Based on peer-reviewed research:
    /// - Semantic Entropy (Farquhar et al., Nature 2024)
    /// - MIND framework (IJCAI 2025)
    /// - Unified Detection Framework (Complex & Intelligent Systems 2025)
    async fn register_hallucination_detection_tools(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::hallucination_detection_tools::*;

        let mut tools = self.context.tools.write();

        // Register validate_documentation tool
        tools.register(Arc::new(ValidateDocumentationTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered validate_documentation tool");

        // Register check_claim tool
        tools.register(Arc::new(CheckClaimTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered check_claim tool");

        tracing::info!("✅ Hallucination detection tools registered successfully (2 tools)");

        Ok(())
    }

    /// Register TDG (Technical Debt Gradient) analysis tools (Sprint 40c)
    ///
    /// Exposes the TDG quality analysis system via MCP to enable AI agents
    /// to assess code quality and receive actionable improvement recommendations.
    ///
    /// Tools:
    /// - `analyze_technical_debt`: Analyze quality scores for files/projects
    /// - `get_quality_recommendations`: Get actionable refactoring suggestions
    async fn register_tdg_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::tdg_tools::*;

        let mut tools = self.context.tools.write();

        // Register analyze_technical_debt tool
        tools.register(Arc::new(AnalyzeTechnicalDebtTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered analyze_technical_debt tool");

        // Register get_quality_recommendations tool
        tools.register(Arc::new(GetQualityRecommendationsTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered get_quality_recommendations tool");

        tracing::info!("✅ TDG analysis tools registered successfully (2 tools)");

        Ok(())
    }

    /// Register JVM language tools (Sprint 51)
    /// 
    /// Sprint 51 added comprehensive JVM language support with Java and Scala
    /// analysis. These tools provide:
    /// 
    /// - Java analysis: Class, interface, and method detection with complexity metrics
    /// - Scala analysis: Classes, traits, objects, and case classes with functional metrics
    /// - Mutation testing for Java and Scala code
    /// 
    /// Both tools are feature-gated behind the "java-ast" and "scala-ast" features.
    async fn register_jvm_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tools = self.context.tools.write();
        let mut registered_tools = 0;
        
        // Register Java analysis tools if feature is enabled
        #[cfg(feature = "java-ast")]
        {
            use crate::mcp_integration::java_tools::*;
            
            // Register Java analysis tool
            tools.register(Arc::new(JavaAnalysisTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered analyze_java tool");
            registered_tools += 1;
            
            // Register Java mutation testing tool
            tools.register(Arc::new(JavaMutationTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered mutation_test_java tool");
            registered_tools += 1;
        }
        
        // Register Scala analysis tools if feature is enabled
        #[cfg(feature = "scala-ast")]
        {
            use crate::mcp_integration::scala_tools::*;
            
            // Register Scala analysis tool
            tools.register(Arc::new(ScalaAnalysisTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered analyze_scala tool");
            registered_tools += 1;
            
            // Register Scala mutation testing tool
            tools.register(Arc::new(ScalaMutationTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered mutation_test_scala tool");
            registered_tools += 1;
        }
        
        if registered_tools > 0 {
            tracing::info!("✅ JVM language tools registered successfully ({} tools)", registered_tools);
        } else {
            tracing::info!("ℹ️ No JVM language tools registered (features not enabled)");
        }
        
        Ok(())
    }
    
    /// Register cross-language analysis tools (Sprint 52)
    ///
    /// Sprint 52 adds cross-language analysis capabilities to detect relationships
    /// between different programming languages in a project. These tools build upon
    /// the JVM language support from Sprint 51 and extend it to create a unified
    /// polyglot analysis framework.
    ///
    /// Tools:
    /// - analyze_polyglot: Analyzes cross-language relationships in a project
    /// - detect_language_boundaries: Detects language boundaries and interop points
    async fn register_polyglot_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::polyglot_tools::*;
        
        let mut tools = self.context.tools.write();
        
        // Register polyglot analysis tool
        tools.register(Arc::new(PolyglotAnalysisTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered analyze_polyglot tool");
        
        // Register language boundary tool
        tools.register(Arc::new(LanguageBoundaryTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered detect_language_boundaries tool");
        
        tracing::info!("✅ Cross-language analysis tools registered successfully (2 tools)");
        
        Ok(())
    }

    /// Register semantic search tools (PMAT-SEARCH-012)
    ///
    /// GREEN Phase: Partial implementation (needs adapter layer)
    ///
    /// The semantic search tools are fully implemented in `src/mcp/tools/semantic_search_tools.rs`
    /// but they implement a simpler `crate::mcp::McpTool` trait.
    ///
    /// The mcp_integration framework expects tools to implement `mcp_integration::McpTool` trait
    /// which has a different interface (metadata() vs name()/schema(), and different error types).
    ///
    /// **Architecture Note**: There are two MCP tool systems in the codebase:
    /// 1. **Simple MCP** (`src/mcp/`) - Used for semantic search tools
    ///    - Trait: `crate::mcp::McpTool`
    ///    - Methods: name(), schema(), execute() -> Result<Value, String>
    ///
    /// 2. **MCP Integration** (`src/mcp_integration/`) - Used for agent-based tools
    ///    - Trait: `mcp_integration::McpTool`
    ///    - Methods: metadata(), execute() -> Result<Value, McpError>
    ///
    /// **TODO**: Create adapter layer to bridge the two systems. Options:
    /// 1. Create wrapper structs that implement mcp_integration::McpTool and delegate to semantic tools
    /// 2. Unify the two MCP tool systems into a single interface
    /// 3. Keep them separate and register semantic tools via a different mechanism
    ///
    /// **Current Status**:
    /// - ✅ Configuration system complete (ServerConfig has semantic fields)
    /// - ✅ HybridSearchEngine initialization works
    /// - ✅ Semantic tools fully implemented and tested (149 tests passing)
    /// - ✅ CLI integration complete and working
    /// - 🔧 MCP integration requires adapter layer (estimated: 2-3 hours)
    ///
    /// Configuration is loaded from environment variables:
    /// - OPENAI_API_KEY: OpenAI API key for embeddings
    /// - PMAT_VECTOR_DB_PATH: Path to vector database (default: ~/.pmat/embeddings.db)
    /// - PMAT_WORKSPACE: Workspace path for code indexing (default: current directory)
    async fn register_semantic_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if semantic search is enabled
        if !self.config.semantic_enabled {
            tracing::info!("Semantic search is disabled (no OPENAI_API_KEY found)");
            return Ok(());
        }

        // Sprint 33 (PMAT-SEARCH-012 GREEN): Adapter layer implementation complete
        use crate::mcp_integration::tools::*;
        use crate::services::semantic::HybridSearchEngine;

        // Get configuration
        let api_key = self
            .config
            .semantic_api_key
            .as_ref()
            .ok_or("Semantic search enabled but API key not configured")?;

        let db_path = self
            .config
            .semantic_db_path
            .clone()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| {
                        h.join(".pmat")
                            .join("embeddings.db")
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_else(|| "embeddings.db".to_string())
            });

        let workspace = self
            .config
            .semantic_workspace
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Initialize hybrid search engine
        tracing::info!(
            "Initializing semantic search engine: db={}, workspace={}",
            db_path,
            workspace.display()
        );

        let engine = match HybridSearchEngine::new(api_key, &db_path, &workspace).await {
            Ok(engine) => Arc::new(engine),
            Err(e) => {
                tracing::warn!("Failed to initialize semantic search engine: {}", e);
                tracing::info!("Semantic search tools will not be available");
                return Ok(());
            }
        };

        // Register semantic search tools via adapter layer
        let mut tools = self.context.tools.write();

        tools.register(Arc::new(SemanticSearchToolAdapter::new(engine.clone())));
        tracing::info!("✓ Registered semantic_search tool");

        tools.register(Arc::new(FindSimilarCodeToolAdapter::new(engine.clone())));
        tracing::info!("✓ Registered find_similar_code tool");

        tools.register(Arc::new(ClusterCodeToolAdapter::new(engine.clone())));
        tracing::info!("✓ Registered cluster_code tool");

        tools.register(Arc::new(AnalyzeTopicsToolAdapter::new(engine)));
        tracing::info!("✓ Registered analyze_topics tool");

        tracing::info!("✅ Semantic search tools registered successfully (4 tools)");

        Ok(())
    }

    async fn register_agent_resources(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::resources::*;

        let mut resources = self.context.resources.write();

        // Register agent state resource
        resources.register(Arc::new(AgentStateResource::new(
            self.context.agent_registry.clone(),
        )));

        // Register metrics resource
        resources.register(Arc::new(MetricsResource::new(
            self.context.agent_registry.clone(),
        )));

        // Register quality report resource
        resources.register(Arc::new(QualityReportResource::new()));

        Ok(())
    }

    async fn register_agent_prompts(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::prompts::*;

        let mut prompts = self.context.prompts.write();

        // Register code analysis prompt
        prompts.register(Arc::new(CodeAnalysisPrompt::new()));

        // Register refactoring prompt
        prompts.register(Arc::new(RefactoringPrompt::new()));

        // Register quality assessment prompt
        prompts.register(Arc::new(QualityAssessmentPrompt::new()));

        Ok(())
    }

    pub async fn run_tcp(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.config.bind_address).await?;
        println!("MCP Server listening on {}", self.config.bind_address);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_connections));

        loop {
            tokio::select! {
                accept = listener.accept() => {
                    let (stream, addr) = accept?;

                    let permit = semaphore.clone().acquire_owned().await?;
                    let context = self.context.clone();
                    let config = self.config.clone();

                    tokio::spawn(async move {
                        let transport = Arc::new(TcpTransport::new(stream));
                        let session = McpSession::new(context, transport.clone());

                        if let Err(e) = handle_session(session, config).await {
                            eprintln!("Session error from {}: {}", addr, e);
                        }

                        drop(permit);
                    });
                }
                _ = self.shutdown.notified() => {
                    println!("Shutting down MCP server");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn run_unix(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket_path = self
            .config
            .unix_socket
            .as_ref()
            .ok_or("Unix socket path not configured")?;

        // Remove existing socket file
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path)?;
        println!("MCP Server listening on Unix socket: {}", socket_path);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_connections));

        loop {
            tokio::select! {
                accept = listener.accept() => {
                    let (stream, _) = accept?;

                    let permit = semaphore.clone().acquire_owned().await?;
                    let context = self.context.clone();
                    let config = self.config.clone();

                    tokio::spawn(async move {
                        let transport = Arc::new(UnixTransport::new(stream));
                        let session = McpSession::new(context, transport.clone());

                        if let Err(e) = handle_session(session, config).await {
                            eprintln!("Session error: {}", e);
                        }

                        drop(permit);
                    });
                }
                _ = self.shutdown.notified() => {
                    println!("Shutting down MCP server");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn run_stdio(&self) -> Result<(), Box<dyn std::error::Error>> {
        let transport = Arc::new(StdioTransport::new());
        let session = McpSession::new(self.context.clone(), transport);

        handle_session(session, self.config.clone()).await
    }

    pub fn shutdown(&self) {
        self.shutdown.notify_waiters();
    }
}

async fn handle_session(
    session: McpSession,
    config: ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let transport = session.transport.clone();

    loop {
        tokio::select! {
            message = transport.receive() => {
                let message = message?;

                match message {
                    McpMessage::JsonRpc(JsonRpcMessage::Request(request)) => {
                        let response = match tokio::time::timeout(
                            config.request_timeout,
                            session.handle_request(request)
                        ).await {
                            Ok(response) => response,
                            Err(_) => {
                                return Err(Box::new(McpError {
                                    code: -32002,
                                    message: "Request timed out".to_string(),
                                    data: None,
                                }));
                            }
                        };

                        let message = McpMessage::JsonRpc(JsonRpcMessage::Response(response));
                        transport.send(message).await?;
                    }
                    McpMessage::JsonRpc(JsonRpcMessage::Notification(notification)) => {
                        // Handle notifications
                        match notification.method.as_str() {
                            "notifications/cancelled" => {
                                // Handle cancellation
                            }
                            "notifications/progress" => {
                                // Handle progress updates
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// TCP transport implementation
struct TcpTransport {
    stream: Arc<tokio::sync::Mutex<tokio::net::TcpStream>>,
}

impl TcpTransport {
    fn new(stream: tokio::net::TcpStream) -> Self {
        Self {
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
        }
    }
}

#[async_trait]
impl McpTransport for TcpTransport {
    async fn send(&self, message: McpMessage) -> Result<(), McpError> {
        let json = serde_json::to_string(&message).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to serialize message: {}", e),
            data: None,
        })?;

        let mut stream = self.stream.lock().await;
        stream
            .write_all(json.as_bytes())
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send message: {}", e),
                data: None,
            })?;
        stream.write_all(b"\n").await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to send newline: {}", e),
            data: None,
        })?;
        stream.flush().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to flush stream: {}", e),
            data: None,
        })?;

        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        let mut stream = self.stream.lock().await;
        let mut reader = BufReader::new(&mut *stream);
        let mut line = String::new();

        reader.read_line(&mut line).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to read message: {}", e),
            data: None,
        })?;

        if line.is_empty() {
            return Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Connection closed".to_string(),
                data: None,
            });
        }

        serde_json::from_str(&line).map_err(|e| McpError {
            code: error_codes::PARSE_ERROR,
            message: format!("Failed to parse message: {}", e),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to close connection: {}", e),
            data: None,
        })
    }
}

// Unix socket transport
struct UnixTransport {
    stream: Arc<tokio::sync::Mutex<tokio::net::UnixStream>>,
}

impl UnixTransport {
    fn new(stream: tokio::net::UnixStream) -> Self {
        Self {
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
        }
    }
}

#[async_trait]
impl McpTransport for UnixTransport {
    async fn send(&self, message: McpMessage) -> Result<(), McpError> {
        let json = serde_json::to_string(&message).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to serialize message: {}", e),
            data: None,
        })?;

        let mut stream = self.stream.lock().await;
        stream
            .write_all(json.as_bytes())
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send message: {}", e),
                data: None,
            })?;
        stream.write_all(b"\n").await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to send newline: {}", e),
            data: None,
        })?;
        stream.flush().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to flush stream: {}", e),
            data: None,
        })?;

        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        let mut stream = self.stream.lock().await;
        let mut reader = BufReader::new(&mut *stream);
        let mut line = String::new();

        reader.read_line(&mut line).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to read message: {}", e),
            data: None,
        })?;

        if line.is_empty() {
            return Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Connection closed".to_string(),
                data: None,
            });
        }

        serde_json::from_str(&line).map_err(|e| McpError {
            code: error_codes::PARSE_ERROR,
            message: format!("Failed to parse message: {}", e),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to close connection: {}", e),
            data: None,
        })
    }
}

// Stdio transport for CLI usage
struct StdioTransport;

impl StdioTransport {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send(&self, message: McpMessage) -> Result<(), McpError> {
        let json = serde_json::to_string(&message).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to serialize message: {}", e),
            data: None,
        })?;

        println!("{}", json);
        Ok(())
    }

    async fn receive(&self) -> Result<McpMessage, McpError> {
        use tokio::io::{self, AsyncBufReadExt};

        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin);
        let mut line = String::new();

        reader.read_line(&mut line).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to read message: {}", e),
            data: None,
        })?;

        if line.is_empty() {
            return Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "EOF reached".to_string(),
                data: None,
            });
        }

        serde_json::from_str(&line).map_err(|e| McpError {
            code: error_codes::PARSE_ERROR,
            message: format!("Failed to parse message: {}", e),
            data: None,
        })
    }

    async fn close(&self) -> Result<(), McpError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_address, "127.0.0.1:3000");
        assert_eq!(config.max_connections, 100);
    }

    #[actix_rt::test]
    async fn test_mcp_server_creation() {
        use crate::agents::registry::AgentRegistry;

        let registry = Arc::new(AgentRegistry::new());
        let config = ServerConfig::default();

        let server = McpServer::new(registry, config).unwrap();
        assert_eq!(server.context.server_info.protocol_version, MCP_VERSION);
    }
}
