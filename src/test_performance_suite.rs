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

/// Seconds a single analysis in this suite is allowed to take.
///
/// `handle_analyze_complexity` takes a `timeout` argument and prints
/// "⏰ Analysis timeout set to 60 seconds", but nothing enforces it: `pmat test
/// throughput` sat in `test_large_file_performance` for the whole of a 170s
/// external `timeout` and was killed at exit 124, having printed the "Analyzing
/// complexity of file" line and then nothing. Until the handler honours its own
/// argument, the suite enforces the budget it advertises at the call site — a
/// perf suite that never returns cannot report anything.
const ANALYSIS_BUDGET_SECS: u64 = 60;

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
    // Spawned rather than awaited inline: the analysis is CPU-bound and does
    // not yield, and `tokio::time::timeout` cannot interrupt a future that
    // holds its worker thread. Off on its own task, the timer still fires.
    let analysis = tokio::spawn(complexity_handlers::handle_analyze_complexity(
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
        ANALYSIS_BUDGET_SECS, // timeout
    ));

    match tokio::time::timeout(Duration::from_secs(ANALYSIS_BUDGET_SECS), analysis).await {
        Ok(joined) => joined??,
        Err(_) => {
            println!(
                "❌ Large file performance: no result within {ANALYSIS_BUDGET_SECS}s for {test_lines} LOC"
            );
            anyhow::bail!(
                "Large file analysis exceeded its own {ANALYSIS_BUDGET_SECS}s budget for {test_lines} LOC"
            );
        }
    }

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

    // Peak resident set, in KB, before the analysis. This used to be
    // `get_memory_usage_mb()` — VmRSS truncated to whole MB — and the "usage"
    // was `final.saturating_sub(initial)`, a difference of two rounded-down MB
    // figures over a transient allocation. It reported "✅ Memory usage: 0MB
    // for 20K LOC" every single run, so `memory_used <= 10` could not fail and
    // "Memory tests passed!" was unconditional. VmHWM in KB is the peak the
    // analysis actually reached, at 1024x the resolution.
    let before = MemorySample::read()?;

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

    let after = MemorySample::read()?;
    let peak_growth_kb = after.peak_kb.saturating_sub(before.peak_kb);

    // Memory usage should be reasonable for 20K LOC
    const BUDGET_KB: u64 = 10 * 1024; // 10 MB, as before — now in KB
    if peak_growth_kb > BUDGET_KB {
        println!(
            "❌ Memory usage: peak grew {peak_growth_kb} KB for {}K LOC (budget: ≤{BUDGET_KB} KB)",
            test_lines / 1000
        );
        anyhow::bail!(
            "Peak resident memory grew {peak_growth_kb} KB for {}K LOC, over the {BUDGET_KB} KB budget",
            test_lines / 1000
        );
    }

    println!(
        "✅ Memory usage: peak RSS {} KB (was {} KB), analysis added {} KB for {}K LOC",
        after.peak_kb,
        before.peak_kb,
        peak_growth_kb,
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
    parse_status_kb(status, "VmRSS:")
}

/// A `/proc/self/status` size field, in KB.
fn parse_status_kb(status: &str, field: &str) -> Option<u64> {
    status
        .lines()
        .find(|line| line.starts_with(field))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|kb_str| kb_str.parse::<u64>().ok())
}

/// Resident memory in KB: current (`VmRSS`) and peak (`VmHWM`).
///
/// `read()` fails rather than substituting 0 when the figures are unavailable.
/// The memory gate used to source its numbers from a function that returned 0
/// on any read or parse failure, so a platform where nothing could be measured
/// printed "Memory usage: 0MB" and passed — a gate that cannot measure must not
/// report a pass.
#[derive(Debug, Clone, Copy)]
pub struct MemorySample {
    pub rss_kb: u64,
    pub peak_kb: u64,
}

