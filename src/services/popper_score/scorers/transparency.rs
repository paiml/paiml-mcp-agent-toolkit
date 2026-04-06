#![cfg_attr(coverage_nightly, coverage(off))]
//! Category C: Transparency & Openness (20 points)
//!
//! Popper's requirement for scrutiny necessitates visibility into methods.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | C1 | License Clarity | 5 | OSI-approved license |
//! | C2 | Documentation Accuracy | 8 | Documentation drift detection |
//! | C3 | Design Rationale | 7 | ADRs or design documents |
//!
//! ## Academic Foundation
//!
//! - OSI License Guidelines
//! - Tyree & Akerman (2005): ADR format [10]
//! - Brooks (1995): Design decisions [16]

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{PopperScorer, PopperScorerResult};
use std::path::Path;

/// OSI-approved licenses (common ones)
const OSI_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "Apache 2.0",
    "GPL-3.0",
    "GPL-2.0",
    "BSD-3-Clause",
    "BSD-2-Clause",
    "ISC",
    "MPL-2.0",
    "LGPL-3.0",
    "LGPL-2.1",
    "Unlicense",
    "0BSD",
    "CC0-1.0",
    "Zlib",
    "BSL-1.0",
    "EPL-2.0",
];

/// Scorer for Category C: Transparency & Openness (20 points)
pub struct TransparencyScorer;

