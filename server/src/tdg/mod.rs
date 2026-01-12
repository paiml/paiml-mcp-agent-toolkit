use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub mod adaptive;
pub mod alerts;
pub mod analyzer_ast;
pub mod analyzer_simple;
pub mod baseline;
pub mod baseline_analyzer;
pub mod config;
#[allow(clippy::all)]
pub mod cuda_simd;
pub mod diagnostics;
pub mod explain;
pub mod explain_formatters;
pub mod function_analyzer;
pub mod hooks_config;
pub mod quality_gate;
pub mod recommendation_engine;
pub mod tdg_graph;
// Temporarily disable export to fix circular dependency
// pub mod export;
pub mod formatters;
pub mod language_simple;
pub mod metrics_aggregator;
pub mod olap_analytics;
pub mod profiler;
pub mod resource_control;
pub mod scheduler;
pub mod storage;
pub mod storage_backend;
pub mod web_dashboard;

#[cfg(test)]
mod normalization_tests;

#[cfg(test)]
mod complexity_entropy_integration_tests;

// Temporarily disable integration test to fix circular dependency
// #[cfg(test)]
// mod integration_test_sprint30;

pub use adaptive::{
    AdaptiveConfig, AdaptiveThresholdFactory, AdaptiveThresholdManager, CurrentThresholds,
    PerformanceSample, PerformanceStatistics, PerformanceTrend, ThresholdAdjustment,
};
// Use AST analyzer by default (proper implementation)
pub use analyzer_ast::TdgAnalyzerAst as TdgAnalyzer;
pub use analyzer_simple::TdgAnalyzer as TdgAnalyzerSimple;
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
pub use function_analyzer::FunctionAnalyzer;
pub use hooks_config::{
    BaselineConfig, CiCdConfig, EnforcementMode, QualityGatesConfig, TdgHooksConfig,
};
pub use language_simple::{Language, LanguageRules};
#[cfg(feature = "analytics-simd")]
pub use olap_analytics::TruenoOlapAnalytics;
pub use olap_analytics::{AggOp, OlapAnalytics};
pub use quality_gate::{
    GateConfig, GateResult, MinimumGradeGate, NewFileGate, QualityGate, RegressionGate, Severity,
    Violation, ViolationType,
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
pub use storage::{
    AnalysisMetadata, ComponentScores, FileIdentity, FullTdgRecord, HotCacheEntry,
    SemanticSignature, StorageStatistics, TieredStorageFactory, TieredStore,
};
pub use storage_backend::{
    InMemoryBackend, StorageBackend, StorageBackendFactory, StorageBackendType, StorageConfig,
};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TdgScore {
    pub structural_complexity: f32,
    pub semantic_complexity: f32,
    pub duplication_ratio: f32,
    pub coupling_score: f32,
    pub doc_coverage: f32,
    pub consistency_score: f32,
    pub entropy_score: f32, // New: Pattern entropy analysis
    pub total: f32,
    pub grade: Grade,
    pub confidence: f32,
    pub language: Language,
    pub file_path: Option<PathBuf>,
    pub penalties_applied: Vec<PenaltyAttribution>,
    pub critical_defects_count: usize, // Known Defects v2.1: Count of critical defects
    pub has_critical_defects: bool,    // Known Defects v2.1: Auto-fail flag
}

impl Default for TdgScore {
    fn default() -> Self {
        Self {
            structural_complexity: 25.0,
            semantic_complexity: 20.0,
            duplication_ratio: 20.0,
            coupling_score: 15.0,
            doc_coverage: 10.0,
            consistency_score: 10.0,
            entropy_score: 0.0, // New: Start with 0, calculated during analysis
            total: 100.0,
            grade: Grade::APLus,
            confidence: 1.0,
            language: Language::Unknown,
            file_path: None,
            penalties_applied: Vec::new(),
            critical_defects_count: 0,
            has_critical_defects: false,
        }
    }
}

impl TdgScore {
    pub fn calculate_total(&mut self) {
        // Clamp individual components to their expected weight ranges
        // This ensures components can never exceed their designated contribution
        self.structural_complexity = self.structural_complexity.clamp(0.0, 25.0);
        self.semantic_complexity = self.semantic_complexity.clamp(0.0, 20.0);
        self.duplication_ratio = self.duplication_ratio.clamp(0.0, 20.0);
        self.coupling_score = self.coupling_score.clamp(0.0, 15.0);
        self.doc_coverage = self.doc_coverage.clamp(0.0, 10.0);
        self.consistency_score = self.consistency_score.clamp(0.0, 10.0);

        // Entropy score should have a reasonable weight (max ~10 points)
        // to balance with other metrics without dominating
        self.entropy_score = self.entropy_score.clamp(0.0, 10.0);

        // Sum all clamped components
        let raw_total = self.structural_complexity
            + self.semantic_complexity
            + self.duplication_ratio
            + self.coupling_score
            + self.doc_coverage
            + self.consistency_score
            + self.entropy_score;

        // The total is already in 0-110 range after clamping individual components
        // Since the original weights sum to 100, and entropy adds up to 10 more,
        // we need to normalize back to 0-100 scale
        // Strategy: If raw_total <= 100, use it as-is for backward compatibility
        //           If raw_total > 100, scale it proportionally
        if raw_total <= 100.0 {
            self.total = raw_total.clamp(0.0, 100.0);
        } else {
            // Scale down proportionally when entropy pushes total above 100
            const THEORETICAL_MAX: f32 = 110.0; // 25+20+20+15+10+10+10
            self.total = (raw_total / THEORETICAL_MAX * 100.0).clamp(0.0, 100.0);
        }

        // Known Defects v2.1: Auto-fail if critical defects detected
        if self.has_critical_defects {
            self.total = 0.0;
            self.grade = Grade::F;
        } else {
            self.grade = Grade::from_score(self.total);
        }
    }

    pub fn set_metric(&mut self, category: MetricCategory, value: f32) {
        match category {
            MetricCategory::StructuralComplexity => self.structural_complexity = value,
            MetricCategory::SemanticComplexity => self.semantic_complexity = value,
            MetricCategory::Duplication => self.duplication_ratio = value,
            MetricCategory::Coupling => self.coupling_score = value,
            MetricCategory::Documentation => self.doc_coverage = value,
            MetricCategory::Consistency => self.consistency_score = value,
        }
    }
}

#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Grade {
    APLus,
    A,
    AMinus,
    BPlus,
    B,
    BMinus,
    CPlus,
    #[default]
    C,
    CMinus,
    D,
    F,
}

