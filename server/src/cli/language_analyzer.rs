//! Language-specific complexity analysis module
//!
//! This module provides proper separation of concerns for analyzing
//! complexity across different programming languages, following the
//! Toyota Way principle of quality and single responsibility.

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use anyhow::Result;
use std::path::Path;

/// Supported programming languages for complexity analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("js" | "jsx") => Language::JavaScript,
            Some("ts" | "tsx") => Language::TypeScript,
            Some("py") => Language::Python,
            _ => Language::Unknown,
        }
    }
}

/// Language-specific analyzer trait
pub trait LanguageAnalyzer {
    /// Extract functions from source code
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo>;

    /// Estimate complexity for a function
    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics;
}

/// Information about a detected function
pub struct FunctionInfo {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
}

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
            let after_fn = &line[fn_pos + 3..];
            if let Some(paren_pos) = after_fn.find('(') {
                let name = after_fn[..paren_pos].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        let mut brace_count = 0;
        let mut found_first_brace = false;

        for (i, line) in lines.iter().enumerate().skip(start) {
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_count += 1;
                        found_first_brace = true;
                    }
                    '}' => {
                        brace_count -= 1;
                        if found_first_brace && brace_count == 0 {
                            return i;
                        }
                    }
                    _ => {}
                }
            }
        }

        lines.len() - 1
    }
}

/// JavaScript/TypeScript analyzer
pub struct JavaScriptAnalyzer;

impl LanguageAnalyzer for JavaScriptAnalyzer {
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

impl JavaScriptAnalyzer {
    fn is_function_declaration(&self, line: &str) -> bool {
        line.starts_with("function ")
            || line.starts_with("async function ")
            || line.starts_with("export function ")
            || line.starts_with("export async function ")
            || line.starts_with("export default function ")
            || line.contains("= function")
            || line.contains("= async function")
            || (line.contains("const ") && line.contains(" = ("))
            || (line.contains("let ") && line.contains(" = ("))
            || (line.contains("var ") && line.contains(" = ("))
            || (line.contains("export const ") && line.contains(" = ("))
            || line.contains(" => {")
    }

    fn extract_function_name(&self, line: &str) -> Option<String> {
        // Handle: function name(
        if let Some(pos) = line.find("function ") {
            let after = &line[pos + 9..];
            if let Some(paren_pos) = after.find('(') {
                let name = after[..paren_pos].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        // Handle: const/let/var name =
        for keyword in &["const ", "let ", "var "] {
            if let Some(pos) = line.find(keyword) {
                let after = &line[pos + keyword.len()..];
                if let Some(eq_pos) = after.find(" = ") {
                    let name = after[..eq_pos].trim();
                    return Some(name.to_string());
                }
            }
        }

        // For anonymous functions, use generic name
        Some("anonymous_fn".to_string())
    }

    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        let mut brace_count = 0;
        let mut found_first_brace = false;

        for (i, line) in lines.iter().enumerate().skip(start) {
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_count += 1;
                        found_first_brace = true;
                    }
                    '}' => {
                        brace_count -= 1;
                        if found_first_brace && brace_count == 0 {
                            return i;
                        }
                    }
                    _ => {}
                }
            }
        }

