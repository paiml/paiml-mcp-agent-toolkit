use anyhow::Result;
use pmat::{cli, stateless_server::StatelessTemplateServer};
use std::io::IsTerminal;
use std::sync::Arc;
use tracing::{debug, info, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

enum ExecutionMode {
    Mcp,
    Cli,
}

fn detect_execution_mode() -> ExecutionMode {
    let is_mcp = !std::io::stdin().is_terminal() && std::env::args().len() == 1
        || std::env::var("MCP_VERSION").is_ok();

    if is_mcp {
        debug!("Detected MCP server mode");
        ExecutionMode::Mcp
    } else {
        debug!("Detected CLI mode");
        ExecutionMode::Cli
    }
}

/// Initialize the enhanced tracing system based on CLI flags
fn init_tracing(cli: &cli::EarlyCliArgs) -> Result<()> {
    let filter = if let Some(ref custom) = cli.trace_filter {
        EnvFilter::try_new(custom)?
    } else if cli.trace {
        EnvFilter::new("debug,pmat=trace")
    } else if cli.debug {
        EnvFilter::new("warn,pmat=debug")
    } else if cli.verbose {
        EnvFilter::new("warn,pmat=info")
    } else {
        // Production default: only errors and warnings
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(cli.debug || cli.trace)
                .with_thread_ids(cli.trace)
                .with_file(cli.trace)
                .with_line_number(cli.trace)
                .compact(),
        )
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI to get tracing configuration early
    let cli = cli::parse_early_for_tracing();

    // Initialize enhanced tracing system
    init_tracing(&cli)?;

    info!(
        "Starting PAIML MCP Agent Toolkit v{}",
        env!("CARGO_PKG_VERSION")
    );
    debug!("Debug logging enabled");
    trace!("Trace logging enabled");

    // Create shared template server
    let server = Arc::new(StatelessTemplateServer::new()?);
    debug!("Template server initialized");

    match detect_execution_mode() {
        ExecutionMode::Mcp => {
            // Check if we should run the refactor MCP server
            if std::env::var("PMAT_REFACTOR_MCP").is_ok() {
                info!("Running in Refactor MCP server mode");
                let refactor_server = pmat::mcp_server::McpServer::new();
                refactor_server
                    .run()
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))
            } else {
                info!("Running in standard MCP server mode");
                pmat::run_mcp_server(server).await
            }
        }
        ExecutionMode::Cli => {
            info!("Running in CLI mode");
            cli::run(server).await
        }
    }
}
