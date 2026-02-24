//! Technical Debt Gradient (TDG) analysis handler
//!
//! Provides full-featured TDG analysis with support for single file,
//! multiple files (MCP composition), and project-wide analysis.

use crate::cli::enums::TdgOutputFormat;
use crate::models::tdg::{TDGScore, TDGSummary, TDGHotspot, TDGSeverity};
use crate::services::tdg_calculator::TDGCalculator;
use anyhow::Result;
use std::path::PathBuf;

// Helper utilities: percentile, primary factor identification, hour estimation
include!("tdg_handler_helpers.rs");

// Output formatting: table, JSON, markdown, SARIF, single-file, empty results
include!("tdg_handler_formatting.rs");

// Core analysis: single file, multiple files, project-wide, summary creation
include!("tdg_handler_analysis.rs");

// Main entry point and orchestration
include!("tdg_handler_entry.rs");

// Tests: unit tests and property tests
include!("tdg_handler_tests.rs");
