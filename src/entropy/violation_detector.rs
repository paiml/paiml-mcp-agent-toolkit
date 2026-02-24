#![cfg_attr(coverage_nightly, coverage(off))]
//! Actionable Violation Detection
//!
//! Detects actionable violations with clear fixes and LOC reduction estimates

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::entropy_calculator::EntropyMetrics;
use super::pattern_extractor::{AstPattern, PatternCollection};
use super::{EntropyConfig, PatternType};

// Type definitions: Severity, ActionableViolation, PatternSummary
include!("violation_detector_types.rs");

// ViolationDetector struct and impl block
include!("violation_detector_impl.rs");

// Tests: unit tests, property tests, coverage tests
include!("violation_detector_tests.rs");
