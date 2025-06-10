//! Big-O Complexity Analyzer - Phase 5 implementation
//!
//! Provides algorithmic complexity analysis for functions using
//! pattern matching and heuristic analysis.

use crate::models::complexity_bound::{BigOClass, ComplexityBound};
use crate::services::complexity_patterns::{ComplexityAnalysisResult, ComplexityPatternMatcher};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Big-O complexity analyzer service
pub struct BigOAnalyzer {
    #[allow(dead_code)]
    pattern_matcher: ComplexityPatternMatcher,
}

/// Analysis configuration
#[derive(Debug, Clone)]
pub struct BigOAnalysisConfig {
    pub project_path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub confidence_threshold: u8,
    pub analyze_space_complexity: bool,
}

/// Big-O analysis report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BigOAnalysisReport {
    pub analyzed_functions: usize,
    pub complexity_distribution: ComplexityDistribution,
    pub high_complexity_functions: Vec<FunctionComplexity>,
    pub pattern_matches: Vec<PatternMatch>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplexityDistribution {
    pub constant: usize,
    pub logarithmic: usize,
    pub linear: usize,
    pub linearithmic: usize,
    pub quadratic: usize,
    pub cubic: usize,
    pub exponential: usize,
    pub unknown: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionComplexity {
    pub file_path: PathBuf,
    pub function_name: String,
    pub line_number: usize,
    pub time_complexity: ComplexityBound,
    pub space_complexity: ComplexityBound,
    pub confidence: u8,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub occurrences: usize,
    pub typical_complexity: BigOClass,
}

impl BigOAnalyzer {
    /// Create new Big-O analyzer
    pub fn new() -> Self {
        Self {
            pattern_matcher: ComplexityPatternMatcher::new(),
        }
    }

    fn get_loop_keywords(language: &str) -> Vec<&'static str> {
        match language {
            "rust" => vec!["for", "while", "loop"],
            "javascript" | "typescript" => vec!["for", "while", "do"],
            "python" => vec!["for", "while"],
            _ => vec!["for", "while"],
        }
    }

    fn detect_recursive_call(line: &str, function_name: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains(function_name)
            && !trimmed.starts_with("fn")
            && !trimmed.starts_with("function")
    }

    fn detect_sorting_operation(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains(".sort(") || trimmed.contains("sort(")
    }

    fn detect_binary_search(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains("binary_search") || trimmed.contains("binarySearch")
    }

    fn calculate_loop_depth(lines: &[&str], loop_keywords: &[&str]) -> usize {
        let mut loop_depth = 0;
        let mut max_loop_depth = 0;

        for line in lines {
            let trimmed = line.trim();

            // Track loop depth
            for keyword in loop_keywords {
                if trimmed.starts_with(keyword) {
                    loop_depth += 1;
                    max_loop_depth = max_loop_depth.max(loop_depth);
                }
            }

            if trimmed.contains('}') && loop_depth > 0 {
                loop_depth -= 1;
            }
        }

        max_loop_depth
    }

    fn determine_time_complexity(max_loop_depth: usize, has_recursion: bool) -> ComplexityBound {
        if has_recursion && max_loop_depth == 0 {
            return ComplexityBound::unknown();
        }

        match max_loop_depth {
            0 => ComplexityBound::constant().with_confidence(90),
            1 => ComplexityBound::linear().with_confidence(80),
            2 => ComplexityBound::quadratic().with_confidence(75),
            3 => ComplexityBound::polynomial(3, 1).with_confidence(70),
            n => ComplexityBound::polynomial(n as u32, 1).with_confidence(60),
        }
    }

    fn detect_space_complexity(function_body: &str) -> (ComplexityBound, bool) {
        let space_indicators = [
            "Vec::new",
            "vec!",
            "HashMap::new",
            "HashSet::new",
            "BTreeMap::new",
            "[]",
        ];

        let has_allocation = space_indicators
            .iter()
            .any(|indicator| function_body.contains(indicator));

        if has_allocation {
            (ComplexityBound::linear().with_confidence(70), true)
        } else {
            (ComplexityBound::constant().with_confidence(90), false)
        }
    }

    /// Analyze project for algorithmic complexity
    pub async fn analyze(&self, config: BigOAnalysisConfig) -> Result<BigOAnalysisReport> {
        info!("🔍 Starting Big-O complexity analysis");
        info!("📂 Project path: {}", config.project_path.display());

        // Discover source files
        let source_files = self.discover_source_files(&config).await?;
        info!("📁 Found {} source files", source_files.len());

        // Analyze each file
        let mut all_functions = Vec::new();
        let mut pattern_counts = rustc_hash::FxHashMap::default();

        for file in &source_files {
            let functions = self.analyze_file(file, &config).await?;

            // Count pattern matches
            for func in &functions {
                for pattern in &func.notes {
                    if pattern.starts_with("Pattern: ") {
                        let pattern_name = pattern.trim_start_matches("Pattern: ");
                        *pattern_counts.entry(pattern_name.to_string()).or_insert(0) += 1;
                    }
                }
            }

            all_functions.extend(functions);
        }

        // Build report
        let report = self.build_report(all_functions, pattern_counts);

        info!("✅ Big-O analysis completed");
        info!("📊 Analyzed {} functions", report.analyzed_functions);

        Ok(report)
    }

    /// Discover source files based on patterns
    async fn discover_source_files(&self, config: &BigOAnalysisConfig) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let extensions = [
            "rs", "js", "ts", "jsx", "tsx", "py", "cpp", "c", "java", "go",
        ];
        let exclude_dirs = [
            "target",
            "node_modules",
            ".git",
            "build",
            "dist",
            "__pycache__",
        ];

        let mut files = Vec::new();

        for entry in WalkDir::new(&config.project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Check exclusions
            let should_exclude = path.components().any(|comp| {
                if let Some(name) = comp.as_os_str().to_str() {
                    exclude_dirs.contains(&name)
                } else {
                    false
                }
            });

            if should_exclude {
                continue;
            }

            // Check extensions
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    files.push(path.to_path_buf());
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Analyze single file for complexity
    async fn analyze_file(
        &self,
        file_path: &PathBuf,
        config: &BigOAnalysisConfig,
    ) -> Result<Vec<FunctionComplexity>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let mut functions = Vec::new();

        // Simple function detection
        let function_patterns = [
            (r"fn\s+(\w+)", "rust"),
            (r"function\s+(\w+)", "javascript"),
            (r"def\s+(\w+)", "python"),
            (r"func\s+(\w+)", "go"),
            (
                r"(public|private|protected)?\s*(static)?\s*\w+\s+(\w+)\s*\(",
                "java",
            ),
        ];

        for (pattern, lang) in &function_patterns {
            let regex = regex::Regex::new(pattern)?;
            for cap in regex.captures_iter(&content) {
                if let Some(name_match) = cap.get(cap.len() - 1) {
                    let function_name = name_match.as_str().to_string();
                    let line_number = content[..name_match.start()].lines().count();

                    // Analyze function complexity
                    let complexity = self.analyze_function_complexity(
                        &function_name,
                        &content[name_match.start()..],
                        lang,
                    );

                    if complexity.confidence >= config.confidence_threshold {
                        functions.push(FunctionComplexity {
                            file_path: file_path.clone(),
                            function_name,
                            line_number,
                            time_complexity: complexity.time_complexity,
                            space_complexity: complexity.space_complexity,
                            confidence: complexity.confidence,
                            notes: complexity.notes,
                        });
                    }
                }
            }
        }

        Ok(functions)
    }

    /// Analyze function complexity using patterns and heuristics
    fn analyze_function_complexity(
        &self,
        function_name: &str,
        function_body: &str,
        language: &str,
    ) -> ComplexityAnalysisResult {
        let mut notes = Vec::new();
        let lines: Vec<&str> = function_body.lines().take(100).collect();

        // Get language-specific loop keywords
        let loop_keywords = Self::get_loop_keywords(language);

        // Calculate loop depth
        let max_loop_depth = Self::calculate_loop_depth(&lines, &loop_keywords);

        // Check for patterns
        let mut has_recursion = false;
        let mut has_sorting = false;

        for line in &lines {
            if Self::detect_recursive_call(line, function_name) {
                has_recursion = true;
                notes.push("Recursive function detected".to_string());
            }

            if Self::detect_sorting_operation(line) {
                has_sorting = true;
                notes.push("Pattern: Sorting operation".to_string());
            }

            if Self::detect_binary_search(line) {
                notes.push("Pattern: Binary search".to_string());
            }
        }

        // Determine time complexity
        let mut time_complexity = Self::determine_time_complexity(max_loop_depth, has_recursion);

        // Adjust for sorting operations
        if has_sorting
            && time_complexity
                .class
                .is_better_than(&BigOClass::Linearithmic)
        {
            time_complexity = ComplexityBound::linearithmic();
        }

        // Determine space complexity
        let (space_complexity, has_allocation) = Self::detect_space_complexity(function_body);
        if has_allocation {
            notes.push("Dynamic memory allocation detected".to_string());
        }

        ComplexityAnalysisResult {
            time_complexity,
            space_complexity,
            matched_patterns: Vec::new(),
            confidence: (time_complexity.confidence + space_complexity.confidence) / 2,
            notes,
        }
    }

    /// Build analysis report
    fn build_report(
        &self,
        functions: Vec<FunctionComplexity>,
        pattern_counts: rustc_hash::FxHashMap<String, usize>,
    ) -> BigOAnalysisReport {
        let mut distribution = ComplexityDistribution {
            constant: 0,
            logarithmic: 0,
            linear: 0,
            linearithmic: 0,
            quadratic: 0,
            cubic: 0,
            exponential: 0,
            unknown: 0,
        };

        let total_functions = functions.len();

        // Count complexity distribution
        for func in &functions {
            match func.time_complexity.class {
                BigOClass::Constant => distribution.constant += 1,
                BigOClass::Logarithmic => distribution.logarithmic += 1,
                BigOClass::Linear => distribution.linear += 1,
                BigOClass::Linearithmic => distribution.linearithmic += 1,
                BigOClass::Quadratic => distribution.quadratic += 1,
                BigOClass::Cubic => distribution.cubic += 1,
                BigOClass::Exponential => distribution.exponential += 1,
                BigOClass::Factorial => distribution.exponential += 1,
                BigOClass::Unknown => distribution.unknown += 1,
            }
        }

        // Find high complexity functions
        let mut high_complexity: Vec<_> = functions
            .into_iter()
            .filter(|f| {
                matches!(
                    f.time_complexity.class,
                    BigOClass::Quadratic
                        | BigOClass::Cubic
                        | BigOClass::Exponential
                        | BigOClass::Factorial
                )
            })
            .collect();

        high_complexity.sort_by_key(|f| f.time_complexity.class as u8);

        // Generate pattern matches
        let pattern_matches: Vec<_> = pattern_counts
            .into_iter()
            .map(|(name, count)| PatternMatch {
                pattern_name: name,
                occurrences: count,
                typical_complexity: BigOClass::Linear, // Default
            })
            .collect();

        // Generate recommendations
        let mut recommendations = Vec::new();

        if distribution.quadratic > 0 {
            recommendations.push(format!(
                "Found {} functions with O(n²) complexity. Consider optimization.",
                distribution.quadratic
            ));
        }

        if distribution.exponential > 0 {
            recommendations.push(format!(
                "⚠️ Found {} functions with exponential complexity! These need immediate attention.",
                distribution.exponential
            ));
        }

        if distribution.unknown > total_functions / 4 {
            recommendations.push(
                "Many functions have unknown complexity. Consider adding more explicit patterns."
                    .to_string(),
            );
        }

        BigOAnalysisReport {
            analyzed_functions: total_functions,
            complexity_distribution: distribution,
            high_complexity_functions: high_complexity,
            pattern_matches,
            recommendations,
        }
    }

    /// Format report as JSON
    pub fn format_as_json(&self, report: &BigOAnalysisReport) -> Result<String> {
        let json = serde_json::json!({
            "summary": {
                "analyzed_functions": report.analyzed_functions,
                "high_complexity_count": report.high_complexity_functions.len(),
            },
            "distribution": {
                "O(1)": report.complexity_distribution.constant,
                "O(log n)": report.complexity_distribution.logarithmic,
                "O(n)": report.complexity_distribution.linear,
                "O(n log n)": report.complexity_distribution.linearithmic,
                "O(n²)": report.complexity_distribution.quadratic,
                "O(n³)": report.complexity_distribution.cubic,
                "O(2^n)": report.complexity_distribution.exponential,
                "O(?)": report.complexity_distribution.unknown,
            },
            "high_complexity_functions": report.high_complexity_functions.iter().map(|f| {
                serde_json::json!({
                    "file": f.file_path.display().to_string(),
                    "function": f.function_name,
                    "line": f.line_number,
                    "time_complexity": f.time_complexity.notation(),
                    "space_complexity": f.space_complexity.notation(),
                    "confidence": f.confidence,
                })
            }).collect::<Vec<_>>(),
            "pattern_matches": report.pattern_matches.iter().map(|p| {
                serde_json::json!({
                    "pattern": p.pattern_name,
                    "occurrences": p.occurrences,
                })
            }).collect::<Vec<_>>(),
            "recommendations": report.recommendations,
        });

        Ok(serde_json::to_string_pretty(&json)?)
    }

    /// Format report as Markdown
    pub fn format_as_markdown(&self, report: &BigOAnalysisReport) -> String {
        let mut md = String::with_capacity(1024);

        md.push_str("# Big-O Complexity Analysis Report\n\n");

        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "- **Total Functions Analyzed**: {}\n",
            report.analyzed_functions
        ));
        md.push_str(&format!(
            "- **High Complexity Functions**: {}\n\n",
            report.high_complexity_functions.len()
        ));

        md.push_str("## Complexity Distribution\n\n");
        md.push_str("| Complexity | Count | Percentage |\n");
        md.push_str("|------------|-------|------------|\n");

        let total = report.analyzed_functions as f64;
        let dist = &report.complexity_distribution;

        md.push_str(&format!(
            "| O(1) | {} | {:.1}% |\n",
            dist.constant,
            (dist.constant as f64 / total) * 100.0
        ));
        md.push_str(&format!(
            "| O(log n) | {} | {:.1}% |\n",
            dist.logarithmic,
            (dist.logarithmic as f64 / total) * 100.0
        ));
        md.push_str(&format!(
            "| O(n) | {} | {:.1}% |\n",
            dist.linear,
            (dist.linear as f64 / total) * 100.0
        ));
        md.push_str(&format!(
            "| O(n log n) | {} | {:.1}% |\n",
            dist.linearithmic,
            (dist.linearithmic as f64 / total) * 100.0
        ));
        md.push_str(&format!(
            "| O(n²) | {} | {:.1}% |\n",
            dist.quadratic,
            (dist.quadratic as f64 / total) * 100.0
        ));
        md.push_str(&format!(
            "| O(n³) | {} | {:.1}% |\n",
            dist.cubic,
            (dist.cubic as f64 / total) * 100.0
        ));
        md.push_str(&format!(
            "| O(2^n) | {} | {:.1}% |\n",
            dist.exponential,
            (dist.exponential as f64 / total) * 100.0
        ));
        md.push_str(&format!(
            "| Unknown | {} | {:.1}% |\n\n",
            dist.unknown,
            (dist.unknown as f64 / total) * 100.0
        ));

        if !report.high_complexity_functions.is_empty() {
            md.push_str("## High Complexity Functions\n\n");
            md.push_str(
                "| File | Function | Line | Time Complexity | Space Complexity | Confidence |\n",
            );
            md.push_str(
                "|------|----------|------|-----------------|------------------|------------|\n",
            );

            for func in &report.high_complexity_functions {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {}% |\n",
                    func.file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    func.function_name,
                    func.line_number,
                    func.time_complexity.notation(),
                    func.space_complexity.notation(),
                    func.confidence
                ));
            }
            md.push('\n');
        }

        if !report.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for rec in &report.recommendations {
                md.push_str(&format!("- {rec}\n"));
            }
        }

        md
    }
}

impl Default for BigOAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let _analyzer = BigOAnalyzer::new();
        // Basic creation test - passes if compilation succeeds
    }

    #[test]
    fn test_complexity_analysis() {
        let analyzer = BigOAnalyzer::new();

        let rust_code = r#"
        fn bubble_sort(mut arr: Vec<i32>) -> Vec<i32> {
            for i in 0..arr.len() {
                for j in 0..arr.len() - 1 - i {
                    if arr[j] > arr[j + 1] {
                        arr.swap(j, j + 1);
                    }
                }
            }
            arr
        }
        "#;

        let result = analyzer.analyze_function_complexity("bubble_sort", rust_code, "rust");
        assert_eq!(result.time_complexity.class, BigOClass::Quadratic);
    }
}
