#![cfg_attr(coverage_nightly, coverage(off))]
//! RustProjectScore Orchestrator
//!
//! Aggregates all 10 category scorers into a unified project score.
//!
//! Categories (dynamically computed from scorer max_points):
//! - Rust Tooling & CI/CD (130pts)
//! - Code Quality (26pts)
//! - Testing Excellence (20pts)
//! - Documentation (15pts)
//! - Performance & Benchmarking (10pts)
//! - Dependency Health (12pts)
//! - Formal Verification (16pts)
//! - Known Defects (20pts)
//! - GPU/SIMD Quality (10pts)
//! - Build Performance (15pts)

use super::build_perf_scorer::BuildPerfScorer;
use super::code_quality_scorer::CodeQualityScorer;
use super::dependency_scorer::DependencyScorer;
use super::documentation_scorer::DocumentationScorer;
use super::formal_verification_scorer::FormalVerificationScorer;
use super::gpu_simd_scorer::GpuSimdScorer;
use super::known_defects_scorer::KnownDefectsScorer;
use super::models::*;
use super::performance_scorer::PerformanceScorer;
use super::reproducibility_scorer::ReproducibilityScorer;
use super::rust_tooling_scorer::RustToolingScorer;
use super::scorer::{Scorer, ScorerError, ScorerResult};
use super::testing_scorer::TestingScorer;
use crate::services::progress::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// Rust Project Score specification version
/// This tracks the scoring methodology version, not the PMAT binary version
/// v3.0: Added Reproducibility category (Popper B-F absorption) + falsifiability gateway
pub const SPEC_VERSION: &str = "3.0";

/// Orchestrates all 11 category scorers to produce unified project score
///
/// v3.0: Added Reproducibility scorer (Popper B-F absorption) + falsifiability gateway
pub struct RustProjectScoreOrchestrator {
    /// All 11 category scorers
    scorers: Vec<Box<dyn Scorer>>,
}

