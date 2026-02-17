/// Helper for provability check execution
async fn execute_provability_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let threshold = load_provability_threshold(project_path);
    execute_quality_check_template(
        check_provability(project_path, threshold),
        |count| results.provability_violations = count,
        violations,
    )
    .await
}

/// Runs all project-wide checks
async fn run_all_project_checks(
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    // Run all checks
    eprint!("  🔍 Checking complexity...");
    let start = if perf { Some(Instant::now()) } else { None };
    let complexity_violations = check_complexity(project_path, max_complexity_p99).await?;
    results.complexity_violations = complexity_violations.len();
    violations.extend(complexity_violations);
    if let Some(s) = start {
        eprintln!(
            " {} violations found ({:.3}s)",
            results.complexity_violations,
            s.elapsed().as_secs_f64()
        );
    } else {
        eprintln!(" {} violations found", results.complexity_violations);
    }

    // Macro to handle timing for each check
    macro_rules! run_check {
        ($name:expr, $check_expr:expr, $result_field:ident) => {{
            eprint!("  🔍 Checking {}...", $name);
            let start = if perf { Some(Instant::now()) } else { None };
            let check_violations = $check_expr.await?;
            results.$result_field = check_violations.len();
            violations.extend(check_violations);
            if let Some(s) = start {
                eprintln!(
                    " {} violations found ({:.3}s)",
                    results.$result_field,
                    s.elapsed().as_secs_f64()
                );
            } else {
                eprintln!(" {} violations found", results.$result_field);
            }
        }};
    }

    run_check!(
        "dead code",
        check_dead_code(project_path, max_dead_code),
        dead_code_violations
    );
    run_check!("technical debt", check_satd(project_path), satd_violations);
    run_entropy_check_gated(project_path, min_entropy, violations, results, perf).await?;
    run_check!(
        "security",
        check_security(project_path),
        security_violations
    );
    run_check!(
        "duplicates",
        check_duplicates(project_path),
        duplicate_violations
    );
    run_check!(
        "test coverage",
        check_coverage(project_path, 80.0),
        coverage_violations
    );
    run_check!(
        "documentation sections",
        check_sections(project_path),
        section_violations
    );
    let provability_threshold = load_provability_threshold(project_path);
    run_check!(
        "provability",
        check_provability(project_path, provability_threshold),
        provability_violations
    );

    Ok(())
}

/// Run entropy check with gate config (#220): enabled, excludes, max_violations.
async fn run_entropy_check_gated(
    project_path: &Path,
    min_entropy: f64,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    let gate_config = load_entropy_gate_config(project_path);
    if !gate_config.enabled {
        eprintln!("  \u{23ed}\u{fe0f}  Skipping code entropy (disabled via .pmat-gates.toml)");
        return Ok(());
    }

    let ent_threshold = load_entropy_threshold(project_path, min_entropy);
    let mut ent_excludes = load_entropy_exclude_paths(project_path);
    merge_excludes(&mut ent_excludes, &gate_config.exclude);

    eprint!("  \u{1f50d} Checking code entropy...");
    let start = if perf { Some(Instant::now()) } else { None };
    let ent_violations =
        check_entropy_with_excludes(project_path, ent_threshold, &ent_excludes).await?;
    results.entropy_violations = ent_violations.len();
    violations.extend(ent_violations);

    if let Some(s) = start {
        eprintln!(
            " {} violations found ({:.3}s)",
            results.entropy_violations,
            s.elapsed().as_secs_f64()
        );
    } else {
        eprintln!(" {} violations found", results.entropy_violations);
    }

    // Apply max_violations threshold (#220)
    if let Some(max) = gate_config.max_violations {
        if results.entropy_violations <= max {
            violations.retain(|v| v.check_type != "entropy");
            results.entropy_violations = 0;
        }
    }

    Ok(())
}

