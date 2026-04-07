#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared types, enums, and traits for language analysis.

use crate::services::complexity::ComplexityMetrics;
use std::path::Path;

/// Supported programming languages for complexity analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    C,
    CPP,
    Go,
    Bash,
    Java,
    Kotlin,
    Ruby,
    PHP,
    Swift,
    CSharp,
    Lua,
    Sql,
    Scala,
    Yaml,
    Markdown,
    Lean,
    Unknown,
}

impl Language {
    /// Detect language from file extension
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("js" | "jsx") => Language::JavaScript,
            Some("ts" | "tsx") => Language::TypeScript,
            Some("py") => Language::Python,
            Some("c" | "h") => Language::C,
            Some("cpp" | "cc" | "cxx" | "hpp" | "hxx" | "h++" | "c++" | "cu" | "cuh") => {
                Language::CPP
            }
            Some("go") => Language::Go,
            Some("sh" | "bash") => Language::Bash,
            Some("java") => Language::Java,
            Some("kt" | "kts") => Language::Kotlin,
            Some("rb") => Language::Ruby,
            Some("php") => Language::PHP,
            Some("swift") => Language::Swift,
            Some("cs") => Language::CSharp,
            Some("lua") => Language::Lua,
            Some("sql" | "ddl" | "dml") => Language::Sql,
            Some("scala" | "sc") => Language::Scala,
            Some("yaml" | "yml") => Language::Yaml,
            Some("md" | "mdx" | "markdown") => Language::Markdown,
            Some("lean") => Language::Lean,
            _ => Language::Unknown,
        }
    }
}

/// Language-specific analyzer trait
pub trait LanguageAnalyzer {
    /// Extract functions from source code
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo>;

    /// Estimate complexity for a function
    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics;
}

/// Information about a detected function
pub struct FunctionInfo {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
}
