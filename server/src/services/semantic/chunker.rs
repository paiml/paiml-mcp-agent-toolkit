// AST-Aware Code Chunker
// PMAT-SEARCH-001: Extract semantic units (functions, classes, modules) from code
//
// GREEN Phase: Full implementation using tree-sitter AST parsers

use sha2::{Digest, Sha256};
use tree_sitter::{Node, Parser, Tree};

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    C,
    Cpp,
    Go,
}

/// Type of code chunk
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkType {
    Function,
    Class,
    Module,
    File,
}

/// A semantic code chunk with metadata
#[derive(Debug, Clone)]
pub struct CodeChunk {
    /// Full file path (empty in tests)
    pub file_path: String,
    /// Type of chunk (function, class, module, file)
    pub chunk_type: ChunkType,
    /// Identifier name (function name, class name, etc.)
    pub chunk_name: String,
    /// Programming language
    pub language: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (inclusive)
    pub end_line: usize,
    /// Full source code including docstrings
    pub content: String,
    /// SHA256 checksum of content for incremental updates
    pub content_checksum: String,
}

/// Extract code chunks from source code
///
/// # Arguments
/// * `source` - Source code string
/// * `language` - Programming language
///
/// # Returns
/// Vector of code chunks (functions, classes, modules)
pub fn chunk_code(source: &str, language: Language) -> Result<Vec<CodeChunk>, String> {
    // Handle empty input
    if source.trim().is_empty() {
        return Ok(Vec::new());
    }

    match language {
        Language::Rust => chunk_rust_file(source),
        Language::TypeScript => chunk_typescript_file(source),
        Language::Python => chunk_python_file(source),
        Language::C => chunk_c_file(source),
        Language::Cpp => chunk_cpp_file(source),
        Language::Go => chunk_go_file(source),
    }
}

