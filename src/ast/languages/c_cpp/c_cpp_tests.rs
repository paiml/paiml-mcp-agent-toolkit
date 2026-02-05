//! Tests for C/C++ language strategies

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::c_cpp_strategy::{CStrategy, CppStrategy};
    use crate::ast::core::{
        AstDag, AstKind, ClassKind, FunctionKind, ImportKind, Language, NodeFlags, StmtKind,
        UnifiedAstNode,
    };
    use crate::ast::languages::LanguageStrategy;
    use std::path::Path;
    #[allow(unused_imports)]
    use std::path::PathBuf;

    // ==================== CStrategy Tests ====================

    #[test]
    fn test_c_strategy_new() {
        let strategy = CStrategy::new();
        assert_eq!(strategy.language(), Language::C);
    }

    #[test]
    fn test_c_strategy_default() {
        let strategy = CStrategy::default();
        assert_eq!(strategy.language(), Language::C);
    }

    #[test]
    fn test_c_can_parse_c_file() {
        let strategy = CStrategy::new();
        assert!(strategy.can_parse(Path::new("test.c")));
        assert!(strategy.can_parse(Path::new("/path/to/source.c")));
    }

    #[test]
    fn test_c_can_parse_h_file() {
        let strategy = CStrategy::new();
        assert!(strategy.can_parse(Path::new("test.h")));
        assert!(strategy.can_parse(Path::new("header.h")));
    }

    #[test]
    fn test_c_can_parse_non_c_files() {
        let strategy = CStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.cpp")));
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("test.hpp")));
        assert!(!strategy.can_parse(Path::new("test")));
        assert!(!strategy.can_parse(Path::new("")));
    }

    #[test]
    fn test_c_extract_imports_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_c_extract_imports_with_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::C);
        dag.add_node(node);

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 1);
    }

    #[test]
    fn test_c_extract_functions_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_c_extract_functions_with_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::C);
        dag.add_node(node);

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 1);
    }

    #[test]
    fn test_c_extract_types_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_c_extract_types_with_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::C);
        dag.add_node(node);

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 1);
    }

    #[test]
    fn test_c_calculate_complexity_empty() {
        let strategy = CStrategy::new();
        let dag = AstDag::new();
        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_c_calculate_complexity_with_control_flow() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        let mut node = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::C);
        node.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 2);
        assert_eq!(cognitive, 1);
    }

    // ==================== CppStrategy Tests ====================

    #[test]
    fn test_cpp_strategy_new() {
        let strategy = CppStrategy::new();
        assert_eq!(strategy.language(), Language::Cpp);
    }

    #[test]
    fn test_cpp_strategy_default() {
        let strategy = CppStrategy::default();
        assert_eq!(strategy.language(), Language::Cpp);
    }

    #[test]
    fn test_cpp_can_parse_cpp_file() {
        let strategy = CppStrategy::new();
        assert!(strategy.can_parse(Path::new("test.cpp")));
        assert!(strategy.can_parse(Path::new("test.cc")));
        assert!(strategy.can_parse(Path::new("test.cxx")));
    }

    #[test]
    fn test_cpp_can_parse_hpp_file() {
        let strategy = CppStrategy::new();
        assert!(strategy.can_parse(Path::new("test.hpp")));
        assert!(strategy.can_parse(Path::new("test.hh")));
        assert!(strategy.can_parse(Path::new("test.hxx")));
    }

    #[test]
    fn test_cpp_can_parse_non_cpp_files() {
        let strategy = CppStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.c")));
        assert!(!strategy.can_parse(Path::new("test.h")));
        assert!(!strategy.can_parse(Path::new("test.rs")));
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("test")));
    }

    #[test]
    fn test_cpp_extract_imports() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::Cpp);
        dag.add_node(node);

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 1);
    }

    #[test]
    fn test_cpp_extract_functions() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Cpp);
        dag.add_node(node);

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 1);
    }

    #[test]
    fn test_cpp_extract_types() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Class(ClassKind::Regular), Language::Cpp);
        dag.add_node(node);

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 1);
    }

    #[test]
    fn test_cpp_calculate_complexity() {
        let strategy = CppStrategy::new();
        let mut dag = AstDag::new();

        let mut node = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::Cpp);
        node.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 2);
        assert_eq!(cognitive, 1);
    }

    // ==================== Additional Coverage Tests ====================

    #[test]
    fn test_c_extract_imports_multiple() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple import nodes
        for _ in 0..5 {
            let node = UnifiedAstNode::new(AstKind::Import(ImportKind::Module), Language::C);
            dag.add_node(node);
        }

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 5);
        assert!(imports[0].starts_with("import_"));
    }

    #[test]
    fn test_c_extract_functions_multiple() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple function nodes
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Method),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Lambda),
            Language::C,
        ));

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 3);
    }

    #[test]
    fn test_c_extract_types_multiple() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple type nodes
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Enum),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Regular),
            Language::C,
        ));

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_c_calculate_complexity_multiple_control_flow() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add multiple control flow nodes
        for _ in 0..5 {
            let mut node = UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::C);
            node.flags.set(NodeFlags::CONTROL_FLOW);
            dag.add_node(node);
        }

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 6); // 1 + 5
        assert_eq!(cognitive, 5);
    }

    #[test]
    fn test_c_extract_imports_mixed_nodes() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add mixed node types
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Module),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Named),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 2);
    }

    #[test]
    fn test_c_extract_functions_no_functions() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add non-function nodes only
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Module),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Statement(StmtKind::If),
            Language::C,
        ));

        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_c_extract_types_no_types() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add non-type nodes only
        dag.add_node(UnifiedAstNode::new(
            AstKind::Import(ImportKind::Module),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));

        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_c_calculate_complexity_no_control_flow() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add nodes without CONTROL_FLOW flag
        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::C,
        ));
        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::C,
        ));

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1); // Base complexity
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_cpp_extract_imports_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_cpp_extract_functions_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    #[test]
    fn test_cpp_extract_types_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    #[test]
    fn test_cpp_calculate_complexity_empty() {
        let strategy = CppStrategy::new();
        let dag = AstDag::new();
        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1);
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_c_can_parse_edge_cases() {
        let strategy = CStrategy::new();
        // Test with dots in directory names
        assert!(strategy.can_parse(Path::new("/path.to.something/file.c")));
        assert!(strategy.can_parse(Path::new("./relative/path.h")));
        // Test with multiple extensions (should only look at last)
        assert!(!strategy.can_parse(Path::new("file.c.bak")));
        assert!(!strategy.can_parse(Path::new("file.h.old")));
    }

    #[test]
    fn test_cpp_can_parse_edge_cases() {
        let strategy = CppStrategy::new();
        // Test with dots in directory names
        assert!(strategy.can_parse(Path::new("/path.to.something/file.cpp")));
        assert!(strategy.can_parse(Path::new("./relative/path.hpp")));
        // Test with multiple extensions
        assert!(!strategy.can_parse(Path::new("file.cpp.bak")));
        assert!(!strategy.can_parse(Path::new("file.hpp.old")));
    }

    #[test]
    fn test_extract_imports_large_dag() {
        let strategy = CStrategy::new();
        let mut dag = AstDag::new();

        // Add many nodes of various types
        for i in 0..100 {
            match i % 4 {
                0 => {
                    dag.add_node(UnifiedAstNode::new(
                        AstKind::Import(ImportKind::Module),
                        Language::C,
                    ));
                }
                1 => {
                    dag.add_node(UnifiedAstNode::new(
                        AstKind::Function(FunctionKind::Regular),
                        Language::C,
                    ));
                }
                2 => {
                    dag.add_node(UnifiedAstNode::new(
                        AstKind::Class(ClassKind::Struct),
                        Language::C,
                    ));
                }
                _ => {
                    let mut node =
                        UnifiedAstNode::new(AstKind::Statement(StmtKind::If), Language::C);
                    node.flags.set(NodeFlags::CONTROL_FLOW);
                    dag.add_node(node);
                }
            }
        }

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 25);

        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 25);

        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 25);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 26); // 1 + 25
        assert_eq!(cognitive, 25);
    }

    #[test]
    fn test_c_language_identity() {
        let strategy = CStrategy::new();
        assert_eq!(strategy.language(), Language::C);

        let strategy2 = CStrategy::default();
        assert_eq!(strategy2.language(), Language::C);
    }

    #[test]
    fn test_cpp_language_identity() {
        let strategy = CppStrategy::new();
        assert_eq!(strategy.language(), Language::Cpp);

        let strategy2 = CppStrategy::default();
        assert_eq!(strategy2.language(), Language::Cpp);
    }
}
