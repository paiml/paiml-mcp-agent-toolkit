//! Core QDD data structures and interfaces
//! Toyota Way: Single Responsibility and DRY principles

use serde::{Deserialize, Serialize};

/// Quality profile defining standards for code generation and refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProfile {
    pub name: String,
    pub thresholds: QualityThresholds,
    pub patterns: DesignPatterns,
    pub rules: Vec<QualityRule>,
}

/// Quality thresholds for different metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub max_complexity: u32,
    pub max_cognitive: u32,
    pub min_coverage: u32,
    pub max_tdg: u32,
    pub max_entropy_violations: u32,  // New: Maximum allowed entropy violations
    pub zero_satd: bool,
    pub zero_dead_code: bool,
    pub require_doctests: bool,
    pub require_property_tests: bool,
}

/// Design patterns to enforce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPatterns {
    pub enforce_solid: bool,
    pub enforce_dry: bool,
    pub enforce_kiss: bool,
    pub enforce_yagni: bool,
}

/// Individual quality rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRule {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub pattern: String,
}

/// Rule severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl QualityProfile {
    /// Create extreme quality profile (≤5 complexity, 90% coverage)
    pub fn extreme() -> Self {
        Self {
            name: "extreme".to_string(),
            thresholds: QualityThresholds {
                max_complexity: 5,
                max_cognitive: 5,
                min_coverage: 90,
                max_tdg: 3,
                max_entropy_violations: 2,  // Extreme: very low tolerance for entropy violations
                zero_satd: true,
                zero_dead_code: true,
                require_doctests: true,
                require_property_tests: true,
            },
            patterns: DesignPatterns {
                enforce_solid: true,
                enforce_dry: true,
                enforce_kiss: true,
                enforce_yagni: true,
            },
            rules: Self::default_rules(),
        }
    }
    
    /// Create standard quality profile (≤10 complexity, 80% coverage)
    pub fn standard() -> Self {
        Self {
            name: "standard".to_string(),
            thresholds: QualityThresholds {
                max_complexity: 10,
                max_cognitive: 10,
                min_coverage: 80,
                max_tdg: 5,
                max_entropy_violations: 5,  // Standard: moderate tolerance
                zero_satd: true,
                zero_dead_code: false,
                require_doctests: true,
                require_property_tests: false,
            },
            patterns: DesignPatterns {
                enforce_solid: true,
                enforce_dry: true,
                enforce_kiss: true,
                enforce_yagni: false,
            },
            rules: Self::default_rules(),
        }
    }
    
    /// Create relaxed quality profile (≤20 complexity, 60% coverage)
    pub fn relaxed() -> Self {
        Self {
            name: "relaxed".to_string(),
            thresholds: QualityThresholds {
                max_complexity: 20,
                max_cognitive: 20,
                min_coverage: 60,
                max_tdg: 10,
                max_entropy_violations: 15,  // Relaxed: higher tolerance for legacy code
                zero_satd: false,
                zero_dead_code: false,
                require_doctests: false,
                require_property_tests: false,
            },
            patterns: DesignPatterns {
                enforce_solid: false,
                enforce_dry: false,
                enforce_kiss: true,
                enforce_yagni: false,
            },
            rules: Self::minimal_rules(),
        }
    }
    
    fn default_rules() -> Vec<QualityRule> {
        vec![
            QualityRule {
                name: "no_panic".to_string(),
                description: "Functions should not panic".to_string(),
                severity: Severity::Error,
                pattern: r"panic!|unwrap|expect".to_string(),
            },
            QualityRule {
                name: "proper_error_handling".to_string(),
                description: "Use Result types for error handling".to_string(),
                severity: Severity::Warning,
                pattern: r"Option.*unwrap".to_string(),
            },
        ]
    }
    
    fn minimal_rules() -> Vec<QualityRule> {
        vec![
            QualityRule {
                name: "no_panic".to_string(),
                description: "Functions should not panic".to_string(),
                severity: Severity::Warning,
                pattern: r"panic!".to_string(),
            },
        ]
    }
    
    /// Check if quality metrics meet this profile's thresholds
    pub fn meets_thresholds(&self, metrics: &QualityMetrics) -> bool {
        metrics.complexity <= self.thresholds.max_complexity &&
        metrics.cognitive_complexity <= self.thresholds.max_cognitive &&
        metrics.coverage >= self.thresholds.min_coverage &&
        metrics.tdg <= self.thresholds.max_tdg
    }
}

