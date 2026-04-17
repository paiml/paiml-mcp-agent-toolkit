    // ==================== Language Enum Tests ====================

    #[test]
    fn test_language_all_variants() {
        // Verify all language variants exist and have correct discriminant values
        assert_eq!(Language::Rust as u8, 0);
        assert_eq!(Language::TypeScript as u8, 1);
        assert_eq!(Language::JavaScript as u8, 2);
        assert_eq!(Language::Python as u8, 3);
        assert_eq!(Language::Markdown as u8, 4);
        assert_eq!(Language::Makefile as u8, 5);
        assert_eq!(Language::Toml as u8, 6);
        assert_eq!(Language::Yaml as u8, 7);
        assert_eq!(Language::Json as u8, 8);
        assert_eq!(Language::Shell as u8, 9);
        assert_eq!(Language::C as u8, 10);
        assert_eq!(Language::Cpp as u8, 11);
        assert_eq!(Language::Cython as u8, 12);
        assert_eq!(Language::Kotlin as u8, 13);
        assert_eq!(Language::AssemblyScript as u8, 14);
        assert_eq!(Language::WebAssembly as u8, 15);
    }

    #[test]
    fn test_language_clone_eq() {
        let lang = Language::Rust;
        let cloned = lang;
        assert_eq!(lang, cloned);
        assert_ne!(Language::Rust, Language::Python);
    }

    #[test]
    fn test_language_serialization() {
        let lang = Language::TypeScript;
        let json = serde_json::to_string(&lang).expect("serialization failed");
        let deserialized: Language = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(lang, deserialized);
    }

    #[test]
    fn test_language_all_variants_serialization_roundtrip() {
        let languages = [
            Language::Rust,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Markdown,
            Language::Makefile,
            Language::Toml,
            Language::Yaml,
            Language::Json,
            Language::Shell,
            Language::C,
            Language::Cpp,
            Language::Cython,
            Language::Kotlin,
            Language::AssemblyScript,
            Language::WebAssembly,
        ];

        for lang in languages {
            let json = serde_json::to_string(&lang).expect("serialization failed");
            let deserialized: Language =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(lang, deserialized);
        }
    }

    // ==================== NodeFlags Tests ====================

    #[test]
    fn test_node_flags_new_is_empty() {
        let flags = NodeFlags::new();
        assert!(!flags.has(NodeFlags::ASYNC));
        assert!(!flags.has(NodeFlags::GENERATOR));
        assert!(!flags.has(NodeFlags::ABSTRACT));
        assert!(!flags.has(NodeFlags::STATIC));
        assert!(!flags.has(NodeFlags::CONST));
        assert!(!flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));
        assert!(!flags.has(NodeFlags::DEPRECATED));
    }

    #[test]
    fn test_node_flags_default() {
        let flags = NodeFlags::default();
        assert!(!flags.has(NodeFlags::ASYNC));
    }

    #[test]
    fn test_node_flags_set_single() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        assert!(flags.has(NodeFlags::ASYNC));
        assert!(!flags.has(NodeFlags::GENERATOR));
    }

    #[test]
    fn test_node_flags_set_multiple() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));
    }

    #[test]
    fn test_node_flags_unset() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC | NodeFlags::EXPORTED);
        flags.unset(NodeFlags::ASYNC);
        assert!(!flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
    }

    #[test]
    fn test_node_flags_unset_not_set() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        flags.unset(NodeFlags::EXPORTED); // Unset a flag that was never set
        assert!(flags.has(NodeFlags::ASYNC));
        assert!(!flags.has(NodeFlags::EXPORTED));
    }

    #[test]
    fn test_node_flags_has_any() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        // has() returns true if ANY of the specified flags are set
        assert!(flags.has(NodeFlags::ASYNC | NodeFlags::EXPORTED));
    }

    #[test]
    fn test_node_flags_all_modifier_constants() {
        // Verify all modifier flag constants have unique non-zero values
        assert_eq!(NodeFlags::ASYNC, 0b0000_0001);
        assert_eq!(NodeFlags::GENERATOR, 0b0000_0010);
        assert_eq!(NodeFlags::ABSTRACT, 0b0000_0100);
        assert_eq!(NodeFlags::STATIC, 0b0000_1000);
        assert_eq!(NodeFlags::CONST, 0b0001_0000);
        assert_eq!(NodeFlags::EXPORTED, 0b0010_0000);
        assert_eq!(NodeFlags::PRIVATE, 0b0100_0000);
        assert_eq!(NodeFlags::DEPRECATED, 0b1000_0000);
    }

    #[test]
    fn test_node_flags_all_type_constants() {
        // Node type flags
        assert_eq!(NodeFlags::FUNCTION, 0b0000_0001);
        assert_eq!(NodeFlags::STRUCT, 0b0000_0010);
        assert_eq!(NodeFlags::CLASS, 0b0000_0100);
        assert_eq!(NodeFlags::ENUM, 0b0000_1000);
        assert_eq!(NodeFlags::TRAIT, 0b0001_0000);
        assert_eq!(NodeFlags::INTERFACE, 0b0010_0000);
        assert_eq!(NodeFlags::IMPORT, 0b0100_0000);
        assert_eq!(NodeFlags::CONTROL_FLOW, 0b1000_0000);
        assert_eq!(NodeFlags::TYPE_ALIAS, 0b0000_0001);
        assert_eq!(NodeFlags::IMPL, 0b0000_0010);
    }

    #[test]
    fn test_node_flags_c_specific() {
        assert_eq!(NodeFlags::INLINE, 0b00000001);
        assert_eq!(NodeFlags::VOLATILE, 0b00000010);
        assert_eq!(NodeFlags::RESTRICT, 0b00000100);
        assert_eq!(NodeFlags::EXTERN, 0b00001000);
    }

    #[test]
    fn test_node_flags_cpp_specific() {
        assert_eq!(NodeFlags::VIRTUAL, 0b00000001);
        assert_eq!(NodeFlags::OVERRIDE, 0b00000010);
        assert_eq!(NodeFlags::FINAL, 0b00000100);
        assert_eq!(NodeFlags::MUTABLE, 0b00001000);
        assert_eq!(NodeFlags::CONSTEXPR, 0b00010000);
        assert_eq!(NodeFlags::NOEXCEPT, 0b00100000);
    }

    #[test]
    fn test_node_flags_clone_copy() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::ASYNC);
        let cloned = flags;
        let copied = flags;
        assert!(cloned.has(NodeFlags::ASYNC));
        assert!(copied.has(NodeFlags::ASYNC));
    }

    // ==================== AstKind Tests ====================

    #[test]
    fn test_ast_kind_function_variants() {
        let kinds = [
            AstKind::Function(FunctionKind::Regular),
            AstKind::Function(FunctionKind::Method),
            AstKind::Function(FunctionKind::Constructor),
            AstKind::Function(FunctionKind::Getter),
            AstKind::Function(FunctionKind::Setter),
            AstKind::Function(FunctionKind::Lambda),
            AstKind::Function(FunctionKind::Closure),
            AstKind::Function(FunctionKind::Destructor),
            AstKind::Function(FunctionKind::Operator),
        ];

        for kind in kinds {
            assert!(matches!(kind, AstKind::Function(_)));
        }
    }

    #[test]
    fn test_ast_kind_class_variants() {
        let kinds = [
            AstKind::Class(ClassKind::Regular),
            AstKind::Class(ClassKind::Abstract),
            AstKind::Class(ClassKind::Interface),
            AstKind::Class(ClassKind::Trait),
            AstKind::Class(ClassKind::Enum),
            AstKind::Class(ClassKind::Struct),
        ];

        for kind in kinds {
            assert!(matches!(kind, AstKind::Class(_)));
        }
    }

    #[test]
    fn test_ast_kind_serialization() {
        let kind = AstKind::Function(FunctionKind::Regular);
        let json = serde_json::to_string(&kind).expect("serialization failed");
        let deserialized: AstKind = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(kind, deserialized);
    }

    #[test]
    fn test_ast_kind_all_variants_serialization() {
        let kinds = [
            AstKind::Function(FunctionKind::Regular),
            AstKind::Class(ClassKind::Struct),
            AstKind::Variable(VarKind::Let),
            AstKind::Import(ImportKind::Named),
            AstKind::Expression(ExprKind::Call),
            AstKind::Statement(StmtKind::If),
            AstKind::Type(TypeKind::Primitive),
            AstKind::Module(ModuleKind::File),
            AstKind::Macro(MacroKind::ObjectLike),
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("serialization failed");
            let deserialized: AstKind =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(kind, deserialized);
        }
    }

    // ==================== FunctionKind Tests ====================

    #[test]
    fn test_function_kind_all_variants() {
        let variants = [
            FunctionKind::Regular,
            FunctionKind::Method,
            FunctionKind::Constructor,
            FunctionKind::Getter,
            FunctionKind::Setter,
            FunctionKind::Lambda,
            FunctionKind::Closure,
            FunctionKind::Destructor,
            FunctionKind::Operator,
        ];
        assert_eq!(variants.len(), 9);
    }

    #[test]
    fn test_function_kind_eq() {
        assert_eq!(FunctionKind::Regular, FunctionKind::Regular);
        assert_ne!(FunctionKind::Regular, FunctionKind::Method);
    }

    // ==================== VarKind Tests ====================

    #[test]
    fn test_var_kind_all_variants() {
        let variants = [
            VarKind::Let,
            VarKind::Const,
            VarKind::Static,
            VarKind::Field,
            VarKind::Parameter,
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_var_kind_serialization() {
        for kind in [
            VarKind::Let,
            VarKind::Const,
            VarKind::Static,
            VarKind::Field,
            VarKind::Parameter,
        ] {
            let json = serde_json::to_string(&kind).expect("serialization failed");
            let deserialized: VarKind =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(kind, deserialized);
        }
    }

    // ==================== ImportKind Tests ====================

    #[test]
    fn test_import_kind_all_variants() {
        let variants = [
            ImportKind::Module,
            ImportKind::Named,
            ImportKind::Default,
            ImportKind::Namespace,
            ImportKind::Dynamic,
        ];
        assert_eq!(variants.len(), 5);
    }

    // ==================== ExprKind Tests ====================

    #[test]
    fn test_expr_kind_all_variants() {
        let variants = [
            ExprKind::Call,
            ExprKind::Member,
            ExprKind::Binary,
            ExprKind::Unary,
            ExprKind::Literal,
            ExprKind::Identifier,
            ExprKind::Array,
            ExprKind::Object,
            ExprKind::New,
            ExprKind::Delete,
            ExprKind::Lambda,
            ExprKind::Conditional,
            ExprKind::This,
        ];
        assert_eq!(variants.len(), 13);
    }

    // ==================== StmtKind Tests ====================

    #[test]
    fn test_stmt_kind_all_variants() {
        let variants = [
            StmtKind::Block,
            StmtKind::If,
            StmtKind::For,
            StmtKind::While,
            StmtKind::Return,
            StmtKind::Throw,
            StmtKind::Try,
            StmtKind::Switch,
            StmtKind::Goto,
            StmtKind::Label,
            StmtKind::DoWhile,
            StmtKind::ForEach,
            StmtKind::Catch,
            StmtKind::Break,
            StmtKind::Continue,
            StmtKind::Case,
        ];
        assert_eq!(variants.len(), 16);
    }

    // ==================== TypeKind Tests ====================

    #[test]
    fn test_type_kind_all_variants() {
        let variants = [
            TypeKind::Primitive,
            TypeKind::Array,
            TypeKind::Tuple,
            TypeKind::Union,
            TypeKind::Intersection,
            TypeKind::Generic,
            TypeKind::Function,
            TypeKind::Object,
            TypeKind::Pointer,
            TypeKind::Struct,
            TypeKind::Enum,
            TypeKind::Typedef,
            TypeKind::Class,
            TypeKind::Template,
            TypeKind::Namespace,
            TypeKind::Alias,
            TypeKind::Interface,
            TypeKind::Module,
            TypeKind::Annotation,
            TypeKind::Mapped,
            TypeKind::Conditional,
        ];
        assert_eq!(variants.len(), 21);
    }

    // ==================== ModuleKind Tests ====================

    #[test]
    fn test_module_kind_all_variants() {
        let variants = [ModuleKind::File, ModuleKind::Namespace, ModuleKind::Package];
        assert_eq!(variants.len(), 3);
    }

    // ==================== MacroKind Tests ====================

    #[test]
    fn test_macro_kind_all_variants() {
        let variants = [
            MacroKind::ObjectLike,
            MacroKind::FunctionLike,
            MacroKind::Variadic,
            MacroKind::Include,
            MacroKind::Conditional,
            MacroKind::Export,
            MacroKind::Decorator,
        ];
        assert_eq!(variants.len(), 7);
    }

    // ==================== ConfidenceLevel Tests ====================

    #[test]
    fn test_confidence_level_ordering() {
        assert!(ConfidenceLevel::Low < ConfidenceLevel::Medium);
        assert!(ConfidenceLevel::Medium < ConfidenceLevel::High);
        assert!(ConfidenceLevel::Low < ConfidenceLevel::High);
    }

    #[test]
    fn test_confidence_level_values() {
        assert_eq!(ConfidenceLevel::Low as u8, 1);
        assert_eq!(ConfidenceLevel::Medium as u8, 2);
        assert_eq!(ConfidenceLevel::High as u8, 3);
    }

    #[test]
    fn test_confidence_level_serialization() {
        for level in [
            ConfidenceLevel::Low,
            ConfidenceLevel::Medium,
            ConfidenceLevel::High,
        ] {
            let json = serde_json::to_string(&level).expect("serialization failed");
            let deserialized: ConfidenceLevel =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(level, deserialized);
        }
    }

