//! Entropy Calculation Module
//! 
//! Calculates pattern entropy metrics at different levels

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{EntropyConfig, PatternType};
use super::pattern_extractor::{AstPattern, PatternCollection};
use super::violation_detector::PatternSummary;
use super::violation_detector::ActionableViolation;

/// Entropy metrics for the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyMetrics {
    pub file_level_entropy: f64,
    pub module_level_entropy: f64,
    pub project_level_entropy: f64,
    pub pattern_diversity: f64,
    pub total_patterns: usize,
    pub total_instances: usize,
    pub total_loc: usize,
    pub patterns_by_type: HashMap<PatternType, usize>,
}

/// Report of entropy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyReport {
    pub total_files_analyzed: usize,
    pub actionable_violations: Vec<ActionableViolation>,
    pub pattern_summary: PatternSummary,
    pub entropy_metrics: EntropyMetrics,
}

impl EntropyReport {
    /// Get total estimated LOC reduction
    pub fn total_loc_reduction(&self) -> usize {
        self.actionable_violations
            .iter()
            .map(|v| v.estimated_loc_reduction)
            .sum()
    }

    /// Get reduction percentage
    pub fn reduction_percentage(&self) -> f64 {
        if self.entropy_metrics.total_loc > 0 {
            (self.total_loc_reduction() as f64 / self.entropy_metrics.total_loc as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Format as human-readable report
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("Entropy Analysis Results\n");
        report.push_str("========================\n\n");
        
        report.push_str(&format!("Files Analyzed: {}\n", self.total_files_analyzed));
        report.push_str(&format!("Actionable Violations: {}\n\n", self.actionable_violations.len()));
        
        // Group violations by severity
        let mut high = Vec::new();
        let mut medium = Vec::new();
        let mut low = Vec::new();
        
        for violation in &self.actionable_violations {
            match violation.severity {
                super::violation_detector::Severity::High => high.push(violation),
                super::violation_detector::Severity::Medium => medium.push(violation),
                super::violation_detector::Severity::Low => low.push(violation),
            }
        }
        
        if !high.is_empty() {
            report.push_str(&format!("HIGH SEVERITY ({}):\n", high.len()));
            for (i, v) in high.iter().enumerate() {
                report.push_str(&format!(
                    "{}. {}\n   Fix: {} - saves {} lines\n\n",
                    i + 1,
                    v.message,
                    v.fix_suggestion,
                    v.estimated_loc_reduction
                ));
            }
        }
        
        if !medium.is_empty() {
            report.push_str(&format!("MEDIUM SEVERITY ({}):\n", medium.len()));
            for (i, v) in medium.iter().enumerate() {
                report.push_str(&format!(
                    "{}. {}\n   Fix: {} - saves {} lines\n\n",
                    i + 1,
                    v.message,
                    v.fix_suggestion,
                    v.estimated_loc_reduction
                ));
            }
        }
        
        report.push_str(&format!(
            "Total Potential Reduction: {} lines ({:.1}% of analyzed code)\n",
            self.total_loc_reduction(),
            self.reduction_percentage()
        ));
        
        report
    }
}

/// Calculates entropy metrics
pub struct EntropyCalculator {
    config: EntropyConfig,
}

impl EntropyCalculator {
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Calculate entropy metrics from patterns
    pub fn calculate(&self, patterns: &PatternCollection) -> Result<EntropyMetrics> {
        let total_patterns = patterns.patterns.len();
        let total_instances: usize = patterns.patterns.values()
            .map(|p| p.frequency)
            .sum();
        
        let total_loc: usize = patterns.patterns.values()
            .map(|p| p.estimated_loc * p.frequency)
            .sum();
        
        // Calculate pattern diversity (Shannon entropy of pattern distribution)
        let pattern_diversity = self.calculate_pattern_diversity(patterns);
        
        // Calculate entropy at different levels
        let file_level_entropy = self.calculate_file_level_entropy(patterns);
        let module_level_entropy = self.calculate_module_level_entropy(patterns);
        let project_level_entropy = self.calculate_project_level_entropy(patterns);
        
        // Count patterns by type
        let mut patterns_by_type = HashMap::new();
        for pattern in patterns.patterns.values() {
            *patterns_by_type.entry(pattern.pattern_type).or_insert(0) += pattern.frequency;
        }
        
        Ok(EntropyMetrics {
            file_level_entropy,
            module_level_entropy,
            project_level_entropy,
            pattern_diversity,
            total_patterns,
            total_instances,
            total_loc,
            patterns_by_type,
        })
    }

    /// Calculate Shannon entropy of pattern distribution
    fn calculate_pattern_diversity(&self, patterns: &PatternCollection) -> f64 {
        if patterns.patterns.is_empty() {
            return 0.0;
        }
        
        let total_instances: usize = patterns.patterns.values()
            .map(|p| p.frequency)
            .sum();
        
        if total_instances == 0 {
            return 0.0;
        }
        
        let mut entropy = 0.0;
        for pattern in patterns.patterns.values() {
            let probability = pattern.frequency as f64 / total_instances as f64;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }
        
        // Normalize to 0-1 scale (assuming max entropy of 8 bits for code patterns)
        (entropy / 8.0).min(1.0)
    }

    /// Calculate average entropy at file level
    fn calculate_file_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Calculate how diverse patterns are within each file
        let mut file_entropies = Vec::new();
        
        for file_patterns in patterns.file_patterns.values() {
            if file_patterns.is_empty() {
                continue;
            }
            
            // Count pattern frequencies in this file
            let mut pattern_counts = HashMap::new();
            for pattern_hash in file_patterns {
                *pattern_counts.entry(pattern_hash).or_insert(0) += 1;
            }
            
            // Calculate entropy for this file
            let total = file_patterns.len() as f64;
            let mut entropy = 0.0;
            
            for count in pattern_counts.values() {
                let p = *count as f64 / total;
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }
            
            file_entropies.push(entropy);
        }
        
        if file_entropies.is_empty() {
            return 0.0;
        }
        
        // Return average file entropy
        let sum: f64 = file_entropies.iter().sum();
        (sum / file_entropies.len() as f64 / 8.0).min(1.0)
    }

    /// Calculate entropy at module level
    fn calculate_module_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Group files by module (simplified: by directory)
        let mut modules: HashMap<String, Vec<&AstPattern>> = HashMap::new();
        
        for pattern in patterns.patterns.values() {
            for location in &pattern.locations {
                let module = location.file
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("root")
                    .to_string();
                
                modules.entry(module).or_default().push(pattern);
            }
        }
        
        // Calculate entropy for each module
        let mut module_entropies = Vec::new();
        
        for module_patterns in modules.values() {
            if module_patterns.is_empty() {
                continue;
            }
            
            let mut pattern_counts = HashMap::new();
            for pattern in module_patterns {
                *pattern_counts.entry(pattern.pattern_type).or_insert(0) += 1;
            }
            
            let total = module_patterns.len() as f64;
            let mut entropy = 0.0;
            
            for count in pattern_counts.values() {
                let p = *count as f64 / total;
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }
            
            module_entropies.push(entropy);
        }
        
        if module_entropies.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = module_entropies.iter().sum();
        (sum / module_entropies.len() as f64 / 3.0).min(1.0) // Lower max for module level
    }