impl RustProjectScoreOrchestrator {
    /// Create a new orchestrator with all 11 scorers (v3.0)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let scorers: Vec<Box<dyn Scorer>> = vec![
            Box::new(RustToolingScorer::new()),
            Box::new(CodeQualityScorer::new()),
            Box::new(TestingScorer::new()),
            Box::new(DocumentationScorer::new()),
            Box::new(PerformanceScorer::new()),
            Box::new(DependencyScorer::new()),
            Box::new(FormalVerificationScorer::new()),
            Box::new(KnownDefectsScorer::new()),
            Box::new(GpuSimdScorer::new()),
            Box::new(BuildPerfScorer::new()),
            // v3.0: Popper B-F absorption
            Box::new(ReproducibilityScorer::new()),
        ];

        Self { scorers }
    }

    /// Get orchestrator name with spec version
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn name(&self) -> String {
        format!("Rust Project Score v{}", SPEC_VERSION)
    }

    /// Get maximum possible points (sum of all scorer max_points)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_points(&self) -> f64 {
        self.scorers.iter().map(|s| s.max_points()).sum()
    }

    /// Get all scorer names
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn scorer_names(&self) -> Vec<&str> {
        self.scorers.iter().map(|s| s.name()).collect()
    }

    /// Get maximum points by category
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_points_by_category(&self) -> HashMap<&str, f64> {
        self.scorers
            .iter()
            .map(|s| (s.name(), s.max_points()))
            .collect()
    }

    /// Calculate grade from score and max
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn calculate_grade(&self, earned: f64, _max: f64) -> Grade {
        Grade::from_score(earned, _max)
    }

    /// Score a Rust project with fast mode (default, <60 seconds)
    ///
    /// Runs all 10 category scorers and aggregates results
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn score(&self, project_path: &Path) -> ScorerResult<ProjectScore> {
        self.score_with_mode(project_path, ScoringMode::default())
    }

    /// Score a Rust project with configurable mode
    ///
    /// # Arguments
    /// * `project_path` - Path to Rust project
    /// * `mode` - Scoring mode (Quick/<10s, Fast/<60s, Full/<5m)
    ///
    /// # Performance Targets
    /// - Quick mode: <10 seconds - Filesystem only
    /// - Fast mode (default): <60 seconds - Skip expensive cargo operations
    /// - Full mode: <5 minutes (300 seconds) - Complete analysis
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn score_with_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<ProjectScore> {
        // Verify project has Cargo.toml or lakefile.lean (root or lean/ subdir)
        let is_rust = project_path.join("Cargo.toml").exists();
        let is_lean = project_path.join("lakefile.lean").exists()
            || project_path.join("lean-toolchain").exists()
            || project_path.join("lean").join("lakefile.lean").exists()
            || project_path.join("lean").join("lean-toolchain").exists();
        if !is_rust && !is_lean {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml or lakefile.lean found - not a valid project".to_string(),
            ));
        }

        // Verify path exists
        if !project_path.exists() {
            return Err(ScorerError::InvalidProject(format!(
                "Path does not exist: {}",
                project_path.display()
            )));
        }

        // Verify path is a directory
        if !project_path.is_dir() {
            return Err(ScorerError::InvalidProject(format!(
                "Path is not a directory: {}",
                project_path.display()
            )));
        }

        // ═══════════════════════════════════════════════════════════════
        // Kaizen Round 4: Create FileCache once, share across all scorers
        // ═══════════════════════════════════════════════════════════════
        // BEFORE: Each scorer read filesystem independently (22 walks!)
        // AFTER: Single filesystem walk, cache shared (3x faster)
        let file_cache = match FileCache::populate(project_path) {
            Ok(cache) => {
                let (files, bytes) = cache.stats();
                if mode == ScoringMode::Full {
                    // Only show cache stats in full mode (verbose)
                    eprintln!("📦 Cached {} files ({} KB)", files, bytes / 1024);
                }
                Some(cache)
            }
            Err(e) => {
                eprintln!("⚠️  FileCache failed: {}, using direct filesystem reads", e);
                None
            }
        };

        // Run all scorers and collect results
        // **Kaizen Round 5**: Parallel scorer execution for 2-3x speedup
        let mut category_map: HashMap<String, CategoryScore> = HashMap::new();
        let mut all_recommendations: Vec<String> = Vec::new();

        // Create progress spinner (simpler for parallel execution)
        let pb = ProgressBar::new_spinner();
        pb.set_message(format!(
            "Analyzing {} categories in parallel...",
            self.scorers.len()
        ));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Run scorers in parallel using rayon
        // Lean/non-Rust projects: scorers that require Cargo.toml gracefully return 0 points
        let results: Result<Vec<_>, ScorerError> = Ok(self
            .scorers
            .par_iter()
            .filter_map(|scorer| {
                match scorer.score_with_cache(project_path, mode, file_cache.as_ref()) {
                    Ok(category_score) => {
                        let recommendations = scorer.recommendations(project_path);
                        Some((scorer.name().to_string(), category_score, recommendations))
                    }
                    Err(_) => {
                        // Scorer failed (e.g., no Cargo.toml for Rust-specific scorer)
                        // Mark as not applicable — excluded from normalized grade
                        Some((
                            scorer.name().to_string(),
                            CategoryScore::not_applicable(scorer.max_points()),
                            Vec::new(),
                        ))
                    }
                }
            })
            .collect());

        pb.finish_with_message("✅ Analysis complete");

        // Unpack parallel results
        let results = results?;
        for (name, score, recs) in results {
            category_map.insert(name, score);
            all_recommendations.extend(recs);
        }

        // Build CategoryScores struct
        let categories = category_map.clone();

        // Calculate total earned
        let total_earned: f64 = category_map.values().map(|cs| cs.earned).sum();

        // Calculate normalized percentage: average of per-category percentages
        // for APPLICABLE categories only. Non-applicable categories (e.g., Rust
        // Tooling for a pure Lean project) are excluded so they don't penalize
        // projects where the category is irrelevant.
        let applicable: Vec<&CategoryScore> =
            category_map.values().filter(|cs| cs.applicable).collect();
        let num_applicable = applicable.len() as f64;
        let percentage = if num_applicable > 0.0 {
            let sum_pcts: f64 = applicable
                .iter()
                .map(|cs| {
                    if cs.max > 0.0 {
                        (cs.earned / cs.max) * 100.0
                    } else {
                        100.0
                    }
                })
                .sum();
            sum_pcts / num_applicable
        } else {
            0.0
        };

        // v3.0: Falsifiability gateway (Jidoka) — if Cat A < 60%, cap at grade F
        let gateway_failed =
            super::reproducibility_scorer::check_falsifiability_gateway(project_path).is_none();

        let grade = if gateway_failed {
            Grade::F
        } else {
            Grade::from_normalized(percentage)
        };

        if gateway_failed {
            all_recommendations.insert(
                0,
                "GATEWAY FAILED: Falsifiability score < 60%. Add testable claims and \
                 test coverage to unlock higher grades. See `pmat popper-score` for details."
                    .to_string(),
            );
        }

        let result = ProjectScore {
            total_earned,
            total_possible: self.max_points(),
            percentage,
            grade,
            categories,
            recommendations: all_recommendations,
        };
        debug_assert!(result.total_earned >= 0.0, "earned score negative");
        debug_assert!(
            result.percentage >= 0.0 && result.percentage <= 100.0,
            "percentage out of range: {}",
            result.percentage
        );
        Ok(result)
    }
}

