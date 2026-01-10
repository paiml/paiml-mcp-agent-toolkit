// Data models for Five Whys root cause analysis
//
// GREEN PHASE: Minimal implementation to make tests pass

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete Five Whys analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugAnalysis {
    pub issue: String,
    pub whys: Vec<WhyIteration>,
    pub root_cause: Option<String>,
    pub recommendations: Vec<Recommendation>,
    pub evidence_summary: EvidenceSummary,
}

impl DebugAnalysis {
    pub fn new(issue: String) -> Self {
        Self {
            issue,
            whys: Vec::new(),
            root_cause: None,
            recommendations: Vec::new(),
            evidence_summary: EvidenceSummary::default(),
        }
    }
}

/// Single "Why" iteration with hypothesis and evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhyIteration {
    pub depth: u8,
    pub question: String,
    pub hypothesis: String,
    pub evidence: Vec<Evidence>,
    pub confidence: f64,
}

impl WhyIteration {
    pub fn new(depth: u8, question: String, hypothesis: String) -> Self {
        Self {
            depth,
            question,
            hypothesis,
            evidence: Vec::new(),
            confidence: 0.5, // Default medium confidence
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn add_evidence(&mut self, evidence: Evidence) {
        self.evidence.push(evidence);
    }
}

/// Evidence from PMAT analysis tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source: EvidenceSource,
    pub file: PathBuf,
    pub metric: String,
    pub value: serde_json::Value,
    pub interpretation: String,
}

impl Evidence {
    pub fn new(
        source: EvidenceSource,
        file: PathBuf,
        metric: String,
        value: serde_json::Value,
        interpretation: String,
    ) -> Self {
        Self {
            source,
            file,
            metric,
            value,
            interpretation,
        }
    }
}

/// Source of evidence (which PMAT service)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceSource {
    Complexity,
    SATD,
    DeadCode,
    GitChurn,
    TDG,
    ManualInspection,
}

/// Actionable recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub action: String,
    pub file: Option<PathBuf>,
}

impl Recommendation {
    pub fn new(priority: Priority, action: String, file: Option<PathBuf>) -> Self {
        Self {
            priority,
            action,
            file,
        }
    }

    pub fn high(action: String, file: Option<PathBuf>) -> Self {
        Self::new(Priority::High, action, file)
    }

    pub fn medium(action: String, file: Option<PathBuf>) -> Self {
        Self::new(Priority::Medium, action, file)
    }

    pub fn low(action: String, file: Option<PathBuf>) -> Self {
        Self::new(Priority::Low, action, file)
    }
}

/// Recommendation priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Summary of evidence across all Why iterations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub complexity_violations: usize,
    pub satd_markers: usize,
    pub tdg_score: f64,
    pub git_churn_high: bool,
}

