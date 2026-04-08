    /// Creates a mock AstItem for testing purposes
    fn create_mock_function(name: &str, is_async: bool, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "public".to_string(),
            is_async,
            line,
        }
    }

    /// Creates a mock class/struct for testing
    fn create_mock_class(name: &str, fields_count: usize, line: usize) -> AstItem {
        AstItem::Struct {
            name: name.to_string(),
            visibility: "public".to_string(),
            fields_count,
            derives: vec![],
            line,
        }
    }

    /// Validates that function names follow enhanced naming conventions
    fn validate_enhanced_function_names(names: &[String]) -> Vec<String> {
        let mut issues = Vec::new();

        for name in names {
            // Should not be generic placeholders
            if name.starts_with("function_") && name.chars().last().unwrap().is_ascii_digit() {
                issues.push(format!("Generic name found: {}", name));
            }

            // Should not be "anonymous" unless truly anonymous
            if name == "anonymous" {
                issues.push(format!("Anonymous function without context: {}", name));
            }

            // Should include context for methods (ClassName::method)
            if name.contains("::") {
                // Good - has context
            } else if name.len() < 3 {
                issues.push(format!("Name too short, might lack context: {}", name));
            }
        }

        issues
    }

    /// Validates WASM function names are descriptive
    fn validate_wasm_function_names(names: &[String], module_name: &str) -> Vec<String> {
        let mut issues = Vec::new();

        for name in names {
            // Should include module context
            if !name.contains(module_name) {
                issues.push(format!("Missing module context in WASM function: {}", name));
            }

            // Should not be just numeric suffixes
            if name.ends_with("_0") || name.ends_with("_1") || name.ends_with("_2") {
                issues.push(format!("Numeric suffix suggests generic naming: {}", name));
            }
        }

        issues
    }
