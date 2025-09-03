//! Additional language strategies (Kotlin, Makefile, etc.)

use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;

use crate::ast::core::{AstDag, Language, UnifiedAstNode, NodeFlags};
use super::LanguageStrategy;

/// Placeholder strategy for languages not yet fully implemented
pub struct PlaceholderStrategy {
    language: Language,
    extensions: Vec<&'static str>,
}

impl PlaceholderStrategy {
    pub fn kotlin() -> Self {
        Self {
            language: Language::Kotlin,
            extensions: vec!["kt", "kts"],
        }
    }
    
    pub fn makefile() -> Self {
        Self {
            language: Language::Makefile,
            extensions: vec!["mk", "makefile", "Makefile", "GNUmakefile"],
        }
    }
    
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