//! CLI Test Runner - Core framework for CLI acceptance testing
//!
//! Provides a structured testing framework for CLI commands, arguments, and scenarios.
//! Implements the test execution patterns defined in cli-acceptance-testing.md specification.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};
use tempfile::{TempDir, tempdir};
use serde_json::Value;
use anyhow::{Result, Context};

/// Test execution result with performance metrics and validation data
#[derive(Debug, Clone)]
pub struct TestResult {
    pub output: Output,
    pub execution_time: Duration,
    pub exit_code: i32,
    pub stdout_text: String,
    pub stderr_text: String,
    pub command_line: String,
}

/// CLI test runner for executing and validating pmat commands
pub struct CliTestRunner {
    pub pmat_binary_path: PathBuf,
    pub test_workspace: TempDir,
    pub environment_vars: HashMap<String, String>,
}

impl CliTestRunner {
    /// Create a new CLI test runner with isolated test environment
    pub fn new() -> Result<Self> {
        let pmat_binary = locate_pmat_binary()?;
        let workspace = tempdir().context("Failed to create test workspace")?;
        
        Ok(Self {
            pmat_binary_path: pmat_binary,
            test_workspace: workspace,
            environment_vars: HashMap::new(),
        })
    }
    
    /// Execute a pmat command with arguments and capture results
    pub fn run_command<I, S>(&self, args: I) -> Result<TestResult>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let start_time = Instant::now();
        let args_vec: Vec<String> = args.into_iter()
            .map(|s| s.as_ref().to_string_lossy().to_string())
            .collect();
        
        let mut command = Command::new(&self.pmat_binary_path);
        command.args(&args_vec);
        command.current_dir(self.test_workspace.path());
        
        // Set environment variables
        for (key, value) in &self.environment_vars {
            command.env(key, value);
        }
        
        let output = command.output()
            .context("Failed to execute pmat command")?;
        
        let execution_time = start_time.elapsed();
        let exit_code = output.status.code().unwrap_or(-1);
        
        let stdout_text = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
        let command_line = format!("{} {}", 
            self.pmat_binary_path.display(), 
            args_vec.join(" ")
        );
        
        Ok(TestResult {
            output,
            execution_time,
            exit_code,
            stdout_text,
            stderr_text,
            command_line,
        })
    }
    
    /// Run command and expect success (exit code 0)
    pub fn run_success<I, S>(&self, args: I) -> Result<TestResult>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let result = self.run_command(args)?;
        if result.exit_code != 0 {
            anyhow::bail!(
                "Command failed with exit code {}: {}\nstderr: {}",
                result.exit_code,
                result.command_line,
                result.stderr_text
            );
        }
        Ok(result)
    }
    
    /// Run command and expect failure (non-zero exit code)
    pub fn run_failure<I, S>(&self, args: I) -> Result<TestResult>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let result = self.run_command(args)?;
        if result.exit_code == 0 {
            anyhow::bail!(
                "Command unexpectedly succeeded: {}\nstdout: {}",
                result.command_line,
                result.stdout_text
            );
        }
        Ok(result)
    }
    
    /// Set environment variable for test execution
    pub fn set_env<K, V>(&mut self, key: K, value: V) -> &mut Self 
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.environment_vars.insert(key.into(), value.into());
        self
    }
    
    /// Create a sample project structure for testing
    pub fn create_sample_project(&self) -> Result<PathBuf> {
        let project_path = self.test_workspace.path().join("sample_project");
        std::fs::create_dir_all(&project_path)?;
        
        // Create sample Rust project structure
        std::fs::create_dir_all(project_path.join("src"))?;
        std::fs::write(
            project_path.join("Cargo.toml"),
            r#"[package]
name = "sample-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
        )?;
        
        std::fs::write(
            project_path.join("src/main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#
        )?;
        
        std::fs::write(
            project_path.join("src/lib.rs"),
            r#"pub fn complex_function(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for item in data {
        if *item > 0 {
            if *item % 2 == 0 {
                result.push(*item * 2);
            } else {
                result.push(*item * 3);
            }
        } else if *item < 0 {
            result.push(item.abs());
        }
    }
    result.sort();
    result.dedup();
    result
}
"#
        )?;
        
        Ok(project_path)
    }
    
    /// Get test workspace path
    pub fn workspace_path(&self) -> &Path {
        self.test_workspace.path()
    }
}

