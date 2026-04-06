#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-REPORT-V1: Universal Rich Reporting with Data Science and ASCII Visualization
//!
//! This module provides a unified reporting framework for ALL PMAT commands,
//! integrating advanced data science methods with Toyota Way principles.
//!
//! # Toyota Way Foundations
//!
//! - **Mieruka (Visual Management)**: ASCII progress bars, box drawing, color-coded severity
//! - **Genchi Genbutsu (Go and See)**: Evidence-based reports with confidence scores
//! - **Jidoka (Built-in Quality)**: Prioritized recommendations with auto-fix markers
//! - **Muda Elimination**: Inverted pyramid structure, progressive disclosure
//!
//! # Data Science Methods
//!
//! - **K-Means Clustering**: Group similar defects for batch remediation
//! - **PageRank Centrality**: Identify high-impact defects via dependency graph
//! - **Louvain Community Detection**: Discover architectural boundaries
//! - **Isolation Forest**: Detect anomalous code patterns
//! - **Time Series Analysis**: Track quality trends with change point detection
//!
//! # Example
//!
//! ```rust,ignore
//! use pmat::services::rich_reporter::{RichReporter, ReportConfig};
//!
//! let config = ReportConfig::default();
//! let reporter = RichReporter::new(config);
//!
//! // Add findings
//! reporter.add_finding(finding);
//!
//! // Analyze with data science
//! reporter.analyze();
//!
//! // Render report
//! let output = reporter.render_text();
//! println!("{}", output);
//! ```

pub mod ascii_viz;
pub mod data_science;
pub mod types;

pub use ascii_viz::*;
pub use data_science::DataScienceAnalyzer;
pub use types::*;

use std::fmt::Write;

/// Universal rich reporter for PMAT commands
pub struct RichReporter {
    /// Report configuration
    config: ReportConfig,
    /// The report being built
    report: RichReport,
    /// Data science analyzer
    analyzer: DataScienceAnalyzer,
    /// File dependencies for PageRank/Louvain
    dependencies: Vec<(String, String)>,
    /// Metric history for trends
    metric_history: Vec<(String, Vec<(i64, f64)>)>,
}

impl RichReporter {
    /// Create a new rich reporter
    pub fn new(config: ReportConfig) -> Self {
        let analyzer = DataScienceAnalyzer::new(
            config.k_clusters,
            config.pagerank_damping,
            config.louvain_resolution,
            config.anomaly_threshold,
        );

        RichReporter {
            config,
            report: RichReport::new("PMAT Report", ""),
            analyzer,
            dependencies: Vec::new(),
            metric_history: Vec::new(),
        }
    }

    /// Set report title
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.report.title = title.into();
        self
    }

    /// Set project name
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.report.project = project.into();
        self
    }

    /// Add a finding to the report
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_finding(&mut self, finding: Finding) {
        self.report.findings.push(finding);
    }

    /// Add multiple findings
    pub fn add_findings(&mut self, findings: impl IntoIterator<Item = Finding>) {
        self.report.findings.extend(findings);
    }

    /// Add file dependencies for PageRank/Louvain analysis
    pub fn add_dependency(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.dependencies.push((from.into(), to.into()));
    }

    /// Add metric history for trend analysis
    pub fn add_metric_history(&mut self, name: impl Into<String>, data: Vec<(i64, f64)>) {
        self.metric_history.push((name.into(), data));
    }

    /// Set quality score
    pub fn set_quality_score(&mut self, score: f64) {
        self.report.quality_score = score;
    }

    /// Add a summary metric
    pub fn add_summary(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.report.summary.insert(key.into(), value.into());
    }

    /// Add a recommendation
    pub fn add_recommendation(&mut self, recommendation: impl Into<String>) {
        self.report.recommendations.push(recommendation.into());
    }
}

// Data science analysis: analyze(), generate_recommendations()
include!("rich_reporter_analysis.rs");

// Rendering: render_text(), render_json(), render_markdown(), render(), report(), report_mut()
include!("rich_reporter_rendering.rs");

// Unit tests
include!("rich_reporter_tests.rs");
