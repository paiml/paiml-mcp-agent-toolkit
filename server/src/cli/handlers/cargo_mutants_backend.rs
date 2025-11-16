//! cargo-mutants Backend Handler (Sprint 70 - Phase 3 GREEN/REFACTOR)
//!
//! Executes cargo-mutants and parses results into PMAT format.
//! This module bridges cargo-mutants JSON output with PMAT's mutation testing types.

use crate::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;
use crate::services::mutation::json_parser::{CargoMutantsReport, MutantOutcome};
use anyhow::{Context, Result};
use console::style;
use std::path::PathBuf;
use std::process::Command;

/// Configuration for cargo-mutants execution
#[derive(Debug, Clone)]
pub struct CargoMutantsConfig {
    pub path: PathBuf,
    pub output: Option<PathBuf>,
    pub timeout: u64,
    pub jobs: Option<usize>,
    pub features: Option<Vec<String>>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub no_shuffle: bool,
}

/// Execute cargo-mutants and return path to output directory
pub fn execute(config: CargoMutantsConfig) -> Result<PathBuf> {
    // 1. Detect and validate cargo-mutants installation
    eprintln!("{}", style("🧪 cargo-mutants Backend").bold());
    eprintln!();

    let wrapper = CargoMutantsWrapper::new().map_err(|e| {
        anyhow::anyhow!(
            "cargo-mutants not found. Install: cargo install cargo-mutants. Error: {}",
            e
        )
    })?;

    wrapper.validate_version()
        .map_err(|e| anyhow::anyhow!("cargo-mutants version too old. Minimum v24.7.0 required. Upgrade: cargo install --force cargo-mutants. Error: {}", e))?;

    let version = wrapper
        .version()
        .map_err(|e| anyhow::anyhow!("Failed to get cargo-mutants version: {}", e))?;
    eprintln!("{} {}", style("✅ Detected:").green(), version);
    eprintln!();

    // Determine output directory
    let output_dir = if let Some(ref output) = config.output {
        output.clone()
    } else {
        config.path.join("mutants.out")
    };

    // 2. Build cargo mutants command
    let mut cmd = Command::new("cargo");
    cmd.arg("mutants");
    cmd.arg("--output").arg(&output_dir);

    // Set working directory
    cmd.current_dir(&config.path);

    // Add timeout
    cmd.arg("--timeout").arg(config.timeout.to_string());

    // Add parallel jobs
    if let Some(j) = config.jobs {
        cmd.arg("--jobs").arg(j.to_string());
    }

    // Add features
    if config.all_features {
        cmd.arg("--all-features");
    } else if config.no_default_features {
        cmd.arg("--no-default-features");
        if let Some(ref feats) = config.features {
            cmd.arg("--features").arg(feats.join(","));
        }
    } else if let Some(ref feats) = config.features {
        cmd.arg("--features").arg(feats.join(","));
    }

    // Add no-shuffle flag
    if config.no_shuffle {
        cmd.arg("--no-shuffle");
    }

    // Display command being executed
    eprintln!(
        "{} cargo mutants --output {} --timeout {} {}",
        style("🔧 Executing:").cyan(),
        output_dir.display(),
        config.timeout,
        if let Some(j) = config.jobs {
            format!("--jobs {}", j)
        } else {
            String::new()
        }
    );
    eprintln!();

    // 3. Execute cargo-mutants
    eprintln!(
        "{}",
        style("⏳ Running mutation tests... (this may take several minutes)").yellow()
    );
    eprintln!();

    let output_result = cmd
        .output()
        .context("Failed to execute cargo mutants command")?;

    // cargo-mutants exit codes:
    // 0 - Success (all mutants caught)
    // 2 - Success with missed mutants (this is expected!)
    // Other - Actual failure
    let exit_code = output_result.status.code().unwrap_or(-1);
    if exit_code != 0 && exit_code != 2 {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        anyhow::bail!(
            "cargo-mutants execution failed with exit code {}:\n{}",
            exit_code,
            stderr
        );
    }

    eprintln!("{}", style("✅ Mutation testing complete").green());
    eprintln!();

    // cargo-mutants may create a nested directory structure
    // Check if outcomes.json exists, if not check nested location
    let actual_output = if output_dir.join("outcomes.json").exists() {
        output_dir
    } else if output_dir
        .join("mutants.out")
        .join("outcomes.json")
        .exists()
    {
        output_dir.join("mutants.out")
    } else {
        output_dir
    };

    Ok(actual_output)
}

/// Display mutation testing statistics
pub fn display_statistics(report: &CargoMutantsReport) {
    eprintln!("{}", style("📊 Mutation Testing Results:").bold());
    eprintln!();

    let total = report.mutants.len();
    let caught = report.count_by_outcome(MutantOutcome::Caught);
    let missed = report.count_by_outcome(MutantOutcome::Missed);
    let timeout = report.count_by_outcome(MutantOutcome::Timeout);
    let unviable = report.count_by_outcome(MutantOutcome::Unviable);

    eprintln!("   Total mutants: {}", total);

    if total > 0 {
        eprintln!(
            "   {} {} ({:.1}%)",
            style("Caught:").green(),
            caught,
            (caught as f64 / total as f64) * 100.0
        );
        eprintln!(
            "   {} {} ({:.1}%)",
            style("Missed:").red(),
            missed,
            (missed as f64 / total as f64) * 100.0
        );

        if timeout > 0 {
            eprintln!(
                "   {} {} ({:.1}%)",
                style("Timeout:").yellow(),
                timeout,
                (timeout as f64 / total as f64) * 100.0
            );
        }

        if unviable > 0 {
            eprintln!(
                "   {} {} ({:.1}%)",
                style("Unviable:").yellow(),
                unviable,
                (unviable as f64 / total as f64) * 100.0
            );
        }
    }

    eprintln!();

    // Calculate and display mutation score
    let mutation_score = report.mutation_score();
    let score_styled = if mutation_score >= 80.0 {
        style(format!("{:.1}%", mutation_score)).green().bold()
    } else if mutation_score >= 60.0 {
        style(format!("{:.1}%", mutation_score)).yellow().bold()
    } else {
        style(format!("{:.1}%", mutation_score)).red().bold()
    };

    eprintln!("{} {}", style("📈 Mutation Score:").bold(), score_styled);

    // Quality assessment
    if mutation_score >= 90.0 {
        eprintln!(
            "{}",
            style("✅ Excellent! Test suite quality is very high").green()
        );
    } else if mutation_score >= 75.0 {
        eprintln!(
            "{}",
            style("👍 Good test coverage, but room for improvement").green()
        );
    } else if mutation_score >= 50.0 {
        eprintln!(
            "{}",
            style("⚠️  Moderate coverage - consider adding more tests").yellow()
        );
    } else {
        eprintln!(
            "{}",
            style("❌ Low coverage - significant test gaps detected").red()
        );
    }

    eprintln!();
}
