//! Command Executor for AGENTS.md
//!
//! Safely executes commands from AGENTS.md with quality gate enforcement.

use super::{PathBuf, Command};
use anyhow::Result;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Command executor with safety checks
pub struct AgentsMdExecutor {
    /// Sandbox environment
    sandbox: SandboxEnvironment,

    /// Execution config
    config: ExecutorConfig,
}

/// Executor configuration
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Default timeout in seconds
    pub default_timeout: u64,

    /// Maximum output size in bytes
    pub max_output_size: usize,

    /// Allow network access
    pub allow_network: bool,

    /// Allowed commands whitelist
    pub allowed_commands: Vec<String>,

    /// Environment variables to set
    pub env_vars: Vec<(String, String)>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            default_timeout: 60,
            max_output_size: 10 * 1024 * 1024, // 10MB
            allow_network: true,
            allowed_commands: vec![
                "cargo".to_string(),
                "npm".to_string(),
                "make".to_string(),
                "git".to_string(),
                "python".to_string(),
                "node".to_string(),
            ],
            env_vars: Vec::new(),
        }
    }
}

/// Sandbox environment for command execution
#[derive(Debug, Clone)]
pub struct SandboxEnvironment {
    /// Working directory
    pub working_dir: PathBuf,

    /// Temporary directory for outputs
    pub temp_dir: PathBuf,

    /// Resource limits
    pub limits: ResourceLimits,
}

/// Resource limits for sandboxed execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Max CPU time in seconds
    pub cpu_time: u64,

    /// Max memory in bytes
    pub memory: usize,

    /// Max file size in bytes
    pub file_size: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_time: 300,                // 5 minutes
            memory: 1024 * 1024 * 1024,   // 1GB
            file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// Command execution output
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Exit code
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution duration
    pub duration: Duration,

    /// Whether command was killed due to timeout
    pub timed_out: bool,
}

/// Safety validation report
#[derive(Debug, Clone)]
pub struct SafetyReport {
    /// Whether command is safe
    pub safe: bool,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Specific risks identified
    pub risks: Vec<Risk>,

    /// Recommended mitigations
    pub mitigations: Vec<String>,
}

/// Risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Identified risk
#[derive(Debug, Clone)]
pub struct Risk {
    /// Risk type
    pub risk_type: RiskType,

    /// Description
    pub description: String,

    /// Severity
    pub severity: RiskLevel,
}

/// Types of risks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskType {
    FileSystem,
    Network,
    System,
    Resource,
    Unknown,
}

