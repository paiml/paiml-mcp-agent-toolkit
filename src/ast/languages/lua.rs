#![cfg_attr(coverage_nightly, coverage(off))]
//! Lua language AST parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "lua-ast")]
use tree_sitter::{Parser as TsParser, Tree};

use super::LanguageStrategy;
use crate::ast::core::{AstDag, AstKind, Language, NodeFlags, UnifiedAstNode};

#[cfg(feature = "lua-ast")]
use crate::ast::core::{ClassKind, FunctionKind, ImportKind, StmtKind, VarKind};

/// Lua language parsing strategy
pub struct LuaStrategy;

impl Default for LuaStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "lua-ast")]
    fn parse_with_tree_sitter(&self, content: &str) -> Result<Tree> {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_lua::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Lua language: {e}"))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Lua code"))?;

        if Self::has_syntax_errors(&tree) {
            return Err(anyhow::anyhow!("Lua syntax error detected in source"));
        }

        Ok(tree)
    }

    #[cfg(feature = "lua-ast")]
    fn has_syntax_errors(tree: &Tree) -> bool {
        let root = tree.root_node();
        Self::node_has_error(&root)
    }

    #[cfg(feature = "lua-ast")]
    fn node_has_error(node: &tree_sitter::Node) -> bool {
        if node.kind() == "ERROR" || node.is_error() || node.is_missing() {
            return true;
        }

        for child in node.children(&mut node.walk()) {
            if Self::node_has_error(&child) {
                return true;
            }
        }

        false
    }

    #[cfg(not(feature = "lua-ast"))]
    #[allow(dead_code)]
    fn parse_with_tree_sitter(&self, _content: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "Lua AST parsing not available - compile with 'lua-ast' feature"
        ))
    }

    #[cfg(feature = "lua-ast")]
    fn convert_tree_to_dag(&self, tree: &Tree, content: &str) -> AstDag {
        let mut dag = AstDag::new();
        let root = tree.root_node();
        let mut visitor = LuaTreeSitterVisitor::new(&mut dag, content);
        visitor.visit_node(&root, None);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for LuaStrategy {
    fn language(&self) -> Language {
        Language::Lua
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "lua")
    }

    #[cfg(feature = "lua-ast")]
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let tree = self.parse_with_tree_sitter(content)?;
        Ok(self.convert_tree_to_dag(&tree, content))
    }

    #[cfg(not(feature = "lua-ast"))]
    async fn parse_file(&self, _path: &Path, _content: &str) -> Result<AstDag> {
        Err(anyhow::anyhow!(
            "Lua AST parsing not available - compile with 'lua-ast' feature"
        ))
    }

    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        let mut imports = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Import(_)) {
                    imports.push(format!("import_{i}"));
                }
            }
        }
        imports
    }

    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut functions = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Function(_)) {
                    functions.push(node.clone());
                }
            }
        }
        functions
    }

    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut types = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Class(_)) {
                    types.push(node.clone());
                }
            }
        }
        types
    }

    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        let mut cyclomatic = 1;
        let mut cognitive = 0;

        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::CONTROL_FLOW) {
                    cyclomatic += 1;
                    cognitive += 1;
                }
            }
        }

        (cyclomatic, cognitive)
    }
}

/// Tree-sitter-based Lua AST visitor
#[cfg(feature = "lua-ast")]
struct LuaTreeSitterVisitor<'a> {
    dag: &'a mut AstDag,
    content: &'a str,
    current_parent: Option<u32>,
}

