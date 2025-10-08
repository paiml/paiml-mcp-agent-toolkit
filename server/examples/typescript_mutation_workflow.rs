// TypeScript mutation testing workflow - Complete end-to-end example
// Run with: cargo run --example typescript_mutation_workflow --features typescript-ast
//
// This demonstrates:
// 1. Generating mutants from TypeScript source
// 2. Writing mutants to temp files
// 3. Running npm tests on each mutant
// 4. Calculating mutation score

use anyhow::{Context, Result};
use pmat::services::mutation::{MutantStatus, TypeScriptMutationGenerator};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧬 TypeScript Mutation Testing Workflow\n");

    // Configuration
    let source_file = PathBuf::from("fixtures/typescript/calculator.ts");
    let project_root = PathBuf::from("fixtures/typescript");

    // Step 1: Read source file
    println!("📝 Reading source file: {}", source_file.display());
    let source = fs::read_to_string(&source_file)
        .await
        .context("Failed to read source file")?;

    println!("   Size: {} bytes\n", source.len());

    // Step 2: Generate mutants
    println!("🔧 Generating mutants...");
    let generator = TypeScriptMutationGenerator::with_default_operators();

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

    // Step 3: Run baseline tests (original code should pass)
    println!("✅ Running baseline tests...");
    let baseline_passed = run_tests(&project_root, None).await?;

    if !baseline_passed {
        println!("❌ Baseline tests failed! Fix tests before mutation testing.");
        return Ok(());
    }
    println!("   Baseline tests passed ✅\n");

    // Step 4: Test each mutant
    println!("🧪 Testing mutants ({} total)...\n", mutants.len());

    let mut killed = 0;
    let mut survived = 0;
    let mut timeout_count = 0;

    let total = mutants.len();
    for (i, mutant) in mutants.iter_mut().enumerate() {
        print!("   [{}/{}] Testing mutant: {} ",
            i + 1, total, mutant.id);

        match test_mutant(&source_file, &project_root, &mutant.mutated_source).await {
            Ok(false) => {
                // Tests failed = mutant killed
                println!("☠️  KILLED");
                mutant.status = MutantStatus::Killed;
                killed += 1;
            }
            Ok(true) => {
                // Tests passed = mutant survived
                println!("🧟 SURVIVED");
                mutant.status = MutantStatus::Survived;
                survived += 1;
            }
            Err(e) => {
                // Timeout or error
                println!("⏱️  TIMEOUT/ERROR: {}", e);
                mutant.status = MutantStatus::Timeout;
                timeout_count += 1;
            }
        }
    }

    // Step 5: Calculate mutation score
    println!("\n📊 Mutation Testing Results\n");
    println!("   Total Mutants:    {}", total);
    println!("   Killed:           {} ({}%)",
        killed, (killed * 100) / total);
    println!("   Survived:         {} ({}%)",
        survived, (survived * 100) / total);
    println!("   Timeout/Error:    {}", timeout_count);

    let mutation_score = if total > timeout_count {
        (killed * 100) / (total - timeout_count)
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

    // Step 6: Show surviving mutants
    if survived > 0 {
        println!("\n🧟 Surviving Mutants (weaknesses in tests):\n");
        for mutant in mutants.iter().filter(|m| m.status == MutantStatus::Survived) {
            println!("   • {} at line {}:{}",
                mutant.id,
                mutant.location.line,
                mutant.location.column
            );

            // Show the mutated line
            let lines: Vec<&str> = mutant.mutated_source.lines().collect();
            if mutant.location.line > 0 && mutant.location.line <= lines.len() {
                let line = lines[mutant.location.line - 1].trim();
                println!("     Code: {}\n", line);
            }
        }
    }

    println!("\n🎉 Mutation testing complete!");

    Ok(())
}

/// Run tests on the project
async fn run_tests(project_root: &Path, output_filter: Option<&str>) -> Result<bool> {
    // Detect test command
    let package_json_path = project_root.join("package.json");
    let package_json = fs::read_to_string(&package_json_path).await?;

    let test_cmd = if package_json.contains("\"vitest\"") {
        "test"
    } else if package_json.contains("\"jest\"") {
        "test"
    } else {
        return Err(anyhow::anyhow!("No test framework detected"));
    };

    // Run tests
    let output = Command::new("npm")
        .arg("run")
        .arg(test_cmd)
        .current_dir(project_root)
        .output()
        .await?;

    if let Some(filter) = output_filter {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.contains(filter) && !stderr.contains(filter) {
            return Ok(false); // Filter didn't match, treat as inconclusive
        }
    }

    Ok(output.status.success())
}

/// Test a single mutant
async fn test_mutant(
    source_file: &Path,
    project_root: &Path,
    mutated_source: &str,
) -> Result<bool> {
    // Create backup of original file
    let backup_path = source_file.with_extension("ts.backup");
    fs::copy(source_file, &backup_path).await?;

    // Write mutated source
    fs::write(source_file, mutated_source).await?;

    // Run tests with timeout
    let test_result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        run_tests(project_root, None)
    ).await;

    // Restore original file
    fs::copy(&backup_path, source_file).await?;
    fs::remove_file(&backup_path).await?;

    // Return test result
    match test_result {
        Ok(Ok(passed)) => Ok(passed),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(anyhow::anyhow!("Test execution timeout")),
    }
}
