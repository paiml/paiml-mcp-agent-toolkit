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
                    "name": "paiml-mcp-agent-toolkit",
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
