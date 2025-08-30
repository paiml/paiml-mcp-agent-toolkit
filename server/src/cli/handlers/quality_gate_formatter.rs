//! Toyota Way: Quality Gate Formatting Handler
//! Complexity: Reduced from 20 to individual functions ≤8
//! Purpose: Quality gate report formatting with clean separation of concerns

use crate::cli::analysis_utilities::{QualityGateResults, QualityViolation};
use crate::cli::{QualityCheckType, QualityGateOutputFormat};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

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
pub fn format_single_file_summary(
    file_path: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> String {
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

/// Toyota Way: Extract Method - Add single violation entry (complexity ≤3)
fn add_violation_entry(output: &mut String, violation: &QualityViolation) {
    let severity_icon = get_severity_icon(&violation.severity);

    if let Some(line) = violation.line {
        output.push_str(&format!(
            "- {} Line {}: {}\n",
            severity_icon, line, violation.message
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

/// Toyota Way: Extract Method - Print checks to run (complexity ≤8)
/// Console output utility for displaying which quality checks will be executed
pub fn print_checks_to_run(checks: &[QualityCheckType]) {
    eprintln!("\n📋 Checks to run:");

    if checks.contains(&QualityCheckType::All) {
        print_all_checks();
    } else {
        print_specific_checks(checks);
    }
    eprintln!();
}

/// Toyota Way: Extract Method - Print all check types (complexity ≤3)
fn print_all_checks() {
    eprintln!("  ✓ Complexity analysis");
    eprintln!("  ✓ Dead code detection");
    eprintln!("  ✓ Self-admitted technical debt (SATD)");
    eprintln!("  ✓ Security vulnerabilities");
    eprintln!("  ✓ Code entropy");
    eprintln!("  ✓ Duplicate code");
    eprintln!("  ✓ Test coverage");
}

/// Toyota Way: Extract Method - Print specific check types (complexity ≤8)
fn print_specific_checks(checks: &[QualityCheckType]) {
    for check in checks {
        let check_name = match check {
            QualityCheckType::Complexity => "✓ Complexity analysis",
            QualityCheckType::DeadCode => "✓ Dead code detection",
            QualityCheckType::Satd => "✓ Self-admitted technical debt (SATD)",
            QualityCheckType::Security => "✓ Security vulnerabilities",
            QualityCheckType::Entropy => "✓ Code entropy",
            QualityCheckType::Duplicates => "✓ Duplicate code",
            QualityCheckType::Coverage => "✓ Test coverage",
            _ => continue, // Skip other types
        };
        eprintln!("  {}", check_name);
    }
}

/// Configuration for quality checks (SPRINT-23)
#[derive(Debug, Clone)]
pub struct QualityCheckConfig<'a> {
    pub project_path: &'a Path,
    pub checks: &'a [QualityCheckType],
    pub max_dead_code: f64,
    pub min_entropy: f64,
    pub max_complexity_p99: u32,
    pub perf: bool,
}

/// Toyota Way: Extract Method - Run project quality checks (complexity ≤8)
/// Orchestrates the execution of quality checks based on the specified types
pub async fn run_project_checks(
    config: QualityCheckConfig<'_>,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    // If checks contains All, run the comprehensive check
    if config.checks.contains(&QualityCheckType::All) {
        run_all_checks(
            config.project_path,
            config.max_dead_code,
            config.min_entropy,
            config.max_complexity_p99,
            violations,
            results,
            config.perf,
        )
        .await?;
    } else {
        // Run individual checks with performance timing
        run_individual_checks(
            config.checks,
            config.project_path,
            config.max_dead_code,
            config.min_entropy,
            config.max_complexity_p99,
            violations,
            results,
            config.perf,
        )
        .await?;
    }
    Ok(())
}

/// Toyota Way: Extract Method - Run all quality checks (complexity ≤3)
async fn run_all_checks(
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    crate::cli::analysis_utilities::run_single_project_check(
        &QualityCheckType::All,
        project_path,
        max_dead_code,
        min_entropy,
        max_complexity_p99,
        violations,
        results,
        perf,
    )
    .await
}

/// Toyota Way: Extract Method - Run individual checks with timing (complexity ≤8)
async fn run_individual_checks(
    checks: &[QualityCheckType],
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    for check in checks {
        let check_start = if perf { Some(Instant::now()) } else { None };

        crate::cli::analysis_utilities::run_single_project_check(
            check,
            project_path,
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            violations,
            results,
            perf,
        )
        .await?;

        // Print timing if performance monitoring is enabled
        if let Some(start) = check_start {
            print_check_timing(check, start.elapsed().as_secs_f64());
        }
    }
    Ok(())
}

/// Toyota Way: Extract Method - Print check timing (complexity ≤8)
fn print_check_timing(check: &QualityCheckType, elapsed_secs: f64) {
    let check_name = match check {
        QualityCheckType::Complexity => "Complexity",
        QualityCheckType::DeadCode => "Dead code",
        QualityCheckType::Satd => "SATD",
        QualityCheckType::Security => "Security",
        QualityCheckType::Entropy => "Entropy",
        QualityCheckType::Duplicates => "Duplicates",
        QualityCheckType::Coverage => "Coverage",
        QualityCheckType::Sections => "Sections",
        QualityCheckType::Provability => "Provability",
        QualityCheckType::All => "All",
    };
    eprintln!("    ⏱️  {} check: {:.3}s", check_name, elapsed_secs);
}

/// Toyota Way: Extract Method - Format quality gate results as JUnit XML (complexity ≤8)
/// Creates JUnit-compatible XML output for CI/CD integration
pub fn format_qg_as_junit(violations: &[QualityViolation]) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    // XML header and test suite opening
    writeln!(&mut output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(&mut output, r#"<testsuites name="Quality Gate">"#)?;
    writeln!(
        &mut output,
        r#"  <testsuite name="Quality Checks" tests="{}" failures="{}">"#,
        violations.len(),
        violations.len()
    )?;

    // Generate test cases for each violation
    for violation in violations {
        write_junit_test_case(&mut output, violation)?;
    }

    // Close XML structure
    writeln!(&mut output, r"  </testsuite>")?;
    writeln!(&mut output, r"</testsuites>")?;
    Ok(output)
}

/// Toyota Way: Extract Method - Write single JUnit test case (complexity ≤3)
fn write_junit_test_case(
    output: &mut String,
    violation: &QualityViolation,
) -> Result<(), std::fmt::Error> {
    use std::fmt::Write;

    writeln!(
        output,
        r#"    <testcase name="{}" classname="{}">"#,
        violation.message, violation.check_type
    )?;
    writeln!(
        output,
        r#"      <failure message="{}" type="{}"/>"#,
        violation.message, violation.severity
    )?;
    writeln!(output, r"    </testcase>")?;
    Ok(())
}
