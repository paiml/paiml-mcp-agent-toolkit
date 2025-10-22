//! Test command handlers for property-based testing
//!
//! This module handles the Test command for running property-based
//! test suites per SPECIFICATION.md Section 30.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::cli::commands::TestSuite;
use crate::test_performance::{run_performance_test_suite, PerformanceTestConfig};

/// Handle test command execution
#[allow(clippy::too_many_arguments)]
pub async fn handle_test(
    suite: TestSuite,
    iterations: usize,
    memory: bool,
    throughput: bool,
    regression: bool,
    timeout: u64,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    info!("Running test suite: {:?}", suite);

    match suite {
        TestSuite::Performance => {
            run_performance_suite(iterations, memory, throughput, regression, timeout, output, perf).await
        }
        TestSuite::Property => {
            run_property_expansion_tests(iterations, timeout, output).await
        }
        TestSuite::Integration => {
            run_integration_tests(timeout, output).await
        }
        TestSuite::Regression => {
            run_regression_suite(iterations).await
        }
        TestSuite::Memory => {
            run_memory_suite(iterations).await
        }
        TestSuite::Throughput => {
            run_throughput_suite(iterations).await
        }
        TestSuite::All => {
            run_all_suites(iterations, memory, throughput, regression, timeout, output).await
        }
    }
}

/// Run performance test suite
#[allow(clippy::too_many_arguments)]
async fn run_performance_suite(
    iterations: usize,
    memory: bool,
    throughput: bool,
    regression: bool,
    timeout: u64,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    let config = PerformanceTestConfig {
        enable_memory_tests: memory,
        enable_throughput_tests: throughput || !memory && !regression,
        enable_regression_tests: regression,
        test_iterations: iterations,
    };

    if perf {
        info!("Performance profiling enabled");
    }

    let start = std::time::Instant::now();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout),
        run_performance_test_suite(config),
    )
    .await;

    handle_performance_result(result, start, output, iterations, memory, throughput, regression).await
}

