// ReadmeScorer - Category A: Documentation Quality (15 points)
//
// Scores based on:
// - A1: README Accuracy (5 points) - No broken links, valid images
// - A2: README Comprehensiveness (5 points) - Required sections present
// - A3: Professional Structure (5 points) - Hero image, ToC, centered header, no bot patterns

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct ReadmeScorer;

impl ReadmeScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score README accuracy (A1: 5 points)
    /// Checks for broken links and valid image references
    async fn score_accuracy(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A1".to_string(),
                name: "README Accuracy".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: "README.md not found".to_string(),
                    location: Some(readme_path.display().to_string()),
                    impact_points: -5.0,
                }],
            });
        }

        let content = tokio::fs::read_to_string(&readme_path).await?;
        let mut score: f64 = 5.0;
        let mut findings = vec![];

        // Check file is not empty
        if content.trim().is_empty() {
            return Ok(SubcategoryScore {
                id: "A1".to_string(),
                name: "README Accuracy".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: "README.md is empty".to_string(),
                    location: Some(readme_path.display().to_string()),
                    impact_points: -5.0,
                }],
            });
        }

        // Check for broken image references
        let image_pattern =
            regex::Regex::new(r#"!\[[^\]]*\]\(([^)]+)\)|<img[^>]+src=["']([^"']+)["']"#).unwrap();
        for cap in image_pattern.captures_iter(&content) {
            let img_path = cap
                .get(1)
                .or_else(|| cap.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");

            // Skip external URLs
            if img_path.starts_with("http://") || img_path.starts_with("https://") {
                continue;
            }

            // Check if local file exists
            let full_path = repo_path.join(img_path);
            if !full_path.exists() {
                score -= 1.0;
                findings.push(Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: format!("Broken image link: {}", img_path),
                    location: Some("README.md".to_string()),
                    impact_points: -1.0,
                });
            }
        }

        // Check for broken relative links
        let link_pattern = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        for cap in link_pattern.captures_iter(&content) {
            let link_path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Skip external URLs, anchors, and mailto
            if link_path.starts_with("http://")
                || link_path.starts_with("https://")
                || link_path.starts_with("#")
                || link_path.starts_with("mailto:")
            {
                continue;
            }

            // Remove anchor from path
            let clean_path = link_path.split('#').next().unwrap_or(link_path);
            if clean_path.is_empty() {
                continue;
            }

            let full_path = repo_path.join(clean_path);
            if !full_path.exists() {
                score -= 0.5;
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Documentation".to_string(),
                    message: format!("Broken link: {}", link_path),
                    location: Some("README.md".to_string()),
                    impact_points: -0.5,
                });
            }
        }

        score = score.max(0.0);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "All links and images valid".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "A1".to_string(),
            name: "README Accuracy".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }

    /// Score README comprehensiveness (A2: 5 points)
    async fn score_comprehensiveness(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A2".to_string(),
                name: "README Comprehensiveness".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&readme_path).await?;

        // Required sections (1.0 points each, 5 sections = 5 points)
        let required_sections = vec![
            (
                "Project Description",
                vec![
                    r"(?i)##\s*(overview|about|description)",
                    r"(?i)#\s+[^#\n]+\n\n[^#]", // Project title followed by description
                ],
            ),
            ("Installation", vec![r"(?i)##\s*install(ation)?"]),
            (
                "Usage",
                vec![r"(?i)##\s*(usage|getting\s+started|quick\s*start)"],
            ),
            (
                "License",
                vec![r"(?i)##\s*license", r"(?i)\bMIT\b", r"(?i)\bApache\b"],
            ),
            (
                "Contributing",
                vec![r"(?i)##\s*contribut(ing|e)", r"(?i)CONTRIBUTING\.md"],
            ),
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
                score += 1.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Documentation".to_string(),
                    message: format!("{} section found", section_name),
                    location: Some("README.md".to_string()),
                    impact_points: 1.0,
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
            max_score: 5.0,
            findings,
        })
    }

    /// Score professional structure (A3: 5 points)
    /// Detects professional README patterns vs bot-generated/stream-of-consciousness
    async fn score_professional_structure(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A3".to_string(),
                name: "Professional Structure".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&readme_path).await?;
        let mut score = 0.0;
        let mut findings = vec![];

        // 1. Hero image present (1.5 points)
        let has_hero_image = self.check_hero_image(repo_path, &content).await;
        if has_hero_image {
            score += 1.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Hero image present".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 1.5,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Documentation".to_string(),
                message: "Hero image missing (add docs/hero.svg or image at top of README)"
                    .to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        // 2. Centered header (0.5 points)
        let has_centered_header =
            content.contains("<p align=\"center\">") || content.contains("<h1 align=\"center\">");
        if has_centered_header {
            score += 0.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Professional centered header".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.5,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Documentation".to_string(),
                message: "Consider using centered header for professional look".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        // 3. Table of Contents (1.0 point)
        let has_toc = regex::Regex::new(r"(?i)##?\s*(table\s+of\s+contents|contents|toc)")
            .map(|re| re.is_match(&content))
            .unwrap_or(false)
            || content.matches("](#").count() >= 4; // Multiple anchor links indicate ToC

        if has_toc {
            score += 1.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Table of Contents present".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 1.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Documentation".to_string(),
                message: "Table of Contents missing".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        // 4. Features section with bullet points or table (0.5 points)
        let has_features_section = regex::Regex::new(r"(?i)##\s*features")
            .map(|re| re.is_match(&content))
            .unwrap_or(false);
        if has_features_section {
            score += 0.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Features section present".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.5,
            });
        }

        // 5. No stream-of-consciousness patterns (1.5 points)
        // Bot-generated READMEs often have release notes, changelogs, or version history at the top
        let first_50_lines: String = content.lines().take(50).collect::<Vec<_>>().join("\n");
        let stream_of_consciousness_patterns = [
            r"(?i)##?\s*(current\s+release|what'?s\s+new|v?\d+\.\d+\.\d+\s*[-:])",
            r"(?i)##?\s*(changelog|release\s+notes|version\s+history)",
            r"(?i)##?\s*(latest\s+bug\s+fixes|recent\s+changes)",
            r"(?i)\*\*major\s+feature\*\*",
            r"(?i)##?\s*previous\s+release",
        ];

        let mut is_stream_of_consciousness = false;
        for pattern in stream_of_consciousness_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&first_50_lines) {
                    is_stream_of_consciousness = true;
                    break;
                }
            }
        }

        if !is_stream_of_consciousness {
            score += 1.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Professional structure (no release notes at top)".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 1.5,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Documentation".to_string(),
                message: "Stream-of-consciousness pattern detected: release notes/changelog at top of README appears bot-generated. Move to CHANGELOG.md".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "A3".to_string(),
            name: "Professional Structure".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }

    /// Check if a hero image is present and valid
    async fn check_hero_image(&self, repo_path: &Path, content: &str) -> bool {
        // Check for hero.svg in standard locations
        let hero_paths = [
            "docs/hero.svg",
            "docs/hero.png",
            ".github/hero.svg",
            ".github/hero.png",
            "assets/hero.svg",
            "assets/hero.png",
        ];

        for path in hero_paths {
            if repo_path.join(path).exists() {
                return true;
            }
        }

        // Check if README starts with an image (within first 20 lines)
        let first_20_lines: String = content.lines().take(20).collect::<Vec<_>>().join("\n");
        regex::Regex::new(r#"<img[^>]+src=|!\[[^\]]*\]\("#)
            .map(|re| re.is_match(&first_20_lines))
            .unwrap_or(false)
    }
}

