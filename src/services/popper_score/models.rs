//! Core data models for Popper Falsifiability Score v1.1
//!
//! Defines types for the 100-point normalized scoring system
//! with 6 categories following Karl Popper's falsifiability criterion.
//!
//! Academic Foundation: 31 peer-reviewed papers (2022-2025)

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

// ============================================================================
// PopperScore - Main Score Container
// ============================================================================

/// Comprehensive Popper Falsifiability Score (v1.1)
///
/// Total: 100 points normalized across 6 categories with gateway logic.
///
/// ## Falsifiability Gateway
///
/// If Category A (Falsifiability) scores below 15/25 (60%), the total
/// score is 0 regardless of other categories. This implements Popper's
/// demarcation criterion: unfalsifiable claims are not science.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperScore {
    /// Raw points earned (before normalization)
    pub raw_score: f64,

    /// Maximum available points (may be <100 if ML is N/A)
    pub max_available: f64,

    /// Normalized score (0-100%)
    pub normalized_score: f64,

    /// Letter grade (A+ to F)
    pub grade: PopperGrade,

    /// Whether the falsifiability gateway was passed
    pub gateway_passed: bool,

    /// Breakdown by category
    pub categories: PopperCategoryScores,

    /// Actionable recommendations
    pub recommendations: Vec<PopperRecommendation>,

    /// Metadata (timestamp, project, version)
    pub metadata: PopperMetadata,

    /// Popper analysis summary
    pub analysis: PopperAnalysis,
}

impl PopperScore {
    /// Create a new empty score
    pub fn new() -> Self {
        Self {
            raw_score: 0.0,
            max_available: 100.0,
            normalized_score: 0.0,
            grade: PopperGrade::F,
            gateway_passed: false,
            categories: PopperCategoryScores::default(),
            recommendations: Vec::new(),
            metadata: PopperMetadata::new("unknown".to_string()),
            analysis: PopperAnalysis::default(),
        }
    }

    /// Calculate normalized score with gateway logic
    ///
    /// ## Algorithm (v1.1)
    ///
    /// Phase 1: Gateway Check
    /// ```text
    /// IF Category_A < 15 THEN:
    ///     Total_Score = 0
    ///     Status = "INSUFFICIENT FALSIFIABILITY"
    /// ```
    ///
    /// Phase 2: Normalized Calculation
    /// ```text
    /// Normalized_Score = (Points_Earned / Points_Available) × 100
    /// ```
    pub fn calculate(&mut self) {
        // Check gateway
        let falsifiability_score = self.categories.falsifiability.earned;
        self.gateway_passed = falsifiability_score >= 15.0;

        if !self.gateway_passed {
            self.raw_score = 0.0;
            self.normalized_score = 0.0;
            self.grade = PopperGrade::InsufficientFalsifiability;
            return;
        }

        // Calculate raw score
        self.raw_score = self.categories.total_earned();
        self.max_available = self.categories.total_available();

        // Normalize to 100%
        if self.max_available > 0.0 {
            self.normalized_score = (self.raw_score / self.max_available) * 100.0;
        } else {
            self.normalized_score = 0.0;
        }

        // Assign grade
        self.grade = PopperGrade::from_normalized_score(self.normalized_score);
    }
}

impl Default for PopperScore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PopperGrade - Letter Grade with Special Case
// ============================================================================

/// Letter grade based on normalized percentage
///
/// Special case: InsufficientFalsifiability when gateway fails
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopperGrade {
    /// Exemplary Popperian Science (95-100%)
    APlus,
    /// Strong Scientific Standards (90-94%)
    A,
    /// Meets Reproducibility Requirements (85-89%)
    AMinus,
    /// Good Practices, Minor Gaps (80-84%)
    BPlus,
    /// Acceptable, Improvement Needed (70-79%)
    B,
    /// Significant Reproducibility Gaps (60-69%)
    C,
    /// Major Falsifiability Issues (50-59%)
    D,
    /// Insufficient Rigor for Independent Verification (0-49%)
    F,
    /// Gateway failed: Category A < 15/25
    InsufficientFalsifiability,
}

impl PopperGrade {
    /// Calculate grade from normalized score (0-100)
    pub fn from_normalized_score(score: f64) -> Self {
        if score >= 95.0 {
            PopperGrade::APlus
        } else if score >= 90.0 {
            PopperGrade::A
        } else if score >= 85.0 {
            PopperGrade::AMinus
        } else if score >= 80.0 {
            PopperGrade::BPlus
        } else if score >= 70.0 {
            PopperGrade::B
        } else if score >= 60.0 {
            PopperGrade::C
        } else if score >= 50.0 {
            PopperGrade::D
        } else {
            PopperGrade::F
        }
    }

