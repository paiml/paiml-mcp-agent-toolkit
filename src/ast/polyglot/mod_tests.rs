#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("java"), Some(Language::Java));
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("scala"), Some(Language::Scala));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_language_from_path() {
        let java_path = Path::new("/path/to/MyClass.java");
        let scala_path = Path::new("/path/to/MyClass.scala");
        let unknown_path = Path::new("/path/to/file.unknown");

        assert_eq!(Language::from_path(java_path), Some(Language::Java));
        assert_eq!(Language::from_path(scala_path), Some(Language::Scala));
        assert_eq!(Language::from_path(unknown_path), None);
    }

    #[test]
    fn test_node_kind_from_ast_item_kind() {
        assert_eq!(NodeKind::from_ast_item_kind("class"), NodeKind::Class);
        assert_eq!(NodeKind::from_ast_item_kind("method"), NodeKind::Method);
        assert_eq!(NodeKind::from_ast_item_kind("unknown"), NodeKind::Unknown);
    }

    #[test]
    fn test_polyglot_config_default() {
        let config = PolyglotConfig::default();

        assert!(config.languages.contains(&Language::Java));
        assert!(config.languages.contains(&Language::Scala));
        assert!(config.languages.contains(&Language::TypeScript));
        assert!(config.detect_relationships);
        assert_eq!(config.relationship_depth, 3);
    }

    #[test]
    fn test_node_kind_comprehensive_string_conversion() {
        // Test conversion from strings to NodeKind
        assert_eq!(NodeKind::from_ast_item_kind("function"), NodeKind::Function);
        assert_eq!(NodeKind::from_ast_item_kind("struct"), NodeKind::Struct);
        assert_eq!(NodeKind::from_ast_item_kind("enum"), NodeKind::Enum);
        assert_eq!(NodeKind::from_ast_item_kind("trait"), NodeKind::Trait);
        assert_eq!(
            NodeKind::from_ast_item_kind("implements"),
            NodeKind::Implements
        );
        assert_eq!(NodeKind::from_ast_item_kind("import"), NodeKind::Import);
        assert_eq!(NodeKind::from_ast_item_kind("module"), NodeKind::Module);
        assert_eq!(NodeKind::from_ast_item_kind("type"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typedef"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typealias"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("macro"), NodeKind::Macro);
        assert_eq!(NodeKind::from_ast_item_kind("variable"), NodeKind::Variable);

        // Test NodeKind to string conversion
        assert_eq!(NodeKind::Function.as_str(), "function");
        assert_eq!(NodeKind::Struct.as_str(), "struct");
        assert_eq!(NodeKind::Enum.as_str(), "enum");
        assert_eq!(NodeKind::Trait.as_str(), "trait");
        assert_eq!(NodeKind::Implements.as_str(), "implements");
        assert_eq!(NodeKind::Import.as_str(), "import");
        assert_eq!(NodeKind::Module.as_str(), "module");
        assert_eq!(NodeKind::Type.as_str(), "type");
        assert_eq!(NodeKind::Macro.as_str(), "macro");
        assert_eq!(NodeKind::Variable.as_str(), "variable");
    }
}
