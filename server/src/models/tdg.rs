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

#[cfg(test)]
mod new_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_tdg_score_creation() {
        let components = TDGComponents {
            complexity: 1.5,
            churn: 0.8,
            coupling: 0.3,
            domain_risk: 0.2,
            duplication: 0.4,
        };

        let score = TDGScore {
            value: 3.2,
            components,
            severity: TDGSeverity::Warning,
            percentile: 75.0,
            confidence: 0.9,
        };

        assert_eq!(score.value, 3.2);
        assert_eq!(score.components.complexity, 1.5);
        assert_eq!(score.severity, TDGSeverity::Warning);
        assert_eq!(score.percentile, 75.0);
        assert_eq!(score.confidence, 0.9);
    }

    #[test]
    fn test_tdg_severity_ordering() {
        // TDGSeverity doesn't implement Ord, just test equality
        assert_eq!(TDGSeverity::Normal, TDGSeverity::Normal);
        assert_eq!(TDGSeverity::Warning, TDGSeverity::Warning);
        assert_eq!(TDGSeverity::Critical, TDGSeverity::Critical);
        
        assert_eq!(TDGSeverity::Normal, TDGSeverity::Normal);
        assert_ne!(TDGSeverity::Normal, TDGSeverity::Critical);
    }

    #[test]
    fn test_tdg_components_equality() {
        let comp1 = TDGComponents {
            complexity: 1.0,
            churn: 2.0,
            coupling: 3.0,
            domain_risk: 4.0,
            duplication: 5.0,
        };

        let comp2 = TDGComponents {
            complexity: 1.0,
            churn: 2.0,
            coupling: 3.0,
            domain_risk: 4.0,
            duplication: 5.0,
        };

        assert_eq!(comp1, comp2);
    }

    #[test]
    fn test_tdg_summary_creation() {
        let summary = TDGSummary {
            total_files: 95,
            critical_files: 10,
            warning_files: 20,
            average_tdg: 2.5,
            p95_tdg: 4.5,
            p99_tdg: 4.9,
            estimated_debt_hours: 120.0,
            hotspots: vec![],
        };

        assert_eq!(summary.total_files, 95);
        assert_eq!(summary.critical_files, 10);
        assert_eq!(summary.warning_files, 20);
        assert_eq!(summary.average_tdg, 2.5);
        assert_eq!(summary.p95_tdg, 4.5);
        assert_eq!(summary.p99_tdg, 4.9);
        assert_eq!(summary.estimated_debt_hours, 120.0);
    }

    #[test]
    fn test_tdg_hotspot() {
        let hotspot = TDGHotspot {
            path: "src/complex.rs".to_string(),
            tdg_score: 4.5,
            primary_factor: "High complexity and churn".to_string(),
            estimated_hours: 8.0,
        };

        assert_eq!(hotspot.path, "src/complex.rs");
        assert_eq!(hotspot.tdg_score, 4.5);
        assert_eq!(hotspot.primary_factor, "High complexity and churn");
        assert_eq!(hotspot.estimated_hours, 8.0);
    }

    #[test]
    fn test_tdg_analysis() {
        let analysis = TDGAnalysis {
            score: TDGScore {
                value: 2.5,
                components: TDGComponents {
                    complexity: 0.8,
                    churn: 0.5,
                    coupling: 0.4,
                    domain_risk: 0.3,
                    duplication: 0.5,
                },
                severity: TDGSeverity::Warning,
                percentile: 75.0,
                confidence: 0.95,
            },
            explanation: "Test explanation".to_string(),
            recommendations: vec![],
        };

        assert_eq!(analysis.score.value, 2.5);
        assert_eq!(analysis.score.percentile, 75.0);
        assert!(analysis.recommendations.is_empty());
    }

    #[test]
    fn test_recommendation_type() {
        assert_eq!(RecommendationType::ReduceComplexity, RecommendationType::ReduceComplexity);
        assert_ne!(RecommendationType::ReduceComplexity, RecommendationType::StabilizeChurn);
        
        let rec = TDGRecommendation {
            recommendation_type: RecommendationType::ReduceComplexity,
            action: "Refactor complex function into smaller units".to_string(),
            expected_reduction: 0.5,
            estimated_hours: 4.0,
            priority: 3,
        };
        
        assert_eq!(rec.priority, 3);
        assert_eq!(rec.action, "Refactor complex function into smaller units");
        assert_eq!(rec.expected_reduction, 0.5);
    }

    #[test]
    fn test_tdg_distribution() {
        let dist = TDGDistribution {
            buckets: vec![
                TDGBucket {
                    min: 0.0,
                    max: 1.5,
                    count: 50,
                    percentage: 50.0,
                },
                TDGBucket {
                    min: 1.5,
                    max: 3.0,
                    count: 30,
                    percentage: 30.0,
                },
                TDGBucket {
                    min: 3.0,
                    max: 5.0,
                    count: 20,
                    percentage: 20.0,
                },
            ],
            total_files: 100,
        };
        
        assert_eq!(dist.buckets.len(), 3);
        assert_eq!(dist.buckets[0].count, 50);
        assert_eq!(dist.buckets[0].percentage, 50.0);
        assert_eq!(dist.total_files, 100);
    }

    #[test]
    fn test_tdg_config() {
        let config = TDGConfig {
            complexity_weight: 0.30,
            churn_weight: 0.35,
            coupling_weight: 0.15,
            domain_risk_weight: 0.10,
            duplication_weight: 0.10,
            normal_threshold: 1.5,
            warning_threshold: 2.5,
            percentile_window: 100,
        };
        
        assert_eq!(config.complexity_weight, 0.30);
        assert_eq!(config.churn_weight, 0.35);
        assert_eq!(config.normal_threshold, 1.5);
        assert_eq!(config.warning_threshold, 2.5);
    }

    #[test]
    fn test_satd_item_creation() {
        let item = SatdItem {
            file_path: PathBuf::from("lib.rs"),
            line_number: 123,
            comment_text: "TODO: Fix this hack".to_string(),
            debt_type: "TODO".to_string(),
            severity: SatdSeverity::Medium,
            confidence: 0.95,
        };

        assert_eq!(item.file_path, PathBuf::from("lib.rs"));
        assert_eq!(item.line_number, 123);
        assert_eq!(item.comment_text, "TODO: Fix this hack");
        assert_eq!(item.debt_type, "TODO");
        assert_eq!(item.severity, SatdSeverity::Medium);
        assert_eq!(item.confidence, 0.95);
    }

    #[test]
    fn test_satd_severity_ordering() {
        assert!(SatdSeverity::Low < SatdSeverity::Medium);
        assert!(SatdSeverity::Medium < SatdSeverity::High);
        assert!(SatdSeverity::High < SatdSeverity::Critical);
        
        let mut severities = vec![
            SatdSeverity::High,
            SatdSeverity::Low,
            SatdSeverity::Critical,
            SatdSeverity::Medium,
        ];
        
        severities.sort();
        
        assert_eq!(severities, vec![
            SatdSeverity::Low,
            SatdSeverity::Medium,
            SatdSeverity::High,
            SatdSeverity::Critical,
        ]);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = TDGScore {
            value: 3.14,
            components: TDGComponents {
                complexity: 1.1,
                churn: 2.2,
                coupling: 3.3,
                domain_risk: 4.4,
                duplication: 5.5,
            },
            severity: TDGSeverity::Critical,
            percentile: 90.0,
            confidence: 0.99,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TDGScore = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(original.value, deserialized.value);
        assert_eq!(original.components, deserialized.components);
        assert_eq!(original.severity, deserialized.severity);
    }

}
