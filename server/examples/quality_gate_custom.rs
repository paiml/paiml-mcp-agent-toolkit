//! Example demonstrating custom quality gate implementation
//!
//! This example shows how to build a custom quality gate using the individual
//! quality check functions with custom thresholds and reporting.

use anyhow::Result;
use pmat::cli::analysis_utilities::{
    calculate_provability_score, check_dead_code, check_entropy, QualityViolation,
};
use std::collections::HashMap;
use std::path::Path;

#[derive(Default)]
struct QualityReport {
    passed: bool,
    checks: HashMap<String, CheckResult>,
}

struct CheckResult {
    passed: bool,
    score: Option<f64>,
    violations: Vec<QualityViolation>,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚦 Custom Quality Gate Example\n");

    let project_path = std::env::args()
        .nth(1)
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());

    println!("Running quality gate for: {}\n", project_path.display());

    let mut report = QualityReport {
        passed: true,
        ..Default::default()
    };

    // Define custom thresholds
    let dead_code_threshold = 10.0; // Max 10% dead code
    let entropy_threshold = 0.6; // Min entropy of 0.6
    let provability_threshold = 0.7; // Min provability of 0.7

    // Check 1: Dead Code
    println!("🔍 Checking dead code (max {}%)...", dead_code_threshold);
    match check_dead_code(&project_path, dead_code_threshold).await {
        Ok(violations) => {
            let passed = violations.is_empty();
            if !passed {
                report.passed = false;
            }

            let message = if passed {
                "✅ PASS: No excessive dead code found".to_string()
            } else {
                format!("❌ FAIL: Found {} dead code violations", violations.len())
            };

            println!("   {}", message);

            report.checks.insert(
                "dead_code".to_string(),
                CheckResult {
                    passed,
                    score: None,
                    violations,
                    message,
                },
            );
        }
        Err(e) => {
            println!("   ⚠️  WARNING: Could not check dead code: {}", e);
        }
    }

    // Check 2: Code Entropy
    println!("\n🔍 Checking code entropy (min {})...", entropy_threshold);
    match check_entropy(&project_path, entropy_threshold).await {
        Ok(violations) => {
            let passed = violations.is_empty();
            if !passed {
                report.passed = false;
            }

            let message = if passed {
                "✅ PASS: Code diversity is acceptable".to_string()
            } else {
                format!(
                    "❌ FAIL: {} files have low code diversity",
                    violations.len()
                )
            };

            println!("   {}", message);

            report.checks.insert(
                "entropy".to_string(),
                CheckResult {
                    passed,
                    score: None,
                    violations,
                    message,
                },
            );
        }
        Err(e) => {
            println!("   ⚠️  WARNING: Could not check entropy: {}", e);
        }
    }

    // Check 3: Provability Score
    println!(
        "\n🔍 Checking provability (min {})...",
        provability_threshold
    );
    match calculate_provability_score(&project_path).await {
        Ok(score) => {
            let passed = score >= provability_threshold;
            if !passed {
                report.passed = false;
            }

            let message = if passed {
                format!("✅ PASS: Provability score {:.2} meets threshold", score)
            } else {
                format!(
                    "❌ FAIL: Provability score {:.2} below threshold {}",
                    score, provability_threshold
                )
            };

            println!("   {}", message);

            report.checks.insert(
                "provability".to_string(),
                CheckResult {
                    passed,
                    score: Some(score),
                    violations: vec![],
                    message,
                },
            );
        }
        Err(e) => {
            println!("   ⚠️  WARNING: Could not check provability: {}", e);
        }
    }

    // Final Report
    println!("\n{}", "=".repeat(50));
    println!("📊 QUALITY GATE SUMMARY");
    println!("{}", "=".repeat(50));

    let passed_checks = report.checks.values().filter(|c| c.passed).count();
    let total_checks = report.checks.len();

    println!("Checks passed: {}/{}", passed_checks, total_checks);
    println!(
        "Overall status: {}",
        if report.passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    );

    // Display scores for checks that have them
    println!("\n📈 Check scores:");
    for (check_name, result) in &report.checks {
        if let Some(score) = result.score {
            println!("   - {}: {:.2}", check_name, score);
        }
    }

    if !report.passed {
        println!("\n📋 Failed checks:");
        for (check_name, result) in &report.checks {
            if !result.passed {
                println!("   - {}: {}", check_name, result.message);
                if !result.violations.is_empty() {
                    println!("     First few violations:");
                    for violation in result.violations.iter().take(3) {
                        println!("     • {} - {}", violation.file, violation.message);
                    }
                }
            }
        }

        // In a real CI/CD scenario, you would exit with non-zero code here
        // std::process::exit(1);
        println!("\n💡 Note: In CI/CD, this would exit with code 1");
    }

    Ok(())
}
