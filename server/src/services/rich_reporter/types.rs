//! Core types for PMAT-REPORT-V1 Universal Rich Reporting
//!
//! Implements the Unified Finding and Report structures per specification.
//! Toyota Way: Mieruka (Visual Management) - all findings include visual indicators

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Severity level with Andon-style color mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity - dimmed in output
    Low,
    /// Medium severity - cyan
    Medium,
    /// High severity - yellow
    High,
    /// Critical severity - red (Andon RED)
    Critical,
}

impl Severity {
    /// Get the ASCII indicator for this severity
    pub fn indicator(&self) -> &'static str {
        match self {
            Severity::Critical => "●",
            Severity::High => "◐",
            Severity::Medium => "○",
            Severity::Low => "◌",
        }
    }

    /// Get the ANSI color code for this severity
    pub fn color_code(&self) -> &'static str {
        match self {
            Severity::Critical => "\x1b[31m", // Red
            Severity::High => "\x1b[33m",     // Yellow
            Severity::Medium => "\x1b[36m",   // Cyan
            Severity::Low => "\x1b[2m",       // Dim
        }
    }
}

/// Andon status (Toyota Way visual signal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AndonStatus {
    /// All checks pass - quality target met
    Green,
    /// Minor issues - attention needed
    Yellow,
    /// Critical issues - stop the line (Jidoka)
    Red,
}

impl AndonStatus {
    /// Get the ASCII representation
    pub fn display(&self) -> &'static str {
        match self {
            AndonStatus::Green => "GREEN ✓",
            AndonStatus::Yellow => "YELLOW ⚠",
            AndonStatus::Red => "RED ✗",
        }
    }
}

/// Trend direction for time-series metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving (value decreasing for costs/times)
    Improving,
    /// Stable (within tolerance)
    Stable,
    /// Degrading (value increasing for costs/times)
    Degrading,
}

impl TrendDirection {
    /// Get the ASCII arrow indicator
    pub fn arrow(&self) -> &'static str {
        match self {
            TrendDirection::Improving => "↑",
            TrendDirection::Degrading => "↓",
            TrendDirection::Stable => "→",
        }
    }
}

/// Source location for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Function or scope name
    pub scope: Option<String>,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file.display(), self.line)
    }
}

/// Suggested fix for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSuggestion {
    /// Description of the fix
    pub description: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Can this fix be auto-applied?
    pub auto_fixable: bool,
    /// Estimated effort (in minutes)
    pub effort_minutes: Option<u32>,
}

/// Individual finding with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier
    pub id: String,
    /// Defect category (from UDS)
    pub category: String,
    /// Severity level
    pub severity: Severity,
    /// Source code location
    pub location: SourceLocation,
    /// Human-readable message
    pub message: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// K-means cluster assignment
    pub cluster_id: Option<usize>,
    /// PageRank centrality score
    pub pagerank: Option<f32>,
    /// Louvain community assignment
    pub community: Option<String>,
    /// Isolation Forest anomaly score
    pub anomaly_score: Option<f32>,
    /// Suggested fix
    pub fix_suggestion: Option<FixSuggestion>,
}

/// Cluster of related findings (K-means output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingCluster {
    /// Cluster ID
    pub id: usize,
    /// Number of findings in cluster
    pub size: usize,
    /// Dominant category in cluster
    pub primary_category: String,
    /// Cluster cohesion score (0.0 - 1.0)
    pub cohesion: f64,
    /// Centroid description
    pub description: String,
    /// Finding IDs in this cluster
    pub finding_ids: Vec<String>,
}

/// Code community detected by Louvain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCommunity {
    /// Community name (derived from dominant file/module)
    pub name: String,
    /// Modularity score for this community
    pub modularity: f64,
    /// Files in this community
    pub files: Vec<PathBuf>,
    /// Primary issue type in this community
    pub primary_issue: Option<String>,
    /// Number of defects in community
    pub defect_count: usize,
}

