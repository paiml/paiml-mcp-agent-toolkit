//! Anchored quality metrics and baseline management

use std::collections::VecDeque;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

/// Multi-point baseline system for quality assessment
#[derive(Debug, Clone)]
pub struct QualityBaseline {
    release_anchor: Metrics,      // Last major release (immutable)
    stable_anchor: Metrics,        // Last stable tag
    rolling_window: RollingStats, // Recent 30 days
}

impl QualityBaseline {
    pub fn new(release_metrics: Metrics, stable_metrics: Metrics) -> Self {
        Self {
            release_anchor: release_metrics,
            stable_anchor: stable_metrics,
            rolling_window: RollingStats::new(30),
        }
    }

    /// Evaluate current metrics against baselines
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
                increase_percent: ((current.binary_size as f64 / self.release_anchor.binary_size as f64 - 1.0) * 100.0),
                severity: Severity::Warning,
            });
        }
        
        // Performance regression checks
        if current.init_time_ms > (self.stable_anchor.init_time_ms as f64 * 1.5) as u32 {
            violations.push(Violation::PerformanceRegression {
                metric: "initialization".to_string(),
                current: current.init_time_ms as f64,
                baseline: self.stable_anchor.init_time_ms as f64,
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
        let complexity_ratio = current.complexity_p90 as f64 / self.stable_anchor.complexity_p90 as f64;
        if complexity_ratio > 1.0 {
            score -= (complexity_ratio - 1.0) * 20.0;
        }
        
        // Binary size penalty
        let size_ratio = current.binary_size as f64 / self.stable_anchor.binary_size as f64;
        if size_ratio > 1.0 {
            score -= (size_ratio - 1.0) * 15.0;
        }
        
        // Performance penalty
        let perf_ratio = current.init_time_ms as f64 / self.stable_anchor.init_time_ms as f64;
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
        
        let critical_count = violations.iter()
            .filter(|v| matches!(v.severity(), Severity::Error))
            .count();
        
        if critical_count > 0 {
            format!("⚠️ {} critical violations detected. Immediate action required to address quality regressions.", critical_count)
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
            let y = point.complexity_p90 as f64;
            
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
    pub fn is_passing(&self) -> bool {
        self.violations.iter()
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
    pub fn severity(&self) -> &Severity {
        match self {
            Self::ComplexityRegression { severity, .. } |
            Self::ComplexityCreep { severity, .. } |
            Self::QualityErosion { severity, .. } |
            Self::BinarySizeIncrease { severity, .. } |
            Self::PerformanceRegression { severity, .. } => severity,
        }
    }
    
    pub fn description(&self) -> String {
        match self {
            Self::ComplexityRegression { current, limit, .. } => {
                format!("Complexity regression: {} exceeds limit {}", current, limit)
            }
            Self::ComplexityCreep { current, baseline, .. } => {
                format!("Complexity creep: {} exceeds baseline {}", current, baseline)
            }
            Self::QualityErosion { slope, .. } => {
                format!("Quality erosion detected with slope {:.2}", slope)
            }
            Self::BinarySizeIncrease { increase_percent, .. } => {
                format!("Binary size increased by {:.1}%", increase_percent)
            }
            Self::PerformanceRegression { metric, current, baseline, .. } => {
                format!("{} regression: {:.1}ms exceeds baseline {:.1}ms", metric, current, baseline)
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