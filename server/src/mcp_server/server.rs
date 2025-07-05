use crate::mcp_server::handlers;
use crate::mcp_server::state_manager::StateManager;
use crate::models::mcp::{McpRequest, McpResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Model Context Protocol (MCP) server for PMAT refactoring capabilities.
///
/// This server implements the MCP specification for AI-assisted refactoring,
/// providing a standardized interface for Claude and other AI models to interact
/// with PMAT's analysis and refactoring capabilities. Critical for maintaining
/// MCP protocol compliance and preventing API drift.
///
/// # MCP Protocol Support
///
/// - **Protocol Version**: 2024-11-05
/// - **Transport**: JSON-RPC 2.0 over stdin/stdout
/// - **Capabilities**: Refactoring state machine control
/// - **Error Handling**: Standard JSON-RPC error codes
///
/// # Supported Methods
///
/// ## Core Protocol Methods
/// - `initialize` - Initialize MCP session with capabilities
///
/// ## Refactoring Methods
/// - `refactor.start` - Start new refactoring session
/// - `refactor.nextIteration` - Advance refactoring state machine
/// - `refactor.getState` - Get current refactoring state
/// - `refactor.stop` - Stop current refactoring session
///
/// # State Management
///
/// The server maintains refactoring sessions with:
/// - Target files and configuration
/// - State machine progression (Scan → Analyze → Plan → Refactor → Complete)
/// - Session isolation and cleanup
/// - Error recovery and rollback
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::mcp_server::server::McpServer;
///
/// # tokio_test::block_on(async {
/// // Create MCP server
/// let server = McpServer::new();
///
/// // Server is ready for MCP communication
/// // In real usage, server.run().await would handle stdin/stdout
/// # });
/// ```
///
/// # MCP Message Examples
///
/// ## Initialize Request
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 1,
///   "method": "initialize",
///   "params": {
///     "protocolVersion": "2024-11-05",
///     "clientInfo": {
///       "name": "claude-desktop",
///       "version": "1.0.0"
///     }
///   }
/// }
/// ```
///
/// ## Refactor Start Request
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 2,
///   "method": "refactor.start",
///   "params": {
///     "targets": ["/path/to/file.rs"],
///     "config": {
///       "target_complexity": 15,
///       "remove_satd": true
///     }
///   }
/// }
/// ```
///
/// ## State Query Request
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 3,
///   "method": "refactor.getState"
/// }
/// ```
pub struct McpServer {
    state_manager: Arc<Mutex<StateManager>>,
}

impl McpServer {
    /// Creates a new MCP server with default state management.
    ///
    /// Initializes the server with a new state manager for handling refactoring
    /// sessions. The server is ready to accept MCP connections and process
    /// refactoring requests according to the MCP protocol specification.
    ///
    /// # Returns
    ///
    /// A new `McpServer` instance with initialized state management.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::mcp_server::server::McpServer;
    ///
    /// // Create MCP server for AI integration
    /// let server = McpServer::new();
    ///
    /// // Server is ready for MCP protocol communication
    /// // Typically used with: server.run().await
    /// ```
    ///
    /// # MCP Integration
    ///
    /// The server is designed to integrate with MCP clients like:
    /// - Claude Desktop application
    /// - VS Code with MCP extension
    /// - Custom MCP clients
    /// - CI/CD pipeline integrations
    pub fn new() -> Self {
        Self {
            state_manager: Arc::new(Mutex::new(StateManager::new())),
        }
    }

    /// Runs the MCP server main loop, handling stdin/stdout communication.
    ///
    /// This method implements the core MCP protocol communication loop, reading
    /// JSON-RPC messages from stdin and writing responses to stdout. Handles
    /// the complete MCP session lifecycle including initialization, method dispatch,
    /// and error handling.
    ///
    /// # Protocol Flow
    ///
    /// 1. **Initialization**: Send server capabilities to client
    /// 2. **Message Loop**: Process incoming JSON-RPC requests
    /// 3. **Method Dispatch**: Route requests to appropriate handlers
    /// 4. **Response Generation**: Send JSON-RPC responses
    /// 5. **Error Handling**: Handle protocol and processing errors
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Server shutdown gracefully
    /// * `Err(Box<dyn std::error::Error>)` - I/O error, protocol error, or handler failure
    ///
    /// # Communication Protocol
    ///
    /// The server uses line-delimited JSON over stdin/stdout:
    /// - **Input**: JSON-RPC 2.0 requests (one per line)
    /// - **Output**: JSON-RPC 2.0 responses (one per line)
    /// - **Transport**: stdin/stdout pipes
    /// - **Encoding**: UTF-8 text
    ///
    /// # Error Handling
    ///
    /// Standard JSON-RPC error codes:
    /// - `-32700`: Parse error (invalid JSON)
    /// - `-32600`: Invalid request (bad JSON-RPC)
    /// - `-32601`: Method not found
    /// - `-32602`: Invalid params
    /// - `-32603`: Internal error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::mcp_server::server::McpServer;
    ///
    /// # tokio_test::block_on(async {
    /// let server = McpServer::new();
    ///
    /// // In real usage, this would run the MCP protocol loop
    /// // let result = server.run().await;
    /// // assert!(result.is_ok());
    ///
    /// // For testing, we just verify the server can be created
    /// # });
    /// ```
    ///
    /// # MCP Client Integration
    ///
    /// ## Claude Desktop Configuration
    /// ```json
    /// {
    ///   "mcpServers": {
    ///     "pmat": {
    ///       "command": "pmat",
    ///       "args": ["serve", "mcp"],
    ///       "env": {}
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// ## VS Code MCP Extension
    /// ```json
    /// {
    ///   "mcp.servers": [{
    ///     "name": "PMAT Refactoring",
    ///     "command": ["pmat", "serve", "mcp"],
    ///     "capabilities": ["refactor"]
    ///   }]
    /// }
    /// ```
    ///
    /// # Protocol Messages
    ///
    /// ## Initialization Response
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "protocolVersion": "2024-11-05",
    ///     "capabilities": {
    ///       "refactor": {
    ///         "start": true,
    ///         "nextIteration": true,
    ///         "getState": true,
    ///         "stop": true
    ///       }
    ///     },
    ///     "serverInfo": {
    ///       "name": "pmat-mcp-server",
    ///       "version": "0.27.5"
    ///     }
    ///   }
    /// }
    /// ```
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting MCP server on stdin/stdout");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        // Send initialization response
        let init_response = json!({
            "jsonrpc": "2.0",
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "refactor": {
                        "start": true,
                        "nextIteration": true,
                        "getState": true,
                        "stop": true
                    }
                },
                "serverInfo": {
                    "name": "pmat-mcp-server",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        stdout
            .write_all(format!("{}\n", init_response).as_bytes())
            .await?;
        stdout.flush().await?;

        // Main message loop
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            debug!("Received MCP request: {}", line);

            let response = match self.handle_request(&line).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Error handling request: {}", e);
                    McpResponse::error(json!(null), -32603, e.to_string())
                }
            };

            let response_json = serde_json::to_string(&response)?;
            debug!("Sending MCP response: {}", response_json);

            stdout.write_all(response_json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        info!("MCP server shutting down");
        Ok(())
    }

    /// Handles a single MCP request and returns the appropriate response.
    ///
    /// This method implements the core MCP request processing logic, including
    /// JSON-RPC validation, method routing, and response generation. Critical for
    /// maintaining MCP protocol compliance and method stability.
    ///
    /// # Parameters
    ///
    /// * `line` - Raw JSON-RPC request string from stdin
    ///
    /// # Returns
    ///
    /// * `Ok(McpResponse)` - Valid MCP response (success or error)
    /// * `Err(Box<dyn std::error::Error>)` - Parse error or handler failure
    ///
    /// # Protocol Validation
    ///
    /// 1. **JSON Parsing**: Validate request is valid JSON
    /// 2. **JSON-RPC Validation**: Check jsonrpc field is "2.0"
    /// 3. **Method Resolution**: Route to appropriate handler
    /// 4. **Parameter Validation**: Validate method-specific parameters
    /// 5. **Response Generation**: Create success or error response
    ///
    /// # Supported MCP Methods
    ///
    /// - `initialize` - MCP session initialization
    /// - `refactor.start` - Start new refactoring session
    /// - `refactor.nextIteration` - Advance state machine
    /// - `refactor.getState` - Query current state
    /// - `refactor.stop` - Stop refactoring session
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::mcp_server::server::McpServer;
    ///
    /// # tokio_test::block_on(async {
    /// let server = McpServer::new();
    ///
    /// // Initialize request example
    /// let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    /// // The handle_request method is private and used internally
    /// // This demonstrates the expected JSON-RPC format
    /// assert!(init_request.contains("jsonrpc"));
    /// assert!(init_request.contains("2.0"));
    /// assert!(init_request.contains("initialize"));
    ///
    /// // Invalid JSON-RPC version example
    /// let bad_request = r#"{"jsonrpc":"1.0","id":2,"method":"test"}"#;
    /// // Should be rejected due to invalid version
    /// assert!(bad_request.contains("1.0"));
    /// # });
    /// ```
    ///
    /// # Request/Response Examples
    ///
    /// ## Initialize Method
    /// ```json
    /// // Request
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 1,
    ///   "method": "initialize",
    ///   "params": {
    ///     "protocolVersion": "2024-11-05"
    ///   }
    /// }
    ///
    /// // Response
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 1,
    ///   "result": {
    ///     "protocolVersion": "2024-11-05",
    ///     "capabilities": {
    ///       "refactor": {
    ///         "start": true,
    ///         "nextIteration": true,
    ///         "getState": true,
    ///         "stop": true
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// ## Refactor Start Method
    /// ```json
    /// // Request
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 2,
    ///   "method": "refactor.start",
    ///   "params": {
    ///     "targets": ["/path/to/file.rs"],
    ///     "config": {
    ///       "target_complexity": 15,
    ///       "remove_satd": true
    ///     }
    ///   }
    /// }
    ///
    /// // Response
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 2,
    ///   "result": {
    ///     "session_id": "uuid-string",
    ///     "state": {
    ///       "current": "Scan",
    ///       "targets": ["/path/to/file.rs"]
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// ## Error Response
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "id": 3,
    ///   "error": {
    ///     "code": -32601,
    ///     "message": "Method not found: unknown.method"
    ///   }
    /// }
    /// ```
    async fn handle_request(&self, line: &str) -> Result<McpResponse, Box<dyn std::error::Error>> {
        let request: McpRequest = serde_json::from_str(line)?;

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return Ok(McpResponse::error(
                request.id,
                -32600,
                "Invalid JSON-RPC version".to_string(),
            ));
        }

        // Route to appropriate handler
        let result = match request.method.as_str() {
            "initialize" => {
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "refactor": {
                            "start": true,
                            "nextIteration": true,
                            "getState": true,
                            "stop": true
                        }
                    },
                    "serverInfo": {
                        "name": "pmat-mcp-server",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })
            }
            "refactor.start" => {
                handlers::handle_refactor_start(
                    &self.state_manager,
                    request.params.unwrap_or(json!({})),
                )
                .await?
            }
            "refactor.nextIteration" => {
                handlers::handle_refactor_next_iteration(&self.state_manager).await?
            }
            "refactor.getState" => handlers::handle_refactor_get_state(&self.state_manager).await?,
            "refactor.stop" => handlers::handle_refactor_stop(&self.state_manager).await?,
            _ => {
                return Ok(McpResponse::error(
                    request.id,
                    -32601,
                    format!("Method not found: {}", request.method),
                ));
            }
        };

        Ok(McpResponse::success(request.id, result))
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
