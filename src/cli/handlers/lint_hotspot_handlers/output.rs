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
/// // #700: this result covers exactly one file, so the total is that file's
/// // total and is labelled as such — never as the project's.
/// assert!(output.contains("**Total Violations in `src/main.rs`**: 5"));
/// assert!(!output.contains("**Total Project Violations**"));
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
    push_totals_header(&mut output, result);

    // Show top files with lint issues (consistent with other analyze commands)
    output.push_str("## Top Files with Lint Issues\n\n");
    let sorted_files = sort_files_by_density(result);

    // `--top-files` is documented as "0 = all"; this used to silently show 10
    // for 0, so the flag did not do what it says.
    let files_to_show = if _top_files == 0 {
        sorted_files.len()
    } else {
        _top_files
    };
    if sorted_files.len() > files_to_show {
        // A TOTAL THAT IS SECRETLY A CAP is a fabrication: name both numbers.
        output.push_str(&format!(
            "_Showing {} of {} files with issues (--top-files {})._\n\n",
            files_to_show,
            sorted_files.len(),
            files_to_show
        ));
    }
    for (i, (file, summary)) in sorted_files.iter().take(files_to_show).enumerate() {
        let filename = file.file_name().unwrap_or_default().to_string_lossy();
        // measured_or_absent: with sloc 0 the density is not a measurement, it
        // is the `violations / 0 -> 0.0` guard. Never print it as "0.00".
        if summary.sloc == 0 {
            output.push_str(&format!(
                "{}. `{}` - {} violations, SLOC not measured (density unavailable)\n",
                i + 1,
                filename,
                summary.total_violations
            ));
        } else {
            output.push_str(&format!(
                "{}. `{}` - {:.2} violations/SLOC ({} violations, {} SLOC)\n",
                i + 1,
                filename,
                summary.defect_density,
                summary.total_violations,
                summary.sloc
            ));
        }
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
    // DETERMINISM: sorting on the count alone left equal-count files in
    // `HashMap` order, so this list was reshuffled on every run.
    let mut sorted_files: Vec<_> = result.summary_by_file.iter().collect();
    sorted_files.sort_by(|a, b| {
        b.1.total_violations
            .cmp(&a.1.total_violations)
            .then_with(|| a.0.cmp(b.0))
    });

    let files_to_show = if top_files == 0 {
        sorted_files.len()
    } else {
        top_files
    };
    if sorted_files.len() > files_to_show {
        output.push_str(&format!(
            "_Showing {} of {} files with issues (--top-files {})._\n",
            files_to_show,
            sorted_files.len(),
            files_to_show
        ));
    }
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

/// Write the two count lines that head every summary.
///
/// #700: the first line said "**Total Project Violations**" unconditionally, but
/// `total_project_violations` is — in BOTH modes — the sum over the files in
/// `summary_by_file`, and `--file` mode puts exactly the target file there.
/// Observed on a two-binary fixture with 20 project-wide findings:
/// `--file src/main.rs --format summary` printed
/// "**Total Project Violations**: 14" — 14 is that one file's count, not the
/// project's. Widening the number is not an option (`--file` deliberately
/// measures one file, and the quality gate keys off its density), so the report
/// now names the scope the number actually covers.
fn push_totals_header(output: &mut String, result: &LintHotspotResult) {
    if let Some(only_file) = single_file_scope(result) {
        output.push_str(&format!(
            "**Total Violations in `{}`**: {}\n",
            only_file.display(),
            result.total_project_violations
        ));
        output.push_str(
            "**Files with Issues**: 1 (only this file is included in the total above)\n\n",
        );
    } else {
        output.push_str(&format!(
            "**Total Project Violations**: {}\n",
            result.total_project_violations
        ));
        output.push_str(&format!(
            "**Files with Issues**: {}\n\n",
            result.summary_by_file.len()
        ));
    }
}

/// The single file every reported violation belongs to, when the result covers
/// exactly one file — which is what `--file` mode always produces.
///
/// #700: `LintHotspotResult` carries no scope marker (adding one would touch
/// every constructor), but the totals are derivable: `total_project_violations`
/// is the sum over `summary_by_file`, so when that map holds only the hotspot
/// itself the "total" is that one file's total and must not be announced as the
/// project's. A project scan whose only dirty file is the hotspot lands here
/// too, and the sentence is equally true of it.
fn single_file_scope(result: &LintHotspotResult) -> Option<&std::path::PathBuf> {
    if result.summary_by_file.len() == 1
        && result.summary_by_file.contains_key(&result.hotspot.file)
    {
        Some(&result.hotspot.file)
    } else {
        None
    }
}

/// Order `summary_by_file` deterministically: density descending, path ascending.
///
/// DETERMINISM: the previous comparator looked only at `defect_density`, so
/// every file that tied (very common — most files sit at the same small
/// density) came out in `HashMap` order and the rendered list differed between
/// runs on an unchanged tree.
fn sort_files_by_density(result: &LintHotspotResult) -> Vec<(&std::path::PathBuf, &FileSummary)> {
    let mut sorted: Vec<_> = result.summary_by_file.iter().collect();
    sorted.sort_by(|a, b| {
        b.1.defect_density
            .partial_cmp(&a.1.defect_density)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    sorted
}

/// Render a measured-clean project in the requested format.
///
/// #679: the clean path used to print a line to stderr and emit NOTHING on
/// stdout, so `--format json` produced an empty document. `hotspot` is
/// explicitly `null` — there is no hotspot, and inventing a zeroed one would be
/// a fabricated measurement.
pub(crate) fn format_clean_result(format: &LintHotspotOutputFormat) -> Result<String> {
    let empty_gate = serde_json::json!({
        "passed": true,
        "violations": [],
        "blocking": false
    });

    match format {
        LintHotspotOutputFormat::Json | LintHotspotOutputFormat::EnforcementJson => {
            serde_json::to_string_pretty(&serde_json::json!({
                "hotspot": serde_json::Value::Null,
                "all_violations": [],
                "summary_by_file": {},
                "total_project_violations": 0,
                "enforcement": serde_json::Value::Null,
                "refactor_chain": serde_json::Value::Null,
                "quality_gate": empty_gate,
            }))
            .context("Failed to serialize clean result to JSON")
        }
        LintHotspotOutputFormat::Sarif => serde_json::to_string_pretty(&serde_json::json!({
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [{
                "tool": { "driver": {
                    "name": "pmat-lint-hotspot",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }},
                "results": []
            }]
        }))
        .context("Failed to serialize clean SARIF"),
        LintHotspotOutputFormat::Summary | LintHotspotOutputFormat::Detailed => Ok(concat!(
            "# Lint Hotspot Analysis (EXTREME Quality Mode)\n\n",
            "**Total Project Violations**: 0\n",
            "**Files with Issues**: 0\n\n",
            "`cargo clippy` ran to completion and reported no violations. ",
            "There is no hotspot file.\n"
        )
        .to_string()),
    }
}

/// Format JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
pub(super) fn format_json(result: &LintHotspotResult, enforcement: bool) -> Result<String> {
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
pub(super) fn format_sarif(result: &LintHotspotResult) -> Result<String> {
    // SARIF used to be built from `quality_gate.violations` — the handful of
    // threshold breaches — so whenever the gate passed the document was
    // `"results": []` even though `-f detailed` listed 13 located clippy findings
    // for the same run, and CI consuming the SARIF saw a clean project. The located
    // findings in `all_violations` are the results; the gate breaches ride along
    // after them without a region, since they describe the project, not a line.
    let mut results: Vec<serde_json::Value> = result
        .all_violations
        .iter()
        .map(|v| {
            serde_json::json!({
                "ruleId": v.lint_name,
                "level": if v.severity == "error" { "error" } else { "warning" },
                "message": { "text": v.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": v.file.to_string_lossy()
                        },
                        "region": {
                            "startLine": v.line.max(1),
                            "startColumn": v.column.max(1),
                            "endLine": v.end_line.max(v.line).max(1),
                            "endColumn": v.end_column.max(1)
                        }
                    }
                }]
            })
        })
        .collect();

    results.extend(result.quality_gate.violations.iter().map(|v| {
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
    }));

    serde_json::to_string_pretty(&sarif_envelope(results)).context("Failed to serialize to SARIF")
}

/// The single SARIF envelope builder, shared by the populated and the
/// "nothing found" renderers so the two cannot disagree about tool metadata.
fn sarif_envelope(results: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
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
            "results": results
        }]
    })
}

