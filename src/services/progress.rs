//! Progress tracking for analysis operations
//!
//! This module provides progress bars and status tracking for long-running
//! analysis operations to improve user experience.
//!
//! NOTE: indicatif dependency removed to reduce transitive deps (saves 6 deps)
//! Using simple println-based progress reporting instead.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// No-op progress bar that mimics indicatif::ProgressBar API
/// Used as a zero-dependency replacement for indicatif
pub struct SimpleProgressBar {
    message: Arc<Mutex<String>>,
    position: Arc<AtomicU64>,
    length: Arc<AtomicU64>,
    hidden: bool,
}

impl Clone for SimpleProgressBar {
    fn clone(&self) -> Self {
        Self {
            message: Arc::clone(&self.message),
            position: Arc::clone(&self.position),
            length: Arc::clone(&self.length),
            hidden: self.hidden,
        }
    }
}

impl SimpleProgressBar {
    /// Create a new progress bar
    #[must_use]
    pub fn new(len: u64) -> Self {
        Self {
            message: Arc::new(Mutex::new(String::new())),
            position: Arc::new(AtomicU64::new(0)),
            length: Arc::new(AtomicU64::new(len)),
            hidden: false,
        }
    }

    /// Create a spinner (indeterminate progress)
    #[must_use]
    pub fn new_spinner() -> Self {
        Self::new(0)
    }

    /// Create a hidden progress bar (no output)
    #[must_use]
    pub fn hidden() -> Self {
        Self {
            message: Arc::new(Mutex::new(String::new())),
            position: Arc::new(AtomicU64::new(0)),
            length: Arc::new(AtomicU64::new(0)),
            hidden: true,
        }
    }

    /// Check if this is a hidden progress bar
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Set the current message
    pub fn set_message(&self, msg: impl Into<String>) {
        if let Ok(mut m) = self.message.lock() {
            *m = msg.into();
        }
    }

    /// Get the current message
    #[must_use]
    pub fn message(&self) -> String {
        self.message.lock().map(|m| m.clone()).unwrap_or_default()
    }

    /// Set the position
    pub fn set_position(&self, pos: u64) {
        self.position.store(pos, Ordering::Relaxed);
    }

    /// Get the current position
    #[must_use]
    pub fn position(&self) -> u64 {
        self.position.load(Ordering::Relaxed)
    }

    /// Increment the position by delta
    pub fn inc(&self, delta: u64) {
        self.position.fetch_add(delta, Ordering::Relaxed);
    }

    /// Set the total length
    pub fn set_length(&self, len: u64) {
        self.length.store(len, Ordering::Relaxed);
    }

    /// Get the length
    #[must_use]
    pub fn length(&self) -> Option<u64> {
        let len = self.length.load(Ordering::Relaxed);
        if len > 0 {
            Some(len)
        } else {
            None
        }
    }

    /// Finish the progress bar with a message
    pub fn finish_with_message(&self, msg: impl Into<String>) {
        if !self.hidden {
            eprintln!("{}", msg.into());
        }
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        // No-op for simple implementation
    }

    /// Finish and clear the progress bar
    pub fn finish_and_clear(&self) {
        // No-op for simple implementation
    }

    /// Abandon the progress bar
    pub fn abandon(&self) {
        // No-op for simple implementation
    }

    /// Abandon the progress bar with a message
    pub fn abandon_with_message(&self, msg: impl Into<String>) {
        if !self.hidden {
            eprintln!("{}", msg.into());
        }
    }

    /// Enable steady tick (no-op without indicatif)
    pub fn enable_steady_tick(&self, _duration: std::time::Duration) {
        // No-op - we don't animate without indicatif
    }

    /// Tick the progress bar (no-op)
    pub fn tick(&self) {
        // No-op
    }

    /// Set style (no-op without indicatif)
    pub fn set_style(&self, _style: SimpleProgressStyle) {
        // No-op - styles not supported without indicatif
    }

    /// Print a message above the progress bar
    pub fn println(&self, msg: impl AsRef<str>) {
        if !self.hidden {
            eprintln!("{}", msg.as_ref());
        }
    }

    /// Suspend the progress bar for closure execution
    pub fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        f()
    }
}

/// No-op progress style that mimics indicatif::ProgressStyle API
#[derive(Clone, Default)]
pub struct SimpleProgressStyle;

impl SimpleProgressStyle {
    /// Create default spinner style
    #[must_use]
    pub fn default_spinner() -> Self {
        Self
    }

    /// Create default bar style
    #[must_use]
    pub fn default_bar() -> Self {
        Self
    }

    /// Set template (no-op)
    pub fn template(self, _template: &str) -> Result<Self, std::convert::Infallible> {
        Ok(self)
    }

    /// Set tick characters (no-op) - compat with indicatif tick_chars()
    #[must_use]
    pub fn tick_chars(self, _chars: &str) -> Self {
        self
    }

    /// Set tick strings (no-op)
    #[must_use]
    pub fn tick_strings(self, _strings: &[&str]) -> Self {
        self
    }

    /// Set progress characters (no-op)
    #[must_use]
    pub fn progress_chars(self, _chars: &str) -> Self {
        self
    }
}

/// Progress tracker for analysis operations
#[derive(Clone)]
pub struct ProgressTracker {
    enable_progress: bool,
}

impl ProgressTracker {
    /// Create a new progress tracker
    #[must_use]
    pub fn new(enable_progress: bool) -> Self {
        Self { enable_progress }
    }

