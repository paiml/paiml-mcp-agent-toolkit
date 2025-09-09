//! WASM analysis handler for CLI
//! GREEN Phase: Implementation to make tests pass

use crate::cli::WasmOutputFormat;
use crate::wasm::{
    analyzer::WasmAnalyzer,
    baseline::QualityBaseline,
    security::PatternDetector,
    verifier::IncrementalVerifier,
    profiler::AsyncProfiler,
};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, debug};

/// Handle WASM analysis command
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
    
    // Read WASM binary
    let binary = std::fs::read(&wasm_file)?;
    debug!("Loaded WASM binary: {} bytes", binary.len());
    
    // Create analyzer
    let mut analyzer = WasmAnalyzer::new()?;
    
    // Run basic analysis
    let analysis_result = analyzer.analyze(&binary)?;
    
    // Optional: Run verification
    let verification_result = if verify {
        info!("🔒 Running formal verification...");
        let verifier = IncrementalVerifier::new()?;
        Some(verifier.verify_module(&binary)?)
    } else {
        None
    };
    
    // Optional: Run security scanning
    let security_results = if security {
        info!("🛡️ Running security vulnerability scanning...");
        let mut detector = PatternDetector::new();
        for payload in wasmparser::Parser::new(0).parse_all(&binary) {
            let payload = payload?;
            detector.scan(&payload)?;
        }
        Some(detector.finalize())
    } else {
        None
    };
    
    // Optional: Run profiling
    let profiling_results = if profile {
        info!("📊 Running performance profiling...");
        let profiler = AsyncProfiler::new();
        Some(profiler.profile_module(&binary).await?)
    } else {
        None
    };
    
    // Optional: Compare with baseline
    let baseline_comparison = if let Some(baseline_path) = baseline {
        info!("📈 Comparing with baseline: {}", baseline_path.display());
        let baseline_binary = std::fs::read(&baseline_path)?;
        let baseline_analyzer = WasmAnalyzer::new()?;
        let baseline_result = baseline_analyzer.analyze(&baseline_binary)?;
        
        // Create baseline comparison
        let release_metrics = create_metrics_from_analysis(&baseline_result);
        let current_metrics = create_metrics_from_analysis(&analysis_result);
        
        let quality_baseline = QualityBaseline::new(release_metrics.clone(), release_metrics);
        Some(quality_baseline.evaluate(&current_metrics))
    } else {
        None
    };
    
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
    
    // Write output
    if let Some(output_path) = output {
        std::fs::write(&output_path, &output_str)?;
        info!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{}", output_str);
    }
    
    // Check for failures
    if let Some(verification) = &verification_result {
        if !verification.is_safe() {
            anyhow::bail!("❌ Verification failed: {:?}", verification);
        }
    }
    
    if let Some(security) = &security_results {
        let critical_count = security.iter()
            .filter(|v| v.severity == crate::wasm::security::Severity::Critical)
            .count();
        if critical_count > 0 {
            anyhow::bail!("❌ Found {} critical security vulnerabilities", critical_count);
        }
    }
    
    if let Some(baseline_comp) = &baseline_comparison {
        if !baseline_comp.is_passing() {
            anyhow::bail!("❌ Quality regression detected");
        }
    }
    
    info!("✅ WASM analysis complete");
    Ok(())
}

