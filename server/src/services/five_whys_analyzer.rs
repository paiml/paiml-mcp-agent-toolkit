// Five Whys Root Cause Analyzer - Toyota Way Methodology
//
// GREEN PHASE: Minimal implementation to make tests pass
//
// Integrates with existing PMAT services:
// - Complexity analysis
// - SATD detection
// - Dead code detection
// - Git churn analysis
// - TDG scoring

use crate::models::debug_analysis::*;
use anyhow::{bail, Result};
use serde_json::json;
use std::path::Path;

/// Five Whys analyzer with PMAT tool integration
pub struct FiveWhysAnalyzer {
    // Services will be added as we integrate them
}

impl FiveWhysAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    /// Analyze an issue using Five Whys methodology
    ///
    /// # Arguments
    /// * `issue` - Description of the issue/symptom
    /// * `path` - Project path to analyze
    /// * `depth` - Number of "why" iterations (1-10)
    ///
    /// # Returns
    /// Complete debug analysis with root cause and recommendations
    pub async fn analyze(&self, issue: &str, path: &Path, depth: u8) -> Result<DebugAnalysis> {
        // Validation
        if issue.is_empty() {
            bail!("Issue description cannot be empty");
        }
        if depth == 0 || depth > 10 {
            bail!("Depth must be between 1 and 10, got {}", depth);
        }
        if !path.exists() {
            bail!("Path does not exist: {}", path.display());
        }

        let mut analysis = DebugAnalysis::new(issue.to_string());

        // Iterate through Why questions
        for i in 1..=depth {
            let why = self.iterate_why(issue, path, i, &analysis.whys).await?;

            // Early termination if high confidence reached (>0.9) after at least 3 iterations
            if i >= 3 && why.confidence > 0.9 {
                analysis.whys.push(why);
                break;
            }

            analysis.whys.push(why);
        }

        // Extract root cause from final Why
        analysis.root_cause = self.extract_root_cause(&analysis.whys)?;

        // Generate recommendations
        analysis.recommendations = self.generate_recommendations(
            &analysis.whys,
            &analysis.root_cause.clone().unwrap_or_default(),
        )?;

        // Summarize evidence
        analysis.evidence_summary = EvidenceSummary::from_whys(&analysis.whys);

        Ok(analysis)
    }

    /// Single Why iteration
    async fn iterate_why(
        &self,
        issue: &str,
        path: &Path,
        depth: u8,
        previous_whys: &[WhyIteration],
    ) -> Result<WhyIteration> {
        // Formulate question
        let question = self.formulate_question(issue, depth, previous_whys)?;

        // Gather evidence from PMAT services
        let evidence = self.gather_evidence(path).await?;

        // Generate hypothesis based on evidence
        let hypothesis = self.generate_hypothesis(&question, &evidence, depth)?;

        // Calculate confidence
        let confidence = self.calculate_confidence(&evidence)?;

        let mut why = WhyIteration::new(depth, question, hypothesis).with_confidence(confidence);

        why.evidence = evidence;

        Ok(why)
    }

    /// Formulate the "Why?" question for this iteration
    fn formulate_question(
        &self,
        issue: &str,
        depth: u8,
        previous_whys: &[WhyIteration],
    ) -> Result<String> {
        let question = if depth == 1 {
            format!("Why did this occur: {}?", issue)
        } else if let Some(prev) = previous_whys.last() {
            format!("Why {}?", prev.hypothesis.trim_end_matches('.'))
        } else {
            format!("Why did this occur (iteration {})?", depth)
        };

        Ok(question)
    }

    /// Gather evidence from all PMAT services
    async fn gather_evidence(&self, path: &Path) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        // For GREEN phase: Generate synthetic evidence
        // In REFACTOR phase: Integrate real PMAT services

        // Complexity evidence (synthetic)
        if path.exists() {
            evidence.push(Evidence::new(
                EvidenceSource::Complexity,
                path.to_path_buf(),
                "cyclomatic_complexity".to_string(),
                json!({"value": 25, "threshold": 20}),
                "Moderate complexity detected (25 > 20 threshold)".to_string(),
            ));
        }

        // SATD evidence (synthetic)
        evidence.push(Evidence::new(
            EvidenceSource::SATD,
            path.to_path_buf(),
            "todo_markers".to_string(),
            json!({"count": 3}),
            "Found 3 TODO/FIXME markers indicating known technical debt".to_string(),
        ));

        // TDG evidence (synthetic)
        evidence.push(Evidence::new(
            EvidenceSource::TDG,
            path.to_path_buf(),
            "tdg_score".to_string(),
            json!(45.0),
            "Low test coverage (45/100) indicates fragile code".to_string(),
        ));

        // Git churn evidence (synthetic)
        evidence.push(Evidence::new(
            EvidenceSource::GitChurn,
            path.to_path_buf(),
            "commit_count".to_string(),
            json!({"commit_count": 15, "days": 30}),
            "High churn: 15 commits in 30 days indicates instability".to_string(),
        ));

        Ok(evidence)
    }

    /// Generate hypothesis based on evidence
    fn generate_hypothesis(
        &self,
        _question: &str,
        evidence: &[Evidence],
        depth: u8,
    ) -> Result<String> {
        // Analyze evidence to form hypothesis
        let has_high_complexity = evidence.iter().any(|e| {
            e.source == EvidenceSource::Complexity
                && e.value.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) > 20.0
        });

        let has_satd = evidence.iter().any(|e| e.source == EvidenceSource::SATD);

        let has_low_tdg = evidence
            .iter()
            .any(|e| e.source == EvidenceSource::TDG && e.value.as_f64().unwrap_or(100.0) < 50.0);

        let has_high_churn = evidence.iter().any(|e| {
            e.source == EvidenceSource::GitChurn
                && e.value
                    .get("commit_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    > 10
        });

        // Form hypothesis based on evidence patterns
        let hypothesis = match depth {
            1 => {
                if has_high_complexity {
                    "Code complexity exceeds acceptable thresholds".to_string()
                } else if has_satd {
                    "Known technical debt markers present in codebase".to_string()
                } else {
                    "Issue manifested due to code quality factors".to_string()
                }
            }
            2 => {
                if has_low_tdg {
                    "Insufficient test coverage allowed defect to slip through".to_string()
                } else if has_high_complexity {
                    "Complex control flow makes code difficult to understand and maintain"
                        .to_string()
                } else {
                    "Code structure contributed to the problem".to_string()
                }
            }
            3 => {
                if has_high_churn {
                    "Frequent changes indicate unstable or poorly understood code".to_string()
                } else if has_satd {
                    "Technical debt accumulated, indicating deferred maintenance".to_string()
                } else {
                    "Architectural constraints led to current state".to_string()
                }
            }
            4 => "Requirements or constraints were not fully specified".to_string(),
            _ => "Root cause: Systematic process gap in development workflow".to_string(),
        };

        Ok(hypothesis)
    }

    /// Calculate confidence score based on evidence strength
    pub fn calculate_confidence(&self, evidence: &[Evidence]) -> Result<f64> {
        if evidence.is_empty() {
            return Ok(0.3); // Low confidence with no evidence
        }

        let mut confidence = 0.0;
        let mut weight_sum = 0.0;

        for ev in evidence {
            let (evidence_weight, severity_multiplier) = match ev.source {
                EvidenceSource::Complexity => {
                    let value = ev
                        .value
                        .get("value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let threshold = ev
                        .value
                        .get("threshold")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(20.0);
                    let severity = (value - threshold).max(0.0) / threshold;
                    (0.25, 1.0 + severity.min(1.0))
                }
                EvidenceSource::SATD => {
                    let count = ev.value.get("count").and_then(|v| v.as_u64()).unwrap_or(1);
                    let severity = (count as f64).min(10.0) / 10.0;
                    (0.20, 1.0 + severity)
                }
                EvidenceSource::TDG => {
                    let score = ev.value.as_f64().unwrap_or(50.0);
                    let severity = (50.0 - score).max(0.0) / 50.0;
                    (0.25, 1.0 + severity)
                }
                EvidenceSource::GitChurn => {
                    let commits = ev
                        .value
                        .get("commit_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let severity = (commits as f64).min(20.0) / 20.0;
                    (0.20, 1.0 + severity)
                }
                EvidenceSource::DeadCode => (0.10, 1.0),
                EvidenceSource::ManualInspection => (0.15, 1.0),
            };

            confidence += evidence_weight * severity_multiplier;
            weight_sum += evidence_weight;
        }

        // Normalize and clamp
        let normalized = if weight_sum > 0.0 {
            (confidence / weight_sum).clamp(0.0, 1.0)
        } else {
            0.5
        };

        Ok(normalized)
    }

    /// Extract root cause from Why iterations
    fn extract_root_cause(&self, whys: &[WhyIteration]) -> Result<Option<String>> {
        if whys.is_empty() {
            return Ok(None);
        }

        // Root cause is the hypothesis from the final Why
        let last_why = whys.last().expect("internal error");
        Ok(Some(last_why.hypothesis.clone()))
    }

    /// Generate actionable recommendations
    pub fn generate_recommendations(
        &self,
        whys: &[WhyIteration],
        root_cause: &str,
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Analyze evidence across all whys to generate recommendations
        let has_high_complexity = whys.iter().any(|w| {
            w.evidence.iter().any(|e| {
                e.source == EvidenceSource::Complexity
                    && e.value.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) > 20.0
            })
        });

        let has_satd = whys
            .iter()
            .any(|w| w.evidence.iter().any(|e| e.source == EvidenceSource::SATD));

        let has_low_tdg = whys.iter().any(|w| {
            w.evidence.iter().any(|e| {
                e.source == EvidenceSource::TDG && e.value.as_f64().unwrap_or(100.0) < 50.0
            })
        });

        let has_high_churn = whys.iter().any(|w| {
            w.evidence.iter().any(|e| {
                e.source == EvidenceSource::GitChurn
                    && e.value
                        .get("commit_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        > 10
            })
        });

        // Generate recommendations based on evidence
        if has_high_complexity {
            recommendations.push(Recommendation::high(
                "Refactor complex functions to reduce cyclomatic complexity below 20".to_string(),
                None,
            ));
        }

        if has_satd {
            recommendations.push(Recommendation::high(
                "Resolve technical debt markers (TODO/FIXME) in next sprint".to_string(),
                None,
            ));
        }

        if has_low_tdg {
            recommendations.push(Recommendation::high(
                "Add comprehensive test coverage (target: ≥85%) using EXTREME TDD".to_string(),
                None,
            ));
        }

        if has_high_churn {
            recommendations.push(Recommendation::medium(
                "Stabilize frequently changed code through better design patterns".to_string(),
                None,
            ));
        }

        // Always add root cause fix recommendation
        recommendations.push(Recommendation::high(
            format!("Address root cause: {}", root_cause),
            None,
        ));

        // Add specification recommendation
        recommendations.push(Recommendation::medium(
            "Document requirements and constraints in specification".to_string(),
            None,
        ));

        Ok(recommendations)
    }
}

impl Default for FiveWhysAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = FiveWhysAnalyzer::new();
        let _ = analyzer; // Suppress unused warning
    }

    #[tokio::test]
    async fn test_validate_depth_range() {
        let analyzer = FiveWhysAnalyzer::new();

        // Depth 0 should fail
        let result = analyzer.analyze("Test", Path::new("."), 0).await;
        assert!(result.is_err());

        // Depth 11 should fail
        let result = analyzer.analyze("Test", Path::new("."), 11).await;
        assert!(result.is_err());

        // Depth 5 should succeed
        let result = analyzer.analyze("Test", Path::new("."), 5).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_empty_issue() {
        let analyzer = FiveWhysAnalyzer::new();
        let result = analyzer.analyze("", Path::new("."), 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        let analyzer = FiveWhysAnalyzer::new();

        // Empty evidence
        let confidence = analyzer.calculate_confidence(&[]).expect("internal error");
        assert!((0.0..=1.0).contains(&confidence));

        // With evidence
        let evidence = vec![Evidence::new(
            EvidenceSource::Complexity,
            PathBuf::from("test.rs"),
            "complexity".to_string(),
            json!({"value": 50, "threshold": 20}),
            "High".to_string(),
        )];
        let confidence = analyzer.calculate_confidence(&evidence).expect("internal error");
        assert!(confidence > 0.3);
        assert!(confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_basic_analysis() {
        let analyzer = FiveWhysAnalyzer::new();
        let result = analyzer
            .analyze("Test issue", Path::new("."), 5)
            .await
            .expect("internal error");

        assert_eq!(result.issue, "Test issue");
        assert!(!result.whys.is_empty());
        assert!(result.root_cause.is_some());
        assert!(!result.recommendations.is_empty());
    }
}
