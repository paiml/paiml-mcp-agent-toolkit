#[cfg(feature = "kotlin-ast")]
impl KotlinAstVisitor {
    /// Creates a new Kotlin AST visitor
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            package_name: String::new(),
            class_count: 0,
            coroutine_count: 0,
        }
    }

    /// Analyzes Kotlin source code and extracts AST items (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_kotlin_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic Kotlin syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_kotlin_syntax(source) {
            return Err("Invalid Kotlin syntax".to_string());
        }

        self.extract_package_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_function_declarations(source)?;
        self.extract_interface_declarations(source)?;
        self.extract_coroutine_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic Kotlin syntax validity (complexity ≤10)
    fn is_valid_kotlin_syntax(&self, source: &str) -> bool {
        let open_braces = source.chars().filter(|&c| c == '{').count();
        let close_braces = source.chars().filter(|&c| c == '}').count();

        // Basic brace matching and no obvious syntax errors
        open_braces == close_braces && !source.contains("!!!")
    }

    /// Extracts package declaration (complexity ≤10)
    fn extract_package_declaration(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(package_part) = trimmed.strip_prefix("package ") {
                self.package_name = package_part.trim().to_string();
                return Ok(());
            }
        }
        Ok(())
    }

    /// Extracts class declarations (complexity ≤10)
    fn extract_class_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(class_name) = self.extract_class_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&class_name);
                let visibility = "public"; // Kotlin classes are public by default
                let fields_count = self.count_class_members(source, &class_name);

                self.items.push(AstItem::Struct {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    fields_count,
                    derives: vec![],
                    line: 1,
                });
                self.class_count += 1;
            }
        }
        Ok(())
    }

    /// Helper to extract class name from line (complexity ≤10)
    fn extract_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i + 1 < parts.len() {
                    // Split at '(' FIRST: a Kotlin primary constructor sits
                    // directly against the class name, so `class Person(val
                    // name: String)` tokenises as `Person(val` and trimming
                    // only `{` and `:` left the parameter list glued on. Every
                    // class with a primary constructor — idiomatic Kotlin — was
                    // reported under a name like `com.example.demo::Person(val`.
                    // `extract_function_name_from_line` already splits at '('
                    // for exactly this reason; this arm did not.
                    let class_name = parts[i + 1]
                        .split('(')
                        .next()?
                        .trim_end_matches('{')
                        .trim_end_matches(':');
                    if class_name.is_empty() {
                        return None;
                    }
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }

    /// Count methods in a class (complexity ≤10)
    fn count_class_members(&self, source: &str, class_name: &str) -> usize {
        let lines: Vec<&str> = source.lines().collect();
        let mut count = 0;
        let mut in_class = false;
        let mut brace_count = 0;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.contains(&format!("class {class_name}")) {
                in_class = true;
                if trimmed.contains('{') {
                    brace_count += 1;
                }
                continue;
            }

            if in_class {
                brace_count += trimmed.chars().filter(|&c| c == '{').count() as i32;
                brace_count -= trimmed.chars().filter(|&c| c == '}').count() as i32;

                if brace_count <= 0 {
                    break;
                }

                if trimmed.contains("fun ") && !trimmed.contains("class") {
                    count += 1;
                }
            }
        }
        count
    }

    /// Extracts function declarations (complexity ≤10)
    fn extract_function_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(function_name) = self.extract_function_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&function_name);
                let visibility = self.extract_function_visibility(trimmed);
                let is_suspend = trimmed.contains("suspend");

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility,
                    is_async: is_suspend,
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract function name from line (complexity ≤10)
    fn extract_function_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("fun ") && line.contains('(') && !line.contains("class") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "fun" && i + 1 < parts.len() {
                    let next_part = parts[i + 1];
                    let function_name = next_part.split('(').next()?;
                    return Some(function_name.to_string());
                }
            }
        }
        None
    }

    /// Helper to extract function visibility (complexity ≤10)
    fn extract_function_visibility(&self, line: &str) -> String {
        if line.contains("public") {
            "public".to_string()
        } else if line.contains("private") {
            "private".to_string()
        } else if line.contains("protected") {
            "protected".to_string()
        } else {
            "public".to_string()
        }
    }

    /// Extracts interface declarations (complexity ≤10)
    fn extract_interface_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(interface_name) = self.extract_interface_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&interface_name);
                let visibility = "public"; // Kotlin classes are public by default

                self.items.push(AstItem::Trait {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract interface name from line (complexity ≤10)
    fn extract_interface_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("interface ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "interface" && i + 1 < parts.len() {
                    let interface_name = parts[i + 1].trim_end_matches('{');
                    return Some(interface_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts coroutine declarations (complexity ≤10)
    fn extract_coroutine_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("suspend fun")
                || trimmed.contains("async {")
                || trimmed.contains("launch {")
            {
                self.coroutine_count += 1;
            }
        }
        Ok(())
    }

    /// Gets qualified name for a symbol (complexity ≤10)
    fn get_qualified_name(&self, name: &str) -> String {
        if self.package_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.package_name, name)
        }
    }
}
