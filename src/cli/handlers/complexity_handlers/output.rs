//! Output formatting and display for complexity and SATD analysis results.

use crate::cli::ComplexityOutputFormat;
use crate::services::complexity::{ComplexityReport, FileComplexityMetrics};
use crate::services::satd_detector::SATDAnalysisResult;
use anyhow::Result;
use std::path::PathBuf;

/// What the emitted `files` list is a slice of.
///
/// Exists so a capped list can never be mistaken for the whole analysis:
/// `summary.total_files` describes what the aggregates were computed over,
/// `files_listed` describes the array itself.
pub(super) struct ListingDisclosure {
    pub top_files: usize,
    pub files_listed: usize,
    pub files_analyzed: usize,
    pub files_discovered: usize,
    /// Issue #1050 P3: what the walk saw and did not measure, per reason.
    /// `None` where there was no population to compare against (single-file
    /// and explicit-file modes) — which is NOT the same as "nothing was
    /// skipped", and serialises as `null` rather than an empty breakdown.
    pub skipped_note: Option<SkipCensus>,
    pub truncated: bool,
}

/// The walk's skip breakdown, in the units the human note already prints.
pub(super) struct SkipCensus {
    pub not_analyzed: usize,
    pub no_analyzer: std::collections::BTreeMap<String, usize>,
    pub unmeasured_supported: std::collections::BTreeMap<String, usize>,
    pub excluded_by_ignore: std::collections::BTreeMap<String, usize>,
}

/// Format and write complexity analysis output
pub(super) async fn format_and_write_output(
    summary: &ComplexityReport,
    _file_metrics: &[FileComplexityMetrics],
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    listing: ListingDisclosure,
) -> Result<()> {
    use crate::services::complexity::{
        format_as_sarif, format_complexity_report, format_complexity_summary,
    };

    let formatted_output = match format {
        ComplexityOutputFormat::Summary => Ok(format_complexity_summary(summary)),
        ComplexityOutputFormat::Full => Ok(format_complexity_report(summary)),
        ComplexityOutputFormat::Sarif => {
            format_as_sarif(summary).map_err(|e| anyhow::anyhow!("SARIF serialization failed: {e}"))
        }
        ComplexityOutputFormat::Json => {
            let mut json_output = serde_json::to_value(summary)
                .map_err(|e| anyhow::anyhow!("JSON conversion failed: {e}"))?;

            if let Some(obj) = json_output.as_object_mut() {
                if listing.top_files > 0 {
                    obj.insert(
                        "top_files_limit".to_string(),
                        serde_json::json!(listing.top_files),
                    );
                }
                // Name both numbers: the array is a slice, the summary is not.
                obj.insert(
                    "files_listed".to_string(),
                    serde_json::json!(listing.files_listed),
                );
                obj.insert(
                    "files_analyzed".to_string(),
                    serde_json::json!(listing.files_analyzed),
                );
                obj.insert(
                    "files_discovered".to_string(),
                    serde_json::json!(listing.files_discovered),
                );
                obj.insert(
                    "files_truncated".to_string(),
                    serde_json::json!(listing.truncated),
                );
                // Issue #1050 P3. `files_discovered` above is now the walk's
                // count, not the metrics vector's length; this is the rest of
                // the sentence the human formatter already prints, so a
                // consumer can act on the gap instead of only seeing it.
                obj.insert(
                    "files_not_analyzed".to_string(),
                    match &listing.skipped_note {
                        Some(c) => serde_json::json!({
                            "total": c.not_analyzed,
                            "no_complexity_analyzer": c.no_analyzer,
                            "supported_but_unmeasured": c.unmeasured_supported,
                            "excluded_by_ignore_rules": c.excluded_by_ignore,
                        }),
                        // Not `{"total": 0}`: no census means no population was
                        // compared, and a fabricated zero is exactly the
                        // "absence rendered as success" this field exists to
                        // stop.
                        None => serde_json::Value::Null,
                    },
                );
            }

            serde_json::to_string_pretty(&json_output)
                .map_err(|e| anyhow::anyhow!("JSON serialization failed: {e}"))
        }
    }?;

    if listing.truncated {
        crate::status_eprintln!(
            "ℹ️  Listing the {} most complex of {} analyzed files (--top-files {}); \
             the summary covers all {}.",
            listing.files_listed,
            listing.files_analyzed,
            listing.top_files,
            listing.files_analyzed
        );
    }

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &formatted_output).await?;
        crate::status_eprintln!("📝 Results written to: {}", output_path.display());
    } else {
        println!("{formatted_output}");
    }

    Ok(())
}

