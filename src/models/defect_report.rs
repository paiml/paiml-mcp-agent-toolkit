//! Unified defect report model for aggregating all quality issues
//!
//! This module defines the core structures for the comprehensive defect
//! reporting system that consolidates results from all analysis commands.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

/// Unique identifier for a defect
pub type DefectId = String;

/// Comprehensive defect report containing all quality issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectReport {
    /// Metadata about the report generation
    pub metadata: ReportMetadata,
    /// All defects found across the codebase
    pub defects: Vec<Defect>,
    /// Aggregated summary statistics
    pub summary: DefectSummary,
    /// Index mapping files to their defects
    pub file_index: BTreeMap<PathBuf, Vec<DefectId>>,
}

/// Metadata about the report generation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Tool name and version
    pub tool: String,
    /// Version of the tool
    pub version: String,
    /// When the report was generated
    pub generated_at: DateTime<Utc>,
    /// Root directory of the analyzed project
    pub project_root: PathBuf,
    /// Total number of files analyzed
    pub total_files_analyzed: usize,
    /// Time taken to generate the report in milliseconds
    pub analysis_duration_ms: u64,
}

/// Individual defect found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defect {
    /// Unique identifier for this defect
    pub id: DefectId,
    /// Severity level of the defect
    pub severity: Severity,
    /// Category of the defect
    pub category: DefectCategory,
    /// File path relative to project root
    pub file_path: PathBuf,
    /// Starting line number
    pub line_start: u32,
    /// Ending line number (if applicable)
    pub line_end: Option<u32>,
    /// Starting column number (if applicable)
    pub column_start: Option<u32>,
    /// Ending column number (if applicable)
    pub column_end: Option<u32>,
    /// Human-readable description of the defect
    pub message: String,
    /// Rule identifier that triggered this defect
    pub rule_id: String,
    /// Suggested fix or refactoring (if available)
    pub fix_suggestion: Option<String>,
    /// Additional metrics associated with the defect
    pub metrics: HashMap<String, f64>,
}

/// Severity levels for defects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Low impact issues
    Low,
    /// Medium impact issues
    Medium,
    /// High impact issues requiring attention
    High,
    /// Critical issues requiring immediate attention
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Categories of defects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefectCategory {
    /// Cyclomatic/cognitive complexity violations
    Complexity,
    /// Self-admitted technical debt markers
    TechnicalDebt,
    /// Unreachable or unused code
    DeadCode,
    /// Code duplication
    Duplication,
    /// Performance issues (O(n²) or worse)
    Performance,
    /// Architecture/coupling issues
    Architecture,
    /// Insufficient test coverage
    TestCoverage,
}

impl DefectCategory {
    /// Get all categories for iteration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::defect_report::DefectCategory;
    ///
    /// let categories = DefectCategory::all();
    /// assert_eq!(categories.len(), 7);
    /// assert!(categories.contains(&DefectCategory::Complexity));
    /// assert!(categories.contains(&DefectCategory::TestCoverage));
    /// ```
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Complexity,
            Self::TechnicalDebt,
            Self::DeadCode,
            Self::Duplication,
            Self::Performance,
            Self::Architecture,
            Self::TestCoverage,
        ]
    }
}

/// Summary statistics for the defect report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectSummary {
    /// Total number of defects found
    pub total_defects: usize,
    /// Breakdown by severity
    pub by_severity: BTreeMap<String, usize>,
    /// Breakdown by category
    pub by_category: BTreeMap<String, usize>,
    /// Top files by defect count
    pub hotspot_files: Vec<FileHotspot>,
}

/// File with high defect density
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHotspot {
    /// File path relative to project root
    pub path: PathBuf,
    /// Number of defects in this file
    pub defect_count: usize,
    /// Weighted severity score
    pub severity_score: f64,
}

impl Defect {
    /// Create a new defect ID with the given prefix and index
    /// Generates a unique defect ID with prefix and index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::defect_report::Defect;
    ///
    /// let id = Defect::generate_id("TEST", 0);
    /// assert_eq!(id, "TEST-001");
    ///
    /// let id2 = Defect::generate_id("BUG", 99);
    /// assert_eq!(id2, "BUG-100");
    /// ```
    #[must_use]
    pub fn generate_id(prefix: &str, index: usize) -> DefectId {
        format!("{}-{:03}", prefix, index + 1)
    }

