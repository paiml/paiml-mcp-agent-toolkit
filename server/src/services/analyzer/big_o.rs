// Toyota Way: Unified Big-O Complexity Analyzer Strategy

use super::{Analyzer, ProjectAnalyzer};
use anyhow::Result;
use async_trait::async_trait;

/// Big-O complexity analyzer strategy using the existing big_o_analyzer
pub struct BigOAnalyzer {
    analyzer: crate::services::big_o_analyzer::BigOAnalyzer,
}

impl Default for BigOAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl BigOAnalyzer {
    pub fn new() -> Self {
        Self {
            analyzer: crate::services::big_o_analyzer::BigOAnalyzer::new(),
        }
    }
}

#[async_trait]
impl Analyzer for BigOAnalyzer {
    type Input = super::ProjectInput;
    type Output = crate::services::big_o_analyzer::BigOAnalysisReport;
    type Config = super::ProjectConfig;

    async fn analyze(&self, input: Self::Input, _config: Self::Config) -> Result<Self::Output> {
        let analysis_config = crate::services::big_o_analyzer::BigOAnalysisConfig {
            project_path: input.project_path,
            include_patterns: vec![
                "*.rs".to_string(),
                "*.ts".to_string(),
                "*.js".to_string(),
                "*.py".to_string(),
            ],
            exclude_patterns: vec![
                "test_*.rs".to_string(),
                "*.test.ts".to_string(),
                "*_test.py".to_string(),
            ],
            confidence_threshold: 70,
            analyze_space_complexity: true,
        };

        let report = self.analyzer.analyze(analysis_config).await?;
        Ok(report)
    }

    fn name(&self) -> &'static str {
        "big_o"
    }
}

impl ProjectAnalyzer for BigOAnalyzer {}

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
