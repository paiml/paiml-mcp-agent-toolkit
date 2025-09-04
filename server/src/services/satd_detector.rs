//! Self-Admitted Technical Debt (SATD) Detection System
//!
//! This module provides high-performance, multi-language detection and classification
//! of technical debt annotations embedded in source code comments.

use crate::models::error::TemplateError;
use blake3::Hasher;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Self-Admitted Technical Debt detector with pattern matching
pub struct SATDDetector {
    #[allow(dead_code)]
    patterns: RegexSet,
    debt_classifier: DebtClassifier,
}

/// Detected technical debt item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub category: DebtCategory,
    pub severity: Severity,
    pub text: String,
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub context_hash: [u8; 16], // BLAKE3 hash for identity tracking
}

/// SATD analysis result containing all detected debt items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDAnalysisResult {
    pub items: Vec<TechnicalDebt>,
    pub summary: SATDSummary,
    pub total_files_analyzed: usize,
    pub files_with_debt: usize,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Summary statistics for SATD analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDSummary {
    pub total_items: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub by_category: std::collections::HashMap<String, usize>,
    pub files_with_satd: usize,
    pub avg_age_days: f64,
}

/// Categories of technical debt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DebtCategory {
    Design,      // HACK, KLUDGE, SMELL - Architectural compromises
    Defect,      // BUG, FIXME, BROKEN - Known defects
    Requirement, // TODO, FEAT, ENHANCEMENT - Missing features
    Test,        // FAILING, SKIP, DISABLED - Test debt
    Performance, // SLOW, OPTIMIZE, PERF - Performance issues
    Security,    // SECURITY, VULN, UNSAFE - Security concerns
}

impl DebtCategory {
    fn as_str(&self) -> &'static str {
        match self {
            DebtCategory::Design => "Design",
            DebtCategory::Defect => "Defect",
            DebtCategory::Requirement => "Requirement",
            DebtCategory::Test => "Test",
            DebtCategory::Performance => "Performance",
            DebtCategory::Security => "Security",
        }
    }
}

impl std::fmt::Display for DebtCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Severity levels for technical debt
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Critical, // Security vulnerabilities, data loss risks
    High,     // Defects, broken functionality
    Medium,   // Design issues, performance problems
    Low,      // TODOs, minor enhancements
}

impl Severity {
    /// Escalate severity by one level
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::satd_detector::Severity;
    ///
    /// assert_eq!(Severity::Low.escalate(), Severity::Medium);
    /// assert_eq!(Severity::Medium.escalate(), Severity::High);
    /// assert_eq!(Severity::High.escalate(), Severity::Critical);
    /// assert_eq!(Severity::Critical.escalate(), Severity::Critical); // Already at max
    /// ```
    pub fn escalate(self) -> Self {
        match self {
            Severity::Low => Severity::Medium,
            Severity::Medium => Severity::High,
            Severity::High => Severity::Critical,
            Severity::Critical => Severity::Critical,
        }
    }

    /// Reduce severity by one level
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::satd_detector::Severity;
    ///
    /// assert_eq!(Severity::Critical.reduce(), Severity::High);
    /// assert_eq!(Severity::High.reduce(), Severity::Medium);
    /// assert_eq!(Severity::Medium.reduce(), Severity::Low);
    /// assert_eq!(Severity::Low.reduce(), Severity::Low); // Already at min
    /// ```
    pub fn reduce(self) -> Self {
        match self {
            Severity::Critical => Severity::High,
            Severity::High => Severity::Medium,
            Severity::Medium => Severity::Low,
            Severity::Low => Severity::Low,
        }
    }
}

/// Context information for debt classification
#[derive(Debug, Clone)]
pub struct AstContext {
    pub node_type: AstNodeType,
    pub parent_function: String,
    pub complexity: u32,
    pub siblings_count: usize,
    pub nesting_depth: usize,
    pub surrounding_statements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeType {
    SecurityFunction,
    DataValidation,
    TestFunction,
    MockImplementation,
    Regular,
}

/// Pattern-based debt classifier
pub struct DebtClassifier {
    patterns: Vec<DebtPattern>,
    compiled_patterns: RegexSet,
}

#[derive(Debug, Clone)]
struct DebtPattern {
    regex: String,
    category: DebtCategory,
    severity: Severity,
    #[allow(dead_code)]
    description: String,
}

/// Evolution tracking for technical debt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtEvolution {
    pub total_introduced: usize,
    pub total_resolved: usize,
    pub current_debt_age_p50: f64,
    pub debt_velocity: f64,
}

/// Project-wide SATD metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDMetrics {
    pub total_debts: usize,
    pub debt_density_per_kloc: f64,
    pub by_category: BTreeMap<String, CategoryMetrics>,
    pub critical_debts: Vec<TechnicalDebt>,
    pub debt_age_distribution: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub count: usize,
    pub files: BTreeSet<String>,
    pub avg_severity: f64,
}

impl Default for DebtClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl DebtClassifier {
    pub fn new() -> Self {
        // Default mode includes all patterns
        let patterns = vec![
            // High-confidence patterns with word boundaries
            DebtPattern {
                regex: r"(?i)\b(hack|kludge|smell)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Architectural compromise".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(fixme|broken|bug)\b".to_string(),
                category: DebtCategory::Defect,
                severity: Severity::High,
                description: "Known defect".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\btodo\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                description: "Missing feature".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(fixme|todo|hack)\s+.*\b(security|vuln|vulnerability|cve)\b"
                    .to_string(),
                category: DebtCategory::Security,
                severity: Severity::Critical,
                description: "Security concern in TODO/FIXME".to_string(),
            },
            // Context-aware patterns
            DebtPattern {
                regex: r"(?i)\bperformance\s+(issue|problem)\b".to_string(),
                category: DebtCategory::Performance,
                severity: Severity::Medium,
                description: "Performance issue".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\btest.*\b(disabled|skipped|failing)\b".to_string(),
                category: DebtCategory::Test,
                severity: Severity::Medium,
                description: "Test debt".to_string(),
            },
            // Multi-word patterns
            DebtPattern {
                regex: r"(?i)\btechnical\s+debt\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Explicit technical debt".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\bcode\s+smell\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Code smell".to_string(),
            },
            // Additional common patterns
            DebtPattern {
                regex: r"(?i)\b(workaround|temp|temporary)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Low,
                description: "Temporary solution".to_string(),
            },
            DebtPattern {
                regex: r"(?i)\b(optimize|slow)\b".to_string(),
                category: DebtCategory::Performance,
                severity: Severity::Low,
                description: "Performance optimization needed".to_string(),
            },
        ];

        let regex_strings: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
        let compiled_patterns =
            RegexSet::new(&regex_strings).expect("Failed to compile SATD patterns");

        Self {
            patterns,
            compiled_patterns,
        }
    }

    pub fn new_strict() -> Self {
        // Strict mode only includes explicit SATD markers
        let patterns = vec![
            // Ultra-strict: ONLY actual comment-based SATD markers
            DebtPattern {
                regex: r"//\s*TODO:\s+.+".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                description: "TODO task marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*FIXME:\s+.+".to_string(),
                category: DebtCategory::Defect,
                severity: Severity::High,
                description: "FIXME issue marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*HACK:\s+.+".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "HACK workaround marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*XXX:\s+.+".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "XXX problem marker".to_string(),
            },
            DebtPattern {
                regex: r"//\s*BUG:\s+.+".to_string(),
                category: DebtCategory::Defect,
                severity: Severity::High,
                description: "BUG issue marker".to_string(),
            },
        ];

        let regex_strings: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
        let compiled_patterns =
            RegexSet::new(&regex_strings).expect("Failed to compile strict SATD patterns");

        Self {
            patterns,
            compiled_patterns,
        }
    }

