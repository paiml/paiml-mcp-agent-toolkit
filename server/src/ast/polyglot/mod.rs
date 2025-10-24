//! Polyglot AST module for cross-language analysis
//! 
//! This module provides a unified representation of AST nodes across different
//! programming languages, enabling cross-language analysis and dependency tracking.
//! The polyglot AST framework is designed to map language-specific AST nodes into
//! a common representation that preserves semantic meaning while abstracting away
//! language-specific details.
//!
//! # Architecture
//!
//! The polyglot AST system consists of three main components:
//!
//! 1. **UnifiedNode**: A language-agnostic representation of code elements
//! 2. **LanguageMapper**: Translates language-specific ASTs to UnifiedNodes
//! 3. **CrossLanguageDependencies**: Tracks relationships between nodes in different languages
//!
//! # Example
//!
//! ```rust,no_run
//! use crate::ast::polyglot::{UnifiedNode, LanguageMapper, JavaMapper, TypeScriptMapper};
//! use std::path::Path;
//!
//! // Create language mappers
//! let java_mapper = JavaMapper::new();
//! let ts_mapper = TypeScriptMapper::new();
//!
//! // Analyze files from different languages
//! let java_file = Path::new("src/main/java/com/example/Model.java");
//! let ts_file = Path::new("src/frontend/models/Model.ts");
//!
//! // Map to unified representation
//! let java_nodes = java_mapper.map_file(java_file).await?;
//! let ts_nodes = ts_mapper.map_file(ts_file).await?;
//!
//! // Find relationships between nodes
//! let dependencies = CrossLanguageDependencies::detect(&java_nodes, &ts_nodes);
//! ```

use crate::ast::core::{AstItem, ItemKind};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub mod unified_node;
pub mod language_mapper;
pub mod cross_language_dependencies;

pub use unified_node::UnifiedNode;
pub use language_mapper::LanguageMapper;
pub use cross_language_dependencies::CrossLanguageDependencies;

/// Common language identifiers used throughout the polyglot AST system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Java,
    Kotlin,
    Scala,
    TypeScript,
    JavaScript,
    Python,
    Rust,
    Go,
    Cpp,
    CSharp,
    Ruby,
    Swift,
    Php,
    Other(u32), // Numeric identifier for other languages
}

impl Language {
    /// Returns the name of the language as a string
    pub fn name(&self) -> &'static str {
        match self {
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Rust => "Rust",
            Language::Go => "Go",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Php => "PHP",
            Language::Other(_) => "Other",
        }
    }
    
    /// Returns the language from a file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "java" => Some(Language::Java),
            "kt" | "kts" => Some(Language::Kotlin),
            "scala" | "sc" => Some(Language::Scala),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "rs" => Some(Language::Rust),
            "go" => Some(Language::Go),
            "cpp" | "cc" | "cxx" | "c++" | "h" | "hpp" => Some(Language::Cpp),
            "cs" => Some(Language::CSharp),
            "rb" => Some(Language::Ruby),
            "swift" => Some(Language::Swift),
            "php" => Some(Language::Php),
            _ => None,
        }
    }
    
    /// Returns the language from a file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }
    
    /// Returns common file extensions for this language
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            Language::Java => vec!["java"],
            Language::Kotlin => vec!["kt", "kts"],
            Language::Scala => vec!["scala", "sc"],
            Language::TypeScript => vec!["ts", "tsx"],
            Language::JavaScript => vec!["js", "jsx"],
            Language::Python => vec!["py"],
            Language::Rust => vec!["rs"],
            Language::Go => vec!["go"],
            Language::Cpp => vec!["cpp", "cc", "cxx", "c++", "h", "hpp"],
            Language::CSharp => vec!["cs"],
            Language::Ruby => vec!["rb"],
            Language::Swift => vec!["swift"],
            Language::Php => vec!["php"],
            Language::Other(_) => vec![],
        }
    }
}

/// Common node types across languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    // Declarations
    Package,
    Import,
    Module,
    Namespace,
    
    // Types
    Class,
    Interface,
    Trait,
    Enum,
    Struct,
    Union,
    
    // Type variations
    Record,      // Java record, Kotlin data class, TypeScript interface
    CaseClass,   // Scala case class
    AbstractType, // Abstract class/interface/trait
    
    // Functions & methods
    Method,
    Function,
    Constructor,
    Lambda,
    Closure,
    
    // Variables
    Field,
    Property,
    LocalVariable,
    Parameter,
    
    // Other elements
    Annotation,
    Decorator,
    Comment,
    
    // Relationships
    Inherits,
    Implements,
    Uses,
    
    // For any language-specific constructs
    LanguageSpecific(u32), // Numeric identifier for language-specific nodes
    Unknown,
}

