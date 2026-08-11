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
///
/// GH #716: this substring-matched the *user-supplied* path string
/// (`path_str.contains("/tests/")`), so the verdict depended on nothing but how
/// the caller spelled the path relative to their cwd. The identical bytes came
/// back two ways from one binary:
/// `pmat tdg <abs>/tcorp/tests/m00.rs` printed "Skipping test file" and no
/// score, while `cd tcorp/tests && pmat tdg m00.rs` scored it 92.58/A — the
/// relative spelling has no `/tests/` in it. Resolve the path first, then match
/// whole path COMPONENTS, so the answer is a property of the file and not of
/// the shell that asked.
fn should_skip_path(config: &TdgCommandConfig) -> bool {
    if !config.path.is_file() {
        return false;
    }
    is_test_or_bench_path(&config.path)
}

/// Whether `path` lives under a directory named `tests` or `benches`, decided
/// on the RESOLVED path so `./m00.rs` and `/abs/tcorp/tests/m00.rs` agree.
///
/// `canonicalize` needs the file to exist; `std::path::absolute` is the
/// lexical fallback so a nonexistent path still gets a cwd-independent answer
/// instead of silently reverting to the substring behaviour.
fn is_test_or_bench_path(path: &Path) -> bool {
    use std::path::Component;

    let resolved = fs::canonicalize(path)
        .or_else(|_| std::path::absolute(path))
        .unwrap_or_else(|_| path.to_path_buf());

    // Only the DIRECTORIES above the file count — a file named `tests.rs` is
    // source, exactly as the old `/tests/` substring required.
    resolved
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| {
            matches!(component, Component::Normal(name) if name == "tests" || name == "benches")
        })
}

/// The output for a path pmat declines to score.
///
/// GH #716: the skip was announced with a bare `println!`, so
/// `pmat tdg <test file> --format json` wrote `Skipping test file: …` — not
/// JSON — and exited 0, leaving every machine consumer to parse a sentence. A
/// declared machine format must produce that format, and "not scored" has to be
/// said explicitly (`analyzed: false`, null score) rather than implied by a
/// missing key or faked with a 0.0/F.
fn skipped_output(config: &TdgCommandConfig) -> Result<String> {
    let reason = "test-or-bench file: TDG does not grade test sources";
    match config.format {
        TdgOutputFormat::Json => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "file": config.path.display().to_string(),
            "analyzed": false,
            "skipped": true,
            "skip_reason": reason,
            "score": serde_json::Value::Null,
            "grade": serde_json::Value::Null,
            "not_measured": ["score", "grade"],
        }))?),
        // A SARIF consumer must get SARIF, not a sentence and not the JSON
        // object above: an analysis that produced no finding is an empty run,
        // and `properties` says why it is empty rather than leaving it to look
        // like a clean bill of health.
        TdgOutputFormat::Sarif => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": { "driver": {
                    "name": "pmat-tdg",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                }},
                "properties": {
                    "analyzed": false,
                    "skip_reason": reason,
                    "not_measured": ["score", "grade"],
                },
                "results": []
            }]
        }))?),
        TdgOutputFormat::Table | TdgOutputFormat::Markdown => {
            Ok(format!("Skipping test file: {}", config.path.display()))
        }
    }
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
            // GH #716: was a bare `println!`, which put a sentence on stdout
            // under `--format json`/`--format sarif` and ignored `--output`.
            let output_str = skipped_output(&config)?;
            formatting::write_tdg_output(&output_str, &config)?;
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

// GH #716 regression tests live in their own file (CB-040 file health).
#[cfg(test)]
#[path = "skip_path_tests.rs"]
mod skip_path_tests;
