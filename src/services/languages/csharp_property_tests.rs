// C# property-based tests - included from csharp.rs

#[cfg(all(test, feature = "csharp-ast"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_csharp_visitor_handles_any_valid_namespace_name(
            namespace_name in "[A-Z][a-zA-Z0-9_]*\\.[A-Z][a-zA-Z0-9_]*"
        ) {
            let source = format!("namespace {} {{ public class TestClass {{}} }}", namespace_name);
            let visitor = CSharpAstVisitor::new(Path::new("test.cs"));

            if let Ok(items) = visitor.analyze_csharp_source(&source) {
                // Should extract namespace and class
                prop_assert!(!items.is_empty());

                // Check that namespace name is included in qualified names
                let has_namespace_prefix = items.iter().any(|item| match item {
                    AstItem::Struct { name, .. } => name.starts_with(&format!("{}::", namespace_name)),
                    _ => false,
                });
                prop_assert!(has_namespace_prefix);
            }
        }

        #[test]
        fn test_csharp_complexity_analyzer_bounds(
            method_count in 1usize..10
        ) {
            let mut source = String::from("namespace Test { public class Test {\n");
            for i in 0..method_count {
                source.push_str(&format!("public void Method{}() {{}}\n", i));
            }
            source.push_str("} }\n");

            let visitor = CSharpAstVisitor::new(Path::new("test.cs"));
            if let Ok(items) = visitor.analyze_csharp_source(&source) {
                let method_items: Vec<_> = items.iter()
                    .filter(|item| matches!(item, AstItem::Function { .. }))
                    .collect();

                // Should extract all methods
                prop_assert_eq!(method_items.len(), method_count);

                // All should be methods with real names
                for (i, item) in method_items.iter().enumerate() {
                    if let AstItem::Function { name, .. } = item {
                        let expected_name = format!("Method{}", i);
                        prop_assert!(name.contains(&expected_name));
                    }
                }
            }
        }

        #[test]
        fn test_csharp_complexity_stays_bounded(
            depth in 1u32..5
        ) {
            let mut source = String::from("namespace Test { public class Test { public void ComplexMethod() {\n");
            for _ in 0..depth {
                source.push_str("if (true) {\n");
            }
            source.push_str("return;\n");
            for _ in 0..depth {
                source.push_str("}\n");
            }
            source.push_str("} } }\n");

            let mut analyzer = CSharpComplexityAnalyzer::new();
            if let Ok((cyclomatic, cognitive)) = analyzer.analyze_complexity(&source) {
                // Complexity should grow but stay reasonable
                prop_assert!(cyclomatic >= depth);
                prop_assert!(cognitive >= depth);
                prop_assert!(cyclomatic <= depth * 2 + 5); // Reasonable upper bound
                prop_assert!(cognitive <= depth * 3 + 5); // Reasonable upper bound
            }
        }
    }
}