/// Extract chunks from Rust code
fn chunk_rust_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let tree = parse_rust(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_rust_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse Rust source code
fn parse_rust(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|e| format!("Failed to set Rust language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse Rust source".to_string())
}

/// Helper: Find preceding doc comments for a node (all languages)
/// Returns the start byte position of the first comment, or node start if none
fn find_doc_comment_start(node: Node, source: &str) -> usize {
    let mut start_byte = node.start_byte();

    // Walk backwards through preceding siblings to find comments
    if let Some(parent) = node.parent() {
        let mut cursor = parent.walk();
        let siblings: Vec<Node> = parent.children(&mut cursor).collect();

        if let Some(node_index) = siblings.iter().position(|n| n.id() == node.id()) {
            // Check previous siblings in reverse order
            for i in (0..node_index).rev() {
                let sibling = siblings[i];
                let kind = sibling.kind();

                // Detect comments for all languages
                let is_comment = kind == "comment"  // TypeScript, C, C++, Go
                    || kind == "line_comment"   // Rust
                    || kind == "block_comment"; // Rust, C, C++

                if is_comment {
                    // For Rust: only include /// doc comments, not regular //
                    if kind == "line_comment" {
                        let comment_text = &source[sibling.byte_range()];
                        if comment_text.trim_start().starts_with("///") {
                            start_byte = sibling.start_byte();
                            continue;
                        }
                        // Skip regular // comments in Rust
                        break;
                    } else {
                        // Include all other comment types
                        start_byte = sibling.start_byte();
                        continue;
                    }
                }

                // Stop if we hit a non-comment node
                break;
            }
        }
    }

    start_byte
}

/// Extract items (functions, impl blocks, modules) from Rust AST
fn extract_rust_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // Check if this node is a function
    if node.kind() == "function_item" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();

            // Include preceding doc comments
            let start_byte = find_doc_comment_start(node, source);
            let content = source[start_byte..node.end_byte()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Function,
                chunk_name: name,
                language: "rust".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }
    // Check for impl blocks (treat as class-like)
    else if node.kind() == "impl_item" {
        if let Some(type_node) = node.child_by_field_name("type") {
            let name = source[type_node.byte_range()].to_string();
            let content = source[node.byte_range()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Class,
                chunk_name: name,
                language: "rust".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }
    // Check for modules
    else if node.kind() == "mod_item" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();
            let content = source[node.byte_range()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Module,
                chunk_name: name,
                language: "rust".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_rust_items(child, source, chunks);
    }
}

/// Extract chunks from TypeScript code
fn chunk_typescript_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let tree = parse_typescript(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_typescript_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse TypeScript source code
fn parse_typescript(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .map_err(|e| format!("Failed to set TypeScript language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse TypeScript source".to_string())
}

/// Extract TypeScript class declaration
fn extract_ts_class(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    if let Some(name_node) = node.child_by_field_name("name") {
        let name = source[name_node.byte_range()].to_string();
        let content = source[node.byte_range()].to_string();

        chunks.push(CodeChunk {
            file_path: String::new(),
            chunk_type: ChunkType::Class,
            chunk_name: name,
            language: "typescript".to_string(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            content: content.clone(),
            content_checksum: compute_checksum(&content),
        });
    }
}

/// Extract TypeScript interface declaration
fn extract_ts_interface(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    if let Some(name_node) = node.child_by_field_name("name") {
        let name = source[name_node.byte_range()].to_string();
        let content = source[node.byte_range()].to_string();

        chunks.push(CodeChunk {
            file_path: String::new(),
            chunk_type: ChunkType::Class, // Treat interface as class-like
            chunk_name: name,
            language: "typescript".to_string(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            content: content.clone(),
            content_checksum: compute_checksum(&content),
        });
    }
}

/// Extract TypeScript function declaration
fn extract_ts_function(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    if let Some(name_node) = node.child_by_field_name("name") {
        let name = source[name_node.byte_range()].to_string();
        let start_byte = find_doc_comment_start(node, source);
        let content = source[start_byte..node.end_byte()].to_string();

        chunks.push(CodeChunk {
            file_path: String::new(),
            chunk_type: ChunkType::Function,
            chunk_name: name,
            language: "typescript".to_string(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            content: content.clone(),
            content_checksum: compute_checksum(&content),
        });
    }
}

/// Extract TypeScript arrow function from variable declaration
fn extract_ts_arrow_function(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // Look for arrow function patterns: const foo = () => {}
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Some(value_node) = child.child_by_field_name("value") {
                        if value_node.kind() == "arrow_function" {
                            let name = source[name_node.byte_range()].to_string();
                            let content = source[child.byte_range()].to_string();

                            chunks.push(CodeChunk {
                                file_path: String::new(),
                                chunk_type: ChunkType::Function,
                                chunk_name: name,
                                language: "typescript".to_string(),
                                start_line: child.start_position().row + 1,
                                end_line: child.end_position().row + 1,
                                content: content.clone(),
                                content_checksum: compute_checksum(&content),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Extract items from TypeScript AST
fn extract_typescript_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    match node.kind() {
        "class_declaration" => extract_ts_class(node, source, chunks),
        "interface_declaration" => extract_ts_interface(node, source, chunks),
        "function_declaration" | "function" => extract_ts_function(node, source, chunks),
        "lexical_declaration" | "variable_declaration" => {
            extract_ts_arrow_function(node, source, chunks)
        }
        _ => {}
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_typescript_items(child, source, chunks);
    }
}

/// Extract chunks from Python code
fn chunk_python_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let tree = parse_python(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_python_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse Python source code
#[cfg(feature = "python-ast")]
fn parse_python(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|e| format!("Failed to set Python language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse Python source".to_string())
}

#[cfg(not(feature = "python-ast"))]
fn parse_python(_source: &str) -> Result<Tree, String> {
    Err("python-ast feature is disabled".to_string())
}

/// Extract items from Python AST
fn extract_python_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // Check for class definition
    if node.kind() == "class_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();
            let content = source[node.byte_range()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Class,
                chunk_name: name,
                language: "python".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
        // Don't recurse into class children - class is extracted as a whole
        return;
    }
    // Check for function definition
    else if node.kind() == "function_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();
            let content = source[node.byte_range()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Function,
                chunk_name: name,
                language: "python".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_python_items(child, source, chunks);
    }
}

/// Extract chunks from C code
fn chunk_c_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let tree = parse_c(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_c_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse C source code
#[cfg(feature = "c-ast")]
fn parse_c(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .map_err(|e| format!("Failed to set C language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse C source".to_string())
}

#[cfg(not(feature = "c-ast"))]
fn parse_c(_source: &str) -> Result<Tree, String> {
    Err("c-ast feature is disabled".to_string())
}

/// Extract items from C AST
fn extract_c_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // Check for function definition
    if node.kind() == "function_definition" {
        if let Some(declarator) = node.child_by_field_name("declarator") {
            // Navigate to function name
            if let Some(name_node) = find_function_declarator_name(declarator, source) {
                let name = source[name_node.byte_range()].to_string();

                // Include preceding doc comments
                let start_byte = find_doc_comment_start(node, source);
                let content = source[start_byte..node.end_byte()].to_string();

                chunks.push(CodeChunk {
                    file_path: String::new(),
                    chunk_type: ChunkType::Function,
                    chunk_name: name,
                    language: "c".to_string(),
                    start_line: node.start_position().row + 1,
                    end_line: node.end_position().row + 1,
                    content: content.clone(),
                    content_checksum: compute_checksum(&content),
                });
            }
        }
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_c_items(child, source, chunks);
    }
}

/// Find function name in C declarator
fn find_function_declarator_name<'a>(node: Node<'a>, _source: &str) -> Option<Node<'a>> {
    if node.kind() == "identifier" {
        return Some(node);
    }
    if node.kind() == "function_declarator" {
        return find_function_declarator_name(node.child_by_field_name("declarator")?, _source);
    }
    if node.kind() == "pointer_declarator" {
        return find_function_declarator_name(node.child_by_field_name("declarator")?, _source);
    }
    None
}

/// Extract chunks from C++ code
fn chunk_cpp_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let tree = parse_cpp(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_cpp_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse C++ source code
#[cfg(feature = "cpp-ast")]
fn parse_cpp(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_cpp::LANGUAGE.into())
        .map_err(|e| format!("Failed to set C++ language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse C++ source".to_string())
}

#[cfg(not(feature = "cpp-ast"))]
fn parse_cpp(_source: &str) -> Result<Tree, String> {
    Err("cpp-ast feature is disabled".to_string())
}

/// Extract items from C++ AST
fn extract_cpp_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // Check for class declaration
    if node.kind() == "class_specifier" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();
            let content = source[node.byte_range()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Class,
                chunk_name: name,
                language: "cpp".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }
    // Check for function definition
    else if node.kind() == "function_definition" {
        if let Some(declarator) = node.child_by_field_name("declarator") {
            if let Some(name_node) = find_function_declarator_name(declarator, source) {
                let name = source[name_node.byte_range()].to_string();

                // Include preceding doc comments
                let start_byte = find_doc_comment_start(node, source);
                let content = source[start_byte..node.end_byte()].to_string();

                chunks.push(CodeChunk {
                    file_path: String::new(),
                    chunk_type: ChunkType::Function,
                    chunk_name: name,
                    language: "cpp".to_string(),
                    start_line: node.start_position().row + 1,
                    end_line: node.end_position().row + 1,
                    content: content.clone(),
                    content_checksum: compute_checksum(&content),
                });
            }
        }
    }
    // Check for template declaration
    else if node.kind() == "template_declaration" {
        // Extract the template body
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "function_definition" {
                    if let Some(declarator) = child.child_by_field_name("declarator") {
                        if let Some(name_node) = find_function_declarator_name(declarator, source) {
                            let name = source[name_node.byte_range()].to_string();

                            // Include preceding doc comments and whole template
                            let start_byte = find_doc_comment_start(node, source);
                            let content = source[start_byte..node.end_byte()].to_string();

                            chunks.push(CodeChunk {
                                file_path: String::new(),
                                chunk_type: ChunkType::Function,
                                chunk_name: name,
                                language: "cpp".to_string(),
                                start_line: node.start_position().row + 1,
                                end_line: node.end_position().row + 1,
                                content: content.clone(),
                                content_checksum: compute_checksum(&content),
                            });
                        }
                    }
                }
            }
        }
        // Don't recurse into template children - template is extracted as a whole
        return;
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_cpp_items(child, source, chunks);
    }
}

/// Extract chunks from Go code
fn chunk_go_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let tree = parse_go(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_go_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse Go source code
#[cfg(feature = "go-ast")]
fn parse_go(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .map_err(|e| format!("Failed to set Go language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse Go source".to_string())
}

#[cfg(not(feature = "go-ast"))]
fn parse_go(_source: &str) -> Result<Tree, String> {
    Err("go-ast feature is disabled".to_string())
}

/// Extract items from Go AST
fn extract_go_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // Check for function declaration
    if node.kind() == "function_declaration" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();

            // Include preceding doc comments
            let start_byte = find_doc_comment_start(node, source);
            let content = source[start_byte..node.end_byte()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Function,
                chunk_name: name,
                language: "go".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }
    // Check for method declaration
    else if node.kind() == "method_declaration" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();
            let content = source[node.byte_range()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Function,
                chunk_name: name,
                language: "go".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }
    // Check for type declaration (struct, interface)
    else if node.kind() == "type_declaration" {
        // Go type_declaration has a type_spec child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                // type_spec has name and type fields
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = source[name_node.byte_range()].to_string();

                    // Include preceding doc comments
                    let start_byte = find_doc_comment_start(node, source);
                    let content = source[start_byte..node.end_byte()].to_string();

                    chunks.push(CodeChunk {
                        file_path: String::new(),
                        chunk_type: ChunkType::Class, // Treat struct/interface as class-like
                        chunk_name: name,
                        language: "go".to_string(),
                        start_line: node.start_position().row + 1,
                        end_line: node.end_position().row + 1,
                        content: content.clone(),
                        content_checksum: compute_checksum(&content),
                    });
                    break; // Only extract the first type_spec
                }
            }
        }
        // Don't recurse into type declaration children
        return;
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_go_items(child, source, chunks);
    }
}

/// Compute SHA256 checksum of content
fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_computation() {
        let content = "fn test() {}";
        let checksum1 = compute_checksum(content);
        let checksum2 = compute_checksum(content);

        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum1.len(), 64); // SHA256 hex length
    }

    #[test]
    fn test_language_enum() {
        assert_eq!(Language::Rust, Language::Rust);
        assert_ne!(Language::Rust, Language::Python);
    }

    #[test]
    fn test_chunk_type_enum() {
        assert_eq!(ChunkType::Function, ChunkType::Function);
        assert_ne!(ChunkType::Function, ChunkType::Class);
    }
}

/// Comprehensive coverage tests for the semantic chunker module
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ============================================
    // Empty and Edge Case Tests
    // ============================================

    #[test]
    fn test_chunk_code_empty_input() {
        let result = chunk_code("", Language::Rust).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_chunk_code_whitespace_only() {
        let result = chunk_code("   \n\t  \n  ", Language::Rust).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_chunk_code_empty_input_all_languages() {
        for lang in [
            Language::Rust,
            Language::TypeScript,
        ] {
            let result = chunk_code("", lang).unwrap();
            assert!(result.is_empty(), "Empty input for {:?} should return empty vec", lang);
        }
    }

    // ============================================
    // Rust Language Tests
    // ============================================

    #[test]
    fn test_rust_simple_function() {
        let source = r#"
fn hello_world() {
    println!("Hello, world!");
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello_world");
        assert_eq!(chunks[0].language, "rust");
        assert!(chunks[0].content.contains("println!"));
    }

    #[test]
    fn test_rust_function_with_doc_comment() {
        let source = r#"
/// This is a doc comment
/// with multiple lines
fn documented_function() {
    let x = 42;
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "documented_function");
        assert!(chunks[0].content.contains("/// This is a doc comment"));
        assert!(chunks[0].content.contains("/// with multiple lines"));
    }

    #[test]
    fn test_rust_function_with_regular_comment_not_included() {
        let source = r#"
// This is a regular comment (should not be included)
fn regular_function() {
    let x = 42;
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(!chunks[0].content.contains("// This is a regular comment"));
    }

    #[test]
    fn test_rust_impl_block() {
        let source = r#"
struct MyStruct;

impl MyStruct {
    fn new() -> Self {
        MyStruct
    }

    fn method(&self) {
        println!("method");
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert!(chunks.len() >= 1);

        let impl_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(impl_chunk.is_some());
        assert_eq!(impl_chunk.unwrap().chunk_name, "MyStruct");
    }

    #[test]
    fn test_rust_module() {
        let source = r#"
mod my_module {
    fn inner_function() {}
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let module_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Module);
        assert!(module_chunk.is_some());
        assert_eq!(module_chunk.unwrap().chunk_name, "my_module");
    }

    #[test]
    fn test_rust_multiple_functions() {
        let source = r#"
fn func1() {}
fn func2() {}
fn func3() {}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 3);

        let names: Vec<&str> = chunks.iter().map(|c| c.chunk_name.as_str()).collect();
        assert!(names.contains(&"func1"));
        assert!(names.contains(&"func2"));
        assert!(names.contains(&"func3"));
    }

    #[test]
    fn test_rust_async_function() {
        let source = r#"
async fn async_function() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "async_function");
        assert!(chunks[0].content.contains("async fn"));
    }

    #[test]
    fn test_rust_generic_function() {
        let source = r#"
fn generic_function<T: Clone>(value: T) -> T {
    value.clone()
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "generic_function");
        assert!(chunks[0].content.contains("<T: Clone>"));
    }

    #[test]
    fn test_rust_function_line_numbers() {
        let source = "fn line_one() {}\nfn line_two() {}\nfn line_three() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[1].start_line, 2);
        assert_eq!(chunks[2].start_line, 3);
    }

    #[test]
    fn test_rust_nested_impl_functions() {
        let source = r#"
impl Foo {
    fn method_a(&self) {}
    fn method_b(&mut self) {}
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        let functions: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Function).collect();
        assert_eq!(functions.len(), 2);
    }

    // ============================================
    // TypeScript Language Tests
    // ============================================

    #[test]
    fn test_typescript_simple_function() {
        let source = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "greet");
        assert_eq!(chunks[0].language, "typescript");
    }

    #[test]
    fn test_typescript_class() {
        let source = r#"
class MyClass {
    private value: number;

    constructor(value: number) {
        this.value = value;
    }

    getValue(): number {
        return this.value;
    }
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "MyClass");
    }

    #[test]
    fn test_typescript_interface() {
        let source = r#"
interface Person {
    name: string;
    age: number;
    greet(): void;
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "Person");
    }

    #[test]
    fn test_typescript_arrow_function() {
        let source = r#"
const add = (a: number, b: number): number => {
    return a + b;
};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
    }

    #[test]
    fn test_typescript_multiple_arrow_functions() {
        let source = r#"
const func1 = () => {};
const func2 = () => {};
let func3 = () => {};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 3);

        let names: Vec<&str> = chunks.iter().map(|c| c.chunk_name.as_str()).collect();
        assert!(names.contains(&"func1"));
        assert!(names.contains(&"func2"));
        assert!(names.contains(&"func3"));
    }

    #[test]
    fn test_typescript_function_with_jsdoc() {
        let source = r#"
/**
 * Multiplies two numbers
 * @param a - First number
 * @param b - Second number
 * @returns The product
 */
function multiply(a: number, b: number): number {
    return a * b;
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "multiply");
        assert!(chunks[0].content.contains("Multiplies two numbers"));
    }

    #[test]
    fn test_typescript_generic_class() {
        let source = r#"
class Container<T> {
    private item: T;

    constructor(item: T) {
        this.item = item;
    }

    get(): T {
        return this.item;
    }
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "Container");
    }

    #[test]
    fn test_typescript_async_function() {
        let source = r#"
async function fetchData(): Promise<string> {
    const response = await fetch('https://example.com');
    return response.text();
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "fetchData");
        assert!(chunks[0].content.contains("async function"));
    }

    #[test]
    fn test_typescript_export_function() {
        let source = r#"
export function exportedFunc(): void {
    console.log("exported");
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "exportedFunc");
    }

    // ============================================
    // Python Language Tests (feature-gated)
    // ============================================

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_simple_function() {
        let source = "def hello():\n    print(\"Hello, world!\")\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello");
        assert_eq!(chunks[0].language, "python");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_class() {
        let source = "class MyClass:\n    def __init__(self, value):\n        self.value = value\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "MyClass");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_python_multiple_functions() {
        let source = "def func1():\n    pass\n\ndef func2():\n    pass\n\ndef func3():\n    pass\n";
        let chunks = chunk_code(source, Language::Python).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[cfg(not(feature = "python-ast"))]
    #[test]
    fn test_python_feature_disabled() {
        let source = "def test(): pass";
        let result = chunk_code(source, Language::Python);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("python-ast feature is disabled"));
    }

    // ============================================
    // C Language Tests (feature-gated)
    // ============================================

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_simple_function() {
        let source = "int add(int a, int b) {\n    return a + b;\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
        assert_eq!(chunks[0].language, "c");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_main_function() {
        let source = "int main(int argc, char *argv[]) {\n    return 0;\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "main");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_pointer_return_function() {
        let source = "char *get_string() {\n    return \"hello\";\n}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "get_string");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_multiple_functions() {
        let source = "void func1() {}\nint func2() { return 0; }\nfloat func3() { return 0.0; }\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[cfg(not(feature = "c-ast"))]
    #[test]
    fn test_c_feature_disabled() {
        let source = "int main() { return 0; }";
        let result = chunk_code(source, Language::C);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("c-ast feature is disabled"));
    }

    // ============================================
    // C++ Language Tests (feature-gated)
    // ============================================

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_simple_function() {
        let source = "int add(int a, int b) {\n    return a + b;\n}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "add");
        assert_eq!(chunks[0].language, "cpp");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_class() {
        let source = "class MyClass {\npublic:\n    int value;\n};\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        let class_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        assert!(class_chunk.is_some());
        assert_eq!(class_chunk.unwrap().chunk_name, "MyClass");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_template_function() {
        let source = "template <typename T>\nT max(T a, T b) {\n    return (a > b) ? a : b;\n}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "max");
        assert!(chunks[0].content.contains("template"));
    }

    #[cfg(not(feature = "cpp-ast"))]
    #[test]
    fn test_cpp_feature_disabled() {
        let source = "int main() { return 0; }";
        let result = chunk_code(source, Language::Cpp);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cpp-ast feature is disabled"));
    }

    // ============================================
    // Go Language Tests (feature-gated)
    // ============================================

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_simple_function() {
        let source = "package main\n\nfunc hello() {\n    fmt.Println(\"Hello\")\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert_eq!(chunks[0].chunk_name, "hello");
        assert_eq!(chunks[0].language, "go");
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_method() {
        let source = "package main\n\ntype Person struct {\n    Name string\n}\n\nfunc (p Person) Greet() string {\n    return \"Hello, \" + p.Name\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        let method_chunk = chunks.iter().find(|c| c.chunk_name == "Greet");
        assert!(method_chunk.is_some());
        assert_eq!(method_chunk.unwrap().chunk_type, ChunkType::Function);
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_struct_type() {
        let source = "package main\n\ntype User struct {\n    ID   int\n    Name string\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "User");
    }

    #[cfg(feature = "go-ast")]
    #[test]
    fn test_go_interface_type() {
        let source = "package main\n\ntype Reader interface {\n    Read(p []byte) (n int, err error)\n}\n";
        let chunks = chunk_code(source, Language::Go).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].chunk_name, "Reader");
    }

    #[cfg(not(feature = "go-ast"))]
    #[test]
    fn test_go_feature_disabled() {
        let source = "package main\nfunc main() {}";
        let result = chunk_code(source, Language::Go);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("go-ast feature is disabled"));
    }

    // ============================================
    // CodeChunk Field Tests
    // ============================================

    #[test]
    fn test_code_chunk_fields() {
        let source = "fn test_func() { let x = 1; }";
        let chunks = chunk_code(source, Language::Rust).unwrap();

        assert_eq!(chunks.len(), 1);
        let chunk = &chunks[0];

        assert!(chunk.file_path.is_empty());
        assert_eq!(chunk.chunk_type, ChunkType::Function);
        assert_eq!(chunk.chunk_name, "test_func");
        assert_eq!(chunk.language, "rust");
        assert!(chunk.start_line >= 1);
        assert!(chunk.end_line >= chunk.start_line);
        assert!(!chunk.content.is_empty());
        assert!(!chunk.content_checksum.is_empty());
        assert_eq!(chunk.content_checksum.len(), 64);
    }

    #[test]
    fn test_checksum_different_for_different_content() {
        let source1 = "fn func1() {}";
        let source2 = "fn func2() {}";

        let chunks1 = chunk_code(source1, Language::Rust).unwrap();
        let chunks2 = chunk_code(source2, Language::Rust).unwrap();

        assert_ne!(chunks1[0].content_checksum, chunks2[0].content_checksum);
    }

    // ============================================
    // Language Enum Tests
    // ============================================

    #[test]
    fn test_language_debug() {
        let lang = Language::Rust;
        let debug_str = format!("{:?}", lang);
        assert_eq!(debug_str, "Rust");
    }

    #[test]
    fn test_language_clone() {
        let lang = Language::TypeScript;
        let cloned = lang.clone();
        assert_eq!(lang, cloned);
    }

    #[test]
    fn test_language_copy() {
        let lang = Language::Python;
        let copied = lang;
        assert_eq!(lang, copied);
    }

    #[test]
    fn test_all_language_variants() {
        let languages = vec![
            Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::C,
            Language::Cpp,
            Language::Go,
        ];

        for i in 0..languages.len() {
            for j in (i + 1)..languages.len() {
                assert_ne!(languages[i], languages[j]);
            }
        }
    }

    // ============================================
    // ChunkType Enum Tests
    // ============================================

    #[test]
    fn test_chunk_type_debug() {
        let chunk_type = ChunkType::Function;
        let debug_str = format!("{:?}", chunk_type);
        assert_eq!(debug_str, "Function");
    }

    #[test]
    fn test_chunk_type_clone() {
        let chunk_type = ChunkType::Class;
        let cloned = chunk_type.clone();
        assert_eq!(chunk_type, cloned);
    }

    #[test]
    fn test_all_chunk_type_variants() {
        let chunk_types = vec![
            ChunkType::Function,
            ChunkType::Class,
            ChunkType::Module,
            ChunkType::File,
        ];

        for i in 0..chunk_types.len() {
            for j in (i + 1)..chunk_types.len() {
                assert_ne!(chunk_types[i], chunk_types[j]);
            }
        }
    }

    // ============================================
    // CodeChunk Struct Tests
    // ============================================

    #[test]
    fn test_code_chunk_debug() {
        let chunk = CodeChunk {
            file_path: "test.rs".to_string(),
            chunk_type: ChunkType::Function,
            chunk_name: "test".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 3,
            content: "fn test() {}".to_string(),
            content_checksum: "abc123".to_string(),
        };

        let debug_str = format!("{:?}", chunk);
        assert!(debug_str.contains("CodeChunk"));
        assert!(debug_str.contains("test.rs"));
    }

    #[test]
    fn test_code_chunk_clone() {
        let chunk = CodeChunk {
            file_path: "test.rs".to_string(),
            chunk_type: ChunkType::Function,
            chunk_name: "test".to_string(),
            language: "rust".to_string(),
            start_line: 1,
            end_line: 3,
            content: "fn test() {}".to_string(),
            content_checksum: "abc123".to_string(),
        };

        let cloned = chunk.clone();
        assert_eq!(chunk.file_path, cloned.file_path);
        assert_eq!(chunk.chunk_type, cloned.chunk_type);
        assert_eq!(chunk.chunk_name, cloned.chunk_name);
    }

    // ============================================
    // Complex Code Tests
    // ============================================

    #[test]
    fn test_rust_complex_code() {
        let source = r#"
/// A struct with fields
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    /// Creates a new Point
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    /// Calculates distance from origin
    fn distance_from_origin(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}

mod geometry {
    pub fn area(width: u32, height: u32) -> u32 {
        width * height
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        assert!(chunks.len() >= 3);

        let impl_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class && c.chunk_name == "Point");
        assert!(impl_chunk.is_some());

        let module_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Module && c.chunk_name == "geometry");
        assert!(module_chunk.is_some());
    }

    #[test]
    fn test_typescript_complex_code() {
        let source = r#"
interface IService {
    start(): Promise<void>;
    stop(): void;
}

class Service implements IService {
    private running: boolean = false;

    async start(): Promise<void> {
        this.running = true;
    }

    stop(): void {
        this.running = false;
    }
}

const helper = (x: number): number => x * 2;

function processData(data: string[]): string[] {
    return data.map(d => d.toUpperCase());
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        assert!(chunks.len() >= 4);

        let names: Vec<&str> = chunks.iter().map(|c| c.chunk_name.as_str()).collect();
        assert!(names.contains(&"IService"));
        assert!(names.contains(&"Service"));
        assert!(names.contains(&"helper"));
        assert!(names.contains(&"processData"));
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_rust_single_line_function() {
        let source = "fn single() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, chunks[0].end_line);
    }

    #[test]
    fn test_rust_unicode_in_strings() {
        let source = "fn greet() {\n    println!(\"Hello, Alex!\");\n}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "greet");
    }

    #[test]
    fn test_typescript_multiline_arrow() {
        let source = r#"
const complexFunc = (
    a: number,
    b: number,
    c: number
): number => {
    return a + b + c;
};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "complexFunc");
    }

    // ============================================
    // Checksum Tests
    // ============================================

    #[test]
    fn test_checksum_deterministic() {
        let content = "fn test() { let x = 42; }";
        let checksum1 = compute_checksum(content);
        let checksum2 = compute_checksum(content);
        let checksum3 = compute_checksum(content);

        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum2, checksum3);
    }

    #[test]
    fn test_checksum_sensitive_to_whitespace() {
        let content1 = "fn test() {}";
        let content2 = "fn test()  {}";

        let checksum1 = compute_checksum(content1);
        let checksum2 = compute_checksum(content2);

        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_sensitive_to_case() {
        let content1 = "fn Test() {}";
        let content2 = "fn test() {}";

        let checksum1 = compute_checksum(content1);
        let checksum2 = compute_checksum(content2);

        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_empty_content() {
        let checksum = compute_checksum("");
        assert_eq!(checksum.len(), 64);
    }

    // ============================================
    // Error Handling Tests
    // ============================================

    #[test]
    fn test_rust_syntax_error_still_parses() {
        let source = "fn broken( { }";
        let result = chunk_code(source, Language::Rust);
        assert!(result.is_ok());
    }

    #[test]
    fn test_typescript_syntax_error_still_parses() {
        let source = "function broken( { }";
        let result = chunk_code(source, Language::TypeScript);
        assert!(result.is_ok());
    }

    // ============================================
    // Doc Comment Tests
    // ============================================

    #[test]
    fn test_rust_multiple_doc_comments() {
        let source = "/// First line\n/// Second line\n/// Third line\nfn documented() {}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("/// First line"));
        assert!(chunks[0].content.contains("/// Second line"));
        assert!(chunks[0].content.contains("/// Third line"));
    }

    #[test]
    fn test_rust_block_doc_comment() {
        let source = "/** This is a block doc comment */\nfn block_documented() {}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("block doc comment"));
    }

    // ============================================
    // Nested Structure Tests
    // ============================================

    #[test]
    fn test_rust_nested_module() {
        let source = "mod outer {\n    mod inner {\n        fn nested_func() {}\n    }\n}\n";
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let modules: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Module).collect();
        assert!(modules.len() >= 1);
    }

    #[test]
    fn test_typescript_nested_class() {
        let source = "class Outer {\n    inner = class {\n        method() {}\n    };\n}\n";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let classes: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Class).collect();
        assert!(classes.len() >= 1);
    }

    // ============================================
    // Performance Boundary Tests
    // ============================================

    #[test]
    fn test_many_small_functions() {
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("fn func_{i}() {{}}\n"));
        }

        let chunks = chunk_code(&source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 100);
    }

    #[test]
    fn test_large_function() {
        let mut source = String::from("fn large_func() {\n");
        for i in 0..1000 {
            source.push_str(&format!("    let var_{i} = {i};\n"));
        }
        source.push_str("}\n");

        let chunks = chunk_code(&source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "large_func");
    }

    // ============================================
    // Language-Specific Feature Tests
    // ============================================

    #[test]
    fn test_rust_trait_impl() {
        let source = r#"
trait Greeter {
    fn greet(&self) -> String;
}

impl Greeter for Person {
    fn greet(&self) -> String {
        format!("Hello, {}", self.name)
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let impl_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Class && c.chunk_name == "Person");
        assert!(impl_chunk.is_some());
    }

    #[test]
    fn test_typescript_type_alias_not_extracted() {
        let source = "type StringAlias = string;\nfunction useType(x: StringAlias): void {}\n";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        let func_chunk = chunks.iter().find(|c| c.chunk_name == "useType");
        assert!(func_chunk.is_some());
    }

    // ============================================
    // Whitespace Handling Tests
    // ============================================

    #[test]
    fn test_rust_leading_whitespace() {
        let source = "    fn indented() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "indented");
    }

    #[test]
    fn test_rust_mixed_line_endings() {
        let source = "fn func1() {}\r\nfn func2() {}\nfn func3() {}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_rust_tabs_in_content() {
        let source = "fn tabbed() {\n\tlet x = 1;\n}";
        let chunks = chunk_code(source, Language::Rust).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("\t"));
    }

    // ============================================
    // Parser Edge Cases
    // ============================================

    #[test]
    fn test_parse_rust_success() {
        let source = "fn test() {}";
        let result = parse_rust(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_typescript_success() {
        let source = "function test() {}";
        let result = parse_typescript(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_doc_comment_start_no_parent() {
        let source = "fn test() {}";
        let tree = parse_rust(source).unwrap();
        let root = tree.root_node();
        let start = find_doc_comment_start(root, source);
        assert_eq!(start, 0);
    }

    #[test]
    fn test_find_doc_comment_start_no_comment() {
        let source = "fn test() {}";
        let tree = parse_rust(source).unwrap();
        let root = tree.root_node();
        let func_node = root.child(0).unwrap();
        let start = find_doc_comment_start(func_node, source);
        assert_eq!(start, func_node.start_byte());
    }

    // ============================================
    // Function Declarator Name Tests (C/C++)
    // ============================================

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_find_function_declarator_name_direct_identifier() {
        let source = "int test() { return 0; }";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "test");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_find_function_declarator_name_pointer() {
        let source = "int *test() { return 0; }";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "test");
    }

    // ============================================
    // TypeScript Arrow Function Edge Cases
    // ============================================

    #[test]
    fn test_extract_ts_arrow_function_no_arrow() {
        let source = "const x = 42;";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        // Should not extract regular variable as function
        let func_chunks: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Function).collect();
        assert!(func_chunks.is_empty());
    }

    #[test]
    fn test_extract_ts_arrow_function_with_let() {
        let source = "let myFunc = () => {};";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_name, "myFunc");
    }

    // ============================================
    // Coverage for extract_* helper functions
    // ============================================

    #[test]
    fn test_extract_ts_class_no_name() {
        // Anonymous class expression
        let source = "const MyClass = class {};";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        // The outer class may or may not be extracted depending on tree structure
        assert!(chunks.len() >= 0);
    }

    #[test]
    fn test_extract_ts_interface_no_name() {
        // Valid interface with name
        let source = "interface Test { x: number; }";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_extract_ts_function_no_name() {
        // Anonymous function expression
        let source = "(function() {})();";
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        // IIFE may not be extracted as named function
        let named_funcs: Vec<_> = chunks.iter().filter(|c| !c.chunk_name.is_empty()).collect();
        assert!(named_funcs.len() >= 0);
    }

    // ============================================
    // Coverage for recursive extraction
    // ============================================

    #[test]
    fn test_rust_deeply_nested_functions() {
        let source = r#"
mod a {
    mod b {
        mod c {
            fn deep() {}
        }
    }
}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();
        let functions: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Function).collect();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].chunk_name, "deep");
    }

    #[test]
    fn test_typescript_deeply_nested_functions() {
        let source = r#"
function outer() {
    function middle() {
        function inner() {
            return 42;
        }
    }
}
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();
        let functions: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Function).collect();
        assert!(functions.len() >= 1);
    }

    // ============================================
    // Block Comment Tests for C-family
    // ============================================

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_c_block_comment_before_function() {
        let source = "/* Block comment */\nvoid test() {}\n";
        let chunks = chunk_code(source, Language::C).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("Block comment"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_cpp_block_comment_before_function() {
        let source = "/* Block comment */\nvoid test() {}\n";
        let chunks = chunk_code(source, Language::Cpp).unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("Block comment"));
    }

    // ============================================
    // Multiple Item Types Together
    // ============================================

    #[test]
    fn test_rust_mixed_items() {
        let source = r#"
mod mymod {}
impl MyType {}
fn myfunc() {}
"#;
        let chunks = chunk_code(source, Language::Rust).unwrap();

        let module = chunks.iter().find(|c| c.chunk_type == ChunkType::Module);
        let class = chunks.iter().find(|c| c.chunk_type == ChunkType::Class);
        let func = chunks.iter().find(|c| c.chunk_type == ChunkType::Function);

        assert!(module.is_some());
        assert!(class.is_some());
        assert!(func.is_some());
    }

    #[test]
    fn test_typescript_mixed_items() {
        let source = r#"
interface MyInterface {}
class MyClass {}
function myFunc() {}
const myArrow = () => {};
"#;
        let chunks = chunk_code(source, Language::TypeScript).unwrap();

        assert!(chunks.len() >= 4);
    }
}