/// Merge exclude patterns, deduplicating.
fn merge_excludes(base: &mut Vec<String>, extra: &[String]) {
    for pattern in extra {
        if !base.contains(pattern) {
            base.push(pattern.clone());
        }
    }
}

/// Formats and outputs project results
async fn output_project_results(
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = format_quality_gate_output(results, violations, format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Quality gate report written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Prints the final quality gate status
fn print_quality_gate_final_status(results: &QualityGateResults, violations: &[QualityViolation]) {
    if results.passed {
        eprintln!("\n✅ Quality gate PASSED");
    } else {
        eprintln!("\n⚠️ Quality gate found {} violations", violations.len());
    }
}

/// Handles the exit status based on quality gate results
fn handle_quality_gate_exit_status(fail_on_violation: bool, passed: bool) {
    if fail_on_violation && !passed {
        eprintln!("\n❌ Quality gate FAILED");
        std::process::exit(1);
    }
}


// Single file quality check functions - extracted for file health (CB-040)
async fn check_single_file_complexity(
    project_path: &Path,
    file_path: &Path,
    max_complexity_p99: u32,
) -> Result<Vec<QualityViolation>> {
    let abs_file_path = resolve_absolute_file_path(project_path, file_path);
    validate_file_exists(&abs_file_path)?;

    let mut violations = Vec::new();
    analyze_file_complexity(
        &abs_file_path,
        file_path,
        max_complexity_p99,
        &mut violations,
    )
    .await?;

    Ok(violations)
}

/// Resolve file path to absolute path
fn resolve_absolute_file_path(project_path: &Path, file_path: &Path) -> PathBuf {
    if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    }
}

/// Validate that file exists
fn validate_file_exists(abs_file_path: &Path) -> Result<()> {
    if !abs_file_path.exists() {
        return Err(anyhow::anyhow!(
            "File not found: {}",
            abs_file_path.display()
        ));
    }
    Ok(())
}

/// Analyze file complexity based on file extension
async fn analyze_file_complexity(
    abs_file_path: &Path,
    original_path: &Path,
    max_complexity: u32,
    violations: &mut Vec<QualityViolation>,
) -> Result<()> {
    if let Some(ext) = abs_file_path.extension() {
        if ext == "rs" {
            analyze_rust_file_complexity(abs_file_path, original_path, max_complexity, violations)
                .await?;
        }
        // Add support for other languages as needed
    }
    Ok(())
}

/// Analyze Rust file complexity and generate violations
async fn analyze_rust_file_complexity(
    abs_file_path: &Path,
    original_path: &Path,
    max_complexity: u32,
    violations: &mut Vec<QualityViolation>,
) -> Result<()> {
    use crate::services::ast_rust::analyze_rust_file_with_complexity;

    let metrics = analyze_rust_file_with_complexity(abs_file_path).await?;

    for func in &metrics.functions {
        if function_exceeds_complexity_threshold(func, max_complexity) {
            violations.push(create_complexity_violation(
                func,
                original_path,
                max_complexity,
            ));
        }
    }

    Ok(())
}

/// Check if function exceeds complexity threshold
fn function_exceeds_complexity_threshold(
    func: &crate::services::complexity::FunctionComplexity,
    max_complexity: u32,
) -> bool {
    func.metrics.cyclomatic > max_complexity as u16
}

/// Create complexity violation for a function
fn create_complexity_violation(
    func: &crate::services::complexity::FunctionComplexity,
    file_path: &Path,
    max_complexity: u32,
) -> QualityViolation {
    QualityViolation {
        check_type: "complexity".to_string(),
        severity: "error".to_string(),
        file: file_path.to_string_lossy().to_string(),
        line: Some(func.line_start as usize),
        message: format!(
            "Function '{}' has cyclomatic complexity {} (max: {})",
            func.name, func.metrics.cyclomatic, max_complexity
        ),
    }
}

async fn check_single_file_dead_code(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations); // No violations if file doesn't exist
    }

    // Read file content
    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    // Check for common dead code patterns
    let dead_code_patterns = vec![
        (r"#\[allow\(dead_code\)\]", "Dead code attribute found"),
        (r"^\s*//\s*fn\s+\w+", "Commented out function"),
        (r"^\s*//\s*struct\s+\w+", "Commented out struct"),
        (r"^\s*//\s*impl\s+", "Commented out implementation"),
    ];

    for (pattern_str, message) in dead_code_patterns {
        let regex = Regex::new(pattern_str)?;
        for (line_no, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                violations.push(QualityViolation {
                    check_type: "dead_code".to_string(),
                    severity: "warning".to_string(),
                    file: file_path.to_string_lossy().to_string(),
                    line: Some(line_no + 1),
                    message: message.to_string(),
                });
            }
        }
    }

    Ok(violations)
}

