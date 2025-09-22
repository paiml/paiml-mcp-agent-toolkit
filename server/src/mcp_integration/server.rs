use super::*;
use crate::agents::registry::AgentRegistry;
use std::sync::Arc;
use tokio::net::{TcpListener, UnixListener};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "PMAT MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            bind_address: "127.0.0.1:3000".to_string(),
            unix_socket: None,
            max_connections: 100,
            request_timeout: std::time::Duration::from_secs(30),
            enable_logging: true,
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
        tools.register(Arc::new(AnalyzeTool::new(self.context.agent_registry.clone())));
        
        // Register transform tool
        tools.register(Arc::new(TransformTool::new(self.context.agent_registry.clone())));
        
        // Register validate tool
        tools.register(Arc::new(ValidateTool::new(self.context.agent_registry.clone())));
        
        // Register orchestrate tool
        tools.register(Arc::new(OrchestrateTool::new(self.context.agent_registry.clone())));
        
        Ok(())
    }

    async fn register_agent_resources(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::resources::*;
        
        let mut resources = self.context.resources.write();
        
        // Register agent state resource
        resources.register(Arc::new(AgentStateResource::new(self.context.agent_registry.clone())));
        
        // Register metrics resource
        resources.register(Arc::new(MetricsResource::new(self.context.agent_registry.clone())));
        
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
        let socket_path = self.config.unix_socket.as_ref()
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
                        let response = tokio::time::timeout(
                            config.request_timeout,
                            session.handle_request(request)
                        ).await??;
                        
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
        let json = serde_json::to_string(&message)
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to serialize message: {}", e),
                data: None,
            })?;
        
        let mut stream = self.stream.lock().await;
        stream.write_all(json.as_bytes()).await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send message: {}", e),
                data: None,
            })?;
        stream.write_all(b"\n").await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send newline: {}", e),
                data: None,
            })?;
        stream.flush().await
            .map_err(|e| McpError {
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
        
        reader.read_line(&mut line).await
            .map_err(|e| McpError {
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
        
        serde_json::from_str(&line)
            .map_err(|e| McpError {
                code: error_codes::PARSE_ERROR,
                message: format!("Failed to parse message: {}", e),
                data: None,
            })
    }

    async fn close(&self) -> Result<(), McpError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await
            .map_err(|e| McpError {
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
        let json = serde_json::to_string(&message)
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to serialize message: {}", e),
                data: None,
            })?;
        
        let mut stream = self.stream.lock().await;
        stream.write_all(json.as_bytes()).await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send message: {}", e),
                data: None,
            })?;
        stream.write_all(b"\n").await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to send newline: {}", e),
                data: None,
            })?;
        stream.flush().await
            .map_err(|e| McpError {
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
        
        reader.read_line(&mut line).await
            .map_err(|e| McpError {
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
        
        serde_json::from_str(&line)
            .map_err(|e| McpError {
                code: error_codes::PARSE_ERROR,
                message: format!("Failed to parse message: {}", e),
                data: None,
            })
    }

    async fn close(&self) -> Result<(), McpError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await
            .map_err(|e| McpError {
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
        let json = serde_json::to_string(&message)
            .map_err(|e| McpError {
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
        
        reader.read_line(&mut line).await
            .map_err(|e| McpError {
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
        
        serde_json::from_str(&line)
            .map_err(|e| McpError {
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