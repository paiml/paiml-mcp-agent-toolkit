//! Adapter layer to map existing inconsistent CLI/MCP parameters to uniform contracts
//!
//! This module provides backward compatibility by translating legacy parameter formats
//! to the current uniform contract system, ensuring seamless operation during API evolution.

use super::{
    AnalyzeComplexityContract, AnalyzeDeadCodeContract, AnalyzeLintHotspotContract,
    AnalyzeSatdContract, AnalyzeTdgContract, BaseAnalysisContract, ContractValidation,
    OutputFormat,
};
use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Adapter to handle current CLI inconsistencies
pub struct ContractAdapter;

/// Parameters for complexity command mapping
struct ComplexityMapParams<'a> {
    project_path: &'a Path,
    _file: &'a Option<PathBuf>,
    _files: &'a [PathBuf],
    output: &'a Option<PathBuf>,
    max_cyclomatic: &'a Option<u16>,
    max_cognitive: &'a Option<u16>,
    top_files: &'a usize,
    timeout: &'a u64,
}

/// Parameters for SATD command mapping
struct SatdMapParams<'a> {
    path: &'a Path,
    critical_only: &'a bool,
    strict: &'a bool,
    include_tests: &'a bool,
    output: &'a Option<PathBuf>,
    top_files: &'a usize,
    fail_on_violation: &'a bool,
    timeout: &'a u64,
}

/// Parameters for dead code command mapping
struct DeadCodeMapParams<'a> {
    path: &'a Path,
    top_files: &'a Option<usize>,
    include_unreachable: &'a bool,
    min_dead_lines: &'a usize,
    include_tests: &'a bool,
    output: &'a Option<PathBuf>,
    fail_on_violation: &'a bool,
    max_percentage: &'a f64,
    timeout: &'a u64,
}

/// Parameters for lint hotspot command mapping
struct LintHotspotMapParams<'a> {
    project_path: &'a Path,
    file: &'a Option<PathBuf>,
    max_density: &'a f64,
    min_confidence: &'a f64,
    enforce: &'a bool,
    dry_run: &'a bool,
    output: &'a Option<PathBuf>,
    top_files: &'a usize,
}

impl ContractAdapter {
    /// Generate deprecation warnings for inconsistent parameters
    #[must_use]
    pub fn deprecation_warnings(cmd: &AnalyzeCommands) -> Vec<String> {
        let mut warnings = Vec::new();

        if let AnalyzeCommands::Complexity {
            project_path: Some(_),
            ..
        } = cmd
        {
            warnings.push("Warning: --project-path is deprecated, use --path instead".to_string());
        }

        warnings
    }

    /// Map existing CLI analyze commands to uniform contracts
    pub fn from_cli(cmd: &AnalyzeCommands) -> Result<Box<dyn ContractValidation>> {
        match cmd {
            AnalyzeCommands::Complexity {
                path,
                project_path,
                file,
                files,
                output,
                max_cyclomatic,
                max_cognitive,
                top_files,
                timeout,
                ..
            } => {
                // Handle parameter migration: use new 'path' or deprecated 'project_path'
                let analysis_path = if let Some(deprecated_path) = project_path {
                    deprecated_path.clone()
                } else {
                    path.clone()
                };
                let params = ComplexityMapParams {
                    project_path: &analysis_path,
                    _file: file,
                    _files: files,
                    output,
                    max_cyclomatic,
                    max_cognitive,
                    top_files,
                    timeout,
                };
                Self::map_complexity_command(params)
            }
            AnalyzeCommands::Satd {
                path,
                critical_only,
                strict,
                include_tests,
                output,
                top_files,
                fail_on_violation,
                timeout,
                ..
            } => {
                let params = SatdMapParams {
                    path,
                    critical_only,
                    strict,
                    include_tests,
                    output,
                    top_files,
                    fail_on_violation,
                    timeout,
                };
                Self::map_satd_command(params)
            }
            AnalyzeCommands::DeadCode {
                path,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
                fail_on_violation,
                max_percentage,
                timeout,
                ..
            } => {
                let params = DeadCodeMapParams {
                    path,
                    top_files,
                    include_unreachable,
                    min_dead_lines,
                    include_tests,
                    output,
                    fail_on_violation,
                    max_percentage,
                    timeout,
                };
                Self::map_dead_code_command(params)
            }
            AnalyzeCommands::Tdg {
                path,
                threshold,
                top_files,
                include_components,
                output,
                critical_only,
                ..
            } => Self::map_tdg_command(
                path,
                threshold,
                top_files,
                include_components,
                output,
                critical_only,
            ),
            AnalyzeCommands::LintHotspot {
                project_path,
                file,
                max_density,
                min_confidence,
                enforce,
                dry_run,
                output,
                top_files,
                ..
            } => {
                let params = LintHotspotMapParams {
                    project_path,
                    file,
                    max_density,
                    min_confidence,
                    enforce,
                    dry_run,
                    output,
                    top_files,
                };
                Self::map_lint_hotspot_command(params)
            }
            _ => {
                anyhow::bail!("Command not yet adapted to uniform contract")
            }
        }
    }

