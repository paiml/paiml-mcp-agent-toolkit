//! Feature-gated tests for C/C++ language strategies

#[cfg(test)]
#[cfg(feature = "c-ast")]
mod c_ast_tests {
    use crate::ast::core::{AstKind, ClassKind, NodeFlags};
    use crate::ast::languages::c_cpp::c_cpp_strategy::CStrategy;
    use crate::ast::languages::LanguageStrategy;
    use std::path::PathBuf;

    #[test]
    fn test_c_parse_with_tree_sitter_simple_function() {
        let strategy = CStrategy::new();
        let code = "int main() { return 0; }";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_parse_with_tree_sitter_struct() {
        let strategy = CStrategy::new();
        let code = "struct Point { int x; int y; };";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_parse_with_tree_sitter_enum() {
        let strategy = CStrategy::new();
        let code = "enum Color { RED, GREEN, BLUE };";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_parse_with_tree_sitter_include() {
        let strategy = CStrategy::new();
        let code = "#include <stdio.h>\nint main() { return 0; }";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_convert_to_dag_function() {
        let strategy = CStrategy::new();
        let code = "int add(int a, int b) { return a + b; }";
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_function = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Function(_)));
        assert!(has_function, "Should have a function node");
    }

    #[test]
    fn test_c_convert_to_dag_struct() {
        let strategy = CStrategy::new();
        let code = "struct Person { char* name; int age; };";
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_struct = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Struct)));
        assert!(has_struct, "Should have a struct node");
    }

    #[test]
    fn test_c_convert_to_dag_enum() {
        let strategy = CStrategy::new();
        let code = "enum Status { OK, ERROR, PENDING };";
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_enum = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Enum)));
        assert!(has_enum, "Should have an enum node");
    }

    #[test]
    fn test_c_convert_to_dag_control_flow() {
        let strategy = CStrategy::new();
        let code = r#"
            int foo(int x) {
                if (x > 0) {
                    return x;
                }
                while (x < 10) {
                    x++;
                }
                return x;
            }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let control_flow_count = dag
            .nodes
            .iter()
            .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
            .count();
        assert!(control_flow_count >= 2, "Should have control flow nodes");
    }

    #[test]
    fn test_c_convert_to_dag_include() {
        let strategy = CStrategy::new();
        let code = "#include <stdio.h>";
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_import = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Import(_)));
        assert!(has_import, "Should have an import node");
    }

    #[tokio::test]
    async fn test_c_parse_file_success() {
        let strategy = CStrategy::new();
        let path = PathBuf::from("test.c");
        let code = "int main() { return 0; }";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_c_parse_file_complex() {
        let strategy = CStrategy::new();
        let path = PathBuf::from("test.c");
        let code = r#"
            #include <stdio.h>
            #include <stdlib.h>

            struct Node {
                int value;
                struct Node* next;
            };

            enum ErrorCode {
                SUCCESS = 0,
                FAILURE = -1
            };

            int calculate(int x, int y) {
                if (x > y) {
                    return x - y;
                } else if (x < y) {
                    return y - x;
                }
                return 0;
            }

            int main() {
                for (int i = 0; i < 10; i++) {
                    printf("%d\n", i);
                }
                return 0;
            }
        "#;
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());

        let dag = result.unwrap();
        let functions = strategy.extract_functions(&dag);
        let types = strategy.extract_types(&dag);

        assert!(functions.len() >= 2);
        assert!(types.len() >= 2);
    }

    #[test]
    fn test_c_parse_empty_code() {
        let strategy = CStrategy::new();
        let code = "";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_parse_whitespace_only() {
        let strategy = CStrategy::new();
        let code = "   \n\t  \n  ";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_parse_comment_only() {
        let strategy = CStrategy::new();
        let code = "// This is a comment\n/* Multi-line\ncomment */";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_c_convert_to_dag_empty() {
        let strategy = CStrategy::new();
        let code = "";
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);
        assert!(dag.nodes.is_empty());
    }

    #[test]
    fn test_c_convert_to_dag_multiple_includes() {
        let strategy = CStrategy::new();
        let code = r#"
            #include <stdio.h>
            #include <stdlib.h>
            #include <string.h>
            #include "myheader.h"
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let import_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Import(_)))
            .count();
        assert_eq!(import_count, 4);
    }

    #[test]
    fn test_c_convert_to_dag_for_loop() {
        let strategy = CStrategy::new();
        let code = r#"
            void loop_test() {
                for (int i = 0; i < 10; i++) {
                    // body
                }
            }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let control_flow_count = dag
            .nodes
            .iter()
            .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
            .count();
        assert!(
            control_flow_count >= 1,
            "Should detect for loop as control flow"
        );
    }

    #[test]
    fn test_c_convert_to_dag_switch_statement() {
        let strategy = CStrategy::new();
        let code = r#"
            int switch_test(int x) {
                switch (x) {
                    case 0: return 0;
                    case 1: return 1;
                    default: return -1;
                }
            }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let control_flow_count = dag
            .nodes
            .iter()
            .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
            .count();
        assert!(
            control_flow_count >= 1,
            "Should detect switch as control flow"
        );
    }

    #[test]
    fn test_c_convert_to_dag_nested_control_flow() {
        let strategy = CStrategy::new();
        let code = r#"
            void nested_test(int x) {
                if (x > 0) {
                    while (x > 0) {
                        for (int i = 0; i < x; i++) {
                            if (i % 2 == 0) {
                                // even
                            }
                        }
                        x--;
                    }
                }
            }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let control_flow_count = dag
            .nodes
            .iter()
            .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
            .count();
        assert!(
            control_flow_count >= 4,
            "Should detect nested control flow structures"
        );
    }

    #[test]
    fn test_c_convert_to_dag_multiple_functions() {
        let strategy = CStrategy::new();
        let code = r#"
            int func1() { return 1; }
            int func2() { return 2; }
            int func3() { return 3; }
            int main() { return 0; }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let function_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Function(_)))
            .count();
        assert!(function_count >= 4, "Should detect all 4 functions");
    }

    #[test]
    fn test_c_convert_to_dag_multiple_structs() {
        let strategy = CStrategy::new();
        let code = r#"
            struct Point { int x; int y; };
            struct Line { struct Point start; struct Point end; };
            struct Rectangle { struct Point topLeft; int width; int height; };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let struct_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Struct)))
            .count();
        assert!(struct_count >= 3, "Should detect all 3 structs");
    }

    #[test]
    fn test_c_convert_to_dag_mixed_types() {
        let strategy = CStrategy::new();
        let code = r#"
            struct Point { int x; int y; };
            enum Color { RED, GREEN, BLUE };
            typedef struct { float r; float g; float b; } RGB;
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let struct_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Struct)))
            .count();
        let enum_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Enum)))
            .count();

        assert!(struct_count >= 1, "Should detect struct");
        assert!(enum_count >= 1, "Should detect enum");
    }

    #[tokio::test]
    async fn test_c_parse_file_empty() {
        let strategy = CStrategy::new();
        let path = PathBuf::from("empty.c");
        let code = "";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_c_parse_file_minimal() {
        let strategy = CStrategy::new();
        let path = PathBuf::from("minimal.c");
        let code = "void f(){}";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());

        let dag = result.unwrap();
        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty());
    }

    #[test]
    fn test_c_complexity_integration() {
        let strategy = CStrategy::new();
        let code = r#"
            int complex_function(int x, int y) {
                if (x > 0) {
                    if (y > 0) {
                        return x + y;
                    }
                }
                while (x < y) {
                    for (int i = 0; i < 10; i++) {
                        x++;
                    }
                }
                switch (x) {
                    case 0: return 0;
                    case 1: return 1;
                    default: return -1;
                }
            }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert!(
            cyclomatic > 1,
            "Complex function should have high cyclomatic complexity"
        );
        assert!(
            cognitive > 0,
            "Complex function should have high cognitive complexity"
        );
    }
}

