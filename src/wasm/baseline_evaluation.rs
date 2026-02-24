// QualityBaseline and RollingStats implementations
// Included from baseline.rs — shares parent module scope

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
