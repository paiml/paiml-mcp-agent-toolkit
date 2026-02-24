#![cfg_attr(coverage_nightly, coverage(off))]
//! Sprint 31 Week 2 - Export Capabilities
//!
//! Provides comprehensive export functionality for TDG analysis results,
//! metrics, and reports in various industry-standard formats.
//!
//! ## Module Organization
//! - `export_json.rs` - JSON export implementations
//! - `export_csv.rs` - CSV export implementations
//! - `export_sarif.rs` - SARIF (Static Analysis Results Interchange Format) exports
//! - `export_html.rs` - HTML export with styled reports and recommendations
//! - `export_markdown.rs` - Markdown export with recommendations engine
//! - `export_xml.rs` - XML export implementations
//! - `export_tests.rs` - Unit and property-based tests

use crate::tdg::{Comparison, Grade, ProjectScore, TdgScore};
use anyhow::Result;
use serde_json::json;
use std::time::SystemTime;

/// Export format types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Sarif,
    Html,
    Markdown,
    Xml,
    Prometheus,
    Grafana,
}

/// Export options configuration
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_recommendations: bool,
    pub include_metrics: bool,
    pub include_trends: bool,
    pub pretty_print: bool,
    pub compression: Option<CompressionType>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_metadata: true,
            include_recommendations: true,
            include_metrics: true,
            include_trends: false,
            pretty_print: true,
            compression: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    Gzip,
    Zstd,
    Lz4,
}

/// Main exporter for TDG data
pub struct TdgExporter;

impl TdgExporter {
    /// Export TDG score
    pub fn export_score(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        match options.format {
            ExportFormat::Json => Self::score_to_json(score, options),
            ExportFormat::Csv => Self::score_to_csv(score, options),
            ExportFormat::Sarif => Self::score_to_sarif(score, options),
            ExportFormat::Html => Self::score_to_html(score, options),
            ExportFormat::Markdown => Self::score_to_markdown(score, options),
            ExportFormat::Xml => Self::score_to_xml(score, options),
            _ => Err(anyhow::anyhow!("Unsupported format for score export")),
        }
    }

    /// Export project scores
    pub fn export_project(project: &ProjectScore, options: &ExportOptions) -> Result<String> {
        match options.format {
            ExportFormat::Json => Self::project_to_json(project, options),
            ExportFormat::Csv => Self::project_to_csv(project, options),
            ExportFormat::Sarif => Self::project_to_sarif(project, options),
            ExportFormat::Html => Self::project_to_html(project, options),
            ExportFormat::Markdown => Self::project_to_markdown(project, options),
            _ => Err(anyhow::anyhow!("Unsupported format for project export")),
        }
    }

    /// Export comparison results
    pub fn export_comparison(comparison: &Comparison, options: &ExportOptions) -> Result<String> {
        match options.format {
            ExportFormat::Json => Self::comparison_to_json(comparison, options),
            ExportFormat::Csv => Self::comparison_to_csv(comparison, options),
            ExportFormat::Html => Self::comparison_to_html(comparison, options),
            ExportFormat::Markdown => Self::comparison_to_markdown(comparison, options),
            _ => Err(anyhow::anyhow!("Unsupported format for comparison export")),
        }
    }
}

// Format-specific implementations
include!("export_json.rs");
include!("export_csv.rs");
include!("export_sarif.rs");
include!("export_html.rs");
include!("export_markdown.rs");
include!("export_xml.rs");

// Tests
include!("export_tests.rs");
