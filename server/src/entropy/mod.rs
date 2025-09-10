//! Actionable Entropy Analysis Module
//! 
//! AST-based pattern entropy detection for identifying real code quality issues.
//! Focuses on actionable violations with clear fixes and LOC reduction estimates.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod pattern_extractor;
pub mod violation_detector;
pub mod entropy_calculator;

pub use pattern_extractor::{AstPattern, PatternType, PatternExtractor, PatternCollection, Location};
pub use violation_detector::{ActionableViolation, Severity, ViolationDetector, PatternSummary};
pub use entropy_calculator::{EntropyCalculator, EntropyReport, EntropyMetrics};

/// Configuration for entropy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyConfig {
    /// Maximum allowed pattern repetitions before violation
    pub max_pattern_repetition: usize,
    /// Minimum required pattern diversity (0.0-1.0)
    pub min_pattern_diversity: f64,
    /// Maximum allowed cross-file similarity (0.0-1.0)
    pub max_cross_file_similarity: f64,
    /// Maximum allowed pattern inconsistency score (0.0-1.0)
    pub max_inconsistency_score: f64,
    /// Minimum severity level to report
    pub min_severity: Severity,
    /// Pattern types to analyze
    pub pattern_types: Vec<PatternType>,
    /// Paths to exclude from analysis
    pub exclude_paths: Vec<String>,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            max_pattern_repetition: 5,
            min_pattern_diversity: 0.3,
            max_cross_file_similarity: 0.7,
            max_inconsistency_score: 0.8,
            min_severity: Severity::Medium,
            pattern_types: vec![
                PatternType::ErrorHandling,
                PatternType::DataValidation,
                PatternType::ResourceManagement,
                PatternType::ControlFlow,
                PatternType::DataTransformation,
                PatternType::ApiCall,
            ],
            exclude_paths: vec!["tests/**".to_string(), "examples/**".to_string()],
        }
    }
}

/// Main entropy analyzer
pub struct EntropyAnalyzer {
    config: EntropyConfig,
    pattern_extractor: PatternExtractor,
    violation_detector: ViolationDetector,
    entropy_calculator: EntropyCalculator,
}

impl Default for EntropyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyAnalyzer {
    /// Create new analyzer with default config
    pub fn new() -> Self {
        Self::with_config(EntropyConfig::default())
    }

    /// Create analyzer with custom config
    pub fn with_config(config: EntropyConfig) -> Self {
        Self {
            config: config.clone(),
            pattern_extractor: PatternExtractor::new(config.clone()),
            violation_detector: ViolationDetector::new(config.clone()),
            entropy_calculator: EntropyCalculator::new(config),
        }
    }

    /// Analyze entropy for a project
    pub async fn analyze(&self, project_path: &Path) -> Result<EntropyReport> {
        // Step 1: Extract AST patterns from project context
        let patterns = self.pattern_extractor.extract_patterns(project_path).await?;
        
        // Step 2: Calculate entropy metrics
        let entropy_metrics = self.entropy_calculator.calculate(&patterns)?;
        
        // Step 3: Detect actionable violations
        let violations = self.violation_detector.detect_violations(&patterns, &entropy_metrics)?;
        
        // Step 4: Generate report
        Ok(EntropyReport {
            total_files_analyzed: patterns.file_count(),
            actionable_violations: violations,
            pattern_summary: patterns.summary(),
            entropy_metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EntropyConfig::default();
        assert_eq!(config.max_pattern_repetition, 5);
        assert_eq!(config.min_pattern_diversity, 0.3);
        assert_eq!(config.max_cross_file_similarity, 0.7);
    }

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = EntropyAnalyzer::new();
        assert!(analyzer.config.pattern_types.len() > 0);
    }
}