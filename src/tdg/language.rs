#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Parser, Tree};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Language.
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    C,
    Cpp,
    Ruby,
    Swift,
    Kotlin,
    Lua,
    Unknown,
}

impl Language {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// From extension.
    pub fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => Language::Rust,
            Some("py") => Language::Python,
            Some("js") | Some("jsx") => Language::JavaScript,
            Some("ts") | Some("tsx") => Language::TypeScript,
            Some("go") => Language::Go,
            Some("java") => Language::Java,
            Some("c") | Some("h") => Language::C,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("cu") | Some("cuh") => Language::Cpp,
            Some("rb") => Language::Ruby,
            Some("swift") => Language::Swift,
            Some("kt") | Some("kts") => Language::Kotlin,
            Some("lua") => Language::Lua,
            _ => Language::Unknown,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Confidence.
    pub fn confidence(&self) -> f32 {
        match self {
            Language::Rust => 1.0,
            Language::Python => 0.95,
            Language::JavaScript => 0.90,
            Language::TypeScript => 0.90,
            Language::Go => 0.95,
            Language::Java => 0.85,
            Language::C => 0.80,
            Language::Cpp => 0.75,
            Language::Ruby => 0.85,
            Language::Swift => 0.85,
            Language::Kotlin => 0.85,
            Language::Lua => 0.90,
            Language::Unknown => 0.5,
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Python => write!(f, "Python"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::Go => write!(f, "Go"),
            Language::Java => write!(f, "Java"),
            Language::C => write!(f, "C"),
            Language::Cpp => write!(f, "C++"),
            Language::Ruby => write!(f, "Ruby"),
            Language::Swift => write!(f, "Swift"),
            Language::Kotlin => write!(f, "Kotlin"),
            Language::Lua => write!(f, "Lua"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Adapter trait for language implementations.
pub trait LanguageAdapter: Send + Sync {
    fn detect(&self, path: &Path) -> bool;
    fn parse(&self, source: &str) -> Result<Tree>;
    fn confidence(&self) -> f32;
    fn language(&self) -> Language;
    fn naming_rules(&self) -> LanguageRules;
}

/// Language adapter for rust.
pub struct RustAdapter {
    parser: Parser,
}

/// Language adapter for python.
pub struct PythonAdapter {
    parser: Parser,
}

/// Language adapter for java script.
pub struct JavaScriptAdapter {
    parser: Parser,
}

/// Language adapter for type script.
pub struct TypeScriptAdapter {
    parser: Parser,
}

/// Language adapter for go.
pub struct GoAdapter {
    parser: Parser,
}

/// Language adapter for lua.
pub struct LuaAdapter {
    parser: Parser,
}

/// Registry of language instances.
pub struct LanguageRegistry {
    adapters: HashMap<Language, Box<dyn LanguageAdapter>>,
}

#[derive(Debug, Clone)]
/// Naming style convention for naming.
pub enum NamingStyle {
    SnakeCase,
    CamelCase,
    PascalCase,
    ScreamingSnakeCase,
    KebabCase,
}

#[derive(Debug, Clone)]
/// Language rules.
pub struct LanguageRules {
    pub language: Language,
    pub function_style: NamingStyle,
    pub type_style: NamingStyle,
    pub constant_style: NamingStyle,
    pub variable_style: NamingStyle,
}

// Adapter constructors and LanguageAdapter trait implementations
include!("language_adapters.rs");

// NamingStyle matching, LanguageRules factory methods, and tests
include!("language_naming.rs");
