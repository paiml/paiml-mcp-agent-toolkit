#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Property: Class method extraction must be stable across valid identifiers
        /// NASA-style: 1000+ iterations to verify edge cases
        #[test]
        fn property_class_method_extraction_is_stable(
            class_name in "[A-Z][a-zA-Z0-9]{2,20}",
            method_name in "[a-z][a-zA-Z0-9]{2,20}"
        ) {
            let analyzer = JavaScriptAnalyzer;
            let content = format!(
                "class {} {{\n    {}() {{\n        return 42;\n    }}\n}}",
                class_name, method_name
            );

            let functions = analyzer.extract_functions(&content);

            // Property: Must extract at least 1 function (the method)
            prop_assert!(
                !functions.is_empty(),
                "Class with method must extract at least 1 function, found {}. Content: {}",
                functions.len(),
                content
            );

            // Property: Method name must be detected in qualified form
            prop_assert!(
                functions.iter().any(|f| f.name.contains(&method_name)),
                "Must detect method '{}' in class '{}'. Found: {:?}",
                method_name,
                class_name,
                functions.iter().map(|f| &f.name).collect::<Vec<_>>()
            );
        }

        #[test]
        fn property_constructor_always_detected(
            class_name in "[A-Z][a-zA-Z0-9]{2,20}"
        ) {
            let analyzer = JavaScriptAnalyzer;
            let content = format!(
                "class {} {{\n    constructor() {{\n        this.value = 0;\n    }}\n}}",
                class_name
            );

            let functions = analyzer.extract_functions(&content);

            // Property: Constructor must always be detected
            prop_assert!(
                functions.iter().any(|f| f.name.contains("constructor")),
                "Class '{}' with constructor must detect constructor. Found: {:?}",
                class_name,
                functions.iter().map(|f| &f.name).collect::<Vec<_>>()
            );
        }

        #[test]
        fn property_static_methods_detected(
            class_name in "[A-Z][a-zA-Z0-9]{2,20}",
            method_name in "[a-z][a-zA-Z0-9]{2,20}"
        ) {
            let analyzer = JavaScriptAnalyzer;
            let content = format!(
                "class {} {{\n    static {}() {{\n        return true;\n    }}\n}}",
                class_name, method_name
            );

            let functions = analyzer.extract_functions(&content);

            // Property: Static methods must be detected
            prop_assert!(
                !functions.is_empty(),
                "Static method in class '{}' must be detected. Found {} functions",
                class_name,
                functions.len()
            );

            prop_assert!(
                functions.iter().any(|f| f.name.contains(&method_name)),
                "Static method '{}' in class '{}' must be detected. Found: {:?}",
                method_name,
                class_name,
                functions.iter().map(|f| &f.name).collect::<Vec<_>>()
            );
        }

        #[test]
        fn property_multiple_methods_counted_correctly(
            class_name in "[A-Z][a-zA-Z0-9]{2,20}",
            num_methods in 1usize..10
        ) {
            let analyzer = JavaScriptAnalyzer;

            // Generate class with N methods
            let mut methods = String::new();
            for i in 0..num_methods {
                methods.push_str(&format!("    method{}() {{ return {}; }}\n", i, i));
            }

            let content = format!("class {} {{\n{}}}",  class_name, methods);
            let functions = analyzer.extract_functions(&content);

            // Property: Number of extracted functions must match number of methods
            prop_assert!(
                functions.len() == num_methods,
                "Class '{}' with {} methods must extract exactly {} functions, found {}. Content:\n{}",
                class_name,
                num_methods,
                num_methods,
                functions.len(),
                content
            );
        }
    }

}