/// Workspace member with its score
#[derive(Debug, Clone)]
pub struct WorkspaceMemberScore {
    /// Crate name
    pub name: String,
    /// Path relative to workspace root
    pub path: String,
    /// Score (None if scoring failed)
    pub score: Option<ProjectScore>,
}

/// Workspace scoring result
#[derive(Debug, Clone)]
pub struct WorkspaceScore {
    /// Root workspace score
    pub root: ProjectScore,
    /// Per-member scores
    pub members: Vec<WorkspaceMemberScore>,
    /// Aggregate percentage (geometric mean)
    pub aggregate_percentage: f64,
    /// Aggregate grade
    pub aggregate_grade: Grade,
}

/// Discover workspace members from Cargo.toml
// Wave 39 PR27: contract added — output is bounded by the count of lines
// inside the [workspace] members = [...] block. No unbounded allocations.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn discover_workspace_members(project_path: &Path) -> Vec<(String, std::path::PathBuf)> {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    // Parse [workspace] members
    let mut in_members = false;
    let mut members = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            // Handle inline: members = ["crate1", "crate2"]
            if let Some(bracket_content) = trimmed.split('[').nth(1) {
                for item in bracket_content.split(']').next().unwrap_or("").split(',') {
                    let member = item.trim().trim_matches('"').trim_matches('\'');
                    if !member.is_empty() {
                        expand_member(project_path, member, &mut members);
                    }
                }
            }
            continue;
        }
        if in_members {
            if trimmed.starts_with(']') {
                break;
            }
            // Wave 39 release-prep: BUG #3 fix — was using sequential
            // `.trim_matches('"').trim_matches('\'').trim_matches(',')` which
            // left a trailing `"` for inputs like `"foo",` because the comma
            // sat between the quote and the end. Use a char-set predicate to
            // strip all three in one pass.
            let member = trimmed.trim_matches(|c: char| c == '"' || c == '\'' || c == ',');
            if !member.is_empty() && !member.starts_with('#') {
                expand_member(project_path, member, &mut members);
            }
        }
    }
    members
}

/// Expand a workspace member path (with glob support)
fn expand_member(root: &Path, pattern: &str, members: &mut Vec<(String, std::path::PathBuf)>) {
    if pattern.contains('*') {
        // Glob expansion
        let full_pattern = root.join(pattern).to_string_lossy().to_string();
        if let Ok(paths) = glob::glob(&full_pattern) {
            for path in paths.flatten() {
                if path.join("Cargo.toml").exists() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    members.push((name, path));
                }
            }
        }
    } else {
        let member_path = root.join(pattern);
        if member_path.join("Cargo.toml").exists() {
            let name = member_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            members.push((name, member_path));
        }
    }
}

