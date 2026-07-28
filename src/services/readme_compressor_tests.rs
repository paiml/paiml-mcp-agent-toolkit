// Tests for ReadmeCompressor
// Included by readme_compressor.rs — no `use` imports or `#!` attributes allowed

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_basic_readme() {
        let content = r#"# My Project

[![Build Status](https://travis-ci.org/user/project.svg)](https://travis-ci.org/user/project)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A powerful tool for developers that simplifies complex workflows.

## Features

- Fast performance with async processing
- Intelligent caching system
- Plugin architecture for extensibility
- Cross-platform support

## Installation

```bash
npm install -g myproject
```

## Usage

Basic usage:

```bash
myproject analyze --path ./src
```

## Architecture

The system is built on a modular architecture with three main components:

1. **Core Engine**: Handles the main processing logic
2. **Plugin System**: Allows for extensibility
3. **Cache Layer**: Improves performance

## Contributing

Please read CONTRIBUTING.md for details.

## License

MIT
"#;

        let compressor = ReadmeCompressor::new();
        let result = compressor.compress(content);

        // Should extract project description
        assert!(result.project_description.is_some());
        assert!(result
            .project_description
            .as_ref()
            .expect("internal error")
            .contains("A powerful tool for developers"));

        // Should extract key features
        assert!(!result.key_features.is_empty());
        assert!(result
            .key_features
            .iter()
            .any(|f| f.contains("Fast performance")));
        assert!(result
            .key_features
            .iter()
            .any(|f| f.contains("Intelligent caching")));

        // Should include high-value sections
        let section_titles: Vec<&str> = result.sections.iter().map(|s| s.title.as_str()).collect();
        assert!(section_titles.contains(&"Architecture"));
        assert!(section_titles.contains(&"Features"));

        // Should exclude low-value sections
        assert!(!section_titles.contains(&"Contributing"));
        assert!(!section_titles.contains(&"License"));

        // Architecture section should be included with content
        let arch_section = result
            .sections
            .iter()
            .find(|s| s.title == "Architecture")
            .expect("internal error");
        assert!(arch_section.content.contains("modular architecture"));
        assert!(arch_section.content.contains("Core Engine"));
    }

    #[test]
    fn test_section_scoring() {
        let compressor = ReadmeCompressor::new();

        // High-value sections
        let arch_section = Section {
            title: "Architecture Overview".to_string(),
            level: 2,
            paragraphs: vec!["Some content".to_string()],
            lists: vec![],
            code_snippets: vec![],
        };
        assert_eq!(compressor.calculate_section_score(&arch_section), 0.9);

        // Medium-value sections
        let usage_section = Section {
            title: "Usage".to_string(),
            level: 2,
            paragraphs: vec!["Some content".to_string()],
            lists: vec![],
            code_snippets: vec![],
        };
        assert_eq!(compressor.calculate_section_score(&usage_section), 0.6);

        // Low-value sections
        let faq_section = Section {
            title: "FAQ".to_string(),
            level: 2,
            paragraphs: vec!["Some content".to_string()],
            lists: vec![],
            code_snippets: vec![],
        };
        assert_eq!(compressor.calculate_section_score(&faq_section), 0.3);

        // Very low-value sections
        let license_section = Section {
            title: "License".to_string(),
            level: 2,
            paragraphs: vec!["MIT".to_string()],
            lists: vec![],
            code_snippets: vec![],
        };
        assert_eq!(compressor.calculate_section_score(&license_section), 0.1);

        // Top-level section with content
        let main_section = Section {
            title: "Overview".to_string(),
            level: 1,
            paragraphs: vec!["Important content".to_string()],
            lists: vec![],
            code_snippets: vec![],
        };
        assert_eq!(compressor.calculate_section_score(&main_section), 0.9);
    }

    #[test]
    fn test_truncate_intelligently() {
        let compressor = ReadmeCompressor::new();

        // Test sentence boundary truncation
        let text = "This is a sentence. This is another sentence. This won't fit.";
        let truncated = compressor.truncate_intelligently(text, 46); // "This is a sentence. This is another sentence." is 46 chars
        assert_eq!(truncated, "This is a sentence. This is another sentence.");

        // Test word boundary truncation
        let text = "This is a very long sentence without periods that needs truncation";
        let truncated = compressor.truncate_intelligently(text, 30);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 30);

        // Test short text (no truncation needed)
        let text = "Short text";
        let truncated = compressor.truncate_intelligently(text, 50);
        assert_eq!(truncated, "Short text");
    }

    #[test]
    fn test_extract_project_description() {
        let compressor = ReadmeCompressor::new();

        // Test with badges at the top
        let content = r#"# Project

[![Badge1](url)](link)
[![Badge2](url)](link)

This is the main project description that explains what this project does.

## Installation
"#;
        let desc = compressor
            .extract_project_description(content)
            .expect("internal error");
        assert!(desc.contains("This is the main project description"));

        // Test without badges
        let content2 = r#"# Project

A simple tool for doing things efficiently.

## Features
"#;
        let desc2 = compressor
            .extract_project_description(content2)
            .expect("internal error");
        assert!(desc2.contains("A simple tool for doing things"));

        // Test empty content
        let content3 = r#"# Project

## Installation
"#;
        let desc3 = compressor.extract_project_description(content3);
        assert!(desc3.is_none());
    }

    #[test]
    fn test_markdown_parsing() {
        let compressor = ReadmeCompressor::new();
        let content = r#"# Main Title

First paragraph under main title.

## Section 1

Section 1 content.

### Subsection 1.1

- Item 1
- Item 2
- Item 3

## Section 2

Another paragraph.

```rust
fn main() {
    println!("Hello");
}
```ignore
"#;

        let sections = compressor.parse_markdown_sections(content);

        // Should have correct number of sections
        assert_eq!(sections.len(), 4);

        // Check main title section
        assert_eq!(sections[0].title, "Main Title");
        assert_eq!(sections[0].level, 1);
        assert_eq!(sections[0].paragraphs.len(), 1);
        assert!(sections[0].paragraphs[0].contains("First paragraph"));

        // Check section with list
        let subsection = sections
            .iter()
            .find(|s| s.title == "Subsection 1.1")
            .expect("internal error");
        assert_eq!(subsection.lists.len(), 1);
        assert_eq!(subsection.lists[0].items.len(), 3);
        assert_eq!(subsection.lists[0].items[0], "Item 1");
    }

    #[test]
    fn test_feature_extraction() {
        let compressor = ReadmeCompressor::new();
        let mut features = Vec::new();

        let section = Section {
            title: "Features".to_string(),
            level: 2,
            paragraphs: vec![
                "The system provides automatic backup functionality.".to_string(),
                "It enables real-time synchronization across devices.".to_string(),
            ],
            lists: vec![List {
                items: vec![
                    "Fast processing with multi-threading".to_string(),
                    "Intelligent caching for improved performance".to_string(),
                    "x".to_string(), // Too short, should be ignored
                    "Plugin system for extensibility".to_string(),
                ],
            }],
            code_snippets: vec![],
        };

        compressor.extract_features_from_section(&section, &mut features);

        // Should extract features from lists
        assert!(features.iter().any(|f| f.contains("Fast processing")));
        assert!(features.iter().any(|f| f.contains("Intelligent caching")));
        assert!(features.iter().any(|f| f.contains("Plugin system")));

        // Should extract from paragraphs with feature keywords
        assert!(features.iter().any(|f| f.contains("automatic backup")));
        assert!(features
            .iter()
            .any(|f| f.contains("real-time synchronization")));

        // Should not include too short items
        assert!(!features.iter().any(|f| f == "x"));
    }

    #[test]
    fn test_compress_section_with_budget() {
        let compressor = ReadmeCompressor::new();

        let section = Section {
            title: "Overview".to_string(),
            level: 2,
            paragraphs: vec![
                "This is a very long paragraph that contains a lot of information about the project. It goes on and on with many details that might need to be truncated to fit within the token budget.".to_string(),
            ],
            lists: vec![
                List {
                    items: vec![
                        "Feature 1".to_string(),
                        "Feature 2".to_string(),
                        "Feature 3".to_string(),
                        "Feature 4".to_string(),
                        "Feature 5".to_string(),
                        "Feature 6".to_string(),
                        "Feature 7".to_string(),
                    ],
                },
            ],
            code_snippets: vec![],
        };

        let compressed = compressor.compress_section(&section, 100);
        assert_eq!(compressed.title, "Overview");

        // Should include truncated content
        assert!(compressed.content.len() <= 400); // 100 tokens * 4 chars

        // Should include first paragraph
        assert!(compressed.content.contains("This is a very long paragraph"));

        // Should limit list items
        assert!(compressed.content.contains("- Feature 1"));
        assert!(compressed.content.contains("- Feature 5"));
        assert!(compressed.content.contains("- ..."));
    }
}

