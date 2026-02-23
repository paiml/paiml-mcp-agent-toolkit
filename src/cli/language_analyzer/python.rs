#![cfg_attr(coverage_nightly, coverage(off))]
//! Python language analysis.

use super::complexity::ComplexityVisitor;
use super::types::{FunctionInfo, LanguageAnalyzer};
use crate::services::complexity::ComplexityMetrics;

/// Python analyzer
pub struct PythonAnalyzer;

impl LanguageAnalyzer for PythonAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
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

impl PythonAnalyzer {
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let line = line.trim();
        if let Some(pos) = line.find("def ") {
            let after = line.get(pos + 4..).unwrap_or_default();
            if let Some(paren_pos) = after.find('(') {
                let name = after.get(..paren_pos).unwrap_or_default().trim();
                return Some(name.to_string());
            }
        }
        None
    }

    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        if lines.is_empty() || start >= lines.len() {
            return start;
        }

        // Get indentation level of function definition
        let def_indent = lines[start].len() - lines[start].trim_start().len();

        // Find next line with same or lower indentation
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            if !line.trim().is_empty() {
                let indent = line.len() - line.trim_start().len();
                if indent <= def_indent {
                    return i - 1;
                }
            }
        }

        lines.len() - 1
    }
}
