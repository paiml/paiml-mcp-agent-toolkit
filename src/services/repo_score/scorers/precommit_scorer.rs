#![cfg_attr(coverage_nightly, coverage(off))]
// PrecommitScorer - Category B: Pre-commit Hooks (20 points)
//
// Scores based on:
// - B1: Pre-commit Hook Present (10 points) - .git/hooks/pre-commit exists and executable
// - B2: Hook Gate Coverage (10 points) - which quality gates the hook actually invokes

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Precommit scorer.
pub struct PrecommitScorer;

impl PrecommitScorer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }

    /// Score pre-commit hook presence (B1: 10 points)
    async fn score_hook_present(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let git_hooks_path = repo_path.join(".git/hooks");
        let precommit_path = git_hooks_path.join("pre-commit");

        if !precommit_path.exists() {
            return Ok(SubcategoryScore {
                id: "B1".to_string(),
                name: "Pre-commit Hook Present".to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Pre-commit".to_string(),
                    message: "No pre-commit hook found".to_string(),
                    location: Some(precommit_path.display().to_string()),
                    impact_points: -10.0,
                }],
            });
        }

        // Check if executable (Unix only)
        #[cfg(unix)]
        {
            let metadata = std::fs::metadata(&precommit_path)?;
            let permissions = metadata.permissions();
            let is_executable = permissions.mode() & 0o111 != 0;

            if !is_executable {
                return Ok(SubcategoryScore {
                    id: "B1".to_string(),
                    name: "Pre-commit Hook Present".to_string(),
                    score: 5.0, // Partial credit
                    max_score: 10.0,
                    findings: vec![Finding {
                        severity: Severity::Warning,
                        category: "Pre-commit".to_string(),
                        message: "Pre-commit hook exists but is not executable".to_string(),
                        location: Some(precommit_path.display().to_string()),
                        impact_points: -5.0,
                    }],
                });
            }
        }

        // Check file is not empty
        let content = tokio::fs::read_to_string(&precommit_path).await?;
        if content.trim().is_empty() {
            return Ok(SubcategoryScore {
                id: "B1".to_string(),
                name: "Pre-commit Hook Present".to_string(),
                score: 2.0, // Minimal credit
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Pre-commit".to_string(),
                    message: "Pre-commit hook is empty".to_string(),
                    location: Some(precommit_path.display().to_string()),
                    impact_points: -8.0,
                }],
            });
        }

        Ok(SubcategoryScore {
            id: "B1".to_string(),
            name: "Pre-commit Hook Present".to_string(),
            score: 10.0,
            max_score: 10.0,
            findings: vec![Finding {
                severity: Severity::Success,
                category: "Pre-commit".to_string(),
                message: "Pre-commit hook present and executable".to_string(),
                location: Some(precommit_path.display().to_string()),
                impact_points: 0.0,
            }],
        })
    }

    /// Score which quality gates the hook actually invokes (B2: 10 points).
    ///
    /// This subcategory used to be called "Hook Execution Time" and claimed to
    /// score "runs in <30 seconds" — while never executing or timing anything.
    /// It grepped the script text: a hook whose entire body was `# clippy` +
    /// `sleep 300` matched the `clippy` substring and scored 10/10 "likely
    /// fast", and a hook that returned in 2 ms scored 5/10 "may be slow"
    /// because it mentioned `cargo test`. The ranking was exactly inverted, the
    /// verdicts were fabricated, and 5 of repo-score's 100 points turned on it
    /// (#940).
    ///
    /// repo-score will not execute the audited repository's hooks to get a real
    /// timing: a pre-commit hook routinely runs formatters and test suites that
    /// rewrite the working tree, and a scoring command must not mutate what it
    /// scores. So this measures what the script *is* rather than pretending to
    /// measure what it costs — the gates it invokes — and says so in its name.
    /// No timing claim is emitted anywhere.
    ///
    /// A gate mentioned only in a comment is not invoked, so comments are
    /// stripped before matching; that alone reverses the case in #940.
    async fn score_hook_gate_coverage(
        &self,
        repo_path: &Path,
        _config: &ScorerConfig,
    ) -> Result<SubcategoryScore> {
        let precommit_path = repo_path.join(".git/hooks/pre-commit");

        if !precommit_path.exists() {
            // No hook = no gates to find.
            return Ok(SubcategoryScore {
                id: "B2".to_string(),
                name: B2_NAME.to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Pre-commit".to_string(),
                    message: "No pre-commit hook, so no gate runs before a commit".to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        let content = tokio::fs::read_to_string(&precommit_path).await?;
        let gates = detect_hook_gates(&content);
        // Folded from an explicit `0.0`, not `Iterator::sum`: Rust's float sum
        // uses -0.0 as its additive identity, so an empty gate list serialised
        // as `"score": -0.0` in the JSON report.
        let score = gates
            .iter()
            .fold(0.0_f64, |acc, g| acc + g.points)
            .min(10.0);

        let finding = if gates.is_empty() {
            Finding {
                severity: Severity::Warning,
                category: "Pre-commit".to_string(),
                message: "Hook invokes no recognised quality gate (format, lint, test or pmat) - commits are unguarded".to_string(),
                location: Some(precommit_path.display().to_string()),
                impact_points: -10.0,
            }
        } else {
            let names: Vec<&str> = gates.iter().map(|g| g.name).collect();
            Finding {
                severity: if score >= 8.0 {
                    Severity::Success
                } else {
                    Severity::Info
                },
                category: "Pre-commit".to_string(),
                message: format!("Hook invokes {} gate(s): {}", gates.len(), names.join(", ")),
                location: Some(precommit_path.display().to_string()),
                impact_points: score - 10.0,
            }
        };

        Ok(SubcategoryScore {
            id: "B2".to_string(),
            name: B2_NAME.to_string(),
            score,
            max_score: 10.0,
            findings: vec![finding],
        })
    }
}

/// B2's name, in one place: the subcategory and its tests must not drift.
const B2_NAME: &str = "Hook Gate Coverage";

/// A quality gate a pre-commit hook can invoke, and what it is worth.
struct HookGate {
    name: &'static str,
    points: f64,
}

/// The gate table: (display name, points, invocation substrings).
///
/// Points sum to 10 exactly, so a hook that formats, lints, tests and runs
/// pmat's own gate scores B2 in full.
const HOOK_GATES: [(&str, f64, &[&str]); 4] = [
    (
        "format",
        2.0,
        &[
            "cargo fmt",
            "rustfmt",
            "prettier",
            "black ",
            "gofmt",
            "ruff format",
        ],
    ),
    (
        "lint",
        3.0,
        &[
            "clippy",
            "eslint",
            "pylint",
            "ruff check",
            "shellcheck",
            "bashrs",
            "golangci-lint",
            "lint",
        ],
    ),
    (
        "test",
        3.0,
        &[
            "cargo test",
            "cargo nextest",
            "pytest",
            "npm test",
            "yarn test",
            "make test",
            "go test",
        ],
    ),
    (
        "pmat",
        2.0,
        &[
            "pmat verify",
            "pmat quality-gate",
            "pmat analyze",
            "pmat comply",
            "llvm-cov",
            "coverage",
        ],
    ),
];

/// Strip `#` comments so a gate named only in a comment earns nothing.
///
/// A `#` inside single or double quotes is not a comment; everything else from
/// `#` to end of line is dropped, as is the `#!` shebang.
///
/// One rule, one implementation: `pmat comply`'s CB-1337 reads the same hook
/// script for the same reason and calls this, rather than growing a second
/// answer to "does this hook invoke X".
pub(crate) fn strip_shell_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        let mut in_single = false;
        let mut in_double = false;
        let mut cut = line.len();
        for (i, ch) in line.char_indices() {
            match ch {
                '\'' if !in_double => in_single = !in_single,
                '"' if !in_single => in_double = !in_double,
                '#' if !in_single && !in_double => {
                    cut = i;
                    break;
                }
                _ => {}
            }
        }
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}

/// The gates a hook script actually invokes.
fn detect_hook_gates(content: &str) -> Vec<HookGate> {
    let code = strip_shell_comments(content).to_lowercase();
    HOOK_GATES
        .iter()
        .filter(|(_, _, patterns)| patterns.iter().any(|p| code.contains(p)))
        .map(|(name, points, _)| HookGate {
            name,
            points: *points,
        })
        .collect()
}

#[async_trait]
impl Scorer for PrecommitScorer {
    fn category_name(&self) -> &str {
        "Pre-commit Hooks"
    }

    fn max_score(&self) -> f64 {
        20.0
    }

    async fn score(&self, repo_path: &Path, config: &ScorerConfig) -> Result<CategoryScore> {
        let b1 = self.score_hook_present(repo_path).await?;
        let b2 = self.score_hook_gate_coverage(repo_path, config).await?;

        let total_score = b1.score + b2.score;

        let mut findings = b1.findings.clone();
        findings.extend(b2.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![b1, b2],
            findings,
        ))
    }
}

