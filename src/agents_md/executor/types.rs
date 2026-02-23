#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the command executor.
//!
//! Contains all structs and enums used by the executor subsystem.

use std::path::PathBuf;
use std::time::Duration;

/// Command executor with safety checks
pub struct AgentsMdExecutor {
    /// Sandbox environment
    pub(super) sandbox: SandboxEnvironment,

    /// Execution config
    pub(super) config: ExecutorConfig,
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
