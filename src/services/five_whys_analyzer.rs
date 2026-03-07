#![cfg_attr(coverage_nightly, coverage(off))]
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

    /// Gather evidence from real project data.
    /// Scans SATD markers, measures git churn, reads TDG baseline, and estimates complexity.
    async fn gather_evidence(&self, path: &Path) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        // Real SATD evidence: count TODO/FIXME/HACK/WORKAROUND in source
        if let Some(satd_ev) = Self::gather_satd_evidence(path) {
            evidence.push(satd_ev);
        }

        // Real Git churn evidence: count commits in last 30 days
        if let Some(churn_ev) = Self::gather_git_churn_evidence(path) {
            evidence.push(churn_ev);
        }

        // Real TDG evidence: read baseline if available
        if let Some(tdg_ev) = Self::gather_tdg_evidence(path) {
            evidence.push(tdg_ev);
        }

        // Real complexity evidence: count Rust source files and estimate complexity
        if let Some(cx_ev) = Self::gather_complexity_evidence(path) {
            evidence.push(cx_ev);
        }

        Ok(evidence)
    }

    /// Count SATD markers (TODO, FIXME, HACK, WORKAROUND, XXX) in source files.
    fn gather_satd_evidence(path: &Path) -> Option<Evidence> {
        let src_dir = path.join("src");
        let dir = if src_dir.is_dir() { &src_dir } else { path };
        let count = Self::count_satd_markers(dir);
        let description = if count == 0 {
            "No SATD markers found — codebase is clean of admitted technical debt".to_string()
        } else {
            format!(
                "Found {} TODO/FIXME/HACK markers indicating known technical debt",
                count
            )
        };
        Some(Evidence::new(
            EvidenceSource::SATD,
            path.to_path_buf(),
            "todo_markers".to_string(),
            json!({"count": count}),
            description,
        ))
    }

    const SATD_EXTENSIONS: &'static [&'static str] =
        &["rs", "py", "ts", "js", "go", "lua", "c", "cpp", "java"];
    const SATD_MARKERS: &'static [&'static str] = &["TODO", "FIXME", "HACK", "WORKAROUND", "XXX"];

    fn count_satd_markers(dir: &Path) -> usize {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return 0,
        };
        entries
            .flatten()
            .map(|entry| entry.path())
            .map(|p| {
                if p.is_dir() {
                    return Self::count_satd_markers(&p);
                }
                let is_source = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| Self::SATD_EXTENSIONS.contains(&e));
                if !is_source {
                    return 0;
                }
                std::fs::read_to_string(&p)
                    .unwrap_or_default()
                    .lines()
                    .filter(|line| Self::SATD_MARKERS.iter().any(|m| line.contains(m)))
                    .count()
            })
            .sum()
    }

    /// Count git commits in last 30 days.
    fn gather_git_churn_evidence(path: &Path) -> Option<Evidence> {
        let output = std::process::Command::new("git")
            .args(["rev-list", "--count", "--since=30.days", "HEAD"])
            .current_dir(path)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let count: u64 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0);
        let description = if count > 20 {
            format!(
                "High churn: {} commits in 30 days indicates active/unstable area",
                count
            )
        } else if count > 5 {
            format!("Moderate churn: {} commits in 30 days", count)
        } else {
            format!("Low churn: {} commits in 30 days — stable code", count)
        };
        Some(Evidence::new(
            EvidenceSource::GitChurn,
            path.to_path_buf(),
            "commit_count".to_string(),
            json!({"commit_count": count, "days": 30}),
            description,
        ))
    }

    /// Read TDG baseline average score if available.
    fn gather_tdg_evidence(path: &Path) -> Option<Evidence> {
        let baseline_path = path.join(".pmat/baseline.json");
        let content = std::fs::read_to_string(&baseline_path).ok()?;
        let data: serde_json::Value = serde_json::from_str(&content).ok()?;
        let avg_score = data
            .get("summary")
            .and_then(|s| s.get("avg_score"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let total_files = data
            .get("summary")
            .and_then(|s| s.get("total_files"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let description = if avg_score >= 80.0 {
            format!(
                "Strong TDG baseline: {:.1}/100 across {} files",
                avg_score, total_files
            )
        } else if avg_score >= 50.0 {
            format!(
                "Moderate TDG score: {:.1}/100 across {} files — room for improvement",
                avg_score, total_files
            )
        } else if total_files == 0 {
            "No TDG baseline available — run `pmat tdg` to generate".to_string()
        } else {
            format!(
                "Low TDG score: {:.1}/100 across {} files — significant debt",
                avg_score, total_files
            )
        };
        Some(Evidence::new(
            EvidenceSource::TDG,
            path.to_path_buf(),
            "tdg_score".to_string(),
            json!(avg_score),
            description,
        ))
    }

    /// Estimate complexity by counting Rust source lines and deeply-nested functions.
    fn gather_complexity_evidence(path: &Path) -> Option<Evidence> {
        let src_dir = path.join("src");
        if !src_dir.is_dir() {
            return None;
        }
        let (total_lines, deep_nesting_count) = Self::count_lines_and_nesting(&src_dir);
        let estimated_avg_complexity = if total_lines > 0 {
            // Rough heuristic: deep nesting count per 1000 lines
            (deep_nesting_count as f64 / total_lines as f64 * 1000.0).round() as u64
        } else {
            0
        };
        let description = format!(
            "{} source lines, {} deeply-nested blocks (est. complexity density: {}/1000 lines)",
            total_lines, deep_nesting_count, estimated_avg_complexity
        );
        Some(Evidence::new(
            EvidenceSource::Complexity,
            path.to_path_buf(),
            "estimated_complexity".to_string(),
            json!({"total_lines": total_lines, "deep_nesting": deep_nesting_count, "threshold": 20}),
            description,
        ))
    }

    fn count_lines_and_nesting(dir: &Path) -> (usize, usize) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return (0, 0),
        };
        entries
            .flatten()
            .map(|entry| entry.path())
            .fold((0usize, 0usize), |(lines, deep), p| {
                if p.is_dir() {
                    let (l, d) = Self::count_lines_and_nesting(&p);
                    return (lines + l, deep + d);
                }
                let is_rs = p.extension().and_then(|e| e.to_str()) == Some("rs");
                if !is_rs {
                    return (lines, deep);
                }
                let (l, d) = Self::count_file_nesting(&p);
                (lines + l, deep + d)
            })
    }

    fn count_file_nesting(path: &Path) -> (usize, usize) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return (0, 0),
        };
        let mut brace_depth = 0i32;
        let mut deep = 0usize;
        let mut line_count = 0usize;
        for line in content.lines() {
            line_count += 1;
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;
            if brace_depth > 5 {
                deep += 1;
            }
        }
        (line_count, deep)
    }

    /// Generate hypothesis based on evidence
    fn generate_hypothesis(
        &self,
        _question: &str,
        evidence: &[Evidence],
        depth: u8,
    ) -> Result<String> {
        let signals = EvidenceSignals::from_evidence(evidence);
        Ok(signals.hypothesis_for_depth(depth))
    }
}