    /// Calculate entropy at project level
    fn calculate_project_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Overall project pattern diversity
        self.calculate_pattern_diversity(patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_metrics_creation() {
        let metrics = EntropyMetrics {
            file_level_entropy: 0.5,
            module_level_entropy: 0.6,
            project_level_entropy: 0.7,
            pattern_diversity: 0.4,
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: HashMap::new(),
        };
        
        assert_eq!(metrics.total_patterns, 10);
        assert_eq!(metrics.total_instances, 50);
    }

    #[test]
    fn test_entropy_report_calculations() {
        let report = EntropyReport {
            total_files_analyzed: 10,
            actionable_violations: vec![
                ActionableViolation {
                    severity: super::Severity::High,
                    pattern: PatternSummary {
                        pattern_type: PatternType::ErrorHandling,
                        repetitions: 10,
                        variation_score: 0.0,
                        example_code: "test".to_string(),
                    },
                    message: "test".to_string(),
                    fix_suggestion: "test".to_string(),
                    estimated_loc_reduction: 100,
                    affected_files: vec![],
                    priority_score: 10.0,
                },
            ],
            pattern_summary: PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 10,
                variation_score: 0.0,
                example_code: "test".to_string(),
            },
            entropy_metrics: EntropyMetrics {
                file_level_entropy: 0.5,
                module_level_entropy: 0.6,
                project_level_entropy: 0.7,
                pattern_diversity: 0.4,
                total_patterns: 10,
                total_instances: 50,
                total_loc: 1000,
                patterns_by_type: HashMap::new(),
            },
        };
        
        assert_eq!(report.total_loc_reduction(), 100);
        assert_eq!(report.reduction_percentage(), 10.0);
    }
}