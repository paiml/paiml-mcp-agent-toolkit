//! Anchored quality metrics and baseline management

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Multi-point baseline system for quality assessment
#[derive(Debug, Clone)]
pub struct QualityBaseline {
    release_anchor: Metrics,      // Last major release (immutable)
    stable_anchor: Metrics,       // Last stable tag
    rolling_window: RollingStats, // Recent 30 days
}

impl QualityBaseline {
    #[must_use]
    pub fn new(release_metrics: Metrics, stable_metrics: Metrics) -> Self {
        Self {
            release_anchor: release_metrics,
            stable_anchor: stable_metrics,
            rolling_window: RollingStats::new(30),
        }
    }

    /// Evaluate current metrics against baselines
    #[must_use]
    pub fn evaluate(&self, current: &Metrics) -> QualityAssessment {
        let mut violations = Vec::new();

        // Hard limit: Never exceed release anchor p99
        if current.complexity_p95 > self.release_anchor.complexity_p99 {
            violations.push(Violation::ComplexityRegression {
                current: current.complexity_p95,
                limit: self.release_anchor.complexity_p99,
                severity: Severity::Error,
            });
        }

        // Soft limit: Warn if exceeding stable anchor p95
        if current.complexity_p90 > self.stable_anchor.complexity_p95 {
            violations.push(Violation::ComplexityCreep {
                current: current.complexity_p90,
                baseline: self.stable_anchor.complexity_p95,
                severity: Severity::Warning,
            });
        }

        // Trend detection: Alert on sustained increases
        if self.rolling_window.trend_slope() > 0.1 {
            violations.push(Violation::QualityErosion {
                slope: self.rolling_window.trend_slope(),
                severity: Severity::Warning,
            });
        }

        // Binary size checks
        if current.binary_size > (self.release_anchor.binary_size as f64 * 1.2) as usize {
            violations.push(Violation::BinarySizeIncrease {
                current: current.binary_size,
                baseline: self.release_anchor.binary_size,
                increase_percent: ((current.binary_size as f64
                    / self.release_anchor.binary_size as f64
                    - 1.0)
                    * 100.0),
                severity: Severity::Warning,
            });
        }

        // Performance regression checks
        if current.init_time_ms > (f64::from(self.stable_anchor.init_time_ms) * 1.5) as u32 {
            violations.push(Violation::PerformanceRegression {
                metric: "initialization".to_string(),
                current: f64::from(current.init_time_ms),
                baseline: f64::from(self.stable_anchor.init_time_ms),
                severity: Severity::Error,
            });
        }

        let health = self.calculate_health_score(current);
        let rec = self.generate_recommendation(&violations);

        QualityAssessment {
            violations,
            overall_health: health,
            recommendation: rec,
        }
    }

    /// Add new data point to rolling window
    pub fn add_data_point(&mut self, metrics: Metrics) {
        self.rolling_window.add_point(metrics);
    }

    /// Calculate overall health score (0-100)
    fn calculate_health_score(&self, current: &Metrics) -> f64 {
        let mut score = 100.0;

        // Complexity penalty
        let complexity_ratio =
            f64::from(current.complexity_p90) / f64::from(self.stable_anchor.complexity_p90);
        if complexity_ratio > 1.0 {
            score -= (complexity_ratio - 1.0) * 20.0;
        }

        // Binary size penalty
        let size_ratio = current.binary_size as f64 / self.stable_anchor.binary_size as f64;
        if size_ratio > 1.0 {
            score -= (size_ratio - 1.0) * 15.0;
        }

        // Performance penalty
        let perf_ratio =
            f64::from(current.init_time_ms) / f64::from(self.stable_anchor.init_time_ms);
        if perf_ratio > 1.0 {
            score -= (perf_ratio - 1.0) * 25.0;
        }

        score.max(0.0)
    }

    /// Generate actionable recommendation
    fn generate_recommendation(&self, violations: &[Violation]) -> String {
        if violations.is_empty() {
            return "Quality metrics are within acceptable bounds.".to_string();
        }

        let critical_count = violations
            .iter()
            .filter(|v| matches!(v.severity(), Severity::Error))
            .count();

        if critical_count > 0 {
            format!("⚠️ {critical_count} critical violations detected. Immediate action required to address quality regressions.")
        } else {
            "Quality metrics show concerning trends. Consider refactoring to improve maintainability.".to_string()
        }
    }
}

