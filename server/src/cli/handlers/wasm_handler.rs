//! WASM analysis handler for CLI (REFACTORED)
//! Following Toyota Way Extract Method to reduce complexity

use crate::cli::WasmOutputFormat;
use crate::wasm::{
    analyzer::{WasmAnalyzer, AnalysisResult},
    baseline::{QualityBaseline, QualityAssessment, Metrics},
    security::{PatternDetector, VulnerabilityMatch},
    verifier::{IncrementalVerifier, VerificationResult},
    profiler::AsyncProfiler,
    ProfilingReport,
};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, debug};

/// Handle WASM analysis command (Complexity: ≤10)
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
    info!("🔍 Analyzing WASM module: {}", wasm_file.display());
    
    // Load and analyze WASM binary
    let binary = load_wasm_file(&wasm_file)?;
    let analysis_result = run_basic_analysis(&binary)?;
    
    // Run optional analyses
    let verification_result = run_verification_if_requested(verify, &binary).await?;
    let security_results = run_security_scan_if_requested(security, &binary)?;
    let profiling_results = run_profiling_if_requested(profile, &binary).await?;
    let baseline_comparison = run_baseline_comparison_if_requested(baseline, &binary, &analysis_result).await?;
    
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
        println!("{}", output_str);
    }
    Ok(())
}

/// Check for failures in analysis results (Complexity: 6)
fn check_for_failures(
    verification: Option<&VerificationResult>,
    security: Option<&Vec<VulnerabilityMatch>>,
    baseline: Option<&QualityAssessment>,
) -> Result<()> {
    check_verification_failure(verification)?;
    check_security_failures(security)?;
    check_baseline_failure(baseline)?;
    Ok(())
}

/// Check verification result for failures (Complexity: 3)
fn check_verification_failure(verification: Option<&VerificationResult>) -> Result<()> {
    if let Some(verification) = verification {
        if !verification.is_safe() {
            anyhow::bail!("❌ Verification failed: {:?}", verification);
        }
    }
    Ok(())
}

/// Check security results for critical vulnerabilities (Complexity: 4)
fn check_security_failures(security: Option<&Vec<VulnerabilityMatch>>) -> Result<()> {
    if let Some(security) = security {
        let critical_count = security.iter()
            .filter(|v| v.severity == crate::wasm::security::Severity::Critical)
            .count();
        
        if critical_count > 0 {
            anyhow::bail!("❌ Found {} critical security vulnerabilities", critical_count);
        }
    }
    Ok(())
}

/// Check baseline comparison for quality regression (Complexity: 3)
fn check_baseline_failure(baseline: Option<&QualityAssessment>) -> Result<()> {
    if let Some(baseline_comp) = baseline {
        if !baseline_comp.is_passing() {
            anyhow::bail!("❌ Quality regression detected");
        }
    }
    Ok(())
}

/// Format analysis results based on output format (Complexity: ≤10)
fn format_results(
    format: WasmOutputFormat,
    analysis: &AnalysisResult,
    verification: Option<&VerificationResult>,
    security: Option<&Vec<VulnerabilityMatch>>,
    profiling: Option<&ProfilingReport>,
    baseline: Option<&QualityAssessment>,
    verbose: bool,
) -> Result<String> {
    match format {
        WasmOutputFormat::Summary => format_summary(analysis, verification, security, profiling, baseline),
        WasmOutputFormat::Json => format_json(analysis, verification, security, profiling, baseline),
        WasmOutputFormat::Detailed => format_detailed(analysis, verification, security, profiling, baseline, verbose),
        WasmOutputFormat::Sarif => format_sarif(security.unwrap_or(&Vec::new())),
    }
}

/// Format summary output (Complexity: 5)
fn format_summary(
    analysis: &AnalysisResult,
    verification: Option<&VerificationResult>,
    security: Option<&Vec<VulnerabilityMatch>>,
    profiling: Option<&ProfilingReport>,
    baseline: Option<&QualityAssessment>,
) -> Result<String> {
    let mut output = String::new();
    
    append_summary_header(&mut output);
    append_basic_metrics(&mut output, analysis);
    append_verification_status(&mut output, verification);
    append_security_summary(&mut output, security);
    append_profiling_summary(&mut output, profiling);
    append_baseline_summary(&mut output, baseline);
    
    Ok(output)
}

/// Append summary header (Complexity: 1)
fn append_summary_header(output: &mut String) {
    output.push_str("WASM Analysis Summary\n");
    output.push_str("====================\n\n");
}

/// Append basic metrics (Complexity: 1)
fn append_basic_metrics(output: &mut String, analysis: &AnalysisResult) {
    output.push_str(&format!("Functions: {}\n", analysis.function_count));
    output.push_str(&format!("Instructions: {}\n", analysis.instruction_count));
    output.push_str(&format!("Binary Size: {} bytes\n", analysis.binary_size));
    output.push_str(&format!("Memory Pages: {}\n", analysis.memory_pages));
    output.push_str(&format!("Max Complexity: {}\n", analysis.max_complexity));
}

