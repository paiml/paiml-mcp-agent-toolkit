//! Category B: Reproducibility Infrastructure (25 points)
//!
//! Independent verification requires exact environment replication.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | B1 | Artifact Availability | 10 | Locked dependencies, DOI/archival |
//! | B2 | Environment Reproducibility | 8 | Nix/Guix (bonus) or Docker |
//! | B3 | Result Reproduction | 7 | Single-command full reproduction |
//!
//! ## Academic Foundation
//!
//! - ACM Artifact Evaluation Badging (2020) [3]
//! - Wilkinson et al. (2016): FAIR Principles [5]
//! - Boettiger (2015): Docker for Reproducibility [8]

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{PopperScorer, PopperScorerResult};
use std::path::Path;

/// Scorer for Category B: Reproducibility Infrastructure (25 points)
pub struct ReproducibilityScorer;

impl ReproducibilityScorer {
    /// Create a new reproducibility scorer
    pub fn new() -> Self {
        Self
    }

    /// B1: Artifact Availability (10 points)
    ///
    /// Checks for:
    /// - Lock files for dependencies (5 points)
    /// - DOI or archival reference (3 points)
    /// - Release artifacts (2 points)
    fn score_artifact_availability(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 10.0;
        let mut description = Vec::new();

        // Check for lock files (5 points)
        let lock_files = [
            ("Cargo.lock", 5.0, "Cargo.lock found"),
            ("package-lock.json", 5.0, "package-lock.json found"),
            ("yarn.lock", 5.0, "yarn.lock found"),
            ("pnpm-lock.yaml", 5.0, "pnpm-lock.yaml found"),
            ("poetry.lock", 5.0, "poetry.lock found"),
            ("Pipfile.lock", 5.0, "Pipfile.lock found"),
            ("Gemfile.lock", 5.0, "Gemfile.lock found"),
            ("go.sum", 5.0, "go.sum found"),
            ("flake.lock", 5.0, "flake.lock found"),
        ];

        let mut lock_found = false;
        for (file, points, desc) in lock_files {
            if project_path.join(file).exists() {
                earned += points;
                description.push(desc);
                lock_found = true;
                break;
            }
        }

        // Check for DOI or archival (3 points)
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let content_lower = content.to_lowercase();
                if content.contains("10.") && content.contains("/")
                    || content_lower.contains("doi")
                    || content_lower.contains("zenodo")
                    || content_lower.contains("figshare")
                    || content_lower.contains("osf.io")
                {
                    earned += 3.0;
                    description.push("DOI/archival reference found");
                }
            }
        }

        // Check CITATION.cff (2 points)
        if project_path.join("CITATION.cff").exists() {
            earned += 2.0;
            description.push("CITATION.cff found");
        } else if project_path.join("CITATION").exists() {
            earned += 1.0;
            description.push("CITATION file found");
        }

        if !lock_found && description.is_empty() {
            description.push("no lock files or archival references");
        }

        PopperSubScore::new(
            "B1",
            "Artifact Availability",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// B2: Environment Reproducibility (8 points)
    ///
    /// Checks for:
    /// - Nix/Guix flake (5 points - BONUS over Docker)
    /// - Docker/Containerfile (3 points)
    /// - Dev container config (2 points)
    fn score_environment_reproducibility(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 8.0;
        let mut description: Vec<String> = Vec::new();

        // Nix/Guix hermetic builds (5 points - preferred per Heijunka)
        let nix_files = ["flake.nix", "shell.nix", "default.nix"];
        for nix_file in nix_files {
            if project_path.join(nix_file).exists() {
                earned += 5.0;
                description.push(format!("{} found (hermetic)", nix_file));
                break;
            }
        }

        // Guix (5 points)
        if project_path.join("guix.scm").exists() {
            earned = earned.max(5.0);
            description.push("guix.scm found (hermetic)".to_string());
        }

        // Docker (3 points - less preferred)
        if earned < 3.0 {
            let docker_files = ["Dockerfile", "docker-compose.yml", "docker-compose.yaml"];
            for docker_file in docker_files {
                if project_path.join(docker_file).exists() {
                    earned = earned.max(3.0);
                    description.push(format!("{} found", docker_file));
                    break;
                }
            }

            // Containerfile (3 points)
            if project_path.join("Containerfile").exists() {
                earned = earned.max(3.0);
                description.push("Containerfile found".to_string());
            }
        }

        // Dev container (2 points)
        let devcontainer_paths = [".devcontainer/devcontainer.json", ".devcontainer.json"];
        for devcontainer in devcontainer_paths {
            if project_path.join(devcontainer).exists() {
                earned = (earned + 2.0).min(max);
                description.push("devcontainer config found".to_string());
                break;
            }
        }

        if description.is_empty() {
            description.push("no environment reproducibility config".to_string());
        }

        PopperSubScore::new(
            "B2",
            "Environment Reproducibility",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// B3: Result Reproduction (7 points)
    ///
    /// Checks for:
    /// - Single-command build (2 points)
    /// - Single-command test (2 points)
    /// - Makefile/justfile with documented targets (2 points)
    /// - Documented reproduction steps (1 point)
    fn score_result_reproduction(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 7.0;
        let mut description = Vec::new();

        // Check for Makefile/justfile (2 points)
        if project_path.join("Makefile").exists() {
            earned += 2.0;
            description.push("Makefile found");
        } else if project_path.join("justfile").exists() {
            earned += 2.0;
            description.push("justfile found");
        }

        // Check for standard build commands (2 points)
        let cargo_toml = project_path.join("Cargo.toml");
        let package_json = project_path.join("package.json");
        let pyproject = project_path.join("pyproject.toml");

        if cargo_toml.exists() || package_json.exists() || pyproject.exists() {
            earned += 2.0;
            description.push("standard build config found");
        }

        // Check README for reproduction steps (2 points)
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let content_lower = content.to_lowercase();

                // Check for installation/build instructions
                if content_lower.contains("## install")
                    || content_lower.contains("## build")
                    || content_lower.contains("## getting started")
                    || content_lower.contains("```bash")
                    || content_lower.contains("```shell")
                {
                    earned += 2.0;
                    description.push("installation instructions found");
                }

                // Check for reproduction/usage instructions (1 point)
                if content_lower.contains("## usage")
                    || content_lower.contains("## reproduce")
                    || content_lower.contains("## quickstart")
                {
                    earned += 1.0;
                    description.push("usage documentation found");
                }
            }
        }

        if description.is_empty() {
            description.push("no reproduction infrastructure");
        }

        PopperSubScore::new(
            "B3",
            "Result Reproduction",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }
}

impl Default for ReproducibilityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for ReproducibilityScorer {
    fn name(&self) -> &str {
        "Reproducibility Infrastructure"
    }

    fn category_id(&self) -> char {
        'B'
    }

    fn max_points(&self) -> f64 {
        25.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());

        // Score each sub-category
        let b1 = self.score_artifact_availability(project_path);
        let b2 = self.score_environment_reproducibility(project_path);
        let b3 = self.score_result_reproduction(project_path);

        // Add findings based on scores
        if b1.earned < 5.0 {
            category.add_finding(PopperFinding::warning(
                "Artifact availability needs improvement - add lock files and archival references",
                10.0 - b1.earned,
            ));
        } else {
            category.add_finding(PopperFinding::positive("Good artifact availability"));
        }

        if b2.earned < 3.0 {
            category.add_finding(PopperFinding::warning(
                "Consider adding Nix/Docker for environment reproducibility",
                8.0 - b2.earned,
            ));
        } else if b2.earned >= 5.0 {
            category.add_finding(PopperFinding::positive(
                "Excellent hermetic builds with Nix/Guix",
            ));
        }

        if b3.earned < 4.0 {
            category.add_finding(PopperFinding::warning(
                "Result reproduction could be easier - document build steps",
                7.0 - b3.earned,
            ));
        }

        // Add sub-scores
        category.add_sub_score(b1);
        category.add_sub_score(b2);
        category.add_sub_score(b3);

        Ok(category)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_reproducibility_scorer_basics() {
        let scorer = ReproducibilityScorer::new();
        assert_eq!(scorer.name(), "Reproducibility Infrastructure");
        assert_eq!(scorer.category_id(), 'B');
        assert_eq!(scorer.max_points(), 25.0);
        assert!(!scorer.is_gateway());
    }

    #[test]
    fn test_project_with_cargo_lock() {
        let temp_dir = tempdir().unwrap();

        // Create Cargo.lock
        fs::write(temp_dir.path().join("Cargo.lock"), "# Lock file").unwrap();

        let scorer = ReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have earned points for lock file
        let b1 = result.sub_scores.iter().find(|s| s.id == "B1").unwrap();
        assert!(b1.earned >= 5.0);
    }

    #[test]
    fn test_project_with_nix_flake() {
        let temp_dir = tempdir().unwrap();

        // Create flake.nix
        fs::write(temp_dir.path().join("flake.nix"), "{ outputs = ... }").unwrap();

        let scorer = ReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have earned bonus points for Nix
        let b2 = result.sub_scores.iter().find(|s| s.id == "B2").unwrap();
        assert!(b2.earned >= 5.0);
    }

    #[test]
    fn test_project_with_dockerfile() {
        let temp_dir = tempdir().unwrap();

        // Create Dockerfile
        fs::write(temp_dir.path().join("Dockerfile"), "FROM rust:1.70").unwrap();

        let scorer = ReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have earned points for Docker (less than Nix)
        let b2 = result.sub_scores.iter().find(|s| s.id == "B2").unwrap();
        assert!(b2.earned >= 3.0);
        assert!(b2.earned < 5.0);
    }

    #[test]
    fn test_project_with_makefile() {
        let temp_dir = tempdir().unwrap();

        // Create Makefile
        fs::write(
            temp_dir.path().join("Makefile"),
            "build:\n\tcargo build\ntest:\n\tcargo test",
        )
        .unwrap();

        let scorer = ReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have earned points for Makefile
        let b3 = result.sub_scores.iter().find(|s| s.id == "B3").unwrap();
        assert!(b3.earned >= 2.0);
    }

    #[test]
    fn test_full_reproducibility_setup() {
        let temp_dir = tempdir().unwrap();

        // Create complete reproducibility setup
        fs::write(temp_dir.path().join("Cargo.lock"), "# Lock").unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(temp_dir.path().join("flake.nix"), "{ }").unwrap();
        fs::write(temp_dir.path().join("Makefile"), "build:\n\t").unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            "# Project\n## Installation\n```bash\nmake build\n```\n## Usage\nRun it",
        )
        .unwrap();
        fs::write(temp_dir.path().join("CITATION.cff"), "cff-version: 1.2.0").unwrap();

        let scorer = ReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have high score
        assert!(result.earned > 15.0);
    }
}
