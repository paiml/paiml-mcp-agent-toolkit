// Property-based tests for TDG formatter using proptest.
// Validates invariants: header presence, component toggling, hotspot numbering,
// decimal formatting precision, and percentage calculation correctness.

#[cfg_attr(coverage_nightly, coverage(off))]
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
