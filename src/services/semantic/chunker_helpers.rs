/// Check whether a sibling node is a doc comment that should be included.
/// Returns `true` if the sibling is a doc comment (include and continue scanning),
/// `false` if scanning should stop (non-comment or non-doc comment).
fn is_doc_comment(kind: &str, source: &str, sibling: Node) -> bool {
    let is_comment = kind == "comment"      // TypeScript, C, C++, Go
        || kind == "line_comment"           // Rust
        || kind == "block_comment"; // Rust, C, C++

    if !is_comment {
        return false;
    }

    // For Rust line_comment: only include /// doc comments, not regular //
    if kind == "line_comment" {
        let comment_text = &source[sibling.byte_range()];
        return comment_text.trim_start().starts_with("///");
    }

    // All other comment types (block_comment, generic comment) are included
    true
}

/// Helper: Find preceding doc comments for a node (all languages)
/// Returns the start byte position of the first comment, or node start if none
fn find_doc_comment_start(node: Node, source: &str) -> usize {
    let mut start_byte = node.start_byte();

    let parent = match node.parent() {
        Some(p) => p,
        None => return start_byte,
    };

    let mut cursor = parent.walk();
    let siblings: Vec<Node> = parent.children(&mut cursor).collect();

    let node_index = match siblings.iter().position(|n| n.id() == node.id()) {
        Some(idx) => idx,
        None => return start_byte,
    };

    // Check previous siblings in reverse order
    for i in (0..node_index).rev() {
        let sibling = siblings[i];
        if !is_doc_comment(sibling.kind(), source, sibling) {
            break;
        }
        start_byte = sibling.start_byte();
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

/// Compute SHA256 checksum of content
fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
