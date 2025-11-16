//! BUG-005: Broken Progress Output - RED Phase Tests
//!
//! These tests verify that progress output updates in place (single line)
//! instead of creating multiple lines.
//!
//! Current Status: 🔴 RED - These tests will FAIL until progress is fixed
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write failing tests that verify single-line progress updates
//! 2. GREEN: Implement proper ANSI escape codes or indicatif library
//! 3. REFACTOR: Ensure clean code
//! 4. COMMIT: Single atomic commit with fix

use std::process::{Command, Stdio};
use tempfile::TempDir;

// =============================================================================
// RED TEST 1: Progress Should Not Create Multiple Lines
// =============================================================================

#[test]
#[ignore = "BUG-005: RED test - will fail until progress output fixed"]
fn test_progress_uses_single_line() {
    // Arrange: Create a simple project
    let project = create_simple_rust_project();

    // Act: Run context command and capture stderr
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "pmat",
            "--",
            "context",
            "--language",
            "rust",
        ])
        .current_dir(project.path())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run pmat context");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert: Progress should use carriage return (\r) for single-line updates
    // Count lines that start with progress indicators
    let progress_lines: Vec<&str> = stderr
        .lines()
        .filter(|line| line.contains("Auto-detecting") || line.contains("Detected:"))
        .collect();

    // Should only show final "Detected:" line, not "Auto-detecting" + "Detected:"
    // because "Auto-detecting" should be overwritten
    assert!(
        progress_lines.len() <= 1,
        "Progress should update single line, found {} lines: {:?}",
        progress_lines.len(),
        progress_lines
    );
}

// =============================================================================
// RED TEST 2: Progress Should Use ANSI Escape Codes
// =============================================================================

#[test]
#[ignore = "BUG-005: RED test - will fail until ANSI codes implemented"]
fn test_progress_uses_ansi_codes() {
    // Arrange: Create a simple project
    let project = create_simple_rust_project();

    // Act: Run context command and capture raw stderr
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "pmat",
            "--",
            "context",
            "--language",
            "rust",
        ])
        .current_dir(project.path())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run pmat context");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert: Should contain carriage return (\r) or ANSI clear line code (\x1b[K)
    let has_carriage_return = stderr.contains('\r');
    let has_clear_line = stderr.contains("\x1b[K");

    assert!(
        has_carriage_return || has_clear_line,
        "Progress should use ANSI escape codes (\\r or \\x1b[K) for in-place updates"
    );
}

// =============================================================================
// RED TEST 3: Auto-detecting Line Should Be Overwritten
// =============================================================================

#[test]
#[ignore = "BUG-005: RED test - will fail until overwrite implemented"]
fn test_auto_detecting_line_overwritten() {
    // Arrange: Create a simple project
    let project = create_simple_rust_project();

    // Act: Run context command
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "pmat",
            "--",
            "context",
            "--language",
            "rust",
        ])
        .current_dir(project.path())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run pmat context");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert: If we see "Detected:" we should NOT also see "Auto-detecting" on separate line
    // Because the "Auto-detecting" line should have been overwritten
    if stderr.contains("Detected:") {
        // Split by carriage returns to see actual visible output
        let visible_lines: Vec<&str> = stderr.split('\n').filter(|line| !line.is_empty()).collect();

        let auto_detect_count = visible_lines
            .iter()
            .filter(|line| line.contains("Auto-detecting"))
            .count();

        let detected_count = visible_lines
            .iter()
            .filter(|line| line.contains("Detected:"))
            .count();

        // If both appear as separate lines, that's the bug
        assert!(
            !(auto_detect_count > 0 && detected_count > 0),
            "Bug: Both 'Auto-detecting' and 'Detected:' visible on separate lines\n\
             Expected: 'Auto-detecting' should be overwritten by 'Detected:'\n\
             Found {} 'Auto-detecting' and {} 'Detected:' lines",
            auto_detect_count,
            detected_count
        );
    }
}

// =============================================================================
// RED TEST 4: No Visual Corruption in Output
// =============================================================================

#[test]
#[ignore = "BUG-005: RED test - will fail until corruption fixed"]
fn test_no_visual_corruption() {
    // Arrange: Create a simple project
    let project = create_simple_rust_project();

    // Act: Run context command
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "pmat",
            "--",
            "context",
            "--language",
            "rust",
        ])
        .current_dir(project.path())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run pmat context");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert: No lines should have overlapping text
    // This is hard to test directly, but we can check for common corruption patterns
    for line in stderr.lines() {
        // Check for multiple progress indicators on same line (corruption)
        let indicator_count =
            line.matches("🔍").count() + line.matches("✅").count() + line.matches("⚠️").count();

        assert!(
            indicator_count <= 1,
            "Visual corruption detected: Multiple indicators on same line: {}",
            line
        );
    }
}

// =============================================================================
// RED TEST 5: Final Output Should Be Clean
// =============================================================================

#[test]
#[ignore = "BUG-005: RED test - will fail until clean output implemented"]
fn test_final_output_is_clean() {
    // Arrange: Create a simple project
    let project = create_simple_rust_project();

    // Act: Run context command
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "pmat",
            "--",
            "context",
            "--language",
            "rust",
        ])
        .current_dir(project.path())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run pmat context");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert: Final visible output should be clean
    // Count total lines that contain progress indicators
    let progress_line_count = stderr
        .lines()
        .filter(|line| {
            line.contains("Auto-detecting")
                || line.contains("Detected:")
                || line.contains("Discovering")
        })
        .count();

    // Should see at most 2-3 clean lines (detecting, detected, maybe discovering)
    // NOT multiple instances of the same progress message
    assert!(
        progress_line_count <= 3,
        "Too many progress lines ({}), suggests not updating in place",
        progress_line_count
    );
}

// =============================================================================
// Helper Functions (Test Support)
// =============================================================================

fn create_simple_rust_project() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();

    // Create a simple main.rs file
    let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;

    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/main.rs"), code).unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    temp
}
