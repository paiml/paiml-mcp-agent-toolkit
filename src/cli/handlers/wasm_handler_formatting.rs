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
        WasmOutputFormat::Summary => {
            format_summary(analysis, verification, security, profiling, baseline)
        }
        WasmOutputFormat::Json => {
            format_json(analysis, verification, security, profiling, baseline)
        }
        WasmOutputFormat::Detailed => format_detailed(
            analysis,
            verification,
            security,
            profiling,
            baseline,
            verbose,
        ),
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
        output.push_str(&format!(
            "\nVerification: {}\n",
            if ver.is_safe() {
                "✅ SAFE"
            } else {
                "❌ UNSAFE"
            }
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
        output.push_str(&format!("  Critical: {critical}\n"));
        output.push_str(&format!("  High: {high}\n"));
        output.push_str(&format!("  Medium: {medium}\n"));
        output.push_str(&format!("  Low: {low}\n"));
    }
}

/// Count vulnerabilities by severity (Complexity: 2)
fn count_by_severity(
    vulnerabilities: &[VulnerabilityMatch],
    severity: crate::wasm::security::Severity,
) -> usize {
    vulnerabilities
        .iter()
        .filter(|v| v.severity == severity)
        .count()
}

/// Append profiling summary (Complexity: 3)
fn append_profiling_summary(output: &mut String, profiling: Option<&ProfilingReport>) {
    if let Some(prof) = profiling {
        output.push_str("\nPerformance Profile:\n");

        let control_pct = calculate_percentage(
            prof.instruction_mix.control_flow,
            prof.instruction_mix.total_instructions,
        );
        let memory_pct = calculate_percentage(
            prof.instruction_mix.memory_ops,
            prof.instruction_mix.total_instructions,
        );
        let arith_pct = calculate_percentage(
            prof.instruction_mix.arithmetic,
            prof.instruction_mix.total_instructions,
        );
        let call_pct = calculate_percentage(
            prof.instruction_mix.calls,
            prof.instruction_mix.total_instructions,
        );

        output.push_str(&format!("  Control Flow: {control_pct}%\n"));
        output.push_str(&format!("  Memory Ops: {memory_pct}%\n"));
        output.push_str(&format!("  Arithmetic: {arith_pct}%\n"));
        output.push_str(&format!("  Function Calls: {call_pct}%\n"));
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
        output.push_str(&format!(
            "  Status: {}\n",
            if base.is_passing() {
                "✅ PASSING"
            } else {
                "❌ FAILING"
            }
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
        output.push_str(&format!(
            "  Total: {}\n",
            prof.instruction_mix.total_instructions
        ));
        output.push_str(&format!(
            "  Control Flow: {}\n",
            prof.instruction_mix.control_flow
        ));
        output.push_str(&format!(
            "  Memory Operations: {}\n",
            prof.instruction_mix.memory_ops
        ));
        output.push_str(&format!(
            "  Arithmetic: {}\n",
            prof.instruction_mix.arithmetic
        ));
        output.push_str(&format!("  Calls: {}\n", prof.instruction_mix.calls));

        append_hot_functions(output, prof);
    }
}

/// Append hot functions list (Complexity: 3)
fn append_hot_functions(output: &mut String, prof: &ProfilingReport) {
    if !prof.hot_functions.is_empty() {
        output.push_str("\nHot Functions:\n");
        for func in &prof.hot_functions {
            output.push_str(&format!(
                "  {} - {:.1}% ({} samples)\n",
                func.name, func.percentage, func.samples
            ));
        }
    }
}

/// Append detailed vulnerabilities (Complexity: 3)
fn append_detailed_vulnerabilities(
    output: &mut String,
    security: Option<&Vec<VulnerabilityMatch>>,
) {
    if let Some(sec) = security {
        if !sec.is_empty() {
            output.push_str("\nVulnerability Details:\n");
            for vuln in sec {
                output.push_str(&format!(
                    "  [{:?}] {} at offset {}\n",
                    vuln.severity, vuln.pattern, vuln.operator_index
                ));
            }
        }
    }
}