impl AgentsMdExecutor {
    /// Create new executor
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("agents_md_executor");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            sandbox: SandboxEnvironment {
                working_dir: std::env::current_dir()?,
                temp_dir,
                limits: ResourceLimits::default(),
            },
            config: ExecutorConfig::default(),
        })
    }

    /// Create with custom config
    pub fn with_config(config: ExecutorConfig) -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("agents_md_executor");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            sandbox: SandboxEnvironment {
                working_dir: std::env::current_dir()?,
                temp_dir,
                limits: ResourceLimits::default(),
            },
            config,
        })
    }

    /// Execute command with safety checks
    pub async fn execute_command(&self, cmd: &Command) -> Result<CommandOutput> {
        // Validate safety first
        let safety = self.validate_command(cmd)?;
        if !safety.safe && safety.risk_level >= RiskLevel::High {
            return Err(anyhow::anyhow!(
                "Command rejected due to high risk: {:?}",
                safety.risks
            ));
        }

        // Parse command
        let parts = shell_words::split(&cmd.command)?;
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        let program = &parts[0];
        let args = &parts[1..];

        // Build tokio command
        let mut tokio_cmd = TokioCommand::new(program);
        tokio_cmd.args(args);

        // Set working directory
        if let Some(ref dir) = cmd.working_dir {
            tokio_cmd.current_dir(dir);
        } else {
            tokio_cmd.current_dir(&self.sandbox.working_dir);
        }

        // Set environment variables
        for (key, value) in &cmd.env {
            tokio_cmd.env(key, value);
        }
        for (key, value) in &self.config.env_vars {
            tokio_cmd.env(key, value);
        }

        // Configure stdio
        tokio_cmd.stdout(Stdio::piped());
        tokio_cmd.stderr(Stdio::piped());
        tokio_cmd.stdin(Stdio::null());

        // Execute with timeout
        let timeout_duration =
            Duration::from_secs(cmd.timeout.unwrap_or(self.config.default_timeout));

        let start = std::time::Instant::now();

        let result = timeout(timeout_duration, async { tokio_cmd.output().await }).await;

        let duration = start.elapsed();

        match result {
            Ok(Ok(output)) => {
                // Truncate output if too large
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let stdout = if stdout.len() > self.config.max_output_size {
                    format!(
                        "{}... (truncated, {} bytes total)",
                        &stdout[..self.config.max_output_size],
                        stdout.len()
                    )
                } else {
                    stdout.to_string()
                };

                let stderr = if stderr.len() > self.config.max_output_size {
                    format!(
                        "{}... (truncated, {} bytes total)",
                        &stderr[..self.config.max_output_size],
                        stderr.len()
                    )
                } else {
                    stderr.to_string()
                };

                Ok(CommandOutput {
                    exit_code: output.status.code().unwrap_or(-1),
                    stdout,
                    stderr,
                    duration,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Command execution failed: {e}")),
            Err(_) => Ok(CommandOutput {
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Command timed out after {timeout_duration:?}"),
                duration,
                timed_out: true,
            }),
        }
    }

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

/// Quality report for command output
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Whether all quality checks passed
    pub passed: bool,

    /// Individual quality checks
    pub checks: Vec<QualityCheck>,

    /// Quality violations found
    pub violations: Vec<QualityViolation>,
}

/// Individual quality check
#[derive(Debug, Clone)]
pub struct QualityCheck {
    /// Check name
    pub name: String,

    /// Whether check passed
    pub passed: bool,

    /// Check message
    pub message: String,
}

/// Quality violation
#[derive(Debug, Clone)]
pub struct QualityViolation {
    /// Violation type
    pub violation_type: ViolationType,

    /// Violation message
    pub message: String,

    /// Severity
    pub severity: Severity,
}

/// Types of violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationType {
    Error,
    Warning,
    Timeout,
    ResourceLimit,
    SecurityRisk,
}

/// Severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_command() {
        let executor = AgentsMdExecutor::new().unwrap();

        let cmd = Command {
            name: "Build".to_string(),
            command: "cargo build".to_string(),
            working_dir: None,
            env: Vec::new(),
            timeout: Some(60),
            safe: true,
        };

        let report = executor.validate_command(&cmd).unwrap();
        assert!(report.safe);
        assert_eq!(report.risk_level, RiskLevel::None);
        assert!(report.risks.is_empty());
    }

    #[test]
    fn test_validate_dangerous_command() {
        let executor = AgentsMdExecutor::new().unwrap();

        let cmd = Command {
            name: "Dangerous".to_string(),
            command: "sudo rm -rf /".to_string(),
            working_dir: None,
            env: Vec::new(),
            timeout: Some(60),
            safe: false,
        };

        let report = executor.validate_command(&cmd).unwrap();
        assert!(!report.safe);
        assert_eq!(report.risk_level, RiskLevel::Critical);
        assert!(!report.risks.is_empty());
        assert!(!report.mitigations.is_empty());
    }

    #[tokio::test]
    async fn test_execute_simple_command() {
        let executor = AgentsMdExecutor::new().unwrap();

        let cmd = Command {
            name: "Echo".to_string(),
            command: "echo hello".to_string(),
            working_dir: None,
            env: Vec::new(),
            timeout: Some(5),
            safe: true,
        };

        let output = executor.execute_command(&cmd).await.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("hello"));
        assert!(!output.timed_out);
    }

    #[test]
    fn test_quality_gate_success() {
        let executor = AgentsMdExecutor::new().unwrap();

        let output = CommandOutput {
            exit_code: 0,
            stdout: "Success".to_string(),
            stderr: String::new(),
            duration: Duration::from_secs(1),
            timed_out: false,
        };

        let report = executor.apply_quality_gates(&output).unwrap();
        assert!(report.passed);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_quality_gate_failure() {
        let executor = AgentsMdExecutor::new().unwrap();

        let output = CommandOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "error: compilation failed".to_string(),
            duration: Duration::from_secs(1),
            timed_out: false,
        };

        let report = executor.apply_quality_gates(&output).unwrap();
        assert!(!report.passed);
        assert!(!report.violations.is_empty());
    }
}
