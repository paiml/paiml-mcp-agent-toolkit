// ReadmeScorer - Category A: Documentation Quality (15 points)
//
// Scores based on:
// - A1: README Accuracy (7.5 points) - No broken links or code examples
// - A2: README Comprehensiveness (7.5 points) - Required sections present

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::models::*;
use crate::services::repo_score::error::Result;
use async_trait::async_trait;
use std::path::Path;

pub struct ReadmeScorer;

impl ReadmeScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score README accuracy (A1: 7.5 points)
    async fn score_accuracy(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A1".to_string(),
                name: "README Accuracy".to_string(),
                score: 0.0,
                max_score: 7.5,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: "README.md not found".to_string(),
                    location: Some(readme_path.display().to_string()),
                    impact_points: -7.5,
                }],
            });
        }

        // For now, assume no broken links (full implementation would use validate-readme)
        // TODO: Integrate with existing validate-readme functionality
        let mut score = 7.5;
        let mut findings = vec![Finding {
            severity: Severity::Success,
            category: "Documentation".to_string(),
            message: "README.md exists".to_string(),
            location: Some(readme_path.display().to_string()),
            impact_points: 0.0,
        }];

        // Check file is not empty
        let content = tokio::fs::read_to_string(&readme_path).await?;
        if content.trim().is_empty() {
            score = 0.0;
            findings.push(Finding {
                severity: Severity::Error,
                category: "Documentation".to_string(),
                message: "README.md is empty".to_string(),
                location: Some(readme_path.display().to_string()),
                impact_points: -7.5,
            });
        }

        Ok(SubcategoryScore {
            id: "A1".to_string(),
            name: "README Accuracy".to_string(),
            score,
            max_score: 7.5,
            findings,
        })
    }

    /// Score README comprehensiveness (A2: 7.5 points)
    async fn score_comprehensiveness(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A2".to_string(),
                name: "README Comprehensiveness".to_string(),
                score: 0.0,
                max_score: 7.5,
                findings: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&readme_path).await?;

        // Required sections (1.5 points each, 5 sections = 7.5 points)
        let required_sections = vec![
            ("Project Description", vec![
                r"(?i)##\s*(overview|about|description)",
                r"(?i)#\s+[^#\n]+\n\n[^#]", // Project title followed by description
            ]),
            ("Installation", vec![
                r"(?i)##\s*install(ation)?",
            ]),
            ("Usage", vec![
                r"(?i)##\s*(usage|getting\s+started|quick\s*start)",
            ]),
            ("License", vec![
                r"(?i)##\s*license",
                r"(?i)\bMIT\b",
                r"(?i)\bApache\b",
            ]),
            ("Contributing", vec![
                r"(?i)##\s*contribut(ing|e)",
                r"(?i)CONTRIBUTING\.md",
            ]),
        ];

        let mut score = 0.0;
        let mut findings = vec![];

        for (section_name, patterns) in required_sections {
            let mut found = false;
            for pattern in patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(&content) {
                        found = true;
                        break;
                    }
                }
            }

            if found {
                score += 1.5;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Documentation".to_string(),
                    message: format!("{} section found", section_name),
                    location: Some("README.md".to_string()),
                    impact_points: 1.5,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Documentation".to_string(),
                    message: format!("{} section missing", section_name),
                    location: Some("README.md".to_string()),
                    impact_points: 0.0,
                });
            }
        }

        Ok(SubcategoryScore {
            id: "A2".to_string(),
            name: "README Comprehensiveness".to_string(),
            score,
            max_score: 7.5,
            findings,
        })
    }
}

#[async_trait]
impl Scorer for ReadmeScorer {
    fn category_name(&self) -> &str {
        "Documentation Quality"
    }

    fn max_score(&self) -> f64 {
        15.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        let a1 = self.score_accuracy(repo_path).await?;
        let a2 = self.score_comprehensiveness(repo_path).await?;

        let total_score = a1.score + a2.score;

        let mut findings = a1.findings.clone();
        findings.extend(a2.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![a1, a2],
            findings,
        ))
    }
}

impl Default for ReadmeScorer {
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

    fn create_readme(repo_path: &std::path::Path, content: &str) {
        let readme_path = repo_path.join("README.md");
        fs::write(readme_path, content).unwrap();
    }

    const PERFECT_README: &str = r#"
# Test Project

## Overview
This is a comprehensive description of the project.

## Installation
```bash
cargo install test-project
```

## Usage
```rust
use test_project::run;
run();
```

## License
MIT License

## Contributing
See CONTRIBUTING.md for details.
"#;

    const MINIMAL_README: &str = r#"
# Test Project
Just a title.
"#;

    #[tokio::test]
    async fn test_readme_scorer_missing_file() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        assert_eq!(result.score, 0.0);
        assert_eq!(result.max_score, 15.0);
        assert_eq!(result.percentage, 0.0);
        assert_eq!(result.status, ScoreStatus::Fail);
        assert!(result.findings.iter().any(|f| f.message.contains("README.md not found")));
    }

    #[tokio::test]
    async fn test_readme_scorer_perfect_readme() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, PERFECT_README);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        assert_eq!(result.score, 15.0);
        assert_eq!(result.percentage, 100.0);
        assert_eq!(result.status, ScoreStatus::Pass);
        assert_eq!(result.subcategories.len(), 2);
    }

    #[tokio::test]
    async fn test_readme_scorer_minimal_readme() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, MINIMAL_README);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Minimal README: gets 7.5 points for accuracy (exists, not empty)
        // but 0-1.5 points for comprehensiveness (maybe 1 section detected)
        // Total: 7.5-9.0/15.0 = 50-60% → Fail status
        assert!(result.score >= 7.0 && result.score <= 10.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_readme_accuracy_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, PERFECT_README);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let a1 = result.subcategories.iter().find(|s| s.id == "A1").unwrap();
        assert_eq!(a1.name, "README Accuracy");
        assert_eq!(a1.score, 7.5);
        assert_eq!(a1.max_score, 7.5);
    }

    #[tokio::test]
    async fn test_readme_comprehensiveness_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, PERFECT_README);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let a2 = result.subcategories.iter().find(|s| s.id == "A2").unwrap();
        assert_eq!(a2.name, "README Comprehensiveness");
        assert_eq!(a2.score, 7.5);
        assert_eq!(a2.max_score, 7.5);
    }

    #[tokio::test]
    async fn test_readme_required_sections_detection() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let partial_readme = r#"
# Project Name

## Installation
Instructions here.

## Usage
Usage here.

## License
MIT
"#;
        create_readme(repo_path, partial_readme);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // 3/5 sections × 1.5 points = 4.5 points (comprehensiveness)
        // + 7.5 points (accuracy) = 12.0 total
        assert!(result.score >= 11.5 && result.score <= 12.5);
    }

    #[tokio::test]
    async fn test_readme_empty_file() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, "");

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Empty README should score 0
        assert_eq!(result.score, 0.0);
        assert!(result.findings.iter().any(|f| f.message.contains("empty")));
    }

    #[tokio::test]
    async fn test_readme_category_name() {
        let scorer = ReadmeScorer::new();
        assert_eq!(scorer.category_name(), "Documentation Quality");
        assert_eq!(scorer.max_score(), 15.0);
    }
}
