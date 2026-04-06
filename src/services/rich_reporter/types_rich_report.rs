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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn findings_by_severity(&self) -> HashMap<Severity, usize> {
        let mut counts = HashMap::new();
        for finding in &self.findings {
            *counts.entry(finding.severity).or_insert(0) += 1;
        }
        counts
    }
}
