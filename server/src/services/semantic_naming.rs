use rustc_hash::FxHashMap;
use std::path::Path;

use crate::models::dag::NodeInfo;

/// Manages semantic naming for graph nodes to ensure deterministic and meaningful names
#[derive(Debug, Clone)]
pub struct SemanticNamer {
    /// Language-specific separator patterns
    patterns: FxHashMap<String, &'static str>,
}

impl SemanticNamer {
    /// Create a new SemanticNamer with default language patterns
    pub fn new() -> Self {
        let mut patterns = FxHashMap::default();
        patterns.insert("rust".to_string(), "::");
        patterns.insert("python".to_string(), ".");
        patterns.insert("typescript".to_string(), ".");
        patterns.insert("javascript".to_string(), ".");
        patterns.insert("go".to_string(), "/");
        patterns.insert("java".to_string(), ".");

        Self { patterns }
    }

    /// Get a semantic name for a node based on its available fields
    pub fn get_semantic_name(&self, id: &str, node: &NodeInfo) -> String {
        // Priority 1: Use label if not empty and meaningful
        if !node.label.is_empty() && node.label != id {
            return node.label.clone();
        }

        // Priority 2: Convert file_path to module notation
        if !node.file_path.is_empty() {
            let ext = std::path::Path::new(&node.file_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let language = Self::detect_language(ext);
            return self.path_to_module(&node.file_path, language);
        }

        // Priority 3: Clean the raw ID
        self.clean_id(id)
    }

    /// Convert a file path to module notation based on language
    fn path_to_module(&self, path: &str, language: &str) -> String {
        let separator = self.patterns.get(language).copied().unwrap_or("::");

        // Strip common prefixes using string manipulation for simplicity
        let clean_path = if let Some(stripped) = path.strip_prefix("src/") {
            stripped
        } else if let Some(stripped) = path.strip_prefix("lib/") {
            stripped
        } else if let Some(stripped) = path.strip_prefix("app/") {
            stripped
        } else {
            path
        };

        let path_obj = Path::new(clean_path);

        // Remove file extension
        let without_ext = path_obj.with_extension("");

        // Convert path separators to language-specific module separators
        let module_path = without_ext
            .to_string_lossy()
            .replace(['/', '\\'], separator);

        // Special handling for index/mod files
        if module_path.ends_with(&format!("{separator}index")) {
            module_path
                .trim_end_matches(&format!("{separator}index"))
                .to_string()
        } else if module_path.ends_with(&format!("{separator}mod")) {
            module_path
                .trim_end_matches(&format!("{separator}mod"))
                .to_string()
        } else {
            module_path
        }
    }

    /// Clean a raw ID to make it more readable
    fn clean_id(&self, id: &str) -> String {
        // Remove common prefixes
        let cleaned = id
            .trim_start_matches("node_")
            .trim_start_matches("module_")
            .trim_start_matches("file_");

        // Replace underscores with dots for better readability
        cleaned.replace('_', ".")
    }

    /// Get the language from a file extension
    pub fn detect_language(extension: &str) -> &'static str {
        match extension {
            "rs" => "rust",
            "py" => "python",
            "ts" | "tsx" => "typescript",
            "js" | "jsx" => "javascript",
            "go" => "go",
            "java" => "java",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
            _ => "unknown",
        }
    }
}

impl Default for SemanticNamer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_module_rust() {
        let namer = SemanticNamer::new();
        assert_eq!(
            namer.path_to_module("src/services/ast_rust.rs", "rust"),
            "services::ast_rust"
        );
        assert_eq!(namer.path_to_module("src/models/mod.rs", "rust"), "models");
    }

    #[test]
    fn test_path_to_module_python() {
        let namer = SemanticNamer::new();
        assert_eq!(
            namer.path_to_module("lib/auth/login.py", "python"),
            "auth.login"
        );
        assert_eq!(
            namer.path_to_module("app/models/__init__.py", "python"),
            "models.__init__"
        );
    }

    #[test]
    fn test_get_semantic_name_priority() {
        let namer = SemanticNamer::new();

        let node = NodeInfo {
            id: "node_123".to_string(),
            label: "MyModule".to_string(),
            node_type: crate::models::dag::NodeType::Module,
            file_path: "src/my_module.rs".to_string(),
            line_number: 1,
            complexity: 5,
            metadata: FxHashMap::default(),
        };

        assert_eq!(namer.get_semantic_name("node_123", &node), "MyModule");
    }

    #[test]
    fn test_clean_id() {
        let namer = SemanticNamer::new();
        assert_eq!(namer.clean_id("node_123"), "123");
        assert_eq!(namer.clean_id("module_foo_bar"), "foo.bar");
        assert_eq!(namer.clean_id("file_test_module"), "test.module");
    }
}
