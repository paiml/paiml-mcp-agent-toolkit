// Toyota Way: Unified Analyzer Framework for Structural Complexity Reduction
//
// This module consolidates multiple analyzer implementations under a single,
// extensible framework to reduce structural complexity for A+ grade achievement.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod complexity;
pub mod dead_code;
pub mod satd;

/// Core analyzer trait for unified analysis framework
#[async_trait]
pub trait Analyzer {
    type Input;
    type Output;
    type Config;

    /// Perform analysis with given input and configuration
    async fn analyze(&self, input: Self::Input, config: Self::Config) -> Result<Self::Output>;

    /// Get analyzer name for metrics and logging
    fn name(&self) -> &'static str;

    /// Get analyzer version for compatibility
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// Project-level analyzer for comprehensive analysis
#[async_trait]
pub trait ProjectAnalyzer: Analyzer<Input = ProjectInput, Config = ProjectConfig> {
    /// Analyze entire project directory
    async fn analyze_project(&self, project_path: &Path) -> Result<Self::Output> {
        let input = ProjectInput {
            project_path: project_path.to_path_buf(),
        };
        let config = ProjectConfig::default();
        self.analyze(input, config).await
    }
}

/// File-level analyzer for focused analysis
#[async_trait]
pub trait FileAnalyzer: Analyzer<Input = FileInput, Config = FileConfig> {
    /// Analyze single file
    async fn analyze_file(&self, file_path: &Path, content: Option<&str>) -> Result<Self::Output> {
        let input = FileInput {
            file_path: file_path.to_path_buf(),
            content: content.map(String::from),
        };
        let config = FileConfig::default();
        self.analyze(input, config).await
    }
}

/// Common input for project-level analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInput {
    pub project_path: std::path::PathBuf,
}

/// Common configuration for project analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub include_tests: bool,
    pub max_depth: Option<usize>,
    pub parallel: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            include_tests: true,
            max_depth: None,
            parallel: true,
        }
    }
}

/// Common input for file-level analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInput {
    pub file_path: std::path::PathBuf,
    pub content: Option<String>,
}

/// Common configuration for file analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileConfig {
    pub language: Option<String>,
    pub strict_mode: bool,
}

/// Registry for managing multiple analyzers
pub struct AnalyzerRegistry {
    analyzers: std::collections::HashMap<String, Box<dyn AnalyzerInfo>>,
}

/// Trait for analyzer metadata
pub trait AnalyzerInfo {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
}

impl AnalyzerRegistry {
    pub fn new() -> Self {
        Self {
            analyzers: std::collections::HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, analyzer: T)
    where
        T: AnalyzerInfo + 'static,
    {
        self.analyzers
            .insert(analyzer.name().to_string(), Box::new(analyzer));
    }

    pub fn get_info(&self, name: &str) -> Option<&dyn AnalyzerInfo> {
        self.analyzers.get(name).map(|a| a.as_ref())
    }

    pub fn list_analyzers(&self) -> Vec<&str> {
        self.analyzers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility trait for common analysis patterns
pub trait AnalysisPatterns {
    /// Filter results based on confidence threshold
    fn filter_by_confidence<T>(&self, results: Vec<T>, threshold: f64) -> Vec<T>
    where
        T: HasConfidence;

    /// Sort results by priority
    fn sort_by_priority<T>(&self, results: &mut [T])
    where
        T: HasPriority;
}

/// Trait for items with confidence scores
pub trait HasConfidence {
    fn confidence(&self) -> f64;
}

/// Trait for items with priority levels
pub trait HasPriority {
    fn priority(&self) -> i32;
}

/// Default implementation of analysis patterns
pub struct DefaultAnalysisPatterns;

impl AnalysisPatterns for DefaultAnalysisPatterns {
    fn filter_by_confidence<T>(&self, results: Vec<T>, threshold: f64) -> Vec<T>
    where
        T: HasConfidence,
    {
        results
            .into_iter()
            .filter(|item| item.confidence() >= threshold)
            .collect()
    }

    fn sort_by_priority<T>(&self, results: &mut [T])
    where
        T: HasPriority,
    {
        results.sort_by_key(|b| std::cmp::Reverse(b.priority()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_registry() {
        let mut registry = AnalyzerRegistry::new();
        assert_eq!(registry.list_analyzers().len(), 0);

        // Registry starts empty
        assert!(registry.get_info("nonexistent").is_none());
    }

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default();
        assert!(config.include_tests);
        assert!(config.parallel);
        assert!(config.max_depth.is_none());
    }

    #[test]
    fn test_file_config_default() {
        let config = FileConfig::default();
        assert!(!config.strict_mode);
        assert!(config.language.is_none());
    }
}
