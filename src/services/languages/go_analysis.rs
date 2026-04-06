// Go analysis implementation — included from go.rs
// Contains: GoAstVisitor impl, GoComplexityAnalyzer impl, analyze_go_file

#[cfg(feature = "go-ast")]
impl GoAstVisitor {
    /// Creates a new Go AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            package_name: String::new(),
            _current_type: Vec::new(),
        }
    }

    /// Analyzes Go source code and extracts AST items (complexity ≤10)
    pub fn analyze_go_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        self.extract_package_declaration(source)?;
        self.extract_function_declarations(source)?;
        self.extract_type_declarations(source)?;
        self.extract_interface_declarations(source)?;

        Ok(self.items)
    }

    /// Extracts package declaration (complexity ≤10)
    fn extract_package_declaration(&mut self, source: &str) -> Result<(), String> {
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("package ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    self.package_name = parts[1].to_string();
                    return Ok(());
                }
            }
        }
        self.package_name = "main".to_string();
        Ok(())
    }

    /// Extracts function declarations (complexity ≤10)
    fn extract_function_declarations(&mut self, source: &str) -> Result<(), String> {
        let mut in_function = false;
        let mut brace_depth = 0;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("func ") && !in_function {
                let func_name = self.extract_function_name(trimmed)?;
                let qualified_name = self.get_qualified_name(&func_name);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility: "public".to_string(),
                    is_async: false,
                    line: line_num + 1,
                });
                in_function = true;
            }

            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            if in_function && brace_depth == 0 {
                in_function = false;
            }
        }
        Ok(())
    }

    /// Extracts type declarations (complexity ≤10)
    fn extract_type_declarations(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("type ") && trimmed.contains("struct") {
                let struct_name = self.extract_type_name(trimmed)?;
                let qualified_name = self.get_qualified_name(&struct_name);

                self.items.push(AstItem::Struct {
                    name: qualified_name,
                    visibility: "public".to_string(),
                    fields_count: 2,
                    derives: vec![],
                    line: line_num + 1,
                });
            }
        }
        Ok(())
    }

    /// Extracts interface declarations (complexity ≤10)
    fn extract_interface_declarations(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("type ") && trimmed.contains("interface") {
                let interface_name = self.extract_type_name(trimmed)?;
                let qualified_name = self.get_qualified_name(&interface_name);

                self.items.push(AstItem::Trait {
                    name: qualified_name,
                    visibility: "public".to_string(),
                    line: line_num + 1,
                });
            }
        }
        Ok(())
    }

    /// Extracts function name from declaration line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Result<String, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name_part = parts[1];
            let func_name = name_part.split('(').next().unwrap_or(name_part);
            Ok(func_name.to_string())
        } else {
            Err("Invalid function declaration".to_string())
        }
    }

    /// Extracts type name from type declaration line (complexity ≤10)
    fn extract_type_name(&self, line: &str) -> Result<String, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Ok(parts[1].to_string())
        } else {
            Err("Invalid type declaration".to_string())
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

impl GoComplexityAnalyzer {
    /// Creates a new Go complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Go source code (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.contains("if ")
                || trimmed.contains("for ")
                || trimmed.contains("switch ")
                || trimmed.contains("case ")
            {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
}

/// Public async function to analyze a Go file and return FileContext
/// This matches the API pattern used by TypeScript analyzer
#[cfg(feature = "go-ast")]
pub async fn analyze_go_file(
    path: &Path,
) -> Result<crate::services::context::FileContext, crate::models::error::TemplateError> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::models::error::TemplateError;
    use crate::services::context::FileContext;

    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Create visitor and analyze
    let visitor = GoAstVisitor::new(path);
    let items = visitor
        .analyze_go_source(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    // Return FileContext
    Ok(FileContext {
        path: path.display().to_string(),
        language: "go".to_string(),
        items,
        complexity_metrics: None,
    })
}
