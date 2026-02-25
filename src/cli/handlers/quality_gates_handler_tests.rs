// Extracted from quality_gates_handler.rs — unit tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::gates::GateResult;
    use std::time::Duration;

    #[test]
    fn test_load_config_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[gates]
run_clippy = true
clippy_strict = false
run_tests = true
test_timeout = 300
check_coverage = true
min_coverage = 85.0
check_complexity = true
max_complexity = 8
"#
        )
        .unwrap();

        let config = load_config_from_file(&temp_file.path().to_path_buf()).unwrap();

        assert!(config.run_clippy);
        assert!(!config.clippy_strict);
        assert_eq!(config.min_coverage, 85.0);
        assert_eq!(config.max_complexity, 8);
    }

    #[test]
    fn test_output_json() {
        let report = QualityReport {
            gates: vec![GateResult {
                name: "test".to_string(),
                passed: true,
                duration: Duration::from_secs(1),
                message: "ok".to_string(),
            }],
            passed: true,
            total_duration: Duration::from_secs(1),
            timestamp: "2025-10-05T10:00:00Z".to_string(),
        };

        // Should not panic
        output_json(&report).unwrap();
    }

    #[test]
    fn test_output_markdown() {
        let report = QualityReport {
            gates: vec![GateResult {
                name: "test".to_string(),
                passed: true,
                duration: Duration::from_secs(1),
                message: "ok".to_string(),
            }],
            passed: true,
            total_duration: Duration::from_secs(1),
            timestamp: "2025-10-05T10:00:00Z".to_string(),
        };

        // Should not panic
        output_markdown(&report).unwrap();
    }

    #[test]
    fn test_output_summary() {
        let report = QualityReport {
            gates: vec![
                GateResult {
                    name: "clippy".to_string(),
                    passed: true,
                    duration: Duration::from_secs(5),
                    message: "ok".to_string(),
                },
                GateResult {
                    name: "tests".to_string(),
                    passed: false,
                    duration: Duration::from_secs(10),
                    message: "Failed:\nTest 1\nTest 2".to_string(),
                },
            ],
            passed: false,
            total_duration: Duration::from_secs(15),
            timestamp: "2025-10-05T10:00:00Z".to_string(),
        };

        // Should not panic
        output_summary(&report).unwrap();
    }

    #[test]
    fn test_gate_config_toml_conversion() {
        let toml = GateConfigToml {
            gates: GateConfigInner {
                run_clippy: true,
                clippy_strict: false,
                run_tests: true,
                test_timeout: 300,
                check_coverage: true,
                min_coverage: 80.0,
                check_complexity: true,
                max_complexity: 10,
            },
        };

        let config: GateConfig = toml.into();

        assert!(config.run_clippy);
        assert!(!config.clippy_strict);
        assert!(config.run_tests);
        assert_eq!(config.test_timeout, 300);
        assert!(config.check_coverage);
        assert_eq!(config.min_coverage, 80.0);
        assert!(config.check_complexity);
        assert_eq!(config.max_complexity, 10);
    }

    // TICKET-PMAT-5024: Configuration management tests

    #[test]
    fn test_handle_init_config() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".pmat-gates.toml");

        handle_init_config(&config_path, false).unwrap();

        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[gates]"));
        assert!(content.contains("run_clippy = true"));
    }

    #[test]
    fn test_handle_init_config_already_exists() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let result = handle_init_config(temp_file.path(), false);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_handle_init_config_force_overwrite() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "old content").unwrap();

        handle_init_config(temp_file.path(), true).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("[gates]"));
        assert!(!content.contains("old content"));
    }

    #[test]
    fn test_handle_validate_config_valid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[gates]
run_clippy = true
clippy_strict = true
run_tests = true
test_timeout = 300
check_coverage = true
min_coverage = 80.0
check_complexity = true
max_complexity = 10
"#
        )
        .unwrap();

        // This function exits with status 0 on success, so we can't easily test it
        // But we can test the underlying validate_config function
        let config = load_config_from_file(&temp_file.path().to_path_buf()).unwrap();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_handle_show_config_toml() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[gates]
run_clippy = true
clippy_strict = false
run_tests = true
test_timeout = 120
check_coverage = true
min_coverage = 85.0
check_complexity = true
max_complexity = 8
"#
        )
        .unwrap();

        let result = handle_show_config(temp_file.path(), ConfigFormat::Toml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_show_config_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[gates]
run_clippy = true
clippy_strict = true
run_tests = true
test_timeout = 300
check_coverage = true
min_coverage = 80.0
check_complexity = true
max_complexity = 10
"#
        )
        .unwrap();

        let result = handle_show_config(temp_file.path(), ConfigFormat::Json);
        assert!(result.is_ok());
    }
}
