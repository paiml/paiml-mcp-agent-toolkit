use anyhow::Result;
use paiml_mcp_agent_toolkit::{cli, stateless_server::StatelessTemplateServer};
use std::io::IsTerminal;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

enum ExecutionMode {
    Mcp,
    Cli,
}

fn detect_execution_mode() -> ExecutionMode {
    let is_mcp = !std::io::stdin().is_terminal() && std::env::args().len() == 1
        || std::env::var("MCP_VERSION").is_ok();

    if is_mcp {
        ExecutionMode::Mcp
    } else {
        ExecutionMode::Cli
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for both modes
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Create shared template server
    let server = Arc::new(StatelessTemplateServer::new()?);

    match detect_execution_mode() {
        ExecutionMode::Mcp => paiml_mcp_agent_toolkit::run_mcp_server(server).await,
        ExecutionMode::Cli => cli::run(server).await,
    }
}
