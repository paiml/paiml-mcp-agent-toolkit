//! Kaizen Test Reliability Patterns
//!
//! Implements Toyota Way reliability principles to eliminate flaky tests
//! and improve test determinism through Poka-yoke (error prevention).

use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};

/// Kaizen retry pattern with exponential backoff
pub async fn kaizen_retry<F, Fut, T, E>(
    operation_name: &str,
    mut operation: F,
    max_attempts: usize,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempts = 0;
    let mut delay = Duration::from_millis(10);

    loop {
        attempts += 1;

        match operation().await {
            Ok(result) => {
                if attempts > 1 {
                    println!("Kaizen: {operation_name} succeeded after {attempts} attempts");
                }
                return Ok(result);
            }
            Err(error) if attempts >= max_attempts => {
                eprintln!("Kaizen: {operation_name} failed after {attempts} attempts: {error}");
                return Err(error);
            }
            Err(error) => {
                println!("Kaizen: {operation_name} attempt {attempts} failed: {error}, retrying in {delay:?}");
                sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(1)); // Cap at 1 second
            }
        }
    }
}

/// Poka-yoke timeout wrapper to prevent hanging tests
pub async fn poka_yoke_timeout<F, T>(
    operation_name: &str,
    operation: F,
    timeout_duration: Duration,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: Future<Output = T>,
{
    let start = Instant::now();

    match timeout(timeout_duration, operation).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            if elapsed > timeout_duration / 2 {
                println!("Kaizen warning: {operation_name} took {elapsed:?} (close to timeout {timeout_duration:?})");
            }
            Ok(result)
        }
        Err(_) => {
            Err(format!("Poka-yoke timeout: {operation_name} exceeded {timeout_duration:?}").into())
        }
    }
}

/// Jidoka - Build quality in through deterministic test setup
pub struct JidokaTestSetup {
    cleanup_functions: Vec<Box<dyn FnOnce() + Send>>,
}

impl JidokaTestSetup {
    pub fn new() -> Self {
        Self {
            cleanup_functions: Vec::new(),
        }
    }

    /// Register cleanup function to ensure test isolation
    pub fn register_cleanup<F>(&mut self, cleanup: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_functions.push(Box::new(cleanup));
    }

    /// Create deterministic temp directory with automatic cleanup
    pub fn create_temp_dir(&mut self) -> Result<tempfile::TempDir, std::io::Error> {
        let temp_dir = tempfile::TempDir::new()?;
        let path = temp_dir.path().to_owned();

        // Register cleanup
        self.register_cleanup(move || {
            if path.exists() {
                let _ = std::fs::remove_dir_all(&path);
            }
        });

        Ok(temp_dir)
    }

    /// Set deterministic environment variable with cleanup
    pub fn set_env_var(&mut self, key: &str, value: &str) {
        let key = key.to_string();
        let original_value = std::env::var(&key).ok();

        std::env::set_var(&key, value);

        // Register cleanup to restore original state
        self.register_cleanup(move || match original_value {
            Some(orig_val) => std::env::set_var(&key, orig_val),
            None => std::env::remove_var(&key),
        });
    }
}

impl Drop for JidokaTestSetup {
    fn drop(&mut self) {
        // Execute all cleanup functions in reverse order (LIFO)
        while let Some(cleanup) = self.cleanup_functions.pop() {
            cleanup();
        }
    }
}

/// Genchi Genbutsu - Go see the actual test state
pub struct TestStateInspector {
    start_time: Instant,
    operation_name: String,
    checkpoints: Vec<(String, Instant, Duration)>,
}

impl TestStateInspector {
    pub fn new(operation_name: impl Into<String>) -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: operation_name.into(),
            checkpoints: Vec::new(),
        }
    }

    /// Record checkpoint for performance analysis
    pub fn checkpoint(&mut self, name: impl Into<String>) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time);
        self.checkpoints.push((name.into(), now, elapsed));
    }

    /// Generate Genchi Genbutsu report
    pub fn generate_report(&self) -> String {
        let mut report = format!("Genchi Genbutsu Report for: {}\n", self.operation_name);
        report.push_str(&format!(
            "Total Duration: {:?}\n",
            self.start_time.elapsed()
        ));
        report.push_str("Checkpoints:\n");

        for (name, _time, elapsed) in &self.checkpoints {
            report.push_str(&format!("  - {name}: {elapsed:?}\n"));
        }

        report
    }
}

