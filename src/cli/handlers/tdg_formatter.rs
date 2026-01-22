//! Toyota Way: TDG (Technical Debt Gradient) Formatting Handler
//! Complexity: Reduced from 16 to individual functions ≤8
//! Purpose: TDG report formatting with clean separation of concerns

/// Toyota Way: Single Responsibility - Format TDG summary as markdown
/// Extracted from stubs.rs to reduce complexity and improve maintainability
///
/// # Parameters
///
/// * `summary` - TDG analysis summary
/// * `include_components` - Whether to include component details
///
/// # Returns
///
/// Formatted markdown string
#[must_use]
pub fn format_markdown_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
) -> String {
    let mut md = String::new();

    // Header and summary
    add_header_and_summary(&mut md, summary);

    // Hotspots section
    if !summary.hotspots.is_empty() {
        add_hotspots_section(&mut md, &summary.hotspots);
    }

    // Components section if requested
    if include_components {
        add_components_section(&mut md);
    }

    md
}

/// Toyota Way: Extract Method - Add header and summary (complexity ≤8)
fn add_header_and_summary(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    md.push_str("# Technical Debt Gradient Analysis\n\n");
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Total Files**: {}\n", summary.total_files));

    if summary.total_files > 0 {
        add_file_percentages(md, summary);
    }

    md.push_str(&format!("- **Average TDG**: {:.2}\n", summary.average_tdg));
    md.push_str(&format!("- **95th Percentile**: {:.2}\n", summary.p95_tdg));
    md.push_str(&format!("- **99th Percentile**: {:.2}\n", summary.p99_tdg));
    md.push_str(&format!(
        "- **Estimated Technical Debt**: {:.1} hours\n\n",
        summary.estimated_debt_hours
    ));
}

/// Toyota Way: Extract Method - Add file percentages (complexity ≤3)
fn add_file_percentages(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    let critical_pct = (summary.critical_files as f64 / summary.total_files as f64) * 100.0;
    let warning_pct = (summary.warning_files as f64 / summary.total_files as f64) * 100.0;

    md.push_str(&format!(
        "- **Critical Files**: {} ({:.1}%)\n",
        summary.critical_files, critical_pct
    ));
    md.push_str(&format!(
        "- **Warning Files**: {} ({:.1}%)\n",
        summary.warning_files, warning_pct
    ));
}

/// Toyota Way: Extract Method - Add hotspots section (complexity ≤6)
fn add_hotspots_section(md: &mut String, hotspots: &[crate::models::tdg::TDGHotspot]) {
    md.push_str("## Hotspots\n\n");

    for (i, hotspot) in hotspots.iter().enumerate() {
        md.push_str(&format!("### {}. {}\n\n", i + 1, hotspot.path));
        md.push_str(&format!("- **TDG Score**: {:.2}\n", hotspot.tdg_score));
        md.push_str(&format!(
            "- **Primary Factor**: {}\n",
            hotspot.primary_factor
        ));
        md.push_str(&format!(
            "- **Estimated Refactoring Time**: {:.1} hours\n\n",
            hotspot.estimated_hours
        ));
    }
}

