// Score aggregation and recommendation generation

use crate::services::repo_score::models::*;
use crate::services::repo_score::error::Result;
use crate::services::repo_score::scorers::*;
use crate::services::repo_score::bonus::BonusDetector;
use std::path::Path;
use std::time::Instant;

pub struct ScoreAggregator;

impl ScoreAggregator {
    pub fn new() -> Self {
        Self
    }

    /// Aggregate all scores for a repository
    pub async fn aggregate(&self, repo_path: &Path, config: &ScorerConfig) -> Result<RepoScore> {
        let start = Instant::now();

        // Run all scorers
        let readme_scorer = ReadmeScorer::new();
        let precommit_scorer = PrecommitScorer::new();
        let hygiene_scorer = HygieneScorer::new();
        let makefile_scorer = MakefileScorer::new();
        let ci_scorer = CiScorer::new();
        let pmat_scorer = PmatScorer::new();

        let documentation = readme_scorer.score(repo_path, config).await?;
        let precommit_hooks = precommit_scorer.score(repo_path, config).await?;
        let repository_hygiene = hygiene_scorer.score(repo_path, config).await?;
        let build_test_automation = makefile_scorer.score(repo_path, config).await?;
        let continuous_integration = ci_scorer.score(repo_path, config).await?;
        let pmat_compliance = pmat_scorer.score(repo_path, config).await?;

        let categories = CategoryScores {
            documentation,
            precommit_hooks,
            repository_hygiene,
            build_test_automation,
            continuous_integration,
            pmat_compliance,
        };

        // Detect bonus points
        let bonus_detector = BonusDetector::new();
        let bonus = bonus_detector.detect(repo_path).await?;

        // Calculate final scores
        let total_score = categories.total();
        let bonus_points = bonus.total();
        let final_score = total_score + bonus_points;
        let grade = Grade::from_score(final_score);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&categories, &bonus);

        // Create metadata
        let mut metadata = ScoreMetadata::new(repo_path.to_path_buf());
        metadata.execution_time_ms = start.elapsed().as_millis() as u64;

        // Try to get git context
        if let Ok(git_branch) = self.get_git_branch(repo_path) {
            metadata.git_branch = Some(git_branch);
        }
        if let Ok(git_commit) = self.get_git_commit(repo_path) {
            metadata.git_commit = Some(git_commit);
        }

