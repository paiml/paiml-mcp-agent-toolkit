#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG (Technical Debt Gradient) command handlers
//!
//! Split into semantic submodules for file health compliance (CB-040).

mod baseline;
mod display;
mod explain;
mod formatting;
mod history;
mod quality_gates;
mod subcommands;
#[cfg(feature = "viz")]
mod viz;

use crate::cli::commands::TdgCommand;
use crate::cli::TdgOutputFormat;
use crate::tdg::{Grade, TdgAnalyzer, TdgConfig};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(feature = "git-lib"))]
use std::process::Command;

// Re-export submodule functions for test access via `use super::*`
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use display::{display_gate_result_table, format_explain_output, format_history_output};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use formatting::{
    format_comparison, format_tdg_output, format_tdg_score, write_tdg_output,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use quality_gates::{execute_tdg_analysis, validate_minimum_grade};

/// Discover git working directory from a path (shell fallback when git-lib disabled)
fn discover_git_workdir(path: &Path) -> Option<PathBuf> {
    #[cfg(feature = "git-lib")]
    {
        git2::Repository::discover(path)
            .ok()
            .and_then(|repo| repo.workdir().map(Path::to_path_buf))
    }
    #[cfg(not(feature = "git-lib"))]
    {
        Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
    }
}

/// Configuration for TDG command handling
pub struct TdgCommandConfig {
    pub path: PathBuf,
    pub command: Option<TdgCommand>,
    pub format: TdgOutputFormat,
    pub config: Option<PathBuf>,
    pub quiet: bool,
    pub include_components: bool,
    pub min_grade: Option<String>,
    pub output: Option<PathBuf>,
    /// Sprint 65: Include git context (commit SHA, branch, author)
    pub with_git_context: bool,
    /// Issue #78: Enable detailed explanation mode with function-level breakdown
    pub explain: bool,
    /// Issue #78: Complexity threshold for filtering functions in --explain mode
    pub threshold: u32,
    /// Issue #78: Baseline git ref for progress tracking in --explain mode
    pub baseline: Option<String>,
    /// Terminal graph visualization of dependencies
    pub viz: bool,
    /// Visualization theme
    pub viz_theme: String,
}

/// Check if path should be skipped (test/bench files)
fn should_skip_path(config: &TdgCommandConfig) -> bool {
    if !config.path.is_file() {
        return false;
    }
    let path_str = config.path.to_string_lossy();
    path_str.contains("/tests/") || path_str.contains("/benches/")
}

/// Setup git context for analyzer if enabled
fn setup_git_context(analyzer: &mut TdgAnalyzer, config: &TdgCommandConfig) {
    if !config.with_git_context {
        return;
    }
    let search_path = if config.path.is_file() {
        config.path.parent().unwrap_or(&config.path)
    } else {
        &config.path
    };
    let git_context = discover_git_workdir(search_path)
        .and_then(|workdir| crate::models::git_context::GitContext::try_from_current_dir(&workdir));
    analyzer.set_git_context(git_context);
}

/// Load TDG configuration from file or use default (cognitive complexity ≤3)
fn load_tdg_configuration(config: &TdgCommandConfig) -> Result<TdgConfig> {
    if let Some(config_path) = &config.config {
        let config_content = fs::read_to_string(config_path)?;
        Ok(toml::from_str(&config_content)?)
    } else {
        Ok(TdgConfig::default())
    }
}

/// Check if file is analyzable by extension
fn is_analyzable_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str(),
            Some(
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "java"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "go"
                    | "rb"
                    | "php"
                    | "swift"
                    | "kt"
                    | "kts"
            )
        )
    } else {
        false
    }
}

/// Parse grade string to Grade enum
fn parse_grade(s: &str) -> Result<crate::tdg::Grade> {
    use crate::tdg::Grade;
    match s.to_uppercase().as_str() {
        "A+" => Ok(Grade::APlus),
        "A" => Ok(Grade::A),
        "A-" => Ok(Grade::AMinus),
        "B+" => Ok(Grade::BPlus),
        "B" => Ok(Grade::B),
        "B-" => Ok(Grade::BMinus),
        "C+" => Ok(Grade::CPlus),
        "C" => Ok(Grade::C),
        "C-" => Ok(Grade::CMinus),
        "D" => Ok(Grade::D),
        "F" => Ok(Grade::F),
        _ => Err(anyhow::anyhow!(
            "Invalid grade: {s}. Valid grades: A+, A, A-, B+, B, B-, C+, C, C-, D, F"
        )),
    }
}

fn format_grade(grade: Grade) -> String {
    match grade {
        Grade::APlus => "A+",
        Grade::A => "A",
        Grade::AMinus => "A-",
        Grade::BPlus => "B+",
        Grade::B => "B",
        Grade::BMinus => "B-",
        Grade::CPlus => "C+",
        Grade::C => "C",
        Grade::CMinus => "C-",
        Grade::D => "D",
        Grade::F => "F",
    }
    .to_string()
}

/// Truncate string to max length with ellipsis, padded with spaces when shorter
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        batuta_common::display::truncate_str(s, max_len)
    }
}

/// Handle TDG command execution
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_tdg_command(config: TdgCommandConfig) -> Result<()> {
    if should_skip_path(&config) {
        if !config.quiet {
            println!("Skipping test file: {}", config.path.display());
        }
        return Ok(());
    }

    let tdg_config = load_tdg_configuration(&config)?;
    let mut analyzer = TdgAnalyzer::with_storage(tdg_config)?;
    setup_git_context(&mut analyzer, &config);

    if let Some(ref cmd) = config.command {
        return subcommands::handle_tdg_subcommand(cmd.clone(), &analyzer, &config).await;
    }

    if config.explain {
        return explain::handle_explain_mode(&analyzer, &config).await;
    }

    #[cfg(feature = "viz")]
    if config.viz {
        return viz::handle_viz_mode(&analyzer, &config).await;
    }

    // Issue #669, second round: ONE analysis, many renderers. SARIF used to
    // fork here into its own `analyze_project` call and reported a score no
    // other format agreed with.
    let analysis = quality_gates::run_tdg_analysis(&analyzer, &config).await?;
    quality_gates::validate_minimum_grade(&analysis.score, &config)?;

    let git_context = analyzer.get_git_context();
    let output_str = formatting::format_tdg_analysis(&analysis, git_context, &config)?;
    formatting::write_tdg_output(&output_str, &config)?;

    Ok(())
}

// Tests extracted to tdg_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "../tdg_handlers_tests.rs"]
mod tests;
