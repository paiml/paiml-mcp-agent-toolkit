//! Language-aware analysis dispatcher per SPECIFICATION.md Section 6.2
//!
//! This module provides language-specific analysis capabilities by integrating
//! the language registry with existing analysis services.

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

/// Output format options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Json,
    Yaml,
    Plain,
    Markdown,
}

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
    pub fn new() -> Self {
        Self {
            language_registry: LanguageRegistry::new(),
            metrics: Arc::new(std::sync::Mutex::new(ServiceMetrics::default())),
        }
    }

    /// Analyze a file with automatic language detection
    pub async fn analyze_file(
        &self,
        path: &Path,
        analysis_types: Vec<AnalysisType>,
    ) -> Result<LanguageAnalysisResult> {
        let start_time = std::time::Instant::now();

        // Detect language
        let language = self.language_registry.detect_language(path);

        // Read file for analysis
        let content = tokio::fs::read_to_string(path).await?;
        let metadata = self.analyze_file_metadata(&content, language);

        // Perform language-specific analysis
        let analysis_results = self
            .perform_analyses(&content, language, &analysis_types)
            .await?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Update metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_request(start_time.elapsed(), true);
        }

        Ok(LanguageAnalysisResult {
            path: path.to_path_buf(),
            language,
            analysis_results,
            metadata,
            processing_time_ms: processing_time,
        })
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> &[Language] {
        self.language_registry.supported_languages()
    }

    /// Check if language supports specific analysis type
    pub fn supports_analysis(&self, language: Language, analysis_type: &AnalysisType) -> bool {
        match analysis_type {
            AnalysisType::Complexity => language.supports_complexity(),
            AnalysisType::Satd => true, // SATD can be detected in any text file
            AnalysisType::DeadCode => language.has_ast_support(),
            AnalysisType::Security => language.supports_complexity(), // Security analysis needs AST
            AnalysisType::Style => language.has_ast_support(),
            AnalysisType::Documentation => matches!(
                language,
                Language::Markdown | Language::LaTeX | Language::AsciiDoc | Language::Unknown
            ), // Include unknown for potential docs
            AnalysisType::Dependencies => language.has_ast_support(),
            AnalysisType::Metrics => true, // Basic metrics available for all files
        }
    }

    /// Analyze file metadata (lines, size, etc.)
    fn analyze_file_metadata(&self, content: &str, language: Language) -> FileMetadata {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank_lines += 1;
            } else if self.is_comment_line(trimmed, language) {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
        }

        FileMetadata {
            lines_total: total_lines,
            lines_code: code_lines,
            lines_comment: comment_lines,
            lines_blank: blank_lines,
            file_size_bytes: content.len() as u64,
            detected_language: language,
            confidence: 1.0, // For now, assume high confidence
        }
    }

    /// Check if a line is a comment for the given language
    fn is_comment_line(&self, line: &str, language: Language) -> bool {
        match language {
            // C-style comments
            Language::Rust
            | Language::C
            | Language::Cpp
            | Language::Go
            | Language::Java
            | Language::Kotlin
            | Language::JavaScript
            | Language::TypeScript
            | Language::CSharp
            | Language::Swift
            | Language::Dart
            | Language::Scala
            | Language::Groovy => {
                line.starts_with("//") || line.starts_with("/*") || line.starts_with("*")
            }

            // Hash comments
            Language::Python
            | Language::Ruby
            | Language::Bash
            | Language::Zsh
            | Language::Fish
            | Language::Perl
            | Language::R
            | Language::YAML
            | Language::TOML
            | Language::Makefile => line.starts_with('#'),

            // Semicolon comments
            Language::Clojure => line.starts_with(';'),

            // Percent comments
            Language::Erlang | Language::Matlab => line.starts_with('%'),

            // Double dash comments
            Language::SQL | Language::Haskell => line.starts_with("--"),

            // HTML/XML comments
            Language::XML => line.starts_with("<!--"),

            // Other languages
            _ => false,
        }
    }

    /// Perform language-specific analyses
    async fn perform_analyses(
        &self,
        content: &str,
        language: Language,
        analysis_types: &[AnalysisType],
    ) -> Result<Vec<AnalysisResult>> {
        let mut results = Vec::new();

        for analysis_type in analysis_types {
            if !self.supports_analysis(language, analysis_type) {
                results.push(AnalysisResult {
                    analysis_type: analysis_type.clone(),
                    success: false,
                    data: serde_json::json!({"error": "Analysis not supported for this language"}),
                    error: Some(format!(
                        "Analysis {:?} not supported for language {:?}",
                        analysis_type, language
                    )),
                });
                continue;
            }

            let result = match analysis_type {
                AnalysisType::Complexity => self.analyze_complexity(content, language).await,
                AnalysisType::Satd => self.analyze_satd(content, language).await,
                AnalysisType::DeadCode => self.analyze_dead_code(content, language).await,
                AnalysisType::Security => self.analyze_security(content, language).await,
                AnalysisType::Style => self.analyze_style(content, language).await,
                AnalysisType::Documentation => self.analyze_documentation(content, language).await,
                AnalysisType::Dependencies => self.analyze_dependencies(content, language).await,
                AnalysisType::Metrics => self.analyze_metrics(content, language).await,
            };

            results.push(result);
        }

        Ok(results)
    }

    /// Analyze complexity for the given language
    async fn analyze_complexity(&self, content: &str, language: Language) -> AnalysisResult {
        // Simplified complexity analysis - count control flow keywords
        let complexity_keywords = match language {
            Language::Rust | Language::C | Language::Cpp | Language::Go => {
                vec!["if", "else", "for", "while", "match", "switch", "case"]
            }
            Language::Python => vec!["if", "elif", "else", "for", "while", "try", "except"],
            Language::JavaScript | Language::TypeScript => {
                vec![
                    "if", "else", "for", "while", "switch", "case", "try", "catch",
                ]
            }
            Language::Java | Language::Kotlin => {
                vec![
                    "if", "else", "for", "while", "switch", "case", "try", "catch", "when",
                ]
            }
            _ => vec!["if", "else", "for", "while"], // Basic keywords for other languages
        };

        let mut complexity = 1; // Base complexity
        for keyword in complexity_keywords {
            complexity += content.matches(keyword).count();
        }

        AnalysisResult {
            analysis_type: AnalysisType::Complexity,
            success: true,
            data: serde_json::json!({
                "cyclomatic_complexity": complexity,
                "language": language.name(),
                "method": "keyword_counting"
            }),
            error: None,
        }
    }

    /// Analyze SATD (Self-Admitted Technical Debt)
    async fn analyze_satd(&self, content: &str, _language: Language) -> AnalysisResult {
        let satd_keywords = ["TODO", "FIXME", "HACK", "XXX", "BUG", "KLUDGE"];
        let mut satd_items = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for keyword in &satd_keywords {
                if line.to_uppercase().contains(keyword) {
                    satd_items.push(serde_json::json!({
                        "line": line_num + 1,
                        "keyword": keyword,
                        "text": line.trim()
                    }));
                }
            }
        }

        AnalysisResult {
            analysis_type: AnalysisType::Satd,
            success: true,
            data: serde_json::json!({
                "satd_count": satd_items.len(),
                "items": satd_items
            }),
            error: None,
        }
    }

    /// Analyze dead code (simplified)
    async fn analyze_dead_code(&self, _content: &str, language: Language) -> AnalysisResult {
        AnalysisResult {
            analysis_type: AnalysisType::DeadCode,
            success: true,
            data: serde_json::json!({
                "dead_code_detected": false,
                "note": format!("Dead code analysis for {} requires full AST parsing", language.name())
            }),
            error: None,
        }
    }

    /// Analyze security issues (simplified)
    async fn analyze_security(&self, content: &str, language: Language) -> AnalysisResult {
        let security_patterns = match language {
            Language::JavaScript | Language::TypeScript => {
                vec!["eval(", "innerHTML", "document.write"]
            }
            Language::Python => vec!["exec(", "eval(", "os.system"],
            Language::SQL => vec!["DROP", "DELETE", "UPDATE"],
            _ => vec!["password", "secret", "token"],
        };

        let mut issues = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            for pattern in &security_patterns {
                if line.contains(pattern) {
                    issues.push(serde_json::json!({
                        "line": line_num + 1,
                        "pattern": pattern,
                        "severity": "medium"
                    }));
                }
            }
        }

        AnalysisResult {
            analysis_type: AnalysisType::Security,
            success: true,
            data: serde_json::json!({
                "issues_count": issues.len(),
                "issues": issues
            }),
            error: None,
        }
    }

    /// Analyze code style
    async fn analyze_style(&self, content: &str, language: Language) -> AnalysisResult {
        let line_lengths: Vec<usize> = content.lines().map(|line| line.len()).collect();
        let avg_line_length = if line_lengths.is_empty() {
            0.0
        } else {
            line_lengths.iter().sum::<usize>() as f64 / line_lengths.len() as f64
        };
        let max_line_length = line_lengths.iter().max().copied().unwrap_or(0);

        AnalysisResult {
            analysis_type: AnalysisType::Style,
            success: true,
            data: serde_json::json!({
                "average_line_length": avg_line_length,
                "max_line_length": max_line_length,
                "long_lines": line_lengths.iter().filter(|&&len| len > 120).count(),
                "language": language.name()
            }),
            error: None,
        }
    }

    /// Analyze documentation
    async fn analyze_documentation(&self, content: &str, language: Language) -> AnalysisResult {
        let total_lines = content.lines().count();
        let comment_lines = content
            .lines()
            .filter(|line| self.is_comment_line(line.trim(), language))
            .count();
        let doc_ratio = if total_lines > 0 {
            comment_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        AnalysisResult {
            analysis_type: AnalysisType::Documentation,
            success: true,
            data: serde_json::json!({
                "comment_lines": comment_lines,
                "total_lines": total_lines,
                "documentation_ratio": doc_ratio,
                "assessment": if doc_ratio > 0.2 { "good" } else if doc_ratio > 0.1 { "moderate" } else { "low" }
            }),
            error: None,
        }
    }

    /// Analyze dependencies (simplified)
    async fn analyze_dependencies(&self, content: &str, language: Language) -> AnalysisResult {
        let import_patterns = match language {
            Language::Rust => vec!["use ", "extern crate"],
            Language::Python => vec!["import ", "from "],
            Language::JavaScript | Language::TypeScript => vec!["import ", "require("],
            Language::Java | Language::Kotlin => vec!["import "],
            Language::Go => vec!["import "],
            _ => vec!["import", "include", "require"],
        };

        let mut imports = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            for pattern in &import_patterns {
                if line.trim().starts_with(pattern) {
                    imports.push(serde_json::json!({
                        "line": line_num + 1,
                        "import": line.trim()
                    }));
                }
            }
        }

        AnalysisResult {
            analysis_type: AnalysisType::Dependencies,
            success: true,
            data: serde_json::json!({
                "import_count": imports.len(),
                "imports": imports
            }),
            error: None,
        }
    }

    /// Analyze basic metrics
    async fn analyze_metrics(&self, content: &str, language: Language) -> AnalysisResult {
        let lines: Vec<&str> = content.lines().collect();
        let functions = match language {
            Language::Rust => content.matches("fn ").count(),
            Language::Python => content.matches("def ").count(),
            Language::JavaScript | Language::TypeScript => {
                content.matches("function ").count() + content.matches("=> ").count()
            }
            Language::Java | Language::Kotlin => {
                content.matches("public ").count() + content.matches("private ").count()
            }
            _ => 0,
        };

        AnalysisResult {
            analysis_type: AnalysisType::Metrics,
            success: true,
            data: serde_json::json!({
                "total_lines": lines.len(),
                "estimated_functions": functions,
                "file_size_bytes": content.len(),
                "language": language.name()
            }),
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_language_analyzer_basic() {
        let analyzer = LanguageAnalyzer::new();
        assert!(analyzer.supported_languages().len() >= 50);
    }

    #[tokio::test]
    async fn test_analysis_support() {
        let analyzer = LanguageAnalyzer::new();

        assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Complexity));
        assert!(analyzer.supports_analysis(Language::Python, &AnalysisType::Satd));
        assert!(!analyzer.supports_analysis(Language::JSON, &AnalysisType::Complexity));
        assert!(analyzer.supports_analysis(Language::Markdown, &AnalysisType::Documentation));
    }

    #[test]
    fn test_comment_detection() {
        let analyzer = LanguageAnalyzer::new();

        assert!(analyzer.is_comment_line("// This is a comment", Language::Rust));
        assert!(analyzer.is_comment_line("# This is a comment", Language::Python));
        assert!(analyzer.is_comment_line("/* Comment */", Language::Java));
        assert!(!analyzer.is_comment_line("let x = 5;", Language::Rust));
    }
}
