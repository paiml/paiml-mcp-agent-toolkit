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
    pub total_files_analyzed: usize,
    pub files_with_debt: usize,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Critical, // Security vulnerabilities, data loss risks
    High,     // Defects, broken functionality
    Medium,   // Design issues, performance problems
    Low,      // TODOs, minor enhancements
}

impl Severity {
    /// Escalate severity by one level
    pub fn escalate(self) -> Self {
        match self {
            Severity::Low => Severity::Medium,
            Severity::Medium => Severity::High,
            Severity::High => Severity::Critical,
            Severity::Critical => Severity::Critical,
        }
    }

    /// Reduce severity by one level
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
                regex: r"(?i)\b(security|vuln|cve)\b".to_string(),
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

        let regex_strings: Vec<String> = patterns.iter().map(|p| p.regex.clone()).collect();
        let compiled_patterns =
            RegexSet::new(&regex_strings).expect("Failed to compile SATD patterns");

        Self {
            patterns,
            compiled_patterns,
        }
    }

    /// Classify a comment text and return debt information
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
        let debt_classifier = DebtClassifier::new();
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

        for (line_num, line) in content.lines().enumerate() {
            if let Some(debt) = self.extract_from_line(line, file_path, line_num as u32 + 1)? {
                debts.push(debt);
            }
        }

        // Sort by file, line, column for deterministic output
        debts.sort_by_key(|d| (d.file.clone(), d.line, d.column));

        Ok(debts)
    }

    /// Extract debt from a single line
    fn extract_from_line(
        &self,
        line: &str,
        file_path: &Path,
        line_num: u32,
    ) -> Result<Option<TechnicalDebt>, TemplateError> {
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
    pub async fn analyze_project(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<SATDAnalysisResult, TemplateError> {
        let mut all_debts = Vec::new();

        let files = self.find_source_files(root).await?;
        let mut files_with_debt = 0;
        let mut total_files_analyzed = 0;

        for file_path in files {
            // Skip test files if not requested
            if !include_tests && self.is_test_file(&file_path) {
                continue;
            }

            total_files_analyzed += 1;

            match tokio::fs::read_to_string(&file_path).await {
                Ok(content) => {
                    // Validate file size before processing
                    if content.len() > 10_000_000 {
                        eprintln!(
                            "Warning: Skipping large file {}: {} bytes",
                            file_path.display(),
                            content.len()
                        );
                        continue;
                    }
                    
                    match self.extract_from_content(&content, &file_path) {
                        Ok(debts) => {
                            if !debts.is_empty() {
                                files_with_debt += 1;
                            }
                            all_debts.extend(debts);
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

        Ok(SATDAnalysisResult {
            items: all_debts,
            total_files_analyzed,
            files_with_debt,
            analysis_timestamp: chrono::Utc::now(),
        })
    }

    /// Analyze debt in a directory recursively
    pub async fn analyze_directory(
        &self,
        root: &Path,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        let mut all_debts = Vec::new();

        let files = self.find_source_files(root).await?;

        for file_path in files {
            match tokio::fs::read_to_string(&file_path).await {
                Ok(content) => {
                    // Validate file size before processing
                    if content.len() > 10_000_000 {
                        eprintln!(
                            "Warning: Skipping large file {}: {} bytes",
                            file_path.display(),
                            content.len()
                        );
                        continue;
                    }
                    
                    match self.extract_from_content(&content, &file_path) {
                        Ok(debts) => {
                            all_debts.extend(debts);
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

        Ok(all_debts)
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), TemplateError>> + 'a>> {
        Box::pin(async move {
            if !dir.is_dir() {
                return Ok(());
            }

            let mut entries = tokio::fs::read_dir(dir).await.map_err(TemplateError::Io)?;

            while let Some(entry) = entries.next_entry().await.map_err(TemplateError::Io)? {
                let path = entry.path();

                if path.is_dir() {
                    // Skip hidden directories and common non-source directories
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.')
                            || ["target", "node_modules", "dist", "build", "__pycache__"]
                                .contains(&name)
                        {
                            continue;
                        }
                    }
                    self.collect_files_recursive(&path, files).await?;
                } else if self.is_source_file(&path) {
                    files.push(path);
                }
            }

            Ok(())
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_classification() {
        let classifier = DebtClassifier::new();

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
            classifier.classify_comment("// Just a regular comment"),
            None
        );
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
    }

    #[tokio::test]
    async fn test_extract_from_content() {
        let detector = SATDDetector::new();
        let content = r#"
// This is a regular comment
// TODO: implement this feature
fn some_function() {
    // FIXME: bug in the logic
    println!("Hello");
    // SECURITY: check input validation
}
"#;

        let debts = detector
            .extract_from_content(content, Path::new("test.rs"))
            .unwrap();

        assert_eq!(debts.len(), 3);

        // Check TODO
        assert_eq!(debts[0].category, DebtCategory::Requirement);
        assert_eq!(debts[0].severity, Severity::Low);
        assert_eq!(debts[0].line, 3);

        // Check FIXME
        assert_eq!(debts[1].category, DebtCategory::Defect);
        assert_eq!(debts[1].severity, Severity::High);
        assert_eq!(debts[1].line, 5);

        // Check SECURITY
        assert_eq!(debts[2].category, DebtCategory::Security);
        assert_eq!(debts[2].severity, Severity::Critical);
        assert_eq!(debts[2].line, 7);
    }

    #[test]
    fn test_comment_extraction() {
        let detector = SATDDetector::new();

        // Rust/C++ style
        assert_eq!(
            detector
                .extract_comment_content("    // TODO: fix this")
                .unwrap(),
            Some("TODO: fix this".to_string())
        );

        // Python style
        assert_eq!(
            detector.extract_comment_content("# FIXME: broken").unwrap(),
            Some("FIXME: broken".to_string())
        );

        // Multi-line
        assert_eq!(
            detector
                .extract_comment_content("/* TODO: refactor */")
                .unwrap(),
            Some("TODO: refactor".to_string())
        );

        // HTML style
        assert_eq!(
            detector
                .extract_comment_content("<!-- HACK: workaround -->")
                .unwrap(),
            Some("HACK: workaround".to_string())
        );

        // Not a comment
        assert_eq!(
            detector.extract_comment_content("let x = 5;").unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn test_directory_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        let rust_file = temp_path.join("lib.rs");
        fs::write(
            &rust_file,
            r#"
// TODO: add documentation
pub fn hello() {
    // FIXME: handle errors
    println!("Hello");
}
"#,
        )
        .unwrap();

        let python_file = temp_path.join("script.py");
        fs::write(
            &python_file,
            r#"
# TODO: optimize performance
def greet():
    # SECURITY: validate input
    print("Hello")
"#,
        )
        .unwrap();

        let detector = SATDDetector::new();
        let debts = detector.analyze_directory(temp_path).await.unwrap();

        assert_eq!(debts.len(), 4);

        // Should find debts from both files
        let rust_debts: Vec<_> = debts
            .iter()
            .filter(|d| d.file.extension() == Some(std::ffi::OsStr::new("rs")))
            .collect();
        let python_debts: Vec<_> = debts
            .iter()
            .filter(|d| d.file.extension() == Some(std::ffi::OsStr::new("py")))
            .collect();

        assert_eq!(rust_debts.len(), 2);
        assert_eq!(python_debts.len(), 2);
    }

    #[test]
    fn test_severity_adjustment() {
        let classifier = DebtClassifier::new();

        let security_context = AstContext {
            node_type: AstNodeType::SecurityFunction,
            parent_function: "validate_input".to_string(),
            complexity: 5,
            siblings_count: 3,
            nesting_depth: 2,
            surrounding_statements: vec![],
        };

        let test_context = AstContext {
            node_type: AstNodeType::TestFunction,
            parent_function: "test_something".to_string(),
            complexity: 2,
            siblings_count: 1,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };

        // Security context should escalate severity
        assert_eq!(
            classifier.adjust_severity(Severity::Medium, &security_context),
            Severity::High
        );

        // Test context should reduce severity
        assert_eq!(
            classifier.adjust_severity(Severity::High, &test_context),
            Severity::Medium
        );
    }

    #[test]
    fn test_metrics_generation() {
        let detector = SATDDetector::new();

        let debts = vec![
            TechnicalDebt {
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                text: "TODO: implement".to_string(),
                file: PathBuf::from("file1.rs"),
                line: 1,
                column: 1,
                context_hash: [0; 16],
            },
            TechnicalDebt {
                category: DebtCategory::Security,
                severity: Severity::Critical,
                text: "SECURITY: fix vulnerability".to_string(),
                file: PathBuf::from("file2.rs"),
                line: 10,
                column: 5,
                context_hash: [1; 16],
            },
            TechnicalDebt {
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                text: "TODO: add tests".to_string(),
                file: PathBuf::from("file1.rs"),
                line: 20,
                column: 1,
                context_hash: [2; 16],
            },
        ];

        let metrics = detector.generate_metrics(&debts, 1000);

        assert_eq!(metrics.total_debts, 3);
        assert_eq!(metrics.debt_density_per_kloc, 3.0);
        assert_eq!(metrics.critical_debts.len(), 1);
        assert_eq!(metrics.by_category.len(), 2);

        // Check category metrics
        let req_metrics = &metrics.by_category["Requirement"];
        assert_eq!(req_metrics.count, 2);
        assert_eq!(req_metrics.files.len(), 1); // Both in file1.rs

        let sec_metrics = &metrics.by_category["Security"];
        assert_eq!(sec_metrics.count, 1);
        assert_eq!(sec_metrics.files.len(), 1);
    }
}
