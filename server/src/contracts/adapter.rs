//! Adapter layer to map existing inconsistent CLI/MCP parameters to uniform contracts
//! This is a TEMPORARY layer until we can refactor all commands to use uniform contracts

use super::*;
use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Adapter to handle current CLI inconsistencies
pub struct ContractAdapter;

impl ContractAdapter {
    /// Generate deprecation warnings for inconsistent parameters
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
                Self::map_complexity_command(
                    &analysis_path,
                    file,
                    files,
                    output,
                    max_cyclomatic,
                    max_cognitive,
                    top_files,
                    timeout,
                )
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
            } => Self::map_satd_command(
                path,
                critical_only,
                strict,
                include_tests,
                output,
                top_files,
                fail_on_violation,
                timeout,
            ),
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
            } => Self::map_dead_code_command(
                path,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
                fail_on_violation,
                max_percentage,
                timeout,
            ),
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
            } => Self::map_lint_hotspot_command(
                project_path,
                file,
                max_density,
                min_confidence,
                enforce,
                dry_run,
                output,
                top_files,
            ),
            _ => {
                anyhow::bail!("Command not yet adapted to uniform contract")
            }
        }
    }

    fn map_complexity_command(
        project_path: &Path,
        _file: &Option<PathBuf>,
        _files: &[PathBuf],
        output: &Option<PathBuf>,
        max_cyclomatic: &Option<u16>,
        max_cognitive: &Option<u16>,
        top_files: &usize,
        timeout: &u64,
    ) -> Result<Box<dyn ContractValidation>> {
        let path = project_path;

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: path.to_path_buf(),
                format: OutputFormat::Table,
                output: output.clone(),
                top_files: Some(*top_files),
                include_tests: false,
                timeout: *timeout,
            },
            max_cyclomatic: max_cyclomatic.map(|v| v as u32),
            max_cognitive: max_cognitive.map(|v| v as u32),
            max_halstead: None,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }

    fn map_satd_command(
        path: &Path,
        critical_only: &bool,
        strict: &bool,
        include_tests: &bool,
        output: &Option<PathBuf>,
        top_files: &usize,
        fail_on_violation: &bool,
        timeout: &u64,
    ) -> Result<Box<dyn ContractValidation>> {
        let contract = AnalyzeSatdContract {
            base: BaseAnalysisContract {
                path: path.to_path_buf(),
                format: OutputFormat::Summary,
                output: output.clone(),
                top_files: Some(*top_files),
                include_tests: *include_tests,
                timeout: *timeout,
            },
            severity: None,
            critical_only: *critical_only,
            strict: *strict,
            fail_on_violation: *fail_on_violation,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }

    fn map_dead_code_command(
        path: &Path,
        top_files: &Option<usize>,
        include_unreachable: &bool,
        min_dead_lines: &usize,
        include_tests: &bool,
        output: &Option<PathBuf>,
        fail_on_violation: &bool,
        max_percentage: &f64,
        timeout: &u64,
    ) -> Result<Box<dyn ContractValidation>> {
        let contract = AnalyzeDeadCodeContract {
            base: BaseAnalysisContract {
                path: path.to_path_buf(),
                format: OutputFormat::Summary,
                output: output.clone(),
                top_files: *top_files,
                include_tests: *include_tests,
                timeout: *timeout,
            },
            include_unreachable: *include_unreachable,
            min_dead_lines: *min_dead_lines,
            max_percentage: *max_percentage,
            fail_on_violation: *fail_on_violation,
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
        project_path: &Path,
        file: &Option<PathBuf>,
        max_density: &f64,
        min_confidence: &f64,
        enforce: &bool,
        dry_run: &bool,
        output: &Option<PathBuf>,
        top_files: &usize,
    ) -> Result<Box<dyn ContractValidation>> {
        let contract = AnalyzeLintHotspotContract {
            base: BaseAnalysisContract {
                path: project_path.to_path_buf(),
                format: OutputFormat::Summary,
                output: output.clone(),
                top_files: Some(*top_files),
                include_tests: false,
                timeout: 60,
            },
            file: file.clone(),
            max_density: *max_density,
            min_confidence: *min_confidence,
            enforce: *enforce,
            dry_run: *dry_run,
        };

        contract.validate()?;
        Ok(Box::new(contract))
    }
}

/// Backward compatibility mapping for old parameter names
pub struct BackwardCompatibility;

impl BackwardCompatibility {
    /// Map old parameter names to new ones in JSON
    pub fn map_json_params(mut params: serde_json::Value) -> serde_json::Value {
        if let Some(obj) = params.as_object_mut() {
            Self::map_project_path_to_path(obj);
            Self::map_file_to_files(obj);
            Self::map_format_types(obj);
        }
        params
    }

    fn map_project_path_to_path(obj: &mut serde_json::Map<String, serde_json::Value>) {
        if let Some(project_path) = obj.remove("project_path") {
            obj.insert("path".to_string(), project_path);
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
