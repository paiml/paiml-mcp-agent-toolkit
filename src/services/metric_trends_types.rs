/// Metric observation (single data point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricObservation {
    /// Metric name (lint, test-fast, coverage, etc.)
    pub metric: String,
    /// Value (duration_ms, binary_size, etc.)
    pub value: f64,
    /// Unix timestamp (seconds since epoch)
    pub timestamp: i64,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Metric name
    pub metric: String,
    /// Number of observations
    pub count: usize,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Trend direction
    pub direction: TrendDirection,
    /// Regression slope (change per day)
    pub slope: f64,
    /// Statistical significance (p-value)
    pub p_value: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving (decreasing for durations/sizes)
    Improving,
    /// Stable (no significant change)
    Stable,
    /// Regressing (increasing for durations/sizes)
    Regressing,
}

/// Forecast point (Phase 4: Predictive Quality Gates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    /// Days ahead from last observation
    pub days_ahead: usize,
    /// Predicted value
    pub predicted_value: f64,
    /// Lower bound (95% confidence interval)
    pub lower_bound: f64,
    /// Upper bound (95% confidence interval)
    pub upper_bound: f64,
}

/// Prediction result (Phase 4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Metric name
    pub metric: String,
    /// Current value (last observation)
    pub current_value: f64,
    /// Threshold being checked
    pub threshold: f64,
    /// Days until threshold exceeded (None if no breach predicted)
    pub breach_in_days: Option<usize>,
    /// Predicted value at breach point
    pub predicted_value: Option<f64>,
    /// Prediction confidence (R² score, 0.0-1.0)
    pub confidence: f64,
    /// Actionable recommendations
    pub recommendations: Vec<String>,
    /// Forecast for next N days
    pub forecast: Vec<ForecastPoint>,
}

/// Linear regression model (internal)
#[derive(Debug, Clone)]
struct LinearModel {
    slope: f64,
    intercept: f64,
    r_squared: f64,
    last_timestamp: i64,
}

/// Metric trend storage (trueno-graph CSR backed)
pub struct MetricTrendStore {
    /// Storage directory (.pmat-metrics/trends/)
    storage_path: PathBuf,
    /// In-memory cache (metric_name → observations)
    cache: HashMap<String, Vec<MetricObservation>>,
    /// CSR graph for temporal relationships (Phase 3.2)
    /// Nodes: timestamp → MetricObservation
    /// Edges: (t_i → t_i+1) with weight Δt
    graph: CsrGraph,
    /// Node ID mapping (timestamp → NodeId)
    node_map: HashMap<i64, NodeId>,
    /// Reverse mapping (NodeId → timestamp)
    reverse_node_map: HashMap<NodeId, i64>,
    /// PageRank scores (metric_name → hotness score)
    hotness_cache: HashMap<String, f32>,
    /// Next node ID counter
    next_node_id: u32,
}
