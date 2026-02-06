#![cfg_attr(coverage_nightly, coverage(off))]
//! Additional language strategies (Kotlin, Makefile, etc.)

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

use super::LanguageStrategy;
use crate::ast::core::{AstDag, Language, UnifiedAstNode};

/// Placeholder strategy for languages not yet fully implemented
pub struct PlaceholderStrategy {
    language: Language,
    extensions: Vec<&'static str>,
}

impl PlaceholderStrategy {
    #[must_use]
    pub fn kotlin() -> Self {
        Self {
            language: Language::Kotlin,
            extensions: vec!["kt", "kts"],
        }
    }

    #[must_use]
    pub fn makefile() -> Self {
        Self {
            language: Language::Makefile,
            extensions: vec!["mk", "makefile", "Makefile", "GNUmakefile"],
        }
    }

    #[must_use]
    pub fn shell() -> Self {
        Self {
            language: Language::Shell,
            extensions: vec!["sh", "bash", "zsh", "fish"],
        }
    }
}

#[async_trait]
impl LanguageStrategy for PlaceholderStrategy {
    fn language(&self) -> Language {
        self.language
    }

    fn can_parse(&self, path: &Path) -> bool {
        // Check file extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.extensions.contains(&ext) {
                return true;
            }
        }

        // Check filename for Makefile
        if self.language == Language::Makefile {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                return self.extensions.contains(&name);
            }
        }

        false
    }

    async fn parse_file(&self, _path: &Path, _content: &str) -> Result<AstDag> {
        // Return a basic AST with minimal structure
        Ok(AstDag::new())
    }

    fn extract_imports(&self, _ast: &AstDag) -> Vec<String> {
        Vec::new()
    }

    fn extract_functions(&self, _ast: &AstDag) -> Vec<UnifiedAstNode> {
        Vec::new()
    }

    fn extract_types(&self, _ast: &AstDag) -> Vec<UnifiedAstNode> {
        Vec::new()
    }

    fn calculate_complexity(&self, _ast: &AstDag) -> (u32, u32) {
        (1, 0) // Base complexity
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // ==================== PlaceholderStrategy::kotlin() Tests ====================

    #[test]
    fn test_kotlin_strategy_language() {
        let strategy = PlaceholderStrategy::kotlin();
        assert_eq!(strategy.language(), Language::Kotlin);
    }

    #[test]
    fn test_kotlin_can_parse_kt_file() {
        let strategy = PlaceholderStrategy::kotlin();
        assert!(strategy.can_parse(Path::new("test.kt")));
        assert!(strategy.can_parse(Path::new("/path/to/Main.kt")));
    }

    #[test]
    fn test_kotlin_can_parse_kts_file() {
        let strategy = PlaceholderStrategy::kotlin();
        assert!(strategy.can_parse(Path::new("build.gradle.kts")));
        assert!(strategy.can_parse(Path::new("script.kts")));
    }

    #[test]
    fn test_kotlin_cannot_parse_other_files() {
        let strategy = PlaceholderStrategy::kotlin();
        assert!(!strategy.can_parse(Path::new("test.java")));
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("Makefile")));
    }

    // ==================== PlaceholderStrategy::makefile() Tests ====================

    #[test]
    fn test_makefile_strategy_language() {
        let strategy = PlaceholderStrategy::makefile();
        assert_eq!(strategy.language(), Language::Makefile);
    }

    #[test]
    fn test_makefile_can_parse_mk_extension() {
        let strategy = PlaceholderStrategy::makefile();
        assert!(strategy.can_parse(Path::new("rules.mk")));
        assert!(strategy.can_parse(Path::new("build.mk")));
    }

    #[test]
    fn test_makefile_can_parse_makefile_filename() {
        let strategy = PlaceholderStrategy::makefile();
        assert!(strategy.can_parse(Path::new("Makefile")));
        assert!(strategy.can_parse(Path::new("makefile")));
        assert!(strategy.can_parse(Path::new("GNUmakefile")));
    }

    #[test]
    fn test_makefile_can_parse_full_path() {
        let strategy = PlaceholderStrategy::makefile();
        assert!(strategy.can_parse(Path::new("/home/user/project/Makefile")));
        assert!(strategy.can_parse(Path::new("./Makefile")));
    }

    #[test]
    fn test_makefile_cannot_parse_other_files() {
        let strategy = PlaceholderStrategy::makefile();
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("CMakeLists.txt")));
        assert!(!strategy.can_parse(Path::new("test.py")));
    }

    // ==================== PlaceholderStrategy::shell() Tests ====================

    #[test]
    fn test_shell_strategy_language() {
        let strategy = PlaceholderStrategy::shell();
        assert_eq!(strategy.language(), Language::Shell);
    }

    #[test]
    fn test_shell_can_parse_sh_file() {
        let strategy = PlaceholderStrategy::shell();
        assert!(strategy.can_parse(Path::new("script.sh")));
        assert!(strategy.can_parse(Path::new("/usr/bin/test.sh")));
    }

    #[test]
    fn test_shell_can_parse_bash_file() {
        let strategy = PlaceholderStrategy::shell();
        assert!(strategy.can_parse(Path::new("script.bash")));
        assert!(strategy.can_parse(Path::new("install.bash")));
    }

    #[test]
    fn test_shell_can_parse_zsh_file() {
        let strategy = PlaceholderStrategy::shell();
        assert!(strategy.can_parse(Path::new("script.zsh")));
        assert!(strategy.can_parse(Path::new(".zshrc.zsh")));
    }

    #[test]
    fn test_shell_can_parse_fish_file() {
        let strategy = PlaceholderStrategy::shell();
        assert!(strategy.can_parse(Path::new("config.fish")));
        assert!(strategy.can_parse(Path::new("functions.fish")));
    }

    #[test]
    fn test_shell_cannot_parse_other_files() {
        let strategy = PlaceholderStrategy::shell();
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("Makefile")));
        assert!(!strategy.can_parse(Path::new(".bashrc"))); // No extension
    }

    // ==================== Common LanguageStrategy trait tests ====================

    #[tokio::test]
    async fn test_kotlin_parse_file_returns_empty_dag() {
        let strategy = PlaceholderStrategy::kotlin();
        let path = PathBuf::from("test.kt");
        let code = "fun main() { println(\"Hello\") }";
        let result = strategy.parse_file(&path, code).await;

        assert!(result.is_ok());
        let dag = result.unwrap();
        assert!(dag.nodes.is_empty());
    }

    #[tokio::test]
    async fn test_makefile_parse_file_returns_empty_dag() {
        let strategy = PlaceholderStrategy::makefile();
        let path = PathBuf::from("Makefile");
        let code = "all:\n\techo 'hello'";
        let result = strategy.parse_file(&path, code).await;

        assert!(result.is_ok());
        let dag = result.unwrap();
        assert!(dag.nodes.is_empty());
    }

    #[tokio::test]
    async fn test_shell_parse_file_returns_empty_dag() {
        let strategy = PlaceholderStrategy::shell();
        let path = PathBuf::from("test.sh");
        let code = "#!/bin/bash\necho 'hello'";
        let result = strategy.parse_file(&path, code).await;

        assert!(result.is_ok());
        let dag = result.unwrap();
        assert!(dag.nodes.is_empty());
    }

    #[test]
    fn test_extract_imports_returns_empty() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        let makefile_strategy = PlaceholderStrategy::makefile();
        let shell_strategy = PlaceholderStrategy::shell();

        let dag = AstDag::new();

        assert!(kotlin_strategy.extract_imports(&dag).is_empty());
        assert!(makefile_strategy.extract_imports(&dag).is_empty());
        assert!(shell_strategy.extract_imports(&dag).is_empty());
    }

    #[test]
    fn test_extract_functions_returns_empty() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        let makefile_strategy = PlaceholderStrategy::makefile();
        let shell_strategy = PlaceholderStrategy::shell();

        let dag = AstDag::new();

        assert!(kotlin_strategy.extract_functions(&dag).is_empty());
        assert!(makefile_strategy.extract_functions(&dag).is_empty());
        assert!(shell_strategy.extract_functions(&dag).is_empty());
    }

    #[test]
    fn test_extract_types_returns_empty() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        let makefile_strategy = PlaceholderStrategy::makefile();
        let shell_strategy = PlaceholderStrategy::shell();

        let dag = AstDag::new();

        assert!(kotlin_strategy.extract_types(&dag).is_empty());
        assert!(makefile_strategy.extract_types(&dag).is_empty());
        assert!(shell_strategy.extract_types(&dag).is_empty());
    }

    #[test]
    fn test_calculate_complexity_returns_base() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        let makefile_strategy = PlaceholderStrategy::makefile();
        let shell_strategy = PlaceholderStrategy::shell();

        let dag = AstDag::new();

        let (cyclomatic, cognitive) = kotlin_strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);

        let (cyclomatic, cognitive) = makefile_strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);

        let (cyclomatic, cognitive) = shell_strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_can_parse_empty_path() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        let makefile_strategy = PlaceholderStrategy::makefile();
        let shell_strategy = PlaceholderStrategy::shell();

        assert!(!kotlin_strategy.can_parse(Path::new("")));
        // Makefile might match on filename, so empty should not match
        assert!(!makefile_strategy.can_parse(Path::new("")));
        assert!(!shell_strategy.can_parse(Path::new("")));
    }

    #[test]
    fn test_can_parse_no_extension() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        let shell_strategy = PlaceholderStrategy::shell();

        assert!(!kotlin_strategy.can_parse(Path::new("README")));
        assert!(!shell_strategy.can_parse(Path::new("README")));
    }

    #[test]
    fn test_extensions_stored_correctly() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        assert!(kotlin_strategy.extensions.contains(&"kt"));
        assert!(kotlin_strategy.extensions.contains(&"kts"));
        assert_eq!(kotlin_strategy.extensions.len(), 2);

        let makefile_strategy = PlaceholderStrategy::makefile();
        assert!(makefile_strategy.extensions.contains(&"mk"));
        assert!(makefile_strategy.extensions.contains(&"Makefile"));
        assert!(makefile_strategy.extensions.contains(&"makefile"));
        assert!(makefile_strategy.extensions.contains(&"GNUmakefile"));

        let shell_strategy = PlaceholderStrategy::shell();
        assert!(shell_strategy.extensions.contains(&"sh"));
        assert!(shell_strategy.extensions.contains(&"bash"));
        assert!(shell_strategy.extensions.contains(&"zsh"));
        assert!(shell_strategy.extensions.contains(&"fish"));
        assert_eq!(shell_strategy.extensions.len(), 4);
    }

    #[test]
    fn test_language_stored_correctly() {
        let kotlin_strategy = PlaceholderStrategy::kotlin();
        assert_eq!(kotlin_strategy.language, Language::Kotlin);

        let makefile_strategy = PlaceholderStrategy::makefile();
        assert_eq!(makefile_strategy.language, Language::Makefile);

        let shell_strategy = PlaceholderStrategy::shell();
        assert_eq!(shell_strategy.language, Language::Shell);
    }
}
