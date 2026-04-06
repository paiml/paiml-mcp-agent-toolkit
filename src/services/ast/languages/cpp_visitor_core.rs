#[cfg(feature = "cpp-ast")]
impl CppAstVisitor {
    /// Creates a new C++ AST visitor
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        // Check if file is a header file
        let is_header = file_path
            .extension()
            .map(|ext| ext == "hpp" || ext == "h" || ext == "hxx" || ext == "hh")
            .unwrap_or(false);

        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            current_namespace: Vec::new(),
            current_class: None,
            is_header,
        }
    }

    /// Analyzes C++ source code and extracts AST items (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_cpp_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_namespace_declarations(source)?;
        self.extract_class_declarations(source)?;
        self.extract_function_declarations(source)?;
        self.extract_method_declarations(source)?;
        self.extract_enum_declarations(source)?;
        self.extract_typedef_declarations(source)?;
        self.extract_template_declarations(source)?;

        Ok(self.items)
    }

    /// Extracts namespace declarations (complexity ≤10)
    #[allow(clippy::cast_possible_truncation)]
    fn extract_namespace_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut in_namespace = false;
        let mut brace_depth = 0;
        let mut _current_namespace = String::new();

        for line in source.lines() {
            let trimmed = line.trim();

            // Track namespace declarations
            if trimmed.starts_with("namespace ") && !in_namespace {
                if let Some(name) = self.extract_namespace_name(trimmed) {
                    _current_namespace = name;
                    self.current_namespace.push(_current_namespace.clone());
                    in_namespace = true;

                    // No AST item for namespaces yet, just track it for qualification
                }
            }

            // Track opening and closing braces
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Check if we're exiting a namespace
            if in_namespace && trimmed.contains("}") && brace_depth == 0 {
                in_namespace = false;
                self.current_namespace.pop();
            }
        }
        Ok(())
    }

    /// Extracts class declarations (complexity ≤10)
    #[allow(clippy::cast_possible_truncation)]
    fn extract_class_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut in_class = false;
        let mut brace_depth = 0;
        let mut current_class_name = String::new();
        let mut class_start_line = 0;
        let mut fields_count = 0;
        let mut visibility = "public".to_string();

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Check for class/struct declaration
            if (trimmed.starts_with("class ") || trimmed.starts_with("struct ")) && !in_class {
                let class_type = if trimmed.starts_with("class ") {
                    "class"
                } else {
                    "struct"
                };

                if let Some(name) = self.extract_class_name(trimmed) {
                    // Set default visibility based on type
                    visibility = if class_type == "class" {
                        "private"
                    } else {
                        "public"
                    }
                    .to_string();

                    current_class_name = self.get_qualified_name(&name);
                    self.current_class = Some(current_class_name.clone());
                    class_start_line = line_num + 1;
                    in_class = true;
                    fields_count = 0;
                }
            }

            // Count fields when in a class
            if in_class && !trimmed.is_empty() {
                // Skip certain lines that aren't fields
                let is_field = !trimmed.starts_with("public:")
                    && !trimmed.starts_with("private:")
                    && !trimmed.starts_with("protected:")
                    && !trimmed.starts_with("{")
                    && !trimmed.starts_with("}")
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("/*")
                    && trimmed.contains(";");

                // Track access modifiers
                if trimmed.starts_with("public:") {
                    visibility = "public".to_string();
                } else if trimmed.starts_with("private:") {
                    visibility = "private".to_string();
                } else if trimmed.starts_with("protected:") {
                    visibility = "protected".to_string();
                }

                // Count field if it looks like one
                if is_field {
                    fields_count += 1;
                }
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Check if we're at the end of a class definition
            if in_class && trimmed.contains("}") && (brace_depth == 0 || trimmed.ends_with("};")) {
                in_class = false;

                // Only add class if it has a name
                if !current_class_name.is_empty() {
                    // Use Struct for Class
                    self.items.push(AstItem::Struct {
                        name: current_class_name.clone(),
                        visibility: visibility.clone(),
                        fields_count,
                        derives: Vec::new(),
                        line: class_start_line,
                    });
                }

                self.current_class = None;
            }
        }
        Ok(())
    }
}
