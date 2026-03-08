#![cfg_attr(coverage_nightly, coverage(off))]
//! Formatting functions for complexity reports (CLI and SARIF output).

use super::types::{ComplexityReport, Violation};

/// Format complexity summary for CLI output
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::*;
///
/// let file_metrics = vec![
///     FileComplexityMetrics {
///         path: "src/main.rs".to_string(),
///         total_complexity: ComplexityMetrics {
///             cyclomatic: 5,
///             cognitive: 7,
///             nesting_max: 2,
///             lines: 30,
///             halstead: None,
///         },
///         functions: vec![],
///         classes: vec![],
///     },
///     FileComplexityMetrics {
///         path: "src/lib.rs".to_string(),
///         total_complexity: ComplexityMetrics {
///             cyclomatic: 3,
///             cognitive: 4,
///             nesting_max: 1,
///             lines: 20,
///             halstead: None,
///         },
///         functions: vec![],
///         classes: vec![],
///     },
/// ];
///
/// let report = aggregate_results(file_metrics);
/// let summary = format_complexity_summary(&report);
///
/// assert!(summary.contains("Complexity Analysis Summary"));
/// assert!(summary.contains("Files analyzed"));
/// assert!(summary.contains("Top Files by Complexity"));
/// assert!(summary.contains("main.rs")); // First file (higher complexity, in cyan)
/// assert!(summary.contains("lib.rs"));  // Second file (in cyan)
/// ```
#[must_use]
pub fn format_complexity_summary(report: &ComplexityReport) -> String {
    use crate::cli::colors as c;

    let mut output = String::new();

    output.push_str(&format!("{}\n\n", c::header("Complexity Analysis Summary")));

    output.push_str(&format!(
        "  {} {}\n",
        c::label("Files analyzed:"),
        c::number(&report.summary.total_files.to_string())
    ));
    output.push_str(&format!(
        "  {} {}\n\n",
        c::label("Total functions:"),
        c::number(&report.summary.total_functions.to_string())
    ));

    output.push_str(&format!("{}\n\n", c::subheader("Complexity Metrics")));
    output.push_str(&format!(
        "  {} {}\n",
        c::label("Median Cyclomatic:"),
        c::number(&format!("{:.1}", report.summary.median_cyclomatic))
    ));
    output.push_str(&format!(
        "  {} {}\n",
        c::label("Median Cognitive:"),
        c::number(&format!("{:.1}", report.summary.median_cognitive))
    ));
    output.push_str(&format!(
        "  {} {}\n",
        c::label("Max Cyclomatic:"),
        c::number(&report.summary.max_cyclomatic.to_string())
    ));
    output.push_str(&format!(
        "  {} {}\n",
        c::label("Max Cognitive:"),
        c::number(&report.summary.max_cognitive.to_string())
    ));
    output.push_str(&format!(
        "  {} {}\n",
        c::label("90th Percentile Cyclomatic:"),
        c::number(&report.summary.p90_cyclomatic.to_string())
    ));
    output.push_str(&format!(
        "  {} {}\n\n",
        c::label("90th Percentile Cognitive:"),
        c::number(&report.summary.p90_cognitive.to_string())
    ));

    if report.summary.technical_debt_hours > 0.0 {
        output.push_str(&format!(
            "  {} {}\n\n",
            c::label("Estimated Refactoring Time:"),
            c::number(&format!("{:.1} hours", report.summary.technical_debt_hours))
        ));
    }

    // Violations summary
    let error_count = report
        .violations
        .iter()
        .filter(|v| matches!(v, Violation::Error { .. }))
        .count();
    let warning_count = report
        .violations
        .iter()
        .filter(|v| matches!(v, Violation::Warning { .. }))
        .count();

    if error_count > 0 || warning_count > 0 {
        output.push_str(&format!("{}\n\n", c::subheader("Issues Found")));
        if error_count > 0 {
            output.push_str(&format!(
                "  {}Errors:{} {}\n",
                c::BOLD_RED,
                c::RESET,
                c::number(&error_count.to_string())
            ));
        }
        if warning_count > 0 {
            output.push_str(&format!(
                "  {}Warnings:{} {}\n",
                c::BOLD_YELLOW,
                c::RESET,
                c::number(&warning_count.to_string())
            ));
        }
        output.push('\n');
    }

    // Top files by complexity
    if !report.files.is_empty() {
        output.push_str(&format!("{}\n\n", c::subheader("Top Files by Complexity")));

        // Sort files by total complexity (cyclomatic + cognitive)
        let mut files_with_score: Vec<_> = report
            .files
            .iter()
            .map(|f| {
                let total_score = f64::from(f.total_complexity.cyclomatic)
                    + f64::from(f.total_complexity.cognitive);
                (f, total_score)
            })
            .collect();
        files_with_score
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (i, (file, _score)) in files_with_score.iter().take(10).enumerate() {
            // Use relative path for better identification, not just filename
            let display_path = file.path.strip_prefix("./").unwrap_or(&file.path);
            output.push_str(&format!(
                "  {}. {} - Cyclomatic: {}, Cognitive: {}, Functions: {}\n",
                c::number(&(i + 1).to_string()),
                c::path(display_path),
                c::number(&file.total_complexity.cyclomatic.to_string()),
                c::number(&file.total_complexity.cognitive.to_string()),
                c::number(&file.functions.len().to_string())
            ));
        }
        output.push('\n');

        // Show all functions when there's only one file (e.g., single file analysis)
        if report.files.len() == 1 && !report.files[0].functions.is_empty() {
            output.push_str(&format!("{}\n\n", c::subheader("Functions in File")));

            // Sort functions by total complexity
            let mut functions_with_score: Vec<_> = report.files[0]
                .functions
                .iter()
                .map(|f| {
                    let total = f64::from(f.metrics.cyclomatic) + f64::from(f.metrics.cognitive);
                    (f, total)
                })
                .collect();
            functions_with_score.sort_unstable_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });

            for (i, (func, _)) in functions_with_score.iter().enumerate() {
                output.push_str(&format!(
                    "  {}. {} {} - Cyclomatic: {}, Cognitive: {}\n",
                    c::number(&(i + 1).to_string()),
                    c::path(&func.name),
                    c::dim(&format!("(line {}-{})", func.line_start, func.line_end)),
                    c::number(&func.metrics.cyclomatic.to_string()),
                    c::number(&func.metrics.cognitive.to_string())
                ));
            }
            output.push('\n');
        }
    }

    // Top hotspots
    if !report.hotspots.is_empty() {
        output.push_str(&format!("{}\n\n", c::subheader("Top Complexity Hotspots")));
        for (i, hotspot) in report.hotspots.iter().take(5).enumerate() {
            let display_path = hotspot.file.strip_prefix("./").unwrap_or(&hotspot.file);
            let func_name = hotspot.function.as_deref().unwrap_or("<file>");
            output.push_str(&format!(
                "  {}. {} {}:{} - {} complexity: {}\n",
                c::number(&(i + 1).to_string()),
                c::path(func_name),
                c::dim(display_path),
                c::dim(&hotspot.line.to_string()),
                hotspot.complexity_type,
                c::number(&hotspot.complexity.to_string())
            ));
        }
    }

    output
}

