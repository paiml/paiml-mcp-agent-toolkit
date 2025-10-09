// C++ mutation testing workflow example
// Run with: cargo run --example cpp_mutation_workflow --features cpp-ast
//
// Demonstrates:
// 1. Generating mutants from C++ source
// 2. Running baseline tests with CMake/CTest
// 3. Testing each mutant
// 4. Calculating mutation score

use anyhow::{Context, Result};
use pmat::services::mutation::{CppMutationGenerator, MutantStatus};
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔷 C++ Mutation Testing Workflow\n");

    // Configuration
    let source_file = PathBuf::from("../fixtures/cpp/calculator.cpp");
    let project_root = PathBuf::from("../fixtures/cpp");
    let build_dir = project_root.join("build");

    // Step 1: Read source file
    println!("📝 Reading source file: {}", source_file.display());
    let source = std::fs::read_to_string(&source_file)
        .context("Failed to read source file")?;

    println!("   Size: {} bytes\n", source.len());

    // Step 2: Generate mutants
    println!("🔧 Generating mutants...");
    let generator = CppMutationGenerator::with_default_operators();

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

    // Step 3: Setup build directory
    println!("🏗️  Setting up CMake build...");
    let cmake_setup = setup_cmake(&project_root, &build_dir).await?;

    if !cmake_setup {
        println!("⚠️  CMake not available, skipping test execution");
        println!("   Install CMake from: https://cmake.org/download/");
        return Ok(());
    }

    // Step 4: Run baseline tests
    println!("✅ Running baseline tests...");
    let baseline_passed = run_tests(&build_dir).await?;

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

        match test_mutant(&source_file, &build_dir, &mutant.mutated_source).await {
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

/// Setup CMake build directory
async fn setup_cmake(project_root: &PathBuf, build_dir: &PathBuf) -> Result<bool> {
    // Check if CMake is available
    let has_cmake = Command::new("cmake")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_cmake {
        return Ok(false);
    }

    // Create build directory if it doesn't exist
    std::fs::create_dir_all(build_dir).ok();

    // Run CMake configuration
    let output = Command::new("cmake")
        .arg("-S")
        .arg(project_root)
        .arg("-B")
        .arg(build_dir)
        .output()
        .await
        .context("Failed to run CMake configuration")?;

    if !output.status.success() {
        println!("   ⚠️  CMake configuration failed");
        return Ok(false);
    }

    // Build the project
    let output = Command::new("cmake")
        .arg("--build")
        .arg(build_dir)
        .output()
        .await
        .context("Failed to build project")?;

    Ok(output.status.success())
}

/// Run tests in the C++ project using CTest
async fn run_tests(build_dir: &PathBuf) -> Result<bool> {
    let output = Command::new("ctest")
        .arg("--test-dir")
        .arg(build_dir)
        .arg("--output-on-failure")
        .output()
        .await
        .context("Failed to run tests")?;

    Ok(output.status.success())
}

/// Test a single mutant by temporarily replacing the source file
async fn test_mutant(
    source_file: &PathBuf,
    build_dir: &PathBuf,
    mutated_source: &str,
) -> Result<bool> {
    // Create backup
    let backup_path = source_file.with_extension("cpp.backup");
    std::fs::copy(source_file, &backup_path)
        .context("Failed to create backup")?;

    // Write mutant
    std::fs::write(source_file, mutated_source)
        .context("Failed to write mutated source")?;

    // Rebuild the project
    let build_result = Command::new("cmake")
        .arg("--build")
        .arg(build_dir)
        .output()
        .await;

    // Check if build succeeded
    let build_ok = match &build_result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    };

    let test_result = if build_ok {
        // Run tests
        Command::new("ctest")
            .arg("--test-dir")
            .arg(build_dir)
            .arg("--output-on-failure")
            .output()
            .await
    } else {
        // Build failed, treat as killed mutant
        Err(anyhow::anyhow!("Build failed"))
    };

    // Restore original
    std::fs::copy(&backup_path, source_file)
        .context("Failed to restore original")?;
    std::fs::remove_file(&backup_path)
        .context("Failed to remove backup")?;

    // Rebuild original
    Command::new("cmake")
        .arg("--build")
        .arg(build_dir)
        .output()
        .await
        .ok();

    // Return result
    match test_result {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Err(anyhow::anyhow!("Test execution failed")),
    }
}