        lines.len() - 1
    }
}

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
            let after = &line[pos + 4..];
            if let Some(paren_pos) = after.find('(') {
                let name = after[..paren_pos].trim();
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

/// Complexity visitor for analyzing code metrics
struct ComplexityVisitor {
    cyclomatic: u16,
    cognitive: u16,
    nesting: u8,
    max_nesting: u8,
    lines: u16,
}

impl ComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic: 1, // Base complexity
            cognitive: 0,
            nesting: 0,
            max_nesting: 0,
            lines: 0,
        }
    }

    fn analyze_lines(&mut self, lines: &[&str]) {
        self.lines = lines.len() as u16;

        for line in lines {
            let trimmed = line.trim();

            // Count control flow keywords
            if self.is_control_flow(trimmed) {
                self.cyclomatic += 1;
                self.cognitive += 1 + u16::from(self.nesting);
            }

            if trimmed.contains("else") {
                self.cyclomatic += 1;
                self.cognitive += 1;
            }

            // Track nesting
            if trimmed.ends_with('{') || trimmed.ends_with(':') {
                self.nesting += 1;
                self.max_nesting = self.max_nesting.max(self.nesting);
            }
            if trimmed.starts_with('}') || (trimmed.is_empty() && self.nesting > 0) {
                self.nesting = self.nesting.saturating_sub(1);
            }
        }
    }

    fn is_control_flow(&self, line: &str) -> bool {
        line.contains("if ")
            || line.contains("while ")
            || line.contains("for ")
            || line.contains("match ")
            || line.contains("switch ")
            || line.contains("case ")
            || line.contains("elif ")
            || line.contains("except ")
            || line.contains("catch ")
    }

    fn into_metrics(self) -> ComplexityMetrics {
        ComplexityMetrics {
            cyclomatic: self.cyclomatic.min(255),
            cognitive: self.cognitive.min(255),
            nesting_max: self.max_nesting,
            lines: self.lines,
            halstead: None,
        }
    }
}

/// Analyze file complexity using appropriate language analyzer
pub async fn analyze_file_complexity(path: &Path, content: &str) -> Result<FileComplexityMetrics> {
    let language = Language::from_path(path);

    // Try AST analysis first for Rust files
    if let Some(metrics) = try_ast_analysis(path, language).await {
        return Ok(metrics);
    }

    // Fall back to heuristic analysis
    analyze_with_heuristics(path, content, language)
}

async fn try_ast_analysis(path: &Path, language: Language) -> Option<FileComplexityMetrics> {
    if language != Language::Rust {
        return None;
    }

    match crate::services::ast_rust::analyze_rust_file_with_complexity(path).await {
        Ok(metrics) => Some(metrics),
        Err(_) => {
            eprintln!(
                "Warning: AST analysis failed for {}, using heuristic fallback",
                path.display()
            );
            None
        }
    }
}

fn analyze_with_heuristics(
    path: &Path,
    content: &str,
    language: Language,
) -> Result<FileComplexityMetrics> {
    match language {
        Language::Unknown => Ok(create_empty_metrics(path, content)),
        _ => {
            let analyzer = create_analyzer(language);
            analyze_functions_with_analyzer(path, content, &*analyzer)
        }
    }
}

fn create_empty_metrics(path: &Path, content: &str) -> FileComplexityMetrics {
    FileComplexityMetrics {
        path: path.to_string_lossy().to_string(),
        total_complexity: ComplexityMetrics {
            cyclomatic: 1,
            cognitive: 0,
            nesting_max: 0,
            lines: content.lines().count() as u16,
            halstead: None,
        },
        functions: vec![],
        classes: vec![],
    }
}

fn create_analyzer(language: Language) -> Box<dyn LanguageAnalyzer> {
    match language {
        Language::Rust => Box::new(RustAnalyzer),
        Language::JavaScript | Language::TypeScript => Box::new(JavaScriptAnalyzer),
        Language::Python => Box::new(PythonAnalyzer),
        Language::Unknown => unreachable!("Unknown language should be handled earlier"),
    }
}

fn analyze_functions_with_analyzer(
    path: &Path,
    content: &str,
    analyzer: &dyn LanguageAnalyzer,
) -> Result<FileComplexityMetrics> {
    let function_infos = analyzer.extract_functions(content);
    let functions = process_function_infos(content, function_infos, analyzer);
    let total_complexity = calculate_total_complexity(&functions, content);

    Ok(FileComplexityMetrics {
        path: path.to_string_lossy().to_string(),
        total_complexity,
        functions,
        classes: vec![],
    })
}

fn process_function_infos(
    content: &str,
    function_infos: Vec<FunctionInfo>,
    analyzer: &dyn LanguageAnalyzer,
) -> Vec<FunctionComplexity> {
    function_infos
        .into_iter()
        .map(|info| {
            let metrics = analyzer.estimate_complexity(content, &info);
            FunctionComplexity {
                name: info.name,
                line_start: (info.line_start + 1) as u32,
                line_end: (info.line_end + 1) as u32,
                metrics,
            }
        })
        .collect()
}

