// Toyota Way: Unified AST Processing
//
// Consolidates functionality from unified_ast_parser.rs and unified_ast_engine.rs

use super::{AstAnalysisResult, AstConfig, AstRegistry};
use crate::services::file_classifier::FileClassifier;
use anyhow::Result;
use std::path::Path;

/// High-level unified AST processor
pub struct UnifiedAstProcessor {
    registry: AstRegistry,
    classifier: FileClassifier,
    config: AstConfig,
}

impl UnifiedAstProcessor {
    pub fn new() -> Self {
        Self {
            registry: AstRegistry::new(),
            classifier: FileClassifier::default(),
            config: AstConfig::default(),
        }
    }

    pub fn with_config(config: AstConfig) -> Self {
        Self {
            registry: AstRegistry::new(),
            classifier: FileClassifier::default(),
            config,
        }
    }

    /// Process a single file using the unified AST approach
    pub async fn process_file(&self, file_path: &Path) -> Result<AstAnalysisResult> {
        let start = std::time::Instant::now();

        // Get file extension
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Find appropriate strategy
        let strategy = self
            .registry
            .get_strategy(extension)
            .ok_or_else(|| anyhow::anyhow!("No AST strategy found for extension: {}", extension))?;

        // Analyze using strategy
        let context = strategy.analyze(file_path, &self.classifier).await?;

        let duration = start.elapsed();

        Ok(AstAnalysisResult {
            file_path: file_path.to_path_buf(),
            language: context.language.clone(),
            context,
            analysis_duration_ms: duration.as_millis() as u64,
        })
    }

    /// Process multiple files in parallel
    pub async fn process_files(&self, file_paths: &[&Path]) -> Vec<Result<AstAnalysisResult>> {
        use futures::future::join_all;

        let futures = file_paths.iter().map(|&path| self.process_file(path));
        join_all(futures).await
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<&str> {
        self.registry.list_supported_extensions()
    }
}

impl Default for UnifiedAstProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_unified_processor_creation() {
        let processor = UnifiedAstProcessor::new();
        let languages = processor.supported_languages();

        assert!(languages.contains(&"rs"));
        assert!(!languages.is_empty());
    }

    #[tokio::test]
    async fn test_file_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"fn test() -> i32 { 42 }").unwrap();
        temp_file.flush().unwrap();

        let rust_file_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &rust_file_path).unwrap();

        let processor = UnifiedAstProcessor::new();
        let result = processor.process_file(&rust_file_path).await.unwrap();

        assert_eq!(result.language, "Rust");
        assert!(result.analysis_duration_ms > 0);

        std::fs::remove_file(&rust_file_path).unwrap();
    }

    #[tokio::test]
    async fn test_multiple_file_processing() {
        let processor = UnifiedAstProcessor::new();

        // Create temporary test files
        let mut temp1 = NamedTempFile::new().unwrap();
        let mut temp2 = NamedTempFile::new().unwrap();

        temp1.write_all(b"fn main() {}").unwrap();
        temp2.write_all(b"fn test() {}").unwrap();
        temp1.flush().unwrap();
        temp2.flush().unwrap();

        let rust_file1 = temp1.path().with_extension("rs");
        let rust_file2 = temp2.path().with_extension("rs");

        std::fs::copy(temp1.path(), &rust_file1).unwrap();
        std::fs::copy(temp2.path(), &rust_file2).unwrap();

        let file_paths = [rust_file1.as_path(), rust_file2.as_path()];
        let results = processor.process_files(&file_paths).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        std::fs::remove_file(&rust_file1).unwrap();
        std::fs::remove_file(&rust_file2).unwrap();
    }
}