/// Write top files with SATD section
pub(super) fn write_top_files_with_satd_section(output: &mut String, result: &SATDAnalysisResult) {
    use crate::cli::colors as c;
    use std::collections::HashMap;
    use std::fmt::Write;

    writeln!(output, "\n{}\n", c::subheader("Top Files with SATD")).expect("internal error");

    // Group items by file and count them
    let mut file_counts: HashMap<&std::path::Path, usize> = HashMap::new();
    for item in &result.items {
        *file_counts.entry(&item.file).or_insert(0) += 1;
    }

    // Sort files by SATD count (descending)
    let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
    sorted_files.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    // Show top 10 files with their SATD counts
    for (i, (file, count)) in sorted_files.iter().take(10).enumerate() {
        let filename = file.file_name().unwrap_or_default().to_string_lossy();
        writeln!(
            output,
            "  {}. {} - {} SATD items",
            c::number(&(i + 1).to_string()),
            c::path(&filename),
            c::number(&count.to_string())
        )
        .expect("internal error");
    }
}

/// Write critical SATD items section
pub(super) fn write_critical_items_section(output: &mut String, result: &SATDAnalysisResult) {
    use crate::cli::colors as c;
    use std::fmt::Write;

    writeln!(output, "\n{}\n", c::subheader("Critical Items")).expect("internal error");
    for item in result
        .items
        .iter()
        .filter(|i| i.severity == crate::services::satd_detector::Severity::Critical)
        .take(5)
    {
        writeln!(
            output,
            "  {}{}!{} {}:{} - {}",
            c::BOLD,
            c::RED,
            c::RESET,
            c::path(&item.file.file_name().unwrap_or_default().to_string_lossy()),
            c::dim(&item.line.to_string()),
            item.text
        )
        .expect("internal error");
    }
}

/// Generate SARIF format for SATD results
pub(super) fn generate_satd_sarif(result: &SATDAnalysisResult) -> serde_json::Value {
    serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": [{
                        "id": "satd",
                        "name": "Self-Admitted Technical Debt",
                        "shortDescription": {
                            "text": "Technical debt explicitly documented in code comments"
                        },
                        "fullDescription": {
                            "text": "Detects TODO, FIXME, HACK, and other technical debt markers in comments"
                        },
                        "defaultConfiguration": {
                            "level": "warning"
                        }
                    }]
                }
            },
            "results": result.items.iter().map(|item| {
                let level = match item.severity {
                    crate::services::satd_detector::Severity::Critical => "error",
                    crate::services::satd_detector::Severity::High => "error",
                    crate::services::satd_detector::Severity::Medium => "warning",
                    crate::services::satd_detector::Severity::Low => "note",
                };
                serde_json::json!({
                    "ruleId": "satd",
                    "level": level,
                    "message": {
                        "text": format!("{} debt: {}", item.category, item.text)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": item.file.to_string_lossy()
                            },
                            "region": {
                                "startLine": item.line,
                                "startColumn": item.column
                            }
                        }
                    }]
                })
            }).collect::<Vec<_>>()
        }]
    })
}

