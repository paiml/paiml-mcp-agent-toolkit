//! Kaizen Test Performance Optimizations
//!
//! This module implements continuous improvement principles for test reliability,
//! speed, and maintainability following Toyota Production System philosophy.

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Global test performance tracker for Kaizen improvements
#[allow(dead_code)] // Used for future global metrics collection
static TEST_METRICS: Mutex<TestMetrics> = Mutex::new(TestMetrics::new());

/// Test metrics collector for continuous improvement
#[derive(Debug, Clone)]
pub struct TestMetrics {
    pub total_tests: usize,
    pub avg_duration: Duration,
    pub slow_tests: Vec<SlowTest>,
    pub flaky_tests: Vec<FlakyTest>,
    pub parallel_efficiency: f64,
}

#[derive(Debug, Clone)]
pub struct SlowTest {
    pub name: String,
    pub duration: Duration,
    pub category: TestCategory,
}

#[derive(Debug, Clone)]
pub struct FlakyTest {
    pub name: String,
    pub failure_rate: f64,
    #[allow(dead_code)] // Future use for flaky test analysis
    pub last_failure: String,
}

#[derive(Debug, Clone)]
pub enum TestCategory {
    Unit,
    #[allow(dead_code)] // Future use for integration test categorization
    Integration,
    #[allow(dead_code)] // Future use for E2E test categorization
    E2E,
    #[allow(dead_code)] // Future use for property test categorization
    Property,
}

impl TestMetrics {
    const fn new() -> Self {
        Self {
            total_tests: 0,
            avg_duration: Duration::from_millis(0),
            slow_tests: Vec::new(),
            flaky_tests: Vec::new(),
            parallel_efficiency: 0.0,
        }
    }

    /// Record test execution for Kaizen analysis
    pub fn record_test(&mut self, name: String, duration: Duration, category: TestCategory) {
        self.total_tests += 1;

        // Update average duration
        let total_millis =
            self.avg_duration.as_millis() * (self.total_tests - 1) as u128 + duration.as_millis();
        self.avg_duration = Duration::from_millis((total_millis / self.total_tests as u128) as u64);

        // Identify slow tests (>100ms for unit, >1s for integration, >10s for E2E)
        let threshold = match category {
            TestCategory::Unit => Duration::from_millis(100),
            TestCategory::Integration => Duration::from_secs(1),
            TestCategory::E2E => Duration::from_secs(10),
            TestCategory::Property => Duration::from_millis(500),
        };

        if duration > threshold {
            self.slow_tests.push(SlowTest {
                name,
                duration,
                category,
            });
        }
    }

    /// Generate Kaizen improvement report
    pub fn generate_kaizen_report(&self) -> String {
        let mut report = String::with_capacity(1024);
        report.push_str("# Kaizen Test Performance Report\n\n");

        report.push_str("## Test Suite Health\n");
        report.push_str(&format!("- Total Tests: {}\n", self.total_tests));
        report.push_str(&format!("- Average Duration: {:?}\n", self.avg_duration));
        report.push_str(&format!(
            "- Parallel Efficiency: {:.1}%\n\n",
            self.parallel_efficiency * 100.0
        ));

        if !self.slow_tests.is_empty() {
            report.push_str("## Slow Tests (Muda - Waste)\n");
            for test in &self.slow_tests {
                report.push_str(&format!(
                    "- `{}`: {:?} ({:?})\n",
                    test.name, test.duration, test.category
                ));
            }
            report.push('\n');
        }

        if !self.flaky_tests.is_empty() {
            report.push_str("## Flaky Tests (Defects)\n");
            for test in &self.flaky_tests {
                report.push_str(&format!(
                    "- `{}`: {:.1}% failure rate\n",
                    test.name,
                    test.failure_rate * 100.0
                ));
            }
            report.push('\n');
        }

        report.push_str("## Kaizen Recommendations\n");
        report.push_str("1. **Jidoka**: Eliminate flaky tests through root cause analysis\n");
        report.push_str("2. **Muda Reduction**: Optimize slow tests or parallelize operations\n");
        report.push_str("3. **Poka-yoke**: Add test stability patterns and timeouts\n");
        report.push_str("4. **Kaizen**: Continuous monitoring and improvement\n");

        report
    }
}

/// Enhanced test runner with Kaizen principles
pub struct KaizenTestRunner {
    concurrency_limit: Arc<Semaphore>,
    test_metrics: Arc<Mutex<TestMetrics>>,
}

impl KaizenTestRunner {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent)),
            test_metrics: Arc::new(Mutex::new(TestMetrics::new())),
        }
    }

    /// Run test with performance tracking
    pub async fn run_test<F, Fut>(
        &self,
        name: &str,
        category: TestCategory,
        test_fn: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<()>>,
    {
        let _permit = self.concurrency_limit.acquire().await?;
        let start = Instant::now();

        let result = test_fn().await;
        let duration = start.elapsed();

        // Record metrics
        {
            let mut metrics = self.test_metrics.lock();
            metrics.record_test(name.to_string(), duration, category);
        }

        result
    }

    /// Get current test metrics for analysis
    pub fn get_metrics(&self) -> TestMetrics {
        self.test_metrics.lock().clone()
    }
}

/// Kaizen test utilities for eliminating waste (Muda)
pub mod utils {