    /// Check if grade meets Popperian scientific standards
    pub fn meets_standards(&self) -> bool {
        matches!(
            self,
            PopperGrade::APlus | PopperGrade::A | PopperGrade::AMinus
        )
    }

    /// Get interpretation text
    pub fn interpretation(&self) -> &'static str {
        match self {
            PopperGrade::APlus => "Exemplary Popperian Science",
            PopperGrade::A => "Strong Scientific Standards",
            PopperGrade::AMinus => "Meets Reproducibility Requirements",
            PopperGrade::BPlus => "Good Practices, Minor Gaps",
            PopperGrade::B => "Acceptable, Improvement Needed",
            PopperGrade::C => "Significant Reproducibility Gaps",
            PopperGrade::D => "Major Falsifiability Issues",
            PopperGrade::F => "Insufficient Rigor for Independent Verification",
            PopperGrade::InsufficientFalsifiability => "GATEWAY FAILED - Not Evaluable as Science",
        }
    }
}

impl fmt::Display for PopperGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PopperGrade::APlus => write!(f, "A+"),
            PopperGrade::A => write!(f, "A"),
            PopperGrade::AMinus => write!(f, "A-"),
            PopperGrade::BPlus => write!(f, "B+"),
            PopperGrade::B => write!(f, "B"),
            PopperGrade::C => write!(f, "C"),
            PopperGrade::D => write!(f, "D"),
            PopperGrade::F => write!(f, "F"),
            PopperGrade::InsufficientFalsifiability => write!(f, "GATEWAY FAILED"),
        }
    }
}

// ============================================================================
// PopperCategoryScores - 6 Scoring Categories
// ============================================================================

/// Six scoring categories (100 points total when all applicable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperCategoryScores {
    /// A. Falsifiability & Testability (25pts) - GATEWAY
    pub falsifiability: PopperCategoryScore,

    /// B. Reproducibility Infrastructure (25pts)
    pub reproducibility: PopperCategoryScore,

    /// C. Transparency & Openness (20pts)
    pub transparency: PopperCategoryScore,

    /// D. Statistical Rigor (15pts)
    pub statistical_rigor: PopperCategoryScore,

    /// E. Historical Integrity (10pts)
    pub historical_integrity: PopperCategoryScore,

    /// F. ML/AI Reproducibility (5pts or N/A)
    pub ml_reproducibility: PopperCategoryScore,
}

impl PopperCategoryScores {
    /// Calculate total earned points
    pub fn total_earned(&self) -> f64 {
        let mut total = self.falsifiability.earned
            + self.reproducibility.earned
            + self.transparency.earned
            + self.statistical_rigor.earned
            + self.historical_integrity.earned;

        // Only add ML if applicable
        if !self.ml_reproducibility.is_not_applicable {
            total += self.ml_reproducibility.earned;
        }

        total
    }

    /// Calculate total available points (may be <100 if ML is N/A)
    pub fn total_available(&self) -> f64 {
        let mut total = self.falsifiability.max
            + self.reproducibility.max
            + self.transparency.max
            + self.statistical_rigor.max
            + self.historical_integrity.max;

        // Only add ML if applicable
        if !self.ml_reproducibility.is_not_applicable {
            total += self.ml_reproducibility.max;
        }

        total
    }
}

impl Default for PopperCategoryScores {
    fn default() -> Self {
        Self {
            falsifiability: PopperCategoryScore::new("Falsifiability & Testability", 0.0, 25.0),
            reproducibility: PopperCategoryScore::new("Reproducibility Infrastructure", 0.0, 25.0),
            transparency: PopperCategoryScore::new("Transparency & Openness", 0.0, 20.0),
            statistical_rigor: PopperCategoryScore::new("Statistical Rigor", 0.0, 15.0),
            historical_integrity: PopperCategoryScore::new("Historical Integrity", 0.0, 10.0),
            ml_reproducibility: PopperCategoryScore::new_na("ML/AI Reproducibility", 5.0),
        }
    }
}

// ============================================================================
// PopperCategoryScore - Individual Category Metrics
// ============================================================================

/// Score for an individual category with sub-scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperCategoryScore {
    /// Category name
    pub name: String,

    /// Points earned
    pub earned: f64,

    /// Maximum possible points
    pub max: f64,

    /// Whether this category is applicable (true by default)
    /// Used for conditional categories like ML/AI Reproducibility
    pub is_applicable: bool,

    /// Whether this category is N/A (e.g., ML for non-ML projects)
    /// This is the inverse of is_applicable for backwards compatibility
    pub is_not_applicable: bool,

    /// Sub-scores within this category
    pub sub_scores: Vec<PopperSubScore>,

    /// Findings for this category
    pub findings: Vec<PopperFinding>,
}

