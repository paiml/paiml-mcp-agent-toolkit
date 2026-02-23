//! SATD detection, scanning, and file processing logic.

use blake3::Hasher;
use std::path::{Path, PathBuf};

use crate::models::error::TemplateError;

use super::types::{
    AstContext, AstNodeType, DebtClassifier, ProjectAnalysisStats, SATDAnalysisResult,
    SATDDetector, SATDSummary, TechnicalDebt, TestBlockTracker,
};

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
        let mut hasher = Hasher::new();

        // Hash structural elements for stability across refactorings
        hasher.update(file_path.to_string_lossy().as_bytes());
        hasher.update(&line_num.to_le_bytes());
        hasher.update(content.as_bytes());

        let hash = hasher.finalize();
        hash.as_bytes()[..16].try_into().expect("internal error")
    }

    /// Analyze project for SATD patterns
    /// Toyota Way: Extract Method - reduced complexity from 25-><=8
    pub async fn analyze_project(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<SATDAnalysisResult, TemplateError> {
        let files = self.find_source_files(root).await?;
        let mut analysis_stats = ProjectAnalysisStats::new();

        self.process_project_files(&files, include_tests, &mut analysis_stats)
            .await;
        let avg_age_days = self
            .calculate_project_debt_age(&analysis_stats.all_debts, root)
            .await;

        Ok(self.build_analysis_result(analysis_stats, avg_age_days))
    }

    /// Toyota Way: Extract Method - process all files in project (complexity <=8)
    async fn process_project_files(
        &self,
        files: &[std::path::PathBuf],
        include_tests: bool,
        stats: &mut ProjectAnalysisStats,
    ) {
        for file_path in files {
            if self.should_skip_file(file_path, include_tests).await {
                continue;
            }

            stats.total_files_analyzed += 1;
            self.process_single_file(file_path, stats).await;
        }
    }

    /// Toyota Way: Extract Method - check if file should be skipped (complexity <=8)
    async fn should_skip_file(&self, file_path: &Path, include_tests: bool) -> bool {
        // Skip test files if not requested
        if !include_tests && self.is_test_file(file_path) {
            return true;
        }

        // Skip minified/vendor files
        if self.is_minified_or_vendor_file(file_path) {
            return true;
        }

        // Check file size constraints
        if let Ok(metadata) = tokio::fs::metadata(file_path).await {
            if metadata.len() > crate::services::file_classifier::LARGE_FILE_THRESHOLD as u64 {
                eprintln!("Warning: Skipped: {} (large file >500KB)", file_path.display());
                return true;
            }

            if metadata.len() > 1_000_000 && self.is_likely_minified_content(file_path).await {
                eprintln!("Warning: Skipped: {} (minified content)", file_path.display());
                return true;
            }
        }

        false
    }

    /// Toyota Way: Extract Method - process individual file (complexity <=8)
    async fn process_single_file(&self, file_path: &Path, stats: &mut ProjectAnalysisStats) {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                if content.len() > 10_000_000 {
                    eprintln!(
                        "Warning: Skipping large file {}: {} bytes",
                        file_path.display(),
                        content.len()
                    );
                    return;
                }

                match self.extract_from_content(&content, file_path) {
                    Ok(debts) => {
                        if !debts.is_empty() {
                            stats.files_with_debt += 1;
                        }
                        stats.all_debts.extend(debts);
                    }
                    Err(_e) => {
                        // Silently skip files that fail parsing (e.g., line too long)
                        // Analysis continues successfully with remaining files
                        // BUG-010: Removed noisy warning that interleaved with progress
                    }
                }
            }
            Err(_e) => {
                // Silently skip unreadable files
                // BUG-010: Removed noisy warning that interleaved with progress
            }
        }
    }

    /// Toyota Way: Extract Method - calculate debt age (complexity <=3)
    async fn calculate_project_debt_age(&self, debts: &[TechnicalDebt], root: &Path) -> f64 {
        if !debts.is_empty() && root.join(".git").exists() {
            self.calculate_average_debt_age(debts, root)
                .await
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Toyota Way: Extract Method - build analysis result (complexity <=5)
    fn build_analysis_result(
        &self,
        stats: ProjectAnalysisStats,
        avg_age_days: f64,
    ) -> SATDAnalysisResult {
        SATDAnalysisResult {
            items: stats.all_debts.clone(),
            summary: SATDSummary {
                total_items: stats.all_debts.len(),
                by_severity: self.group_debts_by_severity(&stats.all_debts),
                by_category: self.group_debts_by_category(&stats.all_debts),
                files_with_satd: stats.files_with_debt,
                avg_age_days,
            },
            total_files_analyzed: stats.total_files_analyzed,
            files_with_debt: stats.files_with_debt,
            analysis_timestamp: chrono::Utc::now(),
        }
    }

    /// Toyota Way: Extract Method - group debts by severity (complexity <=3)
    fn group_debts_by_severity(
        &self,
        debts: &[TechnicalDebt],
    ) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::with_capacity(3);
        for debt in debts {
            *map.entry(format!("{:?}", debt.severity)).or_insert(0) += 1;
        }
        map
    }

    /// Toyota Way: Extract Method - group debts by category (complexity <=3)
    fn group_debts_by_category(
        &self,
        debts: &[TechnicalDebt],
    ) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::with_capacity(5);
        for debt in debts {
            *map.entry(format!("{:?}", debt.category)).or_insert(0) += 1;
        }
        map
    }

    /// Analyze debt in a directory recursively (excluding test files by default)
    pub async fn analyze_directory(
        &self,
        root: &Path,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        self.analyze_directory_with_tests(root, false).await
    }

    /// Analyze debt in a directory recursively with test file inclusion control
    pub async fn analyze_directory_with_tests(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        let mut all_debts = Vec::new();
        let files = self.find_source_files(root).await?;

        for file_path in files {
            if self
                .should_skip_file_for_analysis(&file_path, include_tests)
                .await
            {
                continue;
            }

            let debts = self.process_file_for_debts(&file_path).await;
            all_debts.extend(debts);
        }

        Ok(all_debts)
    }

    async fn should_skip_file_for_analysis(&self, file_path: &Path, include_tests: bool) -> bool {
        // Skip test files unless explicitly requested
        if !include_tests && self.is_test_file(file_path) {
            return true;
        }

        // Skip minified/vendor files
        if self.is_minified_or_vendor_file(file_path) {
            return true;
        }

        // Check file size and minification for large files
        self.should_skip_large_file(file_path).await
    }

    async fn should_skip_large_file(&self, file_path: &Path) -> bool {
        if let Ok(metadata) = tokio::fs::metadata(file_path).await {
            if metadata.len() > 1_000_000 && self.is_likely_minified_content(file_path).await {
                return true;
            }
        }
        false
    }

    async fn process_file_for_debts(&self, file_path: &Path) -> Vec<TechnicalDebt> {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => self.extract_debts_from_content(&content, file_path),
            Err(_e) => {
                // Silently skip unreadable files
                // BUG-010: Removed noisy warning that interleaved with progress
                Vec::new()
            }
        }
    }

    fn extract_debts_from_content(&self, content: &str, file_path: &Path) -> Vec<TechnicalDebt> {
        // Validate file size before processing
        if content.len() > 10_000_000 {
            eprintln!(
                "Warning: Skipping large file {}: {} bytes",
                file_path.display(),
                content.len()
            );
            return Vec::new();
        }

        // Silently skip files that fail parsing (BUG-010: Removed noisy warning)
        self.extract_from_content(content, file_path)
            .unwrap_or_default()
    }

    /// Find all source files in a directory, respecting .gitignore.
    /// Uses `git ls-files` for tracked repos, falls back to recursive walk.
    pub(crate) async fn find_source_files(
        &self,
        root: &Path,
    ) -> Result<Vec<PathBuf>, TemplateError> {
        // Try git ls-files first to respect .gitignore
        if let Ok(output) = tokio::process::Command::new("git")
            .args(["ls-files", "--cached", "--others", "--exclude-standard"])
            .current_dir(root)
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let files: Vec<PathBuf> = stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| root.join(line))
                    .filter(|path| self.is_valid_source_file(path))
                    .collect();
                if !files.is_empty() {
                    return Ok(files);
                }
            }
        }
        // Fallback: recursive walk (non-git projects)
        let mut files = Vec::new();
        self.collect_files_recursive(root, &mut files).await?;
        Ok(files)
    }

    /// Recursively collect source files
    fn collect_files_recursive<'a>(
        &'a self,
        dir: &'a Path,
        files: &'a mut Vec<PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), TemplateError>> + Send + 'a>>
    {
        Box::pin(async move {
            if !dir.is_dir() {
                return Ok(());
            }

            let mut entries = tokio::fs::read_dir(dir).await.map_err(TemplateError::Io)?;

            while let Some(entry) = entries.next_entry().await.map_err(TemplateError::Io)? {
                let path = entry.path();
                self.process_directory_entry(&path, files).await?;
            }

            Ok(())
        })
    }

    async fn process_directory_entry(
        &self,
        path: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        if path.is_dir() {
            self.process_subdirectory(path, files).await
        } else {
            self.process_file(path, files);
            Ok(())
        }
    }

    async fn process_subdirectory(
        &self,
        path: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        if self.should_skip_directory(path) {
            return Ok(());
        }
        self.collect_files_recursive(path, files).await
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.is_excluded_directory_name(name)
        } else {
            false
        }
    }

    fn is_excluded_directory_name(&self, name: &str) -> bool {
        name.starts_with('.') || self.is_common_build_directory(name)
    }

    fn is_common_build_directory(&self, name: &str) -> bool {
        [
            "target",
            "node_modules",
            "dist",
            "build",
            "__pycache__",
            "book",
        ]
        .contains(&name)
    }

    fn process_file(&self, path: &Path, files: &mut Vec<PathBuf>) {
        if self.is_valid_source_file(path) {
            files.push(path.to_path_buf());
        }
    }

    fn is_valid_source_file(&self, path: &Path) -> bool {
        self.is_source_file(path) && !self.is_test_file(path)
    }

    /// Check if a file is a supported source file
    pub(crate) fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "java"
                    | "cpp"
                    | "c"
                    | "h"
                    | "hpp"
                    | "cs"
                    | "go"
                    | "php"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "scala"
                    | "clj"
                    | "hs"
                    | "ml"
                    | "elm"
            )
        } else {
            false
        }
    }

    /// Check if a file is a test file
    pub(crate) fn is_test_file(&self, path: &Path) -> bool {
        // Check if path contains test directories
        let path_str = path.to_string_lossy();
        if path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.contains("\\tests\\")
            || path_str.contains("\\test\\")
        {
            return true;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Common test file patterns
            file_name.contains("test")
                || file_name.contains("spec")
                || file_name.ends_with("_test.rs")
                || file_name.ends_with("_test.py")
                || file_name.ends_with("_test.js")
                || file_name.ends_with("_test.ts")
                || file_name.ends_with(".test.js")
                || file_name.ends_with(".test.ts")
                || file_name.ends_with(".spec.js")
                || file_name.ends_with(".spec.ts")
        } else {
            false
        }
    }

    /// Check if file is minified or in vendor directory
    /// Check if file should be excluded from SATD analysis
    pub(crate) fn should_exclude_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        self.is_satd_analysis_tool(&path_str)
            || self.is_build_or_config_file(&path_str)
            || self.is_example_or_demo(&path_str)
            || self.is_fuzz_target(&path_str)
            || self.is_generated_or_vendor(&path_str)
    }

    fn is_satd_analysis_tool(&self, path_str: &str) -> bool {
        path_str.contains("satd_detector")
            || path_str.contains("satd_property_tests")
            || path_str.contains("quality_proxy")
            || (path_str.contains("test") && path_str.contains("satd"))
    }

    fn is_build_or_config_file(&self, path_str: &str) -> bool {
        path_str.contains("/build.rs")
            || path_str.contains("/Cargo.toml")
            || path_str.contains(".gitignore")
            || path_str.contains("README")
    }

    fn is_example_or_demo(&self, path_str: &str) -> bool {
        path_str.contains("/examples/") || path_str.contains("/demo/") || path_str.contains("_demo")
    }

    fn is_fuzz_target(&self, path_str: &str) -> bool {
        path_str.contains("/fuzz/") || path_str.contains("fuzz_targets")
    }

    fn is_generated_or_vendor(&self, path_str: &str) -> bool {
        path_str.contains("/target/")
            || path_str.contains("/vendor/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/book/")
            || path_str.contains(".generated")
    }

    /// Check if line is false positive SATD
    pub(crate) fn is_false_positive_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        self.is_string_literal(trimmed)
            || self.is_raw_string_literal(trimmed)
            || self.is_satd_processing_code(trimmed)
            || self.is_assignment_with_satd(trimmed)
            || self.is_format_string(trimmed)
            || self.is_url_or_path(trimmed)
            || self.is_markdown_header(trimmed)
            || self.is_security_documentation(trimmed)
            || self.is_pattern_definition(trimmed)
            || self.is_enum_or_struct_field(trimmed)
            || self.is_functional_description(trimmed)
    }

    fn is_string_literal(&self, trimmed: &str) -> bool {
        trimmed.contains(r#""TODO"#)
            || trimmed.contains(r#""FIXME"#)
            || trimmed.contains(r#""HACK"#)
            || trimmed.contains(r#"'TODO'"#)
            || trimmed.contains(r#"'FIXME'"#)
            || trimmed.contains(r#"'HACK'"#)
    }

    fn is_raw_string_literal(&self, trimmed: &str) -> bool {
        trimmed.contains("r#\"") || trimmed.contains("r\"")
    }

    fn is_satd_processing_code(&self, trimmed: &str) -> bool {
        trimmed.contains(".matches(")
            || trimmed.contains("regex:")
            || trimmed.contains("DebtPattern")
            || trimmed.contains("comment_text:")
            || trimmed.contains("classify_comment")
            || trimmed.contains("debt_classifier")
            || trimmed.contains("SATDAnalysis")
    }

    fn is_assignment_with_satd(&self, trimmed: &str) -> bool {
        trimmed.contains(" = ") && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_format_string(&self, trimmed: &str) -> bool {
        (trimmed.contains("format!")
            || trimmed.contains("println!")
            || trimmed.contains("write!")
            || trimmed.contains("{}"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_url_or_path(&self, trimmed: &str) -> bool {
        // Check for actual URLs or file paths, not just comment markers
        (trimmed.contains("http://")
            || trimmed.contains("https://")
            || trimmed.contains("file://")
            || trimmed.contains(".com/")
            || (trimmed.contains('/') && !trimmed.starts_with("//"))
            || trimmed.contains('\\'))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_security_documentation(&self, trimmed: &str) -> bool {
        // Security-related documentation/comments (not actual security debt)
        (trimmed.contains("Security") || trimmed.contains("security"))
            && (trimmed.contains("check")
                || trimmed.contains("validation")
                || trimmed.contains("properties")
                || trimmed.contains("vulnerabilities")
                || trimmed.contains("patterns")
                || trimmed.contains("issues")
                || trimmed.contains("concerns")
                || trimmed.starts_with("//")
                || trimmed.starts_with('*')
                || trimmed.starts_with('/'))
    }

    fn is_pattern_definition(&self, trimmed: &str) -> bool {
        // Pattern definitions in SATD detection code
        trimmed.contains("let valid_patterns")
            || trimmed.contains("let patterns")
            || trimmed.contains("vec![\"")
            || (trimmed.contains("\"TODO\"") && trimmed.contains('['))
            || (trimmed.contains("FIXME") && trimmed.contains("regex"))
    }

    fn is_enum_or_struct_field(&self, trimmed: &str) -> bool {
        // Enum variants or struct fields that mention SATD concepts
        (trimmed.contains("Security") || trimmed.contains("Design") || trimmed.contains("Defect"))
            && (trimmed.contains(',') || trimmed.contains('=') || trimmed.contains("::"))
    }

    fn is_markdown_header(&self, trimmed: &str) -> bool {
        // Markdown headers: # Security, ## Security, ### Security, etc.
        // Common in CHANGELOG.md, README.md, and documentation templates
        let starts_with_hash = trimmed.starts_with('#');
        if !starts_with_hash {
            return false;
        }

        // Remove leading # symbols and whitespace to get header content
        let content = trimmed.trim_start_matches('#').trim();

        // Check if it's a common section header (especially CHANGELOG sections)
        // or a version header pattern like [1.0.0]
        content == "Security"
            || content == "Added"
            || content == "Changed"
            || content == "Deprecated"
            || content == "Removed"
            || content == "Fixed"
            || content == "Unreleased"
            || content == "Changelog"
            || content == "CHANGELOG"
            || content.starts_with('[') // [Unreleased], [1.0.0], etc.
    }

    fn is_functional_description(&self, trimmed: &str) -> bool {
        // Comments describing functionality, not admitting technical debt
        if trimmed.starts_with("//") {
            let comment_text = trimmed.trim_start_matches("//").trim().to_lowercase();

            // Section headers with separators (=== or ---)
            if comment_text.contains("===")
                || comment_text.contains("---")
                || comment_text.contains("───")
            {
                return true;
            }

            // Mathematical notation (e.g., "s^T x temp")
            if comment_text.contains("\u{00d7}")
                || comment_text.contains("\u{2211}")
                || comment_text.contains("^t ")
                || comment_text.contains("^t\u{00d7}")
            {
                return true;
            }

            // Section header patterns (capitalized with parenthetical)
            if comment_text.contains("mitigation")
                || comment_text.contains("isolation")
                || comment_text.starts_with("output ")
                || comment_text.starts_with("input ")
                || comment_text.starts_with("all ")
            {
                return true;
            }

            // Phone/format patterns with XXX
            if comment_text.contains("xxx-xxx") || comment_text.contains("xxx.xxx") {
                return true;
            }

            // Check for common functional description patterns
            comment_text.starts_with("check for")
                || comment_text.starts_with("handle ")
                || comment_text.starts_with("phase ")
                || comment_text.starts_with("load ")
                || comment_text.starts_with("create ")
                || comment_text.starts_with("process ")
                || comment_text.starts_with("detect ")
                || comment_text.starts_with("scan ")
                || comment_text.starts_with("parse ")
                || comment_text.starts_with("analyze ")
                || comment_text.starts_with("extract ")
                || comment_text.starts_with("find ")
                || comment_text.starts_with("search ")
                || comment_text.starts_with("identify ")
                || comment_text.starts_with("validate ")
                || comment_text.starts_with("verify ")
                || comment_text.contains("relative links")
                || comment_text.contains("special modes")
                || comment_text.contains("documentation issues")
                || comment_text.contains("single file")
                || (comment_text.contains("broken") && comment_text.contains("links"))
                || (comment_text.contains("bug") && comment_text.contains("report"))
                // False positive fixes: Comments describing bug-related functionality
                || (comment_text.contains("broken") && comment_text.contains("dep"))
                || (comment_text.contains("bug") && comment_text.contains("fix") && (comment_text.contains("pattern") || comment_text.contains("patterns")))
                || (comment_text.contains("bug") && comment_text.contains("fix") && (comment_text.contains("claim") || comment_text.contains("claims")))
                || (comment_text.contains("bug") && comment_text.contains("fix") && comment_text.contains("commit"))
                || (comment_text.contains("describes functionality") && comment_text.contains("bug"))
                || (comment_text.contains("extract") && comment_text.contains("bug"))
                // Bug tracking ID patterns (BUG-XXX, PMAT-BUG-XXX like JIRA tickets)
                || self.is_bug_tracking_id(&comment_text)
                // Fixed bug descriptions ("Bug: Previously...", "BUG-064 FIX:")
                || self.is_fixed_bug_description(&comment_text)
                // Bug estimation/metrics functionality
                || (comment_text.contains("bug") && comment_text.contains("estimate"))
                // Comments about detection/markers (describing functionality, not debt)
                || (comment_text.contains("marker") && !comment_text.contains("add"))
                || (comment_text.contains("detection") && !comment_text.contains("need"))
                || (comment_text.contains("pattern") && comment_text.contains("match"))
        } else {
            false
        }
    }

    /// Check if comment contains bug tracking ID (like JIRA tickets)
    /// Patterns: BUG-123, PMAT-BUG-456, Issue-789
    fn is_bug_tracking_id(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        // Pattern 1: BUG-XXX (where XXX is digits)
        if text_lower.contains("bug-") {
            // Check if followed by digits
            if let Some(pos) = text_lower.find("bug-") {
                let after_dash = &text[pos + 4..];
                if after_dash.chars().take(3).all(|c| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
        // Pattern 2: PMAT-BUG-XXX, PROJECT-BUG-XXX
        if text_lower.contains("-bug-") {
            return true;
        }
        // Pattern 3: "BUG-XXX FIX:" or "BUG-XXX:" at start
        if text_lower.contains("bug-") && (text_lower.contains(" fix:") || text_lower.contains(":"))
        {
            return true;
        }
        false
    }

    /// Check if comment describes a FIXED bug (not a current bug)
    /// Patterns: "Bug: Previously...", "CRITICAL FIX:", "Root cause:", etc.
    fn is_fixed_bug_description(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        // Pattern 1: "Bug: Previously..." - past tense description
        if text_lower.starts_with("bug:") && text_lower.contains("previous") {
            return true;
        }
        // Pattern 2: "CRITICAL FIX:", "BUG FIX:"
        if text_lower.contains(" fix:") {
            return true;
        }
        // Pattern 3: "This ensures..." after "Bug: ..." (describing fix)
        if text_lower.contains("bug:")
            && (text_lower.contains("ensure") || text_lower.contains("prevent"))
        {
            return true;
        }
        // Pattern 4: "Root cause:" explanations (often follow bug IDs)
        if text_lower.contains("root cause") {
            return true;
        }
        false
    }

    /// Check if line is documentation, test, or metadata about SATD
    pub(crate) fn is_documentation_or_metadata(&self, line: &str) -> bool {
        let trimmed = line.trim();

        self.is_documentation_comment(trimmed)
            || self.is_test_code(trimmed)
            || self.is_log_message(trimmed)
            || self.is_error_description(trimmed)
    }

    fn is_documentation_comment(&self, trimmed: &str) -> bool {
        self.is_module_documentation(trimmed)
            || self.is_technical_debt_documentation(trimmed)
            || self.is_api_documentation(trimmed)
            || self.is_doctest_example(trimmed)
    }

    fn is_module_documentation(&self, trimmed: &str) -> bool {
        trimmed.starts_with("//!") || trimmed.starts_with("///")
    }

    fn is_technical_debt_documentation(&self, trimmed: &str) -> bool {
        let lower = trimmed.to_lowercase();
        let mentions_td_concepts = lower.contains("technical debt")
            || trimmed.contains("TDG")
            || trimmed.contains("SATD")
            || lower.contains("self-admitted")
            || lower.contains("debt marker")
            || lower.contains("debt detection")
            || lower.contains("debt pattern");
        let is_comment =
            trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with('/');
        mentions_td_concepts && is_comment
    }

    fn is_api_documentation(&self, trimmed: &str) -> bool {
        let is_doc_marker = trimmed.starts_with('*')
            || trimmed.contains("@param")
            || trimmed.contains("@return")
            || trimmed.contains("Example:")
            || trimmed.contains("# Examples")
            || trimmed.contains("# Parameters");
        let mentions_markers =
            trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("security");
        is_doc_marker && mentions_markers
    }

    fn is_doctest_example(&self, trimmed: &str) -> bool {
        let has_comment_marker = trimmed.contains("// ");
        let has_debt_marker = trimmed.contains("TODO") || trimmed.contains("FIXME");
        let has_code_marker =
            trimmed.contains("let ") || trimmed.contains("assert") || trimmed.contains("unwrap");
        has_comment_marker && has_debt_marker && has_code_marker
    }

    fn is_test_code(&self, trimmed: &str) -> bool {
        (trimmed.contains("assert")
            || trimmed.contains("expect")
            || trimmed.contains(".unwrap()")
            || trimmed.contains("panic!"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_log_message(&self, trimmed: &str) -> bool {
        (trimmed.contains("log::")
            || trimmed.contains("debug!")
            || trimmed.contains("info!")
            || trimmed.contains("warn!")
            || trimmed.contains("error!")
            || trimmed.contains("trace!"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    fn is_error_description(&self, trimmed: &str) -> bool {
        (trimmed.contains("Error:")
            || trimmed.contains("error:")
            || trimmed.contains("message:")
            || trimmed.contains("description:"))
            && (trimmed.contains("TODO") || trimmed.contains("FIXME"))
    }

    /// Comprehensive false positive detection for SATD
    pub(crate) fn is_likely_test_data_or_pattern(&self, line: &str, file_path: &Path) -> bool {
        // First check: Should we exclude this entire file?
        if self.should_exclude_file(file_path) {
            return true;
        }

        // Second check: Is this line a false positive?
        if self.is_false_positive_line(line) {
            return true;
        }

        // Third check: Is this documentation or metadata?
        if self.is_documentation_or_metadata(line) {
            return true;
        }

        false
    }

    pub(crate) fn is_minified_or_vendor_file(&self, path: &Path) -> bool {
        // Check if path contains vendor directory
        if path.components().any(|c| c.as_os_str() == "vendor") {
            return true;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Common minified file patterns
            file_name.contains(".min.")
                || file_name.contains(".bundle.")
                || file_name.contains("-min.")
                || file_name.contains(".production.")
                || file_name.ends_with(".min.js")
                || file_name.ends_with(".min.css")
                || file_name.ends_with(".bundle.js")
                || file_name.ends_with(".production.js")
        } else {
            false
        }
    }

    /// Check if file content suggests it's minified (has very long lines)
    pub(crate) async fn is_likely_minified_content(&self, path: &Path) -> bool {
        use tokio::io::{AsyncBufReadExt, BufReader};

        match tokio::fs::File::open(path).await {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines = reader.lines();

                // Check first few lines for length
                for _ in 0..3 {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            if line.len() > 5000 {
                                return true; // Very long line, likely minified
                            }
                        }
                        Ok(None) => break,
                        Err(_) => return false,
                    }
                }
                false
            }
            Err(_) => false,
        }
    }
}
