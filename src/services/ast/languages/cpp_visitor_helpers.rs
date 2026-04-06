#[cfg(feature = "cpp-ast")]
impl CppAstVisitor {
    /// Checks if a line is a function declaration (complexity ≤10)
    fn is_function_declaration(&self, line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Basic check: contains parentheses and is not a preprocessing directive
        if !line.contains("(") || line.starts_with("#") {
            return false;
        }

        // Exclude control statements
        if line.starts_with("if") || line.starts_with("while") || line.starts_with("for") {
            return false;
        }

        // Exclude variable declarations with function calls (e.g., "int result = add(5, 3);")
        // If there's an = before the (, it's an assignment/initialization, not a function declaration
        if let Some(paren_pos) = line.find("(") {
            if let Some(equals_pos) = line.find("=") {
                if equals_pos < paren_pos {
                    return false;
                }
            }
        }

        // Check for common function return types and modifiers
        let common_types = [
            "void", "int", "char", "float", "double", "auto", "bool", "string",
        ];
        let common_modifiers = ["static", "inline", "virtual", "explicit", "constexpr"];

        for typ in &common_types {
            if line.contains(&format!("{} ", typ)) && line.contains("(") {
                return true;
            }
        }

        for modifier in &common_modifiers {
            if line.contains(&format!("{} ", modifier)) && line.contains("(") {
                return true;
            }
        }

        // Also check for function pointers or constructors/destructors
        line.contains("(")
            && line.contains(")")
            && (line.contains("*") || line.contains("~") || line.contains("::"))
    }

    /// Checks if a function declaration is a class method (complexity ≤10)
    fn is_class_method(&self, line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Class methods have :: in their name
        line.contains("::") && line.contains("(")
    }

    /// Extracts function name from declaration line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Result<String, String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // Handle method with :: scope resolution
        if line.contains("::") {
            let parts: Vec<&str> = line.split("::").collect();
            let after_scope = parts.last().unwrap_or(&"");

            // Get text before opening parenthesis
            let before_paren = after_scope.split('(').next().unwrap_or("");

            // Get last word which should be the function name
            let name = before_paren.split_whitespace().last().unwrap_or("");

            if !name.is_empty() {
                return Ok(name.to_string());
            }
        } else {
            // Regular function
            let before_paren = line.split('(').next().unwrap_or("");
            let words: Vec<&str> = before_paren.split_whitespace().collect();

            if !words.is_empty() {
                return Ok(words.last().unwrap_or(&"").to_string());
            }
        }

        Err("Could not extract function name".to_string())
    }

    /// Extracts class name from declaration line (complexity ≤10)
    fn extract_class_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let words: Vec<&str> = line.split_whitespace().collect();

        // Find the word after "class" or "struct"
        for (i, word) in words.iter().enumerate() {
            if (*word == "class" || *word == "struct") && i + 1 < words.len() {
                let next_word = words[i + 1];
                // Strip any trailing characters like { or : for inheritance
                let name = next_word.trim_end_matches(['{', ':', ';']).to_string();

                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    /// Extracts namespace name from declaration line (complexity ≤10)
    fn extract_namespace_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let words: Vec<&str> = line.split_whitespace().collect();

        // Find the word after "namespace"
        for (i, word) in words.iter().enumerate() {
            if *word == "namespace" && i + 1 < words.len() {
                let next_word = words[i + 1];
                // Strip any trailing characters like {
                let name = next_word.trim_end_matches(['{', ';']).to_string();

                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    /// Extracts enum name from declaration line (complexity ≤10)
    fn extract_enum_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let words: Vec<&str> = line.split_whitespace().collect();

        // Handle both "enum Foo" and "enum class Foo"
        for (i, word) in words.iter().enumerate() {
            if *word == "enum" {
                if i + 1 < words.len() && words[i + 1] == "class" {
                    // Handle "enum class Foo"
                    if i + 2 < words.len() {
                        let enum_name = words[i + 2].trim_end_matches(['{', ':', ';']).to_string();
                        if !enum_name.is_empty() {
                            return Some(enum_name);
                        }
                    }
                } else if i + 1 < words.len() {
                    // Handle "enum Foo"
                    let enum_name = words[i + 1].trim_end_matches(['{', ':', ';']).to_string();
                    if !enum_name.is_empty() {
                        return Some(enum_name);
                    }
                }
            }
        }
        None
    }

    /// Extracts typedef name from declaration line (complexity ≤10)
    fn extract_typedef_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
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

    /// Extracts 'using' alias name (C++11 style typedef) (complexity ≤10)
    fn extract_using_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        // For "using Alias = Type;", extract "Alias"
        if line.contains("=") {
            let parts: Vec<&str> = line.split('=').collect();
            if !parts.is_empty() {
                let name_part = parts[0].trim();
                if name_part.starts_with("using ") {
                    let name = name_part.trim_start_matches("using ").trim().to_string();

                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }
        None
    }

    /// Gets qualified name for a symbol (complexity ≤10)
    fn get_qualified_name(&self, name: &str) -> String {
        debug_assert!(!name.is_empty(), "name must not be empty");
        // Build qualified name based on current namespace and class
        let mut qualified_name = String::new();

        // Add namespaces
        if !self.current_namespace.is_empty() {
            qualified_name.push_str(&self.current_namespace.join("::"));
            qualified_name.push_str("::");
        }

        // Add class if we're in one
        if let Some(ref class_name) = self.current_class {
            qualified_name.push_str(class_name);
            qualified_name.push_str("::");
        }

        // Add the name itself
        qualified_name.push_str(name);

        qualified_name
    }
}
