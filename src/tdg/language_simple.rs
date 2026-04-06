#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    Ruchy,
    Lua,
    Lean,
    Sql,
    Scala,
    Yaml,
    Markdown,
    Unknown,
}

impl Language {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => Language::Rust,
            Some("py") => Language::Python,
            Some("js" | "jsx") => Language::JavaScript,
            Some("ts" | "tsx") => Language::TypeScript,
            Some("go") => Language::Go,
            Some("java") => Language::Java,
            Some("c" | "h") => Language::C,
            Some("cpp" | "cc" | "cxx" | "hpp" | "cu" | "cuh") => Language::Cpp,
            Some("rb") => Language::Ruby,
            Some("swift") => Language::Swift,
            Some("kt" | "kts") => Language::Kotlin,
            Some("ruchy" | "rh") => Language::Ruchy,
            Some("lua") => Language::Lua,
            Some("lean") => Language::Lean,
            Some("sql" | "ddl" | "dml") => Language::Sql,
            Some("scala" | "sc") => Language::Scala,
            Some("yaml" | "yml") => Language::Yaml,
            Some("md" | "mdx" | "markdown") => Language::Markdown,
            _ => Language::Unknown,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            Language::Ruchy => 0.95,
            Language::Lua => 0.90,
            Language::Lean => 0.95,
            Language::Sql => 0.80,
            Language::Scala => 0.85,
            Language::Yaml => 0.75,
            Language::Markdown => 0.70,
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
            Language::Ruchy => write!(f, "Ruchy"),
            Language::Lua => write!(f, "Lua"),
            Language::Lean => write!(f, "Lean"),
            Language::Sql => write!(f, "SQL"),
            Language::Scala => write!(f, "Scala"),
            Language::Yaml => write!(f, "YAML"),
            Language::Markdown => write!(f, "Markdown"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NamingStyle {
    SnakeCase,
    CamelCase,
    PascalCase,
    ScreamingSnakeCase,
    KebabCase,
}

impl NamingStyle {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn matches(&self, name: &str) -> bool {
        match self {
            NamingStyle::SnakeCase => name
                .chars()
                .all(|c| c.is_lowercase() || c == '_' || c.is_numeric()),
            NamingStyle::CamelCase => {
                !name.is_empty()
                    && name.chars().next().expect("internal error").is_lowercase()
                    && !name.contains('_')
                    && !name.contains('-')
            }
            NamingStyle::PascalCase => {
                !name.is_empty()
                    && name.chars().next().expect("internal error").is_uppercase()
                    && !name.contains('_')
                    && !name.contains('-')
            }
            NamingStyle::ScreamingSnakeCase => name
                .chars()
                .all(|c| c.is_uppercase() || c == '_' || c.is_numeric()),
            NamingStyle::KebabCase => name
                .chars()
                .all(|c| c.is_lowercase() || c == '-' || c.is_numeric()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanguageRules {
    pub language: Language,
    pub function_style: NamingStyle,
    pub type_style: NamingStyle,
    pub constant_style: NamingStyle,
    pub variable_style: NamingStyle,
}

// LanguageRules impl methods (for_language, rust_rules, python_rules, etc.)
include!("language_simple_rules.rs");

// Unit tests and property tests
include!("language_simple_tests.rs");
