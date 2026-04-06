impl DocumentationScorer {
    /// Score README quality (5pts)
    /// Checks for comprehensive README with key sections
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for README.md
    fn score_readme(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let readme_path = project_path.join("README.md");

        if !readme_path.exists() {
            return Ok(0.0);
        }

        // Try cache first, fall back to filesystem
        let content = if let Some(cache) = cache {
            cache
                .get(&readme_path)
                .cloned()
                .ok_or_else(|| ScorerError::IoError("README.md not in cache".to_string()))?
        } else {
            std::fs::read_to_string(&readme_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

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
}
