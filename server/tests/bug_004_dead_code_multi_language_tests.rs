//! BUG-004: Dead Code Multi-Language Tests (RED Phase)
//!
//! These tests define expected behavior for dead code analysis
//! across multiple programming languages without requiring Cargo.toml.
//!
//! Current Status: 🔴 RED - These tests will FAIL until implementation complete
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write failing tests that define expected behavior
//! 2. GREEN: Implement minimum code to make tests pass
//! 3. REFACTOR: Clean up implementation
//! 4. COMMIT: Single atomic commit with fix

use anyhow::Result;
use tempfile::TempDir;

// Import the actual implementation
use pmat::services::dead_code_multi_language::{
    analyze_dead_code_multi_language, DeadCodeResult, DeadFunction,
};

// =============================================================================
// RED TEST 1: C Project Dead Code Analysis
// =============================================================================

#[test]
#[ignore = "BUG-004: RED test - will fail until multi-language dead code implemented"]
fn test_c_project_dead_code_without_cargo_toml() {
    // Arrange: Create C project (no Cargo.toml)
    let project = create_c_project_with_dead_code();

    // Act: Analyze dead code
    let result = analyze_dead_code_multi_language(project.path());

    // Assert: Should succeed and detect C language
    assert!(result.is_ok(), "Should succeed on C project without Cargo.toml");
    let result = result.unwrap();

    assert_eq!(result.language, "c", "Should detect C language");
    assert_eq!(result.total_functions, 2, "Should find 2 functions total");
    assert_eq!(result.dead_functions.len(), 1, "Should find 1 dead function");

    let dead_fn = &result.dead_functions[0];
    assert_eq!(dead_fn.name, "unused_function");
    assert!(dead_fn.file.contains("utils.c"));
}

#[test]
#[ignore = "BUG-004: RED test - will fail until C++ support implemented"]
fn test_cpp_project_dead_code_with_cmake() {
    // Arrange: Create C++ project with CMakeLists.txt
    let project = create_cpp_project_with_dead_code();

    // Act: Analyze dead code
    let result = analyze_dead_code_multi_language(project.path());

    // Assert: Should detect C++ and find dead code
    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.language, "cpp", "Should detect C++ language");
    assert!(result.dead_functions.len() > 0, "Should find dead C++ functions");
}

// =============================================================================
// RED TEST 2: Python Project Dead Code Analysis
// =============================================================================

#[test]
#[ignore = "BUG-004: RED test - will fail until Python support implemented"]
fn test_python_project_dead_code_without_cargo_toml() {
    // Arrange: Create Python project (no Cargo.toml)
    let project = create_python_project_with_dead_code();

    // Act: Analyze dead code
    let result = analyze_dead_code_multi_language(project.path());

    // Assert: Should succeed and detect Python language
    assert!(result.is_ok(), "Should succeed on Python project without Cargo.toml");
    let result = result.unwrap();

    assert_eq!(result.language, "python", "Should detect Python language");
    assert!(result.total_functions >= 2, "Should find at least 2 functions");
    assert!(result.dead_functions.len() >= 1, "Should find at least 1 dead function");

    // Check that unused_function is detected as dead
    assert!(
        result.dead_functions.iter().any(|f| f.name.contains("unused")),
        "Should detect unused_function as dead code"
    );
}

// =============================================================================
// RED TEST 3: Rust Project (Should Still Work)
// =============================================================================

#[test]
#[ignore = "BUG-004: RED test - ensure Rust still works after refactor"]
fn test_rust_project_dead_code_still_works() {
    // Arrange: Create Rust project with Cargo.toml
    let project = create_rust_project_with_dead_code();

    // Act: Analyze dead code
    let result = analyze_dead_code_multi_language(project.path());

    // Assert: Should still work for Rust projects
    assert!(result.is_ok(), "Should still work for Rust projects");
    let result = result.unwrap();

    assert_eq!(result.language, "rust", "Should detect Rust language");
    assert!(result.dead_functions.len() > 0, "Should find dead Rust functions");
}

// =============================================================================
// RED TEST 4: Unsupported Language Error
// =============================================================================

#[test]
#[ignore = "BUG-004: RED test - will fail until error handling implemented"]
fn test_unsupported_language_returns_error() {
    // Arrange: Create project with unsupported language (e.g., Fortran)
    let project = create_fortran_project();

    // Act: Analyze dead code
    let result = analyze_dead_code_multi_language(project.path());

    // Assert: Should return error with helpful message
    assert!(result.is_err(), "Should return error for unsupported language");
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("not supported") || error.contains("unsupported"),
        "Error should mention unsupported language: {}",
        error
    );
}

// =============================================================================
// RED TEST 5: Language Detection Integration
// =============================================================================

#[test]
#[ignore = "BUG-004: RED test - integration with enhanced_language_detection"]
fn test_uses_enhanced_language_detection() {
    // Arrange: Create polyglot project (C++ primary, Python secondary)
    let project = create_polyglot_project();

    // Act: Analyze dead code
    let result = analyze_dead_code_multi_language(project.path());

    // Assert: Should use enhanced language detection from BUG-011 fix
    assert!(result.is_ok());
    let result = result.unwrap();

    // Should detect C++ as primary (has CMakeLists.txt)
    assert_eq!(
        result.language, "cpp",
        "Should use enhanced_language_detection to identify C++ as primary"
    );
}