/// Format SATD summary
///
/// # Example
///
/// ```no_run
/// use pmat::services::satd_detector::{SATDAnalysisResult, SATDSummary, SkipCounts, TechnicalDebt, DebtCategory, Severity};
/// use pmat::cli::handlers::complexity_handlers::format_satd_summary;
/// use std::collections::HashMap;
/// use std::path::PathBuf;
/// use chrono::Utc;
///
/// let result = SATDAnalysisResult {
///     items: vec![
///         TechnicalDebt {
///             category: DebtCategory::Defect,
///             severity: Severity::High,
///             text: "Handle error properly".to_string(),
///             file: PathBuf::from("src/main.rs"),
///             line: 42,
///             column: 8,
///             context_hash: [0; 16],
///         },
///         TechnicalDebt {
///             category: DebtCategory::Requirement,
///             severity: Severity::Medium,
///             text: "Add validation".to_string(),
///             file: PathBuf::from("src/lib.rs"),
///             line: 25,
///             column: 4,
///             context_hash: [1; 16],
///         },
///     ],
///     summary: SATDSummary {
///         total_items: 2,
///         by_severity: HashMap::new(),
///         by_category: HashMap::new(),
///         files_with_satd: 2,
///         avg_age_days: 30.0,
///     },
///     total_files_analyzed: 10,
///     files_with_debt: 2,
///     // All 10 candidate files were read, so nothing was skipped.
///     skipped: SkipCounts::default(),
///     analysis_timestamp: Utc::now(),
/// };
///
/// let summary = format_satd_summary(&result, false);
///
/// assert!(summary.contains("SATD Analysis Summary"));
/// assert!(summary.contains("Files analyzed:"));
/// assert!(summary.contains("10"));
/// assert!(summary.contains("Files with SATD:"));
/// assert!(summary.contains("2"));
/// assert!(summary.contains("Total SATD items:"));
/// assert!(summary.contains("Top Files with SATD"));
/// // Note: Files are sorted by SATD count, then alphabetically
/// assert!(summary.contains("SATD items"));
/// assert!(summary.contains("SATD items"));
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_satd_summary(result: &SATDAnalysisResult, metrics: bool) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "{}\n", c::header("SATD Analysis Summary")).expect("internal error");
    writeln!(
        &mut output,
        "  {} {}",
        c::label("Files analyzed:"),
        c::number(&result.total_files_analyzed.to_string())
    )
    .expect("internal error");
    writeln!(
        &mut output,
        "  {} {}",
        c::label("Files with SATD:"),
        c::number(&result.files_with_debt.to_string())
    )
    .expect("internal error");
    writeln!(
        &mut output,
        "  {} {}",
        c::label("Total SATD items:"),
        c::number(&result.items.len().to_string())
    )
    .expect("internal error");

    if metrics && !result.summary.by_severity.is_empty() {
        writeln!(&mut output, "\n{}\n", c::subheader("By Severity")).expect("internal error");
        for (severity, count) in &result.summary.by_severity {
            let sev_color = match severity.to_lowercase().as_str() {
                "critical" | "high" => c::RED,
                "medium" => c::YELLOW,
                _ => c::GREEN,
            };
            writeln!(
                &mut output,
                "  {}{severity}{}: {}",
                sev_color,
                c::RESET,
                c::number(&count.to_string())
            )
            .expect("internal error");
        }
    }

    if metrics && !result.summary.by_category.is_empty() {
        writeln!(&mut output, "\n{}\n", c::subheader("By Category")).expect("internal error");
        for (category, count) in &result.summary.by_category {
            writeln!(
                &mut output,
                "  {}: {}",
                c::label(category),
                c::number(&count.to_string())
            )
            .expect("internal error");
        }
    }

    // Show additional sections if items exist
    if !result.items.is_empty() {
        write_top_files_with_satd_section(&mut output, result);
        write_critical_items_section(&mut output, result);
    }

    output
}

