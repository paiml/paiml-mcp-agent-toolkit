//! Integration tests for complexity threshold filtering
//!
//! These tests verify that the --max-cyclomatic and --max-cognitive flags
//! properly filter files based on their function complexity thresholds.
//! This addresses GitHub issue #32.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Create a test file with known complexity values
fn create_test_file(dir: &TempDir, name: &str, complexity: u16) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let content = match complexity {
        1..=5 => {
            // Simple function (complexity 1)
            String::from(
                r#"
fn simple_function(x: i32) -> i32 {
    x + 1
}
"#,
            )
        }
        6..=10 => {
            // Medium complexity function (complexity ~8)
            String::from(
                r#"
fn medium_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 20 {
                x * 3
            } else {
                x * 2
            }
        } else {
            x + 1
        }
    } else if x < 0 {
        if x < -10 {
            -x * 2
        } else {
            -x
        }
    } else {
        0
    }
}
"#,
            )
        }
        11..=15 => {
            // Function with exact complexity values (14 or 15)
            let extra_conditions = if complexity == 15 { 2 } else { 1 };
            let mut code = String::from(
                r#"
fn high_function(x: i32, y: i32) -> i32 {
    // Base complexity: 1 (function entry)
    let mut result = 0;

    // Add conditions to reach target complexity
    if x > 0 {           // +1 = 2
        result += 1;
        if x > 10 {      // +1 = 3
            result += 2;
            if x > 20 {  // +1 = 4
                result += 3;
            }
        }
        if x > 30 {      // +1 = 5
            result += 4;
        }
    } else if x < 0 {    // +1 = 6
        result -= 1;
        if x < -10 {     // +1 = 7
            result -= 2;
        }
    } else {             // else is not a branch
        result = 0;
    }

    // Add more branches to reach target
    if y > 0 {           // +1 = 8
        result += y;
        if y > 10 {      // +1 = 9
            result *= 2;
        }
    } else if y < 0 {    // +1 = 10
        result -= y;
    }

    // Add match to increase complexity
    match result % 3 {   // +2 (for 2 non-default cases) = 12
        0 => result * 2,
        1 => result + 1,
        _ => result,
    }
"#,
            );

            // Add extra conditions if needed for complexity 15
            if extra_conditions == 2 {
                code.push_str(
                    r#"

    // Extra conditions for complexity 15
    if result > 100 {    // +1 = 13
        result / 2
    } else if result < -100 {  // +1 = 14
        result * -1
    } else if result == 0 {    // +1 = 15
        1
    } else {
        result
    }"#,
                );
            } else {
                code.push_str(
                    r#"

    // Extra condition for complexity 14
    if result > 100 {    // +1 = 13
        result / 2
    } else if result < -100 {  // +1 = 14
        result * -1
    } else {
        result
    }"#,
                );
            }

            code.push_str("\n}\n");
            code
        }
        16..=20 => {
            // Function with exact complexity (16-20)
            // Cyclomatic complexity = 1 + number of decision points
            // Each if adds 1, each else-if adds 1
            let target = complexity;
            let mut code = String::from(
                r#"
fn very_high_function(x: i32, y: i32, z: i32) -> i32 {
    let mut result = 0;
"#,
            );

            // Add independent if statements - each adds 1 to complexity
            // We need target-1 decision points since base complexity is 1
            for i in 1..target {
                code.push_str(&format!(
                    r#"
    if x > {} {{
        result += {};
    }}"#,
                    i * 10,
                    i
                ));
            }

            code.push_str(
                r#"

    result
}
"#,
            );
            code
        }
        21..=25 => {
            // Extremely high complexity function (complexity ~25)
            let if_statements = (0..15)
                .map(|i| format!("    if x == {} {{\n        return {};\n    }}", i, i * 2))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                r#"
fn extremely_complex_function(x: i32) -> i32 {{
{}
    x
}}
"#,
                if_statements
            )
        }
        _ => {
            // Ultra high complexity function - generate many linear if statements
            let if_statements = (0..complexity.saturating_sub(1))
                .map(|i| format!("    if x == {} {{\n        return {};\n    }}", i, i * 2))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                r#"
fn ultra_complex_function(x: i32) -> i32 {{
{}
    x
}}
"#,
                if_statements
            )
        }
    };

    fs::write(&path, content).unwrap();
    path
}

