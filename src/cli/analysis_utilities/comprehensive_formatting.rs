fn format_comprehensive_report(
    report: &ComprehensiveReport,
    format: ComprehensiveOutputFormat,
    executive_summary: bool,
) -> Result<String> {
    match format {
        ComprehensiveOutputFormat::Json => format_comp_as_json(report),
        ComprehensiveOutputFormat::Markdown => format_comp_as_markdown(report, executive_summary),
        _ => Ok("Comprehensive analysis completed.".to_string()),
    }
}

// Helper: Format comprehensive report as JSON
fn format_comp_as_json(report: &ComprehensiveReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

// Helper: Format comprehensive report as Markdown
fn format_comp_as_markdown(
    report: &ComprehensiveReport,
    executive_summary: bool,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Comprehensive Code Analysis Report\n")?;

    if executive_summary {
        write_comp_executive_summary(&mut output)?;
    }

    write_comp_analysis_sections(&mut output, report)?;

    Ok(output)
}

// Helper: Write executive summary
fn write_comp_executive_summary(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Executive Summary\n")?;
    writeln!(
        output,
        "This report provides a comprehensive analysis of code quality metrics.\n"
    )?;
    Ok(())
}

// Helper: Write all analysis sections
fn write_comp_analysis_sections(output: &mut String, report: &ComprehensiveReport) -> Result<()> {
    if let Some(complexity) = &report.complexity {
        write_comp_complexity_section(output, complexity)?;
    }

    if let Some(satd) = &report.satd {
        write_comp_satd_section(output, satd)?;
    }

    if let Some(tdg) = &report.tdg {
        write_comp_tdg_section(output, tdg)?;
    }

    if let Some(dead_code) = &report.dead_code {
        write_comp_dead_code_section(output, dead_code)?;
    }

    if let Some(defects) = &report.defects {
        write_comp_defects_section(output, defects)?;
    }

    if let Some(duplicates) = &report.duplicates {
        write_comp_duplicates_section(output, duplicates)?;
    }

    Ok(())
}

// Helper: Write complexity section
fn write_comp_complexity_section(output: &mut String, complexity: &ComplexityReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Complexity Analysis\n")?;
    writeln!(output, "- Total functions: {}", complexity.total_functions)?;
    writeln!(
        output,
        "- High complexity functions: {}",
        complexity.high_complexity_count
    )?;
    writeln!(
        output,
        "- Average complexity: {:.2}",
        complexity.average_complexity
    )?;
    writeln!(output, "- P99 complexity: {}\n", complexity.p99_complexity)?;
    Ok(())
}

// Helper: Write SATD section
fn write_comp_satd_section(output: &mut String, satd: &SatdReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Technical Debt (SATD)\n")?;
    writeln!(output, "- Total items: {}", satd.total_items)?;
    writeln!(output, "- By type:")?;
    for (t, count) in &satd.by_type {
        writeln!(output, "  - {t}: {count}")?;
    }
    writeln!(output)?;
    Ok(())
}

// Helper: Write TDG section
fn write_comp_tdg_section(output: &mut String, tdg: &TdgReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Technical Debt Gradient\n")?;
    writeln!(output, "- Average TDG: {:.2}", tdg.average_tdg)?;
    writeln!(output, "- Critical files: {}", tdg.critical_files.len())?;
    writeln!(output, "- Hotspot count: {}\n", tdg.hotspot_count)?;
    Ok(())
}

// Helper: Write dead code section
fn write_comp_dead_code_section(output: &mut String, dead_code: &DeadCodeReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Dead Code\n")?;
    writeln!(output, "- Total items: {}", dead_code.total_items)?;
    writeln!(
        output,
        "- Percentage: {:.1}%\n",
        dead_code.dead_code_percentage
    )?;
    Ok(())
}

// Helper: Write defects section
fn write_comp_defects_section(output: &mut String, defects: &DefectReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Defect Prediction\n")?;
    writeln!(output, "- Total analyzed: {}", defects.total_analyzed)?;
    writeln!(output, "- High risk files: {}\n", defects.high_risk_count)?;
    Ok(())
}

// Helper: Write duplicates section
fn write_comp_duplicates_section(output: &mut String, duplicates: &DuplicateReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Code Duplication\n")?;
    writeln!(
        output,
        "- Duplicate blocks: {}",
        duplicates.duplicate_blocks
    )?;
    writeln!(output, "- Duplicate lines: {}", duplicates.duplicate_lines)?;
    writeln!(
        output,
        "- Percentage: {:.1}%\n",
        duplicates.duplicate_percentage
    )?;
    Ok(())
}

#[cfg(test)]
mod comprehensive_formatting_tests {
    //! PMAT-633 cohort: cover comprehensive_formatting.rs (0% on broad,
    //! 106 missed lines). The Report* structs are module-private but
    //! constructible inside this include!()'d scope.
    use super::*;

    fn complexity_report() -> ComplexityReport {
        ComplexityReport {
            total_functions: 42,
            high_complexity_count: 3,
            average_complexity: 4.5,
            p99_complexity: 25,
            hotspots: vec![ComplexityHotspot {
                function: "foo".into(),
                file: "src/a.rs".into(),
                complexity: 30,
            }],
        }
    }

    fn satd_report() -> SatdReport {
        let mut by_type = HashMap::new();
        by_type.insert("TODO".to_string(), 5);
        let mut by_sev = HashMap::new();
        by_sev.insert("Low".to_string(), 5);
        SatdReport {
            total_items: 5,
            by_type,
            by_severity: by_sev,
            items: vec![SatdItem {
                file: "src/a.rs".into(),
                line: 10,
                text: "TODO fix".into(),
                satd_type: "TODO".into(),
                severity: "Low".into(),
            }],
        }
    }

    fn tdg_report() -> TdgReport {
        TdgReport {
            average_tdg: 2.5,
            critical_files: vec![TdgFile {
                file: "src/c.rs".into(),
                tdg_score: 3.0,
                complexity: 20,
                churn: 8,
            }],
            hotspot_count: 1,
        }
    }

    fn dead_code_report() -> DeadCodeReport {
        DeadCodeReport {
            total_items: 7,
            dead_code_percentage: 1.5,
            items: vec![DeadCodeItem {
                name: "unused".into(),
                file: "src/d.rs".into(),
                line: 99,
                item_type: "function".into(),
            }],
        }
    }

    fn defect_report() -> DefectReport {
        DefectReport {
            high_risk_files: vec![DefectPrediction {
                file: "src/e.rs".into(),
                probability: 0.85,
                factors: vec!["high churn".into()],
            }],
            total_analyzed: 100,
            high_risk_count: 1,
        }
    }

    fn duplicate_report() -> DuplicateReport {
        DuplicateReport {
            duplicate_blocks: 3,
            duplicate_lines: 150,
            duplicate_percentage: 4.2,
            blocks: vec![DuplicateBlock {
                files: vec!["src/a.rs".into(), "src/b.rs".into()],
                lines: 50,
                tokens: 200,
            }],
        }
    }

    fn full_report() -> ComprehensiveReport {
        ComprehensiveReport {
            complexity: Some(complexity_report()),
            satd: Some(satd_report()),
            tdg: Some(tdg_report()),
            dead_code: Some(dead_code_report()),
            defects: Some(defect_report()),
            duplicates: Some(duplicate_report()),
        }
    }

    // ── format_comp_as_json ──

    #[test]
    fn test_format_comp_as_json_empty_report_is_valid_json() {
        let report = ComprehensiveReport::default();
        let out = format_comp_as_json(&report).unwrap();
        // Should parse back as JSON.
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_format_comp_as_json_populated_contains_all_sections() {
        let report = full_report();
        let out = format_comp_as_json(&report).unwrap();
        for section in ["complexity", "satd", "tdg", "dead_code", "defects", "duplicates"] {
            assert!(out.contains(section), "missing {section} in JSON: {out}");
        }
    }

    // ── format_comp_as_markdown ──

    #[test]
    fn test_format_comp_as_markdown_no_exec_summary() {
        let out = format_comp_as_markdown(&full_report(), false).unwrap();
        assert!(out.contains("# Comprehensive Code Analysis Report"));
        assert!(!out.contains("Executive Summary"));
    }

    #[test]
    fn test_format_comp_as_markdown_with_exec_summary() {
        let out = format_comp_as_markdown(&full_report(), true).unwrap();
        assert!(out.contains("# Comprehensive Code Analysis Report"));
        assert!(out.contains("Executive Summary"));
    }

    #[test]
    fn test_format_comp_as_markdown_none_sections_are_skipped() {
        // Empty report → headers for individual analyses absent.
        let report = ComprehensiveReport::default();
        let out = format_comp_as_markdown(&report, false).unwrap();
        assert!(!out.contains("## Complexity Analysis"));
        assert!(!out.contains("## Technical Debt (SATD)"));
    }

    #[test]
    fn test_format_comp_as_markdown_populated_contains_all_section_headers() {
        let out = format_comp_as_markdown(&full_report(), false).unwrap();
        for header in [
            "## Complexity Analysis",
            "## Technical Debt (SATD)",
            "## Technical Debt Gradient",
            "## Dead Code",
            "## Defect Prediction",
            "## Code Duplication",
        ] {
            assert!(out.contains(header), "missing header `{header}`");
        }
    }

    // ── format_comprehensive_report top-level dispatcher ──

    #[test]
    fn test_format_comprehensive_report_json_dispatch() {
        let out = format_comprehensive_report(
            &full_report(),
            ComprehensiveOutputFormat::Json,
            false,
        )
        .unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_format_comprehensive_report_markdown_dispatch() {
        let out = format_comprehensive_report(
            &full_report(),
            ComprehensiveOutputFormat::Markdown,
            false,
        )
        .unwrap();
        assert!(out.contains("# Comprehensive Code Analysis Report"));
    }

    #[test]
    fn test_format_comprehensive_report_unrecognized_format_fallback() {
        // Summary / Detailed / Sarif all fall into the catch-all string arm.
        let out = format_comprehensive_report(
            &full_report(),
            ComprehensiveOutputFormat::Summary,
            false,
        )
        .unwrap();
        assert_eq!(out, "Comprehensive analysis completed.");

        let out = format_comprehensive_report(
            &full_report(),
            ComprehensiveOutputFormat::Sarif,
            false,
        )
        .unwrap();
        assert_eq!(out, "Comprehensive analysis completed.");
    }
}

// Incremental coverage stub data structures
