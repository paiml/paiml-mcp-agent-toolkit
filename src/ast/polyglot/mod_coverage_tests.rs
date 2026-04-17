#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ==========================================================================
    // Language enum tests - complete coverage
    // ==========================================================================

    #[test]
    fn test_language_name_all_variants() {
        assert_eq!(Language::Java.name(), "Java");
        assert_eq!(Language::Kotlin.name(), "Kotlin");
        assert_eq!(Language::Scala.name(), "Scala");
        assert_eq!(Language::TypeScript.name(), "TypeScript");
        assert_eq!(Language::JavaScript.name(), "JavaScript");
        assert_eq!(Language::Python.name(), "Python");
        assert_eq!(Language::Rust.name(), "Rust");
        assert_eq!(Language::Go.name(), "Go");
        assert_eq!(Language::Cpp.name(), "C++");
        assert_eq!(Language::CSharp.name(), "C#");
        assert_eq!(Language::Ruby.name(), "Ruby");
        assert_eq!(Language::Swift.name(), "Swift");
        assert_eq!(Language::Php.name(), "PHP");
        assert_eq!(Language::Other(42).name(), "Other");
        assert_eq!(Language::Other(0).name(), "Other");
    }

    #[test]
    fn test_language_from_extension_all_variants() {
        // Java
        assert_eq!(Language::from_extension("java"), Some(Language::Java));

        // Kotlin
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("kts"), Some(Language::Kotlin));

        // Scala
        assert_eq!(Language::from_extension("scala"), Some(Language::Scala));
        assert_eq!(Language::from_extension("sc"), Some(Language::Scala));

        // TypeScript
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::TypeScript));

        // JavaScript
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("jsx"), Some(Language::JavaScript));

        // Python
        assert_eq!(Language::from_extension("py"), Some(Language::Python));

        // Rust
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));

        // Go
        assert_eq!(Language::from_extension("go"), Some(Language::Go));

        // C++
        assert_eq!(Language::from_extension("cpp"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("cc"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("cxx"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("c++"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("h"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("hpp"), Some(Language::Cpp));

        // C#
        assert_eq!(Language::from_extension("cs"), Some(Language::CSharp));

        // Ruby
        assert_eq!(Language::from_extension("rb"), Some(Language::Ruby));

        // Swift
        assert_eq!(Language::from_extension("swift"), Some(Language::Swift));

        // PHP
        assert_eq!(Language::from_extension("php"), Some(Language::Php));

        // Unknown extensions
        assert_eq!(Language::from_extension("txt"), None);
        assert_eq!(Language::from_extension("md"), None);
        assert_eq!(Language::from_extension(""), None);
    }

    #[test]
    fn test_language_from_extension_case_insensitive() {
        assert_eq!(Language::from_extension("JAVA"), Some(Language::Java));
        assert_eq!(Language::from_extension("Java"), Some(Language::Java));
        assert_eq!(Language::from_extension("TS"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("PY"), Some(Language::Python));
        assert_eq!(Language::from_extension("RS"), Some(Language::Rust));
    }

    #[test]
    fn test_language_from_path_all_extensions() {
        assert_eq!(
            Language::from_path(Path::new("Test.java")),
            Some(Language::Java)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.kt")),
            Some(Language::Kotlin)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.scala")),
            Some(Language::Scala)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.ts")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.js")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.py")),
            Some(Language::Python)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.go")),
            Some(Language::Go)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.cpp")),
            Some(Language::Cpp)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.cs")),
            Some(Language::CSharp)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.rb")),
            Some(Language::Ruby)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.swift")),
            Some(Language::Swift)
        );
        assert_eq!(
            Language::from_path(Path::new("Test.php")),
            Some(Language::Php)
        );
    }

    #[test]
    fn test_language_from_path_edge_cases() {
        // No extension
        assert_eq!(Language::from_path(Path::new("Makefile")), None);
        assert_eq!(Language::from_path(Path::new("README")), None);

        // Hidden files
        assert_eq!(Language::from_path(Path::new(".gitignore")), None);

        // Deep paths
        assert_eq!(
            Language::from_path(Path::new("/a/b/c/d/e.java")),
            Some(Language::Java)
        );

        // Relative paths
        assert_eq!(
            Language::from_path(Path::new("./src/main.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::from_path(Path::new("../lib.py")),
            Some(Language::Python)
        );
    }

    #[test]
    fn test_language_file_extensions_all_variants() {
        assert_eq!(Language::Java.file_extensions(), vec!["java"]);
        assert_eq!(Language::Kotlin.file_extensions(), vec!["kt", "kts"]);
        assert_eq!(Language::Scala.file_extensions(), vec!["scala", "sc"]);
        assert_eq!(Language::TypeScript.file_extensions(), vec!["ts", "tsx"]);
        assert_eq!(Language::JavaScript.file_extensions(), vec!["js", "jsx"]);
        assert_eq!(Language::Python.file_extensions(), vec!["py"]);
        assert_eq!(Language::Rust.file_extensions(), vec!["rs"]);
        assert_eq!(Language::Go.file_extensions(), vec!["go"]);
        assert_eq!(
            Language::Cpp.file_extensions(),
            vec!["cpp", "cc", "cxx", "c++", "h", "hpp"]
        );
        assert_eq!(Language::CSharp.file_extensions(), vec!["cs"]);
        assert_eq!(Language::Ruby.file_extensions(), vec!["rb"]);
        assert_eq!(Language::Swift.file_extensions(), vec!["swift"]);
        assert_eq!(Language::Php.file_extensions(), vec!["php"]);
        assert!(Language::Other(0).file_extensions().is_empty());
        assert!(Language::Other(100).file_extensions().is_empty());
    }

    #[test]
    fn test_language_equality_and_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Language::Java);
        set.insert(Language::Kotlin);
        set.insert(Language::Other(1));
        set.insert(Language::Other(2));

        assert!(set.contains(&Language::Java));
        assert!(set.contains(&Language::Kotlin));
        assert!(set.contains(&Language::Other(1)));
        assert!(set.contains(&Language::Other(2)));
        assert!(!set.contains(&Language::Python));
        assert!(!set.contains(&Language::Other(3)));
    }

    #[test]
    fn test_language_clone_and_copy() {
        let lang = Language::TypeScript;
        let cloned = lang;
        let copied = lang;

        assert_eq!(lang, cloned);
        assert_eq!(lang, copied);
    }

    #[test]
    fn test_language_debug() {
        let debug_str = format!("{:?}", Language::Java);
        assert_eq!(debug_str, "Java");

        let debug_other = format!("{:?}", Language::Other(42));
        assert_eq!(debug_other, "Other(42)");
    }

    // ==========================================================================
    // NodeKind enum tests - complete coverage
    // ==========================================================================

    #[test]
    fn test_node_kind_from_ast_item_all_variants() {
        let function_item = AstItem::Function {
            name: "test".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&function_item), NodeKind::Function);

        let struct_item = AstItem::Struct {
            name: "Test".to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&struct_item), NodeKind::Struct);

        let enum_item = AstItem::Enum {
            name: "MyEnum".to_string(),
            visibility: "pub".to_string(),
            variants_count: 3,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&enum_item), NodeKind::Enum);

        let trait_item = AstItem::Trait {
            name: "MyTrait".to_string(),
            visibility: "pub".to_string(),
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&trait_item), NodeKind::Trait);

        let impl_item = AstItem::Impl {
            type_name: "MyType".to_string(),
            trait_name: None,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&impl_item), NodeKind::Implements);

        let use_item = AstItem::Use {
            path: "std::io".to_string(),
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&use_item), NodeKind::Uses);

        let module_item = AstItem::Module {
            name: "my_module".to_string(),
            visibility: "pub".to_string(),
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&module_item), NodeKind::Module);

        let import_item = AstItem::Import {
            module: "external".to_string(),
            items: vec![],
            alias: None,
            line: 1,
        };
        assert_eq!(NodeKind::from_ast_item(&import_item), NodeKind::Import);
    }
}
