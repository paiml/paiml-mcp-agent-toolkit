#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for defect prediction analysis to reduce complexity

use crate::services::defect_probability::{DefectScore, FileMetrics};
use anyhow::Result;
use std::path::{Path, PathBuf};

// Core analysis: structs, discovery, complexity, churn, metrics, filtering, risk distribution
include!("defect_prediction_analysis.rs");

// Output formatters: summary, recommendations, detailed, JSON, markdown, CSV, SARIF
include!("defect_prediction_formatters.rs");

// Tests and property tests
include!("defect_prediction_tests.rs");
