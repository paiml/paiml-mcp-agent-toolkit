use anyhow::Result;
use mcp_template_server::handlers;
use mcp_template_server::models::mcp::{McpRequest, McpResponse};
use mcp_template_server::stateless_server::StatelessTemplateServer;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Check for --version flag before initializing anything else
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("paiml-mcp-agent-toolkit {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Initialize tracing with configurable log level
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("mcp_agent_toolkit=debug".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    info!("Starting MCP Agent Toolkit server (stateless mode)");

    // Create stateless server instance (no AWS dependencies)
    let server = Arc::new(StatelessTemplateServer::new()?);

    info!("MCP server ready, waiting for requests on stdin...");

    // Read JSON-RPC requests from stdin
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON-RPC request
        match serde_json::from_str::<McpRequest>(&line) {
            Ok(request) => {
                info!(
                    "Received request: method={}, id={:?}",
                    request.method, request.id
                );

                // Handle the request using the existing handler
                let response = handlers::handle_request(Arc::clone(&server), request).await;

                // Write response to stdout
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);

                // Send error response
                let error_response = McpResponse::error(
                    serde_json::Value::Null,
                    -32700,
                    format!("Parse error: {}", e),
                );

                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}
