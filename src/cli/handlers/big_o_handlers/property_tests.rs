#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use crate::cli::handlers::big_o_handlers::filters::{
        get_complexity_class_score, get_top_file_paths, is_high_complexity_class,
    };
    use crate::cli::handlers::big_o_handlers::handlers::build_analysis_config;
    use crate::cli::handlers::big_o_handlers::output::format_big_o_summary;
    use crate::models::complexity_bound::BigOClass;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_complexity_score_is_always_positive(class in 0u8..9u8) {
            let big_o_class = match class {
                0 => BigOClass::Constant,
                1 => BigOClass::Logarithmic,
                2 => BigOClass::Linear,
                3 => BigOClass::Linearithmic,
                4 => BigOClass::Quadratic,
                5 => BigOClass::Cubic,
                6 => BigOClass::Exponential,
                7 => BigOClass::Factorial,
                _ => BigOClass::Unknown,
            };
            let score = get_complexity_class_score(&big_o_class);
            prop_assert!(score > 0.0);
        }

        #[test]
        fn test_is_high_complexity_consistent(class in 0u8..9u8) {
            let big_o_class = match class {
                0 => BigOClass::Constant,
                1 => BigOClass::Logarithmic,
                2 => BigOClass::Linear,
                3 => BigOClass::Linearithmic,
                4 => BigOClass::Quadratic,
                5 => BigOClass::Cubic,
                6 => BigOClass::Exponential,
                7 => BigOClass::Factorial,
                _ => BigOClass::Unknown,
            };

            let is_high = is_high_complexity_class(&big_o_class);
            let expected_high = matches!(
                big_o_class,
                BigOClass::Quadratic | BigOClass::Cubic | BigOClass::Exponential | BigOClass::Factorial
            );

            prop_assert_eq!(is_high, expected_high);
        }

        #[test]
        fn test_build_config_preserves_threshold(threshold in 0u8..=100u8) {
            let config = build_analysis_config(
                std::path::PathBuf::from("/test"),
                vec![],
                vec![],
                threshold,
                false,
            );
            prop_assert_eq!(config.confidence_threshold, threshold);
        }

        #[test]
        fn test_build_config_preserves_patterns(
            include_count in 0usize..10,
            exclude_count in 0usize..10,
        ) {
            let includes: Vec<String> = (0..include_count).map(|i| format!("*.{i}")).collect();
            let excludes: Vec<String> = (0..exclude_count).map(|i| format!("exclude_{i}")).collect();

            let config = build_analysis_config(
                std::path::PathBuf::from("/test"),
                includes.clone(),
                excludes.clone(),
                50,
                true,
            );

            prop_assert_eq!(config.include_patterns.len(), include_count);
            prop_assert_eq!(config.exclude_patterns.len(), exclude_count);
        }

        #[test]
        fn test_top_files_never_exceeds_input(
            file_count in 1usize..20,
            top_n in 0usize..25,
        ) {
            let scores: Vec<(std::path::PathBuf, f64)> = (0..file_count)
                .map(|i| (std::path::PathBuf::from(format!("file_{i}.rs")), i as f64))
                .collect();

            let top = get_top_file_paths(scores, top_n);

            prop_assert!(top.len() <= file_count);
            prop_assert!(top.len() <= top_n);
        }

        #[test]
        fn test_summary_format_never_panics(
            analyzed in 0usize..1000,
            constant in 0usize..100,
            linear in 0usize..100,
        ) {
            let report = crate::services::big_o_analyzer::BigOAnalysisReport {
                analyzed_functions: analyzed,
                complexity_distribution: crate::services::big_o_analyzer::ComplexityDistribution {
                    constant,
                    logarithmic: 0,
                    linear,
                    linearithmic: 0,
                    quadratic: 0,
                    cubic: 0,
                    exponential: 0,
                    unknown: 0,
                },
                high_complexity_functions: vec![],
                pattern_matches: vec![],
                recommendations: vec![],
            };

            // Should not panic
            let _ = format_big_o_summary(&report);
        }
    }
}
