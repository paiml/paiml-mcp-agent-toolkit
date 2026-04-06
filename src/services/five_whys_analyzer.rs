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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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

    /// Gather evidence from real project data (v2 weights, PMAT-510).
    ///
    /// v2 evidence sources: Complexity (25%), SATD (20%), Git churn (15%),
    /// EvoScore trajectory (15%), Coverage delta (15%), Dead code (10%).
    /// TDG removed (redundant with complexity+churn).
    async fn gather_evidence(&self, path: &Path) -> Result<Vec<Evidence>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let mut evidence = Vec::new();

        // Real SATD evidence: count TODO/FIXME/HACK/WORKAROUND in source
        if let Some(satd_ev) = Self::gather_satd_evidence(path) {
            evidence.push(satd_ev);
        }

        // Real Git churn evidence: count commits in last 30 days
        if let Some(churn_ev) = Self::gather_git_churn_evidence(path) {
            evidence.push(churn_ev);
        }

        // Real complexity evidence: count Rust source files and estimate complexity
        if let Some(cx_ev) = Self::gather_complexity_evidence(path) {
            evidence.push(cx_ev);
        }

        // EvoScore trajectory (CB-142): is the affected area improving or regressing?
        if let Some(evo_ev) = Self::gather_evoscore_evidence(path) {
            evidence.push(evo_ev);
        }

        // Coverage delta: did recent changes decrease test coverage?
        if let Some(cov_ev) = Self::gather_coverage_delta_evidence(path) {
            evidence.push(cov_ev);
        }

        Ok(evidence)
    }

    /// Count SATD markers (TODO, FIXME, HACK, WORKAROUND, XXX) in source files.
    fn gather_satd_evidence(path: &Path) -> Option<Evidence> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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

    // NOTE: gather_tdg_evidence removed in v2 (PMAT-510).
    // TDG weight set to 0% — redundant with complexity + churn.
    // EvidenceSource::TDG variant kept for backward compat (deserialization).

    /// Estimate complexity by counting Rust source lines and deeply-nested functions.
    fn gather_complexity_evidence(path: &Path) -> Option<Evidence> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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

    /// Compute EvoScore trajectory from .pmat-metrics/ test data (CB-142).
    ///
    /// Uses the same gamma-weighted computation as `check_swe_ci_evoscore`.
    /// Returns None (neutral) if insufficient data (<3 commits).
    fn gather_evoscore_evidence(path: &Path) -> Option<Evidence> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let metrics_dir = path.join(".pmat-metrics");
        if !metrics_dir.exists() {
            return None;
        }

        // Collect commit test data files
        let mut test_files: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&metrics_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("commit-") && name.ends_with("-tests.json") {
                        test_files.push(p);
                    }
                }
            }
        }
        test_files.sort();

        let mut test_data: Vec<(u64, u64)> = Vec::new(); // (pass, total)
        for file_path in &test_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let pass = data["pass"].as_u64().unwrap_or(0);
                    let total = data["total"].as_u64().unwrap_or(0);
                    if total > 0 {
                        test_data.push((pass, total));
                    }
                }
            }
        }

        // Need at least 3 commits for meaningful trajectory
        if test_data.len() < 3 {
            return None;
        }

        // Compute EvoScore with gamma = 1.5 (matches CB-142 comply check)
        let gamma: f64 = 1.5;
        let base_pass = test_data[0].0 as f64;
        let oracle_pass = test_data.iter().map(|(p, _)| *p).max().unwrap_or(0) as f64;

        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for (i, (pass, _total)) in test_data.iter().enumerate().skip(1) {
            let current_pass = *pass as f64;
            let a_c = if current_pass >= base_pass {
                let gap = oracle_pass - base_pass;
                if gap > 0.0 {
                    (current_pass - base_pass) / gap
                } else {
                    1.0
                }
            } else if base_pass > 0.0 {
                (current_pass - base_pass) / base_pass
            } else {
                0.0
            };

            let weight = gamma.powi(i as i32);
            weighted_sum += weight * a_c;
            weight_total += weight;
        }

        let evoscore = if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.0
        };

        let description = if evoscore >= 0.5 {
            format!(
                "Positive trajectory: EvoScore {:.3} — area is improving",
                evoscore
            )
        } else if evoscore >= 0.0 {
            format!(
                "Mixed trajectory: EvoScore {:.3} — some improvement, some regression",
                evoscore
            )
        } else {
            format!(
                "Negative trajectory: EvoScore {:.3} — area is regressing",
                evoscore
            )
        };

        Some(Evidence::new(
            EvidenceSource::EvoScoreTrajectory,
            path.to_path_buf(),
            "evoscore_trajectory".to_string(),
            json!({"evoscore": evoscore, "commits": test_data.len(), "gamma": gamma}),
            description,
        ))
    }

    /// Compute coverage delta from .pmat/coverage-cache.json.
    ///
    /// Reads cached coverage data and computes a simple coverage ratio.
    /// Returns None if no coverage data is available.
    fn gather_coverage_delta_evidence(path: &Path) -> Option<Evidence> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let cache_path = path.join(".pmat/coverage-cache.json");
        let content = std::fs::read_to_string(&cache_path).ok()?;
        let data: serde_json::Value = serde_json::from_str(&content).ok()?;

        let files = data.get("files")?.as_object()?;
        if files.is_empty() {
            return None;
        }

        // Compute aggregate coverage from line hit data
        let mut total_lines: usize = 0;
        let mut covered_lines: usize = 0;

        for (_file_path, line_hits) in files {
            if let Some(hits_map) = line_hits.as_object() {
                for (_line_no, hit_count) in hits_map {
                    total_lines += 1;
                    if hit_count.as_u64().unwrap_or(0) > 0 {
                        covered_lines += 1;
                    }
                }
            }
        }

        let coverage_pct = if total_lines > 0 {
            covered_lines as f64 / total_lines as f64 * 100.0
        } else {
            return None;
        };

        // Delta: compare against 85% baseline (industry standard target)
        // Positive delta = above target, negative = below target
        let delta = coverage_pct - 85.0;

        let description = if delta >= 0.0 {
            format!(
                "Coverage {:.1}% (delta +{:.1}% vs 85% baseline) — above target",
                coverage_pct, delta
            )
        } else {
            format!(
                "Coverage {:.1}% (delta {:.1}% vs 85% baseline) — below target",
                coverage_pct, delta
            )
        };

        Some(Evidence::new(
            EvidenceSource::CoverageDelta,
            path.to_path_buf(),
            "coverage_delta".to_string(),
            json!({"coverage_pct": coverage_pct, "delta": delta, "total_lines": total_lines, "covered_lines": covered_lines}),
            description,
        ))
    }

    fn count_lines_and_nesting(dir: &Path) -> (usize, usize) {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
    high_churn: bool,
    regressing_evoscore: bool,
    low_coverage: bool,
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
            high_churn: evidence.iter().any(|e| {
                e.source == EvidenceSource::GitChurn
                    && e.value
                        .get("commit_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        > 10
            }),
            regressing_evoscore: evidence.iter().any(|e| {
                e.source == EvidenceSource::EvoScoreTrajectory
                    && e.value
                        .get("evoscore")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                        < 0.0
            }),
            low_coverage: evidence.iter().any(|e| {
                e.source == EvidenceSource::CoverageDelta
                    && e.value.get("delta").and_then(|v| v.as_f64()).unwrap_or(0.0) < 0.0
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
        if self.low_coverage {
            "Insufficient test coverage allowed defect to slip through".to_string()
        } else if self.high_complexity {
            "Complex control flow makes code difficult to understand and maintain".to_string()
        } else {
            "Code structure contributed to the problem".to_string()
        }
    }

    fn depth_3_hypothesis(&self) -> String {
        if self.regressing_evoscore {
            "Quality trajectory is declining — area has been getting worse over time".to_string()
        } else if self.high_churn {
            "Frequent changes indicate unstable or poorly understood code".to_string()
        } else if self.satd_present {
            "Technical debt accumulated, indicating deferred maintenance".to_string()
        } else {
            "Architectural constraints led to current state".to_string()
        }
    }
}

impl FiveWhysAnalyzer {
    /// Calculate confidence score based on evidence strength (v2 weights, PMAT-510).
    ///
    /// v2 weights: Complexity 25%, SATD 20%, GitChurn 15%,
    /// EvoScoreTrajectory 15%, CoverageDelta 15%, DeadCode 10%.
    /// TDG weight removed (0%) — redundant with complexity+churn.
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
                // v2: TDG removed (redundant with complexity+churn). Weight = 0.
                EvidenceSource::TDG => (0.0, 1.0),
                EvidenceSource::GitChurn => {
                    let commits = ev
                        .value
                        .get("commit_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let severity = (commits as f64).min(20.0) / 20.0;
                    (0.15, 1.0 + severity)
                }
                EvidenceSource::DeadCode => (0.10, 1.0),
                EvidenceSource::ManualInspection => (0.15, 1.0),
                EvidenceSource::EvoScoreTrajectory => {
                    let evoscore = ev
                        .value
                        .get("evoscore")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    // Negative evoscore = regressing = higher severity
                    // Positive evoscore = improving = lower severity
                    let severity = if evoscore < 0.0 {
                        1.0 + (-evoscore).min(1.0) // Regression amplifies confidence
                    } else {
                        1.0 // Improvement is neutral
                    };
                    (0.15, severity)
                }
                EvidenceSource::CoverageDelta => {
                    let delta = ev
                        .value
                        .get("delta")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    // Negative delta = below 85% baseline = higher severity
                    let severity = if delta < 0.0 {
                        1.0 + (-delta / 85.0).min(1.0) // Scale by baseline
                    } else {
                        1.0
                    };
                    (0.15, severity)
                }
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

    /// Generate actionable recommendations (v2 evidence sources, PMAT-510)
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

        let has_regressing_evoscore = whys.iter().any(|w| {
            w.evidence.iter().any(|e| {
                e.source == EvidenceSource::EvoScoreTrajectory
                    && e.value
                        .get("evoscore")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                        < 0.0
            })
        });

        let has_low_coverage = whys.iter().any(|w| {
            w.evidence.iter().any(|e| {
                e.source == EvidenceSource::CoverageDelta
                    && e.value.get("delta").and_then(|v| v.as_f64()).unwrap_or(0.0) < 0.0
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

        if has_low_coverage {
            recommendations.push(Recommendation::high(
                "Add comprehensive test coverage (target: >=85%) using EXTREME TDD".to_string(),
                None,
            ));
        }

        if has_regressing_evoscore {
            recommendations.push(Recommendation::high(
                "Quality trajectory is declining — investigate and reverse regression trend"
                    .to_string(),
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

// Design-by-contract specifications (Verus-style)
// #[requires(project_path.is_dir())]
// #[ensures(result.is_ok() ==> ret.len() > 0)]
