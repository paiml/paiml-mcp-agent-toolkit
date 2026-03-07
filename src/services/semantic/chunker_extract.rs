// --- File extract types and functions for `pmat extract --list` ---

/// Rich file-level extraction result with imports, test boundaries, and per-item visibility.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileExtract {
    pub file: String,
    pub language: String,
    pub imports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_test_line: Option<usize>,
    pub items: Vec<ExtractedItem>,
}

/// A single extracted item with visibility metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractedItem {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub start_line: usize,
    pub end_line: usize,
    pub lines: usize,
    pub visibility: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ExtractedItem>,
}

/// Extract rich file details: imports, test boundaries, and items with visibility.
///
/// Single-parse approach: parses once, then collects imports, cfg_test boundary,
/// chunks, and visibility from the same AST.
pub fn extract_file_details(
    path: &str,
    source: &str,
    language: Language,
) -> Result<FileExtract, String> {
    if source.trim().is_empty() {
        return Ok(FileExtract {
            file: path.to_string(),
            language: language.as_str().to_string(),
            imports: Vec::new(),
            cfg_test_line: None,
            items: Vec::new(),
        });
    }

    let tree = parse_for_language(source, language)?;
    let root = tree.root_node();

    let imports = collect_imports(root, source, language);
    let cfg_test_line = if language == Language::Rust {
        find_cfg_test_line(root, source)
    } else {
        None
    };

    // Reuse existing extract functions (same module, no double parse)
    let mut chunks = Vec::new();
    match language {
        #[cfg(feature = "rust-ast")]
        Language::Rust => extract_rust_items(root, source, &mut chunks),
        #[cfg(feature = "typescript-ast")]
        Language::TypeScript => extract_typescript_items(root, source, &mut chunks),
        #[cfg(feature = "python-ast")]
        Language::Python => extract_python_items(root, source, &mut chunks),
        #[cfg(feature = "c-ast")]
        Language::C => extract_c_items(root, source, &mut chunks),
        #[cfg(feature = "c-ast")]
        Language::Cpp => extract_cpp_items(root, source, &mut chunks),
        Language::Go => extract_go_items(root, source, &mut chunks),
        Language::Lua => extract_lua_items(root, source, &mut chunks),
        #[allow(unreachable_patterns)]
        _ => {}
    }

    let visibility_map = collect_visibility(root, source, language);

    let mut flat_items: Vec<ExtractedItem> = chunks
        .into_iter()
        .filter(|c| c.chunk_type != ChunkType::File)
        .map(|c| {
            let vis = visibility_map
                .get(&c.start_line)
                .cloned()
                .unwrap_or_default();
            // Detect #[cfg(test)] modules: module whose start_line is 1 line after cfg_test_line
            let item_type = if c.chunk_type == ChunkType::Module {
                if let Some(ct_line) = cfg_test_line {
                    if c.start_line == ct_line + 1 || c.start_line == ct_line {
                        ChunkType::TestModule.as_str().to_string()
                    } else {
                        c.chunk_type.as_str().to_string()
                    }
                } else {
                    c.chunk_type.as_str().to_string()
                }
            } else {
                c.chunk_type.as_str().to_string()
            };
            ExtractedItem {
                name: c.chunk_name,
                item_type,
                start_line: c.start_line,
                end_line: c.end_line,
                lines: c.end_line.saturating_sub(c.start_line) + 1,
                visibility: vis,
                children: Vec::new(),
            }
        })
        .collect();

    flat_items.sort_by_key(|i| i.start_line);

    // Post-process: nest items inside their parent modules/test_modules
    let items = nest_children_into_modules(flat_items);

    Ok(FileExtract {
        file: path.to_string(),
        language: language.as_str().to_string(),
        imports,
        cfg_test_line,
        items,
    })
}

/// Nest items that fall within a module's line range into that module's `children`.
/// Items not inside any module remain at the top level.
fn nest_children_into_modules(flat_items: Vec<ExtractedItem>) -> Vec<ExtractedItem> {
    // Identify module indices and their line ranges
    let module_ranges: Vec<(usize, usize, usize)> = flat_items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.item_type == "module" || item.item_type == "test_module")
        .map(|(idx, item)| (idx, item.start_line, item.end_line))
        .collect();

    if module_ranges.is_empty() {
        return flat_items;
    }

    // Determine which items belong to which module
    let mut parent_of: Vec<Option<usize>> = vec![None; flat_items.len()];
    for (i, item) in flat_items.iter().enumerate() {
        for &(mod_idx, mod_start, mod_end) in &module_ranges {
            if i != mod_idx
                && item.start_line > mod_start
                && item.end_line <= mod_end
            {
                parent_of[i] = Some(mod_idx);
                break; // innermost module wins since ranges are sorted
            }
        }
    }

    // Build result: modules with children, skip items that became children
    let mut result = Vec::new();
    let mut children_by_parent: Vec<Vec<ExtractedItem>> = vec![Vec::new(); flat_items.len()];

    for (i, item) in flat_items.into_iter().enumerate() {
        if let Some(parent_idx) = parent_of[i] {
            children_by_parent[parent_idx].push(item);
        } else {
            result.push((i, item));
        }
    }

    // Attach children to their parent modules
    result
        .into_iter()
        .map(|(orig_idx, mut item)| {
            let children = std::mem::take(&mut children_by_parent[orig_idx]);
            if !children.is_empty() {
                item.children = children;
            }
            item
        })
        .collect()
}

