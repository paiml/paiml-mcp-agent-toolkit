// Included from progress.rs — SimpleProgressStyle, ProgressTracker, FileClassificationReporter,
// SimpleMultiProgress impls
// NO use imports or #! inner attributes allowed (shares parent module scope)

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
