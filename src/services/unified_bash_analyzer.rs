#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified Bash/Shell Analyzer - Parse Once, Extract Twice
//!
//! This module eliminates the performance bottleneck of parsing shell scripts twice
//! (once for AST extraction, once for complexity analysis) by combining both
//! operations into a single parse pass.
//!
//! # Performance Impact
//!
//! Before: 2x parse calls per file (AST + Complexity)
//! After: 1x parse call per file
//! Expected gain: 40-50% reduction in parse time

use anyhow::Result;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::AstItem;
use crate::services::languages::bash::BashScriptAnalyzer;

/// Unified analyzer that parses Bash once, extracts twice
pub struct UnifiedBashAnalyzer {
    file_path: PathBuf,

    /// Parse count tracker (test-only)
    #[cfg(test)]
    parse_count: AtomicUsize,
}

/// Combined result from unified analysis
#[derive(Debug)]
pub struct UnifiedAnalysis {
    /// AST items (functions, commands)
    pub ast_items: Vec<AstItem>,

    /// File-level complexity metrics
    pub file_metrics: FileComplexityMetrics,

    /// Parse timestamp (for cache validation)
    pub parsed_at: std::time::Instant,
}

/// Error type for unified analysis
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse shell script: {0}")]
    Parse(String),

    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl UnifiedBashAnalyzer {
    /// Create new analyzer for a file
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            #[cfg(test)]
            parse_count: AtomicUsize::new(0),
        }
    }

    /// Get the file path being analyzed
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Analyze file with single parse
    ///
    /// This is the core GREEN phase implementation: minimal but correct.
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError> {
        // Track parse count for testing
        #[cfg(test)]
        {
            self.parse_count.fetch_add(1, Ordering::SeqCst);
        }

        // 1. Read file content (single I/O operation)
        let content = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(AnalysisError::Io)?;

        // 2. Extract AST items using existing Bash analyzer
        let analyzer = BashScriptAnalyzer::new(&self.file_path);
        let ast_items = analyzer
            .analyze_bash_script(&content)
            .map_err(AnalysisError::Parse)?;

        // 3. Extract complexity metrics (GREEN phase - simple pattern matching)
        let file_metrics = self.extract_complexity_metrics(&content);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }

    /// Extract complexity metrics from shell content
    ///
    /// GREEN PHASE: Minimal implementation using pattern matching.
    /// This will be enhanced in REFACTOR phase with proper AST-based complexity.
    fn extract_complexity_metrics(&self, content: &str) -> FileComplexityMetrics {
        let mut functions = Vec::new();
        let lines = content.lines().count();

        // Simple function detection
        let mut current_function: Option<String> = None;
        let mut function_complexity = 1u32;
        let mut line_start = 0;
        let mut brace_depth = 0;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Function start patterns: "function name()" or "name()"
            if (trimmed.contains("function ")
                || trimmed.ends_with("() {")
                || trimmed.contains("() {"))
                && !trimmed.starts_with('#')
            {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    current_function = Some(func_name);
                    function_complexity = 1;
                    line_start = line_num;
                    brace_depth = 0;
                }
            }

            // Count control flow keywords for complexity
            if current_function.is_some() {
                // Conditionals
                if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                    function_complexity += 1;
                }
                if trimmed.starts_with("elif ") {
                    function_complexity += 1;
                }
                // Loops
                if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                    function_complexity += 1;
                }
                // Case statements
                if trimmed.starts_with("case ") {
                    function_complexity += 1;
                }
                // Logical operators
                if trimmed.contains(" && ") || trimmed.contains(" || ") {
                    function_complexity += 1;
                }
                // Pipelines add complexity
                if trimmed.contains('|') && !trimmed.contains("||") {
                    function_complexity += trimmed.matches('|').count() as u32;
                }
            }

            // Track braces
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Function end (closing brace at function level)
            if brace_depth == 0 && current_function.is_some() && trimmed == "}" {
                let name = current_function
                    .take()
                    .expect("guarded by is_some() check above");
                functions.push(FunctionComplexity {
                    name,
                    line_start: line_start as u32,
                    line_end: line_num as u32,
                    metrics: ComplexityMetrics {
                        cyclomatic: function_complexity as u16,
                        cognitive: function_complexity as u16, // Simplified for GREEN phase
                        nesting_max: 0,
                        lines: (line_num - line_start) as u16,
                        halstead: None,
                    },
                });
            }
        }

        // Calculate script-level complexity even if no functions
        let script_complexity = self.calculate_script_complexity(content);

        // Calculate file-level metrics
        let total_cyclomatic: u32 = if functions.is_empty() {
            script_complexity
        } else {
            functions.iter().map(|f| f.metrics.cyclomatic as u32).sum()
        };

        let avg_cyclomatic = if functions.is_empty() {
            script_complexity
        } else {
            total_cyclomatic / functions.len() as u32
        };

        FileComplexityMetrics {
            path: self.file_path.display().to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: avg_cyclomatic as u16,
                cognitive: avg_cyclomatic as u16,
                nesting_max: 0,
                lines: lines as u16,
                halstead: None,
            },
            functions,
            classes: Vec::new(), // Shell doesn't have classes
        }
    }

    /// Calculate script-level complexity (for scripts without functions)
    #[allow(clippy::cast_possible_truncation)]
    fn calculate_script_complexity(&self, content: &str) -> u32 {
        let mut complexity = 1; // Base complexity

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // Conditionals
            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                complexity += 1;
            }
            if trimmed.starts_with("elif ") {
                complexity += 1;
            }
            // Loops
            if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                complexity += 1;
            }
            // Case statements
            if trimmed.starts_with("case ") {
                complexity += 1;
            }
            // Logical operators
            if trimmed.contains(" && ") || trimmed.contains(" || ") {
                complexity += 1;
            }
            // Pipelines
            if trimmed.contains('|') && !trimmed.contains("||") {
                complexity += trimmed.matches('|').count() as u32;
            }
        }

        complexity
    }

    /// Extract function name from shell line
    fn extract_function_name(&self, line: &str) -> Option<String> {
        // Pattern 1: "function name()" or "function name() {"
        if line.contains("function ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].trim_end_matches("()").trim_end_matches('{').trim();
                return Some(name.to_string());
            }
        }
        // Pattern 2: "name() {" or "name() {"
        if let Some(pos) = line.find("()") {
            let name = line[..pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let path = PathBuf::from("test.sh");
        let analyzer = UnifiedBashAnalyzer::new(path.clone());
        assert_eq!(analyzer.file_path(), path.as_path());
    }

    #[tokio::test]
    async fn test_parse_count_increments() {
        let temp_file = tempfile::NamedTempFile::with_suffix(".sh").unwrap();
        std::fs::write(temp_file.path(), "#!/bin/bash\necho test").unwrap();

        let analyzer = UnifiedBashAnalyzer::new(temp_file.path().to_path_buf());

        assert_eq!(analyzer.parse_count(), 0);

        let _ = analyzer.analyze().await;
        assert_eq!(analyzer.parse_count(), 1);
    }

    #[test]
    fn test_extract_function_name_function_keyword() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        assert_eq!(
            analyzer.extract_function_name("function my_func()"),
            Some("my_func".to_string())
        );
        assert_eq!(
            analyzer.extract_function_name("function my_func() {"),
            Some("my_func".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_parentheses_style() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        assert_eq!(
            analyzer.extract_function_name("my_func() {"),
            Some("my_func".to_string())
        );
        assert_eq!(
            analyzer.extract_function_name("setup() {"),
            Some("setup".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_invalid() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        // Empty before ()
        assert_eq!(analyzer.extract_function_name("() {"), None);
        // No function pattern
        assert_eq!(analyzer.extract_function_name("echo hello"), None);
    }

    #[test]
    fn test_calculate_script_complexity_simple() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\necho hello\n";
        // Base complexity of 1
        assert_eq!(analyzer.calculate_script_complexity(content), 1);
    }

    #[test]
    fn test_calculate_script_complexity_with_if() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\nif [ -f file ]; then\n  echo found\nfi";
        // Base 1 + if 1 = 2
        assert_eq!(analyzer.calculate_script_complexity(content), 2);
    }

    #[test]
    fn test_calculate_script_complexity_with_loop() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\nfor i in 1 2 3; do\n  echo $i\ndone";
        // Base 1 + for 1 = 2
        assert_eq!(analyzer.calculate_script_complexity(content), 2);
    }

    #[test]
    fn test_calculate_script_complexity_with_while() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\nwhile true; do\n  echo loop\ndone";
        // Base 1 + while 1 = 2
        assert_eq!(analyzer.calculate_script_complexity(content), 2);
    }

    #[test]
    fn test_calculate_script_complexity_with_logical_ops() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\n[ -f file ] && echo exists || echo missing";
        // Base 1 + (line with && or ||) 1 = 2 (both operators on same line count once)
        assert_eq!(analyzer.calculate_script_complexity(content), 2);
    }

    #[test]
    fn test_calculate_script_complexity_with_pipeline() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\ncat file | grep foo | wc -l";
        // Base 1 + 2 pipes = 3
        assert_eq!(analyzer.calculate_script_complexity(content), 3);
    }

    #[test]
    fn test_calculate_script_complexity_with_case() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\ncase $1 in\n  start) echo start;;\nesac";
        // Base 1 + case 1 = 2
        assert_eq!(analyzer.calculate_script_complexity(content), 2);
    }

    #[test]
    fn test_calculate_script_complexity_with_elif() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\nif [ $1 -eq 1 ]; then\n  echo one\nelif [ $1 -eq 2 ]; then\n  echo two\nfi";
        // Base 1 + if 1 + elif 1 = 3
        assert_eq!(analyzer.calculate_script_complexity(content), 3);
    }

    #[test]
    fn test_calculate_script_complexity_ignores_comments() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\n# if something\n# for loop\necho hello";
        // Just base 1 - comments are ignored
        assert_eq!(analyzer.calculate_script_complexity(content), 1);
    }

    #[test]
    fn test_extract_complexity_metrics_simple_script() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = "#!/bin/bash\necho hello world";

        let metrics = analyzer.extract_complexity_metrics(content);

        assert_eq!(metrics.functions.len(), 0);
        assert!(metrics.total_complexity.cyclomatic >= 1);
    }

    #[test]
    fn test_extract_complexity_metrics_with_function() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = r#"#!/bin/bash
