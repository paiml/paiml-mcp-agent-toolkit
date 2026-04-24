#![cfg_attr(coverage_nightly, coverage(off))]
//! Output formatting and display for lint hotspot analysis

use super::types::*;
use crate::cli::LintHotspotOutputFormat;
use anyhow::{Context, Result};
use serde::Serialize;

/// Format output based on selected format
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_output(
    result: &LintHotspotResult,
    format: LintHotspotOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
    top_files: usize,
) -> Result<String> {
    match format {
        LintHotspotOutputFormat::Summary => format_summary(result, perf, elapsed, top_files),
        LintHotspotOutputFormat::Detailed => format_detailed(result, perf, elapsed, top_files),
        LintHotspotOutputFormat::Json => format_json(result, false),
        LintHotspotOutputFormat::EnforcementJson => format_json(result, true),
        LintHotspotOutputFormat::Sarif => format_sarif(result),
    }
}

/// Format summary output
///
/// # Example
///
/// ```no_run
/// use pmat::cli::handlers::lint_hotspot_handlers::{LintHotspotResult, LintHotspot, FileSummary, SeverityDistribution, QualityGateStatus};
/// use std::collections::HashMap;
/// use std::path::PathBuf;
/// use std::time::Duration;
///
/// let hotspot = LintHotspot {
///     file: PathBuf::from("src/main.rs"),
///     defect_density: 0.05,
///     total_violations: 5,
///     sloc: 100,
///     severity_distribution: SeverityDistribution {
///         error: 2,
///         warning: 3,
///         suggestion: 0,
///         note: 0,
///     },
///     top_lints: vec![
///         ("clippy::too_many_arguments".to_string(), 2),
///         ("unused_variable".to_string(), 3),
///     ],
///     detailed_violations: vec![],
/// };
///
/// let mut summary_by_file = HashMap::new();
/// summary_by_file.insert(
///     PathBuf::from("src/main.rs"),
///     FileSummary {
///         total_violations: 5,
///         errors: 2,
///         warnings: 3,
///         sloc: 100,
///         defect_density: 0.05,
///     }
/// );
///
/// let result = LintHotspotResult {
///     hotspot,
///     all_violations: vec![],
///     summary_by_file,
///     total_project_violations: 5,
///     enforcement: None,
///     refactor_chain: None,
///     quality_gate: QualityGateStatus {
///         passed: true,
///         violations: vec![],
///         blocking: false,
///     },
/// };
///
/// let output = pmat::cli::handlers::lint_hotspot_handlers::format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
///
/// assert!(output.contains("# Lint Hotspot Analysis"));
/// assert!(output.contains("**Total Project Violations**: 5"));
/// assert!(output.contains("## Top Files with Lint Issues"));
/// assert!(output.contains("1. `main.rs` - 0.05 violations/SLOC"));
/// assert!(output.contains("## Hottest File Details"));
/// assert!(output.contains("**File**: src/main.rs"));
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_summary(
    result: &LintHotspotResult,
    perf: bool,
    elapsed: std::time::Duration,
    _top_files: usize,
) -> Result<String> {
    let mut output = String::new();

    output.push_str("# Lint Hotspot Analysis (EXTREME Quality Mode)\n\n");
    output.push_str(&format!(
        "**Total Project Violations**: {}\n",
        result.total_project_violations
    ));
    output.push_str(&format!(
        "**Files with Issues**: {}\n\n",
        result.summary_by_file.len()
    ));

    // Show top files with lint issues (consistent with other analyze commands)
    output.push_str("## Top Files with Lint Issues\n\n");
    let mut sorted_files: Vec<_> = result.summary_by_file.iter().collect();
    sorted_files.sort_by(|a, b| {
        b.1.defect_density
            .partial_cmp(&a.1.defect_density)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let files_to_show = if _top_files == 0 { 10 } else { _top_files };
    for (i, (file, summary)) in sorted_files.iter().take(files_to_show).enumerate() {
        let filename = file.file_name().unwrap_or_default().to_string_lossy();
        output.push_str(&format!(
            "{}. `{}` - {:.2} violations/SLOC ({} violations, {} SLOC)\n",
            i + 1,
            filename,
            summary.defect_density,
            summary.total_violations,
            summary.sloc
        ));
    }
    output.push('\n');

    output.push_str("## Hottest File Details\n");
    output.push_str(&format!("**File**: {}\n", result.hotspot.file.display()));
    output.push_str(&format!(
        "**Defect Density**: {:.2} violations/SLOC\n",
        result.hotspot.defect_density
    ));
    output.push_str(&format!(
        "**Total Violations**: {}\n",
        result.hotspot.total_violations
    ));
    output.push_str(&format!("**Lines of Code**: {}\n\n", result.hotspot.sloc));

    output.push_str("## Severity Distribution\n");
    output.push_str(&format!(
        "- Errors: {}\n",
        result.hotspot.severity_distribution.error
    ));
    output.push_str(&format!(
        "- Warnings: {}\n",
        result.hotspot.severity_distribution.warning
    ));
    output.push_str(&format!(
        "- Suggestions: {}\n\n",
        result.hotspot.severity_distribution.suggestion
    ));

    output.push_str("## Top Violations\n");
    for (lint, count) in result.hotspot.top_lints.iter().take(5) {
        output.push_str(&format!("- {lint}: {count} occurrences\n"));
    }

    if let Some(enforcement) = &result.enforcement {
        output.push_str("\n## Enforcement Metadata\n");
        output.push_str(&format!(
            "- Score: {:.1}/10\n",
            enforcement.enforcement_score
        ));
        output.push_str(&format!(
            "- Priority: {}\n",
            enforcement.enforcement_priority
        ));
        output.push_str(&format!(
            "- Estimated Fix Time: {} minutes\n",
            enforcement.estimated_fix_time / 60
        ));
        output.push_str(&format!(
            "- Automation Confidence: {:.0}%\n",
            enforcement.automation_confidence * 100.0
        ));
    }

    if !result.quality_gate.passed {
        output.push_str("\n## ❌ Quality Gate Failed\n");
        for violation in &result.quality_gate.violations {
            output.push_str(&format!(
                "- {} exceeded: {:.2} > {:.2}\n",
                violation.rule, violation.actual, violation.threshold
            ));
        }
    }

    if perf {
        output.push_str(&format!(
            "\n⏱️  Analysis completed in {:.2}s\n",
            elapsed.as_secs_f64()
        ));
    }

    Ok(output)
}

/// Format detailed output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_detailed(
    result: &LintHotspotResult,
    perf: bool,
    elapsed: std::time::Duration,
    top_files: usize,
) -> Result<String> {
    let mut output = format_summary(result, perf, elapsed, top_files)?;

    // Add detailed violations for the hotspot file
    output.push_str("\n## Detailed Violations in Hotspot File\n");
    for violation in &result.hotspot.detailed_violations {
        output.push_str(&format!(
            "- **{}:{}:{}** [{}] {}\n",
            violation.file.display(),
            violation.line,
            violation.column,
            violation.lint_name,
            violation.message
        ));
        if let Some(suggestion) = &violation.suggestion {
            output.push_str(&format!("  Suggestion: {suggestion}\n"));
        }
    }

    // Add top files by violation count
    output.push_str("\n## Top Files by Violations\n");
    let mut sorted_files: Vec<_> = result.summary_by_file.iter().collect();
    sorted_files.sort_by_key(|b| std::cmp::Reverse(b.1.total_violations));

    let files_to_show = if top_files == 0 {
        sorted_files.len()
    } else {
        top_files
    };
    for (file, summary) in sorted_files.iter().take(files_to_show) {
        output.push_str(&format!(
            "- {}: {} violations ({} errors, {} warnings, density: {:.2})\n",
            file.display(),
            summary.total_violations,
            summary.errors,
            summary.warnings,
            summary.defect_density
        ));
    }

    if let Some(chain) = &result.refactor_chain {
        output.push_str("\n## Refactor Chain\n");
        output.push_str(&format!("ID: {}\n", chain.id));
        output.push_str(&format!(
            "Estimated Reduction: {} violations\n",
            chain.estimated_reduction
        ));
        output.push_str(&format!(
            "Automation Confidence: {:.0}%\n\n",
            chain.automation_confidence * 100.0
        ));

        output.push_str("### Steps\n");
        for (i, step) in chain.steps.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} - {} (confidence: {:.0}%, impact: {})\n",
                i + 1,
                step.description,
                step.lint,
                step.confidence * 100.0,
                step.impact
            ));
        }
    }

    Ok(output)
}

