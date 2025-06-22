//! Progress tracking for analysis operations
//!
//! This module provides progress bars and status tracking for long-running
//! analysis operations to improve user experience.

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

/// Progress tracker for analysis operations
#[derive(Clone)]
pub struct ProgressTracker {
    multi: Arc<MultiProgress>,
    enable_progress: bool,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(enable_progress: bool) -> Self {
        Self {
            multi: Arc::new(MultiProgress::new()),
            enable_progress,
        }
    }

    /// Create a spinner for an indeterminate operation
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        if !self.enable_progress {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    /// Create a progress bar for file processing
    pub fn create_file_progress(&self, total_files: u64, message: &str) -> ProgressBar {
        if !self.enable_progress {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new(total_files));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {per_sec}",
                )
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Create a bytes progress bar
    pub fn create_bytes_progress(&self, total_bytes: u64, message: &str) -> ProgressBar {
        if !self.enable_progress {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new(total_bytes));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%)",
                )
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Log a skipped file
    pub fn log_skipped_file(&self, file_path: &std::path::Path, reason: &str) {
        if self.enable_progress {
            eprintln!("⚠️  Skipped: {} ({})", file_path.display(), reason);
        }
    }

    /// Create a sub-progress for parallel operations
    pub fn create_sub_progress(&self, message: &str, total: u64) -> ProgressBar {
        if !self.enable_progress {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {msg} [{bar:30.green/white}] {pos}/{len}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Clear all progress bars
    pub fn clear(&self) {
        if self.enable_progress {
            self.multi.clear().ok();
        }
    }
}

/// Progress reporter for file classification
pub struct FileClassificationReporter {
    tracker: ProgressTracker,
    skipped_count: std::sync::atomic::AtomicU64,
    large_files_skipped: std::sync::Arc<std::sync::Mutex<Vec<std::path::PathBuf>>>,
}

impl FileClassificationReporter {
    /// Create a new file classification reporter
    pub fn new(tracker: ProgressTracker) -> Self {
        Self {
            tracker,
            skipped_count: std::sync::atomic::AtomicU64::new(0),
            large_files_skipped: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Report a skipped file
    pub fn report_skipped(&self, path: &std::path::Path, reason: crate::services::file_classifier::SkipReason) {
        use crate::services::file_classifier::SkipReason;
        
        self.skipped_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
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
        let count = self.skipped_count.load(std::sync::atomic::Ordering::Relaxed);
        let files = self.large_files_skipped.lock().unwrap().clone();
        (count, files)
    }
}