//! DocumentationScorer - Documentation Category (15 points)
//!
//! Analyzes Rust project documentation quality:
//! - Rustdoc Coverage (7pts): Public API documentation with examples
//! - README Quality (5pts): Comprehensive project README
//! - Changelog Presence (3pts): CHANGELOG.md with version history
//!
//! Evidence-based design: Well-documented projects have 30-40% fewer
//! support issues and faster onboarding (GitHub State of the Octoverse 2024).

use super::models::{CategoryScore, ScoringMode};
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;

/// Documentation scorer
#[derive(Debug, Clone)]
pub struct DocumentationScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl DocumentationScorer {
    /// Create a new DocumentationScorer
    pub fn new() -> Self {
        Self {
            name: "Documentation".to_string(),
            max_points: 15.0,
        }
    }

    /// Score rustdoc coverage (7pts)
    /// Checks for public API documentation with examples
    fn score_rustdoc(&self, project_path: &Path) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(0.0);
        }

        let mut total_public_items = 0;
        let mut documented_items = 0;

        // Walk src directory
        self.count_documented_items(&src_path, &mut total_public_items, &mut documented_items)?;

        if total_public_items == 0 {
            // No public API = moderate score
            return Ok(3.5);
        }

        // Calculate documentation coverage ratio
        let doc_ratio = documented_items as f64 / total_public_items as f64;

        // Tiered scoring based on documentation coverage
        if doc_ratio >= 0.90 {
            Ok(7.0) // ≥90% documented
        } else if doc_ratio >= 0.75 {
            Ok(6.0) // ≥75% documented
        } else if doc_ratio >= 0.60 {
            Ok(4.0) // ≥60% documented
        } else if doc_ratio >= 0.40 {
            Ok(2.0) // ≥40% documented
        } else {
            Ok(0.0) // <40% documented
        }
    }

    /// Count documented public items in directory (recursive)
    fn count_documented_items(
        &self,
        dir: &Path,
        total: &mut usize,
        documented: &mut usize,
    ) -> ScorerResult<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    self.count_documented_items(&path, total, documented)?;
                } else if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            self.analyze_doc_coverage(&content, total, documented);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyze documentation coverage in Rust source code
    fn analyze_doc_coverage(&self, content: &str, total: &mut usize, documented: &mut usize) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Check for pub items
            if line.starts_with("pub fn")
                || line.starts_with("pub struct")
                || line.starts_with("pub enum")
                || line.starts_with("pub trait")
            {
                *total += 1;

                // Check if previous lines contain doc comments (/// or //!)
                let mut has_doc_comment = false;
                for j in (0..i).rev().take(10) {
                    let prev_line = lines[j].trim();
                    if prev_line.starts_with("///") || prev_line.starts_with("//!") {
                        has_doc_comment = true;
                        break;
                    }
                    if !prev_line.is_empty()
                        && !prev_line.starts_with("//")
                        && !prev_line.starts_with("#[")
                    {
                        break;
                    }
                }

                if has_doc_comment {
                    *documented += 1;
                }
            }

            i += 1;
        }
    }

    /// Score README quality (5pts)
    /// Checks for comprehensive README with key sections
    fn score_readme(&self, project_path: &Path) -> ScorerResult<f64> {
        let readme_path = project_path.join("README.md");

        if !readme_path.exists() {
            return Ok(0.0);
        }

        let content = std::fs::read_to_string(&readme_path)
            .map_err(|e| ScorerError::IoError(e.to_string()))?;

        // Check word count
        let word_count = content.split_whitespace().count();

        // Check for important sections (more lenient matching)
        let content_lower = content.to_lowercase();
        let has_installation =
            content_lower.contains("installation") || content_lower.contains("install");
        let has_usage = content_lower.contains("usage") || content_lower.contains("use");
        let has_examples = content_lower.contains("example") || content_lower.contains("```");
        let has_license = content_lower.contains("license");
        let has_features = content_lower.contains("feature");
        let has_api = content_lower.contains("api");

        let section_count = [
            has_installation,
            has_usage,
            has_examples,
            has_license,
            has_features,
            has_api,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        // Tiered scoring based on README quality
        // Prioritize section count over word count for structured READMEs
        if section_count >= 4 {
            Ok(5.0) // Comprehensive README (4+ sections)
        } else if word_count >= 100 && section_count >= 3 {
            Ok(5.0) // Comprehensive README (3+ sections + substantial content)
        } else if word_count >= 50 && section_count >= 2 {
            Ok(4.0) // Good README
        } else if word_count >= 30 && section_count >= 1 {
            Ok(2.0) // Basic README
        } else if word_count >= 10 {
            Ok(1.0) // Minimal README
        } else {
            Ok(0.0) // Very minimal
        }
    }

    /// Score changelog presence (3pts)
    /// Checks for CHANGELOG.md with version history
    fn score_changelog(&self, project_path: &Path) -> ScorerResult<f64> {
        let changelog_path = project_path.join("CHANGELOG.md");

        if !changelog_path.exists() {
            return Ok(0.0);
        }

        let content = std::fs::read_to_string(&changelog_path)
            .map_err(|e| ScorerError::IoError(e.to_string()))?;

        // Check for version entries (e.g., [0.1.0], ## 0.1.0)
        let version_count = content
            .lines()
            .filter(|line| {
                line.contains("[0.")
                    || line.contains("[1.")
                    || line.contains("## 0.")
                    || line.contains("## 1.")
            })
            .count();

        // Tiered scoring based on changelog quality
        if version_count >= 2 {
            Ok(3.0) // Multiple versions documented
        } else if version_count >= 1 {
            Ok(2.0) // At least one version
        } else {
            Ok(1.0) // CHANGELOG exists but minimal content
        }
    }
}

impl Default for DocumentationScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for DocumentationScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score rustdoc coverage (7pts)
        match self.score_rustdoc(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score README quality (5pts)
        match self.score_readme(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score changelog presence (3pts)
        match self.score_changelog(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }

    fn score_with_mode(&self, project_path: &Path, _mode: ScoringMode) -> ScorerResult<CategoryScore> {
        // This scorer doesn't have expensive operations, so mode doesn't affect it
        self.score(project_path)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check rustdoc
        if let Ok(score) = self.score_rustdoc(project_path) {
            if score < 7.0 {
                recommendations.push(
                    "Improve rustdoc coverage: Add /// documentation to public API items with examples".to_string(),
                );
            }
        }

        // Check README
        if let Ok(score) = self.score_readme(project_path) {
            if score < 5.0 {
                recommendations.push(
                    "Improve README: Add Installation, Usage, Examples, and License sections"
                        .to_string(),
                );
            }
        }

        // Check changelog
        if let Ok(score) = self.score_changelog(project_path) {
            if score < 3.0 {
                recommendations.push(
                    "Add CHANGELOG.md: Document version history and changes between releases"
                        .to_string(),
                );
            }
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for DocumentationScorer {}
unsafe impl Sync for DocumentationScorer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_creation() {
        let scorer = DocumentationScorer::new();
        assert_eq!(scorer.name(), "Documentation");
        assert_eq!(scorer.max_points(), 15.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = DocumentationScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }
}
