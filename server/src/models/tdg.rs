use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Technical Debt Gradient (TDG) - Primary code quality metric
/// Replaces defect probability throughout the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TDGScore {
    /// The calculated TDG value (typically 0.0 - 5.0)
    pub value: f64,

    /// Component breakdown for transparency
    pub components: TDGComponents,

    /// Severity classification based on thresholds
    pub severity: TDGSeverity,

    /// Percentile ranking within the codebase
    pub percentile: f64,

    /// Confidence level of the calculation (0.0 - 1.0)
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TDGComponents {
    /// Complexity contribution (cognitive + cyclomatic)
    pub complexity: f64,

    /// Code churn velocity contribution
    pub churn: f64,

    /// Coupling score contribution
    pub coupling: f64,

    /// Domain-specific risk factors
    pub domain_risk: f64,

    /// Code duplication factor
    pub duplication: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TDGSeverity {
    /// TDG < 1.5 - Normal technical debt levels
    Normal,

    /// TDG 1.5-2.5 - Elevated technical debt requiring attention
    Warning,

    /// TDG > 2.5 - Critical technical debt requiring immediate action
    Critical,
}

impl From<f64> for TDGSeverity {
    fn from(value: f64) -> Self {
        if value > 2.5 {
            TDGSeverity::Critical
        } else if value > 1.5 {
            TDGSeverity::Warning
        } else {
            TDGSeverity::Normal
        }
    }
}

impl TDGSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            TDGSeverity::Normal => "normal",
            TDGSeverity::Warning => "warning",
            TDGSeverity::Critical => "critical",
        }
    }
}

/// Configuration for TDG calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGConfig {
    /// Weight for complexity component (default: 0.30)
    pub complexity_weight: f64,

    /// Weight for churn component (default: 0.35)
    pub churn_weight: f64,

    /// Weight for coupling component (default: 0.15)
    pub coupling_weight: f64,

    /// Weight for domain risk component (default: 0.10)
    pub domain_risk_weight: f64,

    /// Weight for duplication component (default: 0.10)
    pub duplication_weight: f64,

    /// Threshold for critical severity (default: 2.5)
    pub critical_threshold: f64,

    /// Threshold for warning severity (default: 1.5)
    pub warning_threshold: f64,
}

impl Default for TDGConfig {
    fn default() -> Self {
        Self {
            complexity_weight: 0.30,
            churn_weight: 0.35,
            coupling_weight: 0.15,
            domain_risk_weight: 0.10,
            duplication_weight: 0.10,
            critical_threshold: 2.5,
            warning_threshold: 1.5,
        }
    }
}

/// Summary statistics for TDG across a codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGSummary {
    /// Total number of files analyzed
    pub total_files: usize,

    /// Number of files with critical TDG scores
    pub critical_files: usize,

    /// Number of files with warning TDG scores
    pub warning_files: usize,

    /// Average TDG score across all files
    pub average_tdg: f64,

    /// 95th percentile TDG score
    pub p95_tdg: f64,

    /// 99th percentile TDG score
    pub p99_tdg: f64,

    /// Estimated technical debt in hours
    pub estimated_debt_hours: f64,

    /// Files with highest TDG scores
    pub hotspots: Vec<TDGHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGHotspot {
    /// File path
    pub path: String,

    /// TDG score
    pub tdg_score: f64,

    /// Primary contributor to high TDG
    pub primary_factor: String,

    /// Estimated hours to refactor
    pub estimated_hours: f64,
}

/// TDG calculation result with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGAnalysis {
    /// The calculated TDG score
    pub score: TDGScore,

    /// Detailed explanation of the calculation
    pub explanation: String,

    /// Specific recommendations for reducing TDG
    pub recommendations: Vec<TDGRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGRecommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,

    /// Specific action to take
    pub action: String,

    /// Expected TDG reduction
    pub expected_reduction: f64,

    /// Estimated effort in hours
    pub estimated_hours: f64,

    /// Priority level (1-5, 5 being highest)
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationType {
    /// Reduce function complexity
    ReduceComplexity,

    /// Stabilize frequently changing code
    StabilizeChurn,

    /// Reduce coupling between modules
    ReduceCoupling,

    /// Address domain-specific risks
    AddressDomainRisk,

    /// Remove duplicate code
    RemoveDuplication,

    /// Split large files
    SplitFile,

    /// Add test coverage
    AddTests,
}

/// TDG distribution for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGDistribution {
    /// Histogram buckets
    pub buckets: Vec<TDGBucket>,

    /// Total number of files
    pub total_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGBucket {
    /// Lower bound of the bucket (inclusive)
    pub min: f64,

    /// Upper bound of the bucket (exclusive)
    pub max: f64,

    /// Number of files in this bucket
    pub count: usize,

    /// Percentage of total files
    pub percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tdg_severity_from_value() {
        assert_eq!(TDGSeverity::from(0.5), TDGSeverity::Normal);
        assert_eq!(TDGSeverity::from(1.5), TDGSeverity::Normal);
        assert_eq!(TDGSeverity::from(1.6), TDGSeverity::Warning);
        assert_eq!(TDGSeverity::from(2.5), TDGSeverity::Warning);
        assert_eq!(TDGSeverity::from(2.6), TDGSeverity::Critical);
        assert_eq!(TDGSeverity::from(5.0), TDGSeverity::Critical);
    }

    #[test]
    fn test_tdg_config_default() {
        let config = TDGConfig::default();
        let total_weight = config.complexity_weight
            + config.churn_weight
            + config.coupling_weight
            + config.domain_risk_weight
            + config.duplication_weight;

        // Weights should sum to 1.0
        assert!((total_weight - 1.0).abs() < f64::EPSILON);
    }
}

// Additional types for SATD analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdItem {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub comment_text: String,
    pub debt_type: String,
    pub severity: SatdSeverity,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SatdSeverity {
    Low,
    Medium,
    High,
    Critical,
}
