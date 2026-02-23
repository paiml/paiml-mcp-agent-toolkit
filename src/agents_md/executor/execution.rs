#![cfg_attr(coverage_nightly, coverage(off))]
//! Command execution logic.
//!
//! Implements the core command execution with timeout, sandbox, and output handling.

use super::types::*;
use super::Command;
use anyhow::Result;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

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
                        stdout.get(..self.config.max_output_size).unwrap_or(&stdout),
                        stdout.len()
                    )
                } else {
                    stdout.to_string()
                };

                let stderr = if stderr.len() > self.config.max_output_size {
                    format!(
                        "{}... (truncated, {} bytes total)",
                        stderr.get(..self.config.max_output_size).unwrap_or(&stderr),
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
}
