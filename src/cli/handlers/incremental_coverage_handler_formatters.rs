// Formatting functions for incremental coverage results.
// Included by incremental_coverage_handler.rs — do NOT add `use` imports here.
//
// Coverage values are percentages in 0-100 and are `Option`: `None` means pmat
// did not measure the file, and must never render as 0.0% (GH #658). Every
// value here used to be multiplied by 100 on the way out, which is how the
// documented threshold default of 80.0 was printed as 8000.0%.

/// Render a measured percentage, or say plainly that it was not measured.
fn pct_or_unmeasured(value: Option<f64>) -> String {
    value.map_or_else(|| "not measured".to_string(), |v| format!("{v:.1}%"))
}

/// Render a signed delta, or say plainly that it was not measured.
fn delta_or_unmeasured(value: Option<f64>) -> String {
    value.map_or_else(|| "not measured".to_string(), |v| format!("{v:+.1}%"))
}

/// Row count `--top-files N` permits out of `total`, where `0` means all.
///
/// Every renderer below used a bare `.iter().take(top_files)`, which reads `0`
/// as "show nothing" — the exact opposite of the documented "0 = all", so
/// `--top-files 0` printed an empty file list and a report that looked clean.
/// The shared authority is `crate::cli::top_files_count`; this is its one call
/// site for this command so no renderer can drift from another.
fn incremental_rows_shown(total: usize, top_files: usize) -> usize {
    crate::cli::top_files_count(total, top_files)
}

/// Format as summary
fn format_summary(result: &IncrementalCoverageResult, top_files: usize) -> String {
    let mut output = String::new();
    output.push_str("# Incremental Coverage Summary\n\n");
    output.push_str(&result.summary);
    output.push_str("\n\n## Top Changed Files\n");

    let shown = incremental_rows_shown(result.changed_files.len(), top_files);
    for (i, file) in result.changed_files.iter().take(shown).enumerate() {
        output.push_str(&format!(
            "{}. {} - {} → {} (Δ{})\n",
            i + 1,
            file.file_path,
            pct_or_unmeasured(file.coverage_before),
            pct_or_unmeasured(file.coverage_after),
            delta_or_unmeasured(file.coverage_delta)
        ));
    }
    if shown < result.changed_files.len() {
        output.push_str(&format!(
            "… {} more not shown (--top-files {top_files}, 0 = all)\n",
            result.changed_files.len() - shown
        ));
    }

    output
}

/// Format as detailed report
fn format_detailed(result: &IncrementalCoverageResult, top_files: usize) -> String {
    let mut output = String::new();
    output.push_str("# Incremental Coverage Detailed Report\n\n");
    output.push_str(&format!("Total files analyzed: {}\n", result.total_files));
    output.push_str(&format!("Files with coverage: {}\n", result.covered_files));
    output.push_str(&format!(
        "Overall coverage: {}\n",
        pct_or_unmeasured(result.coverage_percentage)
    ));
    output.push_str(&format!(
        "Files above threshold: {}\n",
        result.files_above_threshold
    ));
    output.push_str(&format!(
        "Files below threshold: {}\n",
        result.files_below_threshold
    ));
    output.push_str(&format!(
        "Files not measured: {}\n\n",
        result.files_not_measured
    ));

    let shown = incremental_rows_shown(result.changed_files.len(), top_files);
    output.push_str(&format!(
        "## Changed Files (showing {shown} of {})\n",
        result.changed_files.len()
    ));
    for file in result.changed_files.iter().take(shown) {
        output.push_str(&format!("\n### {}\n", file.file_path));
        output.push_str(&format!("- Status: {:?}\n", file.status));
        output.push_str(&format!(
            "- Coverage: {} → {}\n",
            pct_or_unmeasured(file.coverage_before),
            pct_or_unmeasured(file.coverage_after)
        ));
        output.push_str(&format!(
            "- Delta: {}\n",
            delta_or_unmeasured(file.coverage_delta)
        ));
        output.push_str(&format!(
            "- Lines: {}/{}\n",
            file.lines_covered, file.lines_total
        ));
    }

    output
}