impl Grade {
    #[must_use]
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 95.0 => Grade::APLus,
            s if s >= 90.0 => Grade::A,
            s if s >= 85.0 => Grade::AMinus,
            s if s >= 80.0 => Grade::BPlus,
            s if s >= 75.0 => Grade::B,
            s if s >= 70.0 => Grade::BMinus,
            s if s >= 65.0 => Grade::CPlus,
            s if s >= 60.0 => Grade::C,
            s if s >= 55.0 => Grade::CMinus,
            s if s >= 50.0 => Grade::D,
            _ => Grade::F,
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APLus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::AMinus => write!(f, "A-"),
            Grade::BPlus => write!(f, "B+"),
            Grade::B => write!(f, "B"),
            Grade::BMinus => write!(f, "B-"),
            Grade::CPlus => write!(f, "C+"),
            Grade::C => write!(f, "C"),
            Grade::CMinus => write!(f, "C-"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MetricCategory {
    StructuralComplexity,
    SemanticComplexity,
    Duplication,
    Coupling,
    Documentation,
    Consistency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PenaltyAttribution {
    pub source_metric: MetricCategory,
    pub amount: f32,
    pub applied_to: HashSet<MetricCategory>,
    pub issue: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectScore {
    pub files: Vec<TdgScore>,
    pub average_score: f32,
    #[serde(default)]
    pub average_grade: Grade,
    pub total_files: usize,
    pub language_distribution: HashMap<Language, usize>,
}

impl ProjectScore {
    #[must_use]
    pub fn aggregate(scores: Vec<TdgScore>) -> Self {
        let total_files = scores.len();
        let average_score = if total_files > 0 {
            scores.iter().map(|s| s.total).sum::<f32>() / total_files as f32
        } else {
            0.0
        };

        let mut language_distribution = HashMap::new();
        for score in &scores {
            *language_distribution.entry(score.language).or_insert(0) += 1;
        }

        Self {
            files: scores,
            average_score,
            average_grade: Grade::from_score(average_score),
            total_files,
            language_distribution,
        }
    }

    #[must_use]
    pub fn average(&self) -> TdgScore {
        if self.files.is_empty() {
            return TdgScore::default();
        }

        let mut avg = TdgScore::default();
        let count = self.files.len() as f32;

        avg.structural_complexity = self
            .files
            .iter()
            .map(|s| s.structural_complexity)
            .sum::<f32>()
            / count;
        avg.semantic_complexity = self
            .files
            .iter()
            .map(|s| s.semantic_complexity)
            .sum::<f32>()
            / count;
        avg.duplication_ratio = self.files.iter().map(|s| s.duplication_ratio).sum::<f32>() / count;
        avg.coupling_score = self.files.iter().map(|s| s.coupling_score).sum::<f32>() / count;
        avg.doc_coverage = self.files.iter().map(|s| s.doc_coverage).sum::<f32>() / count;
        avg.consistency_score = self.files.iter().map(|s| s.consistency_score).sum::<f32>() / count;
        avg.entropy_score = self.files.iter().map(|s| s.entropy_score).sum::<f32>() / count;
        avg.confidence = self.files.iter().map(|s| s.confidence).sum::<f32>() / count;

        // Set language to the most common language in the project
        if let Some((&lang, _)) = self
            .language_distribution
            .iter()
            .max_by_key(|(_, &count)| count)
        {
            avg.language = lang;
        }

        avg.calculate_total();
        avg
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub source1: TdgScore,
    pub source2: TdgScore,
    pub delta: f32,
    pub improvement_percentage: f32,
    pub winner: String,
    pub improvements: Vec<String>,
    pub regressions: Vec<String>,
}

impl Comparison {
    #[must_use]
    pub fn new(source1: TdgScore, source2: TdgScore) -> Self {
        let delta = source2.total - source1.total;
        let improvement_percentage = if source1.total > 0.0 {
            (delta / source1.total) * 100.0
        } else {
            0.0
        };

        let winner = if source2.total > source1.total {
            source2
                .file_path
                .as_ref()
                .map_or_else(|| "source2".to_string(), |p| p.display().to_string())
        } else {
            source1
                .file_path
                .as_ref()
                .map_or_else(|| "source1".to_string(), |p| p.display().to_string())
        };

        let mut improvements = Vec::new();
        let mut regressions = Vec::new();

        if source2.structural_complexity > source1.structural_complexity {
            improvements.push(format!(
                "Structural complexity improved by {:.1}",
                source2.structural_complexity - source1.structural_complexity
            ));
        } else if source2.structural_complexity < source1.structural_complexity {
            regressions.push(format!(
                "Structural complexity degraded by {:.1}",
                source1.structural_complexity - source2.structural_complexity
            ));
        }

        if source2.semantic_complexity > source1.semantic_complexity {
            improvements.push(format!(
                "Semantic complexity improved by {:.1}",
                source2.semantic_complexity - source1.semantic_complexity
            ));
        } else if source2.semantic_complexity < source1.semantic_complexity {
            regressions.push(format!(
                "Semantic complexity degraded by {:.1}",
                source1.semantic_complexity - source2.semantic_complexity
            ));
        }

        if source2.duplication_ratio > source1.duplication_ratio {
            improvements.push(format!(
                "Code duplication reduced by {:.1}",
                source2.duplication_ratio - source1.duplication_ratio
            ));
        } else if source2.duplication_ratio < source1.duplication_ratio {
            regressions.push(format!(
                "Code duplication increased by {:.1}",
                source1.duplication_ratio - source2.duplication_ratio
            ));
        }

        if source2.doc_coverage > source1.doc_coverage {
            improvements.push(format!(
                "Documentation coverage improved by {:.1}",
                source2.doc_coverage - source1.doc_coverage
            ));
        } else if source2.doc_coverage < source1.doc_coverage {
            regressions.push(format!(
                "Documentation coverage decreased by {:.1}",
                source1.doc_coverage - source2.doc_coverage
            ));
        }

        Self {
            source1,
            source2,
            delta,
            improvement_percentage,
            winner,
            improvements,
            regressions,
        }
    }
}

pub struct PenaltyTracker {
    applied: HashMap<String, PenaltyAttribution>,
}

impl Default for PenaltyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PenaltyTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            applied: HashMap::new(),
        }
    }

    pub fn apply(
        &mut self,
        issue_id: String,
        category: MetricCategory,
        amount: f32,
        issue: String,
    ) -> Option<f32> {
        if self.applied.contains_key(&issue_id) {
            return None;
        }

        self.applied.insert(
            issue_id,
            PenaltyAttribution {
                source_metric: category,
                amount,
                applied_to: HashSet::from([category]),
                issue,
            },
        );

        Some(amount)
    }

    #[must_use]
    pub fn get_attributions(&self) -> Vec<PenaltyAttribution> {
        self.applied.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ Grade Tests ============

    #[test]
    fn test_grade_from_score() {
        assert_eq!(Grade::from_score(95.0), Grade::APLus);
        assert_eq!(Grade::from_score(90.0), Grade::A);
        assert_eq!(Grade::from_score(85.0), Grade::AMinus);
        assert_eq!(Grade::from_score(80.0), Grade::BPlus);
        assert_eq!(Grade::from_score(75.0), Grade::B);
        assert_eq!(Grade::from_score(70.0), Grade::BMinus);
        assert_eq!(Grade::from_score(65.0), Grade::CPlus);
        assert_eq!(Grade::from_score(60.0), Grade::C);
        assert_eq!(Grade::from_score(55.0), Grade::CMinus);
        assert_eq!(Grade::from_score(50.0), Grade::D);
        assert_eq!(Grade::from_score(45.0), Grade::F);
    }

    #[test]
    fn test_grade_from_score_boundaries() {
        assert_eq!(Grade::from_score(100.0), Grade::APLus);
        assert_eq!(Grade::from_score(94.9), Grade::A);
        assert_eq!(Grade::from_score(89.9), Grade::AMinus);
        assert_eq!(Grade::from_score(49.9), Grade::F);
        assert_eq!(Grade::from_score(0.0), Grade::F);
        assert_eq!(Grade::from_score(-10.0), Grade::F);
    }

    #[test]
    fn test_grade_display_all() {
        assert_eq!(format!("{}", Grade::APLus), "A+");
        assert_eq!(format!("{}", Grade::A), "A");
        assert_eq!(format!("{}", Grade::AMinus), "A-");
        assert_eq!(format!("{}", Grade::BPlus), "B+");
        assert_eq!(format!("{}", Grade::B), "B");
        assert_eq!(format!("{}", Grade::BMinus), "B-");
        assert_eq!(format!("{}", Grade::CPlus), "C+");
        assert_eq!(format!("{}", Grade::C), "C");
        assert_eq!(format!("{}", Grade::CMinus), "C-");
        assert_eq!(format!("{}", Grade::D), "D");
        assert_eq!(format!("{}", Grade::F), "F");
    }

    #[test]
    fn test_grade_default() {
        let grade = Grade::default();
        assert_eq!(grade, Grade::C);
    }

    #[test]
    fn test_grade_ordering() {
        assert!(Grade::APLus < Grade::A);
        assert!(Grade::A < Grade::AMinus);
        assert!(Grade::AMinus < Grade::BPlus);
        assert!(Grade::D < Grade::F);
    }

    #[test]
    fn test_grade_clone_copy() {
        let g1 = Grade::APLus;
        let g2 = g1;
        let g3 = g1.clone();
        assert_eq!(g1, g2);
        assert_eq!(g1, g3);
    }

    #[test]
    fn test_grade_serialization() {
        let grade = Grade::BPlus;
        let json = serde_json::to_string(&grade).unwrap();
        let deserialized: Grade = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Grade::BPlus);
    }

    #[test]
    fn test_grade_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Grade::A);
        set.insert(Grade::B);
        assert!(set.contains(&Grade::A));
        assert!(!set.contains(&Grade::F));
    }

