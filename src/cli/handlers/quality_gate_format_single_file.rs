/// Toyota Way: Single Responsibility - Format single file quality gate output
/// Extracted from stubs.rs to reduce complexity and improve maintainability
///
/// # Parameters
///
/// * `single_file` - Path to the file being analyzed
/// * `results` - Quality gate results with pass/fail status
/// * `violations` - List of quality violations found
/// * `format` - Output format for the results
///
/// # Returns
///
/// * `Ok(String)` - Formatted output string
/// * `Err(anyhow::Error)` - Formatting failed
pub fn format_single_file_output(
    single_file: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
) -> Result<String> {
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    match format {
        QualityGateOutputFormat::Json => Ok(serde_json::to_string_pretty(&json!({
            "file": single_file,
            "passed": results.passed,
            "results": results,
            "violations": violations,
        }))?),
        QualityGateOutputFormat::Summary
        | QualityGateOutputFormat::Markdown
        | QualityGateOutputFormat::Detailed
        | QualityGateOutputFormat::Human
        | QualityGateOutputFormat::Junit => {
            Ok(format_single_file_summary(single_file, results, violations))
        }
    }
}

/// Toyota Way: Extract Method - Format single file summary report (complexity ≤8)
/// Creates a comprehensive markdown report for single file quality gate results
///
/// # Parameters
///
/// * `file_path` - Path to the analyzed file
/// * `results` - Quality gate results summary
/// * `violations` - Detailed list of violations
///
/// # Returns
///
/// Formatted markdown string with quality gate report
#[must_use]
pub fn format_single_file_summary(
    file_path: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> String {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    let mut output = String::new();

    // Header with file path
    output.push_str(&format!(
        "# Quality Gate Report: {}\n\n",
        file_path.display()
    ));

    // Pass/Fail status with emoji
    if results.passed {
        output.push_str("✅ **Quality Gate: PASSED**\n\n");
    } else {
        output.push_str("❌ **Quality Gate: FAILED**\n\n");
    }

    // Summary section with metrics
    add_summary_section(&mut output, results);

    // Violations section if any exist
    if !violations.is_empty() {
        add_violations_section(&mut output, violations);
    }

    output
}

/// Toyota Way: Extract Method - Add summary section (complexity ≤3)
fn add_summary_section(output: &mut String, results: &QualityGateResults) {
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Total Violations: {}\n",
        results.total_violations
    ));
    output.push_str(&format!(
        "- Complexity Issues: {}\n",
        results.complexity_violations
    ));
    output.push_str(&format!("- Dead Code: {}\n", results.dead_code_violations));
    output.push_str(&format!(
        "- Technical Debt (SATD): {}\n",
        results.satd_violations
    ));
    output.push_str(&format!(
        "- Security Issues: {}\n",
        results.security_violations
    ));
}

/// Toyota Way: Extract Method - Add violations section (complexity ≤8)
fn add_violations_section(output: &mut String, violations: &[QualityViolation]) {
    output.push_str("\n## Violations\n\n");

    // Group violations by type for better organization
    let mut by_type: HashMap<String, Vec<&QualityViolation>> = HashMap::new();
    for violation in violations {
        by_type
            .entry(violation.check_type.clone())
            .or_default()
            .push(violation);
    }

    // Format each violation type section
    for (check_type, type_violations) in by_type {
        output.push_str(&format!(
            "### {} ({})\n\n",
            check_type.to_uppercase(),
            type_violations.len()
        ));

        for violation in type_violations {
            add_violation_entry(output, violation);
        }
        output.push('\n');
    }
}

/// Toyota Way: Extract Method - Add single violation entry with file location (complexity ≤5)
fn add_violation_entry(output: &mut String, violation: &QualityViolation) {
    let severity_icon = get_severity_icon(&violation.severity);

    // Format file path - use short relative path for readability
    let file_display = if violation.file.is_empty() {
        String::new()
    } else {
        let path = std::path::Path::new(&violation.file);
        let short_path = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| violation.file.clone());
        format!(" {}", short_path)
    };

    if let Some(line) = violation.line {
        output.push_str(&format!(
            "- {}{}:{}: {}\n",
            severity_icon, file_display, line, violation.message
        ));
    } else if !violation.file.is_empty() {
        output.push_str(&format!(
            "- {}{}: {}\n",
            severity_icon, file_display, violation.message
        ));
    } else {
        output.push_str(&format!("- {} {}\n", severity_icon, violation.message));
    }
}

/// Toyota Way: Extract Method - Get severity icon (complexity ≤2)
fn get_severity_icon(severity: &str) -> &'static str {
    match severity {
        "error" => "🔴",
        "warning" => "🟡",
        _ => "🟢",
    }
}
