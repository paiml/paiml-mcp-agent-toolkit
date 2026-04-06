// Core analysis methods for LanguageAnalyzer
// Included from language_analyzer.rs - NO imports or inner attributes here

impl LanguageAnalyzer {
    /// Analyze a file with automatic language detection
    pub async fn analyze_file(
        &self,
        path: &Path,
        analysis_types: Vec<AnalysisType>,
    ) -> Result<LanguageAnalysisResult> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
    #[must_use]
    pub fn supported_languages(&self) -> &[Language] {
        self.language_registry.supported_languages()
    }

    /// Check if language supports specific analysis type
    #[must_use]
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
        debug_assert!(!content.is_empty(), "content must not be empty");
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
        debug_assert!(!line.is_empty(), "line must not be empty");
        match self.get_comment_style(language) {
            CommentStyle::CStyle => self.is_c_style_comment(line),
            CommentStyle::Hash => line.starts_with('#'),
            CommentStyle::Semicolon => line.starts_with(';'),
            CommentStyle::Percent => line.starts_with('%'),
            CommentStyle::DoubleDash => line.starts_with("--"),
            CommentStyle::Xml => line.starts_with("<!--"),
            CommentStyle::None => false,
        }
    }

    /// Get the comment style for a language
    fn get_comment_style(&self, language: Language) -> CommentStyle {
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
            | Language::Groovy => CommentStyle::CStyle,

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
            | Language::Makefile => CommentStyle::Hash,

            // Other comment styles
            Language::Clojure => CommentStyle::Semicolon,
            Language::Erlang | Language::Matlab => CommentStyle::Percent,
            Language::SQL | Language::Haskell | Language::Lean => CommentStyle::DoubleDash,
            Language::XML => CommentStyle::Xml,

            // No comment style
            _ => CommentStyle::None,
        }
    }

    /// Check if line is C-style comment
    fn is_c_style_comment(&self, line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        line.starts_with("//") || line.starts_with("/*") || line.starts_with('*')
    }
}