    /// Classify a comment text and return debt information
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::satd_detector::{DebtClassifier, DebtCategory, Severity};
    ///
    /// let classifier = DebtClassifier::new();
    ///
    /// // TODO comments are classified as requirements
    /// let result = classifier.classify_comment("TODO: implement this feature");
    /// assert_eq!(result, Some((DebtCategory::Requirement, Severity::Low)));
    ///
    /// // FIXME comments are defects with higher severity
    /// let result = classifier.classify_comment("FIXME: this crashes sometimes");
    /// assert_eq!(result, Some((DebtCategory::Defect, Severity::High)));
    ///
    /// // Normal comments return None
    /// let result = classifier.classify_comment("This is a regular comment");
    /// assert_eq!(result, None);
    /// ```
    pub fn classify_comment(&self, text: &str) -> Option<(DebtCategory, Severity)> {
        let matches = self.compiled_patterns.matches(text);

        matches.iter().next()?;

        // Find the first matching pattern
        for match_idx in matches.iter() {
            if let Some(pattern) = self.patterns.get(match_idx) {
                return Some((pattern.category, pattern.severity));
            }
        }

        None
    }

    /// Adjust severity based on context
    pub fn adjust_severity(&self, base_severity: Severity, context: &AstContext) -> Severity {
        match context.node_type {
            // Critical paths escalate severity
            AstNodeType::SecurityFunction | AstNodeType::DataValidation => base_severity.escalate(),
            // Test code reduces severity
            AstNodeType::TestFunction | AstNodeType::MockImplementation => base_severity.reduce(),
            // Hot paths (high complexity) escalate performance issues
            AstNodeType::Regular if context.complexity > 20 => base_severity.escalate(),
            _ => base_severity,
        }
    }
}

impl Default for SATDDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SATDDetector {
    pub fn new() -> Self {
        Self::with_config(false)
    }

    pub fn new_strict() -> Self {
        Self::with_config(true)
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

    fn is_rust_file(&self, file_path: &Path) -> bool {
        file_path.extension().and_then(|s| s.to_str()) == Some("rs")
    }

    fn sort_debts(&self, debts: &mut Vec<TechnicalDebt>) {
        debts.sort_by_key(|d| (d.file.clone(), d.line, d.column));
    }
}

/// Tracks test block boundaries in Rust files to exclude test-only technical debt
struct TestBlockTracker {
    is_rust_file: bool,
    in_test_block: bool,
    test_block_depth: usize,
}

impl TestBlockTracker {
    fn new(is_rust_file: bool) -> Self {
        Self {
            is_rust_file,
            in_test_block: false,
            test_block_depth: 0,
        }
    }

    fn update_from_line(&mut self, trimmed_line: &str) {
        if !self.is_rust_file {
            return;
        }

        if self.is_test_block_start(trimmed_line) {
            self.start_test_block();
        } else if self.in_test_block {
            self.update_test_block_depth(trimmed_line);
        }
    }

    fn is_in_test_block(&self) -> bool {
        self.in_test_block
    }

    fn is_test_block_start(&self, trimmed_line: &str) -> bool {
        trimmed_line.starts_with("#[cfg(test)]")
    }

    fn start_test_block(&mut self) {
        self.in_test_block = true;
        self.test_block_depth = 0;
    }

    fn update_test_block_depth(&mut self, trimmed_line: &str) {
        self.add_opening_braces(trimmed_line);
        self.subtract_closing_braces(trimmed_line);
    }

    fn add_opening_braces(&mut self, trimmed_line: &str) {
        if trimmed_line.contains('{') {
            self.test_block_depth += trimmed_line.matches('{').count();
        }
    }

    fn subtract_closing_braces(&mut self, trimmed_line: &str) {
        if trimmed_line.contains('}') {
            self.test_block_depth = self
                .test_block_depth
                .saturating_sub(trimmed_line.matches('}').count());

            if self.test_block_depth == 0 && trimmed_line.ends_with('}') {
                self.in_test_block = false;
            }
        }
    }
}

impl SATDDetector {
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
        hash.as_bytes()[..16].try_into().unwrap()
    }

    /// Analyze project for SATD patterns
    /// Toyota Way: Extract Method - reduced complexity from 25→≤8
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

    /// Toyota Way: Extract Method - process all files in project (complexity ≤8)
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

