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

    // ── banner_enabled: JSON mode must keep stdout jq-parseable ──

    #[test]
    fn test_banner_enabled_suppressed_for_json() {
        assert!(!banner_enabled(OracleOutputFormat::Json));
    }

    #[test]
    fn test_banner_enabled_for_human_formats() {
        assert!(banner_enabled(OracleOutputFormat::Text));
        assert!(banner_enabled(OracleOutputFormat::Markdown));
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

    fn make_iter_result(iteration: usize, converged: bool) -> crate::services::oracle::PdcaIterationResult {
        let metrics = crate::services::oracle::ProjectMetrics::default();
        crate::services::oracle::PdcaIterationResult {
            iteration,
            defects_found: 5,
            defects_fixed: 3,
            defects_skipped: 2,
            metrics_before: metrics.clone(),
            metrics_after: metrics,
            converged,
        }
    }

    fn default_targets() -> crate::services::oracle::ConvergenceTargets {
        crate::services::oracle::ConvergenceTargets::default()
    }

    // ── format_pdca_results dispatcher ──

    #[test]
    fn test_format_pdca_results_dispatcher_text_arm() {
        let results = vec![make_iter_result(1, true)];
        let r = format_pdca_results(&results, &default_targets(), OracleOutputFormat::Text).unwrap();
        assert!(r.contains("PDCA Loop Results"));
    }

    #[test]
    fn test_format_pdca_results_dispatcher_json_arm() {
        let results = vec![make_iter_result(1, true)];
        let r = format_pdca_results(&results, &default_targets(), OracleOutputFormat::Json).unwrap();
        assert!(r.contains("\"iterations\""));
    }

    #[test]
    fn test_format_pdca_results_dispatcher_markdown_arm() {
        let results = vec![make_iter_result(1, true)];
        let r =
            format_pdca_results(&results, &default_targets(), OracleOutputFormat::Markdown).unwrap();
        assert!(r.contains("# PMAT Oracle"));
    }

    // ── format_pdca_text: converged + non-converged + summary section ──

    #[test]
    fn test_format_pdca_text_with_results_emits_summary() {
        let results = vec![make_iter_result(1, false), make_iter_result(2, true)];
        let r = format_pdca_text(&results, &default_targets()).unwrap();
        assert!(r.contains("Iteration 1"));
        assert!(r.contains("Iteration 2"));
        assert!(r.contains("=== Summary ==="));
        // Last iteration converged → "CONVERGED" final status.
        assert!(r.contains("CONVERGED"));
    }

    #[test]
    fn test_format_pdca_text_last_not_converged_shows_not_converged() {
        let results = vec![make_iter_result(1, false)];
        let r = format_pdca_text(&results, &default_targets()).unwrap();
        assert!(r.contains("NOT CONVERGED"));
    }

    #[test]
    fn test_format_pdca_text_empty_results_no_summary() {
        let r = format_pdca_text(&[], &default_targets()).unwrap();
        assert!(r.contains("PDCA Loop Results"));
        assert!(!r.contains("=== Summary ==="));
    }

    #[test]
    fn test_format_pdca_json_with_iterations_returns_correct_total() {
        let results = vec![make_iter_result(1, true), make_iter_result(2, false)];
        let r = format_pdca_json(&results).unwrap();
        assert!(r.contains("\"total_iterations\": 2"));
        // last.converged == false.
        assert!(r.contains("\"converged\": false"));
    }

    #[test]
    fn test_format_pdca_markdown_emits_table_row_per_iteration() {
        let results = vec![make_iter_result(1, true), make_iter_result(2, false)];
        let r = format_pdca_markdown(&results, &default_targets()).unwrap();
        // Both iterations as table rows.
        assert!(r.contains("| 1 |"));
        assert!(r.contains("| 2 |"));
        // Convergence-targets section.
        assert!(r.contains("## Convergence Targets"));
    }

    // ── format_iteration_result dispatcher (no output path) ──

    #[test]
    fn test_format_iteration_result_text_to_stdout() {
        let r = make_iter_result(1, true);
        // None output_path → println! to stdout.
        format_iteration_result(&r, &OracleOutputFormat::Text, None).unwrap();
    }

    #[test]
    fn test_format_iteration_result_json_to_stdout() {
        let r = make_iter_result(1, true);
        format_iteration_result(&r, &OracleOutputFormat::Json, None).unwrap();
    }

    #[test]
    fn test_format_iteration_result_markdown_to_stdout() {
        let r = make_iter_result(1, true);
        format_iteration_result(&r, &OracleOutputFormat::Markdown, None).unwrap();
    }

    #[test]
    fn test_format_iteration_result_writes_to_provided_path() {
        let temp = tempfile::TempDir::new().unwrap();
        let out_path = temp.path().join("iter.txt");
        let r = make_iter_result(1, true);
        format_iteration_result(&r, &OracleOutputFormat::Text, Some(&out_path)).unwrap();
        let written = std::fs::read_to_string(&out_path).unwrap();
        assert!(written.contains("Defects found:"));
    }

    // ── format_single_result dispatcher (3 arms) ──

    #[test]
    fn test_format_single_result_text_arm() {
        let r = make_iter_result(1, true);
        let out = format_single_result(&r, OracleOutputFormat::Text).unwrap();
        assert!(out.contains("Single PDCA Iteration"));
        assert!(out.contains("Converged: YES"));
    }

    #[test]
    fn test_format_single_result_json_arm() {
        let r = make_iter_result(2, false);
        let out = format_single_result(&r, OracleOutputFormat::Json).unwrap();
        // Verify it parses as JSON with expected keys.
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["iteration"], 2);
        assert_eq!(v["converged"], false);
    }

    #[test]
    fn test_format_single_result_markdown_arm() {
        let r = make_iter_result(1, false);
        let out = format_single_result(&r, OracleOutputFormat::Markdown).unwrap();
        assert!(out.contains("# Single PDCA Iteration"));
        // Non-converged → ❌ in table.
        assert!(out.contains("❌"));
    }

    // ── format_status Markdown arm (existing tests cover Text + Json) ──

    #[test]
    fn test_format_status_markdown_emits_table_with_status_icons() {
        let metrics = crate::services::oracle::ProjectMetrics {
            test_coverage: 0.95,
            mutation_score: 0.80,
            compiler_errors: 0,
            ..Default::default()
        };
        let targets = crate::services::oracle::ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.70,
            max_compiler_errors: 5,
            ..crate::services::oracle::ConvergenceTargets::default()
        };
        let status = crate::services::oracle::ConvergenceStatus::Converged;
        let r =
            format_status(&metrics, &targets, &status, OracleOutputFormat::Markdown).unwrap();
        assert!(r.contains("# Project Quality Status"));
        // All three metrics should pass with ✅.
        assert!(r.contains("✅"));
    }

    #[test]
    fn test_format_status_text_json() {
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
        assert!(parsed["converged"].as_bool().unwrap());
    }
}