#[ignore]
#[test]
fn test_max_cyclomatic_filters_correctly() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files with different complexity levels
    create_test_file(&temp_dir, "simple.rs", 3);
    create_test_file(&temp_dir, "medium.rs", 10);
    create_test_file(&temp_dir, "complex.rs", 25);
    create_test_file(&temp_dir, "very_complex.rs", 40);

    // Test with threshold of 20 - should only show complex.rs and very_complex.rs
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir).args([
        "analyze",
        "complexity",
        "--max-cyclomatic",
        "20",
        "--format",
        "json",
    ]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Files with complexity <= 20 should NOT appear
    assert!(
        !stdout.contains("simple.rs"),
        "simple.rs should be filtered out"
    );
    assert!(
        !stdout.contains("medium.rs"),
        "medium.rs should be filtered out"
    );

    // Files with complexity > 20 should appear
    assert!(
        stdout.contains("complex.rs"),
        "complex.rs should be included"
    );
    assert!(
        stdout.contains("very_complex.rs"),
        "very_complex.rs should be included"
    );
}

#[ignore]
#[test]
fn test_max_cyclomatic_with_top_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create 5 files with varying complexity
    create_test_file(&temp_dir, "file1.rs", 5); // Below threshold
    create_test_file(&temp_dir, "file2.rs", 15); // Below threshold
    create_test_file(&temp_dir, "file3.rs", 25); // Above threshold
    create_test_file(&temp_dir, "file4.rs", 35); // Above threshold
    create_test_file(&temp_dir, "file5.rs", 45); // Above threshold

    // Test with threshold of 20 and top-files of 2
    // Should show only file4.rs and file5.rs (the top 2 files above threshold)
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir).args([
        "analyze",
        "complexity",
        "--max-cyclomatic",
        "20",
        "--top-files",
        "2",
        "--format",
        "json",
    ]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Files below threshold should not appear
    assert!(
        !stdout.contains("file1.rs"),
        "file1.rs should be filtered out"
    );
    assert!(
        !stdout.contains("file2.rs"),
        "file2.rs should be filtered out"
    );

    // Only top 2 files above threshold should appear
    assert!(
        !stdout.contains("file3.rs"),
        "file3.rs should be excluded by top-files limit"
    );
    assert!(stdout.contains("file4.rs"), "file4.rs should be included");
    assert!(stdout.contains("file5.rs"), "file5.rs should be included");
}

#[ignore]
#[test]
fn test_exact_threshold_boundary() {
    let temp_dir = TempDir::new().unwrap();

    // Create files at and around the threshold
    create_test_file(&temp_dir, "below.rs", 14);
    create_test_file(&temp_dir, "exact.rs", 15);
    create_test_file(&temp_dir, "above.rs", 16);

    // Test with threshold of 15
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir).args([
        "analyze",
        "complexity",
        "--max-cyclomatic",
        "15",
        "--format",
        "json",
    ]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Functions at or below threshold should not appear
    assert!(
        !stdout.contains("below.rs"),
        "below.rs (14) should be filtered out"
    );
    assert!(
        !stdout.contains("exact.rs"),
        "exact.rs (15) should be filtered out"
    );

    // Only functions above threshold should appear
    assert!(
        stdout.contains("above.rs"),
        "above.rs (16) should be included"
    );
}

#[ignore]
#[test]
fn test_no_files_above_threshold() {
    let temp_dir = TempDir::new().unwrap();

    // Create only simple files
    create_test_file(&temp_dir, "simple1.rs", 3);
    create_test_file(&temp_dir, "simple2.rs", 5);
    create_test_file(&temp_dir, "simple3.rs", 8);

    // Test with high threshold - no files should match
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "complexity", "--max-cyclomatic", "50"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("**Files analyzed**: 0"));
}