/// Format JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
fn format_json(result: &LintHotspotResult, enforcement: bool) -> Result<String> {
    if enforcement {
        // Full enforcement-ready JSON
        serde_json::to_string_pretty(result).context("Failed to serialize to JSON")
    } else {
        // Simple JSON without enforcement details
        #[derive(Serialize)]
        struct SimpleResult<'a> {
            hotspot: &'a LintHotspot,
            quality_gate: &'a QualityGateStatus,
        }

        let simple = SimpleResult {
            hotspot: &result.hotspot,
            quality_gate: &result.quality_gate,
        };

        serde_json::to_string_pretty(&simple).context("Failed to serialize to JSON")
    }
}

/// Format SARIF output
///
/// # Errors
///
/// Returns an error if the operation fails
fn format_sarif(result: &LintHotspotResult) -> Result<String> {
    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-lint-hotspot",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": result.quality_gate.violations.iter().map(|v| {
                serde_json::json!({
                    "ruleId": v.rule,
                    "level": if v.severity == "blocking" { "error" } else { "warning" },
                    "message": {
                        "text": format!("{} exceeded: {:.2} > {:.2}", v.rule, v.actual, v.threshold)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": result.hotspot.file.to_string_lossy()
                            }
                        }
                    }]
                })
            }).collect::<Vec<_>>()
        }]
    });

    serde_json::to_string_pretty(&sarif).context("Failed to serialize to SARIF")
}

