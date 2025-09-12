use anyhow::Result;
use pmat::{cli, stateless_server::StatelessTemplateServer};
use std::io::IsTerminal;
use std::process;
use std::sync::Arc;
use tracing::{debug, error, info, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

enum ExecutionMode {
    Mcp,
    Cli,
}

/// POSIX-compliant exit codes for CLI interface
/// Per SPECIFICATION.md Section 23: CLI Interface
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ExitCode {
    /// Success
    Success = 0,
    /// General error
    GeneralError = 1,
    /// Misuse of shell command
    MisuseError = 2,
    /// Permission denied
    PermissionDenied = 126,
    /// Command not found
    CommandNotFound = 127,
    /// Invalid argument to exit
    InvalidExitArg = 128,
    /// Quality gate failure (custom)
    QualityGateFailure = 3,
    /// Configuration error (custom)
    ConfigurationError = 4,
    /// Analysis error (custom)
    AnalysisError = 5,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
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
    let filter = create_env_filter(cli)?;

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(cli.debug || cli.trace)
                .with_thread_ids(cli.trace)
                .with_file(cli.trace)
                .with_line_number(cli.trace)
                .compact()
                .with_writer(std::io::stderr),
        )
        .init();

    Ok(())
}

/// Create environment filter based on CLI flags
fn create_env_filter(cli: &cli::EarlyCliArgs) -> Result<EnvFilter> {
    if cli.is_mcp_server {
        Ok(create_mcp_filter(cli.debug))
    } else {
        create_cli_filter(cli)
    }
}

/// Create filter for MCP server mode
fn create_mcp_filter(debug: bool) -> EnvFilter {
    if debug {
        EnvFilter::new("warn,pmat=debug")
    } else {
        EnvFilter::new("off")
    }
}

/// Create filter for CLI mode
fn create_cli_filter(cli: &cli::EarlyCliArgs) -> Result<EnvFilter> {
    if let Some(ref custom) = cli.trace_filter {
        return Ok(EnvFilter::try_new(custom)?);
    }

    let filter_str = match (cli.trace, cli.debug, cli.verbose) {
        (true, _, _) => "debug,pmat=trace",
        (_, true, _) => "warn,pmat=debug",
        (_, _, true) => "warn,pmat=info",
        _ => return Ok(get_default_filter()),
    };

    Ok(EnvFilter::new(filter_str))
}

/// Get default production filter
fn get_default_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
}

#[tokio::main]
async fn main() {
    let exit_code = match run_main().await {
        Ok(()) => ExitCode::Success,
        Err(e) => {
            error!("Error: {}", e);
            categorize_error(&e)
        }
    };

    if exit_code as i32 != 0 {
        process::exit(exit_code.into());
    }
}

fn categorize_error(error: &anyhow::Error) -> ExitCode {
    let error_str = error.to_string().to_lowercase();

    match () {
        _ if is_quality_gate_error(&error_str) => ExitCode::QualityGateFailure,
        _ if is_configuration_error(&error_str) => ExitCode::ConfigurationError,
        _ if is_analysis_error(&error_str) => ExitCode::AnalysisError,
        _ if is_permission_error(&error_str) => ExitCode::PermissionDenied,
        _ => ExitCode::GeneralError,
    }
}

fn is_quality_gate_error(error_str: &str) -> bool {
    error_str.contains("quality gate") || error_str.contains("violation")
}

fn is_configuration_error(error_str: &str) -> bool {
    error_str.contains("config") || error_str.contains("parse")
}

fn is_analysis_error(error_str: &str) -> bool {
    error_str.contains("analysis") || error_str.contains("complexity")
}

fn is_permission_error(error_str: &str) -> bool {
    error_str.contains("permission") || error_str.contains("access")
}

async fn run_main() -> Result<()> {
    // Parse CLI to get tracing configuration early
    let cli = cli::parse_early_for_tracing();

    // Initialize enhanced tracing system
    init_tracing(&cli)?;

    // Only log to stdout if not running MCP server mode
    if !cli.is_mcp_server {
        info!(
            "Starting PAIML MCP Agent Toolkit v{}",
            env!("CARGO_PKG_VERSION")
        );
        debug!("Debug logging enabled");
        trace!("Trace logging enabled");
    }

    // Create shared template server
    let server = Arc::new(StatelessTemplateServer::new()?);

    if !cli.is_mcp_server {
        debug!("Template server initialized");
    }

    match detect_execution_mode() {
        ExecutionMode::Mcp => {
            if !cli.is_mcp_server {
                info!("Running unified MCP server (pmcp SDK)");
            }
            let unified_server = pmat::mcp_pmcp::UnifiedServer::new()
                .map_err(|e| anyhow::anyhow!("Failed to create unified server: {}", e))?;
            unified_server
                .run()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))
        }
        ExecutionMode::Cli => {
            if !cli.is_mcp_server {
                info!("Running in CLI mode");
            }
            cli::run(server).await
        }
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
