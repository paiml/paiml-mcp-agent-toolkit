#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Dead Code Analysis Facade
//!
//! Provides a simplified interface for dead code detection and analysis.

use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::Serialize;
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
#[derive(Debug, Clone, Serialize)]
pub struct DeadCodeAnalysisResult {
    pub total_files: usize,
    pub dead_items: Vec<DeadCodeItem>,
    pub dead_percentage: f64,
    pub summary: String,
}

/// Individual dead code item
#[derive(Debug, Clone, Serialize)]
pub struct DeadCodeItem {
    pub file_path: String,
    pub item_name: String,
    pub item_type: DeadCodeType,
    pub line_number: usize,
    pub reason: String,
}

/// Types of dead code
#[derive(Debug, Clone, Serialize)]
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
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform dead code analysis on a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_project(
        &self,
        request: DeadCodeAnalysisRequest,
    ) -> Result<DeadCodeAnalysisResult> {
        // Wired to the same analyzer `pmat analyze dead-code` uses.
        //
        // This returned a fixed `unused_function` at line 42 with a 5.0% dead
        // rate for every project, so `analyze comprehensive` — which reaches
        // this facade through the orchestrator — reported a dead function that
        // did not exist, in every repository it was ever run against.
        use crate::services::cargo_dead_code_analyzer::{analyze_dead_code, DeadCodeKind};

        let report = analyze_dead_code(&request.path).await?;

        let dead_items: Vec<DeadCodeItem> = report
            .files_with_dead_code
            .iter()
            .flat_map(|file| {
                file.dead_items.iter().map(move |item| DeadCodeItem {
                    file_path: file.file_path.display().to_string(),
                    item_name: item.name.clone(),
                    item_type: match item.kind {
                        DeadCodeKind::Function => DeadCodeType::Function,
                        DeadCodeKind::Struct | DeadCodeKind::Enum | DeadCodeKind::Trait => {
                            DeadCodeType::Class
                        }
                        DeadCodeKind::Constant | DeadCodeKind::Static | DeadCodeKind::Field => {
                            DeadCodeType::Variable
                        }
                        _ => DeadCodeType::UnreachableCode,
                    },
                    line_number: item.line,
                    reason: item.message.clone(),
                })
            })
            .collect();

        Ok(DeadCodeAnalysisResult {
            total_files: report.total_files,
            summary: format!(
                "Found {} dead code item(s) in {}",
                dead_items.len(),
                request.path.display()
            ),
            dead_items,
            dead_percentage: report.dead_code_percentage,
        })
    }

    /// Analyze a single file for dead code
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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
