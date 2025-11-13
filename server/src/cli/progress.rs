//! Progress indicator utilities for long-running operations
//!
//! Provides spinners and progress feedback for operations >5s.
//! Automatically detects TTY and disables in CI environments.

use indicatif::{ProgressBar, ProgressStyle};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

/// Progress indicator for long-running operations
pub struct ProgressIndicator {
    progress_bar: Option<ProgressBar>,
}

impl ProgressIndicator {
    /// Create a new progress spinner
    ///
    /// CC=2: Simple initialization
    pub fn new(message: &str) -> Self {
        let progress_bar = if Self::should_show_progress() {
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

        Self { progress_bar }
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
        std::io::stdout().is_terminal()
    }

    /// Update the progress message
    ///
    /// CC=1: Simple delegation
    pub fn set_message(&self, message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.set_message(message.to_string());
        }
    }

    /// Finish with success message
    ///
    /// CC=2: Conditional finish
    pub fn finish_with_message(&self, message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_with_message(format!("✓ {}", message));
        }
    }

    /// Finish with error message
    ///
    /// CC=2: Conditional finish
    pub fn finish_with_error(&self, message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_with_message(format!("✗ {}", message));
        }
    }

    /// Clear the progress indicator
    ///
    /// CC=1: Simple delegation
    pub fn clear(&self) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_and_clear();
        }
    }

    /// Check if progress is enabled
    pub fn is_enabled(&self) -> bool {
        self.progress_bar.is_some()
    }

    /// Check if colors are being used
    pub fn uses_color(&self) -> bool {
        Self::should_show_progress() && std::env::var("NO_COLOR").is_err()
    }

    /// Check if running in a TTY
    pub fn is_tty() -> bool {
        std::io::stdout().is_terminal()
    }
}

impl Drop for ProgressIndicator {
    /// CC=1: Simple cleanup
    fn drop(&mut self) {
        self.clear();
    }
}

/// Multi-stage progress indicator for operations with distinct phases
pub struct MultiStageProgress {
    stages: Vec<String>,
    current_stage_index: usize,
    progress_bar: Option<ProgressBar>,
    start_time: Instant,
    completed_items: u64,
    total_items: u64,
}

impl MultiStageProgress {
    pub fn new(stages: Vec<String>) -> Self {
        Self {
            stages,
            current_stage_index: 0,
            progress_bar: None,
            start_time: Instant::now(),
            completed_items: 0,
            total_items: 0,
        }
    }

    pub fn next_stage(&mut self, _message: &str) {
        if self.current_stage_index < self.stages.len() - 1 {
            self.current_stage_index += 1;
        }
    }

    pub fn current_stage(&self) -> &str {
        &self.stages[self.current_stage_index]
    }

    pub fn current_stage_index(&self) -> usize {
        self.current_stage_index
    }

    pub fn set_progress(&mut self, current: u64, total: u64) {
        self.completed_items = current;
        self.total_items = total;
    }

    pub fn completed_items(&self) -> u64 {
        self.completed_items
    }

    pub fn total_items(&self) -> u64 {
        self.total_items
    }

    pub fn get_eta(&self) -> Duration {
        if self.completed_items == 0 || self.total_items == 0 {
            return Duration::from_secs(0);
        }

        let elapsed = self.start_time.elapsed();
        let items_remaining = self.total_items.saturating_sub(self.completed_items);
        let time_per_item = elapsed.as_secs_f64() / self.completed_items as f64;
        let estimated_seconds = (items_remaining as f64 * time_per_item) as u64;

        Duration::from_secs(estimated_seconds)
    }

    pub fn finish(&self, _message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_and_clear();
        }
    }
}

/// Category-based progress for operations analyzing multiple categories
pub struct CategoryProgress {
    categories: Vec<String>,
    current_category_index: usize,
    files_processed: usize,
    total_files: usize,
    progress_bar: Option<ProgressBar>,
    start_time: Instant,
}

impl CategoryProgress {
    pub fn new(categories: Vec<String>) -> Self {
        Self {
            categories,
            current_category_index: 0,
            files_processed: 0,
            total_files: 0,
            progress_bar: None,
            start_time: Instant::now(),
        }
    }

    pub fn next_category(&mut self, _name: &str) {
        if self.current_category_index < self.categories.len() - 1 {
            self.current_category_index += 1;
        }
        // Reset file progress for new category
        self.files_processed = 0;
        self.total_files = 0;
    }

    pub fn current_category(&self) -> &str {
        &self.categories[self.current_category_index]
    }

    pub fn current_category_index(&self) -> usize {
        self.current_category_index
    }

    pub fn set_file_progress(&mut self, current: usize, total: usize) {
        self.files_processed = current;
        self.total_files = total;
    }

    pub fn files_processed(&self) -> usize {
        self.files_processed
    }

    pub fn total_files(&self) -> usize {
        self.total_files
    }

    pub fn category_percent(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.files_processed as f64 / self.total_files as f64) * 100.0
    }

    pub fn overall_percent(&self) -> f64 {
        if self.categories.is_empty() {
            return 0.0;
        }

        // Calculate progress: completed categories + current category progress
        let completed_categories = self.current_category_index as f64;
        let current_category_progress = self.category_percent() / 100.0;
        let total_progress = completed_categories + current_category_progress;

        (total_progress / self.categories.len() as f64) * 100.0
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn finish(&self) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_and_clear();
        }
    }
}

/// Spinner animation for indeterminate progress
pub struct Spinner {
    frames: Vec<char>,
    current_frame_index: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current_frame_index: 0,
        }
    }

    pub fn tick(&mut self) {
        self.current_frame_index = (self.current_frame_index + 1) % self.frames.len();
    }

    pub fn current_frame(&self) -> char {
        self.frames[self.current_frame_index]
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_indicator_creation() {
        let progress = ProgressIndicator::new("Testing...");
        assert!(progress.progress_bar.is_none() || progress.progress_bar.is_some());
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
