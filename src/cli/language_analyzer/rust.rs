#![cfg_attr(coverage_nightly, coverage(off))]
//! Rust-specific language analysis.

use super::complexity::{find_brace_balanced_end, ComplexityVisitor};
use super::types::{FunctionInfo, LanguageAnalyzer};
use crate::services::complexity::ComplexityMetrics;

/// Rust language analyzer
pub struct RustAnalyzer;

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if self.is_function_declaration(trimmed) {
                if let Some(name) = self.extract_function_name(trimmed) {
                    let line_end = self.find_function_end(&lines, line_num);
                    functions.push(FunctionInfo {
                        name,
                        line_start: line_num,
                        line_end,
                    });
                }
            }
        }

        functions
    }

    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics {
        let lines: Vec<&str> = content.lines().collect();
        let function_lines = &lines[function.line_start..=function.line_end];

        let mut visitor = ComplexityVisitor::new();
        visitor.analyze_lines(function_lines);
        visitor.into_metrics()
    }
}

impl RustAnalyzer {
    fn is_function_declaration(&self, line: &str) -> bool {
        line.starts_with("fn ")
            || line.starts_with("pub fn ")
            || line.starts_with("async fn ")
            || line.starts_with("pub async fn ")
            || line.starts_with("pub(crate) fn ")
            || line.starts_with("pub(super) fn ")
            || line.starts_with("pub(in ") && line.contains(") fn ")
    }

    fn extract_function_name(&self, line: &str) -> Option<String> {
        let line = line.trim();
        if let Some(fn_pos) = line.find("fn ") {
            let after_fn = line.get(fn_pos + 3..).unwrap_or_default();
            if let Some(paren_pos) = after_fn.find('(') {
                let name = after_fn.get(..paren_pos).unwrap_or_default().trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        find_brace_balanced_end(lines, start, true)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_function_extraction() {
        let analyzer = RustAnalyzer;
        let content = r#"
/// Test function.
pub fn test_function() {
    println!("Hello");
}

async fn async_function() {
    // Some async code
}
"#;

        let functions = analyzer.extract_functions(content);
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "test_function");
        assert_eq!(functions[1].name, "async_function");
    }
}
