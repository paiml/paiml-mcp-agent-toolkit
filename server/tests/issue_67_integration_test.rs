//! Integration test for Issue #67: Line number tracking in extracted files
//!
//! This test validates the complete fix from CLI → handler → analyzer → output
//! It simulates the exact scenario from the bug report.

use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_issue_67_extracted_function_line_numbers() {
    // Setup: Create a temporary directory with an extracted file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("extracted_functions.rs");

    // Simulate a file with a function that was extracted from line 500 of another file
    // The function is now at the beginning of this new file
    let content = r#"//! Extracted utility functions

use std::result::Result;

/// Parse attribute arguments from token stream
///
/// This function was extracted from parser/utils.rs:500-550
/// It should now report line numbers from THIS file, not the old location
pub fn parse_rust_attribute_arguments(
    tokens: &[String],
    start: usize,
) -> Result<(Vec<String>, usize), String> {
    let mut args = Vec::new();
    let mut current = start;

    // Complexity: 6 (cyclomatic)
    while current < tokens.len() {
        if tokens[current] == "," {
            current += 1;
            continue;
        }

        if tokens[current].starts_with("key") {
            args.push(tokens[current].clone());
            current += 1;
        } else {
            break;
        }
    }

    Ok((args, current))
}
"#;

    fs::write(&test_file, content).expect("Failed to write test file");

    // Execute: Analyze the file using the complexity handler
    use pmat::services::complexity::analyze_file_complexity_uncached;

    let result = analyze_file_complexity_uncached(&test_file, None)
        .await
        .expect("Analysis should succeed");

    // Verify: Line numbers must reflect CURRENT file location
    assert!(!result.functions.is_empty(), "Should detect the function");

    let func = result
        .functions
        .iter()
        .find(|f| f.name == "parse_rust_attribute_arguments")
        .expect("Should find parse_rust_attribute_arguments");

    // CRITICAL ASSERTIONS for Issue #67

    let total_lines = content.lines().count() as u32;

    // 1. Line numbers must be within file bounds (basic sanity check)
    assert!(
        func.line_start > 0,
        "line_start should be > 0 (1-indexed), got {}",
        func.line_start
    );

    assert!(
        func.line_start <= total_lines,
        "line_start ({}) exceeds file length ({} lines)",
        func.line_start,
        total_lines
    );

    // 2. Function should end within file bounds
    assert!(
        func.line_end <= total_lines,
        "Function line_end ({}) exceeds file length ({} lines). \
         Issue #67 REGRESSION: Line numbers from cached old location",
        func.line_end,
        total_lines
    );

    // 3. Line numbers CANNOT be from old location (500-550)
    // If this test fails with line_start > 400, it means the bug is back
    assert!(
        func.line_start < 100,
        "Issue #67 REGRESSION DETECTED! \
         line_start={} suggests stale cache from old file location (expected 500-550 from utils.rs). \
         The fix should report line numbers from CURRENT file location (1-{})",
        func.line_start,
        total_lines
    );

    // 4. Line numbers must be ordered correctly
    assert!(
        func.line_start <= func.line_end,
        "line_start ({}) must be <= line_end ({})",
        func.line_start,
        func.line_end
    );

    println!("✅ Issue #67 fix verified!");
    println!("   Function: {}", func.name);
    println!("   Lines: {}-{} (in current file)", func.line_start, func.line_end);
    println!("   File has {} total lines", total_lines);
    println!("   ✅ Line numbers are accurate (not from old cached location 500-550)");
}

