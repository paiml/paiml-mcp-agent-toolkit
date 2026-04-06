#[cfg(feature = "python-ast")]
fn build_qualified_name(class_stack: &[String], name: &str) -> String {
    if class_stack.is_empty() {
        name.to_string()
    } else {
        format!("{}::{}", class_stack.join("::"), name)
    }
}

#[cfg(feature = "python-ast")]
fn visit_function_def(
    node: &tree_sitter::Node,
    source: &str,
    items: &mut Vec<AstItem>,
    class_stack: &[String],
) {
    if let Some(name_node) = node.child_by_field_name("name") {
        let name = &source[name_node.byte_range()];
        items.push(AstItem::Function {
            name: build_qualified_name(class_stack, name),
            visibility: "public".to_string(),
            is_async: false,
            line: node.start_position().row + 1,
        });
    }
}

#[cfg(feature = "python-ast")]
fn visit_class_def(
    node: &tree_sitter::Node,
    source: &str,
    items: &mut Vec<AstItem>,
    class_stack: &mut Vec<String>,
) -> bool {
    let name_node = match node.child_by_field_name("name") {
        Some(n) => n,
        None => return false,
    };
    let name = &source[name_node.byte_range()];
    items.push(AstItem::Struct {
        name: build_qualified_name(class_stack, name),
        visibility: "public".to_string(),
        fields_count: 0,
        derives: Vec::new(),
        line: node.start_position().row + 1,
    });
    class_stack.push(name.to_string());
    true
}

impl UnifiedPythonAnalyzer {
    #[cfg(feature = "python-ast")]
    fn extract_ast_items(&self, tree: &Tree, source: &str) -> Vec<AstItem> {
        let mut items = Vec::new();
        let root = tree.root_node();
        self.visit_node_for_items(&root, source, &mut items, &mut Vec::new());
        items
    }

    #[cfg(feature = "python-ast")]
    fn visit_node_for_items(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        items: &mut Vec<AstItem>,
        class_stack: &mut Vec<String>,
    ) {
        match node.kind() {
            "function_definition" => {
                visit_function_def(node, source, items, class_stack);
                for child in node.children(&mut node.walk()) {
                    self.visit_node_for_items(&child, source, items, class_stack);
                }
            }
            "class_definition" => {
                let pushed = visit_class_def(node, source, items, class_stack);
                for child in node.children(&mut node.walk()) {
                    self.visit_node_for_items(&child, source, items, class_stack);
                }
                if pushed {
                    class_stack.pop();
                }
            }
            _ => {
                for child in node.children(&mut node.walk()) {
                    self.visit_node_for_items(&child, source, items, class_stack);
                }
            }
        }
    }
}
