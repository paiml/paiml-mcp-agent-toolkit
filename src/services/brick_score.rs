#![cfg_attr(coverage_nightly, coverage(off))]
//! ComputeBrick Profiling Score Service (PMAT-446)
//!
//! Reads BrickProfiler JSON output and calculates a 100-point score:
//! - Performance (40 pts): Throughput vs theoretical peak
//! - Efficiency (25 pts): Backend utilization, memory efficiency
//! - Correctness (20 pts): Assertions passing, numerical accuracy
//! - Stability (15 pts): CV < 5%, reproducibility
//!
//! PMAT-448: Hardware-aware scoring via ~/.pmat/hardware.toml
//!
//! Reference: aprender/docs/specifications/qwen2.5-coder-showcase-demo.md §2.5

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Hardware Capability Types (PMAT-448)
// ============================================================================
include!("brick_score_hardware.rs");

// ============================================================================
// Brick Types, Budgets, and Arithmetic Intensity
// ============================================================================
include!("brick_score_types.rs");

// ============================================================================
// Scoring Functions
// ============================================================================
include!("brick_score_scoring.rs");

// ============================================================================
// Tests
// ============================================================================
include!("brick_score_tests.rs");
