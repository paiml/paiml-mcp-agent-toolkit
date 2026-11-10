#![cfg_attr(coverage_nightly, coverage(off))]
//! Progress tracking for analysis operations
//!
//! This module provides progress bars and status tracking for long-running
//! analysis operations to improve user experience.
//!
//! NOTE: indicatif dependency removed to reduce transitive deps (saves 6 deps)
//! Using simple println-based progress reporting instead.
//!
//! ## Module structure
//!
//! - `progress_bar.rs`: SimpleProgressBar impl (Clone + methods)
//! - `progress_tracking.rs`: SimpleProgressStyle, ProgressTracker,
//!   FileClassificationReporter, SimpleMultiProgress impls
//! - `progress_tests.rs`: Unit tests and property tests

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Zero-dependency progress bar with the indicatif::ProgressBar API.
/// `set_message` / `tick` / `inc` emit a throttled one-line stderr update
/// (`\r`-rewritten); nothing is written unless stderr is a TTY, and `hidden`
/// bars and `--quiet` stay silent (#1236).
pub struct SimpleProgressBar {
    last_emit: Arc<Mutex<Option<std::time::Instant>>>,
    message: Arc<Mutex<String>>,
    position: Arc<AtomicU64>,
    length: Arc<AtomicU64>,
    hidden: bool,
}

/// No-op progress style that mimics indicatif::ProgressStyle API
#[derive(Clone, Default)]
pub struct SimpleProgressStyle;

/// Progress tracker for analysis operations
#[derive(Clone)]
pub struct ProgressTracker {
    enable_progress: bool,
}

/// Progress reporter for file classification
pub struct FileClassificationReporter {
    tracker: ProgressTracker,
    skipped_count: AtomicU64,
    large_files_skipped: Arc<Mutex<Vec<std::path::PathBuf>>>,
}

/// Simple MultiProgress that doesn't do anything fancy
/// Used as a zero-dependency replacement for indicatif::MultiProgress
#[derive(Clone, Default)]
pub struct SimpleMultiProgress;

// Type aliases to ease transition from indicatif
pub type ProgressBar = SimpleProgressBar;
pub type ProgressStyle = SimpleProgressStyle;
pub type MultiProgress = SimpleMultiProgress;

// --- Submodule includes ---

include!("progress_bar.rs");
include!("progress_tracking.rs");
include!("progress_tests.rs");