async fn check_single_file_satd(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();
    let satd_pattern = Regex::new(r"(?i)\b(TODO|FIXME|HACK|XXX|BUG|REFACTOR):\s*(.+)")?;

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations);
    }

    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    for (line_no, line) in content.lines().enumerate() {
        if let Some(captures) = satd_pattern.captures(line) {
            let satd_type = captures
                .get(1)
                .expect("Match group 1 exists for successful regex match")
                .as_str();
            let text = captures
                .get(2)
                .expect("Match group 2 exists for successful regex match")
                .as_str();

            violations.push(QualityViolation {
                check_type: "satd".to_string(),
                severity: "warning".to_string(),
                file: file_path.to_string_lossy().to_string(),
                line: Some(line_no + 1),
                message: format!("Self-admitted technical debt: {satd_type} - {text}"),
            });
        }
    }

    Ok(violations)
}

async fn check_single_file_security(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();

    // Security patterns to check
    let security_patterns = vec![
        (
            r#"(?i)password\s*=\s*["'][^"']+["']"#,
            "Hardcoded password detected",
        ),
        (
            r#"(?i)api_key\s*=\s*["'][^"']+["']"#,
            "Hardcoded API key detected",
        ),
        (
            r#"(?i)secret\s*=\s*["'][^"']+["']"#,
            "Hardcoded secret detected",
        ),
        (
            r#"(?i)token\s*=\s*["'][^"']+["']"#,
            "Hardcoded token detected",
        ),
        (r"(?i)unsafe\s*\{", "Unsafe code block detected"),
        (
            r"std::env::var\(.*\)\.unwrap\(\)",
            "Unsafe environment variable access",
        ),
    ];

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations);
    }

    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    for (pattern_str, message) in security_patterns {
        let regex = Regex::new(pattern_str)?;
        for (line_no, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                violations.push(QualityViolation {
                    check_type: "security".to_string(),
                    severity: "error".to_string(),
                    file: file_path.to_string_lossy().to_string(),
                    line: Some(line_no + 1),
                    message: message.to_string(),
                });
            }
        }
    }

    Ok(violations)
}