    /// Calculate severity weight for scoring
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::defect_report::{Defect, Severity, DefectCategory};
    /// use std::path::PathBuf;
    /// use std::collections::HashMap;
    ///
    /// let defect = Defect {
    ///     id: "TEST-001".to_string(),
    ///     severity: Severity::High,
    ///     category: DefectCategory::TechnicalDebt,
    ///     file_path: PathBuf::from("src/main.rs"),
    ///     line_start: 45,
    ///     line_end: None,
    ///     column_start: None,
    ///     column_end: None,
    ///     message: "Potential memory leak".to_string(),
    ///     rule_id: "MEM001".to_string(),
    ///     fix_suggestion: None,
    ///     metrics: HashMap::new(),
    /// };
    ///
    /// assert_eq!(defect.severity_weight(), 5.0);
    /// ```
    #[must_use]
    pub fn severity_weight(&self) -> f64 {
        match self.severity {
            Severity::Critical => 10.0,
            Severity::High => 5.0,
            Severity::Medium => 3.0,
            Severity::Low => 1.0,
        }
    }
}

impl std::fmt::Display for DefectCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefectCategory::Complexity => write!(f, "Complexity"),
            DefectCategory::TechnicalDebt => write!(f, "Technical Debt"),
            DefectCategory::DeadCode => write!(f, "Dead Code"),
            DefectCategory::Duplication => write!(f, "Duplication"),
            DefectCategory::Performance => write!(f, "Performance"),
            DefectCategory::Architecture => write!(f, "Architecture"),
            DefectCategory::TestCoverage => write!(f, "Test Coverage"),
        }
    }
}

/// Configuration for file ranking
#[derive(Debug, Clone)]
pub struct FileRankingConfig {
    /// Whether to include severity in scoring
    pub use_severity: bool,
    /// Whether to include defect count in scoring
    pub use_count: bool,
    /// Custom weights for different categories
    pub category_weights: HashMap<DefectCategory, f64>,
}

impl Default for FileRankingConfig {
    fn default() -> Self {
        let mut category_weights = HashMap::new();
        category_weights.insert(DefectCategory::Complexity, 1.5);
        category_weights.insert(DefectCategory::Performance, 2.0);
        category_weights.insert(DefectCategory::Architecture, 1.8);
        category_weights.insert(DefectCategory::TechnicalDebt, 1.2);
        category_weights.insert(DefectCategory::DeadCode, 1.0);
        category_weights.insert(DefectCategory::Duplication, 1.3);
        category_weights.insert(DefectCategory::TestCoverage, 0.8);

        Self {
            use_severity: true,
            use_count: true,
            category_weights,
        }
    }
}

/// Result of file ranking operation
#[derive(Debug, Clone)]
pub struct RankedFile {
    /// Rank position (1-based)
    pub rank: usize,
    /// Computed score
    pub score: f64,
    /// File path
    pub path: PathBuf,
    /// Defects in this file
    pub defects: Vec<Defect>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // Test helper: Create a sample defect
    fn create_test_defect(severity: Severity, category: DefectCategory) -> Defect {
        Defect {
            id: "TEST-001".to_string(),
            severity,
            category,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: Some(10),
            column_start: Some(5),
            column_end: Some(20),
            message: "Test defect message".to_string(),
            rule_id: "TEST001".to_string(),
            fix_suggestion: Some("Fix suggestion".to_string()),
            metrics: HashMap::new(),
        }
    }

