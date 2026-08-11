//! Test Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains test suite execution and helper functions.

#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::TestSuite;
use std::path::PathBuf;

impl CommandDispatcher {
    /// Execute test commands using handler pattern (reduces CC)
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_test_command(
        suite: TestSuite,
        iterations: usize,
        memory: bool,
        throughput: bool,
        regression: bool,
        timeout: u64,
        output: Option<PathBuf>,
        perf: bool,
    ) -> anyhow::Result<()> {
        let config = Self::create_test_config(&suite, iterations, memory, throughput, regression);
        Self::print_test_startup_info(&suite, iterations, timeout);

        let start = std::time::Instant::now();
        let test_future = Self::execute_test_suite(&suite, config);

        Self::execute_with_timeout_and_reporting(
            test_future,
            timeout,
            start,
            &suite,
            iterations,
            output,
            perf,
        )
        .await
    }

    /// Create test configuration from CLI parameters (Toyota Way Extract Method)
    ///
    /// `TestSuite::Performance` — the CLI default — used to appear in none of
    /// these arms, so `pmat test performance` produced a config with all three
    /// `enable_*` flags false; `run_performance_test_suite` then skipped every
    /// block and printed "✅ All performance tests passed!" without running a
    /// single measurement, identically in an empty directory and in a
    /// 4260-file repository. Performance means "everything performance-related".
    pub(crate) fn create_test_config(
        suite: &TestSuite,
        iterations: usize,
        memory: bool,
        throughput: bool,
        regression: bool,
    ) -> crate::test_performance::PerformanceTestConfig {
        crate::test_performance::PerformanceTestConfig {
            enable_regression_tests: regression
                || matches!(
                    suite,
                    TestSuite::Regression | TestSuite::Performance | TestSuite::All
                ),
            enable_memory_tests: memory
                || matches!(
                    suite,
                    TestSuite::Memory | TestSuite::Performance | TestSuite::All
                ),
            enable_throughput_tests: throughput
                || matches!(
                    suite,
                    TestSuite::Throughput | TestSuite::Performance | TestSuite::All
                ),
            test_iterations: iterations,
        }
    }

    /// Print test startup information (Toyota Way Extract Method)
    pub(crate) fn print_test_startup_info(suite: &TestSuite, iterations: usize, timeout: u64) {
        println!("Starting Performance Testing Suite (SPECIFICATION.md Section 30)");
        println!("Suite: {suite:?}, Iterations: {iterations}, Timeout: {timeout}s");
    }

    /// Execute the specific test suite (Toyota Way Extract Method)
    pub(crate) async fn execute_test_suite(
        suite: &TestSuite,
        config: crate::test_performance::PerformanceTestConfig,
    ) -> anyhow::Result<()> {
        use crate::test_performance::run_performance_test_suite;

        match suite {
            TestSuite::Performance | TestSuite::All => run_performance_test_suite(config).await,
            TestSuite::Regression => Self::execute_regression_tests(config).await,
            TestSuite::Memory => Self::execute_memory_tests(config).await,
            TestSuite::Throughput => Self::execute_throughput_tests(config).await,
            TestSuite::Property => Self::execute_property_tests().await,
            TestSuite::Integration => Self::execute_integration_tests().await,
        }
    }

    /// Execute regression tests (Toyota Way Extract Method)
    pub(crate) async fn execute_regression_tests(
        config: crate::test_performance::PerformanceTestConfig,
    ) -> anyhow::Result<()> {
        if config.enable_regression_tests {
            println!("Running regression tests...");
            crate::test_performance::test_performance_regression_detection().await?;
            println!("Regression tests passed!");
        }
        Ok(())
    }

    /// Execute memory tests (Toyota Way Extract Method)
    pub(crate) async fn execute_memory_tests(
        config: crate::test_performance::PerformanceTestConfig,
    ) -> anyhow::Result<()> {
        if config.enable_memory_tests {
            println!("Running memory tests...");
            crate::test_performance::test_memory_usage_patterns().await?;
            println!("Memory tests passed!");
        }
        Ok(())
    }

    /// Execute throughput tests (Toyota Way Extract Method)
    pub(crate) async fn execute_throughput_tests(
        config: crate::test_performance::PerformanceTestConfig,
    ) -> anyhow::Result<()> {
        if config.enable_throughput_tests {
            println!("Running throughput tests...");
            crate::test_performance::test_single_threaded_throughput().await?;
            crate::test_performance::test_realistic_project_analysis().await?;
            crate::test_performance::test_large_file_performance().await?;
            println!("Throughput tests passed!");
        }
        Ok(())
    }

