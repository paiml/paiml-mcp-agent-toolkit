//! C++ Language Support for PMAT
//!
//! This module provides C++-specific analysis capabilities using tree-sitter-cpp parser
//! for AST extraction and complexity analysis aligned with C++ best practices.

#[cfg(feature = "cpp-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "cpp-ast")]
use std::path::{Path, PathBuf};

/// C++ AST visitor that extracts C++-specific AST information
#[cfg(feature = "cpp-ast")]
pub struct CppAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    current_namespace: Vec<String>,
    current_class: Option<String>,
    #[allow(dead_code)]
    is_header: bool,
}

#[cfg(feature = "cpp-ast")]
impl CppAstVisitor {
    /// Creates a new C++ AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
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
    pub fn analyze_cpp_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
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
    fn extract_namespace_declarations(&mut self, source: &str) -> Result<(), String> {
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
    fn extract_class_declarations(&mut self, source: &str) -> Result<(), String> {
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
                let class_type = if trimmed.starts_with("class ") { "class" } else { "struct" };
                
                if let Some(name) = self.extract_class_name(trimmed) {
                    // Set default visibility based on type
                    visibility = if class_type == "class" { "private" } else { "public" }.to_string();
                    
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

    /// Extracts function declarations (complexity ≤10)
    fn extract_function_declarations(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and preprocessor directives
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                continue;
            }
            
            // Skip class methods (handled separately)
            if self.current_class.is_some() {
                continue;
            }
            
            // Check for function declaration
            if self.is_function_declaration(trimmed) && !self.is_class_method(trimmed) {
                if let Ok(name) = self.extract_function_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&name);
                    
                    // Check for function visibility
                    let visibility = if trimmed.contains("static ") {
                        "private"
                    } else {
                        "public"
                    }.to_string();
                    
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
    fn extract_method_declarations(&mut self, source: &str) -> Result<(), String> {
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
                let class_type = if trimmed.starts_with("class ") { "class" } else { "struct" };
                
                if let Some(name) = self.extract_class_name(trimmed) {
                    // Set default visibility based on type
                    visibility = if class_type == "class" { "private" } else { "public" }.to_string();
                    
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
    fn extract_enum_declarations(&mut self, source: &str) -> Result<(), String> {
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            
            if (trimmed.starts_with("enum ") || trimmed.starts_with("enum class ")) && trimmed.contains("{") {
                if let Some(name) = self.extract_enum_name(trimmed) {
                    let qualified_name = self.get_qualified_name(&name);
                    
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
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            
            // Check for typedef and using declarations
            if trimmed.starts_with("typedef ") || (trimmed.starts_with("using ") && trimmed.contains("=")) {
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
    fn count_enum_variants(&self, source: &str, enum_start_line: usize) -> usize {
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

    /// Checks if a line is a function declaration (complexity ≤10)
    fn is_function_declaration(&self, line: &str) -> bool {
        // Basic check: contains parentheses and is not a preprocessing directive
        if !line.contains("(") || line.starts_with("#") {
            return false;
        }
        
        // Exclude control statements
        if line.starts_with("if") || line.starts_with("while") || line.starts_with("for") {
            return false;
        }
        
        // Check for common function return types and modifiers
        let common_types = ["void", "int", "char", "float", "double", "auto", "bool", "string"];
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
        line.contains("(") && line.contains(")") && 
            (line.contains("*") || line.contains("~") || line.contains("::"))
    }
    
    /// Checks if a function declaration is a class method (complexity ≤10)
    fn is_class_method(&self, line: &str) -> bool {
        // Class methods have :: in their name
        line.contains("::") && line.contains("(")
    }

    /// Extracts function name from declaration line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Result<String, String> {
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
        let words: Vec<&str> = line.split_whitespace().collect();
        
        // Find the word after "class" or "struct"
        for (i, word) in words.iter().enumerate() {
            if (*word == "class" || *word == "struct") && i + 1 < words.len() {
                let next_word = words[i + 1];
                // Strip any trailing characters like { or : for inheritance
                let name = next_word
                    .trim_end_matches(['{', ':', ';'])
                    .to_string();
                    
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    /// Extracts namespace name from declaration line (complexity ≤10)
    fn extract_namespace_name(&self, line: &str) -> Option<String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        
        // Find the word after "namespace"
        for (i, word) in words.iter().enumerate() {
            if *word == "namespace" && i + 1 < words.len() {
                let next_word = words[i + 1];
                // Strip any trailing characters like {
                let name = next_word
                    .trim_end_matches(['{', ';'])
                    .to_string();
                    
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    /// Extracts enum name from declaration line (complexity ≤10)
    fn extract_enum_name(&self, line: &str) -> Option<String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        
        // Handle both "enum Foo" and "enum class Foo"
        for (i, word) in words.iter().enumerate() {
            if *word == "enum" {
                if i + 1 < words.len() && words[i + 1] == "class" {
                    // Handle "enum class Foo"
                    if i + 2 < words.len() {
                        let enum_name = words[i + 2]
                            .trim_end_matches(['{', ':', ';'])
                            .to_string();
                        if !enum_name.is_empty() {
                            return Some(enum_name);
                        }
                    }
                } else if i + 1 < words.len() {
                    // Handle "enum Foo"
                    let enum_name = words[i + 1]
                        .trim_end_matches(['{', ':', ';'])
                        .to_string();
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
        // For typedef, the name is typically the last token before the semicolon
        let before_semicolon = line.split(';').next().unwrap_or("");
        let words: Vec<&str> = before_semicolon.split_whitespace().collect();
        
        if !words.is_empty() {
            // Last word is usually the new type name
            Some(words.last().unwrap().to_string())
        } else {
            None
        }
    }
    
    /// Extracts 'using' alias name (C++11 style typedef) (complexity ≤10)
    fn extract_using_name(&self, line: &str) -> Option<String> {
        // For "using Alias = Type;", extract "Alias"
        if line.contains("=") {
            let parts: Vec<&str> = line.split('=').collect();
            if !parts.is_empty() {
                let name_part = parts[0].trim();
                if name_part.starts_with("using ") {
                    let name = name_part
                        .trim_start_matches("using ")
                        .trim()
                        .to_string();
                        
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

/// C++ complexity analyzer for extracting C++-specific metrics (complexity ≤10)
#[cfg(feature = "cpp-ast")]
pub struct CppComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "cpp-ast")]
impl Default for CppComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CppComplexityAnalyzer {
    /// Creates a new C++ complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of C++ source code (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 0;
        
        let mut nesting_depth = 0;
        let mut in_comment = false;
        let mut in_function = false;

        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip preprocessor directives
            if trimmed.starts_with("#") {
                continue;
            }
            
            // Handle comments
            if trimmed.contains("/*") {
                in_comment = true;
            }
            if in_comment {
                if trimmed.contains("*/") {
                    in_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("//") {
                continue;
            }
            
            // Track function scope
            if !in_function && trimmed.contains("{") && 
               (trimmed.contains("(") || trimmed.contains(")")) {
                in_function = true;
            }
            
            // Only analyze inside function bodies
            if in_function {
                // Check for control flow statements
                let is_if_stmt = trimmed.starts_with("if ") || trimmed.starts_with("if(");
                let is_else_if = trimmed.starts_with("else if") || trimmed.contains("} else if");
                let _is_else = trimmed.starts_with("else") && !is_else_if;
                let is_switch = trimmed.starts_with("switch ");
                let is_case = trimmed.starts_with("case ") || trimmed.starts_with("default:");
                let is_loop = trimmed.starts_with("while ") || 
                              trimmed.starts_with("for ") || 
                              trimmed.starts_with("do ");
                let is_try = trimmed.starts_with("try ");
                let is_catch = trimmed.starts_with("catch ");
                let is_goto = trimmed.starts_with("goto ");
                let is_ternary = trimmed.contains(" ? ") && trimmed.contains(" : ");
                
                // C++ specific control flow
                let is_range_for = trimmed.contains("for (") && trimmed.contains(" : ");
                let is_lambda = trimmed.contains("[") && trimmed.contains("]") && 
                                (trimmed.contains("(") || trimmed.contains("mutable"));
                let is_template = trimmed.contains("<") && trimmed.contains(">");
                
                // Increment cyclomatic complexity for decision points
                if is_if_stmt || is_else_if || is_switch || is_case || is_loop || 
                   is_catch || is_goto || is_ternary || is_range_for {
                    self.cyclomatic_complexity += 1;
                }
                
                // Cognitive complexity considers nesting
                if is_if_stmt || is_else_if || is_switch || is_loop || is_try || is_lambda {
                    self.cognitive_complexity += 1 + nesting_depth;
                    nesting_depth += 1;
                } else if is_template {
                    // Templates add complexity but less than control flow
                    self.cognitive_complexity += 1;
                }
                
                // Track nesting depth with braces
                if trimmed.contains("{") && !trimmed.contains("}") {
                    nesting_depth += 1;
                }
                
                // Track closing braces
                if trimmed.contains("}") {
                    nesting_depth = nesting_depth.saturating_sub(1);
                    
                    // Check if we're exiting the function
                    if nesting_depth == 0 {
                        in_function = false;
                    }
                }
            }
        }
        
        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
}

/// Public async function to analyze a C++ file and return FileContext
#[cfg(feature = "cpp-ast")]
pub async fn analyze_cpp_file(
    path: &Path,
) -> Result<crate::services::context::FileContext, crate::models::error::TemplateError> {
    use crate::models::error::TemplateError;
    use crate::services::complexity::ComplexityMetrics;
    use crate::services::context::FileContext;

    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Create visitor and analyze
    let visitor = CppAstVisitor::new(path);
    let items = visitor
        .analyze_cpp_source(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    // Analyze complexity
    let mut analyzer = CppComplexityAnalyzer::new();
    let (cyclomatic, cognitive) = analyzer
        .analyze_complexity(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    // Convert to correct types for ComplexityMetrics::new
    // Create function complexity metrics
    let func_metrics = ComplexityMetrics::new(
        (cyclomatic & 0xFFFF) as u16, // Convert to u16 with clamping
        (cognitive & 0xFFFF) as u16,  // Convert to u16 with clamping
        0,                            // nesting_max (not calculated)
        std::cmp::min(items.len(), 65535) as u16 // lines (clamped to u16 max)
    );
    
    // Create a file complexity metrics object
    let file_complexity = crate::services::complexity::FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: func_metrics,
        functions: vec![
            crate::services::complexity::FunctionComplexity {
                name: path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_start: 1,
                line_end: std::cmp::min(items.len(), 65535) as u32,
                metrics: func_metrics,
            }
        ],
        classes: vec![],
    };
    
    let complexity_metrics = Some(file_complexity);

    // Return FileContext
    Ok(FileContext {
        path: path.display().to_string(),
        language: "cpp".to_string(),
        items,
        complexity_metrics,
    })
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "cpp-ast")]
    use super::*;
    #[cfg(feature = "cpp-ast")]
    use std::path::Path;

    #[cfg(feature = "cpp-ast")]
    const SIMPLE_CPP_FUNCTION: &str = r#"
#include <iostream>

// Simple function
int add(int a, int b) {
    return a + b;
}

// Main function
int main() {
    int result = add(5, 3);
    std::cout << "Result: " << result << std::endl;
    return 0;
}
"#;

    #[cfg(feature = "cpp-ast")]
    const CPP_CLASS_EXAMPLE: &str = r#"
#include <string>
#include <iostream>

class Person {
private:
    std::string name;
    int age;

public:
    Person(std::string n, int a) : name(n), age(a) {}

    void setName(std::string n) {
        name = n;
    }

    std::string getName() const {
        return name;
    }

    int getAge() const {
        return age;
    }

    virtual void display() const {
        std::cout << "Name: " << name << ", Age: " << age << std::endl;
    }
};

class Student : public Person {
private:
    std::string studentId;

public:
    Student(std::string n, int a, std::string id) 
        : Person(n, a), studentId(id) {}

    std::string getStudentId() const {
        return studentId;
    }

    void display() const override {
        std::cout << "Student - Name: " << getName() 
                  << ", Age: " << getAge() 
                  << ", ID: " << studentId << std::endl;
    }
};
"#;

    #[cfg(feature = "cpp-ast")]
    const CPP_COMPLEX_EXAMPLE: &str = r#"
#include <iostream>
#include <vector>
#include <algorithm>

namespace example {

// Template class
template<typename T>
class Container {
private:
    std::vector<T> data;

public:
    void add(const T& item) {
        data.push_back(item);
    }

    bool contains(const T& item) const {
        return std::find(data.begin(), data.end(), item) != data.end();
    }
    
    size_t size() const {
        return data.size();
    }
    
    // Template method
    template<typename Func>
    void forEach(Func func) {
        std::for_each(data.begin(), data.end(), func);
    }
};

// Enum class
enum class Color {
    Red,
    Green,
    Blue
};

// Function with complex control flow
int complexFunction(int a, int b) {
    int result = 0;
    
    // Lambda expression
    auto calculate = [](int x, int y) -> int {
        if (x > y) {
            return x * 2;
        } else {
            return y * 2;
        }
    };
    
    if (a > b) {
        try {
            if (a > 10) {
                result = calculate(a, b);
            } else {
                result = a;
            }
        } catch (const std::exception& e) {
            std::cerr << "Error: " << e.what() << std::endl;
            return -1;
        }
    } else if (b > a) {
        // Range-based for loop
        std::vector<int> values(b);
        for (int i = 0; i < b; i++) {
            values[i] = i;
        }
        
        for (const auto& val : values) {
            result += val;
            if (result > 100) {
                break;
            }
        }
    } else {
        switch (a) {
            case 0:
                result = 0;
                break;
            case 1:
                result = 1;
                break;
            default:
                result = a + b;
                break;
        }
    }
    
    return result;
}

} // namespace example

int main() {
    example::Container<int> container;
    container.add(5);
    container.add(10);
    
    int x = 10;
    int y = 20;
    int result = example::complexFunction(x, y);
    
    // C++11 type alias
    using IntContainer = example::Container<int>;
    IntContainer another;
    
    return 0;
}
"#;

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_simple_cpp_function_analysis() {
        let visitor = CppAstVisitor::new(Path::new("test.cpp"));
        let items = visitor
            .analyze_cpp_source(SIMPLE_CPP_FUNCTION)
            .expect("Should parse C++ functions");

        assert!(!items.is_empty(), "Should extract at least one AST item");

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(function_items.len(), 2, "Should extract two functions");

        // Check function names
        let func_names: Vec<_> = function_items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert!(func_names.contains(&"add"), "Should extract 'add' function");
        assert!(func_names.contains(&"main"), "Should extract 'main' function");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_class_analysis() {
        let visitor = CppAstVisitor::new(Path::new("person.cpp"));
        let items = visitor
            .analyze_cpp_source(CPP_CLASS_EXAMPLE)
            .expect("Should parse C++ classes");

        // Check for class definitions
        let class_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // Classes stored as Struct
            .collect();

        assert_eq!(class_items.len(), 2, "Should extract two classes");

        // Check class names
        let class_names: Vec<_> = class_items
            .iter()
            .filter_map(|item| match item {
                AstItem::Struct { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert!(class_names.contains(&"Person"), "Should extract 'Person' class");
        assert!(class_names.contains(&"Student"), "Should extract 'Student' class");

        // Check for methods
        let method_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. })) // Methods stored as Function
            .collect();

        assert!(!method_items.is_empty(), "Should extract methods");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_complex_example() {
        let visitor = CppAstVisitor::new(Path::new("complex.cpp"));
        let items = visitor
            .analyze_cpp_source(CPP_COMPLEX_EXAMPLE)
            .expect("Should parse complex C++ code");

        // Check for template class
        let template_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // GenericTypes stored as Struct
            .collect();

        assert!(!template_items.is_empty(), "Should extract template class");

        // Check for enum
        let enum_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Enum { .. }))
            .collect();

        assert_eq!(enum_items.len(), 1, "Should extract one enum");

        if let AstItem::Enum { name, .. } = &enum_items[0] {
            assert_eq!(name, "example::Color", "Should extract correct enum name with namespace");
        }

        // Check for type alias
        let alias_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // TypeAlias stored as Struct
            .collect();

        assert!(!alias_items.is_empty(), "Should extract type alias");

        // Check for complex function
        let complex_function: Vec<_> = items
            .iter()
            .filter(|item| match item {
                AstItem::Function { name, .. } => name.contains("complexFunction"),
                _ => false,
            })
            .collect();

        assert_eq!(complex_function.len(), 1, "Should extract complex function");
        
        // Check for namespace qualification
        if let AstItem::Function { name, .. } = &complex_function[0] {
            assert!(name.contains("example::"), "Should include namespace qualification");
        }
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_complexity_analysis() {
        let mut analyzer = CppComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(CPP_COMPLEX_EXAMPLE)
            .expect("Should analyze C++ complexity");

        // Complex example should have significant complexity
        assert!(cyclomatic > 5, "Cyclomatic complexity should be significant");
        assert!(cognitive > 5, "Cognitive complexity should be significant");
    }
}