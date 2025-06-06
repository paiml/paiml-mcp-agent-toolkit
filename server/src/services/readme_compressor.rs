use crate::models::project_meta::{CompressedReadme, CompressedSection};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::collections::HashMap;
use tracing::debug;

pub struct ReadmeCompressor {
    section_importance: HashMap<String, f32>,
    #[allow(dead_code)]
    max_section_tokens: usize,
}

impl ReadmeCompressor {
    pub fn new() -> Self {
        let mut section_importance = HashMap::new();

        // High-value sections (0.9)
        section_importance.insert("overview".to_string(), 0.9);
        section_importance.insert("architecture".to_string(), 0.9);
        section_importance.insert("api".to_string(), 0.9);
        section_importance.insert("philosophy".to_string(), 0.9);
        section_importance.insert("core concepts".to_string(), 0.9);
        section_importance.insert("design principles".to_string(), 0.9);

        // Medium-value sections (0.6)
        section_importance.insert("features".to_string(), 0.6);
        section_importance.insert("usage".to_string(), 0.6);
        section_importance.insert("quickstart".to_string(), 0.6);
        section_importance.insert("getting started".to_string(), 0.6);
        section_importance.insert("installation".to_string(), 0.6);
        section_importance.insert("configuration".to_string(), 0.6);

        // Low-value sections (0.3)
        section_importance.insert("examples".to_string(), 0.3);
        section_importance.insert("troubleshooting".to_string(), 0.3);
        section_importance.insert("faq".to_string(), 0.3);

        // Very low-value sections (0.1) - will be filtered
        section_importance.insert("badges".to_string(), 0.1);
        section_importance.insert("license".to_string(), 0.1);
        section_importance.insert("contributing".to_string(), 0.1);
        section_importance.insert("changelog".to_string(), 0.1);
        section_importance.insert("acknowledgments".to_string(), 0.1);
        section_importance.insert("sponsors".to_string(), 0.1);

        Self {
            section_importance,
            max_section_tokens: 500, // ~2KB assuming 4 chars per token
        }
    }

    pub fn compress(&self, content: &str) -> CompressedReadme {
        let sections = self.parse_markdown_sections(content);
        let mut scored_sections = Vec::new();

        // Phase 1: Score sections
        for section in sections {
            let score = self.calculate_section_score(&section);
            if score > 0.3 {
                scored_sections.push((section, score));
            }
        }

        // Phase 2: Sort by importance
        scored_sections.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Phase 3: Allocate token budget
        let mut token_budget = 2000; // Target ~2KB compressed
        let mut result = CompressedReadme::default();

        // Extract project description from first paragraph or overview
        if let Some(desc) = self.extract_project_description(content) {
            result.project_description = Some(desc);
            token_budget -= 100; // Reserve tokens for description
        }

        // Compress sections within budget
        for (section, _score) in scored_sections {
            if token_budget < 100 {
                break;
            }

            let compressed = self.compress_section(&section, token_budget);
            let estimated_tokens = compressed.content.len() / 4; // Rough estimate

            // Extract key features from feature sections
            if section.title.to_lowercase().contains("feature") {
                self.extract_features_from_section(&section, &mut result.key_features);
            }

            token_budget = token_budget.saturating_sub(estimated_tokens);
            result.sections.push(compressed);
        }

        debug!(
            "Compressed README: {} sections, {} key features",
            result.sections.len(),
            result.key_features.len()
        );

        result
    }

    fn handle_heading(
        &self,
        level: u8,
        current_section: &mut Option<Section>,
        sections: &mut Vec<Section>,
        text_buffer: &mut String,
    ) {
        // Save previous section if exists
        if let Some(mut section) = current_section.take() {
            if !text_buffer.is_empty() {
                section.paragraphs.push(text_buffer.clone());
                text_buffer.clear();
            }
            sections.push(section);
        }
        *current_section = Some(Section {
            title: String::new(),
            level,
            paragraphs: Vec::new(),
            lists: Vec::new(),
            code_snippets: Vec::new(),
        });
    }

    fn handle_text(
        &self,
        text: &str,
        current_section: &mut Option<Section>,
        in_list: bool,
        list_items: &mut Vec<String>,
        in_code_block: bool,
        text_buffer: &mut String,
    ) {
        if let Some(ref mut section) = current_section {
            if section.title.is_empty() {
                section.title = text.to_string();
            } else if in_list {
                list_items.push(text.to_string());
            } else if !in_code_block {
                text_buffer.push_str(text);
            }
        }
    }

    fn handle_list_end(&self, current_section: &mut Option<Section>, list_items: &mut Vec<String>) {
        if let Some(ref mut section) = current_section {
            if !list_items.is_empty() {
                section.lists.push(List {
                    items: list_items.clone(),
                });
                list_items.clear();
            }
        }
    }

    fn handle_paragraph_end(
        &self,
        current_section: &mut Option<Section>,
        text_buffer: &mut String,
    ) {
        if let Some(ref mut section) = current_section {
            if !text_buffer.is_empty() {
                section.paragraphs.push(text_buffer.clone());
                text_buffer.clear();
            }
        }
    }