/// Validation helpers for test results
pub struct TestValidators;

impl TestValidators {
    /// Validate that output contains expected JSON structure
    pub fn assert_json_output(result: &TestResult, expected_fields: &[&str]) -> Result<Value> {
        let json: Value = serde_json::from_str(&result.stdout_text)
            .context("Output is not valid JSON")?;
        
        for field in expected_fields {
            if !json.get(field).is_some() {
                anyhow::bail!("Missing expected field '{}' in JSON output", field);
            }
        }
        
        Ok(json)
    }
    
    /// Validate that stderr contains specific error message
    pub fn assert_error_message(result: &TestResult, expected_message: &str) -> Result<()> {
        if !result.stderr_text.contains(expected_message) {
            anyhow::bail!(
                "Expected error message '{}' not found in stderr: '{}'",
                expected_message,
                result.stderr_text
            );
        }
        Ok(())
    }
    
    /// Validate exit code matches expected value
    pub fn assert_exit_code(result: &TestResult, expected_code: i32) -> Result<()> {
        if result.exit_code != expected_code {
            anyhow::bail!(
                "Expected exit code {} but got {}: {}",
                expected_code,
                result.exit_code,
                result.stderr_text
            );
        }
        Ok(())
    }
    
    /// Validate execution time is within acceptable limits
    pub fn assert_performance(result: &TestResult, max_duration: Duration) -> Result<()> {
        if result.execution_time > max_duration {
            anyhow::bail!(
                "Command took too long: {:?} > {:?}",
                result.execution_time,
                max_duration
            );
        }
        Ok(())
    }
    
    /// Validate output format for specific command types
    pub fn assert_output_format(result: &TestResult, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => {
                serde_json::from_str::<Value>(&result.stdout_text)
                    .context("Output is not valid JSON")?;
            }
            OutputFormat::Csv => {
                if !result.stdout_text.contains(',') {
                    anyhow::bail!("Output does not appear to be CSV format");
                }
            }
            OutputFormat::Human => {
                if result.stdout_text.trim().is_empty() {
                    anyhow::bail!("Human format output should not be empty");
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Csv,
    Human,
}

/// Locate the pmat binary for testing
fn locate_pmat_binary() -> Result<PathBuf> {
    // First try relative to current directory (development)
    let dev_path = Path::new("target/debug/pmat");
    if dev_path.exists() {
        return Ok(dev_path.to_path_buf());
    }
    
    // Try target/release/pmat
    let release_path = Path::new("target/release/pmat");
    if release_path.exists() {
        return Ok(release_path.to_path_buf());
    }
    
    // Try system PATH
    if let Ok(output) = Command::new("which").arg("pmat").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(PathBuf::from(path_str));
        }
    }
    
    anyhow::bail!(
        "Could not locate pmat binary. Try running 'cargo build' first."
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_runner_creation() {
        let runner = CliTestRunner::new();
        assert!(runner.is_ok());
        
        if let Ok(runner) = runner {
            assert!(runner.pmat_binary_path.exists());
            assert!(runner.test_workspace.path().exists());
        }
    }
    
    #[test]
    fn test_sample_project_creation() {
        let runner = CliTestRunner::new().unwrap();
        let project_path = runner.create_sample_project().unwrap();
        
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/main.rs").exists());
        assert!(project_path.join("src/lib.rs").exists());
    }
    
    #[test]
    fn test_environment_variables() {
        let mut runner = CliTestRunner::new().unwrap();
        runner.set_env("TEST_VAR", "test_value");
        
        assert_eq!(runner.environment_vars.get("TEST_VAR"), Some(&"test_value".to_string()));
    }
}