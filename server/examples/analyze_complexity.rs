//! Analyze Complexity Example
//!
//! This example demonstrates how to use pmat's complexity analysis command
//! with the new --fail-on-violation flag for CI/CD integration.
//!
//! Run with: `cargo run --example analyze_complexity`

use anyhow::Result;
use pmat::cli::handlers::complexity_handlers::handle_analyze_complexity;
use pmat::cli::ComplexityOutputFormat;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Analyze Complexity Example\n");

    // Example 1: Basic complexity analysis
    println!("Example 1: Basic complexity analysis of current directory");
    println!("{}", "=".repeat(60));
    
    let result = handle_analyze_complexity(
        PathBuf::from("."),
        None,    // No specific file
        vec![],  // No file list
        None,    // Auto-detect toolchain
        ComplexityOutputFormat::Full,
        None,    // Output to stdout
        None,    // Default max cyclomatic (20)
        None,    // Default max cognitive (15)
        vec![],  // No include patterns
        false,   // No watch mode
        10,      // Top 10 files
        false,   // Don't fail on violation
    ).await;

    match result {
        Ok(_) => println!("✅ Analysis completed successfully\n"),
        Err(e) => println!("❌ Analysis failed: {}\n", e),
    }

    // Example 2: Strict CI/CD mode with exit code
    println!("\nExample 2: CI/CD mode with strict thresholds and exit code");
    println!("{}", "=".repeat(60));
    
    let strict_result = handle_analyze_complexity(
        PathBuf::from("."),
        None,
        vec![],
        None,
        ComplexityOutputFormat::Json,  // JSON for CI parsing
        None,
        Some(10),  // Max cyclomatic complexity of 10
        Some(5),   // Max cognitive complexity of 5
        vec![String::from("src/**/*.rs")],  // Only analyze src files, not tests
        false,
        5,         // Top 5 files only
        false,     // Don't fail on violation in example (to avoid CI failure)
    ).await;

    match strict_result {
        Ok(_) => println!("✅ Analysis completed!"),
        Err(e) => {
            println!("❌ Analysis failed: {}", e);
            return Err(e);
        }
    }
    
    println!("Note: In real CI, you would use --fail-on-violation to exit(1) on violations");

    // Example 3: Analyze specific files
    println!("\nExample 3: Analyzing specific files with custom thresholds");
    println!("{}", "=".repeat(60));
    
    let specific_files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/cli/mod.rs"),
    ];
    
    let file_result = handle_analyze_complexity(
        PathBuf::from("."),
        None,
        specific_files,
        None,
        ComplexityOutputFormat::Summary,
        None,
        Some(15),  // Moderate threshold
        Some(10),
        vec![],
        false,
        0,         // Show all files (not just top N)
        false,
    ).await;

    match file_result {
        Ok(_) => println!("✅ File analysis complete!"),
        Err(e) => println!("❌ File analysis failed: {}", e),
    }

    // Example 4: GitHub Actions integration example
    println!("\nExample 4: GitHub Actions CI integration");
    println!("{}", "=".repeat(60));
    println!("In your GitHub Actions workflow, use:");
    println!("```yaml");
    println!("- name: Check code complexity");
    println!("  run: |");
    println!("    pmat analyze complexity \\");
    println!("      --max-cyclomatic 15 \\");
    println!("      --max-cognitive 10 \\");
    println!("      --fail-on-violation \\");
    println!("      --format json");
    println!("```");
    println!("This will fail the CI build if any function exceeds the thresholds.");

    println!("\n🎉 Complexity analysis examples completed!");
    Ok(())
}