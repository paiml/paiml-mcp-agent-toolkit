//! Analyze Dead Code Example
//!
//! This example demonstrates how to use pmat's dead code analysis command
//! with the new --fail-on-violation flag for CI/CD integration.
//!
//! Run with: `cargo run --example analyze_dead_code`

use anyhow::Result;
use pmat::cli::handlers::complexity_handlers::handle_analyze_dead_code;
use pmat::cli::DeadCodeOutputFormat;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Analyze Dead Code Example\n");

    // Example 1: Basic dead code analysis
    println!("Example 1: Basic dead code analysis");
    println!("{}", "=".repeat(60));

    let result = handle_analyze_dead_code(
        PathBuf::from("."),
        DeadCodeOutputFormat::Summary,
        Some(10), // Top 10 files
        true,     // Include unreachable code
        5,        // Min 5 dead lines to report
        false,    // Exclude tests
        None,     // Output to stdout
        false,    // Don't fail on violation
        15.0,     // Default max percentage
        60,       // Timeout in seconds
        vec![],   // Include patterns
        vec![],   // Exclude patterns
        8,        // Max depth
    )
    .await;

    match result {
        Ok(_) => println!("✅ Dead code analysis completed\n"),
        Err(e) => println!("❌ Analysis failed: {}\n", e),
    }

    // Example 2: Strict CI/CD mode with custom threshold
    println!("\nExample 2: CI/CD mode with strict dead code threshold");
    println!("{}", "=".repeat(60));

    let strict_result = handle_analyze_dead_code(
        PathBuf::from("."),
        DeadCodeOutputFormat::Json, // JSON for CI parsing
        None,                       // Analyze all files
        true,                       // Include unreachable
        3,                          // Report even small dead code blocks
        false,                      // Exclude tests
        None,                       // Output to stdout
        false,                      // Don't fail on violation in example (to avoid CI failure)
        5.0,                        // Max 5% dead code allowed (strict!)
        60,                         // Timeout in seconds
        vec![],                     // Include patterns
        vec![],                     // Exclude patterns
        8,                          // Max depth
    )
    .await;

    match strict_result {
        Ok(_) => println!("✅ Dead code analysis completed!"),
        Err(e) => {
            println!("❌ Analysis failed: {}", e);
            return Err(e);
        }
    }

    println!(
        "Note: In real CI, you would use --fail-on-violation to exit(1) on threshold exceeded"
    );

    // Example 3: Summary format with moderate threshold
    println!("\nExample 3: Summary format with moderate threshold");
    println!("{}", "=".repeat(60));

    let summary_result = handle_analyze_dead_code(
        PathBuf::from("."),
        DeadCodeOutputFormat::Summary,
        Some(5), // Top 5 files
        false,   // Don't include unreachable
        10,      // Only report significant dead code
        false,   // Exclude tests
        None,
        false,  // Don't fail
        10.0,   // 10% threshold (moderate)
        60,     // Timeout in seconds
        vec![], // Include patterns
        vec![], // Exclude patterns
        8,      // Max depth
    )
    .await;

    match summary_result {
        Ok(_) => println!("✅ Summary analysis complete!"),
        Err(e) => println!("❌ Summary analysis failed: {}", e),
    }

    // Example 4: Save results to file
    println!("\nExample 4: Save dead code report to file");
    println!("{}", "=".repeat(60));

    let output_path = PathBuf::from("dead-code-report.json");
    let file_result = handle_analyze_dead_code(
        PathBuf::from("."),
        DeadCodeOutputFormat::Json,
        None, // All files
        true, // Include unreachable
        5,    // Standard threshold
        true, // Include tests this time
        Some(output_path.clone()),
        false,
        15.0,   // Default threshold
        60,     // Timeout in seconds
        vec![], // Include patterns
        vec![], // Exclude patterns
        8,      // Max depth
    )
    .await;

    match file_result {
        Ok(_) => println!("✅ Dead code report saved to: {}", output_path.display()),
        Err(e) => println!("❌ Failed to save report: {}", e),
    }

    // Example 5: GitHub Actions usage
    println!("\nExample 5: GitHub Actions CI integration");
    println!("{}", "=".repeat(60));
    println!("In your GitHub Actions workflow, use:");
    println!("```yaml");
    println!("- name: Check for dead code");
    println!("  run: |");
    println!("    pmat analyze dead-code \\");
    println!("      --max-percentage 10.0 \\");
    println!("      --fail-on-violation \\");
    println!("      --format json \\");
    println!("      --include-unreachable");
    println!("```");
    println!("This ensures your codebase maintains less than 10% dead code.");

    println!("\n🎉 Dead code analysis examples completed!");
    Ok(())
}