fn format_single_file_summary(
    file_path: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> String {
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
    match severity {
        "error" => "🔴",
        "warning" => "🟡",
        _ => "🟢",
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_gate_unit_tests {
    use super::*;

    // ===================
    // get_severity_icon Tests
    // ===================

    #[test]
    fn test_get_severity_icon_error() {
        assert_eq!(get_severity_icon("error"), "🔴");
    }

    #[test]
    fn test_get_severity_icon_warning() {
        assert_eq!(get_severity_icon("warning"), "🟡");
    }

    #[test]
    fn test_get_severity_icon_other() {
        assert_eq!(get_severity_icon("info"), "🟢");
        assert_eq!(get_severity_icon("note"), "🟢");
        assert_eq!(get_severity_icon("suggestion"), "🟢");
        assert_eq!(get_severity_icon(""), "🟢");
    }

    // ===================
    // get_check_message Tests
    // ===================

    #[test]
    fn test_get_check_message_complexity() {
        let result = get_check_message(&QualityCheckType::Complexity);
        assert_eq!(result, Some("Complexity analysis"));
    }

    #[test]
    fn test_get_check_message_dead_code() {
        let result = get_check_message(&QualityCheckType::DeadCode);
        assert_eq!(result, Some("Dead code detection"));
    }

    #[test]
    fn test_get_check_message_satd() {
        let result = get_check_message(&QualityCheckType::Satd);
        assert_eq!(result, Some("Self-admitted technical debt (SATD)"));
    }

    #[test]
    fn test_get_check_message_security() {
        let result = get_check_message(&QualityCheckType::Security);
        assert_eq!(result, Some("Security vulnerabilities"));
    }

    #[test]
    fn test_get_check_message_entropy() {
        let result = get_check_message(&QualityCheckType::Entropy);
        assert_eq!(result, Some("Code entropy"));
    }

    #[test]
    fn test_get_check_message_duplicates() {
        let result = get_check_message(&QualityCheckType::Duplicates);
        assert_eq!(result, Some("Duplicate code"));
    }

    #[test]
    fn test_get_check_message_coverage() {
        let result = get_check_message(&QualityCheckType::Coverage);
        assert_eq!(result, Some("Test coverage"));
    }

    #[test]
    fn test_get_check_message_all() {
        let result = get_check_message(&QualityCheckType::All);
        assert!(result.is_none());
    }

    // ===================
    // format_report_header Tests
    // ===================

    #[test]
    fn test_format_report_header_passed() {
        let mut output = String::new();
        format_report_header(&mut output, Path::new("src/test.rs"), true);
        assert!(output.contains("Quality Gate Report: src/test.rs"));
        assert!(output.contains("PASSED"));
        assert!(output.contains("✅"));
    }

    #[test]
    fn test_format_report_header_failed() {
        let mut output = String::new();
        format_report_header(&mut output, Path::new("src/main.rs"), false);
        assert!(output.contains("Quality Gate Report: src/main.rs"));
        assert!(output.contains("FAILED"));
        assert!(output.contains("❌"));
    }

    // ===================
    // format_results_summary Tests
    // ===================

    #[test]
    fn test_format_results_summary_zeros() {
        let results = QualityGateResults::default();

        let mut output = String::new();
        format_results_summary(&mut output, &results);

        assert!(output.contains("## Summary"));
        assert!(output.contains("Total Violations: 0"));
        assert!(output.contains("Complexity Issues: 0"));
        assert!(output.contains("Dead Code: 0"));
        assert!(output.contains("Technical Debt (SATD): 0"));
        assert!(output.contains("Security Issues: 0"));
    }

    #[test]
    fn test_format_results_summary_with_violations() {
        let mut results = QualityGateResults::default();
        results.passed = false;
        results.total_violations = 10;
        results.complexity_violations = 3;
        results.dead_code_violations = 2;
        results.satd_violations = 4;
        results.security_violations = 1;

        let mut output = String::new();
        format_results_summary(&mut output, &results);

        assert!(output.contains("Total Violations: 10"));
        assert!(output.contains("Complexity Issues: 3"));
        assert!(output.contains("Dead Code: 2"));
        assert!(output.contains("Technical Debt (SATD): 4"));
        assert!(output.contains("Security Issues: 1"));
    }

    // ===================
    // QualityViolation Tests
    // ===================

    #[test]
    fn test_quality_violation_struct() {
        let violation = QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
        };

        assert_eq!(violation.check_type, "complexity");
        assert_eq!(violation.severity, "error");
        assert_eq!(violation.file, "src/main.rs");
        assert_eq!(violation.line, Some(42));
    }

    #[test]
    fn test_quality_violation_no_line() {
        let violation = QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "warning".to_string(),
            file: "src/lib.rs".to_string(),
            line: None,
            message: "Unused function".to_string(),
        };

        assert!(violation.line.is_none());
    }

    // ===================
    // QualityGateResults Tests
    // ===================

    #[test]
    fn test_quality_gate_results_default() {
        let results = QualityGateResults::default();
        // Default is passed: true when no violations
        assert!(results.passed);
        assert_eq!(results.total_violations, 0);
        assert_eq!(results.complexity_violations, 0);
        assert_eq!(results.dead_code_violations, 0);
        assert_eq!(results.satd_violations, 0);
        assert_eq!(results.security_violations, 0);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_quality_gate_results_with_values() {
        let mut results = QualityGateResults::default();
        results.passed = true;
        results.total_violations = 5;
        results.complexity_violations = 2;
        results.dead_code_violations = 1;
        results.satd_violations = 1;
        results.security_violations = 1;

        assert!(results.passed);
        assert_eq!(results.total_violations, 5);
    }

    // ===================
    // format_violations_section Tests
    // ===================

    #[test]
    fn test_format_violations_section_empty() {
        let violations: Vec<QualityViolation> = vec![];
        let mut output = String::new();
        format_violations_section(&mut output, &violations);
        assert!(output.contains("## Violations"));
    }

    #[test]
    fn test_format_violations_section_single() {
        let violations = vec![QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(10),
            message: "Too complex".to_string(),
        }];

        let mut output = String::new();
        format_violations_section(&mut output, &violations);

        assert!(output.contains("## Violations"));
        assert!(output.contains("COMPLEXITY"));
        assert!(output.contains("main.rs"));
    }

    #[test]
    fn test_format_violations_section_multiple_types() {
        let violations = vec![
            QualityViolation {
                check_type: "complexity".to_string(),
                severity: "error".to_string(),
                file: "src/a.rs".to_string(),
                line: Some(10),
                message: "Complex".to_string(),
            },
            QualityViolation {
                check_type: "security".to_string(),
                severity: "error".to_string(),
                file: "src/b.rs".to_string(),
                line: Some(20),
                message: "Unsafe".to_string(),
            },
        ];

        let mut output = String::new();
        format_violations_section(&mut output, &violations);

        assert!(output.contains("COMPLEXITY"));
        assert!(output.contains("SECURITY"));
    }

    // ===================
    // format_single_violation Tests
    // ===================

    #[test]
    fn test_format_single_violation_with_line() {
        let violation = QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🔴")); // error icon
        assert!(output.contains("main.rs"));
        assert!(output.contains("42"));
        assert!(output.contains("Function too complex"));
    }

    #[test]
    fn test_format_single_violation_without_line() {
        let violation = QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "warning".to_string(),
            file: "src/lib.rs".to_string(),
            line: None,
            message: "Unused code".to_string(),
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🟡")); // warning icon
        assert!(output.contains("lib.rs"));
        assert!(output.contains("Unused code"));
    }

    #[test]
    fn test_format_single_violation_no_file() {
        let violation = QualityViolation {
            check_type: "satd".to_string(),
            severity: "info".to_string(),
            file: String::new(),
            line: None,
            message: "Technical debt found".to_string(),
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🟢")); // other/info icon
        assert!(output.contains("Technical debt found"));
    }

    // ===================
    // resolve_absolute_file_path Tests
    // ===================

    #[test]
    fn test_resolve_absolute_file_path_already_absolute() {
        let project = Path::new("/home/user/project");
        let file = Path::new("/home/user/project/src/main.rs");
        let result = resolve_absolute_file_path(project, file);
        assert_eq!(result, PathBuf::from("/home/user/project/src/main.rs"));
    }

    #[test]
    fn test_resolve_absolute_file_path_relative() {
        let project = Path::new("/home/user/project");
        let file = Path::new("src/main.rs");
        let result = resolve_absolute_file_path(project, file);
        assert_eq!(result, PathBuf::from("/home/user/project/src/main.rs"));
    }

    // ===================
    // load_provability_threshold Tests (GH-172)
    // ===================

    #[test]
    fn test_load_provability_threshold_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when file is missing"
        );
    }

    #[test]
    fn test_load_provability_threshold_from_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
provability_min = 0.60
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - 0.60).abs() < f64::EPSILON,
            "Should read provability_min from config, got {threshold}"
        );
    }

    #[test]
    fn test_load_provability_threshold_missing_key() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