        Ok(RepoScore {
            total_score,
            bonus_points,
            final_score,
            grade,
            categories,
            bonus,
            recommendations,
            metadata,
        })
    }

    /// Generate recommendations based on findings
    fn generate_recommendations(&self, categories: &CategoryScores, bonus: &BonusScores) -> Vec<Recommendation> {
        let mut recommendations = vec![];

        // Check each category for failures
        if categories.documentation.status == ScoreStatus::Fail {
            recommendations.push(Recommendation {
                priority: Priority::Critical,
                category: "Documentation".to_string(),
                title: "Add comprehensive README.md".to_string(),
                description: "Your repository is missing a complete README.md with required sections (Overview, Installation, Usage, License, Contributing).".to_string(),
                impact_points: 20.0 - categories.documentation.score,
                estimated_effort: "30 minutes".to_string(),
                commands: vec![
                    "# Create README.md with all required sections".to_string(),
                    "touch README.md".to_string(),
                ],
            });
        }

        if categories.precommit_hooks.status == ScoreStatus::Fail {
            recommendations.push(Recommendation {
                priority: Priority::High,
                category: "Pre-commit Hooks".to_string(),
                title: "Install pre-commit hooks".to_string(),
                description: "Add a pre-commit hook to run linting before commits.".to_string(),
                impact_points: 20.0 - categories.precommit_hooks.score,
                estimated_effort: "15 minutes".to_string(),
                commands: vec![
                    "mkdir -p .git/hooks".to_string(),
                    "cat > .git/hooks/pre-commit << 'EOF'\n#!/bin/bash\ncargo clippy -- -D warnings\nEOF".to_string(),
                    "chmod +x .git/hooks/pre-commit".to_string(),
                ],
            });
        }

        if categories.repository_hygiene.status == ScoreStatus::Fail || categories.repository_hygiene.status == ScoreStatus::Warning {
            if categories.repository_hygiene.score < 10.0 {
                recommendations.push(Recommendation {
                    priority: Priority::Medium,
                    category: "Repository Hygiene".to_string(),
                    title: "Clean up repository files".to_string(),
                    description: "Remove cruft files (.tmp, .bak) and team-specific files (.idea/, .vscode/). Add them to .gitignore.".to_string(),
                    impact_points: 10.0 - categories.repository_hygiene.score,
                    estimated_effort: "10 minutes".to_string(),
                    commands: vec![
                        "# Remove temporary files".to_string(),
                        "find . -name '*.tmp' -delete".to_string(),
                        "find . -name '*.bak' -delete".to_string(),
                        "# Add to .gitignore".to_string(),
                        "echo '.idea/' >> .gitignore".to_string(),
                        "echo '.vscode/' >> .gitignore".to_string(),
                    ],
                });
            }
        }

        if categories.build_test_automation.status == ScoreStatus::Fail {
            recommendations.push(Recommendation {
                priority: Priority::Critical,
                category: "Build & Test".to_string(),
                title: "Create Makefile with required targets".to_string(),
                description: "Add a Makefile with targets: test-fast, test, lint, coverage".to_string(),
                impact_points: 25.0 - categories.build_test_automation.score,
                estimated_effort: "1 hour".to_string(),
                commands: vec![
                    "# Create Makefile".to_string(),
                    "cat > Makefile << 'EOF'\n.PHONY: test-fast test lint coverage\n\ntest-fast:\n\tcargo test --lib\n\ntest:\n\tcargo test\n\nlint:\n\tcargo clippy -- -D warnings\n\ncoverage:\n\tcargo llvm-cov --html\nEOF".to_string(),
                ],
            });
        }

        if categories.continuous_integration.status == ScoreStatus::Fail {
            recommendations.push(Recommendation {
                priority: Priority::High,
                category: "CI/CD".to_string(),
                title: "Add GitHub Actions workflow".to_string(),
                description: "Create a CI workflow to run tests and linting on every push".to_string(),
                impact_points: 20.0 - categories.continuous_integration.score,
                estimated_effort: "30 minutes".to_string(),
                commands: vec![
                    "mkdir -p .github/workflows".to_string(),
                    "# Create ci.yml workflow file".to_string(),
                ],
            });
        }

        if categories.pmat_compliance.status == ScoreStatus::Fail {
            recommendations.push(Recommendation {
                priority: Priority::Medium,
                category: "PMAT Compliance".to_string(),
                title: "Add PMAT quality gates configuration".to_string(),
                description: "Create .pmat-gates.toml with quality thresholds".to_string(),
                impact_points: 5.0 - categories.pmat_compliance.score,
                estimated_effort: "15 minutes".to_string(),
                commands: vec![
                    "cat > .pmat-gates.toml << 'EOF'\n[complexity]\nmax_complexity = 10\n\n[coverage]\nminimum_coverage = 80.0\nEOF".to_string(),
                ],
            });
        }

        // Suggest bonus features if not detected
        if !bonus.property_tests.detected {
            recommendations.push(Recommendation {
                priority: Priority::Low,
                category: "Bonus: Property Testing".to_string(),
                title: "Add property-based testing".to_string(),
                description: "Implement property-based tests with proptest for +3 bonus points".to_string(),
                impact_points: 3.0,
                estimated_effort: "2 hours".to_string(),
                commands: vec![
                    "cargo add proptest --dev".to_string(),
                ],
            });
        }

        if !bonus.fuzzing.detected {
            recommendations.push(Recommendation {
                priority: Priority::Low,
                category: "Bonus: Fuzzing".to_string(),
                title: "Add fuzzing tests".to_string(),
                description: "Set up cargo-fuzz for +2 bonus points".to_string(),
                impact_points: 2.0,
                estimated_effort: "1 hour".to_string(),
                commands: vec![
                    "cargo install cargo-fuzz".to_string(),
                    "cargo fuzz init".to_string(),
                ],
            });
        }

        if !bonus.mutation_testing.detected {
            recommendations.push(Recommendation {
                priority: Priority::Low,
                category: "Bonus: Mutation Testing".to_string(),
                title: "Add mutation testing".to_string(),
                description: "Set up cargo-mutants for +2 bonus points".to_string(),
                impact_points: 2.0,
                estimated_effort: "30 minutes".to_string(),
                commands: vec![
                    "cargo install cargo-mutants".to_string(),
                    "cargo mutants --list".to_string(),
                ],
            });
        }

        if !bonus.living_docs.detected {
            recommendations.push(Recommendation {
                priority: Priority::Low,
                category: "Bonus: Living Documentation".to_string(),
                title: "Create mdBook documentation".to_string(),
                description: "Set up mdBook for living documentation (+3 bonus points)".to_string(),
                impact_points: 3.0,
                estimated_effort: "2 hours".to_string(),
                commands: vec![
                    "cargo install mdbook".to_string(),
                    "mdbook init".to_string(),
                ],
            });
        }

        // Sort by priority (Critical > High > Medium > Low)
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));

        recommendations
    }

    fn get_git_branch(&self, repo_path: &Path) -> Result<String> {
        let git_head = repo_path.join(".git/HEAD");
        if git_head.exists() {
            let content = std::fs::read_to_string(git_head)?;
            if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
                return Ok(branch.trim().to_string());
            }
        }
        Ok("unknown".to_string())
    }

    fn get_git_commit(&self, repo_path: &Path) -> Result<String> {
        let git_head = repo_path.join(".git/HEAD");
        if git_head.exists() {
            let content = std::fs::read_to_string(git_head)?;
            if content.starts_with("ref:") {
                // Read the ref file
                if let Some(ref_path) = content.strip_prefix("ref: ") {
                    let ref_file = repo_path.join(".git").join(ref_path.trim());
                    if ref_file.exists() {
                        let commit = std::fs::read_to_string(ref_file)?;
                        return Ok(commit.trim().to_string());
                    }
                }
            } else {
                // Direct commit hash
                return Ok(content.trim().to_string());
            }
        }
        Ok("unknown".to_string())
    }
}

