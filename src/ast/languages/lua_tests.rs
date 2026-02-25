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