impl Default for PrecommitScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_git_repo(repo_path: &Path) {
        let git_dir = repo_path.join(".git");
        let hooks_dir = git_dir.join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
    }

    fn create_precommit_hook(repo_path: &Path, content: &str, executable: bool) {
        let hook_path = repo_path.join(".git/hooks/pre-commit");
        fs::write(&hook_path, content).unwrap();

        #[cfg(unix)]
        if executable {
            let metadata = fs::metadata(&hook_path).unwrap();
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook_path, permissions).unwrap();
        }

        #[cfg(not(unix))]
        let _ = executable;
    }

    const LINTING_HOOK: &str = r#"#!/bin/bash
cargo clippy -- -D warnings
"#;

    /// B2 for a lint-only hook: the `lint` gate alone.
    const LINTING_HOOK_B2: f64 = 3.0;

    #[tokio::test]
    async fn test_precommit_scorer_no_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 0 (no hook), B2: 0 (no hook) = 0 total
        assert_eq!(result.score, 0.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_precommit_scorer_valid_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 10 (valid hook), B2: 3 (lint gate only) = 13 total.
        assert_eq!(result.score, 10.0 + LINTING_HOOK_B2);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_precommit_scorer_non_executable_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, false);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 5 (not executable), B2: 3 (lint gate only) = 8 total.
        assert_eq!(result.score, 5.0 + LINTING_HOOK_B2);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("not executable")));
    }

    #[tokio::test]
    async fn test_precommit_scorer_empty_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, "", true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 2 (empty), B2: 0 (an empty hook gates nothing) = 2 total.
        assert_eq!(result.score, 2.0);
        assert!(result.findings.iter().any(|f| f.message.contains("empty")));
    }

    #[tokio::test]
    async fn test_precommit_hook_present_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let b1 = result.subcategories.iter().find(|s| s.id == "B1").unwrap();
        assert_eq!(b1.name, "Pre-commit Hook Present");
        assert_eq!(b1.score, 10.0);
        assert_eq!(b1.max_score, 10.0);
    }

    #[tokio::test]
    async fn test_precommit_gate_coverage_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let b2 = result.subcategories.iter().find(|s| s.id == "B2").unwrap();
        assert_eq!(b2.name, B2_NAME);
        assert_eq!(b2.max_score, 10.0);
    }

    // ── #940: B2 claimed a timing verdict it never measured ──

    async fn b2_for(hook: &str) -> SubcategoryScore {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, hook, true);
        let result = PrecommitScorer::new()
            .score(repo_path, &ScorerConfig::default())
            .await
            .unwrap();
        result
            .subcategories
            .into_iter()
            .find(|s| s.id == "B2")
            .expect("B2")
    }

    /// The exact pair from #940. The 300-second hook mentions `clippy` only in
    /// a COMMENT and runs nothing; the 2 ms hook mentions `cargo test` only in
    /// a comment and also runs nothing. B2 used to rank the sleeper 10/10
    /// "likely fast" ABOVE the instant one at 5/10 "may be slow". Neither
    /// invokes a gate, so neither may score above the other — and no timing
    /// verdict may be emitted at all.
    #[tokio::test]
    async fn commented_out_gates_earn_nothing_and_no_timing_is_claimed() {
        let sleeper = b2_for("#!/bin/sh\n# clippy\nsleep 300\n").await;
        let instant = b2_for("#!/bin/sh\ntrue # cargo test\n").await;

        assert_eq!(sleeper.score, 0.0, "{:?}", sleeper.findings);
        assert_eq!(instant.score, 0.0, "{:?}", instant.findings);
        assert!(
            !sleeper.score.is_sign_negative(),
            "a zero score must serialise as 0.0, not -0.0"
        );
        assert!(
            sleeper.score <= instant.score,
            "a hook that sleeps 300s must never outrank one that returns in 2ms"
        );

        for sub in [&sleeper, &instant] {
            assert_eq!(sub.name, B2_NAME);
            let text = format!("{sub:?}").to_lowercase();
            for claim in ["likely fast", "may be slow", "assumed acceptable", "second"] {
                assert!(
                    !text.contains(claim),
                    "B2 must not emit the timing verdict {claim:?}: {text}"
                );
            }
        }
    }

    /// More gates is more points, never fewer: a hook that also runs tests
    /// used to be PENALISED 5 points relative to a lint-only one.
    #[tokio::test]
    async fn adding_a_test_gate_never_lowers_the_score() {
        let lint_only = b2_for("#!/bin/sh\ncargo clippy -- -D warnings\n").await;
        let lint_and_test = b2_for("#!/bin/sh\ncargo clippy -- -D warnings\ncargo test\n").await;

        assert_eq!(lint_only.score, LINTING_HOOK_B2);
        assert!(
            lint_and_test.score > lint_only.score,
            "lint+test {} must beat lint-only {}",
            lint_and_test.score,
            lint_only.score
        );
    }

    #[tokio::test]
    async fn a_hook_running_every_gate_scores_full_marks() {
        let full = b2_for(
            "#!/bin/sh\ncargo fmt --all -- --check\ncargo clippy -- -D warnings\ncargo test\npmat verify\n",
        )
        .await;
        assert_eq!(full.score, 10.0);
        assert_eq!(full.max_score, 10.0);
    }

    #[test]
    fn shell_comments_are_stripped_before_matching() {
        assert!(detect_hook_gates("# cargo test\n").is_empty());
        assert!(detect_hook_gates("echo \"# cargo test\"\ncargo test\n").len() == 1);
        assert_eq!(detect_hook_gates("#!/bin/sh\ncargo test\n").len(), 1);
    }

    #[test]
    fn gate_points_sum_to_the_subcategory_maximum() {
        let total: f64 = HOOK_GATES.iter().map(|(_, p, _)| p).sum();
        assert_eq!(total, 10.0, "HOOK_GATES must total B2's max_score");
    }

    #[tokio::test]
    async fn test_precommit_category_name() {
        let scorer = PrecommitScorer::new();
        assert_eq!(scorer.category_name(), "Pre-commit Hooks");
        assert_eq!(scorer.max_score(), 20.0);
    }
}
