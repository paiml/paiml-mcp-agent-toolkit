//! SATD (Self-Admitted Technical Debt) Analysis Handler
//!
//! Refactored handler using the service facade pattern.

use crate::cli::{SatdOutputFormat, SatdSeverity};
use crate::services::facades::satd_facade::{SatdAnalysisRequest, SatdAnalysisResult, SatdFacade};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Refactored handler for SATD analysis using the facade pattern.
pub async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    strict: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
    _top_files: usize,
    _fail_on_violation: bool,
    _timeout: u64,
) -> Result<()> {
    eprintln!("🔍 Analyzing Self-Admitted Technical Debt (SATD)...");

    // Create service registry and facade
    let registry = Arc::new(ServiceRegistry::new());
    let facade = SatdFacade::new(registry);

    // Build analysis request
    let request = SatdAnalysisRequest {
        path,
        strict_mode: strict,
        include_tests,
    };

    // Perform analysis
    let result = facade.analyze_project(request).await?;

    // Apply filters
    let filtered_result = apply_filters(result, severity, critical_only);

    // Format and output
    let content = format_output(&filtered_result, format, evolution, days, metrics);

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    if metrics {
        print_metrics(&filtered_result);
    }

    Ok(())
}

/// Apply severity and criticality filters
fn apply_filters(
    mut result: SatdAnalysisResult,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) -> SatdAnalysisResult {
    if let Some(min_severity) = severity {
        result.violations.retain(|v| match min_severity {
            SatdSeverity::Critical => matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            ),
            SatdSeverity::High => matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
                    | crate::services::facades::satd_facade::SatdSeverity::High
            ),
            SatdSeverity::Medium => !matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Low
            ),
            SatdSeverity::Low => true,
        });
    }

    if critical_only {
        result.violations.retain(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
                    | crate::services::facades::satd_facade::SatdSeverity::High
            )
        });
    }

    result
}

/// Format output based on format type
fn format_output(
    result: &SatdAnalysisResult,
    format: SatdOutputFormat,
    evolution: bool,
    days: u32,
    metrics: bool,
) -> String {
    match format {
        SatdOutputFormat::Summary => format_summary(result),
        SatdOutputFormat::Json => format_json(result, metrics, evolution),
        SatdOutputFormat::Sarif => format_sarif(result),
        SatdOutputFormat::Markdown => format_markdown(result, evolution, days),
    }
}

/// Format as summary
fn format_summary(result: &SatdAnalysisResult) -> String {
    let mut output = String::new();
    output.push_str("# SATD Analysis Summary\n\n");
    output.push_str(&result.summary);
    output.push_str(&format!(
        "\n\nTotal violations: {}\n",
        result.violations.len()
    ));

    // Group by severity
    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();
    let medium_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Medium
            )
        })
        .count();
    let low_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Low
            )
        })
        .count();

    output.push_str("\n## Severity Distribution\n");
    output.push_str(&format!("- Critical: {}\n", critical_count));
    output.push_str(&format!("- High: {}\n", high_count));
    output.push_str(&format!("- Medium: {}\n", medium_count));
    output.push_str(&format!("- Low: {}\n", low_count));

    if !result.violations.is_empty() {
        output.push_str("\n## Top Violations\n");
        for (i, violation) in result.violations.iter().take(10).enumerate() {
            output.push_str(&format!(
                "{}. {}:{} - {} ({:?})\n",
                i + 1,
                violation.file_path,
                violation.line_number,
                violation.violation_type,
                violation.severity
            ));
        }
    }

    output
}

/// Format as JSON
fn format_json(result: &SatdAnalysisResult, metrics: bool, evolution: bool) -> String {
    let mut json_data = serde_json::json!({
        "total_files": result.total_files,
        "total_violations": result.violations.len(),
        "summary": result.summary,
        "violations": result.violations.iter().map(|v| {
            serde_json::json!({
                "file": v.file_path,
                "line": v.line_number,
                "type": v.violation_type,
                "message": v.message,
                "severity": format!("{:?}", v.severity)
            })
        }).collect::<Vec<_>>()
    });

    if metrics {
        json_data["metrics"] = serde_json::json!({
            "critical_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Critical))
                .count(),
            "high_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::High))
                .count(),
            "medium_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Medium))
                .count(),
            "low_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Low))
                .count(),
        });
    }

    if evolution {
        json_data["evolution"] = serde_json::json!({
            "message": "Evolution tracking would show SATD trends over time"
        });
    }

    serde_json::to_string_pretty(&json_data).unwrap_or_else(|_| "{}".to_string())
}