impl NodeKind {
    /// Convert from AstItem's ItemKind
    pub fn from_ast_item_kind(kind: &ItemKind) -> Self {
        match kind.as_str() {
            "package" => NodeKind::Package,
            "import" | "require" => NodeKind::Import,
            "module" | "namespace" => NodeKind::Module,
            
            "class" => NodeKind::Class,
            "interface" => NodeKind::Interface,
            "trait" => NodeKind::Trait,
            "enum" => NodeKind::Enum,
            "struct" => NodeKind::Struct,
            "union" => NodeKind::Union,
            
            "record" | "data class" => NodeKind::Record,
            "case class" => NodeKind::CaseClass,
            "abstract class" | "abstract interface" => NodeKind::AbstractType,
            
            "method" => NodeKind::Method,
            "function" => NodeKind::Function,
            "constructor" => NodeKind::Constructor,
            "lambda" | "arrow function" => NodeKind::Lambda,
            "closure" => NodeKind::Closure,
            
            "field" | "member" => NodeKind::Field,
            "property" => NodeKind::Property,
            "variable" | "var" | "let" | "const" => NodeKind::LocalVariable,
            "parameter" | "param" => NodeKind::Parameter,
            
            "annotation" | "attribute" => NodeKind::Annotation,
            "decorator" => NodeKind::Decorator,
            "comment" | "doc comment" => NodeKind::Comment,
            
            _ => NodeKind::Unknown,
        }
    }
    
    /// Returns a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Package => "package",
            NodeKind::Import => "import",
            NodeKind::Module => "module",
            NodeKind::Namespace => "namespace",
            
            NodeKind::Class => "class",
            NodeKind::Interface => "interface",
            NodeKind::Trait => "trait",
            NodeKind::Enum => "enum",
            NodeKind::Struct => "struct",
            NodeKind::Union => "union",
            
            NodeKind::Record => "record",
            NodeKind::CaseClass => "caseClass",
            NodeKind::AbstractType => "abstractType",
            
            NodeKind::Method => "method",
            NodeKind::Function => "function",
            NodeKind::Constructor => "constructor",
            NodeKind::Lambda => "lambda",
            NodeKind::Closure => "closure",
            
            NodeKind::Field => "field",
            NodeKind::Property => "property",
            NodeKind::LocalVariable => "localVariable",
            NodeKind::Parameter => "parameter",
            
            NodeKind::Annotation => "annotation",
            NodeKind::Decorator => "decorator",
            NodeKind::Comment => "comment",
            
            NodeKind::Inherits => "inherits",
            NodeKind::Implements => "implements",
            NodeKind::Uses => "uses",
            
            NodeKind::LanguageSpecific(_) => "languageSpecific",
            NodeKind::Unknown => "unknown",
        }
    }
}

/// Configuration for polyglot AST analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotConfig {
    /// List of languages to include in analysis
    pub languages: Vec<Language>,
    
    /// Whether to detect cross-language relationships
    pub detect_relationships: bool,
    
    /// Maximum depth for relationship analysis (inheritance, implementation, etc.)
    pub relationship_depth: usize,
    
    /// Whether to include language-specific details
    pub include_language_specific: bool,
}

impl Default for PolyglotConfig {
    fn default() -> Self {
        Self {
            languages: vec![
                Language::Java,
                Language::Kotlin,
                Language::Scala,
                Language::TypeScript,
                Language::JavaScript,
            ],
            detect_relationships: true,
            relationship_depth: 3,
            include_language_specific: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("java"), Some(Language::Java));
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("scala"), Some(Language::Scala));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("unknown"), None);
    }
    
    #[test]
    fn test_language_from_path() {
        let java_path = Path::new("/path/to/MyClass.java");
        let scala_path = Path::new("/path/to/MyClass.scala");
        let unknown_path = Path::new("/path/to/file.unknown");
        
        assert_eq!(Language::from_path(java_path), Some(Language::Java));
        assert_eq!(Language::from_path(scala_path), Some(Language::Scala));
        assert_eq!(Language::from_path(unknown_path), None);
    }
    
    #[test]
    fn test_node_kind_from_ast_item_kind() {
        assert_eq!(NodeKind::from_ast_item_kind(&"class".into()), NodeKind::Class);
        assert_eq!(NodeKind::from_ast_item_kind(&"method".into()), NodeKind::Method);
        assert_eq!(NodeKind::from_ast_item_kind(&"unknown".into()), NodeKind::Unknown);
    }
    
    #[test]
    fn test_polyglot_config_default() {
        let config = PolyglotConfig::default();
        
        assert!(config.languages.contains(&Language::Java));
        assert!(config.languages.contains(&Language::Scala));
        assert!(config.languages.contains(&Language::TypeScript));
        assert!(config.detect_relationships);
        assert_eq!(config.relationship_depth, 3);
    }
}