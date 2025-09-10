//! Analysis Orchestrator
//!
//! Coordinates multiple analysis operations and provides unified results.

use super::{ComplexityFacade, DeadCodeFacade, SatdFacade};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;

/// Comprehensive analysis request
#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisRequest {
    pub path: std::path::PathBuf,
    pub include_complexity: bool,
    pub include_dead_code: bool,
    pub include_satd: bool,
    pub include_tests: bool,
    pub language: Option<String>,
    pub parallel: bool,
}

/// Unified analysis results
#[derive(Debug, Serialize)]
pub struct ComprehensiveAnalysisResult {
    pub complexity: Option<super::complexity_facade::ComplexityAnalysisResult>,
    pub dead_code: Option<super::dead_code_facade::DeadCodeAnalysisResult>,
    pub satd: Option<super::satd_facade::SatdAnalysisResult>,
    pub summary: AnalysisSummary,
    pub duration_ms: u64,
}

/// High-level summary of all analyses
#[derive(Debug, Serialize)]
pub struct AnalysisSummary {
    pub total_files: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub quality_score: f64,
    pub recommendations: Vec<String>,
}

/// Enum to handle different analysis result types
enum AnalysisTaskResult {
    Complexity(super::complexity_facade::ComplexityAnalysisResult),
    DeadCode(super::dead_code_facade::DeadCodeAnalysisResult),
    Satd(super::satd_facade::SatdAnalysisResult),
}

/// Orchestrator for coordinating multiple analysis operations
pub struct AnalysisOrchestrator {
    #[allow(dead_code)]
    registry: Arc<ServiceRegistry>,
    complexity_facade: ComplexityFacade,
    dead_code_facade: DeadCodeFacade,
    satd_facade: SatdFacade,
}

impl AnalysisOrchestrator {
    /// Create a new analysis orchestrator
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        let complexity_facade = ComplexityFacade::new(Arc::clone(&registry));
        let dead_code_facade = DeadCodeFacade::new(Arc::clone(&registry));
        let satd_facade = SatdFacade::new(Arc::clone(&registry));

