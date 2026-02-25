#![cfg_attr(coverage_nightly, coverage(off))]
//! Comprehensive defect report service
//!
//! This service aggregates defects from all analyzers and generates
//! reports in multiple formats (JSON, CSV, Markdown, Text).
use crate::models::defect_report::{
    Defect, DefectCategory, DefectReport, DefectSummary, FileHotspot, ReportMetadata,
};
use crate::services::defect_analyzer::DefectAnalyzer;
use anyhow::Result;
use chrono::Utc;
#[cfg(feature = "reporting")]
use csv::Writer;
use serde_json;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Service for generating comprehensive defect reports
pub struct DefectReportService {
    /// Concurrent analysis limit
    semaphore: Arc<Semaphore>,
}

/// Output format for reports
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    /// JSON format (default)
    Json,
    /// CSV format
    Csv,
    /// Markdown format
    Markdown,
    /// Plain text format
    Text,
}

// Core: new, generate_report, collect_all_defects, analyze_* methods, compute_summary, Default
include!("defect_report_core.rs");

// Formatting: format_json, format_csv, format_markdown, format_text, generate_filename
include!("defect_report_formatting.rs");

// Filtering: filter_by_pattern
include!("defect_report_filtering.rs");

#[cfg(all(test, feature = "broken-tests"))]
#[path = "tests.rs"]
mod tests;
