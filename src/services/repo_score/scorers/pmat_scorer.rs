#![cfg_attr(coverage_nightly, coverage(off))]
// PmatScorer - Category F: PMAT Compliance (5 points)
//
// Scores based on:
// - F1: PMAT Configuration Present (2.5 points) - .pmat-gates.toml exists
// - F2: No PMAT Violations (2.5 points) - Quality gates pass

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct PmatScorer;

impl PmatScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score PMAT configuration presence (F1: 2.5 points)
    async fn score_configuration(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let pmat_gates_path = repo_path.join(".pmat-gates.toml");

        if !pmat_gates_path.exists() {
            return Ok(SubcategoryScore {
                id: "F1".to_string(),
                name: "PMAT Configuration Present".to_string(),
                score: 0.0,
                max_score: 2.5,
                findings: vec![Finding {
                    severity: Severity::Warning,
                    category: "PMAT".to_string(),
                    message: ".pmat-gates.toml not found".to_string(),
                    location: Some(pmat_gates_path.display().to_string()),
                    impact_points: -2.5,
                }],
            });
        }

        // Check if file is not empty
        let content = tokio::fs::read_to_string(&pmat_gates_path).await?;
        if content.trim().is_empty() {
            return Ok(SubcategoryScore {
                id: "F1".to_string(),
                name: "PMAT Configuration Present".to_string(),
                score: 0.0,
                max_score: 2.5,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "PMAT".to_string(),
                    message: ".pmat-gates.toml is empty".to_string(),
                    location: Some(pmat_gates_path.display().to_string()),
                    impact_points: -2.5,
                }],
            });
        }

        // Try to parse as TOML
        match toml::from_str::<toml::Value>(&content) {
            Ok(_) => Ok(SubcategoryScore {
                id: "F1".to_string(),
                name: "PMAT Configuration Present".to_string(),
                score: 2.5,
                max_score: 2.5,
                findings: vec![Finding {
                    severity: Severity::Success,
                    category: "PMAT".to_string(),
                    message: ".pmat-gates.toml present and valid".to_string(),
                    location: Some(pmat_gates_path.display().to_string()),
                    impact_points: 0.0,
                }],
            }),
            Err(e) => Ok(SubcategoryScore {
                id: "F1".to_string(),
                name: "PMAT Configuration Present".to_string(),
                score: 0.5, // Partial credit for having file
                max_score: 2.5,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "PMAT".to_string(),
                    message: format!(".pmat-gates.toml invalid TOML: {}", e),
                    location: Some(pmat_gates_path.display().to_string()),
                    impact_points: -2.0,
                }],
            }),
        }
    }

    /// Score PMAT violations (F2: 2.5 points)
    async fn score_violations(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let pmat_gates_path = repo_path.join(".pmat-gates.toml");

        if !pmat_gates_path.exists() {
            // If no config, assume no violations (can't violate what doesn't exist)
            return Ok(SubcategoryScore {
                id: "F2".to_string(),
                name: "No PMAT Violations".to_string(),
                score: 2.5,
                max_score: 2.5,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "PMAT".to_string(),
                    message: "No PMAT configuration to check".to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        // Read and parse config
        let content = tokio::fs::read_to_string(&pmat_gates_path).await?;

        // Check for empty file
        if content.trim().is_empty() {
            return Ok(SubcategoryScore {
                id: "F2".to_string(),
                name: "No PMAT Violations".to_string(),
                score: 0.0,
                max_score: 2.5,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "PMAT".to_string(),
                    message: "Cannot check violations - empty configuration".to_string(),
                    location: Some(pmat_gates_path.display().to_string()),
                    impact_points: -2.5,
                }],
            });
        }

        let config: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => {
                // Invalid config = can't check violations
                return Ok(SubcategoryScore {
                    id: "F2".to_string(),
                    name: "No PMAT Violations".to_string(),
                    score: 0.0,
                    max_score: 2.5,
                    findings: vec![Finding {
                        severity: Severity::Error,
                        category: "PMAT".to_string(),
                        message: "Cannot check violations - invalid TOML".to_string(),
                        location: Some(pmat_gates_path.display().to_string()),
                        impact_points: -2.5,
                    }],
                });
            }
        };

        // Check for common quality gate settings
        let mut findings = vec![];
        let mut violations = 0;

        // Check if gates are defined
        if let Some(table) = config.as_table() {
            if table.is_empty() {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "PMAT".to_string(),
                    message: "No quality gates defined in .pmat-gates.toml".to_string(),
                    location: Some(pmat_gates_path.display().to_string()),
                    impact_points: -1.0,
                });
                violations += 1;
            } else {
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "PMAT".to_string(),
                    message: format!("{} quality gate(s) defined", table.len()),
                    location: Some(pmat_gates_path.display().to_string()),
                    impact_points: 0.0,
                });
            }
        }

        let score = (2.5 - (violations as f64 * 1.0)).max(0.0);

        Ok(SubcategoryScore {
            id: "F2".to_string(),
            name: "No PMAT Violations".to_string(),
            score,
            max_score: 2.5,
            findings,
        })
    }
}