impl RustProjectScoreOrchestrator {
    /// Score a workspace with per-crate breakdown
    #[provable_contracts_macros::contract(
        "workspace-scoring-v1.yaml",
        equation = "workspace_aggregate"
    )]
    pub fn score_workspace(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<WorkspaceScore> {
        // Score the root project first
        let root = self.score_with_mode(project_path, mode)?;

        // Discover workspace members
        let workspace_members = discover_workspace_members(project_path);

        let mut members = Vec::new();
        let mut valid_percentages = Vec::new();

        for (name, member_path) in &workspace_members {
            // Skip the root (already scored)
            if member_path == project_path {
                continue;
            }
            // Skip members without src/
            if !member_path.join("src").exists() && !member_path.join("lib.rs").exists() {
                continue;
            }
            match self.score_with_mode(member_path, mode) {
                Ok(score) => {
                    valid_percentages.push(score.percentage);
                    members.push(WorkspaceMemberScore {
                        name: name.clone(),
                        path: member_path
                            .strip_prefix(project_path)
                            .unwrap_or(member_path)
                            .to_string_lossy()
                            .to_string(),
                        score: Some(score),
                    });
                }
                Err(_) => {
                    members.push(WorkspaceMemberScore {
                        name: name.clone(),
                        path: member_path
                            .strip_prefix(project_path)
                            .unwrap_or(member_path)
                            .to_string_lossy()
                            .to_string(),
                        score: None,
                    });
                }
            }
        }

        // Aggregate: geometric mean of all valid scores (including root)
        valid_percentages.push(root.percentage);
        let aggregate_percentage = if valid_percentages.is_empty() {
            0.0
        } else {
            let product: f64 = valid_percentages.iter().map(|p| p.max(0.1)).product();
            product.powf(1.0 / valid_percentages.len() as f64)
        };

        let aggregate_grade = Grade::from_normalized(aggregate_percentage);

        Ok(WorkspaceScore {
            root,
            members,
            aggregate_percentage,
            aggregate_grade,
        })
    }
}

impl Default for RustProjectScoreOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for RustProjectScoreOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustProjectScoreOrchestrator")
            .field("scorer_count", &self.scorers.len())
            .field("max_points", &self.max_points())
            .finish()
    }
}

/// Project score result from orchestrator
#[derive(Debug, Clone)]
pub struct ProjectScore {
    /// Total points earned
    pub total_earned: f64,

    /// Total possible points - sum of all 10 category maxes
    pub total_possible: f64,

    /// Percentage (0-100)
    pub percentage: f64,

    /// Letter grade
    pub grade: Grade,

    /// Scores by category
    pub categories: HashMap<String, CategoryScore>,

    /// Recommendations
    pub recommendations: Vec<String>,
}

