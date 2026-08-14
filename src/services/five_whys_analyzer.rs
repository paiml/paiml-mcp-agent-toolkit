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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

            // Early termination if high confidence reached (>0.9) after at
            // least 3 iterations. Say so: the caller asked for `depth` whys and
            // is getting fewer, and silence made `--depth` look inert (#962).
            if i >= 3 && why.confidence > 0.9 {
                let confidence = why.confidence;
                analysis.whys.push(why);
                if i < depth {
                    analysis.stopped_early = Some(format!(
                        "converged after {i} of {depth} whys: confidence {confidence:.2} exceeded 0.90"
                    ));
                }
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
        let evidence = self.gather_evidence(issue, path).await?;

        // Generate hypothesis based on evidence
        let hypothesis = self.generate_hypothesis(&question, &evidence, depth)?;

        // Calculate confidence.
        //
        // A hypothesis the report itself tags "repo-level signal" cannot carry a
        // confident causal claim about the reported issue, for exactly the
        // reason `NO_ISSUE_EVIDENCE_CEILING` exists: the rungs of the ladder
        // below localisation are keyed on churn/SATD/coverage, which are
        // identical whatever issue was typed. Stamping one 100.0% directly above
        // the sentence disclaiming it ("Beyond localisation no causal chain was
        // derived") gave a reader who trusts the number a maximally-confident
        // non-answer (#962).
        //
        // This is also what made `--depth` inert. Every severity scale saturated
        // on any real repository, so confidence was exactly 1.0, so the
        // `i >= 3 && confidence > 0.9` early exit fired on iteration 3 every
        // time: `--depth 5`, `7` and `10` all returned three whys. Capping the
        // repo-level rungs below the threshold means depth is honoured again,
        // without weakening the early exit where it is meaningful — a located,
        // issue-specific hypothesis is not capped.
        let confidence = self.calculate_confidence(&evidence)?;
        let confidence = if hypothesis.contains(Self::REPO_LEVEL_TAG) {
            confidence.min(Self::REPO_LEVEL_CEILING)
        } else {
            confidence
        };

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
    async fn gather_evidence(&self, issue: &str, path: &Path) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        // Evidence about the reported issue. Everything below this line is a
        // repo-wide metric that is identical whatever the issue was, so this is
        // the only source that makes the analysis about the question asked.
        if let Some(loc_ev) = Self::gather_issue_location_evidence(issue, path) {
            evidence.push(loc_ev);
        }

        // Real SATD evidence, from the SAME detector `pmat analyze satd` uses.
        if let Some(satd_ev) = Self::gather_satd_evidence(path).await {
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

    /// Count SATD markers using the detector `pmat analyze satd` uses.
    ///
    /// This used to be `count_satd_markers`, a raw recursive substring scan for
    /// TODO/FIXME/HACK/WORKAROUND/XXX with no comment awareness and no
    /// exclusions: it reported 808 markers for this repo while
    /// `pmat analyze satd` reported 39 for the same path in the same session —
    /// a 20x disagreement between two surfaces of one binary. It was counting
    /// the detector's own pattern tables, test fixtures and doc prose, and then
    /// presenting the total as measured technical-debt evidence for the
    /// root-cause chain.
    async fn gather_satd_evidence(path: &Path) -> Option<Evidence> {
        use crate::services::satd_detector::SATDDetector;

        let result = SATDDetector::new()
            .analyze_project(path, false)
            .await
            .ok()?;
        let count = result.summary.total_items;
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

    /// Source extensions scanned when locating the reported issue.
    const SATD_EXTENSIONS: &'static [&'static str] =
        &["rs", "py", "ts", "js", "go", "lua", "c", "cpp", "java"];

    /// Words too common to identify anything, dropped from issue terms.
    const ISSUE_STOPWORDS: &'static [&'static str] = &[
        "the", "this", "that", "with", "when", "from", "into", "have", "does", "than", "then",
        "them", "they", "there", "which", "while", "will", "would", "could", "should", "been",
        "being", "before", "after", "because", "always", "never", "error", "fails", "failed",
        "failure", "issue", "problem", "broken", "wrong", "silent", "silently", "reported",
        "returns", "return", "value", "values", "code", "test", "tests",
    ];

    /// Most matching locations to report; enough to orient, few enough to read.
    const MAX_ISSUE_LOCATIONS: usize = 12;

    /// Highest confidence an analysis may claim when it never located the issue.
    ///
    /// Repo-wide metrics describe the repository, not the defect. Collecting
    /// more of them must not raise confidence in a causal claim about a
    /// specific issue.
    pub const NO_ISSUE_EVIDENCE_CEILING: f64 = 0.35;

    /// Marker the hypothesis ladder puts on every rung below localisation.
    ///
    /// One spelling, used by the renderer, by `extract_root_cause` and by the
    /// confidence ceiling — three readers of one rule, which is how this
    /// codebase's recurring defect starts when they are allowed to drift.
    pub const REPO_LEVEL_TAG: &'static str = "repo-level signal";

    /// Highest confidence a rung tagged [`Self::REPO_LEVEL_TAG`] may claim.
    ///
    /// Deliberately below the `> 0.9` early-exit threshold in [`Self::analyze`]:
    /// a chain of repo-wide signals must not be able to terminate the analysis
    /// by looking certain. Above [`Self::NO_ISSUE_EVIDENCE_CEILING`] because
    /// these runs did locate the issue — the localisation is real evidence, the
    /// rungs built on top of it are not.
    pub const REPO_LEVEL_CEILING: f64 = 0.60;

    /// Severity that rises with `count` and never quite reaches 1.0.
    ///
    /// `count / (count + half)` — `half` is the value scoring 0.5. The scales
    /// this replaces were hard clamps (`count.min(10.0) / 10.0`), and every one
    /// of them saturated on any real repository: pmat reports 62 SATD markers
    /// against a cap of 10, 29 commits against 20, 12 matched locations against
    /// 6. A 10-marker repo and a 62-marker repo scored identically at 1.0, so
    /// the metric could not discriminate between the codebases it was there to
    /// compare — and with every severity pinned at 1.0 the weighted mean was
    /// exactly 1.0, which is what tripped the early exit (#962).
    ///
    /// Monotone, so more evidence is still more severity, and asymptotic, so no
    /// finite amount of repo-wide signal alone reaches certainty.
    fn saturating_severity(count: f64, half: f64) -> f64 {
        if count <= 0.0 || half <= 0.0 {
            return 0.0;
        }
        count / (count + half)
    }

    /// Did any evidence actually pertain to the reported issue?
    fn has_issue_evidence(evidence: &[Evidence]) -> bool {
        evidence
            .iter()
            .any(|e| e.source == EvidenceSource::IssueLocation)
    }

    /// Distinctive terms from the issue text, used to locate relevant source.
    ///
    /// Keeps tokens of 4+ characters that are not stopwords, so
    /// "MCP stdio server drops responses when stdin reaches EOF" yields
    /// `stdio`, `server`, `drops`, `responses`, `stdin`, `reaches` — terms that
    /// can actually be found in code.
    fn issue_terms(issue: &str) -> Vec<String> {
        let mut terms: Vec<String> = issue
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|t| t.len() >= 4)
            .map(str::to_lowercase)
            .filter(|t| !Self::ISSUE_STOPWORDS.contains(&t.as_str()))
            .collect();
        terms.sort();
        terms.dedup();
        terms
    }

    /// Locate source lines mentioning the issue's distinctive terms.
    ///
    /// This is the only evidence source tied to the *reported issue*; every
    /// other one is a repo-wide metric identical for any input. Before this
    /// existed, asking about an EOF race in the MCP transport produced the same
    /// four repo metrics as asking about anything else, and the analysis
    /// concluded "Frequent changes indicate unstable or poorly understood code"
    /// at 100% confidence — a statement about the repository, not the defect
    /// (GH #637).
    ///
    /// Returns `None` when nothing matches, which callers must treat as "the
    /// issue was not located" rather than as an absence of problems.
    fn gather_issue_location_evidence(issue: &str, path: &Path) -> Option<Evidence> {
        let terms = Self::issue_terms(issue);
        if terms.is_empty() {
            return None;
        }
        let src_dir = path.join("src");
        let dir = if src_dir.is_dir() { &src_dir } else { path };

        let mut hits = Vec::new();
        Self::collect_term_matches(dir, &terms, &mut hits);
        if hits.is_empty() {
            return None;
        }

        // Rank by how many distinct issue terms a file matches: a file
        // mentioning several of them is likelier to be the subject.
        hits.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        hits.truncate(Self::MAX_ISSUE_LOCATIONS);

        let locations: Vec<serde_json::Value> = hits
            .iter()
            .map(|(file, line, score, term)| {
                json!({"file": file, "line": line, "terms_matched": score, "term": term})
            })
            .collect();
        let top = hits
            .iter()
            .take(3)
            .map(|(f, l, _, _)| format!("{f}:{l}"))
            .collect::<Vec<_>>()
            .join(", ");

        Some(Evidence::new(
            EvidenceSource::IssueLocation,
            path.to_path_buf(),
            "issue_terms".to_string(),
            json!({"terms": terms, "locations": locations}),
            format!(
                "Located {} source line(s) matching issue terms; strongest: {top}",
                hits.len()
            ),
        ))
    }

    /// Walk `dir`, recording `(file, line_no, distinct_terms_on_line, term)`.
    fn collect_term_matches(
        dir: &Path,
        terms: &[String],
        out: &mut Vec<(String, usize, usize, String)>,
    ) {
        // Bail once we have plenty; this walks a whole source tree.
        if out.len() >= Self::MAX_ISSUE_LOCATIONS * 8 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                Self::collect_term_matches(&p, terms, out);
                continue;
            }
            let is_source = p
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| Self::SATD_EXTENSIONS.contains(&e));
            if !is_source {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&p) else {
                continue;
            };
            for (idx, line) in content.lines().enumerate() {
                let lower = line.to_lowercase();
                let matched: Vec<&String> = terms.iter().filter(|t| lower.contains(*t)).collect();
                // Two or more distinct issue terms on one line is a real
                // signal; one is usually an incidental word.
                if matched.len() >= 2 {
                    out.push((
                        p.display().to_string(),
                        idx + 1,
                        matched.len(),
                        matched
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join("+"),
                    ));
                }
            }
        }
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

    // NOTE: gather_tdg_evidence removed in v2 (PMAT-510).
    // TDG weight set to 0% — redundant with complexity + churn.
    // EvidenceSource::TDG variant kept for backward compat (deserialization).

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
            // `value` is the key `EvidenceSummary::process_complexity_evidence`
            // reads. This payload used to carry only total_lines/deep_nesting/
            // threshold, so the consumer's `value` lookup fell back to 0.0 and
            // `0.0 > 20.0` could never fire: `complexity_violations` was
            // structurally 0 for every path — an empty directory and this
            // 979k-line repo reported the same 0 while the evidence text beside
            // it quoted a density of 17/1000 lines.
            json!({
                "value": estimated_avg_complexity,
                "threshold": 20,
                "total_lines": total_lines,
                "deep_nesting": deep_nesting_count,
            }),
            description,
        ))
    }

    /// Compute EvoScore trajectory from .pmat-metrics/ test data (CB-142).
    ///
    /// Uses the same gamma-weighted computation as `check_swe_ci_evoscore`.
    /// Returns None (neutral) if insufficient data (<3 commits).
    fn gather_evoscore_evidence(path: &Path) -> Option<Evidence> {
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
    high_churn: bool,
    regressing_evoscore: bool,
    low_coverage: bool,
    /// `(file, total_locations)` for the strongest issue-term match, when the
    /// issue was located at all. Hypotheses must cite this when present:
    /// without it they describe the repository, not the reported defect.
    located: Option<(String, usize)>,
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
            located: evidence
                .iter()
                .find(|e| e.source == EvidenceSource::IssueLocation)
                .and_then(|e| {
                    let locs = e.value.get("locations")?.as_array()?;
                    let first = locs.first()?;
                    let file = first.get("file")?.as_str()?.to_string();
                    let line = first.get("line").and_then(serde_json::Value::as_u64);
                    Some((
                        line.map_or(file.clone(), |l| format!("{file}:{l}")),
                        locs.len(),
                    ))
                }),
        }
    }

    fn hypothesis_for_depth(&self, depth: u8) -> String {
        // When the issue was located, lead with that: it is the only statement
        // here derived from the issue rather than from repo-wide metrics. The
        // deeper rungs remain repo-level and say so, instead of being presented
        // as the cause of a specific defect (GH #637).
        if let Some((where_, count)) = &self.located {
            return match depth {
                1 => format!(
                    "The issue's terms concentrate at {where_} ({count} matching location(s)) — \
                     the defect most likely originates in that code path"
                ),
                2 => format!(
                    "{} (repo-level signal, not specific to {where_})",
                    self.depth_2_hypothesis()
                ),
                3 => format!(
                    "{} (repo-level signal, not specific to {where_})",
                    self.depth_3_hypothesis()
                ),
                4 => "Requirements or constraints were not fully specified (repo-level signal)"
                    .to_string(),
                _ => {
                    "Systematic process gap in development workflow (repo-level signal)".to_string()
                }
            };
        }
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
    ///
    /// # Two corrections (GH #637)
    ///
    /// **The score could only ever be 1.0.** Each source contributed
    /// `weight * (1.0 + severity)`, and the total was divided by the sum of the
    /// weights alone — so the ratio was always at least 1.0 and the final
    /// `clamp(0.0, 1.0)` pinned it to exactly 100%. Severity is now in `[0, 1]`
    /// so the score genuinely varies. `test_calculate_confidence_increases_with
    /// _severity` asserted only `high >= low`, which `1.0 >= 1.0` satisfied, so
    /// it could not detect this; it now asserts strict inequality.
    ///
    /// **Confidence measured volume, not relevance.** Every source except
    /// [`EvidenceSource::IssueLocation`] is a repo-wide metric that is identical
    /// whatever issue was reported. Collecting five of them said nothing about
    /// the question asked, yet produced 100%. Without at least one
    /// issue-specific location the score is now capped at
    /// [`Self::NO_ISSUE_EVIDENCE_CEILING`].
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
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
                    (0.25, severity.min(1.0))
                }
                EvidenceSource::SATD => {
                    let count = ev.value.get("count").and_then(|v| v.as_u64()).unwrap_or(1);
                    (0.20, Self::saturating_severity(count as f64, 10.0))
                }
                // v2: TDG removed (redundant with complexity+churn). Weight = 0.
                EvidenceSource::TDG => (0.0, 0.0),
                EvidenceSource::GitChurn => {
                    let commits = ev
                        .value
                        .get("commit_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    (0.15, Self::saturating_severity(commits as f64, 20.0))
                }
                EvidenceSource::DeadCode => (0.10, 0.5),
                EvidenceSource::ManualInspection => (0.15, 1.0),
                EvidenceSource::EvoScoreTrajectory => {
                    let evoscore = ev
                        .value
                        .get("evoscore")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    // Negative evoscore = regressing = higher severity
                    // Positive evoscore = improving = lower severity
                    // Regression is a stronger signal than improvement, but
                    // neither is evidence about a specific reported issue.
                    let severity = if evoscore < 0.0 {
                        (-evoscore).min(1.0)
                    } else {
                        0.0
                    };
                    (0.15, severity)
                }
                EvidenceSource::IssueLocation => {
                    // Severity scales with how many locations matched two or
                    // more distinct issue terms.
                    let found = ev
                        .value
                        .get("locations")
                        .and_then(|v| v.as_array())
                        .map_or(0, Vec::len);
                    (0.35, Self::saturating_severity(found as f64, 6.0))
                }
                EvidenceSource::CoverageDelta => {
                    let delta = ev
                        .value
                        .get("delta")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    // Negative delta = below 85% baseline = higher severity
                    let severity = if delta < 0.0 {
                        (-delta / 85.0).min(1.0) // Scale by baseline
                    } else {
                        0.0
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

        // Repo-wide metrics alone cannot support a confident causal claim about
        // a specific issue, however many of them were collected.
        if Self::has_issue_evidence(evidence) {
            Ok(normalized)
        } else {
            Ok(normalized.min(Self::NO_ISSUE_EVIDENCE_CEILING))
        }
    }

    /// Extract root cause from Why iterations
    /// Extract root cause from Why iterations.
    ///
    /// Returns `None` when no evidence pertained to the reported issue. The
    /// final hypothesis is drawn from a fixed ladder keyed on repo-wide
    /// metrics, so presenting it as *the root cause* of an unlocated issue
    /// states a conclusion that was never derived — this is what produced
    /// "Frequent changes indicate unstable or poorly understood code" as the
    /// root cause of an EOF race in the MCP transport (GH #637).
    ///
    /// Withholding follows the precedent set by
    /// `FalsificationResult::unmeasured()` in v3.26.0: a receipt that says
    /// nothing is better than one that overstates.
    fn extract_root_cause(&self, whys: &[WhyIteration]) -> Result<Option<String>> {
        let Some(last_why) = whys.last() else {
            return Ok(None);
        };

        let located = whys
            .iter()
            .any(|why| Self::has_issue_evidence(&why.evidence));
        if !located {
            return Ok(None);
        }

        // Report the deepest hypothesis that was actually derived from the
        // issue. The deeper rungs of the ladder are repo-wide signals (churn,
        // SATD, coverage) tagged as such; presenting one of those as "the root
        // cause" is what made an EOF race read as "Frequent changes indicate
        // unstable or poorly understood code" (GH #637).
        let derived = whys
            .iter()
            .rev()
            .find(|why| !why.hypothesis.contains(Self::REPO_LEVEL_TAG))
            .unwrap_or(last_why);

        if derived.hypothesis.contains(Self::REPO_LEVEL_TAG) {
            return Ok(None);
        }

        Ok(Some(format!(
            "{}\n\nBeyond localisation no causal chain was derived: the remaining \
             \"why\" steps are repo-wide signals, not findings about this defect. \
             Confirm by reading the cited locations.",
            derived.hypothesis
        )))
    }

    /// Generate actionable recommendations (v2 evidence sources, PMAT-510)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        // Root cause fix recommendation — only when there *is* one.
        //
        // `analyze` passes `root_cause.unwrap_or_default()`, so a withheld cause
        // arrives here as an empty string and this printed a bare
        // "Address root cause: " with nothing after it. When the issue could not
        // be located, the actionable advice is to make it locatable.
        if root_cause.trim().is_empty() {
            recommendations.push(Recommendation::high(
                "No root cause was determined — the reported issue could not be \
                 located in the source. Re-run with terms that appear in the code \
                 (identifiers, module or file names, a log string)."
                    .to_string(),
                None,
            ));
        } else {
            recommendations.push(Recommendation::high(
                format!("Address root cause: {root_cause}"),
                None,
            ));
        }

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
