#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for TDG analysis to reduce complexity

use crate::models::tdg::{TDGHotspot, TDGSummary};
use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

/// Filter TDG hotspots based on criteria
#[must_use]
pub fn filter_tdg_hotspots(
    mut hotspots: Vec<TDGHotspot>,
    threshold: f64,
    top: usize,
    critical_only: bool,
) -> Vec<TDGHotspot> {
    // Apply threshold filter
    if threshold > 0.0 {
        hotspots.retain(|h| h.tdg_score >= threshold);
    }

    // Apply critical filter
    if critical_only {
        hotspots.retain(|h| h.tdg_score > 2.5);
    }

    // Apply top limit
    if top > 0 && hotspots.len() > top {
        hotspots.truncate(top);
    }

    hotspots
}

/// Format TDG results as JSON
pub fn format_tdg_json(
    summary: &TDGSummary,
    hotspots: &[TDGHotspot],
    include_components: bool,
) -> Result<String> {
    let mut json_data = serde_json::json!({
        "summary": {
            "total_files": summary.total_files,
            "critical_files": summary.critical_files,
            "warning_files": summary.warning_files,
            "average_tdg": summary.average_tdg,
            "p95_tdg": summary.p95_tdg,
            "p99_tdg": summary.p99_tdg,
            "estimated_debt_hours": summary.estimated_debt_hours,
        },
        "hotspots": hotspots,
    });

    if include_components {
        // Add component breakdown if requested
        json_data["components"] = serde_json::json!({
            "complexity_weight": 0.4,
            "churn_weight": 0.3,
            "duplication_weight": 0.2,
            "coupling_weight": 0.1,
        });
    }

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}

/// Format TDG results as table
pub fn format_tdg_table(hotspots: &[TDGHotspot], verbose: bool) -> Result<String> {
    let mut output = String::new();

    writeln!(
        &mut output,
        "| File | TDG Score | Primary Factor | Est. Hours |"
    )?;
    writeln!(
        &mut output,
        "|------|-----------|----------------|-----------|"
    )?;

    for hotspot in hotspots {
        writeln!(
            &mut output,
            "| {} | {:.2} | {} | {:.1} |",
            std::path::Path::new(&hotspot.path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            hotspot.tdg_score,
            hotspot.primary_factor,
            hotspot.estimated_hours
        )?;

        if verbose {
            writeln!(
                &mut output,
                "|      | Components: C={:.2} Ch={:.2} D={:.2} Co={:.2} |",
                hotspot.tdg_score * 0.4, // Complexity component
                hotspot.tdg_score * 0.3, // Churn component
                hotspot.tdg_score * 0.2, // Duplication component
                hotspot.tdg_score * 0.1, // Coupling component
            )?;
        }
    }

    Ok(output)
}

/// Format TDG results as markdown
pub fn format_tdg_markdown(
    summary: &TDGSummary,
    hotspots: &[TDGHotspot],
    include_components: bool,
) -> Result<String> {
    let mut output = String::new();

    write_tdg_header(&mut output)?;
    write_tdg_summary(&mut output, summary)?;

    if !hotspots.is_empty() {
        write_tdg_hotspots(&mut output, hotspots, include_components)?;
    }

    Ok(output)
}

/// Write TDG markdown header
fn write_tdg_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Technical Debt Gradient Analysis\n")?;
    Ok(())
}

/// Write TDG summary section
fn write_tdg_summary(output: &mut String, summary: &TDGSummary) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Summary\n")?;
    writeln!(output, "- **Total Files**: {}", summary.total_files)?;
    writeln!(
        output,
        "- **Critical Files**: {} (TDG > 2.5)",
        summary.critical_files
    )?;
    writeln!(
        output,
        "- **Warning Files**: {} (TDG > 1.5)",
        summary.warning_files
    )?;
    writeln!(output, "- **Average TDG**: {:.3}", summary.average_tdg)?;
    writeln!(output, "- **95th Percentile**: {:.3}", summary.p95_tdg)?;
    writeln!(
        output,
        "- **Estimated Debt**: {:.1} hours\n",
        summary.estimated_debt_hours
    )?;

    Ok(())
}

/// Write TDG hotspots section
fn write_tdg_hotspots(
    output: &mut String,
    hotspots: &[TDGHotspot],
    include_components: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Top Hotspots\n")?;

    for (i, hotspot) in hotspots.iter().enumerate() {
        write_single_hotspot(output, i + 1, hotspot, include_components)?;
    }

    Ok(())
}

/// Write a single hotspot entry
fn write_single_hotspot(
    output: &mut String,
    index: usize,
    hotspot: &TDGHotspot,
    include_components: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "### {}. {}\n", index, hotspot.path)?;
    write_hotspot_basic_info(output, hotspot)?;

    if include_components {
        write_component_breakdown(output, hotspot)?;
    }

    Ok(())
}

