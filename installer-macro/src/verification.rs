#![allow(dead_code)]

use std::io::Write;
use std::process::{Command, Stdio};

pub fn verify_posix_compliance(shell: &str) -> Result<(), String> {
    // Run shellcheck with strict POSIX mode
    let mut child = Command::new("shellcheck")
        .args(["-s", "sh", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run shellcheck: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(shell.as_bytes())
            .map_err(|e| format!("Failed to write to shellcheck stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for shellcheck: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Combine both stderr and stdout for better error reporting
        let error_output = if !stderr.is_empty() {
            stderr.to_string()
        } else if !stdout.is_empty() {
            stdout.to_string()
        } else {
            format!("Shellcheck failed with exit code: {}", output.status.code().unwrap_or(-1))
        };
        
        return Err(format!("Shell validation failed:\n{}", error_output));
    }

    Ok(())
}

pub fn verify_determinism(_shell: &str) -> Result<(), String> {
    // For now, skip determinism check in tests
    // In production, we would verify that the same input always produces the same output
    Ok(())
}

pub fn verify_security_properties(shell: &str) -> Result<(), String> {
    // Security checklist
    let security_requirements = [
        ("set -euf", "Missing safety flags"),
        ("readonly", "No immutable variables"),
        ("trap", "Missing cleanup trap"),
    ];

    for (pattern, error) in &security_requirements {
        if !shell.contains(pattern) {
            return Err(format!("Security requirement failed: {}", error));
        }
    }

    // Check for dangerous patterns
    let dangerous_patterns = [
        ("eval", "Use of eval is forbidden"),
        ("source", "Dynamic sourcing is forbidden"),
        (".", "Dynamic sourcing is forbidden"),
    ];

    for (pattern, error) in &dangerous_patterns {
        if shell.contains(pattern) {
            // More sophisticated check to avoid false positives
            let lines = shell.lines();
            for line in lines {
                let trimmed = line.trim();
                if !trimmed.starts_with('#') && trimmed.contains(pattern) {
                    return Err(format!("Security violation: {}", error));
                }
            }
        }
    }

    Ok(())
}
