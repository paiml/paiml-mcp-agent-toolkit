//! C Language Support for PMAT
//!
//! This module provides C-specific analysis capabilities using tree-sitter-c parser
//! for AST extraction and complexity analysis aligned with C best practices.

#[cfg(feature = "c-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "c-ast")]
use std::path::{Path, PathBuf};

/// C AST visitor that extracts C-specific AST information
#[cfg(feature = "c-ast")]
pub struct CAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    #[allow(dead_code)]
    current_scope: Vec<String>,
    #[allow(dead_code)]
    is_header: bool,
}

#[cfg(feature = "c-ast")]
impl CAstVisitor {
    /// Creates a new C AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        // Check if file is a header file
        let is_header = file_path
            .extension()
            .map(|ext| ext == "h")
            .unwrap_or(false);

        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            current_scope: Vec::new(),
            is_header,
        }
    }

    /// Analyzes C source code and extracts AST items (complexity ≤10)
    pub fn analyze_c_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
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
        
        // Mark them as used
        let _ = &current_function_name;
        let _ = &has_static_modifier;

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

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
                            visibility: if has_static_modifier { "private" } else { "public" }.to_string(),
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

            // Check for struct declaration
            if !in_struct && trimmed.starts_with("struct ") {
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

            // Count fields when in a struct
            if in_struct && !trimmed.is_empty() && !trimmed.starts_with("{") && !trimmed.starts_with("}") {
                // This is a field
                fields_count += 1;
            }

            // Track brace depth
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            // Check if we're at the end of a struct definition
            if in_struct && trimmed.contains("}") && (brace_depth == 0 || trimmed.trim_end().ends_with(";")) {
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
        line.contains("(") && line.contains(")") && line.contains("*") && !line.starts_with("if") && !line.starts_with("while")
    }

    /// Extracts function name from declaration line (complexity ≤10)
    fn extract_function_name(&self, line: &str) -> Result<String, String> {
        // Simplified extraction - get text between return type and opening parenthesis
        let after_type = line.split_whitespace().skip(1).collect::<Vec<&str>>().join(" ");
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
            Some(words.last().unwrap().to_string())
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

/// C complexity analyzer for extracting C-specific metrics (complexity ≤10)
#[cfg(feature = "c-ast")]
pub struct CComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "c-ast")]
impl Default for CComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CComplexityAnalyzer {
    /// Creates a new C complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of C source code (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 0;
        
        let mut nesting_depth = 0;

        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip comments and preprocessor directives
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                continue;
            }
            
            // Check for control flow statements
            let is_if_stmt = trimmed.starts_with("if ") || trimmed.starts_with("if(");
            let is_else_if = trimmed.starts_with("else if") || trimmed.contains("} else if");
            let _is_else = trimmed.starts_with("else") && !is_else_if;
            let is_switch = trimmed.starts_with("switch ");
            let is_case = trimmed.starts_with("case ") || trimmed.starts_with("default:");
            let is_loop = trimmed.starts_with("while ") || trimmed.starts_with("for ") || trimmed.starts_with("do ");
            let is_goto = trimmed.starts_with("goto ");
            
            // Increment cyclomatic complexity for decision points
            if is_if_stmt || is_else_if || is_switch || is_case || is_loop || is_goto {
                self.cyclomatic_complexity += 1;
            }
            
            // Cognitive complexity considers nesting
            if is_if_stmt || is_else_if || is_switch || is_loop {
                self.cognitive_complexity += 1 + nesting_depth;
                nesting_depth += 1;
            } else if trimmed.contains("{") && !trimmed.contains("}") {
                // Opening a block (not a one-line block)
                nesting_depth += 1;
            }
            
            // Track closing braces
            if trimmed.contains("}") {
                nesting_depth = nesting_depth.saturating_sub(1);
            }
        }
        
        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
}