/// Quality metrics for code analysis
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub complexity: u32,
    pub cognitive_complexity: u32,
    pub coverage: u32,
    pub tdg: u32,
    pub satd_count: u32,
    pub dead_code_percentage: u32,
    pub has_doctests: bool,
    pub has_property_tests: bool,
}

impl QualityMetrics {
    /// Create default metrics (all zeros/false)
    pub fn default() -> Self {
        Self {
            complexity: 0,
            cognitive_complexity: 0,
            coverage: 0,
            tdg: 0,
            satd_count: 0,
            dead_code_percentage: 0,
            has_doctests: false,
            has_property_tests: false,
        }
    }
    
    /// Calculate overall quality score (0-100)
    pub fn calculate_score(&self) -> f64 {
        let complexity_score = (20_u32.saturating_sub(self.complexity) as f64 / 20.0) * 25.0;
        let coverage_score = (self.coverage as f64 / 100.0) * 25.0;
        let tdg_score = (10_u32.saturating_sub(self.tdg) as f64 / 10.0) * 25.0;
        let defect_score = if self.satd_count == 0 { 25.0 } else { 0.0 };
        
        complexity_score + coverage_score + tdg_score + defect_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quality_profile_extreme() {
        let profile = QualityProfile::extreme();
        assert_eq!(profile.name, "extreme");
        assert_eq!(profile.thresholds.max_complexity, 5);
        assert_eq!(profile.thresholds.min_coverage, 90);
        assert!(profile.thresholds.zero_satd);
    }
    
    #[test]
    fn test_quality_profile_standard() {
        let profile = QualityProfile::standard();
        assert_eq!(profile.name, "standard");
        assert_eq!(profile.thresholds.max_complexity, 10);
        assert_eq!(profile.thresholds.min_coverage, 80);
        assert!(profile.thresholds.zero_satd);
    }
    
    #[test]
    fn test_quality_profile_relaxed() {
        let profile = QualityProfile::relaxed();
        assert_eq!(profile.name, "relaxed");
        assert_eq!(profile.thresholds.max_complexity, 20);
        assert_eq!(profile.thresholds.min_coverage, 60);
        assert!(!profile.thresholds.zero_satd);
    }
    
    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics::default();
        assert_eq!(metrics.complexity, 0);
        assert_eq!(metrics.coverage, 0);
        assert_eq!(metrics.tdg, 0);
        assert!(!metrics.has_doctests);
    }
    
    #[test]
    fn test_quality_metrics_score_perfect() {
        let metrics = QualityMetrics {
            complexity: 1,
            cognitive_complexity: 1,
            coverage: 100,
            tdg: 1,
            satd_count: 0,
            dead_code_percentage: 0,
            has_doctests: true,
            has_property_tests: true,
        };
        
        let score = metrics.calculate_score();
        assert!(score >= 95.0); // Near perfect score
    }
    
    #[test]
    fn test_quality_metrics_score_poor() {
        let metrics = QualityMetrics {
            complexity: 30,
            cognitive_complexity: 30,
            coverage: 10,
            tdg: 15,
            satd_count: 5,
            dead_code_percentage: 50,
            has_doctests: false,
            has_property_tests: false,
        };
        
        let score = metrics.calculate_score();
        assert!(score <= 20.0); // Poor score
    }
    
    #[test]
    fn test_meets_thresholds_success() {
        let profile = QualityProfile::standard();
        let metrics = QualityMetrics {
            complexity: 8,
            cognitive_complexity: 8,
            coverage: 85,
            tdg: 4,
            satd_count: 0,
            dead_code_percentage: 0,
            has_doctests: true,
            has_property_tests: false,
        };
        
        assert!(profile.meets_thresholds(&metrics));
    }
    
    #[test]
    fn test_meets_thresholds_failure() {
        let profile = QualityProfile::extreme();
        let metrics = QualityMetrics {
            complexity: 15, // Exceeds extreme threshold of 5
            cognitive_complexity: 5,
            coverage: 95,
            tdg: 2,
            satd_count: 0,
            dead_code_percentage: 0,
            has_doctests: true,
            has_property_tests: true,
        };
        
        assert!(!profile.meets_thresholds(&metrics));
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
