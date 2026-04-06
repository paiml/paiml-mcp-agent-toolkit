// C language strategy
#[cfg(feature = "c-ast")]
pub struct CAstStrategy;

#[cfg(feature = "c-ast")]
fn c_node_to_ast_item(
    node: &crate::models::unified_ast::UnifiedAstNode,
    content: &str,
    content_lines: &[&str],
) -> Option<crate::services::context::AstItem> {
    let name = CAstStrategy::extract_name_from_node(node, content);
    let line_number = CAstStrategy::byte_pos_to_line(node.source_range.start as usize, content_lines);
    match &node.kind {
        crate::models::unified_ast::AstKind::Function(_) => {
            Some(crate::services::context::AstItem::Function {
                name: name.unwrap_or_else(|| "anonymous_function".to_string()),
                visibility: "public".to_string(),
                is_async: false,
                line: line_number,
            })
        }
        crate::models::unified_ast::AstKind::Type(type_kind) => {
            c_type_to_ast_item(type_kind, name, line_number)
        }
        _ => None,
    }
}

#[cfg(feature = "c-ast")]
fn c_type_to_ast_item(
    type_kind: &crate::models::unified_ast::TypeKind,
    name: Option<String>,
    line: usize,
) -> Option<crate::services::context::AstItem> {
    match type_kind {
        crate::models::unified_ast::TypeKind::Struct => {
            Some(crate::services::context::AstItem::Struct {
                name: name.unwrap_or_else(|| "anonymous_struct".to_string()),
                visibility: "public".to_string(),
                fields_count: 0,
                derives: vec![],
                line,
            })
        }
        crate::models::unified_ast::TypeKind::Enum => {
            Some(crate::services::context::AstItem::Enum {
                name: name.unwrap_or_else(|| "anonymous_enum".to_string()),
                visibility: "public".to_string(),
                variants_count: 0,
                line,
            })
        }
        _ => None,
    }
}

#[cfg(feature = "c-ast")]
#[async_trait]
impl AstStrategy for CAstStrategy {
    async fn analyze(&self, path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        use crate::services::ast_c::CAstParser;
        use tokio::fs;

        let content = fs::read_to_string(path).await?;
        let mut parser = CAstParser::new();
        match parser.parse_file(path, &content) {
            Ok(ast_dag) => {
                let content_lines: Vec<&str> = content.lines().collect();
                let items = ast_dag.nodes.iter()
                    .filter_map(|node| c_node_to_ast_item(node, &content, &content_lines))
                    .collect();
                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "c".to_string(),
                    items,
                    complexity_metrics: None,
                })
            }
            Err(e) => {
                tracing::warn!("Failed to parse C file {}: {}", path.display(), e);
                Ok(FileContext {
                    path: path.to_string_lossy().to_string(),
                    language: "c".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            }
        }
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "c" | "h")
    }
}

#[cfg(feature = "c-ast")]
impl CAstStrategy {
    /// Extract name from `UnifiedAstNode` by analyzing the source range
    fn extract_name_from_node(
        node: &crate::models::unified_ast::UnifiedAstNode,
        content: &str,
    ) -> Option<String> {
        // For now, extract a reasonable segment from the source range
        let start = node.source_range.start as usize;
        let end = node.source_range.end as usize;

        if start >= content.len() || end > content.len() || start >= end {
            return None;
        }

        let source_text = &content[start..end];

        // Use simple heuristics to extract identifiers from the source text
        match &node.kind {
            crate::models::unified_ast::AstKind::Function(_) => {
                Self::extract_function_name(source_text)
            }
            crate::models::unified_ast::AstKind::Type(_) => Self::extract_type_name(source_text),
            _ => None,
        }
    }

    /// Extract function name from source text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn extract_function_name(source_text: &str) -> Option<String> {
        // Look for pattern: type name(...) or name(...)
        if let Some(paren_pos) = source_text.find('(') {
            let before_paren = &source_text[..paren_pos];
            // Split by whitespace and take the last word (function name)
            before_paren
                .split_whitespace()
                .last()
                .map(|s| s.trim_start_matches('*').to_string())
        } else {
            None
        }
    }

    /// Extract type name from source text (struct, enum, etc.)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn extract_type_name(source_text: &str) -> Option<String> {
        // Look for patterns like "struct name" or "enum name"
        let words: Vec<&str> = source_text.split_whitespace().collect();
        if words.len() >= 2 {
            // Usually the name follows the keyword (struct/enum/typedef)
            for i in 0..words.len() - 1 {
                if matches!(words[i], "struct" | "enum" | "union" | "typedef") {
                    return Some(words[i + 1].trim_end_matches('{').to_string());
                }
            }
        }
        None
    }

    /// Convert byte position to line number
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn byte_pos_to_line(byte_pos: usize, content_lines: &[&str]) -> usize {
        let mut current_pos = 0;
        for (line_idx, line) in content_lines.iter().enumerate() {
            if current_pos + line.len() >= byte_pos {
                return line_idx + 1; // 1-based line numbers
            }
            current_pos += line.len() + 1; // +1 for newline character
        }
        content_lines.len() // Return last line if position is beyond content
    }
}
