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

    // Check for function declaration (prototype without body, e.g., in .h headers)
    if node.kind() == "declaration" {
        if let Some(name) = extract_c_declaration_name(node, source) {
            let content = source
                .get(node.start_byte()..node.end_byte())
                .unwrap_or_default()
                .to_string();
            // Only index declarations that look like function prototypes (have parentheses)
            if content.contains('(') {
                chunks.push(CodeChunk {
                    file_path: String::new(),
                    chunk_type: ChunkType::Function,
                    chunk_name: format!("{name} [decl]"),
                    language: "c".to_string(),
                    start_line: node.start_position().row + 1,
                    end_line: node.end_position().row + 1,
                    content: content.clone(),
                    content_checksum: compute_checksum(&content),
                });
            }
        }
        return;
    }

    // Recursively process children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_c_items(child, source, chunks);
    }
}

/// Extract function name from a C declaration node (prototype)
fn extract_c_declaration_name(node: Node, source: &str) -> Option<String> {
    let declarator = node.child_by_field_name("declarator")?;
    // declaration -> declarator -> function_declarator -> declarator -> identifier
    find_function_declarator_name(declarator, source)
        .map(|n| source[n.byte_range()].to_string())
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

/// Extract items from C++ AST with namespace/class qualification
fn extract_cpp_items(node: Node, source: &str, chunks: &mut Vec<CodeChunk>) {
    extract_cpp_items_qualified(node, source, chunks, &[]);
}

/// Recursive C++ item extraction with scope tracking for qualified names
fn extract_cpp_items_qualified(
    node: Node,
    source: &str,
    chunks: &mut Vec<CodeChunk>,
    scope: &[String],
) {
    match node.kind() {
        "namespace_definition" => {
            extract_cpp_namespace(node, source, chunks, scope);
            return;
        }
        "class_specifier" | "struct_specifier" => {
            extract_cpp_class(node, source, chunks, scope);
            return;
        }
        "function_definition" => {
            extract_cpp_func_def(node, source, chunks, scope);
            return;
        }
        "template_declaration" => {
            extract_cpp_template(node, source, chunks, scope);
            return;
        }
        "declaration" => {
            // Function declaration/prototype (no body)
            if let Some(name) = extract_cpp_declaration_name(node, source, scope) {
                let content = source
                    .get(node.start_byte()..node.end_byte())
                    .unwrap_or_default()
                    .to_string();
                if content.contains('(') {
                    chunks.push(CodeChunk {
                        file_path: String::new(),
                        chunk_type: ChunkType::Function,
                        chunk_name: format!("{name} [decl]"),
                        language: "cpp".to_string(),
                        start_line: node.start_position().row + 1,
                        end_line: node.end_position().row + 1,
                        content: content.clone(),
                        content_checksum: compute_checksum(&content),
                    });
                }
            }
            return;
        }
        _ => {}
    }

    recurse_children(node, source, chunks, scope);
}

fn extract_cpp_namespace(node: Node, source: &str, chunks: &mut Vec<CodeChunk>, scope: &[String]) {
    let ns_name = node
        .child_by_field_name("name")
        .map(|n| source[n.byte_range()].to_string());
    let Some(body) = node.child_by_field_name("body") else { return };
    let mut new_scope = scope.to_vec();
    if let Some(name) = ns_name {
        new_scope.push(name);
    }
    recurse_children(body, source, chunks, &new_scope);
}

fn extract_cpp_class(node: Node, source: &str, chunks: &mut Vec<CodeChunk>, scope: &[String]) {
    let Some(name_node) = node.child_by_field_name("name") else { return };
    let bare_name = source[name_node.byte_range()].to_string();
    let qualified = qualify_name(scope, &bare_name);
    let content = source[node.byte_range()].to_string();
    push_chunk(chunks, ChunkType::Class, qualified, "cpp", node, content);

    if let Some(body) = node.child_by_field_name("body") {
        let mut class_scope = scope.to_vec();
        class_scope.push(bare_name);
        recurse_children(body, source, chunks, &class_scope);
    }
}

fn extract_cpp_func_def(node: Node, source: &str, chunks: &mut Vec<CodeChunk>, scope: &[String]) {
    let Some(bare_name) = extract_cpp_function_name(node, source) else { return };
    let qualified = qualify_name(scope, &bare_name);
    let start_byte = find_doc_comment_start(node, source);
    let content = source
        .get(start_byte..node.end_byte())
        .unwrap_or_default()
        .to_string();
    push_chunk(chunks, ChunkType::Function, qualified, "cpp", node, content);
}

fn extract_cpp_template(node: Node, source: &str, chunks: &mut Vec<CodeChunk>, scope: &[String]) {
    // Extract template parameter list for tracking (e.g., "<typename T, int N>")
    let template_params = extract_template_params(node, source);

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                // Use template_declaration node for content (includes template<...> prefix)
                if let Some(bare_name) = extract_cpp_function_name(child, source) {
                    let name_with_params = if template_params.is_empty() {
                        qualify_name(scope, &bare_name)
                    } else {
                        format!("{}<{}>", qualify_name(scope, &bare_name), template_params)
                    };
                    let start_byte = find_doc_comment_start(node, source);
                    let content = source
                        .get(start_byte..node.end_byte())
                        .unwrap_or_default()
                        .to_string();
                    push_chunk(chunks, ChunkType::Function, name_with_params, "cpp", node, content);
                }
            }
            "class_specifier" | "struct_specifier" => {
                extract_cpp_items_qualified(child, source, chunks, scope);
            }
            _ => {}
        }
    }
}

/// Extract template parameter names from a template_declaration node.
/// Returns e.g., "T, N" from "template<typename T, int N>".
fn extract_template_params(node: Node, source: &str) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "template_parameter_list" {
            let params_src = source
                .get(child.start_byte()..child.end_byte())
                .unwrap_or("");
            // Extract just the parameter names from "<typename T, int N, ...>"
            let inner = params_src
                .trim_start_matches('<')
                .trim_end_matches('>');
            // Extract the last identifier from each comma-separated param
            let params: Vec<&str> = inner
                .split(',')
                .filter_map(|p| p.split_whitespace().last())
                .filter(|p| !p.is_empty() && !p.starts_with('.'))
                .collect();
            return params.join(", ");
        }
    }
    String::new()
}

fn recurse_children(node: Node, source: &str, chunks: &mut Vec<CodeChunk>, scope: &[String]) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_cpp_items_qualified(child, source, chunks, scope);
    }
}

/// Build a qualified name from scope segments
fn qualify_name(scope: &[String], name: &str) -> String {
    if scope.is_empty() {
        name.to_string()
    } else {
        format!("{}::{}", scope.join("::"), name)
    }
}

/// Extract function name from a C++ declaration node (prototype) with scope qualification
fn extract_cpp_declaration_name(node: Node, source: &str, scope: &[String]) -> Option<String> {
    let declarator = node.child_by_field_name("declarator")?;
    let name_node = find_function_declarator_name(declarator, source)?;
    let bare_name = source[name_node.byte_range()].to_string();
    Some(qualify_name(scope, &bare_name))
}
