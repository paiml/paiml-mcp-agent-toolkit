//! Quality gates command handler for TICKET-PMAT-5023 and TICKET-PMAT-5024
//!
//! Executes quality gates using the gate executor from TICKET-PMAT-5020.

use crate::cli::commands::{ConfigFormat, QualityGatesCommand};
use crate::quality::gates::{execute_all_gates, format_report, GateConfig, QualityReport};
use crate::quality::{generate_config_toml, generate_default_config, validate_config};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Handle quality-gates command with subcommands
///
/// # Complexity
/// - Time: O(1) for subcommands, O(n) for gate execution
/// - Cyclomatic: 4
pub async fn handle_quality_gates_command(
    command: Option<QualityGatesCommand>,
    config_path: PathBuf,
    report: bool,
    json: bool,
    project_dir: PathBuf,
) -> Result<()> {
    match command {
        Some(QualityGatesCommand::Init { force }) => {
            handle_init_config(&config_path, force)?;
        }
        Some(QualityGatesCommand::Validate) => {
            handle_validate_config(&config_path)?;
        }
        Some(QualityGatesCommand::Show { format }) => {
            handle_show_config(&config_path, format)?;
        }
        None => {
            // Original behavior: run gates
            run_quality_gates(config_path, report, json, project_dir).await?;
        }
    }
    Ok(())
}

/// Run quality gates (original functionality from TICKET-PMAT-5023)
///
/// # Complexity
/// - Time: O(n) where n is codebase size
/// - Cyclomatic: 5
async fn run_quality_gates(
    config_path: PathBuf,
    report: bool,
    json: bool,
    project_dir: PathBuf,
) -> Result<()> {
    // Load configuration
    let config = if config_path.exists() {
        load_config_from_file(&config_path)?
    } else {
        GateConfig::default()
    };

    // Run quality gates
    let gate_report = execute_all_gates(&config, &project_dir)?;

    // Output results
    if json {
        output_json(&gate_report)?;
    } else if report {
        output_markdown(&gate_report)?;
    } else {
        output_summary(&gate_report)?;
    }

    // Exit with appropriate code
    if gate_report.passed {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Initialize configuration file (TICKET-PMAT-5024)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn handle_init_config(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(anyhow::anyhow!(
            "Configuration file already exists. Use --force to overwrite."
        ));
    }

    let config = generate_default_config();
    std::fs::write(path, config)?;

    println!("✅ Created {}", path.display());
    Ok(())
}

/// Validate configuration file (TICKET-PMAT-5024)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn handle_validate_config(path: &Path) -> Result<()> {
    let config = load_config_from_file(&path.to_path_buf())?;

    match validate_config(&config) {
        Ok(()) => {
            println!("✅ Configuration is valid");
            Ok(())
        }
        Err(errors) => {
            println!("❌ Configuration has {} error(s):", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
            std::process::exit(1);
        }
    }
}

/// Show configuration (TICKET-PMAT-5024)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn handle_show_config(path: &Path, format: ConfigFormat) -> Result<()> {
    let config = load_config_from_file(&path.to_path_buf())?;

    match format {
        ConfigFormat::Toml => {
            let toml = generate_config_toml(&config);
            println!("{}", toml);
        }
        ConfigFormat::Json => {
            let json = serde_json::to_string_pretty(&config)?;
            println!("{}", json);
        }
        ConfigFormat::Env => {
            // Not applicable for quality gates config
            return Err(anyhow::anyhow!(
                "Environment variable format not supported for quality gates configuration"
            ));
        }
    }

    Ok(())
}

/// Load gate configuration from TOML file
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn load_config_from_file(path: &PathBuf) -> Result<GateConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: GateConfigToml = toml::from_str(&content)?;
    Ok(config.into())
}

/// TOML configuration structure
#[derive(Debug, serde::Deserialize)]
struct GateConfigToml {
    gates: GateConfigInner,
}

/// Inner gate configuration
#[derive(Debug, serde::Deserialize)]
struct GateConfigInner {
    run_clippy: bool,
    clippy_strict: bool,
    run_tests: bool,
    test_timeout: u64,
    check_coverage: bool,
    min_coverage: f64,
    check_complexity: bool,
    max_complexity: u32,
}

impl From<GateConfigToml> for GateConfig {
    fn from(toml: GateConfigToml) -> Self {
        let g = toml.gates;
        GateConfig {
            run_clippy: g.run_clippy,
            clippy_strict: g.clippy_strict,
            run_tests: g.run_tests,
            test_timeout: g.test_timeout,
            check_coverage: g.check_coverage,
            min_coverage: g.min_coverage,
            check_complexity: g.check_complexity,
            max_complexity: g.max_complexity,
        }
    }
}

/// Output JSON results
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn output_json(report: &QualityReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

/// Output markdown report
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 1
fn output_markdown(report: &QualityReport) -> Result<()> {
    let markdown = format_report(report);
    println!("{}", markdown);
    Ok(())
}

/// Output summary to console
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 3
fn output_summary(report: &QualityReport) -> Result<()> {
    println!("\n{} Quality Gate Results", if report.passed { "✅" } else { "❌" });
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for gate in &report.gates {
        let icon = if gate.passed { "✓" } else { "✗" };
        let color = if gate.passed { "\x1b[32m" } else { "\x1b[31m" };
        let reset = "\x1b[0m";

        println!(
            "{}{} {}{} ({:.2}s)",
            color,
            icon,
            gate.name,
            reset,
            gate.duration.as_secs_f64()
        );

        if !gate.passed && !gate.message.is_empty() {
            // Show first few lines of error
            for line in gate.message.lines().take(5) {
                println!("  {}", line);
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total time: {:.2}s", report.total_duration.as_secs_f64());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::gates::GateResult;

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
