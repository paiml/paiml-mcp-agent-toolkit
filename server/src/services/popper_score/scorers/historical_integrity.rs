//! Category E: Historical Integrity (10 points)
//!
//! Tracks evolution of claims over time to prevent HARKing and p-hacking.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | E1 | Version Control Hygiene | 4 | Atomic commits, clear messages |
//! | E2 | Pre-registration | 3 | Design docs before implementation |
//! | E3 | Claim Timestamping | 3 | Immutable claim records |
//!
//! ## Academic Foundation
//!
//! - Bird et al. (2009): Fair and Balanced Review [22]
//! - Kerr (1998): HARKing Prevention [23]
//! - Simmons et al. (2011): P-hacking [24]
//!
//! ## Note on New Projects
//!
//! This category uses normalized scoring to avoid penalizing new projects
//! (Annotation 7: "rich get richer" problem).

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{PopperScorer, PopperScorerResult};
use std::path::Path;

/// Scorer for Category E: Historical Integrity (10 points)
pub struct HistoricalIntegrityScorer;

impl HistoricalIntegrityScorer {
    /// Create a new historical integrity scorer
    pub fn new() -> Self {
        Self
    }

    /// E1: Version Control Hygiene (4 points)
    ///
    /// Checks for:
    /// - Git repository exists (1 point)
    /// - Meaningful commit messages (1 point)
    /// - Multiple contributors (1 point)
    /// - Protected main branch (1 point)
    fn score_version_control_hygiene(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 4.0;
        let mut description = Vec::new();

        // Check for .git directory (1 point)
        if project_path.join(".git").exists() {
            earned += 1.0;
            description.push("git repository");

            // Check for meaningful commit messages by checking hooks or conventional commits
            if project_path.join(".git/hooks/commit-msg").exists()
                || project_path.join("commitlint.config.js").exists()
                || project_path.join(".commitlintrc").exists()
                || project_path.join(".commitlintrc.json").exists()
            {
                earned += 1.0;
                description.push("commit linting configured");
            }
        }

        // Check for CODEOWNERS (indicates protected review)
        if project_path.join("CODEOWNERS").exists()
            || project_path.join(".github/CODEOWNERS").exists()
        {
            earned += 1.0;
            description.push("CODEOWNERS defined");
        }

        // Check for branch protection configuration
        if project_path.join(".github/branch-protection.yml").exists()
            || project_path.join(".github/settings.yml").exists()
        {
            earned += 1.0;
            description.push("branch protection configured");
        }

        // Check for PR template (indicates review process)
        let pr_templates = [
            ".github/PULL_REQUEST_TEMPLATE.md",
            ".github/pull_request_template.md",
            "PULL_REQUEST_TEMPLATE.md",
        ];
        for template in pr_templates {
            if project_path.join(template).exists() {
                earned = (earned + 0.5).min(max);
                description.push("PR template exists");
                break;
            }
        }

        if description.is_empty() {
            description.push("no version control");
        }

        PopperSubScore::new(
            "E1",
            "Version Control Hygiene",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// E2: Pre-registration (3 points)
    ///
    /// Checks for:
    /// - DESIGN.md or RFC documents before implementation (2 points)
    /// - Issue templates for feature proposals (1 point)
    fn score_preregistration(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 3.0;
        let mut description = Vec::new();

        // Check for design documents (2 points)
        let design_docs = [
            "DESIGN.md",
            "RFC.md",
            "docs/rfc",
            "docs/design",
            "docs/proposals",
            "docs/specifications",
        ];
        for doc in design_docs {
            let path = project_path.join(doc);
            if path.exists() {
                earned += 2.0;
                description.push("pre-registration docs found");
                break;
            }
        }

        // Check for feature request template (1 point)
        let issue_templates = [
            ".github/ISSUE_TEMPLATE/feature_request.md",
            ".github/ISSUE_TEMPLATE/feature_request.yml",
            ".github/ISSUE_TEMPLATE.md",
        ];
        for template in issue_templates {
            if project_path.join(template).exists() {
                earned += 1.0;
                description.push("feature request template");
                break;
            }
        }

        // Check for roadmap (indicates planned development)
        let roadmap_files = ["ROADMAP.md", "docs/roadmap.md", "TODO.md"];
        for roadmap in roadmap_files {
            if project_path.join(roadmap).exists() {
                earned = (earned + 0.5).min(max);
                description.push("roadmap documented");
                break;
            }
        }

        if description.is_empty() {
            description.push("no pre-registration");
        }

        PopperSubScore::new(
            "E2",
            "Pre-registration",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// E3: Claim Timestamping (3 points)
    ///
    /// Checks for:
    /// - CHANGELOG with dated entries (1 point)
    /// - Release tags (1 point)
    /// - Semantic versioning (1 point)
    fn score_claim_timestamping(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 3.0;
        let mut description = Vec::new();

        // Check CHANGELOG with dates (1 point)
        let changelog_files = ["CHANGELOG.md", "CHANGELOG", "CHANGES.md", "HISTORY.md"];
        for changelog in changelog_files {
            let path = project_path.join(changelog);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Look for date patterns like 2024-01-15 or January 2024
                    if content.contains('-') && (content.contains("202") || content.contains("201"))
                    {
                        earned += 1.0;
                        description.push("dated CHANGELOG");
                        break;
                    }
                }
            }
        }

        // Check for git tags (requires .git)
        if project_path.join(".git/refs/tags").exists() {
            if let Ok(entries) = std::fs::read_dir(project_path.join(".git/refs/tags")) {
                if entries.count() > 0 {
                    earned += 1.0;
                    description.push("release tags exist");
                }
            }
        }

        // Check for semantic versioning in Cargo.toml or package.json
        let cargo_toml = project_path.join("Cargo.toml");
        let package_json = project_path.join("package.json");

        let semver_pattern = regex::Regex::new(r#"version\s*=\s*["']?\d+\.\d+\.\d+"#).expect("internal error");

        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if semver_pattern.is_match(&content) {
                    earned += 1.0;
                    description.push("semantic versioning");
                }
            }
        } else if package_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_json) {
                if content.contains("\"version\"") && content.contains(".") {
                    earned += 1.0;
                    description.push("semantic versioning");
                }
            }
        }

