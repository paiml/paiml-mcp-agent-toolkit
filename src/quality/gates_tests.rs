// Unit and integration tests for quality gate execution.
// Tests cover: config defaults, coverage parsing, report formatting,
// serialization, cleanup utilities, and integration tests (ignored).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_gate_config_default() {
        let config = GateConfig::default();

        assert!(config.run_clippy);
        assert!(config.clippy_strict);
        assert!(config.run_tests);
        assert_eq!(config.test_timeout, 300);
        assert!(config.check_coverage);
        assert_eq!(config.min_coverage, 80.0);
        assert!(config.check_complexity);
        assert_eq!(config.max_complexity, 10);
    }

    #[test]
    fn test_parse_coverage_from_output() {
        let output = "TOTAL    lines: 1000    85.50%";
        let coverage = parse_coverage_from_output(output);
        assert_eq!(coverage, 85.5);
    }

    #[test]
    fn test_parse_coverage_multiline() {
        let output = "file.rs    100    90.0%\nTOTAL    1000    85.5%\nother data";
        let coverage = parse_coverage_from_output(output);
        assert_eq!(coverage, 85.5);
    }

    #[test]
    fn test_parse_coverage_no_match() {
        let output = "No coverage data";
        let coverage = parse_coverage_from_output(output);
        assert_eq!(coverage, 0.0);
    }

    #[test]
    fn test_format_report_pass() {
        use std::time::Duration;

        let report = QualityReport {
            gates: vec![
                GateResult {
                    name: "clippy".to_string(),
                    passed: true,
                    duration: Duration::from_secs(5),
                    message: "✓ Clippy passed".to_string(),
                },
                GateResult {
                    name: "tests".to_string(),
                    passed: true,
                    duration: Duration::from_secs(10),
                    message: "✓ Tests passed".to_string(),
                },
            ],
            passed: true,
            total_duration: Duration::from_secs(15),
            timestamp: "2025-10-05T10:00:00Z".to_string(),
        };

        let formatted = format_report(&report);

        assert!(formatted.contains("Quality Gate Report"));
        assert!(formatted.contains("✅ PASS"));
        assert!(formatted.contains("clippy"));
        assert!(formatted.contains("tests"));
    }

    #[test]
    fn test_format_report_fail() {
        use std::time::Duration;

        let report = QualityReport {
            gates: vec![GateResult {
                name: "clippy".to_string(),
                passed: false,
                duration: Duration::from_secs(5),
                message: "✗ Clippy failed".to_string(),
            }],
            passed: false,
            total_duration: Duration::from_secs(5),
            timestamp: "2025-10-05T10:00:00Z".to_string(),
        };

        let formatted = format_report(&report);

        assert!(formatted.contains("❌ FAIL"));
        assert!(formatted.contains("✗"));
    }

    #[test]
    fn test_gate_result_serialization() {
        use std::time::Duration;

        let result = GateResult {
            name: "test".to_string(),
            passed: true,
            duration: Duration::from_millis(1500),
            message: "ok".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: GateResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_quality_report_all_pass() {
        use std::time::Duration;

        let report = QualityReport {
            gates: vec![
                GateResult {
                    name: "gate1".to_string(),
                    passed: true,
                    duration: Duration::from_secs(1),
                    message: "ok".to_string(),
                },
                GateResult {
                    name: "gate2".to_string(),
                    passed: true,
                    duration: Duration::from_secs(1),
                    message: "ok".to_string(),
                },
            ],
            passed: true,
            total_duration: Duration::from_secs(2),
            timestamp: "2025-10-05T10:00:00Z".to_string(),
        };

        assert!(report.passed);
    }

    #[test]
    fn test_quality_report_some_fail() {
        use std::time::Duration;

        let report = QualityReport {
            gates: vec![
                GateResult {
                    name: "gate1".to_string(),
                    passed: true,
                    duration: Duration::from_secs(1),
                    message: "ok".to_string(),
                },
                GateResult {
                    name: "gate2".to_string(),
                    passed: false,
                    duration: Duration::from_secs(1),
                    message: "fail".to_string(),
                },
            ],
            passed: false,
            total_duration: Duration::from_secs(2),
            timestamp: "2025-10-05T10:00:00Z".to_string(),
        };

        assert!(!report.passed);
    }

    /// SLOW: 106s - excluded from fast test suite
    #[test]
    #[ignore = "requires quality gate setup"]
    fn integration_execute_clippy() {
        let config = GateConfig::default();
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let result = execute_clippy(&config, &project_dir);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Integration test that runs full test suite + clippy (PMAT-COVERAGE-003)
              // Takes 12+ minutes, times out at 600s, causes recursive test execution
              // Run manually with: cargo test integration_execute_all_gates -- --ignored
    fn integration_execute_all_gates() {
        let config = GateConfig {
            run_clippy: true,
            clippy_strict: false,
            run_tests: true,
            test_timeout: 600,
            check_coverage: false,
            min_coverage: 0.0,
            check_complexity: false,
            max_complexity: 10,
        };
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let report = execute_all_gates(&config, &project_dir).unwrap();
        assert!(!report.gates.is_empty());
    }

    // Tests for cleanup_coverage_artifacts (TICKET-PMAT-9)

    #[test]
    fn test_cleanup_coverage_artifacts_nonexistent_dir() {
        // Should not panic when directories don't exist
        let nonexistent = PathBuf::from("/nonexistent/path/12345");
        cleanup_coverage_artifacts(&nonexistent);
        // Success if no panic
    }

    #[test]
    fn test_cleanup_coverage_artifacts_current_dir() {
        use std::fs;
        use tempfile::tempdir;

        // Use a tempdir — calling this with CARGO_MANIFEST_DIR during a coverage
        // run would wipe llvm-cov's own target/llvm-cov-target and break
        // object-file collection (caused PR #296 coverage job failure).
        let temp = tempdir().unwrap();
        let project_dir = temp.path();
        fs::create_dir_all(project_dir.join("target").join("llvm-cov-target")).unwrap();
        cleanup_coverage_artifacts(project_dir);
        assert!(!project_dir.join("target").join("llvm-cov-target").exists());
    }

    #[test]
    fn test_clean_old_files_nonexistent() {
        // Should not panic on nonexistent directory
        let nonexistent = Path::new("/nonexistent/path/12345");
        clean_old_files(nonexistent, 3600);
        // Success if no panic
    }

    #[test]
    fn test_clean_old_files_empty_dir() {
        use std::fs;
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let empty_dir = temp.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();

        clean_old_files(&empty_dir, 0); // 0 seconds = clean all
                                        // Should not panic
        assert!(empty_dir.exists()); // Directory itself should still exist
    }

    #[test]
    fn test_clean_old_files_with_old_file() {
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let test_dir = temp.path().join("test");
        fs::create_dir(&test_dir).unwrap();

        // Create a file
        let file_path = test_dir.join("old_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test").unwrap();

        // Clean with 0 second max age (everything is old)
        clean_old_files(&test_dir, 0);

        // File should be deleted
        assert!(!file_path.exists());
    }

    #[test]
    fn test_clean_old_files_preserves_new_files() {
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let test_dir = temp.path().join("test");
        fs::create_dir(&test_dir).unwrap();

        // Create a file
        let file_path = test_dir.join("new_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test").unwrap();

        // Clean with very high max age (nothing is old enough)
        clean_old_files(&test_dir, 86400 * 365); // 1 year

        // File should still exist
        assert!(file_path.exists());
    }

    /// gates_checks.rs:231-233 — `if path.is_dir() { remove_dir_all(&path) }`.
    /// Previous tests only exercised the `else { remove_file(&path) }` arm for
    /// stale files. This creates a stale SUB-DIRECTORY (with nested content)
    /// under a 0-second max_age window so the dir branch fires.
    #[test]
    fn test_clean_old_files_removes_old_directory() {
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir(&root).unwrap();

        // Stale sub-dir with nested file — must be removed via remove_dir_all.
        let stale_subdir = root.join("stale_sub");
        fs::create_dir(&stale_subdir).unwrap();
        let nested = stale_subdir.join("inner.txt");
        let mut f = File::create(&nested).unwrap();
        writeln!(f, "stale").unwrap();

        clean_old_files(&root, 0); // everything counts as "old"

        assert!(
            !stale_subdir.exists(),
            "is_dir() arm must delete the whole stale subdirectory recursively"
        );
        assert!(
            !nested.exists(),
            "nested file under the deleted dir must be gone too"
        );
    }

    // ── R5 extracted helpers (wave 35 PR1) ──────────────────────────────────

    #[test]
    fn test_build_clippy_message_passed() {
        let m = build_clippy_message(true, "");
        assert_eq!(m, "✓ Clippy passed");
    }

    #[test]
    fn test_build_clippy_message_failed_includes_stderr_first_10_lines() {
        let stderr = (1..=15)
            .map(|i| format!("err line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let m = build_clippy_message(false, &stderr);
        assert!(m.starts_with("✗ Clippy failed:"));
        assert!(m.contains("err line 1"));
        assert!(m.contains("err line 10"));
        assert!(!m.contains("err line 11"), "must take only first 10 lines");
    }

    #[test]
    fn test_build_clippy_message_failed_empty_stderr() {
        let m = build_clippy_message(false, "");
        // Empty stderr → just the header with empty body
        assert_eq!(m, "✗ Clippy failed:\n");
    }

    #[test]
    fn test_build_test_message_passed() {
        assert_eq!(build_test_message(true, "any", "any"), "✓ Tests passed");
    }

    #[test]
    fn test_build_test_message_failed_picks_failed_lines_from_stdout() {
        let stdout = "running 5 tests\ntest a::b ... FAILED\ntest c ... ok";
        let m = build_test_message(false, stdout, "");
        assert!(m.contains("test a::b ... FAILED"));
        assert!(!m.contains("running 5"));
    }

    #[test]
    fn test_build_test_message_failed_picks_panicked_lines() {
        let stdout = "thread 'foo' panicked at 'oops'";
        let m = build_test_message(false, stdout, "");
        assert!(m.contains("panicked"));
    }

    #[test]
    fn test_build_test_message_failed_picks_compile_error_lines() {
        let stdout = "error[E0308]: mismatched types";
        let m = build_test_message(false, stdout, "");
        assert!(m.contains("error[E0308]"));
    }

    #[test]
    fn test_build_test_message_failed_picks_failures_summary() {
        let stdout = "failures:\n    a::b";
        let m = build_test_message(false, stdout, "");
        assert!(m.contains("failures:"));
        assert!(m.contains("    a::b"));
    }

    #[test]
    fn test_build_test_message_failed_falls_back_to_stderr_when_stdout_clean() {
        // Stdout has no failure lines; stderr has compile errors
        let stdout = "compiling pkg\nfinished";
        let stderr = "error: cannot find type";
        let m = build_test_message(false, stdout, stderr);
        assert!(m.contains("error: cannot find type"));
    }

    #[test]
    fn test_build_test_message_failed_caps_stdout_at_15_lines() {
        let stdout = (1..=20)
            .map(|i| format!("test t_{} ... FAILED", i))
            .collect::<Vec<_>>()
            .join("\n");
        let m = build_test_message(false, &stdout, "");
        assert!(m.contains("t_15"));
        assert!(!m.contains("t_16"));
    }

    #[test]
    fn test_build_test_message_failed_caps_stderr_fallback_at_10_lines() {
        let stderr = (1..=15)
            .map(|i| format!("err {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let m = build_test_message(false, "no failures here", &stderr);
        assert!(m.contains("err 10"));
        assert!(!m.contains("err 11"));
    }

    #[test]
    fn test_build_coverage_decision_passes_at_threshold() {
        // coverage >= min_coverage → passed
        let (p, m) = build_coverage_decision(85.0, 85.0);
        assert!(p);
        assert!(m.contains("✓"));
        assert!(m.contains("85.0%"));
        assert!(m.contains(">="));
    }

    #[test]
    fn test_build_coverage_decision_passes_above_threshold() {
        let (p, m) = build_coverage_decision(95.5, 85.0);
        assert!(p);
        assert!(m.contains("95.5%"));
    }

    #[test]
    fn test_build_coverage_decision_fails_below_threshold() {
        let (p, m) = build_coverage_decision(84.9, 85.0);
        assert!(!p);
        assert!(m.contains("✗"));
        assert!(m.contains("<"));
        assert!(m.contains("84.9%"));
    }

    #[test]
    fn test_build_coverage_decision_zero_below_threshold() {
        let (p, m) = build_coverage_decision(0.0, 50.0);
        assert!(!p);
        assert!(m.contains("0.0%"));
    }

    // ── parse_coverage_from_output (was untested) ───────────────────────────

    #[test]
    fn test_parse_coverage_from_output_finds_total_pct() {
        let output = "Filename ...\nfoo.rs   80.0%\nTOTAL    85.50%   12   3";
        assert!((parse_coverage_from_output(output) - 85.5).abs() < 1e-6);
    }

    #[test]
    fn test_parse_coverage_from_output_no_total_line() {
        let output = "Filename ...\nfoo.rs   80.0%\nbar.rs   90.0%";
        assert_eq!(parse_coverage_from_output(output), 0.0);
    }

    #[test]
    fn test_parse_coverage_from_output_total_no_pct_returns_zero() {
        let output = "TOTAL  no-percentages here";
        assert_eq!(parse_coverage_from_output(output), 0.0);
    }

    #[test]
    fn test_parse_coverage_from_output_empty_string() {
        assert_eq!(parse_coverage_from_output(""), 0.0);
    }

    #[test]
    fn test_parse_coverage_from_output_picks_first_pct_on_total_line() {
        // First %-suffixed token on the TOTAL line is what's picked
        let output = "TOTAL  77.7%   88.8%   99.9%";
        assert!((parse_coverage_from_output(output) - 77.7).abs() < 1e-6);
    }
}