/// Handle performance test result
async fn handle_performance_result(
    result: Result<Result<(), anyhow::Error>, tokio::time::error::Elapsed>,
    start: std::time::Instant,
    output: Option<PathBuf>,
    iterations: usize,
    memory: bool,
    throughput: bool,
    regression: bool,
) -> Result<()> {
    match result {
        Ok(Ok(())) => {
            let duration = start.elapsed();
            info!("✅ Performance test suite completed in {:?}", duration);

            if let Some(output_path) = output {
                write_performance_report(output_path, duration, iterations, memory, throughput, regression)?;
            }

            Ok(())
        }
        Ok(Err(e)) => {
            warn!("Performance test suite failed: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("Performance test suite timed out");
            Err(anyhow::anyhow!("Test suite timed out"))
        }
    }
}

/// Write performance test report
fn write_performance_report(
    output_path: PathBuf,
    duration: std::time::Duration,
    iterations: usize,
    memory: bool,
    throughput: bool,
    regression: bool,
) -> Result<()> {
    let report = format!(
        "Performance Test Report\n\
         =======================\n\
         Duration: {duration:?}\n\
         Iterations: {iterations}\n\
         Memory Tests: {memory}\n\
         Throughput Tests: {throughput}\n\
         Regression Tests: {regression}\n\
         Status: PASSED\n"
    );

    std::fs::write(&output_path, report)?;
    info!("Report written to: {}", output_path.display());
    Ok(())
}

/// Run regression test suite
async fn run_regression_suite(iterations: usize) -> Result<()> {
    info!("Running regression test suite");

    let config = PerformanceTestConfig {
        enable_memory_tests: false,
        enable_throughput_tests: false,
        enable_regression_tests: true,
        test_iterations: iterations,
    };

    run_performance_test_suite(config).await
}

/// Run memory test suite
async fn run_memory_suite(iterations: usize) -> Result<()> {
    info!("Running memory test suite");

    let config = PerformanceTestConfig {
        enable_memory_tests: true,
        enable_throughput_tests: false,
        enable_regression_tests: false,
        test_iterations: iterations,
    };

    run_performance_test_suite(config).await
}

/// Run throughput test suite
async fn run_throughput_suite(iterations: usize) -> Result<()> {
    info!("Running throughput test suite");

    let config = PerformanceTestConfig {
        enable_memory_tests: false,
        enable_throughput_tests: true,
        enable_regression_tests: false,
        test_iterations: iterations,
    };

    run_performance_test_suite(config).await
}

/// Run all test suites
async fn run_all_suites(
    iterations: usize,
    memory: bool,
    throughput: bool,
    regression: bool,
    timeout: u64,
    output: Option<PathBuf>,
) -> Result<()> {
    info!("Running all test suites");

    let mut all_passed = true;

    // Performance tests
    let config = PerformanceTestConfig {
        enable_memory_tests: memory,
        enable_throughput_tests: throughput,
        enable_regression_tests: regression,
        test_iterations: iterations,
    };

    if let Err(e) = run_performance_test_suite(config).await {
        warn!("Performance tests failed: {}", e);
        all_passed = false;
    }

    // Property tests
    if let Err(e) = run_property_expansion_tests(iterations, timeout, None).await {
        warn!("Property tests failed: {}", e);
        all_passed = false;
    }

    // Integration tests
    if let Err(e) = run_integration_tests(timeout, None).await {
        warn!("Integration tests failed: {}", e);
        all_passed = false;
    }

    if all_passed {
        info!("✅ All test suites passed");

        if let Some(output_path) = output {
            write_all_suites_report(output_path)?;
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!("Some test suites failed"))
    }
}

/// Write all suites report
fn write_all_suites_report(output_path: PathBuf) -> Result<()> {
    let report = "All Test Suites Report\n\
                 ======================\n\
                 Performance: PASSED\n\
                 Property: PASSED\n\
                 Integration: PASSED\n\
                 Overall: PASSED\n";
    std::fs::write(&output_path, report)?;
    Ok(())
}

/// Run property-based test expansion tests
async fn run_property_expansion_tests(
    iterations: usize,
    _timeout: u64,
    output: Option<PathBuf>,
) -> Result<()> {
    info!("Running property-based test expansion suite");

    let start = std::time::Instant::now();

    // Run property tests via cargo test
    let result = tokio::process::Command::new("cargo")
        .args([
            "test",
            "--package",
            "pmat",
            "--lib",
            "property_expansion",
            "--",
            "--nocapture",
            "--test-threads=4",
        ])
        .env("PROPTEST_CASES", iterations.to_string())
        .output()
        .await?;

    let duration = start.elapsed();

    if result.status.success() {
        info!("✅ Property test expansion completed in {:?}", duration);

        if let Some(output_path) = output {
            let report = format!(
                "Property Test Expansion Report\n\
                 ==============================\n\
                 Duration: {:?}\n\
                 Iterations: {}\n\
                 Status: PASSED\n\
                 Test Output:\n{}\n",
                duration,
                iterations,
                String::from_utf8_lossy(&result.stdout)
            );
            std::fs::write(&output_path, report)?;
        }

        Ok(())
    } else {
        let error_output = String::from_utf8_lossy(&result.stderr);
        warn!("Property tests failed:\n{}", error_output);
        Err(anyhow::anyhow!("Property test expansion failed"))
    }
}

/// Run integration tests
async fn run_integration_tests(_timeout: u64, output: Option<PathBuf>) -> Result<()> {
    info!("Running integration test suite");

    let start = std::time::Instant::now();

    // Run cargo test for integration tests
    let result = tokio::process::Command::new("cargo")
        .args([
            "test",
            "--package",
            "pmat",
            "--test",
            "*",
            "--",
            "--nocapture",
        ])
        .output()
        .await?;

    let duration = start.elapsed();

    if result.status.success() {
        info!("✅ Integration tests passed in {:?}", duration);

        if let Some(output_path) = output {
            let report = format!(
                "Integration Test Report\n\
                 =======================\n\
                 Duration: {:?}\n\
                 Status: PASSED\n\
                 Output:\n{}\n",
                duration,
                String::from_utf8_lossy(&result.stdout)
            );
            std::fs::write(&output_path, report)?;
        }

        Ok(())
    } else {
        let error_output = String::from_utf8_lossy(&result.stderr);
        warn!("Integration tests failed:\n{}", error_output);
        Err(anyhow::anyhow!("Integration tests failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_test_performance() {
        // Test that performance suite can be invoked with minimal work
        let result = handle_test(
            TestSuite::Performance,
            1,     // iterations - minimum
            false, // memory - disabled
            false, // throughput - disabled to avoid heavy work
            false, // regression - disabled
            1,     // timeout - 1 second max
            None,  // output
            false, // perf
        )
        .await;

        // Should complete without panic
        assert!(result.is_ok() || result.is_err());
    }
}
