/// Report a throughput measurement against its target — and fail when it misses.
///
/// These checks used to compute the comparison, emit a `Warning:` line on
/// stderr, then print the ✅ marker and return `Ok(())` unconditionally, so
/// `pmat test throughput` reported
/// "✅ Single-threaded throughput: 1593 LOC/s (target: ≥487000 LOC/s)" and
/// exited 0 — a measurement 300x under its own target read as a pass. The
/// comparison now decides both the marker and the return value.
fn report_throughput(label: &str, actual: f64, required: f64, extra: &str) -> Result<()> {
    println!(
        "{} {label}: {actual:.0} LOC/s (target: ≥{required:.0} LOC/s){extra}",
        if actual < required { "❌" } else { "✅" }
    );
    if actual < required {
        anyhow::bail!("{label}: {actual:.0} LOC/s is below the required {required:.0} LOC/s");
    }
    Ok(())
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

    // 20% slack for test-environment variance; past that the measurement decides.
    let required = targets.loc_per_sec_st as f64 * 0.8;
    report_throughput("Single-threaded throughput", actual_throughput, required, "")
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
    report_throughput(
        "Multi-file analysis",
        actual_throughput,
        f64::from(min_throughput),
        &format!(", duration: {duration:?}"),
    )
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
    let throughput = (test_lines as f64) / duration.as_secs_f64();
    // Same defect as the throughput checks above: this used to warn on stderr
    // and print ✅ anyway.
    if duration.as_secs() > max_duration_secs {
        println!(
            "❌ Large file performance: {throughput:.0} LOC/s, duration: {duration:?} (budget: ≤{max_duration_secs}s for 100K LOC)"
        );
        anyhow::bail!(
            "Large file analysis took {}s, over the {}s budget for 100K LOC",
            duration.as_secs(),
            max_duration_secs
        );
    }

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

fn parse_vmrss_kb(status: &str) -> Option<u64> {
    status
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|kb_str| kb_str.parse::<u64>().ok())
}

/// Approximate memory usage in MB (platform-specific)
#[must_use]
pub fn get_memory_usage_mb() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            return parse_vmrss_kb(&status).unwrap_or(0) / 1024;
        }
    }

    // Fallback for other platforms or if reading fails
    0
}

#[cfg(test)]
mod throughput_verdict_tests {
    use super::*;

    /// A measurement below its target must not report success.
    #[test]
    fn test_report_throughput_fails_below_target() {
        // The exact numbers `pmat test throughput` printed a ✅ for.
        let err = report_throughput("Single-threaded throughput", 1593.0, 487_000.0, "")
            .expect_err("1593 LOC/s against a 487000 LOC/s target is a failure");
        assert!(err.to_string().contains("below the required"), "{err}");
        assert!(
            report_throughput("Multi-file analysis", 67944.0, 100_000.0, "").is_err(),
            "67944 LOC/s against a 100000 LOC/s target is a failure"
        );
    }

    #[test]
    fn test_report_throughput_passes_at_or_above_target() {
        assert!(report_throughput("t", 100_000.0, 100_000.0, "").is_ok());
        assert!(report_throughput("t", 500_000.0, 487_000.0, "").is_ok());
    }
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