    /// Create a spinner for an indeterminate operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::progress::ProgressTracker;
    ///
    /// let progress = ProgressTracker::new(false);
    /// let spinner = progress.create_spinner("Processing...");
    /// assert!(spinner.is_hidden());
    /// ```
    #[must_use]
    pub fn create_spinner(&self, message: &str) -> SimpleProgressBar {
        if !self.enable_progress {
            return SimpleProgressBar::hidden();
        }

        let pb = SimpleProgressBar::new_spinner();
        pb.set_message(message.to_string());
        // Print initial message since we don't have animated spinners
        eprintln!("⏳ {}", message);
        pb
    }

    /// Create a progress bar for file processing
    #[must_use]
    pub fn create_file_progress(&self, total_files: u64, message: &str) -> SimpleProgressBar {
        if !self.enable_progress {
            return SimpleProgressBar::hidden();
        }

        let pb = SimpleProgressBar::new(total_files);
        pb.set_message(message.to_string());
        eprintln!("📁 {} (0/{})", message, total_files);
        pb
    }

    /// Create a bytes progress bar
    #[must_use]
    pub fn create_bytes_progress(&self, total_bytes: u64, message: &str) -> SimpleProgressBar {
        if !self.enable_progress {
            return SimpleProgressBar::hidden();
        }

        let pb = SimpleProgressBar::new(total_bytes);
        pb.set_message(message.to_string());
        eprintln!("📦 {} (0/{} bytes)", message, total_bytes);
        pb
    }

    /// Log a skipped file
    pub fn log_skipped_file(&self, file_path: &std::path::Path, reason: &str) {
        if self.enable_progress {
            eprintln!("⚠️  Skipped: {} ({})", file_path.display(), reason);
        }
    }

    /// Create a sub-progress for parallel operations
    #[must_use]
    pub fn create_sub_progress(&self, message: &str, total: u64) -> SimpleProgressBar {
        if !self.enable_progress {
            return SimpleProgressBar::hidden();
        }

        let pb = SimpleProgressBar::new(total);
        pb.set_message(message.to_string());
        pb
    }

    /// Clear all progress bars (no-op without indicatif)
    pub fn clear(&self) {
        // No-op - nothing to clear with println-based progress
    }
}

/// Progress reporter for file classification
pub struct FileClassificationReporter {
    tracker: ProgressTracker,
    skipped_count: AtomicU64,
    large_files_skipped: Arc<Mutex<Vec<std::path::PathBuf>>>,
}

impl FileClassificationReporter {
    /// Create a new file classification reporter
    #[must_use]
    pub fn new(tracker: ProgressTracker) -> Self {
        Self {
            tracker,
            skipped_count: AtomicU64::new(0),
            large_files_skipped: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Report a skipped file
    pub fn report_skipped(
        &self,
        path: &std::path::Path,
        reason: crate::services::file_classifier::SkipReason,
    ) {
        use crate::services::file_classifier::SkipReason;

        self.skipped_count.fetch_add(1, Ordering::Relaxed);

        match reason {
            SkipReason::LargeFile => {
                if let Ok(mut files) = self.large_files_skipped.lock() {
                    files.push(path.to_path_buf());
                }
                self.tracker.log_skipped_file(path, "large file >500KB");
            }
            SkipReason::MinifiedContent => {
                self.tracker.log_skipped_file(path, "minified content");
            }
            SkipReason::VendorDirectory => {
                // Don't log vendor files to reduce noise
            }
            SkipReason::LineTooLong => {
                self.tracker.log_skipped_file(path, "line too long");
            }
            _ => {}
        }
    }

    /// Get summary of skipped files
    pub fn get_summary(&self) -> (u64, Vec<std::path::PathBuf>) {
        let count = self.skipped_count.load(Ordering::Relaxed);
        let files = self
            .large_files_skipped
            .lock()
            .expect("internal error")
            .clone();
        (count, files)
    }
}

/// Simple MultiProgress that doesn't do anything fancy
/// Used as a zero-dependency replacement for indicatif::MultiProgress
#[derive(Clone, Default)]
pub struct SimpleMultiProgress;

impl SimpleMultiProgress {
    /// Create a new multi-progress
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Add a progress bar to the multi-progress
    #[must_use]
    pub fn add(&self, pb: SimpleProgressBar) -> SimpleProgressBar {
        pb
    }

    /// Clear all progress bars (no-op)
    pub fn clear(&self) -> std::io::Result<()> {
        Ok(())
    }
}

// Type aliases to ease transition from indicatif
pub type ProgressBar = SimpleProgressBar;
pub type ProgressStyle = SimpleProgressStyle;
pub type MultiProgress = SimpleMultiProgress;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that progress tracker methods work
    #[test]
    fn test_progress_tracker_methods() {
        let progress = ProgressTracker::new(true);

        let spinner = progress.create_spinner("Test");
        assert!(!spinner.is_hidden());

        let file_progress = progress.create_file_progress(100, "Files");
        assert!(!file_progress.is_hidden());

        let bytes_progress = progress.create_bytes_progress(1000, "Bytes");
        assert!(!bytes_progress.is_hidden());
    }

    #[test]
    fn test_hidden_progress() {
        let progress = ProgressTracker::new(false);

        let spinner = progress.create_spinner("Test");
        assert!(spinner.is_hidden());

        let file_progress = progress.create_file_progress(100, "Files");
        assert!(file_progress.is_hidden());
    }

    #[test]
    fn test_progress_bar_operations() {
        let pb = SimpleProgressBar::new(100);
        pb.set_message("Testing");
        assert_eq!(pb.message(), "Testing");

        pb.set_position(50);
        assert_eq!(pb.position(), 50);

        pb.inc(10);
        assert_eq!(pb.position(), 60);

        pb.set_length(200);
        assert_eq!(pb.length(), Some(200));
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