/// Format as Markdown
fn format_markdown(result: &IncrementalCoverageResult, top_files: usize) -> String {
    let mut output = String::new();
    output.push_str("# Incremental Coverage Report\n\n");
    output.push_str(&format!("**Summary:** {}\n\n", result.summary));

    output.push_str("## Metrics\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| Total Files | {} |\n", result.total_files));
    output.push_str(&format!("| Covered Files | {} |\n", result.covered_files));
    output.push_str(&format!(
        "| Coverage | {} |\n",
        pct_or_unmeasured(result.coverage_percentage)
    ));
    output.push_str(&format!(
        "| Above Threshold | {} |\n",
        result.files_above_threshold
    ));
    output.push_str(&format!(
        "| Below Threshold | {} |\n",
        result.files_below_threshold
    ));
    output.push_str(&format!(
        "| Not Measured | {} |\n\n",
        result.files_not_measured
    ));

    output.push_str("## Top Changed Files\n\n");
    output.push_str("| File | Before | After | Delta | Status |\n");
    output.push_str("|------|--------|-------|-------|--------|\n");

    let shown = incremental_rows_shown(result.changed_files.len(), top_files);
    for file in result.changed_files.iter().take(shown) {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {:?} |\n",
            file.file_path,
            pct_or_unmeasured(file.coverage_before),
            pct_or_unmeasured(file.coverage_after),
            delta_or_unmeasured(file.coverage_delta),
            file.status
        ));
    }

    output
}

/// Format as LCOV
///
/// Only files whose coverage was actually measured are emitted: an LCOV record
/// is a measurement claim, and `LH:0/LF:0` for an unmeasured file reads as
/// "nothing is covered".
fn format_lcov(result: &IncrementalCoverageResult) -> String {
    let mut output = String::new();

    for file in result
        .changed_files
        .iter()
        .filter(|f| f.coverage_after.is_some())
    {
        output.push_str(&format!("SF:{}\n", file.file_path));
        output.push_str(&format!("LH:{}\n", file.lines_covered));
        output.push_str(&format!("LF:{}\n", file.lines_total));
        output.push_str("end_of_record\n");
    }

    output
}

/// Format as delta report
fn format_delta(result: &IncrementalCoverageResult, top_files: usize) -> String {
    let mut output = String::new();
    output.push_str("Coverage Delta Report\n");
    output.push_str("====================\n\n");

    let all_improved: Vec<_> = result
        .changed_files
        .iter()
        .filter(|f| f.coverage_delta.is_some_and(|d| d > 0.0))
        .collect();
    let improved = &all_improved[..incremental_rows_shown(all_improved.len(), top_files)];

    let all_degraded: Vec<_> = result
        .changed_files
        .iter()
        .filter(|f| f.coverage_delta.is_some_and(|d| d < 0.0))
        .collect();
    let degraded = &all_degraded[..incremental_rows_shown(all_degraded.len(), top_files)];

    let unmeasured = result
        .changed_files
        .iter()
        .filter(|f| f.coverage_delta.is_none())
        .count();

    if !improved.is_empty() {
        output.push_str("✅ Improved Coverage:\n");
        for file in improved {
            output.push_str(&format!(
                "  {} {}\n",
                file.file_path,
                delta_or_unmeasured(file.coverage_delta)
            ));
        }
        output.push('\n');
    }

    if !degraded.is_empty() {
        output.push_str("⚠️  Degraded Coverage:\n");
        for file in degraded {
            output.push_str(&format!(
                "  {} {}\n",
                file.file_path,
                delta_or_unmeasured(file.coverage_delta)
            ));
        }
        output.push('\n');
    }

    if unmeasured > 0 {
        output.push_str(&format!(
            "ℹ️  {unmeasured} changed file(s) had no coverage data; delta not measured.\n"
        ));
    }

    output
}

/// Format as SARIF
///
/// A degradation is a claim about two measurements. With no baseline coverage
/// on disk there is no delta to claim, so no results are emitted rather than
/// warnings derived from an invented "before".
fn format_sarif(result: &IncrementalCoverageResult) -> String {
    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-incremental-coverage",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": result.changed_files.iter().filter_map(|file| {
                let delta = file.coverage_delta?;
                if delta >= 0.0 {
                    return None;
                }
                Some(serde_json::json!({
                    "ruleId": "coverage-degradation",
                    "level": "warning",
                    "message": {
                        "text": format!(
                            "Coverage degraded by {:.1}% (from {} to {})",
                            delta.abs(),
                            pct_or_unmeasured(file.coverage_before),
                            pct_or_unmeasured(file.coverage_after)
                        )
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file.file_path.clone()
                            }
                        }
                    }]
                }))
            }).collect::<Vec<_>>()
        }]
    })
    .to_string()
}
