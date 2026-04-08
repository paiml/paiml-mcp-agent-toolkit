// Core implementation methods for ReadmeCompressor
// Included by readme_compressor.rs — no `use` imports or `#!` attributes allowed

impl ReadmeCompressor {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Compress.
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
        scored_sections.sort_by(|a, b| b.1.total_cmp(&a.1));

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
        let truncated = text.get(..max_len).unwrap_or(text);
        if let Some(pos) = truncated.rfind(". ") {
            return text.get(..=pos).unwrap_or(text).to_string(); // Include the period
        }

        // Fall back to word boundary
        if let Some(pos) = truncated.rfind(' ') {
            let word_truncated = text.get(..pos).unwrap_or(text);
            if word_truncated.len() + 3 <= max_len {
                return format!("{word_truncated}...");
            }
        }

        // Hard truncation with ellipsis
        let truncate_len = max_len.saturating_sub(3);
        format!("{}...", text.get(..truncate_len).unwrap_or(text))
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
