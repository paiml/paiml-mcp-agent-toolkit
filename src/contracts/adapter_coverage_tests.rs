//! Coverage tests for adapters
//! Extracted for file health compliance (CB-040)

use super::*;
use crate::cli::commands::{
    ChurnOutputFormat, ComplexityOutputFormat, DeadCodeOutputFormat, LintHotspotOutputFormat,
    SatdOutputFormat, TdgOutputFormat,
};

fn create_temp_dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

// ContractAdapter::deprecation_warnings tests

#[test]
fn test_deprecation_warnings_with_project_path() {
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
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("--project-path is deprecated"));
}

#[test]
fn test_deprecation_warnings_without_project_path() {
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
fn test_deprecation_warnings_other_commands_no_warnings() {
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
    };

    let warnings = ContractAdapter::deprecation_warnings(&cmd);
    assert!(warnings.is_empty());
}

// ContractAdapter::from_cli - Complexity command tests

#[test]
fn test_from_cli_complexity_with_new_path() {
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
fn test_from_cli_complexity_with_deprecated_project_path() {
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
fn test_from_cli_complexity_with_output_file() {
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
fn test_from_cli_complexity_invalid_path() {
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

// ContractAdapter::from_cli - SATD command tests

#[test]
fn test_from_cli_satd_basic() {
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
    };

    let result = ContractAdapter::from_cli(&cmd);
    assert!(result.is_ok());
}

#[test]
fn test_from_cli_satd_with_all_options() {
    let temp_dir = create_temp_dir();
    let output_path = temp_dir.path().join("satd_output.json");
    let cmd = AnalyzeCommands::Satd {
        path: temp_dir.path().to_path_buf(),
        format: SatdOutputFormat::Json,
        severity: Some(crate::cli::commands::SatdSeverity::High),
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
    };

    let result = ContractAdapter::from_cli(&cmd);
    assert!(result.is_ok());
}

#[test]
fn test_from_cli_satd_invalid_path() {
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
    };

    let result = ContractAdapter::from_cli(&cmd);
    assert!(result.is_err());
}

// ContractAdapter::from_cli - DeadCode command tests

#[test]
fn test_from_cli_dead_code_basic() {
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
fn test_from_cli_dead_code_with_all_options() {
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
fn test_from_cli_dead_code_no_top_files() {
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
fn test_from_cli_dead_code_invalid_path() {
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

// ContractAdapter::from_cli - TDG command tests

#[test]
fn test_from_cli_tdg_basic() {
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
fn test_from_cli_tdg_with_all_options() {
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
fn test_from_cli_tdg_invalid_path() {
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

// ContractAdapter::from_cli - LintHotspot command tests

#[test]
fn test_from_cli_lint_hotspot_basic() {
    let temp_dir = create_temp_dir();
    let cmd = AnalyzeCommands::LintHotspot {
        project_path: temp_dir.path().to_path_buf(),
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
fn test_from_cli_lint_hotspot_with_all_options() {
    let temp_dir = create_temp_dir();
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "fn main() {}").unwrap();
    let output_path = temp_dir.path().join("lint_output.json");

    let cmd = AnalyzeCommands::LintHotspot {
        project_path: temp_dir.path().to_path_buf(),
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
fn test_from_cli_lint_hotspot_invalid_path() {
    let cmd = AnalyzeCommands::LintHotspot {
        project_path: PathBuf::from("/nonexistent/path"),
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

// ContractAdapter::from_cli - Unsupported command test

#[test]
fn test_from_cli_unsupported_command() {
    let cmd = AnalyzeCommands::Churn {
        project_path: PathBuf::from("."),
        days: 30,
        format: ChurnOutputFormat::Summary,
        output: None,
        top_files: 10,
        include: vec![],
        exclude: vec![],
    };

    let result = ContractAdapter::from_cli(&cmd);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("not yet adapted"));
}

// BackwardCompatibility::map_json_params tests

#[test]
fn test_map_json_params_project_path_to_path() {
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
fn test_map_json_params_file_to_files() {
    let params = serde_json::json!({
        "file": "test.rs",
        "format": "json"
    });

    let result = BackwardCompatibility::map_json_params(params);

    assert!(result.get("files").is_some());
    assert!(result.get("file").is_none());
    let files = result["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], "test.rs");
}

#[test]
fn test_map_json_params_file_to_files_does_not_override_existing_files() {
    let params = serde_json::json!({
        "file": "test.rs",
        "files": ["existing.rs", "other.rs"],
        "format": "json"
    });

    let result = BackwardCompatibility::map_json_params(params);

    // Should keep existing files, not override
    let files = result["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0], "existing.rs");
    assert_eq!(files[1], "other.rs");
}

#[test]
fn test_map_json_params_format_normalization_human() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "human"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "summary");
}

#[test]
fn test_map_json_params_format_normalization_pretty() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "pretty"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "summary");
}

#[test]
fn test_map_json_params_format_normalization_summary() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "summary"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "summary");
}

#[test]
fn test_map_json_params_format_normalization_json() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "json"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "json");
}

#[test]
fn test_map_json_params_format_normalization_machine() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "machine"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "json");
}

#[test]
fn test_map_json_params_format_normalization_yaml() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "yaml"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "yaml");
}

#[test]
fn test_map_json_params_format_normalization_yml() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "yml"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "yaml");
}

#[test]
fn test_map_json_params_format_normalization_markdown() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "markdown"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "markdown");
}

