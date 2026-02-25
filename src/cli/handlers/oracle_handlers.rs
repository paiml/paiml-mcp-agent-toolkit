//! CLI handlers for PMAT Oracle - PDCA loop for automated quality improvement
//!
//! Toyota Way: Converges ANY Rust project toward perfect quality using CITL signals

use crate::cli::commands::{OracleCommands, OracleOutputFormat};
use crate::services::oracle::{ConvergenceTargets, OracleConfig, PdcaLoop, ProjectMetrics};
use anyhow::Result;
use std::path::Path;

/// Handle oracle command dispatch
pub async fn handle_oracle_command(command: OracleCommands) -> Result<()> {
    match command {
        OracleCommands::Fix {
            path,
            max_iterations,
            auto_apply_threshold,
            review_threshold,
            dry_run,
            format,
            output,
        } => {
            handle_oracle_fix(
                &path,
                max_iterations,
                auto_apply_threshold,
                review_threshold,
                dry_run,
                format,
                output.as_deref(),
            )
            .await
        }
        OracleCommands::Status { path, format } => handle_oracle_status(&path, format).await,
        OracleCommands::Single {
            path,
            format,
            output,
        } => handle_oracle_single(&path, format, output.as_deref()).await,
    }
}

// Command handler implementations (fix, status, single, collect_project_metrics)
include!("oracle_handlers_commands.rs");

// Output formatting functions (text, json, markdown)
include!("oracle_handlers_formatting.rs");

// Unit tests
include!("oracle_handlers_tests.rs");