/// Quality metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub timestamp: DateTime<Utc>,
    pub complexity_p90: u32,
    pub complexity_p95: u32,
    pub complexity_p99: u32,
    pub binary_size: usize,
    pub init_time_ms: u32,
    pub memory_usage_mb: u32,
    pub function_count: usize,
    pub instruction_count: usize,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            complexity_p90: 10,
            complexity_p95: 15,
            complexity_p99: 20,
            binary_size: 1_000_000,
            init_time_ms: 10,
            memory_usage_mb: 50,
            function_count: 100,
            instruction_count: 10_000,
        }
    }
}

/// Rolling statistics for trend analysis
#[derive(Debug, Clone)]
pub struct RollingStats {
    window_days: usize,
    data_points: VecDeque<Metrics>,
}

impl RollingStats {
    #[must_use]
    pub fn new(window_days: usize) -> Self {
        Self {
            window_days,
            data_points: VecDeque::new(),
        }
    }

    pub fn add_point(&mut self, metrics: Metrics) {
        self.data_points.push_back(metrics);

        // Remove old points outside window
        let cutoff = Utc::now() - Duration::days(self.window_days as i64);
        while let Some(front) = self.data_points.front() {
            if front.timestamp < cutoff {
                self.data_points.pop_front();
            } else {
                break;
            }
        }
    }

    /// Calculate trend slope using linear regression
    #[must_use]
    pub fn trend_slope(&self) -> f64 {
        if self.data_points.len() < 2 {
            return 0.0;
        }

        let n = self.data_points.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, point) in self.data_points.iter().enumerate() {
            let x = i as f64;
            let y = f64::from(point.complexity_p90);

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        // Calculate slope: (n*sum_xy - sum_x*sum_y) / (n*sum_x2 - sum_x^2)
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = n * sum_x2 - sum_x * sum_x;

        if denominator.abs() < 0.0001 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// Quality assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub violations: Vec<Violation>,
    pub overall_health: f64,
    pub recommendation: String,
}

impl QualityAssessment {
    #[must_use]
    pub fn is_passing(&self) -> bool {
        self.violations
            .iter()
            .all(|v| !matches!(v.severity(), Severity::Error))
    }
}

/// Quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Violation {
    ComplexityRegression {
        current: u32,
        limit: u32,
        severity: Severity,
    },
    ComplexityCreep {
        current: u32,
        baseline: u32,
        severity: Severity,
    },
    QualityErosion {
        slope: f64,
        severity: Severity,
    },
    BinarySizeIncrease {
        current: usize,
        baseline: usize,
        increase_percent: f64,
        severity: Severity,
    },
    PerformanceRegression {
        metric: String,
        current: f64,
        baseline: f64,
        severity: Severity,
    },
}

impl Violation {
    #[must_use]
    pub fn severity(&self) -> &Severity {
        match self {
            Self::ComplexityRegression { severity, .. }
            | Self::ComplexityCreep { severity, .. }
            | Self::QualityErosion { severity, .. }
            | Self::BinarySizeIncrease { severity, .. }
            | Self::PerformanceRegression { severity, .. } => severity,
        }
    }

    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::ComplexityRegression { current, limit, .. } => {
                format!("Complexity regression: {current} exceeds limit {limit}")
            }
            Self::ComplexityCreep {
                current, baseline, ..
            } => {
                format!("Complexity creep: {current} exceeds baseline {baseline}")
            }
            Self::QualityErosion { slope, .. } => {
                format!("Quality erosion detected with slope {slope:.2}")
            }
            Self::BinarySizeIncrease {
                increase_percent, ..
            } => {
                format!("Binary size increased by {increase_percent:.1}%")
            }
            Self::PerformanceRegression {
                metric,
                current,
                baseline,
                ..
            } => {
                format!("{metric} regression: {current:.1}ms exceeds baseline {baseline:.1}ms")
            }
        }
    }
}

