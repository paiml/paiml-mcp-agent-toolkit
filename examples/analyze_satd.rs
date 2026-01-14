//! Analyze SATD (Self-Admitted Technical Debt) Example
//!
//! This example demonstrates how to use pmat's SATD analysis command
//! with the new --fail-on-violation flag for CI/CD integration.
//!
//! Run with: `cargo run --example analyze_satd`

use anyhow::Result;
use pmat::cli::handlers::complexity_handlers::handle_analyze_satd;
use pmat::cli::{SatdOutputFormat, SatdSeverity};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Analyze SATD (Technical Debt) Example\n");

    // Example 1: Basic SATD analysis
    println!("Example 1: Basic SATD analysis");
    println!("{}", "=".repeat(60));

    let result = handle_analyze_satd(
        PathBuf::from("."),
        SatdOutputFormat::Summary,
        None,  // All severities
        false, // Not critical only
        false, // Exclude tests
        false, // Not strict mode
        false, // No evolution tracking
        30,    // 30 days for evolution
        true,  // Include metrics
        None,  // Output to stdout
        10,    // Top 10 files
        false, // Don't fail on violation
        60,    // Timeout in seconds
    )
    .await;

    match result {
        Ok(_) => println!("✅ SATD analysis completed\n"),
        Err(e) => println!("❌ Analysis failed: {}\n", e),
    }

    // Example 2: Zero-tolerance CI/CD mode
    println!("\nExample 2: Zero-tolerance mode for CI/CD");
    println!("{}", "=".repeat(60));

    let strict_result = handle_analyze_satd(
        PathBuf::from("."),
        SatdOutputFormat::Json, // JSON for CI parsing
        None,                   // All severities
        false,                  // All debt, not just critical
        false,                  // Exclude tests
        true,                   // STRICT MODE - catches all debt patterns
        false,                  // No evolution
        30,
        true, // Include metrics
        None,
        0,     // Check all files
        false, // Don't fail on violation in example (to avoid CI failure)
        60,    // Timeout in seconds
    )
    .await;

    match strict_result {
        Ok(_) => println!("✅ SATD analysis completed!"),
        Err(e) => {
            println!("❌ Analysis failed: {}", e);
            return Err(e);
        }
    }

    println!("Note: In real CI, you would use --fail-on-violation to exit(1) on any debt found");

    // Example 3: Critical debt only
    println!("\nExample 3: Check for critical technical debt only");
    println!("{}", "=".repeat(60));

    let critical_result = handle_analyze_satd(
        PathBuf::from("."),
        SatdOutputFormat::Summary,
        Some(SatdSeverity::Critical), // Only critical severity
        true,                         // Critical only flag too
        false,                        // Exclude tests
        false,                        // Normal mode
        false,                        // No evolution
        30,
        false, // No detailed metrics
        None,
        5,     // Top 5 files
        false, // Don't fail on violation in example
        60,    // Timeout in seconds
    )
    .await;

    match critical_result {
        Ok(_) => println!("✅ Critical debt analysis completed!"),
        Err(e) => {
            println!("❌ Critical debt analysis failed: {}", e);
            return Err(e);
        }
    }

    // Example 4: Evolution tracking
    println!("\nExample 4: Track debt evolution over time");
    println!("{}", "=".repeat(60));

    let evolution_result = handle_analyze_satd(
        PathBuf::from("."),
        SatdOutputFormat::Summary,
        None,
        false,
        false,
        false,
        true, // Enable evolution tracking
        60,   // Look back 60 days
        true, // Include metrics
        None,
        10,
        false,
        60, // Timeout in seconds
    )
    .await;

    match evolution_result {
        Ok(_) => println!("✅ Debt evolution analysis complete!"),
        Err(e) => println!("❌ Evolution analysis failed: {}", e),
    }

    // Example 5: Save detailed report
    println!("\nExample 5: Save detailed SATD report");
    println!("{}", "=".repeat(60));

    let output_path = PathBuf::from("satd-report.json");
    let report_result = handle_analyze_satd(
        PathBuf::from("."),
        SatdOutputFormat::Json,
        None, // All severities
        false,
        true, // Include tests this time
        true, // Strict mode for comprehensive detection
        false,
        30,
        true, // Detailed metrics
        Some(output_path.clone()),
        0, // All files
        false,
        60, // Timeout in seconds
    )
    .await;

    match report_result {
        Ok(_) => println!("✅ SATD report saved to: {}", output_path.display()),
        Err(e) => println!("❌ Failed to save report: {}", e),
    }

    // Example 6: GitHub Actions usage
    println!("\nExample 6: GitHub Actions CI integration");
    println!("{}", "=".repeat(60));
    println!("In your GitHub Actions workflow, use:");
    println!("```yaml");
    println!("- name: Check for technical debt");
    println!("  run: |");
    println!("    # Fail if ANY technical debt is found");
    println!("    pmat analyze satd \\");
    println!("      --strict \\");
    println!("      --fail-on-violation \\");
    println!("      --format json");
    println!();
    println!("    # Or allow some debt but fail on critical");
    println!("    pmat analyze satd \\");
    println!("      --critical-only \\");
    println!("      --fail-on-violation");
    println!("```");
    println!("This enforces a zero-tolerance policy for technical debt.");

    println!("\n🎉 SATD analysis examples completed!");
    Ok(())
}
