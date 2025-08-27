//! SATD (Self-Admitted Technical Debt) Analysis Facade
//!
//! Provides a simplified interface for SATD detection and analysis.

use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Request for SATD analysis
#[derive(Debug, Clone)]
pub struct SatdAnalysisRequest {
    pub path: std::path::PathBuf,
    pub strict_mode: bool,
    pub include_tests: bool,
}

/// Result of SATD analysis
#[derive(Debug, Clone)]
pub struct SatdAnalysisResult {
    pub total_files: usize,
    pub violations: Vec<SatdViolation>,
    pub summary: String,
}

/// Individual SATD violation
#[derive(Debug, Clone)]
pub struct SatdViolation {
    pub file_path: String,
    pub line_number: usize,
    pub violation_type: String,
    pub message: String,
    pub severity: SatdSeverity,
}

/// SATD severity levels
#[derive(Debug, Clone)]
pub enum SatdSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Facade for SATD analysis operations
#[derive(Clone)]
pub struct SatdFacade {
    registry: Arc<ServiceRegistry>,
}

impl SatdFacade {
    /// Create a new SATD facade
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform SATD analysis on a project
    pub async fn analyze_project(&self, request: SatdAnalysisRequest) -> Result<SatdAnalysisResult> {
        // Mock implementation for interface establishment
        Ok(SatdAnalysisResult {
            total_files: 1,
            violations: vec![SatdViolation {
                file_path: request.path.display().to_string(),
                line_number: 42,
                violation_type: "TODO".to_string(),
                message: "Implement proper error handling".to_string(),
                severity: SatdSeverity::Medium,
            }],
            summary: format!("Found 1 SATD violation in {}", request.path.display()),
        })
    }

    /// Analyze a single file for SATD
    pub async fn analyze_file<P: AsRef<Path>>(&self, path: P) -> Result<SatdAnalysisResult> {
        let request = SatdAnalysisRequest {
            path: path.as_ref().to_path_buf(),
            strict_mode: false,
            include_tests: true,
        };
        
        self.analyze_project(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    #[tokio::test]
    async fn test_satd_facade_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let _facade = SatdFacade::new(registry);
    }
}