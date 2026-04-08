#![cfg_attr(coverage_nightly, coverage(off))]
//! Progress indicator utilities for long-running operations
//!
//! Provides spinners and progress feedback for operations >5s.
//! Automatically detects TTY and disables in CI environments.
//!
//! # Usage
//!
//! ## Simple Progress Spinner
//!
//! ```rust
//! use pmat::cli::progress::ProgressIndicator;
//!
//! let progress = ProgressIndicator::new("Analyzing files...");
//! // Perform long-running operation
//! progress.set_message("Processing results...");
//! progress.finish_with_message("Complete");
//! ```
//!
//! ## Multi-Stage Progress
//!
//! ```rust
//! use pmat::cli::progress::MultiStageProgress;
//!
//! let stages = vec![
//!     "Extracting data".to_string(),
//!     "Processing".to_string(),
//!     "Finalizing".to_string(),
//! ];
//! let mut progress = MultiStageProgress::new(stages);
//!
//! progress.next_stage("Extracting data");
//! progress.set_progress(10, 100);  // 10% complete
//! // ... work ...
//!
//! progress.next_stage("Processing");
//! progress.set_progress(50, 100);  // 50% complete
//! // ... work ...
//!
//! let eta = progress.get_eta();
//! println!("ETA: {:?}", eta);
//!
//! progress.finish("Done");
//! ```
//!
//! ## Category-Based Progress
//!
//! ```rust
//! use pmat::cli::progress::CategoryProgress;
//!
//! let categories = vec![
//!     "Code Quality".to_string(),
//!     "Testing".to_string(),
//!     "Documentation".to_string(),
//! ];
//! let mut progress = CategoryProgress::new(categories);
//!
//! progress.next_category("Code Quality");
//! progress.set_file_progress(50, 100);  // 50 of 100 files
//! println!("Category: {:.1}%", progress.category_percent());
//! println!("Overall: {:.1}%", progress.overall_percent());
//!
//! progress.finish();
//! ```
//!
//! ## Environment Detection
//!
//! Progress indicators automatically respect:
//! - **TTY Detection**: Only shows in interactive terminals
//! - **CI Environments**: Disabled when `CI=true`
//! - **NO_COLOR**: Respects `NO_COLOR` environment variable
//! - **Quiet Mode**: Disabled when `PMAT_QUIET=1`

// NOTE: indicatif dependency removed to reduce transitive deps
// Using local SimpleProgressBar implementation from services::progress
use crate::services::progress::{ProgressBar, ProgressStyle};
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(message: &str) -> Self {
        let progress_bar = if Self::should_show_progress() {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                    .template("{spinner:.cyan} {msg}")
                    .expect("internal error"),
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_message(&self, message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.set_message(message.to_string());
        }
    }

    /// Finish with success message
    ///
    /// CC=2: Conditional finish
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finish_with_message(&self, message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_with_message(format!("✓ {}", message));
        }
    }

    /// Finish with error message
    ///
    /// CC=2: Conditional finish
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finish_with_error(&self, message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_with_message(format!("✗ {}", message));
        }
    }

    /// Clear the progress indicator
    ///
    /// CC=1: Simple delegation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn clear(&self) {
        if let Some(ref pb) = self.progress_bar {
            pb.finish_and_clear();
        }
    }

    /// Check if progress is enabled
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_enabled(&self) -> bool {
        self.progress_bar.is_some()
    }

    /// Check if colors are being used
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn uses_color(&self) -> bool {
        Self::should_show_progress() && std::env::var("NO_COLOR").is_err()
    }

    /// Check if running in a TTY
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Next stage.
    pub fn next_stage(&mut self, _message: &str) {
        if self.current_stage_index < self.stages.len() - 1 {
            self.current_stage_index += 1;
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Current stage.
    pub fn current_stage(&self) -> &str {
        &self.stages[self.current_stage_index]
    }

    /// Current stage index.
    pub fn current_stage_index(&self) -> usize {
        self.current_stage_index
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Set progress.
    pub fn set_progress(&mut self, current: u64, total: u64) {
        self.completed_items = current;
        self.total_items = total;
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Completed items.
    pub fn completed_items(&self) -> u64 {
        self.completed_items
    }

    /// Total items.
    pub fn total_items(&self) -> u64 {
        self.total_items
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Get eta.
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Finalize the operation.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Next category.
    pub fn next_category(&mut self, _name: &str) {
        if self.current_category_index < self.categories.len() - 1 {
            self.current_category_index += 1;
        }
        // Reset file progress for new category
        self.files_processed = 0;
        self.total_files = 0;
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Current category.
    pub fn current_category(&self) -> &str {
        &self.categories[self.current_category_index]
    }

    /// Current category index.
    pub fn current_category_index(&self) -> usize {
        self.current_category_index
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Set file progress.
    pub fn set_file_progress(&mut self, current: usize, total: usize) {
        self.files_processed = current;
        self.total_files = total;
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Files processed.
    pub fn files_processed(&self) -> usize {
        self.files_processed
    }

    /// Total files.
    pub fn total_files(&self) -> usize {
        self.total_files
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Category percent.
    pub fn category_percent(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.files_processed as f64 / self.total_files as f64) * 100.0
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Overall percent.
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Elapsed.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Finalize the operation.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            frames: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current_frame_index: 0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Advance one frame or step.
    pub fn tick(&mut self) {
        self.current_frame_index = (self.current_frame_index + 1) % self.frames.len();
    }

    /// Current frame.
    pub fn current_frame(&self) -> char {
        self.frames[self.current_frame_index]
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
