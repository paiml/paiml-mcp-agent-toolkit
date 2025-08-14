//! Example demonstrating the use of quality check functions
//!
//! This example shows how to use check_dead_code, check_entropy, and calculate_provability_score
//! to assess code quality in a project.

use anyhow::Result;
use pmat::cli::stubs::{calculate_provability_score, check_dead_code, check_entropy};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Code Quality Check Example\n");

    // Get the project path from command line or use current directory
    let project_path = std::env::args()
        .nth(1)
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());

    println!("Analyzing project at: {}\n", project_path.display());

    // Check dead code
    println!("1️⃣ Checking for dead code (max 15% threshold)...");
    match check_dead_code(&project_path, 15.0).await {
        Ok(violations) => {
            if violations.is_empty() {
                println!("   ✅ No dead code violations found!");
            } else {
                println!("   ⚠️  Found {} dead code violations:", violations.len());
                for (i, violation) in violations.iter().take(5).enumerate() {
                    println!(
                        "      {}. {} - {}",
                        i + 1,
                        violation.file,
                        violation.message
                    );
                }
                if violations.len() > 5 {
                    println!("      ... and {} more", violations.len() - 5);
                }
            }
        }
        Err(e) => println!("   ❌ Error checking dead code: {}", e),
    }

    println!();

    // Check code entropy (diversity)
    println!("2️⃣ Checking code entropy (min 0.7 threshold)...");
    match check_entropy(&project_path, 0.7).await {
        Ok(violations) => {
            if violations.is_empty() {
                println!("   ✅ Code diversity is good!");
            } else {
                println!("   ⚠️  Found {} low entropy files:", violations.len());
                for (i, violation) in violations.iter().take(5).enumerate() {
                    println!(
                        "      {}. {} - {}",
                        i + 1,
                        violation.file,
                        violation.message
                    );
                }
                if violations.len() > 5 {
                    println!("      ... and {} more", violations.len() - 5);
                }
            }
        }
        Err(e) => println!("   ❌ Error checking entropy: {}", e),
    }

    println!();

    // Calculate provability score
    println!("3️⃣ Calculating provability score...");
    match calculate_provability_score(&project_path).await {
        Ok(score) => {
            println!("   📊 Provability score: {:.2}", score);
            let interpretation = match score {
                s if s >= 0.9 => "Excellent! Code is highly amenable to formal verification",
                s if s >= 0.7 => "Good - Most code can be formally verified",
                s if s >= 0.5 => "Moderate - Consider refactoring complex functions",
                _ => "Low - Significant refactoring needed for formal verification",
            };
            println!("   💡 {}", interpretation);
        }
        Err(e) => println!("   ❌ Error calculating provability: {}", e),
    }

    println!("\n✨ Code quality check complete!");

    Ok(())
}