impl Drop for TestStateInspector {
    fn drop(&mut self) {
        if self.start_time.elapsed() > Duration::from_millis(100) {
            println!("{}", self.generate_report());
        }
    }
}

/// Muda elimination - Reduce waste in test execution
pub mod muda_elimination {

    /// Fast assertion that fails quickly without expensive operations
    pub fn fast_assert_eq<T: PartialEq + std::fmt::Debug>(
        left: T,
        right: T,
        message: &str,
    ) -> Result<(), String> {
        if left == right {
            Ok(())
        } else {
            Err(format!("Muda elimination fast assertion failed: {message}\nLeft: {left:?}\nRight: {right:?}"))
        }
    }

    /// Skip expensive operations in fast test mode
    pub fn should_skip_expensive_operation() -> bool {
        std::env::var("KAIZEN_FAST_TESTS").is_ok() || std::env::var("CI").is_ok()
    }

    /// Create minimal test data instead of large realistic data
    #[allow(dead_code)] // Utility function for future minimal test data
    pub fn create_minimal_test_data<T: Default>() -> T {
        T::default()
    }
}

#[cfg(test)]
mod kaizen_reliability_tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_kaizen_retry_succeeds_first_attempt() {
        let mut attempt_count = 0;

        let result = kaizen_retry(
            "test_operation",
            || {
                attempt_count += 1;
                async { Ok::<i32, &str>(42) }
            },
            3,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count, 1);
    }

    #[tokio::test]
    async fn test_kaizen_retry_succeeds_after_retries() {
        let mut attempt_count = 0;

        let result = kaizen_retry(
            "test_operation",
            || {
                attempt_count += 1;
                async move {
                    if attempt_count < 3 {
                        Err("temporary failure")
                    } else {
                        Ok(42)
                    }
                }
            },
            5,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count, 3);
    }

    #[tokio::test]
    async fn test_poka_yoke_timeout_success() {
        let result =
            poka_yoke_timeout("fast_operation", async { 42 }, Duration::from_millis(100)).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_poka_yoke_timeout_failure() {
        let result = poka_yoke_timeout(
            "slow_operation",
            async {
                sleep(Duration::from_millis(200)).await;
                42
            },
            Duration::from_millis(100),
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Poka-yoke timeout"));
    }

    #[test]
    fn test_jidoka_test_setup_cleanup() {
        let _temp_path = {
            let mut setup = JidokaTestSetup::new();
            setup.set_env_var("KAIZEN_TEST_VAR", "test_value");

            let temp_dir = setup.create_temp_dir().unwrap();
            let path = temp_dir.path().to_owned();

            // Verify env var is set
            assert_eq!(std::env::var("KAIZEN_TEST_VAR").unwrap(), "test_value");

            path
        }; // setup drops here, cleanup should happen

        // Verify cleanup happened
        assert!(std::env::var("KAIZEN_TEST_VAR").is_err());
        // Note: temp_dir cleanup is handled by tempfile crate
    }

    #[test]
    fn test_test_state_inspector() {
        let mut inspector = TestStateInspector::new("test_operation");

        inspector.checkpoint("setup");
        std::thread::sleep(Duration::from_millis(10));
        inspector.checkpoint("execution");
        std::thread::sleep(Duration::from_millis(10));
        inspector.checkpoint("cleanup");

        let report = inspector.generate_report();
        assert!(report.contains("test_operation"));
        assert!(report.contains("setup"));
        assert!(report.contains("execution"));
        assert!(report.contains("cleanup"));
    }

    #[test]
    fn test_muda_elimination_fast_assert() {
        use muda_elimination::*;

        // Should succeed
        assert!(fast_assert_eq(42, 42, "equal values").is_ok());

        // Should fail quickly
        let result = fast_assert_eq(42, 43, "unequal values");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Muda elimination"));
    }

    #[test]
    fn test_should_skip_expensive_operation() {
        use muda_elimination::*;

        // Without env var, should not skip
        std::env::remove_var("KAIZEN_FAST_TESTS");
        std::env::remove_var("CI");
        assert!(!should_skip_expensive_operation());

        // With env var, should skip
        std::env::set_var("KAIZEN_FAST_TESTS", "1");
        assert!(should_skip_expensive_operation());

        // Cleanup
        std::env::remove_var("KAIZEN_FAST_TESTS");
    }
}