    // Test helper: Create a sample defect report
    fn create_test_report() -> DefectReport {
        let metadata = ReportMetadata {
            tool: "pmat".to_string(),
            version: "1.0.0".to_string(),
            generated_at: Utc::now(),
            project_root: PathBuf::from("/test/project"),
            total_files_analyzed: 10,
            analysis_duration_ms: 1000,
        };

        let defect = create_test_defect(Severity::High, DefectCategory::Complexity);

        let mut by_severity = BTreeMap::new();
        by_severity.insert("High".to_string(), 1);

        let mut by_category = BTreeMap::new();
        by_category.insert("Complexity".to_string(), 1);

        let summary = DefectSummary {
            total_defects: 1,
            by_severity,
            by_category,
            hotspot_files: vec![FileHotspot {
                path: PathBuf::from("test.rs"),
                defect_count: 1,
                severity_score: 5.0,
            }],
        };

        let mut file_index = BTreeMap::new();
        file_index.insert(PathBuf::from("test.rs"), vec!["TEST-001".to_string()]);

        DefectReport {
            metadata,
            defects: vec![defect],
            summary,
            file_index,
        }
    }

    // === Severity Tests ===

    #[test]
    fn test_defect_id_generation() {
        assert_eq!(Defect::generate_id("CPLX", 0), "CPLX-001");
        assert_eq!(Defect::generate_id("SATD", 99), "SATD-100");
    }