/// Format analysis results based on output format
fn format_results(
    format: WasmOutputFormat,
    analysis: &crate::wasm::analyzer::AnalysisResult,
    verification: Option<&crate::wasm::verifier::VerificationResult>,
    security: Option<&Vec<crate::wasm::security::VulnerabilityMatch>>,
    profiling: Option<&crate::wasm::ProfilingReport>,
    baseline: Option<&crate::wasm::baseline::QualityAssessment>,
    verbose: bool,
) -> Result<String> {
    use crate::wasm::security::Severity;
    
    match format {
        WasmOutputFormat::Summary => {
            let mut output = String::new();
            
            // Basic metrics
            output.push_str("WASM Analysis Summary\n");
            output.push_str("====================\n\n");
            
            output.push_str(&format!("Functions: {}\n", analysis.function_count));
            output.push_str(&format!("Instructions: {}\n", analysis.instruction_count));
            output.push_str(&format!("Binary Size: {} bytes\n", analysis.binary_size));
            output.push_str(&format!("Memory Pages: {}\n", analysis.memory_pages));
            output.push_str(&format!("Max Complexity: {}\n", analysis.max_complexity));
            
            // Verification status
            if let Some(ver) = verification {
                output.push_str(&format!("\nVerification: {}\n", 
                    if ver.is_safe() { "✅ SAFE" } else { "❌ UNSAFE" }
                ));
            }
            
            // Security summary
            if let Some(sec) = security {
                let critical = sec.iter().filter(|v| v.severity == Severity::Critical).count();
                let high = sec.iter().filter(|v| v.severity == Severity::High).count();
                let medium = sec.iter().filter(|v| v.severity == Severity::Medium).count();
                let low = sec.iter().filter(|v| v.severity == Severity::Low).count();
                
                output.push_str(&format!("\nSecurity Vulnerabilities:\n"));
                output.push_str(&format!("  Critical: {}\n", critical));
                output.push_str(&format!("  High: {}\n", high));
                output.push_str(&format!("  Medium: {}\n", medium));
                output.push_str(&format!("  Low: {}\n", low));
            }
            
            // Profiling summary
            if let Some(prof) = profiling {
                output.push_str(&format!("\nPerformance Profile:\n"));
                output.push_str(&format!("  Control Flow: {}%\n", 
                    (prof.instruction_mix.control_flow * 100 / prof.instruction_mix.total_instructions)));
                output.push_str(&format!("  Memory Ops: {}%\n",
                    (prof.instruction_mix.memory_ops * 100 / prof.instruction_mix.total_instructions)));
                output.push_str(&format!("  Arithmetic: {}%\n",
                    (prof.instruction_mix.arithmetic * 100 / prof.instruction_mix.total_instructions)));
                output.push_str(&format!("  Function Calls: {}%\n",
                    (prof.instruction_mix.calls * 100 / prof.instruction_mix.total_instructions)));
            }
            
            // Baseline comparison
            if let Some(base) = baseline {
                output.push_str(&format!("\nQuality Assessment:\n"));
                output.push_str(&format!("  Health Score: {:.1}%\n", base.overall_health));
                output.push_str(&format!("  Status: {}\n", 
                    if base.is_passing() { "✅ PASSING" } else { "❌ FAILING" }
                ));
                output.push_str(&format!("  {}\n", base.recommendation));
            }
            
            Ok(output)
        }
        
        WasmOutputFormat::Json => {
            let json_output = serde_json::json!({
                "analysis": analysis,
                "verification": verification,
                "security": security,
                "profiling": profiling,
                "baseline": baseline,
            });
            Ok(serde_json::to_string_pretty(&json_output)?)
        }
        
        WasmOutputFormat::Detailed => {
            let mut output = format_results(WasmOutputFormat::Summary, 
                analysis, verification, security, profiling, baseline, verbose)?;
            
            // Add detailed information
            if verbose {
                output.push_str("\n\nDetailed Analysis\n");
                output.push_str("=================\n\n");
                
                // Detailed instruction breakdown
                if let Some(prof) = profiling {
                    output.push_str("Instruction Breakdown:\n");
                    output.push_str(&format!("  Total: {}\n", prof.instruction_mix.total_instructions));
                    output.push_str(&format!("  Control Flow: {}\n", prof.instruction_mix.control_flow));
                    output.push_str(&format!("  Memory Operations: {}\n", prof.instruction_mix.memory_ops));
                    output.push_str(&format!("  Arithmetic: {}\n", prof.instruction_mix.arithmetic));
                    output.push_str(&format!("  Calls: {}\n", prof.instruction_mix.calls));
                    
                    // Hot functions
                    if !prof.hot_functions.is_empty() {
                        output.push_str("\nHot Functions:\n");
                        for func in &prof.hot_functions {
                            output.push_str(&format!("  {} - {:.1}% ({} samples)\n", 
                                func.name, func.percentage, func.samples));
                        }
                    }
                }
                
                // Detailed vulnerabilities
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
            
            Ok(output)
        }
        
        WasmOutputFormat::Sarif => {
            // SARIF format for security results
            let sarif_output = create_sarif_output(security.unwrap_or(&Vec::new()));
            Ok(serde_json::to_string_pretty(&sarif_output)?)
        }
    }
}

/// Create metrics from analysis result for baseline comparison
fn create_metrics_from_analysis(
    analysis: &crate::wasm::analyzer::AnalysisResult,
) -> crate::wasm::baseline::Metrics {
    crate::wasm::baseline::Metrics {
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

/// Create SARIF output for security results
fn create_sarif_output(
    vulnerabilities: &[crate::wasm::security::VulnerabilityMatch],
) -> serde_json::Value {
    use crate::wasm::security::Severity;
    
    let rules: Vec<serde_json::Value> = vulnerabilities
        .iter()
        .map(|v| &v.pattern)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .map(|pattern| {
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
        })
        .collect();
    
    let results: Vec<serde_json::Value> = vulnerabilities
        .iter()
        .map(|v| {
            let level = match v.severity {
                Severity::Critical => "error",
                Severity::High => "error",
                Severity::Medium => "warning",
                Severity::Low => "note",
            };
            
            serde_json::json!({
                "ruleId": v.pattern,
                "level": level,
                "message": {
                    "text": format!("Found {} vulnerability at instruction {}", v.pattern, v.operator_index)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": "module.wasm"
                        },
                        "region": {
                            "startLine": v.operator_index,
                            "startColumn": 1
                        }
                    }
                }]
            })
        })
        .collect();
    
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