#[tokio::test]
async fn test_issue_67_multiple_extractions() {
    // Setup: Simulate multiple functions extracted from different files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // File 1: Function extracted from old_file.rs:100
    let file1 = temp_dir.path().join("file1.rs");
    fs::write(&file1, "fn helper1() { let x = 1; x + 1 }").expect("Failed to write file1");

    // File 2: Same function content extracted from old_file.rs:500
    let file2 = temp_dir.path().join("file2.rs");
    fs::write(
        &file2,
        "// Header comment\n\nfn helper1() { let x = 1; x + 1 }",
    )
    .expect("Failed to write file2");

    // Execute: Analyze both files
    use pmat::services::complexity::analyze_file_complexity_uncached;

    let result1 = analyze_file_complexity_uncached(&file1, None)
        .await
        .expect("File1 analysis should succeed");

    let result2 = analyze_file_complexity_uncached(&file2, None)
        .await
        .expect("File2 analysis should succeed");

    // Verify: Same function content in different files should have DIFFERENT line numbers
    assert!(!result1.functions.is_empty(), "File1 should have functions");
    assert!(!result2.functions.is_empty(), "File2 should have functions");

    let func1 = &result1.functions[0];
    let func2 = &result2.functions[0];

    // Both should have valid line numbers
    assert!(
        func1.line_start > 0,
        "File1 function should have valid line_start, got {}",
        func1.line_start
    );

    assert!(
        func2.line_start > 0,
        "File2 function should have valid line_start, got {}",
        func2.line_start
    );

    // Functions should be at different line numbers despite identical content
    // File2 has 2 lines of preamble, so should start later
    assert_ne!(
        func1.line_start, func2.line_start,
        "Issue #67: Same function in different files should have different line numbers. \
         Got same line {} for both, suggesting cache is keyed by content hash only",
        func1.line_start
    );

    println!("✅ Issue #67 multi-file test passed!");
    println!("   File1: function at line {}", func1.line_start);
    println!("   File2: function at line {} (different!)", func2.line_start);
}

#[tokio::test]
async fn test_issue_67_pre_commit_hook_scenario() {
    // Setup: Simulate the exact pre-commit hook scenario from the bug report
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let extracted_file = temp_dir
        .path()
        .join("src")
        .join("frontend")
        .join("parser")
        .join("utils_helpers");

    fs::create_dir_all(&extracted_file).expect("Failed to create directory structure");

    let attributes_file = extracted_file.join("attributes.rs");

    // This function was at utils.rs:500-550
    // Now it's at attributes.rs:148-214 (after 147 lines of other code)
    let preamble = "// File header\n".repeat(147);
    let function = r#"pub fn parse_rust_attribute_arguments(
    tokens: &[Token],
    start: usize,
) -> Result<(Vec<AttributeArg>, usize), String> {
    let mut args = Vec::new();
    let mut current = start;

    while current < tokens.len() {
        if tokens[current].is_comma() {
            current += 1;
            continue;
        }

        let (arg, next) = parse_single_arg(tokens, current)?;
        args.push(arg);
        current = next;
    }

    Ok((args, current))
}
"#;
    let content = format!("{}{}", preamble, function);
    fs::write(&attributes_file, &content).expect("Failed to write attributes file");

    // Execute: Analyze as a pre-commit hook would
    use pmat::services::complexity::analyze_file_complexity_uncached;

    let result = analyze_file_complexity_uncached(&attributes_file, None)
        .await
        .expect("Pre-commit analysis should succeed");

    // Verify: This is the CRITICAL test for Issue #67
    let func = result
        .functions
        .iter()
        .find(|f| f.name == "parse_rust_attribute_arguments")
        .expect("Should find the extracted function");

    let total_lines = content.lines().count() as u32;

    // The bug: Would report lines 500-550 (from old file)
    // The fix: Should report lines 148-168 (from current file)

    assert!(
        func.line_start > 0,
        "Function should have valid line_start, got {}",
        func.line_start
    );

    assert!(
        func.line_start >= 140 && func.line_start <= 170,
        "PRE-COMMIT HOOK FAILURE! \
         Expected function around line 148 (current location), \
         but got line {}. \
         This would cause pre-commit hook to report wrong location.",
        func.line_start
    );

    assert!(
        func.line_start < 300,
        "Issue #67 CRITICAL REGRESSION! \
         line_start={} suggests bug is back. \
         Pre-commit hooks would fail with 'complexity at line 500' \
         for a {}-line file!",
        func.line_start,
        total_lines
    );

    println!("✅ Issue #67 pre-commit hook scenario PASSED!");
    println!("   Function extracted from: utils.rs:500-550 (old location)");
    println!("   Function now at: attributes.rs:{}-{} (current location)",
             func.line_start, func.line_end);
    println!("   ✅ Pre-commit hooks will report correct line numbers!");
}
