// SATD extraction: constructors, content parsing, comment extraction, and context hashing.

impl Default for SATDDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SATDDetector {
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(false)
    }

    #[must_use]
    pub fn new_strict() -> Self {
        Self::with_classifier(DebtClassifier::new_strict())
    }

    /// Extended mode: detects euphemisms like placeholder, stub, "for now"
    /// See issue #149
    #[must_use]
    pub fn new_extended() -> Self {
        Self::with_classifier(DebtClassifier::new_extended())
    }

    fn with_classifier(debt_classifier: DebtClassifier) -> Self {
        let patterns = debt_classifier.compiled_patterns.clone();
        Self {
            patterns,
            debt_classifier,
        }
    }

    fn with_config(strict_mode: bool) -> Self {
        let debt_classifier = if strict_mode {
            DebtClassifier::new_strict()
        } else {
            DebtClassifier::new()
        };
        let patterns = debt_classifier.compiled_patterns.clone();

        Self {
            patterns,
            debt_classifier,
        }
    }

    /// Extract technical debt from source code content
    pub fn extract_from_content(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut debts = Vec::new();
        let mut test_tracker = TestBlockTracker::new(self.is_rust_file(file_path));

        for (line_num, line) in content.lines().enumerate() {
            test_tracker.update_from_line(line.trim());

            if !test_tracker.is_in_test_block() {
                if let Some(debt) = self.extract_from_line(line, file_path, line_num as u32 + 1)? {
                    debts.push(debt);
                }
            }
        }

        self.sort_debts(&mut debts);
        Ok(debts)
    }

    pub(crate) fn is_rust_file(&self, file_path: &Path) -> bool {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        file_path.extension().and_then(|s| s.to_str()) == Some("rs")
    }

    fn sort_debts(&self, debts: &mut [TechnicalDebt]) {
        debts.sort_by_key(|d| (d.file.clone(), d.line, d.column));
    }

    /// Extract debt from a single line
    fn extract_from_line(
        &self,
        line: &str,
        file_path: &Path,
        line_num: u32,
    ) -> Result<Option<TechnicalDebt>, TemplateError> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Skip lines that are likely test data or pattern definitions
        if self.is_likely_test_data_or_pattern(line, file_path) {
            return Ok(None);
        }

        // Look for comment patterns
        let comment_content = self.extract_comment_content(line)?;

        if let Some(content) = comment_content {
            if let Some((category, severity)) = self.debt_classifier.classify_comment(&content) {
                // Create basic context (could be enhanced with actual AST analysis)
                let context = AstContext {
                    node_type: AstNodeType::Regular,
                    parent_function: "unknown".to_string(),
                    complexity: 1,
                    siblings_count: 0,
                    nesting_depth: 0,
                    surrounding_statements: vec![],
                };

                let adjusted_severity = self.debt_classifier.adjust_severity(severity, &context);
                let context_hash = self.hash_context(file_path, line_num, &content);

                return Ok(Some(TechnicalDebt {
                    category,
                    severity: adjusted_severity,
                    text: content.trim().to_string(),
                    file: file_path.to_path_buf(),
                    line: line_num,
                    column: self.find_comment_column(line),
                    context_hash,
                }));
            }
        }

        Ok(None)
    }

    /// Extract comment content from various comment styles
    fn extract_comment_content(&self, line: &str) -> Result<Option<String>, TemplateError> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Input validation
        if line.len() > 10000 {
            return Err(TemplateError::ValidationError {
                parameter: "line".to_string(),
                reason: "Line too long for comment extraction (>10000 chars)".to_string(),
            });
        }

        let trimmed = line.trim();

        // Rust/C++/JavaScript style comments
        if let Some(content) = trimmed.strip_prefix("//") {
            return Ok(Some(content.trim().to_string()));
        }

        // Python/Shell style comments
        if let Some(content) = trimmed.strip_prefix('#') {
            return Ok(Some(content.trim().to_string()));
        }

        // Multi-line comment content (/* ... */)
        if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
            let content = &trimmed[2..trimmed.len() - 2];
            return Ok(Some(content.trim().to_string()));
        }

        // HTML/XML comments
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            let content = &trimmed[4..trimmed.len() - 3];
            return Ok(Some(content.trim().to_string()));
        }

        Ok(None)
    }

    /// Find the column where the comment starts
    fn find_comment_column(&self, line: &str) -> u32 {
        debug_assert!(!line.is_empty(), "line must not be empty");
        if let Some(pos) = line.find("//") {
            return pos as u32 + 1;
        }
        if let Some(pos) = line.find('#') {
            return pos as u32 + 1;
        }
        if let Some(pos) = line.find("/*") {
            return pos as u32 + 1;
        }
        if let Some(pos) = line.find("<!--") {
            return pos as u32 + 1;
        }
        1
    }

    /// Generate context hash for debt identity tracking
    fn hash_context(&self, file_path: &Path, line_num: u32, content: &str) -> [u8; 16] {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let mut hasher = Hasher::new();

        // Hash structural elements for stability across refactorings
        hasher.update(file_path.to_string_lossy().as_bytes());
        hasher.update(&line_num.to_le_bytes());
        hasher.update(content.as_bytes());

        let hash = hasher.finalize();
        hash.as_bytes()[..16].try_into().expect("internal error")
    }
}