impl EvidenceSummary {
    pub fn from_whys(whys: &[WhyIteration]) -> Self {
        let mut summary = Self::default();

        for why in whys {
            for evidence in &why.evidence {
                match evidence.source {
                    EvidenceSource::Complexity => {
                        if let Some(value) = evidence.value.get("value") {
                            if let Some(threshold) = evidence.value.get("threshold") {
                                if value.as_f64().unwrap_or(0.0)
                                    > threshold.as_f64().unwrap_or(20.0)
                                {
                                    summary.complexity_violations += 1;
                                }
                            }
                        }
                    }
                    EvidenceSource::SATD => {
                        if let Some(count) = evidence.value.get("count") {
                            summary.satd_markers += count.as_u64().unwrap_or(0) as usize;
                        } else {
                            summary.satd_markers += 1; // Single marker
                        }
                    }
                    EvidenceSource::TDG => {
                        if let Some(score) = evidence.value.as_f64() {
                            summary.tdg_score = score;
                        }
                    }
                    EvidenceSource::GitChurn => {
                        if let Some(commits) = evidence.value.get("commit_count") {
                            if commits.as_u64().unwrap_or(0) > 10 {
                                summary.git_churn_high = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_analysis_creation() {
        let analysis = DebugAnalysis::new("Test issue".to_string());
        assert_eq!(analysis.issue, "Test issue");
        assert!(analysis.whys.is_empty());
        assert!(analysis.root_cause.is_none());
    }

    #[test]
    fn test_why_iteration_confidence_clamping() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(1.5);
        assert_eq!(why.confidence, 1.0);

        let why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string())
            .with_confidence(-0.5);
        assert_eq!(why.confidence, 0.0);
    }

    #[test]
    fn test_evidence_summary_from_whys() {
        let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());

        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "cyclomatic".to_string(),
            serde_json::json!({"value": 50, "threshold": 20}),
            "High complexity".to_string(),
        ));

        why.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("test.rs"),
            "todo_count".to_string(),
            serde_json::json!({"count": 3}),
            "3 TODO markers".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 1);
        assert_eq!(summary.satd_markers, 3);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================================
    // Test Fixtures and Helpers
    // ============================================================================

    /// Create a basic DebugAnalysis for testing
    fn create_test_debug_analysis() -> DebugAnalysis {
        DebugAnalysis::new("Test issue description".to_string())
    }

    /// Create a WhyIteration with specified parameters
    fn create_test_why_iteration(depth: u8, confidence: f64) -> WhyIteration {
        WhyIteration::new(
            depth,
            format!("Why did this happen (depth {})?", depth),
            format!("Hypothesis at depth {}", depth),
        )
        .with_confidence(confidence)
    }

    /// Create test Evidence with specified source
    fn create_test_evidence(source: EvidenceSource) -> Evidence {
        match source {
            EvidenceSource::Complexity => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "cyclomatic_complexity".to_string(),
                serde_json::json!({"value": 30, "threshold": 20}),
                "High complexity detected".to_string(),
            ),
            EvidenceSource::SATD => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "todo_markers".to_string(),
                serde_json::json!({"count": 5}),
                "5 TODO markers found".to_string(),
            ),
            EvidenceSource::TDG => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "tdg_score".to_string(),
                serde_json::json!(40.0),
                "Low test coverage".to_string(),
            ),
            EvidenceSource::GitChurn => Evidence::new(
                source,
                PathBuf::from("src/test.rs"),
                "commit_count".to_string(),
                serde_json::json!({"commit_count": 15, "days": 30}),
                "High churn detected".to_string(),
            ),
            EvidenceSource::DeadCode => Evidence::new(
                source,
                PathBuf::from("src/unused.rs"),
                "unused_functions".to_string(),
                serde_json::json!({"count": 3}),
                "3 unused functions".to_string(),
            ),
            EvidenceSource::ManualInspection => Evidence::new(
                source,
                PathBuf::from("src/main.rs"),
                "manual_review".to_string(),
                serde_json::json!({"notes": "Reviewed by engineer"}),
                "Manual code review".to_string(),
            ),
        }
    }

    /// Create a Recommendation with specified priority
    fn create_test_recommendation(priority: Priority) -> Recommendation {
        Recommendation::new(
            priority,
            "Test action".to_string(),
            Some(PathBuf::from("test.rs")),
        )
    }

    // ============================================================================
    // DebugAnalysis Unit Tests
    // ============================================================================

    #[test]
    fn test_debug_analysis_new_initializes_correctly() {
        let issue = "Memory leak in parser".to_string();
        let analysis = DebugAnalysis::new(issue.clone());

        assert_eq!(analysis.issue, issue);
        assert!(analysis.whys.is_empty());
        assert!(analysis.root_cause.is_none());
        assert!(analysis.recommendations.is_empty());
        assert_eq!(analysis.evidence_summary.complexity_violations, 0);
        assert_eq!(analysis.evidence_summary.satd_markers, 0);
        assert_eq!(analysis.evidence_summary.tdg_score, 0.0);
        assert!(!analysis.evidence_summary.git_churn_high);
    }

    #[test]
    fn test_debug_analysis_with_empty_issue() {
        let analysis = DebugAnalysis::new(String::new());
        assert_eq!(analysis.issue, "");
    }

    #[test]
    fn test_debug_analysis_with_unicode_issue() {
        let issue = "Error in 日本語 module: 🔥 critical failure".to_string();
        let analysis = DebugAnalysis::new(issue.clone());
        assert_eq!(analysis.issue, issue);
    }

    #[test]
    fn test_debug_analysis_serialization_roundtrip() {
        let mut analysis = create_test_debug_analysis();
        analysis.root_cause = Some("Root cause identified".to_string());
        analysis.whys.push(create_test_why_iteration(1, 0.8));
        analysis
            .recommendations
            .push(Recommendation::high("Fix the issue".to_string(), None));

        let json = serde_json::to_string(&analysis).expect("Serialization should succeed");
        let deserialized: DebugAnalysis =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.issue, analysis.issue);
        assert_eq!(deserialized.root_cause, analysis.root_cause);
        assert_eq!(deserialized.whys.len(), 1);
        assert_eq!(deserialized.recommendations.len(), 1);
    }

    #[test]
    fn test_debug_analysis_clone() {
        let mut analysis = create_test_debug_analysis();
        analysis.root_cause = Some("Test root cause".to_string());

        let cloned = analysis.clone();
        assert_eq!(cloned.issue, analysis.issue);
        assert_eq!(cloned.root_cause, analysis.root_cause);
    }

    // ============================================================================
    // WhyIteration Unit Tests
    // ============================================================================

    #[test]
    fn test_why_iteration_new_defaults() {
        let why = WhyIteration::new(1, "Why?".to_string(), "Because".to_string());

        assert_eq!(why.depth, 1);
        assert_eq!(why.question, "Why?");
        assert_eq!(why.hypothesis, "Because");
        assert!(why.evidence.is_empty());
        assert_eq!(why.confidence, 0.5); // Default confidence
    }

    #[test]
    fn test_why_iteration_with_confidence_valid_range() {
        let why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string())
            .with_confidence(0.75);
        assert_eq!(why.confidence, 0.75);
    }

    #[test]
    fn test_why_iteration_with_confidence_zero() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(0.0);
        assert_eq!(why.confidence, 0.0);
    }

    #[test]
    fn test_why_iteration_with_confidence_one() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(1.0);
        assert_eq!(why.confidence, 1.0);
    }

    #[test]
    fn test_why_iteration_with_confidence_clamps_above_one() {
        let why =
            WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string()).with_confidence(2.5);
        assert_eq!(why.confidence, 1.0);
    }

    #[test]
    fn test_why_iteration_with_confidence_clamps_below_zero() {
        let why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string())
            .with_confidence(-1.0);
        assert_eq!(why.confidence, 0.0);
    }

    #[test]
    fn test_why_iteration_add_evidence() {
        let mut why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string());
        assert!(why.evidence.is_empty());

        why.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        assert_eq!(why.evidence.len(), 1);

        why.add_evidence(create_test_evidence(EvidenceSource::SATD));
        assert_eq!(why.evidence.len(), 2);
    }

    #[test]
    fn test_why_iteration_depth_range() {
        for depth in 1..=10 {
            let why = WhyIteration::new(
                depth,
                format!("Why {}?", depth),
                format!("Hypothesis {}", depth),
            );
            assert_eq!(why.depth, depth);
        }
    }

    #[test]
    fn test_why_iteration_serialization_roundtrip() {
        let mut why = create_test_why_iteration(3, 0.85);
        why.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        why.add_evidence(create_test_evidence(EvidenceSource::TDG));

        let json = serde_json::to_string(&why).expect("Serialization should succeed");
        let deserialized: WhyIteration =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.depth, why.depth);
        assert_eq!(deserialized.question, why.question);
        assert_eq!(deserialized.hypothesis, why.hypothesis);
        assert_eq!(deserialized.confidence, why.confidence);
        assert_eq!(deserialized.evidence.len(), 2);
    }

    // ============================================================================
    // Evidence Unit Tests
    // ============================================================================

    #[test]
    fn test_evidence_new_complexity() {
        let evidence = create_test_evidence(EvidenceSource::Complexity);

        assert_eq!(evidence.source, EvidenceSource::Complexity);
        assert_eq!(evidence.file, PathBuf::from("src/test.rs"));
        assert_eq!(evidence.metric, "cyclomatic_complexity");
        assert!(evidence.value.get("value").is_some());
        assert!(evidence.value.get("threshold").is_some());
    }

    #[test]
    fn test_evidence_new_satd() {
        let evidence = create_test_evidence(EvidenceSource::SATD);

        assert_eq!(evidence.source, EvidenceSource::SATD);
        assert_eq!(evidence.metric, "todo_markers");
        assert_eq!(evidence.value.get("count").unwrap().as_u64(), Some(5));
    }

    #[test]
    fn test_evidence_new_tdg() {
        let evidence = create_test_evidence(EvidenceSource::TDG);

        assert_eq!(evidence.source, EvidenceSource::TDG);
        assert_eq!(evidence.value.as_f64(), Some(40.0));
    }

    #[test]
    fn test_evidence_new_git_churn() {
        let evidence = create_test_evidence(EvidenceSource::GitChurn);

        assert_eq!(evidence.source, EvidenceSource::GitChurn);
        assert_eq!(
            evidence.value.get("commit_count").unwrap().as_u64(),
            Some(15)
        );
        assert_eq!(evidence.value.get("days").unwrap().as_u64(), Some(30));
    }

    #[test]
    fn test_evidence_new_dead_code() {
        let evidence = create_test_evidence(EvidenceSource::DeadCode);

        assert_eq!(evidence.source, EvidenceSource::DeadCode);
        assert_eq!(evidence.file, PathBuf::from("src/unused.rs"));
    }

    #[test]
    fn test_evidence_new_manual_inspection() {
        let evidence = create_test_evidence(EvidenceSource::ManualInspection);

        assert_eq!(evidence.source, EvidenceSource::ManualInspection);
        assert!(evidence.value.get("notes").is_some());
    }

    #[test]
    fn test_evidence_serialization_roundtrip() {
        let evidence = create_test_evidence(EvidenceSource::Complexity);

        let json = serde_json::to_string(&evidence).expect("Serialization should succeed");
        let deserialized: Evidence =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.source, evidence.source);
        assert_eq!(deserialized.file, evidence.file);
        assert_eq!(deserialized.metric, evidence.metric);
        assert_eq!(deserialized.interpretation, evidence.interpretation);
    }

    #[test]
    fn test_evidence_with_null_value() {
        let evidence = Evidence::new(
            EvidenceSource::ManualInspection,
            PathBuf::from("test.rs"),
            "observation".to_string(),
            serde_json::Value::Null,
            "No additional data".to_string(),
        );

        assert!(evidence.value.is_null());
    }

    #[test]
    fn test_evidence_with_complex_json_value() {
        let complex_value = serde_json::json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"}
            },
            "boolean": true
        });

        let evidence = Evidence::new(
            EvidenceSource::ManualInspection,
            PathBuf::from("test.rs"),
            "complex".to_string(),
            complex_value.clone(),
            "Complex structure".to_string(),
        );

        assert_eq!(evidence.value, complex_value);
    }

    // ============================================================================
    // EvidenceSource Unit Tests
    // ============================================================================

    #[test]
    fn test_evidence_source_equality() {
        assert_eq!(EvidenceSource::Complexity, EvidenceSource::Complexity);
        assert_ne!(EvidenceSource::Complexity, EvidenceSource::SATD);
    }

    #[test]
    fn test_evidence_source_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();

        set.insert(EvidenceSource::Complexity);
        set.insert(EvidenceSource::SATD);
        set.insert(EvidenceSource::TDG);
        set.insert(EvidenceSource::Complexity); // Duplicate

        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_evidence_source_all_variants_serialization() {
        let sources = vec![
            EvidenceSource::Complexity,
            EvidenceSource::SATD,
            EvidenceSource::DeadCode,
            EvidenceSource::GitChurn,
            EvidenceSource::TDG,
            EvidenceSource::ManualInspection,
        ];

        for source in sources {
            let json = serde_json::to_string(&source).expect("Serialization should succeed");
            let deserialized: EvidenceSource =
                serde_json::from_str(&json).expect("Deserialization should succeed");
            assert_eq!(deserialized, source);
        }
    }

    #[test]
    fn test_evidence_source_copy() {
        let source = EvidenceSource::Complexity;
        let copied = source;
        assert_eq!(source, copied);
    }

    // ============================================================================
    // Recommendation Unit Tests
    // ============================================================================

    #[test]
    fn test_recommendation_new() {
        let rec = Recommendation::new(
            Priority::High,
            "Fix the bug".to_string(),
            Some(PathBuf::from("src/buggy.rs")),
        );

        assert_eq!(rec.priority, Priority::High);
        assert_eq!(rec.action, "Fix the bug");
        assert_eq!(rec.file, Some(PathBuf::from("src/buggy.rs")));
    }

    #[test]
    fn test_recommendation_high_factory() {
        let rec = Recommendation::high("Critical fix".to_string(), None);

        assert_eq!(rec.priority, Priority::High);
        assert_eq!(rec.action, "Critical fix");
        assert!(rec.file.is_none());
    }

    #[test]
    fn test_recommendation_medium_factory() {
        let rec = Recommendation::medium(
            "Improve test coverage".to_string(),
            Some(PathBuf::from("tests/")),
        );

        assert_eq!(rec.priority, Priority::Medium);
        assert_eq!(rec.action, "Improve test coverage");
    }

    #[test]
    fn test_recommendation_low_factory() {
        let rec = Recommendation::low("Refactor for readability".to_string(), None);

        assert_eq!(rec.priority, Priority::Low);
    }

    #[test]
    fn test_recommendation_without_file() {
        let rec = Recommendation::new(Priority::Medium, "General improvement".to_string(), None);

        assert!(rec.file.is_none());
    }

    #[test]
    fn test_recommendation_serialization_roundtrip() {
        let rec = create_test_recommendation(Priority::High);

        let json = serde_json::to_string(&rec).expect("Serialization should succeed");
        let deserialized: Recommendation =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.priority, rec.priority);
        assert_eq!(deserialized.action, rec.action);
        assert_eq!(deserialized.file, rec.file);
    }

    // ============================================================================
    // Priority Unit Tests
    // ============================================================================

    #[test]
    fn test_priority_equality() {
        assert_eq!(Priority::High, Priority::High);
        assert_eq!(Priority::Medium, Priority::Medium);
        assert_eq!(Priority::Low, Priority::Low);
        assert_ne!(Priority::High, Priority::Low);
    }

    #[test]
    fn test_priority_serialization_roundtrip() {
        for priority in [Priority::High, Priority::Medium, Priority::Low] {
            let json = serde_json::to_string(&priority).expect("Serialization should succeed");
            let deserialized: Priority =
                serde_json::from_str(&json).expect("Deserialization should succeed");
            assert_eq!(deserialized, priority);
        }
    }

    #[test]
    fn test_priority_copy() {
        let priority = Priority::High;
        let copied = priority;
        assert_eq!(priority, copied);
    }

    // ============================================================================
    // EvidenceSummary Unit Tests
    // ============================================================================

    #[test]
    fn test_evidence_summary_default() {
        let summary = EvidenceSummary::default();

        assert_eq!(summary.complexity_violations, 0);
        assert_eq!(summary.satd_markers, 0);
        assert_eq!(summary.tdg_score, 0.0);
        assert!(!summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_from_empty_whys() {
        let summary = EvidenceSummary::from_whys(&[]);

        assert_eq!(summary.complexity_violations, 0);
        assert_eq!(summary.satd_markers, 0);
        assert_eq!(summary.tdg_score, 0.0);
        assert!(!summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_counts_complexity_violations() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("a.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 25, "threshold": 20}),
            "High".to_string(),
        ));
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("b.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 30, "threshold": 20}),
            "Very high".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 2);
    }

    #[test]
    fn test_evidence_summary_ignores_below_threshold_complexity() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("simple.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 15, "threshold": 20}),
            "OK".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 0);
    }

    #[test]
    fn test_evidence_summary_counts_satd_with_count() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("test.rs"),
            "satd".to_string(),
            serde_json::json!({"count": 7}),
            "TODOs".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.satd_markers, 7);
    }

    #[test]
    fn test_evidence_summary_counts_satd_without_count() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("test.rs"),
            "satd".to_string(),
            serde_json::json!({}), // No count field
            "Single marker".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.satd_markers, 1); // Defaults to 1
    }

    #[test]
    fn test_evidence_summary_accumulates_satd_across_whys() {
        let mut why1 = create_test_why_iteration(1, 0.5);
        why1.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("a.rs"),
            "satd".to_string(),
            serde_json::json!({"count": 3}),
            "TODOs".to_string(),
        ));

        let mut why2 = create_test_why_iteration(2, 0.6);
        why2.add_evidence(Evidence::new(
            EvidenceSource::SATD,
            PathBuf::from("b.rs"),
            "satd".to_string(),
            serde_json::json!({"count": 5}),
            "FIXMEs".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why1, why2]);
        assert_eq!(summary.satd_markers, 8);
    }

    #[test]
    fn test_evidence_summary_extracts_tdg_score() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::TDG,
            PathBuf::from("test.rs"),
            "tdg".to_string(),
            serde_json::json!(75.5),
            "Moderate coverage".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!((summary.tdg_score - 75.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evidence_summary_detects_high_git_churn() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::GitChurn,
            PathBuf::from("test.rs"),
            "churn".to_string(),
            serde_json::json!({"commit_count": 15}),
            "High churn".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!(summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_low_git_churn() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::GitChurn,
            PathBuf::from("test.rs"),
            "churn".to_string(),
            serde_json::json!({"commit_count": 5}),
            "Low churn".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert!(!summary.git_churn_high);
    }

    #[test]
    fn test_evidence_summary_ignores_dead_code_and_manual() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(create_test_evidence(EvidenceSource::DeadCode));
        why.add_evidence(create_test_evidence(EvidenceSource::ManualInspection));

        let summary = EvidenceSummary::from_whys(&[why]);
        // These sources don't contribute to the summary fields
        assert_eq!(summary.complexity_violations, 0);
        assert_eq!(summary.satd_markers, 0);
    }

    #[test]
    fn test_evidence_summary_serialization_roundtrip() {
        let summary = EvidenceSummary {
            complexity_violations: 3,
            satd_markers: 7,
            tdg_score: 65.5,
            git_churn_high: true,
        };

        let json = serde_json::to_string(&summary).expect("Serialization should succeed");
        let deserialized: EvidenceSummary =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.complexity_violations, 3);
        assert_eq!(deserialized.satd_markers, 7);
        assert!((deserialized.tdg_score - 65.5).abs() < f64::EPSILON);
        assert!(deserialized.git_churn_high);
    }

    // ============================================================================
    // Edge Cases and Error Paths
    // ============================================================================

    #[test]
    fn test_evidence_summary_handles_missing_value_in_complexity() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "complexity".to_string(),
            serde_json::json!({"threshold": 20}), // Missing value
            "Unknown".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.complexity_violations, 0); // Should not count as violation
    }

    #[test]
    fn test_evidence_summary_handles_missing_threshold_in_complexity() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "complexity".to_string(),
            serde_json::json!({"value": 25}), // Missing threshold
            "High".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        // Missing threshold means no violation is counted (requires both value and threshold)
        assert_eq!(summary.complexity_violations, 0);
    }

    #[test]
    fn test_evidence_summary_handles_non_numeric_tdg() {
        let mut why = create_test_why_iteration(1, 0.5);
        why.add_evidence(Evidence::new(
            EvidenceSource::TDG,
            PathBuf::from("test.rs"),
            "tdg".to_string(),
            serde_json::json!("not a number"),
            "Invalid".to_string(),
        ));

        let summary = EvidenceSummary::from_whys(&[why]);
        assert_eq!(summary.tdg_score, 0.0); // Default when parsing fails
    }

    #[test]
    fn test_why_iteration_with_many_evidence_items() {
        let mut why = create_test_why_iteration(1, 0.5);

        for _ in 0..100 {
            why.add_evidence(create_test_evidence(EvidenceSource::SATD));
        }

        assert_eq!(why.evidence.len(), 100);
    }

    #[test]
    fn test_debug_analysis_with_many_whys() {
        let mut analysis = create_test_debug_analysis();

        for depth in 1..=10 {
            analysis
                .whys
                .push(create_test_why_iteration(depth, depth as f64 / 10.0));
        }

        assert_eq!(analysis.whys.len(), 10);
        assert_eq!(analysis.whys[9].depth, 10);
    }

    #[test]
    fn test_evidence_with_very_long_strings() {
        let long_metric = "a".repeat(10000);
        let long_interpretation = "b".repeat(10000);

        let evidence = Evidence::new(
            EvidenceSource::ManualInspection,
            PathBuf::from("test.rs"),
            long_metric.clone(),
            serde_json::json!(null),
            long_interpretation.clone(),
        );

        assert_eq!(evidence.metric.len(), 10000);
        assert_eq!(evidence.interpretation.len(), 10000);
    }

    #[test]
    fn test_recommendation_with_very_long_action() {
        let long_action = "x".repeat(10000);
        let rec = Recommendation::high(long_action.clone(), None);
        assert_eq!(rec.action.len(), 10000);
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest! {
        #[test]
        fn prop_confidence_always_clamped(confidence in -100.0f64..100.0) {
            let why = WhyIteration::new(
                1,
                "Why?".to_string(),
                "Hypothesis".to_string(),
            ).with_confidence(confidence);

            prop_assert!(why.confidence >= 0.0);
            prop_assert!(why.confidence <= 1.0);
        }

        #[test]
        fn prop_depth_preserved(depth in 1u8..=10) {
            let why = WhyIteration::new(
                depth,
                "Question".to_string(),
                "Hypothesis".to_string(),
            );
            prop_assert_eq!(why.depth, depth);
        }

        #[test]
        fn prop_evidence_count_preserved(count in 0usize..50) {
            let mut why = WhyIteration::new(1, "Q".to_string(), "H".to_string());

            for _ in 0..count {
                why.add_evidence(Evidence::new(
                    EvidenceSource::ManualInspection,
                    PathBuf::from("test.rs"),
                    "metric".to_string(),
                    serde_json::json!(null),
                    "interpretation".to_string(),
                ));
            }

            prop_assert_eq!(why.evidence.len(), count);
        }

        #[test]
        fn prop_serialization_roundtrip_debug_analysis(issue in "\\PC{1,100}") {
            let analysis = DebugAnalysis::new(issue.clone());
            let json = serde_json::to_string(&analysis)
                .expect("Serialization should succeed");
            let deserialized: DebugAnalysis = serde_json::from_str(&json)
                .expect("Deserialization should succeed");
            prop_assert_eq!(deserialized.issue, issue);
        }

        #[test]
        fn prop_satd_count_accumulates(counts in proptest::collection::vec(0u64..100, 1..10)) {
            let mut whys = Vec::new();

            for (i, count) in counts.iter().enumerate() {
                let mut why = create_test_why_iteration((i + 1) as u8, 0.5);
                why.add_evidence(Evidence::new(
                    EvidenceSource::SATD,
                    PathBuf::from("test.rs"),
                    "satd".to_string(),
                    serde_json::json!({"count": count}),
                    "markers".to_string(),
                ));
                whys.push(why);
            }

            let summary = EvidenceSummary::from_whys(&whys);
            let expected: u64 = counts.iter().sum();
            prop_assert_eq!(summary.satd_markers, expected as usize);
        }

        #[test]
        fn prop_complexity_violations_count_correctly(
            values in proptest::collection::vec(0.0f64..100.0, 1..20)
        ) {
            let mut why = create_test_why_iteration(1, 0.5);

            for (i, value) in values.iter().enumerate() {
                why.add_evidence(Evidence::new(
                    EvidenceSource::Complexity,
                    PathBuf::from(format!("file{}.rs", i)),
                    "complexity".to_string(),
                    serde_json::json!({"value": value, "threshold": 20.0}),
                    "metrics".to_string(),
                ));
            }

            let summary = EvidenceSummary::from_whys(&[why]);
            let expected = values.iter().filter(|&&v| v > 20.0).count();
            prop_assert_eq!(summary.complexity_violations, expected);
        }

        #[test]
        fn prop_git_churn_threshold_at_10(commit_count in 0u64..100) {
            let mut why = create_test_why_iteration(1, 0.5);
            why.add_evidence(Evidence::new(
                EvidenceSource::GitChurn,
                PathBuf::from("test.rs"),
                "churn".to_string(),
                serde_json::json!({"commit_count": commit_count}),
                "churn".to_string(),
            ));

            let summary = EvidenceSummary::from_whys(&[why]);
            prop_assert_eq!(summary.git_churn_high, commit_count > 10);
        }

        #[test]
        fn prop_priority_serialization_stable(priority in prop::sample::select(vec![
            Priority::High,
            Priority::Medium,
            Priority::Low,
        ])) {
            let json = serde_json::to_string(&priority).unwrap();
            let roundtrip: Priority = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(roundtrip, priority);
        }
    }

    // ============================================================================
    // Integration-Style Tests
    // ============================================================================

    #[test]
    fn test_complete_analysis_workflow() {
        // Simulate a complete Five Whys analysis workflow
        let mut analysis = DebugAnalysis::new("Memory leak in parser".to_string());

        // Why 1
        let mut why1 = WhyIteration::new(
            1,
            "Why is there a memory leak?".to_string(),
            "Buffer not freed after use".to_string(),
        )
        .with_confidence(0.6);
        why1.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        analysis.whys.push(why1);

        // Why 2
        let mut why2 = WhyIteration::new(
            2,
            "Why is the buffer not freed?".to_string(),
            "Early return bypasses cleanup".to_string(),
        )
        .with_confidence(0.7);
        why2.add_evidence(create_test_evidence(EvidenceSource::SATD));
        analysis.whys.push(why2);

        // Why 3
        let mut why3 = WhyIteration::new(
            3,
            "Why does early return bypass cleanup?".to_string(),
            "Missing RAII pattern".to_string(),
        )
        .with_confidence(0.85);
        why3.add_evidence(create_test_evidence(EvidenceSource::TDG));
        analysis.whys.push(why3);

        // Set root cause
        analysis.root_cause = Some("Missing RAII pattern for resource management".to_string());

        // Add recommendations
        analysis.recommendations.push(Recommendation::high(
            "Implement Drop trait for buffer wrapper".to_string(),
            Some(PathBuf::from("src/parser/buffer.rs")),
        ));
        analysis.recommendations.push(Recommendation::medium(
            "Add unit tests for cleanup paths".to_string(),
            Some(PathBuf::from("tests/parser_tests.rs")),
        ));

        // Update evidence summary
        analysis.evidence_summary = EvidenceSummary::from_whys(&analysis.whys);

        // Verify the complete analysis
        assert_eq!(analysis.issue, "Memory leak in parser");
        assert_eq!(analysis.whys.len(), 3);
        assert!(analysis.root_cause.is_some());
        assert_eq!(analysis.recommendations.len(), 2);
        assert!(analysis.evidence_summary.complexity_violations > 0);
        assert!(analysis.evidence_summary.satd_markers > 0);
    }

    #[test]
    fn test_all_evidence_sources_in_summary() {
        let mut why = create_test_why_iteration(1, 0.5);

        // Add evidence from all sources
        why.add_evidence(create_test_evidence(EvidenceSource::Complexity));
        why.add_evidence(create_test_evidence(EvidenceSource::SATD));
        why.add_evidence(create_test_evidence(EvidenceSource::TDG));
        why.add_evidence(create_test_evidence(EvidenceSource::GitChurn));
        why.add_evidence(create_test_evidence(EvidenceSource::DeadCode));
        why.add_evidence(create_test_evidence(EvidenceSource::ManualInspection));

        let summary = EvidenceSummary::from_whys(&[why]);

        assert_eq!(summary.complexity_violations, 1);
        assert_eq!(summary.satd_markers, 5);
        assert!((summary.tdg_score - 40.0).abs() < f64::EPSILON);
        assert!(summary.git_churn_high);
    }
}
