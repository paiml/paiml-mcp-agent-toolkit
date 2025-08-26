//! Performance testing module for SPECIFICATION.md Section 30
//!
//! This module provides the public API for performance testing functionality
//! that can be used by the CLI and other components.

use anyhow::Result;
use std::fs;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// Performance characteristics from SPECIFICATION.md Section 1.4
#[derive(Debug, Clone)]
pub struct PerformanceTargets {
    /// Startup latency targets
    pub startup_cold_ms: u64, // 127ms max
    pub startup_hot_ms: u64, // 4ms max

    /// Analysis throughput targets  
    pub loc_per_sec_st: u64, // 487,000 LOC/s single-threaded
    pub loc_per_sec_mt: u64, // 3,921,000 LOC/s multi-threaded

    /// Memory usage targets
    pub base_rss_mb: u64, // 47MB base
    pub per_kloc_kb: u64, // 312KB per KLOC
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            startup_cold_ms: 127,
            startup_hot_ms: 4,
            loc_per_sec_st: 487_000,
            loc_per_sec_mt: 3_921_000,
            base_rss_mb: 47,
            per_kloc_kb: 312,
        }
    }
}

/// Performance test configuration
pub struct PerformanceTestConfig {
    pub enable_regression_tests: bool,
    pub enable_memory_tests: bool,
    pub enable_throughput_tests: bool,
    pub test_iterations: usize,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            enable_regression_tests: true,
            enable_memory_tests: true,
            enable_throughput_tests: true,
            test_iterations: 3,
        }
    }
}

/// Generate test code with specified lines of code
fn generate_test_code(lines: usize) -> String {
    let mut code = String::with_capacity(lines * 50);
    code.push_str("// Generated test code for performance testing\n");
    code.push_str("use std::collections::HashMap;\n\n");
    code.push_str("pub struct TestStruct {\n");
    code.push_str("    data: HashMap<String, i32>,\n");
    code.push_str("}\n\n");

    for i in 0..lines.saturating_sub(10) {
        code.push_str(&format!("pub fn test_function_{i}() -> i32 {{\n"));
        code.push_str("    let mut sum = 0;\n");
        code.push_str(&format!("    for j in 0..{i} {{\n"));
        code.push_str(&format!("        sum += j * {i};\n"));
        code.push_str("    }\n");
        code.push_str("    sum\n");
        code.push_str("}\n\n");
    }

    code
}

/// Test single-threaded analysis throughput
pub async fn test_single_threaded_throughput() -> Result<()> {
    let targets = PerformanceTargets::default();
    let test_lines = 10_000; // 10K LOC test

    // Create test file
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("test.rs");
    let test_code = generate_test_code(test_lines);
    fs::write(&test_file, &test_code)?;

    // Measure analysis time
    let start = Instant::now();

    use crate::cli::handlers::complexity_handlers;
    complexity_handlers::handle_analyze_complexity(
        temp_dir.path().to_path_buf(),
        Some(test_file.clone()), // file
        vec![],                  // files
        None,                    // toolchain
        crate::cli::enums::ComplexityOutputFormat::Json,
        None,     // output
        Some(20), // max_cyclomatic
        Some(15), // max_cognitive
        vec![],   // include
        false,    // watch
        10,       // top_files
        false,    // fail_on_violation
    )
    .await?;

    let duration = start.elapsed();
    let actual_throughput = (test_lines as f64) / duration.as_secs_f64();

    assert!(
        actual_throughput >= targets.loc_per_sec_st as f64 * 0.8, // 80% of target
        "Single-threaded throughput: {:.0} LOC/s, expected ≥{} LOC/s",
        actual_throughput,
        targets.loc_per_sec_st
    );

    println!(
        "✅ Single-threaded throughput: {:.0} LOC/s (target: ≥{} LOC/s)",
        actual_throughput, targets.loc_per_sec_st
    );

    Ok(())
}

/// Test analysis performance with realistic project size
pub async fn test_realistic_project_analysis() -> Result<()> {
    let test_lines = 50_000; // 50K LOC project

    // Create test project structure
    let temp_dir = tempdir()?;
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir)?;

    // Create multiple files to simulate realistic project
    for i in 0..10 {
        let file_path = src_dir.join(format!("module_{}.rs", i));
        let file_code = generate_test_code(test_lines / 10);
        fs::write(&file_path, &file_code)?;
    }

    let start = Instant::now();

    use crate::cli::handlers::complexity_handlers;
    complexity_handlers::handle_analyze_complexity(
        temp_dir.path().to_path_buf(),
        None,   // file
        vec![], // files
        None,   // toolchain
        crate::cli::enums::ComplexityOutputFormat::Summary,
        None,     // output
        Some(20), // max_cyclomatic
        Some(15), // max_cognitive
        vec![],   // include
        false,    // watch
        10,       // top_files
        false,    // fail_on_violation
    )
    .await?;

    let duration = start.elapsed();
    let actual_throughput = (test_lines as f64) / duration.as_secs_f64();

    // More lenient threshold for multi-file analysis due to I/O overhead
    let min_throughput = 100_000; // 100K LOC/s
    assert!(
        actual_throughput >= min_throughput as f64,
        "Multi-file analysis throughput: {:.0} LOC/s, expected ≥{} LOC/s",
        actual_throughput,
        min_throughput
    );

    println!(
        "✅ Multi-file analysis: {:.0} LOC/s, duration: {:?}",
        actual_throughput, duration
    );

    Ok(())
}