/// Format SATD as markdown report
pub(super) fn format_satd_markdown(
    result: &SATDAnalysisResult,
    metrics: bool,
    _evolution: bool,
    _days: u32,
) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Self-Admitted Technical Debt Report\n").expect("internal error");
    writeln!(
        &mut output,
        "Generated: {}",
        result.analysis_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    )
    .expect("internal error");

    writeln!(&mut output, "\n## Summary\n").expect("internal error");
    writeln!(&mut output, "| Metric | Value |").expect("internal error");
    writeln!(&mut output, "|--------|-------|").expect("internal error");
    writeln!(
        &mut output,
        "| Files Analyzed | {} |",
        result.total_files_analyzed
    )
    .expect("internal error");
    writeln!(
        &mut output,
        "| Files with SATD | {} |",
        result.files_with_debt
    )
    .expect("internal error");
    writeln!(&mut output, "| Total SATD Items | {} |", result.items.len()).expect("internal error");

    if metrics {
        writeln!(&mut output, "\n## Distribution\n").expect("internal error");
        writeln!(&mut output, "### By Severity\n").expect("internal error");
        writeln!(&mut output, "| Severity | Count |").expect("internal error");
        writeln!(&mut output, "|----------|-------|").expect("internal error");
        for (severity, count) in &result.summary.by_severity {
            writeln!(&mut output, "| {severity} | {count} |").expect("internal error");
        }

        writeln!(&mut output, "\n### By Category\n").expect("internal error");
        writeln!(&mut output, "| Category | Count |").expect("internal error");
        writeln!(&mut output, "|----------|-------|").expect("internal error");
        for (category, count) in &result.summary.by_category {
            writeln!(&mut output, "| {category} | {count} |").expect("internal error");
        }
    }

    // Group items by file
    use std::collections::BTreeMap;
    let mut by_file: BTreeMap<
        &std::path::Path,
        Vec<&crate::services::satd_detector::TechnicalDebt>,
    > = BTreeMap::new();
    for item in &result.items {
        by_file.entry(&item.file).or_default().push(item);
    }

    writeln!(&mut output, "\n## SATD Items by File\n").expect("internal error");
    for (file, items) in by_file.iter().take(20) {
        writeln!(&mut output, "### {}\n", file.display()).expect("internal error");
        writeln!(&mut output, "| Line | Severity | Category | Text |").expect("internal error");
        writeln!(&mut output, "|------|----------|----------|------|").expect("internal error");
        for item in items {
            writeln!(
                &mut output,
                "| {} | {:?} | {} | {} |",
                item.line,
                item.severity,
                item.category,
                item.text.replace('|', "\\|")
            )
            .expect("internal error");
        }
        writeln!(&mut output).expect("internal error");
    }

    output
}

/// Format SATD output in the requested format
pub(super) fn format_satd_output(
    result: &SATDAnalysisResult,
    format: crate::cli::SatdOutputFormat,
    metrics: bool,
    evolution: bool,
    days: u32,
) -> Result<String> {
    use crate::cli::SatdOutputFormat;

    match format {
        SatdOutputFormat::Json => Ok(serde_json::to_string_pretty(&result)?),
        SatdOutputFormat::Sarif => {
            let sarif = generate_satd_sarif(result);
            Ok(serde_json::to_string_pretty(&sarif)?)
        }
        SatdOutputFormat::Summary => Ok(format_satd_summary(result, metrics)),
        SatdOutputFormat::Markdown => Ok(format_satd_markdown(result, metrics, evolution, days)),
    }
}

/// Write SATD output to file or stdout
pub(super) async fn write_satd_output(content: String, output: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod satd_formatting_tests {
    use super::*;
    use crate::services::satd_detector::{SATDAnalysisResult, SATDSummary};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_satd_result(
        items: Vec<crate::services::satd_detector::TechnicalDebt>,
    ) -> SATDAnalysisResult {
        SATDAnalysisResult {
            skipped: Default::default(),
            items,
            summary: SATDSummary {
                total_items: 0,
                by_severity: HashMap::new(),
                by_category: HashMap::new(),
                files_with_satd: 0,
                avg_age_days: 0.0,
            },
            total_files_analyzed: 5,
            files_with_debt: 2,
            analysis_timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_format_satd_summary_empty() {
        let result = create_test_satd_result(vec![]);
        let output = format_satd_summary(&result, false);

        assert!(output.contains("SATD Analysis Summary"));
        assert!(output.contains("Files analyzed"));
        assert!(output.contains("5")); // total_files_analyzed
        assert!(output.contains("Files with SATD"));
        assert!(output.contains("2")); // files_with_debt
    }

    #[test]
    fn test_format_satd_summary_with_metrics() {
        let mut result = create_test_satd_result(vec![]);
        result.summary.by_severity.insert("high".to_string(), 3);
        result.summary.by_severity.insert("low".to_string(), 5);
        result.summary.by_category.insert("TODO".to_string(), 4);

        let output = format_satd_summary(&result, true);

        assert!(output.contains("By Severity"));
        assert!(output.contains("high"));
        assert!(output.contains("3"));
        assert!(output.contains("By Category"));
        assert!(output.contains("TODO"));
    }

    #[test]
    fn test_format_satd_summary_no_metrics() {
        let mut result = create_test_satd_result(vec![]);
        result.summary.by_severity.insert("high".to_string(), 3);

        let output = format_satd_summary(&result, false);

        // With metrics=false, severity breakdown should not appear
        assert!(!output.contains("By Severity"));
    }
}
