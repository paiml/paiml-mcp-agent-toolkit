//! Analyze SATD (Self-Admitted Technical Debt) Example
//!
//! This example demonstrates how to use pmat's SATD analysis command
//! with the new --extended flag for detecting euphemisms (issue #149)
//! and --fail-on-violation flag for CI/CD integration.
//!
//! Run with: `cargo run --example analyze_satd`

use anyhow::Result;
use pmat::cli::handlers::satd_handler::{handle_analyze_satd, SatdAnalysisConfig};
use pmat::cli::{SatdOutputFormat, SatdSeverity};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Analyze SATD (Technical Debt) Example\n");

    // Example 1: Basic SATD analysis
    println!("Example 1: Basic SATD analysis");
    println!("{}", "=".repeat(60));

    let config = SatdAnalysisConfig {
        path: PathBuf::from("."),
        format: SatdOutputFormat::Summary,
        severity: None,        // All severities
        critical_only: false,  // Not critical only
        include_tests: false,  // Exclude tests
        strict: false,         // Not strict mode
        evolution: false,      // No evolution tracking
        days: 30,              // 30 days for evolution
        metrics: true,         // Include metrics
        output: None,          // Output to stdout
        top_files: 10,         // Top 10 files
        fail_on_violation: false, // Don't fail on violation
        timeout: 60,           // Timeout in seconds
        include: vec![],       // No include patterns
        exclude: vec![],       // No exclude patterns
        extended: false,       // Standard detection only
    };

    match handle_analyze_satd(config).await {
        Ok(_) => println!("✅ SATD analysis completed\n"),
        Err(e) => println!("❌ Analysis failed: {}\n", e),
    }

    // Example 2: Extended mode - Detect euphemisms (Issue #149)
    println!("\nExample 2: Extended mode - Detect AI-generated euphemisms");
    println!("{}", "=".repeat(60));
    println!("Extended mode detects hidden technical debt:");
    println!("  - placeholder, stub, simplified");
    println!("  - mock, dummy, fake, hardcoded");
    println!("  - 'for now', WIP, skip/bypass\n");

    let extended_config = SatdAnalysisConfig {
        path: PathBuf::from("src"),
        format: SatdOutputFormat::Summary,
        severity: None,
        critical_only: false,
        include_tests: false,
        strict: false,
        evolution: false,
        days: 30,
        metrics: true,
        output: None,
        top_files: 10,
        fail_on_violation: false,
        timeout: 120,
        include: vec![],
        exclude: vec![],
        extended: true, // EXTENDED MODE - catches euphemisms!
    };

    match handle_analyze_satd(extended_config).await {
        Ok(_) => println!("✅ Extended SATD analysis completed!\n"),
        Err(e) => println!("❌ Extended analysis failed: {}\n", e),
    }

    // Example 3: Zero-tolerance CI/CD mode
    println!("\nExample 3: Zero-tolerance mode for CI/CD");
    println!("{}", "=".repeat(60));

    let ci_config = SatdAnalysisConfig {
        path: PathBuf::from("."),
        format: SatdOutputFormat::Json, // JSON for CI parsing
        severity: None,
        critical_only: false,
        include_tests: false,
        strict: true, // STRICT MODE - catches all debt patterns
        evolution: false,
        days: 30,
        metrics: true,
        output: None,
        top_files: 0, // Check all files
        fail_on_violation: false, // Don't fail in example
        timeout: 60,
        include: vec![],
        exclude: vec!["target/**".to_string()],
        extended: true, // Also use extended detection
    };

    match handle_analyze_satd(ci_config).await {
        Ok(_) => println!("✅ CI/CD analysis completed!"),
        Err(e) => println!("❌ Analysis failed: {}", e),
    }

    println!("Note: In real CI, use --fail-on-violation to exit(1) on any debt found");

    // Example 4: Critical debt only
    println!("\nExample 4: Check for critical technical debt only");
    println!("{}", "=".repeat(60));

    let critical_config = SatdAnalysisConfig {
        path: PathBuf::from("."),
        format: SatdOutputFormat::Summary,
        severity: Some(SatdSeverity::Critical),
        critical_only: true,
        include_tests: false,
        strict: false,
        evolution: false,
        days: 30,
        metrics: false,
        output: None,
        top_files: 5,
        fail_on_violation: false,
        timeout: 60,
        include: vec![],
        exclude: vec![],
        extended: false,
    };

    match handle_analyze_satd(critical_config).await {
        Ok(_) => println!("✅ Critical debt analysis completed!"),
        Err(e) => println!("❌ Critical debt analysis failed: {}", e),
    }

    // Example 5: Evolution tracking
    println!("\nExample 5: Track debt evolution over time");
    println!("{}", "=".repeat(60));

    let evolution_config = SatdAnalysisConfig {
        path: PathBuf::from("."),
        format: SatdOutputFormat::Summary,
        severity: None,
        critical_only: false,
        include_tests: false,
        strict: false,
        evolution: true, // Enable evolution tracking
        days: 60,        // Look back 60 days
        metrics: true,
        output: None,
        top_files: 10,
        fail_on_violation: false,
        timeout: 60,
        include: vec![],
        exclude: vec![],
        extended: false,
    };

    match handle_analyze_satd(evolution_config).await {
        Ok(_) => println!("✅ Debt evolution analysis complete!"),
        Err(e) => println!("❌ Evolution analysis failed: {}", e),
    }

    // Example 6: Save detailed report with file filters
    println!("\nExample 6: Save detailed SATD report with filters");
    println!("{}", "=".repeat(60));

    let output_path = PathBuf::from("satd-report.json");
    let report_config = SatdAnalysisConfig {
        path: PathBuf::from("."),
        format: SatdOutputFormat::Json,
        severity: None,
        critical_only: false,
        include_tests: true, // Include tests this time
        strict: true,
        evolution: false,
        days: 30,
        metrics: true,
        output: Some(output_path.clone()),
        top_files: 0, // All files
        fail_on_violation: false,
        timeout: 60,
        include: vec!["src/**/*.rs".to_string()], // Only Rust source files
        exclude: vec!["**/tests/**".to_string()], // Exclude test directories
        extended: true,                            // Use extended detection
    };

    match handle_analyze_satd(report_config).await {
        Ok(_) => println!("✅ SATD report saved to: {}", output_path.display()),
        Err(e) => println!("❌ Failed to save report: {}", e),
    }

    // Example 7: CLI usage reference
    println!("\nExample 7: CLI Usage Reference");
    println!("{}", "=".repeat(60));
    println!("# Standard SATD detection");
    println!("pmat analyze satd --path src/");
    println!();
    println!("# Extended mode (detect euphemisms like 'placeholder', 'stub')");
    println!("pmat analyze satd --extended --path src/");
    println!();
    println!("# CI/CD zero-tolerance (fail on any debt)");
    println!("pmat analyze satd --strict --extended --fail-on-violation");
    println!();
    println!("# Critical debt only");
    println!("pmat analyze satd --critical-only --fail-on-violation");

    // Example 8: GitHub Actions usage
    println!("\nExample 8: GitHub Actions CI integration");
    println!("{}", "=".repeat(60));
    println!("```yaml");
    println!("- name: Check for technical debt");
    println!("  run: |");
    println!("    # Fail if ANY technical debt is found (including euphemisms)");
    println!("    pmat analyze satd \\");
    println!("      --extended \\");
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
