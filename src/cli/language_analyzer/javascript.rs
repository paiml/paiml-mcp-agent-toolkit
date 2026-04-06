#![cfg_attr(coverage_nightly, coverage(off))]
//! JavaScript/TypeScript language analysis.

use super::complexity::{find_brace_balanced_end, ComplexityVisitor};
use super::types::{FunctionInfo, LanguageAnalyzer};
use crate::services::complexity::ComplexityMetrics;

/// JavaScript/TypeScript analyzer
pub struct JavaScriptAnalyzer;

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Track class context for method name qualification
        let mut current_class: Option<String> = None;
        let mut class_brace_depth = 0;
        let mut global_brace_depth = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track class declarations
            if let Some(class_name) = self.extract_class_name(trimmed) {
                current_class = Some(class_name);
                class_brace_depth = global_brace_depth + 1;
            }

            // Track brace depth to know when we exit class
            for ch in line.chars() {
                match ch {
                    '{' => global_brace_depth += 1,
                    '}' => {
                        global_brace_depth -= 1;
                        // Exit class when we close its braces
                        if current_class.is_some() && global_brace_depth < class_brace_depth {
                            current_class = None;
                        }
                    }
                    _ => {}
                }
            }

            // Detect class methods
            if let Some(class_name) = &current_class {
                if let Some(method_name) = self.extract_method_name(trimmed) {
                    let line_end = self.find_function_end(&lines, line_num);
                    let qualified_name = format!("{}::{}", class_name, method_name);
                    functions.push(FunctionInfo {
                        name: qualified_name,
                        line_start: line_num,
                        line_end,
                    });
                    continue;
                }
            }

            // Detect regular function declarations
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
    /// Extract class name from class declaration
    ///
    /// Detects: `class Name`, `export class Name`, `export default class Name`
    fn extract_class_name(&self, line: &str) -> Option<String> {
        let patterns = ["export default class ", "export class ", "class "];

        for pattern in &patterns {
            if let Some(pos) = line.find(pattern) {
                let after = line.get(pos + pattern.len()..).unwrap_or_default();
                // Extract until space or {
                if let Some(end) = after.find(|c: char| c.is_whitespace() || c == '{') {
                    let name = after.get(..end).unwrap_or_default().trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract method name from class method declaration
    ///
    /// Detects:
    /// - Regular methods: `methodName(params) {`
    /// - Async methods: `async methodName(params) {`
    /// - Static methods: `static methodName(params) {`
    /// - Constructors: `constructor(params) {`
    /// - Getters/Setters: `get propertyName()`, `set propertyName(value)`
    fn extract_method_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Skip non-method lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            return None;
        }

        // Skip property declarations (e.g., `private name: string;`)
        if !trimmed.contains('(') {
            return None;
        }

        // Handle: static methodName(
        if let Some(after) = trimmed.strip_prefix("static ") {
            // Skip "static "
            return self
                .extract_simple_method_name(after)
                .map(|n| format!("static {}", n));
        }

        // Handle: async methodName(
        if let Some(after) = trimmed.strip_prefix("async ") {
            // Skip "async "
            return self.extract_simple_method_name(after);
        }

        // Handle: get propertyName() or set propertyName(value)
        if let Some(after) = trimmed.strip_prefix("get ") {
            return self.extract_simple_method_name(after);
        }
        if let Some(after) = trimmed.strip_prefix("set ") {
            return self.extract_simple_method_name(after);
        }

        // Handle: constructor(
        if trimmed.starts_with("constructor(") || trimmed.starts_with("constructor (") {
            return Some("constructor".to_string());
        }

        // Handle: methodName( or methodName (
        self.extract_simple_method_name(trimmed)
    }

    /// Extract simple method name from pattern: `methodName(params)`
    fn extract_simple_method_name(&self, text: &str) -> Option<String> {
        if let Some(paren_pos) = text.find('(') {
            let before_paren = &text.get(..paren_pos).unwrap_or_default().trim();
            // Extract last word before '('
            if let Some(last_word_start) = before_paren.rfind(|c: char| c.is_whitespace()) {
                let name = before_paren
                    .get(last_word_start..)
                    .unwrap_or_default()
                    .trim();
                if !name.is_empty()
                    && name
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_alphabetic() || c == '_')
                {
                    return Some(name.to_string());
                }
            } else if !before_paren.is_empty()
                && before_paren
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
            {
                return Some(before_paren.to_string());
            }
        }
        None
    }

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
            let after = line.get(pos + 9..).unwrap_or_default();
            if let Some(paren_pos) = after.find('(') {
                let name = after.get(..paren_pos).unwrap_or_default().trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        // Handle: const/let/var name =
        for keyword in &["const ", "let ", "var "] {
            if let Some(pos) = line.find(keyword) {
                let after = line.get(pos + keyword.len()..).unwrap_or_default();
                if let Some(eq_pos) = after.find(" = ") {
                    let name = after.get(..eq_pos).unwrap_or_default().trim();
                    return Some(name.to_string());
                }
            }
        }

        // For anonymous functions, use generic name
        Some("anonymous_fn".to_string())
    }

    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        find_brace_balanced_end(lines, start, false)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// RED TEST (PMAT-BUG-001): TypeScript/JavaScript class methods must be extracted
    ///
    /// **BUG**: JavaScriptAnalyzer uses regex/heuristic parsing that ONLY detects:
    /// - `function name()` declarations
    /// - Arrow functions `const x = () => {}`
    /// - Variable assignments to functions
    ///
    /// But it does NOT detect:
    /// - Class methods (e.g., `add(a, b) { ... }` inside a class)
    /// - Constructors (e.g., `constructor() { ... }`)
    /// - Static methods (e.g., `static create() { ... }`)
    ///
    /// **ROOT CAUSE**: CLI uses `JavaScriptAnalyzer` (heuristics) instead of
    /// `EnhancedTypeScriptVisitor` (full AST analysis).
    ///
    /// **EXPECTED**: After fix, this test must PASS.
    /// **ACTUAL**: Currently FAILS because class methods return empty `[]`.
    ///
    /// **FIX STRATEGY**:
    /// 1. Modify `JavaScriptAnalyzer::extract_functions()` to detect class methods
    /// 2. Add regex patterns for: `methodName(params)`, `constructor(params)`, `static methodName(params)`
    /// 3. Track class context using brace counting
    /// 4. Qualify method names with class name (e.g., `Calculator::add`)
    ///
    /// **Quality Gate**: This test must pass before v2.162.0 release.
    #[test]
    fn red_test_typescript_class_methods_must_be_extracted() {
        let analyzer = JavaScriptAnalyzer;
        let content = r#"
export class Calculator {
    add(a: number, b: number): number {
        return a + b;
    }

    divide(a: number, b: number): number {
        if (b === 0) {
            throw new Error("Division by zero");
        }
        return a / b;
    }

    constructor(private name: string) {}
}
"#;

        let functions = analyzer.extract_functions(content);

        // RED: This assertion WILL FAIL until the fix is implemented
        assert!(
            functions.len() >= 3,
            "PMAT-BUG-001: JavaScriptAnalyzer must extract class methods. \
             Expected >=3 (add, divide, constructor), found {}. \
             Functions: {:?}",
            functions.len(),
            functions.iter().map(|f| &f.name).collect::<Vec<_>>()
        );

        // Verify specific method names are detected
        let method_names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(
            method_names.iter().any(|n| n.contains("add")),
            "Must detect 'add' method"
        );
        assert!(
            method_names.iter().any(|n| n.contains("divide")),
            "Must detect 'divide' method"
        );
        assert!(
            method_names.iter().any(|n| n.contains("constructor")),
            "Must detect 'constructor' method"
        );
    }

    /// RED TEST (PMAT-BUG-001): JavaScript class methods must be extracted
    ///
    /// Same bug affects JavaScript ES6 classes. This test validates plain JavaScript
    /// class syntax without TypeScript types.
    ///
    /// **Quality Gate**: Must pass before v2.162.0 release.
    #[test]
    fn red_test_javascript_class_methods_must_be_extracted() {
        debug_assert!(
            true,
            "contract: red_test_javascript_class_methods_must_be_extracted"
        );
        let analyzer = JavaScriptAnalyzer;
        let content = r#"
class Server {
    constructor(port) {
        this.port = port;
    }

    start() {
        console.log(`Starting on port ${this.port}`);
    }

    stop() {
        console.log('Stopping server');
    }

    static create(port) {
        return new Server(port);
    }
}
"#;

        let functions = analyzer.extract_functions(content);

        // RED: This assertion WILL FAIL until the fix is implemented
        assert!(
            functions.len() >= 4,
            "PMAT-BUG-001: JavaScriptAnalyzer must extract class methods. \
             Expected >=4 (constructor, start, stop, static create), found {}. \
             Functions: {:?}",
            functions.len(),
            functions.iter().map(|f| &f.name).collect::<Vec<_>>()
        );

        // Verify specific method names
        let method_names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(
            method_names.iter().any(|n| n.contains("constructor")),
            "Must detect 'constructor'"
        );
        assert!(
            method_names.iter().any(|n| n.contains("start")),
            "Must detect 'start' method"
        );
        assert!(
            method_names.iter().any(|n| n.contains("stop")),
            "Must detect 'stop' method"
        );
        assert!(
            method_names.iter().any(|n| n.contains("create")),
            "Must detect static 'create' method"
        );
    }
}

include!("javascript_property_tests.rs");