impl PopperCategoryScore {
    /// Create a new category score
    pub fn new(name: &str, earned: f64, max: f64) -> Self {
        Self {
            name: name.to_string(),
            earned,
            max,
            is_applicable: true,
            is_not_applicable: false,
            sub_scores: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Create a N/A category score
    pub fn new_na(name: &str, max: f64) -> Self {
        Self {
            name: name.to_string(),
            earned: 0.0,
            max,
            is_applicable: false,
            is_not_applicable: true,
            sub_scores: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Calculate percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.is_not_applicable || self.max == 0.0 {
            0.0
        } else {
            (self.earned / self.max) * 100.0
        }
    }

    /// Mark as applicable (for ML projects)
    pub fn mark_applicable(&mut self) {
        self.is_applicable = true;
        self.is_not_applicable = false;
    }

    /// Mark as not applicable
    pub fn mark_not_applicable(&mut self) {
        self.is_applicable = false;
        self.is_not_applicable = true;
    }

    /// Add a sub-score
    pub fn add_sub_score(&mut self, sub: PopperSubScore) {
        self.earned += sub.earned;
        self.sub_scores.push(sub);
    }

    /// Add a finding
    pub fn add_finding(&mut self, finding: PopperFinding) {
        self.findings.push(finding);
    }
}

// ============================================================================
// PopperSubScore - Sub-category Breakdown
// ============================================================================

/// Sub-score within a category (e.g., A1, A2, A3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperSubScore {
    /// Sub-category identifier (e.g., "A1", "A2")
    pub id: String,

    /// Sub-category name
    pub name: String,

    /// Points earned
    pub earned: f64,

    /// Maximum possible points
    pub max: f64,

    /// Description of what was checked
    pub description: String,
}

impl PopperSubScore {
    /// Create a new sub-score
    pub fn new(id: &str, name: &str, earned: f64, max: f64, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            earned,
            max,
            description: description.to_string(),
        }
    }
}

// ============================================================================
// PopperFinding - Evidence Found During Analysis
// ============================================================================

/// Severity level for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    /// Positive finding (evidence of good practice)
    Positive,
    /// Informational (neutral observation)
    Info,
    /// Warning (potential issue)
    Warning,
    /// Critical (significant gap)
    Critical,
}

/// Finding discovered during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperFinding {
    /// Severity level
    pub severity: FindingSeverity,

    /// Short description
    pub message: String,

    /// Location (file path, if applicable)
    pub location: Option<PathBuf>,

    /// Impact on score (points affected)
    pub impact: f64,
}

impl PopperFinding {
    /// Create a positive finding
    pub fn positive(message: &str) -> Self {
        Self {
            severity: FindingSeverity::Positive,
            message: message.to_string(),
            location: None,
            impact: 0.0,
        }
    }

    /// Create an informational finding
    pub fn info(message: &str) -> Self {
        Self {
            severity: FindingSeverity::Info,
            message: message.to_string(),
            location: None,
            impact: 0.0,
        }
    }

    /// Create a warning finding
    pub fn warning(message: &str, impact: f64) -> Self {
        Self {
            severity: FindingSeverity::Warning,
            message: message.to_string(),
            location: None,
            impact,
        }
    }

    /// Create a critical finding
    pub fn critical(message: &str, impact: f64) -> Self {
        Self {
            severity: FindingSeverity::Critical,
            message: message.to_string(),
            location: None,
            impact,
        }
    }
}

// ============================================================================
// PopperRecommendation - Actionable Improvements
// ============================================================================

/// Priority level for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Actionable recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperRecommendation {
    /// Category this recommendation applies to
    pub category: String,

    /// Description of the recommendation
    pub description: String,

    /// Priority level
    pub priority: RecommendationPriority,

    /// Potential normalized percentage points to gain
    pub potential_percent: f64,

    /// Command to run (if applicable)
    pub command: Option<String>,
}

impl PopperRecommendation {
    /// Create a new recommendation
    pub fn new(
        category: &str,
        description: &str,
        priority: RecommendationPriority,
        potential_percent: f64,
    ) -> Self {
        Self {
            category: category.to_string(),
            description: description.to_string(),
            priority,
            potential_percent,
            command: None,
        }
    }

    /// Add command to recommendation
    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }
}

// ============================================================================
// PopperAnalysis - Summary of Popperian Analysis
// ============================================================================