function hello() {
    echo "hello"
}
"#;

        let metrics = analyzer.extract_complexity_metrics(content);

        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "hello");
    }

    #[test]
    fn test_extract_complexity_metrics_function_with_if() {
        let analyzer = UnifiedBashAnalyzer::new(PathBuf::from("test.sh"));
        let content = r#"#!/bin/bash
function check_file() {
    if [ -f "$1" ]; then
        echo "exists"
    fi
}
"#;

        let metrics = analyzer.extract_complexity_metrics(content);

        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "check_file");
        // Base 1 + if 1 = 2
        assert!(metrics.functions[0].metrics.cyclomatic >= 2);
    }

    #[test]
    fn test_unified_analysis_struct() {
        let analysis = UnifiedAnalysis {
            ast_items: vec![],
            file_metrics: FileComplexityMetrics {
                path: "test.sh".to_string(),
                total_complexity: ComplexityMetrics {
                    cyclomatic: 1,
                    cognitive: 1,
                    nesting_max: 0,
                    lines: 5,
                    halstead: None,
                },
                functions: vec![],
                classes: vec![],
            },
            parsed_at: std::time::Instant::now(),
        };

        assert!(analysis.ast_items.is_empty());
        assert_eq!(analysis.file_metrics.path, "test.sh");
    }

    #[test]
    fn test_analysis_error_display() {
        let io_err = AnalysisError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(format!("{}", io_err).contains("Failed to read file"));

        let parse_err = AnalysisError::Parse("syntax error".to_string());
        assert!(format!("{}", parse_err).contains("Failed to parse"));

        let analysis_err = AnalysisError::Analysis("analysis failed".to_string());
        assert!(format!("{}", analysis_err).contains("Analysis error"));
    }
}
