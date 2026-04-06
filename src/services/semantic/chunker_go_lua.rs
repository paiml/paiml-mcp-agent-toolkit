/// Extract chunks from Go code
fn chunk_go_file(source: &str) -> Result<Vec<CodeChunk>, String> {
    debug_assert!(!source.is_empty(), "source must not be empty");
    let tree = parse_go(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_go_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse Go source code
#[cfg(feature = "go-ast")]
fn parse_go(source: &str) -> Result<Tree, String> {
    debug_assert!(!source.is_empty(), "source must not be empty");
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
    debug_assert!(!_source.is_empty(), "_source must not be empty");
    Err("go-ast feature is disabled".to_string())
}

/// Extract a Go type name from the first type_spec child of a type_declaration
fn extract_go_type_name(node: Node, source: &str) -> Option<String> {
    debug_assert!(!source.is_empty(), "source must not be empty");
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
    debug_assert!(!source.is_empty(), "source must not be empty");
    match node.kind() {
        "function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = source[name_node.byte_range()].to_string();
                let start_byte = find_doc_comment_start(node, source);
                let content = source
                    .get(start_byte..node.end_byte())
                    .unwrap_or_default()
                    .to_string();
                push_chunk(chunks, ChunkType::Function, name, "go", node, content);
            }
            return; // Don't recurse into function body
        }
        "method_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = source[name_node.byte_range()].to_string();
                let content = source[node.byte_range()].to_string();
                push_chunk(chunks, ChunkType::Function, name, "go", node, content);
            }
            return; // Don't recurse into method body
        }
        "type_declaration" => {
            if let Some(name) = extract_go_type_name(node, source) {
                let start_byte = find_doc_comment_start(node, source);
                let content = source
                    .get(start_byte..node.end_byte())
                    .unwrap_or_default()
                    .to_string();
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
    debug_assert!(!source.is_empty(), "source must not be empty");
    let tree = parse_lua(source)?;
    let root = tree.root_node();
    let mut chunks = Vec::new();

    extract_lua_items(root, source, &mut chunks);

    Ok(chunks)
}

/// Parse Lua source code
#[cfg(feature = "lua-ast")]
fn parse_lua(source: &str) -> Result<Tree, String> {
    debug_assert!(!source.is_empty(), "source must not be empty");
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
    debug_assert!(!_source.is_empty(), "_source must not be empty");
    Err("lua-ast feature is disabled".to_string())
}

/// Extract items from Lua AST
fn extract_lua_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    debug_assert!(!source.is_empty(), "source must not be empty");
    // function_declaration: `function foo()` or `function M.foo()` or `function M:foo()`
    if node.kind() == "function_declaration" {
        if let Some(name) = extract_lua_function_name(&node, source) {
            let start_byte = find_doc_comment_start(node, source);
            let content = source
                .get(start_byte..node.end_byte())
                .unwrap_or_default()
                .to_string();

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
        // Don't recurse into function body - nested items are implementation details
        return;
    }
    // local_function_declaration: `local function foo()`
    else if node.kind() == "local_function_declaration" {
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
                language: "lua".to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                content: content.clone(),
                content_checksum: compute_checksum(&content),
            });
        }
        // Don't recurse into function body
        return;
    }
    // variable_declaration with function_definition: `local f = function()`
    else if node.kind() == "variable_declaration" {
        extract_lua_variable_function(node, source, chunks);
        // Don't recurse into variable declaration body
        return;
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
    debug_assert!(!source.is_empty(), "source must not be empty");
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
    debug_assert!(!source.is_empty(), "source must not be empty");
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
    debug_assert!(!source.is_empty(), "source must not be empty");
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
            push_chunk(
                chunks,
                ChunkType::Function,
                name.clone(),
                "lua",
                node,
                content,
            );
            return; // Only extract one function per variable_declaration
        }
    }
}
