// Individual analysis implementations for LanguageAnalyzer
// Included from language_analyzer.rs - NO imports or inner attributes here

impl LanguageAnalyzer {
    /// Perform language-specific analyses
    async fn perform_analyses(
        &self,
        content: &str,
        language: Language,
        analysis_types: &[AnalysisType],
    ) -> Result<Vec<AnalysisResult>> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut results = Vec::new();

        for analysis_type in analysis_types {
            let result = if self.supports_analysis(language, analysis_type) {
                self.perform_single_analysis(content, language, analysis_type)
                    .await
            } else {
                self.create_unsupported_analysis_result(analysis_type.clone(), language)
            };

            results.push(result);
        }

        Ok(results)
    }

    async fn perform_single_analysis(
        &self,
        content: &str,
        language: Language,
        analysis_type: &AnalysisType,
    ) -> AnalysisResult {
        debug_assert!(!content.is_empty(), "content must not be empty");
        match analysis_type {
            AnalysisType::Complexity => self.analyze_complexity(content, language).await,
            AnalysisType::Satd => self.analyze_satd(content, language).await,
            AnalysisType::DeadCode => self.analyze_dead_code(content, language).await,
            AnalysisType::Security => self.analyze_security(content, language).await,
            AnalysisType::Style => self.analyze_style(content, language).await,
            AnalysisType::Documentation => self.analyze_documentation(content, language).await,
            AnalysisType::Dependencies => self.analyze_dependencies(content, language).await,
            AnalysisType::Metrics => self.analyze_metrics(content, language).await,
        }
    }

    fn create_unsupported_analysis_result(
        &self,
        analysis_type: AnalysisType,
        language: Language,
    ) -> AnalysisResult {
        AnalysisResult {
            analysis_type: analysis_type.clone(),
            success: false,
            data: serde_json::json!({"error": "Analysis not supported for this language"}),
            error: Some(format!(
                "Analysis {analysis_type:?} not supported for language {language:?}"
            )),
        }
    }

    /// Analyze complexity for the given language
    async fn analyze_complexity(&self, content: &str, language: Language) -> AnalysisResult {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let complexity_keywords = self.get_complexity_keywords(language);
        let complexity = self.calculate_keyword_complexity(content, &complexity_keywords);

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

    /// Get complexity keywords for a language
    fn get_complexity_keywords(&self, language: Language) -> Vec<&'static str> {
        match language {
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
        }
    }

    /// Calculate complexity based on keyword counting
    fn calculate_keyword_complexity(&self, content: &str, keywords: &[&str]) -> usize {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut complexity = 1; // Base complexity
        for keyword in keywords {
            complexity += content.matches(keyword).count();
        }
        complexity
    }

    /// Analyze SATD (Self-Admitted Technical Debt)
    async fn analyze_satd(&self, content: &str, _language: Language) -> AnalysisResult {
        debug_assert!(!content.is_empty(), "content must not be empty");
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
        debug_assert!(!_content.is_empty(), "_content must not be empty");
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
        debug_assert!(!content.is_empty(), "content must not be empty");
        let security_patterns = self.get_security_patterns(language);
        let issues = self.find_security_issues(content, &security_patterns);

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

    /// Get security patterns for a language
    fn get_security_patterns(&self, language: Language) -> Vec<&'static str> {
        match language {
            Language::JavaScript | Language::TypeScript => {
                vec!["eval(", "innerHTML", "document.write"]
            }
            Language::Python => vec!["exec(", "eval(", "os.system"],
            Language::SQL => vec!["DROP", "DELETE", "UPDATE"],
            _ => vec!["password", "secret", "token"],
        }
    }

    /// Find security issues in content
    fn find_security_issues(&self, content: &str, patterns: &[&str]) -> Vec<serde_json::Value> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if line.contains(pattern) {
                    issues.push(serde_json::json!({
                        "line": line_num + 1,
                        "pattern": pattern,
                        "severity": "medium"
                    }));
                }
            }
        }

        issues
    }

    /// Analyze code style
    async fn analyze_style(&self, content: &str, language: Language) -> AnalysisResult {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let line_lengths: Vec<usize> = content.lines().map(str::len).collect();
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
        debug_assert!(!content.is_empty(), "content must not be empty");
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
        debug_assert!(!content.is_empty(), "content must not be empty");
        let import_patterns = self.get_import_patterns(language);
        let imports = self.find_imports(content, &import_patterns);

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

    /// Get import patterns for a language
    fn get_import_patterns(&self, language: Language) -> Vec<&'static str> {
        match language {
            Language::Rust => vec!["use ", "extern crate"],
            Language::Python => vec!["import ", "from "],
            Language::JavaScript | Language::TypeScript => vec!["import ", "require("],
            Language::Java | Language::Kotlin => vec!["import "],
            Language::Go => vec!["import "],
            _ => vec!["import", "include", "require"],
        }
    }

    /// Find imports in content
    fn find_imports(&self, content: &str, patterns: &[&str]) -> Vec<serde_json::Value> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut imports = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if line.trim().starts_with(pattern) {
                    imports.push(serde_json::json!({
                        "line": line_num + 1,
                        "import": line.trim()
                    }));
                }
            }
        }

        imports
    }

    /// Analyze basic metrics
    async fn analyze_metrics(&self, content: &str, language: Language) -> AnalysisResult {
        debug_assert!(!content.is_empty(), "content must not be empty");
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
