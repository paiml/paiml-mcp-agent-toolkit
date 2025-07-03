use crate::models::mcp::{McpRequest, McpResponse};
use crate::mcp_server::state_manager::StateManager;
use crate::mcp_server::handlers;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub struct McpServer {
    state_manager: Arc<Mutex<StateManager>>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            state_manager: Arc::new(Mutex::new(StateManager::new())),
        }
    }

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