#[cfg(test)]
#[cfg(feature = "cpp-ast")]
mod cpp_ast_tests {
    use crate::ast::core::{AstKind, ClassKind, NodeFlags};
    use crate::ast::languages::c_cpp::c_cpp_strategy::CppStrategy;
    use crate::ast::languages::LanguageStrategy;
    use std::path::PathBuf;

    #[test]
    fn test_cpp_parse_with_tree_sitter_simple_function() {
        let strategy = CppStrategy::new();
        let code = "int main() { return 0; }";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cpp_parse_with_tree_sitter_class() {
        let strategy = CppStrategy::new();
        let code = "class MyClass { public: void doSomething(); };";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cpp_parse_with_tree_sitter_struct() {
        let strategy = CppStrategy::new();
        let code = "struct Point { int x; int y; };";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cpp_convert_to_dag_class() {
        let strategy = CppStrategy::new();
        let code = r#"
            class Rectangle {
            public:
                Rectangle(int w, int h) : width(w), height(h) {}
                int area() { return width * height; }
            private:
                int width;
                int height;
            };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_class = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)));
        assert!(has_class, "Should have a class node");
    }

    #[test]
    fn test_cpp_convert_to_dag_function() {
        let strategy = CppStrategy::new();
        let code = "int add(int a, int b) { return a + b; }";
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_function = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Function(_)));
        assert!(has_function, "Should have a function node");
    }

    #[test]
    fn test_cpp_convert_to_dag_control_flow() {
        let strategy = CppStrategy::new();
        let code = r#"
            int foo(int x) {
                if (x > 0) {
                    return x;
                }
                switch (x) {
                    case 0: return 0;
                    default: return -1;
                }
            }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let control_flow_count = dag
            .nodes
            .iter()
            .filter(|node| node.flags.has(NodeFlags::CONTROL_FLOW))
            .count();
        assert!(control_flow_count >= 2, "Should have control flow nodes");
    }

    #[tokio::test]
    async fn test_cpp_parse_file_success() {
        let strategy = CppStrategy::new();
        let path = PathBuf::from("test.cpp");
        let code = "int main() { return 0; }";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cpp_parse_file_complex() {
        let strategy = CppStrategy::new();
        let path = PathBuf::from("test.cpp");
        let code = r#"
            #include <iostream>
            #include <vector>

            class Stack {
            public:
                void push(int value) {
                    data.push_back(value);
                }

                int pop() {
                    if (data.empty()) {
                        return -1;
                    }
                    int val = data.back();
                    data.pop_back();
                    return val;
                }

                bool isEmpty() const {
                    return data.empty();
                }

            private:
                std::vector<int> data;
            };

            enum class Status {
                OK,
                ERROR,
                PENDING
            };

            int main() {
                Stack s;
                for (int i = 0; i < 10; ++i) {
                    s.push(i);
                }
                while (!s.isEmpty()) {
                    std::cout << s.pop() << std::endl;
                }
                return 0;
            }
        "#;
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());

        let dag = result.unwrap();
        let functions = strategy.extract_functions(&dag);
        let types = strategy.extract_types(&dag);

        assert!(functions.len() >= 3);
        assert!(types.len() >= 1);
    }

    #[test]
    fn test_cpp_parse_empty_code() {
        let strategy = CppStrategy::new();
        let code = "";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cpp_parse_whitespace_only() {
        let strategy = CppStrategy::new();
        let code = "   \n\t  \n  ";
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cpp_parse_namespace() {
        let strategy = CppStrategy::new();
        let code = r#"
            namespace MyNamespace {
                class MyClass {
                public:
                    void doSomething();
                };
            }
        "#;
        let result = strategy.parse_with_tree_sitter(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cpp_convert_to_dag_empty() {
        let strategy = CppStrategy::new();
        let code = "";
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);
        assert!(dag.nodes.is_empty());
    }

    #[test]
    fn test_cpp_convert_to_dag_multiple_classes() {
        let strategy = CppStrategy::new();
        let code = r#"
            class Shape { public: virtual void draw() = 0; };
            class Circle : public Shape { public: void draw() override {} };
            class Rectangle : public Shape { public: void draw() override {} };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let class_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)))
            .count();
        assert!(class_count >= 3, "Should detect all 3 classes");
    }

    #[test]
    fn test_cpp_convert_to_dag_template() {
        let strategy = CppStrategy::new();
        let code = r#"
            template<typename T>
            class Container {
            public:
                void add(T item);
                T get(int index);
            };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_class = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Class(_)));
        assert!(has_class, "Should detect template class");
    }

    #[test]
    fn test_cpp_convert_to_dag_enum_class() {
        let strategy = CppStrategy::new();
        let code = r#"
            enum class Color { Red, Green, Blue };
            enum class Direction : int { North = 0, South = 1, East = 2, West = 3 };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let enum_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Enum)))
            .count();
        assert!(enum_count >= 2, "Should detect enum classes");
    }

    #[test]
    fn test_cpp_convert_to_dag_lambda() {
        let strategy = CppStrategy::new();
        let code = r#"
            void lambda_test() {
                auto lambda = [](int x) { return x * 2; };
                auto capture = [&](int y) { return y + 1; };
            }
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let has_function = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Function(_)));
        assert!(has_function, "Should detect function with lambdas");
    }

    #[test]
    fn test_cpp_convert_to_dag_nested_classes() {
        let strategy = CppStrategy::new();
        let code = r#"
            class Outer {
            public:
                class Inner {
                public:
                    void innerMethod();
                };
                void outerMethod();
            };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let class_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)))
            .count();
        assert!(class_count >= 2, "Should detect nested classes");
    }

    #[tokio::test]
    async fn test_cpp_parse_file_empty() {
        let strategy = CppStrategy::new();
        let path = PathBuf::from("empty.cpp");
        let code = "";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cpp_parse_file_minimal() {
        let strategy = CppStrategy::new();
        let path = PathBuf::from("minimal.cpp");
        let code = "void f(){}";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());

        let dag = result.unwrap();
        let functions = strategy.extract_functions(&dag);
        assert!(!functions.is_empty());
    }

    #[test]
    fn test_cpp_convert_to_dag_with_inheritance() {
        let strategy = CppStrategy::new();
        let code = r#"
            class Base {
            public:
                virtual void method() {}
            };
            class Derived : public Base {
            public:
                void method() override {}
            };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let class_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Class(ClassKind::Regular)))
            .count();
        assert!(
            class_count >= 2,
            "Should detect both base and derived classes"
        );
    }

    #[test]
    fn test_cpp_complexity_integration() {
        let strategy = CppStrategy::new();
        let code = r#"
            class Calculator {
            public:
                int compute(int x, int y) {
                    if (x > 0) {
                        if (y > 0) {
                            return x + y;
                        }
                    }
                    while (x < y) {
                        for (int i = 0; i < 10; i++) {
                            x++;
                        }
                    }
                    return 0;
                }
            };
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert!(
            cyclomatic > 1,
            "Complex method should have high cyclomatic complexity"
        );
        assert!(
            cognitive > 0,
            "Complex method should have high cognitive complexity"
        );
    }

    #[test]
    fn test_cpp_convert_to_dag_multiple_includes() {
        let strategy = CppStrategy::new();
        let code = r#"
            #include <iostream>
            #include <vector>
            #include <string>
            #include <memory>
            #include "myheader.hpp"
        "#;
        let tree = strategy.parse_with_tree_sitter(code).unwrap();
        let dag = strategy.convert_to_dag(&tree, code);

        let import_count = dag
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, AstKind::Import(_)))
            .count();
        assert_eq!(import_count, 5);
    }
}

