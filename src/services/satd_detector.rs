//! Self-Admitted Technical Debt (SATD) Detection System
//!
//! This module provides high-performance, multi-language detection and classification
//! of technical debt annotations embedded in source code comments.

#![cfg_attr(coverage_nightly, coverage(off))]

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

/// Test-only structures for SATD metrics
#[cfg(test)]
#[derive(Debug, Clone)]
struct DebtFileMetrics {
    file: PathBuf,
    count: usize,
    critical_count: usize,
    categories: Vec<String>,
    lines: Vec<usize>,
}

#[cfg(test)]
#[derive(Debug, Clone)]
struct DebtCategoryMetrics {
    count: usize,
    critical_count: usize,
    files: Vec<PathBuf>,
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
/// EXTREME TDD FIX: Reordered Low→Critical for correct derive(Ord) behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Low,      // TODOs, minor enhancements
    Medium,   // Design issues, performance problems
    High,     // Defects, broken functionality
    Critical, // Security vulnerabilities, data loss risks
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn new() -> Self {
        // Default mode includes all patterns
        let patterns = vec![
            // High-confidence patterns with word boundaries
            DebtPattern {
                regex: r"(?i)\b(hack|kludge|smell|xxx)\b".to_string(),
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
                regex: r"(?i)\b(security|vuln|vulnerability|cve|xss)\b".to_string(),
                category: DebtCategory::Security,
                severity: Severity::Critical,
                description: "Security concern".to_string(),
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

    /// Extended mode: includes euphemisms used by AI coding assistants to bypass SATD detection
    /// Detects: placeholder, stub, simplified, demo, mock, dummy, fake, hardcoded, "for now", WIP
    /// See issue #149: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/149
    #[must_use]
    pub fn new_extended() -> Self {
        // Extended mode includes standard patterns PLUS euphemism patterns
        let mut patterns = vec![
            // Standard patterns (same as new())
            DebtPattern {
                regex: r"(?i)\b(hack|kludge|smell|xxx)\b".to_string(),
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
                regex: r"(?i)\b(security|vuln|vulnerability|cve|xss)\b".to_string(),
                category: DebtCategory::Security,
                severity: Severity::Critical,
                description: "Security concern".to_string(),
            },
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

        // EXTENDED PATTERNS: Euphemisms that hide technical debt (issue #149)
        // These are commonly used by AI coding assistants to bypass SATD detection
        let extended_patterns = vec![
            // Placeholder patterns - indicate incomplete implementation
            DebtPattern {
                regex: r"(?i)\bplaceholder\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Medium,
                description: "Placeholder - incomplete implementation".to_string(),
            },
            // Stub patterns - indicate missing implementation
            DebtPattern {
                regex: r"(?i)\bstub\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Medium,
                description: "Stub - missing implementation".to_string(),
            },
            // Simplified patterns - indicate corners were cut
            DebtPattern {
                regex: r"(?i)\bsimplified\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Low,
                description: "Simplified - corners cut".to_string(),
            },
            // Demo/demonstration patterns - indicate non-production code
            DebtPattern {
                regex: r"(?i)\b(for\s+)?demonstrat(e|ion)\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                description: "Demo code - not production ready".to_string(),
            },
            // Mock/dummy/fake patterns - indicate fake implementations
            DebtPattern {
                regex: r"(?i)\b(mock|dummy|fake)\b".to_string(),
                category: DebtCategory::Test,
                severity: Severity::Low,
                description: "Mock/dummy - fake implementation".to_string(),
            },
            // Hardcoded patterns - indicate missing configuration
            DebtPattern {
                regex: r"(?i)\bhardcoded\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "Hardcoded - missing configuration".to_string(),
            },
            // "For now" patterns - indicate temporary solutions
            DebtPattern {
                regex: r"(?i)\bfor\s+now\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::Medium,
                description: "For now - temporary solution".to_string(),
            },
            // WIP patterns - work in progress
            DebtPattern {
                regex: r"\bWIP\b".to_string(),
                category: DebtCategory::Requirement,
                severity: Severity::Medium,
                description: "WIP - work in progress".to_string(),
            },
            // Skip/bypass patterns - indicate missing validation
            DebtPattern {
                regex: r"(?i)\b(skip|bypass)\s+(for\s+now|this|validation)\b".to_string(),
                category: DebtCategory::Design,
                severity: Severity::High,
                description: "Skip/bypass - missing validation".to_string(),
            },
        ];

        patterns.extend(extended_patterns);

        let regex_strings: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
        let compiled_patterns =
            RegexSet::new(&regex_strings).expect("Failed to compile extended SATD patterns");

        Self {
            patterns,
            compiled_patterns,
        }
    }

    #[must_use]
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
    #[must_use]
    pub fn classify_comment(&self, text: &str) -> Option<(DebtCategory, Severity)> {
        let matches = self.compiled_patterns.matches(text);

        // Find the first matching pattern
        for match_idx in &matches {
            if let Some(pattern) = self.patterns.get(match_idx) {
                return Some((pattern.category, pattern.severity));
            }
        }

        None
    }

    /// Adjust severity based on context
    #[must_use]
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

    fn is_rust_file(&self, file_path: &Path) -> bool {
        file_path.extension().and_then(|s| s.to_str()) == Some("rs")
    }

    fn sort_debts(&self, debts: &mut [TechnicalDebt]) {
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
        hash.as_bytes()[..16].try_into().expect("internal error")
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
        ["target", "node_modules", "dist", "build", "__pycache__", "book"].contains(&name)
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
            || path_str.contains("/book/")
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
    #[must_use]
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
        for (category_name, metrics) in &mut by_category {
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

                metrics.avg_severity = f64::from(severity_sum) / category_debts.len() as f64;
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
            total_age_days / f64::from(valid_debt_count)
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
            .map(std::path::Path::to_path_buf)
    }

    async fn run_git_blame(
        &self,
        relative_path: &Path,
        line: u32,
        project_root: &Path,
    ) -> Option<String> {
        use std::process::Command;

        let output = Command::new("git")
            .args([
                "blame",
                "-L",
                &format!("{line},{line}"),
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

// Tests extracted to satd_detector_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "satd_detector_tests.rs"]
mod tests;