impl TransparencyScorer {
    /// Create a new transparency scorer
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }

    /// C1: License Clarity (5 points)
    ///
    /// Checks for:
    /// - LICENSE file exists (2 points)
    /// - OSI-approved license (3 points)
    fn score_license_clarity(&self, project_path: &Path) -> PopperSubScore {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let mut earned: f64 = 0.0;
        let max: f64 = 5.0;
        let mut description: Vec<String> = Vec::new();

        // Check for LICENSE file
        let license_files = [
            "LICENSE",
            "LICENSE.md",
            "LICENSE.txt",
            "LICENCE",
            "LICENCE.md",
            "COPYING",
        ];

        let mut license_content = None;
        for license_file in license_files {
            let path = project_path.join(license_file);
            if path.exists() {
                earned += 2.0;
                description.push(format!("{} exists", license_file));
                license_content = std::fs::read_to_string(&path).ok();
                break;
            }
        }

        // Check for OSI-approved license
        if let Some(content) = &license_content {
            let content_upper = content.to_uppercase();
            for license in OSI_LICENSES {
                if content_upper.contains(&license.to_uppercase()) {
                    earned += 3.0;
                    description.push("OSI-approved license".to_string());
                    break;
                }
            }
        }

        // Also check Cargo.toml or package.json for license field
        let cargo_toml = project_path.join("Cargo.toml");
        if cargo_toml.exists() && earned < 5.0 {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                for license in OSI_LICENSES {
                    if content.contains(license) {
                        earned = (earned + 1.0).min(max);
                        description.push("license in Cargo.toml".to_string());
                        break;
                    }
                }
            }
        }

        if description.is_empty() {
            description.push("no license found".to_string());
        }

        PopperSubScore::new(
            "C1",
            "License Clarity",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// C2: Documentation Accuracy (8 points)
    ///
    /// Checks for:
    /// - README exists with content (2 points)
    /// - API documentation (3 points)
    /// - Code comments present (2 points)
    /// - Changelog exists (1 point)
    fn score_documentation_accuracy(&self, project_path: &Path) -> PopperSubScore {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let mut earned: f64 = 0.0;
        let max: f64 = 8.0;
        let mut description: Vec<String> = Vec::new();

        // Check README (2 points)
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                if content.len() > 500 {
                    earned += 2.0;
                    description.push("comprehensive README".to_string());
                } else if content.len() > 100 {
                    earned += 1.0;
                    description.push("basic README".to_string());
                }
            }
        }

        // Check for API documentation (3 points)
        // Rust: doc comments, TypeScript: TSDoc, Python: docstrings
        if self.has_api_docs(project_path) {
            earned += 3.0;
            description.push("API documentation found".to_string());
        }

        // Check for code comments (2 points)
        if self.has_code_comments(project_path) {
            earned += 2.0;
            description.push("code comments present".to_string());
        }

        // Check for CHANGELOG (1 point)
        let changelog_files = ["CHANGELOG.md", "CHANGELOG", "CHANGES.md", "HISTORY.md"];
        for changelog in changelog_files {
            if project_path.join(changelog).exists() {
                earned += 1.0;
                description.push("CHANGELOG exists".to_string());
                break;
            }
        }

        if description.is_empty() {
            description.push("minimal documentation".to_string());
        }

        PopperSubScore::new(
            "C2",
            "Documentation Accuracy",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// C3: Design Rationale (7 points)
    ///
    /// Checks for:
    /// - ADRs (Architecture Decision Records) (4 points)
    /// - Design documents (2 points)
    /// - Contributing guidelines (1 point)
    fn score_design_rationale(&self, project_path: &Path) -> PopperSubScore {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let mut earned: f64 = 0.0;
        let max: f64 = 7.0;
        let mut description: Vec<String> = Vec::new();

        // Check for ADR directory (4 points)
        let adr_dirs = [
            "docs/adr",
            "docs/adrs",
            "adr",
            "decisions",
            "docs/decisions",
        ];
        for adr_dir in adr_dirs {
            let path = project_path.join(adr_dir);
            if path.exists() && path.is_dir() {
                earned += 4.0;
                description.push("ADRs found".to_string());
                break;
            }
        }

        // Check for design documents (2 points)
        let design_docs = [
            "DESIGN.md",
            "ARCHITECTURE.md",
            "docs/design.md",
            "docs/architecture.md",
            "SPEC.md",
            "docs/spec.md",
        ];
        for doc in design_docs {
            if project_path.join(doc).exists() {
                earned += 2.0;
                description.push("design documentation found".to_string());
                break;
            }
        }

        // Check for CONTRIBUTING (1 point)
        let contributing_files = ["CONTRIBUTING.md", "CONTRIBUTING", ".github/CONTRIBUTING.md"];
        for contrib in contributing_files {
            if project_path.join(contrib).exists() {
                earned += 1.0;
                description.push("CONTRIBUTING guide exists".to_string());
                break;
            }
        }

        if description.is_empty() {
            description.push("no design rationale documentation".to_string());
        }

        PopperSubScore::new(
            "C3",
            "Design Rationale",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// Check if project has API documentation
    fn has_api_docs(&self, project_path: &Path) -> bool {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        // Check for docs directory
        if project_path.join("docs").exists() {
            return true;
        }

        // Check for rustdoc/typedoc/etc config
        let doc_configs = [
            ".rustdoc.json",
            "typedoc.json",
            "docs.json",
            ".readthedocs.yml",
            ".readthedocs.yaml",
            "mkdocs.yml",
        ];
        for config in doc_configs {
            if project_path.join(config).exists() {
                return true;
            }
        }

        // Check Rust files for doc comments
        let src_dir = project_path.join("src");
        if src_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&src_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "rs").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("///") || content.contains("//!") {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if project has meaningful code comments
    fn has_code_comments(&self, project_path: &Path) -> bool {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let src_dir = project_path.join("src");
        if !src_dir.exists() {
            return false;
        }

        if let Ok(entries) = std::fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Count comment lines
                        let comment_lines = content
                            .lines()
                            .filter(|l| {
                                let trimmed = l.trim();
                                trimmed.starts_with("//")
                                    || trimmed.starts_with("#")
                                    || trimmed.starts_with("/*")
                                    || trimmed.starts_with("*")
                            })
                            .count();

                        let total_lines = content.lines().count();
                        if total_lines > 0 && comment_lines as f64 / total_lines as f64 > 0.05 {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }
}

impl Default for TransparencyScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for TransparencyScorer {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "Transparency & Openness"
    }

    fn category_id(&self) -> char {
        'C'
    }

    fn max_points(&self) -> f64 {
        20.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());

        // Score each sub-category
        let c1 = self.score_license_clarity(project_path);
        let c2 = self.score_documentation_accuracy(project_path);
        let c3 = self.score_design_rationale(project_path);

        // Add findings based on scores
        if c1.earned < 3.0 {
            category.add_finding(PopperFinding::warning(
                "License clarity needs attention - add an OSI-approved LICENSE file",
                5.0 - c1.earned,
            ));
        } else {
            category.add_finding(PopperFinding::positive("Clear OSI-approved license"));
        }

        if c2.earned < 5.0 {
            category.add_finding(PopperFinding::warning(
                "Documentation could be more comprehensive",
                8.0 - c2.earned,
            ));
        }

        if c3.earned < 3.0 {
            category.add_finding(PopperFinding::warning(
                "Consider adding ADRs to document design decisions",
                7.0 - c3.earned,
            ));
        }

        // Add sub-scores
        category.add_sub_score(c1);
        category.add_sub_score(c2);
        category.add_sub_score(c3);

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
    fn test_transparency_scorer_basics() {
        let scorer = TransparencyScorer::new();
        assert_eq!(scorer.name(), "Transparency & Openness");
        assert_eq!(scorer.category_id(), 'C');
        assert_eq!(scorer.max_points(), 20.0);
        assert!(!scorer.is_gateway());
    }

    #[test]
    fn test_project_with_mit_license() {
        let temp_dir = tempdir().unwrap();

        // Create MIT LICENSE
        fs::write(
            temp_dir.path().join("LICENSE"),
            "MIT License\n\nCopyright (c) 2025",
        )
        .unwrap();

        let scorer = TransparencyScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have full license points
        let c1 = result.sub_scores.iter().find(|s| s.id == "C1").unwrap();
        assert_eq!(c1.earned, 5.0);
    }

    #[test]
    fn test_project_with_comprehensive_docs() {
        let temp_dir = tempdir().unwrap();

        // Create comprehensive README
        fs::write(
            temp_dir.path().join("README.md"),
            &"# Project\n\n".repeat(100), // Long content
        )
        .unwrap();

        // Create CHANGELOG
        fs::write(
            temp_dir.path().join("CHANGELOG.md"),
            "# Changelog\n\n## v1.0.0\n",
        )
        .unwrap();

        // Create docs directory
        fs::create_dir_all(temp_dir.path().join("docs")).unwrap();

        let scorer = TransparencyScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have documentation points
        let c2 = result.sub_scores.iter().find(|s| s.id == "C2").unwrap();
        assert!(c2.earned >= 4.0);
    }

    #[test]
    fn test_project_with_adr() {
        let temp_dir = tempdir().unwrap();

        // Create ADR directory
        fs::create_dir_all(temp_dir.path().join("docs/adr")).unwrap();
        fs::write(
            temp_dir.path().join("docs/adr/0001-use-rust.md"),
            "# Use Rust\n\n## Context\n",
        )
        .unwrap();

        let scorer = TransparencyScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have ADR points
        let c3 = result.sub_scores.iter().find(|s| s.id == "C3").unwrap();
        assert!(c3.earned >= 4.0);
    }

    #[test]
    fn test_project_with_contributing() {
        let temp_dir = tempdir().unwrap();

        // Create CONTRIBUTING.md
        fs::write(
            temp_dir.path().join("CONTRIBUTING.md"),
            "# Contributing\n\nHow to contribute...",
        )
        .unwrap();

        let scorer = TransparencyScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have contributing points
        let c3 = result.sub_scores.iter().find(|s| s.id == "C3").unwrap();
        assert!(c3.earned >= 1.0);
    }
}
