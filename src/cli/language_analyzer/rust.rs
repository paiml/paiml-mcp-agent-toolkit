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
    /// True when the trimmed line opens a function definition.
    ///
    /// The previous version enumerated seven literal prefixes, so
    /// `pub(crate) async fn`, `const fn`, `unsafe fn` and `extern "C" fn` were
    /// silently invisible — one reason `analyze big-o` "missed" real functions
    /// (#655). Qualifiers are now stripped in any order before checking for the
    /// `fn` keyword. Comments still cannot match, because a comment line trims
    /// to `//…` and never reduces to `fn …`.
    fn is_function_declaration(&self, line: &str) -> bool {
        let rest = Self::strip_visibility(line.trim());
        Self::strip_fn_qualifiers(rest).starts_with("fn ")
    }

    /// Remove a leading `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`.
    fn strip_visibility(line: &str) -> &str {
        let Some(after_pub) = line.strip_prefix("pub") else {
            return line;
        };
        match after_pub.chars().next() {
            Some('(') => match after_pub.find(')') {
                Some(close) => after_pub.get(close + 1..).unwrap_or("").trim_start(),
                None => line,
            },
            Some(c) if c.is_whitespace() => after_pub.trim_start(),
            _ => line,
        }
    }

    /// Remove leading `const` / `async` / `unsafe` / `default` / `extern "ABI"`.
    fn strip_fn_qualifiers(line: &str) -> &str {
        let mut rest = line;
        loop {
            let before = rest;
            for keyword in ["default ", "const ", "async ", "unsafe "] {
                if let Some(stripped) = rest.strip_prefix(keyword) {
                    rest = stripped.trim_start();
                }
            }
            rest = Self::strip_extern_abi(rest);
            if rest.len() == before.len() {
                return rest;
            }
        }
    }

    fn strip_extern_abi(line: &str) -> &str {
        let Some(after) = line.strip_prefix("extern") else {
            return line;
        };
        let after = after.trim_start();
        let Some(quoted) = after.strip_prefix('"') else {
            return after;
        };
        match quoted.find('"') {
            Some(close) => quoted.get(close + 1..).unwrap_or("").trim_start(),
            None => line,
        }
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