    fn parse_markdown_sections(&self, content: &str) -> Vec<Section> {
        let parser = Parser::new(content);
        let mut sections = Vec::new();
        let mut current_section: Option<Section> = None;
        let mut in_list = false;
        let mut list_items = Vec::new();
        let mut in_code_block = false;
        let mut text_buffer = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    self.handle_heading(
                        level as u8,
                        &mut current_section,
                        &mut sections,
                        &mut text_buffer,
                    );
                }
                Event::Text(text) => {
                    self.handle_text(
                        &text,
                        &mut current_section,
                        in_list,
                        &mut list_items,
                        in_code_block,
                        &mut text_buffer,
                    );
                }
                Event::Start(Tag::List(_)) => {
                    in_list = true;
                    list_items.clear();
                }
                Event::End(TagEnd::List(_)) => {
                    in_list = false;
                    self.handle_list_end(&mut current_section, &mut list_items);
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    in_code_block = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                }
                Event::SoftBreak | Event::HardBreak => {
                    text_buffer.push(' ');
                }
                Event::End(TagEnd::Paragraph) => {
                    self.handle_paragraph_end(&mut current_section, &mut text_buffer);
                }
                _ => {}
            }
        }

        // Save last section
        if let Some(mut section) = current_section {
            if !text_buffer.is_empty() {
                section.paragraphs.push(text_buffer);
            }
            sections.push(section);
        }

        sections
    }

    fn calculate_section_score(&self, section: &Section) -> f32 {
        let title_lower = section.title.to_lowercase();

        // Check exact matches first
        for (key, &score) in &self.section_importance {
            if title_lower.contains(key) {
                return score;
            }
        }

        // Additional heuristics
        if section.level == 1 && !section.paragraphs.is_empty() {
            return 0.7; // Top-level sections with content
        }

        if !section.lists.is_empty() && title_lower.contains("feature") {
            return 0.7; // Feature lists are valuable
        }

        0.4 // Default score
    }

    fn compress_section(&self, section: &Section, budget: usize) -> CompressedSection {
        let mut content = String::new();
        let max_chars = budget * 4; // Rough estimate of 4 chars per token

        // Include first paragraph (usually the summary)
        if let Some(first_para) = section.paragraphs.first() {
            let trimmed = self.truncate_intelligently(first_para, max_chars / 2);
            content.push_str(&trimmed);
        }

        // Include key bullet points
        if !section.lists.is_empty() && content.len() < max_chars {
            content.push('\n');
            for list in &section.lists {
                for (i, item) in list.items.iter().enumerate() {
                    if content.len() + item.len() > max_chars {
                        break;
                    }
                    // Only include first 5 items
                    if i >= 5 {
                        content.push_str("- ...\n");
                        break;
                    }
                    content.push_str(&format!("- {}\n", self.summarize_list_item(item)));
                }
            }
        }

        CompressedSection {
            title: section.title.clone(),
            content: content.trim().to_string(),
        }
    }

    fn truncate_intelligently(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            return text.to_string();
        }

        // Try to break at sentence boundary
        let truncated = &text[..max_len];
        if let Some(pos) = truncated.rfind(". ") {
            return text[..pos + 1].to_string(); // Include the period
        }

        // Fall back to word boundary
        if let Some(pos) = truncated.rfind(' ') {
            let word_truncated = &text[..pos];
            if word_truncated.len() + 3 <= max_len {
                return format!("{word_truncated}...");
            }
        }

        // Hard truncation with ellipsis
        let truncate_len = max_len.saturating_sub(3);
        format!("{}...", &text[..truncate_len])
    }

    fn extract_project_description(&self, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();

        // Skip initial badges and empty lines
        let mut start_idx = 0;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("![")
                && !trimmed.starts_with("[![")
                && !trimmed.starts_with('#')
            {
                start_idx = i;
                break;
            }
        }

        // Extract first meaningful paragraph
        let mut description = String::new();
        for line in lines.iter().skip(start_idx).take(5) {
            let trimmed = line.trim();
            if trimmed.is_empty() && !description.is_empty() {
                break;
            }
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                if !description.is_empty() {
                    description.push(' ');
                }
                description.push_str(trimmed);
            }
        }

        if description.is_empty() {
            None
        } else {
            Some(self.truncate_intelligently(&description, 300))
        }
    }

    fn extract_features_from_section(&self, section: &Section, features: &mut Vec<String>) {
        // Extract from lists
        for list in &section.lists {
            for item in list.items.iter().take(5) {
                // Only take first 5 features
                let summarized = self.summarize_list_item(item);
                if summarized.len() > 10 && summarized.len() < 100 {
                    features.push(summarized);
                }
            }
        }

        // Extract from paragraphs with feature keywords
        for para in &section.paragraphs {
            if para.to_lowercase().contains("support")
                || para.to_lowercase().contains("provide")
                || para.to_lowercase().contains("enable")
            {
                // Extract sentences that describe features
                for sentence in para.split(". ") {
                    if sentence.len() > 20 && sentence.len() < 100 {
                        features.push(sentence.trim().to_string());
                        if features.len() >= 10 {
                            return;
                        }
                    }
                }
            }
        }
    }

    fn summarize_list_item(&self, item: &str) -> String {
        // Remove common prefixes
        let cleaned = item
            .trim_start_matches("- ")
            .trim_start_matches("* ")
            .trim_start_matches("• ");

        // Truncate very long items
        if cleaned.len() > 100 {
            self.truncate_intelligently(cleaned, 97)
        } else {
            cleaned.to_string()
        }
    }
}

impl Default for ReadmeCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Section {
    title: String,
    level: u8,
    paragraphs: Vec<String>,
    lists: Vec<List>,
    #[allow(dead_code)]
    code_snippets: Vec<String>,
}

#[derive(Debug)]
struct List {
    items: Vec<String>,
}

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
            .unwrap()
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
            .unwrap();
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
        let desc = compressor.extract_project_description(content).unwrap();
        assert!(desc.contains("This is the main project description"));

        // Test without badges
        let content2 = r#"# Project

A simple tool for doing things efficiently.

## Features
"#;
        let desc2 = compressor.extract_project_description(content2).unwrap();
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
```
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
            .unwrap();
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
