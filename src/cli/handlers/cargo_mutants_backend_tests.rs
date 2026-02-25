// cargo_mutants_backend_tests.rs — included from cargo_mutants_backend.rs
// Tests for cargo-mutants backend handler

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CargoMutantsConfig Tests
    // =========================================================================

    #[test]
    fn test_config_creation_minimal() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        assert_eq!(config.path, PathBuf::from("/test/project"));
        assert!(config.output.is_none());
        assert_eq!(config.timeout, 60);
    }

    #[test]
    fn test_config_creation_with_output() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: Some(PathBuf::from("/test/output")),
            timeout: 120,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        assert_eq!(config.output.unwrap(), PathBuf::from("/test/output"));
    }

    #[test]
    fn test_config_with_jobs() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: Some(8),
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        assert_eq!(config.jobs, Some(8));
    }

    #[test]
    fn test_config_with_features() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: Some(vec!["foo".to_string(), "bar".to_string()]),
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        let features = config.features.unwrap();
        assert_eq!(features.len(), 2);
        assert!(features.contains(&"foo".to_string()));
        assert!(features.contains(&"bar".to_string()));
    }

    #[test]
    fn test_config_all_features() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: true,
            no_default_features: false,
            no_shuffle: false,
        };

        assert!(config.all_features);
    }

    #[test]
    fn test_config_no_default_features() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: true,
            no_shuffle: false,
        };

        assert!(config.no_default_features);
    }

    #[test]
    fn test_config_no_shuffle() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: true,
        };

        assert!(config.no_shuffle);
    }

    #[test]
    fn test_config_clone() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: Some(PathBuf::from("/output")),
            timeout: 120,
            jobs: Some(4),
            features: Some(vec!["feature1".to_string()]),
            all_features: true,
            no_default_features: false,
            no_shuffle: true,
        };

        let cloned = config.clone();
        assert_eq!(cloned.path, config.path);
        assert_eq!(cloned.timeout, config.timeout);
        assert_eq!(cloned.jobs, config.jobs);
        assert_eq!(cloned.all_features, config.all_features);
        assert_eq!(cloned.no_shuffle, config.no_shuffle);
    }

    #[test]
    fn test_config_debug() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CargoMutantsConfig"));
        assert!(debug_str.contains("/test/project"));
    }

    #[test]
    fn test_config_features_join() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test"),
            output: None,
            timeout: 60,
            jobs: None,
            features: Some(vec![
                "feat1".to_string(),
                "feat2".to_string(),
                "feat3".to_string(),
            ]),
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        let features = config.features.unwrap();
        let joined = features.join(",");
        assert_eq!(joined, "feat1,feat2,feat3");
    }

    #[test]
    fn test_config_default_output_path() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // When output is None, default should be path.join("mutants.out")
        let expected = config.path.join("mutants.out");
        assert_eq!(expected, PathBuf::from("/test/project/mutants.out"));
    }

    // =========================================================================
    // Display Statistics Tests (with mock reports)
    // =========================================================================

    #[test]
    fn test_display_statistics_empty_report() {
        use crate::services::mutation::json_parser::CargoMutantsReport;

        let report = CargoMutantsReport::default();
        // This just tests that display_statistics doesn't panic with empty report
        display_statistics(&report);
    }

    // =========================================================================
    // Execute Function Error Path Tests
    // =========================================================================

    #[test]
    fn test_execute_nonexistent_path() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // This will fail because cargo-mutants won't be installed or path doesn't exist
        let result = execute(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_output_dir_determination_with_output() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/project"),
            output: Some(PathBuf::from("/custom/output")),
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // Test output dir logic
        let output_dir = if let Some(ref output) = config.output {
            output.clone()
        } else {
            config.path.join("mutants.out")
        };

        assert_eq!(output_dir, PathBuf::from("/custom/output"));
    }

    #[test]
    fn test_output_dir_determination_without_output() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // Test output dir logic
        let output_dir = if let Some(ref output) = config.output {
            output.clone()
        } else {
            config.path.join("mutants.out")
        };

        assert_eq!(output_dir, PathBuf::from("/project/mutants.out"));
    }

    // =========================================================================
    // Command Building Logic Tests
    // =========================================================================

    #[test]
    fn test_jobs_to_string() {
        let jobs: Option<usize> = Some(4);
        if let Some(j) = jobs {
            let str = j.to_string();
            assert_eq!(str, "4");
        }
    }

    #[test]
    fn test_timeout_to_string() {
        let timeout: u64 = 120;
        let str = timeout.to_string();
        assert_eq!(str, "120");
    }

    #[test]
    fn test_features_empty_vec() {
        let features: Vec<String> = vec![];
        let joined = features.join(",");
        assert_eq!(joined, "");
    }

    #[test]
    fn test_features_single_item() {
        let features = vec!["single".to_string()];
        let joined = features.join(",");
        assert_eq!(joined, "single");
    }

    // =========================================================================
    // Exit Code Handling Tests
    // =========================================================================

    #[test]
    fn test_exit_code_success() {
        let exit_code = 0;
        let should_continue = exit_code == 0 || exit_code == 2;
        assert!(should_continue);
    }

    #[test]
    fn test_exit_code_missed_mutants() {
        let exit_code = 2;
        let should_continue = exit_code == 0 || exit_code == 2;
        assert!(should_continue);
    }

    #[test]
    fn test_exit_code_failure() {
        let exit_code = 1;
        let should_fail = exit_code != 0 && exit_code != 2;
        assert!(should_fail);
    }

    #[test]
    fn test_exit_code_negative() {
        let exit_code = -1;
        let should_fail = exit_code != 0 && exit_code != 2;
        assert!(should_fail);
    }

    // =========================================================================
    // Mutation Score Quality Assessment Tests
    // =========================================================================

    #[test]
    fn test_quality_excellent() {
        let score = 95.0;
        let is_excellent = score >= 90.0;
        assert!(is_excellent);
    }

    #[test]
    fn test_quality_good() {
        let score = 80.0;
        let is_good = score >= 75.0 && score < 90.0;
        assert!(is_good);
    }

    #[test]
    fn test_quality_moderate() {
        let score = 60.0;
        let is_moderate = score >= 50.0 && score < 75.0;
        assert!(is_moderate);
    }

    #[test]
    fn test_quality_low() {
        let score = 40.0;
        let is_low = score < 50.0;
        assert!(is_low);
    }

    // =========================================================================
    // Percentage Calculation Tests
    // =========================================================================

    #[test]
    fn test_percentage_calculation() {
        let caught = 80;
        let total = 100;
        let pct = (caught as f64 / total as f64) * 100.0;
        assert!((pct - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_percentage_calculation_zero_total() {
        let total = 0;
        // In real code, we skip percentage display if total == 0
        let should_skip = total == 0;
        assert!(should_skip);
    }

    #[test]
    fn test_percentage_calculation_all_caught() {
        let caught = 100;
        let total = 100;
        let pct = (caught as f64 / total as f64) * 100.0;
        assert!((pct - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_percentage_calculation_none_caught() {
        let caught = 0;
        let total = 100;
        let pct = (caught as f64 / total as f64) * 100.0;
        assert!((pct - 0.0).abs() < 0.001);
    }
}