#[async_trait]
impl Scorer for ReadmeScorer {
    fn category_name(&self) -> &str {
        "Documentation"
    }

    fn max_score(&self) -> f64 {
        15.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        let a1 = self.score_accuracy(repo_path).await?;
        let a2 = self.score_comprehensiveness(repo_path).await?;
        let a3 = self.score_professional_structure(repo_path).await?;

        let total_score = a1.score + a2.score + a3.score;

        let mut findings = a1.findings.clone();
        findings.extend(a2.findings.clone());
        findings.extend(a3.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![a1, a2, a3],
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
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_readme(repo_path: &std::path::Path, content: &str) {
        let readme_path = repo_path.join("README.md");
        fs::write(readme_path, content).unwrap()
    }

    fn create_hero_image(repo_path: &std::path::Path) {
        let docs_dir = repo_path.join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(docs_dir.join("hero.svg"), "<svg></svg>").unwrap();
    }

    const PROFESSIONAL_README: &str = r#"<p align="center">
  <img src="docs/hero.svg" alt="project" width="800">
</p>

<h1 align="center">Project Name</h1>

<p align="center">
  <b>A professional project description.</b>
</p>

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Features

- Feature one
- Feature two

## Installation

```bash
cargo install project
```

## Usage

```rust
use project::run;
```

## Contributing

See CONTRIBUTING.md

## License

MIT License
"#;

    const BOT_GENERATED_README: &str = r#"# Project

## 🎉 Current Release: v3.20.0 - Major Feature!

**Major Feature** - Something new!

### What's New in v3.20.0

- Feature A
- Feature B

### Latest Bug Fixes (v3.19.1)

- Fixed something

### Previous Release: v3.19.0

More stuff here...

## Installation

cargo install project
"#;

    const MINIMAL_README: &str = r#"
# Test Project
Just a title.
"#;

    #[tokio::test]
    async fn test_professional_readme_full_score() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, PROFESSIONAL_README);
        create_hero_image(repo_path);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should get close to full score
        assert!(
            result.score >= 14.0,
            "Professional README should score >= 14.0, got {}",
            result.score
        );
        assert_eq!(result.subcategories.len(), 3);
    }