    fn map_complexity_command(params: ComplexityMapParams) -> Result<Box<dyn ContractValidation>> {
        let path = params.project_path;

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: path.to_path_buf(),
                format: OutputFormat::Table,
                output: params.output.clone(),
                top_files: Some(*params.top_files),
                include_tests: false,
                timeout: *params.timeout,
            },
            max_cyclomatic: params.max_cyclomatic.map(u32::from),
            max_cognitive: params.max_cognitive.map(u32::from),
            max_halstead: None,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }

    fn map_satd_command(params: SatdMapParams) -> Result<Box<dyn ContractValidation>> {
        let contract = AnalyzeSatdContract {
            base: BaseAnalysisContract {
                path: params.path.to_path_buf(),
                format: OutputFormat::Summary,
                output: params.output.clone(),
                top_files: Some(*params.top_files),
                include_tests: *params.include_tests,
                timeout: *params.timeout,
            },
            severity: None,
            critical_only: *params.critical_only,
            strict: *params.strict,
            fail_on_violation: *params.fail_on_violation,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }

    fn map_dead_code_command(params: DeadCodeMapParams) -> Result<Box<dyn ContractValidation>> {
        let contract = AnalyzeDeadCodeContract {
            base: BaseAnalysisContract {
                path: params.path.to_path_buf(),
                format: OutputFormat::Summary,
                output: params.output.clone(),
                top_files: *params.top_files,
                include_tests: *params.include_tests,
                timeout: *params.timeout,
            },
            include_unreachable: *params.include_unreachable,
            min_dead_lines: *params.min_dead_lines,
            max_percentage: *params.max_percentage,
            fail_on_violation: *params.fail_on_violation,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }

    fn map_tdg_command(
        path: &Path,
        threshold: &f64,
        top_files: &usize,
        include_components: &bool,
        output: &Option<PathBuf>,
        critical_only: &bool,
    ) -> Result<Box<dyn ContractValidation>> {
        let contract = AnalyzeTdgContract {
            base: BaseAnalysisContract {
                path: path.to_path_buf(),
                format: OutputFormat::Table,
                output: output.clone(),
                top_files: Some(*top_files),
                include_tests: false,
                timeout: 60,
            },
            threshold: *threshold,
            include_components: *include_components,
            critical_only: *critical_only,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }

    fn map_lint_hotspot_command(
        params: LintHotspotMapParams,
    ) -> Result<Box<dyn ContractValidation>> {
        let contract = AnalyzeLintHotspotContract {
            base: BaseAnalysisContract {
                path: params.project_path.to_path_buf(),
                format: OutputFormat::Summary,
                output: params.output.clone(),
                top_files: Some(*params.top_files),
                include_tests: false,
                timeout: 60,
            },
            file: params.file.clone(),
            max_density: *params.max_density,
            min_confidence: *params.min_confidence,
            enforce: *params.enforce,
            dry_run: *params.dry_run,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }
}

/// Backward compatibility mapping for old parameter names
pub struct BackwardCompatibility;

impl BackwardCompatibility {
    /// Map old parameter names to new ones in JSON
    #[must_use]
    pub fn map_json_params(mut params: serde_json::Value) -> serde_json::Value {
        if let Some(obj) = params.as_object_mut() {
            Self::map_project_path_to_path(obj);
            // Note: We no longer convert file to files as some tools (refactor_auto, lint_hotspot, entropy)
            // use `file` as their own parameter, not as `files` array.
            Self::map_format_types(obj);
        }
        params
    }

    /// Map old parameter names to new ones in JSON for complexity analysis
    /// This variant converts `file` to `files` array for tools that expect an array
    #[must_use]
    pub fn map_json_params_for_complexity(mut params: serde_json::Value) -> serde_json::Value {
        if let Some(obj) = params.as_object_mut() {
            Self::map_project_path_to_path(obj);
            Self::map_file_to_files(obj);
            Self::map_format_types(obj);
        }
        params
    }

    fn map_project_path_to_path(obj: &mut serde_json::Map<String, serde_json::Value>) {
        if let Some(project_path) = obj.remove("project_path") {
            // Only set path if it's not already present
            if !obj.contains_key("path") {
                obj.insert("path".to_string(), project_path);
            }
        }
    }

    fn map_file_to_files(obj: &mut serde_json::Map<String, serde_json::Value>) {
        if let Some(file) = obj.remove("file") {
            if !obj.contains_key("files") {
                obj.insert("files".to_string(), serde_json::json!([file]));
            }
        }
    }

    fn map_format_types(obj: &mut serde_json::Map<String, serde_json::Value>) {
        if let Some(format) = obj.get_mut("format") {
            if let Some(fmt_str) = format.as_str() {
                let unified = Self::normalize_format_string(fmt_str);
                *format = serde_json::json!(unified);
            }
        }
    }

    fn normalize_format_string(fmt_str: &str) -> &'static str {
        match fmt_str {
            "human" | "pretty" | "summary" => "summary",
            "json" | "machine" => "json",
            "yaml" | "yml" => "yaml",
            "markdown" | "md" => "markdown",
            "csv" | "tsv" => "csv",
            _ => "table",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // BackwardCompatibility::map_json_params tests
    // ==========================================================================

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

    // ==========================================================================
    // BackwardCompatibility::map_json_params_for_complexity tests
    // ==========================================================================

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

    // ==========================================================================
    // ContractAdapter::deprecation_warnings tests
    // ==========================================================================

    mod deprecation_warnings_tests {
        use super::*;
        use crate::cli::ComplexityOutputFormat;

        #[test]
        fn test_complexity_with_project_path_emits_warning() {
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
            assert!(warnings[0].contains("use --path instead"));
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
                project_path: PathBuf::from("."),
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

    // ==========================================================================
    // ContractAdapter::from_cli tests
    // ==========================================================================

    mod from_cli_tests {
        use super::*;
        use crate::cli::{
            ComplexityOutputFormat, DeadCodeOutputFormat, LintHotspotOutputFormat, SatdOutputFormat,
            TdgOutputFormat,
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
        fn test_lint_hotspot_with_all_options() {
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
        fn test_lint_hotspot_with_invalid_path_fails() {
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

        // ----- Unsupported command test -----

        #[test]
        fn test_unsupported_command_fails() {
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
            // Check error message using if let since Result<Box<dyn T>> doesn't implement unwrap_err properly
            if let Err(e) = result {
                let err_msg = e.to_string();
                assert!(err_msg.contains("not yet adapted"));
            }
        }
    }

    // ==========================================================================
    // normalize_format_string edge cases (tested via public interface)
    // ==========================================================================

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

    // ==========================================================================
    // Property-based tests
    // ==========================================================================

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
mod coverage_tests {
    use super::*;
    use crate::cli::commands::{
        ChurnOutputFormat, ComplexityOutputFormat, DeadCodeOutputFormat, LintHotspotOutputFormat,
        SatdOutputFormat, TdgOutputFormat,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a temporary directory for tests
    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // ==========================================================================
    // ContractAdapter::deprecation_warnings tests
    // ==========================================================================

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

    // ==========================================================================
    // ContractAdapter::from_cli - Complexity command tests
    // ==========================================================================

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

    // ==========================================================================
    // ContractAdapter::from_cli - SATD command tests
    // ==========================================================================

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

    // ==========================================================================
    // ContractAdapter::from_cli - DeadCode command tests
    // ==========================================================================

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

    // ==========================================================================
    // ContractAdapter::from_cli - TDG command tests
    // ==========================================================================

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

    // ==========================================================================
    // ContractAdapter::from_cli - LintHotspot command tests
    // ==========================================================================

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

    // ==========================================================================
    // ContractAdapter::from_cli - Unsupported command test
    // ==========================================================================

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

    // ==========================================================================
    // BackwardCompatibility::map_json_params tests
    // ==========================================================================

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

    // ==========================================================================
    // Edge cases and boundary condition tests
    // ==========================================================================

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

    // ==========================================================================
    // Tests for internal mapping functions via public interface
    // ==========================================================================

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

    // ==========================================================================
    // Property-based tests for BackwardCompatibility
    // ==========================================================================

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
}
