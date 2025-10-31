//! BUG-004: Multi-Language Dead Code Analysis
//!
//! This example demonstrates the fix for BUG-004 where dead code analysis
//! now works for C/C++/Python projects without requiring Cargo.toml.
//!
//! Run with: `cargo run --example bug_004_dead_code_c_project`

use anyhow::Result;
use pmat::services::dead_code_multi_language::analyze_dead_code_multi_language;
use std::fs;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("🐛 BUG-004: Multi-Language Dead Code Analysis\n");

    // Example 1: C Project (no Cargo.toml required!)
    println!("Example 1: C project dead code analysis");
    println!("{}", "=".repeat(60));

    let c_project = create_c_project()?;
    println!("Created C project at: {:?}", c_project.path());
    println!("Files:");
    println!("  - src/main.c (calls used_function)");
    println!("  - src/utils.c (defines used_function and unused_function)");
    println!("  - include/utils.h (declarations)");
    println!("  - Makefile (C project indicator)");
    println!("  - NO Cargo.toml!");

    println!("\n🔍 Running dead code analysis...");

    let result = analyze_dead_code_multi_language(c_project.path())?;

    println!("✅ Analysis successful!");
    println!("  Language: {}", result.language);
    println!("  Total functions: {}", result.total_functions);
    println!("  Dead functions: {}", result.dead_functions.len());
    println!("  Dead code %: {:.1}%", result.dead_code_percentage);

    println!("\n📊 Dead Functions:");
    for func in &result.dead_functions {
        println!("  - {} ({}:{})", func.name, func.file, func.line);
        println!("    Reason: {}", func.reason);
    }

    // Example 2: Python Project
    println!("\n\nExample 2: Python project dead code analysis");
    println!("{}", "=".repeat(60));

    let py_project = create_python_project()?;
    println!("Created Python project at: {:?}", py_project.path());

    let result = analyze_dead_code_multi_language(py_project.path())?;

    println!("✅ Analysis successful!");
    println!("  Language: {}", result.language);
    println!("  Dead functions: {}", result.dead_functions.len());

    println!("\n✅ BUG-004 FIXED: Multi-language dead code analysis works!");

    Ok(())
}

fn create_c_project() -> Result<TempDir> {
    let temp = TempDir::new()?;
    let base = temp.path();

    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("include"))?;

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
    )?;

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

    // Makefile (C project indicator)
    fs::write(base.join("Makefile"), "CC=gcc\nCFLAGS=-Wall\n\nall: main\n")?;

    Ok(temp)
}

fn create_python_project() -> Result<TempDir> {
    let temp = TempDir::new()?;
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
    )?;

    // utils.py with dead code
    fs::write(
        base.join("utils.py"),
        r#"
def used_function():
    print("Used")

def unused_function():
    print("DEAD CODE!")
"#,
    )?;

    // pyproject.toml
    fs::write(base.join("pyproject.toml"), "[project]\nname = \"test\"\n")?;

    Ok(temp)
}
