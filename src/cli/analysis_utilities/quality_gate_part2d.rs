fn format_single_file_summary(
    file_path: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> String {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    let mut output = String::new();

    format_report_header(&mut output, file_path, results.passed);
    format_results_summary(&mut output, results);

    if !violations.is_empty() {
        format_violations_section(&mut output, violations);
    }

    output
}

/// Format the report header with title and pass/fail status
fn format_report_header(output: &mut String, file_path: &Path, passed: bool) {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    output.push_str(&format!(
        "# Quality Gate Report: {}\n\n",
        file_path.display()
    ));

    if passed {
        output.push_str("✅ **Quality Gate: PASSED**\n\n");
    } else {
        output.push_str("❌ **Quality Gate: FAILED**\n\n");
    }
}

/// Format the summary section with violation counts
fn format_results_summary(output: &mut String, results: &QualityGateResults) {
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

/// Format the violations section grouped by type
fn format_violations_section(output: &mut String, violations: &[QualityViolation]) {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    use std::collections::HashMap;

    output.push_str("\n## Violations\n\n");

    // Group violations by type
    let mut by_type: HashMap<String, Vec<&QualityViolation>> = HashMap::new();
    for violation in violations {
        by_type
            .entry(violation.check_type.clone())
            .or_default()
            .push(violation);
    }

    for (check_type, type_violations) in by_type {
        format_violation_type_group(output, &check_type, &type_violations);
    }
}

/// Format a single violation type group
fn format_violation_type_group(
    output: &mut String,
    check_type: &str,
    violations: &[&QualityViolation],
) {
    debug_assert!(!check_type.is_empty(), "check_type must not be empty");
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    output.push_str(&format!(
        "### {} ({})\n\n",
        check_type.to_uppercase(),
        violations.len()
    ));

    for violation in violations {
        format_single_violation(output, violation);
    }
    output.push('\n');
}

/// Format a single violation with severity icon, file path, and location
fn format_single_violation(output: &mut String, violation: &QualityViolation) {
    let severity_icon = get_severity_icon(&violation.severity);

    // Format file path - use short relative path if possible
    let file_display = if violation.file.is_empty() {
        String::new()
    } else {
        // Extract just the filename or short path for display
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

/// Get the appropriate icon for violation severity
pub fn get_severity_icon(severity: &str) -> &'static str {
    debug_assert!(!severity.is_empty(), "severity must not be empty");
    match severity {
        "error" => "🔴",
        "warning" => "🟡",
        _ => "🟢",
    }
}
