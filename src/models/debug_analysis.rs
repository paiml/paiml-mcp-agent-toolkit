#![cfg_attr(coverage_nightly, coverage(off))]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(depth: u8, question: String, hypothesis: String) -> Self {
        Self {
            depth,
            question,
            hypothesis,
            evidence: Vec::new(),
            confidence: 0.5, // Default medium confidence
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With confidence.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add evidence.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
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
///
/// v2 weights (PMAT-510): Complexity 25%, SATD 20%, GitChurn 15%,
/// EvoScoreTrajectory 15%, CoverageDelta 15%, DeadCode 10%.
/// TDG removed (redundant with complexity+churn). ManualInspection kept for manual overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceSource {
    Complexity,
    SATD,
    DeadCode,
    GitChurn,
    /// Kept for backward compatibility but no longer weighted in v2
    TDG,
    ManualInspection,
    /// CB-142 EvoScore trajectory: is the affected area improving or regressing?
    EvoScoreTrajectory,
    /// Coverage delta: did recent changes decrease test coverage?
    CoverageDelta,
    /// Source locations matching distinctive terms from the reported issue.
    ///
    /// Every other variant here is a *repo-wide* metric that is identical no
    /// matter what issue was reported. Without at least one of these, an
    /// analysis has not looked at the problem it was asked about, and must not
    /// present a root cause (GH #637).
    IssueLocation,
}

/// Actionable recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub action: String,
    pub file: Option<PathBuf>,
}

impl Recommendation {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
    pub fn new(priority: Priority, action: String, file: Option<PathBuf>) -> Self {
        Self {
            priority,
            action,
            file,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// High.
    pub fn high(action: String, file: Option<PathBuf>) -> Self {
        Self::new(Priority::High, action, file)
    }

    /// Medium.
    pub fn medium(action: String, file: Option<PathBuf>) -> Self {
        Self::new(Priority::Medium, action, file)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Low.
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
    /// Was `tdg_score` actually populated by TDG evidence?
    ///
    /// TDG was dropped from the v2 evidence set (weight 0, redundant with
    /// complexity + churn), so nothing produces TDG evidence and `tdg_score`
    /// keeps its `0.0` default. Reports printed that as "TDG score 0.0/100 ❌",
    /// a failing grade for something never measured — and one that contradicted
    /// `pmat analyze`, which reported an average of 95.4 for the same tree
    /// (GH #637).
    #[serde(default)]
    pub tdg_measured: bool,
    pub git_churn_high: bool,
    /// CB-142 EvoScore trajectory: positive = improving, negative = regressing
    #[serde(default)]
    pub evoscore_trajectory: f64,
    /// Coverage delta: positive = coverage increased, negative = decreased
    #[serde(default)]
    pub coverage_delta: f64,
}

impl EvidenceSummary {
    fn process_complexity_evidence(&mut self, evidence: &Evidence) {
        let value = evidence
            .value
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let threshold = evidence
            .value
            .get("threshold")
            .and_then(|t| t.as_f64())
            .unwrap_or(20.0);
        if value > threshold {
            self.complexity_violations += 1;
        }
    }

    fn process_satd_evidence(&mut self, evidence: &Evidence) {
        self.satd_markers += evidence
            .value
            .get("count")
            .and_then(|c| c.as_u64())
            .unwrap_or(1) as usize;
    }

    fn process_evidence(&mut self, evidence: &Evidence) {
        match evidence.source {
            EvidenceSource::Complexity => self.process_complexity_evidence(evidence),
            EvidenceSource::SATD => self.process_satd_evidence(evidence),
            EvidenceSource::TDG => {
                if let Some(score) = evidence.value.as_f64() {
                    self.tdg_score = score;
                    self.tdg_measured = true;
                }
            }
            // Locations are rendered per-why, not in the aggregate table.
            EvidenceSource::IssueLocation => {}
            EvidenceSource::GitChurn => {
                let commits = evidence
                    .value
                    .get("commit_count")
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);
                if commits > 10 {
                    self.git_churn_high = true;
                }
            }
            EvidenceSource::EvoScoreTrajectory => {
                self.evoscore_trajectory = evidence
                    .value
                    .get("evoscore")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
            }
            EvidenceSource::CoverageDelta => {
                self.coverage_delta = evidence
                    .value
                    .get("delta")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
            }
            _ => {}
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// From whys.
    pub fn from_whys(whys: &[WhyIteration]) -> Self {
        let mut summary = Self::default();
        for why in whys {
            for evidence in &why.evidence {
                summary.process_evidence(evidence);
            }
        }
        summary
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("debug_analysis_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use proptest::prelude::*;

    include!("debug_analysis_coverage_helpers.rs");
    include!("debug_analysis_coverage_unit.rs");
    include!("debug_analysis_coverage_recommend.rs");
    include!("debug_analysis_coverage_props.rs");
}
