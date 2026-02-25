impl SatdDetector {
    pub fn new() -> Self {
        Self {
            patterns: SATD_PATTERNS.clone(),
            extended: false,
        }
    }

    /// Create a detector with extended patterns (euphemism detection)
    /// This catches hidden technical debt like "placeholder", "stub", "for now"
    pub fn with_extended() -> Self {
        let mut patterns = SATD_PATTERNS.clone();
        patterns.extend(EXTENDED_PATTERNS.clone());
        Self {
            patterns,
            extended: true,
        }
    }

    /// Check if extended mode is enabled
    pub fn is_extended(&self) -> bool {
        self.extended
    }

    pub fn detect(&self, source: &str) -> SatdResult {
        let mut count = 0;
        let mut found_patterns = Vec::new();

        for (pattern_name, regex) in &self.patterns {
            let matches = regex.find_iter(source).count();
            if matches > 0 {
                count += matches;
                if !found_patterns.contains(&pattern_name.to_string()) {
                    found_patterns.push(pattern_name.to_string());
                }
            }
        }

        SatdResult {
            count,
            patterns: found_patterns,
        }
    }

    pub fn detect_in_comments(&self, source: &str) -> SatdResult {
        // Extract only comments from source
        let comments = self.extract_comments(source);
        self.detect(&comments)
    }

    fn extract_comments(&self, source: &str) -> String {
        let mut in_block_comment = false;
        let mut comments = String::new();
        let lines = source.lines();

        for line in lines {
            let trimmed = line.trim();

            // Block comment handling
            if trimmed.starts_with("/*") {
                in_block_comment = true;
                comments.push_str(line);
                comments.push('\n');
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            if in_block_comment {
                comments.push_str(line);
                comments.push('\n');
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            // Line comment handling
            if let Some(comment_start) = line.find("//") {
                comments.push_str(line.get(comment_start..).unwrap_or_default());
                comments.push('\n');
            }
        }

        comments
    }
}
