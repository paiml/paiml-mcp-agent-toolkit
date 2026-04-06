#![cfg_attr(coverage_nightly, coverage(off))]
//! C/C++ language analysis.

use super::complexity::{find_brace_balanced_end, ComplexityVisitor};
use super::types::{FunctionInfo, LanguageAnalyzer};
use crate::services::complexity::ComplexityMetrics;

/// C language analyzer
pub struct CAnalyzer;

impl LanguageAnalyzer for CAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            // Detect C function declarations
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

impl CAnalyzer {
    /// Check if line is a C function declaration
    /// Pattern: [storage-class] <type> <name>(<params>) {
    /// Examples: int add(int a, int b) {
    ///          static void* malloc(size_t size) {
    ///          extern inline char get_char(void) {
    fn is_function_declaration(&self, line: &str) -> bool {
        // Must contain both '(' and '{'
        if !line.contains('(') || !line.contains('{') {
            return false;
        }

        // Skip preprocessor directives
        if line.starts_with('#') {
            return false;
        }

        // Skip control flow keywords (if, while, for, switch)
        let trimmed = line.trim();
        if trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("while(")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.starts_with("switch ")
            || trimmed.starts_with("switch(")
        {
            return false;
        }

        // Basic pattern: something followed by identifier(params) {
        // This catches most C function declarations
        let has_paren = line.contains('(');
        let has_brace = line.ends_with('{') || line.contains(") {");

        has_paren && has_brace
    }

    /// Extract function name from C function declaration
    /// Handles: int add(int a, int b) {
    ///         static void* malloc(size_t size) {
    ///         extern inline char get_char(void) {
    fn extract_function_name(&self, line: &str) -> Option<String> {
        // Remove storage class specifiers
        let mut cleaned = line.to_string();
        for keyword in &["static ", "extern ", "inline ", "__inline__ "] {
            cleaned = cleaned.replace(keyword, "");
        }

        let cleaned = cleaned.trim();

        // Find the opening parenthesis
        let paren_pos = cleaned.find('(')?;

        // Work backwards from '(' to find the function name
        let before_paren = cleaned.get(..paren_pos).unwrap_or_default();

        // Split by whitespace and get the last token (the function name)
        let tokens: Vec<&str> = before_paren.split_whitespace().collect();

        if tokens.is_empty() {
            return None;
        }

        // Last token before '(' is the function name
        let name = tokens.last()?.trim();

        // Handle pointer syntax: "void* name" -> extract "name"
        let name = if name.starts_with('*') {
            name.trim_start_matches('*')
        } else {
            name
        };

        if name.is_empty() || !self.is_valid_identifier(name) {
            return None;
        }

        Some(name.to_string())
    }

    /// Check if string is a valid C identifier
    fn is_valid_identifier(&self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let first = s.chars().next().expect("internal error");
        if !first.is_alphabetic() && first != '_' {
            return false;
        }

        s.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Find the closing brace of a function
    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        find_brace_balanced_end(lines, start, false)
    }
}
