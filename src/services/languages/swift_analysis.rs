// SwiftSourceAnalyzer implementation methods
// Provides lexical analysis and partial AST extraction for Swift source files.

impl SwiftSourceAnalyzer {
    /// Creates a new Swift source analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            source_name: file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            class_count: 0,
            method_count: 0,
        }
    }

    /// Analyzes Swift source and extracts AST items (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_swift_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_functions(source)?;
        self.extract_classes(source)?;
        self.extract_methods(source)?;

        Ok(self.items)
    }

    /// Extracts function definitions from Swift source (complexity ≤10)
    fn extract_functions(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: func functionName(...) {
            // Skip class/struct methods (will be extracted separately)
            if trimmed.starts_with("func ") && trimmed.contains('(') {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&func_name);

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: self.extract_visibility(trimmed),
                        is_async: trimmed.contains("async"),
                        line: line_num + 1,
                    });
                    self.function_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts class definitions from Swift source (complexity ≤10)
    fn extract_classes(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: class ClassName {
            // Match: struct StructName {
            if (trimmed.starts_with("class ") || trimmed.starts_with("struct "))
                && trimmed.contains('{')
            {
                if let Some(class_name) = self.extract_class_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&class_name);

                    self.items.push(AstItem::Struct {
                        name: qualified_name,
                        visibility: self.extract_visibility(trimmed),
                        fields_count: 0, // Swift field extraction not implemented yet
                        derives: vec![], // Swift doesn't have derives
                        line: line_num + 1,
                    });
                    self.class_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts method definitions from Swift classes (complexity ≤10)
    fn extract_methods(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut in_class = false;
        let mut brace_count = 0;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Track class/struct context
            if trimmed.starts_with("class ") || trimmed.starts_with("struct ") {
                in_class = true;
                brace_count = 0;
            }

            // Count braces to track nesting
            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;

            // Exit class context when braces balance
            if in_class && brace_count == 0 && trimmed.contains('}') {
                in_class = false;
            }

            // Extract methods inside classes
            if in_class && trimmed.starts_with("func ") && trimmed.contains('(') {
                if let Some(method_name) = self.extract_function_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&method_name);

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: self.extract_visibility(trimmed),
                        is_async: trimmed.contains("async"),
                        line: line_num + 1,
                    });
                    self.method_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts function name from Swift line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // func functionName(...) {
        let after_func = line.strip_prefix("func ")?.trim();

        // Handle private/public/internal modifiers
        let after_func = if let Some(stripped) = after_func.strip_prefix("private ") {
            stripped
        } else if let Some(stripped) = after_func.strip_prefix("public ") {
            stripped
        } else if let Some(stripped) = after_func.strip_prefix("internal ") {
            stripped
        } else {
            after_func
        };

        let name_part = after_func.split('(').next()?;
        Some(name_part.trim().to_string())
    }

    /// Extracts class name from Swift line (complexity ≤10)
    fn extract_class_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // class ClassName {
        // struct StructName {
        let after_keyword = if let Some(stripped) = line.strip_prefix("class ") {
            stripped
        } else if let Some(stripped) = line.strip_prefix("struct ") {
            stripped
        } else {
            return None;
        };

        let name_part = after_keyword
            .split_whitespace()
            .next()?
            .trim_end_matches('{')
            .trim_end_matches(':');
        Some(name_part.trim().to_string())
    }

    /// Extracts visibility from Swift line (complexity ≤10)
    fn extract_visibility(&self, line: &str) -> String {
        debug_assert!(!line.is_empty(), "line must not be empty");
        if line.contains("private ") {
            "private".to_string()
        } else if line.contains("public ") {
            "public".to_string()
        } else if line.contains("internal ") {
            "internal".to_string()
        } else {
            "internal".to_string() // Swift default visibility
        }
    }

    /// Gets qualified name for Swift symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        debug_assert!(!symbol_name.is_empty(), "symbol_name must not be empty");
        if self.source_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.source_name, symbol_name)
        }
    }
}
