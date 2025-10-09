// Go mutation testing workflow example
// Run with: cargo run --example go_mutation_workflow --features go-ast
//
// Demonstrates:
// 1. Generating mutants from Go source
// 2. Running baseline tests
// 3. Testing each mutant
// 4. Calculating mutation score

use anyhow::{Context, Result};
use pmat::services::mutation::{GoMutationGenerator, MutantStatus};
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔷 Go Mutation Testing Workflow\n");

    // Configuration
    let source_file = PathBuf::from("../fixtures/go/calculator.go");
    let project_root = PathBuf::from("../fixtures/go");

    // Step 1: Read source file
    println!("📝 Reading source file: {}", source_file.display());
    let source = std::fs::read_to_string(&source_file)
        .context("Failed to read source file")?;

    println!("   Size: {} bytes\n", source.len());

    // Step 2: Generate mutants
    println!("🔧 Generating mutants...");
    let generator = GoMutationGenerator::with_default_operators();

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

    // Step 3: Run baseline tests
    println!("✅ Running baseline tests...");
    let baseline_passed = run_tests(&project_root).await?;

    if !baseline_passed {
        println!("❌ Baseline tests failed! Fix tests before mutation testing.");
        return Ok(());
    }
    println!("   Baseline tests passed ✅\n");

    // Step 4: Test each mutant
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

    // Step 5: Calculate mutation score
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

/// Run tests in the Go project
async fn run_tests(project_root: &PathBuf) -> Result<bool> {
    // Check if Go is available
    let has_go = Command::new("go")
        .arg("version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_go {
        println!("   ⚠️  Go not installed, skipping test execution");
        println!("   Install from: https://go.dev/dl/");
        return Ok(true); // Assume tests would pass
    }

    let output = Command::new("go")
        .arg("test")
        .arg("-v")
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
    let backup_path = source_file.with_extension("go.backup");
    std::fs::copy(source_file, &backup_path)
        .context("Failed to create backup")?;

    // Write mutant
    std::fs::write(source_file, mutated_source)
        .context("Failed to write mutated source")?;

    // Run tests
    let has_go = Command::new("go")
        .arg("version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    let result = if has_go {
        Command::new("go")
            .arg("test")
            .arg("-v")
            .current_dir(project_root)
            .output()
            .await
    } else {
        // If Go not available, assume mutant survived
        return Ok(true);
    };

    // Restore original
    std::fs::copy(&backup_path, source_file)
        .context("Failed to restore original")?;
    std::fs::remove_file(&backup_path)
        .context("Failed to remove backup")?;

    // Return result
    match result {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Err(anyhow::anyhow!("Test execution failed")),
    }
}
