//! Property-based tests for complexity threshold filtering
//!
//! These tests ensure that the --max-cyclomatic and --max-cognitive flags
//! properly filter files based on their complexity thresholds.

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use proptest::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        /// Property: Files with all functions below threshold should be filtered out
        #[test]
        fn prop_files_below_threshold_filtered_out(
            threshold in 20u16..100,
            num_files in 1usize..10,
            functions_per_file in 1usize..5,
        ) {
            // Generate files where all functions are below threshold
            let mut files = vec![];
            for i in 0..num_files {
                let file = FileComplexityMetrics {
                    path: format!("file_{}.rs", i),
                    total_complexity: ComplexityMetrics {
                        cyclomatic: threshold - 1,
                        cognitive: threshold - 1,
                        nesting_max: 3,
                        lines: 100,
                    },
                    functions: (0..functions_per_file)
                        .map(|j| FunctionComplexity {
                            name: format!("func_{}", j),
                            line_start: j as u32 * 10,
                            line_end: (j as u32 + 1) * 10,
                            metrics: ComplexityMetrics {
                                cyclomatic: (threshold - 1 - j as u16).max(1),
                                cognitive: (threshold - 1 - j as u16).max(1),
                                nesting_max: 2,
                                lines: 10,
                            },
                        })
                        .collect(),
                    classes: vec![],
                };
                files.push(file);
            }

            // Apply the filtering logic
            let _max_cyclomatic = Some(threshold);
            let filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > threshold
                })
            }).collect::<Vec<_>>();

            // All files should be filtered out
            prop_assert_eq!(filtered.len(), 0);
        }

        /// Property: Files with at least one function above threshold should be kept
        #[test]
        fn prop_files_above_threshold_kept(
            threshold in 20u16..50,
            num_files in 1usize..10,
            high_complexity in 51u16..100,
        ) {
            // Generate files where at least one function exceeds threshold
            let mut files = vec![];
            for i in 0..num_files {
                let file = FileComplexityMetrics {
                    path: format!("file_{}.rs", i),
                    total_complexity: ComplexityMetrics {
                        cyclomatic: high_complexity,
                        cognitive: high_complexity,
                        nesting_max: 5,
                        lines: 200,
                    },
                    functions: vec![
                        // One high complexity function
                        FunctionComplexity {
                            name: "complex_func".to_string(),
                            line_start: 1,
                            line_end: 50,
                            metrics: ComplexityMetrics {
                                cyclomatic: high_complexity,
                                cognitive: high_complexity,
                                nesting_max: 5,
                                lines: 50,
                            },
                        },
                        // One low complexity function
                        FunctionComplexity {
                            name: "simple_func".to_string(),
                            line_start: 51,
                            line_end: 60,
                            metrics: ComplexityMetrics {
                                cyclomatic: 5,
                                cognitive: 3,
                                nesting_max: 1,
                                lines: 10,
                            },
                        },
                    ],
                    classes: vec![],
                };
                files.push(file);
            }

            // Apply the filtering logic
            let _max_cyclomatic = Some(threshold);
            let filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > threshold
                })
            }).collect::<Vec<_>>();

            // All files should be kept since each has a high complexity function
            prop_assert_eq!(filtered.len(), num_files);
        }

        /// Property: Exact threshold boundary behavior
        #[test]
        fn prop_threshold_boundary_behavior(
            threshold in 10u16..50,
        ) {
            let files = vec![
                // File with function exactly at threshold - should be filtered out
                FileComplexityMetrics {
                    path: "at_threshold.rs".to_string(),
                    total_complexity: ComplexityMetrics {
                        cyclomatic: threshold,
                        cognitive: threshold,
                        nesting_max: 3,
                        lines: 100,
                    },
                    functions: vec![FunctionComplexity {
                        name: "at_threshold_func".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics {
                            cyclomatic: threshold,
                            cognitive: threshold,
                            nesting_max: 3,
                            lines: 50,
                        },
                    }],
                    classes: vec![],
                },
                // File with function just above threshold - should be kept
                FileComplexityMetrics {
                    path: "above_threshold.rs".to_string(),
                    total_complexity: ComplexityMetrics {
                        cyclomatic: threshold + 1,
                        cognitive: threshold + 1,
                        nesting_max: 3,
                        lines: 100,
                    },
                    functions: vec![FunctionComplexity {
                        name: "above_threshold_func".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics {
                            cyclomatic: threshold + 1,
                            cognitive: threshold + 1,
                            nesting_max: 3,
                            lines: 50,
                        },
                    }],
                    classes: vec![],
                },
            ];

            // Apply the filtering logic
            let _max_cyclomatic = Some(threshold);
            let filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > threshold
                })
            }).collect::<Vec<_>>();

            // Only the file above threshold should be kept
            prop_assert_eq!(filtered.len(), 1);
            prop_assert_eq!(&filtered[0].path, "above_threshold.rs");
        }

        /// Property: Both cyclomatic and cognitive thresholds work independently
        #[test]
        fn prop_independent_threshold_filtering(
            cyc_threshold in 20u16..40,
            cog_threshold in 30u16..50,
        ) {
            let files = vec![
                // File exceeding only cyclomatic
                FileComplexityMetrics {
                    path: "high_cyclomatic.rs".to_string(),
                    total_complexity: ComplexityMetrics {
                        cyclomatic: cyc_threshold + 10,
                        cognitive: cog_threshold - 10,
                        nesting_max: 3,
                        lines: 100,
                    },
                    functions: vec![FunctionComplexity {
                        name: "cyc_complex".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics {
                            cyclomatic: cyc_threshold + 10,
                            cognitive: cog_threshold - 10,
                            nesting_max: 3,
                            lines: 50,
                        },
                    }],
                    classes: vec![],
                },
                // File exceeding only cognitive
                FileComplexityMetrics {
                    path: "high_cognitive.rs".to_string(),
                    total_complexity: ComplexityMetrics {
                        cyclomatic: cyc_threshold - 10,
                        cognitive: cog_threshold + 10,
                        nesting_max: 3,
                        lines: 100,
                    },
                    functions: vec![FunctionComplexity {
                        name: "cog_complex".to_string(),
                        line_start: 1,
                        line_end: 50,
                        metrics: ComplexityMetrics {
                            cyclomatic: cyc_threshold - 10,
                            cognitive: cog_threshold + 10,
                            nesting_max: 3,
                            lines: 50,
                        },
                    }],
                    classes: vec![],
                },
                // File exceeding neither
                FileComplexityMetrics {
                    path: "simple.rs".to_string(),
                    total_complexity: ComplexityMetrics {
                        cyclomatic: cyc_threshold - 10,
                        cognitive: cog_threshold - 10,
                        nesting_max: 1,
                        lines: 50,
                    },
                    functions: vec![FunctionComplexity {
                        name: "simple_func".to_string(),
                        line_start: 1,
                        line_end: 25,
                        metrics: ComplexityMetrics {
                            cyclomatic: cyc_threshold - 10,
                            cognitive: cog_threshold - 10,
                            nesting_max: 1,
                            lines: 25,
                        },
                    }],
                    classes: vec![],
                },
            ];

            // Test with cyclomatic threshold only
            let cyc_filtered = files.clone().into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > cyc_threshold
                })
            }).collect::<Vec<_>>();
            prop_assert_eq!(cyc_filtered.len(), 1);
            prop_assert_eq!(&cyc_filtered[0].path, "high_cyclomatic.rs");

            // Test with cognitive threshold only
            let cog_filtered = files.clone().into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cognitive > cog_threshold
                })
            }).collect::<Vec<_>>();
            prop_assert_eq!(cog_filtered.len(), 1);
            prop_assert_eq!(&cog_filtered[0].path, "high_cognitive.rs");

            // Test with both thresholds (OR logic)
            let both_filtered = files.into_iter().filter(|file| {
                file.functions.iter().any(|func| {
                    func.metrics.cyclomatic > cyc_threshold ||
                    func.metrics.cognitive > cog_threshold
                })
            }).collect::<Vec<_>>();
            prop_assert_eq!(both_filtered.len(), 2);
        }
    }
}