    /// Execute property tests (Toyota Way Extract Method)
    pub(crate) async fn execute_property_tests() -> anyhow::Result<()> {
        println!("Running property-based test suite...");
        println!("This validates code properties with generated test cases");

        // Run property tests via cargo
        use std::process::Command;
        let output = Command::new("cargo")
            .arg("test")
            .arg("--package")
            .arg("pmat")
            .arg("--lib")
            .arg("--")
            .arg("property")
            .output()?;

        if output.status.success() {
            println!("Property tests completed successfully");
            Ok(())
        } else {
            anyhow::bail!("Property tests failed")
        }
    }

    /// Execute integration tests (Toyota Way Extract Method)
    pub(crate) async fn execute_integration_tests() -> anyhow::Result<()> {
        println!("Running integration test suite...");
        println!("This validates component interactions and system behavior");

        // Check if integration test exists
        use std::path::Path;
        if !Path::new("tests/integration.rs").exists() {
            println!("No separate integration test file found");
            println!("Integration tests are embedded in unit tests");
            return Ok(());
        }

        // Run integration tests via cargo if they exist
        use std::process::Command;
        let output = Command::new("cargo")
            .arg("test")
            .arg("--package")
            .arg("pmat")
            .arg("--test")
            .arg("integration")
            .output()?;

        if output.status.success() {
            println!("Integration tests completed successfully");
            Ok(())
        } else {
            anyhow::bail!("Integration tests failed")
        }
    }

    /// Execute test with timeout and generate reports (Toyota Way Extract Method)
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_with_timeout_and_reporting(
        test_future: impl std::future::Future<Output = anyhow::Result<()>>,
        timeout: u64,
        start: std::time::Instant,
        suite: &TestSuite,
        iterations: usize,
        output: Option<PathBuf>,
        perf: bool,
    ) -> anyhow::Result<()> {
        let timeout_duration = std::time::Duration::from_secs(timeout);

        if let Ok(result) = tokio::time::timeout(timeout_duration, test_future).await {
            let elapsed = start.elapsed();
            Self::print_performance_summary_if_requested(perf, elapsed, suite, iterations);
            Self::write_test_results_if_requested(output, suite, elapsed, iterations, &result)?;
            result
        } else {
            eprintln!("Test execution timed out after {timeout}s");
            anyhow::bail!("Performance tests timed out");
        }
    }

    /// Print performance summary if requested (Toyota Way Extract Method)
    pub(crate) fn print_performance_summary_if_requested(
        perf: bool,
        elapsed: std::time::Duration,
        suite: &TestSuite,
        iterations: usize,
    ) {
        if perf {
            println!("\nPerformance Summary:");
            println!("   Total execution time: {elapsed:?}");
            println!("   Suite: {suite:?}");
            println!("   Iterations: {iterations}");
        }
    }

    /// Write test results to file if requested (Toyota Way Extract Method)
    pub(crate) fn write_test_results_if_requested(
        output: Option<PathBuf>,
        suite: &TestSuite,
        elapsed: std::time::Duration,
        iterations: usize,
        result: &anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        if let Some(output_path) = output {
            let results = format!(
                "Performance Test Results\n\
                ======================\n\
                Suite: {:?}\n\
                Execution time: {:?}\n\
                Iterations: {}\n\
                Status: {}\n",
                suite,
                elapsed,
                iterations,
                if result.is_ok() { "PASSED" } else { "FAILED" }
            );
            std::fs::write(&output_path, results)?;
            println!("Results written to: {}", output_path.display());
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_config_selection_tests {
    use super::*;

    /// `pmat test performance` — the default suite — must actually select
    /// something to measure. It used to select nothing and report a pass.
    #[test]
    fn test_performance_suite_enables_every_sub_suite() {
        let config =
            CommandDispatcher::create_test_config(&TestSuite::Performance, 3, false, false, false);
        assert!(
            config.enable_throughput_tests,
            "Performance selected no throughput tests"
        );
        assert!(
            config.enable_regression_tests,
            "Performance selected no regression tests"
        );
        assert!(
            config.enable_memory_tests,
            "Performance selected no memory tests"
        );
    }

    #[test]
    fn test_named_sub_suites_stay_narrow() {
        let memory =
            CommandDispatcher::create_test_config(&TestSuite::Memory, 1, false, false, false);
        assert!(memory.enable_memory_tests);
        assert!(!memory.enable_throughput_tests);
        assert!(!memory.enable_regression_tests);
    }
}
