//! Zero-overhead complexity analysis system
//!
//! This module provides code complexity analysis without increasing binary size
//! beyond 2% by leveraging existing AST infrastructure.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Core complexity metrics for a code unit (function/method/class)
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct ComplexityMetrics {
    /// McCabe cyclomatic complexity
    pub cyclomatic: u16,
    /// Cognitive complexity (Sonar method)
    pub cognitive: u16,
    /// Maximum nesting depth
    pub nesting_max: u8,
    /// Logical lines of code
    pub lines: u16,
}

/// Complexity metrics for an entire file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileComplexityMetrics {
    pub path: String,
    pub total_complexity: ComplexityMetrics,
    pub functions: Vec<FunctionComplexity>,
    pub classes: Vec<ClassComplexity>,
}

/// Complexity metrics for a single function
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionComplexity {
    pub name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub metrics: ComplexityMetrics,
}

/// Complexity metrics for a class
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassComplexity {
    pub name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub metrics: ComplexityMetrics,
    pub methods: Vec<FunctionComplexity>,
}

/// Configuration thresholds for complexity rules
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplexityThresholds {
    pub cyclomatic_warn: u16,
    pub cyclomatic_error: u16,
    pub cognitive_warn: u16,
    pub cognitive_error: u16,
    pub nesting_max: u8,
    pub method_length: u16,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            cyclomatic_warn: 10,
            cyclomatic_error: 20,
            cognitive_warn: 15,
            cognitive_error: 30,
            nesting_max: 5,
            method_length: 50,
        }
    }
}

/// A violation of complexity thresholds
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "severity", rename_all = "lowercase")]
pub enum Violation {
    Error {
        rule: String,
        message: String,
        value: u16,
        threshold: u16,
        file: String,
        line: u32,
        function: Option<String>,
    },
    Warning {
        rule: String,
        message: String,
        value: u16,
        threshold: u16,
        file: String,
        line: u32,
        function: Option<String>,
    },
}

/// Summary statistics for complexity analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplexitySummary {
    pub total_files: usize,
    pub total_functions: usize,
    pub median_cyclomatic: f32,
    pub median_cognitive: f32,
    pub max_cyclomatic: u16,
    pub max_cognitive: u16,
    pub p90_cyclomatic: u16,
    pub p90_cognitive: u16,
    pub technical_debt_hours: f32,
}

/// A hotspot of high complexity in the codebase
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplexityHotspot {
    pub file: String,
    pub function: Option<String>,
    pub line: u32,
    pub complexity: u16,
    pub complexity_type: String,
}

/// Complete complexity analysis report
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplexityReport {
    pub summary: ComplexitySummary,
    pub violations: Vec<Violation>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub files: Vec<FileComplexityMetrics>,
}

/// Zero-allocation complexity visitor for AST traversal
pub struct ComplexityVisitor<'a> {
    pub complexity: &'a mut ComplexityMetrics,
    pub nesting_level: u8,
    pub current_function: Option<String>,
    pub functions: Vec<FunctionComplexity>,
    pub classes: Vec<ClassComplexity>,
}

impl<'a> ComplexityVisitor<'a> {
    pub fn new(complexity: &'a mut ComplexityMetrics) -> Self {
        Self {
            complexity,
            nesting_level: 0,
            current_function: None,
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }

    /// Calculate cognitive complexity increment based on node type and nesting
    #[inline(always)]
    pub fn calculate_cognitive_increment(&self, is_nesting_construct: bool) -> u16 {
        if is_nesting_construct {
            1 + self.nesting_level.saturating_sub(1) as u16
        } else {
            1
        }
    }

    /// Enter a nesting level
    #[inline(always)]
    pub fn enter_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_add(1);
        if self.nesting_level > self.complexity.nesting_max {
            self.complexity.nesting_max = self.nesting_level;
        }
    }

    /// Exit a nesting level
    #[inline(always)]
    pub fn exit_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_sub(1);
    }
}

/// Cache key computation for complexity metrics
pub fn compute_complexity_cache_key(path: &Path, content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    path.hash(&mut hasher);
    format!("cx:{:x}", hasher.finish())
}

/// Trait for complexity rules
pub trait ComplexityRule: Send + Sync {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation>;

    #[inline(always)]
    fn exceeds_threshold(&self, value: u16, threshold: u16) -> bool {
        value > threshold
    }
}

/// Cyclomatic complexity rule implementation
pub struct CyclomaticComplexityRule {
    warn_threshold: u16,
    error_threshold: u16,
}

impl CyclomaticComplexityRule {
    pub fn new(thresholds: &ComplexityThresholds) -> Self {
        Self {
            warn_threshold: thresholds.cyclomatic_warn,
            error_threshold: thresholds.cyclomatic_error,
        }
    }
}