lint_max_ms = 150000
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when key is missing"
        );
    }

    #[test]
    fn test_load_provability_threshold_invalid_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join(".pmat-metrics.toml"),
            "this is not valid toml {{{{",
        )
        .unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when TOML is invalid"
        );
    }

    #[test]
    fn test_load_provability_threshold_no_thresholds_section() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[quality_gates]
min_coverage_pct = 85.0
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_provability_threshold(temp_dir.path());
        assert!(
            (threshold - DEFAULT_PROVABILITY_THRESHOLD).abs() < f64::EPSILON,
            "Should fall back to default when [thresholds] section is missing"
        );
    }

    // --- Entropy threshold tests (#194) ---

    #[test]
    fn test_load_entropy_threshold_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let threshold = load_entropy_threshold(temp_dir.path(), 0.3);
        assert!(
            (threshold - 0.3).abs() < f64::EPSILON,
            "Should fall back to CLI value when file is missing"
        );
    }

    #[test]
    fn test_load_entropy_threshold_from_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
entropy_min_diversity = 0.0
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_entropy_threshold(temp_dir.path(), 0.3);
        assert!(
            threshold.abs() < f64::EPSILON,
            "Should read entropy_min_diversity=0.0 from config, got {threshold}"
        );
    }

    #[test]
    fn test_load_entropy_threshold_missing_key_falls_back_to_cli() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[thresholds]