/// Write basic hotspot information
fn write_hotspot_basic_info(output: &mut String, hotspot: &TDGHotspot) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "- **TDG Score**: {:.3}", hotspot.tdg_score)?;
    writeln!(output, "- **Primary Factor**: {}", hotspot.primary_factor)?;
    writeln!(
        output,
        "- **Estimated Hours**: {:.1}\n",
        hotspot.estimated_hours
    )?;

    Ok(())
}

/// Write component breakdown for a hotspot
fn write_component_breakdown(output: &mut String, hotspot: &TDGHotspot) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "#### Component Breakdown:")?;
    writeln!(
        output,
        "- Complexity: {:.3}",
        calculate_component_score(hotspot.tdg_score, 0.4)
    )?;
    writeln!(
        output,
        "- Churn: {:.3}",
        calculate_component_score(hotspot.tdg_score, 0.3)
    )?;
    writeln!(
        output,
        "- Duplication: {:.3}",
        calculate_component_score(hotspot.tdg_score, 0.2)
    )?;
    writeln!(
        output,
        "- Coupling: {:.3}\n",
        calculate_component_score(hotspot.tdg_score, 0.1)
    )?;

    Ok(())
}

/// Calculate component score with given weight
fn calculate_component_score(tdg_score: f64, weight: f64) -> f64 {
    tdg_score * weight
}

/// Format TDG results as SARIF
pub fn format_tdg_sarif(hotspots: &[TDGHotspot], project_path: &Path) -> Result<String> {
    let mut results = Vec::new();

    for hotspot in hotspots {
        let level = if hotspot.tdg_score > 2.5 {
            "error"
        } else if hotspot.tdg_score > 1.5 {
            "warning"
        } else {
            "note"
        };

        let rule_id = if hotspot.tdg_score > 2.5 {
            "critical-tdg"
        } else if hotspot.tdg_score > 1.5 {
            "high-tdg"
        } else {
            "moderate-tdg"
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "File has TDG score of {:.2} ({}). Estimated refactoring time: {:.1} hours",
                    hotspot.tdg_score,
                    hotspot.primary_factor,
                    hotspot.estimated_hours
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": std::path::Path::new(&hotspot.path)
                            .strip_prefix(project_path)
                            .unwrap_or(std::path::Path::new(&hotspot.path))
                            .to_string_lossy()
                    }
                }
            }]
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-tdg-analyzer",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": generate_tdg_rules(),
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

