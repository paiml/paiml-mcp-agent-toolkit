#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! ASCII Visualization Primitives for PMAT-REPORT-V1
//!
//! Implements Toyota Way Mieruka (Visual Management) through:
//! - Progress bars with thresholds
//! - Box drawing for structured output
//! - Sparklines for trends
//! - Tables for data presentation

use super::types::{Severity, TrendDirection};

// Widget primitives: ProgressBar, Sparkline, StatusIndicator
include!("ascii_viz_widgets.rs");

// Layout primitives: BoxDrawer, TableRenderer, TreeRenderer
include!("ascii_viz_layout.rs");

// Unit tests: basic tests + comprehensive coverage (widgets/sparklines)
include!("ascii_viz_tests.rs");

// Unit tests: comprehensive coverage (layout/table/tree/edge cases)
include!("ascii_viz_tests_part2.rs");