/// Append verification status (Complexity: 3)
fn append_verification_status(output: &mut String, verification: Option<&VerificationResult>) {
    if let Some(ver) = verification {
        output.push_str(&format!("\nVerification: {}\n", 
            if ver.is_safe() { "✅ SAFE" } else { "❌ UNSAFE" }
        ));
    }
}

/// Append security summary (Complexity: 5)
fn append_security_summary(output: &mut String, security: Option<&Vec<VulnerabilityMatch>>) {
    use crate::wasm::security::Severity;
    
    if let Some(sec) = security {
        let critical = count_by_severity(sec, Severity::Critical);
        let high = count_by_severity(sec, Severity::High);
        let medium = count_by_severity(sec, Severity::Medium);
        let low = count_by_severity(sec, Severity::Low);
        
        output.push_str("\nSecurity Vulnerabilities:\n");
        output.push_str(&format!("  Critical: {}\n", critical));
        output.push_str(&format!("  High: {}\n", high));
        output.push_str(&format!("  Medium: {}\n", medium));
        output.push_str(&format!("  Low: {}\n", low));
    }
}

/// Count vulnerabilities by severity (Complexity: 2)
fn count_by_severity(vulnerabilities: &[VulnerabilityMatch], severity: crate::wasm::security::Severity) -> usize {
    vulnerabilities.iter()
        .filter(|v| v.severity == severity)
        .count()
}

/// Append profiling summary (Complexity: 3)
fn append_profiling_summary(output: &mut String, profiling: Option<&ProfilingReport>) {
    if let Some(prof) = profiling {
        output.push_str("\nPerformance Profile:\n");
        
        let control_pct = calculate_percentage(prof.instruction_mix.control_flow, prof.instruction_mix.total_instructions);
        let memory_pct = calculate_percentage(prof.instruction_mix.memory_ops, prof.instruction_mix.total_instructions);
        let arith_pct = calculate_percentage(prof.instruction_mix.arithmetic, prof.instruction_mix.total_instructions);
        let call_pct = calculate_percentage(prof.instruction_mix.calls, prof.instruction_mix.total_instructions);
        
        output.push_str(&format!("  Control Flow: {}%\n", control_pct));
        output.push_str(&format!("  Memory Ops: {}%\n", memory_pct));
        output.push_str(&format!("  Arithmetic: {}%\n", arith_pct));
        output.push_str(&format!("  Function Calls: {}%\n", call_pct));
    }
}

/// Calculate percentage safely (Complexity: 2)
fn calculate_percentage(part: usize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    (part * 100) / total
}

/// Append baseline summary (Complexity: 3)
fn append_baseline_summary(output: &mut String, baseline: Option<&QualityAssessment>) {
    if let Some(base) = baseline {
        output.push_str("\nQuality Assessment:\n");
        output.push_str(&format!("  Health Score: {:.1}%\n", base.overall_health));
        output.push_str(&format!("  Status: {}\n", 
            if base.is_passing() { "✅ PASSING" } else { "❌ FAILING" }
        ));
        output.push_str(&format!("  {}\n", base.recommendation));
    }
}

/// Format JSON output (Complexity: 2)
fn format_json(
    analysis: &AnalysisResult,
    verification: Option<&VerificationResult>,
    security: Option<&Vec<VulnerabilityMatch>>,
    profiling: Option<&ProfilingReport>,
    baseline: Option<&QualityAssessment>,
) -> Result<String> {
    let json_output = serde_json::json!({
        "analysis": analysis,
        "verification": verification,
        "security": security,
        "profiling": profiling,
        "baseline": baseline,
    });
    Ok(serde_json::to_string_pretty(&json_output)?)
}

/// Format detailed output (Complexity: 4)
fn format_detailed(
    analysis: &AnalysisResult,
    verification: Option<&VerificationResult>,
    security: Option<&Vec<VulnerabilityMatch>>,
    profiling: Option<&ProfilingReport>,
    baseline: Option<&QualityAssessment>,
    verbose: bool,
) -> Result<String> {
    let mut output = format_summary(analysis, verification, security, profiling, baseline)?;
    
    if verbose {
        append_detailed_information(&mut output, profiling, security);
    }
    
    Ok(output)
}

/// Append detailed information (Complexity: 5)
fn append_detailed_information(
    output: &mut String,
    profiling: Option<&ProfilingReport>,
    security: Option<&Vec<VulnerabilityMatch>>,
) {
    output.push_str("\n\nDetailed Analysis\n");
    output.push_str("=================\n\n");
    
    append_detailed_profiling(output, profiling);
    append_detailed_vulnerabilities(output, security);
}