// =============================================================================
// RED TEST 6: Dead Code Percentage Calculation
// =============================================================================

#[test]
#[ignore = "BUG-004: RED test - percentage calculation"]
fn test_dead_code_percentage_calculation() {
    // Arrange: Project with known dead code ratio (1 dead out of 3 functions)
    let project = create_project_with_known_ratio();

    // Act: Analyze dead code
    let result = analyze_dead_code_multi_language(project.path()).unwrap();

    // Assert: Should calculate percentage correctly
    assert_eq!(result.total_functions, 3, "Should find 3 total functions");
    assert_eq!(result.dead_functions.len(), 1, "Should find 1 dead function");

    let expected_percentage = (1.0 / 3.0) * 100.0;
    assert!(
        (result.dead_code_percentage - expected_percentage).abs() < 0.1,
        "Dead code percentage should be ~33.3%, got {}",
        result.dead_code_percentage
    );
}

// =============================================================================
// Mock Project Creators
// =============================================================================

fn create_c_project_with_dead_code() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let base = temp.path();

    fs::create_dir_all(base.join("src")).unwrap();
    fs::create_dir_all(base.join("include")).unwrap();

    // main.c
    fs::write(
        base.join("src/main.c"),
        r#"
#include <stdio.h>
#include "utils.h"

int main() {
    used_function();
    return 0;
}
"#,
    )
    .unwrap();

    // utils.c with dead code
    fs::write(
        base.join("src/utils.c"),
        r#"
#include <stdio.h>
#include "utils.h"

void used_function() {
    printf("Used\n");
}

void unused_function() {
    printf("DEAD CODE!\n");
}
"#,
    )
    .unwrap();

    // utils.h
    fs::write(
        base.join("include/utils.h"),
        r#"
#ifndef UTILS_H
#define UTILS_H

void used_function();
void unused_function();

#endif
"#,
    )
    .unwrap();

    // Makefile (C projects often use Make instead of CMake)
    fs::write(
        base.join("Makefile"),
        "CC=gcc\nCFLAGS=-Wall\n\nall: main\n",
    )
    .unwrap();

    temp
}

fn create_cpp_project_with_dead_code() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let base = temp.path();

    fs::create_dir_all(base.join("src")).unwrap();

    // main.cpp
    fs::write(
        base.join("src/main.cpp"),
        r#"
#include <iostream>

void used_function() {
    std::cout << "Used" << std::endl;
}

void unused_function() {
    std::cout << "DEAD CODE!" << std::endl;
}

int main() {
    used_function();
    return 0;
}
"#,
    )
    .unwrap();

    // CMakeLists.txt
    fs::write(
        base.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(Test CXX)\n",
    )
    .unwrap();

    temp
}

fn create_python_project_with_dead_code() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // main.py
    fs::write(
        base.join("main.py"),
        r#"
from utils import used_function

def main():
    used_function()

if __name__ == "__main__":
    main()
"#,
    )
    .unwrap();

    // utils.py with dead code
    fs::write(
        base.join("utils.py"),
        r#"
def used_function():
    print("Used")

def unused_function():
    print("DEAD CODE!")
"#,
    )
    .unwrap();

    // pyproject.toml
    fs::write(
        base.join("pyproject.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    temp
}

fn create_rust_project_with_dead_code() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let base = temp.path();

    fs::create_dir_all(base.join("src")).unwrap();

    // main.rs
    fs::write(
        base.join("src/main.rs"),
        r#"
fn used_function() {
    println!("Used");
}

fn unused_function() {
    println!("DEAD CODE!");
}

fn main() {
    used_function();
}
"#,
    )
    .unwrap();

    // Cargo.toml
    fs::write(
        base.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    temp
}

fn create_fortran_project() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // main.f90
    fs::write(
        base.join("main.f90"),
        r#"
program main
    print *, "Hello"
end program main
"#,
    )
    .unwrap();

    temp
}

fn create_polyglot_project() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let base = temp.path();

    fs::create_dir_all(base.join("src")).unwrap();

    // C++ files (primary)
    for i in 0..70 {
        fs::write(
            base.join(format!("src/file_{}.cpp", i)),
            "int main() { return 0; }",
        )
        .unwrap();
    }

    // Python files (secondary)
    for i in 0..30 {
        fs::write(
            base.join(format!("src/tool_{}.py", i)),
            "print('hello')",
        )
        .unwrap();
    }

    // CMakeLists.txt (indicates C++ primary)
    fs::write(
        base.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(Test CXX)\n",
    )
    .unwrap();

    temp
}

fn create_project_with_known_ratio() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let base = temp.path();

    fs::write(
        base.join("main.py"),
        r#"
def used1():
    print("Used 1")

def used2():
    print("Used 2")

def unused():
    print("DEAD CODE!")

def main():
    used1()
    used2()

if __name__ == "__main__":
    main()
"#,
    )
    .unwrap();

    fs::write(base.join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();

    temp
}
