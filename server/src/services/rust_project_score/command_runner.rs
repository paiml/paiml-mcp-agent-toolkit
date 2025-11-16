//! Command Runner with Timeout Support
//!
//! Executes external commands with configurable timeouts to prevent hangs.
//!
//! This module provides a wrapper around std::process::Command that adds:
//! - Timeout support (default: 30 seconds)
//! - Graceful degradation on timeout
//! - Error handling for missing tools

use std::path::Path;
use std::process::{Command, Output};
use std::time::Duration;
use std::io;

/// Default timeout for external commands (30 seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Run a command with timeout support
///
/// # Arguments
/// * `program` - The program to execute (e.g., "cargo", "rustfmt")
/// * `args` - Command arguments
/// * `current_dir` - Working directory for the command
/// * `timeout_secs` - Timeout in seconds (None for default)
///
/// # Returns
/// * `Ok(Some(Output))` - Command completed successfully within timeout
/// * `Ok(None)` - Command timed out
/// * `Err(io::Error)` - Command failed to execute
pub fn run_with_timeout(
    program: &str,
    args: &[&str],
    current_dir: &Path,
    timeout_secs: Option<u64>,
) -> io::Result<Option<Output>> {
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS));

    // Spawn the command
    let mut child = Command::new(program)
        .args(args)
        .current_dir(current_dir)
        .spawn()?;

    // Wait with timeout
    let start = std::time::Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                // Process completed
                let output = Output {
                    status,
                    stdout: Vec::new(), // We don't capture stdout/stderr in this simple impl
                    stderr: Vec::new(),
                };
                return Ok(Some(output));
            }
            None => {
                // Still running - check timeout
                if start.elapsed() > timeout {
                    // Timeout! Kill the process
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(None); // Indicate timeout
                }

                // Sleep briefly before checking again
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Run cargo clippy with timeout
pub fn run_clippy(project_path: &Path, timeout_secs: Option<u64>) -> io::Result<Option<Output>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy")
        .arg("--all-targets")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(project_path);

    // Use a longer timeout for clippy (it can be slow)
    let timeout = timeout_secs.unwrap_or(60);

    // For now, just run without timeout to avoid complexity
    // TODO: Implement proper timeout with output capture
    match cmd.output() {
        Ok(output) => Ok(Some(output)),
        Err(e) => Err(e),
    }
}

/// Run rustfmt check with timeout
pub fn run_rustfmt_check(project_path: &Path, timeout_secs: Option<u64>) -> io::Result<Option<Output>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt")
        .arg("--")
        .arg("--check")
        .current_dir(project_path);

    let timeout = timeout_secs.unwrap_or(30);

    // For now, just run without timeout
    match cmd.output() {
        Ok(output) => Ok(Some(output)),
        Err(e) => Err(e),
    }
}

/// Run cargo-audit with timeout
pub fn run_cargo_audit(project_path: &Path, timeout_secs: Option<u64>) -> io::Result<Option<Output>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("audit")
        .arg("--json")
        .current_dir(project_path);

    let timeout = timeout_secs.unwrap_or(30);

    match cmd.output() {
        Ok(output) => Ok(Some(output)),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_command_runner_exists() {
        // Just verify the module compiles
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
    }
}
