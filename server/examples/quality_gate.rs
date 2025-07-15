//! Quality Gate Example
//!
//! This example demonstrates how to use pmat's quality gate functionality
//! to enforce code quality standards in a project.
//!
//! Run with: `cargo run --example quality_gate`

use anyhow::Result;
use pmat::cli::stubs::handle_quality_gate;
use pmat::cli::enums::{QualityCheckType, QualityGateOutputFormat};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Quality Gate Example\n");

    // Example 1: Run all quality checks on current directory
    println!("Example 1: Running all quality checks on current directory");
    println!("{}", "=".repeat(60));
    
    let result = handle_quality_gate(
        PathBuf::from("."),
        None, // No specific file
        QualityGateOutputFormat::Summary,
        false, // Don't fail on violation
        vec![], // All checks
        15.0,  // Max 15% dead code
        2.0,   // Min entropy of 2.0
        50,    // Max cyclomatic complexity of 50
        false, // Don't include provability
        None,  // No output file
        false, // No performance mode
    ).await;

    match result {
        Ok(_) => println!("✅ Quality gate passed!\n"),
        Err(e) => println!("❌ Quality gate failed: {}\n", e),
    }

    // Example 2: Run specific checks with stricter thresholds
    println!("\nExample 2: Running specific checks with stricter thresholds");
    println!("{}", "=".repeat(60));
    
    let strict_result = handle_quality_gate(
        PathBuf::from("."),
        None,
        QualityGateOutputFormat::Human,
        false, // Don't fail on violation in example (to avoid CI failure)
        vec![
            QualityCheckType::Complexity,
            QualityCheckType::DeadCode,
            QualityCheckType::Satd,
        ],
        5.0,  // Max 5% dead code (stricter)
        3.0,  // Min entropy of 3.0 (stricter)
        20,   // Max cyclomatic complexity of 20 (stricter)
        false,
        None,
        false,
    ).await;

    match strict_result {
        Ok(_) => println!("✅ Strict quality gate passed!"),
        Err(e) => println!("⚠️  Strict quality gate failed (expected): {}", e),
    }

    // Example 3: Analyze a specific file
    println!("\nExample 3: Analyzing a specific file");
    println!("{}", "=".repeat(60));
    
    let file_result = handle_quality_gate(
        PathBuf::from("."),
        Some(PathBuf::from("src/main.rs")),
        QualityGateOutputFormat::Json,
        false,
        vec![QualityCheckType::Complexity],
        15.0,
        2.0,
        30, // Max complexity for single file
        false,
        None,
        false,
    ).await;

    match file_result {
        Ok(_) => println!("✅ Single file analysis complete!"),
        Err(e) => println!("❌ Single file analysis failed: {}", e),
    }

    // Example 4: Save results to file
    println!("\nExample 4: Saving quality gate results to file");
    println!("{}", "=".repeat(60));
    
    let output_path = PathBuf::from("quality-report.json");
    let output_result = handle_quality_gate(
        PathBuf::from("."),
        None,
        QualityGateOutputFormat::Json,
        false,
        vec![],
        15.0,
        2.0,
        50,
        true, // Include provability score
        Some(output_path.clone()),
        false,
    ).await;

    match output_result {
        Ok(_) => println!("✅ Quality report saved to: {}", output_path.display()),
        Err(e) => println!("❌ Failed to save report: {}", e),
    }

    println!("\n🎉 Quality gate examples completed!");
    Ok(())
}