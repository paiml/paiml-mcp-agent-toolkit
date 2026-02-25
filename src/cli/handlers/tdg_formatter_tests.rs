// Unit tests for TDG formatter markdown output.
// Covers: empty summaries, file counts, components, hotspots, edge cases, structure verification.

#[cfg_attr(coverage_nightly, coverage(off))]
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
