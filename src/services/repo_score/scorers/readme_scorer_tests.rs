// ReadmeScorer unit tests - all tests for accuracy, comprehensiveness, and structure scoring

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().expect("internal error")
    }

    fn create_readme(repo_path: &std::path::Path, content: &str) {
        let readme_path = repo_path.join("README.md");
        fs::write(readme_path, content).expect("internal error")
    }

    fn create_hero_image(repo_path: &std::path::Path) {
        let docs_dir = repo_path.join("docs");
        fs::create_dir_all(&docs_dir).expect("internal error");
        fs::write(docs_dir.join("hero.svg"), "<svg></svg>").expect("internal error");
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

## Current Release: v3.20.0 - Major Feature!

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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // Bot-generated should lose points for:
        // - No hero image (-1.5)
        // - No centered header (-0.5)
        // - No ToC (-1.0)
        // - Stream of consciousness pattern (-1.5)
        // A3 should score ~0.5/5.0
        let a3 = result
            .subcategories
            .iter()
            .find(|s| s.id == "A3")
            .expect("internal error");
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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

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
