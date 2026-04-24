//! Language-aware analysis dispatcher per SPECIFICATION.md Section 6.2
//!
//! This module provides language-specific analysis capabilities by integrating
//! the language registry with existing analysis services.
//!
//! Split into submodules for file health compliance:
//! - language_analyzer_core.rs: Core analysis, comment detection, metadata
//! - language_analyzer_analyses.rs: Individual analysis implementations

#![cfg_attr(coverage_nightly, coverage(off))]

use super::language_registry::{Language, LanguageRegistry};
use super::service_base::ServiceMetrics;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Language-specific analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysisRequest {
    pub path: PathBuf,
    pub language: Option<Language>,
    pub analysis_types: Vec<AnalysisType>,
    pub options: AnalysisOptions,
}

/// Available analysis types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    Complexity,
    Satd,
    DeadCode,
    Security,
    Style,
    Documentation,
    Dependencies,
    Metrics,
}

/// Analysis options for language-specific analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub complexity_threshold: u32,
    pub include_comments: bool,
    pub include_tests: bool,
    pub parallel_analysis: bool,
    pub output_format: OutputFormat,
}

pub use crate::contracts::OutputFormat;

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            complexity_threshold: 20,
            include_comments: true,
            include_tests: false,
            parallel_analysis: true,
            output_format: OutputFormat::Json,
        }
    }
}

/// Analysis results for a language-specific file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysisResult {
    pub path: PathBuf,
    pub language: Language,
    pub analysis_results: Vec<AnalysisResult>,
    pub metadata: FileMetadata,
    pub processing_time_ms: u64,
}

/// Individual analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub analysis_type: AnalysisType,
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub lines_total: usize,
    pub lines_code: usize,
    pub lines_comment: usize,
    pub lines_blank: usize,
    pub file_size_bytes: u64,
    pub detected_language: Language,
    pub confidence: f64,
}

/// Comment style for different languages
#[derive(Debug, Clone, PartialEq)]
enum CommentStyle {
    CStyle,     // //
    Hash,       // #
    Semicolon,  // ;
    Percent,    // %
    DoubleDash, // --
    Xml,        // <!--
    None,       // No comments
}

/// Language-aware analysis service
pub struct LanguageAnalyzer {
    language_registry: LanguageRegistry,
    metrics: Arc<std::sync::Mutex<ServiceMetrics>>,
}

impl Default for LanguageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageAnalyzer {
    /// Create a new language analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            language_registry: LanguageRegistry::new(),
            metrics: Arc::new(std::sync::Mutex::new(ServiceMetrics::default())),
        }
    }
}

// Core analysis methods: analyze_file, supports_analysis, comment detection
include!("language_analyzer_core.rs");

// Individual analysis implementations: complexity, SATD, security, style, etc.
include!("language_analyzer_analyses.rs");

// Pure-compute helpers in language_analyzer_analyses.rs were 0%-covered on
// broad (239 missed lines). The legacy `language_analyzer_tests.rs` split
// across part1..4.rs has unbalanced braces per file and cannot be wired into
// the module tree as-is. This inline test module covers the key leaf
// helpers (keyword lists, pattern counters) that don't require async setup.
#[cfg(test)]
mod inline_tests {
    use super::*;

    /// get_complexity_keywords returns language-specific control-flow tokens.
    #[test]
    fn test_get_complexity_keywords_returns_nonempty_for_known_languages() {
        let a = LanguageAnalyzer::new();
        for lang in [
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Go,
            Language::Java,
        ] {
            let kws = a.get_complexity_keywords(lang);
            assert!(
                !kws.is_empty(),
                "language {lang:?} must have complexity keywords"
            );
            for kw in &kws {
                assert!(!kw.is_empty(), "no empty keyword for {lang:?}");
            }
        }
    }

    /// calculate_keyword_complexity counts occurrences of any of the given
    /// keywords plus 1 base complexity (mirrors cyclomatic McCabe).
    #[test]
    fn test_calculate_keyword_complexity_counts_matches_plus_base() {
        let a = LanguageAnalyzer::new();
        let content = "if a { for b in c { while d { if e {} } } }";
        let count = a.calculate_keyword_complexity(content, &["if", "for", "while"]);
        // 2*"if" + 1*"for" + 1*"while" = 4, + 1 base = 5
        assert_eq!(count, 5);
    }

    #[test]
    fn test_calculate_keyword_complexity_returns_base_on_no_match() {
        let a = LanguageAnalyzer::new();
        let count = a.calculate_keyword_complexity("plain text", &["if", "for"]);
        assert_eq!(count, 1, "base complexity is 1 when no keyword matches");
    }

    #[test]
    fn test_calculate_keyword_complexity_empty_keyword_list_returns_base() {
        let a = LanguageAnalyzer::new();
        let count = a.calculate_keyword_complexity("if a { for b {} }", &[]);
        assert_eq!(count, 1);
    }

    /// get_security_patterns returns language-specific known-bad patterns.
    #[test]
    fn test_get_security_patterns_nonempty_for_common_langs() {
        let a = LanguageAnalyzer::new();
        for lang in [Language::Python, Language::JavaScript, Language::Rust] {
            let pats = a.get_security_patterns(lang);
            // Not every language has a pattern list but these common ones do.
            assert!(!pats.is_empty(), "expected security patterns for {lang:?}");
        }
    }

    /// find_security_issues returns a non-empty vec when the content contains
    /// the pattern, empty otherwise. Drives both arms.
    #[test]
    fn test_find_security_issues_match_vs_miss() {
        let a = LanguageAnalyzer::new();
        let patterns = ["eval("];
        let hits = a.find_security_issues("x = eval(expr)", &patterns);
        assert!(!hits.is_empty(), "must detect eval(");

        let misses = a.find_security_issues("x = 1 + 2", &patterns);
        assert!(misses.is_empty(), "clean code → no hits");
    }

    /// get_import_patterns returns language-specific import-statement prefixes.
    #[test]
    fn test_get_import_patterns_nonempty_for_common_langs() {
        let a = LanguageAnalyzer::new();
        for lang in [
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::Go,
        ] {
            let pats = a.get_import_patterns(lang);
            assert!(!pats.is_empty(), "{lang:?} should have import patterns");
        }
    }

    /// find_imports collects lines matching any of the import-prefix patterns.
    #[test]
    fn test_find_imports_match_vs_miss() {
        let a = LanguageAnalyzer::new();
        let patterns = ["use ", "import "];
        let src = "use foo::bar;\nlet x = 1;\nimport baz;\n// use commented";
        let hits = a.find_imports(src, &patterns);
        // Two matches: `use foo::bar;` and `import baz;`
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_find_imports_empty_on_no_match() {
        let a = LanguageAnalyzer::new();
        let hits = a.find_imports("fn main() {}\n", &["use ", "import "]);
        assert!(hits.is_empty());
    }

    /// create_unsupported_analysis_result returns a result flagged as
    /// unsuccessful with an error field that names the language + analysis.
    #[test]
    fn test_create_unsupported_analysis_result_shape() {
        let a = LanguageAnalyzer::new();
        let result =
            a.create_unsupported_analysis_result(AnalysisType::Security, Language::Markdown);
        assert!(!result.success, "unsupported → success=false");
        assert!(
            result.error.is_some(),
            "unsupported → must carry an error message"
        );
    }
}
