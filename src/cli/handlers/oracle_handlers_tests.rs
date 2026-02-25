#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_oracle_status_nonexistent_path() {
        let result =
            handle_oracle_status(Path::new("/nonexistent/path"), OracleOutputFormat::Text).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_oracle_fix_nonexistent_path() {
        let result = handle_oracle_fix(
            Path::new("/nonexistent/path"),
            10,
            0.9,
            0.7,
            true,
            OracleOutputFormat::Text,
            None,
        )
        .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_format_pdca_json() {
        let results = vec![];
        let json = format_pdca_json(&results).unwrap();
        assert!(json.contains("iterations"));
        assert!(json.contains("converged"));
    }

    #[test]
    fn test_format_status_text_converged() {
        let metrics = crate::services::oracle::ProjectMetrics {
            test_coverage: 0.95,
            mutation_score: 0.80,
            compiler_errors: 0,
            clippy_warnings: 2,
            test_failures: 0,
            tdg_score: 4.5,
            rust_project_score: 85,
            satd_markers: 10,
            dead_code_items: 5,
            max_cyclomatic_complexity: 12,
            max_cognitive_complexity: 8,
            build_time: std::time::Duration::from_secs(30),
        };
        let targets = crate::services::oracle::ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.70,
            max_compiler_errors: 0,
            max_clippy_warnings: 5,
            max_test_failures: 0,
            min_tdg_score: 3.0,
            min_rust_project_score: 70,
            max_satd_markers: 20,
            max_dead_code: 10,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 15,
            max_build_time: std::time::Duration::from_secs(300),
        };
        let status = crate::services::oracle::ConvergenceStatus::Converged;

        let result = format_status(&metrics, &targets, &status, OracleOutputFormat::Text).unwrap();
        assert!(result.contains("Test Coverage"));
        assert!(result.contains("95.0%"));
        assert!(result.contains("CONVERGED"));
    }

    #[test]
    fn test_format_status_text_not_converged() {
        let metrics = crate::services::oracle::ProjectMetrics {
            test_coverage: 0.50,
            mutation_score: 0.30,
            compiler_errors: 5,
            clippy_warnings: 20,
            test_failures: 3,
            tdg_score: 1.5,
            rust_project_score: 40,
            satd_markers: 50,
            dead_code_items: 30,
            max_cyclomatic_complexity: 25,
            max_cognitive_complexity: 20,
            build_time: std::time::Duration::from_secs(60),
        };
        let targets = crate::services::oracle::ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.70,
            max_compiler_errors: 0,
            max_clippy_warnings: 5,
            max_test_failures: 0,
            min_tdg_score: 3.0,
            min_rust_project_score: 70,
            max_satd_markers: 20,
            max_dead_code: 10,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 15,
            max_build_time: std::time::Duration::from_secs(300),
        };
        let remaining = vec!["Coverage below 90%".to_string()];
        let status = crate::services::oracle::ConvergenceStatus::NotConverged { remaining };

        let result = format_status(&metrics, &targets, &status, OracleOutputFormat::Text).unwrap();
        assert!(result.contains("NOT CONVERGED"));
        assert!(result.contains("Coverage below 90%"));
    }

    #[test]
    fn test_format_status_json() {
        let metrics = crate::services::oracle::ProjectMetrics {
            test_coverage: 0.85,
            mutation_score: 0.75,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 4.0,
            rust_project_score: 80,
            satd_markers: 5,
            dead_code_items: 2,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 8,
            build_time: std::time::Duration::from_secs(45),
        };
        let targets = crate::services::oracle::ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.70,
            max_compiler_errors: 0,
            max_clippy_warnings: 5,
            max_test_failures: 0,
            min_tdg_score: 3.0,
            min_rust_project_score: 70,
            max_satd_markers: 20,
            max_dead_code: 10,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 15,
            max_build_time: std::time::Duration::from_secs(300),
        };
        let status = crate::services::oracle::ConvergenceStatus::Converged;

        let result = format_status(&metrics, &targets, &status, OracleOutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("metrics").is_some());
        assert!(parsed.get("converged").is_some());
        assert_eq!(parsed["converged"].as_bool().unwrap(), true);
    }
}
