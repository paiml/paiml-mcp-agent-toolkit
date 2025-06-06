//! C language AST parser implementation

use crate::models::unified_ast::{
    AstDag, AstKind, ExprKind, FunctionKind, Language, MacroKind, NodeFlags, NodeKey, StmtKind,
    TypeKind, UnifiedAstNode, VarKind,
};
use anyhow::Result;
use std::path::Path;

#[cfg(feature = "c-ast")]
use tree_sitter_c;

#[cfg(feature = "c-ast")]
use tree_sitter::{Node, Parser, TreeCursor};

/// C language AST parser implementation
pub struct CAstParser {
    #[cfg(feature = "c-ast")]
    parser: Parser,
}

impl Default for CAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CAstParser {
    pub fn new() -> Self {
        #[cfg(feature = "c-ast")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(&tree_sitter_c::language())
                .expect("Error loading C grammar");
            Self { parser }
        }

        #[cfg(not(feature = "c-ast"))]
        {
            Self {}
        }
    }

    pub fn parse_file(&mut self, path: &Path, content: &str) -> Result<AstDag> {
        #[cfg(feature = "c-ast")]
        {
            let tree = self
                .parser
                .parse(content, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse C file: {}", path.display()))?;

            let mut dag = AstDag::new();
            let mut cursor = tree.walk();

            // Create root module node
            let mut root_node = UnifiedAstNode::new(
                AstKind::Module(crate::models::unified_ast::ModuleKind::File),
                Language::C,
            );
            root_node.source_range = 0..content.len() as u32;
            let root_key = dag.add_node(root_node);

            self.walk_tree(&mut cursor, content, &mut dag, root_key)?;

            Ok(dag)
        }

        #[cfg(not(feature = "c-ast"))]
        {
            let _ = (path, content);
            Err(anyhow::anyhow!(
                "C AST parsing requires the 'c-ast' feature"
            ))
        }
    }

    #[cfg(feature = "c-ast")]
    fn walk_tree(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        dag: &mut AstDag,
        parent_key: NodeKey,
    ) -> Result<()> {
        if !cursor.goto_first_child() {
            return Ok(());
        }

        loop {
            let node = cursor.node();
            if let Some(ast_node) = self.convert_node(&node, source)? {
                let node_key = dag.add_node(ast_node);

                // Link to parent
                if let Some(parent) = dag.nodes.get_mut(parent_key) {
                    if parent.first_child == 0 {
                        parent.first_child = node_key;
                    } else {
                        // Find last sibling and link
                        let mut sibling = parent.first_child;
                        while let Some(sibling_node) = dag.nodes.get(sibling) {
                            if sibling_node.next_sibling == 0 {
                                break;
                            }
                            sibling = sibling_node.next_sibling;
                        }
                        if let Some(sibling_node) = dag.nodes.get_mut(sibling) {
                            sibling_node.next_sibling = node_key;
                        }
                    }
                }

                if let Some(child_node) = dag.nodes.get_mut(node_key) {
                    child_node.parent = parent_key;
                }

                self.walk_tree(cursor, source, dag, node_key)?;
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
        Ok(())
    }

    #[cfg(feature = "c-ast")]
    fn convert_node(&self, node: &Node, source: &str) -> Result<Option<UnifiedAstNode>> {
        let kind = self.node_kind_to_ast_kind(node.kind());

        if let Some(ast_kind) = kind {
            let start_pos = node.start_position();
            let end_pos = node.end_position();

            let mut ast_node = UnifiedAstNode::new(ast_kind, Language::C);
            ast_node.source_range = start_pos.column as u32..end_pos.column as u32;

            // Extract name and set flags
            self.extract_node_info(node, source, &mut ast_node);

            // Calculate complexity for functions
            if matches!(ast_node.kind, AstKind::Function(_)) {
                let complexity = self.calculate_complexity(node);
                ast_node.set_complexity(complexity);
            }

            Ok(Some(ast_node))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "c-ast")]
    fn node_kind_to_ast_kind(&self, node_kind: &str) -> Option<AstKind> {
        match node_kind {
            // Functions and declarations
            "function_definition" => Some(AstKind::Function(FunctionKind::Regular)),
            "declaration" => Some(AstKind::Variable(VarKind::Let)),

            // Type definitions
            "struct_specifier" => Some(AstKind::Type(TypeKind::Struct)),
            "enum_specifier" => Some(AstKind::Type(TypeKind::Enum)),
            "union_specifier" => Some(AstKind::Type(TypeKind::Union)),
            "typedef_declaration" => Some(AstKind::Type(TypeKind::Typedef)),

            // Preprocessor directives
            "preproc_def" => Some(AstKind::Macro(MacroKind::ObjectLike)),
            "preproc_function_def" => Some(AstKind::Macro(MacroKind::FunctionLike)),
            "preproc_include" => Some(AstKind::Macro(MacroKind::Include)),
            "preproc_ifdef" | "preproc_ifndef" | "preproc_if" => {
                Some(AstKind::Macro(MacroKind::Conditional))
            }

            // Statements
            "if_statement" => Some(AstKind::Statement(StmtKind::If)),
            "while_statement" => Some(AstKind::Statement(StmtKind::While)),
            "do_statement" => Some(AstKind::Statement(StmtKind::DoWhile)),
            "for_statement" => Some(AstKind::Statement(StmtKind::For)),
            "switch_statement" => Some(AstKind::Statement(StmtKind::Switch)),
            "goto_statement" => Some(AstKind::Statement(StmtKind::Goto)),
            "labeled_statement" => Some(AstKind::Statement(StmtKind::Label)),
            "return_statement" => Some(AstKind::Statement(StmtKind::Return)),
            "compound_statement" => Some(AstKind::Statement(StmtKind::Block)),

            // Expressions
            "call_expression" => Some(AstKind::Expression(ExprKind::Call)),
            "identifier" => Some(AstKind::Expression(ExprKind::Identifier)),
            "binary_expression" => Some(AstKind::Expression(ExprKind::Binary)),
            "unary_expression" => Some(AstKind::Expression(ExprKind::Unary)),
            "number_literal" | "string_literal" | "char_literal" => {
                Some(AstKind::Expression(ExprKind::Literal))
            }

            _ => None,
        }
    }

    #[cfg(feature = "c-ast")]
    fn extract_node_info(&self, node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
        // Extract name for named entities
        if let Some(name) = self.extract_name(node, source) {
            // Store name in metadata (we'd need to extend UnifiedAstNode to store names properly)
            let name_hash = self.hash_name(&name);
            ast_node.name_vector = name_hash;
        }

        // Set appropriate flags
        match node.kind() {
            "function_definition" => {
                self.extract_function_flags(node, source, ast_node);
            }
            "declaration" => {
                self.extract_variable_flags(node, source, ast_node);
            }
            _ => {}
        }
    }

    #[cfg(feature = "c-ast")]
    fn extract_name(&self, node: &Node, source: &str) -> Option<String> {
        // Try to find identifier nodes
        let mut cursor = node.walk();
        if Self::find_child_by_kind(&mut cursor, "identifier") {
            let identifier_node = cursor.node();
            return Some(
                identifier_node
                    .utf8_text(source.as_bytes())
                    .ok()?
                    .to_string(),
            );
        }

        // For includes, extract the file name
        if node.kind() == "preproc_include"
            && (Self::find_child_by_kind(&mut cursor, "string_literal")
                || Self::find_child_by_kind(&mut cursor, "system_lib_string"))
        {
            let text = cursor.node().utf8_text(source.as_bytes()).ok()?;
            return Some(
                text.trim_matches('"')
                    .trim_matches('<')
                    .trim_matches('>')
                    .to_string(),
            );
        }

        None
    }

    #[cfg(feature = "c-ast")]
    fn find_child_by_kind(cursor: &mut TreeCursor, kind: &str) -> bool {
        if !cursor.goto_first_child() {
            return false;
        }

        loop {
            if cursor.node().kind() == kind {
                return true;
            }

            if Self::find_child_by_kind(cursor, kind) {
                return true;
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
        false
    }

    #[cfg(feature = "c-ast")]
    fn extract_function_flags(&self, node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    match text {
                        "static" => ast_node.flags.set(NodeFlags::STATIC),
                        "inline" => ast_node.flags.set(NodeFlags::INLINE),
                        "extern" => ast_node.flags.set(NodeFlags::EXTERN),
                        _ => {}
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    #[cfg(feature = "c-ast")]
    fn extract_variable_flags(&self, node: &Node, source: &str, ast_node: &mut UnifiedAstNode) {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    match text {
                        "const" => ast_node.flags.set(NodeFlags::CONST),
                        "volatile" => ast_node.flags.set(NodeFlags::VOLATILE),
                        "static" => ast_node.flags.set(NodeFlags::STATIC),
                        "extern" => ast_node.flags.set(NodeFlags::EXTERN),
                        "restrict" => ast_node.flags.set(NodeFlags::RESTRICT),
                        _ => {}
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    #[cfg(feature = "c-ast")]
    fn calculate_complexity(&self, node: &Node) -> u32 {
        let mut complexity = 1; // Base complexity
        let mut cursor = node.walk();

        Self::count_complexity_nodes(&mut cursor, &mut complexity);

        complexity
    }

    #[cfg(feature = "c-ast")]
    fn count_complexity_nodes(cursor: &mut TreeCursor, complexity: &mut u32) {
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            let node = cursor.node();
            match node.kind() {
                "if_statement" | "while_statement" | "for_statement" | "do_statement" => {
                    *complexity += 1;
                }
                "switch_statement" => {
                    *complexity += 1;
                    // Count case statements
                    let mut case_cursor = node.walk();
                    Self::count_case_statements(&mut case_cursor, complexity);
                }
                "goto_statement" => {
                    *complexity += 3; // Goto adds significant complexity
                }
                "logical_expression" => {
                    if let Ok(op) = node.utf8_text(&[]) {
                        if op.contains("&&") || op.contains("||") {
                            *complexity += 1;
                        }
                    }
                }
                _ => {}
            }

            Self::count_complexity_nodes(cursor, complexity);

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }

    #[cfg(feature = "c-ast")]
    fn count_case_statements(cursor: &mut TreeCursor, complexity: &mut u32) {
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            if cursor.node().kind() == "case_statement" {
                *complexity += 1;
            }

            Self::count_case_statements(cursor, complexity);

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }

    #[cfg(feature = "c-ast")]
    fn hash_name(&self, name: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_parse_simple_c_function() {
        let mut parser = CAstParser::new();
        let content = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_ok());

        let dag = result.unwrap();
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_parse_c_with_pointers() {
        let mut parser = CAstParser::new();
        let content = r#"
void process(int *ptr, char **argv) {
    *ptr = 42;
}
"#;
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_parse_c_with_goto() {
        let mut parser = CAstParser::new();
        let content = r#"
void example() {
    int i = 0;
    start:
    if (i < 10) {
        i++;
        goto start;
    }
}
"#;
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_ok());

        // Verify goto complexity
        let dag = result.unwrap();
        let mut found_goto = false;
        for node in dag.nodes.iter() {
            if matches!(node.kind, AstKind::Statement(StmtKind::Goto)) {
                found_goto = true;
                break;
            }
        }
        assert!(found_goto, "Should have found a goto statement");
    }

    #[test]
    #[cfg(not(feature = "c-ast"))]
    fn test_c_ast_disabled() {
        let mut parser = CAstParser::new();
        let content = "int main() { return 0; }";
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_err());
    }
}