/// Render the report for a project with zero lint violations.
///
/// Two defects are fixed here at once.
///
/// 1. `--format summary` and `--format detailed` used to write **zero bytes**
///    to stdout on a clean project (md5 `d41d8cd9…`, the empty digest) and put
///    everything on stderr, so `pmat analyze lint-hotspot | …` handed the pipe
///    nothing. Analysis output belongs on stdout.
/// 2. `--format json` and `--format enforcement-json` used to emit one shared
///    174-byte blob (md5 `572e6006…`) — two separately declared formats with
///    different `--help` descriptions producing byte-identical output. Each
///    format now emits the *same key set it emits when violations exist*:
///    `json` the two-key summary document, `enforcement-json` the full
///    enforcement document.
///
/// `hotspot` is `null` rather than a zero-filled object: there is no hotspot to
/// report, and a fabricated `{"sloc": 0, "defect_density": 0.0}` would read as
/// a measurement.
pub(crate) fn format_clean_output(
    format: &LintHotspotOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    let doc = match format {
        LintHotspotOutputFormat::Summary => return Ok(clean_human_report(false, perf, elapsed)),
        LintHotspotOutputFormat::Detailed => return Ok(clean_human_report(true, perf, elapsed)),
        // Mirrors `SimpleResult` in `format_json` (hotspot + quality_gate).
        LintHotspotOutputFormat::Json => serde_json::json!({
            "hotspot": serde_json::Value::Null,
            "quality_gate": clean_quality_gate(),
        }),
        // Mirrors the full `LintHotspotResult` serialization.
        LintHotspotOutputFormat::EnforcementJson => serde_json::json!({
            "hotspot": serde_json::Value::Null,
            "all_violations": [],
            "summary_by_file": {},
            "total_project_violations": 0,
            "enforcement": serde_json::Value::Null,
            "refactor_chain": serde_json::Value::Null,
            "quality_gate": clean_quality_gate(),
        }),
        LintHotspotOutputFormat::Sarif => sarif_envelope(vec![]),
    };

    serde_json::to_string_pretty(&doc).context("Failed to serialize clean lint-hotspot result")
}

