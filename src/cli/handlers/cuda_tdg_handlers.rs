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

    let output = format_result(&result, &config)?;
    write_output(&output, &config)?;

    // Quality gate enforcement is only in `pmat cuda-tdg gate` subcommand.
    // Default report mode prints results without hard-failing.

    Ok(())
}

/// Handle CUDA-TDG subcommands
async fn handle_cuda_tdg_subcommand(
    cmd: &CudaTdgCommand,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    match cmd {
        CudaTdgCommand::Analyze { path } => handle_analyze(path, config).await,
        CudaTdgCommand::Score { path, breakdown } => handle_score(path, *breakdown, config).await,
        CudaTdgCommand::Report {
            path,
            format,
            output,
        } => handle_report(path, format, output.as_ref(), config).await,
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
        } => handle_gate(path, *min_score, *fail_on_p0, config).await,
        CudaTdgCommand::Kaizen { path, since } => {
            handle_kaizen(path, since.as_deref(), config).await
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
