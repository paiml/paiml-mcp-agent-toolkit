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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Low, Severity::Low);
        assert_ne!(Severity::Low, Severity::Critical);
    }

    #[test]
    fn test_severity_indicator() {
        assert_eq!(Severity::Critical.indicator(), "●");
        assert_eq!(Severity::High.indicator(), "◐");
        assert_eq!(Severity::Medium.indicator(), "○");
        assert_eq!(Severity::Low.indicator(), "◌");
    }

    #[test]
    fn test_severity_color_code() {
        assert_eq!(Severity::Critical.color_code(), "\x1b[31m");
        assert_eq!(Severity::High.color_code(), "\x1b[33m");
        assert_eq!(Severity::Medium.color_code(), "\x1b[36m");
        assert_eq!(Severity::Low.color_code(), "\x1b[2m");
    }

    #[test]
    fn test_severity_serialization() {
        for severity in [
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ] {
            let json = serde_json::to_string(&severity).unwrap();
            let parsed: Severity = serde_json::from_str(&json).unwrap();
            assert_eq!(severity, parsed);
        }
    }

    // ==================== AndonStatus Tests ====================

    #[test]
    fn test_andon_status_display() {
        assert_eq!(AndonStatus::Green.display(), "GREEN ✓");
        assert_eq!(AndonStatus::Yellow.display(), "YELLOW ⚠");
        assert_eq!(AndonStatus::Red.display(), "RED ✗");
    }

    #[test]
    fn test_andon_status_equality() {
        assert_eq!(AndonStatus::Green, AndonStatus::Green);
        assert_ne!(AndonStatus::Green, AndonStatus::Red);
    }

    #[test]
    fn test_andon_status_serialization() {
        for status in [AndonStatus::Green, AndonStatus::Yellow, AndonStatus::Red] {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: AndonStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, parsed);
        }
    }

    // ==================== TrendDirection Tests ====================

    #[test]
    fn test_trend_direction_arrow() {
        assert_eq!(TrendDirection::Improving.arrow(), "↑");
        assert_eq!(TrendDirection::Degrading.arrow(), "↓");
        assert_eq!(TrendDirection::Stable.arrow(), "→");
    }

    #[test]
    fn test_trend_direction_serialization() {
        for direction in [
            TrendDirection::Improving,
            TrendDirection::Stable,
            TrendDirection::Degrading,
        ] {
            let json = serde_json::to_string(&direction).unwrap();
            let parsed: TrendDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(direction, parsed);
        }
    }

    // ==================== SourceLocation Tests ====================

    #[test]
    fn test_source_location_display() {
        let location = SourceLocation {
            file: PathBuf::from("src/main.rs"),
            line: 42,
            column: 10,
            scope: Some("main".to_string()),
        };

        let display = format!("{}", location);
        assert!(display.contains("src/main.rs"));
        assert!(display.contains("42"));
    }

    #[test]
    fn test_source_location_serialization() {
        let location = SourceLocation {
            file: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            scope: None,
        };

        let json = serde_json::to_string(&location).unwrap();
        let parsed: SourceLocation = serde_json::from_str(&json).unwrap();

        assert_eq!(location.file, parsed.file);
        assert_eq!(location.line, parsed.line);
    }

    // ==================== FixSuggestion Tests ====================

    #[test]
    fn test_fix_suggestion_creation() {
        let fix = FixSuggestion {
            description: "Add missing import".to_string(),
            confidence: 0.95,
            auto_fixable: true,
            effort_minutes: Some(5),
        };

        assert_eq!(fix.confidence, 0.95);
        assert!(fix.auto_fixable);
        assert_eq!(fix.effort_minutes, Some(5));
    }

    #[test]
    fn test_fix_suggestion_serialization() {
        let fix = FixSuggestion {
            description: "Fix".to_string(),
            confidence: 0.8,
            auto_fixable: false,
            effort_minutes: None,
        };

        let json = serde_json::to_string(&fix).unwrap();
        let parsed: FixSuggestion = serde_json::from_str(&json).unwrap();

        assert_eq!(fix.description, parsed.description);
        assert_eq!(fix.confidence, parsed.confidence);
    }

    // ==================== Finding Tests ====================

    fn create_test_finding(severity: Severity) -> Finding {
        Finding {
            id: "test-1".to_string(),
            category: "type_error".to_string(),
            severity,
            location: SourceLocation {
                file: PathBuf::from("test.rs"),
                line: 1,
                column: 1,
                scope: None,
            },
            message: "Test message".to_string(),
            confidence: 0.9,
            cluster_id: None,
            pagerank: None,
            community: None,
            anomaly_score: None,
            fix_suggestion: None,
        }
    }

    #[test]
    fn test_finding_creation() {
        let finding = create_test_finding(Severity::High);

        assert_eq!(finding.id, "test-1");
        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.confidence, 0.9);
    }

    #[test]
    fn test_finding_with_optional_fields() {
        let finding = Finding {
            id: "test-2".to_string(),
            category: "complexity".to_string(),
            severity: Severity::Medium,
            location: SourceLocation {
                file: PathBuf::from("lib.rs"),
                line: 10,
                column: 5,
                scope: Some("func".to_string()),
            },
            message: "High complexity".to_string(),
            confidence: 0.85,
            cluster_id: Some(1),
            pagerank: Some(0.5),
            community: Some("core".to_string()),
            anomaly_score: Some(0.3),
            fix_suggestion: Some(FixSuggestion {
                description: "Refactor".to_string(),
                confidence: 0.7,
                auto_fixable: false,
                effort_minutes: Some(30),
            }),
        };

        assert_eq!(finding.cluster_id, Some(1));
        assert_eq!(finding.pagerank, Some(0.5));
        assert!(finding.fix_suggestion.is_some());
    }

    #[test]
    fn test_finding_serialization() {
        let finding = create_test_finding(Severity::Low);

        let json = serde_json::to_string(&finding).unwrap();
        let parsed: Finding = serde_json::from_str(&json).unwrap();

        assert_eq!(finding.id, parsed.id);
        assert_eq!(finding.severity, parsed.severity);
    }

    // ==================== FindingCluster Tests ====================

    #[test]
    fn test_finding_cluster_creation() {
        let cluster = FindingCluster {
            id: 0,
            size: 5,
            primary_category: "type_errors".to_string(),
            cohesion: 0.85,
            description: "Type-related issues".to_string(),
            finding_ids: vec!["f1".to_string(), "f2".to_string()],
        };

        assert_eq!(cluster.id, 0);
        assert_eq!(cluster.size, 5);
        assert_eq!(cluster.finding_ids.len(), 2);
    }

    #[test]
    fn test_finding_cluster_serialization() {
        let cluster = FindingCluster {
            id: 1,
            size: 3,
            primary_category: "complexity".to_string(),
            cohesion: 0.9,
            description: "Complex functions".to_string(),
            finding_ids: vec![],
        };

        let json = serde_json::to_string(&cluster).unwrap();
        let parsed: FindingCluster = serde_json::from_str(&json).unwrap();

        assert_eq!(cluster.id, parsed.id);
        assert_eq!(cluster.cohesion, parsed.cohesion);
    }

    // ==================== CodeCommunity Tests ====================

    #[test]
    fn test_code_community_creation() {
        let community = CodeCommunity {
            name: "core".to_string(),
            modularity: 0.75,
            files: vec![PathBuf::from("core/mod.rs"), PathBuf::from("core/types.rs")],
            primary_issue: Some("complexity".to_string()),
            defect_count: 10,
        };

        assert_eq!(community.name, "core");
        assert_eq!(community.files.len(), 2);
        assert_eq!(community.defect_count, 10);
    }

    #[test]
    fn test_code_community_serialization() {
        let community = CodeCommunity {
            name: "utils".to_string(),
            modularity: 0.6,
            files: vec![],
            primary_issue: None,
            defect_count: 0,
        };

        let json = serde_json::to_string(&community).unwrap();
        let parsed: CodeCommunity = serde_json::from_str(&json).unwrap();

        assert_eq!(community.name, parsed.name);
        assert_eq!(community.modularity, parsed.modularity);
    }

    // ==================== AnomalyPoint Tests ====================

    #[test]
    fn test_anomaly_point_creation() {
        let anomaly = AnomalyPoint {
            finding_id: "f-42".to_string(),
            score: 0.95,
            reason: "Unusually high complexity".to_string(),
            action: "Review for potential refactoring".to_string(),
        };

        assert_eq!(anomaly.finding_id, "f-42");
        assert_eq!(anomaly.score, 0.95);
    }

    #[test]
    fn test_anomaly_point_serialization() {
        let anomaly = AnomalyPoint {
            finding_id: "test".to_string(),
            score: 0.5,
            reason: "reason".to_string(),
            action: "action".to_string(),
        };

        let json = serde_json::to_string(&anomaly).unwrap();
        let parsed: AnomalyPoint = serde_json::from_str(&json).unwrap();

        assert_eq!(anomaly.score, parsed.score);
    }

    // ==================== MetricTrend Tests ====================

    #[test]
    fn test_metric_trend_creation() {
        let trend = MetricTrend {
            name: "coverage".to_string(),
            current: 85.5,
            direction: TrendDirection::Improving,
            change_percent: 2.5,
            sparkline: vec![3, 4, 5, 5, 6, 6, 7],
            forecast: Some(88.0),
        };

        assert_eq!(trend.name, "coverage");
        assert_eq!(trend.direction, TrendDirection::Improving);
        assert_eq!(trend.sparkline.len(), 7);
    }

    #[test]
    fn test_metric_trend_serialization() {
        let trend = MetricTrend {
            name: "test".to_string(),
            current: 50.0,
            direction: TrendDirection::Stable,
            change_percent: 0.0,
            sparkline: vec![4, 4, 4],
            forecast: None,
        };

        let json = serde_json::to_string(&trend).unwrap();
        let parsed: MetricTrend = serde_json::from_str(&json).unwrap();

        assert_eq!(trend.name, parsed.name);
        assert_eq!(trend.current, parsed.current);
    }

    // ==================== OutputFormat Tests ====================

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::default();
        assert_eq!(format, OutputFormat::Text);
    }

    #[test]
    fn test_output_format_serialization() {
        for format in [
            OutputFormat::Text,
            OutputFormat::Json,
            OutputFormat::Markdown,
        ] {
            let json = serde_json::to_string(&format).unwrap();
            let parsed: OutputFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(format, parsed);
        }
    }

    // ==================== ColorMode Tests ====================

    #[test]
    fn test_color_mode_default() {
        let mode = ColorMode::default();
        assert_eq!(mode, ColorMode::Auto);
    }

    #[test]
    fn test_color_mode_serialization() {
        for mode in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: ColorMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, parsed);
        }
    }

    // ==================== ReportConfig Tests ====================

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();

        assert_eq!(config.format, OutputFormat::Text);
        assert_eq!(config.color, ColorMode::Auto);
        assert_eq!(config.width, 80);
        assert_eq!(config.k_clusters, 4);
        assert_eq!(config.pagerank_damping, 0.85);
        assert_eq!(config.louvain_resolution, 1.0);
        assert_eq!(config.anomaly_threshold, 0.7);
        assert_eq!(config.trend_window_days, 30);
    }

    #[test]
    fn test_report_config_serialization() {
        let config = ReportConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        let parsed: ReportConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.width, parsed.width);
        assert_eq!(config.k_clusters, parsed.k_clusters);
    }

    // ==================== RichReport Tests ====================

    #[test]
    fn test_rich_report_new() {
        let report = RichReport::new("Test Report", "test-project");

        assert_eq!(report.title, "Test Report");
        assert_eq!(report.project, "test-project");
        assert_eq!(report.andon_status, AndonStatus::Green);
        assert_eq!(report.quality_score, 100.0);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_rich_report_calculate_andon_status_green() {
        let mut report = RichReport::new("Test", "proj");
        report.findings.push(create_test_finding(Severity::Low));
        report.findings.push(create_test_finding(Severity::Medium));

        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Green);
    }

    #[test]
    fn test_rich_report_calculate_andon_status_yellow() {
        let mut report = RichReport::new("Test", "proj");
        report.findings.push(create_test_finding(Severity::Low));
        report.findings.push(create_test_finding(Severity::High));

        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Yellow);
    }

    #[test]
    fn test_rich_report_calculate_andon_status_red() {
        let mut report = RichReport::new("Test", "proj");
        report.findings.push(create_test_finding(Severity::Low));
        report
            .findings
            .push(create_test_finding(Severity::Critical));

        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Red);
    }

    #[test]
    fn test_rich_report_findings_by_severity() {
        let mut report = RichReport::new("Test", "proj");
        report.findings.push(create_test_finding(Severity::Low));
        report.findings.push(create_test_finding(Severity::Low));
        report.findings.push(create_test_finding(Severity::High));
        report
            .findings
            .push(create_test_finding(Severity::Critical));

        let counts = report.findings_by_severity();

        assert_eq!(counts.get(&Severity::Low), Some(&2));
        assert_eq!(counts.get(&Severity::High), Some(&1));
        assert_eq!(counts.get(&Severity::Critical), Some(&1));
        assert_eq!(counts.get(&Severity::Medium), None);
    }

    #[test]
    fn test_rich_report_findings_by_severity_empty() {
        let report = RichReport::new("Test", "proj");
        let counts = report.findings_by_severity();
        assert!(counts.is_empty());
    }

    #[test]
    fn test_rich_report_serialization() {
        let report = RichReport::new("Test Report", "test-project");

        let json = serde_json::to_string(&report).unwrap();
        let parsed: RichReport = serde_json::from_str(&json).unwrap();

        assert_eq!(report.title, parsed.title);
        assert_eq!(report.project, parsed.project);
        assert_eq!(report.andon_status, parsed.andon_status);
    }

    #[test]
    fn test_rich_report_with_all_data() {
        let mut report = RichReport::new("Complete Report", "full-project");

        // Add findings
        report.findings.push(create_test_finding(Severity::High));

        // Add cluster
        report.clusters.push(FindingCluster {
            id: 0,
            size: 1,
            primary_category: "type".to_string(),
            cohesion: 0.9,
            description: "test".to_string(),
            finding_ids: vec!["test-1".to_string()],
        });

        // Add community
        report.communities.push(CodeCommunity {
            name: "main".to_string(),
            modularity: 0.8,
            files: vec![PathBuf::from("main.rs")],
            primary_issue: None,
            defect_count: 1,
        });

        // Add anomaly
        report.anomalies.push(AnomalyPoint {
            finding_id: "test-1".to_string(),
            score: 0.8,
            reason: "anomalous".to_string(),
            action: "investigate".to_string(),
        });

        // Add trend
        report.trends.push(MetricTrend {
            name: "coverage".to_string(),
            current: 80.0,
            direction: TrendDirection::Improving,
            change_percent: 5.0,
            sparkline: vec![4, 5, 6],
            forecast: Some(85.0),
        });

        // Add summary
        report
            .summary
            .insert("total_findings".to_string(), "1".to_string());

        // Add recommendations
        report.recommendations.push("Fix type error".to_string());

        // Calculate status
        report.calculate_andon_status();

        assert_eq!(report.andon_status, AndonStatus::Yellow);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.communities.len(), 1);
        assert_eq!(report.anomalies.len(), 1);
        assert_eq!(report.trends.len(), 1);
        assert_eq!(report.recommendations.len(), 1);
    }
}
