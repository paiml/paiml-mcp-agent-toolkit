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
        let content = source
            .get(start_byte..node.end_byte())
            .unwrap_or_default()
            .to_string();

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
    push_chunk(
        chunks,
        ChunkType::Function,
        name,
        "typescript",
        decl,
        content,
    );
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

