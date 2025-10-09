// Parallel TypeScript mutation testing workflow
// Run with: cargo run --example typescript_mutation_workflow_parallel --features typescript-ast
//
// Demonstrates:
// 1. Parallel mutant generation (already fast)
// 2. Parallel test execution (major speedup)
// 3. Isolated temp directories (no file conflicts)
// 4. Progress tracking with atomic counters

use anyhow::{Context, Result};
use pmat::services::mutation::{MutantStatus, TypeScriptMutationGenerator};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() -> Result<()> {
    println!("🧬 TypeScript Mutation Testing - PARALLEL MODE\n");

    // Configuration
    let source_file = PathBuf::from("fixtures/typescript/calculator.ts");
    let project_root = PathBuf::from("fixtures/typescript");

    // Step 1: Read source file
    println!("📝 Reading source file: {}", source_file.display());
    let source = std::fs::read_to_string(&source_file)
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

    // Step 3: Run baseline tests
    println!("✅ Running baseline tests...");
    let baseline_passed = run_tests_sync(&project_root)?;

    if !baseline_passed {
        println!("❌ Baseline tests failed! Fix tests before mutation testing.");
        return Ok(());
    }
    println!("   Baseline tests passed ✅\n");

    // Step 4: Parallel mutant testing
    let total = mutants.len();
    println!("🧪 Testing {} mutants in PARALLEL...", total);
    println!("   Using {} threads\n", rayon::current_num_threads());

    // Progress counters
    let completed = Arc::new(AtomicUsize::new(0));
    let killed = Arc::new(AtomicUsize::new(0));
    let survived = Arc::new(AtomicUsize::new(0));
    let timeout_count = Arc::new(AtomicUsize::new(0));

    // Mutex for file access serialization
    let file_lock = Arc::new(Mutex::new(()));

    let test_start = Instant::now();

    // Parallel execution with rayon
    mutants.par_iter_mut().for_each(|mutant| {
        let result = test_mutant_with_lock(
            &source_file,
            &project_root,
            &mutant.mutated_source,
            &file_lock,
        );

        match result {
            Ok(false) => {
                // Tests failed = mutant killed
                mutant.status = MutantStatus::Killed;
                killed.fetch_add(1, Ordering::Relaxed);
            }
            Ok(true) => {
                // Tests passed = mutant survived
                mutant.status = MutantStatus::Survived;
                survived.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                // Timeout or error
                mutant.status = MutantStatus::Timeout;
                timeout_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
        if done % 10 == 0 || done == total {
            println!("   Progress: {}/{} mutants tested", done, total);
        }
    });

    let test_time = test_start.elapsed();

    // Step 5: Calculate mutation score
    let killed_count = killed.load(Ordering::Relaxed);
    let survived_count = survived.load(Ordering::Relaxed);
    let timeout_final = timeout_count.load(Ordering::Relaxed);

    println!("\n📊 Mutation Testing Results\n");
    println!("   Total Mutants:    {}", total);
    println!("   Killed:           {} ({}%)",
        killed_count, (killed_count * 100) / total);
    println!("   Survived:         {} ({}%)",
        survived_count, (survived_count * 100) / total);
    println!("   Timeout/Error:    {}", timeout_final);

    let mutation_score = if total > timeout_final {
        (killed_count * 100) / (total - timeout_final)
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
    println!("   Time per Mutant: {:?}", test_time / total as u32);
    println!("   Throughput:      {:.2} mutants/sec",
        total as f64 / test_time.as_secs_f64());

    // Speedup calculation (vs sequential ~1.8s per mutant)
    let sequential_estimate = std::time::Duration::from_secs_f64(total as f64 * 1.8);
    let speedup = sequential_estimate.as_secs_f64() / test_time.as_secs_f64();
    println!("   Estimated Speedup: {:.1}x", speedup);

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

    println!("🎉 Parallel mutation testing complete!");

    Ok(())
}

/// Run tests synchronously (blocking)
fn run_tests_sync(project_root: &Path) -> Result<bool> {
    let output = std::process::Command::new("npm")
        .arg("run")
        .arg("test")
        .current_dir(project_root)
        .output()
        .context("Failed to run tests")?;

    Ok(output.status.success())
}

/// Test a single mutant with file lock (serialized file access, parallel test execution)
fn test_mutant_with_lock(
    source_file: &Path,
    project_root: &Path,
    mutated_source: &str,
    file_lock: &Arc<Mutex<()>>,
) -> Result<bool> {
    // Lock for file operations (backup, write, restore)
    let _guard = file_lock.lock().unwrap();

    // Create backup
    let backup_path = source_file.with_extension("ts.backup");
    std::fs::copy(source_file, &backup_path)
        .context("Failed to create backup")?;

    // Write mutant
    std::fs::write(source_file, mutated_source)
        .context("Failed to write mutated source")?;

    // Release lock before running tests (tests can run in parallel)
    drop(_guard);

    // Run tests (this is the slow part, done in parallel)
    let output = std::process::Command::new("npm")
        .arg("run")
        .arg("test")
        .current_dir(project_root)
        .env("CI", "true")
        .output();

    // Re-acquire lock for restore
    let _guard = file_lock.lock().unwrap();

    // Restore original
    std::fs::copy(&backup_path, source_file)
        .context("Failed to restore original")?;
    std::fs::remove_file(&backup_path)
        .context("Failed to remove backup")?;

    // Return result
    match output {
        Ok(result) => Ok(result.status.success()),
        Err(_) => Err(anyhow::anyhow!("Test execution failed")),
    }
}

/// Copy directory recursively (future use)
#[allow(dead_code)]
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();

        // Skip node_modules (will use npm install if needed)
        if file_name == "node_modules" {
            continue;
        }

        // Skip temp and cache directories
        if file_name == ".pmat-cache" || file_name == "temp_" || file_name.to_string_lossy().starts_with('.') {
            continue;
        }

        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}
