#![cfg_attr(coverage_nightly, coverage(off))]
//! File Health Score Service
//!
//! Implements the File Health Score specification (docs/specifications/max-lines.md).
//! Detects, prevents, and reports on excessively large source files.
//!
//! # Scientific Foundation
//!
//! Based on peer-reviewed research showing correlation between file size and defects:
//! - Hindle et al. (2008): Files >500 lines exhibit exponential defect density increase
//! - Nagappan et al. (2006): r=0.67 correlation between LOC and defect count
//! - Bird et al. (2011): Files >400 LOC show ownership fragmentation
//!
//! # Toyota Way Principles
//!
//! - **Jidoka**: Pre-commit hook blocks large file creation
//! - **Kaizen**: Ratchet mechanism forces gradual reduction
//! - **Muda**: Large files create cognitive waste

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ── Types: FileSizeClass, HealthGrade, FileHealthMetrics ───────────────────
include!("file_health_types.rs");

// ── FileHealthReport ───────────────────────────────────────────────────────
include!("file_health_report.rs");

// ── Baselines, analysis functions, stack types, constants ──────────────────
include!("file_health_baseline.rs");

// ── Tests ──────────────────────────────────────────────────────────────────
include!("file_health_tests.rs");