/// Extracted evidence signals to reduce cognitive complexity in hypothesis generation.
struct EvidenceSignals {
    high_complexity: bool,
    satd_present: bool,
    low_tdg: bool,
    high_churn: bool,
}

impl EvidenceSignals {
    fn from_evidence(evidence: &[Evidence]) -> Self {
        Self {
            high_complexity: evidence.iter().any(|e| {
                e.source == EvidenceSource::Complexity
                    && e.value
                        .get("deep_nesting")
                        .or_else(|| e.value.get("value"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                        > 20.0
            }),
            satd_present: evidence.iter().any(|e| e.source == EvidenceSource::SATD),
            low_tdg: evidence.iter().any(|e| {
                e.source == EvidenceSource::TDG && e.value.as_f64().unwrap_or(100.0) < 50.0
            }),
            high_churn: evidence.iter().any(|e| {
                e.source == EvidenceSource::GitChurn
                    && e.value
                        .get("commit_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        > 10
            }),
        }
    }

    fn hypothesis_for_depth(&self, depth: u8) -> String {
        match depth {
            1 => self.depth_1_hypothesis(),
            2 => self.depth_2_hypothesis(),
            3 => self.depth_3_hypothesis(),
            4 => "Requirements or constraints were not fully specified".to_string(),
            _ => "Root cause: Systematic process gap in development workflow".to_string(),
        }
    }

    fn depth_1_hypothesis(&self) -> String {
        if self.high_complexity {
            "Code complexity exceeds acceptable thresholds".to_string()
        } else if self.satd_present {
            "Known technical debt markers present in codebase".to_string()
        } else {
            "Issue manifested due to code quality factors".to_string()
        }
    }

    fn depth_2_hypothesis(&self) -> String {
        if self.low_tdg {
            "Insufficient test coverage allowed defect to slip through".to_string()
        } else if self.high_complexity {
            "Complex control flow makes code difficult to understand and maintain".to_string()
        } else {
            "Code structure contributed to the problem".to_string()
        }
    }

    fn depth_3_hypothesis(&self) -> String {
        if self.high_churn {
            "Frequent changes indicate unstable or poorly understood code".to_string()
        } else if self.satd_present {
            "Technical debt accumulated, indicating deferred maintenance".to_string()
        } else {
            "Architectural constraints led to current state".to_string()
        }
    }
}

impl FiveWhysAnalyzer {
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
                    // Accept both "deep_nesting" (real evidence) and "value" (legacy/tests)
                    let metric = ev
                        .value
                        .get("deep_nesting")
                        .or_else(|| ev.value.get("value"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let threshold = ev
                        .value
                        .get("threshold")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(20.0);
                    let severity = if threshold > 0.0 {
                        (metric - threshold).max(0.0) / threshold
                    } else {
                        0.0
                    };
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

// Tests extracted to five_whys_analyzer_tests.rs for file health (CB-040).
include!("five_whys_analyzer_tests.rs");
