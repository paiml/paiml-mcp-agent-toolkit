#![allow(dead_code)]

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;

pub fn verify_posix_compliance(shell: &str) -> Result<(), String> {
    // Run shellcheck with strict POSIX mode
    let mut child = Command::new("shellcheck")
        .args(["-s", "sh", "-e", "all", "-"])
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
        return Err(format!("Shell validation failed:\n{}", stderr));
    }

    Ok(())
}

pub fn verify_determinism(shell: &str) -> Result<(), String> {
    // Thread-local storage for tracking generation
    thread_local! {
        static GENERATED_HASHES: Mutex<Vec<[u8; 32]>> = const { Mutex::new(Vec::new()) };
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(shell.as_bytes());
    let hash: [u8; 32] = hasher.finalize().into();

    GENERATED_HASHES.with(|hashes| {
        let mut hashes = hashes.lock().unwrap();
        if let Some(prev) = hashes.first() {
            if prev != &hash {
                return Err(format!(
                    "Non-deterministic generation detected!\nPrevious: {:x?}\nCurrent: {:x?}",
                    prev, hash
                ));
            }
        }
        hashes.push(hash);
        Ok(())
    })
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
