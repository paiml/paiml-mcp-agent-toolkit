#![cfg_attr(coverage_nightly, coverage(off))]

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
    Lua,
}

/// Type of code chunk
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChunkType {
    Function,
    Class,
    Module,
    File,
    /// Rust struct definition
    Struct,
    /// Rust enum definition
    Enum,
    /// Rust trait definition
    Trait,
    /// Rust type alias
    TypeAlias,
    /// Rust impl block
    Impl,
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
        Language::Lua => chunk_lua_file(source),
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

/// Map Rust AST node kind to chunk type and name field
fn rust_node_to_chunk(kind: &str) -> Option<(ChunkType, &'static str, bool)> {
    // Returns (chunk_type, name_field, include_doc_comments)
    match kind {
        "function_item" => Some((ChunkType::Function, "name", true)),
        "impl_item" => Some((ChunkType::Class, "type", false)),
        "mod_item" => Some((ChunkType::Module, "name", false)),
        "struct_item" => Some((ChunkType::Struct, "name", true)),
        "enum_item" => Some((ChunkType::Enum, "name", true)),
        "trait_item" => Some((ChunkType::Trait, "name", true)),
        "type_item" => Some((ChunkType::TypeAlias, "name", true)),
        _ => None,
    }
}

/// Extract items (functions, impl blocks, modules) from Rust AST
fn extract_rust_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    if let Some((chunk_type, name_field, include_docs)) = rust_node_to_chunk(node.kind()) {
        if let Some(name_node) = node.child_by_field_name(name_field) {
            let name = source[name_node.byte_range()].to_string();
            let start_byte = if include_docs {
                find_doc_comment_start(node, source)
            } else {
                node.start_byte()
            };
            let content = source[start_byte..node.end_byte()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type,
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

/// Check if a variable_declarator contains an arrow function and extract it
fn try_extract_arrow_function(decl: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    let name_node = match decl.child_by_field_name("name") {
        Some(n) => n,
        None => return,
    };
    let value_node = match decl.child_by_field_name("value") {
        Some(n) => n,
        None => return,
    };
    if value_node.kind() != "arrow_function" {
        return;
    }
    let name = source[name_node.byte_range()].to_string();
    let content = source[decl.byte_range()].to_string();
    push_chunk(chunks, ChunkType::Function, name, "typescript", decl, content);
}

/// Extract TypeScript arrow function from variable declaration
fn extract_ts_arrow_function(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "variable_declarator" {
            try_extract_arrow_function(child, source, chunks);
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

/// Push a code chunk with standard fields
fn push_chunk(
    chunks: &mut Vec<CodeChunk>,
    chunk_type: ChunkType,
    name: String,
    language: &str,
    node: Node,
    content: String,
) {
    chunks.push(CodeChunk {
        file_path: String::new(),
        chunk_type,
        chunk_name: name,
        language: language.to_string(),
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        content: content.clone(),
        content_checksum: compute_checksum(&content),
    });
}

/// Extract C++ function name from a declarator node
fn extract_cpp_function_name<'a>(node: Node<'a>, source: &str) -> Option<String> {
    let declarator = node.child_by_field_name("declarator")?;
    let name_node = find_function_declarator_name(declarator, source)?;
    Some(source[name_node.byte_range()].to_string())
}

/// Extract function definitions from a C++ template declaration
fn extract_cpp_template_functions(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "function_definition" {
            continue;
        }
        if let Some(name) = extract_cpp_function_name(child, source) {
            let start_byte = find_doc_comment_start(node, source);
            let content = source[start_byte..node.end_byte()].to_string();
            push_chunk(chunks, ChunkType::Function, name, "cpp", node, content);
        }
    }
}

/// Extract items from C++ AST
fn extract_cpp_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    match node.kind() {
        "class_specifier" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = source[name_node.byte_range()].to_string();
                let content = source[node.byte_range()].to_string();
                push_chunk(chunks, ChunkType::Class, name, "cpp", node, content);
            }
        }
        "function_definition" => {
            if let Some(name) = extract_cpp_function_name(node, source) {
                let start_byte = find_doc_comment_start(node, source);
                let content = source[start_byte..node.end_byte()].to_string();
                push_chunk(chunks, ChunkType::Function, name, "cpp", node, content);
            }
        }
        "template_declaration" => {
            extract_cpp_template_functions(node, source, chunks);
            return; // Don't recurse - template extracted as whole
        }
        _ => {}
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

/// Extract a Go type name from the first type_spec child of a type_declaration
fn extract_go_type_name(node: Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "type_spec" {
            if let Some(name_node) = child.child_by_field_name("name") {
                return Some(source[name_node.byte_range()].to_string());
            }
        }
    }
    None
}

/// Extract items from Go AST
fn extract_go_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    match node.kind() {
        "function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = source[name_node.byte_range()].to_string();
                let start_byte = find_doc_comment_start(node, source);
                let content = source[start_byte..node.end_byte()].to_string();
                push_chunk(chunks, ChunkType::Function, name, "go", node, content);
            }
        }
        "method_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = source[name_node.byte_range()].to_string();
                let content = source[node.byte_range()].to_string();
                push_chunk(chunks, ChunkType::Function, name, "go", node, content);
            }
        }
        "type_declaration" => {
            if let Some(name) = extract_go_type_name(node, source) {
                let start_byte = find_doc_comment_start(node, source);
                let content = source[start_byte..node.end_byte()].to_string();
                push_chunk(chunks, ChunkType::Class, name, "go", node, content);
            }
            return; // Don't recurse into type declaration children
        }
        _ => {}
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_go_items(child, source, chunks);
    }
}

