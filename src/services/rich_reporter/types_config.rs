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
