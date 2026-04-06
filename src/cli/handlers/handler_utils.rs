//! Handler utility functions - unit testable pure functions extracted from CLI handlers
//!
//! This module contains pure functions that were extracted from CLI handlers
//! to improve unit testability and coverage. These functions have no side effects
//! and can be easily tested without CLI argument parsing overhead.
//!
//! Submodule layout:
//! - handler_utils_formatting.rs: display formatting, output routing, severity/percentage
//! - handler_utils_analysis.rs: code inspection predicates (cfg gates, test markers, etc.)
//! - handler_utils_tests.rs: comprehensive unit tests for all utilities

use crate::cli;

/// Convert deep context DAG type to standard DAG type
///
/// # Examples
///
/// ```
/// use pmat::cli::handlers::handler_utils::convert_deep_context_dag_type;
/// use pmat::cli::enums::{DeepContextDagType, DagType};
///
/// let result = convert_deep_context_dag_type(DeepContextDagType::CallGraph);
/// assert_eq!(result, DagType::CallGraph);
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn convert_deep_context_dag_type(dag_type: cli::DeepContextDagType) -> cli::DagType {
    match dag_type {
        cli::DeepContextDagType::CallGraph => cli::DagType::CallGraph,
        cli::DeepContextDagType::ImportGraph => cli::DagType::ImportGraph,
        cli::DeepContextDagType::Inheritance => cli::DagType::Inheritance,
        cli::DeepContextDagType::FullDependency => cli::DagType::FullDependency,
    }
}

/// Convert cache strategy enum to string representation
///
/// # Examples
///
/// ```
/// use pmat::cli::handlers::handler_utils::convert_cache_strategy;
/// use pmat::cli::enums::DeepContextCacheStrategy;
///
/// assert_eq!(convert_cache_strategy(DeepContextCacheStrategy::Normal), "normal");
/// assert_eq!(convert_cache_strategy(DeepContextCacheStrategy::ForceRefresh), "force-refresh");
/// assert_eq!(convert_cache_strategy(DeepContextCacheStrategy::Offline), "offline");
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn convert_cache_strategy(strategy: cli::DeepContextCacheStrategy) -> String {
    match strategy {
        cli::DeepContextCacheStrategy::Normal => "normal".to_string(),
        cli::DeepContextCacheStrategy::ForceRefresh => "force-refresh".to_string(),
        cli::DeepContextCacheStrategy::Offline => "offline".to_string(),
    }
}

/// Parse threshold value with bounds checking
///
/// Ensures threshold is within valid range (0.0 - 1.0 or 0 - 100 depending on format)
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn normalize_threshold(threshold: f64, is_percentage: bool) -> f64 {
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
    let normalized = if is_percentage {
        threshold / 100.0
    } else {
        threshold
    };
    normalized.clamp(0.0, 1.0)
}

// Formatting and display utilities (output routing, severity, pluralization, duration, percentage)
include!("handler_utils_formatting.rs");

// Code analysis predicates (cfg gates, exclusions, test markers, declarations)
include!("handler_utils_analysis.rs");

// Comprehensive unit tests for all handler utilities
include!("handler_utils_tests.rs");
