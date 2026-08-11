pub mod adaptive;
pub mod alerts;
pub mod analyzer_ast;
mod analyzer_ast_ruchy; // Ruchy helpers extracted for file health (CB-040)
pub mod analyzer_simple;
pub mod baseline;
pub mod baseline_analyzer;
pub mod config;
#[allow(clippy::all)]
pub mod cuda_simd;
pub mod cuda_simd_defects; // Defect taxonomy extracted for file health (CB-040)
pub mod cuda_simd_results; // Result types extracted for file health (CB-040)
pub mod cuda_simd_scores; // Score types extracted for file health (CB-040)
pub mod diagnostics;
pub mod explain;
pub mod explain_formatters;
#[cfg(feature = "rust-ast")]
pub mod function_analyzer;
pub mod grade;
pub mod hooks_config;
pub mod penalty_tracker;
pub mod project_score;
pub mod quality_gate;
pub mod recommendation_engine;
pub mod score;

#[cfg(test)]
#[path = "critical_defect_gating_tests.rs"]
mod critical_defect_gating_tests;
pub mod tdg_graph;
// Temporarily disable export to fix circular dependency
// pub mod export;
pub mod formatters;
pub mod hooks_cache;
pub mod language_simple;
pub mod metrics_aggregator;
pub mod olap_analytics;
pub mod profiler;
pub mod resource_control;
pub mod scheduler;
pub mod storage;
pub mod storage_backend;
#[cfg(feature = "http-server")]
pub mod web_dashboard;

#[cfg(test)]
mod normalization_tests;

#[cfg(test)]
mod complexity_entropy_integration_tests;

// Temporarily disable integration test to fix circular dependency
// #[cfg(test)]
// mod integration_test_sprint30;

#[cfg(test)]
mod core_tests;

#[cfg(test)]
mod core_property_tests;

#[cfg(test)]
mod top_files_determinism_tests;

pub use adaptive::{
    AdaptiveConfig, AdaptiveThresholdFactory, AdaptiveThresholdManager, CurrentThresholds,
    PerformanceSample, PerformanceStatistics, PerformanceTrend, ThresholdAdjustment,
};
// Use AST analyzer by default (proper implementation)
pub use analyzer_ast::TdgAnalyzerAst as TdgAnalyzer;
pub use analyzer_simple::{ensure_parseable, grades_source, TdgAnalyzer as TdgAnalyzerSimple};
pub use baseline::{
    BaselineComparison, BaselineEntry, BaselineSummary, FileComparison, TdgBaseline,
};
pub use config::TdgConfig;
pub use diagnostics::{
    AdaptiveDiagnostics, HealthStatus, ResourceDiagnostics, SchedulerDiagnostics,
    StorageDiagnostics, SystemDiagnostics,
};
pub use explain::{
    ActionableRecommendation, ComplexitySeverity, ExplainBaselineComparison, ExplainedTDGScore,
    FunctionComplexity, RecommendationType,
};
pub use explain_formatters::{format_explain_json, format_explain_text};
pub use formatters::{format_human, format_json, format_markdown};
#[cfg(feature = "rust-ast")]
pub use function_analyzer::FunctionAnalyzer;
pub use grade::{Grade, MetricCategory, PenaltyAttribution};
pub use hooks_config::{
    BaselineConfig, CiCdConfig, EnforcementMode, QualityGatesConfig, TdgHooksConfig,
};
pub use language_simple::{Language, LanguageRules};
#[cfg(feature = "analytics-simd")]
pub use olap_analytics::TruenoOlapAnalytics;
pub use olap_analytics::{AggOp, OlapAnalytics};
pub use penalty_tracker::PenaltyTracker;
pub use project_score::{Comparison, ProjectScore};
pub use quality_gate::{
    CriticalDefectGate, FGradeGate, GateConfig, GateResult, MinimumGradeGate, NewFileGate,
    QualityGate, RegressionGate, Severity, Violation, ViolationType,
};
pub use recommendation_engine::generate_recommendations;
pub use resource_control::{
    OperationPriority, OperationType as ResourceOperationType, PlatformResourceController,
    ResourceAction, ResourceAllocation, ResourceControllerFactory, ResourceEnforcementStats,
    ResourceLimits, ResourcePressure, ResourceUsage,
};
pub use scheduler::{
    OperationType as SchedulerOperationType, ScheduleError, ScheduleGuard, SchedulePermit,
    SchedulerFactory, SchedulingStatistics, SimpleFairScheduler,
};
pub use score::TdgScore;
pub use storage::{
    AnalysisMetadata, ComponentScores, FileIdentity, FullTdgRecord, HotCacheEntry,
    SemanticSignature, StorageStatistics, TieredStorageFactory, TieredStore,
};
pub use storage_backend::{
    InMemoryBackend, StorageBackend, StorageBackendFactory, StorageBackendType, StorageConfig,
};
#[cfg(feature = "http-server")]
pub use web_dashboard::{
    create_dashboard_router, start_dashboard_server, DashboardState,
    HealthStatus as DashboardHealthStatus, PerformanceMetrics as DashboardPerformanceMetrics,
    StorageMetrics, SystemMetrics,
};

// CUDA-SIMD TDG exports (100-point Popper falsification scoring)
pub use cuda_simd::{
    AccessPattern, BarrierIssue, BarrierSafetyResult, CoalescingResult, CudaSimdAnalyzer,
    CudaSimdConfig, CudaSimdTdgResult, CudaTdgGrade, DefectClass, DefectSeverity, DefectTaxonomy,
    DetectedDefect, FalsifiabilityScore, GpuSimdSpecificScore, HistoricalIntegrityScore,
    KaizenMetrics, MemoryAccessIssue, PopperScore, ReproducibilityScore, StatisticalRigorScore,
    TileDimensionResult, TileIssue, TransparencyScore,
};
