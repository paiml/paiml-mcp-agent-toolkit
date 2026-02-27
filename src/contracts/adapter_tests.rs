// Tests for contract adapter
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    // BackwardCompatibility::map_json_params tests

    mod map_json_params_tests {
        use super::*;

        #[test]
        fn test_maps_project_path_to_path() {
            let params = serde_json::json!({
                "project_path": "/some/path",
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params(params);

            assert!(result.get("path").is_some());
            assert!(result.get("project_path").is_none());
            assert_eq!(result["path"], "/some/path");
        }

        #[test]
        fn test_project_path_does_not_override_existing_path() {
            let params = serde_json::json!({
                "project_path": "/old/path",
                "path": "/new/path",
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params(params);

            // Existing path should be preserved
            assert_eq!(result["path"], "/new/path");
            assert!(result.get("project_path").is_none());
        }

        #[test]
        fn test_does_not_convert_file_to_files_in_generic_mapping() {
            // In the generic map_json_params, file should NOT be converted to files
            // because some tools (refactor_auto, lint_hotspot, entropy) use file directly
            let params = serde_json::json!({
                "file": "test.rs",
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params(params);

            // file should remain as file (not converted to files)
            assert!(result.get("file").is_some());
            assert!(result.get("files").is_none());
        }

        #[test]
        fn test_format_normalization_human_to_summary() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "human"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "summary");
        }

        #[test]
        fn test_format_normalization_pretty_to_summary() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "pretty"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "summary");
        }

        #[test]
        fn test_format_normalization_summary_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "summary"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "summary");
        }

        #[test]
        fn test_format_normalization_json_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "json");
        }

        #[test]
        fn test_format_normalization_machine_to_json() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "machine"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "json");
        }

        #[test]
        fn test_format_normalization_yaml_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "yaml"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "yaml");
        }

        #[test]
        fn test_format_normalization_yml_to_yaml() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "yml"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "yaml");
        }

        #[test]
        fn test_format_normalization_markdown_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "markdown"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "markdown");
        }

        #[test]
        fn test_format_normalization_md_to_markdown() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "md"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "markdown");
        }

        #[test]
        fn test_format_normalization_csv_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "csv"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "csv");
        }

        #[test]
        fn test_format_normalization_tsv_to_csv() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "tsv"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "csv");
        }

        #[test]
        fn test_format_normalization_unknown_to_table() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "unknown_format"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "table");
        }

        #[test]
        fn test_non_object_input_returned_unchanged() {
            let params = serde_json::json!("string value");
            let result = BackwardCompatibility::map_json_params(params.clone());
            assert_eq!(result, params);
        }

        #[test]
        fn test_null_input_returned_unchanged() {
            let params = serde_json::json!(null);
            let result = BackwardCompatibility::map_json_params(params.clone());
            assert_eq!(result, params);
        }

        #[test]
        fn test_array_input_returned_unchanged() {
            let params = serde_json::json!([1, 2, 3]);
            let result = BackwardCompatibility::map_json_params(params.clone());
            assert_eq!(result, params);
        }

        #[test]
        fn test_number_input_returned_unchanged() {
            let params = serde_json::json!(42);
            let result = BackwardCompatibility::map_json_params(params.clone());
            assert_eq!(result, params);
        }

        #[test]
        fn test_boolean_input_returned_unchanged() {
            let params = serde_json::json!(true);
            let result = BackwardCompatibility::map_json_params(params.clone());
            assert_eq!(result, params);
        }

        #[test]
        fn test_no_format_key_does_not_add_one() {
            let params = serde_json::json!({
                "path": "/some/path",
                "other_key": "value"
            });

            let result = BackwardCompatibility::map_json_params(params);

            assert!(result.get("format").is_none());
            assert_eq!(result["path"], "/some/path");
        }

        #[test]
        fn test_format_not_string_remains_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": 123
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], 123);
        }

        #[test]
        fn test_format_as_object_remains_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": {"nested": "value"}
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"]["nested"], "value");
        }

        #[test]
        fn test_format_as_array_remains_unchanged() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": ["a", "b"]
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"][0], "a");
        }

        #[test]
        fn test_combined_project_path_and_format_migration() {
            let params = serde_json::json!({
                "project_path": "/old/path",
                "format": "human"
            });

            let result = BackwardCompatibility::map_json_params(params);

            assert_eq!(result["path"], "/old/path");
            assert!(result.get("project_path").is_none());
            assert_eq!(result["format"], "summary");
        }

        #[test]
        fn test_empty_object_returns_empty_object() {
            let params = serde_json::json!({});
            let result = BackwardCompatibility::map_json_params(params);
            assert!(result.as_object().unwrap().is_empty());
        }

        #[test]
        fn test_preserves_unknown_keys() {
            let params = serde_json::json!({
                "path": "/some/path",
                "custom_key": "custom_value",
                "another_key": 123,
                "nested": {"inner": "data"}
            });

            let result = BackwardCompatibility::map_json_params(params);

            assert_eq!(result["path"], "/some/path");
            assert_eq!(result["custom_key"], "custom_value");
            assert_eq!(result["another_key"], 123);
            assert_eq!(result["nested"]["inner"], "data");
        }

        #[test]
        fn test_idempotent_on_already_normalized_params() {
            let params = serde_json::json!({
                "path": "/some/path",
                "files": ["a.rs", "b.rs"],
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params(params.clone());
            let result2 = BackwardCompatibility::map_json_params(result.clone());

            assert_eq!(result, result2);
        }
    }

    // BackwardCompatibility::map_json_params_for_complexity tests

    mod map_json_params_for_complexity_tests {
        use super::*;

        #[test]
        fn test_converts_file_to_files_array() {
            let params = serde_json::json!({
                "file": "test.rs",
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params_for_complexity(params);

            assert!(result.get("files").is_some());
            assert!(result.get("file").is_none());
            let files = result["files"].as_array().unwrap();
            assert_eq!(files.len(), 1);
            assert_eq!(files[0], "test.rs");
        }

        #[test]
        fn test_file_does_not_override_existing_files() {
            let params = serde_json::json!({
                "file": "test.rs",
                "files": ["existing.rs", "other.rs"],
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params_for_complexity(params);

            // Should keep existing files, not override
            let files = result["files"].as_array().unwrap();
            assert_eq!(files.len(), 2);
            assert_eq!(files[0], "existing.rs");
            assert_eq!(files[1], "other.rs");
        }

        #[test]
        fn test_maps_project_path_to_path() {
            let params = serde_json::json!({
                "project_path": "/old/path",
                "format": "json"
            });

            let result = BackwardCompatibility::map_json_params_for_complexity(params);

            assert_eq!(result["path"], "/old/path");
            assert!(result.get("project_path").is_none());
        }

        #[test]
        fn test_normalizes_format() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "human"
            });

            let result = BackwardCompatibility::map_json_params_for_complexity(params);
            assert_eq!(result["format"], "summary");
        }

        #[test]
        fn test_all_migrations_combined() {
            let params = serde_json::json!({
                "project_path": "/old/path",
                "file": "single.rs",
                "format": "pretty"
            });

            let result = BackwardCompatibility::map_json_params_for_complexity(params);

            assert_eq!(result["path"], "/old/path");
            assert!(result.get("project_path").is_none());
            assert_eq!(result["files"][0], "single.rs");
            assert!(result.get("file").is_none());
            assert_eq!(result["format"], "summary");
        }

        #[test]
        fn test_non_object_returns_unchanged() {
            let params = serde_json::json!([1, 2, 3]);
            let result = BackwardCompatibility::map_json_params_for_complexity(params.clone());
            assert_eq!(result, params);
        }
    }

    // ContractAdapter::deprecation_warnings tests

    mod deprecation_warnings_tests {
        use super::*;
        use crate::cli::ComplexityOutputFormat;

        #[test]
        fn test_complexity_with_project_path_no_warning() {
            // No deprecation warnings - silently accept both --path and --project-path
            let cmd = AnalyzeCommands::Complexity {
                path: PathBuf::from("."),
                project_path: Some(PathBuf::from("/deprecated/path")),
                file: None,
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Summary,
                output: None,
                max_cyclomatic: None,
                max_cognitive: None,
                include: vec![],
                watch: false,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                ml: false,
            };

            let warnings = ContractAdapter::deprecation_warnings(&cmd);
            assert!(warnings.is_empty());
        }

        #[test]
        fn test_complexity_without_project_path_no_warning() {
            let cmd = AnalyzeCommands::Complexity {
                path: PathBuf::from("."),
                project_path: None,
                file: None,
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Summary,
                output: None,
                max_cyclomatic: None,
                max_cognitive: None,
                include: vec![],
                watch: false,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                ml: false,
            };

            let warnings = ContractAdapter::deprecation_warnings(&cmd);
            assert!(warnings.is_empty());
        }

        #[test]
        fn test_satd_command_no_warnings() {
            use crate::cli::SatdOutputFormat;

            let cmd = AnalyzeCommands::Satd {
                path: PathBuf::from("."),
                format: SatdOutputFormat::Summary,
                severity: None,
                critical_only: false,
                include_tests: false,
                strict: false,
                evolution: false,
                days: 30,
                metrics: false,
                output: None,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                extended: false,
            };

            let warnings = ContractAdapter::deprecation_warnings(&cmd);
            assert!(warnings.is_empty());
        }

        #[test]
        fn test_dead_code_command_no_warnings() {
            use crate::cli::DeadCodeOutputFormat;

            let cmd = AnalyzeCommands::DeadCode {
                path: PathBuf::from("."),
                format: DeadCodeOutputFormat::Summary,
                top_files: Some(10),
                include_unreachable: false,
                min_dead_lines: 10,
                include_tests: false,
                output: None,
                fail_on_violation: false,
                max_percentage: 15.0,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                max_depth: 8,
            };

            let warnings = ContractAdapter::deprecation_warnings(&cmd);
            assert!(warnings.is_empty());
        }

        #[test]
        fn test_tdg_command_no_warnings() {
            use crate::cli::TdgOutputFormat;

            let cmd = AnalyzeCommands::Tdg {
                path: PathBuf::from("."),
                threshold: 1.5,
                top_files: 10,
                format: TdgOutputFormat::Table,
                include_components: false,
                output: None,
                critical_only: false,
                verbose: false,
                ml: false,
            };

            let warnings = ContractAdapter::deprecation_warnings(&cmd);
            assert!(warnings.is_empty());
        }

        #[test]
        fn test_churn_command_no_warnings() {
            use crate::models::churn::ChurnOutputFormat;

            let cmd = AnalyzeCommands::Churn {
                path: PathBuf::from("."),
                project_path: Some(PathBuf::from(".")),
                days: 30,
                format: ChurnOutputFormat::Summary,
                output: None,
                top_files: 10,
                include: vec![],
                exclude: vec![],
            };

            let warnings = ContractAdapter::deprecation_warnings(&cmd);
            assert!(warnings.is_empty());
        }
    }

    // ContractAdapter::from_cli tests

    mod from_cli_tests {
        use super::*;
        use crate::cli::{
            ComplexityOutputFormat, DeadCodeOutputFormat, LintHotspotOutputFormat,
            SatdOutputFormat, TdgOutputFormat,
        };
        use crate::models::churn::ChurnOutputFormat;
        use tempfile::TempDir;

        fn create_temp_dir() -> TempDir {
            tempfile::tempdir().expect("Failed to create temp dir")
        }

        // ----- Complexity command tests -----

        #[test]
        fn test_complexity_with_valid_path_succeeds() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::Complexity {
                path: temp_dir.path().to_path_buf(),
                project_path: None,
                file: None,
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Summary,
                output: None,
                max_cyclomatic: Some(20),
                max_cognitive: Some(15),
                include: vec![],
                watch: false,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_complexity_with_deprecated_project_path_succeeds() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::Complexity {
                path: PathBuf::from("."),
                project_path: Some(temp_dir.path().to_path_buf()),
                file: None,
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Summary,
                output: None,
                max_cyclomatic: None,
                max_cognitive: None,
                include: vec![],
                watch: false,
                top_files: 5,
                fail_on_violation: false,
                timeout: 120,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_complexity_with_invalid_path_fails() {
            let cmd = AnalyzeCommands::Complexity {
                path: PathBuf::from("/nonexistent/path/that/does/not/exist"),
                project_path: None,
                file: None,
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Summary,
                output: None,
                max_cyclomatic: None,
                max_cognitive: None,
                include: vec![],
                watch: false,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_err());
        }

        #[test]
        fn test_complexity_with_output_file() {
            let temp_dir = create_temp_dir();
            let output_path = temp_dir.path().join("output.json");
            let cmd = AnalyzeCommands::Complexity {
                path: temp_dir.path().to_path_buf(),
                project_path: None,
                file: Some(PathBuf::from("test.rs")),
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Json,
                output: Some(output_path),
                max_cyclomatic: Some(25),
                max_cognitive: Some(20),
                include: vec![],
                watch: false,
                top_files: 20,
                fail_on_violation: true,
                timeout: 90,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_complexity_with_zero_top_files() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::Complexity {
                path: temp_dir.path().to_path_buf(),
                project_path: None,
                file: None,
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Summary,
                output: None,
                max_cyclomatic: None,
                max_cognitive: None,
                include: vec![],
                watch: false,
                top_files: 0,
                fail_on_violation: false,
                timeout: 60,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_complexity_with_max_thresholds() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::Complexity {
                path: temp_dir.path().to_path_buf(),
                project_path: None,
                file: None,
                files: vec![],
                toolchain: None,
                format: ComplexityOutputFormat::Summary,
                output: None,
                max_cyclomatic: Some(u16::MAX),
                max_cognitive: Some(u16::MAX),
                include: vec![],
                watch: false,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        // ----- SATD command tests -----

        #[test]
        fn test_satd_with_valid_path_succeeds() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::Satd {
                path: temp_dir.path().to_path_buf(),
                format: SatdOutputFormat::Summary,
                severity: None,
                critical_only: false,
                include_tests: false,
                strict: false,
                evolution: false,
                days: 30,
                metrics: false,
                output: None,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                extended: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_satd_with_all_options() {
            let temp_dir = create_temp_dir();
            let output_path = temp_dir.path().join("satd_output.json");
            let cmd = AnalyzeCommands::Satd {
                path: temp_dir.path().to_path_buf(),
                format: SatdOutputFormat::Json,
                severity: Some(crate::cli::SatdSeverity::High),
                critical_only: true,
                include_tests: true,
                strict: true,
                evolution: true,
                days: 60,
                metrics: true,
                output: Some(output_path),
                top_files: 20,
                fail_on_violation: true,
                timeout: 120,
                include: vec!["**/*.rs".to_string()],
                exclude: vec!["target/**".to_string()],
                extended: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_satd_with_invalid_path_fails() {
            let cmd = AnalyzeCommands::Satd {
                path: PathBuf::from("/nonexistent/path"),
                format: SatdOutputFormat::Summary,
                severity: None,
                critical_only: false,
                include_tests: false,
                strict: false,
                evolution: false,
                days: 30,
                metrics: false,
                output: None,
                top_files: 10,
                fail_on_violation: false,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                extended: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_err());
        }

        // ----- DeadCode command tests -----

        #[test]
        fn test_dead_code_with_valid_path_succeeds() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::DeadCode {
                path: temp_dir.path().to_path_buf(),
                format: DeadCodeOutputFormat::Summary,
                top_files: Some(10),
                include_unreachable: false,
                min_dead_lines: 10,
                include_tests: false,
                output: None,
                fail_on_violation: false,
                max_percentage: 15.0,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                max_depth: 8,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_dead_code_with_all_options() {
            let temp_dir = create_temp_dir();
            let output_path = temp_dir.path().join("dead_code_output.json");
            let cmd = AnalyzeCommands::DeadCode {
                path: temp_dir.path().to_path_buf(),
                format: DeadCodeOutputFormat::Json,
                top_files: Some(20),
                include_unreachable: true,
                min_dead_lines: 5,
                include_tests: true,
                output: Some(output_path),
                fail_on_violation: true,
                max_percentage: 10.0,
                timeout: 120,
                include: vec!["src/**".to_string()],
                exclude: vec!["tests/**".to_string()],
                max_depth: 10,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_dead_code_with_none_top_files() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::DeadCode {
                path: temp_dir.path().to_path_buf(),
                format: DeadCodeOutputFormat::Summary,
                top_files: None,
                include_unreachable: false,
                min_dead_lines: 10,
                include_tests: false,
                output: None,
                fail_on_violation: false,
                max_percentage: 15.0,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                max_depth: 8,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_dead_code_with_invalid_path_fails() {
            let cmd = AnalyzeCommands::DeadCode {
                path: PathBuf::from("/nonexistent/path"),
                format: DeadCodeOutputFormat::Summary,
                top_files: Some(10),
                include_unreachable: false,
                min_dead_lines: 10,
                include_tests: false,
                output: None,
                fail_on_violation: false,
                max_percentage: 15.0,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                max_depth: 8,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_err());
        }

        #[test]
        fn test_dead_code_boundary_max_percentage_100() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::DeadCode {
                path: temp_dir.path().to_path_buf(),
                format: DeadCodeOutputFormat::Summary,
                top_files: Some(10),
                include_unreachable: false,
                min_dead_lines: 10,
                include_tests: false,
                output: None,
                fail_on_violation: false,
                max_percentage: 100.0,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                max_depth: 8,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_dead_code_boundary_max_percentage_0() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::DeadCode {
                path: temp_dir.path().to_path_buf(),
                format: DeadCodeOutputFormat::Summary,
                top_files: Some(10),
                include_unreachable: false,
                min_dead_lines: 10,
                include_tests: false,
                output: None,
                fail_on_violation: false,
                max_percentage: 0.0,
                timeout: 60,
                include: vec![],
                exclude: vec![],
                max_depth: 8,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        // ----- TDG command tests -----

        #[test]
        fn test_tdg_with_valid_path_succeeds() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::Tdg {
                path: temp_dir.path().to_path_buf(),
                threshold: 1.5,
                top_files: 10,
                format: TdgOutputFormat::Table,
                include_components: false,
                output: None,
                critical_only: false,
                verbose: false,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_tdg_with_all_options() {
            let temp_dir = create_temp_dir();
            let output_path = temp_dir.path().join("tdg_output.json");
            let cmd = AnalyzeCommands::Tdg {
                path: temp_dir.path().to_path_buf(),
                threshold: 2.5,
                top_files: 20,
                format: TdgOutputFormat::Json,
                include_components: true,
                output: Some(output_path),
                critical_only: true,
                verbose: true,
                ml: true,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_tdg_with_invalid_path_fails() {
            let cmd = AnalyzeCommands::Tdg {
                path: PathBuf::from("/nonexistent/path"),
                threshold: 1.5,
                top_files: 10,
                format: TdgOutputFormat::Table,
                include_components: false,
                output: None,
                critical_only: false,
                verbose: false,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_err());
        }

        #[test]
        fn test_tdg_boundary_zero_threshold() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::Tdg {
                path: temp_dir.path().to_path_buf(),
                threshold: 0.0,
                top_files: 10,
                format: TdgOutputFormat::Table,
                include_components: false,
                output: None,
                critical_only: false,
                verbose: false,
                ml: false,
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        // ----- LintHotspot command tests -----

        #[test]
        fn test_lint_hotspot_with_valid_path_succeeds() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::LintHotspot {
                path: PathBuf::from("."),
                project_path: Some(temp_dir.path().to_path_buf()),
                file: None,
                format: LintHotspotOutputFormat::Summary,
                max_density: 5.0,
                min_confidence: 0.8,
                enforce: false,
                dry_run: false,
                enforcement_metadata: false,
                output: None,
                perf: false,
                clippy_flags: "-W warnings".to_string(),
                top_files: 10,
                include: vec![],
                exclude: vec![],
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lint_hotspot_with_all_options() {
            let temp_dir = create_temp_dir();
            let test_file = temp_dir.path().join("test.rs");
            std::fs::write(&test_file, "fn main() {}").unwrap();
            let output_path = temp_dir.path().join("lint_output.json");

            let cmd = AnalyzeCommands::LintHotspot {
                path: PathBuf::from("."),
                project_path: Some(temp_dir.path().to_path_buf()),
                file: Some(test_file),
                format: LintHotspotOutputFormat::Json,
                max_density: 3.0,
                min_confidence: 0.9,
                enforce: true,
                dry_run: true,
                enforcement_metadata: true,
                output: Some(output_path),
                perf: true,
                clippy_flags: "-W clippy::pedantic".to_string(),
                top_files: 20,
                include: vec!["src/**".to_string()],
                exclude: vec!["tests/**".to_string()],
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lint_hotspot_with_invalid_path_fails() {
            let cmd = AnalyzeCommands::LintHotspot {
                path: PathBuf::from("."),
                project_path: Some(PathBuf::from("/nonexistent/path")),
                file: None,
                format: LintHotspotOutputFormat::Summary,
                max_density: 5.0,
                min_confidence: 0.8,
                enforce: false,
                dry_run: false,
                enforcement_metadata: false,
                output: None,
                perf: false,
                clippy_flags: "-W warnings".to_string(),
                top_files: 10,
                include: vec![],
                exclude: vec![],
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_err());
        }

        #[test]
        fn test_lint_hotspot_boundary_confidence_zero() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::LintHotspot {
                path: PathBuf::from("."),
                project_path: Some(temp_dir.path().to_path_buf()),
                file: None,
                format: LintHotspotOutputFormat::Summary,
                max_density: 5.0,
                min_confidence: 0.0,
                enforce: false,
                dry_run: false,
                enforcement_metadata: false,
                output: None,
                perf: false,
                clippy_flags: "-W warnings".to_string(),
                top_files: 10,
                include: vec![],
                exclude: vec![],
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lint_hotspot_boundary_confidence_one() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::LintHotspot {
                path: PathBuf::from("."),
                project_path: Some(temp_dir.path().to_path_buf()),
                file: None,
                format: LintHotspotOutputFormat::Summary,
                max_density: 5.0,
                min_confidence: 1.0,
                enforce: false,
                dry_run: false,
                enforcement_metadata: false,
                output: None,
                perf: false,
                clippy_flags: "-W warnings".to_string(),
                top_files: 10,
                include: vec![],
                exclude: vec![],
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lint_hotspot_boundary_zero_max_density() {
            let temp_dir = create_temp_dir();
            let cmd = AnalyzeCommands::LintHotspot {
                path: PathBuf::from("."),
                project_path: Some(temp_dir.path().to_path_buf()),
                file: None,
                format: LintHotspotOutputFormat::Summary,
                max_density: 0.0,
                min_confidence: 0.8,
                enforce: false,
                dry_run: false,
                enforcement_metadata: false,
                output: None,
                perf: false,
                clippy_flags: "-W warnings".to_string(),
                top_files: 10,
                include: vec![],
                exclude: vec![],
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_ok());
        }

        // ----- Unsupported command test -----

        #[test]
        fn test_unsupported_command_fails() {
            let cmd = AnalyzeCommands::Churn {
                path: PathBuf::from("."),
                project_path: Some(PathBuf::from(".")),
                days: 30,
                format: ChurnOutputFormat::Summary,
                output: None,
                top_files: 10,
                include: vec![],
                exclude: vec![],
            };

            let result = ContractAdapter::from_cli(&cmd);
            assert!(result.is_err());
            // Check error message using if let since Result<Box<dyn T>> doesn't implement unwrap_err properly
            if let Err(e) = result {
                let err_msg = e.to_string();
                assert!(err_msg.contains("not yet adapted"));
            }
        }
    }

    // normalize_format_string edge cases (tested via public interface)

    mod format_normalization_edge_cases {
        use super::*;

        #[test]
        fn test_empty_format_string_returns_table() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": ""
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "table");
        }

        #[test]
        fn test_whitespace_format_string_returns_table() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "   "
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "table");
        }

        #[test]
        fn test_case_sensitive_format_strings() {
            // The format normalization is case-sensitive
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "JSON"
            });

            let result = BackwardCompatibility::map_json_params(params);
            // "JSON" (uppercase) should not match "json", so falls through to "table"
            assert_eq!(result["format"], "table");
        }

        #[test]
        fn test_format_with_leading_trailing_spaces() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": " json "
            });

            let result = BackwardCompatibility::map_json_params(params);
            // Spaces are not trimmed, so it doesn't match
            assert_eq!(result["format"], "table");
        }

        #[test]
        fn test_mixed_case_format_returns_table() {
            let params = serde_json::json!({
                "path": "/some/path",
                "format": "Json"
            });

            let result = BackwardCompatibility::map_json_params(params);
            assert_eq!(result["format"], "table");
        }
    }

    // Property-based tests

    mod property_tests {
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_map_json_params_never_panics(s in ".*") {
                // Property: map_json_params should never panic on any input
                let params = serde_json::json!(s);
                let _ = super::BackwardCompatibility::map_json_params(params);
            }

            #[test]
            fn test_map_json_params_preserves_value_types(
                int_val in any::<i64>(),
                float_val in any::<f64>().prop_filter("must be finite", |f| f.is_finite()),
                bool_val in any::<bool>()
            ) {
                let params = serde_json::json!({
                    "int_key": int_val,
                    "float_key": float_val,
                    "bool_key": bool_val
                });

                let result = super::BackwardCompatibility::map_json_params(params);

                // Use as_i64() and as_bool() for comparison to avoid move errors
                prop_assert_eq!(result["int_key"].as_i64(), Some(int_val));
                prop_assert_eq!(result["bool_key"].as_bool(), Some(bool_val));
                // Float comparison with tolerance
                if let Some(r) = result["float_key"].as_f64() {
                    prop_assert!((r - float_val).abs() < f64::EPSILON || (r.is_nan() && float_val.is_nan()));
                }
            }

            #[test]
            fn test_idempotent_map_json_params(path in "[a-z/]+") {
                let params = serde_json::json!({
                    "path": path,
                    "format": "json"
                });

                let result1 = super::BackwardCompatibility::map_json_params(params);
                let result2 = super::BackwardCompatibility::map_json_params(result1.clone());

                prop_assert_eq!(result1, result2);
            }
        }
    }
}

/// NOTE: Temporarily disabled due to private type access issues
#[cfg(all(test, feature = "broken-tests"))]
#[path = "adapter_coverage_tests.rs"]
mod coverage_tests;