provability_min = 0.70
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let threshold = load_entropy_threshold(temp_dir.path(), 0.5);
        assert!(
            (threshold - 0.5).abs() < f64::EPSILON,
            "Should fall back to CLI value when key is missing"
        );
    }

    // --- Exclude paths tests (#195) ---

    #[test]
    fn test_load_entropy_exclude_paths_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert!(paths.is_empty(), "Should return empty when file is missing");
    }

    #[test]
    fn test_load_entropy_exclude_paths_from_exclude_section() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
[exclude]
paths = ["reference/", "vendor/"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths, vec!["reference/", "vendor/"]);
    }

    #[test]
    fn test_load_entropy_exclude_paths_from_top_level() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
exclude_paths = ["third_party/"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths, vec!["third_party/"]);
    }

    #[test]
    fn test_load_entropy_exclude_paths_section_takes_precedence() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_content = r#"
exclude_paths = ["old/"]

[exclude]
paths = ["new/"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), config_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths, vec!["new/"], "[exclude] paths should take precedence over top-level exclude_paths");
    }

    #[test]
    fn test_load_entropy_exclude_paths_from_gates_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let gates_content = r#"
[quality-gates]
exclude = [
    "**/*_generated.rs",
    "demos/**",
    "examples/**",
]
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), gates_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths.len(), 3, "Should load excludes from .pmat-gates.toml [quality-gates] exclude");
        assert!(paths.contains(&"**/*_generated.rs".to_string()));
        assert!(paths.contains(&"demos/**".to_string()));
        assert!(paths.contains(&"examples/**".to_string()));
    }

    #[test]
    fn test_load_entropy_exclude_paths_merges_both_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let metrics_content = r#"
[exclude]
paths = ["target/**"]
"#;
        let gates_content = r#"
