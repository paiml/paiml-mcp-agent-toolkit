#[cfg(feature = "cpp-ast")]
impl CppAstVisitor {

    /// Extracts function declarations (complexity ≤10)
    #[allow(clippy::cast_possible_truncation)]
    fn extract_function_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Track namespace context while parsing
        let mut namespace_stack: Vec<String> = Vec::new();
        let mut brace_depth = 0;
        let mut class_depth = 0; // Track if we're inside a class
        let mut in_multiline_comment = false; // BUG-009 FIX: Track multiline comment state

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // BUG-009 FIX: Handle multiline comment state
            // Check if we're entering a multiline comment
            if trimmed.contains("/*") {
                in_multiline_comment = true;
            }

            // Check if we're exiting a multiline comment
            if trimmed.contains("*/") {
                in_multiline_comment = false;
                continue; // Skip the line with the closing comment
            }

            // Skip if we're inside a multiline comment
            if in_multiline_comment {
                continue;
            }

            // Skip single-line comments and preprocessor directives
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                continue;
            }

            // Track namespace declarations
            if trimmed.starts_with("namespace ") {
                if let Some(ns_name) = self.extract_namespace_name(trimmed) {
                    namespace_stack.push(ns_name);
                }
            }

            // Track class/struct declarations to avoid extracting methods
            if trimmed.starts_with("class ") || trimmed.starts_with("struct ") {
                class_depth = brace_depth + 1; // Methods will be at this depth or deeper
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Reset class depth when we exit the class
            if class_depth > 0 && brace_depth < class_depth {
                class_depth = 0;
            }

            // Check for closing namespace
            if trimmed.contains("}")
                && !namespace_stack.is_empty()
                && brace_depth < namespace_stack.len() as i32
            {
                namespace_stack.pop();
            }

            // Skip if we're inside a class
            if class_depth > 0 && brace_depth >= class_depth {
                continue;
            }

            // Check for function declaration
            if self.is_function_declaration(trimmed) && !self.is_class_method(trimmed) {
                if let Ok(name) = self.extract_function_name(trimmed) {
                    // Build qualified name with current namespace
                    let qualified_name = if !namespace_stack.is_empty() {
                        format!("{}::{}", namespace_stack.join("::"), name)
                    } else {
                        name
                    };

                    // Check for function visibility
                    let visibility = if trimmed.contains("static ") {
                        "private"
                    } else {
                        "public"
                    }
                    .to_string();

                    // Check if function is async (C++20 feature)
                    let is_async = trimmed.contains("async ") || trimmed.contains("co_await ");

                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: visibility.clone(),
                        is_async,
                        line: line_num + 1,
                    });
                }
            }
        }
        Ok(())
    }

    /// Extracts class method declarations (complexity ≤10)
    #[allow(clippy::cast_possible_truncation)]
    fn extract_method_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut in_class = false;
        let mut current_class_name = String::new();
        let mut brace_depth = 0;
        let mut visibility = "private".to_string();

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and preprocessor directives
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                continue;
            }

            // Track class declarations
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
                    in_class = true;
                }
            }

            // Track access modifiers within class
            if in_class {
                if trimmed.starts_with("public:") {
                    visibility = "public".to_string();
                } else if trimmed.starts_with("private:") {
                    visibility = "private".to_string();
                } else if trimmed.starts_with("protected:") {
                    visibility = "protected".to_string();
                }
            }

            // Check for method declaration within class
            if in_class && self.is_function_declaration(trimmed) {
                if let Ok(method_name) = self.extract_function_name(trimmed) {
                    let qualified_name = format!("{}::{}", current_class_name, method_name);

                    // Check for virtual/static/const modifiers
                    let is_virtual = trimmed.contains("virtual ");
                    let is_static = trimmed.contains("static ");
                    let is_const = trimmed.contains(" const");

                    // Add more detail to visibility
                    let method_visibility = if is_virtual {
                        format!("{}_virtual", visibility)
                    } else if is_static {
                        format!("{}_static", visibility)
                    } else if is_const {
                        format!("{}_const", visibility)
                    } else {
                        visibility.clone()
                    };

                    // Check if method is async
                    let is_async = trimmed.contains("async ") || trimmed.contains("co_await ");

                    // Use Function for Method
                    self.items.push(AstItem::Function {
                        name: qualified_name,
                        visibility: method_visibility,
                        is_async,
                        line: line_num + 1,
                    });
                }
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Check if we're exiting a class
            if in_class && trimmed.contains("}") && (brace_depth == 0 || trimmed.ends_with("};")) {
                in_class = false;
            }
        }
        Ok(())
    }

    /// Extracts enum declarations (complexity ≤10)
    #[allow(clippy::cast_possible_truncation)]
    fn extract_enum_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Track namespace context while parsing
        let mut namespace_stack: Vec<String> = Vec::new();
        let mut brace_depth = 0;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Track namespace declarations
            if trimmed.starts_with("namespace ") {
                if let Some(ns_name) = self.extract_namespace_name(trimmed) {
                    namespace_stack.push(ns_name);
                }
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Check for closing namespace
            if trimmed.contains("}")
                && !namespace_stack.is_empty()
                && brace_depth < namespace_stack.len() as i32
            {
                namespace_stack.pop();
            }

            if (trimmed.starts_with("enum ") || trimmed.starts_with("enum class "))
                && trimmed.contains("{")
            {
                if let Some(name) = self.extract_enum_name(trimmed) {
                    // Build qualified name with current namespace
                    let qualified_name = if !namespace_stack.is_empty() {
                        format!("{}::{}", namespace_stack.join("::"), name)
                    } else {
                        name
                    };

                    // Count enum variants (simplified)
                    let variants_count = self.count_enum_variants(source, line_num);

                    self.items.push(AstItem::Enum {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        variants_count,
                        line: line_num + 1,
                    });
                }
            }
        }
        Ok(())
    }

    /// Extracts typedef declarations (complexity ≤10)
    fn extract_typedef_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Check for typedef and using declarations
            if trimmed.starts_with("typedef ")
                || (trimmed.starts_with("using ") && trimmed.contains("="))
            {
                if let Some(name) = if trimmed.starts_with("typedef ") {
                    self.extract_typedef_name(trimmed)
                } else {
                    self.extract_using_name(trimmed)
                } {
                    let qualified_name = self.get_qualified_name(&name);

                    // Use Struct for TypeAlias
                    self.items.push(AstItem::Struct {
                        name: qualified_name,
                        visibility: "public".to_string(),
                        fields_count: 0,
                        derives: Vec::new(),
                        line: line_num + 1,
                    });
                }
            }
        }
        Ok(())
    }

    /// Extracts template declarations (complexity ≤10)
    fn extract_template_declarations(&mut self, source: &str) -> Result<(), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut in_template = false;
        let mut template_line = 0;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Check for template declaration start
            if trimmed.starts_with("template<") || trimmed.starts_with("template <") {
                in_template = true;
                template_line = line_num;
                continue;
            }

            // Check what follows the template
            if in_template {
                in_template = false;

                if trimmed.starts_with("class ") || trimmed.starts_with("struct ") {
                    // Template class/struct
                    if let Some(name) = self.extract_class_name(trimmed) {
                        let qualified_name = self.get_qualified_name(&name);

                        // Use Struct for GenericType
                        self.items.push(AstItem::Struct {
                            name: qualified_name,
                            visibility: "public".to_string(),
                            fields_count: 0,
                            derives: Vec::new(),
                            line: template_line + 1,
                        });
                    }
                } else if self.is_function_declaration(trimmed) {
                    // Template function
                    if let Ok(name) = self.extract_function_name(trimmed) {
                        let qualified_name = self.get_qualified_name(&name);

                        self.items.push(AstItem::Function {
                            name: format!("template::{}", qualified_name),
                            visibility: "public".to_string(),
                            is_async: false,
                            line: template_line + 1,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Counts enum variants (complexity ≤10)
    #[allow(clippy::cast_possible_truncation)]
    fn count_enum_variants(&self, source: &str, enum_start_line: usize) -> usize {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let mut in_enum = false;
        let mut brace_depth = 0;
        let mut variant_count = 0;

        for (i, line) in source.lines().enumerate().skip(enum_start_line) {
            let trimmed = line.trim();

            // Find the enum start
            if i == enum_start_line && trimmed.contains("{") {
                in_enum = true;
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Count variants
            if in_enum && trimmed.contains(",") {
                // Each comma represents a variant (except trailing comma)
                variant_count += trimmed.chars().filter(|&c| c == ',').count();
            }

            // Exit when enum is closed
            if in_enum && trimmed.contains("}") && brace_depth == 0 {
                // Add 1 for the last variant (after the last comma)
                variant_count += 1;
                break;
            }
        }

        // Ensure at least 1 variant
        std::cmp::max(1, variant_count)
    }

}