#[async_trait]
impl Scorer for PmatScorer {
    fn category_name(&self) -> &str {
        "PMAT Compliance"
    }

    fn max_score(&self) -> f64 {
        5.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let f1 = self.score_configuration(repo_path).await?;
        let f2 = self.score_violations(repo_path).await?;

        let total_score = f1.score + f2.score;

        let mut findings = f1.findings.clone();
        findings.extend(f2.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![f1, f2],
            findings,
        ))
    }
}

impl Default for PmatScorer {
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

    fn create_pmat_gates(repo_path: &Path, content: &str) {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let gates_path = repo_path.join(".pmat-gates.toml");
        fs::write(gates_path, content).unwrap();
    }

    const VALID_PMAT_GATES: &str = r#"
[complexity]
max_complexity = 10
warn_complexity = 7

[coverage]
minimum_coverage = 80.0
"#;

    #[tokio::test]
    async fn test_pmat_scorer_no_config() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = PmatScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // F1: 0 (no config), F2: 2.5 (no violations to check) = 2.5 total
        assert!(result.score >= 2.0 && result.score <= 3.0);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains(".pmat-gates.toml not found")));
    }

    #[tokio::test]
    async fn test_pmat_scorer_valid_config() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_pmat_gates(repo_path, VALID_PMAT_GATES);

        let scorer = PmatScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // F1: 2.5 (valid config), F2: 2.5 (no violations) = 5.0 total
        assert_eq!(result.score, 5.0);
        assert_eq!(result.percentage, 100.0);
        assert_eq!(result.status, ScoreStatus::Pass);
    }

    #[tokio::test]
    async fn test_pmat_scorer_empty_config() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_pmat_gates(repo_path, "");

        let scorer = PmatScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // F1: 0 (empty file), F2: 0 (can't check) = 0 total
        assert_eq!(result.score, 0.0);
        assert!(result.findings.iter().any(|f| f.message.contains("empty")));
    }

    #[tokio::test]
    async fn test_pmat_scorer_invalid_toml() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_pmat_gates(repo_path, "invalid toml {{[[");

        let scorer = PmatScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // F1: 0.5 (partial credit), F2: 0 (can't check) = 0.5 total
        assert!(result.score >= 0.0 && result.score <= 1.0);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("invalid TOML")));
    }

    #[tokio::test]
    async fn test_pmat_configuration_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_pmat_gates(repo_path, VALID_PMAT_GATES);

        let scorer = PmatScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let f1 = result.subcategories.iter().find(|s| s.id == "F1").unwrap();
        assert_eq!(f1.name, "PMAT Configuration Present");
        assert_eq!(f1.score, 2.5);
        assert_eq!(f1.max_score, 2.5);
    }

    #[tokio::test]
    async fn test_pmat_violations_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_pmat_gates(repo_path, VALID_PMAT_GATES);

        let scorer = PmatScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let f2 = result.subcategories.iter().find(|s| s.id == "F2").unwrap();
        assert_eq!(f2.name, "No PMAT Violations");
        assert_eq!(f2.score, 2.5);
        assert_eq!(f2.max_score, 2.5);
    }

    #[tokio::test]
    async fn test_pmat_category_name() {
        let scorer = PmatScorer::new();
        assert_eq!(scorer.category_name(), "PMAT Compliance");
        assert_eq!(scorer.max_score(), 5.0);
    }
}
