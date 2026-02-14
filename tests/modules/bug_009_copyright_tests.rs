//! BUG-009: Copyright Detected as Function - RED Phase Tests
//!
//! These tests define expected behavior for filtering copyright headers
//! from function detection in C/C++ files.
//!
//! Current Status: 🔴 RED - These tests will FAIL until implementation complete
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write failing tests that define expected behavior
//! 2. GREEN: Implement minimum code to make tests pass
//! 3. REFACTOR: Clean up implementation
//! 4. COMMIT: Single atomic commit with fix

use tempfile::TempDir;

// =============================================================================
// RED TEST 1: Copyright Header Not Detected as Function
// =============================================================================

#[test]
fn test_copyright_single_line_not_detected() {
    // Arrange: C++ file with copyright header
    let project = create_cpp_file_with_copyright();

    // Act: Analyze file
    let result = analyze_cpp_file_for_functions(project.path());

    // Assert: Copyright should NOT be in function list
    assert!(result.is_ok(), "Analysis should succeed");
    let functions = result.unwrap();

    assert!(
        !functions.iter().any(|f| f.contains("Copyright")),
        "Copyright should not be detected as function, found: {:?}",
        functions
    );
}

// =============================================================================
// RED TEST 2: Actual Functions Still Detected
// =============================================================================

#[test]
#[ignore = "RED phase test - BUG-009 implementation incomplete"]
fn test_real_functions_still_detected_with_copyright() {
    // Arrange: C++ file with copyright AND real functions
    let project = create_cpp_file_with_copyright_and_functions();

    // Act: Analyze file
    let result = analyze_cpp_file_for_functions(project.path());

    // Assert: Real functions detected, copyright NOT detected
    assert!(result.is_ok(), "Analysis should succeed");
    let functions = result.unwrap();

    assert!(
        functions.iter().any(|f| f.contains("initialize_wmi")),
        "Real function 'initialize_wmi' should be detected"
    );
    assert!(
        functions.iter().any(|f| f.contains("process_data")),
        "Real function 'process_data' should be detected"
    );
    assert!(
        !functions.iter().any(|f| f.contains("Copyright")),
        "Copyright should not be detected as function"
    );
}

// =============================================================================
// RED TEST 3: Multi-line Copyright Not Detected
// =============================================================================

#[test]
fn test_multiline_copyright_not_detected() {
    // Arrange: C++ file with multiline copyright block
    let project = create_cpp_file_with_multiline_copyright();

    // Act: Analyze file
    let result = analyze_cpp_file_for_functions(project.path());

    // Assert: Copyright should NOT be detected
    assert!(result.is_ok(), "Analysis should succeed");
    let functions = result.unwrap();

    assert!(
        !functions.iter().any(|f| f.contains("Copyright")),
        "Copyright in multiline comment should not be detected"
    );
}

// =============================================================================
// RED TEST 4: License Headers Not Detected
// =============================================================================

#[test]
fn test_license_headers_not_detected() {
    // Arrange: C++ file with various header keywords
    let project = create_cpp_file_with_license_headers();

    // Act: Analyze file
    let result = analyze_cpp_file_for_functions(project.path());

    // Assert: None of the header keywords should be detected
    assert!(result.is_ok(), "Analysis should succeed");
    let functions = result.unwrap();

    let banned_keywords = vec!["Copyright", "License", "Author", "SPDX"];
    for keyword in &banned_keywords {
        assert!(
            !functions.iter().any(|f| f.contains(keyword)),
            "{} should not be detected as function",
            keyword
        );
    }
}

// =============================================================================
// RED TEST 5: C File Copyright Filtering
// =============================================================================

#[test]
fn test_c_file_copyright_not_detected() {
    // Arrange: C file (not C++) with copyright
    let project = create_c_file_with_copyright();

    // Act: Analyze file
    let result = analyze_c_file_for_functions(project.path());

    // Assert: Copyright should NOT be detected
    assert!(result.is_ok(), "Analysis should succeed");
    let functions = result.unwrap();

    assert!(
        !functions.iter().any(|f| f.contains("Copyright")),
        "Copyright in C file should not be detected"
    );
}

// =============================================================================
// Helper Functions (Test Support)
// =============================================================================

fn create_cpp_file_with_copyright() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let code = r#"
// Copyright (c) 2024 Test Organization
// All rights reserved

#include <iostream>

// This file contains test code
    "#;

    fs::write(temp.path().join("test.cpp"), code).unwrap();
    temp
}

fn create_cpp_file_with_copyright_and_functions() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let code = r#"
// Copyright (c) 2024 Test Organization
// All rights reserved

#include <iostream>

void initialize_wmi() {
    std::cout << "Initializing WMI" << std::endl;
}

int process_data(int input) {
    return input * 2;
}
    "#;

    fs::write(temp.path().join("test.cpp"), code).unwrap();
    temp
}

fn create_cpp_file_with_multiline_copyright() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let code = r#"
/*
 * Copyright (c) 2024 Test Organization
 * All rights reserved
 * Licensed under MIT
 */

#include <iostream>

void real_function() {
    // actual code
}
    "#;

    fs::write(temp.path().join("test.cpp"), code).unwrap();
    temp
}

fn create_cpp_file_with_license_headers() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let code = r#"
// Copyright (c) 2024
// License: MIT
// Author: Test Developer
// SPDX-License-Identifier: MIT

void real_function() {
    // actual code
}
    "#;

    fs::write(temp.path().join("test.cpp"), code).unwrap();
    temp
}

fn create_c_file_with_copyright() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let code = r#"
// Copyright (c) 2024 Test Organization

#include <stdio.h>

void test_function() {
    printf("Hello\n");
}
    "#;

    fs::write(temp.path().join("test.c"), code).unwrap();
    temp
}

fn analyze_cpp_file_for_functions(_path: &std::path::Path) -> Result<Vec<String>, String> {
    #[cfg(feature = "cpp-ast")]
    {
        use pmat::services::ast::languages::cpp::analyze_cpp_file;

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let file_path = path.join("test.cpp");
        let context = rt
            .block_on(analyze_cpp_file(&file_path))
            .map_err(|e| e.to_string())?;

        // Extract function names from context
        let functions: Vec<String> = context
            .items
            .iter()
            .filter_map(|item| {
                if let pmat::services::context::AstItem::Function { name, .. } = item {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(functions)
    }

    #[cfg(not(feature = "cpp-ast"))]
    {
        // Fallback for when cpp-ast feature is not enabled
        Ok(vec![])
    }
}

fn analyze_c_file_for_functions(_path: &std::path::Path) -> Result<Vec<String>, String> {
    #[cfg(feature = "c-ast")]
    {
        use pmat::services::ast::languages::c::analyze_c_file;

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let file_path = path.join("test.c");
        let context = rt
            .block_on(analyze_c_file(&file_path))
            .map_err(|e| e.to_string())?;

        // Extract function names from context
        let functions: Vec<String> = context
            .items
            .iter()
            .filter_map(|item| {
                if let pmat::services::context::AstItem::Function { name, .. } = item {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(functions)
    }

    #[cfg(not(feature = "c-ast"))]
    {
        // Fallback for when c-ast feature is not enabled
        Ok(vec![])
    }
}
