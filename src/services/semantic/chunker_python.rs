/// Extract chunks from Python code
fn chunk_python_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    debug_assert!(!source.is_empty(), "source must not be empty");
    let tree = parse_python(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_python_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse Python source code
#[cfg(feature = "python-ast")]
fn parse_python(source: &str) -> Result<Tree, String> {
    debug_assert!(!source.is_empty(), "source must not be empty");
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
    debug_assert!(!_source.is_empty(), "_source must not be empty");
    Err("python-ast feature is disabled".to_string())
}

/// Extract items from Python AST
fn extract_python_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    debug_assert!(!source.is_empty(), "source must not be empty");
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
        // Don't recurse into function body - nested items are implementation details
        return;
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_python_items(child, source, chunks);
    }
}