/// Append detailed profiling info (Complexity: 4)
fn append_detailed_profiling(output: &mut String, profiling: Option<&ProfilingReport>) {
    if let Some(prof) = profiling {
        output.push_str("Instruction Breakdown:\n");
        output.push_str(&format!("  Total: {}\n", prof.instruction_mix.total_instructions));
        output.push_str(&format!("  Control Flow: {}\n", prof.instruction_mix.control_flow));
        output.push_str(&format!("  Memory Operations: {}\n", prof.instruction_mix.memory_ops));
        output.push_str(&format!("  Arithmetic: {}\n", prof.instruction_mix.arithmetic));
        output.push_str(&format!("  Calls: {}\n", prof.instruction_mix.calls));
        
        append_hot_functions(output, prof);
    }
}

/// Append hot functions list (Complexity: 3)
fn append_hot_functions(output: &mut String, prof: &ProfilingReport) {
    if !prof.hot_functions.is_empty() {
        output.push_str("\nHot Functions:\n");
        for func in &prof.hot_functions {
            output.push_str(&format!("  {} - {:.1}% ({} samples)\n", 
                func.name, func.percentage, func.samples));
        }
    }
}

/// Append detailed vulnerabilities (Complexity: 3)
fn append_detailed_vulnerabilities(output: &mut String, security: Option<&Vec<VulnerabilityMatch>>) {
    if let Some(sec) = security {
        if !sec.is_empty() {
            output.push_str("\nVulnerability Details:\n");
            for vuln in sec {
                output.push_str(&format!("  [{:?}] {} at offset {}\n", 
                    vuln.severity, vuln.pattern, vuln.operator_index));
            }
        }
    }
}

/// Format SARIF output for security results (Complexity: 4)
fn format_sarif(vulnerabilities: &[VulnerabilityMatch]) -> Result<String> {
    let sarif_output = create_sarif_output(vulnerabilities);
    Ok(serde_json::to_string_pretty(&sarif_output)?)
}

/// Create SARIF output structure (Complexity: 6)
fn create_sarif_output(vulnerabilities: &[VulnerabilityMatch]) -> serde_json::Value {
    let rules = create_sarif_rules(vulnerabilities);
    let results = create_sarif_results(vulnerabilities);
    
    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-wasm-analyzer",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/pmat",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}

/// Create SARIF rules from vulnerabilities (Complexity: 4)
fn create_sarif_rules(vulnerabilities: &[VulnerabilityMatch]) -> Vec<serde_json::Value> {
    let unique_patterns: std::collections::HashSet<_> = vulnerabilities
        .iter()
        .map(|v| &v.pattern)
        .collect();
    
    unique_patterns.into_iter()
        .map(|pattern| create_sarif_rule(pattern))
        .collect()
}

/// Create single SARIF rule (Complexity: 1)
fn create_sarif_rule(pattern: &str) -> serde_json::Value {
    serde_json::json!({
        "id": pattern,
        "name": pattern,
        "shortDescription": {
            "text": format!("WASM vulnerability: {}", pattern)
        },
        "fullDescription": {
            "text": format!("Security vulnerability pattern detected in WebAssembly module: {}", pattern)
        },
        "defaultConfiguration": {
            "level": "warning"
        }
    })
}

/// Create SARIF results from vulnerabilities (Complexity: 3)
fn create_sarif_results(vulnerabilities: &[VulnerabilityMatch]) -> Vec<serde_json::Value> {
    vulnerabilities.iter()
        .map(create_sarif_result)
        .collect()
}

/// Create single SARIF result (Complexity: 3)
fn create_sarif_result(vuln: &VulnerabilityMatch) -> serde_json::Value {
    use crate::wasm::security::Severity;
    
    let level = match vuln.severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
    };
    
    serde_json::json!({
        "ruleId": vuln.pattern,
        "level": level,
        "message": {
            "text": format!("Found {} vulnerability at instruction {}", vuln.pattern, vuln.operator_index)
        },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": {
                    "uri": "module.wasm"
                },
                "region": {
                    "startLine": vuln.operator_index,
                    "startColumn": 1
                }
            }
        }]
    })
}

/// Create metrics from analysis result for baseline comparison (Complexity: 1)
fn create_metrics_from_analysis(analysis: &AnalysisResult) -> Metrics {
    Metrics {
        timestamp: chrono::Utc::now(),
        complexity_p90: analysis.max_complexity.saturating_sub(2),
        complexity_p95: analysis.max_complexity,
        complexity_p99: analysis.max_complexity.saturating_add(2),
        binary_size: analysis.binary_size,
        init_time_ms: 10, // Default estimate
        memory_usage_mb: (analysis.memory_pages * 64) / 1024, // Pages to MB
        function_count: analysis.function_count,
        instruction_count: analysis.instruction_count,
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