/// Quality-gate object for a project with no violations.
fn clean_quality_gate() -> serde_json::Value {
    serde_json::json!({
        "passed": true,
        "violations": [],
        "blocking": false,
    })
}

/// Human-readable report for a clean project. `detailed` adds the two extra
/// sections `format_detailed` adds, so the two formats never coincide.
fn clean_human_report(detailed: bool, perf: bool, elapsed: std::time::Duration) -> String {
    let mut output = String::new();
    output.push_str("# Lint Hotspot Analysis (EXTREME Quality Mode)\n\n");
    output.push_str("**Total Project Violations**: 0\n");
    output.push_str("**Files with Issues**: 0\n\n");
    output.push_str("## Top Files with Lint Issues\n\n");
    output.push_str("_No lint violations found — project is clean._\n\n");
    output.push_str("## Hottest File Details\n");
    output.push_str("**File**: none (no file has any violation)\n");

    if detailed {
        output.push_str("\n## Detailed Violations in Hotspot File\n");
        output.push_str("_None._\n");
        output.push_str("\n## Top Files by Violations\n");
        output.push_str("_None._\n");
    }

    if perf {
        output.push_str(&format!(
            "\n⏱️  Analysis completed in {:.2}s\n",
            elapsed.as_secs_f64()
        ));
    }

    output
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

    /// A located clippy finding must appear as a SARIF result even when the quality
    /// gate passes — SARIF used to serialise only the gate's threshold breaches, so a
    /// project with 13 located violations handed CI `"results": []`.
    #[test]
    fn test_sarif_emits_located_violations_when_gate_passes() {
        let mut result = empty_result();
        result.all_violations.push(ViolationDetail {
            file: "src/lib.rs".into(),
            line: 17,
            column: 40,
            end_line: 17,
            end_column: 52,
            lint_name: "clippy::clone_on_copy".to_string(),
            message: "using `clone` on type `i32` which implements the `Copy` trait".to_string(),
            severity: "warning".to_string(),
            suggestion: None,
            machine_applicable: true,
        });
        // Gate passes: no threshold breaches at all.
        assert!(result.quality_gate.passed);

        let out = format_output(
            &result,
            LintHotspotOutputFormat::Sarif,
            false,
            std::time::Duration::from_millis(10),
            5,
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(
            results.len(),
            1,
            "located violation missing from SARIF: {out}"
        );
        assert_eq!(results[0]["ruleId"], "clippy::clone_on_copy");
        let region = &results[0]["locations"][0]["physicalLocation"]["region"];
        assert_eq!(region["startLine"], 17);
        assert_eq!(region["startColumn"], 40);
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
            "src/lib.rs"
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

    // ── clean-project output (the round-1 regression) ───────────────────────

    fn clean(format: LintHotspotOutputFormat) -> String {
        format_clean_output(&format, false, std::time::Duration::from_millis(7))
            .unwrap_or_else(|e| panic!("{format} must render a clean report: {e}"))
    }

    /// Every declared format must put SOMETHING on stdout on a clean project.
    /// `summary` and `detailed` shipped writing zero bytes (md5 d41d8cd9…).
    #[test]
    fn test_clean_output_no_format_is_empty() {
        for format in [
            LintHotspotOutputFormat::Summary,
            LintHotspotOutputFormat::Detailed,
            LintHotspotOutputFormat::Json,
            LintHotspotOutputFormat::EnforcementJson,
            LintHotspotOutputFormat::Sarif,
        ] {
            let out = clean(format.clone());
            assert!(!out.trim().is_empty(), "{format} emitted zero bytes");
        }
    }

    /// `json` and `enforcement-json` shipped BYTE-IDENTICAL on a clean project
    /// (both md5 572e6006…, 174 bytes) despite being two declared formats with
    /// distinct `--help` text. No two declared formats may coincide.
    #[test]
    fn test_clean_output_formats_are_pairwise_distinct() {
        let formats = [
            LintHotspotOutputFormat::Summary,
            LintHotspotOutputFormat::Detailed,
            LintHotspotOutputFormat::Json,
            LintHotspotOutputFormat::EnforcementJson,
            LintHotspotOutputFormat::Sarif,
        ];
        for (i, a) in formats.iter().enumerate() {
            for b in formats.iter().skip(i + 1) {
                assert_ne!(
                    clean(a.clone()),
                    clean(b.clone()),
                    "--format {a} and --format {b} produced identical bytes"
                );
            }
        }
    }

    /// The clean document must carry the same key set the populated document
    /// carries, so a consumer does not have to special-case "no findings".
    #[test]
    fn test_clean_json_key_sets_match_populated_key_sets() {
        let populated_simple: serde_json::Value =
            serde_json::from_str(&format_json(&empty_result(), false).unwrap()).unwrap();
        let clean_simple: serde_json::Value =
            serde_json::from_str(&clean(LintHotspotOutputFormat::Json)).unwrap();
        let mut a: Vec<_> = populated_simple.as_object().unwrap().keys().collect();
        let mut b: Vec<_> = clean_simple.as_object().unwrap().keys().collect();
        a.sort();
        b.sort();
        assert_eq!(a, b, "json clean/populated key sets diverge");

        let populated_full: serde_json::Value =
            serde_json::from_str(&format_json(&empty_result(), true).unwrap()).unwrap();
        let clean_full: serde_json::Value =
            serde_json::from_str(&clean(LintHotspotOutputFormat::EnforcementJson)).unwrap();
        let mut a: Vec<_> = populated_full.as_object().unwrap().keys().collect();
        let mut b: Vec<_> = clean_full.as_object().unwrap().keys().collect();
        a.sort();
        b.sort();
        assert_eq!(a, b, "enforcement-json clean/populated key sets diverge");
    }

    /// `hotspot` must be absent (null), never a zero-filled object: a
    /// fabricated `sloc: 0 / defect_density: 0.0` reads as a measurement.
    #[test]
    fn test_clean_json_reports_hotspot_as_absent_not_zero() {
        for format in [
            LintHotspotOutputFormat::Json,
            LintHotspotOutputFormat::EnforcementJson,
        ] {
            let doc: serde_json::Value = serde_json::from_str(&clean(format.clone())).unwrap();
            assert!(doc["hotspot"].is_null(), "{format} fabricated a hotspot");
        }
    }

    /// Identical input, five runs, identical bytes — for every format.
    #[test]
    fn test_output_is_deterministic_across_five_runs() {
        for format in [
            LintHotspotOutputFormat::Summary,
            LintHotspotOutputFormat::Detailed,
            LintHotspotOutputFormat::Json,
            LintHotspotOutputFormat::EnforcementJson,
            LintHotspotOutputFormat::Sarif,
        ] {
            let mut baseline: Option<String> = None;
            for run in 0..5 {
                // Rebuild the result each time: `summary_by_file` is a HashMap
                // whose iteration order is randomised per map instance.
                let out = format_output(
                    &multi_file_result(),
                    format.clone(),
                    false,
                    std::time::Duration::from_millis(1),
                    10,
                )
                .unwrap();
                match &baseline {
                    None => baseline = Some(out),
                    Some(first) => assert_eq!(
                        first, &out,
                        "--format {format} differed on run {run} for identical input"
                    ),
                }
            }
        }
    }

    /// Several files at an identical defect density, so any hash-order
    /// dependence shows up as a reordered report.
    fn multi_file_result() -> LintHotspotResult {
        let mut summary_by_file = std::collections::HashMap::new();
        let mut all_violations = Vec::new();
        for name in [
            "src/alpha.rs",
            "src/beta.rs",
            "src/gamma.rs",
            "src/delta.rs",
            "src/epsilon.rs",
            "src/zeta.rs",
            "src/eta.rs",
            "src/theta.rs",
        ] {
            summary_by_file.insert(
                std::path::PathBuf::from(name),
                FileSummary {
                    total_violations: 4,
                    errors: 1,
                    warnings: 3,
                    sloc: 100,
                    defect_density: 0.04,
                },
            );
            all_violations.push(ViolationDetail {
                file: std::path::PathBuf::from(name),
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 2,
                lint_name: "clippy::needless_range_loop".to_string(),
                message: "m".to_string(),
                severity: "warning".to_string(),
                suggestion: None,
                machine_applicable: false,
            });
        }

        let mut result = empty_result();
        result.summary_by_file = summary_by_file;
        result.all_violations = all_violations;
        result.total_project_violations = 32;
        result
    }

    #[test]
    fn test_format_sarif_empty_result_valid_json() {
        let out = format_sarif(&empty_result()).unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_clean_result_is_non_empty_in_every_declared_format() {
        // #679: the clean path wrote to STDERR and emitted NOTHING on stdout, so
        // `--format json` — a declared format — produced an empty document.
        for format in [
            LintHotspotOutputFormat::Json,
            LintHotspotOutputFormat::EnforcementJson,
            LintHotspotOutputFormat::Sarif,
            LintHotspotOutputFormat::Summary,
            LintHotspotOutputFormat::Detailed,
        ] {
            let out = format_clean_result(&format).unwrap();
            assert!(!out.trim().is_empty(), "empty output for {format:?}");
        }
    }

    #[test]
    fn test_clean_json_says_hotspot_null_not_a_zeroed_hotspot() {
        // A zeroed hotspot with a made-up file path would be a fabricated
        // measurement; the absence must be explicit.
        let out = format_clean_result(&LintHotspotOutputFormat::Json).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["hotspot"].is_null());
        assert_eq!(v["total_project_violations"], 0);
        assert!(v["quality_gate"]["passed"].as_bool().unwrap());
    }

    #[test]
    fn test_clean_sarif_is_a_valid_envelope_with_no_results() {
        let out = format_clean_result(&LintHotspotOutputFormat::Sarif).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["version"], "2.1.0");
        assert_eq!(v["runs"][0]["results"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_summary_names_both_numbers_when_top_files_truncates() {
        // "A TOTAL THAT IS SECRETLY A CAP is a fabrication": when the list is
        // cut to --top-files, say how many files there really are.
        let mut result = empty_result();
        for i in 0..7 {
            result.summary_by_file.insert(
                std::path::PathBuf::from(format!("src/f{i}.rs")),
                FileSummary {
                    total_violations: i + 1,
                    errors: 0,
                    warnings: i + 1,
                    sloc: 100,
                    defect_density: (i + 1) as f64 / 100.0,
                },
            );
        }
        let out = format_summary(&result, false, std::time::Duration::from_secs(0), 3).unwrap();
        assert!(out.contains("Showing 3 of 7 files"), "{out}");
    }

    #[test]
    fn test_top_files_zero_means_all_as_documented() {
        // `--top-files` help says "0 = all"; the summary renderer used to
        // substitute 10, so `0` silently truncated a 12-file list to 10.
        let mut result = empty_result();
        for i in 0..12 {
            result.summary_by_file.insert(
                std::path::PathBuf::from(format!("src/f{i:02}.rs")),
                FileSummary {
                    total_violations: i + 1,
                    errors: 0,
                    warnings: i + 1,
                    sloc: 100,
                    defect_density: (i + 1) as f64 / 100.0,
                },
            );
        }
        let out = format_summary(&result, false, std::time::Duration::from_secs(0), 0).unwrap();
        // Count only the numbered file-list lines (the "Hottest File Details"
        // block also carries the "violations/SLOC" suffix).
        let listed = out
            .lines()
            .filter(|l| l.contains("violations/SLOC") && l.contains("`f"))
            .count();
        assert_eq!(listed, 12, "--top-files 0 must list every file:\n{out}");
        assert!(
            !out.contains("Showing"),
            "no truncation notice when nothing is cut"
        );
    }

    #[test]
    fn test_sloc_zero_is_reported_as_unmeasured_not_as_zero_density() {
        // measured_or_absent: `violations / 0` returns the 0.0 guard, which is
        // not a measurement and must not print as "0.00 violations/SLOC".
        let mut result = empty_result();
        result.summary_by_file.insert(
            std::path::PathBuf::from("src/ghost.rs"),
            FileSummary {
                total_violations: 7,
                errors: 0,
                warnings: 7,
                sloc: 0,
                defect_density: 0.0,
            },
        );
        let out = format_summary(&result, false, std::time::Duration::from_secs(0), 10).unwrap();
        assert!(out.contains("SLOC not measured"), "{out}");
        assert!(
            !out.contains("`ghost.rs` - 0.00 violations/SLOC"),
            "an unmeasurable density was rendered as 0.00:\n{out}"
        );
    }

    /// #700: `--file` mode's total is the target file's total, and the header
    /// announced it as the PROJECT's. Measured on a two-binary fixture whose
    /// project total is 20: `--file src/main.rs --format summary` printed
    /// "**Total Project Violations**: 14".
    #[test]
    fn test_single_file_scope_total_is_not_called_a_project_total() {
        let mut result = empty_result();
        // Exactly what `create_single_file_result` builds: one entry, keyed by
        // the target file, and a "total" that is only that file's.
        result.hotspot.file = std::path::PathBuf::from("src/main.rs");
        result.hotspot.total_violations = 14;
        result.hotspot.sloc = 11;
        result.hotspot.defect_density = 14.0 / 11.0;
        result.summary_by_file.insert(
            std::path::PathBuf::from("src/main.rs"),
            FileSummary {
                total_violations: 14,
                errors: 0,
                warnings: 14,
                sloc: 11,
                defect_density: 14.0 / 11.0,
            },
        );
        result.total_project_violations = 14;

        let out = format_summary(&result, false, std::time::Duration::from_secs(0), 10).unwrap();
        assert!(
            !out.contains("**Total Project Violations**"),
            "14 is one file's count; calling it the project total is the #700 defect:\n{out}"
        );
        assert!(
            out.contains("**Total Violations in `src/main.rs`**: 14"),
            "the total must name the scope it covers:\n{out}"
        );
    }

    /// The relabelling must not leak into a genuine project scan.
    #[test]
    fn test_project_scope_total_is_still_called_a_project_total() {
        let out = format_summary(
            &multi_file_result(),
            false,
            std::time::Duration::from_secs(0),
            10,
        )
        .unwrap();
        assert!(out.contains("**Total Project Violations**"), "{out}");
    }

    #[test]
    fn test_summary_is_byte_identical_across_repeated_renders() {
        // DETERMINISM over >= 5 iterations: files that TIE on defect_density
        // used to come out of the HashMap in a different order each render.
        let mut result = empty_result();
        for i in 0..12 {
            result.summary_by_file.insert(
                std::path::PathBuf::from(format!("src/tie{i:02}.rs")),
                FileSummary {
                    total_violations: 2,
                    errors: 0,
                    warnings: 2,
                    sloc: 100,
                    defect_density: 0.02,
                },
            );
        }
        let renders: Vec<String> = (0..8)
            .map(|_| format_summary(&result, false, std::time::Duration::from_secs(0), 10).unwrap())
            .collect();
        if let Some(i) = (1..renders.len()).find(|&i| renders[i] != renders[i - 1]) {
            panic!("summary render differs between runs {}/{i}", i - 1);
        }
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