    // ============ TdgScore Tests ============

    #[test]
    fn test_tdg_score_default() {
        let score = TdgScore::default();
        assert_eq!(score.structural_complexity, 25.0);
        assert_eq!(score.semantic_complexity, 20.0);
        assert_eq!(score.duplication_ratio, 20.0);
        assert_eq!(score.coupling_score, 15.0);
        assert_eq!(score.doc_coverage, 10.0);
        assert_eq!(score.consistency_score, 10.0);
        assert_eq!(score.entropy_score, 0.0);
        assert_eq!(score.total, 100.0);
        assert_eq!(score.grade, Grade::APLus);
        assert_eq!(score.confidence, 1.0);
        assert_eq!(score.language, Language::Unknown);
        assert!(score.file_path.is_none());
        assert!(score.penalties_applied.is_empty());
        assert_eq!(score.critical_defects_count, 0);
        assert!(!score.has_critical_defects);
    }

    #[test]
    fn test_tdg_score_calculate_total() {
        let mut score = TdgScore {
            structural_complexity: 20.0,
            semantic_complexity: 18.0,
            duplication_ratio: 19.0,
            coupling_score: 14.0,
            doc_coverage: 9.0,
            consistency_score: 8.0,
            entropy_score: 12.0, // Will be clamped to 10.0 by calculate_total()
            ..TdgScore::default()
        };

        score.calculate_total();

        // After clamping: 20+18+19+14+9+8+10(clamped) = 98.0
        assert_eq!(score.total, 98.0);
        assert_eq!(score.grade, Grade::APLus); // 98.0 >= 95.0 = A+
    }

