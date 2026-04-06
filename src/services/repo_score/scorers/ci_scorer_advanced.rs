// CiScorer advanced CI features scoring (E3)
// Included from ci_scorer.rs - do NOT add `use` imports or `#!` attributes here.

struct FeatureCheck {
    keywords: &'static [&'static str],
    success_msg: &'static str,
    missing_msg: &'static str,
    points: f64,
}

fn check_ci_feature(content: &str, check: &FeatureCheck) -> (f64, Finding) {
    let found = check.keywords.iter().any(|kw| content.contains(kw));
    if found {
        (check.points, Finding {
            severity: Severity::Success,
            category: "CI".to_string(),
            message: check.success_msg.to_string(),
            location: None,
            impact_points: check.points,
        })
    } else {
        (0.0, Finding {
            severity: Severity::Info,
            category: "CI".to_string(),
            message: check.missing_msg.to_string(),
            location: None,
            impact_points: 0.0,
        })
    }
}

const ADVANCED_CHECKS: &[FeatureCheck] = &[
    FeatureCheck {
        keywords: &["codecov", "coveralls", "llvm-cov", "coverage"],
        success_msg: "\u{2713} Code coverage reporting (+2 pts)",
        missing_msg: "Missing: Add coverage reporting (codecov, coveralls) (+2 pts)",
        points: 2.0,
    },
    FeatureCheck {
        keywords: &["security", "audit", "trivy", "snyk", "codeql", "dependabot"],
        success_msg: "\u{2713} Security scanning enabled (+2 pts)",
        missing_msg: "Missing: Add security scanning (cargo audit, CodeQL, Trivy) (+2 pts)",
        points: 2.0,
    },
    FeatureCheck {
        keywords: &["cache", "actions/cache"],
        success_msg: "\u{2713} Build caching configured (+2 pts)",
        missing_msg: "Missing: Add caching (actions/cache) for faster builds (+2 pts)",
        points: 2.0,
    },
    FeatureCheck {
        keywords: &["matrix:", "strategy:"],
        success_msg: "\u{2713} Matrix/strategy builds configured (+2 pts)",
        missing_msg: "Missing: Add matrix builds for multi-platform testing (+2 pts)",
        points: 2.0,
    },
];

impl CiScorer {
    /// Score advanced CI features (E3: 8 points)
    /// Issue #72: Provides actionable feedback for advanced CI improvements
    async fn score_advanced_features(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let workflows_dir = repo_path.join(".github/workflows");

        if !workflows_dir.exists() {
            return Ok(SubcategoryScore {
                id: "E3".to_string(),
                name: "Advanced CI Features".to_string(),
                score: 0.0,
                max_score: 8.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "CI".to_string(),
                    message:
                        "Add workflows first to unlock advanced CI features (+8 pts available)"
                            .to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        let all_content = collect_workflow_content(&workflows_dir).await;
        let uses_sovereign_ci = all_content.contains("sovereign-ci");

        let mut total_score: f64 = 0.0;
        let mut findings = vec![];

        if uses_sovereign_ci {
            // sovereign-ci.yml provides coverage, security, caching, and matrix builds
            for check in ADVANCED_CHECKS {
                total_score += check.points;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "CI".to_string(),
                    message: format!("{} (via sovereign-ci.yml)", check.success_msg),
                    location: None,
                    impact_points: check.points,
                });
            }
        } else {
            for check in ADVANCED_CHECKS {
                let (pts, finding) = check_ci_feature(&all_content, check);
                total_score += pts;
                findings.push(finding);
            }
        }

        Ok(SubcategoryScore {
            id: "E3".to_string(),
            name: "Advanced CI Features".to_string(),
            score: total_score.min(8.0),
            max_score: 8.0,
            findings,
        })
    }
}