/// Summary of Popperian analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopperAnalysis {
    /// Falsifiability: Claims are testable and potentially refutable
    pub falsifiability_status: AnalysisStatus,

    /// Reproducibility: Independent verification is possible
    pub reproducibility_status: AnalysisStatus,

    /// Scrutiny: Documentation enables full scrutiny
    pub scrutiny_status: AnalysisStatus,

    /// Methodology: Statistical practices are sound
    pub methodology_status: AnalysisStatus,

    /// Validation: External replication evidence
    pub validation_status: AnalysisStatus,

    /// Overall verdict
    pub verdict: String,
}

/// Status for each analysis dimension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnalysisStatus {
    /// Fully meets standards
    Pass,
    /// Partially meets standards
    Partial,
    /// Does not meet standards
    #[default]
    Fail,
}

impl fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisStatus::Pass => write!(f, "PASS"),
            AnalysisStatus::Partial => write!(f, "PARTIAL"),
            AnalysisStatus::Fail => write!(f, "FAIL"),
        }
    }
}

// ============================================================================
// PopperMetadata - Project Information
// ============================================================================

/// Metadata about the scoring analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperMetadata {
    /// Timestamp of analysis
    pub timestamp: String,

    /// Project name
    pub project_name: String,

    /// Specification version
    pub version: String,

    /// Project path analyzed
    pub project_path: Option<PathBuf>,
}