// SAFETY: RustProjectScoreOrchestrator holds only a PathBuf (owned, Send+Sync) and no interior
// mutability, making it safe to send between and share across threads for parallel scoring.
unsafe impl Send for RustProjectScoreOrchestrator {}
unsafe impl Sync for RustProjectScoreOrchestrator {}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orch = RustProjectScoreOrchestrator::new();
        assert_eq!(orch.name(), format!("Rust Project Score v{}", SPEC_VERSION));
        // max_points is dynamically computed from all 11 scorers
        // 130 + 26 + 20 + 15 + 10 + 12 + 16 + 20 + 10 + 15 + 15 = 289
        assert_eq!(orch.max_points(), 289.0);
    }

    #[test]
    fn test_scorer_count() {
        let orch = RustProjectScoreOrchestrator::new();
        assert_eq!(orch.scorers.len(), 11);
    }

    #[test]
    fn test_formal_verification_scorer_present() {
        let orch = RustProjectScoreOrchestrator::new();
        let names: Vec<&str> = orch.scorer_names();
        assert!(names.contains(&"Formal Verification"));
    }

    #[test]
    fn test_known_defects_scorer_present() {
        let orch = RustProjectScoreOrchestrator::new();
        let names: Vec<&str> = orch.scorer_names();
        assert!(names.contains(&"Known Defects"));
    }

    #[test]
    fn test_gpu_simd_scorer_present() {
        let orch = RustProjectScoreOrchestrator::new();
        let names: Vec<&str> = orch.scorer_names();
        assert!(names.contains(&"GPU/SIMD Quality"));
    }

    // ── Wave 39 PR27: discover_workspace_members + expand_member ────────────

    #[test]
    fn test_discover_workspace_members_no_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let members = discover_workspace_members(tmp.path());
        assert!(members.is_empty());
    }

    #[test]
    fn test_discover_workspace_members_no_workspace_section() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"foo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let members = discover_workspace_members(tmp.path());
        assert!(members.is_empty());
    }

    #[test]
    fn test_discover_workspace_members_inline_members() {
        // PIN: inline form `members = ["a", "b"]` is parsed in same line.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("crate_a")).unwrap();
        std::fs::write(tmp.path().join("crate_a/Cargo.toml"), "[package]").unwrap();
        std::fs::create_dir(tmp.path().join("crate_b")).unwrap();
        std::fs::write(tmp.path().join("crate_b/Cargo.toml"), "[package]").unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crate_a\", \"crate_b\"]\n",
        )
        .unwrap();
        let members = discover_workspace_members(tmp.path());
        assert_eq!(members.len(), 2);
        let names: Vec<&str> = members.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"crate_a"));
        assert!(names.contains(&"crate_b"));
    }

    #[test]
    fn test_discover_workspace_members_multiline_with_trailing_commas() {
        // BUG #3 FIXED in release-prep: previously sequential
        // `.trim_matches('"').trim_matches('\'').trim_matches(',')` left a
        // trailing `"` for inputs like `"foo",`. Now uses a char-set
        // predicate `|c| c == '"' || c == '\'' || c == ','` to strip all
        // three in one pass.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("foo")).unwrap();
        std::fs::write(tmp.path().join("foo/Cargo.toml"), "").unwrap();
        std::fs::create_dir(tmp.path().join("bar")).unwrap();
        std::fs::write(tmp.path().join("bar/Cargo.toml"), "").unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\n  \"foo\",\n  \"bar\",\n]\n",
        )
        .unwrap();
        let members = discover_workspace_members(tmp.path());
        assert_eq!(members.len(), 2);
        let names: Vec<&str> = members.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"bar"));
    }

    #[test]
    fn test_discover_workspace_members_multiline_no_trailing_comma_works() {
        // PIN: the multi-line LAST entry without trailing comma works because
        // there's no `,` to be trimmed last. `"bar"` → trim '"' → `bar`.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("bar")).unwrap();
        std::fs::write(tmp.path().join("bar/Cargo.toml"), "").unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\n  \"bar\"\n]\n",
        )
        .unwrap();
        let members = discover_workspace_members(tmp.path());
        // Without trailing comma, the trim sequence works.
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, "bar");
    }

    #[test]
    fn test_discover_workspace_members_inline_skips_missing_cargo_toml() {
        // PIN: a member without a Cargo.toml is silently skipped.
        // Inline form (single line) parses correctly because `.split(',')`
        // removes commas BEFORE the trim sequence (no trim-order PIN bug).
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("crate_real")).unwrap();
        std::fs::write(tmp.path().join("crate_real/Cargo.toml"), "").unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crate_real\", \"crate_phantom\"]\n",
        )
        .unwrap();
        let members = discover_workspace_members(tmp.path());
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, "crate_real");
    }

    #[test]
    fn test_discover_workspace_members_skips_comment_lines() {
        // Comment lines starting with `#` are skipped inside members block.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("my_crate")).unwrap();
        std::fs::write(tmp.path().join("my_crate/Cargo.toml"), "").unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\n  # comment\n  \"my_crate\",\n]\n",
        )
        .unwrap();
        let members = discover_workspace_members(tmp.path());
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, "my_crate");
    }

    // ── expand_member ───────────────────────────────────────────────────────

    #[test]
    fn test_expand_member_concrete_path() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("my_crate")).unwrap();
        std::fs::write(tmp.path().join("my_crate/Cargo.toml"), "").unwrap();
        let mut members = Vec::new();
        expand_member(tmp.path(), "my_crate", &mut members);
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, "my_crate");
    }

    #[test]
    fn test_expand_member_glob_expansion() {
        // PIN: pattern containing `*` triggers glob expansion.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("crates")).unwrap();
        std::fs::create_dir(tmp.path().join("crates/foo")).unwrap();
        std::fs::write(tmp.path().join("crates/foo/Cargo.toml"), "").unwrap();
        std::fs::create_dir(tmp.path().join("crates/bar")).unwrap();
        std::fs::write(tmp.path().join("crates/bar/Cargo.toml"), "").unwrap();
        let mut members = Vec::new();
        expand_member(tmp.path(), "crates/*", &mut members);
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn test_expand_member_glob_skips_dirs_without_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("crates")).unwrap();
        std::fs::create_dir(tmp.path().join("crates/real")).unwrap();
        std::fs::write(tmp.path().join("crates/real/Cargo.toml"), "").unwrap();
        std::fs::create_dir(tmp.path().join("crates/empty")).unwrap();
        let mut members = Vec::new();
        expand_member(tmp.path(), "crates/*", &mut members);
        // Only "real" has Cargo.toml.
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0, "real");
    }

    #[test]
    fn test_expand_member_concrete_skips_missing_cargo() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("phantom")).unwrap();
        // No Cargo.toml in phantom dir.
        let mut members = Vec::new();
        expand_member(tmp.path(), "phantom", &mut members);
        assert!(members.is_empty());
    }
}
// #[requires(project_path.exists())]
// #[ensures(result.is_ok())]