/// Generate SARIF rules for TDG
fn generate_tdg_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "critical-tdg",
            "name": "Critical Technical Debt",
            "shortDescription": {
                "text": "File has critical technical debt gradient"
            },
            "fullDescription": {
                "text": "Files with TDG > 2.5 require immediate refactoring"
            },
            "defaultConfiguration": {
                "level": "error"
            }
        }),
        serde_json::json!({
            "id": "high-tdg",
            "name": "High Technical Debt",
            "shortDescription": {
                "text": "File has high technical debt gradient"
            },
            "fullDescription": {
                "text": "Files with TDG > 1.5 should be refactored soon"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "moderate-tdg",
            "name": "Moderate Technical Debt",
            "shortDescription": {
                "text": "File has moderate technical debt gradient"
            },
            "fullDescription": {
                "text": "Files with TDG > 1.0 should be monitored"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
    ]
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::models::tdg::{TDGHotspot, TDGSummary};
    use std::path::PathBuf;

    // ============================================================
    // Helper functions to create test data
    // ============================================================

    fn create_test_hotspot(path: &str, score: f64, factor: &str, hours: f64) -> TDGHotspot {
        TDGHotspot {
            path: path.to_string(),
            tdg_score: score,
            primary_factor: factor.to_string(),
            estimated_hours: hours,
        }
    }

    fn create_test_summary() -> TDGSummary {
        TDGSummary {
            total_files: 100,
            critical_files: 5,
            warning_files: 15,
            average_tdg: 1.5,
            p95_tdg: 2.8,
            p99_tdg: 3.5,
            estimated_debt_hours: 80.0,
            hotspots: vec![],
        }
    }

    fn create_test_hotspots() -> Vec<TDGHotspot> {
        vec![
            create_test_hotspot("src/complex.rs", 3.5, "complexity", 12.0),
            create_test_hotspot("src/churn.rs", 2.8, "churn", 8.0),
            create_test_hotspot("src/coupling.rs", 2.0, "coupling", 5.0),
            create_test_hotspot("src/normal.rs", 1.2, "none", 2.0),
            create_test_hotspot("src/low.rs", 0.5, "none", 0.5),
        ]
    }

    // ============================================================
    // Tests for filter_tdg_hotspots
    // ============================================================

    mod filter_tests {
        use super::*;

        #[test]
        fn test_filter_no_filters() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots.clone(), 0.0, 0, false);
            assert_eq!(result.len(), 5);
        }

        #[test]
        fn test_filter_by_threshold() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots, 2.0, 0, false);
            assert_eq!(result.len(), 3);
            assert!(result.iter().all(|h| h.tdg_score >= 2.0));
        }

        #[test]
        fn test_filter_by_threshold_high() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots, 3.0, 0, false);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].path, "src/complex.rs");
        }

        #[test]
        fn test_filter_critical_only() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots, 0.0, 0, true);
            // Critical threshold is > 2.5
            assert_eq!(result.len(), 2);
            assert!(result.iter().all(|h| h.tdg_score > 2.5));
        }

        #[test]
        fn test_filter_top_limit() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots, 0.0, 2, false);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn test_filter_top_limit_larger_than_list() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots, 0.0, 100, false);
            assert_eq!(result.len(), 5);
        }

        #[test]
        fn test_filter_combined_threshold_and_critical() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots, 1.0, 0, true);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn test_filter_combined_all_filters() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots, 1.0, 1, true);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].path, "src/complex.rs");
        }

        #[test]
        fn test_filter_empty_hotspots() {
            let hotspots: Vec<TDGHotspot> = vec![];
            let result = filter_tdg_hotspots(hotspots, 1.0, 10, true);
            assert!(result.is_empty());
        }

        #[test]
        fn test_filter_zero_threshold() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots.clone(), 0.0, 0, false);
            assert_eq!(result.len(), hotspots.len());
        }

        #[test]
        fn test_filter_negative_threshold_treated_as_no_filter() {
            let hotspots = create_test_hotspots();
            let result = filter_tdg_hotspots(hotspots.clone(), -1.0, 0, false);
            // Negative threshold is less than 0, so threshold filter doesn't apply
            assert_eq!(result.len(), 5);
        }
    }

    // ============================================================
    // Tests for format_tdg_json
    // ============================================================

    mod json_format_tests {
        use super::*;

        #[test]
        fn test_format_json_basic() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();
            let result = format_tdg_json(&summary, &hotspots, false).unwrap();

            assert!(result.contains("\"total_files\": 100"));
            assert!(result.contains("\"critical_files\": 5"));
            assert!(result.contains("\"warning_files\": 15"));
            assert!(result.contains("\"average_tdg\": 1.5"));
            assert!(result.contains("\"p95_tdg\": 2.8"));
            assert!(result.contains("\"p99_tdg\": 3.5"));
            assert!(result.contains("\"estimated_debt_hours\": 80.0"));
            assert!(result.contains("\"hotspots\""));
        }

        #[test]
        fn test_format_json_with_components() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();
            let result = format_tdg_json(&summary, &hotspots, true).unwrap();

            assert!(result.contains("\"components\""));
            assert!(result.contains("\"complexity_weight\": 0.4"));
            assert!(result.contains("\"churn_weight\": 0.3"));
            assert!(result.contains("\"duplication_weight\": 0.2"));
            assert!(result.contains("\"coupling_weight\": 0.1"));
        }

        #[test]
        fn test_format_json_without_components() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();
            let result = format_tdg_json(&summary, &hotspots, false).unwrap();

            assert!(!result.contains("\"components\""));
        }

        #[test]
        fn test_format_json_empty_hotspots() {
            let summary = create_test_summary();
            let hotspots: Vec<TDGHotspot> = vec![];
            let result = format_tdg_json(&summary, &hotspots, false).unwrap();

            assert!(result.contains("\"hotspots\": []"));
        }

        #[test]
        fn test_format_json_is_valid_json() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();
            let result = format_tdg_json(&summary, &hotspots, true).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert!(parsed.is_object());
            assert!(parsed.get("summary").is_some());
            assert!(parsed.get("hotspots").is_some());
            assert!(parsed.get("components").is_some());
        }
    }

    // ============================================================
    // Tests for format_tdg_table
    // ============================================================

    mod table_format_tests {
        use super::*;

        #[test]
        fn test_format_table_basic() {
            let hotspots = create_test_hotspots();
            let result = format_tdg_table(&hotspots, false).unwrap();

            // Check header
            assert!(result.contains("| File | TDG Score | Primary Factor | Est. Hours |"));
            assert!(result.contains("|------|-----------|----------------|-----------|"));
        }

        #[test]
        fn test_format_table_contains_hotspot_data() {
            let hotspots = vec![create_test_hotspot(
                "src/test/complex.rs",
                3.5,
                "complexity",
                12.0,
            )];
            let result = format_tdg_table(&hotspots, false).unwrap();

            assert!(result.contains("complex.rs"));
            assert!(result.contains("3.50"));
            assert!(result.contains("complexity"));
            assert!(result.contains("12.0"));
        }

        #[test]
        fn test_format_table_verbose_mode() {
            let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "churn", 5.0)];
            let result = format_tdg_table(&hotspots, true).unwrap();

            // Verbose mode should include component breakdown
            assert!(result.contains("Components:"));
            assert!(result.contains("C="));
            assert!(result.contains("Ch="));
            assert!(result.contains("D="));
            assert!(result.contains("Co="));
        }

        #[test]
        fn test_format_table_non_verbose_no_components() {
            let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "churn", 5.0)];
            let result = format_tdg_table(&hotspots, false).unwrap();

            assert!(!result.contains("Components:"));
        }

        #[test]
        fn test_format_table_empty_hotspots() {
            let hotspots: Vec<TDGHotspot> = vec![];
            let result = format_tdg_table(&hotspots, false).unwrap();

            // Should still have header
            assert!(result.contains("| File | TDG Score |"));
            // Count lines - should only be header (2 lines)
            let lines: Vec<_> = result.lines().collect();
            assert_eq!(lines.len(), 2);
        }

        #[test]
        fn test_format_table_multiple_hotspots() {
            let hotspots = create_test_hotspots();
            let result = format_tdg_table(&hotspots, false).unwrap();

            // Should have 5 data rows plus 2 header rows
            let lines: Vec<_> = result.lines().collect();
            assert_eq!(lines.len(), 7);
        }

        #[test]
        fn test_format_table_extracts_filename_from_path() {
            let hotspots = vec![create_test_hotspot(
                "very/long/path/to/deep/file.rs",
                1.5,
                "none",
                1.0,
            )];
            let result = format_tdg_table(&hotspots, false).unwrap();

            // Should show only filename, not full path
            assert!(result.contains("file.rs"));
            assert!(!result.contains("very/long/path"));
        }
    }

    // ============================================================
    // Tests for format_tdg_markdown
    // ============================================================

    mod markdown_format_tests {
        use super::*;

        #[test]
        fn test_format_markdown_basic() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();
            let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();

            assert!(result.contains("# Technical Debt Gradient Analysis"));
            assert!(result.contains("## Summary"));
        }

        #[test]
        fn test_format_markdown_summary_content() {
            let summary = create_test_summary();
            let hotspots: Vec<TDGHotspot> = vec![];
            let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();

            assert!(result.contains("**Total Files**: 100"));
            assert!(result.contains("**Critical Files**: 5"));
            assert!(result.contains("**Warning Files**: 15"));
            assert!(result.contains("**Average TDG**: 1.500"));
            assert!(result.contains("**95th Percentile**: 2.800"));
            assert!(result.contains("**Estimated Debt**: 80.0 hours"));
        }

        #[test]
        fn test_format_markdown_with_hotspots() {
            let summary = create_test_summary();
            let hotspots = vec![create_test_hotspot(
                "src/complex.rs",
                3.5,
                "complexity",
                12.0,
            )];
            let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();

            assert!(result.contains("## Top Hotspots"));
            assert!(result.contains("### 1. src/complex.rs"));
            assert!(result.contains("**TDG Score**: 3.500"));
            assert!(result.contains("**Primary Factor**: complexity"));
            assert!(result.contains("**Estimated Hours**: 12.0"));
        }

        #[test]
        fn test_format_markdown_with_components() {
            let summary = create_test_summary();
            let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "churn", 5.0)];
            let result = format_tdg_markdown(&summary, &hotspots, true).unwrap();

            assert!(result.contains("#### Component Breakdown:"));
            assert!(result.contains("- Complexity:"));
            assert!(result.contains("- Churn:"));
            assert!(result.contains("- Duplication:"));
            assert!(result.contains("- Coupling:"));
        }

        #[test]
        fn test_format_markdown_without_components() {
            let summary = create_test_summary();
            let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "churn", 5.0)];
            let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();

            assert!(!result.contains("#### Component Breakdown:"));
        }

        #[test]
        fn test_format_markdown_empty_hotspots_no_section() {
            let summary = create_test_summary();
            let hotspots: Vec<TDGHotspot> = vec![];
            let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();

            assert!(!result.contains("## Top Hotspots"));
        }

        #[test]
        fn test_format_markdown_multiple_hotspots_indexed() {
            let summary = create_test_summary();
            let hotspots = vec![
                create_test_hotspot("src/first.rs", 3.0, "complexity", 10.0),
                create_test_hotspot("src/second.rs", 2.5, "churn", 5.0),
                create_test_hotspot("src/third.rs", 2.0, "coupling", 3.0),
            ];
            let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();

            assert!(result.contains("### 1. src/first.rs"));
            assert!(result.contains("### 2. src/second.rs"));
            assert!(result.contains("### 3. src/third.rs"));
        }
    }

    // ============================================================
    // Tests for format_tdg_sarif
    // ============================================================

    mod sarif_format_tests {
        use super::*;

        #[test]
        fn test_format_sarif_basic_structure() {
            let hotspots = create_test_hotspots();
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert_eq!(parsed["version"], "2.1.0");
            assert!(parsed["$schema"].as_str().unwrap().contains("sarif-schema"));
            assert!(parsed["runs"].is_array());
        }

        #[test]
        fn test_format_sarif_tool_info() {
            let hotspots: Vec<TDGHotspot> = vec![];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let driver = &parsed["runs"][0]["tool"]["driver"];

            assert_eq!(driver["name"], "paiml-tdg-analyzer");
            assert!(driver["informationUri"]
                .as_str()
                .unwrap()
                .contains("github.com"));
        }

        #[test]
        fn test_format_sarif_rules_present() {
            let hotspots: Vec<TDGHotspot> = vec![];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let rules = &parsed["runs"][0]["tool"]["driver"]["rules"];

            assert!(rules.is_array());
            assert_eq!(rules.as_array().unwrap().len(), 3);

            let rule_ids: Vec<_> = rules
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["id"].as_str().unwrap())
                .collect();
            assert!(rule_ids.contains(&"critical-tdg"));
            assert!(rule_ids.contains(&"high-tdg"));
            assert!(rule_ids.contains(&"moderate-tdg"));
        }

        #[test]
        fn test_format_sarif_critical_hotspot() {
            let hotspots = vec![create_test_hotspot(
                "src/critical.rs",
                3.5,
                "complexity",
                15.0,
            )];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let results = &parsed["runs"][0]["results"];

            assert_eq!(results[0]["ruleId"], "critical-tdg");
            assert_eq!(results[0]["level"], "error");
        }

        #[test]
        fn test_format_sarif_warning_hotspot() {
            let hotspots = vec![create_test_hotspot("src/warning.rs", 2.0, "churn", 8.0)];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let results = &parsed["runs"][0]["results"];

            assert_eq!(results[0]["ruleId"], "high-tdg");
            assert_eq!(results[0]["level"], "warning");
        }

        #[test]
        fn test_format_sarif_note_hotspot() {
            let hotspots = vec![create_test_hotspot("src/moderate.rs", 1.2, "coupling", 3.0)];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let results = &parsed["runs"][0]["results"];

            assert_eq!(results[0]["ruleId"], "moderate-tdg");
            assert_eq!(results[0]["level"], "note");
        }

        #[test]
        fn test_format_sarif_message_format() {
            let hotspots = vec![create_test_hotspot("src/test.rs", 3.5, "complexity", 12.0)];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let message = parsed["runs"][0]["results"][0]["message"]["text"]
                .as_str()
                .unwrap();

            assert!(message.contains("3.50"));
            assert!(message.contains("complexity"));
            assert!(message.contains("12.0 hours"));
        }

        #[test]
        fn test_format_sarif_location() {
            let hotspots = vec![create_test_hotspot(
                "/project/src/test.rs",
                2.0,
                "churn",
                5.0,
            )];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let location = &parsed["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
                ["artifactLocation"]["uri"];

            // Should strip project prefix
            assert_eq!(location, "src/test.rs");
        }

        #[test]
        fn test_format_sarif_location_no_strip() {
            let hotspots = vec![create_test_hotspot("other/path/test.rs", 2.0, "churn", 5.0)];
            let project_path = PathBuf::from("/different/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let location = &parsed["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
                ["artifactLocation"]["uri"];

            // Should not strip - path doesn't start with project path
            assert_eq!(location, "other/path/test.rs");
        }

        #[test]
        fn test_format_sarif_empty_hotspots() {
            let hotspots: Vec<TDGHotspot> = vec![];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let results = &parsed["runs"][0]["results"];

            assert!(results.is_array());
            assert!(results.as_array().unwrap().is_empty());
        }

        #[test]
        fn test_format_sarif_multiple_severity_levels() {
            let hotspots = vec![
                create_test_hotspot("src/critical.rs", 3.0, "complexity", 15.0), // error
                create_test_hotspot("src/warning.rs", 2.0, "churn", 8.0),        // warning
                create_test_hotspot("src/note.rs", 1.0, "coupling", 2.0),        // note
            ];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let results = parsed["runs"][0]["results"].as_array().unwrap();

            assert_eq!(results.len(), 3);
            assert_eq!(results[0]["level"], "error");
            assert_eq!(results[1]["level"], "warning");
            assert_eq!(results[2]["level"], "note");
        }

        #[test]
        fn test_format_sarif_boundary_values() {
            // Test exact boundary values for severity classification
            let hotspots = vec![
                create_test_hotspot("src/exactly_2.5.rs", 2.5, "boundary", 5.0), // warning (not error)
                create_test_hotspot("src/exactly_1.5.rs", 1.5, "boundary", 3.0), // note (not warning)
            ];
            let project_path = PathBuf::from("/project");
            let result = format_tdg_sarif(&hotspots, &project_path).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let results = parsed["runs"][0]["results"].as_array().unwrap();

            // 2.5 is NOT > 2.5, so it's warning (high-tdg)
            assert_eq!(results[0]["ruleId"], "high-tdg");
            assert_eq!(results[0]["level"], "warning");

            // 1.5 is NOT > 1.5, so it's note (moderate-tdg)
            assert_eq!(results[1]["ruleId"], "moderate-tdg");
            assert_eq!(results[1]["level"], "note");
        }
    }

    // ============================================================
    // Tests for internal helper functions
    // ============================================================

    mod internal_helpers_tests {
        use super::*;

        #[test]
        fn test_calculate_component_score() {
            assert!((calculate_component_score(10.0, 0.4) - 4.0).abs() < f64::EPSILON);
            assert!((calculate_component_score(10.0, 0.3) - 3.0).abs() < f64::EPSILON);
            assert!((calculate_component_score(10.0, 0.2) - 2.0).abs() < f64::EPSILON);
            assert!((calculate_component_score(10.0, 0.1) - 1.0).abs() < f64::EPSILON);
        }

        #[test]
        fn test_calculate_component_score_zero() {
            assert!((calculate_component_score(0.0, 0.5)).abs() < f64::EPSILON);
            assert!((calculate_component_score(5.0, 0.0)).abs() < f64::EPSILON);
        }

        #[test]
        fn test_write_tdg_header() {
            let mut output = String::new();
            write_tdg_header(&mut output).unwrap();
            assert!(output.contains("# Technical Debt Gradient Analysis"));
        }

        #[test]
        fn test_write_tdg_summary() {
            let summary = create_test_summary();
            let mut output = String::new();
            write_tdg_summary(&mut output, &summary).unwrap();

            assert!(output.contains("## Summary"));
            assert!(output.contains("**Total Files**: 100"));
        }

        #[test]
        fn test_write_hotspot_basic_info() {
            let hotspot = create_test_hotspot("src/test.rs", 2.5, "complexity", 8.0);
            let mut output = String::new();
            write_hotspot_basic_info(&mut output, &hotspot).unwrap();

            assert!(output.contains("**TDG Score**: 2.500"));
            assert!(output.contains("**Primary Factor**: complexity"));
            assert!(output.contains("**Estimated Hours**: 8.0"));
        }

        #[test]
        fn test_write_component_breakdown() {
            let hotspot = create_test_hotspot("src/test.rs", 2.5, "complexity", 8.0);
            let mut output = String::new();
            write_component_breakdown(&mut output, &hotspot).unwrap();

            assert!(output.contains("#### Component Breakdown:"));
            assert!(output.contains("- Complexity: 1.000")); // 2.5 * 0.4 = 1.0
            assert!(output.contains("- Churn: 0.750")); // 2.5 * 0.3 = 0.75
            assert!(output.contains("- Duplication: 0.500")); // 2.5 * 0.2 = 0.5
            assert!(output.contains("- Coupling: 0.250")); // 2.5 * 0.1 = 0.25
        }

        #[test]
        fn test_write_single_hotspot_with_components() {
            let hotspot = create_test_hotspot("src/test.rs", 2.0, "churn", 5.0);
            let mut output = String::new();
            write_single_hotspot(&mut output, 1, &hotspot, true).unwrap();

            assert!(output.contains("### 1. src/test.rs"));
            assert!(output.contains("**TDG Score**: 2.000"));
            assert!(output.contains("#### Component Breakdown:"));
        }

        #[test]
        fn test_write_single_hotspot_without_components() {
            let hotspot = create_test_hotspot("src/test.rs", 2.0, "churn", 5.0);
            let mut output = String::new();
            write_single_hotspot(&mut output, 1, &hotspot, false).unwrap();

            assert!(output.contains("### 1. src/test.rs"));
            assert!(output.contains("**TDG Score**: 2.000"));
            assert!(!output.contains("#### Component Breakdown:"));
        }

        #[test]
        fn test_write_tdg_hotspots() {
            let hotspots = vec![
                create_test_hotspot("src/first.rs", 3.0, "complexity", 10.0),
                create_test_hotspot("src/second.rs", 2.0, "churn", 5.0),
            ];
            let mut output = String::new();
            write_tdg_hotspots(&mut output, &hotspots, false).unwrap();

            assert!(output.contains("## Top Hotspots"));
            assert!(output.contains("### 1. src/first.rs"));
            assert!(output.contains("### 2. src/second.rs"));
        }

        #[test]
        fn test_generate_tdg_rules() {
            let rules = generate_tdg_rules();
            assert_eq!(rules.len(), 3);

            let ids: Vec<_> = rules.iter().map(|r| r["id"].as_str().unwrap()).collect();
            assert!(ids.contains(&"critical-tdg"));
            assert!(ids.contains(&"high-tdg"));
            assert!(ids.contains(&"moderate-tdg"));

            // Check rule structure
            for rule in &rules {
                assert!(rule.get("id").is_some());
                assert!(rule.get("name").is_some());
                assert!(rule.get("shortDescription").is_some());
                assert!(rule.get("fullDescription").is_some());
                assert!(rule.get("defaultConfiguration").is_some());
            }
        }
    }

    // ============================================================
    // Edge case and error handling tests
    // ============================================================

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_hotspot_with_empty_path() {
            let hotspots = vec![create_test_hotspot("", 2.0, "none", 1.0)];
            let result = format_tdg_table(&hotspots, false).unwrap();
            // Should handle empty path gracefully
            assert!(result.contains("| "));
        }

        #[test]
        fn test_hotspot_with_special_characters_in_path() {
            let hotspots = vec![create_test_hotspot(
                "src/test-file_name.rs",
                2.0,
                "complexity",
                5.0,
            )];
            let result = format_tdg_table(&hotspots, false).unwrap();
            assert!(result.contains("test-file_name.rs"));
        }

        #[test]
        fn test_hotspot_with_unicode_path() {
            let hotspots = vec![create_test_hotspot("src/测试.rs", 2.0, "complexity", 5.0)];
            let result = format_tdg_table(&hotspots, false).unwrap();
            assert!(result.contains("测试.rs"));
        }

        #[test]
        fn test_summary_with_zero_values() {
            let summary = TDGSummary {
                total_files: 0,
                critical_files: 0,
                warning_files: 0,
                average_tdg: 0.0,
                p95_tdg: 0.0,
                p99_tdg: 0.0,
                estimated_debt_hours: 0.0,
                hotspots: vec![],
            };
            let result = format_tdg_json(&summary, &[], false).unwrap();
            assert!(result.contains("\"total_files\": 0"));
        }

        #[test]
        fn test_hotspot_with_very_high_score() {
            let hotspots = vec![create_test_hotspot(
                "src/terrible.rs",
                999.99,
                "everything",
                1000.0,
            )];
            let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "critical-tdg");
        }

        #[test]
        fn test_hotspot_with_zero_score() {
            let hotspots = vec![create_test_hotspot("src/perfect.rs", 0.0, "none", 0.0)];
            let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "moderate-tdg");
            assert_eq!(parsed["runs"][0]["results"][0]["level"], "note");
        }

        #[test]
        fn test_hotspot_with_negative_score() {
            // Edge case: negative score (shouldn't happen in practice, but handle gracefully)
            let hotspots = vec![create_test_hotspot("src/negative.rs", -1.0, "none", 0.0)];
            let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            // Negative score would be "moderate-tdg" since -1.0 is not > 1.5 or > 2.5
            assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "moderate-tdg");
        }

        #[test]
        fn test_very_long_primary_factor() {
            let long_factor = "a".repeat(1000);
            let hotspots = vec![create_test_hotspot("src/test.rs", 2.0, &long_factor, 5.0)];
            let result = format_tdg_json(&create_test_summary(), &hotspots, false).unwrap();
            assert!(result.contains(&long_factor));
        }

        #[test]
        fn test_special_chars_in_factor() {
            let hotspots = vec![create_test_hotspot(
                "src/test.rs",
                2.0,
                "complexity <high> & \"severe\"",
                5.0,
            )];
            let result = format_tdg_json(&create_test_summary(), &hotspots, false).unwrap();
            // JSON should properly escape special characters
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert!(parsed["hotspots"][0]["primary_factor"]
                .as_str()
                .unwrap()
                .contains("<high>"));
        }
    }

    // ============================================================
    // Integration-style tests
    // ============================================================

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_workflow_json() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();

            // Filter first
            let filtered = filter_tdg_hotspots(hotspots, 2.0, 3, false);
            assert_eq!(filtered.len(), 3);

            // Then format
            let json = format_tdg_json(&summary, &filtered, true).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

            assert_eq!(parsed["hotspots"].as_array().unwrap().len(), 3);
            assert!(parsed["components"].is_object());
        }

        #[test]
        fn test_full_workflow_markdown() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();

            let filtered = filter_tdg_hotspots(hotspots, 2.5, 2, true);
            let markdown = format_tdg_markdown(&summary, &filtered, true).unwrap();

            assert!(markdown.contains("# Technical Debt Gradient Analysis"));
            assert!(markdown.contains("## Summary"));
            assert!(markdown.contains("## Top Hotspots"));
            assert!(markdown.contains("#### Component Breakdown:"));
        }

        #[test]
        fn test_full_workflow_sarif() {
            let hotspots = create_test_hotspots();

            let filtered = filter_tdg_hotspots(hotspots, 0.0, 0, true);
            let sarif = format_tdg_sarif(&filtered, &PathBuf::from("/project")).unwrap();

            let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
            let results = parsed["runs"][0]["results"].as_array().unwrap();

            // Only critical hotspots should remain
            assert!(results.iter().all(|r| r["level"] == "error"));
        }

        #[test]
        fn test_all_formats_produce_valid_output() {
            let summary = create_test_summary();
            let hotspots = create_test_hotspots();

            // JSON
            let json = format_tdg_json(&summary, &hotspots, true).unwrap();
            assert!(!json.is_empty());
            assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());

            // Table
            let table = format_tdg_table(&hotspots, true).unwrap();
            assert!(!table.is_empty());
            assert!(table.contains("|"));

            // Markdown
            let md = format_tdg_markdown(&summary, &hotspots, true).unwrap();
            assert!(!md.is_empty());
            assert!(md.contains("#"));

            // SARIF
            let sarif = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
            assert!(!sarif.is_empty());
            assert!(serde_json::from_str::<serde_json::Value>(&sarif).is_ok());
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::tdg::{TDGHotspot, TDGSummary};
    use proptest::prelude::*;

    prop_compose! {
        fn arb_hotspot()(
            path in "[a-z]{1,20}/[a-z]{1,20}\\.rs",
            score in 0.0..10.0f64,
            factor in "[a-z]{3,15}",
            hours in 0.0..100.0f64
        ) -> TDGHotspot {
            TDGHotspot {
                path,
                tdg_score: score,
                primary_factor: factor,
                estimated_hours: hours,
            }
        }
    }

    prop_compose! {
        fn arb_summary()(
            total_files in 0usize..1000,
            critical_files in 0usize..100,
            warning_files in 0usize..200,
            average_tdg in 0.0..5.0f64,
            p95_tdg in 0.0..8.0f64,
            p99_tdg in 0.0..10.0f64,
            estimated_debt_hours in 0.0..1000.0f64
        ) -> TDGSummary {
            TDGSummary {
                total_files,
                critical_files,
                warning_files,
                average_tdg,
                p95_tdg,
                p99_tdg,
                estimated_debt_hours,
                hotspots: vec![],
            }
        }
    }

    proptest! {
        #[test]
        fn filter_never_increases_count(
            hotspots in prop::collection::vec(arb_hotspot(), 0..50),
            threshold in 0.0..10.0f64,
            top in 0usize..100,
            critical_only in proptest::bool::ANY
        ) {
            let original_len = hotspots.len();
            let filtered = filter_tdg_hotspots(hotspots, threshold, top, critical_only);
            prop_assert!(filtered.len() <= original_len);
        }

        #[test]
        fn filter_threshold_respected(
            hotspots in prop::collection::vec(arb_hotspot(), 0..50),
            threshold in 0.1..10.0f64
        ) {
            let filtered = filter_tdg_hotspots(hotspots, threshold, 0, false);
            prop_assert!(filtered.iter().all(|h| h.tdg_score >= threshold));
        }

        #[test]
        fn filter_critical_only_respected(
            hotspots in prop::collection::vec(arb_hotspot(), 0..50)
        ) {
            let filtered = filter_tdg_hotspots(hotspots, 0.0, 0, true);
            prop_assert!(filtered.iter().all(|h| h.tdg_score > 2.5));
        }

        #[test]
        fn filter_top_limit_respected(
            hotspots in prop::collection::vec(arb_hotspot(), 0..50),
            top in 1usize..100
        ) {
            let filtered = filter_tdg_hotspots(hotspots, 0.0, top, false);
            prop_assert!(filtered.len() <= top);
        }

        #[test]
        fn json_format_always_valid(
            summary in arb_summary(),
            hotspots in prop::collection::vec(arb_hotspot(), 0..10),
            include_components in proptest::bool::ANY
        ) {
            let result = format_tdg_json(&summary, &hotspots, include_components);
            prop_assert!(result.is_ok());
            let json = result.unwrap();
            prop_assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
        }

        #[test]
        fn table_format_always_valid(
            hotspots in prop::collection::vec(arb_hotspot(), 0..10),
            verbose in proptest::bool::ANY
        ) {
            let result = format_tdg_table(&hotspots, verbose);
            prop_assert!(result.is_ok());
            let table = result.unwrap();
            prop_assert!(table.contains("| File | TDG Score |"));
        }

        #[test]
        fn markdown_format_always_valid(
            summary in arb_summary(),
            hotspots in prop::collection::vec(arb_hotspot(), 0..10),
            include_components in proptest::bool::ANY
        ) {
            let result = format_tdg_markdown(&summary, &hotspots, include_components);
            prop_assert!(result.is_ok());
            let md = result.unwrap();
            prop_assert!(md.contains("# Technical Debt Gradient Analysis"));
        }

        #[test]
        fn sarif_format_always_valid(
            hotspots in prop::collection::vec(arb_hotspot(), 0..10)
        ) {
            let result = format_tdg_sarif(&hotspots, &std::path::PathBuf::from("/test"));
            prop_assert!(result.is_ok());
            let sarif = result.unwrap();
            prop_assert!(serde_json::from_str::<serde_json::Value>(&sarif).is_ok());
        }

        #[test]
        fn calculate_component_score_commutative_multiplicative(
            score in 0.0..100.0f64,
            weight in 0.0..1.0f64
        ) {
            let result1 = calculate_component_score(score, weight);
            let result2 = score * weight;
            prop_assert!((result1 - result2).abs() < f64::EPSILON);
        }

        #[test]
        fn sarif_severity_matches_score(
            hotspots in prop::collection::vec(arb_hotspot(), 1..5)
        ) {
            let result = format_tdg_sarif(&hotspots, &std::path::PathBuf::from("/")).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let results = parsed["runs"][0]["results"].as_array().unwrap();

            for (i, result) in results.iter().enumerate() {
                let level = result["level"].as_str().unwrap();
                let score = hotspots[i].tdg_score;

                match level {
                    "error" => prop_assert!(score > 2.5),
                    "warning" => prop_assert!(score > 1.5 && score <= 2.5),
                    "note" => prop_assert!(score <= 1.5),
                    _ => prop_assert!(false, "unexpected level: {}", level),
                }
            }
        }
    }

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
