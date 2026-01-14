// Quality gate enforcement for Claude integration
// Stricter than standard PMAT gates due to cross-language complexity

/// Quality gate specifically for Claude integration code
#[derive(Debug, Clone)]
pub struct ClaudeIntegrationQualityGate {
    pub max_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub min_test_coverage: f64,
    pub max_coupling: usize,
}

impl Default for ClaudeIntegrationQualityGate {
    fn default() -> Self {
        Self {
            max_complexity: 15,           // Stricter than PMAT's 20
            max_cognitive_complexity: 10, // Bridge code must be simple
            min_test_coverage: 0.95,      // 95% coverage requirement
            max_coupling: 3,              // Maximum 3 dependencies
        }
    }
}

#[derive(Debug)]
pub enum QualityResult {
    Pass,
    Failure(String),
}

impl ClaudeIntegrationQualityGate {
    pub fn check(&self, metrics: &QualityMetrics) -> QualityResult {
        // Zero tolerance for SATD in integration layer
        if metrics.satd_count > 0 {
            return QualityResult::Failure(format!(
                "SATD detected in integration layer: {}. Zero-tolerance policy violated.",
                metrics.satd_count
            ));
        }

        // Complexity must be below threshold
        if metrics.cyclomatic_complexity > self.max_complexity {
            return QualityResult::Failure(format!(
                "Cyclomatic complexity {} exceeds maximum {}",
                metrics.cyclomatic_complexity, self.max_complexity
            ));
        }

        // Enforce cognitive complexity for maintainability
        if metrics.cognitive_complexity > self.max_cognitive_complexity {
            return QualityResult::Failure(format!(
                "Cognitive complexity {} exceeds maximum {}",
                metrics.cognitive_complexity, self.max_cognitive_complexity
            ));
        }

        // Test coverage requirement
        if metrics.test_coverage < self.min_test_coverage {
            return QualityResult::Failure(format!(
                "Test coverage {:.2}% below minimum {:.2}%",
                metrics.test_coverage * 100.0,
                self.min_test_coverage * 100.0
            ));
        }

        QualityResult::Pass
    }
}

#[derive(Debug, Default)]
pub struct QualityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub satd_count: usize,
    pub test_coverage: f64,
    pub coupling_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gate_pass() {
        let gate = ClaudeIntegrationQualityGate::default();
        let metrics = QualityMetrics {
            cyclomatic_complexity: 10,
            cognitive_complexity: 8,
            satd_count: 0,
            test_coverage: 0.96,
            coupling_count: 2,
        };

        let result = gate.check(&metrics);
        assert!(matches!(result, QualityResult::Pass));
    }

    #[test]
    fn test_quality_gate_satd_failure() {
        let gate = ClaudeIntegrationQualityGate::default();
        let metrics = QualityMetrics {
            cyclomatic_complexity: 10,
            cognitive_complexity: 8,
            satd_count: 1,
            test_coverage: 0.96,
            coupling_count: 2,
        };

        let result = gate.check(&metrics);
        assert!(matches!(result, QualityResult::Failure(_)));
    }

    #[test]
    fn test_quality_gate_complexity_failure() {
        let gate = ClaudeIntegrationQualityGate::default();
        let metrics = QualityMetrics {
            cyclomatic_complexity: 20,
            cognitive_complexity: 8,
            satd_count: 0,
            test_coverage: 0.96,
            coupling_count: 2,
        };

        let result = gate.check(&metrics);
        assert!(matches!(result, QualityResult::Failure(_)));
    }

    #[test]
    fn test_quality_gate_coverage_failure() {
        let gate = ClaudeIntegrationQualityGate::default();
        let metrics = QualityMetrics {
            cyclomatic_complexity: 10,
            cognitive_complexity: 8,
            satd_count: 0,
            test_coverage: 0.90,
            coupling_count: 2,
        };

        let result = gate.check(&metrics);
        assert!(matches!(result, QualityResult::Failure(_)));
    }
}