#[cfg(test)]
mod lint_hotspot_output_tests {
    //! Covers format_output dispatcher + format_json/sarif in
    //! lint_hotspot_handlers/output.rs (40 uncov on broad, 0% cov).
    use super::*;

    fn empty_result() -> LintHotspotResult {
        LintHotspotResult {
            hotspot: LintHotspot {
                file: "src/a.rs".into(),
                defect_density: 0.0,
                total_violations: 0,
                sloc: 100,
                severity_distribution: SeverityDistribution::default(),
                top_lints: vec![],
                detailed_violations: vec![],
            },
            all_violations: vec![],
            summary_by_file: std::collections::HashMap::new(),
            total_project_violations: 0,
            enforcement: None,
            refactor_chain: None,
            quality_gate: QualityGateStatus {
                passed: true,
                violations: vec![],
                blocking: false,
            },
        }
    }

    #[test]
    fn test_format_output_json_returns_valid_json() {
        let out = format_output(
            &empty_result(),
            LintHotspotOutputFormat::Json,
            false,
            std::time::Duration::from_millis(10),
            5,
        )
        .unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_format_output_enforcement_json_dispatch() {
        let out = format_output(
            &empty_result(),
            LintHotspotOutputFormat::EnforcementJson,
            false,
            std::time::Duration::from_millis(10),
            5,
        )
        .unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_format_output_sarif_returns_sarif_envelope() {
        let out = format_output(
            &empty_result(),
            LintHotspotOutputFormat::Sarif,
            false,
            std::time::Duration::from_millis(10),
            5,
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.is_object());
        let obj = parsed.as_object().unwrap();
        assert!(
            obj.contains_key("version") || obj.contains_key("runs"),
            "SARIF envelope missing"
        );
    }

    #[test]
    fn test_format_output_summary_dispatch() {
        let out = format_output(
            &empty_result(),
            LintHotspotOutputFormat::Summary,
            false,
            std::time::Duration::from_millis(10),
            5,
        )
        .unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn test_format_output_detailed_dispatch() {
        let out = format_output(
            &empty_result(),
            LintHotspotOutputFormat::Detailed,
            false,
            std::time::Duration::from_millis(10),
            5,
        )
        .unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn test_format_json_enforcement_vs_non_enforcement_both_valid() {
        let non_enf = format_json(&empty_result(), false).unwrap();
        let enf = format_json(&empty_result(), true).unwrap();
        let _: serde_json::Value = serde_json::from_str(&non_enf).unwrap();
        let _: serde_json::Value = serde_json::from_str(&enf).unwrap();
    }

    #[test]
    fn test_format_sarif_empty_result_valid_json() {
        let out = format_sarif(&empty_result()).unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_format_output_perf_flag_toggle() {
        let out_off = format_output(
            &empty_result(),
            LintHotspotOutputFormat::Summary,
            false,
            std::time::Duration::from_millis(5),
            3,
        )
        .unwrap();
        let out_on = format_output(
            &empty_result(),
            LintHotspotOutputFormat::Summary,
            true,
            std::time::Duration::from_millis(5),
            3,
        )
        .unwrap();
        assert!(!out_off.is_empty());
        assert!(!out_on.is_empty());
    }
}
