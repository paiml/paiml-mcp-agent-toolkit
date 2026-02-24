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
                let content = source
                    .get(start_byte..node.end_byte())
                    .unwrap_or_default()
                    .to_string();

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
        // Don't recurse into function body - nested items are implementation details
        return;
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_c_items(child, source, chunks);
    }
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
            let content = source
                .get(start_byte..node.end_byte())
                .unwrap_or_default()
                .to_string();
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
            return; // Don't recurse - class extracted as whole
        }
        "function_definition" => {
            if let Some(name) = extract_cpp_function_name(node, source) {
                let start_byte = find_doc_comment_start(node, source);
                let content = source
                    .get(start_byte..node.end_byte())
                    .unwrap_or_default()
                    .to_string();
                push_chunk(chunks, ChunkType::Function, name, "cpp", node, content);
            }
            return; // Don't recurse into function body
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