[quality-gates]
exclude = ["demos/**", "examples/**"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), metrics_content).unwrap();
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), gates_content).unwrap();

        let paths = load_entropy_exclude_paths(temp_dir.path());
        assert_eq!(paths.len(), 3, "Should merge excludes from both config files");
        assert!(paths.contains(&"target/**".to_string()));
        assert!(paths.contains(&"demos/**".to_string()));
        assert!(paths.contains(&"examples/**".to_string()));
    }

    // --- Entropy gate config tests (#220) ---

    #[test]
    fn test_load_entropy_gate_config_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(config.enabled);
        assert!(config.max_violations.is_none());
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_load_entropy_gate_config_enabled_false() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
enabled = false
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(!config.enabled);
    }

    #[test]
    fn test_load_entropy_gate_config_max_violations() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
max_violations = 5
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(config.enabled);
        assert_eq!(config.max_violations, Some(5));
    }

    #[test]
    fn test_load_entropy_gate_config_excludes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
exclude = ["**/gqa.rs", "benches/**"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert_eq!(config.exclude.len(), 2);
        assert!(config.exclude.contains(&"**/gqa.rs".to_string()));
        assert!(config.exclude.contains(&"benches/**".to_string()));
    }

    #[test]
    fn test_load_entropy_gate_config_full() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = r#"
[entropy]
enabled = true
max_pattern_repetition = 12
min_pattern_diversity = 0.3
max_violations = 3
exclude = ["**/gqa.rs"]
"#;
        std::fs::write(temp_dir.path().join(".pmat-gates.toml"), content).unwrap();
        let config = load_entropy_gate_config(temp_dir.path());
        assert!(config.enabled);
        assert_eq!(config.max_violations, Some(3));
        assert_eq!(config.exclude, vec!["**/gqa.rs"]);
    }

    // --- Filter violations by exclude paths tests (#196) ---

    #[test]
    fn test_filter_violations_excludes_matching_files() {
        let mut violations = vec![
            QualityViolation { check_type: "satd".into(), severity: "warning".into(), file: "reference/kong/init.lua".into(), line: Some(10), message: "TODO".into() },
            QualityViolation { check_type: "satd".into(), severity: "warning".into(), file: "src/main.rs".into(), line: Some(5), message: "TODO".into() },
            QualityViolation { check_type: "duplicates".into(), severity: "info".into(), file: "reference/apisix/core.lua".into(), line: None, message: "dup".into() },
        ];
        let excludes = vec!["reference/".to_string()];
        filter_violations_by_exclude(&mut violations, &excludes);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].file, "src/main.rs");
    }

    #[test]
    fn test_filter_violations_keeps_project_level() {
        let mut violations = vec![
            QualityViolation { check_type: "entropy".into(), severity: "warning".into(), file: "project".into(), line: None, message: "low diversity".into() },
            QualityViolation { check_type: "satd".into(), severity: "info".into(), file: "vendor/lib.lua".into(), line: Some(1), message: "hack".into() },
        ];
        let excludes = vec!["vendor/".to_string()];
        filter_violations_by_exclude(&mut violations, &excludes);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].file, "project");
    }

    #[test]
    fn test_filter_violations_empty_excludes_no_change() {
        let mut violations = vec![
            QualityViolation { check_type: "satd".into(), severity: "info".into(), file: "src/lib.rs".into(), line: Some(1), message: "fixme".into() },
        ];
        filter_violations_by_exclude(&mut violations, &[]);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_recalculate_from_violations() {
        let violations = vec![
            QualityViolation { check_type: "satd".into(), severity: "warning".into(), file: "a.rs".into(), line: Some(1), message: "todo".into() },
            QualityViolation { check_type: "satd".into(), severity: "info".into(), file: "b.rs".into(), line: Some(2), message: "hack".into() },
            QualityViolation { check_type: "complexity".into(), severity: "error".into(), file: "c.rs".into(), line: Some(3), message: "high".into() },
        ];
        let mut results = QualityGateResults::default();
        results.recalculate_from(&violations);
        assert_eq!(results.satd_violations, 2);
        assert_eq!(results.complexity_violations, 1);
        assert_eq!(results.total_violations, 3);
        assert_eq!(results.entropy_violations, 0);
    }
}

