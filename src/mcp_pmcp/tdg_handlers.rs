//! TDG (Technical Debt Grading) MCP tool handlers for Sprint 31.
//!
//! This module provides comprehensive MCP tools for the Transactional Hashed TDG System,
//! enabling external clients to interact with enterprise-grade technical debt analysis
//! through the Model Context Protocol.
//!
//! The tools include:
//! - System diagnostics and health monitoring
//! - Storage management operations
//! - Transactional analysis with caching
//! - Performance metrics and adaptive thresholds
//! - Storage backend configuration
//! - Comprehensive health checks

use crate::mcp_pmcp::tool_functions::{
    tdg_analyze_with_storage, tdg_configure_storage, tdg_health_check, tdg_performance_metrics,
    tdg_storage_management, tdg_system_diagnostics,
};
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

// === Argument Types ===

/// MCP args for tdg.system_diagnostics: optional detailed flag and list of components to inspect.
#[derive(Debug, Deserialize)]
struct TdgSystemDiagnosticsArgs {
    #[serde(default)]
    detailed: bool,
    #[serde(default)]
    components: Vec<String>,
}

/// MCP args for tdg.storage_management: action name (compact, purge, etc.) with freeform options.
#[derive(Debug, Deserialize)]
struct TdgStorageManagementArgs {
    action: String,
    #[serde(default)]
    options: Value,
}

/// MCP args for tdg.analyze_with_storage: paths to analyze, optional storage_backend and priority.
#[derive(Debug, Deserialize)]
struct TdgAnalyzeWithStorageArgs {
    paths: Vec<String>,
    #[serde(default)]
    storage_backend: Option<String>,
    #[serde(default)]
    priority: Option<String>,
}

/// MCP args for tdg.performance_metrics: optional include_history flag and list of metric names.
#[derive(Debug, Deserialize)]
struct TdgPerformanceMetricsArgs {
    #[serde(default)]
    #[allow(dead_code)]
    include_history: bool,
    #[serde(default)]
    #[allow(dead_code)]
    metrics: Vec<String>,
}

/// MCP args for tdg.configure_storage: backend_type with optional path, cache_size_mb, and compression.
#[derive(Debug, Deserialize)]
struct TdgConfigureStorageArgs {
    backend_type: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    cache_size_mb: Option<u32>,
    #[serde(default)]
    compression: Option<bool>,
}

/// MCP args for tdg.health_check: flags for include_recommendations, check_storage, and check_performance.
#[derive(Debug, Deserialize)]
struct TdgHealthCheckArgs {
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    include_recommendations: bool,
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    check_storage: bool,
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    check_performance: bool,
}

fn default_true() -> bool {
    debug_assert!(true, "contract: default_true");
    true
}

// === Tool Structs ===

/// Tool handler for TDG system diagnostics.
pub struct TdgSystemDiagnosticsTool;

/// Tool handler for TDG storage management.
pub struct TdgStorageManagementTool;

/// Tool handler for TDG file analysis with storage.
pub struct TdgAnalyzeWithStorageTool;

/// Tool handler for TDG performance metrics.
pub struct TdgPerformanceMetricsTool;

/// Tool handler for TDG storage configuration.
pub struct TdgConfigureStorageTool;

/// Tool handler for TDG health check.
pub struct TdgHealthCheckTool;

// === Implementations (split into submodule files) ===

include!("tdg_handlers_impl.rs");
include!("tdg_handlers_tests.rs");