impl PopperMetadata {
    /// Create new metadata
    pub fn new(project_name: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            project_name,
            version: "1.1.0".to_string(),
            project_path: None,
        }
    }

    /// Set project path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // RED TESTS - Falsifiability Gateway (Annotation 2)
    // ========================================================================

    #[test]
    fn test_gateway_fails_when_falsifiability_below_15() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 14.0; // Below threshold
        score.categories.reproducibility.earned = 25.0;
        score.categories.transparency.earned = 20.0;
        score.categories.statistical_rigor.earned = 15.0;
        score.categories.historical_integrity.earned = 10.0;

        score.calculate();

        assert!(!score.gateway_passed);
        assert_eq!(score.normalized_score, 0.0);
        assert_eq!(score.grade, PopperGrade::InsufficientFalsifiability);
    }

    #[test]
    fn test_gateway_passes_when_falsifiability_at_15() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 15.0; // At threshold
        score.categories.reproducibility.earned = 20.0;
        score.categories.transparency.earned = 15.0;
        score.categories.statistical_rigor.earned = 10.0;
        score.categories.historical_integrity.earned = 8.0;

        score.calculate();

        assert!(score.gateway_passed);
        assert!(score.normalized_score > 0.0);
    }

    #[test]
    fn test_gateway_passes_when_falsifiability_above_15() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 20.0;
        score.categories.reproducibility.earned = 20.0;
        score.categories.transparency.earned = 15.0;
        score.categories.statistical_rigor.earned = 10.0;
        score.categories.historical_integrity.earned = 8.0;

        score.calculate();

        assert!(score.gateway_passed);
        assert!(score.normalized_score > 70.0);
    }

    // ========================================================================
    // RED TESTS - Score Normalization (Annotation 8)
    // ========================================================================

    #[test]
    fn test_normalization_without_ml() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 22.0;
        score.categories.reproducibility.earned = 23.0;
        score.categories.transparency.earned = 17.0;
        score.categories.statistical_rigor.earned = 12.0;
        score.categories.historical_integrity.earned = 8.0;
        // ML is N/A by default

        score.calculate();

        // Expected: 82/95 = 86.3%
        assert!(score.gateway_passed);
        assert_eq!(score.max_available, 95.0); // 100 - 5 for N/A ML
        assert!((score.normalized_score - 86.3).abs() < 0.5);
        assert_eq!(score.grade, PopperGrade::AMinus);
    }

    #[test]
    fn test_normalization_with_ml() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 22.0;
        score.categories.reproducibility.earned = 23.0;
        score.categories.transparency.earned = 17.0;
        score.categories.statistical_rigor.earned = 12.0;
        score.categories.historical_integrity.earned = 8.0;
        score.categories.ml_reproducibility.mark_applicable();
        score.categories.ml_reproducibility.earned = 4.0;

        score.calculate();

        // Expected: 86/100 = 86%
        assert!(score.gateway_passed);
        assert_eq!(score.max_available, 100.0);
        assert_eq!(score.normalized_score, 86.0);
        assert_eq!(score.grade, PopperGrade::AMinus);
    }

    // ========================================================================
    // RED TESTS - Grade Assignment (Annotation 9)
    // ========================================================================

    #[test]
    fn test_grade_thresholds() {
        assert_eq!(
            PopperGrade::from_normalized_score(100.0),
            PopperGrade::APlus
        );
        assert_eq!(PopperGrade::from_normalized_score(95.0), PopperGrade::APlus);
        assert_eq!(PopperGrade::from_normalized_score(94.9), PopperGrade::A);
        assert_eq!(PopperGrade::from_normalized_score(90.0), PopperGrade::A);
        assert_eq!(
            PopperGrade::from_normalized_score(89.9),
            PopperGrade::AMinus
        );
        assert_eq!(
            PopperGrade::from_normalized_score(85.0),
            PopperGrade::AMinus
        );
        assert_eq!(PopperGrade::from_normalized_score(84.9), PopperGrade::BPlus);
        assert_eq!(PopperGrade::from_normalized_score(80.0), PopperGrade::BPlus);
        assert_eq!(PopperGrade::from_normalized_score(79.9), PopperGrade::B);
        assert_eq!(PopperGrade::from_normalized_score(70.0), PopperGrade::B);
        assert_eq!(PopperGrade::from_normalized_score(69.9), PopperGrade::C);
        assert_eq!(PopperGrade::from_normalized_score(60.0), PopperGrade::C);
        assert_eq!(PopperGrade::from_normalized_score(59.9), PopperGrade::D);
        assert_eq!(PopperGrade::from_normalized_score(50.0), PopperGrade::D);
        assert_eq!(PopperGrade::from_normalized_score(49.9), PopperGrade::F);
        assert_eq!(PopperGrade::from_normalized_score(0.0), PopperGrade::F);
    }

    #[test]
    fn test_grade_meets_standards() {
        assert!(PopperGrade::APlus.meets_standards());
        assert!(PopperGrade::A.meets_standards());
        assert!(PopperGrade::AMinus.meets_standards());
        assert!(!PopperGrade::BPlus.meets_standards());
        assert!(!PopperGrade::B.meets_standards());
        assert!(!PopperGrade::C.meets_standards());
        assert!(!PopperGrade::D.meets_standards());
        assert!(!PopperGrade::F.meets_standards());
        assert!(!PopperGrade::InsufficientFalsifiability.meets_standards());
    }

    #[test]
    fn test_grade_interpretation_not_science_removed() {
        // Verify we use "Insufficient Rigor" not "NOT SCIENCE" (Annotation 9)
        let f_interpretation = PopperGrade::F.interpretation();
        assert!(!f_interpretation.contains("NOT SCIENCE"));
        assert!(f_interpretation.contains("Insufficient Rigor"));
    }

    // ========================================================================
    // RED TESTS - Category Scores
    // ========================================================================

    #[test]
    fn test_category_total_excludes_na() {
        let scores = PopperCategoryScores::default();
        // Default has ML as N/A, so max should be 95
        assert_eq!(scores.total_available(), 95.0);
    }

    #[test]
    fn test_category_total_includes_ml_when_applicable() {
        let mut scores = PopperCategoryScores::default();
        scores.ml_reproducibility.mark_applicable();
        assert_eq!(scores.total_available(), 100.0);
    }

    #[test]
    fn test_sub_score_accumulation() {
        let mut category = PopperCategoryScore::new("Test", 0.0, 25.0);

        category.add_sub_score(PopperSubScore::new("T1", "Test 1", 5.0, 8.0, "First test"));
        category.add_sub_score(PopperSubScore::new(
            "T2",
            "Test 2",
            7.0,
            10.0,
            "Second test",
        ));

        assert_eq!(category.earned, 12.0);
        assert_eq!(category.sub_scores.len(), 2);
    }

    // ========================================================================
    // RED TESTS - Findings
    // ========================================================================

    #[test]
    fn test_finding_creation() {
        let positive = PopperFinding::positive("Good test coverage");
        assert_eq!(positive.severity, FindingSeverity::Positive);

        let warning = PopperFinding::warning("Missing documentation", 2.0);
        assert_eq!(warning.severity, FindingSeverity::Warning);
        assert_eq!(warning.impact, 2.0);

        let critical = PopperFinding::critical("No tests", 10.0);
        assert_eq!(critical.severity, FindingSeverity::Critical);
        assert_eq!(critical.impact, 10.0);
    }

    // ========================================================================
    // RED TESTS - Recommendations
    // ========================================================================

    #[test]
    fn test_recommendation_with_command() {
        let rec = PopperRecommendation::new(
            "Testing",
            "Add mutation testing",
            RecommendationPriority::High,
            5.0,
        )
        .with_command("cargo mutants");

        assert_eq!(rec.command, Some("cargo mutants".to_string()));
    }
}