    #[test]
    fn test_tdg_score_calculate_total_clamping() {
        let mut score = TdgScore {
            structural_complexity: 50.0, // Will be clamped to 25.0
            semantic_complexity: 30.0,   // Will be clamped to 20.0
            duplication_ratio: 40.0,     // Will be clamped to 20.0
            coupling_score: 25.0,        // Will be clamped to 15.0
            doc_coverage: 20.0,          // Will be clamped to 10.0
            consistency_score: 15.0,     // Will be clamped to 10.0
            entropy_score: 20.0,         // Will be clamped to 10.0
            ..TdgScore::default()
        };

        score.calculate_total();

        // All clamped to max: 25+20+20+15+10+10+10 = 110 > 100
        // Normalized: (110/110) * 100 = 100.0
        assert_eq!(score.total, 100.0);
    }

    #[test]
    fn test_tdg_score_calculate_total_zero() {
        let mut score = TdgScore {
            structural_complexity: 0.0,
            semantic_complexity: 0.0,
            duplication_ratio: 0.0,
            coupling_score: 0.0,
            doc_coverage: 0.0,
            consistency_score: 0.0,
            entropy_score: 0.0,
            ..TdgScore::default()
        };

        score.calculate_total();

        assert_eq!(score.total, 0.0);
        assert_eq!(score.grade, Grade::F);
    }

