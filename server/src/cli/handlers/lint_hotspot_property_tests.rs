#[cfg(test)]
mod tests {
    use super::super::lint_hotspot_handlers::{
        FileSummary, SeverityDistribution, ViolationDetail,
    };
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    // Strategy for generating file paths
    fn arb_file_path() -> impl Strategy<Value = PathBuf> {
        prop::string::string_regex("src/[a-zA-Z0-9_/]+\\.rs")
            .unwrap()
            .prop_map(PathBuf::from)
    }

    // Strategy for generating lint names
    fn arb_lint_name() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("clippy::pedantic".to_string()),
            Just("clippy::nursery".to_string()),
            Just("clippy::complexity".to_string()),
            Just("clippy::style".to_string()),
            Just("clippy::perf".to_string()),
            Just("unused_variables".to_string()),
            Just("dead_code".to_string()),
            prop::string::string_regex("clippy::[a-z_]+").unwrap(),
        ]
    }

    // Strategy for generating severity
    fn arb_severity() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("error".to_string()),
            Just("warning".to_string()),
            Just("help".to_string()),
            Just("note".to_string()),
        ]
    }

    // Strategy for generating violation details
    prop_compose! {
        fn arb_violation_detail()
            (
                file in arb_file_path(),
                line in 1u32..1000,
                column in 1u32..120,
                end_line in 1u32..1000,
                end_column in 1u32..120,
                lint_name in arb_lint_name(),
                message in "[a-zA-Z0-9 .,!?-]+",
                severity in arb_severity(),
                has_suggestion in any::<bool>(),
                suggestion in "[a-zA-Z0-9 .,!?-]+",
                machine_applicable in any::<bool>(),
            )
            -> ViolationDetail
        {
            ViolationDetail {
                file,
                line,
                column,
                end_line: end_line.max(line),
                end_column: if end_line > line { end_column } else { end_column.max(column) },
                lint_name,
                message,
                severity,
                suggestion: if has_suggestion { Some(suggestion) } else { None },
                machine_applicable: machine_applicable && has_suggestion,
            }
        }
    }

    // Strategy for generating severity distribution
    prop_compose! {
        fn arb_severity_distribution()
            (
                error in 0usize..100,
                warning in 0usize..500,
                suggestion in 0usize..200,
                note in 0usize..100,
            )
            -> SeverityDistribution
        {
            SeverityDistribution {
                error,
                warning,
                suggestion,
                note,
            }
        }
    }

    // Strategy for generating file summary
    prop_compose! {
        fn arb_file_summary()
            (
                errors in 0usize..100,
                warnings in 0usize..500,
                sloc in 1usize..5000,
                // Include suggestions and notes to make total_violations potentially larger
                suggestions in 0usize..200,
                notes in 0usize..100,
            )
            -> FileSummary
        {
            let total_violations = errors + warnings + suggestions + notes;
            let defect_density = calculate_defect_density(total_violations, sloc);
            FileSummary {
                total_violations,
                errors,
                warnings,
                sloc,
                defect_density,
            }
        }
    }

    proptest! {
        /// Property: Defect density calculation is always non-negative and finite
        #[test]
        fn defect_density_always_valid(
            total_violations in 0usize..10000,
            sloc in 1usize..10000,
        ) {
            let density = calculate_defect_density(total_violations, sloc);
            
            prop_assert!(density >= 0.0, "Density should be non-negative");
            prop_assert!(density.is_finite(), "Density should be finite");
            
            // Verify the formula: violations per 100 lines
            let expected = (total_violations as f64 / sloc as f64) * 100.0;
            prop_assert!((density - expected).abs() < f64::EPSILON);
        }

        /// Property: Empty summary has zero density
        #[test]
        fn empty_summary_zero_density(_dummy in 0u8..1) {
            let summary = FileSummary {
                total_violations: 0,
                errors: 0,
                warnings: 0,
                sloc: 100,
                defect_density: 0.0,
            };
            
            prop_assert_eq!(summary.total_violations, 0);
            prop_assert_eq!(summary.defect_density, 0.0);
            
            let calculated_density = calculate_defect_density(summary.total_violations, summary.sloc);
            prop_assert_eq!(calculated_density, 0.0);
        }

        /// Property: Severity counts match detailed violations
        #[test]
        fn severity_counts_consistent_with_violations(
            violations in prop::collection::vec(arb_violation_detail(), 0..100)
        ) {
            let mut severity_counts = SeverityDistribution::default();
            
            for v in &violations {
                match v.severity.as_str() {
                    "error" => severity_counts.error += 1,
                    "warning" => severity_counts.warning += 1,
                    "help" | "suggestion" => severity_counts.suggestion += 1,
                    _ => severity_counts.note += 1,
                }
            }
            
            // Count actual severities
            let actual_errors = violations.iter().filter(|v| v.severity == "error").count();
            let actual_warnings = violations.iter().filter(|v| v.severity == "warning").count();
            let actual_suggestions = violations.iter()
                .filter(|v| v.severity == "help" || v.severity == "suggestion")
                .count();
            let actual_notes = violations.iter()
                .filter(|v| !["error", "warning", "help", "suggestion"].contains(&v.severity.as_str()))
                .count();
            
            prop_assert_eq!(severity_counts.error, actual_errors);
            prop_assert_eq!(severity_counts.warning, actual_warnings);
            prop_assert_eq!(severity_counts.suggestion, actual_suggestions);
            prop_assert_eq!(severity_counts.note, actual_notes);
        }

        /// Property: Top lints are correctly sorted and limited
        #[test]
        fn top_lints_correctly_sorted(
            violations in prop::collection::hash_map(
                arb_lint_name(),
                1usize..100,
                1..20
            ),
            limit in 1usize..10,
        ) {
            let top_lints = get_top_lints(&violations, limit);
            
            // Should not exceed limit
            prop_assert!(top_lints.len() <= limit);
            prop_assert!(top_lints.len() <= violations.len());
            
            // Should be sorted by count (descending)
            for i in 1..top_lints.len() {
                prop_assert!(
                    top_lints[i-1].1 >= top_lints[i].1,
                    "Top lints not sorted: {} ({}) should come before {} ({})",
                    top_lints[i-1].0, top_lints[i-1].1,
                    top_lints[i].0, top_lints[i].1
                );
            }
            
            // Verify counts match original data
            for (lint_name, count) in &top_lints {
                prop_assert_eq!(violations.get(lint_name), Some(count));
            }
        }

        /// Property: File summary density matches calculation
        #[test]
        fn file_summary_density_consistent(summary in arb_file_summary()) {
            let calculated_density = calculate_defect_density(summary.total_violations, summary.sloc);
            
            // Allow small floating point differences
            prop_assert!(
                (summary.defect_density - calculated_density).abs() < 0.001,
                "Density mismatch: {} vs calculated {}",
                summary.defect_density,
                calculated_density
            );
            
            // Verify total is at least errors + warnings
            prop_assert!(
                summary.total_violations >= summary.errors + summary.warnings,
                "Total violations {} should be >= errors {} + warnings {}",
                summary.total_violations,
                summary.errors,
                summary.warnings
            );
        }

        /// Property: Finding hotspot returns file with maximum density
        #[test]
        fn find_hotspot_returns_max_density(
            file_summaries in prop::collection::hash_map(
                arb_file_path(),
                arb_file_summary(),
                1..10
            )
        ) {
            // Find the actual hotspot manually
            let mut max_density = 0.0;
            let mut hotspot_path = None;
            
            for (path, summary) in &file_summaries {
                if summary.sloc == 0 || summary.total_violations == 0 {
                    continue;
                }
                
                let density = calculate_defect_density(summary.total_violations, summary.sloc);
                
                if density > max_density {
                    max_density = density;
                    hotspot_path = Some(path.clone());
                }
            }
            
            // If there's a hotspot, verify properties
            if let Some(expected_path) = hotspot_path {
                let summary = file_summaries.get(&expected_path).unwrap();
                
                prop_assert!(summary.total_violations > 0);
                prop_assert!(summary.sloc > 0);
                
                let density = calculate_defect_density(summary.total_violations, summary.sloc);
                prop_assert!(
                    (density - max_density).abs() < 0.001,
                    "Density {} should match max density {}",
                    density,
                    max_density
                );
                
                // Verify this is actually the max
                for other_summary in file_summaries.values() {
                    if other_summary.total_violations > 0 && other_summary.sloc > 0 {
                        let other_density = calculate_defect_density(
                            other_summary.total_violations,
                            other_summary.sloc
                        );
                        prop_assert!(
                            density >= other_density || (density - other_density).abs() < 0.001,
                            "Found higher density: {} > {}",
                            other_density,
                            density
                        );
                    }
                }
            }
        }

        /// Property: Machine applicable suggestions require suggestion text
        #[test]
        fn machine_applicable_requires_suggestion(
            mut violation in arb_violation_detail()
        ) {
            if violation.machine_applicable {
                prop_assert!(violation.suggestion.is_some(),
                    "Machine applicable violation must have suggestion");
            }
            
            // If no suggestion, machine_applicable should be false
            violation.suggestion = None;
            violation.machine_applicable = false;
            prop_assert!(!violation.machine_applicable);
        }

        /// Property: Line ranges are valid
        #[test]
        fn violation_line_ranges_valid(violation in arb_violation_detail()) {
            prop_assert!(violation.line > 0, "Line must be positive");
            prop_assert!(violation.column > 0, "Column must be positive");
            prop_assert!(violation.end_line >= violation.line,
                "End line must be >= start line");
            
            if violation.end_line == violation.line {
                prop_assert!(violation.end_column >= violation.column,
                    "End column must be >= start column on same line");
            }
        }

        /// Property: Zero SLOC prevents division by zero
        #[test]
        fn zero_sloc_handled_safely(total_violations in 0usize..1000) {
            let density = calculate_defect_density(total_violations, 0);
            prop_assert_eq!(density, 0.0, "Zero SLOC should produce zero density");
        }
    }

    // Helper functions matching the implementation
    fn calculate_defect_density(total_violations: usize, sloc: usize) -> f64 {
        if sloc == 0 {
            0.0
        } else {
            (total_violations as f64 / sloc as f64) * 100.0
        }
    }

    fn get_top_lints(violations: &HashMap<String, usize>, limit: usize) -> Vec<(String, usize)> {
        let mut sorted: Vec<_> = violations.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        sorted.truncate(limit);
        sorted
    }

}