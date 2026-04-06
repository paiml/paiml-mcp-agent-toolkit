// Kotlin language strategy
#[cfg(feature = "kotlin-ast")]
pub struct KotlinAstStrategy;

#[cfg(feature = "kotlin-ast")]
impl KotlinAstStrategy {
    /// Extract name from `UnifiedAstNode` by analyzing the source range
    fn extract_name_from_node(
        node: &crate::models::unified_ast::UnifiedAstNode,
        content: &str,
    ) -> Option<String> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        // For now, extract a reasonable segment from the source range
        let start = node.source_range.start as usize;
        let end = node.source_range.end as usize;

        if start >= content.len() || end > content.len() || start >= end {
            return None;
        }

        let source_text = &content[start..end];

        // Use simple heuristics to extract identifiers from the source text
        match &node.kind {
            crate::models::unified_ast::AstKind::Function(_) => {
                Self::extract_function_name(source_text)
            }
            crate::models::unified_ast::AstKind::Type(_) => Self::extract_class_name(source_text),
            _ => None,
        }
    }

    /// Extract function name from Kotlin source text
    pub(crate) fn extract_function_name(source_text: &str) -> Option<String> {
        debug_assert!(!source_text.is_empty(), "source_text must not be empty");
        // Look for pattern: fun name(...)
        if let Some(fun_pos) = source_text.find("fun ") {
            let after_fun = &source_text[fun_pos + 4..];
            if let Some(paren_pos) = after_fun.find('(') {
                let name_part = &after_fun[..paren_pos];
                // Take the first word as the function name
                return name_part
                    .split_whitespace()
                    .next()
                    .map(std::string::ToString::to_string);
            }
        }
        None
    }

    /// Extract class/interface/object name from source text
    pub(crate) fn extract_class_name(source_text: &str) -> Option<String> {
        debug_assert!(!source_text.is_empty(), "source_text must not be empty");
        // Look for patterns like "class Name", "interface Name", "object Name",
        // "data class Name", "enum class Name"
        let lines = source_text.lines().next()?; // Get first line
        let words: Vec<&str> = lines.split_whitespace().collect();

        // Handle enum class first
        if words.len() >= 3 && words[0] == "enum" && words[1] == "class" {
            let name_with_extras = words[2];
            let name = name_with_extras
                .split(['(', ':', '<', '{'])
                .next()
                .unwrap_or(name_with_extras)
                .trim();
            return Some(name.to_string());
        }

        // Handle data class
        if words.len() >= 3 && words[0] == "data" && words[1] == "class" {
            let name_with_extras = words[2];
            let name = name_with_extras
                .split(['(', ':', '<'])
                .next()
                .unwrap_or(name_with_extras);
            return Some(name.to_string());
        }

        // Handle regular class/interface/object
        for i in 0..words.len() {
            if matches!(words[i], "class" | "interface" | "object") && i + 1 < words.len() {
                let name_with_extras = words[i + 1];
                // Remove everything after the first '(' or ':' or '<'
                let name = name_with_extras
                    .split(['(', ':', '<'])
                    .next()
                    .unwrap_or(name_with_extras);
                return Some(name.to_string());
            }
        }

        None
    }

    /// Convert byte position to line number
    pub(crate) fn byte_pos_to_line(byte_pos: usize, content_lines: &[&str]) -> usize {
        let mut current_pos = 0;
        for (line_idx, line) in content_lines.iter().enumerate() {
            if current_pos + line.len() >= byte_pos {
                return line_idx + 1; // 1-based line numbers
            }
            current_pos += line.len() + 1; // +1 for newline character
        }
        content_lines.len() // Return last line if position is beyond content
    }
}

#[cfg(feature = "kotlin-ast")]
#[async_trait]
impl AstStrategy for KotlinAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Delegate to the new implementation in ast::languages::kotlin_strategy
        use crate::services::ast::languages::kotlin_strategy::KotlinStrategy;
        use crate::services::ast::AstStrategy as UnifiedAstStrategy;

        let kotlin_strategy = KotlinStrategy;
        UnifiedAstStrategy::analyze(&kotlin_strategy, path, classifier).await
    }

    fn supports_extension(&self, ext: &str) -> bool {
        debug_assert!(!ext.is_empty(), "ext must not be empty");
        matches!(ext, "kt" | "kts")
    }
}