/// Extract chunks from Lua code
fn chunk_lua_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    let tree = parse_lua(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_lua_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse Lua source code
#[cfg(feature = "lua-ast")]
fn parse_lua(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_lua::LANGUAGE.into())
        .map_err(|e| format!("Failed to set Lua language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse Lua source".to_string())
}

#[cfg(not(feature = "lua-ast"))]
fn parse_lua(_source: &str) -> Result<Tree, String> {
    Err("lua-ast feature is disabled".to_string())
}

/// Extract items from Lua AST
fn extract_lua_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    // function_declaration: `function foo()` or `function M.foo()` or `function M:foo()`
    if node.kind() == "function_declaration" {
        if let Some(name) = extract_lua_function_name(&node, source) {
            let start_byte = find_doc_comment_start(node, source);
            let content = source[start_byte..node.end_byte()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Function,
                chunk_name: name,
                language: "lua".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }
    // local_function_declaration: `local function foo()`
    else if node.kind() == "local_function_declaration" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();
            let start_byte = find_doc_comment_start(node, source);
            let content = source[start_byte..node.end_byte()].to_string();

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type: ChunkType::Function,
                chunk_name: name,
                language: "lua".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
    }
    // variable_declaration with function_definition: `local f = function()`
    else if node.kind() == "variable_declaration" {
        extract_lua_variable_function(node, source, chunks);
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_lua_items(child, source, chunks);
    }
}

/// Extract function name from Lua function_declaration node
/// Handles: `function foo()`, `function M.foo()`, `function M:foo()`
fn extract_lua_function_name(node: &Node, source: &str) -> Option<String> {
    // Try named field "name" first
    if let Some(name_node) = node.child_by_field_name("name") {
        return Some(source[name_node.byte_range()].to_string());
    }
    // Fallback: walk children looking for identifier or dot/method index
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "dot_index_expression" | "method_index_expression" => {
                return Some(source[child.byte_range()].to_string());
            }
            _ => {}
        }
    }
    None
}

/// Find the first identifier child in a Lua variable_list or assignment node
fn find_lua_var_name(node: Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return Some(source[child.byte_range()].to_string());
        }
    }
    None
}

/// Check if a node or its children contain a function_definition
fn has_function_definition(node: Node) -> bool {
    if node.kind() == "function_definition" {
        return true;
    }
    let mut cursor = node.walk();
    let found = node
        .children(&mut cursor)
        .any(|c| c.kind() == "function_definition");
    found
}

