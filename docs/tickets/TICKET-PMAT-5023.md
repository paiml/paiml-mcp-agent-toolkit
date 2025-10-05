# TICKET-PMAT-5023: Quality Gate CLI Commands

**Status**: GREEN
**Priority**: P0
**Complexity**: 4
**Estimated Time**: 1.5 hours
**Dependencies**: TICKET-PMAT-5020 (gate executor)
**Sprint**: Sprint 18 - Quality Gate Automation

## Objective

Add CLI commands for running quality gates manually. This allows developers to run full quality checks locally before pushing, complementing the fast pre-commit hooks.

## Success Criteria

- [ ] `pmat quality-gates` command runs all gates
- [ ] `pmat quality-gates --config <path>` loads custom configuration
- [ ] `pmat quality-gates --report` generates markdown report
- [ ] `pmat quality-gates --json` outputs JSON results
- [ ] Exit code 0 on success, 1 on failure
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_quality_gates_command` - Basic command structure
- [ ] `test_config_loading` - Load .pmat-gates.toml
- [ ] `test_json_output` - JSON serialization
- [ ] `test_report_output` - Markdown generation
- [ ] `test_exit_codes` - Correct exit codes

### Integration Tests
- [ ] `integration_run_gates` - Execute on real project
- [ ] `integration_config_file` - Load from file

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: CLI Structure

```rust
// server/src/cli/commands/quality_gates.rs

use crate::quality::{execute_all_gates, format_report, GateConfig, QualityReport};
use clap::Parser;
use std::path::PathBuf;

/// Run quality gates on the current project
#[derive(Debug, Parser)]
pub struct QualityGatesCommand {
    /// Path to quality gate configuration file
    #[arg(long, default_value = ".pmat-gates.toml")]
    pub config: PathBuf,

    /// Generate markdown report
    #[arg(long)]
    pub report: bool,

    /// Output JSON format
    #[arg(long)]
    pub json: bool,

    /// Project directory
    #[arg(long, default_value = ".")]
    pub project_dir: PathBuf,
}

impl QualityGatesCommand {
    /// Execute quality gates command
    ///
    /// # Complexity
    /// - Time: O(n) where n is codebase size
    /// - Cyclomatic: 5
    pub fn execute(&self) -> anyhow::Result<()> {
        // Load configuration
        let config = if self.config.exists() {
            load_config_from_file(&self.config)?
        } else {
            GateConfig::default()
        };

        // Run quality gates
        let report = execute_all_gates(&config, &self.project_dir)?;

        // Output results
        if self.json {
            output_json(&report)?;
        } else if self.report {
            output_markdown(&report)?;
        } else {
            output_summary(&report)?;
        }

        // Exit with appropriate code
        if report.passed {
            Ok(())
        } else {
            std::process::exit(1);
        }
    }
}
```

### Phase 2: Configuration Loading

```rust
/// Load gate configuration from TOML file
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn load_config_from_file(path: &PathBuf) -> anyhow::Result<GateConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: GateConfigToml = toml::from_str(&content)?;
    Ok(config.into())
}

/// TOML configuration structure
#[derive(Debug, Deserialize)]
struct GateConfigToml {
    gates: GateConfigInner,
}

#[derive(Debug, Deserialize)]
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
```

### Phase 3: Output Formatters

```rust
/// Output JSON results
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn output_json(report: &QualityReport) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

/// Output markdown report
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 1
fn output_markdown(report: &QualityReport) -> anyhow::Result<()> {
    let markdown = format_report(report);
    println!("{}", markdown);
    Ok(())
}

/// Output summary to console
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 3
fn output_summary(report: &QualityReport) -> anyhow::Result<()> {
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

    println!(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!(
        "Total time: {:.2}s",
        report.total_duration.as_secs_f64()
    );

    Ok(())
}
```

### Phase 4: CLI Integration

```rust
// server/src/cli/mod.rs (add to existing CLI)

use quality_gates::QualityGatesCommand;

#[derive(Debug, Parser)]
pub enum Commands {
    // ... existing commands ...

    /// Run quality gates
    #[command(name = "quality-gates")]
    QualityGates(QualityGatesCommand),
}

// In handle_command()
Commands::QualityGates(cmd) => cmd.execute(),
```

### Phase 5: Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gates_command_default() {
        let cmd = QualityGatesCommand {
            config: PathBuf::from(".pmat-gates.toml"),
            report: false,
            json: false,
            project_dir: PathBuf::from("."),
        };

        assert_eq!(cmd.config, PathBuf::from(".pmat-gates.toml"));
        assert!(!cmd.report);
        assert!(!cmd.json);
    }

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
        use std::time::Duration;
        use crate::quality::GateResult;

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
        use std::time::Duration;
        use crate::quality::GateResult;

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
        use std::time::Duration;
        use crate::quality::GateResult;

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
    #[ignore] // Requires project setup
    fn integration_run_gates() {
        let cmd = QualityGatesCommand {
            config: PathBuf::from(".pmat-gates.toml"),
            report: false,
            json: false,
            project_dir: PathBuf::from("."),
        };

        // Should execute without panic
        // May fail if gates don't pass
        let _ = cmd.execute();
    }
}
```

## Complexity Analysis

Functions with complexity:
- `QualityGatesCommand::execute`: CC=5
- `load_config_from_file`: CC=2
- `output_json`: CC=1
- `output_markdown`: CC=1
- `output_summary`: CC=3

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Run quality gates with default config
pmat quality-gates

# Run with custom config
pmat quality-gates --config custom-gates.toml

# Generate markdown report
pmat quality-gates --report > quality-report.md

# Output JSON
pmat quality-gates --json > quality.json

# Run on specific directory
pmat quality-gates --project-dir ../my-project
```

## Files to Create/Modify

### New Files
- `server/src/cli/commands/quality_gates.rs` - CLI command implementation

### Modified Files
- `server/src/cli/commands/mod.rs` - Add quality_gates module
- `server/src/cli/mod.rs` - Add QualityGates command to enum

## Risk Assessment

**Low Risk:**
- Uses existing gate executor
- Standard CLI patterns
- Clear error messages

**Mitigation:**
- Comprehensive error handling
- Graceful degradation if config missing
- Integration tests

## Notes

This ticket provides manual quality gate execution:

**Use Cases:**
1. **Before Push**: Run full gates locally before pushing
2. **Debugging**: Investigate gate failures with detailed output
3. **CI Validation**: Test CI configuration locally
4. **Custom Config**: Experiment with different gate thresholds

**Integration:**
- Complements fast pre-commit hooks (PMAT-5021)
- Uses gate executor (PMAT-5020)
- Reads .pmat-gates.toml (generated by PMAT-5021)
- Same gates as GitHub Actions (PMAT-5022)

**Output Formats:**
- **Default**: Colored console summary
- **--report**: Markdown for documentation
- **--json**: Machine-readable for tooling

**TDD Cycle Duration**: Estimated 1.5 hours for RED → GREEN → REFACTOR