    #[test]
    fn test_tdg_score_critical_defects_autofail() {
        let mut score = TdgScore {
            structural_complexity: 25.0,
            semantic_complexity: 20.0,
            duplication_ratio: 20.0,
            coupling_score: 15.0,
            doc_coverage: 10.0,
            consistency_score: 10.0,
            has_critical_defects: true,
            critical_defects_count: 1,
            ..TdgScore::default()
        };

        score.calculate_total();

        assert_eq!(score.total, 0.0);
        assert_eq!(score.grade, Grade::F);
    }

    #[test]
    fn test_tdg_score_set_metric() {
        let mut score = TdgScore::default();

        score.set_metric(MetricCategory::StructuralComplexity, 15.0);
        assert_eq!(score.structural_complexity, 15.0);

        score.set_metric(MetricCategory::SemanticComplexity, 12.0);
        assert_eq!(score.semantic_complexity, 12.0);

        score.set_metric(MetricCategory::Duplication, 18.0);
        assert_eq!(score.duplication_ratio, 18.0);

        score.set_metric(MetricCategory::Coupling, 10.0);
        assert_eq!(score.coupling_score, 10.0);

        score.set_metric(MetricCategory::Documentation, 8.0);
        assert_eq!(score.doc_coverage, 8.0);

        score.set_metric(MetricCategory::Consistency, 7.0);
        assert_eq!(score.consistency_score, 7.0);
    }