#[cfg(feature = "lua-ast")]
impl<'a> LuaTreeSitterVisitor<'a> {
    fn new(dag: &'a mut AstDag, content: &'a str) -> Self {
        Self {
            dag,
            content,
            current_parent: None,
        }
    }

    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, Language::Lua);

        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }

    /// Check if a function_call node is a `require(...)` call
    fn is_require_call(&self, node: &tree_sitter::Node) -> bool {
        // In tree-sitter-lua 0.2.0, function_call has a child "name" or the
        // first named child is an identifier
        for child in node.children(&mut node.walk()) {
            if child.kind() == "identifier" {
                let text = child.utf8_text(self.content.as_bytes()).unwrap_or_default();
                if text == "require" {
                    return true;
                }
            }
            // Also check dot_index_expression for nested calls like foo.require
            // but the simple case is just `require("module")`
        }
        false
    }

    fn visit_node(&mut self, node: &tree_sitter::Node, parent: Option<u32>) {
        let old_parent = self.current_parent;
        self.current_parent = parent;

        match node.kind() {
            "function_definition" | "function_declaration" => {
                let key = self.add_node(AstKind::Function(FunctionKind::Regular));

                self.current_parent = Some(key);
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, Some(key));
                }
            }
            "variable_declaration" => {
                let _key = self.add_node(AstKind::Variable(VarKind::Let));

                // Visit children to capture nested function definitions
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "if_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "for_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::For), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "while_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::While), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "repeat_statement" => {
                let mut n =
                    UnifiedAstNode::new(AstKind::Statement(StmtKind::DoWhile), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "do_statement" => {
                let _key = self.add_node(AstKind::Statement(StmtKind::Block));

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "return_statement" => {
                let _key = self.add_node(AstKind::Statement(StmtKind::Return));

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "table_constructor" => {
                let _key = self.add_node(AstKind::Class(ClassKind::Regular));

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "function_call" => {
                if self.is_require_call(node) {
                    let mut n =
                        UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::Lua);
                    n.flags.set(NodeFlags::IMPORT);
                    self.dag.add_node(n);
                }

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "elseif_statement" => {
                let mut n = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Lua);
                n.flags.set(NodeFlags::CONTROL_FLOW);
                self.dag.add_node(n);

                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
            "binary_expression" => {
                // Check for `and` / `or` operators which add to cyclomatic complexity
                for child in node.children(&mut node.walk()) {
                    if child.kind() == "and" || child.kind() == "or" {
                        let mut n =
                            UnifiedAstNode::new(AstKind::Statement(StmtKind::Block), Language::Lua);
                        n.flags.set(NodeFlags::CONTROL_FLOW);
                        self.dag.add_node(n);
                    }
                    self.visit_node(&child, parent);
                }
            }
            _ => {
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child, parent);
                }
            }
        }

        self.current_parent = old_parent;
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_lua_strategy_language() {
        let strategy = LuaStrategy::new();
        assert_eq!(strategy.language(), Language::Lua);
    }

    #[test]
    fn test_lua_strategy_default() {
        let strategy = LuaStrategy::default();
        assert_eq!(strategy.language(), Language::Lua);
    }

    #[test]
    fn test_lua_can_parse() {
        let strategy = LuaStrategy::new();
        assert!(strategy.can_parse(Path::new("test.lua")));
        assert!(strategy.can_parse(Path::new("/path/to/script.lua")));
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.luac")));
        assert!(!strategy.can_parse(Path::new("test")));
        assert!(!strategy.can_parse(Path::new("")));
    }

    #[test]
    fn test_extract_imports_empty() {
        let strategy = LuaStrategy::new();
        let dag = AstDag::new();
        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_extract_functions_empty() {
        let strategy = LuaStrategy::new();
        let dag = AstDag::new();
        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_extract_types_empty() {
        let strategy = LuaStrategy::new();
        let dag = AstDag::new();
        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_calculate_complexity_empty() {
        let strategy = LuaStrategy::new();
        let dag = AstDag::new();
        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_calculate_complexity_with_control_flow() {
        let strategy = LuaStrategy::new();
        let mut dag = AstDag::new();

        let mut node1 = UnifiedAstNode::new(
            AstKind::Statement(crate::ast::core::StmtKind::If),
            Language::Lua,
        );
        node1.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node1);

        let mut node2 = UnifiedAstNode::new(
            AstKind::Statement(crate::ast::core::StmtKind::For),
            Language::Lua,
        );
        node2.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node2);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 3); // 1 base + 2 control flow
        assert_eq!(cognitive, 2);
    }

    #[cfg(feature = "lua-ast")]
    mod lua_ast_tests {
        use super::*;

        #[test]
        fn test_lua_parse_simple_function() {
            let strategy = LuaStrategy::new();
            let code = "function hello()\n  print('hello')\nend";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lua_parse_local_function() {
            let strategy = LuaStrategy::new();
            let code = "local function greet(name)\n  return 'Hello ' .. name\nend";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lua_parse_control_flow() {
            let strategy = LuaStrategy::new();
            let code = r#"
function classify(x)
    if x > 0 then
        return "positive"
    elseif x < 0 then
        return "negative"
    else
        return "zero"
    end
end

for i = 1, 10 do
    print(i)
end

while true do
    break
end

repeat
    x = x - 1
until x == 0
"#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let (cyclomatic, _cognitive) = strategy.calculate_complexity(&dag);
            // if + elseif + for + while + repeat = 5 control flow, base 1
            assert!(
                cyclomatic >= 6,
                "Expected cyclomatic >= 6, got {cyclomatic}"
            );
        }

        #[test]
        fn test_lua_extract_functions() {
            let strategy = LuaStrategy::new();
            let code = r#"
function hello()
    print("hello")
end

function goodbye()
    print("goodbye")
end
"#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);
            let functions = strategy.extract_functions(&dag);
            assert!(
                functions.len() >= 2,
                "Expected at least 2 functions, got {}",
                functions.len()
            );
        }

        #[test]
        fn test_lua_require_as_import() {
            let strategy = LuaStrategy::new();
            let code = r#"local json = require("dkjson")"#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);
            let imports = strategy.extract_imports(&dag);
            assert!(
                !imports.is_empty(),
                "require() should be detected as import"
            );
        }

        #[test]
        fn test_lua_table_constructor() {
            let strategy = LuaStrategy::new();
            let code = r#"local config = { width = 1920, height = 1080 }"#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);
            let types = strategy.extract_types(&dag);
            assert!(
                !types.is_empty(),
                "Table constructor should be detected as type"
            );
        }

        #[test]
        fn test_lua_and_or_complexity() {
            let strategy = LuaStrategy::new();
            let code = r#"
function check(a, b, c)
    if a and b or c then
        return true
    end
    return false
end
"#;
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);
            let (cyclomatic, _) = strategy.calculate_complexity(&dag);
            // if + and + or = 3 control flow, base 1
            assert!(
                cyclomatic >= 4,
                "Expected cyclomatic >= 4, got {cyclomatic}"
            );
        }

        #[test]
        fn test_lua_syntax_error() {
            let strategy = LuaStrategy::new();
            let code = "function foo(\n  end";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_lua_parse_file() {
            let strategy = LuaStrategy::new();
            let path = PathBuf::from("test.lua");
            let code = "function hello()\n  print('hello')\nend";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_ok());
        }

        #[test]
        fn test_lua_visitor_add_node() {
            let mut dag = AstDag::new();
            let content = "function foo() end";
            let mut visitor = LuaTreeSitterVisitor::new(&mut dag, content);

            let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));
            assert_eq!(key, 0);
            assert_eq!(dag.nodes.len(), 1);
        }

        #[test]
        fn test_lua_visitor_add_node_with_parent() {
            let mut dag = AstDag::new();
            let content = "function foo() end";
            let mut visitor = LuaTreeSitterVisitor::new(&mut dag, content);

            let parent_key = visitor.add_node(AstKind::Class(ClassKind::Regular));
            visitor.current_parent = Some(parent_key);

            let child_key = visitor.add_node(AstKind::Function(FunctionKind::Method));
            let child_node = dag.nodes.get(child_key).unwrap();
            assert_eq!(child_node.parent, parent_key);
        }

        #[test]
        fn test_lua_do_statement() {
            let strategy = LuaStrategy::new();
            let code = "do\n  local x = 1\nend";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);
            assert!(!dag.nodes.is_empty());
        }

        #[test]
        fn test_lua_return_statement() {
            let strategy = LuaStrategy::new();
            let code = "function foo()\n  return 42\nend";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_return = dag
                .nodes
                .iter()
                .any(|node| matches!(node.kind, AstKind::Statement(StmtKind::Return)));
            assert!(has_return, "Should have a return statement node");
        }

        #[test]
        fn test_lua_for_generic_statement() {
            let strategy = LuaStrategy::new();
            let code = "for k, v in pairs(t) do\n  print(k, v)\nend";
            let tree = strategy.parse_with_tree_sitter(code).unwrap();
            let dag = strategy.convert_tree_to_dag(&tree, code);

            let has_control_flow = dag
                .nodes
                .iter()
                .any(|node| node.flags.has(NodeFlags::CONTROL_FLOW));
            assert!(
                has_control_flow,
                "for-in (generic for) should add control flow"
            );
        }
    }

    #[cfg(not(feature = "lua-ast"))]
    mod non_lua_ast_tests {
        use super::*;

        #[tokio::test]
        async fn test_parse_file_without_feature() {
            let strategy = LuaStrategy::new();
            let path = PathBuf::from("test.lua");
            let code = "function hello() end";
            let result = strategy.parse_file(&path, code).await;
            assert!(result.is_err());
            assert!(result.err().unwrap().to_string().contains("lua-ast"));
        }

        #[test]
        fn test_parse_with_tree_sitter_without_feature() {
            let strategy = LuaStrategy::new();
            let code = "function hello() end";
            let result = strategy.parse_with_tree_sitter(code);
            assert!(result.is_err());
        }
    }
}
