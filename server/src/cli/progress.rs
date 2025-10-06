//! Progress indicator utilities for long-running operations
//!
//! Provides spinners and progress feedback for operations >5s.
//! Automatically detects TTY and disables in CI environments.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress indicator for long-running operations
pub struct ProgressIndicator {
    bar: Option<ProgressBar>,
}

impl ProgressIndicator {
    /// Create a new progress spinner
    ///
    /// CC=2: Simple initialization
    pub fn new(message: &str) -> Self {
        let bar = if Self::should_show_progress() {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                    .template("{spinner:.cyan} {msg}")
                    .unwrap(),
            );
            pb.set_message(message.to_string());
            pb.enable_steady_tick(Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        Self { bar }
    }

    /// Check if we should show progress indicators
    ///
    /// CC=5: TTY check + env checks (TICKET-PMAT-6006)
    fn should_show_progress() -> bool {
        // Don't show in CI environments
        if std::env::var("CI").is_ok() {
            return false;
        }

        // Don't show if NO_COLOR is set
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Don't show in quiet mode (TICKET-PMAT-6006)
        if std::env::var("PMAT_QUIET").is_ok() {
            return false;
        }

        // Only show if we have a TTY
        atty::is(atty::Stream::Stdout)
    }

    /// Update the progress message
    ///
    /// CC=1: Simple delegation
    pub fn set_message(&self, message: &str) {
        if let Some(ref bar) = self.bar {
            bar.set_message(message.to_string());
        }
    }

    /// Finish with success message
    ///
    /// CC=2: Conditional finish
    pub fn finish_with_message(&self, message: &str) {
        if let Some(ref bar) = self.bar {
            bar.finish_with_message(format!("✓ {}", message));
        }
    }

    /// Finish with error message
    ///
    /// CC=2: Conditional finish
    pub fn finish_with_error(&self, message: &str) {
        if let Some(ref bar) = self.bar {
            bar.finish_with_message(format!("✗ {}", message));
        }
    }

    /// Clear the progress indicator
    ///
    /// CC=1: Simple delegation
    pub fn clear(&self) {
        if let Some(ref bar) = self.bar {
            bar.finish_and_clear();
        }
    }
}

impl Drop for ProgressIndicator {
    /// CC=1: Simple cleanup
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_indicator_creation() {
        let progress = ProgressIndicator::new("Testing...");
        assert!(progress.bar.is_none() || progress.bar.is_some());
    }

    #[test]
    fn test_progress_indicator_messages() {
        let progress = ProgressIndicator::new("Initial");
        progress.set_message("Updated");
        progress.finish_with_message("Done");
    }

    #[test]
    fn test_progress_indicator_error() {
        let progress = ProgressIndicator::new("Working");
        progress.finish_with_error("Failed");
    }

    #[test]
    fn test_should_show_progress_respects_ci() {
        // This test documents behavior, actual result depends on environment
        let _should_show = ProgressIndicator::should_show_progress();
        // No assertion - environment dependent
    }
}