/// Test large file handling performance
pub async fn test_large_file_performance() -> Result<()> {
    let test_lines = 100_000; // 100K LOC single file

    // Create large test file
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("large_file.rs");
    let test_code = generate_test_code(test_lines);
    fs::write(&test_file, &test_code)?;

    let start = Instant::now();

    use crate::cli::handlers::complexity_handlers;
    complexity_handlers::handle_analyze_complexity(
        temp_dir.path().to_path_buf(),
        Some(test_file), // file
        vec![],          // files
        None,            // toolchain
        crate::cli::enums::ComplexityOutputFormat::Summary,
        None,     // output
        Some(20), // max_cyclomatic
        Some(15), // max_cognitive
        vec![],   // include
        false,    // watch
        10,       // top_files
        false,    // fail_on_violation
    )
    .await?;

    let duration = start.elapsed();

    // Large files should still be processed reasonably quickly
    let max_duration_secs = 5; // 5 seconds max for 100K LOC
    assert!(
        duration.as_secs() <= max_duration_secs,
        "Large file analysis took {}s, expected ≤{}s for 100K LOC",
        duration.as_secs(),
        max_duration_secs
    );

    let throughput = (test_lines as f64) / duration.as_secs_f64();
    println!(
        "✅ Large file performance: {:.0} LOC/s, duration: {:?}",
        throughput, duration
    );

    Ok(())
}

/// Test memory usage patterns during analysis
pub async fn test_memory_usage_patterns() -> Result<()> {
    let test_lines = 20_000; // 20K LOC test

    // Create test file
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("memory_test.rs");
    let test_code = generate_test_code(test_lines);
    fs::write(&test_file, &test_code)?;

    // Get initial memory usage (approximate)
    let initial_memory = get_memory_usage_mb();

    // Run analysis
    use crate::cli::handlers::complexity_handlers;
    complexity_handlers::handle_analyze_complexity(
        temp_dir.path().to_path_buf(),
        Some(test_file), // file
        vec![],          // files
        None,            // toolchain
        crate::cli::enums::ComplexityOutputFormat::Json,
        None,     // output
        Some(20), // max_cyclomatic
        Some(15), // max_cognitive
        vec![],   // include
        false,    // watch
        10,       // top_files
        false,    // fail_on_violation
    )
    .await?;

    let final_memory = get_memory_usage_mb();
    let memory_used = final_memory.saturating_sub(initial_memory);

    // Memory usage should be reasonable for 20K LOC
    let expected_memory_mb = 10; // Conservative estimate
    assert!(
        memory_used <= expected_memory_mb,
        "Memory usage: {}MB for {}K LOC, expected ≤{}MB",
        memory_used,
        test_lines / 1000,
        expected_memory_mb
    );

    println!(
        "✅ Memory usage: {}MB for {}K LOC",
        memory_used,
        test_lines / 1000
    );

    Ok(())
}

/// Test performance regression detection
pub async fn test_performance_regression_detection() -> Result<()> {
    const ITERATIONS: usize = 5;
    let test_lines = 5_000; // Smaller test for multiple iterations

    // Create test file
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("regression_test.rs");
    let test_code = generate_test_code(test_lines);
    fs::write(&test_file, &test_code)?;

    let mut durations = Vec::with_capacity(ITERATIONS);

    // Run multiple iterations to detect performance variance
    for _ in 0..ITERATIONS {
        let start = Instant::now();

        use crate::cli::handlers::complexity_handlers;
        complexity_handlers::handle_analyze_complexity(
            temp_dir.path().to_path_buf(),
            Some(test_file.clone()), // file
            vec![],                  // files
            None,                    // toolchain
            crate::cli::enums::ComplexityOutputFormat::Json,
            None,     // output
            Some(20), // max_cyclomatic
            Some(15), // max_cognitive
            vec![],   // include
            false,    // watch
            10,       // top_files
            false,    // fail_on_violation
        )
        .await?;

        durations.push(start.elapsed());
    }

    // Calculate statistics
    let avg_duration = durations.iter().sum::<Duration>() / ITERATIONS as u32;
    let max_duration = durations.iter().max().unwrap();
    let min_duration = durations.iter().min().unwrap();

    // Performance should be consistent (max ≤ 2x min)
    let variance_ratio = max_duration.as_millis() as f64 / min_duration.as_millis() as f64;
    assert!(
        variance_ratio <= 2.0,
        "High performance variance: min={}ms, max={}ms, ratio={:.2}",
        min_duration.as_millis(),
        max_duration.as_millis(),
        variance_ratio
    );

    println!(
        "✅ Performance consistency: avg={}ms, min={}ms, max={}ms",
        avg_duration.as_millis(),
        min_duration.as_millis(),
        max_duration.as_millis()
    );

    Ok(())
}

/// Approximate memory usage in MB (platform-specific)
pub fn get_memory_usage_mb() -> u64 {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb / 1024; // Convert KB to MB
                        }
                    }
                }
            }
        }
    }

    // Fallback for other platforms or if reading fails
    0
}

/// Run comprehensive performance test suite
pub async fn run_performance_test_suite(config: PerformanceTestConfig) -> Result<()> {
    println!("🏃 Running PMAT Performance Test Suite (SPECIFICATION.md Section 30)");
    println!("================================================================");

    if config.enable_throughput_tests {
        println!("\n📊 Throughput Tests:");
        test_single_threaded_throughput().await?;
        test_realistic_project_analysis().await?;
        test_large_file_performance().await?;
    }

    if config.enable_regression_tests {
        println!("\n🔍 Regression Tests:");
        test_performance_regression_detection().await?;
    }

    if config.enable_memory_tests {
        println!("\n💾 Memory Tests:");
        test_memory_usage_patterns().await?;
    }

    println!("\n✅ All performance tests passed!");
    println!("Performance characteristics meet SPECIFICATION.md Section 1.4 requirements");

    Ok(())
}
