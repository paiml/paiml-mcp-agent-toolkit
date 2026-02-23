#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG subcommand routing and compare handler
//!
//! Routes TdgCommand variants to their respective handlers.

use super::formatting::format_comparison;
use super::TdgCommandConfig;
use crate::cli::commands::TdgCommand;
use crate::tdg::TdgAnalyzer;
use anyhow::Result;
use std::path::Path;

/// Handle TDG subcommands (cognitive complexity ≤8)
pub(super) async fn handle_tdg_subcommand(
    cmd: TdgCommand,
    analyzer: &TdgAnalyzer,
    config: &TdgCommandConfig,
) -> Result<()> {
    match cmd {
        TdgCommand::Compare { source1, source2 } => {
            handle_compare_command(analyzer, &source1, &source2, config).await
        }
        TdgCommand::History {
            commit,
            since,
            range,
            path,
            format,
        } => {
            super::history::handle_history_command(
                analyzer, commit, since, range, path, format, config,
            )
            .await
        }
        TdgCommand::Baseline { command } => {
            super::baseline::handle_baseline_command(command, analyzer, config).await
        }
        TdgCommand::CheckRegression {
            baseline,
            path,
            format,
            fail_on_regression,
            max_score_drop,
            allow_grade_drop,
        } => {
            super::quality_gates::handle_check_regression(
                analyzer,
                baseline.as_path(),
                path.as_path(),
                format.clone(),
                fail_on_regression,
                max_score_drop,
                allow_grade_drop,
            )
            .await
        }
        TdgCommand::CheckQuality {
            path,
            min_grade,
            format,
            fail_on_violation,
            new_files_only,
            baseline,
        } => {
            super::quality_gates::handle_check_quality(
                analyzer,
                path.as_path(),
                min_grade.as_deref(),
                format.clone(),
                fail_on_violation,
                new_files_only,
                baseline.as_ref(),
            )
            .await
        }
        TdgCommand::Diagnostics { .. }
        | TdgCommand::Storage { .. }
        | TdgCommand::Dashboard { .. }
        | TdgCommand::Config(_) => {
            super::super::tdg_diagnostic_handler::handle_tdg_diagnostics(&cmd, &config.path).await
        }
    }
}

/// Handle TDG compare subcommand (cognitive complexity ≤4)
async fn handle_compare_command(
    analyzer: &TdgAnalyzer,
    source1: &Path,
    source2: &Path,
    config: &TdgCommandConfig,
) -> Result<()> {
    let comparison = analyzer.compare(source1, source2).await?;
    let output_str = format_comparison(comparison, config.format.clone())?;

    if let Some(output_path) = &config.output {
        std::fs::write(output_path, output_str)?;
    } else {
        println!("{output_str}");
    }

    Ok(())
}
