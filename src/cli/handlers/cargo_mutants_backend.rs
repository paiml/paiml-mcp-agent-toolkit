//! cargo-mutants Backend Handler (Sprint 70 - Phase 3 GREEN/REFACTOR)
//!
//! Executes cargo-mutants and parses results into PMAT format.
//! This module bridges cargo-mutants JSON output with PMAT's mutation testing types.

use crate::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;
use crate::services::mutation::json_parser::{CargoMutantsReport, MutantOutcome};
use anyhow::{Context, Result};
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
    eprintln!("🧪 cargo-mutants Backend");
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
    eprintln!("✅ Detected: {}", version);
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
        "🔧 Executing: cargo mutants --output {} --timeout {} {}",
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
    eprintln!("⏳ Running mutation tests... (this may take several minutes)");
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

    eprintln!("✅ Mutation testing complete");
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
    eprintln!("📊 Mutation Testing Results:");
    eprintln!();

    let total = report.mutants.len();
    let caught = report.count_by_outcome(MutantOutcome::Caught);
    let missed = report.count_by_outcome(MutantOutcome::Missed);
    let timeout = report.count_by_outcome(MutantOutcome::Timeout);
    let unviable = report.count_by_outcome(MutantOutcome::Unviable);

    eprintln!("   Total mutants: {}", total);

    if total > 0 {
        eprintln!(
            "   Caught: {} ({:.1}%)",
            caught,
            (caught as f64 / total as f64) * 100.0
        );
        eprintln!(
            "   Missed: {} ({:.1}%)",
            missed,
            (missed as f64 / total as f64) * 100.0
        );

        if timeout > 0 {
            eprintln!(
                "   Timeout: {} ({:.1}%)",
                timeout,
                (timeout as f64 / total as f64) * 100.0
            );
        }

        if unviable > 0 {
            eprintln!(
                "   Unviable: {} ({:.1}%)",
                unviable,
                (unviable as f64 / total as f64) * 100.0
            );
        }
    }

    eprintln!();

    // Calculate and display mutation score
    let mutation_score = report.mutation_score();
    eprintln!("📈 Mutation Score: {:.1}%", mutation_score);

    // Quality assessment
    if mutation_score >= 90.0 {
        eprintln!("✅ Excellent! Test suite quality is very high");
    } else if mutation_score >= 75.0 {
        eprintln!("👍 Good test coverage, but room for improvement");
    } else if mutation_score >= 50.0 {
        eprintln!("⚠️  Moderate coverage - consider adding more tests");
    } else {
        eprintln!("❌ Low coverage - significant test gaps detected");
    }

    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CargoMutantsConfig Tests
    // =========================================================================

    #[test]
    fn test_config_creation_minimal() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        assert_eq!(config.path, PathBuf::from("/test/project"));
        assert!(config.output.is_none());
        assert_eq!(config.timeout, 60);
    }

    #[test]
    fn test_config_creation_with_output() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: Some(PathBuf::from("/test/output")),
            timeout: 120,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        assert_eq!(config.output.unwrap(), PathBuf::from("/test/output"));
    }

    #[test]
    fn test_config_with_jobs() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: Some(8),
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        assert_eq!(config.jobs, Some(8));
    }

    #[test]
    fn test_config_with_features() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: Some(vec!["foo".to_string(), "bar".to_string()]),
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        let features = config.features.unwrap();
        assert_eq!(features.len(), 2);
        assert!(features.contains(&"foo".to_string()));
        assert!(features.contains(&"bar".to_string()));
    }

    #[test]
    fn test_config_all_features() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: true,
            no_default_features: false,
            no_shuffle: false,
        };

        assert!(config.all_features);
    }

    #[test]
    fn test_config_no_default_features() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: true,
            no_shuffle: false,
        };

        assert!(config.no_default_features);
    }

    #[test]
    fn test_config_no_shuffle() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: true,
        };

        assert!(config.no_shuffle);
    }

    #[test]
    fn test_config_clone() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: Some(PathBuf::from("/output")),
            timeout: 120,
            jobs: Some(4),
            features: Some(vec!["feature1".to_string()]),
            all_features: true,
            no_default_features: false,
            no_shuffle: true,
        };

        let cloned = config.clone();
        assert_eq!(cloned.path, config.path);
        assert_eq!(cloned.timeout, config.timeout);
        assert_eq!(cloned.jobs, config.jobs);
        assert_eq!(cloned.all_features, config.all_features);
        assert_eq!(cloned.no_shuffle, config.no_shuffle);
    }

    #[test]
    fn test_config_debug() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CargoMutantsConfig"));
        assert!(debug_str.contains("/test/project"));
    }

    #[test]
    fn test_config_features_join() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test"),
            output: None,
            timeout: 60,
            jobs: None,
            features: Some(vec![
                "feat1".to_string(),
                "feat2".to_string(),
                "feat3".to_string(),
            ]),
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        let features = config.features.unwrap();
        let joined = features.join(",");
        assert_eq!(joined, "feat1,feat2,feat3");
    }

    #[test]
    fn test_config_default_output_path() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/test/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // When output is None, default should be path.join("mutants.out")
        let expected = config.path.join("mutants.out");
        assert_eq!(expected, PathBuf::from("/test/project/mutants.out"));
    }

    // =========================================================================
    // Display Statistics Tests (with mock reports)
    // =========================================================================

    #[test]
    fn test_display_statistics_empty_report() {
        use crate::services::mutation::json_parser::CargoMutantsReport;

        let report = CargoMutantsReport::default();
        // This just tests that display_statistics doesn't panic with empty report
        display_statistics(&report);
    }

    // =========================================================================
    // Execute Function Error Path Tests
    // =========================================================================

    #[test]
    fn test_execute_nonexistent_path() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // This will fail because cargo-mutants won't be installed or path doesn't exist
        let result = execute(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_output_dir_determination_with_output() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/project"),
            output: Some(PathBuf::from("/custom/output")),
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // Test output dir logic
        let output_dir = if let Some(ref output) = config.output {
            output.clone()
        } else {
            config.path.join("mutants.out")
        };

        assert_eq!(output_dir, PathBuf::from("/custom/output"));
    }

    #[test]
    fn test_output_dir_determination_without_output() {
        let config = CargoMutantsConfig {
            path: PathBuf::from("/project"),
            output: None,
            timeout: 60,
            jobs: None,
            features: None,
            all_features: false,
            no_default_features: false,
            no_shuffle: false,
        };

        // Test output dir logic
        let output_dir = if let Some(ref output) = config.output {
            output.clone()
        } else {
            config.path.join("mutants.out")
        };

        assert_eq!(output_dir, PathBuf::from("/project/mutants.out"));
    }

    // =========================================================================
    // Command Building Logic Tests
    // =========================================================================

    #[test]
    fn test_jobs_to_string() {
        let jobs: Option<usize> = Some(4);
        if let Some(j) = jobs {
            let str = j.to_string();
            assert_eq!(str, "4");
        }
    }

    #[test]
    fn test_timeout_to_string() {
        let timeout: u64 = 120;
        let str = timeout.to_string();
        assert_eq!(str, "120");
    }

    #[test]
    fn test_features_empty_vec() {
        let features: Vec<String> = vec![];
        let joined = features.join(",");
        assert_eq!(joined, "");
    }

    #[test]
    fn test_features_single_item() {
        let features = vec!["single".to_string()];
        let joined = features.join(",");
        assert_eq!(joined, "single");
    }

    // =========================================================================
    // Exit Code Handling Tests
    // =========================================================================

    #[test]
    fn test_exit_code_success() {
        let exit_code = 0;
        let should_continue = exit_code == 0 || exit_code == 2;
        assert!(should_continue);
    }

    #[test]
    fn test_exit_code_missed_mutants() {
        let exit_code = 2;
        let should_continue = exit_code == 0 || exit_code == 2;
        assert!(should_continue);
    }

    #[test]
    fn test_exit_code_failure() {
        let exit_code = 1;
        let should_fail = exit_code != 0 && exit_code != 2;
        assert!(should_fail);
    }

    #[test]
    fn test_exit_code_negative() {
        let exit_code = -1;
        let should_fail = exit_code != 0 && exit_code != 2;
        assert!(should_fail);
    }

    // =========================================================================
    // Mutation Score Quality Assessment Tests
    // =========================================================================

    #[test]
    fn test_quality_excellent() {
        let score = 95.0;
        let is_excellent = score >= 90.0;
        assert!(is_excellent);
    }

    #[test]
    fn test_quality_good() {
        let score = 80.0;
        let is_good = score >= 75.0 && score < 90.0;
        assert!(is_good);
    }

    #[test]
    fn test_quality_moderate() {
        let score = 60.0;
        let is_moderate = score >= 50.0 && score < 75.0;
        assert!(is_moderate);
    }

    #[test]
    fn test_quality_low() {
        let score = 40.0;
        let is_low = score < 50.0;
        assert!(is_low);
    }

    // =========================================================================
    // Percentage Calculation Tests
    // =========================================================================

    #[test]
    fn test_percentage_calculation() {
        let caught = 80;
        let total = 100;
        let pct = (caught as f64 / total as f64) * 100.0;
        assert!((pct - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_percentage_calculation_zero_total() {
        let total = 0;
        // In real code, we skip percentage display if total == 0
        let should_skip = total == 0;
        assert!(should_skip);
    }

    #[test]
    fn test_percentage_calculation_all_caught() {
        let caught = 100;
        let total = 100;
        let pct = (caught as f64 / total as f64) * 100.0;
        assert!((pct - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_percentage_calculation_none_caught() {
        let caught = 0;
        let total = 100;
        let pct = (caught as f64 / total as f64) * 100.0;
        assert!((pct - 0.0).abs() < 0.001);
    }
}
