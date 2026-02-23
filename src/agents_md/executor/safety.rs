#![cfg_attr(coverage_nightly, coverage(off))]
//! Safety validation and quality gate enforcement.
//!
//! Validates commands for dangerous patterns and applies quality gates to outputs.

use super::quality_types::*;
use super::types::*;
use super::Command;
use anyhow::Result;

impl AgentsMdExecutor {
    /// Validate command safety
    pub fn validate_command(&self, cmd: &Command) -> Result<SafetyReport> {
        let mut report = SafetyReport {
            safe: true,
            risk_level: RiskLevel::None,
            risks: Vec::new(),
            mitigations: Vec::new(),
        };

        // Check if command is in whitelist
        let parts = shell_words::split(&cmd.command)?;
        if !parts.is_empty() {
            let program = &parts[0];
            if !self
                .config
                .allowed_commands
                .iter()
                .any(|allowed| program == allowed)
            {
                report.risks.push(Risk {
                    risk_type: RiskType::System,
                    description: format!("Command '{program}' not in whitelist"),
                    severity: RiskLevel::Medium,
                });
                report.risk_level = RiskLevel::Medium;
            }
        }

        // Check for dangerous patterns
        let dangerous_patterns = [
            ("rm -rf", RiskType::FileSystem, RiskLevel::Critical),
            ("sudo", RiskType::System, RiskLevel::High),
            ("chmod 777", RiskType::FileSystem, RiskLevel::High),
            ("eval", RiskType::System, RiskLevel::High),
            ("> /dev/", RiskType::System, RiskLevel::Critical),
            ("curl | sh", RiskType::Network, RiskLevel::Critical),
            ("wget | bash", RiskType::Network, RiskLevel::Critical),
        ];

        for (pattern, risk_type, severity) in dangerous_patterns {
            if cmd.command.contains(pattern) {
                report.risks.push(Risk {
                    risk_type,
                    description: format!("Dangerous pattern detected: {pattern}"),
                    severity,
                });
                if severity > report.risk_level {
                    report.risk_level = severity;
                }
                report.safe = false;
            }
        }

        // Add mitigations
        if !report.safe {
            report
                .mitigations
                .push("Run in isolated container".to_string());
            report
                .mitigations
                .push("Review command manually before execution".to_string());
            if report.risk_level >= RiskLevel::High {
                report
                    .mitigations
                    .push("Consider alternative safer commands".to_string());
            }
        }

        Ok(report)
    }

    /// Apply quality gates to output
    pub fn apply_quality_gates(&self, output: &CommandOutput) -> Result<QualityReport> {
        let mut report = QualityReport {
            passed: true,
            checks: Vec::new(),
            violations: Vec::new(),
        };

        // Check exit code
        if output.exit_code != 0 {
            report.checks.push(QualityCheck {
                name: "Exit Code".to_string(),
                passed: false,
                message: format!("Command failed with exit code {}", output.exit_code),
            });
            report.passed = false;
        }

        // Check for error patterns in stderr
        let error_patterns = ["error:", "failed:", "fatal:", "panic:"];
        for pattern in error_patterns {
            if output.stderr.to_lowercase().contains(pattern) {
                report.violations.push(QualityViolation {
                    violation_type: ViolationType::Error,
                    message: format!("Error pattern '{pattern}' found in output"),
                    severity: Severity::High,
                });
                report.passed = false;
            }
        }

        // Check for timeout
        if output.timed_out {
            report.violations.push(QualityViolation {
                violation_type: ViolationType::Timeout,
                message: "Command execution timed out".to_string(),
                severity: Severity::Critical,
            });
            report.passed = false;
        }

        Ok(report)
    }
}
