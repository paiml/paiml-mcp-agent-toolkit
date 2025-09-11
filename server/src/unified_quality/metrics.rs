//! Metrics definitions for the unified quality system

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Core quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Cyclomatic complexity
    pub complexity: u32,
    
    /// Cognitive complexity
    pub cognitive: u32,
    
    /// SATD comment count
    pub satd_count: u32,
    
    /// Code coverage percentage
    pub coverage: f64,
    
    /// Lines of code
    pub lines: u32,
    
    /// Number of functions
    pub functions: u32,
    
    /// Timestamp of measurement
    pub timestamp: SystemTime,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            complexity: 0,
            cognitive: 0,
            satd_count: 0,
            coverage: 0.0,
            lines: 0,
            functions: 0,
            timestamp: SystemTime::now(),
        }
    }
}

impl Metrics {
    /// Calculate quality score (0.0 - 1.0)
    pub fn quality_score(&self) -> f64 {
        let mut score = 1.0;
        
        // Penalize high complexity
        if self.complexity > 20 {
            score -= 0.3;
        } else if self.complexity > 10 {
            score -= 0.1;
        }
        
        // Penalize high cognitive complexity
        if self.cognitive > 15 {
            score -= 0.2;
        } else if self.cognitive > 10 {
            score -= 0.1;
        }
        
        // Penalize SATD
        if self.satd_count > 0 {
            score -= (self.satd_count as f64 * 0.05).min(0.3);
        }
        
        // Penalize low coverage
        if self.coverage < 0.6 {
            score -= 0.2;
        } else if self.coverage < 0.8 {
            score -= 0.1;
        }
        
        score.max(0.0)
    }
    
    /// Check if metrics meet quality thresholds
    pub fn meets_thresholds(&self, thresholds: &QualityThresholds) -> bool {
        self.complexity <= thresholds.max_complexity &&
        self.cognitive <= thresholds.max_cognitive &&
        self.satd_count <= thresholds.max_satd &&
        self.coverage >= thresholds.min_coverage
    }
}

/// Quality thresholds for enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub max_complexity: u32,
    pub max_cognitive: u32,
    pub max_satd: u32,
    pub min_coverage: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_complexity: 20,
            max_cognitive: 15,
            max_satd: 0,
            min_coverage: 0.8,
        }
    }
}

/// Aggregated metrics for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    /// Total files analyzed
    pub total_files: usize,
    
    /// Total functions analyzed
    pub total_functions: usize,
    
    /// Average complexity
    pub avg_complexity: f64,
    
    /// Average cognitive complexity
    pub avg_cognitive: f64,
    
    /// Total SATD issues
    pub total_satd: u32,
    
    /// Average coverage
    pub avg_coverage: f64,
    
    /// Files violating thresholds
    pub violations: Vec<Violation>,
    
    /// Overall quality score
    pub quality_score: f64,
    
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Quality violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub file: String,
    pub violation_type: ViolationType,
    pub severity: Severity,
    pub value: f64,
    pub threshold: f64,
}

/// Types of violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ViolationType {
    Complexity,
    Cognitive,
    Satd,
    Coverage,
    DeadCode,
    UnusedImport,
    Formatting,
}

/// Violation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_calculation() {
        let metrics = Metrics {
            complexity: 5,
            cognitive: 8,
            satd_count: 0,
            coverage: 0.85,
            lines: 100,
            functions: 5,
            timestamp: SystemTime::now(),
        };
        
        assert!(metrics.quality_score() > 0.9);
    }

    #[test]
    fn test_quality_score_with_violations() {
        let metrics = Metrics {
            complexity: 25,
            cognitive: 20,
            satd_count: 5,
            coverage: 0.5,
            lines: 200,
            functions: 10,
            timestamp: SystemTime::now(),
        };
        
        assert!(metrics.quality_score() < 0.5);
    }

    #[test]
    fn test_meets_thresholds() {
        let metrics = Metrics {
            complexity: 10,
            cognitive: 10,
            satd_count: 0,
            coverage: 0.8,
            lines: 100,
            functions: 5,
            timestamp: SystemTime::now(),
        };
        
        let thresholds = QualityThresholds::default();
        assert!(metrics.meets_thresholds(&thresholds));
    }
}