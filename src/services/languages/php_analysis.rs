// PHP script analyzer implementation methods
// Included from php.rs - shares parent module scope

impl PhpScriptAnalyzer {
    /// Creates a new PHP script analyzer
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            script_name: file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            class_count: 0,
            method_count: 0,
        }
    }

    /// Analyzes PHP script and extracts AST items (complexity ≤10)
    pub fn analyze_php_script(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_functions(source)?;
        self.extract_classes(source)?;
        self.extract_methods(source)?;

        Ok(self.items)
    }

    /// Extracts function definitions from PHP script (complexity ≤10)
    fn extract_functions(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: function functionName(...) {
            if trimmed.starts_with("function ") && trimmed.contains('(') {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&func_name);

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        is_async: false,
                        line: line_num + 1,
                    });
                    self.function_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts class definitions from PHP script (complexity ≤10)
    fn extract_classes(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Match: class ClassName {
            if trimmed.starts_with("class ") && trimmed.contains('{') {
                if let Some(class_name) = self.extract_class_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&class_name);

                    self.items.push(AstItem::Struct {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        fields_count: 0, // PHP class field extraction not implemented yet
                        derives: vec![], // PHP doesn't have derives
                        line: line_num + 1,
                    });
                    self.class_count += 1;
                }
            }
        }
        Ok(())
    }

    /// Extracts method definitions from PHP classes (complexity ≤10)
    fn extract_methods(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut current_class: Option<String> = None;
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            // Track current class scope
            if trimmed.starts_with("class ") && trimmed.contains('{') {
                current_class = self.extract_class_name(trimmed);
            }
            if let Some(item) = parse_php_method_line(trimmed, line_num, &current_class) {
                self.items.push(item);
                self.method_count += 1;
            }
        }
        Ok(())
    }

    /// Extracts function name from PHP line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // function functionName(...) {
        let after_function = line.strip_prefix("function ")?.trim();
        let name_part = after_function.split('(').next()?;
        Some(name_part.trim().to_string())
    }

    /// Extracts class name from PHP line (complexity ≤10)
    fn extract_class_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // class ClassName {
        let after_class = line.strip_prefix("class ")?.trim();
        let name_part = after_class.split_whitespace().next()?.trim_end_matches('{');
        Some(name_part.trim().to_string())
    }

    /// Extracts method name from PHP line (complexity ≤10)
    fn extract_method_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // public/private/protected function methodName(...) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "function" {
            let name_part = parts[2].split('(').next()?;
            Some(name_part.trim().to_string())
        } else {
            None
        }
    }

    /// Gets qualified name for PHP symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        debug_assert!(!symbol_name.is_empty(), "symbol_name must not be empty");
        if self.script_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.script_name, symbol_name)
        }
    }
}

/// Parse a single PHP source line into a method AstItem if it matches a method declaration
fn parse_php_method_line(trimmed: &str, line_num: usize, current_class: &Option<String>) -> Option<AstItem> {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    let is_method = (trimmed.starts_with("public function ")
        || trimmed.starts_with("private function ")
        || trimmed.starts_with("protected function "))
        && trimmed.contains('(');

    if !is_method {
        return None;
    }

    let after_vis = trimmed
        .strip_prefix("public function ")
        .or_else(|| trimmed.strip_prefix("private function "))
        .or_else(|| trimmed.strip_prefix("protected function "))?;
    let method_name = after_vis.split('(').next()?.trim().to_string();

    let qualified_name = match current_class {
        Some(class) => format!("{}::{}", class, method_name),
        None => method_name,
    };

    let visibility = if trimmed.starts_with("private") {
        "private"
    } else if trimmed.starts_with("protected") {
        "protected"
    } else {
        "public"
    };

    Some(AstItem::Function {
        name: qualified_name,
        visibility: visibility.to_string(),
        is_async: false,
        line: line_num + 1,
    })
}
