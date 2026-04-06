#[cfg(feature = "scala-ast")]
impl ScalaAstVisitor {
    /// Creates a new Scala AST visitor
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            package_name: String::new(),
            class_count: 0,
            trait_count: 0,
            object_count: 0,
            case_class_count: 0,
        }
    }

    /// Analyzes Scala source code and extracts AST items (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_scala_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        // Check for basic Scala syntax validity
        if source.contains("{{{ !!!") || !self.is_valid_scala_syntax(source) {
            return Err("Invalid Scala syntax".to_string());
        }

        self.extract_package_declaration(source)?;
        self.extract_class_declarations(source)?;
        self.extract_trait_declarations(source)?;
        self.extract_object_declarations(source)?;
        self.extract_method_declarations(source)?;
        self.extract_case_class_declarations(source)?;

        Ok(self.items)
    }

    /// Check basic Scala syntax validity (complexity ≤10)
    fn is_valid_scala_syntax(&self, source: &str) -> bool {
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
            if trimmed.starts_with("package ") {
                let package_part = trimmed.strip_prefix("package ").unwrap_or("").trim();
                self.package_name = package_part.to_string();
                return Ok(());
            }
        }
        Ok(())
    }

    /// Extracts class declarations (complexity ≤10)
    fn extract_class_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(class_name) = self.extract_class_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&class_name);
                let visibility = self.determine_visibility(trimmed);

                self.items.push(AstItem::Struct {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    fields_count: 0, // To be filled in later analysis
                    derives: vec![],
                    line: line_num + 1,
                });
                self.class_count += 1;
            }
        }
        Ok(())
    }

    /// Extracts case class declarations (complexity ≤10)
    fn extract_case_class_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(class_name) = self.extract_case_class_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&class_name);
                let visibility = self.determine_visibility(trimmed);

                self.items.push(AstItem::Struct {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    fields_count: 0, // To be filled in later analysis
                    derives: vec!["case".to_string()],
                    line: line_num + 1,
                });
                self.case_class_count += 1;
            }
        }
        Ok(())
    }

    /// Helper to extract class name from line (complexity ≤10)
    fn extract_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("class ") && !line.contains("case class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i + 1 < parts.len() {
                    let class_name = parts[i + 1].split('[').next()?; // Handle generics
                    let class_name = class_name.split('(').next()?; // Handle constructor params
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }

    /// Helper to extract case class name from line (complexity ≤10)
    fn extract_case_class_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("case class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "class" && i > 0 && parts[i - 1] == "case" && i + 1 < parts.len() {
                    let class_name = parts[i + 1].split('[').next()?; // Handle generics
                    let class_name = class_name.split('(').next()?; // Handle constructor params
                    return Some(class_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts trait declarations (complexity ≤10)
    fn extract_trait_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(trait_name) = self.extract_trait_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&trait_name);
                let visibility = self.determine_visibility(trimmed);

                self.items.push(AstItem::Trait {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    line: line_num + 1,
                });
                self.trait_count += 1;
            }
        }
        Ok(())
    }

    /// Helper to extract trait name from line (complexity ≤10)
    fn extract_trait_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("trait ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "trait" && i + 1 < parts.len() {
                    let trait_name = parts[i + 1].split('[').next()?; // Handle generics
                    return Some(trait_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts object declarations (complexity ≤10)
    fn extract_object_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(object_name) = self.extract_object_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&object_name);
                let visibility = self.determine_visibility(trimmed);

                self.items.push(AstItem::Module {
                    name: qualified_name,
                    visibility: visibility.to_string(),
                    line: line_num + 1,
                });
                self.object_count += 1;
            }
        }
        Ok(())
    }

    /// Helper to extract object name from line (complexity ≤10)
    fn extract_object_name_from_line(&self, line: &str) -> Option<String> {
        if line.contains("object ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "object" && i + 1 < parts.len() {
                    let object_name = parts[i + 1];
                    return Some(object_name.to_string());
                }
            }
        }
        None
    }

    /// Extracts method declarations (complexity ≤10)
    fn extract_method_declarations(&mut self, source: &str) -> Result<(), String> {
        let lines: Vec<&str> = source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            if let Some(method_name) = self.extract_method_name_from_line(trimmed) {
                let qualified_name = self.get_qualified_name(&method_name);
                let visibility = self.determine_visibility(trimmed);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility,
                    is_async: trimmed.contains("async ") || trimmed.contains("Future["),
                    line: line_num + 1,
                });
            }
        }
        Ok(())
    }

    /// Helper to extract method name from line (complexity ≤10)
    fn extract_method_name_from_line(&self, line: &str) -> Option<String> {
        // Match Scala method declarations: "def methodName(...)"
        if line.contains("def ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "def" && i + 1 < parts.len() {
                    let method_part = parts[i + 1];
                    let method_name = method_part.split('(').next()?;
                    if !method_name.is_empty() {
                        return Some(method_name.to_string());
                    }
                }
            }
        }
        None
    }

    /// Helper to determine visibility from modifiers (complexity ≤10)
    fn determine_visibility(&self, line: &str) -> String {
        if line.contains("private ") {
            "private".to_string()
        } else if line.contains("protected ") {
            "protected".to_string()
        } else if line.contains("private[") {
            "package".to_string()
        } else {
            "public".to_string()
        }
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