/// Format full complexity report for CLI output
#[must_use]
pub fn format_complexity_report(report: &ComplexityReport) -> String {
    use crate::cli::colors as c;

    let mut output = format_complexity_summary(report);

    output.push_str(&format!("\n{}\n\n", c::subheader("Detailed Violations")));

    // Group violations by file
    let mut violations_by_file: rustc_hash::FxHashMap<&str, Vec<&Violation>> =
        rustc_hash::FxHashMap::default();
    for violation in &report.violations {
        let file = match violation {
            Violation::Error { file, .. } | Violation::Warning { file, .. } => file.as_str(),
        };
        violations_by_file.entry(file).or_default().push(violation);
    }

    for (file, violations) in violations_by_file {
        output.push_str(&format!("  {}\n\n", c::path(file)));

        for violation in violations {
            match violation {
                Violation::Error {
                    rule,
                    message,
                    line,
                    function,
                    ..
                } => {
                    output.push_str(&format!(
                        "    {} {}:{} {} - {}\n",
                        c::fail(""),
                        c::dim(&line.to_string()),
                        c::dim(function.as_deref().unwrap_or("")),
                        rule,
                        message
                    ));
                }
                Violation::Warning {
                    rule,
                    message,
                    line,
                    function,
                    ..
                } => {
                    output.push_str(&format!(
                        "    {} {}:{} {} - {}\n",
                        c::warn(""),
                        c::dim(&line.to_string()),
                        c::dim(function.as_deref().unwrap_or("")),
                        rule,
                        message
                    ));
                }
            }
        }
        output.push('\n');
    }

    output
}

/// Format complexity report as SARIF for IDE integration
/// Formats a complexity report as SARIF (Static Analysis Results Interchange Format)
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::{format_as_sarif, ComplexityReport, ComplexitySummary};
///
/// let report = ComplexityReport {
///     summary: ComplexitySummary {
///         total_files: 1,
///         total_functions: 1,
///         median_cyclomatic: 5.0,
///         median_cognitive: 5.0,
///         max_cyclomatic: 10,
///         max_cognitive: 10,
///         p90_cyclomatic: 8,
///         p90_cognitive: 8,
///         technical_debt_hours: 1.0,
///     },
///     violations: vec![],
///     hotspots: vec![],
///     files: vec![],
/// };
///
/// let sarif = format_as_sarif(&report).unwrap();
/// assert!(sarif.contains("\"version\": \"2.1.0\""));
/// assert!(sarif.contains("cyclomatic-complexity"));
/// ```
pub fn format_as_sarif(report: &ComplexityReport) -> Result<String, serde_json::Error> {
    use serde_json::json;

    let rules = vec![
        json!({
            "id": "cyclomatic-complexity",
            "name": "Cyclomatic Complexity",
            "shortDescription": {
                "text": "Function has high cyclomatic complexity"
            },
            "fullDescription": {
                "text": "Cyclomatic complexity measures the number of linearly independent paths through a function"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        json!({
            "id": "cognitive-complexity",
            "name": "Cognitive Complexity",
            "shortDescription": {
                "text": "Function has high cognitive complexity"
            },
            "fullDescription": {
                "text": "Cognitive complexity measures how difficult the function is to understand"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
    ];

    let mut results = Vec::new();
    for violation in &report.violations {
        let (rule_id, message, level, file, line, _function) = match violation {
            Violation::Error {
                rule,
                message,
                file,
                line,
                function,
                ..
            } => (rule, message, "error", file, line, function),
            Violation::Warning {
                rule,
                message,
                file,
                line,
                function,
                ..
            } => (rule, message, "warning", file, line, function),
        };

        results.push(json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": message
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": file
                    },
                    "region": {
                        "startLine": line
                    }
                }
            }]
        }));
    }

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif)
}