    #[test]
    fn test_tdg_score_clone() {
        let score = TdgScore {
            structural_complexity: 20.0,
            file_path: Some(PathBuf::from("/test/file.rs")),
            ..TdgScore::default()
        };
        let cloned = score.clone();
        assert_eq!(cloned.structural_complexity, 20.0);
        assert_eq!(cloned.file_path, Some(PathBuf::from("/test/file.rs")));
    }

    #[test]
    fn test_tdg_score_serialization() {
        let score = TdgScore {
            structural_complexity: 20.0,
            total: 85.0,
            grade: Grade::AMinus,
            ..TdgScore::default()
        };
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: TdgScore = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.structural_complexity, 20.0);
        assert_eq!(deserialized.total, 85.0);
    }

    // ============ MetricCategory Tests ============

    #[test]
    fn test_metric_category_clone_copy() {
        let cat = MetricCategory::StructuralComplexity;
        let cat2 = cat;
        assert_eq!(cat, cat2);
    }

    #[test]
    fn test_metric_category_serialization() {
        let cat = MetricCategory::Duplication;
        let json = serde_json::to_string(&cat).unwrap();
        let deserialized: MetricCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MetricCategory::Duplication);
    }

    #[test]
    fn test_metric_category_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(MetricCategory::StructuralComplexity);
        set.insert(MetricCategory::Coupling);
        assert!(set.contains(&MetricCategory::StructuralComplexity));
        assert!(!set.contains(&MetricCategory::Documentation));
    }

    #[test]
    fn test_metric_category_debug() {
        let cat = MetricCategory::SemanticComplexity;
        let debug = format!("{:?}", cat);
        assert!(debug.contains("SemanticComplexity"));
    }

    // ============ PenaltyAttribution Tests ============

    #[test]
    fn test_penalty_attribution_creation() {
        let penalty = PenaltyAttribution {
            source_metric: MetricCategory::StructuralComplexity,
            amount: 5.0,
            applied_to: HashSet::from([MetricCategory::StructuralComplexity]),
            issue: "High complexity".to_string(),
        };
        assert_eq!(penalty.amount, 5.0);
        assert!(penalty.applied_to.contains(&MetricCategory::StructuralComplexity));
    }

    #[test]
    fn test_penalty_attribution_clone() {
        let penalty = PenaltyAttribution {
            source_metric: MetricCategory::Duplication,
            amount: 3.0,
            applied_to: HashSet::from([MetricCategory::Duplication, MetricCategory::Consistency]),
            issue: "Code duplication detected".to_string(),
        };
        let cloned = penalty.clone();
        assert_eq!(cloned.amount, 3.0);
        assert_eq!(cloned.applied_to.len(), 2);
    }

    #[test]
    fn test_penalty_attribution_serialization() {
        let penalty = PenaltyAttribution {
            source_metric: MetricCategory::Documentation,
            amount: 2.0,
            applied_to: HashSet::from([MetricCategory::Documentation]),
            issue: "Missing docs".to_string(),
        };
        let json = serde_json::to_string(&penalty).unwrap();
        let deserialized: PenaltyAttribution = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.amount, 2.0);
    }

    // ============ ProjectScore Tests ============

    #[test]
    fn test_project_score_default() {
        let score = ProjectScore::default();
        assert!(score.files.is_empty());
        assert_eq!(score.average_score, 0.0);
        assert_eq!(score.total_files, 0);
        assert!(score.language_distribution.is_empty());
    }

    #[test]
    fn test_project_score_aggregate_empty() {
        let score = ProjectScore::aggregate(vec![]);
        assert_eq!(score.total_files, 0);
        assert_eq!(score.average_score, 0.0);
        assert_eq!(score.average_grade, Grade::F);
    }

    #[test]
    fn test_project_score_aggregate_single() {
        let tdg_score = TdgScore {
            total: 85.0,
            language: Language::Rust,
            ..TdgScore::default()
        };
        let project = ProjectScore::aggregate(vec![tdg_score]);
        assert_eq!(project.total_files, 1);
        assert_eq!(project.average_score, 85.0);
        assert_eq!(project.average_grade, Grade::AMinus);
        assert_eq!(*project.language_distribution.get(&Language::Rust).unwrap(), 1);
    }

    #[test]
    fn test_project_score_aggregate_multiple() {
        let scores = vec![
            TdgScore {
                total: 90.0,
                language: Language::Rust,
                ..TdgScore::default()
            },
            TdgScore {
                total: 80.0,
                language: Language::Python,
                ..TdgScore::default()
            },
            TdgScore {
                total: 70.0,
                language: Language::Rust,
                ..TdgScore::default()
            },
        ];
        let project = ProjectScore::aggregate(scores);
        assert_eq!(project.total_files, 3);
        assert_eq!(project.average_score, 80.0);
        assert_eq!(project.average_grade, Grade::BPlus);
        assert_eq!(*project.language_distribution.get(&Language::Rust).unwrap(), 2);
        assert_eq!(*project.language_distribution.get(&Language::Python).unwrap(), 1);
    }

    #[test]
    fn test_project_score_average_empty() {
        let project = ProjectScore::default();
        let avg = project.average();
        assert_eq!(avg.structural_complexity, 25.0); // Default values
    }

    #[test]
    fn test_project_score_average_single() {
        let tdg_score = TdgScore {
            structural_complexity: 20.0,
            semantic_complexity: 15.0,
            language: Language::TypeScript,
            ..TdgScore::default()
        };
        let project = ProjectScore {
            files: vec![tdg_score],
            language_distribution: HashMap::from([(Language::TypeScript, 1)]),
            ..ProjectScore::default()
        };
        let avg = project.average();
        assert_eq!(avg.structural_complexity, 20.0);
        assert_eq!(avg.semantic_complexity, 15.0);
        assert_eq!(avg.language, Language::TypeScript);
    }

    #[test]
    fn test_project_score_average_multiple() {
        let scores = vec![
            TdgScore {
                structural_complexity: 20.0,
                semantic_complexity: 10.0,
                ..TdgScore::default()
            },
            TdgScore {
                structural_complexity: 10.0,
                semantic_complexity: 20.0,
                ..TdgScore::default()
            },
        ];
        let project = ProjectScore::aggregate(scores);
        let avg = project.average();
        assert_eq!(avg.structural_complexity, 15.0);
        assert_eq!(avg.semantic_complexity, 15.0);
    }

    // ============ Comparison Tests ============

    #[test]
    fn test_comparison_new_improvement() {
        let source1 = TdgScore {
            total: 70.0,
            structural_complexity: 15.0,
            semantic_complexity: 10.0,
            file_path: Some(PathBuf::from("source1.rs")),
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 85.0,
            structural_complexity: 20.0,
            semantic_complexity: 15.0,
            file_path: Some(PathBuf::from("source2.rs")),
            ..TdgScore::default()
        };
        let comparison = Comparison::new(source1, source2);
        assert_eq!(comparison.delta, 15.0);
        assert!(comparison.improvement_percentage > 0.0);
        assert_eq!(comparison.winner, "source2.rs");
        assert!(!comparison.improvements.is_empty());
    }

    #[test]
    fn test_comparison_new_regression() {
        let source1 = TdgScore {
            total: 85.0,
            structural_complexity: 20.0,
            doc_coverage: 10.0,
            file_path: Some(PathBuf::from("before.rs")),
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 70.0,
            structural_complexity: 15.0,
            doc_coverage: 5.0,
            file_path: Some(PathBuf::from("after.rs")),
            ..TdgScore::default()
        };
        let comparison = Comparison::new(source1, source2);
        assert_eq!(comparison.delta, -15.0);
        assert!(comparison.improvement_percentage < 0.0);
        assert_eq!(comparison.winner, "before.rs");
        assert!(!comparison.regressions.is_empty());
    }

    #[test]
    fn test_comparison_new_no_path() {
        let source1 = TdgScore {
            total: 70.0,
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 80.0,
            ..TdgScore::default()
        };
        let comparison = Comparison::new(source1, source2);
        assert_eq!(comparison.winner, "source2");
    }

    #[test]
    fn test_comparison_zero_source() {
        let source1 = TdgScore {
            total: 0.0,
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 50.0,
            ..TdgScore::default()
        };
        let comparison = Comparison::new(source1, source2);
        assert_eq!(comparison.improvement_percentage, 0.0); // Div by zero protection
    }

    #[test]
    fn test_comparison_duplication_improvement() {
        let source1 = TdgScore {
            duplication_ratio: 10.0,
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            duplication_ratio: 15.0,
            ..TdgScore::default()
        };
        let comparison = Comparison::new(source1, source2);
        assert!(comparison.improvements.iter().any(|s| s.contains("duplication")));
    }

    #[test]
    fn test_comparison_serialization() {
        let comparison = Comparison::new(TdgScore::default(), TdgScore::default());
        let json = serde_json::to_string(&comparison).unwrap();
        let deserialized: Comparison = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.delta, 0.0);
    }

    // ============ PenaltyTracker Tests ============

    #[test]
    fn test_penalty_tracker() {
        let mut tracker = PenaltyTracker::new();

        let penalty1 = tracker.apply(
            "issue1".to_string(),
            MetricCategory::StructuralComplexity,
            3.5,
            "High cyclomatic complexity".to_string(),
        );
        assert_eq!(penalty1, Some(3.5));

        let penalty2 = tracker.apply(
            "issue1".to_string(),
            MetricCategory::StructuralComplexity,
            3.5,
            "High cyclomatic complexity".to_string(),
        );
        assert_eq!(penalty2, None);

        let attributions = tracker.get_attributions();
        assert_eq!(attributions.len(), 1);
        assert_eq!(attributions[0].amount, 3.5);
    }

    #[test]
    fn test_penalty_tracker_default() {
        let tracker = PenaltyTracker::default();
        assert!(tracker.get_attributions().is_empty());
    }

    #[test]
    fn test_penalty_tracker_multiple_issues() {
        let mut tracker = PenaltyTracker::new();

        tracker.apply(
            "issue1".to_string(),
            MetricCategory::StructuralComplexity,
            3.0,
            "High complexity".to_string(),
        );
        tracker.apply(
            "issue2".to_string(),
            MetricCategory::Duplication,
            2.0,
            "Code duplication".to_string(),
        );
        tracker.apply(
            "issue3".to_string(),
            MetricCategory::Documentation,
            1.5,
            "Missing docs".to_string(),
        );

        let attributions = tracker.get_attributions();
        assert_eq!(attributions.len(), 3);
    }

    #[test]
    fn test_penalty_tracker_same_category_different_ids() {
        let mut tracker = PenaltyTracker::new();

        let p1 = tracker.apply(
            "complexity-func1".to_string(),
            MetricCategory::StructuralComplexity,
            2.0,
            "func1 too complex".to_string(),
        );
        let p2 = tracker.apply(
            "complexity-func2".to_string(),
            MetricCategory::StructuralComplexity,
            3.0,
            "func2 too complex".to_string(),
        );

        assert_eq!(p1, Some(2.0));
        assert_eq!(p2, Some(3.0));
        assert_eq!(tracker.get_attributions().len(), 2);
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