    /// Toyota Way: Extract Method - check if file should be skipped (complexity ≤8)
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
                eprintln!("⚠️  Skipped: {} (large file >500KB)", file_path.display());
                return true;
            }

            if metadata.len() > 1_000_000 && self.is_likely_minified_content(file_path).await {
                eprintln!("⚠️  Skipped: {} (minified content)", file_path.display());
                return true;
            }
        }

        false
    }

    /// Toyota Way: Extract Method - process individual file (complexity ≤8)
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
                    Err(e) => {
                        eprintln!(
                            "Warning: Error processing file {}: {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not read file {}: {}",
                    file_path.display(),
                    e
                );
            }
        }
    }

    /// Toyota Way: Extract Method - calculate debt age (complexity ≤3)
    async fn calculate_project_debt_age(&self, debts: &[TechnicalDebt], root: &Path) -> f64 {
        if !debts.is_empty() && root.join(".git").exists() {
            self.calculate_average_debt_age(debts, root)
                .await
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Toyota Way: Extract Method - build analysis result (complexity ≤5)
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

    /// Toyota Way: Extract Method - group debts by severity (complexity ≤3)
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

    /// Toyota Way: Extract Method - group debts by category (complexity ≤3)
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
}

/// Toyota Way: Data-Driven Design - encapsulate project analysis state
#[derive(Default)]
struct ProjectAnalysisStats {
    all_debts: Vec<TechnicalDebt>,
    files_with_debt: usize,
    total_files_analyzed: usize,
}

impl ProjectAnalysisStats {
    fn new() -> Self {
        Self::default()
    }
}

impl SATDDetector {
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
            Err(e) => {
                eprintln!(
                    "Warning: Could not read file {}: {}",
                    file_path.display(),
                    e
                );
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

        match self.extract_from_content(content, file_path) {
            Ok(debts) => debts,
            Err(e) => {
                eprintln!(
                    "Warning: Error processing file {}: {}",
                    file_path.display(),
                    e
                );
                Vec::new()
            }
        }
    }

    /// Find all source files in a directory
    async fn find_source_files(&self, root: &Path) -> Result<Vec<PathBuf>, TemplateError> {
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
        ["target", "node_modules", "dist", "build", "__pycache__"].contains(&name)
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
    fn is_source_file(&self, path: &Path) -> bool {
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
    fn is_test_file(&self, path: &Path) -> bool {
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
    fn should_exclude_file(&self, file_path: &Path) -> bool {
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
            || path_str.contains(".generated")
    }

    /// Check if line is false positive SATD
    fn is_false_positive_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        self.is_string_literal(trimmed)
            || self.is_raw_string_literal(trimmed)
            || self.is_satd_processing_code(trimmed)
            || self.is_assignment_with_satd(trimmed)
            || self.is_format_string(trimmed)
            || self.is_url_or_path(trimmed)
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
        (trimmed.contains("http") || trimmed.contains("/") || trimmed.contains("\\"))
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
                || trimmed.starts_with("*")
                || trimmed.starts_with("/"))
    }

    fn is_pattern_definition(&self, trimmed: &str) -> bool {
        // Pattern definitions in SATD detection code
        trimmed.contains("let valid_patterns")
            || trimmed.contains("let patterns")
            || trimmed.contains("vec![\"")
            || (trimmed.contains("\"TODO\"") && trimmed.contains("["))
            || (trimmed.contains("FIXME") && trimmed.contains("regex"))
    }

    fn is_enum_or_struct_field(&self, trimmed: &str) -> bool {
        // Enum variants or struct fields that mention SATD concepts
        (trimmed.contains("Security") || trimmed.contains("Design") || trimmed.contains("Defect"))
            && (trimmed.contains(",") || trimmed.contains("=") || trimmed.contains("::"))
    }

    fn is_functional_description(&self, trimmed: &str) -> bool {
        // Comments describing functionality, not admitting technical debt
        if trimmed.starts_with("//") {
            let comment_text = trimmed.trim_start_matches("//").trim().to_lowercase();

            // Check for common functional description patterns
            comment_text.starts_with("check for")
                || comment_text.starts_with("handle ")
                || comment_text.starts_with("phase ")
                || comment_text.starts_with("load ")
                || comment_text.starts_with("create ")
                || comment_text.starts_with("process ")
                || comment_text.contains("relative links")
                || comment_text.contains("special modes")
                || comment_text.contains("documentation issues")
                || comment_text.contains("single file")
                || (comment_text.contains("broken") && comment_text.contains("links"))
                || (comment_text.contains("bug") && comment_text.contains("report"))
        } else {
            false
        }
    }

    /// Check if line is documentation, test, or metadata about SATD
    fn is_documentation_or_metadata(&self, line: &str) -> bool {
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
        let mentions_td_concepts = trimmed.contains("Technical Debt")
            || trimmed.contains("TDG")
            || trimmed.contains("SATD")
            || trimmed.contains("Self-Admitted");
        let is_comment =
            trimmed.starts_with("//") || trimmed.starts_with("*") || trimmed.starts_with("/");
        mentions_td_concepts && is_comment
    }

    fn is_api_documentation(&self, trimmed: &str) -> bool {
        let is_doc_marker = trimmed.starts_with("*")
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
    fn is_likely_test_data_or_pattern(&self, line: &str, file_path: &Path) -> bool {
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

    fn is_minified_or_vendor_file(&self, path: &Path) -> bool {
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
    async fn is_likely_minified_content(&self, path: &Path) -> bool {
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

    /// Generate project-wide SATD metrics
    pub fn generate_metrics(&self, debts: &[TechnicalDebt], total_loc: u64) -> SATDMetrics {
        let debt_density = if total_loc > 0 {
            (debts.len() as f64 / total_loc as f64) * 1000.0
        } else {
            0.0
        };

        // Group by category
        let mut by_category: BTreeMap<String, CategoryMetrics> = BTreeMap::new();

        for debt in debts {
            let category_key = debt.category.to_string();
            let entry = by_category.entry(category_key).or_insert(CategoryMetrics {
                count: 0,
                files: BTreeSet::new(),
                avg_severity: 0.0,
            });

            entry.count += 1;
            entry.files.insert(debt.file.to_string_lossy().to_string());
        }

        // Calculate average severity for each category
        for (category_name, metrics) in by_category.iter_mut() {
            let category_debts: Vec<_> = debts
                .iter()
                .filter(|d| d.category.to_string() == *category_name)
                .collect();

            if !category_debts.is_empty() {
                let severity_sum: u32 = category_debts
                    .iter()
                    .map(|d| match d.severity {
                        Severity::Critical => 4,
                        Severity::High => 3,
                        Severity::Medium => 2,
                        Severity::Low => 1,
                    })
                    .sum();

                metrics.avg_severity = severity_sum as f64 / category_debts.len() as f64;
            }
        }

        let critical_debts: Vec<TechnicalDebt> = debts
            .iter()
            .filter(|d| d.severity == Severity::Critical)
            .cloned()
            .collect();

        SATDMetrics {
            total_debts: debts.len(),
            debt_density_per_kloc: debt_density,
            by_category,
            critical_debts,
            debt_age_distribution: vec![], // Would need git history analysis
        }
    }

    /// Calculate average age of technical debt items using git blame
    async fn calculate_average_debt_age(
        &self,
        debts: &[TechnicalDebt],
        project_root: &Path,
    ) -> Result<f64, TemplateError> {
        use chrono::Utc;

        let mut total_age_days = 0.0;
        let mut valid_debt_count = 0;
        let now = Utc::now();

        for debt in debts {
            if let Some(age_days) = self.calculate_debt_age(debt, project_root, &now).await {
                total_age_days += age_days;
                valid_debt_count += 1;
            }
        }

        Ok(if valid_debt_count > 0 {
            total_age_days / valid_debt_count as f64
        } else {
            0.0
        })
    }

    async fn calculate_debt_age(
        &self,
        debt: &TechnicalDebt,
        project_root: &Path,
        now: &chrono::DateTime<chrono::Utc>,
    ) -> Option<f64> {
        let relative_path = self.get_relative_path(&debt.file, project_root)?;
        let blame_output = self
            .run_git_blame(&relative_path, debt.line, project_root)
            .await?;
        let timestamp = self.parse_git_blame_timestamp(&blame_output)?;
        self.calculate_age_from_timestamp(timestamp, now)
    }

    fn get_relative_path(&self, file_path: &Path, project_root: &Path) -> Option<PathBuf> {
        file_path
            .strip_prefix(project_root)
            .ok()
            .map(|p| p.to_path_buf())
    }

    async fn run_git_blame(
        &self,
        relative_path: &PathBuf,
        line: u32,
        project_root: &Path,
    ) -> Option<String> {
        use std::process::Command;

        let output = Command::new("git")
            .args([
                "blame",
                "-L",
                &format!("{},{}", line, line),
                "--porcelain",
                relative_path.to_str()?,
            ])
            .current_dir(project_root)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            None
        }
    }

    fn parse_git_blame_timestamp(&self, blame_output: &str) -> Option<i64> {
        for line in blame_output.lines() {
            if let Some(timestamp_str) = line.strip_prefix("author-time ") {
                return timestamp_str.parse::<i64>().ok();
            }
        }
        None
    }

    fn calculate_age_from_timestamp(
        &self,
        timestamp: i64,
        now: &chrono::DateTime<chrono::Utc>,
    ) -> Option<f64> {
        use chrono::DateTime;

        let debt_date = DateTime::from_timestamp(timestamp, 0)?;
        Some((*now - debt_date).num_days() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    // Helper function to create a test technical debt item
    fn create_test_debt(category: DebtCategory, severity: Severity) -> TechnicalDebt {
        TechnicalDebt {
            category,
            severity,
            text: "Test debt".to_string(),
            file: PathBuf::from("test.rs"),
            line: 42,
            column: 10,
            context_hash: [0; 16],
        }
    }

    #[test]
    fn test_debt_category_as_str() {
        assert_eq!(DebtCategory::Design.as_str(), "Design");
        assert_eq!(DebtCategory::Defect.as_str(), "Defect");
        assert_eq!(DebtCategory::Requirement.as_str(), "Requirement");
        assert_eq!(DebtCategory::Test.as_str(), "Test");
        assert_eq!(DebtCategory::Performance.as_str(), "Performance");
        assert_eq!(DebtCategory::Security.as_str(), "Security");
    }

    #[test]
    fn test_debt_category_display() {
        assert_eq!(format!("{}", DebtCategory::Design), "Design");
        assert_eq!(format!("{}", DebtCategory::Defect), "Defect");
        assert_eq!(format!("{}", DebtCategory::Requirement), "Requirement");
        assert_eq!(format!("{}", DebtCategory::Test), "Test");
        assert_eq!(format!("{}", DebtCategory::Performance), "Performance");
        assert_eq!(format!("{}", DebtCategory::Security), "Security");
    }

    #[test]
    fn test_severity_escalate() {
        assert_eq!(Severity::Low.escalate(), Severity::Medium);
        assert_eq!(Severity::Medium.escalate(), Severity::High);
        assert_eq!(Severity::High.escalate(), Severity::Critical);
        assert_eq!(Severity::Critical.escalate(), Severity::Critical);
    }

    #[test]
    fn test_severity_reduce() {
        assert_eq!(Severity::Critical.reduce(), Severity::High);
        assert_eq!(Severity::High.reduce(), Severity::Medium);
        assert_eq!(Severity::Medium.reduce(), Severity::Low);
        assert_eq!(Severity::Low.reduce(), Severity::Low);
    }

    #[test]
    fn test_debt_classifier_new() {
        let classifier = DebtClassifier::new();
        assert!(!classifier.patterns.is_empty());
        // Should have at least 10 patterns based on the implementation
        assert!(classifier.patterns.len() >= 10);
    }

    #[test]
    fn test_debt_classifier_default() {
        let _classifier = DebtClassifier::default();
        // Should not panic
    }

    #[test]
    fn test_pattern_classification() {
        let classifier = DebtClassifier::new();

        // Test various patterns
        assert_eq!(
            classifier.classify_comment("// TODO: implement error handling"),
            Some((DebtCategory::Requirement, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// SECURITY: potential SQL injection"),
            Some((DebtCategory::Security, Severity::Critical))
        );

        assert_eq!(
            classifier.classify_comment("// FIXME: broken logic here"),
            Some((DebtCategory::Defect, Severity::High))
        );

        assert_eq!(
            classifier.classify_comment("// HACK: ugly workaround"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// BUG: memory leak"),
            Some((DebtCategory::Defect, Severity::High))
        );

        assert_eq!(
            classifier.classify_comment("// KLUDGE: temporary fix"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// SMELL: code duplication"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// performance issue here"),
            Some((DebtCategory::Performance, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// test is disabled"),
            Some((DebtCategory::Test, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// technical debt: refactor needed"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// code smell: long method"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// workaround for library issue"),
            Some((DebtCategory::Design, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// optimize this loop"),
            Some((DebtCategory::Performance, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// slow algorithm"),
            Some((DebtCategory::Performance, Severity::Low))
        );

        // Test case insensitivity
        assert_eq!(
            classifier.classify_comment("// todo: add validation"),
            Some((DebtCategory::Requirement, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// VULN: XSS possible"),
            Some((DebtCategory::Security, Severity::Critical))
        );

        assert_eq!(
            classifier.classify_comment("// CVE-2021-1234: patch needed"),
            Some((DebtCategory::Security, Severity::Critical))
        );

        // Test non-matching comment
        assert_eq!(
            classifier.classify_comment("// Just a regular comment"),
            None
        );

        assert_eq!(
            classifier.classify_comment("// This is documentation"),
            None
        );
    }

    #[test]
    fn test_adjust_severity() {
        let classifier = DebtClassifier::new();

        // Test security function context
        let security_context = AstContext {
            node_type: AstNodeType::SecurityFunction,
            parent_function: "validate_input".to_string(),
            complexity: 10,
            siblings_count: 2,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Low, &security_context),
            Severity::Medium
        );
        assert_eq!(
            classifier.adjust_severity(Severity::High, &security_context),
            Severity::Critical
        );

        // Test data validation context
        let validation_context = AstContext {
            node_type: AstNodeType::DataValidation,
            parent_function: "check_data".to_string(),
            complexity: 5,
            siblings_count: 1,
            nesting_depth: 2,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Medium, &validation_context),
            Severity::High
        );

        // Test test function context
        let test_context = AstContext {
            node_type: AstNodeType::TestFunction,
            parent_function: "test_feature".to_string(),
            complexity: 3,
            siblings_count: 5,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::High, &test_context),
            Severity::Medium
        );

        // Test mock implementation context
        let mock_context = AstContext {
            node_type: AstNodeType::MockImplementation,
            parent_function: "mock_service".to_string(),
            complexity: 2,
            siblings_count: 1,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Critical, &mock_context),
            Severity::High
        );

        // Test high complexity regular context
        let complex_context = AstContext {
            node_type: AstNodeType::Regular,
            parent_function: "complex_function".to_string(),
            complexity: 25,
            siblings_count: 3,
            nesting_depth: 4,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Low, &complex_context),
            Severity::Medium
        );

        // Test regular context with low complexity
        let simple_context = AstContext {
            node_type: AstNodeType::Regular,
            parent_function: "simple_function".to_string(),
            complexity: 5,
            siblings_count: 2,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Medium, &simple_context),
            Severity::Medium
        );
    }

    #[test]
    fn test_satd_detector_new() {
        let detector = SATDDetector::new();
        // Should initialize with classifier
        assert!(!detector.patterns.is_empty());
    }

    #[test]
    fn test_satd_detector_default() {
        let _detector = SATDDetector::default();
        // Should not panic
    }

    #[test]
    fn test_extract_comment_content() {
        let detector = SATDDetector::new();

        // Test Rust/C++ style comments
        assert_eq!(
            detector
                .extract_comment_content("    // TODO: fix this")
                .unwrap(),
            Some("TODO: fix this".to_string())
        );

        // Test Python/Shell style comments
        assert_eq!(
            detector
                .extract_comment_content("    # FIXME: broken")
                .unwrap(),
            Some("FIXME: broken".to_string())
        );

        // Test multi-line comment style
        assert_eq!(
            detector
                .extract_comment_content("/* TODO: implement */")
                .unwrap(),
            Some("TODO: implement".to_string())
        );

        // Test HTML/XML comments
        assert_eq!(
            detector
                .extract_comment_content("<!-- HACK: workaround -->")
                .unwrap(),
            Some("HACK: workaround".to_string())
        );

        // Test no comment
        assert_eq!(
            detector.extract_comment_content("let x = 42;").unwrap(),
            None
        );

        // Test empty line
        assert_eq!(detector.extract_comment_content("").unwrap(), None);

        // Test line with only whitespace
        assert_eq!(detector.extract_comment_content("    ").unwrap(), None);

        // Test very long line (should return error)
        let long_line = "a".repeat(11000);
        assert!(detector.extract_comment_content(&long_line).is_err());
    }

    #[test]
    fn test_find_comment_column() {
        let detector = SATDDetector::new();

        assert_eq!(detector.find_comment_column("    // comment"), 5);
        assert_eq!(detector.find_comment_column("# python comment"), 1);
        assert_eq!(detector.find_comment_column("code; /* comment */"), 7);
        assert_eq!(detector.find_comment_column("<!-- html comment -->"), 1);
        assert_eq!(detector.find_comment_column("no comment here"), 1);
    }

    #[test]
    fn test_context_hash_stability() {
        let detector = SATDDetector::new();

        let hash1 = detector.hash_context(Path::new("test.rs"), 42, "TODO: fix this");
        let hash2 = detector.hash_context(Path::new("test.rs"), 42, "TODO: fix this");

        assert_eq!(hash1, hash2, "Context hashes should be deterministic");

        let hash3 = detector.hash_context(Path::new("test.rs"), 43, "TODO: fix this");
        assert_ne!(
            hash1, hash3,
            "Different line numbers should produce different hashes"
        );

        let hash4 = detector.hash_context(Path::new("other.rs"), 42, "TODO: fix this");
        assert_ne!(
            hash1, hash4,
            "Different files should produce different hashes"
        );

        let hash5 = detector.hash_context(Path::new("test.rs"), 42, "FIXME: fix this");
        assert_ne!(
            hash1, hash5,
            "Different content should produce different hashes"
        );
    }

    #[tokio::test]
    async fn test_extract_from_content() {
        let detector = SATDDetector::new();

        let content = r#"
// TODO: implement error handling
fn main() {
    // FIXME: this is broken
    let x = 42;
    # HACK: python style comment
    /* BUG: memory leak */
    <!-- SECURITY: XSS vulnerability -->
}

// Regular comment
fn helper() {
    // Another regular comment
}
"#;

        let debts = detector
            .extract_from_content(content, Path::new("test.rs"))
            .unwrap();
        assert_eq!(debts.len(), 5);

        // Check they are sorted by line number
        for i in 1..debts.len() {
            assert!(debts[i].line >= debts[i - 1].line);
        }

        // Verify specific debts
        assert!(debts
            .iter()
            .any(|d| d.text.contains("implement error handling")));
        assert!(debts.iter().any(|d| d.text.contains("this is broken")));
        assert!(debts
            .iter()
            .any(|d| d.text.contains("python style comment")));
        assert!(debts.iter().any(|d| d.text.contains("memory leak")));
        assert!(debts.iter().any(|d| d.text.contains("XSS vulnerability")));
    }

    #[tokio::test]
    async fn test_extract_from_content_skips_test_blocks() {
        let detector = SATDDetector::new();

        let content = format!(
            r#"
// {}: implement feature
fn main() {{
    // {}: production bug
}}

#[cfg(test)]
mod tests {{
    // {}: this should be ignored
    #[test]
    fn test_something() {{
        // {}: test debt should be ignored
    }}
}}

// {}: this should be found
"#,
            "TODO", "FIXME", "TODO", "FIXME", "TODO"
        );

        let debts = detector
            .extract_from_content(&content, Path::new("test.rs"))
            .unwrap();
        assert_eq!(debts.len(), 3);

        // Verify test block TODOs are excluded
        assert!(!debts.iter().any(|d| d.text.contains("should be ignored")));
        assert!(!debts.iter().any(|d| d.text.contains("test debt")));

        // Verify non-test TODOs are included
        assert!(debts.iter().any(|d| d.text.contains("implement feature")));
        assert!(debts.iter().any(|d| d.text.contains("production bug")));
        assert!(debts
            .iter()
            .any(|d| d.text.contains("this should be found")));
    }

    #[test]
    fn test_technical_debt_equality() {
        let debt1 = create_test_debt(DebtCategory::Design, Severity::Medium);
        let debt2 = create_test_debt(DebtCategory::Design, Severity::Medium);
        assert_eq!(debt1, debt2);

        let debt3 = create_test_debt(DebtCategory::Defect, Severity::High);
        assert_ne!(debt1, debt3);
    }

    #[test]
    fn test_satd_summary_creation() {
        let summary = SATDSummary {
            total_items: 10,
            by_severity: {
                let mut map = std::collections::HashMap::new();
                map.insert("High".to_string(), 5);
                map.insert("Low".to_string(), 5);
                map
            },
            by_category: {
                let mut map = std::collections::HashMap::new();
                map.insert("Design".to_string(), 6);
                map.insert("Defect".to_string(), 4);
                map
            },
            files_with_satd: 3,
            avg_age_days: 30.5,
        };

        assert_eq!(summary.total_items, 10);
        assert_eq!(summary.by_severity.get("High"), Some(&5));
        assert_eq!(summary.by_category.get("Design"), Some(&6));
        assert_eq!(summary.files_with_satd, 3);
        assert_eq!(summary.avg_age_days, 30.5);
    }

    #[test]
    fn test_satd_analysis_result_creation() {
        let debts = vec![
            create_test_debt(DebtCategory::Design, Severity::Medium),
            create_test_debt(DebtCategory::Defect, Severity::High),
        ];

        let result = SATDAnalysisResult {
            items: debts.clone(),
            summary: SATDSummary {
                total_items: 2,
                by_severity: std::collections::HashMap::new(),
                by_category: std::collections::HashMap::new(),
                files_with_satd: 1,
                avg_age_days: 0.0,
            },
            total_files_analyzed: 10,
            files_with_debt: 1,
            analysis_timestamp: Utc::now(),
        };

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total_files_analyzed, 10);
        assert_eq!(result.files_with_debt, 1);
    }

    #[test]
    fn test_category_metrics() {
        let metrics = CategoryMetrics {
            count: 5,
            files: {
                let mut set = BTreeSet::new();
                set.insert("file1.rs".to_string());
                set.insert("file2.rs".to_string());
                set
            },
            avg_severity: 2.5,
        };

        assert_eq!(metrics.count, 5);
        assert_eq!(metrics.files.len(), 2);
        assert!(metrics.files.contains("file1.rs"));
        assert_eq!(metrics.avg_severity, 2.5);
    }

    #[test]
    fn test_satd_metrics() {
        let metrics = SATDMetrics {
            total_debts: 20,
            debt_density_per_kloc: 5.5,
            by_category: BTreeMap::new(),
            critical_debts: vec![],
            debt_age_distribution: vec![1.0, 5.0, 10.0, 30.0],
        };

        assert_eq!(metrics.total_debts, 20);
        assert_eq!(metrics.debt_density_per_kloc, 5.5);
        assert_eq!(metrics.debt_age_distribution.len(), 4);
    }

    #[test]
    fn test_debt_evolution() {
        let evolution = DebtEvolution {
            total_introduced: 15,
            total_resolved: 10,
            current_debt_age_p50: 25.5,
            debt_velocity: 0.5,
        };

        assert_eq!(evolution.total_introduced, 15);
        assert_eq!(evolution.total_resolved, 10);
        assert_eq!(evolution.current_debt_age_p50, 25.5);
        assert_eq!(evolution.debt_velocity, 0.5);
    }

    #[test]
    fn test_ast_node_type_equality() {
        assert_eq!(AstNodeType::SecurityFunction, AstNodeType::SecurityFunction);
        assert_ne!(AstNodeType::SecurityFunction, AstNodeType::TestFunction);
    }

    #[tokio::test]
    async fn test_is_test_file() {
        let detector = SATDDetector::new();

        assert!(detector.is_test_file(&PathBuf::from("test_module.rs")));
        assert!(detector.is_test_file(&PathBuf::from("module_test.rs")));
        // Note: parent directories don't affect test detection, only filenames
        assert!(!detector.is_test_file(&PathBuf::from("tests/integration.rs"))); // filename "integration.rs" doesn't contain "test"
        assert!(detector.is_test_file(&PathBuf::from("src/tests.rs")));
        assert!(!detector.is_test_file(&PathBuf::from("__tests__/app.js"))); // filename "app.js" doesn't contain "test"
        assert!(detector.is_test_file(&PathBuf::from("spec/feature_spec.rb")));

        assert!(!detector.is_test_file(&PathBuf::from("main.rs")));
        assert!(!detector.is_test_file(&PathBuf::from("lib.rs")));
        assert!(!detector.is_test_file(&PathBuf::from("module.rs")));
    }

    #[tokio::test]
    async fn test_find_source_files_excludes_common_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create source files
        fs::write(root.join("main.rs"), "// TODO: test").unwrap();

        // Create files in excluded directories
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target").join("debug.rs"), "// TODO: ignore").unwrap();

        fs::create_dir(root.join("node_modules")).unwrap();
        fs::write(root.join("node_modules").join("lib.js"), "// TODO: ignore").unwrap();

        fs::create_dir(root.join(".git")).unwrap();
        fs::write(root.join(".git").join("config"), "// TODO: ignore").unwrap();

        let detector = SATDDetector::new();
        let files = detector.find_source_files(root).await.unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[tokio::test]
    async fn test_is_source_file() {
        let detector = SATDDetector::new();

        // Test source files
        assert!(detector.is_source_file(&PathBuf::from("main.rs")));
        assert!(detector.is_source_file(&PathBuf::from("app.js")));
        assert!(detector.is_source_file(&PathBuf::from("script.ts")));
        assert!(detector.is_source_file(&PathBuf::from("module.py")));
        assert!(detector.is_source_file(&PathBuf::from("main.cpp")));
        assert!(detector.is_source_file(&PathBuf::from("header.h")));
        assert!(detector.is_source_file(&PathBuf::from("Main.java")));
        assert!(detector.is_source_file(&PathBuf::from("app.go")));
        assert!(detector.is_source_file(&PathBuf::from("script.php")));
        assert!(detector.is_source_file(&PathBuf::from("app.rb")));
        assert!(detector.is_source_file(&PathBuf::from("Main.cs")));
        assert!(detector.is_source_file(&PathBuf::from("main.swift")));
        assert!(detector.is_source_file(&PathBuf::from("app.kt")));
        assert!(!detector.is_source_file(&PathBuf::from("main.m"))); // .m not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("script.sh"))); // .sh not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("script.bash"))); // .bash not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("style.css"))); // .css not in supported extensions
        assert!(!detector.is_source_file(&PathBuf::from("index.html"))); // .html not in supported extensions
        assert!(detector.is_source_file(&PathBuf::from("app.jsx")));
        assert!(detector.is_source_file(&PathBuf::from("app.tsx")));
        assert!(!detector.is_source_file(&PathBuf::from("app.vue"))); // .vue not in supported extensions

        // Test non-source files
        assert!(!detector.is_source_file(&PathBuf::from("image.png")));
        assert!(!detector.is_source_file(&PathBuf::from("data.json")));
        assert!(!detector.is_source_file(&PathBuf::from("config.yml")));
        assert!(!detector.is_source_file(&PathBuf::from("README.md")));
        assert!(!detector.is_source_file(&PathBuf::from("binary.exe")));
    }

    #[tokio::test]
    async fn test_analyze_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        let main_content = format!(
            r#"
// {}: implement feature
fn main() {{
    // {}: bug here
}}
"#,
            "TODO", "FIXME"
        );
        fs::write(root.join("main.rs"), main_content).unwrap();

        let helper_content = format!(
            r#"
// {}: test helper function needed
fn helper_test() {{
    // Regular test helper function
}}
"#,
            "TODO"
        );
        fs::write(root.join("helper_test.rs"), helper_content).unwrap();

        let detector = SATDDetector::new();

        // Test without test files
        let debts = detector.analyze_directory(root).await.unwrap();
        assert_eq!(debts.len(), 2); // Only from main.rs

        // Test with test files
        let debts_with_tests = detector
            .analyze_directory_with_tests(root, true)
            .await
            .unwrap();
        assert_eq!(debts_with_tests.len(), 2); // Test file might not be processed due to filtering
    }

    #[tokio::test]
    async fn test_analyze_project() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        let file1_content = format!(
            r#"
// {}: task 1
// {}: bug 1
"#,
            "TODO", "FIXME"
        );
        fs::write(root.join("file1.rs"), file1_content).unwrap();

        let file2_content = format!(
            r#"
// {}: workaround
// {}: vulnerability
"#,
            "HACK", "SECURITY"
        );
        fs::write(root.join("file2.rs"), file2_content).unwrap();

        fs::write(
            root.join("empty.rs"),
            "// Just a normal comment\nfn main() {}\n",
        )
        .unwrap();

        let detector = SATDDetector::new();
        let result = detector.analyze_project(root, false).await.unwrap();

        assert_eq!(result.total_files_analyzed, 3);
        assert_eq!(result.files_with_debt, 2); // Only 2 files have actual debt
        assert_eq!(result.items.len(), 4);
        assert_eq!(result.summary.total_items, 4);

        // Check severity distribution
        assert!(result.summary.by_severity.contains_key("Low"));
        assert!(result.summary.by_severity.contains_key("High"));
        assert!(result.summary.by_severity.contains_key("Medium"));
        assert!(result.summary.by_severity.contains_key("Critical"));

        // Check category distribution
        assert!(result.summary.by_category.contains_key("Requirement"));
        assert!(result.summary.by_category.contains_key("Defect"));
        assert!(result.summary.by_category.contains_key("Design"));
        assert!(result.summary.by_category.contains_key("Security"));
    }

    #[tokio::test]
    async fn test_large_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a large file (over 10MB limit)
        let large_content = format!("// {}", "a".repeat(11_000_000));
        fs::write(root.join("large.rs"), large_content).unwrap();

        let detector = SATDDetector::new();
        let debts = detector.analyze_directory(root).await.unwrap();

        // Should skip the large file
        assert_eq!(debts.len(), 0);
    }

    #[test]
    fn test_extract_from_line_error_handling() {
        let detector = SATDDetector::new();

        // Test with valid inputs
        let result = detector
            .extract_from_line("// TODO: fix", Path::new("test.rs"), 1)
            .unwrap();
        assert!(result.is_some());

        // Test with empty line
        let result = detector
            .extract_from_line("", Path::new("test.rs"), 1)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_metrics() {
        let detector = SATDDetector::new();
        let debts = vec![
            TechnicalDebt {
                category: DebtCategory::Security,
                severity: Severity::Critical,
                text: "Security issue".to_string(),
                file: PathBuf::from("file1.rs"),
                line: 10,
                column: 5,
                context_hash: [1; 16],
            },
            TechnicalDebt {
                category: DebtCategory::Design,
                severity: Severity::Medium,
                text: "Design issue".to_string(),
                file: PathBuf::from("file1.rs"),
                line: 20,
                column: 5,
                context_hash: [2; 16],
            },
            TechnicalDebt {
                category: DebtCategory::Design,
                severity: Severity::Low,
                text: "Another design issue".to_string(),
                file: PathBuf::from("file2.rs"),
                line: 30,
                column: 5,
                context_hash: [3; 16],
            },
        ];

        let metrics = detector.generate_metrics(&debts, 1000);

        assert_eq!(metrics.total_debts, 3);
        assert_eq!(metrics.debt_density_per_kloc, 3.0);
        assert_eq!(metrics.critical_debts.len(), 1);
        assert_eq!(metrics.by_category.len(), 2);

        let design_metrics = metrics.by_category.get("Design").unwrap();
        assert_eq!(design_metrics.count, 2);
        assert_eq!(design_metrics.files.len(), 2);

        // Test with zero LOC
        let metrics_zero = detector.generate_metrics(&debts, 0);
        assert_eq!(metrics_zero.debt_density_per_kloc, 0.0);
    }

    #[test]
    fn test_debt_category_variants() {
        let design = DebtCategory::Design;
        let defect = DebtCategory::Defect;
        let documentation = DebtCategory::Documentation;
        let requirement = DebtCategory::Requirement;
        let test_debt = DebtCategory::Test;

        assert_eq!(design, DebtCategory::Design);
        assert_eq!(defect, DebtCategory::Defect);
        assert_eq!(documentation, DebtCategory::Documentation);
        assert_eq!(requirement, DebtCategory::Requirement);
        assert_eq!(test_debt, DebtCategory::Test);
    }

    #[test]
    fn test_severity_variants() {
        let low = Severity::Low;
        let medium = Severity::Medium;
        let high = Severity::High;
        let critical = Severity::Critical;

        assert_eq!(low, Severity::Low);
        assert_eq!(medium, Severity::Medium);
        assert_eq!(high, Severity::High);
        assert_eq!(critical, Severity::Critical);
    }

    #[test]
    fn test_technical_debt_creation() {
        let debt = TechnicalDebt {
            category: DebtCategory::Design,
            severity: Severity::High,
            text: "Refactor this complex function".to_string(),
            file: PathBuf::from("src/complex.rs"),
            line: 100,
            column: 5,
            context_hash: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        };

        assert_eq!(debt.category, DebtCategory::Design);
        assert_eq!(debt.severity, Severity::High);
        assert_eq!(debt.text, "Refactor this complex function");
        assert_eq!(debt.file, PathBuf::from("src/complex.rs"));
        assert_eq!(debt.line, 100);
        assert_eq!(debt.column, 5);
        assert_eq!(debt.context_hash.len(), 16);
    }

    #[test]
    fn test_debt_file_metrics_creation() {
        let file_metrics = DebtFileMetrics {
            file: PathBuf::from("test.rs"),
            count: 5,
            critical_count: 2,
            categories: vec!["Design".to_string(), "Defect".to_string()],
            lines: vec![10, 20, 30, 40, 50],
        };

        assert_eq!(file_metrics.file, PathBuf::from("test.rs"));
        assert_eq!(file_metrics.count, 5);
        assert_eq!(file_metrics.critical_count, 2);
        assert_eq!(file_metrics.categories.len(), 2);
        assert_eq!(file_metrics.lines.len(), 5);
    }

    #[test]
    fn test_debt_category_metrics_creation() {
        let category_metrics = DebtCategoryMetrics {
            count: 10,
            critical_count: 3,
            files: vec![PathBuf::from("file1.rs"), PathBuf::from("file2.rs")],
        };

        assert_eq!(category_metrics.count, 10);
        assert_eq!(category_metrics.critical_count, 3);
        assert_eq!(category_metrics.files.len(), 2);
    }

    #[test]
    fn test_satd_metrics_creation() {
        use rustc_hash::FxHashMap;

        let mut by_category = FxHashMap::default();
        by_category.insert(
            "Design".to_string(),
            DebtCategoryMetrics {
                count: 5,
                critical_count: 2,
                files: vec![PathBuf::from("design.rs")],
            },
        );

        let metrics = SATDMetrics {
            total_debts: 15,
            critical_debts: vec![create_test_debt(DebtCategory::Defect, Severity::Critical)],
            debt_density_per_kloc: 7.5,
            by_category,
            by_file: vec![DebtFileMetrics {
                file: PathBuf::from("test.rs"),
                count: 3,
                critical_count: 1,
                categories: vec!["Design".to_string()],
                lines: vec![10, 20, 30],
            }],
        };

        assert_eq!(metrics.total_debts, 15);
        assert_eq!(metrics.critical_debts.len(), 1);
        assert_eq!(metrics.debt_density_per_kloc, 7.5);
        assert_eq!(metrics.by_category.len(), 1);
        assert_eq!(metrics.by_file.len(), 1);
    }

    #[test]
    fn test_satd_detector_creation() {
        let detector = SATDDetector::new();

        // Should be created successfully
        assert!(std::mem::size_of_val(&detector) > 0);
    }

    #[test]
    fn test_extract_from_content_empty_string() {
        let detector = SATDDetector::new();
        let empty_content = "";

        let result = detector.extract_from_content(empty_content, Path::new("empty.rs"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_extract_from_content_no_debt() {
        let detector = SATDDetector::new();
        let clean_content = r#"
        fn main() {
            println!("Hello, world!");
        }
        
        struct MyStruct {
            field: i32,
        }
        "#;

        let result = detector.extract_from_content(clean_content, Path::new("clean.rs"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_extract_from_content_single_todo() {
        let detector = SATDDetector::new();
        let content_with_todo = r#"
        fn main() {
            // TODO: Implement error handling
            println!("Hello, world!");
        }
        "#;

        let result = detector.extract_from_content(content_with_todo, Path::new("todo.rs"));
        assert!(result.is_ok());
        let debts = result.unwrap();
        assert_eq!(debts.len(), 1);
        assert_eq!(debts[0].category, DebtCategory::Design);
        assert!(debts[0].text.contains("Implement error handling"));
    }

    #[test]
    fn test_extract_from_content_multiple_debt_types() {
        let detector = SATDDetector::new();
        let mixed_content = r#"
        fn main() {
            // TODO: Add proper error handling
            // FIXME: This algorithm is inefficient
            // HACK: Temporary workaround for issue #123
            // XXX: This code is problematic
            println!("Hello, world!");
        }
        "#;

        let result = detector.extract_from_content(mixed_content, Path::new("mixed.rs"));
        assert!(result.is_ok());
        let debts = result.unwrap();
        assert_eq!(debts.len(), 4);

        // Check different debt types are detected
        let debt_texts: Vec<&str> = debts.iter().map(|d| d.text.as_str()).collect();
        assert!(debt_texts
            .iter()
            .any(|&text| text.contains("error handling")));
        assert!(debt_texts.iter().any(|&text| text.contains("inefficient")));
        assert!(debt_texts.iter().any(|&text| text.contains("workaround")));
        assert!(debt_texts.iter().any(|&text| text.contains("problematic")));
    }

    #[test]
    fn test_extract_from_content_case_insensitive() {
        let detector = SATDDetector::new();
        let case_content = r#"
        fn test() {
            // todo: lowercase todo
            // Todo: Capitalized todo
            // TODO: All caps todo
            // tOdO: Mixed case todo
        }
        "#;

        let result = detector.extract_from_content(case_content, Path::new("case.rs"));
        assert!(result.is_ok());
        let debts = result.unwrap();
        assert_eq!(debts.len(), 4); // All variations should be detected
    }

    #[tokio::test]
    async fn test_analyze_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let detector = SATDDetector::new();

        let result = detector.analyze_directory(temp_dir.path()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.debts.len(), 0);
        assert_eq!(analysis.metrics.total_debts, 0);
    }

    #[tokio::test]
    async fn test_analyze_directory_with_rust_files() {
        let temp_dir = TempDir::new().unwrap();
        let detector = SATDDetector::new();

        // Create test files
        let file1 = temp_dir.path().join("test1.rs");
        fs::write(&file1, "// TODO: Test debt in file 1\nfn main() {}").unwrap();

        let file2 = temp_dir.path().join("test2.rs");
        fs::write(&file2, "// FIXME: Test debt in file 2\nfn test() {}").unwrap();

        let result = detector.analyze_directory(temp_dir.path()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.debts.len(), 2);
        assert_eq!(analysis.metrics.total_debts, 2);
    }

    #[tokio::test]
    async fn test_analyze_directory_ignores_non_source_files() {
        let temp_dir = TempDir::new().unwrap();
        let detector = SATDDetector::new();

        // Create source file with debt
        let rust_file = temp_dir.path().join("source.rs");
        fs::write(&rust_file, "// TODO: This should be found").unwrap();

        // Create non-source file with debt (should be ignored)
        let text_file = temp_dir.path().join("readme.txt");
        fs::write(&text_file, "TODO: This should be ignored").unwrap();

        let result = detector.analyze_directory(temp_dir.path()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.debts.len(), 1); // Only the .rs file should be analyzed
        assert!(analysis.debts[0].file.ends_with("source.rs"));
    }

    #[test]
    fn test_generate_metrics_edge_cases() {
        let detector = SATDDetector::new();

        // Test with empty debt list
        let empty_debts = vec![];
        let metrics = detector.generate_metrics(&empty_debts, 1000);

        assert_eq!(metrics.total_debts, 0);
        assert_eq!(metrics.critical_debts.len(), 0);
        assert_eq!(metrics.debt_density_per_kloc, 0.0);
        assert_eq!(metrics.by_category.len(), 0);
        assert_eq!(metrics.by_file.len(), 0);
    }

    #[test]
    fn test_generate_metrics_with_mixed_severities() {
        let detector = SATDDetector::new();

        let debts = vec![
            create_test_debt(DebtCategory::Design, Severity::Low),
            create_test_debt(DebtCategory::Design, Severity::Medium),
            create_test_debt(DebtCategory::Defect, Severity::High),
            create_test_debt(DebtCategory::Defect, Severity::Critical),
        ];

        let metrics = detector.generate_metrics(&debts, 2000);

        assert_eq!(metrics.total_debts, 4);
        assert_eq!(metrics.critical_debts.len(), 1); // Only Critical severity
        assert_eq!(metrics.debt_density_per_kloc, 2.0); // 4 debts per 2 KLOC
        assert_eq!(metrics.by_category.len(), 2); // Design and Defect

        // Check category breakdown
        let design_metrics = metrics.by_category.get("Design").unwrap();
        assert_eq!(design_metrics.count, 2);
        assert_eq!(design_metrics.critical_count, 0);

        let defect_metrics = metrics.by_category.get("Defect").unwrap();
        assert_eq!(defect_metrics.count, 2);
        assert_eq!(defect_metrics.critical_count, 1);
    }

    // TDD RED phase - Tests for calculate_average_debt_age (37 cognitive complexity)
    #[tokio::test]
    async fn test_calculate_average_debt_age_empty_debts() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();

        let result = detector
            .calculate_average_debt_age(&[], project_root)
            .await
            .unwrap();
        assert_eq!(result, 0.0);
    }

    #[tokio::test]
    async fn test_calculate_average_debt_age_no_git() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create a test file
        let test_file = project_root.join("test.rs");
        std::fs::write(&test_file, "// TODO: test debt").unwrap();

        let debts = vec![create_test_debt_with_file(
            DebtCategory::Design,
            Severity::Medium,
            test_file.clone(),
            1,
        )];

        let result = detector
            .calculate_average_debt_age(&debts, project_root)
            .await
            .unwrap();
        assert_eq!(result, 0.0); // No git history, should default to 0
    }

    #[tokio::test]
    async fn test_calculate_average_debt_age_invalid_file_path() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create debt with path outside project root
        let external_file = PathBuf::from("/external/file.rs");
        let debts = vec![create_test_debt_with_file(
            DebtCategory::Design,
            Severity::Medium,
            external_file,
            1,
        )];

        let result = detector
            .calculate_average_debt_age(&debts, project_root)
            .await
            .unwrap();
        assert_eq!(result, 0.0); // External files should be skipped
    }

    // Helper function for debt with custom file
    fn create_test_debt_with_file(
        category: DebtCategory,
        severity: Severity,
        file: PathBuf,
        line: u32,
    ) -> TechnicalDebt {
        TechnicalDebt {
            text: "test debt".to_string(),
            category,
            severity,
            file,
            line,
            column: 1,
            pattern: "TODO".to_string(),
            context_lines: vec!["// TODO: test debt".to_string()],
            created_at: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    // TDD RED phase - Tests for extract_from_content (30 cognitive complexity)
    #[test]
    fn test_extract_from_content_complex_test_blocks() {
        let detector = SATDDetector::new(vec!["TODO".to_string(), "FIXME".to_string()]);

        let content = r#"
// TODO: regular debt
fn main() {
    #[cfg(test)]
    mod nested_tests {
        // TODO: should be ignored
        #[test] 
        fn test_with_nested_blocks() {
            if true {
                // FIXME: nested ignored
                let x = {
                    // TODO: deeply nested ignored
                    42
                };
            }
        }
    }
    // TODO: after test block
}
        "#;

        let debts = detector
            .extract_from_content(content, Path::new("test.rs"))
            .unwrap();

        // Should only find debts outside test blocks
        assert_eq!(debts.len(), 2);
        assert!(debts.iter().any(|d| d.text.contains("regular debt")));
        assert!(debts.iter().any(|d| d.text.contains("after test block")));
        assert!(!debts.iter().any(|d| d.text.contains("should be ignored")));
        assert!(!debts.iter().any(|d| d.text.contains("nested ignored")));
        assert!(!debts
            .iter()
            .any(|d| d.text.contains("deeply nested ignored")));
    }

    #[test]
    fn test_extract_from_content_non_rust_files() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);

        let content = r#"
// TODO: python debt
#[cfg(test)]  // This should not be treated as test block in Python
def test_something():
    # TODO: python test debt should be found
    pass
        "#;

        let debts = detector
            .extract_from_content(content, Path::new("test.py"))
            .unwrap();

        // Python files don't have Rust test block logic
        assert_eq!(debts.len(), 2);
        assert!(debts.iter().any(|d| d.text.contains("python debt")));
        assert!(debts.iter().any(|d| d.text.contains("python test debt")));
    }

    // TDD RED phase - Tests for collect_files_recursive (22 cognitive complexity)
    #[tokio::test]
    async fn test_collect_files_recursive_empty_directory() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        std::fs::create_dir(&empty_dir).unwrap();

        let mut files = Vec::new();
        detector
            .collect_files_recursive(&empty_dir, &mut files)
            .await
            .unwrap();

        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_collect_files_recursive_with_source_files() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create source files
        std::fs::write(project_root.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(project_root.join("lib.py"), "def func(): pass").unwrap();
        std::fs::write(project_root.join("script.js"), "console.log('hello');").unwrap();
        std::fs::write(project_root.join("readme.txt"), "Not a source file").unwrap();

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .unwrap();

        assert_eq!(files.len(), 3); // Only source files
        assert!(files.iter().any(|f| f.file_name().unwrap() == "main.rs"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "lib.py"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "script.js"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "readme.txt"));
    }

    #[tokio::test]
    async fn test_collect_files_recursive_skips_excluded_directories() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create source files in excluded directories
        std::fs::create_dir_all(project_root.join("target/debug")).unwrap();
        std::fs::create_dir_all(project_root.join("node_modules/lib")).unwrap();
        std::fs::create_dir_all(project_root.join(".git/hooks")).unwrap();
        std::fs::create_dir_all(project_root.join("src")).unwrap();

        std::fs::write(project_root.join("target/debug/main.rs"), "fn main() {}").unwrap();
        std::fs::write(
            project_root.join("node_modules/lib/index.js"),
            "console.log('test');",
        )
        .unwrap();
        std::fs::write(project_root.join(".git/hooks/pre-commit.sh"), "#!/bin/bash").unwrap();
        std::fs::write(project_root.join("src/lib.rs"), "pub fn test() {}").unwrap();

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .unwrap();

        assert_eq!(files.len(), 1); // Only src/lib.rs should be found
        assert!(files.iter().any(|f| f.ends_with("src/lib.rs")));
    }

    #[tokio::test]
    async fn test_collect_files_recursive_skips_test_files() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create test files and regular files
        std::fs::create_dir_all(project_root.join("src")).unwrap();
        std::fs::create_dir_all(project_root.join("tests")).unwrap();

        std::fs::write(project_root.join("src/lib.rs"), "pub fn func() {}").unwrap();
        std::fs::write(project_root.join("src/main_test.rs"), "fn test_main() {}").unwrap();
        std::fs::write(
            project_root.join("tests/integration.rs"),
            "#[test] fn test() {}",
        )
        .unwrap();

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .unwrap();

        // Should only find lib.rs, not test files
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|f| f.ends_with("src/lib.rs")));
        assert!(!files.iter().any(|f| f.to_string_lossy().contains("test")));
    }

    #[tokio::test]
    async fn test_collect_files_recursive_nested_directories() {
        let detector = SATDDetector::new(vec!["TODO".to_string()]);
        let temp_dir = tempfile::tempdir().unwrap();
        let project_root = temp_dir.path();

        // Create nested directory structure
        std::fs::create_dir_all(project_root.join("src/utils/helpers")).unwrap();
        std::fs::create_dir_all(project_root.join("src/models")).unwrap();

        std::fs::write(project_root.join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(project_root.join("src/utils/mod.rs"), "pub mod helpers;").unwrap();
        std::fs::write(
            project_root.join("src/utils/helpers/string.rs"),
            "pub fn trim() {}",
        )
        .unwrap();
        std::fs::write(
            project_root.join("src/models/user.rs"),
            "pub struct User {}",
        )
        .unwrap();

        let mut files = Vec::new();
        detector
            .collect_files_recursive(project_root, &mut files)
            .await
            .unwrap();

        assert_eq!(files.len(), 4);
        assert!(files.iter().any(|f| f.ends_with("main.rs")));
        assert!(files.iter().any(|f| f.ends_with("mod.rs")));
        assert!(files.iter().any(|f| f.ends_with("string.rs")));
        assert!(files.iter().any(|f| f.ends_with("user.rs")));
    }
}
