#![cfg_attr(coverage_nightly, coverage(off))]
//! Deep Context Orchestrator - Phase 4 implementation
//!
//! High-performance deep context analysis with parallel AST building,
//! unified DAG construction, and comprehensive code intelligence.

use crate::models::unified_ast::AstDag;
use crate::services::{
    unified_ast_engine::UnifiedAstEngine,
    code_intelligence::{CodeIntelligence, AnalysisRequest as CodeAnalysisRequest},
    cache::{unified_manager::UnifiedCacheManager, unified::UnifiedCacheConfig, config::CacheConfig},
};
use anyhow::Result;
use std::{
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use tokio::sync::Semaphore;
use tracing::{info, debug};
use dashmap::DashMap;

/// Deep context orchestrator for multi-language analysis
pub struct DeepContextOrchestrator {
    ast_engine: Arc<UnifiedAstEngine>,
    intelligence: Arc<CodeIntelligence>,
    cache_manager: Arc<UnifiedCacheManager>,
    max_concurrency: usize,
}

/// Configuration for deep context analysis
#[derive(Debug, Clone)]
pub struct DeepContextConfig {
    pub project_path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub features: FeatureFlags,
    pub cache_strategy: CacheStrategy,
    pub performance_mode: PerformanceMode,
}

/// Feature flags for analysis components
#[derive(Debug, Clone, Copy)]
pub struct FeatureFlags {
    pub ast_analysis: bool,
    pub complexity_analysis: bool,
    pub churn_analysis: bool,
    pub satd_analysis: bool,
    pub provability_analysis: bool,
    pub dead_code_analysis: bool,
    pub dependency_analysis: bool,
    pub hotspot_detection: bool,
}

/// Cache strategy for analysis
#[derive(Debug, Clone, Copy)]
pub enum CacheStrategy {
    /// Use all cache layers (L1 + L2 + persistent)
    Aggressive,
    /// Use memory caches only (L1 + L2)
    Normal,
    /// Use thread-local cache only (L1)
    Minimal,
    /// No caching (for testing/benchmarking)
    None,
}

/// Performance optimization mode
#[derive(Debug, Clone, Copy)]
pub enum PerformanceMode {
    /// Maximum performance, may use more memory
    Fast,
    /// Balanced performance and memory usage
    Balanced,
    /// Minimize memory usage, slower execution
    Memory,
}

/// Analysis request for code intelligence engine
pub struct OrchestrationRequest {
    pub dag: Arc<AstDag>,
    pub features: FeatureFlags,
    pub performance_hint: PerformanceMode,
}

/// Comprehensive analysis report
#[derive(Debug)]
pub struct DeepContextReport {
    pub file_count: usize,
    pub analysis_duration: std::time::Duration,
    pub ast_nodes: usize,
    pub dependencies: usize,
    pub complexity_summary: ComplexitySummary,
    pub hotspots: Vec<CodeHotspot>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug)]
/// Summary of complexity analysis.
pub struct ComplexitySummary {
    pub total_functions: usize,
    pub high_complexity_functions: usize,
    pub avg_cyclomatic: f64,
    pub avg_cognitive: f64,
    pub complexity_distribution: Vec<(u32, usize)>, // (complexity_level, function_count)
}

#[derive(Debug)]
/// Hotspot identified in code analysis.
pub struct CodeHotspot {
    pub file_path: PathBuf,
    pub function_name: String,
    pub hotspot_type: HotspotType,
    pub severity: HotspotSeverity,
    pub metrics: HotspotMetrics,
}

#[derive(Debug)]
/// Type classification for hotspot.
pub enum HotspotType {
    HighComplexity,
    HighChurn,
    LargeFunction,
    DeepNesting,
    ManyParameters,
    PotentialDefect,
}

#[derive(Debug)]
/// Severity level classification for hotspot.
pub enum HotspotSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug)]
/// Hotspot metrics.
pub struct HotspotMetrics {
    pub complexity_score: f64,
    pub defect_probability: f64,
    pub maintenance_cost: f64,
}

#[derive(Debug)]
/// Recommendation.
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub impact: RecommendationImpact,
    pub effort: RecommendationEffort,
}

#[derive(Debug)]
/// Category classification for recommendation.
pub enum RecommendationCategory {
    Refactoring,
    Performance,
    Maintainability,
    Testing,
    Architecture,
}

#[derive(Debug)]
/// Recommendation impact.
pub enum RecommendationImpact {
    High,
    Medium,
    Low,
}

#[derive(Debug)]
/// Recommendation effort.
pub enum RecommendationEffort {
    High,
    Medium,
    Low,
}

// Pipeline methods: analyze, discover_files, build_unified_dag, perform_analysis, generate_report
include!("deep_context_orchestrator_pipeline.rs");

// Factory for creating orchestrator instances
include!("deep_context_orchestrator_factory.rs");

// Unit tests and property tests
include!("deep_context_orchestrator_tests.rs");
