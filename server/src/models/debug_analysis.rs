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
        let why = WhyIteration::new(1, "Why?".to_string(), "Hypothesis".to_string())
            .with_confidence(1.5);
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