/// Route to the correct language parser.
fn parse_for_language(source: &str, language: Language) -> Result<Tree, String> {
    match language {
        #[cfg(feature = "rust-ast")]
        Language::Rust => parse_rust(source),
        #[cfg(feature = "typescript-ast")]
        Language::TypeScript => parse_typescript(source),
        #[cfg(feature = "python-ast")]
        Language::Python => parse_python(source),
        #[cfg(feature = "c-ast")]
        Language::C => parse_c(source),
        #[cfg(feature = "c-ast")]
        Language::Cpp => parse_cpp(source),
        Language::Go => parse_go(source),
        Language::Lua => parse_lua(source),
        #[allow(unreachable_patterns)]
        _ => Err(format!("language {:?} not enabled", language.as_str())),
    }
}

/// Collect top-level import statements from the AST root.
fn collect_imports(root: Node, source: &str, language: Language) -> Vec<String> {
    let import_kinds: &[&str] = match language {
        Language::Rust => &["use_declaration", "extern_crate_declaration"],
        Language::Python => &["import_statement", "import_from_statement"],
        Language::TypeScript => &["import_statement"],
        Language::Go => &["import_declaration"],
        Language::C | Language::Cpp => &["preproc_include"],
        Language::Lua | Language::Ptx => return Vec::new(),
    };

    let mut imports = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if import_kinds.contains(&child.kind()) {
            let text = source[child.byte_range()].trim().to_string();
            if !text.is_empty() {
                imports.push(text);
            }
        }
    }
    imports
}

/// Find the line where `#[cfg(test)]` appears (Rust only).
/// Returns the 1-indexed line of the attribute, which marks the start of test code.
fn find_cfg_test_line(root: Node, source: &str) -> Option<usize> {
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "attribute_item" {
            let text = &source[child.byte_range()];
            if text.contains("cfg(test)") {
                return Some(child.start_position().row + 1);
            }
        }
    }
    None
}

/// Collect visibility per item start_line.
fn collect_visibility(root: Node, source: &str, language: Language) -> HashMap<usize, String> {
    let mut map = HashMap::new();
    match language {
        Language::Rust => collect_rust_visibility(root, source, &mut map),
        Language::Go => collect_go_visibility(root, source, &mut map),
        Language::TypeScript => collect_ts_visibility(root, source, &mut map, false),
        _ => {}
    }
    map
}

/// Find visibility_modifier child node (it's a child kind, not a named field).
fn find_visibility_modifier<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let result = node
        .children(&mut cursor)
        .find(|child| child.kind() == "visibility_modifier");
    result
}

/// Walk Rust AST mirroring extract_rust_items to collect visibility per start_line.
fn collect_rust_visibility(node: Node, source: &str, map: &mut HashMap<usize, String>) {
    let is_container = matches!(node.kind(), "impl_item" | "mod_item" | "trait_item");

    if rust_node_to_chunk(node.kind()).is_some() {
        let line = node.start_position().row + 1;
        let vis = if let Some(vis_node) = find_visibility_modifier(node) {
            source[vis_node.byte_range()].to_string()
        } else {
            String::new()
        };
        map.insert(line, vis);

        if !is_container {
            return;
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_rust_visibility(child, source, map);
    }
}

/// Walk Go AST to determine exported (uppercase) vs unexported visibility.
fn collect_go_visibility(node: Node, source: &str, map: &mut HashMap<usize, String>) {
    match node.kind() {
        "function_declaration" | "method_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &source[name_node.byte_range()];
                let line = node.start_position().row + 1;
                let vis = if name.starts_with(|c: char| c.is_uppercase()) {
                    "pub".to_string()
                } else {
                    String::new()
                };
                map.insert(line, vis);
            }
            return;
        }
        "type_declaration" => {
            if let Some(name) = extract_go_type_name(node, source) {
                let line = node.start_position().row + 1;
                let vis = if name.starts_with(|c: char| c.is_uppercase()) {
                    "pub".to_string()
                } else {
                    String::new()
                };
                map.insert(line, vis);
            }
            return;
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_go_visibility(child, source, map);
    }
}

/// Walk TypeScript AST to detect `export` visibility.
fn collect_ts_visibility(
    node: Node,
    source: &str,
    map: &mut HashMap<usize, String>,
    exported: bool,
) {
    let is_export = node.kind() == "export_statement";
    let currently_exported = exported || is_export;

    match node.kind() {
        "function_declaration" | "class_declaration" | "interface_declaration" => {
            if currently_exported {
                let line = node.start_position().row + 1;
                map.insert(line, "export".to_string());
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            // Arrow function chunks use variable_declarator's start_line
            let mut inner = node.walk();
            for child in node.children(&mut inner) {
                if child.kind() == "variable_declarator" && currently_exported {
                    let line = child.start_position().row + 1;
                    map.insert(line, "export".to_string());
                }
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_visibility(child, source, map, currently_exported);
    }
}