impl ComplexityRule for CyclomaticComplexityRule {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation> {
        if self.exceeds_threshold(metrics.cyclomatic, self.error_threshold) {
            Some(Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: format!(
                    "Cyclomatic complexity of {} exceeds maximum allowed complexity of {}",
                    metrics.cyclomatic, self.error_threshold
                ),
                value: metrics.cyclomatic,
                threshold: self.error_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else if self.exceeds_threshold(metrics.cyclomatic, self.warn_threshold) {
            Some(Violation::Warning {
                rule: "cyclomatic-complexity".to_string(),
                message: format!(
                    "Cyclomatic complexity of {} exceeds recommended complexity of {}",
                    metrics.cyclomatic, self.warn_threshold
                ),
                value: metrics.cyclomatic,
                threshold: self.warn_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else {
            None
        }
    }
}

/// Cognitive complexity rule implementation
pub struct CognitiveComplexityRule {
    warn_threshold: u16,
    error_threshold: u16,
}

impl CognitiveComplexityRule {
    pub fn new(thresholds: &ComplexityThresholds) -> Self {
        Self {
            warn_threshold: thresholds.cognitive_warn,
            error_threshold: thresholds.cognitive_error,
        }
    }
}

impl ComplexityRule for CognitiveComplexityRule {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation> {
        if self.exceeds_threshold(metrics.cognitive, self.error_threshold) {
            Some(Violation::Error {
                rule: "cognitive-complexity".to_string(),
                message: format!(
                    "Cognitive complexity of {} exceeds maximum allowed complexity of {}",
                    metrics.cognitive, self.error_threshold
                ),
                value: metrics.cognitive,
                threshold: self.error_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else if self.exceeds_threshold(metrics.cognitive, self.warn_threshold) {
            Some(Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: format!(
                    "Cognitive complexity of {} exceeds recommended complexity of {}",
                    metrics.cognitive, self.warn_threshold
                ),
                value: metrics.cognitive,
                threshold: self.warn_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else {
            None
        }
    }
}

/// Aggregate complexity results from multiple files
pub fn aggregate_results(file_metrics: Vec<FileComplexityMetrics>) -> ComplexityReport {
    let mut all_cyclomatic: Vec<u16> = Vec::new();
    let mut all_cognitive: Vec<u16> = Vec::new();
    let mut violations = Vec::new();
    let mut hotspots = Vec::new();
    let mut total_functions = 0;

    let thresholds = ComplexityThresholds::default();
    let cyclomatic_rule = CyclomaticComplexityRule::new(&thresholds);
    let cognitive_rule = CognitiveComplexityRule::new(&thresholds);

    for file in &file_metrics {
        for func in &file.functions {
            total_functions += 1;
            all_cyclomatic.push(func.metrics.cyclomatic);
            all_cognitive.push(func.metrics.cognitive);

            // Check for violations
            if let Some(violation) = cyclomatic_rule.evaluate(
                &func.metrics,
                &file.path,
                func.line_start,
                Some(&func.name),
            ) {
                violations.push(violation);
            }
            if let Some(violation) = cognitive_rule.evaluate(
                &func.metrics,
                &file.path,
                func.line_start,
                Some(&func.name),
            ) {
                violations.push(violation);
            }

            // Track hotspots
            if func.metrics.cyclomatic > thresholds.cyclomatic_warn {
                hotspots.push(ComplexityHotspot {
                    file: file.path.clone(),
                    function: Some(func.name.clone()),
                    line: func.line_start,
                    complexity: func.metrics.cyclomatic,
                    complexity_type: "cyclomatic".to_string(),
                });
            }
        }

        for class in &file.classes {
            for method in &class.methods {
                total_functions += 1;
                all_cyclomatic.push(method.metrics.cyclomatic);
                all_cognitive.push(method.metrics.cognitive);

                // Check for violations
                if let Some(violation) = cyclomatic_rule.evaluate(
                    &method.metrics,
                    &file.path,
                    method.line_start,
                    Some(&method.name),
                ) {
                    violations.push(violation);
                }
                if let Some(violation) = cognitive_rule.evaluate(
                    &method.metrics,
                    &file.path,
                    method.line_start,
                    Some(&method.name),
                ) {
                    violations.push(violation);
                }
            }
        }
    }

    // Sort for percentile calculation
    all_cyclomatic.sort_unstable();
    all_cognitive.sort_unstable();

    // Calculate percentiles
    let p90_index = (all_cyclomatic.len() as f32 * 0.9) as usize;
    let p90_cyclomatic = all_cyclomatic.get(p90_index).copied().unwrap_or(0);
    let p90_cognitive = all_cognitive.get(p90_index).copied().unwrap_or(0);

    // Calculate medians (NO AVERAGES per spec)
    let median_cyclomatic = if !all_cyclomatic.is_empty() {
        let mid = all_cyclomatic.len() / 2;
        if all_cyclomatic.len() % 2 == 0 {
            (all_cyclomatic[mid - 1] + all_cyclomatic[mid]) as f32 / 2.0
        } else {
            all_cyclomatic[mid] as f32
        }
    } else {
        0.0
    };

    let median_cognitive = if !all_cognitive.is_empty() {
        let mid = all_cognitive.len() / 2;
        if all_cognitive.len() % 2 == 0 {
            (all_cognitive[mid - 1] + all_cognitive[mid]) as f32 / 2.0
        } else {
            all_cognitive[mid] as f32
        }
    } else {
        0.0
    };

    // Calculate max values
    let max_cyclomatic = all_cyclomatic.iter().max().copied().unwrap_or(0);
    let max_cognitive = all_cognitive.iter().max().copied().unwrap_or(0);

    // Sort hotspots by complexity
    hotspots.sort_by(|a, b| b.complexity.cmp(&a.complexity));
    hotspots.truncate(10); // Top 10 hotspots

    // Estimate refactoring time (simplified: 30 min per complexity point over threshold)
    let debt_minutes: f32 = violations
        .iter()
        .map(|v| match v {
            Violation::Error {
                value, threshold, ..
            } => (value - threshold) as f32 * 30.0,
            Violation::Warning {
                value, threshold, ..
            } => (value - threshold) as f32 * 15.0,
        })
        .sum();
    let technical_debt_hours = debt_minutes / 60.0;

    ComplexityReport {
        summary: ComplexitySummary {
            total_files: file_metrics.len(),
            total_functions,
            median_cyclomatic,
            median_cognitive,
            max_cyclomatic,
            max_cognitive,
            p90_cyclomatic,
            p90_cognitive,
            technical_debt_hours,
        },
        violations,
        hotspots,
        files: file_metrics,
    }
}

/// Format complexity summary for CLI output
pub fn format_complexity_summary(report: &ComplexityReport) -> String {
    let mut output = String::new();

    output.push_str("# Complexity Analysis Summary\n\n");

    output.push_str(&format!(
        "📊 **Files analyzed**: {}\n",
        report.summary.total_files
    ));
    output.push_str(&format!(
        "🔧 **Total functions**: {}\n\n",
        report.summary.total_functions
    ));

    output.push_str("## Complexity Metrics\n\n");
    output.push_str(&format!(
        "- **Median Cyclomatic**: {:.1}\n",
        report.summary.median_cyclomatic
    ));
    output.push_str(&format!(
        "- **Median Cognitive**: {:.1}\n",
        report.summary.median_cognitive
    ));
    output.push_str(&format!(
        "- **Max Cyclomatic**: {}\n",
        report.summary.max_cyclomatic
    ));
    output.push_str(&format!(
        "- **Max Cognitive**: {}\n",
        report.summary.max_cognitive
    ));
    output.push_str(&format!(
        "- **90th Percentile Cyclomatic**: {}\n",
        report.summary.p90_cyclomatic
    ));
    output.push_str(&format!(
        "- **90th Percentile Cognitive**: {}\n\n",
        report.summary.p90_cognitive
    ));

    if report.summary.technical_debt_hours > 0.0 {
        output.push_str(&format!(
            "⏱️  **Estimated Refactoring Time**: {:.1} hours\n\n",
            report.summary.technical_debt_hours
        ));
    }

    // Violations summary
    let error_count = report
        .violations
        .iter()
        .filter(|v| matches!(v, Violation::Error { .. }))
        .count();
    let warning_count = report
        .violations
        .iter()
        .filter(|v| matches!(v, Violation::Warning { .. }))
        .count();

    if error_count > 0 || warning_count > 0 {
        output.push_str("## Issues Found\n\n");
        if error_count > 0 {
            output.push_str(&format!("❌ **Errors**: {error_count}\n"));
        }
        if warning_count > 0 {
            output.push_str(&format!("⚠️  **Warnings**: {warning_count}\n"));
        }
        output.push('\n');
    }

    // Top hotspots
    if !report.hotspots.is_empty() {
        output.push_str("## Top Complexity Hotspots\n\n");
        for (i, hotspot) in report.hotspots.iter().take(5).enumerate() {
            output.push_str(&format!(
                "{}. `{}` - {} complexity: {}\n",
                i + 1,
                hotspot.function.as_deref().unwrap_or("<file>"),
                hotspot.complexity_type,
                hotspot.complexity
            ));
            output.push_str(&format!("   📁 {}:{}\n", hotspot.file, hotspot.line));
        }
    }

    output
}

/// Format full complexity report for CLI output
pub fn format_complexity_report(report: &ComplexityReport) -> String {
    let mut output = format_complexity_summary(report);

    output.push_str("\n## Detailed Violations\n\n");

    // Group violations by file
    let mut violations_by_file: rustc_hash::FxHashMap<&str, Vec<&Violation>> =
        rustc_hash::FxHashMap::default();
    for violation in &report.violations {
        let file = match violation {
            Violation::Error { file, .. } | Violation::Warning { file, .. } => file.as_str(),
        };
        violations_by_file.entry(file).or_default().push(violation);
    }

    for (file, violations) in violations_by_file {
        output.push_str(&format!("### {file}\n\n"));

        for violation in violations {
            match violation {
                Violation::Error {
                    rule,
                    message,
                    line,
                    function,
                    ..
                } => {
                    output.push_str(&format!(
                        "❌ **{}:{}** {} - {}\n",
                        line,
                        function.as_deref().unwrap_or(""),
                        rule,
                        message
                    ));
                }
                Violation::Warning {
                    rule,
                    message,
                    line,
                    function,
                    ..
                } => {
                    output.push_str(&format!(
                        "⚠️  **{}:{}** {} - {}\n",
                        line,
                        function.as_deref().unwrap_or(""),
                        rule,
                        message
                    ));
                }
            }
        }
        output.push('\n');
    }

    output
}

/// Format complexity report as SARIF for IDE integration
pub fn format_as_sarif(report: &ComplexityReport) -> Result<String, serde_json::Error> {
    use serde_json::json;

    let rules = vec![
        json!({
            "id": "cyclomatic-complexity",
            "name": "Cyclomatic Complexity",
            "shortDescription": {
                "text": "Function has high cyclomatic complexity"
            },
            "fullDescription": {
                "text": "Cyclomatic complexity measures the number of linearly independent paths through a function"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        json!({
            "id": "cognitive-complexity",
            "name": "Cognitive Complexity",
            "shortDescription": {
                "text": "Function has high cognitive complexity"
            },
            "fullDescription": {
                "text": "Cognitive complexity measures how difficult the function is to understand"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
    ];

    let mut results = Vec::new();
    for violation in &report.violations {
        let (rule_id, message, level, file, line, _function) = match violation {
            Violation::Error {
                rule,
                message,
                file,
                line,
                function,
                ..
            } => (rule, message, "error", file, line, function),
            Violation::Warning {
                rule,
                message,
                file,
                line,
                function,
                ..
            } => (rule, message, "warning", file, line, function),
        };

        results.push(json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": message
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": file
                    },
                    "region": {
                        "startLine": line
                    }
                }
            }]
        }));
    }

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // Helper function to create test complexity metrics
    fn create_test_metrics(
        cyclomatic: u16,
        cognitive: u16,
        nesting_max: u8,
        lines: u16,
    ) -> ComplexityMetrics {
        ComplexityMetrics {
            cyclomatic,
            cognitive,
            nesting_max,
            lines,
        }
    }

    // Helper function to create test function complexity
    fn create_test_function(
        name: &str,
        line_start: u32,
        line_end: u32,
        metrics: ComplexityMetrics,
    ) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start,
            line_end,
            metrics,
        }
    }

    #[test]
    fn test_complexity_metrics_default() {
        let metrics = ComplexityMetrics::default();
        assert_eq!(metrics.cyclomatic, 0);
        assert_eq!(metrics.cognitive, 0);
        assert_eq!(metrics.nesting_max, 0);
        assert_eq!(metrics.lines, 0);
    }

    #[test]
    fn test_complexity_metrics_creation() {
        let metrics = create_test_metrics(5, 10, 3, 25);
        assert_eq!(metrics.cyclomatic, 5);
        assert_eq!(metrics.cognitive, 10);
        assert_eq!(metrics.nesting_max, 3);
        assert_eq!(metrics.lines, 25);
    }

    #[test]
    fn test_complexity_thresholds_default() {
        let thresholds = ComplexityThresholds::default();
        assert_eq!(thresholds.cyclomatic_warn, 10);
        assert_eq!(thresholds.cyclomatic_error, 20);
        assert_eq!(thresholds.cognitive_warn, 15);
        assert_eq!(thresholds.cognitive_error, 30);
        assert_eq!(thresholds.nesting_max, 5);
        assert_eq!(thresholds.method_length, 50);
    }

    #[test]
    fn test_complexity_thresholds_custom() {
        let thresholds = ComplexityThresholds {
            cyclomatic_warn: 8,
            cyclomatic_error: 15,
            cognitive_warn: 12,
            cognitive_error: 25,
            nesting_max: 4,
            method_length: 40,
        };
        assert_eq!(thresholds.cyclomatic_warn, 8);
        assert_eq!(thresholds.cyclomatic_error, 15);
        assert_eq!(thresholds.cognitive_warn, 12);
        assert_eq!(thresholds.cognitive_error, 25);
        assert_eq!(thresholds.nesting_max, 4);
        assert_eq!(thresholds.method_length, 40);
    }

    #[test]
    fn test_function_complexity_creation() {
        let metrics = create_test_metrics(3, 8, 2, 15);
        let func = create_test_function("test_function", 10, 25, metrics);
        assert_eq!(func.name, "test_function");
        assert_eq!(func.line_start, 10);
        assert_eq!(func.line_end, 25);
        assert_eq!(func.metrics.cyclomatic, 3);
        assert_eq!(func.metrics.cognitive, 8);
    }

    #[test]
    fn test_class_complexity_creation() {
        let metrics = create_test_metrics(15, 25, 4, 100);
        let method = create_test_function("method1", 5, 15, create_test_metrics(3, 5, 2, 10));
        let class = ClassComplexity {
            name: "TestClass".to_string(),
            line_start: 1,
            line_end: 50,
            metrics,
            methods: vec![method],
        };
        assert_eq!(class.name, "TestClass");
        assert_eq!(class.line_start, 1);
        assert_eq!(class.line_end, 50);
        assert_eq!(class.methods.len(), 1);
        assert_eq!(class.methods[0].name, "method1");
    }

    #[test]
    fn test_file_complexity_metrics_creation() {
        let total_metrics = create_test_metrics(20, 35, 5, 200);
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(5, 8, 2, 10));
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(7, 12, 3, 15));

        let file_metrics = FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: total_metrics,
            functions: vec![func1, func2],
            classes: vec![],
        };

        assert_eq!(file_metrics.path, "test.rs");
        assert_eq!(file_metrics.functions.len(), 2);
        assert_eq!(file_metrics.classes.len(), 0);
        assert_eq!(file_metrics.total_complexity.cyclomatic, 20);
    }

    #[test]
    fn test_complexity_visitor_creation() {
        let mut metrics = ComplexityMetrics::default();
        let visitor = ComplexityVisitor::new(&mut metrics);
        assert_eq!(visitor.nesting_level, 0);
        assert!(visitor.current_function.is_none());
        assert!(visitor.functions.is_empty());
        assert!(visitor.classes.is_empty());
    }

    #[test]
    fn test_complexity_visitor_cognitive_increment() {
        let mut metrics = ComplexityMetrics::default();
        let visitor = ComplexityVisitor::new(&mut metrics);

        // Test non-nesting construct
        assert_eq!(visitor.calculate_cognitive_increment(false), 1);

        // Test nesting construct at level 0
        assert_eq!(visitor.calculate_cognitive_increment(true), 1);
    }

    #[test]
    fn test_complexity_visitor_cognitive_increment_with_nesting() {
        let mut metrics = ComplexityMetrics::default();
        let mut visitor = ComplexityVisitor::new(&mut metrics);

        // Increase nesting level
        visitor.nesting_level = 3;

        // Test nesting construct with nesting level
        assert_eq!(visitor.calculate_cognitive_increment(true), 3); // 1 + (3 - 1)

        // Test non-nesting construct
        assert_eq!(visitor.calculate_cognitive_increment(false), 1);
    }

    #[test]
    fn test_complexity_visitor_nesting_management() {
        let mut metrics = ComplexityMetrics::default();
        let mut visitor = ComplexityVisitor::new(&mut metrics);

        assert_eq!(visitor.nesting_level, 0);
        assert_eq!(visitor.complexity.nesting_max, 0);

        // Enter nesting
        visitor.enter_nesting();
        assert_eq!(visitor.nesting_level, 1);
        assert_eq!(visitor.complexity.nesting_max, 1);

        visitor.enter_nesting();
        assert_eq!(visitor.nesting_level, 2);
        assert_eq!(visitor.complexity.nesting_max, 2);

        // Exit nesting
        visitor.exit_nesting();
        assert_eq!(visitor.nesting_level, 1);
        assert_eq!(visitor.complexity.nesting_max, 2); // Max should remain

        visitor.exit_nesting();
        assert_eq!(visitor.nesting_level, 0);
        assert_eq!(visitor.complexity.nesting_max, 2);
    }

    #[test]
    fn test_complexity_visitor_nesting_saturation() {
        let mut metrics = ComplexityMetrics::default();
        let mut visitor = ComplexityVisitor::new(&mut metrics);

        // Test saturation at maximum nesting
        visitor.nesting_level = 255; // u8::MAX
        visitor.enter_nesting();
        assert_eq!(visitor.nesting_level, 255); // Should saturate

        // Test saturation at zero
        visitor.nesting_level = 0;
        visitor.exit_nesting();
        assert_eq!(visitor.nesting_level, 0); // Should saturate at 0
    }

    #[test]
    fn test_compute_complexity_cache_key() {
        let path = Path::new("test.rs");
        let content1 = b"fn test() {}";
        let content2 = b"fn test() { println!(\"hello\"); }";

        let key1 = compute_complexity_cache_key(path, content1);
        let key2 = compute_complexity_cache_key(path, content1);
        let key3 = compute_complexity_cache_key(path, content2);

        // Same content should produce same key
        assert_eq!(key1, key2);

        // Different content should produce different key
        assert_ne!(key1, key3);

        // Key should start with "cx:"
        assert!(key1.starts_with("cx:"));
        assert!(key3.starts_with("cx:"));
    }

    #[test]
    fn test_compute_complexity_cache_key_different_paths() {
        let path1 = Path::new("test1.rs");
        let path2 = Path::new("test2.rs");
        let content = b"fn test() {}";

        let key1 = compute_complexity_cache_key(path1, content);
        let key2 = compute_complexity_cache_key(path2, content);

        // Different paths should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cyclomatic_complexity_rule_creation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        assert_eq!(rule.warn_threshold, 10);
        assert_eq!(rule.error_threshold, 20);
    }

    #[test]
    fn test_cyclomatic_complexity_rule_exceeds_threshold() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);

        assert!(!rule.exceeds_threshold(5, 10));
        assert!(!rule.exceeds_threshold(10, 10)); // Equal should not exceed
        assert!(rule.exceeds_threshold(15, 10));
    }

    #[test]
    fn test_cyclomatic_complexity_rule_no_violation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(5, 0, 0, 0); // Below warn threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_none());
    }

    #[test]
    fn test_cyclomatic_complexity_rule_warning() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(15, 0, 0, 0); // Above warn, below error

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Warning {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cyclomatic-complexity");
                assert!(message.contains("15"));
                assert!(message.contains("10"));
                assert_eq!(value, 15);
                assert_eq!(threshold, 10);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected warning violation"),
        }
    }