    /// Fast setup for unit tests - minimal dependencies
    #[allow(dead_code)] // Utility function for future test optimization
    pub fn fast_unit_test_setup() -> anyhow::Result<()> {
        // Initialize only essential components
        std::env::set_var("RUST_LOG", "warn"); // Reduce logging noise
        Ok(())
    }

    /// Optimized temp directory that cleans up quickly
    pub fn fast_temp_dir() -> anyhow::Result<tempfile::TempDir> {
        // Use memory-backed temp dir if available
        let temp_dir = if cfg!(target_os = "linux") {
            tempfile::TempDir::new_in("/dev/shm")
                .unwrap_or_else(|_| tempfile::TempDir::new().unwrap())
        } else {
            tempfile::TempDir::new()?
        };
        Ok(temp_dir)
    }

    /// Parallel-safe test data generator
    #[allow(dead_code)] // Utility function for future test data generation
    pub fn generate_test_data(size: usize) -> Vec<String> {
        (0..size).map(|i| format!("test_data_{i}")).collect()
    }

    /// Mock heavy operations for faster tests
    pub struct MockHeavyOperation;

    impl MockHeavyOperation {
        #[allow(dead_code)] // Mock function for future performance testing
        pub async fn fast_analysis() -> anyhow::Result<String> {
            // Return pre-computed result instead of actual analysis
            Ok("mock_result".to_string())
        }

        pub fn fast_file_system() -> Vec<std::path::PathBuf> {
            // Return synthetic file list instead of real FS scan
            vec![
                "/mock/file1.rs".into(),
                "/mock/file2.rs".into(),
                "/mock/file3.rs".into(),
            ]
        }
    }
}

/// Property-based test optimizations
pub mod property_testing {
    use proptest::prelude::*;

    /// Optimized proptest config for Kaizen efficiency
    pub fn fast_proptest_config() -> ProptestConfig {
        ProptestConfig {
            cases: 20,             // Reduced from default 256 for speed
            max_shrink_iters: 100, // Reduced from 1024
            timeout: 1000,         // 1 second timeout
            ..ProptestConfig::default()
        }
    }

    /// Generate small, focused test inputs
    pub fn small_string_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{1,10}".prop_map(|s| s)
    }

    pub fn small_vec_strategy<T: Clone + std::fmt::Debug + 'static>(
        element: impl Strategy<Value = T>,
    ) -> impl Strategy<Value = Vec<T>> {
        prop::collection::vec(element, 0..5) // Small collections for speed
    }
}

/// Global test metrics collector
impl Drop for TestMetrics {
    fn drop(&mut self) {
        if self.total_tests > 0 {
            println!("\n{}", self.generate_kaizen_report());
        }
    }
}

#[cfg(test)]
mod kaizen_optimization_tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_kaizen_runner_tracks_metrics() {
        let runner = KaizenTestRunner::new(4);

        // Run a fast test
        runner
            .run_test("fast_test", TestCategory::Unit, || async {
                sleep(Duration::from_millis(10)).await;
                Ok(())
            })
            .await
            .unwrap();

        // Run a slow test
        runner
            .run_test("slow_test", TestCategory::Unit, || async {
                sleep(Duration::from_millis(150)).await; // Above 100ms threshold
                Ok(())
            })
            .await
            .unwrap();

        let metrics = runner.get_metrics();
        assert_eq!(metrics.total_tests, 2);
        assert_eq!(metrics.slow_tests.len(), 1);
        assert_eq!(metrics.slow_tests[0].name, "slow_test");
    }

    #[test]
    fn test_fast_temp_dir_creation() {
        let start = Instant::now();
        let _temp_dir = utils::fast_temp_dir().unwrap();
        let duration = start.elapsed();

        // Should be faster than 10ms
        assert!(duration < Duration::from_millis(10));
    }

    #[test]
    fn test_mock_heavy_operation_performance() {
        let start = Instant::now();
        let _data = utils::MockHeavyOperation::fast_file_system();
        let duration = start.elapsed();

        // Should be nearly instantaneous
        assert!(duration < Duration::from_millis(1));
    }

    #[tokio::test]
    async fn test_concurrent_test_execution() {
        let runner = KaizenTestRunner::new(2); // Limit to 2 concurrent

        let start = Instant::now();

        // Run 4 tests concurrently (should batch in groups of 2)
        let handles = (0..4).map(|i| {
            let runner = &runner;
            async move {
                runner
                    .run_test(&format!("test_{i}"), TestCategory::Unit, || async {
                        sleep(Duration::from_millis(50)).await;
                        Ok(())
                    })
                    .await
            }
        });

        futures::future::try_join_all(handles).await.unwrap();

        let total_duration = start.elapsed();
        // With concurrency limit of 2, should take ~100ms (2 batches of 50ms each)
        // Allow some overhead
        assert!(total_duration < Duration::from_millis(150));
        assert!(total_duration > Duration::from_millis(90));
    }

    use proptest::proptest;

    proptest! {
        #![proptest_config(property_testing::fast_proptest_config())]

        #[test]
        fn test_property_small_strings(s in property_testing::small_string_strategy()) {
            assert!(s.len() <= 10);
            assert!(!s.is_empty() || s.is_empty()); // Tautology for demo
        }

        #[test]
        fn test_property_small_vectors(
            vec in property_testing::small_vec_strategy(proptest::prelude::any::<u32>())
        ) {
            assert!(vec.len() <= 5);
        }
    }
}