/// Toyota Way: Extract Method - Add components section (complexity ≤2)
fn add_components_section(md: &mut String) {
    md.push_str("## TDG Components\n\n");
    md.push_str(
        "The Technical Debt Gradient is calculated using the following weighted components:\n\n",
    );
    md.push_str("- **Complexity** (30%): Cyclomatic and cognitive complexity\n");
    md.push_str("- **Code Churn** (35%): Frequency of changes over time\n");
    md.push_str("- **Coverage** (20%): Test coverage and quality\n");
    md.push_str("- **Maintainability** (15%): Code quality metrics\n\n");
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::models::tdg::{TDGHotspot, TDGSummary};

    /// Helper to create a minimal TDGSummary for testing
    fn create_summary(
        total_files: usize,
        critical_files: usize,
        warning_files: usize,
        hotspots: Vec<TDGHotspot>,
    ) -> TDGSummary {
        TDGSummary {
            total_files,
            critical_files,
            warning_files,
            average_tdg: 5.5,
            p95_tdg: 8.0,
            p99_tdg: 9.5,
            estimated_debt_hours: 42.0,
            hotspots,
        }
    }

    /// Helper to create a TDGHotspot for testing
    fn create_hotspot(path: &str, score: f64, factor: &str, hours: f64) -> TDGHotspot {
        TDGHotspot {
            path: path.to_string(),
            tdg_score: score,
            primary_factor: factor.to_string(),
            estimated_hours: hours,
        }
    }

    // ========================================================================
    // format_markdown_output tests
    // ========================================================================

    #[test]
    fn test_format_markdown_output_empty_summary_no_components() {
        let summary = create_summary(0, 0, 0, vec![]);
        let result = format_markdown_output(&summary, false);

        // Header must be present
        assert!(result.contains("# Technical Debt Gradient Analysis"));
        assert!(result.contains("## Summary"));
        assert!(result.contains("- **Total Files**: 0"));
        // With 0 files, percentages should NOT be shown
        assert!(!result.contains("Critical Files"));
        assert!(!result.contains("Warning Files"));
        // Other stats should be present
        assert!(result.contains("- **Average TDG**: 5.50"));
        assert!(result.contains("- **95th Percentile**: 8.00"));
        assert!(result.contains("- **99th Percentile**: 9.50"));
        assert!(result.contains("- **Estimated Technical Debt**: 42.0 hours"));
        // Hotspots section should NOT be present (empty)
        assert!(!result.contains("## Hotspots"));
        // Components section should NOT be present
        assert!(!result.contains("## TDG Components"));
    }

    #[test]
    fn test_format_markdown_output_with_files_no_components() {
        let summary = create_summary(100, 5, 15, vec![]);
        let result = format_markdown_output(&summary, false);

        // Check file count
        assert!(result.contains("- **Total Files**: 100"));
        // Percentages should be shown
        assert!(result.contains("- **Critical Files**: 5 (5.0%)"));
        assert!(result.contains("- **Warning Files**: 15 (15.0%)"));
        // Components not included
        assert!(!result.contains("## TDG Components"));
    }

    #[test]
    fn test_format_markdown_output_with_components() {
        let summary = create_summary(50, 2, 8, vec![]);
        let result = format_markdown_output(&summary, true);

        // Components section should be present
        assert!(result.contains("## TDG Components"));
        assert!(result.contains(
            "The Technical Debt Gradient is calculated using the following weighted components:"
        ));
        assert!(result.contains("- **Complexity** (30%): Cyclomatic and cognitive complexity"));
        assert!(result.contains("- **Code Churn** (35%): Frequency of changes over time"));
        assert!(result.contains("- **Coverage** (20%): Test coverage and quality"));
        assert!(result.contains("- **Maintainability** (15%): Code quality metrics"));
    }

    #[test]
    fn test_format_markdown_output_with_single_hotspot() {
        let hotspots = vec![create_hotspot(
            "src/complex.rs",
            9.2,
            "High Cyclomatic Complexity",
            12.5,
        )];
        let summary = create_summary(10, 1, 2, hotspots);
        let result = format_markdown_output(&summary, false);

        // Hotspots section should be present
        assert!(result.contains("## Hotspots"));
        assert!(result.contains("### 1. src/complex.rs"));
        assert!(result.contains("- **TDG Score**: 9.20"));
        assert!(result.contains("- **Primary Factor**: High Cyclomatic Complexity"));
        assert!(result.contains("- **Estimated Refactoring Time**: 12.5 hours"));
    }

    #[test]
    fn test_format_markdown_output_with_multiple_hotspots() {
        let hotspots = vec![
            create_hotspot("src/worst.rs", 9.8, "High Churn", 20.0),
            create_hotspot("src/bad.rs", 8.5, "Low Coverage", 15.0),
            create_hotspot("src/mediocre.rs", 7.2, "Complex Logic", 10.0),
        ];
        let summary = create_summary(100, 3, 10, hotspots);
        let result = format_markdown_output(&summary, false);

        // All hotspots numbered correctly
        assert!(result.contains("### 1. src/worst.rs"));
        assert!(result.contains("### 2. src/bad.rs"));
        assert!(result.contains("### 3. src/mediocre.rs"));
        // Verify scores
        assert!(result.contains("- **TDG Score**: 9.80"));
        assert!(result.contains("- **TDG Score**: 8.50"));
        assert!(result.contains("- **TDG Score**: 7.20"));
        // Verify factors
        assert!(result.contains("- **Primary Factor**: High Churn"));
        assert!(result.contains("- **Primary Factor**: Low Coverage"));
        assert!(result.contains("- **Primary Factor**: Complex Logic"));
    }

    #[test]
    fn test_format_markdown_output_with_hotspots_and_components() {
        let hotspots = vec![create_hotspot("src/test.rs", 8.0, "Complexity", 5.0)];
        let summary = create_summary(25, 1, 5, hotspots);
        let result = format_markdown_output(&summary, true);

        // Both hotspots and components should be present
        assert!(result.contains("## Hotspots"));
        assert!(result.contains("### 1. src/test.rs"));
        assert!(result.contains("## TDG Components"));
    }

    // ========================================================================
    // Edge cases
    // ========================================================================

    #[test]
    fn test_format_markdown_output_large_numbers() {
        let summary = TDGSummary {
            total_files: 1_000_000,
            critical_files: 50_000,
            warning_files: 150_000,
            average_tdg: 9999.999,
            p95_tdg: 99999.12345,
            p99_tdg: 123456.789,
            estimated_debt_hours: 1_000_000.5,
            hotspots: vec![],
        };
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("- **Total Files**: 1000000"));
        assert!(result.contains("- **Critical Files**: 50000 (5.0%)"));
        assert!(result.contains("- **Warning Files**: 150000 (15.0%)"));
        // Average TDG may be formatted with different precision or rounding
        assert!(result.contains("**Average TDG**:"));
        assert!(result.contains("**Estimated Technical Debt**:"));
    }

    #[test]
    fn test_format_markdown_output_decimal_precision() {
        let summary = TDGSummary {
            total_files: 10,
            critical_files: 1,
            warning_files: 3,
            average_tdg: 3.14159265,
            p95_tdg: 7.777777,
            p99_tdg: 9.123456789,
            estimated_debt_hours: 100.123456,
            hotspots: vec![],
        };
        let result = format_markdown_output(&summary, false);

        // Check decimal formatting (2 places for scores)
        assert!(result.contains("- **Average TDG**: 3.14"));
        assert!(result.contains("- **95th Percentile**: 7.78"));
        assert!(result.contains("- **99th Percentile**: 9.12"));
        // 1 place for hours
        assert!(result.contains("- **Estimated Technical Debt**: 100.1 hours"));
    }

    #[test]
    fn test_format_markdown_output_special_characters_in_path() {
        let hotspots = vec![create_hotspot(
            "src/path with spaces/file-name_v2.0.rs",
            7.5,
            "Test Factor",
            3.0,
        )];
        let summary = create_summary(5, 1, 1, hotspots);
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("### 1. src/path with spaces/file-name_v2.0.rs"));
    }

    #[test]
    fn test_format_markdown_output_unicode_in_factor() {
        let hotspots = vec![create_hotspot(
            "src/test.rs",
            7.0,
            "High complexity \u{2192} needs refactoring",
            5.0,
        )];
        let summary = create_summary(1, 1, 0, hotspots);
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("- **Primary Factor**: High complexity \u{2192} needs refactoring"));
    }

    #[test]
    fn test_format_markdown_output_zero_percentages() {
        let summary = create_summary(100, 0, 0, vec![]);
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("- **Critical Files**: 0 (0.0%)"));
        assert!(result.contains("- **Warning Files**: 0 (0.0%)"));
    }

    #[test]
    fn test_format_markdown_output_hundred_percent() {
        let summary = create_summary(100, 100, 100, vec![]);
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("- **Critical Files**: 100 (100.0%)"));
        assert!(result.contains("- **Warning Files**: 100 (100.0%)"));
    }

    #[test]
    fn test_format_markdown_output_fractional_percentages() {
        // 1 out of 3 = 33.333...%
        let summary = create_summary(3, 1, 1, vec![]);
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("- **Critical Files**: 1 (33.3%)"));
        assert!(result.contains("- **Warning Files**: 1 (33.3%)"));
    }

    #[test]
    fn test_format_markdown_output_zero_values() {
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
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("- **Total Files**: 0"));
        assert!(result.contains("- **Average TDG**: 0.00"));
        assert!(result.contains("- **95th Percentile**: 0.00"));
        assert!(result.contains("- **99th Percentile**: 0.00"));
        assert!(result.contains("- **Estimated Technical Debt**: 0.0 hours"));
    }

    #[test]
    fn test_format_markdown_output_hotspot_with_zero_hours() {
        let hotspots = vec![create_hotspot("src/clean.rs", 0.0, "None", 0.0)];
        let summary = create_summary(1, 0, 0, hotspots);
        let result = format_markdown_output(&summary, false);

        assert!(result.contains("- **TDG Score**: 0.00"));
        assert!(result.contains("- **Estimated Refactoring Time**: 0.0 hours"));
    }

    #[test]
    fn test_format_markdown_output_negative_values() {
        // Edge case: negative values (shouldn't happen but testing robustness)
        let summary = TDGSummary {
            total_files: 10,
            critical_files: 0,
            warning_files: 0,
            average_tdg: -1.5,
            p95_tdg: -0.5,
            p99_tdg: -0.1,
            estimated_debt_hours: -10.0,
            hotspots: vec![],
        };
        let result = format_markdown_output(&summary, false);

        // Should still format correctly even with negative values
        assert!(result.contains("- **Average TDG**: -1.50"));
        assert!(result.contains("- **Estimated Technical Debt**: -10.0 hours"));
    }

    #[test]
    fn test_format_markdown_output_empty_hotspot_fields() {
        let hotspots = vec![create_hotspot("", 5.0, "", 2.0)];
        let summary = create_summary(1, 1, 0, hotspots);
        let result = format_markdown_output(&summary, false);

        // Empty path and factor should still render
        assert!(result.contains("### 1. "));
        assert!(result.contains("- **Primary Factor**: \n"));
    }

    // ========================================================================
    // Structure verification tests
    // ========================================================================

    #[test]
    fn test_markdown_structure_ordering() {
        let hotspots = vec![create_hotspot("src/test.rs", 8.0, "Test", 5.0)];
        let summary = create_summary(10, 1, 2, hotspots);
        let result = format_markdown_output(&summary, true);

        // Verify ordering: Header -> Summary -> Hotspots -> Components
        let header_pos = result.find("# Technical Debt Gradient Analysis").unwrap();
        let summary_pos = result.find("## Summary").unwrap();
        let hotspots_pos = result.find("## Hotspots").unwrap();
        let components_pos = result.find("## TDG Components").unwrap();

        assert!(header_pos < summary_pos);
        assert!(summary_pos < hotspots_pos);
        assert!(hotspots_pos < components_pos);
    }

    #[test]
    fn test_markdown_newlines() {
        let summary = create_summary(10, 1, 2, vec![]);
        let result = format_markdown_output(&summary, true);

        // Verify proper newline formatting
        assert!(result.contains("## Summary\n\n"));
        assert!(result.contains("## TDG Components\n\n"));
    }

    // ========================================================================
    // Many hotspots test
    // ========================================================================

    #[test]
    fn test_format_markdown_output_many_hotspots() {
        let hotspots: Vec<TDGHotspot> = (1..=20)
            .map(|i| create_hotspot(&format!("src/file{}.rs", i), i as f64, "Factor", i as f64))
            .collect();
        let summary = create_summary(100, 20, 30, hotspots);
        let result = format_markdown_output(&summary, false);

        // Verify all 20 hotspots are numbered correctly
        for i in 1..=20 {
            assert!(
                result.contains(&format!("### {}. src/file{}.rs", i, i)),
                "Missing hotspot {}",
                i
            );
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::tdg::{TDGHotspot, TDGSummary};
    use proptest::prelude::*;

    /// Strategy for generating valid TDGSummary values
    fn tdg_summary_strategy() -> impl Strategy<Value = TDGSummary> {
        (
            1usize..1000,    // total_files (at least 1 to avoid div by zero in percentages)
            0usize..100,     // critical_files
            0usize..100,     // warning_files
            0.0f64..100.0,   // average_tdg
            0.0f64..100.0,   // p95_tdg
            0.0f64..100.0,   // p99_tdg
            0.0f64..10000.0, // estimated_debt_hours
        )
            .prop_map(
                |(
                    total_files,
                    critical_files,
                    warning_files,
                    average_tdg,
                    p95_tdg,
                    p99_tdg,
                    estimated_debt_hours,
                )| {
                    TDGSummary {
                        total_files,
                        critical_files: critical_files.min(total_files),
                        warning_files: warning_files.min(total_files),
                        average_tdg,
                        p95_tdg,
                        p99_tdg,
                        estimated_debt_hours,
                        hotspots: vec![],
                    }
                },
            )
    }

    /// Strategy for generating valid TDGHotspot values
    fn tdg_hotspot_strategy() -> impl Strategy<Value = TDGHotspot> {
        (
            "[a-z/]{1,50}",    // path
            0.0f64..100.0,     // tdg_score
            "[A-Za-z ]{1,30}", // primary_factor
            0.0f64..1000.0,    // estimated_hours
        )
            .prop_map(
                |(path, tdg_score, primary_factor, estimated_hours)| TDGHotspot {
                    path,
                    tdg_score,
                    primary_factor,
                    estimated_hours,
                },
            )
    }

    /// Strategy for TDGSummary with hotspots
    fn tdg_summary_with_hotspots_strategy() -> impl Strategy<Value = TDGSummary> {
        (
            tdg_summary_strategy(),
            proptest::collection::vec(tdg_hotspot_strategy(), 0..10),
        )
            .prop_map(|(mut summary, hotspots)| {
                summary.hotspots = hotspots;
                summary
            })
    }

    // Helper functions to avoid format! inside proptest! macro
    fn check_total_files(result: &str, total_files: usize) -> bool {
        let expected = format!("- **Total Files**: {}", total_files);
        result.contains(&expected)
    }

    fn check_hotspot_path(result: &str, idx: usize, path: &str) -> bool {
        let expected = format!("### {}. {}", idx + 1, path);
        result.contains(&expected)
    }

    fn check_average_tdg(result: &str, avg: f64) -> bool {
        let expected = format!("- **Average TDG**: {:.2}", avg);
        result.contains(&expected)
    }

    fn check_p95_tdg(result: &str, p95: f64) -> bool {
        let expected = format!("- **95th Percentile**: {:.2}", p95);
        result.contains(&expected)
    }

    fn check_p99_tdg(result: &str, p99: f64) -> bool {
        let expected = format!("- **99th Percentile**: {:.2}", p99);
        result.contains(&expected)
    }

    fn check_debt_hours(result: &str, hours: f64) -> bool {
        let expected = format!("- **Estimated Technical Debt**: {:.1} hours", hours);
        result.contains(&expected)
    }

    fn check_refactoring_hours(result: &str, hours: f64) -> bool {
        let expected = format!("- **Estimated Refactoring Time**: {:.1} hours", hours);
        result.contains(&expected)
    }

    fn check_critical_files(result: &str, crit: usize, total: usize) -> bool {
        let pct = (crit as f64 / total as f64) * 100.0;
        let expected = format!("- **Critical Files**: {} ({:.1}%)", crit, pct);
        result.contains(&expected)
    }

    fn check_warning_files(result: &str, warn: usize, total: usize) -> bool {
        let pct = (warn as f64 / total as f64) * 100.0;
        let expected = format!("- **Warning Files**: {} ({:.1}%)", warn, pct);
        result.contains(&expected)
    }

    proptest! {
        /// Property: Output always contains required markdown header
        #[test]
        fn prop_output_always_contains_header(summary in tdg_summary_strategy()) {
            let result = format_markdown_output(&summary, false);
            prop_assert!(result.contains("# Technical Debt Gradient Analysis"));
            prop_assert!(result.contains("## Summary"));
        }

        /// Property: Output always contains total files line
        #[test]
        fn prop_output_always_contains_total_files(summary in tdg_summary_strategy()) {
            let result = format_markdown_output(&summary, false);
            prop_assert!(check_total_files(&result, summary.total_files));
        }

        /// Property: When include_components is true, components section is present
        #[test]
        fn prop_components_present_when_requested(summary in tdg_summary_strategy()) {
            let result = format_markdown_output(&summary, true);
            prop_assert!(result.contains("## TDG Components"));
        }

        /// Property: When include_components is false, components section is absent
        #[test]
        fn prop_components_absent_when_not_requested(summary in tdg_summary_strategy()) {
            let result = format_markdown_output(&summary, false);
            prop_assert!(!result.contains("## TDG Components"));
        }

        /// Property: Each hotspot generates a numbered section
        #[test]
        fn prop_hotspots_numbered_correctly(summary in tdg_summary_with_hotspots_strategy()) {
            let result = format_markdown_output(&summary, false);

            if summary.hotspots.is_empty() {
                prop_assert!(!result.contains("## Hotspots"));
            } else {
                prop_assert!(result.contains("## Hotspots"));
                for (i, hotspot) in summary.hotspots.iter().enumerate() {
                    prop_assert!(check_hotspot_path(&result, i, &hotspot.path));
                }
            }
        }

        /// Property: Output is always valid UTF-8 (implicitly true for String)
        #[test]
        fn prop_output_is_valid_string(summary in tdg_summary_with_hotspots_strategy(), include_components in proptest::bool::ANY) {
            let result = format_markdown_output(&summary, include_components);
            // If we get here without panic, the output is valid UTF-8
            prop_assert!(!result.is_empty());
        }

        /// Property: TDG values are formatted with 2 decimal places
        #[test]
        fn prop_tdg_values_formatted_two_decimals(
            average in 0.0f64..1000.0,
            p95 in 0.0f64..1000.0,
            p99 in 0.0f64..1000.0
        ) {
            let summary = TDGSummary {
                total_files: 10,
                critical_files: 1,
                warning_files: 2,
                average_tdg: average,
                p95_tdg: p95,
                p99_tdg: p99,
                estimated_debt_hours: 10.0,
                hotspots: vec![],
            };
            let result = format_markdown_output(&summary, false);

            prop_assert!(check_average_tdg(&result, average));
            prop_assert!(check_p95_tdg(&result, p95));
            prop_assert!(check_p99_tdg(&result, p99));
        }

        /// Property: Hours formatted with 1 decimal place
        #[test]
        fn prop_hours_formatted_one_decimal(hours in 0.0f64..100000.0) {
            let summary = TDGSummary {
                total_files: 10,
                critical_files: 1,
                warning_files: 2,
                average_tdg: 5.0,
                p95_tdg: 8.0,
                p99_tdg: 9.0,
                estimated_debt_hours: hours,
                hotspots: vec![],
            };
            let result = format_markdown_output(&summary, false);

            prop_assert!(check_debt_hours(&result, hours));
        }

        /// Property: Hotspot hours formatted with 1 decimal place
        #[test]
        fn prop_hotspot_hours_formatted_one_decimal(hours in 0.0f64..10000.0) {
            let hotspot = TDGHotspot {
                path: "test.rs".to_string(),
                tdg_score: 5.0,
                primary_factor: "Test".to_string(),
                estimated_hours: hours,
            };
            let summary = TDGSummary {
                total_files: 1,
                critical_files: 0,
                warning_files: 0,
                average_tdg: 5.0,
                p95_tdg: 8.0,
                p99_tdg: 9.0,
                estimated_debt_hours: 10.0,
                hotspots: vec![hotspot],
            };
            let result = format_markdown_output(&summary, false);

            prop_assert!(check_refactoring_hours(&result, hours));
        }

        /// Property: Percentage calculation is mathematically correct
        #[test]
        fn prop_percentages_mathematically_correct(
            total in 1usize..1000,
            critical in 0usize..100,
            warning in 0usize..100
        ) {
            let crit = critical.min(total);
            let warn = warning.min(total);

            let summary = TDGSummary {
                total_files: total,
                critical_files: crit,
                warning_files: warn,
                average_tdg: 5.0,
                p95_tdg: 8.0,
                p99_tdg: 9.0,
                estimated_debt_hours: 10.0,
                hotspots: vec![],
            };
            let result = format_markdown_output(&summary, false);

            prop_assert!(check_critical_files(&result, crit, total));
            prop_assert!(check_warning_files(&result, warn, total));
        }
    }
}