    #[test]
    fn test_cyclomatic_complexity_rule_error() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(25, 0, 0, 0); // Above error threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Error {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cyclomatic-complexity");
                assert!(message.contains("25"));
                assert!(message.contains("20"));
                assert_eq!(value, 25);
                assert_eq!(threshold, 20);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected error violation"),
        }
    }

    #[test]
    fn test_cyclomatic_complexity_rule_without_function_name() {
        let thresholds = ComplexityThresholds::default();
        let rule = CyclomaticComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(15, 0, 0, 0);

        let result = rule.evaluate(&metrics, "test.rs", 10, None);
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Warning { function, .. } => {
                assert_eq!(function, None);
            }
            _ => panic!("Expected warning violation"),
        }
    }

    #[test]
    fn test_cognitive_complexity_rule_creation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        assert_eq!(rule.warn_threshold, 15);
        assert_eq!(rule.error_threshold, 30);
    }

    #[test]
    fn test_cognitive_complexity_rule_no_violation() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(0, 10, 0, 0); // Below warn threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_none());
    }

    #[test]
    fn test_cognitive_complexity_rule_warning() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(0, 20, 0, 0); // Above warn, below error

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Warning {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cognitive-complexity");
                assert!(message.contains("20"));
                assert!(message.contains("15"));
                assert_eq!(value, 20);
                assert_eq!(threshold, 15);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected warning violation"),
        }
    }

    #[test]
    fn test_cognitive_complexity_rule_error() {
        let thresholds = ComplexityThresholds::default();
        let rule = CognitiveComplexityRule::new(&thresholds);
        let metrics = create_test_metrics(0, 35, 0, 0); // Above error threshold

        let result = rule.evaluate(&metrics, "test.rs", 10, Some("test_function"));
        assert!(result.is_some());

        match result.unwrap() {
            Violation::Error {
                rule: rule_name,
                message,
                value,
                threshold,
                file,
                line,
                function,
            } => {
                assert_eq!(rule_name, "cognitive-complexity");
                assert!(message.contains("35"));
                assert!(message.contains("30"));
                assert_eq!(value, 35);
                assert_eq!(threshold, 30);
                assert_eq!(file, "test.rs");
                assert_eq!(line, 10);
                assert_eq!(function, Some("test_function".to_string()));
            }
            _ => panic!("Expected error violation"),
        }
    }

    #[test]
    fn test_complexity_hotspot_creation() {
        let hotspot = ComplexityHotspot {
            file: "test.rs".to_string(),
            function: Some("complex_function".to_string()),
            line: 42,
            complexity: 25,
            complexity_type: "cyclomatic".to_string(),
        };

        assert_eq!(hotspot.file, "test.rs");
        assert_eq!(hotspot.function, Some("complex_function".to_string()));
        assert_eq!(hotspot.line, 42);
        assert_eq!(hotspot.complexity, 25);
        assert_eq!(hotspot.complexity_type, "cyclomatic");
    }

    #[test]
    fn test_aggregate_results_empty() {
        let file_metrics = vec![];
        let report = aggregate_results(file_metrics);

        assert_eq!(report.summary.total_files, 0);
        assert_eq!(report.summary.total_functions, 0);
        assert_eq!(report.summary.median_cyclomatic, 0.0);
        assert_eq!(report.summary.median_cognitive, 0.0);
        assert_eq!(report.summary.max_cyclomatic, 0);
        assert_eq!(report.summary.max_cognitive, 0);
        assert_eq!(report.summary.p90_cyclomatic, 0);
        assert_eq!(report.summary.p90_cognitive, 0);
        assert_eq!(report.summary.technical_debt_hours, 0.0);
        assert!(report.violations.is_empty());
        assert!(report.hotspots.is_empty());
        assert!(report.files.is_empty());
    }

    #[test]
    fn test_aggregate_results_single_file() {
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(5, 8, 2, 10));
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(15, 20, 3, 15)); // Should trigger warning

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(20, 28, 3, 25),
            functions: vec![func1, func2],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        assert_eq!(report.summary.total_files, 1);
        assert_eq!(report.summary.total_functions, 2);
        assert_eq!(report.summary.median_cyclomatic, 10.0); // (5 + 15) / 2
        assert_eq!(report.summary.median_cognitive, 14.0); // (8 + 20) / 2
        assert_eq!(report.summary.max_cyclomatic, 15);
        assert_eq!(report.summary.max_cognitive, 20);

        // Should have violations for func2
        assert!(!report.violations.is_empty());

        // Should have hotspots for func2
        assert!(!report.hotspots.is_empty());
        assert_eq!(report.hotspots[0].function, Some("func2".to_string()));
    }

    #[test]
    fn test_aggregate_results_with_classes() {
        let method1 = create_test_function("method1", 5, 15, create_test_metrics(8, 12, 2, 10));
        let method2 = create_test_function("method2", 20, 30, create_test_metrics(25, 35, 4, 15)); // Should trigger errors

        let class = ClassComplexity {
            name: "TestClass".to_string(),
            line_start: 1,
            line_end: 50,
            metrics: create_test_metrics(33, 47, 4, 25),
            methods: vec![method1, method2],
        };

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(33, 47, 4, 25),
            functions: vec![],
            classes: vec![class],
        }];

        let report = aggregate_results(file_metrics);

        assert_eq!(report.summary.total_files, 1);
        assert_eq!(report.summary.total_functions, 2); // Methods count as functions
        assert_eq!(report.summary.max_cyclomatic, 25);
        assert_eq!(report.summary.max_cognitive, 35);

        // Should have violations for method2 (both cyclomatic and cognitive)
        assert!(report.violations.len() >= 2);

        // Check for error violations
        let error_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| matches!(v, Violation::Error { .. }))
            .collect();
        assert!(!error_violations.is_empty());
    }

    #[test]
    fn test_aggregate_results_median_calculation_odd() {
        // Test with odd number of functions for median calculation
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(5, 10, 1, 10));
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(7, 12, 2, 15));
        let func3 = create_test_function("func3", 50, 60, create_test_metrics(9, 15, 2, 20));

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(21, 37, 2, 45),
            functions: vec![func1, func2, func3],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // With values [5, 7, 9], median should be 7
        assert_eq!(report.summary.median_cyclomatic, 7.0);
        // With values [10, 12, 15], median should be 12
        assert_eq!(report.summary.median_cognitive, 12.0);
    }

    #[test]
    fn test_aggregate_results_percentile_calculation() {
        // Create 10 functions to test p90 calculation
        let mut functions = Vec::new();
        for i in 1..=10 {
            functions.push(create_test_function(
                &format!("func{}", i),
                i * 10,
                i * 10 + 10,
                create_test_metrics(i as u16, i as u16 * 2, 1, 10),
            ));
        }

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(55, 110, 1, 100),
            functions,
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // p90 of [1,2,3,4,5,6,7,8,9,10] should be around 9 or 10 depending on implementation
        assert!(report.summary.p90_cyclomatic >= 9 && report.summary.p90_cyclomatic <= 10);
        // p90 of [2,4,6,8,10,12,14,16,18,20] should be around 18 or 20 depending on implementation
        assert!(report.summary.p90_cognitive >= 18 && report.summary.p90_cognitive <= 20);
    }

    #[test]
    fn test_aggregate_results_technical_debt_calculation() {
        // Create functions that exceed thresholds to test debt calculation
        let func1 = create_test_function("func1", 10, 20, create_test_metrics(15, 20, 2, 10)); // Warning: 5 over cyc, 5 over cog
        let func2 = create_test_function("func2", 30, 40, create_test_metrics(25, 35, 3, 15)); // Error: 5 over cyc, 5 over cog

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(40, 55, 3, 25),
            functions: vec![func1, func2],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // Should have violations and technical debt
        assert!(!report.violations.is_empty());
        assert!(report.summary.technical_debt_hours > 0.0);

        // Debt calculation: warnings = 15min per point, errors = 30min per point
        // func1: 5 cyc warn (75min) + 5 cog warn (75min) = 150min = 2.5h
        // func2: 5 cyc error (150min) + 5 cog error (150min) = 300min = 5h
        // Total: 7.5h
        let expected_debt = (5.0 * 15.0 + 5.0 * 15.0 + 5.0 * 30.0 + 5.0 * 30.0) / 60.0;
        assert!((report.summary.technical_debt_hours - expected_debt).abs() < 0.1);
    }

    #[test]
    fn test_aggregate_results_hotspot_sorting() {
        let func1 =
            create_test_function("low_complexity", 10, 20, create_test_metrics(12, 18, 2, 10)); // Medium hotspot
        let func2 = create_test_function(
            "high_complexity",
            30,
            40,
            create_test_metrics(25, 35, 3, 15),
        ); // High hotspot
        let func3 = create_test_function(
            "medium_complexity",
            50,
            60,
            create_test_metrics(15, 22, 2, 12),
        ); // Lower hotspot

        let file_metrics = vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: create_test_metrics(52, 75, 3, 37),
            functions: vec![func1, func2, func3],
            classes: vec![],
        }];

        let report = aggregate_results(file_metrics);

        // Hotspots should be sorted by complexity (descending)
        assert!(report.hotspots.len() >= 3);
        assert_eq!(
            report.hotspots[0].function,
            Some("high_complexity".to_string())
        );
        assert_eq!(report.hotspots[0].complexity, 25);
        assert_eq!(
            report.hotspots[1].function,
            Some("medium_complexity".to_string())
        );
        assert_eq!(report.hotspots[1].complexity, 15);
        assert_eq!(
            report.hotspots[2].function,
            Some("low_complexity".to_string())
        );
        assert_eq!(report.hotspots[2].complexity, 12);
    }

    #[test]
    fn test_format_complexity_summary_empty() {
        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 0,
                total_functions: 0,
                median_cyclomatic: 0.0,
                median_cognitive: 0.0,
                max_cyclomatic: 0,
                max_cognitive: 0,
                p90_cyclomatic: 0,
                p90_cognitive: 0,
                technical_debt_hours: 0.0,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_summary(&report);

        assert!(output.contains("# Complexity Analysis Summary"));
        assert!(output.contains("**Files analyzed**: 0"));
        assert!(output.contains("**Total functions**: 0"));
        assert!(output.contains("**Median Cyclomatic**: 0.0"));
        assert!(output.contains("**Median Cognitive**: 0.0"));
        assert!(output.contains("**Max Cyclomatic**: 0"));
        assert!(output.contains("**Max Cognitive**: 0"));
        assert!(!output.contains("**Estimated Refactoring Time**")); // Should not show 0 hours
        assert!(!output.contains("## Issues Found")); // No violations
        assert!(!output.contains("## Top Complexity Hotspots")); // No hotspots
    }

    #[test]
    fn test_format_complexity_summary_with_data() {
        let violations = vec![
            Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: "Too complex".to_string(),
                value: 25,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 10,
                function: Some("test_func".to_string()),
            },
            Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: "Getting complex".to_string(),
                value: 18,
                threshold: 15,
                file: "test.rs".to_string(),
                line: 20,
                function: Some("other_func".to_string()),
            },
        ];

        let hotspots = vec![
            ComplexityHotspot {
                file: "test.rs".to_string(),
                function: Some("complex_function".to_string()),
                line: 42,
                complexity: 25,
                complexity_type: "cyclomatic".to_string(),
            },
            ComplexityHotspot {
                file: "test2.rs".to_string(),
                function: Some("another_complex".to_string()),
                line: 100,
                complexity: 20,
                complexity_type: "cognitive".to_string(),
            },
        ];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 2,
                total_functions: 5,
                median_cyclomatic: 8.5,
                median_cognitive: 12.3,
                max_cyclomatic: 25,
                max_cognitive: 30,
                p90_cyclomatic: 20,
                p90_cognitive: 25,
                technical_debt_hours: 2.5,
            },
            violations,
            hotspots,
            files: vec![],
        };

        let output = format_complexity_summary(&report);

        assert!(output.contains("**Files analyzed**: 2"));
        assert!(output.contains("**Total functions**: 5"));
        assert!(output.contains("**Median Cyclomatic**: 8.5"));
        assert!(output.contains("**Median Cognitive**: 12.3"));
        assert!(output.contains("**Max Cyclomatic**: 25"));
        assert!(output.contains("**Max Cognitive**: 30"));
        assert!(output.contains("**90th Percentile Cyclomatic**: 20"));
        assert!(output.contains("**90th Percentile Cognitive**: 25"));
        assert!(output.contains("**Estimated Refactoring Time**: 2.5 hours"));
        assert!(output.contains("## Issues Found"));
        assert!(output.contains("**Errors**: 1"));
        assert!(output.contains("**Warnings**: 1"));
        assert!(output.contains("## Top Complexity Hotspots"));
        assert!(output.contains("`complex_function` - cyclomatic complexity: 25"));
        assert!(output.contains("📁 test.rs:42"));
    }

    #[test]
    fn test_format_complexity_report() {
        let violations = vec![Violation::Error {
            rule: "cyclomatic-complexity".to_string(),
            message: "Function too complex".to_string(),
            value: 25,
            threshold: 20,
            file: "test.rs".to_string(),
            line: 10,
            function: Some("test_func".to_string()),
        }];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 1,
                total_functions: 1,
                median_cyclomatic: 25.0,
                median_cognitive: 30.0,
                max_cyclomatic: 25,
                max_cognitive: 30,
                p90_cyclomatic: 25,
                p90_cognitive: 30,
                technical_debt_hours: 1.0,
            },
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let output = format_complexity_report(&report);

        // Should include summary
        assert!(output.contains("# Complexity Analysis Summary"));

        // Should include detailed violations
        assert!(output.contains("## Detailed Violations"));
        assert!(output.contains("### test.rs"));
        assert!(output.contains("❌ **10:test_func** cyclomatic-complexity - Function too complex"));
    }

    #[test]
    fn test_format_as_sarif() {
        let violations = vec![
            Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: "Function too complex".to_string(),
                value: 25,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 10,
                function: Some("test_func".to_string()),
            },
            Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: "Function getting complex".to_string(),
                value: 18,
                threshold: 15,
                file: "test.rs".to_string(),
                line: 20,
                function: Some("other_func".to_string()),
            },
        ];

        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 1,
                total_functions: 2,
                median_cyclomatic: 21.5,
                median_cognitive: 18.0,
                max_cyclomatic: 25,
                max_cognitive: 18,
                p90_cyclomatic: 25,
                p90_cognitive: 18,
                technical_debt_hours: 0.5,
            },
            violations,
            hotspots: vec![],
            files: vec![],
        };

        let sarif_output = format_as_sarif(&report).expect("Should generate SARIF");

        // Basic SARIF structure checks
        assert!(sarif_output.contains("\"version\": \"2.1.0\""));
        assert!(sarif_output.contains("\"$schema\""));
        assert!(sarif_output.contains("\"runs\""));
        assert!(sarif_output.contains("\"tool\""));
        assert!(sarif_output.contains("\"driver\""));
        assert!(sarif_output.contains("\"name\": \"pmat\""));
        assert!(sarif_output.contains("\"rules\""));
        assert!(sarif_output.contains("\"results\""));

        // Rule definitions
        assert!(sarif_output.contains("\"id\": \"cyclomatic-complexity\""));
        assert!(sarif_output.contains("\"id\": \"cognitive-complexity\""));

        // Results
        assert!(sarif_output.contains("\"ruleId\": \"cyclomatic-complexity\""));
        assert!(sarif_output.contains("\"ruleId\": \"cognitive-complexity\""));
        assert!(sarif_output.contains("\"level\": \"error\""));
        assert!(sarif_output.contains("\"level\": \"warning\""));
        assert!(sarif_output.contains("\"text\": \"Function too complex\""));
        assert!(sarif_output.contains("\"text\": \"Function getting complex\""));
        assert!(sarif_output.contains("\"uri\": \"test.rs\""));
        assert!(sarif_output.contains("\"startLine\": 10"));
        assert!(sarif_output.contains("\"startLine\": 20"));
    }

    #[test]
    fn test_format_as_sarif_empty() {
        let report = ComplexityReport {
            summary: ComplexitySummary {
                total_files: 0,
                total_functions: 0,
                median_cyclomatic: 0.0,
                median_cognitive: 0.0,
                max_cyclomatic: 0,
                max_cognitive: 0,
                p90_cyclomatic: 0,
                p90_cognitive: 0,
                technical_debt_hours: 0.0,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        };

        let sarif_output = format_as_sarif(&report).expect("Should generate SARIF");

        // Should still have valid SARIF structure with empty results
        assert!(sarif_output.contains("\"version\": \"2.1.0\""));
        assert!(sarif_output.contains("\"results\": []"));
    }

    #[test]
    fn test_violation_serialization() {
        let error_violation = Violation::Error {
            rule: "test-rule".to_string(),
            message: "Test message".to_string(),
            value: 25,
            threshold: 20,
            file: "test.rs".to_string(),
            line: 10,
            function: Some("test_func".to_string()),
        };

        let warning_violation = Violation::Warning {
            rule: "test-rule".to_string(),
            message: "Test warning".to_string(),
            value: 15,
            threshold: 10,
            file: "test.rs".to_string(),
            line: 20,
            function: None,
        };

        // Test that violations can be serialized/deserialized
        let error_json = serde_json::to_string(&error_violation).expect("Should serialize");
        let warning_json = serde_json::to_string(&warning_violation).expect("Should serialize");

        assert!(error_json.contains("\"severity\":\"error\""));
        assert!(warning_json.contains("\"severity\":\"warning\""));

        let _: Violation = serde_json::from_str(&error_json).expect("Should deserialize");
        let _: Violation = serde_json::from_str(&warning_json).expect("Should deserialize");
    }
}