/// Extract function from variable assignment: `local f = function() ... end`
fn extract_lua_variable_function(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    let mut cursor = node.walk();
    let mut var_name = None;
    for child in node.children(&mut cursor) {
        if child.kind() == "assignment_statement" || child.kind() == "variable_list" {
            var_name = var_name.or_else(|| find_lua_var_name(child, source));
        }
        if !has_function_definition(child) {
            continue;
        }
        if let Some(name) = &var_name {
            let content = source[node.byte_range()].to_string();
            push_chunk(chunks, ChunkType::Function, name.clone(), "lua", node, content);
            return; // Only extract one function per variable_declaration
        }
    }
}

// TRUENO-RAG-3-CHUNKER: Text Chunking with Overlap
// Integrates trueno-rag RecursiveChunker for RAG pipelines

/// Chunk text with fixed-size chunks and overlap for RAG retrieval
///
/// This function uses trueno-rag's chunking approach with overlap to ensure
/// that context is preserved across chunk boundaries, improving retrieval quality.
///
/// # Arguments
/// * `text` - The text to chunk
/// * `chunk_size` - Target chunk size in characters
/// * `overlap` - Number of characters to overlap between chunks
///
/// # Returns
/// Vector of text chunks with overlap applied
pub fn chunk_text_with_overlap(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    // Use trueno-rag's RecursiveChunker internally
    use trueno_rag::chunk::{Chunker, RecursiveChunker};
    use trueno_rag::Document;

    let chunker = RecursiveChunker::new(chunk_size, overlap);
    let doc = Document::new(text);

    match chunker.chunk(&doc) {
        Ok(chunks) => chunks
            .into_iter()
            .map(|c: trueno_rag::Chunk| c.content)
            .collect(),
        Err(_) => {
            // Fallback to simple fixed-size chunking
            chunk_text_fixed(text, chunk_size, overlap)
        }
    }
}

/// Chunk text using recursive separators (paragraph, sentence, word boundaries)
///
/// This function prefers semantic boundaries over arbitrary character splits,
/// producing more coherent chunks for embedding and retrieval.
///
/// # Arguments
/// * `text` - The text to chunk
/// * `chunk_size` - Target chunk size in characters
/// * `overlap` - Number of characters to overlap between chunks
///
/// # Returns
/// Vector of text chunks respecting semantic boundaries
pub fn chunk_text_recursive(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    // Use trueno-rag's RecursiveChunker with custom separators
    use trueno_rag::chunk::{Chunker, RecursiveChunker};
    use trueno_rag::Document;

    let chunker = RecursiveChunker::new(chunk_size, overlap).with_separators(vec![
        "\n\n".to_string(), // Paragraph boundary
        "\n".to_string(),   // Line boundary
        ". ".to_string(),   // Sentence boundary
        ", ".to_string(),   // Clause boundary
        " ".to_string(),    // Word boundary
    ]);

    let doc = Document::new(text);

    match chunker.chunk(&doc) {
        Ok(chunks) => chunks
            .into_iter()
            .map(|c: trueno_rag::Chunk| c.content)
            .collect(),
        Err(_) => {
            // Fallback to overlap chunking
            chunk_text_with_overlap(text, chunk_size, overlap)
        }
    }
}

/// Simple fixed-size text chunking with overlap (fallback implementation)
///
/// # Arguments
/// * `text` - The text to chunk
/// * `chunk_size` - Target chunk size in characters
/// * `overlap` - Number of characters to overlap between chunks
///
/// # Returns
/// Vector of text chunks with overlap applied
pub fn chunk_text_fixed(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);

        if end >= chars.len() {
            break;
        }

        // Move start, accounting for overlap
        let step = chunk_size.saturating_sub(overlap);
        start += if step == 0 { 1 } else { step };
    }

    chunks
}

/// Compute SHA256 checksum of content
fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Tests extracted to chunker_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "chunker_tests.rs"]
mod tests;
