//! Unified defect analyzer trait and implementations
//!
//! This module provides the trait and base implementations for analyzing
//! different types of defects in the codebase.

use crate::models::defect_report::{Defect, DefectCategory};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Configuration for defect analyzers
pub trait AnalyzerConfig: Default + Clone + Send + Sync {}

/// Base trait for all defect analyzers
#[async_trait]
pub trait DefectAnalyzer: Send + Sync {
    /// Configuration type for this analyzer
    type Config: AnalyzerConfig;

    /// Analyze the project and return defects
    async fn analyze(&self, project: &Path, config: Self::Config) -> Result<Vec<Defect>>;

    /// Get the category this analyzer handles
    fn category(&self) -> DefectCategory;

    /// Check if this analyzer supports incremental analysis
    fn supports_incremental(&self) -> bool {
        false
    }

    /// Get a human-readable name for this analyzer
    fn name(&self) -> &'static str {
        match self.category() {
            DefectCategory::Complexity => "Complexity Analyzer",
            DefectCategory::TechnicalDebt => "Technical Debt Analyzer",
            DefectCategory::DeadCode => "Dead Code Analyzer",
            DefectCategory::Duplication => "Duplication Analyzer",
            DefectCategory::Performance => "Performance Analyzer",
            DefectCategory::Architecture => "Architecture Analyzer",
            DefectCategory::TestCoverage => "Test Coverage Analyzer",
        }
    }
}

/// File ranking engine for prioritizing files by defect density
pub struct FileRankingEngine {
    scorer: Box<dyn FileScorer + Send + Sync>,
    cache: std::sync::Arc<dashmap::DashMap<std::path::PathBuf, f64>>,
}

/// Trait for scoring files based on defects
pub trait FileScorer: Send + Sync {
    /// Compute a score for a file based on its defects
    fn compute_score(&self, defects: &[Defect]) -> f64;
}

/// Simple scorer that uses defect count and severity
pub struct SimpleScorer;

impl FileScorer for SimpleScorer {
    fn compute_score(&self, defects: &[Defect]) -> f64 {
        defects.iter().map(|d| d.severity_weight()).sum()
    }
}

/// Ranked file with score and defects
#[derive(Debug, Clone)]
pub struct RankedFile {
    /// Rank position (1-based)
    pub rank: usize,
    /// Computed score
    pub score: f64,
    /// File path
    pub path: std::path::PathBuf,
    /// Defects in this file
    pub defects: Vec<Defect>,
}

impl FileRankingEngine {
    /// Create a new ranking engine with the given scorer
    pub fn new(scorer: Box<dyn FileScorer + Send + Sync>) -> Self {
        Self {
            scorer,
            cache: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Rank files by their defect scores
    pub fn rank_files(&self, defects: Vec<Defect>, limit: usize) -> Vec<RankedFile> {
        use rayon::prelude::*;
        use std::cmp::Ordering;
        use std::collections::BTreeMap;

        // Group defects by file
        let mut defects_by_file: BTreeMap<std::path::PathBuf, Vec<Defect>> = BTreeMap::new();
        for defect in defects {
            defects_by_file
                .entry(defect.file_path.clone())
                .or_default()
                .push(defect);
        }

        // Compute scores in parallel
        let mut scored: Vec<_> = defects_by_file
            .into_par_iter()
            .map(|(path, file_defects)| {
                let score = self.cache.get(&path).map(|s| *s).unwrap_or_else(|| {
                    let s = self.scorer.compute_score(&file_defects);
                    self.cache.insert(path.clone(), s);
                    s
                });

                RankedFile {
                    rank: 0, // Will be set after sorting
                    score,
                    path,
                    defects: file_defects,
                }
            })
            .collect();

        // Stable sort for deterministic output
        scored.par_sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path)) // Secondary sort by path
        });

        // Apply limit: 0 means all files
        let take_count = if limit == 0 {
            scored.len()
        } else {
            limit.min(scored.len())
        };
        scored.truncate(take_count);

        // Assign ranks
        for (i, file) in scored.iter_mut().enumerate() {
            file.rank = i + 1;
        }

        scored
    }
}

/// Analysis result with file context
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// File path relative to project root
    pub file_path: std::path::PathBuf,
    /// Absolute file path
    pub absolute_path: std::path::PathBuf,
    /// Line range of the analysis
    pub line_range: LineRange,
    /// Metrics collected
    pub metrics: std::collections::BTreeMap<String, MetricValue>,
    /// Additional context
    pub context: AnalysisContext,
}

/// Line range information
#[derive(Debug, Clone)]
pub struct LineRange {
    /// Starting line information
    pub start: LineInfo,
    /// Ending line information (if applicable)
    pub end: Option<LineInfo>,
}

/// Detailed line information
#[derive(Debug, Clone)]
pub struct LineInfo {
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
    /// Byte offset in file
    pub byte_offset: usize,
}

/// Metric value types
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    /// Integer metric
    Integer(i64),
    /// Floating point metric
    Float(f64),
    /// String metric
    String(String),
    /// Boolean metric
    Boolean(bool),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Integer(i) => write!(f, "{}", i),
            MetricValue::Float(fl) => write!(f, "{:.2}", fl),
            MetricValue::String(s) => write!(f, "{}", s),
            MetricValue::Boolean(b) => write!(f, "{}", b),
        }
    }
}

/// Analysis context information
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Human-readable description
    pub description: String,
    /// Function or class name (if applicable)
    pub entity_name: Option<String>,
    /// Entity type (function, class, module, etc.)
    pub entity_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::defect_report::{DefectCategory, Severity};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_simple_scorer() {
        let scorer = SimpleScorer;
        let defects = vec![
            Defect {
                id: "TEST-001".to_string(),
                severity: Severity::Critical,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("test.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
            Defect {
                id: "TEST-002".to_string(),
                severity: Severity::High,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("test.rs"),
                line_start: 10,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test 2".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
        ];

        assert_eq!(scorer.compute_score(&defects), 15.0); // 10.0 + 5.0
    }

    #[test]
    fn test_file_ranking_engine() {
        let engine = FileRankingEngine::new(Box::new(SimpleScorer));
        let defects = vec![
            Defect {
                id: "TEST-001".to_string(),
                severity: Severity::Critical,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file1.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
            Defect {
                id: "TEST-002".to_string(),
                severity: Severity::Low,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file2.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
        ];

        let ranked = engine.rank_files(defects, 0);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].path, PathBuf::from("file1.rs"));
        assert_eq!(ranked[0].rank, 1);
        assert_eq!(ranked[0].score, 10.0);
        assert_eq!(ranked[1].path, PathBuf::from("file2.rs"));
        assert_eq!(ranked[1].rank, 2);
        assert_eq!(ranked[1].score, 1.0);
    }

    #[test]
    fn test_file_ranking_with_limit() {
        let engine = FileRankingEngine::new(Box::new(SimpleScorer));
        let defects = vec![
            Defect {
                id: "TEST-001".to_string(),
                severity: Severity::High,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file1.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
            Defect {
                id: "TEST-002".to_string(),
                severity: Severity::Medium,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file2.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
            Defect {
                id: "TEST-003".to_string(),
                severity: Severity::Low,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file3.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
        ];

        let ranked = engine.rank_files(defects, 2);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].path, PathBuf::from("file1.rs"));
        assert_eq!(ranked[1].path, PathBuf::from("file2.rs"));
    }
}
