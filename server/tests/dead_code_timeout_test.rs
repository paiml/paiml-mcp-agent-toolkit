//! TDD Test for Dead Code Analysis Timeout
//!
//! Following TDD methodology, we write this test FIRST before fixing the bug.
//! This test verifies that dead-code analysis completes within a reasonable time.

use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const MAX_TIMEOUT_SECS: u64 = 30; // Dead code analysis should complete within 30 seconds (large project)

#[test]
fn test_dead_code_completes_within_timeout() {
    let (tx, rx) = mpsc::channel();

    // Spawn thread to run the command
    let _handle = thread::spawn(move || {
        let start = Instant::now();

        let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
            .args(["analyze", "dead-code", "--path", "."])
            .output();

        let duration = start.elapsed();
        tx.send((output, duration)).unwrap();
    });

    // Wait for completion with timeout
    let result = rx.recv_timeout(Duration::from_secs(MAX_TIMEOUT_SECS));

    match result {
        Ok((output_result, duration)) => {
            match output_result {
                Ok(output) => {
                    // Command completed within timeout
                    println!("Dead code analysis completed in {:?}", duration);

                    // Verify it actually ran successfully
                    assert!(
                        output.status.success(),
                        "Dead code analysis failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(
                        stdout.contains("Dead Code Analysis") || stdout.contains("Files analyzed"),
                        "Output doesn't contain expected content: {}",
                        stdout
                    );
                }
                Err(e) => {
                    panic!("Failed to run dead code analysis: {}", e);
                }
            }
        }
        Err(_) => {
            // Timeout occurred - this is the bug we're fixing
            panic!("CRITICAL BUG: Dead code analysis timed out after {} seconds! The command is hanging indefinitely.", MAX_TIMEOUT_SECS);
        }
    }
}

#[test]
fn test_dead_code_handles_empty_directory() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args([
            "analyze",
            "dead-code",
            "--path",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    let duration = start.elapsed();

    // Should complete quickly on empty directory
    assert!(
        duration.as_secs() < 2,
        "Empty directory analysis took too long: {:?}",
        duration
    );
    assert!(
        output.status.success(),
        "Should handle empty directory gracefully"
    );
}

#[test]
fn test_dead_code_handles_single_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(temp_file, "fn main() {{ println!(\"Hello\"); }}").expect("Failed to write");

    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args([
            "analyze",
            "dead-code",
            "--path",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    let duration = start.elapsed();

    // Single file analysis should be instant
    assert!(
        duration.as_secs() < 1,
        "Single file analysis took too long: {:?}",
        duration
    );
    assert!(
        output.status.success(),
        "Should handle single file analysis"
    );
}

#[test]
fn test_dead_code_with_max_depth_limit() {
    // Test that max-depth parameter prevents infinite recursion
    let output = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(["analyze", "dead-code", "--path", ".", "--max-depth", "2"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success(), "Max depth should prevent hanging");
}
