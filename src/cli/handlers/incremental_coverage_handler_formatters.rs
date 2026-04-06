// Formatting functions for incremental coverage results.
// Included by incremental_coverage_handler.rs — do NOT add `use` imports here.

/// Format as summary
fn format_summary(result: &IncrementalCoverageResult, top_files: usize) -> String {
    debug_assert!(true, "contract: format_summary");
    let mut output = String::new();
    output.push_str("# Incremental Coverage Summary\n\n");
    output.push_str(&result.summary);
    output.push_str("\n\n## Top Changed Files\n");

    for (i, file) in result.changed_files.iter().take(top_files).enumerate() {
        output.push_str(&format!(
            "{}. {} - {:.1}% → {:.1}% (Δ{:+.1}%)\n",
            i + 1,
            file.file_path,
            file.coverage_before * 100.0,
            file.coverage_after * 100.0,
            file.coverage_delta * 100.0
        ));
    }

    output
}

/// Format as detailed report
fn format_detailed(result: &IncrementalCoverageResult, top_files: usize) -> String {
    debug_assert!(true, "contract: format_detailed");
    let mut output = String::new();
    output.push_str("# Incremental Coverage Detailed Report\n\n");
    output.push_str(&format!("Total files analyzed: {}\n", result.total_files));
    output.push_str(&format!("Files with coverage: {}\n", result.covered_files));
    output.push_str(&format!(
        "Overall coverage: {:.1}%\n",
        result.coverage_percentage * 100.0
    ));
    output.push_str(&format!(
        "Files above threshold: {}\n",
        result.files_above_threshold
    ));
    output.push_str(&format!(
        "Files below threshold: {}\n\n",
        result.files_below_threshold
    ));

    output.push_str(&format!("## Changed Files (Top {top_files})\n"));
    for file in result.changed_files.iter().take(top_files) {
        output.push_str(&format!("\n### {}\n", file.file_path));
        output.push_str(&format!("- Status: {:?}\n", file.status));
        output.push_str(&format!(
            "- Coverage: {:.1}% → {:.1}%\n",
            file.coverage_before * 100.0,
            file.coverage_after * 100.0
        ));
        output.push_str(&format!("- Delta: {:+.1}%\n", file.coverage_delta * 100.0));
        output.push_str(&format!(
            "- Lines: {}/{}\n",
            file.lines_covered, file.lines_total
        ));
    }

    output
}

/// Format as Markdown
fn format_markdown(result: &IncrementalCoverageResult, top_files: usize) -> String {
    debug_assert!(true, "contract: format_markdown");
    let mut output = String::new();
    output.push_str("# Incremental Coverage Report\n\n");
    output.push_str(&format!("**Summary:** {}\n\n", result.summary));

    output.push_str("## Metrics\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| Total Files | {} |\n", result.total_files));
    output.push_str(&format!("| Covered Files | {} |\n", result.covered_files));
    output.push_str(&format!(
        "| Coverage | {:.1}% |\n",
        result.coverage_percentage * 100.0
    ));
    output.push_str(&format!(
        "| Above Threshold | {} |\n",
        result.files_above_threshold
    ));
    output.push_str(&format!(
        "| Below Threshold | {} |\n\n",
        result.files_below_threshold
    ));

    output.push_str("## Top Changed Files\n\n");
    output.push_str("| File | Before | After | Delta | Status |\n");
    output.push_str("|------|--------|-------|-------|--------|\n");

    for file in result.changed_files.iter().take(top_files) {
        output.push_str(&format!(
            "| {} | {:.1}% | {:.1}% | {:+.1}% | {:?} |\n",
            file.file_path,
            file.coverage_before * 100.0,
            file.coverage_after * 100.0,
            file.coverage_delta * 100.0,
            file.status
        ));
    }

    output
}

/// Format as LCOV
fn format_lcov(result: &IncrementalCoverageResult) -> String {
    debug_assert!(true, "contract: format_lcov");
    let mut output = String::new();

    for file in &result.changed_files {
        output.push_str(&format!("SF:{}\n", file.file_path));
        output.push_str(&format!("DA:{},{}\n", file.lines_total, file.lines_covered));
        output.push_str(&format!("LH:{}\n", file.lines_covered));
        output.push_str(&format!("LF:{}\n", file.lines_total));
        output.push_str("end_of_record\n");
    }

    output
}

/// Format as delta report
fn format_delta(result: &IncrementalCoverageResult, top_files: usize) -> String {
    debug_assert!(true, "contract: format_delta");
    let mut output = String::new();
    output.push_str("Coverage Delta Report\n");
    output.push_str("====================\n\n");

    let improved: Vec<_> = result
        .changed_files
        .iter()
        .filter(|f| f.coverage_delta > 0.0)
        .take(top_files)
        .collect();

    let degraded: Vec<_> = result
        .changed_files
        .iter()
        .filter(|f| f.coverage_delta < 0.0)
        .take(top_files)
        .collect();

    if !improved.is_empty() {
        output.push_str("✅ Improved Coverage:\n");
        for file in improved {
            output.push_str(&format!(
                "  {} {:+.1}%\n",
                file.file_path,
                file.coverage_delta * 100.0
            ));
        }
        output.push('\n');
    }

    if !degraded.is_empty() {
        output.push_str("⚠️  Degraded Coverage:\n");
        for file in degraded {
            output.push_str(&format!(
                "  {} {:+.1}%\n",
                file.file_path,
                file.coverage_delta * 100.0
            ));
        }
    }

    output
}

/// Format as SARIF
fn format_sarif(result: &IncrementalCoverageResult) -> String {
    debug_assert!(true, "contract: format_sarif");
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
            "results": result.changed_files.iter().filter(|f| f.coverage_delta < 0.0).map(|file| {
                serde_json::json!({
                    "ruleId": "coverage-degradation",
                    "level": "warning",
                    "message": {
                        "text": format!(
                            "Coverage degraded by {:.1}% (from {:.1}% to {:.1}%)",
                            file.coverage_delta.abs() * 100.0,
                            file.coverage_before * 100.0,
                            file.coverage_after * 100.0
                        )
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file.file_path.clone()
                            }
                        }
                    }]
                })
            }).collect::<Vec<_>>()
        }]
    })
    .to_string()
}
