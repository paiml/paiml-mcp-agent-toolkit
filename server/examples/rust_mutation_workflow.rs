// Rust mutation testing workflow example
// Run with: cargo run --example rust_mutation_workflow --features rust-ast
//
// Demonstrates:
// 1. Generating mutants from Rust source
// 2. Running baseline tests with cargo test
// 3. Testing each mutant
// 4. Calculating mutation score

use anyhow::{Context, Result};
use pmat::services::mutation::{RustMutationGenerator, MutantStatus};
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Rust Mutation Testing Workflow\n");

    // Configuration
    let source_file = PathBuf::from("../fixtures/rust/calculator.rs");
    let project_root = PathBuf::from("../fixtures/rust");

    // Step 1: Read source file
    println!("📝 Reading source file: {}", source_file.display());
    let source = std::fs::read_to_string(&source_file)
        .context("Failed to read source file")?;

    println!("   Size: {} bytes\n", source.len());

    // Step 2: Generate mutants
    println!("🔧 Generating mutants...");
    let generator = RustMutationGenerator::with_default_operators();

    let generation_start = Instant::now();
    let mut mutants = generator
        .generate_mutants(&source, source_file.to_str().unwrap())
        .context("Failed to generate mutants")?;
    let generation_time = generation_start.elapsed();

    println!("   Generated: {} mutants", mutants.len());
    println!("   Time: {:?}\n", generation_time);

    if mutants.is_empty() {
        println!("⚠️  No mutants generated!");
        return Ok(());
    }

    // Step 3: Check cargo availability
    println!("🏗️  Checking cargo availability...");
    let has_cargo = Command::new("cargo")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_cargo {
        println!("⚠️  Cargo not available, skipping test execution");
        println!("   Install Rust from: https://rustup.rs/");
        return Ok(());
    }

    // Step 4: Run baseline tests
    println!("✅ Running baseline tests...");
    let baseline_passed = run_tests(&project_root).await?;

    if !baseline_passed {
        println!("❌ Baseline tests failed! Fix tests before mutation testing.");
        return Ok(());
    }
    println!("   Baseline tests passed ✅\n");

    // Step 5: Test each mutant
    let total = mutants.len();
    println!("🧪 Testing {} mutants sequentially...\n", total);

    let test_start = Instant::now();

    for (i, mutant) in mutants.iter_mut().enumerate() {
        print!("   [{}/{}] Testing mutant: {} ... ", i + 1, total, mutant.id);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        match test_mutant(&source_file, &project_root, &mutant.mutated_source).await {
            Ok(false) => {
                // Tests failed = mutant killed
                mutant.status = MutantStatus::Killed;
                println!("🗡️  KILLED");
            }
            Ok(true) => {
                // Tests passed = mutant survived
                mutant.status = MutantStatus::Survived;
                println!("🧟 SURVIVED");
            }
            Err(_) => {
                // Timeout or error
                mutant.status = MutantStatus::Timeout;
                println!("⏱️  TIMEOUT/ERROR");
            }
        }
    }

    let test_time = test_start.elapsed();

    // Step 6: Calculate mutation score
    let killed_count = mutants.iter().filter(|m| m.status == MutantStatus::Killed).count();
    let survived_count = mutants.iter().filter(|m| m.status == MutantStatus::Survived).count();
    let timeout_count = mutants.iter().filter(|m| m.status == MutantStatus::Timeout).count();

    println!("\n📊 Mutation Testing Results\n");
    println!("   Total Mutants:    {}", total);
    println!("   Killed:           {} ({}%)",
        killed_count, (killed_count * 100) / total);
    println!("   Survived:         {} ({}%)",
        survived_count, (survived_count * 100) / total);
    println!("   Timeout/Error:    {}", timeout_count);

    let mutation_score = if total > timeout_count {
        (killed_count * 100) / (total - timeout_count)
    } else {
        0
    };

    println!("\n🎯 Mutation Score: {}%", mutation_score);

    if mutation_score >= 80 {
        println!("✅ EXCELLENT! Test suite quality is high.");
    } else if mutation_score >= 60 {
        println!("⚠️  GOOD, but room for improvement.");
    } else {
        println!("❌ WEAK test suite. Add more tests!");
    }

    // Performance stats
    println!("\n⚡ Performance:");
    println!("   Generation Time: {:?}", generation_time);
    println!("   Test Time:       {:?}", test_time);
    println!("   Total Time:      {:?}", generation_time + test_time);
    if total > 0 {
        println!("   Time per Mutant: {:?}", test_time / total as u32);
    }

    // Show surviving mutants
    if survived_count > 0 {
        println!("\n🧟 Surviving Mutants (weaknesses in tests):\n");
        let mut survivors: Vec<_> = mutants.iter()
            .filter(|m| m.status == MutantStatus::Survived)
            .collect();
        survivors.sort_by_key(|m| m.location.line);

        for mutant in survivors.iter().take(10) {
            println!("   • {} at line {}:{}",
                mutant.id,
                mutant.location.line,
                mutant.location.column
            );

            let lines: Vec<&str> = mutant.mutated_source.lines().collect();
            if mutant.location.line > 0 && mutant.location.line <= lines.len() {
                let line = lines[mutant.location.line - 1].trim();
                println!("     Code: {}\n", line);
            }
        }

        if survivors.len() > 10 {
            println!("   ... and {} more\n", survivors.len() - 10);
        }
    }

    println!("🎉 Mutation testing complete!");

    Ok(())
}

/// Run tests in the Rust project using cargo test
async fn run_tests(project_root: &PathBuf) -> Result<bool> {
    let output = Command::new("cargo")
        .arg("test")
        .arg("--")
        .arg("--test-threads=1")
        .current_dir(project_root)
        .output()
        .await
        .context("Failed to run tests")?;

    Ok(output.status.success())
}

/// Test a single mutant by temporarily replacing the source file
async fn test_mutant(
    source_file: &PathBuf,
    project_root: &PathBuf,
    mutated_source: &str,
) -> Result<bool> {
    // Create backup
    let backup_path = source_file.with_extension("rs.backup");
    std::fs::copy(source_file, &backup_path)
        .context("Failed to create backup")?;

    // Write mutant
    std::fs::write(source_file, mutated_source)
        .context("Failed to write mutated source")?;

    // Run tests
    let test_result = Command::new("cargo")
        .arg("test")
        .arg("--")
        .arg("--test-threads=1")
        .current_dir(project_root)
        .output()
        .await;

    // Restore original
    std::fs::copy(&backup_path, source_file)
        .context("Failed to restore original")?;
    std::fs::remove_file(&backup_path)
        .context("Failed to remove backup")?;

    // Return result
    match test_result {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Err(anyhow::anyhow!("Test execution failed")),
    }
}