impl MemorySample {
    /// Read the current process's resident memory.
    ///
    /// # Errors
    /// Returns an error when `/proc/self/status` cannot be read or does not
    /// carry `VmRSS`/`VmHWM` — i.e. when the measurement does not exist.
    pub fn read() -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let status = std::fs::read_to_string("/proc/self/status").map_err(|e| {
                anyhow::anyhow!("cannot measure memory: /proc/self/status unreadable: {e}")
            })?;
            let rss_kb = parse_status_kb(&status, "VmRSS:")
                .ok_or_else(|| anyhow::anyhow!("cannot measure memory: no VmRSS in status"))?;
            let peak_kb = parse_status_kb(&status, "VmHWM:")
                .ok_or_else(|| anyhow::anyhow!("cannot measure memory: no VmHWM in status"))?;
            Ok(Self { rss_kb, peak_kb })
        }

        #[cfg(not(target_os = "linux"))]
        anyhow::bail!("cannot measure memory: resident-set reporting is Linux-only on this build")
    }
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

    /// Running zero tests is not a pass. `pmat test performance` reached this
    /// function with every sub-suite disabled and printed
    /// "✅ All performance tests passed!".
    #[tokio::test]
    async fn test_suite_with_nothing_enabled_is_an_error() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: false,
            enable_throughput_tests: false,
            test_iterations: 3,
        };
        let err = run_performance_test_suite(config)
            .await
            .expect_err("a run that measured nothing must not report success");
        assert!(err.to_string().contains("nothing was measured"), "{err}");
    }

    /// The memory gate read whole MB and reported the difference of two rounded
    /// figures, which was 0 on every run. KB resolution is the point.
    #[test]
    fn test_memory_sample_reads_kb_resolution() {
        let sample = MemorySample::read().expect("linux test host exposes /proc/self/status");
        assert!(
            sample.rss_kb > 1024,
            "RSS {} KB looks unmeasured",
            sample.rss_kb
        );
        assert!(
            sample.peak_kb >= sample.rss_kb,
            "peak {} KB below current {} KB",
            sample.peak_kb,
            sample.rss_kb
        );

        // The resolution the old reading threw away: a 500 KB growth is 0 MB
        // on both sides of the subtraction the gate used to perform.
        let before = "VmRSS:\t  102400 kB\nVmHWM:\t  102400 kB\n";
        let after = "VmRSS:\t  102900 kB\nVmHWM:\t  102900 kB\n";
        let before_kb = parse_status_kb(before, "VmHWM:").unwrap();
        let after_kb = parse_status_kb(after, "VmHWM:").unwrap();
        assert_eq!(after_kb - before_kb, 500);
        assert_eq!(after_kb / 1024 - before_kb / 1024, 0, "the old MB view");
    }

    #[test]
    fn test_parse_status_kb_reads_peak_field() {
        let status = "Name:\tpmat\nVmRSS:\t  123456 kB\nVmHWM:\t  234567 kB\n";
        assert_eq!(parse_status_kb(status, "VmRSS:"), Some(123_456));
        assert_eq!(parse_status_kb(status, "VmHWM:"), Some(234_567));
        assert_eq!(parse_status_kb(status, "VmNope:"), None);
    }
}

/// Run comprehensive performance test suite
pub async fn run_performance_test_suite(config: PerformanceTestConfig) -> Result<()> {
    println!("🏃 Running PMAT Performance Test Suite (SPECIFICATION.md Section 30)");
    println!("================================================================");

    // With every sub-suite disabled this function skipped all three blocks and
    // printed "✅ All performance tests passed!" anyway — which is what
    // `pmat test performance` (the CLI default suite) did on every invocation,
    // byte-identically in an empty directory and in a 4260-file repository.
    // Zero tests run is not a pass.
    if !config.enable_throughput_tests
        && !config.enable_regression_tests
        && !config.enable_memory_tests
    {
        anyhow::bail!(
            "no performance sub-suite was enabled, so nothing was measured (expected at least one of throughput/regression/memory)"
        );
    }

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