#[cfg(test)]
#[cfg(any(feature = "c-ast", feature = "cpp-ast"))]
mod visitor_tests {
    use crate::ast::core::{AstDag, AstKind, ClassKind, FunctionKind, Language};
    use crate::ast::languages::c_cpp_visitor::CTreeSitterVisitor;

    #[test]
    fn test_visitor_new() {
        let mut dag = AstDag::new();
        let content = "int main() {}";
        let visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);
        assert!(visitor.current_parent.is_none());
    }

    #[test]
    fn test_visitor_add_node() {
        let mut dag = AstDag::new();
        let content = "int main() {}";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));
        assert_eq!(key, 0);
        assert_eq!(dag.nodes.len(), 1);

        let node = dag.nodes.get(key).unwrap();
        assert_eq!(node.lang, Language::C);
    }

    #[test]
    fn test_visitor_add_node_with_parent() {
        let mut dag = AstDag::new();
        let content = "struct Foo { void bar(); };";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);

        // Add parent node
        let parent_key = visitor.add_node(AstKind::Class(ClassKind::Struct));
        visitor.current_parent = Some(parent_key);

        // Add child node
        let child_key = visitor.add_node(AstKind::Function(FunctionKind::Method));

        // Verify parent is set
        let child_node = dag.nodes.get(child_key).unwrap();
        assert_eq!(child_node.parent, parent_key);
    }

    #[test]
    fn test_visitor_with_cpp_language() {
        let mut dag = AstDag::new();
        let content = "class Foo {};";
        let visitor = CTreeSitterVisitor::new(&mut dag, content, Language::Cpp);
        assert_eq!(visitor.language, Language::Cpp);
        assert!(visitor.current_parent.is_none());
    }

    #[test]
    fn test_visitor_multiple_nodes() {
        let mut dag = AstDag::new();
        let content = "struct Foo {}; int bar();";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        // Add multiple nodes
        let _ = visitor.add_node(AstKind::Class(ClassKind::Struct));
        let _ = visitor.add_node(AstKind::Function(FunctionKind::Regular));
        let _ = visitor.add_node(AstKind::Import(crate::ast::core::ImportKind::Module));

        assert_eq!(dag.nodes.len(), 3);
    }

    #[test]
    fn test_visitor_parent_chain() {
        let mut dag = AstDag::new();
        let content = "struct Outer { struct Inner {}; };";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        // Create parent-child relationship
        let outer_key = visitor.add_node(AstKind::Class(ClassKind::Struct));
        visitor.current_parent = Some(outer_key);
        let inner_key = visitor.add_node(AstKind::Class(ClassKind::Struct));

        assert_eq!(dag.nodes.get(inner_key).unwrap().parent, outer_key);
    }

    #[test]
    fn test_visitor_no_parent() {
        let mut dag = AstDag::new();
        let content = "int main() {}";
        let mut visitor = CTreeSitterVisitor::new(&mut dag, content, Language::C);

        let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));
        // Parent should be 0 (default) when no parent is set
        assert_eq!(dag.nodes.get(key).unwrap().parent, 0);
    }
}

#[cfg(test)]
#[cfg(not(feature = "c-ast"))]
mod non_c_ast_tests {
    use crate::ast::languages::c_cpp::c_cpp_strategy::CStrategy;
    use crate::ast::languages::LanguageStrategy;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_c_parse_file_without_feature() {
        let strategy = CStrategy::new();
        let path = PathBuf::from("test.c");
        let code = "int main() { return 0; }";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("c-ast"));
    }
}

#[cfg(test)]
#[cfg(not(feature = "cpp-ast"))]
mod non_cpp_ast_tests {
    use crate::ast::languages::c_cpp::c_cpp_strategy::CppStrategy;
    use crate::ast::languages::LanguageStrategy;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_cpp_parse_file_without_feature() {
        let strategy = CppStrategy::new();
        let path = PathBuf::from("test.cpp");
        let code = "int main() { return 0; }";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("cpp-ast"));
    }
}
