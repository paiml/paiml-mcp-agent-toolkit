// CAstVisitor implementation: constructor, source analysis, and all extraction methods.
// This file is included into c.rs and shares its module scope (no `use` imports here).

#[cfg(feature = "c-ast")]
impl CAstVisitor {
    /// Creates a new C AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        // Check if file is a header file
        let is_header = file_path.extension().map(|ext| ext == "h").unwrap_or(false);

        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            current_scope: Vec::new(),
            is_header,
        }
    }

    /// Analyzes C source code and extracts AST items (complexity ≤10)
    pub fn analyze_c_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_function_declarations(source)?;
        self.extract_struct_declarations(source)?;
        self.extract_enum_declarations(source)?;
        self.extract_typedef_declarations(source)?;
        self.extract_global_variables(source)?;

        Ok(self.items)
    }

    /// Extracts function declarations (complexity ≤10)
    fn extract_function_declarations(&mut self, source: &str) -> Result<(), String> {
        let mut in_function = false;
        let mut brace_depth = 0;
        let mut current_function_name = String::new();
        let mut has_static_modifier = false;
        let mut _has_inline_modifier = false;
        let mut in_multiline_comment = false; // BUG-009 FIX: Track multiline comment state

        // Mark them as used
        let _ = &current_function_name;
        let _ = &has_static_modifier;

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

            // Check for function declaration
            if !in_function && self.is_function_declaration(trimmed) {
                // Check modifiers
                has_static_modifier = trimmed.contains("static ");
                _has_inline_modifier = trimmed.contains("inline ");

                if let Ok(name) = self.extract_function_name(trimmed) {
                    current_function_name = name;

                    // Only add function if it has a body (not just a declaration)
                    if trimmed.contains("{") {
                        self.items.push(AstItem::Function {
                            name: current_function_name.clone(),
                            visibility: if has_static_modifier {
                                "private"
                            } else {
                                "public"
                            }
                            .to_string(),
                            is_async: false,
                            line: line_num + 1,
                        });
                        in_function = true;
                    }
                }
            }

            // Track brace depth to know when we exit the function
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            if in_function && brace_depth == 0 {
                in_function = false;
                let _ = &has_static_modifier; // Mark as used
                _has_inline_modifier = false;
            }
        }
        Ok(())
    }

    /// Extracts struct declarations (complexity ≤10)
    fn extract_struct_declarations(&mut self, source: &str) -> Result<(), String> {
        let mut in_struct = false;
        let mut brace_depth = 0;
        let mut struct_start_line = 0;
        let mut current_struct_name = String::new();
        let mut fields_count = 0;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Check for struct declaration (but not function return types)
            if !in_struct && trimmed.starts_with("struct ") {
                // Skip if this is a function (has parentheses before brace)
                let has_function_params = trimmed.contains("(")
                    && trimmed.find("(").unwrap_or(usize::MAX)
                        < trimmed.find("{").unwrap_or(usize::MAX);

                if !has_function_params {
                    if let Some(name) = self.extract_struct_name(trimmed) {
                        current_struct_name = name;
                        struct_start_line = line_num + 1;

                        // Check if struct definition has opening brace
                        if trimmed.contains("{") {
                            in_struct = true;
                            fields_count = 0;
                        }
                    }
                }
            }

            // Count fields when in a struct
            if in_struct
                && !trimmed.is_empty()
                && !trimmed.starts_with("{")
                && !trimmed.starts_with("}")
            {
                // This is a field
                fields_count += 1;
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Check if we're at the end of a struct definition
            if in_struct
                && trimmed.contains("}")
                && (brace_depth == 0 || trimmed.trim_end().ends_with(";"))
            {
                in_struct = false;

                // Only add struct if it has a name (avoids anonymous structs)
                if !current_struct_name.is_empty() {
                    self.items.push(AstItem::Struct {
                        name: current_struct_name.clone(),
                        visibility: "public".to_string(),
                        fields_count,
                        derives: vec![],
                        line: struct_start_line,
                    });
                }
            }
        }
        Ok(())
    }

    /// Extracts enum declarations (complexity ≤10)
    fn extract_enum_declarations(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("enum ") && trimmed.contains("{") {
                if let Some(name) = self.extract_enum_name(trimmed) {
                    self.items.push(AstItem::Enum {
                        name,
                        visibility: "public".to_string(),
                        variants_count: 1, // Simplified count
                        line: line_num + 1,
                    });
                }
            }
        }
        Ok(())
    }

    /// Extracts typedef declarations (complexity ≤10)
    fn extract_typedef_declarations(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("typedef ") {
                if let Some(name) = self.extract_typedef_name(trimmed) {
                    self.items.push(AstItem::Struct {
                        name,
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

    /// Extracts global variables (complexity ≤10)
    #[allow(clippy::cast_possible_truncation)]
    fn extract_global_variables(&mut self, source: &str) -> Result<(), String> {
        let mut in_function = false;
        let mut brace_depth = 0;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and preprocessor directives
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                continue;
            }

            // Track function scope
            if self.is_function_declaration(trimmed) {
                in_function = true;
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            if in_function && brace_depth == 0 {
                in_function = false;
            }

            // Only process global variables (outside functions)
            if !in_function && brace_depth == 0 && trimmed.contains(";") && !trimmed.contains("(") {
                // Looks like a global variable
                if let Some(name) = self.extract_variable_name(trimmed) {
                    let visibility = if trimmed.contains("static ") {
                        "private"
                    } else {
                        "public"
                    };

                    // Variables not supported in AstItem, use Struct as placeholder
                    self.items.push(AstItem::Struct {
                        name,
                        visibility: visibility.to_string(),
                        fields_count: 0,
                        derives: Vec::new(),
                        line: line_num + 1,
                    });
                }
            }
        }
        Ok(())
    }

    /// Checks if a line is a function declaration (complexity ≤10)
    fn is_function_declaration(&self, line: &str) -> bool {
        // Basic check: contains parentheses and is not a preprocessing directive
        if !line.contains("(") || line.starts_with("#") {
            return false;
        }

        // Check for common function return types
        let common_types = ["void", "int", "char", "float", "double", "size_t", "bool"];

        for typ in &common_types {
            // Check for pattern like "int foo(" or "static void bar("
            let pattern = format!("{} ", typ);
            if line.contains(&pattern) && line.contains("(") {
                return true;
            }
        }

        // Also check for function pointers
        line.contains("(")
            && line.contains(")")
            && line.contains("*")
            && !line.starts_with("if")
            && !line.starts_with("while")
    }

    /// Extracts function name from declaration line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Result<String, String> {
        // Simplified extraction - get text between return type and opening parenthesis
        let after_type = line
            .split_whitespace()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ");
        let before_paren = after_type.split('(').next().unwrap_or("");

        // Get last word which should be the function name
        let name = before_paren.split_whitespace().last().unwrap_or("");

        if name.is_empty() {
            Err("Could not extract function name".to_string())
        } else {
            Ok(name.to_string())
        }
    }

    /// Extracts struct name from declaration line (complexity ≤10)
    fn extract_struct_name(&self, line: &str) -> Option<String> {
        let words: Vec<&str> = line.split_whitespace().collect();

        // Find the word after "struct"
        for (i, word) in words.iter().enumerate() {
            if *word == "struct" && i + 1 < words.len() {
                let next_word = words[i + 1];
                // Strip any trailing characters like { or ;
                let name = next_word.trim_end_matches(['{', ';']);
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    /// Extracts enum name from declaration line (complexity ≤10)
    fn extract_enum_name(&self, line: &str) -> Option<String> {
        let words: Vec<&str> = line.split_whitespace().collect();

        // Find the word after "enum"
        for (i, word) in words.iter().enumerate() {
            if *word == "enum" && i + 1 < words.len() {
                let next_word = words[i + 1];
                // Strip any trailing characters like { or ;
                let name = next_word.trim_end_matches(['{', ';']);
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    /// Extracts typedef name from declaration line (complexity ≤10)
    fn extract_typedef_name(&self, line: &str) -> Option<String> {
        // For typedef, the name is typically the last token before the semicolon
        let before_semicolon = line.split(';').next().unwrap_or("");
        let words: Vec<&str> = before_semicolon.split_whitespace().collect();

        if !words.is_empty() {
            // Last word is usually the new type name
            Some(words.last().expect("checked is_empty").to_string())
        } else {
            None
        }
    }

    /// Extracts variable name from declaration line (complexity ≤10)
    fn extract_variable_name(&self, line: &str) -> Option<String> {
        // Remove type qualifiers
        let clean_line = line
            .trim()
            .replace("const ", "")
            .replace("static ", "")
            .replace("extern ", "")
            .replace("volatile ", "");

        // Split by whitespace to get the type and name
        let parts: Vec<&str> = clean_line.split_whitespace().collect();

        if parts.len() >= 2 {
            // Second word is usually the variable name (after the type)
            let name_part = parts[1];
            // Handle cases where name might include initialization or array dimensions
            let name = name_part
                .split('=')
                .next()
                .unwrap_or(name_part)
                .split('[')
                .next()
                .unwrap_or(name_part)
                .split('(') // Handle function pointer case
                .next()
                .unwrap_or(name_part)
                .split(';')
                .next()
                .unwrap_or(name_part);

            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }
}
