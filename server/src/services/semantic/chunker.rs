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
                    || kind == "block_comment";  // Rust, C, C++

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

/// Extract items from TypeScript AST
fn extract_typescript_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // Check for class declaration
    if node.kind() == "class_declaration" {
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
    // Check for interface declaration
    else if node.kind() == "interface_declaration" {
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
    // Check for function declaration
    else if node.kind() == "function_declaration" || node.kind() == "function" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();

            // Include preceding doc comments
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
    // Check for arrow function with variable declaration
    else if node.kind() == "lexical_declaration" || node.kind() == "variable_declaration" {
        // Look for arrow function patterns: const foo = () => {}
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "variable_declarator" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if let Some(value_node) = child.child_by_field_name("value") {
                            if value_node.kind() == "arrow_function" {
                                let name = source[name_node.byte_range()].to_string();
                                // Include the whole declaration
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
fn parse_python(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|e| format!("Failed to set Python language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse Python source".to_string())
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
fn parse_c(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .map_err(|e| format!("Failed to set C language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse C source".to_string())
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
        return find_function_declarator_name(
            node.child_by_field_name("declarator")?,
            _source,
        );
    }
    if node.kind() == "pointer_declarator" {
        return find_function_declarator_name(
            node.child_by_field_name("declarator")?,
            _source,
        );
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
fn parse_cpp(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_cpp::LANGUAGE.into())
        .map_err(|e| format!("Failed to set C++ language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse C++ source".to_string())
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
fn parse_go(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .map_err(|e| format!("Failed to set Go language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse Go source".to_string())
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