impl Default for ScoreAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_temp_repo() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_file(repo_path: &Path, relative_path: &str, content: &str) {
        let file_path = repo_path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, content).unwrap();
    }

    #[tokio::test]
    async fn test_aggregator_empty_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Empty repo should score very low
        assert!(result.total_score < 20.0);
        assert_eq!(result.bonus_points, 0.0);
        assert_eq!(result.grade, Grade::F);
        assert!(!result.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_aggregator_perfect_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create perfect repository structure
        create_file(repo_path, "README.md", "# Project\n## Overview\n## Installation\n## Usage\n## License\n## Contributing");
        create_file(repo_path, ".git/hooks/pre-commit", "#!/bin/bash\ncargo clippy");
        create_file(repo_path, "Makefile", ".PHONY: test-fast test lint coverage\ntest-fast:\n\tcargo test\ntest:\n\tcargo test\nlint:\n\tcargo clippy\ncoverage:\n\tcargo llvm-cov");
        create_file(repo_path, ".github/workflows/ci.yml", "name: CI\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo test");
        create_file(repo_path, ".pmat-gates.toml", "[complexity]\nmax_complexity = 10");
        create_file(repo_path, "Cargo.toml", "[dependencies]\nproptest = \"1.0\"");
        create_file(repo_path, "book.toml", "[book]");
        fs::create_dir_all(repo_path.join("fuzz")).unwrap();
        create_file(repo_path, "mutants.toml", "[mutants]");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let hook_path = repo_path.join(".git/hooks/pre-commit");
            let mut perms = fs::metadata(&hook_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).unwrap();
        }

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Should score very high (100 base + up to 10 bonus)
        assert!(result.total_score >= 80.0);
        assert!(result.bonus_points > 0.0);
        assert!(result.final_score >= 80.0);
    }

    #[tokio::test]
    async fn test_aggregator_grade_assignment() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Minimal repo for F grade
        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Should be F grade
        assert_eq!(result.grade, Grade::F);
    }

    #[tokio::test]
    async fn test_aggregator_recommendations_generated() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Should have recommendations for missing components
        assert!(!result.recommendations.is_empty());
        assert!(result.recommendations.iter().any(|r| r.category.contains("Documentation")));
    }

    #[tokio::test]
    async fn test_aggregator_metadata_populated() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Metadata should be populated
        assert_eq!(result.metadata.repository_path, repo_path);
        assert_eq!(result.metadata.spec_version, "1.0.0");
    }

    #[tokio::test]
    async fn test_aggregator_bonus_detection() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Add some bonus features
        create_file(repo_path, "Cargo.toml", "[dependencies]\nproptest = \"1.0\"");

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        assert!(result.bonus_points > 0.0);
        assert!(result.bonus.property_tests.detected);
    }

    #[tokio::test]
    async fn test_aggregator_recommendation_priority() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Recommendations should be sorted by priority (Critical first)
        if result.recommendations.len() > 1 {
            assert!(result.recommendations[0].priority >= result.recommendations[1].priority);
        }
    }

    #[tokio::test]
    async fn test_aggregator_score_calculation() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Verify score calculation
        let calculated_total = result.categories.total();
        assert_eq!(result.total_score, calculated_total);
        assert_eq!(result.final_score, result.total_score + result.bonus_points);
    }
}
