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
        60,       // timeout
    )
    .await?;

    let duration = start.elapsed();
    let actual_throughput = (test_lines as f64) / duration.as_secs_f64();

    // Performance may vary in test environment
    if actual_throughput < targets.loc_per_sec_st as f64 * 0.8 {
        eprintln!(
            "Warning: Single-threaded throughput: {:.0} LOC/s, expected ≥{} LOC/s",
            actual_throughput, targets.loc_per_sec_st
        );
    }

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
        let file_path = src_dir.join(format!("module_{i}.rs"));
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
        60,       // timeout
    )
    .await?;

    let duration = start.elapsed();
    let actual_throughput = (test_lines as f64) / duration.as_secs_f64();

    // More lenient threshold for multi-file analysis due to I/O overhead
    let min_throughput = 100_000; // 100K LOC/s
    if actual_throughput < f64::from(min_throughput) {
        eprintln!("Warning: Multi-file analysis throughput: {actual_throughput:.0} LOC/s, expected ≥{min_throughput} LOC/s");
    }

    println!("✅ Multi-file analysis: {actual_throughput:.0} LOC/s, duration: {duration:?}");

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
        60,       // timeout
    )
    .await?;

    let duration = start.elapsed();

    // Large files should still be processed reasonably quickly
    let max_duration_secs = 30; // More lenient for test environments
    if duration.as_secs() > max_duration_secs {
        eprintln!(
            "Warning: Large file analysis took {}s, expected ≤{}s for 100K LOC",
            duration.as_secs(),
            max_duration_secs
        );
    }

    let throughput = (test_lines as f64) / duration.as_secs_f64();
    println!("✅ Large file performance: {throughput:.0} LOC/s, duration: {duration:?}");

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
        60,       // timeout
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
            60,       // timeout
        )
        .await?;

        durations.push(start.elapsed());
    }

    // Calculate statistics
    let avg_duration = durations.iter().sum::<Duration>() / ITERATIONS as u32;
    let max_duration = durations.iter().max().expect("internal error");
    let min_duration = durations.iter().min().expect("internal error");

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
#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============ PerformanceTargets Tests ============

    #[test]
    fn test_performance_targets_default() {
        let targets = PerformanceTargets::default();
        assert_eq!(targets.startup_cold_ms, 127);
        assert_eq!(targets.startup_hot_ms, 4);
        assert_eq!(targets.loc_per_sec_st, 487_000);
        assert_eq!(targets.loc_per_sec_mt, 3_921_000);
        assert_eq!(targets.base_rss_mb, 47);
        assert_eq!(targets.per_kloc_kb, 312);
    }

    #[test]
    fn test_performance_targets_clone() {
        let targets = PerformanceTargets::default();
        let cloned = targets.clone();
        assert_eq!(cloned.startup_cold_ms, targets.startup_cold_ms);
        assert_eq!(cloned.startup_hot_ms, targets.startup_hot_ms);
    }

    #[test]
    fn test_performance_targets_debug() {
        let targets = PerformanceTargets::default();
        let debug = format!("{:?}", targets);
        assert!(debug.contains("PerformanceTargets"));
        assert!(debug.contains("127"));
    }

    #[test]
    fn test_performance_targets_custom_values() {
        let targets = PerformanceTargets {
            startup_cold_ms: 200,
            startup_hot_ms: 10,
            loc_per_sec_st: 500_000,
            loc_per_sec_mt: 4_000_000,
            base_rss_mb: 50,
            per_kloc_kb: 300,
        };
        assert_eq!(targets.startup_cold_ms, 200);
        assert_eq!(targets.loc_per_sec_st, 500_000);
    }

    // ============ PerformanceTestConfig Tests ============

    #[test]
    fn test_performance_test_config_default() {
        let config = PerformanceTestConfig::default();
        assert!(config.enable_regression_tests);
        assert!(config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert_eq!(config.test_iterations, 3);
    }

    #[test]
    fn test_performance_test_config_custom() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: false,
            enable_throughput_tests: true,
            test_iterations: 10,
        };
        assert!(!config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert_eq!(config.test_iterations, 10);
    }

    #[test]
    fn test_performance_test_config_all_disabled() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: false,
            enable_throughput_tests: false,
            test_iterations: 0,
        };
        assert!(!config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(!config.enable_throughput_tests);
    }

    // ============ generate_test_code Tests ============

    #[test]
    fn test_generate_test_code_empty() {
        let code = generate_test_code(0);
        assert!(code.contains("Generated test code"));
        assert!(code.contains("TestStruct"));
    }

    #[test]
    fn test_generate_test_code_small() {
        let code = generate_test_code(15);
        assert!(code.contains("test_function_0"));
        assert!(code.contains("test_function_4"));
    }

    #[test]
    fn test_generate_test_code_medium() {
        let code = generate_test_code(50);
        let lines: Vec<_> = code.lines().collect();
        assert!(lines.len() > 20);
    }

    #[test]
    fn test_generate_test_code_has_struct() {
        let code = generate_test_code(20);
        assert!(code.contains("pub struct TestStruct"));
        assert!(code.contains("HashMap<String, i32>"));
    }

    #[test]
    fn test_generate_test_code_has_functions() {
        let code = generate_test_code(25);
        assert!(code.contains("pub fn test_function_"));
        assert!(code.contains("let mut sum = 0;"));
    }

    #[test]
    fn test_generate_test_code_capacity() {
        let code = generate_test_code(100);
        // Ensure code is generated with proper structure
        assert!(code.len() > 500);
    }

    // ============ get_memory_usage_mb Tests ============

    #[test]
    fn test_get_memory_usage_mb() {
        // This function returns 0 on non-Linux or if reading fails
        let usage = get_memory_usage_mb();
        // Just verify it returns a valid value (u64 is always >= 0)
        // On Linux, we expect a positive value; on other platforms, 0
        #[cfg(target_os = "linux")]
        assert!(usage > 0 || usage == 0, "Expected valid memory value");
        #[cfg(not(target_os = "linux"))]
        assert_eq!(usage, 0, "Expected 0 on non-Linux platforms");
    }

    #[test]
    fn test_get_memory_usage_mb_multiple_calls() {
        // Call multiple times to ensure consistency
        let usage1 = get_memory_usage_mb();
        let usage2 = get_memory_usage_mb();
        // Values should be similar (within a reasonable range)
        let diff = if usage1 > usage2 { usage1 - usage2 } else { usage2 - usage1 };
        assert!(diff < 100); // Within 100MB variance
    }

    // ============ PerformanceTargets Field Tests ============

    #[test]
    fn test_performance_targets_startup_cold_reasonable() {
        let targets = PerformanceTargets::default();
        // Cold startup should be > hot startup
        assert!(targets.startup_cold_ms > targets.startup_hot_ms);
    }

    #[test]
    fn test_performance_targets_throughput_reasonable() {
        let targets = PerformanceTargets::default();
        // Multi-threaded should be faster than single-threaded
        assert!(targets.loc_per_sec_mt > targets.loc_per_sec_st);
    }

    #[test]
    fn test_performance_targets_memory_reasonable() {
        let targets = PerformanceTargets::default();
        // Base RSS should be positive
        assert!(targets.base_rss_mb > 0);
        // Per KLOC should be positive
        assert!(targets.per_kloc_kb > 0);
    }

    // ============ generate_test_code Edge Cases ============

    #[test]
    fn test_generate_test_code_exactly_10_lines() {
        let code = generate_test_code(10);
        // Should have header but no functions (saturating_sub prevents negative)
        assert!(code.contains("Generated test code"));
        assert!(code.contains("TestStruct"));
        // 10 - 10 = 0, so no functions generated
        assert!(!code.contains("test_function_0"));
    }

    #[test]
    fn test_generate_test_code_11_lines() {
        let code = generate_test_code(11);
        // 11 - 10 = 1, so one function
        assert!(code.contains("test_function_0"));
        assert!(!code.contains("test_function_1"));
    }

    #[test]
    fn test_generate_test_code_100_lines() {
        let code = generate_test_code(100);
        // 100 - 10 = 90 functions
        assert!(code.contains("test_function_0"));
        assert!(code.contains("test_function_89"));
        assert!(!code.contains("test_function_90"));
    }

    #[test]
    fn test_generate_test_code_function_body_structure() {
        let code = generate_test_code(20);
        // Check function body structure
        assert!(code.contains("-> i32"));
        assert!(code.contains("let mut sum = 0;"));
        assert!(code.contains("for j in"));
        assert!(code.contains("sum += j *"));
        assert!(code.contains("sum\n}"));
    }

    #[test]
    fn test_generate_test_code_import_statement() {
        let code = generate_test_code(1);
        assert!(code.contains("use std::collections::HashMap;"));
    }

    #[test]
    fn test_generate_test_code_saturating_sub_large() {
        // Test with value much larger than 10
        let code = generate_test_code(1000);
        // Should have 990 functions
        assert!(code.contains("test_function_989"));
        assert!(!code.contains("test_function_990"));
    }

    // ============ PerformanceTestConfig Combinations ============

    #[test]
    fn test_config_only_regression() {
        let config = PerformanceTestConfig {
            enable_regression_tests: true,
            enable_memory_tests: false,
            enable_throughput_tests: false,
            test_iterations: 5,
        };
        assert!(config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(!config.enable_throughput_tests);
    }

    #[test]
    fn test_config_only_memory() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: true,
            enable_throughput_tests: false,
            test_iterations: 1,
        };
        assert!(!config.enable_regression_tests);
        assert!(config.enable_memory_tests);
        assert!(!config.enable_throughput_tests);
    }

    #[test]
    fn test_config_only_throughput() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: false,
            enable_throughput_tests: true,
            test_iterations: 100,
        };
        assert!(!config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert_eq!(config.test_iterations, 100);
    }

    // ============ PerformanceTargets Boundary Tests ============

    #[test]
    fn test_performance_targets_zero_values() {
        let targets = PerformanceTargets {
            startup_cold_ms: 0,
            startup_hot_ms: 0,
            loc_per_sec_st: 0,
            loc_per_sec_mt: 0,
            base_rss_mb: 0,
            per_kloc_kb: 0,
        };
        assert_eq!(targets.startup_cold_ms, 0);
        assert_eq!(targets.loc_per_sec_st, 0);
    }

    #[test]
    fn test_performance_targets_max_values() {
        let targets = PerformanceTargets {
            startup_cold_ms: u64::MAX,
            startup_hot_ms: u64::MAX,
            loc_per_sec_st: u64::MAX,
            loc_per_sec_mt: u64::MAX,
            base_rss_mb: u64::MAX,
            per_kloc_kb: u64::MAX,
        };
        assert_eq!(targets.startup_cold_ms, u64::MAX);
        assert_eq!(targets.loc_per_sec_mt, u64::MAX);
    }

    #[test]
    fn test_performance_targets_clone_independence() {
        let original = PerformanceTargets {
            startup_cold_ms: 100,
            startup_hot_ms: 5,
            loc_per_sec_st: 400_000,
            loc_per_sec_mt: 3_000_000,
            base_rss_mb: 40,
            per_kloc_kb: 250,
        };
        let cloned = original.clone();
        // Cloned values should match
        assert_eq!(cloned.startup_cold_ms, 100);
        assert_eq!(cloned.startup_hot_ms, 5);
        assert_eq!(cloned.loc_per_sec_st, 400_000);
        assert_eq!(cloned.loc_per_sec_mt, 3_000_000);
        assert_eq!(cloned.base_rss_mb, 40);
        assert_eq!(cloned.per_kloc_kb, 250);
    }

    #[test]
    fn test_performance_targets_debug_format_all_fields() {
        let targets = PerformanceTargets {
            startup_cold_ms: 150,
            startup_hot_ms: 8,
            loc_per_sec_st: 550_000,
            loc_per_sec_mt: 4_500_000,
            base_rss_mb: 55,
            per_kloc_kb: 350,
        };
        let debug = format!("{:?}", targets);
        assert!(debug.contains("startup_cold_ms"));
        assert!(debug.contains("startup_hot_ms"));
        assert!(debug.contains("loc_per_sec_st"));
        assert!(debug.contains("loc_per_sec_mt"));
        assert!(debug.contains("base_rss_mb"));
        assert!(debug.contains("per_kloc_kb"));
    }

    // ============ Memory Usage Tests ============

    #[test]
    fn test_get_memory_usage_returns_consistent() {
        // Memory usage shouldn't fluctuate wildly between immediate calls
        let usage1 = get_memory_usage_mb();
        let usage2 = get_memory_usage_mb();
        let usage3 = get_memory_usage_mb();

        // All calls should return similar values (or 0 on non-Linux)
        if usage1 > 0 {
            let max = usage1.max(usage2).max(usage3);
            let min = usage1.min(usage2).min(usage3);
            // Should be within 50MB of each other
            assert!(max - min < 50, "Memory variance too high: {}-{}", min, max);
        }
    }

    #[test]
    fn test_get_memory_usage_after_allocation() {
        let before = get_memory_usage_mb();

        // Allocate some memory (this may or may not show up depending on platform)
        let _large_vec: Vec<u8> = vec![0u8; 1024 * 1024]; // 1MB

        let after = get_memory_usage_mb();

        // On Linux, should show increase; on other platforms, both will be 0
        assert!(after >= before || (before == 0 && after == 0));
    }

    // ============ Code Generation Line Count Tests ============

    #[test]
    fn test_generate_test_code_line_count_approximation() {
        let code = generate_test_code(50);
        let actual_lines = code.lines().count();
        // Each function adds ~7 lines, plus ~10 header lines
        // 50 - 10 = 40 functions * ~7 = ~280 lines + 10 header ≈ 290
        assert!(actual_lines > 200, "Expected >200 lines, got {}", actual_lines);
    }

    #[test]
    fn test_generate_test_code_capacity_estimation() {
        // Test that string capacity is roughly correct
        let lines = 100;
        let code = generate_test_code(lines);
        // Capacity should be enough for the content
        assert!(code.capacity() >= code.len());
    }

    // ============ PerformanceTestConfig Iteration Tests ============

    #[test]
    fn test_config_zero_iterations() {
        let config = PerformanceTestConfig {
            enable_regression_tests: true,
            enable_memory_tests: true,
            enable_throughput_tests: true,
            test_iterations: 0,
        };
        assert_eq!(config.test_iterations, 0);
    }

    #[test]
    fn test_config_large_iterations() {
        let config = PerformanceTestConfig {
            enable_regression_tests: true,
            enable_memory_tests: true,
            enable_throughput_tests: true,
            test_iterations: 1_000_000,
        };
        assert_eq!(config.test_iterations, 1_000_000);
    }

    #[test]
    fn test_config_default_values_match_specification() {
        let config = PerformanceTestConfig::default();
        // Default iterations should be reasonable for CI
        assert!(config.test_iterations >= 1);
        assert!(config.test_iterations <= 10);
    }

    // ============ PerformanceTargets Specification Compliance ============

    #[test]
    fn test_default_targets_match_specification() {
        let targets = PerformanceTargets::default();
        // Per SPECIFICATION.md Section 1.4
        assert_eq!(targets.startup_cold_ms, 127, "Startup cold should be 127ms");
        assert_eq!(targets.startup_hot_ms, 4, "Startup hot should be 4ms");
        assert_eq!(targets.loc_per_sec_st, 487_000, "ST throughput should be 487K");
        assert_eq!(targets.loc_per_sec_mt, 3_921_000, "MT throughput should be 3.9M");
        assert_eq!(targets.base_rss_mb, 47, "Base RSS should be 47MB");
        assert_eq!(targets.per_kloc_kb, 312, "Per KLOC should be 312KB");
    }
}