fn calculate_total_complexity(
    functions: &[FunctionComplexity],
    content: &str,
) -> ComplexityMetrics {
    ComplexityMetrics {
        cyclomatic: functions
            .iter()
            .map(|f| f.metrics.cyclomatic)
            .sum::<u16>()
            .max(1),
        cognitive: functions
            .iter()
            .map(|f| f.metrics.cognitive)
            .sum::<u16>()
            .max(1),
        nesting_max: functions
            .iter()
            .map(|f| f.metrics.nesting_max)
            .max()
            .unwrap_or(0),
        lines: content.lines().count() as u16,
        halstead: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_path(Path::new("test.rs")), Language::Rust);
        assert_eq!(
            Language::from_path(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            Language::from_path(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(Language::from_path(Path::new("test.py")), Language::Python);
        assert_eq!(
            Language::from_path(Path::new("test.txt")),
            Language::Unknown
        );
    }

    #[test]
    fn test_rust_function_extraction() {
        let analyzer = RustAnalyzer;
        let content = r#"
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

    #[test]
    fn test_complexity_visitor() {
        let mut visitor = ComplexityVisitor::new();
        let lines = vec![
            "fn test() {",
            "    if condition {",
            "        while true {",
            "            break;",
            "        }",
            "    }",
            "}",
        ];

        visitor.analyze_lines(&lines);
        let metrics = visitor.into_metrics();

        assert!(metrics.cyclomatic > 1);
        assert!(metrics.cognitive > 0);
        assert_eq!(metrics.nesting_max, 3);
    }

    /// TDD Test: Integration test to expose the real bug
    #[tokio::test]
    async fn test_end_to_end_integration_bug() {
        let content = r#"fn simple_function() {
    println!("hello");
}

pub fn second_function() {
    if true {
        println!("world");
    }
}
"#;
        let path = Path::new("test.rs");

        // Test the RustAnalyzer directly first
        let analyzer = RustAnalyzer;
        let functions = analyzer.extract_functions(content);
        assert_eq!(functions.len(), 2, "RustAnalyzer should detect 2 functions");

        // Test the full integration
        let result = analyze_file_complexity(path, content).await;
        assert!(
            result.is_ok(),
            "analyze_file_complexity should succeed: {:?}",
            result
        );

        let metrics = result.unwrap();

        // THIS MIGHT FAIL - if it does, we found the integration bug
        assert_eq!(
            metrics.functions.len(),
            2,
            "Integration should analyze 2 functions but found {}. Functions: {:?}",
            metrics.functions.len(),
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// TDD Test: CLI Layer integration test using analyze_project_files
    #[tokio::test]
    async fn test_cli_layer_integration_bug() {
        use std::fs;
        use tempfile::TempDir;

        let content = r#"fn simple_function() {
    println!("hello");
}

pub fn second_function() {
    if true {
        println!("world");
    }
}
"#;

        // Create a temporary directory and file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, content).unwrap();

        // Test the CLI stubs layer using analyze_project_files
        let result = crate::cli::analysis_utilities::analyze_project_files(
            temp_dir.path(),
            Some("rust"),
            &[], // empty include patterns
            20,  // cyclomatic threshold
            15,  // cognitive threshold
        )
        .await;

        assert!(
            result.is_ok(),
            "analyze_project_files should succeed: {:?}",
            result
        );

        let file_metrics = result.unwrap();

        // Find our test file
        let test_metrics = file_metrics
            .iter()
            .find(|metrics| metrics.path.ends_with("test.rs"))
            .expect("Should find test.rs in results");

        // THIS SHOULD EXPOSE THE BUG
        assert_eq!(
            test_metrics.functions.len(),
            2,
            "CLI layer should analyze 2 functions but found {}. Functions: {:?}",
            test_metrics.functions.len(),
            test_metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }
}
