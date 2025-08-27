//! Dead Code Analysis Facade
//!
//! Provides a simplified interface for dead code detection and analysis.

use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Request for dead code analysis
#[derive(Debug, Clone)]
pub struct DeadCodeAnalysisRequest {
    pub path: std::path::PathBuf,
    pub include_tests: bool,
    pub include_unreachable: bool,
    pub min_dead_lines: usize,
}

/// Result of dead code analysis
#[derive(Debug, Clone)]
pub struct DeadCodeAnalysisResult {
    pub total_files: usize,
    pub dead_items: Vec<DeadCodeItem>,
    pub dead_percentage: f64,
    pub summary: String,
}

/// Individual dead code item
#[derive(Debug, Clone)]
pub struct DeadCodeItem {
    pub file_path: String,
    pub item_name: String,
    pub item_type: DeadCodeType,
    pub line_number: usize,
    pub reason: String,
}

/// Types of dead code
#[derive(Debug, Clone)]
pub enum DeadCodeType {
    Function,
    Class,
    Variable,
    Import,
    UnreachableCode,
}

/// Facade for dead code analysis operations
#[derive(Clone)]
pub struct DeadCodeFacade {
    registry: Arc<ServiceRegistry>,
}

impl DeadCodeFacade {
    /// Create a new dead code facade
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform dead code analysis on a project
    pub async fn analyze_project(&self, request: DeadCodeAnalysisRequest) -> Result<DeadCodeAnalysisResult> {
        // Mock implementation for interface establishment
        Ok(DeadCodeAnalysisResult {
            total_files: 1,
            dead_items: vec![DeadCodeItem {
                file_path: request.path.display().to_string(),
                item_name: "unused_function".to_string(),
                item_type: DeadCodeType::Function,
                line_number: 42,
                reason: "Function is never called".to_string(),
            }],
            dead_percentage: 5.0,
            summary: format!("Found 1 dead code item in {}", request.path.display()),
        })
    }

    /// Analyze a single file for dead code
    pub async fn analyze_file<P: AsRef<Path>>(&self, path: P) -> Result<DeadCodeAnalysisResult> {
        let request = DeadCodeAnalysisRequest {
            path: path.as_ref().to_path_buf(),
            include_tests: true,
            include_unreachable: true,
            min_dead_lines: 1,
        };
        
        self.analyze_project(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    #[tokio::test]
    async fn test_dead_code_facade_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let _facade = DeadCodeFacade::new(registry);
    }
}