        if description.is_empty() {
            description.push("no claim timestamping");
        }

        PopperSubScore::new(
            "E3",
            "Claim Timestamping",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }
}

impl Default for HistoricalIntegrityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for HistoricalIntegrityScorer {
    fn name(&self) -> &str {
        "Historical Integrity"
    }

    fn category_id(&self) -> char {
        'E'
    }

    fn max_points(&self) -> f64 {
        10.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());

        // Score each sub-category
        let e1 = self.score_version_control_hygiene(project_path);
        let e2 = self.score_preregistration(project_path);
        let e3 = self.score_claim_timestamping(project_path);

        // Add findings based on scores
        if e1.earned < 2.0 {
            category.add_finding(PopperFinding::warning(
                "Version control hygiene could be improved - consider commit linting and CODEOWNERS",
                4.0 - e1.earned,
            ));
        }

        if e2.earned < 2.0 {
            category.add_finding(PopperFinding::warning(
                "Pre-registration missing - add design docs before implementation",
                3.0 - e2.earned,
            ));
        }

        if e3.earned < 2.0 {
            category.add_finding(PopperFinding::warning(
                "Claim timestamping incomplete - use dated CHANGELOG and release tags",
                3.0 - e3.earned,
            ));
        }

        if e1.earned + e2.earned + e3.earned >= 8.0 {
            category.add_finding(PopperFinding::positive(
                "Strong historical integrity practices",
            ));
        }

        // Add sub-scores
        category.add_sub_score(e1);
        category.add_sub_score(e2);
        category.add_sub_score(e3);

        Ok(category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_historical_integrity_scorer_basics() {
        let scorer = HistoricalIntegrityScorer::new();
        assert_eq!(scorer.name(), "Historical Integrity");
        assert_eq!(scorer.category_id(), 'E');
        assert_eq!(scorer.max_points(), 10.0);
        assert!(!scorer.is_gateway());
    }

    #[test]
    fn test_project_with_git() {
        let temp_dir = tempdir().expect("internal error");

        // Create .git directory
        fs::create_dir_all(temp_dir.path().join(".git")).expect("internal error");

        let scorer = HistoricalIntegrityScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have git points
        let e1 = result.sub_scores.iter().find(|s| s.id == "E1").expect("internal error");
        assert!(e1.earned >= 1.0);
    }

    #[test]
    fn test_project_with_codeowners() {
        let temp_dir = tempdir().expect("internal error");

        // Create .git and CODEOWNERS
        fs::create_dir_all(temp_dir.path().join(".git")).expect("internal error");
        fs::create_dir_all(temp_dir.path().join(".github")).expect("internal error");
        fs::write(temp_dir.path().join(".github/CODEOWNERS"), "* @owner").expect("internal error");

        let scorer = HistoricalIntegrityScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have CODEOWNERS points
        let e1 = result.sub_scores.iter().find(|s| s.id == "E1").expect("internal error");
        assert!(e1.earned >= 2.0);
    }

    #[test]
    fn test_project_with_design_docs() {
        let temp_dir = tempdir().expect("internal error");

        // Create design documents
        fs::create_dir_all(temp_dir.path().join("docs/specifications")).expect("internal error");
        fs::write(
            temp_dir.path().join("docs/specifications/feature.md"),
            "# Feature Spec",
        )
        .expect("internal error");

        let scorer = HistoricalIntegrityScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have pre-registration points
        let e2 = result.sub_scores.iter().find(|s| s.id == "E2").expect("internal error");
        assert!(e2.earned >= 2.0);
    }

    #[test]
    fn test_project_with_changelog_and_semver() {
        let temp_dir = tempdir().expect("internal error");

        // Create dated CHANGELOG
        fs::write(
            temp_dir.path().join("CHANGELOG.md"),
            "# Changelog\n\n## [1.0.0] - 2024-01-15\n\n- Initial release",
        )
        .expect("internal error");

        // Create Cargo.toml with version
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"",
        )
        .expect("internal error");

        let scorer = HistoricalIntegrityScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have timestamping points
        let e3 = result.sub_scores.iter().find(|s| s.id == "E3").expect("internal error");
        assert!(e3.earned >= 2.0);
    }
}