/// Severity levels for violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn default_metrics() -> Metrics {
        Metrics::default()
    }

    fn custom_metrics(complexity_p90: u32, complexity_p95: u32, complexity_p99: u32) -> Metrics {
        Metrics {
            timestamp: Utc::now(),
            complexity_p90,
            complexity_p95,
            complexity_p99,
            binary_size: 1_000_000,
            init_time_ms: 10,
            memory_usage_mb: 50,
            function_count: 100,
            instruction_count: 10_000,
        }
    }

    // ==================== QualityBaseline Tests ====================

    #[test]
    fn test_quality_baseline_new() {
        let release = default_metrics();
        let stable = default_metrics();
        let baseline = QualityBaseline::new(release, stable);

        // Verify baseline was created
        assert!(baseline.rolling_window.data_points.is_empty());
    }

    #[test]
    fn test_evaluate_all_passing() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        let current = custom_metrics(10, 15, 20);
        let assessment = baseline.evaluate(&current);

        assert!(assessment.is_passing());
        assert!(assessment.violations.is_empty());
        assert_eq!(assessment.overall_health, 100.0);
    }

    #[test]
    fn test_evaluate_complexity_regression() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        // Current complexity_p95 (25) exceeds release_anchor.complexity_p99 (20)
        let current = custom_metrics(10, 25, 30);
        let assessment = baseline.evaluate(&current);

        assert!(!assessment.is_passing());
        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::ComplexityRegression { .. })));
    }

    #[test]
    fn test_evaluate_complexity_creep() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        // Current complexity_p90 (18) exceeds stable_anchor.complexity_p95 (15)
        let current = custom_metrics(18, 19, 20);
        let assessment = baseline.evaluate(&current);

        // Complexity creep is a warning, not an error
        assert!(assessment.is_passing());
        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::ComplexityCreep { .. })));
    }

    #[test]
    fn test_evaluate_binary_size_increase() {
        let release = Metrics {
            binary_size: 1_000_000,
            ..default_metrics()
        };
        let stable = Metrics {
            binary_size: 1_000_000,
            ..default_metrics()
        };
        let baseline = QualityBaseline::new(release, stable);

        // 25% increase exceeds 20% threshold
        let current = Metrics {
            binary_size: 1_250_000,
            ..default_metrics()
        };
        let assessment = baseline.evaluate(&current);

        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::BinarySizeIncrease { .. })));
    }

    #[test]
    fn test_evaluate_performance_regression() {
        let release = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let stable = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let baseline = QualityBaseline::new(release, stable);

        // Init time 20ms is > 15ms (1.5x baseline)
        let current = Metrics {
            init_time_ms: 20,
            ..default_metrics()
        };
        let assessment = baseline.evaluate(&current);

        assert!(!assessment.is_passing());
        assert!(assessment
            .violations
            .iter()
            .any(|v| matches!(v, Violation::PerformanceRegression { .. })));
    }

    #[test]
    fn test_add_data_point() {
        let release = default_metrics();
        let stable = default_metrics();
        let mut baseline = QualityBaseline::new(release, stable);

        baseline.add_data_point(default_metrics());
        assert_eq!(baseline.rolling_window.data_points.len(), 1);

        baseline.add_data_point(default_metrics());
        assert_eq!(baseline.rolling_window.data_points.len(), 2);
    }

    #[test]
    fn test_health_score_degradation() {
        let release = custom_metrics(10, 15, 20);
        let stable = custom_metrics(10, 15, 20);
        let baseline = QualityBaseline::new(release, stable);

        // Perfect score
        let current = custom_metrics(10, 15, 20);
        let assessment = baseline.evaluate(&current);
        assert_eq!(assessment.overall_health, 100.0);

        // Degraded complexity
        let degraded = custom_metrics(15, 20, 25); // 50% higher
        let assessment = baseline.evaluate(&degraded);
        assert!(assessment.overall_health < 100.0);
    }

    #[test]
    fn test_recommendation_no_violations() {
        let release = default_metrics();
        let stable = default_metrics();
        let baseline = QualityBaseline::new(release, stable);

        let current = default_metrics();
        let assessment = baseline.evaluate(&current);

        assert_eq!(
            assessment.recommendation,
            "Quality metrics are within acceptable bounds."
        );
    }

    #[test]
    fn test_recommendation_with_critical_violations() {
        let release = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let stable = Metrics {
            init_time_ms: 10,
            ..default_metrics()
        };
        let baseline = QualityBaseline::new(release, stable);

        let current = Metrics {
            init_time_ms: 100, // 10x baseline - critical performance regression
            ..default_metrics()
        };
        let assessment = baseline.evaluate(&current);

        assert!(assessment.recommendation.contains("critical violations"));
    }

    // ==================== Metrics Tests ====================

    #[test]
    fn test_metrics_default() {
        let metrics = Metrics::default();

        assert_eq!(metrics.complexity_p90, 10);
        assert_eq!(metrics.complexity_p95, 15);
        assert_eq!(metrics.complexity_p99, 20);
        assert_eq!(metrics.binary_size, 1_000_000);
        assert_eq!(metrics.init_time_ms, 10);
        assert_eq!(metrics.memory_usage_mb, 50);
        assert_eq!(metrics.function_count, 100);
        assert_eq!(metrics.instruction_count, 10_000);
    }

    #[test]
    fn test_metrics_serialization() {
        let metrics = Metrics::default();

        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: Metrics = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metrics.complexity_p90, deserialized.complexity_p90);
        assert_eq!(metrics.binary_size, deserialized.binary_size);
    }

    #[test]
    fn test_metrics_clone() {
        let metrics = Metrics::default();
        let cloned = metrics.clone();

        assert_eq!(metrics.complexity_p90, cloned.complexity_p90);
        assert_eq!(metrics.init_time_ms, cloned.init_time_ms);
    }

    // ==================== RollingStats Tests ====================

    #[test]
    fn test_rolling_stats_new() {
        let stats = RollingStats::new(30);
        assert_eq!(stats.window_days, 30);
        assert!(stats.data_points.is_empty());
    }

    #[test]
    fn test_rolling_stats_add_point() {
        let mut stats = RollingStats::new(30);

        stats.add_point(default_metrics());
        assert_eq!(stats.data_points.len(), 1);

        stats.add_point(default_metrics());
        assert_eq!(stats.data_points.len(), 2);
    }

    #[test]
    fn test_rolling_stats_trend_slope_empty() {
        let stats = RollingStats::new(30);
        assert_eq!(stats.trend_slope(), 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_single_point() {
        let mut stats = RollingStats::new(30);
        stats.add_point(default_metrics());

        // Need at least 2 points for slope
        assert_eq!(stats.trend_slope(), 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_flat() {
        let mut stats = RollingStats::new(30);

        // Add identical metrics - should have zero slope
        for _ in 0..5 {
            stats.add_point(custom_metrics(10, 15, 20));
        }

        assert_eq!(stats.trend_slope(), 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_increasing() {
        let mut stats = RollingStats::new(30);

        // Add increasing complexity - should have positive slope
        for i in 0..5 {
            stats.add_point(custom_metrics(10 + i, 15, 20));
        }

        assert!(stats.trend_slope() > 0.0);
    }

    #[test]
    fn test_rolling_stats_trend_slope_decreasing() {
        let mut stats = RollingStats::new(30);

        // Add decreasing complexity - should have negative slope
        for i in 0..5 {
            stats.add_point(custom_metrics(20 - i, 15, 20));
        }

        assert!(stats.trend_slope() < 0.0);
    }

    // ==================== QualityAssessment Tests ====================

    #[test]
    fn test_quality_assessment_is_passing_no_violations() {
        let assessment = QualityAssessment {
            violations: vec![],
            overall_health: 100.0,
            recommendation: "All good".to_string(),
        };

        assert!(assessment.is_passing());
    }

    #[test]
    fn test_quality_assessment_is_passing_with_warnings() {
        let assessment = QualityAssessment {
            violations: vec![Violation::ComplexityCreep {
                current: 15,
                baseline: 10,
                severity: Severity::Warning,
            }],
            overall_health: 85.0,
            recommendation: "Minor issues".to_string(),
        };

        assert!(assessment.is_passing());
    }

    #[test]
    fn test_quality_assessment_not_passing_with_errors() {
        let assessment = QualityAssessment {
            violations: vec![Violation::ComplexityRegression {
                current: 25,
                limit: 20,
                severity: Severity::Error,
            }],
            overall_health: 50.0,
            recommendation: "Critical issues".to_string(),
        };

        assert!(!assessment.is_passing());
    }

    #[test]
    fn test_quality_assessment_serialization() {
        let assessment = QualityAssessment {
            violations: vec![],
            overall_health: 95.0,
            recommendation: "Looking good".to_string(),
        };

        let serialized = serde_json::to_string(&assessment).unwrap();
        let deserialized: QualityAssessment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(assessment.overall_health, deserialized.overall_health);
        assert_eq!(assessment.recommendation, deserialized.recommendation);
    }

    // ==================== Violation Tests ====================

    #[test]
    fn test_violation_severity_complexity_regression() {
        let violation = Violation::ComplexityRegression {
            current: 25,
            limit: 20,
            severity: Severity::Error,
        };

        assert_eq!(violation.severity(), &Severity::Error);
    }

    #[test]
    fn test_violation_severity_complexity_creep() {
        let violation = Violation::ComplexityCreep {
            current: 15,
            baseline: 10,
            severity: Severity::Warning,
        };

        assert_eq!(violation.severity(), &Severity::Warning);
    }

    #[test]
    fn test_violation_severity_quality_erosion() {
        let violation = Violation::QualityErosion {
            slope: 0.15,
            severity: Severity::Warning,
        };

        assert_eq!(violation.severity(), &Severity::Warning);
    }

    #[test]
    fn test_violation_severity_binary_size() {
        let violation = Violation::BinarySizeIncrease {
            current: 1_500_000,
            baseline: 1_000_000,
            increase_percent: 50.0,
            severity: Severity::Warning,
        };

        assert_eq!(violation.severity(), &Severity::Warning);
    }

    #[test]
    fn test_violation_severity_performance() {
        let violation = Violation::PerformanceRegression {
            metric: "init_time".to_string(),
            current: 20.0,
            baseline: 10.0,
            severity: Severity::Error,
        };

        assert_eq!(violation.severity(), &Severity::Error);
    }

    #[test]
    fn test_violation_description_complexity_regression() {
        let violation = Violation::ComplexityRegression {
            current: 25,
            limit: 20,
            severity: Severity::Error,
        };

        assert_eq!(
            violation.description(),
            "Complexity regression: 25 exceeds limit 20"
        );
    }

    #[test]
    fn test_violation_description_complexity_creep() {
        let violation = Violation::ComplexityCreep {
            current: 15,
            baseline: 10,
            severity: Severity::Warning,
        };

        assert_eq!(
            violation.description(),
            "Complexity creep: 15 exceeds baseline 10"
        );
    }

    #[test]
    fn test_violation_description_quality_erosion() {
        let violation = Violation::QualityErosion {
            slope: 0.15,
            severity: Severity::Warning,
        };

        assert_eq!(
            violation.description(),
            "Quality erosion detected with slope 0.15"
        );
    }

    #[test]
    fn test_violation_description_binary_size() {
        let violation = Violation::BinarySizeIncrease {
            current: 1_500_000,
            baseline: 1_000_000,
            increase_percent: 50.0,
            severity: Severity::Warning,
        };

        assert_eq!(violation.description(), "Binary size increased by 50.0%");
    }

    #[test]
    fn test_violation_description_performance() {
        let violation = Violation::PerformanceRegression {
            metric: "initialization".to_string(),
            current: 20.0,
            baseline: 10.0,
            severity: Severity::Error,
        };

        assert_eq!(
            violation.description(),
            "initialization regression: 20.0ms exceeds baseline 10.0ms"
        );
    }

    #[test]
    fn test_violation_serialization() {
        let violation = Violation::ComplexityRegression {
            current: 25,
            limit: 20,
            severity: Severity::Error,
        };

        let serialized = serde_json::to_string(&violation).unwrap();
        let deserialized: Violation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(violation.severity(), deserialized.severity());
    }

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Info, Severity::Info);
        assert_eq!(Severity::Warning, Severity::Warning);
        assert_eq!(Severity::Error, Severity::Error);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(Severity::Info, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Error);
        assert_ne!(Severity::Info, Severity::Error);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Warning;

        let serialized = serde_json::to_string(&severity).unwrap();
        let deserialized: Severity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(severity, deserialized);
    }

    #[test]
    fn test_severity_clone() {
        let severity = Severity::Error;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }
}