/// Format as SARIF
fn format_sarif(result: &SatdAnalysisResult) -> String {
    let rules = vec![serde_json::json!({
        "id": "satd-violation",
        "shortDescription": {
            "text": "Self-Admitted Technical Debt"
        },
        "fullDescription": {
            "text": "Code contains self-admitted technical debt that should be addressed"
        }
    })];

    let results: Vec<_> = result
        .violations
        .iter()
        .map(|violation| {
            let level = match violation.severity {
                crate::services::facades::satd_facade::SatdSeverity::Critical => "error",
                crate::services::facades::satd_facade::SatdSeverity::High => "error",
                crate::services::facades::satd_facade::SatdSeverity::Medium => "warning",
                crate::services::facades::satd_facade::SatdSeverity::Low => "note",
            };

            serde_json::json!({
                "ruleId": "satd-violation",
                "level": level,
                "message": {
                    "text": format!("{}: {}", violation.violation_type, violation.message)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": violation.file_path.clone()
                        },
                        "region": {
                            "startLine": violation.line_number
                        }
                    }
                }]
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-satd-detector",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
    .to_string()
}

/// Format as Markdown
fn format_markdown(result: &SatdAnalysisResult, evolution: bool, days: u32) -> String {
    let mut output = String::new();
    output.push_str("# SATD Analysis Report\n\n");
    output.push_str(&format!("**Summary:** {}\n\n", result.summary));

    output.push_str("## Metrics\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| Total Files | {} |\n", result.total_files));
    output.push_str(&format!(
        "| Total Violations | {} |\n",
        result.violations.len()
    ));

    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();

    output.push_str(&format!("| Critical Violations | {} |\n", critical_count));
    output.push_str(&format!("| High Violations | {} |\n\n", high_count));

    if evolution {
        output.push_str(&format!("## Evolution (Last {} Days)\n\n", days));
        output.push_str("*Evolution tracking would show SATD trends over time*\n\n");
    }

    if !result.violations.is_empty() {
        output.push_str("## Violations\n\n");
        output.push_str("| File | Line | Type | Severity | Message |\n");
        output.push_str("|------|------|------|----------|----------|\n");

        for violation in &result.violations {
            output.push_str(&format!(
                "| {} | {} | {} | {:?} | {} |\n",
                violation.file_path,
                violation.line_number,
                violation.violation_type,
                violation.severity,
                violation.message
            ));
        }
    }

    output
}

/// Print metrics to stderr
fn print_metrics(result: &SatdAnalysisResult) {
    eprintln!("\n📊 SATD Metrics:");
    eprintln!("  Total files analyzed: {}", result.total_files);
    eprintln!("  Total violations: {}", result.violations.len());

    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();

    eprintln!("  Critical violations: {}", critical_count);
    eprintln!("  High violations: {}", high_count);

    if !result.violations.is_empty() {
        eprintln!("\n  Top violation types:");
        use std::collections::HashMap;
        let mut type_counts: HashMap<&str, usize> = HashMap::new();
        for violation in &result.violations {
            *type_counts.entry(&violation.violation_type).or_insert(0) += 1;
        }

        let mut sorted_types: Vec<_> = type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));

        for (violation_type, count) in sorted_types.iter().take(5) {
            eprintln!("    - {}: {}", violation_type, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::facades::satd_facade::{SatdSeverity as FacadeSeverity, SatdViolation};

    #[test]
    fn test_format_summary() {
        let result = SatdAnalysisResult {
            total_files: 10,
            violations: vec![SatdViolation {
                file_path: "test.rs".to_string(),
                line_number: 42,
                violation_type: "TODO".to_string(),
                message: "Implement feature".to_string(),
                severity: FacadeSeverity::Medium,
            }],
            summary: "Test summary".to_string(),
        };

        let output = format_summary(&result);
        assert!(output.contains("Test summary"));
        assert!(output.contains("Total violations: 1"));
    }
}