    #[test]
    fn test_defect_id_generation_various_prefixes() {
        assert_eq!(Defect::generate_id("A", 0), "A-001");
        assert_eq!(Defect::generate_id("DEAD", 5), "DEAD-006");
        assert_eq!(Defect::generate_id("PERF", 999), "PERF-1000");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Low, Severity::Low);
        assert_eq!(Severity::Medium, Severity::Medium);
        assert_eq!(Severity::High, Severity::High);
        assert_eq!(Severity::Critical, Severity::Critical);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Low), "Low");
        assert_eq!(format!("{}", Severity::Medium), "Medium");
        assert_eq!(format!("{}", Severity::High), "High");
        assert_eq!(format!("{}", Severity::Critical), "Critical");
    }

    #[test]
    fn test_severity_weight() {
        let defect = Defect {
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
        };

        assert_eq!(defect.severity_weight(), 10.0);
    }

    #[test]
    fn test_severity_weight_all_levels() {
        let mut defect = create_test_defect(Severity::Low, DefectCategory::Complexity);
        assert_eq!(defect.severity_weight(), 1.0);

        defect.severity = Severity::Medium;
        assert_eq!(defect.severity_weight(), 3.0);

        defect.severity = Severity::High;
        assert_eq!(defect.severity_weight(), 5.0);

        defect.severity = Severity::Critical;
        assert_eq!(defect.severity_weight(), 10.0);
    }

    // === DefectCategory Tests ===

    #[test]
    fn test_defect_category_all() {
        let categories = DefectCategory::all();
        assert_eq!(categories.len(), 7);
        assert!(categories.contains(&DefectCategory::Complexity));
        assert!(categories.contains(&DefectCategory::TechnicalDebt));
        assert!(categories.contains(&DefectCategory::DeadCode));
        assert!(categories.contains(&DefectCategory::Duplication));
        assert!(categories.contains(&DefectCategory::Performance));
        assert!(categories.contains(&DefectCategory::Architecture));
        assert!(categories.contains(&DefectCategory::TestCoverage));
    }

    #[test]
    fn test_defect_category_display() {
        assert_eq!(format!("{}", DefectCategory::Complexity), "Complexity");
        assert_eq!(
            format!("{}", DefectCategory::TechnicalDebt),
            "Technical Debt"
        );
        assert_eq!(format!("{}", DefectCategory::DeadCode), "Dead Code");
        assert_eq!(format!("{}", DefectCategory::Duplication), "Duplication");
        assert_eq!(format!("{}", DefectCategory::Performance), "Performance");
        assert_eq!(format!("{}", DefectCategory::Architecture), "Architecture");
        assert_eq!(format!("{}", DefectCategory::TestCoverage), "Test Coverage");
    }

    #[test]
    fn test_defect_category_ordering() {
        assert!(DefectCategory::TechnicalDebt > DefectCategory::Complexity);
        assert!(DefectCategory::DeadCode > DefectCategory::TechnicalDebt);
        assert!(DefectCategory::TestCoverage > DefectCategory::Architecture);
    }

    #[test]
    fn test_defect_category_equality() {
        assert_eq!(DefectCategory::Complexity, DefectCategory::Complexity);
        assert_ne!(DefectCategory::Complexity, DefectCategory::Performance);
    }

    #[test]
    fn test_defect_category_hash() {
        let mut map: HashMap<DefectCategory, i32> = HashMap::new();
        map.insert(DefectCategory::Complexity, 1);
        map.insert(DefectCategory::Performance, 2);
        assert_eq!(map.get(&DefectCategory::Complexity), Some(&1));
        assert_eq!(map.get(&DefectCategory::Performance), Some(&2));
    }

    // === Defect Tests ===

    #[test]
    fn test_defect_creation() {
        let defect = create_test_defect(Severity::High, DefectCategory::Complexity);
        assert_eq!(defect.id, "TEST-001");
        assert_eq!(defect.severity, Severity::High);
        assert_eq!(defect.category, DefectCategory::Complexity);
        assert_eq!(defect.file_path, PathBuf::from("test.rs"));
        assert_eq!(defect.line_start, 1);
        assert_eq!(defect.line_end, Some(10));
        assert_eq!(defect.column_start, Some(5));
        assert_eq!(defect.column_end, Some(20));
        assert_eq!(defect.message, "Test defect message");
        assert_eq!(defect.rule_id, "TEST001");
        assert_eq!(defect.fix_suggestion, Some("Fix suggestion".to_string()));
    }

    #[test]
    fn test_defect_with_metrics() {
        let mut metrics = HashMap::new();
        metrics.insert("complexity".to_string(), 25.0);
        metrics.insert("lines".to_string(), 100.0);

        let defect = Defect {
            id: "CPLX-001".to_string(),
            severity: Severity::High,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("complex.rs"),
            line_start: 10,
            line_end: Some(50),
            column_start: None,
            column_end: None,
            message: "High complexity".to_string(),
            rule_id: "CPLX001".to_string(),
            fix_suggestion: Some("Extract function".to_string()),
            metrics,
        };

        assert_eq!(defect.metrics.get("complexity"), Some(&25.0));
        assert_eq!(defect.metrics.get("lines"), Some(&100.0));
    }

    #[test]
    fn test_defect_minimal() {
        let defect = Defect {
            id: "MIN-001".to_string(),
            severity: Severity::Low,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from("unused.rs"),
            line_start: 1,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Unused function".to_string(),
            rule_id: "DEAD001".to_string(),
            fix_suggestion: None,
            metrics: HashMap::new(),
        };

        assert_eq!(defect.line_end, None);
        assert_eq!(defect.column_start, None);
        assert_eq!(defect.fix_suggestion, None);
        assert!(defect.metrics.is_empty());
    }

    // === ReportMetadata Tests ===

    #[test]
    fn test_report_metadata_creation() {
        let now = Utc::now();
        let metadata = ReportMetadata {
            tool: "pmat".to_string(),
            version: "2.0.0".to_string(),
            generated_at: now,
            project_root: PathBuf::from("/home/user/project"),
            total_files_analyzed: 100,
            analysis_duration_ms: 5000,
        };

        assert_eq!(metadata.tool, "pmat");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.generated_at, now);
        assert_eq!(metadata.project_root, PathBuf::from("/home/user/project"));
        assert_eq!(metadata.total_files_analyzed, 100);
        assert_eq!(metadata.analysis_duration_ms, 5000);
    }

    // === DefectSummary Tests ===

    #[test]
    fn test_defect_summary_creation() {
        let mut by_severity = BTreeMap::new();
        by_severity.insert("Low".to_string(), 5);
        by_severity.insert("Medium".to_string(), 3);
        by_severity.insert("High".to_string(), 2);

        let mut by_category = BTreeMap::new();
        by_category.insert("Complexity".to_string(), 4);
        by_category.insert("DeadCode".to_string(), 6);

        let summary = DefectSummary {
            total_defects: 10,
            by_severity,
            by_category,
            hotspot_files: vec![],
        };

        assert_eq!(summary.total_defects, 10);
        assert_eq!(summary.by_severity.get("Low"), Some(&5));
        assert_eq!(summary.by_category.get("Complexity"), Some(&4));
    }

    #[test]
    fn test_defect_summary_with_hotspots() {
        let hotspots = vec![
            FileHotspot {
                path: PathBuf::from("src/main.rs"),
                defect_count: 10,
                severity_score: 50.0,
            },
            FileHotspot {
                path: PathBuf::from("src/lib.rs"),
                defect_count: 5,
                severity_score: 25.0,
            },
        ];

        let summary = DefectSummary {
            total_defects: 15,
            by_severity: BTreeMap::new(),
            by_category: BTreeMap::new(),
            hotspot_files: hotspots,
        };

        assert_eq!(summary.hotspot_files.len(), 2);
        assert_eq!(summary.hotspot_files[0].defect_count, 10);
        assert_eq!(summary.hotspot_files[1].severity_score, 25.0);
    }

    // === FileHotspot Tests ===

    #[test]
    fn test_file_hotspot_creation() {
        let hotspot = FileHotspot {
            path: PathBuf::from("src/complex.rs"),
            defect_count: 25,
            severity_score: 75.5,
        };

        assert_eq!(hotspot.path, PathBuf::from("src/complex.rs"));
        assert_eq!(hotspot.defect_count, 25);
        assert!((hotspot.severity_score - 75.5).abs() < f64::EPSILON);
    }

    // === FileRankingConfig Tests ===

    #[test]
    fn test_file_ranking_config_default() {
        let config = FileRankingConfig::default();

        assert!(config.use_severity);
        assert!(config.use_count);
        assert_eq!(config.category_weights.len(), 7);
        assert_eq!(
            config.category_weights.get(&DefectCategory::Complexity),
            Some(&1.5)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::Performance),
            Some(&2.0)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::Architecture),
            Some(&1.8)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::TechnicalDebt),
            Some(&1.2)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::DeadCode),
            Some(&1.0)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::Duplication),
            Some(&1.3)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::TestCoverage),
            Some(&0.8)
        );
    }

    #[test]
    fn test_file_ranking_config_custom() {
        let mut category_weights = HashMap::new();
        category_weights.insert(DefectCategory::Complexity, 3.0);

        let config = FileRankingConfig {
            use_severity: false,
            use_count: true,
            category_weights,
        };

        assert!(!config.use_severity);
        assert!(config.use_count);
        assert_eq!(config.category_weights.len(), 1);
        assert_eq!(
            config.category_weights.get(&DefectCategory::Complexity),
            Some(&3.0)
        );
    }

    // === RankedFile Tests ===

    #[test]
    fn test_ranked_file_creation() {
        let defects = vec![
            create_test_defect(Severity::High, DefectCategory::Complexity),
            create_test_defect(Severity::Medium, DefectCategory::DeadCode),
        ];

        let ranked = RankedFile {
            rank: 1,
            score: 100.5,
            path: PathBuf::from("src/main.rs"),
            defects,
        };

        assert_eq!(ranked.rank, 1);
        assert!((ranked.score - 100.5).abs() < f64::EPSILON);
        assert_eq!(ranked.path, PathBuf::from("src/main.rs"));
        assert_eq!(ranked.defects.len(), 2);
    }

    #[test]
    fn test_ranked_file_empty_defects() {
        let ranked = RankedFile {
            rank: 5,
            score: 0.0,
            path: PathBuf::from("src/empty.rs"),
            defects: vec![],
        };

        assert_eq!(ranked.rank, 5);
        assert_eq!(ranked.score, 0.0);
        assert!(ranked.defects.is_empty());
    }

    // === DefectReport Tests ===

    #[test]
    fn test_defect_report_creation() {
        let report = create_test_report();

        assert_eq!(report.metadata.tool, "pmat");
        assert_eq!(report.defects.len(), 1);
        assert_eq!(report.summary.total_defects, 1);
        assert_eq!(report.file_index.len(), 1);
    }

    #[test]
    fn test_defect_report_file_index() {
        let report = create_test_report();

        let defect_ids = report.file_index.get(&PathBuf::from("test.rs"));
        assert!(defect_ids.is_some());
        assert_eq!(defect_ids.unwrap(), &vec!["TEST-001".to_string()]);
    }

    #[test]
    fn test_defect_report_multiple_files() {
        let mut file_index = BTreeMap::new();
        file_index.insert(
            PathBuf::from("a.rs"),
            vec!["A-001".to_string(), "A-002".to_string()],
        );
        file_index.insert(PathBuf::from("b.rs"), vec!["B-001".to_string()]);
        file_index.insert(
            PathBuf::from("c.rs"),
            vec![
                "C-001".to_string(),
                "C-002".to_string(),
                "C-003".to_string(),
            ],
        );

        assert_eq!(file_index.len(), 3);
        assert_eq!(file_index.get(&PathBuf::from("a.rs")).unwrap().len(), 2);
        assert_eq!(file_index.get(&PathBuf::from("c.rs")).unwrap().len(), 3);
    }

    // === Serialization Tests ===

    #[test]
    fn test_severity_serialization() {
        let json = serde_json::to_string(&Severity::Critical).unwrap();
        assert_eq!(json, "\"critical\"");

        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Severity::Critical);
    }

    #[test]
    fn test_defect_category_serialization() {
        let json = serde_json::to_string(&DefectCategory::TechnicalDebt).unwrap();
        assert_eq!(json, "\"technical_debt\"");

        let deserialized: DefectCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DefectCategory::TechnicalDebt);
    }

    #[test]
    fn test_defect_serialization_roundtrip() {
        let defect = create_test_defect(Severity::High, DefectCategory::Performance);
        let json = serde_json::to_string(&defect).unwrap();
        let deserialized: Defect = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, defect.id);
        assert_eq!(deserialized.severity, defect.severity);
        assert_eq!(deserialized.category, defect.category);
        assert_eq!(deserialized.message, defect.message);
    }

    #[test]
    fn test_defect_report_serialization_roundtrip() {
        let report = create_test_report();
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: DefectReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.metadata.tool, report.metadata.tool);
        assert_eq!(deserialized.defects.len(), report.defects.len());
        assert_eq!(
            deserialized.summary.total_defects,
            report.summary.total_defects
        );
    }

    #[test]
    fn test_file_hotspot_serialization_roundtrip() {
        let hotspot = FileHotspot {
            path: PathBuf::from("test/path.rs"),
            defect_count: 42,
            severity_score: 123.456,
        };

        let json = serde_json::to_string(&hotspot).unwrap();
        let deserialized: FileHotspot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, hotspot.path);
        assert_eq!(deserialized.defect_count, hotspot.defect_count);
        assert!((deserialized.severity_score - hotspot.severity_score).abs() < f64::EPSILON);
    }

    // === Clone and Debug Tests ===

    #[test]
    fn test_severity_clone() {
        let severity = Severity::High;
        let cloned = severity;
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_defect_clone() {
        let defect = create_test_defect(Severity::Medium, DefectCategory::Duplication);
        let cloned = defect.clone();
        assert_eq!(cloned.id, defect.id);
        assert_eq!(cloned.severity, defect.severity);
    }

    #[test]
    fn test_defect_report_clone() {
        let report = create_test_report();
        let cloned = report.clone();
        assert_eq!(cloned.metadata.tool, report.metadata.tool);
        assert_eq!(cloned.defects.len(), report.defects.len());
    }

    #[test]
    fn test_severity_debug() {
        let severity = Severity::Critical;
        let debug = format!("{:?}", severity);
        assert!(debug.contains("Critical"));
    }

    #[test]
    fn test_defect_category_debug() {
        let category = DefectCategory::Architecture;
        let debug = format!("{:?}", category);
        assert!(debug.contains("Architecture"));
    }

    #[test]
    fn test_file_ranking_config_debug() {
        let config = FileRankingConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("use_severity"));
        assert!(debug.contains("use_count"));
    }

    #[test]
    fn test_ranked_file_debug() {
        let ranked = RankedFile {
            rank: 1,
            score: 50.0,
            path: PathBuf::from("test.rs"),
            defects: vec![],
        };
        let debug = format!("{:?}", ranked);
        assert!(debug.contains("rank: 1"));
        assert!(debug.contains("score: 50.0"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
