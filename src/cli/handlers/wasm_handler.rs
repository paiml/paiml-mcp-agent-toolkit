//! WASM analysis handler for CLI (REFACTORED)
//! Following Toyota Way Extract Method to reduce complexity
//!
//! Submodules (via include!()):
//! - wasm_handler_formatting.rs: Output formatting (summary, JSON, detailed)
//! - wasm_handler_sarif.rs: SARIF security report generation
//! - wasm_handler_validation.rs: Result validation and metrics creation

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::WasmOutputFormat;
use crate::wasm::{
    analyzer::{AnalysisResult, WasmAnalyzer},
    baseline::{Metrics, QualityAssessment, QualityBaseline},
    profiler::AsyncProfiler,
    security::{PatternDetector, VulnerabilityMatch},
    verifier::{IncrementalVerifier, VerificationResult},
    ProfilingReport,
};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

/// Handle WASM analysis command (Complexity: ≤10)
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_wasm(
    wasm_file: PathBuf,
    format: WasmOutputFormat,
    verify: bool,
    security: bool,
    profile: bool,
    baseline: Option<PathBuf>,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    debug_assert!(
        wasm_file.exists(),
        "wasm_file must exist: {}",
        wasm_file.display()
    );
    info!("🔍 Analyzing WASM module: {}", wasm_file.display());

    // Load and analyze WASM binary
    let binary = load_wasm_file(&wasm_file)?;
    let analysis_result = run_basic_analysis(&binary)?;

    // Run optional analyses
    let verification_result = run_verification_if_requested(verify, &binary).await?;
    let security_results = run_security_scan_if_requested(security, &binary)?;
    let profiling_results = run_profiling_if_requested(profile, &binary).await?;
    let baseline_comparison =
        run_baseline_comparison_if_requested(baseline, &binary, &analysis_result).await?;

    // Format and output results
    let output_str = format_results(
        format,
        &analysis_result,
        verification_result.as_ref(),
        security_results.as_ref(),
        profiling_results.as_ref(),
        baseline_comparison.as_ref(),
        verbose,
    )?;

    write_output(output_str, output)?;

    // Check for failures
    check_for_failures(
        verification_result.as_ref(),
        security_results.as_ref(),
        baseline_comparison.as_ref(),
    )?;

    info!("✅ WASM analysis complete");
    Ok(())
}

/// Load WASM file from disk (Complexity: 1)
fn load_wasm_file(wasm_file: &PathBuf) -> Result<Vec<u8>> {
    let binary = std::fs::read(wasm_file)?;
    debug!("Loaded WASM binary: {} bytes", binary.len());
    Ok(binary)
}

/// Run basic WASM analysis (Complexity: 2)
fn run_basic_analysis(binary: &[u8]) -> Result<AnalysisResult> {
    let analyzer = WasmAnalyzer::new()?;
    analyzer.analyze(binary)
}

/// Run verification if requested (Complexity: 3)
async fn run_verification_if_requested(
    verify: bool,
    binary: &[u8],
) -> Result<Option<VerificationResult>> {
    if !verify {
        return Ok(None);
    }

    info!("🔒 Running formal verification...");
    let verifier = IncrementalVerifier::new()?;
    Ok(Some(verifier.verify_module(binary)?))
}

/// Run security scan if requested (Complexity: 4)
fn run_security_scan_if_requested(
    security: bool,
    binary: &[u8],
) -> Result<Option<Vec<VulnerabilityMatch>>> {
    if !security {
        return Ok(None);
    }

    info!("🛡️ Running security vulnerability scanning...");
    let mut detector = PatternDetector::new();

    for payload in wasmparser::Parser::new(0).parse_all(binary) {
        let payload = payload?;
        detector.scan(&payload)?;
    }

    Ok(Some(detector.finalize()))
}

/// Run profiling if requested (Complexity: 3)
async fn run_profiling_if_requested(
    profile: bool,
    binary: &[u8],
) -> Result<Option<ProfilingReport>> {
    if !profile {
        return Ok(None);
    }

    info!("📊 Running performance profiling...");
    let profiler = AsyncProfiler::new();
    Ok(Some(profiler.profile_module(binary).await?))
}

/// Run baseline comparison if requested (Complexity: 5)
async fn run_baseline_comparison_if_requested(
    baseline: Option<PathBuf>,
    _binary: &[u8],
    analysis_result: &AnalysisResult,
) -> Result<Option<QualityAssessment>> {
    let baseline_path = match baseline {
        Some(path) => path,
        None => return Ok(None),
    };

    info!("📈 Comparing with baseline: {}", baseline_path.display());

    let baseline_metrics = load_and_analyze_baseline(&baseline_path)?;
    let current_metrics = create_metrics_from_analysis(analysis_result);

    let quality_baseline = QualityBaseline::new(baseline_metrics.clone(), baseline_metrics);
    Ok(Some(quality_baseline.evaluate(&current_metrics)))
}

/// Load and analyze baseline WASM file (Complexity: 3)
fn load_and_analyze_baseline(baseline_path: &PathBuf) -> Result<Metrics> {
    let baseline_binary = std::fs::read(baseline_path)?;
    let baseline_analyzer = WasmAnalyzer::new()?;
    let baseline_result = baseline_analyzer.analyze(&baseline_binary)?;
    Ok(create_metrics_from_analysis(&baseline_result))
}

/// Write output to file or stdout (Complexity: 3)
fn write_output(output_str: String, output: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output {
        std::fs::write(&output_path, &output_str)?;
        info!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{output_str}");
    }
    Ok(())
}

// Result validation and metrics creation
include!("wasm_handler_validation.rs");

// Output formatting (summary, JSON, detailed)
include!("wasm_handler_formatting.rs");

// SARIF security report generation
include!("wasm_handler_sarif.rs");

// Tests extracted to wasm_handler_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "wasm_handler_tests.rs"]
mod tests;