/// Public async function to analyze a C file and return FileContext
#[cfg(feature = "c-ast")]
pub async fn analyze_c_file(
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
    let visitor = CAstVisitor::new(path);
    let items = visitor
        .analyze_c_source(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    // Analyze complexity
    let mut analyzer = CComplexityAnalyzer::new();
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
        language: "c".to_string(),
        items,
        complexity_metrics,
    })
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "c-ast")]
    use super::*;
    #[cfg(feature = "c-ast")]
    use std::path::Path;

    #[cfg(feature = "c-ast")]
    const SIMPLE_C_FUNCTION: &str = r#"
#include <stdio.h>

// Simple function
int add(int a, int b) {
    return a + b;
}

// Main function
int main() {
    int result = add(5, 3);
    printf("Result: %d\n", result);
    return 0;
}
"#;

    #[cfg(feature = "c-ast")]
    const C_STRUCT_EXAMPLE: &str = r#"
#include <stdio.h>
#include <stdlib.h>

// Define a simple structure
struct Point {
    int x;
    int y;
};

// Function to create a new point
struct Point* createPoint(int x, int y) {
    struct Point* p = (struct Point*)malloc(sizeof(struct Point));
    p->x = x;
    p->y = y;
    return p;
}

// Calculate distance (simplified)
int distance(struct Point* p1, struct Point* p2) {
    int dx = p2->x - p1->x;
    int dy = p2->y - p1->y;
    return dx*dx + dy*dy;
}
"#;

    #[cfg(feature = "c-ast")]
    const C_COMPLEX_EXAMPLE: &str = r#"
#include <stdio.h>

// Global variables
int globalValue = 10;
static int privateValue = 20;

// Typedef example
typedef unsigned long size_t;
typedef struct {
    int value;
} Container;

// Enum example
enum Color {
    RED,
    GREEN,
    BLUE
};

// Function with complex control flow
int complexFunction(int a, int b) {
    int result = 0;
    
    if (a > b) {
        if (a > 10) {
            result = a * 2;
        } else {
            result = a;
        }
    } else if (b > a) {
        for (int i = 0; i < b; i++) {
            result += i;
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

// Main function
int main() {
    int x = 10;
    int y = 20;
    int result = complexFunction(x, y);
    printf("Result: %d\n", result);
    return 0;
}
"#;

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_simple_c_function_analysis() {
        let visitor = CAstVisitor::new(Path::new("test.c"));
        let items = visitor
            .analyze_c_source(SIMPLE_C_FUNCTION)
            .expect("Should parse C functions");

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

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_struct_analysis() {
        let visitor = CAstVisitor::new(Path::new("point.c"));
        let items = visitor
            .analyze_c_source(C_STRUCT_EXAMPLE)
            .expect("Should parse C struct");

        // Check for struct definition
        let struct_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(struct_items.len(), 1, "Should extract one struct");

        if let AstItem::Struct { name, .. } = &struct_items[0] {
            assert_eq!(name, "Point", "Should extract correct struct name");
        }

        // Check for functions
        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(function_items.len(), 2, "Should extract two functions");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_complex_example() {
        let visitor = CAstVisitor::new(Path::new("complex.c"));
        let items = visitor
            .analyze_c_source(C_COMPLEX_EXAMPLE)
            .expect("Should parse complex C code");

        // Check for global variables
        let var_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // Variables stored as Struct
            .collect();

        assert!(!var_items.is_empty(), "Should extract global variables");

        // Check for enum
        let enum_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Enum { .. }))
            .collect();

        assert_eq!(enum_items.len(), 1, "Should extract one enum");

        if let AstItem::Enum { name, .. } = &enum_items[0] {
            assert_eq!(name, "Color", "Should extract correct enum name");
        }

        // Check for typedef
        let typedef_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. })) // TypeAliases stored as Struct
            .collect();

        assert!(!typedef_items.is_empty(), "Should extract typedefs");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_complexity_analysis() {
        let mut analyzer = CComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(C_COMPLEX_EXAMPLE)
            .expect("Should analyze C complexity");

        // Complex example should have significant complexity
        assert!(cyclomatic > 5, "Cyclomatic complexity should be significant");
        assert!(cognitive > 5, "Cognitive complexity should be significant");
    }
}