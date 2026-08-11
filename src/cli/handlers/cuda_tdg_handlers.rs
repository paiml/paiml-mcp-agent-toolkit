//! CUDA-SIMD TDG CLI handlers
//!
//! Implements command handlers for the 100-point Popper falsification scoring system.
//! Analyzes CUDA PTX, SIMD (AVX2/AVX-512/NEON), and WGPU compute code.
//!
//! # Toyota Way Integration
//!
//! - **Jidoka**: Automatic quality gates that stop on P0 defect detection
//! - **Kaizen**: Continuous improvement through historical Tauranta fault analysis
//! - **Poka-Yoke**: Error-proofing through static analysis

#![cfg_attr(coverage_nightly, coverage(off))]
#![allow(clippy::wildcard_in_or_patterns)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_char_add_str)]

use crate::cli::commands::{CudaTdgCommand, CudaTdgOutputFormat};
use crate::tdg::{
    CudaSimdAnalyzer, CudaSimdConfig, CudaSimdTdgResult, CudaTdgGrade, DefectSeverity,
    DefectTaxonomy, PopperScore,
};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

/// Configuration for CUDA-TDG command handling
pub struct CudaTdgCommandConfig {
    pub path: PathBuf,
    pub command: Option<CudaTdgCommand>,
    pub format: CudaTdgOutputFormat,
    pub min_score: f64,
    pub fail_on_p0: bool,
    pub simd: bool,
    pub wgpu: bool,
    pub output: Option<PathBuf>,
    pub quiet: bool,
}

/// Main handler for cuda-tdg command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_cuda_tdg_command(config: CudaTdgCommandConfig) -> Result<()> {
    if let Some(ref cmd) = config.command {
        return handle_cuda_tdg_subcommand(cmd, &config).await;
    }

    let analyzer_config = CudaSimdConfig {
        min_score: config.min_score,
        fail_on_p0: config.fail_on_p0,
        analyze_simd: config.simd,
        analyze_wgpu: config.wgpu,
        ..Default::default()
    };

    let analyzer = CudaSimdAnalyzer::with_config(analyzer_config);
    let result = analyzer.analyze(&config.path)?;

    if report_if_unmeasured(&result, &config)? {
        return Ok(());
    }

    let output = format_result(&result, &config)?;
    write_output(&output, &config)?;

    // Quality gate enforcement is only in `pmat cuda-tdg gate` subcommand.
    // Default report mode prints results without hard-failing.

    Ok(())
}

/// Render the "nothing was measured" report for a path with no analysable files.
///
/// GH-662: with `files_analyzed == 0` the scoring path still produced a number —
/// an identical 55.5/100, Grade D, "Gateway: PASSED" for two different
/// nonexistent paths *and* for an empty real directory. A score manufactured
/// from no measured files is worse than no score, because it looks like a
/// finding, so say so instead of printing one.
fn format_unmeasured(result: &CudaSimdTdgResult, config: &CudaTdgCommandConfig) -> Result<String> {
    match config.format {
        CudaTdgOutputFormat::Json => format_unmeasured_json(result),
        _ => Ok(format_unmeasured_text(result)),
    }
}

/// Machine-readable form of the unmeasured report: explicit nulls, never zeros —
/// a zero score would read as a finding.
fn format_unmeasured_json(result: &CudaSimdTdgResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "path": result.path.display().to_string(),
        "files_analyzed": 0,
        "measured": false,
        "score": serde_json::Value::Null,
        "grade": serde_json::Value::Null,
        "gateway_passed": serde_json::Value::Null,
        "reason": "no analysable source files were read",
    }))?)
}

fn format_unmeasured_text(result: &CudaSimdTdgResult) -> String {
    format!(
        "CUDA-SIMD TDG Analysis\n\
         ======================\n\n\
         Path: {}\n\
         Files: 0 analysable source files found\n\n\
         Score: not measured\n\
         Gateway: not measured\n\n\
         Nothing was read, so nothing can be scored.\n",
        result.path.display()
    )
}

/// Write the unmeasured report when nothing was analysed; `Ok(true)` means the
/// caller must stop rather than print a score.
fn report_if_unmeasured(result: &CudaSimdTdgResult, config: &CudaTdgCommandConfig) -> Result<bool> {
    if result.files_analyzed > 0 {
        return Ok(false);
    }
    let output = format_unmeasured(result, config)?;
    write_output(&output, config)?;
    Ok(true)
}

/// Resolve which path a subcommand analyses.
///
/// `pmat cuda-tdg [PATH] <SUBCOMMAND>` accepted the top-level positional and
/// then threw it away: every subcommand carried its own
/// `#[arg(default_value = ".")]`, so `pmat cuda-tdg /does/not/exist score`
/// graded the CURRENT DIRECTORY and exited 0 — 81.0/100 "Gateway: PASSED" from
/// the pmat repo, 56.5/100 from a small corpus, for the same nonexistent
/// argument. A path the user typed must never be silently replaced by the cwd.
///
/// An explicit path after the subcommand still wins (`cuda-tdg score /path`);
/// otherwise the top-level path is honoured, which itself defaults to `.`.
fn resolve_subcommand_path<'a>(
    path: Option<&'a PathBuf>,
    config: &'a CudaTdgCommandConfig,
) -> &'a PathBuf {
    path.unwrap_or(&config.path)
}

/// Handle CUDA-TDG subcommands
async fn handle_cuda_tdg_subcommand(
    cmd: &CudaTdgCommand,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    match cmd {
        CudaTdgCommand::Analyze { path } => handle_analyze(path, config).await,
        CudaTdgCommand::Score { path, breakdown } => {
            handle_score(
                resolve_subcommand_path(path.as_ref(), config),
                *breakdown,
                config,
            )
            .await
        }
        CudaTdgCommand::Report {
            path,
            format,
            output,
        } => {
            handle_report(
                resolve_subcommand_path(path.as_ref(), config),
                format,
                output.as_ref(),
                config,
            )
            .await
        }
        CudaTdgCommand::BarrierCheck { path } => handle_barrier_check(path, config).await,
        CudaTdgCommand::ValidateTiles {
            head_dim,
            tile_kv,
            shared_memory,
        } => handle_validate_tiles(*head_dim, *tile_kv, *shared_memory, config).await,
        CudaTdgCommand::Gate {
            path,
            min_score,
            fail_on_p0,
        } => {
            handle_gate(
                resolve_subcommand_path(path.as_ref(), config),
                *min_score,
                *fail_on_p0,
                config,
            )
            .await
        }
        CudaTdgCommand::Kaizen { path, since } => {
            handle_kaizen(
                resolve_subcommand_path(path.as_ref(), config),
                since.as_deref(),
                config,
            )
            .await
        }
        CudaTdgCommand::Taxonomy => handle_taxonomy(config).await,
    }
}

// Subcommand handlers: analyze, score, report, barrier_check, validate_tiles
include!("cuda_tdg_handlers_subcommands.rs");

// Subcommand handlers: gate, kaizen, taxonomy
include!("cuda_tdg_handlers_gate_kaizen.rs");

// Formatting: format_result, format_analysis, format_score_summary/breakdown, format_barrier_safety
include!("cuda_tdg_handlers_format_score.rs");

// Formatting: format_terminal_output, format_markdown_report, format_html_report, format_sarif, write_output
include!("cuda_tdg_handlers_format_report.rs");

// Tests extracted to cuda_tdg_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "cuda_tdg_handlers_tests.rs"]
mod tests;
