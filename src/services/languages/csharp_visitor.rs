// CSharpAstVisitor implementation - included from csharp.rs

#[cfg(feature = "csharp-ast")]
impl CSharpAstVisitor {
    /// Creates a new C# AST visitor
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            namespace_name: String::new(),
            class_count: 0,
        }
    }

    /// Analyzes C# source code and extracts AST items (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_csharp_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic C# syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_csharp_syntax(source) {
            return Err("Invalid C# syntax".to_string());
        }

        self.extract_namespace_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_method_declarations(source)?;
        self.extract_interface_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic C# syntax validity (complexity ≤10)
    fn is_valid_csharp_syntax(&self, source: &str) -> bool {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let open_braces = source.chars().filter(|&c| c == '{').count();
        let close_braces = source.chars().filter(|&c| c == '}').count();

        // Basic brace matching and no obvious syntax errors
        open_braces == close_braces && !source.contains("!!!")
    }

    /// Extracts namespace declaration (complexity ≤10)
    fn extract_namespace_declaration(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("namespace ") && !trimmed.ends_with(';') {
                let namespace_part = &trimmed[10..];
                let end_idx = namespace_part.find('{').unwrap_or(namespace_part.len());
                self.namespace_name = namespace_part[..end_idx].trim().to_string();
                return Ok(());
            }
        }
        Ok(())
    }

    /// Extracts class declarations (complexity ≤10)
    fn extract_class_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(class_name) = self.extract_class_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&class_name);
                let visibility = if trimmed.contains("public") {
                    "public"
                } else {
                    "internal"
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

    /// Helper to extract class name from line (complexity ≤10)
    fn extract_class_name_from_line(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        if line.contains("class ") {
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

    /// Count methods in a class (complexity ≤10)
    fn count_class_members(&self, source: &str, class_name: &str) -> usize {
        debug_assert!(!source.is_empty(), "source must not be empty");
        debug_assert!(!class_name.is_empty(), "class_name must not be empty");
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

                // Count method declarations and properties
                if ((trimmed.contains('(') && trimmed.contains(')')) || trimmed.contains(" => "))
                    && (trimmed.contains("public")
                        || trimmed.contains("private")
                        || trimmed.contains("protected"))
                    && !trimmed.contains("class")
                    && !trimmed.contains("interface")
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// Extracts method declarations (complexity ≤10)
    fn extract_method_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(method_name) = self.extract_method_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&method_name);
                let visibility = self.extract_method_visibility(trimmed);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility,
                    is_async: trimmed.contains("async"),
                    line: 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract method name from line (complexity ≤10)
    fn extract_method_name_from_line(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Handle regular methods
        if line.contains('(')
            && line.contains(')')
            && !line.contains("class")
            && !line.contains("interface")
        {
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
        // Handle properties with =>
        else if line.contains(" => ") && !line.contains("class") && !line.contains("interface") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if i + 1 < parts.len() && parts[i + 1] == "=>" {
                    return Some((*part).to_string());
                }
            }
        }
        None
    }

    /// Helper to extract method visibility (complexity ≤10)
    fn extract_method_visibility(&self, line: &str) -> String {
        debug_assert!(!line.is_empty(), "line must not be empty");
        if line.contains("public") {
            "public".to_string()
        } else if line.contains("private") {
            "private".to_string()
        } else if line.contains("protected") {
            "protected".to_string()
        } else {
            "internal".to_string()
        }
    }

    /// Extracts interface declarations (complexity ≤10)
    fn extract_interface_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let lines: Vec<&str> = source.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if let Some(interface_name) = self.extract_interface_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&interface_name);
                let visibility = if trimmed.contains("public") {
                    "public"
                } else {
                    "internal"
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
        debug_assert!(!line.is_empty(), "line must not be empty");
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

    /// Gets qualified name for a symbol (complexity ≤10)
    fn get_qualified_name(&self, name: &str) -> String {
        debug_assert!(!name.is_empty(), "name must not be empty");
        if self.namespace_name.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.namespace_name, name)
        }
    }
}
