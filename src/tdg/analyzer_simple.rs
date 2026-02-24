#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::config::TdgConfig;
use super::language_simple::Language;
use crate::tdg::{Comparison, MetricCategory, PenaltyTracker, ProjectScore, TdgScore};

pub struct TdgAnalyzer {
    config: TdgConfig,
}

// Public API: new, with_config, analyze_file, analyze_source, analyze_project, compare
include!("analyzer_simple_core.rs");

// Metric analysis: structural complexity, semantic complexity, duplication, coupling, docs, consistency
include!("analyzer_simple_metrics.rs");

// Estimation helpers and file discovery
include!("analyzer_simple_helpers.rs");

// Unit tests and property tests
include!("analyzer_simple_tests.rs");
