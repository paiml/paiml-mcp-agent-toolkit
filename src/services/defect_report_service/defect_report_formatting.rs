// Report formatting: JSON, CSV, Markdown, Text, and filename generation

impl DefectReportService {
    /// Format report as JSON
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_json(&self, report: &DefectReport) -> Result<String> {
        serde_json::to_string_pretty(report).map_err(Into::into)
    }

    /// Format report as CSV
    ///
    /// Issue #672: this was `#[cfg(feature = "reporting")]`, and `reporting`
    /// was not in `default`, so on a `cargo install pmat` binary
    /// `pmat report --format csv` exited rc=1 with
    /// "CSV reporting requires the 'reporting' feature" while `--help` still
    /// advertised csv. CSV is now always compiled in.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_csv(&self, report: &DefectReport) -> Result<String> {
        let mut wtr = Writer::from_writer(vec![]);

        // Write headers
        wtr.write_record([
            "id",
            "severity",
            "category",
            "file_path",
            "line_start",
            "line_end",
            "message",
            "rule_id",
            "cyclomatic",
            "cognitive",
        ])?;

        // Write defects
        for defect in &report.defects {
            wtr.write_record([
                &defect.id,
                &format!("{:?}", defect.severity).to_lowercase(),
                &format!("{:?}", defect.category),
                &defect.file_path.display().to_string(),
                &defect.line_start.to_string(),
                &defect.line_end.map(|l| l.to_string()).unwrap_or_default(),
                &defect.message,
                &defect.rule_id,
                &defect
                    .metrics
                    .get("cyclomatic")
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
                &defect
                    .metrics
                    .get("cognitive")
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
            ])?;
        }

        let data = wtr.into_inner()?;
        Ok(String::from_utf8(data)?)
    }

    /// Format report as Markdown
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_markdown(&self, report: &DefectReport) -> Result<String> {
        let mut md = String::with_capacity(100_000);

        // Header
        md.push_str("# Code Quality Report\n\n");
        md.push_str(&format!(
            "Generated: {}\n\n",
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Executive Summary
        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!(
            "- **Total Defects**: {}\n",
            report.summary.total_defects
        ));
        md.push_str(&format!(
            "- **Files Analyzed**: {}\n",
            report.metadata.total_files_analyzed
        ));
        md.push_str(&format!(
            "- **Analysis Duration**: {}ms\n\n",
            report.metadata.analysis_duration_ms
        ));

        // Severity Distribution
        md.push_str("### Severity Distribution\n\n");
        md.push_str("```\n");

        let total = report.summary.total_defects as f64;
        for (severity, count) in &report.summary.by_severity {
            let percentage = (*count as f64 / total) * 100.0;
            let bar_length = (percentage / 5.0) as usize;
            let progress_bar = "\u{2588}".repeat(bar_length);
            let empty = "\u{2591}".repeat(20 - bar_length);
            md.push_str(&format!(
                "{severity:<8} {progress_bar}{empty} {count} ({percentage:.1}%)\n"
            ));
        }
        md.push_str("```\n\n");

        // Top Hotspot Files
        if !report.summary.hotspot_files.is_empty() {
            md.push_str("### Top 10 Hotspot Files\n\n");
            md.push_str("| Rank | File | Defects | Severity Score |\n");
            md.push_str("|------|------|---------|----------------|\n");

            for (i, hotspot) in report.summary.hotspot_files.iter().enumerate() {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.1} |\n",
                    i + 1,
                    hotspot.path.display(),
                    hotspot.defect_count,
                    hotspot.severity_score
                ));
            }
            md.push('\n');
        }

        // Detailed Findings by Category
        md.push_str("## Detailed Findings\n\n");

        for category in DefectCategory::all() {
            let category_defects: Vec<_> = report
                .defects
                .iter()
                .filter(|d| d.category == category)
                .collect();

            if !category_defects.is_empty() {
                md.push_str(&format!(
                    "### {} ({} issues)\n\n",
                    category,
                    category_defects.len()
                ));

                for defect in category_defects.iter().take(10) {
                    md.push_str(&format!(
                        "#### {}:{}-{}\n\n",
                        defect.file_path.display(),
                        defect.line_start,
                        defect.line_end.unwrap_or(defect.line_start)
                    ));
                    md.push_str(&format!("**{}** - {}\n\n", defect.severity, defect.message));

                    if let Some(fix) = &defect.fix_suggestion {
                        md.push_str(&format!("> \u{1f4a1} **Suggestion**: {fix}\n\n"));
                    }
                }

                if category_defects.len() > 10 {
                    md.push_str(&format!(
                        "_...and {} more {}_\n\n",
                        category_defects.len() - 10,
                        category
                    ));
                }
            }
        }

        Ok(md)
    }

    /// Format report as plain text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_text(&self, report: &DefectReport) -> Result<String> {
        let mut txt = String::with_capacity(50_000);

        // Header
        txt.push_str("CODE QUALITY REPORT\n");
        txt.push_str("===================\n\n");
        txt.push_str(&format!(
            "Generated: {}\n",
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        txt.push_str(&format!(
            "Project: {}\n",
            report.metadata.project_root.display()
        ));
        txt.push_str(&format!(
            "Total Defects: {}\n",
            report.summary.total_defects
        ));
        txt.push_str(&format!(
            "Files Analyzed: {}\n\n",
            report.metadata.total_files_analyzed
        ));

        // Summary by severity
        txt.push_str("SEVERITY BREAKDOWN\n");
        txt.push_str("------------------\n");
        for (severity, count) in &report.summary.by_severity {
            txt.push_str(&format!("{severity:<10} {count}\n"));
        }
        txt.push('\n');

        // Summary by category
        txt.push_str("CATEGORY BREAKDOWN\n");
        txt.push_str("------------------\n");
        for (category, count) in &report.summary.by_category {
            txt.push_str(&format!("{category:<20} {count}\n"));
        }
        txt.push('\n');

        // Top hotspot files
        if !report.summary.hotspot_files.is_empty() {
            txt.push_str("TOP HOTSPOT FILES\n");
            txt.push_str("-----------------\n");
            for (i, hotspot) in report.summary.hotspot_files.iter().enumerate() {
                txt.push_str(&format!(
                    "{}. {} ({} defects, score: {:.1})\n",
                    i + 1,
                    hotspot.path.display(),
                    hotspot.defect_count,
                    hotspot.severity_score
                ));
            }
            txt.push('\n');
        }

        // List defects
        txt.push_str("DEFECTS\n");
        txt.push_str("-------\n");
        for defect in &report.defects {
            txt.push_str(&format!(
                "[{}] {} - {}:{}",
                defect.severity,
                defect.category,
                defect.file_path.display(),
                defect.line_start
            ));
            if let Some(end) = defect.line_end {
                txt.push_str(&format!("-{end}"));
            }
            txt.push_str(&format!("\n  {}\n", defect.message));
            if let Some(fix) = &defect.fix_suggestion {
                txt.push_str(&format!("  Fix: {fix}\n"));
            }
            txt.push('\n');
        }

        Ok(txt)
    }

    /// Generate filename with timestamp
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_filename(&self, format: ReportFormat) -> String {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        match format {
            ReportFormat::Json => format!("defect-report-{timestamp}.json"),
            ReportFormat::Csv => format!("defect-report-{timestamp}.csv"),
            ReportFormat::Markdown => format!("defect-report-{timestamp}.md"),
            ReportFormat::Text => format!("defect-report-{timestamp}.txt"),
        }
    }
}
