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

/// Extract items (functions, impl blocks, modules) from Rust AST
fn extract_rust_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    let is_container = matches!(node.kind(), "impl_item" | "mod_item" | "trait_item");

    if let Some((chunk_type, name_field, include_docs)) = rust_node_to_chunk(node.kind()) {
        if let Some(name_node) = node.child_by_field_name(name_field) {
            let name = source[name_node.byte_range()].to_string();
            let start_byte = if include_docs {
                find_doc_comment_start(node, source)
            } else {
                node.start_byte()
            };
            let content = source
                .get(start_byte..node.end_byte())
                .unwrap_or_default()
                .to_string();
            let checksum = compute_checksum(&content);

            chunks.push(CodeChunk {
                file_path: String::new(),
                chunk_type,
                chunk_name: name,
                language: "rust".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content,
                content_checksum: checksum,
            });
        }

        // Only recurse into containers (impl, mod, trait) — not into
        // function/struct/enum bodies where nested items are implementation details
        if !is_container {
            return;
        }
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_rust_items(child, source, chunks);
    }
}
