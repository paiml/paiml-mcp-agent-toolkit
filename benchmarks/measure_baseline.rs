#!/usr/bin/env rust-script
//! Baseline measurement for dependency reduction benchmarking
//! Pattern: Modeled after trueno-db competitive benchmarking methodology
//!
//! ```cargo
//! [dependencies]
//! chrono = "0.4"
//! ```

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let results_dir = Path::new("benchmarks/results");
    fs::create_dir_all(results_dir)?;

    let result_file = results_dir.join(format!("baseline_{}.md", timestamp));

    println!("🔬 Starting baseline measurements at {}", timestamp);
    println!("Results will be saved to: {}", result_file.display());

    let mut output = String::new();

    // Header
    output.push_str(&format!("# Baseline Measurement Results\n\n"));
    output.push_str(&format!("**Timestamp**: {}\n", timestamp));
    output.push_str("**Spec**: docs/specifications/dependency-reduction-benchmarking-framework.md\n\n");

    // Environment
    output.push_str("## Environment\n\n```\n");
    output.push_str(&format!("Rust: {}\n", run_command("rustc", &["--version"])?));
    output.push_str(&format!("Cargo: {}\n", run_command("cargo", &["--version"])?));
    output.push_str(&format!("OS: {} {}\n",
        run_command("uname", &["-s"])?,
        run_command("uname", &["-r"])?));

    let cpu_info = run_command("lscpu", &[])
        .unwrap_or_default()
        .lines()
        .find(|l| l.contains("Model name"))
        .map(|l| l.split(':').nth(1).unwrap_or("").trim())
        .unwrap_or("Unknown");
    output.push_str(&format!("CPU: {}\n", cpu_info));
    output.push_str(&format!("Cores: {}\n", run_command("nproc", &[])?));

    let mem = run_command("free", &["-h"])?
        .lines()
        .find(|l| l.starts_with("Mem:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("Unknown");
    output.push_str(&format!("RAM: {}\n", mem));
    output.push_str("```\n\n");

    // Dependency counts
    println!("📦 Measuring dependency counts...");
    output.push_str("## Dependency Counts\n\n");
    output.push_str("| Configuration | Count | Delta from Default |\n");
    output.push_str("|---------------|-------|--------------------|\\n");

    let deps_minimal = count_dependencies(&["--no-default-features", "--features", "rust-only"])?;
    let deps_default = count_dependencies(&[])?;
    let deps_all = count_dependencies(&["--all-features"])?;

    let delta_minimal = deps_default as i32 - deps_minimal as i32;
    let pct_minimal = (delta_minimal as f64 / deps_default as f64 * 100.0).abs();

    let delta_all = deps_all as i32 - deps_default as i32;
    let pct_all = (delta_all as f64 / deps_default as f64 * 100.0).abs();

    output.push_str(&format!("| Minimal (rust-only) | {} | {} ({:.1}%) |\n",
        deps_minimal, delta_minimal, pct_minimal));
    output.push_str(&format!("| Default | {} | baseline |\n", deps_default));
    output.push_str(&format!("| All features | {} | +{} (+{:.1}%) |\n\n",
        deps_all, delta_all, pct_all));

    output.push_str("```bash\n");
    output.push_str("# Commands used\n");
    output.push_str("cargo tree | wc -l  # Default\n");
    output.push_str("cargo tree --no-default-features --features rust-only | wc -l  # Minimal\n");
    output.push_str("cargo tree --all-features | wc -l  # All\n");
    output.push_str("```\n\n");

    // Build times
    println!("⏱️  Measuring build times (this will take several minutes)...");
    output.push_str("## Build Times (Clean Builds)\n\n");
    output.push_str("| Configuration | Time | Command |\n");
    output.push_str("|---------------|------|---------|\\n");

    println!("  Measuring: dev-default");
    let dev_time = measure_build(&[], false)?;
    output.push_str(&format!("| Dev (default) | {:.2}s | `cargo build` |\n", dev_time));

    println!("  Measuring: release-default");
    let release_time = measure_build(&[], true)?;
    output.push_str(&format!("| Release (default) | {:.2}s | `cargo build --release` |\n", release_time));

    println!("  Measuring: release-minimal");
    let minimal_time = measure_build(&["--no-default-features", "--features", "rust-only"], true)?;
    output.push_str(&format!("| Release (minimal) | {:.2}s | `cargo build --release --features rust-only` |\n\n", minimal_time));

    // Binary sizes
    println!("📏 Measuring binary sizes...");
    output.push_str("## Binary Sizes (Release, Stripped)\n\n");
    output.push_str("| Configuration | Size | Delta from Default |\n");
    output.push_str("|---------------|------|--------------------|\n");

    build_release(&[])?;
    let size_default = get_binary_size()?;

    build_release(&["--no-default-features", "--features", "rust-only"])?;
    let size_minimal = get_binary_size()?;

    build_release(&["--all-features"])?;
    let size_all = get_binary_size()?;

    let delta_min = size_default as i64 - size_minimal as i64;
    let pct_min = (delta_min as f64 / size_default as f64 * 100.0).abs();

    let delta_all = size_all as i64 - size_default as i64;
    let pct_all_size = (delta_all as f64 / size_default as f64 * 100.0).abs();

    output.push_str(&format!("| Minimal (rust-only) | {} | {} bytes ({:.1}%) |\n",
        format_size(size_minimal), delta_min, pct_min));
    output.push_str(&format!("| Default | {} | baseline |\n", format_size(size_default)));
    output.push_str(&format!("| All features | {} | +{} bytes (+{:.1}%) |\n\n",
        format_size(size_all), delta_all, pct_all_size));

    // Write results
    fs::write(&result_file, output)?;

    println!("✅ Baseline measurements complete!");
    println!();
    println!("📊 Results saved to: {}", result_file.display());
    println!();
    println!("📖 View results:");
    println!("   cat {}", result_file.display());

    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(cmd).args(args).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn count_dependencies(args: &[&str]) -> Result<usize, Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .arg("tree")
        .args(args)
        .output()?;
    Ok(output.stdout.lines().count())
}

fn measure_build(extra_args: &[&str], release: bool) -> Result<f64, Box<dyn std::error::Error>> {
    // Clean build
    Command::new("cargo").arg("clean").output()?;

    let start = Instant::now();

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if release {
        cmd.arg("--release");
    }
    cmd.args(extra_args);

    cmd.output()?;

    let duration = start.elapsed();
    Ok(duration.as_secs_f64())
}

fn build_release(extra_args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .args(extra_args)
        .output()?;
    Ok(())
}

fn get_binary_size() -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = fs::metadata("target/release/pmat")?;
    Ok(metadata.len())
}

fn format_size(bytes: u64) -> String {
    batuta_common::fmt::format_bytes(bytes)
}
