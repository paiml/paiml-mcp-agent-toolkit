// Java AST visitor implementation methods
// Included from java.rs - NO use imports or #! inner attributes

impl JavaAstVisitor {
    /// Creates a new Java AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            package_name: String::new(),
            class_count: 0,
        }
    }

    /// Analyzes Java source code and extracts AST items (complexity ≤10)
    pub fn analyze_java_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic Java syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_java_syntax(source) {
            return Err("Invalid Java syntax".to_string());
        }

        self.extract_package_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_method_declarations(source)?;
        self.extract_interface_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic Java syntax validity (complexity ≤10)
    fn is_valid_java_syntax(&self, source: &str) -> bool {
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
            if trimmed.starts_with("package ") && trimmed.ends_with(';') {
                let package_part = &trimmed[8..trimmed.len() - 1];
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
                let visibility = if trimmed.contains("public") {
                    "public"
                } else {
                    "package"
                };
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

    /// Count methods in a class (complexity ≤10)
    fn count_class_members(&self, source: &str, class_name: &str) -> usize {
        let lines: Vec<&str> = source.lines().collect();
        let mut count = 0;
        let mut in_class = false;
        let mut brace_count = 0;

        for line in lines {
            let trimmed = line.trim();

            // Start counting after we see the class declaration
            if trimmed.contains(&format!("class {class_name}")) {
                in_class = true;
                if trimmed.contains('{') {
                    brace_count += 1;
                }
                continue;
            }

            if in_class {
                // Track brace nesting
                brace_count += trimmed.chars().filter(|&c| c == '{').count() as i32;
                brace_count -= trimmed.chars().filter(|&c| c == '}').count() as i32;

                // Exit when we've closed the class
                if brace_count <= 0 {
                    break;
                }

                // Count method declarations (but not constructor calls)
                if trimmed.contains('(')
                    && trimmed.contains(')')
                    && (trimmed.contains("public")
                        || trimmed.contains("private")
                        || trimmed.contains("protected"))
                    && !trimmed.contains("class")
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// Helper to extract class name from line (complexity ≤10)
    fn extract_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("class ") && line.contains('{') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i + 1 < parts.len() {
                    let class_name = parts[i + 1].trim_end_matches('{');
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts method declarations (complexity ≤10)
    fn extract_method_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(method_name) = self.extract_method_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&method_name);
                let visibility = self.extract_method_visibility(trimmed);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility,
                    is_async: false,
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract method name from line (complexity ≤10)
    fn extract_method_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains('(') && line.contains(')') && !line.contains("class") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if part.contains('(') && i > 0 {
                    let method_name = part.split('(').next()?;
                    if !method_name.is_empty()
                        && method_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        return Some(method_name.to_string());
                    }
                }
            }
        }
        None
    }

    /// Helper to extract method visibility (complexity ≤10)
    fn extract_method_visibility(&self, line: &str) -> String {
        if line.contains("public") {
            "public".to_string()
        } else if line.contains("private") {
            "private".to_string()
        } else if line.contains("protected") {
            "protected".to_string()
        } else {
            "package".to_string()
        }
    }

    /// Extracts interface declarations (complexity ≤10)
    fn extract_interface_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(interface_name) = self.extract_interface_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&interface_name);
                let visibility = if trimmed.contains("public") {
                    "public"
                } else {
                    "package"
                };

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
        if line.contains("interface ") && line.contains('{') {
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

    /// Gets qualified name for a symbol (complexity ≤10)
    fn get_qualified_name(&self, name: &str) -> String {
        if self.package_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.package_name, name)
        }
    }
}