    #[tokio::test]
    async fn test_bot_generated_readme_low_score() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, BOT_GENERATED_README);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Bot-generated should lose points for:
        // - No hero image (-1.5)
        // - No centered header (-0.5)
        // - No ToC (-1.0)
        // - Stream of consciousness pattern (-1.5)
        // A3 should score ~0.5/5.0
        let a3 = result.subcategories.iter().find(|s| s.id == "A3").unwrap();
        assert!(
            a3.score <= 1.0,
            "Bot-generated README A3 should score <= 1.0, got {}",
            a3.score
        );

        // Check for stream-of-consciousness warning
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.message.contains("Stream-of-consciousness")
                    || f.message.contains("bot-generated")),
            "Should warn about bot-generated pattern"
        );
    }

    #[tokio::test]
    async fn test_readme_missing_file() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        assert_eq!(result.score, 0.0);
        assert_eq!(result.max_score, 15.0);
    }

    #[tokio::test]
    async fn test_broken_image_link_detection() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let readme_with_broken_image = r#"# Project

![Hero](docs/nonexistent.png)

## Installation

cargo install project
"#;
        create_readme(repo_path, readme_with_broken_image);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should have finding about broken image
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.message.contains("Broken image link")),
            "Should detect broken image link"
        );
    }

    #[tokio::test]
    async fn test_hero_image_detection() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, MINIMAL_README);
        create_hero_image(repo_path);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should detect hero image from docs/hero.svg
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.message.contains("Hero image present")),
            "Should detect hero image in docs/"
        );
    }

    #[tokio::test]
    async fn test_toc_detection_via_anchors() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let readme_with_toc = r#"# Project

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [License](#license)

## Features
...
"#;
        create_readme(repo_path, readme_with_toc);

        let scorer = ReadmeScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should detect ToC via anchor links
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.message.contains("Table of Contents present")),
            "Should detect ToC via anchor links"
        );
    }

    #[tokio::test]
    async fn test_category_name_and_max_score() {
        let scorer = ReadmeScorer::new();
        assert_eq!(scorer.category_name(), "Documentation");
        assert_eq!(scorer.max_score(), 15.0);
    }
}