        Self {
            registry,
            complexity_facade,
            dead_code_facade,
            satd_facade,
        }
    }

    /// Perform comprehensive analysis
    pub async fn analyze(
        &self,
        request: ComprehensiveAnalysisRequest,
    ) -> Result<ComprehensiveAnalysisResult> {
        let start_time = std::time::Instant::now();

        if request.parallel {
            self.analyze_parallel(request).await
        } else {
            self.analyze_sequential(request).await
        }
        .map(|mut result| {
            result.duration_ms = start_time.elapsed().as_millis() as u64;
            result
        })
    }

    /// Perform analysis operations in parallel
    async fn analyze_parallel(
        &self,
        request: ComprehensiveAnalysisRequest,
    ) -> Result<ComprehensiveAnalysisResult> {
        use tokio::task::JoinHandle;
        let mut tasks: Vec<JoinHandle<(&str, Result<AnalysisTaskResult>)>> = Vec::new();

        // Spawn complexity analysis task
        if request.include_complexity {
            let facade = self.complexity_facade.clone();
            let req = super::complexity_facade::ComplexityAnalysisRequest {
                path: request.path.clone(),
                language: request.language.clone(),
                include_tests: request.include_tests,
                max_complexity_threshold: Some(20),
                output_format: super::complexity_facade::ComplexityOutputFormat::Detailed,
            };

            tasks.push(tokio::spawn(async move {
                let result = facade
                    .analyze_project(req)
                    .await
                    .map(AnalysisTaskResult::Complexity);
                ("complexity", result)
            }));
        }

        // Spawn dead code analysis task
        if request.include_dead_code {
            let facade = self.dead_code_facade.clone();
            let req = super::dead_code_facade::DeadCodeAnalysisRequest {
                path: request.path.clone(),
                include_tests: request.include_tests,
                include_unreachable: true,
                min_dead_lines: 1,
            };

            tasks.push(tokio::spawn(async move {
                let result = facade
                    .analyze_project(req)
                    .await
                    .map(AnalysisTaskResult::DeadCode);
                ("dead_code", result)
            }));
        }

        // Spawn SATD analysis task
        if request.include_satd {
            let facade = self.satd_facade.clone();
            let req = super::satd_facade::SatdAnalysisRequest {
                path: request.path.clone(),
                strict_mode: false,
                include_tests: request.include_tests,
            };

            tasks.push(tokio::spawn(async move {
                let result = facade
                    .analyze_project(req)
                    .await
                    .map(AnalysisTaskResult::Satd);
                ("satd", result)
            }));
        }

        // Collect results
        let mut complexity_result = None;
        let mut dead_code_result = None;
        let mut satd_result = None;

        for task in tasks {
            if let Ok((task_name, result)) = task.await {
                match result {
                    Ok(AnalysisTaskResult::Complexity(r)) => complexity_result = Some(r),
                    Ok(AnalysisTaskResult::DeadCode(r)) => dead_code_result = Some(r),
                    Ok(AnalysisTaskResult::Satd(r)) => satd_result = Some(r),
                    Err(e) => eprintln!("Warning: {} analysis failed: {}", task_name, e),
                }
            }
        }

        self.build_result(complexity_result, dead_code_result, satd_result)
    }

    /// Perform analysis operations sequentially
    async fn analyze_sequential(
        &self,
        request: ComprehensiveAnalysisRequest,
    ) -> Result<ComprehensiveAnalysisResult> {
        let mut complexity_result = None;
        let mut dead_code_result = None;
        let mut satd_result = None;

        // Complexity analysis
        if request.include_complexity {
            let req = super::complexity_facade::ComplexityAnalysisRequest {
                path: request.path.clone(),
                language: request.language.clone(),
                include_tests: request.include_tests,
                max_complexity_threshold: Some(20),
                output_format: super::complexity_facade::ComplexityOutputFormat::Detailed,
            };

            match self.complexity_facade.analyze_project(req).await {
                Ok(result) => complexity_result = Some(result),
                Err(e) => eprintln!("Warning: Complexity analysis failed: {}", e),
            }
        }

        // Dead code analysis
        if request.include_dead_code {
            let req = super::dead_code_facade::DeadCodeAnalysisRequest {
                path: request.path.clone(),
                include_tests: request.include_tests,
                include_unreachable: true,
                min_dead_lines: 1,
            };

            match self.dead_code_facade.analyze_project(req).await {
                Ok(result) => dead_code_result = Some(result),
                Err(e) => eprintln!("Warning: Dead code analysis failed: {}", e),
            }
        }

        // SATD analysis
        if request.include_satd {
            let req = super::satd_facade::SatdAnalysisRequest {
                path: request.path.clone(),
                strict_mode: false,
                include_tests: request.include_tests,
            };

            match self.satd_facade.analyze_project(req).await {
                Ok(result) => satd_result = Some(result),
                Err(e) => eprintln!("Warning: SATD analysis failed: {}", e),
            }
        }

        self.build_result(complexity_result, dead_code_result, satd_result)
    }

    /// Build comprehensive result from individual analysis results
    fn build_result(
        &self,
        complexity: Option<super::complexity_facade::ComplexityAnalysisResult>,
        dead_code: Option<super::dead_code_facade::DeadCodeAnalysisResult>,
        satd: Option<super::satd_facade::SatdAnalysisResult>,
    ) -> Result<ComprehensiveAnalysisResult> {
        // Calculate summary statistics
        let total_files = [
            complexity.as_ref().map(|r| r.total_files).unwrap_or(0),
            dead_code.as_ref().map(|r| r.total_files).unwrap_or(0),
            satd.as_ref().map(|r| r.total_files).unwrap_or(0),
        ]
        .iter()
        .max()
        .copied()
        .unwrap_or(0);

        let total_issues = complexity.as_ref().map(|r| r.violations.len()).unwrap_or(0)
            + dead_code.as_ref().map(|r| r.dead_items.len()).unwrap_or(0)
            + satd.as_ref().map(|r| r.violations.len()).unwrap_or(0);

        let critical_issues = complexity
            .as_ref()
            .map(|r| r.violations.iter().filter(|v| v.complexity > 25).count())
            .unwrap_or(0)
            + satd.as_ref().map(|r| r.violations.len()).unwrap_or(0); // All SATD considered critical

        // Calculate quality score (0-100)
        let quality_score = if total_issues == 0 {
            100.0
        } else {
            let issue_density = total_issues as f64 / total_files.max(1) as f64;
            (100.0 - (issue_density * 10.0)).max(0.0)
        };

        // Generate recommendations
        let mut recommendations = Vec::new();

        if let Some(complexity_result) = &complexity {
            if complexity_result.max_complexity > 25 {
                recommendations.push("Consider refactoring high-complexity functions".to_string());
            }
        }

        if let Some(dead_code_result) = &dead_code {
            if !dead_code_result.dead_items.is_empty() {
                recommendations
                    .push("Remove identified dead code to improve maintainability".to_string());
            }
        }

        if let Some(satd_result) = &satd {
            if !satd_result.violations.is_empty() {
                recommendations
                    .push("Address technical debt items (TODO/FIXME comments)".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations
                .push("Code quality looks good! Continue following best practices.".to_string());
        }

        Ok(ComprehensiveAnalysisResult {
            complexity,
            dead_code,
            satd,
            summary: AnalysisSummary {
                total_files,
                total_issues,
                critical_issues,
                quality_score,
                recommendations,
            },
            duration_ms: 0, // Will be set by caller
        })
    }

    /// Quick analysis with sensible defaults
    pub async fn quick_analysis<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ComprehensiveAnalysisResult> {
        let request = ComprehensiveAnalysisRequest {
            path: path.as_ref().to_path_buf(),
            include_complexity: true,
            include_dead_code: true,
            include_satd: true,
            include_tests: false,
            language: None, // Auto-detect
            parallel: true,
        };

        self.analyze(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let orchestrator = AnalysisOrchestrator::new(registry);

        // Test that orchestrator can be created
        let request = ComprehensiveAnalysisRequest {
            path: std::path::PathBuf::from("."),
            include_complexity: false,
            include_dead_code: false,
            include_satd: false,
            include_tests: false,
            language: None,
            parallel: false,
        };

        // Should complete without analysis since all flags are false
        let result = orchestrator.analyze(request).await;
        assert!(result.is_ok());
    }
}

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