/// Anomaly detected by Isolation Forest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPoint {
    /// Finding ID that is anomalous
    pub finding_id: String,
    /// Anomaly score (0.0 = normal, 1.0 = highly anomalous)
    pub score: f64,
    /// Reason for anomaly
    pub reason: String,
    /// Suggested action
    pub action: String,
}

/// Metric time series for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrend {
    /// Metric name
    pub name: String,
    /// Current value
    pub current: f64,
    /// Trend direction
    pub direction: TrendDirection,
    /// Change percentage
    pub change_percent: f64,
    /// Sparkline data (last N values normalized 0-7)
    pub sparkline: Vec<u8>,
    /// Forecast value (if available)
    pub forecast: Option<f64>,
}

/// Output format for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    /// Terminal-friendly text with ASCII art
    #[default]
    Text,
    /// Structured JSON
    Json,
    /// Markdown for documentation
    Markdown,
}

/// Color mode for terminal output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorMode {
    /// Auto-detect from terminal capabilities
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

/// Configuration for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Output format
    pub format: OutputFormat,
    /// Color mode
    pub color: ColorMode,
    /// Terminal width (for wrapping)
    pub width: usize,
    /// Number of clusters for K-means
    pub k_clusters: usize,
    /// PageRank damping factor
    pub pagerank_damping: f64,
    /// Louvain resolution parameter
    pub louvain_resolution: f64,
    /// Anomaly score threshold (0.0 - 1.0)
    pub anomaly_threshold: f64,
    /// Time window for trend analysis (days)
    pub trend_window_days: usize,
}

impl Default for ReportConfig {
    fn default() -> Self {
        ReportConfig {
            format: OutputFormat::Text,
            color: ColorMode::Auto,
            width: 80,
            k_clusters: 4,
            pagerank_damping: 0.85,
            louvain_resolution: 1.0,
            anomaly_threshold: 0.7,
            trend_window_days: 30,
        }
    }
}

/// Rich report output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichReport {
    /// Report title
    pub title: String,
    /// Project name
    pub project: String,
    /// Generation timestamp
    pub timestamp: String,
    /// Andon status
    pub andon_status: AndonStatus,
    /// Quality score (0.0 - 100.0)
    pub quality_score: f64,
    /// All findings
    pub findings: Vec<Finding>,
    /// Finding clusters
    pub clusters: Vec<FindingCluster>,
    /// Code communities
    pub communities: Vec<CodeCommunity>,
    /// Anomalies detected
    pub anomalies: Vec<AnomalyPoint>,
    /// Metric trends
    pub trends: Vec<MetricTrend>,
    /// Summary metrics
    pub summary: HashMap<String, String>,
    /// Recommendations (prioritized)
    pub recommendations: Vec<String>,
}

impl RichReport {
    /// Create a new empty report
    pub fn new(title: impl Into<String>, project: impl Into<String>) -> Self {
        RichReport {
            title: title.into(),
            project: project.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            andon_status: AndonStatus::Green,
            quality_score: 100.0,
            findings: Vec::new(),
            clusters: Vec::new(),
            communities: Vec::new(),
            anomalies: Vec::new(),
            trends: Vec::new(),
            summary: HashMap::new(),
            recommendations: Vec::new(),
        }
    }

    /// Calculate Andon status from findings
    pub fn calculate_andon_status(&mut self) {
        let critical_count = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count();
        let high_count = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .count();

        self.andon_status = if critical_count > 0 {
            AndonStatus::Red
        } else if high_count > 0 {
            AndonStatus::Yellow
        } else {
            AndonStatus::Green
        };
    }

    /// Count findings by severity
    pub fn findings_by_severity(&self) -> HashMap<Severity, usize> {
        let mut counts = HashMap::new();
        for finding in &self.findings {
            *counts.entry(finding.severity).or_insert(0) += 1;
        }
        counts
    }
}
