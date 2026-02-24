#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_part2 {
    use super::*;

    #[test]
    fn test_node_kind_from_ast_item_kind_all_variants() {
        // Declarations
        assert_eq!(NodeKind::from_ast_item_kind("package"), NodeKind::Package);
        assert_eq!(NodeKind::from_ast_item_kind("import"), NodeKind::Import);
        assert_eq!(NodeKind::from_ast_item_kind("module"), NodeKind::Module);
        assert_eq!(
            NodeKind::from_ast_item_kind("namespace"),
            NodeKind::Namespace
        );

        // Types
        assert_eq!(NodeKind::from_ast_item_kind("class"), NodeKind::Class);
        assert_eq!(
            NodeKind::from_ast_item_kind("interface"),
            NodeKind::Interface
        );
        assert_eq!(NodeKind::from_ast_item_kind("trait"), NodeKind::Trait);
        assert_eq!(NodeKind::from_ast_item_kind("enum"), NodeKind::Enum);
        assert_eq!(NodeKind::from_ast_item_kind("struct"), NodeKind::Struct);
        assert_eq!(NodeKind::from_ast_item_kind("union"), NodeKind::Union);
        assert_eq!(NodeKind::from_ast_item_kind("type"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typealias"), NodeKind::Type);
        assert_eq!(NodeKind::from_ast_item_kind("typedef"), NodeKind::Type);

        // Type variations
        assert_eq!(NodeKind::from_ast_item_kind("record"), NodeKind::Record);
        assert_eq!(
            NodeKind::from_ast_item_kind("caseclass"),
            NodeKind::CaseClass
        );
        assert_eq!(
            NodeKind::from_ast_item_kind("abstracttype"),
            NodeKind::AbstractType
        );

        // Functions & methods
        assert_eq!(NodeKind::from_ast_item_kind("method"), NodeKind::Method);
        assert_eq!(NodeKind::from_ast_item_kind("function"), NodeKind::Function);
        assert_eq!(
            NodeKind::from_ast_item_kind("constructor"),
            NodeKind::Constructor
        );
        assert_eq!(NodeKind::from_ast_item_kind("lambda"), NodeKind::Lambda);
        assert_eq!(NodeKind::from_ast_item_kind("closure"), NodeKind::Closure);

        // Variables
        assert_eq!(NodeKind::from_ast_item_kind("field"), NodeKind::Field);
        assert_eq!(NodeKind::from_ast_item_kind("property"), NodeKind::Property);
        assert_eq!(
            NodeKind::from_ast_item_kind("localvariable"),
            NodeKind::LocalVariable
        );
        assert_eq!(
            NodeKind::from_ast_item_kind("parameter"),
            NodeKind::Parameter
        );
        assert_eq!(NodeKind::from_ast_item_kind("variable"), NodeKind::Variable);

        // Other elements
        assert_eq!(
            NodeKind::from_ast_item_kind("annotation"),
            NodeKind::Annotation
        );
        assert_eq!(
            NodeKind::from_ast_item_kind("decorator"),
            NodeKind::Decorator
        );
        assert_eq!(NodeKind::from_ast_item_kind("comment"), NodeKind::Comment);
        assert_eq!(NodeKind::from_ast_item_kind("macro"), NodeKind::Macro);

        // Relationships
        assert_eq!(NodeKind::from_ast_item_kind("inherits"), NodeKind::Inherits);
        assert_eq!(
            NodeKind::from_ast_item_kind("implements"),
            NodeKind::Implements
        );
        assert_eq!(NodeKind::from_ast_item_kind("uses"), NodeKind::Uses);

        // Unknown
        assert_eq!(NodeKind::from_ast_item_kind("unknown"), NodeKind::Unknown);
        assert_eq!(
            NodeKind::from_ast_item_kind("not_a_real_kind"),
            NodeKind::Unknown
        );
        assert_eq!(NodeKind::from_ast_item_kind(""), NodeKind::Unknown);
    }

    #[test]
    fn test_node_kind_as_str_all_variants() {
        assert_eq!(NodeKind::Package.as_str(), "package");
        assert_eq!(NodeKind::Import.as_str(), "import");
        assert_eq!(NodeKind::Module.as_str(), "module");
        assert_eq!(NodeKind::Namespace.as_str(), "namespace");
        assert_eq!(NodeKind::Class.as_str(), "class");
        assert_eq!(NodeKind::Interface.as_str(), "interface");
        assert_eq!(NodeKind::Trait.as_str(), "trait");
        assert_eq!(NodeKind::Enum.as_str(), "enum");
        assert_eq!(NodeKind::Struct.as_str(), "struct");
        assert_eq!(NodeKind::Union.as_str(), "union");
        assert_eq!(NodeKind::Type.as_str(), "type");
        assert_eq!(NodeKind::Record.as_str(), "record");
        assert_eq!(NodeKind::CaseClass.as_str(), "caseClass");
        assert_eq!(NodeKind::AbstractType.as_str(), "abstractType");
        assert_eq!(NodeKind::Method.as_str(), "method");
        assert_eq!(NodeKind::Function.as_str(), "function");
        assert_eq!(NodeKind::Constructor.as_str(), "constructor");
        assert_eq!(NodeKind::Lambda.as_str(), "lambda");
        assert_eq!(NodeKind::Closure.as_str(), "closure");
        assert_eq!(NodeKind::Field.as_str(), "field");
        assert_eq!(NodeKind::Property.as_str(), "property");
        assert_eq!(NodeKind::LocalVariable.as_str(), "localVariable");
        assert_eq!(NodeKind::Parameter.as_str(), "parameter");
        assert_eq!(NodeKind::Variable.as_str(), "variable");
        assert_eq!(NodeKind::Annotation.as_str(), "annotation");
        assert_eq!(NodeKind::Decorator.as_str(), "decorator");
        assert_eq!(NodeKind::Comment.as_str(), "comment");
        assert_eq!(NodeKind::Macro.as_str(), "macro");
        assert_eq!(NodeKind::Inherits.as_str(), "inherits");
        assert_eq!(NodeKind::Implements.as_str(), "implements");
        assert_eq!(NodeKind::Uses.as_str(), "uses");
        assert_eq!(NodeKind::LanguageSpecific(0).as_str(), "languageSpecific");
        assert_eq!(NodeKind::LanguageSpecific(100).as_str(), "languageSpecific");
        assert_eq!(NodeKind::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_node_kind_equality_and_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(NodeKind::Function);
        set.insert(NodeKind::Method);
        set.insert(NodeKind::LanguageSpecific(1));
        set.insert(NodeKind::LanguageSpecific(2));

        assert!(set.contains(&NodeKind::Function));
        assert!(set.contains(&NodeKind::Method));
        assert!(set.contains(&NodeKind::LanguageSpecific(1)));
        assert!(set.contains(&NodeKind::LanguageSpecific(2)));
        assert!(!set.contains(&NodeKind::Class));
        assert!(!set.contains(&NodeKind::LanguageSpecific(3)));
    }

    #[test]
    fn test_node_kind_clone_and_copy() {
        let kind = NodeKind::Function;
        let cloned = kind.clone();
        let copied = kind;

        assert_eq!(kind, cloned);
        assert_eq!(kind, copied);
    }

    #[test]
    fn test_node_kind_debug() {
        let debug_str = format!("{:?}", NodeKind::Function);
        assert_eq!(debug_str, "Function");

        let debug_specific = format!("{:?}", NodeKind::LanguageSpecific(42));
        assert_eq!(debug_specific, "LanguageSpecific(42)");
    }

    // ==========================================================================
    // PolyglotConfig tests
    // ==========================================================================

    #[test]
    fn test_polyglot_config_default_values() {
        let config = PolyglotConfig::default();

        assert_eq!(config.languages.len(), 5);
        assert!(config.languages.contains(&Language::Java));
        assert!(config.languages.contains(&Language::Kotlin));
        assert!(config.languages.contains(&Language::Scala));
        assert!(config.languages.contains(&Language::TypeScript));
        assert!(config.languages.contains(&Language::JavaScript));
        assert!(config.detect_relationships);
        assert_eq!(config.relationship_depth, 3);
        assert!(config.include_language_specific);
    }

    #[test]
    fn test_polyglot_config_custom() {
        let config = PolyglotConfig {
            languages: vec![Language::Rust, Language::Python],
            detect_relationships: false,
            relationship_depth: 5,
            include_language_specific: false,
        };

        assert_eq!(config.languages.len(), 2);
        assert!(config.languages.contains(&Language::Rust));
        assert!(config.languages.contains(&Language::Python));
        assert!(!config.detect_relationships);
        assert_eq!(config.relationship_depth, 5);
        assert!(!config.include_language_specific);
    }

    #[test]
    fn test_polyglot_config_clone() {
        let config = PolyglotConfig::default();
        let cloned = config.clone();

        assert_eq!(config.languages, cloned.languages);
        assert_eq!(config.detect_relationships, cloned.detect_relationships);
        assert_eq!(config.relationship_depth, cloned.relationship_depth);
        assert_eq!(
            config.include_language_specific,
            cloned.include_language_specific
        );
    }

    #[test]
    fn test_polyglot_config_debug() {
        let config = PolyglotConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("PolyglotConfig"));
        assert!(debug_str.contains("languages"));
        assert!(debug_str.contains("detect_relationships"));
    }

    #[test]
    fn test_polyglot_config_serialize_deserialize() {
        let config = PolyglotConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PolyglotConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.languages, deserialized.languages);
        assert_eq!(
            config.detect_relationships,
            deserialized.detect_relationships
        );
        assert_eq!(config.relationship_depth, deserialized.relationship_depth);
    }

    // ==========================================================================
    // Serialization tests
    // ==========================================================================

    #[test]
    fn test_language_serialize_deserialize() {
        let languages = vec![Language::Java, Language::TypeScript, Language::Other(42)];

        for lang in languages {
            let json = serde_json::to_string(&lang).unwrap();
            let deserialized: Language = serde_json::from_str(&json).unwrap();
            assert_eq!(lang, deserialized);
        }
    }

    #[test]
    fn test_node_kind_serialize_deserialize() {
        let kinds = vec![
            NodeKind::Function,
            NodeKind::Class,
            NodeKind::LanguageSpecific(42),
            NodeKind::Unknown,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let deserialized: NodeKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, deserialized);
        }
    }

    // ==========================================================================
    // Edge cases and boundary tests
    // ==========================================================================

    #[test]
    fn test_language_from_extension_with_dots() {
        // Extensions should not include the dot
        assert_eq!(Language::from_extension(".java"), None);
        assert_eq!(Language::from_extension("..."), None);
    }

    #[test]
    fn test_language_from_path_no_extension() {
        assert_eq!(Language::from_path(Path::new("noextension")), None);
        assert_eq!(Language::from_path(Path::new("/path/to/noextension")), None);
    }

    #[test]
    fn test_language_other_variant_equality() {
        // Different numeric values should not be equal
        assert_ne!(Language::Other(1), Language::Other(2));
        assert_eq!(Language::Other(100), Language::Other(100));
    }

    #[test]
    fn test_node_kind_language_specific_variant() {
        // Different numeric values should not be equal
        assert_ne!(NodeKind::LanguageSpecific(1), NodeKind::LanguageSpecific(2));
        assert_eq!(
            NodeKind::LanguageSpecific(100),
            NodeKind::LanguageSpecific(100)
        );
    }

    #[test]
    fn test_language_from_ast_item_kind_case_insensitive() {
        // Should handle case insensitivity
        assert_eq!(NodeKind::from_ast_item_kind("FUNCTION"), NodeKind::Function);
        assert_eq!(NodeKind::from_ast_item_kind("Function"), NodeKind::Function);
        assert_eq!(NodeKind::from_ast_item_kind("CLASS"), NodeKind::Class);
        assert_eq!(NodeKind::from_ast_item_kind("Class"), NodeKind::Class);
    }
}
