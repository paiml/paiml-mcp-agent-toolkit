//! BUG-004: Dead Code Analysis Requires Cargo.toml for Non-Rust Projects
//!
//! This example reproduces BUG-004 where:
//! 1. Dead code analysis fails on C/C++ projects without Cargo.toml
//! 2. Error: "could not find `Cargo.toml` in ... or any parent directory"
//! 3. Feature completely broken for non-Rust projects
//!
//! Expected behavior:
//! - Should detect project language (C, C++, Python, etc.)
//! - Should use appropriate dead code detection for that language
//! - Should NOT require Cargo.toml for non-Rust projects
//!
//! Run with: `cargo run --example bug_004_dead_code_c_project`

use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🐛 BUG-004: Dead Code Analysis Requires Cargo.toml\n");

    // Example 1: Reproduce the bug - C project without Cargo.toml
    println!("Example 1: C project dead code analysis (reproduces bug)");
    println!("{}", "=".repeat(60));

    let c_project = create_mock_c_project().await?;
    println!("Created mock C project at: {:?}", c_project.path());
    println!("Files:");
    println!("  - src/main.c (has main function)");
    println!("  - src/utils.c (has unused_function)");
    println!("  - include/utils.h (header)");
    println!("  - CMakeLists.txt (C project indicator)");
    println!("  - NO Cargo.toml (not a Rust project!)");

    println!("\n🔍 Attempting dead code analysis...");

    // This will fail with current implementation
    match analyze_dead_code_current(c_project.path()).await {
        Ok(result) => {
            println!("✅ Analysis succeeded: {}", result);
        }
        Err(e) => {
            println!("❌ BUG REPRODUCED: {}", e);
            println!("   Error mentions Cargo.toml even though this is a C project!");
        }
    }

    // Example 2: What the fix should enable
    println!("\n\nExample 2: Multi-language dead code detection (after fix)");
    println!("{}", "=".repeat(60));
    println!("After fix, this should work:");
    println!("  1. Detect language: C (from CMakeLists.txt and .c files)");
    println!("  2. Use C-appropriate dead code detection:");
    println!("     - AST-based analysis (tree-sitter)");
    println!("     - Or call graph analysis");
    println!("     - Or cppcheck integration");
    println!("  3. Report: unused_function is dead code");

    // Example 3: Python project
    println!("\n\nExample 3: Python project (also broken)");
    println!("{}", "=".repeat(60));

    let py_project = create_mock_python_project().await?;
    println!("Created mock Python project at: {:?}", py_project.path());
    println!("Files:");
    println!("  - main.py (imports utils)");
    println!("  - utils.py (has unused_function)");
    println!("  - pyproject.toml (Python project indicator)");

    println!("\n🔍 Attempting dead code analysis...");

    match analyze_dead_code_current(py_project.path()).await {
        Ok(result) => {
            println!("✅ Analysis succeeded: {}", result);
        }
        Err(e) => {
            println!("❌ BUG REPRODUCED: {}", e);
            println!("   Python project also fails!");
        }
    }

    println!("\n🎯 To fix this bug:");
    println!("  1. Add language detection to dead code analyzer");
    println!("  2. Create DeadCodeStrategy trait for language-specific analysis");
    println!("  3. Implement C/C++ dead code strategy (AST-based)");
    println!("  4. Implement Python dead code strategy (AST-based)");
    println!("  5. Only use cargo check for Rust projects");

    Ok(())
}

/// Current dead code analysis (has the bug)
async fn analyze_dead_code_current(path: &std::path::Path) -> Result<String> {
    // Try to use the existing dead code analyzer
    // This will fail for non-Rust projects

    use pmat::cli::handlers::analyze_handlers::handle_analyze_dead_code;

    // Current implementation requires Cargo.toml
    handle_analyze_dead_code(
        path.to_path_buf(),
        60, // timeout
        std::io::stdout(),
    ).await?;

    Ok("Analysis completed".to_string())
}

/// Create a mock C project
async fn create_mock_c_project() -> Result<TempDir> {
    use std::fs;

    let temp = TempDir::new()?;
    let base = temp.path();

    // Create C source files
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("include"))?;

    // main.c - uses only used_function
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
    )?;

    // utils.c - has one used and one unused function
    fs::write(
        base.join("src/utils.c"),
        r#"
#include <stdio.h>
#include "utils.h"

void used_function() {
    printf("This is used\n");
}

void unused_function() {
    printf("This is NEVER called - DEAD CODE!\n");
}
"#,
    )?;

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
    )?;

    // CMakeLists.txt (indicates C project)
    fs::write(
        base.join("CMakeLists.txt"),
        r#"
cmake_minimum_required(VERSION 3.10)
project(TestProject C)

add_executable(main src/main.c src/utils.c)
target_include_directories(main PRIVATE include)
"#,
    )?;

    Ok(temp)
}

/// Create a mock Python project
async fn create_mock_python_project() -> Result<TempDir> {
    use std::fs;

    let temp = TempDir::new()?;
    let base = temp.path();

    // main.py - uses only used_function
    fs::write(
        base.join("main.py"),
        r#"
from utils import used_function

def main():
    used_function()

if __name__ == "__main__":
    main()
"#,
    )?;

    // utils.py - has one used and one unused function
    fs::write(
        base.join("utils.py"),
        r#"
def used_function():
    print("This is used")

def unused_function():
    print("This is NEVER called - DEAD CODE!")
"#,
    )?;

    // pyproject.toml (indicates Python project)
    fs::write(
        base.join("pyproject.toml"),
        r#"
[project]
name = "test-project"
version = "0.1.0"
"#,
    )?;

    Ok(temp)
}