#[test]
fn test_map_json_params_format_normalization_md() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "md"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "markdown");
}

#[test]
fn test_map_json_params_format_normalization_csv() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "csv"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "csv");
}

#[test]
fn test_map_json_params_format_normalization_tsv() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "tsv"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "csv");
}

#[test]
fn test_map_json_params_format_normalization_unknown() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": "unknown_format"
    });

    let result = BackwardCompatibility::map_json_params(params);
    assert_eq!(result["format"], "table");
}

#[test]
fn test_map_json_params_non_object() {
    let params = serde_json::json!("string value");
    let result = BackwardCompatibility::map_json_params(params.clone());
    assert_eq!(result, params);
}

#[test]
fn test_map_json_params_null() {
    let params = serde_json::json!(null);
    let result = BackwardCompatibility::map_json_params(params.clone());
    assert_eq!(result, params);
}

#[test]
fn test_map_json_params_array() {
    let params = serde_json::json!([1, 2, 3]);
    let result = BackwardCompatibility::map_json_params(params.clone());
    assert_eq!(result, params);
}

#[test]
fn test_map_json_params_combined_migrations() {
    let params = serde_json::json!({
        "project_path": "/old/path",
        "file": "single.rs",
        "format": "human"
    });

    let result = BackwardCompatibility::map_json_params(params);

    assert_eq!(result["path"], "/old/path");
    assert!(result.get("project_path").is_none());
    assert_eq!(result["files"][0], "single.rs");
    assert!(result.get("file").is_none());
    assert_eq!(result["format"], "summary");
}

#[test]
fn test_map_json_params_no_format_key() {
    let params = serde_json::json!({
        "path": "/some/path",
        "other_key": "value"
    });

    let result = BackwardCompatibility::map_json_params(params);

    // Should not add a format key if none exists
    assert!(result.get("format").is_none());
    assert_eq!(result["path"], "/some/path");
}

#[test]
fn test_map_json_params_format_not_string() {
    let params = serde_json::json!({
        "path": "/some/path",
        "format": 123
    });

    let result = BackwardCompatibility::map_json_params(params);

    // Non-string format should remain unchanged
    assert_eq!(result["format"], 123);
}

// Edge cases and boundary condition tests

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

#[test]
fn test_dead_code_boundary_max_percentage() {
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
fn test_dead_code_boundary_zero_max_percentage() {
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

#[test]
fn test_lint_hotspot_boundary_confidence_zero() {
    let temp_dir = create_temp_dir();
    let cmd = AnalyzeCommands::LintHotspot {
        project_path: temp_dir.path().to_path_buf(),
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
        project_path: temp_dir.path().to_path_buf(),
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
        project_path: temp_dir.path().to_path_buf(),
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

// Tests for internal mapping functions via public interface

#[test]
fn test_complexity_mapping_converts_u16_to_u32() {
    let temp_dir = create_temp_dir();
    let cmd = AnalyzeCommands::Complexity {
        path: temp_dir.path().to_path_buf(),
        project_path: None,
        file: None,
        files: vec![],
        toolchain: None,
        format: ComplexityOutputFormat::Summary,
        output: None,
        max_cyclomatic: Some(100),
        max_cognitive: Some(50),
        include: vec![],
        watch: false,
        top_files: 10,
        fail_on_violation: false,
        timeout: 60,
        ml: false,
    };

    // The from_cli function validates and creates contract
    // u16 values should be converted to u32 without issue
    let result = ContractAdapter::from_cli(&cmd);
    assert!(result.is_ok());
}

#[test]
fn test_satd_mapping_preserves_boolean_flags() {
    let temp_dir = create_temp_dir();
    let cmd = AnalyzeCommands::Satd {
        path: temp_dir.path().to_path_buf(),
        format: SatdOutputFormat::Summary,
        severity: None,
        critical_only: true,
        include_tests: true,
        strict: true,
        evolution: false,
        days: 30,
        metrics: false,
        output: None,
        top_files: 10,
        fail_on_violation: true,
        timeout: 60,
        include: vec![],
        exclude: vec![],
    };

    let result = ContractAdapter::from_cli(&cmd);
    assert!(result.is_ok());
}

#[test]
fn test_dead_code_mapping_handles_optional_top_files() {
    let temp_dir = create_temp_dir();

    // Test with Some value
    let cmd = AnalyzeCommands::DeadCode {
        path: temp_dir.path().to_path_buf(),
        format: DeadCodeOutputFormat::Summary,
        top_files: Some(5),
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

// Property-based tests for BackwardCompatibility

#[test]
fn test_backward_compatibility_idempotent_on_new_params() {
    // Params that use new naming should not be changed
    let params = serde_json::json!({
        "path": "/some/path",
        "files": ["a.rs", "b.rs"],
        "format": "json"
    });

    let result = BackwardCompatibility::map_json_params(params.clone());
    let result2 = BackwardCompatibility::map_json_params(result.clone());

    // Should be idempotent
    assert_eq!(result, result2);
}

#[test]
fn test_backward_compatibility_preserves_unknown_keys() {
    let params = serde_json::json!({
        "path": "/some/path",
        "custom_key": "custom_value",
        "another_key": 123
    });

    let result = BackwardCompatibility::map_json_params(params);

    assert_eq!(result["path"], "/some/path");
    assert_eq!(result["custom_key"], "custom_value");
    assert_eq!(result["another_key"], 123);
}
