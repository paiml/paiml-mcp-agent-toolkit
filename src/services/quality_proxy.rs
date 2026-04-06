#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::proxy::{
    ProxyMode, ProxyOperation, ProxyRequest, ProxyResponse, ProxyStatus, QualityConfig,
    QualityMetrics, QualityReport, QualityViolation, ViolationSeverity, ViolationType,
};
use crate::services::ast_rust::analyze_rust_file_with_complexity;
use crate::services::complexity::aggregate_results_with_thresholds;
use crate::services::satd_detector::SATDDetector;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Service for proxying code changes through quality gates.
///
/// This service intercepts code operations from AI agents and ensures
/// they meet quality standards before being applied.
///
/// # Example
///
/// ```ignore
/// use pmat::services::quality_proxy::QualityProxyService;
/// use pmat::models::proxy::{ProxyRequest, ProxyOperation, ProxyMode};
///
/// # async fn example() -> anyhow::Result<()> {
/// let service = QualityProxyService::new();
/// let request = ProxyRequest {
///     operation: ProxyOperation::Write,
///     file_path: "test.rs".to_string(),
///     content: Some("fn main() { println!(\"Hello\"); }".to_string()),
///     old_content: None,
///     new_content: None,
///     mode: ProxyMode::Strict,
///     quality_config: Default::default(),
/// };
///
/// let response = service.proxy_operation(request).await?;
/// assert!(response.quality_report.passed);
/// # Ok(())
/// # }
/// ```ignore
pub struct QualityProxyService {
    satd_detector: SATDDetector,
}

impl QualityProxyService {
    /// Creates a new quality proxy service.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            satd_detector: SATDDetector::new(),
        }
    }
}

impl Default for QualityProxyService {
    fn default() -> Self {
        Self::new()
    }
}

// Proxy operation orchestration and content extraction
include!("quality_proxy_operations.rs");

// Quality analysis: complexity, SATD, lint, documentation, formatting
include!("quality_proxy_analysis.rs");

// Auto-fix refactoring logic
include!("quality_proxy_autofix.rs");

// Unit tests and property tests
include!("quality_proxy_tests.rs");
